//! The matchmaking-server client of gw_netplay.c (`rdv`, `rnd`): room codes, the server's
//! introduction, hole punching, the relay fallback and the random-opponent queue. The server is
//! `tools/netplay/server/gdmelee_server.py`; packets start with `GDMR` (control, text) or `GDMD`
//! (relayed data).
//!
//! `Rdv` is the shared state; [`RdvTransport`] is the wrapper the netcode sees: it eats the
//! server's control packets, and in relay mode wraps outgoing traffic for the server and unwraps
//! what comes back as if it came from the peer. UPnP is not ported (a host behind a router with no
//! forward uses the server's relay).

use crate::transport::{Addr, Transport};
use std::cell::RefCell;
use std::net::{ToSocketAddrs, UdpSocket};
use std::rc::Rc;
use std::time::Instant;

pub const NP_SERVER_DEFAULT_PORT: u16 = 51600;
pub const NP_RELAY_AFTER_MS: u32 = 3000;
pub const NP_ALPHABET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandState {
    Off,
    Waiting,
    Matched,
    Timeout,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Rand {
    pub state: RandState,
    pub host: bool,
    pub code: String,
    pub peer_pub: String,
    pub peer_lan: String,
    pub waiting: i32,
    pub since: u32,
    pub next_send: u32,
    pub mods_hash: u64,
}

impl Default for Rand {
    fn default() -> Self {
        Rand { state: RandState::Off, host: false, code: String::new(), peer_pub: String::new(), peer_lan: String::new(), waiting: 0, since: 0, next_send: 0, mods_hash: 0 }
    }
}

pub struct Rdv {
    pub srv: Addr,
    inner: Box<dyn Transport>,
    pub host: bool,
    pub peer_code: String,
    pub have_code: bool,
    pub have_peer: bool,
    pub relay: bool,
    pub got_direct: bool,
    pub code: String,
    pub err: String,
    pub peer: Addr,
    pub status: String,
    pub next_send: u32,
    pub peer_at: u32,
    pub mypub: Option<Addr>,
    pub lan_override: Option<u32>,
    pub punch: Option<Addr>,
    pub next_punch: u32,
    pub rnd: Rand,
    pub force_relay: bool,
    pub log: Vec<String>,
    /// The session exists (the guest may be waiting for the introduction before it does).
    pub net_open: bool,
    t0: Instant,
}

pub type RdvRef = Rc<RefCell<Rdv>>;

pub fn fmt_addr(a: &Addr) -> String {
    a.to_string()
}

/// An address literal of the server's messages: `1.2.3.4:5` or `[v6]:5` (no name lookups).
pub fn addr_lit(s: &str) -> Option<Addr> {
    if let Ok(sa) = s.parse::<std::net::SocketAddr>() {
        return Some(Addr::from_socket(sa));
    }
    crate::addr_parse(s, 1)
}

/// A CODE is "public:port/lan:port"; a plain "ip[:port]" works too (`np_parse_code`).
pub fn parse_code(s: &str) -> Option<(Addr, Option<Addr>)> {
    let (a, lan) = match s.split_once('/') {
        Some((a, l)) => (a, crate::addr_resolve(l, 51500, false).ok().filter(|x| x.ip != 0)),
        None => (s, None),
    };
    let p = crate::addr_resolve(a, 51500, false).ok().filter(|x| x.ip != 0)?;
    Some((p, lan))
}

/// Which of a peer's addresses to dial (`np_pick`).
pub fn pick(mypub: Option<Addr>, pubaddr: Addr, lan: Option<Addr>) -> Addr {
    match lan {
        Some(l) if mypub.map_or(true, |m| m.ip == pubaddr.ip && m.ip6 == pubaddr.ip6) => l,
        _ => pubaddr,
    }
}

/// This machine's LAN address: the interface the default route uses (0 if none).
pub fn lan_ip() -> u32 {
    let Ok(s) = UdpSocket::bind("0.0.0.0:0") else { return 0 };
    if s.connect("8.8.8.8:53").is_err() {
        return 0;
    }
    match s.local_addr() {
        Ok(std::net::SocketAddr::V4(a)) => u32::from(*a.ip()),
        _ => 0,
    }
}

/// STUN (RFC 5389) over the session's socket: the address the router gave THIS socket.
pub fn stun(t: &mut dyn Transport) -> Option<Addr> {
    let servers = [("stun.l.google.com", 19302u16), ("stun1.l.google.com", 19302), ("stun.cloudflare.com", 3478)];
    let t0 = Instant::now();
    for (host, port) in servers {
        let Some(std::net::SocketAddr::V4(sa)) = (host, port).to_socket_addrs().ok().and_then(|mut i| i.find(|a| a.is_ipv4())) else { continue };
        let to = Addr::new(u32::from(*sa.ip()), sa.port());
        let mut req = [0u8; 20];
        req[1] = 0x01;
        req[4..8].copy_from_slice(&[0x21, 0x12, 0xA4, 0x42]);
        for (k, b) in req[8..].iter_mut().enumerate() {
            *b = (t0.elapsed().as_nanos() as u32 >> (k % 8)) as u8 ^ (k as u8).wrapping_mul(37) ^ (std::process::id() as u8);
        }
        let start = Instant::now();
        t.send(&to, &req);
        let mut retried = false;
        while start.elapsed().as_millis() < 1500 {
            let mut rsp = [0u8; 512];
            let mut from = Addr::default();
            let n = t.recv(&mut from, &mut rsp);
            if n <= 0 {
                std::thread::sleep(std::time::Duration::from_millis(5));
                if !retried && start.elapsed().as_millis() > 500 {
                    retried = true;
                    t.send(&to, &req);
                }
                continue;
            }
            let n = n as usize;
            if n >= 20 && rsp[0] == 0x01 && rsp[1] == 0x01 && rsp[4..20] == req[4..20] {
                let mut off = 20;
                while off + 4 <= n {
                    let ty = ((rsp[off] as usize) << 8) | rsp[off + 1] as usize;
                    let len = ((rsp[off + 2] as usize) << 8) | rsp[off + 3] as usize;
                    if off + 4 + len > n {
                        break;
                    }
                    let v = &rsp[off + 4..off + 4 + len];
                    if (ty == 0x0020 || ty == 0x0001) && len >= 8 && v[1] == 0x01 {
                        let mut port = ((v[2] as u16) << 8) | v[3] as u16;
                        let mut ip = u32::from_be_bytes([v[4], v[5], v[6], v[7]]);
                        if ty == 0x0020 {
                            port ^= 0x2112;
                            ip ^= 0x2112A442;
                        }
                        return Some(Addr::new(ip, port));
                    }
                    off += 4 + ((len + 3) & !3);
                }
            }
        }
    }
    None
}

impl Rdv {
    /// `server` is "host[:port]". `inner` is the real socket.
    pub fn new(server: &str, inner: Box<dyn Transport>) -> Result<RdvRef, String> {
        let srv = crate::addr_resolve(server, NP_SERVER_DEFAULT_PORT, false).map_err(|e| format!("matchmaking server \"{server}\": {e}"))?;
        Ok(Rc::new(RefCell::new(Rdv {
            srv,
            inner,
            host: false,
            peer_code: String::new(),
            have_code: false,
            have_peer: false,
            relay: false,
            got_direct: false,
            code: String::new(),
            err: String::new(),
            peer: Addr::default(),
            status: String::new(),
            next_send: 0,
            peer_at: 0,
            mypub: None,
            lan_override: None,
            punch: None,
            next_punch: 0,
            rnd: Rand::default(),
            force_relay: false,
            log: Vec::new(),
            net_open: false,
            t0: Instant::now(),
        })))
    }

    pub fn now(&self) -> u32 {
        self.t0.elapsed().as_millis() as u32
    }

    pub fn ctl(&mut self, text: &str) {
        let mut b = b"GDMR".to_vec();
        b.extend_from_slice(&text.as_bytes()[..text.len().min(196)]);
        let srv = self.srv;
        self.inner.send(&srv, &b);
    }

    /// This side's LAN address text for the server ("-" if unknown).
    pub fn lan(&self, host_port: u16) -> String {
        let ip = self.lan_override.unwrap_or_else(lan_ip);
        if ip == 0 {
            return "-".into();
        }
        let port = if self.host { host_port } else { self.inner.local_port() };
        fmt_addr(&Addr::new(ip, port))
    }

    pub fn local_port(&self) -> u16 {
        self.inner.local_port()
    }

    /// np_rdv_handle: one control message from the server.
    pub fn handle(&mut self, msg: &str) {
        let w: Vec<&str> = msg.split_whitespace().collect();
        let w0 = w.first().copied().unwrap_or("");
        let w1 = w.get(1).copied().unwrap_or("");
        let w2 = w.get(2).copied().unwrap_or("");
        if w0 == "QUEUED" && self.rnd.state == RandState::Waiting {
            self.rnd.waiting = w1.parse().unwrap_or(0);
            return;
        }
        if w0 == "MATCH" && self.rnd.state == RandState::Waiting {
            self.rnd.host = w1 == "HOST";
            self.rnd.code = w2.chars().take(4).collect();
            self.rnd.peer_pub = w.get(3).copied().unwrap_or("").to_string();
            self.rnd.peer_lan = w.get(4).copied().unwrap_or("-").to_string();
            self.rnd.state = RandState::Matched;
            self.log.push(format!("random: matched as {} in room {} with {}", if self.rnd.host { "HOST" } else { "GUEST" }, self.rnd.code, self.rnd.peer_pub));
            return;
        }
        if w0 == "TIMEOUT" && self.rnd.state == RandState::Waiting {
            self.rnd.state = RandState::Timeout;
            return;
        }
        if w0 == "CANCELED" {
            return;
        }
        if w0 == "CODE" && !self.have_code {
            self.code = w1.chars().take(4).collect();
            self.have_code = true;
            if let Some(me) = addr_lit(w2) {
                if self.mypub.is_none() {
                    self.mypub = Some(me);
                }
            }
            self.status = "Waiting for an opponent to join...".into();
        } else if w0 == "PEER" && !self.have_peer {
            let lan = if w2 != "-" { addr_lit(w2).filter(|a| a.ip != 0) } else { None };
            let Some(p) = addr_lit(w1) else { return };
            self.peer = pick(self.mypub, p, lan);
            self.have_peer = true;
            self.peer_at = self.now();
            self.punch = Some(self.peer);
            self.next_punch = 0;
            self.log.push(format!("the server introduced {w1} (dialling {})", fmt_addr(&self.peer)));
        } else if w0 == "RELAY" {
            self.relay = true;
        } else if w0 == "ERR" {
            let r = msg.split_once(' ').map(|x| x.1).unwrap_or("server error");
            self.err = r.to_string();
            self.status = if !self.host && r.contains("no room") { format!("No room {}. Check the code with your host.", self.peer_code) } else { format!("Server: {r}") };
        } else if w0 == "BYE" && !self.host {
            self.err = "The host closed the room".into();
        }
    }

    pub fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        if self.relay {
            if data.len() > 1596 {
                return -1;
            }
            let mut b = b"GDMD".to_vec();
            b.extend_from_slice(data);
            let srv = self.srv;
            self.inner.send(&srv, &b);
            return data.len() as i32;
        }
        self.inner.send(to, data)
    }

    pub fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        let mut b = [0u8; 1600];
        loop {
            let n = self.inner.recv(from, &mut b);
            if n <= 0 {
                return n;
            }
            let n = n as usize;
            if *from == self.srv {
                if n >= 4 && &b[..4] == b"GDMR" {
                    let msg = String::from_utf8_lossy(&b[4..n.min(203)]).into_owned();
                    self.handle(&msg);
                    continue;
                }
                if n >= 4 && &b[..4] == b"GDMD" {
                    if !self.have_peer || n - 4 > buf.len() {
                        continue;
                    }
                    buf[..n - 4].copy_from_slice(&b[4..n]);
                    *from = self.peer;
                    return (n - 4) as i32;
                }
                continue;
            }
            if self.have_peer && from.ip == self.peer.ip {
                self.got_direct = true;
            }
            if n > buf.len() {
                continue;
            }
            buf[..n].copy_from_slice(&b[..n]);
            return n as i32;
        }
    }

    /// np_rdv_service: registration, joining, keepalives, punching and the relay decision.
    /// `net_exists`: the netcode owns the socket (otherwise this drains it itself).
    pub fn service(&mut self, host_port: u16, net_exists: bool) {
        let now = self.now();
        if !net_exists {
            let mut from = Addr::default();
            let mut b = [0u8; 1600];
            while self.recv(&mut from, &mut b) > 0 {}
        }
        if (now.wrapping_sub(self.next_send) as i32) >= 0 {
            let lan = self.lan(host_port);
            if self.host && !self.have_code {
                self.ctl(&format!("REG {lan}"));
                self.next_send = now + 500;
            } else if !self.host && !self.have_peer {
                let m = format!("JOIN {} {lan}", self.peer_code);
                self.ctl(&m);
                self.next_send = now + 500;
            } else {
                self.ctl("KA");
                self.next_send = now + 3000;
            }
        }
        if self.have_peer && !self.relay && self.force_relay {
            self.relay = true;
            self.ctl("RELAY");
            self.log.push("relay forced".into());
        }
        if self.have_peer && !self.relay && !self.got_direct && now.wrapping_sub(self.peer_at) > NP_RELAY_AFTER_MS {
            self.relay = true;
            self.ctl("RELAY");
            self.log.push(format!("no direct path after {NP_RELAY_AFTER_MS} ms - relaying through the server"));
        }
        if let Some(p) = self.punch {
            if (now.wrapping_sub(self.next_punch) as i32) >= 0 && !self.relay {
                self.inner.send(&p, b"GDPH");
                self.next_punch = now + 200;
            }
        }
    }

    /// Queue for a random opponent (`gw_Netplay_RandomBegin`'s server half).
    pub fn begin_random(&mut self, mods_hash: u64) {
        self.rnd = Rand { state: RandState::Waiting, since: self.now(), mods_hash, ..Rand::default() };
    }

    /// np_rand_poll's queue step: repeat RAND every 2 s. Returns false on a server error.
    pub fn rand_poll(&mut self, host_port: u16) -> bool {
        let now = self.now();
        let mut from = Addr::default();
        let mut b = [0u8; 1600];
        while self.recv(&mut from, &mut b) > 0 {}
        if self.rnd.state == RandState::Waiting {
            if (now.wrapping_sub(self.rnd.next_send) as i32) >= 0 {
                let m = format!("RAND {:016x} {}", self.rnd.mods_hash, self.lan(host_port));
                self.ctl(&m);
                self.rnd.next_send = now + 2000;
            }
            self.status = format!("Looking for an opponent... {}s{}", now.wrapping_sub(self.rnd.since) / 1000, if self.rnd.waiting > 1 { " (someone else is looking)" } else { "" });
            if !self.err.is_empty() {
                self.rnd.state = RandState::Failed;
                return false;
            }
        }
        true
    }

    /// The pair is a room: continue as that room's host or guest (np_rand_poll's MATCHED step).
    pub fn adopt_match(&mut self) {
        self.host = self.rnd.host;
        self.peer_code = self.rnd.code.clone();
        self.code = self.rnd.code.clone();
        self.have_code = self.host;
        let intro = format!("PEER {} {}", self.rnd.peer_pub, self.rnd.peer_lan);
        self.handle(&intro);
    }

    pub fn cancel_random(&mut self) {
        if self.rnd.state == RandState::Waiting {
            self.ctl("RANDCANCEL");
        }
        self.rnd.state = RandState::Off;
    }

    /// Closing a host's session closes its room.
    pub fn bye(&mut self) {
        if self.host && self.have_code {
            self.ctl("BYE");
        }
    }
}

/// What the netcode sees.
pub struct RdvTransport(pub RdvRef);

impl Transport for RdvTransport {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        self.0.borrow_mut().send(to, data)
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        self.0.borrow_mut().recv(from, buf)
    }
    fn local_port(&self) -> u16 {
        self.0.borrow().local_port()
    }
}

impl Drop for RdvTransport {
    fn drop(&mut self) {
        self.0.borrow_mut().bye();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Null;
    impl Transport for Null {
        fn send(&mut self, _: &Addr, d: &[u8]) -> i32 {
            d.len() as i32
        }
        fn recv(&mut self, _: &mut Addr, _: &mut [u8]) -> i32 {
            0
        }
        fn local_port(&self) -> u16 {
            4242
        }
    }

    #[test]
    fn server_messages() {
        let r = Rdv::new("127.0.0.1:51600", Box::new(Null)).unwrap();
        let mut r = r.borrow_mut();
        r.host = true;
        r.handle("CODE ABCD 1.2.3.4:5555");
        assert!(r.have_code && r.code == "ABCD" && r.mypub == Some(Addr::new(0x01020304, 5555)));
        r.handle("PEER 5.6.7.8:9 192.168.1.5:9");
        assert!(r.have_peer);
        assert_eq!(r.peer, Addr::new(0x05060708, 9), "different public ip: dial the public address");
        let mut g = Rdv::new("127.0.0.1", Box::new(Null)).unwrap();
        let mut g = g.borrow_mut();
        g.mypub = Some(Addr::new(0x05060708, 1));
        g.handle("PEER 5.6.7.8:9 192.168.1.5:9");
        assert_eq!(g.peer, Addr::new(0xC0A80105, 9), "same public ip: dial the LAN address");
        g.rnd.state = RandState::Waiting;
        g.handle("MATCH HOST WXYZ 9.9.9.9:1 10.0.0.2:1");
        assert_eq!((g.rnd.state, g.rnd.host, g.rnd.code.as_str()), (RandState::Matched, true, "WXYZ"));
        assert_eq!(parse_code("1.2.3.4:5/10.0.0.1:6").unwrap().1, Some(Addr::new(0x0A000001, 6)));
    }
}
