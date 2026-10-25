//! fighter_mod and hitstop over mocks. Headless; no input sent.
use melee_script::{Config, Engine, Host, MemHost, ScriptGame};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct H(Rc<Cell<f64>>, MemHost);
impl Host for H {
    fn log(&mut self, m: &str) {
        self.1.log(m)
    }
    fn now_ms(&self) -> f64 {
        self.0.get()
    }
}
#[derive(Default)]
struct M {
    owner: i32,
    vals: [i32; 11],
    enabled: bool,
}
struct G(Rc<RefCell<M>>);
impl ScriptGame for G {
    fn fighter_mod_owner(&self, _s: i32) -> i32 {
        self.0.borrow().owner
    }
    fn fighter_mod_read(&self, _s: i32, f: i32) -> i32 {
        self.0.borrow().vals[f as usize]
    }
    fn fighter_mod(&mut self, _s: i32, owner: i32, enabled: bool, bits: &[i32]) -> i32 {
        let mut m = self.0.borrow_mut();
        m.owner = if enabled { owner } else { 0 };
        m.enabled = enabled;
        m.vals[..8].copy_from_slice(bits);
        1
    }
    fn fighter_mod_extra(&mut self, _s: i32, _o: i32, a: i32, b: i32, c: i32) -> i32 {
        let mut m = self.0.borrow_mut();
        m.vals[8] = a;
        m.vals[9] = b;
        m.vals[10] = c;
        1
    }
}

#[test]
fn modifiers_and_hitstop() {
    let clock = Rc::new(Cell::new(1000.0));
    let m = Rc::new(RefCell::new(M::default()));
    let host = Rc::new(RefCell::new(H(clock.clone(), MemHost::default())));
    let dir = std::env::temp_dir().join(format!("melee_script_fmod_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(m.clone())), cfg).unwrap();
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        n0 = gd.fighter_mod(1)\n\
        ok = gd.fighter_mod(1, {run_speed=2, weight=9, damage_taken=0.01})\n\
        r = gd.fighter_mod(1)\n\
        bad, m1 = pcall(gd.fighter_mod, 1, {nope=1})\n\
        bad2, m2 = pcall(gd.fighter_mod, 1, {run_speed=1/0})\n\
        hs = gd.hitstop(30)\n\
        badhs, mh = pcall(gd.hitstop, 0)\n\
        cleared = gd.fighter_mod(1, nil)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<mlua::Value>("n0").unwrap().is_nil());
    assert!(env.get::<bool>("ok").unwrap());
    let r: mlua::Table = env.get("r").unwrap();
    assert_eq!(r.get::<f64>("run_speed").unwrap(), 2.0);
    assert_eq!(r.get::<f64>("weight").unwrap(), 4.0); // clamped to 4
    assert!((r.get::<f64>("damage_taken").unwrap() - 0.1).abs() < 1e-6); // clamped to 0.1
    assert!(env.get::<String>("m1").unwrap().contains("unknown fighter modifier"));
    assert!(env.get::<String>("m2").unwrap().contains("must be finite"));
    assert!(env.get::<bool>("cleared").unwrap());
    assert!(!m.borrow().enabled);
    // hitstop: 30 frames = 500 ms of wall clock
    assert!(env.get::<bool>("hs").unwrap());
    assert!(env.get::<String>("mh").unwrap().contains("frames must be 1..36000"));
    assert!(eng.st.borrow().hitstop_live());
    clock.set(1400.0);
    assert!(eng.st.borrow().hitstop_live());
    clock.set(1501.0);
    assert!(!eng.st.borrow().hitstop_live());
    eng.tick();
    assert_eq!(eng.st.borrow().hitstop.active, 0);
}
