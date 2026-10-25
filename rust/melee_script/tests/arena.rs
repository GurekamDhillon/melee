//! arena-hooks (pc/tests/arena-hooks.lua's API surface) and the merged gd.stage_bounds over a mock arena.
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

fn b(f: f32) -> i32 {
    f.to_bits() as i32
}
fn f(i: i32) -> f32 {
    f32::from_bits(i as u32)
}

#[derive(Default)]
struct Arena {
    owner: i32,
    cam: [f32; 4],
    blast: [f32; 4],
    origin: [f32; 2],
    frames: i32,
    groups: Vec<(bool, i32)>,
    spawns: Vec<(i32, f32, f32)>,
    map: [[f32; 4]; 2],
    released: Vec<i32>,
}
struct G(Rc<RefCell<Arena>>);
impl ScriptGame for G {
    fn arena_owner(&self) -> i32 {
        self.0.borrow().owner
    }
    fn arena_origin(&mut self, owner: i32, x: i32, y: i32, frames: i32) {
        let mut a = self.0.borrow_mut();
        a.owner = owner;
        a.origin = [f(x), f(y)];
        a.frames = frames;
    }
    fn arena_spawn(&mut self, owner: i32, slot: i32, x: i32, y: i32) -> i32 {
        let mut a = self.0.borrow_mut();
        a.owner = owner;
        a.spawns.push((slot, f(x), f(y)));
        1
    }
    fn arena_spawn_pos(&self, slot: i32, axis: i32, fallback: f32) -> f32 {
        match self.0.borrow().spawns.iter().rev().find(|s| s.0 == slot) {
            Some(s) => [s.1, s.2, 0.0][axis as usize],
            None => fallback,
        }
    }
    fn arena_bounds(&mut self, owner: i32, blast: bool, l: i32, r: i32, t: i32, bt: i32) {
        let mut a = self.0.borrow_mut();
        a.owner = owner;
        let v = [f(l), f(r), f(t), f(bt)];
        if blast { a.blast = v } else { a.cam = v }
    }
    fn arena_restore(&mut self) {
        let mut a = self.0.borrow_mut();
        a.owner = 0;
        a.cam = [-1.0, 1.0, 2.0, -2.0];
    }
    fn arena_read(&self, field: i32) -> i32 {
        let a = self.0.borrow();
        match field {
            0..=3 => b(a.cam[field as usize]),
            4..=7 => b(a.blast[field as usize - 4]),
            8 => b(a.origin[0]),
            9 => b(a.origin[1]),
            _ => a.frames,
        }
    }
    fn arena_group(&self, id: i32, field: i32) -> i32 {
        let a = self.0.borrow();
        if id < 0 {
            return a.groups.len() as i32;
        }
        let g = a.groups[id as usize];
        if field == 0 { g.0 as i32 } else { g.1 }
    }
    fn arena_group_set(&mut self, owner: i32, id: i32, enabled: bool) {
        self.0.borrow_mut().groups[id as usize] = (enabled, owner);
    }
    fn arena_release(&mut self, owner: i32) {
        self.0.borrow_mut().released.push(owner);
    }
    fn stage_bounds_i(&self, field: i32) -> i32 {
        match field {
            12 => 1,
            13 => b(7.5),
            8..=11 => b(field as f32),
            _ => b(99.0),
        }
    }
    fn map_bounds(&mut self, which: i32, l: i32, r: i32, bt: i32, t: i32) {
        let mut a = self.0.borrow_mut();
        if which < 0 {
            a.map = [[0.0; 4]; 2];
        } else {
            a.map[which as usize] = [f(l), f(r), f(bt), f(t)];
        }
    }
    fn map_bound(&self, which: i32, field: i32) -> i32 {
        b(self.0.borrow().map[which as usize][field as usize])
    }
}

fn rig() -> (Engine, Rc<RefCell<Arena>>) {
    let st = Rc::new(RefCell::new(Arena { groups: vec![(true, 0), (true, 0)], ..Default::default() }));
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_arena_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host, Box::new(G(st.clone())), cfg).unwrap();
    (eng, st)
}

fn run(eng: &Engine, id: &str, src: &str) -> mlua::Table {
    let full = format!("-- @gameplay: true\n{src}");
    let i = eng.load_text(id, &format!("{id}.lua"), full.as_bytes(), None, "test").expect("load");
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    env
}

#[test]
fn needs_a_match_and_a_gameplay_script() {
    let (eng, _) = rig();
    let env = run(&eng, "a", "ok, err = pcall(gd.stage_set_origin, 0, 0)");
    assert!(!env.get::<bool>("ok").unwrap());
    assert!(env.get::<String>("err").unwrap().contains("requires an active match"));
    eng.st.borrow_mut().match_active = true;
    let env = run(&eng, "b", "ok, err = pcall(gd.stage_set_origin, 0, 0) b = gd.stage_bounds()");
    assert!(env.get::<bool>("ok").unwrap());
    assert!(env.get::<mlua::Table>("b").is_ok());
}

#[test]
fn validation_matches_arena_hooks_lua() {
    let (eng, _) = rig();
    eng.st.borrow_mut().match_active = true;
    let env = run(
        &eng,
        "v",
        "r1 = pcall(gd.stage_set_camera_bounds, 10, -10, 50, -50)\n\
         r2 = pcall(gd.stage_set_origin, 0/0, 0)\n\
         r3 = pcall(gd.stage_set_origin, 0, 0, {frames=-1})\n\
         r4 = pcall(gd.stage_collision_group, -1, false)\n\
         r5 = pcall(gd.stage_collision_group, 0)\n\
         r6, e6 = pcall(gd.stage_set_spawn, 9, 0, 0)\n\
         r7 = pcall(gd.stage_set_spawn, 130, 1, 2)",
    );
    for k in ["r1", "r2", "r3", "r4", "r5", "r6"] {
        assert!(!env.get::<bool>(k).unwrap(), "{k}");
    }
    assert!(env.get::<String>("e6").unwrap().contains("spawn slot must be 0-7"));
    assert!(env.get::<bool>("r7").unwrap());
}

#[test]
fn bounds_origin_groups_and_ownership() {
    let (eng, st) = rig();
    eng.st.borrow_mut().match_active = true;
    let env = run(
        &eng,
        "own",
        "gd.stage_set_camera_bounds(-60, 60, 90, -35)\n\
         gd.stage_set_blast_bounds(-90, 90, 120, -65)\n\
         gd.stage_set_origin(12, 6, {frames=60})\n\
         bb = gd.stage_bounds()\n\
         n = #gd.stage_collision_groups()\n\
         gd.stage_collision_group(1, false)\n\
         en = gd.stage_collision_groups()[2].enabled\n\
         sx, sy, sz = gd.stage_spawn(3)\n\
         applied = gd.stage_set_spawn(3, 4.5, -2)\n\
         tx, ty = gd.stage_spawn(3)",
    );
    let bb: mlua::Table = env.get("bb").unwrap();
    let cam: mlua::Table = bb.get("camera").unwrap();
    assert_eq!(cam.get::<f64>("left").unwrap(), -60.0);
    assert_eq!(cam.get::<f64>("top").unwrap(), 90.0);
    let blast: mlua::Table = bb.get("blast").unwrap();
    assert_eq!(blast.get::<f64>("bottom").unwrap(), -65.0);
    assert_eq!(bb.get::<mlua::Table>("origin").unwrap().get::<f64>("x").unwrap(), 12.0);
    assert_eq!(bb.get::<i64>("frames").unwrap(), 60);
    // the model-gaps reader contributes main_floor and surface_top, not its own camera/blast
    assert_eq!(bb.get::<f64>("surface_top").unwrap(), 7.5);
    assert_eq!(bb.get::<mlua::Table>("main_floor").unwrap().get::<f64>("right").unwrap(), 9.0);
    assert_eq!(env.get::<i64>("n").unwrap(), 2);
    assert!(!env.get::<bool>("en").unwrap());
    assert!(env.get::<bool>("applied").unwrap());
    assert_eq!(env.get::<f64>("tx").unwrap(), 4.5);
    assert_eq!(env.get::<f64>("sx").unwrap(), 0.0);
    let owner = st.borrow().owner;
    assert_eq!(owner, 1 + eng.st.borrow().scripts.iter().position(|s| s.id == "own").unwrap() as i32);

    // another script cannot touch what the first owns
    let env2 = run(&eng, "other", "ok, err = pcall(gd.stage_set_origin, 1, 1)\nok2, err2 = pcall(gd.stage_collision_group, 1, true)");
    assert!(!env2.get::<bool>("ok").unwrap());
    assert!(env2.get::<String>("err").unwrap().contains("bounds are owned by another script"));
    assert!(env2.get::<String>("err2").unwrap().contains("collision group is owned by another script"));

    // unload releases the arena
    let slot = eng.st.borrow().scripts.iter().position(|s| s.id == "own").unwrap();
    eng.unload(slot);
    assert_eq!(st.borrow().released, vec![slot as i32 + 1]);
}

#[test]
fn largemap_set_and_restore() {
    let (eng, st) = rig();
    eng.st.borrow_mut().match_active = true;
    let env = run(
        &eng,
        "lm",
        "r = gd.stage_bounds({camera={left=-5,right=5,bottom=-3,top=3}})\n\
         bad = pcall(gd.stage_bounds, {blast={left=5,right=-5,bottom=0,top=1}})\n\
         partial = pcall(gd.stage_bounds, {blast={left=1}})\n\
         gd.stage_bounds(false)\n\
         z = gd.stage_bounds(false)",
    );
    // the first call set the camera rectangle (read back through MapBound) before the restore
    assert!(!env.get::<bool>("bad").unwrap());
    assert!(!env.get::<bool>("partial").unwrap());
    let r: mlua::Table = env.get("r").unwrap();
    assert_eq!(r.get::<mlua::Table>("camera").unwrap().get::<f64>("right").unwrap(), 5.0);
    assert_eq!(st.borrow().map[0], [0.0; 4]);
}
