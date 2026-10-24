//! Native port of src/melee/ft/ftaction.c (fighter subaction commands: the 49-entry exec and
//! anim-skip tables, their handlers, and the three script runners), TARGET_PC variant (ACE
//! hitbox id >= 4 remap, script hit rules, Geno's opcode 59 escape).
//!
//! Command words are read through melee_types' generated `CmdUnion` member accessors (the one
//! type generator: bitfields in GameCube byte order). `NEXT_CMD` / `SKIP_CMD` step `cmd->u` by
//! one `CmdUnion`, as `crate::ftaction::skip_cmd` (already ported, with the 25 pure skip
//! handlers and 4 flag setters, which are re-exported here). Float math: `0.003906f * int`
//! products only, no fused sites. Callees in other TUs go through `ext`.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use crate::ftaction::skip_cmd;
use crate::types::*;

pub use crate::ftaction::{
    ftaction_800711dc as ftAction_800711DC, ftaction_800715ec as ftAction_800715EC, ftaction_8007168c as ftAction_8007168C,
    ftaction_800716f8 as ftAction_800716F8, ftaction_80071774 as ftAction_80071774, ftaction_800717c8 as ftAction_800717C8,
    ftaction_80071810 as ftAction_80071810, ftaction_80071908 as ftAction_80071908, ftaction_8007192c as ftAction_8007192C,
    ftaction_80071950 as ftAction_80071950, ftaction_80071974 as ftAction_80071974, ftaction_80071ca4 as ftAction_80071CA4,
    ftaction_80071d30 as ftAction_80071D30, ftaction_80071f0c as ftAction_80071F0C, ftaction_800722c8 as ftAction_800722C8,
    ftaction_800726c0 as ftAction_800726C0, ftaction_8007296c as ftAction_8007296C, ftaction_800729c4 as ftAction_800729C4,
    ftaction_80072a4c as ftAction_80072A4C, ftaction_80072aac as ftAction_80072AAC, ftaction_80072b04 as ftAction_80072B04,
    ftaction_80072b84 as ftAction_80072B84, ftaction_80072be4 as ftAction_80072BE4, ftaction_80072c5c as ftAction_80072C5C,
    ftaction_80072e24 as ftAction_80072E24, ftaction_80072fe0 as ftAction_80072FE0, ftaction_8007309c as ftAction_8007309C,
    ftaction_80073108 as ftAction_80073108, ftaction_8007320c as ftAction_8007320C,
};

type G = *mut HSD_GObj;
pub type FtCmd = fn(G, &mut CommandInfo);

/// `F32_MAX`.
pub const F32_MAX: f32 = f32::MAX;
/// `GENO_FTCMD_OP` (pc/geno/geno.h).
pub const GENO_FTCMD_OP: u32 = 59;
pub const GENO_MODE_EXEC: i32 = 0;
pub const GENO_MODE_ANIM: i32 = 1;
pub const GENO_MODE_SKIP: i32 = 2;

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    type F = *mut Fighter;
    pub fn ftCo_8009F834(_g: G, _gfx: i32, _bone: i32, _common: i32, _destroy: i32, _off: *const Vec3, _range: *const Vec3, _unk: f32) { todo!("ftCo_8009F834 (ftCommon/ftCo_09F7.c)") }
    pub fn ftColl_800768A0(_fp: F, _h: *mut HitCapsule) { todo!("ftColl_800768A0 (ft/ftcoll.c)") }
    pub fn ftColl_8007ABD0(_h: *mut HitCapsule, _dmg: u32, _g: G) { todo!("ftColl_8007ABD0 (ft/ftcoll.c)") }
    pub fn ftColl_8007AC9C(_h: *mut HitCapsule, _angle: u32, _g: G) { todo!("ftColl_8007AC9C (ft/ftcoll.c)") }
    pub fn ftColl_8007AD18(_fp: F, _h: *mut HitCapsule) { todo!("ftColl_8007AD18 (ft/ftcoll.c)") }
    pub fn ftColl_8007AFC8(_g: G, _i: u32) { todo!("ftColl_8007AFC8 (ft/ftcoll.c)") }
    pub fn ftColl_8007AFF8(_g: G) { todo!("ftColl_8007AFF8 (ft/ftcoll.c)") }
    pub fn ftColl_8007B62C(_g: G, _s: u32) { todo!("ftColl_8007B62C (ft/ftcoll.c)") }
    pub fn ftColl_8007B0C0(_g: G, _s: u32) { todo!("ftColl_8007B0C0 (ft/ftcoll.c)") }
    pub fn ftColl_8007B128(_g: G, _b: u32, _s: u32) { todo!("ftColl_8007B128 (ft/ftcoll.c)") }
    pub fn ftCommon_80080484(_fp: F) { unsafe { let _ = crate::ftcommon2::ftcommon_80080484(&mut *(_fp as *mut _)); } }
    pub fn ftCommon_8007D7FC(_fp: F) { unsafe { let _ = crate::ftcommon3::ftcommon_8007d7fc(&mut *(_fp as *mut _)); } }
    pub fn ftCommon_8007D5D4(_fp: F) { unsafe { let _ = crate::ftcommon2::ftcommon_8007d5d4(&mut *(_fp as *mut _)); } }
    pub fn ftCommon_8007D60C(_fp: F) { unsafe { let _ = crate::ftcommon2::ftcommon_8007d60c(&mut *(_fp as *mut _)); } }
    pub fn ftCommon_8007F5CC(_g: G, _v: u32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007f5cc(_g as _, _v as _); } }
    pub fn ftCommon_8007E83C(_g: G, _a: u32, _b: u32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007e83c(_g as _, _a as _, _b as _); } }
    pub fn ftCommon_8007EC30(_a: u32, _b: u32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007ec30(_a as _, _b as _); } }
    pub fn ftCommon_8007EBAC(_fp: F, _a: u32, _b: u32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ebac(&mut *(_fp as *mut _), _a as _, _b as _); } }
    pub fn ftCommon_8007ECD4(_fp: F, _a: u32) { unsafe { let _ = crate::ftcommon2::ftcommon_8007ecd4(&mut *(_fp as *mut _), _a as _); } }
    pub fn ftCommon_8007EEC8(_fp: F, _a: i32, _b: i32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007eec8(&mut *(_fp as *mut _), _a as _, _b as _); } }
    pub fn ftCommon_8007EF5C(_fp: F, _a: i32) { unsafe { let _ = crate::ftcommon3::ftcommon_8007ef5c(&mut *(_fp as *mut _), _a as _); } }
    pub fn ftParts_80074B0C(_g: G, _i: i32, _v: i32) { unsafe { let _ = crate::tu::ftparts::ftParts_80074B0C(_g as _, _i as _, _v as _); } }
    pub fn ftParts_80074A8C(_g: G) { crate::ft0df::ftparts_80074a8c(_g) }
    pub fn ftParts_80074ACC(_g: G) { crate::ft0df::ftparts_80074acc(_g) }
    pub fn ft_PlaySFX(_fp: F, _sfx: i32, _vol: i32, _pan: i32) { todo!("ft_PlaySFX (ft/ft_0877.c)") }
    pub fn ft_sfx_variant(_which: u32, _fp: F, _sfx: i32, _vol: i32, _pan: i32) { todo!("ft_80088478/800881D8/80088510/800885A8/80088640/80088328 (ft/ft_0877.c)") }
    pub fn ft_sfx_stop(_which: u32, _fp: F) { todo!("ft_80088828/80088770/80088884/800888E0/8008893C/800887CC (ft/ft_0877.c)") }
    pub fn ft_800889F4(_fp: F, _s: *mut FtSFXArr) { unsafe { let _ = crate::ft0881::ft_800889f4(&mut *(_fp as *mut _), _s as _); } }
    pub fn ft_80087D0C(_fp: F, _id: u32) -> i32 { unsafe { crate::ft0877::ft_80087d0c(&*(_fp as *const _), _id as _) } }
    pub fn ft_80084BFC(_g: G, _a: &mut i32, _b: &mut i32, _c: &mut i32) -> bool { unsafe { crate::tu::ft_081b::ft_80084bfc(_g as _, _a as *mut _ as *mut _, _b as *mut _ as *mut _, _c as *mut _ as *mut _) } }
    pub fn ft_80084C38(_g: G, _a: &mut i32, _b: &mut i32, _c: &mut i32) -> bool { unsafe { crate::tu::ft_081b::ft_80084c38(_g as _, _a as *mut _ as *mut _, _b as *mut _ as *mut _, _c as *mut _ as *mut _) } }
    pub fn ft_8008A1B8(_g: G, _a: u32) { unsafe { let _ = crate::tu::ft_0899::ft_8008A1B8(_g as _, _a as _); } }
    pub fn HSD_Randi(_n: i32) -> i32 { todo!("HSD_Randi (baselib/random.c)") }
    pub fn lbAudioAx_800263E8(_dir: f32, _g: G, _beh: u32, _sfx: i32, _a: i32, _b: i32, _p0: i32, _p1: i32, _p2: i32, _s8: i32, _sc: i32) -> i32 { todo!("lbAudioAx_800263E8 (lb/lbaudio_ax.c)") }
    pub fn lbAudioAx_800264E4(_h: i32) -> i32 { todo!("lbAudioAx_800264E4 (lb/lbaudio_ax.c)") }
    pub fn ftAnim_800704F0(g: G, idx: i32, frame: f32) { unsafe { crate::tu::ftanim::ftAnim_800704F0(g, idx, frame) } }
    pub fn ftAnim_ApplyPartAnim(g: G, a: i32, b: i32, c: f32) { unsafe { crate::tu::ftanim::ftAnim_ApplyPartAnim(g, a, b, c) } }
    pub fn ftCo_800BFFD0(_fp: F, _a: u32, _b: u32) { unsafe { let _ = crate::tu::ftcolanim::ftco_800bffd0(&mut *(_fp as *mut _), _a as _, _b as _); } }
    pub fn ftCo_800C0200(_fp: F, _a: u32) { todo!("ftCo_800C0200 (ft/ft_0C88.c)") }
    pub fn ftCo_8009E318(_g: G, _a: i32, _f: f32) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E318(_g as _, _a as _, _f as _); } }
    pub fn Fighter_TakeDamage_8006CC7C(_fp: F, _d: f32) { unsafe { let _ = crate::tu::fighter::Fighter_TakeDamage_8006CC7C(&mut *(_fp as *mut _), _d as _); } }
    pub fn ftCo_800DEE84(_g: G, _color: i32, _frames: f32, _mult: f32) { unsafe { let _ = crate::ft0df::ftco_800dee84(_g as _, _color as _, _frames as _, _mult as _); } }
    pub fn ftCo_800C8B60(_fp: F, _a: u32, _b: u32) { unsafe { let _ = crate::tu::ft_0c88::ftco_800c8b60(&mut *(_fp as *mut _), _a as _, _b as _); } }
    pub fn ftCo_8009E714(_g: G, _idx: u8, _timer: i32, _x: f32, _y: f32, _mag: f32, _decay: f32, _angle: f32) { unsafe { let _ = crate::tu::ftdynamics::ftCo_8009E714(_g as _, _idx as _, _timer as _, _x as _, _y as _, _mag as _, _decay as _, _angle as _); } }
    pub fn Command_Execute(_cmd: *mut CommandInfo, _code: u32) -> bool { unsafe { melee_runtime::tu::lbcommand::Command_Execute(_cmd as _, _code as _) } }
    pub fn Geno_FtCmd(_g: G, _cmd: *mut CommandInfo, _mode: i32) { todo!("Geno_FtCmd (pc/geno)") }
    pub fn ScriptGame_HitRuleForget(_h: *mut HitCapsule) { todo!("ScriptGame_HitRuleForget (pc/gameworld/script_hit_rules)") }
    pub fn ScriptGame_HitRuleCreate(_fp: F, _h: *mut HitCapsule, _k: i32) { todo!("ScriptGame_HitRuleCreate (pc/gameworld/script_hit_rules)") }
    /// `HSD_GObj_CurrentInvokedProc` (NULL when no proc is running).
    pub fn HSD_GObj_CurrentInvokedProc() -> *mut HSD_GObjProc { todo!("HSD_GObj_CurrentInvokedProc (baselib/gobjproc.c)") }
    pub fn OSReport(_s: &str) {}
}

#[inline(always)]
unsafe fn fpm<'a>(g: G) -> &'a mut Fighter { &mut *((*g).user_data as *mut Fighter) }
#[inline(always)]
unsafe fn u<'a>(cmd: &CommandInfo) -> &'a CmdUnion { &*cmd.anon_8.u }
#[inline(always)]
fn next(cmd: &mut CommandInfo) { skip_cmd(cmd, 1) }
const K: f32 = 0.003906f32;

/// `ftAction_80071028`: spawn GFX (5 words).
pub fn ftAction_80071028(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        if fp.invisible() != 0 {
            return ftAction_800711DC(g, cmd);
        }
        let w = &u(cmd).spawn_gfx_0;
        let bone = if w.useUnkBone() != 0 { (*(*fp.ft_data).x8).x12 as i32 } else { w.boneId() as i32 };
        let common = w.useCommonBoneIDs() as i32;
        let destroy = w.destroyOnStateChange() as i32;
        next(cmd);
        let gfx_id = u(cmd).spawn_gfx_1.gfxID();
        let unk = u(cmd).spawn_gfx_1.unkFloat() as f32;
        next(cmd);
        let mut off = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut range = off;
        off.x = K * u(cmd).spawn_gfx_2.offsetZ() as f32;
        off.y = K * u(cmd).spawn_gfx_2.offsetY() as f32;
        next(cmd);
        off.z = K * u(cmd).spawn_gfx_3.offsetX() as f32;
        range.x = K * u(cmd).spawn_gfx_3.rangeZ() as f32;
        next(cmd);
        range.y = K * u(cmd).spawn_gfx_4.rangeY() as f32;
        range.z = K * u(cmd).spawn_gfx_4.rangeX() as f32;
        next(cmd);
        ext::ftCo_8009F834(g, gfx_id as i32, bone, common, destroy, &off, &range, unk);
    }
}

/// `ftAction_HitboxSlot` (static, TARGET_PC): ACE hitbox ids >= 4 go to a capsule slot 0-3.
pub unsafe fn ftAction_HitboxSlot(fp: &mut Fighter, id: u32, create: bool) -> *mut HitCapsule {
    let n = fp.x914.len();
    if (id as usize) < n {
        return &mut fp.x914[id as usize];
    }
    for i in 0..n {
        if fp.x914[i].x43_b7() != 0 && fp.x914[i].state != HitCapsule_Disabled {
            return &mut fp.x914[i];
        }
    }
    if create {
        for i in 0..n {
            if fp.x914[i].state == HitCapsule_Disabled {
                fp.x914[i].set_x43_b7(1);
                ext::OSReport("ace: hitbox id remapped\n");
                return &mut fp.x914[i];
            }
        }
        ext::OSReport("ace: hitbox id: no free slot, retail indexing\n");
    }
    // Retail indexing past x914 lands in xDF4 (the C reads the next member).
    (fp.x914.as_mut_ptr()).add(id as usize)
}

/// `ftAction_8007121C`: create hitbox (6 words).
pub fn ftAction_8007121C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let skip = &*(cmd.anon_8.u as *const spawn_hitbox_skip);
        if skip.xF_b4() != 0 && fp.x1064_thrownHitbox.anon_134.owner.is_null() {
            ftAction_800715EC(g, cmd);
        } else {
            let w0 = &u(cmd).create_hitbox_0;
            let hit_group = w0.hit_group();
            let hb = ftAction_HitboxSlot(fp, w0.id(), true);
            let h = &mut *hb;
            if (w0.id() as usize) < fp.x914.len() {
                h.set_x43_b7(0);
            }
            if h.state == HitCapsule_Disabled || h.x4 != hit_group {
                h.x4 = hit_group;
                h.state = HitCapsule_Enabled;
                fp.set_x2219_b3(1);
                ext::ftColl_800768A0(fp, hb);
            }
            let idx = w0.bone() as i32;
            h.jobj = if w0.use_common_bone_ids() != 0 {
                (*fp.parts.offset(crate::ftdmgfall::ftparts_get_bone_index(fp, idx) as isize)).joint
            } else {
                (*fp.parts.offset(idx as isize)).joint
            };
            ext::ScriptGame_HitRuleForget(hb);
            ext::ftColl_8007ABD0(hb, w0.damage(), g);
            next(cmd);
            h.scale = K * u(cmd).create_hitbox_1.size() as f32;
            h.b_offset.x = K * u(cmd).create_hitbox_1.z_offset() as f32;
            next(cmd);
            h.b_offset.y = K * u(cmd).create_hitbox_2.y_offset() as f32;
            h.b_offset.z = K * u(cmd).create_hitbox_2.x_offset() as f32;
            next(cmd);
            let w3 = &u(cmd).create_hitbox_3;
            ext::ftColl_8007AC9C(hb, w3.angle(), g);
            h.x24 = w3.knockback_growth();
            h.x28 = w3.weight_set_knockback();
            h.set_x43_b0(w3.item_hit_interaction() as u8);
            h.set_x43_b1(w3.ignore_fighter_scale() as u8);
            h.set_x40_b0(w3.clank() as u16);
            h.set_x40_b1(w3.rebound() as u16);
            next(cmd);
            let w4 = &u(cmd).create_hitbox_4;
            h.x2C = w4.base_knockback();
            h.element = w4.element();
            h.x34 = w4.shield_damage();
            h.sfx_severity = w4.hit_sfx_severity() as i32;
            h.sfx_kind = w4.hit_sfx_kind() as _;
            h.set_x40_b2(w4.hit_aerial() as u16);
            h.set_x40_b3(w4.hit_grounded() as u16);
            next(cmd);
            h.set_x42_b5(1);
            h.set_x42_b7(1);
            h.set_x41_b4(0);
            h.set_x41_b6(0);
            h.set_x41_b5(0);
            h.set_x42_b0(0);
            h.set_x42_b4(0);
            h.set_x41_b7(0);
            h.anon_134.set_hit_grabbed_victim_only(u(cmd).create_hitbox_5.x1_b4() as u8);
            h.set_x42_b1(1);
            h.set_x42_b2(0);
            h.set_x43_b2(0);
            ext::ScriptGame_HitRuleCreate(fp, hb, 0);
            let p = ext::HSD_GObj_CurrentInvokedProc();
            if !p.is_null() && (*p).s_link > 9 {
                ext::ftColl_8007AD18(fp, hb);
            }
        }
        ext::ftCommon_80080484(fp);
    }
}

/// `ftAction_8007162C`: adjust hitbox damage.
pub fn ftAction_8007162C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).set_hitbox_damage;
        let h = ftAction_HitboxSlot(fpm(g), w.idx(), false);
        ext::ftColl_8007ABD0(h, w.value(), g);
    }
    next(cmd);
}

/// `ftAction_8007169C`: adjust hitbox scale.
pub fn ftAction_8007169C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).set_hitbox_scale;
        let h = ftAction_HitboxSlot(fpm(g), w.idx(), false);
        (*h).scale = K * w.value() as f32;
    }
    next(cmd);
}

/// `ftAction_80071708`: hitbox x42 b5/b7 flags.
pub fn ftAction_80071708(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).set_hitbox_x42_b57;
        let h = &mut *ftAction_HitboxSlot(fpm(g), w.idx(), false);
        // `type` is the 2nd field of the record: the generator names it `idx`'s neighbour; read
        // through the raw word as the C bitfield `type` is not generated separately.
        match set_hitbox_type(w) {
            0 => h.set_x42_b5(w.value() as u8),
            1 => h.set_x42_b7(w.value() as u8),
            _ => {}
        }
    }
    next(cmd);
}

/// `set_hitbox_x42_b57.type`.
fn set_hitbox_type(w: &set_hitbox_x42_b57) -> u32 { w.r#type() }

/// `ftAction_80071784`.
pub fn ftAction_80071784(g: G, cmd: &mut CommandInfo) {
    ext::ftColl_8007AFC8(g, unsafe { u(cmd).set_throw_flags.hit_idx() });
    next(cmd);
}

/// `ftAction_800717D8`.
pub fn ftAction_800717D8(g: G, cmd: &mut CommandInfo) {
    ext::ftColl_8007AFF8(g);
    next(cmd);
}

/// `ftAction_80071820`: set one of `cmd_vars`.
pub fn ftAction_80071820(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).set_cmd_var;
        let i = w.idx();
        if i < 4 {
            fpm(g).cmd_vars[i as usize] = w.value();
        }
    }
    next(cmd);
}

/// `ftAction_800718A4`.
pub fn ftAction_800718A4(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        match u(cmd).set_throw_flags.hit_idx() {
            0 => {
                fp.anon_2210.anon_0.set_throw_flags_b3(1);
                fp.cmd_timer = cmd.timer;
            }
            1 => fp.anon_2210.anon_0.set_throw_flags_b4(1),
            _ => {}
        }
    }
    next(cmd);
}

/// `ftAction_80071998`: airborne state.
pub fn ftAction_80071998(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        match u(cmd).set_airborne_state.state() {
            0 => ext::ftCommon_8007D7FC(fp),
            1 => ext::ftCommon_8007D5D4(fp),
            2 => ext::ftCommon_8007D60C(fp),
            _ => {}
        }
    }
    next(cmd);
}

/// `ftAction_80071A14`.
pub fn ftAction_80071A14(g: G, cmd: &mut CommandInfo) {
    ext::ftColl_8007B62C(g, unsafe { u(cmd).set_airborne_state.state() });
    next(cmd);
}
/// `ftAction_80071A58`.
pub fn ftAction_80071A58(g: G, cmd: &mut CommandInfo) {
    ext::ftColl_8007B0C0(g, unsafe { u(cmd).set_airborne_state.state() });
    next(cmd);
}
/// `ftAction_80071A9C`: hurt capsule state.
pub fn ftAction_80071A9C(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftColl_8007B128(g, u(cmd).set_hurt_state.bone_idx(), u(cmd).set_hurt_state.state()) };
    next(cmd);
}
/// `ftAction_80071AE8`: jab combo.
pub fn ftAction_80071AE8(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        if u(cmd).set_jab_combo.disabled() == 0 || !fp.x197C.is_null() {
            fp.set_x2218_b1(1);
        }
    }
    next(cmd);
}
/// `ftAction_80071B28`: rapid jab.
pub fn ftAction_80071B28(g: G, cmd: &mut CommandInfo) {
    unsafe { fpm(g).set_x2218_b2(u(cmd).set_jab_rapid.state() as u8) };
    next(cmd);
}

fn play_behavior(fp: &mut Fighter, behavior: u32, sfx: i32, vol: i32, pan: i32) {
    match behavior {
        0 => ext::ft_PlaySFX(fp, sfx, vol, pan),
        1..=6 => ext::ft_sfx_variant(behavior, fp, sfx, vol, pan),
        _ => {}
    }
}

/// `ftAction_80071B50`: sound effect (3 words).
pub fn ftAction_80071B50(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let behavior = u(cmd).sound_effect_0.behavior();
        next(cmd);
        match behavior {
            0..=6 => {
                let sfx = u(cmd).sound_effect_1.sfx_id;
                next(cmd);
                let vol = u(cmd).sound_effect_2.volume() as u8;
                let pan = u(cmd).sound_effect_2.panning() as u8;
                play_behavior(fp, behavior, sfx as i32, vol as i32, pan as i32);
            }
            10..=15 => {
                next(cmd);
                ext::ft_sfx_stop(behavior, fp);
            }
            _ => {}
        }
    }
    next(cmd);
}

/// `ftAction_80071CCC`: smash charge sfx.
pub fn ftAction_80071CCC(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let s = (*(*fp.ft_data).x4C_sfx).smash;
        if !s.is_null() {
            ext::ft_800889F4(fp, s);
        }
    }
    next(cmd);
}

/// `ftAction_80071D40`.
pub fn ftAction_80071D40(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftParts_80074B0C(g, u(cmd).set_dobj_flags.idx(), u(cmd).set_dobj_flags.value()) };
    next(cmd);
}
/// `ftAction_80071D94`.
pub fn ftAction_80071D94(g: G, cmd: &mut CommandInfo) {
    ext::ftParts_80074A8C(g);
    next(cmd);
}
/// `ftAction_80071DCC`.
pub fn ftAction_80071DCC(g: G, cmd: &mut CommandInfo) {
    ext::ftParts_80074ACC(g);
    next(cmd);
}

/// `ftAction_80071E04`: throw hitbox (3 words).
pub fn ftAction_80071E04(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let h: *mut HitCapsule = &mut fp.xDF4[u(cmd).set_throw_hitbox_0.idx() as usize];
        ext::ScriptGame_HitRuleForget(h);
        ext::ftColl_8007ABD0(h, u(cmd).set_throw_hitbox_0.damage(), g);
        next(cmd);
        let w1 = &u(cmd).set_throw_hitbox_1;
        ext::ftColl_8007AC9C(h, w1.unk0(), g);
        (*h).x24 = w1.hit_x24();
        (*h).x28 = w1.hit_x28();
        next(cmd);
        let w2 = &u(cmd).set_throw_hitbox_2;
        (*h).x2C = w2.hit_x2C();
        (*h).element = w2.element();
        (*h).sfx_severity = w2.sfx_severity() as i32;
        (*h).sfx_kind = w2.sfx_kind() as _;
        ext::ScriptGame_HitRuleCreate(fp, h, 0);
    }
    next(cmd);
}

/// `ftAction_80071F34`.
pub fn ftAction_80071F34(g: G, cmd: &mut CommandInfo) {
    ext::ftCommon_8007F5CC(g, unsafe { u(cmd).unk27.value() });
    next(cmd);
}
/// `ftAction_80071F78`: article visibility.
pub fn ftAction_80071F78(g: G, cmd: &mut CommandInfo) {
    unsafe { fpm(g).set_x221E_b4(u(cmd).set_article_vis.value() as u8) };
    next(cmd);
}
/// `ftAction_80071FA0`: fighter visibility.
pub fn ftAction_80071FA0(g: G, cmd: &mut CommandInfo) {
    unsafe { fpm(g).set_x221E_b5(u(cmd).set_fighter_vis.value() as u8) };
    next(cmd);
}

/// `ftAction_80071FC8`: pseudo-random sfx (7 words). An out-of-range roll leaves `sfx_id`
/// unset in the C (UB); this plays id 0 then.
pub fn ftAction_80071FC8(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let w = &u(cmd).pseudo_random_sfx_0;
        let vol = w.volume() as u8;
        let pan = w.panning() as u8;
        let behavior = w.behavior();
        let range = w.random_range() as i32;
        next(cmd);
        let roll = ext::HSD_Randi(range);
        let mut sfx = 0i32;
        if (0..=5).contains(&roll) {
            skip_cmd(cmd, roll as usize);
            sfx = u(cmd).pseudo_random_sfx_1.sfx_id as i32;
            skip_cmd(cmd, (6 - roll) as usize);
        }
        next(cmd);
        play_behavior(fp, behavior, sfx, vol as i32, pan as i32);
    }
}

/// `ftAction_80072320`: stage-relative sfx (4 words). `direction` is unset for a
/// pitch_select past 3 in the C; 0.0 here.
pub fn ftAction_80072320(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let w = &u(cmd).stage_sfx_0;
        let pitch = w.pitch_select() as u8;
        let behavior = w.sfx_base();
        let base = w.x2_b0_7();
        let dir = match pitch {
            1 => 1.0f32,
            2 => -1.0,
            3 => fp.facing_dir,
            _ => 0.0,
        };
        next(cmd);
        let sfx = ext::ft_80087D0C(fp, u(cmd).stage_sfx_1.sfx_id);
        next(cmd);
        let p2 = u(cmd).stage_sfx_2.x2_b0_15() as i32;
        next(cmd);
        let p0 = u(cmd).stage_sfx_3.x2_b0_7() as i32;
        let p1 = u(cmd).stage_sfx_3.x3_b0_7() as i32;
        next(cmd);
        let slot = fp.player_id as i32 + fp.is_sub_fighter() as i32;
        let play = |s8: i32| ext::lbAudioAx_800264E4(ext::lbAudioAx_800263E8(dir, g, behavior, sfx, 127, 127, p0, p1, p2, s8, -1));
        let gw_or_samus = fp.kind == Ft_Kind_GameWatch || fp.kind == Ft_Kind_Samus;
        match base {
            0 => fp.x2160 = ext::lbAudioAx_800264E4(ext::lbAudioAx_800263E8(dir, g, behavior, sfx, 127, 127, p0, p1, p2, 0, -1)),
            1 => fp.x214C = play(slot + 0x36),
            2 => {
                if fp.x2225_b6() == 0 || gw_or_samus {
                    fp.x2144 = play(slot + 0x1E);
                }
            }
            3 => fp.x2150 = play(slot + 0x42),
            4 => fp.x2154 = play(slot + 0x4E),
            5 => fp.x2158 = play(slot + 0x5A),
            6 => {
                if fp.x2225_b6() == 0 || gw_or_samus {
                    fp.x2148 = play(slot + 0x2A);
                }
            }
            _ => {}
        }
    }
}

/// `ftAction_800726F4`: texture anim.
pub fn ftAction_800726F4(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).set_tex_anim;
        ext::ftAnim_800704F0(g, w.idx(), w.frame() as f32);
        if w.b() != 0 {
            ext::ftAnim_800704F0(g, w.idx2(), w.frame() as f32);
        }
    }
    next(cmd);
}
/// `ftAction_800727C8`: part anim.
pub fn ftAction_800727C8(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).part_anim;
        ext::ftAnim_ApplyPartAnim(g, w.unk1(), w.unk2(), w.unk3() as f32);
    }
    next(cmd);
}
/// `ftAction_8007283C`: part anim, no blend.
pub fn ftAction_8007283C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).part_anim;
        ext::ftAnim_ApplyPartAnim(g, w.unk1(), w.unk2(), 0.0);
    }
    next(cmd);
}
/// `ftAction_80072894`.
pub fn ftAction_80072894(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCommon_8007E83C(g, u(cmd).unk9.unk1(), u(cmd).unk9.unk2()) };
    next(cmd);
}
/// `ftAction_800728F8`.
pub fn ftAction_800728F8(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).unk10;
        if w.unk1() != 0 {
            ext::ftCommon_8007EC30(w.unk2(), w.unk3());
        } else {
            ext::ftCommon_8007EBAC(fpm(g), w.unk2(), w.unk3());
        }
    }
    next(cmd);
}
/// `ftAction_8007297C`.
pub fn ftAction_8007297C(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCommon_8007ECD4(fpm(g), u(cmd).unk11.unk1()) };
    next(cmd);
}
/// `ftAction_800729D4`.
pub fn ftAction_800729D4(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let w = &u(cmd).unk12;
        let (a, b) = (w.unk2() as i32, w.unk3() as i32);
        match w.unk1() {
            0 => ext::ftCommon_8007EEC8(fpm(g), a, b),
            1 => ext::ftCommon_8007EF5C(fpm(g), a),
            _ => {}
        }
    }
    next(cmd);
}
/// `ftAction_80072A5C`.
pub fn ftAction_80072A5C(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCo_800BFFD0(fpm(g), u(cmd).unk13.unk1(), u(cmd).unk13.unk2()) };
    next(cmd);
}
/// `ftAction_80072ABC`.
pub fn ftAction_80072ABC(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCo_800C0200(fpm(g), u(cmd).unk14.unk1()) };
    next(cmd);
}
/// `ftAction_80072B14`.
pub fn ftAction_80072B14(g: G, cmd: &mut CommandInfo) {
    unsafe { fpm(g).set_x221E_b1(u(cmd).unk15.unk1() as u8) };
    next(cmd);
}
/// `ftAction_80072B3C`.
pub fn ftAction_80072B3C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        fp.x2100 = u(cmd).unk16.unk4() as i8;
        fp.set_x2101_bits_8(u(cmd).unk16.unk3() as u8);
    }
    next(cmd);
}
/// `ftAction_80072B94`.
pub fn ftAction_80072B94(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCo_8009E318(g, u(cmd).unk17.unk1(), fpm(g).cur_anim_frame) };
    next(cmd);
}
/// `ftAction_80072BF4`: self damage.
pub fn ftAction_80072BF4(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::Fighter_TakeDamage_8006CC7C(fpm(g), u(cmd).unk18.damage_amount() as f32) };
    next(cmd);
}
/// `ftAction_80072C6C`.
pub fn ftAction_80072C6C(g: G, cmd: &mut CommandInfo) {
    ext::ft_8008A1B8(g, unsafe { u(cmd).unk19.unk1() });
    next(cmd);
}
/// `ftAction_80072CB0`.
pub fn ftAction_80072CB0(g: G, cmd: &mut CommandInfo) {
    unsafe { fpm(g).set_x2225_b2(u(cmd).unk20.unk1() as u8) };
    next(cmd);
}

/// The C's `cmd_words[3]` copy (word 0, an override for word 1, word 2) run through 80071B50.
unsafe fn sfx_with_override(g: G, cmd: &CommandInfo, sfx: i32) {
    let src = cmd.anon_8.u;
    let mut words: [CmdUnion; 3] = [*src, *src.add(1), *src.add(2)];
    words[1].sound_effect_1.sfx_id = sfx as u32;
    let mut c: CommandInfo = core::mem::zeroed();
    c.anon_8.u = words.as_mut_ptr();
    ftAction_80071B50(g, &mut c);
}

/// `ftAction_80072CD8`: footstep sfx/gfx.
pub fn ftAction_80072CD8(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let (mut sp64, mut sp60, mut gfx) = (0i32, 1i32, 0i32);
        if ext::ft_80084BFC(g, &mut sp64, &mut sp60, &mut gfx) {
            if sp64 != -1 {
                sfx_with_override(g, cmd, sp64);
            }
            if gfx != -1 {
                let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                let x8 = &*(*fp.ft_data).x8;
                let part = if u(cmd).footstep_fx_0.use_alt_bone() == 0 { x8.x13 } else { x8.x14 };
                ext::ftCo_8009F834(g, gfx, part as i32, 0, 0, &z, &z, 0.0);
            }
        }
        if sp60 != 0 {
            ftAction_80071B50(g, cmd);
            return;
        }
        skip_cmd(cmd, 3);
    }
}

/// `ftAction_80072E4C`.
pub fn ftAction_80072E4C(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let fp = fpm(g);
        let (mut sp64, mut sp60, mut gfx) = (0i32, 1i32, -1i32);
        let flag = u(cmd).unk_fx_0.x1_b0_7() as i32 & 1;
        if ext::ft_80084C38(g, &mut sp64, &mut sp60, &mut gfx) && flag == 0 && sp64 != -1 {
            sfx_with_override(g, cmd, sp64);
        }
        if gfx == -1 {
            // ((u16*) cmd->u)[1]: the low half of the first word, in the stream's byte order.
            let b = cmd.anon_8.u as *const u8;
            gfx = u16::from_be_bytes([*b.add(2), *b.add(3)]) as i32;
        }
        let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::ftCo_8009F834(g, gfx, FtPart_TopN as i32, 0, 0, &z, &z, 0.0);
        if sp60 != 0 {
            ext::ft_PlaySFX(fp, 0x46, 0x7F, 0x40);
            if flag == 0 {
                ftAction_80071B50(g, cmd);
            }
        }
        if sp60 == 0 || flag != 0 {
            skip_cmd(cmd, 3);
        }
        ext::ftCommon_8007EBAC(fp, 0x16, 0);
    }
}

/// `ftAction_80073008`: smash charge.
pub fn ftAction_80073008(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let rate = u(cmd).smash_charge_0.charge_rate() as f32;
        let frames = u(cmd).smash_charge_0.charge_frames() as f32;
        let mult = K * rate;
        next(cmd);
        let color = u(cmd).smash_charge_1.color_anim() as f32;
        next(cmd);
        ext::ftCo_800DEE84(g, color as i32, frames, mult);
    }
}

/// `ftAction_800730B8`.
pub fn ftAction_800730B8(g: G, cmd: &mut CommandInfo) {
    unsafe { ext::ftCo_800C8B60(fpm(g), u(cmd).unk21.unk1(), u(cmd).unk21.unk2()) };
    next(cmd);
}

/// `ftAction_80073118`: wind effect (4 words).
pub fn ftAction_80073118(g: G, cmd: &mut CommandInfo) {
    unsafe {
        let idx = u(cmd).wind_fx_0.bone() as u8;
        next(cmd);
        let x = K * u(cmd).wind_fx_1.timer() as f32;
        let y = K * u(cmd).wind_fx_1.x() as f32;
        next(cmd);
        let mag = K * u(cmd).wind_fx_2.y() as f32;
        let decay = K * u(cmd).wind_fx_2.mag() as f32;
        next(cmd);
        let timer = u(cmd).wind_fx_3.angle() as i32;
        let angle = K * u(cmd).wind_fx_3.decay() as f32;
        next(cmd);
        ext::ftCo_8009E714(g, idx, timer, x, y, mag, decay, angle);
    }
}

/// `ftAction_803C06E8`: the exec table (opcode - 10).
pub static ftAction_803C06E8: [FtCmd; 49] = [
    ftAction_80071028, ftAction_8007121C, ftAction_8007162C, ftAction_8007169C,
    ftAction_80071708, ftAction_80071784, ftAction_800717D8, ftAction_80071B50,
    ftAction_80071CCC, ftAction_80071820, ftAction_800718A4, ftAction_80071908,
    ftAction_8007192C, ftAction_80071950, ftAction_80071974, ftAction_80071998,
    ftAction_80071A14, ftAction_80071A58, ftAction_80071A9C, ftAction_80071AE8,
    ftAction_80071B28, ftAction_80071D40, ftAction_80071D94, ftAction_80071DCC,
    ftAction_80071E04, ftAction_80071F34, ftAction_80071F78, ftAction_80071FA0,
    ftAction_80071FC8, ftAction_80072320, ftAction_800726F4, ftAction_800727C8,
    ftAction_80072894, ftAction_800728F8, ftAction_8007297C, ftAction_800729D4,
    ftAction_80072A5C, ftAction_80072ABC, ftAction_80072B14, ftAction_80072B3C,
    ftAction_80072B94, ftAction_80072BF4, ftAction_80072C6C, ftAction_80072CB0,
    ftAction_80072CD8, ftAction_80072E4C, ftAction_80073008, ftAction_800730B8,
    ftAction_80073118,
];

/// `ftAction_803C07AC`: the anim-only table.
pub static ftAction_803C07AC: [FtCmd; 49] = [
    ftAction_800711DC, ftAction_800715EC, ftAction_8007168C, ftAction_800716F8,
    ftAction_80071774, ftAction_800717C8, ftAction_80071810, ftAction_80071CA4,
    ftAction_80071D30, ftAction_80071820, ftAction_800718A4, ftAction_80071908,
    ftAction_8007192C, ftAction_80071950, ftAction_80071974, ftAction_80071998,
    ftAction_80071A14, ftAction_80071A58, ftAction_80071A9C, ftAction_80071AE8,
    ftAction_80071B28, ftAction_80071D40, ftAction_80071D94, ftAction_80071DCC,
    ftAction_80071F0C, ftAction_80071F34, ftAction_80071F78, ftAction_80071FA0,
    ftAction_800722C8, ftAction_800726C0, ftAction_800726F4, ftAction_8007283C,
    ftAction_80072894, ftAction_8007296C, ftAction_800729C4, ftAction_80072A4C,
    ftAction_80072AAC, ftAction_80072B04, ftAction_80072B14, ftAction_80072B84,
    ftAction_80072BE4, ftAction_80072C5C, ftAction_80072C6C, ftAction_80072CB0,
    ftAction_80072E24, ftAction_80072FE0, ftAction_8007309C, ftAction_80073108,
    ftAction_8007320C,
];

/// `ftAction_803C0870`: words per command (opcode - 10).
pub static ftAction_803C0870: [u8; 49] = [
    5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7, 4, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 2, 1, 4,
];

#[inline]
unsafe fn opcode(cmd: &CommandInfo) -> u32 { (*cmd.anon_8.u).Command_09.id() }

/// Shared timer step of the three runners; `false` = stop.
fn runner_ready(cmd: &mut CommandInfo, speed: f32) -> bool {
    if unsafe { cmd.anon_8.u.is_null() } {
        return false;
    }
    if cmd.timer == F32_MAX {
        if cmd.frame_count >= speed {
            return false;
        }
        cmd.timer = -cmd.frame_count;
    } else if cmd.timer > 0.0 {
        return false;
    }
    true
}

/// `ftAction_80073240`: run the subaction script (exec table).
pub fn ftAction_80073240(g: G) {
    unsafe {
        let fp = fpm(g);
        let speed = fp.frame_speed_mul;
        let cmd = &mut fp.x3E4_fighterCmdScript;
        cmd.frame_count = fpm(g).cur_anim_frame + fpm(g).x898_unk;
        if cmd.anon_8.u.is_null() {
            return;
        }
        if cmd.timer != F32_MAX {
            cmd.timer -= speed;
        }
        loop {
            if !runner_ready(cmd, speed) {
                break;
            }
            let op = opcode(cmd);
            if !ext::Command_Execute(cmd, op) {
                if op == GENO_FTCMD_OP {
                    ext::Geno_FtCmd(g, cmd, GENO_MODE_EXEC);
                    if cmd.timer == F32_MAX { break; }
                    continue;
                }
                ftAction_803C06E8[(op - 10) as usize](g, cmd);
            }
            if cmd.timer == F32_MAX {
                break;
            }
        }
    }
}

/// `ftAction_80073354`: the anim-only runner (clears `throw_flags` when a wait elapses).
pub fn ftAction_80073354(g: G) {
    unsafe {
        let fp = fpm(g);
        let speed = fp.frame_speed_mul;
        fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
        fp.anon_2210.throw_flags = 0;
        let cmd = &mut *(&mut fp.x3E4_fighterCmdScript as *mut CommandInfo);
        if cmd.anon_8.u.is_null() {
            return;
        }
        if cmd.timer != F32_MAX {
            cmd.timer -= speed;
        }
        loop {
            if !runner_ready(cmd, speed) {
                break;
            }
            let timer = cmd.timer;
            let op = opcode(cmd);
            if !ext::Command_Execute(cmd, op) {
                if op == GENO_FTCMD_OP {
                    ext::Geno_FtCmd(g, cmd, GENO_MODE_ANIM);
                } else {
                    ftAction_803C07AC[(op - 10) as usize](g, cmd);
                }
            }
            if cmd.timer != timer && cmd.timer <= 0.0 {
                fpm(g).anon_2210.throw_flags = 0;
            }
            if cmd.timer == F32_MAX {
                break;
            }
        }
    }
}

/// `ftAction_8007349C`: the skip runner (advances by the word-count table).
pub fn ftAction_8007349C(g: G) {
    unsafe {
        let fp = fpm(g);
        let speed = fp.frame_speed_mul;
        fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
        let cmd = &mut fp.x3E4_fighterCmdScript;
        if cmd.anon_8.u.is_null() {
            return;
        }
        if cmd.timer != F32_MAX {
            cmd.timer -= speed;
        }
        loop {
            if !runner_ready(cmd, speed) {
                break;
            }
            let id = opcode(cmd);
            if !ext::Command_Execute(cmd, id) {
                if id == GENO_FTCMD_OP {
                    ext::Geno_FtCmd(g, cmd, GENO_MODE_SKIP);
                    if cmd.timer == F32_MAX { break; }
                    continue;
                }
                skip_cmd(cmd, ftAction_803C0870[(id - 10) as usize] as usize);
            }
            if cmd.timer == F32_MAX {
                break;
            }
        }
    }
}
