//! fly_target / fly_attack / fly_clear / fly_state and camera_params over a mock. Headless; no input sent.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct F {
    flying: bool,
    target: Option<(f32, f32)>,
    attack: Option<(i32, f32, i32)>,
    cam: [f32; 10],
    cam_owner: i32,
}
struct G(Rc<RefCell<F>>);
impl ScriptGame for G {
    fn geno_fly_get(&self, _s: i32) -> i32 {
        self.0.borrow().flying as i32
    }
    fn geno_fly_set(&mut self, _s: i32, mode: i32) -> i32 {
        self.0.borrow_mut().flying = mode == 1;
        0
    }
    fn geno_fly_target(&mut self, _s: i32, x: i32, y: i32) -> i32 {
        self.0.borrow_mut().target = Some((f32::from_bits(x as u32), f32::from_bits(y as u32)));
        0
    }
    fn geno_fly_attack_config(&mut self, _s: i32, flags: i32, damage: i32, radius: i32, active_gap: i32, _kb: i32, _angle: i32) -> i32 {
        let mut f = self.0.borrow_mut();
        f.attack = if flags & 1 != 0 { Some((damage, f32::from_bits(radius as u32), active_gap)) } else { None };
        0
    }
    fn geno_fly_state(&self, _s: i32, field: i32) -> i32 {
        let f = self.0.borrow();
        match field {
            0 => f.attack.is_some() as i32,
            1 => f.target.is_some() as i32,
            2 => f.attack.map_or(0, |a| a.0),
            3 => f.attack.map_or(0, |a| a.1.to_bits() as i32),
            _ => 0,
        }
    }
    fn geno_fly_clear(&mut self, _s: i32) {
        let mut f = self.0.borrow_mut();
        f.target = None;
        f.attack = None;
    }
    fn camera_param_read(&self, i: i32) -> i32 {
        self.0.borrow().cam[i as usize].to_bits() as i32
    }
    fn camera_param_owner(&self) -> i32 {
        self.0.borrow().cam_owner
    }
    fn camera_params(&mut self, owner: i32, mask: i32, bits: &[i32]) -> i32 {
        let mut f = self.0.borrow_mut();
        for i in 0..10 {
            if mask & (1 << i) != 0 {
                f.cam[i] = f32::from_bits(bits[i] as u32);
            }
        }
        f.cam_owner = if mask != 0 { owner } else { 0 };
        1
    }
}

#[test]
fn fly_and_camera() {
    let f = Rc::new(RefCell::new(F::default()));
    f.borrow_mut().cam = [10.0, 500.0, 30.0, 1.0, 1.0, 1.0, 30.0, 0.0, 1.0, 1.0];
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_fly_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(f.clone())), cfg).unwrap();
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        t = gd.fly_target(1, 12.5, -3)\n\
        a = gd.fly_attack(1, true, {damage=7, radius=4, active=5, gap=10})\n\
        st = gd.fly_state(1)\n\
        bad1, m1 = pcall(gd.fly_attack, 1, true, {damage=99})\n\
        bad2, m2 = pcall(gd.fly_attack, 1, true, {kb=1.5})\n\
        bad3, m3 = pcall(gd.fly_target, 1, 1/0, 0)\n\
        prev = gd.camera_params({fov=40, tilt=40, min_dist=20})\n\
        now = gd.camera_params()\n\
        bad4, m4 = pcall(gd.camera_params, {fov=40, tilt=50})\n\
        bad5, m5 = pcall(gd.camera_params, {nope=1})\n\
        bad6, m6 = pcall(gd.camera_params, {fov=1000})\n\
        gd.fly_clear(1)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert!(env.get::<bool>("t").unwrap());
    assert!(env.get::<bool>("a").unwrap());
    let st: mlua::Table = env.get("st").unwrap();
    assert!(st.get::<bool>("flying").unwrap() && st.get::<bool>("targeting").unwrap() && st.get::<bool>("attacking").unwrap());
    assert_eq!(st.get::<i64>("damage").unwrap(), 7);
    assert_eq!(st.get::<f64>("radius").unwrap(), 4.0);
    assert!(env.get::<String>("m1").unwrap().contains("requires damage 1..30") || env.get::<String>("m1").unwrap().contains("damage must be an integer 1..30"));
    assert!(env.get::<String>("m2").unwrap().contains("kb must be an integer 1..2000"));
    assert!(env.get::<String>("m3").unwrap().contains("finite coordinates"));
    assert_eq!(env.get::<mlua::Table>("prev").unwrap().get::<f64>("fov").unwrap(), 30.0);
    assert_eq!(env.get::<mlua::Table>("now").unwrap().get::<f64>("min_dist").unwrap(), 20.0);
    assert!(env.get::<String>("m4").unwrap().contains("fov and tilt must agree"));
    assert!(env.get::<String>("m5").unwrap().contains("unknown camera parameter"));
    assert!(env.get::<String>("m6").unwrap().contains("fov must be in 1.000000..89.000000"));
    assert!(f.borrow().target.is_none() && f.borrow().attack.is_none());
    assert_eq!(f.borrow().cam[2], 40.0);
}
