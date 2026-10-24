//! The Geno Lab's long rewind (gw_script.c "the long rewind": `gd.history` / `step_back` / `rewind_to` / `rewind_test`,
//! the per-frame input log, the keyframe cadence) and the hot reload that sits on it (docs/geno.md 14.10).
//! The keyframes themselves are the game's (`ScriptGame::rw_*`, gw_snap.c); the script half keeps the cadence, the
//! input log and the requests. A frame's `tag` is the match frame about to run (`ring_now`).
use crate::engine::{Engine, State, GS_GREEN, GS_RED, GS_YELLOW};
use crate::gd::savestate::SaveSlot;
use mlua::Value;

pub const GS_RW_MAX_FRAMES: i32 = 60 * 60;
pub const GS_RW_INTERVAL: i32 = 30;
pub const GS_LOG_N: usize = 4096;
pub const GS_LOG_SFX: usize = 16;
pub const GS_LOG_Q: usize = 48;

/// `GsLogEntry`
#[derive(Clone)]
pub struct LogEntry {
    pub tag: i32,
    /// the frame renewed the pads (else it kept the previous statuses)
    pub have: bool,
    /// the sound handles are still being recorded (the frame's first run)
    pub sfx_open: bool,
    pub pad: [u8; 64],
    /// CPU/script input for fighter slots 5-6
    pub virtual_pad: [[i32; 9]; 2],
    pub sfx: Vec<(i32, i32)>,
    pub q: Vec<(u8, i32, i32)>,
}

impl Default for LogEntry {
    fn default() -> Self {
        LogEntry { tag: i32::MIN, have: false, sfx_open: false, pad: [0; 64], virtual_pad: [[0; 9]; 2], sfx: Vec::new(), q: Vec::new() }
    }
}

/// The rewind / self-test / hot-reload / state-library bookkeeping (the `rw_*`, `rwt_*`, `hot_*`, `st_*` fields of `gs`).
pub struct Rewind {
    pub frames: i32,
    pub interval: i32,
    pub began: bool,
    pub head: i32,
    pub log_start: i32,
    pub force_key: bool,
    pub pending_on: bool,
    pub pending_to: i32,
    pub resim: i32,
    pub resim_run: i32,
    pub notify: bool,
    pub t0: f64,
    pub last_ms: f64,
    pub last_load_ms: f64,
    pub last_frames: i32,
    pub rwt_phase: i32,
    pub rwt_back: i32,
    pub rwt_keep: bool,
    pub rwt_arm: i32,
    pub rwt_pass: i32,
    pub rwt_tag: i32,
    pub rwt_diff: [f64; 2],
    pub rwt_text: String,
    pub hot_phase: i32,
    pub hot_start: i32,
    pub hot_ok: bool,
    pub hot_text: String,
    pub st_pending_save: String,
    pub st_pending_load: String,
    pub st_save_name: String,
    pub st_save_desc: String,
    pub st_gen: i32,
    pub st_counter: i32,
    pub log: Vec<LogEntry>,
    pub sfx_claimed: u32,
    pub q_pos: usize,
    pub in_frame: bool,
    /// this logic frame: 0 live, 1 re-simulated (silent), 2 a burst's last
    pub frame_kind: i32,
    pub sfx_map: [(i32, i32); 16],
    pub sfx_map_next: usize,
    /// `gs_lab_handle` / `gs_lab_value` / `gs_lab_real`
    pub lab_handle: i32,
    pub lab_value: i32,
    pub lab_real: i32,
}

impl Default for Rewind {
    fn default() -> Self {
        Rewind {
            frames: 0,
            interval: 0,
            began: false,
            head: 0,
            log_start: 0,
            force_key: false,
            pending_on: false,
            pending_to: 0,
            resim: 0,
            resim_run: 0,
            notify: false,
            t0: 0.0,
            last_ms: 0.0,
            last_load_ms: 0.0,
            last_frames: 0,
            rwt_phase: 0,
            rwt_back: 0,
            rwt_keep: false,
            rwt_arm: 0,
            rwt_pass: -1,
            rwt_tag: 0,
            rwt_diff: [0.0; 2],
            rwt_text: String::new(),
            hot_phase: 0,
            hot_start: 0,
            hot_ok: false,
            hot_text: String::new(),
            st_pending_save: String::new(),
            st_pending_load: String::new(),
            st_save_name: String::new(),
            st_save_desc: String::new(),
            st_gen: 1,
            st_counter: 0,
            log: Vec::new(),
            sfx_claimed: 0,
            q_pos: 0,
            in_frame: false,
            frame_kind: 0,
            sfx_map: [(-1, -1); 16],
            sfx_map_next: 0,
            lab_handle: -1,
            lab_value: 0,
            lab_real: 0,
        }
    }
}

/// The script bookkeeping saved beside a keyframe (`GsSaveSlot` as bytes).
pub fn slot_to_bytes(s: &SaveSlot) -> Vec<u8> {
    let mut v = Vec::with_capacity(SLOT_BYTES);
    for x in [s.used as i32, s.scene_epoch, s.frame, s.match_frame] {
        v.extend_from_slice(&x.to_le_bytes());
    }
    for x in s.last_action.iter().chain(s.state_frame.iter()) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    v
}
pub const SLOT_BYTES: usize = 4 * (4 + 12);

pub fn slot_from_bytes(b: &[u8]) -> Option<SaveSlot> {
    if b.len() < SLOT_BYTES {
        return None;
    }
    let r = |i: usize| i32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
    let mut s = SaveSlot { used: r(0) != 0, scene_epoch: r(1), frame: r(2), match_frame: r(3), ..Default::default() };
    for k in 0..6 {
        s.last_action[k] = r(4 + k);
        s.state_frame[k] = r(10 + k);
    }
    Some(s)
}

impl State {
    /// `gs_ring_now`: the rewind's "now", the match frame about to run
    pub fn ring_now(&self) -> i32 {
        if self.match_active {
            self.match_frame + 1
        } else {
            0
        }
    }

    fn slot_now(&self) -> SaveSlot {
        SaveSlot {
            used: true,
            scene_epoch: self.scene_epoch,
            frame: self.frame,
            match_frame: self.match_frame,
            last_action: self.last_action,
            state_frame: self.state_frame,
        }
    }

    pub fn log_entry_mut(&mut self, tag: i32) -> &mut LogEntry {
        self.log_entry(tag)
    }

    fn log_entry(&mut self, tag: i32) -> &mut LogEntry {
        if self.rw.log.is_empty() {
            self.rw.log = vec![LogEntry::default(); GS_LOG_N];
        }
        &mut self.rw.log[(tag as u32 as usize) % GS_LOG_N]
    }

    /// `gs_rw_replaying`
    pub fn rw_replaying(&self, tag: i32) -> bool {
        self.rw.began && tag >= self.rw.log_start && tag < self.rw.head
    }

    /// `gs_rw_stop`
    pub fn rw_stop(&mut self) {
        self.game.rw_end();
        self.rw.began = false;
        self.rw.resim = 0;
        self.rw.resim_run = 0;
        self.rw.pending_on = false;
        self.rw.notify = false;
        self.rw.head = 0;
        self.rw.log_start = 0;
        self.game.snap_lab_resim(false);
    }

    /// `gs_rw_branch`: a write changed the game at this boundary; the logged frames from here on belong to the old
    /// timeline, and the next boundary keeps a keyframe of the new one.
    pub fn rw_branch(&mut self) {
        let now = self.ring_now();
        if !self.rw.began {
            return;
        }
        if self.rw.head > now {
            let who = self.cur.filter(|&i| i < self.scripts.len()).map(|i| self.scripts[i].id.clone()).unwrap_or_else(|| "-".into());
            self.log(&format!(
                "rewind: a gameplay write from Lua (script {who}) dropped {} logged frames (replay ends here)",
                self.rw.head - now
            ));
            self.rw.head = now;
        }
        if self.game.rw_drop_from(now) != 0 {
            self.rw.began = false; // the base itself is this frame or later: start over here
        }
        self.rw.force_key = true;
    }

    /// `gs_rw_request`: queue a rewind to `target` for the next frame boundary.
    pub fn rw_request(&mut self, target: i32) -> Result<(), &'static str> {
        if self.game.stage_slots_active() {
            return Err("stage slot session active: queue/cache cannot resimulate");
        }
        if self.game.mode_active() {
            return Err("mode state active: Lua director cannot resimulate");
        }
        if self.rw.frames <= 0 || !self.rw.began {
            return Err("history is off (gd.history(n) turns it on)");
        }
        if self.host.borrow().session() {
            return Err("not during a netplay/rollback session");
        }
        if self.rw.resim > 0 || self.rw.resim_run > 0 || self.rw.pending_on {
            return Err("busy rewinding");
        }
        if target < self.game.rw_base_tag() || target < self.rw.log_start {
            return Err("not that far back in the history");
        }
        if target > self.rw.head {
            return Err("not that far forward");
        }
        self.rw.pending_on = true;
        self.rw.pending_to = target;
        self.paused = true;
        self.step = 0;
        Ok(())
    }

    /// `gs_rw_frame`: at every frame boundary start the history, keep a keyframe every interval, slide the window.
    pub fn rw_frame(&mut self) {
        let now = self.ring_now();
        if self.rw.frames <= 0 || !self.match_active || self.host.borrow().session() {
            return;
        }
        if self.rw.began && now > self.rw.head {
            self.rw.began = false; // frames ran that the log did not see
        }
        if !self.rw.began {
            let u = slot_to_bytes(&self.slot_now());
            if self.game.rw_begin(now, &u) != 0 {
                self.print(GS_RED, "history unavailable (no memory for the base image)");
                self.rw.frames = 0;
                return;
            }
            self.rw.began = true;
            self.rw.head = now;
            self.rw.log_start = now;
            self.rw.force_key = false;
            let (f, i) = (self.rw.frames, self.rw.interval);
            self.log(&format!("rewind: on at frame {now}: {f} frames, a keyframe every {i}"));
            return;
        }
        let latest = self.game.rw_latest();
        if now > latest && (self.rw.force_key || now - latest >= self.rw.interval) {
            let u = slot_to_bytes(&self.slot_now());
            if self.game.rw_save(now, &u) != 0 {
                self.log(&format!("rewind: keyframe at frame {now} failed - starting over"));
                self.rw.began = false;
                return;
            }
            self.rw.force_key = false;
        }
        let edge = self.rw.head.max(now);
        self.game.rw_trim(edge - self.rw.frames);
        let base = self.game.rw_base_tag();
        if self.rw.log_start < base {
            self.rw.log_start = base;
        }
    }

    pub fn rwt_fail_pub(&mut self, text: &str) {
        self.rwt_fail(text)
    }

    fn rwt_fail(&mut self, text: &str) {
        let _ = self.game.rw_compare(); // frees the copy
        self.game.rw_measure(false);
        self.rw.rwt_arm = 0;
        self.rw.rwt_phase = 0;
        self.rw.rwt_pass = 0;
        self.rw.rwt_text = format!("FAIL: {text}");
        let t = self.rw.rwt_text.clone();
        self.print(GS_RED, &format!("rewind test: {t}"));
        self.log(&format!("rewind test: {t}"));
    }

    /// `gs_rwt_frame`: the exactness self-test (`gd.rewind_test`), at the very start of a frame
    pub fn rwt_frame(&mut self) {
        let now = self.ring_now();
        if self.rw.rwt_phase == 1 {
            if !self.rw.began || !self.match_active {
                self.rwt_fail("history is off");
                return;
            }
            // measure render-owned bytes from here on, and capture once a keyframe has been kept since
            if self.rw.rwt_arm == 0 {
                self.game.rw_measure(true);
                self.rw.rwt_arm = now;
                return;
            }
            if self.game.rw_latest() < self.rw.rwt_arm {
                return;
            }
            self.rw.rwt_arm = 0;
            if self.game.rw_capture() != 0 {
                self.rwt_fail("no memory for the copy");
                return;
            }
            self.rw.rwt_tag = now;
            self.rw.rwt_phase = 2;
        } else if self.rw.rwt_phase == 2 && now >= self.rw.rwt_tag + self.rw.rwt_back {
            let tag = self.rw.rwt_tag;
            match self.rw_request(tag) {
                Err(why) => self.rwt_fail(why),
                Ok(()) => self.rw.rwt_phase = 3,
            }
        } else if self.rw.rwt_phase == 4 {
            if !self.rw.rwt_keep {
                self.paused = true;
                self.step = 0;
            }
            if now != self.rw.rwt_tag {
                let t = format!("landed on frame {now}, not {}", self.rw.rwt_tag);
                self.rwt_fail(&t);
                return;
            }
            self.rw.rwt_phase = 0;
            let Some((d, h)) = self.game.rw_compare() else {
                self.rwt_fail("the copy was lost");
                return;
            };
            self.game.rw_measure(false);
            self.rw.rwt_diff = d;
            self.rw.rwt_pass = (d[1] == 0.0 && h[0] == h[1]) as i32;
            let pass = self.rw.rwt_pass != 0;
            self.rw.rwt_text = format!(
                "{}: frame {}, {} frames later rewound to it (keyframe load {:.2} ms + {} frames re-simulated, {:.1} ms): {:.0} simulation bytes differ, hash {:016x} vs {:016x} ({:.0} counting render-owned bytes, the disc's async blocks and pad-side rumble)",
                if pass { "PASS" } else { "FAIL" },
                now,
                self.rw.rwt_back,
                self.rw.last_load_ms,
                self.rw.last_frames,
                self.rw.last_ms,
                d[1],
                h[0],
                h[1],
                d[0]
            );
            let t = self.rw.rwt_text.clone();
            self.print(if pass { GS_GREEN } else { GS_RED }, &format!("rewind test: {t}"));
            self.log(&format!("rewind test: {t}"));
        }
    }

    // ---- the input log (gw_LabPad_Record / gw_LabPad_Replay)

    /// `gw_LabPad_Replay`: 0 = not replaying, 1 = `out` holds the logged statuses, 2 = the frame renewed nothing when it first ran.
    pub fn pad_replay(&mut self, out: &mut [u8]) -> i32 {
        if !self.match_active {
            return 0;
        }
        let tag = self.ring_now();
        if !self.rw.began || !self.rw_replaying(tag) {
            return 0; // (a LAB recording playing back is gw_script_rec.inc, not ported)
        }
        let e = self.log_entry(tag).clone();
        if e.tag != tag {
            return 0;
        }
        for i in 0..2 {
            for k in 0..9 {
                self.game.virtual_pad_field(i as i32 + 4, k as i32, e.virtual_pad[i][k], true);
            }
        }
        if !e.have {
            return 2;
        }
        let n = out.len().min(e.pad.len());
        out[..n].copy_from_slice(&e.pad[..n]);
        1
    }

    /// `gw_LabPad_Record`: a live frame's statuses (None: it renewed nothing), logged at the live edge.
    pub fn pad_record(&mut self, st: Option<&[u8]>) {
        if !self.rw.began || !self.match_active || self.host.borrow().session() {
            return;
        }
        let tag = self.ring_now();
        if self.rw_replaying(tag) {
            return;
        }
        if tag != self.rw.head {
            self.rw.began = false; // a gap in the log: the next boundary starts the history over
            return;
        }
        let mut vp = [[0i32; 9]; 2];
        for i in 0..2 {
            for k in 0..9 {
                vp[i][k] = self.game.virtual_pad_field(i as i32 + 4, k as i32, 0, false);
            }
        }
        let e = self.log_entry(tag);
        e.tag = tag;
        e.have = st.is_some();
        e.virtual_pad = vp;
        if let Some(st) = st {
            let n = st.len().min(e.pad.len());
            e.pad[..n].copy_from_slice(&st[..n]);
        }
        e.sfx.clear();
        e.q.clear();
        e.sfx_open = true;
        self.rw.head = tag + 1;
    }

    /// `gw_LabSfx_Replay`: 0 = not replaying (play, then Record); 1 = the logged handle, silent; 2 = the logged handle,
    /// play it too; 3 = not logged: play it. The handle is `self.rw.lab_handle`.
    pub fn sfx_replay(&mut self, sound_id: i32) -> i32 {
        self.rw.lab_handle = -1;
        if !self.rw.began || !self.match_active || !self.rw.in_frame {
            return 0;
        }
        let tag = self.ring_now();
        if !self.rw_replaying(tag) {
            return 0;
        }
        let claimed = self.rw.sfx_claimed;
        let kind = self.rw.frame_kind;
        let e = self.log_entry(tag);
        if e.tag != tag {
            return 3;
        }
        if e.sfx_open {
            return 0; // the frame's first pass on this timeline (after a hot reload): record
        }
        for (i, &(id, h)) in e.sfx.iter().enumerate() {
            if claimed & (1 << i) == 0 && id == sound_id {
                self.rw.sfx_claimed |= 1 << i;
                self.rw.lab_handle = h;
                return if kind != 0 { 1 } else { 2 };
            }
        }
        3
    }

    /// `gw_LabSfx_Record`
    pub fn sfx_record(&mut self, sound_id: i32, handle: i32) {
        if !self.rw.began || !self.match_active || !self.rw.in_frame {
            return;
        }
        let tag = self.ring_now();
        let e = self.log_entry(tag);
        if e.tag == tag && e.sfx_open && e.sfx.len() < GS_LOG_SFX {
            e.sfx.push((sound_id, handle));
        }
    }

    /// `gw_LabSfx_Map`
    pub fn sfx_map(&mut self, logged: i32, real: i32) {
        if logged == real || logged < 0 {
            return;
        }
        let n = self.rw.sfx_map_next;
        self.rw.sfx_map[n] = (logged, real);
        self.rw.sfx_map_next = (n + 1) % self.rw.sfx_map.len();
    }

    fn sfx_real(&self, h: i32) -> i32 {
        for &(from, to) in self.rw.sfx_map.iter() {
            if from == h && h >= 0 {
                return to;
            }
        }
        h
    }

    /// `gw_LabAudio_Replay`: 0 = live (call it, then Record); 1 = re-simulated (the logged answer, no side effect); 2 = replayed at
    /// speed (the logged answer; the call goes to the voice the replay started). The answer is `self.rw.lab_value`.
    pub fn audio_replay(&mut self, kind: i32, arg: i32) -> i32 {
        self.rw.lab_value = 0;
        self.rw.lab_real = self.sfx_real(arg);
        if !self.rw.began || !self.match_active || !self.rw.in_frame {
            return 0;
        }
        let tag = self.ring_now();
        if !self.rw_replaying(tag) {
            return 0;
        }
        let (pos, fk) = (self.rw.q_pos, self.rw.frame_kind);
        let e = self.log_entry(tag);
        if e.tag != tag || e.sfx_open {
            return 0;
        }
        if pos < e.q.len() && e.q[pos].0 as i32 == kind && e.q[pos].1 == arg {
            let v = e.q[pos].2;
            self.rw.q_pos += 1;
            self.rw.lab_value = v;
            return if fk != 0 { 1 } else { 2 };
        }
        0 // the replay asks something the first run did not: answer live
    }

    /// `gw_LabAudio_Record`
    pub fn audio_record(&mut self, kind: i32, arg: i32, value: i32) {
        if !self.rw.began || !self.match_active || !self.rw.in_frame {
            return;
        }
        let tag = self.ring_now();
        let e = self.log_entry(tag);
        if e.tag == tag && e.sfx_open && e.q.len() < GS_LOG_Q {
            e.q.push((kind as u8, arg, value));
        }
    }

    /// `gw_Script_Iterations`: how many logic frames this tick runs. A rewind re-simulates every frame from the keyframe to
    /// the target in one tick; paused, only the requested steps (up to 30 per tick).
    pub fn iterations(&mut self, count: i32) -> i32 {
        let session = self.host.borrow().session();
        if self.rw.resim > 0 && !session {
            let n = self.rw.resim;
            self.rw.resim = 0;
            self.rw.resim_run = n;
            return n;
        }
        if !self.paused || session {
            return count;
        }
        if self.step > 0 && count > 0 {
            let n = self.step.min(30);
            self.step -= n;
            return n;
        }
        0
    }
}

impl Engine {
    /// `gs_hot_do`: the hot reload, once the rewind to its start frame is done (the loop top, between frames)
    fn hot_do(&self) {
        let (alive, now) = {
            let mut s = self.st.borrow_mut();
            s.rw.hot_phase = 0;
            (s.match_active && s.game.geno_lab_mode_active(), s.ring_now())
        };
        if !alive {
            {
                let mut s = self.st.borrow_mut();
                s.rw.hot_ok = false;
                s.paused = false;
                s.step = 0;
                s.rw.hot_text = "reload cancelled: LAB match ended".into();
                let t = s.rw.hot_text.clone();
                s.print(GS_YELLOW, &format!("hot reload: {t}"));
                s.log(&format!("hot reload: {t}"));
            }
            self.hot_done(false);
            return;
        }
        let mut msg = String::new();
        let safe = self.st.borrow_mut().game.geno_reload(&mut msg);
        if safe < 0 {
            {
                let mut s = self.st.borrow_mut();
                s.rw.hot_ok = false;
                let m = if msg.is_empty() { "geno.json could not be loaded" } else { &msg };
                s.rw.hot_text = format!("{m}; reload refused, previous data retained");
                let t = s.rw.hot_text.clone();
                s.print(GS_RED, &format!("hot reload: {t}"));
                s.log(&format!("hot reload: {t}"));
            }
            self.hot_done(false);
            return;
        }
        if safe == 0 {
            {
                let mut s = self.st.borrow_mut();
                s.rw.hot_text = format!("{msg} - the state no longer fits the fighters' layout: restarting the match");
                s.rw.hot_ok = false;
                let t = s.rw.hot_text.clone();
                s.print(GS_YELLOW, &format!("hot reload: {t}"));
                s.log(&format!("hot reload: {t}"));
                s.paused = false;
                s.step = 0;
                s.rw_stop();
                if !s.game.geno_lab_leave(3) {
                    s.paused = true;
                    s.print(GS_RED, "hot reload: not a LAB match - restart it yourself");
                }
            }
            self.hot_done(false);
            return;
        }
        let n = self.st.borrow_mut().game.geno_lab_reload();
        let script_rc = self.exec("reload geno-lab/lab");
        {
            let mut s = self.st.borrow_mut();
            // the timeline from here on is the new data's: re-record its sounds, restart the history here,
            // keep the logged input to replay
            if s.rw.began {
                for t in now..s.rw.head {
                    let e = s.log_entry(t);
                    if e.tag == t {
                        e.sfx_open = true;
                        e.sfx.clear();
                        e.q.clear();
                    }
                }
                let u = slot_to_bytes(&s.slot_now());
                if s.game.rw_begin(now, &u) != 0 {
                    s.rw_stop();
                } else {
                    s.rw.log_start = now;
                    s.rw.force_key = false;
                }
            }
            s.rw.hot_ok = script_rc == 0;
            let replay = if s.rw.began && s.rw.head > now { (s.rw.head - now) as f64 / 60.0 } else { 0.0 };
            s.rw.hot_text = format!(
                "{}; {} fighter(s) re-applied; Lab script {}; replaying {:.1} s",
                if msg.is_empty() { "geno.json unchanged" } else { &msg },
                n,
                if script_rc == 0 { "reloaded" } else { "NOT reloaded" },
                replay
            );
            let (ok, t) = (s.rw.hot_ok, s.rw.hot_text.clone());
            s.print(if ok { GS_GREEN } else { GS_RED }, &format!("hot reload: {t}"));
            s.log(&format!("hot reload: {t}"));
            s.paused = !ok;
            s.step = 0;
        }
        let ok = self.st.borrow().rw.hot_ok;
        self.hot_done(ok);
    }

    fn hot_done(&self, ok: bool) {
        self.hook_all("on_hot_reload", &[Value::Boolean(ok)]);
    }

    /// `gs_rw_tick_top`: the loop top, the work that waits for a finished re-simulation
    pub fn rw_tick_top(&self) {
        let (notify_now, hot) = {
            let mut s = self.st.borrow_mut();
            if s.rw.resim_run > 0 {
                s.rw.resim_run = 0; // the loop ended the burst early (a scene change)
                s.game.snap_lab_resim(false);
            }
            if s.rw.resim > 0 {
                return;
            }
            let mut notify = false;
            if s.rw.notify {
                s.rw.notify = false;
                s.nev_reset();
                notify = true;
                if s.rw.rwt_phase == 3 {
                    s.rw.rwt_phase = 4;
                    s.paused = false; // the self-test compares at the next frame's start
                    s.step = 0;
                }
            }
            (notify, s.rw.hot_phase == 2 && !s.rw.pending_on)
        };
        if notify_now {
            self.hook_all("on_loadstate", &[Value::Integer(0)]);
        }
        if hot {
            self.hot_do();
        }
    }
}
