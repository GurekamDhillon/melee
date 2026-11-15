//! Petey Piranha boss for Envoy fight 5 (notes/petey-boss.md, notes/petey-behavior.md).
//!
//! Opt-in: the Envoy plan marks the boss fight (`envoy::P_BOSS`) and `MELEE_ENVOY_BOSS=petey` is set; otherwise the
//! level-9 Ganondorf stand-in plays as before and nothing here runs.
//!
//! Petey and every other VS / Envoy boss run as Geno actors (geno_boss, lowered at load from the owner's local IR),
//! as enemy-command VM actors (geno_vm_host) or as grab actors (geno_boss_grab). No boss numbers are typed in here:
//! statuses, rules, timings, boxes, sounds and the camera come from the IR; shapes from the owner's model files.
//!
//! Shape:
//! * the match's one CPU is a *carrier*: invisible, intangible, standing still. Petey's hitboxes are the carrier's
//!   capsule slots (the engine's normal hit path); when Petey is beaten the carrier takes the bottom-blast-zone KO so
//!   Envoy's win check runs.
//! * The host is plain statics (rollback snapshot covered), no wall clock, no host input.
//! * Hurt and hit shapes follow the animated bones when the owner's local model is present (`tracks`, built at load
//!   from the files, kept in memory only); without it a stand-in set of shapes is used. The two can give different
//!   sims, so a replay recorded with the model present must be played with it present.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_fighter::tu::player as pl;
use melee_types::*;
use melee_core::ppc::{fctiwz_f32, Ppc};

// console-arithmetic helpers (notes/semantics.md): plain comparisons, explicit conversions
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn fmax(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
fn fclamp(v: f32, lo: f32, hi: f32) -> f32 {
    fmax(fmin(v, hi), lo)
}
/// Round half away from zero through the explicit truncation.
fn round_i(v: f32) -> i32 {
    fctiwz_f32(if v < 0.0 { v - 0.5 } else { v + 0.5 })
}

// ---------------------------------------------------------------- difficulty plumbing (Envoy)

/// The difficulty index a fresh Normal game uses (0-14), and Envoy's step per endless loop (one tier).
const DIFFICULTY: usize = 5;
const DIFFICULTY_PER_LOOP: i32 = 3;
/// The plan's boss-HP word grows by this much per loop (envoy_sim `LOOP_BOSS_HP`): the loop number is read back from it.
const PLAN_HP_PER_LOOP: i32 = 250;

/// No boss running.
pub const ST_OFF: i32 = -1;

#[derive(Clone, Copy)]
pub struct Boss {
    pub st: i32,
    pub prev_st: i32,
    /// frames since entering the status (0, 1, 2, ...)
    pub sf: i32,
    /// status timeout T
    pub t_out: f32,
    pub mot: i32,
    /// motion frame
    pub mf: i32,
    pub px: f32,
    pub vx: f32,
    pub cage: [i32; 2],
    pub phase: i32,
    /// cage work counter (0 / 1 / 2) and the phase-up rules the cages enabled
    pub cage_ctr: i32,
    pub rule28: bool,
    pub rule29: bool,
    pub jump_count: i32,
    pub jumps_wanted: i32,
    pub side: i32,
    /// which status the flinch returns to (rule 19 -> 4, rule 20 -> 5)
    pub flinch_to: i32,
    /// dive direction (+1 / -1)
    pub aim: f32,
    pub cage_broken: bool,
    /// 0 none, 1 death by the cage threshold, 2 cage broken
    pub cause: i32,
    /// the freed side (0 left cage / Peach, 1 right cage / Zelda)
    pub freed: i32,
    pub ko_done: bool,
    pub down_t: i32,
    pub rng: u32,
    pub carrier_slot: i32,
    pub last_frame: u32,
    /// bumps on every status entry and motion start: hit groups key on it
    pub inst: u32,
    pub flash: [i32; 3],
    /// last player slot targeted (repeat weighting)
    pub last_slot: i32,
    /// the locked target slot (-1 none): rule 13 locks once per jump; landing and the menu clear it
    pub locked: i32,
    /// the last attack-menu pick (`menuPick`, for the 0.3 repeat cut; the file's init leaves it 0)
    pub menu_pick: i32,
    /// absolute world goal x set by rule 13 (spec §7.2); `has_goal` false = unset
    pub has_goal: bool,
    pub goal_x: f32,
    /// Envoy endless loops: each cage's full HP this fight (CAGE_HP x the plan's P_BOSS_HP, set as the fight begins)
    pub cage_max: i32,
    /// the carrier's hitlag last frame (a rise = one of Petey's boxes connected this frame)
    pub lag_prev: f32,
    /// the adventure follow camera (spec §2.3 / §10, Q6): started, centre x / y and eye distance
    pub cam_on: bool,
    pub cam: [f32; 3],
    /// the difficulty index (0-14) this fight uses for the L table
    pub diff: i32,
    /// the last motion frame whose timeline events (shakes, sounds) have fired (-1 = none yet)
    pub tl_mf: i32,
    /// sound events for the presentation side: a running count and the last eight Brawl sound ids
    pub snd_seq: u32,
    pub snd: [u16; 8],
    /// frames ticked since the fight began (the test hits key on it)
    pub age: u32,
}

const OFF: Boss = Boss {
    st: ST_OFF, prev_st: ST_OFF, sf: 0, t_out: 0.0, mot: 0, mf: 0, px: 0.0, vx: 0.0, cage: [0; 2],
    phase: 0, cage_ctr: 0, rule28: false, rule29: false, jump_count: 1, jumps_wanted: 1, side: 0, flinch_to: ST_OFF,
    aim: 1.0, cage_broken: false, cause: 0, freed: -1, ko_done: false, down_t: 0, rng: 1, carrier_slot: -1,
    last_frame: u32::MAX, inst: 1, flash: [0; 3], last_slot: -1, locked: -1, menu_pick: 0, has_goal: false, goal_x: 0.0, cage_max: 0, lag_prev: 0.0,
    cam_on: false, cam: [0.0; 3], diff: DIFFICULTY as i32, tl_mf: -1, snd_seq: 0, snd: [0; 8], age: 0,
};

/// Each cage's full HP this fight (the define's, Envoy's loop scaling applied).
fn cage_max() -> i32 {
    unsafe { B.cage_max }.max(1)
}
pub static mut B: Boss = OFF;
/// The match about to start is a Petey fight (set by `envoy::match_begin`, consumed at the first tick).
pub static mut ARMED: bool = false;
/// Per player slot, entity, hitbox: targets already hit by that live hitbox (bit 0 body, 1 cage L, 2 cage R).
static mut LATCH: [[[u8; 4]; 2]; 4] = [[[0; 4]; 2]; 4];
/// Our copies of the carrier's capsule slots (put back if the carrier's own state change cleared them) and the hit
/// group each slot belongs to.
static mut SAVED: [HitCapsule; 4] = unsafe { core::mem::zeroed() };
static mut SLOT_INST: [u32; 4] = [0; 4];
/// Frames left of the knockback log after a hit (MELEE_PETEY_LOG only).
static mut KB_LOG: i32 = 0;
/// Item hitbox latches: (item gobj, per hitbox target bits).
static mut LATCH_IT: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];

/// Is Petey selected for boss fights (`MELEE_ENVOY_BOSS=petey`; fixed for the process).
pub fn selected() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_ENVOY_BOSS").map(|v| v.eq_ignore_ascii_case("petey")).unwrap_or(false))
}

/// VS boss: `MELEE_VS_BOSS=petey` (fixed for the process) puts Petey on the first CPU slot of a VS match.
pub fn vs_selected() -> bool {
    vs_boss().is_some()
}

/// `MELEE_VS_BOSS=<name>`: the boss IR `<name>.brawl.ir.json` (any boss; Petey keeps his model and arena, the others
/// run on stand-in shapes on Final Destination). Lower-case, fixed for the process.
pub fn vs_boss() -> Option<&'static str> {
    static V: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    V.get_or_init(|| std::env::var("MELEE_VS_BOSS").ok().map(|v| v.trim().to_ascii_lowercase()).filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))).as_deref()
}

/// The boss this process runs: the VS boss, else Envoy's (Petey).
pub fn boss_name() -> &'static str {
    vs_boss().unwrap_or("petey")
}

/// Petey's own model, arena, sounds and stand-in shapes apply.
pub fn is_petey() -> bool {
    boss_name() == "petey"
}

pub fn active() -> bool {
    unsafe { B.st != ST_OFF || ARMED }
}

/// The partner used to sit "in a cage" for the whole boss fight (no tags until a cage broke). The owner wants to
/// switch characters against Petey like in every other fight, so the partner is never caged now; kept as the one
/// switch point (tag_swap::partner_kind) should a caged-partner rule come back.
pub fn partner_caged() -> bool {
    false
}

/// Called by `envoy::match_begin` for the boss fight. The player count does not scale anything (Brawl has no such
/// rule); it is kept in the signature for the caller.
pub unsafe fn arm(_humans: i32, carrier_slot: i32, seed: u32) {
    let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || melee_core::frameprof::Text::new().s("boss: arm"));
    B = OFF;
    crate::tu::geno_boss::stop();
    let _ = geno();
    melee_fighter::tu::fighter::BOSS_HASH = Some(crate::tu::geno_boss::state_hash);
    B.carrier_slot = carrier_slot;
    HITS = [0; 4];
    DMG = [0; 4];
    B.rng = seed | 1;
    ARMED = true;
    LATCH = [[[0; 4]; 2]; 4];
    LATCH_IT = [(0, [0; 4]); 16];
    SLOT_INST = [0; 4];
    let _ = tracks(); // load (or miss) the local model before frame 0
}

/// Hits each player slot landed on Petey or a cage (Envoy's drives charge on them: his hits never reach the
/// take-damage hook, the carrier being intangible).
pub static mut HITS: [u32; 4] = [0; 4];
/// Envoy V3 (`gd_envoy_boss_dmg`): the damage each player slot dealt Petey or a cage, x1000, a running total that
/// wraps (the same `dmg` the hit test charges the cages with).
pub static mut DMG: [u32; 4] = [0; 4];

/// The carrier's slot while a Petey fight runs (-1 otherwise): Envoy's drives treat it as the boss.
pub fn carrier() -> i32 {
    unsafe { if B.st != ST_OFF { B.carrier_slot } else { -1 } }
}

/// Petey's position for Envoy's drives (the body centre).
pub fn pos() -> (f32, f32) {
    unsafe { (B.px, 18.0) }
}

/// `damage(amount)` (envoy-core-v2.md 2.6): a drive's burst or Burning tick on the boss, whose HP is separate from
/// percent. Body damage, as a player hit: both cages lose it, then the cage and death rules run as on a hit frame.
pub unsafe fn damage(amount: f32) {
    if B.st == ST_OFF || B.cause != 0 || B.ko_done || amount <= 0.0 {
        return;
    }
    let d = round_i(amount).max(1);
    B.flash[0] = 8;
    if vm().is_some() {
        VM_DMG += d;
    } else if let Some(g) = geno() {
        crate::tu::geno_boss::damage(g, 0, d);
    }
}

/// Not a Petey match: everything off.
pub unsafe fn disarm() {
    crate::tu::geno_boss::stop();
    B = OFF;
    ARMED = false;
}

/// The difficulty scalar L (the define's, or the VM actor's).
pub fn level() -> f32 {
    if vm().is_some() {
        return crate::tu::geno_vm::actor().level();
    }
    geno().map(crate::tu::geno_boss::level).unwrap_or(0.0)
}

/// The pinned difficulty index (`MELEE_PETEY_DIFFICULTY`), read once per process.
fn difficulty_pin() -> Option<i32> {
    static V: std::sync::OnceLock<Option<i32>> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_PETEY_DIFFICULTY").ok().and_then(|v| v.trim().parse::<i32>().ok()).map(|v| v.clamp(0, 14)))
}

/// `MELEE_PETEY_LOG` (fixed for the process).
fn log_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_PETEY_LOG").is_some())
}

/// Test knob (scripted checks; fixed for the process, so a run stays deterministic):
/// `MELEE_PETEY_TEST_HITS="<fight frame>:<b|l|r>:<damage>,..."` lands a player hit of that damage on the body (b), the
/// left cage (l) or the right cage (r) at that frame of the fight, as a fighter's hitbox would.
fn test_hits() -> &'static [(u32, u8, i32); 32] {
    static V: std::sync::OnceLock<[(u32, u8, i32); 32]> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        let mut out = [(u32::MAX, 0u8, 0i32); 32];
        if let Ok(v) = std::env::var("MELEE_PETEY_TEST_HITS") {
            for (i, item) in v.split(',').take(32).enumerate() {
                let mut it = item.trim().split(':');
                let f = it.next().and_then(|x| x.parse::<u32>().ok());
                let t = it.next().and_then(|x| x.bytes().next());
                let d = it.next().and_then(|x| x.parse::<i32>().ok());
                if let (Some(f), Some(t), Some(d)) = (f, t, d) {
                    out[i] = (f, t, d);
                }
            }
        }
        out
    })
}

/// A sound event for the presentation side (a Brawl sound id; `sfx_drain` hands them to the audio host).
unsafe fn push_snd(id: u16, f: i32) {
    if log_on() {
        eprintln!("petey: f{} sound {} (motion {} frame {})", B.last_frame, id, B.mot, f);
    }
    B.snd[(B.snd_seq & 7) as usize] = id;
    B.snd_seq = B.snd_seq.wrapping_add(1);
}

/// Presentation only: the Brawl sound ids queued since the last call (at most eight; a rollback or a new fight that
/// moves the count backwards plays nothing). Never feeds the sim.
pub fn sfx_drain(out: &mut [u16; 8]) -> usize {
    use std::sync::atomic::{AtomicU32, Ordering};
    static SEEN: AtomicU32 = AtomicU32::new(0);
    let (seq, ring) = unsafe { (B.snd_seq, B.snd) };
    let seen = SEEN.swap(seq, Ordering::Relaxed);
    if seq <= seen {
        return 0;
    }
    let n = (seq - seen).min(8);
    for i in 0..n {
        out[i as usize] = ring[((seq - n + i) & 7) as usize];
    }
    n as usize
}

unsafe fn fighter_of(slot: i32, i: usize) -> *mut Fighter {
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let p = pl::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[i];
    if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

fn cage_pct(i: usize) -> f32 {
    100.0 * unsafe { B.cage[i] }.max(0) as f32 / cage_max() as f32
}

/// Bumped by every [`tick`] of a live Petey match (presentation: the host plays Brawl's boss music while it moves).
pub static HEARTBEAT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// The Brawl stream sound Petey's fight plays (Brawl's own boss battle theme), by its name in the sound archive.
pub const BOSS_MUSIC: &str = "snd_bgm_X18_BOSSBATTLE";

/// Per logic frame, after the post chain (next to `tag_swap::tick`).
pub unsafe fn tick() {
    if !active() {
        return;
    }
    HEARTBEAT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let carrier = fighter_of(B.carrier_slot, 0);
    if carrier.is_null() {
        return;
    }
    let frame = crate::tu_ext::gm_GetFrameCount();
    if frame == B.last_frame {
        return; // frozen / paused: Petey waits too
    }
    B.last_frame = frame;
    if ARMED {
        let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || melee_core::frameprof::Text::new().s("boss: first frame"));
        ARMED = false;
        // in his arena: the arena's enemy spawn point (its GEG1 record); on Final Destination the centre
        B.px = if crate::tu::explore::ARENA {
            crate::tu::explore::arena().and_then(|m| m.spawns.first()).map(|s| s.1).unwrap_or(0.0)
        } else {
            0.0
        };
        // Envoy endless loops: the plan's boss-HP word (0 / no plan: retail's)
        let hp = crate::tu::envoy::PLAN[crate::tu::envoy::P_BOSS_HP];
        // the difficulty index: Normal, one Brawl tier up per Envoy loop (read back from the plan's HP word), or pinned
        let loops = if hp > 1000 { (hp.clamp(1000, 20_000) - 1000) / PLAN_HP_PER_LOOP } else { 0 };
        B.diff = difficulty_pin().unwrap_or((DIFFICULTY as i32 + DIFFICULTY_PER_LOOP * loops).clamp(0, 14));
        if let Some(p) = vm() {
            // the enemy-command VM: the scripts from the player's files; the IR's start status, L from strength 1
            let st = std::env::var("MELEE_VS_BOSS_STATUS").ok().and_then(|v| v.parse().ok())
                .or_else(vm_start_status)
                .unwrap_or(2);
            let p = vm_arena(p);
            crate::tu::geno_vm_host::reset();
            crate::tu::geno_vm::begin(p, 1, B.diff, B.rng, st, B.px);
            B.cage = [0; 2];
            B.cage_max = 0;
            B.st = crate::tu::geno_vm::actor().status;
            eprintln!("petey: vm actor {} (status {}, hp {:.1}, L {:.2})", boss_name(), st, crate::tu::geno_vm::actor().hp(), crate::tu::geno_vm::actor().level());
        } else if let Some(g) = geno() {
            // the Geno actor: the define's own start status, HP and init; the plan scales the weak points' HP
            // cage HP is trunc(base + slope L) in the define: Envoy's loops scale it through the difficulty (B.diff)
            let scale = 1000;
            let px = B.px;
            crate::tu::geno_boss::begin(g, world(), B.diff, scale, B.rng);
            if grab_actor().is_some() {
                crate::tu::geno_boss::S.x = (*carrier).cur_pos.x; // the actor starts where its body stands
            }
            B.cage_max = if g.parts.len() > 1 { crate::tu::geno_boss::S.hp_max[1] } else { crate::tu::geno_boss::S.hp_max[0] };
            if crate::tu::explore::ARENA {
                crate::tu::geno_boss::S.x = px;
            }
            mirror(g);
            eprintln!("petey: geno actor ({} statuses, {} rules from the IR)", g.statuses.len(), g.rules.len());
            if crate::tu::geno_boss_grab::has_grabs(g) {
                eprintln!("geno_grab: {} actor ({} statuses, {} rules, grabs from the IR) on slot {}", boss_name(), g.statuses.len(), g.rules.len(), B.carrier_slot);
            }
        } else {
            // no define and no VM program (the owner's files absent): nothing to run; envoy / the VS setup checked
            eprintln!("petey: {} has no Geno define or VM program; boss off", boss_name());
            B = OFF;
            return;
        }
        eprintln!("petey: fight begins, cages {} difficulty {} L {:.1}", B.cage_max, B.diff, level());
        eprintln!("petey: stage {}", if crate::tu::explore::ARENA { "his arena" } else { "Final Destination" });
        match is_petey().then(crate::tu::petey_model::get).flatten().and_then(|m| m.arena_blast) {
            Some(z) => eprintln!("petey: arena blast zones {:.0} {:.0} {:.0} {:.0}, knockback x{}", z[0], z[1], z[2], z[3], kb_scale_pm()),
            None => eprintln!("petey: no arena blast zones (stage's own), knockback x{}", kb_scale_pm()),
        }
        melee_world::tu::ifstatus::ifStatus_802F6788(B.carrier_slot as u8);
    }
    hide_marker(B.carrier_slot);
    // the arena's own blast zones, read from the player's Brawl files at run time (the boss archive's stage position
    // model): the fight runs on Final Destination, whose closer zones turned Brawl's launch speeds into one-shots.
    // Applied every frame of the fight (idempotent, rollback-safe); the next stage load restores FD's. Without the
    // files FD's zones stay and the knockback falls back to x0.8 (kb_scale_pm).
    if let Some(z) = is_petey().then(crate::tu::petey_model::get).flatten().and_then(|m| m.arena_blast) {
        let bz = &mut *core::ptr::addr_of_mut!(crate::tu::script_game::stage_info.blast_zone);
        bz.left = z[0];
        bz.right = z[1];
        bz.top = z[2];
        bz.bottom = z[3];
    }
    let cf = &mut *carrier;
    if !B.ko_done {
        cf.set_invisible(1);
        cf.x1988 = 2; // intangible
        // the CPUs (a CPU ally, the level-9 test players) aim at the carrier's position: keep it on the floor under
        // Petey's body, not at its parked spawn at the stage's edge (the camera already frames Petey: camera_box).
        // Game state in the arena (rolled back with the fighter); Petey's own targeting skips the carrier.
        // a grab actor's body is the carrier itself (geno_boss_grab::body_before / body_after move it)
        if cf.ground_or_air == GA_Ground && grab_actor().is_none() {
            cf.cur_pos.x = B.px;
        }
    }
    for f in B.flash.iter_mut() {
        *f = (*f - 1).max(0);
    }
    // hits on the boss this frame (the define routes body / weak-point damage)
    if B.cause == 0 {
        player_hits(B.carrier_slot);
    }
    B.age = B.age.wrapping_add(1);
    if B.cause == 0 {
        for &(f, t, d) in test_hits().iter() {
            if f == B.age {
                if vm().is_some() {
                    VM_DMG += d;
                } else if let Some(g) = geno() {
                    crate::tu::geno_boss::damage(g, match t { b'l' => 1, b'r' => 2, _ => 0 }, d);
                }
                eprintln!("petey: f{} test hit {} {}", B.last_frame, t as char, d);
            }
        }
    }
    // hitlag: when one of the boss's boxes connects the carrier (the attacker) gets Melee's hitlag; the actor's motion
    // and movement freeze with it, as in Brawl
    let lag = cf.dmg.x195c_hitlag_frames;
    if lag > B.lag_prev && std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{} hit lands: motion {} frame {} attacker hitlag {}", B.last_frame, B.mot, B.mf, lag);
    }
    if std::env::var_os("MELEE_PETEY_LOG").is_some() && (B.lag_prev > 0.0 || KB_LOG > 0) {
        // the victims' first frames of flight (launch angle / speed), for the Brawl comparison
        KB_LOG = if B.lag_prev > 0.0 && cf.dmg.x195c_hitlag_frames <= 0.0 { 150 } else { KB_LOG - 1 };
        if KB_LOG > 0 {
            for s in 0..4 {
                let v = fighter_of(s, 0);
                if s != B.carrier_slot && !v.is_null() {
                    eprintln!("petey: f{} victim {} pos {:.2} {:.2} pct {}", B.last_frame, s, (*v).cur_pos.x, (*v).cur_pos.y, (*v).dmg.x1830_percent);
                }
            }
        }
    }
    B.lag_prev = cf.dmg.x195c_hitlag_frames;
    if cf.dmg.x195c_hitlag_frames <= 0.0 || B.cause != 0 {
        if let Some(p) = vm() {
            vm_frame(vm_run(p));
        } else if let Some(g) = geno() {
            let grab = grab_actor().is_some();
            let driving = grab && crate::tu::geno_boss_grab::body_before(cf);
            geno_frame(g);
            if grab {
                crate::tu::geno_boss_grab::body_after(cf, driving, B.last_frame);
            }
            // grabs (geno_boss_grab): escapes, releases, then a catch, on Melee's capture path with the carrier as grabber
            crate::tu::geno_boss_grab::host_frame(g, B.carrier_slot, B.last_frame);
        }
    }
    hitboxes(cf);
    follow_camera();
    if B.cause != 0 {
        B.down_t += 1;
        if !B.ko_done && B.down_t >= down_end() {
            B.ko_done = true;
            disable_all(cf);
            cf.x1988 = 0;
            melee_fighter::tu::ft_0d31::ftCo_800D3BC8(cf.gobj);
            eprintln!("petey: f{} defeated (cause {}, freed side {}), carrier KO", B.last_frame, B.cause, B.freed);
        }
    }
}

/// The boss's stage camera from the define's camera block (Petey's IR: the adventure follow camera): it follows the
/// player only, at the block's eye distance approached by its ease per frame, its tilt and FOV, keeping the visible
/// rectangle inside the block's limit box (the arena's own in his arena). The centre eases in too. Width uses the
/// camera's authored aspect (not the window), so the logic is the same on every machine. Driven through Melee's
/// scripted (boss intro) mode with fixed interest and eye. State is in `B` and `game_camera` (statics): rollback-safe.
/// A define without a camera block keeps Melee's own camera.
unsafe fn follow_camera() {
    let Some(cd) = geno().and_then(|g| g.camera) else { return };
    use melee_world::tu::camera as cm;
    // the player: the first fighter that is not the carrier
    let mut tgt = None;
    for s in 0..4 {
        if s == B.carrier_slot {
            continue;
        }
        let f = fighter_of(s, 0);
        if !f.is_null() && !(*f).x890_cameraBox.is_null() {
            let p = (*(*f).x890_cameraBox).pos;
            tgt = Some((p.x, p.y));
            break;
        }
    }
    let gc = &mut *core::ptr::addr_of_mut!(cm::game_camera);
    if !B.cam_on {
        let (i, e) = (gc.transform.interest, gc.transform.position);
        let (dx, dy, dz) = (e.x - i.x, e.y - i.y, e.z - i.z);
        let d = melee_runtime::gekko::sqrtf(dx * dx + dy * dy + dz * dz);
        B.cam = [i.x, i.y, if d > 1.0 { d } else { cd.distance }];
        B.cam_on = true;
    }
    if gc.mode != CAMERA_BOSS_INTRO as u32 {
        cm::Camera_8002FE38();
    }
    // MELEE_PETEY_CAM=boss (captures only): frame Petey instead of the player, so a recording keeps him in view
    static BOSS_CAM: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *BOSS_CAM.get_or_init(|| std::env::var("MELEE_PETEY_CAM").map(|v| v == "boss").unwrap_or(false)) {
        tgt = Some((B.px, hurt()[0].map_or(0.0, |h| h[1])));
    }
    let (tx, ty) = tgt.unwrap_or((B.cam[0], B.cam[1]));
    B.cam[2] += (cd.distance - B.cam[2]) * cd.ease;
    B.cam[0] += (tx - B.cam[0]) * cd.ease;
    B.cam[1] += (ty - B.cam[1]) * cd.ease;
    let hh = B.cam[2] * melee_runtime::gekko::tanf(0.5 * cd.fov * 0.017453292);
    let hw = hh * cm::cm_803BCB64.aspect;
    let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { 0.5 * (lo + hi) } else { fmin(fmax(v, lo), hi) };
    // in his arena the camera limit is the arena's own (stage position model, CamLimit0N / CamLimit1N)
    let lim = if crate::tu::explore::ARENA {
        crate::tu::explore::arena().and_then(|m| m.cam).map(|c| [c[0], c[1], c[3], c[2]]).unwrap_or(cd.limits)
    } else {
        cd.limits
    };
    B.cam[0] = clamp(B.cam[0], lim[0] + hw, lim[1] - hw);
    B.cam[1] = clamp(B.cam[1], lim[2] + hh, lim[3] - hh);
    let t = cd.tilt * 0.017453292;
    let (ts, tc) = (melee_runtime::gekko::sinf(t), melee_runtime::gekko::cosf(t));
    let interest = Vec3 { x: B.cam[0], y: B.cam[1], z: 0.0 };
    let eye = Vec3 { x: B.cam[0], y: B.cam[1] - B.cam[2] * ts, z: B.cam[2] * tc };
    gc.set_x341_b1_b2(2);
    gc.set_x341_b3_b4(1);
    gc.set_x341_b5_b6(0);
    gc.x344.vec = interest;
    gc.x35C.vec = eye;
    gc.x350 = interest;
    gc.x368.vec = eye;
    gc.x374 = cd.fov;
    gc.transform.target_interest = interest;
    gc.transform.interest = interest;
    gc.transform.target_position = eye;
    gc.transform.position = eye;
    gc.transform.target_fov = cd.fov;
    gc.transform.fov = cd.fov;
    // the scripted mode's proc does not run the quakes: step them here (the quake gobjs write the offset, this turns
    // it into the render translation and counts the kinds down)
    let mut qb = CameraBounds::zeroed();
    qb.z_pos = B.cam[2];
    cm::Camera_UpdateQuakes(&mut qb);
    cm::Camera_ApplyQuake(&mut qb, core::ptr::addr_of_mut!(gc.transform) as *mut CameraTransformState);
    if log_on() && (gc.translation.x != 0.0 || gc.translation.y != 0.0) {
        eprintln!("petey: f{} quake offset {:.3} {:.3}", B.last_frame, gc.translation.x, gc.translation.y);
    }
    if B.last_frame % 120 == 0 && std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{} camera centre ({:.1}, {:.1}) dist {:.1} mode {} player {:?}", B.last_frame, B.cam[0], B.cam[1], B.cam[2], gc.mode, tgt);
    }
}

/// Hide the carrier's "CP" pointer: its proc shows it again each frame, this runs after the procs, before the draw.
unsafe fn hide_marker(slot: i32) {
    let g = melee_world::tu::ifnametag::un_804A1EE0[slot as usize];
    if g.is_null() {
        return;
    }
    let j = (*g).hsd_obj as *mut HSD_JObj;
    if !j.is_null() && !(*j).child.is_null() {
        melee_runtime::tu::jobj_world::HSD_JObjSetFlags((*j).child as _, JOBJ_HIDDEN as u32);
    }
}

/// Hurt points per motion frame (model space): body, left cage, right cage, each a capsule from the model's parameter
/// hurtbox (its bone, offset and stretch end).
const P_BODY_A: usize = 0;
const P_BODY_B: usize = 1;
const P_CAGE_L_A: usize = 2;
const P_CAGE_L_B: usize = 3;
const P_CAGE_R_A: usize = 4;
const P_CAGE_R_B: usize = 5;
const NP: usize = 6;

/// Model scale Brawl units -> ours (open question Q1).
pub const MODEL_SCALE: f32 = 1.0;

/// Per define motion, per whole frame: the hurt points in model space, and the hurt radii (body, cage, cage). Built
/// once from the owner's local files (derived numbers, in memory only, never written).
pub struct Tracks {
    pub mots: std::vec::Vec<std::vec::Vec<[[f32; 3]; NP]>>,
    pub radii: [f32; 3],
    pub zc: f32,
    /// the Geno define's hitboxes: keys (attack, box, owning part) and, per motion and frame, their points
    pub gkeys: std::vec::Vec<(u8, u8, u8)>,
    pub gpts: std::vec::Vec<std::vec::Vec<std::vec::Vec<[f32; 3]>>>,
}

pub fn tracks() -> Option<&'static Tracks> {
    static T: std::sync::OnceLock<Option<Tracks>> = std::sync::OnceLock::new();
    // built on a helper thread, as the model is: the first call comes inside a game frame, where allocations land in
    // the game arena that rollback snapshots rewrite
    T.get_or_init(|| std::thread::spawn(build_tracks).join().ok().flatten()).as_ref()
}

fn build_tracks() -> Option<Tracks> {
    if !is_petey() {
        return None; // other bosses: stand-in shapes (no model loader for them yet)
    }
    let g = geno()?;
    let m = crate::tu::petey_model::get()?;
    let (al, ar) = (m.arm_l.as_ref()?, m.arm_r.as_ref()?);
    let p = m.params.as_ref()?;
    // the hurtboxes the model's parameter file declares: (bone, offset, stretch end, radius)
    let hb = |h: &crate::tu::petey_model::Hurt| (h.flags.pshr(23), h.off, h.stretch, h.r);
    let (hbody, hl, hr) = (hb(p.body.first()?), hb(p.arm_l.first()?), hb(p.arm_r.first()?));
    let (bb, bl, br) = (m.body.bone_by_index(hbody.0)?, al.bone_by_index(hl.0)?, ar.bone_by_index(hr.0)?);
    // the define's boxes: every (attack, box, part) a box can sit on (a limb box on each weak point)
    let mut gkeys = vec![];
    for (ai, a) in g.attacks.iter().enumerate() {
        for (bi, bx) in a.boxes.iter().enumerate() {
            let limb = g.parts.get(bx.part as usize).is_some_and(|p| p.role == crate::tu::geno_boss::Role::Limb);
            if limb {
                for (pi, p) in g.parts.iter().enumerate() {
                    if p.role == crate::tu::geno_boss::Role::WeakPoint {
                        gkeys.push((ai as u8, bi as u8, pi as u8));
                    }
                }
            } else {
                gkeys.push((ai as u8, bi as u8, bx.part));
            }
        }
    }
    let mut gpts = vec![];
    let mut mots = vec![];
    for mo in &g.motions {
        let name = mo.name.as_str();
        let n = m.body.anim(name).map(|a| fctiwz_f32(a.frames).max(0) as usize).unwrap_or(mo.len.max(1) as usize);
        let (ab, aal, aar) = (m.body.anim(name), al.anim(name), ar.anim(name));
        let mut v = std::vec::Vec::with_capacity(n + 1);
        let mut gv = std::vec::Vec::with_capacity(n + 1);
        for f in 0..=n {
            let f = f as f32;
            // part 0 the body, 1 / 2 the left / right cage (this host's models)
            let mut row = std::vec::Vec::with_capacity(gkeys.len());
            for &(ai, bi, pi) in &gkeys {
                let bx = &g.attacks[ai as usize].boxes[bi as usize];
                let (mdl, an) = match pi {
                    1 => (al, aal),
                    2 => (ar, aar),
                    _ => (&m.body, ab),
                };
                row.push(match mdl.bone_by_index(bx.joint as u32) {
                    Some(bn) => mdl.bone_point(an, f, bn, bx.offset),
                    None => [0.0; 3],
                });
            }
            gv.push(row);
            let mut c = [[0.0f32; 3]; NP];
            c[P_BODY_A] = m.body.bone_point(ab, f, bb, hbody.1);
            c[P_BODY_B] = m.body.bone_point(ab, f, bb, hbody.2);
            c[P_CAGE_L_A] = al.bone_point(aal, f, bl, hl.1);
            c[P_CAGE_L_B] = al.bone_point(aal, f, bl, hl.2);
            c[P_CAGE_R_A] = ar.bone_point(aar, f, br, hr.1);
            c[P_CAGE_R_B] = ar.bone_point(aar, f, br, hr.2);
            v.push(c);
        }
        mots.push(v);
        gpts.push(gv);
    }
    let zc = (m.body.min[2] + m.body.max[2]) * 0.5;
    eprintln!("petey: bone-following hurt / hit shapes from the local data");
    Some(Tracks { mots, radii: [hbody.3, hl.3, hr.3], zc, gkeys, gpts })
}

/// This frame's points in world x y.
pub fn points() -> [(f32, f32); NP] {
    let b = unsafe { B };
    let mut out = [(0.0, 0.0); NP];
    if let Some(t) = tracks() {
        if let Some(v) = t.mots.get(b.mot.max(0) as usize).filter(|v| !v.is_empty()) {
            let c = &v[(b.mf.max(0) as usize).min(v.len() - 1)];
            for (k, q) in c.iter().enumerate() {
                out[k] = (b.px + q[0] * MODEL_SCALE, q[1] * MODEL_SCALE);
            }
            return out;
        }
    }
    stand_in_points(&b)
}

/// Without a model: plain shapes of our own (not the boss's), so a boss without its files still plays its rules.
const STAND_IN_BODY_Y: f32 = 40.0;
const STAND_IN_BODY_R: f32 = 30.0;
const STAND_IN_CAGE_DX: f32 = 30.0;
const STAND_IN_CAGE_Y: f32 = 25.0;
const STAND_IN_CAGE_R: f32 = 10.0;

fn stand_in_points(b: &Boss) -> [(f32, f32); NP] {
    let x = b.px;
    let mut o = [(0.0, 0.0); NP];
    o[P_BODY_A] = (x, STAND_IN_BODY_Y);
    o[P_BODY_B] = (x, STAND_IN_BODY_Y);
    o[P_CAGE_L_A] = (x + STAND_IN_CAGE_DX, STAND_IN_CAGE_Y + STAND_IN_CAGE_R);
    o[P_CAGE_L_B] = (x + STAND_IN_CAGE_DX, STAND_IN_CAGE_Y - STAND_IN_CAGE_R);
    o[P_CAGE_R_A] = (x - STAND_IN_CAGE_DX, STAND_IN_CAGE_Y + STAND_IN_CAGE_R);
    o[P_CAGE_R_B] = (x - STAND_IN_CAGE_DX, STAND_IN_CAGE_Y - STAND_IN_CAGE_R);
    o
}

/// Hurt capsules this frame: (ax, ay, bx, by, r) for body, weak point 1, weak point 2; `None` when not hittable (the
/// define's intangibility, a part at 0, a cause set).
fn hurt() -> [Option<[f32; 5]>; 3] {
    let b = unsafe { B };
    let p = points();
    let radii = tracks().map(|t| t.radii).unwrap_or([STAND_IN_BODY_R, STAND_IN_CAGE_R, STAND_IN_CAGE_R]);
    let cap = |a: usize, c: usize, r: f32| Some([p[a].0, p[a].1, p[c].0, p[c].1, r * MODEL_SCALE]);
    if vm().is_some() {
        // a VM boss: one stand-in body (no part shapes in the VM yet), gone once it is down
        return [if b.cause != 0 { None } else { cap(P_BODY_A, P_BODY_B, radii[0]) }, None, None];
    }
    let Some(g) = geno() else { return [None; 3] };
    let s = unsafe { &*core::ptr::addr_of!(crate::tu::geno_boss::S) };
    let live = |k: usize| k < g.parts.len() && s.intangible & 1u8.pshl(k as i64) == 0 && (k == 0 || s.hp[k] > 0);
    let body = if b.cause != 0 || !live(0) { None } else { cap(P_BODY_A, P_BODY_B, radii[0]) };
    let wp = !b.cage_broken && b.cause == 0;
    [
        body,
        if wp && live(1) { cap(P_CAGE_L_A, P_CAGE_L_B, radii[1]) } else { None },
        if wp && live(2) { cap(P_CAGE_R_A, P_CAGE_R_B, radii[2]) } else { None },
    ]
}

// ---------------------------------------------------------------- hitboxes (§8)

/// Petey's knockback scale for Melee fighters, per mille (growth and base; see `hitboxes`).
pub const KB_SCALE_PM_DEFAULT: u32 = 1000;
/// The knockback scale knob (`MELEE_PETEY_KB_PM`, per mille, 500..1500; fixed for the process). 1000 = Brawl's.
fn kb_scale_pm() -> u32 {
    static V: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        std::env::var("MELEE_PETEY_KB_PM").ok().and_then(|v| v.trim().parse::<u32>().ok()).map(|v| v.clamp(500, 1500)).unwrap_or(KB_SCALE_PM_DEFAULT)
    })
}

/// Live boxes per frame (the carrier's four hit capsules): the boss's attack boxes first, then, for a VM boss, its
/// projectiles' (geno_boss_spawn) in the capsules left.
const LIVE_HITS: usize = 4;

#[derive(Clone, Copy, Default)]
struct HitDef {
    x: f32,
    y: f32,
    r: f32,
    dmg: u32,
    angle: i32,
    kbg: u32,
    wkb: u32,
    bkb: u32,
    /// hit group (a new one re-arms the box: the rehit interval)
    grp: u32,
    /// knockback facing
    face: f32,
    /// the attack the box belongs to (0 body contact, 1 basket, 2 landing): one attack's boxes share their victims,
    /// so a victim is hit once per attack (Brawl, measured: one basket / landing hit, never two)
    atk: u8,
}

/// The boxes live this frame.
fn hit_list() -> ([HitDef; LIVE_HITS], usize) {
    let b = unsafe { B };
    let mut v = [HitDef::default(); LIVE_HITS];
    let mut n = 0;
    if b.cause != 0 || b.st == ST_OFF {
        return (v, 0);
    }
    if vm().is_some() {
        return vm_hits(&b);
    }
    match geno() {
        Some(g) => geno_hits(g, tracks().filter(|t| !t.gkeys.is_empty()), &b),
        None => (v, n),
    }
}

/// The Geno define's live boxes (geno_boss::live_boxes), on the bone tracks, with the numbers at the owning part's L.
fn geno_hits(g: &crate::tu::geno_boss::Define, t: Option<&Tracks>, b: &Boss) -> ([HitDef; LIVE_HITS], usize) {
    use crate::tu::geno_boss as gb;
    let mut v = [HitDef::default(); LIVE_HITS];
    let mut live = [gb::LiveBox::default(); 8];
    let n = gb::live_boxes(g, &mut live);
    let f = b.mf;
    let mot = b.mot.max(0) as usize;
    let row = t.and_then(|t| t.gpts.get(mot)).and_then(|m| m.get((f.max(0) as usize).min(m.len().saturating_sub(1))));
    // without bone tracks: the box offset from the stand-in body centre (no joint motion)
    let centre = stand_in_points(b)[P_BODY_A];
    let mut k = 0;
    for lb in &live[..n] {
        if k >= 4 {
            break;
        }
        let bx = &g.attacks[lb.attack as usize].boxes[lb.boxi as usize];
        let q = match t {
            Some(t) => {
                let Some(ix) = t.gkeys.iter().position(|&x| x == (lb.attack, lb.boxi, lb.part)) else { continue };
                let Some(q) = row.and_then(|r| r.get(ix)) else { continue };
                *q
            }
            None => [centre.0 - b.px + bx.offset[0], centre.1 + bx.offset[1], 0.0],
        };
        let l = gb::part_level(g, lb.part);
        let x = b.px + q[0] * MODEL_SCALE;
        let y = q[1] * MODEL_SCALE;
        // knockback facing: the owning part's fixed facing; a facing-less part launches away from its own centre
        let pf = g.parts.get(lb.part as usize).map(|p| p.facing).unwrap_or(0.0);
        let face = if pf != 0.0 {
            pf
        } else if bx.offset[0] != 0.0 {
            if bx.offset[0] < 0.0 { -1.0 } else { 1.0 }
        } else if bx.angle == 270 {
            b.aim
        } else if x >= b.px {
            1.0
        } else {
            -1.0
        };
        let rehit = bx.rehit.max(1);
        let id = lb.attack as u32 * 8 + bx.group as u32;
        v[k] = HitDef {
            x,
            y,
            r: gb::num_at(bx.size, l) * MODEL_SCALE,
            dmg: fctiwz_f32(gb::num_at(bx.damage, l)).max(0) as u32,
            angle: bx.angle,
            kbg: fctiwz_f32(gb::num_at(bx.kbg, l)).max(0) as u32,
            wkb: fctiwz_f32(gb::num_at(bx.wkb, l)).max(0) as u32,
            bkb: fctiwz_f32(gb::num_at(bx.bkb, l)).max(0) as u32,
            grp: b.inst.wrapping_mul(16).wrapping_add(id).wrapping_add(f.pdiv(rehit) as u32 * 0x1000_0000),
            face,
            atk: lb.attack + 1,
        };
        k += 1;
    }
    (v, k)
}

unsafe fn disable_all(cf: &mut Fighter) {
    for i in 0..4 {
        if SLOT_INST[i] != 0 {
            cf.x914[i].state = HitCapsule_Disabled;
            SLOT_INST[i] = 0;
        }
    }
}

/// Mirror the hit list into the carrier's capsule slots (offsets local to its root joint).
unsafe fn hitboxes(cf: &mut Fighter) {
    let (list, n) = hit_list();
    if n == 0 || B.ko_done {
        disable_all(cf);
        return;
    }
    let root = (*cf.parts).joint;
    if root.is_null() {
        return;
    }
    let inv = melee_runtime::hsd_mtx::inverse(&(*root).mtx);
    for i in 0..4 {
        if i >= n {
            if SLOT_INST[i] != 0 {
                cf.x914[i].state = HitCapsule_Disabled;
                SLOT_INST[i] = 0;
            }
            continue;
        }
        let d = list[i];
        let cfp: *mut Fighter = cf;
        let h = &mut *core::ptr::addr_of_mut!((*cfp).x914[i]);
        if SLOT_INST[i] == d.grp && h.state == HitCapsule_Disabled {
            *h = SAVED[i]; // cleared by the carrier's own state change: same box, same victims
        }
        if SLOT_INST[i] != d.grp || h.state == HitCapsule_Disabled {
            *h = core::mem::zeroed();
            h.x4 = d.grp;
            h.state = HitCapsule_Enabled;
            melee_fighter::tu::ftcoll::ftcoll_800768a0(cfp, h);
            h.jobj = root;
            h.damage = d.dmg as f32;
            h.unk_count = d.dmg;
            h.kb_angle = d.angle;
            // knockback scale knob (MELEE_PETEY_KB_PM, default 1000 = Brawl's): the one-shots came from FD's close blast
            // zones, now replaced by the Brawl arena's (read at run time), so Brawl's growth and base apply unchanged.
            h.x24 = d.kbg * kb_scale_pm() / 1000;
            h.x28 = d.wkb;
            h.x2C = d.bkb * kb_scale_pm() / 1000;
            h.element = 0;
            h.x34 = 0;
            h.sfx_severity = 1;
            h.set_x40_b2(1);
            h.set_x40_b3(1);
            h.set_x42_b5(1);
            h.set_x42_b7(1);
            h.set_x42_b1(1);
            (*cfp).set_x2219_b3(1);
            SLOT_INST[i] = d.grp;
        }
        // knockback direction: Melee launches a fighter's victim relative to the attacker's (the invisible carrier's) x,
        // Brawl in the box owner's facing (cage L +1, cage R -1; landing sides outward). Mirror the angle so the launch
        // matches the facing for the player nearest the box (measured: basket L -> -x, basket R -> +x at 145).
        if d.angle < 180 && d.angle != 90 {
            let mut best: Option<f32> = None;
            for s in 0..4 {
                let v = fighter_of(s, 0);
                if s != B.carrier_slot && !v.is_null() {
                    let x = (*v).cur_pos.x;
                    if best.map_or(true, |b| (x - d.x).abs() < (b - d.x).abs()) {
                        best = Some(x);
                    }
                }
            }
            if let Some(vx) = best {
                let away = if vx > (*cfp).cur_pos.x { 1.0f32 } else { -1.0 };
                // launch x sign = away * sign(cos a): keep a when away == facing, else mirror it (180 - a)
                h.kb_angle = if away == d.face { d.angle } else { 180 - d.angle };
            }
        }
        let m = &inv;
        h.b_offset = Vec3 {
            x: m[0][0] * d.x + m[0][1] * d.y + m[0][3],
            y: m[1][0] * d.x + m[1][1] * d.y + m[1][3],
            z: m[2][0] * d.x + m[2][1] * d.y + m[2][3],
        };
        h.scale = d.r;
        h.set_x43_b1(1);
        SAVED[i] = *h;
        cf.facing_dir = d.face;
    }
    // one attack's boxes share their victims
    for i in 0..n {
        for j in 0..n {
            if i == j || list[i].atk == 0 || list[i].atk != list[j].atk {
                continue;
            }
            for k in 0..cf.x914[i].victims_1.len() {
                let v = cf.x914[i].victims_1[k];
                if v.victim.is_null() || cf.x914[j].victims_1.iter().any(|w| w.victim == v.victim) {
                    continue;
                }
                if let Some(e) = cf.x914[j].victims_1.iter_mut().find(|w| w.victim.is_null()) {
                    *e = v;
                }
            }
        }
    }
    for i in 0..n {
        SAVED[i] = cf.x914[i];
    }
}

// ---------------------------------------------------------------- hits on Petey

/// 2D distance between segments pq and ab.
fn seg_dist(p: (f32, f32), q: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    fn pt(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = dx * dx + dy * dy;
        let t = if l > 0.0 { fclamp(((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l, 0.0, 1.0) } else { 0.0 };
        let (ex, ey) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
        melee_runtime::gekko::sqrtf(ex * ex + ey * ey)
    }
    fmin(fmin(pt(p, a, b), pt(q, a, b)), fmin(pt(a, p, q), pt(b, p, q)))
}

/// One live hitbox against the body and both cages (bits in `latch`, once per hitbox lifetime). Body damage is
/// forwarded to both cages (§5); a cage hit damages that cage. Returns (hit anything, cage bits hit).
unsafe fn hit_test(hs: &[Option<[f32; 5]>; 3], a: (f32, f32), b: (f32, f32), r: f32, dmg: f32, latch: &mut u8) -> (bool, u8) {
    hit_test_n(hs, a, b, r, dmg, latch, false)
}

/// `single`: stop at the first part hit, cages before the body (a projectile is spent on one part).
unsafe fn hit_test_n(hs: &[Option<[f32; 5]>; 3], a: (f32, f32), b: (f32, f32), r: f32, dmg: f32, latch: &mut u8, single: bool) -> (bool, u8) {
    let mut any = false;
    let mut cages = 0u8;
    let d = round_i(dmg);
    let order: [usize; 3] = if single { [1, 2, 0] } else { [0, 1, 2] };
    for k in order {
        if single && any {
            break;
        }
        let h = &hs[k];
        let bit = 1u8.pshl(k as i64);
        let Some(c) = h else { continue };
        if *latch & bit != 0 {
            continue;
        }
        if seg_dist(a, b, (c[0], c[1]), (c[2], c[3])) <= r + c[4] {
            *latch |= bit;
            any = true;
            B.flash[k] = 8;
            if vm().is_some() {
                VM_DMG += d;
            } else if let Some(g) = geno() {
                crate::tu::geno_boss::damage(g, k, d);
                if k > 0 {
                    cages |= 1u8.pshl(k as i64 - 1);
                }
            } else if k == 0 {
                B.cage[0] -= d;
                B.cage[1] -= d;
            } else {
                B.cage[k - 1] -= d;
                cages |= 1u8.pshl(k as i64 - 1);
            }
        }
    }
    (any, cages)
}

unsafe fn player_hits(carrier: i32) -> (bool, [bool; 2]) {
    let hs = hurt();
    let mut any = false;
    let mut cages = 0u8;
    for s in 0..4i32 {
        if s == carrier {
            continue;
        }
        for e in 0..2usize {
            let f = fighter_of(s, e);
            if f.is_null() {
                continue;
            }
            let ratio = pl::Player_GetAttackRatio(s);
            for i in 0..4 {
                let h: *mut HitCapsule = core::ptr::addr_of_mut!((*f).x914[i]);
                let latch = &mut LATCH[s as usize][e][i];
                if (*h).state == HitCapsule_Disabled {
                    *latch = 0;
                    continue;
                }
                let r = if (*h).x43_b1() != 0 { (*h).scale } else { (*h).scale * (*f).x34_scale.y };
                let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
                let dmg = (*h).damage * ratio;
                let (hit, cg) = hit_test(&hs, a, b, r, dmg, latch);
                if hit {
                    any = true;
                    cages |= cg;
                    HITS[s as usize] = HITS[s as usize].wrapping_add(1);
                    DMG[s as usize] = DMG[s as usize].wrapping_add(fctiwz_f32(dmg * 1000.0).max(0) as u32);
                    // the attacker's side of a landed hit, as for hitting an item: hitlag next frame, the hit sound
                    let fp = &mut *f;
                    let d = round_i(dmg).max(1);
                    if (fp.dmg.x1914 as i32) < d {
                        fp.dmg.x1914 = d as _;
                    }
                    let slot = 0x7E + fp.player_idx as i32 * 2 + fp.is_sub_fighter() as i32;
                    melee_runtime::tu::lbcollision::lbColl_80005BB0(h, slot);
                }
            }
        }
    }
    let (ia, ic) = item_hits(&hs, carrier);
    any |= ia;
    cages |= ic;
    (any, [cages & 1 != 0, cages & 2 != 0])
}

/// Items and projectiles hit Petey and the cages. Latched per item and hitbox until that hitbox goes off.
unsafe fn item_hits(hs: &[Option<[f32; 5]>; 3], carrier: i32) -> (bool, u8) {
    let mut next: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];
    let mut n = 0;
    let mut any = false;
    let mut cages = 0u8;
    let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !g.is_null() {
        let ip = (*g).user_data as *mut Item;
        g = (*g).next;
        if ip.is_null() {
            continue;
        }
        let it = &mut *ip;
        let owner = it.owner;
        if !owner.is_null() {
            let of = (*owner).user_data as *mut Fighter;
            if (*owner).classifier == HSD_GOBJ_CLASS_FIGHTER as _ && !of.is_null() && (*of).player_idx as i32 == carrier {
                continue;
            }
        }
        let key = it.entity as usize;
        // a projectile hits Petey once in its life, on the first of his parts it touches (Brawl: a laser or a thrown
        // item is spent on the boss). The latch lives as long as the item (a laser's hitbox re-arms every frame, which
        // re-hit him each frame and hit the body and both cages in turn: ~10 cage HP per 3% laser, owner Nov 13).
        let mut lat = LATCH_IT.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or([0; 4]);
        for i in 0..4 {
            let h: *mut HitCapsule = core::ptr::addr_of_mut!(it.x5D4_hitboxes[i].hit);
            if (*h).state == HitCapsule_Disabled || lat[0] == 0xFF {
                continue;
            }
            let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
            let mut one = 0u8;
            let before = B.cage;
            let (hit, cg) = hit_test_n(hs, a, b, (*h).scale, (*h).damage, &mut one, true);
            if hit {
                any = true;
                cages |= cg;
                lat = [0xFF; 4];
                melee_runtime::tu::lbcollision::lbColl_80005BB0(h, 0x7E);
                if log_on() {
                    eprintln!("petey: f{} item hit {:.1} on {} cages {} {} -> {} {} ({:.1}% {:.1}%)", B.last_frame, (*h).damage, if cg == 0 { "body" } else { "cage" }, before[0], before[1], B.cage[0], B.cage[1], cage_pct(0), cage_pct(1));
                }
            }
        }
        if n < next.len() {
            next[n] = (key, lat);
            n += 1;
        }
    }
    LATCH_IT = next;
    (any, cages)
}

// ---------------------------------------------------------------- presentation

/// Words for `gd_ui_petey`: active, hp (cage HP left above the death line), hp max, cage at screen left, cage at
/// screen right, cage max, state (7 = beaten), phase >= 1, cause, status, partner kind, real model drawn.
pub fn view() -> [i32; 12] {
    let b = unsafe { B };
    let line = 0; // the death threshold lives in the define's rules: the bar shows the cages' whole HP
    let low = b.cage[0].min(b.cage[1]).max(0);
    [
        (b.st != ST_OFF) as i32,
        (low - line).max(0),
        cage_max() - line,
        b.cage[1].max(0), // the right cage hangs at screen left (model -x)
        b.cage[0].max(0),
        cage_max(),
        if b.cause != 0 { 7 } else { 0 },
        (b.phase >= 1) as i32,
        b.cause,
        b.st,
        unsafe { crate::tu::envoy::ROSTER[1] },
        crate::tu::petey_model::loaded() as i32,
    ]
}

/// Draw: the owner's model when present, else plain capsules, in the carrier's draw pass (opaque pass only).
pub unsafe fn draw(g: *mut HSD_GObj, pass: i32) {
    if pass != 0 || B.st == ST_OFF || B.carrier_slot < 0 {
        return;
    }
    let cf = fighter_of(B.carrier_slot, 0);
    if cf.is_null() || (*cf).gobj != g {
        return;
    }
    if let Some(p) = vm() {
        if crate::tu::vm_boss_model::draw(vm_run(p), crate::tu::geno_vm::actor(), B.flash[0] > 0)
            && std::env::var_os("MELEE_PETEY_SHAPES").is_none()
        {
            return;
        }
    }
    // a grab-define actor (Tabuu, Bucculus): its model posed by the define's motion and frame
    if is_grab_actor(boss_name()) {
        let gs = &*core::ptr::addr_of!(crate::tu::geno_boss::S);
        let name = geno().and_then(|g| g.motions.get(gs.motion.max(0) as usize)).map(|m| m.name.as_str());
        if let Some(n) = name {
            if crate::tu::vm_boss_model::draw_named(n, gs.mf, gs.x, gs.aim, B.flash[0] > 0) && std::env::var_os("MELEE_PETEY_SHAPES").is_none() {
                return;
            }
        }
    }
    if let Some(m) = is_petey().then(crate::tu::petey_model::get).flatten() {
        draw_model(m);
        if std::env::var_os("MELEE_PETEY_SHAPES").is_none() {
            return;
        }
    }
    let z = 0.0;
    let col = |r: u8, gg: u8, b: u8| GXColor { r, g: gg, b, a: 0xFF };
    let cap = |a: [f32; 5], c: GXColor| {
        let d = col(c.r / 2, c.g / 2, c.b / 2);
        melee_runtime::tu::lbcollision::lbColl_80008FC8(Vec3 { x: a[0], y: a[1], z }, Vec3 { x: a[2], y: a[3], z }, &c, &d, a[4]);
    };
    let hit = |i: usize, c: GXColor| if B.flash[i] > 0 { col(0xFF, 0xFF, 0xFF) } else { c };
    let hs = hurt();
    let colors = [
        if B.phase >= 1 { col(0x8A, 0x4A, 0x9A) } else { col(0x4A, 0x7A, 0x9A) },
        col(0xE0, 0x90, 0xB0),
        col(0xB0, 0xB0, 0xE0),
    ];
    for (k, h) in hs.iter().enumerate() {
        if let Some(c) = h {
            cap(*c, hit(k, colors[k]));
        }
    }
    let (list, n) = hit_list();
    for d in list.iter().take(n) {
        if d.dmg > 0 {
            cap([d.x, d.y, d.x, d.y, d.r], col(0xE0, 0x30, 0x30));
        }
    }
}

/// The owner's model, posed by the sim's motion and motion frame (presentation only; a pure function of the
/// statics, so a replay or rollback redraws the same picture).
unsafe fn draw_model(m: &'static crate::tu::petey_model::Model) {
    use crate::tu::petey_model as pm;
    // one buffer set per part, kept until the next draw (the GX bridge may read the vertex data after the call
    // returns), made once on a helper thread: a draw inside the game scope must not allocate, or rollback rewrites
    // the arena under the buffers (write-after-free, desyncs, a stalled synctest with the owner's files present)
    static BUFS: std::sync::OnceLock<std::sync::Mutex<std::vec::Vec<pm::DrawBufs>>> = std::sync::OnceLock::new();
    let bufs = BUFS.get_or_init(|| {
        let v = std::thread::spawn(move || {
            let mut v = std::vec::Vec::with_capacity(3);
            v.push(m.body.draw_bufs());
            for r in [&m.arm_l, &m.arm_r] {
                if let Some(r) = r {
                    v.push(r.draw_bufs());
                }
            }
            std::sync::Mutex::new(v)
        });
        v.join().unwrap_or_else(|_| std::sync::Mutex::new(std::vec::Vec::new()))
    });
    let Ok(mut bufs) = bufs.lock() else { return };
    if bufs.is_empty() {
        return;
    }
    let Some(name) = geno().and_then(|g| g.motions.get(B.mot.max(0) as usize)).map(|m| m.name.as_str()) else { return };
    let fr = B.mf as f32;
    let zc = (m.body.min[2] + m.body.max[2]) * 0.5;
    // the clip's own root carries the jump height; y 0 is the stage
    let org = [0.0, zc];
    let at = [B.px, 0.0, 0.0];
    let a = m.body.anim(name);
    m.body.skin_with(a, fr, &mut bufs[0]);
    pm::draw_tris(&m.body, &bufs[0].tris, org, at, MODEL_SCALE, 0.0, B.flash[0] > 0);
    let mut k = 1;
    for (i, arm) in [&m.arm_l, &m.arm_r].into_iter().enumerate() {
        let Some(r) = arm else { continue };
        let slot = k;
        k += 1;
        if B.cage_broken && B.freed == i as i32 {
            continue; // the broken cage hides its model (§6.2 proc 17)
        }
        if let Some(b) = bufs.get_mut(slot) {
            r.skin_with(r.anim(name), fr, b);
            pm::draw_tris(r, &b.tris, org, at, MODEL_SCALE, 0.0, B.flash[1 + i] > 0);
        }
    }
}

// ---------------------------------------------------------------- the Geno actor (docs/specs/geno-v5.2-boss.md)

/// The motions the define is lowered with: the IR's own (names, loops) with each clip's length from the owner's model
/// when it has that clip (the IR's timeline length otherwise).
fn petey_motions(ir: &crate::tu::geno_boss_ir::J) -> std::vec::Vec<(String, i32, bool)> {
    let mut ms = crate::tu::geno_boss_ir::ir_motions(ir);
    if let Some(m) = crate::tu::petey_model::get() {
        for x in ms.iter_mut() {
            if let Some(a) = m.body.anim(&x.0) {
                x.1 = fctiwz_f32(a.frames).max(1);
            }
        }
    }
    ms
}

/// The boss's Geno define, lowered from the local IR once per process (None without the IR: no boss).
pub fn geno() -> Option<&'static crate::tu::geno_boss::Define> {
    // grab actors (geno_boss_grab): Tabuu's and Bucculus's defines, built from their IR's grab data
    if let Some(d) = grab_actor() {
        return Some(d);
    }
    static D: std::sync::OnceLock<Option<crate::tu::geno_boss::Define>> = std::sync::OnceLock::new();
    D.get_or_init(|| {
        // lowered on a helper thread: its allocations stay on the system heap, outside the game arena
        let name = boss_name();
        let r = std::thread::spawn(move || {
            if name == "petey" {
                let dir = crate::tu::geno_boss_ir::ir_dir().ok_or("no boss IR directory")?;
                let text = std::fs::read_to_string(dir.join("petey.brawl.ir.json")).map_err(|_| "petey.brawl.ir.json: not readable".to_string())?;
                let j = crate::tu::geno_boss_ir::parse_json(&text)?;
                let ms = petey_motions(&j);
                let hm: std::vec::Vec<crate::tu::geno_boss_ir::HostMotion> = ms.iter().map(|m| crate::tu::geno_boss_ir::HostMotion { name: &m.0, len: m.1, looping: m.2 }).collect();
                crate::tu::geno_boss_ir::lower(&j, &hm)
            } else if vm_boss(name) {
                Err("run by the enemy-command VM".into())
            } else {
                crate::tu::geno_boss_ir::load_with_ir_motions(&format!("{name}.brawl.ir.json"))
            }
        })
        .join()
        .unwrap_or_else(|_| Err("lowering panicked".into()));
        match r {
            Ok(d) => {
                eprintln!("petey: Geno boss define from the local IR ({})", name);
                Some(d)
            }
            Err(e) => {
                eprintln!("petey: no Geno define ({e})");
                None
            }
        }
    })
    .as_ref()
}

unsafe fn world() -> crate::tu::geno_boss::World {
    let mut w = crate::tu::geno_boss::World::default();
    for s in 0..4i32 {
        if s == B.carrier_slot {
            continue;
        }
        let f = fighter_of(s, 0);
        if !f.is_null() {
            w.slots[s as usize] = Some((*f).cur_pos.x);
        }
    }
    // in the air while a jump clip plays (the IR's motion names)
    w.airborne = geno().and_then(|g| g.motions.get(B.mot.max(0) as usize)).is_some_and(|m| m.name == "JumpLoop" || m.name == "JumpAttack");
    w
}

/// The VS bosses built by geno_boss_grab (their grab define, not the full IR or the VM).
fn is_grab_actor(name: &str) -> bool {
    matches!(name, "tabuu" | "bucculus")
}

fn grab_actor() -> Option<&'static crate::tu::geno_boss::Define> {
    match boss_name() {
        "tabuu" => crate::tu::geno_boss_grab::tabuu(),
        "bucculus" => crate::tu::geno_boss_grab::bucculus(),
        _ => None,
    }
}

/// Bosses the enemy-command VM runs (geno_vm): every boss it has files for except Petey (his define) and the
/// grab actors (their grab defines).
pub fn vm_boss(name: &str) -> bool {
    name != "petey" && !is_grab_actor(name) && brawl_boss::vm::FILES.iter().any(|f| f.0 == name)
}

/// The VS boss's start status from its IR (the IR is parsed once, ahead of the match by the engine-start helper).
pub fn vm_start_status() -> Option<i32> {
    static S: std::sync::OnceLock<Option<i32>> = std::sync::OnceLock::new();
    *S.get_or_init(|| {
        brawl_boss::vm::FILES.iter().find(|f| f.0 == boss_name()).and_then(|f| crate::tu::geno_boss_ir::ir_start_status(&format!("{}.brawl.ir.json", f.1)))
    })
}

/// The VS boss's own Brawl arena box: the blast zones (Dead0N / Dead1N) of the stage position model in the boss's
/// stage archive, as [left, right, top, bottom]. The boss files' goals assume this arena (Galleom's charge aims at
/// +-225 inside his +-240). Read once, ahead of the match, by the engine-start helper.
pub fn vm_brawl_arena() -> Option<[f32; 4]> {
    static S: std::sync::OnceLock<Option<[f32; 4]>> = std::sync::OnceLock::new();
    *S.get_or_init(|| {
        let f = brawl_boss::vm::FILES.iter().find(|f| f.0 == boss_name())?;
        let leaves = crate::tu::petey_model::pac_leaves(f.2)?;
        leaves.iter().find_map(|l| crate::tu::petey_model::arena_blast(l))
    })
}

/// The VS boss's VM program (None: not a VM boss, or the player's files are absent).
pub fn vm() -> Option<&'static brawl_boss::vm::Program> {
    static P: std::sync::OnceLock<Option<&'static brawl_boss::vm::Program>> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        if !vm_boss(boss_name()) {
            return None;
        }
        // the model sets the program's motion lengths from its clips: wait for its load (started at the engine start)
        let _ = crate::tu::vm_boss_model::get(boss_name());
        crate::tu::geno_vm::program(boss_name())
    })
}

/// The VS boss's program with the arena box set from this stage's blast zones (geno_vm `Program::bounds`: the VM's
/// movement stops at its edges). One copy per distinct box, made at the match's start, outside the frame loop.
static mut VM_ARENA: Option<([f32; 4], *const brawl_boss::vm::Program, &'static brawl_boss::vm::Program)> = None;

unsafe fn vm_arena(p: &'static brawl_boss::vm::Program) -> &'static brawl_boss::vm::Program {
    // the latest program: the model loader replaces it once it has the motion lengths (geno_vm::set_motion_lengths),
    // after `vm()` cached the first copy; without them every motion ended at the default 60 frames (Galleom's Shoot
    // ended before its missiles at frames 74-119)
    let p = crate::tu::geno_vm::program(boss_name()).unwrap_or(p);
    // the boss area: the boss's own Brawl arena (its blast zones), which its scripts' goals assume. It is used as is
    // when it fits inside this stage's blast zones; otherwise it is cut to them, logged (goals beyond the cut are
    // then taken at the edge, see the VM's arrival stand-in). Without the arena: the stage's camera bounds.
    let bz = &*core::ptr::addr_of!(crate::tu::script_game::stage_info.blast_zone);
    let cb = &*core::ptr::addr_of!(crate::tu::script_game::stage_info.cam_info.cam_bounds);
    let (bounds, src) = match vm_brawl_arena() {
        Some([l, r, t, b]) => {
            let cut = [fmax(l, bz.left), fmin(r, bz.right), fmax(b, bz.bottom), fmin(t, bz.top)];
            if cut == [l, r, b, t] {
                (cut, "the boss's Brawl arena")
            } else {
                eprintln!(
                    "petey: WARNING: {}'s Brawl arena x {:.0}..{:.0} y {:.0}..{:.0} does not fit this stage's blast zones x {:.0}..{:.0} y {:.0}..{:.0}; cut to them",
                    boss_name(), l, r, b, t, bz.left, bz.right, bz.bottom, bz.top
                );
                (cut, "the boss's Brawl arena, cut to the stage")
            }
        }
        None => ([fmax(cb.left, bz.left), fmin(cb.right, bz.right), fmax(cb.bottom, bz.bottom), fmin(cb.top, bz.top)], "stage camera bounds: no Brawl arena"),
    };
    if let Some((b, base, q)) = VM_ARENA {
        if b == bounds && core::ptr::eq(base, p) {
            return q;
        }
    }
    let mut q = p.clone();
    q.bounds = bounds;
    let q: &'static brawl_boss::vm::Program = Box::leak(Box::new(q));
    VM_ARENA = Some((bounds, p as *const _, q));
    eprintln!("petey: vm arena box x {:.0}..{:.0} y {:.0}..{:.0} ({src})", bounds[0], bounds[1], bounds[2], bounds[3]);
    q
}

/// The program this match runs (the arena copy once the match began).
unsafe fn vm_run(p: &'static brawl_boss::vm::Program) -> &'static brawl_boss::vm::Program {
    VM_ARENA.map(|(_, _, q)| q).unwrap_or(p)
}

/// Damage the players' boxes dealt the VM boss this frame (summed in hit_test, consumed by vm_frame).
static mut VM_DMG: i32 = 0;

/// One VM frame (geno_vm_host::frame): this frame's damage in, the actor stepped, its state mirrored.
unsafe fn vm_frame(p: &'static brawl_boss::vm::Program) {
    unsafe fn fo(s: i32) -> *mut Fighter {
        fighter_of(s, 0)
    }
    let d = VM_DMG;
    VM_DMG = 0;
    let o = crate::tu::geno_vm_host::frame(p, B.carrier_slot, B.last_frame, d, true, fo, log_on());
    B.st = o.status;
    B.px = o.x;
    B.aim = if o.lr < 0.0 { -1.0 } else { 1.0 };
    if B.cause == 0 && o.dead {
        B.cause = 1;
        eprintln!("petey: f{} dying (vm hp {:.1})", B.last_frame, crate::tu::geno_vm::actor().hp());
    }
}

/// The VM boss's live attack boxes (geno_vm_host::BOXES from its forwarded 0619s) as hit capsules: placed at the
/// its bone (vm_boss_model::bone_world), or the actor's position plus the box offset without the model.
fn vm_hits(b: &Boss) -> ([HitDef; LIVE_HITS], usize) {
    let mut v = [HitDef::default(); LIVE_HITS];
    let mut k = 0;
    let a = crate::tu::geno_vm::actor();
    for bx in unsafe { crate::tu::geno_vm_host::BOXES }.iter().filter(|x| x.on) {
        if k >= 4 {
            break;
        }
        // on its bone when the boss has its model (the bone's pose at the clip frame, turned by the facing / rotation);
        // else at the actor's position plus the offset
        let (x, y) = vm()
            .and_then(|p| crate::tu::vm_boss_model::bone_world(unsafe { vm_run(p) }, a, bx.bone, bx.offset))
            .map(|q| (q[0], q[1]))
            .unwrap_or((b.px + bx.offset[0] * b.aim, a.pos[1] + bx.offset[1]));
        v[k] = HitDef {
            x,
            y,
            r: bx.size * MODEL_SCALE,
            dmg: fctiwz_f32(bx.damage).max(0) as u32,
            angle: bx.angle,
            kbg: fctiwz_f32(bx.kbg).max(0) as u32,
            wkb: fctiwz_f32(bx.wkb).max(0) as u32,
            bkb: fctiwz_f32(bx.bkb).max(0) as u32,
            grp: bx.inst.wrapping_mul(16).wrapping_add(bx.id as u32),
            face: b.aim,
            atk: bx.id.wrapping_add(1),
        };
        k += 1;
    }
    // the projectiles (650E weapons): each live box at the projectile, its numbers at the weapon's L, one attack
    // per projectile so a victim is hit once per phase
    crate::tu::geno_boss_spawn::live_boxes(unsafe { crate::tu::geno_vm_host::WEAPONS }, |slot, x, y, bx, l, grp| {
        if k >= LIVE_HITS {
            return;
        }
        let (dmg, kbg, bkb, skb) = crate::tu::geno_boss_spawn::at_l(bx, l);
        if log_on() {
            eprintln!("petey: f{} weapon box slot {slot} at ({x:.1}, {y:.1}) r {:.1} dmg {dmg:.1} capsule {k}", b.last_frame, bx.size);
        }
        v[k] = HitDef {
            x,
            y,
            r: bx.size * MODEL_SCALE,
            dmg: fctiwz_f32(dmg).max(0) as u32,
            angle: bx.angle,
            kbg: fctiwz_f32(kbg).max(0) as u32,
            wkb: fctiwz_f32(skb).max(0) as u32,
            bkb: fctiwz_f32(bkb).max(0) as u32,
            grp,
            face: b.aim,
            atk: 0x80 | slot as u8,
        };
        k += 1;
    });
    (v, k)
}

/// Copy the actor's state into the host's view (draw, hit shapes, HUD and Envoy read `B`).
unsafe fn mirror(g: &crate::tu::geno_boss::Define) {
    use crate::tu::geno_boss::S;
    let var = |n: &str| g.vars.iter().position(|(k, _)| k == n).map(|i| S.vars[i]).unwrap_or(0.0);
    B.prev_st = S.prev_status;
    B.st = S.status;
    B.sf = S.sf;
    B.mot = S.motion;
    B.mf = S.mf;
    B.tl_mf = S.tl_mf;
    B.px = S.x;
    B.vx = S.vx;
    B.aim = S.aim;
    B.cage = [S.hp[1], S.hp[2]];
    B.locked = S.locked;
    B.inst = S.inst;
    let ph = fctiwz_f32(var("phase"));
    if ph != B.phase {
        eprintln!("petey: phase {} (L {:.1})", ph, level());
    }
    B.phase = ph;
    B.t_out = var("T");
    B.cage_broken = var("cageBroken") != 0.0;
    if B.cause == 0 && S.cause != 0 {
        if S.cause == 1 {
            B.freed = if var("lowerSideIsLeft") != 0.0 { 0 } else { 1 };
            eprintln!("petey: f{} dying (lower cage {} at {:.1}%)", B.last_frame, B.freed, fmin(cage_pct(0), cage_pct(1)));
        } else {
            eprintln!("petey: f{} cage broken: Down", B.last_frame);
        }
        B.cause = S.cause;
    }
}

/// Frames from the cause (death / break) to the carrier's KO: the define's first stage notification past a motion's
/// opening frames (Petey's Down timeline), or 0 (at once) when its timelines have none.
fn down_end() -> i32 {
    static V: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *V.get_or_init(|| {
        let Some(g) = geno() else { return 0 };
        g.motions
            .iter()
            .flat_map(|m| m.timeline.iter())
            .filter(|e| e.frame > 1 && matches!(e.op, crate::tu::geno_boss::Op::StageEvent(_) | crate::tu::geno_boss::Op::StageEventIf(..)))
            .map(|e| e.frame)
            .min()
            .unwrap_or(0)
    })
}

/// One Geno frame: the actor ticks, its events go to the presentation (sounds, shakes, stage events).
unsafe fn geno_frame(g: &crate::tu::geno_boss::Define) {
    use crate::tu::geno_boss as gb;
    let mut last = B.st;
    gb::tick(g, world());
    let ev = gb::S.ev;
    let n = gb::S.ev_n as usize;
    for &(k, v) in ev.iter().take(n) {
        match k {
            gb::EV_STATUS => {
                if log_on() {
                    let name = g.status(v as u8).map(|s| s.name.as_str()).unwrap_or("?");
                    eprintln!("petey: f{} status {} -> {} ({name}) px {:.1} cages {} {}", B.last_frame, last, v, gb::S.x, gb::S.hp[1], gb::S.hp[2]);
                }
                last = v;
            }
            gb::EV_SOUND => {
                let f = gb::S.mf;
                // the IR's sound id (an id the IR left unread plays nothing)
                if v >= 0 {
                    let id = v as u16;
                    B.mot = gb::S.motion;
                    push_snd(id, f);
                }
            }
            gb::EV_SHAKE => {
                let kind = [QuakeKind_Small, QuakeKind_Medium, QuakeKind_Large][(v as usize).min(2)];
                if log_on() {
                    eprintln!("petey: f{} shake {} (motion {} frame {})", B.last_frame, v, gb::S.motion, gb::S.mf);
                }
                melee_world::tu::camera::Camera_RequestQuake(kind, core::ptr::null_mut());
            }
            gb::EV_STAGE => {
                eprintln!("petey: f{} stage event {}", B.last_frame, v);
                match v {
                    1000 => B.freed = 0,
                    1001 => B.freed = 1,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    mirror(g);
}
