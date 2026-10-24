//! Native, pointer-level port of the parts of `src/sysdolphin/baselib/jobj.c` and the `jobj.h` inlines that
//! the item/trophy/vi TUs call through their `ext` stubs: the matrix-dirty propagation, the flag accessors
//! (`SetFlags`/`ClearFlags` and their `All` walkers), `GetDObj`, `AddChild`, and the `*WithMtxDirty` setters.
//! They operate on `*mut HSD_JObj` exactly as the C does (the tree is the C's `child`/`next`/`parent` links).
//! Checked by tests/native_jobj.rs against the C (clang, shipping contraction).
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ptr::null_mut;
use melee_types::*;

const JOBJ_ROOT_OPA: u32 = 1 << 28;
const JOBJ_ROOT_XLU: u32 = 1 << 29;
const JOBJ_ROOT_TEXEDGE: u32 = 1 << 30;
const JOBJ_ROOT_MASK: u32 = JOBJ_ROOT_OPA | JOBJ_ROOT_TEXEDGE | JOBJ_ROOT_XLU;

/// `HSD_JObjMtxIsDirty` (jobj.h inline).
#[inline(always)]
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "HSD_ASSERT(564, jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirtySub`: mark `jobj` and, unless it is an instance, every descendant whose matrix is
/// parent-dependent and not already dirty.
pub unsafe fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) {
    (*jobj).flags |= 0x40;
    if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 {
        let mut child = (*jobj).child;
        while !child.is_null() {
            if (*child).flags & JOBJ_MTX_INDEP_PARENT as u32 == 0 && !HSD_JObjMtxIsDirty(child) {
                HSD_JObjSetMtxDirtySub(child);
            }
            child = (*child).next;
        }
    }
}

/// The `HSD_JObjSetMtxDirty` macro.
#[inline(always)]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        HSD_JObjSetMtxDirtySub(jobj);
    }
}

pub unsafe fn HSD_JObjGetFlags(jobj: *mut HSD_JObj) -> u32 {
    if !jobj.is_null() {
        return (*jobj).flags;
    }
    0
}

pub unsafe fn HSD_JObjSetFlags(jobj: *mut HSD_JObj, flags: u32) {
    if !jobj.is_null() {
        if ((*jobj).flags ^ flags) & JOBJ_CLASSICAL_SCALE as u32 != 0 {
            // manually inlined HSD_JObjSetMtxDirty
            if !HSD_JObjMtxIsDirty(jobj) {
                HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        (*jobj).flags |= flags;
    }
}

pub unsafe fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    if !jobj.is_null() {
        HSD_JObjSetFlags(jobj, flags);
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjSetFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

pub unsafe fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, arg1: u32) {
    if !jobj.is_null() {
        if ((*jobj).flags ^ arg1) & JOBJ_CLASSICAL_SCALE as u32 != 0 {
            // manually inlined HSD_JObjSetMtxDirty
            if !HSD_JObjMtxIsDirty(jobj) {
                HSD_JObjSetMtxDirtySub(jobj);
            }
        }
        (*jobj).flags &= !arg1;
    }
}

pub unsafe fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    if !jobj.is_null() {
        HSD_JObjClearFlags(jobj, flags);
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjClearFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

/// `HSD_JObjGetDObj`: the DObj list head unless the union holds a particle list or a spline.
pub unsafe fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    if jobj.is_null() || (*jobj).flags & (JOBJ_PTCL | JOBJ_SPLINE) as u32 != 0 {
        return null_mut();
    }
    (*jobj).u.dobj
}

/// `UpdateParentTrspBits` (jobj.c, static).
unsafe fn UpdateParentTrspBits(mut jobj: *mut HSD_JObj, child: *mut HSD_JObj) {
    let flags = ((*child).flags | ((*child).flags << 10)) & JOBJ_ROOT_MASK;
    while !jobj.is_null() {
        if flags & !(*jobj).flags == 0 {
            break;
        }
        (*jobj).flags |= flags;
        jobj = (*jobj).parent;
    }
}

pub unsafe fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj) {
    if jobj.is_null() || child.is_null() {
        return;
    }
    assert!((*child).parent.is_null(), "HSD_ASSERTREPORT(1350, child should be a orphan.)");
    assert!((*child).next.is_null(), "HSD_ASSERTREPORT(1351, child should not have siblings)");
    if (*jobj).child.is_null() {
        (*jobj).child = child;
    } else {
        assert!((*jobj).flags & JOBJ_INSTANCE as u32 == 0, "HSD_ASSERT(1357, !(jobj->flags & JOBJ_INSTANCE))");
        let mut last = (*jobj).child;
        while !(*last).next.is_null() {
            assert!(last != child, "HSD_ASSERT(1360, last != child)");
            last = (*last).next;
        }
        (*last).next = child;
    }
    (*child).parent = jobj;
    UpdateParentTrspBits(jobj, child);
}

// ---- jobj.h static inlines with the dirty marking (HSD_JObjSetMtxDirty macro expanded) ----

pub unsafe fn HSD_JObjSetRotationYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetScaleXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(776, jobj)");
    (*jobj).scale.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetScaleYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(791, jobj)");
    (*jobj).scale.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetScaleZWithMtxDirty(jobj: *mut HSD_JObj, z: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(806, jobj)");
    (*jobj).scale.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetTranslateXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(932, jobj)");
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetTranslateYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(947, jobj)");
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetTranslateZWithMtxDirty(jobj: *mut HSD_JObj, z: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(962, jobj)");
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}
