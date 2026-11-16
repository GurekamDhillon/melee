//! The in-game bracket client: a native host service for the autobracket server (github.com/GurekamDhillon/
//! melee-autobracket, docs/api.md), shown by the bundled `rust/mods/bracket` presentation mod through gd-ui-v1.4
//! `UI_HOST` catalogs and requests (docs/rust-port/notes/wasm-ui-lab-api.md A6.2).
//!
//! * Identity: an Ed25519 key made on first use, kept in `<config dir>/bracket/client.key` (the folder of
//!   settings.cfg), user-only (0600 on Unix; on Windows the file's ACL is cut to the current user). Only the worker
//!   thread reads it; it is never given to a mod and never logged. The UI shows a fingerprint (the public key's first
//!   16 hex digits), which identifies but cannot sign.
//! * Signing: `autobracket_proto` (vendored from the server repo's `proto::sign`, byte-equal, tests there).
//! * Transport: `http.rs`, one pinned origin (`https://bracket.gsd.sh`, or `MELEE_BRACKET_SERVER`), Nucleus style.
//! * Auto-upload (setting `bracket_upload=1`, default off): every .slp the exporter writes for a netplay game goes into
//!   `<config dir>/bracket/inbox/` (one small file per replay, so a child process, such as the netplay run's headless
//!   .slp playback, can add to it without sharing a lock). The worker moves the inbox into `queue.tsv` (rewritten
//!   atomically: a sibling .tmp, then a rename), asks `GET /v1/me/next?event=<bracket_event>` which game of which set
//!   is being played, and uploads with the phase-1 route `POST /v1/sets/{event}/{set}/games/{n}/replay` (header
//!   `x-ab-build`). Transport failures, 408, 429 and 5xx retry after 5, 15, 45, 135, then every 300 s (or Retry-After);
//!   another 4xx fails the item with the server's message. The queue survives restarts.
//! * Threads: one worker. The game thread only `try_lock`s the shared state for the lists (a frame where the worker
//!   holds it shows the previous lists) and posts requests; it never waits on the network.
//!
//! Not here yet (the server's (b) API): linking an account to the key (BRACKET_LINK), my sets, ready, join. Those
//! requests answer "not available yet" (see `stub`).

pub mod http;
pub mod mock;

use autobracket_proto as proto;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub const DEFAULT_SERVER: &str = "https://bracket.gsd.sh";
const MAX_REPLAY: usize = 16 << 20;
const BACKOFF: [i64; 5] = [5, 15, 45, 135, 300];
const KEEP_DONE: usize = 20;

fn log(m: &str) {
    eprintln!("bracket: {m}");
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// `<folder of settings.cfg>/bracket` (`MELEE_BRACKET_DIR` for tests).
pub fn dir() -> PathBuf {
    if let Some(d) = std::env::var_os("MELEE_BRACKET_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(d);
    }
    let cfg = melee_data::settings::default_path();
    cfg.parent().map_or_else(|| PathBuf::from("bracket"), |p| p.join("bracket"))
}

/// The configured server's origin, or the reason it is refused.
pub fn origin() -> Result<http::Origin, String> {
    let s = std::env::var("MELEE_BRACKET_SERVER").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| DEFAULT_SERVER.into());
    http::Origin::parse(&s).ok_or_else(|| format!("MELEE_BRACKET_SERVER {s:?} is not an https origin (plain http only to 127.0.0.1 / localhost)"))
}

/// Setting `bracket_upload` (MELEE_BRACKET_UPLOAD): auto-upload netplay replays. Off unless set.
pub fn auto_upload() -> bool {
    crate::cfg::flag("bracket_upload", false)
}

/// Setting `bracket_event` (MELEE_BRACKET_EVENT): the event the uploads are for (until linking picks it).
pub fn event() -> Option<i64> {
    crate::cfg::text("bracket_event").and_then(|s| s.trim().parse().ok()).filter(|&e: &i64| e >= 0)
}

/// The slp exporter wrote `path`: queue it for upload when it is a netplay game and auto-upload is on. Called at the
/// end of a match (one small file write; no network).
pub fn replay_written(path: &str, played_on: &str) {
    if played_on != "network" || !auto_upload() {
        return;
    }
    let full = std::fs::canonicalize(path).map_or_else(|_| PathBuf::from(path), |p| p);
    let inbox = dir().join("inbox");
    let _ = std::fs::create_dir_all(&inbox);
    let name = format!("{}-{}-{}.txt", now(), std::process::id(), INBOX_N.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let tmp = inbox.join(format!("{name}.tmp"));
    let full = full.to_string_lossy().trim_start_matches(r"\\?\").to_string();
    if std::fs::write(&tmp, full.as_bytes()).and_then(|_| std::fs::rename(&tmp, inbox.join(&name))).is_ok() {
        log(&format!("queued {} for upload", Path::new(&full).file_name().map_or(String::new(), |f| f.to_string_lossy().into_owned())));
    }
}
static INBOX_N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

// ------------------------------------------------------------------------------------------------------- the key

/// Load the key, or make one (first use). The file holds the 32-byte seed as hex.
fn load_or_make_key(dir: &Path) -> Result<proto::SigningKey, String> {
    let p = dir.join("client.key");
    if let Ok(t) = std::fs::read_to_string(&p) {
        return proto::unhex::<32>(t.trim()).map(|s| proto::SigningKey::from_bytes(&s)).ok_or_else(|| "client.key is damaged (delete it to make a new key)".to_string());
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let mut seed = [0u8; 32];
    getrandom(&mut seed)?;
    let sk = proto::SigningKey::from_bytes(&seed);
    let tmp = dir.join("client.key.tmp");
    write_private(&tmp, proto::hex(&seed).as_bytes()).map_err(|e| format!("cannot write the key: {e}"))?;
    seed.fill(0);
    std::fs::rename(&tmp, &p).map_err(|e| format!("cannot write the key: {e}"))?;
    log(&format!("made a new client key (fingerprint {})", fingerprint(&sk)));
    Ok(sk)
}

fn getrandom(b: &mut [u8; 32]) -> Result<(), String> {
    let (a, c) = (proto::fresh_nonce(), proto::fresh_nonce());
    b[..16].copy_from_slice(&a);
    b[16..].copy_from_slice(&c);
    Ok(())
}

/// A file only this user can read: mode 0600 on Unix; on Windows the inherited ACL is replaced by one entry for the
/// current user (icacls, once, at key creation).
fn write_private(p: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    let mut f = o.open(p)?;
    f.write_all(data)?;
    f.sync_all()?;
    drop(f);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if let Ok(user) = std::env::var("USERNAME") {
            let st = std::process::Command::new("icacls").arg(p).args(["/inheritance:r", "/grant:r"]).arg(format!("{user}:F")).creation_flags(0x0800_0000).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
            if !st.is_ok_and(|s| s.success()) {
                log("could not restrict the key file's permissions (icacls)");
            }
        }
    }
    Ok(())
}

pub fn fingerprint(sk: &proto::SigningKey) -> String {
    let h = proto::hex(&sk.verifying_key().to_bytes());
    format!("{} {} {} {}", &h[0..4], &h[4..8], &h[8..12], &h[12..16])
}

/// The build string the server records with each upload (crates/client `engine_build`): the first line of
/// `melee version`, `+`, the first 12 hex digits of the executable's SHA-256.
fn engine_build() -> String {
    let exe = std::env::current_exe().ok();
    let line = exe.as_ref().and_then(|e| std::fs::read_to_string(e.with_file_name("version.txt")).ok()).and_then(|t| t.lines().next().map(str::to_string));
    let line = format!("melee (rust) {} {}", melee_core::VERSION, line.unwrap_or_else(|| "development build".into()));
    let h = exe.and_then(|e| std::fs::read(e).ok()).map(|b| proto::hex(&proto::sha256(&b)[..6])).unwrap_or_default();
    format!("{line}+{h}")
}

// ------------------------------------------------------------------------------------------------------ the queue

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum St {
    Queued,
    Uploading,
    Done,
    Failed,
}
const ST_NAMES: [&str; 4] = ["queued", "uploading", "done", "failed"];
impl St {
    pub fn name(self) -> &'static str {
        ST_NAMES[self as usize]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub file: String,
    pub state: St,
    pub attempts: u32,
    pub next_try: i64,
    /// event, set, game: decided once (from `/v1/me/next`), then kept for every retry
    pub target: Option<(i64, u32, u32)>,
    pub reason: String,
}

fn clean(s: &str) -> String {
    s.chars().map(|c| if c == '\t' || c.is_control() { ' ' } else { c }).take(160).collect()
}

pub fn queue_to_text(q: &[Item]) -> String {
    let mut s = String::from("# bracket upload queue v1: state attempts next_try event set game file reason\n");
    for t in q {
        let (e, st, g) = t.target.map_or(("-".into(), "-".into(), "-".into()), |(e, s, g)| (e.to_string(), s.to_string(), g.to_string()));
        s.push_str(&format!("{}\t{}\t{}\t{e}\t{st}\t{g}\t{}\t{}\n", t.state.name(), t.attempts, t.next_try, clean(&t.file), clean(&t.reason)));
    }
    s
}

/// The saved queue: an upload that was running is queued again; done ones are dropped.
pub fn queue_from_text(s: &str) -> Vec<Item> {
    let mut v = Vec::new();
    for l in s.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() < 8 {
            continue;
        }
        let state = match f[0] {
            "queued" | "uploading" => St::Queued,
            "failed" => St::Failed,
            _ => continue,
        };
        let target = match (f[3].parse(), f[4].parse(), f[5].parse()) {
            (Ok(e), Ok(s), Ok(g)) => Some((e, s, g)),
            _ => None,
        };
        v.push(Item { file: f[6].into(), state, attempts: f[1].parse().unwrap_or(0), next_try: f[2].parse().unwrap_or(0), target, reason: f[7].into() });
    }
    v
}

fn save_queue(dir: &Path, q: &[Item]) {
    let _ = std::fs::create_dir_all(dir);
    let tmp = dir.join("queue.tsv.tmp");
    if std::fs::write(&tmp, queue_to_text(q)).and_then(|_| std::fs::rename(&tmp, dir.join("queue.tsv"))).is_err() {
        log("could not save the upload queue");
    }
}

// ---------------------------------------------------------------------------------------------------- the service

/// What the UI shows (A6.2 BRACKET_STATE): one record.
#[derive(Clone, Debug, Default)]
pub struct Status {
    pub fingerprint: String,
    pub linked: bool,
    /// 0 not asked yet, 1 reachable, 2 not reachable
    pub reachable: u8,
    pub message: String,
    pub error: bool,
}

#[derive(Default)]
struct Shared {
    quit: bool,
    wake: bool,
    probe: bool,
    status: Status,
    queue: Vec<Item>,
    ver: u64,
    busy: bool,
}

pub struct Service {
    sh: Arc<(Mutex<Shared>, Condvar)>,
    th: Option<std::thread::JoinHandle<()>>,
    seen: u64,
    server: String,
}

impl Service {
    /// Start the worker (it loads or makes the key, loads the queue, then waits for work).
    pub fn start() -> Service {
        let sh = Arc::new((Mutex::new(Shared { probe: true, ..Default::default() }), Condvar::new()));
        let (o, server) = match origin() {
            Ok(o) => (Some(o.clone()), o.base()),
            Err(e) => {
                log(&e);
                (None, String::new())
            }
        };
        let w = Worker { sh: sh.clone(), origin: o, dir: dir(), key: None, build: String::new() };
        let th = std::thread::Builder::new().name("bracket".into()).spawn(move || w.run()).ok();
        log(&format!("service started (server {server}, auto-upload {})", if auto_upload() { "on" } else { "off" }));
        Service { sh, th, seen: u64::MAX, server }
    }

    /// Re-send the lists at the next `view` (a setting they show changed).
    pub fn touch(&mut self) {
        self.seen = u64::MAX;
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    fn poke(&self, f: impl FnOnce(&mut Shared)) {
        let (m, cv) = &*self.sh;
        let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut g);
        g.wake = true;
        cv.notify_all();
    }

    /// BRACKET_STATUS: probe the server again.
    pub fn probe(&self) {
        self.poke(|s| s.probe = true);
    }
    /// New inbox files, or the auto-upload setting changed.
    pub fn wake(&self) {
        self.poke(|_| {});
    }

    /// The lists when they changed since the last call: `None` when unchanged or the worker holds the lock.
    pub fn view(&mut self) -> Option<(Status, Vec<Item>)> {
        let g = self.sh.0.try_lock().ok()?;
        if g.ver == self.seen {
            return None;
        }
        self.seen = g.ver;
        Some((g.status.clone(), g.queue.clone()))
    }

    /// Wait (not on the game thread: at the end of a run) until nothing due is left or `limit` passed.
    pub fn flush(&self, limit: Duration) {
        let t0 = std::time::Instant::now();
        self.wake();
        loop {
            {
                let g = self.sh.0.lock().unwrap_or_else(|e| e.into_inner());
                // due before the time is up (a retry a second away counts; one a minute away does not)
                let end = now() + (limit.saturating_sub(t0.elapsed())).as_secs() as i64;
                let due = auto_upload() && g.queue.iter().any(|t| t.state == St::Uploading || (t.state == St::Queued && t.next_try <= end));
                if !g.busy && !g.wake && !due && !inbox_pending() {
                    return;
                }
            }
            if t0.elapsed() >= limit {
                log("flush: time is up; the rest stays queued for the next launch");
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn inbox_pending() -> bool {
    std::fs::read_dir(dir().join("inbox")).is_ok_and(|mut d| d.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|x| x == "txt"))))
}

impl Drop for Service {
    fn drop(&mut self) {
        self.poke(|s| s.quit = true);
        if let Some(t) = self.th.take() {
            // the worker checks `quit` between requests; a request in flight finishes or times out on its own
            let _ = t.join();
        }
    }
}

struct Worker {
    sh: Arc<(Mutex<Shared>, Condvar)>,
    origin: Option<http::Origin>,
    dir: PathBuf,
    key: Option<proto::SigningKey>,
    build: String,
}

/// What the server's answer to `GET /v1/me/next` means for an upload.
#[derive(Debug, PartialEq)]
pub enum NextIs {
    Play { set: u32, game: u32 },
    Other(String),
}

pub fn parse_next(body: &[u8]) -> Option<NextIs> {
    use crate::cosmetics_import::Json;
    let j = Json::parse(std::str::from_utf8(body).ok()?).ok()?;
    match &j {
        Json::Str(s) => Some(NextIs::Other(s.clone())),
        Json::Obj(v) if v.len() == 1 => {
            let (k, inner) = &v[0];
            if k == "Play" {
                let n = |f: &str| match inner.get(f) {
                    Some(Json::Num(x)) if *x >= 0.0 && *x < 4e9 => Some(*x as u32),
                    _ => None,
                };
                Some(NextIs::Play { set: n("set_id")?, game: n("game_no")? })
            } else {
                Some(NextIs::Other(k.clone()))
            }
        }
        _ => None,
    }
}

fn server_error(r: &http::Resp) -> String {
    use crate::cosmetics_import::Json;
    if r.status == 0 {
        return format!("cannot reach the server: {}", r.err);
    }
    let m = std::str::from_utf8(&r.body).ok().and_then(|s| Json::parse(s).ok()).and_then(|j| j.get("error").and_then(|e| e.str().map(str::to_string)));
    format!("{} (HTTP {})", m.unwrap_or_else(|| "error".into()), r.status)
}

fn retryable(status: u32) -> bool {
    status == 0 || status == 408 || status == 429 || status >= 500
}

impl Worker {
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.sh.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn set_msg(&self, m: &str, error: bool) {
        log(m);
        let mut g = self.lock();
        g.status.message = m.into();
        g.status.error = error;
        g.ver += 1;
    }

    fn run(mut self) {
        match load_or_make_key(&self.dir) {
            Ok(k) => {
                let mut g = self.lock();
                g.status.fingerprint = fingerprint(&k);
                g.ver += 1;
                drop(g);
                self.key = Some(k);
            }
            Err(e) => self.set_msg(&e, true),
        }
        {
            let q = queue_from_text(&std::fs::read_to_string(self.dir.join("queue.tsv")).unwrap_or_default());
            let mut g = self.lock();
            g.queue = q;
            g.ver += 1;
        }
        loop {
            let (probe, quit) = {
                let mut g = self.lock();
                g.busy = true;
                g.wake = false;
                (std::mem::take(&mut g.probe), g.quit)
            };
            if quit {
                break;
            }
            self.ingest();
            if probe {
                self.probe();
            }
            let mut wait = 30i64;
            if auto_upload() {
                while let Some(i) = self.due() {
                    if self.lock().quit {
                        break;
                    }
                    self.upload(i);
                }
                if let Some(n) = self.lock().queue.iter().filter(|t| t.state == St::Queued).map(|t| t.next_try).min() {
                    wait = (n - now()).clamp(1, 30);
                }
            }
            let (m, cv) = &*self.sh;
            let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
            g.busy = false;
            if !g.wake && !g.quit {
                g = cv.wait_timeout(g, Duration::from_secs(wait as u64)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
            }
            drop(g);
        }
    }

    /// Inbox files -> queue items (the file is removed once the queue holding it is saved).
    fn ingest(&self) {
        let inbox = self.dir.join("inbox");
        let Ok(rd) = std::fs::read_dir(&inbox) else { return };
        let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "txt")).collect();
        if files.is_empty() {
            return;
        }
        files.sort();
        let mut g = self.lock();
        for f in &files {
            if let Ok(path) = std::fs::read_to_string(f) {
                let path = path.trim().to_string();
                if !path.is_empty() && !g.queue.iter().any(|t| t.file == path) {
                    g.queue.push(Item { file: path, state: St::Queued, attempts: 0, next_try: 0, target: None, reason: String::new() });
                }
            }
        }
        g.ver += 1;
        save_queue(&self.dir, &g.queue);
        drop(g);
        for f in files {
            let _ = std::fs::remove_file(f);
        }
    }

    fn due(&self) -> Option<usize> {
        let t = now();
        self.lock().queue.iter().position(|q| q.state == St::Queued && q.next_try <= t)
    }

    fn call(&mut self, method: &str, path: &str, body: &[u8], extra: &[(&str, String)], long: bool) -> http::Resp {
        let (Some(o), Some(sk)) = (self.origin.as_ref(), self.key.as_ref()) else { return http::Resp::fail("no server or no key") };
        let h = proto::sign_request(sk, method, path, now(), body);
        let mut hs: Vec<(&str, String)> = vec![("x-ab-key", h.key), ("x-ab-time", h.time), ("x-ab-nonce", h.nonce), ("x-ab-sig", h.sig)];
        hs.extend(extra.iter().cloned());
        http::request(o, method, path, &hs, body, long)
    }

    /// BRACKET_STATUS: `GET /` (public) answers 2xx when the server is up.
    fn probe(&mut self) {
        let Some(o) = self.origin.clone() else { return };
        let r = http::request(&o, "GET", "/", &[], b"", false);
        let ok = (200..400).contains(&r.status);
        let mut g = self.lock();
        g.status.reachable = if ok { 1 } else { 2 };
        g.ver += 1;
        drop(g);
        log(&format!("server {}: {}", o.base(), if ok { "reachable".to_string() } else { server_error(&r) }));
    }

    /// Change the item for `file` (items are found by file: finishing one can drop old done ones and move indices).
    fn finish(&self, file: &str, f: impl FnOnce(&mut Item)) {
        let mut g = self.lock();
        if let Some(t) = g.queue.iter_mut().find(|t| t.file == file) {
            f(t);
        }
        // keep the last KEEP_DONE finished uploads on screen
        let done: Vec<usize> = g.queue.iter().enumerate().filter(|(_, t)| t.state == St::Done).map(|(k, _)| k).collect();
        if done.len() > KEEP_DONE {
            let cut = done[done.len() - KEEP_DONE - 1];
            let mut k = 0;
            g.queue.retain(|t| {
                k += 1;
                !(t.state == St::Done && k - 1 <= cut)
            });
        }
        g.ver += 1;
        save_queue(&self.dir, &g.queue);
    }

    fn retry_later(&self, file: &str, why: String, retry_after: u32) {
        self.finish(file, |t| {
            let k = (t.attempts as usize).min(BACKOFF.len() - 1);
            t.attempts += 1;
            t.state = St::Queued;
            t.next_try = now() + if retry_after > 0 { (retry_after as i64).min(300) } else { BACKOFF[k] };
            t.reason = format!("{why}; retry {}", t.attempts);
        });
    }

    fn upload(&mut self, i: usize) {
        let it = {
            let mut g = self.lock();
            g.queue[i].state = St::Uploading;
            g.ver += 1;
            g.queue[i].clone()
        };
        let file = it.file.as_str();
        let name = Path::new(file).file_name().map_or(String::new(), |f| f.to_string_lossy().into_owned());
        let bytes = match std::fs::read(&it.file) {
            Ok(b) if b.len() <= MAX_REPLAY => b,
            Ok(_) => return self.finish(file, |t| (t.state, t.reason) = (St::Failed, "larger than the server takes (16 MiB)".into())),
            Err(e) => return self.finish(file, |t| (t.state, t.reason) = (St::Failed, format!("cannot read the replay: {e}"))),
        };
        let (ev, set, game) = match it.target {
            Some(t) => t,
            None => {
                let Some(ev) = event() else {
                    return self.finish(file, |t| (t.next_try, t.state, t.reason) = (now() + 60, St::Queued, "no event set (bracket_event= in settings.cfg)".into()));
                };
                let r = self.call("GET", &format!("/v1/me/next?event={ev}"), b"", &[], false);
                if r.status != 200 {
                    let why = server_error(&r);
                    return if retryable(r.status) { self.retry_later(file, why, r.retry_after) } else { self.finish(file, |t| (t.next_try, t.state, t.reason) = (now() + 60, St::Queued, why)) };
                }
                match parse_next(&r.body) {
                    Some(NextIs::Play { set, game }) => {
                        self.finish(file, |t| t.target = Some((ev, set, game)));
                        (ev, set, game)
                    }
                    Some(NextIs::Other(w)) => return self.finish(file, |t| (t.next_try, t.state, t.reason) = (now() + 30, St::Queued, format!("no game to upload for: the server says {w}"))),
                    None => return self.retry_later(file, "unexpected answer to /v1/me/next".into(), 0),
                }
            }
        };
        if self.build.is_empty() {
            self.build = engine_build();
        }
        let path = format!("/v1/sets/{ev}/{set}/games/{game}/replay");
        let r = self.call("POST", &path, &bytes, &[("content-type", "application/octet-stream".into()), ("x-ab-build", self.build.clone())], true);
        match r.status {
            200 | 202 => {
                log(&format!("uploaded {name} to event {ev} set {set} game {game} (HTTP {})", r.status));
                self.finish(file, |t| (t.state, t.reason) = (St::Done, format!("event {ev} set {set} game {game}")));
            }
            s if retryable(s) => {
                let why = server_error(&r);
                log(&format!("upload of {name}: {why}"));
                self.retry_later(file, why, r.retry_after)
            }
            _ => {
                let why = server_error(&r);
                log(&format!("upload of {name} refused: {why}"));
                self.finish(file, |t| (t.state, t.reason) = (St::Failed, why));
            }
        }
    }
}

/// The (b) API's requests until the server has them.
pub const NOT_YET: &str = "not available yet: the bracket server does not offer this yet";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_round_trip() {
        let q = vec![
            Item { file: "C:/r/Game_1.slp".into(), state: St::Uploading, attempts: 2, next_try: 99, target: Some((7, 3, 2)), reason: "x\ty".into() },
            Item { file: "/r/b.slp".into(), state: St::Failed, attempts: 1, next_try: 0, target: None, reason: "refused".into() },
            Item { file: "/r/c.slp".into(), state: St::Done, attempts: 0, next_try: 0, target: None, reason: String::new() },
        ];
        let back = queue_from_text(&queue_to_text(&q));
        assert_eq!(back.len(), 2, "done is dropped");
        assert_eq!(back[0].state, St::Queued, "uploading becomes queued");
        assert_eq!(back[0].target, Some((7, 3, 2)));
        assert_eq!(back[0].reason, "x y");
        assert_eq!(back[1].state, St::Failed);
    }

    #[test]
    fn next_answers() {
        let play = br#"{"Play":{"set_id":3,"opponent_tag":"Armada","role":"Host","relay":"r:1","room_code":null,"game_no":2,"wins":[1,0],"best_of":3,"stocks":4,"timer_secs":480,"awaiting_verdict":false}}"#;
        assert_eq!(parse_next(play), Some(NextIs::Play { set: 3, game: 2 }));
        assert_eq!(parse_next(br#""Waiting""#), Some(NextIs::Other("Waiting".into())));
        assert_eq!(parse_next(br#"{"Review":{"set_id":3}}"#), Some(NextIs::Other("Review".into())));
        assert_eq!(parse_next(b"nonsense"), None);
    }

    /// A loopback stand-in for the server's two routes the upload uses. It checks every request's signature the way
    /// the server does (`verify_strict` over the canonical string) and answers the first upload 503 (Retry-After 1).
    fn test_server() -> (String, std::sync::mpsc::Receiver<(String, Vec<u8>, String)>) {
        use std::io::{BufRead, BufReader, Read, Write};
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", l.local_addr().unwrap().port());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut uploads = 0;
            for c in l.incoming() {
                let Ok(c) = c else { continue };
                let mut r = BufReader::new(c.try_clone().unwrap());
                let mut line = String::new();
                if r.read_line(&mut line).is_err() {
                    continue;
                }
                let mut it = line.split_whitespace();
                let (m, p) = (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").to_string());
                let mut h = std::collections::HashMap::new();
                loop {
                    let mut l = String::new();
                    r.read_line(&mut l).unwrap();
                    let l = l.trim_end();
                    if l.is_empty() {
                        break;
                    }
                    if let Some((k, v)) = l.split_once(':') {
                        h.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                    }
                }
                let n: usize = h.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
                let mut body = vec![0u8; n];
                r.read_exact(&mut body).unwrap();
                let g = |k: &str| h.get(k).cloned().unwrap_or_default();
                let sh = proto::SignedHeaders { key: g("x-ab-key"), time: g("x-ab-time"), nonce: g("x-ab-nonce"), sig: g("x-ab-sig") };
                let (code, ans): (u32, String) = if p == "/" {
                    (200, "ok".into())
                } else if proto::verify_request(&sh, &m, &p, now(), &body).is_err() {
                    (401, r#"{"error":"invalid signature"}"#.into())
                } else if p.starts_with("/v1/me/next?event=7") {
                    (200, r#"{"Play":{"set_id":3,"opponent_tag":"B","role":"Host","relay":"r:1","room_code":"KQ7X","game_no":2,"wins":[1,0],"best_of":3,"stocks":4,"timer_secs":480,"awaiting_verdict":false}}"#.into())
                } else if m == "POST" && p == "/v1/sets/7/3/games/2/replay" {
                    uploads += 1;
                    if uploads == 1 {
                        (503, r#"{"error":"busy"}"#.into())
                    } else {
                        let _ = tx.send((p.clone(), body.clone(), g("x-ab-build")));
                        (202, String::new())
                    }
                } else {
                    (404, r#"{"error":"no route"}"#.into())
                };
                let mut c = c;
                let extra = if code == 503 { "Retry-After: 1\r\n" } else { "" };
                let _ = write!(c, "HTTP/1.1 {code} X\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{ans}", ans.len());
            }
        });
        (base, rx)
    }

    /// The whole path on loopback: inbox -> queue -> /v1/me/next -> signed upload, a 503 retried, the queue saved.
    #[test]
    fn upload_to_local_server() {
        let (base, rx) = test_server();
        let d = std::env::temp_dir().join(format!("bracket-up-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let slp = d.join("Game_test.slp");
        let data: Vec<u8> = (0..5000u32).map(|i| (i * 7) as u8).collect();
        std::fs::write(&slp, &data).unwrap();
        std::env::set_var("MELEE_BRACKET_DIR", &d);
        std::env::set_var("MELEE_BRACKET_SERVER", &base);
        std::env::set_var("MELEE_BRACKET_UPLOAD", "1");
        std::env::set_var("MELEE_BRACKET_EVENT", "7");
        replay_written(slp.to_str().unwrap(), "console"); // not netplay: not queued
        assert!(!inbox_pending());
        replay_written(slp.to_str().unwrap(), "network");
        assert!(inbox_pending());
        let mut s = Service::start();
        s.flush(Duration::from_secs(20));
        let (path, body, build) = rx.recv_timeout(Duration::from_secs(5)).expect("the upload arrived");
        assert_eq!(path, "/v1/sets/7/3/games/2/replay");
        assert_eq!(body, data);
        assert!(build.starts_with("melee (rust) "), "{build}");
        let (st, q) = s.view().unwrap();
        assert_eq!(st.reachable, 1);
        assert_eq!(st.fingerprint.len(), 19);
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].state, St::Done, "{:?}", q[0]);
        assert_eq!(q[0].target, Some((7, 3, 2)));
        drop(s);
        // the key file never holds anything but the seed, and the queue file never holds the key
        let key = std::fs::read_to_string(d.join("client.key")).unwrap();
        assert!(!std::fs::read_to_string(d.join("queue.tsv")).unwrap().contains(key.trim()));
        for k in ["MELEE_BRACKET_DIR", "MELEE_BRACKET_SERVER", "MELEE_BRACKET_UPLOAD", "MELEE_BRACKET_EVENT"] {
            std::env::remove_var(k);
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn key_is_made_once_and_private() {
        let d = std::env::temp_dir().join(format!("bracket-key-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let a = load_or_make_key(&d).unwrap();
        let b = load_or_make_key(&d).unwrap();
        assert_eq!(a.to_bytes(), b.to_bytes());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(d.join("client.key")).unwrap().permissions().mode() & 0o777, 0o600);
        }
        assert_eq!(fingerprint(&a).len(), 19);
        let _ = std::fs::remove_dir_all(&d);
    }
}
