//! Header inlines shared by the item-kind TU ports (`src/melee/it/kinds/*.c`): the `static inline`
//! functions of `sysdolphin/baselib/jobj.h` and `gobj.h`. `HSD_JObjSetMtxDirtySub` lives in an
//! unported TU, so every port reaches it through its own `ext` module: `jobj_inl!()` expands the
//! helpers in the caller's scope (the macro body names `ext::HSD_JObjSetMtxDirtySub`).
//!
//! `HSD_ASSERT` does not return in the C build (`__assert` loops / aborts): `hsd_assert` panics.
#![allow(non_snake_case, unused)]

use melee_types::*;

/// `HSD_ASSERT(line, cond)`
#[inline]
pub fn hsd_assert(cond: bool, what: &str) {
    melee_core::ct!("hsd_assert");
    if !cond {
        panic!("HSD_ASSERT {what}");
    }
}

/// `HSD_GObjGetUserData` (baselib/gobj.h, static inline).
#[inline]
pub unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    (*gobj).user_data
}

/// `HSD_GObjGetHSDObj` (baselib/gobj.h, static inline).
#[inline]
pub unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut core::ffi::c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    (*gobj).hsd_obj
}

/// `GET_ITEM(gobj)` (it/inlines.h).
#[inline]
pub unsafe fn GET_ITEM(gobj: *mut HSD_GObj) -> *mut Item {
    melee_core::ct!("GET_ITEM");
    HSD_GObjGetUserData(gobj) as *mut Item
}

/// `GET_JOBJ(gobj)` (gobj.h).
#[inline]
pub unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    melee_core::ct!("GET_JOBJ");
    HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj
}

/// `itResetVelocity` (it/inlines.h, static inline).
#[inline]
pub unsafe fn itResetVelocity(ip: *mut Item) {
    melee_core::ct!("itResetVelocity");
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
}

/// `HSD_JObjGetChild` (baselib/jobj.h, static inline): NULL-safe.
#[inline]
pub unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetChild");
    if jobj.is_null() {
        core::ptr::null_mut()
    } else {
        (*jobj).child
    }
}

/// Expands the mtx-dirty jobj.h inlines in the caller's scope (they reach
/// `ext::HSD_JObjSetMtxDirtySub`).
#[macro_export]
macro_rules! kinds_jobj_inl {
    () => {
        /// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
        #[inline]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
        }

        /// `HSD_JObjSetMtxDirty` (macro): marks the joint's matrix dirty unless it already is.
        #[inline]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }

        /// `HSD_JObjSetRotation`
        #[inline]
        unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!rotate.is_null(), "(rotate)");
            (*jobj).rotate = *rotate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetRotationX`
        #[inline]
        unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetRotationY`
        #[inline]
        unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetRotationZ`
        #[inline]
        unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjGetRotation`
        #[inline]
        unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!rotate.is_null(), "(rotate)");
            *rotate = (*jobj).rotate;
        }

        /// `HSD_JObjGetRotationX`
        #[inline]
        unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.x
        }

        /// `HSD_JObjGetRotationY`
        #[inline]
        unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.y
        }

        /// `HSD_JObjAddTranslationZ`
        #[inline]
        unsafe fn HSD_JObjAddTranslationZ(jobj: *mut HSD_JObj, z: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.z += z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetScale`
        #[inline]
        unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!scale.is_null(), "(scale)");
            (*jobj).scale = *scale;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetScaleZ`
        #[inline]
        unsafe fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, z: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjGetScaleY`
        #[inline]
        unsafe fn HSD_JObjGetScaleY(jobj: *mut HSD_JObj) -> f32 {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.y
        }

        /// `HSD_JObjGetRotationZ`
        #[inline]
        unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.z
        }

        /// `HSD_JObjGetScale`
        #[inline]
        unsafe fn HSD_JObjGetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!scale.is_null(), "(scale)");
            *scale = (*jobj).scale;
        }

        /// `HSD_JObjSetTranslate`
        #[inline]
        unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!translate.is_null(), "(translate)");
            (*jobj).translate = *translate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetTranslateY`
        #[inline]
        unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjGetTranslation`
        #[inline]
        unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert(!translate.is_null(), "(translate)");
            *translate = (*jobj).translate;
        }

        /// `HSD_JObjGetTranslationY`
        #[inline]
        unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y
        }

        /// `HSD_JObjAddRotationZ`
        #[inline]
        unsafe fn HSD_JObjAddRotationZ(jobj: *mut HSD_JObj, z: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.z += z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetScaleY`
        #[inline]
        unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddRotationX`
        #[inline]
        unsafe fn HSD_JObjAddRotationX(jobj: *mut HSD_JObj, x: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.x += x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddRotationY`
        #[inline]
        unsafe fn HSD_JObjAddRotationY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.y += y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddScaleX`
        #[inline]
        unsafe fn HSD_JObjAddScaleX(jobj: *mut HSD_JObj, x: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.x += x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddScaleY`
        #[inline]
        unsafe fn HSD_JObjAddScaleY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.y += y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddScaleZ`
        #[inline]
        unsafe fn HSD_JObjAddScaleZ(jobj: *mut HSD_JObj, z: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.z += z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjAddTranslationY`
        #[inline]
        unsafe fn HSD_JObjAddTranslationY(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y += y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}
pub use kinds_jobj_inl;

/// A state-table predicate slot from a `bool`-returning function, through a real `int`-returning shim: a Rust `bool` comes back in
/// `al` only, and the C side (`animated`, `collided`, ... read as `int`) tests all of `eax`, whose upper bits are whatever the
/// callee left there (a transmute of the function pointer made items vanish at random: cpu_peach frame 1572).
macro_rules! pred_b {
    ($f:expr) => {{
        unsafe extern "C" fn w(g: *mut HSD_GObj) -> i32 {
            unsafe { ($f)(g) as i32 }
        }
        Some(w as unsafe extern "C" fn(*mut HSD_GObj) -> i32)
    }};
}
pub(crate) use pred_b;

/// A state-table predicate slot from a `void` function (the C casts `(HSD_GObjPredicate) (Event)`).
pub const fn pred_v(f: unsafe extern "C" fn(*mut HSD_GObj)) -> HSD_GObjPredicate {
    unsafe { core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj), HSD_GObjPredicate>(f) }
}

/// A state-table row.
pub const fn st_row(anim_id: i32, animated: HSD_GObjPredicate, phys: HSD_GObjEvent, coll: HSD_GObjPredicate) -> ItemStateTable {
    ItemStateTable { anim_id, animated, physics_updated: phys, collided: coll }
}

/// The `...WithMtxDirty` jobj.h inlines (they call the out-of-line `(HSD_JObjSetMtxDirty)(jobj)`, which an
/// `ext` hook provides); expanded in the caller's scope.
#[macro_export]
macro_rules! kinds_jobj_inl_dirty {
    () => {
        /// `HSD_JObjSetRotationXWithMtxDirty`
        #[inline]
        unsafe fn HSD_JObjSetRotationXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            $crate::tu::kinds_inl::hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                ext::HSD_JObjSetMtxDirty(jobj);
            }
        }

        /// `HSD_JObjSetTranslateYWithMtxDirty`
        #[inline]
        unsafe fn HSD_JObjSetTranslateYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
            $crate::tu::kinds_inl::hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                ext::HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}
pub use kinds_jobj_inl_dirty;

/// `ABS(x)` (`x < 0 ? -x : x`; `ABS(-0.0)` is -0.0).
#[inline]
pub fn ABS(x: f32) -> f32 {
    melee_core::ct!("ABS");
    if x < 0.0 { -x } else { x }
}

/// `M_TAU` (Runtime/platform.h, a double).
pub const M_TAU: f64 = 6.283185307179586;

/// `Item_ClearCmdVars` (it/inlines.h, static inline).
#[inline]
pub unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    melee_core::ct!("Item_ClearCmdVars");
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline).
#[inline]
pub unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    melee_core::ct!("Item_InitSpawnPosition");
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline).
#[inline]
pub unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    melee_core::ct!("Item_InitSpawnCommonFields");
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `Item_NormalizeAngle` (it/kinds/inlines.h, static inline): `*angle += M_TAU` is a double add.
#[inline]
pub unsafe fn Item_NormalizeAngle(angle: *mut f32) {
    melee_core::ct!("Item_NormalizeAngle");
    while (*angle as f64) < 0.0 {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
    while (*angle as f64) > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}
