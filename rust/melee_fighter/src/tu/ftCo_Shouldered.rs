#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Shouldered.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_Shouldered.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007E2FC(g: G) = |_| todo!("ftCommon_8007E2FC: installed by melee_link");
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_800DB464(g: G) = crate::tu::ftCo_CapturePulled::ftCo_800DB464;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, v: f32) = crate::ftcommon2::init_grab;
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ftCo_800DC920(a: G, b: G) = crate::tu::ftCo_CaptureCut::ftCo_800DC920;
        ftParts_GetBoneIndex(f: *mut Fighter, part: i32) -> i32 = |_, _| todo!("ftParts_GetBoneIndex: installed by melee_link");
        lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, out: &mut Vec3) = |jobj: *mut HSD_JObj, pos0: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, out as _); } };
        ftColl_80079C70(f: *mut Fighter, victim: *mut Fighter, hit: *mut HitCapsule, count: i32) -> f32 = |f: *mut Fighter, victim: *mut Fighter, hit: *mut HitCapsule, count: i32| -> f32 { unsafe { crate::tu::ftcoll::ftcoll_80079c70(f as _, victim as _, hit as _, count as _) } };
        ftColl_80078710(victim: G, g: G, facing: *mut f32) = |victim: G, g: G, facing: *mut f32| { unsafe { crate::tu::ftcoll::ftcoll_80078710(victim as _, g as _, facing as _); } };
        // TARGET_PC `FT_SCRIPT_DEALT(fp, base)`.
        ScriptGame_FighterDealtDamage(f: *mut Fighter, damage: f32) -> f32 = |_, _| todo!("TODO(pc script) ScriptGame_FighterDealtDamage");
        ScriptGame_HitRuleContact(hit: *mut HitCapsule, f: *mut Fighter, value: f32, kb: i32) -> f32 = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleContact");
        ScriptGame_HitRuleReport(attacker: *mut Fighter, victim: *mut Fighter, hit: *mut HitCapsule, damage: f32) = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleReport");
        ScriptGame_HitRulePercentQueue(hit: *mut HitCapsule, victim: *mut Fighter, damage: f32) = |_, _, _| todo!("TODO(pc script) ScriptGame_HitRulePercentQueue");
        // rp3 Fighter_UnkTakeDamage_8006CC30
        Fighter_UnkTakeDamage_8006CC30(f: *mut Fighter, dmg: f32) = |f: *mut Fighter, dmg: f32| unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(&mut *(f as *mut _), dmg); };
        // rp3-end
        // rp3 ftCo_Damage_CalcKnockback
        ftCo_Damage_CalcKnockback(f: *mut Fighter) = |f: *mut Fighter| unsafe { crate::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(&mut *(f as *mut _)); };
        // rp3-end
        ftCo_8008E908(g: G, v: f32) = |g: G, v: f32| { unsafe { crate::tu::ftCo_Damage::ftCo_8008E908(g as _, v as _); } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
    }
}

/// `ftCo_800DB464` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DB464_cb(g: G) {
    melee_core::ct!("ftCo_800DB464_cb");
    ext::ftCo_800DB464(g)
}

pub fn ftCo_8009C5A4(g: G, msid: i32) {
    melee_core::ct!("ftCo_8009C5A4");
    let f = fp(g);
    if f.motion_id as i32 != msid || msid != ftCo_MS_ShoulderedWait as i32 {
        let flags = if f.x2222_b6() != 0 { Ft_MF_FreezeState } else { Ft_MF_None };
        ext::Fighter_ChangeMotionState(g, msid, flags as u32, 0.0, 1.0, 0.0, f.victim_gobj);
        ext::ftCommon_8007E2FC(g);
        ext::ftCommon_8007E2F4(fp(g), 0x1FF);
        fp(g).accessory1_cb = Some(ftCo_800DB464_cb);
    }
}

pub fn ftCo_8009C640(g: G, msid: i32) {
    melee_core::ct!("ftCo_8009C640");
    let f = fp(g);
    let c = cd();
    // `(int) (percent * x4A0 + x4A4)`: one fused site (fmadds), saturating conversion.
    let timer = melee_core::fp::fmadds(f.dmg.x1830_percent, c.x4A0, c.x4A4) as i32;
    ext::ftCommon_InitGrab(f, 0, timer as f32);
    let f = fp(g);
    unsafe {
        f.mv.co.shouldered.x0 = 0.0;
        f.mv.co.shouldered.x4 = 0;
    }
    ftCo_8009C5A4(g, msid);
}

/// The throw-release hit shared by `ftCo_8009C744` and `ftCo_Shouldered_Anim`: `victim` takes
/// `attacker`'s `xDF4[1]` at bone `part`, facing `facing`, then is knocked back; `c1`/`c2` are
/// ftColl_80078710's gobj arguments (the two call sites pass them in different orders).
fn release_hit(victim_g: G, victim: *mut Fighter, attacker: *mut Fighter, c1: G, c2: G, part: i32, facing: impl Fn(&Fighter, &Fighter) -> f32) {
    unsafe {
        let hit: *mut HitCapsule = &mut (*attacker).xDF4[1];
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let bone = ext::ftParts_GetBoneIndex(victim, part);
        ext::lb_8000B1CC((*(*victim).parts.offset(bone as isize)).joint, core::ptr::null(), &mut pos);
        (*victim).dmg.kb_applied = ext::ftColl_80079C70(victim, attacker, hit, (*hit).unk_count as i32);
        (*victim).dmg.x1848_kb_angle = (*hit).kb_angle;
        (*victim).dmg.facing_dir_1 = facing(&*victim, &*attacker);
        (*victim).dmg.x184c_damaged_hurtbox = 1;
        (*victim).dmg.x1854_collpos = pos;
        (*victim).dmg.x1860_element = (*hit).element;
        ext::ftColl_80078710(c1, c2, &mut (*victim).dmg.facing_dir_1);
        let d = ext::ScriptGame_FighterDealtDamage(attacker, (*hit).damage);
        let d = ext::ScriptGame_HitRuleContact(hit, victim, d, 0);
        ext::ScriptGame_HitRuleReport(attacker, victim, hit, d);
        let d = ext::ScriptGame_FighterDealtDamage(attacker, (*hit).damage);
        let d = ext::ScriptGame_HitRuleContact(hit, victim, d, 0);
        ext::ScriptGame_HitRulePercentQueue(hit, victim, d);
        let d = ext::ScriptGame_FighterDealtDamage(attacker, (*hit).damage);
        let d = ext::ScriptGame_HitRuleContact(hit, victim, d, 0);
        ext::Fighter_UnkTakeDamage_8006CC30(victim, d);
        ext::ftCo_Damage_CalcKnockback(victim);
        ext::ftCo_8008E908(victim_g, 0.0);
    }
}

/// static `ftCo_8009C744`.
pub fn ftCo_8009C744(g: G) {
    melee_core::ct!("ftCo_8009C744");
    let f = fp(g) as *mut Fighter;
    unsafe {
        let vic_gobj = (*f).victim_gobj;
        let vic_fp = (*vic_gobj).user_data as *mut Fighter;
        ext::ftCo_800DC920((*f).victim_gobj, g);
        release_hit(g, f, vic_fp, vic_gobj, g, FtPart_XRotN as i32, |_, a| -a.facing_dir);
    }
}

/// static inline `inlineA0`.
fn inline_a0(g: G) -> f32 {
    let c = cd();
    if fp(g).ground_or_air == GA_Ground as _ { c.x4A8 } else { c.x4A8 * c.x4AC }
}

pub fn ftCo_Shouldered_Anim(g: G) {
    melee_core::ct!("ftCo_Shouldered_Anim");
    let v = inline_a0(fp(g).victim_gobj);
    let mash = ext::ftCommon_GrabMash(fp(g), v);
    unsafe { fp(g).mv.co.shouldered.x4 = mash as i32 };
    if fp(g).grab_timer <= 0.0 {
        unsafe {
            let gobj1 = fp(g).victim_gobj;
            let fp1 = (*gobj1).user_data as *mut Fighter;
            let fp2 = (*(*fp1).victim_gobj).user_data as *mut Fighter;
            let other = (*fp1).victim_gobj;
            release_hit(gobj1, fp1, fp2, gobj1, other, FtPart_TransN2 as i32, |v, _| v.facing_dir);
        }
        ftCo_8009C744(g);
        return;
    }
    let f = fp(g);
    unsafe {
        if f.mv.co.shouldered.x0 != 0.0 {
            f.mv.co.shouldered.x0 -= 1.0;
            if f.mv.co.shouldered.x0 <= 0.0 && f.mv.co.shouldered.x4 == 0 {
                ext::ftAnim_SetAnimRate(g, 1.0);
                fp(g).mv.co.shouldered.x0 = 0.0;
            }
        }
        let f = fp(g);
        if f.mv.co.shouldered.x0 <= 0.0 && f.mv.co.shouldered.x4 != 0 && f.motion_id == ftCo_MS_ShoulderedWait as _ {
            f.mv.co.shouldered.x0 = cd().x3B0;
            ext::ftAnim_SetAnimRate(g, cd().shouldered_anim_rate);
        }
    }
}

pub fn ftCo_Shouldered_IASA(_g: G) { melee_core::ct!("ftCo_Shouldered_IASA");}

pub fn ftCo_Shouldered_Phys(_g: G) { melee_core::ct!("ftCo_Shouldered_Phys");}

pub fn ftCo_Shouldered_Coll(_g: G) { melee_core::ct!("ftCo_Shouldered_Coll");}
