//! Port of `pc/platform/shim_gx.c`: the layer every game GX call goes through before it reaches
//! the GX implementation ([`crate::api::Gx`], the port of Aurora's).
//!
//! In the C build the shim exists for two reasons: gwtool renames the game's GX calls to `gw_GX*`,
//! and game memory is big-endian while Aurora takes native values, so matrices, float arrays,
//! `GXColorS10` and the render-mode images are byte-swapped at the boundary. The native Rust port
//! keeps game state native little-endian, so those conversions go away: [`GxShim`] takes native
//! matrices and returns native floats. Data the GPU reads straight from loaded DAT files (vertex
//! arrays, display lists, textures, TLUTs) is still the file's big-endian bytes, as on the GC.
//!
//! What remains is the real work the shim does, ported line for line:
//!   * the indirect-stage guard ([`IndGuard`], `gw_gx_guard_indirect`): a leaked refraction setup
//!     (lbRefract_80022998 without its reset) is drawn direct instead of crashing the shader
//!     generator;
//!   * `GXProject` owned here (not forwarded), with one rounding per operation and no
//!     contraction, because its result reaches the simulation (Camera_LogicToScreen ->
//!     lbVector_WorldToScreen -> the fighters' offscreen flag); [`gx_project`];
//!   * `GXInitFogAdjTable` (Aurora has none; the SDK's GXPixel.c body) and `GXGetTexBufferSize`,
//!     `GXInitLightSpot`/`GXInitLightDistAttn` (Aurora's GXLighting.cpp, not in melee_gx's api);
//!   * the 3-argument `GXSetArray` (the array extends to the end of its buffer);
//!   * the R6 camera-shake offset applied to perspective projections (`gw_Fx_ShakeOffset`);
//!   * draw-done routing to HSD's callback (shim_vi.c `gw_gx_set_draw_done` & co.);
//!   * draw suppression during resimulation (`gw_Gx_SuppressDraws`): display lists are skipped;
//!   * the activity counters behind `gw_gx_get_stats`;
//!   * the three render-mode objects (`gw_GXNtsc480IntDf` & co.), now native structs with their
//!     GameCube layout as [`GcLayout`](melee_core::gc::GcLayout) metadata;
//!   * GXCopyDisp ends the frame: the FIFO is decoded and the frame's draws are handed to the
//!     presenter ([`GxShim::take_frame`]); GXCopyTex is counted and logged (EFB copies are not
//!     implemented in melee_gx yet).
//!
//! Not ported: the TEMP DIAG logging (matrix health, depth-grid sampling, per-copy summaries), the
//! PC matrix palette (`GXAuroraLoadPalette`, a Geno render path), the palette and envelope matrix
//! caches (render-side memo tables for pobj.c's PC paths), perf counters, the loading overlay hook.
use crate::api::{Gx, GxColor, GxColorS10, GxLightObj};
use crate::consts::*;
use crate::draw::DrawCmd;
use crate::resources::GxData;
use crate::state::{TexObj, TlutObj};

// ------------------------------------------------------------------ render modes

/// `GXRenderModeObj` (dolphin/gx/GXStruct.h). In the C build the game reads the three SDK modes
/// as big-endian images; here they are plain native values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GxRenderModeObj {
    pub vi_tv_mode: u32,
    pub fb_width: u16,
    pub efb_height: u16,
    pub xfb_height: u16,
    pub vi_x_origin: u16,
    pub vi_y_origin: u16,
    pub vi_width: u16,
    pub vi_height: u16,
    pub xfb_mode: u32,
    pub field_rendering: u8,
    pub aa: u8,
    pub sample_pattern: [[u8; 2]; 12],
    pub vfilter: [u8; 7],
}
melee_core::gc_layout!(GxRenderModeObj, "_GXRenderModeObj", size 0x3C, align 4, [
    vi_tv_mode @ 0x0: 4 "VITVMode",
    fb_width @ 0x4: 2 "u16",
    efb_height @ 0x6: 2 "u16",
    xfb_height @ 0x8: 2 "u16",
    vi_x_origin @ 0xA: 2 "u16",
    vi_y_origin @ 0xC: 2 "u16",
    vi_width @ 0xE: 2 "u16",
    vi_height @ 0x10: 2 "u16",
    xfb_mode @ 0x14: 4 "VIXFBMode",
    field_rendering @ 0x18: 1 "u8",
    aa @ 0x19: 1 "u8",
    sample_pattern @ 0x1A: 24 "u8[12][2]",
    vfilter @ 0x32: 7 "u8[7]",
]);

pub const VI_TVMODE_NTSC_INT: u32 = 0;
pub const VI_TVMODE_NTSC_PROG: u32 = 2;
pub const VI_XFBMODE_SF: u32 = 0;
pub const VI_XFBMODE_DF: u32 = 1;

const RM_PATTERN: [[u8; 2]; 12] = [[6, 6]; 12];
const RM_VFILTER_DF: [u8; 7] = [8, 8, 10, 12, 10, 8, 8];
const RM_VFILTER_INT: [u8; 7] = [0, 0, 21, 22, 21, 0, 0];

const fn render_mode(tv: u32, xfb_mode: u32, vfilter: [u8; 7]) -> GxRenderModeObj {
    // Values from melee's own SDK source, extern/dolphin/src/dolphin/gx/GXFrameBuf.c.
    GxRenderModeObj {
        vi_tv_mode: tv,
        fb_width: 640,
        efb_height: 480,
        xfb_height: 480,
        vi_x_origin: 40,
        vi_y_origin: 0,
        vi_width: 640,
        vi_height: 480,
        xfb_mode,
        field_rendering: 0,
        aa: 0,
        sample_pattern: RM_PATTERN,
        vfilter,
    }
}
/// `GXNtsc480IntDf`.
pub const GX_NTSC_480_INT_DF: GxRenderModeObj = render_mode(VI_TVMODE_NTSC_INT, VI_XFBMODE_DF, RM_VFILTER_DF);
/// `GXNtsc480Int`.
pub const GX_NTSC_480_INT: GxRenderModeObj = render_mode(VI_TVMODE_NTSC_INT, VI_XFBMODE_DF, RM_VFILTER_INT);
/// `GXNtsc480Prog`.
pub const GX_NTSC_480_PROG: GxRenderModeObj = render_mode(VI_TVMODE_NTSC_PROG, VI_XFBMODE_SF, RM_VFILTER_INT);

// ------------------------------------------------------------------ indirect-stage guard

pub const GW_GX_TEV_MAX: usize = 16;
pub const GW_GX_IND_MAX: usize = 4;

/// The fields `gw_gx_guard_indirect` tracks (shim_gx.c `gw_gx_ind`).
///
/// Retail leaves indirect texturing armed after the fighter refraction draw whenever its reset is
/// skipped (lbRefract_80022998 sets GXSetNumIndStages(1) + GXSetTevIndirect(stage 0, ITM_0); only
/// lbRefract_800225D4 / lbRefract_DObjDispReset undo it, and ftCo_800C8AF0 calls the first only
/// while fp->x2226_b5 is set). A later draw with no texgens then has an indirect stage reading a
/// texcoord nothing generates, which Aurora's shader builder treats as fatal. The guard mirrors
/// shader_info.cpp's liveness rule and turns such a stage back into a direct one at the next draw.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct IndGuard {
    pub num_texgens: u32,
    /// Bit per texcoord: a texgen was written at least once since GXInit (Aurora keeps slots).
    pub texgen_seen: u32,
    pub num_tev: u32,
    pub num_ind: u32,
    pub ind_coord: [u8; GW_GX_IND_MAX],
    /// The texcoord each TEV stage's order names (hardware: 3 bits; NULL reads 0).
    pub tev_coord: [u8; GW_GX_TEV_MAX],
    pub tev_ind_stage: [u8; GW_GX_TEV_MAX],
    /// Matrix, wrap or add-previous set.
    pub tev_ind_live: [u8; GW_GX_TEV_MAX],
    pub tev_ind_matrix: [u8; GW_GX_TEV_MAX],
}

impl IndGuard {
    fn texcoord_generated(&self, coord: u32) -> bool {
        coord < 8 && coord < self.num_texgens && (self.texgen_seen & (1u32 << coord)) != 0
    }
    /// The TEV stages to neutralise now, in order, clearing their live/matrix flags as the C does.
    pub fn stages_to_neutralize(&mut self) -> Vec<u32> {
        let mut out = Vec::new();
        if self.num_ind == 0 {
            return out;
        }
        let mut s = 0u32;
        while s < self.num_tev && (s as usize) < GW_GX_TEV_MAX {
            let i = s as usize;
            if self.tev_ind_live[i] != 0 {
                let ind_stage = self.tev_ind_stage[i] as u32;
                if ind_stage < self.num_ind {
                    let ok = self.texcoord_generated(self.tev_coord[i] as u32)
                        && (self.tev_ind_matrix[i] == 0 || self.texcoord_generated(self.ind_coord[ind_stage as usize] as u32));
                    if !ok {
                        out.push(s);
                        self.tev_ind_live[i] = 0;
                        self.tev_ind_matrix[i] = 0;
                    }
                }
            }
            s += 1;
        }
        out
    }
}

// ------------------------------------------------------------------ pure functions

/// `gw_xplat_GXProject`: Aurora's GXTransform.cpp expressions, operation for operation, one f32
/// rounding each (Rust never contracts; the C has `#pragma STDC FP_CONTRACT OFF` around it).
/// `pm` is GXGetProjectionv's 7 floats, `vp` GXGetViewportv's 6. Returns (sx, sy, sz).
pub fn gx_project(x: f32, y: f32, z: f32, mtx: &[[f32; 4]; 3], pm: &[f32; 7], vp: &[f32; 6]) -> (f32, f32, f32) {
    let px = mtx[0][3] + ((mtx[0][2] * z) + ((mtx[0][0] * x) + (mtx[0][1] * y)));
    let py = mtx[1][3] + ((mtx[1][2] * z) + ((mtx[1][0] * x) + (mtx[1][1] * y)));
    let pz = mtx[2][3] + ((mtx[2][2] * z) + ((mtx[2][0] * x) + (mtx[2][1] * y)));
    let (xc, yc, zc, wc);
    if pm[0] == 0.0 {
        xc = (px * pm[1]) + (pz * pm[2]);
        yc = (py * pm[3]) + (pz * pm[4]);
        zc = pm[6] + (pz * pm[5]);
        wc = 1.0f32 / -pz;
    } else {
        xc = pm[2] + (px * pm[1]);
        yc = pm[4] + (py * pm[3]);
        zc = pm[6] + (pz * pm[5]);
        wc = 1.0f32;
    }
    let sx = (vp[2] / 2.0) + (vp[0] + (wc * (xc * vp[2] / 2.0)));
    let sy = (vp[3] / 2.0) + (vp[1] + (wc * (-yc * vp[3] / 2.0)));
    let sz = vp[5] + (wc * (zc * (vp[5] - vp[4])));
    (sx, sy, sz)
}

/// `gw_GXInitFogAdjTable` (the SDK's GXPixel.c body): the 10 range-adjust entries.
pub fn init_fog_adj_table(width: u16, mtx: &[[f32; 4]; 4]) -> [u16; 10] {
    let near_z;
    let side_x;
    if 0.0f32 == mtx[3][3] {
        near_z = mtx[2][3] / (mtx[2][2] - 1.0);
        side_x = (near_z * (1.0 + mtx[0][2])) / mtx[0][0];
    } else {
        near_z = (1.0 + mtx[2][3]) / mtx[2][2];
        side_x = -(mtx[0][3] - 1.0) / mtx[0][0];
    }
    let mut out = [0u16; 10];
    for (i, o) in out.iter_mut().enumerate() {
        let mut xi = (((i as u32) + 1) << 5) as f32;
        xi *= 2.0 / width as f32;
        xi *= side_x;
        let range = (1.0f32 + (xi * xi) / (near_z * near_z)).sqrt();
        // C: (u32)(256.0f * range) & 0xFFF, x86 conversion semantics (Rust's `as` saturates).
        *o = (x86_f32_to_u32(256.0f32 * range) & 0xFFF) as u16;
    }
    out
}

/// C's `(u32)f` as clang emits it for x86 (cvttss2si, with the 2^31 split): NaN, +inf and values
/// >= 2^32 come out with their low bits zero (0x80000000 ^ 0x80000000 or the integer-indefinite
/// value), where Rust's `as` saturates to u32::MAX. Negative inputs are not reached here.
fn x86_f32_to_u32(v: f32) -> u32 {
    if v.is_nan() || v >= 4_294_967_296.0 {
        if v >= 2_147_483_648.0 { 0 } else { 0x8000_0000 }
    } else if v >= 2_147_483_648.0 {
        ((v - 2_147_483_648.0) as i32 as u32) ^ 0x8000_0000
    } else {
        v as i32 as u32
    }
}

/// `GXGetTexBufferSize` (Aurora GXTexture.cpp).
pub fn get_tex_buffer_size(mut width: u16, mut height: u16, fmt: u32, mips: bool, mut max_lod: u8) -> u32 {
    let (sx, sy) = match fmt {
        GX_TF_I4 | GX_TF_C4 | GX_TF_CMPR | GX_CTF_R4 | GX_CTF_Z4 => (3, 3),
        GX_TF_I8 | GX_TF_IA4 | GX_TF_C8 | GX_TF_Z8 | GX_CTF_RA4 | GX_CTF_A8 | GX_CTF_R8 | GX_CTF_G8 | GX_CTF_B8
        | GX_CTF_Z8M | GX_CTF_Z8L => (3, 2),
        GX_TF_IA8 | GX_TF_RGB565 | GX_TF_RGB5A3 | GX_TF_RGBA8 | GX_TF_C14X2 | GX_TF_Z16 | GX_TF_Z24X8 | GX_CTF_RA8
        | GX_CTF_RG8 | GX_CTF_GB8 | GX_CTF_Z16L => (2, 2),
        _ => (0, 0),
    };
    let bit_size: u32 = if fmt == GX_TF_RGBA8 || fmt == GX_TF_Z24X8 { 64 } else { 32 };
    let tiles = |w: u16, h: u16| (((w as u32) + (1 << sx) - 1) >> sx) * (((h as u32) + (1 << sy) - 1) >> sy);
    if !mips {
        return bit_size.wrapping_mul(tiles(width, height));
    }
    let mut len = 0u32;
    while max_lod != 0 {
        len = len.wrapping_add(bit_size.wrapping_mul(tiles(width, height)));
        if width == 1 && height == 1 {
            return len;
        }
        width = if width < 2 { 1 } else { width / 2 };
        height = if height < 2 { 1 } else { height / 2 };
        max_lod -= 1;
    }
    len
}

/// `GXInitLightSpot` (Aurora GXLighting.cpp).
pub fn init_light_spot(o: &mut GxLightObj, cutoff: f32, mut func: u32) {
    if cutoff <= 0.0 || cutoff > 90.0 {
        func = GX_SP_OFF;
    }
    // std::cos(float) on x86 MSVC (and clang i686-msvc) is cosf = (float)cos((double)x): the f64
    // cosine rounded, not a single-precision cosf (they differ by an ulp near 1, which 1 - cr
    // magnifies).
    let cr = (((cutoff * std::f32::consts::PI) / 180.0) as f64).cos() as f32;
    let (mut a0, mut a1, mut a2) = (1.0f32, 0.0f32, 0.0f32);
    match func {
        GX_SP_FLAT => {
            a0 = -1000.0 * cr;
            a1 = 1000.0;
            a2 = 0.0;
        }
        GX_SP_COS => {
            a0 = -cr / (1.0 - cr);
            a1 = 1.0 / (1.0 - cr);
            a2 = 0.0;
        }
        GX_SP_COS2 => {
            a0 = 0.0;
            a1 = -cr / (1.0 - cr);
            a2 = 1.0 / (1.0 - cr);
        }
        GX_SP_SHARP => {
            let d = (1.0 - cr) * (1.0 - cr);
            a0 = cr * (cr - 2.0);
            a1 = 2.0 / d;
            a2 = -1.0 / d;
        }
        GX_SP_RING1 => {
            let d = (1.0 - cr) * (1.0 - cr);
            a0 = 4.0 * cr / d;
            a1 = 4.0 * (1.0 + cr) / d;
            a2 = -4.0 / d;
        }
        GX_SP_RING2 => {
            let d = (1.0 - cr) * (1.0 - cr);
            a0 = 1.0 - 2.0 * cr * cr / d;
            a1 = 4.0 * cr / d;
            a2 = -2.0 / d;
        }
        _ => {}
    }
    o.a0 = a0;
    o.a1 = a1;
    o.a2 = a2;
}

/// `GXInitLightDistAttn` (Aurora GXLighting.cpp).
pub fn init_light_dist_attn(o: &mut GxLightObj, ref_dist: f32, ref_br: f32, mut func: u32) {
    if ref_dist < 0.0 || ref_br < 0.0 || ref_br >= 1.0 {
        func = GX_DA_OFF;
    }
    let (k0, k1, k2) = match func {
        GX_DA_GENTLE => (1.0, (1.0 - ref_br) / (ref_br * ref_dist), 0.0),
        GX_DA_MEDIUM => (
            1.0,
            0.5 * (1.0 - ref_br) / (ref_br * ref_dist),
            0.5 * (1.0 - ref_br) / (ref_br * ref_dist * ref_dist),
        ),
        GX_DA_STEEP => (1.0, 0.0, (1.0 - ref_br) / (ref_br * ref_dist * ref_dist)),
        _ => (1.0, 0.0, 0.0),
    };
    o.k0 = k0;
    o.k1 = k1;
    o.k2 = k2;
}

// ------------------------------------------------------------------ the shim

/// `gw_gx_get_stats` counters (+ `gw_gx_texobj_inits`, MELEE_PROFILE's).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GxStats {
    /// Finished EFB display copies (frames).
    pub copies: u32,
    /// GXBegin primitives.
    pub prims: u32,
    /// Display lists called (not counting suppressed ones).
    pub dlists: u32,
    pub tex_copies: u32,
    pub texobj_inits: u32,
}

/// HSD's draw-done callback (`GXSetDrawDoneCallback`).
pub type DrawDoneCallback = fn();

/// The GX entry points the game calls, in front of [`Gx`]. One per GX context (the game has one).
pub struct GxShim {
    pub gx: Gx,
    pub ind: IndGuard,
    ind_neutralized: u32,
    pub stats: GxStats,
    /// `gw_gx_suppress_draws`: a resimulated frame's display lists are skipped.
    pub suppress_draws: bool,
    /// `gw_Fx_ShakeOffset`'s output for this render frame (NDC shift; zero when no shake).
    pub shake_offset: [f32; 2],
    draw_done_cb: Option<DrawDoneCallback>,
    /// Async decode: [`GxShim::take_frame`] waits for the frame of the last GXCopyDisp (screenshots).
    pub wait_frame: bool,
    /// The last finished frame (GXCopyDisp), waiting for the presenter.
    frame: Option<Vec<DrawCmd>>,
    /// Log lines (`gw_log` in the C); the platform layer drains and prints them.
    pub log: Vec<String>,
    stub_logged: u32,
    /// GXSetTexCopySrc (x, y, w, h) and GXSetTexCopyDst (w, h, format, mipmap) for the next GXCopyTex.
    tex_copy_src: [u16; 4],
    tex_copy_dst: (u16, u16, u32, bool),
}

impl Default for GxShim {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! fwd {
    ($(#[$m:meta])* $name:ident => $inner:ident ( $($a:ident : $t:ty),* )) => {
        $(#[$m])*
        pub fn $name(&mut self, $($a: $t),*) { self.gx.$inner($($a),*) }
    };
}

impl GxShim {
    /// `gw_GXInit`: a fresh GX (GXInit) and a reset indirect guard.
    pub fn new() -> Self {
        Self {
            gx: Gx::new(),
            ind: IndGuard::default(),
            ind_neutralized: 0,
            stats: GxStats::default(),
            suppress_draws: false,
            shake_offset: [0.0; 2],
            draw_done_cb: None,
            frame: None,
            wait_frame: false,
            log: Vec::new(),
            stub_logged: 0,
            tex_copy_src: [0, 0, 640, 480],
            tex_copy_dst: (640, 480, 6, false),
        }
    }
    /// `gw_GXInit` on an existing shim (keeps the counters, as the C's statics do).
    pub fn init(&mut self) {
        self.ind = IndGuard::default();
        self.gx = Gx::new();
    }

    fn stub(&mut self, name: &str) {
        // GW_STUB logs each stub once.
        let bit = match name {
            "GXSetTevClampMode" => 1,
            "GXSetCopyClamp" => 2,
            "GXSetMisc" => 4,
            _ => 8,
        };
        if self.stub_logged & bit == 0 {
            self.stub_logged |= bit;
            self.log.push(format!("gw: stub {name}"));
        }
    }

    fn guard_indirect(&mut self) {
        for s in self.ind.stages_to_neutralize() {
            let n = self.ind_neutralized;
            self.ind_neutralized += 1;
            if n < 8 {
                let i = s as usize;
                let ind = self.ind.tev_ind_stage[i] as usize;
                self.log.push(format!(
                    "gx: indirect TEV stage {s} reads texcoord {}/{} but only {} texgen(s) are set up - drawn direct instead (a leaked refraction setup; Aurora would abort on it)",
                    self.ind.tev_coord[i], self.ind.ind_coord[ind], self.ind.num_texgens
                ));
            }
            self.gx.set_tev_direct(s);
        }
    }

    // ---- draw-done and presentation

    /// `gw_GXSetDrawDone` -> shim_vi.c `gw_gx_set_draw_done`: GXSetDrawDone, then HSD's callback
    /// synchronously (HSD hangs its XFB state machine off it).
    pub fn set_draw_done(&mut self) {
        self.gx.draw_done();
        if let Some(cb) = self.draw_done_cb {
            cb();
        }
    }
    /// `gw_GXWaitDrawDone` -> `gw_gx_wait_draw_done`: GXDrawDone, then the callback.
    pub fn wait_draw_done(&mut self) {
        self.gx.draw_done();
        if let Some(cb) = self.draw_done_cb {
            cb();
        }
    }
    /// `gw_GXSetDrawDoneCallback`: returns the previous callback.
    pub fn set_draw_done_callback(&mut self, cb: Option<DrawDoneCallback>) -> Option<DrawDoneCallback> {
        std::mem::replace(&mut self.draw_done_cb, cb)
    }
    pub fn flush(&mut self) {
        self.gx.flush();
    }
    pub fn pix_mode_sync(&mut self) {
        self.gx.pix_mode_sync();
    }

    /// `gw_GXCopyDisp`: the end of a frame. Aurora's GXCopyDisp is a no-op and its frame ends at
    /// `aurora_end_frame`; here the FIFO is decoded and the frame's draws wait for
    /// [`GxShim::take_frame`]. `clear` is the copy-clear the next frame starts from (the renderer
    /// clears to the copy-clear colour at the start of every frame).
    pub fn copy_disp(&mut self, _clear: bool) {
        if self.gx.is_async() {
            self.gx.draw_done();
            self.gx.end_frame();
            self.stats.copies = self.stats.copies.wrapping_add(1);
            return;
        }
        self.gx.draw_done();
        #[cfg(feature = "corev1")]
        crate::corev1::frame_end(_clear);
        let draws = self.gx.state.take_draws();
        if std::env::var_os("MELEE_GX_WARN").is_some() { for w in &self.gx.state.warnings { eprintln!("gx warn: {w}"); } }
        self.gx.state.warnings.clear();
        if let Ok(v) = std::env::var("MELEE_GX_DRAWLOG") {
            if v.parse::<u32>().ok() == Some(self.stats.copies) {
                for (i, d) in draws.iter().enumerate() {
                    let c = &d.config;
                    eprintln!("DRAW {i} prim {:#x} verts {} blend {} {} {} op {} depth fn {} cmp {} upd {} alpha_upd {} color_upd {} cull {} tex {:#x} stages {}", d.prim, d.vtx_count, c.blend_mode, c.blend_fac_src, c.blend_fac_dst, c.blend_op, c.depth_func, c.depth_compare, c.depth_update, c.alpha_update, c.color_update, c.cull_mode, d.info.sampled_textures, c.shader.tev_stage_count);
                    if std::env::var("MELEE_GX_DRAWLOG_TEV").ok().and_then(|x| x.parse::<usize>().ok()) == Some(i) {
                        for k in 0..c.shader.tev_stage_count as usize { eprintln!("   TEV {k}: {:?}", c.shader.tev_stages[k]); }
                        eprintln!("   ALPHACMP {:?}", c.shader.alpha_compare);
                        eprintln!("   CHAN {:?}", &c.shader.color_channels[..2]);
                        eprintln!("   TCG {:?}", &c.shader.tcgs[..2]);
                        eprintln!("   ATTR {:?} fmt {} verts {:02x?}", &c.shader.attrs[..13], d.fmt, &d.verts[..d.verts.len().min(96)]);
                        for t in 0..2 {
                            let x = &d.textures[t];
                            eprintln!("   TEX{t} {}x{} fmt {} tlut {} mode0 {:#x} mode1 {:#x}", x.width, x.height, x.format, x.tlut, x.mode0, x.mode1);
                        }
                        for (k, t) in d.tluts.iter().enumerate() {
                            let bytes = t.data.as_ref().map(|g| g[..g.len().min(64)].to_vec());
                            eprintln!("   TLUT{k} tlut {} load {:#x} n {} fmt {} data {:02x?}", t.tlut, t.load_tlut0, t.num_entries, t.format, bytes);
                        }
                        eprintln!("   UBUF {:02x?}", &d.uniform[..d.uniform.len().min(512)]);
                    }
                }
            }
        }
        self.frame = Some(draws);
        self.gx.pose = self.gx.frame_pose.take();
        self.stats.copies = self.stats.copies.wrapping_add(1);
    }
    /// `gw_GXCopyTex`: counted; EFB-to-texture copies are not implemented in melee_gx (the decoder
    /// warns once per copy). Aurora resolves the EFB and, with `clear`, clears it.
    pub fn copy_tex(&mut self, dest: u64, clear: bool) {
        self.stats.tex_copies = self.stats.tex_copies.wrapping_add(1);
        let (src, (w, h, fmt, half)) = (self.tex_copy_src, self.tex_copy_dst);
        if { static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new(); *ON.get_or_init(|| std::env::var_os("MELEE_GX_TEXCOPYLOG").is_some()) } { eprintln!("TEXCOPY copy# {} src {:?} {}x{} fmt {:#x}", self.stats.copies, src, w, h, fmt); }
        self.gx.copy_tex(dest, src, w, h, fmt, half, clear);
    }
    /// The last frame GXCopyDisp finished, if any (the presenter renders it against
    /// `self.gx.state`).
    pub fn take_frame(&mut self) -> Option<Vec<DrawCmd>> {
        if self.gx.is_async() {
            return self.gx.take_async_frame(self.wait_frame);
        }
        self.frame.take()
    }
    /// `gw_gx_get_stats`.
    pub fn get_stats(&self) -> (u32, u32, u32) {
        (self.stats.copies, self.stats.prims, self.stats.dlists)
    }

    // ---- geometry

    /// `gw_GXBegin`: the indirect guard runs before every draw.
    pub fn begin(&mut self, prim: u32, fmt: u32, n: u16) {
        self.guard_indirect();
        self.stats.prims = self.stats.prims.wrapping_add(1);
        self.gx.begin(prim, fmt, n);
    }
    pub fn end(&mut self) {
        self.gx.end();
    }
    /// `gw_GXCallDisplayList`: skipped while draws are suppressed.
    pub fn call_display_list(&mut self, list: &[u8]) {
        if self.suppress_draws {
            return;
        }
        self.stats.dlists = self.stats.dlists.wrapping_add(1);
        self.guard_indirect();
        self.gx.call_display_list(list);
    }
    /// `gw_Gx_SuppressDraws`.
    pub fn set_suppress_draws(&mut self, on: bool) {
        // core v1: a re-simulated frame must not reach the host's screen (no draws, no copies); what was written before
        // the switch is the shown frame's and goes out as it is
        #[cfg(feature = "corev1")]
        // (this shim is game state a load restores, so its flag is no guide: the transcoder keeps its own)
        if crate::corev1::active() {
            self.gx.drain();
            crate::corev1::set_quiet(on);
        }
        self.suppress_draws = on;
    }
    fwd!(clear_vtx_desc => clear_vtx_desc());
    fwd!(invalidate_vtx_cache => invalidate_vtx_cache());
    fwd!(enable_tex_offsets => enable_tex_offsets(coord: u32, line: bool, point: bool));
    fwd!(set_line_width => set_line_width(w: u8, offs: u32));
    fwd!(set_point_size => set_point_size(s: u8, offs: u32));
    fwd!(set_vtx_desc => set_vtx_desc(attr: u32, ty: u32));
    fwd!(set_vtx_attr_fmt => set_vtx_attr_fmt(fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8));
    pub fn set_num_tex_gens(&mut self, n: u8) {
        self.ind.num_texgens = n as u32;
        self.gx.set_num_tex_gens(n);
    }
    pub fn set_tex_coord_gen2(&mut self, dst: u32, func: u32, src: u32, mtx: u32, normalize: bool, pt_mtx: u32) {
        if dst < 8 {
            self.ind.texgen_seen |= 1 << dst;
        }
        self.gx.set_tex_coord_gen2(dst, func, src, mtx, normalize, pt_mtx);
    }
    /// The 3-argument GameCube `GXSetArray`. Aurora bounds-checks indexed loads against the
    /// array's byte length; the C passes "base to end of MEM1", here the buffer's own length.
    /// The data is big-endian file data (le = false).
    pub fn set_array(&mut self, attr: u32, data: Option<&GxData>, stride: u8) {
        let size = data.map_or(u32::MAX, |d| d.len().min(u32::MAX as usize) as u32);
        self.gx.set_array(attr, data, size, stride, false);
    }

    // ---- transform

    /// `gw_GXSetProjection`, with the R6 camera shake: `ndc.x = (P00*x + P02*z) / -z`, so a P02 of
    /// -s shifts every point by +s (a cosmetic NDC shift; game cameras are untouched).
    pub fn set_projection(&mut self, m: &[[f32; 4]; 4], ty: u32) {
        let mut m = *m;
        if ty == GX_PERSPECTIVE && m[3][2] == -1.0 {
            m[0][2] -= self.shake_offset[0];
            m[1][2] -= self.shake_offset[1];
        }
        self.gx.set_projection(&m, ty);
    }
    fwd!(load_pos_mtx_imm => load_pos_mtx_imm(m: &[[f32; 4]; 3], id: u32));
    fwd!(load_palette => load_palette(n: u32, key: u32, data: &[f32]));
    fwd!(end_palette => end_palette());
    fwd!(load_nrm_mtx_imm => load_nrm_mtx_imm(m: &[[f32; 4]; 3], id: u32));
    /// `gw_GXLoadTexMtxImm`: a 2x4 matrix is the first two rows.
    pub fn load_tex_mtx_imm(&mut self, m: &[[f32; 4]; 3], id: u32, ty: u32) {
        let mut n = *m;
        if ty == GX_MTX2x4 {
            n[2] = [0.0; 4];
        }
        self.gx.load_tex_mtx_imm(&n, id, ty);
    }
    fwd!(set_current_mtx => set_current_mtx(id: u32));
    fwd!(set_viewport => set_viewport(left: f32, top: f32, w: f32, h: f32, nz: f32, fz: f32));
    fwd!(set_viewport_jitter => set_viewport_jitter(left: f32, top: f32, w: f32, h: f32, nz: f32, fz: f32, field: u32));
    /// `gw_GXProject` (see [`gx_project`]).
    #[allow(clippy::too_many_arguments)]
    pub fn project(&self, x: f32, y: f32, z: f32, mtx: &[[f32; 4]; 3], pm: &[f32; 7], vp: &[f32; 6]) -> (f32, f32, f32) {
        gx_project(x, y, z, mtx, pm, vp)
    }
    pub fn get_projectionv(&self) -> [f32; 7] {
        self.gx.get_projectionv()
    }
    pub fn get_viewportv(&self) -> [f32; 6] {
        self.gx.get_viewportv()
    }

    // ---- texture

    #[allow(clippy::too_many_arguments)]
    pub fn init_tex_obj(&mut self, o: &mut TexObj, data: Option<GxData>, w: u16, h: u16, fmt: u32, ws: u32, wt: u32, mip: bool) {
        self.stats.texobj_inits = self.stats.texobj_inits.wrapping_add(1);
        self.gx.init_tex_obj(o, data, w, h, fmt, ws, wt, mip);
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
        self.stats.texobj_inits = self.stats.texobj_inits.wrapping_add(1);
        self.gx.init_tex_obj_ci(o, data, w, h, fmt, ws, wt, mip, tlut);
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
        Gx::init_tex_obj_lod(o, min_filt, mag_filt, min_lod, max_lod, lod_bias, bias_clamp, edge_lod, aniso);
    }
    pub fn init_tlut_obj(&mut self, o: &mut TlutObj, data: Option<GxData>, fmt: u32, n: u16) {
        self.gx.init_tlut_obj(o, data, fmt, n);
    }
    pub fn load_tex_obj(&mut self, o: &mut TexObj, id: u32) {
        self.gx.load_tex_obj(o, id);
    }
    pub fn load_tlut(&mut self, o: &TlutObj, id: u32) {
        self.gx.load_tlut(o, id);
    }
    /// Aurora's GXInvalidateTexAll is a no-op.
    pub fn invalidate_tex_all(&mut self) {}
    pub fn get_tex_obj_fmt(o: &TexObj) -> u32 {
        o.format
    }
    pub fn get_tex_obj_width(o: &TexObj) -> u16 {
        o.width as u16
    }
    pub fn get_tex_obj_height(o: &TexObj) -> u16 {
        o.height as u16
    }

    // ---- lighting

    pub fn init_light_attn(o: &mut GxLightObj, a0: f32, a1: f32, a2: f32, k0: f32, k1: f32, k2: f32) {
        Gx::init_light_attn(o, a0, a1, a2, k0, k1, k2);
    }
    pub fn init_light_pos(o: &mut GxLightObj, x: f32, y: f32, z: f32) {
        Gx::init_light_pos(o, x, y, z);
    }
    pub fn init_light_dir(o: &mut GxLightObj, x: f32, y: f32, z: f32) {
        Gx::init_light_dir(o, x, y, z);
    }
    pub fn init_light_color(o: &mut GxLightObj, c: GxColor) {
        Gx::init_light_color(o, c);
    }
    fwd!(load_light_obj_imm => load_light_obj_imm(o: &GxLightObj, id: u32));
    fwd!(set_num_chans => set_num_chans(n: u8));
    #[allow(clippy::too_many_arguments)]
    pub fn set_chan_ctrl(&mut self, chan: u32, enable: bool, amb: u32, mat: u32, lights: u32, diff: u32, attn: u32) {
        self.gx.set_chan_ctrl(chan, enable, amb, mat, lights, diff, attn);
    }
    fwd!(set_chan_amb_color => set_chan_amb_color(id: u32, c: GxColor));
    fwd!(set_chan_mat_color => set_chan_mat_color(id: u32, c: GxColor));

    // ---- TEV

    pub fn set_num_tev_stages(&mut self, n: u8) {
        self.ind.num_tev = n as u32;
        self.gx.set_num_tev_stages(n);
    }
    fwd!(set_tev_op => set_tev_op(id: u32, mode: u32));
    fwd!(set_tev_color_in => set_tev_color_in(stage: u32, a: u32, b: u32, c: u32, d: u32));
    fwd!(set_tev_alpha_in => set_tev_alpha_in(stage: u32, a: u32, b: u32, c: u32, d: u32));
    fwd!(set_tev_color_op => set_tev_color_op(stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32));
    fwd!(set_tev_alpha_op => set_tev_alpha_op(stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32));
    fwd!(set_tev_color => set_tev_color(id: u32, c: GxColor));
    /// `gw_GXSetTevColorS10`: native s16 fields (the C swaps them from the big-endian copy).
    pub fn set_tev_color_s10(&mut self, id: u32, c: GxColorS10) {
        self.gx.set_tev_color_s10(id, c);
    }
    fwd!(set_tev_kcolor => set_tev_kcolor(id: u32, c: GxColor));
    fwd!(set_tev_kcolor_sel => set_tev_kcolor_sel(id: u32, sel: u32));
    fwd!(set_tev_kalpha_sel => set_tev_kalpha_sel(id: u32, sel: u32));
    fwd!(set_tev_swap_mode => set_tev_swap_mode(stage: u32, ras: u32, tex: u32));
    fwd!(set_tev_swap_mode_table => set_tev_swap_mode_table(id: u32, r: u32, g: u32, b: u32, a: u32));
    pub fn set_tev_order(&mut self, stage: u32, coord: u32, map: u32, color: u32) {
        if (stage as usize) < GW_GX_TEV_MAX {
            self.ind.tev_coord[stage as usize] = if coord >= 8 { 0 } else { coord as u8 };
        }
        self.gx.set_tev_order(stage, coord, map, color);
    }
    fwd!(set_alpha_compare => set_alpha_compare(comp0: u32, ref0: u8, op: u32, comp1: u32, ref1: u8));
    fwd!(set_z_texture => set_z_texture(op: u32, fmt: u32, bias: u32));
    pub fn set_tev_direct(&mut self, stage: u32) {
        if (stage as usize) < GW_GX_TEV_MAX {
            let i = stage as usize;
            self.ind.tev_ind_live[i] = 0;
            self.ind.tev_ind_matrix[i] = 0;
            self.ind.tev_ind_stage[i] = 0;
        }
        self.gx.set_tev_direct(stage);
    }
    /// GC-only register; Aurora does not implement it (logged stub).
    pub fn set_tev_clamp_mode(&mut self, _stage: u32, _mode: u32) {
        self.stub("GXSetTevClampMode");
    }

    // ---- indirect

    pub fn set_num_ind_stages(&mut self, n: u8) {
        self.ind.num_ind = n as u32;
        self.gx.set_num_ind_stages(n);
    }
    pub fn set_ind_tex_order(&mut self, stage: u32, coord: u32, map: u32) {
        if (stage as usize) < GW_GX_IND_MAX {
            self.ind.ind_coord[stage as usize] = if coord >= 8 { 0 } else { coord as u8 };
        }
        self.gx.set_ind_tex_order(stage, coord, map);
    }
    fwd!(set_ind_tex_coord_scale => set_ind_tex_coord_scale(stage: u32, s: u32, t: u32));
    fwd!(set_ind_tex_mtx => set_ind_tex_mtx(id: u32, m: &[[f32; 3]; 2], scale_exp: i8));
    #[allow(clippy::too_many_arguments)]
    pub fn set_tev_indirect(
        &mut self,
        tev_stage: u32,
        ind_stage: u32,
        fmt: u32,
        bias: u32,
        matrix: u32,
        wrap_s: u32,
        wrap_t: u32,
        add_prev: bool,
        utc_lod: bool,
        alpha: u32,
    ) {
        if (tev_stage as usize) < GW_GX_TEV_MAX {
            let i = tev_stage as usize;
            self.ind.tev_ind_stage[i] = (ind_stage & 3) as u8;
            self.ind.tev_ind_matrix[i] = (matrix != GX_ITM_OFF) as u8;
            self.ind.tev_ind_live[i] =
                (matrix != GX_ITM_OFF || wrap_s != GX_ITW_OFF || wrap_t != GX_ITW_OFF || add_prev) as u8;
        }
        self.gx.set_tev_indirect(tev_stage, ind_stage, fmt, bias, matrix, wrap_s, wrap_t, add_prev, utc_lod, alpha);
    }

    // ---- pixel / frame buffer

    /// `gw_GXSetFogRangeAdj`.
    pub fn set_fog_range_adj(&mut self, enable: bool, center: u16, table: Option<&[u16; 10]>) {
        self.gx.set_fog_range_adj(enable, center, if enable { table } else { None });
    }
    #[allow(clippy::too_many_arguments)]
    pub fn set_fog(&mut self, ty: u32, start_z: f32, end_z: f32, near_z: f32, far_z: f32, color: GxColor) {
        self.gx.set_fog(ty, start_z, end_z, near_z, far_z, color);
    }
    fwd!(set_blend_mode => set_blend_mode(mode: u32, src: u32, dst: u32, op: u32));
    fwd!(set_color_update => set_color_update(e: bool));
    fwd!(set_alpha_update => set_alpha_update(e: bool));
    fwd!(set_z_mode => set_z_mode(compare: bool, func: u32, update: bool));
    fwd!(set_z_comp_loc => set_z_comp_loc(before_tex: bool));
    fwd!(set_pixel_fmt => set_pixel_fmt(pix: u32, z: u32));
    fwd!(set_dither => set_dither(d: bool));
    fwd!(set_dst_alpha => set_dst_alpha(enable: bool, value: u8));
    fwd!(set_field_mode => set_field_mode(field_mode: bool, half_aspect: bool));
    fwd!(set_copy_clear => set_copy_clear(c: GxColor, depth: u32));
    /// Aurora declares GXSetCopyClamp but does not define it (logged stub).
    pub fn set_copy_clamp(&mut self, _clamp: u32) {
        self.stub("GXSetCopyClamp");
    }
    /// No-ops in Aurora (GXFrameBuffer.cpp): the display copy is the swapchain present.
    pub fn set_copy_filter(&mut self, _aa: bool, _pattern: &[[u8; 2]; 12], _vf: bool, _vfilter: &[u8; 7]) {}
    pub fn set_disp_copy_src(&mut self, _left: u16, _top: u16, _wd: u16, _ht: u16) {}
    pub fn set_disp_copy_dst(&mut self, _wd: u16, _ht: u16) {}
    /// Aurora returns 0.
    pub fn set_disp_copy_y_scale(&mut self, _vscale: f32) -> u32 {
        0
    }
    pub fn set_disp_copy_gamma(&mut self, _gamma: u32) {}
    /// EFB-to-texture copy setup: recorded by Aurora for GXCopyTex, which melee_gx does not
    /// implement yet (see [`GxShim::copy_tex`]).
    pub fn set_tex_copy_src(&mut self, left: u16, top: u16, wd: u16, ht: u16) {
        self.tex_copy_src = [left, top, wd, ht];
    }
    pub fn set_tex_copy_dst(&mut self, wd: u16, ht: u16, fmt: u32, mipmap: bool) {
        self.tex_copy_dst = (wd, ht, fmt, mipmap);
    }

    // ---- culling / scissor / manage

    fwd!(set_scissor => set_scissor(left: u32, top: u32, w: u32, h: u32));
    fwd!(set_cull_mode => set_cull_mode(mode: u32));
    /// `GXSetPipelineWaitAURORA`: melee_gx compiles pipelines synchronously, so every draw already
    /// waits for its pipeline.
    pub fn set_pipeline_wait(&mut self, _on: bool) {}
    /// GC-internal flush tuning; nothing to do (logged stub).
    pub fn set_misc(&mut self, _token: u32, _val: u32) {
        self.stub("GXSetMisc");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use melee_core::gc::GcLayout;

    #[test]
    fn render_mode_layout_and_values() {
        assert_eq!(GxRenderModeObj::GC_SIZE, 60);
        assert_eq!(GxRenderModeObj::gc_field("xfb_mode").unwrap().offset, 0x14);
        assert_eq!(GxRenderModeObj::gc_field("vfilter").unwrap().offset, 0x32);
        assert_eq!(GX_NTSC_480_INT_DF.vfilter, [8, 8, 10, 12, 10, 8, 8]);
        assert_eq!(GX_NTSC_480_PROG.xfb_mode, VI_XFBMODE_SF);
        assert_eq!(GX_NTSC_480_PROG.vi_tv_mode, VI_TVMODE_NTSC_PROG);
    }

    /// The leaked-refraction case from shim_gx.c's comment: one indirect stage armed on TEV stage
    /// 0 with ITM_0, then an untextured draw (no texgens): the guard turns stage 0 direct, once.
    #[test]
    fn indirect_guard_neutralizes_leaked_refraction() {
        let mut s = GxShim::new();
        s.set_num_tex_gens(1);
        s.set_tex_coord_gen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY, false, GX_PTIDENTITY);
        s.set_num_ind_stages(1);
        s.set_ind_tex_order(GX_INDTEXSTAGE0, GX_TEXCOORD0, GX_TEXMAP0);
        s.set_tev_indirect(GX_TEVSTAGE0, GX_INDTEXSTAGE0, GX_ITF_8, GX_ITB_ST, GX_ITM_0, GX_ITW_OFF, GX_ITW_OFF, false, false, GX_ITBA_OFF);
        s.set_num_tev_stages(1);
        s.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        // Live setup: texcoord 0 generated, nothing to do.
        assert!(s.ind.clone().stages_to_neutralize().is_empty());
        // The untextured quad that follows the skipped reset.
        s.set_num_tex_gens(0);
        s.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
        s.set_vtx_desc(GX_VA_POS, GX_DIRECT);
        s.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
        s.begin(GX_QUADS, GX_VTXFMT0, 4);
        for _ in 0..4 {
            s.gx.position3f32(0.0, 0.0, 0.0);
        }
        s.end();
        assert_eq!(s.log.len(), 1, "{:?}", s.log);
        assert!(s.log[0].contains("indirect TEV stage 0"));
        assert_eq!(s.ind.tev_ind_live[0], 0);
        assert_eq!(s.stats.prims, 1);
        // Already direct: a second draw logs nothing.
        s.begin(GX_QUADS, GX_VTXFMT0, 4);
        for _ in 0..4 {
            s.gx.position3f32(0.0, 0.0, 0.0);
        }
        s.end();
        assert_eq!(s.log.len(), 1);
        s.copy_disp(true);
        let frame = s.take_frame().expect("frame");
        assert_eq!(frame.len(), 2);
        assert_eq!(s.stats.copies, 1);
        // The decoded shader for the guarded draw has no indirect stage.
        assert!(frame.iter().all(|d| d.info.sampled_ind_textures == 0));
    }

    #[test]
    fn suppressed_display_lists_are_skipped() {
        let mut s = GxShim::new();
        s.set_suppress_draws(true);
        s.call_display_list(&[0u8; 32]);
        assert_eq!(s.stats.dlists, 0);
        s.set_suppress_draws(false);
        s.call_display_list(&[0u8; 32]);
        assert_eq!(s.stats.dlists, 1);
    }

    #[test]
    fn draw_done_callback_routes() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static HITS: AtomicU32 = AtomicU32::new(0);
        fn cb() {
            HITS.fetch_add(1, Ordering::Relaxed);
        }
        let mut s = GxShim::new();
        assert!(s.set_draw_done_callback(Some(cb)).is_none());
        s.set_draw_done();
        s.wait_draw_done();
        assert_eq!(HITS.load(Ordering::Relaxed), 2);
        assert!(s.set_draw_done_callback(None).is_some());
    }

    #[test]
    fn projection_shake_and_getters() {
        let mut s = GxShim::new();
        let p = [[1.5, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0], [0.0, 0.0, -1.01, -10.0], [0.0, 0.0, -1.0, 0.0]];
        s.shake_offset = [0.25, -0.5];
        s.set_projection(&p, GX_PERSPECTIVE);
        assert_eq!(s.get_projectionv(), [0.0, 1.5, -0.25, 2.0, 0.5, -1.01, -10.0]);
        s.set_viewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
        assert_eq!(s.get_viewportv(), [0.0, 0.0, 640.0, 480.0, 0.0, 1.0]);
    }
}
