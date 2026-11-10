//! Playback and recording state of gw_replay.c (everything outside the file parser): the frame
//! counter, the per-frame accessors the fighters read, the seed rules (resync, online, Frame Start),
//! the parity check against the console's post-frame records, the UCF queries, the state trace and
//! the `MELEE_SLP_RECORD` writer (non-rollback path: events are written as they are produced).
//!
//! The C is one translation unit of statics; here that is [`Player`], which owns the loaded
//! [`Replay`] and the [`Recorder`]. Files are in-memory buffers (`trace`, `vel_trace`, `seed_csv`,
//! the recording) so a test compares bytes; log lines collect in `log`. Where the C ends the process
//! (`rp_parity_end`: `_exit`), [`Player::exit`] is set and the caller must stop driving.

use crate::cfmt;
use crate::gdlp;
use crate::replay::{self, Input, Replay, FIRST_FRAME, GAME_INFO, SLOTS};

pub const UNARMED: i32 = -0x7FFF_FFFF;

/// Text-mode line end: the C opens its trace files with "w", so Windows writes CRLF.
const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

/// What the C reads from the environment, settings and other modules, supplied by the host.
#[derive(Clone, Debug, Default)]
pub struct Env {
    /// `MELEE_SLP_RESYNC=1`
    pub resync: bool,
    /// `MELEE_SLP_PARITY=1`
    pub parity: bool,
    /// `MELEE_SLP_NO_UCF073=1`
    pub no_ucf073: bool,
    /// `MELEE_SLIPPI_CODES`
    pub slippi_codes: Option<String>,
    /// `MELEE_SLIPPI_FROZEN_PS=1`
    pub frozen_ps: bool,
    /// `MELEE_DETERMINISTIC`
    pub deterministic: Option<String>,
    /// `MELEE_UCF` (nonempty) else `None`
    pub ucf: Option<String>,
    /// settings.cfg `ucf` (default 84)
    pub ucf_setting: i32,
    /// `gw_rb_slippi_local_port()`: -1 outside the experimental mode
    pub slippi_local_port: i32,
    /// `gw_MatchTurboRules()`
    pub match_turbo_rules: u32,
    /// `gw_StageSkin_Last`: (name, digest) of the stage skin played, if any
    pub stage_skin: Option<(String, u64)>,
    /// `MELEE_GDLP_RECORD` (nonempty value) for whole-match `.gdlp` recording
    pub gdlp_record: Option<String>,
    /// settings.cfg `gdlp_record` (used when the environment variable is empty)
    pub gdlp_record_setting: i32,
    /// `gs_identity` + `GENO_VERSION`: what a `.gdlp` must match to play here
    pub identity: Option<gdlp::Identity>,
    /// `gw_Mods_Describe()` for the `.gdlp` header
    pub mods_describe: String,
    /// `time(NULL)` for the header and the file name
    pub time: i64,
    /// the LAB recordings folder (`gr_dir`)
    pub rec_dir: String,
}

#[derive(Default)]
pub struct Recorder {
    pub tried: bool,
    pub file: Option<Vec<u8>>,
    pub raw_len_at: usize,
    pub raw_start: usize,
    pub frame: i32,
    pub seed: u32,
    pub failed: bool,
    pub closed: Option<Vec<u8>>,
    /// every recording closed so far (path, bytes): a `.gdlp` is one file per match
    pub closed_all: Vec<(String, Vec<u8>)>,
    flush_next: i32,
    /// the open file is a whole-match `.gdlp`
    pub gdlp: bool,
    pub gdlp_path: String,
    gdlp_seq: u32,
    gdlp_wanted: Option<bool>,
}

pub struct Player {
    pub env: Env,
    pub r: Replay,
    pub active: bool,
    pub live: bool,
    pub frame: i32,
    pub seed_diverged: bool,
    pub resync: bool,
    pub parity: bool,
    pub parity_checked: i64,
    pub parity_failed: bool,
    pub parity_seed_frames: i32,
    pub slippi_local_mismatches: i32,
    post_seen: Vec<bool>,
    pub rec: Recorder,
    pub trace: Option<Vec<u8>>,
    pub vel_trace: Option<Vec<u8>>,
    pub seed_csv: Option<Vec<u8>>,
    pub trace_enabled: bool,
    seed_tried: bool,
    noted: bool,
    pub log: Vec<String>,
    /// `gw_FighterControls_MatchGet`: the choices frozen by [`Player::match_begin`].
    pub match_controls: [u32; 4],
    /// `gw_FighterControls_MatchCK`: the character kind each port's choice was frozen for.
    pub match_cks: [i32; 4],
    /// Set when the C would have called `_exit` (parity verdict).
    pub exit: Option<i32>,
    pub resimulating: bool,
    ucf_off_mode: i32,
    ucf_hist: [[[i8; 4]; 8]; 4],
    /// Online replays: the console's forced frame seed minus the Frame Start event's (a multiple of 0x10000).
    pub fs_offset: core::cell::Cell<u32>,
    ucf_head: [u32; 4],
    codes_cache: Option<i32>,
    det_cache: Option<bool>,
    fixture_reason: &'static str,
}

/// Result of [`Player::slippi_fixture_info`].
#[derive(Clone, Debug)]
pub struct FixtureInfo {
    pub version: [u8; 4],
    pub human_ports: [u8; 2],
    pub first_frame: i32,
    pub last_frame: i32,
    pub seed: u32,
    pub game_info: [u8; GAME_INFO],
}

impl Player {
    pub fn new(env: Env) -> Player {
        Player {
            env,
            r: Replay::empty(),
            active: false,
            live: false,
            frame: UNARMED,
            seed_diverged: false,
            resync: false,
            parity: false,
            parity_checked: 0,
            parity_failed: false,
            parity_seed_frames: 0,
            slippi_local_mismatches: 0,
            post_seen: Vec::new(),
            rec: Recorder { frame: UNARMED, flush_next: UNARMED, ..Default::default() },
            trace: None,
            vel_trace: None,
            seed_csv: None,
            trace_enabled: false,
            seed_tried: false,
            noted: false,
            log: Vec::new(),
            match_controls: [0; 4],
            match_cks: [-1; 4],
            exit: None,
            resimulating: false,
            ucf_off_mode: -1,
            ucf_hist: [[[0; 4]; 8]; 4],
            fs_offset: core::cell::Cell::new(0),
            ucf_head: [0; 4],
            codes_cache: None,
            det_cache: None,
            fixture_reason: "not checked",
        }
    }

    fn log(&mut self, s: String) {
        self.log.push(s);
    }

    /// `rp_load` for a replay file: `path` is only used in the log line. `trace` = `MELEE_STATE_TRACE`
    /// is set (the trace buffers are then created if the replay is active).
    pub fn load_slp(&mut self, path: &str, data: &[u8], trace: bool) {
        let mut parsed = Replay::empty();
        let res = replay::parse_into(&mut parsed, data, self.env.parity);
        self.r = parsed;
        match res {
            Ok(()) => {
                self.parity = self.env.parity;
                self.post_seen = vec![false; self.r.post.as_ref().map_or(0, |p| p.len())];
                self.active = true;
                self.resync = self.env.resync;
                let v = self.r.version;
                let line = format!(
                    "replay: {} - Slippi {}.{}.{}, frames {}..{}, seed 0x{:08X}, scene \"{}\"{}{}",
                    path,
                    v[0],
                    v[1],
                    v[2],
                    self.r.first,
                    self.r.last,
                    self.r.seed,
                    self.r.scene,
                    if self.r.online { ", online (per-frame seed)" } else { "" },
                    if self.resync { ", resync on" } else { "" }
                );
                self.log(line);
                if self.r.turbo != 0 {
                    self.log(format!("replay: match rules turbo=0x{:08x}", self.r.turbo));
                }
            }
            Err(e) => {
                let m = e.message();
                if !m.is_empty() {
                    self.log(m);
                }
            }
        }
        self.trace_enabled = trace;
        if self.active && trace {
            self.trace = Some(format!("frame,player,follower,char,action_state,x,y,facing,percent,stocks{NL}").into_bytes());
            self.vel_trace = Some(format!("frame,player,follower,air_x,air_y,kb_x,kb_y,ground_x{NL}").into_bytes());
        }
    }

    /// `rp_load` for `MELEE_GDLP=<file>`: a whole-match recording of this port plays as `MELEE_SLP`
    /// does, but only on the build / disc / mods / Geno data it was made with.
    pub fn load_gdlp(&mut self, path: &str, data: &[u8], trace: bool) {
        let me = self.env.identity.unwrap_or(gdlp::Identity { id: [0; 4], geno_version: 0 });
        match gdlp::check_file(data, &me) {
            Err(why) => {
                self.log(format!("replay: MELEE_GDLP=\"{path}\" refused: {why}"));
            }
            Ok(()) => {
                self.log(format!("replay: MELEE_GDLP=\"{path}\" - identity matches (build, disc, mods, Geno data)"));
                self.load_slp(path, data, trace);
            }
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    /// `gw_Replay_ArmLive`: netplay uses the replay machinery with no recorded inputs.
    pub fn arm_live(&mut self, on: bool) {
        if self.active && !self.live {
            return;
        }
        if !on {
            self.active = false;
            self.live = false;
            self.frame = UNARMED;
            return;
        }
        self.active = true;
        self.live = true;
        self.r.first = FIRST_FRAME;
        self.r.last = 0x3FFF_FFFF;
        self.r.online = true;
        self.r.version = [3, 19, 1, self.r.version[3]];
        self.r.ucf_dashback = [1; 4];
        self.r.ucf_shield = [1; 4];
        self.r.scene.clear();
        self.frame = UNARMED;
        self.log("replay: live mode (netplay) - frames counted from -123, online codes, UCF 0.84".into());
    }

    /// `gw_replay_scene`
    pub fn scene(&self) -> Option<&str> {
        if self.active && !self.live {
            Some(&self.r.scene)
        } else {
            None
        }
    }

    /// `gw_Replay_ApplyMatch`: overwrite the match struct with the replay's (keeping the rules
    /// block's callback pointers, +0x38..+0x5B) and arm the counter; returns the seed. In live mode
    /// `handshake` is `gw_Netplay_Handshake(d, 0x138, 0x38, 0x24)`.
    pub fn apply_match(&mut self, d: &mut [u8], handshake: &mut dyn FnMut(&mut [u8]) -> u32) -> u32 {
        if !self.active {
            return 0;
        }
        if self.live {
            for p in 0..4 {
                let o = 0x60 + 0x24 * p;
                if d[o + 1] != 3 && (d[o + 4] != (p + 1) as u8 || d[o + 7] != p as u8) {
                    let m = format!(
                        "replay: live match - player {} had player slot {} / controller {}, now {} / {}",
                        p + 1,
                        d[o + 4],
                        d[o + 7],
                        p + 1,
                        p
                    );
                    self.log(m);
                    d[o + 4] = (p + 1) as u8;
                    d[o + 7] = p as u8;
                }
            }
            self.r.seed = handshake(d);
            self.r.game_info.copy_from_slice(&d[..GAME_INFO]);
            self.frame = FIRST_FRAME - 1;
            let m = format!("replay: live match armed, seed 0x{:08X}", self.r.seed);
            self.log(m);
            return self.r.seed;
        }
        let mut keep = [0u8; 0x5C - 0x38];
        keep.copy_from_slice(&d[0x38..0x5C]);
        d[..GAME_INFO].copy_from_slice(&self.r.game_info);
        d[0x38..0x5C].copy_from_slice(&keep);
        self.frame = FIRST_FRAME - 1;
        let m = format!("replay: match struct restored, seed 0x{:08X}", self.r.seed);
        self.log(m);
        self.r.seed
    }

    /// `gw_FighterControls_MatchBegin`: freeze each human port's controls choice for the match.
    /// `offline_get(port, character_kind)` is the player's own setting, `netplay(player)` the
    /// peer's announced choice.
    pub fn match_begin(&mut self, d: &[u8], offline_get: &dyn Fn(i32, i32) -> u32, netplay: &dyn Fn(i32) -> u32) {
        self.match_controls = [0; 4];
        self.match_cks = [-1; 4];
        for p in 0..4usize {
            let pl = &d[0x60 + 0x24 * p..];
            // the player's controller as gmvs.c sets it: PlayerInitData +4 `slot` - 1, or the player's own index when
            // slot is 0. gw_replay.c reads +7 (`sub_color`, the team shade): its every human landed on controller 0
            // (fixed there too, port/c-controls-slot)
            let port = if pl[4] == 0 { p } else { pl[4] as usize - 1 };
            let ck = pl[0] as i32;
            if pl[1] != 0 || port >= 4 {
                continue;
            }
            let choice = if self.active && !self.live {
                self.r.controls[port]
            } else if self.live {
                netplay(p as i32)
            } else {
                offline_get(port as i32, ck)
            };
            self.match_controls[port] = choice;
            self.match_cks[port] = ck;
            let m = format!(
                "fighter controls: frozen P{} controller {} ck {} choice 0x{:04x} ({})",
                p + 1,
                port + 1,
                ck,
                self.match_controls[port],
                if self.active {
                    if self.live {
                        "online"
                    } else {
                        "replay"
                    }
                } else {
                    "offline"
                }
            );
            self.log(m);
        }
    }

    // ---- frame counter and input accessors ------------------------------------------------

    pub fn enabled(&mut self) -> bool {
        self.active || self.recording()
    }

    fn rp_cur(&self, port: i32, follower: bool) -> Option<&Input> {
        if !self.active || self.live || self.frame < self.r.first || self.frame > self.r.last || !(0..=3).contains(&port) {
            return None;
        }
        let r = &self.r.inputs[(self.frame - self.r.first) as usize * SLOTS + port as usize * 2 + follower as usize];
        if r.present {
            Some(r)
        } else {
            None
        }
    }

    pub fn has_input(&self, port: i32, follower: bool) -> bool {
        self.rp_cur(port, follower).is_some()
    }

    /// The input the fighter reads (None where the C would dereference NULL).
    pub fn input(&self, port: i32, follower: bool) -> Option<&Input> {
        self.rp_cur(port, follower)
    }

    /// `gw_Replay_Tick`: once per logic frame that runs the fighters; returns the frame now running.
    pub fn tick(&mut self) -> i32 {
        if !self.active || self.frame == UNARMED {
            return UNARMED;
        }
        self.frame += 1;
        if self.frame == self.r.last.wrapping_add(1) {
            let m = format!("replay: past the replay's last frame ({})", self.r.last);
            self.log(m);
            self.parity_verdict();
        }
        self.frame
    }

    pub fn frame(&self) -> i32 {
        self.frame
    }

    pub fn last_frame(&self) -> i32 {
        if self.active {
            self.r.last
        } else {
            0
        }
    }

    pub fn is_live(&self) -> bool {
        self.live
    }

    pub fn port_human(&self, p: i32) -> bool {
        if !self.active || !(0..=3).contains(&p) {
            return false;
        }
        self.r.game_info[0x60 + 0x24 * p as usize + 1] == 0
    }

    fn frame_seed(&self) -> Option<u32> {
        if !self.active || self.live || self.frame < self.r.first || self.frame > self.r.last {
            return None;
        }
        let k = (self.frame - self.r.first) as usize;
        if !self.r.fs_has[k] {
            return None;
        }
        Some(self.r.fs_seed[k])
    }

    // ---- seeds ----------------------------------------------------------------------------

    /// `gw_Replay_ChecksumSeed`
    pub fn checksum_seed(&self, arrived: u32, frame: i32) -> u32 {
        if (self.live || self.env.slippi_local_port >= 0) && frame != UNARMED {
            let s = self.r.seed.wrapping_add((frame as u32).wrapping_sub(FIRST_FRAME as u32) << 16);
            return if s != 0 { s } else { 1 };
        }
        arrived
    }

    /// `gw_Replay_ResyncSeed`: the seed to force at the start of this frame, or 0.
    pub fn resync_seed(&self) -> u32 {
        if self.live || self.env.slippi_local_port >= 0 {
            if self.frame == UNARMED {
                return 0;
            }
            return self.checksum_seed(0, self.frame);
        }
        if self.active && self.frame == FIRST_FRAME {
            let mut r0 = self.rp_cur(0, false);
            let mut p = 0;
            while p < 4 && r0.is_none() {
                r0 = self.rp_cur(p, false);
                p += 1;
            }
            if let Some(r0) = r0 {
                if !self.r.fs_has[0] && r0.seed != 0 {
                    return r0.seed;
                }
            }
            return self.r.seed;
        }
        if self.r.online || self.resync {
            if let Some(s) = self.frame_seed() {
                if self.r.online && s != 0 {
                    // The console forces a seed of the Frame Start form (game seed + n * 0x10000); the event's n can lag by whole
                    // frames late in a match, so the forced value is recovered from the first recorded Pre-Frame seed.
                    let pre = (0..4).find_map(|p| self.rp_cur(p, false).map(|i| i.seed).filter(|&x| x != 0));
                    let mut off = self.fs_offset.get();
                    let forced = pre.and_then(|rec| online_forced_seed(rec, s, &mut off)).unwrap_or(s.wrapping_add(off));
                    self.fs_offset.set(off);
                    return forced;
                }
                return s;
            }
        }
        if self.active && !self.live && !self.r.fs_has[0] && self.frame > FIRST_FRAME {
            for p in 0..4 {
                if let Some(r) = self.rp_cur(p, false) {
                    if r.seed != 0 {
                        return r.seed;
                    }
                }
            }
        }
        0
    }

    /// `gw_Replay_NoteSeed`
    pub fn note_seed(&mut self, arrived: u32) {
        if self.noted {
            return;
        }
        let mut cur = None;
        for p in 0..4 {
            if cur.is_none() {
                if let Some(c) = self.rp_cur(p, false) {
                    if c.seed != 0 {
                        cur = Some(c.seed);
                    }
                }
            }
        }
        let want = match self.frame_seed() {
            Some(s) => s,
            None => cur.unwrap_or(0),
        };
        if want != 0 && self.frame > FIRST_FRAME + 1 && arrived != want {
            self.noted = true;
            let resynced = self.r.online || !self.r.fs_has[0];
            let m = format!(
                "replay: port RNG stream first left the console's at frame {} (port 0x{:08X}, console 0x{:08X}){}",
                self.frame,
                arrived,
                want,
                if resynced { " - resynced per frame from here on" } else { "" }
            );
            self.log(m);
        }
    }

    /// `gw_Replay_CheckSeed`: called at the start of each running frame with the port's seed.
    pub fn check_seed(&mut self, port_seed: u32) {
        if self.resimulating {
            return;
        }
        self.rec_tick(port_seed);
        if !self.seed_tried {
            self.seed_tried = true;
            if self.trace_enabled {
                self.seed_csv = Some(Vec::new());
            }
        }
        if self.frame != UNARMED {
            if let Some(f) = self.seed_csv.as_mut() {
                f.extend_from_slice(format!("{},{:08X}{}", self.frame, port_seed, NL).as_bytes());
            }
        }
        let want = match self.frame_seed() {
            Some(w) if !self.seed_diverged => w,
            _ => return,
        };
        if port_seed != want {
            self.seed_diverged = true;
            let m = format!("replay: RNG seed diverged at frame {}: port 0x{:08X}, console 0x{:08X}", self.frame, port_seed, want);
            self.log(m);
            if self.parity && !self.parity_failed {
                self.parity_failed = true;
                let m = format!(
                    "parity: FAIL frame {} port 0 field rng_seed: console 0x{:08X}, port 0x{:08X}, bits console 0x{:08X} port 0x{:08X}",
                    self.frame, want, port_seed, want, port_seed
                );
                self.log(m);
                self.parity_end(1);
            }
            return;
        }
        self.parity_seed_frames += 1;
        if self.frame == self.r.last {
            let m = format!("replay: RNG seed matched the console on every frame through {}", self.r.last);
            self.log(m);
        }
    }

    // ---- parity ---------------------------------------------------------------------------

    fn parity_end(&mut self, rc: i32) {
        if self.exit.is_none() {
            self.log(format!("hang_final: replay parity complete {rc}"));
            self.exit = Some(rc);
        }
    }

    fn parity_fail_f(&mut self, field: &str, port: i32, fol: bool, console: f64, port_v: f64) {
        self.parity_failed = true;
        let m = format!(
            "parity: FAIL frame {} port {}{} field {}: console {}, port {}, bits console 0x{:08X} port 0x{:08X}",
            self.frame,
            port + 1,
            if fol { " (follower)" } else { "" },
            field,
            cfmt::g(console, 9),
            cfmt::g(port_v, 9),
            (console as f32).to_bits(),
            (port_v as f32).to_bits()
        );
        self.log(m);
        self.parity_end(1);
    }

    #[allow(clippy::too_many_arguments)]
    fn parity_check(&mut self, port: i32, follower: bool, action: i32, x: f32, y: f32, facing: f32, percent: f32, stocks: i32) {
        if !self.parity
            || self.r.post.is_none()
            || self.parity_failed
            || self.frame < self.r.first
            || self.frame > self.r.last
            || !(0..4).contains(&port)
        {
            return;
        }
        let idx = (self.frame - self.r.first) as usize * SLOTS + port as usize * 2 + follower as usize;
        let q = self.r.post.as_ref().unwrap()[idx];
        if !q.has || self.post_seen[idx] {
            return;
        }
        self.post_seen[idx] = true;
        self.parity_checked += 1;
        let fol = follower;
        if q.action != action as u16 {
            self.parity_fail_f("action_state", port, fol, q.action as f64, (action as u16) as f64);
            return;
        }
        if q.x != x {
            self.parity_fail_f("x", port, fol, q.x as f64, x as f64);
            return;
        }
        if q.y != y {
            self.parity_fail_f("y", port, fol, q.y as f64, y as f64);
            return;
        }
        if q.facing != facing {
            self.parity_fail_f("facing", port, fol, q.facing as f64, facing as f64);
            return;
        }
        if q.percent != percent {
            self.parity_fail_f("percent", port, fol, q.percent as f64, percent as f64);
            return;
        }
        if q.stocks != stocks as u8 {
            self.parity_fail_f("stocks", port, fol, q.stocks as f64, (stocks as u8) as f64);
        }
    }

    fn parity_verdict(&mut self) {
        if !self.parity || self.parity_failed {
            return;
        }
        if self.parity_checked != self.r.parity_expected {
            let frames = (self.r.last - self.r.first) as usize;
            for f in 0..=frames {
                for s in 0..SLOTS {
                    let i = f * SLOTS + s;
                    let q = self.r.post.as_ref().unwrap()[i];
                    if q.has && !self.post_seen[i] {
                        let m = format!(
                            "parity: FAIL frame {} port {}{}: the console recorded a post-frame the port never produced ({} of {} compared)",
                            self.r.first + f as i32,
                            s / 2 + 1,
                            if s & 1 != 0 { " (follower)" } else { "" },
                            self.parity_checked,
                            self.r.parity_expected
                        );
                        self.log(m);
                        self.parity_end(1);
                        return;
                    }
                }
            }
        }
        let m = format!(
            "parity: PASS frames {}..{}, {} post-frame records (action, x, y, facing, percent, stocks) identical, RNG seed checked on {} frames",
            self.r.first, self.r.last, self.parity_checked, self.parity_seed_frames
        );
        self.log(m);
        self.parity_end(0);
    }

    // ---- trace ----------------------------------------------------------------------------

    /// `gw_Replay_TraceAtProcMap`
    pub fn trace_at_proc_map(&self) -> bool {
        self.active && self.r.version[0] < 3
    }

    /// `gw_Replay_Tracing`
    pub fn tracing(&self) -> bool {
        ((self.trace.is_some() || self.parity) && self.frame != UNARMED) || (self.rec.file.is_some() && self.rec.frame != UNARMED)
    }

    /// `gw_Replay_TraceFighter`: the post-frame state of one fighter.
    #[allow(clippy::too_many_arguments)]
    pub fn trace_fighter(
        &mut self,
        port: i32,
        follower: bool,
        ckind: i32,
        action: i32,
        x: f32,
        y: f32,
        facing: f32,
        percent: f32,
        stocks: i32,
        air_x: f32,
        air_y: f32,
        kb_x: f32,
        kb_y: f32,
        ground_x: f32,
    ) {
        if self.resimulating {
            return;
        }
        self.rec_post(port, follower, ckind, action, x, y, facing, percent, stocks, air_x, air_y, kb_x, kb_y, ground_x);
        if self.parity && self.frame != UNARMED {
            self.parity_check(port, follower, action, x, y, facing, percent, stocks);
            if self.exit.is_some() {
                return;
            }
        }
        if self.trace.is_none() || self.frame == UNARMED {
            return;
        }
        let g = |v: f32| cfmt::g(v as f64, 17);
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{}{}",
            self.frame,
            port,
            follower as i32,
            ckind,
            action,
            g(x),
            g(y),
            g(facing),
            g(percent),
            stocks,
            NL
        );
        self.trace.as_mut().unwrap().extend_from_slice(row.as_bytes());
        if let Some(v) = self.vel_trace.as_mut() {
            let row = format!("{},{},{},{},{},{},{},{}{}", self.frame, port, follower as i32, g(air_x), g(air_y), g(kb_x), g(kb_y), g(ground_x), NL);
            v.extend_from_slice(row.as_bytes());
        }
    }

    pub fn get_cursor(&self) -> [i32; 4] {
        [self.frame, self.seed_diverged as i32, self.rec.frame, self.rec.seed as i32]
    }

    pub fn set_cursor(&mut self, c: [i32; 4]) {
        self.frame = c[0];
        self.seed_diverged = c[1] != 0;
        self.rec.frame = c[2];
        self.rec.seed = c[3] as u32;
    }

    // ---- UCF ------------------------------------------------------------------------------

    fn ucf_offline_mode(&mut self) -> i32 {
        if self.ucf_off_mode < 0 {
            self.ucf_off_mode = match self.env.ucf.as_deref() {
                Some(v) if !v.is_empty() => {
                    if matches!(v, "0" | "off" | "OFF" | "false" | "no") {
                        0
                    } else {
                        84
                    }
                }
                _ => {
                    if self.env.ucf_setting != 0 {
                        84
                    } else {
                        0
                    }
                }
            };
            let m = format!(
                "gw: UCF (offline play): {}",
                if self.ucf_off_mode == 84 { "0.84 (dashback, shield drop, 1.0 cardinals, SDI/tumble)" } else { "off (vanilla 1.02 controls)" }
            );
            self.log(m);
        }
        self.ucf_off_mode
    }

    /// `gw_Ucf_PadLatch`
    pub fn ucf_pad_latch(&mut self, port: i32, sx: i32, sy: i32, cx: i32, cy: i32) {
        if !(0..=3).contains(&port) {
            return;
        }
        let p = port as usize;
        self.ucf_head[p] = self.ucf_head[p].wrapping_add(1);
        self.ucf_hist[p][(self.ucf_head[p] & 7) as usize] = [sx as i8, sy as i8, cx as i8, cy as i8];
    }

    fn ucf_pad_back(&self, port: i32, which: i32, back: i32) -> i32 {
        if !(0..=3).contains(&port) || !(0..=3).contains(&which) || !(0..=7).contains(&back) {
            return 0;
        }
        let p = port as usize;
        self.ucf_hist[p][(self.ucf_head[p].wrapping_sub(back as u32) & 7) as usize][which as usize] as i32
    }

    /// `gw_Replay_UcfCardinals`
    pub fn ucf_cardinals(&mut self, port: i32) -> bool {
        if !self.active && !self.live {
            return (0..=3).contains(&port) && self.ucf_offline_mode() == 84;
        }
        if !self.active || !(0..=3).contains(&port) || self.r.ucf_dashback[port as usize] != 1 {
            return false;
        }
        self.r.version[0] >= 3 && !self.r.ucf_073_list && !self.r.no_ucf_cardinals
    }

    /// `gw_Replay_RawStick`. A record without the raw C-stick bytes (Slippi before they were added)
    /// gets them back from the processed floats; a C-stick held on a cardinal reads 79/80 = 0.9875
    /// after Melee's clamp while the pad's raw byte was 80 or more, which UCF 0.84 snaps to 1.0
    /// (an ASDI with the C-stick then moves 3.0, not 2.9625). So 0.9875 with 0 on the other axis
    /// is taken as a raw 80. (Over the Pichu set this is right 15 times and wrong once, a stick creeping
    /// 78 -> 79; a "crept there" or "held" rule only trades files.) SLPCHECK_RAWC_OFF=1: leave them 0.
    pub fn raw_stick(&self, port: i32, which: i32) -> i32 {
        if !self.active && !self.live {
            return self.ucf_pad_back(port, which, 0);
        }
        match self.rp_cur(port, false) {
            Some(r) if which >= 2 && which < 4 && !r.raw_c && std::env::var_os("SLPCHECK_RAWC_OFF").is_none() => raw_c_from_processed(r.cx, r.cy, which),
            Some(r) if (0..4).contains(&which) => r.raw[which as usize] as i32,
            _ => 0,
        }
    }

    /// `gw_Replay_UcfVersion`: 0 off, 73 or 84.
    pub fn ucf_version(&mut self, port: i32) -> i32 {
        if !self.active && !self.live {
            return if (0..=3).contains(&port) { self.ucf_offline_mode() } else { 0 };
        }
        if !self.active || !(0..=3).contains(&port) || self.r.ucf_dashback[port as usize] != 1 {
            return 0;
        }
        if self.r.version[0] >= 3 && !self.r.ucf_073_list {
            return 84;
        }
        if self.env.no_ucf073 {
            0
        } else {
            73
        }
    }

    /// `gw_Replay_RawStickBack`
    pub fn raw_stick_back(&self, port: i32, which: i32, back: i32) -> i32 {
        let f = self.frame.wrapping_sub(back);
        if !self.active && !self.live {
            return self.ucf_pad_back(port, which, back);
        }
        if !self.active || self.live || f < self.r.first || f > self.r.last || !(0..=3).contains(&port) || !(0..=3).contains(&which) {
            return 0;
        }
        let r = &self.r.inputs[(f - self.r.first) as usize * SLOTS + port as usize * 2];
        if r.present {
            r.raw[which as usize] as i32
        } else {
            0
        }
    }

    /// `gw_Replay_PeekInput`: the replay's recorded input for (slot, frame).
    pub fn peek_input(&self, slot: i32, frame: i32) -> Option<Input> {
        if !self.active || self.live || frame < self.r.first || frame > self.r.last || !(0..SLOTS as i32).contains(&slot) {
            return None;
        }
        let r = self.r.inputs[(frame - self.r.first) as usize * SLOTS + slot as usize];
        if r.present {
            Some(r)
        } else {
            None
        }
    }

    // ---- Slippi fixture -------------------------------------------------------------------

    pub fn fixture_reason(&self) -> &'static str {
        self.fixture_reason
    }

    /// `gw_Replay_SlippiFixtureInfo`
    pub fn slippi_fixture_info(&mut self) -> Option<FixtureInfo> {
        use crate::slippi_pad;
        macro_rules! no {
            ($why:expr) => {{
                self.fixture_reason = $why;
                return None;
            }};
        }
        if !self.active || self.live || !self.r.game_start_seen || self.r.inputs.is_empty() {
            no!("no complete Game Start");
        }
        if self.r.truncated {
            no!("truncated replay");
        }
        let v = self.r.version;
        if v[0] < 3 || (v[0] == 3 && v[1] < 17) {
            no!("Slippi version before 3.17");
        }
        if !self.r.online {
            no!("not a Slippi online match");
        }
        if self.r.first != FIRST_FRAME || self.r.last < self.r.first {
            no!("invalid frame range");
        }
        let stage = ((self.r.game_info[0x0E] as u16) << 8) | self.r.game_info[0x0F] as u16;
        if stage > 32 {
            no!("non-vanilla stage");
        }
        for port in 0..4 {
            let ty = self.r.game_info[0x60 + 0x24 * port + 1];
            if ty != if port < 2 { 0 } else { 3 } {
                no!("expected human ports 1 and 2 only");
            }
        }
        for frame in self.r.first..=self.r.last {
            let k = (frame - self.r.first) as usize;
            if self.r.fs_has[k] && self.r.fs_seed[k] != self.r.seed.wrapping_add(((frame - FIRST_FRAME) as u32) << 16) {
                no!("online seed rule differs from fixture");
            }
            for port in 0..2 {
                let i = &self.r.inputs[k * SLOTS + port * 2];
                if !i.present {
                    no!("missing player input");
                }
                if !i.physical_complete {
                    no!("missing physical input fields");
                }
                if slippi_pad::from_physical(i.physical_buttons, i.raw[0], i.raw[1], i.raw[2], i.raw[3], i.physical_l, i.physical_r).is_none() {
                    no!("invalid physical trigger");
                }
            }
        }
        self.fixture_reason = "ok";
        Some(FixtureInfo {
            version: self.r.version,
            human_ports: [0, 1],
            first_frame: self.r.first,
            last_frame: self.r.last,
            seed: self.r.seed,
            game_info: self.r.game_info,
        })
    }

    /// `gw_Replay_SlippiPad`
    pub fn slippi_pad(&self, port: i32, slp_frame: i32) -> Option<crate::slippi_pad::SlippiPad> {
        if !(0..=1).contains(&port) || !self.active || self.live || slp_frame < self.r.first || slp_frame > self.r.last || self.r.inputs.is_empty() {
            return None;
        }
        let i = &self.r.inputs[(slp_frame - self.r.first) as usize * SLOTS + port as usize * 2];
        if i.present && i.physical_complete {
            crate::slippi_pad::from_physical(i.physical_buttons, i.raw[0], i.raw[1], i.raw[2], i.raw[3], i.physical_l, i.physical_r)
        } else {
            None
        }
    }

    // ---- Slippi codes and determinism -------------------------------------------------------

    /// `gw_Det_Enabled`
    pub fn det_enabled(&mut self) -> bool {
        if self.live {
            return true;
        }
        if self.det_cache.is_none() {
            let v = match self.env.deterministic.as_deref() {
                Some(v) if !v.is_empty() => v.starts_with('1'),
                _ => self.active,
            };
            self.det_cache = Some(v);
            let m = format!("det: deterministic mode {}", if v { "ON" } else { "off" });
            self.log(m);
        }
        self.det_cache.unwrap()
    }

    /// `gw_Slippi_Codes`: 0 none, 1 console/tournament codeset, 2 online codeset.
    pub fn slippi_codes(&mut self) -> i32 {
        if self.live {
            return 2;
        }
        if self.codes_cache.is_none() {
            let c = match self.env.slippi_codes.as_deref() {
                Some(v) if !v.is_empty() => {
                    let b = v.as_bytes()[0];
                    if b == b'o' || b == b'O' {
                        2
                    } else if b == b't' || b == b'T' || b == b'c' {
                        1
                    } else {
                        0
                    }
                }
                _ => {
                    if self.active {
                        if self.r.online {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    }
                }
            };
            self.codes_cache = Some(c);
            let m = format!("slippi: gameplay codes {}", if c == 2 { "online" } else if c == 1 { "tournament" } else { "off" });
            self.log(m);
        }
        self.codes_cache.unwrap()
    }

    /// `gw_Slippi_Version`: major*10000 + minor*100 + build, or 999999 outside playback.
    pub fn slippi_version(&self) -> i32 {
        if !self.active {
            return 999999;
        }
        self.r.version[0] as i32 * 10000 + self.r.version[1] as i32 * 100 + self.r.version[2] as i32
    }

    /// `gw_Slippi_FrozenStadium`
    pub fn slippi_frozen_stadium(&self) -> bool {
        if self.active {
            return self.r.frozen_ps;
        }
        self.env.frozen_ps
    }

    /// `gw_Replay_ControlsRaw`
    pub fn controls_raw(&self) -> bool {
        !self.active || self.live
    }

    /// `gw_Replay_TurboRules`
    pub fn turbo_rules(&self) -> u32 {
        self.r.turbo
    }

    // ---- recording (MELEE_SLP_RECORD) -------------------------------------------------------

    /// `gw_Replay_Recording` for `MELEE_SLP_RECORD=<path>`: opens the in-memory recording once.
    pub fn start_recording(&mut self, path: &str) {
        if self.rec.tried {
            return;
        }
        self.rec.tried = true;
        let mut f = Vec::new();
        f.extend_from_slice(&[b'{', b'U', 3, b'r', b'a', b'w', b'[', b'$', b'U', b'#', b'l']);
        self.rec.raw_len_at = f.len();
        f.extend_from_slice(&[0, 0, 0, 0]);
        self.rec.raw_start = f.len();
        f.extend_from_slice(&[
            0x35,
            16,
            0x36,
            (replay::SZ_START >> 8) as u8,
            (replay::SZ_START & 0xFF) as u8,
            0x37,
            0,
            replay::SZ_PRE as u8,
            0x38,
            0,
            replay::SZ_POST as u8,
            0x39,
            0,
            replay::SZ_END as u8,
            0x3A,
            0,
            replay::SZ_FSTART as u8,
        ]);
        self.rec.file = Some(f);
        self.log(format!("replay: recording to {path}"));
    }

    fn gdlp_wanted(&mut self) -> bool {
        if self.rec.gdlp_wanted.is_none() {
            let v = match self.env.gdlp_record.as_deref() {
                Some(e) if !e.is_empty() => e != "0",
                _ => self.env.gdlp_record_setting != 0,
            };
            self.rec.gdlp_wanted = Some(v);
        }
        self.rec.gdlp_wanted.unwrap()
    }

    /// `gw_Replay_Recording` after the environment was consulted (a whole-match `.gdlp` is opened
    /// per match, never while a replay plays back; live netplay is not a replay).
    pub fn recording(&mut self) -> bool {
        self.rec.file.is_some() || ((!self.active || self.live) && self.gdlp_wanted())
    }

    /// `gw_Replay_RecordMatch`: the match as it starts (call after [`Player::apply_match`]).
    pub fn record_match(&mut self, game_info: &[u8], seed: u32) {
        if !self.recording() {
            return;
        }
        if (self.rec.file.is_none() || self.rec.gdlp) && (!self.active || self.live) && self.gdlp_wanted() {
            if self.rec.file.is_some() {
                self.rec_close(); // the previous match, if its scene end was missed
            }
            let spec = self.env.gdlp_record.clone().unwrap_or_default();
            self.rec.gdlp_seq += 1;
            let path = gdlp::match_path(&spec, self.rec.gdlp_seq, self.env.time, &self.env.rec_dir);
            let (mut f, raw_len_at, raw_start) = gdlp::open(gdlp::GR_VER);
            self.rec.raw_len_at = raw_len_at;
            self.rec.raw_start = raw_start;
            self.rec.gdlp = true;
            self.rec.failed = false;
            let head = gdlp::match_head(
                game_info,
                &gdlp::HeadInputs {
                    write_ver: gdlp::GR_VER,
                    identity: self.env.identity.map_or([0; 4], |i| i.id),
                    geno_version: self.env.identity.map_or(0, |i| i.geno_version),
                    mods: self.env.mods_describe.clone(),
                    controls_ck: self.match_cks,
                    controls: self.match_controls,
                    time: self.env.time as u32,
                },
            );
            f.extend_from_slice(&head);
            self.rec.file = Some(f);
            self.rec.gdlp_path = path.clone();
            self.log(format!("replay: recording the match to {path} (.gdlp)"));
        }
        if self.rec.file.is_none() {
            return;
        }
        let mut ev = vec![0u8; 1 + replay::SZ_START];
        ev[0] = 0x36;
        ev[1] = 3;
        ev[2] = 19;
        ev[3] = 1;
        ev[4] = 0;
        ev[5..5 + GAME_INFO].copy_from_slice(&game_info[..GAME_INFO]);
        ev[0x13D..0x141].copy_from_slice(&be32(seed));
        ev[0x1A3] = 2;
        ev[0x1A4] = 2;
        if self.active {
            ev[1..5].copy_from_slice(&self.r.version);
            for p in 0..4 {
                ev[0x141 + 8 * p..0x145 + 8 * p].copy_from_slice(&be32(self.r.ucf_dashback[p]));
                ev[0x145 + 8 * p..0x149 + 8 * p].copy_from_slice(&be32(self.r.ucf_shield[p]));
            }
            ev[0x1A4] = if self.r.online { 8 } else { 2 };
        }
        replay::rules_write(&mut ev, self.env.match_turbo_rules);
        replay::controls_write(&mut ev, &self.match_controls);
        self.rec.file.as_mut().unwrap().extend_from_slice(&ev);
        self.rec.frame = FIRST_FRAME - 1;
        self.rec.flush_next = FIRST_FRAME;
        let m = format!("replay: recording the match (seed 0x{:08X}, turbo=0x{:08x})", seed, self.env.match_turbo_rules);
        self.log(m);
    }

    /// Frame Bookend (0x3C): the frame, and the latest finalized frame (the frame itself).
    fn rec_bookend(&mut self, frame: i32) {
        if !self.rec.gdlp || self.rec.file.is_none() || self.rec.failed || frame < FIRST_FRAME {
            return;
        }
        let mut ev = [0u8; 9];
        ev[0] = 0x3C;
        ev[1..5].copy_from_slice(&be32(frame as u32));
        ev[5..9].copy_from_slice(&be32(frame as u32));
        self.rec.file.as_mut().unwrap().extend_from_slice(&ev);
    }

    /// `gw_Replay_MatchRecordEnd`: the scene ended, a whole-match `.gdlp` is complete.
    pub fn match_record_end(&mut self) {
        if self.rec.gdlp && self.rec.frame > FIRST_FRAME - 1 {
            self.rec_close();
        }
    }

    fn rec_tick(&mut self, seed: u32) {
        if self.rec.file.is_none() || self.rec.frame == UNARMED {
            return;
        }
        self.rec_bookend(self.rec.frame); // the frame that just finished (.gdlp only)
        self.rec.frame += 1;
        self.rec.seed = seed;
        if self.rec.flush_next == UNARMED {
            self.rec.flush_next = self.rec.frame;
        }
        let mut ev = vec![0u8; 1 + replay::SZ_FSTART];
        ev[0] = 0x3A;
        ev[1..5].copy_from_slice(&be32(self.rec.frame as u32));
        ev[5..9].copy_from_slice(&be32(seed));
        self.rec.file.as_mut().unwrap().extend_from_slice(&ev);
    }

    /// `gw_Replay_RecordInput`: one fighter's processed input at the pre-frame point.
    #[allow(clippy::too_many_arguments)]
    pub fn record_input(
        &mut self,
        port: i32,
        follower: bool,
        lx: f32,
        ly: f32,
        cx: f32,
        cy: f32,
        trigger: f32,
        buttons: u32,
        action: i32,
        x: f32,
        y: f32,
        facing: f32,
        percent: f32,
    ) {
        let local = self.env.slippi_local_port;
        if local == port && self.active && !self.r.inputs.is_empty() && self.frame >= self.r.first && self.frame <= self.r.last {
            let want = self.r.inputs[(self.frame - self.r.first) as usize * SLOTS + port as usize * 2 + follower as usize];
            let got = [lx, ly, cx, cy, trigger];
            let wantf = [want.lx, want.ly, want.cx, want.cy, want.trigger];
            let same = got.iter().zip(wantf.iter()).all(|(a, b)| a.to_bits() == b.to_bits());
            if !want.present || !same || want.buttons != buttons {
                let n = self.slippi_local_mismatches;
                self.slippi_local_mismatches += 1;
                if n < 4 {
                    let m = format!(
                        "replay: Slippi local processed input mismatch at frame {} port {}{}",
                        self.frame,
                        port + 1,
                        if follower { " follower" } else { "" }
                    );
                    self.log(m);
                }
            }
        }
        if self.rec.file.is_none() || self.rec.frame == UNARMED {
            return;
        }
        let mut ev = vec![0u8; 1 + replay::SZ_PRE];
        ev[0] = 0x37;
        ev[1..5].copy_from_slice(&be32(self.rec.frame as u32));
        ev[5] = port as u8;
        ev[6] = follower as u8;
        ev[0x7..0xB].copy_from_slice(&be32(self.rec.seed));
        ev[0xB] = (action >> 8) as u8;
        ev[0xC] = action as u8;
        let bef = |ev: &mut [u8], o: usize, f: f32| ev[o..o + 4].copy_from_slice(&be32(f.to_bits()));
        bef(&mut ev, 0xD, x);
        bef(&mut ev, 0x11, y);
        bef(&mut ev, 0x15, facing);
        bef(&mut ev, 0x19, lx);
        bef(&mut ev, 0x1D, ly);
        bef(&mut ev, 0x21, cx);
        bef(&mut ev, 0x25, cy);
        bef(&mut ev, 0x29, trigger);
        ev[0x2D..0x31].copy_from_slice(&be32(buttons));
        bef(&mut ev, 0x3C, percent);
        if let Some(r) = self.rp_cur(port, follower).copied() {
            ev[0x3B] = r.raw[0] as u8;
            ev[0x40] = r.raw[1] as u8;
            ev[0x41] = r.raw[2] as u8;
            ev[0x42] = r.raw[3] as u8;
            if r.physical_complete {
                ev[0x31] = (r.physical_buttons >> 8) as u8;
                ev[0x32] = r.physical_buttons as u8;
                bef(&mut ev, 0x33, r.physical_l);
                bef(&mut ev, 0x37, r.physical_r);
            }
        }
        self.rec.file.as_mut().unwrap().extend_from_slice(&ev);
    }

    #[allow(clippy::too_many_arguments)]
    fn rec_post(
        &mut self,
        port: i32,
        follower: bool,
        ckind: i32,
        action: i32,
        x: f32,
        y: f32,
        facing: f32,
        percent: f32,
        stocks: i32,
        air_x: f32,
        air_y: f32,
        kb_x: f32,
        kb_y: f32,
        ground_x: f32,
    ) {
        if self.rec.file.is_none() || self.rec.frame == UNARMED {
            return;
        }
        let mut ev = vec![0u8; 1 + replay::SZ_POST];
        ev[0] = 0x38;
        ev[1..5].copy_from_slice(&be32(self.rec.frame as u32));
        ev[5] = port as u8;
        ev[6] = follower as u8;
        ev[7] = ckind as u8;
        ev[8] = (action >> 8) as u8;
        ev[9] = action as u8;
        let bef = |ev: &mut [u8], o: usize, f: f32| ev[o..o + 4].copy_from_slice(&be32(f.to_bits()));
        bef(&mut ev, 0xA, x);
        bef(&mut ev, 0xE, y);
        bef(&mut ev, 0x12, facing);
        bef(&mut ev, 0x16, percent);
        ev[0x21] = stocks as u8;
        bef(&mut ev, 0x35, air_x);
        bef(&mut ev, 0x39, air_y);
        bef(&mut ev, 0x3D, kb_x);
        bef(&mut ev, 0x41, kb_y);
        bef(&mut ev, 0x45, ground_x);
        self.rec.file.as_mut().unwrap().extend_from_slice(&ev);
    }

    /// `rec_close` / `gw_Replay_FinishRecording`: Game End, metadata, patched raw length. Returns
    /// whether a recording was open and finished without error.
    pub fn finish_recording(&mut self) -> bool {
        let opened = self.rec.file.is_some();
        self.rec_close();
        opened && !self.rec.failed
    }

    fn rec_close(&mut self) {
        let Some(mut f) = self.rec.file.take() else { return };
        let mut meta = b"U\x08metadata{U\x08playedOnSU\x08melee-pc".to_vec();
        if let Some((nm, dg)) = self.env.stage_skin.clone() {
            let val = if dg != 0 {
                let name: String = nm.chars().take(60).collect();
                format!("{:012x}|{}", dg & 0xFFFF_FFFF_FFFF, name)
            } else {
                "orig|Original".to_string()
            };
            if !val.is_empty() && val.len() < 96 {
                meta.extend_from_slice(b"U\x09stageSkinSU");
                meta.push(val.len() as u8);
                meta.extend_from_slice(val.as_bytes());
            }
        }
        meta.extend_from_slice(b"}}");
        if self.rec.gdlp && self.rec.frame != UNARMED {
            let fr = self.rec.frame;
            self.rec.file = Some(f);
            self.rec_bookend(fr);
            f = self.rec.file.take().unwrap();
        }
        let mut ev = [0u8; 1 + replay::SZ_END];
        ev[0] = 0x39;
        ev[1] = 7;
        f.extend_from_slice(&ev);
        let end = f.len();
        if end < self.rec.raw_start {
            self.rec.failed = true;
        }
        f.extend_from_slice(&meta);
        let len = (end - self.rec.raw_start) as u32;
        f[self.rec.raw_len_at..self.rec.raw_len_at + 4].copy_from_slice(&be32(len));
        let suffix = if self.rec.gdlp { format!(" {}", self.rec.gdlp_path) } else { String::new() };
        self.log(format!(
            "replay: recording closed ({} event bytes){}{}",
            end - self.rec.raw_start,
            if self.rec.gdlp { " " } else { "" },
            suffix.trim_start()
        ));
        let path = if self.rec.gdlp { self.rec.gdlp_path.clone() } else { String::new() };
        self.rec.closed_all.push((path, f.clone()));
        self.rec.closed = Some(f);
        if self.rec.gdlp {
            self.rec.gdlp = false;
            self.rec.frame = UNARMED;
            self.rec.flush_next = UNARMED;
        }
    }
}

/// The seed the console forced at the start of an online frame, from the first recorded Pre-Frame seed `rec`, the Frame Start
/// event's `sd` and the offset seen so far (updated when the recording shows a new one). None: not identifiable this frame.
/// The console draws k times (k <= 64) between the forced seed and the first fighter's Pre-Frame point.
pub fn online_forced_seed(rec: u32, sd: u32, off: &mut u32) -> Option<u32> {
    const INV: u32 = {
        let a: u32 = 214013;
        let mut x: u32 = a;
        let mut i = 0;
        while i < 5 {
            x = x.wrapping_mul(2u32.wrapping_sub(a.wrapping_mul(x)));
            i += 1;
        }
        x
    };
    let want = sd.wrapping_add(*off);
    let mut x = rec;
    let mut cand = None;
    for _ in 0..=64 {
        if x == want {
            return Some(want);
        }
        if cand.is_none() && x & 0xFFFF == sd & 0xFFFF && (x.wrapping_sub(sd) as i32 >> 16).unsigned_abs() < 4096 {
            cand = Some(x);
        }
        x = x.wrapping_sub(2531011).wrapping_mul(INV);
    }
    let c = cand?;
    *off = c.wrapping_sub(sd);
    Some(c)
}

fn raw_c_from_processed(cx: f32, cy: f32, which: i32) -> i32 {
    let (v, o) = if which == 2 { (cx, cy) } else { (cy, cx) };
    let b = (v * 80.0).round() as i32;
    if o == 0.0 && b.abs() == 79 {
        b.signum() * 80
    } else {
        b
    }
}
