//! Deadlines, skill/crit/armor events, skill ring and shock over a mock game. Headless; no input sent.
use melee_script::{Config, Engine, Host, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct S {
    shock: Option<(i32, i32, i32, i32)>,
    owner: i32,
}
struct G(Rc<RefCell<S>>);
impl ScriptGame for G {
    fn skill_row_count(&self) -> i32 {
        2
    }
    fn skill_row(&self, idx: i32, field: i32) -> i32 {
        // row 0 newest: lcancel (kind 1) on entity 1; row 1: wavedash (kind 5) on entity 1
        match (idx, field) {
            (0, 0) => 40,
            (0, 1) => 1,
            (0, 2) => 1,
            (0, 3) => 1,  // fair
            (0, 4) => 1,  // hit
            (0, 5) => 3,  // trigger age
            (0, 6) => 2.0f32.to_bits() as i32,
            (0, 7) => 1.0f32.to_bits() as i32,
            (1, 0) => 30,
            (1, 1) => 5,
            (1, 2) => 1,
            _ => 0,
        }
    }
    fn skill_const(&self, i: i32) -> i32 {
        if i == 0 { 1.5f32.to_bits() as i32 } else { 20 + i }
    }
    fn shock_read(&self, _e: i32, field: i32) -> i32 {
        match (&self.0.borrow().shock, field) {
            (None, _) => 0,
            (Some(_), 0) => self.0.borrow().owner,
            (Some(s), 1) => s.0,
            (Some(s), 2) => s.1,
            (Some(s), 3) => s.2,
            (Some(_), 4) => 1.5f32.to_bits() as i32,
            _ => 0,
        }
    }
    fn shock_set(&mut self, _e: i32, owner: i32, frames: i32, charges: i32, _m: i32, bonus: i32, flags: i32) -> i32 {
        self.0.borrow_mut().owner = owner;
        self.0.borrow_mut().shock = if flags == 4 { None } else { Some((frames, charges, bonus, flags)) };
        1
    }
}

#[derive(Default)]
struct H {
    hang: Rc<RefCell<Vec<(i32, i32, String)>>>,
    inner: MemHost,
}
impl Host for H {
    fn log(&mut self, m: &str) {
        self.inner.log(m)
    }
    fn now_ms(&self) -> f64 {
        0.0
    }
    fn hang_script_state(&mut self, p: i32, e: i32, l: &str) {
        self.hang.borrow_mut().push((p, e, l.to_string()));
    }
}

fn engine(s: Rc<RefCell<S>>, hang: Rc<RefCell<Vec<(i32, i32, String)>>>) -> Engine {
    let host = Rc::new(RefCell::new(H { hang, ..Default::default() }));
    let dir = std::env::temp_dir().join(format!("melee_script_sd_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    Engine::new(host, Box::new(G(s)), cfg).unwrap()
}

#[test]
fn deadline_skill_shock() {
    let s = Rc::new(RefCell::new(S::default()));
    let hang = Rc::new(RefCell::new(Vec::new()));
    let eng = engine(s.clone(), hang.clone());
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        log = {}\n\
        function on_deadline(name, waited) log[#log+1] = 'deadline:'..name..':'..waited end\n\
        function on_lcancel(i) log[#log+1] = 'lc:'..i.aerial..':'..tostring(i.hit)..':'..i.trigger_age..':'..i.lag end\n\
        function on_skill(i) log[#log+1] = 'skill:'..i.kind..':'..i.port..':'..i.frame end\n\
        function on_crit(i) log[#log+1] = 'crit:'..i.attacker..':'..i.victim..':'..i.move_tag..':'..i.added_damage..':'..tostring(i.forced) end\n\
        function on_armor(i) log[#log+1] = 'armor:'..i.type..':'..tostring(i.absorbed)..':'..tostring(i.broke)..':'..i.damage end\n\
        d1 = gd.deadline('stage', 2)\n\
        d2 = gd.deadline('stage', 5)\n\
        done = gd.deadline_done('nope')\n\
        hist = gd.skill_history(1, 4)\n\
        only = gd.skill_history(1, 4, 'wavedash')\n\
        kinds = gd.skill_kinds()\n\
        th = gd.skill_thresholds()\n\
        bad = pcall(gd.skill_history, 1, 4, 'nonsense')\n\
        sh0 = gd.shock(1)\n\
        okk = gd.shock(1, {frames=100, charges=3, hitstun=2, stack=true})\n\
        sh1 = gd.shock(1)\n\
        badk, mk = pcall(gd.shock, 1, {frames=0})\n\
        cleared = gd.shock(1, nil)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<bool>("d1").unwrap());
    assert!(!env.get::<bool>("d2").unwrap());
    assert!(!env.get::<bool>("done").unwrap());
    let hist: mlua::Table = env.get("hist").unwrap();
    assert_eq!(hist.raw_len(), 2);
    let r0: mlua::Table = hist.get(1).unwrap();
    assert_eq!(r0.get::<String>("kind").unwrap(), "lcancel");
    assert_eq!(r0.get::<String>("aerial").unwrap(), "fair");
    assert_eq!(r0.get::<f64>("lag").unwrap(), 2.0);
    let only: mlua::Table = env.get("only").unwrap();
    assert_eq!(only.raw_len(), 1);
    assert_eq!(only.get::<mlua::Table>(1).unwrap().get::<String>("kind").unwrap(), "wavedash");
    assert_eq!(env.get::<mlua::Table>("kinds").unwrap().get::<String>(24).unwrap(), "shock_end");
    assert_eq!(env.get::<mlua::Table>("th").unwrap().get::<f64>("wave_min_speed").unwrap(), 1.5);
    assert!(!env.get::<bool>("bad").unwrap());
    assert!(env.get::<mlua::Value>("sh0").unwrap().is_nil());
    assert!(env.get::<bool>("okk").unwrap());
    assert_eq!(env.get::<mlua::Table>("sh1").unwrap().get::<i64>("charges").unwrap(), 3);
    assert!(!env.get::<bool>("badk").unwrap(), "{:?}", env.get::<mlua::Value>("mk"));
    assert!(env.get::<bool>("cleared").unwrap());
    assert_eq!(s.borrow().shock, None);
    assert_eq!(hang.borrow().last().unwrap().0, 1); // one pending deadline published

    eng.update_want_events();
    {
        let mut st = eng.st.borrow_mut();
        st.skill_event(1, 1, 1, 1, 3, 2.0f32.to_bits() as i32, 1.0f32.to_bits() as i32);
        st.crit_event(1, 2, 5 | 0x100, 1.5f32.to_bits() as i32, 10.0f32.to_bits() as i32, 15.0f32.to_bits() as i32, 0, 0, 0, 0, 0, 1.5f32.to_bits() as i32);
        st.armor_event(1, 3, true, false, 4.0f32.to_bits() as i32, 0);
    }
    eng.frame_pre();
    eng.frame_post();
    eng.frame_pre();
    eng.frame_post();
    let log: mlua::Table = env.get("log").unwrap();
    let lines: Vec<String> = log.sequence_values::<String>().map(|v| v.unwrap()).collect();
    assert_eq!(
        lines,
        [
            "lc:fair:true:3:2.0",
            "skill:lcancel:1:40",
            "crit:1:2:aerial:5.0:true",
            "armor:knockback_threshold:true:false:4.0",
            "deadline:stage:2"
        ]
        .map(String::from)
    );
    let last = hang.borrow().last().unwrap().clone();
    assert_eq!((last.0, last.1, last.2.as_str()), (0, 1, "stage"));
}
