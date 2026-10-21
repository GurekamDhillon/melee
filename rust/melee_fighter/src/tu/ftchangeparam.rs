//! src/melee/ft/ftchangeparam.c: fighter attribute scaling (size / bunny hood / metal / x2229_b1)
//! and the attribute reload. One fused site (`fma_sites.py`): ftCo_CalcYScaledKnockback's
//! `(scale - 1) * a0 * a2 + a0` -> `fmadds`. Globals owned by the fighter loader and calls out
//! of the TU are `ext` hooks/pointers that panic while unset.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::types::*;
use melee_core::fp::fmadds;

pub mod ext {
    use crate::types::*;
    /// TODO(move-to-core ftdata) `Fighter_804D6524` (PlCo pData: size-scaling exponents).
    pub static mut Fighter_804D6524: *const Fighter_804D6524_t = core::ptr::null();
    /// TODO(move-to-core ftdata) `Fighter_804D651C` (metal multipliers).
    pub static mut Fighter_804D651C: *const Fighter_804D651C_t = core::ptr::null();
    /// TODO(move-to-core ftdata) `Fighter_804D6518` (x2229_b1 multipliers).
    pub static mut Fighter_804D6518: *const Fighter_804D6518_t = core::ptr::null();
    /// TODO(pc/geno) `Geno_ApplyAttrs(fp)` (TARGET_PC).
    pub static mut Geno_ApplyAttrs: Option<fn(&mut Fighter)> = None;
    /// TODO(pc mex) `Mex_OnReapplyAttrDispatch(kind, gobj, ftKindCalcIndiviParamTable[kind])`;
    /// the hook receives the kind and the gobj and is responsible for the vanilla call.
    pub static mut Mex_OnReapplyAttrDispatch: Option<fn(i32, *mut HSD_GObj)> = None;
    /// TODO(group3 ftdata) `ftKindCalcIndiviParamTable[kind] != NULL` (the assert's check).
    pub static mut ftKindCalcIndiviParamTable_present: Option<fn(i32) -> bool> = None;

    pub fn d6524() -> &'static Fighter_804D6524_t {
        let p = unsafe { Fighter_804D6524 };
        assert!(!p.is_null(), "TODO Fighter_804D6524 not loaded");
        unsafe { &*p }
    }
    pub fn d651c() -> &'static Fighter_804D651C_t {
        let p = unsafe { Fighter_804D651C };
        assert!(!p.is_null(), "TODO Fighter_804D651C not loaded");
        unsafe { &*p }
    }
    pub fn d6518() -> &'static Fighter_804D6518_t {
        let p = unsafe { Fighter_804D6518 };
        assert!(!p.is_null(), "TODO Fighter_804D6518 not loaded");
        unsafe { &*p }
    }
}

/// `ftCo_CalcYScaledKnockback`.
pub fn ftco_calc_y_scaled_knockback(arg0: f32, scale: f32, arg2: f32) -> f32 {
    assert!(scale != 0.0, "ftchangeparam.c:30 scale != 0.0F");
    if arg2 == 0.0 {
        return arg0;
    }
    if arg2 < 0.0 {
        return arg0 / ftco_calc_y_scaled_knockback(1.0, scale, -arg2);
    }
    if scale >= 1.0 || arg2 <= 1.0 {
        return fmadds((scale - 1.0) * arg0, arg2, arg0);
    }
    arg0 * scale / arg2
}

/// `ftCo_800CF6E8`: size-scale every scalable attribute.
pub fn ftco_800cf6e8(a: &mut ftCo_DatAttrs, scale: f32) {
    if scale == 1.0 {
        return;
    }
    let k = ext::d6524();
    let s = |v: f32, e: f32| ftco_calc_y_scaled_knockback(v, scale, e);
    a.slow_walk_max = s(a.slow_walk_max, k.x10);
    a.mid_walk_point = s(a.mid_walk_point, k.x14);
    a.fast_walk_min = s(a.fast_walk_min, k.x18);
    a.dash_max_velocity = s(a.dash_max_velocity, k.x1C);
    a.run_animation_scaling = s(a.run_animation_scaling, k.x20);
    a.jump_startup_time = s(a.jump_startup_time, k.x24);
    a.jump_v_initial_velocity = s(a.jump_v_initial_velocity, k.x28);
    a.hop_v_initial_velocity = s(a.hop_v_initial_velocity, k.x2C);
    a.gravity = s(a.gravity, k.x30);
    a.terminal_velocity = s(a.terminal_velocity, k.x34);
    a.air_drift_stick_mul = s(a.air_drift_stick_mul, k.x38);
    a.aerial_drift_base = s(a.aerial_drift_base, k.x3C);
    a.air_drift_max = s(a.air_drift_max, k.x40);
    a.fast_fall_velocity = s(a.fast_fall_velocity, k.x44);
    a.weight = s(a.weight, k.x48);
    a.shield_break_initial_velocity = s(a.shield_break_initial_velocity, k.x4C);
    a.ledge_jump_horizontal_velocity = s(a.ledge_jump_horizontal_velocity, k.x50);
    a.ledge_jump_vertical_velocity = s(a.ledge_jump_vertical_velocity, k.x54);
    a.item_throw_velocity_multiplier = s(a.item_throw_velocity_multiplier, k.x58);
    a.heavy_throw_velocity_multiplier = s(a.heavy_throw_velocity_multiplier, k.x5C);
    a.xBC.size = s(a.xBC.size, k.x60);
    a.xDC = s(a.xDC, k.x64);
    // C: `cur = &attr->xDC; cur[1] = ...` (the next field).
    a.kirby_b_star_damage = s(a.kirby_b_star_damage, k.x68);
    a.normal_landing_lag = s(a.normal_landing_lag, k.x6C);
    a.landingairn_lag = s(a.landingairn_lag, k.x70);
    a.landingairf_lag = s(a.landingairf_lag, k.x74);
    a.landingairb_lag = s(a.landingairb_lag, k.x78);
    a.landingairhi_lag = s(a.landingairhi_lag, k.x7C);
    a.landingairlw_lag = s(a.landingairlw_lag, k.x80);
    a.name_tag_height = s(a.name_tag_height, k.x84);
    a.screw_attack_launch_velocity = s(a.screw_attack_launch_velocity, k.x88);
    a.damageicejump_vel_y = s(a.damageicejump_vel_y, k.x8C);
    a.damageicejump_vel_x_mult = s(a.damageicejump_vel_x_mult, k.x90);
    a.x170.x = s(a.x170.x, k.x94);
    a.x170.y = s(a.x170.y, k.x94);
    a.x170.z = s(a.x170.z, k.x94);
    a.x17C = s(a.x17C, k.x98);
}

#[inline(always)]
fn fp<'a>(g: *mut HSD_GObj) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCo_800D0CBC`: scale the x2D0 table's x14 entries.
pub fn ftco_800d0cbc(g: *mut HSD_GObj) {
    let f = fp(g);
    let t = f.x2D0;
    let count = unsafe { (*t).x28 };
    let mut scale: f32 = 1.0;
    if scale != f.x34_scale.y {
        scale *= ftco_calc_y_scaled_knockback(1.0, f.x34_scale.y, ext::d6524().x28);
    }
    if !f.x197C.is_null() {
        scale *= crate::hooks::fighter_804d6520().x14;
    }
    if f.is_metal() != 0 {
        scale *= ext::d651c().x4;
    }
    let mut i = 0;
    while i < count {
        // C indexes x14[i] with no bound (5 entries, then x28).
        unsafe {
            let p = (core::ptr::addr_of_mut!((*t).x14) as *mut f32).add(i as usize);
            *p *= scale;
        }
        i += 1;
    }
}

/// `ftCo_800D0EC8`: negated size-scaled common x310.
pub fn ftco_800d0ec8(f: &Fighter) -> f32 {
    -ftco_calc_y_scaled_knockback(crate::hooks::common_data().x310, f.x34_scale.y, ext::d6524().x30)
}

fn reload(f: &mut Fighter) {
    unsafe {
        f.co_attrs = *(*f.ft_data).x0;
    }
    (unsafe { ext::Geno_ApplyAttrs }.expect("TODO(pc/geno) Geno_ApplyAttrs"))(f);
    unsafe {
        f.x294_itPickup = *(*f.ft_data).x40;
        f.x2C4 = *(*f.ft_data).x50;
    }
}

/// `ftCo_800D0FA0`: reload attributes from the DAT (no modifiers).
pub fn ftco_800d0fa0(g: *mut HSD_GObj) {
    reload(fp(g));
}

/// `ftCo_800D105C`: reload attributes, then apply size / bunny hood / metal / x2229_b1 and the
/// per-kind hook.
pub fn ftco_800d105c(g: *mut HSD_GObj) {
    let f = fp(g);
    reload(f);
    if f.x34_scale.y != 1.0 {
        let sy = f.x34_scale.y;
        ftco_800cf6e8(&mut f.co_attrs, sy);
        let t = f.x34_scale.y;
        let p = &mut f.x294_itPickup;
        for q in [&mut p.gr_light_offset, &mut p.gr_heavy_offset, &mut p.air_light_offset] {
            q.x *= t;
            q.y *= t;
            q.z *= t;
            q.w *= t;
        }
        let sy = f.x34_scale.y;
        f.x2C4.x *= sy;
        f.x2C4.y *= sy;
    }
    if !f.x197C.is_null() {
        let b = crate::hooks::fighter_804d6520();
        let a = &mut f.co_attrs;
        a.dash_accel_mul *= b.x4;
        a.dash_accel_base *= b.x8;
        a.dash_max_velocity *= b.xC;
        a.jump_h_initial_velocity *= b.x10;
        a.jump_v_initial_velocity *= b.x14;
        a.jump_h_max_velocity *= b.x1C;
        a.hop_v_initial_velocity *= b.x18;
        a.gravity *= b.x20;
        a.terminal_velocity *= b.x24;
        a.fast_fall_velocity *= b.x28;
        a.ledge_jump_horizontal_velocity *= b.x2C;
        a.ledge_jump_vertical_velocity *= b.x30;
        a.wall_jump_horizontal_velocity *= b.x34;
        a.wall_jump_vertical_velocity *= b.x38;
    }
    if f.is_metal() != 0 {
        let m = ext::d651c();
        let a = &mut f.co_attrs;
        a.jump_v_initial_velocity *= m.x4;
        a.hop_v_initial_velocity *= m.x8;
        a.gravity *= m.xC;
        a.terminal_velocity *= m.x10;
        a.fast_fall_velocity *= m.x14;
        a.weight *= m.x18;
        a.ledge_jump_vertical_velocity *= m.x1C;
        a.wall_jump_vertical_velocity *= m.x20;
    }
    if f.x2229_b1() != 0 {
        let m = ext::d6518();
        f.co_attrs.gravity *= m.x0;
        f.co_attrs.weight *= m.x4;
    }
    let kind = f.kind as i32;
    let present = (unsafe { ext::ftKindCalcIndiviParamTable_present }
        .expect("TODO(group3 ftdata) ftKindCalcIndiviParamTable"))(kind);
    assert!(present, "don't set ftKindCalcIndiviParamTable!!");
    (unsafe { ext::Mex_OnReapplyAttrDispatch }.expect("TODO(pc mex) Mex_OnReapplyAttrDispatch"))(kind, g);
    let f = fp(g);
    if !f.x2D0.is_null() {
        ftco_800d0cbc(g);
    }
    let f = fp(g);
    if f.x2222_b0() != 0 {
        let v = f.x34_scale.y;
        let a = unsafe { &mut *f.x2CC };
        if v != 1.0 {
            a.x14 *= v;
            a.x18 *= f.x34_scale.y;
            a.x1C *= f.x34_scale.y;
        }
    }
}
