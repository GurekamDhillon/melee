#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Bury.c (buried in the ground: Bury, BuryWait,
//! BuryJump, plus the stage bury timers). Every function of the TU; calls out of it go through
//! `ext` (unported ones panic until rewired; ported ones default to the port). The
//! ftCommon/inlines.h `ftCo_ReleaseItemAndVictim` comes from `tu::ftCo_Barrel` (callees hook
//! there). Checked against the C (whole TU, shipping contraction) by tests/tu_ftCo_Bury.rs.

use crate::types::*;
use melee_core::fp::fmadds;
use melee_types::inline::{JObjDirty, HSD_JObjMtxIsDirty, HSD_JObjSetRotationZ, HSD_JObjSetTranslate};
use super::ftCo_Barrel::ftCo_ReleaseItemAndVictim;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

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
    macro_rules! t { ($n:literal) => { |_| todo!(concat!("TODO(port) ", $n)) }; }
    type D = *mut DynamicsDesc;
    type FH = *mut FighterHurtCapsule;
    ext_fns! {
        ftCo_8009EB18(g: G) = crate::tu::ftCo_BarrelWait::ftCo_8009EB18;
        ftColl_800765F0(fp: *mut Fighter, victim: G, v: f32) -> f32 = |_, _, _| todo!("TODO(port) ftColl_800765F0 (ftcoll.c)");
        ftColl_80076640(fp: *mut Fighter, f: *mut f32) -> bool = |_, _| todo!("TODO(port) ftColl_80076640 (ftcoll.c)");
        ftColl_80076764(a: i32, b: i32, g: G, d: D, fp: *mut Fighter, h: FH) = |_, _, _, _, _, _| todo!("TODO(port) ftColl_80076764 (ftcoll.c)");
        lbColl_80008D30(hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1) = |hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1| { unsafe { let _ = melee_runtime::tu::lbcollision::lbColl_80008D30(hit as _, a as _); } };
        ftColl_80078384(fp: *mut Fighter, h: FH, hit: *mut HitCapsule) = |_, _, _| todo!("TODO(port) ftColl_80078384 (ftcoll.c)");
        pl_8003EC30(slot: i32, sub: i32, kind: i32, f: f32) = |slot: i32, sub: i32, kind: i32, f: f32| { unsafe { let _ = crate::tu::plbonuslib::pl_8003EC30(slot as _, sub as _, kind as _, f as _); } };
        stage_info_grkind() -> i32 = || todo!("TODO(port) stage_info.grkind (ground.c global)");
        Ground_801C5700(i: i32) -> D = |_| todo!("TODO(port) Ground_801C5700 (ground.c)");
        ftCo_8008DCE0(g: G, a: i32, facing: f32) = |g: G, a: i32, facing: f32| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, a as _, facing as _); } };
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftCommon_InitGrab(fp: &mut Fighter, a: i32, v: f32) = |_, _, _| todo!("TODO(port) ftCommon_InitGrab (ftcommon.c)");
        Player_GetHandicap(slot: i32) -> i32 = |slot: i32| -> i32 { unsafe { crate::tu::player::Player_GetHandicap(slot as _) as _ } };
        Player_80033BB8(slot: i32) -> i32 = |slot: i32| -> i32 { unsafe { crate::tu::player::Player_80033BB8(slot as _) as _ } };
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ft_80084CB0(fp: &Fighter, b: &mut ftCollisionBox) = crate::ft084e::ft_80084cb0;
        lb_8000B1CC(j: *mut HSD_JObj, p0: *mut Vec3, p1: *mut Vec3) = |j: *mut HSD_JObj, p0: *mut Vec3, p1: *mut Vec3| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, p0 as _, p1 as _); } };
        ftParts_GetBoneIndex(fp: *mut Fighter, part: i32) -> i32 = |_, _| todo!("TODO(port) ftParts_GetBoneIndex (ftparts.c)");
        efSync_Spawn(id: i32, g: G, pos: *mut Vec3, s: *mut f32) = |_, _, _, _| todo!("TODO(port) efSync_Spawn (efsync.c)");
        ftCommon_GrabMash(fp: &mut Fighter, v: f32) -> bool = |_, _| todo!("TODO(port) ftCommon_GrabMash (ftcommon.c)");
        mpLib_80054ED8(line: i32) -> bool = |line: i32| -> bool { unsafe { (melee_world::tu::mplib::mpLib_80054ED8(line as _) != 0 as _) } };
        mpLineGetNormal(line: i32, n: *mut Vec3) -> *mut Vec3 = |_, _| todo!("TODO(port) mpLineGetNormal (mplib.c)");
        atan2f(y: f32, x: f32) -> f32 = |y: f32, x: f32| -> f32 { unsafe { melee_runtime::lbtrigf::atan2f(y as _, x as _) as _ } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = t!("HSD_JObjSetMtxDirtySub (jobj.c)");
        mpGetSpeed(line: i32, pos: *mut Vec3, speed: *mut Vec3) -> bool = |_, _, _| todo!("TODO(port) mpGetSpeed (mplib.c)");
        PSVECAdd(a: *mut Vec3, b: *mut Vec3, c: *mut Vec3) = |_, _, _| todo!("TODO(port) PSVECAdd (mtx)");
        ft_80082888(g: G, b: *mut ftCollisionBox) -> bool = |g: G, b: *mut ftCollisionBox| -> bool { unsafe { crate::tu::ft_081b::ft_80082888(g as _, b as _) as _ } };
        ftCo_800C5240(g: G) -> bool = t!("ftCo_800C5240 (ftCo_HammerWait.c)");
        ftCo_800C5A98(g: G) = t!("ftCo_800C5A98 (ftCo_HammerJump.c)");
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftColl_8007B7A4(g: G, v: i32) = |_, _| todo!("TODO(port) ftColl_8007B7A4 (ftcoll.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_Fall_Enter(g: G) = t!("ftCo_Fall_Enter (ftCo_Fall.c)");
        ftCo_SpecialAir_CheckInput(g: G) -> bool = t!("ftCo_SpecialAir_CheckInput (ftCo_SpecialAir.c)");
        ftCo_80095328(g: G, b: *mut i32) -> bool = |g: G, b: *mut i32| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_80095328(g as _, b as _) as _ } };
        ftCo_800D7100(g: G) -> bool = t!("ftCo_800D7100 (ftCo_HammerJump.c)");
        ftCo_800C3B10(g: G) -> bool = t!("ftCo_800C3B10 (ftCo_AirCatch.c)");
        ftCo_80099A58(g: G) -> bool = t!("ftCo_80099A58 (ftCo_EscapeAir.c)");
        ftCo_AttackAir_CheckItemThrowInput(g: G) -> bool = t!("ftCo_AttackAir_CheckItemThrowInput (ftCo_AttackAir.c)");
        ftCo_800D705C(g: G) -> bool = t!("ftCo_800D705C (ftCo_HammerJump.c)");
        ftCo_800CB870(g: G) -> bool = t!("ftCo_800CB870 (ftCo_JumpAerial.c)");
        ftCommon_Fall(fp: &mut Fighter, grav: f32, term: f32) = |_, _, _| todo!("TODO(port) ftCommon_Fall (ftcommon.c)");
        ftCommon_CalcSelfAccel_Drift(fp: &mut Fighter) = |_| todo!("TODO(port) ftCommon_CalcSelfAccel_Drift (ftcommon.c)");
        ft_80082D40(g: G, v: f32) = |g: G, v: f32| { unsafe { let _ = crate::tu::ft_081b::ft_80082d40(g as _, v as _); } };
        ftCo_AirCatchHit_Coll(g: G) = t!("ftCo_AirCatchHit_Coll (ft_081B.c)");
    }
}

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

pub fn ftCo_800C0874(g: G, _arg1: *mut core::ffi::c_void, arg2: i32) {
    if arg2 == BuryType_Unk2 as i32 {
        ext::ftCo_8009EB18(g);
    }
}

pub fn ftCo_800C08A0(g: G, arg1: G, arg2: *mut DynamicsDesc, arg3: i32) {
    let f = fp(g);
    let mut v = ext::ftColl_800765F0(f, core::ptr::null_mut(), unsafe { (*arg2).count } as f32);
    let f = fp(g);
    if arg3 == BuryType_Unk1 as i32 {
        f.bury_timer_1 = cd().bury_timer_unk1;
    } else if arg3 == BuryType_Unk3 as i32 {
        f.bury_timer_2 = cd().bury_timer_unk3;
    }
    if ext::ftColl_80076640(f, &mut v) {
        let f = fp(g);
        let hurt = &mut f.hurt_capsules[0] as *mut FighterHurtCapsule;
        ext::ftColl_80076764(3, arg3, arg1, arg2, f, hurt);
        let mut hit = SmallerHitCapsule::zeroed();
        let hp = &mut hit as *mut SmallerHitCapsule as *mut HitCapsule;
        ext::lbColl_80008D30(hp, arg2 as *mut lbColl_80008D30_arg1);
        ext::ftColl_80078384(fp(g), hurt, hp);
    }
    let f = fp(g);
    ext::pl_8003EC30(f.player_id as i32, f.is_sub_fighter() as i32, arg3, v);
}

fn bury_stage(k: i32) -> bool {
    k == Gr_Kind_OldKongo as i32 || k == Gr_Kind_Kongo as i32 || k == Gr_Kind_ShrineRoute as i32
        || k == Gr_Kind_Pushon as i32 || k == Gr_Kind_Zebes as i32
}

pub fn ftCo_800C09B4(g: G) {
    let k = ext::stage_info_grkind();
    let f = fp(g);
    f.bury_grkind = k as _;
    f.bury_timer_1 = 0;
    if bury_stage(f.bury_grkind as i32) {
        f.bury_timer_2 = 0;
    }
}

pub fn ftCo_800C0A28(g: G, _arg1: *mut core::ffi::c_void, arg2: i32) -> bool {
    let f = fp(g);
    if arg2 == BuryType_Unk2 as i32 {
        if f.bury_timer_2 != 0 || f.motion_id as i32 == ftCo_MS_BarrelWait as i32 || f.x1A6A & 0x20 != 0
            || f.x2224_b2() != 0
        {
            return false;
        }
    } else if arg2 == BuryType_Unk3 as i32 && f.bury_timer_2 != 0 {
        return false;
    }
    true
}

pub fn ftCo_800C0A98(g: G) {
    let f = fp(g);
    if f.bury_timer_1 != 0 {
        f.bury_timer_1 -= 1;
    }
    if bury_stage(f.bury_grkind as i32) && f.bury_timer_2 != 0 {
        f.bury_timer_2 -= 1;
    }
}

pub fn ftCo_800C0B20(g: G) {
    if fp(g).bury_timer_1 != 0 {
        return;
    }
    let ef = fp(g).coll_data.env_flags;
    let mut d: *mut DynamicsDesc = core::ptr::null_mut();
    if ef & (Collide_FloorPush | Collide_FloorHug) != 0 {
        d = ext::Ground_801C5700(fp(g).coll_data.floor.index);
    }
    let ef = fp(g).coll_data.env_flags;
    if ef & (Collide_CeilingPush | Collide_CeilingHug) != 0 {
        d = ext::Ground_801C5700(fp(g).coll_data.ceiling.index);
    }
    let ef = fp(g).coll_data.env_flags;
    if ef & Collide_RightWallMask != 0 {
        d = ext::Ground_801C5700(fp(g).coll_data.right_facing_wall.index);
    }
    let ef = fp(g).coll_data.env_flags;
    if ef & Collide_LeftWallMask != 0 {
        d = ext::Ground_801C5700(fp(g).coll_data.left_facing_wall.index);
    }
    if !d.is_null() {
        let mut v = ext::ftColl_800765F0(fp(g), core::ptr::null_mut(), unsafe { (*d).count } as f32);
        fp(g).bury_timer_1 = cd().bury_timer_unk1;
        if ext::ftColl_80076640(fp(g), &mut v) {
            let f = fp(g);
            let hurt = &mut f.hurt_capsules[0] as *mut FighterHurtCapsule;
            ext::ftColl_80076764(3, 1, core::ptr::null_mut(), d, f, hurt);
            let mut hit = HitCapsule::zeroed();
            ext::lbColl_80008D30(&mut hit, d as *mut lbColl_80008D30_arg1);
            ext::ftColl_80078384(fp(g), hurt, &mut hit);
        }
        let f = fp(g);
        ext::pl_8003EC30(f.player_id as i32, f.is_sub_fighter() as i32, 1, v);
    }
}

pub fn ftCo_800C0C88(arg0: i32) -> bool {
    arg0 == BuryType_Unk1 as i32 || arg0 == BuryType_Unk3 as i32
}

pub fn ftCo_800C0CB8(g: G) -> bool {
    let f = fp(g);
    if f.dmg.x1860_element == HitElement_Ground as u32 && f.ground_or_air as i32 == GA_Ground as i32
        && f.x2227_b6() == 0
    {
        ftCo_800C0D0C(g);
        return true;
    }
    false
}

pub fn ftCo_800C0D0C(g: G) {
    let mut joint_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut hip_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ftCo_ReleaseItemAndVictim(g);
    ext::ftCo_8008DCE0(g, -1, 0.0);
    ext::ftCommon_8007D7FC(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Bury as i32, (Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim) as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_8007E2FC(g);
    {
        let c = cd();
        let pid = fp(g).player_id as i32;
        let hc = ext::Player_GetHandicap(pid);
        let tmp2 = fmadds(c.x5FC, c.x600 - hc as f32, c.x5F8);
        let p = ext::Player_80033BB8(pid);
        // the C IR fuses all three products (llvm.fmuladd, inner first)
        let sum = fmadds(c.x604, c.x608 - p.wrapping_add(1) as f32, tmp2);
        let f = fp(g);
        let v = fmadds(f.dmg.x1830_percent, c.x60C, sum);
        ext::ftCommon_InitGrab(f, 0, v);
    }
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    let f = fp(g);
    f.set_x221D_b5(1);
    f.set_x2220_b3(1);
    f.set_x2224_b4(1);
    f.mv.co.bury.x0 = cd().x5F4;
    f.mv.co.bury.x20 = f.coll_data.floor.index as _;
    {
        let f = fp(g);
        let bx = unsafe { &mut *(&raw mut f.mv.co.bury.coll_box) };
        ext::ft_80084CB0(fp(g), bx);
    }
    ext::lb_8000B1CC(unsafe { (*fp(g).parts).joint }, core::ptr::null_mut(), &mut joint_pos);
    let idx = ext::ftParts_GetBoneIndex(fp(g), FtPart_HipN as i32);
    ext::lb_8000B1CC(unsafe { (*fp(g).parts.offset(idx as isize)).joint }, core::ptr::null_mut(), &mut hip_pos);
    {
        let y = hip_pos.y - joint_pos.y;
        let f = fp(g);
        f.mv.co.bury.x1C = y / cd().x5F4 as f32;
        f.mv.co.bury.translate = f.cur_pos;
        ext::efSync_Spawn(1095, g, &mut f.cur_pos, &mut f.x34_scale.y);
    }
    fp(g).set_x2219_b0(1);
}

pub fn ftCo_Bury_Anim(g: G) {
    let f = fp(g);
    f.grab_timer -= cd().x610;
    ext::ftCommon_GrabMash(fp(g), cd().x614);
    if fp(g).grab_timer <= 0.0 {
        ftCo_800C13BC(g);
    }
    let f = fp(g);
    unsafe {
        f.mv.co.bury.x0 = f.mv.co.bury.x0.wrapping_sub(1);
        if f.mv.co.bury.x0 == 0 {
            ftCo_800C124C(g);
        }
    }
}

pub fn ftCo_Bury_IASA(_g: G) {}

pub fn ftCo_800C0FCC(arg0: *mut HSD_GObj, arg1: G) {
    let line = unsafe { fp(arg1).mv.co.bury.x20 } as i32;
    if ext::mpLib_80054ED8(line) {
        let mut normal = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let jobj = unsafe { (*arg0).hsd_obj as *mut HSD_JObj };
        ext::mpLineGetNormal(line, &mut normal);
        let r = ext::atan2f(-normal.x, normal.y);
        HSD_JObjSetRotationZ(unsafe { &mut *jobj }, r, &mut Dirty);
        let f = fp(arg1);
        let tr = unsafe { &raw mut f.mv.co.bury.translate };
        if ext::mpGetSpeed(f.coll_data.floor.index, tr, &mut offset) {
            ext::PSVECAdd(tr, &mut offset, tr);
            HSD_JObjSetTranslate(unsafe { &mut *jobj }, unsafe { &*tr }, &mut Dirty);
        }
    }
}

pub fn ftCo_Bury_Phys(g: G) {
    let f = fp(g);
    let x1c = unsafe { f.mv.co.bury.x1C };
    let bx = unsafe { &mut f.mv.co.bury.coll_box };
    let m = if bx.right.y < bx.left.y { bx.right.y } else { bx.left.y };
    bx.bottom += x1c;
    if 1.0 + bx.bottom > m {
        bx.bottom = m - 1.0;
    }
}

pub fn ftCo_Bury_Coll(g: G) {
    let f = fp(g);
    let bx = unsafe { &raw mut f.mv.co.bury.coll_box };
    if !ext::ft_80082888(g, bx)
        || unsafe { fp(g).mv.co.bury.x20 } as i32 != fp(g).coll_data.floor.index
        || !ext::Ground_801C5700(unsafe { fp(g).mv.co.bury.x20 } as i32).is_null()
    {
        ftCo_800C13BC(g);
    }
}

pub fn ftCo_800C124C(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_BuryWait as i32,
        (Ft_MF_KeepGfx | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim) as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_8007E2FC(g);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    let f = fp(g);
    f.set_x221D_b5(1);
    f.set_x2220_b3(1);
    f.set_x2224_b4(1);
}

pub fn ftCo_BuryWait_Anim(g: G) {
    let f = fp(g);
    f.grab_timer -= cd().x610;
    ext::ftCommon_GrabMash(fp(g), cd().x614);
    if fp(g).grab_timer <= 0.0 {
        ftCo_800C13BC(g);
    }
}

pub fn ftCo_BuryWait_IASA(_g: G) {}

pub fn ftCo_BuryWait_Phys(_g: G) {}

pub fn ftCo_BuryWait_Coll(g: G) {
    ftCo_Bury_Coll(g);
}

pub fn ftCo_800C13BC(g: G) {
    if ext::ftCo_800C5240(g) {
        ext::ftCo_800C5A98(g);
        return;
    }
    ext::ftCommon_8007D5D4(fp(g));
    let f = fp(g);
    f.self_vel.x = 0.0;
    f.self_vel.y = cd().x618;
    f.mv.co.buryjump.x0 = 0.0;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_BuryJump as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftColl_8007B7A4(g, cd().x620);
}

pub fn ftCo_BuryJump_Anim(g: G) {
    let f = fp(g);
    unsafe { f.mv.co.buryjump.x0 += 1.0 };
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

pub fn ftCo_BuryJump_IASA(g: G) {
    if unsafe { fp(g).mv.ca.specials.grav } >= cd().x61C {
        if ext::ftCo_SpecialAir_CheckInput(g) { return; }
        if ext::ftCo_80095328(g, core::ptr::null_mut()) { return; }
        if ext::ftCo_800D7100(g) { return; }
        if ext::ftCo_800C3B10(g) { return; }
        if ext::ftCo_80099A58(g) { return; }
        if ext::ftCo_AttackAir_CheckItemThrowInput(g) { return; }
        if ext::ftCo_800D705C(g) { return; }
        if ext::ftCo_800CB870(g) { return; }
    }
}

pub fn ftCo_BuryJump_Phys(g: G) {
    let f = fp(g);
    let (gr, tv) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    ext::ftCommon_Fall(f, gr, tv);
    ext::ftCommon_CalcSelfAccel_Drift(fp(g));
}

pub fn ftCo_BuryJump_Coll(g: G) {
    let vy = fp(g).self_vel.y;
    if vy >= 0.0 {
        ext::ft_80082D40(g, vy);
    } else {
        ext::ftCo_AirCatchHit_Coll(g);
    }
}
