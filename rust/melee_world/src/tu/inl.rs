//! `static inline` helpers of baselib/jobj.h, expanded into the TU that uses them (the C compiler
//! inlines them there; their `HSD_ASSERT`s go to the TU's `ext::__assert`, and the
//! `HSD_JObjSetMtxDirtySub` call to `ext::HSD_JObjSetMtxDirtySub`).
macro_rules! hsd_jobj_inl {
    () => {
        #[inline(always)]
        unsafe fn hsd_assert(line: u32, cond: bool, msg: &core::ffi::CStr) {
            if !cond {
                ext::__assert(c"jobj.h".as_ptr(), line, msg.as_ptr());
            }
        }
        /// `HSD_JObjMtxIsDirty`.
        #[allow(dead_code)]
        unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
            hsd_assert(564, !jobj.is_null(), c"jobj");
            let mut result = false;
            if ((*jobj).flags & JOBJ_USER_DEF_MTX as u32) == 0 && ((*jobj).flags & JOBJ_MTX_DIRTY as u32) != 0 {
                result = true;
            }
            result
        }
        /// The `HSD_JObjSetMtxDirty(jobj)` macro.
        #[inline(always)]
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
            if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(639, !jobj.is_null(), c"jobj");
            hsd_assert(640, ((*jobj).flags & JOBJ_USE_QUATERNION as u32) == 0, c"!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.x = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(660, !jobj.is_null(), c"jobj");
            hsd_assert(661, ((*jobj).flags & JOBJ_USE_QUATERNION as u32) == 0, c"!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.y = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(681, !jobj.is_null(), c"jobj");
            hsd_assert(682, ((*jobj).flags & JOBJ_USE_QUATERNION as u32) == 0, c"!(jobj->flags & JOBJ_USE_QUATERNION)");
            (*jobj).rotate.z = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(776, !jobj.is_null(), c"jobj");
            (*jobj).scale.x = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(791, !jobj.is_null(), c"jobj");
            (*jobj).scale.y = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScaleZ(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(806, !jobj.is_null(), c"jobj");
            (*jobj).scale.z = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslateX(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(932, !jobj.is_null(), c"jobj");
            (*jobj).translate.x = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(947, !jobj.is_null(), c"jobj");
            (*jobj).translate.y = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslateZ(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert(962, !jobj.is_null(), c"jobj");
            (*jobj).translate.z = v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, v: *mut Quaternion) {
            hsd_assert(618, !jobj.is_null(), c"jobj");
            hsd_assert(619, !v.is_null(), c"rotate");
            (*jobj).rotate = *v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetScale(jobj: *mut HSD_JObj, v: *mut Vec3) {
            hsd_assert(760, !jobj.is_null(), c"jobj");
            hsd_assert(761, !v.is_null(), c"scale");
            (*jobj).scale = *v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, v: *mut Vec3) {
            hsd_assert(916, !jobj.is_null(), c"jobj");
            hsd_assert(917, !v.is_null(), c"translate");
            (*jobj).translate = *v;
            if ((*jobj).flags & JOBJ_MTX_INDEP_SRT as u32) == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, v: *mut Quaternion) {
            hsd_assert(699, !jobj.is_null(), c"jobj");
            hsd_assert(700, !v.is_null(), c"rotate");
            *v = (*jobj).rotate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(715, !jobj.is_null(), c"jobj");
            (*jobj).rotate.x
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(730, !jobj.is_null(), c"jobj");
            (*jobj).rotate.y
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(745, !jobj.is_null(), c"jobj");
            (*jobj).rotate.z
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScale(jobj: *mut HSD_JObj, v: *mut Vec3) {
            hsd_assert(823, !jobj.is_null(), c"jobj");
            hsd_assert(824, !v.is_null(), c"scale");
            *v = (*jobj).scale;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScaleX(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(875, !jobj.is_null(), c"jobj");
            (*jobj).scale.x
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScaleY(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(888, !jobj.is_null(), c"jobj");
            (*jobj).scale.y
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetScaleZ(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(901, !jobj.is_null(), c"jobj");
            (*jobj).scale.z
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, v: *mut Vec3) {
            hsd_assert(979, !jobj.is_null(), c"jobj");
            hsd_assert(980, !v.is_null(), c"translate");
            *v = (*jobj).translate;
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslationX(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(993, !jobj.is_null(), c"jobj");
            (*jobj).translate.x
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(1006, !jobj.is_null(), c"jobj");
            (*jobj).translate.y
        }
        #[allow(dead_code)]
        unsafe fn HSD_JObjGetTranslationZ(jobj: *mut HSD_JObj) -> f32 {
            hsd_assert(1019, !jobj.is_null(), c"jobj");
            (*jobj).translate.z
        }
    };
}
pub(crate) use hsd_jobj_inl;
