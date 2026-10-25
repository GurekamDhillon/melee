//! scene_launch / training_select / screenshot / hud_visible / launch_request over mocks. Headless; no input sent.
use melee_script::{Config, Engine, Host, MemHost, SceneLaunchInfo, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct F {
    text: Option<String>,
    kit: bool,
    hidden: bool,
    shots: Vec<String>,
    launched: Vec<i32>,
    transition: bool,
}
struct H(Rc<RefCell<F>>, MemHost);
impl Host for H {
    fn log(&mut self, m: &str) {
        self.1.log(m)
    }
    fn now_ms(&self) -> f64 {
        0.0
    }
    fn scene_launch_set_text(&mut self, t: Option<&str>) {
        self.0.borrow_mut().text = t.map(String::from);
    }
    fn scene_launch_boot_game_mode(&mut self) -> i32 {
        match self.0.borrow().text.as_deref() {
            Some(t) if t.starts_with("mode=training") => 7,
            _ => -1,
        }
    }
    fn scene_launch_info(&self) -> SceneLaunchInfo {
        SceneLaunchInfo { mission: "m1".into(), module: "envoy".into(), maze_seed: 5, maze_size: 3 }
    }
    fn training_select(&self) -> bool {
        self.0.borrow().kit
    }
    fn set_training_select(&mut self, kit: bool) {
        self.0.borrow_mut().kit = kit;
    }
    fn screenshot(&mut self, p: &std::path::Path) -> bool {
        self.0.borrow_mut().shots.push(p.file_name().unwrap().to_string_lossy().to_string());
        true
    }
    fn status_hud_visible(&self) -> bool {
        !self.0.borrow().hidden
    }
    fn set_status_hud_hidden(&mut self, h: bool) {
        self.0.borrow_mut().hidden = h;
    }
    fn hang_transition(&mut self, on: bool) {
        self.0.borrow_mut().transition = on;
    }
}
struct G(Rc<RefCell<F>>);
impl ScriptGame for G {
    fn launch_scene(&mut self, mode: i32) {
        self.0.borrow_mut().launched.push(mode);
    }
}

#[test]
fn frontend() {
    let f = Rc::new(RefCell::new(F::default()));
    let host = Rc::new(RefCell::new(H(f.clone(), MemHost::default())));
    let dir = std::env::temp_dir().join(format!("melee_script_frontend_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(f.clone())), cfg).unwrap();
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        t = gd.scene_launch{mode='training', p1='fox'}\n\
        bad, m = pcall(gd.scene_launch, 'mode=nope')\n\
        ts0 = gd.training_select()\n\
        ts1 = gd.training_select('kit')\n\
        badts, mts = pcall(gd.training_select, 'xx')\n\
        ok, path = gd.screenshot('hello')\n\
        h0 = gd.hud_visible()\n\
        h1 = gd.hud_visible(false)\n\
        lr = gd.launch_request()\n\
        gd.scene_clear()\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<String>("t").unwrap().starts_with("mode=training;"));
    assert!(env.get::<String>("t").unwrap().contains("p1=fox"));
    assert!(!env.get::<bool>("bad").unwrap());
    assert!(env.get::<String>("m").unwrap().contains("was refused or names no scene"));
    assert_eq!(env.get::<String>("ts0").unwrap(), "native");
    assert_eq!(env.get::<String>("ts1").unwrap(), "kit");
    assert!(env.get::<String>("mts").unwrap().contains("\"kit\" or \"native\""));
    assert!(env.get::<bool>("ok").unwrap());
    assert!(env.get::<String>("path").unwrap().ends_with("hello.png"));
    assert!(env.get::<bool>("h0").unwrap());
    assert!(!env.get::<bool>("h1").unwrap());
    let lr: mlua::Table = env.get("lr").unwrap();
    assert_eq!(lr.get::<String>("mission").unwrap(), "m1");
    assert_eq!(lr.get::<i64>("size").unwrap(), 3);
    assert!(f.borrow().text.is_none());
    // the launch happens at the next tick, outside any frame
    assert!(f.borrow().launched.is_empty());
    eng.tick();
    assert_eq!(f.borrow().launched, vec![7]);
    assert!(f.borrow().transition);
    eng.tick();
    assert_eq!(f.borrow().launched, vec![7]);
}
