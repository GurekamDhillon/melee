//! The mods folder registry (port of pc/platform/gw_mods.c, gw_mods.h and the file overlay scan of
//! shim_dvd.c): which mods exist, mod.json, enabled.txt, requirement / conflict resolution, mount
//! order, the toggle API, the netplay mod-set fingerprint and `Describe` / `DiffDescribe`, and the
//! disc-path overlay a set of mounted mods produces over the vanilla disc.
//!
//! Text is bytes, as in the C (`char[]` fields truncated at their capacity); accessors give lossy
//! strings. Not ported: mod.json `menus` entries (the Atlas menu registry, `at_menus_parse`), the
//! mod-sync session journal, zip drops. Stage skins are modelled only as far as the conflict rule
//! needs (the retail stage table and `gw_Skin_PayloadMask`).
//!
//! Nothing read here is written except `enabled.txt` (`Save`). The mods are the user's files.

use std::path::{Path, PathBuf};

pub const MODS_MAX: usize = 256;
pub const ID_MAX: usize = 64;
pub const TEXT_MAX: usize = 128;
pub const LIST_MAX: usize = 8;

pub const MOD_ACTIVE: i32 = 0;
pub const MOD_OFF: i32 = 1;
pub const MOD_MISSING_DEP: i32 = 2;
pub const MOD_CONFLICT: i32 = 3;

pub type Bytes = Vec<u8>;

const FNV64_INIT: u64 = 1469598103934665603;

fn fnv64(mut h: u64, data: &[u8]) -> u64 {
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}

fn fnv64_str_lower(mut h: u64, s: &[u8]) -> u64 {
    for &c0 in s {
        let mut c = if c0 == b'\\' { b'/' } else { c0 };
        if c.is_ascii_uppercase() {
            c += b'a' - b'A';
        }
        h = fnv64(h, &[c]);
    }
    fnv64(h, &[0])
}

/// `copy_str`: at most `cap - 1` bytes, up to a NUL.
fn copy_str(src: &[u8], cap: usize) -> Bytes {
    let src = &src[..src.iter().position(|&c| c == 0).unwrap_or(src.len())];
    src[..src.len().min(cap - 1)].to_vec()
}

fn lower(c: u8) -> u8 {
    c.to_ascii_lowercase()
}

/// `_stricmp(a, b) == 0`
fn ieq(a: &[u8], b: &[u8]) -> bool {
    let a = &a[..a.iter().position(|&c| c == 0).unwrap_or(a.len())];
    let b = &b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())];
    a.len() == b.len() && a.iter().zip(b).all(|(&x, &y)| lower(x) == lower(y))
}

/// `_stricmp` ordering
fn icmp(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let la = a.iter().map(|&c| lower(c));
    let lb = b.iter().map(|&c| lower(c));
    la.cmp(lb)
}

fn lossy(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

// ---- the retail stage table (gw_stageskin.inc) --------------------------------------------------

const RETAIL: [(i32, &str); 29] = [
    (2, "GrIz.dat"),
    (3, "GrPs"),
    (4, "GrCs.dat"),
    (5, "GrKg.dat"),
    (6, "GrZe.dat"),
    (7, "GrCn"),
    (8, "GrSt.dat"),
    (9, "GrOt"),
    (10, "GrMc.dat"),
    (11, "GrRc.dat"),
    (12, "GrGd.dat"),
    (13, "GrGb.dat"),
    (14, "GrSh.dat"),
    (15, "GrKr.dat"),
    (16, "GrYt.dat"),
    (17, "GrGr.dat"),
    (18, "GrFs.dat"),
    (19, "GrI1.dat"),
    (20, "GrI2.dat"),
    (22, "GrVe"),
    (23, "GrPu.dat"),
    (24, "GrBb.dat"),
    (25, "GrIm.dat"),
    (27, "GrFz.dat"),
    (28, "GrOp.dat"),
    (29, "GrOy.dat"),
    (30, "GrOk.dat"),
    (31, "GrNBa.dat"),
    (32, "GrNLa.dat"),
];

/// `gw_Skin_FileRetail`: `<stem>[digits].<letters>` at the payload root -> its retail row.
pub fn skin_file_retail(path: &[u8]) -> i32 {
    let mut path = &path[..path.iter().position(|&c| c == 0).unwrap_or(path.len())];
    while !path.is_empty() && (path[0] == b'/' || path[0] == b'\\') {
        path = &path[1..];
    }
    if path.contains(&b'/') || path.contains(&b'\\') {
        return -1;
    }
    for (r, (_, file)) in RETAIL.iter().enumerate() {
        let fb = file.as_bytes();
        let mut n = fb.len();
        if n > 4 && ieq(&fb[n - 4..], b".dat") {
            n -= 4;
        }
        if path.len() < n || !path[..n].iter().zip(&fb[..n]).all(|(&a, &b)| lower(a) == lower(b)) {
            continue;
        }
        let mut p = n;
        while p < path.len() && path[p].is_ascii_digit() {
            p += 1;
        }
        if p >= path.len() || path[p] != b'.' || p + 1 >= path.len() {
            continue;
        }
        p += 1;
        while p < path.len() && path[p].is_ascii_alphabetic() {
            p += 1;
        }
        if p == path.len() {
            return r as i32;
        }
    }
    -1
}

/// `gw_Skin_PayloadMask`: bit r set for every retail stage a payload folder's root files belong to.
pub fn skin_payload_mask(dir: &Path) -> u32 {
    if dir.as_os_str().is_empty() {
        return 0;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    let mut mask = 0;
    for e in rd.flatten() {
        if e.file_type().map_or(true, |t| t.is_dir()) {
            continue;
        }
        let r = skin_file_retail(e.file_name().to_string_lossy().as_bytes());
        if r >= 0 {
            mask |= 1u32 << r;
        }
    }
    mask
}

// ---- mod.json -------------------------------------------------------------------------------------

/// A NUL-terminated view: reads past the end give 0, like the C string the parser walks.
struct Txt<'a>(&'a [u8]);
impl Txt<'_> {
    fn at(&self, i: usize) -> u8 {
        self.0.get(i).copied().unwrap_or(0)
    }
    fn skip_ws(&self, mut p: usize) -> usize {
        while matches!(self.at(p), b' ' | b'\t' | b'\r' | b'\n') {
            p += 1;
        }
        p
    }
    /// `json_string`: the unescaped text (at most `cap - 1` bytes kept) and the position after it.
    fn string(&self, mut p: usize, cap: usize) -> Option<(Bytes, usize)> {
        if self.at(p) != b'"' {
            return None;
        }
        p += 1;
        let mut out = Vec::new();
        let mut n = 0;
        while self.at(p) != 0 && self.at(p) != b'"' {
            let mut c = self.at(p);
            p += 1;
            if c == b'\\' {
                c = self.at(p);
                p += 1;
                match c {
                    b'n' => c = 10,
                    b't' => c = 9,
                    b'r' => c = 13,
                    b'b' => c = 8,
                    b'f' => c = 12,
                    b'u' => {
                        let mut v: u32 = 0;
                        let mut k = 0;
                        while k < 4 && self.at(p + k) != 0 {
                            let h = self.at(p + k) as i8 as i32;
                            let d = if h >= b'a' as i32 {
                                h - b'a' as i32 + 10
                            } else if h >= b'A' as i32 {
                                h - b'A' as i32 + 10
                            } else {
                                h - b'0' as i32
                            };
                            v = v.wrapping_mul(16).wrapping_add(d as u32);
                            k += 1;
                        }
                        p += k;
                        c = if v < 0x80 { v as u8 } else { b'?' };
                    }
                    0 => return None,
                    _ => {}
                }
            }
            if n + 1 < cap {
                out.push(c);
                n += 1;
            }
        }
        if self.at(p) != b'"' {
            return None;
        }
        // the C holds it as a NUL-terminated string
        out.truncate(out.iter().position(|&c| c == 0).unwrap_or(out.len()));
        Some((out, p + 1))
    }
    /// `json_skip_value`
    fn skip_value(&self, mut p: usize) -> usize {
        let mut depth = 0;
        p = self.skip_ws(p);
        loop {
            let c = self.at(p);
            if c == 0 {
                return p;
            }
            if c == b'"' {
                match self.string(p, 8) {
                    None => {
                        let mut q = p;
                        while self.at(q) != 0 {
                            q += 1;
                        }
                        return q;
                    }
                    Some((_, q)) => {
                        p = q;
                        if depth == 0 {
                            return p;
                        }
                        continue;
                    }
                }
            }
            if c == b'{' || c == b'[' {
                depth += 1;
            } else if c == b'}' || c == b']' {
                if depth == 0 {
                    return p;
                }
                depth -= 1;
                if depth == 0 {
                    return p + 1;
                }
            } else if depth == 0 && c == b',' {
                return p;
            }
            p += 1;
        }
    }
    fn atoi(&self, p: usize) -> i32 {
        let end = self.0.len();
        let s = if p < end { &self.0[p..] } else { &[][..] };
        crate::settings::atoi(&s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())])
    }
}

const ENGINE_FEATURES: [(&str, i32); 1] = [("pobj_palette", 1)];

#[derive(Clone, Debug, Default)]
pub struct Mod {
    pub id: Bytes,
    pub name: Bytes,
    pub version: Bytes,
    pub kind: Bytes,
    pub pack: Bytes,
    pub desc: Bytes,
    pub autostart: Bytes,
    pub hash: Bytes,
    pub source: Bytes,
    pub nucleus_post: i32,
    pub requires_text: Bytes,
    pub req: Vec<Bytes>,
    pub con: Vec<Bytes>,
    pub payload: PathBuf,
    pub payload_is_mod_dir: bool,
    /// in enabled.txt at boot (or no enabled.txt)
    pub enabled_boot: bool,
    /// pending, for the next boot
    pub enabled: bool,
    pub status: i32,
    pub status_text: Bytes,
    pub file_sum: u64,
    pub nfiles: i32,
    pub engine_miss: Bytes,
    pub skin_mask: u32,
}

impl Mod {
    pub fn id_str(&self) -> String {
        lossy(&self.id)
    }
    pub fn kind_str(&self) -> String {
        lossy(&self.kind)
    }
    pub fn status_str(&self) -> String {
        lossy(&self.status_text)
    }

    fn set_field(&mut self, key: &[u8], val: &[u8], log: &mut Vec<String>) {
        let v = |cap: usize| copy_str(val, cap);
        match key {
            b"name" => self.name = v(TEXT_MAX),
            b"version" => self.version = v(32),
            b"kind" => self.kind = v(16),
            b"pack" => self.pack = v(32),
            b"description" => self.desc = v(TEXT_MAX),
            b"autostart" => self.autostart = v(160),
            b"hash" => self.hash = v(80),
            b"source" => self.source = v(12),
            b"id" if !ieq(val, &self.id) => {
                log.push(format!("gw: mods: {}/mod.json says id \"{}\" - the folder name wins", lossy(&self.id), lossy(val)));
            }
            _ => {}
        }
    }

    fn add_list(&mut self, key: &[u8], val: &[u8]) {
        if key == b"requires" && self.req.len() < LIST_MAX {
            self.req.push(copy_str(val, ID_MAX));
        } else if key == b"conflicts" && self.con.len() < LIST_MAX {
            self.con.push(copy_str(val, ID_MAX));
        }
    }

    fn engine_need(&mut self, feature: &[u8], version: i32) {
        let mut have = 0;
        for (n, v) in ENGINE_FEATURES {
            if n.as_bytes() == feature {
                have = v;
            }
        }
        if version > have && self.engine_miss.is_empty() {
            let m = format!(
                "needs engine {} {} (this build: {}{})",
                lossy(feature),
                version,
                if have != 0 { "" } else { "none, " },
                have
            );
            self.engine_miss = copy_str(m.as_bytes(), TEXT_MAX);
        }
    }

    /// `mod_parse_json`: false on a syntax error (fields read before it are kept).
    fn parse_json(&mut self, text: &[u8], log: &mut Vec<String>) -> bool {
        let t = Txt(text);
        let mut p = t.skip_ws(0);
        if t.at(p) == 0xEF && t.at(p + 1) == 0xBB && t.at(p + 2) == 0xBF {
            p = t.skip_ws(p + 3);
        }
        if t.at(p) != b'{' {
            return false;
        }
        p = t.skip_ws(p + 1);
        while t.at(p) != 0 && t.at(p) != b'}' {
            let Some((key, np)) = t.string(p, 32) else { return false };
            p = t.skip_ws(np);
            if t.at(p) != b':' {
                return false;
            }
            p = t.skip_ws(p + 1);
            if t.at(p) == b'"' {
                let Some((val, np)) = t.string(p, TEXT_MAX) else { return false };
                p = np;
                self.set_field(&key, &val, log);
            } else if t.at(p) == b'[' {
                p = t.skip_ws(p + 1);
                while t.at(p) != 0 && t.at(p) != b']' {
                    if t.at(p) == b'"' {
                        let Some((val, np)) = t.string(p, TEXT_MAX) else { return false };
                        p = np;
                        self.add_list(&key, &val);
                    } else {
                        p = t.skip_value(p);
                    }
                    p = t.skip_ws(p);
                    if t.at(p) == b',' {
                        p = t.skip_ws(p + 1);
                    }
                }
                if t.at(p) != b']' {
                    return false;
                }
                p += 1;
            } else if t.at(p) == b'{' && key == b"engine" {
                p = t.skip_ws(p + 1);
                while t.at(p) != 0 && t.at(p) != b'}' {
                    let Some((feat, np)) = t.string(p, 48) else { return false };
                    p = t.skip_ws(np);
                    if t.at(p) != b':' {
                        return false;
                    }
                    p = t.skip_ws(p + 1);
                    self.engine_need(&feat, t.atoi(p));
                    p = t.skip_value(p);
                    p = t.skip_ws(p);
                    if t.at(p) == b',' {
                        p = t.skip_ws(p + 1);
                    }
                }
                if t.at(p) != b'}' {
                    return false;
                }
                p += 1;
            } else if t.at(p) == b'{' && key == b"nucleus" {
                p = t.skip_ws(p + 1);
                while t.at(p) != 0 && t.at(p) != b'}' {
                    let Some((nk, np)) = t.string(p, 32) else { return false };
                    p = t.skip_ws(np);
                    if t.at(p) != b':' {
                        return false;
                    }
                    p = t.skip_ws(p + 1);
                    if nk == b"id" && t.at(p).is_ascii_digit() {
                        self.nucleus_post = t.atoi(p);
                    }
                    p = t.skip_value(p);
                    p = t.skip_ws(p);
                    if t.at(p) == b',' {
                        p = t.skip_ws(p + 1);
                    }
                }
                if t.at(p) != b'}' {
                    return false;
                }
                p += 1;
            } else {
                p = t.skip_value(p);
            }
            p = t.skip_ws(p);
            if t.at(p) == b',' {
                p = t.skip_ws(p + 1);
            }
        }
        t.at(p) == b'}'
    }
}

/// `read_text_file`: None when missing, unreadable or over 1 MiB. The result is what the C string
/// functions see (up to the first NUL).
fn read_text_file(path: &Path) -> Option<Bytes> {
    let d = std::fs::read(path).ok()?;
    if d.len() > 1 << 20 {
        return None;
    }
    Some(d)
}

#[derive(Default)]
pub struct ModSet {
    pub dir: PathBuf,
    pub have_dir: bool,
    /// enabled.txt existed
    pub explicit_set: bool,
    /// MELEE_MODS=0
    pub mods_off: bool,
    pub mods: Vec<Mod>,
    /// mount order: indices of ACTIVE mods
    pub order: Vec<usize>,
    describe: Bytes,
    /// the `gw_log` lines so far
    pub log: Vec<String>,
}

fn kind_rank(m: &Mod) -> i32 {
    match m.kind.as_slice() {
        b"base" => 0,
        b"misc" | b"script" => 1,
        _ => 2,
    }
}

fn mods_conflict(a: &Mod, b: &Mod) -> bool {
    if a.skin_mask & b.skin_mask != 0 {
        return false;
    }
    if a.con.iter().any(|c| ieq(c, &b.id)) || b.con.iter().any(|c| ieq(c, &a.id)) {
        return true;
    }
    kind_rank(a) == 0 && kind_rank(b) == 0
}

impl ModSet {
    pub fn find(&self, id: &[u8]) -> Option<usize> {
        self.mods.iter().position(|m| ieq(&m.id, id))
    }

    /// `set_load`: scan `dir`; returns the number of mods found.
    pub fn load(dir: &Path, mods_off: bool) -> ModSet {
        Self::load_with(dir, mods_off, None)
    }

    /// `load` with the enabled set given by the caller (a `--enable id,id` option) instead of
    /// enabled.txt: exactly the listed ids (case-insensitive) are enabled at boot.
    pub fn load_with(dir: &Path, mods_off: bool, enabled: Option<&[String]>) -> ModSet {
        let mut s = ModSet { mods_off, ..Default::default() };
        if dir.as_os_str().is_empty() {
            return s;
        }
        s.dir = dir.to_path_buf();
        s.have_dir = true;
        let Ok(rd) = std::fs::read_dir(dir) else { return s };
        let mut ids: Vec<Bytes> = Vec::new();
        for e in rd.flatten() {
            if !std::fs::metadata(e.path()).map_or(false, |t| t.is_dir()) {
                continue;
            }
            let name = e.file_name().to_string_lossy().into_owned().into_bytes();
            if name.first() == Some(&b'.') {
                continue;
            }
            if ieq(&name, b"targettest") || ieq(&name, b"_incoming") {
                continue;
            }
            if name.len() >= ID_MAX {
                s.log.push(format!("gw: mods: folder name {} is too long for a mod id - skipped", lossy(&name)));
                continue;
            }
            if ids.len() >= MODS_MAX {
                s.log.push(format!("gw: mods: more than {} mods - {} and later skipped", MODS_MAX, lossy(&name)));
                break;
            }
            ids.push(name);
        }
        ids.sort_by(|a, b| icmp(a, b));
        for id in ids {
            let mut m = Mod { id: id.clone(), ..Default::default() };
            let moddir = dir.join(lossy(&id));
            s.load_meta(&mut m, &moddir);
            s.mods.push(m);
        }
        match enabled {
            None => s.read_enabled(),
            Some(ids) => {
                s.explicit_set = true;
                for m in &mut s.mods {
                    m.enabled_boot = ids.iter().any(|i| ieq(i.as_bytes(), &m.id));
                }
            }
        }
        s.resolve();
        s
    }

    fn load_meta(&mut self, m: &mut Mod, moddir: &Path) {
        let files = moddir.join("files");
        if files.is_dir() {
            m.payload = files;
            m.payload_is_mod_dir = false;
        } else {
            m.payload = moddir.to_path_buf();
            m.payload_is_mod_dir = true;
        }
        if let Some(text) = read_text_file(&moddir.join("mod.json")) {
            let mut log = Vec::new();
            if !m.parse_json(&text, &mut log) {
                log.push(format!("gw: mods: {}/mod.json is not valid JSON - read what parsed, ignored the rest", lossy(&m.id)));
            }
            // (the menus entries of a valid mod.json are the Atlas registry's: not ported)
            self.log.extend(log);
        }
        if m.name.is_empty() {
            m.name = copy_str(&m.id, TEXT_MAX);
        }
        let known = [&b"base"[..], b"fighter", b"stage", b"misc", b"script", b"skin"];
        if m.kind.is_empty() || !known.contains(&m.kind.as_slice()) {
            if !m.kind.is_empty() {
                self.log.push(format!("gw: mods: {}: unknown kind \"{}\" - treated as misc", lossy(&m.id), lossy(&m.kind)));
            }
            m.kind = b"misc".to_vec();
        }
        m.skin_mask = if m.kind == b"stage" { skin_payload_mask(&m.payload) } else { 0 };
        let mut rt = Vec::new();
        for (i, r) in m.req.iter().enumerate() {
            let piece = [if i > 0 { &b","[..] } else { &b""[..] }, r].concat();
            let room = TEXT_MAX - 1 - rt.len();
            rt.extend_from_slice(&piece[..piece.len().min(room)]);
        }
        m.requires_text = rt;
    }

    fn read_enabled(&mut self) {
        let text = read_text_file(&self.dir.join("enabled.txt"));
        self.explicit_set = text.is_some();
        for m in &mut self.mods {
            m.enabled_boot = text.is_none();
        }
        let Some(text) = text else { return };
        // the C walks a NUL-terminated string
        let text = &text[..text.iter().position(|&c| c == 0).unwrap_or(text.len())];
        let mut pos = 0;
        while pos < text.len() {
            let nl = text[pos..].iter().position(|&c| c == b'\n');
            let (mut line, next) = match nl {
                Some(n) => (&text[pos..pos + n], Some(pos + n + 1)),
                None => (&text[pos..], None),
            };
            if let Some(h) = line.iter().position(|&c| c == b'#') {
                line = &line[..h];
            }
            let mut a = 0;
            loop {
                let c = line.get(a).copied().unwrap_or(0);
                let bom = line.get(a) == Some(&0xEF) && line.get(a + 1) == Some(&0xBB) && line.get(a + 2) == Some(&0xBF);
                if c == b' ' || c == b'\t' || c == b'\r' || bom {
                    a += if c == 0xEF { 3 } else { 1 };
                } else {
                    break;
                }
            }
            let mut line = &line[a.min(line.len())..];
            while let Some(&l) = line.last() {
                if l == b' ' || l == b'\t' || l == b'\r' {
                    line = &line[..line.len() - 1];
                } else {
                    break;
                }
            }
            if !line.is_empty() {
                match self.find(line) {
                    None => self.log.push(format!(
                        "gw: mods: enabled.txt names \"{}\", which is not in {} - ignored",
                        lossy(line),
                        self.dir.display()
                    )),
                    Some(i) => self.mods[i].enabled_boot = true,
                }
            }
            match next {
                Some(n) => pos = n,
                None => break,
            }
            if pos >= text.len() {
                break;
            }
        }
    }

    /// `set_resolve`
    fn resolve(&mut self) {
        let n = self.mods.len();
        let mut keep = vec![false; n];
        for i in 0..n {
            let off = self.mods_off;
            let m = &mut self.mods[i];
            keep[i] = m.enabled_boot && !off;
            m.status = if keep[i] { MOD_ACTIVE } else { MOD_OFF };
            m.status_text = copy_str(
                if off {
                    b"all mods off (MELEE_MODS=0)"
                } else if keep[i] {
                    b"on"
                } else {
                    b"off"
                },
                TEXT_MAX,
            );
            if keep[i] && !m.engine_miss.is_empty() {
                keep[i] = false;
                m.status = MOD_MISSING_DEP;
                m.status_text = copy_str(&m.engine_miss, TEXT_MAX);
                self.log.push(format!("gw: mods: {} not mounted: {}", lossy(&m.id), lossy(&m.engine_miss)));
            }
        }
        let mut by_rank = Vec::new();
        for k in 0..3 {
            for i in 0..n {
                if kind_rank(&self.mods[i]) == k {
                    by_rank.push(i);
                }
            }
        }
        loop {
            let mut changed = false;
            for i in 0..n {
                if !keep[i] {
                    continue;
                }
                for j in 0..self.mods[i].req.len() {
                    let r = self.find(&self.mods[i].req[j].clone());
                    if r.map_or(true, |r| !keep[r]) {
                        keep[i] = false;
                        let m = &mut self.mods[i];
                        m.status = MOD_MISSING_DEP;
                        let t = format!("needs {}{}", lossy(&m.req[j]), if r.is_none() { " (not installed)" } else { "" });
                        m.status_text = copy_str(t.as_bytes(), TEXT_MAX);
                        changed = true;
                        break;
                    }
                }
            }
            for i in 0..by_rank.len() {
                let a = by_rank[i];
                if !keep[a] {
                    continue;
                }
                for j in 0..i {
                    let b = by_rank[j];
                    if keep[b] && mods_conflict(&self.mods[a], &self.mods[b]) {
                        keep[a] = false;
                        let t = format!("conflicts with {}", lossy(&self.mods[b].id));
                        self.mods[a].status = MOD_CONFLICT;
                        self.mods[a].status_text = copy_str(t.as_bytes(), TEXT_MAX);
                        changed = true;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        self.order.clear();
        let mut placed = vec![false; n];
        let mut progress = true;
        while progress {
            progress = false;
            for &a in &by_rank {
                if !keep[a] || placed[a] {
                    continue;
                }
                let mut ok = true;
                for r in &self.mods[a].req {
                    match self.find(r) {
                        Some(r) if placed[r] => {}
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    placed[a] = true;
                    self.order.push(a);
                    progress = true;
                    break;
                }
            }
        }
        for i in 0..n {
            if keep[i] && !placed[i] {
                self.mods[i].status = MOD_MISSING_DEP;
                self.mods[i].status_text = b"requirement cycle".to_vec();
            }
        }
        for m in &mut self.mods {
            m.enabled = m.enabled_boot;
        }
    }

    /// `set_log`: the boot summary lines.
    pub fn summary_log(&self) -> Vec<String> {
        let mut v = vec![format!(
            "gw: mods: {} - {} mod(s), enabled set from {}",
            self.dir.display(),
            self.mods.len(),
            if self.explicit_set { "enabled.txt" } else { "none (all enabled)" }
        )];
        for m in &self.mods {
            v.push(format!(
                "gw: mods:   {:<24} {:<7} {:<8} {}",
                lossy(&m.id),
                lossy(&m.kind),
                if m.version.is_empty() { "-".to_string() } else { lossy(&m.version) },
                lossy(&m.status_text)
            ));
        }
        v
    }

    /// `set_enable`: toggle mod `i` for the next boot (requirements follow it on, conflicts and
    /// dependents off). Returns how many mods changed state.
    pub fn set_enable(&mut self, i: usize, on: bool) -> i32 {
        self.enable_depth(i as i64, on, 0)
    }

    fn enable_depth(&mut self, i: i64, on: bool, depth: usize) -> i32 {
        if i < 0 || i as usize >= self.mods.len() || depth > MODS_MAX {
            return 0;
        }
        let i = i as usize;
        let mut changed = 0;
        if !on {
            if !self.mods[i].enabled {
                return 0;
            }
            self.mods[i].enabled = false;
            changed = 1;
            let id = self.mods[i].id.clone();
            for j in 0..self.mods.len() {
                if self.mods[j].req.iter().any(|r| ieq(r, &id)) {
                    changed += self.enable_depth(j as i64, false, depth + 1);
                }
            }
            return changed;
        }
        if !self.mods[i].enabled {
            self.mods[i].enabled = true;
            changed = 1;
        }
        for k in 0..self.mods[i].req.len() {
            let req = self.mods[i].req[k].clone();
            if let Some(r) = self.find(&req) {
                if !self.mods[r].enabled {
                    changed += self.enable_depth(r as i64, true, depth + 1);
                }
            }
        }
        for j in 0..self.mods.len() {
            if j != i && self.mods[j].enabled && mods_conflict(&self.mods[i], &self.mods[j]) {
                changed += self.enable_depth(j as i64, false, depth + 1);
            }
        }
        changed
    }

    /// `gw_Mods_SetEnabled`, with its log line.
    pub fn set_enabled(&mut self, i: usize, on: bool) -> i32 {
        let n = self.set_enable(i, on);
        if n > 0 {
            self.log.push(format!("gw: mods: {} {} for the next boot ({} change(s))", self.mods[i].id_str(), if on { "on" } else { "off" }, n));
        }
        n
    }

    pub fn restart_needed(&self) -> bool {
        self.mods.iter().any(|m| m.enabled != m.enabled_boot)
    }

    /// `set_save`: write enabled.txt for the next boot (0 = ok, -1 = failed).
    pub fn save(&mut self) -> i32 {
        if !self.have_dir {
            return -1;
        }
        let _ = std::fs::create_dir(&self.dir);
        let path = self.dir.join("enabled.txt");
        let tmp = self.dir.join("enabled.txt.tmp");
        let nl = "\n"; // "wb": no text-mode translation
        let mut text = format!(
            "# Mods enabled at the next boot, one folder name per line. Written by the game's{nl}# mods menu; hand edits are fine. Delete this file to enable every mod.{nl}"
        )
        .into_bytes();
        for m in &self.mods {
            if m.enabled {
                text.extend_from_slice(&m.id);
                text.extend_from_slice(nl.as_bytes());
            }
        }
        if std::fs::write(&tmp, &text).is_err() {
            self.log.push(format!("gw: mods: cannot write {}", tmp.display()));
            return -1;
        }
        if std::fs::rename(&tmp, &path).is_err() {
            self.log.push(format!("gw: mods: cannot replace {}", path.display()));
            let _ = std::fs::remove_file(&tmp);
            return -1;
        }
        self.explicit_set = true;
        self.log.push(format!("gw: mods: wrote {}", path.display()));
        0
    }

    // ---- the overlay's report and the fingerprint ---------------------------------------------

    /// `gw_Mods_NoteFile`: the overlay reports every disc path it finally mounted for mod `i`.
    pub fn note_file(&mut self, i: usize, disc_path: &[u8], size: u32) {
        if i >= self.mods.len() {
            return;
        }
        let mut p = disc_path;
        while !p.is_empty() && (p[0] == b'/' || p[0] == b'\\') {
            p = &p[1..];
        }
        let mut h = fnv64_str_lower(FNV64_INIT, p);
        h = fnv64(h, &size.to_le_bytes());
        let m = &mut self.mods[i];
        m.file_sum = m.file_sum.wrapping_add(h);
        m.nfiles += 1;
    }

    /// `mod_digest`
    pub fn digest(&self, i: usize) -> u64 {
        let m = &self.mods[i];
        let mut h = FNV64_INIT;
        h = fnv64_str_lower(h, &m.id);
        let mut v = m.version.clone();
        v.push(0);
        h = fnv64(h, &v);
        let mut x = m.hash.clone();
        x.push(0);
        h = fnv64(h, &x);
        h = fnv64(h, &m.file_sum.to_le_bytes());
        h = fnv64(h, &m.nfiles.to_le_bytes());
        h
    }

    fn active_sorted(&self) -> Vec<usize> {
        (0..self.mods.len()).filter(|&i| self.mods[i].status == MOD_ACTIVE).collect()
    }

    /// `set_fingerprint`: 0 when no mod is mounted.
    pub fn fingerprint(&self) -> u64 {
        let idx = self.active_sorted();
        if idx.is_empty() {
            return 0;
        }
        let mut h = FNV64_INIT;
        for &i in &idx {
            let d = self.digest(i);
            h = fnv64_str_lower(h, &self.mods[i].id);
            h = fnv64(h, &d.to_le_bytes());
        }
        if h != 0 {
            h
        } else {
            1
        }
    }

    /// `set_describe`: "id#hhhh,id#hhhh" (sorted by id), "" when none.
    pub fn describe(&mut self) -> String {
        let idx = self.active_sorted();
        let mut out = String::new();
        for (n, &i) in idx.iter().enumerate() {
            let d = self.digest(i);
            let short = ((d ^ (d >> 16) ^ (d >> 32) ^ (d >> 48)) & 0xFFFF) as u32;
            let piece = format!("{}{}#{:04x}", if n > 0 { "," } else { "" }, self.mods[i].id_str(), short);
            if out.len() + piece.len() >= 2048 {
                // snprintf wrote the part that fit before the C broke out
                let room = 2047 - out.len();
                out += &piece[..room.min(piece.len())];
                break;
            }
            out += &piece;
        }
        self.describe = out.clone().into_bytes();
        out
    }

    /// `gw_Mods_WinnerOf`: the mod that wins a disc path (the last mounted one with the file).
    pub fn winner_of(&self, disc_path: &str) -> Option<String> {
        for &i in self.order.iter().rev() {
            let m = &self.mods[i];
            if m.payload.as_os_str().is_empty() {
                continue;
            }
            let p = PathBuf::from(format!("{}\\{}", m.payload.to_string_lossy(), disc_path.replace('/', "\\")));
            if p.is_file() {
                return Some(if m.name.is_empty() { m.id_str() } else { lossy(&m.name) });
            }
        }
        None
    }
}

/// `strtoul(s, NULL, 16)` truncated to 32 bits (the C stores it in an `unsigned`).
fn strtoul16(s: &[u8]) -> u32 {
    let mut i = 0;
    while i < s.len() && matches!(s[i], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
        i += 1;
    }
    let mut neg = false;
    if i < s.len() && (s[i] == b'+' || s[i] == b'-') {
        neg = s[i] == b'-';
        i += 1;
    }
    if i + 2 < s.len() + 0 && s[i] == b'0' && (s[i + 1] == b'x' || s[i + 1] == b'X') && s[i + 2].is_ascii_hexdigit() {
        i += 2;
    }
    let mut v: u64 = 0;
    let mut sat = false;
    while i < s.len() && s[i].is_ascii_hexdigit() {
        let d = (s[i] as char).to_digit(16).unwrap() as u64;
        if v > (u32::MAX as u64 - d) / 16 {
            sat = true;
        }
        v = (v * 16 + d).min(u32::MAX as u64 + 1);
        i += 1;
    }
    let v = if sat { u32::MAX } else { v as u32 };
    if neg {
        v.wrapping_neg()
    } else {
        v
    }
}

/// `diff_describe`: compare a peer's Describe string with ours; (text, number of differing mods).
pub fn diff_describe(mine: &str, peer: &str, cap: usize) -> (String, i32) {
    fn parse(d: &str) -> (Vec<(String, u32)>, bool) {
        let mut v = Vec::new();
        let mut truncated = false;
        let b = d.as_bytes();
        let mut p = 0;
        while p < b.len() && v.len() < MODS_MAX {
            let comma = b[p..].iter().position(|&c| c == b',');
            let len = comma.unwrap_or(b.len() - p);
            let piece = &b[p..p + len];
            if len >= 3 && &piece[len - 3..] == b"..." {
                truncated = true;
                break;
            }
            let hash = piece.iter().position(|&c| c == b'#');
            let idlen = hash.unwrap_or(len);
            if idlen > 0 && idlen < ID_MAX {
                let h = hash.map_or(0, |h| strtoul16(&b[p + h + 1..]));
                v.push((lossy(&piece[..idlen]), h));
            }
            match comma {
                Some(c) => p += c + 1,
                None => break,
            }
        }
        (v, truncated)
    }
    let (mine, _mt) = parse(mine);
    let (peer, pt) = parse(peer);
    let (mut lack, mut extra, mut files) = (String::new(), String::new(), String::new());
    let mut ndiff = 0;
    // the three lists are 512-byte buffers in the C (append stops at 511)
    let list_add = |l: &mut String, t: &str, sep: bool| {
        let mut app = |t: &str| {
            if l.len() < 511 {
                let room = 511 - l.len();
                l.push_str(&t[..t.len().min(room)]);
            }
        };
        if sep {
            app(", ");
        }
        app(t);
    };
    for (id, h) in &peer {
        let mut found: Option<usize> = None;
        for (j, (mid, _)) in mine.iter().enumerate() {
            if ieq(id.as_bytes(), mid.as_bytes()) {
                found = Some(j);
            }
        }
        match found {
            None => {
                let sep = !lack.is_empty();
                list_add(&mut lack, id, sep);
                ndiff += 1;
            }
            Some(j) if mine[j].1 != *h => {
                let sep = !files.is_empty();
                list_add(&mut files, id, sep);
                ndiff += 1;
            }
            _ => {}
        }
    }
    if !pt {
        for (mid, _) in &mine {
            if !peer.iter().any(|(id, _)| ieq(id.as_bytes(), mid.as_bytes())) {
                let sep = !extra.is_empty();
                list_add(&mut extra, mid, sep);
                ndiff += 1;
            }
        }
    }
    let mut out = String::new();
    let append = |out: &mut String, t: &str| {
        // the C's append(): only while the buffer has room
        if out.len() < cap.saturating_sub(1) {
            let room = cap - 1 - out.len();
            out.push_str(&t[..t.len().min(room)]);
        }
    };
    if !lack.is_empty() {
        append(&mut out, "you lack ");
        append(&mut out, &lack);
    }
    if !extra.is_empty() {
        let e = out.is_empty();
        append(&mut out, if e { "peer lacks " } else { "; peer lacks " });
        append(&mut out, &extra);
    }
    if !files.is_empty() {
        let e = out.is_empty();
        append(&mut out, if e { "different files in " } else { "; different files in " });
        append(&mut out, &files);
    }
    if pt {
        let e = out.is_empty();
        append(&mut out, if e { "peer list truncated" } else { " (peer list truncated)" });
    }
    if out.is_empty() {
        append(&mut out, "same mods");
    }
    (out, ndiff)
}

/// `gw_Mods_JsonEngineMiss`: does this mod.json need an engine feature this build lacks?
pub fn json_engine_miss(json: &[u8]) -> Option<String> {
    let mut m = Mod { id: b"bundle".to_vec(), ..Default::default() };
    let mut log = Vec::new();
    m.parse_json(json, &mut log);
    if m.engine_miss.is_empty() {
        None
    } else {
        Some(lossy(&m.engine_miss))
    }
}

// ---- the overlay (shim_dvd.c: gw_mods_scan / gw_mod_fingerprint_size / gw_mods_load) ---------------

#[derive(Clone, Debug)]
pub struct Provider {
    pub host: PathBuf,
    pub mod_id: String,
    pub mod_index: usize,
    pub size: u32,
}

#[derive(Clone, Debug)]
pub struct OverlayFile {
    /// disc path, '/'-separated, no leading slash
    pub disc: String,
    pub host: PathBuf,
    pub mod_id: String,
    pub mod_index: usize,
    pub size: u32,
    /// every mod's copy of the path, in mount order (at most 48)
    pub prov: Vec<Provider>,
}

pub const OVERLAY_MAX_FILES: usize = 4096;
const PROV_MAX: usize = 48;

/// The disc paths the mounted mods provide over the vanilla disc.
#[derive(Default)]
pub struct Overlay {
    pub files: Vec<OverlayFile>,
    pub log: Vec<String>,
}

fn path_ieq(a: &str, b: &str) -> bool {
    let n = |s: &str| s.bytes().map(|c| if c == b'\\' { b'/' } else { lower(c) }).collect::<Vec<u8>>();
    n(a) == n(b)
}

/// `gw_mod_fingerprint_size`: a text file checked out with CRLF line ends is the same mod as its
/// LF copy, so the CR of every CR LF pair is not counted; binary files count their exact size.
pub fn fingerprint_size(host: &Path, size: u32) -> u32 {
    const TEXT: [&str; 13] = [".lua", ".json", ".md", ".txt", ".wgsl", ".csv", ".cfg", ".toml", ".xml", ".glsl", ".yml", ".yaml", ".ini"];
    let h = host.to_string_lossy().to_ascii_lowercase();
    if !TEXT.iter().any(|t| h.ends_with(t)) {
        return size;
    }
    let Ok(d) = std::fs::read(host) else { return size };
    let mut crlf = 0u32;
    let mut prev_cr = false;
    for &c in &d {
        if c == b'\n' && prev_cr {
            crlf += 1;
        }
        prev_cr = c == b'\r';
    }
    size.wrapping_sub(crlf)
}

impl Overlay {
    /// `gw_mods_load` without the disc: scan every mounting mod in mount order, then report each
    /// final path to `set.note_file`.
    pub fn mount(set: &mut ModSet) -> Overlay {
        let mut o = Overlay::default();
        if set.order.is_empty() {
            if !set.mods.is_empty() {
                o.log.push(format!("gw: mods: none of the {} installed mods is mounting", set.mods.len()));
            }
            return o;
        }
        for k in 0..set.order.len() {
            let idx = set.order[k];
            let before = o.files.len();
            let (id, payload, skip_meta) = (set.mods[idx].id_str(), set.mods[idx].payload.clone(), set.mods[idx].payload_is_mod_dir);
            o.scan(idx, &id, &payload, "", skip_meta);
            o.log.push(format!("gw: mods: {} ({} new paths)", id, o.files.len() - before));
        }
        for f in &o.files {
            set.note_file(f.mod_index, f.disc.as_bytes(), fingerprint_size(&f.host, f.size));
        }
        o
    }

    fn by_path(&self, path: &str) -> Option<usize> {
        let path = path.trim_start_matches(['/', '\\']);
        self.files.iter().position(|f| path_ieq(&f.disc, path))
    }

    fn scan(&mut self, mod_index: usize, mod_id: &str, host_dir: &Path, rel: &str, skip_meta: bool) {
        let Ok(rd) = std::fs::read_dir(host_dir) else { return };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if skip_meta && rel.is_empty() && name.eq_ignore_ascii_case("mod.json") {
                continue;
            }
            if skip_meta && rel.is_empty() && name.eq_ignore_ascii_case("scripts") {
                continue;
            }
            let host = host_dir.join(&name);
            let disc = format!("{rel}{name}");
            let Ok(md) = e.metadata() else { continue };
            if md.is_dir() {
                self.scan(mod_index, mod_id, &host, &format!("{disc}/"), skip_meta);
                continue;
            }
            if md.len() >> 32 != 0 {
                self.log.push(format!("gw: mods: {mod_id}/{disc} is over 4 GiB - skipped"));
                continue;
            }
            let size = md.len() as u32;
            let idx = match self.by_path(&disc) {
                Some(i) => {
                    self.log.push(format!("gw: mods: {} overrides {}'s /{}", mod_id, self.files[i].mod_id, disc));
                    i
                }
                None => {
                    if self.files.len() >= OVERLAY_MAX_FILES {
                        self.log.push(format!("gw: mods: more than {OVERLAY_MAX_FILES} files - /{disc} skipped"));
                        continue;
                    }
                    self.files.push(OverlayFile { disc: disc.clone(), host: host.clone(), mod_id: String::new(), mod_index: 0, size: 0, prov: Vec::new() });
                    self.files.len() - 1
                }
            };
            let f = &mut self.files[idx];
            f.host = host.clone();
            f.mod_id = mod_id.to_string();
            f.mod_index = mod_index;
            f.size = size;
            if f.prov.len() < PROV_MAX {
                f.prov.push(Provider { host, mod_id: mod_id.to_string(), mod_index, size });
            }
        }
    }

    /// `gw_mod_by_path`
    pub fn lookup(&self, path: &str) -> Option<&OverlayFile> {
        self.by_path(path).map(|i| &self.files[i])
    }
}
