//! Connecting two mu_net peers (plan sections 2.2 and 2.7): dial every candidate over one ENet host, punch with
//! one-byte datagrams, run the HELLO on every connection that comes up, let the decider choose one, and compare the
//! descriptor digests with VERIFY. After that the [`Connector`] is the session's protected message pipe.
//!
//! No threads, no blocking: [`Connector::poll`] is called from the caller's loop (each frame, or every few ms while
//! waiting), like the rest of this crate.

use super::crypto::{Bytes32, Identity};
use super::desc::Descriptor;
use super::sec::{Accepted, SecError, Security, MAX_PACKET};
use super::wire::{Msg, BYE_LEFT, BYE_MISMATCH, CH_CONTROL};
use crate::enet::{Event, Host, PeerState};
use crate::transport::{Addr, Transport};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const MAX_CANDIDATES: usize = 6;
pub const MAX_NEIGHBOURS: usize = 8;
/// ENet peers: candidates dialed plus connections that arrive.
pub const MAX_PATHS: usize = 8;
pub const CHANNELS: usize = 2;
pub const DEFAULT_LIMIT: Duration = Duration::from_secs(8);
const PUNCH_EVERY: Duration = Duration::from_millis(100);
const NEIGHBOURS_AFTER: Duration = Duration::from_secs(1);
/// ENet keep-alive and gone-peer timing on every connection (`enet_peer_ping_interval`, `enet_peer_timeout`).
const PING_MS: u32 = 250;
const TIMEOUT_MIN_MS: u32 = 4000;
const TIMEOUT_MAX_MS: u32 = 8000;
/// Messages kept for the layer above before the oldest is dropped.
const INBOX_MAX: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// no connection is up yet
    Connecting,
    /// a connection is up; HELLOs in flight
    Handshaking,
    /// keys agreed; digests being compared
    Verifying,
    /// both sides hold the same descriptor: the game may start
    Ready,
    /// the peer left (BYE 0) or the chosen connection was lost, after Ready
    Closed(String),
    Failed(String),
}

#[derive(Default, Clone, Debug)]
pub struct Counters {
    pub punches: u64,
    pub dials: u64,
    pub connections: u64,
    pub hellos_refused: u64,
    pub packets_refused: u64,
    pub unknown: u64,
    pub inbox_dropped: u64,
}

/// Whether an IPv4 address is a public one (not private, loopback, link-local, carrier-shared or multicast): the
/// candidates whose neighbouring ports get punch bytes.
pub fn is_public(a: &Addr) -> bool {
    if a.is_v6() {
        return false;
    }
    let o = a.ip.to_be_bytes();
    !(o[0] == 0
        || o[0] == 10
        || o[0] == 127
        || o[0] >= 224
        || (o[0] == 172 && (16..32).contains(&o[1]))
        || (o[0] == 192 && o[1] == 168)
        || (o[0] == 169 && o[1] == 254)
        || (o[0] == 100 && (64..128).contains(&o[1])))
}

/// The neighbouring ports of the public candidates (+1, +2, +3, -1 of each, in that order), at most 8.
pub fn neighbours(cands: &[Addr]) -> Vec<Addr> {
    let mut out = Vec::new();
    for c in cands.iter().filter(|c| is_public(c)) {
        for d in [1i32, 2, 3, -1] {
            let p = c.port as i32 + d;
            let a = c.with_port(p as u16);
            if (1..=65535).contains(&p) && out.len() < MAX_NEIGHBOURS && !out.contains(&a) && !cands.contains(&a) {
                out.push(a);
            }
        }
    }
    out
}

pub struct Connector {
    host: Host,
    cands: Vec<Addr>,
    neigh: Vec<Addr>,
    /// the ENet peer dialed per candidate
    dialed: Vec<Option<usize>>,
    start: Instant,
    limit: Duration,
    next_punch: Instant,
    punch_round: u32,
    sec: Security,
    desc: Descriptor,
    local_slot: usize,
    digest: Option<Bytes32>,
    chosen: Option<usize>,
    verify_sent: bool,
    state: State,
    inbox: VecDeque<Msg>,
    pub counters: Counters,
}

impl Connector {
    /// Opens the session over `tp` (the bound UDP socket): `desc` as both sides agreed it (keys not yet bound),
    /// `local_slot` 0 or 1, `cands` the peer's endpoints (<= 6; none = only listen), `expected` the peer identity.
    pub fn start(
        tp: Box<dyn Transport>,
        identity: Identity,
        desc: Descriptor,
        local_slot: usize,
        cands: &[Addr],
        expected: Option<Bytes32>,
        limit: Duration,
        now: Instant,
    ) -> Result<Connector, String> {
        desc.validate().map_err(str::to_string)?;
        if local_slot > 1 {
            return Err("the local slot must be 0 or 1".into());
        }
        let mut cs: Vec<Addr> = Vec::new();
        for c in cands {
            if !cs.contains(c) {
                cs.push(*c);
            }
        }
        if cs.len() > MAX_CANDIDATES {
            return Err(format!("{} candidate endpoints; at most {MAX_CANDIDATES}", cs.len()));
        }
        let sec = Security::begin(identity, expected).ok_or("no random numbers for the session key")?;
        let mut host = Host::new(tp, MAX_PATHS, CHANNELS);
        host.set_timing(PING_MS, TIMEOUT_MIN_MS, TIMEOUT_MAX_MS);
        let neigh = neighbours(&cs);
        let n = cs.len();
        let mut c = Connector {
            host,
            cands: cs,
            neigh,
            dialed: vec![None; n],
            start: now,
            limit,
            next_punch: now,
            punch_round: 0,
            sec,
            desc,
            local_slot,
            digest: None,
            chosen: None,
            verify_sent: false,
            state: State::Connecting,
            inbox: VecDeque::new(),
            counters: Counters::default(),
        };
        c.dial();
        Ok(c)
    }

    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn ready(&self) -> bool {
        self.state == State::Ready
    }
    pub fn local_port(&self) -> u16 {
        self.host.local_port()
    }
    pub fn decider(&self) -> bool {
        self.sec.decider()
    }
    /// The descriptor with both players' keys bound (after the handshake), and its digest.
    pub fn descriptor(&self) -> &Descriptor {
        &self.desc
    }
    pub fn digest(&self) -> Option<Bytes32> {
        self.digest
    }
    pub fn identity(&self) -> &Identity {
        self.sec.identity()
    }
    pub fn peer_identity(&self) -> Option<Bytes32> {
        self.sec.peer_identity()
    }
    pub fn peer_addr(&self) -> Option<Addr> {
        self.chosen.map(|i| self.host.peers[i].addr)
    }
    /// The ENet round trip of the chosen connection (ms, smoothed).
    pub fn enet_rtt_ms(&self) -> Option<u32> {
        self.chosen.map(|i| self.host.peers[i].rtt)
    }

    fn fail(&mut self, why: String) {
        if !matches!(self.state, State::Failed(_) | State::Closed(_)) {
            self.state = State::Failed(why);
        }
    }

    /// Starts an ENet connect to every candidate that has none in progress (while nothing is chosen).
    fn dial(&mut self) {
        if self.chosen.is_some() {
            return;
        }
        for k in 0..self.cands.len() {
            let a = self.cands[k];
            let live = self.dialed[k].is_some_and(|i| {
                let p = &self.host.peers[i];
                p.addr == a && !matches!(p.state, PeerState::Disconnected | PeerState::Zombie)
            });
            if !live {
                self.dialed[k] = self.host.connect(a, CHANNELS, 0);
                self.counters.dials += 1;
            }
        }
    }

    fn any_connected(&self) -> bool {
        self.host.peers.iter().any(|p| p.state == PeerState::Connected)
    }

    fn punch(&mut self, now: Instant) {
        if self.any_connected() || self.chosen.is_some() || now < self.next_punch {
            return;
        }
        self.next_punch = now + PUNCH_EVERY;
        self.punch_round += 1;
        for k in 0..self.cands.len() {
            let a = self.cands[k];
            self.host.send_raw(a, &[0]);
            self.counters.punches += 1;
        }
        if now.duration_since(self.start) >= NEIGHBOURS_AFTER && self.punch_round % 3 == 0 {
            for k in 0..self.neigh.len() {
                let a = self.neigh[k];
                self.host.send_raw(a, &[0]);
                self.counters.punches += 1;
            }
        }
    }

    /// Runs the network: reads, timers, punching, redials, the handshake; queues messages for [`Connector::recv`].
    pub fn poll(&mut self, now: Instant) {
        if matches!(self.state, State::Failed(_)) {
            return;
        }
        while let Some(e) = self.host.service() {
            match e {
                Event::Connect { peer, .. } => self.on_connect(peer),
                Event::Disconnect { peer, .. } => self.on_disconnect(peer),
                Event::Receive { peer, channel, data } => self.on_receive(peer, channel, &data),
            }
            if matches!(self.state, State::Failed(_)) {
                self.host.flush();
                return;
            }
        }
        if self.chosen.is_none() {
            self.dial();
            self.punch(now);
        }
        let waiting = matches!(self.state, State::Connecting | State::Handshaking | State::Verifying);
        if waiting && now.duration_since(self.start) >= self.limit {
            let why = match self.state {
                State::Connecting => "no connection to the other player came up",
                State::Handshaking => "a connection came up but the other player's hello never verified",
                _ => "the other player did not confirm the match setup",
            };
            self.fail(format!("{why} within {} s", self.limit.as_secs()));
        }
        self.host.flush();
    }

    fn on_connect(&mut self, peer: usize) {
        self.counters.connections += 1;
        if self.chosen.is_some_and(|c| c != peer) {
            self.host.disconnect(peer, 0);
            return;
        }
        if self.state == State::Connecting {
            self.state = State::Handshaking;
        }
        let h = *self.sec.hello();
        self.host.send(peer, CH_CONTROL, &h, true);
    }

    fn on_disconnect(&mut self, peer: usize) {
        if self.chosen == Some(peer) {
            self.chosen = None;
            match self.state {
                State::Ready => self.state = State::Closed("the connection to the other player was lost".into()),
                State::Closed(_) | State::Failed(_) => {}
                _ => self.fail("the chosen connection closed before the match was confirmed".into()),
            }
        }
    }

    /// Keeps `peer` as the connection and closes every other one.
    fn choose(&mut self, peer: usize) {
        self.chosen = Some(peer);
        for i in 0..self.host.peers.len() {
            if i != peer && !matches!(self.host.peers[i].state, PeerState::Disconnected | PeerState::Zombie) {
                self.host.disconnect(i, 0);
            }
        }
    }

    fn send_on(&mut self, peer: usize, m: &Msg) -> bool {
        let ch = m.channel();
        match self.sec.seal(ch, &m.encode()) {
            Some(p) => self.host.send(peer, ch, &p, ch == CH_CONTROL),
            None => false,
        }
    }

    /// Sends one message on the chosen connection (control reliable, input unsequenced). False before VERIFY.
    pub fn send(&mut self, m: &Msg) -> bool {
        match self.chosen {
            Some(p) if self.digest.is_some() => self.send_on(p, m),
            _ => false,
        }
    }

    /// Sends what is queued now (after a frame's messages).
    pub fn flush(&mut self) {
        self.host.flush();
    }

    /// The next message for the layer above (anything but VERIFY), once ready.
    pub fn recv(&mut self) -> Option<Msg> {
        self.inbox.pop_front()
    }

    /// Ends the session: BYE `reason` on the chosen connection, then a disconnect.
    pub fn close(&mut self, reason: u8) {
        if let Some(p) = self.chosen {
            if self.digest.is_some() {
                self.send_on(p, &Msg::Bye { reason });
            }
            self.host.flush();
            self.host.disconnect(p, 0);
            self.host.flush();
        }
        if !matches!(self.state, State::Failed(_)) {
            self.state = State::Closed("this side left".into());
        }
    }

    fn on_receive(&mut self, peer: usize, channel: u8, d: &[u8]) {
        if d.is_empty() || d.len() > MAX_PACKET || channel as usize >= CHANNELS {
            self.counters.packets_refused += 1;
            return;
        }
        if self.chosen.is_some_and(|c| c != peer) {
            return;
        }
        if channel == CH_CONTROL && d[0] == b'M' {
            self.on_hello(peer, d);
            return;
        }
        if channel == CH_CONTROL && d[0] != 1 {
            self.counters.packets_refused += 1;
            return;
        }
        let plain = match self.sec.open(channel, d) {
            Ok(p) => p,
            Err(_) => {
                self.counters.packets_refused += 1;
                return;
            }
        };
        let Some(m) = Msg::decode(&plain) else {
            self.counters.packets_refused += 1;
            return;
        };
        if m.channel() != channel && !matches!(m, Msg::Unknown { .. }) {
            self.counters.packets_refused += 1;
            return;
        }
        match m {
            Msg::Verify { digest, slot } => self.on_verify(peer, digest, slot),
            Msg::Unknown { .. } => self.counters.unknown += 1,
            Msg::Bye { reason } if self.state != State::Ready => {
                self.fail(if reason == BYE_MISMATCH { "the other player has a different match setup or build".into() } else { "the other player left".into() });
            }
            m => {
                if let Msg::Bye { reason } = m {
                    self.state = State::Closed(if reason == BYE_LEFT { "the other player left".into() } else { format!("the other player ended the session ({reason})") });
                }
                if self.inbox.len() >= INBOX_MAX {
                    self.inbox.pop_front();
                    self.counters.inbox_dropped += 1;
                }
                self.inbox.push_back(m);
            }
        }
    }

    fn on_hello(&mut self, peer: usize, d: &[u8]) {
        match self.sec.accept_hello(d) {
            Ok(Accepted::Same) => {}
            Ok(Accepted::New) => {
                let own = (self.sec.identity().public_key(), *self.sec.own_ephemeral());
                let theirs = (self.sec.peer_identity().unwrap(), self.sec.peer_ephemeral().unwrap());
                let keys = if self.local_slot == 0 { [own, theirs] } else { [theirs, own] };
                if let Err(e) = self.desc.bind_keys(keys) {
                    self.fail(e.into());
                    return;
                }
                self.digest = Some(self.desc.digest());
                self.state = State::Verifying;
                if self.sec.decider() {
                    // the connection the accepted HELLO came on; our HELLO again on it, then VERIFY
                    self.choose(peer);
                    let h = *self.sec.hello();
                    self.host.send(peer, CH_CONTROL, &h, true);
                    self.send_verify(peer);
                }
            }
            Err(SecError::Version) => self.fail(SecError::Version.text().into()),
            Err(e) => {
                // only the version is final: anything else may be a stranger at a candidate address
                self.counters.hellos_refused += 1;
                if self.counters.hellos_refused == 1 {
                    eprintln!("mu_net: hello refused: {}", e.text());
                }
            }
        }
    }

    fn send_verify(&mut self, peer: usize) {
        let digest = self.digest.unwrap();
        self.send_on(peer, &Msg::Verify { digest, slot: self.local_slot as u8 });
        self.verify_sent = true;
    }

    fn on_verify(&mut self, peer: usize, digest: Bytes32, slot: u8) {
        if self.state != State::Verifying {
            return; // a VERIFY after ready (a resend): nothing to do
        }
        if !self.verify_sent {
            self.choose(peer);
            self.send_verify(peer);
        }
        if Some(digest) != self.digest || slot as usize != 1 - self.local_slot {
            self.send_on(peer, &Msg::Bye { reason: BYE_MISMATCH });
            self.host.flush();
            self.fail("the match setup differs between the two sides (descriptor mismatch: another build, or other match options)".into());
            return;
        }
        self.state = State::Ready;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mu_net::desc::Slot;
    use crate::transport::UdpTransport;

    const LO: u32 = 0x7f00_0001;

    fn desc(build: &str) -> Descriptor {
        let mut d = Descriptor { build_id: build.into(), rng_seed: 7, ..Default::default() };
        d.players[0] = Slot { present: true, character: 2, name: "A".into(), ..Default::default() };
        d.players[1] = Slot { present: true, character: 9, name: "B".into(), ..Default::default() };
        d
    }

    fn sock() -> (Box<dyn Transport>, u16) {
        let t = UdpTransport::open(LO, 0).unwrap();
        let p = t.local_port();
        (Box::new(t), p)
    }

    struct Pair {
        a: Connector,
        b: Connector,
    }

    /// Two peers on loopback. `a_dials` / `b_dials`: whether each has the other as a candidate.
    fn pair(a_dials: bool, b_dials: bool, b_build: &str, b_expects_a: bool, limit_ms: u64) -> Pair {
        let (ta, pa) = sock();
        let (tb, pb) = sock();
        let (ia, ib) = (Identity::from_seed(&[11; 32]), Identity::from_seed(&[22; 32]));
        let (ka, kb) = (ia.public_key(), ib.public_key());
        let now = Instant::now();
        let lim = Duration::from_millis(limit_ms);
        let ca: Vec<Addr> = if a_dials { vec![Addr::new(LO, pb)] } else { vec![] };
        let cb: Vec<Addr> = if b_dials { vec![Addr::new(LO, pa)] } else { vec![] };
        let expect_a = if b_expects_a { ka } else { [5; 32] };
        let a = Connector::start(ta, ia, desc("rust test"), 0, &ca, Some(kb), lim, now).unwrap();
        let b = Connector::start(tb, ib, desc(b_build), 1, &cb, Some(expect_a), lim, now).unwrap();
        Pair { a, b }
    }

    fn run(p: &mut Pair, until: impl Fn(&Pair) -> bool, ms: u64) -> bool {
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end {
            let now = Instant::now();
            p.a.poll(now);
            p.b.poll(now);
            if until(p) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    #[test]
    fn both_dial_and_verify_then_talk() {
        let mut p = pair(true, true, "rust test", true, 5000);
        assert!(run(&mut p, |p| p.a.ready() && p.b.ready(), 4000), "a {:?} b {:?}", p.a.state(), p.b.state());
        assert_ne!(p.a.decider(), p.b.decider());
        assert_eq!(p.a.digest(), p.b.digest());
        assert_eq!(p.a.descriptor(), p.b.descriptor());
        assert_eq!(p.a.peer_identity(), Some(Identity::from_seed(&[22; 32]).public_key()));
        assert!(p.a.send(&Msg::Ping { stamp: 42 }));
        assert!(p.b.send(&Msg::Info { revision: 2, games: 1 }));
        p.a.flush();
        p.b.flush();
        let mut got_a = None;
        let mut got_b = None;
        run(
            &mut p,
            |_| false,
            200,
        );
        while let Some(m) = p.a.recv() {
            got_a = Some(m);
        }
        while let Some(m) = p.b.recv() {
            got_b = Some(m);
        }
        assert_eq!(got_b, Some(Msg::Ping { stamp: 42 }));
        assert_eq!(got_a, Some(Msg::Info { revision: 2, games: 1 }));
        // only one connection is left on each side
        let live = |c: &Connector| c.host.peers.iter().filter(|x| x.state == PeerState::Connected).count();
        assert_eq!((live(&p.a), live(&p.b)), (1, 1));
        p.a.close(BYE_LEFT);
        assert!(run(&mut p, |p| matches!(p.b.state(), State::Closed(_)), 2000), "{:?}", p.b.state());
    }

    #[test]
    fn one_side_dials_the_other_listens() {
        let mut p = pair(false, true, "rust test", true, 5000);
        assert!(run(&mut p, |p| p.a.ready() && p.b.ready(), 4000), "a {:?} b {:?}", p.a.state(), p.b.state());
    }

    #[test]
    fn another_build_is_a_descriptor_mismatch() {
        let mut p = pair(true, true, "rust other", true, 5000);
        assert!(run(&mut p, |p| matches!(p.a.state(), State::Failed(_)) && matches!(p.b.state(), State::Failed(_)), 4000));
        let State::Failed(w) = p.a.state() else { unreachable!() };
        assert!(w.contains("differ") || w.contains("different"), "{w}");
    }

    #[test]
    fn a_stranger_is_never_accepted() {
        // b expects someone else: it refuses a's hello, so neither becomes ready, and both give up at the limit
        let mut p = pair(true, true, "rust test", false, 1200);
        assert!(run(&mut p, |p| matches!(p.a.state(), State::Failed(_)) && matches!(p.b.state(), State::Failed(_)), 3000));
        assert!(p.b.counters.hellos_refused >= 1);
        assert!(!p.a.ready() && !p.b.ready());
    }

    #[test]
    fn nobody_there_fails_at_the_limit_with_punches() {
        let (t, _) = sock();
        let (_dead, port) = sock(); // bound, never polled: nothing answers
        let now = Instant::now();
        let mut c = Connector::start(t, Identity::from_seed(&[1; 32]), desc("x"), 0, &[Addr::new(LO, port)], None, Duration::from_millis(600), now).unwrap();
        let end = Instant::now() + Duration::from_secs(2);
        while Instant::now() < end && !matches!(c.state(), State::Failed(_)) {
            c.poll(Instant::now());
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(matches!(c.state(), State::Failed(w) if w.contains("no connection")), "{:?}", c.state());
        assert!(c.counters.punches >= 4, "{}", c.counters.punches);
    }

    #[test]
    fn neighbour_ports_only_for_public_candidates() {
        let pubc = Addr::new(0x0102_0304, 5000);
        let n = neighbours(&[Addr::new(LO, 9), Addr::new(0xC0A8_0001, 9), pubc]);
        assert_eq!(n.iter().map(|a| a.port).collect::<Vec<_>>(), vec![5001, 5002, 5003, 4999]);
        assert!(!is_public(&Addr::new(0x6440_0001, 1)) && !is_public(&Addr::new(0xAC10_0001, 1)) && is_public(&Addr::new(0x0808_0808, 1)));
        let three = [pubc, Addr::new(0x0102_0305, 6000), Addr::new(0x0102_0306, 7000)];
        assert_eq!(neighbours(&three).len(), MAX_NEIGHBOURS);
    }
}
