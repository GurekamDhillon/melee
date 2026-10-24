#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Guard.c (shield: GuardOn / Guard / GuardOff /
//! GuardSetOff / GuardReflect, powershield, shield SDI), TARGET_PC build: the script caps gate,
//! FT_SCRIPT_VALUE, the Geno guard-pose tree and UCF 0.84 shield SDI included. Every function of
//! the TU; the static inline helpers that differ from a named function are ported as private fns,
//! the ones with identical bodies call the named function. Calls out go through `ext`.
//! Checked against the C by tests/tu_ftCo_Guard.rs.

use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::cabi::{f2i32, f2u8};

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
    ext_fns! {
        // TARGET_PC `FT_CAPS_ACTION_ALLOWED(fp, flag)` / `FT_SCRIPT_VALUE(fp, field, base)`.
        ScriptGame_CapsActionAllowed(fp: &mut Fighter, flag: i32) -> i32 = |_, _| todo!("TODO(port) ScriptGame_CapsActionAllowed (pc/gameworld/script_game.c)");
        ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 = |_, _, _| todo!("TODO(port) ScriptGame_ModValue (pc/gameworld/script_game.c)");
        lb_8000D008(y: f32, x: f32) -> f32 = |y: f32, x: f32| -> f32 { unsafe { melee_runtime::tu::lb_00CE::lb_8000D008(y as _, x as _) as _ } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (baselib/jobj.c)");
        efLib_SetParamAlpha(g: G, a: u8) = |_, _| todo!("TODO(port) efLib_SetParamAlpha (ef/eflib.c)");
        GenoDefine_RestPoseTree(fp: &mut Fighter) -> *mut HSD_Joint = |_| todo!("TODO(port) GenoDefine_RestPoseTree (pc/geno)");
        Geno_MotionAnimRow(fp: &mut Fighter, motion: i32) -> i32 = |_, _| todo!("TODO(port) Geno_MotionAnimRow (pc/geno)");
        ftData_80085E50(fp: &mut Fighter, msid: i32) -> *mut FigaTree = |fp: &mut Fighter, msid: i32| -> *mut FigaTree { unsafe { crate::tu::ftdata::ftData_80085E50(fp as _, msid as _) as _ } };
        ftAnim_8006F4C8(fp: &mut Fighter, b: bool, tree: *mut FigaTree) = |fp: &mut Fighter, b: bool, tree: *mut FigaTree| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006F4C8(fp as _, b as _, tree as _); } };
        ftAnim_80070710(j: *mut HSD_JObj, frame: f32) = |j: *mut HSD_JObj, frame: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_80070710(j as _, frame as _); } };
        ftAnim_8006FB88(fp: &mut Fighter, part: u32, j: *mut HSD_Joint) = |fp: &mut Fighter, part: u32, j: *mut HSD_Joint| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006FB88(fp as _, part as _, j as _); } };
        HSD_JObjAnimAll(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjAnimAll (baselib/jobj.c)");
        // `OSReport("geno: kind %d: shield guard pose samples own Guard row %d\n", kind, row)`.
        OSReport_GuardPose(kind: i32, row: i32) = |_, _| ();
        ftAnim_80070108(fp: &mut Fighter, part: u32, a: f32, b: f32, j: *mut HSD_Joint) = |fp: &mut Fighter, part: u32, a: f32, b: f32, j: *mut HSD_Joint| { unsafe { let _ = crate::tu::ftanim::ftAnim_80070108(fp as _, part as _, a as _, b as _, j as _); } };
        ftAnim_8006FE9C(fp: &mut Fighter, part: u32, a: f32, b: f32) = |fp: &mut Fighter, part: u32, a: f32, b: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006FE9C(fp as _, part as _, a as _, b as _); } };
        ftAnim_8006FF74(fp: &mut Fighter, part: u32) = |fp: &mut Fighter, part: u32| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006FF74(fp as _, part as _); } };
        ftAnim_80070010(fp: &mut Fighter, part: u32, a: f32, b: f32, j: *mut HSD_Joint) = |fp: &mut Fighter, part: u32, a: f32, b: f32, j: *mut HSD_Joint| { unsafe { let _ = crate::tu::ftanim::ftAnim_80070010(fp as _, part as _, a as _, b as _, j as _); } };
        ftAnim_8006FA58(fp: &mut Fighter, part: u32, j: *mut HSD_Joint) = |fp: &mut Fighter, part: u32, j: *mut HSD_Joint| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006FA58(fp as _, part as _, j as _); } };
        Player_GetUnk45(slot: i32) -> u32 = |slot: i32| -> u32 { unsafe { crate::tu::player::Player_GetUnk45(slot as _) as _ } };
        Fighter_804D650C() -> *mut u8 = || todo!("TODO(port) Fighter_804D650C global (fighter.c)");
        // `efSync_Spawn(gfx_id, gobj, jobj, color)` (variadic in C).
        efSync_Spawn(id: i32, g: G, j: *mut HSD_JObj, color: i32) = |_, _, _, _| todo!("TODO(port) efSync_Spawn (ef/efsync.c)");
        ftColl_8007B1B8(g: G, desc: &mut AbsorbDesc, cb: HSD_GObjEvent) = |_, _, _| todo!("TODO(port) ftColl_8007B1B8 (ftcoll.c)");
        ft_PlaySFX(fp: &mut Fighter, id: i32, vol: u8, pan: u8) = |_, _, _, _| todo!("TODO(port) ft_PlaySFX (ft_0881.c)");
        Geno_DefineTemplateSkeleton(kind: i32) -> i32 = |_| todo!("TODO(port) Geno_DefineTemplateSkeleton (pc/geno)");
        Geno_DefineBaseKind(kind: i32) -> i32 = |_| todo!("TODO(port) Geno_DefineBaseKind (pc/geno)");
        ftYs_Init_8012BECC(g: G) = |_| todo!("TODO(port) ftYs_Init_8012BECC (ftYoshi)");
        ftYs_Shield_8012C1D4(g: G) = |_| todo!("TODO(port) ftYs_Shield_8012C1D4 (ftYoshi)");
        ftYs_Shield_8012C49C(g: G) = |_| todo!("TODO(port) ftYs_Shield_8012C49C (ftYoshi)");
        ftYs_Shield_8012C600(g: G, b: bool) = |_, _| todo!("TODO(port) ftYs_Shield_8012C600 (ftYoshi)");
        ftYs_Shield_8012C850(g: G) = |_| todo!("TODO(port) ftYs_Shield_8012C850 (ftYoshi)");
        ftYs_Shield_8012C914(g: G) = |_| todo!("TODO(port) ftYs_Shield_8012C914 (ftYoshi)");
        ftParts_80074B0C(g: G, model: i32, v: i32) = |g: G, model: i32, v: i32| { unsafe { let _ = crate::tu::ftparts::ftParts_80074B0C(g as _, model as _, v as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        pl_8003E0E8(slot: i32, sub: i32) = |slot: i32, sub: i32| { unsafe { let _ = crate::tu::plbonuslib::pl_8003E0E8(slot as _, sub as _); } };
        pl_8003E114(slot: i32, sub: i32, v: f32) = |slot: i32, sub: i32, v: f32| { unsafe { let _ = crate::tu::plbonuslib::pl_8003E114(slot as _, sub as _, v as _); } };
        ftCo_80098B20(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_ShieldBreakFly::ftCo_80098B20(g as _); } };
        ftCo_8009515C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_ItemThrow::ftCo_8009515C(g as _) as _ } };
        ftCo_8009980C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Escape::ftCo_8009980C(g as _) as _ } };
        ftCo_8009917C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Escape::ftCo_8009917C(g as _) as _ } };
        ftCo_800D8B9C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_800D8B9C(g as _) as _ } };
        ftCo_Catch_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Catch::ftCo_Catch_CheckInput(g as _) as _ } };
        ftCo_800CB024(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Jump::ftCo_800CB024(g as _) as _ } };
        ftCo_8009A080(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Pass::ftCo_8009A080(g as _) as _ } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftColl_8007AEE0(g: G) = |g: G| { unsafe { let _ = crate::ft0df::ftcoll_8007aee0(g as _); } };
        ftColl_8007AEF8(g: G) = |g: G| { unsafe { let _ = crate::ft0df::ftcoll_8007aef8(g as _); } };
        ft_800845B4(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_800845b4(g as _); } };
        ft_80084104(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_081b::ft_80084104(g as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput(g as _) as _ } };
        ftCo_800D6824(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D6824(g as _) as _ } };
        ftCo_800D68C0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack100::ftCo_800D68C0(g as _) as _ } };
        ftCo_AttackS4_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput(g as _) as _ } };
        ftCo_AttackHi4_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput(g as _) as _ } };
        ftCo_AttackLw4_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput(g as _) as _ } };
        ftCo_AttackS3_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput(g as _) as _ } };
        ftCo_AttackHi3_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput(g as _) as _ } };
        ftCo_AttackLw3_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput(g as _) as _ } };
        ftCo_Attack1_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput(g as _) as _ } };
        ftCo_800C36DC(g: G) = |g: G| { unsafe { let _ = crate::tu::ftco_0c35::ftco_800c36dc(g as _); } };
        ftAnim_SetAnimRate(g: G, v: f32) = |g: G, v: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_SetAnimRate(g as _, v as _); } };
        lbGetJObjEndFrame(j: *mut HSD_JObj) -> f32 = |j: *mut HSD_JObj| -> f32 { unsafe { melee_runtime::tu::lb_00B0::lbGetJObjEndFrame(j as _) as _ } };
        Replay_UcfVersion(port: i32) -> i32 = |_| todo!("TODO(port) Replay_UcfVersion (pc/platform)");
        ftUcf_PadRaw(port: i32, which_y: i32, back: i32) -> i32 = |_, _, _| todo!("TODO(port) ftUcf_PadRaw (fighter.c)");
        ftColl_CreateReflectHit(g: G, desc: &mut ReflectDesc, cb: HSD_GObjEvent) = |_, _, _| todo!("TODO(port) ftColl_CreateReflectHit (ftcoll.c)");
        ftCommon_8007DB24(g: G) = |g: G| { unsafe { let _ = crate::ftcommon3::ftcommon_8007db24(g as _); } };
        lb_8000B1CC(j: *mut HSD_JObj, p0: *mut Vec3, p1: *mut Vec3) = |j: *mut HSD_JObj, p0: *mut Vec3, p1: *mut Vec3| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, p0 as _, p1 as _); } };
    }
}

const FT_PART_TRANSN: u32 = FtPart_TransN as u32;
const PAD_RL: u32 = (HSD_PAD_R | HSD_PAD_L) as u32;

#[inline(always)]
fn guard<'a>(f: &'a mut Fighter) -> &'a mut ftCommon_MotionVars_guard {
    unsafe { &mut f.mv.co.guard }
}

/// The shield bone `fp->parts[fp->ft_data->x8->x11].joint`.
fn shield_joint(f: &Fighter) -> *mut HSD_JObj {
    unsafe { (*f.parts.add((*(*f.ft_data).x8).x11 as usize)).joint }
}
fn shield_bone(f: &Fighter) -> u8 {
    unsafe { (*(*f.ft_data).x8).x11 }
}

/// The `HSD_JObjSetMtxDirty` macro (jobj.h).
fn jobj_set_mtx_dirty(j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    let fl = unsafe { (*j).flags };
    let dirty = fl & JOBJ_USER_DEF_MTX as u32 == 0 && fl & JOBJ_MTX_DIRTY as u32 != 0;
    if !dirty {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}
/// `HSD_JObjSetScale` (jobj.h inline).
fn jobj_set_scale(j: *mut HSD_JObj, s: &Vec3) {
    assert!(!j.is_null(), "jobj.h:760");
    unsafe {
        (*j).scale = *s;
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            jobj_set_mtx_dirty(j);
        }
    }
}
/// `HSD_JObjSetTranslate` (jobj.h inline).
fn jobj_set_translate(j: *mut HSD_JObj, t: &Vec3) {
    assert!(!j.is_null(), "jobj.h:916");
    unsafe {
        (*j).translate = *t;
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            jobj_set_mtx_dirty(j);
        }
    }
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
fn ftGetTemplateKind(f: &Fighter) -> i32 {
    let kind = f.kind as i32;
    if kind >= Ft_Kind_Mex0 as i32 && ext::Geno_DefineTemplateSkeleton(kind) != 0 {
        return ext::Geno_DefineBaseKind(kind);
    }
    kind
}

fn caps_allowed(g: G) -> bool {
    ext::ScriptGame_CapsActionAllowed(fp(g), 1) != 0
}

pub fn ftCo_80091A2C(g: G) -> bool {
    if !caps_allowed(g) {
        return false;
    }
    fp(g).input.held_buttons[0] & HSD_PAD_LR as u32 != 0
}

/// static inline `inlineA0`.
fn inlineA0(f: &Fighter) -> bool {
    f.input.held_buttons[0] & HSD_PAD_LR as u32 != 0
}

pub fn ftCo_80091A4C(g: G) -> bool {
    if !caps_allowed(g) {
        return false;
    }
    let f = fp(g);
    if f.input.pressed_buttons & PAD_RL != 0 && (f.active_timer.trigger as i32) < cd().powershield_input_window {
        ftCo_800939B4(g);
        return true;
    }
    if inlineA0(f) && f.shield_health != 0.0 {
        ftCo_800923B4(g);
        return true;
    }
    false
}

pub fn ftCo_80091AD8(g: G, mv_x20: i32) -> bool {
    if !caps_allowed(g) {
        return false;
    }
    let f = fp(g);
    let ret0 = if f.input.pressed_buttons & PAD_RL != 0 && (f.active_timer.trigger as i32) < cd().powershield_input_window {
        ftCo_800939B4(g);
        true
    } else if inlineA0(f) && f.shield_health != 0.0 {
        ftCo_800923B4(g);
        true
    } else {
        false
    };
    if ret0 {
        guard(fp(g)).x20 = mv_x20;
        return true;
    }
    false
}

pub fn ftCo_80091B90(g: G, mv_x20: i32) {
    guard(fp(g)).x20 = mv_x20;
}

pub fn ftCo_80091B9C(g: G) {
    guard(fp(g)).x24 = f2i32(cd().x68 as f64);
}

/// static `ftCo_80091BC4`: steer the shield toward the stick.
pub fn ftCo_80091BC4(f: &mut Fighter) {
    let c = cd();
    let mut stick_rad = ext::lb_8000D008(f.input.lstick[0].y, f.input.lstick[0].x * f.facing_dir);
    if stick_rad < 0.0 {
        stick_rad += 2.0 * core::f64::consts::PI as f32;
    }
    let mut stick_deg = stick_rad * 57.29577951f32;
    if stick_deg < 0.0 {
        stick_deg = 0.0;
    }
    if stick_deg > 359.0 {
        stick_deg = 359.0;
    }
    let mut guard_deg = guard(f).x8 - 10.0;
    let mut deg_delta = stick_deg - guard_deg;
    if deg_delta > 180.0 {
        deg_delta -= 360.0;
    } else if deg_delta < -180.0 {
        deg_delta += 360.0;
    }
    let smoothed_deg = fmadds(deg_delta, c.x44C, guard_deg);
    if smoothed_deg > 360.0 {
        guard_deg = smoothed_deg;
        guard_deg -= 360.0;
    } else {
        guard_deg = smoothed_deg;
        if guard_deg < 0.0 {
            guard_deg += 360.0;
        }
    }
    guard(f).x8 = 10.0 + guard_deg;
    let (x, y) = (f.input.lstick[0].x, f.input.lstick[0].y);
    let mut stick_mag = melee_runtime::gekko::sqrtf(fmadds(x, x, y * y));
    if stick_mag > 1.0 {
        stick_mag = 1.0;
    }
    let x4 = guard(f).x4;
    guard(f).x4 = fmadds(c.x44C, stick_mag - x4, x4);
}

/// static inline `inlineB0`: the shield's scale.
fn inlineB0(f: &Fighter) -> f32 {
    let c = cd();
    if f.kind as i32 == Ft_Kind_Yoshi as i32 {
        f.co_attrs.initial_shield_size
    } else {
        let start = ext::ScriptGame_ModValue(f.player_id as i32, 4, c.x260_startShieldHealth);
        let n1 = (f.shield_health / start) * fmadds(f.lightshield_amount, c.x2D8 - c.x2D4, c.x2D4);
        let n2 = 1.0 - c.x264;
        let n3 = fmadds(n2, n1, c.x264);
        n3 * f.co_attrs.initial_shield_size
    }
}

fn set_shield_scale(f: &Fighter) {
    let s = inlineB0(f);
    let scl = Vec3 { x: s, y: s, z: s };
    jobj_set_scale(shield_joint(f), &scl);
}

pub fn ftCo_80091D58(f: &mut Fighter) {
    set_shield_scale(f);
}

/// static inline `inlineD0`: shield alpha from the light-shield amount.
fn inlineD0(g: G) {
    let f = fp(g);
    if f.kind as i32 != Ft_Kind_Yoshi as i32 {
        let alpha = cd().x2F4;
        let ls = f.lightshield_amount;
        let alpha_i = f2i32((alpha + f2i32((ls * (255.0 - alpha)) as f64) as f32) as f64);
        ext::efLib_SetParamAlpha(g, alpha_i as u8);
    }
}

/// static `ftCo_CopyGuardPose` (TARGET_PC): copy `src`'s descriptor tree into `pose`, taking each
/// part's current SRT from its jobj.
fn ftCo_CopyGuardPose(f: &mut Fighter, mut src: *mut HSD_Joint, pose: *mut HSD_Joint, index: &mut i32) -> *mut HSD_Joint {
    let mut head: *mut HSD_Joint = core::ptr::null_mut();
    let mut prev: *mut HSD_Joint = core::ptr::null_mut();
    unsafe {
        while !src.is_null() {
            let parts_num = (**crate::hooks::ft_parts_table().add(f.kind as usize)).parts_num;
            assert!(*index < MAX_FT_PARTS && (*index as u32) < parts_num, "Geno guard pose exceeds the parts limit");
            let dst = pose.add(*index as usize);
            let jobj = (*f.parts.add(*index as usize)).x4_jobj2;
            *index += 1;
            core::ptr::copy_nonoverlapping(src, dst, 1);
            if !jobj.is_null() {
                (*dst).rotation.x = (*jobj).rotate.x;
                (*dst).rotation.y = (*jobj).rotate.y;
                (*dst).rotation.z = (*jobj).rotate.z;
                (*dst).scale = (*jobj).scale;
                (*dst).position = (*jobj).translate;
            }
            (*dst).child = ftCo_CopyGuardPose(f, (*src).child, pose, index);
            (*dst).next = core::ptr::null_mut();
            if !prev.is_null() {
                (*prev).next = dst;
            } else {
                head = dst;
            }
            prev = dst;
            src = (*src).next;
        }
    }
    head
}

static mut GUARD_POSE_SAID: [u8; Ft_Kind_Max as usize] = [0; Ft_Kind_Max as usize];

/// static `ftCo_GuardPoseTree` (TARGET_PC): the guard pose the shield blends toward.
fn ftCo_GuardPoseTree(f: &mut Fighter, pose: *mut HSD_Joint, row: &mut i32) -> *mut HSD_Joint {
    let own = ext::GenoDefine_RestPoseTree(f);
    let mut index = FtPart_TransN as i32;
    unsafe {
        if own.is_null() {
            return *(*(*f.ft_data).x20).x0.add(2);
        }
        *row = ext::Geno_MotionAnimRow(f, ftCo_MS_Guard as i32);
        if *row < 0 {
            *row = (*f.x1C_actionStateList.add(ftCo_MS_Guard as usize)).anim_id;
        }
        let tree = ext::ftData_80085E50(f, *row);
        if tree.is_null() {
            return own;
        }
        ext::ftAnim_8006F4C8(f, true, tree);
        ext::ftAnim_80070710(f.x8AC_animSkeleton, 0.0);
        ext::ftAnim_8006FB88(f, FT_PART_TRANSN, own);
        ext::HSD_JObjAnimAll(f.x8AC_animSkeleton);
        let k = f.kind as usize;
        if GUARD_POSE_SAID[k] == 0 {
            GUARD_POSE_SAID[k] = 1;
            ext::OSReport_GuardPose(f.kind as i32, *row);
        }
    }
    ftCo_CopyGuardPose(f, own, pose, &mut index)
}

pub fn ftCo_80091E78(g: G, arg1: f32) {
    let f = fp(g);
    if f.reflecting() == 0 && f.anon_221b.x221B_b0() == 0 {
        return;
    }
    let mut own_pose: std::vec::Vec<HSD_Joint> = (0..MAX_FT_PARTS).map(|_| HSD_Joint::zeroed()).collect();
    let mut guard_row = 38;
    let guard_pose = ftCo_GuardPoseTree(fp(g), own_pose.as_mut_ptr(), &mut guard_row);
    ftCo_80091BC4(fp(g));
    let f = fp(g);
    if guard(f).x4 != 0.0 {
        let jobj = f.x8AC_animSkeleton;
        let tree = ext::ftData_80085E50(fp(g), guard_row);
        ext::ftAnim_8006F4C8(fp(g), true, tree);
        ext::ftAnim_80070710(jobj, guard(fp(g)).x8);
        let child = unsafe { (*fp(g).x108_costume_joint).child };
        ext::ftAnim_8006FB88(fp(g), FT_PART_TRANSN, child);
        ext::HSD_JObjAnimAll(jobj);
        let x4 = guard(fp(g)).x4;
        if x4 < 1.0 {
            ext::ftAnim_80070108(fp(g), FT_PART_TRANSN, 1.0 - x4, x4, guard_pose);
        }
        if arg1 < 1.0 {
            ext::ftAnim_8006FE9C(fp(g), FT_PART_TRANSN, arg1, 1.0 - arg1);
        } else {
            ext::ftAnim_8006FF74(fp(g), FT_PART_TRANSN);
        }
    } else if arg1 < 1.0 {
        ext::ftAnim_80070010(fp(g), FT_PART_TRANSN, arg1, 1.0 - arg1, guard_pose);
    } else {
        ext::ftAnim_8006FA58(fp(g), FT_PART_TRANSN, guard_pose);
    }
    set_shield_scale(fp(g));
    inlineD0(g);
    drop(own_pose);
}

/// static inline `ftCo_80092158_inline`: the shield effect in the player's colour.
fn ftCo_80092158_inline(g: G, arg1: i32, arg2: *mut HSD_JObj) {
    let offset = ext::Player_GetUnk45(fp(g).player_id as i32) << 2;
    let color = unsafe { ext::Fighter_804D650C().add(offset as usize) };
    let rgb = unsafe { ((*color as i32) << 16) | ((*color.add(1) as i32) << 8) | *color.add(2) as i32 };
    ext::efSync_Spawn(arg1, g, arg2, rgb);
}

/// static inline `ftCo_80092450_inline`: the shield's absorb hitbox.
fn ftCo_80092450_inline(g: G) {
    let f = fp(g);
    let mut absorb = AbsorbDesc::zeroed();
    absorb.x0_bone_id = shield_bone(f) as i32;
    absorb.x10_size = 1.0;
    absorb.x4_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::ftColl_8007B1B8(g, &mut absorb, Some(ftCo_80092E50));
    fp(g).set_x221A_b7(1);
}

/// static `ftCo_80092158`.
pub fn ftCo_80092158(g: G, arg1: i32, arg2: *mut HSD_JObj) {
    ftCo_80092158_inline(g, arg1, arg2);
}

pub fn ftCo_800921DC(g: G) {
    let j = shield_joint(fp(g));
    ftCo_80092158_inline(g, 1047, j);
    let f = fp(g);
    let c = cd();
    f.set_x2219_b0(1);
    let gv = guard(f);
    gv.xC = 0;
    gv.x0 = 0.0;
    gv.x10 = c.x268;
    gv.x8 = 10.0;
    gv.x4 = 0.0;
    gv.x2C = 0.0;
    let mut ls = (f.input.triggers[0] - c.analog_shoulder_deadzone) / (1.0 - c.analog_shoulder_deadzone);
    if ls < 0.0 {
        ls = guard(f).x2C;
    }
    f.lightshield_amount = ls;
    guard(f).x24 = 0;
    guard(f).x20 = 0;
    let trans = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    jobj_set_translate(shield_joint(f), &trans);
    ftCo_80091E78(g, 0.0);
    ext::ft_PlaySFX(fp(g), 110, 127, 64);
}

pub fn ftCo_800923B4(g: G) {
    match ftGetTemplateKind(fp(g)) {
        k if k == Ft_Kind_Yoshi as i32 => ext::ftYs_Init_8012BECC(g),
        k if k == Ft_Kind_Mars as i32 => {
            ftCo_800924C0(g);
            ext::ftParts_80074B0C(g, 1, 1);
            ext::ft_PlaySFX(fp(g), 190115, 127, 64);
        }
        _ => ftCo_800924C0(g),
    }
}

pub fn ftCo_80092450(g: G) {
    ftCo_80092450_inline(g);
}

pub fn ftCo_800924C0(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_GuardOn as i32, Ft_MF_SkipAnim as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.set_x221C_b3(0);
    f.set_x221C_b1(0);
    f.set_x221C_b2(0);
    guard(f).x1C = 0;
    guard(f).x14 = 0.0;
    guard(f).x18 = 0.0;
    ftCo_80092450_inline(g);
    ftCo_800921DC(g);
}

pub fn ftCo_800925A4(g: G) -> bool {
    let f = fp(g);
    let c = cd();
    if f.anon_221b.x221B_b0() != 0 {
        guard(f).x2C = f.lightshield_amount;
        let v = (f.input.triggers[0] - c.analog_shoulder_deadzone) / (1.0 - c.analog_shoulder_deadzone);
        f.lightshield_amount = if v < 0.0 { guard(f).x2C } else { v };
        let inner = fmadds(f.lightshield_amount, c.x2F0 - c.x2EC, c.x2EC);
        f.shield_health = fmadds(-c.x278, inner, f.shield_health);
        if f.shield_health < 0.0 {
            f.shield_health = 0.0;
            f.set_x221A_b7(0);
            f.anon_221b.set_x221B_b0(0);
            ext::pl_8003E0E8(f.player_id as i32, f.is_sub_fighter() as i32);
            ext::ftCo_80098B20(g);
            ext::ft_PlaySFX(fp(g), 129, 127, 64);
            return true;
        }
        let gv = guard(fp(g));
        if gv.x10 > 0.0 {
            gv.x10 -= 1.0;
            if gv.x10 < 0.0 {
                gv.x10 = 0.0;
            }
        }
    }
    false
}

pub fn ftCo_GuardOn_Anim(g: G) {
    guard(fp(g)).x0 += 1.0;
    if !ftCo_800925A4(g) {
        let f = fp(g);
        if guard(f).x0 >= f.x2E8 {
            ftCo_800928CC(g);
        } else {
            ftCo_80091E78(g, guard(f).x0 / f.x2E8);
        }
    }
}

/// static inline `inlineC0` (always with `ftCo_80092BE8`).
fn inlineC0(g: G) -> bool {
    ftCo_80092BCC(g);
    let f = fp(g);
    let gv = guard(f);
    if (gv.xC != 0 && gv.x10 == 0.0) || (f.anon_221b.x221B_b0() == 0 && f.reflecting() == 0) {
        unsafe { ftCo_80092BE8(g) };
        return true;
    }
    let gv = guard(fp(g));
    if gv.x1C != 0 {
        gv.x1C -= 1;
    }
    false
}

pub fn ftCo_GuardOn_IASA(g: G) {
    let _ = inlineC0(g)
        || ftCo_80093694(g)
        || ext::ftCo_8009515C(g)
        || ext::ftCo_8009980C(g)
        || ext::ftCo_8009917C(g)
        || ext::ftCo_800D8B9C(g)
        || ext::ftCo_Catch_CheckInput(g)
        || ext::ftCo_800CB024(g)
        || ext::ftCo_8009A080(g);
}

pub fn ftCo_GuardOn_Phys(g: G) {
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
    ext::ftColl_8007AEF8(g);
}

pub fn ftCo_GuardOn_Coll(g: G) {
    ext::ft_800845B4(g);
}

pub fn ftCo_800928CC(g: G) {
    if ftGetTemplateKind(fp(g)) == Ft_Kind_Yoshi as i32 {
        ext::ftYs_Shield_8012C1D4(g);
    } else {
        ftCo_80092908(g);
    }
}

pub fn ftCo_80092908(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Guard as i32, Ft_MF_SkipAnim as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let j = shield_joint(fp(g));
    ftCo_80092158_inline(g, 1048, j);
    fp(g).set_x2219_b0(1);
    ftCo_80092450_inline(g);
    ftCo_80091E78(g, 1.0);
}

pub fn ftCo_Guard_Anim(g: G) {
    guard(fp(g)).x0 += 1.0;
    if !ftCo_800925A4(g) {
        ftCo_80091E78(g, 1.0);
    }
}

pub fn ftCo_Guard_IASA(g: G) {
    let _ = inlineC0(g)
        || ext::ftCo_8009515C(g)
        || ext::ftCo_8009980C(g)
        || ext::ftCo_8009917C(g)
        || ext::ftCo_Catch_CheckInput(g)
        || ext::ftCo_800CB024(g)
        || ext::ftCo_8009A080(g);
}

pub fn ftCo_Guard_Phys(g: G) {
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
    ext::ftColl_8007AEF8(g);
}

pub fn ftCo_Guard_Coll(g: G) {
    ext::ft_800845B4(g);
}

pub fn ftCo_80092BCC(g: G) {
    let f = fp(g);
    if f.input.held_buttons[0] & HSD_PAD_LR as u32 == 0 {
        guard(f).xC = 1;
    }
}

/// `ftCo_80092BE8` (also the `inlineC0` callback).
pub unsafe extern "C" fn ftCo_80092BE8(g: G) {
    let f = fp(g);
    ext::pl_8003E114(f.player_id as i32, f.is_sub_fighter() as i32, f.shield_health);
    if ftGetTemplateKind(fp(g)) == Ft_Kind_Yoshi as i32 {
        ext::ftYs_Shield_8012C49C(g);
    } else {
        ftCo_80092C54(g);
    }
}

pub fn ftCo_80092C54(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_GuardOff as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ft_PlaySFX(fp(g), 127, 127, 64);
}

pub fn ftCo_GuardOff_Anim(g: G) {
    guard(fp(g)).x0 += 1.0;
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_GuardOff_IASA(g: G) {
    if guard(fp(g)).x1C == 0
        || (!ext::ftCo_SpecialS_CheckInput(g)
            && !ext::ftCo_Attack100_CheckInput(g)
            && !ext::ftCo_800D6824(g)
            && !ext::ftCo_800D68C0(g)
            && !ext::ftCo_Catch_CheckInput(g)
            && !ext::ftCo_AttackS4_CheckInput(g)
            && !ext::ftCo_AttackHi4_CheckInput(g)
            && !ext::ftCo_AttackLw4_CheckInput(g)
            && !ext::ftCo_AttackS3_CheckInput(g)
            && !ext::ftCo_AttackHi3_CheckInput(g)
            && !ext::ftCo_AttackLw3_CheckInput(g)
            && !ext::ftCo_Attack1_CheckInput(g))
    {
        if ext::ftCo_8009980C(g) {
            return;
        }
        if !ext::ftCo_800CB024(g) {
            return;
        }
    }
}

pub fn ftCo_GuardOff_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_GuardOff_Coll(g: G) {
    ext::ft_800845B4(g);
}

/// `ftCo_80092E50`: the absorb hitbox's on-hit callback.
pub unsafe extern "C" fn ftCo_80092E50(g: G) {
    let b = fp(g).x19B0 as u32 == 10;
    if ftGetTemplateKind(fp(g)) == Ft_Kind_Yoshi as i32 {
        ext::ftYs_Shield_8012C600(g, b);
    } else {
        ftCo_80092F2C(g, b);
    }
    if b {
        ext::ftCo_800C36DC(g);
    }
}

pub fn ftCo_80092ED8(arg0: i32, arg1: f32) -> f32 {
    let c = cd();
    let inner = fmadds(arg1, c.x2E8 - c.x2E4, c.x2E4);
    fmadds(c.x28C, arg0 as f32 * (1.0 - inner), c.x290)
}

pub fn ftCo_80092F2C(g: G, arg1: bool) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_GuardSetOff as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.hitlag_cb = Some(ftCo_80093240);
    f.active_timer.lstick.x = (-2i32) as u8;
    f.post_hitlag_cb = Some(ftCo_800932DC);
    if f.x221C_b2() == 0 {
        let j = shield_joint(f);
        ftCo_80092158_inline(g, 1049, j);
        fp(g).set_x2219_b0(1);
    }
    let c = cd();
    let f = fp(g);
    let inner = fmadds(f.lightshield_amount, c.x2E8 - c.x2E4, c.x2E4);
    let fv = fmadds(c.x28C, f.x19A4 as f32 * (1.0 - inner), c.x290);
    let end = ext::lbGetJObjEndFrame(unsafe { (*g).hsd_obj } as *mut HSD_JObj);
    ext::ftAnim_SetAnimRate(g, (0.1 + end) / fv);
    if !arg1 {
        let f = fp(g);
        let mut guard_vel = fv * c.x294;
        if f.x221C_b2() == 0 {
            guard_vel *= c.x2BC;
        }
        let max_guard_vel = c.x298;
        if guard_vel > max_guard_vel {
            guard_vel = max_guard_vel;
        }
        f.gr_vel = if f.specialn_facing_dir < 0.0 { guard_vel } else { -guard_vel };
    }
    ftCo_80092450_inline(g);
    set_shield_scale(fp(g));
}

/// static `ftCo_Ucf084ShieldSdi` (TARGET_PC): UCF 0.84 shield SDI past the stick window.
fn ftCo_Ucf084ShieldSdi(f: &Fighter) -> bool {
    let port = f.x618_player_id as i32;
    let prev = f.input.lstick[1].x;
    if ext::Replay_UcfVersion(port) != 84 {
        return false;
    }
    if f.active_sticky.lstick.x > 1 {
        return false;
    }
    let a = if prev < 0.0 { -prev } else { prev };
    if !(a < cd().sdi_min_stick_mag) {
        return false;
    }
    let d = ext::ftUcf_PadRaw(port, 0, 0).wrapping_sub(ext::ftUcf_PadRaw(port, 0, 2));
    d.wrapping_mul(d) > 3844
}

/// `ftCo_80093240`: GuardSetOff hitlag callback (shield SDI).
pub unsafe extern "C" fn ftCo_80093240(g: G) {
    let f = fp(g);
    let c = cd();
    if f.allow_sdi() != 0 && f.ground_or_air == GA_Ground as _ {
        let x = f.input.lstick[0].x;
        let ax = if x < 0.0 { -x } else { x };
        if ax >= c.sdi_min_stick_mag && ((f.active_timer.lstick.x as i32) < c.sdi_stick_window || ftCo_Ucf084ShieldSdi(f)) {
            let scl = c.x4C0 * (f.input.lstick[0].x * c.sdi_pos_scale);
            f.cur_pos.x = fmadds(f.coll_data.floor.normal.y, scl, f.cur_pos.x);
            f.cur_pos.y = fmadds(-f.coll_data.floor.normal.x, scl, f.cur_pos.y);
            f.active_timer.lstick.x = 254;
        }
    }
}

/// `ftCo_800932DC`: GuardSetOff post-hitlag callback.
pub unsafe extern "C" fn ftCo_800932DC(g: G) {
    let f = fp(g);
    let c = cd();
    if f.ground_or_air == GA_Ground as _ {
        let x = f.input.lstick[0].x;
        let ax = if x < 0.0 { -x } else { x };
        if ax >= c.sdi_min_stick_mag {
            let scl = c.x4C0 * (f.input.lstick[0].x * c.x4BC);
            f.cur_pos.x = fmadds(f.coll_data.floor.normal.y, scl, f.cur_pos.x);
            f.cur_pos.y = fmadds(-f.coll_data.floor.normal.x, scl, f.cur_pos.y);
        }
    }
}

pub fn ftCo_GuardSetOff_Anim(g: G) {
    ftCo_80093BC0(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        if guard(fp(g)).xC != 0 {
            // ftCo_80092BE8_inline
            unsafe { ftCo_80092BE8(g) };
        } else {
            // ftCo_800928CC_inline_arg
            ftCo_800928CC(g);
        }
    } else {
        // ftCo_80091D58_inline_arg
        set_shield_scale(fp(g));
    }
}

pub fn ftCo_GuardSetOff_IASA(_g: G) {}

pub fn ftCo_GuardSetOff_Phys(g: G) {
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
}

pub fn ftCo_GuardSetOff_Coll(g: G) {
    if fp(g).allow_sdi() == 0 {
        ext::ft_800845B4(g);
    } else {
        ext::ft_80084104(g);
    }
}

pub fn ftCo_80093694(g: G) -> bool {
    let f = fp(g);
    let w = cd().powershield_input_window;
    if guard(f).x0 < w as f32 && f.input.pressed_buttons & PAD_RL != 0 && (f.active_timer.trigger as i32) < w {
        ftCo_80093850(g);
        return true;
    }
    false
}

pub fn ftCo_8009370C(g: G, on_reflect: HSD_GObjEvent) {
    let f = fp(g);
    let c = cd();
    let mut reflect = ReflectDesc::zeroed();
    reflect.x0_bone_id = shield_bone(f) as u32;
    reflect.x4_max_damage = f2i32(f.shield_health as f64);
    reflect.x8_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    reflect.x14_size = c.x2A8;
    reflect.x18_damage_mul = c.x2AC;
    reflect.x1C_speed_mul = c.x2B0;
    reflect.x20_behavior = 1;
    ext::ftColl_CreateReflectHit(g, &mut reflect, on_reflect);
}

/// `ftCo_80093790`: the powershield reflect callback.
pub unsafe extern "C" fn ftCo_80093790(g: G) {
    ext::ftCommon_8007DB24(g);
    let j = shield_joint(fp(g));
    ftCo_80092158_inline(g, 1050, j);
    fp(g).set_x2219_b0(1);
    ext::ft_PlaySFX(fp(g), 128, 127, 64);
}

pub fn ftCo_80093850(g: G) {
    if ftGetTemplateKind(fp(g)) == Ft_Kind_Yoshi as i32 {
        ext::ftYs_Shield_8012C850(g);
    } else {
        ftCo_8009388C(g);
    }
}

pub fn ftCo_8009388C(g: G) {
    let start = fp(g).cur_anim_frame;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_GuardReflect as i32, (Ft_MF_SkipAnim | Ft_MF_KeepGfx) as u32, start, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let c = cd();
    f.active_timer.trigger = 0xFE;
    f.set_x221A_b7(0);
    f.anon_221b.set_x221B_b0(0);
    f.set_x221C_b3(1);
    f.set_x221C_b1(1);
    f.set_x221C_b2(1);
    guard(f).x1C = 0;
    guard(f).x14 = c.x2A4;
    guard(f).x18 = c.x2B4;
    ftCo_8009370C(g, Some(ftCo_80093790));
}

pub fn ftCo_800939B4(g: G) {
    match ftGetTemplateKind(fp(g)) {
        k if k == Ft_Kind_Yoshi as i32 => ext::ftYs_Shield_8012C914(g),
        k if k == Ft_Kind_Mars as i32 => {
            ftCo_80093A50(g);
            ext::ftParts_80074B0C(g, 1, 1);
            ext::ft_PlaySFX(fp(g), 190115, 127, 64);
        }
        _ => ftCo_80093A50(g),
    }
}

pub fn ftCo_80093A50(g: G) {
    ext::Fighter_ChangeMotionState(g, 182, Ft_MF_SkipAnim as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    let c = cd();
    f.active_timer.trigger = 0xFE;
    f.set_x221C_b3(1);
    f.set_x221C_b1(1);
    f.set_x221C_b2(1);
    guard(f).x1C = 0;
    guard(f).x14 = c.x2A4;
    guard(f).x18 = c.x2B4;
    ftCo_80092450_inline(g);
    ftCo_8009370C(g, Some(ftCo_80093790));
    ftCo_800921DC(g);
}

pub fn ftCo_80093BC0(g: G) {
    let f = fp(g);
    if f.x221C_b3() != 0 {
        f.set_x221C_b3(0);
    }
    if f.x221C_b1() != 0 {
        guard(f).x14 -= 1.0;
        if guard(f).x14 < 0.0 {
            f.set_x221C_b1(0);
            f.set_reflecting(0);
            ftCo_80092450_inline(g);
        }
    }
    let f = fp(g);
    if f.x221C_b2() != 0 {
        guard(f).x18 -= 1.0;
        if guard(f).x18 < 0.0 {
            f.set_x221C_b2(0);
        }
    }
}

pub fn ftCo_GuardReflect_Anim(g: G) {
    // ftCo_80093BC0_inline: the same body as ftCo_80093BC0.
    ftCo_80093BC0(g);
    // ftCo_GuardOn_Anim_inline: switches on `fp->kind`, not the template kind.
    guard(fp(g)).x0 += 1.0;
    if !ftCo_800925A4(g) {
        let f = fp(g);
        if guard(f).x0 >= f.x2E8 {
            if f.kind as i32 == Ft_Kind_Yoshi as i32 {
                ext::ftYs_Shield_8012C1D4(g);
            } else {
                // ftCo_80092908_inline2
                ftCo_80092908(g);
            }
        } else {
            ftCo_80091E78(g, guard(f).x0 / f.x2E8);
        }
    }
}

pub fn ftCo_GuardReflect_IASA(g: G) {
    let _ = inlineC0(g)
        || ext::ftCo_8009515C(g)
        || ext::ftCo_8009980C(g)
        || ext::ftCo_8009917C(g)
        || ext::ftCo_800D8B9C(g)
        || ext::ftCo_Catch_CheckInput(g)
        || ext::ftCo_800CB024(g)
        || ext::ftCo_8009A080(g);
}

pub fn ftCo_GuardReflect_Phys(g: G) {
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
    ext::ftColl_8007AEF8(g);
}

pub fn ftCo_GuardReflect_Coll(g: G) {
    ext::ft_800845B4(g);
}

pub fn ftCo_80094098(g: G, pos: *mut f32) -> f32 {
    let f = fp(g);
    ext::lb_8000B1CC(shield_joint(f), core::ptr::null_mut(), pos as *mut Vec3);
    inlineB0(fp(g))
}

pub fn ftCo_80094138(f: &mut Fighter) {
    guard(f).x1C = cd().x2B8;
    guard(f).x10 = 0.0;
}

#[allow(dead_code)]
fn _f2u8_used(v: f32) -> u8 {
    f2u8(v as f64)
}
