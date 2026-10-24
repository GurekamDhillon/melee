//! src/melee/ft/kinds/ftCommon/ftCo_0A01.c (the CPU controller): the CPU's stick / trigger / button
//! reads, `ftCo_IsCpuControlled`, and the hurtbox extent probe `ftCo_800A0DA4`.
//!
//! Already ported elsewhere: `ftCo_800A101C` (`crate::ftcpu`), `ftCo_800B3E04` (`tu::ftcmdscript`).
//! The CPU AI proper (`ftCo_800B33B0`, `ftCo_800B2AFC`, `ftCo_800B2790`, `ftCo_800B0AF4`, the body of
//! `ftCo_800B3900`) is not ported.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use crate::types::*;

type F = *mut Fighter;

/// Calls out of the TU. `hook::<name>` overrides; unset = the port.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        Player_8003248C(slot: i32, sub: i32) -> Gm_PKind = |slot: i32, sub: i32| crate::tu::player::Player_8003248C(slot, sub);
        lbColl_800083C4(h: *mut HurtCapsule) = |h: *mut HurtCapsule| unsafe { melee_runtime::tu::lbcollision::lbColl_800083C4(h as _) };
    }
}

/// `convertStickAxis` (static inline): `val / 127.0f` above zero, `val / 128.0f` otherwise, clamped to +-1.
#[inline]
fn convert_stick_axis(val: i8) -> f32 {
    let v = val as i32;
    let ret = if v > 0 { v as f32 / 127.0f32 } else { v as f32 / 128.0f32 };
    if ret > 1.0 { 1.0 } else if ret < -1.0 { -1.0 } else { ret }
}

/// `ftCo_GetCpuLStickX`.
pub fn ftCo_GetCpuLStickX(fp: &Fighter) -> f32 { convert_stick_axis(fp.cpu.lstick.x) }
/// `ftCo_GetCpuLStickY`.
pub fn ftCo_GetCpuLStickY(fp: &Fighter) -> f32 { convert_stick_axis(fp.cpu.lstick.y) }
/// `ftCo_GetCpuCStickX`.
pub fn ftCo_GetCpuCStickX(fp: &Fighter) -> f32 { convert_stick_axis(fp.cpu.cstick.x) }
/// `ftCo_GetCpuCStickY`.
pub fn ftCo_GetCpuCStickY(fp: &Fighter) -> f32 { convert_stick_axis(fp.cpu.cstick.y) }

/// `ftCo_GetCpuLTrigger`: `fp->cpu.ltrigger / 255.0` is a double division, narrowed once.
pub fn ftCo_GetCpuLTrigger(fp: &Fighter) -> f32 {
    let ret = (fp.cpu.ltrigger as f64 / 255.0) as f32;
    if ret as f64 > 1.0 { 1.0 } else { ret }
}
/// `ftCo_GetCpuRTrigger`.
pub fn ftCo_GetCpuRTrigger(fp: &Fighter) -> f32 {
    let ret = (fp.cpu.rtrigger as f64 / 255.0) as f32;
    if ret as f64 > 1.0 { 1.0 } else { ret }
}
/// `ftCo_GetCpuButtons`.
pub fn ftCo_GetCpuButtons(fp: &Fighter) -> u32 { fp.cpu.buttons as u32 }

/// `ftCo_IsCpuControlled`: a CPU slot whose CPU kind is not 5.
pub fn ftCo_IsCpuControlled(fp: &Fighter) -> bool {
    if ext::Player_8003248C(fp.player_id as i32, fp.is_sub_fighter() as i32) != Gm_PKind_Cpu as _ {
        return false;
    }
    if fp.cpu.kind as i32 == 5 {
        return false;
    }
    true
}

/// `ftCo_800A0DA4`: the extent of the fighter's hurtboxes relative to its position, stored in the CPU
/// block (x55C/x560 front/back by facing, x564 their midpoint, x568 height).
pub unsafe fn ftCo_800A0DA4(fp: &mut Fighter) {
    let mut var_f31: f32 = 0.0;
    let mut var_f30: f32 = 0.0;
    let mut var_f29: f32 = 0.0;
    let mut i = 0usize;
    while i < fp.hurt_capsules_len as usize {
        let hurt = &mut fp.hurt_capsules[i];
        ext::lbColl_800083C4(&mut hurt.capsule as *mut HurtCapsule);

        let mut dx = hurt.capsule.a_pos.x - fp.cur_pos.x;
        let dy = hurt.capsule.a_pos.y - fp.cur_pos.y;
        let scale = hurt.capsule.scale * fp.x34_scale.y;

        if var_f31 > dx - scale { var_f31 = dx - scale; }
        if var_f30 < dx + scale { var_f30 = dx + scale; }
        if var_f29 < dy + scale { var_f29 = dy + scale; }

        dx = hurt.capsule.b_pos.x - fp.cur_pos.x;
        let dy = hurt.capsule.b_pos.y - fp.cur_pos.y;

        if var_f31 > dx - scale { var_f31 = dx - scale; }
        if var_f30 < dx + scale { var_f30 = dx + scale; }
        if var_f29 < dy + scale { var_f29 = dy + scale; }
        i += 1;
    }
    let facing_pos = fp.facing_dir as f64 > 0.0;
    let cpu = &mut fp.cpu;
    if facing_pos {
        cpu.x55C = var_f30;
        cpu.x560 = -var_f31;
    } else {
        cpu.x55C = -var_f31;
        cpu.x560 = var_f30;
    }
    // `0.5 * (a + b)`: the sum is a float, the product with the double 0.5 is exact, narrowed on store.
    cpu.x564 = (0.5f64 * (cpu.x55C + cpu.x560) as f64) as f32;
    cpu.x568 = var_f29;
}
