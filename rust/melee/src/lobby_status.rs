//! `--lobby-status-file <json>` (launcher_compat.rs sets `MELEE_LOBBY_STATUS_FILE`): the Melee Unlocked launcher's
//! status mailbox, `lobby-game-status.json` beside it. Same schema as his Source Port (host.cpp
//! `publish_lobby_status`): `{"status":"In match"|"In game","stocks":[n]}`, stocks only in a match. It is written at
//! most once a second and only when the text changes, by a thread of its own (temp file, then a rename over), so a slow
//! disk never holds the simulation.
//!
//! One difference: his port says "In match" only for an online match. Ours also says it in an offline VS match (the
//! scenes `vs` and `suddendeath`), which is what the lobby's presence line shows a friend. Stocks are port 1's.
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

struct Writer {
    tx: Sender<String>,
    last_sent: Option<String>,
    last_at: Option<Instant>,
}

static WRITER: OnceLock<Option<Mutex<Writer>>> = OnceLock::new();

/// The file's text for a scene kind and port 1's stock count.
pub fn text(scene_kind: u8, stocks: i32) -> String {
    let in_match = matches!(scene_kind, 2 | 3); // vs, suddendeath (retail_flow::scene_name)
    let stocks = if in_match && (0..=99).contains(&stocks) { stocks.to_string() } else { String::new() };
    format!("{{\"status\":\"{}\",\"stocks\":[{stocks}]}}", if in_match { "In match" } else { "In game" })
}

fn start(path: std::path::PathBuf) -> Mutex<Writer> {
    let (tx, rx) = channel::<String>();
    std::thread::Builder::new()
        .name("lobby-status".into())
        .spawn(move || {
            let tmp = path.with_extension("json.tmp");
            while let Ok(mut t) = rx.recv() {
                while let Ok(n) = rx.try_recv() {
                    t = n;
                }
                if std::fs::write(&tmp, &t).is_ok() {
                    let _ = std::fs::rename(&tmp, &path);
                }
            }
        })
        .ok();
    Mutex::new(Writer { tx, last_sent: None, last_at: None })
}

/// Called once per tick by the retail flow with the current scene kind. Nothing without `MELEE_LOBBY_STATUS_FILE`.
pub fn frame(scene_kind: u8) {
    // launcher tests only (MELEE_LAUNCHER_TEST): a panic in the first match tick, for the crash files the launcher reads
    if matches!(scene_kind, 2 | 3) && std::env::var_os("MELEE_LAUNCHER_TEST").is_some() && std::env::var("MELEE_CRASH_TEST_IN_MATCH").is_ok_and(|v| v == "1") {
        panic!("MELEE_CRASH_TEST_IN_MATCH: forced panic in the match");
    }
    let w = WRITER.get_or_init(|| std::env::var_os("MELEE_LOBBY_STATUS_FILE").filter(|p| !p.is_empty()).map(|p| start(p.into())));
    let Some(w) = w else { return };
    let Ok(mut w) = w.lock() else { return };
    let now = Instant::now();
    if w.last_at.is_some_and(|t| now - t < Duration::from_secs(1)) {
        return;
    }
    w.last_at = Some(now);
    let stocks = if matches!(scene_kind, 2 | 3) { unsafe { melee_fighter::tu::player::Player_GetStocks(0) } } else { -1 };
    let t = text(scene_kind, stocks);
    if w.last_sent.as_deref() != Some(t.as_str()) {
        let _ = w.tx.send(t.clone());
        w.last_sent = Some(t);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn schema_matches_the_source_ports() {
        assert_eq!(super::text(8, -1), r#"{"status":"In game","stocks":[]}"#);
        assert_eq!(super::text(2, 4), r#"{"status":"In match","stocks":[4]}"#);
        assert_eq!(super::text(3, 1), r#"{"status":"In match","stocks":[1]}"#);
        assert_eq!(super::text(2, -3), r#"{"status":"In match","stocks":[]}"#);
    }
}
