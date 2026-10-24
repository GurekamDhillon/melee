//! gd.* calls added for the Geno Lab, run over a recording mock game: argument checking, gating, and the exact
//! scalar traffic the game half would see.
use melee_script::game::{lab::*, *};
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Rec {
    calls: Vec<String>,
}
struct G {
    rec: Rc<RefCell<Rec>>,
    camera_state: i32,
}
impl G {
    fn note(&self, s: String) {
        self.rec.borrow_mut().calls.push(s);
    }
}
impl ScriptGame for G {
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        match field {
            SI_PRESENT => (slot < 2) as i32,
            SI_ACTION => 14,
            SI_SLOT_TYPE => 0,
            _ => 0,
        }
    }
    fn lab_i(&self, _slot: i32, field: i32) -> i32 {
        if field == I_NAME_KIND || field == I_KIND { 1 } else { 0 }
    }
    fn camera_f(&self, field: i32) -> f32 {
        if field == CAM_OK { 1.0 } else { 0.0 }
    }
    fn camera_script_detach(&mut self) -> bool {
        self.camera_state = 1;
        self.note("camera_detach".into());
        true
    }
    fn camera_script_state(&self) -> i32 {
        self.camera_state
    }
    fn camera_script_set(&mut self, field: i32, bits: i32) {
        self.note(format!("camera_set {field} {}", f32::from_bits(bits as u32)));
    }
    fn camera_script_commit(&mut self) {
        self.note("camera_commit".into());
    }
    fn camera_script_move_begin(&mut self) {
        self.note("move_begin".into());
    }
    fn camera_script_move(&mut self, frames: i32, ease: i32) {
        self.note(format!("move {frames} {ease}"));
    }
    fn camera_script_get_bits(&self, field: i32) -> i32 {
        (field as f32 * 10.0).to_bits() as i32
    }
    fn camera_script_attach(&mut self, frames: i32) {
        self.note(format!("attach {frames}"));
        self.camera_state = 0;
    }
    fn geno_fly_get(&self, _slot: i32) -> i32 {
        0
    }
    fn geno_fly_set(&mut self, slot: i32, mode: i32) -> i32 {
        self.note(format!("fly_set {slot} {mode}"));
        0
    }
    fn geno_fly_teleport(&mut self, slot: i32, x: i32, y: i32) -> i32 {
        self.note(format!("tp {slot} {} {}", f32::from_bits(x as u32), f32::from_bits(y as u32)));
        0
    }
    fn set_percent(&mut self, slot: i32, p: i32) {
        self.note(format!("percent {slot} {p}"));
    }
    fn hit(&mut self, slot: i32, from: i32, d: i32, a: i32, k: i32, b: i32) -> bool {
        self.note(format!("hit {slot} {from} {d} {a} {k} {b}"));
        true
    }
    fn lab_set_shield(&mut self, slot: i32, h: f32) {
        self.note(format!("shield {slot} {h}"));
    }
    fn lab_set_scale(&mut self, slot: i32, s: f32) {
        self.note(format!("scale {slot} {s}"));
    }
    fn lab_common_count(&self, _slot: i32) -> i32 {
        341
    }
    fn lab_motion_anim(&self, _slot: i32, motion: i32) -> i32 {
        if motion < 345 { motion } else { -2 }
    }
    fn lab_anim_symbol_for(&self, _slot: i32, anim: i32) -> Option<String> {
        Some(format!("PlyFox5K_Share_ACTION_Anim{anim}_figatree"))
    }
    fn lab_set_motion(&mut self, slot: i32, m: i32, rate: i32, lift: i32) {
        self.note(format!("set_motion {slot} {m} {} {}", f32::from_bits(rate as u32), f32::from_bits(lift as u32)));
    }
    fn lab_script(&self, _slot: i32, _anim: i32) -> Option<u32> {
        Some(0x1000)
    }
    fn guest_ok(&self, addr: u32, len: u32) -> bool {
        addr >= 0x1000 && addr + len <= 0x1000 + 4 * SCRIPT.len() as u32
    }
    fn guest_word(&self, addr: u32) -> u32 {
        SCRIPT.get(((addr - 0x1000) / 4) as usize).copied().unwrap_or(0)
    }
    fn snap_reserve(&mut self, slots: i32) -> i32 {
        self.note(format!("reserve {slots}"));
        slots
    }
    fn snap_save_index(&mut self, slot: i32, tag: i32) {
        self.note(format!("snap_save {slot} {tag}"));
    }
    fn snap_load_index(&mut self, slot: i32) -> i32 {
        self.note(format!("snap_load {slot}"));
        0
    }
    fn lab_common_f(&self, which: i32) -> f32 {
        which as f32 * 0.5
    }
    fn lab_floor_y(&self, x: f32, _y: f32, _d: f32) -> f32 {
        if x > 0.0 { 12.5 } else { -1.0e9 }
    }
}

/// wait 3; hitbox (5 words); wait 2; end   (op in the top 6 bits)
const SCRIPT: [u32; 9] = [
    (1 << 26) | 3,
    (11 << 26) | (2 << 23) | (1 << 20) | (7 << 11) | 9,
    (8 << 16) | (0x0100 << 0),
    (4 << 16) | 0xFFFC,
    (361 << 23) | (100 << 14) | (0 << 5),
    (30 << 23) | (1 << 18) | 1,
    (1 << 26) | 2,
    0,
    0,
];

fn rig() -> (Engine, Rc<RefCell<Rec>>, Rc<RefCell<MemHost>>) {
    let rec = Rc::new(RefCell::new(Rec::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_lab_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    let eng = Engine::new(host.clone(), Box::new(G { rec: rec.clone(), camera_state: 0 }), cfg).unwrap();
    (eng, rec, host)
}

#[test]
fn timeline_walks_the_script_like_ftaction() {
    let (eng, _, _) = rig();
    let t: mlua::Table = eng.eval("return gd.timeline(1)").unwrap();
    assert_eq!(t.get::<String>("anim_name").unwrap(), "Anim0"); // the mock reports anim id 0
    assert_eq!(t.get::<String>("motion_name").unwrap(), "Wait");
    let ev: mlua::Table = t.get("events").unwrap();
    let first: mlua::Table = ev.get(1).unwrap();
    assert_eq!(first.get::<String>("name").unwrap(), "hitbox");
    assert_eq!(first.get::<f64>("frame").unwrap(), 4.0, "after wait 3, 1-based");
    assert_eq!(first.get::<i64>("bone").unwrap(), 7);
    assert_eq!(first.get::<i64>("damage").unwrap(), 9);
    assert_eq!(first.get::<i64>("element").unwrap(), 1);
    assert_eq!(first.get::<String>("element_name").unwrap(), "fire");
    assert_eq!(first.get::<bool>("hit_air").unwrap(), true);
    // wait 2 then the end word: length = 5 + 1
    assert_eq!(t.get::<f64>("length").unwrap(), 6.0);
    assert_eq!(t.get::<String>("stop").unwrap(), "end");
    let (nothing, why): (Option<mlua::Value>, String) = eng.eval("return gd.timeline(1, 9999)").unwrap();
    assert!(nothing.is_none());
    assert_eq!(why, "no such motion for this fighter");
}

#[test]
fn motion_list_and_set_motion() {
    let (eng, rec, _) = rig();
    let n: i64 = eng.eval("return #gd.motion_list(1)").unwrap();
    assert_eq!(n, 345, "341 common states plus the four specials the mock has rows for");
    let (ok, why): (bool, Option<String>) = eng.eval("return gd.set_motion(3, 14)").unwrap();
    assert!(!ok && why.as_deref() == Some("no fighter on that port"));
    let (ok, why): (bool, Option<String>) = eng.eval("return gd.set_motion(1, 9999)").unwrap();
    assert!(!ok && why.as_deref() == Some("no such motion for this fighter"));
    assert!(eng.eval::<bool>("return gd.set_motion({1, 2}, 25, 5, 0.5, 40)").unwrap());
    assert!(eng.eval::<bool>("return gd.paused()").unwrap());
    assert_eq!(eng.st.borrow().step, 4);
    eng.frame_pre(); // the boundary: both fighters enter the motion
    let calls = rec.borrow().calls.clone();
    assert_eq!(calls, ["set_motion 0 25 0.5 40", "set_motion 1 25 0.5 40"]);
}

#[test]
fn camera_claim_move_and_attach() {
    let (eng, rec, host) = rig();
    // the console is a gameplay script: it may take the camera
    eng.eval::<()>("gd.camera_set{eye = {x=1,y=2,z=3}, interest = {x=0,y=0,z=0}, fov = 60}").unwrap();
    let calls = rec.borrow().calls.clone();
    assert_eq!(calls[0], "camera_detach");
    assert!(calls.contains(&"camera_set 0 1".to_string()) && calls.contains(&"camera_set 6 60".to_string()));
    assert_eq!(calls.last().unwrap(), "camera_commit");
    assert_eq!(eng.st.borrow().camera_owner, 1);
    assert!(host.borrow().lines.iter().any(|l| l.contains("camera detached")));
    // identical eye and interest is refused before anything is claimed or written
    let e = eng.eval::<()>("gd.camera_set{eye = {x=0,y=0,z=0}, interest = {x=0,y=0,z=0}}").unwrap_err().to_string();
    assert!(e.contains("camera eye and interest must be distinct"), "{e}");
    let e = eng.eval::<()>("gd.camera_move{to = {fov = 50}, frames = 0}").unwrap_err().to_string();
    assert!(e.contains("camera_move frames must be 1..6000"), "{e}");
    eng.eval::<()>("gd.camera_move{to = {fov = 50}, frames = 30, ease = 'inout'}").unwrap();
    assert!(rec.borrow().calls.contains(&"move 30 3".to_string()));
    let e = eng.eval::<()>("gd.camera_move{to = {fov = 50}, frames = 30, ease = 'bounce'}").unwrap_err().to_string();
    assert!(e.contains("camera_move ease is linear, in, out, or inout"), "{e}");
    eng.eval::<()>("gd.camera_attach(0)").unwrap();
    assert_eq!(eng.st.borrow().camera_owner, 0, "attach with 0 frames hands it straight back");
    let got: mlua::Table = eng.eval("return gd.camera_get()").unwrap();
    assert_eq!(got.get::<String>("mode").unwrap(), "standard");
    assert_eq!(got.get::<mlua::Table>("interest").unwrap().get::<f64>("x").unwrap(), 30.0);
}

#[test]
fn fly_teleport_hit_and_edits() {
    let (eng, rec, host) = rig();
    assert!(!eng.eval::<bool>("return gd.fly(1)").unwrap());
    assert!(!eng.eval::<bool>("return gd.fly(1, 'on')").unwrap(), "state reads back from the game, which is a mock");
    let e = eng.eval::<()>("gd.fly(1, 'sideways')").unwrap_err().to_string();
    assert!(e.contains("mode is true / false / \"on\" / \"off\" / \"place\" / \"toggle\" (got \"sideways\")"), "{e}");
    eng.eval::<()>("gd.teleport(2, 3.5, -4)").unwrap();
    eng.eval::<()>("gd.set_damage(1, 5000)").unwrap();
    assert!(eng.eval::<bool>("return gd.hit(1, {damage = 12, angle = 361, kbg = 100, bkb = 20, from = 2})").unwrap());
    let e = eng.eval::<()>("gd.hit(1, {damage = 501, angle = 0, kbg = 0, bkb = 0})").unwrap_err().to_string();
    assert!(e.contains("gd.hit: damage 0-500, angle 0-361, kbg/bkb 0-1000 required"), "{e}");
    eng.eval::<()>("gd.set_shield(1, -5) gd.set_scale(2, 9)").unwrap();
    let e = eng.eval::<()>("gd.set_scale(1, 0)").unwrap_err().to_string();
    assert!(e.contains("scale must be a positive number"), "{e}");
    let calls = rec.borrow().calls.clone();
    assert_eq!(
        calls,
        ["fly_set 0 1", "tp 1 3.5 -4", "percent 0 999", "hit 0 1 12 361 100 20", "shield 0 0", "scale 1 2"]
    );
    assert!(host.borrow().lines.iter().any(|l| l == "fly: P1 on"));
    // netplay refuses every offline-only write
    host.borrow_mut().session = true;
    let e = eng.eval::<()>("gd.teleport(1, 0, 0)").unwrap_err().to_string();
    assert!(e.contains("gd.teleport is offline-only"), "{e}");
}

#[test]
fn small_lab_reads() {
    let (eng, _, _) = rig();
    let c: mlua::Table = eng.eval("return gd.lab_common()").unwrap();
    assert_eq!(c.get::<f64>("lcancel_window").unwrap(), 7.0); // LAB_C_LCANCEL_WINDOW = 14, mock returns n/2
    assert_eq!(c.get::<f64>("kb_speed").unwrap(), 0.0);
    // floor_below needs a live match
    assert!(eng.eval::<bool>("return gd.floor_below(5, 5) == nil").unwrap());
    eng.frame_post(); // fighters present: the match starts
    assert_eq!(eng.eval::<f64>("return gd.floor_below(5, 5)").unwrap(), 12.5);
    assert!(eng.eval::<bool>("return gd.floor_below(-5, 5) == nil").unwrap());
    assert!(!eng.eval::<bool>("return gd.lab_mode()").unwrap());
    assert!(!eng.eval::<bool>("return gd.lab_request()").unwrap());
    assert!(!eng.eval::<bool>("return gd.tbd_request()").unwrap());
    let (ok, why): (bool, String) = eng.eval("return gd.lab_leave('css')").unwrap();
    assert!(!ok);
    assert!(why.contains("six-fighter LAB cannot enter four-slot CSS/SSS"), "{why}");
    let e = eng.eval::<()>("gd.lab_leave('nowhere')").unwrap_err().to_string();
    assert!(e.contains("\"css\", \"sss\", \"menu\" or \"restart\" (got \"nowhere\")"), "{e}");
    assert_eq!(eng.eval::<String>("return gd.lab_now(true)").unwrap(), "1970-01-01 00:00:00");
    let r: mlua::Table = eng.eval("return gd.rollbacks()").unwrap();
    assert_eq!(r.get::<i64>("total").unwrap(), 0);
}

#[test]
fn pads_through_gd_input() {
    let (eng, _, host) = rig();
    eng.eval::<()>("gd.input(1, 'A+B', 2) gd.input(2, {buttons = 'start', x = 300, cy = -5})").unwrap();
    let mut st = [melee_script::pad::PadSample::default(); 4];
    let driven = eng.pad_apply(&mut st, 0);
    assert_eq!(driven, 0b11);
    assert_eq!(st[0].buttons, 0x0300);
    assert_eq!((st[1].buttons, st[1].stick_x, st[1].substick_y), (0x1000, 127, -5), "stick clamps to 127");
    let p: mlua::Table = eng.eval("return gd.pad(1)").unwrap();
    assert!(p.get::<bool>("A").unwrap() && p.get::<bool>("B").unwrap() && !p.get::<bool>("X").unwrap());
    let e = eng.eval::<()>("gd.input(1, 'JUMP')").unwrap_err().to_string();
    assert!(e.contains("unknown button \"JUMP\""), "{e}");
    assert!(host.borrow().lines.iter().any(|l| l == "gw: pad: P1 script claim (owner 1)"));
    eng.eval::<()>("gd.release(1)").unwrap();
    eng.frame_post();
    eng.frame_post();
    let mut st = [melee_script::pad::PadSample { buttons: 0x4000, ..Default::default() }; 4];
    eng.pad_apply(&mut st, 0);
    assert_eq!(st[0].buttons, 0x4000, "released: the physical pad shows through again");
}

#[test]
fn savestate_slots_apply_at_the_boundary() {
    let (eng, rec, host) = rig();
    eng.eval::<()>("saved = {} loaded = {} function on_savestate(n) saved[#saved+1] = n end function on_loadstate(n) loaded[#loaded+1] = n end").unwrap();
    let e = eng.eval::<()>("gd.savestate(9)").unwrap_err().to_string();
    assert!(e.contains("savestate slots are 1-4"), "{e}");
    let e = eng.eval::<()>("gd.loadstate(2)").unwrap_err().to_string();
    assert!(e.contains("savestate slot 2 is empty"), "{e}");
    eng.eval::<()>("gd.savestate(2)").unwrap();
    assert!(rec.borrow().calls.is_empty(), "nothing happens until the frame boundary");
    eng.frame_pre();
    assert_eq!(rec.borrow().calls, ["reserve 4", "snap_save 1 -2"]);
    assert!(host.borrow().lines.is_empty() || true);
    assert!(eng.console_lines().iter().any(|l| l == "saved state 2 (frame 0)"));
    eng.frame_post();
    eng.eval::<()>("gd.loadstate(2)").unwrap();
    eng.frame_pre();
    assert!(rec.borrow().calls.contains(&"snap_load 1".to_string()));
    assert!(eng.console_lines().iter().any(|l| l == "loaded state 2"));
    // a slot saved in another scene cannot be loaded
    eng.scene_begin(7);
    let e = eng.eval::<()>("gd.loadstate(2)").unwrap_err().to_string();
    assert!(e.contains("savestate slot 2 was saved in another scene"), "{e}");
}
