//! `settings.cfg`: the port's own settings file (port of pc/platform/gw_settings.c, the table,
//! environment override, load and save halves; widescreen and text entry are platform code).
//!
//! Format: one `key=value` per line, `#` comments; unknown keys are kept on save. Reads first use
//! a nonempty `MELEE_<ASCII uppercase key>`, then the saved value, then the caller's default.
//! Environment values are never stored or saved. Bounds are the C's: 512 rows, keys 31 bytes,
//! values 511 bytes, and the loader reads the file in `fgets` chunks of 547 bytes, so an overlong
//! line is split into several "lines" exactly as the C does.
//!
//! Strings are bytes, not UTF-8. The C opens the file in text mode: on Windows `\r\n` reads as
//! `\n`, a Ctrl-Z (0x1A) ends the file, and every `\n` is written as `\r\n`.

use std::path::{Path, PathBuf};

pub const ST_MAX: usize = 512;
pub const ST_KEY: usize = 32;
pub const ST_VAL: usize = 512;
const LINE_CAP: usize = ST_KEY + ST_VAL + 4;

pub const HEADER: &str =
    "# GD's Melee settings (written by the SETTINGS screens; environment variables win)\n";

type EnvFn = Box<dyn Fn(&str) -> Option<Vec<u8>> + Send>;

pub struct Settings {
    path: PathBuf,
    loaded: bool,
    rows: Vec<(Vec<u8>, Vec<u8>)>,
    save_ok: bool,
    env: EnvFn,
}

fn process_env(name: &str) -> Option<Vec<u8>> {
    let v = std::env::var_os(name)?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        // the C reads the ANSI environment: keep ASCII, replace the rest
        let s: Vec<u8> = v.encode_wide().map(|c| if c < 0x80 { c as u8 } else { b'?' }).collect();
        Some(s)
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(v.as_bytes().to_vec())
    }
}

/// Default location: `MELEE_SETTINGS_CFG`, else `settings.cfg` beside the executable.
pub fn default_path() -> PathBuf {
    if let Some(e) = process_env("MELEE_SETTINGS_CFG") {
        if !e.is_empty() {
            return PathBuf::from(String::from_utf8_lossy(&e).into_owned());
        }
    }
    match std::env::current_exe() {
        Ok(p) => p.with_file_name("settings.cfg"),
        Err(_) => PathBuf::from("settings.cfg"),
    }
}

/// C `atoi` as the Windows CRT does it (`strtol` with a 32-bit `long`: clamps).
pub fn atoi(s: &[u8]) -> i32 {
    let mut i = 0;
    while i < s.len() && matches!(s[i], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        neg = s[i] == b'-';
        i += 1;
    }
    let mut v: i64 = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        v = v * 10 + (s[i] - b'0') as i64;
        if v > 1 << 40 {
            v = 1 << 40;
        }
        i += 1;
    }
    if neg {
        v = -v;
    }
    v.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

fn trim_end(s: &[u8]) -> &[u8] {
    let mut e = s.len();
    while e > 0 && matches!(s[e - 1], b'\n' | b'\r' | b' ' | b'\t') {
        e -= 1;
    }
    &s[..e]
}

/// What `fopen(path, "r")` + `fgets` would see on this platform.
fn text_mode_read(raw: &[u8]) -> Vec<u8> {
    if cfg!(windows) {
        let mut out = Vec::with_capacity(raw.len());
        let mut i = 0;
        while i < raw.len() {
            let c = raw[i];
            if c == 0x1a {
                break;
            }
            if c == b'\r' && i + 1 < raw.len() && raw[i + 1] == b'\n' {
                i += 1;
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    } else {
        raw.to_vec()
    }
}

fn text_mode_write(s: &[u8]) -> Vec<u8> {
    if cfg!(windows) {
        let mut out = Vec::with_capacity(s.len() + s.len() / 16);
        for &c in s {
            if c == b'\n' {
                out.push(b'\r');
            }
            out.push(c);
        }
        out
    } else {
        s.to_vec()
    }
}

fn trunc(s: &[u8], cap: usize) -> Vec<u8> {
    // snprintf(dst, cap, "%s", s): at most cap-1 bytes, and %s stops at a NUL
    let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    s[..end.min(cap - 1)].to_vec()
}

impl Settings {
    /// Settings backed by `path`, reading the process environment.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self::with_env(path, Box::new(process_env))
    }

    pub fn open_default() -> Self {
        Self::new(default_path())
    }

    pub fn with_env(path: impl Into<PathBuf>, env: EnvFn) -> Self {
        Settings { path: path.into(), loaded: false, rows: Vec::new(), save_ok: true, env }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Point at another file without reloading (the C's `st.path`).
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.path = path.into();
    }

    /// Forget the table so the next access reads the file again (a fresh process).
    pub fn reload(&mut self) {
        self.loaded = false;
        self.rows.clear();
    }

    pub fn last_save_ok(&self) -> bool {
        self.save_ok
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn env_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        // fc_* are saved per-player choices: the environment never overrides them
        if key.starts_with(b"fc_") {
            return None;
        }
        // an overlong or empty key must not pick up another key's override
        let klen = key.iter().position(|&c| c == 0).unwrap_or(key.len());
        if klen == 0 || klen > ST_KEY - 1 {
            return None;
        }
        let mut name = String::from("MELEE_");
        for &c in &key[..klen] {
            name.push(if c.is_ascii_lowercase() { c.to_ascii_uppercase() as char } else { c as char });
        }
        match (self.env)(&name) {
            Some(v) if !v.is_empty() => Some(v),
            _ => None,
        }
    }

    fn load(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        self.rows.clear();
        let Ok(raw) = std::fs::read(&self.path) else { return };
        let data = text_mode_read(&raw);
        let mut pos = 0;
        while pos < data.len() && self.rows.len() < ST_MAX {
            // fgets(line, LINE_CAP): up to LINE_CAP-1 bytes, through a newline
            let lim = (pos + LINE_CAP - 1).min(data.len());
            let mut end = lim;
            if let Some(nl) = data[pos..lim].iter().position(|&c| c == b'\n') {
                end = pos + nl + 1;
            }
            let chunk = &data[pos..end];
            pos = end;
            let chunk = &chunk[..chunk.iter().position(|&c| c == 0).unwrap_or(chunk.len())];
            let mut k = 0;
            while k < chunk.len() && (chunk[k] == b' ' || chunk[k] == b'\t') {
                k += 1;
            }
            if k >= chunk.len() || matches!(chunk[k], b'#' | b'\n' | b'\r') {
                continue;
            }
            let Some(eq) = chunk[k..].iter().position(|&c| c == b'=') else { continue };
            let key = trim_end(&chunk[k..k + eq]);
            let val = trim_end(&chunk[k + eq + 1..]);
            self.rows.push((trunc(key, ST_KEY), trunc(val, ST_VAL)));
        }
    }

    fn find(&mut self, key: &[u8]) -> Option<usize> {
        self.load();
        let key = &key[..key.iter().position(|&c| c == 0).unwrap_or(key.len())];
        self.rows.iter().position(|(k, _)| k.as_slice() == key)
    }

    fn save(&mut self) {
        self.save_ok = false;
        let mut text = HEADER.as_bytes().to_vec();
        for (k, v) in &self.rows {
            text.extend_from_slice(k);
            text.push(b'=');
            text.extend_from_slice(v);
            text.push(b'\n');
        }
        if std::fs::write(&self.path, text_mode_write(&text)).is_ok() {
            self.save_ok = true;
        }
    }

    /// `gw_Settings_Str`: (found in environment or file, value truncated to `cap - 1` bytes).
    pub fn get_str(&mut self, key: &str, cap: usize, dflt: Option<&str>) -> (bool, Vec<u8>) {
        let (found, v): (bool, Vec<u8>) = if let Some(v) = self.env_value(key.as_bytes()) {
            (true, v)
        } else if let Some(i) = self.find(key.as_bytes()) {
            (true, self.rows[i].1.clone())
        } else {
            (false, dflt.unwrap_or("").as_bytes().to_vec())
        };
        if cap == 0 {
            return (found, Vec::new());
        }
        (found, trunc(&v, cap))
    }

    /// Convenience: the value as a String (lossy), or `dflt`.
    pub fn str_or(&mut self, key: &str, dflt: &str) -> String {
        let (_, v) = self.get_str(key, 4096, Some(dflt));
        String::from_utf8_lossy(&v).into_owned()
    }

    /// `gw_Settings_SetStr`: set and save; a failed save leaves the previous preference.
    pub fn set_str(&mut self, key: &str, value: &str) {
        let kb = key.as_bytes();
        let existing = self.find(kb);
        let added = existing.is_none();
        let i = match existing {
            Some(i) => i,
            None => {
                if self.rows.len() >= ST_MAX {
                    self.save_ok = false;
                    return;
                }
                self.rows.push((trunc(kb, ST_KEY), Vec::new()));
                self.rows.len() - 1
            }
        };
        let previous = self.rows[i].1.clone();
        self.rows[i].1 = trunc(value.as_bytes(), ST_VAL);
        self.save();
        if !self.save_ok {
            if added {
                self.rows.pop();
            } else {
                self.rows[i].1 = previous;
            }
        }
    }

    /// `gw_Settings_Int`.
    pub fn get_int(&mut self, key: &str, dflt: i32) -> i32 {
        if let Some(v) = self.env_value(key.as_bytes()) {
            return atoi(&v);
        }
        match self.find(key.as_bytes()) {
            Some(i) if !self.rows[i].1.is_empty() => atoi(&self.rows[i].1),
            _ => dflt,
        }
    }

    /// `gw_Settings_SetInt`.
    pub fn set_int(&mut self, key: &str, value: i32) {
        self.set_str(key, &value.to_string());
    }

    /// `gw_Settings_RemovePrefix`: drop every row whose key starts with `prefix` and save once.
    /// Returns whether the save succeeded (false for an empty or overlong prefix).
    pub fn remove_prefix(&mut self, prefix: &str) -> bool {
        let p = prefix.as_bytes();
        if p.is_empty() || p.len() >= ST_KEY {
            return false;
        }
        self.load();
        let backup = self.rows.clone();
        self.rows.retain(|(k, _)| !k.starts_with(p));
        self.save();
        if !self.save_ok {
            self.rows = backup;
        }
        self.save_ok
    }

    /// `gw_Settings_Summary`: every saved `key=value` on one line for a crash report, leaving out
    /// the player's name and the server. `cap` is the C buffer size (NUL included).
    pub fn summary(&mut self, cap: usize) -> (Vec<u8>, usize) {
        let mut out = Vec::new();
        let mut count = 0;
        if cap == 0 {
            return (out, 0);
        }
        self.load();
        for (k, v) in &self.rows {
            if k == b"name" || k == b"server" {
                continue;
            }
            let mut piece = Vec::new();
            if count > 0 {
                piece.push(b' ');
            }
            piece.extend_from_slice(k);
            piece.push(b'=');
            piece.extend_from_slice(v);
            if out.len() + piece.len() >= cap {
                // snprintf already wrote the part that fits (before the NUL) when it broke out
                let room = cap - 1 - out.len();
                out.extend_from_slice(&piece[..room]);
                break;
            }
            out.extend_from_slice(&piece);
            count += 1;
        }
        (out, count)
    }
}
