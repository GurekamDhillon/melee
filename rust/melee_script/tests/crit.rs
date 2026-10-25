//! gd.crit / crit_force / crit_seed / cpu_assist / cpu_technical over a mock game. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
struct S {
    owner: i32,
    mask: i32,
    min_pct: i32,
    force: i32,
    slots: HashMap<i32, [i32; 4]>,
    seed: i32,
    assist: Option<[i32; 9]>,
}
struct G(Rc<RefCell<S>>);
impl ScriptGame for G {
    fn crit_read(&self, _e: i32, field: i32) -> i32 {
        let s = self.0.borrow();
        match field {
            0 => s.owner,
            1 => s.mask,
            2 => s.min_pct,
            3 => s.force,
            10..=49 => {
                let slot = (field - 10) / 4;
                s.slots.get(&slot).map_or(0, |v| v[((field - 10) % 4) as usize])
            }
            _ => 0,
        }
    }
    fn crit_set(&mut self, _e: i32, owner: i32, slot: i32, ch: i32, m: i32, mm: i32, l: i32, minp: i32, force: i32, flags: i32) -> i32 {
        let mut s = self.0.borrow_mut();
        if flags & 2 != 0 {
            *s = S { seed: s.seed, ..Default::default() };
            return 1;
        }
        s.owner = owner;
        if flags & 1 != 0 {
            s.min_pct = minp;
        }
        if flags & 8 != 0 {
            s.force = force;
        }
        if flags & 4 != 0 {
            s.slots.insert(slot, [ch, m, mm, l]);
            s.mask |= 1 << slot;
        }
        1
    }
    fn crit_seed(&mut self, seed: i32) {
        self.0.borrow_mut().seed = seed;
    }
    fn cpu_assist(&mut self, _s: i32, _o: i32, lc: i32, ps: i32, tech: i32, tdir: i32, wd: i32, ff: i32, seed: i32) -> i32 {
        self.0.borrow_mut().assist = Some([lc, ps, tech, tdir, wd, ff, seed, 0, 0]);
        1
    }
    fn cpu_assist_state(&self, _s: i32, _o: i32, f: i32) -> i32 {
        match (f, self.0.borrow().assist) {
            (0, Some(_)) => 1,
            (1, Some(a)) => a[0],
            (3, Some(a)) => a[2],
            (6, Some(a)) => a[3],
            (7, Some(a)) => a[6],
            _ => 0,
        }
    }
}

#[test]
fn crit_and_cpu_assist() {
    let st = Rc::new(RefCell::new(S::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_crit_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(st.clone())), cfg).unwrap();
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        ok = gd.crit(1, {chance=0.25, multiplier=2, min_percent=30, force=2, tags={aerial={chance=1, multiplier=3, multiplier_max=4}}})\n\
        c = gd.crit(1)\n\
        b1, m1 = pcall(gd.crit, 1, {chance=2})\n\
        b2, m2 = pcall(gd.crit, 1, {bogus=1})\n\
        b3, m3 = pcall(gd.crit, 1, {tags={nope={}}})\n\
        b4, m4 = pcall(gd.crit, 1, {multiplier=3, multiplier_max=2})\n\
        f = gd.crit_force(1, 7)\n\
        gd.crit_seed(99)\n\
        b5, m5 = pcall(gd.crit_seed, 0)\n\
        a_ok = gd.cpu_assist(2, {lcancel=0.5, tech=1, tech_dir='away', seed=77})\n\
        a = gd.cpu_assist(2)\n\
        b6, m6 = pcall(gd.cpu_assist, 2, {tech_dir='sideways'})\n\
        b7, m7 = pcall(gd.cpu_assist, 2, {seed=1.5})\n\
        b8, m8 = pcall(gd.cpu_assist, 2, {lcancel='x'})\n\
        cleared = gd.crit(1, nil)\n\
        c2 = gd.crit(1)";
    let i = eng.load_text("c", "c.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<bool>("ok").unwrap());
    let c: mlua::Table = env.get("c").unwrap();
    assert_eq!(c.get::<f64>("chance").unwrap(), 0.25);
    assert_eq!(c.get::<f64>("multiplier_max").unwrap(), 2.0);
    assert_eq!(c.get::<f64>("min_percent").unwrap(), 30.0);
    assert_eq!(c.get::<i64>("force").unwrap(), 2);
    let aerial: mlua::Table = c.get::<mlua::Table>("tags").unwrap().get("aerial").unwrap();
    assert_eq!(aerial.get::<f64>("multiplier_max").unwrap(), 4.0);
    assert!(env.get::<String>("m1").unwrap().contains("finite and in range"));
    assert!(env.get::<String>("m2").unwrap().contains("unknown item option/payload key"));
    assert!(env.get::<String>("m3").unwrap().contains("unknown crit move tag"));
    assert!(env.get::<String>("m4").unwrap().contains("multiplier_max is below multiplier"));
    assert!(env.get::<bool>("f").unwrap());
    assert!(env.get::<String>("m5").unwrap().contains("crit seed must be 1-2147483647"));
    assert_eq!(st.borrow().seed, 99);
    assert!(env.get::<bool>("a_ok").unwrap());
    let a: mlua::Table = env.get("a").unwrap();
    assert!(a.get::<bool>("enabled").unwrap());
    assert_eq!(a.get::<String>("tech_dir").unwrap(), "away");
    assert_eq!(a.get::<i64>("seed").unwrap(), 77);
    assert_eq!(a.get::<mlua::Table>("probability").unwrap().get::<f64>("lcancel").unwrap(), 0.5);
    assert!(env.get::<String>("m6").unwrap().contains("tech_dir is"));
    assert!(env.get::<String>("m7").unwrap().contains("seed must be an integer"));
    assert!(env.get::<String>("m8").unwrap().contains("lcancel must be a number 0..1"));
    assert!(env.get::<bool>("cleared").unwrap());
    assert!(env.get::<mlua::Value>("c2").unwrap().is_nil());
}
