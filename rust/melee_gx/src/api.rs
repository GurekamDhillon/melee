//! The GX API the game calls, a port of Aurora's `lib/dolphin/gx/GX*.cpp` encoders and the `__GXData`
//! shadow registers (`__gx.h`). Every call writes the same register commands Aurora writes (BP, CP,
//! XF, `GX_AURORA` subcommands) into the FIFO, big-endian; [`Gx::drain`] decodes them into
//! [`GxState`]. Keeping the hardware encode/decode split (instead of setting decoded state directly)
//! keeps every SDK quirk Aurora has (channel ID remapping in `GXSetTevOrder`, the cull-mode swap,
//! dirty-state flushes on `GXBegin`, BP register dedupe) and lets display lists recorded with
//! `begin_display_list` replay through the same decoder.
//!
//! Pointers become [`GxData`] handles (see [`crate::resources`]). Texture, TLUT and light objects
//! are native structs with their GameCube SDK layout recorded as [`GcLayout`] metadata.
use crate::consts::*;
use crate::resources::{GxData, Resources};
use crate::state::{GxState, TexObj, TlutObj};

/// `GXColor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GxColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl GxColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
melee_core::gc_layout!(GxColor, "_GXColor", size 0x4, align 1, [
    r @ 0x0: 1 "u8",
    g @ 0x1: 1 "u8",
    b @ 0x2: 1 "u8",
    a @ 0x3: 1 "u8",
]);

/// `GXColorS10`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GxColorS10 {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

/// `GXTexObj` (Aurora's `GXTexObj_`). On the GameCube the SDK's private `__GXTexObjInt` lives in a
/// 0x20-byte `GXTexObj` (melee_types: `_GXTexObj`, 0x40 in this decomp's header); the offsets below
/// are the SDK's (mode0, mode1, image0, image3, userData, fmt, tlutName, loadCnt, loadFmt, flags).
pub type GxTexObj = TexObj;
melee_core::gc_layout!(TexObj, "_GXTexObj", size 0x40, align 4, [
    mode0 @ 0x0: 4 "u32",
    mode1 @ 0x4: 4 "u32",
    image0 @ 0x8: 4 "u32",
    image3 @ 0xC: 4 "u32",
    data @ 0x10: 4 "void*",
    format @ 0x14: 4 "GXTexFmt",
    tlut @ 0x18: 4 "u32",
    flags @ 0x1F: 1 "u8",
]);

/// `GXTlutObj` (Aurora's `GXTlutObj_`); GC SDK `__GXTlutObjInt` offsets.
pub type GxTlutObj = TlutObj;
melee_core::gc_layout!(TlutObj, "_GXTlutObj", size 0x28, align 4, [
    tlut @ 0x0: 4 "u32",
    load_tlut0 @ 0x4: 4 "u32",
    num_entries @ 0x8: 2 "u16",
]);

/// `GXLightObj` (Aurora's `GXLightObj_`); GC SDK `__GXLightObjInt` offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GxLightObj {
    pub color: GxColor,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub k0: f32,
    pub k1: f32,
    pub k2: f32,
    pub px: f32,
    pub py: f32,
    pub pz: f32,
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
}
impl Default for GxLightObj {
    fn default() -> Self {
        Self {
            color: GxColor::default(),
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            k0: 1.0,
            k1: 0.0,
            k2: 0.0,
            px: 0.0,
            py: 0.0,
            pz: 0.0,
            nx: 0.0,
            ny: 0.0,
            nz: 0.0,
        }
    }
}
melee_core::gc_layout!(GxLightObj, "_GXLightObj", size 0x40, align 4, [
    color @ 0xC: 4 "GXColor",
    a0 @ 0x10: 4 "f32",
    a1 @ 0x14: 4 "f32",
    a2 @ 0x18: 4 "f32",
    k0 @ 0x1C: 4 "f32",
    k1 @ 0x20: 4 "f32",
    k2 @ 0x24: 4 "f32",
    px @ 0x28: 4 "f32",
    py @ 0x2C: 4 "f32",
    pz @ 0x30: 4 "f32",
    nx @ 0x34: 4 "f32",
    ny @ 0x38: 4 "f32",
    nz @ 0x3C: 4 "f32",
]);

/// `GXVtxDescList` / `GXVtxAttrFmtList` entries.
#[derive(Clone, Copy, Debug)]
pub struct VtxDesc {
    pub attr: u32,
    pub ty: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct VtxAttrFmtEntry {
    pub attr: u32,
    pub cnt: u32,
    pub ty: u32,
    pub frac: u8,
}

/// Aurora `__GXData_struct` (only the fields the encoders read back).
#[derive(Clone, Debug, Default)]
struct Shadow {
    v_num: u16,
    bp_sent: u16,
    vcd_lo: u32,
    vcd_hi: u32,
    vat_a: [u32; 8],
    vat_b: [u32; 8],
    vat_c: [u32; 8],
    lp_size: u32,
    mat_idx_a: u32,
    mat_idx_b: u32,
    amb_color: [u32; 2],
    mat_color: [u32; 2],
    su_ts0: [u32; 8],
    su_ts1: [u32; 8],
    tcs_man_enab: u8,
    su_scis0: u32,
    su_scis1: u32,
    tref: [u32; 8],
    iref: u32,
    bp_mask: u32,
    ind_tex_scale0: u32,
    ind_tex_scale1: u32,
    tevc: [u32; 16],
    teva: [u32; 16],
    tev_ksel: [u32; 8],
    cmode0: u32,
    cmode1: u32,
    zmode: u32,
    pe_ctrl: u32,
    gen_mode: u32,
    t_image0: [u32; 8],
    t_mode0: [u32; 8],
    texmap_id: [u32; 16],
    nrm_type: u32,
    has_nrms: bool,
    has_bi_nrms: bool,
    proj_type: u32,
    proj_mtx: [f32; 6],
    vp: [f32; 6],
    dl_save_context: bool,
    dirty_vat: u8,
    dirty_state: u32,
}

#[inline]
fn set_field(reg: &mut u32, size: u32, shift: u32, val: u32) {
    let mask = ((1u64 << size) - 1) as u32;
    *reg = (*reg & !(mask << shift)) | (val << shift);
}
#[inline]
fn get_field(reg: u32, size: u32, shift: u32) -> u32 {
    (reg >> shift) & (((1u64 << size) - 1) as u32)
}

const TEX_MODE0_IDS: [u32; 8] = [0x80, 0x81, 0x82, 0x83, 0xA0, 0xA1, 0xA2, 0xA3];
const TEX_MODE1_IDS: [u32; 8] = [0x84, 0x85, 0x86, 0x87, 0xA4, 0xA5, 0xA6, 0xA7];
const TEX_IMAGE0_IDS: [u32; 8] = [0x88, 0x89, 0x8A, 0x8B, 0xA8, 0xA9, 0xAA, 0xAB];
const TEX_IMAGE1_IDS: [u32; 8] = [0x8C, 0x8D, 0x8E, 0x8F, 0xAC, 0xAD, 0xAE, 0xAF];
const TEX_IMAGE2_IDS: [u32; 8] = [0x90, 0x91, 0x92, 0x93, 0xB0, 0xB1, 0xB2, 0xB3];
const TEX_IMAGE3_IDS: [u32; 8] = [0x94, 0x95, 0x96, 0x97, 0xB4, 0xB5, 0xB6, 0xB7];
const TEX_TLUT_IDS: [u32; 8] = [0x98, 0x99, 0x9A, 0x9B, 0xB8, 0xB9, 0xBA, 0xBB];
const GX2HW_FILT_CONV: [u32; 6] = [0x00, 0x04, 0x01, 0x05, 0x02, 0x06];

const AURORA_LOAD_VIEWPORT_RENDER: u16 = 0x0001;
const AURORA_LOAD_SCISSOR_RENDER: u16 = 0x0002;
const AURORA_LOAD_PROJECTION_FULL: u16 = 0x0003;
const AURORA_LOAD_ARRAYBASE: u16 = 0x0010;
const AURORA_LOAD_TEXOBJ: u16 = 0x0030;
const AURORA_LOAD_TLUT: u16 = 0x0031;
const AURORA_DRAW_SIZED: u16 = 0x0040;
const AURORA_DRAW_INDEXED: u16 = 0x0041;
const GX2_SET_POLYGON_OFFSET: u16 = 0x1000;
const CP_REG_ARRAYSTRIDE_ID: u8 = 0xB0;
const BP_REG_DRAWDONE: u32 = 0x45;
pub const GX_AUTO: u16 = 0xFFFF;

/// A frame decoded on the [`Gx::enable_async`] worker: its draws and the copy-clear the presenter needs.
pub struct AsyncFrame {
    pub draws: Vec<crate::draw::DrawCmd>,
    pub clear_color: crate::state::Vec4,
    pub clear_depth: u32,
}

enum Job {
    /// FIFO bytes and the handle table they refer to.
    Process(Vec<u8>, Resources),
    /// GXCopyDisp: hand the draws decoded so far over as a frame.
    End,
}

/// The decode thread: it owns the `GxState` while async decoding is on.
struct Worker {
    tx: std::sync::mpsc::Sender<Job>,
    frames: std::sync::mpsc::Receiver<AsyncFrame>,
    /// Emptied FIFO buffers coming back for reuse.
    pool: std::sync::mpsc::Receiver<Vec<u8>>,
}

/// The GX: shadow registers + FIFO + the decoded state and resources it drives.
pub struct Gx {
    worker: Option<Worker>,
    sh: Shadow,
    saved: Option<Shadow>,
    fifo: Vec<u8>,
    dl: Option<Vec<u8>>,
    pub state: GxState,
    pub res: Resources,
    next_tex_obj_id: u32,
    next_tlut_obj_id: u32,
    in_begin: bool,
    begin_auto: bool,
    begin_size_offset: usize,
}

impl Default for Gx {
    fn default() -> Self {
        Self::new()
    }
}

impl Gx {
    /// A GX after `GXInit` (and its FIFO drained).
    pub fn new() -> Self {
        let mut gx = Self {
            worker: None,
            sh: Shadow::default(),
            saved: None,
            fifo: Vec::with_capacity(1 << 16),
            dl: None,
            state: GxState::default(),
            res: Resources::default(),
            next_tex_obj_id: 1,
            next_tlut_obj_id: 1,
            in_begin: false,
            begin_auto: false,
            begin_size_offset: 0,
        };
        gx.init();
        gx.drain();
        gx
    }

    // ------------------------------------------------------------------ FIFO

    fn w8(&mut self, v: u8) {
        match &mut self.dl {
            Some(dl) => dl.push(v),
            None => self.fifo.push(v),
        }
    }
    fn wdata(&mut self, d: &[u8]) {
        match &mut self.dl {
            Some(dl) => dl.extend_from_slice(d),
            None => self.fifo.extend_from_slice(d),
        }
    }
    fn w16(&mut self, v: u16) {
        self.wdata(&v.to_be_bytes());
    }
    fn w32(&mut self, v: u32) {
        self.wdata(&v.to_be_bytes());
    }
    fn w64(&mut self, v: u64) {
        self.wdata(&v.to_be_bytes());
    }
    fn wf32(&mut self, v: f32) {
        self.w32(v.to_bits());
    }
    fn write_ras_reg(&mut self, v: u32) {
        self.w8(GX_LOAD_BP_REG);
        self.w32(v);
    }
    fn write_xf_reg(&mut self, addr: u32, v: u32) {
        self.w8(0x10);
        self.w32(0x1000 + addr);
        self.w32(v);
    }
    fn write_cp_reg(&mut self, addr: u8, v: u32) {
        self.w8(GX_LOAD_CP_REG);
        self.w8(addr);
        self.w32(v);
    }
    fn write_aurora(&mut self, cmd: u16) {
        self.w8(GX_AURORA);
        self.w16(cmd);
    }
    fn buffer_size(&self) -> usize {
        self.dl.as_ref().map_or(self.fifo.len(), |d| d.len())
    }

    /// Decode on a thread of its own from now on: [`Gx::drain`] sends the FIFO there, [`Gx::end_frame`] closes a
    /// frame and [`Gx::take_async_frame`] collects the newest finished one (older unpresented ones are dropped).
    /// The decoded state moves to the thread: `self.state` only mirrors the copy-clear of the last collected frame.
    /// `init` runs first on the new thread. Decoding is render-only: no game code reads `GxState`.
    pub fn enable_async(&mut self, init: fn()) {
        if self.worker.is_some() {
            return;
        }
        self.drain();
        let mut st = std::mem::take(&mut self.state);
        self.state.clear_color = st.clear_color;
        self.state.clear_depth = st.clear_depth;
        let (tx, jobs) = std::sync::mpsc::channel::<Job>();
        let (ftx, frames) = std::sync::mpsc::channel::<AsyncFrame>();
        let (ptx, pool) = std::sync::mpsc::channel::<Vec<u8>>();
        std::thread::Builder::new()
            .name("gx decode".into())
            .spawn(move || {
                init();
                while let Ok(job) = jobs.recv() {
                    match job {
                        Job::Process(mut buf, res) => {
                            st.process(&buf, &res);
                            drop(res);
                            buf.clear();
                            let _ = ptx.send(buf);
                        }
                        Job::End => {
                            st.warnings.clear();
                            let f = AsyncFrame { draws: st.take_draws(), clear_color: st.clear_color, clear_depth: st.clear_depth };
                            if ftx.send(f).is_err() {
                                break;
                            }
                        }
                    }
                }
            })
            .expect("gx decode thread");
        self.worker = Some(Worker { tx, frames, pool });
    }
    pub fn is_async(&self) -> bool {
        self.worker.is_some()
    }
    /// Async mode: close the current frame (see [`Gx::enable_async`]).
    pub fn end_frame(&mut self) {
        if let Some(w) = &self.worker {
            let _ = w.tx.send(Job::End);
        }
    }
    /// Async mode: the newest frame the worker finished since the last call (the copy-clear is mirrored into `state`).
    pub fn take_async_frame(&mut self) -> Option<Vec<crate::draw::DrawCmd>> {
        let w = self.worker.as_ref()?;
        let mut last = None;
        while let Ok(f) = w.frames.try_recv() {
            last = Some(f);
        }
        let f = last?;
        self.state.clear_color = f.clear_color;
        self.state.clear_depth = f.clear_depth;
        Some(f.draws)
    }

    /// Decode everything written so far (Aurora `fifo::drain`).
    pub fn drain(&mut self) {
        if let Some(w) = &self.worker {
            let next = w.pool.try_recv().unwrap_or_else(|_| Vec::with_capacity(1 << 16));
            let buf = std::mem::replace(&mut self.fifo, next);
            let res = std::mem::take(&mut self.res);
            let _ = w.tx.send(Job::Process(buf, res));
            return;
        }
        let buf = std::mem::take(&mut self.fifo);
        self.state.process(&buf, &self.res);
        self.res.clear();
        self.fifo = buf;
        self.fifo.clear();
    }

    /// The raw FIFO bytes not yet drained (for tests / tools).
    pub fn pending(&self) -> &[u8] {
        &self.fifo
    }

    // ------------------------------------------------------------------ GXManage.cpp

    fn init(&mut self) {
        self.sh = Shadow { dl_save_context: true, ..Default::default() };
        let s = &mut self.sh;
        set_field(&mut s.gen_mode, 8, 24, 0x00);
        s.bp_mask = 0xFF;
        set_field(&mut s.bp_mask, 8, 24, 0x0F);
        set_field(&mut s.lp_size, 8, 24, 0x22);
        for i in 0..16 {
            s.tevc[i] = 0;
            s.teva[i] = 0;
            s.tref[i / 2] = 0;
            s.texmap_id[i] = GX_TEXMAP_NULL;
            set_field(&mut s.tevc[i], 8, 24, 0xC0 + i as u32 * 2);
            set_field(&mut s.teva[i], 8, 24, 0xC1 + i as u32 * 2);
            set_field(&mut s.tev_ksel[i / 2], 8, 24, 0xF6 + i as u32 / 2);
            set_field(&mut s.tref[i / 2], 8, 24, 0x28 + i as u32 / 2);
        }
        set_field(&mut s.iref, 8, 24, 0x27);
        for i in 0..8 {
            set_field(&mut s.su_ts0[i], 8, 24, 0x30 + i as u32 * 2);
            set_field(&mut s.su_ts1[i], 8, 24, 0x31 + i as u32 * 2);
        }
        set_field(&mut s.su_scis0, 8, 24, 0x20);
        set_field(&mut s.su_scis1, 8, 24, 0x21);
        set_field(&mut s.cmode0, 8, 24, 0x41);
        set_field(&mut s.cmode1, 8, 24, 0x42);
        set_field(&mut s.zmode, 8, 24, 0x40);
        set_field(&mut s.pe_ctrl, 8, 24, 0x43);
        set_field(&mut s.ind_tex_scale0, 8, 24, 0x25);
        set_field(&mut s.ind_tex_scale1, 8, 24, 0x26);
        s.dirty_state = 0;
        s.dirty_vat = 0;
        for i in 0..8 {
            set_field(&mut self.sh.vat_a[i], 1, 30, 1);
            set_field(&mut self.sh.vat_b[i], 1, 31, 1);
            self.w8(8);
            self.w8(i as u8 | 0x80);
            let v = self.sh.vat_b[i];
            self.w32(v);
        }
        {
            let mut reg1 = 0;
            for b in 0..6 {
                set_field(&mut reg1, 1, b, 1);
            }
            self.write_xf_reg(0, reg1);
            self.write_xf_reg(0x12, 1);
        }
        {
            let mut reg = 0;
            for b in 0..4 {
                set_field(&mut reg, 1, b, 1);
            }
            set_field(&mut reg, 8, 24, 0x58);
            self.write_ras_reg(reg);
        }
        let clear = GxColor::new(64, 64, 64, 255);
        let black = GxColor::new(0, 0, 0, 0);
        let white = GxColor::new(255, 255, 255, 255);
        self.set_copy_clear(clear, 0xFF_FFFF);
        for i in 0..8 {
            self.set_tex_coord_gen(GX_TEXCOORD0 + i, GX_TG_MTX2x4, GX_TG_TEX0 + i, 0x3C);
        }
        self.set_num_tex_gens(1);
        self.clear_vtx_desc();
        self.invalidate_vtx_cache();
        self.set_line_width(6, GX_TO_ZERO);
        self.set_point_size(6, GX_TO_ZERO);
        for i in 0..8 {
            self.enable_tex_offsets(GX_TEXCOORD0 + i, false, false);
        }
        let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        self.load_pos_mtx_imm(&id, GX_PNMTX0);
        self.load_nrm_mtx_imm(&id, GX_PNMTX0);
        self.set_current_mtx(GX_PNMTX0);
        self.load_tex_mtx_imm(&id, GX_IDENTITY, GX_MTX3x4);
        self.load_tex_mtx_imm(&id, GX_PTIDENTITY, GX_MTX3x4);
        self.set_viewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
        self.set_co_planar(false);
        self.set_cull_mode(GX_CULL_BACK);
        self.set_clip_mode(GX_CLIP_ENABLE);
        self.set_scissor(0, 0, 640, 480);
        self.set_num_chans(0);
        self.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
        self.set_chan_amb_color(GX_COLOR0A0, black);
        self.set_chan_mat_color(GX_COLOR0A0, white);
        self.set_chan_ctrl(GX_COLOR1A1, false, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
        self.set_chan_amb_color(GX_COLOR1A1, black);
        self.set_chan_mat_color(GX_COLOR1A1, white);
        for i in 0..8 {
            self.set_tev_order(GX_TEVSTAGE0 + i, GX_TEXCOORD0 + i, GX_TEXMAP0 + i, GX_COLOR0A0);
        }
        for i in 8..16 {
            self.set_tev_order(GX_TEVSTAGE0 + i, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR_NULL);
        }
        self.set_num_tev_stages(1);
        self.set_tev_op(GX_TEVSTAGE0, GX_REPLACE);
        self.set_alpha_compare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
        for i in 0..16 {
            self.set_tev_kcolor_sel(i, GX_TEV_KCSEL_1_4);
            self.set_tev_kalpha_sel(i, GX_TEV_KASEL_1);
            self.set_tev_swap_mode(i, GX_TEV_SWAP0, GX_TEV_SWAP0);
        }
        self.set_tev_swap_mode_table(GX_TEV_SWAP0, GX_CH_RED, GX_CH_GREEN, GX_CH_BLUE, GX_CH_ALPHA);
        self.set_tev_swap_mode_table(GX_TEV_SWAP1, GX_CH_RED, GX_CH_RED, GX_CH_RED, GX_CH_ALPHA);
        self.set_tev_swap_mode_table(GX_TEV_SWAP2, GX_CH_GREEN, GX_CH_GREEN, GX_CH_GREEN, GX_CH_ALPHA);
        self.set_tev_swap_mode_table(GX_TEV_SWAP3, GX_CH_BLUE, GX_CH_BLUE, GX_CH_BLUE, GX_CH_ALPHA);
        for i in 0..16 {
            self.set_tev_direct(i);
        }
        self.set_num_ind_stages(0);
        for i in 0..4 {
            self.set_ind_tex_coord_scale(i, GX_ITS_1, GX_ITS_1);
        }
        self.set_fog(GX_FOG_NONE, 0.0, 1.0, 0.1, 1.0, black);
        self.set_fog_range_adj(false, 0, None);
        self.set_blend_mode(GX_BM_NONE, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        self.set_color_update(true);
        self.set_alpha_update(true);
        self.set_z_mode(true, GX_LEQUAL, true);
        self.set_z_comp_loc(true);
        self.set_dither(true);
        self.set_dst_alpha(false, 0);
        self.set_pixel_fmt(GX_PF_RGB8_Z24, GX_ZC_LINEAR);
        self.set_field_mask(true, true);
        self.set_field_mode(false, false);
    }

    /// `GXFlush`.
    pub fn flush(&mut self) {
        if self.sh.dirty_state != 0 {
            self.set_dirty_state();
        }
    }

    /// `GXDrawDone`: flush, then decode everything.
    pub fn draw_done(&mut self) {
        self.flush();
        self.write_ras_reg(BP_REG_DRAWDONE << 24 | 2);
        self.drain();
    }

    pub fn pix_mode_sync(&mut self) {
        let v = self.sh.pe_ctrl;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn tex_mode_sync(&mut self) {
        self.write_ras_reg(0x6300_0000);
        self.sh.bp_sent = 1;
    }

    fn set_dirty_state(&mut self) {
        let d = self.sh.dirty_state;
        if d & 1 != 0 {
            self.set_su_tex_regs();
        }
        if d & 2 != 0 {
            self.update_bp_mask();
        }
        if d & 4 != 0 {
            self.set_gen_mode();
        }
        if d & 8 != 0 {
            self.set_vcd();
        }
        if d & 0x10 != 0 {
            self.set_vat();
        }
        self.sh.dirty_state = 0;
    }

    fn set_gen_mode(&mut self) {
        let v = self.sh.gen_mode;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }

    fn send_flush_prim(&mut self) {
        self.sh.bp_sent = 0;
    }

    fn set_vcd(&mut self) {
        let (lo, hi) = (self.sh.vcd_lo, self.sh.vcd_hi);
        self.write_cp_reg(0x50, lo);
        self.write_cp_reg(0x60, hi);
        let mut ncols = (get_field(lo, 2, 13) != 0) as u32;
        ncols += (get_field(lo, 2, 15) != 0) as u32;
        let nnrm = if self.sh.has_bi_nrms {
            2
        } else if self.sh.has_nrms {
            1
        } else {
            0
        };
        let mut ntex = 0;
        for i in 0..8 {
            ntex += (get_field(hi, 2, i * 2) != 0) as u32;
        }
        self.write_xf_reg(8, ncols | (nnrm << 2) | (ntex << 4));
        self.sh.bp_sent = 0;
        // vLim (flush-prim size) only matters for the dummy flush primitive Aurora never sends.
    }

    fn set_vat(&mut self) {
        for i in 0..8u8 {
            if self.sh.dirty_vat & (1 << i) != 0 {
                let (a, b, c) = (self.sh.vat_a[i as usize], self.sh.vat_b[i as usize], self.sh.vat_c[i as usize]);
                self.write_cp_reg(i | 0x70, a);
                self.write_cp_reg(i | 0x80, b);
                self.write_cp_reg(i | 0x90, c);
            }
        }
        self.sh.dirty_vat = 0;
    }

    fn set_su_regs(&mut self, tmap: usize, tcoord: usize) {
        let w = get_field(self.sh.t_image0[tmap], 10, 0);
        let h = get_field(self.sh.t_image0[tmap], 10, 10);
        set_field(&mut self.sh.su_ts0[tcoord], 16, 0, w);
        set_field(&mut self.sh.su_ts1[tcoord], 16, 0, h);
        let sb = (get_field(self.sh.t_mode0[tmap], 2, 0) == 1) as u32;
        let tb = (get_field(self.sh.t_mode0[tmap], 2, 2) == 1) as u32;
        set_field(&mut self.sh.su_ts0[tcoord], 1, 16, sb);
        set_field(&mut self.sh.su_ts1[tcoord], 1, 16, tb);
        let (a, b) = (self.sh.su_ts0[tcoord], self.sh.su_ts1[tcoord]);
        self.write_ras_reg(a);
        self.write_ras_reg(b);
        self.sh.bp_sent = 1;
    }

    fn set_su_tex_regs(&mut self) {
        if self.sh.tcs_man_enab == 0xFF {
            return;
        }
        let nstages = get_field(self.sh.gen_mode, 4, 10) + 1;
        let nind = get_field(self.sh.gen_mode, 3, 16);
        for i in 0..nind {
            let iref = self.sh.iref;
            let (tmap, coord) = match i {
                0 => (get_field(iref, 3, 0), get_field(iref, 3, 3)),
                1 => (get_field(iref, 3, 6), get_field(iref, 3, 9)),
                2 => (get_field(iref, 3, 12), get_field(iref, 3, 15)),
                _ => (get_field(iref, 3, 18), get_field(iref, 3, 21)),
            };
            if self.sh.tcs_man_enab & (1 << coord) == 0 {
                self.set_su_regs(tmap as usize, coord as usize);
            }
        }
        for i in 0..nstages as usize {
            let tref = self.sh.tref[i / 2];
            let map = self.sh.texmap_id[i];
            let tmap = map & !0x100;
            let coord = if i & 1 != 0 { get_field(tref, 3, 15) } else { get_field(tref, 3, 3) };
            if tmap != 0xFF && self.sh.tcs_man_enab & (1 << coord) == 0 {
                // texmap_id can be GX_TEXMAP_NULL (0xFF) or a real map < 8 here.
                self.set_su_regs(tmap as usize, coord as usize);
            }
        }
    }

    fn update_bp_mask(&mut self) {
        let nind = get_field(self.sh.gen_mode, 3, 16);
        let mut imask = 0u32;
        for i in 0..nind {
            imask |= 1 << get_field(self.sh.iref, 3, i * 6);
        }
        if self.sh.bp_mask as u8 != imask as u8 {
            set_field(&mut self.sh.bp_mask, 8, 0, imask);
            let v = self.sh.bp_mask;
            self.write_ras_reg(v);
            self.sh.bp_sent = 1;
        }
    }

    fn set_matrix_index(&mut self, attr: u32) {
        if attr < GX_VA_TEX4MTXIDX {
            let v = self.sh.mat_idx_a;
            self.write_cp_reg(0x30, v);
            self.write_xf_reg(24, v);
        } else {
            let v = self.sh.mat_idx_b;
            self.write_cp_reg(0x40, v);
            self.write_xf_reg(25, v);
        }
        self.sh.bp_sent = 0;
    }

    fn flush_texture_state(&mut self) {
        let v = self.sh.bp_mask;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }

    // ------------------------------------------------------------------ GXGeometry.cpp

    fn setvcdattr(&mut self, attr: u32, ty: u32) {
        let s = &mut self.sh;
        match attr {
            a if a <= GX_VA_TEX7MTXIDX => set_field(&mut s.vcd_lo, 1, a, ty),
            GX_VA_POS => set_field(&mut s.vcd_lo, 2, 9, ty),
            GX_VA_NRM => {
                s.has_nrms = ty != 0;
                if ty != GX_NONE {
                    s.nrm_type = ty;
                }
            }
            GX_VA_NBT => {
                s.has_bi_nrms = ty != 0;
                if ty != GX_NONE {
                    s.nrm_type = ty;
                }
            }
            GX_VA_CLR0 => set_field(&mut s.vcd_lo, 2, 13, ty),
            GX_VA_CLR1 => set_field(&mut s.vcd_lo, 2, 15, ty),
            a if (GX_VA_TEX0..=GX_VA_TEX7).contains(&a) => set_field(&mut s.vcd_hi, 2, (a - GX_VA_TEX0) * 2, ty),
            _ => {}
        }
    }

    fn finish_vcd(&mut self) {
        let s = &mut self.sh;
        let nt = if s.has_nrms || s.has_bi_nrms { s.nrm_type } else { 0 };
        set_field(&mut s.vcd_lo, 2, 11, nt);
        s.dirty_state |= 8;
    }

    pub fn set_vtx_desc(&mut self, attr: u32, ty: u32) {
        self.setvcdattr(attr, ty);
        self.finish_vcd();
    }
    pub fn set_vtx_descv(&mut self, list: &[VtxDesc]) {
        for d in list {
            if d.attr == GX_VA_NULL {
                break;
            }
            self.setvcdattr(d.attr, d.ty);
        }
        self.finish_vcd();
    }
    pub fn clear_vtx_desc(&mut self) {
        let s = &mut self.sh;
        s.vcd_lo = 0;
        set_field(&mut s.vcd_lo, 2, 9, 1);
        s.vcd_hi = 0;
        s.has_nrms = false;
        s.has_bi_nrms = false;
        s.dirty_state |= 8;
    }

    fn setvat(&mut self, fmt: usize, attr: u32, cnt: u32, ty: u32, frac: u8) {
        let (va, vb, vc) = {
            let s = &mut self.sh;
            (&mut s.vat_a[fmt] as *mut u32, &mut s.vat_b[fmt] as *mut u32, &mut s.vat_c[fmt] as *mut u32)
        };
        // SAFETY: three distinct array elements of self.sh, no other borrow alive.
        let (va, vb, vc) = unsafe { (&mut *va, &mut *vb, &mut *vc) };
        let f = frac as u32;
        match attr {
            GX_VA_POS => {
                set_field(va, 1, 0, cnt);
                set_field(va, 3, 1, ty);
                set_field(va, 5, 4, f);
            }
            GX_VA_NRM | GX_VA_NBT => {
                set_field(va, 3, 10, ty);
                if cnt == GX_NRM_NBT3 {
                    set_field(va, 1, 9, 1);
                    set_field(va, 1, 31, 1);
                } else {
                    set_field(va, 1, 9, cnt);
                    set_field(va, 1, 31, 0);
                }
            }
            GX_VA_CLR0 => {
                set_field(va, 1, 13, cnt);
                set_field(va, 3, 14, ty);
            }
            GX_VA_CLR1 => {
                set_field(va, 1, 17, cnt);
                set_field(va, 3, 18, ty);
            }
            GX_VA_TEX0 => {
                set_field(va, 1, 21, cnt);
                set_field(va, 3, 22, ty);
                set_field(va, 5, 25, f);
            }
            GX_VA_TEX1 => {
                set_field(vb, 1, 0, cnt);
                set_field(vb, 3, 1, ty);
                set_field(vb, 5, 4, f);
            }
            GX_VA_TEX2 => {
                set_field(vb, 1, 9, cnt);
                set_field(vb, 3, 10, ty);
                set_field(vb, 5, 13, f);
            }
            GX_VA_TEX3 => {
                set_field(vb, 1, 18, cnt);
                set_field(vb, 3, 19, ty);
                set_field(vb, 5, 22, f);
            }
            GX_VA_TEX4 => {
                set_field(vb, 1, 27, cnt);
                set_field(vb, 3, 28, ty);
                set_field(vc, 5, 0, f);
            }
            GX_VA_TEX5 => {
                set_field(vc, 1, 5, cnt);
                set_field(vc, 3, 6, ty);
                set_field(vc, 5, 9, f);
            }
            GX_VA_TEX6 => {
                set_field(vc, 1, 14, cnt);
                set_field(vc, 3, 15, ty);
                set_field(vc, 5, 18, f);
            }
            GX_VA_TEX7 => {
                set_field(vc, 1, 23, cnt);
                set_field(vc, 3, 24, ty);
                set_field(vc, 5, 27, f);
            }
            _ => {}
        }
    }

    pub fn set_vtx_attr_fmt(&mut self, fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8) {
        assert!(fmt < GX_MAX_VTXFMT, "invalid vtxfmt {fmt}");
        assert!((GX_VA_POS..GX_VA_MAX_ATTR).contains(&attr), "invalid attr {attr}");
        self.setvat(fmt as usize, attr, cnt, ty, frac);
        self.sh.dirty_state |= 0x10;
        self.sh.dirty_vat |= 1 << fmt;
    }
    pub fn set_vtx_attr_fmtv(&mut self, fmt: u32, list: &[VtxAttrFmtEntry]) {
        for e in list {
            if e.attr == GX_VA_NULL {
                break;
            }
            self.setvat(fmt as usize, e.attr, e.cnt, e.ty, e.frac);
        }
        self.sh.dirty_state |= 0x10;
        self.sh.dirty_vat |= 1 << fmt;
    }

    /// `GXSetArray(attr, data, size, stride, le)`.
    pub fn set_array(&mut self, attr: u32, data: Option<&GxData>, size: u32, stride: u8, le: bool) {
        let cp_attr = if attr == GX_VA_NBT { GX_VA_NRM } else { attr };
        let idx = cp_attr - GX_VA_POS;
        assert!(idx & !0xF == 0);
        let h = self.res.handle(data);
        self.write_aurora(AURORA_LOAD_ARRAYBASE | idx as u16);
        self.w64(h);
        self.w32(size);
        self.w8(le as u8);
        self.write_cp_reg(CP_REG_ARRAYSTRIDE_ID | idx as u8, stride as u32);
    }

    pub fn set_tex_coord_gen(&mut self, dst: u32, ty: u32, src: u32, mtx: u32) {
        self.set_tex_coord_gen2(dst, ty, src, mtx, false, GX_PTIDENTITY);
    }

    pub fn set_tex_coord_gen2(&mut self, dst: u32, ty: u32, src: u32, mtx: u32, normalize: bool, post_mtx: u32) {
        assert!(dst <= GX_TEXCOORD7, "invalid tex coord {dst}");
        let mut reg = 0u32;
        let (row, form) = match src {
            GX_TG_POS => (0, 1),
            GX_TG_NRM => (1, 1),
            GX_TG_BINRM => (3, 1),
            GX_TG_TANGENT => (4, 1),
            GX_TG_COLOR0 | GX_TG_COLOR1 => (2, 0),
            s if (GX_TG_TEX0..=GX_TG_TEX7).contains(&s) => (5 + (s - GX_TG_TEX0), 0),
            _ => (5, 0),
        };
        match ty {
            GX_TG_MTX2x4 | GX_TG_MTX3x4 => {
                set_field(&mut reg, 1, 1, (ty == GX_TG_MTX3x4) as u32);
                set_field(&mut reg, 1, 2, form);
                set_field(&mut reg, 3, 4, 0);
                set_field(&mut reg, 5, 7, row);
            }
            GX_TG_SRTG => {
                set_field(&mut reg, 1, 1, 0);
                set_field(&mut reg, 1, 2, form);
                set_field(&mut reg, 3, 4, if src == GX_TG_COLOR0 { 2 } else { 3 });
                set_field(&mut reg, 5, 7, 2);
            }
            t if (2..=9).contains(&t) => {
                set_field(&mut reg, 1, 1, 0);
                set_field(&mut reg, 1, 2, form);
                set_field(&mut reg, 3, 4, 1);
                set_field(&mut reg, 5, 7, row);
                set_field(&mut reg, 3, 12, src.wrapping_sub(12));
                set_field(&mut reg, 3, 15, t - 2);
            }
            _ => {}
        }
        self.write_xf_reg(dst + 0x40, reg);
        let mut post = 0u32;
        set_field(&mut post, 6, 0, post_mtx.wrapping_sub(64));
        set_field(&mut post, 1, 8, normalize as u32);
        self.write_xf_reg(dst + 0x50, post);
        let s = &mut self.sh;
        match dst {
            0 => set_field(&mut s.mat_idx_a, 6, 6, mtx),
            1 => set_field(&mut s.mat_idx_a, 6, 12, mtx),
            2 => set_field(&mut s.mat_idx_a, 6, 18, mtx),
            3 => set_field(&mut s.mat_idx_a, 6, 24, mtx),
            4 => set_field(&mut s.mat_idx_b, 6, 0, mtx),
            5 => set_field(&mut s.mat_idx_b, 6, 6, mtx),
            6 => set_field(&mut s.mat_idx_b, 6, 12, mtx),
            _ => set_field(&mut s.mat_idx_b, 6, 18, mtx),
        }
        self.set_matrix_index(dst + 1);
    }

    pub fn set_num_tex_gens(&mut self, n: u8) {
        set_field(&mut self.sh.gen_mode, 4, 0, n as u32);
        self.write_xf_reg(0x3F, n as u32);
        self.sh.dirty_state |= 4;
    }
    pub fn invalidate_vtx_cache(&mut self) {
        self.w8(0x48);
    }
    pub fn set_line_width(&mut self, w: u8, offs: u32) {
        set_field(&mut self.sh.lp_size, 8, 0, w as u32);
        set_field(&mut self.sh.lp_size, 3, 16, offs);
        let v = self.sh.lp_size;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_point_size(&mut self, s: u8, offs: u32) {
        set_field(&mut self.sh.lp_size, 8, 8, s as u32);
        set_field(&mut self.sh.lp_size, 3, 19, offs);
        let v = self.sh.lp_size;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn enable_tex_offsets(&mut self, coord: u32, line: bool, point: bool) {
        let c = coord as usize;
        set_field(&mut self.sh.su_ts0[c], 1, 18, line as u32);
        set_field(&mut self.sh.su_ts0[c], 1, 19, point as u32);
        let v = self.sh.su_ts0[c];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }

    // ------------------------------------------------------------------ GXCull.cpp

    pub fn set_scissor(&mut self, left: u32, top: u32, width: u32, height: u32) {
        let tp = top + 342;
        let lf = left + 342;
        let bm = tp + height - 1;
        let rt = lf + width - 1;
        let s = &mut self.sh;
        set_field(&mut s.su_scis0, 11, 0, tp);
        set_field(&mut s.su_scis0, 11, 12, lf);
        set_field(&mut s.su_scis1, 11, 0, bm);
        set_field(&mut s.su_scis1, 11, 12, rt);
        let (a, b) = (s.su_scis0, s.su_scis1);
        self.write_ras_reg(a);
        self.write_ras_reg(b);
        self.sh.bp_sent = 1;
    }
    pub fn set_cull_mode(&mut self, mode: u32) {
        let hw = match mode {
            GX_CULL_FRONT => GX_CULL_BACK,
            GX_CULL_BACK => GX_CULL_FRONT,
            m => m,
        };
        set_field(&mut self.sh.gen_mode, 2, 14, hw);
        self.sh.dirty_state |= 4;
    }
    pub fn set_clip_mode(&mut self, mode: u32) {
        self.write_xf_reg(5, mode);
        self.sh.bp_sent = 1;
    }
    pub fn set_co_planar(&mut self, enable: bool) {
        set_field(&mut self.sh.gen_mode, 1, 19, enable as u32);
        let mut reg = 0x0008000;
        set_field(&mut reg, 8, 24, 0xFE);
        set_field(&mut reg, 24, 0, 1 << 19);
        self.write_ras_reg(reg);
        let g = self.sh.gen_mode;
        self.write_ras_reg(g);
    }

    // ------------------------------------------------------------------ GXTev.cpp

    pub fn set_tev_op(&mut self, id: u32, mode: u32) {
        let (ic, ia) = if id != GX_TEVSTAGE0 { (GX_CC_CPREV, GX_CA_APREV) } else { (GX_CC_RASC, GX_CA_RASA) };
        match mode {
            GX_MODULATE => {
                self.set_tev_color_in(id, GX_CC_ZERO, GX_CC_TEXC, ic, GX_CC_ZERO);
                self.set_tev_alpha_in(id, GX_CA_ZERO, GX_CA_TEXA, ia, GX_CA_ZERO);
            }
            GX_DECAL => {
                self.set_tev_color_in(id, ic, GX_CC_TEXC, GX_CC_TEXA, GX_CC_ZERO);
                self.set_tev_alpha_in(id, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, ia);
            }
            GX_BLEND => {
                self.set_tev_color_in(id, ic, GX_CC_ONE, GX_CC_TEXC, GX_CC_ZERO);
                self.set_tev_alpha_in(id, GX_CA_ZERO, GX_CA_TEXA, ia, GX_CA_ZERO);
            }
            GX_REPLACE => {
                self.set_tev_color_in(id, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_TEXC);
                self.set_tev_alpha_in(id, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_TEXA);
            }
            GX_PASSCLR => {
                self.set_tev_color_in(id, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, ic);
                self.set_tev_alpha_in(id, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, ia);
            }
            _ => {}
        }
        self.set_tev_color_op(id, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
        self.set_tev_alpha_op(id, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
    }

    pub fn set_tev_color_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32) {
        let r = &mut self.sh.tevc[stage as usize];
        set_field(r, 4, 12, a);
        set_field(r, 4, 8, b);
        set_field(r, 4, 4, c);
        set_field(r, 4, 0, d);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_alpha_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32) {
        let r = &mut self.sh.teva[stage as usize];
        set_field(r, 3, 13, a);
        set_field(r, 3, 10, b);
        set_field(r, 3, 7, c);
        set_field(r, 3, 4, d);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    fn tev_op_bits(r: &mut u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32) {
        set_field(r, 1, 18, op & 1);
        if op <= 1 {
            set_field(r, 2, 20, scale);
            set_field(r, 2, 16, bias);
        } else {
            set_field(r, 2, 20, (op >> 1) & 3);
            set_field(r, 2, 16, 3);
        }
        set_field(r, 1, 19, clamp as u32);
        set_field(r, 2, 22, out);
    }
    pub fn set_tev_color_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32) {
        Self::tev_op_bits(&mut self.sh.tevc[stage as usize], op, bias, scale, clamp, out);
        let v = self.sh.tevc[stage as usize];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_alpha_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32) {
        Self::tev_op_bits(&mut self.sh.teva[stage as usize], op, bias, scale, clamp, out);
        let v = self.sh.teva[stage as usize];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_color(&mut self, id: u32, c: GxColor) {
        assert!(id < GX_MAX_TEVREG, "bad tevreg {id}");
        let mut ra = 0;
        set_field(&mut ra, 11, 0, c.r as u32);
        set_field(&mut ra, 11, 12, c.a as u32);
        set_field(&mut ra, 8, 24, 0xE0 + id * 2);
        let mut bg = 0;
        set_field(&mut bg, 11, 0, c.b as u32);
        set_field(&mut bg, 11, 12, c.g as u32);
        set_field(&mut bg, 8, 24, 0xE1 + id * 2);
        self.write_ras_reg(ra);
        self.write_ras_reg(bg);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_color_s10(&mut self, id: u32, c: GxColorS10) {
        let m = |v: i16| (v as i32 as u32) & 0x7FF;
        let mut ra = 0;
        set_field(&mut ra, 11, 0, m(c.r));
        set_field(&mut ra, 11, 12, m(c.a));
        set_field(&mut ra, 8, 24, 0xE0 + id * 2);
        let mut bg = 0;
        set_field(&mut bg, 11, 0, m(c.b));
        set_field(&mut bg, 11, 12, m(c.g));
        set_field(&mut bg, 8, 24, 0xE1 + id * 2);
        self.write_ras_reg(ra);
        self.write_ras_reg(bg);
        self.sh.bp_sent = 1;
    }
    pub fn set_alpha_compare(&mut self, comp0: u32, ref0: u8, op: u32, comp1: u32, ref1: u8) {
        let mut reg = 0;
        set_field(&mut reg, 8, 0, ref0 as u32);
        set_field(&mut reg, 8, 8, ref1 as u32);
        set_field(&mut reg, 3, 16, comp0);
        set_field(&mut reg, 3, 19, comp1);
        set_field(&mut reg, 2, 22, op);
        set_field(&mut reg, 8, 24, 0xF3);
        self.write_ras_reg(reg);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_order(&mut self, id: u32, tcid: u32, tmid: u32, cid: u32) {
        const C2R: [u32; 9] = [0, 1, 0, 1, 0, 1, 7, 5, 6];
        let i = id as usize;
        self.sh.texmap_id[i] = tmid;
        let tmap = tmid & !0x100;
        let tmap = if tmap >= GX_MAX_TEXMAP { GX_TEXMAP0 } else { tmap };
        let tcoord = if tcid >= GX_MAX_TEXCOORD { GX_TEXCOORD0 } else { tcid };
        let ch = if cid == GX_COLOR_NULL { 7 } else { C2R[cid as usize] };
        let en = (tmid != GX_TEXMAP_NULL && tmid & 0x100 == 0) as u32;
        let r = &mut self.sh.tref[i / 2];
        if id & 1 != 0 {
            set_field(r, 3, 12, tmap);
            set_field(r, 3, 15, tcoord);
            set_field(r, 3, 19, ch);
            set_field(r, 1, 18, en);
        } else {
            set_field(r, 3, 0, tmap);
            set_field(r, 3, 3, tcoord);
            set_field(r, 3, 7, ch);
            set_field(r, 1, 6, en);
        }
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
        self.sh.dirty_state |= 1;
    }
    pub fn set_num_tev_stages(&mut self, n: u8) {
        set_field(&mut self.sh.gen_mode, 4, 10, (n as u32).wrapping_sub(1));
        self.sh.dirty_state |= 4;
    }
    pub fn set_tev_kcolor(&mut self, id: u32, c: GxColor) {
        assert!(id < GX_MAX_KCOLOR, "bad kcolor {id}");
        let mut ra = 0;
        set_field(&mut ra, 8, 0, c.r as u32);
        set_field(&mut ra, 8, 12, c.a as u32);
        set_field(&mut ra, 4, 20, 8);
        set_field(&mut ra, 8, 24, 0xE0 + id * 2);
        let mut bg = 0;
        set_field(&mut bg, 8, 0, c.b as u32);
        set_field(&mut bg, 8, 12, c.g as u32);
        set_field(&mut bg, 4, 20, 8);
        set_field(&mut bg, 8, 24, 0xE1 + id * 2);
        self.write_ras_reg(ra);
        self.write_ras_reg(bg);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_kcolor_sel(&mut self, id: u32, sel: u32) {
        let k = (id / 2) as usize;
        if id & 1 != 0 {
            set_field(&mut self.sh.tev_ksel[k], 5, 14, sel);
        } else {
            set_field(&mut self.sh.tev_ksel[k], 5, 4, sel);
        }
        let v = self.sh.tev_ksel[k];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_kalpha_sel(&mut self, id: u32, sel: u32) {
        let k = (id / 2) as usize;
        if id & 1 != 0 {
            set_field(&mut self.sh.tev_ksel[k], 5, 19, sel);
        } else {
            set_field(&mut self.sh.tev_ksel[k], 5, 9, sel);
        }
        let v = self.sh.tev_ksel[k];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_swap_mode(&mut self, stage: u32, ras: u32, tex: u32) {
        let r = &mut self.sh.teva[stage as usize];
        set_field(r, 2, 0, ras);
        set_field(r, 2, 2, tex);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_swap_mode_table(&mut self, id: u32, red: u32, green: u32, blue: u32, alpha: u32) {
        assert!(id < GX_MAX_TEVSWAP, "bad tev swap sel {id}");
        let k = (id * 2) as usize;
        set_field(&mut self.sh.tev_ksel[k], 2, 0, red);
        set_field(&mut self.sh.tev_ksel[k], 2, 2, green);
        let v = self.sh.tev_ksel[k];
        self.write_ras_reg(v);
        set_field(&mut self.sh.tev_ksel[k + 1], 2, 0, blue);
        set_field(&mut self.sh.tev_ksel[k + 1], 2, 2, alpha);
        let v = self.sh.tev_ksel[k + 1];
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }

    // ------------------------------------------------------------------ GXBump.cpp

    pub fn set_num_ind_stages(&mut self, n: u8) {
        set_field(&mut self.sh.gen_mode, 3, 16, n as u32);
        self.sh.dirty_state |= 6;
    }
    pub fn set_ind_tex_order(&mut self, stage: u32, coord: u32, map: u32) {
        let r = &mut self.sh.iref;
        let sh = stage * 6;
        set_field(r, 3, sh, map);
        set_field(r, 3, sh + 3, coord);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
        self.sh.dirty_state |= 3;
    }
    pub fn set_ind_tex_coord_scale(&mut self, stage: u32, s: u32, t: u32) {
        let r = if stage < GX_INDTEXSTAGE2 { &mut self.sh.ind_tex_scale0 } else { &mut self.sh.ind_tex_scale1 };
        let sh = (stage & 1) * 8;
        set_field(r, 4, sh, s);
        set_field(r, 4, sh + 4, t);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    /// `GXSetIndTexMtx(id, mtx[2][3], scaleExp)`.
    pub fn set_ind_tex_mtx(&mut self, id: u32, m: &[[f32; 3]; 2], scale_exp: i8) {
        assert!((GX_ITM_0..=GX_ITM_2).contains(&id), "invalid ind tex mtx ID {id}");
        let adj = scale_exp as i32 + 17;
        let idx = id - 1;
        let q = |v: f32| ((1024.0f32 * v) as i32 & 0x7FF) as u32;
        for col in 0..3u32 {
            let mut r = 0;
            set_field(&mut r, 11, 0, q(m[0][col as usize]));
            set_field(&mut r, 11, 11, q(m[1][col as usize]));
            set_field(&mut r, 2, 22, ((adj >> (col * 2)) & 3) as u32);
            set_field(&mut r, 8, 24, idx * 3 + 6 + col);
            self.write_ras_reg(r);
        }
        self.sh.bp_sent = 1;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn set_tev_indirect(
        &mut self,
        tev_stage: u32,
        ind_stage: u32,
        fmt: u32,
        bias: u32,
        mtx: u32,
        wrap_s: u32,
        wrap_t: u32,
        add_prev: bool,
        ind_lod: bool,
        alpha: u32,
    ) {
        let mut r = 0;
        set_field(&mut r, 2, 0, ind_stage);
        set_field(&mut r, 2, 2, fmt);
        set_field(&mut r, 3, 4, bias);
        set_field(&mut r, 2, 7, alpha);
        set_field(&mut r, 4, 9, mtx);
        set_field(&mut r, 3, 13, wrap_s);
        set_field(&mut r, 3, 16, wrap_t);
        set_field(&mut r, 1, 19, ind_lod as u32);
        set_field(&mut r, 1, 20, add_prev as u32);
        set_field(&mut r, 8, 24, tev_stage + 16);
        self.write_ras_reg(r);
        self.sh.bp_sent = 1;
    }
    pub fn set_tev_direct(&mut self, stage: u32) {
        self.set_tev_indirect(stage, 0, GX_ITF_8, GX_ITB_NONE, GX_ITM_OFF, GX_ITW_OFF, GX_ITW_OFF, false, false, GX_ITBA_OFF);
    }

    // ------------------------------------------------------------------ GXLighting.cpp

    pub fn init_light_attn(o: &mut GxLightObj, a0: f32, a1: f32, a2: f32, k0: f32, k1: f32, k2: f32) {
        (o.a0, o.a1, o.a2, o.k0, o.k1, o.k2) = (a0, a1, a2, k0, k1, k2);
    }
    pub fn init_light_pos(o: &mut GxLightObj, x: f32, y: f32, z: f32) {
        (o.px, o.py, o.pz) = (x, y, z);
    }
    pub fn init_light_dir(o: &mut GxLightObj, x: f32, y: f32, z: f32) {
        (o.nx, o.ny, o.nz) = (-x, -y, -z);
    }
    pub fn init_light_color(o: &mut GxLightObj, c: GxColor) {
        o.color = c;
    }
    pub fn load_light_obj_imm(&mut self, o: &GxLightObj, id: u32) {
        let idx = id.trailing_zeros();
        let addr = 0x600 + idx * 0x10;
        let c = o.color;
        let packed = (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32;
        self.w8(0x10);
        self.w32(addr | (0xF << 16));
        self.w32(0);
        self.w32(0);
        self.w32(0);
        self.w32(packed);
        for v in [o.a0, o.a1, o.a2, o.k0, o.k1, o.k2, o.px, o.py, o.pz, o.nx, o.ny, o.nz] {
            self.wf32(v);
        }
    }
    fn chan_color(&mut self, id: u32, c: GxColor, mat: bool) {
        if id == GX_COLOR0A0 {
            self.chan_color(GX_COLOR0, c, mat);
            self.chan_color(GX_ALPHA0, c, mat);
            return;
        } else if id == GX_COLOR1A1 {
            self.chan_color(GX_COLOR1, c, mat);
            self.chan_color(GX_ALPHA1, c, mat);
            return;
        }
        assert!(id <= GX_ALPHA1, "bad channel {id}");
        let packed = (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32;
        let ch = if id == GX_COLOR0 || id == GX_ALPHA0 { 0 } else { 1 };
        if mat {
            self.sh.mat_color[ch] = packed;
            self.write_xf_reg(0xC + ch as u32, packed);
        } else {
            self.sh.amb_color[ch] = packed;
            self.write_xf_reg(0xA + ch as u32, packed);
        }
        self.sh.bp_sent = 0;
    }
    pub fn set_chan_amb_color(&mut self, id: u32, c: GxColor) {
        self.chan_color(id, c, false);
    }
    pub fn set_chan_mat_color(&mut self, id: u32, c: GxColor) {
        self.chan_color(id, c, true);
    }
    pub fn set_num_chans(&mut self, n: u8) {
        set_field(&mut self.sh.gen_mode, 3, 4, n as u32);
        self.write_xf_reg(9, n as u32);
        self.sh.dirty_state |= 4;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn set_chan_ctrl(&mut self, id: u32, enable: bool, amb: u32, mat: u32, lights: u32, diff: u32, attn: u32) {
        if id == GX_COLOR0A0 {
            self.set_chan_ctrl(GX_COLOR0, enable, amb, mat, lights, diff, attn);
            self.set_chan_ctrl(GX_ALPHA0, enable, amb, mat, lights, diff, attn);
            return;
        } else if id == GX_COLOR1A1 {
            self.set_chan_ctrl(GX_COLOR1, enable, amb, mat, lights, diff, attn);
            self.set_chan_ctrl(GX_ALPHA1, enable, amb, mat, lights, diff, attn);
            return;
        }
        assert!(id <= GX_ALPHA1, "bad channel {id}");
        let mut r = 0;
        set_field(&mut r, 1, 0, mat);
        set_field(&mut r, 1, 1, enable as u32);
        set_field(&mut r, 4, 2, lights & 0xF);
        set_field(&mut r, 1, 6, amb);
        set_field(&mut r, 2, 7, if attn == GX_AF_SPEC { GX_DF_NONE } else { diff });
        set_field(&mut r, 1, 9, (attn != GX_AF_NONE) as u32);
        set_field(&mut r, 1, 10, (attn != GX_AF_SPEC) as u32);
        set_field(&mut r, 4, 11, (lights >> 4) & 0xF);
        self.write_xf_reg(0xE + id, r);
        self.sh.bp_sent = 0;
    }

    // ------------------------------------------------------------------ GXPixel.cpp

    pub fn set_fog(&mut self, ty: u32, start_z: f32, end_z: f32, near_z: f32, far_z: f32, color: GxColor) {
        let ortho = ty & 0x08 != 0;
        let (a, c);
        let mut bm_raw = 0u32;
        let mut b_scale = 0u32;
        if ortho {
            if far_z == near_z || end_z == start_z {
                a = 0.0f32;
                c = 0.0f32;
            } else {
                let scale = 1.0f32 / (end_z - start_z);
                a = scale * (far_z - near_z);
                c = scale * (start_z - near_z);
            }
        } else {
            let (ac, bc, cc) = if far_z == near_z || end_z == start_z {
                (0.0f32, 0.5f32, 0.0f32)
            } else {
                ((far_z * near_z) / ((far_z - near_z) * (end_z - start_z)), far_z / (far_z - near_z), start_z / (end_z - start_z))
            };
            let mut bm = bc;
            let mut be: u32 = 0;
            while bm > 1.0 {
                bm *= 0.5;
                be = be.wrapping_add(1);
            }
            while bm > 0.0 && bm < 0.5 {
                bm *= 2.0;
                be = be.wrapping_sub(1);
            }
            a = ac / (1i32.wrapping_shl(be.wrapping_add(1)) as f32);
            bm_raw = (8.388_638e6f32 * bm) as u32;
            b_scale = be.wrapping_add(1);
            c = cc;
        }
        let ah = a.to_bits();
        let ch = c.to_bits();
        let mut f0 = 0;
        set_field(&mut f0, 11, 0, (ah >> 12) & 0x7FF);
        set_field(&mut f0, 8, 11, (ah >> 23) & 0xFF);
        set_field(&mut f0, 1, 19, ah >> 31);
        set_field(&mut f0, 8, 24, 0xEE);
        let mut f1 = 0;
        set_field(&mut f1, 24, 0, bm_raw);
        set_field(&mut f1, 8, 24, 0xEF);
        let mut f2 = 0;
        set_field(&mut f2, 5, 0, b_scale);
        set_field(&mut f2, 8, 24, 0xF0);
        let mut f3 = 0;
        set_field(&mut f3, 11, 0, (ch >> 12) & 0x7FF);
        set_field(&mut f3, 8, 11, (ch >> 23) & 0xFF);
        set_field(&mut f3, 1, 19, ch >> 31);
        set_field(&mut f3, 1, 20, ortho as u32);
        set_field(&mut f3, 3, 21, ty & 7);
        set_field(&mut f3, 8, 24, 0xF1);
        let mut fc = 0;
        set_field(&mut fc, 8, 0, color.b as u32);
        set_field(&mut fc, 8, 8, color.g as u32);
        set_field(&mut fc, 8, 16, color.r as u32);
        set_field(&mut fc, 8, 24, 0xF2);
        for r in [f0, f1, f2, f3, fc] {
            self.write_ras_reg(r);
        }
        self.sh.bp_sent = 1;
    }
    pub fn set_fog_color(&mut self, color: GxColor) {
        let mut fc = 0;
        set_field(&mut fc, 8, 0, color.b as u32);
        set_field(&mut fc, 8, 8, color.g as u32);
        set_field(&mut fc, 8, 16, color.r as u32);
        set_field(&mut fc, 8, 24, 0xF2);
        self.write_ras_reg(fc);
        self.sh.bp_sent = 1;
    }
    /// `GXSetFogRangeAdj(enable, center, table)`; `table` is the 10 `GXFogAdjTable.r` entries.
    pub fn set_fog_range_adj(&mut self, enable: bool, center: u16, table: Option<&[u16; 10]>) {
        if enable {
            let t = table.expect("fog range table");
            for i in (0..10).step_by(2) {
                let mut r = 0;
                set_field(&mut r, 12, 0, t[i] as u32);
                set_field(&mut r, 12, 12, t[i + 1] as u32);
                set_field(&mut r, 8, 24, (i as u32 >> 1) + 0xE9);
                self.write_ras_reg(r);
            }
        }
        let mut rc = 0;
        set_field(&mut rc, 10, 0, center as u32 + 342);
        set_field(&mut rc, 1, 10, enable as u32);
        set_field(&mut rc, 8, 24, 0xE8);
        self.write_ras_reg(rc);
        self.sh.bp_sent = 1;
    }
    pub fn set_blend_mode(&mut self, mode: u32, src: u32, dst: u32, op: u32) {
        let r = &mut self.sh.cmode0;
        set_field(r, 1, 0, (mode == GX_BM_BLEND || mode == GX_BM_SUBTRACT) as u32);
        set_field(r, 1, 11, (mode == GX_BM_SUBTRACT) as u32);
        set_field(r, 1, 1, (mode == GX_BM_LOGIC) as u32);
        set_field(r, 4, 12, op);
        set_field(r, 3, 8, src);
        set_field(r, 3, 5, dst);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_color_update(&mut self, e: bool) {
        set_field(&mut self.sh.cmode0, 1, 3, e as u32);
        let v = self.sh.cmode0;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_alpha_update(&mut self, e: bool) {
        set_field(&mut self.sh.cmode0, 1, 4, e as u32);
        let v = self.sh.cmode0;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_z_mode(&mut self, compare: bool, func: u32, update: bool) {
        let r = &mut self.sh.zmode;
        set_field(r, 1, 0, compare as u32);
        set_field(r, 3, 1, func);
        set_field(r, 1, 4, update as u32);
        let v = *r;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_z_comp_loc(&mut self, before_tex: bool) {
        set_field(&mut self.sh.pe_ctrl, 1, 6, before_tex as u32);
        let v = self.sh.pe_ctrl;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_pixel_fmt(&mut self, pix: u32, z: u32) {
        let old = self.sh.pe_ctrl;
        let hw = match pix {
            GX_PF_RGB8_Z24 => 0,
            GX_PF_RGBA6_Z24 => 1,
            GX_PF_RGB565_Z16 => 2,
            GX_PF_Z24 => 3,
            GX_PF_Y8 | GX_PF_U8 | GX_PF_V8 => 4,
            GX_PF_YUV420 => 5,
            _ => panic!("GXSetPixelFmt: unsupported pixel format {pix}"),
        };
        set_field(&mut self.sh.pe_ctrl, 3, 0, hw);
        set_field(&mut self.sh.pe_ctrl, 3, 3, z);
        if old != self.sh.pe_ctrl {
            let v = self.sh.pe_ctrl;
            self.write_ras_reg(v);
            set_field(&mut self.sh.gen_mode, 1, 9, (pix == GX_PF_RGB565_Z16) as u32);
            self.sh.dirty_state |= 4;
        }
        if hw == 4 {
            set_field(&mut self.sh.cmode1, 2, 9, (pix - GX_PF_Y8) & 3);
            let v = self.sh.cmode1;
            self.write_ras_reg(v);
        }
        self.sh.bp_sent = 1;
    }
    pub fn set_dither(&mut self, d: bool) {
        set_field(&mut self.sh.cmode0, 1, 2, d as u32);
        let v = self.sh.cmode0;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_dst_alpha(&mut self, enable: bool, value: u8) {
        set_field(&mut self.sh.cmode1, 8, 0, value as u32);
        set_field(&mut self.sh.cmode1, 1, 8, enable as u32);
        let v = self.sh.cmode1;
        self.write_ras_reg(v);
        self.sh.bp_sent = 1;
    }
    pub fn set_field_mask(&mut self, odd: bool, even: bool) {
        let mut r = 0;
        set_field(&mut r, 1, 0, even as u32);
        set_field(&mut r, 1, 1, odd as u32);
        set_field(&mut r, 8, 24, 0x44);
        self.write_ras_reg(r);
        self.sh.bp_sent = 1;
    }
    pub fn set_field_mode(&mut self, field_mode: bool, half_aspect: bool) {
        set_field(&mut self.sh.lp_size, 1, 22, half_aspect as u32);
        let v = self.sh.lp_size;
        self.write_ras_reg(v);
        self.flush_texture_state();
        let mut r = 0;
        set_field(&mut r, 8, 24, 0x68);
        set_field(&mut r, 1, 0, field_mode as u32);
        self.write_ras_reg(r);
        self.flush_texture_state();
    }

    // ------------------------------------------------------------------ GXTransform.cpp

    /// `GXSetProjection(mtx[4][4], type)`.
    pub fn set_projection(&mut self, m: &[[f32; 4]; 4], ty: u32) {
        let ortho = ty == GX_ORTHOGRAPHIC;
        let p = [
            m[0][0],
            if ortho { m[0][3] } else { m[0][2] },
            m[1][1],
            if ortho { m[1][3] } else { m[1][2] },
            m[2][2],
            m[2][3],
        ];
        self.sh.proj_type = ty;
        self.sh.proj_mtx = p;
        self.w8(0x10);
        self.w32(0x0006_1020);
        for v in p {
            self.wf32(v);
        }
        self.w32(ty);
        self.sh.bp_sent = 0;
    }
    /// `GXSetProjectionv(p[7])`.
    pub fn set_projectionv(&mut self, v: &[f32; 7]) {
        let ty = if v[0] == 0.0 { GX_PERSPECTIVE } else { GX_ORTHOGRAPHIC };
        self.sh.proj_type = ty;
        self.sh.proj_mtx.copy_from_slice(&v[1..]);
        self.w8(0x10);
        self.w32(0x0006_1020);
        for x in &v[1..] {
            self.wf32(*x);
        }
        self.w32(ty);
        self.sh.bp_sent = 0;
    }
    pub fn load_pos_mtx_imm(&mut self, m: &[[f32; 4]; 3], id: u32) {
        assert!(id <= GX_PNMTX9, "invalid pn mtx {id}");
        self.w8(0x10);
        self.w32((id * 4) | 0xB0000);
        for r in m {
            for &x in r {
                self.wf32(x);
            }
        }
    }
    pub fn load_pos_mtx_indx(&mut self, idx: u16, id: u32) {
        self.w8(GX_LOAD_INDX_A);
        self.w16(idx);
        self.w16(((11u32 << 12) | id * 4) as u16);
    }
    pub fn load_nrm_mtx_imm(&mut self, m: &[[f32; 4]; 3], id: u32) {
        assert!(id <= GX_PNMTX9, "invalid pn mtx {id}");
        self.w8(0x10);
        self.w32((id * 3 + 0x400) | 0x80000);
        for r in m {
            for &x in &r[..3] {
                self.wf32(x);
            }
        }
    }
    pub fn set_current_mtx(&mut self, id: u32) {
        assert!(id <= GX_PNMTX9, "invalid pn mtx {id}");
        set_field(&mut self.sh.mat_idx_a, 6, 0, id);
        self.set_matrix_index(GX_VA_PNMTXIDX);
    }
    /// `GXLoadTexMtxImm(mtx, id, type)`: `m` holds 2 rows for GX_MTX2x4 (the third is ignored).
    pub fn load_tex_mtx_imm(&mut self, m: &[[f32; 4]; 3], id: u32, ty: u32) {
        let addr = if id >= GX_PTTEXMTX0 {
            assert!(ty == GX_MTX3x4, "invalid pt mtx type {ty}");
            (id - GX_PTTEXMTX0) * 4 + 0x500
        } else {
            id * 4
        };
        let count = if ty == GX_MTX2x4 { 8 } else { 12 };
        self.w8(0x10);
        self.w32(addr | ((count - 1) << 16));
        for i in 0..count as usize {
            self.wf32(m[i / 4][i % 4]);
        }
    }
    /// `GXGetProjectionv` (GXGet.cpp): `[0]` is 0 for perspective, 1 otherwise, then the six
    /// projection parameters as GXSetProjection stored them.
    pub fn get_projectionv(&self) -> [f32; 7] {
        let mut p = [0.0f32; 7];
        p[0] = if self.sh.proj_type == GX_PERSPECTIVE { 0.0 } else { 1.0 };
        p[1..].copy_from_slice(&self.sh.proj_mtx);
        p
    }
    /// `GXGetViewportv`: left, top, width, height, near, far as last set (before jitter).
    pub fn get_viewportv(&self) -> [f32; 6] {
        self.sh.vp
    }
    pub fn set_viewport(&mut self, left: f32, top: f32, w: f32, h: f32, nz: f32, fz: f32) {
        self.set_viewport_jitter(left, top, w, h, nz, fz, 1);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn set_viewport_jitter(&mut self, left: f32, mut top: f32, w: f32, h: f32, nz: f32, fz: f32, field: u32) {
        if field == 0 {
            top -= 0.5;
        }
        let sx = w / 2.0;
        let sy = -h / 2.0;
        let ox = 340.0 + (left + w / 2.0);
        let oy = 340.0 + (top + h / 2.0);
        let zmin = 1.677_721_5e7f32 * nz;
        let zmax = 1.677_721_5e7f32 * fz;
        let sz = zmax - zmin;
        let oz = zmax;
        self.sh.vp = [left, top, w, h, nz, fz];
        self.w8(0x10);
        self.w32(0x0005_101A);
        for v in [sx, sy, sz, ox, oy, oz] {
            self.wf32(v);
        }
        self.sh.bp_sent = 0;
    }

    // ------------------------------------------------------------------ GXTexture.cpp

    fn init_texobj_common(&mut self, o: &mut TexObj, data: Option<GxData>, w: u16, h: u16, fmt: u32, ws: u32, wt: u32, mip: bool) {
        *o = TexObj::default();
        o.width = w as u32;
        o.height = h as u32;
        o.format = fmt;
        o.tlut = GX_TLUT0;
        o.flags = 2;
        o.tex_obj_id = self.next_tex_obj_id;
        self.next_tex_obj_id += 1;
        o.tex_data_version = 1;
        set_field(&mut o.mode0, 2, 0, ws);
        set_field(&mut o.mode0, 2, 2, wt);
        set_field(&mut o.mode0, 1, 4, 1);
        if mip {
            o.flags |= 1;
            o.mode0 = (o.mode0 & 0xFFFF_FF1F) | 0xC0;
            let max_dim = w.max(h) as u32;
            let lmax = (16.0f32 * (31 - max_dim.leading_zeros() as i32) as f32) as u8;
            set_field(&mut o.mode1, 8, 8, lmax as u32);
        } else {
            o.mode0 = (o.mode0 & 0xFFFF_FF1F) | 0x80;
        }
        set_field(&mut o.image0, 10, 0, (w as u32).wrapping_sub(1));
        set_field(&mut o.image0, 10, 10, (h as u32).wrapping_sub(1));
        set_field(&mut o.image0, 4, 20, fmt & 0xF);
        o.data = data;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn init_tex_obj(&mut self, o: &mut TexObj, data: Option<GxData>, w: u16, h: u16, fmt: u32, ws: u32, wt: u32, mip: bool) {
        self.init_texobj_common(o, data, w, h, fmt, ws, wt, mip);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn init_tex_obj_ci(
        &mut self,
        o: &mut TexObj,
        data: Option<GxData>,
        w: u16,
        h: u16,
        fmt: u32,
        ws: u32,
        wt: u32,
        mip: bool,
        tlut: u32,
    ) {
        self.init_texobj_common(o, data, w, h, fmt, ws, wt, mip);
        o.tlut = tlut;
        o.flags &= !2;
    }
    #[allow(clippy::too_many_arguments)]
    pub fn init_tex_obj_lod(
        o: &mut TexObj,
        min_filt: u32,
        mag_filt: u32,
        min_lod: f32,
        max_lod: f32,
        lod_bias: f32,
        bias_clamp: bool,
        edge_lod: bool,
        aniso: u32,
    ) {
        let b = lod_bias.clamp(-4.0, 3.99);
        // static_cast<u8>(32.0f * bias): float -> u8 of a possibly negative value; MSVC x86 goes
        // through int, so -32.0 gives 0xE0 (the encoding the hardware expects).
        let lb = ((32.0f32 * b) as i32 as u32) & 0xFF;
        set_field(&mut o.mode0, 8, 9, lb);
        set_field(&mut o.mode0, 1, 4, (mag_filt == GX_LINEAR) as u32);
        set_field(&mut o.mode0, 3, 5, GX2HW_FILT_CONV[min_filt as usize]);
        set_field(&mut o.mode0, 1, 8, (!edge_lod) as u32);
        o.mode0 &= 0xFFFD_FFFF;
        o.mode0 &= 0xFFFB_FFFF;
        set_field(&mut o.mode0, 2, 19, aniso);
        set_field(&mut o.mode0, 1, 21, bias_clamp as u32);
        set_field(&mut o.mode1, 8, 0, (16.0f32 * min_lod.clamp(0.0, 10.0)) as u8 as u32);
        set_field(&mut o.mode1, 8, 8, (16.0f32 * max_lod.clamp(0.0, 10.0)) as u8 as u32);
    }
    pub fn init_tex_obj_data(o: &mut TexObj, data: Option<GxData>) {
        o.data = data;
        o.tex_data_version += 1;
    }
    pub fn init_tex_obj_wrap_mode(o: &mut TexObj, s: u32, t: u32) {
        set_field(&mut o.mode0, 2, 0, s);
        set_field(&mut o.mode0, 2, 2, t);
    }
    pub fn init_tex_obj_tlut(o: &mut TexObj, tlut: u32) {
        o.tlut = tlut;
    }
    pub fn init_tex_obj_filter(o: &mut TexObj, min_filt: u32, mag_filt: u32) {
        set_field(&mut o.mode0, 3, 5, GX2HW_FILT_CONV[min_filt as usize]);
        set_field(&mut o.mode0, 1, 4, (mag_filt == GX_LINEAR) as u32);
    }
    pub fn load_tex_obj(&mut self, o: &mut TexObj, id: u32) {
        let i = id as usize;
        set_field(&mut o.mode0, 8, 24, TEX_MODE0_IDS[i]);
        set_field(&mut o.mode1, 8, 24, TEX_MODE1_IDS[i]);
        set_field(&mut o.image0, 8, 24, TEX_IMAGE0_IDS[i]);
        let mut image1 = 0;
        let mut image2 = 0;
        set_field(&mut image1, 8, 24, TEX_IMAGE1_IDS[i]);
        set_field(&mut image2, 8, 24, TEX_IMAGE2_IDS[i]);
        set_field(&mut o.image3, 8, 24, TEX_IMAGE3_IDS[i]);
        for r in [o.mode0, o.mode1, o.image0, image1, image2, o.image3] {
            self.write_ras_reg(r);
        }
        if o.flags & 2 == 0 {
            let mut t = 0;
            set_field(&mut t, 10, 0, o.tlut);
            set_field(&mut t, 8, 24, TEX_TLUT_IDS[i]);
            self.write_ras_reg(t);
        }
        self.sh.t_image0[i] = o.image0;
        self.sh.t_mode0[i] = o.mode0;
        self.sh.dirty_state |= 1;
        self.sh.bp_sent = 1;
        let h = self.res.handle(o.data.as_ref());
        self.write_aurora(AURORA_LOAD_TEXOBJ);
        self.w8(id as u8);
        self.w64(h);
        self.w32(o.width());
        self.w32(o.height());
        self.w32(o.format());
        self.w32(o.tlut);
        self.w8(o.has_mips() as u8);
        self.w32(o.tex_obj_id);
        self.w32(o.tex_data_version);
    }
    pub fn init_tlut_obj(&mut self, o: &mut TlutObj, data: Option<GxData>, fmt: u32, entries: u16) {
        *o = TlutObj::default();
        o.data = data;
        o.format = fmt;
        o.num_entries = entries;
        o.tlut_obj_id = self.next_tlut_obj_id;
        self.next_tlut_obj_id += 1;
        o.tlut_data_version = 1;
        set_field(&mut o.tlut, 2, 10, fmt);
        set_field(&mut o.load_tlut0, 8, 24, 0x64);
    }
    pub fn load_tlut(&mut self, o: &TlutObj, idx: u32) {
        self.flush_texture_state();
        self.write_ras_reg(o.load_tlut0);
        let mut l1 = 0;
        set_field(&mut l1, 10, 0, idx);
        set_field(&mut l1, 10, 10, (o.num_entries as u32).wrapping_sub(1));
        set_field(&mut l1, 8, 24, 0x65);
        self.write_ras_reg(l1);
        self.flush_texture_state();
        let h = self.res.handle(o.data.as_ref());
        self.write_aurora(AURORA_LOAD_TLUT);
        self.w8(idx as u8);
        self.w64(h);
        self.w32(o.format);
        self.w16(o.num_entries);
        self.w32(o.tlut_obj_id);
        self.w32(o.tlut_data_version);
    }
    pub fn set_tex_coord_scale_manually(&mut self, coord: u32, enable: bool, ss: u16, ts: u16) {
        let c = coord as usize;
        self.sh.tcs_man_enab = (self.sh.tcs_man_enab & !(1 << coord)) | ((enable as u8) << coord);
        if enable {
            set_field(&mut self.sh.su_ts0[c], 16, 0, ss.wrapping_sub(1) as u32);
            set_field(&mut self.sh.su_ts1[c], 16, 0, ts.wrapping_sub(1) as u32);
            let (a, b) = (self.sh.su_ts0[c], self.sh.su_ts1[c]);
            self.write_ras_reg(a);
            self.write_ras_reg(b);
            self.sh.bp_sent = 1;
        }
    }

    // ------------------------------------------------------------------ GXFrameBuffer.cpp / GXAurora.cpp

    pub fn set_copy_clear(&mut self, c: GxColor, depth: u32) {
        let mut r0 = 0;
        set_field(&mut r0, 8, 0, c.r as u32);
        set_field(&mut r0, 8, 8, c.a as u32);
        set_field(&mut r0, 8, 24, 0x4F);
        let mut r1 = 0;
        set_field(&mut r1, 8, 0, c.b as u32);
        set_field(&mut r1, 8, 8, c.g as u32);
        set_field(&mut r1, 8, 24, 0x50);
        let mut r2 = 0;
        set_field(&mut r2, 24, 0, depth);
        set_field(&mut r2, 8, 24, 0x51);
        self.write_ras_reg(r0);
        self.write_ras_reg(r1);
        self.write_ras_reg(r2);
        self.sh.bp_sent = 1;
    }
    pub fn set_viewport_render(&mut self, left: f32, top: f32, w: f32, h: f32, nz: f32, fz: f32) {
        self.write_aurora(AURORA_LOAD_VIEWPORT_RENDER);
        for v in [left, top, w, h, nz, fz] {
            self.wf32(v);
        }
    }
    pub fn set_scissor_render(&mut self, left: u32, top: u32, w: u32, h: u32) {
        self.write_aurora(AURORA_LOAD_SCISSOR_RENDER);
        for v in [left, top, w, h] {
            self.w32(v);
        }
    }
    pub fn set_projection_full(&mut self, m: &[[f32; 4]; 4]) {
        self.write_aurora(AURORA_LOAD_PROJECTION_FULL);
        for r in m {
            for &x in r {
                self.wf32(x);
            }
        }
    }
    pub fn set_polygon_offset(&mut self, fo: f32, fs: f32, bo: f32, bs: f32, clamp: f32) {
        self.write_aurora(GX2_SET_POLYGON_OFFSET);
        for v in [fo, fs, bo, bs, clamp] {
            self.wf32(v);
        }
    }

    // ------------------------------------------------------------------ GXDispList.cpp

    pub fn begin_display_list(&mut self) {
        assert!(self.dl.is_none(), "Display list began twice!");
        if self.sh.dirty_state != 0 {
            self.set_dirty_state();
        }
        if self.sh.dl_save_context {
            self.saved = Some(self.sh.clone());
        }
        self.dl = Some(Vec::new());
    }
    /// `GXEndDisplayList`: the recorded list, zero-padded to 32 bytes.
    pub fn end_display_list(&mut self) -> Vec<u8> {
        if self.sh.dirty_state != 0 {
            self.set_dirty_state();
        }
        let mut dl = self.dl.take().expect("no display list");
        let padded = (dl.len() + 31) & !31;
        dl.resize(padded, 0);
        if let Some(s) = self.saved.take() {
            self.sh = s;
        }
        dl
    }
    pub fn call_display_list(&mut self, list: &[u8]) {
        if self.sh.dirty_state != 0 {
            self.set_dirty_state();
        }
        if self.sh.v_num != 0 || self.sh.bp_sent != 0 {
            self.send_flush_prim();
        }
        self.wdata(list);
    }

    // ------------------------------------------------------------------ GXVert.cpp

    fn pre_begin(&mut self) {
        assert!(!self.in_begin, "GXBegin: called without matching GXEnd");
        if self.sh.dirty_state != 0 {
            self.set_dirty_state();
        }
        if self.sh.v_num != 0 || self.sh.bp_sent != 0 {
            // Aurora tests the (vNum, bpSent) pair as one u32; both only clear bpSent.
            self.send_flush_prim();
        }
    }
    pub fn begin(&mut self, prim: u32, fmt: u32, n: u16) {
        self.pre_begin();
        let cmd = (fmt | prim) as u8;
        self.begin_auto = n == GX_AUTO;
        if self.begin_auto {
            assert!(self.dl.is_none(), "GXBegin: GX_AUTO not supported in display lists");
            self.write_aurora(AURORA_DRAW_SIZED);
            self.w8(cmd);
            self.begin_size_offset = self.buffer_size();
            self.w32(0);
        } else {
            self.w8(cmd);
            self.w16(n);
        }
        self.in_begin = true;
    }
    /// `GXBeginIndexed`: triangles with host-endian 16-bit indices.
    pub fn begin_indexed(&mut self, fmt: u32, n: u16, indices: &[u16]) {
        self.pre_begin();
        assert!(n != 0 && !indices.is_empty() && indices.len() % 3 == 0);
        self.begin_auto = false;
        self.write_aurora(AURORA_DRAW_INDEXED);
        self.w8((fmt | GX_TRIANGLES) as u8);
        self.w16(n);
        self.w32(indices.len() as u32);
        let bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        self.wdata(&bytes);
        self.in_begin = true;
    }
    pub fn end(&mut self) {
        if self.in_begin {
            if self.begin_auto {
                let off = self.begin_size_offset;
                let n = (self.buffer_size() - off - 4) as u32;
                let buf = self.dl.as_mut().unwrap_or(&mut self.fifo);
                buf[off..off + 4].copy_from_slice(&n.to_be_bytes());
                self.begin_auto = false;
            }
            self.in_begin = false;
        }
    }
    pub fn position3f32(&mut self, x: f32, y: f32, z: f32) {
        self.wf32(x);
        self.wf32(y);
        self.wf32(z);
    }
    pub fn position2f32(&mut self, x: f32, y: f32) {
        self.wf32(x);
        self.wf32(y);
    }
    pub fn position3s16(&mut self, x: i16, y: i16, z: i16) {
        self.w16(x as u16);
        self.w16(y as u16);
        self.w16(z as u16);
    }
    pub fn position1x8(&mut self, i: u8) {
        self.w8(i);
    }
    pub fn position1x16(&mut self, i: u16) {
        self.w16(i);
    }
    pub fn normal3f32(&mut self, x: f32, y: f32, z: f32) {
        self.position3f32(x, y, z);
    }
    pub fn normal1x8(&mut self, i: u8) {
        self.w8(i);
    }
    pub fn color4u8(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.wdata(&[r, g, b, a]);
    }
    pub fn color1u32(&mut self, c: u32) {
        self.w32(c);
    }
    pub fn color1x8(&mut self, i: u8) {
        self.w8(i);
    }
    pub fn tex_coord2f32(&mut self, s: f32, t: f32) {
        self.position2f32(s, t);
    }
    pub fn tex_coord2s16(&mut self, s: i16, t: i16) {
        self.w16(s as u16);
        self.w16(t as u16);
    }
    pub fn tex_coord1x8(&mut self, i: u8) {
        self.w8(i);
    }
    pub fn cmd1u8(&mut self, x: u8) {
        self.w8(x);
    }
    pub fn cmd1u16(&mut self, x: u16) {
        self.w16(x);
    }
    pub fn cmd1u32(&mut self, x: u32) {
        self.w32(x);
    }
}
