//! The pick/ban lobby of gw_netplay.c (`lb`): the competitive ruleset, its wire messages and the
//! countdown. A port of `lb_new_set`, `lb_reset_game`, `lb_apply_`, `lb_countdown_at`, `lb_encode`,
//! `lb_decode`, `lb_list_rx`, `lb_list_encode`, the starter/counterpick grouping and the stage list
//! builders. Pure state: no sockets, no clock (the countdown takes elapsed frames).
//!
//! Not ported: the Envoy reward picks (`E`/`W`/`V`/`RPICK`), the stage run director (`R`/`K`/`F`/
//! `X`), the saved-run record (`U`/`Q`/`T`/`Y`/`Z`), the identity and mod-sync exchanges. A message
//! of those kinds is ignored like any unknown letter, which is what the C does for a peer that
//! never sends them.

pub const LB_OFF: i32 = 0;
pub const LB_CHAR_BLIND: i32 = 1;
pub const LB_STRIKE: i32 = 2;
pub const LB_BAN: i32 = 3;
pub const LB_PICK: i32 = 4;
pub const LB_CHAR_WINNER: i32 = 5;
pub const LB_CHAR_LOSER: i32 = 6;
pub const LB_READY: i32 = 7;
pub const LB_GO: i32 = 8;

pub const LB_FREE: i32 = 0;
pub const LB_STRUCK_P1: i32 = 1;
pub const LB_STRUCK_P2: i32 = 2;
pub const LB_BANNED: i32 = 3;
pub const LB_PICKED: i32 = 4;

pub const LB_MAX_STAGES: usize = 256;
pub const LB_LIST_CHUNK: usize = 8;
pub const LB_COUNTDOWN: i32 = 180;

/// The starters (external stage ids) in grid order: Battlefield, Final Destination, Dream Land,
/// Yoshi's Story, Fountain of Dreams. Every other stage is a counterpick.
pub const LB_STARTER_EXT: [i32; 5] = [31, 32, 28, 8, 2];
/// The competitive list: the starters, then the counterpick (Pokemon Stadium).
pub const LB_DEFAULT_EXT: [i32; 6] = [31, 32, 28, 8, 2, 3];
const LB_STRIKE_COUNTS: [i32; 3] = [1, 2, 2];

#[derive(Clone, Debug)]
pub struct Lobby {
    pub phase: i32,
    pub game: i32,
    pub winner: i32,
    pub score: [i32; 2],
    /// Whose action it is (strike / ban / pick / counterpick character).
    pub turn: i32,
    pub left: i32,
    pub step: i32,
    /// Game 1: who strikes first (the coin flip).
    pub first: i32,
    /// The stage list: 0 competitive, 1 all stages.
    pub mode: i32,
    pub nstages: i32,
    pub stage_ext: Vec<i32>,
    pub list_id: i32,
    pub list_final: bool,
    /// Host: next index to send in "L" chunks, -1 = all sent.
    pub list_tx: i32,
    pub rx_id: i32,
    pub rx_total: i32,
    pub rx_count: i32,
    pub list_ok: bool,
    pub stage: Vec<i32>,
    pub chosen: i32,
    pub ck: [i32; 2],
    pub color: [i32; 2],
    pub locked: [bool; 2],
    pub ready: [bool; 2],
    pub countdown: i32,
    pub seed: u32,
    pub have_state: bool,
    /// Wall-clock ms of the countdown's zero (None = not running).
    pub cd_t0: Option<u32>,
    pub seq: u32,
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            phase: LB_OFF,
            game: 0,
            winner: -1,
            score: [0; 2],
            turn: 0,
            left: 0,
            step: 0,
            first: 0,
            mode: 0,
            nstages: 0,
            stage_ext: vec![0; LB_MAX_STAGES],
            list_id: 0,
            list_final: false,
            list_tx: -1,
            rx_id: 0,
            rx_total: 0,
            rx_count: 0,
            list_ok: false,
            stage: vec![LB_FREE; LB_MAX_STAGES],
            chosen: -1,
            ck: [0; 2],
            color: [0; 2],
            locked: [false; 2],
            ready: [false; 2],
            countdown: 0,
            seed: 0,
            have_state: false,
            cd_t0: None,
            seq: 0,
        }
    }
}

/// What the lobby asks about the two installs (the content identities of gw_mexid.c).
pub trait LobbyEnv {
    /// `gw_Netplay_StageAvailable`: both players have the stage (external id).
    fn stage_available(&self, ext: i32) -> bool;
    /// Every stage both have, for the all-stages list (local external ids).
    fn all_stages(&self) -> Vec<i32>;
    /// The opponent's identity lists are in (`gw_MexId_PeerReady`).
    fn peer_ready(&self) -> bool;
    /// Scene / list token for a stage (`gw_MexId_TokenForExt`).
    fn token_for_ext(&self, ext: i32) -> String;
    /// A list token back to this install's external stage id (-1 unknown).
    fn ext_for_token(&self, tok: &str) -> i32;
}

impl Lobby {
    pub fn new() -> Self {
        Self::default()
    }

    fn st(&self, i: usize) -> i32 {
        self.stage.get(i).copied().unwrap_or(LB_FREE)
    }

    pub fn default_stages(&mut self, env: &dyn LobbyEnv) {
        self.nstages = 0;
        for &e in LB_DEFAULT_EXT.iter() {
            if env.stage_available(e) {
                self.stage_ext[self.nstages as usize] = e;
                self.nstages += 1;
            }
        }
    }

    fn starter_rank(ext: i32) -> i32 {
        LB_STARTER_EXT.iter().position(|&e| e == ext).map_or(-1, |p| p as i32)
    }

    fn count_starters(&self) -> i32 {
        (0..self.nstages as usize).filter(|&i| Self::starter_rank(self.stage_ext[i]) >= 0).count() as i32
    }

    /// 1 if list entry i is a starter. A list with fewer than two starters has no groups.
    pub fn is_starter(&self, i: i32) -> bool {
        if i < 0 || i >= self.nstages {
            return false;
        }
        if self.count_starters() < 2 {
            return true;
        }
        Self::starter_rank(self.stage_ext[i as usize]) >= 0
    }

    fn order_groups(&mut self) {
        let mut tmp = Vec::new();
        for r in LB_STARTER_EXT.iter() {
            if (0..self.nstages as usize).any(|i| self.stage_ext[i] == *r) {
                tmp.push(*r);
            }
        }
        for i in 0..self.nstages as usize {
            if Self::starter_rank(self.stage_ext[i]) < 0 {
                tmp.push(self.stage_ext[i]);
            }
        }
        for (i, e) in tmp.iter().enumerate() {
            self.stage_ext[i] = *e;
        }
        self.nstages = tmp.len() as i32;
    }

    /// Host: the stage list for the room's mode (`lb_build_list`).
    pub fn build_list(&mut self, env: &dyn LobbyEnv) {
        if self.mode == 0 {
            self.default_stages(env);
        } else {
            self.nstages = 0;
            for ext in env.all_stages() {
                if ext <= 0 || !env.stage_available(ext) || self.nstages as usize >= LB_MAX_STAGES {
                    continue;
                }
                if !(0..self.nstages as usize).any(|k| self.stage_ext[k] == ext) {
                    self.stage_ext[self.nstages as usize] = ext;
                    self.nstages += 1;
                }
            }
            if self.nstages == 0 {
                self.default_stages(env);
            }
            self.order_groups();
        }
        self.list_id += 1;
        self.list_tx = 0;
        self.list_final = env.peer_ready();
    }

    /// A new opponent: game 1, no winner, 0-0 (`lb_new_set`).
    pub fn new_set(&mut self, host_ck: i32, guest_ck: i32, env: &dyn LobbyEnv) {
        self.game = 1;
        self.winner = -1;
        self.score = [0; 2];
        self.ck = [host_ck, guest_ck];
        self.color = [0; 2];
        if self.nstages <= 0 {
            self.default_stages(env);
        }
        self.seq += 1;
    }

    /// The next game of the set (`lb_reset_game`).
    pub fn reset_game(&mut self, env: &dyn LobbyEnv) {
        if self.nstages <= 0 {
            self.default_stages(env);
        }
        for s in self.stage.iter_mut() {
            *s = LB_FREE;
        }
        self.chosen = -1;
        self.locked = [false; 2];
        self.ready = [false; 2];
        self.countdown = 0;
        self.step = 0;
        if self.game <= 1 || self.winner < 0 {
            self.phase = LB_CHAR_BLIND;
            self.first = ((self.seed >> 7) & 1) as i32; // the coin flip, from the host's seed
            self.turn = self.first;
            self.left = LB_STRIKE_COUNTS[0];
        } else {
            self.phase = LB_BAN; // winner bans 2, loser picks
            self.turn = self.winner;
            self.left = 2;
        }
        self.seq += 1;
    }

    fn stages_free(&self) -> i32 {
        (0..self.nstages as usize).filter(|&i| self.st(i) == LB_FREE).count() as i32
    }

    fn starters_free(&self) -> i32 {
        (0..self.nstages).filter(|&i| self.st(i as usize) == LB_FREE && self.is_starter(i)).count() as i32
    }

    fn after_stage(&mut self) {
        if self.game <= 1 || self.winner < 0 {
            self.phase = LB_READY;
        } else {
            self.phase = LB_CHAR_WINNER;
            self.turn = self.winner;
        }
    }

    /// Apply one player's action (host). Returns true if the state changed. `blocked` is the
    /// C's READY refusals that need app state (different mods: the caller passes true).
    pub fn apply(&mut self, who: i32, act: &str, a: i32, b: i32, ready_blocked: bool) -> bool {
        if !(0..=1).contains(&who) {
            return false;
        }
        let w = who as usize;
        if act == "CHAR" {
            if self.phase == LB_CHAR_BLIND && !self.locked[w] {
                self.ck[w] = a;
                self.color[w] = b;
                self.locked[w] = true;
                if self.locked[0] && self.locked[1] {
                    if self.mode == 1 {
                        self.phase = LB_BAN; // all stages: the coin winner bans 2, the other picks
                        self.turn = self.first;
                        self.left = 2;
                    } else {
                        self.phase = LB_STRIKE;
                    }
                }
                return true;
            }
            if (self.phase == LB_CHAR_WINNER || self.phase == LB_CHAR_LOSER) && self.turn == who {
                self.ck[w] = a;
                self.color[w] = b;
                self.locked[w] = true;
                if self.phase == LB_CHAR_WINNER {
                    self.phase = LB_CHAR_LOSER;
                    self.turn = 1 - who;
                } else {
                    self.phase = LB_READY;
                }
                return true;
            }
            return false;
        }
        if act == "READY" {
            if self.phase != LB_READY || self.ready[w] == (a != 0) {
                return false;
            }
            if a != 0 && ready_blocked {
                return false;
            }
            self.ready[w] = a != 0;
            self.countdown = if self.ready[0] && self.ready[1] { LB_COUNTDOWN } else { 0 };
            return true;
        }
        if a < 0 || a >= self.nstages || self.turn != who || self.st(a as usize) != LB_FREE {
            return false;
        }
        let ai = a as usize;
        if act == "STRIKE" && self.phase == LB_STRIKE {
            if !self.is_starter(a) {
                return false; // a counterpick: not in game 1
            }
            self.stage[ai] = if who == 0 { LB_STRUCK_P1 } else { LB_STRUCK_P2 };
            self.left -= 1;
            if self.left <= 0 && self.starters_free() > 1 {
                self.step += 1;
                self.turn = 1 - self.turn;
                self.left = LB_STRIKE_COUNTS[if self.step < 3 { self.step as usize } else { 2 }];
            }
            if self.left > self.starters_free() - 1 {
                self.left = self.starters_free() - 1;
            }
            if self.starters_free() <= 1 {
                for i in 0..self.nstages {
                    if self.st(i as usize) == LB_FREE && self.is_starter(i) {
                        self.stage[i as usize] = LB_PICKED;
                        self.chosen = i;
                    }
                }
                self.left = 0;
                self.after_stage();
            }
            return true;
        }
        if act == "BAN" && self.phase == LB_BAN {
            self.stage[ai] = LB_BANNED;
            self.left -= 1;
            if self.left <= 0 || self.stages_free() <= 1 {
                self.phase = LB_PICK;
                self.turn = 1 - who;
                self.left = 1;
            }
            return true;
        }
        if act == "PICK" && self.phase == LB_PICK {
            self.stage[ai] = LB_PICKED;
            self.chosen = a;
            self.left = 0;
            self.after_stage();
            return true;
        }
        false
    }

    /// The countdown by the clock: `elapsed` frames (1/60 s) since both became ready. Returns
    /// true when it ran out with both still ready: the match starts (`lb_countdown_at`).
    pub fn countdown_at(&mut self, elapsed: i32) -> bool {
        if self.phase != LB_READY || self.countdown <= 0 {
            return false;
        }
        if !(self.ready[0] && self.ready[1]) {
            self.countdown = 0;
            return false;
        }
        self.countdown = LB_COUNTDOWN - elapsed.max(0);
        if self.countdown <= 0 {
            self.countdown = 0;
            return true;
        }
        false
    }

    /// "S ..." - the whole state, host to guest (`lb_encode`).
    pub fn encode(&self) -> String {
        let mut st = String::new();
        for i in 0..self.nstages as usize {
            if st.len() >= 108 {
                break;
            }
            if self.st(i) != LB_FREE {
                if !st.is_empty() {
                    st.push(',');
                }
                st += &format!("{}.{}", i, self.st(i));
            }
        }
        format!(
            "S {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.phase,
            self.game,
            self.winner,
            self.score[0],
            self.score[1],
            self.turn,
            self.left,
            self.step,
            self.first,
            if st.is_empty() { "-" } else { &st },
            self.chosen,
            self.ck[0],
            self.color[0],
            self.ck[1],
            self.color[1],
            self.locked[0] as i32,
            self.locked[1] as i32,
            self.ready[0] as i32,
            self.ready[1] as i32,
            self.countdown,
            self.mode,
            self.nstages,
            self.list_id
        )
    }

    /// `lb_decode` (guest): returns how many fields were read (sscanf's count).
    pub fn decode(&mut self, m: &str) -> usize {
        let body = m.get(2..).unwrap_or("");
        let mut it = body.split_whitespace();
        let mut k = 0usize;
        let mut v = [0i32; 23];
        let mut st = String::new();
        for i in 0..23 {
            let Some(tok) = it.next() else { break };
            if i == 9 {
                st = tok.chars().take(123).collect();
                k += 1;
                continue;
            }
            match tok.parse::<i64>() {
                Ok(n) => {
                    v[i] = n as i32;
                    k += 1;
                }
                Err(_) => break,
            }
        }
        // assign in order as far as sscanf got
        let set = |k: usize, i: usize| k > i;
        if set(k, 0) {
            self.phase = v[0];
        }
        if set(k, 1) {
            self.game = v[1];
        }
        if set(k, 2) {
            self.winner = v[2];
        }
        if set(k, 3) {
            self.score[0] = v[3];
        }
        if set(k, 4) {
            self.score[1] = v[4];
        }
        if set(k, 5) {
            self.turn = v[5];
        }
        if set(k, 6) {
            self.left = v[6];
        }
        if set(k, 7) {
            self.step = v[7];
        }
        if set(k, 8) {
            self.first = v[8];
        }
        if set(k, 10) {
            self.chosen = v[10];
        }
        if set(k, 11) {
            self.ck[0] = v[11];
        }
        if set(k, 12) {
            self.color[0] = v[12];
        }
        if set(k, 13) {
            self.ck[1] = v[13];
        }
        if set(k, 14) {
            self.color[1] = v[14];
        }
        if set(k, 15) {
            self.locked[0] = v[15] != 0;
        }
        if set(k, 16) {
            self.locked[1] = v[16] != 0;
        }
        if set(k, 17) {
            self.ready[0] = v[17] != 0;
        }
        if set(k, 18) {
            self.ready[1] = v[18] != 0;
        }
        if set(k, 19) {
            self.countdown = v[19];
        }
        if k >= 10 {
            for s in self.stage.iter_mut() {
                *s = LB_FREE;
            }
            let mut c = st.as_str();
            while !c.is_empty() && !c.starts_with('-') {
                let Some((ix, rest)) = c.split_once('.') else { break };
                let (vs, tail) = match rest.split_once(',') {
                    Some((a, b)) => (a, b),
                    None => (rest, ""),
                };
                let (Ok(ix), Ok(val)) = (ix.parse::<i32>(), vs.parse::<i32>()) else { break };
                if (0..LB_MAX_STAGES as i32).contains(&ix) {
                    self.stage[ix as usize] = val;
                }
                c = tail;
            }
        }
        if k >= 23 {
            self.mode = v[20];
            let n = v[21];
            let id = v[22];
            self.list_ok = id == self.rx_id && self.rx_count >= self.rx_total && self.rx_total == n;
            if self.list_ok {
                self.nstages = n;
            }
        }
        self.have_state = true;
        self.seq += 1;
        k
    }

    /// Guest: one "L <id> <start> <total> <tok,tok,...>" chunk of the host's stage list.
    /// Returns true when the list is complete (`lb_list_rx`).
    pub fn list_rx(&mut self, m: &str, env: &dyn LobbyEnv) -> bool {
        let body = m.get(2..).unwrap_or("");
        let mut parts = body.splitn(4, ' ');
        let (Some(a), Some(b), Some(c)) = (parts.next(), parts.next(), parts.next()) else { return false };
        let (Ok(id), Ok(mut start), Ok(total)) = (a.parse::<i32>(), b.parse::<i32>(), c.parse::<i32>()) else { return false };
        if total < 0 || total > LB_MAX_STAGES as i32 {
            return false;
        }
        if id != self.rx_id {
            self.rx_id = id;
            self.rx_total = total;
            self.rx_count = 0;
        }
        let toks = parts.next().unwrap_or("");
        for tok in toks.split(',') {
            if tok.is_empty() || start >= total {
                break;
            }
            let ext = env.ext_for_token(tok);
            if (start as usize) < LB_MAX_STAGES {
                self.stage_ext[start as usize] = ext;
            }
            start += 1;
            self.rx_count += 1;
        }
        self.rx_count >= self.rx_total
    }

    /// The "L" chunk of the list from `start`; returns (message, index after it).
    pub fn list_encode(&self, start: i32, env: &dyn LobbyEnv) -> (String, i32) {
        let end = (start + LB_LIST_CHUNK as i32).min(self.nstages);
        let mut m = format!("L {} {} {} ", self.list_id, start, self.nstages);
        for i in start..end {
            if i > start {
                m.push(',');
            }
            m += &env.token_for_ext(self.stage_ext[i as usize]);
        }
        (m, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AllStages;
    impl LobbyEnv for AllStages {
        fn stage_available(&self, _e: i32) -> bool {
            true
        }
        fn all_stages(&self) -> Vec<i32> {
            (1..=32).collect()
        }
        fn peer_ready(&self) -> bool {
            true
        }
        fn token_for_ext(&self, e: i32) -> String {
            format!("ext:{e}")
        }
        fn ext_for_token(&self, t: &str) -> i32 {
            t.strip_prefix("ext:").and_then(|n| n.parse().ok()).unwrap_or(-1)
        }
    }

    fn fresh(seed: u32) -> Lobby {
        let e = AllStages;
        let mut lb = Lobby::new();
        lb.seed = seed;
        lb.default_stages(&e);
        lb.new_set(2, 9, &e);
        lb.reset_game(&e);
        lb
    }

    #[test]
    fn competitive_game_one_flow() {
        let e = AllStages;
        let mut lb = fresh(0x80); // coin: (0x80>>7)&1 = 1 -> guest strikes first
        assert_eq!(lb.phase, LB_CHAR_BLIND);
        assert_eq!(lb.first, 1);
        assert_eq!(lb.nstages, 6);
        assert!(lb.apply(0, "CHAR", 2, 0, false));
        assert_eq!(lb.phase, LB_CHAR_BLIND);
        assert!(!lb.apply(0, "CHAR", 3, 0, false), "locked");
        assert!(lb.apply(1, "CHAR", 9, 1, false));
        assert_eq!(lb.phase, LB_STRIKE);
        // 1-2-2 over five starters: guest strikes 1, host 2, guest 2 -> but capped at one left
        assert!(!lb.apply(0, "STRIKE", 0, 0, false), "not host's turn");
        assert!(lb.apply(1, "STRIKE", 0, 0, false));
        assert_eq!(lb.turn, 0);
        assert_eq!(lb.left, 2);
        assert!(!lb.apply(0, "STRIKE", 5, 0, false), "a counterpick is not strikable in game 1");
        assert!(lb.apply(0, "STRIKE", 1, 0, false));
        assert!(lb.apply(0, "STRIKE", 2, 0, false));
        assert_eq!(lb.turn, 1);
        assert_eq!(lb.left, 1, "capped so one stage remains");
        assert!(lb.apply(1, "STRIKE", 3, 0, false));
        assert_eq!(lb.phase, LB_READY);
        assert_eq!(lb.chosen, 4);
        assert_eq!(lb.stage[4], LB_PICKED);
        assert!(lb.apply(0, "READY", 1, 0, false));
        assert_eq!(lb.countdown, 0);
        assert!(lb.apply(1, "READY", 1, 0, false));
        assert_eq!(lb.countdown, LB_COUNTDOWN);
        assert!(!lb.countdown_at(60));
        assert_eq!(lb.countdown, 120);
        assert!(lb.countdown_at(180));
        let _ = e;
    }

    #[test]
    fn game_two_ban_pick_counterpick() {
        let e = AllStages;
        let mut lb = fresh(0);
        lb.winner = 1;
        lb.game = 2;
        lb.reset_game(&e);
        assert_eq!(lb.phase, LB_BAN);
        assert_eq!(lb.turn, 1);
        assert!(lb.apply(1, "BAN", 0, 0, false));
        assert!(lb.apply(1, "BAN", 1, 0, false));
        assert_eq!(lb.phase, LB_PICK);
        assert_eq!(lb.turn, 0);
        assert!(!lb.apply(0, "PICK", 0, 0, false), "banned");
        assert!(lb.apply(0, "PICK", 5, 0, false));
        assert_eq!(lb.phase, LB_CHAR_WINNER);
        assert_eq!(lb.turn, 1);
        assert!(lb.apply(1, "CHAR", 20, 0, false));
        assert_eq!(lb.phase, LB_CHAR_LOSER);
        assert!(lb.apply(0, "CHAR", 0, 2, false));
        assert_eq!(lb.phase, LB_READY);
        assert!(!lb.apply(0, "READY", 1, 0, true), "blocked ready");
    }

    #[test]
    fn state_roundtrips_over_the_wire() {
        let e = AllStages;
        let mut host = fresh(0x80);
        host.apply(0, "CHAR", 2, 3, false);
        host.apply(1, "CHAR", 9, 1, false);
        host.apply(1, "STRIKE", 0, 0, false);
        host.build_list(&e);
        let mut guest = Lobby::new();
        // the list first, then the state
        let mut at = 0;
        while at < host.nstages {
            let (m, end) = host.list_encode(at, &e);
            assert!(m.len() <= crate::LOBBY_MAX);
            guest.list_rx(&m, &e);
            at = end;
        }
        let m = host.encode();
        assert!(m.len() <= crate::LOBBY_MAX, "{m}");
        assert_eq!(guest.decode(&m), 23);
        assert!(guest.list_ok);
        assert_eq!(guest.phase, host.phase);
        assert_eq!(guest.stage[..6], host.stage[..6]);
        assert_eq!(guest.ck, host.ck);
        assert_eq!(guest.color, host.color);
        assert_eq!(guest.nstages, host.nstages);
        assert_eq!(&guest.stage_ext[..6], &host.stage_ext[..6]);
        let (g, h) = (guest.encode(), m.clone());
        assert_eq!(g[..g.len() - 1], h[..h.len() - 1], "decode then encode is the identity (list_id is the host's alone)");
    }

    #[test]
    fn all_stages_mode_bans_then_picks() {
        let e = AllStages;
        let mut lb = Lobby::new();
        lb.mode = 1;
        lb.seed = 0;
        lb.build_list(&e);
        assert_eq!(lb.nstages, 32);
        assert_eq!(lb.stage_ext[..5], LB_STARTER_EXT, "starters first");
        lb.new_set(2, 9, &e);
        lb.reset_game(&e);
        assert!(lb.apply(0, "CHAR", 2, 0, false));
        assert!(lb.apply(1, "CHAR", 9, 0, false));
        assert_eq!(lb.phase, LB_BAN);
        assert_eq!(lb.turn, lb.first);
    }
}
