//! script_game.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj, ScriptCapsMovement, HSD_GOBJ_PLINK_FIGHTER};

// ---- folded from script_caps.rs (consolidation steps 21/22) ----
/// `script_gobj_live_fp(gobj)`: `gobj` is on the fighter player link.
pub unsafe extern "C" fn script_gobj_live_fp(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("script_gobj_live_fp");
    // The C returns 0 while `HSD_GObjPLinkHead` is NULL; an unset hook is that case here.
    if gobj.is_null() || crate::hooks::PLINK_HEAD.is_none() {
        return 0;
    }
    let mut cur = crate::hooks::plink_head(HSD_GOBJ_PLINK_FIGHTER as i32);
    while !cur.is_null() {
        if cur == gobj {
            return 1;
        }
        cur = (*cur).next as *mut HSD_GObj;
    }
    0
}
