//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandthrow.c. No float math. Smoke-tested by
//! tests/tu_ftcrazyhandthrow.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



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
    pub static mut FTCH_GRABUNK1_8015BC88: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCh_GrabUnk1_8015BC88(gobj: *mut HSD_GObj)  {
                match unsafe { FTCH_GRABUNK1_8015BC88 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj as _); } return; },
                }
            }
}

pub fn ftCh_Init_8015A560(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015A560");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[1] = 0;
        (*fp).x1A5C = melee_fighter::tu::ftbosslib::ftBossLib_GetFighterGObj(Ft_Kind_MasterH);
        st::Fighter_ChangeMotionState(gobj, 0x17B, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftCh_Throw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Throw_Anim");
    use melee_fighter::tu::ftbosslib as bl;
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[1] != 0 {
            bl::ftBossLib_8015C5F8(gobj);
            (*fp).cmd_vars[1] = 0;
        }
        if bl::ftBossLib_8015C2E0() || bl::ftBossLib_8015C358() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_Throw_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Throw_Coll");}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftCh_Throw_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandthrow.c).
pub fn ftCh_Throw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Throw_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Throw_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandthrow.c).
pub fn ftCh_Throw_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Throw_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
