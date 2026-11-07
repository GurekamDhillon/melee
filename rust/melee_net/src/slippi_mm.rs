//! The client half of Slippi's matchmaking protocol (`Externals/SlippiRustExtensions/matchmaking`: `protocol.rs`,
//! `client.rs`; `SlippiMatchmaking.cpp`'s `match` handler), enough to be paired through a matchmaking server with
//! a Slippi Dolphin. Plain WebSocket (RFC 6455 client: masked frames, text only, ping answered) over TCP.
//!
//! **Loopback only.** [`join`] refuses any address that is not 127.0.0.1: it exists for the local mock server
//! (`tools/rust-port/slippi/mock_mm.py`), never a Slippi service. No STUN, no account: the user record it sends is
//! made up by the caller.
//!
//! Flow: `join{user, search{mode, connectCode}, appVersion, port, ipAddressLan, candidates, natType}` ->
//! `joined{ticketId, token}`, `status`... -> `match{matchId, isHost, players[{port, isLocalPlayer, ipAddress,
//! ipAddressLan, ...}], stages, items}` or `error{error}`.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
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

/// What we tell the server about ourselves.
#[derive(Clone, Debug)]
pub struct JoinRequest {
    pub uid: String,
    pub display_name: String,
    pub connect_code: String,
    /// 2 = direct (0 ranked, 1 unranked, 3 teams, 4 party)
    pub mode: u8,
    /// the opponent's code for direct mode
    pub search_code: String,
    pub app_version: String,
    /// our netplay (ENet) UDP port; the netplay client binds it after the match is found
    pub port: u16,
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
    /// The remote addresses as SlippiMatchmaking picks them: the LAN address when both share the external IP.
    pub fn remote_addresses(&self) -> Vec<(u8, String)> {
        let Some(me) = self.players.iter().find(|p| p.is_local) else { return Vec::new() };
        let my_ext = me.ip_address.split(':').next().unwrap_or("").to_string();
        self.players
            .iter()
            .filter(|p| !p.is_local)
            .map(|p| {
                let ext = p.ip_address.split(':').next().unwrap_or("");
                let a = if ext != my_ext || p.ip_address_lan.is_empty() { &p.ip_address } else { &p.ip_address_lan };
                (p.port.saturating_sub(1), a.clone())
            })
            .collect()
    }

    pub fn from_json(j: &Json) -> Option<MatchResult> {
        if j.get("type")?.str()? != "match" {
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
    let code_bytes: Vec<String> = r.search_code.bytes().map(|b| b.to_string()).collect();
    let lan = format!("127.0.0.1:{}", r.port);
    format!(
        "{{\"type\":\"join\",\"user\":{{\"uid\":{},\"playKey\":\"\",\"connectCode\":{},\"displayName\":{}}},\"search\":{{\"mode\":{},\"connectCode\":[{}]}},\"appVersion\":{},\"port\":{},\"ipAddressLan\":{},\"candidates\":[{{\"kind\":\"host\",\"address\":{}}}],\"natType\":\"unknown\"}}",
        quote(&r.uid),
        quote(&r.connect_code),
        quote(&r.display_name),
        r.mode,
        code_bytes.join(","),
        quote(&r.app_version),
        r.port,
        quote(&lan),
        quote(&lan)
    )
}

/// A WebSocket client connection (text frames).
pub struct Ws {
    s: TcpStream,
    buf: Vec<u8>,
    mask: u32,
}

impl Ws {
    /// `ws://127.0.0.1:PORT/PATH` only.
    pub fn connect(url: &str, timeout: Duration) -> Result<Ws, String> {
        let rest = url.strip_prefix("ws://").ok_or("only ws:// URLs")?;
        let (hostport, path) = rest.split_once('/').map(|(a, b)| (a, format!("/{b}"))).unwrap_or((rest, "/".into()));
        let (host, port) = hostport.split_once(':').ok_or("URL needs host:port")?;
        let ip: Ipv4Addr = host.parse().map_err(|_| "the matchmaking host must be a literal IPv4 address".to_string())?;
        if !ip.is_loopback() {
            return Err(format!("{ip} is not loopback: only a local mock matchmaking server is allowed"));
        }
        let port: u16 = port.parse().map_err(|_| "bad port")?;
        let s = TcpStream::connect_timeout(&SocketAddrV4::new(ip, port).into(), timeout).map_err(|e| e.to_string())?;
        s.set_nodelay(true).ok();
        let key = "R0QtbWVsZWUtc2xpcHBpMw=="; // any 16 bytes, base64
        let req = format!("GET {path} HTTP/1.1\r\nHost: {hostport}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n");
        let mut ws = Ws { s, buf: Vec::new(), mask: 0x5EED_1234 };
        ws.s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        ws.s.set_read_timeout(Some(timeout)).ok();
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
                            8 => return Err("server closed the connection".into()),
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
pub fn join(url: &str, req: &JoinRequest, timeout: Duration, mut on_msg: impl FnMut(&str)) -> Result<MatchResult, String> {
    let t0 = Instant::now();
    let mut ws = Ws::connect(url, Duration::from_secs(5))?;
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
        let Some(j) = Json::parse(&t) else { continue };
        match j.get("type").and_then(|v| v.str()) {
            Some("match") => return MatchResult::from_json(&j).ok_or_else(|| "malformed match".to_string()),
            Some("error") => return Err(format!("server error: {}", j.get("error").and_then(|v| v.str()).unwrap_or("?"))),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_match() {
        let t = r#"{"type": "match", "matchId": "m1", "isHost": false, "stages": [2, 32],
            "players": [{"uid": "a", "port": 1, "isLocalPlayer": false, "ipAddress": "127.0.0.1:5000", "ipAddressLan": "127.0.0.1:5000", "displayName": "Aé"},
                        {"uid": "b", "port": 2, "isLocalPlayer": true, "ipAddress": "127.0.0.1:6000", "ipAddressLan": "127.0.0.1:6000"}]}"#;
        let m = MatchResult::from_json(&Json::parse(t).unwrap()).unwrap();
        assert_eq!(m.local_index(), Some(1));
        assert_eq!(m.stages, vec![2, 32]);
        assert_eq!(m.remote_addresses(), vec![(0, "127.0.0.1:5000".to_string())]);
        assert_eq!(m.players[0].display_name, "Aé");
        let j = join_message(&JoinRequest { uid: "u".into(), display_name: "D".into(), connect_code: "AB#1".into(), mode: 2, search_code: "CD#2".into(), app_version: "4.0.0".into(), port: 7 });
        let p = Json::parse(&j).unwrap();
        assert_eq!(p.get("search").unwrap().get("connectCode").unwrap().arr().len(), 4);
        assert_eq!(p.get("port").unwrap().num(), Some(7.0));
    }

    #[test]
    fn refuses_non_loopback() {
        assert!(Ws::connect("ws://10.0.0.1:1/v1/queue", Duration::from_millis(10)).is_err());
        assert!(Ws::connect("ws://matchmaking.example:1/v1/queue", Duration::from_millis(10)).is_err());
    }
}
