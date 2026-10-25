//! Contact observer, waits and the contact console over a mock game. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct W {
    on_floor: bool,
    x: f32,
    y: f32,
    action: i32,
}
struct G(Rc<RefCell<W>>);
impl ScriptGame for G {
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        match (slot, field) {
            (0, 0) => 1,                          // present
            (0, 3) => self.0.borrow().action,     // action
            (0, 4) => !self.0.borrow().on_floor as i32, // airborne
            _ => 0,
        }
    }
    fn fighter_f(&self, slot: i32, field: i32) -> f32 {
        let w = self.0.borrow();
        match (slot, field) {
            (0, 0) => w.x,
            (0, 1) => w.y,
            (0, 5) => 1.0,
            _ => 0.0,
        }
    }
    fn contact_field(&self, slot: i32, surface: i32, field: i32, _line: i32) -> i32 {
        if slot != 0 || surface != 0 || !self.0.borrow().on_floor {
            return -1;
        }
        let f = |v: f32| v.to_bits() as i32;
        match field {
            0 => 4,      // line
            1 => 0,      // owner: the stage
            2 => 1,      // part
            3 => 0,      // kind
            4 => 0,      // pass
            5 => f(-50.0),
            6 => f(0.0),
            7 => f(50.0),
            8 => f(0.0),
            9 => f(self.0.borrow().x),
            10 => f(0.0),
            _ => 0,
        }
    }
}

fn engine(w: Rc<RefCell<W>>) -> (Engine, std::path::PathBuf) {
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_contacts_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let _ = std::fs::create_dir_all(&cfg.data_dir);
    (Engine::new(host, Box::new(G(w)), cfg).unwrap(), dir)
}

fn frame(eng: &Engine) {
    eng.frame_pre();
    eng.frame_post();
}

#[test]
fn contacts_events_and_waits() {
    let w = Rc::new(RefCell::new(W::default()));
    w.borrow_mut().y = 30.0;
    let (eng, dir) = engine(w.clone());
    let src = "-- @gameplay: true\n\
        c0 = gd.contacts(1)\n\
        h = gd.wait_until{port=1, x=10, y=0, radius=2, timeout=20}\n\
        h2 = gd.wait_until{port=1, airborne=false, timeout=3}\n\
        bad, m1 = pcall(gd.wait_until, {port=1})\n\
        bad2, m2 = pcall(gd.wait_until, {port=1, bogus=1})\n\
        bad3, m3 = pcall(gd.wait_until, {port=9, x=1})\n\
        st0 = gd.wait_status(h)\n\
        stale, why = gd.wait_status(999)\n\
        ok = gd.contact_trace{file='ct.jsonl'}\n\
        function status() return gd.wait_status(h), gd.wait_status(h2) end\n\
        function arm() h3 = gd.wait_until{port=1, y=500, radius=1, timeout=3} end\n\
        function status3() return gd.wait_status(h3) end\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    // match not active yet: absent
    assert!(env.get::<mlua::Value>("c0").unwrap().is_nil());
    assert!(!env.get::<bool>("bad").unwrap());
    assert!(env.get::<String>("m1").unwrap().contains("wait needs at least one condition"));
    assert!(env.get::<String>("m2").unwrap().contains("unknown wait condition"));
    assert!(env.get::<String>("m3").unwrap().contains("wait port must be 1..6"));
    assert!(!env.get::<mlua::Table>("st0").unwrap().get::<bool>("done").unwrap());
    assert!(env.get::<String>("why").unwrap().contains("stale wait handle"));
    assert!(env.get::<bool>("ok").unwrap());

    // the fighter arrives: first frame starts the match (airborne), then lands on the floor
    frame(&eng);
    assert!(eng.st.borrow().match_active);
    w.borrow_mut().on_floor = true;
    w.borrow_mut().x = 10.0;
    w.borrow_mut().y = 0.0;
    w.borrow_mut().action = 14;
    frame(&eng);
    w.borrow_mut().on_floor = false;
    w.borrow_mut().y = 5.0;
    frame(&eng);
    let r = eng
        .eval::<mlua::MultiValue>("local ev, nxt = gd.contact_events(0); return #ev, ev[1].event, ev[1].label, ev[1].kind, ev[2].event, ev[2].reason, nxt")
        .unwrap();
    let v: Vec<String> = r.iter().map(|v| format!("{v:?}")).collect();
    assert!(v[1].contains("land"), "{v:?}");
    assert!(v[3].contains("floor"), "{v:?}");
    assert!(v[4].contains("leave"), "{v:?}");
    assert!(v[5].contains("lost_contact"), "{v:?}");
    // waits: h matched when the fighter stood within 2 of (10,0); h2 (airborne=false) matched on landing
    let f: mlua::Function = env.get("status").unwrap();
    let r = eng.pcall(i, &f, mlua::MultiValue::new(), "status").unwrap();
    let t1 = match &r[0] {
        mlua::Value::Table(t) => t.clone(),
        o => panic!("{o:?}"),
    };
    let t2 = match &r[1] {
        mlua::Value::Table(t) => t.clone(),
        o => panic!("{o:?}"),
    };
    assert!(t1.get::<bool>("done").unwrap() && t1.get::<bool>("ok").unwrap());
    assert_eq!(t1.get::<String>("reason").unwrap(), "conditions reached");
    assert!(t2.get::<bool>("done").unwrap() && t2.get::<bool>("ok").unwrap());
    // a timeout reports why
    eng.pcall(i, &env.get::<mlua::Function>("arm").unwrap(), mlua::MultiValue::new(), "arm").unwrap();
    for _ in 0..3 {
        frame(&eng);
    }
    let r = eng.pcall(i, &env.get::<mlua::Function>("status3").unwrap(), mlua::MultiValue::new(), "s3").unwrap();
    let t3 = match &r[0] {
        mlua::Value::Table(t) => t.clone(),
        o => panic!("{o:?}"),
    };
    assert!(t3.get::<bool>("done").unwrap() && !t3.get::<bool>("ok").unwrap());
    assert!(t3.get::<String>("reason").unwrap().contains("timeout 3 frames; last at x=10.0 y=5.0 in "), "{:?}", t3.get::<String>("reason"));
    // the trace file got the land and leave records
    eng.exec("trace off");
    let text = std::fs::read_to_string(dir.join("data").join("a").join("ct.jsonl")).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].starts_with("{\"seq\":"));
    assert!(lines[0].contains("\"event\":\"land\""));
    assert!(lines[1].contains("\"event\":\"leave\",\"reason\":\"lost_contact\""));
    // console
    assert_eq!(eng.exec("contacts 1"), 0);
    assert_ne!(eng.exec("contacts 9"), 0);
    assert_eq!(eng.exec("wait port=1 y=5 radius=1 timeout=5"), 0);
    let _ = std::fs::remove_dir_all(&dir);
}
