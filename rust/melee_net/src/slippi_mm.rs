//! The client half of Slippi's matchmaking protocol (`Externals/SlippiRustExtensions/matchmaking`: `protocol.rs`,
//! `client.rs`; `SlippiMatchmaking.cpp`'s `match` handler), enough to be paired through a matchmaking server with
//! a Slippi Dolphin. Plain WebSocket (RFC 6455 client: masked frames, text only, ping answered) over TCP.
//!
//! **Loopback by default.** [`join`] refuses any address that is not 127.0.0.1 (the local mock server,
//! `tools/rust-port/slippi/mock_mm.py`). Only [`join_to`] with [`Target::Real`], which the `melee slippi-netplay
//! --mm-real` flag alone selects, reaches another host: `ws://` or `wss://` (TLS: the `real-net` feature), with the
//! account's play key in the user record. No STUN: the only candidate is the host address.
//!
//! Flow: `join{user, search{mode, connectCode}, appVersion, port, ipAddressLan, candidates, natType}` ->
//! `joined{ticketId, token}`, `status`... -> `match{matchId, isHost, players[{port, isLocalPlayer, ipAddress,
//! ipAddressLan, ...}], stages, items}` or `error{error}`.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// A tiny JSON value (what the server sends: objects, arrays, strings, numbers, bools, null).
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(v) => v.iter().find(|(a, _)| a == k).map(|(_, b)| b),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        if let Json::Str(s) = self { Some(s) } else { None }
    }
    pub fn num(&self) -> Option<f64> {
        if let Json::Num(n) = self { Some(*n) } else { None }
    }
    pub fn bool(&self) -> Option<bool> {
        if let Json::Bool(b) = self { Some(*b) } else { None }
    }
    pub fn arr(&self) -> &[Json] {
        if let Json::Arr(a) = self { a } else { &[] }
    }

    pub fn parse(s: &str) -> Option<Json> {
        let b = s.as_bytes();
        let mut i = 0;
        let v = parse_value(b, &mut i)?;
        skip_ws(b, &mut i);
        (i == b.len()).then_some(v)
    }
}

fn skip_ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && b[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn parse_str(b: &[u8], i: &mut usize) -> Option<String> {
    if b.get(*i) != Some(&b'"') {
        return None;
    }
    *i += 1;
    let mut out: Vec<u8> = Vec::new();
    while *i < b.len() {
        let c = b[*i];
        *i += 1;
        match c {
            b'"' => return String::from_utf8(out).ok(),
            b'\\' => {
                let e = *b.get(*i)?;
                *i += 1;
                match e {
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    b'b' => out.push(8),
                    b'f' => out.push(12),
                    b'u' => {
                        let h = std::str::from_utf8(b.get(*i..*i + 4)?).ok()?;
                        *i += 4;
                        let c = char::from_u32(u32::from_str_radix(h, 16).ok()?).unwrap_or('?');
                        let mut t = [0u8; 4];
                        out.extend_from_slice(c.encode_utf8(&mut t).as_bytes());
                    }
                    o => out.push(o),
                }
            }
            o => out.push(o),
        }
    }
    None
}

fn parse_value(b: &[u8], i: &mut usize) -> Option<Json> {
    skip_ws(b, i);
    match *b.get(*i)? {
        b'{' => {
            *i += 1;
            let mut v = Vec::new();
            loop {
                skip_ws(b, i);
                if b.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Some(Json::Obj(v));
                }
                let k = parse_str(b, i)?;
                skip_ws(b, i);
                if b.get(*i) != Some(&b':') {
                    return None;
                }
                *i += 1;
                v.push((k, parse_value(b, i)?));
                skip_ws(b, i);
                match b.get(*i) {
                    Some(b',') => *i += 1,
                    Some(b'}') => {}
                    _ => return None,
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            loop {
                skip_ws(b, i);
                if b.get(*i) == Some(&b']') {
                    *i += 1;
                    return Some(Json::Arr(v));
                }
                v.push(parse_value(b, i)?);
                skip_ws(b, i);
                match b.get(*i) {
                    Some(b',') => *i += 1,
                    Some(b']') => {}
                    _ => return None,
                }
            }
        }
        b'"' => parse_str(b, i).map(Json::Str),
        b't' if b[*i..].starts_with(b"true") => {
            *i += 4;
            Some(Json::Bool(true))
        }
        b'f' if b[*i..].starts_with(b"false") => {
            *i += 5;
            Some(Json::Bool(false))
        }
        b'n' if b[*i..].starts_with(b"null") => {
            *i += 4;
            Some(Json::Null)
        }
        _ => {
            let s = *i;
            while *i < b.len() && (b[*i].is_ascii_digit() || b"+-.eE".contains(&b[*i])) {
                *i += 1;
            }
            std::str::from_utf8(&b[s..*i]).ok()?.parse().ok().map(Json::Num)
        }
    }
}

/// JSON string literal.
pub fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o + "\""
}

/// The Slippi netplay version this port speaks, sent as the join's `appVersion` (Hero's `SLIPPI_SEMVER`,
/// port/runtime/hle/slippi_net.cpp). Slippi's servers refuse a join whose version they do not accept, so this is a
/// release number, never a beta tag.
pub const SLIPPI_SEMVER: &str = "3.7.0";

/// Hero's `release_version_valid` (slippi_version.h): 2 to 4 dotted numbers of at most 8 digits. An absent,
/// truncated or malformed version is not evidence of compatibility.
pub fn release_version_valid(v: &str) -> bool {
    let (mut parts, mut digits) = (0u32, 0u32);
    for c in v.chars() {
        if c.is_ascii_digit() {
            digits += 1;
            if digits > 8 {
                return false;
            }
        } else if c == '.' && digits > 0 && parts < 3 {
            parts += 1;
            digits = 0;
        } else {
            return false;
        }
    }
    digits > 0 && (2..=3).contains(&parts)
}

/// Hero's `version_newer`: dotted `a` newer than `b` ("3.6.10" > "3.6.4"); a tag after the numbers is ignored, a
/// missing part counts as 0, an empty `a` is never newer.
pub fn version_newer(a: &str, b: &str) -> bool {
    fn parts(s: &str) -> [u64; 4] {
        let mut out = [0u64; 4];
        let mut rest = s;
        for slot in out.iter_mut() {
            let n = rest.bytes().take_while(u8::is_ascii_digit).count();
            *slot = rest[..n].parse().unwrap_or(0);
            rest = &rest[n..];
            match rest.strip_prefix('.') {
                Some(r) => rest = r,
                None => break,
            }
        }
        out
    }
    parts(a) > parts(b)
}

/// The version gate (Hero's `Matchmaking::FindMatch`): the account (user.json `latestVersion`) or the service names
/// a newer Slippi than this build speaks, so it stays out of matchmaking. The message is Hero's, with this game's name.
pub fn version_gate(latest: &str) -> Result<(), String> {
    if release_version_valid(latest) && version_newer(latest, SLIPPI_SEMVER) {
        Err(format!("Slippi {latest} is out. Update Melee"))
    } else {
        Ok(())
    }
}

/// What we tell the server about ourselves.
#[derive(Clone, Debug)]
pub struct JoinRequest {
    pub uid: String,
    /// the account's play key (empty for the mock); never logged
    pub play_key: String,
    /// our LAN address as the server is told it (`ipAddressLan`, the host candidate); empty: 127.0.0.1
    pub lan_ip: String,
    pub display_name: String,
    pub connect_code: String,
    /// 2 = direct (0 ranked, 1 unranked, 3 teams, 4 party)
    pub mode: u8,
    /// the opponent's code for direct mode
    pub search_code: String,
    pub app_version: String,
    /// our netplay (ENet) UDP port; the netplay client binds it after the match is found
    pub port: u16,
    /// Hero's `build_join`: the netplay port's public address as STUN saw it (the `srflx` candidate; it also
    /// becomes the join's `port`). None: the server falls back to the address it observes.
    pub reflexive: Option<std::net::SocketAddrV4>,
    /// "cone", "symmetric" or "unknown" (Hero's `classify_nat` of two STUN answers); empty: not sent
    pub nat_type: String,
}

#[derive(Clone, Debug)]
pub struct MatchPlayer {
    pub port: u8,
    pub is_local: bool,
    pub uid: String,
    pub display_name: String,
    pub connect_code: String,
    pub ip_address: String,
    pub ip_address_lan: String,
}

/// The server's `match` message.
#[derive(Clone, Debug)]
pub struct MatchResult {
    pub match_id: String,
    /// is_decider for the netplay client
    pub is_host: bool,
    pub players: Vec<MatchPlayer>,
    pub stages: Vec<u16>,
    pub items: u32,
}

impl MatchResult {
    /// `m_local_player_idx` (port - 1 of the isLocalPlayer entry).
    pub fn local_index(&self) -> Option<u8> {
        self.players.iter().find(|p| p.is_local).map(|p| p.port.saturating_sub(1))
    }
    /// The remote addresses as Hero's `remote_address` (slippi_address.h) picks them: (index, first, fallback).
    /// Same external IP and a distinct LAN address: the LAN address first, the public one as the fallback.
    pub fn remote_addresses(&self) -> Vec<(u8, String, Option<String>)> {
        let Some(me) = self.players.iter().find(|p| p.is_local) else { return Vec::new() };
        let my_ext = me.ip_address.split(':').next().unwrap_or("").to_string();
        self.players
            .iter()
            .filter(|p| !p.is_local)
            .map(|p| {
                let ext = p.ip_address.split(':').next().unwrap_or("");
                let lan = &p.ip_address_lan;
                if ext == my_ext && !lan.is_empty() && *lan != p.ip_address {
                    (p.port.saturating_sub(1), lan.clone(), Some(p.ip_address.clone()))
                } else {
                    (p.port.saturating_sub(1), p.ip_address.clone(), None)
                }
            })
            .collect()
    }

    pub fn from_json(j: &Json) -> Option<MatchResult> {
        // `match` (the WebSocket service) or `get-ticket-resp` (the ENet service of Slippi Dolphin 3.x)
        if !matches!(j.get("type")?.str()?, "match" | "get-ticket-resp") {
            return None;
        }
        let s = |o: &Json, k: &str| o.get(k).and_then(|v| v.str()).unwrap_or("").to_string();
        Some(MatchResult {
            match_id: s(j, "matchId"),
            is_host: j.get("isHost").and_then(|v| v.bool()).unwrap_or(false),
            players: j
                .get("players")?
                .arr()
                .iter()
                .map(|p| MatchPlayer {
                    port: p.get("port").and_then(|v| v.num()).unwrap_or(0.0) as u8,
                    is_local: p.get("isLocalPlayer").and_then(|v| v.bool()).unwrap_or(false),
                    uid: s(p, "uid"),
                    display_name: s(p, "displayName"),
                    connect_code: s(p, "connectCode"),
                    ip_address: s(p, "ipAddress"),
                    ip_address_lan: s(p, "ipAddressLan"),
                })
                .collect(),
            stages: j.get("stages").map(|a| a.arr().iter().filter_map(|v| v.num()).map(|n| n as u16).collect()).unwrap_or_default(),
            items: j.get("items").and_then(|v| v.num()).unwrap_or(0.0) as u32,
        })
    }
}

/// `join` as `protocol.rs` serializes it (the search code goes as its Shift-JIS bytes; ASCII codes are those bytes).
pub fn join_message(r: &JoinRequest) -> String {
    // Hero's `join_json` (slippi_mm_service.cpp, the 3.7.0 service): `port` is the public (STUN) port when known,
    // candidates are `{ip, port, type}` (srflx first, then the LAN host), natType from two STUN answers
    let lan_ip = if r.lan_ip.is_empty() { "127.0.0.1" } else { &r.lan_ip };
    let lan = format!("{lan_ip}:{}", r.port);
    let mut cands = Vec::new();
    if let Some(x) = r.reflexive {
        cands.push(format!("{{\"ip\":{},\"port\":{},\"type\":\"srflx\"}}", quote(&x.ip().to_string()), x.port()));
    }
    cands.push(format!("{{\"ip\":{},\"port\":{},\"type\":\"host\"}}", quote(lan_ip), r.port));
    let mut s = format!(
        "{{\"type\":\"join\",\"user\":{{\"uid\":{},\"playKey\":{},\"connectCode\":{},\"displayName\":{}}},\"search\":{{\"mode\":{},\"connectCode\":[{}]}},\"appVersion\":{},\"port\":{},\"ipAddressLan\":{},\"candidates\":[{}]",
        quote(&r.uid),
        quote(&r.play_key),
        quote(&r.connect_code),
        quote(&r.display_name),
        r.mode,
        r.search_code.bytes().map(|b| b.to_string()).collect::<Vec<_>>().join(","),
        quote(&r.app_version),
        r.reflexive.map(|x| x.port()).unwrap_or(r.port),
        quote(&lan),
        cands.join(","),
    );
    if !r.nat_type.is_empty() {
        s += &format!(",\"natType\":{}", quote(&r.nat_type));
    }
    s.push('}');
    s
}

/// Which servers a connection may reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// `ws://127.0.0.1:PORT/...` only (the default: the local mock).
    Loopback,
    /// Any host, `ws://` or `wss://` (Slippi's real matchmaking; `--mm-real` only).
    Real,
    /// `ws://` to a literal private IPv4 address (loopback, 10/8, 172.16/12, 192.168/16, link-local): a mock server on
    /// another machine of the same LAN, or across a WSL/VM virtual switch (`--mm-lan`, the cross-OS tests).
    Lan,
}

/// The address a [`Target::Lan`] connection may reach.
fn lan_ok(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local()
}

enum Stream {
    Plain(TcpStream),
    #[cfg(feature = "real-net")]
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Read for Stream {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Plain(s) => s.read(b),
            #[cfg(feature = "real-net")]
            Stream::Tls(s) => s.read(b),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Plain(s) => s.write(b),
            #[cfg(feature = "real-net")]
            Stream::Tls(s) => s.write(b),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Plain(s) => s.flush(),
            #[cfg(feature = "real-net")]
            Stream::Tls(s) => s.flush(),
        }
    }
}

/// A WebSocket client connection (text frames).
pub struct Ws {
    s: Stream,
    buf: Vec<u8>,
    mask: u32,
}

impl Ws {
    /// `ws://127.0.0.1:PORT/PATH` only.
    pub fn connect(url: &str, timeout: Duration) -> Result<Ws, String> {
        Self::connect_to(url, timeout, Target::Loopback)
    }

    /// With [`Target::Real`]: `ws://` or `wss://` to any host (a name is resolved; `wss://` defaults to port 443).
    pub fn connect_to(url: &str, timeout: Duration, target: Target) -> Result<Ws, String> {
        let (tls, rest) = match (url.strip_prefix("ws://"), url.strip_prefix("wss://")) {
            (Some(r), _) => (false, r),
            (_, Some(r)) => (true, r),
            _ => return Err("only ws:// or wss:// URLs".into()),
        };
        let (hostport, path) = rest.split_once('/').map(|(a, b)| (a, format!("/{b}"))).unwrap_or((rest, "/".into()));
        let (host, port) = match hostport.split_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().map_err(|_| "bad port")?),
            None if tls => (hostport, 443),
            None => (hostport, 80),
        };
        let addr = match target {
            Target::Loopback => {
                if tls {
                    return Err("wss:// is a real server: it needs --mm-real".into());
                }
                let ip: Ipv4Addr = host.parse().map_err(|_| "the matchmaking host must be a literal IPv4 address".to_string())?;
                if !ip.is_loopback() {
                    return Err(format!("{ip} is not loopback: only a local mock matchmaking server is allowed (a real server needs --mm-real)"));
                }
                SocketAddrV4::new(ip, port).into()
            }
            Target::Real => (host, port).to_socket_addrs().map_err(|e| format!("resolve {host}: {e}"))?.next().ok_or("no address")?,
            Target::Lan => {
                if tls {
                    return Err("wss:// is a real server: it needs --mm-real".into());
                }
                let ip: Ipv4Addr = host.parse().map_err(|_| "the matchmaking host must be a literal IPv4 address".to_string())?;
                if !lan_ok(ip) {
                    return Err(format!("{ip} is not a private address: --mm-lan reaches a mock server on the LAN only"));
                }
                SocketAddrV4::new(ip, port).into()
            }
        };
        let s = TcpStream::connect_timeout(&addr, timeout).map_err(|e| e.to_string())?;
        s.set_nodelay(true).ok();
        s.set_read_timeout(Some(timeout)).ok();
        let s = if tls { Self::tls(host, s)? } else { Stream::Plain(s) };
        let key = "R0QtbWVsZWUtc2xpcHBpMw=="; // any 16 bytes, base64
        let req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n");
        let mut ws = Ws { s, buf: Vec::new(), mask: 0x5EED_1234 };
        ws.s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        loop {
            if let Some(p) = ws.buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&ws.buf[..p]).to_string();
                ws.buf.drain(..p + 4);
                if !head.starts_with("HTTP/1.1 101") {
                    return Err(format!("upgrade refused: {}", head.lines().next().unwrap_or("")));
                }
                return Ok(ws);
            }
            ws.fill()?;
        }
    }

    #[cfg(feature = "real-net")]
    fn tls(host: &str, s: TcpStream) -> Result<Stream, String> {
        use std::sync::Arc;
        let roots = rustls::RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
        let cfg = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_root_certificates(roots)
            .with_no_client_auth();
        let name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| e.to_string())?;
        let c = rustls::ClientConnection::new(Arc::new(cfg), name).map_err(|e| e.to_string())?;
        Ok(Stream::Tls(Box::new(rustls::StreamOwned::new(c, s))))
    }
    #[cfg(not(feature = "real-net"))]
    fn tls(_host: &str, _s: TcpStream) -> Result<Stream, String> {
        Err("wss:// needs a build with the melee_net `real-net` feature".into())
    }

    /// How long one read may block (`recv_text` then returns a "timed out" error; buffered data is kept).
    pub fn set_read_timeout(&mut self, d: Duration) {
        let t = Some(d.max(Duration::from_millis(1)));
        match &self.s {
            Stream::Plain(s) => s.set_read_timeout(t).ok(),
            #[cfg(feature = "real-net")]
            Stream::Tls(s) => s.sock.set_read_timeout(t).ok(),
        };
    }

    /// Close the connection (a close frame; errors ignored).
    pub fn close(&mut self) {
        let _ = self.send_frame(8, &[0x03, 0xE8]);
    }

    fn fill(&mut self) -> Result<(), String> {
        let mut t = [0u8; 4096];
        match self.s.read(&mut t) {
            Ok(0) => Err("connection closed".into()),
            Ok(n) => {
                self.buf.extend_from_slice(&t[..n]);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn send_frame(&mut self, op: u8, data: &[u8]) -> Result<(), String> {
        let mut f = vec![0x80 | op];
        let n = data.len();
        if n < 126 {
            f.push(0x80 | n as u8);
        } else if n < 65536 {
            f.push(0x80 | 126);
            f.extend_from_slice(&(n as u16).to_be_bytes());
        } else {
            f.push(0x80 | 127);
            f.extend_from_slice(&(n as u64).to_be_bytes());
        }
        self.mask = self.mask.wrapping_mul(1664525).wrapping_add(1013904223);
        let m = self.mask.to_be_bytes();
        f.extend_from_slice(&m);
        f.extend(data.iter().enumerate().map(|(i, b)| b ^ m[i % 4]));
        self.s.write_all(&f).map_err(|e| e.to_string())
    }

    pub fn send_text(&mut self, t: &str) -> Result<(), String> {
        self.send_frame(1, t.as_bytes())
    }

    /// The next text message (pings answered, other frames skipped); `Err` on close or timeout.
    pub fn recv_text(&mut self) -> Result<String, String> {
        loop {
            if self.buf.len() >= 2 {
                let (b0, b1) = (self.buf[0], self.buf[1]);
                let (mut n, mut p) = ((b1 & 0x7F) as usize, 2usize);
                if n == 126 && self.buf.len() >= 4 {
                    n = u16::from_be_bytes([self.buf[2], self.buf[3]]) as usize;
                    p = 4;
                } else if n == 127 && self.buf.len() >= 10 {
                    n = u64::from_be_bytes(self.buf[2..10].try_into().unwrap()) as usize;
                    p = 10;
                }
                let masked = b1 & 0x80 != 0;
                let need = p + if masked { 4 } else { 0 } + n;
                if (b1 & 0x7F) < 126 || p > 2 {
                    if self.buf.len() >= need {
                        let mut data: Vec<u8> = self.buf[need - n..need].to_vec();
                        if masked {
                            let m: [u8; 4] = self.buf[p..p + 4].try_into().unwrap();
                            data.iter_mut().enumerate().for_each(|(i, b)| *b ^= m[i % 4]);
                        }
                        self.buf.drain(..need);
                        match b0 & 0x0F {
                            1 => return String::from_utf8(data).map_err(|e| e.to_string()),
                            8 => {
                                let code = if data.len() >= 2 { u16::from_be_bytes([data[0], data[1]]) } else { 1005 };
                                return Err(format!("server closed the connection (code {code})"));
                            }
                            9 => self.send_frame(10, &data)?,
                            _ => {}
                        }
                        continue;
                    }
                }
            }
            self.fill()?;
        }
    }
}

/// Join the queue and wait for `match` (or `error`), up to `timeout`.
pub fn join(url: &str, req: &JoinRequest, timeout: Duration, on_msg: impl FnMut(&str)) -> Result<MatchResult, String> {
    join_to(url, req, timeout, Target::Loopback, on_msg, |_| false)
}

/// [`join`] to `target`. `leave(msg)` is asked after every server message; true closes the connection at once
/// (the real-server check leaves the queue as soon as it has been accepted) and returns `Err("left")`.
pub fn join_to(url: &str, req: &JoinRequest, timeout: Duration, target: Target, mut on_msg: impl FnMut(&str), mut leave: impl FnMut(&str) -> bool) -> Result<MatchResult, String> {
    let t0 = Instant::now();
    let mut ws = Ws::connect_to(url, Duration::from_secs(5), target)?;
    ws.send_text(&join_message(req))?;
    loop {
        if t0.elapsed() > timeout {
            return Err("matchmaking timed out".into());
        }
        let t = match ws.recv_text() {
            Ok(t) => t,
            Err(e) if e.contains("timed out") || e.contains("os error 10060") => continue,
            Err(e) => return Err(e),
        };
        on_msg(&t);
        if leave(&t) {
            ws.close();
            return Err("left".into());
        }
        let Some(j) = Json::parse(&t) else { continue };
        match j.get("type").and_then(|v| v.str()) {
            Some("match") => return MatchResult::from_json(&j).ok_or_else(|| "malformed match".to_string()),
            Some("error") => {
                let err = j.get("error").and_then(|v| v.str()).unwrap_or("?").to_string();
                // Hero (slippi_mm_service): the service names the latest release; past this build's, the gate's words
                if let Some(latest) = j.get("latestVersion").and_then(|v| v.str()).filter(|v| !v.is_empty()) {
                    eprintln!("slippi: mm: the service names Slippi {latest} as the latest (this build speaks {SLIPPI_SEMVER})");
                    if let Err(gate) = version_gate(latest) {
                        return Err(format!("server error: {gate}"));
                    }
                }
                return Err(format!("server error: {err}"));
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The ENet matchmaking service of production Slippi Dolphin (Ishiiruka 3.x, `SlippiMatchmaking.cpp` v3.6.4, read as a
// protocol reference only): `mm.slippi.gg:43113` (`mm2.slippi.gg` is the dev host, for builds whose version says
// "dev"). The client binds its netplay port (41000 + rand % 10000), ENet-connects to the server with 3 channels and
// connect data 0, and sends JSON text as reliable packets on channel 0:
//   `{"type":"create-ticket","user":{uid,playKey,connectCode,displayName},"search":{"mode":M,"connectCode":[bytes]},
//     "appVersion":V,"ipAddressLan":"ip:port"}`
// -> `create-ticket-resp{error?}` within 5 s, then (any time later) `get-ticket-resp{error?, latestVersion?, matchId,
// isHost, players[{uid, displayName, connectCode, port, isLocalPlayer, ipAddress, ipAddressLan, chatMessages, rank}],
// stages, items}`. The server's view of the source port is the hole punch: the netplay client binds the same port
// once the mm connection is closed.

/// The production ENet matchmaking host (`MM_HOST_PROD`).
pub const ENET_MM_HOST: &str = "mm.slippi.gg";
pub const ENET_MM_PORT: u16 = 43113;

/// `create-ticket` as SlippiMatchmaking 3.x builds it (no `port`/`candidates`/`natType`: the server observes those).
pub fn create_ticket_message(r: &JoinRequest) -> String {
    let code_bytes: Vec<String> = r.search_code.bytes().map(|b| b.to_string()).collect();
    let lan = format!("{}:{}", if r.lan_ip.is_empty() { "127.0.0.1" } else { &r.lan_ip }, r.port);
    format!(
        "{{\"type\":\"create-ticket\",\"user\":{{\"uid\":{},\"playKey\":{},\"connectCode\":{},\"displayName\":{}}},\"search\":{{\"mode\":{},\"connectCode\":[{}]}},\"appVersion\":{},\"ipAddressLan\":{}}}",
        quote(&r.uid),
        quote(&r.play_key),
        quote(&r.connect_code),
        quote(&r.display_name),
        r.mode,
        code_bytes.join(","),
        quote(&r.app_version),
        quote(&lan)
    )
}

/// The local address the OS would use to reach `to` (`getLocalAddress`: a connected UDP socket, nothing sent).
pub fn local_ip_towards(to: std::net::SocketAddr) -> Option<Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect(to).ok()?;
    match s.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) if !ip.is_unspecified() => Some(ip),
        _ => None,
    }
}

/// Join through an ENet matchmaking server (`server` = "host:port"). With [`Target::Loopback`] only a literal
/// 127.0.0.1 server is allowed; [`Target::Real`] (`--mm-real`) resolves any name. The UDP socket is bound to
/// `bind_ip:req.port` and closed (dropped) before this returns, so the netplay client can bind that port.
/// `req.lan_ip` empty: the address the OS routes to the server from. `leave(msg)` as in [`join_to`], asked after
/// the ticket is accepted.
pub fn join_enet(server: &str, bind_ip: u32, req: &JoinRequest, timeout: Duration, target: Target, mut on_msg: impl FnMut(&str), mut leave: impl FnMut(&str) -> bool) -> Result<MatchResult, String> {
    use crate::enet::{Event, Host};
    use crate::transport::{Addr, UdpTransport};
    let (host, port) = server.rsplit_once(':').map(|(h, p)| (h, p.parse::<u16>().unwrap_or(ENET_MM_PORT))).unwrap_or((server, ENET_MM_PORT));
    let sa: std::net::SocketAddr = match target {
        Target::Loopback => {
            let ip: Ipv4Addr = host.parse().map_err(|_| "the matchmaking host must be a literal IPv4 address".to_string())?;
            if !ip.is_loopback() {
                return Err(format!("{ip} is not loopback: a real server needs --mm-real"));
            }
            SocketAddrV4::new(ip, port).into()
        }
        Target::Real => (host, port).to_socket_addrs().map_err(|e| format!("resolve {host}: {e}"))?.find(|a| a.is_ipv4()).ok_or("no IPv4 address")?,
        Target::Lan => {
            let ip: Ipv4Addr = host.parse().map_err(|_| "the matchmaking host must be a literal IPv4 address".to_string())?;
            if !lan_ok(ip) {
                return Err(format!("{ip} is not a private address: --mm-lan reaches a mock server on the LAN only"));
            }
            SocketAddrV4::new(ip, port).into()
        }
    };
    let mut req = req.clone();
    if req.lan_ip.is_empty() {
        req.lan_ip = if target == Target::Loopback { "127.0.0.1".into() } else { local_ip_towards(sa).map(|i| i.to_string()).unwrap_or_default() };
    }
    let udp = UdpTransport::open(bind_ip, req.port).map_err(|e| format!("bind {}: {e}", req.port))?;
    let mut h = Host::new(Box::new(udp), 1, 3);
    let peer = h.connect(Addr::from_socket(sa), 3, 0).ok_or("no peer slot")?;
    let t0 = Instant::now();
    let (mut connected, mut sent, mut ticket_ok) = (false, false, false);
    let finish = |h: &mut Host| {
        h.disconnect(peer, 0);
        let t = Instant::now();
        while t.elapsed() < Duration::from_millis(500) {
            h.flush();
            if let Some(Event::Disconnect { .. }) = h.service() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    loop {
        if t0.elapsed() > timeout {
            finish(&mut h);
            return Err("matchmaking timed out".into());
        }
        if !connected && t0.elapsed() > Duration::from_secs(10) {
            return Err("could not connect to the mm server".into());
        }
        if connected && !sent {
            sent = h.send(peer, 0, create_ticket_message(&req).as_bytes(), true);
        }
        h.flush();
        let Some(ev) = h.service() else {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        };
        match ev {
            Event::Connect { .. } => connected = true,
            Event::Disconnect { .. } => return Err("lost connection to the mm server".into()),
            Event::Receive { data, .. } => {
                let t = String::from_utf8_lossy(&data).to_string();
                on_msg(&t);
                let Some(j) = Json::parse(&t) else { continue };
                let err = j.get("error").and_then(|v| v.str()).unwrap_or("").to_string();
                match j.get("type").and_then(|v| v.str()) {
                    Some("create-ticket-resp") => {
                        if !err.is_empty() {
                            finish(&mut h);
                            return Err(format!("server error: {err}"));
                        }
                        ticket_ok = true;
                        if leave(&t) {
                            finish(&mut h);
                            return Err("left".into());
                        }
                    }
                    Some("get-ticket-resp") if ticket_ok => {
                        finish(&mut h);
                        if !err.is_empty() {
                            return Err(format!("server error: {err}"));
                        }
                        return MatchResult::from_json(&j).ok_or_else(|| "malformed get-ticket-resp".to_string());
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_gate_follows_hero() {
        assert!(release_version_valid("3.7.0") && !release_version_valid("3.6") && !release_version_valid("4.0.0-mainline-beta.20"));
        assert!(!release_version_valid("") && !release_version_valid("3") && !release_version_valid("1.2.3.4.5"));
        assert!(version_newer("3.6.10", "3.6.4") && !version_newer("3.6.4", "3.6.10") && !version_newer("", "3.7.0"));
        assert!(!version_newer("3.7", "3.7.0") && version_newer("3.7.1", "3.7.0") && version_newer("4.0.0", "3.7.0"));
        assert!(version_gate("3.7.0").is_ok() && version_gate("3.5.2").is_ok() && version_gate("garbage").is_ok());
        assert_eq!(version_gate("3.8.0").unwrap_err(), "Slippi 3.8.0 is out. Update Melee");
    }

    #[test]
    fn parses_a_match() {
        let t = r#"{"type": "match", "matchId": "m1", "isHost": false, "stages": [2, 32],
            "players": [{"uid": "a", "port": 1, "isLocalPlayer": false, "ipAddress": "127.0.0.1:5000", "ipAddressLan": "127.0.0.1:5000", "displayName": "Aé"},
                        {"uid": "b", "port": 2, "isLocalPlayer": true, "ipAddress": "127.0.0.1:6000", "ipAddressLan": "127.0.0.1:6000"}]}"#;
        let m = MatchResult::from_json(&Json::parse(t).unwrap()).unwrap();
        assert_eq!(m.local_index(), Some(1));
        assert_eq!(m.stages, vec![2, 32]);
        assert_eq!(m.remote_addresses(), vec![(0, "127.0.0.1:5000".to_string(), None)]);
        assert_eq!(m.players[0].display_name, "Aé");
        let j = join_message(&JoinRequest { uid: "u".into(), display_name: "D".into(), connect_code: "AB#1".into(), mode: 2, search_code: "CD#2".into(), app_version: "4.0.0".into(), port: 7, play_key: String::new(), lan_ip: String::new(), reflexive: None, nat_type: String::new() });
        let p = Json::parse(&j).unwrap();
        assert_eq!(p.get("search").unwrap().get("connectCode").unwrap().arr().len(), 4);
        assert_eq!(p.get("port").unwrap().num(), Some(7.0));
        // Hero's 3.7.0 join: srflx candidate first and its port as `port`, `{ip, port, type}` fields, natType
        let j = join_message(&JoinRequest { uid: "u".into(), display_name: "D".into(), connect_code: "AB#1".into(), mode: 1, search_code: String::new(), app_version: "3.7.0".into(), port: 51441, play_key: String::new(), lan_ip: "10.0.0.5".into(), reflexive: Some("1.2.3.4:40000".parse().unwrap()), nat_type: "cone".into() });
        let p = Json::parse(&j).unwrap();
        assert_eq!(p.get("port").unwrap().num(), Some(40000.0));
        let c = p.get("candidates").unwrap().arr();
        assert_eq!((c[0].get("type").unwrap().str(), c[0].get("ip").unwrap().str(), c[0].get("port").unwrap().num()), (Some("srflx"), Some("1.2.3.4"), Some(40000.0)));
        assert_eq!((c[1].get("type").unwrap().str(), c[1].get("ip").unwrap().str(), c[1].get("port").unwrap().num()), (Some("host"), Some("10.0.0.5"), Some(51441.0)));
        assert_eq!(p.get("natType").unwrap().str(), Some("cone"));
        assert_eq!(p.get("ipAddressLan").unwrap().str(), Some("10.0.0.5:51441"));
    }

    #[test]
    fn create_ticket_shape() {
        let r = JoinRequest { uid: "u".into(), display_name: "D".into(), connect_code: "AB#1".into(), mode: 2, search_code: "CD#2".into(), app_version: "3.6.4".into(), port: 41234, play_key: "k".into(), lan_ip: "10.0.0.5".into(), reflexive: None, nat_type: String::new() };
        let p = Json::parse(&create_ticket_message(&r)).unwrap();
        assert_eq!(p.get("type").unwrap().str(), Some("create-ticket"));
        assert_eq!(p.get("ipAddressLan").unwrap().str(), Some("10.0.0.5:41234"));
        assert_eq!(p.get("search").unwrap().get("mode").unwrap().num(), Some(2.0));
        assert!(join_enet("10.0.0.1:43113", 0x7F000001, &r, Duration::from_millis(10), Target::Loopback, |_| {}, |_| false).is_err());
        let g = r#"{"type":"get-ticket-resp","matchId":"x","isHost":true,"players":[{"port":1,"isLocalPlayer":true,"ipAddress":"1.2.3.4:5"},{"port":2,"isLocalPlayer":false,"ipAddress":"1.2.3.4:6","ipAddressLan":"10.0.0.2:6"}]}"#;
        let m = MatchResult::from_json(&Json::parse(g).unwrap()).unwrap();
        assert_eq!(m.remote_addresses(), vec![(1, "10.0.0.2:6".to_string(), Some("1.2.3.4:6".to_string()))]);
    }

    #[test]
    fn refuses_non_loopback() {
        assert!(Ws::connect("ws://10.0.0.1:1/v1/queue", Duration::from_millis(10)).is_err());
        assert!(Ws::connect("ws://matchmaking.example:1/v1/queue", Duration::from_millis(10)).is_err());
        assert!(Ws::connect("wss://matchmaking.example/v1/queue", Duration::from_millis(10)).is_err());
    }

    #[test]
    fn lan_refuses_public_and_tls() {
        // --mm-lan (also for Unranked): a mock on a private address only; these fail before any connection is tried
        let t = Duration::from_millis(10);
        for url in ["ws://8.8.8.8:43110/v1/queue", "ws://matchmaking.example:43110/v1/queue", "wss://192.168.1.2:43110/v1/queue", "wss://matchmaking.example/v1/queue"] {
            let e = Ws::connect_to(url, t, Target::Lan).err().expect(url);
            assert!(e.contains("not a private address") || e.contains("literal IPv4") || e.contains("wss://"), "{url}: {e}");
        }
        assert!(lan_ok(Ipv4Addr::new(192, 168, 1, 221)) && lan_ok(Ipv4Addr::new(10, 0, 0, 1)) && lan_ok(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(!lan_ok(Ipv4Addr::new(8, 8, 8, 8)) && !lan_ok(Ipv4Addr::new(100, 66, 101, 57)));
    }
}
