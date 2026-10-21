//! gw_rollback.c's session (melee_net::session) over the PORTED native game code, in-process:
//! - SyncTest (gw_snap.c's, GGPO's): roll back k = 1..7 frames every frame and resimulate; the
//!   whole-state hash and the curated neutral hash at the start of every resimulated frame must
//!   equal the first pass's. A negative control (state outside the snapshot) must fail it.
//! - The fake network (MELEE_RB_FAKE=<lat>,<jit>,<loss> + MELEE_RB_INPUT=padgen): the final curated
//!   hashes of a session with a late remote port equal an all-local run's, with real rollbacks.
//! - Two sessions over gw_net (the Rust port) on the lossy simnet, live-pad source: both peers'
//!   final checksums agree with each other and with an offline replay of the inputs they sent.
//!
//! The "game" is a small native match driven by the ported fighter code (melee_fighter ftcommon:
//! ground accel from the stick, air drift, fall, jump; melee_runtime HsdRand; the player table's
//! stocks in melee_fighter::stale; the neutral hash from melee_types): the vs_match.rs Sim of the
//! melee binary reads the disc and runs only Entry frames 0-4, which do not read input. The
//! attribute constants below are typed in for the test (not PlCo's), and the stick conversion is a
//! plain deadzone, not the game's pad pipeline.

use melee_fighter::{ftcommon, stale};
use melee_net::net::{Config, Net};
use melee_net::rollback::NetLog;
use melee_net::session::{rb_gen, run_tick, InputSource, RbConfig, RbGame, RbSession, SyncTest, FIRST, SLOTS};
use melee_net::sim::{SimEndpoint, SimNet, SimRef};
use melee_net::*;
use melee_runtime::random::HsdRand;
use melee_types::neutral_hash::{nh_fighter, nh_game};
use melee_types::{Fighter, StaticPlayer, GA_Air, GA_Ground};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

/// The player table is a process global (melee_fighter::stale): one match runs at a time.
static GLOBALS: Mutex<()> = Mutex::new(());

fn bytes_of<T>(v: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }
}

/// The native match state. Snapshots are byte images of it, restored IN PLACE (the fighters stay
/// at their addresses, as MEM1 does in the C), so pointers into it stay valid.
struct Match {
    fighters: Vec<Box<Fighter>>,
    players: Box<[StaticPlayer; stale::PLAYER_NUM_MAX]>,
    rand_seed: u32,
    frame: i32,
    /// Side effects (a "sound" per A press) -- output, not state; gated while resimulating.
    pub sfx: u32,
    pub live: bool,
    /// Negative control: a counter outside the snapshot that changes the physics.
    pub leak: Option<u32>,
}

#[derive(Clone)]
struct Snap {
    fighters: Vec<Vec<u8>>,
    players: Vec<u8>,
    rand_seed: u32,
    frame: i32,
}

fn copy_in<T>(dst: &mut T, src: &[u8]) {
    assert_eq!(src.len(), std::mem::size_of::<T>());
    unsafe { std::ptr::copy_nonoverlapping(src.as_ptr(), dst as *mut T as *mut u8, src.len()) }
}

impl Match {
    fn new() -> Self {
        let mut fighters = Vec::new();
        for i in 0..2 {
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            fp.player_id = i as u8;
            let a = &mut fp.co_attrs;
            a.walk_max_vel = 1.6;
            a.walk_accel_base = 0.1;
            a.ground_friction = 0.08;
            a.ground_max_horizontal_velocity = 3.0;
            a.gravity = 0.13;
            a.terminal_velocity = 2.9;
            a.air_drift_stick_mul = 0.06;
            a.aerial_drift_base = 0.02;
            a.air_drift_max = 1.12;
            a.aerial_friction = 0.02;
            a.air_max_horizontal_velocity = 3.0;
            a.jump_v_initial_velocity = 3.1;
            a.max_jumps = 2;
            fp.coll_data.floor.normal.y = 1.0;
            fp.cur_pos.x = if i == 0 { -30.0 } else { 30.0 };
            fp.facing_dir = if i == 0 { 1.0 } else { -1.0 };
            fp.ground_or_air = GA_Ground;
            fp.shield_health = 60.0;
            fp.x67F = 255;
            fp.x680 = 255;
            fighters.push(fp);
        }
        let mut players: Box<[StaticPlayer; stale::PLAYER_NUM_MAX]> = Box::new(unsafe { std::mem::zeroed() });
        players[0].stocks = 4;
        players[1].stocks = 4;
        let mut r = HsdRand::default();
        r.set_logic_seed(12345);
        Match { fighters, players, rand_seed: r.logic_seed(), frame: FIRST - 1, sfx: 0, live: false, leak: None }
    }

    fn stick(raw: i8) -> f32 {
        let v = raw as f32 / 80.0;
        if (raw as i32).abs() < 23 {
            0.0
        } else {
            v.clamp(-1.0, 1.0)
        }
    }

    fn step_fighter(fp: &mut Fighter, inp: &RbInput, rand: &mut HsdRand, sfx: &mut u32, resim: bool, leak: f32) {
        fp.input.lstick[0].x = Self::stick(inp.raw[0]);
        fp.input.lstick[0].y = Self::stick(inp.raw[1]);
        let b = inp.buttons;
        if fp.ground_or_air == GA_Ground {
            if b & 0x0400 != 0 {
                fp.ground_or_air = GA_Air;
                fp.self_vel.y = fp.co_attrs.jump_v_initial_velocity;
                fp.x1968_jumpsUsed = 1;
            } else {
                ftcommon::calc_ground_accel_accel_to_lstick_x(fp, 0.2, fp.co_attrs.walk_accel_base, fp.co_attrs.walk_max_vel);
                fp.gr_vel += fp.xE4_ground_accel_1 + leak;
                ftcommon::clamp_ground_vel(fp, fp.co_attrs.ground_max_horizontal_velocity);
                ftcommon::set_self_movement_from_grounded_movement(fp);
            }
        } else {
            ftcommon::calc_self_accel_drift(fp);
            fp.self_vel.x += fp.x74_self_accel.x;
            ftcommon::clamp_air_drift(fp);
            ftcommon::fall_basic(fp);
        }
        fp.cur_pos.x += fp.self_vel.x;
        fp.cur_pos.y += fp.self_vel.y;
        if fp.ground_or_air == GA_Air && fp.cur_pos.y <= 0.0 {
            fp.ground_or_air = GA_Ground;
            fp.cur_pos.y = 0.0;
            fp.self_vel.y = 0.0;
            fp.gr_vel = fp.self_vel.x;
            fp.x1968_jumpsUsed = 0;
        }
        if b & 0x0100 != 0 {
            fp.dmg.x1830_percent += rand.randf() * 3.0; // HSD_Randf
            if !resim {
                *sfx += 1;
            }
        }
        if b & 0x0020 != 0 {
            fp.shield_health -= 0.28;
        }
        if fp.cur_pos.x.abs() > 160.0 {
            let id = fp.player_id as i32;
            stale::set_stocks(id, stale::stocks(id) - 1);
            fp.cur_pos.x = 0.0;
            fp.self_vel.x = 0.0;
            fp.gr_vel = 0.0;
            fp.dmg.x1830_percent = 0.0;
        }
        fp.x3E4_fighterCmdScript.frame_count += 1.0;
    }
}

impl RbGame for Match {
    type Snapshot = Snap;
    fn frame(&self) -> i32 {
        self.frame
    }
    fn live(&self) -> bool {
        self.live
    }
    fn port_human(&self, p: usize) -> bool {
        p < 2
    }
    fn peek_input(&self, slot: usize, _frame: i32) -> Option<RbInput> {
        (slot == 0 || slot == 2).then(|| RbInput { present: 1, is_raw: 1, ..RbInput::default() })
    }
    fn save(&self) -> Snap {
        Snap {
            fighters: self.fighters.iter().map(|f| bytes_of(&**f).to_vec()).collect(),
            players: bytes_of(&*self.players).to_vec(),
            rand_seed: self.rand_seed,
            frame: self.frame,
        }
    }
    fn load(&mut self, s: &Snap) {
        for (f, b) in self.fighters.iter_mut().zip(&s.fighters) {
            copy_in(&mut **f, b);
        }
        copy_in(&mut *self.players, &s.players);
        self.rand_seed = s.rand_seed;
        self.frame = s.frame;
    }
    fn state_hash(&self) -> u32 {
        let mut h = 0u64;
        for f in &self.fighters {
            h = hash64(bytes_of(&**f), h);
        }
        h = hash64(bytes_of(&*self.players), h);
        h = hash64(&self.rand_seed.to_le_bytes(), h);
        h = hash64(&self.frame.to_le_bytes(), h);
        (h ^ (h >> 32)) as u32
    }
    fn game_hash(&self) -> u32 {
        let mut w = [None; 12];
        for (i, f) in self.fighters.iter().enumerate() {
            w[i * 2] = Some(nh_fighter(f, (i * 2) as i32, self.players[i].stocks as i32, false));
        }
        nh_game(self.rand_seed, &w, 0, 0)
    }
    fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], resim: bool) {
        // the player table is the ported global: install this match's, run, take it back
        unsafe { stale::PLAYER_SLOTS = *self.players };
        let mut rand = HsdRand::default();
        rand.set_logic_seed(self.rand_seed);
        let leak = match &mut self.leak {
            Some(n) => {
                *n += 1;
                (*n % 3) as f32 * 0.001
            }
            None => 0.0,
        };
        for (i, fp) in self.fighters.iter_mut().enumerate() {
            let inp = inputs[i * 2].unwrap_or(RbInput { present: 1, is_raw: 1, ..RbInput::default() });
            Match::step_fighter(fp, &inp, &mut rand, &mut self.sfx, resim, leak);
        }
        self.rand_seed = rand.logic_seed();
        unsafe { *self.players = stale::PLAYER_SLOTS };
        self.frame += 1;
    }
}

fn gen_inputs(seed: u32) -> impl FnMut(i32) -> [Option<RbInput>; SLOTS] {
    move |f| {
        let mut a = [None; SLOTS];
        a[0] = Some(rb_gen(seed, 0, f));
        a[2] = Some(rb_gen(seed, 2, f));
        a
    }
}

#[test]
fn synctest_ported_sim() {
    let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
    // the reference pass, no rollback: the curated hashes at the start of every frame
    let mut m = Match::new();
    let mut want = Vec::new();
    let mut inp = gen_inputs(777);
    for f in FIRST..FIRST + 900 {
        want.push(m.game_hash());
        let i = inp(f);
        m.run_frame(&i, false);
    }
    let sfx_ref = m.sfx;
    let moved = m.fighters.iter().any(|f| f.cur_pos.x != -30.0 && f.cur_pos.x != 30.0);
    assert!(moved && m.fighters[0].dmg.x1830_percent > 0.0, "the inputs do not drive the sim");
    for k in 1..=7 {
        let mut m = Match::new();
        let mut st: SyncTest<Match> = SyncTest::new(k);
        let mut inp = gen_inputs(777);
        let mut got = Vec::new();
        for _ in 0..900 {
            got.push(m.game_hash());
            st.tick(&mut m, &mut inp);
        }
        assert!(st.mismatches.is_empty(), "k={k}: {} mismatches, first {:?}", st.mismatches.len(), st.mismatches[0]);
        assert!(st.passes > 890 && st.checked as i32 >= st.passes as i32 * k, "k={k}: {} passes {} checks", st.passes, st.checked);
        assert_eq!(got, want, "k={k}: the resimulated timeline differs from the reference");
        assert_eq!(m.sfx, sfx_ref, "k={k}: side effects not gated while resimulating");
    }
    // negative control: state outside the snapshot must be caught
    let mut m = Match::new();
    m.leak = Some(0);
    let mut st: SyncTest<Match> = SyncTest::new(2);
    let mut inp = gen_inputs(777);
    for _ in 0..300 {
        st.tick(&mut m, &mut inp);
    }
    assert!(!st.mismatches.is_empty(), "SyncTest did not catch unsnapshotted state");
}

fn fake_run(cfg: RbConfig, ticks: usize) -> (Vec<(i32, u32)>, melee_net::session::RbStats, u32) {
    let mut m = Match::new();
    let mut s: RbSession<Match> = RbSession::new(RbConfig { hash_log: true, ..cfg });
    s.scene_begin(2);
    for _ in 0..ticks {
        run_tick(&mut s, &mut m, None);
    }
    (s.hash_log.clone(), s.stats, m.sfx)
}

#[test]
fn fake_network_matches_all_local() {
    let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
    let base = RbConfig { src: InputSource::PadGen, gen_seed: 4242, ..RbConfig::default() };
    let (want, ws, _) = fake_run(RbConfig { remote_mask: 0, ..base.clone() }, 1500);
    assert_eq!(ws.rollbacks, 0);
    assert!(want.len() > 1400, "{} final frames", want.len());
    for (lat, jit, loss) in [(1, 0, 0), (4, 2, 0), (6, 3, 15), (9, 0, 5)] {
        let (got, st, _) = fake_run(RbConfig { fake_lat: lat, fake_jit: jit, fake_loss: loss, ..base.clone() }, 1700);
        let n = got.len().min(want.len());
        assert!(n > 1200, "lat {lat}: only {n} final frames");
        assert_eq!(got[..n], want[..n], "lat {lat} jit {jit} loss {loss}: final hashes differ from the all-local run");
        if lat > 2 {
            assert!(st.rollbacks > 0, "lat {lat}: no rollback"); // latency within the input delay costs nothing
        }
        assert_eq!(st.desyncs, 0);
        eprintln!("fake lat {lat} jit {jit} loss {loss}: {n} final, {:?}", st);
    }
}

// ---- two peers over gw_net on the simnet --------------------------------------------------------

type Sess = RbSession<Match>;

struct Side {
    sess: Rc<RefCell<Sess>>,
    game: Match,
    net: Net,
    log: Rc<RefCell<NetLog>>,
    port: usize,
    finals: Vec<(i32, u32)>,
    sent: Vec<(i32, RbInput)>,
}

fn pad(port: usize, frame: i32) -> RbInput {
    rb_gen(99, port * 2, frame)
}

fn side(sim: &SimRef, host: bool, offset: u32) -> Side {
    let port = if host { 0 } else { 1 };
    let cfg = RbConfig { net: true, local_port: port, delay: 2, ..RbConfig::default() };
    let sess = Rc::new(RefCell::new(Sess::new(cfg)));
    sess.borrow_mut().scene_begin(2);
    let log = Rc::new(RefCell::new(NetLog::default()));
    let s2 = sim.clone();
    let clock: Rc<dyn Fn() -> u32> = Rc::new(move || s2.borrow().now.wrapping_add(offset));
    let cb = Box::new(NetAdapter::new(sess.clone(), log.clone(), Some(clock)));
    let ncfg = Config {
        first_frame: FIRST,
        payload_bytes: NP_PAYLOAD as u8,
        input_delay: 2,
        host_slots: 0x01,
        guest_slots: 0x04,
        ..Config::default()
    };
    let net = if host {
        Net::host(&ncfg, cb, SimEndpoint::new(sim, 0)).unwrap()
    } else {
        Net::join(&ncfg, cb, SimEndpoint::new(sim, 1), SimNet::addr(0)).unwrap()
    };
    let mut game = Match::new();
    game.live = true;
    Side { sess, game, net, log, port, finals: Vec::new(), sent: Vec::new() }
}

fn tick(s: &mut Side, end: i32) {
    NetAdapter::tick(&mut s.net, &s.sess);
    if !s.net.started() {
        return;
    }
    let mut sess = s.sess.borrow_mut();
    let f = sess.current_frame();
    let d = sess.cfg.delay;
    // the pad poll (PADRead -> gw_RB_PadLatch), channel 0 = this machine's controller
    let p = pad(s.port, f + d);
    sess.pad_latch(0, p.buttons as i32, p.raw[0] as i32, p.raw[1] as i32, p.raw[2] as i32, p.raw[3] as i32, p.pad_l as i32, p.pad_r as i32, 0, 0, 0);
    // past the end too: a late correction still resimulates (the extra new frames are harmless)
    let _ = end;
    run_tick(&mut sess, &mut s.game, None);
    let next = s.finals.last().map_or(FIRST, |x| x.0 + 1);
    for f in next..sess.current_frame() {
        let h = sess.checksum(f);
        if h == 0 {
            break;
        }
        s.finals.push((f, h));
    }
    let nsent = s.sent.last().map_or(FIRST, |x| x.0 + 1);
    for f in nsent..sess.current_frame() + d {
        match sess.local_input_for_send(s.port * 2, f) {
            Some(i) => s.sent.push((f, i)),
            None => break,
        }
    }
}

#[test]
fn two_peers_over_gw_net() {
    let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
    for (loss, jitter, base, seed) in [(0, 0, 10, 1u64), (0, 10, 60, 2), (20, 60, 40, 3)] {
        let frames = 1000;
        let end = FIRST + frames;
        let sim = SimNet::shared(seed);
        {
            let mut s = sim.borrow_mut();
            s.loss_pct = loss;
            s.jitter_ms = jitter;
            s.base_ms = base;
        }
        let mut a = side(&sim, true, 0);
        let mut b = side(&sim, false, 0x7000_0000);
        for _ in 0..(frames * 5) {
            sim.borrow_mut().step();
            tick(&mut a, end);
            tick(&mut b, end);
            if a.finals.len() >= frames as usize - 1 && b.finals.len() >= frames as usize - 1 {
                break;
            }
        }
        // offline: the inputs each side sent, every one known up front
        let mut m = Match::new();
        let mut want = Vec::new();
        for f in FIRST..end {
            want.push((f, m.game_hash() | 1));
            let mut i = [None; SLOTS];
            for (slot, s) in [(0usize, &a), (2, &b)] {
                i[slot] = s.sent.iter().find(|e| e.0 == f).map(|e| e.1);
            }
            m.run_frame(&i, false);
        }
        let n = a.finals.len().min(b.finals.len());
        assert!(n >= frames as usize - 1, "loss {loss}: only {n} final frames ({} / {})", a.finals.len(), b.finals.len());
        assert_eq!(a.finals[..n], b.finals[..n], "loss {loss}: the peers' final checksums differ");
        assert_eq!(a.finals[..n], want[..n], "loss {loss}: the networked run differs from the offline run");
        let (sa, sb) = (a.sess.borrow().stats, b.sess.borrow().stats);
        if loss > 0 || base >= 40 {
            assert!(sa.rollbacks + sb.rollbacks > 0, "loss {loss}: no rollback");
        }
        assert!(a.log.borrow().desyncs.is_empty() && b.log.borrow().desyncs.is_empty(), "transport desync");
        assert_eq!(sa.desyncs + sb.desyncs, 0);
        eprintln!("net loss {loss} jitter {jitter} base {base}: host {:?} guest {:?}", sa, sb);
    }
}
