//! src/melee/ft/ftcoll.c: fighter hit / hurt / shield / reflect / absorb collision, the damage
//! logs, knockback, combo counting and the stage device (wind / bury / dynamics) checks.
//!
//! Already ported elsewhere and re-exported: `ftColl_800765E0` and `getEnvDmg`
//! (`crate::ftcoll`), `ftColl_8007AEE0/AEF8/AF10/AF28/B8A8/B8CC` (`crate::ft0df`).
//! The TARGET_PC build is ported: the COLL_* echo macros, the script / Geno / Mex hooks and the
//! enlarged echo log capacity. Every out-of-TU callee without a Rust port is an `ext` hook that
//! panics while unset (named exactly like the C function).
//!
//! Fused sites (tools/rust-port/fma_sites.py, every one an `fmadds` here): comboCount_Push 2,
//! inlineA0/inlineA1/inlineItemA0 1 each, the knockback formula 3 per branch
//! (80079AB0 / 80079C70 / 80079EA8 / 8007A06C), and 8007A06C's two 0.5*(a+b)-p sites.
#![allow(non_snake_case, static_mut_refs, non_upper_case_globals, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcoll::{DMG_LOG0_IDX, DMG_LOG1_IDX};
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::ftcommon2 as fc2;
#[allow(unused_imports)]
use crate::ftcommon3::ftco_get_cstick_angle;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::ft0df::{G as G_ft0df, fp};
use crate::types::*;
use core::ffi::c_void;
use melee_core::fp::fmadds;

pub use crate::ftcoll::{get_env_dmg};
pub type G = *mut HSD_GObj;
type F = *mut Fighter;
type HC = *mut HitCapsule;
type It = *mut Item;

pub mod ext {
    use super::{F, G, HC, It};
    use crate::types::*;
    use core::ffi::c_void;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $d:expr)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(unreachable_code)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => {
                        // a callee with a port in this crate runs it; the rest are unset seams
                        $( return $d; )?
                        melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported"))
                    }
                }
            }
        )*};
    }
    ext! {
        // ---- TARGET_PC echo capsules (pc/gameworld/script_echo.c) ----
        SCRIPTGAME_ECHO_CAPSULE_COUNT ScriptGame_EchoCapsuleCount(fp: F) -> i32;
        SCRIPTGAME_ECHO_CAPSULE ScriptGame_EchoCapsule(fp: F, i: i32) -> HC;
        SCRIPTGAME_ECHO_IS_CAPSULE ScriptGame_EchoIsCapsule(hit: HC) -> i32;
        SCRIPTGAME_ECHO_BLOCKED ScriptGame_EchoBlocked(hit: HC, fp: F) -> i32;
        SCRIPTGAME_ECHO_SAME_GROUP ScriptGame_EchoSameGroup(a: HC, b: HC) -> i32;
        SCRIPTGAME_ECHO_TARGET_ALLOWED ScriptGame_EchoTargetAllowed(src: F, tgt: F) -> i32;
        SCRIPTGAME_ECHO_CONNECTED ScriptGame_EchoConnected(hit: HC, fp: F);
        SCRIPTGAME_ECHO_PREPARE ScriptGame_EchoPrepare();
        SCRIPTGAME_ECHO_HIT_INDEX ScriptGame_EchoHitIndex(fp: F, hit: HC) -> i32;
        SCRIPTGAME_ECHO_REPORT_CONTEXT ScriptGame_EchoReportContext(a: F, v: F, hit: HC) -> i32;
        SCRIPTGAME_ECHO_CREDIT ScriptGame_EchoCredit(a: F, v: F, hit: HC, dmg: f32) -> i32;
        SCRIPTGAME_ECHO_SOURCE_X ScriptGame_EchoSourceX(hit: HC, fallback: i32) -> i32;
        // ---- TARGET_PC script / Geno / Mex hooks ----
        SCRIPT_GAME_EVENT Script_GameEvent(what: i32, a: i32, b: i32, c: i32, d: i32);
        SCRIPT_ENEMY_GAME_EVENT Script_EnemyGameEvent(e: i32, a: i32, b: i32, c: i32, d: i32, f: i32);
        SCRIPTGAME_ENEMY_CONTACT ScriptGame_EnemyContact(g: G, slot: i32, group: u32) -> i32;
        SCRIPTGAME_SKILL_PERFECT_SHIELD ScriptGame_SkillPerfectShield(fp: F, atk: F, kind: i32, dmg: f32);
        SCRIPTGAME_INTR_WIN_HIT ScriptGame_IntrWinHit(fp: F, kind: i32);
        GENO_COMBO_CONTACT Geno_ComboContact(fp: F, kind: i32);
        SCRIPTGAME_HIT_RULE_CONTACT ScriptGame_HitRuleContact(hit: HC, fp: F, v: f32, kind: i32) -> f32;
        SCRIPTGAME_FIGHTER_DEALT_DAMAGE ScriptGame_FighterDealtDamage(fp: F, dmg: f32) -> f32;
        SCRIPTGAME_HIT_RULE_WON ScriptGame_HitRuleWon(hit: HC, fp: F);
        GENO_HIT_STUN_BONUS Geno_HitStunBonus(atk: F, idx: i32, vic: F);
        GENO_HIT_FLAGS Geno_HitFlags(atk: F, idx: i32, vic: F) -> i32;
        SCRIPTGAME_HIT_RULE_PERCENT_QUEUE ScriptGame_HitRulePercentQueue(hit: HC, fp: F, dmg: f32);
        SCRIPTGAME_CRIT_HIT ScriptGame_CritHit(atk: F, hit: HC, vic: F, dmg: f32, kind: i32);
        SCRIPTGAME_REPORT_HIT_CONTEXT ScriptGame_ReportHitContext(atk: F, vic: F, element: u32, kind: i32);
        SCRIPTGAME_SKILL_HIT ScriptGame_SkillHit(atk: F, vic: F, dmg: f32);
        SCRIPTGAME_HIT_RULE_CONTEXT ScriptGame_HitRuleContext(hit: HC, fp: F);
        SCRIPTGAME_HIT_RULE_DAMAGE ScriptGame_HitRuleDamage(hit: HC) = unsafe { melee_runtime::tu::script_hit::ScriptGame_HitRuleDamage(hit) };
        SCRIPTGAME_ITEM_MOD_DAMAGE ScriptGame_ItemModDamage(item: It, dmg: f32) -> f32;
        SCRIPT_CLANK_WANTED Script_ClankWanted() -> i32;
        SCRIPT_CLANK Script_Clank(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32, i: i32, j: i32, k: i32, l: i32);
        SCRIPTMODS_CLANK ScriptMods_Clank(a: i32, b: i32, da: f32, db: f32);
        GENO_ITEM_FAMILY_EVENT Geno_ItemFamilyEvent(g: G, what: i32, slot: i32);
        GENO_ARTICLE_STUN_BONUS Geno_ArticleStunBonus(item: It, hit: HC, fp: F);
        GENO_AUTOLINK Geno_Autolink(atk: F, hit: HC, dir: &mut f32, angle: &mut f32, kb: &mut f32) -> i32;
        GENO_DEFINE_IS_NONE Geno_DefineIsNone(kind: i32) -> i32;
        GENO_DEFINE_TEMPLATE_SKELETON Geno_DefineTemplateSkeleton(kind: i32) -> i32;
        MEX_ENABLED Mex_Enabled(name: &str) -> i32;
        // ---- game callees not yet ported ----
        IT_GET_OWNER itGetOwner(g: G) -> G;
        IT_GET_ATTACK_ID itGetAttackId(g: G) -> i32;
        LBCOLL_80008688 lbColl_80008688(hit: HC, kind: i32, victim: *mut c_void) -> bool;
        LBCOLL_80008820 lbColl_80008820(hit: HC, kind: i32, victim: *mut c_void) -> bool;
        LBCOLL_COPY_HIT_CAPSULE lbColl_CopyHitCapsule(src: HC, dst: HC);
        LBCOLL_80008440 lbColl_80008440(hit: HC) = unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(hit) };
        LBCOLL_80008428 lbColl_80008428(hit: HC) = unsafe { melee_runtime::tu::lbcollision::lbColl_80008428(hit) };
        LBCOLL_80008434 lbColl_80008434(hit: HC) = unsafe { melee_runtime::tu::lbcollision::lbColl_80008434(hit) };
        LBCOLL_80005BB0 lbColl_80005BB0(hit: HC, i: i32) -> i32;
        LBCOLL_8000ACFC lbColl_8000ACFC(victim: *mut c_void, hit: HC) -> bool;
        LBCOLL_80007ECC lbColl_80007ECC(hit: HC, hurt: *mut HurtCapsule, mtx: MtxPtr, a: f32, b: f32, z: f32) -> bool;
        LBCOLL_80007AFC lbColl_80007AFC(a: HC, b: HC, sa: f32, sb: f32) -> bool;
        LBCOLL_80007BCC lbColl_80007BCC(hit: HC, res: *mut HitResult, mtx: MtxPtr, i: i32, a: f32, b: f32, z: f32) -> bool;
        LBCOLL_8000805C lbColl_8000805C(hit: HC, hurt: *mut HurtCapsule, mtx: MtxPtr, i: i32, a: f32, b: f32, z: f32) -> bool;
        LBCOLL_80007DD8 lbColl_80007DD8(hit: HC, res: *mut HitResult, mtx: MtxPtr, pos: *mut Vec3, dist: *mut f32, scl: f32);
        LBCOLL_80008248 lbColl_80008248(hit: HC, hurt: *mut HurtCapsule, mtx: MtxPtr, a: f32, b: f32, z: f32) -> bool;
        LBCOLL_80008D30 lbColl_80008D30(hit: HC, desc: *mut c_void);
        EFSYNC_SPAWN efSync_Spawn(id: i32, g: G, pos: *const Vec3, arg: *const f32);
        FTCO_80094138 ftCo_80094138(fp: F);
        PL_8003E150 pl_8003E150(slot: i32, sub: i32);
        PL_8003E17C pl_8003E17C(slot: i32, sub: i32, g: G);
        PL_80038144 pl_80038144(a: G, v: G, i: i32, p: *mut c_void, inst: u16, grounded: i32, prev: i32);
        PL_8003EB30 pl_8003EB30(dmg: f32, a: i32, b: i32, c: i32, d: i32, e: i32);
        PL_80041B08 pl_80041B08(slot: i32, sub: *mut c_void, v: u16);
        PL_8003D60C pl_8003D60C(kind: i32) -> bool;
        PL_80037ECC pl_80037ECC(g: G);
        IT_8026FAC4 it_8026FAC4(item: It, hit: HC, mode: i32, fp: *mut c_void, b: bool);
        IT_8026FC00 it_8026FC00(item: It, hit: HC, mode: i32, fp: F);
        IT_80272818 it_80272818(item: It) -> F;
        IT_8026B1D4 it_8026B1D4(g: G, hit: HC) -> f32;
        IT_8026B588 it_8026B588() -> i32;
        FT_80084CE4 ft_80084CE4(a: F, v: F) -> bool;
        FT_80088080 ft_80088080(fp: F);
        FT_80089228 ft_80089228(fp: F, id: i32, inst: i32, dmg: f32) -> f32 = unsafe { crate::stale::apply_staling(&*fp, id, inst, dmg) };
        FTCO_800DEEB8 ftCo_800DEEB8(fp: F, dmg: f32) -> f32 = unsafe { crate::ft0df::ftco_800deeb8(&*fp, dmg) };
        GM_8016B0D4 gm_8016B0D4() -> bool;
        GM_8016B1C4 gm_8016B1C4() -> bool;
        GM_8016B248 gm_8016B248() -> f32;
        PLAYER_GET_DEFENSE_RATIO Player_GetDefenseRatio(slot: i32) -> f32;
        PLAYER_GET_ATTACK_RATIO Player_GetAttackRatio(slot: i32) -> f32;
        PLSTALE_UPDATE_STALE_MOVES_FROM_FIGHTER plStale_UpdateStaleMovesFromFighter(a: G, b: G);
        PLSTALE_UPDATE_STALE_MOVES_FROM_ITEM plStale_UpdateStaleMovesFromItem(a: G, b: G);
        FTCO_800C0A28 ftCo_800C0A28(g: G, ground: *mut c_void, kind: u32) -> bool;
        FTCO_800C0874 ftCo_800C0874(g: G, ground: *mut c_void, kind: u32);
        FTCO_800C08A0 ftCo_800C08A0(g: G, ground: G, desc: *mut DynamicsDesc, kind: u32);
        FTCO_800C0B20 ftCo_800C0B20(g: G);
        FTLIB_80086960 ftLib_80086960(g: G) -> bool = crate::ftlib::ftlib_80086960(g);
        FTCOMMON_8007F804 ftCommon_8007F804(fp: F) -> MtxPtr = unsafe { crate::ftcommon2::ftcommon_8007f804(&mut *fp) };
        FTCOMMON_8007EBAC ftCommon_8007EBAC(fp: F, a: u32, b: u32) = unsafe { crate::ftcommon2::ftcommon_8007ebac(&mut *fp, a, b) };
        FTCO_CALC_Y_SCALED_KNOCKBACK ftCo_CalcYScaledKnockback(a: f32, scale: f32, c: f32) -> f32 = crate::tu::ftchangeparam::ftCo_CalcYScaledKnockback(a, scale, c);
        FTCO_800C0694 ftCo_800C0694(fp: F) -> i32 = unsafe { crate::tu::ftdevice::ftco_800c0694(&mut *fp) as i32 };
        FTCO_800C0200 ftCo_800C0200(fp: F, a: i32) = unsafe { crate::tu::ftcolanim::ftco_800c0200(&mut *fp, a) };
        FTCO_800BFFD0 ftCo_800BFFD0(fp: F, a: i32, b: i32) -> bool = unsafe { crate::tu::ftcolanim::ftco_800bffd0(&mut *fp, a, b) };
    }
    /// `it_804D6D28` (ItemCommonData; set by the item loader). TODO(group5 it) move to the item crate.
    pub use melee_runtime::shared::it_804D6D28;
    /// `ScriptStageHostHidden` (TARGET_PC).
    pub static mut ScriptStageHostHidden: i32 = 0;
    pub fn icd() -> &'static ItemCommonData {
        let p = unsafe { it_804D6D28 };
        assert!(!p.is_null(), "TODO it_804D6D28 not loaded");
        unsafe { &*p }
    }
}

// ---- .data / .bss ------------------------------------------------------------------------

/// `ECHO_COLLISION_CAPS` / `ECHO_COLLISION_LOG_CAPACITY` (pc/gameworld/script_echo_limits.h).
pub const ECHO_COLLISION_CAPS: usize = 4 + 8 * 5;
pub const ECHO_COLLISION_LOG_CAPACITY: usize = 20 + 12 * ECHO_COLLISION_CAPS;

pub static ftColl_803C0C40: [i32; 3] = [141, 142, 143];
pub static ftColl_803C0C4C: [i32; 3] = [107, 108, 109];

/// `dmg_log0` (static) / `dmg_log1`.
pub static mut dmg_log0: [DmgLogEntry; ECHO_COLLISION_LOG_CAPACITY] = unsafe { core::mem::zeroed() };
pub static mut dmg_log1: [DmgLogEntry; ECHO_COLLISION_LOG_CAPACITY] = unsafe { core::mem::zeroed() };
/// `ftColl_804D6560` (static).
pub static mut ftColl_804D6560: [i8; ECHO_COLLISION_CAPS] = [0; ECHO_COLLISION_CAPS];

#[inline(always)]
fn log0_idx() -> i32 { unsafe { crate::ftcoll::DMG_LOG0_IDX } }
#[inline(always)]
fn log1_idx() -> i32 { unsafe { crate::ftcoll::DMG_LOG1_IDX } }
#[inline(always)]
fn set_log0_idx(v: i32) { unsafe { crate::ftcoll::DMG_LOG0_IDX = v } }
#[inline(always)]
fn set_log1_idx(v: i32) { unsafe { crate::ftcoll::DMG_LOG1_IDX = v } }

/// `hit_effect_ids` (static), indexed by HitElement.
pub static hit_effect_ids: [i32; 17] = [
    Ef_Id_Unk1000 as i32, Ef_Id_Unk1002 as i32, Ef_Id_Unk1001 as i32, Ef_Id_Unk1004 as i32, Ef_Id_Unk1145 as i32,
    Ef_Id_Unk1005 as i32, -1, -1, -1, Ef_Id_Unk1000 as i32, Ef_Id_Unk1000 as i32, -1, -1, Ef_Id_Unk1046 as i32, -1, -1, 0,
];

const PLINK_FIGHTER: i32 = HSD_GOBJ_PLINK_FIGHTER as i32;
const PLINK_ITEM: i32 = HSD_GOBJ_PLINK_ITEM as i32;
const HC_DISABLED: u32 = HitCapsule_Disabled as u32;

// ---- helpers ------------------------------------------------------------------------------

#[inline(always)]
unsafe fn fpg(g: G) -> F {
    (*g).user_data as F
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
#[inline(always)]
fn cap_count(fp: F) -> u32 {
    ext::ScriptGame_EchoCapsuleCount(fp) as u32
}
#[inline(always)]
fn cap(fp: F, i: u32) -> HC {
    ext::ScriptGame_EchoCapsule(fp, i as i32)
}
#[inline(always)]
fn is_echo(hit: HC) -> bool {
    ext::ScriptGame_EchoIsCapsule(hit) != 0
}
#[inline(always)]
fn same_group(a: HC, b: HC) -> bool {
    ext::ScriptGame_EchoSameGroup(a, b) != 0
}
#[inline(always)]
fn echo_allowed(s: F, t: F) -> bool {
    ext::ScriptGame_EchoTargetAllowed(s, t) != 0
}
#[inline(always)]
fn spawn(id: i32, pos: *const Vec3) {
    ext::efSync_Spawn(id, core::ptr::null_mut(), pos, core::ptr::null());
}
#[inline(always)]
fn sfx(fp: F, id: i32) {
    crate::hooks::play_sfx(unsafe { &mut *fp }, id, 0x7F, 0x40);
}
#[inline(always)]
fn ga(v: GroundOrAir) -> i32 {
    v as i32
}
const GROUND: i32 = GA_Ground as i32;
const AIR: i32 = GA_Air as i32;
#[inline(always)]
fn dir_lt(a: f32, b: f32, t: f32, f: f32) -> f32 {
    if a < b { t } else { f }
}
#[inline(always)]
fn mtx(fp: F) -> MtxPtr {
    ext::ftCommon_8007F804(fp)
}

// ---- combo counting -----------------------------------------------------------------------

/// `ftColl_800763C0`: combo count logic.
pub fn ftcoll_800763c0(attacker: G, victim: G, attack_id: i32) {
    melee_core::ct!("ftcoll_800763c0");
    if attacker == victim {
        return;
    }
    unsafe {
        let fp = &mut *fpg(attacker);
        let cur = fp.x2094;
        if cur.is_null() {
            fp.x208C = attack_id as _;
            fp.x2090 = 1;
            fp.x2094 = victim;
        } else if cur == victim {
            if attack_id != 1 && fp.x208C as i32 == attack_id {
                fp.x2090 = fp.x2090.wrapping_add(1);
                if fp.x2090 as i32 >= cd().x4C4 as i32 {
                    fp.x2092 = cd().x4D8 as _;
                }
            } else {
                fp.x2090 = 0;
                fp.x208C = attack_id as _;
            }
        }
    }
}

/// `ftColl_80076444`.
pub fn ftcoll_80076444(attacker: G, victim: G) {
    melee_core::ct!("ftcoll_80076444");
    let id = unsafe { (*fpg(attacker)).x2068_attackID } as i32;
    ftcoll_800763c0(attacker, victim, id);
}

/// `ftColl_8007646C`: combo count from an item's owner.
pub fn ftcoll_8007646c(item: G, victim: G) {
    melee_core::ct!("ftcoll_8007646c");
    let owner = ext::itGetOwner(item);
    let msid = ext::itGetAttackId(item);
    if ext::ftLib_80086960(owner) {
        ftcoll_800763c0(owner, victim, msid);
    }
}

/// `ftColl_800764DC`: check to end the victim's combo.
pub fn ftcoll_800764dc(g: G) {
    melee_core::ct!("ftcoll_800764dc");
    unsafe {
        let fp = &mut *fpg(g);
        if fp.x2098 != 0 {
            fp.x2098 -= 1;
        }
        if !fp.x2094.is_null() {
            let fp1 = &*fpg(fp.x2094);
            if fp1.x221C_b6() == 0 && fp1.x2098 == 0 {
                fp.x2094 = core::ptr::null_mut();
            }
        }
    }
}

/// `comboCount_Push` (static inline).
fn combo_count_push(fp: &mut Fighter) {
    let pos = fp.coll_data.floor.normal;
    let v = if (fp.x2090 as i32) < cd().x4C8 as i32 { cd().x4D0 } else { cd().x4D4 };
    let t = fp.facing_dir * v;
    fp.cur_pos.x = -fmadds(pos.y, t, -fp.cur_pos.x);
    fp.cur_pos.y = -fmadds(-pos.x, t, -fp.cur_pos.y);
}

/// `ftColl_80076528`.
pub fn ftcoll_80076528(g: G) {
    melee_core::ct!("ftcoll_80076528");
    unsafe {
        let fp = &mut *fpg(g);
        let v = fp.x2092;
        if v != 0 {
            fp.x2092 = v.wrapping_sub(1);
            if fp.victim_gobj.is_null() && ga(fp.ground_or_air) == GROUND {
                combo_count_push(fp);
            }
        }
    }
}

/// `ftColl_800765AC`: clear `victim` from every fighter's combo target.
pub fn ftcoll_800765ac(victim: G) {
    melee_core::ct!("ftcoll_800765ac");
    let mut cur = crate::hooks::plink_head(PLINK_FIGHTER);
    while !cur.is_null() {
        unsafe {
            let fp = &mut *fpg(cur);
            if victim == fp.x2094 {
                fp.x2094 = core::ptr::null_mut();
            }
            cur = (*cur).next;
        }
    }
}

/// `ftColl_800765F0`.
pub fn ftcoll_800765f0(fp: &Fighter, victim: G, mut arg2: f32) -> f32 {
    melee_core::ct!("ftcoll_800765f0");
    let cur = fp.victim_gobj;
    if !cur.is_null() && fp.anon_221b.x221B_b5() == 0 && cur != victim {
        arg2 *= cd().x128;
    }
    if fp.motion_id == ftCo_MS_DamageIce as _ {
        arg2 *= cd().x714;
    }
    arg2 * fp.dmg.x182c_behavior
}

/// `ftColl_80076640`.
pub fn ftcoll_80076640(fp: &mut Fighter, dmg: &mut f32) -> bool {
    melee_core::ct!("ftcoll_80076640");
    let env = get_env_dmg(*dmg);
    if fp.x221C_b4() != 0 {
        fp.dmg.x1834 -= *dmg;
        if fp.dmg.x1834 < 0.0 {
            *dmg = -fp.dmg.x1834;
            fp.set_x221C_b4(0);
        }
    }
    if fp.x221C_b4() == 0 {
        // HSD_ASSERTREPORT(*dmg > 500) only reports (OSReport + __assert, which the port logs).
        fp.dmg.x1838_percentTemp += *dmg;
        if env > fp.dmg.x183C_applied as i32 {
            fp.dmg.x183C_applied = env as _;
        }
        return true;
    }
    false
}

/// `ftColl_80076764`: append an environment entry to damage log 0.
pub fn ftcoll_80076764(arg0: i32, arg1: i32, arg2: G, arg3: *mut DynamicsDesc, fp: &Fighter, hurt: *mut FighterHurtCapsule) {
    melee_core::ct!("ftcoll_80076764");
    let idx = log0_idx();
    if (idx as u32 as usize) < ECHO_COLLISION_LOG_CAPACITY {
        unsafe {
            let e = &mut dmg_log0[idx as usize];
            e.x0 = arg0 as _;
            e.kind = arg1 as _;
            e.gobj = arg2;
            e.anon_c.unk_anim0 = arg3;
            e.anon_10.hurt1 = hurt;
            e.pos = fp.cur_pos;
            e.size_of_xC = (*arg3).count as usize;
        }
        set_log0_idx(idx + 1);
    } else {
        panic!("damage log over {}!!", ECHO_COLLISION_LOG_CAPACITY);
    }
}

/// `tiplog` (static): append a phantom hit to damage log 1.
fn tiplog(kind: i32, gobj: G, hit0: HC, hit1: HC, len: u32, temp_dmg: f32) {
    let idx = log1_idx();
    if (idx as u32 as usize) < ECHO_COLLISION_LOG_CAPACITY {
        unsafe {
            let e = &mut dmg_log1[idx as usize];
            e.x0 = 1;
            e.kind = kind as _;
            e.gobj = gobj;
            e.anon_c.hit0 = hit0;
            e.anon_10.hit1 = hit1;
            e.pos = (*hit0).hurt_coll_pos;
            e.x20 = temp_dmg;
            e.size_of_xC = len as usize;
        }
        set_log1_idx(idx + 1);
    } else {
        panic!("tip log over {}!!", ECHO_COLLISION_LOG_CAPACITY);
    }
}

/// `ftColl_80076808`: register `victim` on every enabled capsule of `hit`'s group.
pub fn ftcoll_80076808(fp: F, hit: HC, arg2: i32, victim: *mut c_void, arg4: bool) {
    melee_core::ct!("ftcoll_80076808");
    let mut j: i8 = 0;
    let mut i = 0u32;
    while i < cap_count(fp) {
        let cur = cap(fp, i);
        if unsafe { (*cur).state } != HC_DISABLED && same_group(cur, hit) && ext::lbColl_80008688(cur, arg2, victim) && arg4 {
            unsafe { ftColl_804D6560[j as u8 as usize] = 0 };
        }
        i += 1;
        j = j.wrapping_add(1);
    }
}

/// `ftColl_800768A0`: copy the victim list from a sibling capsule, else reset it.
pub fn ftcoll_800768a0(fp: F, dst: HC) {
    melee_core::ct!("ftcoll_800768a0");
    let mut i = 0u32;
    while i < cap_count(fp) {
        let h = cap(fp, i);
        if h != dst && unsafe { (*h).state } != HC_DISABLED && same_group(h, dst) {
            ext::lbColl_CopyHitCapsule(h, dst);
            return;
        }
        i += 1;
    }
    ext::lbColl_80008440(dst);
}

fn hitlag_dir(fp: &mut Fighter, int_dmg: i32, dir: f32) {
    fp.dmg.x191C = fmadds(int_dmg as f32, cd().x3D0, cd().x3D4) as _;
    fp.dmg.facing_dir = dir;
}

/// `inlineA0` (static inline).
unsafe fn inline_a0(fp0: F, fp1: F, hit1: HC, ef_pos: *const Vec3, dmg: f32) {
    let int_dmg = get_env_dmg(dmg);
    let mut j: i8 = 0;
    let mut i = 0u32;
    while i < cap_count(fp1) {
        let cur = cap(fp1, i);
        if (*cur).state != HC_DISABLED && same_group(cur, hit1) && ext::lbColl_80008688(cur, 3, fp0 as *mut c_void) {
            ftColl_804D6560[j as u8 as usize] = 0;
        }
        i += 1;
        j = j.wrapping_add(1);
    }
    let f1 = &mut *fp1;
    if !is_echo(hit1) && int_dmg > f1.dmg.int_value as i32 {
        f1.dmg.int_value = int_dmg as _;
        if (*hit1).x40_b1() == 1 && ga(f1.ground_or_air) == GROUND {
            let d = dir_lt(f1.cur_pos.x, (*fp0).cur_pos.x, 1.0, -1.0);
            hitlag_dir(f1, int_dmg, d);
        }
    }
    spawn(1052, ef_pos);
}

/// `inlineA1` (static inline).
unsafe fn inline_a1(fp0: F, hit0: HC, fp1: F, hit1: HC, ef_pos: *const Vec3, dmg: f32) -> bool {
    let int_dmg = get_env_dmg(dmg);
    let mut i = 0u32;
    while i < cap_count(fp0) {
        let h = cap(fp0, i);
        if (*h).state != HC_DISABLED && same_group(h, hit0) {
            ext::lbColl_80008688(h, 3, fp1 as *mut c_void);
        }
        i += 1;
    }
    let f0 = &mut *fp0;
    if !is_echo(hit0) && int_dmg > f0.dmg.int_value as i32 {
        f0.dmg.int_value = int_dmg as _;
        if (*hit0).x40_b1() == 1 && ga(f0.ground_or_air) == GROUND {
            let d = if (*fp1).cur_pos.x > f0.cur_pos.x { 1.0 } else { -1.0 };
            hitlag_dir(f0, int_dmg, d);
        }
    }
    spawn(1052, ef_pos);
    ftcoll_800784b4(fp1, hit0, hit1);
    true
}

/// `script_clank_observe` (pc/gameworld/script_clank_observe.inc).
unsafe fn script_clank_observe(a: F, ha: HC, b: F, item: It, hb: HC) {
    let x3cc = cd().x3CC as i32;
    let ca = (((*ha).damage as i32).wrapping_sub(x3cc) < (*hb).damage as i32) as i32;
    let cb = (((*hb).damage as i32).wrapping_sub(x3cc) < (*ha).damage as i32) as i32;
    if ca == 0 && cb == 0 {
        return;
    }
    if !b.is_null() {
        ext::ScriptMods_Clank((*a).player_id as i32, (*b).player_id as i32, (*ha).damage, (*hb).damage);
    }
    if ext::Script_ClankWanted() == 0 {
        return;
    }
    let (pa, pb) = ((*ha).hurt_coll_pos, (*hb).hurt_coll_pos);
    ext::Script_Clank(
        (*a).player_id as i32,
        if b.is_null() { -1 } else { (*b).player_id as i32 },
        if item.is_null() { 0 } else { (*item).entity as usize as i32 },
        if item.is_null() { -1 } else { (*item).kind as i32 },
        ((pa.x + pb.x) * 0.5).to_bits() as i32,
        ((pa.y + pb.y) * 0.5).to_bits() as i32,
        ((pa.z + pb.z) * 0.5).to_bits() as i32,
        (*ha).damage.to_bits() as i32,
        (*hb).damage.to_bits() as i32,
        ca | (cb << 1),
        (*a).gobj as usize as i32,
        if b.is_null() { (*item).entity as usize as i32 } else { (*b).gobj as usize as i32 },
    );
}

/// `ftColl_8007699C`: hitbox-vs-hitbox clank between two fighters.
pub fn ftcoll_8007699c(fp0: F, hit0: HC, fp1: F, hit1: HC) -> bool {
    melee_core::ct!("ftcoll_8007699c");
    unsafe {
        let (a, b) = ((*hit0).hurt_coll_pos, (*hit1).hurt_coll_pos);
        let mid = Vec3 { x: 0.5 * (a.x + b.x), y: 0.5 * (a.y + b.y), z: 0.5 * (a.z + b.z) };
        script_clank_observe(fp0, hit0, fp1, core::ptr::null_mut(), hit1);
        let x3cc = cd().x3CC as i32;
        let dmg = (*hit1).damage;
        if (dmg as i32).wrapping_sub(x3cc) < (*hit0).damage as i32 {
            inline_a0(fp0, fp1, hit1, &mid, dmg);
        }
        let dmg = (*hit0).damage;
        if (dmg as i32).wrapping_sub(x3cc) < (*hit1).damage as i32 {
            return inline_a1(fp0, hit0, fp1, hit1, &mid, dmg);
        }
        false
    }
}

/// `ftColl_80076CBC`: `fp0`'s hitbox struck `fp1`'s shield.
pub fn ftcoll_80076cbc(fp0: F, hit0: HC, fp1: F) {
    melee_core::ct!("ftcoll_80076cbc");
    ftcoll_80076808(fp0, hit0, 1, fp1 as *mut c_void, false);
    unsafe {
        ext::ScriptGame_EchoConnected(hit0, fp1);
        {
            let f1 = &*fp1;
            ext::Script_GameEvent(16, f1.player_id as i32, f1.motion_id as i32, 7, f1.is_sub_fighter() as i32);
            if f1.x221C_b2() != 0 {
                ext::Script_GameEvent(16, f1.player_id as i32, f1.motion_id as i32, 8, f1.is_sub_fighter() as i32);
            }
            if f1.x221C_b2() != 0 {
                ext::ScriptGame_SkillPerfectShield(fp1, fp0, 0, (*hit0).damage);
            }
            ext::ScriptGame_IntrWinHit(fp0, 1);
            ext::Geno_ComboContact(fp0, 1);
        }
        let int_dmg = get_env_dmg((*hit0).damage);
        {
            let (f0, f1) = (&mut *fp0, &*fp1);
            if !is_echo(hit0) && int_dmg > f0.dmg.x1924 as i32 {
                f0.dmg.x1924 = int_dmg as _;
                if ga(f0.ground_or_air) == GROUND {
                    f0.dmg.x1928 = f1.lightshield_amount * int_dmg as f32;
                    f0.dmg.x192c = if f1.cur_pos.x > f0.cur_pos.x { 1.0 } else { -1.0 };
                }
            }
        }
        {
            let f1 = &mut *fp1;
            if int_dmg > f1.x19A4 as i32 {
                f1.x19A4 = int_dmg as _;
                f1.specialn_facing_dir = if f1.cur_pos.x > (*fp0).cur_pos.x { -1.0 } else { 1.0 };
                f1.x19B0 = (*hit0).element as _;
                f1.x19A8 = (*fp0).gobj;
            }
            if f1.x221C_b2() == 0 {
                let tmp = int_dmg.wrapping_add((*hit0).x34);
                f1.x19A0_shieldDamageTaken = f1.x19A0_shieldDamageTaken.wrapping_add(if tmp < 0 { 0 } else { tmp } as _);
                f1.x19BC_shieldDamageTaken3 = (*fp0).player_id as _;
                f1.set_x221F_b6((*fp0).is_sub_fighter() as _);
                spawn(1052, &(*hit0).hurt_coll_pos);
            } else {
                ext::ftCo_80094138(fp1);
                ext::ftCo_800BFFD0(&mut *fp1, 118, 0);
                sfx(fp1, 104);
                ext::pl_8003E150((*fp1).player_id as i32, (*fp1).is_sub_fighter() as i32);
                spawn(27, &(*hit0).hurt_coll_pos);
            }
        }
        let tmp = (*fp1).shield_unk0;
        if tmp != 0.0 {
            if !is_echo(hit0) {
                (*fp0).x1964 = tmp;
            }
            (*fp1).x1964 = tmp;
        }
    }
}

/// `checkTipLog` (static inline): is `victim` among the capsule's phantom victims?
unsafe fn check_tip_log(victim: *mut c_void, hit: HC) -> u32 {
    let v = &(*hit).victims_2;
    let mut i = 0;
    while i < v.len() {
        if v[i].victim as *mut c_void == victim {
            break;
        }
        i += 1;
    }
    if i == v.len() { HitCapsule_Disabled as u32 } else { HitCapsule_Enabled as u32 }
}

/// `inlineB1` (static inline): phantom (tip) range.
unsafe fn inline_b1(hit0: HC) -> bool {
    (*hit0).coll_distance < cd().x7A8
}

/// `inlineB2` (static inline).
fn inline_b2(fp1: &mut Fighter, dmg: f32, count: i32) -> bool {
    if fp1.x221C_b4() == 0 {
        fp1.dmg.x1838_percentTemp = fp1.dmg.x1838_percentTemp + dmg;
        if count > fp1.dmg.x183C_applied as i32 {
            fp1.dmg.x183C_applied = count as _;
        }
        return true;
    }
    false
}

/// `inlineB3` (static inline): staled / scaled damage of `hit0` against `fp1` (TARGET_PC).
unsafe fn inline_b3(fp0: F, hit0: HC, fp1: F) -> f32 {
    let f1 = &*fp1;
    let vg = f1.victim_gobj;
    let gobj = (*fp0).gobj;
    let mut ret = (*hit0).damage;
    if !vg.is_null() && f1.anon_221b.x221B_b5() == 0 && vg != gobj {
        ret *= cd().x128;
    }
    if f1.motion_id == ftCo_MS_DamageIce as _ {
        ret *= cd().x714;
    }
    ext::ScriptGame_HitRuleContact(hit0, fp1, ext::ScriptGame_FighterDealtDamage(fp0, ret * f1.dmg.x182c_behavior), 0)
}

/// `ftColl_80076ED8`: `fp0`'s hitbox connected with `fp1`'s hurtbox `hit1`.
pub fn ftcoll_80076ed8(fp0: F, hit0: HC, fp1: F, hit1: HC) -> bool {
    melee_core::ct!("ftcoll_80076ed8");
    unsafe {
        let mut dmg = inline_b3(fp0, hit0, fp1);
        if inline_b1(hit0) {
            if log0_idx() == 0 && (*fp1).dmg.x189C_unk_num_frames == 0.0 && check_tip_log(fp1 as *mut c_void, hit0) == HitCapsule_Disabled as u32 {
                let mut temp_dmg = 0.5 * dmg;
                if temp_dmg as i32 == 0 && dmg != 0.0 {
                    temp_dmg = 1.0;
                }
                let mut i = 0u32;
                while i < cap_count(fp0) {
                    let cur = cap(fp0, i);
                    if (*cur).state != HC_DISABLED && same_group(cur, hit0) {
                        ext::lbColl_80008820(cur, 0, fp1 as *mut c_void);
                    }
                    i += 1;
                }
                let f1 = &mut *fp1;
                if f1.x1988 == 0 && f1.x198C == 0 && f1.x221D_b6() == 0 && (*hit1).state == HC_DISABLED {
                    let count = (*hit0).unk_count;
                    let mut len = count >> 1;
                    if len == 0 && count != 0 {
                        len = 1;
                    }
                    let ti = temp_dmg as i32;
                    if ti > f1.dmg.x1840 as i32 {
                        f1.dmg.x1840 = ti as _;
                    }
                    let mut fp = fp0;
                    if !(*fp0).x1064_thrownHitbox.anon_134.owner.is_null() && !is_echo(hit0) {
                        fp = fpg((*fp0).x1064_thrownHitbox.anon_134.owner);
                    }
                    tiplog((*fp).kind as i32, (*fp).gobj, hit0, hit1, len, temp_dmg);
                }
                ftcoll_80078488(fp1);
                return true;
            }
        } else {
            let int_dmg = get_env_dmg(dmg);
            set_log1_idx(0);
            let mut inner_ret = false;
            let mut i = 0u32;
            while i < cap_count(fp0) {
                let cur = cap(fp0, i);
                if (*cur).state != HC_DISABLED && same_group(cur, hit0) {
                    ext::lbColl_80008688(cur, 0, fp1 as *mut c_void);
                }
                i += 1;
            }
            if !is_echo(hit0) && int_dmg > (*fp0).dmg.x1914 as i32 {
                (*fp0).dmg.x1914 = int_dmg as _;
            }
            let f1 = &mut *fp1;
            if f1.x1988 == 0 && f1.x198C == 0 && f1.x221D_b6() == 0 && (*hit1).state == HC_DISABLED {
                let count = get_env_dmg(dmg);
                if f1.x221C_b4() != 0 {
                    f1.dmg.x1834 = f1.dmg.x1834 - dmg;
                    if f1.dmg.x1834 < 0.0 {
                        dmg = -f1.dmg.x1834;
                        f1.set_x221C_b4(0);
                    }
                }
                if inline_b2(f1, dmg, count) {
                    let mut fp = fp0;
                    if !(*fp0).x1064_thrownHitbox.anon_134.owner.is_null() && !is_echo(hit0) {
                        fp = fpg((*fp0).x1064_thrownHitbox.anon_134.owner);
                    }
                    let len = (*hit0).unk_count as usize;
                    let (gobj, kind) = ((*fp).gobj, (*fp).kind);
                    let idx = log0_idx();
                    if (idx as u32 as usize) < ECHO_COLLISION_LOG_CAPACITY {
                        let e = &mut dmg_log0[idx as usize];
                        e.x0 = 1;
                        e.kind = kind as _;
                        e.gobj = gobj;
                        e.anon_c.hit0 = hit0;
                        e.anon_10.hit1 = hit1;
                        e.pos = (*hit0).hurt_coll_pos;
                        e.x20 = (*hit0).damage;
                        e.size_of_xC = len;
                        set_log0_idx(idx + 1);
                    } else {
                        panic!("damage log over {}!!", ECHO_COLLISION_LOG_CAPACITY);
                    }
                    inner_ret = true;
                }
                // TARGET_PC: Lua on_hit / Geno / script hit rules.
                {
                    let idx = ext::ScriptGame_EchoHitIndex(fp0, hit0);
                    ext::ScriptGame_HitRuleWon(hit0, fp1);
                    if idx >= 0 {
                        ext::Geno_HitStunBonus(fp0, idx, fp1);
                    }
                    let flags = move |idx: i32| if idx >= 0 { ext::Geno_HitFlags(fp0, idx, fp1) } else { 0 };
                    if flags(idx) & 4 != 0 {
                        let f1 = &mut *fp1;
                        f1.dmg.x1838_percentTemp -= dmg;
                        if f1.dmg.x1838_percentTemp < 0.0 {
                            f1.dmg.x1838_percentTemp = 0.0;
                        }
                    }
                    if inner_ret && flags(idx) & 4 == 0 {
                        ext::ScriptGame_HitRulePercentQueue(hit0, fp1, dmg);
                    }
                    if inner_ret && flags(idx) & 4 == 0 {
                        ext::ScriptGame_CritHit(fp0, hit0, fp1, dmg, 0);
                    }
                    let (f0, f1) = (&*fp0, &*fp1);
                    ext::Script_GameEvent(
                        2,
                        f0.player_id as i32,
                        f1.player_id as i32,
                        (if (0..4).contains(&idx) { idx } else { 0xFF })
                            | if f0.is_sub_fighter() != 0 { 0x100 } else { 0 }
                            | if f1.is_sub_fighter() != 0 { 0x200 } else { 0 },
                        dmg.to_bits() as i32,
                    );
                    if ext::ScriptGame_EchoReportContext(fp0, fp1, hit0) == 0 {
                        ext::ScriptGame_ReportHitContext(fp0, fp1, (*hit0).element, 0);
                    }
                    if inner_ret {
                        ext::ScriptGame_EchoConnected(hit0, fp1);
                    }
                    if inner_ret {
                        ext::ScriptGame_IntrWinHit(fp0, 0);
                    }
                    if inner_ret {
                        ext::Geno_ComboContact(fp0, 0);
                    }
                    if inner_ret {
                        ext::ScriptGame_SkillHit(fp0, fp1, dmg);
                    }
                    ext::ScriptGame_HitRuleContext(hit0, fp1);
                }
                if ext::ScriptGame_EchoCredit(fp0, fp1, hit0, dmg) == 0 {
                    ftcoll_8007891c((*fp0).gobj, (*fp1).gobj, dmg);
                }
            }
            if !inner_ret {
                spawn(1052, &(*hit0).hurt_coll_pos);
            }
            return true;
        }
        false
    }
}

/// `ftColl_80077464`: an item's hitbox hit `fp`'s reflector.
pub fn ftcoll_80077464(item: It, hit: HC, fp: F) {
    melee_core::ct!("ftcoll_80077464");
    unsafe {
        ext::it_8026FAC4(item, hit, 7, fp as *mut c_void, false);
        let damage = get_env_dmg((*hit).damage);
        let (it, f) = (&mut *item, &mut *fp);
        let reflect_dir = |it: &Item, f: &Fighter| {
            if it.x40_vel.x != 0.0 {
                if it.x40_vel.x > 0.0 { -1.0 } else { 1.0 }
            } else if it.pos.x > f.cur_pos.x {
                -1.0
            } else {
                1.0
            }
        };
        if damage > f.ReflectAttr.x1A30_maxDamage as i32 {
            if (*hit).x41_b5() != 0 {
                if damage > it.xC4C {
                    it.xC4C = damage;
                }
            } else if damage > it.xC34_damageDealt {
                it.xC34_damageDealt = damage;
            }
            it.xC68 = if f.cur_pos.x > it.pos.x { -1.0 } else { 1.0 };
            f.ReflectAttr.x1A3C_damageOver = damage as _;
            f.ReflectAttr.x1A2C_reflectHitDirection = reflect_dir(it, f);
        } else {
            it.xC64_reflectGObj = f.gobj;
            it.xC74 = f.x2070.x2070_int;
            it.xC78 = f.x2074.x2074_vec;
            it.xC80 = f.x2074.x207C;
            it.xC88 = f.x2074.anon_10.x2084 as i32;
            it.xC8C = f.x2074.x2088;
            it.xC68 = if f.cur_pos.x > it.pos.x { -1.0 } else { 1.0 };
            it.xC6C = f.ReflectAttr.x1A34_damageMul;
            it.xC70 = f.ReflectAttr.x1A38_speedMul;
            it.xDCC_flag.set_b2(f.x2218_b5() as _);
            if f.x2218_b4() != 0 {
                it.xDCC_flag.set_b1(1);
            }
            f.ReflectAttr.x1A2C_reflectHitDirection = reflect_dir(it, f);
        }
    }
}

/// `ftColl_80077688`: an item's hitbox hit `fp`'s shield.
pub fn ftcoll_80077688(item: It, hurt: HC, fp: F, pos: *mut Vec3, mut val: f32) {
    melee_core::ct!("ftcoll_80077688");
    unsafe {
        {
            let f = &*fp;
            ext::Script_GameEvent(16, f.player_id as i32, f.motion_id as i32, 7, f.is_sub_fighter() as i32);
            if f.x221C_b2() != 0 {
                ext::ScriptGame_SkillPerfectShield(fp, core::ptr::null_mut(), 1, 0.0);
            }
        }
        let mode = if (*hurt).x41_b6() != 0 { 2 } else { 1 };
        ext::it_8026FAC4(item, hurt, mode, fp as *mut c_void, false);
        let dmg = get_env_dmg((*hurt).damage);
        let (it, f) = (&mut *item, &mut *fp);
        if dmg > it.xC34_damageDealt {
            it.xC50 = dmg;
            if f.anon_221b.x221B_b2() == 0 {
                it.xDCE_flag.anon_0.set_b5((*hurt).x42_b3() as _);
            } else {
                it.xDCE_flag.anon_0.set_b5(0);
            }
            it.xCF4_fighterGObjUnk = f.gobj;
            let vel_x = it.x40_vel.x;
            let mag = if vel_x < 0.0 { -vel_x } else { vel_x };
            it.xCB8_outDamageDirection = if mag < ext::icd().xD4 {
                if it.pos.x > f.cur_pos.x { -1.0 } else { 1.0 }
            } else if vel_x < 0.0 {
                -1.0
            } else {
                1.0
            };
            if it.xDCE_flag.anon_0.b5() != 0 {
                if f.anon_221b.x221B_b1() != 0 {
                    val = cd().x2D0 * 0.01745329252_f32;
                    it.xC54 = 0.0;
                    let c = melee_runtime::gekko::cosf(val);
                    val = melee_runtime::gekko::sinf(val);
                    it.xC58.x = if (*pos).x >= 0.0 { c } else { -c };
                    it.xC58.y = val;
                    it.xC58.z = 0.0;
                    it.xDCE_flag.anon_0.set_b4(1);
                } else {
                    it.xC54 = val;
                    it.xC58 = *pos;
                }
            }
            if f.shield_unk1 != 0.0 {
                it.xCC0 = f.shield_unk1;
            }
        }
        let total = dmg.wrapping_add((*hurt).x34);
        f.x19A0_shieldDamageTaken = f.x19A0_shieldDamageTaken.wrapping_add(if total < 0 { 0 } else { total } as _);
        if ext::ftLib_80086960(it.owner) {
            let o = &*fpg(it.owner);
            f.x19BC_shieldDamageTaken3 = o.player_id as _;
            f.set_x221F_b6(o.is_sub_fighter() as _);
        } else {
            f.x19BC_shieldDamageTaken3 = 6;
        }
        if dmg > f.x19A4 as i32 {
            f.x19A4 = dmg as _;
            f.specialn_facing_dir = if f.cur_pos.x > it.pos.x { -1.0 } else { 1.0 };
            f.x19B0 = (*hurt).element as _;
            f.x19A8 = it.entity;
        }
        if f.shield_unk1 != 0.0 && (*hurt).x42_b4() != 0 {
            f.x1964 = f.shield_unk1;
        }
        spawn(0x41C, &(*hurt).hurt_coll_pos);
    }
}

/// `inlineItemA0` (static inline).
unsafe fn inline_item_a0(item: It, fp: F, hit: HC, ef_pos: *const Vec3, dmg: f32) {
    let int_dmg = get_env_dmg(dmg);
    let mut j: i8 = 0;
    let mut i = 0u32;
    while i < cap_count(fp) {
        let cur = cap(fp, i);
        if (*cur).state != HC_DISABLED && same_group(cur, hit) && ext::lbColl_80008688(cur, 3, item as *mut c_void) {
            ftColl_804D6560[j as u8 as usize] = 0;
        }
        i += 1;
        j = j.wrapping_add(1);
    }
    let f = &mut *fp;
    if !is_echo(hit) && int_dmg > f.dmg.int_value as i32 {
        f.dmg.int_value = int_dmg as _;
        if (*hit).x40_b1() == 1 && ga(f.ground_or_air) == GROUND {
            let d = dir_lt(f.cur_pos.x, (*item).pos.x, 1.0, -1.0);
            hitlag_dir(f, int_dmg, d);
        }
    }
    spawn(0x41C, ef_pos);
}

/// `inlineItemA1` (static inline).
unsafe fn inline_item_a1(item: It, hit: HC, fp: F, ef_pos: *const Vec3, dmg: f32) {
    let int_dmg = get_env_dmg(dmg);
    let mode = if (*hit).x41_b5() != 0 { 4 } else { 3 };
    ext::it_8026FAC4(item, hit, mode, fp as *mut c_void, false);
    let it = &mut *item;
    if int_dmg > it.xC48 {
        it.xCF4_fighterGObjUnk = (*fp).gobj;
        it.xC48 = int_dmg;
        let vel_x = it.x40_vel.x;
        let mag = if vel_x < 0.0 { -vel_x } else { vel_x };
        it.xCB8_outDamageDirection = if mag < ext::icd().xD4 {
            if it.pos.x > (*fp).cur_pos.x { -1.0 } else { 1.0 }
        } else if vel_x < 0.0 {
            -1.0
        } else {
            1.0
        };
    }
    spawn(0x41C, ef_pos);
}

/// `ftColl_80077970`: item hitbox vs fighter hitbox clank.
pub fn ftcoll_80077970(item: It, hit1: HC, fp: F, hit2: HC) {
    melee_core::ct!("ftcoll_80077970");
    unsafe {
        let (a, b) = ((*hit1).hurt_coll_pos, (*hit2).hurt_coll_pos);
        let mid = Vec3 { x: (a.x + b.x) * 0.5, y: (a.y + b.y) * 0.5, z: (a.z + b.z) * 0.5 };
        script_clank_observe(fp, hit2, core::ptr::null_mut(), item, hit1);
        let x3cc = cd().x3CC as i32;
        let dmg = (*hit2).damage;
        if (dmg as i32).wrapping_sub(x3cc) < (*hit1).damage as i32 {
            inline_item_a0(item, fp, hit2, &mid, dmg);
        }
        let dmg = (*hit1).damage;
        if (dmg as i32).wrapping_sub(x3cc) < (*hit2).damage as i32 {
            inline_item_a1(item, hit1, fp, &mid, dmg);
        }
    }
}

/// `ftColl_80077C60`: an item's hitbox connected with `fp`'s hurtbox `hit2`.
pub fn ftcoll_80077c60(item: It, hit: HC, fp: F, hit2: HC) -> bool {
    melee_core::ct!("ftcoll_80077c60");
    unsafe {
        let it = &mut *item;
        if (*it.xCC_item_attr).x0_78() == 4 {
            let mode = if (*hit).x41_b5() != 0 { 5 } else { 0 };
            ext::it_8026FAC4(item, hit, mode, fp as *mut c_void, false);
            let f = &mut *fp;
            if f.x2224_b2() == 0 {
                let k = it.kind as i32;
                if k == It_Kind_Star as i32 || k == It_Kind_Kinoko as i32 || k == It_Kind_DKinoko as i32 {
                    ext::Geno_ItemFamilyEvent(it.entity, 0, f.player_id as i32);
                }
                if k == It_Kind_Star as i32 {
                    (*hit).state = HC_DISABLED as _;
                    it.xC34_damageDealt = 1;
                    ftcoll_8007b7fc(fp, ext::it_80272818(item) as usize as i32);
                    sfx(fp, 0xF9);
                    ext::ftCommon_8007EBAC(fp, 0x11, 0);
                } else if k == It_Kind_Kinoko as i32 {
                    (*hit).state = HC_DISABLED as _;
                    f.x200C = f.x200C.wrapping_add(1);
                    it.xC34_damageDealt = 1;
                } else if k == It_Kind_DKinoko as i32 {
                    (*hit).state = HC_DISABLED as _;
                    f.x2010 = f.x2010.wrapping_add(1);
                    it.xC34_damageDealt = 1;
                }
                ext::pl_8003E17C((*fp).player_id as i32, (*fp).is_sub_fighter() as i32, it.entity);
            }
            return false;
        }
        let raw_dmg = ext::it_8026B1D4(it.entity, hit);
        let mut f3 = raw_dmg;
        {
            let f = &*fp;
            if !f.victim_gobj.is_null() && f.anon_221b.x221B_b5() == 0 && !f.victim_gobj.is_null() {
                f3 *= cd().x128;
            }
            if f.motion_id == ftCo_MS_DamageIce as _ {
                f3 *= cd().x714;
            }
        }
        let mut scaled = f3 * (*fp).dmg.x182c_behavior;
        scaled = ext::ScriptGame_HitRuleContact(hit, fp, ext::ScriptGame_ItemModDamage(item, scaled), 0);

        if inline_b1(hit) {
            if log0_idx() == 0 && (*fp).dmg.x189C_unk_num_frames == 0.0 && check_tip_log(fp as *mut c_void, hit) == HitCapsule_Disabled as u32 {
                let mode = if (*hit).x41_b5() != 0 { 5 } else { 0 };
                ext::it_8026FC00(item, hit, mode, fp);
                let f = &mut *fp;
                if f.x1988 == 0 && f.x198C == 0 && f.x221D_b6() == 0 && (*hit2).state == HC_DISABLED {
                    let int_scaled = scaled as i32;
                    let int_raw = raw_dmg as i32;
                    let temp_dmg = 0.5 * int_scaled as f32;
                    let mut f4 = temp_dmg;
                    if temp_dmg as i32 == 0 && int_scaled != 0 {
                        f4 = 1.0;
                    }
                    let half_raw = int_raw / 2;
                    let mut half_raw_f = half_raw as f32;
                    if half_raw_f == 0.0 && int_raw != 0 {
                        half_raw_f = 1.0;
                    }
                    if f4 as i32 > f.dmg.x1840 as i32 {
                        f.dmg.x1840 = (f4 as i32) as _;
                    }
                    let idx = log1_idx();
                    if (idx as u32 as usize) < ECHO_COLLISION_LOG_CAPACITY {
                        let e = &mut dmg_log1[idx as usize];
                        e.x0 = 2;
                        e.kind = it.kind as _;
                        e.gobj = it.entity;
                        e.anon_c.hit0 = hit;
                        e.anon_10.hit1 = hit2;
                        e.pos = (*hit).hurt_coll_pos;
                        e.x20 = f4;
                        e.size_of_xC = f2usize(half_raw_f);
                        set_log1_idx(idx + 1);
                    } else {
                        panic!("tip log over {}!!", ECHO_COLLISION_LOG_CAPACITY);
                    }
                }
                ftcoll_80078488(fp);
                return true;
            }
        } else {
            set_log1_idx(0);
            let mut inner_ret = false;
            let mut should_log = false;
            let mode = if (*hit).x41_b5() != 0 { 5 } else { 0 };
            ext::it_8026FAC4(item, hit, mode, fp as *mut c_void, false);
            if scaled > it.xC34_damageDealt as f32 {
                it.xC34_damageDealt = scaled as i32;
                it.xCF4_fighterGObjUnk = (*fp).gobj;
                let vel_x = it.x40_vel.x;
                let mag = if vel_x < 0.0 { -vel_x } else { vel_x };
                it.xCB8_outDamageDirection = if mag < ext::icd().xD4 {
                    if it.pos.x > (*fp).cur_pos.x { -1.0 } else { 1.0 }
                } else if vel_x < 0.0 {
                    -1.0
                } else {
                    1.0
                };
            }
            let f = &mut *fp;
            if f.x1988 == 0 && f.x198C == 0 && f.x221D_b6() == 0 && (*hit2).state == HC_DISABLED {
                let count = get_env_dmg(scaled);
                if f.x221C_b4() != 0 {
                    f.dmg.x1834 -= scaled;
                    if f.dmg.x1834 < 0.0 {
                        scaled = -f.dmg.x1834;
                        f.set_x221C_b4(0);
                    }
                }
                if f.x221C_b4() == 0 {
                    f.dmg.x1838_percentTemp += scaled;
                    if count > f.dmg.x183C_applied as i32 {
                        f.dmg.x183C_applied = count as _;
                    }
                    should_log = true;
                }
                if should_log {
                    let idx = log0_idx();
                    if (idx as u32 as usize) < ECHO_COLLISION_LOG_CAPACITY {
                        let e = &mut dmg_log0[idx as usize];
                        e.x0 = 2;
                        e.kind = it.kind as _;
                        e.gobj = it.entity;
                        e.anon_c.hit0 = hit;
                        e.anon_10.hit1 = hit2;
                        e.pos = (*hit).hurt_coll_pos;
                        e.x20 = (*hit).damage;
                        e.size_of_xC = f2usize(raw_dmg);
                        set_log0_idx(idx + 1);
                    } else {
                        panic!("damage log over {}!!", ECHO_COLLISION_LOG_CAPACITY);
                    }
                    inner_ret = true;
                }
                // TARGET_PC: Lua on_hit for item hitbox -> fighter.
                {
                    let mut owner = -1;
                    if !it.owner.is_null() && ext::ftLib_80086960(it.owner) {
                        owner = (*fpg(it.owner)).player_id as i32;
                    }
                    ext::ScriptGame_HitRuleWon(hit, fp);
                    ext::Geno_ArticleStunBonus(item, hit, fp);
                    if should_log {
                        ext::ScriptGame_HitRulePercentQueue(hit, fp, scaled);
                    }
                    if should_log && owner >= 0 {
                        ext::ScriptGame_CritHit(fpg(it.owner), hit, fp, scaled, 1);
                    }
                    let f = &*fp;
                    let enemy = if f.is_sub_fighter() != 0 { 0 } else { ext::ScriptGame_EnemyContact(it.entity, f.player_id as i32, (*hit).x4) };
                    let flags = 0xFF | 0x400 | if f.is_sub_fighter() != 0 { 0x200 } else { 0 };
                    if enemy != 0 {
                        ext::Script_EnemyGameEvent(enemy, owner, f.player_id as i32, flags, scaled.to_bits() as i32, 0);
                    } else {
                        ext::Script_GameEvent(2, owner, f.player_id as i32, flags, scaled.to_bits() as i32);
                    }
                    let ofp = if owner < 0 { core::ptr::null_mut() } else { fpg(it.owner) };
                    ext::ScriptGame_ReportHitContext(ofp, fp, (*hit).element, 1);
                    ext::ScriptGame_HitRuleContext(hit, fp);
                    if inner_ret && owner >= 0 && fpg(it.owner) != fp {
                        ext::ScriptGame_IntrWinHit(fpg(it.owner), 2);
                        ext::Geno_ComboContact(fpg(it.owner), 2);
                    }
                    if inner_ret {
                        ext::ScriptGame_SkillHit(ofp, fp, scaled);
                    }
                }
                ftcoll_80078998(it.entity, (*fp).gobj, scaled);
            }
            if !inner_ret {
                spawn(0x41C, &(*hit).hurt_coll_pos);
            }
            return true;
        }
        false
    }
}

/// C `(size_t) f` on the shipping build: size_t is 32-bit, so fptoui to i32, which gwtool
/// saturates (Rust `as u32`).
#[inline(always)]
fn f2usize(f: f32) -> usize {
    f as u32 as usize
}

#[inline(always)]
fn unk_val(fp: &Fighter, i: i32) -> i32 {
    i + fp.player_id as i32 * 2 + fp.is_sub_fighter() as i32
}

/// The hurt sfx tail shared by 80078384 / 80078C70 / 8007925C: returns true if it played
/// the "no damage" sound.
unsafe fn hurt_sfx(fp: F, hurt_state: u32, hit: HC) -> bool {
    let f = &mut *fp;
    if f.x1988 != 0 || f.x198C != 0 || f.x221D_b6() != 0 || hurt_state != HurtCapsule_Enabled as u32 {
        sfx(fp, ftColl_803C0C40[(*hit).sfx_severity as usize]);
        return true;
    }
    false
}

unsafe fn hit_sfx_slot(fp: F, hit: HC) {
    let f = &mut *fp;
    if (*hit).sfx_kind as u32 == 0xD && (*hit).sfx_severity as u32 == 2 {
        f.x215C = ext::lbColl_80005BB0(hit, unk_val(f, 0x72)) as _;
    } else {
        f.x2160 = ext::lbColl_80005BB0(hit, unk_val(f, 0x7E)) as _;
    }
}

/// `ftColl_80078384`.
pub fn ftcoll_80078384(fp: F, arg1: *mut FighterHurtCapsule, arg2: HC) {
    melee_core::ct!("ftcoll_80078384");
    unsafe {
        if !hurt_sfx(fp, (*arg1).capsule.state as u32, arg2) {
            hit_sfx_slot(fp, arg2);
        }
    }
}

/// `ftColl_80078488`: phantom hit sound.
pub fn ftcoll_80078488(fp: F) {
    melee_core::ct!("ftcoll_80078488");
    sfx(fp, 85);
}

/// `ftColl_800784B4`: clank sound.
pub fn ftcoll_800784b4(fp: F, a: HC, b: HC) {
    melee_core::ct!("ftcoll_800784b4");
    unsafe {
        if (*a).element == HitElement_Slash as u32 && (*b).element == HitElement_Slash as u32 {
            sfx(fp, ftColl_803C0C4C[crate::hooks::hsd_randi(3) as usize]);
        } else {
            sfx(fp, 0x6A);
        }
    }
}

/// `ftColl_80078538`: hit spark.
pub fn ftcoll_80078538(g: G, pos: *const Vec3, dmg: u32, ignored: f32, scale: f32) {
    melee_core::ct!("ftcoll_80078538");
    unsafe {
        let fp = fpg(g);
        if scale < cd().x3F0 {
            ext::efSync_Spawn(Ef_Id_Unk1000 as i32, core::ptr::null_mut(), pos, &ignored);
        } else {
            ext::efSync_Spawn(0x3F3, core::ptr::null_mut(), pos, core::ptr::null());
        }
        if dmg >= 1 {
            match (*fp).co_attrs.hit_spark_variant {
                0 => {
                    if crate::hooks::hsd_randi(cd().x3F4 as i32) == 0 {
                        ext::efSync_Spawn(0x3EF, core::ptr::null_mut(), pos, &(*fp).facing_dir);
                    }
                }
                1 => {
                    crate::hooks::hsd_randi(cd().x3F8 as i32);
                }
                _ => {}
            }
        }
    }
}

/// `ftColl_8007861C`: record who damaged `gobj` and with what.
#[allow(clippy::too_many_arguments)]
pub fn ftcoll_8007861c(arg0: G, gobj: G, arg2: i32, arg3: i32, arg4: i32, arg5: *mut c_void, arg6: u16, arg7: *mut c_void, arg8: i32) {
    melee_core::ct!("ftcoll_8007861c");
    unsafe {
        let attacker: F = if arg0.is_null() { core::ptr::null_mut() } else { fpg(arg0) };
        let victim = &mut *fpg(gobj);
        let mut grounded = 0;
        let prev = victim.dmg.x18c4_source_ply;
        if !attacker.is_null() {
            victim.dmg.x18C0 = if attacker == victim as *mut Fighter { 0 } else { (*attacker).x8_spawnNum as _ };
            victim.dmg.x18c4_source_ply = (*attacker).player_id as _;
            victim.dmg.x18C8 = -1;
            victim.set_x221F_b5((*attacker).is_sub_fighter() as _);
        } else {
            victim.dmg.x18C0 = 0;
            victim.dmg.x18C8 = -1;
            if arg8 == 0 {
                victim.dmg.x18c4_source_ply = 6;
            }
        }
        victim.dmg.x18CC = arg2 as _;
        victim.dmg.x18D0 = arg3 as _;
        if !arg7.is_null() && *(arg7 as *const i32).add(2) == 0 {
            grounded = 1;
        }
        ext::pl_80038144(arg0, gobj, arg4, arg5, arg6, grounded, prev as i32);
    }
}

/// `ftColl_80078710`.
pub fn ftcoll_80078710(arg0: G, arg1: G, arg2: *mut c_void) {
    melee_core::ct!("ftcoll_80078710");
    unsafe {
        let fp = &mut *fpg(arg0);
        let p = &mut fp.x2074 as *mut Struct2074 as *mut c_void;
        ftcoll_8007861c(arg0, arg1, 1, fp.kind as i32, fp.x2070.x2070_int, p, fp.x2074.x2088, arg2, 0);
    }
}

/// `ftColl_80078754`.
pub fn ftcoll_80078754(arg0: G, arg1: G, arg2: bool) {
    melee_core::ct!("ftcoll_80078754");
    unsafe {
        let fp0 = &mut *fpg(arg0);
        let p = &mut fp0.x2074 as *mut Struct2074 as *mut c_void;
        ftcoll_8007861c(arg0, arg1, 1, fp0.kind as i32, fp0.x2070.x2070_int, p, fp0.x2074.x2088, arg2 as usize as *mut c_void, 0);
        let fp1 = &mut *fpg(arg1);
        fp1.dmg.x18c4_source_ply = 6;
        fp1.dmg.x18C8 = -1;
    }
}

unsafe fn item_damage_record(ip: It, victim: G, arg: *mut c_void) {
    let owner = (*ip).owner;
    let p = &mut (*ip).xD94 as *mut Vec2 as *mut c_void;
    let (kind, x2070, short) = ((*ip).kind as i32, (*ip).xD90.x2070_int, (*ip).xDA8_short);
    if ext::ftLib_80086960(owner) {
        ftcoll_8007861c(owner, victim, 2, kind, x2070, p, short, arg, 0);
    } else if ext::pl_8003D60C(kind) {
        ftcoll_8007861c(core::ptr::null_mut(), victim, 2, kind, x2070, p, short, arg, 1);
    } else {
        ftcoll_8007861c(core::ptr::null_mut(), victim, 2, kind, x2070, p, short, arg, 0);
    }
}

/// `ftColl_800787B4`.
pub fn ftcoll_800787b4(arg0: G, arg1: G, arg2: i32) {
    melee_core::ct!("ftcoll_800787b4");
    unsafe {
        let ip = fpg(arg0) as It;
        let fp = &*fpg(arg1);
        if (*ip).kind as i32 == It_Kind_BombHei as i32 {
            ext::pl_80041B08(fp.player_id as i32, fp.is_sub_fighter() as usize as *mut c_void, (*ip).x1C as u16);
        }
        item_damage_record(ip, arg1, arg2 as isize as *mut c_void);
    }
}

/// `ftColl_800788D4`.
pub fn ftcoll_800788d4(g: G) {
    melee_core::ct!("ftcoll_800788d4");
    ftcoll_8007861c(core::ptr::null_mut(), g, 0, -10, 0, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0);
}

/// `ftColl_8007891C`: stale moves + combo + bonus stats for a fighter hit.
pub fn ftcoll_8007891c(arg0: G, arg1: G, arg2: f32) {
    melee_core::ct!("ftcoll_8007891c");
    ext::plStale_UpdateStaleMovesFromFighter(arg0, arg1);
    ftcoll_80076444(arg0, arg1);
    unsafe {
        let (f0, f1) = (&*fpg(arg0), &*fpg(arg1));
        ext::pl_8003EB30(arg2, f0.player_id as i32, f0.is_sub_fighter() as i32, f1.player_id as i32, f1.is_sub_fighter() as i32, f0.x2070.anon_0.x2073 as i32);
    }
}

/// `ftColl_80078998`: the same for an item hit.
pub fn ftcoll_80078998(arg0: G, arg1: G, arg2: f32) {
    melee_core::ct!("ftcoll_80078998");
    ext::plStale_UpdateStaleMovesFromItem(arg0, arg1);
    ftcoll_8007646c(arg0, arg1);
    unsafe {
        let ip = &*(fpg(arg0) as It);
        if ext::ftLib_80086960(ip.owner) {
            let (o, v) = (&*fpg(ip.owner), &*fpg(arg1));
            ext::pl_8003EB30(arg2, o.player_id as i32, o.is_sub_fighter() as i32, v.player_id as i32, v.is_sub_fighter() as i32, ip.xD90.anon_0.x2073 as i32);
        }
    }
}

/// `ftColl_80078A2C`: find grabbable fighters.
pub fn ftcoll_80078a2c(this_gobj: G) {
    melee_core::ct!("ftcoll_80078a2c");
    unsafe {
        let this_fp = fpg(this_gobj);
        (*this_fp).victim_gobj = core::ptr::null_mut();
        (*this_fp).unk_grab_val = f32::MAX;
        let mut vg = crate::hooks::plink_head(PLINK_FIGHTER);
        'next: while !vg.is_null() {
            let cur = vg;
            vg = (*vg).next;
            if crate::hooks::ftlib_80086fd4(this_gobj, cur) {
                continue;
            }
            let v = fpg(cur);
            let vf = &*v;
            let tf = &*this_fp;
            if !(vf.x2219_b1() == 0
                && vf.x222A_b0() == 0
                && (!crate::hooks::gm_8016b168() || ext::gm_8016B0D4() || vf.team != tf.team)
                && vf.x1988 == 0
                && vf.x198C == 0
                && vf.x221D_b6() == 0
                && (vf.x1A6A & tf.x1A68) == 0
                && vf.x2224_b2() == 0)
            {
                continue;
            }
            for i in 0..4u32 {
                let h = cap(this_fp, i);
                if (*h).state != HC_DISABLED
                    && (*h).element == HitElement_Catch as u32
                    && (((*h).x40_b2() != 0 && ga((*v).ground_or_air) == AIR) || ((*h).x40_b3() != 0 && ga((*v).ground_or_air) == GROUND))
                    && !ext::lbColl_8000ACFC(v as *mut c_void, h)
                {
                    let mut j = 0u32;
                    while j < (*v).hurt_capsules_len as u32 {
                        let hc = &mut (*v).hurt_capsules[j as usize];
                        if hc.is_grabbable != 0
                            && ext::lbColl_80007ECC(h, &mut hc.capsule, mtx(v), (*this_fp).x34_scale.y, (*v).x34_scale.y, (*v).cur_pos.z)
                        {
                            if !ext::ft_80084CE4(this_fp, v) {
                                ftcoll_80076808(this_fp, h, 0, v as *mut c_void, false);
                                // ftGrabDist
                                let tf = &mut *this_fp;
                                let mut d = (*v).cur_pos.x - tf.cur_pos.x;
                                if d < 0.0 {
                                    d = -d;
                                }
                                if d < tf.unk_grab_val {
                                    let gg = (*v).gobj;
                                    tf.x1A5C = gg;
                                    tf.victim_gobj = gg;
                                    tf.anon_221b.set_x221B_b5(1);
                                    tf.unk_grab_val = d;
                                }
                            }
                            continue 'next;
                        }
                        j += 1;
                    }
                }
            }
        }
    }
}

/// `ftColl_80078C70`: this fighter's hurt / shield / hitboxes against every other fighter's
/// hitboxes (clank, shield hit, hurtbox hit).
pub fn ftcoll_80078c70(this_gobj: G) {
    melee_core::ct!("ftcoll_80078c70");
    unsafe {
        let this_fp = fpg(this_gobj);
        ext::ScriptGame_EchoPrepare();
        if ext::gm_8016B1C4() {
            return;
        }
        let mut is_same_gobj = false;
        let mut count: i32 = 0;
        let mut vg = crate::hooks::plink_head(PLINK_FIGHTER);
        while !vg.is_null() {
            let victim_gobj = vg;
            vg = (*vg).next;
            if this_gobj == victim_gobj {
                is_same_gobj = true;
                continue;
            }
            let v = fpg(victim_gobj);
            let (vf, tf) = (&*v, &*this_fp);
            let thrown = vf.x1064_thrownHitbox.anon_134.owner;
            let ordinary_allowed = (thrown.is_null() || tf.player_id as i32 != vf.grabber_unk1 as i32)
                && (!thrown.is_null() || tf.player_id != vf.player_id)
                && (!crate::hooks::gm_8016b168()
                    || ext::gm_8016B0D4()
                    || vf.x2225_b4() != 0
                    || tf.team as i32 != if !thrown.is_null() { vf.x119C_teamUnk as i32 } else { vf.team as i32 })
                && thrown != this_gobj;
            if !(ordinary_allowed || (cap_count(v) > 4 && echo_allowed(v, this_fp))) {
                continue;
            }
            if is_same_gobj && (*this_fp).anon_221b.x221B_b5() == 0 {
                count = 0;
                let mut i = 0u32;
                while i < cap_count(this_fp) {
                    let th = cap(this_fp, i);
                    let ok = (*th).state != HC_DISABLED
                        && (ordinary_allowed || (is_echo(th) && echo_allowed(this_fp, v)))
                        && (*th).x43_b2() == 0
                        && (*th).element != HitElement_Catch as u32
                        && (*th).element != HitElement_Inert as u32
                        && (*th).x40_b0() == 1
                        && (*th).x42_b5() == 1
                        && (((*th).x40_b2() != 0 && ga((*v).ground_or_air) == AIR) || ((*th).x40_b3() != 0 && ga((*v).ground_or_air) == GROUND))
                        && !ext::lbColl_8000ACFC(v as *mut c_void, th);
                    if ok {
                        ftColl_804D6560[i as usize] = 1;
                        count += 1;
                    } else {
                        ftColl_804D6560[i as usize] = 0;
                    }
                    i += 1;
                }
            }
            let mut j = 0u32;
            while j < cap_count(v) {
                let hit0 = cap(v, j);
                j += 1;
                let vf = &*v;
                let ok = (*hit0).state != HC_DISABLED
                    && (if is_echo(hit0) { echo_allowed(v, this_fp) } else { ordinary_allowed })
                    && (*hit0).element != HitElement_Catch as u32
                    && (*hit0).x42_b5() == 1
                    && (((*hit0).x40_b2() != 0 && ga((*this_fp).ground_or_air) == AIR) || ((*hit0).x40_b3() != 0 && ga((*this_fp).ground_or_air) == 0))
                    && ((*hit0).anon_134.hit_grabbed_victim_only() == 0
                        || vf.victim_gobj.is_null()
                        || vf.anon_221b.x221B_b5() == 0
                        || vf.victim_gobj == this_gobj)
                    && (ext::ScriptGame_EchoBlocked(hit0, this_fp) == 0 && !ext::lbColl_8000ACFC(this_fp as *mut c_void, hit0));
                if !ok {
                    continue;
                }
                let var_r22 = ((*hit0).x43_b2() != 0) as i32;
                let tf = &*this_fp;
                let clank_cond = is_same_gobj
                    && var_r22 == 0
                    && this_gobj != vf.victim_gobj
                    && ga(tf.ground_or_air) == GROUND
                    && ga(vf.ground_or_air) == GROUND
                    && tf.anon_221b.x221B_b5() == 0
                    && (*hit0).element != HitElement_Inert as u32
                    && (*hit0).x40_b0() == 1
                    && count != 0;
                let mut block_66 = true;
                if clank_cond {
                    let mut flag0 = false;
                    let mut m = 0u32;
                    while m < cap_count(this_fp) {
                        if ftColl_804D6560[m as usize] as u8 != 0 {
                            let hit1 = cap(this_fp, m);
                            if ext::lbColl_80007AFC(hit0, hit1, (*v).x34_scale.y, (*this_fp).x34_scale.y)
                                && ftcoll_8007699c(v, hit0, this_fp, hit1)
                            {
                                flag0 = true;
                                break;
                            }
                        }
                        m += 1;
                    }
                    block_66 = !flag0;
                }
                if !block_66 {
                    continue;
                }
                let mut block_81 = true;
                if (*this_fp).anon_221b.x221B_b0() != 0 {
                    let tf = &*this_fp;
                    let mut var_r3 = true;
                    if tf.anon_221b.x221B_b3() != 0 {
                        if -1.0 == tf.facing_dir {
                            if tf.cur_pos.x < (*v).cur_pos.x {
                                var_r3 = false;
                            }
                        } else if tf.cur_pos.x > (*v).cur_pos.x {
                            var_r3 = false;
                        }
                    }
                    if tf.anon_221b.x221B_b4() != 0 && (*hit0).x42_b4() == 0 {
                        var_r3 = false;
                    }
                    if var_r3
                        && ext::lbColl_80007BCC(
                            hit0,
                            &mut (*this_fp).shield_hit,
                            mtx(this_fp),
                            var_r22,
                            (*v).x34_scale.y,
                            (*this_fp).x34_scale.y,
                            (*this_fp).cur_pos.z,
                        )
                    {
                        if (*hit0).element != HitElement_Inert as u32 {
                            ftcoll_80076cbc(v, hit0, this_fp);
                            block_81 = false;
                        } else {
                            (*v).set_x221C_b5(1);
                            (*v).unk_gobj = this_gobj;
                        }
                    }
                }
                if !block_81 {
                    continue;
                }
                if (*this_fp).x1988 != 2 && (*this_fp).x198C != 2 {
                    let mut n = 0u32;
                    while n < (*this_fp).hurt_capsules_len as u32 {
                        let hc: *mut FighterHurtCapsule = &mut (*this_fp).hurt_capsules[n as usize];
                        if ext::lbColl_8000805C(
                            hit0,
                            &mut (*hc).capsule,
                            mtx(this_fp),
                            var_r22,
                            (*v).x34_scale.y,
                            (*this_fp).x34_scale.y,
                            (*this_fp).cur_pos.z,
                        ) {
                            if (*hit0).element != HitElement_Inert as u32 {
                                if ftcoll_80076ed8(v, hit0, this_fp, hc as HC) && !hurt_sfx(this_fp, (*hc).capsule.state as u32, hit0) {
                                    hit_sfx_slot(this_fp, hit0);
                                }
                            } else {
                                (*v).unk_gobj = this_gobj;
                            }
                            break;
                        }
                        n += 1;
                    }
                }
            }
        }
    }
}

/// `ftColl_8007925C`: this fighter against every item's hitboxes.
pub fn ftcoll_8007925c(gobj: G) {
    melee_core::ct!("ftcoll_8007925c");
    unsafe {
        let fp = fpg(gobj);
        let mut coll_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut coll_dist: f32 = 0.0;
        let mut hit_count: i32 = 0;
        let mut entity = crate::hooks::plink_head(PLINK_ITEM);
        while !entity.is_null() {
            let ent = entity;
            entity = (*entity).next;
            let item = fpg(ent) as It;
            let it = &mut *item;
            if it.kind as i32 == It_Kind_Unk4 as i32 {
                continue;
            }
            if crate::hooks::ftlib_80086fd4(gobj, it.owner) && it.xDCD_flag.anon_0.b5() == 0 {
                continue;
            }
            if crate::hooks::gm_8016b168() && !ext::gm_8016B0D4() && it.xDCD_flag.anon_0.b6() == 0 && it.x20_team_id as i32 == (*fp).team as i32 {
                continue;
            }
            if (*fp).anon_221b.x221B_b5() == 0 {
                hit_count = 0;
                let f = &*fp;
                let thrown = f.x1064_thrownHitbox.anon_134.owner;
                if thrown.is_null()
                    || ((!crate::hooks::ftlib_80086fd4(thrown, it.owner) || it.xDCD_flag.anon_0.b5() != 0)
                        && (!crate::hooks::gm_8016b168()
                            || ext::gm_8016B0D4()
                            || it.xDCD_flag.anon_0.b6() != 0
                            || it.x20_team_id as i32 != f.x119C_teamUnk as i32))
                {
                    let mut i = 0u32;
                    while i < cap_count(fp) {
                        let th = cap(fp, i);
                        let ok = (*th).state != HC_DISABLED
                            && (*th).x43_b2() == 0
                            && (*th).element != HitElement_Catch as u32
                            && (*th).x42_b5() == 1
                            && (((*th).x40_b2() != 0 && ga(it.ground_or_air) == AIR) || ((*th).x40_b3() != 0 && ga(it.ground_or_air) == GROUND))
                            && !ext::lbColl_8000ACFC(item as *mut c_void, th);
                        if ok {
                            ftColl_804D6560[i as usize] = 1;
                            hit_count += 1;
                        } else {
                            ftColl_804D6560[i as usize] = 0;
                        }
                        i += 1;
                    }
                }
            }
            'hitbox: for j in 0..4usize {
                let ih: *mut ItemHitbox = &mut (*item).x5D4_hitboxes[j];
                let hurt: HC = &mut (*ih).hit;
                let f = &mut *fp;
                if (*hurt).state == HC_DISABLED || (*hurt).x42_b5() != 1 {
                    continue;
                }
                if !((*hurt).x40_b2() != 0 && ga(f.ground_or_air) == AIR) && !((*hurt).x40_b3() != 0 && ga(f.ground_or_air) == GROUND) {
                    continue;
                }
                if (*hurt).x42_b2() != 0 && f.facing_dir == (*item).facing_dir {
                    continue;
                }
                if ext::gm_8016B1C4() && (*ih).x138() == 0 {
                    continue;
                }
                if ext::lbColl_8000ACFC(fp as *mut c_void, hurt) {
                    continue;
                }
                let var_r22 = ((*hurt).x43_b2() != 0) as i32;
                let it = &mut *item;
                if (*hurt).element != HitElement_Inert as u32 && var_r22 == 0 && (*hurt).x42_b4() != 0 {
                    if f.reflecting() != 0
                        && (*hurt).x41_b7() != 0
                        && ext::lbColl_80007BCC(hurt, &mut (*fp).reflect_hit, mtx(fp), 0, it.scl, (*fp).x34_scale.y, (*fp).cur_pos.z)
                    {
                        ftcoll_80077464(item, hurt, fp);
                        continue;
                    }
                    let f = &mut *fp;
                    if f.x2218_b6() != 0
                        && ((*hurt).x42_b0() != 0 || (f.x2218_b7() != 0 && (*hurt).x41_b7() != 0))
                        && ext::lbColl_80007BCC(hurt, &mut (*fp).absorb_hit, mtx(fp), 0, it.scl, (*fp).x34_scale.y, (*fp).cur_pos.z)
                    {
                        ext::it_8026FAC4(item, hurt, 6, fp as *mut c_void, false);
                        let dmg_count = get_env_dmg((*hurt).damage);
                        let f = &mut *fp;
                        it.xC90_absorbGObj = f.gobj;
                        f.AbsorbAttr.x1A40_absorbHitDirection = if f.cur_pos.x > it.pos.x { -1.0 } else { 1.0 };
                        if (*hurt).x42_b0() == 0 {
                            continue;
                        }
                        f.AbsorbAttr.x1A44_damageTaken = f.AbsorbAttr.x1A44_damageTaken.wrapping_add(dmg_count as _);
                        f.AbsorbAttr.x1A48_hitsTaken = f.AbsorbAttr.x1A48_hitsTaken.wrapping_add(1);
                        continue;
                    }
                }
                if (*fp).anon_221b.x221B_b5() == 0 && var_r22 == 0 && hit_count != 0 {
                    let mut m = 0u32;
                    while m < cap_count(fp) {
                        if ftColl_804D6560[m as usize] as u8 == 0 {
                            m += 1;
                            continue;
                        }
                        let th = cap(fp, m);
                        m += 1;
                        let inert = HitElement_Inert as u32;
                        if (*hurt).element == inert || (*th).element == inert {
                            if (*hurt).element == (*th).element {
                                continue;
                            }
                            if ext::lbColl_80007AFC(hurt, th, (*item).scl, (*fp).x34_scale.y) {
                                (*item).xDCE_flag.anon_0.set_b6(1);
                                (*item).toucher = gobj;
                                continue 'hitbox;
                            }
                        } else {
                            if (*hurt).x40_b0() != 1 || (*th).x40_b0() != 1 {
                                continue;
                            }
                            if ext::lbColl_80007AFC(hurt, th, (*item).scl, (*fp).x34_scale.y) {
                                ftcoll_80077970(item, hurt, fp, th);
                                continue 'hitbox;
                            }
                        }
                    }
                }
                let f = &*fp;
                if (*hurt).element != HitElement_Inert as u32 && f.anon_221b.x221B_b0() != 0 && (*hurt).x42_b1() != 0 {
                    let mut var_r3 = true;
                    let ix = (*item).pos.x;
                    if f.anon_221b.x221B_b3() != 0 {
                        if -1.0 == f.facing_dir {
                            if f.cur_pos.x < ix {
                                var_r3 = false;
                            }
                        } else if f.cur_pos.x > ix {
                            var_r3 = false;
                        }
                    }
                    if f.anon_221b.x221B_b4() != 0 && (*hurt).x42_b4() == 0 {
                        var_r3 = false;
                    }
                    if var_r3
                        && ext::lbColl_80007BCC(hurt, &mut (*fp).shield_hit, mtx(fp), var_r22, (*item).scl, (*fp).x34_scale.y, (*fp).cur_pos.z)
                    {
                        if (*fp).anon_221b.x221B_b2() == 0 && ((*hurt).x42_b3() != 0 || (*fp).anon_221b.x221B_b1() != 0) {
                            ext::lbColl_80007DD8(hurt, &mut (*fp).shield_hit, mtx(fp), &mut coll_pos, &mut coll_dist, (*item).scl);
                        }
                        ftcoll_80077688(item, hurt, fp, &mut coll_pos, coll_dist);
                        continue;
                    }
                }
                if (*hurt).element == HitElement_Inert as u32 {
                    let mut m = 0u32;
                    while m < (*fp).hurt_capsules_len as u32 {
                        if ext::lbColl_80008248(
                            hurt,
                            &mut (*fp).hurt_capsules[m as usize].capsule,
                            mtx(fp),
                            (*item).scl,
                            (*fp).x34_scale.y,
                            (*fp).cur_pos.z,
                        ) {
                            (*item).xDCE_flag.anon_0.set_b6(1);
                            (*item).toucher = gobj;
                            break;
                        }
                        m += 1;
                    }
                } else {
                    if (*fp).x1988 == 2 || (*fp).x198C == 2 || (*hurt).x42_b5() != 1 {
                        continue;
                    }
                    let mut n = 0u32;
                    while n < (*fp).hurt_capsules_len as u32 {
                        let hc: *mut FighterHurtCapsule = &mut (*fp).hurt_capsules[n as usize];
                        n += 1;
                        if (*hurt).x42_b6() != 0 && (*hc).is_grabbable == 0 {
                            continue;
                        }
                        if ext::lbColl_8000805C(hurt, &mut (*hc).capsule, mtx(fp), var_r22, (*item).scl, (*fp).x34_scale.y, (*fp).cur_pos.z) {
                            if !ftcoll_80077c60(item, hurt, fp, hc as HC) {
                                break;
                            }
                            if hurt_sfx(fp, (*hc).capsule.state as u32, hurt) {
                                break;
                            }
                            hit_sfx_slot(fp, hurt);
                            break;
                        }
                    }
                }
            }
        }
    }
}

// ---- knockback ----------------------------------------------------------------------------

/// `ftColl_GetDamageCount` (static inline).
fn damage_count(fp: &Fighter, ftd: &ftCommonData) -> i32 {
    if fp.x2225_b7() != 0 {
        if fp.x2224_b2() != 0 {
            return ftd.x6D8[0];
        }
        return ftd.x6D4;
    }
    fp.dmg.x1830_percent as i32
}

/// The `KNOCKBACK` macro: `defense * (attack * (arg3 * (0.01*x24 * (x11C * (D * inner) + x120)
/// + x2C)))`, D = `xF8 - (w*xF8)/(one+w)`; the two `a*b+c` there are fused.
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn knockback(defense: f32, attack: f32, arg3: f32, one: f32, ftd: &ftCommonData, x24: u32, x2c: u32, w: f32, inner: f32) -> f32 {
    let decay = ftd.xF8;
    let d = decay - ((w * decay) / (one + w));
    let mid = fmadds(ftd.x11C, d * inner, ftd.x120);
    defense * (attack * (arg3 * fmadds(0.01 * x24 as f32, mid, x2c as f32)))
}

/// The inner scaling term for `hit->x28 != 0` (fixed knockback); fused.
#[inline(always)]
fn inner_fixed(ftd: &ftCommonData, x28: u32) -> f32 {
    let x118 = ftd.x118;
    fmadds(x118, ftd.x110, ftd.x114 * (x118 * x28 as f32))
}

/// The inner scaling term from percent; fused.
#[inline(always)]
fn inner_percent(ftd: &ftCommonData, count: i32, pt: f32, unk_count: u32) -> f32 {
    fmadds(ftd.x110, count as f32 + pt, ftd.x114 * (unk_count as f32 * (count as f32 + pt)))
}

/// `ftColl_80079AB0`.
pub fn ftcoll_80079ab0(fp: &Fighter, hit: &HitCapsule, unk_count: u32, arg3: f32, attack: f32, defense: f32, weight: f32) -> f32 {
    melee_core::ct!("ftcoll_80079ab0");
    let ftd = cd();
    let w = weight * ftd.xF4;
    let mut result = if hit.x28 != 0 {
        knockback(defense, attack, arg3, 1.0, ftd, hit.x24, hit.x2C, w, inner_fixed(ftd, hit.x28))
    } else {
        let count = damage_count(fp, ftd);
        knockback(defense, attack, arg3, 1.0, ftd, hit.x24, hit.x2C, w, inner_percent(ftd, count, fp.dmg.x1838_percentTemp, unk_count))
    };
    if result >= ftd.x108 {
        result = ftd.x108;
    }
    crate::hooks::script_value(fp, 7, result)
}

/// `ftColl_80079C70`.
pub fn ftcoll_80079c70(fp: F, attacker: F, hit: HC, unk_count: i32) -> f32 {
    melee_core::ct!("ftcoll_80079c70");
    unsafe {
        let f = &*fp;
        let weight = crate::hooks::script_value(f, 9, f.co_attrs.weight);
        let defense = ext::Player_GetDefenseRatio(f.player_id as i32);
        let attack = ext::Player_GetAttackRatio((*attacker).player_id as i32);
        let stage = ext::gm_8016B248();
        let ftd = cd();
        let h = &*hit;
        let w = weight * ftd.xF4;
        let mut result = if h.x28 != 0 {
            knockback(defense, attack, stage, 1.0, ftd, h.x24, h.x2C, w, inner_fixed(ftd, h.x28))
        } else {
            let count = damage_count(f, ftd);
            knockback(defense, attack, stage, 1.0, ftd, h.x24, h.x2C, w, inner_percent(ftd, count, f.dmg.x1838_percentTemp, unk_count as u32))
        };
        if result >= ftd.x108 {
            result = ftd.x108;
        }
        ext::ScriptGame_HitRuleWon(hit, fp);
        ext::ScriptGame_HitRuleContact(hit, fp, crate::hooks::script_value(&*fp, 7, result), 1)
    }
}

/// `ftColl_80079EA8`.
pub fn ftcoll_80079ea8(fp: &Fighter, hit: &HitCapsule, unk_count: u32) -> f32 {
    melee_core::ct!("ftcoll_80079ea8");
    let ftd = cd();
    let w = crate::hooks::script_value(fp, 9, fp.co_attrs.weight) * ftd.xF4;
    let mut result = if hit.x28 != 0 {
        knockback(1.0, 1.0, 1.0, 1.0, ftd, hit.x24, hit.x2C, w, inner_fixed(ftd, hit.x28))
    } else {
        let count = damage_count(fp, ftd);
        knockback(1.0, 1.0, 1.0, 1.0, ftd, hit.x24, hit.x2C, w, inner_percent(ftd, count, fp.dmg.x1838_percentTemp, unk_count))
    };
    if result >= ftd.x108 {
        result = ftd.x108;
    }
    crate::hooks::script_value(fp, 7, result)
}

/// `ftColl_AceHideHitFx` (TARGET_PC static; the one-time OSReport is not reproduced).
fn ace_hide_hit_fx(g: G) -> bool {
    let fp = unsafe { fpg(g) };
    if fp.is_null() || unsafe { (*fp).x21FC_flag.byte } & 2 == 0 || ext::Mex_Enabled("ace_bubble_view_hides_hit_fx") == 0 {
        return false;
    }
    true
}

/// `struct DmgResult` (local to ftColl_8007A06C): the layout written over `fp->dmg`.
#[repr(C)]
pub struct DmgResult {
    pub dir: f32,
    pub angle: i32,
    pub hurt_height: i32,
    pub kb: f32,
    pub pos: Vec3,
    pub element: u32,
    pub sfx_severity: i32,
    pub source: *mut HSD_GObj,
    pub damage: f32,
}

unsafe fn hit_fx(g: G, pos: *const Vec3, element: u32, severity: i32, x20: f32, kb: f32) {
    let u_dmg = x20 as u32;
    let sfx_id = hit_effect_ids[element as usize];
    let fp = fpg(g);
    match sfx_id {
        x if x == Ef_Id_Unk1000 as i32 => ftcoll_80078538(g, pos, severity as u32, u_dmg as f32, kb),
        x if x == Ef_Id_Unk1001 as i32
            || x == Ef_Id_Unk1002 as i32
            || x == Ef_Id_Unk1004 as i32
            || x == Ef_Id_Unk1046 as i32
            || x == Ef_Id_Unk1145 as i32
            || x == Ef_Id_Unk1255 as i32 =>
        {
            ext::efSync_Spawn(sfx_id, core::ptr::null_mut(), pos, core::ptr::null())
        }
        x if x == Ef_Id_Unk1005 as i32 => ext::efSync_Spawn(sfx_id, core::ptr::null_mut(), pos, &(*fp).facing_dir),
        _ => {}
    }
}

/// `ftColl_8007A06C`: pick the strongest entry of a damage log and write the result.
pub fn ftcoll_8007a06c(gobj: G, dmg_ptr: *mut c_void, log: *mut DmgLogEntry, idx: usize, arg4: i32) {
    melee_core::ct!("ftcoll_8007a06c");
    if idx == 0 {
        return;
    }
    unsafe {
        let fp = fpg(gobj);
        let mut best_kb: f32 = -1.0;
        let mut best_idx: usize = 0;
        let mut i = 0usize;
        while i < idx {
            let entry = &mut *log.add(i);
            let mut kb: f32 = 0.0;
            match entry.x0 as i32 {
                1 => {
                    let unk_count = entry.size_of_xC as i32;
                    let attacker = fpg(entry.gobj);
                    let hit = entry.anon_c.hit0;
                    // clang evaluates the call arguments right to left (checked against the C oracle).
                    let weight = crate::hooks::script_value(&*fp, 9, (*fp).co_attrs.weight);
                    let defense = ext::Player_GetDefenseRatio((*fp).player_id as i32);
                    let attack = ext::Player_GetAttackRatio((*attacker).player_id as i32);
                    let stage = ext::gm_8016B248();
                    kb = ftcoll_80079ab0(&*fp, &*hit, unk_count as u32, stage, attack, defense, weight);
                    if arg4 != 0 && !ace_hide_hit_fx(gobj) {
                        let h = entry.anon_c.hit0;
                        hit_fx(gobj, &entry.pos, (*h).element, (*h).sfx_severity, entry.x20, kb);
                    }
                }
                2 => {
                    let item = &*(fpg(entry.gobj) as It);
                    let owner = item.owner;
                    let attack = if ext::ftLib_80086960(owner) { ext::Player_GetAttackRatio((*fpg(owner)).player_id as i32) } else { 1.0 };
                    let weight = (*fp).co_attrs.weight;
                    let defense = ext::Player_GetDefenseRatio((*fp).player_id as i32);
                    let stage = ext::gm_8016B248();
                    let hit = &*entry.anon_c.hit0;
                    let ftd = cd();
                    let w = weight * ftd.xF4;
                    let unk_count = entry.size_of_xC as i32;
                    let mut result = if hit.x28 != 0 {
                        knockback(defense, attack, stage, 1.0, ftd, hit.x24, hit.x2C, w, inner_fixed(ftd, hit.x28))
                    } else {
                        let count = damage_count(&*fp, ftd);
                        knockback(defense, attack, stage, 1.0, ftd, hit.x24, hit.x2C, w, inner_percent(ftd, count, (*fp).dmg.x1838_percentTemp, unk_count as u32))
                    };
                    if result >= ftd.x108 {
                        result = ftd.x108;
                    }
                    kb = result;
                    if arg4 != 0 && !ace_hide_hit_fx(gobj) {
                        hit_fx(gobj, &entry.pos, hit.element, hit.sfx_severity, entry.x20, kb);
                    }
                }
                3 => {
                    let mut stack_hit: HitCapsule = core::mem::zeroed();
                    ext::lbColl_80008D30(&mut stack_hit, entry.anon_c.unk_anim0 as *mut c_void);
                    let weight = (*fp).co_attrs.weight;
                    let defense = ext::Player_GetDefenseRatio((*fp).player_id as i32);
                    let stage = ext::gm_8016B248();
                    let ftd = cd();
                    let w = weight * ftd.xF4;
                    let unk_count = stack_hit.unk_count as i32;
                    let mut result = if stack_hit.x28 != 0 {
                        knockback(defense, 1.0, stage, 1.0, ftd, stack_hit.x24, stack_hit.x2C, w, inner_fixed(ftd, stack_hit.x28))
                    } else {
                        let count = damage_count(&*fp, ftd);
                        knockback(defense, 1.0, stage, 1.0, ftd, stack_hit.x24, stack_hit.x2C, w, inner_percent(ftd, count, (*fp).dmg.x1838_percentTemp, unk_count as u32))
                    };
                    if result >= ftd.x108 {
                        result = ftd.x108;
                    }
                    kb = result;
                }
                _ => {}
            }
            if entry.x0 as i32 == 1 || entry.x0 as i32 == 2 {
                kb = ext::ScriptGame_HitRuleContact(entry.anon_c.hit0, fp, kb, 1);
            }
            if kb > best_kb {
                let f = &mut *fp;
                let v = entry.x0 as i32 == 1 && !f.victim_gobj.is_null() && f.anon_221b.x221B_b5() == 0 && f.victim_gobj == entry.gobj;
                f.set_x221C_b0(v as _);
                best_kb = kb;
                best_idx = i;
            }
            i += 1;
        }
        // arg4 != 0: the result overlays `fp->dmg.facing_dir_1 ..` (x1844..x186C, all 4-byte fields up to the source pointer, so the
        // overlay is exact). arg4 == 0: the C overlays `fp->dmg.x1870 ..`, but the native `x1870` is an 8-byte pointer, so the following
        // fields are not where a C struct overlay would put them: build the result locally and store it field by field.
        let mut local: DmgResult = core::mem::zeroed();
        let outp: *mut DmgResult = if arg4 != 0 { dmg_ptr as *mut DmgResult } else { &mut local };
        let dmg_ptr: *mut c_void = outp as *mut c_void;
        let out = &mut *outp;
        let best = &*log.add(best_idx);
        let mut dir: f32 = 0.0;
        let mut angle: f32 = 0.0;
        let mut element: u32 = 0;
        let mut sfx_severity: i32 = 0;
        match best.x0 as i32 {
            1 => {
                let attacker = &*fpg(best.gobj);
                let src = f32::from_bits(ext::ScriptGame_EchoSourceX(best.anon_c.hit0, attacker.cur_pos.x.to_bits() as i32) as u32);
                dir = if (*fp).cur_pos.x > src { -1.0 } else { 1.0 };
                let h = &*best.anon_c.hit0;
                angle = h.kb_angle as u32 as f32;
                element = h.element;
                sfx_severity = h.sfx_severity;
            }
            2 => {
                let item = &*(fpg(best.gobj) as It);
                let mut vel_x = item.x40_vel.x;
                if vel_x < 0.0 {
                    vel_x = -vel_x;
                }
                dir = if vel_x < ext::icd().x78_float {
                    if (*fp).cur_pos.x > item.pos.x { -1.0 } else { 1.0 }
                } else if item.x40_vel.x < 0.0 {
                    1.0
                } else {
                    -1.0
                };
                let h = &*best.anon_c.hit0;
                angle = h.kb_angle as u32 as f32;
                element = h.element;
                sfx_severity = h.sfx_severity;
            }
            3 => {
                let env = &*(best.anon_c.unk_anim0 as *const lbColl_80008D30_arg1);
                dir = (*fp).facing_dir;
                sfx_severity = 0;
                angle = env.kb_angle as u32 as f32;
                element = env.element as u32;
            }
            _ => {}
        }
        if (*best.anon_c.hit0).kb_angle as u32 == 0x16A {
            let hurt = &*best.anon_10.hurt1;
            let dx = fmadds(0.5, hurt.capsule.a_pos.x + hurt.capsule.b_pos.x, -best.pos.x);
            let dy = fmadds(0.5, hurt.capsule.a_pos.y + hurt.capsule.b_pos.y, -best.pos.y);
            dir = if dx < 0.0 { 1.0 } else { -1.0 };
            let abs_dx = if dx < 0.0 { -dx } else { dx };
            let angle_int: i32 = if abs_dx < 1e-5 { 0 } else { (melee_runtime::lbtrigf::atanf(dy / abs_dx) * 57.29577951_f32) as i32 };
            angle = angle_int as f32;
        }
        if best.x0 as i32 == 1 {
            ext::Geno_Autolink(fpg(best.gobj), best.anon_c.hit0, &mut dir, &mut angle, &mut best_kb);
        }
        out.dir = dir;
        out.angle = angle as i32;
        out.hurt_height = (*best.anon_10.hurt1).height as i32;
        out.kb = best_kb;
        out.pos = best.pos;
        out.element = element;
        out.source = best.gobj;
        out.damage = best.x20;
        out.sfx_severity = sfx_severity;
        match best.x0 as i32 {
            1 => {
                let attacker_gobj = (*fpg(best.gobj)).gobj;
                let a = &mut *fpg(attacker_gobj);
                let p = &mut a.x2074 as *mut Struct2074 as *mut c_void;
                ftcoll_8007861c(attacker_gobj, gobj, 1, a.kind as i32, a.x2070.x2070_int, p, a.x2074.x2088, dmg_ptr, 0);
            }
            2 => {
                let item_sub = &*(fpg(best.gobj) as It);
                let victim = &*fpg(gobj);
                let ip = fpg(item_sub.entity) as It;
                if (*ip).kind as i32 == It_Kind_BombHei as i32 {
                    ext::pl_80041B08(victim.player_id as i32, victim.is_sub_fighter() as usize as *mut c_void, (*ip).x1C as u16);
                }
                item_damage_record(ip, gobj, dmg_ptr);
            }
            3 => {
                ftcoll_8007861c(core::ptr::null_mut(), gobj, 3, best.kind as i32, 0, core::ptr::null_mut(), 0, dmg_ptr, 1);
            }
            _ => {}
        }
        if out.element == HitElement_Electric as u32 {
            (*fp).x1960_vibrateMult = cd().x1A4;
        }
        if arg4 == 0 {
            let d = &mut (*fp).dmg;
            *(&mut d.x1870 as *mut *mut DmgLogEntry as *mut f32) = out.dir;
            d.x1874 = out.angle;
            d.x1878 = out.hurt_height;
            d.x187c = out.kb;
            d.x1880 = out.pos;
            d.x188c = out.element as _;
            d.x1890 = out.sfx_severity;
            d.x1894 = out.source as usize as u32 as i32;
            d.x1898 = out.damage;
        }
    }
}

/// `ftColl_8007AB48`: resolve damage log 0 into `fp->dmg`.
pub fn ftcoll_8007ab48(g: G) {
    melee_core::ct!("ftcoll_8007ab48");
    unsafe {
        let fp = &mut *fpg(g);
        let p = &mut fp.dmg.facing_dir_1 as *mut f32 as *mut c_void;
        ftcoll_8007a06c(g, p, dmg_log0.as_mut_ptr(), log0_idx() as u32 as usize, 1);
    }
}

/// `ftColl_8007AB80`: resolve the tip log into `fp->dmg.x1870`.
pub fn ftcoll_8007ab80(g: G) {
    melee_core::ct!("ftcoll_8007ab80");
    unsafe {
        let fp = fpg(g);
        (*fp).dmg.x187c = 0.0;
        let p = &mut (*fp).dmg.x1870 as *mut *mut DmgLogEntry as *mut c_void;
        ftcoll_8007a06c(g, p, dmg_log1.as_mut_ptr(), log1_idx() as u32 as usize, 0);
        (*fp).dmg.x18a0 = (*fp).dmg.x187c;
    }
}

/// `ftColl_8007ABD0`: set a hitbox's damage from a script value.
pub fn ftcoll_8007abd0(arg0: HC, arg1: u32, arg2: G) {
    melee_core::ct!("ftcoll_8007abd0");
    unsafe {
        let fp = fpg(arg2);
        let mut dmg = arg1 as f32;
        if (*fp).x34_scale.y != 1.0 {
            dmg = ext::ftCo_CalcYScaledKnockback(dmg, (*fp).x34_scale.y, super::ftchangeparam::ext::d6524().x4);
        }
        let scaled = ext::ftCo_800DEEB8(fp, dmg);
        (*arg0).unk_count = scaled as i32 as u32;
        (*arg0).damage = ext::ft_80089228(fp, (*fp).x2068_attackID as i32, (*fp).x206C_attack_instance as i32, scaled);
        ext::ScriptGame_HitRuleDamage(arg0);
    }
}

/// `ftColl_8007AC68`.
pub fn ftcoll_8007ac68(kb_angle: u32) -> bool {
    melee_core::ct!("ftcoll_8007ac68");
    let c = cd();
    kb_angle != 361 && c.unk_kb_angle_min <= kb_angle && kb_angle <= c.unk_kb_angle_max
}

/// `ftColl_8007AC9C`.
pub fn ftcoll_8007ac9c(arg0: HC, arg1: i32, arg2: G) {
    melee_core::ct!("ftcoll_8007ac9c");
    unsafe {
        let fp = &mut *fpg(arg2);
        (*arg0).kb_angle = arg1 as _;
        if ftcoll_8007ac68(arg1 as u32) && fp.x2074.anon_10.anon_0.x2084_b0() == 0 {
            fp.x2074.anon_10.anon_0.set_x2084_b0(1);
            ext::pl_80037ECC(arg2);
        }
    }
}

fn hit_offset(fp: &Fighter, h: &HitCapsule) -> Vec3 {
    if h.x43_b1() != 0 {
        let t = 1.0 / fp.x34_scale.y;
        Vec3 { x: h.b_offset.x * t, y: h.b_offset.y * t, z: h.b_offset.z * t }
    } else {
        h.b_offset
    }
}

/// `ftColl_8007AD18`: advance a hitbox's position.
pub fn ftcoll_8007ad18(fp: &Fighter, h: &mut HitCapsule) {
    melee_core::ct!("ftcoll_8007ad18");
    let st = h.state as u32;
    if st == HitCapsule_Enabled as u32 {
        let sp = hit_offset(fp, h);
        crate::hooks::lb_8000b1cc(h.jobj, &sp, &mut h.x4C);
        h.x58 = h.x4C;
        h.state = HitCapsule_Unk2 as _;
    } else if st == HitCapsule_Unk2 as u32 || st == HitCapsule_Unk3 as u32 {
        if st == HitCapsule_Unk2 as u32 {
            h.state = HitCapsule_Unk3 as _;
        }
        h.x58 = h.x4C;
        let sp = hit_offset(fp, h);
        crate::hooks::lb_8000b1cc(h.jobj, &sp, &mut h.x4C);
    }
}

/// `ftColl_8007AE80`.
pub fn ftcoll_8007ae80(g: G) {
    melee_core::ct!("ftcoll_8007ae80");
    unsafe {
        let fp = fpg(g);
        for i in 0..4 {
            ftcoll_8007ad18(&*fp, &mut (*fp).x914[i]);
        }
    }
}

/// `ftColl_8007AF60`: update the dynamics hit positions.
pub fn ftcoll_8007af60(g: G) {
    melee_core::ct!("ftcoll_8007af60");
    unsafe {
        let fp = &mut *fpg(g);
        let mut i = 0usize;
        while (i as u32) < fp.x166C as u32 {
            let e = &mut *fp.x1670.as_mut_ptr().add(i); // the C indexes past the declared [1]
            crate::hooks::lb_8000b1cc(e.jobj, &e.v1, &mut e.x18);
            i += 1;
        }
    }
}

/// `ftColl_8007AFC8`.
pub fn ftcoll_8007afc8(g: G, hit_idx: i32) {
    melee_core::ct!("ftcoll_8007afc8");
    unsafe { ext::lbColl_80008428(&mut (*fpg(g)).x914[hit_idx as usize]) }
}

/// `ftColl_8007AFF8`: clear all hitboxes.
pub fn ftcoll_8007aff8(g: G) {
    melee_core::ct!("ftcoll_8007aff8");
    unsafe {
        let fp = fpg(g);
        for i in 0..4 {
            ext::lbColl_80008428(&mut (*fp).x914[i]);
        }
        (*fp).set_x2219_b3(0);
    }
}

/// `ftColl_8007B064`.
pub fn ftcoll_8007b064(g: G, arg1: i32) {
    melee_core::ct!("ftcoll_8007b064");
    unsafe {
        let fp = fpg(g);
        let c: HC = &mut (*fp).x914[arg1 as usize];
        ext::lbColl_80008434(c);
        ext::lbColl_80008440(c);
        (*fp).set_x2219_b3(1);
    }
}

/// `ftColl_8007B0C0`: set every hurtbox's state.
pub fn ftcoll_8007b0c0(g: G, arg1: u32) {
    melee_core::ct!("ftcoll_8007b0c0");
    unsafe {
        let fp = &mut *fpg(g);
        let mut i = 0usize;
        while (i as u32) < fp.hurt_capsules_len as u32 {
            fp.hurt_capsules[i].capsule.state = arg1 as _;
            fp.hurt_capsules[i].capsule.set_skip_update_pos(0);
            i += 1;
        }
        fp.set_x221A_b5((arg1 != HurtCapsule_Enabled as u32) as _);
    }
}

/// `ftColl_8007B128`: set the state of the hurtbox on `bone_id`.
pub fn ftcoll_8007b128(g: G, bone_id: i32, state: u32) {
    melee_core::ct!("ftcoll_8007b128");
    unsafe {
        let fp = &mut *fpg(g);
        let mut i = 0i32;
        while i < fp.hurt_capsules_len as i32 {
            let hurt = &mut fp.hurt_capsules[i as usize];
            if hurt.capsule.bone_idx as i32 == bone_id {
                hurt.capsule.state = state as _;
                if state != HurtCapsule_Enabled as u32 {
                    fp.set_x221A_b5(1);
                }
                return;
            }
            i += 1;
        }
        if ext::Geno_DefineIsNone(fp.kind as i32) != 0 && ext::Geno_DefineTemplateSkeleton(fp.kind as i32) == 0 {
            return;
        }
        panic!("in ftCollisionSetHitStatus illegal parts!");
    }
}

/// `ftColl_8007B1B8`: start a shield.
pub fn ftcoll_8007b1b8(g: G, shield: &ShieldDesc, cb: HSD_GObjEvent) {
    melee_core::ct!("ftcoll_8007b1b8");
    unsafe {
        let fp = &mut *fpg(g);
        fp.anon_221b.set_x221B_b0(1);
        fp.anon_221b.set_x221B_b1(0);
        fp.anon_221b.set_x221B_b2(0);
        fp.anon_221b.set_x221B_b3(0);
        fp.anon_221b.set_x221B_b4(0);
        fp.shield_hit_cb = cb;
        fp.shield_hit.bone = (*fp.parts.add(shield.bone as usize)).joint;
        fp.shield_hit.size = shield.radius;
        fp.shield_hit.offset = shield.pos;
    }
}

/// `ftColl_CreateReflectHit`.
pub fn ftcoll_create_reflect_hit(g: G, reflect: &ReflectDesc, cb: HSD_GObjEvent) {
    melee_core::ct!("ftcoll_create_reflect_hit");
    unsafe {
        let fp = &mut *fpg(g);
        fp.set_reflecting(1);
        fp.set_x2218_b4(0);
        fp.reflect_hit_cb = cb;
        fp.ReflectAttr.x1A30_maxDamage = reflect.x4_max_damage as _;
        fp.ReflectAttr.x1A34_damageMul = reflect.x18_damage_mul;
        fp.ReflectAttr.x1A38_speedMul = reflect.x1C_speed_mul;
        fp.set_x2218_b5(reflect.x20_behavior as _);
        fp.reflect_hit.bone = (*fp.parts.add(reflect.x0_bone_id as usize)).joint;
        fp.reflect_hit.size = reflect.x14_size;
        fp.reflect_hit.offset = reflect.x8_offset;
    }
}

/// `ftColl_CreateAbsorbHit`.
pub fn ftcoll_create_absorb_hit(g: G, absorb: &AbsorbDesc) {
    melee_core::ct!("ftcoll_create_absorb_hit");
    unsafe {
        let fp = &mut *fpg(g);
        fp.set_x2218_b6(1);
        fp.set_x2218_b7(0);
        fp.absorb_hit.bone = (*fp.parts.add(absorb.x0_bone_id as usize)).joint;
        fp.absorb_hit.size = absorb.x10_size;
        fp.absorb_hit.offset = absorb.x4_offset;
    }
}

unsafe fn hurtbox_init(fp: &mut Fighter, hurt: *mut FighterHurtCapsule, init: &ftHurtboxInit) {
    let parts = fp.parts;
    let h = &mut *hurt;
    h.capsule.bone_idx = init.bone_idx as _;
    h.height = init.height as _;
    h.is_grabbable = init.is_grabbable as _;
    h.capsule.state = HurtCapsule_Enabled as _;
    h.capsule.bone = (*parts.add(h.capsule.bone_idx as usize)).joint;
    h.capsule.a_offset = init.a_offset;
    h.capsule.b_offset = init.b_offset;
    h.capsule.scale = init.scale;
}

/// `ftColl_8007B320`: build the hurtboxes and dynamics hits from the fighter data.
pub fn ftcoll_8007b320(g: G) {
    melee_core::ct!("ftcoll_8007b320");
    unsafe {
        let fp = &mut *fpg(g);
        let x30 = &*(*fp.ft_data).x30;
        let dyn_ = &*(*fp.ft_data).x2C;
        if x30.count > 0xF {
            panic!("fighter hit num over!");
        }
        fp.hurt_capsules_len = x30.count as _;
        let mut i = 0u32;
        while i < x30.count as u32 {
            let h: *mut FighterHurtCapsule = &mut fp.hurt_capsules[i as usize];
            hurtbox_init(fp, h, &*x30.inits.add(i as usize));
            i += 1;
        }
        if dyn_.x4 > 0xB {
            panic!("fighter dynamics hit num over!");
        }
        fp.x166C = dyn_.x4 as _;
        let mut i = 0u32;
        while i < dyn_.x4 as u32 {
            let init = &*dyn_.x8.add(i as usize);
            let joint = (*fp.parts.add(init.x0 as usize)).joint;
            let dst = &mut *fp.x1670.as_mut_ptr().add(i as usize); // the C indexes past the declared [1]
            dst.x24 = init.x0 as _;
            dst.jobj = joint;
            dst.v1 = init.x4;
            dst.v2 = init.x10;
            i += 1;
        }
    }
}

/// `ftColl_8007B4E0`: reset the hurtboxes from the fighter data.
pub fn ftcoll_8007b4e0(g: G) {
    melee_core::ct!("ftcoll_8007b4e0");
    unsafe {
        let fp = &mut *fpg(g);
        let x30 = &*(*fp.ft_data).x30;
        let mut i = 0i32;
        while i < fp.hurt_capsules_len as i32 {
            let h: *mut FighterHurtCapsule = &mut fp.hurt_capsules[i as usize];
            hurtbox_init(fp, h, &*x30.inits.add(i as usize));
            (*h).capsule.set_skip_update_pos(0);
            i += 1;
        }
        fp.set_x221A_b6(0);
    }
}

/// `ftColl_HurtboxInit`.
pub fn ftcoll_hurtbox_init(fp: &mut Fighter, hurt: *mut FighterHurtCapsule, init: &ftHurtboxInit) {
    melee_core::ct!("ftcoll_hurtbox_init");
    unsafe { hurtbox_init(fp, hurt, init) };
    fp.set_x221A_b6(1);
}

/// `ftColl_8007B62C`: set the intangibility kind and its colanim.
pub fn ftcoll_8007b62c(g: G, arg1: i32) {
    melee_core::ct!("ftcoll_8007b62c");
    unsafe {
        let fp = &mut *fpg(g);
        fp.x1988 = arg1 as _;
        match arg1 {
            0 => {
                ext::ftCo_800BFFD0(fp, 1, 0);
            }
            1 => {
                ext::ftCo_800BFFD0(fp, 3, 0);
            }
            2 => {
                ext::ftCo_800BFFD0(fp, 2, 0);
            }
            _ => {}
        }
    }
}

/// `ftColl_8007B6A0`.
pub fn ftcoll_8007b6a0(g: G) {
    melee_core::ct!("ftcoll_8007b6a0");
    unsafe {
        let fp = &mut *fpg(g);
        fp.set_x2221_b0(1);
        fp.x1990 = 0;
        fp.x198C = 2;
        ext::ftCo_800BFFD0(fp, 9, 0);
    }
}

/// `ftColl_8007B6EC`.
pub fn ftcoll_8007b6ec(g: G) {
    melee_core::ct!("ftcoll_8007b6ec");
    unsafe {
        let fp = &mut *fpg(g);
        fp.set_x2221_b0(0);
        if fp.x1990 == 0 {
            fp.x198C = if fp.x1994 != 0 { 1 } else { 0 };
            if ext::ftCo_800C0694(fp) == 9 {
                ext::ftCo_800C0200(fp, 9);
            }
        }
    }
}

/// `ftColl_8007B760`.
pub fn ftcoll_8007b760(g: G, arg1: i32) {
    melee_core::ct!("ftcoll_8007b760");
    unsafe {
        let fp = &mut *fpg(g);
        if arg1 > fp.x1990 {
            fp.x1990 = arg1;
        }
        fp.x198C = 2;
        ext::ftCo_800BFFD0(fp, 9, 0);
    }
}

/// `ftColl_8007B7A4`.
pub fn ftcoll_8007b7a4(g: G, arg1: i32) {
    melee_core::ct!("ftcoll_8007b7a4");
    unsafe {
        let fp = &mut *fpg(g);
        if arg1 > fp.x1994 {
            fp.x1994 = arg1;
        }
        fp.x198C = if fp.x1990 != 0 { 2 } else { 1 };
        ext::ftCo_800BFFD0(fp, 9, 0);
    }
}

/// `ftColl_8007B7FC`: start star invincibility.
pub fn ftcoll_8007b7fc(fp: F, arg1: i32) {
    melee_core::ct!("ftcoll_8007b7fc");
    unsafe {
        let f = &mut *fp;
        f.set_x221D_b6(1);
        f.x2004 = arg1;
        ext::ftCo_800BFFD0(f, 107, 0);
        if arg1 > ext::it_8026B588() {
            ext::ft_80088080(fp);
        }
    }
}

fn hit_status(fp: &Fighter) -> i32 {
    let mut ret = (fp.x221D_b6() != 0) as i32;
    if fp.x1988 as i32 > ret {
        ret = fp.x1988 as i32;
    }
    if fp.x198C as i32 > ret {
        ret = fp.x198C as i32;
    }
    ret
}

/// `ftColl_8007B868`: the strongest hit status.
pub fn ftcoll_8007b868(g: G) -> i32 {
    melee_core::ct!("ftcoll_8007b868");
    unsafe { hit_status(&*fpg(g)) }
}

/// `ftColl_8007B8E8`: release every fighter thrown by `gobj`.
pub fn ftcoll_8007b8e8(gobj: G) {
    melee_core::ct!("ftcoll_8007b8e8");
    let mut cur = crate::hooks::plink_head(PLINK_FIGHTER);
    while !cur.is_null() {
        unsafe {
            if gobj != cur {
                let fp = &mut *fpg(cur);
                if fp.x1064_thrownHitbox.anon_134.owner == gobj {
                    fp.x1064_thrownHitbox.anon_134.owner = core::ptr::null_mut();
                }
            }
            cur = (*cur).next;
        }
    }
}

/// `ftColl_GetWindOffsetVec`: sum of the active wind devices.
pub fn ftcoll_get_wind_offset_vec(fgp: G, out: &mut Vec3) {
    melee_core::ct!("ftcoll_get_wind_offset_vec");
    out.x = 0.0;
    out.y = 0.0;
    out.z = 0.0;
    unsafe {
        if (*fpg(fgp)).x2224_b4() != 0 {
            return;
        }
        use super::ftdevice::{ft_804D6578, ft_80459A68};
        let mut i = 0i32;
        while i < ft_804D6578.x0 {
            let e = &ft_80459A68[i as usize];
            let grp = e.ground;
            i += 1;
            if ext::ScriptStageHostHidden != 0 && !grp.is_null() && (*grp).classifier == 3 {
                continue;
            }
            if !e.ground.is_null() {
                let mut wind = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                if ext::ftCo_800C0A28(fgp, grp as *mut c_void, e.r#type) && (e.active_cb.expect("wind cb"))(grp, fgp, &mut wind) != 0 {
                    out.x += wind.x;
                    out.y += wind.y;
                    out.z += wind.z;
                }
            }
        }
    }
}

/// `ftColl_8007BA0C`: bury devices.
pub fn ftcoll_8007ba0c(g: G) {
    melee_core::ct!("ftcoll_8007ba0c");
    unsafe {
        if (*fpg(g)).x2219_b1() != 0 {
            return;
        }
        use super::ftdevice::{ftDevice_BuryThingCount, ftDevice_BuryThings};
        let mut i = 0i32;
        while i < ftDevice_BuryThingCount {
            let e = &ftDevice_BuryThings[i as usize];
            i += 1;
            let ground = e.x0;
            if !ground.is_null() {
                let ty = e.x4 as u32;
                if ext::ftCo_800C0A28(g, ground, ty) && (e.cb.expect("bury cb"))(ground, g) != 0 {
                    ext::ftCo_800C0874(g, ground, ty);
                }
            }
        }
    }
}

/// `ftColl_8007BAC0`: dynamics (damage) devices.
pub fn ftcoll_8007bac0(g: G) {
    melee_core::ct!("ftcoll_8007bac0");
    if ext::gm_8016B1C4() {
        return;
    }
    unsafe {
        use super::ftdevice::{ft_804D6570, ft_80459A8C};
        let max = hit_status(&*fpg(g));
        let mut i = 0i32;
        while i < ft_804D6570 {
            let e = &ft_80459A8C[i as usize];
            i += 1;
            let ground = e.ground;
            if !ground.is_null() {
                let ty = e.r#type;
                if ext::ftCo_800C0A28(g, ground as *mut c_void, ty) {
                    // The callback writes a DynamicsDesc* through its Vec3* out parameter.
                    let mut desc: [*mut DynamicsDesc; 3] = [core::ptr::null_mut(); 3];
                    if (e.active_cb.expect("dyn cb"))(ground, g, desc.as_mut_ptr() as *mut Vec3) != 0 && max == 0 {
                        ext::ftCo_800C08A0(g, ground, desc[0], ty);
                    }
                }
            }
        }
        if max == 0 {
            ext::ftCo_800C0B20(g);
        }
    }
}

/// `ftColl_8007BBCC`: lipstick damage taken this frame.
pub fn ftcoll_8007bbcc(_g: G) -> f32 {
    melee_core::ct!("ftcoll_8007bbcc");
    let mut dmg: f32 = 0.0;
    let n = log0_idx() as u32 as usize;
    unsafe {
        for e in dmg_log0.iter().take(n) {
            match e.x0 as i32 {
                x if x == EntityKind_Fighter as i32 => {
                    let h = &*e.anon_c.hit0;
                    if h.element == HitElement_Lipstick as u32 {
                        dmg += h.damage;
                    }
                }
                x if x == EntityKind_Item as i32 => {
                    if (*e.anon_c.hit0).element == HitElement_Lipstick as u32 {
                        dmg += ext::it_8026B1D4(e.gobj, e.anon_c.hit0);
                    }
                }
                _ => {}
            }
        }
    }
    dmg
}

/// `ftColl_8007BC90`: find grabbable items.
pub fn ftcoll_8007bc90(g: G) {
    melee_core::ct!("ftcoll_8007bc90");
    unsafe {
        let fp = fpg(g);
        (*fp).target_item_gobj = core::ptr::null_mut();
        (*fp).unk_grab_val = f32::MAX;
        let mut cur = crate::hooks::plink_head(PLINK_ITEM);
        'next: while !cur.is_null() {
            let ipg = cur;
            cur = (*cur).next;
            let ip = fpg(ipg) as It;
            let it = &mut *ip;
            if it.xD0C != 0 || it.xDD0_flag.anon_0.b4() == 0 || it.xD09 != 0 {
                continue;
            }
            for i in 0..4usize {
                let h: HC = &mut (*fp).x914[i];
                if (*h).state == HC_DISABLED || (*h).element != HitElement_Catch as u32 {
                    continue;
                }
                if ((*h).x40_b2() == 0 || ga(it.ground_or_air) != AIR) && ((*h).x40_b3() == 0 || ga(it.ground_or_air) != GROUND) {
                    continue;
                }
                if ext::lbColl_8000ACFC(ip as *mut c_void, h) {
                    continue;
                }
                let mut j = 0u32;
                while j < it.xAC8_hurtboxNum as u32 {
                    if ext::lbColl_80007ECC(h, &mut it.xACC_itemHurtbox[j as usize], core::ptr::null_mut(), (*fp).x34_scale.y, it.scl, 0.0) {
                        ftcoll_80076808(fp, h, 0, ip as *mut c_void, false);
                        let mut dist = it.pos.x;
                        dist = dist - (*fp).cur_pos.x;
                        if dist < 0.0 {
                            dist = -dist;
                        }
                        let f = &mut *fp;
                        if dist < f.unk_grab_val {
                            let ent = it.entity;
                            f.x1A64 = ent as *mut c_void;
                            f.target_item_gobj = ent;
                            f.anon_221b.set_x221B_b6(1);
                            f.unk_grab_val = dist;
                        }
                        continue 'next;
                    }
                    j += 1;
                }
            }
        }
    }
}

/// `ftColl_8007BE3C`: apply a deferred hit (x1894 source / x1898 damage).
pub fn ftcoll_8007be3c(g: G) {
    melee_core::ct!("ftcoll_8007be3c");
    unsafe {
        let fp = &mut *fpg(g);
        let dmg_count = get_env_dmg(fp.dmg.x1898);
        if fp.x221C_b4() != 0 {
            fp.dmg.x1834 = fp.dmg.x1834 - fp.dmg.x1898;
            if fp.dmg.x1834 < 0.0 {
                fp.dmg.x1898 = -fp.dmg.x1834;
                fp.set_x221C_b4(0);
            }
        }
        if fp.x221C_b4() != 0 {
            return;
        }
        fp.dmg.x1838_percentTemp += fp.dmg.x1898;
        if dmg_count > fp.dmg.x183C_applied {
            fp.dmg.x183C_applied = dmg_count;
        }
        // x1894 is an int field holding a GObj pointer (the game's 32-bit pointers).
        let source = fp.dmg.x1894 as u32 as usize as G;
        match (*source).classifier as i32 {
            x if x == HSD_GOBJ_CLASS_FIGHTER as i32 => {
                let victim = fp.gobj;
                let amount = fp.dmg.x1898;
                ext::plStale_UpdateStaleMovesFromFighter(source, victim);
                ftcoll_80076444(source, victim);
                let (s, v) = (&*fpg(source), &*fpg(victim));
                ext::pl_8003EB30(amount, s.player_id as i32, s.is_sub_fighter() as i32, v.player_id as i32, v.is_sub_fighter() as i32, s.x2070.anon_0.x2073 as i32);
            }
            x if x == HSD_GOBJ_CLASS_ITEM as i32 => {
                let victim = (*fpg(g)).gobj;
                let amount = (*fpg(g)).dmg.x1898;
                ext::plStale_UpdateStaleMovesFromItem(source, victim);
                ftcoll_8007646c(source, victim);
                let ip = &*(fpg(source) as It);
                if ext::ftLib_80086960(ip.owner) {
                    let (o, v) = (&*fpg(ip.owner), &*fpg(victim));
                    ext::pl_8003EB30(amount, o.player_id as i32, o.is_sub_fighter() as i32, v.player_id as i32, v.is_sub_fighter() as i32, ip.xD90.anon_0.x2073 as i32);
                }
            }
            _ => {}
        }
        let fp = &mut *fpg(g);
        let x187c = fp.dmg.x187c;
        let dmg_unsigned = fp.dmg.x1898 as u32;
        let x1890 = fp.dmg.x1890 as u32;
        let effect = hit_effect_ids[fp.dmg.x188c as usize];
        let pos: *const Vec3 = &fp.dmg.x1880;
        match effect {
            x if x == Ef_Id_Unk1000 as i32 => ftcoll_80078538(g, pos, x1890, dmg_unsigned as f32, x187c),
            x if x == Ef_Id_Unk1001 as i32
                || x == Ef_Id_Unk1002 as i32
                || x == Ef_Id_Unk1004 as i32
                || x == Ef_Id_Unk1046 as i32
                || x == Ef_Id_Unk1145 as i32
                || x == Ef_Id_Unk1255 as i32 =>
            {
                ext::efSync_Spawn(effect, core::ptr::null_mut(), pos, core::ptr::null())
            }
            x if x == Ef_Id_Unk1005 as i32 => ext::efSync_Spawn(effect, core::ptr::null_mut(), pos, &fp.facing_dir),
            _ => {}
        }
    }
}

// ---- capsule setters (port/fill-melee_chara) ----
#[inline(always)]
fn fpg_r<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftColl_8007ABD0(hit, damage, gobj)`: set a capsule's damage from the move's base damage
/// (size scaling, smash charge, staling) and let the LAB's hit rules adjust it.
pub fn ftColl_8007ABD0(hit: *mut HitCapsule, damage: u32, gobj: G) {
    melee_core::ct!("ftColl_8007ABD0");
    let fp = fpg_r(gobj);
    let mut dmg = damage as f32;
    if fp.x34_scale.y != 1.0 {
        let x4 = crate::tu::ftchangeparam::ext::d6524().x4;
        dmg = crate::tu::ftchangeparam::ftCo_CalcYScaledKnockback(dmg, fp.x34_scale.y, x4);
    }
    let scaled = crate::ft0df::ftco_800deeb8(fp, dmg);
    unsafe {
        (*hit).unk_count = (scaled as i32) as u32;
        (*hit).damage = crate::stale::apply_staling(fp, fp.x2068_attackID, fp.x206C_attack_instance as i32, scaled);
        // TARGET_PC: ScriptGame_HitRuleDamage(hit)
        melee_runtime::tu::script_hit::ScriptGame_HitRuleDamage(hit);
    }
}

/// `ftColl_8007AFF8(gobj)`: disable every hit capsule.
pub fn ftColl_8007AFF8(gobj: G) {
    melee_core::ct!("ftColl_8007AFF8");
    let fp = fpg_r(gobj);
    for i in 0..fp.x914.len() {
        unsafe { melee_runtime::tu::lbcollision::lbColl_80008428(&mut fp.x914[i]) };
    }
    fp.set_x2219_b3(0);
}

/// `ftColl_8007B064(gobj, idx)`: enable hit capsule `idx` and clear its victim lists.
pub fn ftColl_8007B064(gobj: G, idx: i32) {
    melee_core::ct!("ftColl_8007B064");
    let fp = fpg_r(gobj);
    let capsule: *mut HitCapsule = &mut fp.x914[idx as usize];
    unsafe {
        melee_runtime::tu::lbcollision::lbColl_80008434(capsule);
        melee_runtime::tu::lbcollision::lbColl_80008440(capsule);
    }
    fp.set_x2219_b3(1);
}

/// `ftColl_8007B0C0(gobj, state)`: set every hurt capsule's state.
pub fn ftColl_8007B0C0(gobj: G, state: HurtCapsuleState) {
    melee_core::ct!("ftColl_8007B0C0");
    let fp = fpg_r(gobj);
    for i in 0..fp.hurt_capsules_len as usize {
        fp.hurt_capsules[i].capsule.state = state;
        fp.hurt_capsules[i].capsule.set_skip_update_pos(0);
    }
    if state == HurtCapsule_Enabled {
        fp.set_x221A_b5(0);
    } else {
        fp.set_x221A_b5(1);
    }
}

/// `ftColl_8007B128(gobj, bone_id, state)`: set the state of the hurt capsule on a bone.
pub fn ftColl_8007B128(gobj: G, bone_id: i32, state: HurtCapsuleState) {
    melee_core::ct!("ftColl_8007B128");
    let fp = fpg_r(gobj);
    for i in 0..fp.hurt_capsules_len as usize {
        let hurt = &mut fp.hurt_capsules[i];
        if hurt.capsule.bone_idx == bone_id {
            hurt.capsule.state = state;
            if state != HurtCapsule_Enabled {
                fp.set_x221A_b5(1);
            }
            return;
        }
    }
    // TARGET_PC: a base "none" define has its own skeleton, so a script word it borrowed from the
    // donor can name a joint it has no hurtbox on. Skip it (retail data and every other define assert).
    if crate::hooks::geno_define_is_none(fp.kind as i32) != 0 && crate::hooks::geno_define_template_skeleton(fp.kind as i32) == 0 {
        return;
    }
    panic!("ftcoll.c:0x888 in ftCollisionSetHitStatus illegal parts!");
}

/// `ftColl_8007B1B8(gobj, shield, cb)`: set up the shield hit sphere.
pub fn ftColl_8007B1B8(gobj: G, shield: *const ShieldDesc, cb: HSD_GObjEvent) {
    melee_core::ct!("ftColl_8007B1B8");
    let fp = fpg_r(gobj);
    fp.anon_221b.set_x221B_b0(1);
    fp.anon_221b.set_x221B_b1(0);
    fp.anon_221b.set_x221B_b2(0);
    fp.anon_221b.set_x221B_b3(0);
    fp.anon_221b.set_x221B_b4(0);
    fp.shield_hit_cb = cb;
    unsafe {
        fp.shield_hit.bone = (*fp.parts.add((*shield).bone as usize)).joint;
        fp.shield_hit.size = (*shield).radius;
        fp.shield_hit.offset = (*shield).pos;
    }
}

/// `ftColl_8007B62C(gobj, kind)`.
pub fn ftColl_8007B62C(gobj: G, arg1: i32) {
    melee_core::ct!("ftColl_8007B62C");
    let fp = fpg_r(gobj);
    fp.x1988 = arg1;
    match arg1 {
        0 => {
            ext::ftCo_800BFFD0(fp, 1, 0);
        }
        1 => {
            ext::ftCo_800BFFD0(fp, 3, 0);
        }
        2 => {
            ext::ftCo_800BFFD0(fp, 2, 0);
        }
        _ => {}
    }
}

/// `ftColl_8007B760(gobj, x)`.
pub fn ftColl_8007B760(gobj: G, arg1: i32) {
    melee_core::ct!("ftColl_8007B760");
    let fp = fpg_r(gobj);
    if arg1 > fp.x1990 {
        fp.x1990 = arg1;
    }
    fp.x198C = 2;
    ext::ftCo_800BFFD0(fp, 9, 0);
}

/// `ftColl_CreateAbsorbHit(gobj, absorb)`.
pub fn ftColl_CreateAbsorbHit(gobj: G, absorb: *const AbsorbDesc) {
    melee_core::ct!("ftColl_CreateAbsorbHit");
    let fp = fpg_r(gobj);
    fp.set_x2218_b6(1);
    fp.set_x2218_b7(0);
    unsafe {
        fp.absorb_hit.bone = (*fp.parts.add((*absorb).x0_bone_id as usize)).joint;
        fp.absorb_hit.size = (*absorb).x10_size;
        fp.absorb_hit.offset = (*absorb).x4_offset;
    }
}

/// `ftColl_CreateReflectHit(gobj, reflect, cb)`.
pub fn ftColl_CreateReflectHit(gobj: G, reflect: *const ReflectDesc, cb: HSD_GObjEvent) {
    melee_core::ct!("ftColl_CreateReflectHit");
    let fp = fpg_r(gobj);
    fp.set_reflecting(1);
    fp.set_x2218_b4(0);
    fp.reflect_hit_cb = cb;
    unsafe {
        fp.ReflectAttr.x1A30_maxDamage = (*reflect).x4_max_damage;
        fp.ReflectAttr.x1A34_damageMul = (*reflect).x18_damage_mul;
        fp.ReflectAttr.x1A38_speedMul = (*reflect).x1C_speed_mul;
        fp.set_x2218_b5((*reflect).x20_behavior);
        fp.reflect_hit.bone = (*fp.parts.add((*reflect).x0_bone_id as usize)).joint;
        fp.reflect_hit.size = (*reflect).x14_size;
        fp.reflect_hit.offset = (*reflect).x8_offset;
    }
}

/// `ftColl_HurtboxInit(fp, hurt, init)`.
pub fn ftColl_HurtboxInit(fp: *mut Fighter, hurt: *mut FighterHurtCapsule, init: *const ftHurtboxInit) {
    melee_core::ct!("ftColl_HurtboxInit");
    unsafe {
        (*hurt).capsule.bone_idx = (*init).bone_idx as i32;
        (*hurt).height = (*init).height;
        (*hurt).is_grabbable = (*init).is_grabbable as i32;
        (*hurt).capsule.state = HurtCapsule_Enabled;
        (*hurt).capsule.bone = (*(*fp).parts.add((*hurt).capsule.bone_idx as usize)).joint;
        (*hurt).capsule.a_offset = (*init).a_offset;
        (*hurt).capsule.b_offset = (*init).b_offset;
        (*hurt).capsule.scale = (*init).scale;
        (*fp).set_x221A_b6(1);
    }
}

// ---- folded from ft0df.rs (consolidation steps 21/22) ----
// ---- ftcoll.c leaves ---------------------------------------------------------------------

pub fn ftcoll_8007aee0(g: G_ft0df) { melee_core::ct!("ftcoll_8007aee0"); fp(g).shield_hit.set_skip_update_pos(0) }

pub fn ftcoll_8007aef8(g: G_ft0df) { melee_core::ct!("ftcoll_8007aef8"); fp(g).reflect_hit.set_skip_update_pos(0) }

pub fn ftcoll_8007af10(g: G_ft0df) { melee_core::ct!("ftcoll_8007af10"); fp(g).absorb_hit.set_skip_update_pos(0) }

/// `ftColl_8007AF28`: clear `skip_update_pos` on every hurt capsule.
pub fn ftcoll_8007af28(g: G_ft0df) {
    melee_core::ct!("ftcoll_8007af28");
    let f = fp(g);
    let mut i = 0usize;
    while i < f.hurt_capsules_len as usize {
        f.hurt_capsules[i].capsule.set_skip_update_pos(0);
        i += 1;
    }
}

/// `ftColl_8007B8A8(hit, &offset)`.
pub fn ftcoll_8007b8a8(hit: &mut HitCapsule, v: &Vec3) {
    melee_core::ct!("ftcoll_8007b8a8");
    hit.jobj = core::ptr::null_mut();
    hit.b_offset = *v;
}

/// `ftColl_8007B8CC(fp, grabber)`: the thrown hitbox's owner/team come from the grabber.
pub fn ftcoll_8007b8cc(f: &mut Fighter, grabber: G_ft0df) {
    melee_core::ct!("ftcoll_8007b8cc");
    let gf = fp(grabber);
    let (team, pid) = (gf.team, gf.player_id);
    f.x1064_thrownHitbox.anon_134.owner = grabber;
    f.x119C_teamUnk = team;
    f.grabber_unk1 = pid;
}

// ---- folded from ftcoll.rs (consolidation steps 21/22) ----
/// `ftColl_800765E0`.
pub fn ftcoll_800765e0() {
    melee_core::ct!("ftcoll_800765e0");
    unsafe {
        DMG_LOG0_IDX = 0;
        DMG_LOG1_IDX = 0;
    }
}
