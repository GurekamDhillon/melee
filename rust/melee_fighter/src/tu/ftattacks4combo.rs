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
    pub fn ftAnim_8006EBA4(_g: G) { todo!("ftAnim_8006EBA4 (ft/ftanim.c)") }
    pub fn ftCo_AttackS4_Anim(_g: G) { todo!("ftCo_AttackS4_Anim (ftCommon/ftCo_AttackS4.c)") }
    pub fn ftCo_AttackS4_Phys(_g: G) { todo!("ftCo_AttackS4_Phys (ftCommon/ftCo_AttackS4.c)") }
    pub fn ftCo_AttackS4_Coll(_g: G) { todo!("ftCo_AttackS4_Coll (ftCommon/ftCo_AttackS4.c)") }
    pub fn ftCo_Wait_IASA(_g: G) { todo!("ftCo_Wait_IASA (ftCommon/ftCo_Wait.c)") }
}

/// `ftCo_800CECE8`: A pressed during the combo window -> second hit.
pub fn ftCo_800CECE8(g: G) -> bool {
    let f = fp(g);
    if f.cmd_vars[0] != 0 && (f.input.pressed_buttons & HSD_PAD_A as u32) != 0 {
        ftCo_800CED30(g);
        return true;
    }
    false
}

/// `ftCo_800CED30`: only Link and Young Link have AttackS42 (else `__assert` at line 0x36).
pub fn ftCo_800CED30(g: G) {
    let kind = fp(g).kind;
    if kind != Ft_Kind_Link && kind != Ft_Kind_CLink {
        panic!("ftattacks4combo.c:54: don't have smash42 motion!!!");
    }
    fp(g).set_allow_interrupt(0);
    ext::Fighter_ChangeMotionState(g, ftLk_MS_AttackS42 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
}

pub fn ftCo_AttackS42_Anim(g: G) { ext::ftCo_AttackS4_Anim(g) }

pub fn ftCo_AttackS42_IASA(g: G) {
    if fp(g).allow_interrupt() != 0 {
        ext::ftCo_Wait_IASA(g);
    }
}

pub fn ftCo_AttackS42_Phys(g: G) { ext::ftCo_AttackS4_Phys(g) }

pub fn ftCo_AttackS42_Coll(g: G) { ext::ftCo_AttackS4_Coll(g) }
