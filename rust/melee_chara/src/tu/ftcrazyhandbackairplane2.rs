//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane2.c. Float math: only `facing_dir_1 *= -1`
//! (exact negation, no fused site). Smoke-tested by tests/tu_ftcrazyhandbackairplane2.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut FTCH_GRABUNK1_8015B850: Option<fn(*mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_GrabUnk1_8015B850(gobj: *mut HSD_GObj, msid: i32)  {
                match unsafe { FTCH_GRABUNK1_8015B850 } {
                    Some(f) => f(gobj, msid),
                    None => { unsafe { crate::tu::ftcrazyhandthrowncrazyhand::ftCh_GrabUnk1_8015B850(gobj as _, msid as _); } return; },
                }
            }
    pub static mut FTCH_GRABUNK1_8015BC88: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_GrabUnk1_8015BC88(gobj: *mut HSD_GObj)  {
                match unsafe { FTCH_GRABUNK1_8015BC88 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj as _); } return; },
                }
            }
    pub static mut FTCO_800DE2A8: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800DE2A8(gobj: *mut HSD_GObj, vic: *mut HSD_GObj)  {
                match unsafe { FTCO_800DE2A8 } {
                    Some(f) => f(gobj, vic),
                    None => { unsafe { melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(gobj as _, vic as _); } return; },
                }
            }
    /// `ftCo_800DE7C0(gobj, victim, is_upward)`: the native melee_fighter port when unset.
    pub static mut FTCO_800DE7C0: Option<fn(*mut HSD_GObj, *mut HSD_GObj, bool)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800DE7C0(gobj: *mut HSD_GObj, victim: *mut HSD_GObj, up: bool) {
        match unsafe { FTCO_800DE7C0 } {
            Some(f) => f(gobj, victim, up),
            None => melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0(gobj, victim, up),
        }
    }
}

pub fn ftCh_BackAirplane2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane2_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ext::ftCh_GrabUnk1_8015B850((*fp).victim_gobj, 0x153);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj;
                let victim_fp = (*victim).user_data as *mut Fighter;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ext::ftCo_800DE2A8(gobj, victim);
                (*victim_fp).dmg.facing_dir_1 *= -1.0;
                ext::ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
            (*fp).mv.ch.unk0.x20 = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_BackAirplane2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_BackAirplane2_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_BackAirplane2_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane2.c).
pub fn ftCh_BackAirplane2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane2_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_BackAirplane2_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane2.c).
pub fn ftCh_BackAirplane2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_BackAirplane2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Init_801594D4` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandbackairplane2.c).
pub fn ftCh_Init_801594D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_801594D4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 368 /* 0x170 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cmd_vars[0] = 0_u32;
    }
}
