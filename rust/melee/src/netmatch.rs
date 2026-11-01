//! The native VS match under the rollback session (`melee_net::session`): `RbGame` for
//! [`vs_match::VsMatch`], the in-process SyncTest (`melee synctest`) and the netplay peer
//! (`melee netplay`).
//!
//! Frame mapping: the session's first frame is `FIRST` (-123, Slippi's); the match's golden frame
//! `g` (0 = the first logged frame) is session frame `FIRST + g`, so `frame()` reads `FIRST - 1`
//! before the first step, as `gw_Replay_Frame` does once the match is armed.

use crate::vs_match::rollback::{script_input, RbSnapshot};
use crate::vs_match::VsMatch;
use melee_net::pad::RbInput;
use melee_net::session::{RbGame, FIRST, SLOTS};

pub struct NetMatch {
    pub vs: VsMatch,
    /// Ports driven by pads (the scenario has two humans).
    pub humans: [bool; 4],
    pub live: bool,
    /// Live online RNG rule (`gw_Replay_ChecksumSeed` / `ResyncSeed`): every frame starts from
    /// `seed + ((frame + 123) << 16)`, so a draw only one peer makes cannot carry over. None = the
    /// offline free-running seed (the golden CSVs).
    pub live_seed: Option<u32>,
    /// The after-frame neutral hash with the sim's own seed (what MELEE_NEUTRAL_HASHLOG writes),
    /// by golden frame; resimulation overwrites, so the final values are the played timeline.
    pub raw_log: std::collections::BTreeMap<i32, String>,
}

impl NetMatch {
    pub fn new(vs: VsMatch) -> Self {
        NetMatch { vs, humans: [true, true, false, false], live: false, live_seed: None, raw_log: Default::default() }
    }
}

/// `gw_Replay_ChecksumSeed` in live mode: the seed a frame starts from.
pub fn live_frame_seed(seed: u32, frame: i32) -> u32 {
    let s = seed.wrapping_add((frame as u32).wrapping_sub(FIRST as u32) << 16);
    if s != 0 {
        s
    } else {
        1
    }
}

/// The scenario script's inputs for session frame `f` (the "controllers" of the scripted match).
pub fn script_inputs(f: i32) -> [Option<RbInput>; SLOTS] {
    let mut a = [None; SLOTS];
    for p in 0..2 {
        a[p * 2] = Some(script_input(p, f - FIRST));
    }
    a
}

impl RbGame for NetMatch {
    type Snapshot = RbSnapshot;
    fn frame(&self) -> i32 {
        self.vs.frame + FIRST
    }
    fn live(&self) -> bool {
        self.live
    }
    fn port_human(&self, port: usize) -> bool {
        self.humans.get(port).copied().unwrap_or(false)
    }
    fn peek_input(&self, slot: usize, frame: i32) -> Option<RbInput> {
        (slot % 2 == 0 && slot / 2 < 2).then(|| script_input(slot / 2, frame - FIRST))
    }
    fn save(&self) -> RbSnapshot {
        self.vs.rb_save()
    }
    fn load(&mut self, s: &RbSnapshot) {
        self.vs.rb_load(s)
    }
    fn halted(&self) -> bool {
        self.vs.stopped.is_some()
    }
    fn state_hash(&self) -> u32 {
        self.vs.rb_state_hash()
    }
    fn game_hash(&self) -> u32 {
        match self.live_seed {
            Some(sd) => self.vs.rb_game_hash_seeded(live_frame_seed(sd, self.vs.frame + 1 + FIRST)),
            None => self.vs.rb_game_hash(),
        }
    }
    fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], _resim: bool) {
        use crate::Sim;
        let mut pads = [None; 4];
        for p in 0..4 {
            pads[p] = inputs[p * 2];
        }
        self.vs.set_pads(&pads);
        if let Some(sd) = self.live_seed {
            self.vs.rb_force_seed(live_frame_seed(sd, self.vs.frame + 1 + FIRST));
        }
        self.vs.step();
        self.raw_log.insert(self.vs.frame, self.vs.rb_row());
    }
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// The pad_falcon_fox_fd match, built the way `melee golden` builds it.
pub fn open_match(args: &[String]) -> Result<NetMatch, String> {
    let iso = arg(args, "--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()).ok_or("no --iso / GW_ISO_VANILLA")?;
    let mut d = crate::disc::Disc::open(&iso)?;
    let vs = VsMatch::new(&mut d, "GrNLa.dat", &["falcon", "fox"], 12345, 3)?;
    Ok(NetMatch::new(vs))
}

/// A match built for an agreed scene: fighters by name, the agreed seed and stock count.
pub fn open_match_for(args: &[String], stage_file: &str, fighters: &[&str], seed: u32, stocks: i32) -> Result<NetMatch, String> {
    let iso = arg(args, "--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()).ok_or("no --iso / GW_ISO_VANILLA")?;
    let mut d = crate::disc::Disc::open(&iso)?;
    let vs = VsMatch::new(&mut d, stage_file, fighters, seed, stocks)?;
    Ok(NetMatch::new(vs))
}

/// The golden CSV's hash column by frame.
pub fn golden_hashes(path: &str) -> Result<std::collections::HashMap<i32, u32>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut m = std::collections::HashMap::new();
    for l in text.lines().skip(1) {
        let c: Vec<&str> = l.split(',').collect();
        if c.len() > 2 {
            if let (Ok(f), Ok(h)) = (c[0].parse::<i32>(), u32::from_str_radix(c[1], 16)) {
                m.insert(f, h);
            }
        }
    }
    Ok(m)
}

/// `melee synctest [--depth K] [--frames N] [--golden CSV]`: GGPO's SyncTest (gw_snap.c) over the
/// whole scripted match: every frame, save, step, roll back K frames, resimulate and compare the
/// whole-state hash and the neutral hash at every resimulated frame.
pub fn synctest(args: &[String]) -> std::process::ExitCode {
    use melee_net::session::SyncTest;
    let k: i32 = arg(args, "--depth").and_then(|s| s.parse().ok()).unwrap_or(12);
    let frames: i32 = arg(args, "--frames").and_then(|s| s.parse().ok()).unwrap_or(i32::MAX);
    let golden = match arg(args, "--golden").map(|p| golden_hashes(&p)).transpose() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let mut g = match open_match(args) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let mut st = SyncTest::<NetMatch>::new(k);
    let (mut run, mut golden_bad, mut golden_n) = (0, 0, 0);
    while run < frames && g.vs.stopped.is_none() {
        st.tick(&mut g, &mut |f| script_inputs(f));
        if g.vs.stopped.is_some() {
            break;
        }
        run += 1;
        if let Some(gm) = &golden {
            // the state after the tick is the post-resimulation state of the frame just played
            if let Some(&want) = gm.get(&g.vs.frame) {
                golden_n += 1;
                if g.game_hash() != want {
                    golden_bad += 1;
                    if golden_bad < 5 {
                        eprintln!("melee: golden mismatch at frame {}", g.vs.frame);
                    }
                }
            }
        }
    }
    eprintln!("melee: synctest depth {k}: {} frames, {} rollback passes, {} hash checks, {} MISMATCHES; golden {}/{} frames equal; stopped: {:?}",
        run, st.passes, st.checked, st.mismatches.len(), golden_n - golden_bad, golden_n, g.vs.stopped);
    for m in st.mismatches.iter().take(5) {
        eprintln!("melee:   mismatch frame {} full {:08x}/{:08x} curated {:08x}/{:08x}", m.0, m.1, m.2, m.3, m.4);
    }
    if st.mismatches.is_empty() && golden_bad == 0 && st.checked > 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// The consolidated engine under the rollback session (same file as the legacy `NetMatch`: both implement `RbGame`).
pub mod engine_game {

    use melee_engine::snapshot::{Base, SnapRef};
    use melee_engine::Engine;
    use melee_net::pad::RbInput;
    use melee_net::session::{RbGame, FIRST, SLOTS};
    use melee_types::PADStatus;
    use std::cell::RefCell;

    pub struct EngineGame {
        pub e: Engine,
        base: RefCell<Option<Base>>,
    }

    #[derive(Clone)]
    pub struct EngSnap {
        mem: SnapRef,
        frames: u32,
        ended: bool,
    }

    impl EngineGame {
        pub fn new(e: Engine) -> Self {
            EngineGame { e, base: RefCell::new(None) }
        }
        fn with_base<R>(&self, f: impl FnOnce(&Base) -> R) -> R {
            if self.base.borrow().is_none() {
                *self.base.borrow_mut() = Some(Base::capture());
            }
            f(self.base.borrow().as_ref().unwrap())
        }
    }

    pub fn pad_of(i: &Option<RbInput>) -> PADStatus {
        let mut st = PADStatus::zeroed();
        if let Some(i) = i {
            st.button = i.buttons as u16;
            st.stickX = i.raw[0];
            st.stickY = i.raw[1];
            st.substickX = i.raw[2];
            st.substickY = i.raw[3];
            st.triggerLeft = i.pad_l;
            st.triggerRight = i.pad_r;
        }
        st
    }

    impl EngineGame {
        /// Debug (`melee synctest --engine tu --fidelity N`): run N frames, save, tick, save, load the first, tick, save, and
        /// list the memory pages where the two ticks differ.
        pub fn fidelity(&mut self, n: usize) {
            let mut inputs = |f: i32| { let mut a = [None; SLOTS]; for p in 0..2 { a[p * 2] = Some(crate::vs_match::rollback::script_input(p, f - FIRST)); } a };
            for _ in 0..n { let i = inputs(self.frame() + 1); self.run_frame(&i, false); }
            for slot in 0..2 {
                unsafe {
                    let g = melee_fighter::player::get_entity(slot);
                    eprintln!("slot {slot}: gobj {:p} user {:p} (Fighter {:#x} bytes) hsd_obj {:p}", g, (*g).user_data, core::mem::size_of::<melee_types::Fighter>(), (*g).hsd_obj);
                }
            }
            {
                let mut fl: Vec<(&str, usize)> = crate::fighter_fields::FIGHTER_FIELDS.to_vec();
                fl.sort_by_key(|x| x.1);
                if let Ok(o) = std::env::var("MELEE_FID_OFF").map(|v| usize::from_str_radix(&v, 16).unwrap()) {
                    let f = fl.iter().rev().find(|x| x.1 <= o);
                    eprintln!("Fighter offset {o:#x} is in field {f:?}");
                }
            }
            {
                use melee_engine::arena::*;
                let n = NMISSED.load(std::sync::atomic::Ordering::Relaxed);
                eprintln!("arena misses: {n} allocs {} range {:x?}", melee_engine::arena::allocations(), melee_engine::arena::used_range());
                for k in 0..n.min(16) { eprintln!("  miss size {} align {}", MISSED[2*k].load(std::sync::atomic::Ordering::Relaxed), MISSED[2*k+1].load(std::sync::atomic::Ordering::Relaxed)); }
            }
            let s0 = self.save();
            let i = inputs(self.frame() + 1);
            self.run_frame(&i, false);
            let s1 = self.save();
            self.load(&s0);
            let s0b = self.save();
            self.with_base(|b| for l in b.diff(&s0.mem, &s0b.mem, 20) { eprintln!("load differs: {l}"); });
            self.run_frame(&i, false);
            let s2 = self.save();
            self.with_base(|b| for l in b.diff(&s1.mem, &s2.mem, 40) { eprintln!("tick differs: {l}"); });
        }
    }

    impl RbGame for EngineGame {
        type Snapshot = EngSnap;
        fn frame(&self) -> i32 {
            self.e.frames as i32 - 1 + FIRST
        }
        fn peek_input(&self, slot: usize, frame: i32) -> Option<RbInput> {
            (slot % 2 == 0 && slot / 2 < 2).then(|| crate::vs_match::rollback::script_input(slot / 2, frame - FIRST))
        }
        fn save(&self) -> EngSnap {
            EngSnap { mem: self.with_base(|b| b.save()), frames: self.e.frames, ended: self.e.ended }
        }
        fn load(&mut self, s: &EngSnap) {
            self.with_base(|b| b.load(&s.mem));
            self.e.frames = s.frames;
            self.e.ended = s.ended;
        }
        fn state_hash(&self) -> u32 {
            self.with_base(|b| b.page_hash())
        }
        fn game_hash(&self) -> u32 {
            self.e.neutral_row()[0]
        }
        fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], _resim: bool) {
            let mut p = [PADStatus::zeroed(); 4];
            for (port, slot) in p.iter_mut().enumerate() {
                *slot = pad_of(&inputs[port * 2]);
            }
            self.e.set_pads(Some(p));
            self.e.tick();
        }
    }

    /// `melee synctest --engine tu [--scenario S] [--depth K] [--frames N] [--golden CSV]`.
    pub fn synctest(args: &[String]) -> std::process::ExitCode {
        use melee_net::session::SyncTest;
        let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let k: i32 = arg("--depth").and_then(|s| s.parse().ok()).unwrap_or(4);
        let frames: i32 = arg("--frames").and_then(|s| s.parse().ok()).unwrap_or(i32::MAX);
        let scenario = arg("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
        let golden = match arg("--golden").map(|p| super::golden_hashes(&p)).transpose() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("melee: {e}");
                return std::process::ExitCode::from(2);
            }
        };
        let Some(iso) = arg("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
            eprintln!("melee: no --iso / GW_ISO_VANILLA");
            return std::process::ExitCode::from(2);
        };
        if scenario.starts_with("pad_") && std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
            std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
        }
        let e = match crate::boot_engine_scenario(&iso, &scenario) {
            Ok(e) => e,
            Err(er) => {
                eprintln!("melee: {er}");
                return std::process::ExitCode::FAILURE;
            }
        };
        let mut g = EngineGame::new(e);
        if let Some(n) = arg("--fidelity").and_then(|v| v.parse().ok()) {
            g.fidelity(n);
            return std::process::ExitCode::SUCCESS;
        }
        let mut st = SyncTest::<EngineGame>::new(k);
        let (mut run, mut bad, mut n) = (0, 0, 0);
        let t0 = std::time::Instant::now();
        let script = |f: i32| {
            let mut a = [None; SLOTS];
            if scenario.starts_with("pad_") {
                for p in 0..2 {
                    a[p * 2] = Some(crate::vs_match::rollback::script_input(p, f - FIRST));
                }
            }
            a
        };
        while run < frames && !g.e.ended {
            st.tick(&mut g, &mut |f| script(f));
            run += 1;
            if let Some(gm) = &golden {
                let gf = g.e.frames as i32 - 1;
                if let Some(&want) = gm.get(&gf) {
                    n += 1;
                    if g.game_hash() != want {
                        bad += 1;
                        if bad < 5 {
                            eprintln!("melee: golden mismatch at frame {gf}");
                        }
                    }
                }
            }
            if run % 100 == 0 {
                eprintln!("melee: synctest {run} frames, {} mismatches, {:.1}s", st.mismatches.len(), t0.elapsed().as_secs_f32());
            }
        }
        eprintln!("melee: engine synctest depth {k}: {run} frames, {} rollback passes, {} hash checks, {} MISMATCHES; golden {}/{} frames equal",
            st.passes, st.checked, st.mismatches.len(), n - bad, n);
        for m in st.mismatches.iter().take(5) {
            eprintln!("melee:   mismatch frame {} full {:08x}/{:08x} curated {:08x}/{:08x}", m.0, m.1, m.2, m.3, m.4);
        }
        if st.mismatches.is_empty() && bad == 0 && st.checked > 0 { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
    }

}
