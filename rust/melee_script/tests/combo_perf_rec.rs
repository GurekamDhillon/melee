//! gd.combo / gd.combo_ki / gd.perf / gd.prof / gd.rec_* over a mock game and host.
use melee_script::host::{PerfFrame, PerfSnapshot, ProfSnapshot};
use melee_script::{Config, Engine, Host, MemHost, RecRead, RecRow, RecStatus, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

struct G {
    log: Rc<RefCell<Vec<String>>>,
}
impl ScriptGame for G {
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        if field == melee_script::game::SI_PRESENT { (slot < 2) as i32 } else { 0 }
    }
    // port 1 has a combo graph with 3 routes; port 2 has none
    fn combo_info(&self, slot: i32, what: i32) -> i32 {
        if slot != 0 {
            return -1;
        }
        match what {
            3 => 3,
            11 => 5,
            0 => 2,
            7 => 4,
            1 => 150,
            8 => 1000,
            2 => 1,
            9 => 12,
            16 => 0,
            17 => 1,
            18 => 2,
            _ => 0,
        }
    }
    fn combo_set_ki(&mut self, slot: i32, ki: i32) -> i32 {
        self.log.borrow_mut().push(format!("ki {slot} {ki}"));
        if slot == 0 { ki } else { -1 }
    }
    fn combo_word(&self, _p: i32, r: i32, w: i32) -> i32 {
        match (r, w) {
            (_, 0) => 0x0000_0007,                     // motion:7
            (0, 1) => (2u32 << 28 | 3) as i32,         // geno state 3
            (1, 1) => (12u32 << 28 | 10) as i32,       // fair
            (_, 1) => 0xFFFF_FFFEu32 as i32,           // auto
            (_, 2) => 0,                               // light
            (_, 3) => 2,                               // forward
            (_, 4) => 1,                               // hit
            (_, 5) => 2,                               // air
            (_, 6) => 3,
            (_, 7) => 9,
            (_, 11) => 250,
            _ => 0,
        }
    }
    fn combo_name(&self, _p: i32, r: i32) -> String {
        format!("route{r}")
    }
    fn combo_state_name(&self, _p: i32, n: i32) -> String {
        format!("state{n}")
    }
    fn rec_start(&mut self, name: &str) -> Result<String, String> {
        self.log.borrow_mut().push(format!("rec_start {name}"));
        Ok(name.to_string())
    }
    fn rec_stop(&mut self) -> Result<i32, String> {
        Ok(77)
    }
    fn rec_trim(&mut self, name: &str, from: i32, to: i32, dst: &str) -> Result<String, String> {
        self.log.borrow_mut().push(format!("trim {name} {from} {to} {dst}"));
        Ok(dst.to_string())
    }
    fn rec_status(&self) -> RecStatus {
        RecStatus { mode: "recording", name: "a".into(), frames: 5, ..Default::default() }
    }
    fn rec_list(&self) -> Vec<RecRow> {
        vec![RecRow {
            name: "take".into(),
            bytes: 1234,
            read: Some(RecRead { frames: 120, marks: 2, complete: true, format: 2, fighters: "mario".into(), ..Default::default() }),
            ok: true,
            why: String::new(),
        }]
    }
}

struct H(MemHost);
impl Host for H {
    fn log(&mut self, m: &str) {
        self.0.log(m)
    }
    fn now_ms(&self) -> f64 {
        0.0
    }
    fn perf_snapshot(&mut self, want: usize) -> PerfSnapshot {
        PerfSnapshot {
            fps: 60.0,
            target: 60,
            surface: [1, 2, 3],
            frames: (0..want.min(3)).map(|i| PerfFrame { total_ms: i as f32, draw_calls: 10 + i as u32, ..Default::default() }).collect(),
            ..Default::default()
        }
    }
    fn prof_snapshot(&mut self) -> ProfSnapshot {
        ProfSnapshot { enabled: true, frames: 9, ..Default::default() }
    }
    fn prof_command(&mut self, a: &str) -> (bool, String) {
        (a == "on", format!("prof {a}"))
    }
}

fn rig() -> (Engine, Rc<RefCell<Vec<String>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let host = Rc::new(RefCell::new(H(MemHost::default())));
    let dir = std::env::temp_dir().join(format!("melee_script_cpr_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    let eng = Engine::new(host, Box::new(G { log: log.clone() }), cfg).unwrap();
    (eng, log)
}

#[test]
fn combo_reads_the_graph() {
    let (eng, _) = rig();
    let none: Option<mlua::Value> = eng.eval("return gd.combo(2)").unwrap();
    assert!(none.is_none());
    let t: mlua::Table = eng.eval("return gd.combo(1)").unwrap();
    assert_eq!(t.get::<f64>("ki").unwrap(), 1.5);
    assert_eq!(t.get::<f64>("ki_max").unwrap(), 10.0);
    assert_eq!(t.get::<String>("last_name").unwrap(), "route1");
    let routes: mlua::Table = t.get("routes").unwrap();
    assert_eq!(routes.raw_len(), 2, "state 0 route hidden without `all`");
    let r1: mlua::Table = routes.get(1).unwrap();
    assert_eq!(r1.get::<String>("name").unwrap(), "route1");
    assert_eq!(r1.get::<String>("from").unwrap(), "motion:7");
    assert_eq!(r1.get::<String>("to").unwrap(), "fair");
    assert_eq!(r1.get::<String>("input").unwrap(), "light");
    assert_eq!(r1.get::<String>("dir").unwrap(), "forward");
    assert_eq!(r1.get::<String>("where").unwrap(), "air");
    assert_eq!(r1.get::<f64>("ki_cost").unwrap(), 2.5);
    let r2: mlua::Table = routes.get(2).unwrap();
    assert_eq!(r2.get::<String>("to").unwrap(), "auto");
    let all: mlua::Table = eng.eval("return gd.combo(1, true)").unwrap();
    let rs: mlua::Table = all.get("routes").unwrap();
    assert_eq!(rs.raw_len(), 3);
    let r0: mlua::Table = rs.get(1).unwrap();
    assert_eq!(r0.get::<String>("to").unwrap(), "geno:state3");
}

#[test]
fn combo_ki_sets_the_meter() {
    let (eng, log) = rig();
    let v: f64 = eng.eval("return gd.combo_ki(1, 2.5)").unwrap();
    assert_eq!(v, 2.5);
    assert_eq!(log.borrow()[0], "ki 0 250");
    let n: Option<mlua::Value> = eng.eval("return gd.combo_ki(2, 1)").unwrap();
    assert!(n.is_none());
}

#[test]
fn perf_and_prof() {
    let (eng, _) = rig();
    let t: mlua::Table = eng.eval("return gd.perf(2)").unwrap();
    assert_eq!(t.get::<f64>("fps").unwrap(), 60.0);
    assert_eq!(t.get::<i64>("surface_selections").unwrap(), 3);
    let fr: mlua::Table = t.get("frames").unwrap();
    assert_eq!(fr.raw_len(), 2);
    assert!(t.get::<mlua::Table>("profiler").unwrap().get::<bool>("enabled").unwrap());
    let cheap: mlua::Table = eng.eval("return gd.perf(1, false)").unwrap();
    assert!(cheap.get::<Option<mlua::Table>>("profiler").unwrap().is_none());
    let (ok, msg): (bool, String) = eng.eval("return gd.prof('on')").unwrap();
    assert!(ok);
    assert_eq!(msg, "prof on");
}

#[test]
fn rec_gates_and_shapes() {
    let (eng, log) = rig();
    let (ok, why): (bool, String) = eng.eval("return gd.rec_start('x')").unwrap();
    assert!(!ok);
    assert_eq!(why, "not in a match");
    eng.st.borrow_mut().match_active = true;
    let (ok, why): (bool, String) = eng.eval("return gd.rec_start('bad name!')").unwrap();
    assert!(!ok);
    assert_eq!(why, "name: letters, digits, _ and - only (40 max)");
    let n: String = eng.eval("return gd.rec_start('take-1')").unwrap();
    assert_eq!(n, "take-1");
    let (ok, frames): (bool, i64) = eng.eval("return gd.rec_stop()").unwrap();
    assert!(ok);
    assert_eq!(frames, 77);
    let d: String = eng.eval("return gd.rec_trim('take', -5, 60)").unwrap();
    assert_eq!(d, "take-cut");
    assert_eq!(log.borrow().last().unwrap(), "trim take 0 60 take-cut");
    let (ok, why): (bool, String) = eng.eval("return gd.rec_keep_last(3)").unwrap();
    assert!(!ok);
    assert_eq!(why, "no recording named (and none made this session)");
    let s: mlua::Table = eng.eval("return gd.rec_status()").unwrap();
    assert_eq!(s.get::<String>("mode").unwrap(), "recording");
    assert_eq!(s.get::<i64>("frames").unwrap(), 5);
    let l: mlua::Table = eng.eval("return gd.rec_list()").unwrap();
    let row: mlua::Table = l.get(1).unwrap();
    assert_eq!(row.get::<f64>("seconds").unwrap(), 2.0);
    assert_eq!(row.get::<String>("fighters").unwrap(), "mario");
    let (ok, why): (bool, String) = eng.eval("return gd.rec_export_slp('a/b')").unwrap();
    assert!(!ok);
    assert_eq!(why, "bad recording name");
}
