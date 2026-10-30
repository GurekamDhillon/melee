//! C-named entry points of the staling tail of src/melee/ft/ft_0881.c. The bodies live in
//! `crate::stale` (`clear_attack_id`, `new_attack`, `apply_staling`, `renew_attack_instance`);
//! tests/tu_stale_vs_c.rs checks these names against the C.
#![allow(non_snake_case)]
use crate::types::*;

/// `ft_800890BC`: clear the attack id / instance.
pub fn ft_800890BC(fp: &mut Fighter) { melee_core::ct!("ft_800890BC"); crate::stale::clear_attack_id(fp) }
/// `ft_800890D0`: a new attack instance unless the same (non-1) move is still running.
pub fn ft_800890D0(fp: &mut Fighter, attack_id: i32) { melee_core::ct!("ft_800890D0"); crate::stale::new_attack(fp, attack_id as u32) }
/// `ft_80089228`: the staled damage of `attack_id`.
pub fn ft_80089228(fp: &Fighter, attack_id: i32, arg2: i32, damage: f32) -> f32 { melee_core::ct!("ft_80089228"); crate::stale::apply_staling(fp, attack_id, arg2, damage) }
/// `ft_800892A0`: restart the current attack's instance.
pub fn ft_800892A0(gobj: *mut HSD_GObj) { melee_core::ct!("ft_800892A0"); unsafe { crate::stale::renew_attack_instance(&mut *((*gobj).user_data as *mut Fighter)) } }
