//! The SSBM Nucleus mod browser's engine (port of pc/platform/gw_nucleus_engine.inc): worker threads, the catalog and
//! its cache, the download queue, pictures and installs, and the catalogs the in-game browser reads (browser.rs fills
//! the gd-ui lists from [`Engine::view`] and passes its requests here).
//!
//! NO NETWORK OR DISK WORK RUNS ON THE GAME THREAD, AND NONE STARTS DURING A NETPLAY SESSION. The game thread only
//! `try_lock`s the shared state (a worker holding it costs the browser one stale frame, never a wait), copies small
//! finished records out, and sets flags. Threads, started the first time the Nucleus view opens, kept for the process:
//!   catalog : the sync (`catalog::sync_run`: a full keyset page-through once, then `updated_since` + the removed feed,
//!             at most one poll per 10 minutes, back-off on 429 / 5xx / offline), and the cache saves;
//!   files   : the DOWNLOAD QUEUE (one install after another, never online) and the queue saves;
//!   view    : the search (filter + sort over the catalog) into the records the browser shows;
//!   pictures: two threads that fetch, cache and decode the pictures of the rows in view.
//! Nothing here is a static or lives in the arena: the engine is a field of the play flow (`browser::State`), its
//! threads allocate on the system heap (`mark_foreign_thread`), so no rollback or savestate can touch it (the statics
//! rule, notes/net.md). Workers never log directly: they queue lines the game thread prints (`pump`).
//!
//! Where things live: mods dir = `--mods` / `mods_dir=` / `MELEE_MODS_DIR`, else `<exe dir>/mods`; cache =
//! `<mods>/.nucleus` (a dot folder: the mods scanner skips it) or `MELEE_NUCLEUS_DIR`: catalog.jsonl + state.json (the
//! catalog and the sync), queue.json (the queue, atomic after every change), thumbs/<fnv64>.img (pictures as served,
//! trimmed to `MELEE_NUCLEUS_CACHE_MB`, default 200), dl/<fnv64>.bin (downloads, so a retry does not count another
//! download; trimmed to 400 MB). Costumes install into the skin catalog (`cosmetics::dir`), the rest into the mods dir
//! (install.rs). The record shapes are the C client's, so one cache folder serves both builds.
pub mod catalog;
pub mod http;
pub mod install;
pub mod queue;

use catalog::{Catalog, NcMod, Query, SyncState};
use queue::Queue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

/// Pictures: fitted inside a fixed canvas (aspect kept, centred, transparent around): what the browser's slots hold.
pub const PIC_W: u32 = 96;
pub const PIC_H: u32 = 72;
const PIC_MAX_BYTES: usize = 6 << 20;
const PICS: usize = 200;
const WANT_TICKS: u64 = 180;
const DL_CAP: u64 = 400 << 20;
/// Records the browser can hold of a search (its list is 64 rows of 256 bytes).
pub const VIEW_MAX: usize = 60;

#[derive(Default)]
struct Pic {
    url: String,
    state: u8, // 0 want, 1 busy, 2 ready, 3 failed
    last: u64,
    rgba: Option<Arc<Vec<u8>>>,
    retry_at: i64,
    sent: bool,
}

#[derive(Default)]
struct Inner {
    cat: Catalog,
    sync: SyncState,
    loaded: bool,
    cache_stale: bool,
    sync_want: u8,
    syncing: bool,
    sync_failed: bool,
    sync_ok: bool,
    sync_what: String,
    sync_done: usize,
    sync_total: i64,
    sync_msg: String,
    dq: Queue,
    dq_dirty: bool,
    cur_id: i64,
    interrupt: bool,
    job: String,
    job_bad: bool,
    restart: bool,
    installed: u64,
    query: Query,
    query_ver: u64,
    view_built: (u64, u64),
    view: Vec<Vec<u8>>,
    pics: HashMap<u32, Pic>,
    tick: u64,
    logq: Vec<String>,
}

struct Shared {
    st: Mutex<Inner>,
    ver: AtomicU64,
    quit: AtomicBool,
    online: AtomicBool,
    dir: PathBuf,
    mods: PathBuf,
    cosmetics: PathBuf,
    iso: String,
    api: String,
    cache_cap: u64,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.st.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn log(&self, s: String) {
        let mut g = self.lock();
        if g.logq.len() < 64 {
            g.logq.push(s);
        }
    }
    fn bump(&self) {
        self.ver.fetch_add(1, Relaxed);
    }
    fn p(&self, n: &str) -> PathBuf {
        self.dir.join(n)
    }
    fn stopped(&self) -> bool {
        self.quit.load(Relaxed) || self.online.load(Relaxed)
    }
}

/// What the browser shows, copied out under the lock (the game thread).
pub struct View {
    pub results: Vec<Vec<u8>>,
    pub queue: Vec<Vec<u8>>,
    pub state: Vec<u8>,
    pub installed: u64,
}

pub struct Engine {
    sh: Arc<Shared>,
    network: bool,
    seen_ver: u64,
    seen_installed: u64,
}

fn rec(f: &[&str]) -> Vec<u8> {
    let mut v = f.join("\0").into_bytes();
    if v.len() > 255 {
        // the browser's record buffer is 256 bytes: cut the last field (the description) at a character boundary
        let mut n = 255;
        while n > 0 && (v[n] & 0xC0) == 0x80 {
            n -= 1;
        }
        v.truncate(n);
    }
    v
}

pub fn fnv32(s: &str) -> u32 {
    s.bytes().fold(2166136261u32, |h, c| (h ^ c as u32).wrapping_mul(16777619)).max(1)
}
fn fnv64(s: &str) -> u64 {
    s.bytes().fold(1469598103934665603u64, |h, c| (h ^ c as u64).wrapping_mul(1099511628211))
}

/// The picture of a mod's row: a costume's portrait, else its first screenshot (the API's own thumbnails are WebP).
fn pic_url(m: &NcMod) -> String {
    let s = m.files.iter().find(|f| f.costume && !f.csp.is_empty()).map(|f| f.csp.as_str()).or(m.shots.first().map(|s| s.as_str())).unwrap_or("");
    catalog::media_full(s)
}

impl Engine {
    /// Paths and the saved queue; no thread yet (`open` starts them).
    pub fn new(mods: Option<String>, iso: String) -> Engine {
        let mods = mods.map(PathBuf::from).unwrap_or_else(|| std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("mods"))).unwrap_or_else(|| PathBuf::from("mods")));
        let dir = std::env::var("MELEE_NUCLEUS_DIR").ok().filter(|s| !s.is_empty()).map(PathBuf::from).unwrap_or_else(|| mods.join(".nucleus"));
        let api = std::env::var("MELEE_NUCLEUS_API").ok().filter(|s| !s.is_empty() && s.len() < 200).unwrap_or_else(|| catalog::API_BASE.to_string());
        if api != catalog::API_BASE {
            http::set_override(&api);
        }
        let cache_cap = std::env::var("MELEE_NUCLEUS_CACHE_MB").ok().and_then(|v| v.parse::<u64>().ok()).filter(|&v| v > 0).unwrap_or(200) << 20;
        let sh = Arc::new(Shared { st: Mutex::new(Inner::default()), ver: AtomicU64::new(1), quit: AtomicBool::new(false), online: AtomicBool::new(false), dir, mods, cosmetics: crate::cosmetics::dir(), iso, api, cache_cap });
        Engine { sh, network: false, seen_ver: 0, seen_installed: 0 }
    }

    /// The Nucleus view opened (the game thread): start the threads once and ask for a sync (it keeps the 10-minute rule).
    pub fn open(&mut self) {
        if !self.network {
            self.network = true;
            let sh = self.sh.clone();
            let _ = std::fs::create_dir_all(sh.p("thumbs"));
            let _ = std::fs::create_dir_all(sh.p("dl"));
            let _ = std::fs::create_dir_all(sh.p("art"));
            for (name, f) in [("nucleus-sync", sync_thread as fn(Arc<Shared>)), ("nucleus-files", files_thread), ("nucleus-view", view_thread), ("nucleus-pic0", pic_thread), ("nucleus-pic1", pic_thread)] {
                let s = sh.clone();
                let _ = std::thread::Builder::new().name(name.into()).spawn(move || {
                    melee_engine::arena::mark_foreign_thread();
                    f(s)
                });
            }
            sh.log(format!("browser started; cache {}, api {}, user agent {}", sh.dir.display(), sh.api, http::user_agent()));
        }
        if !self.sh.online.load(Relaxed) {
            if let Ok(mut g) = self.sh.st.try_lock() {
                if g.sync_want == 0 && !g.syncing {
                    g.sync_want = 1;
                }
            }
        }
    }

    /// Online (a netplay / spectate session): nothing starts and a running download stops (its entry stays queued).
    pub fn set_online(&mut self, on: bool) {
        self.sh.online.store(on, Relaxed);
    }

    /// The game thread, once per tick: the queued log lines (never blocks; an empty Vec when a worker holds the lock).
    pub fn pump(&mut self) -> Vec<String> {
        match self.sh.st.try_lock() {
            Ok(mut g) => {
                g.tick += 1;
                std::mem::take(&mut g.logq)
            }
            Err(_) => Vec::new(),
        }
    }

    /// The records when something changed since the last call (None: unchanged, or a worker holds the lock).
    pub fn view(&mut self) -> Option<View> {
        let v = self.sh.ver.load(Relaxed);
        if v == self.seen_ver {
            return None;
        }
        let g = match self.sh.st.try_lock() {
            Ok(g) => g,
            Err(TryLockError::WouldBlock) => return None,
            Err(TryLockError::Poisoned(e)) => e.into_inner(),
        };
        self.seen_ver = v;
        let queue = g.dq.it.iter().map(|t| rec(&[&t.id.to_string(), t.state.name(), &t.pct.to_string(), &t.title, &t.reason])).collect();
        let sync = if g.syncing {
            if g.sync_total > 0 { format!("{} {}/{}", g.sync_what, g.sync_done, g.sync_total) } else { format!("{}...", g.sync_what) }
        } else if !g.sync_msg.is_empty() {
            g.sync_msg.clone()
        } else if !g.loaded {
            "Loading the saved catalog...".into()
        } else if g.sync_ok || g.sync.full {
            format!("{} mods from SSBM Nucleus", g.cat.m.len())
        } else {
            "Contacting SSBM Nucleus...".into()
        };
        let state = rec(&[
            &sync,
            &g.cat.m.len().to_string(),
            if g.dq.processing { "1" } else { "0" },
            &g.dq.pending().to_string(),
            &g.job,
            if g.restart { "1" } else { "0" },
            if self.sh.online.load(Relaxed) { "1" } else { "0" },
            if g.job_bad { "1" } else { "0" },
            catalog::SITE,
        ]);
        let installed = g.installed;
        Some(View { results: g.view.clone(), queue, state, installed })
    }

    /// True once after an install finished (the browser reloads the mods and skins lists).
    pub fn take_installed(&mut self, v: &View) -> bool {
        let ch = v.installed != self.seen_installed;
        self.seen_installed = v.installed;
        ch
    }

    pub fn search(&mut self, q: Query) {
        let mut g = self.sh.lock();
        if g.query != q {
            g.query = q;
            g.query_ver += 1;
        }
    }

    /// A on a mod: queue it, or take it out again.
    pub fn toggle(&mut self, id: i64) -> String {
        if self.sh.online.load(Relaxed) {
            return "Mods cannot be installed while you are online.".into();
        }
        let mut g = self.sh.lock();
        let (title, author, why) = match g.cat.find(id) {
            Some(at) => (g.cat.m[at].title.clone(), g.cat.m[at].author.clone(), catalog::why_not(&g.cat.m[at])),
            None => (String::new(), String::new(), ""),
        };
        if !why.is_empty() && g.dq.find(id).is_none() {
            return why.to_string();
        }
        let r = g.dq.toggle(id, &title, &author, catalog::now());
        g.dq_dirty = true;
        drop(g);
        self.sh.bump();
        match r {
            1 => format!("Queued {title}. Start installs the queue."),
            -1 => format!("Took {title} out of the queue."),
            _ => "That mod is being installed right now.".into(),
        }
    }

    pub fn set_processing(&mut self, on: bool) -> String {
        let mut g = self.sh.lock();
        g.dq.processing = on && g.dq.pending() > 0;
        g.dq.ver += 1;
        g.dq_dirty = true;
        let msg = if !on { "Paused the queue.".into() } else if g.dq.processing { format!("Installing {} queued mod(s)...", g.dq.pending()) } else { "The queue is empty.".into() };
        drop(g);
        self.sh.bump();
        msg
    }

    pub fn refresh(&mut self) {
        let mut g = self.sh.lock();
        if !g.syncing {
            g.sync_want = 2;
        }
    }

    /// The browser wants these pictures (keys from the records): renewed every frame a row is on screen.
    pub fn want_pics(&mut self, keys: &[u32]) {
        if keys.is_empty() {
            return;
        }
        let Ok(mut g) = self.sh.st.try_lock() else { return };
        let (t, now) = (g.tick, catalog::now());
        for &k in keys {
            if let Some(p) = g.pics.get_mut(&k) {
                p.last = t;
                if p.state == 3 && now >= p.retry_at {
                    p.state = 0;
                }
                if p.state == 4 {
                    p.state = 0;
                }
                if p.state == 2 && p.sent {
                    p.sent = false; // the host evicted it and asks again
                }
            }
        }
    }

    /// Decoded pictures not yet handed to the host (at most `n`, the freshest wants first).
    pub fn ready_pics(&mut self, n: usize) -> Vec<(u32, Arc<Vec<u8>>)> {
        let Ok(mut g) = self.sh.st.try_lock() else { return Vec::new() };
        let t = g.tick;
        let mut v: Vec<(u64, u32)> = g.pics.iter().filter(|(_, p)| p.state == 2 && !p.sent && t.saturating_sub(p.last) < WANT_TICKS).map(|(k, p)| (p.last, *k)).collect();
        v.sort_by(|a, b| b.cmp(a));
        v.truncate(n);
        v.into_iter()
            .filter_map(|(_, k)| {
                let p = g.pics.get_mut(&k)?;
                p.sent = true;
                Some((k, p.rgba.clone()?))
            })
            .collect()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.sh.quit.store(true, Relaxed);
        if self.network {
            save_queue(&self.sh, true);
        }
    }
}

// ---------------------------------------------------------------- persistence

fn save_queue(sh: &Shared, force: bool) {
    let text = {
        let mut g = sh.lock();
        if !g.dq_dirty && !force {
            return;
        }
        g.dq_dirty = false;
        g.dq.to_json()
    };
    if queue::write_atomic(&sh.p("queue.json"), text.as_bytes()).is_err() {
        sh.log(format!("queue: could not write {}", sh.p("queue.json").display()));
        sh.lock().dq_dirty = true;
    }
}

fn save_catalog(sh: &Shared) {
    let (cat, st) = {
        let g = sh.lock();
        (g.cat.save_text(), g.sync.to_json())
    };
    let _ = queue::write_atomic(&sh.p("catalog.jsonl"), cat.as_bytes());
    let _ = queue::write_atomic(&sh.p("state.json"), st.as_bytes());
}

/// Keep a cache folder under `cap` bytes, least recently used (modified) first.
fn trim(dir: &Path, cap: u64) {
    let mut v: Vec<(std::time::SystemTime, u64, PathBuf)> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().filter_map(|e| e.metadata().ok().filter(|m| m.is_file()).map(|m| (m.modified().unwrap_or(std::time::UNIX_EPOCH), m.len(), e.path()))).collect())
        .unwrap_or_default();
    let mut total: u64 = v.iter().map(|e| e.1).sum();
    v.sort();
    for (_, n, p) in v {
        if total <= cap {
            break;
        }
        if std::fs::remove_file(&p).is_ok() {
            total -= n;
        }
    }
}

fn sleep_unless(sh: &Shared, ms: u64) {
    let mut w = 0;
    while w < ms && !sh.quit.load(Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100.min(ms - w)));
        w += 100;
    }
}

// ---------------------------------------------------------------- the catalog thread

struct Io<'a>(&'a Shared);
impl catalog::SyncIo for Io<'_> {
    fn fetch(&self, url: &str) -> http::Resp {
        let r = http::get(url, catalog::MAX_BODY, &mut |_, _| self.0.quit.load(Relaxed));
        if r.status == 0 {
            self.0.log(format!("net: {}", if r.err.is_empty() { "no answer" } else { &r.err }));
        }
        r
    }
    fn now(&self) -> i64 {
        catalog::now()
    }
    fn sleep_ms(&self, ms: u64) {
        sleep_unless(self.0, ms)
    }
    fn stop(&self) -> bool {
        self.0.stopped()
    }
    fn progress(&self, what: &str, done: usize, total: i64) {
        let mut g = self.0.lock();
        g.sync_what = what.into();
        g.sync_done = done;
        g.sync_total = total;
        drop(g);
        self.0.bump();
    }
    fn checkpoint(&self, s: &SyncState) {
        self.0.lock().sync = s.clone();
        save_catalog(self.0);
    }
    fn with_cat<R>(&self, f: impl FnOnce(&mut Catalog) -> R) -> R {
        let r = f(&mut self.0.lock().cat);
        self.0.bump();
        r
    }
}

fn sync_thread(sh: Arc<Shared>) {
    // what the last session saved; a cache from an older schema (or with no state beside it) is discarded and synced again
    let st = std::fs::read_to_string(sh.p("state.json")).ok();
    let state = st.as_deref().map(SyncState::from_json);
    let stale = state.as_ref().is_none_or(|s| s.stale);
    let mut cat = None;
    if stale {
        if std::fs::remove_file(sh.p("catalog.jsonl")).is_ok() {
            sh.log("catalog: the cache is from an older version and was discarded; syncing again".into());
        }
    } else if let Ok(t) = std::fs::read_to_string(sh.p("catalog.jsonl")) {
        cat = Some(Catalog::load_text(&t));
    }
    {
        let mut g = sh.lock();
        if let Some(s) = &state {
            g.sync = s.clone();
        }
        g.cache_stale = stale || !state.as_ref().is_some_and(|s| s.full);
        if let Some(c) = cat {
            g.logq.push(format!("catalog: {} mods from the cache", c.m.len()));
            g.cat = c;
        }
        g.loaded = true;
    }
    sh.bump();
    trim(&sh.p("thumbs"), sh.cache_cap);
    trim(&sh.p("dl"), DL_CAP);
    while !sh.quit.load(Relaxed) {
        let want = {
            let mut g = sh.lock();
            let w = if sh.online.load(Relaxed) { 0 } else { std::mem::take(&mut g.sync_want) };
            if w > 0 {
                g.syncing = true;
                g.sync_failed = false;
                g.sync_msg.clear();
                g.sync_what = "Contacting SSBM Nucleus".into();
                g.sync_done = 0;
                g.sync_total = 0;
            }
            w
        };
        if want > 0 {
            sh.bump();
            let mut s = sh.lock().sync.clone();
            let rc = catalog::sync_run(&mut s, &Io(&sh), &sh.api, want == 2);
            let n = {
                let mut g = sh.lock();
                g.syncing = false;
                match rc {
                    Err(()) => {
                        g.sync_failed = true;
                        g.sync_msg = s.msg.clone();
                    }
                    Ok(true) => {
                        g.sync_ok = true;
                        g.sync_msg.clear();
                    }
                    Ok(false) => g.sync_msg = s.msg.clone(),
                }
                if rc.is_ok() && s.full {
                    g.cache_stale = false;
                }
                g.sync = s.clone();
                g.cat.m.len()
            };
            sh.bump();
            sh.log(format!("sync {}: {n} mods{}{}", match rc { Ok(true) => "done", Ok(false) => "skipped", Err(()) => "failed" }, if s.msg.is_empty() { "" } else { " - " }, s.msg));
            if rc.is_ok() {
                save_catalog(&sh);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}

// ---------------------------------------------------------------- the view thread (search)

fn view_thread(sh: Arc<Shared>) {
    while !sh.quit.load(Relaxed) {
        let key = {
            let g = sh.lock();
            (g.query_ver, g.cat.version ^ g.dq.ver.rotate_left(32))
        };
        if sh.lock().view_built != key {
            // filter under the lock (the game thread only try_locks: it skips a frame, never waits)
            let mut g = sh.lock();
            let idx = g.cat.filter(&g.query);
            let mut recs = Vec::new();
            let mut urls = Vec::new();
            for &i in idx.iter().take(VIEW_MAX) {
                let m = &g.cat.m[i];
                let (qs, pct) = match g.dq.find(m.id) {
                    Some(a) => (g.dq.it[a].state.name(), g.dq.it[a].pct),
                    None => ("", 0),
                };
                let url = pic_url(m);
                let key = if url.is_empty() { 0 } else { fnv32(&url) };
                if key != 0 {
                    urls.push((key, url));
                }
                let why = catalog::why_not(m);
                let what = if m.typ == catalog::T_COSTUME { m.files.iter().find_map(|f| f.fighter.filter(|_| f.costume)).map_or("Costume", |f| catalog::FIGHTERS[f].label) } else { catalog::TYPE_LABEL[m.typ] };
                recs.push(rec(&[&m.id.to_string(), &m.title, &m.author, what, qs, &pct.to_string(), &key.to_string(), if why.is_empty() { "1" } else { "0" }, why, &m.downloads.to_string(), &m.desc]));
            }
            if idx.len() > VIEW_MAX {
                recs.push(rec(&["0", &format!("... {} more: type more words to narrow it", idx.len() - VIEW_MAX), "", "", "", "0", "0", "0", "", "0", ""]));
            }
            for (k, u) in urls {
                g.pics.entry(k).or_insert_with(|| Pic { url: u, state: 4, ..Default::default() });
            }
            g.view = recs;
            g.view_built = key;
            drop(g);
            sh.bump();
        }
        // a cooldown: a sync that changes the catalog every page re-filters at most 4 times a second
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

// ---------------------------------------------------------------- pictures

/// Fit RGBA `w`x`h` inside PIC_W x PIC_H (aspect kept, centred, box-filtered).
fn fit(src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let s = (PIC_W as f32 / w as f32).min(PIC_H as f32 / h as f32);
    let (dw, dh) = (((w as f32 * s) as u32).clamp(1, PIC_W), ((h as f32 * s) as u32).clamp(1, PIC_H));
    let (ox, oy) = ((PIC_W - dw) / 2, (PIC_H - dh) / 2);
    let mut out = vec![0u8; (PIC_W * PIC_H * 4) as usize];
    for y in 0..dh {
        let (y0, y1) = (y * h / dh, ((y + 1) * h / dh).max(y * h / dh + 1).min(h));
        for x in 0..dw {
            let (x0, x1) = (x * w / dw, ((x + 1) * w / dw).max(x * w / dw + 1).min(w));
            let mut acc = [0u32; 4];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let o = ((sy * w + sx) * 4) as usize;
                    let a = src[o + 3] as u32;
                    for c in 0..3 {
                        acc[c] += src[o + c] as u32 * a;
                    }
                    acc[3] += a;
                }
            }
            let n = (y1 - y0) * (x1 - x0);
            let o = (((y + oy) * PIC_W + x + ox) * 4) as usize;
            if acc[3] > 0 {
                for c in 0..3 {
                    out[o + c] = (acc[c] / acc[3]) as u8;
                }
            }
            out[o + 3] = (acc[3] / n) as u8;
        }
    }
    out
}

fn pic_thread(sh: Arc<Shared>) {
    while !sh.quit.load(Relaxed) {
        let pick = {
            let mut g = sh.lock();
            let t = g.tick;
            // wants not renewed for 3 s (the row scrolled away) go back to idle; the freshest want goes first
            for p in g.pics.values_mut() {
                if p.state == 0 && t.saturating_sub(p.last) > WANT_TICKS {
                    p.state = 4;
                }
            }
            let k = g.pics.iter().filter(|(_, p)| p.state == 0).max_by_key(|(_, p)| p.last).map(|(k, _)| *k);
            if g.pics.len() > PICS {
                let mut old: Vec<(u64, u32)> = g.pics.iter().filter(|(_, p)| p.state != 1).map(|(k, p)| (p.last, *k)).collect();
                old.sort();
                for (_, k) in old.into_iter().take(g.pics.len() - PICS) {
                    g.pics.remove(&k);
                }
            }
            k.and_then(|k| {
                let p = g.pics.get_mut(&k)?;
                p.state = 1;
                Some((k, p.url.clone()))
            })
        };
        let Some((key, url)) = pick else {
            std::thread::sleep(std::time::Duration::from_millis(40));
            continue;
        };
        let path = sh.p("thumbs").join(format!("{:016x}.img", fnv64(&url)));
        let (bytes, from_disk, status) = match std::fs::read(&path) {
            Ok(b) => (Some(b), true, 200),
            Err(_) if sh.stopped() => (None, false, 1),
            Err(_) => {
                let r = http::get(&url, PIC_MAX_BYTES, &mut |_, _| sh.stopped());
                ((r.status == 200 && r.body.len() >= 16).then_some(r.body), false, r.status)
            }
        };
        let decoded = bytes.as_ref().map(|b| crate::cosmetics_import::decode_png_up_to(b, 4096));
        let had = decoded.is_some();
        let mut g = sh.lock();
        let ok = match decoded {
            Some(Ok(p)) => {
                if !from_disk {
                    let _ = queue::write_atomic(&path, bytes.as_deref().unwrap_or_default());
                }
                Some(Arc::new(fit(&p.rgba, p.w, p.h)))
            }
            Some(Err(e)) => {
                g.logq.push(format!("picture {url}: {e}"));
                if from_disk {
                    let _ = std::fs::remove_file(&path);
                }
                None
            }
            None => None,
        };
        if let Some(p) = g.pics.get_mut(&key) {
            match ok {
                Some(px) => {
                    p.rgba = Some(px);
                    p.state = 2;
                    p.sent = false;
                }
                None => {
                    p.state = 3;
                    p.retry_at = catalog::now() + if status == 1 { 20 } else if status == 404 || had { 3600 } else { 120 };
                }
            }
        }
        drop(g);
        sh.bump();
    }
}

// ---------------------------------------------------------------- the files thread (the queue)

fn files_thread(sh: Arc<Shared>) {
    let n = install::clean_staging(&sh.mods);
    let loaded = match std::fs::read_to_string(sh.p("queue.json")) {
        Ok(t) => match Queue::from_json(&t) {
            Some(q) => Some(q),
            None => {
                let _ = std::fs::rename(sh.p("queue.json"), sh.p("queue.json.bad"));
                None
            }
        },
        Err(_) => Some(Queue::default()),
    };
    {
        let mut g = sh.lock();
        let bad = loaded.is_none();
        g.dq = loaded.unwrap_or_default();
        let (cnt, resume) = (g.dq.it.len(), g.dq.processing && g.dq.pending() > 0);
        g.logq.push(format!(
            "queue: {cnt} entr{} restored{}{}{}",
            if cnt == 1 { "y" } else { "ies" },
            if bad { " (queue.json was unreadable and was set aside as queue.json.bad)" } else { "" },
            if resume { "; downloads were running and resume now" } else { "" },
            if n > 0 { "; removed an unfinished install folder" } else { "" }
        ));
    }
    sh.bump();
    let mut disc: Option<melee_engine::gcm::Gcm> = None;
    while !sh.quit.load(Relaxed) {
        save_queue(&sh, false);
        if queue_step(&sh, &mut disc) {
            continue;
        }
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    save_queue(&sh, false);
}

fn item_pct(sh: &Shared, pct: u32) {
    let mut g = sh.lock();
    let id = g.cur_id;
    let v = g.dq.ver;
    g.dq.progress(id, pct);
    if g.dq.ver != v {
        drop(g);
        sh.bump();
    }
}

/// GET for an install, kept in the dl cache under `tag` (a retry or a re-install of the same version does not download,
/// and so count, again). Stops when the player goes online (interrupt).
fn dl(sh: &Shared, url: &str, max: usize, tag: &str, pct: (u32, u32)) -> Result<Vec<u8>, u32> {
    let path = sh.p("dl").join(format!("{:016x}.bin", fnv64(tag)));
    if let Ok(b) = std::fs::read(&path) {
        return Ok(b);
    }
    let r = http::get(url, max, &mut |got, exp| {
        if sh.stopped() {
            sh.lock().interrupt = true;
            return true;
        }
        if exp > 0 {
            item_pct(sh, pct.0 + ((pct.1 - pct.0) as u64 * got as u64 / exp as u64) as u32);
        }
        false
    });
    if r.status != 200 {
        if r.status == 0 {
            sh.log(format!("download {url}: {}", r.err));
        }
        return Err(r.status);
    }
    if r.body.len() <= 48 << 20 {
        let _ = queue::write_atomic(&path, &r.body);
    }
    Ok(r.body)
}

fn queue_step(sh: &Shared, disc: &mut Option<melee_engine::gcm::Gcm>) -> bool {
    if sh.stopped() {
        return false;
    }
    let (id, title, have) = {
        let mut g = sh.lock();
        if !g.dq.processing || !g.loaded {
            return false;
        }
        let Some(i) = g.dq.next() else {
            if g.dq.pending() == 0 {
                g.dq.processing = false;
                g.dq_dirty = true;
                drop(g);
                sh.bump();
            }
            return false;
        };
        let (id, title) = (g.dq.it[i].id, g.dq.it[i].title.clone());
        let have = g.cat.find(id).map(|a| !g.cat.m[a].files.is_empty() || !g.cat.m[a].bundle.is_empty()).unwrap_or(false) && !g.cache_stale;
        g.dq.mark_running(id);
        g.cur_id = id;
        g.interrupt = false;
        g.dq_dirty = true;
        (id, title, have)
    };
    sh.bump();
    save_queue(sh, false);
    // the record: the catalog's, else (missing, an older cache, or the bulk list's empty early copy) straight from the API
    let mut msg = String::new();
    let mut ok = false;
    if !have {
        let r = http::get(&format!("{}/mods/{id}", sh.api), catalog::MAX_BODY, &mut |_, _| sh.stopped());
        if r.status == 200 {
            catalog::apply_one(&Io(sh), &r.body);
        } else if sh.lock().cat.find(id).is_none() {
            msg = match r.status {
                404 => "It is not on SSBM Nucleus any more.".into(),
                0 => "SSBM Nucleus could not be reached. Check the connection and retry.".into(),
                s => catalog::http_reason(s).into(),
            };
        }
    }
    let m = {
        let g = sh.lock();
        g.cat.find(id).map(|a| g.cat.m[a].clone())
    };
    if msg.is_empty() {
        match m {
            None => msg = "That mod is not in the catalog.".into(),
            Some(m) => match install_mod(sh, &m, disc) {
                Ok(s) => {
                    ok = true;
                    msg = s;
                }
                Err(s) => msg = s,
            },
        }
    }
    let mut g = sh.lock();
    g.cur_id = 0;
    if g.interrupt || sh.quit.load(Relaxed) {
        // the player went online or the game is closing: back in line, nothing failed
        g.dq.requeue(id);
        g.dq_dirty = true;
        g.logq.push(format!("queue: {id} ({title}) paused: {}", if sh.quit.load(Relaxed) { "closing" } else { "online" }));
        drop(g);
        sh.bump();
        return false;
    }
    g.dq.finish(id, ok, if ok { "" } else { &msg });
    if ok {
        g.restart = true;
        g.installed += 1;
    }
    if g.dq.next().is_none() {
        g.dq.processing = false; // the run is over (failed entries stay for a retry)
    }
    g.dq_dirty = true;
    g.job = if ok { format!("{title}: {msg}") } else { format!("{title} failed: {msg}") };
    g.job_bad = !ok;
    g.logq.push(format!("queue: {id} ({title}) {}: {msg}", if ok { "installed" } else { "failed" }));
    drop(g);
    sh.bump();
    save_queue(sh, false);
    true
}

fn install_mod(sh: &Shared, m: &NcMod, disc: &mut Option<melee_engine::gcm::Gcm>) -> Result<String, String> {
    let why = catalog::why_not(m);
    if !why.is_empty() {
        return Err(why.into());
    }
    let post = install::Post { id: m.id, title: &m.title, author: &m.author, page: &m.page, updated: &m.updated, installed_at: catalog::iso(catalog::now()) };
    let mut log = |s: String| sh.log(s);
    let paused = || Err::<String, String>("Paused.".into());
    let mut zip: Option<crate::cosmetics_import::Zip> = None;
    let mut zip_status: Option<u32> = None;
    // the post's own zip (GET /mods/{id}/download), fetched once for every zip-only file
    let get_zip = |zip: &mut Option<crate::cosmetics_import::Zip>, zip_status: &mut Option<u32>| -> bool {
        if zip.is_none() && zip_status.is_none() {
            match dl(sh, &catalog::download_url(&sh.api, m.id, 0), 160 << 20, &format!("z{}_{}", m.id, m.updated), (5, 80)) {
                Ok(b) => match crate::cosmetics_import::Zip::parse(b) {
                    Ok(z) => *zip = Some(z),
                    Err(e) => {
                        sh.log(format!("install {}: the post's zip: {e}", m.id));
                        *zip_status = Some(1)
                    }
                },
                Err(s) => *zip_status = Some(s),
            }
        }
        zip.is_some()
    };
    if !m.bundle.is_empty() {
        if !get_zip(&mut zip, &mut zip_status) {
            return if sh.lock().interrupt { paused() } else { Err(catalog::http_reason(zip_status.unwrap_or(0)).into()) };
        }
        item_pct(sh, 85);
        let mut reference = |_: &str| None;
        let mut env = install::Env { reference: &mut reference, can_commit: &|| !sh.stopped() };
        return install::install_bundle(&sh.mods, &post, zip.take().unwrap(), &mut env, &mut log);
    }
    // the files: per-file download, else (a zip-only post, or the per-file address failed) an entry of the post's zip
    let usable: Vec<&catalog::NcFile> = m
        .files
        .iter()
        .enumerate()
        .filter(|(i, f)| if m.typ == catalog::T_COSTUME { f.costume && f.fighter.is_some_and(|g| catalog::FIGHTERS[g].installable) && !catalog::variant_skip(&m.files, *i) } else { f.supported && f.cat != catalog::FC_COSTUME })
        .map(|(_, f)| f)
        .collect();
    let total = usable.len() as u32;
    let mut got: Vec<(&catalog::NcFile, Vec<u8>)> = Vec::new();
    let mut first_err = String::new();
    for (k, f) in usable.iter().enumerate() {
        let span = (k as u32 * 80 / total.max(1), (k as u32 + 1) * 80 / total.max(1));
        item_pct(sh, span.0);
        let cap = if f.cat == catalog::FC_AUDIO { 64 << 20 } else { 32 << 20 };
        let mut data = None;
        if !f.zip {
            match dl(sh, &catalog::download_url(&sh.api, m.id, f.id), cap, &format!("f{}_{}_{}", m.id, f.id, m.updated), span) {
                Ok(b) => data = Some(b),
                Err(s) => {
                    if sh.lock().interrupt {
                        return paused();
                    }
                    sh.log(format!("install {}: file {}: download failed (HTTP {s})", m.id, f.id));
                }
            }
        }
        if data.is_none() {
            if !get_zip(&mut zip, &mut zip_status) {
                if sh.lock().interrupt {
                    return paused();
                }
                if first_err.is_empty() {
                    first_err = catalog::http_reason(zip_status.unwrap_or(0)).into();
                }
                continue;
            }
            let z = zip.as_ref().unwrap();
            match z.entries.iter().find(|e| e.name == f.filename || catalog::basename(&e.name) == catalog::basename(&f.filename)) {
                Some(e) => match z.read(e) {
                    Ok(b) => data = Some(b),
                    Err(e) if first_err.is_empty() => first_err = format!("Could not unpack the post's zip: {e}."),
                    Err(_) => {}
                },
                None if first_err.is_empty() => first_err = format!("Could not unpack the post's zip: {} is not in it.", f.filename),
                None => {}
            }
        }
        if let Some(d) = data {
            got.push((f, d));
        }
    }
    if sh.stopped() {
        return paused();
    }
    item_pct(sh, 90);
    if got.is_empty() {
        return Err(if first_err.is_empty() { "Nothing in this post could be downloaded.".into() } else { first_err });
    }
    let r = if m.typ == catalog::T_COSTUME {
        let mut input = Vec::new();
        for (f, d) in got {
            // the portrait, then the stock icon: art that cannot be had just leaves the default
            let art = |u: &str, kind: &str| -> Option<Vec<u8>> {
                if u.is_empty() {
                    return None;
                }
                let path = sh.p("art").join(format!("{}_{kind}.png", f.id));
                if let Ok(b) = std::fs::read(&path) {
                    return Some(b);
                }
                let r = http::get(&catalog::media_full(u), PIC_MAX_BODY_ART, &mut |_, _| sh.stopped());
                (r.status == 200).then(|| {
                    let _ = queue::write_atomic(&path, &r.body);
                    r.body
                })
            };
            let (csp, stock) = (art(&f.csp, "csp"), art(&f.stock, "stock"));
            input.push(install::CostumeIn { file_id: f.id, filename: f.filename.clone(), color: f.color.clone(), data: d, csp, stock });
        }
        install::install_costumes(&sh.cosmetics, &post, input, &mut log)
    } else {
        let input: Vec<install::LooseIn> = got.into_iter().map(|(f, d)| install::LooseIn { file_id: f.id, cat: f.cat, filename: f.filename.clone(), stem: f.stem.clone(), data: d }).collect();
        if disc.is_none() {
            *disc = melee_engine::gcm::Gcm::open(&sh.iso).ok();
        }
        let mut reference = |p: &str| disc.as_mut().and_then(|d| d.read_vanilla(p).ok());
        let mut env = install::Env { reference: &mut reference, can_commit: &|| !sh.stopped() };
        let _ = std::fs::create_dir_all(&sh.mods);
        install::install_loose(&sh.mods, &post, &input, &mut env, &mut log)
    };
    r.map_err(|e| if first_err.is_empty() { e } else { format!("{e} ({first_err})") })
}

const PIC_MAX_BODY_ART: usize = 4 << 20;
