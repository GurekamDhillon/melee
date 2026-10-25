//! Fighter capability API (gw_script_fighter_caps*.inc) over a mock game. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Caps {
    mv: Option<(i32, i32, i32)>,
    effect: Option<(i32, i32, i32, i32)>,
    armour: Option<(i32, i32)>,
    armor: Vec<(i32, i32, i32, i32, i32, i32, i32)>,
    intr: Option<(i32, i32, i32)>,
    released: Vec<i32>,
    given: Vec<(i32, i32)>,
}
struct G(Rc<RefCell<Caps>>);
impl ScriptGame for G {
    fn caps_movement(&mut self, e: i32, _o: i32, jumps: i32, flags: i32) -> i32 {
        self.0.borrow_mut().mv = Some((e, jumps, flags));
        1
    }
    fn caps_movement_read(&self, _e: i32, field: i32) -> i32 {
        let c = self.0.borrow();
        match (field, c.mv) {
            (0, Some(_)) => 1,
            (1, Some(m)) => m.1,
            (2, Some(m)) => m.2,
            (3, _) => 2,
            _ => 0,
        }
    }
    fn caps_effect(&mut self, e: i32, _o: i32, kind: i32, v: i32, fr: i32) -> i32 {
        self.0.borrow_mut().effect = Some((e, kind, v, fr));
        1
    }
    fn caps_effect_read(&self, _e: i32, _k: i32, field: i32) -> i32 {
        match (field, self.0.borrow().effect) {
            (0, Some(x)) => x.3,
            (1, Some(x)) => x.2,
            _ => 0,
        }
    }
    fn caps_armour(&mut self, _e: i32, _o: i32, d: i32, k: i32) -> i32 {
        self.0.borrow_mut().armour = Some((d, k));
        1
    }
    fn caps_armour_read(&self, _e: i32, field: i32) -> i32 {
        match (field, self.0.borrow().armour) {
            (0, Some(_)) => 1,
            (1, Some(a)) => a.0,
            (2, Some(a)) => a.1,
            _ => 0,
        }
    }
    fn caps_give_item(&mut self, e: i32, _o: i32, k: i32) -> i32 {
        self.0.borrow_mut().given.push((e, k));
        if k == 99 { -2 } else { 1 }
    }
    fn caps_give_item_reason(&self, _r: i32) -> String {
        "no such item".into()
    }
    fn caps_opponent(&self, e: i32, _r: i32, nth: i32, exclude: i32) -> i32 {
        let all = [2, 3, 4].into_iter().filter(|x| *x != e && *x != exclude).collect::<Vec<_>>();
        if nth == 0 { all.first().copied().unwrap_or(0) } else { all.get(nth as usize - 1).copied().unwrap_or(0) }
    }
    fn intr_win_set(&mut self, _e: i32, _o: i32, fr: i32, exits: i32, flags: i32) -> i32 {
        self.0.borrow_mut().intr = Some((fr, exits, flags));
        1
    }
    fn intr_win_read(&self, _e: i32, field: i32) -> i32 {
        match (field, self.0.borrow().intr) {
            (0, Some(i)) => i.0,
            (2, Some(i)) => i.1,
            _ => 0,
        }
    }
    fn armor_set(&mut self, e: i32, _o: i32, ty: i32, v: i32, fr: i32, st: i32, _f: i32, _t: i32, dir: i32) -> i32 {
        self.0.borrow_mut().armor.push((e, ty, v, fr, st, dir, 0));
        1
    }
    fn armor_read(&self, _e: i32, ty: i32, field: i32) -> i32 {
        let c = self.0.borrow();
        let Some(a) = c.armor.iter().find(|a| a.1 == ty) else { return 0 };
        match field {
            0 | 8 => 1,
            1 => a.2,
            3 => a.3,
            4 => a.4,
            5 | 6 => (-1.0f32).to_bits() as i32,
            7 => a.5,
            _ => 0,
        }
    }
    fn armor_clear(&mut self, _e: i32, _o: i32, ty: i32) -> i32 {
        self.0.borrow_mut().armor.retain(|a| ty != 0 && a.1 != ty);
        1
    }
    fn caps_release(&mut self, o: i32) {
        self.0.borrow_mut().released.push(o);
    }
}

fn rig() -> (Engine, Rc<RefCell<Caps>>) {
    let st = Rc::new(RefCell::new(Caps::default()));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_caps_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(st.clone())), cfg).unwrap();
    eng.st.borrow_mut().match_active = true;
    (eng, st)
}
fn run(eng: &Engine, id: &str, src: &str) -> mlua::Table {
    let full = format!("-- @gameplay: true\n{src}");
    let i = eng.load_text(id, &format!("{id}.lua"), full.as_bytes(), None, "test").expect("load");
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    env
}

#[test]
fn caps_effect_and_status_roundtrip() {
    let (eng, st) = rig();
    let env = run(
        &eng,
        "c",
        "c0, r0 = gd.fighter_caps(1)\n\
         ok = gd.fighter_caps(1, {air_jumps=3, shield=true, grab=true})\n\
         c1, r1 = gd.fighter_caps(1)\n\
         e0 = gd.fighter_effect(2, 'size')\n\
         eok = gd.fighter_effect(2, 'size', 2.0, 120)\n\
         e1 = gd.fighter_effect(2, 'size')\n\
         bad1, m1 = pcall(gd.fighter_effect, 2, 'size', 9, 1)\n\
         bad2, m2 = pcall(gd.fighter_effect, 2, 'bogus')\n\
         bad3, m3 = pcall(gd.fighter_caps, 13)\n\
         bad4, m4 = pcall(gd.fighter_caps, 1, {nope=true})\n\
         bad5, m5 = pcall(gd.fighter_caps, 1, {air_jumps=9})",
    );
    assert!(env.get::<mlua::Value>("c0").unwrap().is_nil());
    assert_eq!(env.get::<i64>("r0").unwrap(), 2);
    assert!(env.get::<bool>("ok").unwrap());
    let c1: mlua::Table = env.get("c1").unwrap();
    assert_eq!(c1.get::<i64>("air_jumps").unwrap(), 3);
    assert!(c1.get::<bool>("shield").unwrap());
    assert!(!c1.get::<bool>("run").unwrap());
    assert!(c1.get::<bool>("grab").unwrap());
    assert_eq!(st.borrow().mv, Some((1, 3, 0b01001)));
    assert!(env.get::<mlua::Value>("e0").unwrap().is_nil());
    let e1: mlua::Table = env.get("e1").unwrap();
    assert_eq!(e1.get::<i64>("frames").unwrap(), 120);
    assert_eq!(e1.get::<f64>("value").unwrap(), 2.0);
    assert!(env.get::<String>("m1").unwrap().contains("finite and in range"));
    assert!(env.get::<String>("m2").unwrap().contains("invalid option 'bogus'"));
    assert!(env.get::<String>("m3").unwrap().contains("fighter entity must be 1-12"));
    assert!(env.get::<String>("m4").unwrap().contains("unknown fighter capability"));
    assert!(env.get::<String>("m5").unwrap().contains("air_jumps must be 0-8"));
}

#[test]
fn opponents_items_interrupt_armor() {
    let (eng, st) = rig();
    let env = run(
        &eng,
        "o",
        "n = gd.nearest_opponent(1)\n\
         n2 = gd.nearest_opponent(1, 2)\n\
         list = gd.opponents_in_radius(1, 50)\n\
         g1 = gd.give_item(1, 'random')\n\
         g2, why = gd.give_item(1, 99)\n\
         bi, bim = pcall(gd.give_item, 1, 'banana')\n\
         ok = gd.fighter_interrupt(1, {frames=30, exits={'jab','jump'}, guard=true})\n\
         iw = gd.fighter_interrupt(1)\n\
         a_ok = gd.fighter_armor(1, {type='damage_threshold', value=12, frames=60, direction='front'})\n\
         arm = gd.fighter_armor(1)\n\
         bad, bm = pcall(gd.fighter_armor, 1, {type='hit_count', value=2.5})\n\
         bad2, bm2 = pcall(gd.fighter_armor, 1, {type='super', from=9, to=3})\n\
         cleared = gd.fighter_armor(1, nil)\n\
         arm2 = gd.fighter_armor(1)",
    );
    assert_eq!(env.get::<i64>("n").unwrap(), 2);
    assert_eq!(env.get::<i64>("n2").unwrap(), 3);
    assert_eq!(env.get::<mlua::Table>("list").unwrap().raw_len(), 3);
    assert!(env.get::<bool>("g1").unwrap());
    assert!(!env.get::<bool>("g2").unwrap());
    assert_eq!(env.get::<String>("why").unwrap(), "no such item");
    assert_eq!(st.borrow().given, vec![(1, -1), (1, 99)]);
    assert!(env.get::<String>("bim").unwrap().contains("retail kind ID or 'random'"));
    assert_eq!(st.borrow().intr, Some((30, 0b1000001, 1)));
    let iw: mlua::Table = env.get("iw").unwrap();
    assert_eq!(iw.get::<i64>("frames").unwrap(), 30);
    let ex: mlua::Table = iw.get("exits").unwrap();
    assert!(ex.get::<bool>("jab").unwrap() && ex.get::<bool>("jump").unwrap());
    assert!(ex.get::<mlua::Value>("tilt").unwrap().is_nil());
    let arm: mlua::Table = env.get("arm").unwrap();
    let a1: mlua::Table = arm.get(1).unwrap();
    assert_eq!(a1.get::<String>("type").unwrap(), "damage_threshold");
    assert_eq!(a1.get::<f64>("value").unwrap(), 12.0);
    assert_eq!(a1.get::<String>("direction").unwrap(), "front");
    assert!(env.get::<String>("bm").unwrap().contains("hit_count value must be integer 1-255"));
    assert!(env.get::<String>("bm2").unwrap().contains("range is reversed"));
    assert_eq!(env.get::<mlua::Table>("arm2").unwrap().raw_len(), 0);
}

#[test]
fn offline_gate_and_unload_release() {
    let (eng, st) = rig();
    eng.st.borrow_mut().match_active = false;
    let env = run(&eng, "g", "ok, m = pcall(gd.fighter_caps, 1, {})");
    assert!(!env.get::<bool>("ok").unwrap());
    assert!(env.get::<String>("m").unwrap().contains("requires an active match"));
    let slot = eng.st.borrow().scripts.iter().position(|s| s.id == "g").unwrap();
    eng.unload(slot);
    assert_eq!(st.borrow().released, vec![slot as i32 + 1]);
}
