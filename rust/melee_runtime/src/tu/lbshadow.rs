//! Complete PC `src/melee/lb/lbshadow.c` (11 definitions, including three static tangents).
//! Native records come from melee_types; external systems use named `ext` hooks.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::{fmadds, fmsubs};
use melee_types::*;
use super::lbext::__assert;

/// Native bindings for game globals and callees whose raw-record entrypoints are not linked. Hook
/// signatures are the C prototypes (what gen/mtu/gen_tu.py generates for the state oracle); an unset
/// hook runs the native port where one exists, else panics.
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    pub use crate::tu::lbext::OSReport;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GXBegin(a0: u32, a1: u32, a2: u16) = crate::tu::lbgx::ext::GXBegin(a0 as i32, a1 as i32, a2);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32) = crate::tu::lbgx::ext::GXLoadPosMtxImm(a0 as *const [f32; 4], a1);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32) = crate::tu::lbgx::ext::GXPosition3f32(a0, a1, a2);
    fn GXSetCurrentMtx(a0: u32) = crate::tu::lbgx::ext::GXSetCurrentMtx(a0);
    fn GXSetScissor(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8) = crate::tu::lbgx::ext::GXSetVtxAttrFmt(a0 as i32, a1 as i32, a2 as i32, a3 as i32, a4);
    fn GXSetVtxDesc(a0: u32, a1: u32) = crate::tu::lbgx::ext::GXSetVtxDesc(a0 as i32, a1 as i32);
    fn Ground_801C0508() -> i32;
    fn HSD_CObjGetBottom(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetLeft(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetNear(a0: *mut HSD_CObj) -> f32 = unsafe { crate::tu::cobj::HSD_CObjGetNear(a0) };
    fn HSD_CObjGetRight(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetTop(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) = unsafe { crate::tu::cobj::HSD_CObjSetFar(a0, a1) };
    fn HSD_CObjSetFlags(a0: *mut HSD_CObj, a1: u32) = unsafe { crate::tu::cobj::HSD_CObjSetFlags(a0, a1) };
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) = unsafe { crate::tu::cobj::HSD_CObjSetNear(a0, a1) };
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) = unsafe { crate::tu::cobj::HSD_CObjSetOrtho(a0, a1, a2, a3, a4) };
    fn HSD_CObjSetProjectionType(a0: *mut HSD_CObj, a1: u32);
    fn HSD_CObjSetUpVector(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_ClearVtxDesc() = unsafe { crate::tu::pobj::HSD_ClearVtxDesc() };
    fn HSD_Init_803755A8() = unsafe { crate::tu::initialize::HSD_Init_803755A8() };
    fn HSD_LObjGetFlags(a0: *mut HSD_LObj) -> u32;
    fn HSD_LObjGetInterest(a0: *mut HSD_LObj, a1: *mut Vec) -> i32;
    fn HSD_LObjGetPosition(a0: *mut HSD_LObj, a1: *mut Vec) -> i32;
    fn HSD_SetupChannelAll(a0: *mut HSD_Chan);
    fn HSD_ShadowAddObject(a0: *mut HSD_Shadow, a1: *mut HSD_JObj) = unsafe { crate::tu::shadow::HSD_ShadowAddObject(a0, a1) };
    fn HSD_ShadowAlloc() -> *mut HSD_Shadow = unsafe { crate::tu::shadow::HSD_ShadowAlloc() };
    fn HSD_ShadowDeleteObject(a0: *mut HSD_Shadow, a1: *mut HSD_JObj) = unsafe { crate::tu::shadow::HSD_ShadowDeleteObject(a0, a1) };
    fn HSD_ShadowEndRender(a0: *mut HSD_Shadow) = unsafe { crate::tu::shadow::HSD_ShadowEndRender(a0) };
    fn HSD_ShadowInit(a0: *mut HSD_Shadow) = unsafe { crate::tu::shadow::HSD_ShadowInit(a0) };
    fn HSD_ShadowRemove(a0: *mut HSD_Shadow) = unsafe { crate::tu::shadow::HSD_ShadowRemove(a0) };
    fn HSD_ShadowSetActive(a0: *mut HSD_Shadow, a1: i32) = unsafe { crate::tu::shadow::HSD_ShadowSetActive(a0, a1) };
    fn HSD_ShadowSetSize(a0: *mut HSD_Shadow, a1: u16, a2: u16) = unsafe { crate::tu::shadow::HSD_ShadowSetSize(a0, a1, a2) };
    fn HSD_ShadowSetViewingRect(a0: *mut HSD_Shadow, a1: f32, a2: f32, a3: f32, a4: f32);
    fn HSD_ShadowStartRender(a0: *mut HSD_Shadow) = unsafe { crate::tu::shadow::HSD_ShadowStartRender(a0) };
    fn HSD_StartRender(a0: u32);
    fn HSD_StateSetCullMode(a0: i32);
    fn HSD_ViewingRectAddRect(a0: *mut HSD_ViewingRect, a1: *mut Vec, a2: f32, a3: f32, a4: f32, a5: f32) = unsafe { crate::tu::shadow::HSD_ViewingRectAddRect(a0, a1, a2, a3, a4, a5) };
    fn HSD_ViewingRectCheck(a0: *mut HSD_ViewingRect) -> i32 = unsafe { crate::tu::shadow::HSD_ViewingRectCheck(a0) };
    fn HSD_ViewingRectInit(a0: *mut HSD_ViewingRect, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: i32) = unsafe { crate::tu::shadow::HSD_ViewingRectInit(a0, a1, a2, a3, a4) };
    fn PSVECAdd(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = unsafe { crate::tu::mtx_pc::raw::PSVECAdd(a0, a1, a2) };
    fn lbVector_CrossprodNormalized(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = unsafe { crate::tu::lbvector::lbVector_CrossprodNormalized(a0, a1, a2) };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = unsafe { crate::tu::lbvector::lbVector_Diff(a0, a1, a2) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 = unsafe { crate::tu::lbvector::lbVector_Normalize(a0) };
    fn lbVector_Sub(a0: *mut Vec, a1: *mut Vec) -> *mut Vec = unsafe { crate::tu::lbvector::lbVector_Sub(a0, a1) };
    fn ftDrawCommon_80081168();
    fn ftDrawCommon_80081200();
    fn ftLib_IsFighter(a0: *mut HSD_GObj) -> i32;
    fn ftLib_GetShadow(a0: *mut HSD_GObj) -> *mut LbShadow;
    fn ftLib_CastsShadow(a0: *mut HSD_GObj) -> i32;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObjGXLinkHead;
    pub use crate::tu::gobj_native::HSD_GObjPLinkHead;
    pub use crate::tu::gobj_native::HSD_GObj_804D7814;
    pub use crate::tu::perf::HSD_PerfCurrentStat;
    pub static mut HSD_identityMtx: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
}

const FILE: &str = "src/melee/lb/lbshadow.c";

/// lbshadow.c `splGetCardinalTangent` (14 fused sites).
pub fn splGetCardinalTangent(p: &mut Vec3, cp: &[Vec3], tension: f32, u: f32) {
    melee_core::ct!("splGetCardinalTangent");
    let u2 = u * u;
    let car0 = tension * (fmadds(-3.0, u2, 4.0 * u) - 1.0);
    let car1 = fmadds(3.0 * (2.0 - tension), u2, 2.0 * (tension - 3.0) * u);
    let car2 = tension + fmadds(3.0 * (tension - 2.0), u2, 2.0 * -fmadds(2.0, tension, -3.0) * u);
    let car3 = tension * fmadds(3.0, u2, -(2.0 * u));
    let c = [car0, car1, car2, car3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBSplineTangent` (11 fused sites).
pub fn splGetBSplineTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    melee_core::ct!("splGetBSplineTangent");
    let u2 = u * u;
    let u_1 = 1.0 - u;
    let half = 0.5f32;
    let b0 = u_1 * (-half * u_1);
    let b1 = half * fmadds(3.0, u2, -(4.0 * u));
    let b2 = half * (1.0 + fmadds(-3.0, u2, 2.0 * u));
    let b3 = half * u2;
    let c = [b0, b1, b2, b3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBezierTangent` (12 fused sites).
pub fn splGetBezierTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    melee_core::ct!("splGetBezierTangent");
    let u_1 = u - 1.0;
    let u2 = u * u;
    let bez0 = -3.0 * u_1 * u_1;
    let bez1 = 3.0 * fmadds(3.0, u2, fmadds(-4.0, u, 1.0));
    let bez2 = 3.0 * fmadds(2.0, u, -(3.0 * u2));
    let bez3 = 3.0 * u2;
    let c = [bez0, bez1, bez2, bez3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `lbShadow_8000E9F0`: the tangent of `spline` at `u` in [0, 1] (no-op outside),
/// by type: 0 linear, 1 Bezier, 2 B-spline, 3 cardinal; any other type leaves `p` alone.
#[cfg_attr(fptrace, melee_fptrace::trace)]
pub fn lbShadow_8000E9F0(p: &mut Vec3, spline: &HSD_Spline, mut u: f32) {
    melee_core::ct!("lbShadow_8000E9F0");
    if u < 0.0 || u > 1.0 {
        return;
    }
    let orig_u = u;
    u *= (spline.numcv as i32 - 1) as f32;
    let mut idx = (melee_core::ppc::fctiwz_f32(u)) as i16; // gwtool saturates to i32, then truncates.
    u -= idx as f32;
    // SAFETY: cv holds numcv control points (3*numcv-2 for Bezier); the C indexes it unchecked.
    let cps = |at: isize| unsafe { core::slice::from_raw_parts(spline.cv.offset(at), 4) };
    match spline.r#type {
        0 => {
            if orig_u == 1.0 {
                idx = idx.wrapping_sub(1);
            }
            let cp = unsafe { core::slice::from_raw_parts(spline.cv.offset(idx as isize), 2) };
            p.x = cp[1].x - cp[0].x;
            p.y = cp[1].y - cp[0].y;
            p.z = cp[1].z - cp[0].z;
        }
        1 => splGetBezierTangent(p, cps(idx as isize * 3), u),
        2 => splGetBSplineTangent(p, cps(idx as isize), u),
        3 => splGetCardinalTangent(p, cps(idx as isize), spline.tension, u),
        _ => {}
    }
}

/// lbshadow.c `lbShadow_8000ED54`: allocates the fighter shadow (256x256 ortho camera, intensity
/// 0xC0, texture scale +-0.45), adds `jobj` and activates it, and clears the suppression flags.
///
/// # Safety
/// `lbshadow` must be valid for writes; `jobj` is handed to the shadow system.
pub unsafe fn lbShadow_8000ED54(lbshadow: *mut LbShadow, jobj: *mut HSD_JObj) {
    melee_core::ct!("lbShadow_8000ED54");
    if lbshadow.is_null() {
        __assert(FILE, 0x36, "lbshadow");
    }
    let shadow = ext::HSD_ShadowAlloc();
    if !shadow.is_null() {
        ext::HSD_CObjSetProjectionType((*shadow).camera, PROJ_ORTHO as u32);
        ext::HSD_CObjSetNear((*shadow).camera, 0.001);
        ext::HSD_CObjSetFar((*shadow).camera, 5000.0);
        ext::HSD_CObjSetFlags((*shadow).camera, 1);
        ext::HSD_ShadowSetSize(shadow, 0x100, 0x100);
        (*shadow).intensity = 0xC0;
        (*shadow).scaleS = 0.45;
        (*shadow).scaleT = -0.45;
        ext::HSD_ShadowAddObject(shadow, jobj);
        ext::HSD_ShadowSetActive(shadow, 1);
    }
    (*lbshadow).set_x0_b0(0);
    (*lbshadow).set_x0_b1(0);
    (*lbshadow).set_x0_b3(0);
    (*lbshadow).set_x0_b4(0);
    (*lbshadow).set_x0_b5(0);
    (*lbshadow).shadow = shadow;
}

/// lbshadow.c `lbShadow_8000EE8C`: removes the shadow, if any.
///
/// # Safety
/// `lbshadow` must be valid for reads.
pub unsafe fn lbShadow_8000EE8C(lbshadow: *mut LbShadow) {
    melee_core::ct!("lbShadow_8000EE8C");
    if lbshadow.is_null() {
        __assert(FILE, 0x62, "lbshadow");
    }
    if !(*lbshadow).shadow.is_null() {
        ext::HSD_ShadowRemove((*lbshadow).shadow);
    }
}

/// True when any of the six suppression bits of `lbs` is set.
unsafe fn any_flag(lbs: *mut LbShadow) -> bool {
    (*lbs).x0_b0() != 0 || (*lbs).x0_b1() != 0 || (*lbs).x0_b2() != 0 || (*lbs).x0_b3() != 0 || (*lbs).x0_b4() != 0 || (*lbs).x0_b5() != 0
}

/// lbshadow.c `lbShadow_8000EEE0`: activates the fighter shadow when no suppression bit is set
/// and `ftLib_CastsShadow` allows it, deactivates it otherwise.
///
/// # Safety
/// `gobj` must be a valid fighter GObj.
pub unsafe fn lbShadow_8000EEE0(gobj: *mut HSD_GObj) {
    melee_core::ct!("lbShadow_8000EEE0");
    if (ext::ftLib_IsFighter(gobj) != 0) {
        let lbs = ext::ftLib_GetShadow(gobj);
        if !lbs.is_null() {
            if !any_flag(lbs) && (ext::ftLib_CastsShadow(gobj) != 0) {
                ext::HSD_ShadowSetActive((*lbs).shadow, 1);
            } else {
                ext::HSD_ShadowSetActive((*lbs).shadow, 0);
            }
        }
    }
}

/// lbshadow.c `lbShadow_8000EFEC`: clears every fighter's "too many shadows" bit, then sets it on
/// the 6th and later fighters that would otherwise cast one, refreshing each shadow's activity.
///
/// # Safety
/// The fighter player link (`ext::HSD_GObjPLinkHead`) must be valid.
pub unsafe fn lbShadow_8000EFEC() {
    melee_core::ct!("lbShadow_8000EFEC");
    let mut count = 0i32;
    let head = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    let mut g = head;
    while !g.is_null() {
        let lbs = ext::ftLib_GetShadow(g);
        if !lbs.is_null() {
            (*lbs).set_x0_b2(0);
        }
        g = (*g).next;
    }
    let mut cur = head;
    while !cur.is_null() {
        let lbs = ext::ftLib_GetShadow(cur);
        if !lbs.is_null() {
            if !any_flag(lbs) {
                count += 1;
                if count > 5 {
                    (*lbs).set_x0_b2(1);
                }
            }
            lbShadow_8000EEE0(cur);
        }
        cur = (*cur).next;
    }
}

/// lbshadow.c `lbShadow_803BA0FC`: the channel `lbShadow_8000F214` sets up (black ambient, opaque
/// black material, register sources, no light).
fn shadow_chan() -> HSD_Chan {
    let mut c = HSD_Chan::zeroed();
    c.chan = GX_COLOR0A0;
    c.mat_color.a = 0xFF;
    c.amb_src = GX_SRC_REG;
    c.mat_src = GX_SRC_REG;
    c.light_mask = GX_LIGHT_NULL;
    c.diff_fn = GX_DF_CLAMP;
    c.attn_fn = GX_AF_NONE;
    c
}

/// lbshadow.c `lbShadow_8000F214`: clears the shadow texture with one black quad 1.2x the camera
/// ortho rectangle at 1.1x its near plane.
///
/// # Safety
/// `shadow` (with camera and texture) must be valid.
pub unsafe fn lbShadow_8000F214(shadow: *mut HSD_Shadow) {
    melee_core::ct!("lbShadow_8000F214");
    let cobj = (*shadow).camera;
    let mut chan = shadow_chan();
    ext::HSD_SetupChannelAll(&mut chan);
    let imagedesc = (*(*shadow).texture).imagedesc;
    ext::GXSetScissor(0, 0, (*imagedesc).width as u32, (*imagedesc).height as u32);
    ext::GXLoadPosMtxImm(addr_of_mut!(ext::HSD_identityMtx) as *mut core::ffi::c_void, GX_PNMTX0 as u32);
    ext::HSD_PerfCurrentStat.nb_mtx_load = ext::HSD_PerfCurrentStat.nb_mtx_load.wrapping_add(1);
    ext::GXSetCurrentMtx(GX_PNMTX0 as u32);
    ext::HSD_ClearVtxDesc();
    ext::GXSetVtxDesc(GX_VA_POS as u32, GX_DIRECT as u32);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0 as u32, GX_VA_POS as u32, GX_TEX_ST as u32, GX_RGBA6 as u32, 0);
    ext::HSD_StateSetCullMode(GX_CULL_BACK as i32);

    let mut y0 = ext::HSD_CObjGetTop(cobj);
    let mut y1 = ext::HSD_CObjGetBottom(cobj);
    let mut x0 = ext::HSD_CObjGetLeft(cobj);
    let mut x1 = ext::HSD_CObjGetRight(cobj);
    let mut z = ext::HSD_CObjGetNear(cobj);
    y0 *= 1.2;
    y1 *= 1.2;
    x0 *= 1.2;
    x1 *= 1.2;
    z *= -1.1;

    ext::GXBegin(GX_QUADS as u32, GX_VTXFMT0 as u32, 4);
    ext::GXPosition3f32(x0, y0, z);
    ext::GXPosition3f32(x1, y0, z);
    ext::GXPosition3f32(x1, y1, z);
    ext::GXPosition3f32(x0, y1, z);
}

/// lbshadow.c `lbShadow_8000F38C(arg0)`: finds the stage key light, derives the shadow camera
/// (position, interest, up vector) from it, and re-renders every fighter shadow texture
/// (clearing it first when `arg0 != 0`).
///
/// # Safety
/// The fighter player link, the light GX link and every fighter must be valid.
pub unsafe fn lbShadow_8000F38C(arg0: i32) {
    melee_core::ct!("lbShadow_8000F38C");
    let mut rect = HSD_ViewingRect::zeroed();
    let mut light_pos = Vec3::zeroed();
    let mut light_dir = Vec3::zeroed();
    let mut up_vec = Vec3::zeroed();
    let mut eye_pos = Vec3::zeroed();
    let mut interest_pos = Vec3::zeroed();
    let mut no_light = false;
    let plink = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);

    let mut g = plink;
    while !g.is_null() {
        let lbs = ext::ftLib_GetShadow(g);
        if !lbs.is_null() {
            (*lbs).set_x0_b2(0);
        }
        g = (*g).next;
    }

    let mut lobj: *mut HSD_LObj = null_mut();
    let mut fallback: *mut HSD_LObj = null_mut();
    let mut lgobj = *ext::HSD_GObjGXLinkHead.add(4);
    while !lgobj.is_null() {
        let mut next_gx = (*lgobj).next_gx;
        lobj = (*lgobj).hsd_obj as *mut HSD_LObj;
        while !lobj.is_null() {
            if (*lobj).flags & 3 != 0 {
                fallback = lobj;
            }
            if ext::HSD_LObjGetFlags(lobj) & 0x400 != 0 {
                next_gx = null_mut();
                break;
            }
            lobj = (*lobj).next;
        }
        lgobj = next_gx;
    }
    if lobj.is_null() && !fallback.is_null() {
        lobj = fallback;
    }
    if lobj.is_null() {
        __assert(FILE, 0x181, "lobj");
    }

    if ext::HSD_LObjGetPosition(lobj, &mut light_pos) == 0 {
        ext::OSReport("coudn t get light position ...\n", &[]);
        __assert(FILE, 0x184, "0");
    }

    if ext::HSD_LObjGetInterest(lobj, &mut light_dir) != 0 {
        ext::lbVector_Sub(&mut light_pos, &mut light_dir);
    }
    light_dir = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let mut light_vec = Vec3::zeroed();
    ext::lbVector_Diff(&mut light_pos, &mut light_dir, &mut light_vec);
    // light_vec's address goes to lbVector_Diff: squares not fused on the console (notes/fp-audit.md)
    let mut dist = light_vec.z * light_vec.z + (light_vec.x * light_vec.x + light_vec.y * light_vec.y);
    dist = crate::gekko::sqrtf(dist);

    if dist < 0.001 {
        no_light = true;
        light_pos.x = 0.0;
        light_dir.x = 0.0;
        up_vec.x = 0.0;
        light_pos.y = 0.0;
        light_dir.y = 0.0;
        up_vec.y = 1.0;
        light_pos.z = 1.0;
        light_dir.z = 0.0;
        up_vec.z = 0.0;
    } else {
        let xz_sq = light_vec.x * light_vec.x + light_vec.z * light_vec.z;
        if xz_sq > 0.0000010000001 {
            up_vec.z = 0.0;
            up_vec.x = 0.0;
            up_vec.y = 1.0;
            let mut norm_dir = Vec3::zeroed();
            ext::lbVector_Diff(&mut light_dir, &mut light_pos, &mut norm_dir);
            if ext::lbVector_Normalize(&mut norm_dir) < 100.0 {
                light_pos.x = -fmsubs(100.0, norm_dir.x, light_dir.x);
                light_pos.y = -fmsubs(100.0, norm_dir.y, light_dir.y);
                light_pos.z = -fmsubs(100.0, norm_dir.z, light_dir.z);
            }
            let mut right_vec = Vec3::zeroed();
            ext::lbVector_CrossprodNormalized(&mut up_vec, &mut norm_dir, &mut right_vec);
            ext::lbVector_CrossprodNormalized(&mut norm_dir, &mut right_vec, &mut up_vec);
        } else {
            up_vec.y = 0.0;
            up_vec.x = 0.0;
            up_vec.z = -1.0;
        }
    }

    ext::ftDrawCommon_80081200();

    let mut gobj = plink;
    while !gobj.is_null() {
        let fp = (*gobj).user_data as *mut Fighter;
        let cm = (*fp).x890_cameraBox;

        (*fp).x20A4.set_x0_b0(no_light as u8);

        if !(*fp).x20A4.shadow.is_null() {
            let mut added = false;
            ext::HSD_ShadowDeleteObject((*fp).x20A4.shadow, null_mut());
            if (*fp).x21FC_flag.anon_0.b7() != 0 {
                if (*fp).invisible() == 0
                    && (*fp).x221E_b5() == 0
                    && (!(*fp).x5AC.xC[1].is_null() || (*fp).kind > Ft_Kind_Sandbag)
                {
                    ext::HSD_ShadowAddObject((*fp).x20A4.shadow, (*gobj).hsd_obj as *mut HSD_JObj);
                    added = true;
                }
                if !(*fp).x20A0_accessory.is_null() {
                    ext::HSD_ShadowAddObject((*fp).x20A4.shadow, (*fp).x20A0_accessory);
                    added = true;
                }
            }
            (*fp).x20A4.set_x0_b3(if added { 0 } else { 1 });
        }

        if (ext::ftLib_IsFighter(gobj) != 0) {
            let lbs = ext::ftLib_GetShadow(gobj);
            if !lbs.is_null() {
                if !any_flag(lbs) && (ext::ftLib_CastsShadow(gobj) != 0) {
                    ext::HSD_ShadowSetActive((*lbs).shadow, 1);
                } else {
                    ext::HSD_ShadowSetActive((*lbs).shadow, 0);
                }
            }
        }

        if !(*fp).x20A4.shadow.is_null() && !cm.is_null() && (*fp).x20A4.x0_b3() == 0 {
            let intensity = ext::Ground_801C0508() as u8;
            let shadow2 = (*fp).x20A4.shadow;
            (*shadow2).intensity = intensity;

            ext::PSVECAdd(&mut (*cm).bone_pos, &mut light_pos, &mut eye_pos);
            ext::PSVECAdd(&mut (*cm).bone_pos, &mut light_dir, &mut interest_pos);
            ext::HSD_CObjSetEyePosition((*shadow2).camera, &mut eye_pos);
            ext::HSD_CObjSetInterest((*shadow2).camera, &mut interest_pos);
            ext::HSD_CObjSetUpVector((*shadow2).camera, &mut up_vec);

            let mut cam_pos = (*cm).bone_pos;
            ext::HSD_ViewingRectInit(&mut rect, &mut eye_pos, &mut interest_pos, &mut up_vec, 0);
            let mut i = 0;
            while i < 0x14 {
                let scale = (*cm).target_ext.v.z;
                let top = 1.2 * scale;
                let bot = 1.2 * -scale;
                ext::HSD_ViewingRectAddRect(&mut rect, &mut cam_pos, top, bot, bot, top);
                if ext::HSD_ViewingRectCheck(&mut rect) != 0 {
                    break;
                }
                i += 1;
            }
            if i < 0x14 {
                ext::HSD_ShadowSetViewingRect(shadow2, rect.top, rect.bottom, rect.left, rect.right);
            } else {
                ext::HSD_CObjSetOrtho((*shadow2).camera, 128.0, -128.0, -128.0, 128.0);
            }

            ext::HSD_ShadowInit(shadow2);
            ext::HSD_StartRender(HSD_RP_OFFSCREEN as u32);
            ext::HSD_GObj_804D7814 = gobj;
            ext::HSD_ShadowStartRender(shadow2);
            if arg0 != 0 {
                lbShadow_8000F214(shadow2);
            }
            ext::HSD_ShadowEndRender(shadow2);
            ext::HSD_Init_803755A8();
        }
        gobj = (*gobj).next;
    }

    ext::ftDrawCommon_80081168();
}
