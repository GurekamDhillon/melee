//! The engine under the rollback session (`melee_net::session`): `RbGame` for [`melee_engine::Engine`] (`engine_game`), the in-process
//! SyncTest (`melee synctest`) and what `melee netplay` needs of a game (`NetGame`).
//!
//! Frame mapping: the session's first frame is `FIRST` (-123, Slippi's); the match's golden frame `g` (0 = the first logged
//! frame) is session frame `FIRST + g`, so `frame()` reads `FIRST - 1` before the first step, as `gw_Replay_Frame` does once the
//! match is armed.

use melee_net::pad::RbInput;
use melee_net::session::{RbGame, FIRST, SLOTS};

/// `gw_Replay_ChecksumSeed` in live mode: the seed a frame starts from.
pub fn live_frame_seed(seed: u32, frame: i32) -> u32 {
    let s = seed.wrapping_add((frame as u32).wrapping_sub(FIRST as u32) << 16);
    if s != 0 {
        s
    } else {
        1
    }
}

/// The scenario script's sample as a session input (the "controller" of a scripted player).
pub fn script_input(port: usize, g: i32) -> RbInput {
    let r = crate::pads::script_pad(port, g).unwrap_or_else(|| crate::pads::pad_fixed(port, g));
    RbInput {
        present: 1,
        is_raw: 1,
        confirmed: 1,
        buttons: r.button & 0xFFFF,
        raw: [r.stick_x, r.stick_y, r.sub_x, r.sub_y],
        pad_l: r.analog_l,
        pad_r: r.analog_r,
        ..RbInput::default()
    }
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// What `melee netplay` needs of a game beyond the session's `RbGame`.
pub trait NetGame: RbGame + Sized {
    /// The agreed match: stage file, fighters by name, seed, stocks.
    fn open(args: &[String], stage_file: &str, names: &[&str], seed: u32, stocks: i32) -> Result<Self, String>;
    /// Live play (the pads decide); `live_seed` is the online RNG rule (every frame starts from `seed + (frame << 16)`).
    fn set_live(&mut self, live_seed: Option<u32>);
    /// The after-frame neutral rows by golden frame (what `--raw-out` writes).
    fn raw_log(&self) -> Vec<(i32, String)>;
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

/// The consolidated engine under the rollback session (same file as the legacy `NetMatch`: both implement `RbGame`).
pub mod engine_game {

    use melee_engine::snapshot::{Base, SnapRef};
    use melee_engine::Engine;
    use melee_net::pad::RbInput;
    use melee_net::session::{RbGame, FIRST, SLOTS};
    use melee_types::PADStatus;
    use std::cell::RefCell;

    /// Nanoseconds spent in save / load / state_hash / game_hash / run_frame (printed by `synctest`; the rollback profile).
    /// The counters live in a leaked heap block: a load puts every static back, so a static array would be rolled back too.
    static T_ADDR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn t_ns() -> &'static [std::sync::atomic::AtomicU64; 5] {
        let mut a = T_ADDR.load(std::sync::atomic::Ordering::Relaxed);
        if a == 0 {
            a = Box::leak(Box::new([const { std::sync::atomic::AtomicU64::new(0) }; 5])) as *const _ as usize;
            T_ADDR.store(a, std::sync::atomic::Ordering::Relaxed);
        }
        unsafe { &*(a as *const [std::sync::atomic::AtomicU64; 5]) }
    }
    fn timed<R>(i: usize, f: impl FnOnce() -> R) -> R {
        let t = std::time::Instant::now();
        let r = f();
        t_ns()[i].fetch_add(t.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        r
    }

    pub struct EngineGame {
        pub e: Engine,
        base: RefCell<Option<Base>>,
        pub live: bool,
        pub live_seed: Option<u32>,
        pub rows: std::collections::BTreeMap<i32, String>,
        /// Slippi online mode: the desync checksum of the state entering each online frame (frame 1 = session `FIRST`).
        pub slippi_ck: std::collections::BTreeMap<i32, u32>,
        /// Slippi online mode: byte 8 of each port's pad per session frame (the random bits its owner sent with it;
        /// a frame not here, or a predicted pad, counts 0) and each port's external character, for the seed-bit mix.
        pub slippi_bits: std::collections::BTreeMap<i32, [u8; 4]>,
        pub slippi_chars: [u32; 4],
        /// Slippi online: the frames whose seed-bit mix read pad bytes (a port rolled), with the bytes it read; a
        /// confirmed pad whose byte differs must resimulate that frame even when its buttons and sticks were predicted.
        pub slippi_mixed: std::collections::BTreeMap<i32, [u8; 4]>,
        /// Slippi online (`slippi-netplay --trace`): the pads and seed bits each online frame last ran with (a resimulated
        /// frame overwrites its entry, so a finalized frame holds its confirmed pads).
        pub slippi_trace: Option<std::collections::BTreeMap<i32, String>>,
    }

    #[derive(Clone)]
    pub struct EngSnap {
        mem: SnapRef,
        frames: u32,
        ended: bool,
    }

    impl EngineGame {
        pub fn new(e: Engine) -> Self {
            EngineGame { e, base: RefCell::new(None), live: false, live_seed: None, rows: Default::default(), slippi_ck: Default::default(), slippi_bits: Default::default(), slippi_chars: [u32::MAX; 4], slippi_trace: None, slippi_mixed: Default::default() }
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
            let mut inputs = |f: i32| { let mut a = [None; SLOTS]; for p in 0..2 { a[p * 2] = Some(crate::netmatch::script_input(p, f - FIRST)); } a };
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
            {
                let (bump, free, top) = melee_engine::arena::stats();
                eprintln!("arena: bumped {:.0} MB, free-listed {:.0} MB, top free classes (size, blocks) {:?}", bump as f64 / 1e6, free as f64 / 1e6, top);
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
            (slot % 2 == 0 && slot / 2 < 2).then(|| crate::netmatch::script_input(slot / 2, frame - FIRST))
        }
        fn save(&self) -> EngSnap {
            EngSnap { mem: timed(0, || self.with_base(|b| b.save())), frames: self.e.frames, ended: self.e.ended }
        }
        fn load(&mut self, s: &EngSnap) {
            timed(1, || self.with_base(|b| b.load(&s.mem)));
            self.e.frames = s.frames;
            self.e.ended = s.ended;
        }
        fn state_hash(&self) -> u32 {
            timed(2, || self.with_base(|b| b.page_hash()))
        }
        fn game_hash(&self) -> u32 {
            // the session hash: NHV1 with the sim mods' gd_hash_word mixed in when they gave one
            timed(3, || melee_modhost::session_hash(self.e.neutral_row()[0]))
        }
        fn live(&self) -> bool {
            self.live
        }
        fn port_human(&self, port: usize) -> bool {
            port < 2
        }
        fn halted(&self) -> bool {
            self.e.ended
        }
        fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], _resim: bool) {
            let mut p = [PADStatus::zeroed(); 4];
            for (port, slot) in p.iter_mut().enumerate() {
                *slot = pad_of(&inputs[port * 2]);
            }
            self.e.set_pads(Some(p));
            // Slippi online: the pads TriggerSendInput section 11 mixes for this frame (SyncRNG takes them at its
            // frame, so a resimulated frame mixes its confirmed pads again)
            if let Some(st) = melee_core::slippi_online::state() {
                let bits = self.slippi_bits.get(&(self.frame() + 1)).copied().unwrap_or([0; 4]);
                for port in 0..4 {
                    st.pending[port] = melee_core::slippi_online::PadBits {
                        buttons: p[port].button,
                        stick_x: p[port].stickX,
                        stick_y: p[port].stickY,
                        rng_bits: bits[port],
                        ext_char: self.slippi_chars[port],
                    };
                }
            }
            if let Some(sd) = self.live_seed {
                // gw_Replay_ChecksumSeed in live mode: every frame starts from `seed + ((frame + 123) << 16)`
                melee_runtime::tu::random::global::state().set_logic_seed(super::live_frame_seed(sd, self.frame() + 1));
            }
            let trace_pads = self.slippi_trace.is_some().then(|| {
                let bits = self.slippi_bits.get(&(self.frame() + 1)).copied().unwrap_or([0; 4]);
                (0..2).map(|i| format!("{:04X} {} {} {} {} {} {} {:02X}", p[i].button, p[i].stickX, p[i].stickY, p[i].substickX, p[i].substickY, p[i].triggerLeft, p[i].triggerRight, bits[i])).collect::<Vec<_>>().join(" | ")
            });
            // `slippi-replay --slp`: the straight (no rollback) run is recorded as a Slippi replay
            let slp = crate::slp_export::active();
            if slp {
                crate::slp_export::begin_tick(&p);
            }
            timed(4, || self.e.tick());
            // the hang watchdog's frame counter (synctest / rollback runs never reach the play loops' beat)
            crate::hang::pulse();
            if slp {
                crate::slp_export::end_tick(!self.e.ended);
            }
            let r = self.e.neutral_row();
            let mut t = format!("{:08X},{:08X},{},{:08X}", r[0], r[1], r[2], r[3]);
            for w in &r[4..16] {
                t += &format!(",{w:08X}");
            }
            self.rows.insert(self.e.frames as i32 - 1, t);
            if melee_core::slippi_online::state().is_some() {
                let sf = self.frame();
                if let Some(st) = melee_core::slippi_online::state() {
                    if st.last_rolling != 0 {
                        self.slippi_mixed.insert(sf, self.slippi_bits.get(&sf).copied().unwrap_or([0; 4]));
                    } else {
                        self.slippi_mixed.remove(&sf);
                    }
                }
                let ck = melee_fighter::slippi_online::checksum();
                let of = self.frame() + 1 - FIRST + 1;
                self.slippi_ck.insert(of, ck);
                if let (Some(tr), Some(tp)) = (self.slippi_trace.as_mut(), trace_pads) {
                    tr.insert(of - 1, format!("{tp} | ck {ck:08X} | {}", melee_fighter::slippi_online::trace_line()));
                }
            }
        }
    }

    /// `melee synctest --engine tu [--scenario S] [--depth K] [--frames N] [--golden CSV]`.
    pub fn synctest(args: &[String]) -> std::process::ExitCode {
        use melee_net::session::SyncTest;
        let arg = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let k: i32 = arg("--depth").and_then(|s| s.parse().ok()).unwrap_or(4);
        let frames: i32 = arg("--frames").and_then(|s| s.parse().ok()).unwrap_or(i32::MAX);
        // a hard limit (--timeout SECS, default 900): a hung run (a fault loop, a deadlock) ends itself instead of
        // spinning on until someone kills it
        let limit: u64 = arg("--timeout").and_then(|s| s.parse().ok()).unwrap_or(900);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(limit));
            eprintln!("melee: synctest timed out after {limit} s");
            std::process::abort();
        });
        // MELEE_PROFILE=1: the frame profiler's game-side points run here too (the bit-identity check of the profiler)
        crate::profile::start(&crate::display::current(), args);
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
        let _ = t_ns(); // allocate the timing block before the first snapshot
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
            if scenario.starts_with("pad_") || crate::pads::script_pad(0, 0).is_some() {
                for p in 0..2 {
                    a[p * 2] = Some(crate::netmatch::script_input(p, f - FIRST));
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
                    // the golden CSV is NHV1 (the cross-engine format), not the session hash with the mods' word
                    if g.e.neutral_row()[0] != want {
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
        {
            let t = |i: usize| t_ns()[i].load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / run.max(1) as f64;
            eprintln!("melee: per tick ms: save {:.3} load {:.3} state_hash {:.3} game_hash {:.3} run_frame(tick) {:.3}; total {:.3}", t(0), t(1), t(2), t(3), t(4), t0.elapsed().as_secs_f64() * 1e3 / run.max(1) as f64);
        }
        eprintln!("melee: engine synctest depth {k}: {run} frames, {} rollback passes, {} hash checks, {} MISMATCHES; golden {}/{} frames equal",
            st.passes, st.checked, st.mismatches.len(), n - bad, n);
        for m in st.mismatches.iter().take(5) {
            eprintln!("melee:   mismatch frame {} full {:08x}/{:08x} curated {:08x}/{:08x}", m.0, m.1, m.2, m.3, m.4);
        }
        if st.mismatches.is_empty() && bad == 0 && st.checked > 0 { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
    }


    impl EngineGame {
        /// Boot a VS match from a full scenario string (`p1=ck:2/human/c0;...;stage=ext:32;...`): the Slippi online
        /// match block's rules are given this way (`slippi_np::online_scene`).
        /// Slippi online: the scene's first engine frame runs before online frame 1 and takes no pad (Dolphin's
        /// first in-game tick has no input; online frame F's pads are consumed by tick F + 1, and the checksum Slippi
        /// sends for frame F is the state after F ticks). Measured against Slippi Dolphin's own fighter state per
        /// tick (port/slippi3). Run once before the session starts; it is not a session frame.
        pub fn slippi_pre_tick(&mut self) {
            if let Some(st) = melee_core::slippi_online::state() {
                st.pending = Default::default();
                for port in 0..4 {
                    st.pending[port].ext_char = self.slippi_chars[port];
                }
            }
            self.e.set_pads(Some([PADStatus::zeroed(); 4]));
            let slp = crate::slp_export::active();
            if slp {
                crate::slp_export::begin_tick(&[PADStatus::zeroed(); 4]);
            }
            self.e.tick();
            if slp {
                crate::slp_export::end_tick(true);
            }
            self.e.frames -= 1;
        }
        pub fn open_scene(args: &[String], scene: &str, seed: u32) -> Result<Self, String> {
            // SAFETY: single-threaded start-up, before the engine reads them.
            unsafe {
                std::env::set_var("MELEE_TEST_SEED", seed.to_string());
                if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
                    std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
                }
            }
            let iso = std::env::var("GW_ISO_VANILLA").map_err(|_| "no GW_ISO_VANILLA".to_string())?;
            let iso = super::arg(args, "--iso").unwrap_or(iso);
            let disc = melee_engine::gcm::Gcm::open(&iso).map_err(|e| e.to_string())?;
            let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}"))?;
            let e = melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn)?;
            Ok(EngineGame::new(e))
        }
    }

    impl super::NetGame for EngineGame {
        fn open(_args: &[String], stage_file: &str, names: &[&str], seed: u32, _stocks: i32) -> Result<Self, String> {
            let stage = match stage_file {
                "GrNLa.dat" => "fd",
                "GrNBa.dat" => "bf",
                s => return Err(format!("stage {s} is not wired on the engine")),
            };
            let scene = format!("p1={}/human;p2={}/human;p3=none;p4=none;stage={stage};match=time;minutes=0;items=off", names[0], names[1]);
            // SAFETY: single-threaded start-up, before the engine reads them.
            unsafe {
                std::env::set_var("MELEE_TEST_SEED", seed.to_string());
                if std::env::var_os("MELEE_ENGINE_PRE_DRAWS").is_none() {
                    std::env::set_var("MELEE_ENGINE_PRE_DRAWS", "1");
                }
            }
            let iso = std::env::var("GW_ISO_VANILLA").map_err(|_| "no GW_ISO_VANILLA".to_string())?;
            let iso = super::arg(_args, "--iso").unwrap_or(iso);
            let disc = melee_engine::gcm::Gcm::open(&iso).map_err(|e| e.to_string())?;
            let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}"))?;
            let e = melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn)?;
            Ok(EngineGame::new(e))
        }
        fn set_live(&mut self, live_seed: Option<u32>) {
            self.live = true;
            self.live_seed = live_seed;
        }
        fn raw_log(&self) -> Vec<(i32, String)> {
            self.rows.iter().map(|(f, h)| (*f, h.clone())).collect()
        }
    }
}
