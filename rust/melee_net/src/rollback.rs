//! The rollback session interface and a generic frame/snapshot driver.
//!
//! - [`RollbackSession`] is gw_rollback.h's "NETWORK-FACING INTERFACE": the only calls the
//!   transport makes into the session.
//! - [`NetAdapter`] is gw_netplay.c's transport glue (np_cb_remote / np_cb_local /
//!   np_cb_checksum + the per-tick poll / time-sync actuator), as a [`Callbacks`] object.
//! - [`RollbackDriver`] is a game-agnostic session over any [`Simulation`] that can save and load
//!   snapshots: input delay, prediction (repeat the last confirmed input), rollback to the
//!   earliest mispredicted frame, a maximum prediction window (stall beyond it), time-sync waits
//!   and final-frame checksums. It is NOT a port of gw_rollback.c (that one is wired into the
//!   scene loop, gw_snap.c and the replay module and moves with the game crates); it implements
//!   the same contract so the transport can be tested end to end in-process, and is the shape the
//!   game crate's session will plug into.

use crate::net::{Callbacks, Event, Net};
use crate::pad::{np_decode, np_encode, RbInput};
use crate::MAX_SLOTS;
use std::cell::RefCell;
use std::rc::Rc;

/// gw_rollback.h, the network-facing interface (current epoch).
pub trait RollbackSession {
    /// `gw_rb_submit_remote_input`: any number of times, any order (redelivery harmless).
    fn submit_remote_input(&mut self, slot: usize, frame: i32, input: &RbInput);
    /// `gw_rb_local_input_for_send`: the local input used (or to be used) for `frame`.
    fn local_input_for_send(&self, slot: usize, frame: i32) -> Option<RbInput>;
    /// `gw_rb_checksum`: the checksum at the START of `frame`, only once final; else 0.
    fn checksum(&self, frame: i32) -> u32;
    /// `gw_rb_current_frame`: the frame the simulation runs next.
    fn current_frame(&self) -> i32;
    /// `gw_rb_confirmed_frame`.
    fn confirmed_frame(&self) -> i32;
    /// `gw_rb_request_wait`: skip the next `frames` new-frame iterations (negative ignored).
    fn request_wait(&mut self, frames: i32);
}

/// What the adapter saw (events, desyncs), shared with whoever owns the session.
#[derive(Default, Debug)]
pub struct NetLog {
    pub events: Vec<(Event, String)>,
    pub desyncs: Vec<(i32, u32, u32)>,
}

/// The transport callbacks for a [`RollbackSession`] (gw_netplay.c np_cb_*), with the 11-byte
/// raw-pad payload. The clock is the wall clock unless `clock` is set.
pub struct NetAdapter<S: RollbackSession> {
    pub session: Rc<RefCell<S>>,
    pub log: Rc<RefCell<NetLog>>,
    pub clock: Option<Rc<dyn Fn() -> u32>>,
}

impl<S: RollbackSession> NetAdapter<S> {
    pub fn new(session: Rc<RefCell<S>>, log: Rc<RefCell<NetLog>>, clock: Option<Rc<dyn Fn() -> u32>>) -> Self {
        NetAdapter { session, log, clock }
    }

    /// The per-render-tick wiring: poll at the session's current frame, then feed the time-sync
    /// recommendation to the session's actuator.
    pub fn tick(net: &mut Net, session: &Rc<RefCell<S>>) {
        let f = session.borrow().current_frame();
        net.poll(f);
        let w = net.recommend_wait();
        if w > 0 {
            session.borrow_mut().request_wait(w);
        }
    }
}

impl<S: RollbackSession> Callbacks for NetAdapter<S> {
    fn remote_input(&mut self, frame: i32, slot: usize, payload: &[u8]) {
        let i = np_decode(payload);
        self.session.borrow_mut().submit_remote_input(slot, frame, &i);
    }
    fn has_local_input(&self) -> bool {
        true
    }
    fn local_input(&mut self, frame: i32, slot: usize, out: &mut [u8]) -> bool {
        match self.session.borrow().local_input_for_send(slot, frame) {
            Some(i) => {
                np_encode(&i, out);
                true
            }
            None => false,
        }
    }
    fn has_checksum(&self) -> bool {
        true
    }
    fn checksum(&mut self, frame: i32) -> Option<u32> {
        let h = self.session.borrow().checksum(frame);
        if h != 0 {
            Some(h)
        } else {
            None
        }
    }
    fn event(&mut self, ev: Event, msg: Option<&str>) {
        self.log.borrow_mut().events.push((ev, msg.unwrap_or("").to_string()));
    }
    fn desync(&mut self, frame: i32, local: u32, remote: u32) {
        self.log.borrow_mut().desyncs.push((frame, local, remote));
    }
    fn now_ms(&self) -> Option<u32> {
        self.clock.as_ref().map(|c| c())
    }
}

/// A deterministic simulation the driver can rewind.
pub trait Simulation {
    type Snapshot: Clone;
    fn save(&self) -> Self::Snapshot;
    fn load(&mut self, s: &Self::Snapshot);
    /// Run one frame with one input per slot (`present == 0`: no input for that slot).
    fn advance(&mut self, inputs: &[RbInput; MAX_SLOTS]);
    /// A 32-bit gameplay checksum of the current state.
    fn checksum(&self) -> u32;
}

const RB_RING: usize = 64;

#[derive(Clone, Copy, Default)]
struct Entry {
    tag: Option<i32>,
    input: RbInput,
}

struct Snap<T> {
    tag: Option<i32>,
    state: Option<T>,
    checksum: u32,
}

/// A rollback session over a [`Simulation`]. See the module doc.
pub struct RollbackDriver<G: Simulation> {
    pub sim: G,
    first: i32,
    frame: i32,
    delay: i32,
    max: i32,
    local_mask: u8,
    remote_mask: u8,
    inputs: Vec<[Entry; MAX_SLOTS]>,
    used: Vec<(Option<i32>, [RbInput; MAX_SLOTS])>,
    snaps: Vec<Snap<G::Snapshot>>,
    pending: Option<i32>,
    wait: i32,
    confirmed: i32,
    pub rollbacks: u32,
    pub stalls: u32,
    pub waits: u32,
    pub desyncs: u32,
    pub resimulated: u32,
}

fn ix(f: i32) -> usize {
    f.rem_euclid(RB_RING as i32) as usize
}

impl<G: Simulation> RollbackDriver<G> {
    /// `first`: the first frame (-123 in Melee); `delay`: input delay (2); `max`: the maximum
    /// rollback / prediction window (7, at most 12). The local inputs of frames
    /// `first .. first + delay` are neutral (present, all zero), as nothing was sampled for them.
    pub fn new(sim: G, first: i32, delay: i32, max: i32, local_mask: u8, remote_mask: u8) -> Self {
        let mut d = RollbackDriver {
            sim,
            first,
            frame: first,
            delay,
            max: max.clamp(1, 12),
            local_mask,
            remote_mask,
            inputs: vec![[Entry::default(); MAX_SLOTS]; RB_RING],
            used: vec![(None, [RbInput::default(); MAX_SLOTS]); RB_RING],
            snaps: (0..RB_RING).map(|_| Snap { tag: None, state: None, checksum: 0 }).collect(),
            pending: None,
            wait: 0,
            confirmed: first - 1,
            rollbacks: 0,
            stalls: 0,
            waits: 0,
            desyncs: 0,
            resimulated: 0,
        };
        for f in first..first + delay {
            for s in 0..MAX_SLOTS {
                if local_mask & (1 << s) != 0 {
                    d.store_local(s, f, &RbInput::default());
                }
            }
        }
        d
    }

    fn store_local(&mut self, slot: usize, frame: i32, input: &RbInput) {
        let mut i = *input;
        i.present = 1;
        i.confirmed = 1;
        self.inputs[ix(frame)][slot] = Entry { tag: Some(frame), input: i };
    }

    /// `gw_rb_submit_local_input`: a local input for `frame` (normally current + delay).
    pub fn submit_local_input(&mut self, slot: usize, frame: i32, input: &RbInput) {
        if frame < self.frame || frame >= self.frame + RB_RING as i32 - 8 {
            return;
        }
        self.store_local(slot, frame, input);
    }

    /// Delay in frames: a local input sampled now belongs to `current_frame() + delay()`.
    pub fn delay(&self) -> i32 {
        self.delay
    }

    fn confirmed_at(&self, slot: usize, frame: i32) -> Option<RbInput> {
        let e = &self.inputs[ix(frame)][slot];
        if e.tag == Some(frame) {
            Some(e.input)
        } else {
            None
        }
    }

    /// The input a slot gets at `frame`: confirmed, or a prediction (the newest confirmed input
    /// before it, repeated, marked unconfirmed).
    fn input_for(&self, slot: usize, frame: i32) -> RbInput {
        if let Some(i) = self.confirmed_at(slot, frame) {
            return i;
        }
        let lo = (frame - RB_RING as i32 + 1).max(self.first);
        let mut f = frame - 1;
        while f >= lo {
            if let Some(mut i) = self.confirmed_at(slot, f) {
                i.confirmed = 0;
                return i;
            }
            f -= 1;
        }
        RbInput { present: 1, ..RbInput::default() }
    }

    fn frame_inputs(&self, frame: i32) -> [RbInput; MAX_SLOTS] {
        let mut a = [RbInput::default(); MAX_SLOTS];
        for (s, slot) in a.iter_mut().enumerate() {
            if (self.local_mask | self.remote_mask) & (1 << s) != 0 {
                *slot = self.input_for(s, frame);
            }
        }
        a
    }

    fn advance_confirmed(&mut self) {
        loop {
            let f = self.confirmed + 1;
            let all = (0..MAX_SLOTS)
                .filter(|s| self.remote_mask & (1 << s) != 0)
                .all(|s| self.confirmed_at(s, f).is_some());
            if !all {
                break;
            }
            self.confirmed = f;
        }
    }

    /// Simulate frame `f` from the current state: snapshot its start, then advance.
    fn run_frame(&mut self, f: i32) {
        let snap = self.sim.save();
        let c = self.sim.checksum();
        let k = ix(f);
        self.snaps[k] = Snap { tag: Some(f), state: Some(snap), checksum: c };
        let inputs = self.frame_inputs(f);
        self.used[k] = (Some(f), inputs);
        self.sim.advance(&inputs);
    }

    /// One render tick (gw_RB_Iterations + the iterations): a pending rollback first (load the
    /// snapshot of the earliest mispredicted frame, resimulate to the present), then at most one
    /// new frame unless waiting (time sync) or stalled (more than `max` frames past the newest
    /// confirmed frame). Returns the number of frames simulated.
    pub fn tick(&mut self) -> u32 {
        let ran = self.resolve_rollback();
        if self.wait > 0 {
            self.wait -= 1;
            self.waits += 1;
            return ran;
        }
        if self.frame - self.confirmed > self.max {
            self.stalls += 1;
            return ran;
        }
        let local_ready = (0..MAX_SLOTS)
            .filter(|s| self.local_mask & (1 << s) != 0)
            .all(|s| self.confirmed_at(s, self.frame).is_some());
        if !local_ready {
            self.stalls += 1;
            return ran;
        }
        let f = self.frame;
        self.run_frame(f);
        self.frame += 1;
        ran + 1
    }

    /// Only the rollback half of [`tick`](Self::tick): resimulate a pending correction without
    /// running a new frame (e.g. after the last frame of a match). Returns frames resimulated.
    pub fn resolve_rollback(&mut self) -> u32 {
        let mut ran = 0;
        if let Some(r) = self.pending.take() {
            let k = ix(r);
            if self.snaps[k].tag == Some(r) {
                let st = self.snaps[k].state.clone().expect("snapshot");
                self.sim.load(&st);
                for f in r..self.frame {
                    self.run_frame(f);
                    ran += 1;
                    self.resimulated += 1;
                }
                self.rollbacks += 1;
            } else {
                self.desyncs += 1; // older than the snapshot window: cannot be corrected
            }
        }
        ran
    }

    pub fn rollback_pending(&self) -> Option<i32> {
        self.pending
    }
}

impl<G: Simulation> RollbackSession for RollbackDriver<G> {
    fn submit_remote_input(&mut self, slot: usize, frame: i32, input: &RbInput) {
        if slot >= MAX_SLOTS || self.remote_mask & (1 << slot) == 0 || frame < self.first {
            return;
        }
        if frame > self.frame + 56 || frame <= self.frame - RB_RING as i32 + 8 {
            return; // refused (outside the input ring), as gw_rb_submit_remote_input
        }
        if self.confirmed_at(slot, frame).is_some() {
            return; // redelivery
        }
        let mut i = *input;
        i.present = 1;
        i.confirmed = 1;
        self.inputs[ix(frame)][slot] = Entry { tag: Some(frame), input: i };
        if frame < self.frame {
            let (tag, used) = &self.used[ix(frame)];
            if *tag == Some(frame) && !used[slot].same_pad(&i) {
                self.pending = Some(self.pending.map_or(frame, |p| p.min(frame)));
            }
        }
        self.advance_confirmed();
    }

    fn local_input_for_send(&self, slot: usize, frame: i32) -> Option<RbInput> {
        if self.local_mask & (1 << slot) == 0 {
            return None;
        }
        self.confirmed_at(slot, frame)
    }

    fn checksum(&self, frame: i32) -> u32 {
        if frame >= self.frame || frame - 1 > self.confirmed {
            return 0;
        }
        if let Some(p) = self.pending {
            if frame > p {
                return 0;
            }
        }
        let s = &self.snaps[ix(frame)];
        if s.tag != Some(frame) {
            return 0;
        }
        if s.checksum == 0 {
            1
        } else {
            s.checksum
        }
    }

    fn current_frame(&self) -> i32 {
        self.frame
    }
    fn confirmed_frame(&self) -> i32 {
        self.confirmed
    }
    fn request_wait(&mut self, frames: i32) {
        if frames > 0 {
            self.wait += frames;
        }
    }
}
