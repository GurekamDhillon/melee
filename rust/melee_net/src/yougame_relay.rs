//! The room relay: a match's netplay datagrams carried by the YouGame room's own `msg` relay, for when the punch
//! found no way through (both seats behind NAT, a firewall). A port of the logic of Hero's
//! `port/runtime/hle/yougame_room_relay.h` and of `own_udp_relay` in `find_match` (`yougame_link.cpp`; GPLv3,
//! ported with his permission):
//! - agreement first: a seat whose punch failed sends the room `msg{d:{kind:"mu-relay-ok", v:1, compat}}` and waits
//!   up to 5 s for every other seat's. Only when all agree does the match move onto the relay (a copy without it
//!   never says so, and the search fails as before);
//! - a frame is one room msg with no `to` (the room gives it to the other seats), flushed about every 16 ms:
//!   `{t:"msg", d:{k:"mu.relay", s:<my port>, p:[[<to port>, "<base64>"], ...]}}`, at most 12 KiB each, paced to
//!   60 msg/s and 160 KiB/s (Rooms allows 120 msg/s, 256 KiB/s per socket); a datagram over budget is dropped like
//!   UDP loss and ENet resends what is reliable. Ports are the in-game 1-based ports.
//!
//! Ours keeps the netplay code unchanged: the relay owns a loopback UDP socket that stands in for the opponent. The
//! match's ENet sends to it; each datagram goes into the next frame. Each arriving entry for this seat is sent from
//! that socket to the match's netplay port, so ENet sees it come from the "opponent".

use crate::slippi_mm::{quote, Json};
use std::collections::VecDeque;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};

pub const KIND: &str = "mu.relay";
pub const AGREE_KIND: &str = "mu-relay-ok";
pub const AGREE_MS: u64 = 5000;
pub const FLUSH_MS: u64 = 16;
const MSGS_PER_SEC: f64 = 60.0;
const MSG_BURST: f64 = 6.0;
const BYTES_PER_SEC: f64 = 160.0 * 1024.0;
const BYTE_BURST: f64 = 32.0 * 1024.0;
pub const MAX_FRAME: usize = 12 * 1024;
const MAX_QUEUED: usize = 512;
pub const MAX_DATAGRAM: usize = 1200;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn b64(d: &[u8]) -> String {
    let mut o = String::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let v = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        o.push(B64[(v >> 18 & 63) as usize] as char);
        o.push(B64[(v >> 12 & 63) as usize] as char);
        o.push(if c.len() > 1 { B64[(v >> 6 & 63) as usize] as char } else { '=' });
        o.push(if c.len() > 2 { B64[(v & 63) as usize] as char } else { '=' });
    }
    o
}

/// Strict: padded, no whitespace, at most `max` bytes out.
pub fn unb64(s: &str, max: usize) -> Option<Vec<u8>> {
    let b = s.as_bytes();
    if b.is_empty() || b.len() % 4 != 0 || b.len() / 4 * 3 > max + 2 {
        return None;
    }
    let val = |c: u8| -> i32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as i32,
            b'a'..=b'z' => (c - b'a') as i32 + 26,
            b'0'..=b'9' => (c - b'0') as i32 + 52,
            b'+' => 62,
            b'/' => 63,
            _ => -1,
        }
    };
    let mut out = Vec::with_capacity(b.len() / 4 * 3);
    for (i, q) in b.chunks(4).enumerate() {
        let last = (i + 1) * 4 == b.len();
        let (a, bb) = (val(q[0]), val(q[1]));
        let c = if q[2] == b'=' && last { -2 } else { val(q[2]) };
        let d = if q[3] == b'=' && last { -2 } else { val(q[3]) };
        if a < 0 || bb < 0 || c == -1 || d == -1 || (c == -2 && d != -2) {
            return None;
        }
        out.push((a << 2 | bb >> 4) as u8);
        if c >= 0 {
            out.push(((bb & 15) << 4 | c >> 2) as u8);
        }
        if d >= 0 {
            out.push(((c & 3) << 6 | d) as u8);
        }
    }
    (out.len() <= max).then_some(out)
}

/// The agreement message this seat sends when its punch failed.
pub fn agree_msg(compat: &str) -> String {
    format!(r#"{{"t":"msg","d":{{"kind":"{AGREE_KIND}","v":1,"compat":{}}}}}"#, quote(compat))
}

/// A peer's agreement (`d` of its msg) for this build.
pub fn is_agree(d: &Json, compat: &str) -> bool {
    d.get("kind").and_then(|v| v.str()) == Some(AGREE_KIND) && d.get("v").and_then(|v| v.num()) == Some(1.0) && d.get("compat").and_then(|v| v.str()) == Some(compat)
}

pub fn is_frame(d: &Json) -> bool {
    d.get("k").and_then(|v| v.str()) == Some(KIND)
}

/// Frames for the queued datagrams `(to port, bytes)`: at most `max_msgs`, each at most MAX_FRAME bytes; what does
/// not fit stays queued.
pub fn frames(me: u8, queue: &mut VecDeque<(u8, Vec<u8>)>, max_msgs: usize) -> Vec<String> {
    const ENVELOPE: usize = 64;
    let mut out = Vec::new();
    while !queue.is_empty() && out.len() < max_msgs {
        let mut p = Vec::new();
        let mut size = ENVELOPE;
        while let Some((to, d)) = queue.front() {
            let entry = d.len().div_ceil(3) * 4 + 8;
            if !p.is_empty() && size + entry > MAX_FRAME {
                break;
            }
            p.push(format!("[{to},\"{}\"]", b64(d)));
            size += entry;
            queue.pop_front();
        }
        out.push(format!(r#"{{"t":"msg","d":{{"k":"{KIND}","s":{me},"p":[{}]}}}}"#, p.join(",")));
    }
    out
}

/// The entries of a frame's `d` addressed to `me`, with the sender's port. None for a malformed frame.
pub fn entries_for(d: &Json, me: u8) -> Option<(u8, Vec<Vec<u8>>)> {
    if !is_frame(d) {
        return None;
    }
    let from = d.get("s")?.num()?;
    if from.fract() != 0.0 || !(1.0..=4.0).contains(&from) || from as u8 == me {
        return None;
    }
    let Some(Json::Arr(p)) = d.get("p") else { return None };
    let mut out = Vec::new();
    for e in p {
        let a = e.arr();
        if a.len() != 2 || a[0].num() != Some(me as f64) {
            continue;
        }
        if let Some(b) = a[1].str().and_then(|s| unb64(s, MAX_DATAGRAM)).filter(|b| !b.is_empty()) {
            out.push(b);
        }
    }
    Some((from as u8, out))
}

#[derive(Clone, Debug)]
struct Bucket {
    rate: f64,
    burst: f64,
    tokens: f64,
    last: Option<u64>,
}

impl Bucket {
    fn new(rate: f64, burst: f64) -> Bucket {
        Bucket { rate, burst, tokens: burst, last: None }
    }
    fn refill(&mut self, now: u64) {
        if let Some(l) = self.last {
            self.tokens = self.burst.min(self.tokens + self.rate * now.saturating_sub(l) as f64 / 1000.0);
        }
        self.last = Some(now);
    }
}

/// One seat's relay to its peers. Driven by its owner (the room keeper): `poll` every few ms gives the frames to
/// send; `receive` takes a frame from the room.
pub struct Relay {
    me: u8,
    /// Peer port -> its loopback stand-in socket (the match's ENet sends there for that peer).
    peers: Vec<(u8, UdpSocket)>,
    /// The match's netplay socket (where arriving datagrams go).
    netplay: SocketAddr,
    queue: VecDeque<(u8, Vec<u8>)>,
    msgs: Bucket,
    bytes: Bucket,
    last_flush: u64,
    pub sent: u64,
    pub received: u64,
    pub dropped: u64,
}

impl Relay {
    /// `netplay`: the match's port on this machine. One loopback stand-in per peer port.
    pub fn new(me: u8, peer_ports: &[u8], netplay_port: u16) -> std::io::Result<Relay> {
        let mut peers = Vec::new();
        for &p in peer_ports {
            if p == me || !(1..=4).contains(&p) || peers.iter().any(|(q, _)| *q == p) {
                continue;
            }
            let s = UdpSocket::bind("127.0.0.1:0")?;
            s.set_nonblocking(true)?;
            peers.push((p, s));
        }
        Ok(Relay {
            me,
            peers,
            netplay: SocketAddr::V4(SocketAddrV4::new([127, 0, 0, 1].into(), netplay_port)),
            queue: VecDeque::new(),
            msgs: Bucket::new(MSGS_PER_SEC, MSG_BURST),
            bytes: Bucket::new(BYTES_PER_SEC, BYTE_BURST),
            last_flush: 0,
            sent: 0,
            received: 0,
            dropped: 0,
        })
    }

    /// The address the match sends to for peer `port`.
    pub fn stand_in(&self, port: u8) -> Option<SocketAddrV4> {
        self.peers.iter().find(|(p, _)| *p == port).and_then(|(_, s)| match s.local_addr().ok()? {
            SocketAddr::V4(a) => Some(a),
            _ => None,
        })
    }

    /// Take what the match sent (only from its netplay port) and, every FLUSH_MS, the frames to send now.
    pub fn poll(&mut self, now: u64) -> Vec<String> {
        let mut buf = [0u8; 2048];
        for (p, s) in &self.peers {
            while let Ok((n, from)) = s.recv_from(&mut buf) {
                if from.port() != self.netplay.port() || !from.ip().is_loopback() || n == 0 || n > MAX_DATAGRAM {
                    continue;
                }
                if self.queue.len() >= MAX_QUEUED {
                    self.queue.pop_front();
                    self.dropped += 1;
                }
                self.queue.push_back((*p, buf[..n].to_vec()));
            }
        }
        if now < self.last_flush + FLUSH_MS {
            return Vec::new();
        }
        self.last_flush = now;
        self.flush(now)
    }

    /// One flush now: frames within the budgets.
    pub fn flush(&mut self, now: u64) -> Vec<String> {
        self.msgs.refill(now);
        self.bytes.refill(now);
        let allowed = self.msgs.tokens.max(0.0) as usize;
        if allowed == 0 || self.queue.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for f in frames(self.me, &mut self.queue, allowed.min(2)) {
            if self.bytes.tokens < f.len() as f64 {
                self.dropped += f.matches("\",\"").count().max(1) as u64; // roughly its entries
                continue;
            }
            self.bytes.tokens -= f.len() as f64;
            self.msgs.tokens -= 1.0;
            self.sent += 1;
            out.push(f);
        }
        out
    }

    /// A msg `d` from the room. True when it was a relay frame (consumed, whether or not it was for us).
    pub fn receive(&mut self, d: &Json) -> bool {
        let Some((from, entries)) = entries_for(d, self.me) else { return is_frame(d) };
        if let Some((_, s)) = self.peers.iter().find(|(p, _)| *p == from) {
            for e in entries {
                let _ = s.send_to(&e, self.netplay);
                self.received += 1;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip_and_strict() {
        for n in 0..10usize {
            let d: Vec<u8> = (0..n as u8).map(|i| i.wrapping_mul(37)).collect();
            if n == 0 {
                assert_eq!(unb64(&b64(&d), 100), None);
            } else {
                assert_eq!(unb64(&b64(&d), 100).as_deref(), Some(&d[..]));
            }
        }
        assert_eq!(b64(b"Man"), "TWFu");
        assert_eq!(unb64("TW=u", 10), None);
        assert_eq!(unb64("TWF", 10), None);
        assert_eq!(unb64("TWFu", 2), None);
    }

    #[test]
    fn frames_split_and_parse() {
        let mut q: VecDeque<(u8, Vec<u8>)> = (0..30).map(|i| (2u8, vec![i as u8; 1000])).collect();
        let f = frames(1, &mut q, 2);
        assert_eq!(f.len(), 2);
        assert!(f.iter().all(|x| x.len() <= MAX_FRAME));
        assert!(!q.is_empty());
        let m = Json::parse(&f[0]).unwrap();
        let d = m.get("d").unwrap();
        let (from, e) = entries_for(d, 2).unwrap();
        assert_eq!(from, 1);
        assert_eq!(e[0], vec![0u8; 1000]);
        assert!(entries_for(d, 3).unwrap().1.is_empty()); // not ours
        assert!(entries_for(d, 1).is_none()); // our own
    }

    #[test]
    fn agreement_matches_compat() {
        let m = Json::parse(&agree_msg("x/1")).unwrap();
        assert!(is_agree(m.get("d").unwrap(), "x/1"));
        assert!(!is_agree(m.get("d").unwrap(), "x/2"));
    }

    #[test]
    fn two_relays_carry_datagrams_both_ways() {
        // two "netplay" sockets and two relays; the frames hop through a fake room (just handed across)
        let na = UdpSocket::bind("127.0.0.1:0").unwrap();
        let nb = UdpSocket::bind("127.0.0.1:0").unwrap();
        na.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
        nb.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
        let mut ra = Relay::new(1, &[2], na.local_addr().unwrap().port()).unwrap();
        let mut rb = Relay::new(2, &[1], nb.local_addr().unwrap().port()).unwrap();
        na.send_to(b"hello b", ra.stand_in(2).unwrap()).unwrap();
        nb.send_to(b"hello a", rb.stand_in(1).unwrap()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let fa = ra.poll(1000);
        let fb = rb.poll(1000);
        assert_eq!((fa.len(), fb.len()), (1, 1));
        assert!(rb.receive(Json::parse(&fa[0]).unwrap().get("d").unwrap()));
        assert!(ra.receive(Json::parse(&fb[0]).unwrap().get("d").unwrap()));
        let mut buf = [0u8; 64];
        let (n, from) = nb.recv_from(&mut buf).unwrap();
        assert_eq!((&buf[..n], from), (&b"hello b"[..], SocketAddr::V4(rb.stand_in(1).unwrap())));
        let (n, from) = na.recv_from(&mut buf).unwrap();
        assert_eq!((&buf[..n], from), (&b"hello a"[..], SocketAddr::V4(ra.stand_in(2).unwrap())));
        // a stranger's datagram to the stand-in is never relayed
        let x = UdpSocket::bind("127.0.0.1:0").unwrap();
        x.send_to(b"nope", ra.stand_in(2).unwrap()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        assert!(ra.poll(2000).is_empty());
    }

    #[test]
    fn budget_paces_frames() {
        let mut r = Relay::new(1, &[2], 9).unwrap();
        let mut total = 0;
        for t in 0..100u64 {
            r.queue.push_back((2, vec![1; 100]));
            r.queue.push_back((2, vec![1; 100]));
            total += r.flush(1000 + t).len(); // 100 ms of flushes far faster than every 16 ms
        }
        // burst 6 plus 60/s over 0.1 s = 12 at most
        assert!(total <= 13, "{total}");
    }
}
