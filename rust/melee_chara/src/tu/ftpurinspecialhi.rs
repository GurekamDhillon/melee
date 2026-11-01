//! src/melee/ft/kinds/ftPurin/ftpurinspecialhi.c: Jigglypuff's up-B (Sing). No float arithmetic,
//! so no fused sites (tools/rust-port/fma_sites.py). Includes the TARGET_PC
//! `ScriptGame_HitRuleSpecial` call. Smoke-tested by tests/tu_ftpurinspecialhi.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.


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
    crate::port_hook!(GM_8016B1D8, gm_8016B1D8, crate::adapt::gm_8016B1D8, () -> i32);
    pub static mut GRSTADIUM_801D4FF8: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grStadium_801D4FF8(player_id: i32) -> i32 {
        match unsafe { GRSTADIUM_801D4FF8 } {
            Some(f) => f(player_id),
            None => { unsafe { melee_world::tu::grpstadium::grStadium_801D4FF8(player_id as _) as _ } },
        }
    }
    crate::port_hook!(SCRIPTGAME_HITRULESPECIAL, ScriptGame_HitRuleSpecial, crate::adapt::ScriptGame_HitRuleSpecial, (hit: *mut HitCapsule));
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftPurin_SpecialHi_SetVars` (static inline).
unsafe fn SetVars(gobj: G) {
    let fp = fpp(gobj);
    (*fp).cmd_vars[0] = 0;
    (*fp).accessory4_cb = Some(crate::chars::cb::ftpurin__ftPr_Init_8013C94C);
    if ext::gm_8016B1D8() != 0 && ext::grStadium_801D4FF8((*fp).player_id as i32) != 0 {
        (*fp).mv.pr.specialhi.x0 = 1;
    } else {
        (*fp).mv.pr.specialhi.x0 = 0;
    }
}

/// `ftPurin_SpecialHi_SetActionFromFacingDirection` (static inline).
unsafe fn SetActionFromFacingDirection(gobj: G, left_id: u32, right_id: u32) {
    let fighter = fpp(gobj);
    if -1.0 == (*fighter).facing_dir {
        st::Fighter_ChangeMotionState(gobj, left_id as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    } else {
        st::Fighter_ChangeMotionState(gobj, right_id as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    }
}

pub fn ftPr_SpecialHi_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_Enter");
    unsafe {
        SetActionFromFacingDirection(gobj, 365, 367);
        st::ftAnim_8006EBA4(gobj);
        SetVars(gobj);
    }
}

pub fn ftPr_SpecialAirHi_Enter(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirHi_Enter");
    unsafe {
        SetActionFromFacingDirection(gobj, 366, 368);
        st::ftAnim_8006EBA4(gobj);
        SetVars(gobj);
    }
}

/// The sleep-hit rule shared by both Anim states.
unsafe fn sleep_hit_rule(fighter: *mut Fighter) {
    if (*fighter).mv.pr.specialhi.x0 != 0
        && (*fighter).x914[0].x43_b2() == 0
        && (*fighter).x914[0].state != HitCapsule_Disabled
    {
        (*fighter).x914[0].set_x43_b2(1);
        ext::ScriptGame_HitRuleSpecial(&mut (*fighter).x914[0]);
        (*fighter).x914[0].element = HitElement_Sleep;
    }
}

pub fn ftPr_SpecialHi_Anim(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_Anim");
    unsafe {
        let fighter = fpp(fighter_gobj);
        sleep_hit_rule(fighter);
        if st::ftAnim_IsFramesRemaining(fighter_gobj) == 0 {
            st::ft_8008A2BC(fighter_gobj);
        }
    }
}

pub fn ftPr_SpecialAirHi_Anim(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialAirHi_Anim");
    unsafe {
        let fighter = fpp(fighter_gobj);
        sleep_hit_rule(fighter);
        if st::ftAnim_IsFramesRemaining(fighter_gobj) == 0 {
            st::ftCo_Fall_Enter(fighter_gobj);
        }
    }
}

pub fn ftPr_SpecialHi_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialHi_IASA");}
pub fn ftPr_SpecialAirHi_IASA(_gobj: G) { melee_core::ct!("ftPr_SpecialAirHi_IASA");}

pub fn ftPr_SpecialHi_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftPr_SpecialHi_8013CD34(gobj);
    }
}

pub fn ftPr_SpecialAirHi_Coll(gobj: G) {
    melee_core::ct!("ftPr_SpecialAirHi_Coll");
    if st::ft_CheckGroundAndLedge(gobj, 0) != 0 {
        ftPr_SpecialHi_8013CDD8(gobj);
        return;
    }
    if st::ftCliffCommon_80081298(gobj) != 0 {
        return;
    }
}

pub fn ftPr_SpecialHi_8013CD34(gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_8013CD34");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        if -1.0 == (*fp).facing_dir {
            st::Fighter_ChangeMotionState(
                gobj,
                ftPr_MS_SpecialAirHiL as i32,
                0x0C4C508A,
                (*fp).cur_anim_frame,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        } else {
            st::Fighter_ChangeMotionState(
                gobj,
                ftPr_MS_SpecialAirHiR as i32,
                0x0C4C508A,
                (*fp).cur_anim_frame,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        }
        (*fp).accessory4_cb = Some(crate::chars::cb::ftpurin__ftPr_Init_8013C94C);
    }
}

pub fn ftPr_SpecialHi_8013CDD8(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_8013CDD8");
    unsafe {
        let fighter = fpp(fighter_gobj);
        st::ftCommon_8007D7FC(&mut *fighter);
        if -1.0 == (*fighter).facing_dir {
            st::Fighter_ChangeMotionState(
                fighter_gobj,
                ftPr_MS_SpecialHiL as i32,
                0x0C4C508A,
                (*fighter).cur_anim_frame,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        } else {
            st::Fighter_ChangeMotionState(
                fighter_gobj,
                ftPr_MS_SpecialHiR as i32,
                0x0C4C508A,
                (*fighter).cur_anim_frame,
                1.0,
                0.0,
                core::ptr::null_mut(),
            );
        }
        (*fighter).accessory4_cb = Some(crate::chars::cb::ftpurin__ftPr_Init_8013C94C);
    }
}

pub fn ftPr_SpecialHi_8013CE7C(fighter_gobj: G) {
    melee_core::ct!("ftPr_SpecialHi_8013CE7C");
    unsafe {
        let fighter = fpp(fighter_gobj);
        (*fighter).accessory4_cb = None;
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftPr_SpecialHi_Phys` (src/melee/ft/kinds/ftPurin/ftpurinspecialhi.c).
pub fn ftPr_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialHi_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftPr_SpecialAirHi_Phys` (src/melee/ft/kinds/ftPurin/ftpurinspecialhi.c).
pub fn ftPr_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirHi_Phys");
    st::ft_80084EEC(gobj);
}
