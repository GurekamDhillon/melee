//! The deterministic simulated network of gw_net_tests.c (simnet): loss, duplication, jitter
//! (hence reordering), a blackhole switch, virtual time in 16.667 ms ticks. Reproduced exactly
//! (same xorshift64* stream, same queue discipline) so a run is packet-for-packet identical to the
//! C test harness; `log` records every datagram handed to the network for the oracle comparison.

use crate::transport::{Addr, Transport};
use std::cell::RefCell;
use std::rc::Rc;

pub const SIM_ENDPOINTS: usize = 3;
pub const SIM_QUEUE: usize = 4096;

#[derive(Clone)]
struct SimPkt {
    at: u32,
    to: usize,
    from: Addr,
    data: Vec<u8>,
}

pub struct SimNet {
    pub now: u32,
    pub time_us: u64,
    pub rng: u64,
    pub loss_pct: i32,
    pub dup_pct: i32,
    pub base_ms: u32,
    pub jitter_ms: u32,
    pub blackhole: bool,
    /// endpoint + 1 whose HELLO/ACCEPT is rewritten to advertise protocol 5; 0 disables
    pub old_heap_peer: i32,
    q: Vec<SimPkt>,
    pub sent: u32,
    pub dropped: u32,
    /// When set, every send is appended as `"<now> <from>><to> <len> <fnv64 of the bytes with the
    /// session id masked>"`.
    pub log: Option<Vec<String>>,
}

pub type SimRef = Rc<RefCell<SimNet>>;

impl SimNet {
    pub fn new(seed: u64) -> SimNet {
        SimNet {
            now: 0,
            time_us: 0,
            rng: if seed != 0 { seed } else { 88172645463325252 },
            loss_pct: 0,
            dup_pct: 0,
            base_ms: 20,
            jitter_ms: 0,
            blackhole: false,
            old_heap_peer: 0,
            q: Vec::with_capacity(SIM_QUEUE),
            sent: 0,
            dropped: 0,
            log: None,
        }
    }
    pub fn shared(seed: u64) -> SimRef {
        Rc::new(RefCell::new(SimNet::new(seed)))
    }

    /// xorshift64*
    pub fn rnd(&mut self) -> u32 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(2685821657736338717) >> 32) as u32
    }

    pub fn addr(id: usize) -> Addr {
        Addr::new((10u32 << 24) | (id as u32 + 1), 7000)
    }

    pub fn step(&mut self) {
        self.time_us += 16667;
        self.now = (self.time_us / 1000) as u32;
    }

    fn enqueue(&mut self, from_id: usize, to: usize, data: &[u8]) {
        if self.q.len() >= SIM_QUEUE || data.len() > 1300 {
            return;
        }
        let at = self
            .now
            .wrapping_add(self.base_ms)
            .wrapping_add(if self.jitter_ms != 0 { self.rnd() % (self.jitter_ms + 1) } else { 0 });
        let mut d = data.to_vec();
        if self.old_heap_peer == from_id as i32 + 1 && d.len() >= 20 && (d[2] == 1 || d[2] == 2) {
            d[18] = 5;
            d[19] = 0;
        }
        self.q.push(SimPkt { at, to, from: SimNet::addr(from_id), data: d });
    }

    fn send(&mut self, from_id: usize, to: &Addr, data: &[u8]) -> i32 {
        let dest = (to.ip & 0xFF) as i32 - 1;
        self.sent += 1;
        if let Some(log) = self.log.as_mut() {
            let mut m = data.to_vec();
            if m.len() >= 8 {
                m[4..8].fill(0);
            }
            log.push(format!("{} {}>{} {} {:016x}", self.now, from_id, dest, data.len(), crate::hash64(&m, 0)));
        }
        if dest < 0 || dest >= SIM_ENDPOINTS as i32 {
            return data.len() as i32;
        }
        if self.blackhole || ((self.rnd() % 100) as i32) < self.loss_pct {
            self.dropped += 1;
            return data.len() as i32;
        }
        self.enqueue(from_id, dest as usize, data);
        if ((self.rnd() % 100) as i32) < self.dup_pct {
            self.enqueue(from_id, dest as usize, data);
        }
        data.len() as i32
    }

    fn recv(&mut self, id: usize, from: &mut Addr, buf: &mut [u8]) -> i32 {
        let mut best: Option<usize> = None;
        for (i, p) in self.q.iter().enumerate() {
            if p.to == id
                && (self.now.wrapping_sub(p.at) as i32) >= 0
                && best.is_none_or(|b| (p.at.wrapping_sub(self.q[b].at) as i32) < 0)
            {
                best = Some(i);
            }
        }
        let Some(b) = best else { return 0 };
        let p = self.q.swap_remove(b); // q[best] = q[--nq]
        let n = p.data.len().min(buf.len());
        *from = p.from;
        buf[..n].copy_from_slice(&p.data[..n]);
        n as i32
    }
}

/// One endpoint of a [`SimNet`] (`sim_transport`).
pub struct SimEndpoint {
    pub sim: SimRef,
    pub id: usize,
}

impl SimEndpoint {
    pub fn new(sim: &SimRef, id: usize) -> Box<SimEndpoint> {
        Box::new(SimEndpoint { sim: sim.clone(), id })
    }
}

impl Transport for SimEndpoint {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        self.sim.borrow_mut().send(self.id, to, data)
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        self.sim.borrow_mut().recv(self.id, from, buf)
    }
}
