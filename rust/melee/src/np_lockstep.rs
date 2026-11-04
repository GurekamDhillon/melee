//! Lockstep pads for the scenes outside the match (`melee play --netplay ... --np-css`): the character select, the stage
//! select, the results. Both peers run the same boot and the same menu frames; a frame runs only when both ports' pads
//! for it are known (no prediction, no snapshots: the menus are not under the rollback session, whose snapshot base must
//! not straddle a scene's heap setup). The match itself stays under `RbSession` (play_net.rs).
//!
//! Wire (channel 1 of the run's one socket, np_mux.rs): `MLS1`, u32 ack (the next peer frame this side needs), u32
//! first frame, u8 count, then `count` pads of 12 bytes (button u16 LE, stick x/y, c-stick x/y, L, R, analog A, analog
//! B, err, 0) for frames first.. . Every packet repeats all pads the peer has not acknowledged, so a lost datagram costs
//! nothing but its successor's arrival.
//!
//! Lab sessions (`--np-lab`, `LabNetplay`: the match stays in lockstep too) use `MLS2`: the same header plus u32 hash
//! frame and u32 hash (this side's newest match-frame hash, the desync check), then per frame the 12-byte pad, u8 command
//! count and that many 16-byte lab commands (melee_modhost::lab::LabCmd). A command rides with the pad of the frame it
//! applies on, so the peer can never run that frame without it: both peers apply it on the same frame, whatever the
//! jitter or loss, and a lost datagram is repaired like a pad.
//!
//! Each render tick schedules one local pad (`push_local`) at local frame + delay, the pad script's frame k thus always
//! lands on menu frame k + delay, whatever the stalls; `ready` pops the next frame once the peer's pad for it arrived.

use melee_net::transport::{Addr, Transport};
use melee_types::PADStatus;
use std::time::Instant;

const MAGIC: &[u8; 4] = b"MLS1";
const MAGIC_LAB: &[u8; 4] = b"MLS2";
const CMD: usize = 16;
/// Bytes of frame records per MLS2 packet (the header is 21 bytes): under the 1280-byte minimum MTU with headers.
const LAB_BUDGET: usize = 1060;
const HASHES_KEPT: usize = 4096;
const PAD: usize = 12;
// 13 + 90 * 12 = 1093 bytes: under the 1280-byte IPv6 minimum MTU with the relay and IP/UDP headers
const MAX_PER_PACKET: usize = 90;

pub struct Lockstep {
    tp: Box<dyn Transport>,
    peer: Option<Addr>,
    pub host: bool,
    delay: u32,
    /// local pads by frame (index = frame)
    local: Vec<PADStatus>,
    /// peer pads by frame
    remote: Vec<Option<PADStatus>>,
    /// next frame to run
    pub frame: u32,
    /// the first local frame the peer has not acknowledged
    peer_ack: u32,
    last_heard: Instant,
    pub stalls: u64,
    /// MLS2: lab commands ride with the pads.
    pub lab: bool,
    local_cmds: Vec<Vec<[u8; CMD]>>,
    remote_cmds: Vec<Vec<[u8; CMD]>>,
    /// (frame, hash) of this side's newest checked frames, and the peer's not yet matched.
    hashes: std::collections::VecDeque<(u32, u32)>,
    peer_hashes: std::collections::BTreeMap<u32, u32>,
    newest: (u32, u32),
    last_compared: Option<u32>,
    pub checked: u64,
    pub desyncs: u64,
    pub first_desync: Option<u32>,
}

fn enc(p: &PADStatus, o: &mut Vec<u8>) {
    o.extend_from_slice(&p.button.to_le_bytes());
    o.extend_from_slice(&[p.stickX as u8, p.stickY as u8, p.substickX as u8, p.substickY as u8, p.triggerLeft, p.triggerRight, p.analogA, p.analogB, p.err as u8, 0]);
}
fn dec(b: &[u8]) -> PADStatus {
    let mut p = PADStatus::zeroed();
    p.button = u16::from_le_bytes([b[0], b[1]]);
    p.stickX = b[2] as i8;
    p.stickY = b[3] as i8;
    p.substickX = b[4] as i8;
    p.substickY = b[5] as i8;
    p.triggerLeft = b[6];
    p.triggerRight = b[7];
    p.analogA = b[8];
    p.analogB = b[9];
    p.err = b[10] as i8;
    p
}

impl Lockstep {
    /// `peer` None (the host): learnt from the first packet.
    pub fn new(tp: Box<dyn Transport>, peer: Option<Addr>, host: bool, delay: u32) -> Lockstep {
        let mut local = Vec::new();
        let mut remote = Vec::new();
        // the first `delay` frames have no input from either side: neutral
        for _ in 0..delay {
            local.push(PADStatus::zeroed());
            remote.push(Some(PADStatus::zeroed()));
        }
        let n = delay as usize;
        Lockstep {
            tp, peer, host, delay, local, remote, frame: 0, peer_ack: 0, last_heard: Instant::now(), stalls: 0, lab: false,
            local_cmds: vec![Vec::new(); n], remote_cmds: vec![Vec::new(); n], hashes: Default::default(), peer_hashes: Default::default(),
            newest: (0, 0), last_compared: None, checked: 0, desyncs: 0, first_desync: None,
        }
    }

    pub fn delay(&self) -> u32 {
        self.delay
    }

    /// The frame the next `push_local` schedules (a lab command issued now applies there).
    pub fn next_push_frame(&self) -> u32 {
        self.local.len() as u32
    }

    /// `push_local` with this frame's lab commands: the frame they apply on, or None when the queue is full (keep them).
    pub fn push_local_cmds(&mut self, p: &PADStatus, cmds: Vec<[u8; CMD]>) -> Option<u32> {
        if self.local.len() >= (self.frame + self.delay) as usize + 600 {
            return None;
        }
        let f = self.local.len() as u32;
        self.local.push(*p);
        self.local_cmds.resize(f as usize, Vec::new());
        self.local_cmds.push(cmds);
        Some(f)
    }

    /// This side's hash of a frame it ran (0 = nothing to compare): sent to the peer and checked against its hash.
    pub fn record_hash(&mut self, frame: u32, hash: u32) {
        if hash == 0 {
            return;
        }
        self.newest = (frame, hash);
        if let Some(h) = self.peer_hashes.remove(&frame) {
            if self.last_compared.map_or(true, |f| frame > f) {
                self.last_compared = Some(frame);
                self.compare(frame, hash, h);
            }
        }
        self.hashes.push_back((frame, hash));
        if self.hashes.len() > HASHES_KEPT {
            self.hashes.pop_front();
        }
    }

    fn compare(&mut self, frame: u32, mine: u32, theirs: u32) {
        self.checked += 1;
        if mine != theirs {
            self.desyncs += 1;
            if self.first_desync.is_none() {
                self.first_desync = Some(frame);
                eprintln!("lockstep: DESYNC at frame {frame}: {mine:08x} here, {theirs:08x} on the peer");
            }
        }
    }

    /// The peer's newest (frame, hash): each frame is compared once (packets repeat it until a newer one is hashed).
    fn peer_hash(&mut self, frame: u32, hash: u32) {
        if hash == 0 || self.last_compared.is_some_and(|f| f >= frame) {
            return;
        }
        if let Some(&(_, mine)) = self.hashes.iter().rev().find(|x| x.0 == frame) {
            self.last_compared = Some(frame);
            self.compare(frame, mine, hash);
        } else if frame > self.newest.0 || self.hashes.is_empty() {
            self.peer_hashes.insert(frame, hash);
            while self.peer_hashes.len() > HASHES_KEPT {
                let k = *self.peer_hashes.keys().next().unwrap();
                self.peer_hashes.remove(&k);
            }
        }
    }

    fn poll_lab(&mut self) {
        let mut buf = [0u8; 2048];
        loop {
            let mut from = Addr::default();
            let n = self.tp.recv(&mut from, &mut buf);
            if n <= 0 {
                break;
            }
            let d = &buf[..n as usize];
            if d.len() < 21 || &d[..4] != MAGIC_LAB {
                continue;
            }
            if self.peer.is_none() {
                self.peer = Some(from);
            }
            self.last_heard = Instant::now();
            let u = |i: usize| u32::from_le_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);
            let (ack, first, cnt, hf, hh) = (u(4), u(8), d[12] as usize, u(13), u(17));
            self.peer_ack = self.peer_ack.max(ack);
            let mut o = 21;
            for k in 0..cnt {
                if o + PAD + 1 > d.len() {
                    break;
                }
                let nc = d[o + PAD] as usize;
                if o + PAD + 1 + nc * CMD > d.len() {
                    break;
                }
                let f = first as usize + k;
                if self.remote.len() <= f {
                    self.remote.resize(f + 1, None);
                }
                if self.remote_cmds.len() <= f {
                    self.remote_cmds.resize(f + 1, Vec::new());
                }
                if self.remote[f].is_none() {
                    self.remote[f] = Some(dec(&d[o..o + PAD]));
                    self.remote_cmds[f] = (0..nc).map(|i| d[o + PAD + 1 + i * CMD..o + PAD + 1 + (i + 1) * CMD].try_into().unwrap()).collect();
                }
                o += PAD + 1 + nc * CMD;
            }
            self.peer_hash(hf, hh);
        }
        let Some(peer) = self.peer else { return };
        let first = self.peer_ack.min(self.local.len() as u32);
        let mut o = MAGIC_LAB.to_vec();
        o.extend_from_slice(&self.need().to_le_bytes());
        o.extend_from_slice(&first.to_le_bytes());
        o.push(0);
        o.extend_from_slice(&self.newest.0.to_le_bytes());
        o.extend_from_slice(&self.newest.1.to_le_bytes());
        let mut cnt = 0u8;
        let mut f = first as usize;
        while f < self.local.len() && cnt < u8::MAX {
            let cmds = self.local_cmds.get(f).map_or(&[][..], |v| &v[..]);
            if o.len() - 21 + PAD + 1 + cmds.len() * CMD > LAB_BUDGET && cnt > 0 {
                break;
            }
            enc(&self.local[f], &mut o);
            o.push(cmds.len() as u8);
            for c in cmds {
                o.extend_from_slice(c);
            }
            cnt += 1;
            f += 1;
        }
        o[12] = cnt;
        self.tp.send(&peer, &o);
    }

    /// This tick's local pad: it runs at the next unscheduled frame (frame + delay in a steady state).
    pub fn push_local(&mut self, p: &PADStatus) {
        // a scheduled frame may not run before the peer had the chance to see it: cap the queue at 10 s
        if self.local.len() < (self.frame + self.delay) as usize + 600 {
            self.local.push(*p);
        }
    }

    fn need(&self) -> u32 {
        self.remote.iter().position(|r| r.is_none()).unwrap_or(self.remote.len()) as u32
    }

    /// Receive, then send this side's unacknowledged pads.
    pub fn poll(&mut self) {
        if self.lab {
            return self.poll_lab();
        }
        let mut buf = [0u8; 2048];
        loop {
            let mut from = Addr::default();
            let n = self.tp.recv(&mut from, &mut buf);
            if n <= 0 {
                break;
            }
            let d = &buf[..n as usize];
            if d.len() < 13 || &d[..4] != MAGIC {
                continue;
            }
            if self.peer.is_none() {
                self.peer = Some(from);
            }
            self.last_heard = Instant::now();
            let ack = u32::from_le_bytes([d[4], d[5], d[6], d[7]]);
            let first = u32::from_le_bytes([d[8], d[9], d[10], d[11]]);
            let cnt = d[12] as usize;
            self.peer_ack = self.peer_ack.max(ack);
            for k in 0..cnt {
                let o = 13 + k * PAD;
                if o + PAD > d.len() {
                    break;
                }
                let f = (first as usize) + k;
                if self.remote.len() <= f {
                    self.remote.resize(f + 1, None);
                }
                if self.remote[f].is_none() {
                    self.remote[f] = Some(dec(&d[o..o + PAD]));
                }
            }
        }
        let Some(peer) = self.peer else { return };
        let first = self.peer_ack.min(self.local.len() as u32);
        let cnt = (self.local.len() as u32 - first).min(MAX_PER_PACKET as u32);
        let mut o = MAGIC.to_vec();
        o.extend_from_slice(&self.need().to_le_bytes());
        o.extend_from_slice(&first.to_le_bytes());
        o.push(cnt as u8);
        for f in first..first + cnt {
            enc(&self.local[f as usize], &mut o);
        }
        self.tp.send(&peer, &o);
    }

    /// The next frame's pads as (local, remote), or None while the peer's has not arrived.
    pub fn ready(&mut self) -> Option<(PADStatus, PADStatus)> {
        let f = self.frame as usize;
        match (self.local.get(f), self.remote.get(f).copied().flatten()) {
            (Some(l), Some(r)) => {
                self.frame += 1;
                Some((*l, r))
            }
            _ => {
                self.stalls += 1;
                None
            }
        }
    }

    /// `ready` with the frame's lab commands: (local pad, remote pad, local commands, remote commands, frame).
    pub fn ready_lab(&mut self) -> Option<(PADStatus, PADStatus, Vec<[u8; CMD]>, Vec<[u8; CMD]>, u32)> {
        let f = self.frame as usize;
        let (l, r) = self.ready()?;
        // a frame's commands are kept until the peer acknowledged the frame (resends); remote ones are taken
        let lc = self.local_cmds.get(f).cloned().unwrap_or_default();
        let rc = self.remote_cmds.get_mut(f).map(std::mem::take).unwrap_or_default();
        Some((l, r, lc, rc, f as u32))
    }

    /// Back from a match (the lockstep was idle while the session ran): the silence count starts now.
    pub fn resume(&mut self) {
        self.last_heard = Instant::now();
    }

    /// Seconds since the peer was last heard from.
    pub fn silent_secs(&self) -> u64 {
        self.last_heard.elapsed().as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    /// An in-memory link with jitter, reordering and loss: datagrams are delivered after a random number of polls.
    struct Wire {
        q: [VecDeque<(u64, Vec<u8>)>; 2],
        now: u64,
        seed: u32,
        loss_pct: u32,
        max_delay: u64,
    }
    impl Wire {
        fn rnd(&mut self) -> u32 {
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 17;
            self.seed ^= self.seed << 5;
            self.seed
        }
    }
    struct End(Rc<RefCell<Wire>>, usize);
    impl Transport for End {
        fn send(&mut self, _to: &Addr, data: &[u8]) -> i32 {
            let mut w = self.0.borrow_mut();
            if w.rnd() % 100 < w.loss_pct {
                return data.len() as i32;
            }
            let at = w.now + (w.rnd() as u64 % (w.max_delay + 1));
            let to = 1 - self.1;
            w.q[to].push_back((at, data.to_vec()));
            // reorder: keep the queue sorted by delivery time only, so later sends can overtake
            w.q[to].make_contiguous().sort_by_key(|x| x.0);
            data.len() as i32
        }
        fn recv(&mut self, _from: &mut Addr, buf: &mut [u8]) -> i32 {
            let mut w = self.0.borrow_mut();
            let now = w.now;
            match w.q[self.1].front() {
                Some((at, _)) if *at <= now => {
                    let (_, d) = w.q[self.1].pop_front().unwrap();
                    buf[..d.len()].copy_from_slice(&d);
                    d.len() as i32
                }
                _ => 0,
            }
        }
    }

    fn pad(n: u32) -> PADStatus {
        let mut p = PADStatus::zeroed();
        p.button = (n & 0xFFF) as u16;
        p
    }

    /// Both peers issue commands at pseudo-random ticks over a jittery, lossy, reordering link; every command must
    /// reach both sides on the same frame and in the same order (host first), and the pad streams must agree.
    fn run(loss: u32, max_delay: u64, seed: u32) {
        let w = Rc::new(RefCell::new(Wire { q: Default::default(), now: 0, seed, loss_pct: loss, max_delay }));
        let mut a = Lockstep::new(Box::new(End(w.clone(), 0)), Some(Addr::default()), true, 3);
        let mut b = Lockstep::new(Box::new(End(w.clone(), 1)), Some(Addr::default()), false, 3);
        a.lab = true;
        b.lab = true;
        let mut log_a: Vec<(u32, Vec<[u8; 16]>)> = Vec::new();
        let mut log_b: Vec<(u32, Vec<[u8; 16]>)> = Vec::new();
        let mut sent = [Vec::new(), Vec::new()];
        let mut hash = |f: u32| f.wrapping_mul(2654435761) | 1;
        for t in 0..3000u32 {
            w.borrow_mut().now = t as u64;
            for (side, ls) in [&mut a, &mut b].into_iter().enumerate() {
                // keep at most `delay` + a few frames in flight, like a render tick does
                if ls.next_push_frame() < ls.frame + 6 {
                    let r = w.borrow_mut().rnd();
                    let cmds: Vec<[u8; 16]> = if r % 7 == 0 { vec![[side as u8 + 1, (t & 0xFF) as u8, 0, 0, (t >> 8) as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]] } else { vec![] };
                    let f = ls.push_local_cmds(&pad(t * 2 + side as u32), cmds.clone()).unwrap();
                    if !cmds.is_empty() {
                        sent[side].push((f, cmds[0]));
                    }
                }
                ls.poll();
            }
            while let Some((l, r, lc, rc, f)) = a.ready_lab() {
                assert_eq!(a.local[f as usize].button, l.button);
                let _ = r;
                let mut v = lc;
                v.extend(rc);
                log_a.push((f, v));
                a.record_hash(f, hash(f));
            }
            while let Some((_l, _r, lc, rc, f)) = b.ready_lab() {
                // the guest orders the host's (remote) commands first, as the play flow does
                let mut v = rc;
                v.extend(lc);
                log_b.push((f, v));
                b.record_hash(f, hash(f));
            }
        }
        let n = log_a.len().min(log_b.len());
        assert!(n > 2000, "only {n} frames ran (loss {loss}, delay {max_delay})");
        for i in 0..n {
            assert_eq!(log_a[i], log_b[i], "frame {i} differs");
        }
        // every command issued for a frame that both ran arrived on its frame
        for side in 0..2 {
            for &(f, c) in &sent[side] {
                if (f as usize) < n {
                    assert!(log_a[f as usize].1.contains(&c), "side {side} command for frame {f} missing");
                }
            }
        }
        assert_eq!(a.desyncs + b.desyncs, 0);
        assert!(a.checked > 100 && b.checked > 100, "hash checks {} / {}", a.checked, b.checked);
    }

    #[test]
    fn commands_apply_on_the_same_frame_clean_link() {
        run(0, 0, 1);
    }

    #[test]
    fn commands_apply_on_the_same_frame_with_jitter_and_loss() {
        run(10, 6, 0xBEEF);
        run(30, 12, 0x1234_5678);
    }

    #[test]
    fn a_desync_is_detected() {
        let w = Rc::new(RefCell::new(Wire { q: Default::default(), now: 0, seed: 9, loss_pct: 0, max_delay: 2 }));
        let mut a = Lockstep::new(Box::new(End(w.clone(), 0)), Some(Addr::default()), true, 2);
        let mut b = Lockstep::new(Box::new(End(w.clone(), 1)), Some(Addr::default()), false, 2);
        a.lab = true;
        b.lab = true;
        for t in 0..400u32 {
            w.borrow_mut().now = t as u64;
            for ls in [&mut a, &mut b] {
                if ls.next_push_frame() < ls.frame + 4 {
                    ls.push_local_cmds(&pad(t), vec![]);
                }
                ls.poll();
            }
            while let Some((.., f)) = a.ready_lab() {
                a.record_hash(f, f + 1);
            }
            while let Some((.., f)) = b.ready_lab() {
                b.record_hash(f, if f >= 200 { f + 2 } else { f + 1 });
            }
        }
        assert!(a.desyncs > 0 && b.desyncs > 0);
        assert!(a.first_desync.unwrap() >= 200);
    }
}
