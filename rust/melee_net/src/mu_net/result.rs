//! The signed result file of one game (plan section 2.11): `{"body":{...},"identity":"<hex>","signature":"<hex>"}`,
//! the body in a fixed key order with no optional whitespace, signed with the player's identity over
//! `"MeleeUnlockedNet1 result"` and the body's exact bytes. His launcher reads `has_outcome`, `desync`,
//! `disconnected`, `local_slot`, `winner` and `players[].stocks` from it for its match history.

use super::crypto::{hex, verify, Bytes32, Identity};
use super::desc::{PROTOCOL_VERSION, ROUTE_UNRANKED};

const DOMAIN: &[u8] = b"MeleeUnlockedNet1 result";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerResult {
    pub slot_type: u8,
    pub stocks: u8,
    pub damage_done: f32,
    pub synced_stocks: u8,
    pub synced_damage: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GameResult {
    pub route: u8,
    pub build_id: String,
    pub descriptor_digest: Bytes32,
    pub match_id: [u8; 16],
    pub local_slot: u8,
    pub peer_identity: Bytes32,
    pub transcript_digest: Bytes32,
    pub transcript_frames: i32,
    pub transcript_complete: bool,
    /// Some(true) the peer's RESULT digest equals ours, Some(false) it differs, None not compared
    pub peer_agreement: Option<bool>,
    pub desync: bool,
    pub disconnected: bool,
    pub has_outcome: bool,
    pub mode: u8,
    pub frame_length: u32,
    pub game_index: u32,
    pub tiebreak_index: u32,
    /// 0 or 1, -1 none
    pub winner: i32,
    pub end_method: u8,
    pub lras_initiator: i32,
    pub synced_timer: u32,
    pub players: [PlayerResult; 4],
    pub game_info_hash: Bytes32,
}

impl Default for GameResult {
    fn default() -> Self {
        GameResult {
            route: 1,
            build_id: String::new(),
            descriptor_digest: [0; 32],
            match_id: [0; 16],
            local_slot: 0,
            peer_identity: [0; 32],
            transcript_digest: [0; 32],
            transcript_frames: 0,
            transcript_complete: true,
            peer_agreement: None,
            desync: false,
            disconnected: false,
            has_outcome: false,
            mode: 0,
            frame_length: 0,
            game_index: 0,
            tiebreak_index: 0,
            winner: -1,
            end_method: 0,
            lras_initiator: -1,
            synced_timer: 0,
            players: Default::default(),
            game_info_hash: [0; 32],
        }
    }
}

fn quoted(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                o.push('\\');
                o.push(c);
            }
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

impl GameResult {
    /// The body object exactly as it is signed and stored.
    pub fn body(&self) -> String {
        let b = |v: bool| if v { "true" } else { "false" }.to_string();
        let h = |v: &[u8]| format!("\"{}\"", hex(v));
        let players: Vec<String> = self
            .players
            .iter()
            .map(|p| {
                // thousandths as integers (never a locale's decimal comma)
                let milli = (p.damage_done.clamp(0.0, 1e9) as f64 * 1000.0).round() as i64;
                format!(
                    "{{\"slot_type\":{},\"stocks\":{},\"damage_done\":{}.{:03},\"synced_stocks\":{},\"synced_damage\":{}}}",
                    p.slot_type,
                    p.stocks,
                    milli / 1000,
                    milli % 1000,
                    p.synced_stocks,
                    p.synced_damage
                )
            })
            .collect();
        let fields: [(&str, String); 25] = [
            ("format", "1".into()),
            ("protocol", PROTOCOL_VERSION.to_string()),
            ("route", quoted(if self.route == ROUTE_UNRANKED { "p2p_unranked" } else { "p2p_direct" })),
            ("build_id", quoted(&self.build_id)),
            ("descriptor_digest", h(&self.descriptor_digest)),
            ("match_id", h(&self.match_id)),
            ("local_slot", self.local_slot.to_string()),
            ("peer_identity", h(&self.peer_identity)),
            ("transcript_digest", h(&self.transcript_digest)),
            ("transcript_frames", self.transcript_frames.to_string()),
            ("transcript_complete", b(self.transcript_complete)),
            ("peer_agreement", quoted(match self.peer_agreement { Some(true) => "match", Some(false) => "mismatch", None => "unknown" })),
            ("desync", b(self.desync)),
            ("disconnected", b(self.disconnected)),
            ("has_outcome", b(self.has_outcome)),
            ("mode", self.mode.to_string()),
            ("frame_length", self.frame_length.to_string()),
            ("game_index", self.game_index.to_string()),
            ("tiebreak_index", self.tiebreak_index.to_string()),
            ("winner", self.winner.to_string()),
            ("end_method", self.end_method.to_string()),
            ("lras_initiator", self.lras_initiator.to_string()),
            ("synced_timer", self.synced_timer.to_string()),
            ("players", format!("[{}]", players.join(","))),
            ("game_info_hash", h(&self.game_info_hash)),
        ];
        let parts: Vec<String> = fields.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
        format!("{{{}}}", parts.join(","))
    }

    /// The whole file's text, signed by `id`.
    pub fn file_text(&self, id: &Identity) -> String {
        let body = self.body();
        let sig = id.sign(&[DOMAIN, body.as_bytes()]);
        format!("{{\"body\":{body},\"identity\":\"{}\",\"signature\":\"{}\"}}\n", hex(&id.public_key()), hex(&sig))
    }

    /// Writes the file through a `.part` file renamed into place.
    pub fn write(&self, id: &Identity, path: &str) -> Result<(), String> {
        let p = std::path::Path::new(path);
        if let Some(d) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
            let _ = std::fs::create_dir_all(d);
        }
        let mut tmp = p.as_os_str().to_owned();
        tmp.push(".part");
        std::fs::write(&tmp, self.file_text(id)).map_err(|e| format!("{path}: {e}"))?;
        std::fs::rename(&tmp, p).map_err(|e| format!("{path}: {e}"))
    }
}

/// The file name of game `game` of a session: the name given for the first, `<stem>.g<n><ext>` after it.
pub fn path_for_game(base: &str, game: u32) -> String {
    if game <= 1 {
        return base.to_string();
    }
    let slash = base.rfind(['/', '\\']);
    match base.rfind('.') {
        Some(dot) if slash.is_none_or(|s| dot > s) => format!("{}.g{game}{}", &base[..dot], &base[dot..]),
        _ => format!("{base}.g{game}"),
    }
}

/// Checks a result file's signature (tests; the launcher's own reader does not).
pub fn verify_file(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("{\"body\":") else { return false };
    let Some(i) = rest.rfind(",\"identity\":\"") else { return false };
    let body = &rest[..i];
    let tail = &rest[i + 13..];
    let (mut id, mut sig) = ([0u8; 32], [0u8; 64]);
    tail.len() >= 64 + 15 + 128
        && super::crypto::unhex(&tail[..64], &mut id)
        && tail[64..].starts_with("\",\"signature\":\"")
        && super::crypto::unhex(&tail[79..79 + 128], &mut sig)
        && verify(&id, &[DOMAIN, body.as_bytes()], &sig)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_order_and_format() {
        let mut r = GameResult { build_id: "b\"x".into(), local_slot: 1, has_outcome: true, winner: 1, transcript_frames: 600, peer_agreement: Some(true), ..Default::default() };
        r.players[0] = PlayerResult { slot_type: 0, stocks: 2, damage_done: 57.25, ..Default::default() };
        let b = r.body();
        assert!(b.starts_with("{\"format\":1,\"protocol\":1,\"route\":\"p2p_direct\",\"build_id\":\"b\\\"x\",\"descriptor_digest\":\"0000"), "{b}");
        assert!(b.contains(",\"local_slot\":1,\"peer_identity\":\""));
        assert!(b.contains(",\"transcript_frames\":600,\"transcript_complete\":true,\"peer_agreement\":\"match\",\"desync\":false,\"disconnected\":false,\"has_outcome\":true,\"mode\":0,"));
        assert!(b.contains(",\"winner\":1,\"end_method\":0,\"lras_initiator\":-1,\"synced_timer\":0,\"players\":[{\"slot_type\":0,\"stocks\":2,\"damage_done\":57.250,\"synced_stocks\":0,\"synced_damage\":0},"));
        assert!(b.ends_with("],\"game_info_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\"}"));
        assert!(!b.contains(' '));
        let id = Identity::from_seed(&[9; 32]);
        let t = r.file_text(&id);
        assert!(t.ends_with("\"}\n"));
        assert!(verify_file(t.trim_end()));
        assert!(!verify_file(&t.replace("\"winner\":1", "\"winner\":0")));
    }

    /// `MU_RESULT_CHECK=<result file written by another implementation>`: rebuild the body from the file's values
    /// with our writer and compare bytes (M8: his reference writer against ours). Ignored without the variable.
    #[test]
    #[ignore]
    fn body_bytes_match_a_reference_file() {
        let Ok(path) = std::env::var("MU_RESULT_CHECK") else { return };
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(verify_file(text.trim_end()), "the reference file's signature verifies with our reader");
        let body = &text["{\"body\":".len()..text.rfind(",\"identity\":\"").unwrap()];
        // the values in order of appearance: "key":value (strings unquoted)
        let val = |key: &str, nth: usize| -> String {
            let pat = format!("\"{key}\":");
            let at = body.match_indices(&pat).nth(nth).unwrap().0 + pat.len();
            let rest = &body[at..];
            if let Some(r) = rest.strip_prefix('"') {
                r[..r.find('"').unwrap()].to_string()
            } else {
                rest[..rest.find([',', '}']).unwrap()].to_string()
            }
        };
        let hx = |key: &str, out: &mut [u8]| assert!(super::super::crypto::unhex(&val(key, 0), out));
        let mut r = GameResult { build_id: val("build_id", 0), ..Default::default() };
        r.route = if val("route", 0) == "p2p_unranked" { ROUTE_UNRANKED } else { 1 };
        hx("descriptor_digest", &mut r.descriptor_digest);
        hx("match_id", &mut r.match_id);
        hx("peer_identity", &mut r.peer_identity);
        hx("transcript_digest", &mut r.transcript_digest);
        hx("game_info_hash", &mut r.game_info_hash);
        r.local_slot = val("local_slot", 0).parse().unwrap();
        r.transcript_frames = val("transcript_frames", 0).parse().unwrap();
        r.transcript_complete = val("transcript_complete", 0) == "true";
        r.peer_agreement = match val("peer_agreement", 0).as_str() {
            "match" => Some(true),
            "mismatch" => Some(false),
            _ => None,
        };
        r.desync = val("desync", 0) == "true";
        r.disconnected = val("disconnected", 0) == "true";
        r.has_outcome = val("has_outcome", 0) == "true";
        r.mode = val("mode", 0).parse().unwrap();
        r.frame_length = val("frame_length", 0).parse().unwrap();
        r.game_index = val("game_index", 0).parse().unwrap();
        r.tiebreak_index = val("tiebreak_index", 0).parse().unwrap();
        r.winner = val("winner", 0).parse().unwrap();
        r.end_method = val("end_method", 0).parse().unwrap();
        r.lras_initiator = val("lras_initiator", 0).parse().unwrap();
        r.synced_timer = val("synced_timer", 0).parse().unwrap();
        for (i, p) in r.players.iter_mut().enumerate() {
            *p = PlayerResult {
                slot_type: val("slot_type", i).parse().unwrap(),
                stocks: val("stocks", i).parse().unwrap(),
                damage_done: val("damage_done", i).parse().unwrap(),
                synced_stocks: val("synced_stocks", i).parse().unwrap(),
                synced_damage: val("synced_damage", i).parse().unwrap(),
            };
        }
        assert_eq!(r.body(), body);
        eprintln!("result body: our writer reproduces {path} byte for byte ({} bytes)", body.len());
    }

    #[test]
    fn game_file_names() {
        assert_eq!(path_for_game("C:\\L\\p2p-results\\ab.json", 1), "C:\\L\\p2p-results\\ab.json");
        assert_eq!(path_for_game("C:\\L\\p2p-results\\ab.json", 3), "C:\\L\\p2p-results\\ab.g3.json");
        assert_eq!(path_for_game("/a.b/res", 2), "/a.b/res.g2");
    }
}
