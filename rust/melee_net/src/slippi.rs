//! Slippi Dolphin's online netplay messages and the peer-to-peer client that carries them over ENet
//! (docs/rust-port/notes/slippi-online.md section 1). Our own implementation of the protocol as Slippi Dolphin
//! `0653df819` speaks it (`SlippiNetplay.cpp`): `sf::Packet` encoding (big-endian integers, bool = 1 byte, string =
//! u32 length + bytes), three ENet channels (0 reliable for everything but the pads, 1 unsequenced for pads, acks
//! and suggestions, 2 unsequenced for the acks sent while handling a pad), a resend-until-acked local pad queue of
//! up to 128 frames (sent again on stalled frames, `resend_pads`), acks only for pads that added frames, ping from
//! ack round trips and Slippi's time offset.
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
/// Hero's Melee Unlocked build announcement (`NP_MSG_MU_BUILD`). Receive only here: we never send it (bar the test
/// hook), and never answer the pre-match ping a peer that announces capability bit 1 expects.
pub const MSG_MU_BUILD: u8 = 0xE0;

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

/// `SLIPPI_SYNCED_STATE` (0x86): `u8 player port, string match_id, u32 game_idx, u32 tiebreak_idx, u32 seconds_remaining,
/// 4 x (u8 stocks, u16 current_health)`: what a client sends after a game that ended on a desync (ranked), so both
/// restart the game from the same state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncedMsg {
    pub port: u8,
    pub match_id: String,
    pub game_idx: u32,
    pub tiebreak_idx: u32,
    pub state: melee_core::slippi_online::SyncedState,
}

impl SyncedMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = PacketW::default().u8(MSG_SYNCED_STATE).u8(self.port).string(&self.match_id).u32(self.game_idx).u32(self.tiebreak_idx).u32(self.state.seconds);
        for f in &self.state.fighters {
            w = w.u8(f.stocks).u16(f.percent);
        }
        w.0
    }
    pub fn decode(b: &[u8]) -> Option<SyncedMsg> {
        let mut r = PacketR::new(b);
        if r.u8()? != MSG_SYNCED_STATE {
            return None;
        }
        let (port, match_id, game_idx, tiebreak_idx, seconds) = (r.u8()?, r.string()?, r.u32()?, r.u32()?, r.u32()?);
        let mut state = melee_core::slippi_online::SyncedState { seconds, ..Default::default() };
        for f in &mut state.fighters {
            *f = melee_core::slippi_online::SyncedFighter { stocks: r.u8()?, percent: r.u16()? };
        }
        Some(SyncedMsg { port, match_id, game_idx, tiebreak_idx, state })
    }
}

pub fn encode_ack(frame: i32, port: u8) -> Vec<u8> {
    PacketW::default().u8(MSG_PAD_ACK).i32(frame).u8(port).0
}

/// `Note::Disconnected` reason when the peer address started a new match during ours (its process restarted).
/// The first bytes of a link `Client::export` wrote.
pub const LINK_MAGIC: &[u8] = b"SLNK1";

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

/// What a peer's `NP_MSG_MU_BUILD` (0xE0) said, parsed tolerantly: every field after the 'MUB' tag may be missing.
/// Layout: 'M' 'U' 'B', version (1), player port, mod_view, fingerprint len + bytes, name len + bytes, then optionally
/// capabilities (bit 0 early inputs, 1 answers pre-match pings, 2 Static engine, 3 ranked rules digest follows),
/// shield colour (1 + RGB, or 0), 64-byte rules digest (only with bit 3; not read here).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeerBuild {
    pub version: u8,
    pub port: Option<u8>,
    /// the peer runs a mod (`mod_view`); false: the retail game
    pub mod_view: bool,
    pub fingerprint: String,
    pub name: String,
    /// None: the byte was absent (a build before capabilities)
    pub caps: Option<u8>,
    pub shield: Option<u32>,
    /// the message ended before a field the layout always has
    pub truncated: bool,
    /// the packet as received (carried over a link hand-off)
    pub raw: Vec<u8>,
}

fn printable(b: &[u8]) -> String {
    b.iter().map(|&c| if (0x20..0x7f).contains(&c) { c as char } else { '?' }).collect()
}

impl PeerBuild {
    /// `b` is the whole packet including the 0xE0 id byte. None when it is not a build message (no 'MUB' tag).
    pub fn parse(b: &[u8]) -> Option<PeerBuild> {
        let mut r = PacketR::new(b);
        if r.u8()? != MSG_MU_BUILD || r.u8()? != b'M' || r.u8()? != b'U' || r.u8()? != b'B' {
            return None;
        }
        let mut o = PeerBuild { raw: b.to_vec(), ..Default::default() };
        let Some(v) = r.u8() else {
            o.truncated = true;
            return Some(o);
        };
        o.version = v;
        if v != 1 {
            return Some(o); // an unknown layout: noted, not read
        }
        macro_rules! need {
            ($e:expr) => {
                match $e {
                    Some(x) => x,
                    None => {
                        o.truncated = true;
                        return Some(o);
                    }
                }
            };
        }
        o.port = Some(need!(r.u8()));
        o.mod_view = need!(r.u8()) != 0;
        let n = need!(r.u8()) as usize;
        o.fingerprint = printable(need!(r.take(n)));
        let n = need!(r.u8()) as usize;
        o.name = printable(need!(r.take(n)));
        // from here on absence is normal (older builds stop earlier)
        o.caps = r.u8();
        if r.u8() == Some(1) {
            if let (Some(a), Some(b), Some(c)) = (r.u8(), r.u8(), r.u8()) {
                o.shield = Some(((a as u32) << 16) | ((b as u32) << 8) | c as u32);
            }
        }
        Some(o)
    }

    /// Hero's Static recomp engine (capability bit 2); else the native Source Port.
    pub fn static_engine(&self) -> bool {
        self.caps.is_some_and(|c| c & 4 != 0)
    }

    pub fn capability_names(&self) -> Vec<&'static str> {
        let c = self.caps.unwrap_or(0);
        let mut v = vec![];
        if c & 1 != 0 {
            v.push("early inputs");
        }
        if c & 2 != 0 {
            v.push("answers pre-match pings");
        }
        if c & 4 != 0 {
            v.push("static engine");
        }
        if c & 8 != 0 {
            v.push("ranked rules digest");
        }
        v
    }

    /// "Melee Unlocked build: Source Port; mod <name> <fingerprint prefix>; capabilities ..."
    pub fn describe(&self) -> String {
        if self.version != 1 {
            return format!("Melee Unlocked build: unknown message version {}{}", self.version, if self.truncated { " (truncated)" } else { "" });
        }
        let kind = if self.static_engine() { "Static Recomp" } else { "Source Port" };
        let game = if self.mod_view {
            let fp: String = self.fingerprint.chars().take(16).collect();
            format!("mod {} {}", self.name, fp).trim_end().to_string()
        } else {
            "retail".to_string()
        };
        let caps = self.capability_names();
        let caps = if self.caps.is_none() {
            "unknown (older build)".to_string()
        } else if caps.is_empty() {
            "none".to_string()
        } else {
            caps.join(", ")
        };
        format!("Melee Unlocked build: {kind}; {game}; capabilities {caps}{}", if self.truncated { " (message truncated)" } else { "" })
    }
}

/// Called from the netplay thread when a peer's build message is first seen or changes: the remote player's port and
/// what it said.
pub static BUILD_HOOK: std::sync::OnceLock<fn(u8, &PeerBuild)> = std::sync::OnceLock::new();

#[derive(Clone, Copy)]
struct Timing {
    frame: i32,
    time_us: i64,
}

/// One remote player's link and pad state (1v1: there is one).
struct Remote {
    addr: Addr,
    /// Hero's public fallback (slippi_address.h): the public address tried 2 s into a LAN attempt.
    fallback: Option<Addr>,
    fallback_started: bool,
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
    build: Option<PeerBuild>,
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
    /// The opponent's next lock-in, sent on the match's own link before this side's match was over (Slippi Dolphin
    /// goes back to its select the frame its VS scene ends and locks in on Start; this side lingers for the last
    /// confirmed frames). Held until `end_game`, then taken like any lock-in on the select.
    next_selections: Option<Selections>,
    notes: VecDeque<Note>,
    t0: Instant,
    clock_us: Option<Box<dyn Fn() -> i64>>,
    started_at_us: i64,
    pub connect_timeout_ms: i64,
    /// Every remote (frame, checksum) seen, for the owner's frame-by-frame compare.
    pub remote_checksums: BTreeMap<i32, u32>,
    /// The peer's `SLIPPI_SYNCED_STATE` (desync recovery, ranked): the latest one received, and the one this side sent.
    pub remote_synced: Option<SyncedMsg>,
    pub local_synced: Option<SyncedMsg>,
    pub stats: ClientStats,
    /// Test hook (`MELEE_NP_TEST_SEND_BUILD`) already sent.
    test_build_sent: bool,
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
    /// Pad packets sent by [`Client::resend_pads`]: the unacked queue again, on a frame with no new local pad.
    pub pads_resent: u64,
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
                fallback: None,
                fallback_started: false,
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
                build: None,
            },
            local_queue: VecDeque::new(),
            has_game_started: false,
            in_game: false,
            next_selections: None,
            notes: VecDeque::new(),
            t0: Instant::now(),
            clock_us: None,
            started_at_us: 0,
            connect_timeout_ms: 8000,
            remote_checksums: BTreeMap::new(),
            remote_synced: None,
            local_synced: None,
            stats: ClientStats::default(),
            test_build_sent: false,
            last_probe_us: 0,
        };
        c.started_at_us = c.now_us();
        c
    }
    /// Hero's `public_fallbacks_`: the opponent's public address, connected to when the first address has not
    /// answered after 2 s (the LAN address of a player behind the same router).
    pub fn set_fallback(&mut self, a: Option<Addr>) {
        self.remote.fallback = a.filter(|f| *f != self.remote.addr);
    }

    /// The whole link for another process (`import`): the ENet host, the opponent's state and the notes not yet
    /// read. Nothing is sent: the opponent sees one link that goes on (Slippi Dolphin keeps one client from the select
    /// into the match and does not take a second connection after the first drops).
    pub fn export(&self) -> Vec<u8> {
        use crate::blob::W;
        let mut w = W::default();
        w.0.extend_from_slice(LINK_MAGIC);
        w.bytes(&self.host.export());
        w.i64(self.now_us());
        w.u8(self.local_port);
        w.u8(match self.status {
            Status::Initiated => 0,
            Status::Connected => 1,
            Status::Failed => 2,
            Status::Disconnected => 3,
        });
        let rm = &self.remote;
        w.addr(rm.addr);
        w.u8(rm.fallback.is_some() as u8);
        w.addr(rm.fallback.unwrap_or_default());
        w.u8(rm.fallback_started as u8);
        w.u8(rm.port);
        w.u32(rm.links.len() as u32);
        for &l in &rm.links {
            w.u32(l as u32);
        }
        w.u32(rm.active.map_or(u32::MAX, |a| a as u32));
        w.u32(rm.head_frame as u32);
        w.u32(rm.last_acked as u32);
        w.u32(rm.ack_timers.len() as u32);
        for t in &rm.ack_timers {
            w.u32(t.frame as u32);
            w.i64(t.time_us);
        }
        w.i64(rm.ping_us);
        w.u32(rm.last_timing.frame as u32);
        w.i64(rm.last_timing.time_us);
        w.u32(rm.offsets.len() as u32);
        for &o in &rm.offsets {
            w.u32(o as u32);
        }
        w.u32(rm.offset_idx as u32);
        fn sel(w: &mut crate::blob::W, s: Option<Selections>) {
            match s {
                Some(s) => {
                    w.u8(1);
                    w.bytes(&s.encode()[1..]);
                }
                None => w.u8(0),
            }
        }
        sel(&mut w, rm.selections);
        w.u32(self.local_queue.len() as u32);
        for (f, pad, cf, cs) in &self.local_queue {
            w.u32(*f as u32);
            w.0.extend_from_slice(pad);
            w.u32(*cf as u32);
            w.u32(*cs);
        }
        w.u8(self.has_game_started as u8);
        w.u8(self.in_game as u8);
        w.u32(self.notes.len() as u32);
        for n in &self.notes {
            match n {
                Note::Connected => w.u8(0),
                Note::Selections(s) => {
                    w.u8(1);
                    sel(&mut w, Some(*s));
                }
                Note::Pad { port, frame, pad } => {
                    w.u8(2);
                    w.u8(*port);
                    w.u32(*frame as u32);
                    w.0.extend_from_slice(pad);
                }
                Note::Checksum { port, frame, value } => {
                    w.u8(3);
                    w.u8(*port);
                    w.u32(*frame as u32);
                    w.u32(*value);
                }
                Note::Chat { port, id } => {
                    w.u8(4);
                    w.u8(*port);
                    w.u32(*id as u32);
                }
                Note::Disconnected { reason } => {
                    w.u8(5);
                    w.u32(*reason);
                }
            }
        }
        w.i64(self.started_at_us);
        w.i64(self.connect_timeout_ms);
        w.u32(self.remote_checksums.len() as u32);
        for (f, c) in &self.remote_checksums {
            w.u32(*f as u32);
            w.u32(*c);
        }
        let s = &self.stats;
        for v in [s.pads_sent, s.pads_received, s.acks_sent, s.acks_received, s.max_queue as u64, s.pads_resent] {
            w.u64(v);
        }
        w.i64(self.last_probe_us);
        // the peer's build message (0xE0), when one came: the match process starts from the select's link
        match &self.remote.build {
            Some(pb) => {
                w.u8(1);
                w.bytes(&pb.raw);
            }
            None => w.u8(0),
        }
        w.0
    }

    /// The link `export` wrote, going on over `tp` (a socket on the same local port). None for a malformed blob.
    pub fn import(tp: Box<dyn Transport>, b: &[u8]) -> Option<Client> {
        let b = b.strip_prefix(LINK_MAGIC)?;
        let mut r = crate::blob::R::new(b);
        let hb = r.bytes()?;
        let mut hr = crate::blob::R::new(&hb);
        let host = Host::import(tp, &mut hr)?;
        if !hr.done() {
            return None;
        }
        let now_us = r.i64()?;
        let local_port = r.u8()?;
        let status = match r.u8()? {
            0 => Status::Initiated,
            1 => Status::Connected,
            2 => Status::Failed,
            3 => Status::Disconnected,
            _ => return None,
        };
        fn sel(r: &mut crate::blob::R) -> Option<Option<Selections>> {
            Some(match r.u8()? {
                0 => None,
                _ => Some(Selections::decode(&mut PacketR::new(&r.bytes()?))?),
            })
        }
        fn pad(r: &mut crate::blob::R) -> Option<[u8; PAD_WIRE]> {
            let mut p = [0u8; PAD_WIRE];
            for v in p.iter_mut() {
                *v = r.u8()?;
            }
            Some(p)
        }
        let addr = r.addr()?;
        let has_fb = r.u8()? != 0;
        let fb = r.addr()?;
        let fallback_started = r.u8()? != 0;
        let port = r.u8()?;
        let mut links = Vec::new();
        for _ in 0..r.u32()? {
            links.push(r.u32()? as usize);
        }
        let active = Some(r.u32()?).filter(|&a| a != u32::MAX).map(|a| a as usize);
        let head_frame = r.u32()? as i32;
        let last_acked = r.u32()? as i32;
        let mut ack_timers = VecDeque::new();
        for _ in 0..r.u32()? {
            ack_timers.push_back(Timing { frame: r.u32()? as i32, time_us: r.i64()? });
        }
        let ping_us = r.i64()?;
        let last_timing = Timing { frame: r.u32()? as i32, time_us: r.i64()? };
        let mut offsets = Vec::new();
        for _ in 0..r.u32()? {
            offsets.push(r.u32()? as i32);
        }
        let offset_idx = r.u32()? as usize;
        let selections = sel(&mut r)?;
        let mut local_queue = VecDeque::new();
        for _ in 0..r.u32()? {
            local_queue.push_back((r.u32()? as i32, pad(&mut r)?, r.u32()? as i32, r.u32()?));
        }
        let has_game_started = r.u8()? != 0;
        let in_game = r.u8()? != 0;
        let mut notes = VecDeque::new();
        for _ in 0..r.u32()? {
            notes.push_back(match r.u8()? {
                0 => Note::Connected,
                1 => Note::Selections(sel(&mut r)??),
                2 => Note::Pad { port: r.u8()?, frame: r.u32()? as i32, pad: pad(&mut r)? },
                3 => Note::Checksum { port: r.u8()?, frame: r.u32()? as i32, value: r.u32()? },
                4 => Note::Chat { port: r.u8()?, id: r.u32()? as i32 },
                5 => Note::Disconnected { reason: r.u32()? },
                _ => return None,
            });
        }
        let started_at_us = r.i64()?;
        let connect_timeout_ms = r.i64()?;
        let mut remote_checksums = BTreeMap::new();
        for _ in 0..r.u32()? {
            let f = r.u32()? as i32;
            remote_checksums.insert(f, r.u32()?);
        }
        let stats = ClientStats { pads_sent: r.u64()?, pads_received: r.u64()?, acks_sent: r.u64()?, acks_received: r.u64()?, max_queue: r.u64()? as usize, pads_resent: r.u64()? };
        let last_probe_us = r.i64()?;
        let mut build = None;
        if !r.done() && r.u8()? == 1 {
            build = PeerBuild::parse(&r.bytes()?);
        }
        if !r.done() {
            return None;
        }
        if let (Some(pb), Some(h)) = (&build, BUILD_HOOK.get()) {
            h(port, pb);
        }
        Some(Client {
            host,
            local_port,
            status,
            remote: Remote { addr, fallback: has_fb.then_some(fb), fallback_started, port, links, active, head_frame, last_acked, ack_timers, ping_us, last_timing, offsets, offset_idx, selections, build },
            local_queue,
            has_game_started,
            in_game,
            next_selections: None,
            notes,
            // the client clock continues from the exporter's
            t0: Instant::now().checked_sub(std::time::Duration::from_micros(now_us.max(0) as u64))?,
            clock_us: None,
            started_at_us,
            connect_timeout_ms,
            remote_checksums,
            remote_synced: None,
            local_synced: None,
            stats,
            test_build_sent: true,
            last_probe_us,
        })
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

    /// The peer's parsed build message, if one came (None: stock Slippi Dolphin or another client).
    pub fn remote_build(&self) -> Option<&PeerBuild> {
        self.remote.build.as_ref()
    }

    /// Test only, off unless `MELEE_NP_TEST_SEND_BUILD` is set: once, send a synthetic 0xE0 as Hero's build would
    /// (`1`: a Source Port, retail; `mod`: a Source Port running a mod; `static`: a Static Recomp; any other value: hex
    /// bytes of the whole packet).
    fn test_send_build(&mut self) {
        let Ok(v) = std::env::var("MELEE_NP_TEST_SEND_BUILD") else { return };
        if self.test_build_sent || v.is_empty() {
            return;
        }
        self.test_build_sent = true;
        let mk = |mod_view: bool, caps: u8| {
            let (fp, name) = if mod_view { ("0123456789abcdef0123456789abcdef", "TestMod") } else { ("", "") };
            let mut p = vec![MSG_MU_BUILD, b'M', b'U', b'B', 1, self.local_port, mod_view as u8, fp.len() as u8];
            p.extend_from_slice(fp.as_bytes());
            p.push(name.len() as u8);
            p.extend_from_slice(name.as_bytes());
            p.extend_from_slice(&[caps, 0]);
            p
        };
        let p = match v.as_str() {
            "1" => mk(false, 3),
            "mod" => mk(true, 3),
            "static" => mk(false, 6),
            h => (0..h.len() / 2).filter_map(|i| u8::from_str_radix(h.get(i * 2..i * 2 + 2)?, 16).ok()).collect(),
        };
        eprintln!("slippi: TEST: sending a synthetic build message ({} bytes)", p.len());
        self.send(&p);
    }

    /// `SetMatchSelections`.
    pub fn set_match_selections(&mut self, mut s: Selections) {
        self.test_send_build();
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
        self.next_selections = None;
        self.local_queue.clear();
        let now = self.now_us();
        self.remote.last_timing = Timing { frame: 0, time_us: now };
        self.remote.last_acked = 0;
        self.remote.ack_timers.clear();
    }

    /// The match is over and the link stays for the set's next game (Slippi Dolphin keeps one client through a Direct
    /// set): the next selections are the opponent's next lock-in, not a restarted peer, and the opponent's link going
    /// down is no longer the match's.
    pub fn end_game(&mut self) {
        self.in_game = false;
        if let Some(s) = self.next_selections.take() {
            self.accept_selections(s);
        }
    }

    /// The opponent's selections on the select (`SLIPPI_MATCH_SELECTIONS` in `OnData`: merged, the pad state reset
    /// for the next game).
    fn accept_selections(&mut self, s: Selections) {
        self.remote.selections = Some(s);
        self.has_game_started = false;
        self.remote.head_frame = 0;
        // a set's next game: the last game's checksums are not this one's (its frames start again at 1)
        self.remote_checksums.clear();
        self.notes.push_back(Note::Selections(s));
    }

    /// `SendSyncedGameState`: remember and send this side's recovery state (reliable, channel 0).
    pub fn send_synced_state(&mut self, m: SyncedMsg) {
        let b = m.encode();
        self.local_synced = Some(m);
        self.send(&b);
    }

    /// `GetDesyncRecoveryState`: the merged state once the peer's synced state for the same game has arrived
    /// (`Ok(None)`: still waiting), `Err` when the two cannot be reconciled (Slippi drops the connection).
    pub fn recovery_state(&self) -> Result<Option<melee_core::slippi_online::SyncedState>, &'static str> {
        let Some(l) = &self.local_synced else { return Ok(None) };
        match &self.remote_synced {
            Some(r) if r.game_idx == l.game_idx && r.tiebreak_idx == l.tiebreak_idx => melee_core::slippi_online::merge_recovery(&l.state, &[r.state]).map(Some),
            _ => Ok(None),
        }
    }

    /// `SendSlippiPad`: queue the local pad for `frame` (with the finalized checksum it carries) and send every
    /// unacked pad, newest first.
    pub fn send_pad(&mut self, frame: i32, pad: [u8; PAD_WIRE], checksum_frame: i32, checksum: u32) {
        if matches!(self.status, Status::Failed | Status::Disconnected) {
            return;
        }
        self.local_queue.push_front((frame, pad, checksum_frame, checksum));
        if self.send_local_queue() {
            self.stats.max_queue = self.stats.max_queue.max(self.local_queue.len());
            self.stats.pads_sent += 1;
        }
    }

    /// `SendSlippiPad(nullptr)`: send the unacked pads again with no new one. The game calls it on every frame it
    /// stalls waiting for the peer (Slippi's skipped frame, a full rollback window, past its last frame), as the
    /// game's stalled frame does in Slippi Dolphin: pads are unsequenced and unreliable, so a lost newest packet is
    /// only repaired by a later packet carrying the queue.
    pub fn resend_pads(&mut self) {
        if matches!(self.status, Status::Failed | Status::Disconnected) {
            return;
        }
        if self.send_local_queue() {
            self.stats.pads_resent += 1;
        }
    }

    /// Drop the acked pads (and any older than the 128-frame window), then send the rest as one pad packet, newest
    /// first, with its send timing and ack timer (a resend's too, as in Slippi Dolphin). False when nothing is left
    /// unacked.
    fn send_local_queue(&mut self) -> bool {
        let Some(cur) = self.local_queue.front().map(|q| q.0) else { return false };
        let min_ack = self.remote.last_acked.max(cur - 128);
        while self.local_queue.back().map_or(false, |b| b.0 < min_ack) {
            self.local_queue.pop_back();
        }
        let Some(f) = self.local_queue.front() else { return false };
        let m = PadMsg { frame: f.0, port: self.local_port, checksum_frame: f.2, checksum: f.3, pads: self.local_queue.iter().map(|q| q.1).collect() };
        let b = m.encode();
        self.send(&b);
        let now = self.now_us();
        self.has_game_started = true;
        self.remote.last_timing = Timing { frame: m.frame, time_us: now };
        self.remote.ack_timers.push_back(Timing { frame: m.frame, time_us: now });
        true
    }

    /// Put the queued datagrams on the wire now (`enet_host_flush`) instead of at the next `poll`: called right
    /// after the frame's pads are queued so they do not wait out a tick.
    pub fn flush(&mut self) {
        self.host.flush();
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
        // Hero's public_fallback_due: 2 s with no link to the first address, the public one as well
        if self.status == Status::Initiated && !self.remote.fallback_started && (self.now_us() - self.started_at_us) / 1000 >= 2000 {
            if let Some(f) = self.remote.fallback {
                self.remote.fallback_started = true;
                eprintln!("slippi: LAN connection pending; trying public address fallback");
                if let Some(l) = self.host.connect(f, 3, 0) {
                    self.remote.links.push(l);
                }
            }
        }
        if self.status == Status::Initiated && (self.now_us() - self.started_at_us) / 1000 >= self.connect_timeout_ms {
            self.status = Status::Failed;
        }
    }

    fn on_connect(&mut self, peer: usize) {
        // only the expected player may join: as Slippi Dolphin and Hero match a connecting peer, its exact address,
        // else the same host (a NAT that maps the opponent's port anew for us: its CONNECT comes from another port
        // than matchmaking named), else the public fallback's host. Before the link is up only.
        let a = self.host.peers[peer].addr;
        let same_host = |b: Addr| b.ip == a.ip && b.ip6 == a.ip6;
        let known = a == self.remote.addr || (self.remote.active.is_none() && (same_host(self.remote.addr) || self.remote.fallback.is_some_and(same_host)));
        if !known {
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
                let la = self.host.peers[l].addr;
                if l != peer && self.host.peers[l].state == PeerState::Connecting && (la == self.remote.addr || Some(la) == self.remote.fallback) {
                    self.host.reset_peer(l);
                    self.remote.links.retain(|&x| x != l);
                }
            }
            self.remote.active = Some(peer);
            self.remote.addr = a;
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
            // once: the peer's DISCONNECT (its reason word), or ENet's timeout (0: a dead process, ~5-7 s on a fast link)
            eprintln!("slippi: the opponent's link is down (reason {data}{})", if self.in_game { ", in a match" } else { "" });
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
                if self.in_game && self.remote.active == Some(peer) {
                    // the opponent's next lock-in on this match's link: its match is over and it is back on its select
                    // (Slippi Dolphin merges selections whenever they come, SlippiNetplay.cpp OnData
                    // SLIPPI_MATCH_SELECTIONS, and its Unranked/Direct VSSceneDecide goes straight back to the CSS with
                    // the link kept). Not a restarted peer: held for the end of this side's match
                    if self.next_selections.is_none() {
                        eprintln!("slippi: the opponent locked in for the next game before this match was over; held for the select");
                    }
                    self.next_selections = Some(s);
                    return;
                }
                if self.in_game {
                    // selections in a match on another link: the remote is a new process starting its own match (a
                    // restart on the same address, a new ENet connection); this match's peer is gone
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
                self.accept_selections(s);
            }
            MSG_CHAT_MESSAGE => {
                if let (Some(id), Some(port)) = (r.i32(), r.u8()) {
                    self.notes.push_back(Note::Chat { port, id });
                }
            }
            MSG_SYNCED_STATE => {
                if let Some(m) = SyncedMsg::decode(b).filter(|m| m.port == self.remote.port) {
                    self.remote_synced = Some(m);
                }
            }
            MSG_MU_BUILD => {
                // receive only; a bad or unknown message is ignored, never an error
                if let Some(pb) = PeerBuild::parse(b) {
                    if self.remote.build.as_ref() != Some(&pb) {
                        if let Some(h) = BUILD_HOOK.get() {
                            h(self.remote.port, &pb);
                        }
                        self.remote.build = Some(pb);
                    }
                }
            }
            _ => {} // CONN_SELECTED, game-prep steps, suggestions, drops: not used in a direct 1v1
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
    fn peer_build_parse() {
        // full: retail-less mod, early inputs + pings + static, a shield colour
        let mut p = vec![0xE0, b'M', b'U', b'B', 1, 1, 1, 4];
        p.extend_from_slice(b"abcd");
        p.push(3);
        p.extend_from_slice(b"Mod");
        p.extend_from_slice(&[7, 1, 0x11, 0x22, 0x33]);
        let b = PeerBuild::parse(&p).unwrap();
        assert!(b.mod_view && b.static_engine() && !b.truncated);
        assert_eq!((b.name.as_str(), b.fingerprint.as_str(), b.caps, b.shield), ("Mod", "abcd", Some(7), Some(0x112233)));
        assert_eq!(b.describe(), "Melee Unlocked build: Static Recomp; mod Mod abcd; capabilities early inputs, answers pre-match pings, static engine");
        // an older build: stops after the name, retail
        let b = PeerBuild::parse(&[0xE0, b'M', b'U', b'B', 1, 0, 0, 0, 0]).unwrap();
        assert_eq!(b.describe(), "Melee Unlocked build: Source Port; retail; capabilities unknown (older build)");
        // truncated mid fingerprint, and right after the tag
        let b = PeerBuild::parse(&[0xE0, b'M', b'U', b'B', 1, 0, 1, 9, b'a']).unwrap();
        assert!(b.truncated && b.describe().contains("truncated"));
        assert!(PeerBuild::parse(&[0xE0, b'M', b'U', b'B']).unwrap().truncated);
        // an unknown version is noted, not read
        assert!(PeerBuild::parse(&[0xE0, b'M', b'U', b'B', 9, 1, 2]).unwrap().describe().contains("unknown message version 9"));
        // garbage: not a build message
        assert!(PeerBuild::parse(&[]).is_none());
        assert!(PeerBuild::parse(&[0xE0]).is_none());
        assert!(PeerBuild::parse(&[0xE0, 1, 2, 3, 4, 5]).is_none());
        assert!(PeerBuild::parse(&[0x80, b'M', b'U', b'B', 1]).is_none());
        // control bytes in a name are masked
        let b = PeerBuild::parse(&[0xE0, b'M', b'U', b'B', 1, 0, 1, 0, 2, 0x1b, b'x', 0]).unwrap();
        assert_eq!(b.name, "?x");
    }

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
    fn synced_state_layout() {
        let mut state = melee_core::slippi_online::SyncedState { seconds: 0x0102_0304, ..Default::default() };
        state.fighters[1] = melee_core::slippi_online::SyncedFighter { stocks: 2, percent: 0x0A0B };
        let m = SyncedMsg { port: 1, match_id: "ab".into(), game_idx: 2, tiebreak_idx: 0, state };
        let b = m.encode();
        // id, port, string (u32 len + bytes), game, tiebreak, seconds, 4 x (u8, u16)
        assert_eq!(&b[..12], &[0x86, 1, 0, 0, 0, 2, b'a', b'b', 0, 0, 0, 2]);
        assert_eq!(b.len(), 1 + 1 + 6 + 4 + 4 + 4 + 12);
        assert_eq!(&b[b.len() - 9..b.len() - 6], &[2, 0x0A, 0x0B]);
        assert_eq!(SyncedMsg::decode(&b), Some(m));
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
