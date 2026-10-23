//! Whole-TU native ports of the item engine .c files (shard 649-656).

/// The `static inline` HSD_JObj setters of sysdolphin/baselib/jobj.h and the item-kind spawn helpers of
/// it/kinds/inlines.h, as every including TU compiles them (each TU's own `ext` holds the out-of-line
/// callee `HSD_JObjSetMtxDirtySub`). Invoked inside a TU module that has `ext::HSD_JObjSetMtxDirtySub`.
macro_rules! jobj_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
            !((*jobj).flags & JOBJ_USER_DEF_MTX as u32 != 0) && ((*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0)
        }
        /// `HSD_JObjSetMtxDirty` (macro): `if (jobj != NULL && !HSD_JObjMtxIsDirty(jobj)) HSD_JObjSetMtxDirtySub(jobj)`.
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
            if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            assert!(!jobj.is_null(), "HSD_ASSERT(618, jobj)");
            assert!(!rotate.is_null(), "HSD_ASSERT(619, rotate)");
            (*jobj).rotate = *rotate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
            assert!(!jobj.is_null(), "HSD_ASSERT(699, jobj)");
            assert!(!rotate.is_null(), "HSD_ASSERT(700, rotate)");
            *rotate = (*jobj).rotate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(639, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(640, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(681, jobj)");
            assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682, !(jobj->flags & JOBJ_USE_QUATERNION))");
            (*jobj).rotate.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(760, jobj)");
            assert!(!scale.is_null(), "HSD_ASSERT(761, scale)");
            (*jobj).scale = *scale;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScaleY(jobj: *mut HSD_JObj) -> f32 {
            assert!(!jobj.is_null(), "HSD_ASSERT(888, jobj)");
            (*jobj).scale.y
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(916, jobj)");
            assert!(!translate.is_null(), "HSD_ASSERT(917, translate)");
            (*jobj).translate = *translate;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            assert!(!jobj.is_null(), "HSD_ASSERT(979, jobj)");
            assert!(!translate.is_null(), "HSD_ASSERT(980, translate)");
            *translate = (*jobj).translate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, x: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(776, jobj)");
            (*jobj).scale.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, y: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(791, jobj)");
            (*jobj).scale.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, z: f32) {
            assert!(!jobj.is_null(), "HSD_ASSERT(806, jobj)");
            (*jobj).scale.z = z;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}

/// it/kinds/inlines.h spawn helpers (`Item_InitSpawnPosition`, `Item_InitSpawnCommonFields`).
macro_rules! spawn_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
            (*spawn).prev_pos = *pos;
            if on_plane {
                (*spawn).prev_pos.z = 0.0;
            }
            (*spawn).pos = (*spawn).prev_pos;
        }
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
            (*spawn).facing_dir = facing_dir;
            (*spawn).x3C_damage = 0;
            (*spawn).vel.x = 0.0;
            (*spawn).vel.y = 0.0;
            (*spawn).vel.z = 0.0;
            (*spawn).x0_parent_gobj = parent;
            (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
            (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
            (*spawn).x40 = 0;
        }
    };
}

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h): needs `ext::it_8026BB68`.
macro_rules! spawn_parent_inl {
    () => {
        #[allow(dead_code)]
        unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
            (*spawn).prev_pos = *pos;
            (*spawn).prev_pos.z = 0.0;
            ext::it_8026BB68(parent, &mut (*spawn).pos);
        }
    };
}

pub mod itanimlist;
pub mod itclimbersice;
pub mod itclinkmilk;
pub mod itcrazyhandbomb;
pub mod itdkinoko;
pub mod itcoin;
pub mod itclimbersstring;
pub mod itdrmariopill;
pub mod itdosei;
pub mod itmaterial;
pub mod itspawn;
pub mod itzako;
pub mod it_26B1;
pub mod it_2725;
pub mod it_279C;
pub mod it_3F14;
pub mod it_3F2F;
