//! The rollback driver end to end, in-process: two RollbackDrivers over a toy deterministic
//! simulation, each wired to a Net through NetAdapter (the 11-byte raw-pad payload, pull model,
//! checksums exchanged), over the simulated lossy network. Both peers must end with identical
//! final-frame checksums for every frame, equal to an offline run that knew every input up front,
//! with real rollbacks along the way and no desync reported.

use melee_net::net::{Config, Net};
use melee_net::rollback::NetLog;
use melee_net::sim::{SimEndpoint, SimNet, SimRef};
use melee_net::*;
use std::cell::RefCell;
use std::rc::Rc;

const FIRST: i32 = -123;
const HOST: u8 = 0x01; // slot 0 (port 0)
const GUEST: u8 = 0x04; // slot 2 (port 1)

/// A tiny "game": per-slot position/velocity driven by stick bytes and buttons, f32 math, an RNG.
#[derive(Clone, Default)]
struct Toy {
    pos: [f32; 8],
    vel: [f32; 8],
    rng: u32,
    frame: i32,
}
impl Simulation for Toy {
    type Snapshot = Toy;
    fn save(&self) -> Toy {
        self.clone()
    }
    fn load(&mut self, s: &Toy) {
        *self = s.clone();
    }
    fn advance(&mut self, inputs: &[RbInput; 8]) {
        self.rng = self.rng.wrapping_mul(214013).wrapping_add(2531011);
        for (s, i) in inputs.iter().enumerate() {
            if i.present == 0 {
                continue;
            }
            let ax = i.raw[0] as f32 / 80.0;
            self.vel[s] = self.vel[s] * 0.9 + ax;
            if i.buttons & 0x100 != 0 {
                self.vel[s] += ((self.rng >> 16) & 7) as f32 * 0.25;
            }
            self.pos[s] += self.vel[s];
        }
        self.frame += 1;
    }
    fn checksum(&self) -> u32 {
        let mut b = Vec::new();
        for v in self.pos.iter().chain(self.vel.iter()) {
            b.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        b.extend_from_slice(&self.rng.to_le_bytes());
        b.extend_from_slice(&self.frame.to_le_bytes());
        hash64(&b, 0) as u32
    }
}

/// The input a player "holds" at a frame: changes every few frames so predictions miss.
fn pad(slot: usize, frame: i32) -> RbInput {
    let k = ((frame as u32).wrapping_add(slot as u32 * 977) / 7).wrapping_mul(2654435761) >> 7;
    RbInput {
        raw: [(k & 0xFF) as i8, ((k >> 8) & 0x7F) as i8, 0, 0],
        buttons: if k & 0x3000 == 0x3000 { 0x100 } else { 0 },
        present: 1,
        is_raw: 1,
        ..RbInput::default()
    }
}

type Drv = RollbackDriver<Toy>;

struct Side {
    drv: Rc<RefCell<Drv>>,
    net: Net,
    log: Rc<RefCell<NetLog>>,
    slot: usize,
    finals: Vec<(i32, u32)>,
}

fn side(sim: &SimRef, host: bool, offset: u32) -> Side {
    let (lm, rm) = if host { (HOST, GUEST) } else { (GUEST, HOST) };
    let drv = Rc::new(RefCell::new(RollbackDriver::new(Toy::default(), FIRST, 2, 7, lm, rm)));
    let log = Rc::new(RefCell::new(NetLog::default()));
    let s2 = sim.clone();
    let clock: Rc<dyn Fn() -> u32> = Rc::new(move || s2.borrow().now.wrapping_add(offset));
    let cb = Box::new(NetAdapter::new(drv.clone(), log.clone(), Some(clock)));
    let cfg = Config {
        first_frame: FIRST,
        payload_bytes: NP_PAYLOAD as u8,
        input_delay: 2,
        host_slots: HOST,
        guest_slots: GUEST,
        ..Config::default()
    };
    let net = if host {
        Net::host(&cfg, cb, SimEndpoint::new(sim, 0)).unwrap()
    } else {
        Net::join(&cfg, cb, SimEndpoint::new(sim, 1), SimNet::addr(0)).unwrap()
    };
    Side { drv, net, log, slot: if host { 0 } else { 2 }, finals: Vec::new() }
}

/// One render tick: sample the local pad for current + delay, poll (time sync feeds the driver's
/// wait), run the driver's iterations.
fn tick(s: &mut Side, end: i32) {
    NetAdapter::tick(&mut s.net, &s.drv);
    if !s.net.started() {
        return;
    }
    let (f, d) = {
        let d = s.drv.borrow();
        (d.current_frame(), d.delay())
    };
    if f + d < end {
        s.drv.borrow_mut().submit_local_input(s.slot, f + d, &pad(s.slot, f + d));
    }
    if f < end {
        s.drv.borrow_mut().tick();
    } else {
        s.drv.borrow_mut().resolve_rollback(); // a late correction after the last frame
    }
}

fn run(loss: i32, jitter: u32, base: u32, seed: u64) {
    let frames = 1200;
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
    for _ in 0..(frames * 4) {
        sim.borrow_mut().step();
        tick(&mut a, end);
        tick(&mut b, end);
        for s in [&mut a, &mut b] {
            let d = s.drv.borrow();
            let next = s.finals.last().map_or(FIRST, |x| x.0 + 1);
            for f in next..d.current_frame() {
                let h = d.checksum(f);
                if h == 0 {
                    break;
                }
                s.finals.push((f, h));
            }
        }
        if a.finals.len() >= frames as usize - 1 && b.finals.len() >= frames as usize - 1 {
            break;
        }
    }
    // the reference: every input known, no network
    let mut toy = Toy::default();
    let mut want = Vec::new();
    for f in FIRST..end {
        want.push((f, if toy.checksum() == 0 { 1 } else { toy.checksum() }));
        let mut inp = [RbInput::default(); 8];
        for slot in [0usize, 2] {
            inp[slot] = if f < FIRST + 2 { RbInput { present: 1, ..RbInput::default() } } else { pad(slot, f) };
            inp[slot].present = 1;
        }
        toy.advance(&inp);
    }
    let n = a.finals.len().min(b.finals.len());
    assert!(n >= frames as usize - 1, "only {} final frames (host {}, guest {})", n, a.finals.len(), b.finals.len());
    assert_eq!(a.finals[..n], b.finals[..n], "the peers' final checksums differ");
    assert_eq!(a.finals[..n], want[..n], "the networked run differs from the offline run");
    let (ra, rb) = (a.drv.borrow().rollbacks, b.drv.borrow().rollbacks);
    if loss > 0 || base >= 40 {
        assert!(ra + rb > 0, "no rollback happened: the test is not exercising prediction");
    }
    assert!(a.log.borrow().desyncs.is_empty() && b.log.borrow().desyncs.is_empty(), "transport reported a desync");
    assert_eq!(a.drv.borrow().desyncs + b.drv.borrow().desyncs, 0);
    eprintln!(
        "loss {} jitter {} base {}: rollbacks {}/{}, resimulated {}/{}, stalls {}/{}, waits {}/{}",
        loss,
        jitter,
        base,
        ra,
        rb,
        a.drv.borrow().resimulated,
        b.drv.borrow().resimulated,
        a.drv.borrow().stalls,
        b.drv.borrow().stalls,
        a.drv.borrow().waits,
        b.drv.borrow().waits
    );
}

#[test]
fn rollback_clean_low_latency() {
    run(0, 0, 10, 1);
}
#[test]
fn rollback_high_latency() {
    run(0, 10, 60, 2);
}
#[test]
fn rollback_lossy_jittery() {
    run(20, 60, 40, 3);
}

#[test]
fn np_codec_roundtrip() {
    let i = RbInput { buttons: 0x1F7F, raw: [-128, 127, -1, 5], pad_l: 200, pad_r: 3, pad_a: 4, pad_b: 250, pad_err: -2, ..RbInput::default() };
    let mut b = [0u8; NP_PAYLOAD];
    np_encode(&i, &mut b);
    assert_eq!(b, [0x1F, 0x7F, 0x80, 0x7F, 0xFF, 0x05, 200, 3, 4, 250, 0xFE]);
    let d = np_decode(&b);
    assert_eq!((d.present, d.is_raw, d.confirmed), (1, 1, 1));
    assert!(d.same_pad(&RbInput { present: 1, ..i }));
}
