//! Slippi Dolphin's online netplay messages and the peer-to-peer client that carries them over ENet
//! (docs/rust-port/notes/slippi-online.md section 1). Our own implementation of the protocol as Slippi Dolphin
//! `0653df819` speaks it (`SlippiNetplay.cpp`): `sf::Packet` encoding (big-endian integers, bool = 1 byte, string =
//! u32 length + bytes), three ENet channels (0 reliable for everything but the pads, 1 unsequenced for pads, acks
//! and suggestions, 2 unsequenced for the acks sent while handling a pad), a resend-until-acked local pad queue of
//! up to 128 frames, acks only for pads that added frames, ping from ack round trips and Slippi's time offset.
//!
//! Direct connections only: both sides know the other's address and connect to it at once (as Slippi does after
//! matchmaking). Nothing here talks to a matchmaking server, STUN or any Slippi service.

use crate::enet::{Event, Host, PeerState};
use crate::transport::{Addr, Transport};
use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;

pub const MSG_PAD: u8 = 0x80;
pub const MSG_PAD_ACK: u8 = 0x81;
pub const MSG_MATCH_SELECTIONS: u8 = 0x82;
pub const MSG_CONN_SELECTED: u8 = 0x83;
pub const MSG_CHAT_MESSAGE: u8 = 0x84;
pub const MSG_COMPLETE_STEP: u8 = 0x85;
pub const MSG_SYNCED_STATE: u8 = 0x86;
pub const MSG_PAD_SUGGESTION: u8 = 0x87;
pub const MSG_PLAYER_DROP: u8 = 0x88;

/// `SLIPPI_PAD_WIRE_SIZE`: the game's 8 pad bytes plus the owner's random byte.
pub const PAD_WIRE: usize = 9;
/// Pad packet header: id, s32 frame, u8 port, s32 checksum frame, u32 checksum.
pub const PAD_HEADER: usize = 14;
/// Most pads one packet may add (`OnData` rejects more).
pub const MAX_PADS_PER_PACKET: i64 = 128;
/// `SLIPPI_ONLINE_LOCKSTEP_INTERVAL`: offset samples kept per peer.
pub const OFFSET_SAMPLES: usize = 30;
/// One frame at 59.94 Hz in microseconds, as Slippi counts it.
pub const FRAME_US: i64 = 16683;

/// `sf::Packet` writer.
#[derive(Default, Clone, Debug)]
pub struct PacketW(pub Vec<u8>);
impl PacketW {
    pub fn u8(mut self, v: u8) -> Self {
        self.0.push(v);
        self
    }
    pub fn bool(self, v: bool) -> Self {
        self.u8(v as u8)
    }
    pub fn u16(mut self, v: u16) -> Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }
    pub fn u32(mut self, v: u32) -> Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }
    pub fn i32(self, v: i32) -> Self {
        self.u32(v as u32)
    }
    pub fn bytes(mut self, b: &[u8]) -> Self {
        self.0.extend_from_slice(b);
        self
    }
    pub fn string(self, s: &str) -> Self {
        self.u32(s.len() as u32).bytes(s.as_bytes())
    }
}

/// `sf::Packet` reader: every read fails (None) past the end, as `packet >> x` does.
pub struct PacketR<'a> {
    pub b: &'a [u8],
    pub o: usize,
}
impl<'a> PacketR<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        PacketR { b, o: 0 }
    }
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.o..self.o + n)?;
        self.o += n;
        Some(s)
    }
    pub fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|s| s[0])
    }
    pub fn bool(&mut self) -> Option<bool> {
        self.u8().map(|v| v != 0)
    }
    pub fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|s| u16::from_be_bytes([s[0], s[1]]))
    }
    pub fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
    pub fn i32(&mut self) -> Option<i32> {
        self.u32().map(|v| v as i32)
    }
    pub fn string(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        self.take(n).map(|s| String::from_utf8_lossy(s).into_owned())
    }
}

/// `SlippiPlayerSelections` as MATCH_SELECTIONS carries it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selections {
    pub character_id: u8,
    pub character_color: u8,
    pub is_character_selected: bool,
    pub player_idx: u8,
    pub stage_id: u16,
    pub is_stage_selected: bool,
    pub rng_offset: u32,
    pub team_id: u8,
    pub alt_stage_mode: u8,
}

impl Selections {
    pub fn encode(&self) -> Vec<u8> {
        PacketW::default()
            .u8(MSG_MATCH_SELECTIONS)
            .u8(self.character_id)
            .u8(self.character_color)
            .bool(self.is_character_selected)
            .u8(self.player_idx)
            .u16(self.stage_id)
            .bool(self.is_stage_selected)
            .u32(self.rng_offset)
            .u8(self.team_id)
            .u8(self.alt_stage_mode)
            .0
    }
    /// The body after the id byte.
    pub fn decode(r: &mut PacketR) -> Option<Selections> {
        Some(Selections {
            character_id: r.u8()?,
            character_color: r.u8()?,
            is_character_selected: r.bool()?,
            player_idx: r.u8()?,
            stage_id: r.u16()?,
            is_stage_selected: r.bool()?,
            rng_offset: r.u32()?,
            team_id: r.u8()?,
            alt_stage_mode: r.u8()?,
        })
    }
}

/// A SLIPPI_PAD packet: `pads[0]` is `frame`, `pads[i]` is `frame - i`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PadMsg {
    pub frame: i32,
    pub port: u8,
    pub checksum_frame: i32,
    pub checksum: u32,
    pub pads: Vec<[u8; PAD_WIRE]>,
}

impl PadMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = PacketW::default().u8(MSG_PAD).i32(self.frame).u8(self.port).i32(self.checksum_frame).u32(self.checksum);
        for p in &self.pads {
            w = w.bytes(p);
        }
        w.0
    }
    pub fn decode(b: &[u8]) -> Option<PadMsg> {
        let mut r = PacketR::new(b);
        if r.u8()? != MSG_PAD {
            return None;
        }
        let frame = r.i32()?;
        let port = r.u8()?;
        let checksum_frame = r.i32()?;
        let checksum = r.u32()?;
        let pads = b[PAD_HEADER..].chunks_exact(PAD_WIRE).map(|c| c.try_into().unwrap()).collect();
        Some(PadMsg { frame, port, checksum_frame, checksum, pads })
    }
}

pub fn encode_ack(frame: i32, port: u8) -> Vec<u8> {
    PacketW::default().u8(MSG_PAD_ACK).i32(frame).u8(port).0
}

/// `Note::Disconnected` reason when the peer address started a new match during ours (its process restarted).
pub const REASON_PEER_RESTARTED: u32 = 0xFFFF_FFF0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Initiated,
    Connected,
    Failed,
    Disconnected,
}

/// What the client tells its owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    Connected,
    Selections(Selections),
    /// A remote pad for (online frame); each frame once, in order.
    Pad { port: u8, frame: i32, pad: [u8; PAD_WIRE] },
    /// The peer's latest finalized checksum.
    Checksum { port: u8, frame: i32, value: u32 },
    Chat { port: u8, id: i32 },
    Disconnected { reason: u32 },
}

#[derive(Clone, Copy)]
struct Timing {
    frame: i32,
    time_us: i64,
}

/// One remote player's link and pad state (1v1: there is one).
struct Remote {
    addr: Addr,
    port: u8,
    /// The ENet peer slots we hold to this address (two when both sides connected at once).
    links: Vec<usize>,
    active: Option<usize>,
    head_frame: i32,
    last_acked: i32,
    ack_timers: VecDeque<Timing>,
    ping_us: i64,
    last_timing: Timing,
    offsets: Vec<i32>,
    offset_idx: usize,
    selections: Option<Selections>,
}

/// The netplay client (`SlippiNetplayClient`) for a direct 1v1: ENet host + Slippi messages.
pub struct Client {
    pub host: Host,
    pub local_port: u8,
    pub status: Status,
    remote: Remote,
    local_queue: VecDeque<(i32, [u8; PAD_WIRE], i32, u32)>,
    has_game_started: bool,
    /// Between `start_game` and the end of the match: the link set is fixed (see `on_disconnect`, `on_data`).
    in_game: bool,
    notes: VecDeque<Note>,
    t0: Instant,
    clock_us: Option<Box<dyn Fn() -> i64>>,
    started_at_us: i64,
    pub connect_timeout_ms: i64,
    /// Every remote (frame, checksum) seen, for the owner's frame-by-frame compare.
    pub remote_checksums: BTreeMap<i32, u32>,
    pub stats: ClientStats,
    /// When the last ENet ping for the displayed round trip went out (see [`Client::net_ping_us`]).
    last_probe_us: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClientStats {
    pub pads_sent: u64,
    pub pads_received: u64,
    pub acks_sent: u64,
    pub acks_received: u64,
    pub max_queue: usize,
}

impl Client {
    /// Bind on `tp`, connect to `remote` (the other player at port index `remote_port`).
    pub fn new(tp: Box<dyn Transport>, local_port: u8, remote: Addr, remote_port: u8) -> Client {
        let mut host = Host::new(tp, 10, 3);
        let first = host.connect(remote, 3, 0);
        let mut c = Client {
            host,
            local_port,
            status: Status::Initiated,
            remote: Remote {
                addr: remote,
                port: remote_port,
                links: first.into_iter().collect(),
                active: None,
                head_frame: 0,
                last_acked: 0,
                ack_timers: VecDeque::new(),
                ping_us: 0,
                last_timing: Timing { frame: 0, time_us: 0 },
                offsets: Vec::new(),
                offset_idx: 0,
                selections: None,
            },
            local_queue: VecDeque::new(),
            has_game_started: false,
            in_game: false,
            notes: VecDeque::new(),
            t0: Instant::now(),
            clock_us: None,
            started_at_us: 0,
            connect_timeout_ms: 8000,
            remote_checksums: BTreeMap::new(),
            stats: ClientStats::default(),
            last_probe_us: 0,
        };
        c.started_at_us = c.now_us();
        c
    }
    pub fn set_clock_us(&mut self, f: Box<dyn Fn() -> i64>) {
        self.clock_us = Some(f);
        self.started_at_us = self.now_us();
    }
    fn now_us(&self) -> i64 {
        match &self.clock_us {
            Some(f) => f(),
            None => self.t0.elapsed().as_micros() as i64,
        }
    }
    /// Slippi's ping (pad to pad-ack): what the time offset uses, so it keeps each side's wait for its frame poll.
    pub fn ping_us(&self) -> i64 {
        self.remote.ping_us
    }
    /// The network round trip for display and logs (0 until measured): ENet pings, timed from when each ack
    /// landed, with the echo moved forward by the peer's hold when the peer is this engine (Slippi Dolphin
    /// services ENet on its own thread, so its hold is already small). Never used for game timing.
    pub fn net_ping_us(&self) -> i64 {
        self.remote.active.and_then(|l| self.host.peers.get(l)).map_or(0, |p| (p.net_rtt * 1000.0) as i64)
    }
    pub fn remote_selections(&self) -> Option<Selections> {
        self.remote.selections
    }
    pub fn last_acked(&self) -> i32 {
        self.remote.last_acked
    }
    pub fn next_note(&mut self) -> Option<Note> {
        self.notes.pop_front()
    }

    fn send_to_active(&mut self, data: &[u8], channel: u8, reliable: bool) {
        if let Some(p) = self.remote.active {
            self.host.send(p, channel, data, reliable);
        }
    }
    /// `Send`: pads, acks and suggestions unsequenced on channel 1; the rest reliable on channel 0.
    pub fn send(&mut self, data: &[u8]) {
        let unseq = matches!(data.first(), Some(&MSG_PAD) | Some(&MSG_PAD_ACK) | Some(&MSG_PAD_SUGGESTION));
        self.send_to_active(data, if unseq { 1 } else { 0 }, !unseq);
    }

    /// `SetMatchSelections`.
    pub fn set_match_selections(&mut self, mut s: Selections) {
        s.player_idx = self.local_port;
        let b = s.encode();
        self.send(&b);
    }
    pub fn send_chat(&mut self, id: i32) {
        let b = PacketW::default().u8(MSG_CHAT_MESSAGE).i32(id).u8(self.local_port).0;
        self.send(&b);
    }

    /// `StartSlippiGame`: a new game's pad state.
    pub fn start_game(&mut self) {
        self.has_game_started = false;
        self.in_game = true;
        self.local_queue.clear();
        let now = self.now_us();
        self.remote.last_timing = Timing { frame: 0, time_us: now };
        self.remote.last_acked = 0;
        self.remote.ack_timers.clear();
    }

    /// `SendSlippiPad`: queue the local pad for `frame` (with the finalized checksum it carries) and send every
    /// unacked pad, newest first.
    pub fn send_pad(&mut self, frame: i32, pad: [u8; PAD_WIRE], checksum_frame: i32, checksum: u32) {
        if matches!(self.status, Status::Failed | Status::Disconnected) {
            return;
        }
        self.local_queue.push_front((frame, pad, checksum_frame, checksum));
        let mut min_ack = self.remote.last_acked;
        let cur = self.local_queue.front().unwrap().0;
        min_ack = min_ack.max(cur - 128);
        while self.local_queue.back().map_or(false, |b| b.0 < min_ack) {
            self.local_queue.pop_back();
        }
        if self.local_queue.is_empty() {
            return;
        }
        self.stats.max_queue = self.stats.max_queue.max(self.local_queue.len());
        let f = self.local_queue.front().unwrap();
        let m = PadMsg { frame: f.0, port: self.local_port, checksum_frame: f.2, checksum: f.3, pads: self.local_queue.iter().map(|q| q.1).collect() };
        let b = m.encode();
        self.send(&b);
        self.stats.pads_sent += 1;
        let now = self.now_us();
        self.has_game_started = true;
        self.remote.last_timing = Timing { frame: m.frame, time_us: now };
        self.remote.ack_timers.push_back(Timing { frame: m.frame, time_us: now });
    }

    /// `CalcTimeOffsetUs`: the mean of the middle third of the sorted samples (positive: we are ahead).
    pub fn time_offset_us(&self) -> i32 {
        let mut b = self.remote.offsets.clone();
        if b.is_empty() {
            return 0;
        }
        b.sort_unstable();
        let n = b.len();
        let off = n / 3;
        let end = n - off;
        if end <= off {
            return 0;
        }
        let sum: i64 = b[off..end].iter().map(|&x| x as i64).sum();
        (sum / (end - off) as i64) as i32
    }

    /// Run the network: every waiting ENet event, then the connect timeout. Call once per frame or more often.
    pub fn poll(&mut self) {
        while let Some(ev) = self.host.service() {
            if std::env::var_os("MELEE_NP_DEBUG").is_some() && !matches!(ev, Event::Receive { .. }) {
                eprintln!("np-debug t={}ms {:?} active={:?} links={:?} status={:?}", self.now_us() / 1000, ev, self.remote.active, self.remote.links, self.status);
            }
            match ev {
                Event::Connect { peer, .. } => self.on_connect(peer),
                Event::Disconnect { peer, data } => self.on_disconnect(peer, data),
                Event::Receive { peer, data, .. } => self.on_data(peer, &data),
            }
        }
        // an ENet ping every 250 ms (queued, it goes out with the next flush): the round trip on display
        if self.clock_us.is_none() && self.status == Status::Connected {
            let now = self.now_us();
            if now - self.last_probe_us >= 250_000 {
                if let Some(l) = self.remote.active {
                    self.host.ping(l);
                    self.last_probe_us = now;
                }
            }
        }
        if self.status == Status::Initiated && (self.now_us() - self.started_at_us) / 1000 >= self.connect_timeout_ms {
            self.status = Status::Failed;
        }
    }

    fn on_connect(&mut self, peer: usize) {
        if self.host.peers[peer].addr != self.remote.addr {
            // only the expected player may join a direct match
            self.host.disconnect(peer, 0);
            return;
        }
        // Slippi's disablePacketThrottleDrops: deceleration 0
        self.host.throttle_configure(peer, crate::enet::THROTTLE_INTERVAL, crate::enet::THROTTLE_ACCELERATION, 0);
        if !self.remote.links.contains(&peer) {
            self.remote.links.push(peer);
        }
        if self.remote.active.is_none() {
            // the link is up: our own CONNECT still being retried to the same address (it went out before the peer
            // was listening, so ENet resends it with backoff at ~0.5, 1.5, 3.5, 7.5 s...) is dropped silently,
            // else a late resend opens a second link mid-match that the duplicate-link rule then disconnects
            for l in 0..self.host.peers.len() {
                if l != peer && self.host.peers[l].state == PeerState::Connecting && self.host.peers[l].addr == self.remote.addr {
                    self.host.reset_peer(l);
                    self.remote.links.retain(|&x| x != l);
                }
            }
            self.remote.active = Some(peer);
            if self.status == Status::Initiated {
                self.status = Status::Connected;
                self.notes.push_back(Note::Connected);
            }
        }
    }

    fn live_links(&self) -> usize {
        self.remote.links.iter().filter(|&&l| self.host.peers[l].state == PeerState::Connected).count()
    }

    fn on_disconnect(&mut self, peer: usize, data: u32) {
        self.remote.links.retain(|&l| l != peer);
        // in a match the link the pads use going down ends the session, as in Slippi (its in-game DISCONNECT event): a
        // second link to the same address is then a different remote process (a restarted peer), not a fallback
        if self.remote.active == Some(peer) && self.in_game {
            self.remote.active = None;
        } else if self.remote.active == Some(peer) {
            self.remote.active = self.remote.links.iter().copied().find(|&l| self.host.peers[l].state == PeerState::Connected);
        }
        if self.remote.active.is_none() && self.status == Status::Connected {
            self.status = Status::Disconnected;
            self.notes.push_back(Note::Disconnected { reason: data });
        }
    }

    fn on_data(&mut self, peer: usize, b: &[u8]) {
        let Some(&id) = b.first() else { return };
        let mut r = PacketR::new(b);
        r.o = 1;
        match id {
            MSG_PAD => {
                let now = self.now_us();
                let Some(m) = PadMsg::decode(b) else { return };
                if m.port != self.remote.port {
                    return;
                }
                // duplicate links: the lower port keeps the link the pads arrive on and drops the others
                if self.live_links() > 1 && self.local_port < m.port {
                    self.remote.active = Some(peer);
                    for l in self.remote.links.clone() {
                        if l != peer && self.host.peers[l].state == PeerState::Connected {
                            self.host.disconnect(l, 0);
                        }
                    }
                }
                let rm = &mut self.remote;
                let mut timing = rm.last_timing;
                if !self.has_game_started {
                    timing = Timing { frame: 0, time_us: now };
                }
                let send_time = now - rm.ping_us / 2;
                let off = send_time - timing.time_us + FRAME_US * (timing.frame - m.frame) as i64;
                if rm.offsets.len() < OFFSET_SAMPLES {
                    rm.offsets.push(off as i32);
                } else {
                    rm.offsets[rm.offset_idx] = off as i32;
                }
                rm.offset_idx = (rm.offset_idx + 1) % OFFSET_SAMPLES;
                let to_copy = m.frame as i64 - rm.head_frame as i64;
                if to_copy > m.pads.len() as i64 || to_copy > MAX_PADS_PER_PACKET {
                    return;
                }
                for i in (0..to_copy.max(0)).rev() {
                    let f = m.frame - i as i32;
                    self.notes.push_back(Note::Pad { port: m.port, frame: f, pad: m.pads[i as usize] });
                    self.stats.pads_received += 1;
                }
                if to_copy > 0 {
                    rm.head_frame = m.frame;
                }
                self.remote_checksums.insert(m.checksum_frame, m.checksum);
                self.notes.push_back(Note::Checksum { port: m.port, frame: m.checksum_frame, value: m.checksum });
                if to_copy > 0 {
                    let a = encode_ack(m.frame, self.local_port);
                    self.host.send(peer, 2, &a, false);
                    self.stats.acks_sent += 1;
                }
            }
            MSG_PAD_ACK => {
                let (Some(frame), Some(port)) = (r.i32(), r.u8()) else { return };
                if port != self.remote.port {
                    return;
                }
                self.stats.acks_received += 1;
                let now = self.now_us();
                let rm = &mut self.remote;
                rm.last_acked = rm.last_acked.max(frame);
                while rm.ack_timers.front().map_or(false, |t| t.frame < frame) {
                    rm.ack_timers.pop_front();
                }
                if rm.ack_timers.front().map_or(false, |t| t.frame == frame) {
                    let t = rm.ack_timers.pop_front().unwrap();
                    rm.ping_us = now - t.time_us;
                }
            }
            MSG_MATCH_SELECTIONS => {
                let Some(s) = Selections::decode(&mut r) else { return };
                if s.player_idx != self.remote.port {
                    return;
                }
                if self.in_game {
                    // selections in a match: the remote is a new process starting its own match (a restart on the same
                    // address); this match's peer is gone
                    for l in self.remote.links.clone() {
                        self.host.disconnect(l, 0);
                    }
                    self.remote.active = None;
                    if self.status == Status::Connected {
                        self.status = Status::Disconnected;
                        self.notes.push_back(Note::Disconnected { reason: REASON_PEER_RESTARTED });
                    }
                    return;
                }
                self.remote.selections = Some(s);
                self.has_game_started = false;
                self.remote.head_frame = 0;
                self.notes.push_back(Note::Selections(s));
            }
            MSG_CHAT_MESSAGE => {
                if let (Some(id), Some(port)) = (r.i32(), r.u8()) {
                    self.notes.push_back(Note::Chat { port, id });
                }
            }
            _ => {} // CONN_SELECTED, game-prep steps, synced state, suggestions, drops: not used in a direct 1v1
        }
    }

    /// `Disconnect`: tell the peer (with the reason word) on every link.
    pub fn disconnect(&mut self, reason: u32) {
        for l in self.remote.links.clone() {
            self.host.disconnect(l, reason);
        }
        self.host.flush();
        self.status = Status::Disconnected;
    }
}

/// The 8 game pad bytes (`SLIPPI_PAD_INPUT_SIZE`) of an input: buttons u16 BE, stick x/y, c-stick x/y, L, R.
pub fn pad_bytes(i: &crate::pad::RbInput) -> [u8; 8] {
    [(i.buttons >> 8) as u8, i.buttons as u8, i.raw[0] as u8, i.raw[1] as u8, i.raw[2] as u8, i.raw[3] as u8, i.pad_l, i.pad_r]
}

/// A received pad as a session input (present, confirmed, raw controller).
pub fn pad_input(b: &[u8]) -> crate::pad::RbInput {
    crate::pad::RbInput {
        present: 1,
        is_raw: 1,
        confirmed: 1,
        buttons: ((b[0] as u32) << 8) | b[1] as u32,
        raw: [b[2] as i8, b[3] as i8, b[4] as i8, b[5] as i8],
        pad_l: b[6],
        pad_r: b[7],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selections_layout() {
        let s = Selections { character_id: 2, character_color: 1, is_character_selected: true, player_idx: 1, stage_id: 0x20, is_stage_selected: true, rng_offset: 0x1234, team_id: 0, alt_stage_mode: 0 };
        let b = s.encode();
        assert_eq!(b, vec![0x82, 2, 1, 1, 1, 0, 0x20, 1, 0, 0, 0x12, 0x34, 0, 0]);
        let mut r = PacketR::new(&b);
        r.o = 1;
        assert_eq!(Selections::decode(&mut r), Some(s));
    }

    #[test]
    fn pad_layout() {
        let m = PadMsg { frame: 5, port: 0, checksum_frame: 3, checksum: 0xAABBCCDD, pads: vec![[1; 9], [2; 9]] };
        let b = m.encode();
        assert_eq!(&b[..PAD_HEADER], &[0x80, 0, 0, 0, 5, 0, 0, 0, 0, 3, 0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(b.len(), PAD_HEADER + 18);
        assert_eq!(PadMsg::decode(&b), Some(m));
        assert_eq!(encode_ack(7, 1), vec![0x81, 0, 0, 0, 7, 1]);
    }
}
