//! src/melee/ft/ft_0CE3.c: Fire Flower (FireFlowerShoot / FireFlowerShootAir) states.
//! Float ops: timer `+= 1` / `-= 1` and compares only (no fused site).
#![allow(non_snake_case)]

use super::ft_0cd1::{ext as e, fp, G};
use super::ft_0cdf::{ext as lg, it_8026b594};
use crate::ftcommon2 as c2;
use crate::hooks as h;
use crate::types::*;

const HSD_PAD_A: u32 = 0x100;

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn it_80292EAC(_i: G) { { unsafe { let _ = melee_items::tu::itfflower::it_80292EAC(_i as _); } return; } }
    pub fn it_80292E64(_i: G) { { unsafe { let _ = melee_items::tu::itfflower::it_80292E64(_i as _); } return; } }
    pub fn it_80292EF8(_i: G, _v: &mut Vec3) { { unsafe { let _ = melee_items::tu::itfflower::it_80292EF8(_i as _, _v as _); } return; } }
    pub fn it_80292F14(_i: G, _v: &Vec3, _dir: f32) { unsafe { let _ = melee_items::tu::itfflower::it_80292F14(_i as _, _v as *const _ as *mut _, _dir as _); } }
    /// `lb_8000B1CC(jobj, NULL, out)`.
    pub fn lb_8000B1CC(_j: *mut HSD_JObj, _out: &mut Vec3) { todo!("TODO(group1 lb) lb_8000B1CC") }
    pub fn ft_800892A0(_g: G) { unsafe { let _ = crate::stale::renew_attack_instance(&mut *((*_g).user_data as *mut Fighter)); } }
}

fn reset_vars(f: &mut Fighter) {
    let cd = h::common_data();
    unsafe {
        let t = &mut f.mv.co.itemthrow;
        t.facing_dir = cd.x5A8;
        t.x4 = cd.x5AC;
        t.x8 = 0;
        t.xC = 0;
        t.x10 = 0.0;
        t.x14 = 0;
    }
}

fn set_cbs(f: &mut Fighter) {
    f.accessory4_cb = Some(ftco_800ce650);
    f.take_dmg_cb = Some(ftco_800ce620);
}

/// `ftCo_800CE3E0`: grounded FireFlowerShoot enter.
pub fn ftco_800ce3e0(g: G) {
    melee_core::ct!("ftco_800ce3e0");
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    reset_vars(fp(g));
    e::Fighter_ChangeMotionState(g, ftCo_MS_FireFlowerShoot as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    e::ftAnim_8006EBA4(g);
    set_cbs(fp(g));
}

/// `ftCo_800CE480`: aerial FireFlowerShootAir enter.
pub fn ftco_800ce480(g: G) {
    melee_core::ct!("ftco_800ce480");
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    reset_vars(fp(g));
    e::Fighter_ChangeMotionState(g, ftCo_MS_FireFlowerShootAir as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    e::ftAnim_8006EBA4(g);
    crate::ftcommon::clamp_air_drift(fp(g));
    set_cbs(fp(g));
}

/// `ft_800CE528`: landed.
pub extern "C" fn ft_800ce528(g: G) {
    melee_core::ct!("ft_800ce528");
    let f = fp(g);
    crate::ftcommon3::ftcommon_8007d7fc(f);
    let (a, b) = (f.cur_anim_frame, f.frame_speed_mul);
    e::Fighter_ChangeMotionState(g, ftCo_MS_FireFlowerShoot as i32, 0x0C4C5880, a, b, 0.0, core::ptr::null_mut());
    set_cbs(fp(g));
}

/// `ft_800CE5A0`: left the ground.
pub extern "C" fn ft_800ce5a0(g: G) {
    melee_core::ct!("ft_800ce5a0");
    let f = fp(g);
    c2::ftcommon_8007d5d4(f);
    let (a, b) = (f.cur_anim_frame, f.frame_speed_mul);
    e::Fighter_ChangeMotionState(g, ftCo_MS_FireFlowerShootAir as i32, 0x0C4C5880, a, b, 0.0, core::ptr::null_mut());
    crate::ftcommon::clamp_air_drift(fp(g));
    set_cbs(fp(g));
}

/// `ftCo_800CE620`: take-damage callback.
pub extern "C" fn ftco_800ce620(g: G) {
    melee_core::ct!("ftco_800ce620");
    let it = fp(g).item_gobj;
    if !it.is_null() {
        ext::it_80292EAC(it);
    }
}

fn z() -> Vec3 { Vec3 { x: 0.0, y: 0.0, z: 0.0 } }

/// The flame effect + sfx block (twice in the C).
fn flame_fx(g: G) {
    let f = fp(g);
    if !it_8026b594(f.item_gobj) {
        let sp = 0.0f32;
        let mut p = z();
        ext::lb_8000B1CC(unsafe { (*f.parts).joint }, &mut p);
        let dir = fp(g).facing_dir;
        lg::efSync_Spawn_dir_f(0x3FF, g, &p, &dir, &sp);
        h::play_sfx(fp(g), 0xEE, 0x7F, 0x40);
    } else {
        let (mut a, mut b) = (z(), z());
        ext::it_80292EF8(f.item_gobj, &mut a);
        super::ft_0cdd::ftco_800cde94(fp(g), &a, &mut b);
        lg::efSync_Spawn(0x405, g, &b);
        h::play_sfx(fp(g), 0xEF, 0x7F, 0x40);
    }
}

/// The shot block (twice in the C).
fn shoot(g: G) {
    let f = fp(g);
    let (mut a, mut b) = (z(), z());
    ext::it_80292EF8(f.item_gobj, &mut a);
    super::ft_0cdd::ftco_800cde94(fp(g), &a, &mut b);
    let f = fp(g);
    if !it_8026b594(f.item_gobj) {
        ext::it_80292F14(f.item_gobj, &b, f.facing_dir);
    } else {
        c2::ftcommon_8007ecd4(f, 1);
    }
    let f = fp(g);
    let frame = unsafe {
        f.mv.co.itemthrow.x8 = f.mv.co.itemthrow.x8.wrapping_add(1);
        f.mv.co.itemthrow.x8
    };
    let interval = h::common_data().x5B4;
    if interval.wrapping_sub(1) == frame % interval {
        ext::ft_800892A0(g);
        crate::ft0892::ft_80089824(g);
    }
}

/// `ftCo_800CE650`: accessory4 callback, the flame stream.
pub extern "C" fn ftco_800ce650(g: G) {
    melee_core::ct!("ftco_800ce650");
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    let cd = h::common_data();
    unsafe {
        if f.input.held_buttons[0] as u32 & HSD_PAD_A == 0 {
            f.mv.co.itemthrow.xC = 1;
        }
        if f.mv.co.itemthrow.x10 < cd.x5B0 {
            f.mv.co.itemthrow.x10 += 1.0;
        }
        if f.mv.co.itemthrow.x10 < cd.x5B0 && (f.input.pressed_buttons as u32 & HSD_PAD_A) != 0 {
            f.mv.co.itemthrow.x10 = 0.0;
        }
    }
    if super::ft_0cd1::ft_check_throw_b0(f) {
        unsafe { f.mv.co.itemthrow.x14 = 1 };
        flame_fx(g);
        shoot(g);
        let f = fp(g);
        if !it_8026b594(f.item_gobj) {
            let sp = 0.0f32;
            let mut c4 = z();
            ext::lb_8000B1CC(unsafe { (*f.parts).joint }, &mut c4);
            let dir = fp(g).facing_dir;
            lg::efSync_Spawn_dir_f(0x3FD, g, &c4, &dir, &sp);
            let mut b8 = z();
            ext::it_80292EF8(fp(g).item_gobj, &mut b8);
            super::ft_0cdd::ftco_800cde94(fp(g), &b8, &mut c4);
            lg::efSync_Spawn(0x3F3, g, &c4);
            c2::ftcommon_8007ebac(fp(g), 1, 0);
        }
        h::ftanim_set_anim_rate(g, 0.0);
    }
    let f = fp(g);
    if unsafe { f.mv.co.itemthrow.x14 } == 1 {
        // `mv.co.itemthrow.x4` and `mv.ca.specialhi.vel.x` are the same f32 (+0x4).
        unsafe {
            f.mv.co.itemthrow.x4 -= 1.0;
            if f.mv.ca.specialhi.vel.x <= 0.0 {
                f.mv.ca.specialhi.vel.x = cd.x5AC;
                flame_fx(g);
            }
        }
        let f = fp(g);
        unsafe {
            f.mv.co.itemthrow.facing_dir -= 1.0;
            if f.mv.co.itemthrow.facing_dir <= 0.0 {
                f.mv.co.itemthrow.facing_dir = cd.x5A8;
                shoot(g);
            }
        }
        let f = fp(g);
        unsafe {
            if f.mv.co.itemthrow.x8 >= cd.x5A4 && f.mv.co.itemthrow.xC != 0 && f.mv.co.itemthrow.x10 >= cd.x5B0 {
                f.mv.co.itemthrow.x14 = 0;
                h::ftanim_set_anim_rate(g, 1.0);
                c2::ftcommon_8007ecd4(fp(g), 1);
            }
        }
    }
}

/// `ftCo_FireFlowerShoot_Anim`.
pub fn ftco_fireflowershoot_anim(g: G) {
    melee_core::ct!("ftco_fireflowershoot_anim");
    let f = fp(g);
    if !f.item_gobj.is_null() {
        if super::ft_0cd1::ft_check_throw_b3(f) {
            ext::it_80292E64(f.item_gobj);
        }
        let f = fp(g);
        if super::ft_0cd1::ft_check_throw_b4(f) {
            ext::it_80292EAC(f.item_gobj);
        }
        super::ft_0cdd::ftco_800cde18(g);
    }
}
pub fn ftco_fireflowershootair_anim(g: G) { melee_core::ct!("ftco_fireflowershootair_anim"); ftco_fireflowershoot_anim(g) }
pub fn ftco_fireflowershoot_iasa(_g: G) { melee_core::ct!("ftco_fireflowershoot_iasa");}
pub fn ftco_fireflowershootair_iasa(_g: G) { melee_core::ct!("ftco_fireflowershootair_iasa");}
pub fn ftco_fireflowershoot_phys(g: G) { melee_core::ct!("ftco_fireflowershoot_phys"); super::ft_0cdd::ftco_800cde54(g) }
pub fn ftco_fireflowershootair_phys(g: G) { melee_core::ct!("ftco_fireflowershootair_phys"); super::ft_0cdd::ftco_800cde74(g) }
pub fn ftco_fireflowershoot_coll(g: G) { melee_core::ct!("ftco_fireflowershoot_coll"); lg::ft_800841B8(g, ft_800ce5a0) }
pub fn ftco_fireflowershootair_coll(g: G) { melee_core::ct!("ftco_fireflowershootair_coll"); lg::ft_80082C74(g, ft_800ce528) }
