//! The `static inline` helpers of sysdolphin/baselib/jobj.h that the `if` TUs use, as one macro
//! expanded inside each TU (the macro form `HSD_JObjSetMtxDirty` calls the TU's own
//! `ext::HSD_JObjSetMtxDirtySub`, which is a per-TU hook). Bodies and assert lines follow the C.

#[macro_export]
macro_rules! jobj_inlines {
    () => {
        /// `HSD_ASSERT(line, cond)`: `__assert` does not return.
        #[inline]
        fn hsd_assert(cond: bool, what: &str) {
            if !cond {
                panic!("HSD_ASSERT {what}");
            }
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
            if jobj.is_null() { null_mut() } else { (*jobj).child }
        }
        #[inline]
        unsafe fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
            if jobj.is_null() { null_mut() } else { (*jobj).next }
        }
        #[inline]
        unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.z = z;
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
        unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, x: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.x = x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, y: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y = y;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[inline]
        unsafe fn HSD_JObjSetTranslateZ(jobj: *mut HSD_JObj, z: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.z = z;
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
        /// `HSD_JObjGetTranslation2` (only the first assert).
        #[inline]
        unsafe fn HSD_JObjGetTranslation2(jobj: *mut HSD_JObj, translate: *mut Vec3) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            *translate = (*jobj).translate;
        }
        #[inline]
        unsafe fn HSD_JObjGetTranslationX(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.x
        }
        #[inline]
        unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.y
        }
        #[inline]
        unsafe fn HSD_JObjAddTranslationX(jobj: *mut HSD_JObj, x: f32) {
            hsd_assert(!jobj.is_null(), "(jobj)");
            (*jobj).translate.x += x;
            if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}
