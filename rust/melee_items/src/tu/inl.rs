//! The `static inline` helpers of sysdolphin/baselib/jobj.h, baselib/gobj.h, melee/it/inlines.h and
//! melee/it/kinds/inlines.h that the item-kind TUs use, as macros expanded inside each TU (the bodies
//! call the TU's own `ext::` hooks for the out-of-line callees, e.g. `HSD_JObjSetMtxDirtySub`).
//! Bodies and assert lines follow the C; `HSD_ASSERT` never returns (`__assert`), so it panics.

/// `HSD_GObjGetHSDObj` / `HSD_GObjGetUserData` / `GET_ITEM` / `GET_JOBJ`, the HSD_ASSERT helper and
/// the jobj.h inlines the item kinds use.
#[macro_export]
macro_rules! jobj_inlines_it {
    () => {
        /// `HSD_ASSERT(line, cond)`: `__assert` does not return.
        #[inline]
        fn hsd_assert(cond: bool, what: &str) {
            if !cond {
                panic!("HSD_ASSERT {what}");
            }
        }
        /// `HSD_GObjGetHSDObj` (baselib/gobj.h, static inline).
        #[inline]
        unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
            (*gobj).hsd_obj
        }
        /// `HSD_GObjGetUserData` (baselib/gobj.h, static inline).
        #[inline]
        unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void {
            (*gobj).user_data
        }
        /// `GET_ITEM(gobj)`.
        #[inline]
        unsafe fn GET_ITEM(gobj: *mut HSD_GObj) -> *mut Item {
            HSD_GObjGetUserData(gobj) as *mut Item
        }
        /// `GET_JOBJ(gobj)`.
        #[inline]
        unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
            HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj
        }
        /// `HSD_JObjMtxIsDirty`.
        #[inline]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
        }
        /// The `HSD_JObjSetMtxDirty` macro.
        #[inline]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
            if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child }
        }
        #[inline]
        unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.y
        }
        #[inline]
        unsafe fn HSD_JObjAddRotationY(jobj: *mut HSD_JObj, y: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.y += y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, x: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, y: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, z: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).scale.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!translate.is_null(), "(translate)");
            (*jobj).translate = *translate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!translate.is_null(), "(translate)");
            *translate = (*jobj).translate;
        }
    };
}

// Per-function macros so a TU only pulls the ext hooks it has.

#[macro_export]
macro_rules! inl_itResetVelocity {
    () => {
        /// `itResetVelocity`.
        #[inline]
        unsafe fn itResetVelocity(ip: *mut Item) {
            (*ip).x40_vel.x = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.z = 0.0;
        }
    };
}

#[macro_export]
macro_rules! inl_Item_InitSpawnPosition {
    () => {
        /// `Item_InitSpawnPosition`.
        #[inline]
        unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
            (*spawn).prev_pos = *pos;
            if on_plane {
                (*spawn).prev_pos.z = 0.0;
            }
            (*spawn).pos = (*spawn).prev_pos;
        }
    };
}

#[macro_export]
macro_rules! inl_Item_InitSpawnCommonFields {
    () => {
        /// `Item_InitSpawnCommonFields`.
        #[inline]
        unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
            (*spawn).facing_dir = facing_dir;
            (*spawn).x3C_damage = 0;
            (*spawn).vel.x = 0.0;
            (*spawn).vel.y = 0.0;
            (*spawn).vel.z = 0.0;
            (*spawn).x0_parent_gobj = parent;
            (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
            (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
            (*spawn).x40 = 0;
        }
    };
}

#[macro_export]
macro_rules! inl_Item_InitSpawnPositionFromParent {
    () => {
        /// `Item_InitSpawnPositionFromParent`.
        #[inline]
        unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
            (*spawn).prev_pos = *pos;
            (*spawn).prev_pos.z = 0.0;
            ext::it_8026BB68(parent, &raw mut (*spawn).pos);
        }
    };
}

#[macro_export]
macro_rules! inl_itUpdateVelocityFromBone {
    () => {
        /// `itUpdateVelocityFromBone`.
        #[inline]
        unsafe fn itUpdateVelocityFromBone(jobj: *mut HSD_JObj, ip: *mut Item, previous_pos: *mut Vec3) {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut zero = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            if !jobj.is_null() {
                zero.x = 0.0;
                zero.y = 0.0;
                zero.z = 0.0;
                HSD_JObjGetTranslation(jobj, &raw mut pos);
                (*ip).x40_vel.x = (*ip).facing_dir * (pos.z - (*previous_pos).z);
                (*ip).x40_vel.y = pos.y - (*previous_pos).y;
                (*ip).x40_vel.z = pos.x - (*previous_pos).x;
                *previous_pos = pos;
                HSD_JObjSetTranslate(jobj, &raw mut zero);
            }
        }
    };
}

#[macro_export]
macro_rules! inl_Item_SetEffectHitlagCallbacks {
    () => {
        /// `Item_SetEffectHitlagCallbacks`.
        #[inline]
        unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
            (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
            (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
        }
    };
}

#[macro_export]
macro_rules! inl_Item_EnterAirStateWithHitlag {
    () => {
        /// `Item_EnterAirStateWithHitlag`.
        #[inline]
        unsafe fn Item_EnterAirStateWithHitlag(gobj: *mut HSD_GObj, msid: i32) {
            let item = GET_ITEM(gobj);
            ext::it_802762BC(item);
            ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
            (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
            (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
        }
    };
}

#[macro_export]
macro_rules! inl_Item_ApplyFallingPhysics {
    () => {
        /// `Item_ApplyFallingPhysics`.
        #[inline]
        unsafe fn Item_ApplyFallingPhysics(gobj: *mut HSD_GObj) {
            let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
            ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
            ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
        }
    };
}

#[macro_export]
macro_rules! inl_Item_UpdateRollingShellRotation {
    () => {
        /// `Item_UpdateRollingShellRotation`.
        #[inline]
        unsafe fn Item_UpdateRollingShellRotation(gobj: *mut HSD_GObj, ip: *mut Item, jobj: *mut HSD_JObj, rotation_rate: *const f32) {
            ext::it_80276CB8(gobj);
            let jobj = HSD_JObjGetChild(jobj);
            let v = (*ip).x40_vel.x;
            HSD_JObjAddRotationY(jobj, *rotation_rate * if v < 0.0 { -v } else { v });
        }
    };
}

/// `HSD_JObjGetRotation`.
#[macro_export]
macro_rules! inl_HSD_JObjGetRotation {
    () => {
        #[inline]
        unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!rotate.is_null(), "(rotate)");
            *rotate = (*jobj).rotate;
        }
    };
}
