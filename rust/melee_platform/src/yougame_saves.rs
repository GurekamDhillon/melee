//! Saves that follow the player's YouGame account (sdk.md "Saves from a native build"), over the stdio bridge.
//!
//! The files: the memory card's `.gci` files (`<MELEE_CARD_PATH>\USA\Card A|B\*.gci`), `settings.cfg`
//! (`MELEE_SETTINGS_CFG`) and `audio.cfg` (`MELEE_AUDIO_CFG`), all under YOUGAME_DATA (yougame.rs apply_data_dir).
//! Replays stay local. Account names have no path: `settings.cfg`, `audio.cfg`, `cardA-<file>.gci`, `cardB-<file>.gci`.
//!
//! 1. [`start`], after `hello` and before the game reads any of them: `saveFileList`; per file, equal sha256 is
//!    nothing to do; otherwise the note (`yougame-sync.txt`: name -> sha256 at the last sync) says which side
//!    changed, and only when both did (or there is no note) the newer wins (account `updated_at` vs local mtime).
//!    Account newer / no local copy: `saveFileRead`, base64-decode, write the local file. Local newer / nothing on
//!    the account: upload.
//! 2. A worker looks at the files once a second; a file that changed and then stayed still for 1.5 s (a burst of
//!    card writes is one save event) is uploaded with `saveFileWrite`, off the game's thread. A file the game
//!    deleted that the note knows is `saveFileDelete`d. Never on a timer when nothing changed.
//! 3. [`finish`] at exit: everything changed since the last upload, waiting for the answers.
//! Signed out (`cloud: false`) or no answer: play on the local files, nothing more.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::yougame;

struct Sync {
    data: PathBuf,
    note: Mutex<BTreeMap<String, String>>,
    stop: AtomicBool,
    /// Serialises uploads between the worker and finish().
    io: Mutex<()>,
}
static SYNC: OnceLock<Sync> = OnceLock::new();

const ANSWER: Duration = Duration::from_secs(5);
const NOTE: &str = "yougame-sync.txt";

// ---- the files ------------------------------------------------------------------------------------------------

fn env_path(k: &str, data: &Path, d: &str) -> PathBuf {
    std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| data.join(d))
}
fn card_dir(data: &Path, slot: char) -> PathBuf {
    env_path("MELEE_CARD_PATH", data, "card").join("USA").join(format!("Card {slot}"))
}

/// Whether an account name is one of ours, and where it lives locally.
pub fn local_path(data: &Path, name: &str) -> Option<PathBuf> {
    if !valid_name(name) {
        return None;
    }
    match name {
        "settings.cfg" => return Some(env_path("MELEE_SETTINGS_CFG", data, "settings.cfg")),
        "audio.cfg" => return Some(env_path("MELEE_AUDIO_CFG", data, "audio.cfg")),
        _ => {}
    }
    for slot in ['A', 'B'] {
        if let Some(f) = name.strip_prefix(&format!("card{slot}-")) {
            if f.ends_with(".gci") && f.len() > 4 {
                return Some(card_dir(data, slot).join(f));
            }
        }
    }
    None
}

/// Letters, digits, `.`, `-`, `_`, up to 120 (sdk.md "Save files").
pub fn valid_name(n: &str) -> bool {
    !n.is_empty() && n.len() <= 120 && n.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)) && !n.starts_with('.')
}

/// Every local save file now: account name -> path.
fn local_files(data: &Path) -> BTreeMap<String, PathBuf> {
    let mut m = BTreeMap::new();
    for n in ["settings.cfg", "audio.cfg"] {
        let p = local_path(data, n).unwrap();
        if p.is_file() {
            m.insert(n.to_string(), p);
        }
    }
    for slot in ['A', 'B'] {
        let Ok(rd) = std::fs::read_dir(card_dir(data, slot)) else { continue };
        for e in rd.flatten() {
            let f = e.file_name().to_string_lossy().to_string();
            let name = format!("card{slot}-{f}");
            if f.ends_with(".gci") && valid_name(&name) && e.path().is_file() {
                m.insert(name, e.path());
            }
        }
    }
    m
}

fn mtime_secs(p: &Path) -> Option<i64> {
    std::fs::metadata(p).ok()?.modified().ok()?.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs() as i64)
}

// ---- start ----------------------------------------------------------------------------------------------------

/// What [`start`] did, for the log.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
    pub cloud: bool,
    pub downloaded: Vec<String>,
    pub uploaded: Vec<String>,
    pub same: usize,
}

/// Call once, early, before the card or settings are read. Off (and `None`) without the bridge or YOUGAME_DATA.
pub fn start() -> Option<Report> {
    if !yougame::active() || std::env::var_os("MELEE_YOUGAME_SAVES").is_some_and(|v| v == "0") {
        return None;
    }
    let data = PathBuf::from(yougame::data_dir()?);
    let note = read_note(&data);
    let s = SYNC.get_or_init(|| Sync { data: data.clone(), note: Mutex::new(note), stop: AtomicBool::new(false), io: Mutex::new(()) });
    // hello first: it starts the run (sdk.md "Native builds").
    let _ = yougame::request("hello", "", ANSWER);
    let mut rep = Report::default();
    let Some(list) = yougame::request("saveFileList", "", ANSWER).and_then(|a| result(&a)) else {
        crate::plog!("yougame saves: no answer to saveFileList; playing on the local files");
        return Some(rep);
    };
    rep.cloud = list.get("cloud").and_then(Json::as_bool).unwrap_or(false);
    if !rep.cloud {
        crate::plog!("yougame saves: signed out (cloud false); playing on the local files");
        return Some(rep);
    }
    let mut cloud: BTreeMap<String, (String, Option<i64>)> = BTreeMap::new();
    for f in list.get("files").and_then(Json::as_arr).unwrap_or(&[]) {
        if let (Some(n), Some(h)) = (f.get("name").and_then(Json::as_str), f.get("sha256").and_then(Json::as_str)) {
            cloud.insert(n.to_string(), (h.to_ascii_lowercase(), f.get("updated_at").and_then(Json::as_str).and_then(parse_iso)));
        }
    }
    let local = local_files(&data);
    let _io = s.io.lock().unwrap();
    let mut names: Vec<&String> = cloud.keys().chain(local.keys()).collect();
    names.sort();
    names.dedup();
    for name in names {
        let Some(path) = local_path(&data, name) else { continue }; // not a file of ours
        let lsha = local.get(name).and_then(|p| std::fs::read(p).ok()).map(|b| sha256_hex(&b));
        let c = cloud.get(name);
        let known = s.note.lock().unwrap().get(name).cloned();
        let decision = decide(lsha.as_deref(), mtime_secs(&path), c.map(|c| c.0.as_str()), c.and_then(|c| c.1), known.as_deref());
        match decision {
            Decision::Same => {
                rep.same += 1;
                s.note.lock().unwrap().insert(name.clone(), lsha.unwrap());
            }
            Decision::Download => {
                if download(name, &path).is_some() {
                    s.note.lock().unwrap().insert(name.clone(), c.unwrap().0.clone());
                    rep.downloaded.push(name.clone());
                }
            }
            Decision::Upload => {
                if upload(s, name, &path) {
                    rep.uploaded.push(name.clone());
                }
            }
        }
    }
    write_note(&data, &s.note.lock().unwrap());
    drop(_io);
    std::thread::Builder::new().name("yougame-saves".into()).spawn(worker).ok();
    crate::plog!("yougame saves: synced at start: {} down, {} up, {} unchanged", rep.downloaded.len(), rep.uploaded.len(), rep.same);
    Some(rep)
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Same,
    Download,
    Upload,
}

/// The recipe's step 1 for one file. `note`: the sha256 both sides had at the last sync.
pub fn decide(local: Option<&str>, local_mtime: Option<i64>, cloud: Option<&str>, cloud_updated: Option<i64>, note: Option<&str>) -> Decision {
    match (local, cloud) {
        (None, None) => Decision::Same,
        (Some(_), None) => Decision::Upload,
        (None, Some(_)) => Decision::Download,
        (Some(l), Some(c)) if l.eq_ignore_ascii_case(c) => Decision::Same,
        (Some(l), Some(c)) => {
            let lch = note.map_or(true, |n| !n.eq_ignore_ascii_case(l));
            let cch = note.map_or(true, |n| !n.eq_ignore_ascii_case(c));
            match (lch, cch) {
                (false, true) => Decision::Download,
                (true, false) => Decision::Upload,
                _ => {
                    if cloud_updated.unwrap_or(i64::MIN) > local_mtime.unwrap_or(i64::MIN) {
                        Decision::Download
                    } else {
                        Decision::Upload
                    }
                }
            }
        }
    }
}

fn download(name: &str, path: &Path) -> Option<()> {
    let a = yougame::request("saveFileRead", &format!(r#""name":{}"#, yougame::json_str(name)), ANSWER)?;
    let r = result(&a)?;
    let f = r.get("file")?;
    let bytes = b64_decode(f.get("data")?.as_str()?)?;
    if let Some(h) = f.get("sha256").and_then(Json::as_str) {
        if !h.eq_ignore_ascii_case(&sha256_hex(&bytes)) {
            crate::plog!("yougame saves: {name}: the account's copy does not match its sha256; kept the local file");
            return None;
        }
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).ok()?;
    }
    let tmp = path.with_extension("yg-tmp");
    std::fs::write(&tmp, &bytes).ok()?;
    std::fs::rename(&tmp, path).ok()?;
    Some(())
}

/// Upload one file and note its sha256. Caller holds `io`.
fn upload(s: &Sync, name: &str, path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else { return false };
    let sha = sha256_hex(&bytes);
    let fields = format!(r#""name":{},"data":"{}""#, yougame::json_str(name), b64_encode(&bytes));
    let ok = yougame::request("saveFileWrite", &fields, ANSWER).and_then(|a| result(&a));
    match ok {
        Some(r) if r.get("cloud").and_then(Json::as_bool) != Some(false) => {
            let h = r.get("sha256").and_then(Json::as_str).map(str::to_ascii_lowercase).unwrap_or(sha);
            s.note.lock().unwrap().insert(name.to_string(), h);
            true
        }
        _ => {
            crate::plog!("yougame saves: {name}: upload refused or unanswered; it stays local and is tried again");
            false
        }
    }
}

// ---- save events and exit -------------------------------------------------------------------------------------

fn worker() {
    let Some(s) = SYNC.get() else { return };
    let mut seen: BTreeMap<String, (u64, Option<SystemTime>, Instant)> = BTreeMap::new();
    while !s.stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(1));
        let now = Instant::now();
        let files = local_files(&s.data);
        let mut due = Vec::new();
        for (name, p) in &files {
            let md = std::fs::metadata(p).ok();
            let key = (md.as_ref().map_or(0, |m| m.len()), md.and_then(|m| m.modified().ok()));
            let e = seen.entry(name.clone()).or_insert((key.0, key.1, now));
            if (e.0, e.1) != key {
                *e = (key.0, key.1, now);
            } else if now.duration_since(e.2) >= Duration::from_millis(1500) {
                due.push(name.clone());
            }
        }
        if s.stop.load(Ordering::SeqCst) {
            break;
        }
        sync_changed(s, &files, Some(&due));
    }
}

/// Upload what differs from the note (only `due` names when given) and delete what the game deleted.
fn sync_changed(s: &Sync, files: &BTreeMap<String, PathBuf>, due: Option<&[String]>) -> usize {
    let _io = s.io.lock().unwrap();
    let mut n = 0;
    for (name, p) in files {
        if due.is_some_and(|d| !d.contains(name)) {
            continue;
        }
        let Ok(b) = std::fs::read(p) else { continue };
        if s.note.lock().unwrap().get(name).is_some_and(|h| *h == sha256_hex(&b)) {
            continue;
        }
        if upload(s, name, p) {
            n += 1;
        }
    }
    let gone: Vec<String> = s.note.lock().unwrap().keys().filter(|k| !files.contains_key(*k)).cloned().collect();
    for name in gone {
        let a = yougame::request("saveFileDelete", &format!(r#""name":{}"#, yougame::json_str(&name)), ANSWER);
        if a.and_then(|a| result(&a)).is_some() {
            s.note.lock().unwrap().remove(&name);
            n += 1;
        }
    }
    if n > 0 {
        write_note(&s.data, &s.note.lock().unwrap());
    }
    n
}

/// At exit: upload everything changed since the last sync and wait for the answers (the bridge dies with us).
pub fn finish(_budget: Duration) {
    let Some(s) = SYNC.get() else { return };
    if s.stop.swap(true, Ordering::SeqCst) {
        return;
    }
    let n = sync_changed(s, &local_files(&s.data), None);
    crate::plog!("yougame saves: {n} change(s) sent at exit");
}

fn read_note(data: &Path) -> BTreeMap<String, String> {
    let t = std::fs::read_to_string(data.join(NOTE)).unwrap_or_default();
    t.lines().filter_map(|l| l.split_once(' ')).map(|(n, h)| (n.to_string(), h.trim().to_string())).collect()
}
fn write_note(data: &Path, m: &BTreeMap<String, String>) {
    let t: String = m.iter().map(|(n, h)| format!("{n} {h}\n")).collect();
    let tmp = data.join("yougame-sync.tmp");
    if std::fs::write(&tmp, t).is_ok() {
        let _ = std::fs::rename(&tmp, data.join(NOTE));
    }
}

// ---- answers: a small JSON reader -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}
impl Json {
    pub fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(v) => v.iter().find(|(a, _)| a == k).map(|(_, b)| b),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Json::Str(s) = self { Some(s) } else { None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let Json::Bool(b) = self { Some(*b) } else { None }
    }
    pub fn as_arr(&self) -> Option<&[Json]> {
        if let Json::Arr(a) = self { Some(a) } else { None }
    }
    pub fn parse(t: &str) -> Option<Json> {
        let b = t.as_bytes();
        let mut i = 0;
        let v = value(b, &mut i, 0)?;
        ws(b, &mut i);
        (i == b.len()).then_some(v)
    }
}
fn ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && b[*i].is_ascii_whitespace() {
        *i += 1;
    }
}
fn value(b: &[u8], i: &mut usize, depth: u32) -> Option<Json> {
    if depth > 32 {
        return None;
    }
    ws(b, i);
    match *b.get(*i)? {
        b'{' => {
            *i += 1;
            let mut v = Vec::new();
            ws(b, i);
            if b.get(*i) == Some(&b'}') {
                *i += 1;
                return Some(Json::Obj(v));
            }
            loop {
                ws(b, i);
                let Json::Str(k) = string(b, i)? else { return None };
                ws(b, i);
                (b.get(*i) == Some(&b':')).then_some(())?;
                *i += 1;
                v.push((k, value(b, i, depth + 1)?));
                ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b'}' => {
                        *i += 1;
                        return Some(Json::Obj(v));
                    }
                    _ => return None,
                }
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            ws(b, i);
            if b.get(*i) == Some(&b']') {
                *i += 1;
                return Some(Json::Arr(v));
            }
            loop {
                v.push(value(b, i, depth + 1)?);
                ws(b, i);
                match b.get(*i)? {
                    b',' => *i += 1,
                    b']' => {
                        *i += 1;
                        return Some(Json::Arr(v));
                    }
                    _ => return None,
                }
            }
        }
        b'"' => string(b, i),
        b't' if b[*i..].starts_with(b"true") => {
            *i += 4;
            Some(Json::Bool(true))
        }
        b'f' if b[*i..].starts_with(b"false") => {
            *i += 5;
            Some(Json::Bool(false))
        }
        b'n' if b[*i..].starts_with(b"null") => {
            *i += 4;
            Some(Json::Null)
        }
        _ => {
            let st = *i;
            while *i < b.len() && (b[*i].is_ascii_digit() || b"+-.eE".contains(&b[*i])) {
                *i += 1;
            }
            std::str::from_utf8(&b[st..*i]).ok()?.parse().ok().map(Json::Num)
        }
    }
}
fn string(b: &[u8], i: &mut usize) -> Option<Json> {
    (b.get(*i) == Some(&b'"')).then_some(())?;
    *i += 1;
    let mut o: Vec<u8> = Vec::new();
    loop {
        let c = *b.get(*i)?;
        *i += 1;
        match c {
            b'"' => return String::from_utf8(o).ok().map(Json::Str),
            b'\\' => {
                let e = *b.get(*i)?;
                *i += 1;
                match e {
                    b'n' => o.push(b'\n'),
                    b't' => o.push(b'\t'),
                    b'r' => o.push(b'\r'),
                    b'b' => o.push(8),
                    b'f' => o.push(12),
                    b'u' => {
                        let h = std::str::from_utf8(b.get(*i..*i + 4)?).ok()?;
                        *i += 4;
                        let ch = char::from_u32(u32::from_str_radix(h, 16).ok()?).unwrap_or('\u{fffd}');
                        o.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                    }
                    c => o.push(c),
                }
            }
            c => o.push(c),
        }
    }
}

/// An answer's `result` when `ok` is true.
pub fn result(answer: &str) -> Option<Json> {
    let j = Json::parse(answer)?;
    if j.get("ok").and_then(Json::as_bool) != Some(true) {
        return None;
    }
    j.get("result").cloned()
}

/// `2026-11-08T12:34:56.789Z` (or `+hh:mm`) as Unix seconds.
pub fn parse_iso(t: &str) -> Option<i64> {
    let b = t.as_bytes();
    let n = |a: usize, l: usize| -> Option<i64> { std::str::from_utf8(b.get(a..a + l)?).ok()?.parse().ok() };
    let (y, mo, d, h, mi, s) = (n(0, 4)?, n(5, 2)?, n(8, 2)?, n(11, 2)?, n(14, 2)?, n(17, 2)?);
    // days from civil (Howard Hinnant)
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * (mo + if mo > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    let mut secs = days * 86400 + h * 3600 + mi * 60 + s;
    let tail = &t[19.min(t.len())..];
    let tz = tail.trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    if let Some(sign @ ('+' | '-')) = tz.chars().next() {
        let tb = tz.as_bytes();
        let hh: i64 = std::str::from_utf8(tb.get(1..3)?).ok()?.parse().ok()?;
        let mm: i64 = std::str::from_utf8(tb.get(4..6)?).ok()?.parse().ok()?;
        let off = hh * 3600 + mm * 60;
        secs -= if sign == '+' { off } else { -off };
    }
    Some(secs)
}

// ---- base64, sha256 -------------------------------------------------------------------------------------------

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
pub fn b64_encode(d: &[u8]) -> String {
    let mut o = String::with_capacity(d.len().div_ceil(3) * 4);
    for c in d.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        o.push(B64[(n >> 18) as usize & 63] as char);
        o.push(B64[(n >> 12) as usize & 63] as char);
        o.push(if c.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        o.push(if c.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    o
}
pub fn b64_decode(t: &str) -> Option<Vec<u8>> {
    let mut o = Vec::with_capacity(t.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0);
    for &c in t.as_bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => return None,
        } as u32;
        acc = acc << 6 | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            o.push((acc >> bits) as u8);
        }
    }
    Some(o)
}

pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut m = data.to_vec();
    m.push(0x80);
    while m.len() % 64 != 56 {
        m.push(0);
    }
    m.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for blk in m.chunks(64) {
        let mut w = [0u32; 64];
        for t in 0..16 {
            w[t] = u32::from_be_bytes([blk[t * 4], blk[t * 4 + 1], blk[t * 4 + 2], blk[t * 4 + 3]]);
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16].wrapping_add(s0).wrapping_add(w[t - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for t in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[t]).wrapping_add(w[t]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
        }
        for k in 0..8 {
            h[k] = h[k].wrapping_add(v[k]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primitives() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        for d in [&b""[..], b"f", b"fo", b"foo", b"foob", b"fooba", b"foobar"] {
            assert_eq!(b64_decode(&b64_encode(d)).unwrap(), d);
        }
        assert_eq!(b64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(parse_iso("1970-01-02T00:00:00Z"), Some(86400));
        assert_eq!(parse_iso("2026-11-08T12:00:00.123+01:00"), parse_iso("2026-11-08T11:00:00Z"));
        let j = Json::parse(r#"{"re":3,"ok":true,"result":{"files":[{"name":"aA","bytes":2}],"cloud":true}}"#).unwrap();
        assert_eq!(j.get("result").unwrap().get("files").unwrap().as_arr().unwrap()[0].get("name").unwrap().as_str(), Some("aA"));
    }
    #[test]
    fn decisions() {
        use Decision::*;
        assert_eq!(decide(Some("a"), Some(5), Some("a"), Some(9), None), Same);
        assert_eq!(decide(None, None, Some("c"), Some(1), None), Download);
        assert_eq!(decide(Some("l"), Some(1), None, None, None), Upload);
        // only the account's copy changed since the last sync: it wins even when older
        assert_eq!(decide(Some("n"), Some(100), Some("c"), Some(1), Some("n")), Download);
        assert_eq!(decide(Some("l"), Some(1), Some("n"), Some(100), Some("n")), Upload);
        // both changed / no note: newer wins
        assert_eq!(decide(Some("l"), Some(1), Some("c"), Some(100), Some("x")), Download);
        assert_eq!(decide(Some("l"), Some(100), Some("c"), Some(1), None), Upload);
    }
    #[test]
    fn names() {
        let d = Path::new("D");
        assert!(local_path(d, "settings.cfg").is_some());
        assert!(local_path(d, "cardA-01-GALE-x.gci").unwrap().ends_with("Card A/01-GALE-x.gci") || local_path(d, "cardA-01-GALE-x.gci").unwrap().ends_with(r"Card A\01-GALE-x.gci"));
        assert!(local_path(d, "../x").is_none());
        assert!(local_path(d, "cardA-x.txt").is_none());
        assert!(!valid_name("a/b"));
    }
}
