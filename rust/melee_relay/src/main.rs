//! `melee_relay`: the self-hostable matchmaking server for netplay. One UDP port, IPv4 and IPv6.
//!
//!     melee_relay [--port 51600] [--bind 0.0.0.0] [--bind6 ::|off] [--quiet]
//!
//! Protocol (the one `melee_net::app::rdv` speaks, same as tools/netplay/server/gdmelee_server.py):
//!   control, text: `GDMR` + words
//!     host  -> REG <lan>             server -> CODE <code> <your-public-addr>
//!     guest -> JOIN <code> <lan>     server -> PEER <other-public> <other-lan>  (to both; each then hole-punches)
//!                                    server -> ERR <reason>
//!     either -> KA                   keepalive (answered KA)
//!     any   -> RAND <hash> <lan>     random queue: QUEUED <n> / MATCH HOST|GUEST <code> <pub> <lan> / TIMEOUT
//!     any   -> RANDCANCEL            -> CANCELED
//!     either -> RELAY                no direct path: the server forwards this pair's GDMD packets
//!     either -> BYE                  leave (a host's BYE closes the room; the other side gets BYE)
//!   relay: `GDMD` + payload, forwarded verbatim to the partner.
//! Addresses are `1.2.3.4:5` or `[v6]:5`. Nothing is stored on disk; rooms die after ROOM_IDLE / PAIR_IDLE seconds.

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const ROOM_IDLE: f32 = 120.0;
const PAIR_IDLE: f32 = 900.0;
const QUEUE_IDLE: f32 = 10.0;
const QUEUE_MAX: f32 = 300.0;
const MAX_ROOMS: usize = 5000;
const MAX_QUEUE: usize = 5000;

struct Room {
    host: SocketAddr,
    host_lan: String,
    guest: Option<SocketAddr>,
    guest_lan: String,
    seen: Instant,
    random_mods: Option<String>,
}

struct Waiter {
    mods: String,
    lan: String,
    since: Instant,
    seen: Instant,
}

struct Server {
    socks: Vec<UdpSocket>,
    rooms: HashMap<String, Room>,
    by_addr: HashMap<SocketAddr, String>,
    queue: HashMap<SocketAddr, Waiter>,
    rng: u64,
    relayed: u64,
    quiet: bool,
}

fn canon(a: SocketAddr) -> SocketAddr {
    match a {
        SocketAddr::V6(v) => match v.ip().to_ipv4_mapped() {
            Some(v4) => SocketAddr::new(v4.into(), v.port()),
            None => a,
        },
        _ => a,
    }
}

impl Server {
    fn log(&self, m: &str) {
        if !self.quiet {
            eprintln!("melee_relay: {m}");
        }
    }
    fn raw(&self, to: SocketAddr, d: &[u8]) {
        for s in &self.socks {
            if s.local_addr().map_or(false, |l| l.is_ipv6() == to.is_ipv6()) {
                let _ = s.send_to(d, to);
                return;
            }
        }
    }
    fn send(&self, to: SocketAddr, text: &str) {
        let mut b = b"GDMR".to_vec();
        b.extend_from_slice(text.as_bytes());
        self.raw(to, &b);
    }
    fn new_code(&mut self) -> String {
        loop {
            let mut c = String::new();
            for _ in 0..4 {
                self.rng ^= self.rng << 13;
                self.rng ^= self.rng >> 7;
                self.rng ^= self.rng << 17;
                c.push(ALPHABET[(self.rng % 32) as usize] as char);
            }
            if !self.rooms.contains_key(&c) {
                return c;
            }
        }
    }
    fn drop_room(&mut self, code: &str, why: &str) {
        if let Some(r) = self.rooms.remove(code) {
            self.log(&format!("room {code} closed ({why})"));
            for a in [Some(r.host), r.guest].into_iter().flatten() {
                if self.by_addr.get(&a).map_or(false, |c| c == code) {
                    self.by_addr.remove(&a);
                }
            }
        }
    }
    fn leave(&mut self, a: SocketAddr, why: &str) {
        let Some(code) = self.by_addr.get(&a).cloned() else { return };
        let Some(r) = self.rooms.get_mut(&code) else { return };
        if r.host == a {
            self.drop_room(&code, why);
        } else {
            if r.guest == Some(a) {
                r.guest = None;
                r.guest_lan.clear();
            }
            self.by_addr.remove(&a);
        }
    }
    fn send_match(&self, code: &str, to: SocketAddr) {
        let r = &self.rooms[code];
        let host = to == r.host;
        let other = if host { r.guest.unwrap() } else { r.host };
        let lan = if host { &r.guest_lan } else { &r.host_lan };
        self.send(to, &format!("MATCH {} {code} {other} {}", if host { "HOST" } else { "GUEST" }, if lan.is_empty() { "-" } else { lan }));
    }

    fn packet(&mut self, data: &[u8], from: SocketAddr) {
        if data.len() < 4 {
            return;
        }
        let code = self.by_addr.get(&from).cloned();
        if &data[..4] == b"GDMD" {
            let Some(code) = code else { return };
            let Some(r) = self.rooms.get_mut(&code) else { return };
            let Some(g) = r.guest else { return };
            r.seen = Instant::now();
            let other = if from == r.host { g } else { r.host };
            if self.by_addr.get(&other) != Some(&code) {
                return;
            }
            self.raw(other, data);
            self.relayed += 1;
            return;
        }
        if &data[..4] != b"GDMR" {
            return;
        }
        let text = String::from_utf8_lossy(&data[4..]).into_owned();
        let w: Vec<&str> = text.split_whitespace().collect();
        let Some(cmd) = w.first().map(|c| c.to_ascii_uppercase()) else { return };
        let arg = |i: usize| w.get(i).copied().unwrap_or("").to_string();
        if let Some(c) = &code {
            if let Some(r) = self.rooms.get_mut(c) {
                r.seen = Instant::now();
            }
        }
        match cmd.as_str() {
            "REG" => {
                let lan = arg(1);
                let mine = code.clone().filter(|c| self.rooms.get(c).map_or(false, |r| r.host == from));
                let c = if let Some(c) = mine {
                    // a repeat or a rematch: same code, the room waits for its guest again
                    let r = self.rooms.get_mut(&c).unwrap();
                    if !lan.is_empty() {
                        r.host_lan = lan;
                    }
                    if let Some(g) = r.guest.take() {
                        if self.by_addr.get(&g) == Some(&c) {
                            self.by_addr.remove(&g);
                        }
                    }
                    c
                } else {
                    if self.rooms.len() >= MAX_ROOMS {
                        return self.send(from, "ERR server full");
                    }
                    if code.is_some() {
                        self.leave(from, "host re-registered");
                    }
                    let c = self.new_code();
                    self.rooms.insert(c.clone(), Room { host: from, host_lan: lan.clone(), guest: None, guest_lan: String::new(), seen: Instant::now(), random_mods: None });
                    self.by_addr.insert(from, c.clone());
                    self.log(&format!("room {c} opened by {from} (lan {lan})"));
                    c
                };
                self.queue.remove(&from);
                self.send(from, &format!("CODE {c} {from}"));
            }
            "JOIN" => {
                if w.len() < 2 {
                    return self.send(from, "ERR no code");
                }
                let c = arg(1).to_ascii_uppercase().replace('-', "");
                let lan = arg(2);
                let Some(t) = self.rooms.get(&c) else { return self.send(from, "ERR no room with that code") };
                if t.host == from {
                    return self.send(from, "ERR that is your own room");
                }
                if code.as_ref().map_or(false, |x| *x != c) {
                    self.leave(from, "joined another room");
                }
                self.queue.remove(&from);
                let t = self.rooms.get_mut(&c).unwrap();
                if let Some(old) = t.guest.filter(|g| *g != from) {
                    if self.by_addr.get(&old) == Some(&c) {
                        self.by_addr.remove(&old);
                    }
                }
                t.guest = Some(from);
                t.guest_lan = lan.clone();
                t.seen = Instant::now();
                let (h, hl) = (t.host, if t.host_lan.is_empty() { "-".to_string() } else { t.host_lan.clone() });
                self.by_addr.insert(from, c.clone());
                self.send(from, &format!("PEER {h} {hl}"));
                self.send(h, &format!("PEER {from} {}", if lan.is_empty() { "-" } else { &lan }));
                self.log(&format!("room {c}: {from} joined {h}"));
            }
            "KA" => {
                if code.is_some() {
                    self.send(from, "KA");
                }
            }
            "RELAY" => {
                let Some(c) = code else { return };
                let r = &self.rooms[&c];
                if let Some(g) = r.guest {
                    let other = if from == r.host { g } else { r.host };
                    self.send(other, "RELAY");
                    self.log(&format!("room {c}: relaying"));
                }
            }
            "RAND" => {
                let mods = if w.len() > 1 { arg(1) } else { "-".into() };
                let lan = arg(2);
                if let Some(c) = &code {
                    let r = &self.rooms[c];
                    if r.guest.is_some() && r.random_mods.as_deref() == Some(mods.as_str()) {
                        return self.send_match(c, from);
                    }
                }
                let now = Instant::now();
                if !self.queue.contains_key(&from) {
                    if self.queue.len() >= MAX_QUEUE && !self.queue.values().any(|q| q.mods == mods) {
                        return self.send(from, "ERR server queue full");
                    }
                    if code.is_some() {
                        self.leave(from, "went to random matchmaking");
                    }
                    self.queue.insert(from, Waiter { mods: mods.clone(), lan: lan.clone(), since: now, seen: now });
                } else {
                    let me = self.queue.get_mut(&from).unwrap();
                    me.seen = now;
                    me.mods = mods.clone();
                    if !lan.is_empty() {
                        me.lan = lan;
                    }
                }
                let since = self.queue[&from].since;
                let other = self.queue.iter().filter(|(a, q)| **a != from && q.mods == mods).min_by_key(|(_, q)| q.since).map(|(a, _)| *a);
                let Some(other) = other else {
                    let n = self.queue.values().filter(|q| q.mods == mods).count();
                    return self.send(from, &format!("QUEUED {n}"));
                };
                let (h, g) = if self.queue[&other].since <= since { (other, from) } else { (from, other) };
                let hw = self.queue.remove(&h).unwrap();
                let gw = self.queue.remove(&g).unwrap();
                let c = self.new_code();
                self.rooms.insert(c.clone(), Room { host: h, host_lan: hw.lan, guest: Some(g), guest_lan: gw.lan, seen: now, random_mods: Some(hw.mods) });
                self.by_addr.insert(h, c.clone());
                self.by_addr.insert(g, c.clone());
                self.send_match(&c, h);
                self.send_match(&c, g);
                self.log(&format!("random: room {c} - host {h}, guest {g}"));
            }
            "RANDCANCEL" => {
                self.queue.remove(&from);
                if let Some(c) = &code {
                    if self.rooms.get(c).map_or(false, |r| r.random_mods.is_some()) {
                        self.leave(from, "random matchmaking canceled");
                    }
                }
                self.send(from, "CANCELED");
            }
            "BYE" => {
                let Some(c) = code else { return };
                let r = &self.rooms[&c];
                let other = if from == r.host { r.guest } else { Some(r.host) };
                if let Some(o) = other {
                    self.send(o, "BYE");
                }
                self.drop_room(&c, &format!("bye from {from}"));
            }
            _ => {}
        }
    }

    fn reap(&mut self) {
        let now = Instant::now();
        let mut timed = Vec::new();
        self.queue.retain(|a, q| {
            if now.duration_since(q.since).as_secs_f32() > QUEUE_MAX {
                timed.push(*a);
                false
            } else {
                now.duration_since(q.seen).as_secs_f32() <= QUEUE_IDLE
            }
        });
        for a in timed {
            self.send(a, "TIMEOUT");
        }
        let idle: Vec<String> = self.rooms.iter().filter(|(_, r)| now.duration_since(r.seen).as_secs_f32() > if r.guest.is_some() { PAIR_IDLE } else { ROOM_IDLE }).map(|(c, _)| c.clone()).collect();
        for c in idle {
            self.drop_room(&c, "idle");
        }
        if self.relayed > 0 {
            self.log(&format!("{} rooms, {} packets relayed in the last 10 s", self.rooms.len(), self.relayed));
            self.relayed = 0;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let port: u16 = get("--port").and_then(|p| p.parse().ok()).unwrap_or(51600);
    let bind = get("--bind").unwrap_or_else(|| "0.0.0.0".into());
    let bind6 = get("--bind6").unwrap_or_else(|| "::".into());
    let mut socks = Vec::new();
    match UdpSocket::bind((bind.as_str(), port)) {
        Ok(s) => socks.push(s),
        Err(e) => {
            eprintln!("melee_relay: bind {bind}:{port}: {e}");
            std::process::exit(1);
        }
    }
    if bind6 != "off" {
        match UdpSocket::bind((bind6.as_str(), port)) {
            Ok(s) => socks.push(s),
            Err(e) => eprintln!("melee_relay: no IPv6 ({bind6}:{port}: {e}); IPv4 only"),
        }
    }
    for s in &socks {
        s.set_nonblocking(true).expect("nonblocking");
        eprintln!("melee_relay: listening on udp {}", s.local_addr().unwrap());
    }
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64) | 1;
    let mut sv = Server { socks, rooms: HashMap::new(), by_addr: HashMap::new(), queue: HashMap::new(), rng: seed ^ std::process::id() as u64, relayed: 0, quiet: args.iter().any(|a| a == "--quiet") };
    let mut last_reap = Instant::now();
    let mut buf = [0u8; 2048];
    loop {
        let mut any = false;
        for i in 0..sv.socks.len() {
            loop {
                match sv.socks[i].recv_from(&mut buf) {
                    Ok((n, a)) => {
                        any = true;
                        let d = buf[..n].to_vec();
                        sv.packet(&d, canon(a));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    // ICMP port unreachable from a gone peer (WSAECONNRESET) and the like: keep serving
                    Err(_) => break,
                }
            }
        }
        if last_reap.elapsed().as_secs() >= 10 {
            sv.reap();
            last_reap = Instant::now();
        }
        if !any {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
