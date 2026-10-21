//! `src/sysdolphin/baselib/state.c`: HSD's cached GX render state (cull/blend/z/alpha/PE), the
//! channel mode per render mode and `HSD_SetupRenderModeWithCustomPE`, plus the shared pieces
//! of the native HSD draw path (tu/tev.rs, tu/lobj.rs, tu/tobj.rs, tu/mobj.rs, tu/pobj.rs,
//! tu/dobj.rs, tu/displayfunc.rs):
//!
//! * [`GxBackend`]: the GX calls the HSD display functions make. melee_runtime does not link a GPU
//!   backend (melee_gx pulls in wgpu); the bin implements this trait over `melee_gx::shim::GxShim`.
//! * [`HsdRender`]: the display path's statics (state.c's `state_*`, tev.c's `prev_ch` and stage
//!   counter, pobj.c's matrix marks, lobj.c's light masks), one per renderer.
//! * [`Dat`]: big-endian reads of the descriptors the display path walks (DAT data section,
//!   pointers are data-relative offsets, 0 = NULL).
//!
//! Descriptor-driven: the C draws loaded HSD objects (HSD_DObj/MObj/TObj/PObj built by the
//! *LoadDesc functions); this port reads the same fields straight from their descriptors, which
//! the loaders copy unchanged (MObjLoad adds RENDER_TOON, DObjLoad derives the trsp bits; both are
//! applied where read). Material/texture animation is not applied (no MObj/TObj AObjs here).
#![allow(clippy::too_many_arguments)]

use crate::mtx::Mtx;

/// GX enum values the display path uses (dolphin/gx/GXEnum.h; same values as melee_gx::consts).
pub mod gx {
    pub const GX_PNMTX0: u32 = 0;
    pub const GX_PNMTX1: u32 = 3;
    pub const GX_TEXMTX0: u32 = 30;
    pub const GX_TEXMTX1: u32 = 33;
    pub const GX_IDENTITY: u32 = 60;
    pub const GX_PTIDENTITY: u32 = 125;
    pub const GX_MTX3x4: u32 = 0;
    pub const GX_MTX2x4: u32 = 1;
    pub const GX_VA_PNMTXIDX: u32 = 0;
    pub const GX_VA_TEX7MTXIDX: u32 = 8;
    pub const GX_VA_POS: u32 = 9;
    pub const GX_VA_NULL: u32 = 0xFF;
    pub const GX_DIRECT: u32 = 1;
    pub const GX_INDEX8: u32 = 2;
    pub const GX_INDEX16: u32 = 3;
    pub const GX_VTXFMT0: u32 = 0;
    pub const GX_CULL_NONE: u32 = 0;
    pub const GX_CULL_FRONT: u32 = 1;
    pub const GX_CULL_BACK: u32 = 2;
    pub const GX_COLOR0: u32 = 0;
    pub const GX_ALPHA0: u32 = 2;
    pub const GX_COLOR0A0: u32 = 4;
    pub const GX_COLOR1A1: u32 = 5;
    pub const GX_COLOR_NULL: u32 = 0xFF;
    pub const GX_SRC_REG: u32 = 0;
    pub const GX_SRC_VTX: u32 = 1;
    pub const GX_DF_NONE: u32 = 0;
    pub const GX_DF_CLAMP: u32 = 2;
    pub const GX_AF_SPEC: u32 = 0;
    pub const GX_AF_SPOT: u32 = 1;
    pub const GX_AF_NONE: u32 = 2;
    pub const GX_TEXCOORD_NULL: u32 = 0xFF;
    pub const GX_TEXMAP_NULL: u32 = 0xFF;
    pub const GX_TG_MTX3x4: u32 = 0;
    pub const GX_TG_MTX2x4: u32 = 1;
    pub const GX_TG_POS: u32 = 0;
    pub const GX_TG_NRM: u32 = 1;
    pub const GX_TG_SRTG: u32 = 10;
    pub const GX_TG_TEX0: u32 = 4;
    pub const GX_PASSCLR: u32 = 4;
    pub const GX_TEV_SWAP0: u32 = 0;
    pub const GX_TEV_ADD: u32 = 0;
    pub const GX_TEV_SUB: u32 = 1;
    pub const GX_TB_ZERO: u32 = 0;
    pub const GX_CS_SCALE_1: u32 = 0;
    pub const GX_TEVPREV: u32 = 0;
    pub const GX_TEVREG0: u32 = 1;
    pub const GX_CC_CPREV: u32 = 0;
    pub const GX_CC_APREV: u32 = 1;
    pub const GX_CC_C0: u32 = 2;
    pub const GX_CC_A0: u32 = 3;
    pub const GX_CC_TEXC: u32 = 8;
    pub const GX_CC_TEXA: u32 = 9;
    pub const GX_CC_RASC: u32 = 10;
    pub const GX_CC_RASA: u32 = 11;
    pub const GX_CC_ONE: u32 = 12;
    pub const GX_CC_HALF: u32 = 13;
    pub const GX_CC_KONST: u32 = 14;
    pub const GX_CC_ZERO: u32 = 15;
    pub const GX_CA_APREV: u32 = 0;
    pub const GX_CA_A0: u32 = 1;
    pub const GX_CA_TEXA: u32 = 4;
    pub const GX_CA_RASA: u32 = 5;
    pub const GX_CA_KONST: u32 = 6;
    pub const GX_CA_ZERO: u32 = 7;
    pub const GX_TEV_KCSEL_1: u32 = 0;
    pub const GX_TEV_KCSEL_K0: u32 = 0xC;
    pub const GX_TEV_KASEL_1: u32 = 0;
    pub const GX_TEV_KASEL_K0_A: u32 = 0x1C;
    pub const GX_TLUT0: u32 = 0;
    pub const GX_BIGTLUT0: u32 = 16;
    pub const GX_TF_C4: u32 = 8;
    pub const GX_TF_C8: u32 = 9;
    pub const GX_TF_C14X2: u32 = 10;
    pub const GX_LIN_MIP_LIN: u32 = 5;
    pub const GX_LIN_MIP_NEAR: u32 = 3;
    pub const GX_ANISO_1: u32 = 0;
    pub const GX_MIRROR: u32 = 2;
}
use gx::*;

/// An RGBA8 colour as GX takes it.
pub type Rgba = [u8; 4];

/// `GXInitTexObj[CI]` + `GXInitTexObjLOD` + `GXLoadTexObj` in one call (HSD_TObjSetup). `image`
/// runs from the image data to the end of the DAT; the backend sizes it (GXGetTexBufferSize).
pub struct TexLoad<'a> {
    pub key: u32,
    pub image: &'a [u8],
    pub width: u16,
    pub height: u16,
    pub format: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub mipmap: bool,
    /// CI formats: the TLUT name loaded by [`GxBackend::load_tlut`].
    pub tlut_name: Option<u32>,
    pub min_filt: u32,
    pub mag_filt: u32,
    pub min_lod: f32,
    pub max_lod: f32,
    pub lod_bias: f32,
    pub bias_clamp: bool,
    pub edge_lod: bool,
    pub max_aniso: u32,
    pub map: u32,
}

/// A `GXLightObj` as lobj.c fills it (GXInitLightColor/Pos/Dir/Attn), loaded to `id`.
#[derive(Clone, Copy, Debug)]
pub struct LightLoad {
    pub color: Rgba,
    pub pos: [f32; 3],
    pub dir: [f32; 3],
    /// a0 a1 a2 k0 k1 k2.
    pub attn: [f32; 6],
}

/// The GX entry points HSD's display functions call (the `gw_GX*` shim in the C build).
pub trait GxBackend {
    fn set_cull_mode(&mut self, mode: u32);
    fn set_blend_mode(&mut self, ty: u32, src: u32, dst: u32, op: u32);
    fn set_z_mode(&mut self, enable: bool, func: u32, update: bool);
    fn set_alpha_compare(&mut self, comp0: u32, ref0: u8, op: u32, comp1: u32, ref1: u8);
    fn set_color_update(&mut self, on: bool);
    fn set_alpha_update(&mut self, on: bool);
    fn set_dst_alpha(&mut self, on: bool, a: u8);
    fn set_z_comp_loc(&mut self, before_tex: bool);
    fn set_dither(&mut self, on: bool);
    fn set_num_chans(&mut self, n: u8);
    fn set_chan_amb_color(&mut self, chan: u32, c: Rgba);
    fn set_chan_mat_color(&mut self, chan: u32, c: Rgba);
    fn set_chan_ctrl(&mut self, chan: u32, enable: bool, amb: u32, mat: u32, mask: u32, diff: u32, attn: u32);
    fn set_num_tex_gens(&mut self, n: u8);
    fn set_num_tev_stages(&mut self, n: u8);
    fn set_tev_order(&mut self, stage: u32, coord: u32, map: u32, color: u32);
    fn set_tev_op(&mut self, stage: u32, mode: u32);
    fn set_tev_swap_mode(&mut self, stage: u32, ras: u32, tex: u32);
    fn set_tev_color_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32);
    fn set_tev_alpha_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32);
    fn set_tev_color_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32);
    fn set_tev_alpha_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32);
    fn set_tev_kcolor(&mut self, id: u32, c: Rgba);
    fn set_tev_kcolor_sel(&mut self, stage: u32, sel: u32);
    fn set_tev_kalpha_sel(&mut self, stage: u32, sel: u32);
    fn set_tex_coord_gen2(&mut self, coord: u32, func: u32, src: u32, mtx: u32, normalize: bool, pt: u32);
    fn load_tex_mtx_imm(&mut self, m: &Mtx, id: u32, ty: u32);
    fn load_pos_mtx_imm(&mut self, m: &Mtx, id: u32);
    fn load_nrm_mtx_imm(&mut self, m: &Mtx, id: u32);
    fn set_current_mtx(&mut self, id: u32);
    fn clear_vtx_desc(&mut self);
    fn set_vtx_desc(&mut self, attr: u32, ty: u32);
    fn set_vtx_attr_fmt(&mut self, fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8);
    /// `data` is exactly the bytes the bound display lists index (pobj.rs sizes it).
    fn set_array(&mut self, attr: u32, data: &[u8], stride: u8);
    fn call_display_list(&mut self, list: &[u8]);
    fn load_tlut(&mut self, key: u32, lut: &[u8], fmt: u32, n: u16, name: u32);
    fn load_texture(&mut self, t: &TexLoad);
    fn load_light(&mut self, id: u32, l: &LightLoad);
}

/// Big-endian descriptor reads.
#[derive(Clone, Copy)]
pub struct Dat<'a>(pub &'a [u8]);

impl<'a> Dat<'a> {
    pub fn u8_(&self, at: u32) -> u8 {
        self.0.get(at as usize).copied().unwrap_or(0)
    }
    pub fn u16_(&self, at: u32) -> u16 {
        u16::from_be_bytes([self.u8_(at), self.u8_(at + 1)])
    }
    pub fn u32_(&self, at: u32) -> u32 {
        u32::from_be_bytes([self.u8_(at), self.u8_(at + 1), self.u8_(at + 2), self.u8_(at + 3)])
    }
    pub fn f32_(&self, at: u32) -> f32 {
        f32::from_bits(self.u32_(at))
    }
    /// A pointer field: `None` for NULL or out of range.
    pub fn ptr(&self, at: u32) -> Option<u32> {
        let p = self.u32_(at);
        (p != 0 && (p as usize) < self.0.len()).then_some(p)
    }
    pub fn color(&self, at: u32) -> Rgba {
        [self.u8_(at), self.u8_(at + 1), self.u8_(at + 2), self.u8_(at + 3)]
    }
    pub fn mtx(&self, at: u32) -> Mtx {
        let mut m = [[0.0; 4]; 3];
        for (r, row) in m.iter_mut().enumerate() {
            for (c, v) in row.iter_mut().enumerate() {
                *v = self.f32_(at + 4 * (r as u32 * 4 + c as u32));
            }
        }
        m
    }
    pub fn tail(&self, at: u32) -> &'a [u8] {
        self.0.get(at as usize..).unwrap_or(&[])
    }
}

/// `HSD_Chan` (tev.h): one GXSetChanCtrl + colours.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsdChan {
    pub chan: u32,
    pub amb_color: Rgba,
    pub mat_color: Rgba,
    pub enable: bool,
    pub amb_src: u32,
    pub mat_src: u32,
    pub light_mask: u32,
    pub diff_fn: u32,
    pub attn_fn: u32,
}

/// state.c's `matstate`.
#[derive(Clone, Copy, Debug, Default)]
pub struct MatState {
    pub ambient: Rgba,
    pub diffuse: Rgba,
    pub specular: Rgba,
    pub alpha: u8,
    pub shininess: f32,
}

/// A light as the display path needs it (lobj.c's active list after HSD_LObjSetupInit).
#[derive(Clone, Copy, Debug)]
pub struct ActiveLight {
    /// LOBJ_* flags (type in bits 0-1, LOBJ_DIFFUSE 4, LOBJ_SPECULAR 8, LOBJ_ALPHA 0x10).
    pub flags: u32,
    pub color: Rgba,
    /// GX light id bit (GX_LIGHT0 << n); 0 for ambient.
    pub id: u32,
}

/// The statics of state.c, tev.c, pobj.c (matrix marks) and lobj.c the display path uses.
#[derive(Debug)]
pub struct HsdRender {
    // state.c
    cull_mode: i64,
    blend: Option<(u32, u32, u32, u32)>,
    zmode: Option<(bool, u32, bool)>,
    alpha_cmp: Option<(u32, u8, u32, u32, u8)>,
    color_update: Option<bool>,
    alpha_update: Option<bool>,
    dst_alpha: Option<(bool, u8)>,
    before_tex: Option<bool>,
    dither: Option<bool>,
    pub matstate: MatState,
    // tev.c
    pub(crate) prev_amb: [Option<Rgba>; 2],
    pub(crate) prev_mat: [Option<Rgba>; 2],
    pub(crate) prev_num_chans: i32,
    pub(crate) num_tex_gens: u32,
    pub(crate) current_tev: u32,
    // lobj.c
    pub lights: Vec<ActiveLight>,
    pub lightmask_diffuse: u32,
    pub lightmask_specular: u32,
    pub lightmask_alpha: u32,
    pub lightmask_attnfunc: u32,
    // pobj.c: HSD_PObjGetMtxMark slots 0/1 (obj = joint index).
    pub(crate) mtx_mark: [(Option<usize>, u32); 2],
    pub(crate) prev_vtxdesc: Option<(usize, u32)>,
    pub(crate) prev_array: Option<(usize, u32)>,
    /// pobj.rs: per PObj descriptor, the largest index each attribute's display list uses.
    pub(crate) extents: std::collections::HashMap<(usize, u32), Vec<(u32, u32)>>,
    /// tobj.c's 	obj_head reduced to what pobj.c's GetSetupFlags asks of it: bit n set when the
    /// current MObj has a TObj with 	obj_coord n (_HSD_TObjGetCurrentByType).
    pub tobj_kinds: u32,
    /// Draw counters (what was drawn and what was skipped as unported).
    pub stats: DrawStats,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawStats {
    pub pobjs: u32,
    pub envelope_pobjs: u32,
    pub shape_anim_skipped: u32,
    pub tobjs: u32,
    pub tev_stages: u32,
}

impl Default for HsdRender {
    fn default() -> Self {
        Self::new()
    }
}

impl HsdRender {
    pub fn new() -> Self {
        HsdRender {
            cull_mode: -1,
            blend: None,
            zmode: None,
            alpha_cmp: None,
            color_update: None,
            alpha_update: None,
            dst_alpha: None,
            before_tex: None,
            dither: None,
            matstate: MatState::default(),
            prev_amb: [None; 2],
            prev_mat: [None; 2],
            prev_num_chans: -1,
            num_tex_gens: 0,
            current_tev: 0,
            lights: Vec::new(),
            lightmask_diffuse: 0,
            lightmask_specular: 0,
            lightmask_alpha: 0,
            lightmask_attnfunc: 0,
            mtx_mark: [(None, 0); 2],
            prev_vtxdesc: None,
            prev_array: None,
            extents: Default::default(),
            tobj_kinds: 0,
            stats: DrawStats::default(),
        }
    }

    /// `HSD_StateInvalidate(HSD_STATE_ALL)`: the start of a frame (the GX state is unknown).
    pub fn invalidate_all(&mut self) {
        // _HSD_StateInvalidatePrimitive / RenderMode / VtxAttr / ColorChannel / TevStage / TexCoordGen.
        self.cull_mode = -1;
        self.blend = None;
        self.zmode = None;
        self.alpha_cmp = None;
        self.color_update = None;
        self.alpha_update = None;
        self.dst_alpha = None;
        self.before_tex = None;
        self.dither = None;
        self.prev_amb = [None; 2];
        self.prev_mat = [None; 2];
        self.prev_num_chans = -1;
        self.num_tex_gens = 0;
        self.current_tev = 0;
        self.prev_vtxdesc = None;
        self.prev_array = None;
        self.mtx_mark = [(None, 0); 2];
    }

    /// `HSD_StateSetCullMode`.
    pub fn state_set_cull_mode<G: GxBackend>(&mut self, g: &mut G, mode: u32) {
        if self.cull_mode != mode as i64 {
            g.set_cull_mode(mode);
            self.cull_mode = mode as i64;
        }
    }
    /// `HSD_StateSetBlendMode`.
    pub fn state_set_blend_mode<G: GxBackend>(&mut self, g: &mut G, ty: u32, src: u32, dst: u32, op: u32) {
        if self.blend != Some((ty, src, dst, op)) {
            g.set_blend_mode(ty, src, dst, op);
            self.blend = Some((ty, src, dst, op));
        }
    }
    /// `HSD_StateSetZMode`.
    pub fn state_set_z_mode<G: GxBackend>(&mut self, g: &mut G, enable: bool, func: u32, update: bool) {
        if self.zmode != Some((enable, func, update)) {
            g.set_z_mode(enable, func, update);
            self.zmode = Some((enable, func, update));
        }
    }
    /// `HSD_StateSetAlphaCompare`.
    pub fn state_set_alpha_compare<G: GxBackend>(&mut self, g: &mut G, c0: u32, r0: u8, op: u32, c1: u32, r1: u8) {
        if self.alpha_cmp != Some((c0, r0, op, c1, r1)) {
            g.set_alpha_compare(c0, r0, op, c1, r1);
            self.alpha_cmp = Some((c0, r0, op, c1, r1));
        }
    }
    /// `HSD_StateSetColorUpdate`.
    pub fn state_set_color_update<G: GxBackend>(&mut self, g: &mut G, on: bool) {
        if self.color_update != Some(on) {
            g.set_color_update(on);
            self.color_update = Some(on);
        }
    }
    /// `HSD_StateSetAlphaUpdate`.
    pub fn state_set_alpha_update<G: GxBackend>(&mut self, g: &mut G, on: bool) {
        if self.alpha_update != Some(on) {
            g.set_alpha_update(on);
            self.alpha_update = Some(on);
        }
    }
    /// `HSD_StateSetDstAlpha`.
    pub fn state_set_dst_alpha<G: GxBackend>(&mut self, g: &mut G, on: bool, a: u8) {
        if self.dst_alpha != Some((on, a)) {
            g.set_dst_alpha(on, a);
            self.dst_alpha = Some((on, a));
        }
    }
    /// `HSD_StateSetZCompLoc`.
    pub fn state_set_z_comp_loc<G: GxBackend>(&mut self, g: &mut G, before_tex: bool) {
        if self.before_tex != Some(before_tex) {
            g.set_z_comp_loc(before_tex);
            self.before_tex = Some(before_tex);
        }
    }
    /// `HSD_StateSetDither`.
    pub fn state_set_dither<G: GxBackend>(&mut self, g: &mut G, on: bool) {
        if self.dither != Some(on) {
            g.set_dither(on);
            self.dither = Some(on);
        }
    }

    /// `HSD_SetMaterialColor`.
    pub fn set_material_color(&mut self, ambient: Rgba, diffuse: Rgba, specular: Rgba, alpha: f32) {
        self.matstate.ambient = ambient;
        self.matstate.diffuse = diffuse;
        self.matstate.specular = specular;
        // `matstate.alpha = 255.0 * alpha` (double multiply, then the u8 store's conversion).
        self.matstate.alpha = crate::cabi::f2u8(255.0f64 * alpha as f64);
    }
    /// `HSD_SetMaterialShininess`.
    pub fn set_material_shininess(&mut self, s: f32) {
        self.matstate.shininess = s;
    }

    /// `HSD_SetupChannelMode`.
    pub fn setup_channel_mode<G: GxBackend>(&mut self, g: &mut G, rendermode: u32) {
        let spec = rendermode & 8 != 0;
        self.state_set_num_chans(g, if spec { 2 } else { 1 });
        if spec {
            // HSD_State_80405A38: COLOR1A1, specular lights.
            let ch = HsdChan {
                chan: GX_COLOR1A1,
                amb_color: [0; 4],
                mat_color: [0xFF; 4],
                enable: true,
                amb_src: GX_SRC_REG,
                mat_src: GX_SRC_REG,
                light_mask: self.lightmask_specular,
                diff_fn: GX_DF_CLAMP,
                attn_fn: GX_AF_SPEC,
            };
            self.setup_channel(g, &ch);
            // HSD_LObjSetup per active light with matstate.shininess (diffuse colour reload only).
            let lights = self.lights.clone();
            for l in &lights {
                self.lobj_setup(g, l);
            }
        }
        match rendermode & 7 {
            2 => {
                // HSD_State_80405A38_30: vertex colour, unlit.
                let ch = HsdChan {
                    chan: GX_COLOR0A0,
                    amb_color: [0; 4],
                    mat_color: [0; 4],
                    enable: false,
                    amb_src: GX_SRC_REG,
                    mat_src: GX_SRC_VTX,
                    light_mask: 0,
                    diff_fn: GX_DF_CLAMP,
                    attn_fn: GX_AF_NONE,
                };
                self.setup_channel(g, &ch);
            }
            4 => {
                // HSD_State_80405A38_60 (COLOR0, lit) + _90 / _C0 (ALPHA0).
                let ambient = self.lights.iter().find(|l| l.flags & 3 == 0).copied();
                let amb_color = match ambient {
                    Some(a) if a.flags & 4 != 0 => mul_color(self.matstate.ambient, a.color),
                    _ => [0, 0, 0, 0xFF],
                };
                let c60 = HsdChan {
                    chan: GX_COLOR0,
                    amb_color,
                    mat_color: [0xFF; 4],
                    enable: true,
                    amb_src: GX_SRC_REG,
                    mat_src: GX_SRC_REG,
                    light_mask: self.lightmask_diffuse,
                    diff_fn: GX_DF_CLAMP,
                    attn_fn: GX_AF_SPOT,
                };
                self.setup_channel(g, &c60);
                let mask_a = self.lightmask_alpha;
                let mut c90 = HsdChan {
                    chan: GX_ALPHA0,
                    amb_color: [0, 0, 0, 0xFF],
                    mat_color: [0, 0, 0, 0xFF],
                    enable: true,
                    amb_src: GX_SRC_REG,
                    mat_src: GX_SRC_REG,
                    light_mask: mask_a,
                    diff_fn: GX_DF_CLAMP,
                    attn_fn: GX_AF_SPOT,
                };
                if let Some(a) = ambient.filter(|a| a.flags & 0x10 != 0) {
                    c90.amb_color[3] = a.color[3];
                    self.setup_channel(g, &c90);
                } else if mask_a != 0 {
                    c90.amb_color[3] = 0;
                    self.setup_channel(g, &c90);
                } else {
                    let cc0 = HsdChan {
                        chan: GX_ALPHA0,
                        amb_color: [0; 4],
                        mat_color: [0, 0, 0, 0xFF],
                        enable: false,
                        amb_src: GX_SRC_REG,
                        mat_src: GX_SRC_REG,
                        light_mask: 0,
                        diff_fn: GX_DF_NONE,
                        attn_fn: GX_AF_NONE,
                    };
                    self.setup_channel(g, &cc0);
                }
            }
            _ => {
                // HSD_State_80405A38_F0: constant colour (chan unlit, material white).
                let ch = HsdChan {
                    chan: GX_COLOR0A0,
                    amb_color: [0; 4],
                    mat_color: [0xFF; 4],
                    enable: false,
                    amb_src: GX_SRC_REG,
                    mat_src: GX_SRC_REG,
                    light_mask: 0,
                    diff_fn: GX_DF_CLAMP,
                    attn_fn: GX_AF_NONE,
                };
                self.setup_channel(g, &ch);
            }
        }
    }

    /// `HSD_SetupPEMode`. `pe` is the HSD_PEDesc's 12 bytes.
    pub fn setup_pe_mode<G: GxBackend>(&mut self, g: &mut G, flags: u32, pe: Option<[u8; 12]>) {
        if let Some(pe) = pe {
            let f = pe[0];
            self.state_set_color_update(g, f & 1 != 0);
            self.state_set_alpha_update(g, f & 2 != 0);
            self.state_set_dst_alpha(g, f & 4 != 0, pe[3]);
            self.state_set_blend_mode(g, pe[4] as u32, pe[5] as u32, pe[6] as u32, pe[7] as u32);
            self.state_set_z_mode(g, f & 0x10 != 0, pe[8] as u32, f & 0x20 != 0);
            self.state_set_z_comp_loc(g, f & 8 != 0);
            self.state_set_alpha_compare(g, pe[9] as u32, pe[1], pe[10] as u32, pe[11] as u32, pe[2]);
            self.state_set_dither(g, f & 0x40 != 0);
            return;
        }
        self.state_set_color_update(g, true);
        self.state_set_alpha_update(g, false);
        self.state_set_dst_alpha(g, false, 0);
        self.state_set_blend_mode(g, (flags & 0x4000_0000 != 0) as u32, 4, 5, 0xF);
        let zupd = flags & 0x2000_0000 == 0;
        self.state_set_z_mode(g, true, if flags & 0x0800_0000 != 0 { 7 } else { 3 }, zupd);
        if flags & 0x2000_0000 == 0 && flags & 0x4000_0000 != 0 {
            self.state_set_z_comp_loc(g, false);
            self.state_set_alpha_compare(g, 4, 0, 0, 4, 0);
        } else {
            self.state_set_z_comp_loc(g, true);
            self.state_set_alpha_compare(g, 7, 0, 0, 7, 0);
        }
        self.state_set_dither(g, false);
    }

    /// `setupTevMode_last` (static).
    fn setup_tev_mode_last<G: GxBackend>(&mut self, g: &mut G) {
        if self.current_tev == 0 {
            let stage = self.state_assign_tev();
            // HSD_SetupTevStage with flags 0: TevOrder + TevOp(PASSCLR) + swap 0.
            g.set_tev_order(stage, 0xFF, GX_TEXMAP_NULL, GX_COLOR0A0);
            g.set_tev_op(stage, GX_PASSCLR);
            g.set_tev_swap_mode(stage, GX_TEV_SWAP0, GX_TEV_SWAP0);
        }
    }

    /// `HSD_SetupRenderModeWithCustomPE`. (`HSD_SetTevRegAll`: the TEV registers are written by
    /// the stages that use them, tu/mobj.rs.)
    pub fn setup_render_mode_with_custom_pe<G: GxBackend>(&mut self, g: &mut G, rendermode: u32, pe: Option<[u8; 12]>) {
        self.setup_tev_mode_last(g);
        self.setup_pe_mode(g, rendermode, pe);
        self.state_set_num_tev_stages(g);
        self.state_set_num_tex_gens(g);
        self.setup_channel_mode(g, rendermode);
    }

    /// `HSD_SetupRenderMode`.
    pub fn setup_render_mode<G: GxBackend>(&mut self, g: &mut G, rendermode: u32) {
        self.setup_render_mode_with_custom_pe(g, rendermode, None);
    }
}

/// `HSD_MulColor` (util.c): per channel `a * b / 255`.
pub fn mul_color(a: Rgba, b: Rgba) -> Rgba {
    let m = |x: u8, y: u8| ((x as u32 * y as u32) / 255) as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}
