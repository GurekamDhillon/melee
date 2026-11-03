#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureCut.c (grab release: the CaptureCut
//! state, the shared "cut the grab" ftCo_800DC920 / ftCo_800DCE34 and the grab-release hit
//! ftCo_800DCFD4). TARGET_PC paths (NULL-partner guard, ScriptGame hit rules, FTKB_IS_KIRBY m-ex).
//! Fused sites (tools/rust-port/fma_sites.py): ftCo_800DC920 has two f32 fmuladd (the XRotN
//! position offsets), lowered to fmadds. Calls out of the TU go through `ext` (unported ones
//! panic until rewired). Smoke-tested by tests/tu_ftCo_CaptureCut.rs.

use crate::types::*;
use melee_core::fp::fmadds;
use melee_types::inline as ji;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj<'a>(g: G) -> &'a mut HSD_JObj {
    unsafe { &mut *((*g).hsd_obj as *mut HSD_JObj) }
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
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_8007D92C(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007d92c(a0);
        ftCommon_CalcGroundAccel_Deaccel(f: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |a0| crate::ftcommon_c::ftCommon_SetSelfMovementFromGroundedMovement(a0);
        ft_80084DB0(g: G) = |a0| crate::tu::ft_081b::ft_80084db0(a0);
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ft_80083910(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80083910(g as _, ev)); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_UnlockECB(f: &mut Fighter) = crate::ftcommon2::unlock_ecb;
        Fighter_UnkSetFlag_8006CFBC(g: G) = |g: G| { unsafe { crate::tu::fighter::Fighter_UnkSetFlag_8006CFBC(g); } };
        Fighter_UpdateModelScale(g: G) = |g: G| { unsafe { crate::tu::fighter::Fighter_UpdateModelScale(g as _); } };
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3) = |j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } };
        lb_8000C390(j: *mut HSD_JObj) = |j: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C390(j as _); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |a0| unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(a0 as _) };
        mpLib_80054ED8(index: i32) -> i32 = |index: i32| -> i32 { unsafe { melee_world::tu::mplib::mpLib_80054ED8(index as _) } };
        mpLib_8005199C_Floor(pos: &mut Vec3, a: i32, b: i32) -> i32 = |pos: &mut Vec3, a: i32, b: i32| -> i32 { unsafe { melee_world::tu::mplib::mpLib_8005199C_Floor(pos as *mut _ as *mut _, a as _, b as _) } };
        mpLinesConnected(a: i32, b: i32) -> bool = |a: i32, b: i32| -> bool { unsafe { (melee_world::tu::mplib::mpLinesConnected(a as _, b as _) != 0 as _) } };
        mpLib_8004DD90_Floor(index: i32, pos: &mut Vec3, out: &mut f32, a: i32, b: *mut Vec3) -> bool = |a0, a1, a2, a3, a4| unsafe { melee_world::tu::mplib::mpLib_8004DD90_Floor(a0, a1 as *mut _, a2 as *mut _, a3 as _, a4 as _) != 0 };
        mpColl_80043680(coll: &mut CollData, pos: &Vec3) = |coll: &mut CollData, pos: &Vec3| { unsafe { melee_world::tu::mpcoll::mpColl_80043680(coll as *mut _ as *mut _, pos as *const _ as *mut _); } };
        // rp3 mpColl_80043670
        mpColl_80043670(coll: &mut CollData) = |coll: &mut CollData| unsafe { melee_world::tu::mpcoll::mpColl_80043670(coll as _); };
        // rp3-end
        // rp3 mpColl_80048654
        mpColl_80048654(coll: &mut CollData) -> bool = |coll: &mut CollData| -> bool { unsafe { (melee_world::tu::mpcoll::mpColl_80048654(coll as _) != 0) } };
        // rp3-end
        // rp3 mpColl_800477E0
        mpColl_800477E0(coll: &mut CollData) -> bool = |coll: &mut CollData| -> bool { unsafe { (melee_world::tu::mpcoll::mpColl_800477E0(coll as _) != 0) } };
        // rp3-end
        Mex_Enabled(name: &str) -> i32 = |name: &str| -> i32 { unsafe { crate::platform::Mex_Enabled(name) as i32 } };
        ftColl_8007B62C(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b62c(g as _, n as _); } };
        ftColl_8007891C(victim: G, g: G, damage: f32) = |victim: G, g: G, damage: f32| { unsafe { crate::tu::ftcoll::ftcoll_8007891c(victim as _, g as _, damage as _); } };
        ftColl_80079C70(f: &mut Fighter, victim: &mut Fighter, hit: *mut HitCapsule, count: i32) -> f32 = |f: &mut Fighter, victim: &mut Fighter, hit: *mut HitCapsule, count: i32| -> f32 { unsafe { crate::tu::ftcoll::ftcoll_80079c70(f as *mut _ as *mut _, victim as *mut _ as *mut _, hit as _, count as _) } };
        ftColl_80078710(victim: G, g: G, facing: *mut f32) = |victim: G, g: G, facing: *mut f32| { unsafe { crate::tu::ftcoll::ftcoll_80078710(victim as _, g as _, facing as _); } };
        ScriptGame_FighterDealtDamage(f: *mut Fighter, damage: f32) -> f32 = |_, _| todo!("TODO(pc script) ScriptGame_FighterDealtDamage");
        ScriptGame_HitRuleContact(hit: *mut HitCapsule, f: *mut Fighter, value: f32, kb: i32) -> f32 = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleContact");
        ScriptGame_HitRuleReport(att: *mut Fighter, vic: *mut Fighter, hit: *mut HitCapsule, damage: f32) = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleReport");
        ScriptGame_HitRulePercentQueue(hit: *mut HitCapsule, f: *mut Fighter, damage: f32) = |_, _, _| todo!("TODO(pc script) ScriptGame_HitRulePercentQueue");
        Fighter_UnkTakeDamage_8006CC30(f: &mut Fighter, damage: f32) = |f: &mut Fighter, damage: f32| { unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(f as _, damage as _); } };
        ftCo_Damage_CalcKnockback(f: &mut Fighter) = |f: &mut Fighter| { unsafe { crate::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(f as _); } };
        ftCo_8008E908(g: G, f: f32) = |g: G, f: f32| { unsafe { crate::tu::ftCo_Damage::ftCo_8008E908(g as _, f as _); } };
    }
}

/// The jobj.h inline dirty marks.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}
/// `HSD_JObjSetTranslate` (jobj.h inline).
pub(crate) fn set_translate(j: *mut HSD_JObj, t: &Vec3) {
    melee_core::ct!("set_translate");
    ji::HSD_JObjSetTranslate(unsafe { &mut *j }, t, &mut Dirty);
}

fn xrotn_joint(f: &Fighter) -> *mut HSD_JObj {
    let i = ext::ftParts_GetBoneIndex(f, FtPart_XRotN as i32);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// `FTKB_IS_KIRBY` (TARGET_PC form).
fn is_kirby(k: i32) -> bool {
    k == Ft_Kind_Kirby as i32
        || (k >= Ft_Kind_Mex0 as i32 && k < Ft_Kind_Max as i32 && ext::ftKb_MexKirbyLike(k) != 0)
}

/// `ftCo_CaptureCut_Enter`.
pub fn ftCo_CaptureCut_Enter(g: G) {
    melee_core::ct!("ftCo_CaptureCut_Enter");
    // mod host site (hooks-v1 0x800DC750, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800DC750, [melee_core::modhook::Arg::Fighter(g as _)], ftCo_CaptureCut_Enter__orig(g));
    #[allow(unused_unsafe)] unsafe { ftCo_CaptureCut_Enter__orig(g) }
}

fn ftCo_CaptureCut_Enter__orig(g: G) {
    let f = fp(g);
    ftCo_800DC920(f.victim_gobj, g);
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        f.gr_vel = -f.facing_dir * cd().x370;
    } else {
        f.self_vel.x = -f.facing_dir * cd().x370;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureCut as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

/// `ftCo_CaptureCut_Anim`.
pub fn ftCo_CaptureCut_Anim(g: G) {
    melee_core::ct!("ftCo_CaptureCut_Anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

/// `ftCo_CaptureCut_IASA`.
pub fn ftCo_CaptureCut_IASA(_g: G) { melee_core::ct!("ftCo_CaptureCut_IASA");}

/// `ftCo_CaptureCut_Phys`.
pub fn ftCo_CaptureCut_Phys(g: G) {
    melee_core::ct!("ftCo_CaptureCut_Phys");
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        let fr = cd().x36C * f.co_attrs.ground_friction;
        ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
        ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
        return;
    }
    ext::ft_80084DB0(g);
}

/// `ftCo_CaptureCut_Coll`.
pub fn ftCo_CaptureCut_Coll(g: G) {
    melee_core::ct!("ftCo_CaptureCut_Coll");
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_800841B8(g, Some(fn_800DC8D8));
        return;
    }
    ext::ft_80083910(g, Some(fn_800DC8FC));
}

/// `fn_800DC8D8`.
pub unsafe extern "C" fn fn_800DC8D8(g: G) {
    melee_core::ct!("fn_800DC8D8");
    ext::ftCommon_8007D5D4(fp(g));
}

/// `fn_800DC8FC`.
pub unsafe extern "C" fn fn_800DC8FC(g: G) {
    melee_core::ct!("fn_800DC8FC");
    ext::ftCommon_8007D7FC(fp(g));
}

/// `ftCo_800DC920`: cut the grab between `arg0` and `gobj` (the grabbed one is restored and
/// placed back onto the stage).
pub fn ftCo_800DC920(arg0: G, gobj: G) {
    melee_core::ct!("ftCo_800DC920");
    let mut sp4c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp48: f32 = 0.0;
    let mut r31: *mut Fighter = fp(arg0);
    ext::Fighter_UnkSetFlag_8006CFBC(arg0);
    ext::Fighter_UnkSetFlag_8006CFBC(gobj);
    let r30: *mut Fighter;
    if unsafe { (*r31).anon_221b.x221B_b7() } != 0 {
        r30 = r31;
        r31 = fp(gobj);
    } else {
        r30 = fp(gobj);
    }
    let (a, b) = unsafe { (&mut *r31, &mut *r30) };
    a.x1A5C = core::ptr::null_mut();
    let mut placed = false;
    a.victim_gobj = core::ptr::null_mut();
    a.anon_221b.set_x221B_b5(0);
    a.anon_221b.set_x221B_b7(0);
    if b.x2226_b2() != 0 {
        if is_kirby(a.kind as i32)
            && (a.motion_id == ftKb_MS_SpecialNCapture0 as i32 || a.motion_id == ftKb_MS_SpecialAirNCapture0 as i32)
        {
            ext::Fighter_UpdateModelScale(b.gobj);
            b.set_x2226_b2(0);
            sp4c = b.cur_pos;
        } else if b.x2226_b2() != 0 {
            let j = xrotn_joint(b);
            ext::lb_8000B1CC(j, core::ptr::null(), &mut sp4c);
            sp4c.x = fmadds(b.facing_dir, b.x1A70.z * b.x34_scale.y, sp4c.x);
            sp4c.y = fmadds(b.x1A70.y, b.x34_scale.y, sp4c.y);
            sp4c.z = 0.0;
            ext::lb_8000C390(j);
            b.set_x2226_b2(0);
            let t = b.x2174;
            set_translate(j, &t);
        }
        if ext::mpLib_80054ED8(a.coll_data.floor.index as i32) != 0 {
            let line = ext::mpLib_8005199C_Floor(&mut sp4c, -1, -1);
            if line != -1 {
                let tmp = a.coll_data.floor.index as i32;
                if ext::mpLinesConnected(line, tmp) {
                    b.coll_data.floor.index = line as _;
                    ext::mpLib_8004DD90_Floor(line, &mut sp4c, &mut sp48, 0, core::ptr::null_mut());
                    if sp48 >= cd().x3BC {
                        b.cur_pos.x = sp4c.x;
                        b.cur_pos.y = sp4c.y + sp48;
                        b.cur_pos.z = sp4c.z;
                        let p = b.cur_pos;
                        ext::mpColl_80043680(&mut b.coll_data, &p);
                        placed = true;
                    }
                }
            }
        }
        if !placed {
            let j = unsafe { (*b.gobj).hsd_obj as *mut HSD_JObj };
            let ty = 0.5f32 * (a.coll_data.ecb.top.y + a.coll_data.ecb.bottom.y);
            let sp3c = Vec3 { x: a.cur_pos.x + 0.0, y: a.cur_pos.y + ty, z: a.cur_pos.z + 0.0 };
            b.coll_data.last_pos = sp3c;
            ext::mpColl_80043670(&mut b.coll_data);
            b.coll_data.cur_pos = sp4c;
            set_translate(j, &sp4c);
            let hit = if b.ground_or_air == GA_Ground {
                ext::mpColl_80048654(&mut b.coll_data)
            } else {
                ext::ftCommon_UnlockECB(b);
                ext::mpColl_800477E0(&mut b.coll_data)
            };
            if hit {
                if b.ground_or_air != GA_Ground {
                    ext::ftCommon_8007D7FC(b);
                }
                b.cur_pos = b.coll_data.cur_pos;
            } else {
                if b.ground_or_air != GA_Air {
                    ext::ftCommon_8007D5D4(b);
                }
                b.cur_pos = sp4c;
            }
            let p = b.cur_pos;
            set_translate(j, &p);
        }
    }
    b.x1A5C = core::ptr::null_mut();
    b.victim_gobj = core::ptr::null_mut();
    b.anon_221b.set_x221B_b5(0);
    b.anon_221b.set_x221B_b7(0);
}

/// `ftCo_800DCE34` (TARGET_PC: a NULL partner means the grab was already cut).
pub fn ftCo_800DCE34(gobj0: G, gobj1: G) {
    melee_core::ct!("ftCo_800DCE34");
    if (gobj0.is_null() || gobj1.is_null()) && ext::Mex_Enabled("test_no_capturecut_guard") == 0 {
        return;
    }
    ftCo_800DC920(gobj0, gobj1);
    let fp1 = fp(gobj1);
    if fp1.anon_221b.x221B_b7() != 0 {
        let p = fp(gobj0).cur_pos;
        set_translate(jobj(gobj0), &p);
    } else {
        let p = fp1.cur_pos;
        set_translate(jobj(gobj1), &p);
    }
}

/// `ftCo_800DCFD4`: the grab-release hit (the grabber's second hitbox).
pub fn ftCo_800DCFD4(g: G) {
    melee_core::ct!("ftCo_800DCFD4");
    let f = fp(g);
    let vg = f.victim_gobj;
    let vf = fp(vg);
    let hit: *mut HitCapsule = &mut vf.xDF4[1];
    let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if f.x1988 != 0 {
        ext::ftColl_8007B62C(g, 0);
    }
    ext::ftCommon_8007D5D4(f);
    ext::ftColl_8007891C(vg, g, unsafe { (*hit).damage as f32 });
    ext::lb_8000B1CC(xrotn_joint(f), core::ptr::null(), &mut sp18);
    f.dmg.kb_applied = ext::ftColl_80079C70(f, vf, hit, unsafe { (*hit).unk_count as i32 });
    unsafe {
        f.dmg.x1848_kb_angle = (*hit).kb_angle as _;
        f.dmg.facing_dir_1 = -vf.facing_dir;
        f.dmg.x184c_damaged_hurtbox = 1;
        f.dmg.x1854_collpos = sp18;
        f.dmg.x1860_element = (*hit).element as _;
    }
    ext::ftColl_80078710(vg, g, &mut f.dmg.facing_dir_1);
    let (vp, pp): (*mut Fighter, *mut Fighter) = (vf, f);
    let contact = || {
        let dealt = ext::ScriptGame_FighterDealtDamage(vp, unsafe { (*hit).damage as f32 });
        ext::ScriptGame_HitRuleContact(hit, pp, dealt, 0)
    };
    let c = contact();
    ext::ScriptGame_HitRuleReport(vp, pp, hit, c);
    let c = contact();
    ext::ScriptGame_HitRulePercentQueue(hit, pp, c);
    let c = contact();
    ext::Fighter_UnkTakeDamage_8006CC30(fp(g), c);
    ext::ftCo_Damage_CalcKnockback(fp(g));
    ext::ftCo_8008E908(g, 0.0);
}
