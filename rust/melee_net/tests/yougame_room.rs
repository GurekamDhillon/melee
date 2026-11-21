//! The rooms client (melee_net::yougame_room) against an in-memory fake rooms service: the queue pairs two
//! searches, the room's roster / host / revision, candidates traded, begin -> start, finish -> result, cancels
//! (with a crossing match declined), a refused begin retried, and our socket dropping and rejoining.

use melee_net::slippi_mm::Json;
use melee_net::yougame_room::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct Conn {
    inbox: VecDeque<String>,
    closed: Option<u16>,
    room: Option<String>,
    id: String,
    user: String,
}

#[derive(Default)]
struct Hub {
    conns: Vec<Conn>,
    waiting: Vec<usize>,
    rooms: Vec<(String, Vec<usize>, i64, String)>, // code, seats (conn index; slot = position), revision, match
    sent: Vec<(String, String)>,                    // (conn id, frame)
    refuse_begins: u32,
    next_room: u32,
    tickets: Vec<String>,
}

type H = Arc<Mutex<Hub>>;

fn user_of(ticket: &str) -> String {
    ticket.split('-').next().unwrap_or("").to_string()
}

impl Hub {
    fn roster(&self, room: usize) -> String {
        let (_, seats, rev, _) = &self.rooms[room];
        let ps: Vec<String> = seats
            .iter()
            .enumerate()
            .filter(|(_, &c)| self.conns[c].closed.is_none())
            .map(|(slot, &c)| format!(r#"{{"id":"p-{}","connectionId":"{}","slot":{slot},"localIndex":0}}"#, self.conns[c].user, self.conns[c].id))
            .collect();
        let host = seats.iter().find(|&&c| self.conns[c].closed.is_none()).map(|&c| self.conns[c].id.clone()).unwrap_or_default();
        format!(r#""participants":[{}],"host":"{host}","revision":{rev}"#, ps.join(","))
    }
    fn to_room(&mut self, room: usize, frame: &str) {
        for &c in &self.rooms[room].1.clone() {
            if self.conns[c].closed.is_none() {
                self.conns[c].inbox.push_back(frame.to_string());
            }
        }
    }
    fn room_of(&self, c: usize) -> Option<usize> {
        let code = self.conns[c].room.as_ref()?;
        self.rooms.iter().position(|r| &r.0 == code)
    }
    fn receive(&mut self, c: usize, text: &str) {
        self.sent.push((self.conns[c].id.clone(), text.to_string()));
        let m = Json::parse(text).unwrap();
        let t = m.get("t").and_then(|v| v.str()).unwrap_or("").to_string();
        if self.conns[c].room.is_none() {
            if t == "cancel" {
                self.waiting.retain(|&w| w != c);
            }
            return;
        }
        let r = self.room_of(c).unwrap();
        match t.as_str() {
            "msg" => {
                let to = m.get("to").and_then(|v| v.str()).unwrap_or("").to_string();
                let d = &text[text.find(r#""d":"#).unwrap() + 4..text.len() - 1];
                let from = self.conns[c].id.clone();
                if let Some(k) = self.conns.iter().position(|x| x.id == to && x.closed.is_none()) {
                    self.conns[k].inbox.push_back(format!(r#"{{"t":"msg","from":"{from}","d":{d}}}"#));
                }
            }
            "begin" => {
                if self.refuse_begins > 0 {
                    self.refuse_begins -= 1;
                    self.conns[c].inbox.push_back(r#"{"t":"command-error","action":"begin","error":"not yet"}"#.into());
                    return;
                }
                self.rooms[r].2 += 1;
                let id = format!("{}-m{}", self.rooms[r].0, self.rooms[r].2);
                self.rooms[r].3 = id.clone();
                self.to_room(r, &format!(r#"{{"t":"start","matchId":"{id}"}}"#));
            }
            "finish" => {
                let id = self.rooms[r].3.clone();
                self.to_room(r, &format!(r#"{{"t":"result","matchId":"{id}"}}"#));
            }
            "leave" => {
                self.conns[c].closed = Some(1000);
                self.rooms[r].2 += 1;
                let roster = self.roster(r);
                self.to_room(r, &format!(r#"{{"t":"leave",{roster}}}"#));
            }
            _ => {}
        }
    }
}

struct FakeSocket {
    hub: H,
    c: usize,
}
impl Socket for FakeSocket {
    fn send(&mut self, text: &str) -> bool {
        let mut h = self.hub.lock().unwrap();
        if h.conns[self.c].closed.is_some() {
            return false;
        }
        h.receive(self.c, text);
        true
    }
    fn next(&mut self, wait: Duration) -> Next {
        let until = std::time::Instant::now() + wait;
        loop {
            {
                let mut h = self.hub.lock().unwrap();
                let conn = &mut h.conns[self.c];
                if let Some(m) = conn.inbox.pop_front() {
                    return Next::Msg(m);
                }
                if let Some(code) = conn.closed {
                    return Next::Closed(code);
                }
            }
            if std::time::Instant::now() >= until {
                return Next::Idle;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

struct FakeOpener {
    hub: H,
}
impl Opener for FakeOpener {
    fn open(&mut self, room: Option<&str>, ticket: &str) -> Result<Box<dyn Socket>, String> {
        let mut h = self.hub.lock().unwrap();
        h.tickets.push(ticket.to_string());
        let user = user_of(ticket);
        let c = h.conns.len();
        h.conns.push(Conn { id: format!("c{c}"), user: user.clone(), ..Default::default() });
        match room {
            None => {
                let w = h.waiting.len();
                h.conns[c].inbox.push_back(format!(r#"{{"t":"searching","waiting":{w}}}"#));
                if let Some(o) = h.waiting.pop() {
                    h.next_room += 1;
                    let code = format!("room{}", h.next_room);
                    // seats in queue order (the first searcher is slot 0); the queue sockets end after `matched`
                    h.rooms.push((code.clone(), vec![o, c], 1, String::new()));
                    for k in [o, c] {
                        h.conns[k].inbox.push_back(format!(r#"{{"t":"matched","room":"{code}"}}"#));
                        h.conns[k].closed = Some(1000);
                    }
                } else {
                    h.waiting.push(c);
                }
            }
            Some(code) => {
                let r = h.rooms.iter().position(|x| x.0 == code).ok_or("no such room")?;
                h.conns[c].room = Some(code.to_string());
                // a rejoin takes the same user's seat
                let seats = h.rooms[r].1.clone();
                match seats.iter().position(|&k| h.conns[k].user == user) {
                    Some(slot) => h.rooms[r].1[slot] = c,
                    None => h.rooms[r].1.push(c),
                }
                h.rooms[r].2 += 1;
                let roster = h.roster(r);
                let id = h.conns[c].id.clone();
                h.conns[c].inbox.push_back(format!(r#"{{"t":"welcome","me":"{id}",{roster}}}"#));
                h.to_room(r, &format!(r#"{{"t":"join",{roster}}}"#));
            }
        }
        Ok(Box::new(FakeSocket { hub: self.hub.clone(), c }))
    }
}

fn cand(code: &str, port: u16) -> Candidate {
    Candidate { compat: "gd-melee-rust/test".into(), code: code.into(), lan: format!("127.0.0.1:{port}"), ext: String::new(), token: "1".into() }
}

/// One seat: queue, then join; returns the joined room.
fn seat(hub: H, user: &'static str, port: u16, cancel_at: Option<u64>) -> Result<(Joined, FakeOpener), (String, Option<String>)> {
    let mut op = FakeOpener { hub };
    let mut ticket = format!("{user}-t1");
    let t0 = now_ms();
    let cancelled = move || cancel_at.is_some_and(|c| now_ms() - t0 >= c);
    let mut claim = || true;
    let mut searching = |_w: i32| {};
    let mut renew = || Some(format!("{user}-renewed"));
    let mut log = |_: &str| {};
    let mut h = Hooks { cancelled: &cancelled, claim: &mut claim, searching: &mut searching, renew: &mut renew, now: &now_ms, log: &mut log };
    let (room, pass) = match queue(&mut op, &mut ticket, &mut h) {
        QueueEnd::Matched { room, ticket: t } => (room, t.unwrap_or(ticket)),
        QueueEnd::Cancelled { declined } => return Err(("Cancelled".into(), declined)),
        QueueEnd::Failed(e) => return Err((e, None)),
    };
    let j = join(&mut op, &room, pass, &cand(&format!("{}#1", user.to_uppercase()), port), &mut h, 15_000).map_err(|e| (e, None))?;
    Ok((j, op))
}

fn no_hooks<R>(f: impl FnOnce(&mut Hooks) -> R) -> R {
    let cancelled = || false;
    let mut claim = || true;
    let mut searching = |_w: i32| {};
    let mut renew = || Some("x-renewed".to_string());
    let mut log = |_: &str| {};
    let mut h = Hooks { cancelled: &cancelled, claim: &mut claim, searching: &mut searching, renew: &mut renew, now: &now_ms, log: &mut log };
    f(&mut h)
}

fn pair(hub: &H) -> ((Joined, FakeOpener), (Joined, FakeOpener)) {
    let (h1, h2) = (hub.clone(), hub.clone());
    let a = std::thread::spawn(move || seat(h1, "a", 51001, None));
    std::thread::sleep(Duration::from_millis(50));
    let b = std::thread::spawn(move || seat(h2, "b", 51002, None));
    (a.join().unwrap().unwrap(), b.join().unwrap().unwrap())
}

#[test]
fn queue_pairs_and_candidates_cross() {
    let hub: H = Default::default();
    let ((a, _), (b, _)) = pair(&hub);
    assert_eq!(a.code, b.code);
    assert_eq!((a.seat, a.opp_seat), (0, 1));
    assert_eq!((b.seat, b.opp_seat), (1, 0));
    assert_eq!(a.peer.to_string(), "127.0.0.1:51002");
    assert_eq!(b.peer.to_string(), "127.0.0.1:51001");
    assert_eq!(a.opp_code, "B#1");
    assert!(a.room.is_host() && !b.room.is_host());
    assert_eq!(a.room.port_ids, vec!["p-a".to_string(), "p-b".to_string()]);
    assert!(a.room.revision >= 2);
}

#[test]
fn begin_start_finish_result() {
    let hub: H = Default::default();
    let ((mut a, mut oa), (mut b, mut ob)) = pair(&hub);
    // both games start; only the host asks for the match
    a.room.game_started();
    b.room.game_started();
    let mut started = (false, false);
    for _ in 0..200 {
        no_hooks(|h| {
            for e in pump(&mut a, &mut oa, h, Duration::from_millis(2)) {
                started.0 |= matches!(e, Event::Started(_));
            }
            for e in pump(&mut b, &mut ob, h, Duration::from_millis(2)) {
                started.1 |= matches!(e, Event::Started(_));
            }
        });
        if started == (true, true) {
            break;
        }
    }
    assert_eq!(started, (true, true));
    let begins = hub.lock().unwrap().sent.iter().filter(|(_, f)| f.contains(r#""t":"begin""#)).count();
    assert_eq!(begins, 1, "only the host sends begin");
    assert_eq!(a.room.match_id, b.room.match_id);
    // the winner is port 1 (b): the finish names its participant id
    a.room.game_ended(1);
    let mut result = false;
    for _ in 0..200 {
        no_hooks(|h| {
            for e in pump(&mut a, &mut oa, h, Duration::from_millis(2)) {
                result |= matches!(e, Event::Result(_));
            }
        });
        if result {
            break;
        }
    }
    assert!(result);
    let fin = hub.lock().unwrap().sent.iter().find(|(_, f)| f.contains(r#""t":"finish""#)).cloned().unwrap();
    assert!(fin.1.contains(r#""winner":"p-b""#), "{}", fin.1);
    // no single winner: void
    let mut r = Room::new(2);
    r.match_id = "m".into();
    r.game_ended(-1);
    assert_eq!(r.due(0), vec![r#"{"t":"finish","matchId":"m","void":true}"#.to_string()]);
}

#[test]
fn refused_begin_is_retried() {
    let hub: H = Default::default();
    hub.lock().unwrap().refuse_begins = 1;
    let ((mut a, mut oa), _b) = pair(&hub);
    a.room.game_started();
    let mut started = false;
    let t0 = now_ms();
    while !started && now_ms() - t0 < 5000 {
        no_hooks(|h| started |= pump(&mut a, &mut oa, h, Duration::from_millis(5)).iter().any(|e| matches!(e, Event::Started(_))));
    }
    assert!(started);
    let begins = hub.lock().unwrap().sent.iter().filter(|(_, f)| f.contains(r#""t":"begin""#)).count();
    assert_eq!(begins, 2);
    assert!(now_ms() - t0 >= 1900, "the retry waits 2 s");
}

#[test]
fn cancel_in_queue_and_crossing_match_declined() {
    // alone in the queue: cancel sends `cancel`, nothing declined
    let hub: H = Default::default();
    let r = seat(hub.clone(), "a", 51001, Some(100));
    assert_eq!(r.err().unwrap(), ("Cancelled".to_string(), None));
    assert!(hub.lock().unwrap().sent.iter().any(|(_, f)| f == r#"{"t":"cancel"}"#));
    assert!(hub.lock().unwrap().waiting.is_empty());

    // the blend's claim refuses (Slippi won): the matched room is declined with a leave
    let hub: H = Default::default();
    let h1 = hub.clone();
    let a = std::thread::spawn(move || {
        let mut op = FakeOpener { hub: h1 };
        let mut ticket = "a-t1".to_string();
        let cancelled = || false;
        let mut claim = || false;
        let mut searching = |_w: i32| {};
        let mut renew = || None;
        let mut log = |_: &str| {};
        let mut h = Hooks { cancelled: &cancelled, claim: &mut claim, searching: &mut searching, renew: &mut renew, now: &now_ms, log: &mut log };
        queue(&mut op, &mut ticket, &mut h)
    });
    std::thread::sleep(Duration::from_millis(50));
    let b = std::thread::spawn({
        let h2 = hub.clone();
        move || seat(h2, "b", 51002, None)
    });
    let end = a.join().unwrap();
    assert!(matches!(end, QueueEnd::Cancelled { declined: Some(_) }), "{end:?}");
    // b sees its opponent leave the room and never pairs
    let e = b.join().unwrap().err().unwrap().0;
    assert!(e.contains("did not") || e.contains("join"), "{e}");
    assert!(hub.lock().unwrap().sent.iter().any(|(_, f)| f == r#"{"t":"leave"}"#));
}

#[test]
fn dropped_socket_rejoins_with_a_fresh_ticket() {
    let hub: H = Default::default();
    let ((mut a, mut oa), _b) = pair(&hub);
    let c = hub.lock().unwrap().conns.iter().position(|c| c.id == a.room.me).unwrap();
    hub.lock().unwrap().conns[c].closed = Some(1006);
    let mut ended = false;
    for _ in 0..20 {
        no_hooks(|h| ended |= pump(&mut a, &mut oa, h, Duration::from_millis(5)).iter().any(|e| matches!(e, Event::Ended(_))));
    }
    assert!(!ended);
    assert_eq!(a.rejoins, 1);
    assert_eq!(a.ticket, "x-renewed");
    assert!(hub.lock().unwrap().tickets.iter().any(|t| t == "x-renewed"));
    // a close the room means (1000) ends it
    let c = hub.lock().unwrap().conns.len() - 1;
    hub.lock().unwrap().conns[c].closed = Some(1000);
    no_hooks(|h| ended |= pump(&mut a, &mut oa, h, Duration::from_millis(5)).iter().any(|e| matches!(e, Event::Ended(_))));
    assert!(ended);
}

#[test]
fn roster_reads() {
    let mut r = Room::new(4);
    let m = Json::parse(r#"{"t":"welcome","me":"c1","host":"c0","revision":7,"participants":[{"id":"x","connectionId":"c0","slot":0,"team":1},{"id":"y","connectionId":"c1","slot":2,"team":0},{"id":"z","connectionId":"c1","slot":3,"localIndex":1}]}"#).unwrap();
    r.receive(&m, 0);
    assert_eq!((r.me.as_str(), r.host.as_str(), r.revision), ("c1", "c0", 7));
    assert_eq!(r.my_seat(), 2);
    assert_eq!(r.port_teams, [1, -1, 0, -1]);
    assert_eq!(r.port_ids, vec!["x", "", "y", "z"]);
    // a membership change without a roster clears it
    r.receive(&Json::parse(r#"{"t":"leave"}"#).unwrap(), 0);
    assert!(r.participants.is_empty());
}
