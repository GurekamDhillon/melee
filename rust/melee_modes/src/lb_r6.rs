//! Round 6 (night 3 round 2): lb leaf functions (lbrefract.c, lb_00B0.c), verified bit for bit
//! against the C by `tests/oracle_r6.rs`. TODO(group1): lb leftovers; move into melee_runtime
//! if group 1 takes these TUs.
use melee_core::fp::fmadds;
use melee_types::HSD_JObj;

pub use melee_runtime::tu::lbrefract::my_fmodf;

/// lb_00B0.c `lb_8000B074`: the joint has an AObj that is not AOBJ_NO_ANIM.
pub fn lb_8000B074(jobj: &HSD_JObj) -> bool {
    const AOBJ_NO_ANIM: u32 = 1 << 30;
    // SAFETY: `aobj` is NULL or a live AObj, as in the C.
    match unsafe { jobj.aobj.as_ref() } {
        Some(a) => a.flags & AOBJ_NO_ANIM == 0,
        None => false,
    }
}

/// lb_00B0.c `memzero`: zero `size` bytes (`while (size--)`; size >= 0).
pub fn memzero(mem: &mut [u8], size: isize) {
    for b in &mut mem[..size as usize] {
        *b = 0;
    }
}
