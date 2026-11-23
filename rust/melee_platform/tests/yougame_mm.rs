//! YouGame queue routing and the Unranked blend against a fake app bridge (no YouGame service is reached).

use melee_platform::yougame_mm::*;
use std::time::Duration;

/// A fake YouGame app: canned answers per request type, and every request it was sent.
struct FakeBridge {
    answers: Vec<(&'static str, Option<String>)>,
    sent: Vec<(String, String)>,
}
impl FakeBridge {
    fn new(answers: Vec<(&'static str, Option<String>)>) -> Self {
        FakeBridge { answers, sent: Vec::new() }
    }
}
impl Bridge for FakeBridge {
    fn request(&mut self, ty: &str, fields: &str, _t: Duration) -> Option<String> {
        self.sent.push((ty.to_string(), fields.to_string()));
        self.answers.iter().find(|(t, _)| *t == ty).and_then(|(_, a)| a.clone())
    }
}

fn ok_ticket() -> Option<String> {
    Some(r#"{"re":3,"ok":true,"url":"wss://rooms.invalid/v1","ticket":"t-123","room":"ROOM42"}"#.into())
}

#[test]
fn route_table() {
    use Route::*;
    // mode, yougame, login, agent, friend_room, vanilla
    let rows = [
        (MODE_DIRECT, true, true, false, false, true, Slippi),
        (MODE_DIRECT, true, false, false, false, true, Locked),
        (MODE_TEAMS, true, false, false, false, true, Locked),
        (MODE_PARTY, true, true, true, false, true, Locked), // an agent never reaches Slippi
        (MODE_UNRANKED, true, true, false, false, true, Blend),
        (MODE_UNRANKED, true, true, false, false, false, YouGame), // modded gameplay: YouGame alone
        (MODE_UNRANKED, true, false, false, false, true, YouGame),
        (MODE_UNRANKED, true, true, true, false, true, YouGame), // agent: YouGame only
        (MODE_UNRANKED, true, true, false, true, true, YouGame), // a friend room
        (MODE_UNRANKED, false, false, false, false, true, Slippi), // not YouGame mode
        (MODE_DIRECT, false, false, false, false, true, Slippi),
    ];
    for (m, y, l, a, f, v, want) in rows {
        assert_eq!(matchmaking_route(m, y, l, a, f, v), want, "mode {m} yg {y} login {l} agent {a} friend {f} vanilla {v}");
    }
    assert!(!YOUGAME_RANKED_OFFERED);
}

#[test]
fn unranked_without_login_asks_you_game_for_a_ticket() {
    let mut b = FakeBridge::new(vec![("mpTicket", ok_ticket())]);
    let s = start_search(&mut b, MODE_UNRANKED, false, false, true, "compat-1");
    assert_eq!(s.route, Route::YouGame);
    assert_eq!(s.status, SEARCH_YOUGAME_ONLY);
    let t = s.ticket.unwrap().unwrap();
    assert_eq!((t.url.as_str(), t.ticket.as_str(), t.room.as_deref()), ("wss://rooms.invalid/v1", "t-123", Some("ROOM42")));
    assert_eq!(b.sent.len(), 1);
    let (ty, f) = &b.sent[0];
    assert_eq!(ty, "mpTicket");
    for want in [r#""players":2"#, r#""mode":"singles""#, r#""ranked":false"#, r#""afterMatch":"stay""#, r#""compatibility":"compat-1""#, r#""minPlayers":2"#] {
        assert!(f.contains(want), "{want} missing from {f}");
    }
    assert!(!f.contains("direct"));
}

#[test]
fn unranked_with_login_blends_and_still_asks_you_game() {
    let mut b = FakeBridge::new(vec![("mpTicket", ok_ticket())]);
    let s = start_search(&mut b, MODE_UNRANKED, true, false, true, "c");
    assert_eq!((s.route, s.status), (Route::Blend, SEARCH_BOTH));
    assert!(s.ticket.unwrap().is_ok());
}

#[test]
fn locked_and_slippi_routes_never_call_the_bridge() {
    let mut b = FakeBridge::new(vec![("mpTicket", ok_ticket())]);
    assert_eq!(start_search(&mut b, MODE_DIRECT, false, false, true, "c").route, Route::Locked);
    assert_eq!(start_search(&mut b, MODE_TEAMS, true, false, true, "c").route, Route::Slippi);
    assert!(b.sent.is_empty());
}

/// The staging refusal (Nov 22): "Invalid lobby configuration; public queues allow one local player per device".
/// The ticket body for a public queue must carry exactly one local player and a compatibility the site accepts.
#[test]
fn public_queue_ticket_body_is_one_local_player_and_valid_lobby() {
    let mut b = FakeBridge::new(vec![("mpTicket", ok_ticket())]);
    let s = start_search(&mut b, MODE_UNRANKED, true, false, true, "gd-melee-rust/0.1.0");
    assert!(s.ticket.unwrap().is_ok());
    let (ty, f) = &b.sent[0];
    assert_eq!(ty, "mpTicket");
    assert_eq!(
        f,
        r#""players":2,"mode":"singles","lobby":{"version":1,"minPlayers":2,"maxLocalPlayers":1,"compatibility":"gd-melee-rust-0.1.0"},"ranked":false,"afterMatch":"stay""#
    );
    for bad in ["local_players", "localPlayers", "party_local", "local_partner", "maxLocalPlayers\":2"] {
        assert!(!f.contains(bad), "{bad} in {f}");
    }
    let c = json_string(f, "compatibility").unwrap();
    assert!((1..=64).contains(&c.len()) && c.chars().all(|c| c.is_ascii_alphanumeric() || ".:_-".contains(c)), "{c}");
    assert_eq!(lobby_compat("gd-melee-rust:1.2.3"), "gd-melee-rust:1.2.3");
    assert_eq!(lobby_compat(""), "default");
    assert_eq!(lobby_compat(&"x".repeat(80)).len(), 64);
}

#[test]
fn refusals_and_silence() {
    let mut b = FakeBridge::new(vec![("mpTicket", Some(r#"{"re":1,"ok":false,"error":"signed out"}"#.into()))]);
    assert_eq!(request_ticket(&mut b, MODE_UNRANKED, "c", None), Err("YouGame refused the search: signed out".into()));
    let mut b = FakeBridge::new(vec![("mpTicket", None)]);
    assert_eq!(request_ticket(&mut b, MODE_UNRANKED, "c", None), Err("YouGame did not answer".into()));
    let mut b = FakeBridge::new(vec![("mpTicket", Some(r#"{"re":1,"ok":true}"#.into()))]);
    assert_eq!(request_ticket(&mut b, MODE_UNRANKED, "c", None), Err("YouGame did not return a ticket".into()));
    assert!(ticket_fields(MODE_DIRECT, "c", Some("AB#1")).contains(r#""direct":"AB#1""#));
}

#[test]
fn blend_you_game_first_then_both() {
    let mut c = Coordinator::new(1000);
    c.searching(2, 1200); // someone else waits: YouGame alone for 15 s
    c.tick(1000 + QUEUE_WORD_MS + 1);
    assert!(!c.take_start_slippi());
    c.tick(1200 + YOUGAME_FIRST_MS);
    assert!(c.take_start_slippi());
    assert!(!c.take_start_slippi()); // once
    assert!(c.matched(Side::Slippi, 20_000));
    assert_eq!(c.committed(), Some(Side::Slippi));
    assert!(!c.live(Side::YouGame));
    assert!(!c.matched(Side::YouGame, 20_100)); // late: declined
    let log = c.take_log().join("\n");
    assert!(log.contains("YouGame first") && log.contains("cancelled the other") && log.contains("declined a late YouGame match"), "{log}");
}

#[test]
fn blend_alone_in_queue_starts_both_and_you_game_wins() {
    let mut c = Coordinator::new(0);
    c.searching(1, 100);
    assert!(c.take_start_slippi());
    assert!(c.matched(Side::YouGame, 500));
    assert!(!c.live(Side::Slippi));
    assert!(!c.over());
}

#[test]
fn blend_silent_queue_and_failures() {
    let mut c = Coordinator::new(0);
    c.tick(QUEUE_WORD_MS - 1);
    assert!(!c.take_start_slippi());
    c.tick(QUEUE_WORD_MS);
    assert!(c.take_start_slippi());
    c.failed(Side::Slippi, 4000);
    assert!(c.yougame_only());
    c.failed(Side::YouGame, 5000);
    assert!(c.over());

    // YouGame refused before Slippi started: Slippi starts now
    let mut c = Coordinator::new(0);
    c.failed(Side::YouGame, 10);
    assert!(c.take_start_slippi() && c.live(Side::Slippi));

    // the player left: nothing commits
    let mut c = Coordinator::new(0);
    c.searching(1, 1);
    c.stop();
    assert!(!c.matched(Side::Slippi, 2) && c.committed().is_none() && !c.over());
}
