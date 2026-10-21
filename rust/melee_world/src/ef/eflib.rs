//! Night 3 round 3: the effect-list walkers of src/melee/ef/eflib.c that call nothing
//! (efLib_SetFlags, efLib_PauseAll, efLib_ResumeAll). Checked against the C by
//! `tests/eflib_r3_vs_c.rs` (`cref/gen_trivial_r3_ref.sh`).
//!
//! The C walks the global `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_EFFECT_UNK0]` and `[..._UNK1]`
//! lists; the two heads are passed in (the GObj list owner is group 1's, and its object model
//! is not settled; see docs/rust-port/group-2-world.md).
use melee_types::{EF_Effect, HSD_GObj};

/// `HSD_GOBJ_PLINK_EFFECT_UNK0` (ef/forward.h).
pub const HSD_GOBJ_PLINK_EFFECT_UNK0: usize = 11;
/// `HSD_GOBJ_PLINK_EFFECT_UNK1` (ef/forward.h).
pub const HSD_GOBJ_PLINK_EFFECT_UNK1: usize = 12;
/// `EF_STATE_ENTERING_PAUSE` (ef/types.h).
pub const EF_STATE_ENTERING_PAUSE: u8 = 0x01;
/// `EF_STATE_ASYNC` (ef/types.h).
pub const EF_STATE_ASYNC: u8 = 0x80;

/// The two plink heads, in the C's walk order (UNK0 then UNK1).
fn heads(plink_head: &[*mut HSD_GObj]) -> [*mut HSD_GObj; 2] {
    [plink_head[HSD_GOBJ_PLINK_EFFECT_UNK0], plink_head[HSD_GOBJ_PLINK_EFFECT_UNK1]]
}

/// Apply `f` to every effect (`GET_EFFECT(g)`, non-null) whose `parent_gobj` is `gobj`.
///
/// # Safety
/// The lists must be well formed: every GObj on them live, `next`-linked, null-terminated,
/// and `user_data` null or a live `EF_Effect` (the C dereferences them unchecked).
unsafe fn for_children(plink_head: &[*mut HSD_GObj], gobj: *mut HSD_GObj, mut f: impl FnMut(&mut EF_Effect)) {
    for mut cur in heads(plink_head) {
        while !cur.is_null() {
            let effect = (*cur).user_data as *mut EF_Effect;
            if !effect.is_null() && (*effect).parent_gobj == gobj {
                f(&mut *effect);
            }
            cur = (*cur).next;
        }
    }
}

/// efLib_SetFlags: OR `expire_flags` into every child effect's `expire_flags` (s8; the s32
/// argument is truncated as the C's `|=` does).
///
/// # Safety
/// `plink_head` is `HSD_GObjPLinkHead` (at least 13 entries); see [`for_children`].
pub unsafe fn eflib_set_flags(plink_head: &[*mut HSD_GObj], gobj: *mut HSD_GObj, expire_flags: i32) {
    for_children(plink_head, gobj, |e| e.expire_flags = (e.expire_flags as i32 | expire_flags) as i8);
}

/// efLib_PauseAll: keep only the async bit, set ENTERING_PAUSE.
///
/// # Safety
/// As [`eflib_set_flags`].
pub unsafe fn eflib_pause_all(plink_head: &[*mut HSD_GObj], gobj: *mut HSD_GObj) {
    for_children(plink_head, gobj, |e| e.state_flags = (e.state_flags & EF_STATE_ASYNC) | EF_STATE_ENTERING_PAUSE);
}

/// efLib_ResumeAll: keep only the async bit.
///
/// # Safety
/// As [`eflib_set_flags`].
pub unsafe fn eflib_resume_all(plink_head: &[*mut HSD_GObj], gobj: *mut HSD_GObj) {
    for_children(plink_head, gobj, |e| e.state_flags &= EF_STATE_ASYNC);
}
