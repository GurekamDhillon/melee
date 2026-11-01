//! melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturedamagecrazyhand.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_GrabUnk1_8015B670` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturedamagecrazyhand.c).
pub fn ftCh_GrabUnk1_8015B670(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015B670");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 337 /* 0x151 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).set_invisible(1);
        (*fp).accessory1_cb = Some(crate::chars::cb::ftCo_CapturePulled__ftCo_800DB464);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
        (*fp).set_x2220_b3(1);
        st::ftAnim_8006EBA4(gobj);
    }
}
