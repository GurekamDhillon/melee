//! LAB roadmap item 2 (docs/rust-port/notes/lab-theorycraft.md 4b, 8 C, J, K): drills, the adaptive mixup trainer, the
//! session log, mirror and jitter on load.
//!
//! * A **drill** is one text file ([`parse`]): where an attempt starts (the state when the drill starts, or a save
//!   slot), the dummy's config, the reaction the trainer picks per attempt with its options and starting weights, the
//!   trigger that opens the scoring window, the success predicate, the par (frames late are counted past it), a timeout
//!   and the loop delay. Five ship compiled in (`drills/*.drill`); more load from `<lab dir>/drills/*.drill` (offline).
//! * The **runner** is sim-side ([`LabRun`]): `OP_DRILL` starts it; each attempt reloads the start state, varies it
//!   (mirror / jitter), draws the option from the weights with the lab RNG, and the [`Judge`] scores the attempt from
//!   the frame's events. With `OP_ADAPT` on, a failure adds weight to that option and a success takes one away. Every
//!   step is a function of the commands, the events and the lab RNG, so both peers of the online LAB run the same
//!   attempts with the same picks and weights from the one `OP_DRILL` command.
//! * The **session log** ([`SessionLog`]) is presentation only: one line per attempt and a summary per drill, appended
//!   to `drill-log.tsv` in the lab folder; the trend of a drill is read back from the summaries. It never feeds the sim.

use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const BUILTIN: [(&str, &str); DRILL_BUILTIN] = [
    ("shield_punish", include_str!("drills/shield_punish.drill")),
    ("tech_chase", include_str!("drills/tech_chase.drill")),
    ("hit_confirm", include_str!("drills/hit_confirm.drill")),
    ("ledge", include_str!("drills/ledge.drill")),
    ("oos", include_str!("drills/oos.drill")),
];

/// The adapted "kind" for drills that pick among the Record slots.
pub const ADAPT_REC: i32 = 100;
/// The most options a drill may list (the status carries 8).
pub const MAX_OPTS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    /// The attempt start.
    Start,
    /// The dummy techs (or misses the tech).
    DummyTech,
    /// The dummy leaves shield.
    DummyOos,
    /// The dummy leaves its ledge hang.
    DummyLedge,
    /// The dummy shields a hit.
    DummyShield,
    /// The user hits the dummy or its shield (hit-confirm: the branch is remembered).
    DummyHitOrShield,
    /// The user shields a hit.
    UserShield,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Success {
    /// The user hits the dummy within n frames of the trigger.
    Hit(i32),
    /// Hit branch: another hit within n; shield branch: not hit for m frames.
    Confirm(i32, i32),
    /// Not hit for n frames.
    Safe(i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Start {
    Here,
    Slot(u32),
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Vary {
    pub flags: u16,
    pub pct: u32,
    pub x: u32,
}

#[derive(Clone, Debug)]
pub struct Drill {
    pub name: String,
    pub about: String,
    pub start: Start,
    pub cpu: (i32, i32),
    /// (kind, option, param) as OP_REACTION.
    pub react: Vec<(i32, i32, u32)>,
    /// RK_* or ADAPT_REC; -1: no pick (a fixed setup).
    pub adapt: i32,
    /// (label, option value; for ADAPT_REC the Record slot 0..3).
    pub options: Vec<(String, i32)>,
    pub weights: Vec<u8>,
    pub trigger: Trigger,
    pub success: Success,
    pub par: i32,
    pub timeout: i32,
    pub after: i32,
    pub percent: Option<f32>,
    pub vary: Vary,
}

fn kind_of(s: &str) -> Option<i32> {
    Some(match s {
        "di" => RK_DI,
        "sdi" => RK_SDI,
        "tech" => RK_TECH,
        "getup" => RK_GETUP,
        "ledge" => RK_LEDGE,
        "shield" => RK_SHIELD,
        "oos" => RK_OOS,
        "counter" => RK_COUNTER,
        "recording" => ADAPT_REC,
        _ => return None,
    })
}

/// An option's value by its name for a reaction kind (the names the menus use, lower case with underscores).
pub fn option_of(kind: i32, s: &str) -> Option<i32> {
    if s == "none" {
        return Some(RO_NONE);
    }
    if s == "random" {
        return Some(if kind == RK_OOS { RO_OOS_RANDOM } else if kind == RK_SHIELD { RO_BLOCK_RANDOM } else { RO_RANDOM });
    }
    let v = match kind {
        RK_DI | RK_SDI => match s { "in" => RO_IN, "out" => RO_OUT, "away" => RO_AWAY, "toward" => RO_TOWARD, "up" => RO_UP, "down" => RO_DOWN, _ => return None },
        RK_TECH => match s { "in_place" => RO_TECH_IN_PLACE, "roll_in" => RO_TECH_ROLL_IN, "roll_away" => RO_TECH_ROLL_AWAY, "miss" => RO_TECH_MISS, _ => return None },
        RK_GETUP => match s { "stand" => RO_GETUP_STAND, "roll_in" => RO_GETUP_ROLL_IN, "roll_away" => RO_GETUP_ROLL_AWAY, "attack" => RO_GETUP_ATTACK, _ => return None },
        RK_LEDGE => match s { "stand" => RO_GETUP_STAND, "roll" => RO_GETUP_ROLL_IN, "attack" => RO_GETUP_ATTACK, "jump" => RO_LEDGE_JUMP, "drop" => RO_LEDGE_DROP, _ => return None },
        RK_SHIELD => match s { "all" => RO_BLOCK_ALL, "after_first" => RO_BLOCK_AFTER_FIRST, _ => return None },
        RK_OOS => match s { "jump" => RO_OOS_JUMP, "nair" => RO_OOS_NAIR, "grab" => RO_OOS_GRAB, "upb" => RO_OOS_UPB, "spotdodge" => RO_OOS_SPOTDODGE, "roll" => RO_OOS_ROLL, _ => return None },
        RK_COUNTER => match s { "jump" => RO_COUNTER_JUMP, "attack" => RO_COUNTER_ATTACK, "shield" => RO_COUNTER_SHIELD, "spotdodge" => RO_COUNTER_SPOTDODGE, "grab" => RO_COUNTER_GRAB, _ => return None },
        ADAPT_REC => match s { "rec1" => 0, "rec2" => 1, "rec3" => 2, "rec4" => 3, _ => return None },
        _ => return None,
    };
    Some(v)
}

/// A label for the summary: "roll_away" -> "Roll away", "rec2" -> "Record 2".
fn label(s: &str) -> String {
    if let Some(n) = s.strip_prefix("rec") {
        return format!("Record {n}");
    }
    let t = s.replace('_', " ").replace("upb", "up-B").replace("spotdodge", "spot-dodge");
    let mut c = t.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

/// Parse a drill file: `key: value` lines, `#` comments. Unknown keys are errors (a typo should not silently do nothing).
pub fn parse(src: &str) -> Result<Drill, String> {
    let mut d = Drill {
        name: String::new(), about: String::new(), start: Start::Here, cpu: (CPU_NONE, 0), react: Vec::new(), adapt: -1, options: Vec::new(), weights: Vec::new(),
        trigger: Trigger::Start, success: Success::Hit(30), par: 0, timeout: 600, after: 45, percent: None, vary: Vary::default(),
    };
    let num = |w: Option<&str>, line: usize| -> Result<i32, String> { w.and_then(|w| w.parse().ok()).ok_or(format!("line {line}: a number expected")) };
    let mut opt_names: Vec<String> = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let ln = i + 1;
        let l = raw.split('#').next().unwrap_or("").trim();
        if l.is_empty() {
            continue;
        }
        let (k, v) = l.split_once(':').ok_or(format!("line {ln}: `key: value` expected"))?;
        let v = v.trim();
        let mut w = v.split_whitespace();
        match k.trim() {
            "name" => d.name = v.to_string(),
            "about" => d.about = v.to_string(),
            "start" => {
                d.start = match w.next() {
                    Some("here") => Start::Here,
                    Some("slot") => Start::Slot((num(w.next(), ln)? - 1).clamp(0, SLOTS as i32 - 1) as u32),
                    _ => return Err(format!("line {ln}: start: here | slot N")),
                }
            }
            "cpu" => {
                d.cpu = match w.next() {
                    Some("none") => (CPU_NONE, 0),
                    Some("stand") => (CPU_STAND, 0),
                    Some("crouch") => (CPU_CROUCH, 0),
                    Some("jump") => (CPU_JUMP, 0),
                    Some("shield") => (CPU_SHIELD, 0),
                    Some("level") => (CPU_LEVEL, num(w.next(), ln)?.clamp(1, 9)),
                    _ => return Err(format!("line {ln}: cpu: none | stand | crouch | jump | shield | level N")),
                }
            }
            "react" => {
                let kind = w.next().and_then(kind_of).filter(|&k| k != ADAPT_REC).ok_or(format!("line {ln}: react: a reaction kind"))?;
                let o = w.next().and_then(|o| option_of(kind, o)).ok_or(format!("line {ln}: react: an option of that kind"))?;
                let mut p = ReactParam::default();
                while let Some(key) = w.next() {
                    let n = num(w.next(), ln)?.clamp(0, 255) as u8;
                    match key {
                        "delay" => p.delay = n,
                        "strength" => p.strength = n,
                        "chance" => p.chance = n,
                        _ => return Err(format!("line {ln}: react parameters: delay / strength / chance N")),
                    }
                }
                d.react.push((kind, o, p.to_u32()));
            }
            "adapt" => d.adapt = w.next().and_then(kind_of).ok_or(format!("line {ln}: adapt: a reaction kind or recording"))?,
            "options" => opt_names = w.map(str::to_string).collect(),
            "weights" => d.weights = w.map(|x| x.parse::<u8>().map(|v| v.clamp(0, 15))).collect::<Result<_, _>>().map_err(|_| format!("line {ln}: weights: numbers 0-15"))?,
            "trigger" => {
                d.trigger = match v {
                    "start" => Trigger::Start,
                    "dummy_tech" => Trigger::DummyTech,
                    "dummy_oos" => Trigger::DummyOos,
                    "dummy_ledge" => Trigger::DummyLedge,
                    "dummy_shield" => Trigger::DummyShield,
                    "dummy_hit_or_shield" => Trigger::DummyHitOrShield,
                    "user_shield" => Trigger::UserShield,
                    _ => return Err(format!("line {ln}: unknown trigger `{v}`")),
                }
            }
            "success" => {
                d.success = match w.next() {
                    Some("hit") => Success::Hit(num(w.next(), ln)?),
                    Some("confirm") => Success::Confirm(num(w.next(), ln)?, num(w.next(), ln)?),
                    Some("safe") => Success::Safe(num(w.next(), ln)?),
                    _ => return Err(format!("line {ln}: success: hit N | confirm N M | safe N")),
                }
            }
            "par" => d.par = num(w.next(), ln)?,
            "timeout" => d.timeout = num(w.next(), ln)?.max(30),
            "loop" => {
                if w.next() != Some("after") {
                    return Err(format!("line {ln}: loop: after N"));
                }
                d.after = num(w.next(), ln)?.clamp(1, 600);
            }
            "percent" => d.percent = Some(num(w.next(), ln)?.clamp(0, 999) as f32),
            "mirror" => {
                d.vary.flags = match v {
                    "off" => 0,
                    "on" => VARY_MIRROR,
                    "random" => VARY_MIRROR_RANDOM,
                    _ => return Err(format!("line {ln}: mirror: off | on | random")),
                }
            }
            "jitter_percent" => d.vary.pct = num(w.next(), ln)?.clamp(0, 300) as u32,
            "jitter_x" => d.vary.x = num(w.next(), ln)?.clamp(0, 200) as u32,
            other => return Err(format!("line {ln}: unknown key `{other}`")),
        }
    }
    if d.name.is_empty() {
        return Err("no name".into());
    }
    if d.adapt >= 0 {
        for n in &opt_names {
            let v = option_of(d.adapt, n).ok_or(format!("option `{n}` is not one of the adapted kind's"))?;
            d.options.push((label(n), v));
        }
        if d.options.is_empty() || d.options.len() > MAX_OPTS {
            return Err(format!("options: 1 to {MAX_OPTS}"));
        }
        d.weights.resize(d.options.len(), 4);
    }
    Ok(d)
}

/// The compiled-in drills, then `<lab dir>/drills/*.drill` in name order (a file that does not parse is logged and
/// skipped).
pub fn load_all() -> Vec<Drill> {
    let mut v: Vec<Drill> = BUILTIN.iter().map(|(n, s)| parse(s).unwrap_or_else(|e| panic!("built-in drill {n}: {e}"))).collect();
    if let Some(dir) = lab_dir().map(|d| d.join("drills")) {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "drill")).collect()).unwrap_or_default();
        files.sort();
        for f in files {
            match std::fs::read_to_string(&f).map_err(|e| e.to_string()).and_then(|s| parse(&s)) {
                Ok(d) => v.push(d),
                Err(e) => eprintln!("lab: drill {}: {e} (skipped)", f.display()),
            }
        }
    }
    v
}

/// The LAB's user folder: `MELEE_LAB_DIR`, else `<YOUGAME_DATA>/lab`, else `lab` beside settings.cfg.
pub fn lab_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("MELEE_LAB_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = melee_platform::yougame::data_dir() {
        return Some(PathBuf::from(d).join("lab"));
    }
    melee_data::settings::default_path().parent().map(|p| p.join("lab"))
}

// ------------------------------------------------------------------------------------------------ judge

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Success { late: i32 },
    Missed,
    Punished,
    /// The trigger never came (not scored).
    NoTrigger,
}

/// One frame as the judge sees it: the events and the dummy's action before and after the frame.
pub struct JFrame<'a> {
    pub f: i32,
    pub ev: &'a [LabEvent],
    pub d_prev: i32,
    pub d_motion: i32,
    /// The user's action after the frame (grabbed or thrown counts as punished).
    pub u_motion: i32,
}

/// CapturePulledHi .. the last Thrown state (ftCo_MS_*): the user is grabbed or thrown.
const CAPTURED_FIRST: i32 = 223;
const CAPTURED_LAST: i32 = 243;

/// Scores one attempt from the frames' events (a pure function of them: the lab-check feeds it scripted streams).
#[derive(Clone, Copy, Debug)]
pub struct Judge {
    pub d: usize,
    pub u: usize,
    trigger: Trigger,
    success: Success,
    par: i32,
    timeout: i32,
    start: i32,
    at: Option<i32>,
    branch_hit: bool,
}

/// The ledge hang (CliffWait).
const CLIFF_WAIT: i32 = 253;

impl Judge {
    pub fn new(dr: &Drill, d: usize, u: usize, f: i32) -> Judge {
        Judge { d, u, trigger: dr.trigger, success: dr.success, par: dr.par, timeout: dr.timeout, start: f, at: None, branch_hit: false }
    }

    /// Forget the attempt so far (the user loaded or rewound): wait for the trigger again from `f`.
    pub fn reset(&mut self, f: i32) {
        self.start = f;
        self.at = None;
        self.branch_hit = false;
    }

    /// 0 waiting for the trigger, 1 in the window.
    pub fn phase(&self) -> i32 {
        self.at.is_some() as i32
    }

    pub fn feed(&mut self, fr: &JFrame) -> Option<Verdict> {
        let (d, u) = (self.d as u8, self.u as u8);
        let hit_d = fr.ev.iter().any(|e| e.kind == EV_HIT && e.port == d && (e.a < 0 || e.a == u as i32));
        let hit_u = fr.ev.iter().any(|e| e.kind == EV_HIT && e.port == u);
        let Some(t) = self.at else {
            let fired = match self.trigger {
                Trigger::Start => true,
                Trigger::DummyTech => fr.ev.iter().any(|e| e.kind == EV_TECH && e.port == d),
                Trigger::DummyOos => fr.ev.iter().any(|e| e.kind == EV_OOS && e.port == d),
                Trigger::DummyLedge => fr.d_prev == CLIFF_WAIT && fr.d_motion != CLIFF_WAIT,
                Trigger::DummyShield => fr.ev.iter().any(|e| e.kind == EV_SHIELD && e.port == d),
                Trigger::DummyHitOrShield => {
                    self.branch_hit = hit_d;
                    hit_d || fr.ev.iter().any(|e| e.kind == EV_SHIELD && e.port == d)
                }
                Trigger::UserShield => fr.ev.iter().any(|e| e.kind == EV_SHIELD && e.port == u),
            };
            if fired {
                self.at = Some(fr.f);
                return None;
            }
            return (fr.f - self.start >= self.timeout).then_some(Verdict::NoTrigger);
        };
        let dt = fr.f - t;
        // hit or grabbed (a trade counts against the user)
        if hit_u || (CAPTURED_FIRST..=CAPTURED_LAST).contains(&fr.u_motion) {
            return Some(Verdict::Punished);
        }
        let hit_window = match self.success {
            Success::Hit(n) => Some(n),
            Success::Confirm(n, _) if self.branch_hit => Some(n),
            _ => None,
        };
        if let Some(n) = hit_window {
            return if hit_d { Some(Verdict::Success { late: (dt - self.par).max(0) }) } else { (dt > n).then_some(Verdict::Missed) };
        }
        let m = match self.success {
            Success::Confirm(_, m) | Success::Safe(m) | Success::Hit(m) => m,
        };
        (dt >= m).then_some(Verdict::Success { late: 0 })
    }
}

/// The adaptive update (4b, "weak spots"): a failure against option i adds `rate`, a success takes one away; weights
/// stay in 1..=15 so no option ever disappears.
pub fn adapt(w: &mut [u8], i: usize, success: bool, rate: u8) {
    if let Some(x) = w.get_mut(i) {
        *x = if success { x.saturating_sub(1).max(1) } else { (*x + rate.max(1)).min(15) };
    }
}

/// A weighted draw over `w` (all zero: uniform) from the lab RNG.
pub fn pick(rng: &mut u32, w: &[u8]) -> usize {
    let tot: u32 = w.iter().map(|&x| x as u32).sum();
    if w.is_empty() {
        return 0;
    }
    if tot == 0 {
        return (xorshift(rng) % w.len() as u32) as usize;
    }
    let mut k = xorshift(rng) % tot;
    for (i, &x) in w.iter().enumerate() {
        if k < x as u32 {
            return i;
        }
        k -= x as u32;
    }
    0
}

// ------------------------------------------------------------------------------------------------ session log

/// The session log (presentation: written as attempts finish, never read by the sim).
pub struct SessionLog {
    path: Option<PathBuf>,
    session: u64,
    /// Per drill name: attempts, successes, late sum, late count.
    per: BTreeMap<String, (u32, u32, i64, u32)>,
    written: BTreeMap<String, (u32, u32, i64, u32)>,
}

impl SessionLog {
    pub fn new() -> SessionLog {
        let session = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        SessionLog { path: lab_dir().map(|d| d.join("drill-log.tsv")), session, per: BTreeMap::new(), written: BTreeMap::new() }
    }

    fn append(&self, line: &str) {
        let Some(p) = &self.path else { return };
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        use std::io::Write;
        let new = !p.exists();
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            if new {
                let _ = writeln!(f, "# LAB drill log: attempt rows (session, drill, n, option, verdict, frames late, match frame) and #summary rows (session, drill, attempts, successes, mean frames late)");
            }
            let _ = writeln!(f, "{line}");
        }
    }

    pub fn attempt(&mut self, drill: &str, n: u32, option: &str, v: Verdict, frame: i32) {
        let e = self.per.entry(drill.to_string()).or_default();
        e.0 += 1;
        let (verdict, late) = match v {
            Verdict::Success { late } => {
                e.1 += 1;
                e.2 += late as i64;
                e.3 += 1;
                ("success", late)
            }
            Verdict::Missed => ("missed", -1),
            Verdict::Punished => ("punished", -1),
            Verdict::NoTrigger => ("void", -1),
        };
        self.append(&format!("{}\t{drill}\t{n}\t{option}\t{verdict}\t{late}\t{frame}", self.session));
    }

    /// The per-drill summary rows for what changed since the last call (a drill stops, the match ends).
    pub fn summary(&mut self) {
        let rows: Vec<String> = self
            .per
            .iter()
            .filter(|(k, v)| self.written.get(*k) != Some(v) && v.0 > 0)
            .map(|(k, v)| format!("#summary\t{}\t{k}\t{}\t{}\t{:.1}", self.session, v.0, v.1, if v.3 == 0 { -1.0 } else { v.2 as f64 / v.3 as f64 }))
            .collect();
        for r in rows {
            self.append(&r);
        }
        self.written = self.per.clone();
    }

    /// Earlier sessions of a drill (the last summary row of each session): (success per mille, attempts), oldest first.
    pub fn trend(&self, drill: &str) -> Vec<(i32, i32)> {
        let Some(s) = self.path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) else { return Vec::new() };
        let mut by: Vec<(u64, (i32, i32))> = Vec::new();
        for l in s.lines() {
            let c: Vec<&str> = l.split('\t').collect();
            if c.len() >= 5 && c[0] == "#summary" && c[2] == drill {
                let (Ok(ses), Ok(n), Ok(ok)) = (c[1].parse::<u64>(), c[3].parse::<i32>(), c[4].parse::<i32>()) else { continue };
                if ses == self.session {
                    continue;
                }
                let v = (ok * 1000 / n.max(1), n);
                match by.iter_mut().find(|x| x.0 == ses) {
                    Some(x) => x.1 = v,
                    None => by.push((ses, v)),
                }
            }
        }
        by.into_iter().map(|x| x.1).collect()
    }
}

impl Drop for SessionLog {
    fn drop(&mut self) {
        self.summary();
    }
}

// ------------------------------------------------------------------------------------------------ the runner (sim)

pub struct Run {
    pub idx: usize,
    pub d: usize,
    pub u: usize,
    snap: (i32, ModesSnap),
    recs: [u8; 4],
    pub weights: Vec<u8>,
    /// Per option: attempts, successes.
    pub tries: Vec<(u32, u32)>,
    pub judge: Judge,
    pub pick: usize,
    /// Frames counted while the drill runs; `restart` is the clock of the next attempt.
    clock: i32,
    restart: Option<i32>,
    prev_motion: i32,
    pub n: u32,
    pub ok: u32,
    pub last: (i32, i32, i32),
    pub seq: u32,
    pub streak: u32,
    pub best: u32,
    pub late_sum: i32,
    pub late_n: u32,
}

impl LabRun {
    /// Test hook (lab-check): add a drill after the loaded ones; its index.
    pub fn drill_add(&mut self, d: Drill) -> usize {
        self.drills.push(d);
        self.sh.borrow_mut().drill_names.push(self.drills.last().unwrap().name.clone());
        self.drills.len() - 1
    }

    /// Test hook: the running drill's weights, attempts and successes.
    pub fn drill_state(&self) -> Option<(Vec<u8>, u32, u32)> {
        self.drill.as_ref().map(|r| (r.weights.clone(), r.n, r.ok))
    }

    fn set_percent(&mut self, port: usize, v: f32) {
        melee_fighter::tu::player::Player_SetHUDDamage(port as i32, v as i32);
        if let Some(fp) = unsafe { fighter(port as i32) } {
            fp.dmg.x1830_percent = v;
        }
    }

    /// Load a state as a fork (the forward history goes), as OP_LOAD does.
    pub(super) fn fork_load(&mut self, fr: i32, s: &ModesSnap) {
        let t = std::time::Instant::now();
        self.load_state(s);
        self.perf.loads += 1;
        self.perf.load_ns += t.elapsed().as_nanos() as u64;
        self.f = fr;
        self.ring.clear();
        self.log.clear();
        self.log_f0 = fr;
        self.replay_until = None;
        self.exchange = None;
        self.obs = Default::default();
        self.observe(false);
    }

    /// Mirror and jitter on load (8 K): flip every fighter across the stage centre (x = 0), move `port` by up to `x`
    /// and change its percent by up to `pct`, drawn from the lab RNG. Off (all zero) it draws nothing and changes
    /// nothing, so a plain load stays hash-equal.
    pub(super) fn vary_apply(&mut self, v: Vary, port: usize) {
        if v == Vary::default() {
            return;
        }
        let mirror = v.flags & VARY_MIRROR != 0 || (v.flags & VARY_MIRROR_RANDOM != 0 && self.rnd(2) == 1);
        if mirror {
            for p in 0..4 {
                if let Some(fp) = unsafe { fighter(p) } {
                    for q in [&mut fp.cur_pos, &mut fp.prev_pos, &mut fp.coll_data.cur_pos, &mut fp.coll_data.prev_pos, &mut fp.coll_data.last_pos] {
                        q.x = -q.x;
                    }
                    fp.self_vel.x = -fp.self_vel.x;
                    fp.x8c_kb_vel.x = -fp.x8c_kb_vel.x;
                    fp.gr_vel = -fp.gr_vel;
                    fp.xF0_ground_kb_vel = -fp.xF0_ground_kb_vel;
                    fp.facing_dir = -fp.facing_dir;
                    fp.facing_dir1 = -fp.facing_dir1;
                }
            }
        }
        let mut dp = 0;
        if v.pct > 0 {
            dp = self.rnd(v.pct * 2 + 1) as i32 - v.pct as i32;
            if let Some(cur) = unsafe { fighter(port as i32) }.map(|fp| fp.dmg.x1830_percent) {
                self.set_percent(port, (cur + dp as f32).max(0.0));
            }
        }
        let mut dx = 0;
        if v.x > 0 {
            dx = self.rnd(v.x * 2 + 1) as i32 - v.x as i32;
            if let Some(fp) = unsafe { fighter(port as i32) } {
                for q in [&mut fp.cur_pos, &mut fp.prev_pos, &mut fp.coll_data.cur_pos, &mut fp.coll_data.prev_pos, &mut fp.coll_data.last_pos] {
                    q.x += dx as f32;
                }
            }
        }
        self.obs = Default::default();
        self.observe(false);
        self.note(format!("lab: frame {}: load varied: mirror {}, P{} percent {dp:+}, x {dx:+} (forks the timeline)", self.f, mirror as u8, port + 1));
    }

    pub(super) fn drill_cmd(&mut self, c: &LabCmd) {
        let port = (c.port & 3) as usize;
        match c.op {
            OP_DRILL if c.a == DRILL_STOP => self.drill_stop(),
            OP_DRILL => self.drill_start(c.a as usize, port, c.b.to_le_bytes()),
            OP_ADAPT => {
                self.adapt_mode = (c.a, (c.b as u8).clamp(1, 15));
                self.note(format!("lab: frame {}: adaptive trainer mode {} rate {}", self.f, c.a, c.b));
            }
            OP_LOAD_VARY => {
                self.vary = (Vary { flags: c.a & 3, pct: c.b.min(300), x: c.c.min(200) }, port);
                self.note(format!("lab: frame {}: load variation {:?} on P{}", self.f, self.vary.0, port + 1));
            }
            _ => {}
        }
    }

    fn drill_start(&mut self, idx: usize, d: usize, recs: [u8; 4]) {
        self.drill_stop();
        let Some(dr) = self.drills.get(idx).cloned() else {
            self.drill_err = DE_NO_DRILL;
            return;
        };
        let mut weights = dr.weights.clone();
        if dr.adapt == ADAPT_REC {
            for (i, o) in dr.options.iter().enumerate() {
                let h = recs[o.1 as usize & 3];
                if h == 0 || !self.recs.contains_key(&h) {
                    weights[i] = 0;
                }
            }
            if weights.iter().all(|&w| w == 0) {
                self.drill_err = DE_NEEDS_REC;
                self.note(format!("lab: frame {}: drill `{}` needs a recording in Record slots 1-4", self.f, dr.name));
                return;
            }
        }
        self.drill_err = DE_OK;
        let snap = match dr.start {
            Start::Slot(n) => self.slots.get(n as usize).cloned().flatten(),
            Start::Here => None,
        }
        .unwrap_or_else(|| (self.f, self.save_state()));
        let u = if d == 0 { 1 } else { 0 };
        // the dummy's config, as the commands would set it
        self.apply(&LabCmd::new(OP_CPU, d as u8, dr.cpu.0 as u16, dr.cpu.1 as u32, 0, 0));
        for k in 0..RK_COUNT {
            self.dummy[d].react[k] = React::default();
        }
        for &(k, o, p) in &dr.react {
            self.apply(&LabCmd::new(OP_REACTION, d as u8, k as u16, o as u32, p, 0));
        }
        self.note(format!("lab: frame {}: drill `{}` started on dummy P{} ({} options)", self.f, dr.name, d + 1, dr.options.len()));
        let n_opt = dr.options.len();
        {
            let mut s = self.sh.borrow_mut();
            s.drill_opts = dr.options.iter().map(|o| o.0.clone()).collect();
            s.drill_trend = self.trlog.trend(&dr.name);
        }
        self.drill = Some(Run {
            idx, d, u, snap, recs, weights, tries: vec![(0, 0); n_opt], judge: Judge::new(&dr, d, u, self.f), pick: 0, clock: 0, restart: Some(0), prev_motion: -1,
            n: 0, ok: 0, last: (DV_NONE, 0, 0), seq: 0, streak: 0, best: 0, late_sum: 0, late_n: 0,
        });
        self.drill_begin();
    }

    pub(super) fn drill_stop(&mut self) {
        if let Some(r) = self.drill.take() {
            if r.d < 4 {
                self.play[r.d] = None;
            }
            self.note(format!("lab: frame {}: drill stopped after {} attempts ({} successes)", self.f, r.n, r.ok));
            self.trlog.summary();
        }
    }

    /// An attempt: reload the start, vary it, pick the option.
    fn drill_begin(&mut self) {
        let Some(mut r) = self.drill.take() else { return };
        let dr = self.drills[r.idx].clone();
        let (fr, s) = r.snap.clone();
        self.fork_load(fr, &s);
        let v = if dr.vary != Vary::default() { dr.vary } else { self.vary.0 };
        self.vary_apply(v, r.d);
        if let Some(p) = dr.percent {
            self.set_percent(r.d, p);
        }
        r.pick = if dr.adapt >= 0 { pick(&mut self.rng, &r.weights) } else { 0 };
        if let Some(&(_, o)) = dr.options.get(r.pick) {
            if dr.adapt == ADAPT_REC {
                let h = r.recs[o as usize & 3];
                self.play[r.d] = Some(Play { rec: h, flags: 0, at: 0, set: None, once: true });
            } else if let Some(x) = self.dummy[r.d].react.get_mut(dr.adapt as usize) {
                x.option = o;
            }
        }
        r.judge = Judge::new(&dr, r.d, r.u, self.f);
        r.restart = None;
        r.prev_motion = unsafe { fighter(r.d as i32) }.map_or(-1, |fp| fp.motion_id);
        self.note(format!("lab: frame {}: drill attempt {}: {}", self.f, r.n + 1, dr.options.get(r.pick).map_or("-", |o| o.0.as_str())));
        self.drill = Some(r);
        self.drew = true;
    }

    /// Before a stepped frame: start the next attempt when its time has come.
    pub(super) fn drill_pre(&mut self) {
        if self.drill.as_ref().is_some_and(|r| r.restart.is_some_and(|t| r.clock >= t)) {
            self.drill_begin();
        }
    }

    /// After a stepped frame: score it.
    pub(super) fn drill_post(&mut self) {
        let Some(mut r) = self.drill.take() else { return };
        r.clock += 1;
        let m = unsafe { fighter(r.d as i32) }.map_or(-1, |fp| fp.motion_id);
        if r.restart.is_none() {
            let um = unsafe { fighter(r.u as i32) }.map_or(-1, |fp| fp.motion_id);
            let fr = JFrame { f: self.f, ev: &self.events, d_prev: r.prev_motion, d_motion: m, u_motion: um };
            if let Some(v) = r.judge.feed(&fr) {
                let dr = &self.drills[r.idx];
                let after = dr.after;
                if v == Verdict::NoTrigger {
                    r.restart = Some(r.clock);
                } else {
                    let ok = matches!(v, Verdict::Success { .. });
                    r.n += 1;
                    if let Some(t) = r.tries.get_mut(r.pick) {
                        t.0 += 1;
                        t.1 += ok as u32;
                    }
                    if ok {
                        r.ok += 1;
                        r.streak += 1;
                        r.best = r.best.max(r.streak);
                    } else {
                        r.streak = 0;
                    }
                    let (code, late) = match v {
                        Verdict::Success { late } => {
                            r.late_sum += late;
                            r.late_n += 1;
                            (DV_SUCCESS, late)
                        }
                        Verdict::Punished => (DV_PUNISHED, -1),
                        _ => (DV_MISSED, -1),
                    };
                    r.last = (code, late, r.pick as i32);
                    r.seq += 1;
                    if self.adapt_mode.0 == ADAPT_WEAK && dr.adapt >= 0 {
                        adapt(&mut r.weights, r.pick, ok, self.adapt_mode.1);
                    }
                    let name = dr.name.clone();
                    let opt = dr.options.get(r.pick).map_or("-".to_string(), |o| o.0.clone());
                    if !melee_engine::engine::speculative() {
                        self.trlog.attempt(&name, r.n, &opt, v, self.f);
                    }
                    self.note(format!("lab: frame {}: drill attempt {} ({opt}): {v:?}; weights {:?}", self.f, r.n, r.weights));
                    r.restart = Some(r.clock + after);
                }
            }
        }
        r.prev_motion = m;
        self.drill = Some(r);
    }

    /// The user loaded or rewound during a drill: the attempt so far does not count.
    pub(super) fn drill_void(&mut self) {
        let f = self.f;
        if let Some(r) = self.drill.as_mut() {
            r.judge.reset(f);
            r.prev_motion = unsafe { fighter(r.d as i32) }.map_or(-1, |fp| fp.motion_id);
        }
    }

    pub(super) fn drill_publish(&self, s: &mut LabShared) {
        let v = &mut s.drill;
        *v = [0; DRILL_I32];
        v[15] = self.drill_err;
        v[10] = self.adapt_mode.0 as i32;
        let Some(r) = &self.drill else { return };
        v[0] = 1;
        v[1] = r.idx as i32;
        v[2] = if r.restart.is_some() { 2 } else { r.judge.phase() };
        v[3] = r.n as i32;
        v[4] = r.ok as i32;
        v[5] = r.seq as i32;
        v[6] = r.last.0;
        v[7] = r.last.1;
        v[8] = r.last.2;
        v[9] = r.weights.len() as i32;
        v[11] = r.streak as i32;
        v[12] = r.best as i32;
        v[13] = r.late_sum;
        v[14] = r.late_n as i32;
        for i in 0..r.weights.len().min(MAX_OPTS) {
            v[16 + i] = r.weights[i] as i32;
            v[24 + i] = r.tries[i].0 as i32;
            v[32 + i] = r.tries[i].1 as i32;
        }
    }
}
