//! Header `static inline` helpers shared by the item-kind TU ports of shards 729-736 (the C expands them inside
//! each TU, so they are macros here: their bodies call the TU's own `ext::` hooks). Bodies and `HSD_ASSERT`
//! lines follow baselib/gobj.h, baselib/jobj.h, melee/it/inlines.h and melee/it/kinds/inlines.h.

/// GObj / item accessors, `HSD_ASSERT`, `itResetVelocity`, `Item_ClearCmdVars`, spawn-record initialisers.
#[macro_export]
macro_rules! inl_it_core {
    () => {
        /// `HSD_ASSERT(line, cond)`: `__assert` does not return.
        #[inline]
        fn hsd_assert(cond: bool, what: &str) {
            if !cond {
                panic!("HSD_ASSERT {what}");
            }
        }
        #[inline]
        unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
            (*gobj).hsd_obj
        }
        #[inline]
        unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void {
            (*gobj).user_data
        }
        #[inline]
        unsafe fn GET_ITEM(gobj: *mut HSD_GObj) -> *mut Item {
            HSD_GObjGetUserData(gobj) as *mut Item
        }
        #[inline]
        unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
            HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj
        }
        #[inline]
        unsafe fn itResetVelocity(ip: *mut Item) {
            (*ip).x40_vel.x = 0.0;
            (*ip).x40_vel.y = 0.0;
            (*ip).x40_vel.z = 0.0;
        }
        #[inline]
        unsafe fn Item_ClearCmdVars(ip: *mut Item) {
            (*ip).xDB8_itcmd_var3 = 0;
            (*ip).xDB4_itcmd_var2 = 0;
            (*ip).xDB0_itcmd_var1 = 0;
            (*ip).xDAC_itcmd_var0 = 0;
        }
        #[inline]
        unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
            (*spawn).prev_pos = *pos;
            if on_plane {
                (*spawn).prev_pos.z = 0.0;
            }
            (*spawn).pos = (*spawn).prev_pos;
        }
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

/// `Item_InitSpawnPositionFromParent` (calls `ext::it_8026BB68`).
#[macro_export]
macro_rules! inl_it_spawn_from_parent {
    () => {
        #[inline]
        unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
            (*spawn).prev_pos = *pos;
            (*spawn).prev_pos.z = 0.0;
            ext::it_8026BB68(parent, &raw mut (*spawn).pos);
        }
    };
}

/// jobj.h: `HSD_JObjMtxIsDirty`, the `HSD_JObjSetMtxDirty` macro and the setters/getters the item kinds use
/// (calls `ext::HSD_JObjSetMtxDirtySub`).
#[macro_export]
macro_rules! inl_it_jobj {
    () => {
        #[inline]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
        }
        #[inline]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!scale.is_null(), "(scale)");
            (*jobj).scale = *scale;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}

/// `Item_ApplyFallingPhysics` (calls `ext::it_80272860`, `ext::it_80274658`, reads `ext::it_804D6D28`).
#[macro_export]
macro_rules! inl_it_fall {
    () => {
        #[inline]
        unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
            let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
            ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
            ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
        }
    };
}

/// `Item_SetEffectHitlagCallbacks` (the addresses of `efLib_PauseAll` / `efLib_ResumeAll`: `ext::cb`).
#[macro_export]
macro_rules! inl_it_hitlag_cb {
    () => {
        #[inline]
        unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
            (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
            (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
        }
    };
}

/// `Item_EnterAirStateWithHitlagAndStateDesc` (calls `ext::it_802762BC`, `ext::Item_80268E5C`, `ext::it_80273670`).
#[macro_export]
macro_rules! inl_it_enter_air_hitlag_desc {
    () => {
        #[inline]
        unsafe fn Item_EnterAirStateWithHitlagAndStateDesc(gobj: *mut Item_GObj) {
            let item = GET_ITEM(gobj);
            ext::it_802762BC(item);
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
            (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
            (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
            ext::it_80273670(gobj, 0, 0.0);
        }
    };
}

/// `Item_TickLifetime`.
#[macro_export]
macro_rules! inl_it_tick {
    () => {
        #[inline]
        unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
            if (*ip).xD44_lifeTimer <= 0.0 {
                return true;
            }
            (*ip).xD44_lifeTimer -= 1.0;
            false
        }
    };
}

/// `Item_EnterStateWithEffectHitlag` (calls `ext::it_802762BC`, `ext::Item_80268E5C`; uses the `ext::cb` shims).
#[macro_export]
macro_rules! inl_it_enter_state_hitlag {
    () => {
        #[inline]
        unsafe fn Item_EnterStateWithEffectHitlag(gobj: *mut Item_GObj, msid: i32) {
            let ip = GET_ITEM(gobj);
            ext::it_802762BC(ip);
            ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
            (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
            (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
        }
    };
}

/// jobj.h accessors the item kinds use (need `inl_it_core` and `inl_it_jobj`).
#[macro_export]
macro_rules! inl_it_jobj_acc {
    () => {
        #[inline]
        pub unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
            if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child }
        }
        #[inline]
        pub unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!rotate.is_null(), "(rotate)");
            (*jobj).rotate = *rotate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        pub unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        pub unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        pub unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        pub unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!rotate.is_null(), "(rotate)");
            *rotate = (*jobj).rotate;
        }
        #[inline]
        pub unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).rotate.z
        }
        #[inline]
        pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!translate.is_null(), "(translate)");
            (*jobj).translate = *translate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}

/// jobj.h `HSD_JObjSetupMatrix` / `HSD_JObjCopyMtx` (call `ext::HSD_JObjSetupMatrixSub` / `ext::PSMTXCopy`).
#[macro_export]
macro_rules! inl_it_jobj_mtx {
    () => {
        #[inline]
        pub unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
            if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
                return;
            }
            ext::HSD_JObjSetupMatrixSub(jobj);
        }
        #[inline]
        pub unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *mut c_void) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert(!mtx.is_null(), "(mtx)");
            ext::PSMTXCopy(mtx, (*jobj).mtx.as_mut_ptr() as *mut c_void);
        }
    };
}
