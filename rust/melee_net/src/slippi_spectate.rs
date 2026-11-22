//! The Slippi spectator feed ("broadcast"): the protocol Slippi Dolphin serves on 127.0.0.1 (port 51441 by default)
//! over ENet, which the Slippi Launcher's Broadcast tab and stream overlays (slippi-js `DolphinConnection`) read.
//! Hero's melee-unlocked serves the same feed from his game (`slippi_spectate.cpp`); this is our own implementation of
//! the behaviour, written from the messages on the wire. No code is shared.
//!
//! Messages are JSON text, one per reliable ENet packet on channel 0, keys in sorted order (as Dolphin's JSON writer
//! emits them):
//! - client -> server `{"cursor":N,"type":"connect_request"}`: N is where the client wants to resume (0: from the
//!   start of the current game);
//! - server -> client `{"cursor":N,"nick":..,"type":"connect_reply","version":..}`;
//! - `{"cursor":c,"next_cursor":c+1,"type":"start_game"}` at a game's start;
//! - `{"cursor":c,"next_cursor":c+1,"payload":"<base64>","type":"game_event"}`: Slippi recording events, grouped so
//!   that each group ends with Game Start (0x36), Frame Bookend (0x3C), Game End (0x39) or a Message Splitter (0x10);
//! - `{"cursor":c,"dolphin_closed":b,"next_cursor":c+1,"type":"end_game"}`;
//! - `{"payload":"<base64>","type":"menu_event"}` outside a game (the latest one only).
//!
//! [`Feed`] is the server's state without sockets (the caller owns the ENet host and pumps it); [`Watcher`] is the
//! client's, which turns the messages back into the `.slp` event stream. Both are pure and unit-tested; the sockets
//! are melee/src/slippi_spectate.rs.

use crate::yougame_relay::{b64, unb64};

pub const DEFAULT_PORT: u16 = 51441;
/// The most clients one feed serves (Dolphin's limit).
pub const MAX_CLIENTS: usize = 4;
pub const CHANNELS: usize = 2;

/// What the game hands the feed.
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    StartGame,
    /// One or more whole recording events (command byte and payload each).
    Events(Vec<u8>),
    EndGame { closed: bool },
    /// A menu frame outside a game.
    Menu(Vec<u8>),
}

#[derive(Clone, Debug, Default)]
struct Client {
    /// Index into `buffer` of the next message to send.
    cursor: usize,
    menu_cursor: u64,
    shook: bool,
}

/// The server's state: the current game's messages (a late client is caught up from its start), per-client cursors.
pub struct Feed {
    nick: String,
    version: String,
    in_game: bool,
    buffer: Vec<String>,
    /// Cursor numbers count every message since the feed began: `offset` is the number of the first in `buffer`.
    offset: u64,
    concat: Vec<u8>,
    menu: Option<String>,
    menu_cursor: u64,
    clients: Vec<Option<Client>>,
}

fn esc(s: &str) -> String {
    s.chars().flat_map(|c| match c {
        '"' => vec!['\\', '"'],
        '\\' => vec!['\\', '\\'],
        c if (c as u32) < 0x20 => format!("\\u{:04x}", c as u32).chars().collect(),
        c => vec![c],
    })
    .collect()
}

impl Feed {
    pub fn new(nick: &str, version: &str) -> Feed {
        Feed { nick: nick.into(), version: version.into(), in_game: false, buffer: Vec::new(), offset: 0, concat: Vec::new(), menu: None, menu_cursor: 0, clients: Vec::new() }
    }

    pub fn in_game(&self) -> bool {
        self.in_game
    }

    fn next_cursor(&self) -> u64 {
        self.offset + self.buffer.len() as u64
    }

    /// Take what the game produced.
    pub fn push(&mut self, input: Input) {
        match input {
            Input::StartGame => {
                // a new game: its messages replace the last game's (cursor numbers keep counting)
                self.offset += self.buffer.len() as u64;
                self.buffer.clear();
                for c in self.clients.iter_mut().flatten() {
                    c.cursor = 0;
                }
                self.concat.clear();
                self.in_game = true;
                let c = self.next_cursor();
                self.buffer.push(format!("{{\"cursor\":{c},\"next_cursor\":{},\"type\":\"start_game\"}}", c + 1));
            }
            Input::Events(bytes) => {
                if !self.in_game {
                    return;
                }
                // one call = one group (slp_export hands Event Payloads + Game Start, each Splitter, each frame
                // through its Bookend, Game End), as Dolphin closes a group at 0x36 / 0x10 / 0x3C / 0x39
                self.concat.extend_from_slice(&bytes);
                let c = self.next_cursor();
                let p = b64(&std::mem::take(&mut self.concat));
                self.buffer.push(format!("{{\"cursor\":{c},\"next_cursor\":{},\"payload\":\"{p}\",\"type\":\"game_event\"}}", c + 1));
            }
            Input::EndGame { closed } => {
                if !self.concat.is_empty() {
                    let c = self.next_cursor();
                    let p = b64(&std::mem::take(&mut self.concat));
                    self.buffer.push(format!("{{\"cursor\":{c},\"next_cursor\":{},\"payload\":\"{p}\",\"type\":\"game_event\"}}", c + 1));
                }
                let c = self.next_cursor();
                self.buffer.push(format!("{{\"cursor\":{c},\"dolphin_closed\":{closed},\"next_cursor\":{},\"type\":\"end_game\"}}", c + 1));
                self.in_game = false;
                self.menu = None;
                self.menu_cursor = 0;
            }
            Input::Menu(bytes) => {
                if self.in_game {
                    return;
                }
                self.menu_cursor += 1;
                self.menu = Some(format!("{{\"payload\":\"{}\",\"type\":\"menu_event\"}}", b64(&bytes)));
            }
        }
    }

    /// A client connected (an ENet peer index).
    pub fn connect(&mut self, peer: usize) {
        if self.clients.len() <= peer {
            self.clients.resize(peer + 1, None);
        }
        self.clients[peer] = Some(Client::default());
    }

    pub fn disconnect(&mut self, peer: usize) {
        if let Some(c) = self.clients.get_mut(peer) {
            *c = None;
        }
    }

    pub fn clients(&self) -> usize {
        self.clients.iter().flatten().count()
    }

    /// A message from a client; returns the reply to send it, if any (only `connect_request` has one).
    pub fn message(&mut self, peer: usize, text: &[u8]) -> Option<String> {
        let t = std::str::from_utf8(text).ok()?;
        if json_str(t, "type").as_deref() != Some("connect_request") {
            return None;
        }
        let want = json_u64(t, "cursor")?;
        let (offset, len, in_game) = (self.offset, self.buffer.len() as u64, self.in_game);
        let c = self.clients.get_mut(peer)?.as_mut()?;
        // inside this game's messages: resume there; past them or before them: from the start of this game
        c.cursor = if want >= offset && want <= offset + len { (want - offset) as usize } else { 0 };
        let sent = offset + c.cursor as u64;
        if !in_game {
            c.cursor = len as usize; // joined at the menu: nothing to catch up
        }
        c.shook = true;
        Some(format!(
            "{{\"cursor\":{sent},\"nick\":\"{}\",\"type\":\"connect_reply\",\"version\":\"{}\"}}",
            esc(&self.nick),
            esc(&self.version)
        ))
    }

    /// The messages owed to each client that has shaken hands: (peer, message), in order. Advances the cursors.
    pub fn drain(&mut self) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        for (peer, c) in self.clients.iter_mut().enumerate() {
            let Some(c) = c.as_mut().filter(|c| c.shook) else { continue };
            if !self.in_game && c.menu_cursor != self.menu_cursor {
                if let Some(m) = &self.menu {
                    out.push((peer, m.clone()));
                }
                c.menu_cursor = self.menu_cursor;
            }
            if c.cursor > self.buffer.len() {
                c.cursor = 0;
            }
            while c.cursor < self.buffer.len() {
                out.push((peer, self.buffer[c.cursor].clone()));
                c.cursor += 1;
            }
        }
        out
    }
}

/// A plain `"key":"value"` from a flat JSON object (no escapes inside the values these messages carry).
pub fn json_str(t: &str, key: &str) -> Option<String> {
    let k = format!("\"{key}\"");
    let at = t.find(&k)? + k.len();
    let rest = t[at..].trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// A non-negative integer `"key":N` from a flat JSON object.
pub fn json_u64(t: &str, key: &str) -> Option<u64> {
    let k = format!("\"{key}\"");
    let at = t.find(&k)? + k.len();
    let rest = t[at..].trim_start().strip_prefix(':')?.trim_start();
    let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse().ok()
}

/// `{"cursor":N,"type":"connect_request"}`.
pub fn connect_request(cursor: u64) -> String {
    format!("{{\"cursor\":{cursor},\"type\":\"connect_request\"}}")
}

/// What a [`Watcher`] saw in one message.
#[derive(Clone, Debug, PartialEq)]
pub enum Seen {
    Reply { nick: String, version: String, cursor: u64 },
    GameStart,
    /// Recording events appended to the game's stream.
    Events(usize),
    GameEnd { closed: bool },
    Menu,
    Other,
}

/// The client side: the current game's `.slp` event stream (from the 0x35 Event Payloads on) as it arrives.
#[derive(Default)]
pub struct Watcher {
    pub raw: Vec<u8>,
    pub in_game: bool,
    pub games: u32,
    /// The cursor after the last message seen (to resume a dropped link at).
    pub next_cursor: u64,
}

impl Watcher {
    pub fn message(&mut self, text: &[u8]) -> Seen {
        let Ok(t) = std::str::from_utf8(text) else { return Seen::Other };
        if let Some(n) = json_u64(t, "next_cursor") {
            self.next_cursor = n;
        }
        match json_str(t, "type").as_deref() {
            Some("connect_reply") => Seen::Reply {
                nick: json_str(t, "nick").unwrap_or_default(),
                version: json_str(t, "version").unwrap_or_default(),
                cursor: json_u64(t, "cursor").unwrap_or(0),
            },
            Some("start_game") => {
                self.raw.clear();
                self.in_game = true;
                Seen::GameStart
            }
            Some("game_event") => {
                let Some(d) = json_str(t, "payload").and_then(|p| unb64(&p, 1 << 24)) else { return Seen::Other };
                self.raw.extend_from_slice(&d);
                Seen::Events(d.len())
            }
            Some("end_game") => {
                self.in_game = false;
                self.games += 1;
                Seen::GameEnd { closed: t.contains("\"dolphin_closed\":true") }
            }
            Some("menu_event") => Seen::Menu,
            _ => Seen::Other,
        }
    }

    /// The `.slp` file of the stream so far. `done`: the game ended, so the raw length is written (and an empty
    /// metadata element); while it goes on the length is 0, as Slippi Dolphin writes a game in progress.
    pub fn slp(&self, done: bool) -> Vec<u8> {
        let mut o = b"{U\x03raw[$U#l".to_vec();
        o.extend_from_slice(&(if done { self.raw.len() as u32 } else { 0 }).to_be_bytes());
        o.extend_from_slice(&self.raw);
        if done {
            o.extend_from_slice(b"U\x08metadata{}");
        }
        o.push(b'}');
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(cmd: u8, n: usize) -> Vec<u8> {
        let mut v = vec![0u8; n];
        v[0] = cmd;
        v[n - 1] = 0xAB;
        v
    }

    #[test]
    fn a_game_round_trips_through_feed_and_watcher() {
        let mut f = Feed::new("Melee", "3.18.0");
        f.connect(0);
        assert_eq!(f.message(0, connect_request(0).as_bytes()).unwrap(), "{\"cursor\":0,\"nick\":\"Melee\",\"type\":\"connect_reply\",\"version\":\"3.18.0\"}");
        let mut start = ev(0x35, 22);
        start.extend(ev(0x36, 0x2F9));
        let frame = [ev(0x3A, 13), ev(0x37, 0x43), ev(0x38, 0x55), ev(0x3C, 9)].concat();
        f.push(Input::StartGame);
        f.push(Input::Events(start.clone()));
        f.push(Input::Events(ev(0x10, 0x205)));
        for _ in 0..3 {
            f.push(Input::Events(frame.clone()));
        }
        f.push(Input::Events(ev(0x39, 7)));
        f.push(Input::EndGame { closed: false });
        let msgs = f.drain();
        assert_eq!(msgs.len(), 1 + 1 + 1 + 3 + 1 + 1);
        assert!(msgs[0].1.starts_with("{\"cursor\":0,\"next_cursor\":1,\"type\":\"start_game\"}"));
        assert!(msgs.last().unwrap().1.contains("\"dolphin_closed\":false"));
        let mut w = Watcher::default();
        let seen: Vec<Seen> = msgs.iter().map(|m| w.message(m.1.as_bytes())).collect();
        assert_eq!(seen[0], Seen::GameStart);
        assert_eq!(*seen.last().unwrap(), Seen::GameEnd { closed: false });
        let want = [start, ev(0x10, 0x205), frame.clone(), frame.clone(), frame, ev(0x39, 7)].concat();
        assert_eq!(w.raw, want);
        assert_eq!(w.next_cursor, 8);
        // nothing more is owed
        assert!(f.drain().is_empty());
    }

    #[test]
    fn a_late_client_is_caught_up_from_the_games_start_and_can_resume() {
        let mut f = Feed::new("n", "v");
        f.push(Input::StartGame);
        f.push(Input::Events([ev(0x35, 22), ev(0x36, 0x2F9)].concat()));
        f.push(Input::Events(ev(0x3C, 9)));
        f.connect(2);
        assert!(f.drain().is_empty(), "no handshake yet");
        let r = f.message(2, connect_request(99).as_bytes()).unwrap();
        assert!(r.starts_with("{\"cursor\":0,"), "{r}");
        assert_eq!(f.drain().len(), 3);
        // a second client resuming at cursor 2 gets only the last message
        f.connect(3);
        f.message(3, connect_request(2).as_bytes());
        let d = f.drain();
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].0, 3);
        // the next game renumbers on: its start is cursor 3
        f.push(Input::EndGame { closed: false });
        f.push(Input::StartGame);
        let d = f.drain();
        assert!(d.iter().any(|m| m.1.starts_with("{\"cursor\":4,\"next_cursor\":5,\"type\":\"start_game\"")), "{d:?}");
    }

    #[test]
    fn menu_frames_go_only_to_clients_at_the_menu() {
        let mut f = Feed::new("n", "v");
        f.connect(0);
        f.message(0, connect_request(0).as_bytes());
        f.push(Input::Menu(vec![0x3E, 1, 2]));
        f.push(Input::Menu(vec![0x3E, 3, 4]));
        let d = f.drain();
        assert_eq!(d.len(), 1, "the latest menu frame only");
        assert!(d[0].1.contains("menu_event"));
        f.push(Input::StartGame);
        f.push(Input::Menu(vec![0x3E]));
        assert!(f.drain().iter().all(|m| !m.1.contains("menu_event")));
    }

    #[test]
    fn slp_in_progress_and_done() {
        let mut w = Watcher::default();
        w.raw = vec![0x35, 1];
        let p = w.slp(false);
        assert_eq!(&p[11..15], &[0, 0, 0, 0]);
        let d = w.slp(true);
        assert_eq!(&d[11..15], &[0, 0, 0, 2]);
        assert!(d.ends_with(b"U\x08metadata{}}"));
    }
}
