//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c. No float arithmetic (copies only; no fused
//! sites). Smoke-tested by tests/tu_ftcrazyhandwalk.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/8/9): re-exported under this TU.





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

pub fn ftCh_WalkWait_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkWait_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ch.unk0.x8 -= 1;
        if (*fp).mv.ch.unk0.x8 == 0 {
            ftCh_Init_80157B58(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, 0x160, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
    }
}

pub fn ftCh_WalkWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkWait_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        st::ft_80085134(gobj);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C010(gobj, (*da).x68);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C190(gobj);
    }
}

pub fn ftCh_WalkLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_WalkLoop_Coll");}
pub fn ftCh_WalkWait_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_WalkWait_Coll");}
pub fn ftCh_WalkShoot_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_WalkShoot_Coll");}

pub fn ftCh_WalkShoot_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkShoot_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_801578E8` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn fn_801578E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801578E8");
    st::Fighter_ChangeMotionState(gobj, 351 /* ftMh_MS_WalkLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_WalkLoop_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkLoop_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Init_801579F4` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_Init_801579F4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_801579F4");
    st::Fighter_ChangeMotionState(gobj, 352 /* ftMh_MS_WalkWait */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftCh_WalkShoot_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkShoot_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkShoot_Phys");
    st::ft_80085134(gobj);
}

/// `ftCh_Init_80157B58` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_Init_80157B58(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80157B58");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
        st::Fighter_ChangeMotionState(gobj, 353 /* ftMh_MS_WalkShoot */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftCh_WalkLoop_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_WalkWait_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkWait_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkWait_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_WalkShoot_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkShoot_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkShoot_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_WalkLoop_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwalk.c).
pub fn ftCh_WalkLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_WalkLoop_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            crate::chars::r2::ftCrazyHand::ftcrazyhandwalk::ftCh_Init_801579F4(gobj);
            (*fp).mv.ch.unk0.x8 = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x64;
        }
    }
}
