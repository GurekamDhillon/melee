//! The raw-record tree and flag half of `src/sysdolphin/baselib/jobj.c` that the stage, HUD and
//! effect TUs call on `HSD_JObj` pointers: `HSD_JObjGetFlags`, `SetFlags(All)`, `ClearFlags(All)`,
//! `SetMtxDirtySub`, `GetDObj`, `GetPrev`, `AddChild`, `Reparent`, `AddNext` and the two static
//! root-transparency helpers (`UpdateParentTrspBits`, `RecalcParentTrspBits`). No float math.
//! Asserts of the C (`HSD_ASSERT` halts) are panics here. Checked bit for bit against the C
//! (jobj.c over the real build) by `tests/pure_jobj_world.rs`
//! (`python rust/melee_runtime/gen/pure/run_pure.py jobj_world`).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_types::{HSD_DObj, HSD_JObj};

const JOBJ_CLASSICAL_SCALE: u32 = 1 << 3;
const JOBJ_PTCL: u32 = 1 << 5;
const JOBJ_MTX_DIRTY: u32 = 1 << 6;
const JOBJ_INSTANCE: u32 = 1 << 12;
const JOBJ_SPLINE: u32 = 1 << 14;
const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
const JOBJ_MTX_INDEP_PARENT: u32 = 1 << 24;
const JOBJ_ROOT_MASK: u32 = (1 << 28) | (1 << 30) | (1 << 29);

/// `HSD_JObjMtxIsDirty` (inline; asserts the pointer).
unsafe fn mtx_is_dirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "jobj.h:564 HSD_ASSERT(jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX == 0 && (*jobj).flags & JOBJ_MTX_DIRTY != 0
}

/// `HSD_JObjGetFlags`.
pub unsafe fn HSD_JObjGetFlags(jobj: *mut HSD_JObj) -> u32 {
    melee_core::ct!("HSD_JObjGetFlags");
    if jobj.is_null() {
        return 0;
    }
    (*jobj).flags
}

/// `HSD_JObjSetMtxDirtySub`: mark `jobj` dirty and, unless it is an instance, every descendant
/// that is not independent of its parent.
pub unsafe fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetMtxDirtySub");
    (*jobj).flags |= JOBJ_MTX_DIRTY;
    if (*jobj).flags & JOBJ_INSTANCE == 0 {
        let mut child = (*jobj).child;
        while !child.is_null() {
            if (*child).flags & JOBJ_MTX_INDEP_PARENT == 0 && !mtx_is_dirty(child) {
                HSD_JObjSetMtxDirtySub(child);
            }
            child = (*child).next;
        }
    }
}

/// `HSD_JObjSetFlags`.
pub unsafe fn HSD_JObjSetFlags(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjSetFlags");
    if !jobj.is_null() {
        if ((*jobj).flags ^ flags) & JOBJ_CLASSICAL_SCALE != 0 && !mtx_is_dirty(jobj) {
            HSD_JObjSetMtxDirtySub(jobj);
        }
        (*jobj).flags |= flags;
    }
}

/// `HSD_JObjSetFlagsAll`.
pub unsafe fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjSetFlagsAll");
    if !jobj.is_null() {
        HSD_JObjSetFlags(jobj, flags);
        if (*jobj).flags & JOBJ_INSTANCE == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjSetFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

/// `HSD_JObjClearFlags`.
pub unsafe fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, arg1: u32) {
    melee_core::ct!("HSD_JObjClearFlags");
    if !jobj.is_null() {
        if ((*jobj).flags ^ arg1) & JOBJ_CLASSICAL_SCALE != 0 && !mtx_is_dirty(jobj) {
            HSD_JObjSetMtxDirtySub(jobj);
        }
        (*jobj).flags &= !arg1;
    }
}

/// `HSD_JObjClearFlagsAll`.
pub unsafe fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjClearFlagsAll");
    if !jobj.is_null() {
        HSD_JObjClearFlags(jobj, flags);
        if (*jobj).flags & JOBJ_INSTANCE == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjClearFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

/// `HSD_JObjGetDObj`: the DObj list when the union holds one (not a particle / spline JObj).
pub unsafe fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    melee_core::ct!("HSD_JObjGetDObj");
    if jobj.is_null() || (*jobj).flags & (JOBJ_PTCL | JOBJ_SPLINE) != 0 {
        return core::ptr::null_mut();
    }
    (*jobj).u.dobj
}

/// `HSD_JObjGetPrev`: the sibling before `jobj` (NULL for a first child or an orphan).
pub unsafe fn HSD_JObjGetPrev(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetPrev");
    if jobj.is_null() || (*jobj).parent.is_null() {
        return core::ptr::null_mut();
    }
    if jobj == (*(*jobj).parent).child {
        return core::ptr::null_mut();
    }
    let mut cur = (*(*jobj).parent).child;
    while !cur.is_null() {
        if (*cur).next == jobj {
            return cur;
        }
        cur = (*cur).next;
    }
    panic!("jobj.c:1516 can not find specified jobj. maybe jobj tree is broken.");
}

/// `RecalcParentTrspBits`.
pub unsafe fn RecalcParentTrspBits(mut jobj: *mut HSD_JObj) {
    melee_core::ct!("RecalcParentTrspBits");
    while !jobj.is_null() {
        let mut child = (*jobj).child;
        let mut flags = !JOBJ_ROOT_MASK;
        while !child.is_null() {
            flags |= ((*child).flags | ((*child).flags << 10)) & JOBJ_ROOT_MASK;
            child = (*child).next;
        }
        if (*jobj).flags & !flags == 0 {
            break;
        }
        (*jobj).flags &= flags;
        jobj = (*jobj).next;
    }
}

/// `UpdateParentTrspBits` (static in the C).
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

/// `HSD_JObjAddChild`.
pub unsafe fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjAddChild");
    if jobj.is_null() || child.is_null() {
        return;
    }
    assert!((*child).parent.is_null(), "jobj.c:1350 child should be a orphan.");
    assert!((*child).next.is_null(), "jobj.c:1351 child should not have siblings");
    if (*jobj).child.is_null() {
        (*jobj).child = child;
    } else {
        assert!((*jobj).flags & JOBJ_INSTANCE == 0, "jobj.c:1357 !(jobj->flags & JOBJ_INSTANCE)");
        let mut last = (*jobj).child;
        while !(*last).next.is_null() {
            assert!(last != child, "jobj.c:1360 last != child");
            last = (*last).next;
        }
        (*last).next = child;
    }
    (*child).parent = jobj;
    UpdateParentTrspBits(jobj, child);
}

/// `HSD_JObjReparent`: detach `jobj` from its parent and append it to `parent`; returns the old next sibling.
pub unsafe fn HSD_JObjReparent(jobj: *mut HSD_JObj, parent: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjReparent");
    if jobj.is_null() {
        return core::ptr::null_mut();
    }
    let next = (*jobj).next;
    if !(*jobj).parent.is_null() {
        if (*(*jobj).parent).child == jobj {
            (*(*jobj).parent).child = next;
        } else {
            let prev = HSD_JObjGetPrev(jobj);
            assert!(!prev.is_null(), "jobj.c:0x56F prev");
            (*prev).next = next;
        }
        RecalcParentTrspBits((*jobj).parent);
        (*jobj).parent = core::ptr::null_mut();
    }
    (*jobj).next = core::ptr::null_mut();
    HSD_JObjAddChild(parent, jobj);
    next
}

/// `HSD_JObjAddNext`.
pub unsafe fn HSD_JObjAddNext(jobj: *mut HSD_JObj, next: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjAddNext");
    if jobj.is_null() || next.is_null() {
        return;
    }
    let mut cur;
    if !(*jobj).parent.is_null() {
        cur = (*(*jobj).parent).child;
        (*(*jobj).parent).child = core::ptr::null_mut();
        (*jobj).flags &= !JOBJ_ROOT_MASK;
    } else {
        cur = jobj;
    }
    HSD_JObjReparent(next, (*jobj).parent);
    if !(*next).child.is_null() {
        let mut child = (*next).child;
        while !(*child).next.is_null() {
            child = (*child).next;
        }
        (*child).next = cur;
    } else {
        (*next).child = cur;
    }
    while !cur.is_null() {
        (*cur).parent = next;
        UpdateParentTrspBits(next, cur);
        cur = (*cur).next;
    }
}
