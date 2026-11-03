//! Native port of src/melee/ft/ftafterimage.c (sword swing trails).
//!
//! `ftCo_800C2600` is split: [`afterimage_strip`] builds the triangle-strip vertices exactly as
//! the C fills `vtx_buf` and emits them (the C's order: v0, v2, v1..v[n-1]); the GX state and
//! the emit go through `ext`. Fused sites (`tools/rust-port/fma_sites.py`): 25 f32 fmuladd in
//! ftCo_800C2600, all `a*b + c` with the left product fused (-> `fmadds`), and the
//! `dz*dz + (dx*dx + dy*dy)` distance (`fmadds(dz, dz, fmadds(dx, dx, dy*dy))`). Float->int
//! casts saturate (gwtool). Checked bit for bit against the C by tests/tu_ftafterimage.rs.
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_runtime::gekko::sqrtf;
use melee_runtime::lbvector as lbv;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `AFTERIMAGE_ANGLE_STEP` (5 degrees).
pub const AFTERIMAGE_ANGLE_STEP: f32 = 0.0872664600610733;

/// `AfterimageVtx`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AfterimageVtx {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use super::G;
    use crate::types::*;
    pub fn it_80285300(_i: G) -> *mut itSword_UnkBytes { unsafe { melee_items::tu::itsword::it_80285300(_i as _) as _ } }
    pub fn it_802852B8(_i: G, _a: &mut f32, _b: &mut f32) { unsafe { melee_items::tu::itsword::it_802852B8(_i as _, _a, _b) } }
    pub fn it_80285314(_i: G) -> *mut HSD_JObj { unsafe { melee_items::itkinds14::it_80285314(_i as _) as _ } }
    pub fn Mex_FtTrailData(_g: G) -> *mut core::ffi::c_void { unsafe { crate::platform::Mex_FtTrailData(_g) } }
    pub fn Geno_DefineBaseKind(_k: i32) -> i32 { unsafe { crate::platform::Geno_DefineBaseKind(_k) } }
    pub fn lb_8000B1CC(_j: *mut HSD_JObj, _off: Option<&Vec3>, _out: &mut Vec3) {
        unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(_j, _off.map_or(core::ptr::null_mut(), |v| v as *const Vec3 as *mut Vec3), _out) }
    }
    pub fn HSD_CObjGetCurrent() -> *mut HSD_CObj { unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ } }
    /// `HSD_CObjGetViewingMtxPtrDirect` (cobj.h static inline): `cobj->view_mtx`.
    pub fn HSD_CObjGetViewingMtxPtrDirect(c: *mut HSD_CObj) -> *mut Mtx { unsafe { &mut (*c).view_mtx } }
    pub fn HSD_StateInvalidate(_s: i32) { unsafe { melee_runtime::tu::state_native::HSD_StateInvalidate(_s as _) } }
    // GX (render backend). Arguments are the GX enum values the C passes.
    pub fn GXSetColorUpdate(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetColorUpdate(_a as _) }
    pub fn GXSetAlphaUpdate(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetAlphaUpdate(_a as _) }
    pub fn GXSetBlendMode(_a: i32, _b: i32, _c: i32, _d: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetBlendMode(_a as _, _b as _, _c as _, _d as _) }
    pub fn GXSetAlphaCompare(_a: i32, _b: u8, _c: i32, _d: i32, _e: u8) { melee_runtime::tu::displayfunc_native::ext::GXSetAlphaCompare(_a as _, _b as _, _c as _, _d as _, _e as _) }
    pub fn GXSetZMode(_a: i32, _b: i32, _c: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetZMode(_a as _, _b as _, _c as _) }
    pub fn GXSetZCompLoc(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetZCompLoc(_a as _) }
    pub fn GXSetNumTexGens(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetNumTexGens(_a as _) }
    /// GC-only register; Aurora (and so melee_gx) ignores it.
    pub fn GXSetTevClampMode(_a: i32, _b: i32) {}
    pub fn GXSetNumTevStages(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetNumTevStages(_a as _) }
    pub fn GXSetTevOrder(_a: i32, _b: i32, _c: i32, _d: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetTevOrder(_a as _, _b as _, _c as _, _d as _) }
    pub fn GXSetTevOp(_a: i32, _b: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetTevOp(_a as _, _b as _) }
    pub fn GXSetNumChans(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetNumChans(_a as _) }
    pub fn GXSetChanCtrl(_a: i32, _b: i32, _c: i32, _d: i32, _e: i32, _f: i32, _g: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetChanCtrl(_a as _, _b as _, _c as _, _d as _, _e as _, _f as _, _g as _) }
    pub fn GXSetCullMode(_a: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetCullMode(_a as _) }
    pub fn GXClearVtxDesc() { melee_runtime::tu::displayfunc_native::ext::GXClearVtxDesc() }
    pub fn GXSetVtxAttrFmt(_a: i32, _b: i32, _c: i32, _d: i32, _e: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetVtxAttrFmt(_a as _, _b as _, _c as _, _d as _, _e as _) }
    pub fn GXSetVtxDesc(_a: i32, _b: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetVtxDesc(_a as _, _b as _) }
    pub fn GXLoadPosMtxImm(_m: *mut Mtx, _id: i32) { unsafe { melee_runtime::tu::pobj::native::mx::GXLoadPosMtxImm(_m as _, _id as _) } }
    pub fn GXSetCurrentMtx(_id: i32) { melee_runtime::tu::displayfunc_native::ext::GXSetCurrentMtx(_id as _) }
    pub fn GXBegin(_prim: i32, _fmt: i32, _n: i32) { melee_runtime::tu::displayfunc_native::ext::GXBegin(_prim as _, _fmt as _, _n as _) }
    pub fn GXPosition3f32(_x: f32, _y: f32, _z: f32) { melee_runtime::tu::displayfunc_native::ext::GXPosition3f32(_x as _, _y as _, _z as _) }
    pub fn GXColor4u8(_r: u8, _g: u8, _b: u8, _a: u8) { melee_runtime::tu::displayfunc_native::ext::GXColor4u8(_r as _, _g as _, _b as _, _a as _) }
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC Geno base kind).
fn template_kind(f: &Fighter) -> FighterKind {
    if f.kind >= Ft_Kind_Mex0 && h::geno_define_template_skeleton(f.kind as i32) != 0 {
        return ext::Geno_DefineBaseKind(f.kind as i32) as FighterKind;
    }
    f.kind
}

/// The fighter's own sword attrs (Link / Young Link `x64`, Marth / Roy `x78`); null otherwise
/// (the C reads an uninitialised, zero-initialised on PC, pointer there).
fn own_sword(f: &Fighter) -> *mut SwordAttrs {
    let k = template_kind(f);
    if k == Ft_Kind_Link || k == Ft_Kind_CLink {
        unsafe { &mut (*(f.dat_attrs as *mut ftLk_DatAttrs)).x64 }
    } else if k == Ft_Kind_Mars || k == Ft_Kind_Emblem {
        unsafe { &mut (*(f.dat_attrs as *mut MarsAttributes)).x78 }
    } else {
        // Seak, Ness, Peach, ... (and any other kind): no sword.
        core::ptr::null_mut()
    }
}

/// `ftCo_800C2600_get_params` (static inline).
fn get_params(f: &Fighter) -> *mut itSword_UnkBytes {
    if f.x2101_bits_8() != 0 {
        if h::it_get_kind(f.item_gobj) == It_Kind_Sword {
            ext::it_80285300(f.item_gobj)
        } else {
            panic!("ftafterimage.c:124: no afterimage item!");
        }
    } else {
        own_sword(f) as *mut itSword_UnkBytes
    }
}

#[inline]
fn v3(v: &Vec3) -> melee_core::vec::Vec3 {
    melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z }
}

#[inline]
fn ring_prev(i: i32) -> i32 {
    if i != 0 { i - 1 } else { 2 }
}

/// The vertex work of `ftCo_800C2600`: `fp->x2100` (> 1) ring entries newest first, subdivided
/// every 5 degrees. Returns the vertices in the order the C hands them to GX (`numVerts + 1`).
pub fn afterimage_strip(f: &Fighter, params: &itSword_UnkBytes) -> std::vec::Vec<AfterimageVtx> {
    melee_core::ct!("afterimage_strip");
    let mut vtx_buf = [AfterimageVtx::default(); 152];
    let mut cum_dist = [0.0f32; 3];
    let x20f8 = f.x20F8;
    let x20fc = f.x20FC;
    let n = f.x2100 as i32;
    let mut next_idx: i32 = 0;

    let ring_idx = ring_prev(f.x2101_bits_0to6() as i32);
    let mut total = 0.0f32;
    let mut dp = 1usize;
    {
        let mut cur = ring_idx;
        let mut prev = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut i = n - 1;
        while i >= 0 {
            let e = &f.x20B0[cur as usize];
            let delta = Vec3 {
                x: fmadds(e.xC.x, x20fc, e.x0.x) - prev.x,
                y: fmadds(e.xC.y, x20fc, e.x0.y) - prev.y,
                z: fmadds(e.xC.z, x20fc, e.x0.z) - prev.z,
            };
            if i != n - 1 {
                let d2 = sqrtf(fmadds(delta.z, delta.z, fmadds(delta.x, delta.x, delta.y * delta.y)));
                total += d2;
                cum_dist[dp] = total;
                dp += 1;
            }
            prev = delta;
            if i != 0 {
                next_idx = ring_prev(cur);
            }
            cur = next_idx;
            i -= 1;
        }
    }
    if total < 0.00001 {
        total = 1.0;
    }

    let scale_diff = x20fc - x20f8;
    let blended_inner = fmadds(params.x0, scale_diff, x20f8);
    let blended_outer = fmadds(params.x4, scale_diff, x20f8);
    let mut num_verts = 0i32;
    let mut cur2 = ring_prev(f.x2101_bits_0to6() as i32);
    let inner_diff = x20f8 - blended_inner;
    let mut interp = 1.0f32;
    let outer_diff = x20fc - blended_outer;
    let mut dist_idx = 0usize;
    let a89 = (params.x8 as i32 - params.x9 as i32) as f32;
    let a9 = params.x9 as f32;
    let mut vp = 0usize;
    let mut remaining = n - 1;
    let put = |buf: &mut [AfterimageVtx; 152], at: usize, d: &Vec3, s: f32, b: (f32, f32, f32), c: (u8, u8, u8), a: i32| {
        buf[at] = AfterimageVtx {
            x: fmadds(d.x, s, b.0),
            y: fmadds(d.y, s, b.1),
            z: fmadds(d.z, s, b.2),
            r: c.0,
            g: c.1,
            b: c.2,
            a: a as u8,
        };
    };
    let cin = (params.xA, params.xB, params.xC);
    let cout = (params.xE, params.xF, params.x10);
    while remaining >= 0 {
        let e = f.x20B0[cur2 as usize];
        let mut outer_scale = fmadds(interp, outer_diff, blended_outer);
        let mut inner_scale = fmadds(interp, inner_diff, blended_inner);
        num_verts += 2;
        dist_idx += 1;
        let mut alpha = f2i32(fmadds(interp, a89, a9) as f64);
        let base = (e.x0.x, e.x0.y, e.x0.z);
        put(&mut vtx_buf, vp, &e.xC, inner_scale, base, cin, alpha);
        put(&mut vtx_buf, vp + 1, &e.xC, outer_scale, base, cout, alpha);
        vp += 2;

        if remaining != 0 {
            let nri = ring_prev(cur2);
            let ne = f.x20B0[nri as usize];
            next_idx = nri;
            // lbVector_CrossprodNormalized returns its `result` argument: never NULL.
            let cross = lbv::crossprod_normalized(&v3(&e.xC), &v3(&ne.xC));
            let mut angle = lbv::angle(&v3(&e.xC), &v3(&ne.xC));
            let sub_angle = angle / AFTERIMAGE_ANGLE_STEP;
            let num_subdiv = f2i32(sub_angle as f64);
            interp = 1.0 - cum_dist[dist_idx] / total;
            if num_subdiv != 0 {
                let frac = 1.0f32 / (num_subdiv + 1) as f32;
                let mut cum_angle = 0.0f32;
                angle *= frac;
                let interp_inner2 = frac * (fmadds(interp, inner_diff, blended_inner) - inner_scale);
                let (mut bx, mut by, mut bz) = base;
                let sx = frac * (ne.x0.x - bx);
                let sy = frac * (ne.x0.y - by);
                let sz = frac * (ne.x0.z - bz);
                let alpha_step = f2i32((frac * (fmadds(interp, a89, a9) - alpha as f32)) as f64);
                let interp_outer2 = frac * (fmadds(interp, outer_diff, blended_outer) - outer_scale);
                for _ in 0..num_subdiv {
                    cum_angle += angle;
                    bx += sx;
                    by += sy;
                    bz += sz;
                    inner_scale += interp_inner2;
                    outer_scale += interp_outer2;
                    alpha = alpha.wrapping_add(alpha_step);
                    let mut t = v3(&e.xC);
                    lbv::rotate_about_unit_axis(&mut t, &cross, cum_angle);
                    let td = Vec3 { x: t.x, y: t.y, z: t.z };
                    num_verts += 2;
                    put(&mut vtx_buf, vp, &td, inner_scale, (bx, by, bz), cin, alpha);
                    put(&mut vtx_buf, vp + 1, &td, outer_scale, (bx, by, bz), cout, alpha);
                    vp += 2;
                }
            }
        }
        cur2 = next_idx;
        remaining -= 1;
    }

    let mut out = std::vec::Vec::with_capacity(num_verts as usize + 1);
    out.push(vtx_buf[0]);
    out.push(vtx_buf[2]);
    for i in 1..num_verts as usize {
        out.push(vtx_buf[i]);
    }
    out
}

/// `ftCo_800C2600`: the trail draw callback (render pass 2 only).
pub fn ftCo_800C2600(g: G, arg1: u32) {
    melee_core::ct!("ftCo_800C2600");
    use ext::*;
    if arg1 != 2 {
        return;
    }
    let f = fp(g);
    if f.x2100 <= 1 {
        return;
    }
    GXSetColorUpdate(1);
    GXSetAlphaUpdate(0);
    GXSetBlendMode(GX_BM_BLEND as i32, GX_BL_SRCALPHA as i32, GX_BL_INVSRCALPHA as i32, GX_LO_NOOP as i32);
    GXSetAlphaCompare(GX_GREATER as i32, 0, GX_AOP_AND as i32, GX_GREATER as i32, 0);
    GXSetZMode(1, GX_LEQUAL as i32, 0);
    GXSetZCompLoc(0);
    GXSetNumTexGens(0);
    GXSetTevClampMode(0, 0);
    GXSetNumTevStages(1);
    GXSetTevOrder(GX_TEVSTAGE0 as i32, GX_TEXCOORD_NULL as i32, GX_TEXMAP_NULL as i32, GX_COLOR0A0 as i32);
    GXSetTevOp(GX_TEVSTAGE0 as i32, GX_PASSCLR as i32);
    GXSetNumChans(1);
    GXSetChanCtrl(GX_COLOR0A0 as i32, 0, GX_SRC_REG as i32, GX_SRC_VTX as i32, 0, GX_DF_NONE as i32, GX_AF_NONE as i32);
    GXSetCullMode(GX_CULL_NONE as i32);

    let mut params = get_params(f);
    if f.x2101_bits_8() == 0 && f.kind >= Ft_Kind_Mex0 {
        params = Mex_FtTrailData(g) as *mut itSword_UnkBytes;
        if params.is_null() {
            return;
        }
    }
    assert!(!params.is_null(), "ftCo_800C2600: no trail params (the C reads through NULL)");
    let verts = afterimage_strip(f, unsafe { &*params });

    GXClearVtxDesc();
    GXSetVtxAttrFmt(GX_VTXFMT0 as i32, GX_VA_POS as i32, GX_TEX_ST as i32, GX_RGBA6 as i32, 0);
    GXSetVtxAttrFmt(GX_VTXFMT0 as i32, GX_VA_CLR0 as i32, GX_TEX_ST as i32, GX_RGBA8 as i32, 0);
    GXSetVtxDesc(GX_VA_POS as i32, GX_DIRECT as i32);
    GXSetVtxDesc(GX_VA_CLR0 as i32, GX_DIRECT as i32);
    GXLoadPosMtxImm(HSD_CObjGetViewingMtxPtrDirect(HSD_CObjGetCurrent()), 0);
    GXSetCurrentMtx(0);
    GXBegin(GX_TRIANGLESTRIP as i32, GX_VTXFMT0 as i32, verts.len() as i32);
    for v in &verts {
        GXPosition3f32(v.x, v.y, v.z);
        GXColor4u8(v.r, v.g, v.b, v.a);
    }
    HSD_StateInvalidate(-1);
}

/// `ftCo_800C2FD8`: push the sword's current base point and blade axis onto the 3-entry ring.
pub fn ftCo_800C2FD8(g: G) {
    melee_core::ct!("ftCo_800C2FD8");
    let f = fp(g);
    if f.x2100 == -1 {
        return;
    }
    let slot = f.x2101_bits_0to6() as usize;
    let axis;
    let jobj;
    if f.x2101_bits_8() != 0 {
        if f.item_gobj.is_null() {
            f.x2100 = -1;
            return;
        }
        if h::it_get_kind(f.item_gobj) == It_Kind_Sword {
            let (mut a, mut b) = (f.x20F8, f.x20FC);
            ext::it_802852B8(f.item_gobj, &mut a, &mut b);
            f.x20F8 = a;
            f.x20FC = b;
            axis = 1;
        } else {
            panic!("ftafterimage.c:365: no afterimage item!");
        }
        jobj = ext::it_80285314(f.item_gobj);
    } else {
        let attrs = if f.kind >= Ft_Kind_Mex0 {
            let a = ext::Mex_FtTrailData(g) as *mut SwordAttrs;
            if a.is_null() {
                return;
            }
            a
        } else {
            own_sword(f)
        };
        assert!(!attrs.is_null(), "ftCo_800C2FD8: fighter has no sword (C @bug: reads NULL)");
        let a = unsafe { &*attrs };
        axis = 0;
        f.x20F8 = a.x18;
        f.x20FC = a.x1C;
        jobj = unsafe { (*f.parts.add(a.x14 as usize)).joint };
    }
    let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, None, &mut p);
    let f = fp(g);
    let m = unsafe { &(*jobj).mtx };
    let e = &mut f.x20B0[slot];
    e.x0 = p;
    e.xC.x = m[0][axis];
    e.xC.y = m[1][axis];
    e.xC.z = m[2][axis];
    let next = if f.x2101_bits_0to6() == 2 { 0 } else { f.x2101_bits_0to6() + 1 };
    f.set_x2101_bits_0to6(next);
    if f.x2100 < 3 {
        f.x2100 += 1;
    }
}
