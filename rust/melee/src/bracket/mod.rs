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
pub mod ingame;
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

/// Netplay games go to the upload queue: auto-upload is on, or this is a bracket game's run (started from the set
/// flow: its games are the set's, the server needs them to judge).
pub fn uploads_on() -> bool {
    auto_upload() || std::env::var_os("MELEE_BRACKET_RETURN").is_some()
}

/// Setting `bracket_event` (MELEE_BRACKET_EVENT): the event the uploads are for (until linking picks it).
pub fn event() -> Option<i64> {
    crate::cfg::text("bracket_event").and_then(|s| s.trim().parse().ok()).filter(|&e: &i64| e >= 0)
}

/// The slp exporter wrote `path`: queue it for upload when it is a netplay game and auto-upload is on. Called at the
/// end of a match (one small file write; no network).
pub fn replay_written(path: &str, played_on: &str) {
    if played_on != "network" || !uploads_on() {
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

/// The bracket host's netplay run (`MELEE_BRACKET_ROOM=event/set`): post the relay's match code as the game's room
/// code (`POST /v1/sets/{e}/{s}/room`, RoomReq), on a thread of its own (the connect loop never waits on it). The
/// guest's client reads it from the set's flow.
pub fn post_room(code: &str) {
    let Some((e, s)) = std::env::var("MELEE_BRACKET_ROOM").ok().and_then(|v| v.split_once('/').and_then(|(a, b)| Some((a.parse::<i64>().ok()?, b.parse::<u32>().ok()?)))) else { return };
    let code = code.to_string();
    let _ = std::thread::Builder::new().name("bracket-room".into()).spawn(move || {
        let (Ok(o), Ok(sk)) = (origin(), load_or_make_key(&dir())) else { return log("room: no server or key") };
        let path = format!("/v1/sets/{e}/{s}/room");
        let body = ingame::room_body(&code);
        for attempt in 0..5u64 {
            let h = proto::sign_request(&sk, "POST", &path, now(), body.as_bytes());
            let hs = [("x-ab-key", h.key), ("x-ab-time", h.time), ("x-ab-nonce", h.nonce), ("x-ab-sig", h.sig), ("content-type", "application/json".to_string())];
            let r = http::request(&o, "POST", &path, &hs, body.as_bytes(), false);
            if (200..300).contains(&r.status) {
                return log(&format!("room code posted for event {e} set {s}"));
            }
            log(&format!("room code for event {e} set {s}: {}", server_error(&r)));
            if !retryable(r.status) {
                return;
            }
            std::thread::sleep(Duration::from_secs(2 << attempt));
        }
    });
}

/// A key exists already (the player has opened Brackets before): the main menu may poll the badge.
pub fn have_key() -> bool {
    dir().join("client.key").is_file()
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

/// What the screens watch (BRACKET_WATCH): the worker polls only for that, and nothing while 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Watch {
    #[default]
    None,
    /// the main menu: the badge (ETag / 304), every 10 s
    Menu,
    /// Brackets home: /v1/me/home every 3 s (and the badge with it)
    Home,
    /// the set flow: /v1/sets/{e}/{s}/flow every second
    Set,
}

/// A screen's action, done in order by the worker.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Link(String),
    /// ready for the home screen's called set
    Ready,
    /// a strike / ban / pick of the pool's stage at this index (the turn says which)
    Stage(usize),
    /// fetch home now (a screen was entered, or asked for a refresh)
    Home,
    /// BROWSE EVENTS: the public event list
    Browse,
    /// one event's page (its rules and entrants)
    EventView(i64),
    /// sign up for an event (SignupReq)
    Join(i64),
    /// start the current game: the host hosts a room and posts its code, the guest joins the posted code
    Start,
}

/// How the game thread starts the current game (the netplay relaunch), from the set's flow.
#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    pub host: bool,
    pub relay: String,
    /// the guest: the host's posted room code
    pub code: Option<String>,
    pub stage: u16,
    pub event: i64,
    pub set: u32,
}

/// The in-game records (brackets-ui.md shapes), built on the worker from the server's answers.
#[derive(Clone, Debug, Default)]
pub struct InGame {
    pub tag: String,
    pub badge: bool,
    pub next: Vec<String>,
    pub events: Vec<Vec<String>>,
    pub set: Vec<String>,
    /// BROWSE EVENTS records and the event page's record
    pub browse: Vec<Vec<String>>,
    pub event: Vec<String>,
    /// the last action's answer (error?, text), taken once by the screen
    pub said: Option<(bool, String)>,
    /// start the game now (taken once by the game thread)
    pub launch: Option<Launch>,
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
    watch: Watch,
    acts: Vec<Act>,
    ig: InGame,
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
        let w = Worker { sh: sh.clone(), origin: o, dir: dir(), key: None, build: String::new(), poll: Poll::default(), start_waiting: false };
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
    pub fn view(&mut self) -> Option<(Status, Vec<Item>, InGame)> {
        let mut g = self.sh.0.try_lock().ok()?;
        if g.ver == self.seen {
            return None;
        }
        self.seen = g.ver;
        let ig = g.ig.clone();
        g.ig.said = None;
        Some((g.status.clone(), g.queue.clone(), ig))
    }

    /// What the screens show now (the worker polls for it).
    pub fn watch(&self, w: Watch) {
        let same = self.sh.0.try_lock().is_ok_and(|g| g.watch == w);
        if !same {
            self.poke(|s| s.watch = w);
        }
    }

    /// A screen's action (link, ready, a stage move): the worker does it next.
    pub fn act(&self, a: Act) {
        self.poke(|s| s.acts.push(a));
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
                let due = uploads_on() && g.queue.iter().any(|t| t.state == St::Uploading || (t.state == St::Queued && t.next_try <= end));
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
    poll: Poll,
    start_waiting: bool,
}

/// The worker's in-game polling state.
#[derive(Default)]
struct Poll {
    me_at: Option<std::time::Instant>,
    badge_at: Option<std::time::Instant>,
    home_at: Option<std::time::Instant>,
    flow_at: Option<std::time::Instant>,
    etag: String,
    /// the called set from the last home answer: (event, set, game)
    set: Option<(i64, u32, u32)>,
    /// the home action's words for the next set ("Losers round 1 vs Hbox, in about 5 min")
    after: String,
    flow: Option<crate::cosmetics_import::Json>,
    games_seen: usize,
    result_until: Option<std::time::Instant>,
    /// the watch at the last poll: entering home / the set fetches home once
    last_watch: Watch,
    /// home is wanted at the next poll (screen entered, badge changed, an action changed it)
    home_due: bool,
    /// the flow's phase at the last poll ("done" fetches home once, for NEXT)
    last_phase: String,
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
            let acts = std::mem::take(&mut self.lock().acts);
            for a in acts {
                self.act(a);
            }
            wait = wait.min(self.poll_ingame());
            if uploads_on() {
                while let Some(i) = self.due() {
                    if self.lock().quit {
                        break;
                    }
                    self.upload(i);
                }
                if let Some(n) = self.lock().queue.iter().filter(|t| t.state == St::Queued).map(|t| t.next_try).min() {
                    wait = wait.min((n - now()).clamp(1, 30));
                }
            }
            let (m, cv) = &*self.sh;
            let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
            g.busy = false;
            if !g.wake && !g.quit {
                g = cv.wait_timeout(g, Duration::from_millis(wait.max(0) as u64 * 1000 + if wait == 0 { 250 } else { 0 })).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
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

    fn said(&self, err: bool, m: String) {
        log(&m);
        let mut g = self.lock();
        g.ig.said = Some((err, m));
        g.ver += 1;
    }

    /// A signed GET. 403 `key is not registered` marks the key as not a player yet (the link card).
    fn get_json(&mut self, path: &str) -> Result<crate::cosmetics_import::Json, String> {
        let r = self.call("GET", path, b"", &[], false);
        if r.status != 200 {
            self.check_registered(&r);
            return Err(server_error(&r));
        }
        ingame::parse(&r.body).ok_or_else(|| format!("unexpected answer to {path}"))
    }

    /// A public GET (no signature: GET /v1/events and /v1/events/{id}).
    fn get_public(&mut self, path: &str) -> Result<crate::cosmetics_import::Json, String> {
        let Some(o) = self.origin.clone() else { return Err("no server".into()) };
        let r = http::request(&o, "GET", path, &[], b"", false);
        if r.status != 200 {
            return Err(server_error(&r));
        }
        ingame::parse(&r.body).ok_or_else(|| format!("unexpected answer to {path}"))
    }

    fn post_raw(&mut self, path: &str, body: &str) -> http::Resp {
        let r = self.call("POST", path, body.as_bytes(), &[("content-type", "application/json".into())], false);
        self.check_registered(&r);
        r
    }

    fn post_json(&mut self, path: &str, body: &str) -> Result<Vec<u8>, String> {
        let r = self.post_raw(path, body);
        if (200..300).contains(&r.status) { Ok(r.body) } else { Err(server_error(&r)) }
    }

    fn check_registered(&mut self, r: &http::Resp) {
        if ingame::not_registered(r.status, &r.body) {
            let mut g = self.lock();
            if g.status.linked || !g.ig.tag.is_empty() {
                g.status.linked = false;
                g.ig.tag.clear();
                g.ver += 1;
            }
        }
    }

    /// A screen's action (in order). Each answer goes to the screen's status line, and the state it changed is
    /// fetched again at once.
    fn act(&mut self, a: Act) {
        match a {
            Act::Link(code) => match self.post_json("/v1/me/link", &ingame::link_body(&code)) {
                Ok(b) => {
                    let tag = ingame::parse(&b).map(|j| ingame::me(&j).0).unwrap_or_default();
                    {
                        let mut g = self.lock();
                        g.status.linked = true;
                        g.ig.tag = tag.clone();
                        g.ver += 1;
                    }
                    self.poll.home_due = true;
                    self.poll.me_at = None;
                    self.said(false, format!("Linked: you play as {tag}."));
                }
                Err(e) => self.said(true, format!("Link refused: {e}")),
            },
            Act::Ready => {
                let Some((e, s, _)) = self.poll.set else { return self.said(true, "No set is called.".into()) };
                if self.build.is_empty() {
                    self.build = engine_build();
                }
                let body = ingame::ready_body(&self.build);
                match self.post_json(&format!("/v1/sets/{e}/{s}/ready"), &body) {
                    Ok(_) => self.said(false, "Ready.".into()),
                    Err(er) => self.said(true, format!("Ready refused: {er}")),
                }
                self.poll.flow_at = None;
                self.poll.home_due = true;
            }
            Act::Stage(i) => {
                let (Some((e, s, _)), Some(flow)) = (self.poll.set, self.poll.flow.as_ref()) else { return self.said(true, "No stage selection now.".into()) };
                let (ids, turn, n) = (ingame::pool_ids(flow), ingame::turn(flow), ingame::set_record(flow, "", false, "")[4].clone());
                let (Some(&id), Some((kind, true, _))) = (ids.get(i), turn) else { return self.said(true, "Not your turn.".into()) };
                match self.post_json(&format!("/v1/sets/{e}/{s}/games/{n}/stage"), &ingame::stage_body(&kind, id)) {
                    Ok(_) => self.said(false, format!("{} {}.", match kind.as_str() { "strike" => "Struck", "ban" => "Banned", _ => "Picked" }, ingame::stage_name(id))),
                    Err(er) => self.said(true, format!("{kind} refused: {er}")),
                }
                self.poll.flow_at = None;
            }
            Act::Home => self.poll.home_due = true,
            Act::Browse => match self.get_public("/v1/events") {
                Ok(j) => {
                    let mut g = self.lock();
                    g.ig.browse = ingame::browse(&j);
                    g.ver += 1;
                }
                Err(e) => self.said(true, format!("Events: {e}")),
            },
            Act::EventView(id) => match self.get_public(&format!("/v1/events/{id}")) {
                Ok(j) => {
                    let mut g = self.lock();
                    g.ig.event = ingame::event_view(&j);
                    g.ver += 1;
                }
                Err(e) => self.said(true, format!("Event {id}: {e}")),
            },
            Act::Join(id) => {
                if self.build.is_empty() {
                    self.build = engine_build();
                }
                let r = self.post_raw(&format!("/v1/events/{id}/entrants"), &ingame::ready_body(&self.build));
                if (200..300).contains(&r.status) {
                    self.poll.home_due = true;
                    self.said(false, "Signed up. The event shows in MY EVENTS.".into());
                } else if let Some(acc) = (r.status == 403).then(|| ingame::accepted_builds(&r.body)).flatten() {
                    self.said(true, format!("This event takes other game builds: {}. Yours: {}", if acc.is_empty() { "none listed".into() } else { acc.join(", ") }, self.build));
                } else if ingame::not_registered(r.status, &r.body) {
                    self.said(true, "Link your account first (Brackets home).".into());
                } else {
                    self.said(true, format!("Sign-up refused: {}", server_error(&r)));
                }
            }
            Act::Start => {
                self.poll.flow_at = None;
                self.start_waiting = true;
                self.try_start();
            }
        }
    }

    /// The game card's A: hand the game thread a launch when the flow allows it (the guest waits for the room code).
    fn try_start(&mut self) {
        if !self.start_waiting {
            return;
        }
        let (Some((e, s, _)), Some(flow)) = (self.poll.set, self.poll.flow.as_ref()) else { return };
        let Some((host, relay, code, stage)) = ingame::start_info(flow) else { return self.said(true, "The set has no relay yet.".into()) };
        let Some(stage) = stage else { return self.said(true, "The stage is not chosen yet.".into()) };
        if !host && code.is_none() {
            return self.said(false, "Waiting for the host's room code...".into());
        }
        self.start_waiting = false;
        let mut g = self.lock();
        g.ig.launch = Some(Launch { host, relay, code, stage, event: e, set: s });
        g.ver += 1;
    }

    /// The polls the watched screen needs; returns the seconds until the next one is due. The badge is the only route
    /// with an ETag (If-None-Match, 304); home is fetched on entering a screen, after a badge change or an action,
    /// never on a timer; the set's flow every second while it is on screen.
    fn poll_ingame(&mut self) -> i64 {
        use std::time::{Duration as D, Instant};
        let watch = self.lock().watch;
        if watch != self.poll.last_watch {
            if matches!(watch, Watch::Home | Watch::Set) {
                self.poll.home_due = true;
            }
            self.poll.last_watch = watch;
        }
        if watch == Watch::None || self.origin.is_none() || self.key.is_none() {
            return 30;
        }
        let due = |at: Option<Instant>, every: u64| at.is_none_or(|t| t.elapsed() >= D::from_secs(every));
        let mut dirty = false;
        if due(self.poll.me_at, 60) {
            self.poll.me_at = Some(Instant::now());
            match self.get_json("/v1/me") {
                Ok(j) => {
                    let (tag, linked) = ingame::me(&j);
                    let mut g = self.lock();
                    g.status.linked = linked;
                    g.ig.tag = tag;
                    dirty = true;
                }
                Err(_) => dirty = true, // a 403 cleared the tag (check_registered)
            }
        }
        // the player routes answer only a registered key: nothing more until /v1/me says it is a player
        if self.lock().ig.tag.is_empty() {
            if dirty {
                self.lock().ver += 1;
            }
            return 60;
        }
        if due(self.poll.badge_at, 10) {
            self.poll.badge_at = Some(Instant::now());
            let inm = if self.poll.etag.is_empty() { vec![] } else { vec![("if-none-match", self.poll.etag.clone())] };
            let r = self.call("GET", "/v1/me/badge", b"", &inm, false);
            if r.status == 200 {
                if let Some(j) = ingame::parse(&r.body) {
                    self.poll.etag = r.etag.clone();
                    self.lock().ig.badge = ingame::badge(&j);
                    // the badge changed: what home shows may have too
                    self.poll.home_due = true;
                    dirty = true;
                }
            } else {
                // 304: unchanged
                self.check_registered(&r);
            }
        }
        if matches!(watch, Watch::Home | Watch::Set) && std::mem::take(&mut self.poll.home_due) {
            self.poll.home_at = Some(Instant::now());
            if let Ok(j) = self.get_json("/v1/me/home") {
                let (next, events, badge) = ingame::home(&j);
                if let Some(set) = ingame::home_set(&j) {
                    self.poll.set = Some(set);
                }
                self.poll.after = match next[0].as_str() {
                    "upcoming" => {
                        let eta = next[4].parse::<u64>().ok().map_or(String::new(), |e| format!(", in about {} min", e.div_ceil(60).max(1)));
                        format!("{} vs {}{eta}", next[2], next[3])
                    }
                    "called" | "playing" => format!("{} vs {}, now", next[2], next[3]),
                    _ => String::new(),
                };
                let mut g = self.lock();
                g.ig.next = next;
                g.ig.events = events;
                g.ig.badge = badge;
                dirty = true;
            }
        }
        if watch == Watch::Set && due(self.poll.flow_at, 1) {
            self.poll.flow_at = Some(Instant::now());
            if let Some((e, s, _)) = self.poll.set {
                if let Ok(j) = self.get_json(&format!("/v1/sets/{e}/{s}/flow")) {
                    // a game newly judged: the ~5 s result card
                    let judged = match j.get("games") {
                        Some(crate::cosmetics_import::Json::Arr(v)) => v.iter().filter(|g| matches!(g.get("winner"), Some(crate::cosmetics_import::Json::Str(_)))).count(),
                        _ => 0,
                    };
                    if judged > self.poll.games_seen {
                        if self.poll.flow.is_some() {
                            self.poll.result_until = Some(Instant::now() + D::from_secs(5));
                        }
                        self.poll.games_seen = judged;
                    }
                    let phase = j.get("phase").and_then(|p| p.str()).unwrap_or("").to_string();
                    if phase == "done" && self.poll.last_phase != "done" {
                        // NEXT needs home's word on the next set
                        self.poll.home_due = true;
                    }
                    self.poll.last_phase = phase;
                    let fresh = self.poll.result_until.is_some_and(|t| Instant::now() < t);
                    let tag = self.lock().ig.tag.clone();
                    let rec = ingame::set_record(&j, &tag, fresh, &self.poll.after);
                    self.poll.flow = Some(j);
                    self.try_start();
                    let mut g = self.lock();
                    if g.ig.set != rec {
                        g.ig.set = rec;
                        dirty = true;
                    }
                }
            }
        }
        if dirty {
            self.lock().ver += 1;
        }
        match watch {
            Watch::Set => 1,
            _ => 10,
        }
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
                    // phase 2b: the home screen names the called set (and so the event); no setting needed
                    let r = self.call("GET", "/v1/me/home", b"", &[], false);
                    if r.status != 200 {
                        let why = server_error(&r);
                        return if retryable(r.status) { self.retry_later(file, why, r.retry_after) } else { self.finish(file, |t| (t.next_try, t.state, t.reason) = (now() + 60, St::Queued, format!("no event set (bracket_event=) and /v1/me/home: {why}"))) };
                    }
                    return match ingame::parse(&r.body).as_ref().and_then(ingame::home_set) {
                        Some((e, s, g)) => {
                            self.finish(file, |t| t.target = Some((e, s, g)));
                            self.upload_to(file, &name, bytes, (e, s, g))
                        }
                        None => self.finish(file, |t| (t.next_try, t.state, t.reason) = (now() + 30, St::Queued, "no called set to upload for".into())),
                    };
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
        self.upload_to(file, &name, bytes, (ev, set, game))
    }

    fn upload_to(&mut self, file: &str, name: &str, bytes: Vec<u8>, (ev, set, game): (i64, u32, u32)) {
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
                } else if p == "/v1/me" {
                    let _ = tx.send((format!("{m} {p}"), body.clone(), String::new()));
                    (200, r#"{"tag":"Mango","linked":true,"display_name":"m"}"#.into())
                } else if p == "/v1/me/link" || p == "/v1/sets/1/0/ready" || p == "/v1/sets/1/0/games/1/stage" {
                    let _ = tx.send((format!("{m} {p}"), body.clone(), String::new()));
                    (200, r#"{"tag":"Mango"}"#.into())
                } else if p == "/v1/me/badge" {
                    let _ = tx.send((format!("{m} {p}"), body.clone(), g("if-none-match")));
                    if g("if-none-match") == "\"b1\"" { (304, String::new()) } else { (200, r#"{"set_up":true,"event_id":1,"set_id":0}"#.into()) }
                } else if p == "/v1/me/home" {
                    (200, ingame_tests::HOME.into())
                } else if p == "/v1/sets/1/0/flow" {
                    let _ = tx.send((format!("{m} {p}"), body.clone(), String::new()));
                    (200, ingame_tests::FLOW1.into())
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
                let extra = if code == 503 { "Retry-After: 1\r\n" } else if p == "/v1/me/badge" { "ETag: \"b1\"\r\n" } else { "" };
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
        let _g = ENV.lock().unwrap_or_else(|e| e.into_inner());
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
        let (st, q, _) = s.view().unwrap();
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

    mod ingame_tests {
        pub const HOME: &str = r#"{"badge":true,"action":{"SetUp":{"event_id":1,"set_id":0,"side":"Winners","round":1,"round_name":"Final","opponent_tag":"Armada","best_of":3,"game_no":1,"ready":[false,true]}},"events":[{"event":{"id":1,"name":"Sunday","state":"running","entrants":2,"starts_at":null,"description":null,"created_at":0},"next":"Waiting"}]}"#;
        pub const FLOW1: &str = r#"{"event_id":1,"set_id":0,"side":"Winners","round":1,"opponent_tag":"Armada","role":"Host","relay":"r.example:1","room_code":null,"best_of":3,"wins":[0,0],"game_no":1,"ready":[true,true],"phase":"stage_select","stage":{"mode":"strike","pool":[{"id":3,"name":"Pokemon Stadium"},{"id":8,"name":"Yoshi's Story"},{"id":28,"name":"Dream Land N64"},{"id":31,"name":"Battlefield"},{"id":32,"name":"Final Destination"}],"moves":[],"turn":{"who":"me","kind":"strike","count":1},"chosen":null},"games":[],"verified":false,"outcome":null}"#;
    }

    /// The phase-2b screens' routes on loopback: /v1/me, the badge with If-None-Match (a 304 the second time), home,
    /// link, ready, the flow and a strike by stage id (index 3 of the pool = Battlefield, id 31).
    #[test]
    fn ingame_routes_local_server() {
        let _g = ENV.lock().unwrap_or_else(|e| e.into_inner());
        let (base, rx) = test_server();
        let d = std::env::temp_dir().join(format!("bracket-ig-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::env::set_var("MELEE_BRACKET_DIR", &d);
        std::env::set_var("MELEE_BRACKET_SERVER", &base);
        let mut s = Service::start();
        s.watch(Watch::Set);
        let mut seen: Vec<(String, Vec<u8>, String)> = Vec::new();
        let t0 = std::time::Instant::now();
        let mut set_rec = Vec::new();
        while t0.elapsed() < Duration::from_secs(25) {
            if let Some((_, _, ig)) = s.view() {
                if !ig.set.is_empty() && set_rec.is_empty() {
                    set_rec = ig.set.clone();
                    s.act(Act::Link("ABCD2345".into()));
                    s.act(Act::Ready);
                    s.act(Act::Stage(3));
                }
            }
            while let Ok(x) = rx.try_recv() {
                seen.push(x);
            }
            if seen.iter().any(|x| x.0.ends_with("/stage")) && seen.iter().filter(|x| x.0.ends_with("/badge")).count() >= 2 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        drop(s);
        let names: Vec<&String> = seen.iter().map(|x| &x.0).collect();
        let find = |k: &str| seen.iter().find(|x| x.0 == k).unwrap_or_else(|| panic!("{k} not seen: {names:?}"));
        assert_eq!(set_rec[0], "strike");
        assert_eq!(set_rec[16], "3|8|28|31|32");
        assert_eq!(find("POST /v1/me/link").1, br#"{"code":"ABCD2345"}"#);
        assert!(String::from_utf8_lossy(&find("POST /v1/sets/1/0/ready").1).starts_with(r#"{"engine_build":"melee (rust) "#));
        assert_eq!(find("POST /v1/sets/1/0/games/1/stage").1, br#"{"kind":"strike","stage":31}"#);
        let badges: Vec<&String> = seen.iter().filter(|x| x.0.ends_with("/badge")).map(|x| &x.2).collect();
        assert_eq!(badges[0], "", "the first badge poll has no If-None-Match");
        assert_eq!(badges[1], "\"b1\"", "later polls send the ETag back");
        for k in ["MELEE_BRACKET_DIR", "MELEE_BRACKET_SERVER"] {
            std::env::remove_var(k);
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The tests that set MELEE_BRACKET_* run one at a time.
    static ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
