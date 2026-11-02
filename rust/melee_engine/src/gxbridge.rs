//! The GX the engine's render pass calls (`GXSetTevOrder`, `GXLoadTexObj`, `GXCallDisplayList`, ...) on melee_gx's
//! `GxShim`: pc/platform/shim_gx.c for the native engine. Seams only; every function converts the C call shape
//! (guest-style pointers into the OS-heap block, `GXTexObj` records the game owns) into the shim's own types and
//! forwards. The install table is generated (`gxbridge_gen.rs`, tools/rust-port/engine/gen_gx_bridge.py) from the
//! signatures below.
//!
//! Game memory is big-endian image data (textures, vertex arrays, display lists) read by the GPU path as it is;
//! matrices and floats the game computed are native. Texture / TLUT / light objects the C keeps inside game
//! structs are kept in side tables here, keyed by the address of the game's record (the game never reads them).
#![allow(non_snake_case, clippy::missing_safety_doc, static_mut_refs)]

use core::ffi::c_void;
use melee_gx::api::{GxColor, GxColorS10, GxLightObj};
use melee_gx::draw::DrawCmd;
use melee_gx::shim::{get_tex_buffer_size, GxShim};
use melee_gx::state::{TexObj, TlutObj};
use melee_gx::GxData;
use melee_types::{_GXColor, _GXColorS10};
use std::collections::HashMap;

#[derive(Default)]
struct Objs {
    tex: HashMap<usize, TexObj>,
    tlut: HashMap<usize, TlutObj>,
    light: HashMap<usize, GxLightObj>,
    data: HashMap<(usize, usize), GxData>,
}

static mut SHIM: Option<GxShim> = None;
static mut OBJS: Option<Objs> = None;

/// A GXBegin is open (the game's GXEnd is a macro that vanished in the port: the bridge ends the primitive itself).
static mut OPEN: bool = false;

/// The shim every state call lands in (created on first use; `GxShim::new` is `GXInit`). Ends an open primitive first.
pub fn shim() -> &'static mut GxShim {
    unsafe {
        let s = SHIM.get_or_insert_with(GxShim::new);
        if OPEN {
            OPEN = false;
            s.end();
        }
        s
    }
}
/// The shim for immediate-mode vertex data (does not end the primitive).
fn vtx() -> &'static mut GxShim {
    unsafe { SHIM.get_or_insert_with(GxShim::new) }
}
fn objs() -> &'static mut Objs {
    unsafe { OBJS.get_or_insert_with(Objs::default) }
}

/// The finished frame (`GXCopyDisp` ends it), for the presenter.
pub fn take_frame() -> Option<Vec<DrawCmd>> {
    shim().take_frame()
}

fn col(c: _GXColor) -> GxColor {
    GxColor::new(c.r, c.g, c.b, c.a)
}

/// How many bytes of game memory at `p` the GPU path may read: to the end of the archive image or heap block holding it.
unsafe fn extent(p: usize, cap: usize) -> usize {
    let h = crate::host::Host::get();
    for im in h.archives.values().chain(h.retired.iter()) {
        if p >= im.raw && p < im.raw + im.raw_len {
            return (im.raw + im.raw_len - p).min(cap);
        }
    }
    let m = h.os.mem;
    let (b, e) = (m.base as usize, m.base as usize + m.len);
    if p >= b && p < e {
        return (e - p).min(cap.min(256 << 10));
    }
    0
}

/// Diagnostics: (entries, bytes) of the copied game-memory cache.
pub fn data_cache_stats() -> (usize, usize) {
    let o = objs();
    (o.data.len(), o.data.values().map(|d| d.len()).sum())
}

/// The bytes at `p` as the shared data the shim hands to the renderer (cached by address and length).
unsafe fn data_at(p: *const c_void, want: usize) -> Option<GxData> {
    if p.is_null() {
        return None;
    }
    let n = extent(p as usize, want);
    if n == 0 {
        return None;
    }
    let key = (p as usize, n);
    let o = objs();
    if let Some(d) = o.data.get(&key) {
        return Some(d.clone());
    }
    let d: GxData = core::slice::from_raw_parts(p as *const u8, n).into();
    o.data.insert(key, d.clone());
    Some(d)
}

unsafe fn mtx34<'a>(p: *mut c_void) -> &'a [[f32; 4]; 3] {
    &*(p as *const [[f32; 4]; 3])
}
unsafe fn mtx44<'a>(p: *mut c_void) -> &'a [[f32; 4]; 4] {
    &*(p as *const [[f32; 4]; 4])
}

// ---- vertex description and immediate geometry
pub fn GXBegin(prim: u32, fmt: u32, n: u16) {
    let s = shim();
    unsafe { OPEN = true };
    s.begin(prim, fmt, n)
}
pub fn GXClearVtxDesc() { shim().clear_vtx_desc() }
pub fn GXInvalidateVtxCache() { shim().invalidate_vtx_cache() }
pub fn GXSetVtxDesc(attr: u32, ty: u32) { shim().set_vtx_desc(attr, ty) }
pub fn GXSetVtxAttrFmt(fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8) { shim().set_vtx_attr_fmt(fmt, attr, cnt, ty, frac) }
pub unsafe fn GXSetArray(attr: u32, base: *mut c_void, stride: u8) {
    let d = data_at(base, usize::MAX);
    shim().set_array(attr, d.as_ref(), stride)
}
pub unsafe fn GXCallDisplayList(list: *mut c_void, nbytes: u32) {
    if list.is_null() || nbytes == 0 {
        return;
    }
    shim().call_display_list(core::slice::from_raw_parts(list as *const u8, nbytes as usize))
}
pub fn GXPosition3f32(x: f32, y: f32, z: f32) { vtx().gx.position3f32(x, y, z) }
pub fn GXPosition2f32(x: f32, y: f32) { vtx().gx.position2f32(x, y) }
pub fn GXNormal3f32(x: f32, y: f32, z: f32) { vtx().gx.normal3f32(x, y, z) }
pub fn GXColor4u8(r: u8, g: u8, b: u8, a: u8) { vtx().gx.color4u8(r, g, b, a) }
pub fn GXColor3u8(r: u8, g: u8, b: u8) { vtx().gx.color4u8(r, g, b, 255) }
pub fn GXColor1x8(i: u8) { vtx().gx.color1x8(i) }
pub fn GXColor1x16(i: u16) { vtx().gx.cmd1u16(i) }
pub fn GXColor1u16(c: u16) { vtx().gx.cmd1u16(c) }
pub fn GXTexCoord2f32(s: f32, t: f32) { vtx().gx.tex_coord2f32(s, t) }
pub fn GXTexCoord2u8(s: u8, t: u8) { vtx().gx.cmd1u8(s); vtx().gx.cmd1u8(t) }
pub fn GXTexCoord1x8(i: u8) { vtx().gx.tex_coord1x8(i) }
pub fn GXTexCoord1u8(i: u8) { vtx().gx.cmd1u8(i) }
pub fn GXTexCoord1x16(i: u16) { vtx().gx.cmd1u16(i) }

// ---- transform
pub unsafe fn GXSetProjection(m: *mut c_void, ty: u32) { shim().set_projection(mtx44(m), ty) }
pub unsafe fn GXLoadPosMtxImm(m: *mut c_void, id: u32) { shim().load_pos_mtx_imm(mtx34(m), id) }
pub unsafe fn GXLoadNrmMtxImm(m: *mut c_void, id: u32) { shim().load_nrm_mtx_imm(mtx34(m), id) }
pub unsafe fn GXLoadTexMtxImm(m: *mut c_void, id: u32, ty: u32) { shim().load_tex_mtx_imm(mtx34(m), id, ty) }
pub fn GXSetCurrentMtx(id: u32) { shim().set_current_mtx(id) }
pub fn GXSetViewport(l: f32, t: f32, w: f32, h: f32, nz: f32, fz: f32) { shim().set_viewport(l, t, w, h, nz, fz) }
pub fn GXSetViewportJitter(l: f32, t: f32, w: f32, h: f32, nz: f32, fz: f32, field: u32) { shim().set_viewport_jitter(l, t, w, h, nz, fz, field) }
pub fn GXSetScissor(l: u32, t: u32, w: u32, h: u32) { shim().set_scissor(l, t, w, h) }
pub unsafe fn GXGetProjectionv(out: *mut f32) {
    let v = shim().get_projectionv();
    core::ptr::copy_nonoverlapping(v.as_ptr(), out, 7);
}
pub unsafe fn GXGetViewportv(out: *mut f32) {
    let v = shim().get_viewportv();
    core::ptr::copy_nonoverlapping(v.as_ptr(), out, 6);
}
pub fn GXSetCullMode(m: u32) { shim().set_cull_mode(m) }
pub fn GXSetLineWidth(w: u8, offs: u32) { shim().set_line_width(w, offs) }
pub fn GXSetPointSize(s: u8, offs: u32) { shim().set_point_size(s, offs) }
pub fn GXEnableTexOffsets(coord: u32, line: u8, point: u8) { shim().enable_tex_offsets(coord, line != 0, point != 0) }

// ---- pixel state
pub fn GXSetBlendMode(ty: u32, src: u32, dst: u32, op: u32) { shim().set_blend_mode(ty, src, dst, op) }
pub fn GXSetZMode(enable: u8, func: u32, update: u8) { shim().set_z_mode(enable != 0, func, update != 0) }
pub fn GXSetZCompLoc(before: u8) { shim().set_z_comp_loc(before != 0) }
pub fn GXSetColorUpdate(on: u8) { shim().set_color_update(on != 0) }
pub fn GXSetAlphaUpdate(on: u8) { shim().set_alpha_update(on != 0) }
pub fn GXSetDither(on: u8) { shim().set_dither(on != 0) }
pub fn GXSetDstAlpha(enable: u8, a: u8) { shim().set_dst_alpha(enable != 0, a) }
pub fn GXSetAlphaCompare(c0: u32, r0: u8, op: u32, c1: u32, r1: u8) { shim().set_alpha_compare(c0, r0, op, c1, r1) }
pub fn GXSetPixelFmt(pix: u32, z: u32) { shim().set_pixel_fmt(pix, z) }
pub fn GXSetFieldMode(field: u8, half: u8) { shim().set_field_mode(field != 0, half != 0) }
pub fn GXSetZTexture(op: u32, fmt: u32, bias: u32) { shim().set_z_texture(op, fmt, bias) }
pub fn GXSetFog(ty: u32, start: f32, end: f32, near: f32, far: f32, c: _GXColor) { shim().set_fog(ty, start, end, near, far, col(c)) }
pub unsafe fn GXInitFogAdjTable(table: *mut c_void, width: u16, m: *mut c_void) {
    let t = melee_gx::shim::init_fog_adj_table(width, mtx44(m));
    core::ptr::copy_nonoverlapping(t.as_ptr(), table as *mut u16, 10);
}
pub unsafe fn GXSetFogRangeAdj(enable: u8, center: u16, table: *mut c_void) {
    let t = if table.is_null() { None } else { Some(&*(table as *const [u16; 10])) };
    shim().set_fog_range_adj(enable != 0, center, t)
}

// ---- channels, lights
pub fn GXSetNumChans(n: u8) { shim().set_num_chans(n) }
pub fn GXSetChanAmbColor(id: u32, c: _GXColor) { shim().set_chan_amb_color(id, col(c)) }
pub fn GXSetChanMatColor(id: u32, c: _GXColor) { shim().set_chan_mat_color(id, col(c)) }
pub fn GXSetChanCtrl(chan: u32, enable: u8, amb: u32, mat: u32, lights: u32, diff: u32, attn: u32) { shim().set_chan_ctrl(chan, enable != 0, amb, mat, lights, diff, attn) }
fn light<'a>(o: *mut c_void) -> &'a mut GxLightObj { objs().light.entry(o as usize).or_default() }
pub unsafe fn GXInitLightAttn(o: *mut c_void, a0: f32, a1: f32, a2: f32, k0: f32, k1: f32, k2: f32) { GxShim::init_light_attn(light(o), a0, a1, a2, k0, k1, k2) }
pub unsafe fn GXInitLightColor(o: *mut c_void, c: _GXColor) { GxShim::init_light_color(light(o), col(c)) }
pub unsafe fn GXInitLightDir(o: *mut c_void, x: f32, y: f32, z: f32) { GxShim::init_light_dir(light(o), x, y, z) }
pub unsafe fn GXInitLightPos(o: *mut c_void, x: f32, y: f32, z: f32) { GxShim::init_light_pos(light(o), x, y, z) }
pub unsafe fn GXInitLightDistAttn(o: *mut c_void, ref_dist: f32, ref_br: f32, func: i32) { melee_gx::shim::init_light_dist_attn(light(o), ref_dist, ref_br, func as u32) }
pub unsafe fn GXInitLightSpot(o: *mut c_void, cutoff: f32, func: i32) { melee_gx::shim::init_light_spot(light(o), cutoff, func as u32) }
pub unsafe fn GXLoadLightObjImm(o: *mut c_void, id: u32) {
    let l = light(o).clone();
    shim().load_light_obj_imm(&l, id)
}

// ---- texgen and TEV
pub fn GXSetNumTexGens(n: u8) { shim().set_num_tex_gens(n) }
pub fn GXSetTexCoordGen2(dst: u32, func: u32, src: u32, mtx: u32, normalize: u8, pt: u32) { shim().set_tex_coord_gen2(dst, func, src, mtx, normalize != 0, pt) }
pub fn GXSetNumTevStages(n: u8) { shim().set_num_tev_stages(n) }
pub fn GXSetTevOrder(stage: u32, coord: u32, map: u32, color: u32) { shim().set_tev_order(stage, coord, map, color) }
pub fn GXSetTevOp(stage: u32, mode: u32) { shim().set_tev_op(stage, mode) }
pub fn GXSetTevColorIn(s: u32, a: u32, b: u32, c: u32, d: u32) { shim().set_tev_color_in(s, a, b, c, d) }
pub fn GXSetTevAlphaIn(s: u32, a: u32, b: u32, c: u32, d: u32) { shim().set_tev_alpha_in(s, a, b, c, d) }
pub fn GXSetTevColorOp(s: u32, op: u32, bias: u32, scale: u32, clamp: u8, out: u32) { shim().set_tev_color_op(s, op, bias, scale, clamp != 0, out) }
pub fn GXSetTevAlphaOp(s: u32, op: u32, bias: u32, scale: u32, clamp: u8, out: u32) { shim().set_tev_alpha_op(s, op, bias, scale, clamp != 0, out) }
pub fn GXSetTevColor(id: u32, c: _GXColor) { shim().set_tev_color(id, col(c)) }
pub fn GXSetTevColorS10(id: u32, c: _GXColorS10) { shim().set_tev_color_s10(id, GxColorS10 { r: c.r, g: c.g, b: c.b, a: c.a }) }
pub fn GXSetTevKColor(id: u32, c: _GXColor) { shim().set_tev_kcolor(id, col(c)) }
pub fn GXSetTevKColorSel(id: u32, sel: u32) { shim().set_tev_kcolor_sel(id, sel) }
pub fn GXSetTevKAlphaSel(id: u32, sel: u32) { shim().set_tev_kalpha_sel(id, sel) }
pub fn GXSetTevSwapMode(stage: u32, ras: u32, tex: u32) { shim().set_tev_swap_mode(stage, ras, tex) }
pub fn GXSetTevSwapModeTable(id: u32, r: u32, g: u32, b: u32, a: u32) { shim().set_tev_swap_mode_table(id, r, g, b, a) }
pub fn GXSetTevClampMode(stage: i32, mode: i32) { shim().set_tev_clamp_mode(stage as u32, mode as u32) }
pub fn GXSetTevDirect(stage: u32) { shim().set_tev_direct(stage) }
pub fn GXSetNumIndStages(n: u8) { shim().set_num_ind_stages(n) }
pub fn GXSetIndTexOrder(stage: u32, coord: u32, map: u32) { shim().set_ind_tex_order(stage, coord, map) }
pub fn GXSetIndTexCoordScale(stage: u32, s: u32, t: u32) { shim().set_ind_tex_coord_scale(stage, s, t) }
pub unsafe fn GXSetIndTexMtx(id: u32, m: *mut c_void, scale_exp: i8) { shim().set_ind_tex_mtx(id, &*(m as *const [[f32; 3]; 2]), scale_exp) }
pub fn GXSetTevIndirect(stage: u32, ind_stage: u32, fmt: u32, bias: u32, mtx: u32, wrap_s: u32, wrap_t: u32, add_prev: u8, utc_lod: u8, alpha: u32) {
    shim().set_tev_indirect(stage, ind_stage, fmt, bias, mtx, wrap_s, wrap_t, add_prev != 0, utc_lod != 0, alpha)
}

// ---- textures
pub unsafe fn GXInitTexObj(o: *mut c_void, image: *mut c_void, w: u16, h: u16, fmt: u32, ws: u32, wt: u32, mip: u8) {
    let want = get_tex_buffer_size(w, h, fmt, mip != 0, if mip != 0 { 11 } else { 0 }) as usize;
    let d = data_at(image, want.max(32));
    let t = objs().tex.entry(o as usize).or_default();
    shim().init_tex_obj(t, d, w, h, fmt, ws, wt, mip != 0);
}
pub unsafe fn GXInitTexObjCI(o: *mut c_void, image: *mut c_void, w: u16, h: u16, fmt: u32, ws: u32, wt: u32, mip: u8, tlut: u32) {
    let want = get_tex_buffer_size(w, h, fmt, mip != 0, if mip != 0 { 11 } else { 0 }) as usize;
    let d = data_at(image, want.max(32));
    let t = objs().tex.entry(o as usize).or_default();
    shim().init_tex_obj_ci(t, d, w, h, fmt, ws, wt, mip != 0, tlut);
}
pub unsafe fn GXInitTexObjLOD(o: *mut c_void, min_f: u32, mag_f: u32, min_lod: f32, max_lod: f32, bias: f32, bias_clamp: u8, edge: u8, aniso: u32) {
    let t = objs().tex.entry(o as usize).or_default();
    GxShim::init_tex_obj_lod(t, min_f, mag_f, min_lod, max_lod, bias, bias_clamp != 0, edge != 0, aniso);
}
pub unsafe fn GXInitTlutObj(o: *mut c_void, lut: *mut c_void, fmt: u32, n: u16) {
    let d = data_at(lut, n as usize * 2);
    let t = objs().tlut.entry(o as usize).or_default();
    shim().init_tlut_obj(t, d, fmt, n);
}
pub unsafe fn GXLoadTexObj(o: *mut c_void, id: u32) {
    let t = objs().tex.entry(o as usize).or_default();
    shim().load_tex_obj(t, id)
}
pub unsafe fn GXLoadTlut(o: *mut c_void, name: u32) {
    if let Some(t) = objs().tlut.get(&(o as usize)) {
        shim().load_tlut(t, name)
    }
}
pub fn GXInvalidateTexAll() { shim().invalidate_tex_all() }
pub unsafe fn GXGetTexObjFmt(o: *const c_void) -> u32 { objs().tex.get(&(o as usize)).map_or(0, GxShim::get_tex_obj_fmt) }
pub unsafe fn GXGetTexObjWidth(o: *const c_void) -> u16 { objs().tex.get(&(o as usize)).map_or(0, GxShim::get_tex_obj_width) }
pub unsafe fn GXGetTexObjHeight(o: *const c_void) -> u16 { objs().tex.get(&(o as usize)).map_or(0, GxShim::get_tex_obj_height) }

// ---- copies and the frame
pub fn GXPixModeSync() { shim().pix_mode_sync() }
pub fn GXCopyDisp(_dest: *mut c_void, clear: u8) { shim().copy_disp(clear != 0) }
pub fn GXCopyTex(dest: *mut c_void, clear: u8) { shim().copy_tex(dest as usize as u32, clear != 0) }
pub fn GXSetCopyClear(c: _GXColor, depth: u32) { shim().set_copy_clear(col(c), depth) }
pub fn GXSetCopyClamp(c: u32) { shim().set_copy_clamp(c) }
pub fn GXSetDispCopySrc(l: u16, t: u16, w: u16, h: u16) { shim().set_disp_copy_src(l, t, w, h) }
pub fn GXSetDispCopyDst(w: u16, h: u16) { shim().set_disp_copy_dst(w, h) }
pub fn GXSetDispCopyGamma(g: u32) { shim().set_disp_copy_gamma(g) }
pub fn GXSetDispCopyYScale(v: f32) -> u32 { shim().set_disp_copy_y_scale(v) }
pub fn GXSetTexCopySrc(l: u16, t: u16, w: u16, h: u16) { shim().set_tex_copy_src(l, t, w, h) }
pub fn GXSetTexCopyDst(w: u16, h: u16, fmt: u32, mip: u8) { shim().set_tex_copy_dst(w, h, fmt, mip != 0) }

/// End the frame the way `HSD_VICopyXFBAsync` would (`GXCopyDisp`): the engine's VI state has no XFB buffers, so that
/// function returns before copying; the engine calls this after each scene tick instead.
pub fn finish_frame() {
    shim().copy_disp(true)
}
