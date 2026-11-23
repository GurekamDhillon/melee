//! NAT traversal for a YouGame-pool match, after Hero's own-UDP path (port/runtime/hle/yougame_link.cpp, `stun` and
//! `punch`; GPLv3, ported with his permission):
//! - STUN (RFC 5389 binding request, XOR-MAPPED-ADDRESS) from the netplay port gives this seat's external address,
//!   published in the room candidate as `ext` beside `lan`;
//! - the punch: every seat sends small probes from the netplay port to every address the peer published and answers
//!   each probe it receives. The first address a peer's probe arrives from is the one to use; the peer is reachable
//!   once its ack shows it got ours too. A probe carries its sender's token (the candidate's `token`, passed through
//!   the room), so nobody else can answer for the peer. Probes: `YGP2`, type (0 probe, 1 ack), token (8 bytes LE),
//!   the sender's 1-based port. Slippi's ENet drops a late probe as malformed.
//! The socket is closed afterwards and the match binds the same local port again, so the NAT mapping the punch
//! opened is the one the match uses.

use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

const MAGIC: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];

fn rand64() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(Instant::now().elapsed().as_nanos() ^ std::process::id() as u128);
    h.finish()
}

/// The binding request (20 bytes: type 1, length 0, the magic cookie, a 12-byte transaction id).
pub fn stun_request() -> [u8; 20] {
    let mut r = [0u8; 20];
    r[1] = 1;
    r[4..8].copy_from_slice(&MAGIC);
    r[8..16].copy_from_slice(&rand64().to_le_bytes());
    r[16..20].copy_from_slice(&(rand64() as u32).to_le_bytes());
    r
}

/// The mapped address of a binding success answering `req` (XOR-MAPPED-ADDRESS, else MAPPED-ADDRESS), IPv4 only.
pub fn stun_reply(buf: &[u8], req: &[u8; 20]) -> Option<SocketAddrV4> {
    if buf.len() < 20 || buf[0] != 0x01 || buf[1] != 0x01 || buf[4..20] != req[4..20] {
        return None;
    }
    let len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
    let body = buf.get(20..20 + len)?;
    let mut i = 0;
    let mut plain = None;
    while i + 4 <= body.len() {
        let ty = u16::from_be_bytes([body[i], body[i + 1]]);
        let n = u16::from_be_bytes([body[i + 2], body[i + 3]]) as usize;
        let v = body.get(i + 4..i + 4 + n)?;
        if (ty == 0x0020 || ty == 0x0001) && n >= 8 && v[1] == 0x01 {
            let mut port = u16::from_be_bytes([v[2], v[3]]);
            let mut ip = [v[4], v[5], v[6], v[7]];
            if ty == 0x0020 {
                port ^= 0x2112;
                for k in 0..4 {
                    ip[k] ^= MAGIC[k];
                }
                return Some(SocketAddrV4::new(ip.into(), port));
            }
            plain = Some(SocketAddrV4::new(ip.into(), port));
        }
        i += 4 + ((n + 3) & !3);
    }
    plain
}

/// The external address of `s` as the first answering STUN server sees it (3 tries of 500 ms each per server).
pub fn stun(s: &UdpSocket, servers: &[SocketAddr], cancelled: &dyn Fn() -> bool) -> Option<SocketAddrV4> {
    for to in servers {
        let req = stun_request();
        for _ in 0..3 {
            if cancelled() {
                return None;
            }
            if s.send_to(&req, to).is_err() {
                continue;
            }
            let until = Instant::now() + Duration::from_millis(500);
            while let Some(left) = until.checked_duration_since(Instant::now()) {
                let _ = s.set_read_timeout(Some(left.max(Duration::from_millis(1))));
                let mut buf = [0u8; 512];
                let Ok((n, from)) = s.recv_from(&mut buf) else { break };
                if from != *to {
                    continue; // only the server we asked
                }
                if let Some(a) = stun_reply(&buf[..n], &req) {
                    return Some(a);
                }
            }
        }
    }
    None
}

/// Hero's servers; resolved only in a real run (agent runs pass their own loopback list).
pub const STUN_SERVERS: [&str; 2] = ["stun.cloudflare.com:3478", "stun.l.google.com:19302"];

pub fn resolve(servers: &[&str]) -> Vec<SocketAddr> {
    servers.iter().filter_map(|h| h.to_socket_addrs().ok()?.find(|a| a.is_ipv4())).collect()
}

pub const PROBE: u8 = 0;
pub const ACK: u8 = 1;

pub fn probe_bytes(kind: u8, token: u64, from_port: u8) -> [u8; 14] {
    let mut p = [0u8; 14];
    p[..4].copy_from_slice(b"YGP2");
    p[4] = kind;
    p[5..13].copy_from_slice(&token.to_le_bytes());
    p[13] = from_port;
    p
}

/// (type, token, sender's port) of a probe.
pub fn parse_probe(b: &[u8]) -> Option<(u8, u64, u8)> {
    if b.len() != 14 || &b[..4] != b"YGP2" || b[4] > ACK {
        return None;
    }
    Some((b[4], u64::from_le_bytes(b[5..13].try_into().ok()?), b[13]))
}

/// One peer of the punch.
#[derive(Clone, Debug)]
pub struct Peer {
    /// 1-based port.
    pub port: u8,
    /// My token (my probes to this peer carry it).
    pub mine: u64,
    /// The peer's token (its probes to me carry it).
    pub theirs: u64,
    /// Its published addresses, with their kind ("lan" / "ext").
    pub candidates: Vec<(SocketAddrV4, &'static str)>,
    pub found: bool,
    pub acked: bool,
    pub chosen: Option<SocketAddrV4>,
    pub via: &'static str,
}

impl Peer {
    pub fn new(port: u8, mine: u64, theirs: u64, candidates: Vec<(SocketAddrV4, &'static str)>) -> Peer {
        Peer { port, mine, theirs, candidates, found: false, acked: false, chosen: None, via: "" }
    }
}

/// Punch to every peer within `limit` (Hero: 8 s). True once each answered and acked ours (then 600 ms more of
/// answering, so the other side gets its ack too).
pub fn punch(s: &UdpSocket, my_port: u8, peers: &mut [Peer], limit: Duration, cancelled: &dyn Fn() -> bool, log: &mut dyn FnMut(&str)) -> bool {
    let start = Instant::now();
    let mut next_send = start;
    let mut done_at: Option<Instant> = None;
    loop {
        let now = Instant::now();
        if done_at.is_none() && peers.iter().all(|p| p.found && p.acked) {
            done_at = Some(now);
        }
        if done_at.is_some_and(|d| now - d > Duration::from_millis(600)) {
            return true;
        }
        if cancelled() {
            return false;
        }
        if now - start > limit && done_at.is_none() {
            for p in peers.iter() {
                let tried: Vec<&str> = p.candidates.iter().map(|c| c.1).collect();
                log(&format!(
                    "yougame: connect failed: player {} after {} ms, tried {}, heard {}, acked {}",
                    p.port,
                    (now - start).as_millis(),
                    if tried.is_empty() { "nothing".into() } else { tried.join("+") },
                    if p.found { "yes" } else { "no" },
                    if p.acked { "yes" } else { "no" }
                ));
            }
            return false;
        }
        if now >= next_send {
            for p in peers.iter() {
                let b = probe_bytes(PROBE, p.mine, my_port);
                match p.chosen {
                    Some(c) => {
                        let _ = s.send_to(&b, c);
                    }
                    None => {
                        for (c, _) in &p.candidates {
                            let _ = s.send_to(&b, c);
                        }
                    }
                }
            }
            next_send = now + Duration::from_millis(100);
        }
        let _ = s.set_read_timeout(Some(Duration::from_millis(20)));
        let mut buf = [0u8; 64];
        let Ok((n, from)) = s.recv_from(&mut buf) else { continue };
        let SocketAddr::V4(from) = from else { continue };
        let Some((kind, token, port)) = parse_probe(&buf[..n]) else { continue };
        for p in peers.iter_mut() {
            // the source must be one the peer published, and the token the one it gave us
            let Some(&(_, via)) = p.candidates.iter().find(|c| c.0 == from) else { continue };
            if p.port != port || token != p.theirs {
                continue;
            }
            if !p.found {
                p.found = true;
                p.chosen = Some(from);
                p.via = via;
                log(&format!("yougame: player {} answered after {} ms via {via}", p.port, start.elapsed().as_millis()));
            }
            if kind == PROBE {
                let _ = s.send_to(&probe_bytes(ACK, p.mine, my_port), from);
            } else {
                p.acked = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stun_xor_mapped() {
        let req = stun_request();
        let mut r = vec![0x01, 0x01, 0, 12];
        r.extend_from_slice(&req[4..20]);
        let (ip, port) = ([203u8, 0, 113, 7], 40000u16);
        let xp = port ^ 0x2112;
        r.extend_from_slice(&[0x00, 0x20, 0, 8, 0, 1]);
        r.extend_from_slice(&xp.to_be_bytes());
        for k in 0..4 {
            r.push(ip[k] ^ MAGIC[k]);
        }
        assert_eq!(stun_reply(&r, &req), Some(SocketAddrV4::new(ip.into(), port)));
        let mut other = req;
        other[10] ^= 1;
        assert_eq!(stun_reply(&r, &other), None);
    }

    #[test]
    fn probe_round_trip() {
        let b = probe_bytes(ACK, 0x1122334455667788, 2);
        assert_eq!(parse_probe(&b), Some((ACK, 0x1122334455667788, 2)));
        assert_eq!(parse_probe(&b[..13]), None);
    }

    #[test]
    fn punch_on_loopback_picks_the_working_address() {
        let a = UdpSocket::bind("127.0.0.1:0").unwrap();
        let b = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = |s: &UdpSocket| match s.local_addr().unwrap() {
            SocketAddr::V4(v) => v,
            _ => unreachable!(),
        };
        // a dead "lan" address first (nothing listens there), the working one as "ext"
        let dead = SocketAddrV4::new([127, 0, 0, 1].into(), 9);
        let (ta, tb) = (11u64, 22u64);
        let (aa, ba) = (addr(&a), addr(&b));
        let t = std::thread::spawn(move || {
            let mut pb = [Peer::new(1, tb, ta, vec![(dead, "lan"), (aa, "ext")])];
            let ok = punch(&b, 2, &mut pb, Duration::from_secs(5), &|| false, &mut |_| {});
            (ok, pb[0].chosen, pb[0].via)
        });
        let mut pa = [Peer::new(2, ta, tb, vec![(dead, "lan"), (ba, "ext")])];
        assert!(punch(&a, 1, &mut pa, Duration::from_secs(5), &|| false, &mut |_| {}));
        assert_eq!((pa[0].chosen, pa[0].via), (Some(ba), "ext"));
        assert_eq!(t.join().unwrap(), (true, Some(aa), "ext"));
    }

    #[test]
    fn punch_ignores_a_wrong_token() {
        let a = UdpSocket::bind("127.0.0.1:0").unwrap();
        let b = UdpSocket::bind("127.0.0.1:0").unwrap();
        let SocketAddr::V4(aa) = a.local_addr().unwrap() else { unreachable!() };
        let SocketAddr::V4(ba) = b.local_addr().unwrap() else { unreachable!() };
        let t = std::thread::spawn(move || {
            let mut pb = [Peer::new(1, 99, 1, vec![(aa, "lan")])]; // sends token 99, A expects 2
            punch(&b, 2, &mut pb, Duration::from_millis(800), &|| false, &mut |_| {})
        });
        let mut pa = [Peer::new(2, 1, 2, vec![(ba, "lan")])];
        assert!(!punch(&a, 1, &mut pa, Duration::from_millis(800), &|| false, &mut |_| {}));
        assert!(!pa[0].found);
        let _ = t.join();
    }
}
