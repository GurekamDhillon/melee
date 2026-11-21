//! YouGame's own queue for the Slippi online menu (plan steps 10/11): the routing policy, the Unranked blend over
//! YouGame's pool and Slippi's, and the `mpTicket` request over the app bridge. The logic follows Hero's
//! `yougame_mode_policy.h`, `yougame_blend.h` and `find_match` (Melee Unlocked, GPLv3, ported with permission);
//! the code is ours.
//!
//! Everything here is pure except [`StdioBridge`] (the real call, the app's `yg {...}` stdio lines): the routing and
//! the blend run with a fake clock and a fake bridge in the tests (`tests/yougame_mm.rs`). Agents never reach the
//! real YouGame service: the real path is wired, not exercised.

use std::time::Duration;

/// Slippi's mode ids (Hero `MU_SLP_MODE_*`).
pub const MODE_RANKED: i32 = 0;
pub const MODE_UNRANKED: i32 = 1;
pub const MODE_DIRECT: i32 = 2;
pub const MODE_TEAMS: i32 = 3;
pub const MODE_PARTY: i32 = 4;

/// Where a search goes (Hero `MatchmakingRoute`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// YouGame's queue (rooms) only.
    YouGame,
    /// Slippi's matchmaking (the app granted the Slippi login).
    Slippi,
    /// Unranked over both pools; the first opponent wins.
    Blend,
    /// Not offered (Direct / Teams / Party without a Slippi login).
    Locked,
}

/// The Slippi page's own modes: they never use YouGame's queue. A friend room is the separate Friends flow.
pub fn slippi_menu_mode(mode: i32, friend_room: bool) -> bool {
    (MODE_DIRECT..=MODE_PARTY).contains(&mode) && !friend_room
}

/// Hero's `matchmaking_route`. `vanilla`: the game plays what Slippi's Unranked plays (no gameplay mods).
pub fn matchmaking_route(mode: i32, yougame: bool, slippi_login: bool, agent: bool, friend_room: bool, vanilla: bool) -> Route {
    if yougame && slippi_menu_mode(mode, friend_room) {
        return if slippi_login && !agent { Route::Slippi } else { Route::Locked };
    }
    if agent {
        return Route::YouGame;
    }
    if !yougame {
        return Route::Slippi;
    }
    if mode == MODE_UNRANKED && slippi_login && !friend_room && vanilla {
        Route::Blend
    } else {
        Route::YouGame
    }
}

/// Ranked is not offered in YouGame mode until it plays like Slippi's (Hero `yougame_ranked_offered`).
pub const YOUGAME_RANKED_OFFERED: bool = false;

/// The status line over the select while a search runs (Hero `kSearchBoth` / `kSearchYouGameOnly`).
pub const SEARCH_BOTH: &str = "Searching Slippi + YouGame";
pub const SEARCH_YOUGAME_ONLY: &str = "Searching YouGame players";

// ---- the blend (Hero `yougame::blend::Coordinator`) -------------------------------------------------------------

/// Someone else in YouGame's queue: YouGame alone this long, then Slippi too.
pub const YOUGAME_FIRST_MS: u64 = 15_000;
/// No word from YouGame's queue by then: Slippi too.
pub const QUEUE_WORD_MS: u64 = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    YouGame,
    Slippi,
}
impl Side {
    pub fn name(self) -> &'static str {
        match self {
            Side::YouGame => "YouGame",
            Side::Slippi => "Slippi",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Run {
    Off,
    Live,
    Done,
}

/// One blended Unranked search. YouGame's side searches from the start; Slippi's starts when YouGame's queue says
/// nobody else is there, when it says nothing for [`QUEUE_WORD_MS`], or after [`YOUGAME_FIRST_MS`] alone with
/// someone waiting. The first side given an opponent is the match; the other is cancelled. No clock or I/O: the
/// caller passes `now` (ms) and does what it says.
#[derive(Debug)]
pub struct Coordinator {
    start: u64,
    yougame: Run,
    slippi: Run,
    heard: bool,
    committed: bool,
    stopped: bool,
    start_pending: bool,
    winner: Side,
    solo_until: u64,
    log: Vec<String>,
}

impl Coordinator {
    pub fn new(now: u64) -> Self {
        Coordinator { start: now, yougame: Run::Live, slippi: Run::Off, heard: false, committed: false, stopped: false, start_pending: false, winner: Side::YouGame, solo_until: 0, log: Vec::new() }
    }
    fn run(&mut self, s: Side) -> &mut Run {
        match s {
            Side::YouGame => &mut self.yougame,
            Side::Slippi => &mut self.slippi,
        }
    }
    fn over_for_new_work(&self) -> bool {
        self.committed || self.stopped
    }
    fn say(&mut self, line: String) {
        self.log.push(format!("yougame: unranked blend: {line}"));
    }
    fn start_slippi(&mut self, now: u64) {
        self.slippi = Run::Live;
        self.start_pending = true;
        self.say(format!("Slippi search started after {} ms", now - self.start));
    }

    /// YouGame's queue said how many are searching there, this player included.
    pub fn searching(&mut self, waiting: i32, now: u64) {
        if self.over_for_new_work() || self.yougame != Run::Live {
            return;
        }
        let first = !self.heard;
        self.heard = true;
        if self.slippi != Run::Off {
            if first {
                self.say(format!("YouGame queue waiting {waiting} -> both at once"));
            }
            return;
        }
        if waiting >= 2 {
            if first {
                self.solo_until = now + YOUGAME_FIRST_MS;
                self.say(format!("YouGame queue waiting {waiting} -> YouGame first"));
            }
            return;
        }
        self.say(format!("YouGame queue waiting {waiting} -> both at once"));
        self.start_slippi(now);
    }

    /// A side was given an opponent. True: it is the match and the other side is cancelled. False: decline it.
    pub fn matched(&mut self, side: Side, now: u64) -> bool {
        if self.committed || self.stopped || *self.run(side) != Run::Live {
            self.say(format!("declined a late {} match", side.name()));
            if !self.committed || side != self.winner {
                *self.run(side) = Run::Done;
            }
            return false;
        }
        self.committed = true;
        self.winner = side;
        let other = self.run(if side == Side::YouGame { Side::Slippi } else { Side::YouGame });
        let cancelled = *other == Run::Live;
        *other = Run::Done;
        self.start_pending = false;
        let after = now - self.start;
        self.say(format!("matched on {} after {after} ms; {}", side.name(), if cancelled { "cancelled the other" } else { "the other was not searching" }));
        true
    }

    /// A side's search ended without a match. Before a commit the other goes on; YouGame failing before Slippi
    /// started starts Slippi now.
    pub fn failed(&mut self, side: Side, now: u64) {
        if *self.run(side) != Run::Live {
            return;
        }
        *self.run(side) = Run::Done;
        if self.over_for_new_work() {
            return;
        }
        if side == Side::YouGame && self.slippi == Run::Off {
            self.start_slippi(now);
        }
    }

    pub fn tick(&mut self, now: u64) {
        if self.over_for_new_work() || self.slippi != Run::Off {
            return;
        }
        if !self.heard && now - self.start >= QUEUE_WORD_MS || self.heard && self.solo_until != 0 && now >= self.solo_until {
            self.start_slippi(now);
        }
    }

    /// The player left the search.
    pub fn stop(&mut self) {
        self.stopped = true;
        self.start_pending = false;
        for r in [&mut self.yougame, &mut self.slippi] {
            if *r == Run::Live {
                *r = Run::Done;
            }
        }
    }

    /// Slippi's search is due to start (once).
    pub fn take_start_slippi(&mut self) -> bool {
        std::mem::take(&mut self.start_pending) && self.slippi == Run::Live
    }
    pub fn live(&self, side: Side) -> bool {
        match side {
            Side::YouGame => self.yougame == Run::Live,
            Side::Slippi => self.slippi == Run::Live,
        }
    }
    pub fn committed(&self) -> Option<Side> {
        self.committed.then_some(self.winner)
    }
    pub fn yougame_only(&self) -> bool {
        !self.over_for_new_work() && self.yougame == Run::Live && self.slippi == Run::Done
    }
    /// Nothing committed and neither side can still find an opponent.
    pub fn over(&self) -> bool {
        !self.committed && !self.stopped && self.yougame == Run::Done && self.slippi == Run::Done
    }
    pub fn take_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.log)
    }
}

// ---- the ticket (Hero `find_match`'s `mpTicket`) ----------------------------------------------------------------

/// The app bridge: one request, its answer (the JSON after `yg `), or `None` (no app, timeout).
pub trait Bridge {
    fn request(&mut self, ty: &str, fields: &str, timeout: Duration) -> Option<String>;
}

/// The real bridge: the YouGame app's stdio lines (`crate::yougame::request`).
pub struct StdioBridge;
impl Bridge for StdioBridge {
    fn request(&mut self, ty: &str, fields: &str, timeout: Duration) -> Option<String> {
        crate::yougame::request(ty, fields, timeout)
    }
}

/// The `mpTicket` members for a 1v1 search (Unranked or Direct): two players, singles, our lobby contract,
/// never ranked here; Unranked stays in the room after a match (`afterMatch: stay`), Direct names the code.
pub fn ticket_fields(mode: i32, compat: &str, direct: Option<&str>) -> String {
    use crate::yougame::json_str;
    let mut f = format!(
        r#""players":2,"mode":"singles","lobby":{{"version":1,"minPlayers":2,"maxLocalPlayers":1,"compatibility":{}}},"ranked":false"#,
        json_str(compat)
    );
    if mode == MODE_UNRANKED {
        f.push_str(r#","afterMatch":"stay""#);
    }
    if let (MODE_DIRECT, Some(code)) = (mode, direct) {
        f.push_str(&format!(r#","direct":{}"#, json_str(code)));
    }
    f
}

/// The app's answer to `mpTicket`: the rooms URL, the ticket, and the room code when one was named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub url: String,
    pub ticket: String,
    pub room: Option<String>,
}

/// The string after `"key":` in a JSON object (flat; escapes `\"` and `\\` only).
pub fn json_string(j: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let mut from = 0;
    while let Some(i) = j[from..].find(&pat) {
        let at = from + i + pat.len();
        let rest = j[at..].trim_start();
        if let Some(rest) = rest.strip_prefix(':') {
            let rest = rest.trim_start().strip_prefix('"')?;
            let mut out = String::new();
            let mut it = rest.chars();
            while let Some(c) = it.next() {
                match c {
                    '"' => return Some(out),
                    '\\' => out.push(it.next()?),
                    c => out.push(c),
                }
            }
            return None;
        }
        from = at;
    }
    None
}

/// The answer's verdict: a ticket, or the refusal the player sees.
pub fn parse_ticket(answer: &str) -> Result<Ticket, String> {
    if answer.contains(r#""ok":false"#) {
        return Err(match json_string(answer, "error") {
            Some(e) if !e.is_empty() => format!("YouGame refused the search: {}", e.chars().take(80).collect::<String>()),
            _ => "YouGame refused the search".into(),
        });
    }
    match (json_string(answer, "url"), json_string(answer, "ticket")) {
        (Some(url), Some(ticket)) if !url.is_empty() && !ticket.is_empty() => Ok(Ticket { url, ticket, room: json_string(answer, "room").filter(|r| !r.is_empty()) }),
        _ => Err("YouGame did not return a ticket".into()),
    }
}

/// Ask the app for a ticket (30 s, as Hero's `bridge(request, 30000, ...)`).
pub fn request_ticket(bridge: &mut dyn Bridge, mode: i32, compat: &str, direct: Option<&str>) -> Result<Ticket, String> {
    match bridge.request("mpTicket", &ticket_fields(mode, compat, direct), Duration::from_secs(30)) {
        Some(a) => parse_ticket(&a),
        None => Err("YouGame did not answer".into()),
    }
}

/// What the online menu does with a pick in YouGame mode: the route, the status line, and for a YouGame side the
/// ticket (or its refusal).
#[derive(Debug, PartialEq, Eq)]
pub struct Start {
    pub route: Route,
    pub status: &'static str,
    pub ticket: Option<Result<Ticket, String>>,
}

/// Route a pick and, when YouGame's queue takes part, request its ticket through `bridge`.
pub fn start_search(bridge: &mut dyn Bridge, mode: i32, slippi_login: bool, agent: bool, vanilla: bool, compat: &str) -> Start {
    let route = matchmaking_route(mode, true, slippi_login, agent, false, vanilla);
    let status = match route {
        Route::Blend => SEARCH_BOTH,
        Route::YouGame if mode == MODE_UNRANKED => SEARCH_YOUGAME_ONLY,
        _ => "",
    };
    let ticket = matches!(route, Route::YouGame | Route::Blend).then(|| request_ticket(bridge, mode, compat, None));
    Start { route, status, ticket }
}
