//! echo_add / echo_remove / echoes / fighter_history over a mock echo store. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct E {
    rows: [Vec<[i32; 9]>; 12],
    owner: [i32; 12],
    next: i32,
}
struct G(Rc<RefCell<E>>);
impl ScriptGame for G {
    fn echo_owner(&self, e: i32) -> i32 {
        self.0.borrow().owner[e as usize]
    }
    fn echo_read(&self, e: i32, i: i32, f: i32) -> i32 {
        self.0.borrow().rows[e as usize].get(i as usize).map_or(0, |r| r[f as usize])
    }
    fn echo_add_with_handle(&mut self, e: i32, owner: i32, rest: &[i32], handle: i32) -> i32 {
        let mut b = self.0.borrow_mut();
        if b.rows[e as usize].len() >= 8 {
            return 0;
        }
        let h = if handle != 0 { handle } else { b.next + 1 };
        b.next = b.next.max(h);
        let mut r = [0; 9];
        r[0] = h;
        r[1..].copy_from_slice(rest);
        b.rows[e as usize].push(r);
        b.owner[e as usize] = owner;
        h
    }
    fn echo_remove(&mut self, h: i32, _owner: i32) -> i32 {
        let mut b = self.0.borrow_mut();
        for e in 0..12 {
            if let Some(i) = b.rows[e].iter().position(|r| r[0] == h) {
                b.rows[e].remove(i);
                return 1;
            }
        }
        0
    }
    fn fighter_history_depth(&self) -> i32 {
        61
    }
    fn fighter_history_read(&self, entity: i32, age: i32, hitbox: i32, field: i32) -> i32 {
        if entity != 0 {
            return 0;
        }
        match (hitbox, field) {
            (-1, 0) => (age < 10) as i32,
            (-1, 2) => 42,
            (-1, 10) => 1,
            (0, 0) => 1,
            (0, 2) => 6.0f32.to_bits() as i32,
            (0, 18) => 4,
            _ => 0,
        }
    }
}

#[test]
fn echoes_and_history() {
    let store = Rc::new(RefCell::new(E::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_echo_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(store.clone())), cfg).unwrap();
    let src = "-- @gameplay: true\n\
        h = gd.echo_add(1, {delay=4, match={move='fair', element='fire'}, damage=0.5, once_per_move=false})\n\
        h2 = gd.echo_add(1, {sub=true, match={airborne=true}})\n\
        list = gd.echoes(1)\n\
        bad1, m1 = pcall(gd.echo_add, 1, {handle=3})\n\
        bad2, m2 = pcall(gd.echo_add, 1, {delay=61})\n\
        bad3, m3 = pcall(gd.echo_add, 1, {match={move='teleport'}})\n\
        bad4, m4 = pcall(gd.echo_add, 1, {damage=9})\n\
        removed = gd.echo_remove(h)\n\
        again = gd.echo_remove(h)\n\
        depth = gd.fighter_history_depth()\n\
        hist = gd.fighter_history(1, 3)\n\
        none = gd.fighter_history(1, 20)\n\
        badage = pcall(gd.fighter_history, 1, 61)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert_eq!(env.get::<i64>("h").unwrap(), 1);
    assert_eq!(env.get::<i64>("h2").unwrap(), 2);
    let list: mlua::Table = env.get("list").unwrap();
    assert!(list.get::<bool>("journal").unwrap());
    assert_eq!(list.raw_len(), 1);
    let r: mlua::Table = list.get(1).unwrap();
    assert_eq!(r.get::<i64>("delay").unwrap(), 4);
    assert_eq!(r.get::<f64>("damage").unwrap(), 0.5);
    assert_eq!(r.get::<String>("element").ok(), None);
    let m: mlua::Table = r.get("match").unwrap();
    assert_eq!(m.get::<String>("move").unwrap(), "fair");
    assert_eq!(m.get::<String>("element").unwrap(), "fire");
    assert!(env.get::<String>("m1").unwrap().contains("does not accept a preassigned handle"));
    assert!(env.get::<String>("m2").unwrap().contains("hit rule delay out of range"));
    assert!(env.get::<String>("m3").unwrap().contains("unknown echo move tag"));
    assert!(env.get::<String>("m4").unwrap().contains("echo scale must be finite and 0.1..4"));
    assert!(env.get::<bool>("removed").unwrap());
    assert!(!env.get::<bool>("again").unwrap());
    assert_eq!(env.get::<i64>("depth").unwrap(), 61);
    let h: mlua::Table = env.get("hist").unwrap();
    assert_eq!(h.get::<i64>("action_state").unwrap(), 42);
    let hb: mlua::Table = h.get("hitboxes").unwrap();
    assert_eq!(hb.raw_len(), 1);
    assert_eq!(hb.get::<mlua::Table>(1).unwrap().get::<f64>("damage").unwrap(), 6.0);
    assert_eq!(hb.get::<mlua::Table>(1).unwrap().get::<i64>("owner_port").unwrap(), 3);
    assert!(env.get::<mlua::Value>("none").unwrap().is_nil());
    assert!(!env.get::<bool>("badage").unwrap());
}
