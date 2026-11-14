//! Geno boss grabs (docs/specs/geno-v5.2-boss.md §8 R4 / T1; notes geno-boss.md "Grabs").
//!
//! Engine side: `Op::Grab` arms a catch box on a part's joint for a motion-frame window; a fighter the host finds
//! inside it is *held* (`Cond::Holding`) while the actor's status stays in the grab's `release` mask. Leaving the mask
//! drops the victim (Brawl's `CommonCatchProcExit`); `Op::GrabRelease(Some(attack))` throws it with the attack's
//! first box (damage / angle / growth / base, L-scaled). All of it is plain data in `geno_boss::State::grab`, hashed
//! with the rest of the state.
//!
//! Host side ([`host_frame`]): Melee's own capture path. The actor's carrier fighter is the grabber: the victim
//! enters CapturePulled / CaptureWait (`fn_800DAADC`, `fn_800DB6C8`) held on the carrier's catch joint, so the grab
//! timer, mash-out (`ftCommon_GrabMash`), the escape (`CaptureCut` / `CaptureJump`) and hits on the held fighter are
//! the game's code. A throw is Melee's throw release (`ftCo_800DE2A8` with the box in the carrier's throw slot, then
//! `ftCo_800DE7C0`); a drop is `ftCo_CaptureCut_Enter`.
//!
//! The VS test host `MELEE_VS_BOSS=tabuu` runs a minimal Tabuu define built from his local IR's grab data (the
//! catch commands 060A / 060D in ChainStart and DiveStart2, the hold chains from the statuses' exit text, the throws
//! from ChainAttack's box and the Dive grab's 061A throw). His other systems are not part of it.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use crate::tu::geno_boss::{self as gb, Attack, Cond, Define, HitboxDef, Motion, Num, Op, Part, Role, Rule, Status, TimelineEvent, World};
use crate::tu::geno_boss_ir::J;
use melee_types::*;
use std::vec::Vec;

/// `EV_GRAB` values.
pub const GRAB_ARMED: i32 = 1;
pub const GRAB_THROW: i32 = 2;
pub const GRAB_DROP: i32 = 3;

/// The grab block of the actor state (plain data, fixed size).
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GrabState {
    /// a catch box is armed for status `status`
    pub armed: bool,
    pub part: u8,
    pub status: i32,
    pub joint: i32,
    pub radius: f32,
    pub frames: (i32, i32),
    /// statuses that keep the hold
    pub hold: u64,
    /// the held fighter's player slot, -1 none
    pub victim: i32,
    /// release requested: 0 none, `GRAB_THROW`, `GRAB_DROP`
    pub pending: i32,
    /// the throw's attack index (-1 none)
    pub throw: i32,
    pub catches: u32,
    pub escapes: u32,
}

pub const OFF: GrabState = GrabState { armed: false, part: 0, status: -1, joint: 0, radius: 0.0, frames: (0, -1), hold: 0, victim: -1, pending: 0, throw: -1, catches: 0, escapes: 0 };

pub fn hash_words(g: &GrabState) -> [u32; 12] {
    [
        g.armed as u32 | (g.part as u32) << 8,
        g.status as u32,
        g.joint as u32,
        g.radius.to_bits(),
        g.frames.0 as u32,
        g.frames.1 as u32,
        g.hold as u32,
        (g.hold >> 32) as u32,
        g.victim as u32,
        g.pending as u32 | (g.throw as u32) << 8,
        g.catches,
        g.escapes,
    ]
}

/// `Op::Grab`: arm the catch box for the current status (one victim at a time).
pub fn arm(g: &mut GrabState, part: u8, joint: i32, radius: f32, frames: (i32, i32), hold: u64, status: i32) {
    if g.victim >= 0 {
        return;
    }
    g.armed = true;
    g.part = part;
    g.joint = joint;
    g.radius = radius;
    g.frames = frames;
    g.hold = hold;
    g.status = status;
}

/// A status change: the catch box belongs to the status that armed it; leaving the hold statuses drops the victim.
/// True when a drop was requested.
pub fn on_status(g: &mut GrabState, st: u8) -> bool {
    if g.armed && g.status != st as i32 {
        g.armed = false;
    }
    if g.victim >= 0 && g.pending == 0 && (st >= 64 || g.hold >> st & 1 == 0) {
        g.pending = GRAB_DROP;
        g.throw = -1;
        return true;
    }
    false
}

/// `Op::GrabRelease`: true when there was a victim to release.
pub fn request_release(g: &mut GrabState, throw: Option<u8>) -> bool {
    if g.victim < 0 || g.pending != 0 {
        return false;
    }
    g.pending = if throw.is_some() { GRAB_THROW } else { GRAB_DROP };
    g.throw = throw.map_or(-1, |t| t as i32);
    true
}

/// The live catch box (part, joint, radius) on this frame, if any: the motion frame whose timeline just ran (`tl_mf`;
/// `mf` has already advanced) is inside the window.
pub fn catch_live() -> Option<(u8, i32, f32)> {
    let s = unsafe { &gb::S };
    let g = &s.grab;
    (s.live && g.armed && g.victim < 0 && s.status == g.status && s.tl_mf >= g.frames.0 && s.tl_mf <= g.frames.1).then_some((g.part, g.joint, g.radius))
}

// ------------------------------------------------------------------------------------------------- the host

fn log_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_GENO_BOSS_LOG").is_some() || std::env::var_os("MELEE_GRAB_LOG").is_some())
}

unsafe fn fighter_of(slot: i32) -> *mut Fighter {
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let p = melee_fighter::tu::player::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[0];
    if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

/// The carrier's hold joint: Melee's catch joint of its fighter data (`fn_800DA1D8`). A Brawl bone number has no
/// meaning on a Melee body; the actor's joint is kept for a host with the boss's own model.
unsafe fn hold_joint(cf: &Fighter) -> *mut HSD_JObj {
    (*cf.parts.offset((*(*cf.ft_data).x8).x11 as isize)).joint
}

fn is_captured(m: i32) -> bool {
    (ftCo_MS_CapturePulledHi as i32..=ftCo_MS_CaptureDamageLw as i32).contains(&m)
}

fn seg_point(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = dx * dx + dy * dy;
    let mut t = if l > 0.0 { ((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l } else { 0.0 };
    t = t.clamp(0.0, 1.0);
    let (ex, ey) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
    melee_runtime::gekko::sqrtf(ex * ex + ey * ey)
}

/// The host's grab work for one frame, after `geno_boss::tick`: escapes, releases, then a catch. `slot` is the
/// carrier's player slot. Returns what happened (for the host's log).
pub unsafe fn host_frame(d: &Define, slot: i32, frame: u32) {
    let cf = fighter_of(slot);
    if cf.is_null() || !gb::S.live {
        return;
    }
    let cf = &mut *cf;
    let g = &mut gb::S.grab;
    // a held victim that is no longer captured by the carrier mashed out (CaptureCut / CaptureJump), was hit out
    // of the hold, or died
    if g.victim >= 0 {
        let v = fighter_of(g.victim);
        let held = !v.is_null() && is_captured((*v).motion_id) && (*v).victim_gobj == cf.gobj;
        if !held {
            eprintln!("geno_grab: f{frame} escape: slot {} left the hold ({})", g.victim, if v.is_null() { -1 } else { (*v).motion_id });
            g.victim = -1;
            g.pending = 0;
            g.throw = -1;
            g.escapes += 1;
            if cf.victim_gobj == (if v.is_null() { core::ptr::null_mut() } else { (*v).gobj }) {
                cf.victim_gobj = core::ptr::null_mut();
            }
        } else {
            // the hold joint: the victim's capture states follow the grabber's `capturedamage.x18`
            cf.victim_gobj = (*v).gobj;
            cf.mv.co.capturedamage.x18 = hold_joint(cf);
            // a boss hold ends by its release or by mashing: the passive per-frame decay of Melee's grab timer
            // (CaptureWait, `grab_timer_decrement`) is given back, so the timer moves with the victim's mash only
            if matches!((*v).motion_id, m if m == ftCo_MS_CaptureWaitHi as i32 || m == ftCo_MS_CaptureWaitLw as i32) {
                (*v).grab_timer += melee_fighter::hooks::common_data().grab_timer_decrement;
            }
        }
    }
    if g.victim >= 0 && g.pending != 0 {
        let v = &mut *fighter_of(g.victim);
        let (vg, cg) = (v.gobj, cf.gobj);
        if g.pending == GRAB_THROW {
            let a = d.attacks.get(g.throw as usize).and_then(|a| a.boxes.first()).copied();
            if let Some(b) = a {
                // Melee's throw release: the throw slot's box (ftAction_80071E04 fields) on the victim, then the launch
                let dmg = gb::num(d, b.damage);
                let h = &mut cf.xDF4[0];
                *h = core::mem::zeroed();
                h.damage = dmg;
                h.unk_count = melee_core::ppc::fctiwz_f32(dmg) as u32;
                h.kb_angle = b.angle;
                h.x24 = melee_core::ppc::fctiwz_f32(gb::num(d, b.kbg)) as u32;
                h.x28 = b.wkb as u32;
                h.x2C = melee_core::ppc::fctiwz_f32(gb::num(d, b.bkb)) as u32;
                h.sfx_severity = 1;
                cf.facing_dir = if v.cur_pos.x >= cf.cur_pos.x { 1.0 } else { -1.0 };
                melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(cg, vg);
                melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0(vg, cg, false);
                eprintln!("geno_grab: f{frame} throw: slot {} with {} ({:.1} dmg, angle {}, kbg {}, bkb {}) -> {:.0}%", g.victim, d.attacks[g.throw as usize].name, dmg, b.angle, h_x24(b, d), h_x2c(b, d), v.dmg.x1830_percent);
            } else {
                melee_fighter::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter(vg);
                eprintln!("geno_grab: f{frame} throw without a box: dropped slot {}", g.victim);
            }
        } else {
            melee_fighter::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter(vg);
            eprintln!("geno_grab: f{frame} release: slot {} dropped (status {})", g.victim, gb::S.status);
        }
        cf.victim_gobj = core::ptr::null_mut();
        g.victim = -1;
        g.pending = 0;
        g.throw = -1;
        return;
    }
    let Some((_part, _joint, r)) = catch_live() else { return };
    let j = hold_joint(cf);
    if j.is_null() {
        return;
    }
    let mut c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, core::ptr::null_mut(), &mut c as *mut Vec3 as _);
    for s in 0..4i32 {
        if s == slot {
            continue;
        }
        let v = fighter_of(s);
        if v.is_null() {
            continue;
        }
        let v = &mut *v;
        if is_captured(v.motion_id) || !v.victim_gobj.is_null() || v.x1988 != 0 || v.x198C != 0 || v.dmg.x195c_hitlag_frames > 0.0 {
            continue;
        }
        let mut hit = false;
        for i in 0..(v.hurt_capsules_len as usize).min(v.hurt_capsules.len()) {
            let h = &v.hurt_capsules[i];
            if h.is_grabbable == 0 {
                continue;
            }
            let k = &h.capsule;
            if seg_point((c.x, c.y), (k.a_pos.x, k.a_pos.y), (k.b_pos.x, k.b_pos.y)) <= r + k.scale {
                hit = true;
                break;
            }
        }
        if !hit {
            continue;
        }
        let (vg, cg) = (v.gobj, cf.gobj);
        cf.victim_gobj = vg;
        cf.x1A5C = vg;
        cf.mv.co.capturedamage.x18 = j;
        melee_fighter::tu::ftCo_CapturePulled::fn_800DAADC(vg, cg);
        melee_fighter::tu::ftCo_CaptureWait::fn_800DB6C8(vg);
        let g = &mut gb::S.grab;
        g.victim = s;
        g.armed = false;
        g.catches += 1;
        eprintln!("geno_grab: f{frame} catch: slot {s} held (status {} motion frame {}, radius {r}, grab timer {:.1})", gb::S.status, gb::S.tl_mf, (*fighter_of(s)).grab_timer);
        return;
    }
}

fn h_x24(b: HitboxDef, d: &Define) -> i32 {
    melee_core::ppc::fctiwz_f32(gb::num(d, b.kbg))
}
fn h_x2c(b: HitboxDef, d: &Define) -> i32 {
    melee_core::ppc::fctiwz_f32(gb::num(d, b.bkb))
}

// ------------------------------------------------------------------------------------------------- Tabuu (VS test)

/// `MELEE_VS_BOSS=tabuu` (fixed for the process).
pub fn vs_selected() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_VS_BOSS").map(|v| v.eq_ignore_ascii_case("tabuu")).unwrap_or(false))
}

/// `MELEE_VS_BOSS_SLOT`: the carrier slot (else the first CPU slot).
pub fn slot_override() -> Option<usize> {
    std::env::var("MELEE_VS_BOSS_SLOT").ok().and_then(|v| v.parse().ok()).filter(|v: &usize| *v < 4)
}

#[derive(Clone, Copy)]
struct Host {
    armed: bool,
    slot: i32,
    seed: u32,
    last_frame: u32,
}
static mut H: Host = Host { armed: false, slot: -1, seed: 1, last_frame: u32::MAX };

pub fn active() -> bool {
    unsafe { H.armed || (H.slot >= 0 && gb::S.live) }
}

pub unsafe fn arm_vs(slot: i32, seed: u32) {
    gb::stop();
    let _ = tabuu();
    melee_fighter::tu::fighter::BOSS_HASH = Some(gb::state_hash);
    H = Host { armed: true, slot, seed: seed | 1, last_frame: u32::MAX };
}

/// Not a Tabuu grab test match: everything off.
pub unsafe fn disarm() {
    if H.slot >= 0 || H.armed {
        gb::stop();
    }
    H = Host { armed: false, slot: -1, seed: 1, last_frame: u32::MAX };
}

/// Per logic frame (frame end, next to `petey::tick`).
pub unsafe fn tick() {
    if !active() {
        return;
    }
    let Some(d) = tabuu() else { return };
    let cf = fighter_of(H.slot);
    if cf.is_null() {
        return;
    }
    let frame = crate::tu_ext::gm_GetFrameCount();
    if frame == H.last_frame {
        return;
    }
    H.last_frame = frame;
    let mut w = World::default();
    for s in 0..4i32 {
        let f = fighter_of(s);
        if s != H.slot && !f.is_null() {
            w.slots[s as usize] = Some((*f).cur_pos.x);
        }
    }
    if H.armed {
        H.armed = false;
        gb::begin(d, w, 5, 1000, H.seed);
        eprintln!("geno_grab: tabuu actor ({} statuses, {} rules, grabs from the IR) on slot {}", d.statuses.len(), d.rules.len(), H.slot);
    }
    // the actor stands where its body is
    gb::S.x = (*cf).cur_pos.x;
    if (*cf).dmg.x195c_hitlag_frames > 0.0 {
        return;
    }
    let st0 = gb::S.status;
    gb::tick(d, w);
    if gb::S.status != st0 && log_on() {
        let name = d.status(gb::S.status as u8).map(|s| s.name.as_str()).unwrap_or("?");
        eprintln!("geno_grab: f{frame} status {st0} -> {} ({name})", gb::S.status);
    }
    for &(k, v) in gb::S.ev.iter().take(gb::S.ev_n as usize) {
        if k == gb::EV_GRAB && log_on() {
            eprintln!("geno_grab: f{frame} event grab {v}");
        }
    }
    host_frame(d, H.slot, frame);
}

pub fn tabuu() -> Option<&'static Define> {
    static D: std::sync::OnceLock<Option<Define>> = std::sync::OnceLock::new();
    D.get_or_init(|| {
        // built on a helper thread: its allocations stay outside the game arena
        let r = std::thread::spawn(|| {
            let dir = crate::tu::geno_boss_ir::ir_dir().ok_or_else(|| "no boss IR directory".to_string())?;
            let text = std::fs::read_to_string(dir.join("tabuu.brawl.ir.json")).map_err(|_| "tabuu.brawl.ir.json: not readable".to_string())?;
            let j = crate::tu::geno_boss_ir::parse_json(&text)?;
            tabuu_define(&j)
        })
        .join()
        .unwrap_or_else(|_| Err("panicked".into()));
        match r {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("geno_grab: no Tabuu define ({e})");
                None
            }
        }
    })
    .as_ref()
}

fn timeline_len(ir: &J, motion: &str) -> Option<i32> {
    ir.get("boss").get("timelines").arr().iter().find(|t| t.get("motion").str() == motion).and_then(|t| t.get("frames").num()).map(|v| v as i32)
}

/// The minimal Tabuu define (VS grab test): Wait, the Chain grab (36 ChainStart -> 37 ChainThrow) and the Dive grab
/// (19 Dive -> 20 DiveTeleport -> 21 DiveSlam), alternating. Catch boxes, hold chains and throws are lowered from the
/// IR by `geno_boss_ir::grab_timeline` / `grab_status_frame`; motion lengths are the IR's timeline lengths. The
/// Wait length and the alternation are the test's own (his rule table is not part of this define).
pub fn tabuu_define(ir: &J) -> Result<Define, String> {
    use crate::tu::geno_boss_ir as gi;
    let mut attacks: Vec<Attack> = vec![];
    let motions_wanted = ["Wait", "ChainStart", "ChainAttack", "DiveStart2", "DiveTeleport", "DiveAttack"];
    let mut motions = vec![];
    for m in motions_wanted {
        let len = timeline_len(ir, m).ok_or_else(|| format!("motion {m}: no length"))?;
        let tl = gi::grab_timeline(ir, m, &mut attacks)?;
        motions.push(Motion { name: m.into(), len, looping: m == "Wait", timeline: tl });
    }
    let mi = |n: &str| motions_wanted.iter().position(|m| *m == n).unwrap() as u8;
    let (st_wait, st_dive, st_tele, st_slam, st_cs, st_ct) = (11u8, 19u8, 20u8, 21u8, 36u8, 37u8);
    let bit = |s: u8| 1u64 << s;
    let name = |ix: u8| ir.get("boss").get("statuses").arr().iter().find(|s| s.get("index").num() == Some(ix as f64)).map(|s| s.get("name").str().to_string()).unwrap_or_default();
    let status = |ix: u8, m: &str, extra: Vec<Op>| -> Result<Status, String> {
        let mut frame = gi::grab_status_frame(ir, ix)?;
        frame.extend(extra);
        Ok(Status { index: ix, name: name(ix), enter: vec![Op::Motion(mi(m))], frame, frame_once: false, intangible: false })
    };
    let mut statuses = vec![
        Status { index: st_wait, name: name(st_wait), enter: vec![Op::Motion(mi("Wait")), Op::SetVar(0, Num::K(0.0))], frame: vec![], frame_once: false, intangible: false },
        status(st_cs, "ChainStart", vec![])?,
        status(st_ct, "ChainAttack", vec![])?,
        status(st_dive, "DiveStart2", vec![])?,
        status(st_tele, "DiveTeleport", vec![])?,
        status(st_slam, "DiveAttack", vec![])?,
    ];
    statuses[0].frame.push(Op::AddVar(0, 1.0));
    let wait = std::env::var("MELEE_TABUU_WAIT").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0f32);
    let only = std::env::var("MELEE_TABUU_GRAB").unwrap_or_default().to_ascii_lowercase();
    let r = |index: u16, cond: Cond, ops: Vec<Op>, to: u8| Rule { index, priority: 0, default_on: true, cond, ops, to: Some(to), per_target: false };
    let and = |a: Cond, b: Cond| Cond::And(Box::new(a), Box::new(b));
    let waited = and(Cond::StatusIn(bit(st_wait)), Cond::Var(0, gb::Cmp::Ge, wait));
    let chain_next = match only.as_str() {
        "chain" => Cond::Always,
        "dive" => Cond::Never,
        _ => Cond::Var(1, gb::Cmp::Eq, 0.0),
    };
    let rules = vec![
        // the Wait: alternate Chain / Dive (var 1), or one of them (MELEE_TABUU_GRAB)
        r(0, and(waited.clone(), chain_next.clone()), vec![Op::SetVar(1, Num::K(1.0))], st_cs),
        r(1, and(waited, Cond::Not(Box::new(chain_next))), vec![Op::SetVar(1, Num::K(0.0))], st_dive),
        // the Chain: held at ChainStart's end -> ChainThrow (the hold continues: 36's exit keeps it for 37)
        r(2, and(Cond::StatusIn(bit(st_cs)), and(Cond::Holding, Cond::MotionEnd)), vec![], st_ct),
        // Dive's per-frame "if caught -> DiveTeleport" is in the status's frame ops (lowered from the IR)
        r(3, and(Cond::StatusIn(bit(st_tele)), Cond::MotionEnd), vec![], st_slam),
        r(4, and(Cond::StatusIn(bit(st_cs) | bit(st_ct) | bit(st_dive) | bit(st_slam)), Cond::MotionEnd), vec![], st_wait),
    ];
    let parts = vec![Part { name: "body".into(), role: Role::Core, hp: Num::K(1000.0), on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: false }];
    let lv = ir.get("boss").get("level");
    let tab = lv.get("tables").arr().iter().find(|t| t.get("selected_when").str() == "default").or(lv.get("tables").arr().first());
    let level_rows: Vec<Vec<f32>> = tab.map(|t| t.get("rows").arr().iter().map(|r| r.arr().iter().map(|x| x.num().unwrap_or(0.0) as f32).collect()).collect()).unwrap_or_default();
    Ok(Define {
        name: "tabuu-grab".into(),
        parts,
        vars: vec![("waitFrames".into(), 0.0), ("chainNext".into(), 0.0)],
        statuses,
        rules,
        tables: vec![],
        kinetics: vec![],
        motions,
        attacks,
        messages: vec![],
        level_rows,
        strength0: lv.get("initial").get("strength").num().unwrap_or(1.0) as u8,
        start_status: st_wait,
        slot_w: [1.0; 4],
        slot_repeat: 1.0,
        route_mask: 0,
        route_factor: 1.0,
        defeat_status: [None, None],
        goal_snap: false,
        spawn_x: 0.0,
        camera: None,
        window: (-50, 250),
        passes: vec![(-50, 250, false)],
    })
}

// keep the unused-import lint quiet for the define types used only in signatures
#[allow(dead_code)]
fn _types(_: TimelineEvent) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_status_define(release: Op) -> Define {
        let st = |ix: u8, ops: Vec<Op>| Status { index: ix, name: format!("s{ix}"), enter: ops, frame: vec![], frame_once: false, intangible: false };
        Define {
            name: "grab".into(),
            parts: vec![Part { name: "core".into(), role: Role::Core, hp: Num::K(10.0), on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: false }],
            vars: vec![],
            statuses: vec![
                st(0, vec![Op::Motion(0), Op::Grab { part: 0, joint: 1, radius: 10.0, frames: (2, 4), release: 0b11 }]),
                st(1, vec![Op::Motion(0), release]),
                st(2, vec![Op::Motion(0)]),
            ],
            rules: vec![
                Rule { index: 0, priority: 0, default_on: true, cond: Cond::And(Box::new(Cond::StatusIn(1)), Box::new(Cond::Holding)), ops: vec![], to: Some(1), per_target: false },
                Rule { index: 1, priority: 0, default_on: true, cond: Cond::And(Box::new(Cond::StatusIn(2)), Box::new(Cond::MotionEnd)), ops: vec![], to: Some(2), per_target: false },
            ],
            tables: vec![],
            kinetics: vec![],
            motions: vec![Motion { name: "m".into(), len: 8, looping: false, timeline: vec![] }],
            attacks: vec![Attack { name: "t".into(), boxes: vec![HitboxDef { part: 0, joint: 0, offset: [0.0; 3], frames: (0, 0), damage: Num::Lin(30.0, 2.8), angle: 22, kbg: Num::K(75.0), wkb: 0.0, bkb: Num::K(60.0), size: Num::K(1.0), group: 0, rehit: 0 }] }],
            messages: vec![],
            level_rows: vec![vec![1.0; 15]],
            strength0: 1,
            start_status: 0,
            slot_w: [1.0; 4],
            slot_repeat: 1.0,
            route_mask: 0,
            route_factor: 1.0,
            defeat_status: [None, None],
            goal_snap: false,
            spawn_x: 0.0,
            camera: None,
            window: (-50, 250),
            passes: vec![(-50, 250, false)],
        }
    }

    #[test]
    fn catch_window_hold_and_release() {
        let d = one_status_define(Op::GrabRelease(Some(0)));
        let w = World::default();
        gb::begin(&d, w, 5, 1000, 3);
        let mut live = vec![];
        for _ in 0..6 {
            live.push(catch_live().is_some());
            gb::tick(&d, w);
        }
        assert_eq!(live, [false, false, true, true, true, false]);
        // not caught: the status's motion ends and nothing is held
        gb::begin(&d, w, 5, 1000, 3);
        gb::tick(&d, w);
        gb::tick(&d, w);
        assert!(catch_live().is_some());
        unsafe {
            gb::S.grab.victim = 1; // the host caught slot 1
            gb::S.grab.armed = false;
        }
        assert!(gb::state_hash().is_some());
        gb::tick(&d, w); // Holding -> status 1 (in the hold mask), whose enter throws
        unsafe {
            assert_eq!(gb::S.status, 1);
            assert_eq!(gb::S.grab.pending, GRAB_THROW);
            assert_eq!(gb::S.grab.throw, 0);
        }
    }

    #[test]
    fn leaving_the_hold_drops() {
        let mut g = OFF;
        arm(&mut g, 0, 1, 5.0, (0, 9), 1 << 3 | 1 << 4, 3);
        g.victim = 0;
        assert!(!on_status(&mut g, 4));
        assert_eq!(g.pending, 0);
        assert!(on_status(&mut g, 5));
        assert_eq!(g.pending, GRAB_DROP);
        let h0 = hash_words(&g);
        g.pending = 0;
        assert_ne!(h0, hash_words(&g));
    }

    /// The local IR, when present: the grab define lowers (skipped otherwise).
    #[test]
    fn lowers_local_tabuu_grabs() {
        let Some(dir) = crate::tu::geno_boss_ir::ir_dir() else { return };
        let Ok(text) = std::fs::read_to_string(dir.join("tabuu.brawl.ir.json")) else { return };
        let j = crate::tu::geno_boss_ir::parse_json(&text).unwrap();
        let d = tabuu_define(&j).unwrap();
        let grabs: Vec<&Op> = d.motions.iter().flat_map(|m| m.timeline.iter().map(|e| &e.op)).filter(|o| matches!(o, Op::Grab { .. })).collect();
        assert_eq!(grabs.len(), 2);
        for g in &grabs {
            eprintln!("grab: {g:?}");
        }
        assert!(d.attacks.len() >= 2);
        for a in &d.attacks {
            eprintln!("throw: {} {:?}", a.name, a.boxes.first());
        }
    }
}
