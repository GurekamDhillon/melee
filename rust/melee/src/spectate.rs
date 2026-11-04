//! Read-only spectators of a `melee play --netplay` match (docs/rust-port/notes/spectate-replay.md).
//!
//! The match host streams its replay as it is confirmed: the `.mrpl` header (the play options that set the boot up, the
//! seed, the agreed scene) and then one record per engine tick (the four pads the engine read, the neutral hash after a
//! match frame). Only confirmed frames are in the stream (`NetDriver::tick_match` records a frame once every input of it
//! is final), so a spectator never sees a prediction and never rolls back. The spectator boots the same build with the
//! same options and plays the records back like `--replay`, `delay` ticks behind the newest one it has (the delay
//! buffer), comparing every hash with the host's. Nothing goes from a spectator to the players: the host's spectate
//! socket is its own (not the players' room), it only reads hello / ack packets, and a spectator's stall or loss never
//! holds the match.
//!
//! Joining mid-match: the stream always starts at the boot (the deterministic snapshot every peer starts from), and the
//! spectator replays the history unpaced until it is `delay` ticks behind (catch-up), then plays in real time. A native
//! state snapshot cannot cross processes (it holds this process's addresses), so the boot is the snapshot.
//!
//! Wire: UDP datagrams `GDSP` + kind byte + body.
//!   spectator -> host  `H` u8 version, u64 build, u64 mods digest, mods description (UTF-8)   hello (resent until answered)
//!   host -> spectator  `A`                                                                    accepted
//!                      `R` reason (UTF-8)                                                     refused (build / mods / full)
//!                      `D` u32 offset, bytes                                                  stream bytes at offset
//!                      `E` u32 total                                                          the stream is complete
//!   spectator -> host  `K` u32 bytes held                                                     cumulative ack (also keepalive)
//!                      `B`                                                                    bye
//! Reliability is go-back-N on the byte stream: the host sends up to WINDOW chunks past the spectator's ack and goes back
//! to the ack when it stops moving. Through melee_relay the same datagrams travel inside `GDMS` (see melee_relay).
use crate::play_net::{Replay, REC_BYTES};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

const MAGIC: &[u8; 4] = b"GDSP";
const VERSION: u8 = 1;
const CHUNK: usize = 1100;
const WINDOW: usize = 48;
const MAX_SPECTATORS: usize = 16;

fn log(m: &str) {
    eprintln!("play: spectate: {m}");
}

fn resolve(s: &str) -> Result<SocketAddr, String> {
    s.to_socket_addrs().map_err(|e| format!("{s}: {e}"))?.next().ok_or_else(|| format!("{s}: no address"))
}

/// The stream's first bytes: the replay header with the record count set to u32::MAX (open-ended).
pub fn stream_header(r: &Replay) -> Vec<u8> {
    let h = Replay { flags: r.flags, args: r.args.clone(), meta: r.meta.clone(), ticks: Vec::new() };
    let mut b = h.to_bytes();
    let n = b.len();
    b[n - 4..].copy_from_slice(&u32::MAX.to_le_bytes());
    b
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Key {
    Direct(SocketAddr),
    Relayed(u32),
}

struct Peer {
    acked: usize,
    next: usize,
    last_ack: Instant,
    last_seen: Instant,
    accepted: bool,
    done: bool,
}

/// The match host's spectate service.
pub struct SpecServer {
    sock: UdpSocket,
    relay: Option<SocketAddr>,
    relay_code: Option<String>,
    relay_ka: Instant,
    build: u64,
    mods: u64,
    peers: HashMap<Key, Peer>,
    stream: Vec<u8>,
    streamed_ticks: usize,
    ended: bool,
    pub joined: u32,
    pub refused: u32,
}

impl SpecServer {
    /// `--np-spectate PORT` (direct, bound on `bind`) and / or `--np-spectate-relay HOST:PORT [--np-spectate-code C]`.
    pub fn open(bind: &str, port: u16, relay: Option<&str>, code: Option<&str>, build: u64, mods: u64) -> Result<SpecServer, String> {
        let sock = UdpSocket::bind((bind, port)).map_err(|e| format!("spectate bind {bind}:{port}: {e}"))?;
        sock.set_nonblocking(true).map_err(|e| e.to_string())?;
        let relay = relay.map(resolve).transpose()?;
        let s = SpecServer {
            sock,
            relay,
            relay_code: code.map(str::to_string),
            relay_ka: Instant::now() - Duration::from_secs(60),
            build,
            mods,
            peers: HashMap::new(),
            stream: Vec::new(),
            streamed_ticks: 0,
            ended: false,
            joined: 0,
            refused: 0,
        };
        log(&format!("host: spectators on udp {}{}", s.sock.local_addr().map_or("?".into(), |a| a.to_string()), relay.map_or(String::new(), |r| format!(", relay {r}"))));
        Ok(s)
    }

    fn send(&self, to: Key, body: &[u8]) {
        let mut b = Vec::with_capacity(body.len() + 12);
        match to {
            Key::Direct(a) => {
                b.extend_from_slice(MAGIC);
                b.extend_from_slice(body);
                let _ = self.sock.send_to(&b, a);
            }
            Key::Relayed(id) => {
                let Some(r) = self.relay else { return };
                b.extend_from_slice(b"GDMS");
                b.extend_from_slice(&id.to_le_bytes());
                b.extend_from_slice(MAGIC);
                b.extend_from_slice(body);
                let _ = self.sock.send_to(&b, r);
            }
        }
    }

    fn relay_text(&self, t: &str) {
        if let Some(r) = self.relay {
            let mut b = b"GDMR".to_vec();
            b.extend_from_slice(t.as_bytes());
            let _ = self.sock.send_to(&b, r);
        }
    }

    fn on_packet(&mut self, from: Key, d: &[u8]) {
        if d.len() < 5 || &d[..4] != MAGIC {
            return;
        }
        let now = Instant::now();
        match d[4] {
            b'H' if d.len() >= 22 => {
                let ver = d[5];
                let build = u64::from_le_bytes(d[6..14].try_into().unwrap());
                let mods = u64::from_le_bytes(d[14..22].try_into().unwrap());
                let desc = String::from_utf8_lossy(&d[22..]).into_owned();
                let why = if ver != VERSION {
                    Some(format!("spectate protocol {ver}, the host speaks {VERSION}"))
                } else if build != self.build {
                    Some(format!("different build: spectator {build:015x}, host {:015x} (the sim must be bit-identical)", self.build))
                } else if mods != self.mods {
                    Some(format!("different mods: spectator {mods:016x} ({desc}), host {:016x}", self.mods))
                } else if !self.peers.contains_key(&from) && self.peers.len() >= MAX_SPECTATORS {
                    Some("spectator limit reached".into())
                } else {
                    None
                };
                if let Some(w) = why {
                    if !self.peers.contains_key(&from) {
                        self.refused += 1;
                        log(&format!("host: refused {from:?}: {w}"));
                    }
                    let mut b = vec![b'R'];
                    b.extend_from_slice(w.as_bytes());
                    self.send(from, &b);
                    return;
                }
                if !self.peers.contains_key(&from) {
                    self.joined += 1;
                    log(&format!("host: spectator {from:?} joined ({} stream bytes so far, {} ticks)", self.stream.len(), self.streamed_ticks));
                    self.peers.insert(from, Peer { acked: 0, next: 0, last_ack: now, last_seen: now, accepted: true, done: false });
                }
                self.send(from, b"A");
            }
            b'K' if d.len() >= 9 => {
                let have = u32::from_le_bytes(d[5..9].try_into().unwrap()) as usize;
                if let Some(p) = self.peers.get_mut(&from) {
                    p.last_seen = now;
                    if have > p.acked {
                        p.acked = have.min(self.stream.len());
                        p.last_ack = now;
                        p.next = p.next.max(p.acked);
                    }
                    if self.ended && p.acked >= self.stream.len() {
                        p.done = true;
                    }
                }
            }
            b'B' => {
                if self.peers.remove(&from).is_some() {
                    log(&format!("host: spectator {from:?} left"));
                }
            }
            _ => {}
        }
    }

    fn poll(&mut self) {
        let mut buf = [0u8; 2048];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((n, a)) => {
                    let d = buf[..n].to_vec();
                    if Some(a) == self.relay {
                        if d.starts_with(b"GDMS") && d.len() >= 8 {
                            let id = u32::from_le_bytes(d[4..8].try_into().unwrap());
                            self.on_packet(Key::Relayed(id), &d[8..]);
                        } else if d.starts_with(b"GDMR") {
                            let t = String::from_utf8_lossy(&d[4..]).into_owned();
                            if let Some(c) = t.strip_prefix("SPECOK ") {
                                if self.relay_code.as_deref() != Some(c.trim()) || self.relay_ka.elapsed() > Duration::from_secs(4) {
                                    // the line tools grep for
                                    log(&format!("host: SPECTATE CODE {}", c.trim()));
                                }
                                self.relay_code = Some(c.trim().to_string());
                            } else if t.starts_with("ERR") {
                                log(&format!("host: relay: {t}"));
                            }
                        }
                    } else {
                        self.on_packet(Key::Direct(a), &d);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break, // ICMP reset from a gone spectator
            }
        }
    }

    /// Once per tick: the replay's new confirmed records into the stream, packets in, stream chunks out.
    pub fn update(&mut self, rec: &Replay, ended: bool) {
        if self.stream.is_empty() {
            self.stream = stream_header(rec);
        }
        for (pads, h) in &rec.ticks[self.streamed_ticks.min(rec.ticks.len())..] {
            crate::play_net::encode_tick(pads, *h, &mut self.stream);
        }
        self.streamed_ticks = rec.ticks.len();
        if ended && !self.ended {
            self.ended = true;
            log(&format!("host: stream complete, {} ticks, {} bytes", self.streamed_ticks, self.stream.len()));
        }
        if self.relay.is_some() && self.relay_ka.elapsed() > Duration::from_secs(5) {
            self.relay_ka = Instant::now();
            self.relay_text(&format!("SPECHOST {}", self.relay_code.as_deref().unwrap_or("-")));
            self.relay_text("KA");
        }
        self.poll();
        let now = Instant::now();
        let len = self.stream.len();
        let keys: Vec<Key> = self.peers.keys().copied().collect();
        for k in keys {
            let mut out = Vec::new();
            {
                let p = self.peers.get_mut(&k).unwrap();
                if !p.accepted {
                    continue;
                }
                // go back to the ack when it stalls (a lost chunk)
                if p.next > p.acked && now.duration_since(p.last_ack) > Duration::from_millis(250) {
                    p.next = p.acked;
                    p.last_ack = now;
                }
                while p.next < len && p.next < p.acked + WINDOW * CHUNK {
                    let end = (p.next + CHUNK).min(len);
                    let mut b = vec![b'D'];
                    b.extend_from_slice(&(p.next as u32).to_le_bytes());
                    b.extend_from_slice(&self.stream[p.next..end]);
                    out.push(b);
                    p.next = end;
                }
            }
            for b in &out {
                self.send(k, b);
            }
            if self.ended {
                let mut b = vec![b'E'];
                b.extend_from_slice(&(len as u32).to_le_bytes());
                self.send(k, &b);
            }
        }
        // a spectator silent for 15 s is gone
        self.peers.retain(|k, p| {
            let alive = now.duration_since(p.last_seen) < Duration::from_secs(15);
            if !alive {
                log(&format!("host: spectator {k:?} timed out"));
            }
            alive
        });
    }

    /// The match is over: keep serving until every spectator holds the whole stream (at most `secs`).
    pub fn drain(&mut self, rec: &Replay, secs: u64) {
        let t0 = Instant::now();
        while t0.elapsed() < Duration::from_secs(secs) {
            self.update(rec, true);
            if self.peers.values().all(|p| p.done) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        log(&format!(
            "host: {} spectators joined, {} refused, {} still connected at the end ({} with the whole stream)",
            self.joined,
            self.refused,
            self.peers.len(),
            self.peers.values().filter(|p| p.done).count()
        ));
    }
}

// ------------------------------------------------------------------------------------------------ spectator

enum Via {
    Direct(SocketAddr),
    Relay { relay: SocketAddr },
}

/// A spectator's connection: the stream so far and its parse.
pub struct SpecClient {
    sock: UdpSocket,
    via: Via,
    buf: Vec<u8>,
    rec_at: usize,
    pub total: Option<usize>,
    last_ack: Instant,
    last_rx: Instant,
    pub delay: usize,
    /// ticks to run per render tick at most while catching up
    pub catch_up: usize,
    pub lost: bool,
}

impl SpecClient {
    fn send(&self, body: &[u8]) {
        let mut b = Vec::new();
        match self.via {
            Via::Direct(a) => {
                b.extend_from_slice(MAGIC);
                b.extend_from_slice(body);
                let _ = self.sock.send_to(&b, a);
            }
            Via::Relay { relay } => {
                b.extend_from_slice(b"GDMS");
                b.extend_from_slice(MAGIC);
                b.extend_from_slice(body);
                let _ = self.sock.send_to(&b, relay);
            }
        }
    }

    /// `target`: `HOST:PORT` (direct) or `relay:HOST:PORT/CODE`. Blocks until the host accepts and the stream header
    /// is in (at most `timeout`); returns the replay header (no ticks yet).
    pub fn connect(target: &str, build: u64, mods: (u64, String), timeout: Duration) -> Result<(SpecClient, Replay), String> {
        let (via, code) = match target.strip_prefix("relay:") {
            Some(r) => {
                let (addr, code) = r.rsplit_once('/').ok_or("--spectate relay:HOST:PORT/CODE")?;
                (Via::Relay { relay: resolve(addr)? }, Some(code.to_ascii_uppercase()))
            }
            None => (Via::Direct(resolve(target)?), None),
        };
        let v6 = match &via {
            Via::Direct(a) => a.is_ipv6(),
            Via::Relay { relay } => relay.is_ipv6(),
        };
        let bind = if v6 { "[::]:0" } else { "0.0.0.0:0" };
        let sock = UdpSocket::bind(bind).map_err(|e| format!("spectate bind: {e}"))?;
        sock.set_read_timeout(Some(Duration::from_millis(50))).map_err(|e| e.to_string())?;
        let mut c = SpecClient { sock, via, buf: Vec::new(), rec_at: 0, total: None, last_ack: Instant::now(), last_rx: Instant::now(), delay: 6, catch_up: 16, lost: false };
        let t0 = Instant::now();
        let mut relay_ok = code.is_none();
        let mut accepted = false;
        let mut last_hello = Instant::now() - Duration::from_secs(1);
        let mut rbuf = [0u8; 2048];
        loop {
            if t0.elapsed() > timeout {
                return Err(format!("spectate {target}: no answer in {} s{}", timeout.as_secs(), if relay_ok { "" } else { " (relay)" }));
            }
            if last_hello.elapsed() > Duration::from_millis(250) {
                last_hello = Instant::now();
                if !relay_ok {
                    if let (Via::Relay { relay }, Some(code)) = (&c.via, &code) {
                        let _ = c.sock.send_to(format!("GDMRSPEC {code}").as_bytes(), relay);
                    }
                } else if !accepted {
                    let mut b = vec![b'H', VERSION];
                    b.extend_from_slice(&build.to_le_bytes());
                    b.extend_from_slice(&mods.0.to_le_bytes());
                    b.extend_from_slice(mods.1.as_bytes());
                    c.send(&b);
                } else {
                    c.ack();
                }
            }
            let Ok((n, _)) = c.sock.recv_from(&mut rbuf) else { continue };
            let d = &rbuf[..n];
            if d.starts_with(b"GDMR") {
                let t = String::from_utf8_lossy(&d[4..]).into_owned();
                if t.starts_with("SPECID") {
                    relay_ok = true;
                    last_hello = Instant::now() - Duration::from_secs(1);
                } else if t.starts_with("ERR") {
                    return Err(format!("spectate relay: {t}"));
                }
                continue;
            }
            let d = if d.starts_with(b"GDMS") { &d[4..] } else { d };
            if d.len() < 5 || &d[..4] != MAGIC {
                continue;
            }
            match d[4] {
                b'R' => return Err(format!("spectate: the host refused: {}", String::from_utf8_lossy(&d[5..]))),
                b'A' => {
                    if !accepted {
                        log("connected, waiting for the stream header");
                    }
                    accepted = true;
                }
                _ => {
                    accepted = true;
                    c.on_data(d);
                }
            }
            if let Some(r) = c.header()? {
                log(&format!("joined: {} (stream at {} bytes)", r.meta.iter().map(|m| format!("{}={}", m.0, m.1)).collect::<Vec<_>>().join(" "), c.buf.len()));
                c.sock.set_nonblocking(true).map_err(|e| e.to_string())?;
                return Ok((c, r));
            }
        }
    }

    fn header_len(&self) -> Option<usize> {
        let hl = u32::from_le_bytes(self.buf.get(8..12)?.try_into().ok()?) as usize;
        (self.buf.len() >= 16 + hl).then_some(16 + hl)
    }

    fn header(&self) -> Result<Option<Replay>, String> {
        let Some(n) = self.header_len() else { return Ok(None) };
        let mut h = self.buf[..n].to_vec();
        h[n - 4..].copy_from_slice(&0u32.to_le_bytes());
        Replay::parse(&h).map(Some)
    }

    fn on_data(&mut self, d: &[u8]) {
        self.last_rx = Instant::now();
        match d[4] {
            b'D' if d.len() >= 9 => {
                let off = u32::from_le_bytes(d[5..9].try_into().unwrap()) as usize;
                // in order only (go-back-N); an old chunk is a duplicate
                if off == self.buf.len() {
                    self.buf.extend_from_slice(&d[9..]);
                }
                if off <= self.buf.len() && self.last_ack.elapsed() > Duration::from_millis(5) {
                    self.ack();
                }
            }
            b'E' if d.len() >= 9 => {
                self.total = Some(u32::from_le_bytes(d[5..9].try_into().unwrap()) as usize);
                self.ack();
            }
            _ => {}
        }
    }

    fn ack(&mut self) {
        self.last_ack = Instant::now();
        let mut b = vec![b'K'];
        b.extend_from_slice(&(self.buf.len() as u32).to_le_bytes());
        self.send(&b);
    }

    /// Packets in; the new complete records appended to `ticks`.
    pub fn poll(&mut self, ticks: &mut Vec<([melee_types::PADStatus; 4], u32)>) {
        let mut rbuf = [0u8; 2048];
        loop {
            match self.sock.recv_from(&mut rbuf) {
                Ok((n, _)) => {
                    let d = rbuf[..n].to_vec();
                    let d = if d.starts_with(b"GDMS") { &d[4..] } else { &d[..] };
                    if d.len() >= 5 && &d[..4] == MAGIC {
                        self.on_data(d);
                    }
                }
                Err(_) => break,
            }
        }
        if self.last_ack.elapsed() > Duration::from_millis(100) {
            self.ack();
        }
        if !self.ended() && self.last_rx.elapsed() > Duration::from_secs(20) && !self.lost {
            self.lost = true;
            log("ERROR: no data from the host for 20 s");
        }
        let Some(h) = self.header_len() else { return };
        if self.rec_at < h {
            self.rec_at = h;
        }
        while self.rec_at + REC_BYTES <= self.buf.len() {
            ticks.push(crate::play_net::decode_tick(&self.buf[self.rec_at..self.rec_at + REC_BYTES]));
            self.rec_at += REC_BYTES;
        }
    }

    /// The host sent the whole stream and it is all here.
    pub fn ended(&self) -> bool {
        self.total.is_some_and(|t| self.buf.len() >= t)
    }

    pub fn bye(&self) {
        self.send(b"B");
    }
}
