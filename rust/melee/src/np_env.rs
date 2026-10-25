//! This install's identities for the room / lobby layer (`melee_net::app`), and the scripted lobby
//! player. Tokens are `ck:N` / `ext:N`; the Rust sim plays Captain Falcon (0) and Fox (2) on Final
//! Destination (32); Battlefield (31) is listed so a lobby has a stage to strike.

use melee_net::app::lobby::*;
use melee_net::app::room::{Room, RoomEnv};
use melee_net::app::scene::ParsedScene;

pub struct NativeEnv;

pub fn ck_name(ck: i32) -> Option<&'static str> {
    match ck {
        0 => Some("falcon"),
        2 => Some("fox"),
        _ => None,
    }
}

impl LobbyEnv for NativeEnv {
    fn stage_available(&self, ext: i32) -> bool {
        ext == 32 || ext == 31
    }
    fn all_stages(&self) -> Vec<i32> {
        vec![31, 32]
    }
    fn peer_ready(&self) -> bool {
        true
    }
    fn token_for_ext(&self, ext: i32) -> String {
        format!("ext:{ext}")
    }
    fn ext_for_token(&self, t: &str) -> i32 {
        if let Some(h) = t.strip_prefix("id:") {
            if let Some(v) = id_map(h).and_then(|v| v.strip_prefix("ext:").and_then(|n| n.parse().ok())) {
                return v;
            }
        }
        t.strip_prefix("ext:").and_then(|n| n.parse().ok()).unwrap_or(-1)
    }
}

/// Content identities ("id:<hex>") of this install's fighters and stages, as `hex=ck:N` / `hex=ext:N`
/// pairs separated by `;` in `MELEE_NP_IDS` (the identities are hashes of disc files, so they are
/// supplied at run time, never stored in the tree).
fn id_map(hex: &str) -> Option<String> {
    let v = std::env::var("MELEE_NP_IDS").ok()?;
    v.split(';').find_map(|kv| kv.split_once('=').filter(|(k, _)| k.eq_ignore_ascii_case(hex)).map(|(_, v)| v.to_string()))
}

impl RoomEnv for NativeEnv {
    fn token_for_ck(&self, ck: i32) -> String {
        format!("ck:{ck}")
    }
    fn ck_for_token(&self, tok: &str) -> i32 {
        if let Some(h) = tok.strip_prefix("id:") {
            return id_map(h).and_then(|v| v.strip_prefix("ck:").and_then(|n| n.parse().ok())).unwrap_or(-1);
        }
        tok.strip_prefix("ck:").and_then(|n| n.parse().ok()).unwrap_or(-1)
    }
    fn global_hash(&self) -> u64 {
        std::env::var("MELEE_NP_GLOBAL_HASH").ok().and_then(|s| u64::from_str_radix(&s, 16).ok()).unwrap_or(0)
    }
    fn global_describe(&self) -> String {
        "melee (rust)".into()
    }
    fn scene_check(&self, p: &ParsedScene) -> Result<(), String> {
        for t in [&p.host_tok, &p.guest_tok] {
            if ck_name(self.ck_for_token(t)).is_none() {
                return Err(format!("fighter {t} is not playable in the Rust sim yet"));
            }
        }
        if self.ext_for_token(&p.stage_tok) != 32 {
            return Err(format!("stage {} is not playable in the Rust sim yet", p.stage_tok));
        }
        Ok(())
    }
}

/// The scripted lobby player (the C's lobby_autoplay): lock the character, strike / ban every
/// stage but Final Destination, pick it, ready up.
pub fn lobby_bot(room: &mut Room, ck: i32, host: bool) {
    let me: i32 = if host { 0 } else { 1 };
    let act = {
        let b = room.sh.borrow();
        let lb = &b.lb;
        match lb.phase {
            LB_CHAR_BLIND if !lb.locked[me as usize] => Some((0, ck)),
            LB_CHAR_WINNER | LB_CHAR_LOSER if lb.turn == me => Some((0, ck)),
            LB_STRIKE | LB_BAN if lb.turn == me => (0..lb.nstages)
                .find(|&i| lb.stage[i as usize] == LB_FREE && lb.stage_ext[i as usize] != 32 && (lb.phase == LB_BAN || lb.is_starter(i)))
                .map(|i| (1, i)),
            LB_PICK if lb.turn == me => (0..lb.nstages).find(|&i| lb.stage[i as usize] == LB_FREE && lb.stage_ext[i as usize] == 32).map(|i| (1, i)),
            LB_READY if !lb.ready[me as usize] => Some((2, 1)),
            _ => None,
        }
    };
    match act {
        Some((0, c)) => room.lobby_char(c, 0),
        Some((1, i)) => room.lobby_stage_act(i),
        Some((2, _)) => room.lobby_ready(true),
        _ => {}
    }
}
