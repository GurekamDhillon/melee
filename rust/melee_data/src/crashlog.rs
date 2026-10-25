//! `melee-pc.log` and the crash report (port of pc/platform/gw_log.c): the log categories, the
//! OSReport line assembler, the flood / repeat filters, the privacy redaction, and the compact crash
//! report (`crashlogs/crash-<time>.log`, at most 64 KB) with frames symbolized from `melee-pc.map`.
//!
//! The logger is a plain state machine: the clock, the frame counter, the environment and the
//! settings it reads are fields of [`LogHost`], and the lines it writes (the `[frame=.. ms=..]`
//! stamped text, what the C prints to stdout and appends to melee-pc.log) collect in
//! [`Logger::out`]. Not ported: the Win32 stack scan of a `panic` crash and the process-level
//! plumbing (critical section, `CreateFileA` append handle, exception filter): those are platform
//! code that feeds this module.

use std::collections::HashMap;

pub const GL_LINE: usize = 320;
pub const GL_TAIL: usize = 256;
pub const GL_SCENES: usize = 16;
pub const GL_PADS: usize = 16;
pub const GL_WATCH: usize = 8;
pub const GL_FLOOD_KEYS: usize = 16;
pub const GL_FLOOD_WINDOW_MS: u32 = 10000;
pub const GL_FLOOD_MAX: u32 = 400;
pub const GL_REPORT_MAX: usize = 64 * 1024;

pub const GL_CORE: usize = 0;
pub const GL_SCENE: usize = 1;
pub const GL_PAD: usize = 2;
pub const GL_NET: usize = 3;
pub const GL_RENDER: usize = 4;
pub const GL_WATCHDOG: usize = 5;
pub const GL_TEX: usize = 6;
pub const GL_FRONTEND: usize = 7;
pub const GL_MEX: usize = 8;
pub const GL_HEAP: usize = 9;
pub const GL_DVD: usize = 10;
pub const GL_AUDIO: usize = 11;
pub const GL_SNAP: usize = 12;
pub const GL_GOBJ: usize = 13;
pub const GL_COUNT: usize = 14;
pub const GL_FIRST_QUIET: usize = GL_RENDER;

pub const GL_NAME: [&str; GL_COUNT] =
    ["core", "scene", "pad", "net", "render", "watchdog", "tex", "frontend", "mex", "heap", "dvd", "audio", "snap", "gobj"];

type B = Vec<u8>;

fn starts(s: &[u8], p: &str) -> bool {
    s.starts_with(p.as_bytes())
}

fn contains(s: &[u8], p: &str) -> bool {
    let p = p.as_bytes();
    p.is_empty() || s.windows(p.len()).any(|w| w == p)
}

/// `gl_copy(dst, cap, src)`: at most `cap - 1` bytes (and not past a NUL).
fn copy_cap(src: &[u8], cap: usize) -> B {
    let src = &src[..src.iter().position(|&c| c == 0).unwrap_or(src.len())];
    src[..src.len().min(cap - 1)].to_vec()
}

/// `snprintf(dst, cap, "%s", s)` semantics for a byte string.
fn trunc(s: &[u8], cap: usize) -> B {
    copy_cap(s, cap)
}

/// A quiet line that reads like a problem is written anyway (`gl_problem`).
pub fn problem(s: &[u8]) -> bool {
    const WORDS: [&str; 20] = [
        "fail", "error", "warn", "cannot", "can't", "unable", "invalid", "missing", "refus", "reject", "assert", "fatal", "panic", "bad ", "overflow",
        "exhaust", "unsupported", "corrupt", "not ", "too ",
    ];
    // lowercase copy of at most GL_LINE-1 bytes
    let low: B = s.iter().take(GL_LINE - 1).map(|c| c.to_ascii_lowercase()).collect();
    let low = &low[..low.iter().position(|&c| c == 0).unwrap_or(low.len())];
    WORDS.iter().any(|w| contains(low, w))
}

/// `gl_classify`
pub fn classify(s: &[u8]) -> usize {
    if starts(s, "scene: ") || starts(s, "gw: scene:") {
        return GL_SCENE;
    }
    if starts(s, "gw: DIAG pads")
        || starts(s, "gw: DIAG gcraw")
        || starts(s, "gw: gc adapter")
        || starts(s, "gw: input")
        || starts(s, "gw: pad")
        || starts(s, "pad:")
        || starts(s, "gw: PAD")
    {
        return GL_PAD;
    }
    if starts(s, "netplay:") || starts(s, "rb:") || starts(s, "gw: netplay") || starts(s, "net:") {
        return GL_NET;
    }
    if starts(s, "gw: DIAG ") {
        return GL_RENDER;
    }
    if starts(s, "gw: at ") || starts(s, "gw:   retrace=") {
        return GL_WATCHDOG;
    }
    if starts(s, "gxtex: ") && contains(s, " -> handle ") {
        return GL_TEX;
    }
    if starts(s, "frontend: ") && contains(s, ".json: ") && contains(s, " bytes, ") {
        return GL_FRONTEND;
    }
    if starts(s, "interp: ") {
        return if starts(s, "interp: installed") { GL_CORE } else { GL_MEX };
    }
    if starts(s, "ftfunction: ")
        || starts(s, "itfunction: ")
        || starts(s, "grfunction:   ")
        || starts(s, "grtrace: ")
        || starts(s, "mexdata: MEX_Index")
        || starts(s, "mexeffect: bank ")
        || starts(s, "mexeffect: fighter kind ")
        || starts(s, "item: kind ")
    {
        return GL_MEX;
    }
    if starts(s, "gw: kind ") && contains(s, " Kirby hat: ") {
        return GL_MEX;
    }
    if starts(s, "lbHeap: heap ") || starts(s, "heap: ") || starts(s, "gw: ARAlloc(") {
        return GL_HEAP;
    }
    if starts(s, "gw: dvd: ") || starts(s, "gw: mods:   /") {
        return GL_DVD;
    }
    if starts(s, "synth: bank ") || starts(s, "lbAudioAx: m-ex sfx ") {
        return GL_AUDIO;
    }
    if starts(s, "snap: render pass") {
        return GL_SNAP;
    }
    if starts(s, "gobj: ") {
        return GL_GOBJ;
    }
    GL_CORE
}

fn find_cat(name: &[u8]) -> Option<usize> {
    GL_NAME.iter().position(|n| n.as_bytes().eq_ignore_ascii_case(name))
}

// ---- privacy (gw_disc_privacy.h) ------------------------------------------------------------------

/// `gw_disc_redact`: every case-insensitive, slash-insensitive occurrence of `secret` becomes `<disc>`.
pub fn disc_redact(input: &[u8], cap: usize, secret: Option<&[u8]>) -> B {
    let mut out = Vec::new();
    let mut i = 0;
    let norm = |c: u8| {
        let c = c.to_ascii_lowercase();
        if c == b'\\' {
            b'/'
        } else {
            c
        }
    };
    while i < input.len() && input[i] != 0 && out.len() + 1 < cap {
        if let Some(sec) = secret {
            if !sec.is_empty() {
                let (mut a, mut b) = (i, 0);
                while a < input.len() && input[a] != 0 && b < sec.len() {
                    if norm(input[a]) != norm(sec[b]) {
                        break;
                    }
                    // JSON's escaped backslashes represent one separator
                    if input[a] == b'\\' && input.get(a + 1) == Some(&b'\\') {
                        a += 1;
                    }
                    a += 1;
                    b += 1;
                }
                if b == sec.len() {
                    for &r in b"<disc>" {
                        if out.len() + 1 < cap {
                            out.push(r);
                        }
                    }
                    i = a;
                    continue;
                }
            }
        }
        out.push(input[i]);
        i += 1;
    }
    out
}

/// `gw_disc_clean`: redact the disc path and the disc environment variables.
pub fn disc_clean(input: &[u8], cap: usize, iso_path: Option<&[u8]>, env: &HashMap<String, String>) -> B {
    let mut out = disc_redact(input, cap, iso_path);
    for key in ["MELEE_ISO", "GW_ISO_VANILLA", "GW_ISO_AKANEIA", "GW_ISO_ACE", "GW_ISO"] {
        let secret = env.get(key).map(|v| v.as_bytes().to_vec());
        let t = disc_redact(&out, 8192, secret.as_deref());
        out = copy_cap(&t, cap);
    }
    out
}

// ---- the logger -------------------------------------------------------------------------------------

/// Everything the C reads from outside gw_log.c.
#[derive(Clone, Debug, Default)]
pub struct LogHost {
    /// `GetTickCount64()`
    pub tick64: u64,
    /// `gw_prof_log_frame()`
    pub frame: u32,
    /// the process environment (`getenv` / `_putenv_s`)
    pub env: HashMap<String, String>,
    /// settings.cfg `log=` (`gw_Settings_Str("log", ...)`)
    pub settings_log: B,
    /// `gw_iso_path()`
    pub iso_path: Option<B>,
}

#[derive(Clone, Copy, Default)]
struct Flood {
    key: [u8; 16],
    since: u32,
    n: u32,
    over: u32,
}

pub struct Logger {
    pub host: LogHost,
    state: i32,
    on: [i8; GL_COUNT],
    all: bool,
    spec: B,
    render_state: i32,
    render_lines: i32,
    dropped: [u32; GL_COUNT],
    crashing: bool,
    pub shutting_down: bool,
    crash_mark: u32,
    last: B,
    repeats: u32,
    pending: B,
    flood: [Flood; GL_FLOOD_KEYS],
    t0: u64,
    pub disc_path: B,
    pub disc_id: B,
    pub mods: B,
    tail: Vec<B>,
    ntail: u32,
    scene: Vec<B>,
    nscene: u32,
    pad: Vec<B>,
    npad: u32,
    watch: Vec<B>,
    nwatch: u32,
    /// every line written (stamped), in order
    pub out: Vec<B>,
}

impl Logger {
    pub fn new(host: LogHost) -> Logger {
        Logger {
            host,
            state: 0,
            on: [0; GL_COUNT],
            all: false,
            spec: Vec::new(),
            render_state: 0,
            render_lines: 0,
            dropped: [0; GL_COUNT],
            crashing: false,
            shutting_down: false,
            crash_mark: 0,
            last: Vec::new(),
            repeats: 0,
            pending: Vec::new(),
            flood: [Flood::default(); GL_FLOOD_KEYS],
            t0: 0,
            disc_path: Vec::new(),
            disc_id: Vec::new(),
            mods: Vec::new(),
            tail: vec![Vec::new(); GL_TAIL],
            ntail: 0,
            scene: vec![Vec::new(); GL_SCENES],
            nscene: 0,
            pad: vec![Vec::new(); GL_PADS],
            npad: 0,
            watch: vec![Vec::new(); GL_WATCH],
            nwatch: 0,
            out: Vec::new(),
        }
    }

    fn defaults(&mut self) {
        for i in 0..GL_COUNT {
            self.on[i] = if i < GL_FIRST_QUIET { 1 } else { 0 };
        }
        self.all = false;
    }

    /// `gl_apply`: a MELEE_LOG / `log=` list on top of the current state.
    fn apply(&mut self, spec: &[u8]) {
        let buf = copy_cap(spec, 256);
        for tok in buf.split(|c| matches!(c, b',' | b';' | b' ' | b'\t')).filter(|t| !t.is_empty()) {
            let mut tok = tok;
            let mut neg = false;
            if tok[0] == b'-' || tok[0] == b'!' {
                neg = true;
                tok = &tok[1..];
            } else if tok[0] == b'+' {
                tok = &tok[1..];
            }
            if tok.eq_ignore_ascii_case(b"all") || tok.eq_ignore_ascii_case(b"verbose") {
                self.on = [1; GL_COUNT];
                self.all = true;
            } else if tok.eq_ignore_ascii_case(b"default") {
                self.defaults();
            } else if tok.eq_ignore_ascii_case(b"quiet") || tok.eq_ignore_ascii_case(b"none") {
                self.defaults();
                self.on[GL_RENDER] = -1;
            } else if let Some(i) = find_cat(tok) {
                self.on[i] = if neg {
                    if i == GL_RENDER {
                        -1
                    } else {
                        0
                    }
                } else {
                    1
                };
            }
        }
    }

    fn env_on(&self, name: &str) -> bool {
        self.host.env.get(name).map_or(false, |v| !v.is_empty() && !v.starts_with('0'))
    }

    fn configure(&mut self) {
        const LEGACY: [(usize, &str, bool); 9] = [
            (GL_HEAP, "MELEE_HEAP_TRACE", true),
            (GL_DVD, "MELEE_DVD_TRACE", true),
            (GL_MEX, "MELEE_GR_TRACE", false),
            (GL_MEX, "MELEE_MEX_TRACE_CALLS", false),
            (GL_MEX, "MELEE_MEX_TRACE_PARTS", false),
            (GL_MEX, "MELEE_MEX_TRACE_SCALE", false),
            (GL_MEX, "MELEE_CSS_TRACE", false),
            (GL_NET, "MELEE_RB_LOG", false),
            (GL_SNAP, "MELEE_RB_LOG", false),
        ];
        self.state = 1;
        self.t0 = self.host.tick64;
        self.defaults();
        let env_spec = self.host.env.get("MELEE_LOG").cloned().filter(|v| !v.is_empty());
        let spec: B;
        match env_spec {
            None => {
                // settings.cfg log= (the launcher's Diagnostics box writes MELEE_LOG instead)
                let saved = copy_cap(&self.host.settings_log, 128);
                let mut s = if !saved.is_empty() { b"settings log=".to_vec() } else { b"default".to_vec() };
                s.extend_from_slice(&saved);
                self.spec = copy_cap(&s, 128);
                spec = saved;
            }
            Some(v) => {
                let mut s = b"MELEE_LOG=".to_vec();
                s.extend_from_slice(v.as_bytes());
                self.spec = copy_cap(&s, 128);
                spec = v.into_bytes();
            }
        }
        for (cat, env, _) in LEGACY {
            if self.env_on(env) {
                self.on[cat] = 1;
            }
        }
        if self.env_on("MELEE_PC_TRACE_OSREPORT") {
            self.apply(b"all");
        }
        self.apply(&spec);
        for (cat, env, push) in LEGACY {
            if push && self.on[cat] == 1 && !self.host.env.contains_key(env) {
                self.host.env.insert(env.to_string(), "1".to_string());
            }
        }
        self.state = 2;
    }

    /// `gw_GObjLogEnabled`
    pub fn gobj_log_enabled(&mut self) -> bool {
        if self.state == 2 {
            return self.on[GL_GOBJ] > 0;
        }
        if self.state == 0 {
            self.configure();
        }
        self.on[GL_GOBJ] > 0
    }

    /// The categories as one line (`gl_describe`), for the log and the report.
    pub fn describe(&self) -> B {
        let mut out = Vec::new();
        let cap = GL_LINE;
        let mut i = 0;
        while i < GL_COUNT && out.len() + 24 < cap {
            let st = if self.on[i] > 0 {
                "on"
            } else if i == GL_RENDER && self.on[i] == 0 {
                "once-per-scene"
            } else {
                "off"
            };
            let piece = format!("{}{}={}", if i > 0 { " " } else { "" }, GL_NAME[i], st);
            let room = cap - 1 - out.len();
            out.extend_from_slice(&piece.as_bytes()[..piece.len().min(room)]);
            i += 1;
        }
        out
    }

    fn write(&mut self, s: &[u8]) {
        let private = disc_clean(s, 8192, self.host.iso_path.as_deref(), &self.host.env);
        let stamped = copy_cap(
            &[format!("[frame={} ms={}] ", self.host.frame, self.host.tick64.wrapping_sub(self.t0)).as_bytes(), &private].concat(),
            8192,
        );
        self.out.push(stamped.clone());
        let slot = (self.ntail as usize) % GL_TAIL;
        self.tail[slot] = copy_cap(&stamped, GL_LINE);
        self.ntail = self.ntail.wrapping_add(1);
    }

    fn flush_repeats(&mut self) {
        if self.repeats != 0 {
            let msg = format!("log: (the line above repeated {} more time{})", self.repeats, if self.repeats == 1 { "" } else { "s" });
            self.repeats = 0;
            self.write(msg.as_bytes());
        }
    }

    fn snoop(&mut self, s: &[u8], cat: usize) {
        if cat == GL_SCENE && starts(s, "scene: ") {
            let slot = (self.nscene as usize) % GL_SCENES;
            self.scene[slot] = copy_cap(s, GL_LINE);
            self.nscene += 1;
        }
        if cat == GL_PAD || contains(s, "adapter") {
            let slot = (self.npad as usize) % GL_PADS;
            self.pad[slot] = copy_cap(s, GL_LINE);
            self.npad += 1;
        }
        if starts(s, "gw: at ") {
            let slot = (self.nwatch as usize) % GL_WATCH;
            self.watch[slot] = copy_cap(s, GL_LINE);
            self.nwatch += 1;
        }
        if starts(s, "melee-pc: window closed") {
            self.shutting_down = true;
        }
        if starts(s, "gw: disc image ") {
            let p = &s["gw: disc image ".len()..];
            let key = b", disk id ";
            if let Some(pos) = p.windows(key.len()).position(|w| w == key) {
                self.disc_path = copy_cap(self.host.iso_path.as_deref().unwrap_or(b""), 260);
                self.disc_id = copy_cap(&p[pos + key.len()..], 8);
            }
        }
        if starts(s, "gw: mods: ") && contains(s, " new paths)") {
            let p = &s["gw: mods: ".len()..];
            if let Some(e) = p.iter().position(|&c| c == b' ') {
                let have = self.mods.len();
                if have + e + 3 < 512 {
                    if have > 0 {
                        self.mods.push(b',');
                    }
                    self.mods.extend_from_slice(&p[..e]);
                }
            }
        }
    }

    fn note_dropped(&mut self) {
        let total: u32 = self.dropped.iter().sum();
        if total == 0 {
            return;
        }
        let cap = GL_LINE;
        let mut msg: B = Vec::new();
        let put = |msg: &mut B, t: &str| {
            let room = cap - 1 - msg.len();
            msg.extend_from_slice(&t.as_bytes()[..t.len().min(room)]);
        };
        put(&mut msg, &format!("log: {} quiet line(s) not written since the last scene (", total));
        let mut i = 0;
        while i < GL_COUNT && msg.len() + 24 < cap {
            if self.dropped[i] != 0 {
                put(&mut msg, &format!("{} {}, ", GL_NAME[i], self.dropped[i]));
            }
            self.dropped[i] = 0;
            i += 1;
        }
        // (the C zeroes only the counters its loop visited)
        if msg.len() >= 2 && msg.len() < cap {
            let l = msg.len();
            msg.truncate(l - 2);
        }
        put(&mut msg, ") - MELEE_LOG=<category> writes them");
        self.write(&msg);
    }

    /// 1 = over this window's budget for the line's prefix (always-on categories only).
    fn flooded(&mut self, s: &[u8]) -> bool {
        let now = self.host.tick64 as u32;
        let mut key = [0u8; 16];
        let mut k = 0;
        while k + 1 < key.len() && k < s.len() && s[k] != 0 && s[k] != b':' && s[k] != b' ' {
            key[k] = s[k];
            k += 1;
        }
        let (mut slot, mut oldest) = (None, 0usize);
        for i in 0..GL_FLOOD_KEYS {
            if self.flood[i].key == key {
                slot = Some(i);
                break;
            }
            if self.flood[i].since < self.flood[oldest].since {
                oldest = i;
            }
        }
        let slot = match slot {
            Some(i) => i,
            None => {
                let f = &mut self.flood[oldest];
                f.key = key;
                f.since = now;
                f.n = 0;
                f.over = 0;
                oldest
            }
        };
        if now.wrapping_sub(self.flood[slot].since) > GL_FLOOD_WINDOW_MS {
            if self.flood[slot].over != 0 {
                let name = String::from_utf8_lossy(&key[..key.iter().position(|&c| c == 0).unwrap_or(16)]).into_owned();
                let msg = format!("log: flood limit: {} more '{}' line(s) in {} s were not written", self.flood[slot].over, name, GL_FLOOD_WINDOW_MS / 1000);
                self.write(&copy_cap(msg.as_bytes(), 96));
            }
            let f = &mut self.flood[slot];
            f.since = now;
            f.n = 0;
            f.over = 0;
        }
        self.flood[slot].n += 1;
        if self.flood[slot].n > GL_FLOOD_MAX {
            self.flood[slot].over += 1;
            return true;
        }
        false
    }

    /// One finished line (no newline): decide, note, write.
    fn line(&mut self, s: &[u8]) {
        if self.crashing || self.state == 1 {
            self.flush_repeats();
            self.write(s);
            return;
        }
        if self.state == 0 {
            self.configure();
            let d = self.describe();
            let mut msg = b"log: ".to_vec();
            msg.extend_from_slice(&d);
            msg.extend_from_slice(b" (");
            msg.extend_from_slice(&self.spec);
            msg.extend_from_slice(b") - MELEE_LOG=all writes everything");
            let msg = copy_cap(&msg, GL_LINE + 160);
            self.write(s); // the first line stays first
            self.write(&msg);
            return;
        }
        let cat = classify(s);
        self.snoop(s, cat);
        if cat == GL_SCENE && starts(s, "scene: enter ") {
            self.note_dropped();
            self.render_state = 0;
            self.render_lines = 0;
        }
        let pass;
        if self.on[cat] > 0 {
            pass = true;
        } else if cat == GL_RENDER {
            let mut p = false;
            if self.on[cat] == 0 {
                if self.render_state == 0 {
                    if starts(s, "gw: DIAG copy #") {
                        self.render_state = 1;
                    }
                    p = starts(s, "gw: DIAG proj") || starts(s, "gw: DIAG posmtx") || starts(s, "gw: DIAG copy #") || starts(s, "gw: DIAG   ");
                    if p {
                        self.render_lines += 1;
                        if self.render_lines > 60 {
                            p = false;
                            self.render_state = 2;
                        }
                    }
                } else if self.render_state == 1 {
                    p = starts(s, "gw: DIAG   ");
                    if !p {
                        self.render_state = 2;
                    }
                }
            }
            pass = p;
        } else {
            pass = cat != GL_WATCHDOG && problem(s);
        }
        if !pass {
            self.dropped[cat] += 1;
            return;
        }
        if !self.all && cat < GL_FIRST_QUIET && self.flooded(s) {
            return;
        }
        let cmp_len = GL_LINE - 1;
        let a = &s[..s.len().min(cmp_len)];
        let b = &self.last[..self.last.len().min(cmp_len)];
        // strncmp(s, last, 319) == 0
        let (la, lb) = (a.iter().position(|&c| c == 0).unwrap_or(a.len()), b.iter().position(|&c| c == 0).unwrap_or(b.len()));
        if a[..la] == b[..lb] {
            self.repeats += 1;
            return;
        }
        self.flush_repeats();
        self.last = copy_cap(s, GL_LINE);
        self.write(s);
    }

    /// `gl_text`: split into lines and hand each to `line`.
    fn text(&mut self, text: &[u8]) {
        let mut p = 0;
        loop {
            let nl = text[p..].iter().position(|&c| c == b'\n');
            let mut piece = match nl {
                Some(n) => &text[p..p + n],
                None => &text[p..],
            };
            while piece.last() == Some(&b'\r') {
                piece = &piece[..piece.len() - 1];
            }
            let piece = piece.to_vec();
            self.line(&piece);
            match nl {
                None => break,
                Some(n) => {
                    p += n + 1;
                    if p >= text.len() || text[p] == 0 {
                        break;
                    }
                }
            }
        }
    }

    fn flush_pending(&mut self) {
        if !self.pending.is_empty() {
            let p = std::mem::take(&mut self.pending);
            self.line(&p);
        }
    }

    /// `gw_log`: `text` is the already formatted message (formatted into 2048 bytes by the C).
    pub fn log(&mut self, text: &[u8]) {
        let buf = copy_cap(text, 2048);
        self.flush_pending();
        self.text(&buf);
    }

    /// `gw_log_osreportv`: the game prints lines in pieces; text is held until its newline.
    pub fn osreport(&mut self, text: &[u8]) {
        let buf = copy_cap(text, 2048);
        for &c in &buf {
            if c == b'\n' {
                while self.pending.last() == Some(&b'\r') {
                    self.pending.pop();
                }
                if !self.pending.is_empty() {
                    let p = std::mem::take(&mut self.pending);
                    self.line(&p);
                }
                self.pending.clear();
            } else {
                if self.pending.len() >= 1024 - 1 {
                    self.flush_pending();
                }
                self.pending.push(c);
            }
        }
    }

    /// `gw_log_raw`: two strings verbatim, no format expansion, no filtering.
    pub fn raw(&mut self, prefix: &[u8], text: Option<&[u8]>) {
        let mut buf = prefix.to_vec();
        buf.extend_from_slice(text.unwrap_or(b"(null)"));
        let buf = copy_cap(&buf, GL_LINE * 2);
        self.flush_pending();
        self.write(&buf);
    }

    /// `gw_log_crash_begin`: from here on every line is written unfiltered.
    pub fn crash_begin(&mut self) {
        self.flush_pending();
        self.flush_repeats();
        if !self.crashing {
            self.crashing = true;
            self.crash_mark = self.ntail;
        }
    }

    /// Counters the oracle compares: lines written, scene / pad / watchdog lines noted.
    pub fn counts(&self) -> (u32, u32, u32, u32) {
        (self.ntail, self.nscene, self.npad, self.nwatch)
    }

    pub fn crashing(&self) -> bool {
        self.crashing
    }

    /// What configured the logger (`gl.spec`).
    pub fn spec(&self) -> &[u8] {
        &self.spec
    }

    fn tail_line(&self, t: u32) -> &B {
        &self.tail[(t as usize) % GL_TAIL]
    }
}

// ---- the crash report ---------------------------------------------------------------------------------

/// What the report reads from the machine.
#[derive(Clone, Debug, Default)]
pub struct ReportHost {
    /// local time (year, month, day, hour, minute, second)
    pub time: (u16, u16, u16, u16, u16, u16),
    /// first line of version.txt beside the exe (None = file absent)
    pub version_line: Option<B>,
    /// the exe's link time stamp
    pub link_stamp: u32,
    /// the 0x40 title bytes of the disc image at `disc_path` (None = unreadable / no disc)
    pub disc_title: Option<B>,
    /// `gw_Settings_Summary`
    pub settings_summary: B,
    /// MELEE_* environment variables in block order ("NAME=value")
    pub melee_env: Vec<B>,
    /// the "os:" line without its label
    pub os_line: B,
    /// contents of melee-pc.map (None = absent)
    pub map: Option<B>,
    /// %USERNAME% and %USERPROFILE%
    pub user: B,
    pub profile: B,
}

struct Report {
    buf: B,
    user: B,
    profile: B,
    iso: Option<B>,
    env: HashMap<String, String>,
}

impl Report {
    fn raw(&mut self, s: &[u8]) {
        let private = disc_clean(s, 8192, self.iso.as_deref(), &self.env);
        let mut n = private.len();
        if self.buf.len() + n + 1 >= GL_REPORT_MAX {
            n = GL_REPORT_MAX - 1 - self.buf.len();
        }
        self.buf.extend_from_slice(&private[..n]);
    }

    /// `gl_sanitize`: the user's profile path becomes %USERPROFILE% and the user name `<user>`.
    fn sanitize(&self, input: &[u8], cap: usize) -> B {
        let mut out = Vec::new();
        let (plen, ulen) = (self.profile.len(), self.user.len());
        let mut i = 0;
        while i < input.len() && out.len() + 1 < cap {
            let mut hit = plen >= 4;
            let mut k = 0;
            while hit && k < plen {
                let mut a = input.get(i + k).copied().unwrap_or(0).to_ascii_lowercase();
                let mut b = self.profile[k].to_ascii_lowercase();
                if a == b'/' {
                    a = b'\\';
                }
                if b == b'/' {
                    b = b'\\';
                }
                if input.get(i + k).copied().unwrap_or(0) == 0 || a != b {
                    hit = false;
                }
                k += 1;
            }
            if hit {
                for &r in b"%USERPROFILE%" {
                    if out.len() + 1 < cap {
                        out.push(r);
                    }
                }
                i += plen;
                continue;
            }
            if ulen >= 3 && input.len() >= i + ulen && input[i..i + ulen].eq_ignore_ascii_case(&self.user) {
                for &r in b"<user>" {
                    if out.len() + 1 < cap {
                        out.push(r);
                    }
                }
                i += ulen;
                continue;
            }
            out.push(input[i]);
            i += 1;
        }
        out
    }

    /// `rep()`: format, redact the disc, sanitize the user, append.
    fn line(&mut self, text: &str) {
        let raw = copy_cap(text.as_bytes(), 1024);
        let private = disc_clean(&raw, 1200, self.iso.as_deref(), &self.env);
        let clean = self.sanitize(&private, 1200);
        self.raw(&clean);
    }
}

impl Logger {
    fn rep_symbols(&self, r: &mut Report, from: u32, host: &ReportHost) {
        let mut rva: Vec<u32> = Vec::new();
        let mut t = from;
        while t != self.ntail && rva.len() < 48 {
            let s = self.tail_line(t);
            let mut p = 0;
            while let Some(pos) = s[p..].windows(6).position(|w| w == b"rva 0x") {
                if rva.len() >= 48 {
                    break;
                }
                let at = p + pos + 6;
                let mut e = at;
                while e < s.len() && s[e].is_ascii_hexdigit() {
                    e += 1;
                }
                let a = u32::from_str_radix(std::str::from_utf8(&s[at..e]).unwrap_or("0"), 16).unwrap_or(0);
                if !rva.contains(&a) && a != 0 {
                    rva.push(a);
                }
                p = p + pos + 6;
            }
            t = t.wrapping_add(1);
        }
        if rva.is_empty() {
            return;
        }
        r.line("\n---- frames, symbolized from melee-pc.map ----\n");
        let Some(map) = &host.map else {
            r.line("(no melee-pc.map beside the exe)\n");
            return;
        };
        let lines: Vec<&[u8]> = map.split(|&c| c == b'\n').collect();
        // the first 8 lines must carry "Timestamp is <hex>" equal to the exe's link time
        let mut ok = false;
        for l in lines.iter().take(8) {
            if let Some(pos) = l.windows(13).position(|w| w == b"Timestamp is ") {
                let s = &l[pos + 13..];
                let e = s.iter().position(|c| !c.is_ascii_hexdigit()).unwrap_or(s.len());
                let v = u64::from_str_radix(std::str::from_utf8(&s[..e]).unwrap_or("0"), 16).unwrap_or(0) as u32;
                ok = v == host.link_stamp;
                break;
            }
        }
        if !ok {
            r.line("(melee-pc.map is from a different build - not symbolized)\n");
            return;
        }
        let mut best_addr = vec![0u32; rva.len()];
        let mut best_name: Vec<B> = vec![Vec::new(); rva.len()];
        for l in &lines {
            // " %x:%x %255s %x"
            let Some((sect, off, name, addr)) = parse_map_line(l) else { continue };
            let _ = off;
            if sect != 1 {
                continue;
            }
            for i in 0..rva.len() {
                if addr <= rva[i] && addr >= best_addr[i] {
                    best_addr[i] = addr;
                    best_name[i] = copy_cap(&name, 80);
                }
            }
        }
        for i in 0..rva.len() {
            if !best_name[i].is_empty() && rva[i].wrapping_sub(best_addr[i]) < 0x20000 {
                r.line(&format!("  0x{:08X}  {}+0x{:X}\n", rva[i], String::from_utf8_lossy(&best_name[i]), rva[i] - best_addr[i]));
            } else {
                r.line(&format!("  0x{:08X}  (not in a function: data, or a stale pointer)\n", rva[i]));
            }
        }
    }

    fn rep_ring(&self, r: &mut Report, title: &str, ring: &[B], cap: usize, n: u32) {
        let mut t = if n as usize > cap { n - cap as u32 } else { 0 };
        r.line(&format!("\n---- {title} ----\n"));
        if n == 0 {
            r.line("(none)\n");
        }
        while t < n {
            r.line(&format!("{}\n", String::from_utf8_lossy(&ring[(t as usize) % cap])));
            t += 1;
        }
    }

    /// `gl_write_report`: the compact report as bytes.
    pub fn write_report(&self, kind: &str, reason: Option<&str>, host: &ReportHost) -> B {
        let mut r = Report {
            buf: Vec::new(),
            user: host.user.clone(),
            profile: host.profile.clone(),
            iso: self.host.iso_path.clone(),
            env: self.host.env.clone(),
        };
        let first_tail = if self.ntail as usize > GL_TAIL { self.ntail - GL_TAIL as u32 } else { 0 };
        // header
        r.line("==== GD's Melee crash report ====\n");
        let (y, mo, d, h, mi, s) = host.time;
        r.line(&format!(
            "time:            {:04}-{:02}-{:02} {:02}:{:02}:{:02} (up {} s)\n",
            y,
            mo,
            d,
            h,
            mi,
            s,
            self.host.tick64.wrapping_sub(self.t0) / 1000
        ));
        r.line(&format!("exit path:       {kind}\n"));
        r.line(&format!("reason:          {}\n", reason.unwrap_or("(none)")));
        r.line(&format!(
            "during shutdown: {}\n",
            if self.shutting_down { "yes (the window had been closed; the game was tearing down)" } else { "no" }
        ));
        let ver = match &host.version_line {
            Some(l) => {
                let l = &l[..l.iter().position(|&c| c == b'\r' || c == b'\n').unwrap_or(l.len())];
                String::from_utf8_lossy(l).into_owned()
            }
            None => String::new(),
        };
        r.line(&format!("version:         {}\n", if !ver.is_empty() { ver.as_str() } else { "development build (no version.txt)" }));
        r.line(&format!("build id:        {:08X} (melee-pc.exe link time)\n", host.link_stamp));
        {
            let title = host.disc_title.as_ref().map(|t| String::from_utf8_lossy(&t[..t.iter().position(|&c| c == 0).unwrap_or(t.len())]).into_owned()).unwrap_or_default();
            let id = if !self.disc_id.is_empty() { String::from_utf8_lossy(&self.disc_id).into_owned() } else { "?".to_string() };
            r.line(&format!("disc:            {} \"{}\" ({})\n", id, title, if !self.disc_path.is_empty() { "<disc>" } else { "no disc yet" }));
        }
        r.line(&format!("mods mounted:    {}\n", if !self.mods.is_empty() { String::from_utf8_lossy(&self.mods).into_owned() } else { "none".to_string() }));
        r.line(&format!(
            "settings.cfg:    {}\n",
            if !host.settings_summary.is_empty() { String::from_utf8_lossy(&host.settings_summary).into_owned() } else { "(none)".to_string() }
        ));
        let cats = self.describe();
        r.line(&format!("log:             {} ({})\n", String::from_utf8_lossy(&cats), String::from_utf8_lossy(&self.spec)));
        {
            let mut env = Vec::new();
            for v in &host.melee_env {
                if v.starts_with(b"MELEE_") && env.len() + v.len() + 2 < 1024 {
                    if !env.is_empty() {
                        env.push(b' ');
                    }
                    env.extend_from_slice(v);
                }
            }
            r.line(&format!(
                "environment:     {}\n",
                if !env.is_empty() { String::from_utf8_lossy(&env).into_owned() } else { "(no MELEE_* variables)".to_string() }
            ));
        }
        r.line(&format!("os:              {}\n", String::from_utf8_lossy(&host.os_line)));

        // the fault: a few lines of lead-in, then everything since the crash began (capped)
        let mut fault_from = if self.crashing { self.crash_mark } else { self.ntail };
        fault_from = if fault_from > 12 { fault_from - 12 } else { 0 };
        if fault_from < first_tail {
            fault_from = first_tail;
        }
        r.line("\n---- fault ----\n");
        let mut t = fault_from;
        while t < self.ntail && r.buf.len() < 20 * 1024 {
            r.line(&format!("{}\n", String::from_utf8_lossy(self.tail_line(t))));
            t += 1;
        }
        self.rep_symbols(&mut r, fault_from, host);
        self.rep_ring(&mut r, "last scene lines", &self.scene, GL_SCENES, self.nscene);
        self.rep_ring(&mut r, "pad / adapter", &self.pad, GL_PADS, self.npad);
        self.rep_ring(&mut r, "last watchdog samples", &self.watch, GL_WATCH, self.nwatch);

        // the newest written lines that still fit, up to 200
        let budget = if GL_REPORT_MAX - 256 > r.buf.len() { GL_REPORT_MAX - 256 - r.buf.len() } else { 0 };
        let mut used = 0;
        let mut start = fault_from;
        while start > first_tail && fault_from - (start - 1) <= 200 {
            let len = self.tail_line(start - 1).len() + 1;
            if used + len > budget {
                break;
            }
            used += len;
            start -= 1;
        }
        r.line(&format!("\n---- last {} log lines before the fault ----\n", fault_from - start));
        let mut t = start;
        while t < fault_from {
            r.line(&format!("{}\n", String::from_utf8_lossy(self.tail_line(t))));
            t += 1;
        }
        r.line("---- end ----\n");
        r.buf
    }

    /// The `-full.log` header line of `gw_crash_report`.
    pub fn full_header(&self, kind: &str, reason: Option<&str>, host: &ReportHost) -> String {
        let (y, mo, d, h, mi, s) = host.time;
        format!(
            "==== crash {:04}-{:02}-{:02} {:02}:{:02}:{:02}  {}: {}{} ====\n",
            y,
            mo,
            d,
            h,
            mi,
            s,
            kind,
            reason.unwrap_or("(no reason)"),
            if self.shutting_down { " (during shutdown)" } else { "" }
        )
    }

    /// `crashlogs\crash-<time>` (the stem the two files share).
    pub fn crash_stem(host: &ReportHost) -> String {
        let (y, mo, d, h, mi, s) = host.time;
        format!("crashlogs\\crash-{:04}{:02}{:02}-{:02}{:02}{:02}", y, mo, d, h, mi, s)
    }
}

/// `sscanf(line, " %x:%x %255s %x", &sect, &off, name, &addr)` (4 conversions or None).
fn parse_map_line(l: &[u8]) -> Option<(u32, u32, B, u32)> {
    fn skip_ws(l: &[u8], mut i: usize) -> usize {
        while i < l.len() && matches!(l[i], b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c) {
            i += 1;
        }
        i
    }
    fn hex(l: &[u8], mut i: usize) -> Option<(u32, usize)> {
        i = skip_ws(l, i);
        let st = i;
        // %x accepts an optional 0x prefix
        if i + 2 < l.len() + 0 && l[i] == b'0' && (l[i + 1] == b'x' || l[i + 1] == b'X') && l.get(i + 2).map_or(false, |c| c.is_ascii_hexdigit()) {
            i += 2;
        }
        let ds = i;
        let mut v: u64 = 0;
        while i < l.len() && l[i].is_ascii_hexdigit() {
            v = (v << 4) | (l[i] as char).to_digit(16).unwrap() as u64;
            i += 1;
        }
        if i == ds {
            let _ = st;
            return None;
        }
        Some((v as u32, i))
    }
    let (sect, i) = hex(l, 0)?;
    if l.get(i) != Some(&b':') {
        return None;
    }
    let (off, i) = hex(l, i + 1)?;
    let i = skip_ws(l, i);
    let st = i;
    let mut e = i;
    while e < l.len() && !matches!(l[e], b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c) && e - st < 255 {
        e += 1;
    }
    if e == st {
        return None;
    }
    let name = l[st..e].to_vec();
    let (addr, _) = hex(l, e)?;
    Some((sect, off, name, addr))
}
