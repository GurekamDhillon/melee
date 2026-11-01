//! src/melee/ft/kinds/ftCommon/ftCo_0A01.c (the CPU controller): the CPU's stick / trigger / button
//! reads, `ftCo_IsCpuControlled`, and the hurtbox extent probe `ftCo_800A0DA4`.
//!
//! Already ported elsewhere: `ftCo_800A101C` (`crate::ftcpu`), `ftCo_800B3E04` (`tu::ftcmdscript`).
//! The CPU AI proper (`ftCo_800B33B0`, `ftCo_800B2AFC`, `ftCo_800B2790`, `ftCo_800B0AF4`, the body of
//! `ftCo_800B3900`) is not ported.
#![allow(non_snake_case, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::tu::ftcpuattack::ftco_800b9704;
use melee_types::{Fighter, FighterKind, Ft_Kind_Donkey, Ft_Kind_GKoops, Ft_Kind_Koopa, Ft_Kind_Nana};
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
pub fn ftCo_GetCpuLStickX(fp: &Fighter) -> f32 { melee_core::ct!("ftCo_GetCpuLStickX"); convert_stick_axis(fp.cpu.lstick.x) }
/// `ftCo_GetCpuLStickY`.
pub fn ftCo_GetCpuLStickY(fp: &Fighter) -> f32 { melee_core::ct!("ftCo_GetCpuLStickY"); convert_stick_axis(fp.cpu.lstick.y) }
/// `ftCo_GetCpuCStickX`.
pub fn ftCo_GetCpuCStickX(fp: &Fighter) -> f32 { melee_core::ct!("ftCo_GetCpuCStickX"); convert_stick_axis(fp.cpu.cstick.x) }
/// `ftCo_GetCpuCStickY`.
pub fn ftCo_GetCpuCStickY(fp: &Fighter) -> f32 { melee_core::ct!("ftCo_GetCpuCStickY"); convert_stick_axis(fp.cpu.cstick.y) }

/// `ftCo_GetCpuLTrigger`: `fp->cpu.ltrigger / 255.0` is a double division, narrowed once.
pub fn ftCo_GetCpuLTrigger(fp: &Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuLTrigger");
    let ret = (fp.cpu.ltrigger as f64 / 255.0) as f32;
    if ret as f64 > 1.0 { 1.0 } else { ret }
}
/// `ftCo_GetCpuRTrigger`.
pub fn ftCo_GetCpuRTrigger(fp: &Fighter) -> f32 {
    melee_core::ct!("ftCo_GetCpuRTrigger");
    let ret = (fp.cpu.rtrigger as f64 / 255.0) as f32;
    if ret as f64 > 1.0 { 1.0 } else { ret }
}
/// `ftCo_GetCpuButtons`.
pub fn ftCo_GetCpuButtons(fp: &Fighter) -> u32 { melee_core::ct!("ftCo_GetCpuButtons"); fp.cpu.buttons as u32 }

/// `ftCo_IsCpuControlled`: a CPU slot whose CPU kind is not 5.
pub fn ftCo_IsCpuControlled(fp: &Fighter) -> bool {
    melee_core::ct!("ftCo_IsCpuControlled");
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
    melee_core::ct!("ftCo_800A0DA4");
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

// ---- folded from ftcpu.rs (consolidation steps 21/22) ----
/// `ftCo_800A101C(fp, kind, level, x14)`. `rand` is `HSD_Randf` (called twice, in C order:
/// once for `x7C`/`x80`, once inside `ftCo_800B9704`). `nana_partner` is `ftCo_800A589C(fp)`'s
/// position and facing (only read for Nana).
pub fn ftco_800a101c(
    fp: &mut Fighter,
    kind: i32,
    level: i32,
    x14: i32,
    floor: Option<(f32, f32)>,
    nana_partner: Option<(melee_types::Vec, f32)>,
    rand: &mut dyn FnMut() -> f32,
) {
    melee_core::ct!("ftco_800a101c");
    let ai_kind: FighterKind = fp.kind;
    let cur_pos = fp.cur_pos;
    {
        let c = &mut fp.cpu;
        if ai_kind == Ft_Kind_Nana {
            c.set_xF9_b2(1);
            c.kind = 6;
            c.x3C = 15.0;
        } else {
            c.kind = kind as u32;
            c.x3C = 40.0;
        }
        c.level = level;
        c.x14 = x14;
        match c.kind {
            1 | 25 => {
                c.x18 = 0xC;
                c.x1C = 0xC;
                c.x20 = 0;
            }
            3 => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xB;
            }
            15 => {
                c.x18 = 0;
                c.x1C = 0;
                c.x20 = 0;
            }
            _ => {
                c.x18 = 1;
                c.x1C = 1;
                c.x20 = 0xA;
            }
        }
        c.x24 = 0x12C;
        c.x28 = 0;
        c.x2C = 5;
        c.x30 = 0;
        c.set_xF9_b0(0);
        c.x38 = 5.0;
        c.x40 = 50.0;
        c.x44 = core::ptr::null_mut();
        c.x48 = core::ptr::null_mut();
        c.x50 = 0;
        c.x98 = cur_pos;
        let (fx, fy) = floor.unwrap_or((cur_pos.x, cur_pos.y));
        c.x54.x = fx;
        c.x64.x = fx;
        c.x54.y = fy;
        c.x64.y = fy;
        c.x60 = 0;
        c.x74.y = 0.0;
        c.x74.x = 0.0;
        c.x6C.y = 0.0;
        c.x6C.x = 0.0;
        c.set_xF8_b0(0);
        c.set_xF8_b12(0);
    }
    // `temp_r6 = 10.0 * HSD_Randf()`: a double product truncated to int.
    let r = rand();
    let t = (10.0f64 * r as f64) as i32;
    {
        let c = &mut fp.cpu;
        c.x7C = t;
        c.x80 = t;
        c.x84 = 0;
        c.x90 = 0;
        c.x88 = 0;
        c.x8C = 0;
        c.csP = core::ptr::null_mut();
        c.command_duration = 0;
        c.write_pos = c.buffer.as_mut_ptr();
        c.xA4 = 0;
        c.xC8 = 0;
        c.xEC = 0;
        c.set_xF8_b5(0);
        c.set_xF8_b6(0);
        c.set_xF8_b7(0);
        c.set_xF9_b1(0);
        c.buttons = 0;
        c.lstick.x = 0;
        c.lstick.y = 0;
        c.cstick.x = 0;
        c.cstick.y = 0;
        c.rtrigger = 0;
        c.ltrigger = 0;
        c.set_xF9_b2(0);
        c.set_xF9_b3(0);
        c.set_xF9_b4(0);
        c.set_xF9_b6(0);
        c.set_xF9_b5(0);
        c.set_xF9_b7(0);
        c.set_xFA_b7(0);
        c.set_xFB_b0(0);
        c.x94 = 0;
        c.set_xFA_b1(0);
        c.set_xFA_b2(0);
        c.set_xFA_b34(1);
        c.set_xFA_b5(0);
        c.set_xFA_b6(0);
        c.x55C = 1.0;
        c.x560 = 1.0;
        c.x564 = 0.5 * (c.x55C + c.x560);
        c.x568 = 2.0;
        c.x444 = &mut c.xFC[5];
        c.x448 = c.xFC.as_mut_ptr();
        for e in c.xFC.iter_mut() {
            e.x0 = 0;
            e.lstick.x = 0;
            e.lstick.y = 0;
            e.x4 = 0;
            e.x5 = 0;
            match (ai_kind == Ft_Kind_Nana, nana_partner) {
                (true, Some((pos, facing))) => {
                    e.cur_pos = pos;
                    e.facing_dir = facing;
                }
                (true, None) => panic!("ftCo_800A101C on Nana needs ftCo_800A589C's partner (TODO(group3 nana))"),
                _ => {
                    e.cur_pos.x = 0.0;
                    e.cur_pos.y = 0.0;
                    e.cur_pos.z = 0.0;
                    e.facing_dir = 1.0;
                }
            }
        }
    }
    let g = fp.co_attrs.gravity;
    let v = fp.co_attrs.jump_v_initial_velocity * fp.co_attrs.air_jump_v_multiplier;
    let zero_g = g < 0.00001 && g > -0.00001;
    fp.cpu.x558 = if zero_g { 10.0 } else { ((v * v) as f64 / (2.0 * g as f64)) as f32 };
    fp.cpu.half_width = 10.0;
    fp.cpu.half_height = 10.0;
    fp.cpu.x570 = 1.0;
    let r = rand();
    ftco_800b9704(fp, r);
    fp.cpu.x56C = match ai_kind {
        k if k == Ft_Kind_Donkey || k == Ft_Kind_Koopa => 8.5,
        k if k == Ft_Kind_GKoops => 17.0,
        k if k == Ft_Kind_Nana => 1.0,
        _ => 3.5,
    };
}
