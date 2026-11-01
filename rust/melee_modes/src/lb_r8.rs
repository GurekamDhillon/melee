
//! Round 8 (night 3 round 4): lb leaf functions over nested pointers (lbcollision.c, lb_00B0.c,
//! lbspdisplay.c), verified bit for bit against the C by `tests/oracle_r8.rs`. TODO(group1): lb
//! leftovers; move into melee_runtime if group 1 takes these TUs.
use core::ffi::c_void;

use melee_types::{HitCapsule, HSD_DObj, HSD_JObj};

/// The depth-first walk lb_8000B09C/lb_8000B134 share: true at the first joint whose AObj
/// satisfies `hit`. JOBJ_INSTANCE joints are not descended into.
///
/// # Safety
/// `jobj` is NULL or the root of a well-formed JObj tree (child/next/parent/aobj NULL or live).
pub(crate) unsafe fn jobj_tree_any(mut jobj: *const HSD_JObj, hit: impl Fn(u32) -> bool) -> bool {
    const JOBJ_INSTANCE: u32 = 1 << 12;
    while let Some(j) = jobj.as_ref() {
        if let Some(a) = j.aobj.as_ref() {
            if hit(a.flags) {
                return true;
            }
        }
        if (j.child.is_null() && j.next.is_null()) || j.flags & JOBJ_INSTANCE != 0 {
            let mut cur = j;
            loop {
                match cur.parent.as_ref() {
                    None => {
                        jobj = core::ptr::null();
                        break;
                    }
                    Some(p) if !p.next.is_null() => {
                        jobj = p.next;
                        break;
                    }
                    Some(p) => cur = p,
                }
            }
        } else if !j.child.is_null() {
            jobj = j.child;
        } else {
            jobj = j.next;
        }
    }
    false
}

pub use crate::tu::lbcollision::lbColl_8000ACFC_fp as lbColl_8000ACFC;
pub use crate::tu::lbcollision::lbColl_CopyHitCapsule_fp as lbColl_CopyHitCapsule;
pub use crate::tu::lb_00b0::lb_8000B09C_fp as lb_8000B09C;
pub use crate::tu::lb_00b0::lb_8000B134_fp as lb_8000B134;
pub use crate::tu::lbspdisplay::lb_80011B74_fp as lb_80011B74;
