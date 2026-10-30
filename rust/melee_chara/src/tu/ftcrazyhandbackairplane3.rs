//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane3.c. No float math. Smoke-tested by
//! tests/tu_ftcrazyhandbackairplane3.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftcrazyhandbackairplane2::ext;
use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandbackairplane3::ftCh_BackAirplane3_Phys;
pub use crate::chars::r4::ftCrazyHand::ftcrazyhandbackairplane3::ftCh_Init_80159670;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandbackairplane3::ftCh_BackAirplane3_IASA;

pub fn ftCh_BackAirplane3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane3_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ext::ftCh_GrabUnk1_8015B850((*fp).victim_gobj, 0x153);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ext::ftCo_800DE2A8(gobj, victim);
                ext::ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
            (*fp).mv.ch.unk0.x20 = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_BackAirplane3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackAirplane3_Coll");}
