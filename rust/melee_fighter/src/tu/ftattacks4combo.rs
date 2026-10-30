//! Native port of src/melee/ft/ftattacks4combo.c (Link / Young Link second forward-smash hit).
//! No float arithmetic; smoke-tested by tests/tu_ftattacks4combo.rs.
#![allow(non_snake_case)]

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use super::G;
    pub fn Fighter_ChangeMotionState(_g: G, _msid: i32, _flags: i32, _start: f32, _speed: f32, _blend: f32, _cb: *mut core::ffi::c_void) {
        todo!("Fighter_ChangeMotionState (ft/fighter.c)")
    }
    pub fn ftAnim_8006EBA4(_g: G) { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(_g as _); } }
    pub fn ftCo_AttackS4_Anim(_g: G) { unsafe { let _ = crate::tu::ftCo_AttackS4::ftCo_AttackS4_Anim(_g as _); } }
    pub fn ftCo_AttackS4_Phys(_g: G) { unsafe { let _ = crate::tu::ftCo_AttackS4::ftCo_AttackS4_Phys(_g as _); } }
    pub fn ftCo_AttackS4_Coll(_g: G) { unsafe { let _ = crate::tu::ftCo_AttackS4::ftCo_AttackS4_Coll(_g as _); } }
    pub fn ftCo_Wait_IASA(_g: G) { unsafe { let _ = crate::tu::ftCo_Wait::ftCo_Wait_IASA(_g as _); } }
}

/// `ftCo_800CECE8`: A pressed during the combo window -> second hit.
pub fn ftCo_800CECE8(g: G) -> bool {
    melee_core::ct!("ftCo_800CECE8");
    let f = fp(g);
    if f.cmd_vars[0] != 0 && (f.input.pressed_buttons & HSD_PAD_A as u32) != 0 {
        ftCo_800CED30(g);
        return true;
    }
    false
}

/// `ftCo_800CED30`: only Link and Young Link have AttackS42 (else `__assert` at line 0x36).
pub fn ftCo_800CED30(g: G) {
    melee_core::ct!("ftCo_800CED30");
    let kind = fp(g).kind;
    if kind != Ft_Kind_Link && kind != Ft_Kind_CLink {
        panic!("ftattacks4combo.c:54: don't have smash42 motion!!!");
    }
    fp(g).set_allow_interrupt(0);
    ext::Fighter_ChangeMotionState(g, ftLk_MS_AttackS42 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_AttackS42_Anim(g: G) { melee_core::ct!("ftCo_AttackS42_Anim"); ext::ftCo_AttackS4_Anim(g) }

pub fn ftCo_AttackS42_IASA(g: G) {
    melee_core::ct!("ftCo_AttackS42_IASA");
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackS42_Phys(g: G) { melee_core::ct!("ftCo_AttackS42_Phys"); ext::ftCo_AttackS4_Phys(g) }

pub fn ftCo_AttackS42_Coll(g: G) { melee_core::ct!("ftCo_AttackS42_Coll"); ext::ftCo_AttackS4_Coll(g) }
