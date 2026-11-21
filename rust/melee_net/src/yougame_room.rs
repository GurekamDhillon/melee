//! YouGame rooms client: the queue, the match room, its roster and its match lifecycle. A port of the logic of
//! Hero's `port/runtime/hle/yougame_room.h` (room state, begin / finish, roster reads, the own-socket rejoin) and of
//! `find_match`'s queue and room join (`yougame_link.cpp`), on our own WebSocket (`slippi_mm::Ws`).
//!
//! The wire, as his client speaks it on its own socket (no app broker):
//! - queue: `GET <url>/match?ticket=T` -> `searching{waiting}` ... `matched{room, ticket?}`; we may send `cancel`
//!   (a blend that lost) and `leave`. A close the server means (4000 replaced, 4003/4403/1008) ends the search; any
//!   other close reopens the queue for up to 15 s (renewing the ticket first when it is old).
//! - room: `GET <url>/room/<code>?ticket=T` -> `welcome{me, participants, host, revision, matchId?}`, then `join`,
//!   `leave`, `lobby`, `back`, `away` (roster changes), `msg{from, d}` (a peer's message), `start{matchId}`,
//!   `result{matchId}`, `command-error{action, error}`. We send `msg{to, d}`, `begin{revision}` (the host, once a game
//!   starts), `finish{matchId, winner | void}`, `ping{at}` every 5 s, `leave`.
//! - each seat sends every other seat its candidate `msg{to, d:{kind:"mu-cand", v:2, compat, code, match, lan, ext,
//!   token}}` once a second until it has theirs; the netplay link then goes to the peer's address.
//! - our socket dropping without a close from either side is rejoined: a fresh ticket first, then the same room,
//!   within 6 s, at most 3 times per room.
//!
//! Every `url` is checked by `Ws::connect_to`'s `Target`: tests and agent runs use `Target::Loopback` (a local fake).

use crate::slippi_mm::{quote, Json, Target, Ws};
use std::net::SocketAddrV4;
use std::time::{Duration, Instant};

/// One seat of the server's roster.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Participant {
    pub id: String,
    pub connection: String,
    pub slot: i32,
    pub local_index: i32,
    /// 0 / 1 in a four-seat Teams room, else -1.
    pub team: i32,
}

fn s(j: &Json, k: &str) -> String {
    j.get(k).and_then(|v| v.str()).unwrap_or("").to_string()
}
fn int(j: &Json, k: &str, d: i64) -> i64 {
    j.get(k).and_then(|v| v.num()).map(|n| n as i64).unwrap_or(d)
}

pub fn parse_participants(m: &Json) -> Option<Vec<Participant>> {
    let Some(Json::Arr(a)) = m.get("participants") else { return None };
    Some(
        a.iter()
            .filter(|p| matches!(p, Json::Obj(_)))
            .map(|p| Participant {
                id: s(p, "id"),
                connection: s(p, "connectionId"),
                slot: int(p, "slot", -1) as i32,
                local_index: int(p, "localIndex", 0) as i32,
                team: match int(p, "team", -1) {
                    t @ (0 | 1) => t as i32,
                    _ => -1,
                },
            })
            .collect(),
    )
}

/// The seat (slot) of a connection's first local player, -1 when it has none. Slots, never roster order, are seats.
pub fn connection_seat(ps: &[Participant], connection: &str) -> i32 {
    ps.iter().find(|p| p.connection == connection && p.local_index == 0).map_or(-1, |p| p.slot)
}

/// The participant id seated at each port (0-based), "" where nobody sits.
pub fn port_participants(ps: &[Participant], size: usize) -> Vec<String> {
    let mut ids = vec![String::new(); size];
    for p in ps {
        if p.slot >= 0 && (p.slot as usize) < size {
            ids[p.slot as usize] = p.id.clone();
        }
    }
    ids
}

/// What the room says about the match (for the caller's log and flow).
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    None,
    Started(String),
    Result(String),
    Refused { action: String, error: String },
    Message { from: String, d: Json },
    Ended(String),
}

/// The room's state as this seat sees it (Hero's `RoomState`, own-socket singles).
#[derive(Clone, Debug)]
pub struct Room {
    pub me: String,
    pub host: String,
    pub revision: i64,
    pub participants: Vec<Participant>,
    pub size: usize,
    pub match_id: String,
    pub port_ids: Vec<String>,
    pub port_teams: [i32; 4],
    pending_begin: bool,
    begin_at: u64,
    in_game: bool,
    /// -2 none, -1 no single winner (void), else the 0-based winning port.
    pending_winner: i32,
}

impl Room {
    pub fn new(size: usize) -> Room {
        Room {
            me: String::new(),
            host: String::new(),
            revision: 0,
            participants: Vec::new(),
            size,
            match_id: String::new(),
            port_ids: vec![String::new(); size],
            port_teams: [-1; 4],
            pending_begin: false,
            begin_at: 0,
            in_game: false,
            pending_winner: -2,
        }
    }

    pub fn is_host(&self) -> bool {
        !self.me.is_empty() && self.me == self.host
    }

    /// The seated players (first local player of each connection).
    pub fn seated(&self) -> impl Iterator<Item = &Participant> {
        self.participants.iter().filter(|p| p.local_index == 0 && p.slot >= 0)
    }

    pub fn my_seat(&self) -> i32 {
        connection_seat(&self.participants, &self.me)
    }

    /// The roster parts of any frame: participants (a membership change without one clears it), host, revision,
    /// Teams' seat teams (four seats only).
    fn read_roster(&mut self, t: &str, m: &Json) {
        match parse_participants(m) {
            Some(ps) => self.participants = ps,
            None if matches!(t, "welcome" | "lobby" | "join" | "back" | "leave" | "away") => self.participants.clear(),
            None => {}
        }
        self.port_teams = [-1; 4];
        if self.size == 4 {
            for p in &self.participants {
                if (0..4).contains(&p.slot) {
                    self.port_teams[p.slot as usize] = p.team;
                }
            }
        }
        if let Some(h) = m.get("host").and_then(|v| v.str()) {
            self.host = h.to_string();
        }
        if let Some(r) = m.get("revision").and_then(|v| v.num()) {
            self.revision = r as i64;
        }
        self.port_ids = port_participants(&self.participants, self.size);
    }

    /// Take one frame from the room.
    pub fn receive(&mut self, m: &Json, now: u64) -> Event {
        let t = s(m, "t");
        if t == "welcome" {
            self.me = s(m, "me");
        }
        self.read_roster(&t, m);
        match t.as_str() {
            "start" => {
                self.match_id = s(m, "matchId");
                self.pending_begin = false;
                Event::Started(self.match_id.clone())
            }
            "result" | "game-result" => {
                let id = s(m, "matchId");
                if t == "result" && id == self.match_id {
                    self.match_id.clear();
                }
                Event::Result(id)
            }
            "command-error" => {
                let action = s(m, "action");
                // a refused begin while a game runs (or its finish waits) is tried again in 2 s
                if action == "begin" && (self.in_game || self.pending_winner != -2) {
                    self.pending_begin = true;
                    self.begin_at = now + 2000;
                }
                Event::Refused { action, error: s(m, "error").chars().take(160).collect() }
            }
            "msg" => Event::Message { from: s(m, "from"), d: m.get("d").cloned().unwrap_or(Json::Null) },
            "ended" | "end" | "kick" | "kicked" | "error" => Event::Ended(t),
            _ => Event::None,
        }
    }

    /// A game started on this seat: the host asks the room to begin a match.
    pub fn game_started(&mut self) {
        self.in_game = true;
        self.pending_begin = true;
        self.begin_at = 0;
    }

    /// The game ended: `winner` is the 0-based winning port, or -1 for no single winner.
    pub fn game_ended(&mut self, winner: i32) {
        self.in_game = false;
        self.pending_winner = winner.max(-1);
    }

    /// What the room is owed now: `begin` (the host, no match running, its retry time passed) and `finish`.
    pub fn due(&mut self, now: u64) -> Vec<String> {
        let mut out = Vec::new();
        if self.pending_begin && self.is_host() && self.match_id.is_empty() && now >= self.begin_at {
            out.push(format!(r#"{{"t":"begin","revision":{}}}"#, self.revision));
            self.pending_begin = false;
        }
        if self.pending_winner != -2 && !self.match_id.is_empty() {
            let w = self.pending_winner;
            let tail = match self.port_ids.get(w.max(0) as usize) {
                Some(id) if w >= 0 && !id.is_empty() => format!(r#""winner":{}"#, quote(id)),
                _ => r#""void":true"#.into(),
            };
            out.push(format!(r#"{{"t":"finish","matchId":{},{tail}}}"#, quote(&self.match_id)));
            self.pending_winner = -2;
            self.match_id.clear();
        }
        out
    }
}

// ---- sockets ---------------------------------------------------------------------------------------------------

/// One read from a socket.
#[derive(Debug, PartialEq, Eq)]
pub enum Next {
    Msg(String),
    Idle,
    /// The connection ended; the close code when the server sent one (1006: no close frame).
    Closed(u16),
}

pub trait Socket: Send {
    fn send(&mut self, text: &str) -> bool;
    fn next(&mut self, wait: Duration) -> Next;
}

/// Opens queue (`room` None) and room sockets with a ticket.
pub trait Opener: Send {
    fn open(&mut self, room: Option<&str>, ticket: &str) -> Result<Box<dyn Socket>, String>;
}

fn url_encode(v: &str) -> String {
    v.bytes()
        .map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect()
}

/// The real sockets: `url` is the rooms base the ticket named (`ws://127.0.0.1:P` for a local fake).
pub struct WsOpener {
    pub url: String,
    pub target: Target,
}

struct WsSocket {
    ws: Ws,
    closed: Option<u16>,
}

impl Socket for WsSocket {
    fn send(&mut self, text: &str) -> bool {
        self.closed.is_none() && self.ws.send_text(text).is_ok()
    }
    fn next(&mut self, wait: Duration) -> Next {
        if let Some(c) = self.closed {
            return Next::Closed(c);
        }
        self.ws.set_read_timeout(wait);
        match self.ws.recv_text() {
            Ok(t) => Next::Msg(t),
            Err(e) if e.contains("timed out") || e.contains("10060") || e.contains("would block") || e.contains("10035") => Next::Idle,
            Err(e) => {
                let code = e.split("(code ").nth(1).and_then(|r| r.trim_end_matches(')').parse().ok()).unwrap_or(1006);
                self.closed = Some(code);
                Next::Closed(code)
            }
        }
    }
}

impl Opener for WsOpener {
    fn open(&mut self, room: Option<&str>, ticket: &str) -> Result<Box<dyn Socket>, String> {
        let base = self.url.trim_end_matches('/');
        let path = match room {
            None => format!("{base}/match?ticket={}", url_encode(ticket)),
            Some(r) => format!("{base}/room/{}?ticket={}", url_encode(r), url_encode(ticket)),
        };
        let target = match self.target {
            Target::Loopback => Target::Loopback,
            Target::Lan => Target::Lan,
            Target::Real => Target::Real,
        };
        let ws = Ws::connect_to(&path, Duration::from_secs(5), target)?;
        Ok(Box::new(WsSocket { ws, closed: None }))
    }
}

// ---- the queue -------------------------------------------------------------------------------------------------

/// How a queue wait ended.
#[derive(Debug, PartialEq, Eq)]
pub enum QueueEnd {
    /// Matched into `room`; `ticket` is the room ticket the queue sent with it (else the queue's own).
    Matched { room: String, ticket: Option<String> },
    /// Cancelled; a match that arrived meanwhile was declined (`declined`: its room was left).
    Cancelled { declined: Option<String> },
    Failed(String),
}

/// Hooks of one search: cancel, the blend's claim, the waiting count, a fresh ticket, the clock.
pub struct Hooks<'a> {
    pub cancelled: &'a dyn Fn() -> bool,
    /// The blend asks before taking a room (false: Slippi's side won; the room is declined).
    pub claim: &'a mut dyn FnMut() -> bool,
    pub searching: &'a mut dyn FnMut(i32),
    /// A fresh ticket (`mpTicket` again), None when the app gave none.
    pub renew: &'a mut dyn FnMut() -> Option<String>,
    pub now: &'a dyn Fn() -> u64,
    pub log: &'a mut dyn FnMut(&str),
}

pub fn now_ms() -> u64 {
    use std::sync::OnceLock;
    static T0: OnceLock<Instant> = OnceLock::new();
    T0.get_or_init(Instant::now).elapsed().as_millis() as u64 + 1
}

/// Leave a room we were matched into but do not take (a cancelled or lost blend): open it, say leave, close.
pub fn decline(opener: &mut dyn Opener, room: &str, ticket: &str) {
    if let Ok(mut s) = opener.open(Some(room), ticket) {
        s.send(r#"{"t":"leave"}"#);
    }
}

/// Wait in YouGame's queue until the matchmaker names a room (Hero `find_match`, "Unranked: wait in the queue").
pub fn queue(opener: &mut dyn Opener, ticket: &mut String, h: &mut Hooks) -> QueueEnd {
    let mut ticket_at = (h.now)();
    let mut q = match opener.open(None, ticket) {
        Ok(q) => q,
        Err(e) => return QueueEnd::Failed(format!("Could not reach YouGame matchmaking ({e})")),
    };
    loop {
        if (h.cancelled)() {
            q.send(r#"{"t":"cancel"}"#);
            // a match that crossed the cancel is declined, so the other seat is not left waiting in it
            let until = (h.now)() + 300;
            while (h.now)() < until {
                match q.next(Duration::from_millis(50)) {
                    Next::Msg(t) => {
                        if let Some(m) = Json::parse(&t).filter(|m| s(m, "t") == "matched" && !s(m, "room").is_empty()) {
                            let room = s(&m, "room");
                            let pass = Some(s(&m, "ticket")).filter(|t| !t.is_empty()).unwrap_or_else(|| ticket.clone());
                            decline(opener, &room, &pass);
                            return QueueEnd::Cancelled { declined: Some(room) };
                        }
                    }
                    Next::Closed(_) => break,
                    Next::Idle => {}
                }
            }
            return QueueEnd::Cancelled { declined: None };
        }
        match q.next(Duration::from_millis(250)) {
            Next::Idle => continue,
            Next::Closed(code) => {
                (h.log)(&format!("yougame: matchmaking connection closed (code {code})"));
                match code {
                    4000 => return QueueEnd::Failed("This account started a search somewhere else, so this one stopped".into()),
                    4003 | 4403 | 1008 => return QueueEnd::Failed("Lost the connection to YouGame matchmaking".into()),
                    _ => {}
                }
                // any other close reopens the queue (the player pressed search once): 15 s, a fresh ticket when old
                let until = (h.now)() + 15_000;
                if (h.now)() - ticket_at > 240_000 {
                    if let Some(t) = (h.renew)() {
                        *ticket = t;
                        ticket_at = (h.now)();
                    }
                }
                let mut tries = 0;
                let reopened = loop {
                    if (h.cancelled)() || (h.now)() >= until {
                        break None;
                    }
                    if let Ok(nq) = opener.open(None, ticket) {
                        break Some(nq);
                    }
                    tries += 1;
                    if tries == 2 {
                        if let Some(t) = (h.renew)() {
                            *ticket = t;
                            ticket_at = (h.now)();
                        }
                    }
                    std::thread::sleep(Duration::from_millis(250));
                };
                match reopened {
                    Some(nq) => {
                        (h.log)("yougame: back in the queue");
                        q = nq;
                    }
                    None if (h.cancelled)() => return QueueEnd::Cancelled { declined: None },
                    None => return QueueEnd::Failed("Lost the connection to YouGame matchmaking".into()),
                }
            }
            Next::Msg(text) => {
                let Some(m) = Json::parse(&text) else { continue };
                match s(&m, "t").as_str() {
                    "searching" => {
                        if let Some(w) = m.get("waiting").and_then(|v| v.num()) {
                            (h.searching)(w.clamp(0.0, 1_000_000.0) as i32);
                        }
                    }
                    "matched" if !s(&m, "room").is_empty() => {
                        let room = s(&m, "room");
                        let fresh = Some(s(&m, "ticket")).filter(|t| !t.is_empty());
                        if (h.cancelled)() || !(h.claim)() {
                            decline(opener, &room, fresh.as_deref().unwrap_or(ticket));
                            return QueueEnd::Cancelled { declined: Some(room) };
                        }
                        return QueueEnd::Matched { room, ticket: fresh };
                    }
                    "error" => return QueueEnd::Failed(Some(s(&m, "error")).filter(|e| !e.is_empty()).unwrap_or_else(|| "Matchmaking failed".into())),
                    _ => {}
                }
            }
        }
    }
}

// ---- the match room --------------------------------------------------------------------------------------------

/// This seat's candidate: its connect code, the compatibility key, and the addresses its netplay port is reached at.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub compat: String,
    pub code: String,
    pub lan: String,
    pub ext: String,
    pub token: String,
}

impl Candidate {
    fn json(&self, match_id: &str) -> String {
        format!(
            r#"{{"kind":"mu-cand","v":2,"compat":{},"code":{},"match":{},"lan":{},"ext":{},"token":{}}}"#,
            quote(&self.compat),
            quote(&self.code),
            quote(match_id),
            quote(&self.lan),
            quote(&self.ext),
            quote(&self.token)
        )
    }
}

/// The opponent found in the room: who and where.
#[derive(Debug)]
pub struct Joined {
    pub room: Room,
    pub socket: Box<dyn Socket>,
    pub ticket: String,
    pub code: String,
    /// This seat's 0-based port (the room's slot) and the opponent's.
    pub seat: u8,
    pub opp_seat: u8,
    pub peer: SocketAddrV4,
    pub opp_code: String,
    /// The opponent's published addresses and punch token (its candidate's `lan`, `ext`, `token`).
    pub opp_lan: Option<SocketAddrV4>,
    pub opp_ext: Option<SocketAddrV4>,
    pub opp_token: String,
    pub rejoins: u32,
}

impl std::fmt::Debug for dyn Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Socket")
    }
}

/// Our socket dropped (no close from the room): a fresh ticket first, then the same room, within 6 s.
pub fn rejoin(opener: &mut dyn Opener, code: &str, ticket: &mut String, h: &mut Hooks) -> Option<Box<dyn Socket>> {
    let start = (h.now)();
    let until = start + 6000;
    (h.log)("yougame: room socket dropped; rejoining the room");
    let mut renewed = false;
    while !(h.cancelled)() && (h.now)() < until {
        if !renewed {
            if let Some(t) = (h.renew)() {
                *ticket = t;
                renewed = true;
            }
        }
        if let Ok(s) = opener.open(Some(code), ticket) {
            (h.log)(&format!("yougame: back in the room after {} ms", (h.now)() - start));
            return Some(s);
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    (h.log)("yougame: could not rejoin the room");
    None
}

fn parse_addr(a: &str) -> Option<SocketAddrV4> {
    a.parse().ok().filter(|x: &SocketAddrV4| x.port() != 0 && !x.ip().is_unspecified())
}

/// Join the match room, read the roster, and trade candidates with the opponent (singles: two seats).
/// `timeout_full_ms`: how long a full room may wait for the opponent's candidate (Hero: 15 s).
pub fn join(opener: &mut dyn Opener, code: &str, mut ticket: String, me: &Candidate, h: &mut Hooks, timeout_full_ms: u64) -> Result<Joined, String> {
    let mut sock = opener.open(Some(code), &ticket).map_err(|e| format!("Could not join the YouGame room ({e})"))?;
    let mut room = Room::new(2);
    let mut theirs: Option<(String, Json)> = None;
    let mut sent_at: u64 = 0;
    let mut full_since: Option<u64> = None;
    let mut last_ping = (h.now)();
    let mut rejoins = 0u32;
    let opened = (h.now)();
    loop {
        if (h.cancelled)() {
            sock.send(r#"{"t":"leave"}"#);
            return Err("Cancelled".into());
        }
        match sock.next(Duration::from_millis(100)) {
            Next::Idle => {}
            Next::Closed(code_) => {
                room.participants.clear();
                if rejoins >= 3 || matches!(code_, 4000 | 4003 | 4004 | 4403 | 1000 | 1008) {
                    return Err("The YouGame room closed".into());
                }
                rejoins += 1;
                sock = rejoin(opener, code, &mut ticket, h).ok_or("Lost the connection to the YouGame room")?;
                sent_at = 0;
                continue;
            }
            Next::Msg(text) => {
                let Some(m) = Json::parse(&text) else { continue };
                match room.receive(&m, (h.now)()) {
                    Event::Message { from, d } => {
                        if s(&d, "kind") == "mu-cand" && int(&d, "v", 0) == 2 && connection_seat(&room.participants, &from) >= 0 && from != room.me {
                            if s(&d, "compat") != me.compat {
                                sock.send(r#"{"t":"leave"}"#);
                                return Err("The opponent's game is a different version".into());
                            }
                            theirs = Some((from, d));
                        }
                    }
                    Event::Ended(t) => return Err(format!("The YouGame room ended ({t})")),
                    _ => {}
                }
            }
        }
        let now = (h.now)();
        let seated: Vec<Participant> = room.seated().cloned().collect();
        let full = !room.me.is_empty() && seated.len() >= 2 && connection_seat(&room.participants, &room.me) >= 0;
        if !full {
            full_since = None;
            if now - opened > 30_000 {
                sock.send(r#"{"t":"leave"}"#);
                return Err("Your opponent did not join the room".into());
            }
        } else {
            let since = *full_since.get_or_insert(now);
            let my_seat = connection_seat(&room.participants, &room.me);
            // the seat-0 player names the match id both sides use for the netplay link
            let match_id = if my_seat == 0 { format!("{code}-1") } else { String::new() };
            if now - sent_at >= 1000 {
                for p in seated.iter().filter(|p| p.connection != room.me) {
                    sock.send(&format!(r#"{{"t":"msg","to":{},"d":{}}}"#, quote(&p.connection), me.json(&match_id)));
                }
                sent_at = now;
            }
            if let Some((from, d)) = &theirs {
                let opp_seat = connection_seat(&room.participants, from);
                let peer = parse_addr(&s(d, "lan")).or_else(|| parse_addr(&s(d, "ext")));
                if let Some(peer) = peer {
                    // one more send so the other side surely has ours, then the link
                    sock.send(&format!(r#"{{"t":"msg","to":{},"d":{}}}"#, quote(from), me.json(&match_id)));
                    return Ok(Joined {
                        opp_lan: parse_addr(&s(d, "lan")),
                        opp_ext: parse_addr(&s(d, "ext")),
                        opp_token: s(d, "token"),
                        opp_code: s(d, "code"),
                        room,
                        socket: sock,
                        ticket,
                        code: code.to_string(),
                        seat: my_seat as u8,
                        opp_seat: opp_seat as u8,
                        peer,
                        rejoins,
                    });
                }
                sock.send(r#"{"t":"leave"}"#);
                return Err("The opponent sent no address".into());
            }
            if now - since > timeout_full_ms {
                sock.send(r#"{"t":"leave"}"#);
                return Err("Your opponent did not answer".into());
            }
        }
        if now - last_ping > 5000 {
            sock.send(&format!(r#"{{"t":"ping","at":{now}}}"#));
            last_ping = now;
        }
    }
}

/// Keep the room through the match: deliver `begin` / `finish`, answer pings, rejoin a dropped socket.
/// Call `tick` every few ms from the session's own thread (it never blocks longer than `wait`).
pub fn pump(j: &mut Joined, opener: &mut dyn Opener, h: &mut Hooks, wait: Duration) -> Vec<Event> {
    let mut events = Vec::new();
    match j.socket.next(wait) {
        Next::Idle => {}
        Next::Msg(text) => {
            if let Some(m) = Json::parse(&text) {
                let e = j.room.receive(&m, (h.now)());
                if e != Event::None {
                    events.push(e);
                }
            }
        }
        Next::Closed(code) => {
            if j.rejoins < 3 && !matches!(code, 4000 | 4003 | 4004 | 4403 | 1000 | 1008) {
                j.rejoins += 1;
                let mut t = j.ticket.clone();
                if let Some(s) = rejoin(opener, &j.code.clone(), &mut t, h) {
                    j.socket = s;
                    j.ticket = t;
                    return events;
                }
            }
            events.push(Event::Ended("closed".into()));
            return events;
        }
    }
    for out in j.room.due((h.now)()) {
        j.socket.send(&out);
    }
    events
}
