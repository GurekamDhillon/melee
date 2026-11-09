use super::*;

#[test]
fn cmd_round_trip() {
    let c = LabCmd::new(OP_PLAY_SET, 1, 0x0102, 0xDEAD_BEEF, 7, 0x9876_5432);
    let b = c.to_bytes();
    assert_eq!(b.len(), LAB_CMD_BYTES);
    assert_eq!(LabCmd::from_bytes(&b), Some(c));
    assert_eq!(LabCmd::from_bytes(&[0u8; 16]), None, "op 0 is not a command");
    assert_eq!(LabCmd::from_bytes(&b[..15]), None);
    // the drill / trainer / vary / dummy-option commands ride the lockstep stream; past the last op is not a command
    for op in [OP_DRILL, OP_ADAPT, OP_LOAD_VARY, OP_DUMMY_OPT] {
        let c = LabCmd::new(op, 1, 3, 4, 5, 0);
        assert_eq!(LabCmd::from_bytes(&c.to_bytes()), Some(c));
    }
    let mut x = LabCmd::new(OP_LAST, 0, 0, 0, 0, 0).to_bytes();
    x[0] = OP_LAST + 1;
    assert_eq!(LabCmd::from_bytes(&x), None);
}

#[test]
fn push_is_session_and_state_gated() {
    let mut s = LabShared::default();
    let c = LabCmd::new(OP_SAVE, 0, 3, 0, 0, 0);
    assert_eq!(s.push(c), GD_ESTATE, "no lab match running");
    s.active = true;
    s.apply_frame = 40;
    assert_eq!(s.push(c), 40);
    for _ in 0..LAB_CMDS_PER_FRAME - 1 {
        assert_eq!(s.push(c), 40);
    }
    assert_eq!(s.push(c), 41, "a ninth command waits for the next frame");
    s.set_session(Session::Netplay);
    assert!(s.outbox.is_empty(), "a session change drops queued commands");
    assert_eq!(s.push(c), GD_EOFFLINE);
    s.set_session(Session::LabNetplay);
    assert_eq!(s.push(c), 40);
    s.set_session(Session::Synctest);
    assert_eq!(s.push(c), GD_EOFFLINE);
}

#[test]
fn rec_handles_carry_the_issuer() {
    let mut s = LabShared::default();
    let a = s.new_rec();
    s.issuer = 1;
    let b = s.new_rec();
    assert_ne!(a, b);
    assert_eq!(a >> 6, 0);
    assert_eq!(b >> 6, 1);
    assert_ne!(a & 63, 0);
}

#[test]
fn presentation_rng_is_a_function_of_seed_and_frame() {
    let mut a = LabShared::default();
    let mut b = LabShared::default();
    a.match_frame = 100;
    b.match_frame = 100;
    let va: Vec<u32> = (0..4).map(|_| a.rng()).collect();
    let vb: Vec<u32> = (0..4).map(|_| b.rng()).collect();
    assert_eq!(va, vb);
    assert_ne!(va[0], va[1]);
    a.match_frame = 101;
    assert_ne!(a.rng(), va[0]);
}

#[test]
fn react_param_packs() {
    let p = ReactParam { delay: 3, strength: 80, chance: 25, rec: 0x41 };
    assert_eq!(ReactParam::from_u32(p.to_u32()), p);
}

#[test]
fn event_history_is_bounded() {
    let mut s = LabShared::default();
    for f in 0..2000 {
        s.match_frame = f;
        s.publish_events(&[LabEvent { kind: EV_HIT, port: 0, frame: f, a: 1, b: 2 }]);
    }
    assert_eq!(s.events.len(), 1);
    assert!(s.history.len() as i32 <= EVENT_HISTORY + 1);
    assert_eq!(s.history.front().unwrap().frame, 1999 - EVENT_HISTORY + 0);
}

#[test]
fn lookahead_state_is_dropped_with_the_session() {
    let mut s = LabShared::default();
    assert_eq!(s.look_cfg, LOOK_AUTO, "auto on, live off by default");
    s.look_req = Some((LOOK_HIT, 1));
    s.look[1] = LS_DONE;
    s.set_session(Session::Netplay);
    assert!(s.look_req.is_none() && s.look[1] == LS_NONE);
    assert_eq!(LOOK_I32, LOOK_HEAD + LOOK_RUNS * LOOK_RUN_I32);
}

// ---- gd_geno_* (gd-lab-v1.1)

const GENO_WAT: &str = r#"(module
  (import "gd_lab" "gd_geno_profile" (func $profile (param i32) (result i32)))
  (import "gd_lab" "gd_geno_state_count" (func $count (param i32) (result i32)))
  (import "gd_lab" "gd_geno_state_name" (func $state (param i32 i32 i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_define_name" (func $define (param i32 i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_combo_info" (func $info (param i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_combo_route_name" (func $route (param i32 i32 i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_fly_get" (func $flyget (param i32) (result i32)))
  (import "gd_lab" "gd_geno_fly_state" (func $flystate (param i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_result" (func $result (param i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_fly" (func $fly (param i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_teleport" (func $tp (param i32 f32 f32) (result i32)))
  (import "gd_lab" "gd_geno_fly_attack" (func $atk (param i32 i32 i32 f32 i32 i32 i32 i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_fly_clear" (func $clear (param i32) (result i32)))
  (import "gd_lab" "gd_geno_combo_ki" (func $ki (param i32 i32) (result i32)))
  (import "gd_lab" "gd_geno_reload" (func $reload (result i32)))
  (import "gd_lab" "gd_controls_count" (func $ccount (param i32 i32) (result i32)))
  (import "gd_lab" "gd_controls_name" (func $cname (param i32 i32 i32 i32 i32) (result i32)))
  (import "gd_lab" "gd_controls_get" (func $cget (param i32) (result i32)))
  (import "gd_lab" "gd_controls_set" (func $cset (param i32 i32) (result i32)))
  (memory (export "memory") 1 1)
  (func (export "profile") (param i32) (result i32) (call $profile (local.get 0)))
  (func (export "count") (param i32) (result i32) (call $count (local.get 0)))
  (func (export "state") (param i32 i32 i32 i32) (result i32) (call $state (local.get 0) (local.get 1) (local.get 2) (local.get 3)))
  (func (export "define") (param i32 i32 i32) (result i32) (call $define (local.get 0) (local.get 1) (local.get 2)))
  (func (export "info") (param i32 i32) (result i32) (call $info (local.get 0) (local.get 1)))
  (func (export "route") (param i32 i32 i32 i32) (result i32) (call $route (local.get 0) (local.get 1) (local.get 2) (local.get 3)))
  (func (export "flyget") (param i32) (result i32) (call $flyget (local.get 0)))
  (func (export "flystate") (param i32 i32) (result i32) (call $flystate (local.get 0) (local.get 1)))
  (func (export "result") (param i32 i32) (result i32) (call $result (local.get 0) (local.get 1)))
  (func (export "fly") (param i32 i32) (result i32) (call $fly (local.get 0) (local.get 1)))
  (func (export "tp") (param i32 f32 f32) (result i32) (call $tp (local.get 0) (local.get 1) (local.get 2)))
  (func (export "atk") (param i32 i32 i32 f32 i32 i32 i32 i32 i32) (result i32)
    (call $atk (local.get 0) (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get 5) (local.get 6) (local.get 7) (local.get 8)))
  (func (export "clear") (param i32) (result i32) (call $clear (local.get 0)))
  (func (export "ki") (param i32 i32) (result i32) (call $ki (local.get 0) (local.get 1)))
  (func (export "reload") (result i32) (call $reload))
  (func (export "ccount") (param i32 i32) (result i32) (call $ccount (local.get 0) (local.get 1)))
  (func (export "cname") (param i32 i32 i32 i32 i32) (result i32) (call $cname (local.get 0) (local.get 1) (local.get 2) (local.get 3) (local.get 4)))
  (func (export "cget") (param i32) (result i32) (call $cget (local.get 0)))
  (func (export "cset") (param i32 i32) (result i32) (call $cset (local.get 0) (local.get 1)))
)"#;

struct GenoMod {
    store: wasmi::Store<PresData>,
    inst: wasmi::Instance,
    sh: Rc<RefCell<LabShared>>,
}

impl GenoMod {
    fn new(features: u32) -> GenoMod {
        let engine = wasmi::Engine::new(&crate::engine_config());
        let mut l = wasmi::Linker::new(&engine);
        link(&mut l).unwrap();
        let module = wasmi::Module::new(&engine, wat::parse_str(GENO_WAT).unwrap()).unwrap();
        let sh = LabShared::new();
        let data = PresData { features, ext: Some(Box::new(sh.clone())), ..Default::default() };
        let mut store = wasmi::Store::new(&engine, data);
        store.set_fuel(1_000_000).unwrap();
        let inst = l.instantiate_and_start(&mut store, &module).unwrap();
        let mem = inst.get_memory(&store, "memory").unwrap();
        store.data_mut().mem = Some(mem);
        GenoMod { store, inst, sh }
    }
    fn call<P: wasmi::WasmParams, R: wasmi::WasmResults>(&mut self, name: &str, p: P) -> R {
        let f = self.inst.get_typed_func::<P, R>(&self.store, name).unwrap();
        f.call(&mut self.store, p).unwrap()
    }
    fn mem(&self, at: usize, n: usize) -> Vec<u8> {
        self.inst.get_memory(&self.store, "memory").unwrap().data(&self.store)[at..at + n].to_vec()
    }
}

fn geno_view() -> GenoView {
    let mut v = GenoView::default();
    v.profile = [1, -1, -1, -1];
    v.states = vec![vec![], vec!["Light2".into(), "Heavy".into()]];
    v.routes = vec![vec![], vec!["L1>H".into()]];
    v.defines = vec![None; 36];
    v.defines[35] = Some("Combo T".into());
    v.combo[0][1] = 2000;
    v.combo[0][16] = 2;
    v.fly_mode[0] = 1;
    v.fly[0][2] = 7;
    v
}

const ALL: u32 = FEATURE_LAB | FEATURE_LAB_STATE | FEATURE_LAB_CPU;
type Atk = (i32, i32, i32, f32, i32, i32, i32, i32, i32);
type Str = (i32, i32, i32, i32);

#[test]
fn geno_queries_copy_what_is_published() {
    let mut m = GenoMod::new(FEATURE_LAB);
    m.sh.borrow_mut().geno = geno_view();
    assert_eq!(m.call::<i32, i32>("profile", 0), 1);
    assert_eq!(m.call::<i32, i32>("profile", 1), -1, "no profile");
    assert_eq!(m.call::<i32, i32>("profile", 4), GD_EINVAL);
    let counts: Vec<i32> = [1, 0, -1, 9].iter().map(|&p| m.call::<i32, i32>("count", p)).collect();
    assert_eq!(counts, [2, 0, 0, 0]);
    assert_eq!(m.call::<Str, i32>("state", (1, 1, 100, 64)), 5);
    assert_eq!(m.mem(100, 5), b"Heavy");
    assert_eq!(m.call::<Str, i32>("state", (1, 0, 200, 3)), 3, "truncated to cap");
    assert_eq!(m.mem(200, 4), b"Lig\0", "no NUL written, nothing past cap");
    assert_eq!(m.call::<Str, i32>("state", (1, 2, 100, 64)), GD_EINVAL, "no such state");
    assert_eq!(m.call::<Str, i32>("state", (-1, 0, 100, 64)), GD_EINVAL);
    assert_eq!(m.call::<Str, i32>("state", (1, 1, 65534, 64)), GD_EINVAL, "out of the memory");
    assert_eq!(m.call::<Str, i32>("state", (1, 1, -1, 64)), GD_EINVAL, "address wraps");
    assert_eq!(m.call::<(i32, i32, i32), i32>("define", (35, 300, 32)), 7);
    assert_eq!(m.mem(300, 7), b"Combo T");
    assert_eq!(m.call::<(i32, i32, i32), i32>("define", (34, 300, 32)), GD_EINVAL);
    assert_eq!(m.call::<(i32, i32, i32), i32>("define", (99, 300, 32)), GD_EINVAL);
    assert_eq!(m.call::<Str, i32>("route", (1, 0, 400, 32)), 4);
    assert_eq!(m.mem(400, 4), b"L1>H");
    assert_eq!(m.call::<Str, i32>("route", (1, 1, 400, 32)), GD_EINVAL);
    let info: Vec<i32> = [(0, 1), (0, 16), (1, 1), (0, GENO_COMBO_WHATS as i32), (0, -1), (5, 0)].iter().map(|&a| m.call::<(i32, i32), i32>("info", a)).collect();
    assert_eq!(info, [2000, 2, -1, GD_EINVAL, GD_EINVAL, GD_EINVAL]);
    assert_eq!((m.call::<i32, i32>("flyget", 0), m.call::<i32, i32>("flyget", 2)), (1, -1));
    assert_eq!((m.call::<(i32, i32), i32>("flystate", (0, 2)), m.call::<(i32, i32), i32>("flystate", (0, 14))), (7, GD_EINVAL));
    // the result: none yet, then header + text, truncated to cap
    assert_eq!(m.call::<(i32, i32), i32>("result", (500, 64)), GD_ESTATE);
    m.sh.borrow_mut().geno.result = GenoResult { seq: 3, op: OP_GENO_RELOAD, port: 0, rc: -1, text: "kept".into() };
    assert_eq!(m.call::<(i32, i32), i32>("result", (500, 64)), 20);
    let r = m.mem(500, 20);
    assert_eq!(&r[..16], &[3, 0, 0, 0, OP_GENO_RELOAD, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(&r[16..], b"kept");
    assert_eq!(m.call::<(i32, i32), i32>("result", (500, 18)), 18);
    assert_eq!(m.call::<(i32, i32), i32>("result", (65530, 64)), GD_EINVAL);
    // outside an allowed session every query says so
    m.sh.borrow_mut().set_session(Session::Netplay);
    assert_eq!(m.call::<i32, i32>("profile", 0), GD_EOFFLINE);
    // without the LAB bit: refused
    let mut n = GenoMod::new(FEATURE_UI_DRAW);
    assert_eq!(n.call::<i32, i32>("profile", 0), GD_ECAP);
}

#[test]
fn geno_commands_encode_and_gate() {
    let mut m = GenoMod::new(ALL);
    assert_eq!(m.call::<(i32, i32), i32>("fly", (1, 1)), GD_ESTATE, "no lab match running");
    {
        let mut s = m.sh.borrow_mut();
        s.active = true;
        s.apply_frame = 10;
    }
    assert_eq!(m.call::<(i32, i32), i32>("fly", (1, 2)), 10);
    assert_eq!(m.call::<(i32, f32, f32), i32>("tp", (0, 12.5, -40.0)), 10);
    assert_eq!(m.call::<Atk, i32>("atk", (2, 3, 7, 6.5, 4, 20, 50, 70, 361)), 10);
    assert_eq!(m.call::<Atk, i32>("atk", (2, 0, 99, -1.0, 9999, 0, 0, 0, 999)), 10, "off: the rest is not checked");
    assert_eq!(m.call::<i32, i32>("clear", 3), 10);
    assert_eq!(m.call::<(i32, i32), i32>("ki", (0, 2500)), 10);
    assert_eq!(m.call::<(), i32>("reload", ()), 10);
    let out = m.sh.borrow().outbox.clone();
    assert_eq!(out, vec![
        LabCmd::new(OP_GENO_FLY, 1, 2, 0, 0, 0),
        LabCmd::new(OP_GENO_TELEPORT, 0, 0, 12.5f32.to_bits(), (-40.0f32).to_bits(), 0),
        LabCmd::new(OP_GENO_FLY_ATTACK, 2, 3 | 7 << 2 | 361 << 7, 6.5f32.to_bits(), 4 | 20 << 16, 50 | 70 << 16),
        LabCmd::new(OP_GENO_FLY_ATTACK, 2, 0, 0, 0, 0),
        LabCmd::new(OP_GENO_FLY_CLEAR, 3, 0, 0, 0, 0),
        LabCmd::new(OP_GENO_KI, 0, 0, 2500, 0, 0),
        LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0),
    ]);
    // the wire round trip and the apply side's decoding
    for c in &out {
        assert_eq!(LabCmd::from_bytes(&c.to_bytes()), Some(*c));
        assert!(is_geno_op(c.op));
    }
    assert_eq!(geno_attack_args(&out[2]), (3, 7, 6.5f32.to_bits() as i32, 4 | 20 << 16, 50 | 70 << 16, 361));
    let mut b = out[0].to_bytes();
    b[0] = OP_LAST + 1;
    assert_eq!(LabCmd::from_bytes(&b), None, "past the last opcode");
    // bounds (the ranges GenoFly_AttackConfig / the C's fly checks accept)
    let n0 = m.sh.borrow().outbox.len();
    for a in [(4, 1), (0, 3), (0, -1)] {
        assert_eq!(m.call::<(i32, i32), i32>("fly", a), GD_EINVAL, "{a:?}");
    }
    for a in [(0, f32::NAN, 0.0), (0, 0.0, 49001.0), (0, f32::INFINITY, 0.0), (4, 0.0, 0.0)] {
        assert_eq!(m.call::<(i32, f32, f32), i32>("tp", a), GD_EINVAL, "{a:?}");
    }
    for (a, why) in [
        ((0, 1, 0, 6.0, 0, 0, 0, 0, 0), "damage 0"),
        ((0, 1, 31, 6.0, 0, 0, 0, 0, 0), "damage 31"),
        ((0, 1, 3, 0.5, 0, 0, 0, 0, 0), "radius"),
        ((0, 1, 3, f32::NAN, 0, 0, 0, 0, 0), "radius NaN"),
        ((0, 1, 3, 6.0, 601, 0, 0, 0, 0), "active"),
        ((0, 1, 3, 6.0, 0, 3001, 0, 0, 0), "gap"),
        ((0, 1, 3, 6.0, 0, 0, 2001, 0, 0), "kb"),
        ((0, 1, 3, 6.0, 0, 0, 0, 2001, 0), "kbg"),
        ((0, 1, 3, 6.0, 0, 0, 0, 0, 362), "angle"),
        ((0, 4, 3, 6.0, 0, 0, 0, 0, 0), "flags"),
        ((4, 1, 3, 6.0, 0, 0, 0, 0, 0), "port"),
    ] {
        assert_eq!(m.call::<Atk, i32>("atk", a), GD_EINVAL, "{why}");
    }
    assert_eq!(m.call::<(i32, i32), i32>("ki", (0, -1)), GD_EINVAL);
    assert_eq!(m.call::<i32, i32>("clear", 4), GD_EINVAL);
    assert_eq!(m.sh.borrow().outbox.len(), n0, "nothing queued by a refused call");
    // feature bits: fly / ki need LAB_CPU, reload LAB_STATE
    let mut c = GenoMod::new(FEATURE_LAB | FEATURE_LAB_STATE);
    c.sh.borrow_mut().active = true;
    assert_eq!(c.call::<(i32, i32), i32>("fly", (0, 1)), GD_ECAP);
    assert_eq!(c.call::<(i32, i32), i32>("ki", (0, 1)), GD_ECAP);
    assert_eq!(c.call::<(), i32>("reload", ()), 0);
    let mut d = GenoMod::new(FEATURE_LAB | FEATURE_LAB_CPU);
    d.sh.borrow_mut().active = true;
    assert_eq!(d.call::<(), i32>("reload", ()), GD_ECAP);
}

#[test]
fn geno_commands_are_offline_only() {
    let mut m = GenoMod::new(ALL);
    m.sh.borrow_mut().set_session(Session::LabNetplay);
    m.sh.borrow_mut().active = true;
    assert_eq!(m.call::<(i32, i32), i32>("fly", (0, 1)), GD_EOFFLINE, "the C refuses Geno writes online (gs_require_offline)");
    assert_eq!(m.call::<(), i32>("reload", ()), GD_EOFFLINE);
    assert_eq!(m.call::<(i32, i32), i32>("ki", (0, 1)), GD_EOFFLINE);
    assert!(m.sh.borrow().outbox.is_empty());
    // queries still answer online (read-only)
    m.sh.borrow_mut().geno.profile[0] = 4;
    assert_eq!(m.call::<i32, i32>("profile", 0), 4);
    assert_eq!(LAB_CMD_VERSION, 1, "no Geno op ever rides the lockstep stream: the command version stays");
}

/// gd_controls_*: what the play flow published, and saves queued for it (never a lab command: a preference for the next
/// match, offline and online alike).
#[test]
fn controls_queries_and_saves() {
    let mut m = GenoMod::new(FEATURE_LAB);
    {
        let mut s = m.sh.borrow_mut();
        s.controls[0] = ControlsView { ck: 127, choice: 0x105, layouts: vec!["Swap attacks".into(), "Standard".into()], extras: vec!["Y boost".into()] };
        s.controls[1] = ControlsView { ck: 2, choice: 1, layouts: vec![], extras: vec![] };
        s.controls[2].ck = -1;
    }
    let c: Vec<i32> = [(0, 0), (0, 1), (1, 0), (0, 2), (4, 0)].iter().map(|&a| m.call::<(i32, i32), i32>("ccount", a)).collect();
    assert_eq!(c, [2, 1, 0, GD_EINVAL, GD_EINVAL]);
    assert_eq!(m.call::<(i32, i32, i32, i32, i32), i32>("cname", (0, 0, 1, 100, 32)), 8);
    assert_eq!(m.mem(100, 8), b"Standard");
    assert_eq!(m.call::<(i32, i32, i32, i32, i32), i32>("cname", (0, 1, 0, 100, 3)), 3, "truncated to cap");
    assert_eq!(m.call::<(i32, i32, i32, i32, i32), i32>("cname", (0, 0, 2, 100, 32)), GD_EINVAL, "no such layout");
    assert_eq!(m.call::<(i32, i32, i32, i32, i32), i32>("cname", (1, 1, 0, 100, 32)), GD_EINVAL);
    let g: Vec<i32> = [0, 1, 2, 5].iter().map(|&p| m.call::<i32, i32>("cget", p)).collect();
    assert_eq!(g, [0x105, 1, -1, GD_EINVAL]);
    // a save is queued for the play flow; a bad word, port or a port with no fighter is refused
    assert_eq!(m.call::<(i32, i32), i32>("cset", (0, 0x101)), GD_OK);
    assert_eq!(m.call::<(i32, i32), i32>("cset", (0, 2)), GD_EINVAL, "bit 1 is reserved");
    assert_eq!(m.call::<(i32, i32), i32>("cset", (2, 1)), GD_EINVAL, "no fighter");
    assert_eq!(m.call::<(i32, i32), i32>("cset", (4, 1)), GD_EINVAL);
    assert_eq!(m.sh.borrow().controls_req, vec![(0u32, 0x101u32)]);
    assert!(m.sh.borrow().outbox.is_empty(), "never a lab command");
    let mut n = GenoMod::new(FEATURE_UI_DRAW);
    assert_eq!(n.call::<i32, i32>("cget", 0), GD_ECAP);
}
