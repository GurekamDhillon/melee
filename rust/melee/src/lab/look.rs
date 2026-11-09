//! The lookahead primitive (docs/rust-port/notes/lab-theorycraft.md 3b, 4c): from the anchored state, save, tick the one
//! engine with chosen pads until each fighter can act (or the defender is KO'd, cap [`LOOK_CAP`] frames), load back.
//!
//! - Speculative: the ticks run with `melee_engine::engine::speculative()` set, the sound taken out
//!   (`melee_audio::sim`), GX draws suppressed, and none of `LabRun`'s observers (events, meter cells, observed frame
//!   data, `frame_adv`, ring, input log, recordings, playback, the dummy and the lab RNG, perf counters, log lines) run:
//!   the loop below calls `ModesGame::step` directly. The frame the live tick drew is detached first (a load forgets
//!   the arena's copy) and handed to the play flow by [`LabRun::take_frame`].
//! - The sound is out of the static for every save and load done here, so every snapshot taken here holds "no sound"
//!   and loading one never restores a stale `Sound` header over the live one; it goes back in after the final load.
//! - Frozen: a job runs to completion. Live (LOOK_LIVE): it is sliced, at most the budget per presented frame; a slice
//!   saves the live state, loads the job's resume point, ticks, saves the resume point and loads the live state.
//! - Local only: the request and result live in `LabShared` (never a command); the sim ends hash-equal (verified with
//!   `look_verify`, and `MELEE_SNAP_CHECK` checks every save and load).

use super::*;
use std::time::{Duration, Instant};

/// A pad policy for one port during a lookahead.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Policy {
    Neutral,
    /// Hold shield (R fully pressed).
    Shield,
    /// DI: the stick held while in hitlag and hitstun (DI at the launch, plus the drift it gives), neutral otherwise.
    Di(i8, i8),
}

#[derive(Clone, Copy)]
struct RunSpec {
    tag: i32,
    def: Policy,
    /// RNG sweep: the game's logic seed for this run (None: the anchor's own).
    seed: Option<u32>,
}

struct Resume {
    snap: ModesSnap,
    k: i32,
    def: Option<(i32, i32)>,
    att: Option<i32>,
}

pub(crate) struct Job {
    kind: u32,
    att: usize,
    def: usize,
    anchor: ModesSnap,
    anchor_f: i32,
    runs: Vec<RunSpec>,
    out: Vec<[i32; LOOK_RUN_I32]>,
    resume: Option<Resume>,
    ticks: u64,
    ns: u64,
    slices: u32,
    max_slice_ns: u64,
    verify_bad: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct LookPerf {
    pub jobs: u64,
    pub runs: u64,
    pub ticks: u64,
    pub ns: u64,
    pub slices: u64,
    pub max_slice_ns: u64,
    pub verify_bad: u64,
    /// Lookahead ticks run with the sound still installed (must stay 0).
    pub sound_leaks: u64,
}

/// The numpad direction's stick (5 = neutral; diagonals at 90, the rim).
pub(crate) fn numpad_stick(n: i32) -> (i8, i8) {
    let (col, row) = ((n - 1) % 3 - 1, (n - 1) / 3 - 1);
    let m = if col != 0 && row != 0 { 90 } else { 127 };
    ((col * m) as i8, (row * m) as i8)
}

/// (actionable, KO'd) for a port, as the frame meter's cell would classify it (observe's `actionable`), read-only.
fn look_state(port: usize) -> (bool, bool) {
    let Some(fp) = (unsafe { fighter(port as i32) }) else { return (true, false) };
    let m = fp.motion_id;
    // DeadDown .. DeadUpFallHitCameraIce, Sleep, Rebirth
    if (0..=12).contains(&m) {
        return (false, true);
    }
    let hitlag = fp.dmg.x195c_hitlag_frames;
    let iasa = fp.allow_interrupt() != 0;
    // hitstun (DamageFly*, the fighter's `mv` timer as gd_lab_fighter_v1 publishes it): the interrupt flag can be up
    // before it runs out, and a fighter in hitstun cannot act
    let hitstun = if matches!(m, 75..=91) { unsafe { *(core::ptr::addr_of!(fp.mv) as *const f32) } } else { 0.0 };
    if hitlag > 0.0 || hitstun > 0.0 || (is_stun_motion(m) && !iasa) {
        return (false, false);
    }
    if is_idle(m) {
        return (true, false);
    }
    let active = fp.x914.iter().any(|h| h.state != 0);
    (!active && iasa, false)
}

fn policy_pad(pol: Policy, port: usize, err: i8) -> PADStatus {
    let mut p = PADStatus::zeroed();
    p.err = err;
    match pol {
        Policy::Neutral => {}
        Policy::Shield => {
            p.button |= BTN_R;
            p.triggerRight = 255;
        }
        Policy::Di(x, y) => {
            if let Some(fp) = unsafe { fighter(port as i32) } {
                // held through hitlag (the launch reads it as hitlag ends) and while tumbling the stick stays put: the
                // whole lookahead holds it while the fighter is in hitlag or hitstun
                let h = fp.dmg.x195c_hitlag_frames;
                if h > 0.0 || matches!(fp.motion_id, 75..=91) {
                    p.stickX = x;
                    p.stickY = y;
                }
            }
        }
    }
    p
}

fn look_prof() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var("MELEE_LOOK_PROF").is_ok_and(|v| v == "1"))
}

fn suppress(on: bool) {
    melee_engine::gxbridge::shim().set_suppress_draws(on);
    unsafe { melee_runtime::tu::pobj::ext::gx_suppress_draws = on as i32 };
}

impl LabRun {
    /// A snapshot taken with the sound out of its static (see the module note).
    fn spec_save(&mut self) -> ModesSnap {
        let snd = melee_audio::sim::uninstall();
        let s = self.g.save();
        if let Some(x) = snd {
            melee_audio::sim::install(x);
        }
        s
    }

    /// Start a lookahead about the current state (replacing any unfinished one). `def` is the defender (P1 = 0, P2 = 1).
    pub fn look_request(&mut self, kind: u32, def: usize) {
        let def = def.min(1);
        let runs: Vec<RunSpec> = match kind {
            LOOK_HIT => [5, 1, 2, 3, 4, 6, 7, 8, 9]
                .iter()
                .map(|&n| {
                    let (x, y) = numpad_stick(n);
                    RunSpec { tag: n, def: if n == 5 { Policy::Neutral } else { Policy::Di(x, y) }, seed: None }
                })
                .collect(),
            LOOK_SHIELD => vec![RunSpec { tag: 5, def: Policy::Shield, seed: None }],
            LOOK_RNG => (0..LOOK_RUNS as i32).map(|i| RunSpec { tag: i, def: Policy::Neutral, seed: (i > 0).then(|| mix32(0x5EED_0000 + i as u32)) }).collect(),
            _ => vec![RunSpec { tag: 5, def: Policy::Neutral, seed: None }],
        };
        let anchor = self.spec_save();
        let n = runs.len();
        self.look = Some(Job {
            kind,
            att: 1 - def,
            def,
            anchor,
            anchor_f: self.f,
            runs,
            out: Vec::with_capacity(n),
            resume: None,
            ticks: 0,
            ns: 0,
            slices: 0,
            max_slice_ns: 0,
            verify_bad: 0,
        });
        self.look_publish(LS_PENDING);
    }

    /// Drop an unfinished lookahead (the timeline forked under it).
    pub(super) fn look_drop(&mut self) {
        self.look_keep.clear();
        if self.look.take().is_some() {
            self.sh.borrow_mut().look[1] = LS_NONE;
        }
    }

    /// Run the pending lookahead: to completion (`budget` None) or for at most `budget` (at least one tick of progress).
    /// True when it finished.
    pub fn look_run(&mut self, budget: Option<Duration>) -> bool {
        let Some(mut job) = self.look.take() else { return true };
        let t0 = Instant::now();
        // the live tick's frame: detach it before any load forgets it
        if let Some(f) = self.g.take_frame() {
            self.held = Some(f);
        }
        let snd = melee_audio::sim::uninstall();
        let h0 = self.look_verify.then(|| (self.g.state_hash(), self.frame_hash()));
        // the previous slice's live snapshot (the state this one evolved from): alive, the save below is incremental
        let _prev_live = std::mem::take(&mut self.look_keep);
        let mut _old_resume = None;
        let ts = Instant::now();
        let live = self.g.save();
        let save_ns = ts.elapsed().as_nanos() as u64;
        let mut prof = [0u64; 4]; // MELEE_LOOK_PROF: load-in, ticks, save-resume, load-live (ns)
        let errs: [i8; 4] = core::array::from_fn(|p| self.last_pads[p].err);
        let mut ticked = 0u32;
        let mut max_tick = 0u64;
        let mut finished = true;
        'runs: while job.out.len() < job.runs.len() {
            let spec = job.runs[job.out.len()];
            let tli = Instant::now();
            let (mut k, mut d, mut a) = match job.resume.take() {
                Some(r) => {
                    self.g.load(&r.snap);
                    let v = (r.k, r.def, r.att);
                    // kept until the new resume point is saved (incremental against it)
                    _old_resume = Some(r.snap);
                    v
                }
                None => {
                    self.g.load(&job.anchor);
                    if let Some(seed) = spec.seed {
                        melee_runtime::tu::random::global::state().set_logic_seed(seed);
                    }
                    (0, None, None)
                }
            };
            prof[0] += tli.elapsed().as_nanos() as u64;
            // the loads restore these statics: set them after
            melee_engine::engine::set_speculative(true);
            suppress(true);
            loop {
                let (dact, ddead) = look_state(job.def);
                let (aact, adead) = look_state(job.att);
                if d.is_none() {
                    if ddead {
                        d = Some((k, LO_KO));
                    } else if dact {
                        d = Some((k, LO_ACTS));
                    }
                }
                if a.is_none() && (aact || adead) {
                    a = Some(k);
                }
                // a KO'd defender still waits for the attacker's frame (the advantage is "defender's frame - attacker's")
                if (d.is_some() && a.is_some()) || k >= LOOK_CAP || self.g.e.ended {
                    break;
                }
                if let Some(b) = budget {
                    // stop early enough that the slice's tail (save the resume point, load the live state) fits
                    let tail = Duration::from_nanos(self.look_tail_ns);
                    // and the next tick fits too (the slowest tick of this slice as its estimate): the check sits before a
                    // tick, so without it a slice overran the budget by up to one tick
                    if ticked > 0 && t0.elapsed() + tail + Duration::from_nanos(max_tick) >= b {
                        let tr = Instant::now();
                        job.resume = Some(Resume { snap: self.g.save(), k, def: d, att: a });
                        prof[2] += tr.elapsed().as_nanos() as u64;
                        finished = false;
                        break 'runs;
                    }
                }
                let mut pads: [PADStatus; 4] = core::array::from_fn(|p| policy_pad(Policy::Neutral, p, errs[p]));
                pads[job.def] = policy_pad(spec.def, job.def, errs[job.def]);
                if melee_audio::sim::is_installed() {
                    self.look_perf.sound_leaks += 1;
                }
                // the AI hold is sim state (a respawn re-inits cpu.kind): the speculative ticks keep it like a live step
                let ai_kind = self.ai_kind;
                self.ai_hold();
                self.ai_kind = ai_kind;
                let tt = Instant::now();
                self.g.step(pads);
                let dt = tt.elapsed().as_nanos() as u64;
                prof[1] += dt;
                max_tick = max_tick.max(dt);
                k += 1;
                ticked += 1;
                job.ticks += 1;
            }
            let (outcome, df) = d.map_or((LO_CAP, k), |(f, o)| (o, f));
            job.out.push([spec.tag, outcome, df, a.unwrap_or(-1)]);
        }
        let tl = Instant::now();
        self.g.load(&live);
        // the tail estimate: a save plus a load (moving average)
        prof[3] = tl.elapsed().as_nanos() as u64;
        let tail = save_ns + prof[3];
        if look_prof() {
            eprintln!("lab: lookahead slice: save live {:.2} ms, load in {:.2}, {} ticks {:.2}, save resume {:.2}, load live {:.2}, total {:.2}",
                save_ns as f64 / 1e6, prof[0] as f64 / 1e6, ticked, prof[1] as f64 / 1e6, prof[2] as f64 / 1e6, prof[3] as f64 / 1e6, t0.elapsed().as_secs_f64() * 1e3);
        }
        self.look_tail_ns = if self.look_tail_ns == 0 { tail } else { (self.look_tail_ns * 7 + tail) / 8 };
        melee_engine::engine::set_speculative(false);
        suppress(false);
        if let Some(x) = snd {
            melee_audio::sim::install(x);
        }
        if let Some(h) = h0 {
            if (self.g.state_hash(), self.frame_hash()) != h {
                job.verify_bad += 1;
                self.look_perf.verify_bad += 1;
                eprintln!("lab: lookahead left the state different (hash {:08x} -> {:08x})", h.0, self.g.state_hash());
            }
        }
        let ns = t0.elapsed().as_nanos() as u64;
        job.ns += ns;
        job.slices += 1;
        job.max_slice_ns = job.max_slice_ns.max(ns);
        self.look_perf.ticks += ticked as u64;
        self.look_perf.ns += ns;
        self.look_perf.slices += 1;
        self.look_perf.max_slice_ns = self.look_perf.max_slice_ns.max(ns);
        if !finished {
            // the live state is this snapshot until the next tick: the ring capture and the next slice save against it
            self.look_keep.push(live);
        }
        self.look = Some(job);
        if finished {
            self.look_perf.jobs += 1;
            self.look_perf.runs += self.look.as_ref().map_or(0, |j| j.runs.len() as u64);
            self.look_publish(LS_DONE);
            self.done_look = self.look.take();
        } else {
            self.look_publish(LS_RUNNING);
        }
        finished
    }

    fn look_publish(&mut self, state: i32) {
        let Some(j) = self.look.as_ref() else { return };
        let mut s = self.sh.borrow_mut();
        let gen = s.look[4] + (state == LS_DONE) as i32;
        let v = &mut s.look;
        *v = [0; LOOK_I32];
        // the advantage: the neutral (no-DI / shield) run, attacker relative to the defender
        let adv = j.out.iter().find(|o| o[0] == 5 || (j.kind == LOOK_RNG && o[0] == 0)).filter(|o| o[1] == LO_ACTS && o[3] >= 0).map_or(i32::MIN, |o| o[2] - o[3]);
        let head = [
            1, state, j.kind as i32, j.anchor_f, gen, j.att as i32, j.def as i32, adv, j.runs.len() as i32, j.out.len() as i32, j.ticks.min(i32::MAX as u64) as i32,
            (j.ns / 1000).min(i32::MAX as u64) as i32, j.slices as i32, (j.max_slice_ns / 1000) as i32, j.verify_bad, 0,
        ];
        v[..LOOK_HEAD].copy_from_slice(&head);
        for (i, o) in j.out.iter().take(LOOK_RUNS).enumerate() {
            v[LOOK_HEAD + i * LOOK_RUN_I32..LOOK_HEAD + (i + 1) * LOOK_RUN_I32].copy_from_slice(o);
        }
    }

    /// The frame to present: the one held across a lookahead, else the tick's own.
    pub fn take_frame(&mut self) -> Option<Vec<melee_gx::draw::DrawCmd>> {
        let g = self.g.take_frame();
        let h = self.held.take();
        g.or(h)
    }

    /// After a frame (and on a frozen tick): new requests, auto triggers from this frame's events, then the work.
    pub(super) fn look_frame(&mut self, events: bool) {
        let (cfg, budget, req) = {
            let mut s = self.sh.borrow_mut();
            (s.look_cfg, s.look_budget_us, s.look_req.take())
        };
        if let Some((kind, port)) = req {
            self.look_request(kind, port as usize);
        } else if events && cfg & LOOK_AUTO != 0 {
            // the latest hit or shield hit on P1 / P2 this frame
            let trig = self.events.iter().rev().find_map(|e| match e.kind {
                EV_HIT if e.port < 2 => Some((LOOK_HIT, e.port as usize)),
                EV_SHIELD if e.port < 2 => Some((LOOK_SHIELD, e.port as usize)),
                _ => None,
            });
            if let Some((kind, port)) = trig {
                self.look_request(kind, port);
            }
        }
        if self.look.is_some() {
            if self.paused {
                self.look_run(None);
            } else if cfg & LOOK_LIVE != 0 {
                self.look_run(Some(Duration::from_micros(budget as u64)));
            }
        }
    }

    /// The last finished lookahead's runs (tag, outcome, defender frame, attacker frame) and its anchor frame.
    pub fn look_result(&self) -> Option<(i32, Vec<[i32; LOOK_RUN_I32]>)> {
        self.done_look.as_ref().map(|j| (j.anchor_f, j.out.clone()))
    }

    pub fn look_pending(&self) -> bool {
        self.look.is_some()
    }

    /// A digest of every host-side observer `LabRun` keeps (the speculative test: a lookahead must not change it).
    pub fn observer_digest(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut put = |v: u64| {
            for b in v.to_le_bytes() {
                h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
            }
        };
        put(self.f as u64);
        put(self.events.len() as u64);
        put(self.exchange.map_or(u64::MAX, |x| x as u64));
        put(self.rng as u64);
        put(self.log.len() as u64);
        put(self.log_f0 as u64);
        put(self.replay_until.map_or(u64::MAX, |x| x as u64));
        for r in &self.ring {
            put(r.0 as u64);
        }
        for (k, r) in &self.recs {
            put(*k as u64 ^ (r.frames.len() as u64) << 8);
        }
        for p in self.play.iter() {
            put(p.as_ref().map_or(u64::MAX, |p| p.at as u64));
        }
        for o in &self.obs {
            for v in [o.motion, o.motion_frames, o.act_at, o.tl_first, o.tl_last, o.tl_iasa, o.shield_frames, o.timeline.len() as i32] {
                put(v as u64);
            }
            put(o.hitlag.to_bits() as u64 ^ (o.percent.to_bits() as u64) << 32);
            put(o.actionable as u64);
        }
        for d in &self.dummy {
            put(d.st.hit_at as u64 ^ (d.st.di as u64) << 32);
        }
        let p = &self.perf;
        for v in [p.ring_saves, p.slot_saves, p.loads, p.rewinds, p.resim_frames, p.steps] {
            put(v);
        }
        for lp in &self.last_pads {
            put(lp.button as u64 ^ (lp.stickX as u8 as u64) << 16 ^ (lp.stickY as u8 as u64) << 24);
        }
        let s = self.sh.borrow();
        put(s.match_frame as u64);
        put(s.events.len() as u64);
        put(s.history.len() as u64);
        put(s.frame_adv as u64);
        for m in &s.meter {
            put(m.len() as u64);
            for c in m {
                put(*c as u64);
            }
        }
        for ((ch, a), fd) in &s.framedata {
            put((*ch as u64) << 32 ^ *a as u64);
            put(fd.cells.len() as u64 ^ (fd.first_active as u64) << 16 ^ (fd.iasa as u64) << 40);
        }
        for f in s.fighters.iter().flatten() {
            for c in f.chunks(8) {
                put(c.iter().fold(0u64, |a, &b| a << 8 | b as u64));
            }
        }
        h
    }
}

impl LabRun {
    /// Whether `port` can act now or is KO'd (the lookahead's stop classification; the tests compare against it).
    pub fn actionable_now(&self, port: usize) -> bool {
        let (act, dead) = look_state(port);
        act || dead
    }
}
