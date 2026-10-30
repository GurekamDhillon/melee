//! Round 8 (night 3 round 4): lb leaf functions over nested pointers (lbcollision.c, lb_00B0.c,
//! lbspdisplay.c), verified bit for bit against the C by `tests/oracle_r8.rs`. TODO(group1): lb
//! leftovers; move into melee_runtime if group 1 takes these TUs.
use core::ffi::c_void;
use melee_types::{HitCapsule, HSD_DObj, HSD_JObj};

/// lbcollision.c `lbColl_CopyHitCapsule`: copy both victim arrays and x44/x45.
/// The C struct assignment copies each HitVictim's tail padding too (native: 4 bytes after x4),
/// so the copy is bytewise to keep the record bit-identical.
pub fn lbColl_CopyHitCapsule(src: &HitCapsule, dst: &mut HitCapsule) {
    for i in 0..src.victims_1.len() {
        // SAFETY: distinct `&`/`&mut` records, so the elements do not overlap.
        unsafe {
            core::ptr::copy_nonoverlapping(&src.victims_1[i], &mut dst.victims_1[i], 1);
            core::ptr::copy_nonoverlapping(&src.victims_2[i], &mut dst.victims_2[i], 1);
        }
    }
    dst.x44 = src.x44;
    dst.x45 = src.x45;
}

/// lbcollision.c `lbColl_8000ACFC`: `victim` is in `hitbox.victims_1` (compared by address).
pub fn lbColl_8000ACFC(victim: *mut c_void, hitbox: &HitCapsule) -> bool {
    hitbox.victims_1.iter().any(|v| v.victim == victim)
}

/// The depth-first walk lb_8000B09C/lb_8000B134 share: true at the first joint whose AObj
/// satisfies `hit`. JOBJ_INSTANCE joints are not descended into.
///
/// # Safety
/// `jobj` is NULL or the root of a well-formed JObj tree (child/next/parent/aobj NULL or live).
unsafe fn jobj_tree_any(mut jobj: *const HSD_JObj, hit: impl Fn(u32) -> bool) -> bool {
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

pub unsafe fn lb_8000B09C(jobj: *const HSD_JObj) -> bool {
    unsafe { melee_runtime::tu::lb_00B0::lb_8000B09C(jobj as _) }
}

pub unsafe fn lb_8000B134(jobj: *const HSD_JObj) -> bool {
    unsafe { melee_runtime::tu::lb_00B0::lb_8000B134(jobj as _) }
}

pub use melee_runtime::tu::lbspdisplay::lb_80011B74;
