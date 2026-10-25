//! hit_rule_add / remove / clear / fighter_status / hit_rules over a mock rule store. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Store {
    owner: [i32; 12],
    status: [i32; 12],
    rules: [Vec<[i32; 16]>; 12],
    next_handle: i32,
}
struct G(Rc<RefCell<Store>>);
impl ScriptGame for G {
    fn hit_rules_owner(&self, e: i32) -> i32 {
        self.0.borrow().owner[e as usize]
    }
    fn hit_rules_count(&self, e: i32) -> i32 {
        self.0.borrow().rules[e as usize].len() as i32
    }
    fn fighter_status(&self, e: i32) -> i32 {
        self.0.borrow().status[e as usize]
    }
    fn hit_rule_read(&self, e: i32, i: i32, f: i32) -> i32 {
        self.0.borrow().rules[e as usize].get(i as usize).map_or(0, |r| r[f as usize])
    }
    fn hit_rules_replace(&mut self, e: i32, owner: i32, count: i32, status: i32, rules: &[i32]) -> i32 {
        let mut s = self.0.borrow_mut();
        let mut v: Vec<[i32; 16]> = (0..count as usize).map(|i| rules[i * 16..i * 16 + 16].try_into().unwrap()).collect();
        for r in v.iter_mut() {
            if r[12] == 0 {
                s.next_handle += 1;
                r[12] = s.next_handle;
            }
        }
        s.owner[e as usize] = if count > 0 || status != 0 { owner } else { 0 };
        s.status[e as usize] = status;
        s.rules[e as usize] = v;
        1
    }
}

fn engine(st: Rc<RefCell<Store>>) -> Engine {
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_hr_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    Engine::new(host, Box::new(G(st)), cfg).unwrap()
}

#[test]
fn rules_roundtrip() {
    let st = Rc::new(RefCell::new(Store::default()));
    let eng = engine(st.clone());
    let src = "-- @gameplay: true\n\
        h = gd.hit_rule_add(2, {id=7, match={move='jab', element='fire'}, change={damage=2, shield_damage=5}})\n\
        l = gd.hit_rules(2)\n\
        sub = gd.hit_rules(2, true)\n\
        bad1, m1 = pcall(gd.hit_rule_add, 1, {change={bogus=1}})\n\
        bad2, m2 = pcall(gd.hit_rule_add, 1, {change={damage=9}})\n\
        bad3, m3 = pcall(gd.hit_rule_add, 1, {change={knockback_taken=2}})\n\
        bad4, m4 = pcall(gd.hit_rule_add, 1, {match={element='water'}, change={}})\n\
        s = gd.fighter_status(2, 5)\n\
        removed = gd.hit_rule_remove(h)\n\
        again = gd.hit_rule_remove(h)\n\
        after = #gd.hit_rules(2)\n\
        gd.hit_rule_add(3, {change={percent_damage=2}}, false)\n\
        c = gd.hit_rules_clear(3)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert_eq!(env.get::<i64>("h").unwrap(), 1);
    let l: mlua::Table = env.get("l").unwrap();
    assert!(l.get::<bool>("percent_only").unwrap());
    let r: mlua::Table = l.get(1).unwrap();
    assert_eq!(r.get::<i64>("id").unwrap(), 7);
    let m: mlua::Table = r.get("match").unwrap();
    assert_eq!(m.get::<String>("move").unwrap(), "jab");
    assert_eq!(m.get::<String>("element").unwrap(), "fire");
    let c: mlua::Table = r.get("change").unwrap();
    assert_eq!(c.get::<f64>("damage").unwrap(), 2.0);
    assert_eq!(c.get::<i64>("shield_damage").unwrap(), 5);
    // both entities got the rule (no selector)
    let sub: mlua::Table = env.get("sub").unwrap();
    assert!(sub.get::<bool>("sub").unwrap());
    assert_eq!(sub.get::<mlua::Table>(1).unwrap().get::<i64>("handle").unwrap(), 1);
    assert!(env.get::<String>("m1").unwrap().contains("unknown field 'bogus'"));
    assert!(env.get::<String>("m2").unwrap().contains("hit rule multiplier damage must be 0.100000..4.000000"));
    assert!(env.get::<String>("m3").unwrap().contains("knockback_taken requires status_bits or incoming"));
    assert!(env.get::<String>("m4").unwrap().contains("only normal/fire/electric/ice/darkness"));
    assert_eq!(env.get::<i64>("s").unwrap(), 5);
    assert!(env.get::<bool>("removed").unwrap());
    assert!(!env.get::<bool>("again").unwrap());
    assert_eq!(env.get::<i64>("after").unwrap(), 0);
    assert_eq!(st.borrow().status[2], 5);
    assert!(st.borrow().rules[4].is_empty());
    assert!(env.get::<bool>("c").unwrap());
}
