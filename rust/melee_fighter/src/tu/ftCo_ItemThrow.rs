#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemThrow.c: item throws (light / heavy /
//! smash / dash / air), their direction pick, release velocity and coll/phys. Every function of
//! the TU, static inlines included; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_ItemThrow.rs.
//!
//! `ftCo_ItemThrowCmd.angle` (`s32 angle:12` after `u32 pad:20`) is read with GameCube
//! (MSB-first) bitfield order: the low 12 bits of `cmd_vars[0]`, sign-extended.

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

const HSD_PAD_A: u32 = 0x100;
const HSD_PAD_B: u32 = 0x200;
const HSD_PAD_LR: u32 = 0x8000_0000;

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
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
        it_8026B30C(item: *mut HSD_GObj) -> i32 = |item: *mut HSD_GObj| -> i32 { unsafe { melee_items::tu::it_26B1::it_8026B30C(item as _) as _ } };
        ftCo_GetLStickAngle(fp: &mut Fighter) -> f32 = |f| crate::ftcommon3::ftco_get_lstick_angle(f);
        ftCo_GetCStickAngle(fp: &mut Fighter) -> f32 = |f| crate::ftcommon3::ftco_get_cstick_angle(f);
        ftCo_800DF50C(fp: &mut Fighter) -> bool = |fp: &mut Fighter| -> bool { unsafe { crate::ft0df::ftco_800df50c(fp as _) as _ } };
        ftGetParasolStatus(g: G) -> i32 = crate::ftcommon3::ft_get_parasol_status;
        ftCo_800968C8(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_FallSpecial::ftCo_800968C8(g as _); } };
        ftCo_Fall_Enter(g: G) = |g| crate::tu::ftCo_Jump::ext::ftCo_Fall_Enter(g);
        Item_8026ABD8(item: *mut HSD_GObj, pos: &mut Vec3, f: f32) = |item: *mut HSD_GObj, pos: &mut Vec3, f: f32| { unsafe { let _ = melee_items::tu::item::Item_8026ABD8(item as _, pos as _, f as _); } };
        itGetDamageMultiplier(item: *mut HSD_GObj) -> f32 = |item: *mut HSD_GObj| -> f32 { unsafe { melee_items::tu::it_26B1::itGetDamageMultiplier(item as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCo_800974C4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_800974C4(g as _); } };
        ftCo_80096E68(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_80096E68(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        it_80272C90(item: *mut HSD_GObj) -> *mut HSD_JObj = |item: *mut HSD_GObj| -> *mut HSD_JObj { unsafe { melee_items::tu::it_2725::it_80272C90(item as _) as _ } };
        lb_8000B1CC(j: *mut HSD_JObj, pos0: *const Vec3, pos1: &mut Vec3) = |j: *mut HSD_JObj, pos0: *const Vec3, pos1: &mut Vec3| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, pos0 as _, pos1 as _); } };
        // `Fighter_804D6550` (fighter.c): the item-throw attribute table, 12-byte rows.
        Fighter_804D6550() -> *mut u8 = || todo!("TODO(port) Fighter_804D6550 (fighter.c)");
        pl_8003E978(slot: i32, sub: bool, item: *mut HSD_GObj) = |slot: i32, sub: bool, item: *mut HSD_GObj| { unsafe { let _ = crate::tu::plbonuslib::pl_8003E978(slot as _, sub as _, item as _); } };
        Item_8026AC74(item: *mut HSD_GObj, pos: &mut Vec3, vel: &mut Vec3, f: f32) = |item: *mut HSD_GObj, pos: &mut Vec3, vel: &mut Vec3, f: f32| { unsafe { let _ = melee_items::tu::item::Item_8026AC74(item as _, pos as _, vel as _, f as _); } };
        Item_8026AD20(item: *mut HSD_GObj, pos: &mut Vec3, vel: &mut Vec3, f: f32, smash: bool) = |item: *mut HSD_GObj, pos: &mut Vec3, vel: &mut Vec3, f: f32, smash: bool| { unsafe { let _ = melee_items::tu::item::Item_8026AD20(item as _, pos as _, vel as _, f as _, smash as _); } };
        itIsHeavy(item: *mut HSD_GObj) -> i32 = |item: *mut HSD_GObj| -> i32 { unsafe { melee_items::tu::it_26B1::itIsHeavy(item as _) as _ } };
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80085030(g: G, friction: f32, facing: f32) = crate::ft084e::ft_80085030;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
        ft_80084104(g: G) = crate::tu::ft_081b::ft_80084104;
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
        ft_80082708(g: G) -> u32 = |g| crate::tu::ft_081b::ft_80082708(g) as u32;
        ft_80081D0C(g: G) -> u32 = |g| crate::tu::ft_081b::ft_80081d0c(g) as u32;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
    }
}

pub unsafe extern "C" fn ftCo_80095EFC_c(g: G) {
    ftCo_80095EFC(g)
}
pub unsafe extern "C" fn ftCo_800974C4_c(g: G) {
    ext::ftCo_800974C4(g)
}
pub unsafe extern "C" fn ftCo_800961D0_c(g: G) {
    ftCo_800961D0(g)
}
pub unsafe extern "C" fn ftCo_80096498_c(g: G) {
    ftCo_80096498(g)
}

use melee_runtime::tu::lb_00F9::absf;

pub fn ftCo_80094E54(f: &mut Fighter) -> bool {
    f.input.pressed_buttons & HSD_PAD_A != 0
        && (f.input.held_buttons[0] & HSD_PAD_LR != 0 || ext::it_8026B30C(f.item_gobj) == 0)
}

pub fn ftCo_80094EA4(g: G) -> bool {
    let f = fp(g);
    let mut msid = f.motion_id;
    let (stick_x, stick_y, var_f28, var_f31, stick_angle, ret);
    if f.item_gobj.is_null() {
        return false;
    }
    if f.input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) != 0 {
        stick_x = f.input.lstick[0].x;
        stick_y = f.input.lstick[0].y;
        var_f28 = f.active_sticky.lstick.x as f32;
        var_f31 = f.active_sticky.lstick.y as f32;
        stick_angle = ext::ftCo_GetLStickAngle(fp(g));
        ret = true;
    } else {
        var_f28 = 0.0f32;
        stick_x = f.input.cstick[0].x;
        var_f31 = 0.0f32;
        stick_y = f.input.cstick[0].y;
        stick_angle = ext::ftCo_GetCStickAngle(fp(g));
        ret = false;
    }
    let f = fp(g);
    let c = cd();
    if absf(stick_x) >= c.dash_smash_stick_threshold && var_f28 < c.dash_smash_window as f32 + c.x44 {
        msid = (if stick_x * f.facing_dir >= 0.0 { ftCo_MS_HeavyThrowF4 } else { ftCo_MS_HeavyThrowB4 }) as i32;
    } else if stick_y >= c.xCC && var_f31 < c.xD0 + f.co_attrs.jump_startup_time {
        msid = ftCo_MS_HeavyThrowHi4 as i32;
    } else if stick_y <= c.xD4 && var_f31 < c.xD8 {
        msid = ftCo_MS_HeavyThrowLw4 as i32;
    } else {
        let mut block_32 = true;
        if absf(stick_x) >= c.x98 && absf(stick_angle) <= c.x20_radians {
            msid = (if stick_x * f.facing_dir >= 0.0 { ftCo_MS_HeavyThrowF } else { ftCo_MS_HeavyThrowB }) as i32;
            block_32 = false;
        }
        if block_32 {
            if stick_y >= c.attackhi3_stick_threshold_y && stick_angle > c.x20_radians {
                msid = ftCo_MS_HeavyThrowHi as i32;
            } else if stick_y <= c.xB0 && stick_angle < -c.x20_radians {
                msid = ftCo_MS_HeavyThrowLw as i32;
            } else if ret {
                msid = ftCo_MS_HeavyThrowF as i32;
            }
        }
    }
    if msid != fp(g).motion_id {
        ftCo_800958FC(g, msid);
        return true;
    }
    false
}

pub fn ftCo_8009515C(g: G) -> bool {
    let f = fp(g);
    if !f.item_gobj.is_null() && f.input.pressed_buttons & HSD_PAD_A != 0 {
        if unsafe { f.mv.co.itemthrow.x20 } != 0 {
            ftCo_800957F4(g, ftCo_MS_LightThrowDash as i32);
        } else {
            ftCo_80095A30(g);
        }
        return true;
    }
    unsafe {
        if f.mv.co.itemthrow.x20 != 0 {
            f.mv.co.itemthrow.x20 -= 1;
        }
    }
    false
}

pub fn ftCo_800951D0(g: G) -> bool {
    let f = fp(g);
    if !f.item_gobj.is_null() && ftCo_80094E54(f) {
        ftCo_80095A30(g);
        return true;
    }
    false
}

pub fn ftCo_80095254(g: G) -> bool {
    let f = fp(g);
    if !f.item_gobj.is_null() && ftCo_80094E54(f) {
        ftCo_800957F4(g, ftCo_MS_LightThrowDash as i32);
        return true;
    }
    false
}

pub fn ftCo_800952DC(g: G) -> bool {
    let f = fp(g);
    if !f.item_gobj.is_null() && f.input.held_buttons[0] & HSD_PAD_LR != 0 {
        ftCo_800957F4(g, ftCo_MS_LightThrowDash as i32);
        return true;
    }
    false
}

pub fn ftCo_80095328(g: G, arg1: *mut bool) -> bool {
    let f = fp(g);
    let (angle, var_f31, var_f30, var_f29, var_f28);
    if !f.item_gobj.is_null() && ext::ftCo_800DF50C(fp(g)) {
        let f = fp(g);
        var_f31 = f.input.cstick[0].x;
        var_f28 = f.input.cstick[0].y;
        var_f29 = 0.0f32;
        var_f30 = 0.0f32;
        angle = ext::ftCo_GetCStickAngle(f);
    } else if !fp(g).item_gobj.is_null() && ftCo_80094E54(fp(g)) {
        let f = fp(g);
        var_f31 = f.input.lstick[0].x;
        var_f28 = f.input.lstick[0].y;
        var_f30 = f.active_sticky.lstick.x as f32;
        var_f29 = f.active_sticky.lstick.y as f32;
        angle = ext::ftCo_GetLStickAngle(f);
    } else {
        return false;
    }
    let c = cd();
    if absf(var_f31) < c.xDC && absf(var_f28) < c.xE0 {
        let f = fp(g);
        if ext::it_8026B30C(f.item_gobj) == 0 && fp(g).input.held_buttons[0] & HSD_PAD_LR == 0 {
            ftCo_800957F4(g, 0x64);
            if !arg1.is_null() {
                unsafe { *arg1 = true };
            }
            return true;
        } else {
            ftCo_80095744(g, arg1);
            fp(g).set_x2224_b1(1);
            return true;
        }
    }
    let x3fc = c.x3FC as f32;
    let msid = if angle > c.x20_radians {
        if var_f29 < x3fc { 0x72 } else { 0x66 }
    } else if angle < -c.x20_radians {
        if var_f29 < x3fc { 0x73 } else { 0x67 }
    } else if var_f31 * fp(g).facing_dir >= 0.0 {
        if var_f30 < x3fc { 0x70 } else { 0x64 }
    } else if var_f30 < x3fc {
        0x71
    } else {
        0x65
    };
    ftCo_800957F4(g, msid);
    if !arg1.is_null() {
        unsafe { *arg1 = true };
    }
    true
}

pub fn ftCo_8009563C(g: G) -> bool {
    let f = fp(g);
    if !f.item_gobj.is_null() && ftCo_80094E54(f) && unsafe { fp(g).mv.co.itemthrow4.unk_timer } != 0 {
        let m = if fp(g).motion_id == ftCo_MS_EscapeF as i32 { ftCo_MS_LightThrowF4 } else { ftCo_MS_LightThrowB4 };
        ftCo_800957F4(g, m as i32);
        return true;
    }
    let f = fp(g);
    unsafe {
        if f.mv.co.itemthrow4.unk_timer != 0 {
            f.mv.co.itemthrow4.unk_timer -= 1;
        }
    }
    false
}

/// static `ftCo_80095700`: the throw's facing (reversed for the back throws).
pub fn ftCo_80095700(g: G, arg1: i32) {
    let f = fp(g);
    unsafe {
        match arg1 {
            95 | 101 | 105 | 109 | 113 | 117 => f.mv.co.itemthrow.facing_dir = -f.facing_dir,
            _ => f.mv.co.itemthrow.facing_dir = f.facing_dir,
        }
    }
}

pub fn ftCo_80095744(g: G, arg1: *mut bool) {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::ftGetParasolStatus(g) != -1 {
        if fp(g).motion_id == ftCo_MS_ItemParasolFallSpecial as i32 {
            ext::ftCo_800968C8(g);
        } else {
            ext::ftCo_Fall_Enter(g);
        }
        if !arg1.is_null() {
            unsafe { *arg1 = true };
        }
    } else if !arg1.is_null() {
        unsafe { *arg1 = false };
    }
    ext::Item_8026ABD8(fp(g).item_gobj, &mut vec, 1.0);
}

/// static inline `getAnimSpeed`.
#[inline(always)]
pub fn getAnimSpeed(g: G, msid: i32) -> f32 {
    let mut speed = 1.0f32;
    if msid >= ftCo_MS_LightThrowF4 as i32 {
        speed *= cd().x400;
    }
    speed *= 1.0 / ext::itGetDamageMultiplier(fp(g).item_gobj);
    speed
}

pub fn ftCo_800957F4(g: G, msid: i32) {
    let f = fp(g);
    f.cmd_vars[0] = 0;
    f.cmd_vars[1] = 0;
    f.anon_2210.throw_flags = 0;
    let anim_spd = getAnimSpeed(g, msid);
    unsafe { fp(g).mv.co.itemthrow4.anim_spd = anim_spd };
    ftCo_80095700(g, msid);
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None, 0.0, anim_spd, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    fp(g).accessory4_cb = Some(ftCo_80095EFC_c);
    ftCo_80095EFC(g);
}

pub fn ftCo_800958FC(g: G, msid: i32) {
    let f = fp(g);
    f.cmd_vars[0] = 0;
    f.cmd_vars[1] = 0;
    f.anon_2210.throw_flags = 0;
    let speed = getAnimSpeed(g, msid);
    ftCo_80095700(g, msid);
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, speed, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.accessory4_cb = Some(ftCo_80095EFC_c);
    f.take_dmg_cb = Some(ftCo_800974C4_c);
    if f.x2222_b0() != 0 {
        f.phys_cb = Some(ftCo_800961D0_c);
        f.coll_cb = Some(ftCo_80096498_c);
    }
    ftCo_80095EFC(g);
}

pub fn ftCo_80095A30(g: G) {
    let f = fp(g);
    let c = cd();
    let r29;
    if absf(f.input.lstick[0].x) >= c.dash_smash_stick_threshold
        && (f.active_sticky.lstick.x as f32) < c.dash_smash_window as f32 + c.x44
    {
        r29 = if f.input.lstick[0].x * f.facing_dir >= 0.0 { 0x6C } else { 0x6D };
    } else if f.input.lstick[0].y >= c.xCC
        && (f.active_sticky.lstick.y as f32) < c.xD0 + f.co_attrs.jump_startup_time
    {
        r29 = 0x6E;
    } else if f.input.lstick[0].y <= c.xD4 && (f.active_sticky.lstick.y as f32) < c.xD8 {
        r29 = 0x6F;
    } else {
        let mut res = None;
        if absf(fp(g).input.lstick[0].x) >= c.x98 {
            let a = if ext::ftCo_GetLStickAngle(fp(g)) < 0.0 {
                -ext::ftCo_GetLStickAngle(fp(g))
            } else {
                ext::ftCo_GetLStickAngle(fp(g))
            };
            if a <= c.x20_radians {
                let f = fp(g);
                res = Some(if f.input.lstick[0].x * f.facing_dir >= 0.0 { 0x5E } else { 0x5F });
            }
        }
        r29 = match res {
            Some(v) => v,
            None => {
                if fp(g).input.lstick[0].y >= c.attackhi3_stick_threshold_y
                    && ext::ftCo_GetLStickAngle(fp(g)) > c.x20_radians
                {
                    0x60
                } else if fp(g).input.lstick[0].y <= c.xB0
                    && ext::ftCo_GetLStickAngle(fp(g)) < -c.x20_radians
                {
                    0x61
                } else if ext::it_8026B30C(fp(g).item_gobj) == 0 {
                    0x5E
                } else {
                    0x63
                }
            }
        };
    }
    ftCo_800957F4(g, r29);
}

/// static `ftCo_80095D5C`: the release velocity from the throw table / the script's override.
pub fn ftCo_80095D5C(f: &mut Fighter, arg1: &mut Vec3) {
    let cmd_var0 = f.cmd_vars[0];
    let mut vel = 1.0f32;
    if cmd_var0 != 0 {
        vel = 0.01f32 * ((cmd_var0 >> 12) & 0x3FF) as f32;
    }
    let vel_mul = f.co_attrs.item_throw_velocity_multiplier;
    let row = unsafe { ext::Fighter_804D6550().offset(f.motion_id as isize * 12) };
    let rd = |off: isize| unsafe { (row.offset(off) as *const f32).read_unaligned() };
    vel *= vel_mul * rd(-0x468);
    let angle;
    if cmd_var0 != 0 {
        let int_angle = ((cmd_var0 << 20) as i32) >> 20;
        if int_angle == 361 {
            angle = rd(-0x464);
        } else {
            angle = int_angle as f32 * 0.01745329252f32;
        }
        f.cmd_vars[0] = 0;
    } else {
        angle = rd(-0x464);
    }
    unsafe {
        arg1.x = f.mv.co.itemthrow.facing_dir * (vel * melee_runtime::gekko::cosf(angle));
    }
    arg1.y = vel * melee_runtime::gekko::sinf(angle);
    arg1.z = 0.0;
}

pub fn ftCo_ItemThrow_Anim(g: G) {
    let f = fp(g);
    if unsafe { f.anon_2210.anon_0.throw_flags_b4() } != 0 {
        unsafe { f.anon_2210.anon_0.set_throw_flags_b4(0) };
        f.facing_dir = -f.facing_dir;
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_80095EFC(g: G) {
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    let mut vec0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let j = ext::it_80272C90(f.item_gobj);
    ext::lb_8000B1CC(j, core::ptr::null(), &mut vec0);
    let f = fp(g);
    let b3 = unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    };
    if !b3 {
        unsafe { f.mv.co.itemthrow4.x8 = vec0 };
        return;
    }
    ftCo_80095D5C(f, &mut vec1);
    let f = fp(g);
    let cmd_var1 = f.cmd_vars[1];
    let mut scale = 1.0f32;
    if cmd_var1 != 0 {
        scale = 0.01f32 * (cmd_var1 & 0x3FFFFF) as f32;
        f.cmd_vars[1] = 0;
    }
    let fsm = -f.cmd_timer / f.frame_speed_mul;
    let tbl = ext::Fighter_804D6550();
    let x8 = unsafe {
        (tbl.offset((f.motion_id - ftCo_MS_LightThrowF as i32) as isize * 12 + 8) as *const f32).read_unaligned()
    };
    scale *= f.co_attrs.heavy_throw_velocity_multiplier * x8;
    let throw_speed = scale;
    let (t8x, t8y) = unsafe { (f.mv.co.itemthrow4.x8.x, f.mv.co.itemthrow4.x8.y) };
    // `fsm * (x8 - vec0) + vec0` is one fmuladd -> fmadds.
    let mut vec2 = Vec3 {
        x: fmadds(fsm, t8x - vec0.x, vec0.x),
        y: fmadds(fsm, t8y - vec0.y, vec0.y),
        z: 0.0,
    };
    ext::pl_8003E978(f.player_id as i32, f.is_sub_fighter() != 0, f.item_gobj);
    let f = fp(g);
    let msid = f.motion_id;
    if msid == ftCo_MS_LightThrowDrop as i32 {
        ext::Item_8026AC74(f.item_gobj, &mut vec2, &mut vec1, throw_speed);
    } else if msid >= ftCo_MS_LightThrowF4 as i32 {
        let heavy = ext::itIsHeavy(f.item_gobj) == 1;
        ext::ftCommon_8007EBAC(fp(g), if heavy { 29 } else { 27 }, 0);
        ext::Item_8026AD20(fp(g).item_gobj, &mut vec2, &mut vec1, throw_speed, true);
    } else {
        let heavy = ext::itIsHeavy(f.item_gobj) == 1;
        ext::ftCommon_8007EBAC(fp(g), if heavy { 28 } else { 26 }, 0);
        ext::Item_8026AD20(fp(g).item_gobj, &mut vec2, &mut vec1, throw_speed, false);
    }
}

pub fn ftCo_ItemThrow_IASA(_g: G) {}

pub fn ftCo_ItemThrow_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_LightThrowDash_Phys(g: G) {
    let f = fp(g);
    let c = cd();
    if f.cur_anim_frame <= c.x408 {
        ext::ft_80085030(g, (c.x404 * f.co_attrs.ground_friction) * c.x40C, f.facing_dir);
    } else {
        ext::ft_80085030(g, c.x404 * f.co_attrs.ground_friction, f.facing_dir);
    }
}

pub fn ftCo_LightThrowAir_Phys(g: G) {
    ext::ft_80084DB0(g);
}

pub fn ftCo_800961D0(g: G) {
    if fp(g).ground_or_air == GA_Air {
        ext::ft_80084DB0(g);
    } else {
        ext::ft_80084F3C(g);
    }
}

pub fn ftCo_LightThrowDashDrop_Coll(g: G) {
    ext::ft_80084104(g);
}

pub fn ftCo_LightThrow_Coll(g: G) {
    ext::ft_800841B8(g, ftCo_80096250);
}

/// static inline `inlineA0`: switch ground/air throw keeping frame, speed and facing.
#[inline(always)]
pub fn inlineA0(g: G, msid: i32) {
    let f = fp(g);
    let facing_dir = f.facing_dir;
    if facing_dir != f.facing_dir1 {
        f.facing_dir = f.facing_dir1;
    }
    let flags = Ft_MF_SkipMatAnim
        | Ft_MF_SkipColAnim
        | Ft_MF_UpdateCmd
        | Ft_MF_SkipItemVis
        | Ft_MF_Unk19
        | Ft_MF_SkipModelPartVis
        | Ft_MF_SkipModelFlags
        | Ft_MF_Unk27;
    let (frame, spd) = (f.cur_anim_frame, unsafe { f.mv.co.itemthrow4.anim_spd });
    ext::Fighter_ChangeMotionState(g, msid, flags, frame, spd, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.facing_dir = facing_dir;
    f.accessory4_cb = Some(ftCo_80095EFC_c);
    ftCo_80095EFC(g);
}

pub fn ftCo_80096250(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
    let m = fp(g).motion_id;
    if m >= ftCo_MS_LightThrowF4 as i32 {
        inlineA0(g, m + 4);
    } else {
        inlineA0(g, m + 6);
    }
}

pub fn ftCo_LightThrowAir_Coll(g: G) {
    ext::ft_80082C74(g, ftCo_80096374);
}

pub fn ftCo_80096374(g: G) {
    ext::ftCommon_8007D7FC(fp(g));
    let m = fp(g).motion_id;
    if m >= ftCo_MS_LightThrowF4 as i32 {
        inlineA0(g, m - 4);
    } else {
        inlineA0(g, m - 6);
    }
}

pub fn ftCo_HeavyThrow_Coll(g: G) {
    ext::ft_800841B8(g, ext::ftCo_80096E68);
}

pub fn ftCo_80096498(g: G) {
    if fp(g).ground_or_air == GA_Ground {
        if ext::ft_80082708(g) == GA_Ground {
            ext::ftCommon_8007D5D4(fp(g));
        }
    } else if ext::ft_80081D0C(g) != GA_Ground {
        ext::ftCommon_8007D7FC(fp(g));
    }
}
