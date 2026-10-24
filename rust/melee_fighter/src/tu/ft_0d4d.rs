//! src/melee/ft/ft_0D4D.c: the Rebirth (respawn platform) states, the coin drop on KO, and the
//! end of DeadUpFall. Hand port of every function in the TU.
//!
//! Fused sites (`tools/rust-port/fma_sites.py`, shipping contraction): ftCo_800D4FF4 x3,
//! fn_800D54A4, ftCo_Rebirth_Phys, ftCo_RebirthWait_Phys: each `facing_dir * ftCommon_800804EC(fp)
//! +/- c` is one fmadds/fmsubs (the `-(a*b - c)` sites are fmsubs then negate).
#![allow(non_snake_case, static_mut_refs, unused_unsafe)]

use crate::hooks as h;
use crate::types::*;
use core::ffi::c_void;
use melee_core::fp::{fmadds, fmsubs};

use super::ft_0d31::ftCo_800D331C;

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, non_upper_case_globals, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    /// fighter.c globals (set by Fighter_LoadCommonData).
    pub static mut Fighter_804D6534: *mut *mut c_void = core::ptr::null_mut();
    pub static mut H_Stage_GetCamBoundsBottomOffset: Option<fn() -> f32> = None;
    pub fn Stage_GetCamBoundsBottomOffset() -> f32 { match unsafe { H_Stage_GetCamBoundsBottomOffset } { Some(f) => f(), None => unsafe { melee_world::tu::stage::Stage_GetCamBoundsBottomOffset() } } }
    pub static mut H_Stage_80224E38: Option<fn(&mut Vec3, i32)> = None;
    pub fn Stage_80224E38(out: &mut Vec3, i: i32) { match unsafe { H_Stage_80224E38 } { Some(f) => f(out, i), None => panic!("TODO(group2 stage) Stage_80224E38") } }
    pub static mut H_ftCamera_80076320: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCamera_80076320(g: *mut HSD_GObj) { match unsafe { H_ftCamera_80076320 } { Some(f) => f(g), None => { unsafe { let _ = crate::tu::ftcamera::ftcamera_80076320(g as _); } return; } } }
    pub static mut H_ftCamera_80076018: Option<fn(*mut UnkFloat6_Camera, &mut UnkFloat6_Camera, f32)> = None;
    pub fn ftCamera_80076018(inp: *mut UnkFloat6_Camera, out: &mut UnkFloat6_Camera, mul: f32) { match unsafe { H_ftCamera_80076018 } { Some(f) => f(inp, out, mul), None => unsafe { crate::tu::ftcamera::ftcamera_80076018(&*(inp as *const _), out, mul as _) } } }
    pub static mut H_ftCo_800BFD04: Option<fn(*mut HSD_GObj)> = None;
    pub fn ftCo_800BFD04(g: *mut HSD_GObj) { match unsafe { H_ftCo_800BFD04 } { Some(f) => f(g), None => { unsafe { let _ = crate::tu::ftcolanim::ftco_800bfd04(g as _); } return; } } }
    /// efsync.c `efSync_Spawn(kind, gobj, ...)`; the variadic tail as raw words.
    pub static mut H_efSync_Spawn: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut c_void> = None;
    pub fn efSync_Spawn(kind: i32, g: *mut HSD_GObj, va: &[usize]) -> *mut c_void { match unsafe { H_efSync_Spawn } { Some(f) => f(kind, g, va), None => panic!("TODO(group2 efsync) efSync_Spawn") } }
    pub static mut H_Fighter_ChangeMotionState: Option<fn(*mut HSD_GObj, i32, u32, f32, f32, f32, *mut HSD_GObj)> = None;
    pub fn Fighter_ChangeMotionState(g: *mut HSD_GObj, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, item: *mut HSD_GObj) { match unsafe { H_Fighter_ChangeMotionState } { Some(f) => f(g, msid, flags, start, speed, blend, item), None => { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, item as _); } return; } } }
    pub static mut H_ftCo_8008A7A8: Option<fn(*mut HSD_GObj, *mut c_void)> = None;
    pub fn ftCo_8008A7A8(g: *mut HSD_GObj, p: *mut c_void) { match unsafe { H_ftCo_8008A7A8 } { Some(f) => f(g, p), None => { unsafe { let _ = crate::tu::ftwaitanim::ftCo_8008A7A8(g as _, p as _); } return; } } }
    pub static mut H_ft_80083DCC: Option<fn(*mut HSD_GObj)> = None;
    pub fn ft_80083DCC(g: *mut HSD_GObj) { match unsafe { H_ft_80083DCC } { Some(f) => f(g), None => { unsafe { let _ = crate::tu::ft_081b::ft_80083dcc(g as _); } return; } } }
    pub static mut H_ft_80083844: Option<fn(*mut HSD_GObj, HSD_GObjEvent)> = None;
    pub fn ft_80083844(g: *mut HSD_GObj, cb: HSD_GObjEvent) { match unsafe { H_ft_80083844 } { Some(f) => f(g, cb), None => unsafe { crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80083844(g as _, ev)) } } }
    pub static mut H_mpColl_80043680: Option<fn(&mut CollData, &Vec3)> = None;
    pub fn mpColl_80043680(c: &mut CollData, v: &Vec3) { match unsafe { H_mpColl_80043680 } { Some(f) => f(c, v), None => unsafe { melee_world::tu::mpcoll::mpColl_80043680(c as *mut _ as *mut _, v as *const _ as *mut _) } } }
    pub static mut H_ftColl_8007B7A4: Option<fn(*mut HSD_GObj, i32)> = None;
    pub fn ftColl_8007B7A4(g: *mut HSD_GObj, n: i32) { match unsafe { H_ftColl_8007B7A4 } { Some(f) => f(g, n), None => unsafe { crate::tu::ftcoll::ftcoll_8007b7a4(g as _, n as _) } } }
    pub static mut H_ftCo_SpecialAir_CheckInput: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_SpecialAir_CheckInput(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_SpecialAir_CheckInput } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(g as _) as _ }; } } }
    pub static mut H_ftCo_800D7100: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800D7100(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800D7100 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_Attack100::ftCo_800D7100(g as _) as _ }; } } }
    pub static mut H_ftCo_800C3B10: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800C3B10(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800C3B10 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftco_aircatch::ftco_800c3b10(g as _) as _ }; } } }
    pub static mut H_ftCo_80099A58: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_80099A58(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_80099A58 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_EscapeAir::ftCo_80099A58(g as _) as _ }; } } }
    pub static mut H_ftCo_AttackAir_CheckItemThrowInput: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_AttackAir_CheckItemThrowInput(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_AttackAir_CheckItemThrowInput } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_AttackAir::ftCo_AttackAir_CheckItemThrowInput(g as _) as _ }; } } }
    pub static mut H_ftCo_800D705C: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800D705C(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800D705C } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_Attack100::ftCo_800D705C(g as _) as _ }; } } }
    pub static mut H_ftCo_800CB870: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800CB870(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800CB870 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_JumpAerial::ftCo_800CB870(g as _) as _ }; } } }
    pub static mut H_ftCo_80091A2C: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_80091A2C(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_80091A2C } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_Guard::ftCo_80091A2C(g as _) as _ }; } } }
    pub static mut H_ftCo_800DE9B8: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800DE9B8(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800DE9B8 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_AppealS::ftCo_800DE9B8(g as _) as _ }; } } }
    pub static mut H_fn_800D5F84: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn fn_800D5F84(g: *mut HSD_GObj) -> bool { match unsafe { H_fn_800D5F84 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_Squat::fn_800D5F84(g as _) as _ }; } } }
    pub static mut H_ftCo_800C97A8: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftCo_800C97A8(g: *mut HSD_GObj) -> bool { match unsafe { H_ftCo_800C97A8 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftCo_Turn::ftCo_800C97A8(g as _) as _ }; } } }
    pub static mut H_ftWalkCommon_800DFC70: Option<fn(*mut HSD_GObj) -> bool> = None;
    pub fn ftWalkCommon_800DFC70(g: *mut HSD_GObj) -> bool { match unsafe { H_ftWalkCommon_800DFC70 } { Some(f) => f(g), None => { return unsafe { crate::tu::ftwalkcommon::ftwalkcommon_800dfc70(g as _) as _ }; } } }
}

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    h::common_data()
}

unsafe extern "C" fn fn_800D54A4_cb(g: G) { fn_800D54A4(g) }
unsafe extern "C" fn fn_800D55B4_cb(g: G) { fn_800D55B4(g) }
unsafe extern "C" fn fn_800D5A30_cb(g: G) { fn_800D5A30(g) }
pub(crate) unsafe extern "C" fn fn_800D4DD4_cb(g: G) { fn_800D4DD4(g) }

/// `fn_800D4DD4`: DeadUpFall accessory: once in phase 3, below the camera bottom, drop the GFX.
pub fn fn_800D4DD4(g: G) {
    let f = fp(g);
    if unsafe { f.mv.co.unk_deadup.x44 } == 3 {
        let bottom = ext::Stage_GetCamBoundsBottomOffset();
        if f.cur_pos.y < bottom {
            crate::ftcommon2::ftcommon_8007e2fc(f);
        }
    }
}

pub fn ftCo_DeadUpFall_Cam(g: G) { ext::ftCamera_80076320(g) }

/// `ftCo_800D4E50`: coin mode (gm_8016B0B4): half the coins drop from the KO point.
pub fn ftCo_800D4E50(f: &mut Fighter, arg1: &Vec3, arg2: i32, arg8: f32) {
    if h::gm_8016b0b4() && f.is_sub_fighter() == 0 {
        let slot = f.player_id as i32;
        let coins = crate::player::get_coins(slot);
        let mut n = coins / 2;
        crate::player::set_coins(slot, coins - n);
        crate::player::set_unk9c(slot, n + crate::player::get_unk9c(slot));
        if n > cd().x5C8 {
            n = cd().x5C8;
        }
        h::it_802e5f8c(f.gobj, arg1, n, arg2, arg8, cd().x5CC);
    }
}

/// `ftCo_800D4F24(gobj, index)`: KO without a blast-zone state (index != 0: the star GFX).
pub fn ftCo_800D4F24(g: G, index: i32) {
    let f = fp(g);
    if f.x221F_b3() != 0 {
        return;
    }
    if index != 0 {
        let sp1c: f32 = unsafe { (*(*f.ft_data).x0).x168 } * f.x34_scale.y;
        ext::efSync_Spawn(0x43F, core::ptr::null_mut(), &[&f.cur_pos as *const Vec3 as usize, &sp1c as *const f32 as usize]);
    }
    ftCo_800D331C(g);
    let f = fp(g);
    if f.x221D_b6() != 0 {
        crate::ft0877::ft_800880d8(f);
        f.x2004 = 0;
    }
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x221E_b2(1);
    ext::ftCo_800BFD04(g);
}

/// `ftCo_800D4FF4`: enter Rebirth (descend on the respawn platform).
pub fn ftCo_800D4FF4(g: G) {
    h::fighter_unk_process_death(g);
    let f = fp(g);
    f.smash_attrs.x2135 = crate::player::player_80032f30(f.player_id as i32) as i8;
    unsafe {
        if f.smash_attrs.x2135 == -1 {
            let sp44 = crate::player::get_spawn_platform_pos(f.player_id as i32);
            f.mv.co.turn.facing_after = fmadds(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), sp44.x);
            f.mv.co.turn.x8 = sp44.y;
            f.mv.co.walk.middle_anim_frame = 0.0;
        } else {
            let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::Stage_80224E38(&mut sp14, f.smash_attrs.x2135 as i32);
            let sp20 = crate::player::get_some_pos(f.player_id as i32);
            f.mv.co.turn.facing_after =
                fmadds(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), sp14.x + sp20.x);
            f.mv.co.turn.x8 = sp14.y + sp20.y;
            f.mv.co.walk.middle_anim_frame = 0.0;
        }
    }
    crate::ftcommon2::ftcommon_8007d5d4(f);
    unsafe { f.mv.co.unk_deadup.x40 = cd().rebirth_countdown };
    ext::Fighter_ChangeMotionState(g, 0xC, 0x1002, 0.0, 1.0, -1.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x221E_b2(1);
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x2222_b7(1);
    f.set_x221D_b5(1);
    h::ftco_800bffd0(f, 0xA, 0);
    if f.is_sub_fighter() == 0 {
        let tbl = unsafe { ext::Fighter_804D6534 };
        crate::ftcommon3::set_accessory(f, unsafe { *tbl } as *mut HSD_Joint);
        let s = f.x34_scale.y * f.co_attrs.respawn_platform_scale;
        let scale = Vec3 { x: s, y: s, z: s };
        h::hsd_jobj_set_scale(f.x20A0_accessory, &scale);
        crate::ftcommon3::ftcommon_8007e690(f, unsafe { *tbl.add(1) } as *mut HSD_AnimJoint);
        let pos = Vec3 {
            x: -fmsubs(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), f.cur_pos.x),
            y: f.cur_pos.y,
            z: f.cur_pos.z,
        };
        h::hsd_jobj_set_translate(f.x20A0_accessory, &pos);
        f.accessory1_cb = Some(fn_800D54A4_cb);
    } else {
        f.accessory1_cb = Some(fn_800D55B4_cb);
    }
    crate::ftcommon2::ftcommon_8007efc0(f, 1);
}

pub fn ftCo_Rebirth_Anim(g: G) {
    let f = fp(g);
    ext::ftCo_8008A7A8(g, unsafe { (*f.ft_data).x24 });
    let f = fp(g);
    unsafe {
        f.mv.co.common.x0 -= 1;
        if f.mv.co.common.x0 == 0 {
            ftCo_800D5600(g);
        }
    }
}

pub fn ftCo_Rebirth_IASA(_g: G) {}

/// Shared body of Rebirth_Phys / RebirthWait_Phys (they differ only in the operand order of
/// the final products, which is irrelevant: `inv * d` and `d * inv` round the same).
fn rebirth_phys(g: G) {
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        unsafe {
            if f.smash_attrs.x2135 != -1 {
                let mut st = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::Stage_80224E38(&mut st, f.smash_attrs.x2135 as i32);
                let pl = crate::player::get_some_pos(f.player_id as i32);
                f.mv.co.common.x4.x = fmadds(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), st.x + pl.x);
                f.mv.co.common.x4.y = st.y + pl.y;
                f.mv.co.common.x4.z = 0.0;
            }
            let cur = crate::ftcommon2::ftcommon_8007f8b4(f);
            let inv = 1.0f32 / f.mv.co.common.x0 as f32;
            f.self_vel.x = (f.mv.co.common.x4.x - cur.x) * inv;
            f.self_vel.y = (f.mv.co.common.x4.y - cur.y) * inv;
        }
    } else {
        let o = fp(crate::player::get_entity_at_index(f.player_id as i32, 0));
        let (x, y) = (o.self_vel.x, o.self_vel.y);
        let f = fp(g);
        f.self_vel.x = x;
        f.self_vel.y = y;
    }
}

pub fn ftCo_Rebirth_Phys(g: G) { rebirth_phys(g) }

pub fn ftCo_Rebirth_Coll(g: G) {
    if fp(g).smash_attrs.x2135 == -1 {
        ext::ft_80083DCC(g);
    }
}

/// `fn_800D54A4`: main entity's accessory1 callback: keep the platform under the fighter.
pub fn fn_800D54A4(g: G) {
    let f = fp(g);
    let other = crate::player::get_entity_at_index(f.player_id as i32, 1);
    if !other.is_null() {
        let o = fp(other);
        if o.x221F_b3() == 0 {
            let oy = o.cur_pos.y;
            let f = fp(g);
            if oy > f.cur_pos.y {
                f.cur_pos.y = oy;
            }
        }
    }
    let f = fp(g);
    let sp = Vec3 {
        x: -fmsubs(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), f.cur_pos.x),
        y: f.cur_pos.y,
        z: f.cur_pos.z,
    };
    h::hsd_jobj_set_translate(f.x20A0_accessory, &sp);
}

/// `fn_800D55B4`: sub entity's accessory1 callback: never below the main entity.
pub fn fn_800D55B4(g: G) {
    let f = fp(g);
    let o = fp(crate::player::get_entity_at_index(f.player_id as i32, 0));
    let oy = o.cur_pos.y;
    let f = fp(g);
    if oy > f.cur_pos.y {
        f.cur_pos.y = oy;
    }
}

/// `ftCo_800D5600`: Rebirth -> RebirthWait.
pub fn ftCo_800D5600(g: G) {
    let f = fp(g);
    let pos = f.cur_pos;
    ext::mpColl_80043680(&mut f.coll_data, &pos);
    f.self_vel.y = 0.0;
    unsafe { f.mv.co.common.x0 = cd().rebirth_wait };
    ext::Fighter_ChangeMotionState(
        g,
        ftCo_MS_RebirthWait as i32,
        (Ft_MF_KeepGfx | Ft_MF_SkipColAnim | Ft_MF_KeepAccessory | Ft_MF_SkipNametagVis) as u32,
        0.0,
        1.0,
        0.0,
        core::ptr::null_mut(),
    );
    let f = fp(g);
    f.set_x221E_b2(1);
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x221D_b5(1);
    if f.is_sub_fighter() == 0 {
        f.accessory1_cb = Some(fn_800D54A4_cb);
    } else {
        f.accessory1_cb = Some(fn_800D55B4_cb);
    }
    if f.smash_attrs.x2135 != -1 {
        let pos = f.cur_pos;
        ext::mpColl_80043680(&mut f.coll_data, &pos);
    }
}

pub fn ftCo_RebirthWait_Anim(g: G) {
    let f = fp(g);
    ext::ftCo_8008A7A8(g, unsafe { (*f.ft_data).x24 });
    let f = fp(g);
    unsafe {
        f.mv.co.common.x0 -= 1;
        if f.mv.co.common.x0 == 0 {
            ext::ftColl_8007B7A4(g, cd().x5D8);
            h::ftco_fall_enter(g);
        }
    }
}

pub fn ftCo_RebirthWait_IASA(g: G) {
    let mut var_r30 = false;
    let f = fp(g);
    let (slot, sub) = (f.player_id as i32, f.is_sub_fighter() as i32);
    if sub == 0 {
        let c = crate::player::get_entity_at_index(slot, 1);
        if !c.is_null() && fp(c).x221F_b3() == 0 && !h::ftlib_800873cc(c) {
            var_r30 = true;
        }
    } else {
        if h::ftlib_800873cc(crate::player::get_entity_at_index(slot, 0)) {
            return;
        }
        var_r30 = true;
    }
    if ext::ftCo_SpecialAir_CheckInput(g)
        || ext::ftCo_800D7100(g)
        || ext::ftCo_800C3B10(g)
        || ext::ftCo_80099A58(g)
        || ext::ftCo_AttackAir_CheckItemThrowInput(g)
        || ext::ftCo_800D705C(g)
        || ext::ftCo_800CB870(g)
    {
        // priority input detected
    } else if ext::ftCo_80091A2C(g)
        || ext::ftCo_800DE9B8(g)
        || ext::fn_800D5F84(g)
        || ext::ftCo_800C97A8(g)
        || ext::ftWalkCommon_800DFC70(g)
        || var_r30
    {
        h::ftco_fall_enter(g);
    } else {
        return;
    }
    ext::ftColl_8007B7A4(g, cd().x5D8);
    crate::plbonus::pl_80040374(slot, sub);
}

pub fn ftCo_RebirthWait_Phys(g: G) { rebirth_phys(g) }

pub fn ftCo_RebirthWait_Coll(g: G) { ext::ft_80083844(g, Some(fn_800D5A30_cb)) }

pub fn fn_800D5A30(g: G) {
    ext::ftColl_8007B7A4(g, cd().x5D8);
    h::ft_8008a2bc(g);
}

/// `ftCo_Rebirth_Cam`: the camera box follows the platform target, raised by the camera offset.
pub fn ftCo_Rebirth_Cam(g: G) {
    let f = fp(g);
    let mut spc: UnkFloat6_Camera = unsafe { core::mem::zeroed() };
    ext::ftCamera_80076018(unsafe { (*f.ft_data).x3C }, &mut spc, f.x34_scale.y);
    let b = unsafe { &mut *f.x890_cameraBox };
    unsafe {
        b.pos.x = f.mv.co.common.x4.x;
        b.pos.y = f.mv.co.common.x4.y + spc.x0.x;
    }
    b.pos.z = 0.0;
    crate::ftlib::ftlib_800866dc(g, &mut b.bone_pos);
}

#[allow(dead_code)]
fn _unused(_: *mut c_void) {}
