//! Part overrides, effects lab and lab_peek over mocks. Headless; no input sent.
use melee_script::{Config, Engine, FxQuery, Host, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct P {
    ctl: Vec<(i32, i32, i32, i32, i32)>,
    fx: Vec<String>,
}
struct G(Rc<RefCell<P>>);
impl ScriptGame for G {
    fn fighter_i(&self, slot: i32, f: i32) -> i32 {
        (slot == 0 && f == 0) as i32
    }
    fn parts_i(&self, _s: i32, i: i32, f: i32) -> i32 {
        match f {
            0 | 1 => 2,
            2 => 10 + i,
            _ => 0,
        }
    }
    fn parts_f(&self, _s: i32, i: i32, f: i32) -> f32 {
        if f == 0 { 1.5 + i as f32 } else { 0.0 }
    }
    fn parts_control(&mut self, s: i32, part: i32, ctl: i32, v: i32, o: i32) -> i32 {
        if part >= 5 {
            return -1;
        }
        self.0.borrow_mut().ctl.push((s, part, ctl, v, o));
        1
    }
    fn parts_joint(&self, _s: i32, j: i32) -> i32 {
        0x8000_0000u32 as i32 + j
    }
    fn mem1_read(&self, addr: u32, n: usize) -> Option<Vec<u8>> {
        (addr >= 0x8000_0000 && addr < 0x8000_1000).then(|| (0..n).map(|i| (addr as usize + i) as u8).collect())
    }
}
struct H(Rc<RefCell<P>>, MemHost);
impl Host for H {
    fn log(&mut self, m: &str) {
        self.1.log(m)
    }
    fn now_ms(&self) -> f64 {
        0.0
    }
    fn fx_lab_play(&mut self, p: &str, a: i32, o: i32, s1: i32, j: i32, _f: i32, face: i32, off: [f32; 3], sc: f32, seed: u32) -> i32 {
        self.0.borrow_mut().fx.push(format!("play {p} {a:x} o{o} s{s1} j{j} f{face} {off:?} {sc} {seed}"));
        7
    }
    fn fx_lab_query(&mut self, h: i32, _o: i32) -> (bool, FxQuery) {
        (h == 7, FxQuery { age: 3, particles: 12, ..Default::default() })
    }
    fn fx_lab_end(&mut self, h: i32, _o: i32, fade: i32) -> i32 {
        self.0.borrow_mut().fx.push(format!("end {h} {fade}"));
        1
    }
    fn fx_shake(&mut self, a: f32, f: i32, s: u32) {
        self.0.borrow_mut().fx.push(format!("shake {a} {f} {s}"));
    }
}

#[test]
fn parts_fx_peek() {
    let p = Rc::new(RefCell::new(P::default()));
    let host = Rc::new(RefCell::new(H(p.clone(), MemHost::default())));
    let dir = std::env::temp_dir().join(format!("melee_script_present_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(p.clone())), cfg).unwrap();
    let src = "-- @gameplay: true\n\
        pt = gd.parts(1)\n\
        none = gd.parts(2)\n\
        s1 = gd.dobj_solid(1, 2, 0xFF0000FF)\n\
        t1 = gd.dobj_tint(1, 3, {r=0,g=255,b=0})\n\
        bad, m = pcall(gd.dobj_solid, 1, 9)\n\
        off = gd.dobj_solid_off(1, 2)\n\
        id1 = gd.parts_id(1, true)\n\
        h = gd.fx_play('spark', 1, 4, 0, 9, 0, 2, 5)\n\
        inst = gd.fx_instance(h)\n\
        ended = gd.fx_end(h, 30)\n\
        gd.fx_shake(0.5, 10)\n\
        badfx, mf = pcall(gd.fx_play, 'x', 1, 0, 0, 0, 0, 500)\n\
        peek = gd.lab_peek(0x80000010, 4)\n\
        nopeek = gd.lab_peek(0x70000000)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    let pt: mlua::Table = env.get("pt").unwrap();
    assert_eq!(pt.raw_len(), 2);
    assert_eq!(pt.get::<mlua::Table>(2).unwrap().get::<f64>("area").unwrap(), 2.5);
    assert_eq!(pt.get::<mlua::Table>(1).unwrap().get::<i64>("joint").unwrap(), 10);
    assert_eq!(pt.get::<String>("coverage_mode").unwrap(), "opaque_geometry_paired_ids");
    assert!(env.get::<mlua::Value>("none").unwrap().is_nil());
    assert!(env.get::<bool>("s1").unwrap() && env.get::<bool>("t1").unwrap() && env.get::<bool>("off").unwrap());
    assert!(env.get::<String>("m").unwrap().contains("invalid part or override capacity exhausted (-1)"));
    assert!(env.get::<bool>("id1").unwrap());
    assert_eq!(env.get::<i64>("h").unwrap(), 7);
    let inst: mlua::Table = env.get("inst").unwrap();
    assert!(inst.get::<bool>("alive").unwrap());
    assert_eq!(inst.get::<i64>("particles").unwrap(), 12);
    assert!(env.get::<bool>("ended").unwrap());
    assert!(env.get::<String>("mf").unwrap().contains("fx_play requires finite offsets, scale in (0,100]"));
    assert_eq!(env.get::<String>("peek").unwrap(), "10111213");
    assert!(env.get::<mlua::Value>("nopeek").unwrap().is_nil());
    let b = p.borrow();
    assert_eq!(b.ctl[0], (0, 2, 1, 0xFF0000, 2)); // colour is rgb (rgba >> 8), owner = script slot 1 + 1
    assert_eq!(b.fx[0], "play spark 80000004 o2 s1 j4 f1 [0.0, 9.0, 0.0] 2 5");
    assert_eq!(b.fx[1], "end 7 30");
    assert_eq!(b.fx[2], "shake 0.5 10 1");
}
