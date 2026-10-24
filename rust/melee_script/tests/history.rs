//! The long rewind (`gd.history`, `step_back`, `rewind_to`, `rewind_test`), the state library (`gd.state_*`), the hot reload and
//! `gd.kb_preview`, over a mock game with an in-memory keyframe store.
use melee_script::game::*;
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Store {
    keys: Vec<(i32, Vec<u8>)>,
    ended: bool,
    reload_calls: i32,
    leaves: Vec<i32>,
    resim_flags: Vec<bool>,
    capture: bool,
    measure: bool,
}
struct G {
    s: Rc<RefCell<Store>>,
    lab: bool,
    reload_result: i32,
}
impl ScriptGame for G {
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        match field {
            SI_PRESENT => (slot < 2) as i32,
            SI_CHAR => 1 + slot,
            SI_COSTUME => 0,
            SI_SLOT_TYPE => 0,
            _ => 0,
        }
    }
    fn fighter_f(&self, slot: i32, field: i32) -> f32 {
        if field == SF_X { slot as f32 * 20.0 } else { 0.0 }
    }
    fn stage_kind(&self) -> i32 {
        32
    }
    fn geno_lab_mode_active(&self) -> bool {
        self.lab
    }
    fn rw_begin(&mut self, tag: i32, user: &[u8]) -> i32 {
        let mut s = self.s.borrow_mut();
        s.keys.clear();
        s.keys.push((tag, user.to_vec()));
        0
    }
    fn rw_save(&mut self, tag: i32, user: &[u8]) -> i32 {
        self.s.borrow_mut().keys.push((tag, user.to_vec()));
        0
    }
    fn rw_load(&mut self, k: i32) -> Option<Vec<u8>> {
        self.s.borrow().keys.iter().find(|(t, _)| *t == k).map(|(_, u)| u.clone())
    }
    fn rw_drop_from(&mut self, tag: i32) -> i32 {
        let mut s = self.s.borrow_mut();
        s.keys.retain(|(t, _)| *t < tag);
        s.keys.is_empty() as i32
    }
    fn rw_trim(&mut self, limit: i32) {
        let mut s = self.s.borrow_mut();
        while s.keys.len() > 1 && s.keys[1].0 <= limit {
            s.keys.remove(0);
        }
    }
    fn rw_end(&mut self) {
        let mut s = self.s.borrow_mut();
        s.keys.clear();
        s.ended = true;
    }
    fn rw_find_le(&self, tag: i32) -> Option<i32> {
        self.s.borrow().keys.iter().rev().find(|(t, _)| *t <= tag).map(|(t, _)| *t)
    }
    fn rw_base_tag(&self) -> i32 {
        self.s.borrow().keys.first().map(|k| k.0).unwrap_or(0)
    }
    fn rw_latest(&self) -> i32 {
        self.s.borrow().keys.last().map(|k| k.0).unwrap_or(0)
    }
    fn rw_count(&self) -> i32 {
        self.s.borrow().keys.len() as i32
    }
    fn rw_capture(&mut self) -> i32 {
        self.s.borrow_mut().capture = true;
        0
    }
    fn rw_compare(&mut self) -> Option<([f64; 2], [u64; 2])> {
        let mut s = self.s.borrow_mut();
        if !s.capture {
            return None;
        }
        s.capture = false;
        Some(([12.0, 0.0], [0xabc, 0xabc]))
    }
    fn rw_measure(&mut self, on: bool) {
        self.s.borrow_mut().measure = on;
    }
    fn snap_lab_resim(&mut self, on: bool) {
        self.s.borrow_mut().resim_flags.push(on);
    }
    fn snap_file_save(&mut self, path: &str, hdr: &[u8]) -> i32 {
        std::fs::write(path, hdr).map(|_| 0).unwrap_or(-1)
    }
    fn snap_file_header(&self, path: &str, len: usize) -> Option<Vec<u8>> {
        let b = std::fs::read(path).ok()?;
        Some(b[..b.len().min(len)].to_vec())
    }
    fn snap_file_rewrite_header(&mut self, path: &str, hdr: &[u8]) -> i32 {
        std::fs::write(path, hdr).map(|_| 0).unwrap_or(-1)
    }
    fn snap_file_load(&mut self, _path: &str) -> i32 {
        0
    }
    fn identity(&self) -> [u64; 4] {
        [1, 2, 3, 4]
    }
    fn geno_reload(&mut self, msg: &mut String) -> i32 {
        self.s.borrow_mut().reload_calls += 1;
        *msg = "geno.json reloaded".into();
        self.reload_result
    }
    fn geno_lab_reload(&mut self) -> i32 {
        2
    }
    fn geno_lab_leave(&mut self, w: i32) -> bool {
        self.s.borrow_mut().leaves.push(w);
        true
    }
    // knockback preview scalars
    fn lab_knockback(&self, _s: i32, _a: i32, dmg: i32, _kbg: i32, _w: i32, _b: i32, _p: i32, _post: i32) -> f32 {
        f32::from_bits(dmg as u32) * 4.0
    }
    fn lab_kb_level(&self, kb: i32) -> i32 {
        if f32::from_bits(kb as u32) > 50.0 { 3 } else { 1 }
    }
    fn lab_common_f(&self, which: i32) -> f32 {
        match which {
            0 => 0.03,     // KB_SPEED
            6 => 0.4,      // HITSTUN_MUL
            7 => 18.0,     // DI_DEGREES
            8 => 0.051,    // KB_DECAY
            10 => -224.0,  // BLAST_LEFT
            11 => 224.0,
            12 => 180.0,
            13 => -108.0,
            _ => 0.0,
        }
    }
    fn lab_flight_f(&self, _slot: i32, which: i32) -> f32 {
        match which {
            0 => 0.095, // gravity
            1 => 1.7,   // terminal
            2 => 0.02,  // friction
            3 => 1.0,   // facing
            4 => 1.0,   // airborne
            5 => 100.0,
            6 => 30.0,
            _ => 0.0,
        }
    }
}

fn rig(tag: &str, lab: bool, reload_result: i32) -> (Engine, Rc<RefCell<Store>>, Rc<RefCell<MemHost>>, std::path::PathBuf) {
    let st = Rc::new(RefCell::new(Store::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_hist_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    let eng = Engine::new(host.clone(), Box::new(G { s: st.clone(), lab, reload_result }), cfg).unwrap();
    (eng, st, host, dir)
}

/// one logic frame as the loop runs it: FramePre, the pad read (recorded or replayed), FramePost
fn logic_frame(eng: &Engine, pad: u8) {
    eng.frame_pre();
    let mut out = [0u8; 64];
    let r = eng.st.borrow_mut().pad_replay(&mut out);
    if r == 0 {
        eng.st.borrow_mut().pad_record(Some(&[pad; 64]));
    } else {
        assert_eq!(r, 1);
        assert_eq!(out[0], pad_for_tag(eng), "the replayed frame feeds back the logged pad");
    }
    eng.frame_post();
}
fn pad_for_tag(eng: &Engine) -> u8 {
    (eng.st.borrow().ring_now() % 200) as u8
}
fn live_frame(eng: &Engine) {
    let p = (eng.st.borrow().ring_now() % 200) as u8;
    logic_frame(eng, p);
}

#[test]
fn history_keyframes_and_step_back_resimulate_on_the_log() {
    let (eng, store, host, _) = rig("hist", false, 1);
    eng.frame_post(); // the match starts: match_frame 0, next frame = 1
    let h: mlua::Table = eng.eval("return gd.history()").unwrap();
    assert_eq!(h.get::<i64>("depth").unwrap(), 0);
    let h: mlua::Table = eng.eval("return gd.history(120, 10)").unwrap();
    assert_eq!(h.get::<i64>("depth").unwrap(), 120);
    assert_eq!(h.get::<i64>("interval").unwrap(), 10);
    assert_eq!(h.get::<f64>("seconds").unwrap(), 2.0);
    for _ in 0..35 {
        live_frame(&eng);
    }
    let h: mlua::Table = eng.eval("return gd.history()").unwrap();
    let now = h.get::<i64>("now").unwrap();
    assert_eq!(now, 36);
    assert_eq!(h.get::<i64>("oldest").unwrap(), 1, "the base image was kept at frame 1");
    assert_eq!(h.get::<i64>("keys").unwrap(), 4, "a keyframe every 10 frames: 1, 11, 21, 31");
    assert_eq!(h.get::<i64>("head").unwrap(), 36);
    assert!(store.borrow().keys.iter().map(|k| k.0).eq([1, 11, 21, 31]));
    assert!(host.borrow().lines.iter().any(|l| l.starts_with("rewind: on at frame 1: 120 frames")));

    // step back 5 frames: queued at once, paused, carried out at the loop top
    let (ok, why): (bool, Option<String>) = eng.eval("return gd.step_back(5)").unwrap();
    assert!(ok && why.is_none());
    assert!(eng.eval::<bool>("return gd.paused()").unwrap());
    let h: mlua::Table = eng.eval("return gd.history()").unwrap();
    assert!(h.get::<bool>("busy").unwrap());
    eng.tick(); // paused: applies the pending rewind
    let n = eng.iterations(1);
    assert_eq!(n, 0, "the target is keyframe 31 itself: nothing to re-simulate");
    assert_eq!(eng.st.borrow().ring_now(), 31, "the keyframe's bookkeeping is back");
}

#[test]
fn rewind_replays_logged_input_silently_then_lands() {
    let (eng, store, host, _) = rig("resim", false, 1);
    eng.frame_post();
    eng.eval::<()>("gd.history(120, 10)").unwrap();
    for _ in 0..35 {
        live_frame(&eng);
    }
    // rewind to 26 (keyframe 21 -> 5 frames re-simulated)
    let (ok, _): (bool, Option<String>) = eng.eval("return gd.rewind_to(26)").unwrap();
    assert!(ok);
    eng.tick();
    let n = eng.iterations(1);
    assert_eq!(n, 5);
    for _ in 0..n {
        logic_frame(&eng, 0); // pad value is checked against the log by pad_for_tag
    }
    assert_eq!(eng.st.borrow().ring_now(), 26, "landed on the target frame");
    // the first four re-simulated frames are silent, the last is the burst's end
    assert_eq!(store.borrow().resim_flags, [true, true, true, true, false]);
    assert!(host.borrow().lines.iter().any(|l| l.starts_with("rewind: to frame 26:")));
    let h: mlua::Table = eng.eval("return gd.history()").unwrap();
    assert_eq!(h.get::<i64>("fwd").unwrap(), 10, "the log still holds frames 26..35");
    assert!(h.get::<bool>("replaying").unwrap());
    // a gameplay write ends the replay and drops the logged frames
    eng.eval::<()>("gd.set_percent(1, 40)").unwrap();
    let h: mlua::Table = eng.eval("return gd.history()").unwrap();
    assert_eq!(h.get::<i64>("fwd").unwrap(), 0);
    assert!(host.borrow().lines.iter().any(|l| l.contains("dropped 10 logged frames")));
    // out-of-range requests are refused with the C build's words
    let (ok, why): (bool, String) = eng.eval("return gd.rewind_to(9999)").unwrap();
    assert!(!ok && why == "not that far forward");
    let (ok, why): (bool, String) = eng.eval("return gd.step_back(500)").unwrap();
    assert!(!ok && why == "not that far back in the history");
}

#[test]
fn history_refusals() {
    let (eng, _, _, _) = rig("refuse", false, 1);
    let (ok, why): (bool, String) = eng.eval("return gd.step_back()").unwrap();
    assert!(!ok && why == "history is off (gd.history(n) turns it on)");
    let (ok, why): (bool, String) = eng.eval("return gd.rewind_test()").unwrap();
    assert!(!ok && why == "history is off (gd.history(n) turns it on)");
    let (ok, why): (bool, String) = eng.eval("return gd.hot_reload()").unwrap();
    assert!(!ok && why == "hot reload requires a LAB match");
    let h: mlua::Table = eng.eval("return gd.history(99999, 5000)").unwrap();
    assert_eq!(h.get::<i64>("depth").unwrap(), 3600, "a minute at most");
    assert_eq!(h.get::<i64>("interval").unwrap(), 600);
}

#[test]
fn rewind_test_compares_the_copy() {
    let (eng, store, host, _) = rig("rwt", false, 1);
    eng.frame_post();
    eng.eval::<()>("gd.history(120, 10)").unwrap();
    for _ in 0..3 {
        live_frame(&eng);
    }
    let (ok, why): (bool, Option<String>) = eng.eval("return gd.rewind_test(20)").unwrap();
    assert!(ok, "{why:?}");
    let r: mlua::Table = eng.eval("return gd.rewind_test_result()").unwrap();
    assert_eq!(r.get::<i64>("phase").unwrap(), 1);
    assert_eq!(r.get::<String>("text").unwrap(), "running");
    // run until the test has captured, rewound and compared
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 200, "the self-test never finished");
        let phase = eng.st.borrow().rw.rwt_phase;
        if phase == 0 {
            break;
        }
        if eng.st.borrow().paused {
            eng.tick();
            let n = eng.iterations(1);
            for _ in 0..n.max(1) {
                let p = pad_for_tag(&eng);
                logic_frame(&eng, p);
            }
            continue;
        }
        live_frame(&eng);
    }
    let r: mlua::Table = eng.eval("return gd.rewind_test_result()").unwrap();
    assert!(r.get::<bool>("pass").unwrap(), "{}", r.get::<String>("text").unwrap());
    assert_eq!(r.get::<f64>("diff").unwrap(), 12.0);
    assert!(r.get::<String>("text").unwrap().starts_with("PASS: frame"));
    assert!(!store.borrow().measure);
    assert!(host.borrow().lines.iter().any(|l| l.starts_with("rewind test: PASS")));
}

fn drive_paused(eng: &Engine) {
    eng.tick();
    let n = eng.iterations(1);
    for _ in 0..n {
        let p = pad_for_tag(eng);
        logic_frame(eng, p);
    }
}

#[test]
fn hot_reload_rewinds_reloads_and_replays() {
    let (eng, store, host, _) = rig("hot", true, 1);
    eng.frame_post();
    eng.eval::<()>("gd.history(300, 10) hot_seen = nil function on_hot_reload(ok) hot_seen = ok end").unwrap();
    // the console script owns hooks only when it is a script; use a loaded one
    for _ in 0..70 {
        live_frame(&eng);
    }
    let (ok, start): (bool, i64) = eng.eval("return gd.hot_reload(1)").unwrap();
    assert!(ok);
    assert_eq!(start, 71 - 60);
    let st: mlua::Table = eng.eval("return gd.hot_reload_status()").unwrap();
    assert_eq!(st.get::<i64>("phase").unwrap(), 1);
    assert_eq!(st.get::<String>("text").unwrap(), "reloading");
    let (ok2, why): (bool, String) = eng.eval("return gd.hot_reload()").unwrap();
    assert!(!ok2 && why == "a reload is already running");
    drive_paused(&eng); // the rewind
    drive_paused(&eng); // re-simulation done: the loop top reloads
    eng.tick();
    let st: mlua::Table = eng.eval("return gd.hot_reload_status()").unwrap();
    assert_eq!(st.get::<i64>("phase").unwrap(), 0, "{:?}", host.borrow().lines.last());
    assert_eq!(store.borrow().reload_calls, 1);
    let text = st.get::<String>("text").unwrap();
    assert!(text.starts_with("geno.json reloaded; 2 fighter(s) re-applied; Lab script"), "{text}");
    assert!(text.contains("replaying 1.0 s"), "{text}");
}

#[test]
fn hot_reload_restarts_the_match_when_the_layout_changed() {
    let (eng, store, _, _) = rig("hot0", true, 0);
    eng.frame_post();
    let (ok, _): (bool, i64) = eng.eval("return gd.hot_reload(0)").unwrap();
    assert!(ok, "no history: reload in place");
    eng.tick();
    assert_eq!(store.borrow().leaves, [3]);
    let st: mlua::Table = eng.eval("return gd.hot_reload_status()").unwrap();
    assert!(!st.get::<bool>("ok").unwrap());
    assert!(st.get::<String>("text").unwrap().contains("restarting the match"));
}

#[test]
fn state_library_roundtrip() {
    let (eng, _, host, dir) = rig("lib", false, 1);
    // not in a match yet
    let (ok, why): (bool, String) = eng.eval("return gd.state_save('a')").unwrap();
    assert!(!ok && why == "not in a match");
    eng.frame_post();
    let g0: i64 = eng.eval("return gd.state_gen()").unwrap();
    let file: String = eng.eval("return gd.state_save('Ledge setup', 'fox v falco')").unwrap();
    assert!(file.starts_with("st_") && file.ends_with(".gdst"), "{file}");
    let (ok, why): (bool, String) = eng.eval("return gd.state_save('b')").unwrap();
    assert!(!ok && why == "a save is already waiting");
    eng.frame_pre(); // the frame boundary writes it
    assert!(host.borrow().lines.iter().any(|l| l.contains("lab: state saved to")));
    assert_eq!(eng.eval::<i64>("return gd.state_gen()").unwrap(), g0 + 1);
    let list: mlua::Table = eng.eval("return gd.state_list()").unwrap();
    assert_eq!(list.raw_len(), 1);
    let row: mlua::Table = list.get(1).unwrap();
    assert_eq!(row.get::<String>("file").unwrap(), file);
    assert_eq!(row.get::<String>("name").unwrap(), "Ledge setup");
    assert_eq!(row.get::<String>("what").unwrap(), "fox v falco");
    assert_eq!(row.get::<i64>("frame").unwrap(), 0);
    assert_eq!(row.get::<i64>("stage").unwrap(), 32);
    assert!(row.get::<bool>("ok").unwrap(), "{}", row.get::<String>("why").unwrap());
    let fighters: mlua::Table = row.get("fighters").unwrap();
    assert_eq!(fighters.raw_len(), 2);
    let f2: mlua::Table = fighters.get(2).unwrap();
    assert_eq!(f2.get::<i64>("port").unwrap(), 2);
    assert_eq!(f2.get::<i64>("char").unwrap(), 2);
    // the index.txt is rewritten beside the states
    let idx = std::fs::read_to_string(dir.join("data").join("geno-lab_lab").join("states").join("index.txt")).unwrap();
    assert!(idx.starts_with("# the Geno Lab's saved states"));
    assert!(idx.contains(&format!("{file}\tf0\t")) && idx.contains("\tLedge setup\tfox v falco\n"), "{idx}");
    // rename, load, delete
    assert!(eng.eval::<bool>(&format!("return gd.state_rename('{file}', 'Renamed')")).unwrap());
    let row: mlua::Table = eng.eval("return gd.state_list()[1]").unwrap();
    assert_eq!(row.get::<String>("name").unwrap(), "Renamed");
    let (ok, why): (bool, String) = eng.eval("return gd.state_load('../evil.gdst')").unwrap();
    assert!(!ok && why == "not a state file name");
    let (ok, why): (bool, String) = eng.eval("return gd.state_load('nope.gdst')").unwrap();
    assert!(!ok && why == "no such state");
    assert!(eng.eval::<bool>(&format!("return gd.state_load('{file}')")).unwrap());
    eng.frame_pre();
    assert!(host.borrow().lines.iter().any(|l| l.contains("state loaded: Renamed")) || eng.console_lines().iter().any(|l| l.contains("state loaded: Renamed (frame 0)")));
    assert!(eng.eval::<bool>(&format!("return gd.state_delete('{file}')")).unwrap());
    assert_eq!(eng.eval::<i64>("return #gd.state_list()").unwrap(), 0);
}

#[test]
fn state_load_is_refused_for_another_match() {
    let (eng, _, _, dir) = rig("mismatch", false, 1);
    eng.frame_post();
    let file: String = eng.eval("return gd.state_save('x', 'a Fox match')").unwrap();
    eng.frame_pre();
    // damage the saved identity (exe hash at byte 16)
    let p = dir.join("data").join("geno-lab_lab").join("states").join(&file);
    let mut b = std::fs::read(&p).unwrap();
    b[16] ^= 0xFF;
    std::fs::write(&p, &b).unwrap();
    let (ok, why): (bool, String) = eng.eval(&format!("return gd.state_load('{file}')")).unwrap();
    assert!(!ok && why == "saved by another build of the game", "{why}");
    let row: mlua::Table = eng.eval("return gd.state_list()[1]").unwrap();
    assert!(!row.get::<bool>("ok").unwrap());
    assert_eq!(row.get::<String>("why").unwrap(), "saved by another build of the game");
}

#[test]
fn kb_preview_flies_the_launch_to_the_blast_zone() {
    let (eng, _, _, _) = rig("kb", false, 1);
    eng.frame_post();
    let (none, why): (Option<mlua::Value>, String) = eng.eval("return gd.kb_preview(5, {})").unwrap();
    assert!(none.is_none() && why == "no fighter on that port");
    let t: mlua::Table = eng.eval("return gd.kb_preview(1, {damage = 60, angle = 45, attacker = 2})").unwrap();
    assert_eq!(t.get::<f64>("kb").unwrap(), 240.0);
    assert_eq!(t.get::<i64>("level").unwrap(), 3);
    assert!(t.get::<bool>("tumble").unwrap());
    assert_eq!(t.get::<i64>("hitstun").unwrap(), 96);
    assert_eq!(t.get::<String>("di").unwrap(), "none");
    // P1 is left of P2, so the launch goes away from the attacker: left
    let vx = t.get::<f64>("vx").unwrap();
    assert!(vx < 0.0);
    let pts: mlua::Table = t.get("points").unwrap();
    let blast_frame = t.get::<mlua::Table>("blast").unwrap().get::<i64>("frame").unwrap();
    assert_eq!(pts.raw_len() as i64, blast_frame, "one point per hitstun frame, cut at the blast zone");
    assert!(blast_frame < 96);
    let blast: mlua::Table = t.get("blast").unwrap();
    assert_eq!(blast.get::<String>("side").unwrap(), "left");
    assert!(blast.get::<f64>("x").unwrap() < -224.0);
    // trajectory DI changes the angle; survival/in/out pick a side
    let d: mlua::Table = eng.eval("return gd.kb_preview(1, {damage = 60, angle = 45, attacker = 2, di = 'survival'})").unwrap();
    assert_ne!(d.get::<f64>("angle_di").unwrap(), t.get::<f64>("angle_di").unwrap());
    assert_eq!(d.get::<String>("di").unwrap(), "survival");
    let zones: mlua::Table = d.get("zones").unwrap();
    assert_eq!(zones.get::<f64>("top").unwrap(), 180.0);
}
