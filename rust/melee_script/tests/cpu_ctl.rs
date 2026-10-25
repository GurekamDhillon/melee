//! cpu_pad / cpu_script / cpu_macro / cpu_goto and friends over a recording mock. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct C {
    active: i32,
    ops: Vec<(i32, i32, i32, i32)>,
    bytes: Vec<u8>,
    run: Option<i32>,
}
struct G(Rc<RefCell<C>>);
impl ScriptGame for G {
    fn fighter_i(&self, _slot: i32, f: i32) -> i32 {
        (f == 0) as i32
    }
    fn cpu_attr(&self, _slot: i32, id: i32) -> i32 {
        let v: f32 = match id {
            0 => 5.0,  // jump squat
            1 => 2.1,  // hop v
            3 => 0.23, // gravity
            4 => 2.8,  // terminal
            23 => 20.0,
            _ => 0.0,
        };
        v.to_bits() as i32
    }
}

struct Rec(Rc<RefCell<C>>);
impl ScriptGame for Rec {
    fn fighter_i(&self, _slot: i32, f: i32) -> i32 {
        (f == 0) as i32
    }
    fn fighter_f(&self, _slot: i32, f: i32) -> f32 {
        if f == 5 { 1.0 } else { 0.0 }
    }
    fn cpu_attr(&self, slot: i32, id: i32) -> i32 {
        G(self.0.clone()).cpu_attr(slot, id)
    }
    fn cpu_ctl_read(&self, slot: i32, _sub: i32, op: i32, _a: i32) -> i32 {
        if slot != 0 {
            return -1;
        }
        match op {
            100 => self.0.borrow().active,
            _ => 0,
        }
    }
    fn cpu_ctl(&mut self, slot: i32, sub: i32, op: i32, a: i32) -> i32 {
        let mut c = self.0.borrow_mut();
        match op {
            2 => {
                c.bytes.clear();
                c.ops.push((slot, sub, op, a));
            }
            3 => c.bytes.push(a as u8),
            4 => c.run = Some(a),
            _ => c.ops.push((slot, sub, op, a)),
        }
        1
    }
}

#[test]
fn cpu_controller() {
    let c = Rc::new(RefCell::new(C { active: 1, ..Default::default() }));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_cpu_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(Rec(c.clone())), cfg).unwrap();
    let src = "-- @gameplay: true\n\
        ok = gd.cpu_script(1, {'press_a', {'wait', 3}, 'release_a', {'set_lstick_x', -100}})\n\
        bad1, m1 = pcall(gd.cpu_script, 1, {'nope'})\n\
        bad2, m2 = pcall(gd.cpu_script, 1, {{'wait'}})\n\
        bad3, m3 = pcall(gd.cpu_script, 1, {{'wait', 300}})\n\
        bad4, m4 = pcall(gd.cpu_script, 1, {})\n\
        pad = gd.cpu_pad(1, {x=127, buttons='A+B'}, 4)\n\
        macro_ok, n = gd.cpu_macro(1, 'short_hop')\n\
        mbytes = n\n\
        lc_ok, ln = gd.cpu_macro(1, 'lcancel_aerial', {aerial='fair'})\n\
        bad5, m5 = pcall(gd.cpu_macro, 1, 'bogus')\n\
        bad6, m6 = pcall(gd.cpu_macro, 1, 'dash', {dir='sideways'})\n\
        dx, dy = gd.cpu_dest(1)\n\
        gt = gd.cpu_goto(1, 40, 8, {radius=3})\n\
        gs, ticks = gd.cpu_goto_status(1)\n\
        done = gd.cpu_script_done(1)\n\
        cmds = gd.cpu_commands()\n\
        attrs = gd.cpu_attrs(1)\n\
        bad7, m7 = pcall(gd.cpu_goto, 2, 0, 0)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<bool>("ok").unwrap());
    assert!(env.get::<String>("m1").unwrap().contains("unknown command \"nope\""));
    assert!(env.get::<String>("m2").unwrap().contains("wait takes 1 argument(s)"));
    assert!(env.get::<String>("m3").unwrap().contains("wait argument 0..255, got 300"));
    assert!(env.get::<String>("m4").unwrap().contains("cpu_script: no steps"));
    assert!(env.get::<bool>("pad").unwrap());
    assert!(env.get::<bool>("macro_ok").unwrap());
    // short hop: press_x, wait 1, release_x, wait jump squat (5)
    assert_eq!(env.get::<i64>("mbytes").unwrap(), 6); // press_x, wait 1, release_x, wait 5
    assert!(env.get::<String>("m5").unwrap().contains("unknown macro \"bogus\""));
    assert!(env.get::<String>("m6").unwrap().contains("dir is \"forward\""));
    assert!(env.get::<mlua::Value>("dx").unwrap().is_number());
    assert!(env.get::<bool>("gt").unwrap());
    assert_eq!(env.get::<String>("gs").unwrap(), "idle");
    assert!(env.get::<bool>("done").unwrap());
    assert_eq!(env.get::<mlua::Table>("cmds").unwrap().get::<mlua::Table>("wait").unwrap().get::<i64>("code").unwrap(), 0x8E);
    assert_eq!(env.get::<mlua::Table>("attrs").unwrap().get::<f64>("jump_squat").unwrap(), 5.0);
    assert!(env.get::<String>("m7").unwrap().contains("P2 has no live CPU fighter"));
    // the last script load was the lcancel macro: bytes of the fair macro begin with press_x
    let cb = c.borrow();
    assert_eq!(cb.bytes[0], 5);
    assert!(cb.bytes.contains(&0x82));
    assert_eq!(cb.run, Some(1));
}
