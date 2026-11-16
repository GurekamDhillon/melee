//! Live spectator output of a match's `.slp` events (`--spectate-port N`, `--relay-url URL`): display only, results
//! still come from the upload and its re-simulation.
//!
//! Where the bytes come from: the netplay host starts a headless spectator child (`play --spectate` against its own
//! spectate stream, `spectate.rs`), which plays back the host's *confirmed* frames linearly with the Slippi exporter
//! (`slp_export`) installed. The exporter hands this module every byte it appends to the replay's `raw` element, as
//! each frame finalises: the stream is the final `.slp`'s event stream, byte for byte, and never holds a predicted
//! frame. Nothing here runs on a game thread: the exporter only sends into a channel; the Slippi server (ENet) and the
//! relay (WebSocket) each run on their own thread.
//!
//! A. The Slippi "Dolphin connection" protocol (what Slippi Dolphin serves on 51441), written from the public
//!    consumer side (slippi-js `DolphinConnection`): ENet, 3 channels; the client sends
//!    `{"type":"connect_request","cursor":C}` on channel 0; we answer `connect_reply` (`nick`, `version`, `cursor`) and
//!    then send, reliably on channel 0, one JSON message per cursor step: `start_game`, `game_event` (`payload`: base64
//!    event bytes, whole events only), `end_game`, each with `cursor` and `next_cursor` (= cursor + 1). `keepalive`
//!    while idle; `{"dolphin_closed":true}` when the output closes.
//! B. The relay (docs/design/live-relay.md in the bracket server's repo): the same messages as text frames on a
//!    WebSocket the host opens outbound, the upgrade signed with the player's Ed25519 key (the bracket API's canonical
//!    request signature), resumed from the server's cursor after a reconnect.
//!
//! Back-pressure never reaches the game: a consumer that falls more than `MELEE_LIVE_MAX_BEHIND` messages (default
//! 3600, about a minute) behind is moved to the newest message and the gap is logged.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Raw event bytes per `game_event` at most (whole events; one ENet datagram, the protocol has no fragments here).
const MAX_PAYLOAD: usize = 768;

enum In {
    Start(Vec<u8>),
    Data(Vec<u8>),
    End(Vec<u8>),
}

static TX: OnceLock<Mutex<Sender<In>>> = OnceLock::new();

fn log(m: &str) {
    eprintln!("live: {m}");
}

/// The message log every consumer reads (cursor = index), and whether the output is closing.
#[derive(Default)]
struct Log {
    msgs: Vec<Arc<str>>,
    closed: bool,
}

type Shared = Arc<Mutex<Log>>;

fn max_behind() -> usize {
    std::env::var("MELEE_LIVE_MAX_BEHIND").ok().and_then(|v| v.parse().ok()).unwrap_or(3600).max(1)
}

// -------------------------------------------------------------------------------------------- exporter taps

fn send(m: In) {
    if let Some(tx) = TX.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(m);
        }
    }
}

/// The exporter began a match: Event Payloads + Game Start.
pub fn game_start(raw: &[u8]) {
    send(In::Start(raw.to_vec()));
}
/// One finalised frame's events.
pub fn frame(raw: &[u8]) {
    send(In::Data(raw.to_vec()));
}
/// Game End.
pub fn game_end(raw: &[u8]) {
    send(In::End(raw.to_vec()));
}

pub fn active() -> bool {
    TX.get().is_some()
}

// -------------------------------------------------------------------------------------------- messages

pub fn b64(d: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut o = String::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        o.push(T[(n >> 18) as usize & 63] as char);
        o.push(T[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        o.push(if c.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    o
}

/// Event sizes from an Event Payloads event (command byte -> payload size).
fn sizes_of(raw: &[u8]) -> Option<[u16; 256]> {
    if raw.first() != Some(&0x35) || raw.len() < 2 {
        return None;
    }
    let n = raw[1] as usize;
    let mut s = [0u16; 256];
    s[0x35] = n as u16;
    let mut o = 2;
    while o + 3 <= 1 + n && o + 3 <= raw.len() {
        s[raw[o] as usize] = u16::from_be_bytes([raw[o + 1], raw[o + 2]]);
        o += 3;
    }
    Some(s)
}

/// Split event bytes into chunks of whole events, each at most MAX_PAYLOAD (an event larger than that is one chunk).
fn chunks(raw: &[u8], sizes: &[u16; 256]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let (mut a, mut o) = (0, 0);
    while o < raw.len() {
        let sz = sizes[raw[o] as usize] as usize;
        if sz == 0 {
            // unknown command: keep the rest whole (never cut inside an event)
            o = raw.len();
            break;
        }
        let end = (o + 1 + sz).min(raw.len());
        if end - a > MAX_PAYLOAD && o > a {
            out.push(raw[a..o].to_vec());
            a = o;
        }
        o = end;
    }
    if o > a {
        out.push(raw[a..o].to_vec());
    }
    out
}

struct Hub {
    log: Shared,
    sizes: [u16; 256],
}

impl Hub {
    fn push(&mut self, ty: &str, payload: Option<&[u8]>) {
        let mut l = self.log.lock().unwrap();
        let c = l.msgs.len();
        let m = match payload {
            Some(p) => format!("{{\"type\":\"{ty}\",\"cursor\":{c},\"next_cursor\":{},\"payload\":\"{}\"}}", c + 1, b64(p)),
            None => format!("{{\"type\":\"{ty}\",\"cursor\":{c},\"next_cursor\":{}}}", c + 1),
        };
        l.msgs.push(m.into());
    }
    fn events(&mut self, raw: &[u8]) {
        for c in chunks(raw, &self.sizes) {
            self.push("game_event", Some(&c));
        }
    }
    fn take(&mut self, m: In) {
        match m {
            In::Start(raw) => {
                if let Some(s) = sizes_of(&raw) {
                    self.sizes = s;
                }
                self.push("start_game", None);
                self.events(&raw);
            }
            In::Data(raw) => self.events(&raw),
            In::End(raw) => {
                self.events(&raw);
                self.push("end_game", None);
            }
        }
    }
}

// -------------------------------------------------------------------------------------------- start / finish

pub struct Opts {
    pub slippi_port: Option<u16>,
    pub bind: String,
    pub relay_url: Option<String>,
    pub relay_key: Option<String>,
    pub nick: String,
}

static THREADS: Mutex<Vec<std::thread::JoinHandle<()>>> = Mutex::new(Vec::new());
static SHARED: OnceLock<Shared> = OnceLock::new();

/// Start the output threads (before the exporter runs). Errors (a port in use, an unreadable key) are returned.
pub fn start(o: Opts) -> Result<(), String> {
    let (tx, rx): (Sender<In>, Receiver<In>) = channel();
    let log: Shared = Arc::new(Mutex::new(Log::default()));
    let key = match &o.relay_key {
        Some(p) => Some(read_key(p)?),
        None if o.relay_url.is_some() => return Err("--relay-url needs --relay-key FILE (the bracket client's key)".into()),
        None => None,
    };
    let enet = match o.slippi_port {
        Some(p) => {
            let ip: std::net::Ipv4Addr = o.bind.parse().map_err(|_| format!("--spectate-bind {}: an IPv4 address", o.bind))?;
            let tp = melee_net::transport::UdpTransport::open(u32::from(ip), p).map_err(|e| format!("slippi spectate port {p}: {e}"))?;
            let h = melee_net::enet::Host::new(Box::new(tp), 32, 3);
            log(&format!("Slippi spectate server on udp {}:{}", o.bind, h.local_port()));
            Some(h)
        }
        None => None,
    };
    let mut th = THREADS.lock().unwrap();
    {
        let log = log.clone();
        let nick = o.nick.clone();
        th.push(std::thread::Builder::new().name("live-hub".into()).spawn(move || hub_thread(rx, log, enet, nick)).map_err(|e| e.to_string())?);
    }
    if let (Some(url), Some(k)) = (o.relay_url.clone(), key) {
        let log = log.clone();
        th.push(std::thread::Builder::new().name("live-relay".into()).spawn(move || relay_thread(url, k, log)).map_err(|e| e.to_string())?);
    }
    let _ = SHARED.set(log);
    let _ = TX.set(Mutex::new(tx));
    Ok(())
}

/// The match is over: close the output and give consumers up to `secs` to take the rest.
pub fn finish(secs: u64) {
    let Some(l) = SHARED.get() else { return };
    // let the hub take what the exporter sent last
    std::thread::sleep(Duration::from_millis(50));
    l.lock().unwrap().closed = true;
    let deadline = Instant::now() + Duration::from_secs(secs);
    let hs: Vec<_> = std::mem::take(&mut *THREADS.lock().unwrap());
    for h in hs {
        while !h.is_finished() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    log(&format!("closed ({} messages)", l.lock().unwrap().msgs.len()));
}

// -------------------------------------------------------------------------------------------- A: Slippi (ENet)

struct SPeer {
    /// None until the connect_request
    next: Option<usize>,
    last_send: Instant,
    gap_logged: bool,
}

fn hub_thread(rx: Receiver<In>, log_: Shared, mut enet: Option<melee_net::enet::Host>, nick: String) {
    let mut hub = Hub { log: log_.clone(), sizes: [0; 256] };
    let mut peers: std::collections::HashMap<usize, SPeer> = Default::default();
    let behind = max_behind();
    let mut closing_since: Option<Instant> = None;
    loop {
        while let Ok(m) = rx.try_recv() {
            hub.take(m);
        }
        let (n, closed) = {
            let l = log_.lock().unwrap();
            (l.msgs.len(), l.closed)
        };
        let Some(h) = enet.as_mut() else {
            if closed {
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
            continue;
        };
        while let Some(ev) = h.service() {
            match ev {
                melee_net::enet::Event::Connect { peer, .. } => {
                    log(&format!("slippi: client {} connected", h.peers[peer].addr));
                    peers.insert(peer, SPeer { next: None, last_send: Instant::now(), gap_logged: false });
                }
                melee_net::enet::Event::Disconnect { peer, .. } => {
                    if peers.remove(&peer).is_some() {
                        log("slippi: client left");
                    }
                }
                melee_net::enet::Event::Receive { peer, data, .. } => {
                    let t = String::from_utf8_lossy(&data);
                    let Some(j) = melee_net::slippi_mm::Json::parse(&t) else { continue };
                    if j.get("type").and_then(|v| v.str()) == Some("connect_request") {
                        let want = j.get("cursor").and_then(|v| v.num()).unwrap_or(0.0).max(0.0) as usize;
                        // a cursor this stream never reached (another session's): from the start of this match
                        let c = if want <= n { want } else { 0 };
                        let r = format!("{{\"type\":\"connect_reply\",\"nick\":{},\"version\":\"3.18.0\",\"cursor\":{c}}}", melee_net::slippi_mm::quote(&nick));
                        h.send(peer, 0, r.as_bytes(), true);
                        log(&format!("slippi: handshake, cursor {want} -> {c}"));
                        peers.insert(peer, SPeer { next: Some(c), last_send: Instant::now(), gap_logged: false });
                    }
                }
            }
        }
        let mut all_done = true;
        {
            let l = log_.lock().unwrap();
            for (&p, sp) in peers.iter_mut() {
                let Some(next) = sp.next.as_mut() else { continue };
                if n - *next > behind {
                    log(&format!("slippi: client too slow, {} messages behind: skipped to the newest (gap {}..{})", n - *next, *next, n - 1));
                    *next = n - 1;
                    sp.gap_logged = true;
                }
                while *next < n && h.backlog(p) < 512 {
                    if !h.send(p, 0, l.msgs[*next].as_bytes(), true) {
                        break;
                    }
                    *next += 1;
                    sp.last_send = Instant::now();
                }
                if *next < n || h.backlog(p) > 0 {
                    all_done = false;
                }
                if sp.last_send.elapsed() > Duration::from_secs(1) && !closed {
                    h.send(p, 0, b"{\"type\":\"keepalive\"}", true);
                    sp.last_send = Instant::now();
                }
            }
        }
        if closed {
            let since = *closing_since.get_or_insert_with(Instant::now);
            if all_done || since.elapsed() > Duration::from_secs(10) {
                for &p in peers.keys() {
                    h.send(p, 0, b"{\"dolphin_closed\":true}", true);
                }
                let t = Instant::now();
                while t.elapsed() < Duration::from_millis(300) {
                    while h.service().is_some() {}
                    std::thread::sleep(Duration::from_millis(5));
                }
                for &p in peers.keys() {
                    h.disconnect(p, 0);
                }
                h.flush();
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

// -------------------------------------------------------------------------------------------- B: relay (WebSocket)

/// The Ed25519 seed: a file holding 64 hex digits, or the bracket client's TOML (`secret_key_hex = "..."`).
fn read_key(path: &str) -> Result<ring::signature::Ed25519KeyPair, String> {
    let t = std::fs::read_to_string(path).map_err(|e| format!("--relay-key {path}: {e}"))?;
    let hex = t.lines().find_map(|l| l.trim().strip_prefix("secret_key_hex").map(|r| r.trim_start_matches([' ', '=']).trim().trim_matches('"').to_string())).unwrap_or_else(|| t.trim().to_string());
    let b = unhex(&hex).filter(|b| b.len() == 32).ok_or_else(|| format!("--relay-key {path}: not a 32-byte hex secret key"))?;
    ring::signature::Ed25519KeyPair::from_seed_unchecked(&b).map_err(|_| format!("--relay-key {path}: bad key"))
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// The bracket API's signature headers for `GET target` with an empty body (docs/api.md section 2).
pub fn sign_headers(k: &ring::signature::Ed25519KeyPair, target: &str) -> String {
    use ring::rand::SecureRandom;
    use ring::signature::KeyPair;
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut nonce = [0u8; 16];
    let _ = ring::rand::SystemRandom::new().fill(&mut nonce);
    let nonce = hex(&nonce);
    let canon = format!("GET\n{target}\n{time}\n{nonce}\n{EMPTY_SHA256}");
    let sig = k.sign(canon.as_bytes());
    format!("x-ab-key: {}\r\nx-ab-time: {time}\r\nx-ab-nonce: {nonce}\r\nx-ab-sig: {}\r\n", hex(k.public_key().as_ref()), hex(sig.as_ref()))
}

fn relay_thread(url: String, key: ring::signature::Ed25519KeyPair, log_: Shared) {
    let target = melee_net::slippi_mm::Ws::request_target(&url);
    let behind = max_behind();
    let mut backoff = Duration::from_millis(500);
    let mut next = 0usize;
    let mut first_close: Option<Instant> = None;
    loop {
        if log_.lock().unwrap().closed {
            // closing: a few tries more to deliver the rest, then give up
            if first_close.get_or_insert_with(Instant::now).elapsed() > Duration::from_secs(8) {
                log(&format!("relay: closed with {} of {} messages delivered", next, log_.lock().unwrap().msgs.len()));
                return;
            }
        }
        let hdr = sign_headers(&key, &target);
        let mut ws = match melee_net::slippi_mm::Ws::connect_with(&url, Duration::from_secs(5), melee_net::slippi_mm::Target::Real, &hdr) {
            Ok(w) => w,
            Err(e) => {
                log(&format!("relay: connect failed: {e}; retry in {} ms", backoff.as_millis()));
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(8));
                continue;
            }
        };
        let req = format!("{{\"type\":\"connect_request\",\"cursor\":{next},\"version\":1}}");
        let reply = ws.send_text(&req).and_then(|_| ws.recv_text());
        let resume = match reply.as_deref().ok().and_then(melee_net::slippi_mm::Json::parse) {
            Some(j) if j.get("type").and_then(|v| v.str()) == Some("connect_reply") => j.get("cursor").and_then(|v| v.num()).unwrap_or(0.0).max(0.0) as usize,
            _ => {
                log(&format!("relay: no connect_reply ({:?}); retry in {} ms", reply.err(), backoff.as_millis()));
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(8));
                continue;
            }
        };
        let n = log_.lock().unwrap().msgs.len();
        next = resume.min(n);
        backoff = Duration::from_millis(500);
        log(&format!("relay: connected to {url}, resuming at cursor {next} (server asked {resume})"));
        ws.set_timeouts(None, Some(Duration::from_secs(5)));
        let mut last = Instant::now();
        let err = loop {
            let (batch, closed, n) = {
                let l = log_.lock().unwrap();
                let n = l.msgs.len();
                if n - next > behind {
                    log(&format!("relay: {} messages behind (slow link): skipped to the newest (gap {}..{})", n - next, next, n - 1));
                    next = n - 1;
                }
                (l.msgs[next..n.min(next + 64)].to_vec(), l.closed, n)
            };
            if batch.is_empty() {
                if closed {
                    ws.close();
                    log(&format!("relay: all {n} messages sent"));
                    return;
                }
                if last.elapsed() > Duration::from_secs(2) {
                    if let Err(e) = ws.send_text("{\"type\":\"keepalive\"}") {
                        break e;
                    }
                    last = Instant::now();
                }
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            let mut e = None;
            for m in &batch {
                if let Err(x) = ws.send_text(m) {
                    e = Some(x);
                    break;
                }
                next += 1;
            }
            if let Some(e) = e {
                break e;
            }
            last = Instant::now();
        };
        log(&format!("relay: connection lost at cursor {next}: {err}; reconnecting"));
        std::thread::sleep(backoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648() {
        assert_eq!(b64(b""), "");
        assert_eq!(b64(b"f"), "Zg==");
        assert_eq!(b64(b"fo"), "Zm8=");
        assert_eq!(b64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn chunks_keep_whole_events() {
        let mut s = [0u16; 256];
        s[0x3A] = 400;
        s[0x3C] = 8;
        let mut raw = Vec::new();
        for _ in 0..3 {
            raw.push(0x3A);
            raw.extend(std::iter::repeat(0).take(400));
            raw.push(0x3C);
            raw.extend([0; 8]);
        }
        let c = chunks(&raw, &s);
        assert!(c.len() > 1 && c.iter().all(|x| x.len() <= MAX_PAYLOAD));
        assert_eq!(c.concat(), raw);
    }

    #[test]
    fn signature_matches_the_api_doc_example() {
        // docs/api.md's worked example key (RFC 8032 test key 1): the signature verifies over the canonical string
        use ring::signature::{KeyPair, UnparsedPublicKey, ED25519};
        let k = ring::signature::Ed25519KeyPair::from_seed_unchecked(&unhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60").unwrap()).unwrap();
        assert_eq!(hex(k.public_key().as_ref()), "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
        let h = sign_headers(&k, "/v1/live?x=1");
        let get = |n: &str| h.lines().find_map(|l| l.strip_prefix(n)).unwrap().trim().to_string();
        let canon = format!("GET\n/v1/live?x=1\n{}\n{}\n{EMPTY_SHA256}", get("x-ab-time:"), get("x-ab-nonce:"));
        let pk = UnparsedPublicKey::new(&ED25519, k.public_key().as_ref());
        assert!(pk.verify(canon.as_bytes(), &unhex(&get("x-ab-sig:")).unwrap()).is_ok());
    }
}
