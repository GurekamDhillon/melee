//! The server's phase-2b in-game API (melee-autobracket docs/api.md section 8, `proto::ingame` at phase1 2fb5489),
//! read with the engine's JSON reader and turned into the Brackets screens' records (brackets-ui.md, the same shapes
//! the mock serves). Pure functions: the worker thread calls them, tests feed them api.md's examples.
//!
//! Routes: `GET /v1/me` (MeStatus), `POST /v1/me/link` (LinkReq -> LinkResult), `GET /v1/me/home` (Home),
//! `GET /v1/me/badge` (Badge, polled with If-None-Match), `POST /v1/sets/{e}/{s}/ready` (ReadyReq),
//! `GET /v1/sets/{e}/{s}/flow` (SetFlow), `POST /v1/sets/{e}/{s}/games/{n}/stage` (StageMoveReq). Every pair is
//! `[me, opponent]`; stages travel by their stable Melee id.

use crate::cosmetics_import::Json;

pub fn parse(body: &[u8]) -> Option<Json> {
    Json::parse(std::str::from_utf8(body).ok()?).ok()
}

fn s<'a>(j: Option<&'a Json>) -> &'a str {
    j.and_then(|v| v.str()).unwrap_or("")
}
fn n(j: Option<&Json>) -> Option<f64> {
    match j {
        Some(Json::Num(x)) => Some(*x),
        _ => None,
    }
}
fn u(j: Option<&Json>) -> u64 {
    n(j).filter(|x| *x >= 0.0).map_or(0, |x| x as u64)
}
fn b(j: Option<&Json>) -> bool {
    matches!(j, Some(Json::Bool(true)))
}
fn arr(j: Option<&Json>) -> &[Json] {
    match j {
        Some(Json::Arr(v)) => v,
        _ => &[],
    }
}
/// An externally tagged enum: ("Variant", fields) for an object, ("Variant", Null) for a bare string.
fn tag(j: &Json) -> (&str, &Json) {
    static NULL: Json = Json::Null;
    match j {
        Json::Str(t) => (t.as_str(), &NULL),
        Json::Obj(v) if v.len() == 1 => (v[0].0.as_str(), &v[0].1),
        _ => ("", &NULL),
    }
}
fn pair(j: Option<&Json>) -> [u64; 2] {
    let a = arr(j);
    [u(a.first()), u(a.get(1))]
}
fn pair_b(j: Option<&Json>) -> [bool; 2] {
    let a = arr(j);
    [b(a.first()), b(a.get(1))]
}

/// `MeStatus`: (tag, linked).
pub fn me(j: &Json) -> (String, bool) {
    (s(j.get("tag")).to_string(), b(j.get("linked")))
}

/// `Badge.set_up`.
pub fn badge(j: &Json) -> bool {
    b(j.get("set_up"))
}

/// A `Next` as a status line (the reference client's words).
pub fn next_line(j: &Json) -> String {
    let (t, f) = tag(j);
    match t {
        "NotEntered" => "Not entered".into(),
        "Waiting" => "Waiting for your next set".into(),
        "Review" => format!("Set {} is being reviewed by the organiser", u(f.get("set_id"))),
        "Eliminated" => "Eliminated".into(),
        "Champion" => "Champion!".into(),
        "Play" => {
            let w = pair(f.get("wins"));
            format!("Playing vs {} ({}-{}, game {})", s(f.get("opponent_tag")), w[0], w[1], u(f.get("game_no")))
        }
        _ => String::new(),
    }
}

/// The current set the home screen names: (event, set, game) of `SetUp`.
pub fn home_set(home: &Json) -> Option<(i64, u32, u32)> {
    let (t, f) = tag(home.get("action")?);
    (t == "SetUp").then(|| (u(f.get("event_id")) as i64, u(f.get("set_id")) as u32, u(f.get("game_no")) as u32))
}

/// `Home` -> (BRACKET_NEXT record, BRACKET_SETS records, badge).
/// NEXT: kind (called / playing / upcoming / review / none), event name, round, opponent, eta s, set id, best of,
/// event id.
pub fn home(home: &Json) -> (Vec<String>, Vec<Vec<String>>, bool) {
    let events: Vec<Vec<String>> = arr(home.get("events"))
        .iter()
        .map(|e| {
            let ev = e.get("event");
            vec![u(ev.and_then(|v| v.get("id"))).to_string(), s(ev.and_then(|v| v.get("name"))).to_string(), e.get("next").map_or(String::new(), next_line)]
        })
        .collect();
    let name_of = |id: u64| events.iter().find(|r| r[0] == id.to_string()).map_or(String::new(), |r| r[1].clone());
    let next = match home.get("action").map(tag) {
        Some((t @ ("SetUp" | "Upcoming" | "Review"), f)) => {
            let ev = u(f.get("event_id"));
            let kind = match t {
                "SetUp" if pair_b(f.get("ready"))[0] => "playing",
                "SetUp" => "called",
                "Upcoming" => "upcoming",
                _ => "review",
            };
            let eta = n(f.get("eta_secs")).map_or(String::new(), |x| (x as u64).to_string());
            let opp = match f.get("opponent_tag") {
                Some(Json::Str(o)) => o.clone(),
                _ => "to be decided".into(),
            };
            vec![kind.into(), name_of(ev), s(f.get("round_name")).into(), opp, eta, u(f.get("set_id")).to_string(), u(f.get("best_of")).to_string(), ev.to_string()]
        }
        _ => vec!["none".into(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new(), String::new()],
    };
    (next, events, b(home.get("badge")))
}

/// The pool's stage ids, in the record's order (STAGE requests send an index into it).
pub fn pool_ids(flow: &Json) -> Vec<u16> {
    arr(flow.get("stage").and_then(|st| st.get("pool"))).iter().map(|p| u(p.get("id")) as u16).collect()
}

/// Whose stage move is awaited: (kind "strike" / "ban" / "pick", mine, count), None when no turn.
pub fn turn(flow: &Json) -> Option<(String, bool, u64)> {
    let t = flow.get("stage")?.get("turn")?;
    if matches!(t, Json::Null) {
        return None;
    }
    Some((s(t.get("kind")).to_string(), s(t.get("who")) == "me", u(t.get("count"))))
}

/// `SetFlow` -> the BRACKET_SET record (brackets-ui.md): phase, my score, opponent score, best of, game, my tag,
/// opponent tag, my turn, removals left, stage names ('|'), gone ('0'/'1'), picked index, last game
/// (won|stocks|percent), verified, what's next, round; then 16 pool ids ('|'), 17 relay, 18 room code, 19 mock (0).
/// `my_tag`: from /v1/me. `fresh_result`: a game was judged moments ago (the ~5 s result card). `after`: the home
/// action's words for the next set once this one is done ("" while unknown).
pub fn set_record(flow: &Json, my_tag: &str, fresh_result: bool, after: &str) -> Vec<String> {
    let w = pair(flow.get("wins"));
    let ready = pair_b(flow.get("ready"));
    let phase = s(flow.get("phase"));
    let st = flow.get("stage").filter(|x| !matches!(x, Json::Null));
    let pool = arr(st.and_then(|x| x.get("pool")));
    let moves = arr(st.and_then(|x| x.get("moves")));
    let gone_ids: Vec<u64> = moves.iter().map(|m| u(m.get("stage").and_then(|x| x.get("id")))).collect();
    let chosen = st.and_then(|x| x.get("chosen")).filter(|c| !matches!(c, Json::Null)).map(|c| u(c.get("id")));
    let tn = turn(flow);
    let games = arr(flow.get("games"));
    let last = games.iter().rev().find(|g| !matches!(g.get("winner"), None | Some(Json::Null)));
    let last_s = last.map_or(String::new(), |g| format!("{}|{}|{}", (s(g.get("winner")) == "me") as u8, u(g.get("winner_stocks")), n(g.get("winner_percent")).map_or(0, |p| p.round() as u64)));
    let opp = s(flow.get("opponent_tag"));
    let outcome = flow.get("outcome").filter(|o| !matches!(o, Json::Null));
    let ph = match phase {
        "ready_check" => if ready[0] { "wait_ready" } else { "called" },
        "stage_select" if fresh_result && last.is_some() => "result",
        "stage_select" => tn.as_ref().map_or("game", |t| match t.0.as_str() { "strike" => "strike", "ban" => "ban", _ => "pick" }),
        "playing" => "game",
        "judging" => "judging",
        "review" => "review",
        "done" => "done",
        _ => "",
    };
    let next = match ph {
        "result" => match &tn {
            Some((k, mine, c)) if k == "ban" => if *mine { format!("You ban {c}, {opp} picks") } else { format!("You pick, {opp} bans {c}") },
            Some((_, mine, _)) => if *mine { format!("You pick, {opp} banned") } else { format!("{opp} picks") },
            None => String::new(),
        },
        "done" => {
            let (t, f) = outcome.map_or(("", &Json::Null), tag);
            let base = match t {
                "Advanced" => "You advance",
                "DroppedToLosers" => "Down to Losers",
                "Eliminated" => "Out of the event. Thanks for playing!",
                "Champion" => "Champion!",
                _ => "",
            };
            let nl = if matches!(t, "Advanced" | "DroppedToLosers") { f.get("next").map_or(String::new(), next_line) } else { String::new() };
            if !after.is_empty() && matches!(t, "Advanced" | "DroppedToLosers") {
                format!("{base}: {after}")
            } else if !nl.is_empty() {
                format!("{base}: {nl}")
            } else {
                base.to_string()
            }
        }
        "game" | "judging" => match flow.get("room_code") {
            Some(Json::Str(c)) => format!("Room {c} on {}", s(flow.get("relay"))),
            _ => format!("Relay {}: waiting for the host's room code", s(flow.get("relay"))),
        },
        _ => String::new(),
    };
    let round = {
        let side = s(flow.get("side"));
        format!("{} round {}", match side { "GrandFinal" => "Grand final", "Losers" => "Losers", _ => "Winners" }, u(flow.get("round")))
    };
    let names: Vec<&str> = pool.iter().map(|p| s(p.get("name"))).collect();
    let ids: Vec<u64> = pool.iter().map(|p| u(p.get("id"))).collect();
    let gone: String = ids.iter().map(|i| if gone_ids.contains(i) { '1' } else { '0' }).collect();
    let picked = chosen.and_then(|c| ids.iter().position(|&i| i == c)).map_or(String::new(), |i| i.to_string());
    vec![
        ph.into(),
        w[0].to_string(),
        w[1].to_string(),
        u(flow.get("best_of")).to_string(),
        u(flow.get("game_no")).max(1).to_string(),
        if my_tag.is_empty() { "You".into() } else { my_tag.into() },
        opp.into(),
        if tn.as_ref().is_some_and(|t| t.1) { "1" } else { "0" }.into(),
        tn.as_ref().map_or(0, |t| t.2).to_string(),
        names.join("|"),
        gone,
        picked,
        last_s,
        if b(flow.get("verified")) { "1" } else { "0" }.into(),
        next,
        round,
        ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("|"),
        s(flow.get("relay")).into(),
        match flow.get("room_code") {
            Some(Json::Str(c)) => c.clone(),
            _ => String::new(),
        },
        "0".into(),
    ]
}

fn when(t: Option<&Json>) -> String {
    n(t).filter(|x| *x > 0.0).map_or(String::new(), |x| (x as u64).to_string())
}

/// `GET /v1/events` ([EventSummary]) -> BRACKET_BROWSE records: id, name, state, entrants, starts_at (unix s or ""),
/// description. Sign-up first, then running, then the rest.
pub fn browse(j: &Json) -> Vec<Vec<String>> {
    let mut v: Vec<Vec<String>> = arr(Some(j))
        .iter()
        .map(|e| vec![u(e.get("id")).to_string(), s(e.get("name")).into(), s(e.get("state")).into(), u(e.get("entrants")).to_string(), when(e.get("starts_at")), s(e.get("description")).into()])
        .collect();
    let rank = |st: &str| match st {
        "signup" => 0,
        "running" => 1,
        _ => 2,
    };
    v.sort_by_key(|r| rank(&r[2]));
    v
}

/// `GET /v1/events/{id}` (EventView) -> the BRACKET_EVENT record: id, name, state, entrants, starts_at, description,
/// best of, stocks, timer s, legal stage names ('|'), entrant tags (', ', at most 24).
pub fn event_view(j: &Json) -> Vec<String> {
    let sm = j.get("summary");
    let r = j.get("ruleset");
    let stages: Vec<String> = arr(r.and_then(|r| r.get("legal_stages"))).iter().map(|x| stage_name(u(Some(x)) as u16)).collect();
    let tags: Vec<&str> = arr(j.get("tags")).iter().take(24).map(|t| t.str().unwrap_or("")).collect();
    vec![
        u(sm.and_then(|x| x.get("id"))).to_string(),
        s(sm.and_then(|x| x.get("name"))).into(),
        s(sm.and_then(|x| x.get("state"))).into(),
        u(sm.and_then(|x| x.get("entrants"))).to_string(),
        when(sm.and_then(|x| x.get("starts_at"))),
        s(sm.and_then(|x| x.get("description"))).into(),
        u(r.and_then(|x| x.get("best_of"))).to_string(),
        u(r.and_then(|x| x.get("stocks"))).to_string(),
        u(r.and_then(|x| x.get("timer_secs"))).to_string(),
        stages.join("|"),
        tags.join(", "),
    ]
}

/// The server's stage table (proto::ingame STAGE_NAMES) for the ids a ruleset lists.
pub fn stage_name(id: u16) -> String {
    const T: [(u16, &str); 30] = [
        (2, "Fountain of Dreams"), (3, "Pokémon Stadium"), (4, "Princess Peach's Castle"), (5, "Kongo Jungle"), (6, "Brinstar"),
        (7, "Corneria"), (8, "Yoshi's Story"), (9, "Onett"), (10, "Mute City"), (11, "Rainbow Cruise"), (12, "Jungle Japes"),
        (13, "Great Bay"), (14, "Hyrule Temple"), (15, "Brinstar Depths"), (16, "Yoshi's Island"), (17, "Green Greens"),
        (18, "Fourside"), (19, "Mushroom Kingdom I"), (20, "Mushroom Kingdom II"), (22, "Venom"), (23, "Poké Floats"),
        (24, "Big Blue"), (25, "Icicle Mountain"), (26, "Icetop"), (27, "Flat Zone"), (28, "Dream Land N64"),
        (29, "Yoshi's Island N64"), (30, "Kongo Jungle N64"), (31, "Battlefield"), (32, "Final Destination"),
    ];
    T.iter().find(|t| t.0 == id).map_or_else(|| format!("Stage {id}"), |t| t.1.to_string())
}

/// A sign-up refused for the engine build: 403 `{"error":"engine build not accepted","accepted":[...]}`.
pub fn accepted_builds(body: &[u8]) -> Option<Vec<String>> {
    let j = parse(body)?;
    let a = j.get("accepted")?;
    Some(arr(Some(a)).iter().filter_map(|x| x.str().map(str::to_string)).collect())
}

/// 403 `key is not registered`: the key is not a player yet (the link card).
pub fn not_registered(status: u32, body: &[u8]) -> bool {
    status == 403 && parse(body).is_some_and(|j| s(j.get("error")) == "key is not registered")
}

/// What starting the current game needs from the flow: (host?, relay, room code, chosen stage id).
pub fn start_info(flow: &Json) -> Option<(bool, String, Option<String>, Option<u16>)> {
    let relay = s(flow.get("relay"));
    if relay.is_empty() {
        return None;
    }
    let chosen = flow.get("stage").and_then(|st| st.get("chosen")).filter(|c| !matches!(c, Json::Null)).map(|c| u(c.get("id")) as u16);
    let code = match flow.get("room_code") {
        Some(Json::Str(c)) if !c.is_empty() => Some(c.clone()),
        _ => None,
    };
    Some((s(flow.get("role")) == "Host", relay.into(), code, chosen))
}

pub fn room_body(code: &str) -> String {
    format!("{{\"code\":{}}}", jstr(code))
}

/// The JSON bodies the client sends.
pub fn link_body(code: &str) -> String {
    format!("{{\"code\":{}}}", jstr(code))
}
pub fn ready_body(build: &str) -> String {
    format!("{{\"engine_build\":{}}}", jstr(build))
}
pub fn stage_body(kind: &str, stage: u16) -> String {
    format!("{{\"kind\":{},\"stage\":{stage}}}", jstr(kind))
}
fn jstr(v: &str) -> String {
    let mut o = String::from("\"");
    for c in v.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    // api.md section 8's examples (phase1 2fb5489)
    const HOME: &str = r#"{"badge":true,"action":{"SetUp":{"event_id":1,"set_id":0,"side":"Winners","round":1,"round_name":"Final","opponent_tag":"Armada","best_of":3,"game_no":1,"ready":[false,true]}},"events":[{"event":{"id":1,"name":"Sunday","state":"running","entrants":2,"starts_at":null,"description":null,"created_at":0},"next":"Waiting"}]}"#;
    const FLOW: &str = r#"{"event_id":1,"set_id":0,"side":"Winners","round":1,"opponent_tag":"Armada","role":"Host","relay":"relay.example.org:51600","room_code":null,"best_of":3,"wins":[1,0],"game_no":2,"ready":[true,true],"phase":"stage_select","stage":{"mode":"counterpick","pool":[{"id":2,"name":"Fountain of Dreams"},{"id":3,"name":"Pokémon Stadium"},{"id":31,"name":"Battlefield"},{"id":32,"name":"Final Destination"}],"moves":[{"who":"me","kind":"ban","stage":{"id":31,"name":"Battlefield"}}],"turn":{"who":"me","kind":"ban","count":1},"chosen":null},"games":[{"game_no":1,"stage":{"id":32,"name":"Final Destination"},"winner":"me","winner_stocks":2,"winner_percent":87.4,"counted":true,"verdict":"Verified"}],"verified":true,"outcome":null}"#;

    #[test]
    fn home_record() {
        let j = parse(HOME.as_bytes()).unwrap();
        let (next, events, badge) = home(&j);
        assert!(badge);
        assert_eq!(next, ["called", "Sunday", "Final", "Armada", "", "0", "3", "1"]);
        assert_eq!(events, vec![vec!["1".to_string(), "Sunday".into(), "Waiting for your next set".into()]]);
        assert_eq!(home_set(&j), Some((1, 0, 1)));
        let up = parse(br#"{"badge":false,"action":{"Upcoming":{"event_id":1,"set_id":4,"side":"Losers","round":1,"round_name":"Losers round 1","opponent_tag":null,"eta_secs":300}},"events":[]}"#).unwrap();
        assert_eq!(home(&up).0[..5], ["upcoming", "", "Losers round 1", "to be decided", "300"]);
        assert_eq!(home(&parse(br#"{"badge":false,"action":"None","events":[]}"#).unwrap()).0[0], "none");
    }

    #[test]
    fn flow_record() {
        let j = parse(FLOW.as_bytes()).unwrap();
        let r = set_record(&j, "Mango", false, "");
        assert_eq!(r[0], "ban");
        assert_eq!((r[1].as_str(), r[2].as_str(), r[4].as_str()), ("1", "0", "2"));
        assert_eq!((r[7].as_str(), r[8].as_str()), ("1", "1"));
        assert_eq!(r[9], "Fountain of Dreams|Pokémon Stadium|Battlefield|Final Destination");
        assert_eq!(r[10], "0010");
        assert_eq!(r[12], "1|2|87");
        assert_eq!(r[16], "2|3|31|32");
        assert_eq!(pool_ids(&j), vec![2, 3, 31, 32]);
        let card = set_record(&j, "Mango", true, "");
        assert_eq!((card[0].as_str(), card[14].as_str()), ("result", "You ban 1, Armada picks"));
        let done = FLOW.replace(r#""phase":"stage_select""#, r#""phase":"done""#).replace(r#""outcome":null"#, r#""outcome":{"DroppedToLosers":{"next":"Waiting"}}"#);
        let d = set_record(&parse(done.as_bytes()).unwrap(), "Mango", false, "Losers round 1 vs Hbox, in about 5 min");
        assert_eq!((d[0].as_str(), d[13].as_str()), ("done", "1"));
        assert_eq!(d[14], "Down to Losers: Losers round 1 vs Hbox, in about 5 min");
        let champ = FLOW.replace(r#""phase":"stage_select""#, r#""phase":"done""#).replace(r#""outcome":null"#, r#""outcome":"Champion""#);
        assert_eq!(set_record(&parse(champ.as_bytes()).unwrap(), "", false, "")[14], "Champion!");
    }

    #[test]
    fn browse_and_join() {
        let ev = br#"[{"id":1,"name":"Weekly #12","state":"running","entrants":2,"starts_at":1795000000,"description":"Bo3, Friday 8pm","created_at":1763200000},{"id":2,"name":"Open","state":"signup","entrants":0,"starts_at":null,"description":null,"created_at":0}]"#;
        let b = browse(&parse(ev).unwrap());
        assert_eq!(b[0][..4], ["2", "Open", "signup", "0"], "sign-up first");
        assert_eq!(b[1][4], "1795000000");
        let view = br#"{"summary":{"id":1,"name":"Weekly #12","state":"running","entrants":2,"starts_at":1795000000,"description":"Bo3","created_at":1},"ruleset":{"stocks":4,"timer_secs":480,"best_of":3,"best_of_finals":5,"legal_stages":[2,3,8,28,31,32],"banned_characters":[],"allowed_gecko":[]},"tags":["Mango","Armada"],"bracket":null,"scores":[],"decided_by":[],"unverified_games":0}"#;
        let r = event_view(&parse(view).unwrap());
        assert_eq!(r[6..9], ["3", "4", "480"]);
        assert_eq!(r[9], "Fountain of Dreams|Pokémon Stadium|Yoshi's Story|Dream Land N64|Battlefield|Final Destination");
        assert_eq!(r[10], "Mango, Armada");
        assert_eq!(accepted_builds(br#"{"error":"engine build not accepted","accepted":["melee (rust) 0.1.0 x+abc"]}"#).unwrap(), vec!["melee (rust) 0.1.0 x+abc".to_string()]);
        assert!(not_registered(403, br#"{"error":"key is not registered"}"#));
        assert!(!not_registered(401, br#"{"error":"key is not registered"}"#));
        let f = parse(FLOW.as_bytes()).unwrap();
        assert_eq!(start_info(&f), Some((true, "relay.example.org:51600".into(), None, None)));
        assert_eq!(room_body("KQ7X"), r#"{"code":"KQ7X"}"#);
    }

    #[test]
    fn bodies() {
        assert_eq!(link_body("ABCD2345"), r#"{"code":"ABCD2345"}"#);
        assert_eq!(stage_body("strike", 31), r#"{"kind":"strike","stage":31}"#);
        assert_eq!(ready_body("melee \"x\""), r#"{"engine_build":"melee \"x\""}"#);
        assert_eq!(me(&parse(br#"{"tag":"Mango","linked":true,"display_name":"m"}"#).unwrap()), ("Mango".into(), true));
        assert!(badge(&parse(br#"{"set_up":true,"event_id":1,"set_id":0}"#).unwrap()));
    }
}
