//! The script engine against the behaviours `gw_script.c` promises: the sandbox, manifests, the call budget,
//! hooks and tasks, the console, per-script data files, and the gd reads over a mock game.
use melee_script::game::{lab::*, *};
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct Mock {
    present: [bool; 6],
    action: [i32; 6],
}
impl ScriptGame for Mock {
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        let s = slot as usize;
        match field {
            SI_PRESENT => self.present[s] as i32,
            SI_CHAR => 2,
            SI_ACTION => self.action[s],
            SI_STOCKS => 4,
            SI_KIND => 1,
            _ => 0,
        }
    }
    fn fighter_f(&self, slot: i32, field: i32) -> f32 {
        match field {
            SF_X => 10.5 + slot as f32,
            SF_PERCENT => 33.0,
            SF_FACING => -1.0,
            _ => 0.0,
        }
    }
    fn lab_i(&self, _slot: i32, field: i32) -> i32 {
        match field {
            I_NAME_KIND => 1,
            I_MAX_JUMPS => 2,
            I_JUMPS_USED => 1,
            I_FLOOR_VALID => 1,
            I_HURTBOXES => 2,
            _ => 0,
        }
    }
    fn lab_f(&self, _slot: i32, field: i32) -> f32 {
        if field == F_FLOOR_Y { -3.25 } else { 0.0 }
    }
    fn hit_i(&self, _slot: i32, i: i32, field: i32) -> i32 {
        match (i, field) {
            (0, HI_STATE) => 1,
            (0, HI_ELEMENT) => 1,
            (0, HI_BKB) => 30,
            _ => 0,
        }
    }
    fn hit_f(&self, _slot: i32, i: i32, field: i32) -> f32 {
        if i == 0 && field == HF_DAMAGE { 12.0 } else { 0.0 }
    }
    fn hurt_i(&self, _slot: i32, i: i32, field: i32) -> i32 {
        if field == UI_HEIGHT { i + 1 } else { 0 }
    }
    fn camera_f(&self, field: i32) -> f32 {
        // an identity view looking down -z with a 90 degree fov and a 640x480 viewport
        match field {
            CAM_OK => 1.0,
            f if f == CAM_VIEW || f == CAM_VIEW + 5 || f == CAM_VIEW + 10 => 1.0,
            CAM_PROJ => 0.0,
            CAM_P0 => 90.0,
            CAM_P1 => 640.0 / 480.0,
            CAM_VP_XMIN => 0.0,
            CAM_VP_XMAX => 640.0,
            CAM_VP_YMIN => 0.0,
            CAM_VP_YMAX => 480.0,
            _ => 0.0,
        }
    }
    fn rng_seed(&self) -> u32 {
        0xDEADBEEF
    }
}

struct Rig {
    eng: Engine,
    host: Rc<RefCell<MemHost>>,
    dir: PathBuf,
}

fn rig_with(game: Box<dyn ScriptGame>) -> Rig {
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_test_{}_{}", std::process::id(), rand_tag()));
    let _ = std::fs::create_dir_all(&dir);
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    let eng = Engine::new(host.clone(), game, cfg).expect("engine");
    Rig { eng, host, dir }
}
fn rand_tag() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::SeqCst)
}
fn rig() -> Rig {
    rig_with(Box::new(Mock { present: [true, true, false, false, false, false], action: [14, 24, 0, 0, 0, 0] }))
}
impl Rig {
    fn load(&self, id: &str, src: &str) -> Option<usize> {
        self.eng.load_text(id, &format!("{id}.lua"), src.as_bytes(), None, "test")
    }
    fn logs(&self) -> Vec<String> {
        self.host.borrow().lines.clone()
    }
    fn out(&self) -> String {
        self.eng.console_lines().join("\n")
    }
}

#[test]
fn lua_is_the_vendored_5_4_7() {
    let r = rig();
    assert_eq!(r.eng.eval::<String>("return _VERSION").unwrap(), "Lua 5.4");
    // the same sequence the system lua 5.4.7 produces for math.randomseed(0): gameplay scripts stay reproducible
    assert_eq!(r.eng.eval::<i64>("return math.random(1,1000000)").unwrap(), 275394);
}

#[test]
fn sandbox_has_no_io_os_require_debug_dump() {
    let r = rig();
    for g in ["io", "os", "require", "dofile", "loadfile", "debug", "package", "rawget_", "collectgarbage_"] {
        if g.ends_with('_') {
            continue;
        }
        assert!(r.eng.eval::<bool>(&format!("return {g} == nil")).unwrap(), "{g} must be absent");
    }
    assert!(r.eng.eval::<bool>("return string.dump == nil").unwrap());
    assert_eq!(r.eng.eval::<String>("return getmetatable('')").unwrap(), "locked");
    assert!(r.eng.eval::<bool>("return unpack == table.unpack").unwrap());
    // collectgarbage only answers "count"
    assert!(r.eng.eval::<f64>("return collectgarbage('count')").unwrap() > 0.0);
    let e = r.eng.eval::<()>("collectgarbage('collect')").unwrap_err().to_string();
    assert!(e.contains("only \"count\" is available to scripts"), "{e}");
    // load: text only, in the caller's environment; bytecode refused
    assert_eq!(r.eng.eval::<i64>("x = 41 return load('x = x + 1 return x')()").unwrap(), 42);
    assert!(r.eng.eval::<bool>("local f, err = load('\\27Lua') return f == nil and err ~= nil").unwrap());
}

#[test]
fn environments_are_private() {
    let r = rig();
    r.load("a", "shared = 1; string.marker = 'a'").unwrap();
    r.load("b", "function peek() return shared, string.marker end").unwrap();
    let (a, b): (Option<i64>, Option<String>) = {
        let f: mlua::Function = r.eng.lua.globals().get::<mlua::Function>("tostring").unwrap();
        let _ = f;
        let env_b = r.eng.st.borrow().scripts[2].env.clone().unwrap();
        let peek: mlua::Function = env_b.get("peek").unwrap();
        peek.call(()).unwrap()
    };
    assert_eq!((a, b), (None, None), "script b sees nothing script a stored");
}

#[test]
fn manifest_headers_and_gameplay_gate() {
    let r = rig();
    let src = "-- @name: Demo\n-- @version: 1.2\n-- @gameplay: yes\nfunction on_tick() end\n";
    let i = r.load("demo", src).unwrap();
    {
        let s = r.eng.st.borrow();
        let sc = &s.scripts[i];
        assert_eq!((sc.name.as_str(), sc.version.as_str(), sc.gameplay), ("Demo", "1.2", true));
    }
    // a script without "gameplay" cannot pause the game
    r.load("quiet", "function on_load() end\nok, err = pcall(gd.pause)").unwrap();
    let env = r.eng.st.borrow().scripts[2].env.clone().unwrap();
    assert!(!env.get::<bool>("ok").unwrap());
    let err: String = env.get("err").unwrap();
    assert!(err.contains("changes gameplay: the script's mod.json must say \"gameplay\": true"), "{err}");
    // an API newer than this build refuses to load
    assert!(r.load("future", "-- @api_version: 3\n").is_none());
    assert!(r.out().contains("needs scripting API 3; this build has 2 - not loaded"));
}

#[test]
fn rollback_session_refuses_gameplay_writes() {
    let r = rig();
    r.load("g", "-- @gameplay: true\nok, err = pcall(gd.pause)").unwrap();
    assert!(r.eng.st.borrow().scripts[1].env.clone().unwrap().get::<bool>("ok").unwrap());
    r.host.borrow_mut().session = true;
    r.load("g2", "-- @gameplay: true\nok, err = pcall(gd.pause)").unwrap();
    let env = r.eng.st.borrow().scripts[2].env.clone().unwrap();
    assert!(!env.get::<bool>("ok").unwrap());
    assert!(env.get::<String>("err").unwrap().contains("offline-only"));
}

#[test]
fn runaway_call_is_stopped_by_the_budget() {
    let r = rig();
    // a load that spins: the budget trips, the load is reported and the script is dropped
    assert!(r.load("spin", "while true do end").is_none());
    assert!(r.out().contains("ran too long (limit: 2000000 instructions or 500 ms per call)"), "{}", r.out());
    // the engine is usable afterwards
    assert_eq!(r.eng.eval::<i64>("return 1 + 1").unwrap(), 2);
}

#[test]
fn runaway_task_is_stopped_and_reported() {
    let r = rig();
    r.load("t", "function on_load() end\ngd.run(function() while true do end end)").unwrap();
    r.eng.frame_post();
    assert!(r.out().contains("task: "), "{}", r.out());
    assert!(r.out().contains("ran too long"), "{}", r.out());
}

#[test]
fn errors_disable_a_script_after_twenty() {
    let r = rig();
    let i = r.load("bad", "function on_frame() error('boom') end").unwrap();
    for _ in 0..25 {
        r.eng.frame_post();
    }
    let s = r.eng.st.borrow();
    assert!(s.scripts[i].disabled);
    assert_eq!(s.scripts[i].errors, 20);
    drop(s);
    assert!(r.out().contains("switched off after 20 errors"));
}

#[test]
fn tasks_wait_in_frames() {
    let r = rig();
    r.load(
        "w",
        "n = 0\ngd.run(function() n = n + 1 gd.wait(3) n = n + 10 gd.wait(1) n = n + 100 end)",
    )
    .unwrap();
    let env = r.eng.st.borrow().scripts[1].env.clone().unwrap();
    let n = |e: &mlua::Table| e.get::<i64>("n").unwrap();
    // gd.wait(3) yields 3: skip two more frames, resume on the third after
    r.eng.run_tasks();
    assert_eq!(n(&env), 1);
    r.eng.run_tasks();
    r.eng.run_tasks();
    assert_eq!(n(&env), 1);
    r.eng.run_tasks();
    assert_eq!(n(&env), 11);
    r.eng.run_tasks(); // gd.wait(1) yields 1: the next frame resumes it
    assert_eq!(n(&env), 111);
    // finished: the slot is free again
    assert!(r.eng.st.borrow().scripts[1].tasks.iter().all(|t| t.is_none()));
}

#[test]
fn hooks_frames_and_match_start() {
    let r = rig();
    r.load(
        "h",
        "log = {}\nfunction on_frame_pre() log[#log+1]='pre' end\nfunction on_match_start() log[#log+1]='start' end\n\
         function on_frame() log[#log+1]='frame' end\nfunction on_tick() log[#log+1]='tick' end",
    )
    .unwrap();
    r.eng.frame_pre();
    r.eng.frame_post();
    r.eng.frame_post();
    r.eng.tick();
    let env = r.eng.st.borrow().scripts[1].env.clone().unwrap();
    let log: Vec<String> = env.get::<mlua::Table>("log").unwrap().sequence_values().map(|v| v.unwrap()).collect();
    assert_eq!(log, ["pre", "start", "frame", "frame", "tick"]);
    assert_eq!(r.eng.eval::<i64>("return gd.frame()").unwrap(), 2);
    let m: mlua::Table = r.eng.eval("return gd.match()").unwrap();
    assert!(m.get::<bool>("active").unwrap());
    assert_eq!(m.get::<i64>("frame").unwrap(), 1);
}

#[test]
fn console_commands_and_values() {
    let r = rig();
    let (rc, out) = r.eng.exec_capture("= 1 + 2");
    assert_eq!((rc, out.trim()), (0, "3")); // the "> line" echo is printed before the capture opens
    let (_, out) = r.eng.exec_capture("= {a=1}");
    assert!(out.contains("{a=1}"), "{out}");
    let (_, out) = r.eng.exec_capture("= 1/3");
    assert!(out.contains("0.3333"), "{out}");
    let (rc, _) = r.eng.exec_capture("x = ");
    assert_eq!(rc, -1);
    r.eng.exec("echo hello");
    assert!(r.out().contains("hello"));
    // gd.command registers a console command, run with the rest of the line
    r.eng.exec("gd.command('hi', function(arg) gd.log('got', arg) end, 'say hi')");
    r.eng.exec("HI there");
    assert!(r.out().contains("[console] got there"), "{}", r.out());
    let (_, out) = r.eng.exec_capture("help");
    assert!(out.contains("hi  say hi  [console]"), "{out}");
    r.eng.exec("label Run 7");
    assert_eq!(r.host.borrow().label, "Run 7");
    r.eng.exec("quit");
    assert!(r.host.borrow().quit);
}

#[test]
fn data_files_are_per_script_and_validated() {
    let r = rig();
    r.load("mods/one", "ok1 = pcall(gd.data_write, 'a/b.txt', 'hello')\n_, e1 = pcall(gd.data_write, '../x', 'no')\nr1 = gd.data_read('a/b.txt')\nr2, why = gd.data_read('nope')\nex = gd.data_exists('a/b.txt')\nat = gd.data_write_atomic('c.json', '{}')").unwrap();
    let env = r.eng.st.borrow().scripts[1].env.clone().unwrap();
    assert!(env.get::<bool>("ok1").unwrap());
    assert!(env.get::<String>("e1").unwrap().contains("data file names use letters, digits, _ - . and /"));
    assert_eq!(env.get::<String>("r1").unwrap(), "hello");
    assert_eq!(env.get::<Option<String>>("r2").unwrap(), None);
    assert_eq!(env.get::<String>("why").unwrap(), "missing");
    assert!(env.get::<bool>("ex").unwrap() && env.get::<bool>("at").unwrap());
    // the folder is the script id with '/' flattened
    assert!(r.dir.join("data/mods_one/a/b.txt").is_file());
}

#[test]
fn drawing_lists_and_colours() {
    let r = rig();
    r.load("d", "function on_tick() gd.text(1, 2, 'hi', gd.rgb(255, 0, 0)) gd.box(0, 0, 4, 4) gd.fill(1, 1, 2, 2) gd.line(0, 0, 9, 9, {r = 1, g = 2, b = 3}) end").unwrap();
    r.eng.tick(); // builds list 0
    r.eng.tick(); // swaps it to the shown side and builds the next
    let s = r.eng.st.borrow();
    let shown = &s.draw[1 - s.build];
    assert_eq!(shown.len(), 4);
    assert_eq!(shown[0].text, "hi");
    assert_eq!(shown[0].rgba, 0xFF0000FF);
    assert_eq!(shown[1].rgba, 0xFFFFFFFF);
    assert_eq!(shown[2].rgba, 0x000000A0);
    assert_eq!(shown[3].rgba, 0x010203FF);
}

#[test]
fn bad_arguments_read_like_lauxlib() {
    let r = rig();
    let e = r.eng.eval::<()>("gd.player()").unwrap_err().to_string();
    assert!(e.contains("bad argument #1 to 'player' (number expected, got no value)"), "{e}");
    let e = r.eng.eval::<()>("gd.player(9)").unwrap_err().to_string();
    assert!(e.contains("player/port numbers are 1-6 (got 9)"), "{e}");
    let e = r.eng.eval::<()>("gd.key('NOPE')").unwrap_err().to_string();
    assert!(e.contains("unknown key \"NOPE\""), "{e}");
}

#[test]
fn players_and_hit_reads_over_the_mock_game() {
    let r = rig();
    let n: i64 = r.eng.eval("return #gd.players()").unwrap();
    assert_eq!(n, 2);
    let p: mlua::Table = r.eng.eval("return gd.player(2)").unwrap();
    assert_eq!(p.get::<i64>("port").unwrap(), 2);
    assert_eq!(p.get::<String>("char_name").unwrap(), "Fox");
    assert_eq!(p.get::<i64>("facing").unwrap(), -1);
    assert_eq!(p.get::<f64>("x").unwrap(), 11.5);
    assert_eq!(p.get::<i64>("jumps_left").unwrap(), 1);
    assert_eq!(p.get::<f64>("floor_y").unwrap(), -3.25);
    assert_eq!(p.get::<String>("motion_name").unwrap(), "KneeBend"); // action 24
    assert!(r.eng.eval::<bool>("return gd.player(3) == nil").unwrap());
    let hb: mlua::Table = r.eng.eval("return gd.hitboxes(1)[1]").unwrap();
    assert_eq!(hb.get::<String>("element_name").unwrap(), "fire");
    assert_eq!(hb.get::<f64>("damage").unwrap(), 12.0);
    assert_eq!(r.eng.eval::<i64>("return #gd.hitboxes(1, true)").unwrap(), 5);
    let hu: String = r.eng.eval("return gd.hurtboxes(1)[2].height").unwrap();
    assert_eq!(hu, "high");
    assert_eq!(r.eng.eval::<i64>("return gd.rng()").unwrap(), 0xDEADBEEF);
    // char_name falls back to "character N"
    assert_eq!(r.eng.eval::<String>("return gd.char_name(99)").unwrap(), "character 99");
    assert_eq!(r.eng.eval::<String>("return gd.motion_name(341, 2)").unwrap(), melee_script::motion_names::SPECIAL[1][0]);
    assert_eq!(r.eng.eval::<String>("return gd.motion_name(-1)").unwrap(), "none");
}

#[test]
fn projection_matches_the_game_maths() {
    let r = rig();
    // identity-ish view: a point 10 units down -z at the origin lands at the screen centre
    let (sx, sy, vis, depth): (f64, f64, bool, f64) = r.eng.eval("return gd.project(0, 0, -10)").unwrap();
    assert!((sx - 320.0).abs() < 1e-3 && (sy - 240.0).abs() < 1e-3, "{sx} {sy}");
    assert!(vis);
    assert!((depth - 10.0).abs() < 1e-6);
    // behind the camera: not visible, depth reported
    let (_, _, vis, depth): (f64, f64, bool, f64) = r.eng.eval("return gd.project(0, 0, 5)").unwrap();
    assert!(!vis);
    assert_eq!(depth, -5.0);
}

#[test]
fn no_game_means_no_players() {
    let r = rig_with(Box::new(melee_script::NoGame));
    assert_eq!(r.eng.eval::<i64>("return #gd.players()").unwrap(), 0);
    assert!(r.eng.eval::<bool>("return gd.project(0, 0, 0) == nil").unwrap());
    assert!(r.eng.eval::<bool>("return gd.debug_stage() == nil").unwrap());
    let _ = r.logs();
}

#[test]
fn dir_scan_loads_folders_and_files() {
    let r = rig();
    let sd = r.dir.join("scripts");
    std::fs::create_dir_all(sd.join("pack")).unwrap();
    std::fs::write(sd.join("solo.lua"), "-- @name: Solo\nhello = true").unwrap();
    std::fs::write(sd.join("pack/mod.json"), "{\"id\": \"pack-id\", \"gameplay\": true, \"entry\": \"start.lua\"}").unwrap();
    std::fs::write(sd.join("pack/start.lua"), "ok = true").unwrap();
    std::fs::create_dir_all(sd.join("examples")).unwrap();
    std::fs::write(sd.join("examples/skip.lua"), "x = 1").unwrap();
    r.eng.scan_scripts_dir();
    let s = r.eng.st.borrow();
    let ids: Vec<&str> = s.scripts.iter().filter(|x| x.used).map(|x| x.id.as_str()).collect();
    assert!(ids.contains(&"solo") && ids.contains(&"pack-id") && !ids.contains(&"skip"), "{ids:?}");
    assert!(s.scripts.iter().find(|x| x.id == "pack-id").unwrap().gameplay);
}
