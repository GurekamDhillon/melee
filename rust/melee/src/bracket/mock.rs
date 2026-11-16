//! `MELEE_BRACKET_MOCK=1`: made-up bracket data for the Brackets screens (docs/rust-port/notes/brackets-ui.md) until
//! the server's (b) API exists. A small state machine on the game thread (no network, no files): one event, one called
//! set against "Peace", best of 3, the Slippi-style stage flow (game 1: strike 1-2-1 on the legal list; later games:
//! the winner bans 2, the loser picks), result cards and a set end that turns verified a moment later. The opponent's
//! turns play themselves after a short pause. A in the "game" step stands in for playing the game (it alternates who
//! wins, starting with you). Records go out in the same shapes the real data will use.

pub const STAGES: [&str; 6] = ["Battlefield", "Final Destination", "Dreamland", "Yoshi's Story", "Fountain of Dreams", "Pokemon Stadium"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// the next-thing card says YOUR SET IS UP
    Called,
    /// ready sent, waiting for the opponent (a moment)
    WaitReady,
    /// both on the normal character select
    Css,
    Strike,
    Ban,
    Pick,
    Game,
    Result,
    Done,
}
const PH: [&str; 9] = ["called", "wait_ready", "css", "strike", "ban", "pick", "game", "result", "done"];

pub struct Mock {
    pub phase: Phase,
    pub me: u32,
    pub opp: u32,
    pub game: u32,
    /// stage index -> 0 open, 1 struck / banned
    pub gone: [u8; 6],
    /// whose turn: true me
    pub my_turn: bool,
    /// how many removals are left in this step's turn
    pub left: u32,
    pub stage: Option<usize>,
    /// last game: I won, winner's stocks, winner's percent
    pub last: Option<(bool, u32, u32)>,
    pub verified: bool,
    /// frames until the opponent's (or the server's) next move
    pub wait: u32,
    pub ver: u64,
    pub log: Vec<String>,
}

pub const BEST_OF: u32 = 3;
pub const OPP: &str = "Peace";
pub const ROUND: &str = "Winners Round 1";
pub const EVENT: &str = "Sunday Melee #12";

impl Mock {
    pub fn new() -> Mock {
        Mock { phase: Phase::Called, me: 0, opp: 0, game: 1, gone: [0; 6], my_turn: true, left: 0, stage: None, last: None, verified: false, wait: 0, ver: 1, log: Vec::new() }
    }

    fn set(&mut self, p: Phase, wait: u32) {
        self.phase = p;
        self.wait = wait;
        self.ver += 1;
        self.log.push(format!("mock: {}", PH[p as usize]));
    }

    fn open(&self) -> Vec<usize> {
        (0..6).filter(|&i| self.gone[i] == 0).collect()
    }

    fn start_stage_step(&mut self) {
        self.gone = [0; 6];
        self.stage = None;
        if self.game == 1 {
            // 1-2-1 on the five starters (Fountain of Dreams is a counterpick in the mock's ruleset; the real list comes
            // from the event's legal_stages): I strike first (the higher seed in the mock)
            self.gone[4] = 1;
            self.my_turn = true;
            self.left = 1;
            self.set(Phase::Strike, 0);
        } else {
            let i_won = self.last.is_some_and(|l| l.0);
            self.my_turn = i_won;
            self.left = 2;
            self.set(Phase::Ban, if i_won { 0 } else { 90 });
        }
    }

    /// One removal (strike or ban) of stage `i` by whoever's turn it is.
    fn remove(&mut self, i: usize) {
        self.gone[i] = 1;
        self.left -= 1;
        self.ver += 1;
        let open = self.open();
        if self.phase == Phase::Strike {
            if open.len() == 1 {
                self.stage = Some(open[0]);
                self.set(Phase::Game, 0);
            } else if self.left == 0 {
                self.my_turn = !self.my_turn;
                self.left = if open.len() == 4 { 2 } else { 1 };
                self.wait = if self.my_turn { 0 } else { 90 };
            }
        } else if self.left == 0 {
            // bans done: the loser picks
            self.my_turn = !self.my_turn;
            self.set(Phase::Pick, if self.my_turn { 0 } else { 90 });
        }
    }

    /// A request from the screen. Returns a status line.
    pub fn request(&mut self, what: &str, arg: &str) -> Result<String, String> {
        match (what, self.phase) {
            ("ready", Phase::Called) => {
                self.set(Phase::WaitReady, 90);
                Ok("Ready. Waiting for Peace.".into())
            }
            ("css", Phase::Css) => {
                self.start_stage_step();
                Ok("Characters locked.".into())
            }
            ("stage", Phase::Strike | Phase::Ban | Phase::Pick) => {
                let i: usize = arg.parse().map_err(|_| "no such stage".to_string())?;
                if !self.my_turn {
                    return Err("Not your turn.".into());
                }
                if i >= 6 || self.gone[i] != 0 {
                    return Err("That stage is not open.".into());
                }
                if self.phase == Phase::Pick {
                    self.stage = Some(i);
                    self.set(Phase::Game, 0);
                    return Ok(format!("Game {} on {}.", self.game, STAGES[i]));
                }
                self.remove(i);
                Ok(format!("{} {}.", if self.phase == Phase::Ban { "Banned" } else { "Struck" }, STAGES[i]))
            }
            ("game", Phase::Game) => {
                // the stand-in for a played game: you win game 1, Peace game 2, you game 3
                let i_won = self.game % 2 == 1;
                if i_won { self.me += 1 } else { self.opp += 1 }
                self.last = Some((i_won, 2 + self.game % 2, 40 + 23 * self.game));
                self.set(Phase::Result, 300);
                Ok("Game over.".into())
            }
            _ => Err("Not now.".into()),
        }
    }

    /// Once a frame: the opponent's moves and the timed steps.
    pub fn tick(&mut self) {
        if self.wait > 0 {
            self.wait -= 1;
            return;
        }
        match self.phase {
            Phase::WaitReady => self.set(Phase::Css, 0),
            Phase::Strike | Phase::Ban if !self.my_turn => {
                let i = self.open()[0];
                self.remove(i);
                if !self.my_turn {
                    self.wait = 45;
                }
            }
            Phase::Pick if !self.my_turn => {
                let i = self.open()[0];
                self.stage = Some(i);
                self.set(Phase::Game, 0);
            }
            Phase::Result => {
                if self.me * 2 > BEST_OF || self.opp * 2 > BEST_OF {
                    self.set(Phase::Done, 240);
                } else {
                    self.game += 1;
                    self.start_stage_step();
                }
            }
            Phase::Done if !self.verified => {
                self.verified = true;
                self.ver += 1;
            }
            _ => {}
        }
    }

    /// LIST_BRACKET_NEXT: kind, event, round, opponent, eta seconds, set id, best of.
    pub fn next_rec(&self) -> Vec<String> {
        let kind = match self.phase {
            Phase::Called => "called",
            Phase::Done => "upcoming",
            _ => "playing",
        };
        let (round, opp, eta) = if self.phase == Phase::Done {
            if self.me > self.opp { ("Winners Round 2", "Mango", "540") } else { ("Losers Round 1", "Hbox", "300") }
        } else {
            (ROUND, OPP, "0")
        };
        [kind, EVENT, round, opp, eta, "3", &BEST_OF.to_string()].iter().map(|s| s.to_string()).collect()
    }

    /// LIST_BRACKET_SETS (my events): id, name, status line.
    pub fn events(&self) -> Vec<Vec<String>> {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        vec![
            s(&["12", EVENT, if self.phase == Phase::Done { "in Winners Round 2" } else { "playing Winners Round 1" }]),
            s(&["9", "Thursday Ladder", "checked in, starts 19:00"]),
            s(&["4", "Monthly #3", "finished 5th"]),
        ]
    }

    /// LIST_BRACKET_SET: phase, my score, opponent score, best of, game, my tag, opponent tag, my turn (0/1),
    /// removals left, stages (name list joined by '|'), gone ('0'/'1' per stage), stage picked (index or ""), last game
    /// (i won 0/1, winner's stocks, winner's percent, or ""), verified (0/1), what's next, round.
    pub fn set_rec(&self) -> Vec<String> {
        let gone: String = self.gone.iter().map(|&g| if g != 0 { '1' } else { '0' }).collect();
        let last = self.last.map_or(String::new(), |(w, s, p)| format!("{}|{s}|{p}", w as u8));
        let next = match self.phase {
            Phase::Result if !(self.me * 2 > BEST_OF || self.opp * 2 > BEST_OF) => {
                if self.last.is_some_and(|l| l.0) { format!("You ban 2, {OPP} picks") } else { format!("You pick, {OPP} bans 2") }
            }
            Phase::Done => {
                if self.me > self.opp { "Next: Winners Round 2 vs Mango, in about 9 min".into() } else { "Next: Losers Round 1 vs Hbox, in about 5 min".into() }
            }
            _ => String::new(),
        };
        vec![
            PH[self.phase as usize].into(),
            self.me.to_string(),
            self.opp.to_string(),
            BEST_OF.to_string(),
            self.game.to_string(),
            "You".into(),
            OPP.into(),
            if self.my_turn { "1" } else { "0" }.into(),
            self.left.to_string(),
            STAGES.join("|"),
            gone,
            self.stage.map_or(String::new(), |s| s.to_string()),
            last,
            if self.verified { "1" } else { "0" }.into(),
            next,
            ROUND.into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut Mock, n: u32) {
        for _ in 0..n {
            m.tick();
        }
    }

    #[test]
    fn a_whole_set() {
        let mut m = Mock::new();
        assert!(m.request("stage", "0").is_err());
        m.request("ready", "").unwrap();
        run(&mut m, 200);
        assert_eq!(m.phase, Phase::Css);
        m.request("css", "").unwrap();
        // 1-2-1 on the five starters: me 1, them 2, me 1, and the last one is game 1's stage
        assert_eq!((m.phase, m.my_turn, m.left), (Phase::Strike, true, 1));
        assert!(m.request("stage", "4").is_err(), "not a starter");
        m.request("stage", "1").unwrap();
        assert_eq!((m.my_turn, m.left), (false, 2));
        run(&mut m, 400);
        assert_eq!((m.phase, m.my_turn, m.left, m.open().len()), (Phase::Strike, true, 1, 2));
        let i = m.open()[0];
        m.request("stage", &i.to_string()).unwrap();
        assert_eq!(m.phase, Phase::Game);
        assert!(m.stage.is_some());
        m.request("game", "").unwrap();
        assert_eq!((m.me, m.opp, m.phase), (1, 0, Phase::Result));
        assert_eq!(m.set_rec()[14], "You ban 2, Peace picks");
        run(&mut m, 400);
        // game 2: I won, so I ban 2 and Peace picks
        assert_eq!((m.phase, m.my_turn, m.left), (Phase::Ban, true, 2));
        m.request("stage", "0").unwrap();
        m.request("stage", "2").unwrap();
        assert_eq!((m.phase, m.my_turn), (Phase::Pick, false));
        run(&mut m, 200);
        assert_eq!(m.phase, Phase::Game);
        m.request("game", "").unwrap();
        assert_eq!((m.me, m.opp), (1, 1));
        run(&mut m, 400);
        // game 3: Peace won game 2, Peace bans, I pick
        assert_eq!((m.phase, m.my_turn), (Phase::Ban, false));
        run(&mut m, 400);
        assert_eq!((m.phase, m.my_turn), (Phase::Pick, true));
        let i = m.open()[0];
        m.request("stage", &i.to_string()).unwrap();
        m.request("game", "").unwrap();
        run(&mut m, 400);
        assert_eq!((m.phase, m.me, m.opp), (Phase::Done, 2, 1));
        assert!(!m.verified);
        run(&mut m, 400);
        assert!(m.verified);
        assert_eq!(m.next_rec()[0], "upcoming");
        assert_eq!(m.set_rec().len(), 16);
    }
}
