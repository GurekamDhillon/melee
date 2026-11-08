//! Port of src/melee/gr/grdisplay.c (built `TARGET_PC`): the stage's GX render callback
//! `grDisplay_801C5DB0` (wrapped in `SurfaceBegin(7)` / `SurfaceEnd()`), the instanced-joint
//! walker `grDisplay_801C5B90` it uses to light fighters' shadows per joint, and the camera
//! render callback `grDisplay_801C5F60`. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_grdisplay.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_80030A50() -> *mut HSD_GObj = || unsafe { crate::tu::camera::Camera_80030A50() as _ };
    fn Camera_80030A78() -> i32 = || unsafe { crate::tu::camera::Camera_80030A78() } as i32;
    fn Camera_80030AC4() -> i32 = || unsafe { crate::tu::camera::Camera_80030AC4() } as i32;
    fn Camera_80030B24() -> i32 = || unsafe { crate::tu::camera::Camera_80030B24() } as i32;
    fn Camera_8003108C() -> i32 = || unsafe { crate::tu::camera::Camera_8003108C() };
    fn Camera_800310A0(a0: u8) = |a0: u8| unsafe { crate::tu::camera::Camera_800310A0(a0) };
    fn Ground_801C1E18() -> i32 = || unsafe { crate::tu::ground::Ground_801C1E18() as _ };
    fn Ground_801C1E84() -> *mut HSD_GObj = || unsafe { crate::tu::ground::Ground_801C1E84() as _ };
    fn Ground_801C2C8C(a0: *mut c_void) -> i32 = |a0: *mut c_void| unsafe { crate::tu::ground::Ground_801C2C8C(a0 as _) as _ };
    fn HSD_CObjEndCurrent() = || { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObj_80390EB8(a0: i32) -> u32 = |a0: i32| unsafe { melee_runtime::tu::gobj::HSD_GObj_80390EB8(a0 as _) as _ };
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_JObjDisp(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_MtxInverseConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = |a0: *mut c_void, a1: *mut c_void, a2: *mut c_void| { unsafe { melee_runtime::tu::mtx_world::HSD_MtxInverseConcat(a0 as _, a1 as _, a2 as _); } };
    fn HSD_StateInvalidate(a0: i32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = |a0: *mut c_void, a1: *mut c_void, a2: *mut c_void| { unsafe { melee_runtime::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _); } };
    fn ScriptGame_StageOriginalVisible() -> i32;
    fn ScriptGame_StageSlotOwned(a0: *mut HSD_GObj) -> i32;
    fn SurfaceBegin(a0: i32);
    fn SurfaceEnd();
    fn Wb_RenderOption(a0: i32, a1: i32) -> i32;
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_GetPos(a0 as _, a1 as _); } };
    fn ftLib_GetShadow(a0: *mut HSD_GObj) -> *mut LbShadow = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_GetShadow(a0 as _) as _ };
    fn lbShadow_8000EEE0(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::lbshadow::lbShadow_8000EEE0(a0 as _); } };
    fn lbShadow_8000EFEC() = || { unsafe { melee_runtime::tu::lbshadow::lbShadow_8000EFEC(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_804D7818;
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

/// `WB_RENDER_STAGE` (pc/workbench_render.h).
const WB_RENDER_STAGE: i32 = 7;

/// `HSD_ASSERT(line, cond)`: `__assert` does not return.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("grdisplay.c: HSD_ASSERT {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetupMatrix` (baselib/jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_CObjGetViewingMtxPtrDirect` (baselib/cobj.h, static inline).
#[inline]
unsafe fn HSD_CObjGetViewingMtxPtrDirect(cobj: *mut HSD_CObj) -> MtxPtr {
    (*cobj).view_mtx.as_mut_ptr()
}

/// 1C5B90: draws the joint tree below `jobj`; an instance joint concatenates its child's
/// inverse matrix, a plain joint lights every fighter's shadow flag first.
pub unsafe extern "C" fn grDisplay_801C5B90(mut jobj: *mut HSD_JObj, vmtx: MtxPtr, flags: u32, rendermode: u32) {
    melee_core::ct!("grDisplay_801C5B90");
    let mut mtx: Mtx = [[0.0; 4]; 3];
    let mut fighter_pos: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if !jobj.is_null() {
        if (*jobj).flags & JOBJ_INSTANCE as u32 != 0 {
            if (*jobj).flags & JOBJ_HIDDEN as u32 == 0 {
                HSD_JObjSetupMatrix(jobj);
                let child = (*jobj).child;
                if !child.is_null() && HSD_JObjMtxIsDirty(child) {
                    ext::HSD_JObjSetupMatrixSub(child);
                }

                ext::HSD_MtxInverseConcat((*(*jobj).child).mtx.as_mut_ptr().cast(), (*jobj).mtx.as_mut_ptr().cast(), mtx.as_mut_ptr().cast());
                let cobj = ext::HSD_CObjGetCurrent();
                hsd_assert(!cobj.is_null(), "(cobj)");
                ext::PSMTXConcat(HSD_CObjGetViewingMtxPtrDirect(cobj).cast(), mtx.as_mut_ptr().cast(), mtx.as_mut_ptr().cast());
                grDisplay_801C5B90((*jobj).child, mtx.as_mut_ptr(), flags, rendermode);
            }
        } else {
            if (*jobj).flags & (flags << 18) != 0 {
                let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
                while !cur.is_null() {
                    let shadow = ext::ftLib_GetShadow(cur);
                    ext::ftLib_GetPos(cur, &mut fighter_pos);
                    if ext::Camera_80030B24() != 0
                        || (ext::stage_info.on_check_shadow_render.expect("stage_info.on_check_shadow_render"))(&mut fighter_pos, -1, jobj) != 0
                    {
                        (*shadow).set_x0_b4(0);
                    } else {
                        (*shadow).set_x0_b4(1);
                    }
                    cur = (*cur).next;
                }
                ext::lbShadow_8000EFEC();
                ext::HSD_JObjDisp(jobj, vmtx.cast(), flags, rendermode);
            }
            if (*jobj).flags & (flags << 28) != 0 {
                jobj = (*jobj).child;
                while !jobj.is_null() {
                    grDisplay_801C5B90(jobj, vmtx, flags, rendermode);
                    jobj = (*jobj).next;
                }
            }
        }
    }
}

/// `grDisplay_801C5DB0_surface_impl` (the TARGET_PC body of 1C5DB0).
pub unsafe extern "C" fn grDisplay_801C5DB0_surface_impl(gobj: *mut HSD_GObj, code: i32) {
    melee_core::ct!("grDisplay_801C5DB0_surface_impl");
    // Only the original stage callback is hidden by the workbench option.
    if ext::Wb_RenderOption(WB_RENDER_STAGE, ext::ScriptGame_StageOriginalVisible()) == 0
        && ext::ScriptGame_StageSlotOwned(gobj) == 0
    {
        return;
    }

    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).x11_flags.b012() as i32 == ext::Camera_8003108C() as i32 {
        if !(*gp).x18.is_null() {
            // OSReport("oioi... %08x\n", gp->x18) when the pointer has no high bit: diagnostics only.
            if (*ext::HSD_GObj_804D7818).hsd_obj != (*(*gp).x18).hsd_obj {
                if (*gp).x10_flags.b3() == 0 {
                    return;
                }
                let camgobj = ext::Camera_80030A50();
                if camgobj.is_null() {
                    return;
                }
                let cobj = (*camgobj).hsd_obj as *mut HSD_CObj;
                if cobj.is_null() {
                    return;
                }
                if ext::HSD_CObjGetCurrent() != cobj {
                    return;
                }
            }
        } else if ext::Ground_801C2C8C((*ext::HSD_GObj_804D7818).hsd_obj.cast()) != 0 {
            return;
        }

        // Retail collision display replaces the stage mesh; stage visibility is independent.
        let fallback = (ext::Camera_80030A78() == 0 && ext::Camera_80030AC4() != 0) as i32;
        if ext::Wb_RenderOption(WB_RENDER_STAGE, fallback) != 0 {
            if (*gp).x10_flags.b2() == 0 {
                ext::HSD_FogSet(null_mut());
            }
            ext::HSD_StateInvalidate(-1);

            if (*gp).x10_flags.b5() != 0 {
                let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
                grDisplay_801C5B90(jobj, null_mut(), ext::HSD_GObj_80390EB8(code), 0);
            } else {
                let mut fighter = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
                while !fighter.is_null() {
                    let shadow = ext::ftLib_GetShadow(fighter);
                    (*shadow).set_x0_b4(0);
                    ext::lbShadow_8000EEE0(fighter);
                    fighter = (*fighter).next;
                }
                ext::HSD_GObj_JObjCallback(gobj, code);
            }

            if (*gp).x10_flags.b2() == 0 && ext::Camera_80030A78() == 0 && ext::Ground_801C1E18() != 0 {
                let foggobj = ext::Ground_801C1E84();
                if !foggobj.is_null() && !(*foggobj).hsd_obj.is_null() {
                    ext::HSD_FogSet((*foggobj).hsd_obj as *mut HSD_Fog);
                }
            }
        }
    }
}

/// 1C5DB0 (`TARGET_PC` wrapper): the stage render callback.
pub unsafe extern "C" fn grDisplay_801C5DB0(gobj: *mut HSD_GObj, code: i32) {
    melee_core::ct!("grDisplay_801C5DB0");
    ext::SurfaceBegin(7);
    grDisplay_801C5DB0_surface_impl(gobj, code);
    ext::SurfaceEnd();
}

/// 1C5F60
pub unsafe extern "C" fn grDisplay_801C5F60(gobj: *mut HSD_GObj, _code: i32) {
    melee_core::ct!("grDisplay_801C5F60");
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;
    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        (*gobj).gxlink_prios = 8;
        ext::Camera_800310A0(0);
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}
