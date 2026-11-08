//! An ENet 1.3 host: the wire protocol Slippi Dolphin's netplay runs on (lsalzman/enet 1.3.x; Slippi Dolphin
//! `0653df819` ships 1.3.17, `extern/enet` is 1.3.18, the protocol is the same). Our own implementation of the
//! protocol, written from the wire format (`extern/enet/include/enet/protocol.h`) and the behaviour a 1.3 peer
//! expects; no code is shared with the C library.
//!
//! What is implemented, and what Slippi needs:
//! - the connect / verify-connect handshake with session ids, simultaneous connects in both directions (each side
//!   then holds two links to the other; the Slippi layer prunes one), disconnect with a reason word;
//! - reliable, in-order packets per channel (with acks, retransmission, rtt), unsequenced packets (the Slippi pads
//!   and acks) with the 1024-group duplicate window, unreliable packets (delivered as they come);
//! - ping (reliable on the peer channel 0xFF), bandwidth-limit and throttle-configure commands (accepted and
//!   stored: Slippi sends a throttle-configure with deceleration 0 on connect).
//!
//! Not implemented: compression and the optional checksum (Slippi uses neither), fragments (nothing Slippi sends
//! is larger than one datagram; an incoming fragment is skipped), the bandwidth throttle (we never drop our own
//! unreliable traffic, which is what Slippi's deceleration-0 setting asks of a peer anyway).
//!
//! Like the rest of this crate it runs from `service` calls on the caller's thread: no threads, no blocking. All
//! integers on the wire are big-endian except the connect id, which ENet sends in host order (opaque, echoed).

use crate::transport::{Addr, Transport};
use std::collections::{BTreeMap, VecDeque};

pub const MAX_PEER_ID: u16 = 0xFFF;
pub const DEFAULT_MTU: u32 = 1392;
pub const MIN_MTU: u32 = 576;
pub const MAX_MTU: u32 = 4096;
pub const MIN_WINDOW: u32 = 4096;
pub const MAX_WINDOW: u32 = 65536;
pub const THROTTLE_INTERVAL: u32 = 5000;
pub const THROTTLE_ACCELERATION: u32 = 2;
pub const THROTTLE_DECELERATION: u32 = 2;
pub const PING_INTERVAL_MS: u32 = 500;
/// No ack for a reliable command this long: the peer is gone (ENet's `ENET_PEER_TIMEOUT_MAXIMUM`).
pub const TIMEOUT_MS: u32 = 30000;
const UNSEQ_WINDOW_SIZE: u32 = 1024;
const FREE_UNSEQ_WINDOWS: u32 = 32;

pub const CMD_ACK: u8 = 1;
pub const CMD_CONNECT: u8 = 2;
pub const CMD_VERIFY_CONNECT: u8 = 3;
pub const CMD_DISCONNECT: u8 = 4;
pub const CMD_PING: u8 = 5;
pub const CMD_SEND_RELIABLE: u8 = 6;
pub const CMD_SEND_UNRELIABLE: u8 = 7;
pub const CMD_SEND_FRAGMENT: u8 = 8;
pub const CMD_SEND_UNSEQUENCED: u8 = 9;
pub const CMD_BANDWIDTH_LIMIT: u8 = 10;
pub const CMD_THROTTLE_CONFIGURE: u8 = 11;
pub const CMD_SEND_UNRELIABLE_FRAGMENT: u8 = 12;
pub const FLAG_ACK: u8 = 0x80;
pub const FLAG_UNSEQUENCED: u8 = 0x40;
const HDR_SENT_TIME: u16 = 0x8000;
const HDR_COMPRESSED: u16 = 0x4000;
const HDR_SESSION_MASK: u16 = 0x3000;
const HDR_SESSION_SHIFT: u16 = 12;

/// The fixed size of each command (header included), by command number.
const CMD_SIZES: [usize; 13] = [0, 8, 48, 44, 8, 4, 6, 8, 24, 8, 12, 16, 24];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerState {
    Disconnected,
    /// We sent CONNECT and wait for VERIFY_CONNECT.
    Connecting,
    /// We got CONNECT, sent VERIFY_CONNECT and wait for its ack.
    AcknowledgingConnect,
    Connected,
    /// We sent DISCONNECT and wait for its ack.
    Disconnecting,
    /// Gone; the event is pending and the slot is freed on the next service.
    Zombie,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Connect { peer: usize, data: u32 },
    Disconnect { peer: usize, data: u32 },
    Receive { peer: usize, channel: u8, data: Vec<u8> },
}

#[derive(Clone, Default)]
struct Channel {
    out_reliable: u16,
    out_unreliable: u16,
    in_reliable: u16,
    /// Reliable packets that arrived ahead of `in_reliable + 1`.
    pending: BTreeMap<u16, Vec<u8>>,
}

#[derive(Clone)]
struct SentReliable {
    channel: u8,
    seq: u16,
    command: u8,
    bytes: Vec<u8>,
    first_sent: u32,
    next_send: u32,
    timeout: u32,
}

#[derive(Clone)]
pub struct Peer {
    pub state: PeerState,
    pub addr: Addr,
    pub outgoing_peer_id: u16,
    in_session: u8,
    out_session: u8,
    connect_id: [u8; 4],
    pub mtu: u32,
    window: u32,
    channels: Vec<Channel>,
    peer_reliable_seq: u16,
    out_unseq_group: u16,
    in_unseq_group: u16,
    unseq_window: [u32; (UNSEQ_WINDOW_SIZE / 32) as usize],
    sent: Vec<SentReliable>,
    /// Commands queued for the next datagram (sent once, no ack).
    out_once: Vec<Vec<u8>>,
    /// Reliable commands not yet sent the first time.
    out_reliable: VecDeque<SentReliable>,
    /// (channel, seq, the peer's send time, command, when its datagram landed here)
    acks: Vec<(u8, u16, u16, u8, u32)>,
    /// The network round trip (ms, smoothed; 0 until measured), for display: from the landing time of each ack, so
    /// without our own wait for the service call. Retransmission keeps using `rtt`.
    pub net_rtt: f32,
    /// Smoothed round-trip time and variance (ms), from acks.
    pub rtt: u32,
    pub rtt_var: u32,
    last_ack_time: u32,
    pub last_receive: u32,
    pub throttle: (u32, u32, u32),
    pub event_data: u32,
    /// After sending the ack of the peer's DISCONNECT, go to zombie.
    zombie_after_flush: bool,
    pub stats_sent: u64,
    pub stats_received: u64,
    pub stats_resent: u64,
}

impl Peer {
    fn new() -> Peer {
        Peer {
            state: PeerState::Disconnected,
            addr: Addr::default(),
            outgoing_peer_id: MAX_PEER_ID,
            in_session: 0xFF,
            out_session: 0xFF,
            connect_id: [0; 4],
            mtu: DEFAULT_MTU,
            window: MAX_WINDOW,
            channels: Vec::new(),
            peer_reliable_seq: 0,
            out_unseq_group: 0,
            in_unseq_group: 0,
            unseq_window: [0; (UNSEQ_WINDOW_SIZE / 32) as usize],
            sent: Vec::new(),
            out_once: Vec::new(),
            out_reliable: VecDeque::new(),
            acks: Vec::new(),
            net_rtt: 0.0,
            rtt: 500,
            rtt_var: 0,
            last_ack_time: 0,
            last_receive: 0,
            throttle: (THROTTLE_INTERVAL, THROTTLE_ACCELERATION, THROTTLE_DECELERATION),
            event_data: 0,
            zombie_after_flush: false,
            stats_sent: 0,
            stats_received: 0,
            stats_resent: 0,
        }
    }
    /// Back to a free slot, keeping the session ids (ENet keeps them across a reset so the next link differs).
    fn reset(&mut self) {
        let (i, o) = (self.in_session, self.out_session);
        *self = Peer::new();
        self.in_session = i;
        self.out_session = o;
    }
    pub fn connected(&self) -> bool {
        self.state == PeerState::Connected
    }
    fn has_outgoing(&self) -> bool {
        !self.sent.is_empty() || !self.out_reliable.is_empty() || !self.out_once.is_empty()
    }
}

fn be16(b: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([b[o], b[o + 1]])
}
fn be32(b: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// One ENet host (`ENetHost`): a socket, a fixed table of peer slots, and the queued events.
pub struct Host {
    tp: Box<dyn Transport>,
    pub peers: Vec<Peer>,
    channel_limit: usize,
    clock: Option<Box<dyn Fn() -> u32>>,
    t0: std::time::Instant,
    /// When the datagram being handled landed (host ms; 0 = unknown).
    cur_rx: u32,
    rng: u64,
    events: VecDeque<Event>,
    /// Count of datagrams this host could not parse or had to drop (bad peer id, session, address).
    pub dropped: u64,
}

impl Host {
    /// `enet_host_create(addr, peer_count, channel_limit, 0, 0)` over an open transport.
    pub fn new(tp: Box<dyn Transport>, peer_count: usize, channel_limit: usize) -> Host {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        Host {
            tp,
            peers: (0..peer_count.clamp(1, MAX_PEER_ID as usize)).map(|_| Peer::new()).collect(),
            channel_limit: channel_limit.clamp(1, 255),
            clock: None,
            t0: std::time::Instant::now(),
            cur_rx: 0,
            rng: seed | 1,
            events: VecDeque::new(),
            dropped: 0,
        }
    }
    /// A virtual millisecond clock (tests); the default is the wall clock since creation.
    pub fn set_clock(&mut self, clock: Box<dyn Fn() -> u32>) {
        self.clock = Some(clock);
    }
    pub fn set_seed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }
    pub fn local_port(&self) -> u16 {
        self.tp.local_port()
    }
    fn now(&self) -> u32 {
        match &self.clock {
            Some(c) => c().max(1),
            None => (self.t0.elapsed().as_millis() as u32).wrapping_add(1).max(1),
        }
    }
    fn random(&mut self) -> u32 {
        // xorshift64*
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
    }

    /// `enet_host_connect`: a peer slot in `Connecting`, or None when every slot is in use.
    pub fn connect(&mut self, addr: Addr, channels: usize, data: u32) -> Option<usize> {
        let i = self.peers.iter().position(|p| p.state == PeerState::Disconnected)?;
        let channels = channels.clamp(1, 255);
        let cid = self.random().to_le_bytes();
        let p = &mut self.peers[i];
        p.state = PeerState::Connecting;
        p.addr = addr;
        p.connect_id = cid;
        p.mtu = DEFAULT_MTU;
        p.window = MAX_WINDOW;
        p.channels = vec![Channel::default(); channels];
        let mut c = vec![CMD_CONNECT | FLAG_ACK, 0xFF, 0, 0];
        c.extend_from_slice(&(i as u16).to_be_bytes());
        c.push(p.in_session);
        c.push(p.out_session);
        for v in [p.mtu, p.window, channels as u32, 0, 0, p.throttle.0, p.throttle.1, p.throttle.2] {
            c.extend_from_slice(&v.to_be_bytes());
        }
        c.extend_from_slice(&cid);
        c.extend_from_slice(&data.to_be_bytes());
        Self::queue_reliable(p, 0xFF, c);
        Some(i)
    }

    /// Puts a reliable command (header included, sequence number still 0) in the send queue, numbering it.
    fn queue_reliable(p: &mut Peer, channel: u8, mut bytes: Vec<u8>) {
        let seq = if channel == 0xFF {
            p.peer_reliable_seq = p.peer_reliable_seq.wrapping_add(1);
            p.peer_reliable_seq
        } else {
            let ch = &mut p.channels[channel as usize];
            ch.out_reliable = ch.out_reliable.wrapping_add(1);
            ch.out_unreliable = 0;
            ch.out_reliable
        };
        bytes[2..4].copy_from_slice(&seq.to_be_bytes());
        let command = bytes[0] & 0x0F;
        p.out_reliable.push_back(SentReliable { channel, seq, command, bytes, first_sent: 0, next_send: 0, timeout: 0 });
    }

    /// `enet_peer_send`: `reliable` = ENET_PACKET_FLAG_RELIABLE, else ENET_PACKET_FLAG_UNSEQUENCED (what Slippi uses
    /// for its pads and acks). False when the peer is not connected, the channel does not exist, or it is too big.
    pub fn send(&mut self, peer: usize, channel: u8, data: &[u8], reliable: bool) -> bool {
        let Some(p) = self.peers.get_mut(peer) else { return false };
        if p.state != PeerState::Connected || channel as usize >= p.channels.len() || data.len() + 12 > p.mtu as usize {
            return false;
        }
        if reliable {
            let mut c = vec![CMD_SEND_RELIABLE | FLAG_ACK, channel, 0, 0];
            c.extend_from_slice(&(data.len() as u16).to_be_bytes());
            c.extend_from_slice(data);
            Self::queue_reliable(p, channel, c);
        } else {
            p.out_unseq_group = p.out_unseq_group.wrapping_add(1);
            let mut c = vec![CMD_SEND_UNSEQUENCED | FLAG_UNSEQUENCED, channel, 0, 0];
            c.extend_from_slice(&p.out_unseq_group.to_be_bytes());
            c.extend_from_slice(&(data.len() as u16).to_be_bytes());
            c.extend_from_slice(data);
            p.out_once.push(c);
        }
        true
    }

    /// `enet_peer_throttle_configure`: stores the values and tells the peer (reliable, peer channel).
    pub fn throttle_configure(&mut self, peer: usize, interval: u32, accel: u32, decel: u32) {
        let Some(p) = self.peers.get_mut(peer) else { return };
        if p.state != PeerState::Connected {
            return;
        }
        p.throttle = (interval, accel, decel);
        let mut c = vec![CMD_THROTTLE_CONFIGURE | FLAG_ACK, 0xFF, 0, 0];
        for v in [interval, accel, decel] {
            c.extend_from_slice(&v.to_be_bytes());
        }
        Self::queue_reliable(p, 0xFF, c);
    }

    /// `enet_peer_disconnect`: a connected peer gets an acknowledged DISCONNECT and a Disconnect event once it is
    /// acked (or times out); any other state is dropped at once with an unsequenced DISCONNECT.
    pub fn disconnect(&mut self, peer: usize, data: u32) {
        let now = self.now();
        let Some(p) = self.peers.get_mut(peer) else { return };
        match p.state {
            PeerState::Disconnected | PeerState::Disconnecting | PeerState::Zombie => return,
            _ => {}
        }
        p.sent.clear();
        p.out_reliable.clear();
        p.out_once.clear();
        let mut c = vec![CMD_DISCONNECT, 0xFF, 0, 0];
        c.extend_from_slice(&data.to_be_bytes());
        if p.state == PeerState::Connected {
            c[0] |= FLAG_ACK;
            Self::queue_reliable(p, 0xFF, c);
            p.state = PeerState::Disconnecting;
            p.event_data = data;
        } else {
            c[0] |= FLAG_UNSEQUENCED;
            p.out_once.push(c);
            self.flush_peer(peer, now);
            self.peers[peer].reset();
        }
    }

    /// `enet_peer_reset`: forget the peer without telling it.
    pub fn reset_peer(&mut self, peer: usize) {
        if let Some(p) = self.peers.get_mut(peer) {
            p.reset();
        }
    }

    /// `enet_host_service` with a zero timeout: read every waiting datagram, run the timers, send what is queued,
    /// and return the next event.
    pub fn service(&mut self) -> Option<Event> {
        if let Some(e) = self.events.pop_front() {
            return Some(e);
        }
        let mut buf = vec![0u8; 4096];
        for _ in 0..256 {
            let mut from = Addr::default();
            crate::transport::set_rx_stamp(None);
            let n = self.tp.recv(&mut from, &mut buf);
            if n <= 0 {
                break;
            }
            let d = buf[..n as usize].to_vec();
            let now = self.now();
            // when the datagram landed (0: unknown, no transport stamp or a test clock: acks then echo unchanged)
            self.cur_rx = match crate::transport::rx_stamp() {
                Some(_) if self.clock.is_none() => now.wrapping_sub(crate::transport::rx_wait_ms()).max(1),
                _ => 0,
            };
            self.handle_datagram(from, &d);
        }
        let now = self.now();
        for i in 0..self.peers.len() {
            self.timers(i, now);
            self.flush_peer(i, now);
        }
        self.events.pop_front()
    }

    /// Queue an ENet PING (reliable, acked by any ENet peer, no event) when nothing reliable is in flight: during a
    /// match nothing else is acked, so this is what keeps the displayed round trip ([`Peer::net_rtt`]) fresh. Goes
    /// out with the next flush.
    pub fn ping(&mut self, peer: usize) {
        if let Some(p) = self.peers.get_mut(peer) {
            if p.state == PeerState::Connected && p.sent.is_empty() && p.out_reliable.is_empty() {
                Self::queue_reliable(p, 0xFF, vec![CMD_PING | FLAG_ACK, 0xFF, 0, 0]);
            }
        }
    }

    /// `enet_host_flush`: send what is queued without reading.
    pub fn flush(&mut self) {
        let now = self.now();
        for i in 0..self.peers.len() {
            self.flush_peer(i, now);
        }
    }

    fn timers(&mut self, i: usize, now: u32) {
        let p = &mut self.peers[i];
        match p.state {
            PeerState::Disconnected => return,
            PeerState::Zombie => {
                p.reset();
                return;
            }
            _ => {}
        }
        // a reliable command unacked for too long: the peer is gone
        if let Some(s) = p.sent.iter().find(|s| s.first_sent != 0) {
            if now.wrapping_sub(s.first_sent) >= TIMEOUT_MS && (now.wrapping_sub(s.first_sent) as i32) > 0 {
                let was = p.state;
                let data = p.event_data;
                p.reset();
                if matches!(was, PeerState::Connected | PeerState::Disconnecting | PeerState::AcknowledgingConnect) {
                    self.events.push_back(Event::Disconnect { peer: i, data: if was == PeerState::Disconnecting { data } else { 0 } });
                }
                return;
            }
        }
        // keep-alive: ENet pings every interval when nothing reliable is in flight
        if p.state == PeerState::Connected && p.sent.is_empty() && p.out_reliable.is_empty() && now.wrapping_sub(p.last_ack_time.max(p.last_receive)) >= PING_INTERVAL_MS {
            p.last_ack_time = now;
            Self::queue_reliable(p, 0xFF, vec![CMD_PING | FLAG_ACK, 0xFF, 0, 0]);
        }
    }

    fn header(p: &Peer, now: u32) -> Vec<u8> {
        let mut id = p.outgoing_peer_id;
        if p.outgoing_peer_id < MAX_PEER_ID {
            id |= ((p.out_session as u16) << HDR_SESSION_SHIFT) & HDR_SESSION_MASK;
        }
        id |= HDR_SENT_TIME;
        let mut h = id.to_be_bytes().to_vec();
        h.extend_from_slice(&(now as u16).to_be_bytes());
        h
    }

    fn flush_peer(&mut self, i: usize, now: u32) {
        let p = &mut self.peers[i];
        if p.state == PeerState::Disconnected {
            return;
        }
        let mtu = p.mtu as usize;
        let mut cmds: Vec<Vec<u8>> = Vec::new();
        for (ch, seq, sent_time, _cmd, rx) in p.acks.drain(..) {
            let mut c = vec![CMD_ACK, ch, 0, 0];
            c[2..4].copy_from_slice(&seq.to_be_bytes());
            c.extend_from_slice(&seq.to_be_bytes());
            // the echo moved forward by how long we held the command (landing to this flush), NTP style, so the
            // peer's round trip is the network's and not our frame poll's; never past now, so ENet's sanity check
            // (an echo from the future is dropped) holds
            let hold = now.wrapping_sub(rx);
            let hold = if rx != 0 && (hold as i32) > 0 && hold < 0x8000 { hold as u16 } else { 0 };
            c.extend_from_slice(&sent_time.wrapping_add(hold).to_be_bytes());
            cmds.push(c);
        }
        // resends that are due
        for s in p.sent.iter_mut() {
            if (now.wrapping_sub(s.next_send) as i32) >= 0 {
                s.timeout = (s.timeout * 2).min(4000);
                s.next_send = now.wrapping_add(s.timeout);
                cmds.push(s.bytes.clone());
                p.stats_resent += 1;
            }
        }
        while let Some(mut s) = p.out_reliable.pop_front() {
            s.first_sent = now;
            s.timeout = (p.rtt + 4 * p.rtt_var).clamp(50, 4000);
            s.next_send = now.wrapping_add(s.timeout);
            cmds.push(s.bytes.clone());
            p.sent.push(s);
        }
        cmds.append(&mut p.out_once);
        if cmds.is_empty() {
            return;
        }
        let header = Self::header(p, now);
        let mut dgram = header.clone();
        let mut n = 0u64;
        let mut out = Vec::new();
        for c in cmds {
            if dgram.len() + c.len() > mtu && dgram.len() > header.len() {
                out.push(std::mem::replace(&mut dgram, header.clone()));
            }
            dgram.extend_from_slice(&c);
            n += 1;
        }
        out.push(dgram);
        p.stats_sent += n;
        let addr = p.addr;
        let z = p.zombie_after_flush;
        for d in &out {
            self.tp.send(&addr, d);
        }
        if z {
            let p = &mut self.peers[i];
            p.zombie_after_flush = false;
            p.state = PeerState::Zombie;
        }
    }

    fn handle_datagram(&mut self, from: Addr, d: &[u8]) {
        let rx = self.cur_rx;
        if d.len() < 2 {
            self.dropped += 1;
            return;
        }
        let raw = be16(d, 0);
        let flags = raw & (HDR_SENT_TIME | HDR_COMPRESSED);
        let session = ((raw & HDR_SESSION_MASK) >> HDR_SESSION_SHIFT) as u8;
        let peer_id = raw & !(HDR_SENT_TIME | HDR_COMPRESSED | HDR_SESSION_MASK);
        let hsize = if flags & HDR_SENT_TIME != 0 { 4 } else { 2 };
        if d.len() < hsize || flags & HDR_COMPRESSED != 0 {
            self.dropped += 1;
            return;
        }
        let sent_time = if hsize == 4 { Some(be16(d, 2)) } else { None };
        let mut peer: Option<usize> = None;
        if peer_id != MAX_PEER_ID {
            let i = peer_id as usize;
            let ok = self.peers.get(i).map_or(false, |p| {
                !matches!(p.state, PeerState::Disconnected | PeerState::Zombie)
                    && p.addr == from
                    && !(p.outgoing_peer_id < MAX_PEER_ID && session != p.in_session)
            });
            if !ok {
                self.dropped += 1;
                return;
            }
            peer = Some(i);
        }
        let now = self.now();
        if let Some(i) = peer {
            self.peers[i].stats_received += 1;
        }
        let mut o = hsize;
        while o + 4 <= d.len() {
            let cmd = d[o];
            let num = cmd & 0x0F;
            if num as usize >= CMD_SIZES.len() || CMD_SIZES[num as usize] == 0 || o + CMD_SIZES[num as usize] > d.len() {
                break;
            }
            let c = &d[o..o + CMD_SIZES[num as usize]];
            let channel = c[1];
            let seq = be16(c, 2);
            o += CMD_SIZES[num as usize];
            if peer.is_none() && num != CMD_CONNECT {
                break;
            }
            // the data that follows a send command
            let mut payload: &[u8] = &[];
            let dlen_at = match num {
                CMD_SEND_RELIABLE => Some(4),
                CMD_SEND_UNRELIABLE | CMD_SEND_UNSEQUENCED | CMD_SEND_FRAGMENT | CMD_SEND_UNRELIABLE_FRAGMENT => Some(6),
                _ => None,
            };
            if let Some(at) = dlen_at {
                let n = be16(c, at) as usize;
                if o + n > d.len() {
                    break;
                }
                payload = &d[o..o + n];
                o += n;
            }
            let ok = match num {
                CMD_CONNECT => {
                    if peer.is_some() {
                        false
                    } else {
                        peer = self.handle_connect(from, c);
                        peer.is_some()
                    }
                }
                _ => {
                    let i = peer.unwrap();
                    self.handle_command(i, num, channel, seq, c, payload, now)
                }
            };
            if !ok {
                break;
            }
            if let (Some(i), true) = (peer, cmd & FLAG_ACK != 0) {
                let Some(st) = sent_time else { break };
                let p = &mut self.peers[i];
                match p.state {
                    PeerState::Disconnecting | PeerState::AcknowledgingConnect | PeerState::Disconnected | PeerState::Zombie => {}
                    _ => p.acks.push((channel, seq, st, num, rx)),
                }
            }
        }
    }

    fn handle_connect(&mut self, from: Addr, c: &[u8]) -> Option<usize> {
        let channels = be32(c, 16) as usize;
        if !(1..=255).contains(&channels) {
            return None;
        }
        let cid = [c[40], c[41], c[42], c[43]];
        let mut free = None;
        for (i, p) in self.peers.iter().enumerate() {
            if p.state == PeerState::Disconnected {
                if free.is_none() {
                    free = Some(i);
                }
            } else if p.state != PeerState::Connecting && p.addr == from && p.connect_id == cid {
                return None; // a resent CONNECT for a link we already accepted
            }
        }
        let i = free?;
        let channels = channels.min(self.channel_limit);
        let p = &mut self.peers[i];
        p.channels = vec![Channel::default(); channels];
        p.state = PeerState::AcknowledgingConnect;
        p.connect_id = cid;
        p.addr = from;
        p.mtu = DEFAULT_MTU;
        p.outgoing_peer_id = be16(c, 4);
        p.throttle = (be32(c, 28), be32(c, 32), be32(c, 36));
        p.event_data = be32(c, 44);
        let in_s = c[6];
        let out_s = c[7];
        let mut incoming = if in_s == 0xFF { p.out_session } else { in_s };
        incoming = incoming.wrapping_add(1) & 3;
        if incoming == p.out_session {
            incoming = incoming.wrapping_add(1) & 3;
        }
        p.out_session = incoming;
        let mut outgoing = if out_s == 0xFF { p.in_session } else { out_s };
        outgoing = outgoing.wrapping_add(1) & 3;
        if outgoing == p.in_session {
            outgoing = outgoing.wrapping_add(1) & 3;
        }
        p.in_session = outgoing;
        let mtu = be32(c, 8).clamp(MIN_MTU, MAX_MTU);
        if mtu < p.mtu {
            p.mtu = mtu;
        }
        p.window = MAX_WINDOW;
        let window = be32(c, 12).clamp(MIN_WINDOW, MAX_WINDOW).min(MAX_WINDOW);
        let mut v = vec![CMD_VERIFY_CONNECT | FLAG_ACK, 0xFF, 0, 0];
        v.extend_from_slice(&(i as u16).to_be_bytes());
        v.push(incoming);
        v.push(outgoing);
        for x in [p.mtu, window, channels as u32, 0, 0, p.throttle.0, p.throttle.1, p.throttle.2] {
            v.extend_from_slice(&x.to_be_bytes());
        }
        v.extend_from_slice(&cid);
        Self::queue_reliable(p, 0xFF, v);
        Some(i)
    }

    /// One command for a known peer; false stops the datagram (ENet's commandError).
    #[allow(clippy::too_many_arguments)]
    fn handle_command(&mut self, i: usize, num: u8, channel: u8, seq: u16, c: &[u8], payload: &[u8], now: u32) -> bool {
        let rx = self.cur_rx;
        let p = &mut self.peers[i];
        let live = matches!(p.state, PeerState::Connected);
        match num {
            CMD_ACK => {
                if matches!(p.state, PeerState::Disconnected | PeerState::Zombie) {
                    return true;
                }
                let rseq = be16(c, 4);
                let rtime = be16(c, 6) as u32;
                // the 16-bit echo of our send time, widened against now
                let mut t = (now & 0xFFFF_0000) | rtime;
                if (t & 0x8000) > (now & 0x8000) {
                    t = t.wrapping_sub(0x10000);
                }
                if (now.wrapping_sub(t) as i32) >= 0 {
                    let r = now.wrapping_sub(t).max(1);
                    if p.last_ack_time == 0 {
                        p.rtt = r;
                        p.rtt_var = (r + 1) / 2;
                    } else {
                        p.rtt_var -= p.rtt_var / 4;
                        if r >= p.rtt {
                            let d = r - p.rtt;
                            p.rtt_var += d / 4;
                            p.rtt += d / 8;
                        } else {
                            let d = p.rtt - r;
                            p.rtt_var += d / 4;
                            p.rtt -= d / 8;
                        }
                    }
                    p.last_ack_time = now;
                }
                if rx != 0 {
                    if let Some(r) = ack_rtt_ms(rx, rtime) {
                        p.net_rtt = if p.net_rtt == 0.0 { r as f32 } else { p.net_rtt * 0.875 + r as f32 * 0.125 };
                    }
                }
                p.last_receive = now;
                let mut acked = None;
                if let Some(k) = p.sent.iter().position(|s| s.seq == rseq && s.channel == channel) {
                    acked = Some(p.sent.remove(k).command);
                }
                match p.state {
                    PeerState::AcknowledgingConnect => {
                        if acked != Some(CMD_VERIFY_CONNECT) {
                            return false;
                        }
                        p.state = PeerState::Connected;
                        self.events.push_back(Event::Connect { peer: i, data: 0 });
                    }
                    PeerState::Disconnecting => {
                        if acked != Some(CMD_DISCONNECT) {
                            return false;
                        }
                        let data = p.event_data;
                        p.reset();
                        self.events.push_back(Event::Disconnect { peer: i, data });
                    }
                    _ => {}
                }
                true
            }
            CMD_VERIFY_CONNECT => {
                if p.state != PeerState::Connecting {
                    return true;
                }
                let channels = be32(c, 16) as usize;
                let cid = [c[40], c[41], c[42], c[43]];
                if !(1..=255).contains(&channels) || (be32(c, 28), be32(c, 32), be32(c, 36)) != p.throttle || cid != p.connect_id {
                    p.reset();
                    return false;
                }
                p.sent.retain(|s| !(s.channel == 0xFF && s.seq == 1));
                if channels < p.channels.len() {
                    p.channels.truncate(channels);
                }
                p.outgoing_peer_id = be16(c, 4);
                p.in_session = c[6];
                p.out_session = c[7];
                let mtu = be32(c, 8).clamp(MIN_MTU, MAX_MTU);
                if mtu < p.mtu {
                    p.mtu = mtu;
                }
                let w = be32(c, 12).clamp(MIN_WINDOW, MAX_WINDOW);
                if w < p.window {
                    p.window = w;
                }
                p.state = PeerState::Connected;
                p.last_receive = now;
                self.events.push_back(Event::Connect { peer: i, data: 0 });
                true
            }
            CMD_DISCONNECT => {
                if matches!(p.state, PeerState::Disconnected | PeerState::Zombie) {
                    return true;
                }
                let data = be32(c, 4);
                p.sent.clear();
                p.out_reliable.clear();
                match p.state {
                    PeerState::Connected => {
                        if c[0] & FLAG_ACK != 0 {
                            p.zombie_after_flush = true;
                        } else {
                            p.state = PeerState::Zombie;
                        }
                        self.events.push_back(Event::Disconnect { peer: i, data });
                    }
                    PeerState::Connecting | PeerState::Disconnecting => {
                        p.state = PeerState::Zombie;
                        self.events.push_back(Event::Disconnect { peer: i, data });
                    }
                    _ => p.reset(),
                }
                true
            }
            CMD_PING => live,
            CMD_BANDWIDTH_LIMIT => live,
            CMD_THROTTLE_CONFIGURE => {
                if live {
                    p.throttle = (be32(c, 4), be32(c, 8), be32(c, 12));
                }
                live
            }
            CMD_SEND_RELIABLE => {
                if !live || channel as usize >= p.channels.len() {
                    return false;
                }
                let ch = &mut p.channels[channel as usize];
                let ahead = seq.wrapping_sub(ch.in_reliable) as i16;
                if ahead <= 0 || ahead as u32 >= 0x1000 * 7 {
                    return true; // already delivered (the ack is resent) or outside the window
                }
                ch.pending.insert(seq, payload.to_vec());
                loop {
                    let next = ch.in_reliable.wrapping_add(1);
                    let Some(d) = ch.pending.remove(&next) else { break };
                    ch.in_reliable = next;
                    self.events.push_back(Event::Receive { peer: i, channel, data: d });
                }
                true
            }
            CMD_SEND_UNRELIABLE => {
                if !live || channel as usize >= p.channels.len() {
                    return false;
                }
                self.events.push_back(Event::Receive { peer: i, channel, data: payload.to_vec() });
                true
            }
            CMD_SEND_UNSEQUENCED => {
                if !live || channel as usize >= p.channels.len() {
                    return false;
                }
                let mut group = be16(c, 4) as u32;
                let index = group % UNSEQ_WINDOW_SIZE;
                if group < p.in_unseq_group as u32 {
                    group += 0x10000;
                }
                if group >= p.in_unseq_group as u32 + FREE_UNSEQ_WINDOWS * UNSEQ_WINDOW_SIZE {
                    return true;
                }
                group &= 0xFFFF;
                if group.wrapping_sub(index) as u16 != p.in_unseq_group {
                    p.in_unseq_group = group.wrapping_sub(index) as u16;
                    p.unseq_window = [0; (UNSEQ_WINDOW_SIZE / 32) as usize];
                } else if p.unseq_window[(index / 32) as usize] & (1 << (index % 32)) != 0 {
                    return true; // a duplicate
                }
                p.unseq_window[(index / 32) as usize] |= 1 << (index % 32);
                self.events.push_back(Event::Receive { peer: i, channel, data: payload.to_vec() });
                true
            }
            CMD_SEND_FRAGMENT | CMD_SEND_UNRELIABLE_FRAGMENT => live, // not reassembled (see the module comment)
            _ => false,
        }
    }

    /// The peer slots that are connected to `addr`.
    pub fn peers_at(&self, addr: Addr) -> Vec<usize> {
        (0..self.peers.len()).filter(|&i| self.peers[i].addr == addr && self.peers[i].state != PeerState::Disconnected).collect()
    }
    pub fn has_outgoing(&self, peer: usize) -> bool {
        self.peers.get(peer).map_or(false, |p| p.has_outgoing())
    }
}

/// The round trip of one ack: its landing time `rx` (host ms) minus the 16-bit echo of our send time (which a peer
/// that holds the command moves forward by its hold), widened against `rx`. None for an echo from the future.
pub fn ack_rtt_ms(rx: u32, echo16: u32) -> Option<u32> {
    let mut t = (rx & 0xFFFF_0000) | (echo16 & 0xFFFF);
    if (t & 0x8000) > (rx & 0x8000) {
        t = t.wrapping_sub(0x10000);
    }
    let r = rx.wrapping_sub(t) as i32;
    (0..=10000).contains(&r).then_some((r as u32).max(1))
}

#[cfg(test)]
mod ack_rtt_tests {
    use super::ack_rtt_ms;
    #[test]
    fn widen_and_hold() {
        // sent at 70000 (echo 70000 & 0xFFFF), the peer held it 15 ms and moved the echo to 70015; landed at 70019
        assert_eq!(ack_rtt_ms(70019, (70000 + 15) & 0xFFFF), Some(4));
        // across a 16-bit wrap
        assert_eq!(ack_rtt_ms(0x1_0003, 0xFFFE), Some(5));
        // under a millisecond reads 1 (as ENet clamps), so a measured link never shows as unmeasured
        assert_eq!(ack_rtt_ms(500, 500), Some(1));
        // an echo past the landing time
        assert_eq!(ack_rtt_ms(1000, 1005), None);
    }
}
