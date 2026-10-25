//! fighter_bench / fighter_call / fighter_benched over a mock game. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct B {
    benched: bool,
    called: Vec<(i32, f32, f32, f32, i32)>,
}
struct G(Rc<RefCell<B>>);
impl ScriptGame for G {
    fn fighter_bench_check(&self, _s: i32, _o: i32, calling: i32) -> i32 {
        let b = self.0.borrow();
        if calling == 1 && !b.benched { 9 } else { 0 }
    }
    fn fighter_bench(&mut self, _s: i32, _o: i32) -> i32 {
        self.0.borrow_mut().benched = true;
        0
    }
    fn fighter_call(&mut self, s: i32, _o: i32, x: i32, y: i32, f: i32, fr: i32) -> i32 {
        let fl = |i: i32| f32::from_bits(i as u32);
        self.0.borrow_mut().called.push((s, fl(x), fl(y), fl(f), fr));
        0
    }
    fn fighter_benched(&self, _s: i32) -> i32 {
        self.0.borrow().benched as i32
    }
    fn fighter_reserve_read(&self, _s: i32, e: i32, field: i32) -> i32 {
        match (e, field) {
            (0, 0) => 1,
            (0, 4) => 3.5f32.to_bits() as i32,
            (0, 2) => 14,
            _ => 0,
        }
    }
}

#[test]
fn bench_call_and_read() {
    let st = Rc::new(RefCell::new(B::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_bench_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(st.clone())), cfg).unwrap();
    let src = "-- @gameplay: true\n\
        n0, w0 = gd.fighter_bench(1)\n\
        eng_ok, eng_why = gd.fighter_call(1, 0, 0)\n";
    let go = |id: &str, src: &str| {
        let i = eng.load_text(id, &format!("{id}.lua"), src.as_bytes(), None, "test").unwrap();
        let env = eng.st.borrow().scripts[i].env.clone().unwrap();
        env
    };
    let env = go("a", src);
    assert!(!env.get::<bool>("n0").unwrap());
    assert!(env.get::<String>("w0").unwrap().contains("missing fighter"));

    eng.st.borrow_mut().match_active = true;
    let env = go(
        "b",
        "-- @gameplay: true\n\
         c0, why0 = gd.fighter_call(1, 5, 6)\n\
         b1 = gd.fighter_bench(1)\n\
         b, list = gd.fighter_benched(1)\n\
         c1 = gd.fighter_call(1, 5, 6, {facing=-1, intangible_frames=30})\n\
         bad, m = pcall(gd.fighter_call, 1, 0, 0, {facing=0.5})\n\
         bad2, m2 = pcall(gd.fighter_call, 1, 99999, 0)",
    );
    assert!(!env.get::<bool>("c0").unwrap());
    assert!(env.get::<String>("why0").unwrap().contains("not benched"));
    assert!(env.get::<bool>("b1").unwrap());
    assert!(env.get::<bool>("b").unwrap());
    let list: mlua::Table = env.get("list").unwrap();
    let e0: mlua::Table = list.get(1).unwrap();
    assert!(e0.get::<bool>("present").unwrap());
    assert_eq!(e0.get::<f64>("x").unwrap(), 3.5);
    assert_eq!(e0.get::<i64>("action").unwrap(), 14);
    assert!(!list.get::<mlua::Table>(2).unwrap().get::<bool>("present").unwrap());
    assert!(env.get::<bool>("c1").unwrap());
    assert_eq!(st.borrow().called, vec![(0, 5.0, 6.0, -1.0, 30)]);
    assert!(env.get::<String>("m").unwrap().contains("facing must be -1 or 1"));
    assert!(env.get::<String>("m2").unwrap().contains("within +/-100000") || !env.get::<bool>("bad2").unwrap());
}
