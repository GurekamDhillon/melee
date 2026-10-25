//! Engine events: queue after the frame, hooks in FramePost order, the on_event envelope, entity references.
//! Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

struct G;
impl ScriptGame for G {
    fn entity_epoch(&self) -> i32 {
        3
    }
    fn entity_field(&self, kind: i32, id: i32, field: i32) -> i32 {
        if kind == 1 && (0..12).contains(&id) {
            match field {
                0 => 1,
                1 => 3,
                2 => 7,
                3 => 9,
                _ => 0,
            }
        } else {
            0
        }
    }
    fn action_category(&self, motion: i32) -> i32 {
        if motion == 20 { 1 } else { 0 }
    }
    fn hit_i(&self, _s: i32, _i: i32, f: i32) -> i32 {
        if f == 4 { 55 } else { 0 }
    }
}

fn engine() -> Engine {
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_ev_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    Engine::new(host, Box::new(G), cfg).unwrap()
}

#[test]
fn events_dispatch_after_frame() {
    let eng = engine();
    let src = "-- @gameplay: true\n\
        log = {}\n\
        function on_action_change(p, old, new, sub) log[#log+1] = 'act:'..p..':'..old..':'..new..':'..tostring(sub) end\n\
        function on_jump(p, n, sub) log[#log+1] = 'jump:'..p..':'..n end\n\
        function on_hit(a, v, info) log[#log+1] = 'hit:'..tostring(a)..':'..v..':'..info.dealt..':'..tostring(info.hitbox)..':'..tostring(info.kbg) end\n\
        function on_event(e) log[#log+1] = 'ev:'..e.kind..':'..e.sequence..':'..e.phase..':'..tostring(e.cause)..':'..tostring(e.actor_ref) end\n\
        function on_target_broken(h, r) log[#log+1] = 'tb:'..h..':'..r end\n\
        function on_all_targets_broken() log[#log+1] = 'all' end\n\
        r1 = gd.entity_ref(2)\n\
        r2 = gd.entity_ref{port=1, sub=true}\n\
        rr = gd.entity_resolve(r1)\n\
        v1, v2 = gd.entity_valid(r1), gd.entity_valid('e1:1:1:1:1:1')\n\
        bad = pcall(gd.entity_ref, 'nonsense')\n\
        src = gd.source_ref('item', 'wand', 2, r1)\n\
        sr = gd.source_resolve(src)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert_eq!(env.get::<String>("r1").unwrap(), "e1:3:1:2:7:9");
    assert_eq!(env.get::<String>("r2").unwrap(), "e1:3:1:1:7:9");
    let rr: mlua::Table = env.get("rr").unwrap();
    assert_eq!(rr.get::<i64>("port").unwrap(), 2);
    assert!(env.get::<bool>("v1").unwrap());
    assert!(!env.get::<bool>("v2").unwrap());
    assert!(!env.get::<bool>("bad").unwrap());
    let sr: mlua::Table = env.get("sr").unwrap();
    assert_eq!(sr.get::<i64>("revision").unwrap(), 2);
    assert_eq!(sr.get::<String>("key").unwrap(), "wand");
    assert_eq!(env.get::<String>("src").unwrap(), "s1:4:2/e1:3:1:2:7:9/wand");

    eng.update_want_events();
    assert!(eng.st.borrow().ev.want);
    {
        let mut s = eng.st.borrow_mut();
        s.game_event(1, 0, 14, 20, 1); // action change into a jump-category motion
        s.game_event(2, 0, 1, 0, 1.5f32.to_bits() as i32);
        s.target_broken(4, 0);
    }
    eng.frame_pre();
    eng.frame_post();
    let log: mlua::Table = env.get("log").unwrap();
    let lines: Vec<String> = log.sequence_values::<String>().map(|v| v.unwrap()).collect();
    let want = [
        "ev:on_action_change:1:action:nil:e1:3:1:1:7:9",
        "act:1:14:20:true",
        "ev:on_action_signal:2:action:v1:3:0:1:e1:3:1:1:7:9", // placeholder, corrected below
    ];
    let _ = want;
    assert_eq!(lines[0], "ev:on_action_change:1:action:nil:e1:3:1:1:7:9");
    assert_eq!(lines[1], "act:1:14:20:true");
    assert_eq!(lines[2], "ev:on_jump:2:action:v1:3:0:1:e1:3:1:1:7:9");
    assert_eq!(lines[3], "jump:1:20");
    assert_eq!(lines[4], "ev:on_hit:3:collision:nil:nil");
    assert_eq!(lines[5], "hit:1:2:1.5:0:55");
    assert_eq!(lines[6], "ev:on_target_broken:4:lifecycle:nil:nil");
    assert_eq!(lines[7], "tb:4:0");
    assert_eq!(lines[8], "ev:on_all_targets_broken:5:lifecycle:nil:nil");
    assert_eq!(lines[9], "all");
    assert_eq!(lines.len(), 10);
    // consumed
    assert!(eng.st.borrow().ev.queue.is_empty());
}
