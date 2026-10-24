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
}

impl NetMatch {
    pub fn new(vs: VsMatch) -> Self {
        NetMatch { vs, humans: [true, true, false, false], live: false }
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
    fn state_hash(&self) -> u32 {
        self.vs.rb_state_hash()
    }
    fn game_hash(&self) -> u32 {
        self.vs.rb_game_hash()
    }
    fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], _resim: bool) {
        use crate::Sim;
        let mut pads = [None; 4];
        for p in 0..4 {
            pads[p] = inputs[p * 2];
        }
        self.vs.set_pads(&pads);
        self.vs.step();
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
