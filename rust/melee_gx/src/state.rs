//! The decoded GX state and the command processor that builds it: a port of Aurora's
//! `lib/gx/gx.hpp` (`GXState` and its parts), `lib/gx/regs.cpp` (BP / CP / XF register decode) and
//! `lib/gx/command_processor.cpp` (`process`, `handle_aurora`, the draw opcodes).
//!
//! The GX API ([`crate::api`]) never touches this state directly: like Aurora (and the hardware) it
//! writes register commands into a FIFO, and [`GxState::process`] decodes them. Display lists are
//! the same byte stream. Draw commands are not rendered here; they become [`DrawCmd`]s (vertex bytes
//! plus the state snapshot a renderer needs), collected in [`GxState::draws`].
//!
//! Differences from Aurora, all outside what a draw sees: no frame interpolation (`interp::`), no
//! draw merging / uniform caching / dirty tracking (every draw recomputes its config), no EFB copies,
//! offscreen targets, palettes (`GX_AURORA_LOAD_PALETTE`), surface programs or motion capture.
//! Array and texture pointers in the FIFO are handles into [`Resources`].
use crate::consts::*;
use crate::resources::{GxData, Resources};
use crate::texture;

pub const MAX_TEXTURES: usize = 8;
pub const MAX_TLUTS: usize = 20;
pub const MAX_TEV_STAGES: usize = 16;
pub const MAX_COLOR_CHANNELS: usize = 4;
pub const MAX_TEV_REGS: usize = 4;
pub const MAX_KCOLORS: usize = 4;
pub const MAX_TEX_MTX: usize = 10;
pub const MAX_PT_TEX_MTX: usize = 20;
pub const MAX_TEX_COORD: usize = 8;
pub const MAX_VTX_ATTR: usize = GX_VA_MAX_ATTR as usize;
pub const MAX_TEV_SWAP: usize = 4;
pub const MAX_IND_STAGES: usize = 4;
pub const MAX_IND_TEX_MTXS: usize = 3;
pub const MAX_VTX_FMT: usize = 8;
pub const MAX_PN_MTX: usize = 10;
pub const MAX_INDEX_ATTR: usize = 12;
pub const MAX_LIGHTS: usize = 8;
pub const XF_REG_COUNT: usize = 0x58;
pub const USE_REVERSED_Z: bool = true;

pub type Vec4 = [f32; 4];
/// GX row-major 3x4 matrix (rows), as Aurora's `Mat3x4<float>` stores it.
pub type Mat3x4 = [[f32; 4]; 3];
pub type Mat4x4 = [[f32; 4]; 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TevPass {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TevOp {
    pub op: u32,
    pub bias: u32,
    pub scale: u32,
    pub out_reg: u32,
    pub clamp: bool,
}
impl Default for TevOp {
    fn default() -> Self {
        Self { op: GX_TEV_ADD, bias: GX_TB_ZERO, scale: GX_CS_SCALE_1, out_reg: GX_TEVPREV, clamp: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TevStage {
    pub color_pass: TevPass,
    pub alpha_pass: TevPass,
    pub color_op: TevOp,
    pub alpha_op: TevOp,
    pub kc_sel: u32,
    pub ka_sel: u32,
    pub tex_coord_id: u32,
    pub tex_map_id: u32,
    pub channel_id: u32,
    pub tev_swap_ras: u32,
    pub tev_swap_tex: u32,
    pub ind_tex_stage: u32,
    pub ind_tex_format: u32,
    pub ind_tex_bias_sel: u32,
    pub ind_tex_alpha_sel: u32,
    pub ind_tex_mtx_id: u32,
    pub ind_tex_wrap_s: u32,
    pub ind_tex_wrap_t: u32,
    pub ind_tex_use_orig_lod: bool,
    pub ind_tex_add_prev: bool,
}
impl Default for TevStage {
    fn default() -> Self {
        Self {
            color_pass: TevPass { a: GX_CC_ZERO, b: GX_CC_ZERO, c: GX_CC_ZERO, d: GX_CC_ZERO },
            alpha_pass: TevPass { a: GX_CA_ZERO, b: GX_CA_ZERO, c: GX_CA_ZERO, d: GX_CA_ZERO },
            color_op: TevOp::default(),
            alpha_op: TevOp::default(),
            kc_sel: GX_TEV_KCSEL_1,
            ka_sel: GX_TEV_KASEL_1,
            tex_coord_id: GX_TEXCOORD_NULL,
            tex_map_id: GX_TEXMAP_NULL,
            channel_id: GX_COLOR_NULL,
            tev_swap_ras: GX_TEV_SWAP0,
            tev_swap_tex: GX_TEV_SWAP0,
            ind_tex_stage: GX_INDTEXSTAGE0,
            ind_tex_format: GX_ITF_8,
            ind_tex_bias_sel: GX_ITB_NONE,
            ind_tex_alpha_sel: GX_ITBA_OFF,
            ind_tex_mtx_id: GX_ITM_OFF,
            ind_tex_wrap_s: GX_ITW_OFF,
            ind_tex_wrap_t: GX_ITW_OFF,
            ind_tex_use_orig_lod: false,
            ind_tex_add_prev: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IndStage {
    pub tex_coord_id: u32,
    pub tex_map_id: u32,
    pub scale_s: u32,
    pub scale_t: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorChannelConfig {
    pub mat_src: u32,
    pub amb_src: u32,
    pub diff_fn: u32,
    pub attn_fn: u32,
    pub lighting_enabled: bool,
}
impl Default for ColorChannelConfig {
    fn default() -> Self {
        Self { mat_src: GX_SRC_REG, amb_src: GX_SRC_REG, diff_fn: GX_DF_NONE, attn_fn: GX_AF_NONE, lighting_enabled: false }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorChannelState {
    pub mat_color: Vec4,
    pub amb_color: Vec4,
    pub light_mask: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TcgConfig {
    pub ty: u32,
    pub src: u32,
    pub mtx: u32,
    pub post_mtx: u32,
    pub normalize: bool,
    pub emboss_src: u32,
}
impl Default for TcgConfig {
    fn default() -> Self {
        Self { ty: GX_TG_MTX2x4, src: GX_MAX_TEXGENSRC, mtx: GX_IDENTITY, post_mtx: GX_PTIDENTITY, normalize: false, emboss_src: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogState {
    pub ty: u32,
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub color: Vec4,
    pub range_k: [u16; 10],
    pub range_center: i16,
    pub range_enabled: bool,
    pub fog0_raw: u32,
    pub fog1_raw: u32,
    pub fog2_raw: u32,
}
impl Default for FogState {
    fn default() -> Self {
        Self {
            ty: GX_FOG_NONE,
            a: 0.0,
            b: 0.5,
            c: 0.0,
            color: [0.0; 4],
            range_k: [0; 10],
            range_center: 0,
            range_enabled: false,
            fog0_raw: 0,
            fog1_raw: 0,
            fog2_raw: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TevSwap {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}
impl Default for TevSwap {
    fn default() -> Self {
        Self { red: GX_CH_RED, green: GX_CH_GREEN, blue: GX_CH_BLUE, alpha: GX_CH_ALPHA }
    }
}
impl TevSwap {
    pub const fn new(red: u32, green: u32, blue: u32, alpha: u32) -> Self {
        Self { red, green, blue, alpha }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlphaCompare {
    pub comp0: u32,
    pub ref0: u32,
    pub op: u32,
    pub comp1: u32,
    pub ref1: u32,
}
impl Default for AlphaCompare {
    fn default() -> Self {
        Self { comp0: GX_ALWAYS, ref0: 0, op: GX_AOP_AND, comp1: GX_ALWAYS, ref1: 0 }
    }
}
impl AlphaCompare {
    /// Aurora `explicit operator bool`.
    pub fn enabled(&self) -> bool {
        self.comp0 != GX_ALWAYS || self.comp1 != GX_ALWAYS
    }
}

/// Indirect matrix: `mtx` is Aurora's `Mat3x2` (three columns m0..m2 of (x, y)).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IndTexMtxInfo {
    pub mtx: [[f32; 2]; 3],
    pub scale_exp: i8,
    pub adj_scale_raw: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TexCoordScale {
    pub scale_s: u16,
    pub scale_t: u16,
    pub bias_s: bool,
    pub bias_t: bool,
    pub cyl_wrap_s: bool,
    pub cyl_wrap_t: bool,
    pub line_offset: bool,
    pub point_offset: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VtxAttrFmt {
    pub cnt: u32,
    pub ty: u32,
    pub frac: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PnMtx {
    pub pos: Mat3x4,
    pub nrm: Mat3x4,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Light {
    pub pos: Vec4,
    pub dir: Vec4,
    pub color: Vec4,
    pub cos_att: Vec4,
    pub dist_att: Vec4,
}

/// A vertex array (`GXSetArray`); `data` is the game's bytes (big-endian unless `le`).
#[derive(Clone, Debug)]
pub struct AttrArray {
    pub data: Option<GxData>,
    pub size: u32,
    pub stride: u8,
    pub le: bool,
}
impl Default for AttrArray {
    fn default() -> Self {
        Self { data: None, size: 0, stride: 0, le: true }
    }
}

/// Aurora's `GXTexObj_` (the PC texture object). GC layout metadata: see [`crate::api::GxTexObj`].
#[derive(Clone, Debug)]
pub struct TexObj {
    pub mode0: u32,
    pub mode1: u32,
    pub image0: u32,
    pub image3: u32,
    pub data: Option<GxData>,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub tlut: u32,
    pub tex_obj_id: u32,
    pub tex_data_version: u32,
    pub flags: u8,
}
pub const INVALID_TEXTURE_FORMAT: u32 = u32::MAX;
impl Default for TexObj {
    fn default() -> Self {
        Self {
            mode0: 0,
            mode1: 0,
            image0: u32::MAX,
            image3: 0,
            data: None,
            width: 0,
            height: 0,
            format: INVALID_TEXTURE_FORMAT,
            tlut: GX_TLUT0,
            tex_obj_id: 0,
            tex_data_version: 0,
            flags: 0,
        }
    }
}
#[inline]
pub(crate) fn get_bits(reg: u32, size: u32, shift: u32) -> u32 {
    (reg >> shift) & ((1u32 << size) - 1)
}
impl TexObj {
    pub fn width(&self) -> u32 {
        if self.width != 0 {
            self.width
        } else if self.image0 == u32::MAX {
            0
        } else {
            get_bits(self.image0, 10, 0) + 1
        }
    }
    pub fn height(&self) -> u32 {
        if self.height != 0 {
            self.height
        } else if self.image0 == u32::MAX {
            0
        } else {
            get_bits(self.image0, 10, 10) + 1
        }
    }
    pub fn raw_format(&self) -> u32 {
        get_bits(self.image0, 4, 20)
    }
    pub fn format(&self) -> u32 {
        if self.format != INVALID_TEXTURE_FORMAT {
            self.format
        } else {
            self.raw_format()
        }
    }
    pub fn wrap_s(&self) -> u32 {
        get_bits(self.mode0, 2, 0)
    }
    pub fn wrap_t(&self) -> u32 {
        get_bits(self.mode0, 2, 2)
    }
    pub fn min_filter(&self) -> u32 {
        const HW_TO_GX: [u32; 8] =
            [GX_NEAR, GX_NEAR_MIP_NEAR, GX_LIN_MIP_NEAR, GX_NEAR, GX_LINEAR, GX_NEAR_MIP_LIN, GX_LIN_MIP_LIN, GX_NEAR];
        HW_TO_GX[get_bits(self.mode0, 3, 5) as usize]
    }
    pub fn mag_filter(&self) -> u32 {
        if get_bits(self.mode0, 1, 4) != 0 {
            GX_LINEAR
        } else {
            GX_NEAR
        }
    }
    pub fn has_mips(&self) -> bool {
        self.flags & 1 != 0
    }
    pub fn mip_count(&self) -> u32 {
        if self.has_mips() {
            ((self.max_lod() as u32) + 1).max(1)
        } else {
            1
        }
    }
    pub fn lod_bias(&self) -> f32 {
        (get_bits(self.mode0, 8, 9) as u8 as i8) as f32 / 32.0
    }
    pub fn max_aniso(&self) -> u32 {
        get_bits(self.mode0, 2, 19)
    }
    pub fn min_lod(&self) -> f32 {
        get_bits(self.mode1, 8, 0) as f32 / 16.0
    }
    pub fn max_lod(&self) -> f32 {
        get_bits(self.mode1, 8, 8) as f32 / 16.0
    }
}

/// Aurora's `GXTlutObj_`.
#[derive(Clone, Debug, Default)]
pub struct TlutObj {
    pub tlut: u32,
    pub load_tlut0: u32,
    pub num_entries: u16,
    pub data: Option<GxData>,
    pub format: u32,
    pub tlut_obj_id: u32,
    pub tlut_data_version: u32,
    pub flags: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub znear: f32,
    pub zfar: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Aurora `GXState` (the decoded part; caches and shadow registers below).
#[derive(Clone, Debug)]
pub struct GxState {
    pub pn_mtx: [PnMtx; MAX_PN_MTX],
    pub current_pn_mtx: u32,
    pub proj: Mat4x4,
    pub proj_type: u32,
    pub fog: FogState,
    pub cull_mode: u32,
    pub line_width: u8,
    pub point_size: u8,
    pub line_tex_offset: u32,
    pub point_tex_offset: u32,
    pub line_half_aspect: bool,
    pub blend_mode: u32,
    pub blend_fac_src: u32,
    pub blend_fac_dst: u32,
    pub blend_op: u32,
    pub depth_func: u32,
    pub clear_color: Vec4,
    pub clear_depth: u32,
    pub pixel_fmt: u32,
    pub z_fmt: u32,
    pub z_comp_loc_before_tex: bool,
    /// u8 alpha, or u32::MAX when disabled.
    pub dst_alpha: u32,
    pub alpha_compare: AlphaCompare,
    pub color_regs: [Vec4; MAX_TEV_REGS],
    pub kcolors: [Vec4; MAX_KCOLORS],
    pub color_channel_config: [ColorChannelConfig; MAX_COLOR_CHANNELS],
    pub color_channel_state: [ColorChannelState; MAX_COLOR_CHANNELS],
    pub lights: [Light; MAX_LIGHTS],
    pub tev_stages: [TevStage; MAX_TEV_STAGES],
    pub loaded_textures: [TexObj; MAX_TEXTURES],
    pub loaded_tluts: [TlutObj; MAX_TLUTS],
    pub tex_mtxs: [Mat3x4; MAX_TEX_MTX],
    pub pt_tex_mtxs: [Mat3x4; MAX_PT_TEX_MTX],
    pub tcgs: [TcgConfig; MAX_TEX_COORD],
    pub tex_coord_scales: [TexCoordScale; MAX_TEX_COORD],
    pub vtx_desc: [u32; MAX_VTX_ATTR],
    pub vtx_fmts: [[VtxAttrFmt; MAX_VTX_ATTR]; MAX_VTX_FMT],
    pub tev_swap_table: [TevSwap; MAX_TEV_SWAP],
    pub ind_stages: [IndStage; MAX_IND_STAGES],
    pub ind_tex_mtxs: [IndTexMtxInfo; MAX_IND_TEX_MTXS],
    pub arrays: [AttrArray; MAX_VTX_ATTR],
    pub depth_compare: bool,
    pub depth_update: bool,
    pub color_update: bool,
    pub alpha_update: bool,
    pub num_chans: u8,
    pub num_ind_stages: u8,
    pub num_tev_stages: u8,
    pub num_tex_gens: u8,
    pub front_offset: f32,
    pub front_scale: f32,
    pub back_offset: f32,
    pub back_scale: f32,
    pub clamp: f32,
    /// AURORA_VIEWPORT_FIT (true) maps logical GX coordinates onto the render target.
    pub viewport_fit: bool,
    pub logical_viewport: Viewport,
    pub render_viewport: Viewport,
    pub logical_scissor: ClipRect,
    pub render_scissor: ClipRect,
    /// Logical framebuffer (VI) size and the render target size, for viewport mapping.
    pub logical_fb_size: (u32, u32),
    pub target_size: (u32, u32),

    pub bp_reg_cache: [u32; 0x100],
    pub bp_reg_valid: [bool; 0x100],
    pub cp_reg_cache: [u32; 0x100],
    pub cp_reg_valid: [bool; 0x100],
    pub xf_reg_cache: [u32; XF_REG_COUNT],
    pub xf_reg_valid: [bool; XF_REG_COUNT],

    /// Draws decoded since the last [`GxState::take_draws`].
    pub draws: Vec<crate::draw::DrawCmd>,
    /// Warnings worth surfacing (unknown subcommands, unsupported features).
    pub warnings: Vec<String>,
}

impl Default for GxState {
    fn default() -> Self {
        let vp = Viewport { left: 0.0, top: 0.0, width: 640.0, height: 480.0, znear: 0.0, zfar: 1.0 };
        let sc = ClipRect { x: 0, y: 0, width: 640, height: 480 };
        let mut bp = [0u32; 0x100];
        bp[0xFE] = 0x00FF_FFFF;
        Self {
            pn_mtx: [PnMtx::default(); MAX_PN_MTX],
            current_pn_mtx: 0,
            proj: [[0.0; 4]; 4],
            proj_type: GX_PERSPECTIVE,
            fog: FogState::default(),
            cull_mode: GX_CULL_BACK,
            line_width: 0,
            point_size: 0,
            line_tex_offset: GX_TO_ZERO,
            point_tex_offset: GX_TO_ZERO,
            line_half_aspect: false,
            blend_mode: GX_BM_NONE,
            blend_fac_src: GX_BL_SRCALPHA,
            blend_fac_dst: GX_BL_INVSRCALPHA,
            blend_op: GX_LO_CLEAR,
            depth_func: GX_LEQUAL,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            clear_depth: 0xFF_FFFF,
            pixel_fmt: GX_PF_RGB8_Z24,
            z_fmt: GX_ZC_LINEAR,
            z_comp_loc_before_tex: false,
            dst_alpha: 0,
            alpha_compare: AlphaCompare::default(),
            color_regs: [[0.0; 4]; MAX_TEV_REGS],
            kcolors: [[0.0; 4]; MAX_KCOLORS],
            color_channel_config: [ColorChannelConfig::default(); MAX_COLOR_CHANNELS],
            color_channel_state: [ColorChannelState::default(); MAX_COLOR_CHANNELS],
            lights: [Light::default(); MAX_LIGHTS],
            tev_stages: [TevStage::default(); MAX_TEV_STAGES],
            loaded_textures: Default::default(),
            loaded_tluts: Default::default(),
            tex_mtxs: [[[0.0; 4]; 3]; MAX_TEX_MTX],
            pt_tex_mtxs: [[[0.0; 4]; 3]; MAX_PT_TEX_MTX],
            tcgs: [TcgConfig::default(); MAX_TEX_COORD],
            tex_coord_scales: [TexCoordScale::default(); MAX_TEX_COORD],
            vtx_desc: [GX_NONE; MAX_VTX_ATTR],
            vtx_fmts: [[VtxAttrFmt::default(); MAX_VTX_ATTR]; MAX_VTX_FMT],
            tev_swap_table: [
                TevSwap::default(),
                TevSwap::new(GX_CH_RED, GX_CH_RED, GX_CH_RED, GX_CH_ALPHA),
                TevSwap::new(GX_CH_GREEN, GX_CH_GREEN, GX_CH_GREEN, GX_CH_ALPHA),
                TevSwap::new(GX_CH_BLUE, GX_CH_BLUE, GX_CH_BLUE, GX_CH_ALPHA),
            ],
            ind_stages: [IndStage::default(); MAX_IND_STAGES],
            ind_tex_mtxs: [IndTexMtxInfo::default(); MAX_IND_TEX_MTXS],
            arrays: Default::default(),
            depth_compare: true,
            depth_update: true,
            color_update: true,
            alpha_update: true,
            num_chans: 0,
            num_ind_stages: 0,
            num_tev_stages: 0,
            num_tex_gens: 0,
            front_offset: 0.0,
            front_scale: 0.0,
            back_offset: 0.0,
            back_scale: 0.0,
            clamp: 0.0,
            viewport_fit: true,
            logical_viewport: vp,
            render_viewport: vp,
            logical_scissor: sc,
            render_scissor: sc,
            logical_fb_size: (640, 480),
            target_size: (640, 480),
            bp_reg_cache: bp,
            bp_reg_valid: [false; 0x100],
            cp_reg_cache: [0; 0x100],
            cp_reg_valid: [false; 0x100],
            xf_reg_cache: [0; XF_REG_COUNT],
            xf_reg_valid: [false; XF_REG_COUNT],
            draws: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[inline]
pub(crate) fn reg_get(reg: u32, size: u32, shift: u32) -> u32 {
    (reg >> shift) & ((1u32 << size) - 1)
}

fn unpack_color(packed: u32) -> Vec4 {
    [
        ((packed >> 24) & 0xFF) as f32 / 255.0,
        ((packed >> 16) & 0xFF) as f32 / 255.0,
        ((packed >> 8) & 0xFF) as f32 / 255.0,
        (packed & 0xFF) as f32 / 255.0,
    ]
}

fn sext11(v: u32) -> i32 {
    if v & 0x400 != 0 {
        (v | !0x7FF) as i32
    } else {
        v as i32
    }
}

fn decode_fog_float(v: u32) -> f32 {
    f32::from_bits(reg_get(v, 1, 19) << 31 | reg_get(v, 8, 11) << 23 | reg_get(v, 11, 0) << 12)
}

/// C `ldexpf(x, e)` for the fog decode: exact scaling by a power of two (Aurora uses std::ldexp on a
/// float, which is exact barring overflow/underflow; the fog exponents here are small).
fn ldexp(x: f32, e: i32) -> f32 {
    (x as f64 * 2f64.powi(e)) as f32
}

/// Big-endian cursor over a command stream (Aurora `ByteReader`).
pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    pub(crate) fn empty(&self) -> bool {
        self.pos >= self.data.len()
    }
    pub(crate) fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
    pub(crate) fn take(&mut self, n: usize) -> &'a [u8] {
        let end = (self.pos + n).min(self.data.len());
        let s = &self.data[self.pos..end];
        self.pos += n;
        s
    }
    pub(crate) fn u8(&mut self) -> u8 {
        let b = self.data.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        b
    }
    pub(crate) fn u16(&mut self) -> u16 {
        let s = self.take(2);
        u16::from_be_bytes([s.first().copied().unwrap_or(0), s.get(1).copied().unwrap_or(0)])
    }
    pub(crate) fn u32(&mut self) -> u32 {
        let s = self.take(4);
        let mut b = [0u8; 4];
        b[..s.len()].copy_from_slice(s);
        u32::from_be_bytes(b)
    }
    pub(crate) fn u64(&mut self) -> u64 {
        ((self.u32() as u64) << 32) | self.u32() as u64
    }
    pub(crate) fn f32(&mut self) -> f32 {
        f32::from_bits(self.u32())
    }
}

fn be32(d: &[u8], i: usize) -> u32 {
    u32::from_be_bytes([d[i * 4], d[i * 4 + 1], d[i * 4 + 2], d[i * 4 + 3]])
}

impl GxState {
    /// Decode a command stream (FIFO contents or a display list); draws go to [`GxState::draws`].
    pub fn process(&mut self, data: &[u8], res: &Resources) {
        let mut r = Reader::new(data);
        while !r.empty() {
            let cmd = r.u8();
            let opcode = cmd & GX_OPCODE_MASK;
            match opcode {
                GX_NOP => continue,
                op if op == (GX_LOAD_BP_REG & GX_OPCODE_MASK) => {
                    let value = r.u32();
                    // A GX_BP_REG_DRAWDONE (0x45) write only signals the CPU in Aurora; decoding goes on.
                    self.handle_bp(value);
                }
                GX_LOAD_CP_REG => {
                    let addr = r.u8();
                    let v = r.u32();
                    self.handle_cp(addr, v);
                }
                GX_LOAD_XF_REG => {
                    let header = r.u32();
                    let count = ((header >> 16) & 0xFFFF) as usize + 1;
                    let addr = (header & 0xFFFF) as u16;
                    let d = r.take(count * 4);
                    self.handle_xf(addr, d);
                }
                GX_LOAD_INDX_A | GX_LOAD_INDX_B | GX_LOAD_INDX_C | GX_LOAD_INDX_D => {
                    let array_type = GX_POS_MTX_ARRAY as usize + ((opcode - GX_LOAD_INDX_A) / 8) as usize;
                    let src_idx = r.u16();
                    let addr_len = r.u16();
                    let len = (addr_len >> 12) as u32 + 1;
                    let dst = (addr_len & 0x0FFF) as u32;
                    let array = &self.arrays[array_type];
                    let off = src_idx as usize * array.stride as usize;
                    let size = len as usize * 4;
                    let le = array.le;
                    match &array.data {
                        Some(d) if off + size <= d.len() && off as u64 + size as u64 <= array.size as u64 => {
                            let src = d[off..off + size].to_vec();
                            self.copy_xf_data(dst, &src, len, le);
                        }
                        _ => self.warnings.push(format!("indexed XF load outside array {array_type}")),
                    }
                }
                GX_CMD_CALL_DL => {
                    self.warnings.push("Ignoring nested GX_CMD_CALL_DL".into());
                    r.take(8);
                }
                GX_CMD_INVL_VC => {}
                GX_AURORA => self.handle_aurora(&mut r, res),
                _ => {
                    if cmd >= 0x80 {
                        let fmt = (cmd & GX_VAT_MASK) as u32;
                        let prim = (cmd & GX_OPCODE_MASK) as u32;
                        let n = r.u16();
                        self.draw_prim(prim, fmt, n, &mut r, None);
                    } else {
                        panic!("command_processor: unknown opcode 0x{cmd:02X} at pos {}", r.pos - 1);
                    }
                }
            }
        }
    }

    fn draw_prim(&mut self, prim: u32, fmt: u32, vtx_count: u16, r: &mut Reader, indices: Option<Vec<u16>>) {
        let vtx_size = crate::draw::calc_vtx_size(self, fmt);
        let total = vtx_count as usize * vtx_size as usize;
        if total > r.remaining() {
            panic!("draw vertex data overrun: need {total} bytes, have {}", r.remaining());
        }
        let verts = r.take(total).to_vec();
        if let Some(d) = crate::draw::build_draw(self, prim, fmt, vtx_count, verts, indices) {
            self.draws.push(d);
        }
    }

    fn handle_aurora(&mut self, r: &mut Reader, res: &Resources) {
        const LOAD_VIEWPORT_RENDER: u16 = 0x0001;
        const LOAD_SCISSOR_RENDER: u16 = 0x0002;
        const LOAD_PROJECTION_FULL: u16 = 0x0003;
        const LOAD_ARRAYBASE: u16 = 0x0010;
        const LOAD_TEXOBJ: u16 = 0x0030;
        const LOAD_TLUT: u16 = 0x0031;
        const DRAW_SIZED: u16 = 0x0040;
        const DRAW_INDEXED: u16 = 0x0041;
        const SET_POLYGON_OFFSET: u16 = 0x1000;
        let sub = r.u16();
        match sub {
            LOAD_VIEWPORT_RENDER => {
                let v = Viewport { left: r.f32(), top: r.f32(), width: r.f32(), height: r.f32(), znear: r.f32(), zfar: r.f32() };
                self.render_viewport = v;
            }
            LOAD_SCISSOR_RENDER => {
                self.render_scissor =
                    ClipRect { x: r.u32() as i32, y: r.u32() as i32, width: r.u32() as i32, height: r.u32() as i32 };
            }
            LOAD_PROJECTION_FULL => {
                for row in 0..4 {
                    for c in 0..4 {
                        self.proj[row][c] = r.f32();
                    }
                }
                for reg in 0x20..=0x26 {
                    self.xf_reg_valid[reg] = false;
                }
            }
            s if (LOAD_ARRAYBASE..=LOAD_ARRAYBASE | 0xF).contains(&s) => {
                let attr = (s - LOAD_ARRAYBASE) as usize + GX_VA_POS as usize;
                let handle = r.u64();
                let size = r.u32();
                let le = r.u8() == 1;
                let a = &mut self.arrays[attr];
                a.data = res.get(handle);
                a.size = size;
                a.le = le;
            }
            LOAD_TEXOBJ => {
                let id = r.u8() as usize;
                assert!(id < MAX_TEXTURES, "invalid texture map id {id}");
                let handle = r.u64();
                let slot = &mut self.loaded_textures[id];
                slot.data = res.get(handle);
                slot.width = r.u32();
                slot.height = r.u32();
                slot.format = r.u32();
                slot.tlut = r.u32();
                let mut flags = slot.flags & !0x80;
                if r.u8() != 0 {
                    flags |= 1;
                } else {
                    flags &= !1;
                }
                slot.flags = flags;
                slot.tex_obj_id = r.u32();
                slot.tex_data_version = r.u32();
            }
            LOAD_TLUT => {
                let idx = r.u8() as usize;
                assert!(idx < MAX_TLUTS, "invalid tlut slot {idx}");
                let handle = r.u64();
                let slot = &mut self.loaded_tluts[idx];
                slot.data = res.get(handle);
                slot.format = r.u32();
                slot.num_entries = r.u16();
                slot.tlut_obj_id = r.u32();
                slot.tlut_data_version = r.u32();
                slot.flags &= !0x80;
            }
            SET_POLYGON_OFFSET => {
                self.front_offset = r.f32();
                self.front_scale = r.f32();
                self.back_offset = r.f32();
                self.back_scale = r.f32();
                self.clamp = r.f32();
            }
            DRAW_SIZED => {
                let cmd = r.u8();
                let byte_len = r.u32();
                let fmt = (cmd & GX_VAT_MASK) as u32;
                let prim = (cmd & GX_OPCODE_MASK) as u32;
                if byte_len != 0 {
                    let vs = crate::draw::calc_vtx_size(self, fmt);
                    assert!(vs != 0 && byte_len % vs == 0, "GX_AURORA_DRAW_SIZED: {byte_len} bytes, vertex size {vs}");
                    let n = byte_len / vs;
                    assert!(n <= 0xFFFF);
                    self.draw_prim(prim, fmt, n as u16, r, None);
                }
            }
            DRAW_INDEXED => {
                let cmd = r.u8();
                let vtx_count = r.u16();
                let index_count = r.u32() as usize;
                let fmt = (cmd & GX_VAT_MASK) as u32;
                // Index data is host-endian (little-endian here).
                let idx: Vec<u16> =
                    r.take(index_count * 2).chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                if index_count != 0 {
                    self.draw_prim(GX_TRIANGLES, fmt, vtx_count, r, Some(idx));
                } else {
                    let vs = crate::draw::calc_vtx_size(self, fmt) as usize;
                    r.take(vtx_count as usize * vs);
                }
            }
            _ => self.warnings.push(format!("Unknown/unsupported Aurora subcommand: {sub:04X}")),
        }
    }

    // ------------------------------------------------------------------------------- viewport

    /// Aurora `map_logical_viewport` (AURORA_VIEWPORT_FIT).
    pub fn map_logical_viewport(&self, v: &Viewport) -> Viewport {
        if !self.viewport_fit {
            return *v;
        }
        let (lw, lh) = self.logical_fb_size;
        let (tw, th) = self.target_size;
        if lw == 0 || lh == 0 || tw == 0 || th == 0 {
            return *v;
        }
        let sx = tw as f32 / lw as f32;
        let sy = th as f32 / lh as f32;
        const EPS: f32 = 1.0 / 1024.0;
        let left = (v.left * sx + EPS).floor();
        let top = (v.top * sy + EPS).floor();
        let right = ((v.left + v.width) * sx - EPS).ceil();
        let bottom = ((v.top + v.height) * sy - EPS).ceil();
        Viewport {
            left,
            top,
            width: (right - left).max(1.0),
            height: (bottom - top).max(1.0),
            znear: v.znear,
            zfar: v.zfar,
        }
    }

    /// Aurora `map_logical_scissor`.
    pub fn map_logical_scissor(&self, s: &ClipRect) -> ClipRect {
        if !self.viewport_fit {
            return *s;
        }
        let (lw, lh) = self.logical_fb_size;
        let (tw, th) = self.target_size;
        if lw == 0 || lh == 0 || tw == 0 || th == 0 {
            return *s;
        }
        let sx = tw as f32 / lw as f32;
        let sy = th as f32 / lh as f32;
        let left = s.x as f32 * sx;
        let top = s.y as f32 * sy;
        let right = (s.x + s.width) as f32 * sx;
        let bottom = (s.y + s.height) as f32 * sy;
        let ml = (left.floor() as i32).clamp(0, tw as i32);
        let mt = (top.floor() as i32).clamp(0, th as i32);
        let mr = (right.ceil() as i32).clamp(ml, tw as i32);
        let mb = (bottom.ceil() as i32).clamp(mt, th as i32);
        ClipRect { x: ml, y: mt, width: mr - ml, height: mb - mt }
    }

    pub fn set_logical_viewport(&mut self, v: Viewport) {
        self.logical_viewport = v;
        self.render_viewport = self.map_logical_viewport(&v);
    }
    pub fn set_logical_scissor(&mut self, s: ClipRect) {
        self.logical_scissor = s;
        self.render_scissor = self.map_logical_scissor(&s);
    }

    /// Aurora `clear_depth_value`.
    pub fn clear_depth_value(&self) -> f32 {
        let d = self.clear_depth as f32 / 16_777_215.0;
        if USE_REVERSED_Z {
            1.0 - d
        } else {
            d
        }
    }

    // ------------------------------------------------------------------------------- BP

    fn handle_bp(&mut self, value: u32) {
        let reg_id = (value >> 24) as usize;
        if reg_id == 0xFE {
            self.bp_reg_cache[reg_id] = value & 0x00FF_FFFF;
            return;
        }
        let mask = self.bp_reg_cache[0xFE];
        self.bp_reg_cache[0xFE] = 0x00FF_FFFF;
        let value = ((reg_id as u32) << 24) | (((self.bp_reg_cache[reg_id] & !mask) | (value & mask)) & 0x00FF_FFFF);
        let always = matches!(reg_id, 0x52 | 0x65);
        if self.bp_reg_valid[reg_id] && self.bp_reg_cache[reg_id] == value && !always {
            return;
        }
        self.bp_reg_valid[reg_id] = true;
        self.bp_reg_cache[reg_id] = value;
        let reg = reg_id as u32;
        match reg_id {
            0x00 => self.bp_gen_mode(value),
            0x06..=0x0E => self.bp_ind_mtx(reg, value),
            0x10..=0x1F => self.bp_tev_ind(reg, value),
            0x20 | 0x21 => self.bp_scissor(),
            0x22 => {
                self.line_width = reg_get(value, 8, 0) as u8;
                self.point_size = reg_get(value, 8, 8) as u8;
                self.line_tex_offset = reg_get(value, 3, 16);
                self.point_tex_offset = reg_get(value, 3, 19);
                self.line_half_aspect = reg_get(value, 1, 22) != 0;
            }
            0x25 | 0x26 => {
                let base = (reg_id - 0x25) * 2;
                for i in 0..2 {
                    if base + i < MAX_IND_STAGES {
                        self.ind_stages[base + i].scale_s = reg_get(value, 4, (i * 8) as u32);
                        self.ind_stages[base + i].scale_t = reg_get(value, 4, (i * 8 + 4) as u32);
                    }
                }
            }
            0x27 => {
                for i in 0..MAX_IND_STAGES {
                    self.ind_stages[i].tex_map_id = reg_get(value, 3, (i * 6) as u32);
                    self.ind_stages[i].tex_coord_id = reg_get(value, 3, (i * 6 + 3) as u32);
                }
            }
            0x28..=0x2F => self.bp_tref(reg, value),
            0x30..=0x3F => {
                let t = &mut self.tex_coord_scales[(reg_id - 0x30) / 2];
                if reg_id & 1 != 0 {
                    t.scale_t = reg_get(value, 16, 0) as u16;
                    t.bias_t = reg_get(value, 1, 16) != 0;
                    t.cyl_wrap_t = reg_get(value, 1, 17) != 0;
                } else {
                    t.scale_s = reg_get(value, 16, 0) as u16;
                    t.bias_s = reg_get(value, 1, 16) != 0;
                    t.cyl_wrap_s = reg_get(value, 1, 17) != 0;
                    t.line_offset = reg_get(value, 1, 18) != 0;
                    t.point_offset = reg_get(value, 1, 19) != 0;
                }
            }
            0x40 => {
                self.depth_compare = reg_get(value, 1, 0) != 0;
                self.depth_func = reg_get(value, 3, 1);
                self.depth_update = reg_get(value, 1, 4) != 0;
            }
            0x41 => {
                let blend = reg_get(value, 1, 0) != 0;
                let logic = reg_get(value, 1, 1) != 0;
                self.color_update = reg_get(value, 1, 3) != 0;
                self.alpha_update = reg_get(value, 1, 4) != 0;
                self.blend_fac_dst = reg_get(value, 3, 5);
                self.blend_fac_src = reg_get(value, 3, 8);
                let subtract = reg_get(value, 1, 11) != 0;
                self.blend_op = reg_get(value, 4, 12);
                self.blend_mode = if subtract {
                    GX_BM_SUBTRACT
                } else if blend {
                    GX_BM_BLEND
                } else if logic {
                    GX_BM_LOGIC
                } else {
                    GX_BM_NONE
                };
            }
            0x42 => {
                let alpha = reg_get(value, 8, 0);
                let enabled = reg_get(value, 1, 8) != 0;
                self.dst_alpha = if enabled { alpha } else { u32::MAX };
                self.pixel_fmt = decode_pixel_fmt(self.bp_reg_cache[0x43], value);
            }
            0x43 => {
                self.pixel_fmt = decode_pixel_fmt(value, self.bp_reg_cache[0x42]);
                self.z_fmt = reg_get(value, 3, 3);
                self.z_comp_loc_before_tex = reg_get(value, 1, 6) != 0;
            }
            0x4F => {
                self.clear_color[0] = reg_get(value, 8, 0) as f32 / 255.0;
                self.clear_color[3] = reg_get(value, 8, 8) as f32 / 255.0;
            }
            0x50 => {
                self.clear_color[2] = reg_get(value, 8, 0) as f32 / 255.0;
                self.clear_color[1] = reg_get(value, 8, 8) as f32 / 255.0;
            }
            0x51 => self.clear_depth = reg_get(value, 24, 0),
            0x52 => self.warnings.push("EFB copy (GXCopyTex/GXCopyDisp) not implemented".into()),
            0x65 => {
                let idx = reg_get(value, 10, 0) as usize;
                if idx < MAX_TLUTS {
                    self.loaded_tluts[idx].load_tlut0 = self.bp_reg_cache[0x64];
                    self.loaded_tluts[idx].num_entries = (reg_get(value, 10, 10) + 1) as u16;
                }
            }
            0x80..=0xBF => {
                let idx = reg_id - 0x80;
                let kind = (idx & 0x1F) / 4;
                let map = (idx & 3) + if idx >= 0x20 { 4 } else { 0 };
                let slot = &mut self.loaded_textures[map];
                match kind {
                    0 => slot.mode0 = value,
                    1 => slot.mode1 = value,
                    2 => {
                        slot.image0 = value;
                        slot.width = 0;
                        slot.height = 0;
                        slot.format = INVALID_TEXTURE_FORMAT;
                    }
                    5 => slot.image3 = value,
                    _ => {}
                }
            }
            0xC0..=0xDF => {
                if reg_id & 1 == 0 {
                    self.bp_tev_color(reg, value)
                } else {
                    self.bp_tev_alpha(reg, value)
                }
            }
            0xE0..=0xE7 => self.bp_tev_reg(reg, value),
            0xE8 => {
                self.fog.range_center = (reg_get(value, 10, 0) as i16).wrapping_sub(342);
                self.fog.range_enabled = reg_get(value, 1, 10) != 0;
            }
            0xE9..=0xED => {
                let idx = (reg_id - 0xE9) * 2;
                self.fog.range_k[idx] = reg_get(value, 12, 0) as u16;
                self.fog.range_k[idx + 1] = reg_get(value, 12, 12) as u16;
            }
            0xEE => {
                self.fog.fog0_raw = value;
                self.decode_fog_a();
            }
            0xEF => {
                self.fog.fog1_raw = value;
                let bm = reg_get(value, 24, 0);
                let bs = self.fog.fog2_raw & 0x1F;
                self.fog.b = ldexp(bm as f32 / 8_388_638.0, bs as i32 - 1);
            }
            0xF0 => {
                self.fog.fog2_raw = value;
                let bs = reg_get(value, 5, 0);
                self.decode_fog_a();
                let bm = reg_get(self.fog.fog1_raw, 24, 0);
                self.fog.b = ldexp(bm as f32 / 8_388_638.0, bs as i32 - 1);
            }
            0xF1 => {
                self.fog.ty = reg_get(value, 3, 21) | (reg_get(value, 1, 20) << 3);
                self.decode_fog_a();
                self.fog.c = decode_fog_float(value);
            }
            0xF2 => {
                self.fog.color = [
                    reg_get(value, 8, 16) as f32 / 255.0,
                    reg_get(value, 8, 8) as f32 / 255.0,
                    reg_get(value, 8, 0) as f32 / 255.0,
                    1.0,
                ];
            }
            0xF3 => {
                let ac = &mut self.alpha_compare;
                ac.ref0 = reg_get(value, 8, 0);
                ac.ref1 = reg_get(value, 8, 8);
                ac.comp0 = reg_get(value, 3, 16);
                ac.comp1 = reg_get(value, 3, 19);
                ac.op = reg_get(value, 2, 22);
            }
            0xF6..=0xFD => self.bp_ksel(reg, value),
            _ => {}
        }
    }

    fn decode_fog_a(&mut self) {
        let enc = decode_fog_float(self.fog.fog0_raw);
        if self.fog.ty & 0x08 != 0 {
            self.fog.a = enc;
        } else {
            let bs = self.fog.fog2_raw & 0x1F;
            self.fog.a = ldexp(enc, bs as i32);
        }
    }

    fn bp_gen_mode(&mut self, v: u32) {
        self.num_tex_gens = reg_get(v, 4, 0) as u8;
        self.num_chans = reg_get(v, 3, 4) as u8;
        self.num_tev_stages = (reg_get(v, 4, 10) + 1) as u8;
        let hw = reg_get(v, 2, 14);
        self.cull_mode = match hw {
            GX_CULL_FRONT => GX_CULL_BACK,
            GX_CULL_BACK => GX_CULL_FRONT,
            other => other,
        };
        self.num_ind_stages = reg_get(v, 3, 16) as u8;
        self.xf_reg_valid[0x09] = false;
        self.xf_reg_valid[0x3F] = false;
    }

    fn bp_ind_mtx(&mut self, reg: u32, v: u32) {
        let idx = ((reg - 0x06) / 3) as usize;
        let column = ((reg - 0x06) % 3) as usize;
        let info = &mut self.ind_tex_mtxs[idx];
        info.mtx[column][0] = sext11(reg_get(v, 11, 0)) as f32 / 1024.0;
        info.mtx[column][1] = sext11(reg_get(v, 11, 11)) as f32 / 1024.0;
        let scale_bits = reg_get(v, 2, 22) as u8;
        let shift = (column * 2) as u8;
        if column == 2 {
            info.adj_scale_raw = (info.adj_scale_raw & !(1u8 << shift)) | ((scale_bits & 1) << shift);
        } else {
            info.adj_scale_raw = (info.adj_scale_raw & !(3u8 << shift)) | (scale_bits << shift);
        }
        info.scale_exp = (info.adj_scale_raw as i8).wrapping_sub(17);
    }

    fn bp_tev_ind(&mut self, reg: u32, v: u32) {
        let s = &mut self.tev_stages[(reg - 0x10) as usize];
        s.ind_tex_stage = reg_get(v, 2, 0);
        s.ind_tex_format = reg_get(v, 2, 2);
        s.ind_tex_bias_sel = reg_get(v, 3, 4);
        s.ind_tex_alpha_sel = reg_get(v, 2, 7);
        s.ind_tex_mtx_id = reg_get(v, 4, 9);
        s.ind_tex_wrap_s = reg_get(v, 3, 13);
        s.ind_tex_wrap_t = reg_get(v, 3, 16);
        s.ind_tex_use_orig_lod = reg_get(v, 1, 19) != 0;
        s.ind_tex_add_prev = reg_get(v, 1, 20) != 0;
    }

    fn bp_scissor(&mut self) {
        let s0 = self.bp_reg_cache[0x20];
        let s1 = self.bp_reg_cache[0x21];
        let tp = reg_get(s0, 11, 0) as i32 - 342;
        let lf = reg_get(s0, 11, 12) as i32 - 342;
        let bm = reg_get(s1, 11, 0) as i32 - 342;
        let rt = reg_get(s1, 11, 12) as i32 - 342;
        let wd = (rt - lf + 1).max(0);
        let ht = (bm - tp + 1).max(0);
        self.set_logical_scissor(ClipRect { x: lf, y: tp, width: wd, height: ht });
    }

    fn bp_tref(&mut self, reg: u32, v: u32) {
        const R2C: [u32; 8] =
            [GX_COLOR0A0, GX_COLOR1A1, GX_COLOR0A0, GX_COLOR1A1, GX_COLOR0A0, GX_ALPHA_BUMP, GX_ALPHA_BUMPN, GX_COLOR_ZERO];
        let idx = reg - 0x28;
        for half in 0..2 {
            let stage = (idx * 2 + half) as usize;
            let shift = half * 12;
            let s = &mut self.tev_stages[stage];
            s.tex_map_id = reg_get(v, 3, shift);
            s.tex_coord_id = reg_get(v, 3, shift + 3);
            if reg_get(v, 1, shift + 6) == 0 {
                s.tex_map_id = GX_TEXMAP_NULL;
            }
            let ch = reg_get(v, 3, shift + 7);
            s.channel_id = R2C[ch as usize];
        }
    }

    fn bp_tev_color(&mut self, reg: u32, v: u32) {
        let s = &mut self.tev_stages[((reg - 0xC0) / 2) as usize];
        s.color_pass = TevPass { d: reg_get(v, 4, 0), c: reg_get(v, 4, 4), b: reg_get(v, 4, 8), a: reg_get(v, 4, 12) };
        s.color_op.clamp = reg_get(v, 1, 19) != 0;
        s.color_op.out_reg = reg_get(v, 2, 22);
        if reg_get(v, 2, 16) == 3 {
            let hw = reg_get(v, 1, 18) | (reg_get(v, 2, 20) << 1);
            s.color_op.op = hw + 8;
            s.color_op.bias = GX_TB_ZERO;
            s.color_op.scale = GX_CS_SCALE_1;
        } else {
            s.color_op.op = reg_get(v, 1, 18);
            s.color_op.bias = reg_get(v, 2, 16);
            s.color_op.scale = reg_get(v, 2, 20);
        }
    }

    fn bp_tev_alpha(&mut self, reg: u32, v: u32) {
        let s = &mut self.tev_stages[((reg - 0xC1) / 2) as usize];
        s.tev_swap_ras = reg_get(v, 2, 0);
        s.tev_swap_tex = reg_get(v, 2, 2);
        s.alpha_pass = TevPass { d: reg_get(v, 3, 4), c: reg_get(v, 3, 7), b: reg_get(v, 3, 10), a: reg_get(v, 3, 13) };
        s.alpha_op.clamp = reg_get(v, 1, 19) != 0;
        s.alpha_op.out_reg = reg_get(v, 2, 22);
        if reg_get(v, 2, 16) == 3 {
            let hw = reg_get(v, 1, 18) | (reg_get(v, 2, 20) << 1);
            s.alpha_op.op = hw + 8;
            s.alpha_op.bias = GX_TB_ZERO;
            s.alpha_op.scale = GX_CS_SCALE_1;
        } else {
            s.alpha_op.op = reg_get(v, 1, 18);
            s.alpha_op.bias = reg_get(v, 2, 16);
            s.alpha_op.scale = reg_get(v, 2, 20);
        }
    }

    fn bp_tev_reg(&mut self, reg: u32, v: u32) {
        let idx = ((reg - 0xE0) / 2) as usize;
        let is_ra = reg & 1 == 0;
        if reg_get(v, 1, 23) != 0 {
            if idx < MAX_KCOLORS {
                let k = &mut self.kcolors[idx];
                if is_ra {
                    k[0] = reg_get(v, 8, 0) as f32 / 255.0;
                    k[3] = reg_get(v, 8, 12) as f32 / 255.0;
                } else {
                    k[2] = reg_get(v, 8, 0) as f32 / 255.0;
                    k[1] = reg_get(v, 8, 12) as f32 / 255.0;
                }
            }
        } else if idx < MAX_TEV_REGS {
            let c = &mut self.color_regs[idx];
            if is_ra {
                c[0] = sext11(reg_get(v, 11, 0)) as f32 / 255.0;
                c[3] = sext11(reg_get(v, 11, 12)) as f32 / 255.0;
            } else {
                c[2] = sext11(reg_get(v, 11, 0)) as f32 / 255.0;
                c[1] = sext11(reg_get(v, 11, 12)) as f32 / 255.0;
            }
        }
    }

    fn bp_ksel(&mut self, reg: u32, v: u32) {
        let k = (reg - 0xF6) as usize;
        if k < MAX_TEV_SWAP * 2 {
            let sw = &mut self.tev_swap_table[k / 2];
            if k & 1 != 0 {
                sw.blue = reg_get(v, 2, 0);
                sw.alpha = reg_get(v, 2, 2);
            } else {
                sw.red = reg_get(v, 2, 0);
                sw.green = reg_get(v, 2, 2);
            }
        }
        let s0 = k * 2;
        let s1 = k * 2 + 1;
        if s0 < MAX_TEV_STAGES {
            self.tev_stages[s0].kc_sel = reg_get(v, 5, 4);
            self.tev_stages[s0].ka_sel = reg_get(v, 5, 9);
        }
        if s1 < MAX_TEV_STAGES {
            self.tev_stages[s1].kc_sel = reg_get(v, 5, 14);
            self.tev_stages[s1].ka_sel = reg_get(v, 5, 19);
        }
    }

    // ------------------------------------------------------------------------------- CP

    fn handle_cp(&mut self, addr: u8, v: u32) {
        let a = addr as usize;
        let always = (0xA0..=0xAF).contains(&a);
        if !always {
            if self.cp_reg_valid[a] && self.cp_reg_cache[a] == v {
                return;
            }
            self.cp_reg_valid[a] = true;
            self.cp_reg_cache[a] = v;
        }
        match a {
            0x30 => {
                self.current_pn_mtx = reg_get(v, 6, 0) / 3;
                self.xf_reg_valid[0x18] = false;
            }
            0x50 => {
                let d = &mut self.vtx_desc;
                for i in 0..9 {
                    d[GX_VA_PNMTXIDX as usize + i] = reg_get(v, 1, i as u32);
                }
                d[GX_VA_POS as usize] = reg_get(v, 2, 9);
                d[GX_VA_NRM as usize] = reg_get(v, 2, 11);
                d[GX_VA_CLR0 as usize] = reg_get(v, 2, 13);
                d[GX_VA_CLR1 as usize] = reg_get(v, 2, 15);
            }
            0x60 => {
                for i in 0..8 {
                    self.vtx_desc[GX_VA_TEX0 as usize + i] = reg_get(v, 2, (i * 2) as u32);
                }
            }
            0x70..=0x77 => {
                let f = &mut self.vtx_fmts[a - 0x70];
                let pos = &mut f[GX_VA_POS as usize];
                pos.cnt = reg_get(v, 1, 0);
                pos.ty = reg_get(v, 3, 1);
                pos.frac = reg_get(v, 5, 4) as u8;
                let nrm_cnt = reg_get(v, 1, 9);
                let nbt3 = reg_get(v, 1, 31);
                let n = &mut f[GX_VA_NRM as usize];
                n.cnt = if nbt3 != 0 {
                    GX_NRM_NBT3
                } else if nrm_cnt != 0 {
                    GX_NRM_NBT
                } else {
                    GX_NRM_XYZ
                };
                n.ty = reg_get(v, 3, 10);
                n.frac = match n.ty {
                    GX_U8 | GX_S8 => 6,
                    GX_U16 | GX_S16 => 14,
                    _ => 0,
                };
                f[GX_VA_CLR0 as usize] = VtxAttrFmt { cnt: reg_get(v, 1, 13), ty: reg_get(v, 3, 14), frac: 0 };
                f[GX_VA_CLR1 as usize] = VtxAttrFmt { cnt: reg_get(v, 1, 17), ty: reg_get(v, 3, 18), frac: 0 };
                f[GX_VA_TEX0 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 21), ty: reg_get(v, 3, 22), frac: reg_get(v, 5, 25) as u8 };
            }
            0x80..=0x87 => {
                let f = &mut self.vtx_fmts[a - 0x80];
                f[GX_VA_TEX1 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 0), ty: reg_get(v, 3, 1), frac: reg_get(v, 5, 4) as u8 };
                f[GX_VA_TEX2 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 9), ty: reg_get(v, 3, 10), frac: reg_get(v, 5, 13) as u8 };
                f[GX_VA_TEX3 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 18), ty: reg_get(v, 3, 19), frac: reg_get(v, 5, 22) as u8 };
                let t4 = &mut f[GX_VA_TEX4 as usize];
                t4.cnt = reg_get(v, 1, 27);
                t4.ty = reg_get(v, 3, 28);
            }
            0x90..=0x97 => {
                let f = &mut self.vtx_fmts[a - 0x90];
                f[GX_VA_TEX4 as usize].frac = reg_get(v, 5, 0) as u8;
                f[GX_VA_TEX5 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 5), ty: reg_get(v, 3, 6), frac: reg_get(v, 5, 9) as u8 };
                f[GX_VA_TEX6 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 14), ty: reg_get(v, 3, 15), frac: reg_get(v, 5, 18) as u8 };
                f[GX_VA_TEX7 as usize] =
                    VtxAttrFmt { cnt: reg_get(v, 1, 23), ty: reg_get(v, 3, 24), frac: reg_get(v, 5, 27) as u8 };
            }
            0xA0..=0xAF => self.warnings.push("CP_REG_ARRAYBASE_ID is not supported".into()),
            0xB0..=0xBF => {
                let attr = a - 0xB0 + GX_VA_POS as usize;
                if attr < MAX_VTX_ATTR {
                    self.arrays[attr].stride = v as u8;
                }
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------------------- XF

    fn store_f32(dst: &mut f32, bits: u32) {
        *dst = f32::from_bits(bits);
    }

    /// Aurora `copy_xf_data`: XF memory (matrices, lights). `data` is `len` words.
    fn copy_xf_data(&mut self, addr: u32, data: &[u8], len: u32, le: bool) -> bool {
        let word = |i: usize| -> u32 {
            let b = [data[i * 4], data[i * 4 + 1], data[i * 4 + 2], data[i * 4 + 3]];
            if le {
                u32::from_le_bytes(b)
            } else {
                u32::from_be_bytes(b)
            }
        };
        let len = len as usize;
        if addr < 0x78 {
            let idx = (addr / 12) as usize;
            assert!(addr % 12 == 0 && len == 12, "XF: PosMtx sub-copy unsupported");
            for i in 0..len {
                Self::store_f32(&mut self.pn_mtx[idx].pos[i / 4][i % 4], word(i));
            }
            return true;
        }
        if addr < 0xF0 {
            let base = addr - 0x78;
            let idx = (base / 12) as usize;
            assert!(base % 12 == 0 && (len == 8 || len == 12), "XF TexMtx sub-copy unsupported");
            for i in 0..len {
                Self::store_f32(&mut self.tex_mtxs[idx][i / 4][i % 4], word(i));
            }
            return true;
        }
        if (0x400..0x45A).contains(&addr) {
            let base = addr - 0x400;
            let idx = (base / 9) as usize;
            assert!(base % 9 == 0 && len == 9, "XF: NrmMtx sub-copy unsupported");
            for i in 0..len {
                Self::store_f32(&mut self.pn_mtx[idx].nrm[i / 3][i % 3], word(i));
            }
            return true;
        }
        if (0x500..0x5F0).contains(&addr) {
            let base = addr - 0x500;
            let idx = (base / 12) as usize;
            assert!(base % 12 == 0 && len == 12, "XF: PTTexMtx sub-copy unsupported");
            for i in 0..len {
                Self::store_f32(&mut self.pt_tex_mtxs[idx][i / 4][i % 4], word(i));
            }
            return true;
        }
        if (0x600..0x680).contains(&addr) {
            let base = addr - 0x600;
            let li = (base / 0x10) as usize;
            let start = (base % 0x10) as usize;
            assert!(start + len <= 0x10, "XF: Light copy crossing lights unsupported");
            let l = &mut self.lights[li];
            for i in 0..len {
                let v = word(i);
                match start + i {
                    3 => l.color = unpack_color(v),
                    4 => Self::store_f32(&mut l.cos_att[0], v),
                    5 => Self::store_f32(&mut l.cos_att[1], v),
                    6 => Self::store_f32(&mut l.cos_att[2], v),
                    7 => Self::store_f32(&mut l.dist_att[0], v),
                    8 => Self::store_f32(&mut l.dist_att[1], v),
                    9 => Self::store_f32(&mut l.dist_att[2], v),
                    10 => Self::store_f32(&mut l.pos[0], v),
                    11 => Self::store_f32(&mut l.pos[1], v),
                    12 => Self::store_f32(&mut l.pos[2], v),
                    13 => Self::store_f32(&mut l.dir[0], v),
                    14 => Self::store_f32(&mut l.dir[1], v),
                    15 => Self::store_f32(&mut l.dir[2], v),
                    _ => {}
                }
            }
            return true;
        }
        false
    }

    fn handle_xf(&mut self, addr: u16, data: &[u8]) {
        let count = data.len() / 4;
        if self.copy_xf_data(addr as u32, data, count as u32, false) {
            return;
        }
        if addr < 0x1000 {
            return;
        }
        let xf = (addr - 0x1000) as usize;
        let mut i = 0usize;
        while i < count {
            let reg = xf + i;
            if reg == 0x1A && count - i >= 6 {
                self.xf_load_viewport(&data[i * 4..]);
                i += 6;
                continue;
            }
            if reg == 0x20 && count - i >= 7 {
                let mut unchanged = true;
                for w in 0..7 {
                    unchanged &= self.xf_reg_unchanged(0x20 + w, be32(&data[i * 4..], w));
                }
                if !unchanged {
                    self.xf_load_projection(&data[i * 4..]);
                }
                i += 7;
                continue;
            }
            if (0x1A..=0x26).contains(&reg) {
                i += 1;
                continue;
            }
            let val = be32(data, i);
            i += 1;
            if reg >= XF_REG_COUNT {
                continue;
            }
            if self.xf_reg_unchanged(reg, val) {
                continue;
            }
            self.decode_xf(reg, val);
        }
    }

    fn xf_reg_unchanged(&mut self, reg: usize, val: u32) -> bool {
        if self.xf_reg_valid[reg] && self.xf_reg_cache[reg] == val {
            return true;
        }
        self.xf_reg_valid[reg] = true;
        self.xf_reg_cache[reg] = val;
        false
    }

    fn decode_xf(&mut self, reg: usize, v: u32) {
        match reg {
            0x09 => {
                self.num_chans = v as u8;
                self.bp_reg_valid[0] = false;
            }
            0x0A..=0x0D => {
                let chan = (reg - 0x0A) & 1;
                let color = unpack_color(v);
                if reg <= 0x0B {
                    self.color_channel_state[GX_COLOR0 as usize + chan].amb_color = color;
                    self.color_channel_state[GX_ALPHA0 as usize + chan].amb_color = color;
                } else {
                    self.color_channel_state[GX_COLOR0 as usize + chan].mat_color = color;
                    self.color_channel_state[GX_ALPHA0 as usize + chan].mat_color = color;
                }
            }
            0x0E..=0x11 => {
                let id = reg - 0x0E;
                let c = &mut self.color_channel_config[id];
                c.mat_src = reg_get(v, 1, 0);
                c.lighting_enabled = reg_get(v, 1, 1) != 0;
                let lo = reg_get(v, 4, 2);
                c.amb_src = reg_get(v, 1, 6);
                c.diff_fn = reg_get(v, 2, 7);
                let bit9 = reg_get(v, 1, 9) != 0;
                let bit10 = reg_get(v, 1, 10) != 0;
                let hi = reg_get(v, 4, 11);
                c.attn_fn = if !bit10 {
                    GX_AF_SPEC
                } else if !bit9 {
                    GX_AF_NONE
                } else {
                    GX_AF_SPOT
                };
                self.color_channel_state[id].light_mask = lo | (hi << 4);
            }
            0x18 => {
                self.current_pn_mtx = reg_get(v, 6, 0) / 3;
                self.cp_reg_valid[0x30] = false;
                for i in 0..4 {
                    self.tcgs[i].mtx = reg_get(v, 6, 6 + i as u32 * 6);
                }
            }
            0x19 => {
                for i in 0..4 {
                    self.tcgs[i + 4].mtx = reg_get(v, 6, i as u32 * 6);
                }
            }
            0x3F => {
                self.num_tex_gens = v as u8;
                self.bp_reg_valid[0] = false;
            }
            0x40..=0x4F => {
                let i = reg - 0x40;
                if i >= MAX_TEX_COORD {
                    return;
                }
                let t = &mut self.tcgs[i];
                let proj = reg_get(v, 1, 1) != 0;
                let tg = reg_get(v, 3, 4);
                let row = reg_get(v, 5, 7);
                if tg == 0 {
                    t.ty = if proj { GX_TG_MTX3x4 } else { GX_TG_MTX2x4 };
                } else if tg == 1 {
                    t.ty = reg_get(v, 3, 15) + 2;
                } else if tg == 2 || tg == 3 {
                    t.ty = GX_TG_SRTG;
                    t.src = if tg == 2 { GX_TG_COLOR0 } else { GX_TG_COLOR1 };
                }
                t.emboss_src = reg_get(v, 3, 12);
                const ROW_TO_SRC: [u32; 13] = [
                    GX_TG_POS, GX_TG_NRM, GX_TG_COLOR0, GX_TG_BINRM, GX_TG_TANGENT, GX_TG_TEX0, GX_TG_TEX1, GX_TG_TEX2,
                    GX_TG_TEX3, GX_TG_TEX4, GX_TG_TEX5, GX_TG_TEX6, GX_TG_TEX7,
                ];
                if tg != 2 && tg != 3 && row < 13 {
                    t.src = ROW_TO_SRC[row as usize];
                }
            }
            0x50..=0x57 => {
                let t = &mut self.tcgs[reg - 0x50];
                t.post_mtx = reg_get(v, 6, 0) + 64;
                t.normalize = reg_get(v, 1, 8) != 0;
            }
            _ => {}
        }
    }

    fn xf_load_viewport(&mut self, d: &[u8]) {
        let f = |i: usize| f32::from_bits(be32(d, i));
        let (sx, sy, sz, ox, oy, oz) = (f(0), f(1), f(2), f(3), f(4), f(5));
        let width = sx * 2.0;
        let height = -sy * 2.0;
        self.set_logical_viewport(Viewport {
            left: ox - 340.0 - width / 2.0,
            top: oy - 340.0 - height / 2.0,
            width,
            height,
            znear: (oz - sz) / 1.677_721_5e7,
            zfar: oz / 1.677_721_5e7,
        });
    }

    fn xf_load_projection(&mut self, d: &[u8]) {
        let f = |i: usize| f32::from_bits(be32(d, i));
        let ty = be32(d, 6);
        self.proj_type = ty;
        let mut p = [[0.0f32; 4]; 4];
        p[0][0] = f(0);
        p[1][1] = f(2);
        p[2][2] = f(4);
        p[2][3] = f(5);
        if ty == GX_ORTHOGRAPHIC {
            p[0][3] = f(1);
            p[1][3] = f(3);
            p[3][3] = 1.0;
        } else {
            p[0][2] = f(1);
            p[1][2] = f(3);
            p[3][2] = -1.0;
        }
        self.proj = p;
    }

    /// Decoded texture for map `id`: Aurora `resolve_static_texture` / `resolve_static_palette_texture`
    /// (no caches; the renderer caches by object id + version).
    pub fn decode_texture(&self, id: usize) -> Option<texture::Converted> {
        let obj = &self.loaded_textures[id];
        let data = obj.data.as_ref()?;
        let fmt = obj.format();
        let (w, h, mips) = (obj.width(), obj.height(), obj.mip_count());
        if w == 0 || h == 0 {
            return None;
        }
        if matches!(fmt, GX_TF_C4 | GX_TF_C8 | GX_TF_C14X2) {
            let t = &self.loaded_tluts[obj.tlut as usize];
            let td = t.data.as_ref()?;
            Some(texture::convert_texture_palette(fmt, w, h, mips, data, t.format, t.num_entries, td))
        } else {
            Some(texture::convert_texture(fmt, w, h, mips, data))
        }
    }

    /// Take the draws decoded so far.
    pub fn take_draws(&mut self) -> Vec<crate::draw::DrawCmd> {
        std::mem::take(&mut self.draws)
    }
}

fn decode_pixel_fmt(pe: u32, cmode1: u32) -> u32 {
    match reg_get(pe, 3, 0) {
        0 => GX_PF_RGB8_Z24,
        1 => GX_PF_RGBA6_Z24,
        2 => GX_PF_RGB565_Z16,
        3 => GX_PF_Z24,
        4 => match reg_get(cmode1, 2, 9) {
            1 => GX_PF_U8,
            2 => GX_PF_V8,
            _ => GX_PF_Y8,
        },
        5 => GX_PF_YUV420,
        _ => GX_PF_RGB8_Z24,
    }
}
