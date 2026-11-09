//! The skin registry: the port of pc/platform/gw_skins_core.h (the pure logic: rows, order, the 255 cap, team colours,
//! part visibility, Kirby hat rows, the netplay wire costume) and gw_skins_boot.inc (the engine half: every mounted
//! mod of kind "skin" has its mod.json read with Geno's strict JSON reader, its costume files are stat'ed, never read,
//! and the rows are handed to the core).
//!
//! A skin costume is one row appended to a fighter's own costume list (retail, m-ex or a Geno define: target
//! `{"geno": "<define key>"}`). The game asks through the `Skins_*` callees (ftdata's costume tables, gm_1601's costume
//! counts and team colours, mex_data's part visibility and Kirby hat rows) and through the define queries in
//! geno_host (`DefineCostumeCount`, `DefineCostumeName`, `DefineHasArt`, `DefineArtOpen`).
//!
//! Nothing here is simulation state: costumes are cosmetic, and a skin the other side lacks is shown as the default.
//! The C folds nothing of a skin into the netplay identity (gw_mexid.c), and neither does this.
#![allow(non_snake_case, static_mut_refs)]

use melee_script::geno::json::{Doc, Ty};

pub const SK_FK_MAX: usize = 160;
pub const SK_CAP: i32 = 255;
pub const SK_WIRE_TAG: i32 = 0x4000_0000;
pub const SK_WIRE_MASK: i32 = 0x3FFF_FFFF;
const SK_PATH: usize = 160;
const SK_SYM: usize = 96;
const SK_NAME: usize = 24;
const SK_ID: usize = 64;

/// `sk_costume`.
#[derive(Clone, Debug, Default)]
pub struct Costume {
    pub fk: i32,
    pub mod_id: String,
    pub entry: i32,
    pub order: i32,
    pub name: String,
    pub file: String,
    pub joint: String,
    pub matanim: String,
    pub csp: String,
    pub stock: String,
    pub team: i32,
    pub like: i32,
    pub kirby_hat: i32,
    pub partner: bool,
    pub pfile: String,
    pub pjoint: String,
    pub pmat: String,
    pub id: u64,
    pub index: i32,
    pub partner_row: bool,
    /// The mod's payload folder (files/), where the art files are read (gw_DVDReadFileAlloc of the mounted path).
    pub payload: std::path::PathBuf,
}

/// `sk_registry`.
pub struct Reg {
    pub c: Vec<Costume>,
    pub base: [i32; SK_FK_MAX],
    pub first: [i32; SK_FK_MAX],
    pub added: [i32; SK_FK_MAX],
    pub refused: i32,
    pub finalized: bool,
    /// sk_sink: the messages (the engine logs them; the tests read them)
    pub log: Vec<String>,
}
impl Default for Reg {
    fn default() -> Self {
        Reg { c: Vec::new(), base: [0; SK_FK_MAX], first: [-1; SK_FK_MAX], added: [0; SK_FK_MAX], refused: 0, finalized: false, log: Vec::new() }
    }
}

/// snprintf into a char[cap] (ASCII-safe truncation at a char boundary).
fn capped(s: &str, cap: usize) -> String {
    if s.len() < cap {
        return s.to_string();
    }
    let mut e = cap - 1;
    while e > 0 && !s.is_char_boundary(e) {
        e -= 1;
    }
    s[..e].to_string()
}

fn sk_mix(mut h: u64, v: u64) -> u64 {
    h ^= v;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    h
}

/// `sk_identity(mod, entry, stamp)`.
pub fn sk_identity(m: &str, entry: i32, stamp: &str) -> u64 {
    let mut h = 0x534B_494E_0000_0001u64;
    for &b in m.as_bytes() {
        h = sk_mix(h, b.to_ascii_lowercase() as u64);
    }
    h = sk_mix(h, 0xFF);
    h = sk_mix(h, entry as u32 as u64);
    h = sk_mix(h, 0xFE);
    for &b in stamp.as_bytes() {
        h = sk_mix(h, b as u64);
    }
    h
}

fn strcasecmp(a: &str, b: &str) -> core::cmp::Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    for i in 0.. {
        let x = a.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        let y = b.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        if x != y {
            return x.cmp(&y);
        }
        if x == 0 {
            break;
        }
    }
    core::cmp::Ordering::Equal
}

/// `sk_cmp`: (fk, partner row, order, mod id case-folded then exact, entry).
fn sk_cmp(a: &Costume, b: &Costume) -> core::cmp::Ordering {
    a.fk.cmp(&b.fk)
        .then(a.partner_row.cmp(&b.partner_row))
        .then(a.order.cmp(&b.order))
        .then_with(|| strcasecmp(&a.mod_id, &b.mod_id))
        .then_with(|| a.mod_id.as_bytes().cmp(b.mod_id.as_bytes()))
        .then(a.entry.cmp(&b.entry))
}

/// `sk_partner_of`: Popo -> Nana, Zelda -> Sheik.
fn sk_partner_of(fk: i32) -> i32 {
    match fk {
        10 => 11,
        19 => 7,
        _ => -1,
    }
}
fn sk_is_partner_kind(fk: i32) -> bool {
    fk == 11 || fk == 7
}

impl Reg {
    fn say(&mut self, m: String) {
        self.log.push(m);
    }

    /// `sk_add`: a copy of a validated row; -1 for a FighterKind outside the table.
    pub fn add(&mut self, s: &Costume) -> i32 {
        if s.fk < 0 || s.fk as usize >= SK_FK_MAX {
            return -1;
        }
        let mut r = s.clone();
        r.partner_row = false;
        r.index = -1;
        self.c.push(r);
        0
    }

    fn fighter_name(fk: i32, name_fn: &dyn Fn(i32) -> Option<String>) -> String {
        match name_fn(fk) {
            Some(n) if !n.is_empty() => n,
            _ => format!("fighter {fk}"),
        }
    }

    /// `sk_finalize(base_fn, name_fn)`: order the rows, give indices after each fighter's own costumes, refuse past
    /// SK_CAP, resolve team claims, generate the partner rows. Idempotent.
    pub fn finalize(&mut self, base_fn: &dyn Fn(i32) -> i32, name_fn: &dyn Fn(i32) -> Option<String>) {
        if self.finalized {
            return;
        }
        self.finalized = true;
        for fk in 0..SK_FK_MAX {
            self.base[fk] = base_fn(fk as i32);
        }
        let mut rows = core::mem::take(&mut self.c);
        rows.sort_by(sk_cmp);
        let mut out: Vec<Costume> = Vec::with_capacity(rows.len() * 2 + 1);
        let mut seen = [0i32; SK_FK_MAX];
        for s in &rows {
            let fk = s.fk;
            let f = fk as usize;
            if sk_is_partner_kind(fk) {
                let m = format!(
                    "skins: {}/{}: {} takes its main fighter's skins - target {} instead",
                    s.mod_id, s.entry, Self::fighter_name(fk, name_fn), if fk == 11 { "popo" } else { "zelda" }
                );
                self.say(m);
                self.refused += 1;
                continue;
            }
            if self.base[f] <= 0 {
                let m = format!("skins: {}/{}: {} has no costume list - skipped", s.mod_id, s.entry, Self::fighter_name(fk, name_fn));
                self.say(m);
                self.refused += 1;
                continue;
            }
            let idx = self.base[f] + seen[f];
            if idx >= SK_CAP {
                self.refused += 1;
                seen[f] += 1;
                if idx == SK_CAP {
                    let m = format!("skins: {} is at {} costumes - refused {} (and any later ones)", Self::fighter_name(fk, name_fn), SK_CAP, s.mod_id);
                    self.say(m);
                }
                continue;
            }
            seen[f] += 1;
            let mut r = s.clone();
            r.index = idx;
            out.push(r);
            self.added[f] += 1;
            let partner = sk_partner_of(fk);
            if partner >= 0 && self.base[partner as usize] == self.base[f] {
                let mut p = out.last().unwrap().clone();
                p.fk = partner;
                p.partner_row = true;
                p.team = -1;
                if p.partner {
                    p.file = p.pfile.clone();
                    p.joint = p.pjoint.clone();
                    p.matanim = p.pmat.clone();
                } else {
                    // the installer copies the partner's costume 0
                    p.file.clear();
                    p.joint.clear();
                    p.matanim.clear();
                }
                p.csp.clear();
                p.stock.clear();
                self.added[partner as usize] += 1;
                out.push(p);
            } else if partner >= 0 {
                let m = format!(
                    "skins: {}/{}: the partner kind has {} costumes against {} - it keeps its default",
                    s.mod_id, s.entry, self.base[partner as usize], self.base[f]
                );
                self.say(m);
            }
        }
        out.sort_by(|a, b| a.fk.cmp(&b.fk).then(a.index.cmp(&b.index)));
        self.c = out;
        self.first = [-1; SK_FK_MAX];
        for i in 0..self.c.len() {
            let f = self.c[i].fk as usize;
            if self.first[f] < 0 {
                self.first[f] = i as i32;
            }
            // a team colour is claimed once; the first row in order keeps it
            if self.c[i].team >= 0 {
                for j in self.first[f] as usize..i {
                    if self.c[j].team == self.c[i].team {
                        let t = self.c[i].team;
                        let m = format!(
                            "skins: {}/{}: team {} is already costume {} - ignored",
                            self.c[i].mod_id, self.c[i].entry, if t == 0 { "red" } else if t == 1 { "blue" } else { "green" }, self.c[j].index
                        );
                        self.say(m);
                        self.c[i].team = -1;
                        break;
                    }
                }
            }
        }
        // two skins of one fighter with the same 30-bit wire identity would show each other's costume on a peer
        let n = self.c.len();
        for i in 0..n {
            let mut j = i + 1;
            while j < n && self.c[j].fk == self.c[i].fk {
                if (self.c[j].id & SK_WIRE_MASK as u64) == (self.c[i].id & SK_WIRE_MASK as u64) {
                    let m = format!(
                        "skins: {}/{} and {}/{} share a netplay identity (cosmetic only)",
                        self.c[i].mod_id, self.c[i].entry, self.c[j].mod_id, self.c[j].entry
                    );
                    self.say(m);
                }
                j += 1;
            }
        }
    }

    fn valid(fk: i32) -> bool {
        fk >= 0 && (fk as usize) < SK_FK_MAX
    }
    pub fn base_of(&self, fk: i32) -> i32 {
        if Self::valid(fk) { self.base[fk as usize] } else { 0 }
    }
    pub fn added_of(&self, fk: i32) -> i32 {
        if Self::valid(fk) { self.added[fk as usize] } else { 0 }
    }
    /// `sk_total`: the costume count with the skins.
    pub fn total(&self, fk: i32) -> i32 {
        if Self::valid(fk) { self.base[fk as usize] + self.added[fk as usize] } else { 0 }
    }
    /// `sk_row`: the row of absolute costume `idx`, None for an original costume or none at all.
    pub fn row(&self, fk: i32, idx: i32) -> Option<&Costume> {
        if !Self::valid(fk) || self.first[fk as usize] < 0 || idx < self.base[fk as usize] {
            return None;
        }
        let j = self.first[fk as usize] + (idx - self.base[fk as usize]);
        let r = self.c.get(usize::try_from(j).ok()?)?;
        (r.fk == fk && r.index == idx).then_some(r)
    }
    /// `sk_team`: the costume a skin declared team colour `t`, or -1.
    pub fn team(&self, fk: i32, t: i32) -> i32 {
        if !Self::valid(fk) || self.first[fk as usize] < 0 {
            return -1;
        }
        self.c[self.first[fk as usize] as usize..].iter().take_while(|r| r.fk == fk).find(|r| r.team == t).map_or(-1, |r| r.index)
    }
    /// `sk_like`.
    pub fn like(&self, fk: i32, idx: i32) -> i32 {
        let Some(r) = self.row(fk, idx) else { return -1 };
        if r.like >= 0 && r.like < self.base[fk as usize] { r.like } else { 0 }
    }
    /// `sk_kirby_row`.
    pub fn kirby_row(&self, fk: i32, idx: i32) -> i32 {
        let Some(r) = self.row(fk, idx) else { return -1 };
        if (0..=5).contains(&r.kirby_hat) {
            return r.kirby_hat;
        }
        self.like(fk, idx) % 6
    }
    /// `sk_to_wire`.
    pub fn to_wire(&self, fk: i32, idx: i32) -> i32 {
        if idx < 0 {
            return 0;
        }
        if idx < self.base_of(fk) {
            return idx;
        }
        if let Some(r) = self.row(fk, idx) {
            return SK_WIRE_TAG | (r.id as i32 & SK_WIRE_MASK);
        }
        if idx < SK_CAP { idx } else { 0 }
    }
    /// `sk_from_wire`: a peer's wire value -> this install's index (unknown: 0, the default).
    pub fn from_wire(&self, fk: i32, wire: i32) -> i32 {
        if wire < 0 {
            return 0;
        }
        if wire & SK_WIRE_TAG == 0 {
            let b = self.base_of(fk);
            if b <= 0 {
                return if wire < SK_CAP { wire } else { 0 };
            }
            return if wire < b { wire } else { 0 };
        }
        if !Self::valid(fk) || self.first[fk as usize] < 0 {
            return 0;
        }
        self.c[self.first[fk as usize] as usize..]
            .iter()
            .take_while(|r| r.fk == fk)
            .find(|r| (r.id as i32 & SK_WIRE_MASK) == (wire & SK_WIRE_MASK) && !r.partner_row)
            .map_or(0, |r| r.index)
    }
}

// ---- the engine half (gw_skins_boot.inc) ----

/// How a skin target resolves (the host's fighter tables); a trait so the tests can stand in.
pub trait Targets {
    /// gw_Geno_DefineResolve: a define's CK, or < 1.
    fn define_resolve(&self, key: &str) -> i32;
    fn define_base_kind(&self, kind: i32) -> i32;
    fn define_costume_count_native(&self, kind: i32) -> i32;
    /// gn_kind_for_file: this install's FighterKind for a Pl file, -1 when not installed.
    fn kind_for_file(&self, pl: &str) -> i32;
}

/// `sk_fighter_label`.
fn fighter_label(fk: i32) -> Option<String> {
    use melee_script::geno::registry::VANILLA;
    if (0..27).contains(&fk) {
        return Some(VANILLA[fk as usize].1.to_string());
    }
    if fk < 0 || fk as usize >= SK_FK_MAX {
        return Some("?".into());
    }
    Some(format!("fighter {fk}"))
}

/// `sk_safe_path`: a plain relative path inside a mod's files/.
fn safe_path(p: &str) -> bool {
    let n = p.len();
    if n == 0 || n >= SK_PATH || p.starts_with('/') || p.contains('\\') || p.contains(':') {
        return false;
    }
    if p.bytes().any(|c| c < 32) {
        return false;
    }
    !p.contains("..")
}
fn printable(s: &str, min: usize, max: usize) -> bool {
    let n = s.len();
    n >= min && n <= max && s.bytes().all(|c| (32..=126).contains(&c))
}
fn file_exists(payload: &std::path::Path, rel: &str) -> bool {
    std::fs::File::open(payload.join(rel)).is_ok()
}

struct Rd<'a> {
    d: &'a Doc,
    id: &'a str,
    log: Vec<String>,
}
impl Rd<'_> {
    fn say(&mut self, m: String) {
        self.log.push(m);
    }
    /// `sk_strkey`: 1 set, 0 absent (and optional), -1 refused.
    fn strkey(&mut self, obj: i32, key: &str, out: &mut String, cap: usize, what: &str, required: bool) -> i32 {
        let v = self.d.get(obj, key);
        out.clear();
        if v < 0 {
            if required {
                let m = format!("skins: {}: {} needs \"{}\"", self.id, what, key);
                self.say(m);
            }
            return if required { -1 } else { 0 };
        }
        let n = &self.d.n[v as usize];
        if n.ty != Ty::Str || n.str.len() >= cap {
            let m = format!("skins: {}: {} \"{}\" must be a string of at most {} characters", self.id, what, key, cap - 1);
            self.say(m);
            return -1;
        }
        *out = String::from_utf8_lossy(&n.str).into_owned();
        1
    }
    fn known_keys(&mut self, obj: i32, keys: &[&str], what: &str) -> bool {
        for c in self.d.kids(obj) {
            let k = &self.d.n[c as usize].key;
            if !keys.iter().any(|x| x.as_bytes() == k.as_slice()) {
                let m = format!("skins: {}: {} has a key \"{}\" this build does not know - the mod is skipped", self.id, what, String::from_utf8_lossy(k));
                self.say(m);
                return false;
            }
        }
        true
    }
    /// `sk_int`: Ok(value, def when absent) or Err (logged).
    fn int(&mut self, obj: i32, key: &str, lo: i32, hi: i32, def: i32, what: &str) -> Result<i32, ()> {
        let v = self.d.get(obj, key);
        if v < 0 {
            return Ok(def);
        }
        let n = &self.d.n[v as usize];
        if n.ty != Ty::Num || n.num != (n.num as i32) as f64 || n.num < lo as f64 || n.num > hi as f64 {
            let m = format!("skins: {}: {} \"{}\" must be a whole number {}..{}", self.id, what, key, lo, hi);
            self.say(m);
            return Err(());
        }
        Ok(n.num as i32)
    }

    /// `sk_resolve_target`: the FighterKind a target names, or -1 (logged).
    fn target(&mut self, t: i32, tg: &dyn Targets) -> i32 {
        let id = self.id;
        if !self.d.is(t, Ty::Obj) {
            self.say(format!("skins: {id}: skin.target must be an object"));
            return -1;
        }
        if !self.known_keys(t, &["retail", "mex", "geno"], "skin.target") {
            return -1;
        }
        let kids = self.d.kids(t);
        if kids.len() != 1 {
            self.say(format!("skins: {id}: skin.target names exactly one of retail, mex, geno"));
            return -1;
        }
        let c = &self.d.n[kids[0] as usize];
        let key = String::from_utf8_lossy(&c.key).into_owned();
        if c.ty != Ty::Str || c.str.len() >= SK_PATH {
            self.say(format!("skins: {id}: skin.target.{key} must be a string"));
            return -1;
        }
        let name = String::from_utf8_lossy(&c.str).into_owned();
        if key == "geno" {
            let ck = tg.define_resolve(&name);
            if ck < 1 {
                self.say(format!("skins: {id}: Geno define \"{name}\" is not installed - skipped"));
                return -1;
            }
            if tg.define_base_kind(ck - 1) >= 0 && tg.define_costume_count_native(ck - 1) == 0 {
                self.say(format!("skins: {id}: define \"{name}\" wears Mario's costumes - its skins are installed as Mario's"));
                return 0;
            }
            return ck - 1;
        }
        if key == "retail" {
            let pl = melee_script::geno::registry::target_file(&name);
            let fk = if pl.is_empty() { -1 } else { tg.kind_for_file(&pl) };
            if !(0..27).contains(&fk) {
                self.say(format!("skins: {id}: \"{name}\" is not a retail fighter (mario, fox, captain, ...)"));
                return -1;
            }
            return fk;
        }
        // mex: the fighter's Pl file
        let pl = if name.len() > 4 && name[name.len() - 4..].eq_ignore_ascii_case(".dat") { name.clone() } else { format!("{name}.dat") };
        let fk = tg.kind_for_file(&pl);
        if fk < melee_script::geno::registry::GN_MEX_KIND0 {
            self.say(format!("skins: {id}: m-ex fighter {pl} is not installed - skipped"));
            return -1;
        }
        fk
    }
}

/// `sk_read_text(id, payload, text)`: one skin mod's mod.json -> rows added to `reg`. The number of costumes taken.
pub fn read_text(reg: &mut Reg, id: &str, payload: &std::path::Path, text: &[u8], tg: &dyn Targets) -> i32 {
    const COS_KEYS: [&str; 11] = ["name", "file", "joint", "matanim", "team", "like", "kirby_hat", "csp", "stock", "icon", "partner"];
    let (d, root) = Doc::parse(text);
    let mut r = Rd { d: &d, id, log: Vec::new() };
    let taken = read_doc(&mut r, reg, root, payload, tg, &COS_KEYS);
    reg.log.append(&mut r.log);
    taken
}

fn read_doc(r: &mut Rd, reg: &mut Reg, root: i32, payload: &std::path::Path, tg: &dyn Targets, cos_keys: &[&str]) -> i32 {
    let (d, id) = (r.d, r.id);
    if root < 0 || !d.is(root, Ty::Obj) {
        let e = if d.err.is_empty() { "?" } else { d.err };
        r.say(format!("skins: {id}: mod.json is not valid JSON ({e})"));
        return 0;
    }
    let sk = d.get(root, "skin");
    if !d.is(sk, Ty::Obj) {
        r.say(format!("skins: {id}: kind \"skin\" needs a \"skin\" object"));
        return 0;
    }
    let str_of = |k: &str, cap: usize| {
        let v = d.get(root, k);
        if d.is(v, Ty::Str) { capped(&d.text(v), cap) } else { String::new() }
    };
    let mut stamp = str_of("hash", 96);
    if stamp.is_empty() {
        stamp = str_of("version", 96);
    }
    let modname = str_of("name", SK_NAME * 2);
    if !r.known_keys(sk, &["format", "target", "order", "costumes"], "skin") {
        return 0;
    }
    let Ok(format) = r.int(sk, "format", 1, 1_000_000, 1, "skin") else { return 0 };
    if format != 1 {
        r.say(format!("skins: {id}: skin format {format} is newer than this build reads (1) - skipped"));
        return 0;
    }
    let Ok(order) = r.int(sk, "order", -1_000_000, 1_000_000, 0, "skin") else { return 0 };
    let fk = r.target(d.get(sk, "target"), tg);
    if fk < 0 {
        return 0;
    }
    let c = d.get(sk, "costumes");
    if !d.is(c, Ty::Arr) || d.n[c as usize].first < 0 {
        r.say(format!("skins: {id}: skin.costumes must list at least one costume"));
        return 0;
    }
    let kids = d.kids(c);
    let ncost = kids.len();
    let mut taken = 0;
    for (entry, &v) in kids.iter().enumerate() {
        let entry = entry as i32;
        let what = format!("costume {}", entry + 1);
        let mut s = Costume {
            fk,
            mod_id: capped(id, SK_ID),
            entry,
            order,
            team: -1,
            kirby_hat: -1,
            payload: payload.to_path_buf(),
            ..Default::default()
        };
        if !d.is(v, Ty::Obj) {
            r.say(format!("skins: {id}: {what} must be an object"));
            continue;
        }
        if !r.known_keys(v, cos_keys, &what) {
            continue;
        }
        let mut bad = r.strkey(v, "name", &mut s.name, SK_NAME, &what, false) < 0;
        if !bad && !s.name.is_empty() && !printable(&s.name, 1, SK_NAME - 1) {
            r.say(format!("skins: {id}: {what} name must be printable ASCII, 1..{} characters", SK_NAME - 1));
            bad = true;
        }
        if !bad && s.name.is_empty() {
            let cut = |t: &str, n: usize| -> String { capped(t, n + 1) };
            s.name = if !modname.is_empty() && ncost == 1 {
                cut(&modname, 23)
            } else if !modname.is_empty() {
                capped(&format!("{} {}", cut(&modname, 18), entry + 1), SK_NAME)
            } else {
                capped(&format!("{} {}", cut(id, 18), entry + 1), SK_NAME)
            };
        }
        if !bad && r.strkey(v, "file", &mut s.file, SK_PATH, &what, true) < 0 {
            bad = true;
        }
        if !bad && !safe_path(&s.file) {
            r.say(format!("skins: {id}: {what} file \"{}\" must be a plain relative path inside files/", s.file));
            bad = true;
        }
        if !bad && !file_exists(payload, &s.file) {
            r.say(format!("skins: {id}: {what} file {} is not in the mod's files/ - skipped", s.file));
            bad = true;
        }
        if !bad && r.strkey(v, "joint", &mut s.joint, SK_SYM, &what, true) < 0 {
            bad = true;
        }
        if !bad && r.strkey(v, "matanim", &mut s.matanim, SK_SYM, &what, false) < 0 {
            bad = true;
        }
        if !bad {
            let p = d.get(v, "team");
            if p >= 0 {
                let t = if d.is(p, Ty::Str) { d.n[p as usize].str.as_slice() } else { b"" };
                s.team = match t {
                    b"red" => 0,
                    b"blue" => 1,
                    b"green" => 2,
                    _ => {
                        bad = true;
                        -1
                    }
                };
                if bad {
                    r.say(format!("skins: {id}: {what} team must be \"red\", \"blue\" or \"green\""));
                }
            }
        }
        if !bad {
            match r.int(v, "like", 0, 254, 0, &what) {
                Ok(x) => s.like = x,
                Err(()) => bad = true,
            }
        }
        if !bad {
            match r.int(v, "kirby_hat", 0, 5, -1, &what) {
                Ok(x) => s.kirby_hat = x,
                Err(()) => bad = true,
            }
        }
        if !bad {
            for key in ["csp", "stock"] {
                let mut dst = String::new();
                if r.strkey(v, key, &mut dst, SK_PATH, &what, false) < 0 {
                    bad = true;
                    break;
                }
                if !dst.is_empty() && !safe_path(&dst) {
                    r.say(format!("skins: {id}: {what} {key} \"{dst}\" must be a plain relative path"));
                    bad = true;
                    break;
                }
                if !dst.is_empty() && !file_exists(payload, &dst) {
                    r.say(format!("skins: {id}: {what} {key} {dst} is not in the mod's files/ - default art"));
                    dst.clear();
                }
                if key == "csp" { s.csp = dst } else { s.stock = dst }
            }
        }
        if !bad {
            let p = d.get(v, "partner");
            if p >= 0 {
                if !d.is(p, Ty::Obj) || !r.known_keys(p, &["file", "joint", "matanim"], "partner") {
                    bad = true;
                } else {
                    // the C's || chain: stops at the first failing step
                    let ok = r.strkey(p, "file", &mut s.pfile, SK_PATH, "partner", true) >= 0
                        && safe_path(&s.pfile)
                        && file_exists(payload, &s.pfile)
                        && r.strkey(p, "joint", &mut s.pjoint, SK_SYM, "partner", true) >= 0
                        && r.strkey(p, "matanim", &mut s.pmat, SK_SYM, "partner", false) >= 0;
                    if !ok {
                        r.say(format!("skins: {id}: {what} partner needs file (inside files/), joint, optional matanim"));
                        bad = true;
                    } else {
                        s.partner = true;
                    }
                }
            }
        }
        if bad {
            continue;
        }
        s.id = sk_identity(id, entry, &stamp);
        if reg.add(&s) == 0 {
            taken += 1;
        }
    }
    taken
}

/// A mounted mod of kind "skin": its id, its folder (mod.json) and its payload (files/).
#[derive(Clone)]
pub struct SkinMod {
    pub id: String,
    pub dir: std::path::PathBuf,
    pub payload: std::path::PathBuf,
}

/// `gw_Skins_Build`'s body: every skin mod's mod.json, then sk_finalize. Returns the registry and the summary line.
pub fn build(mods: &[SkinMod], base_fn: &dyn Fn(i32) -> i32, tg: &dyn Targets) -> Reg {
    let t0 = std::time::Instant::now();
    let mut reg = Reg::default();
    let (mut read, mut bad) = (0, 0);
    for m in mods {
        read += 1;
        // gn_read_file: at most 1 MiB
        let text = std::fs::read(m.dir.join("mod.json")).ok().filter(|t| t.len() <= 1 << 20);
        let n = match text {
            Some(t) => read_text(&mut reg, &m.id, &m.payload, &t, tg),
            None => {
                reg.log.push(format!("skins: {}: mod.json cannot be read", m.id));
                0
            }
        };
        if n == 0 {
            bad += 1;
        }
    }
    reg.finalize(base_fn, &fighter_label);
    let (mut fighters, mut costumes) = (0, 0);
    for fk in 0..SK_FK_MAX {
        if reg.added[fk] > 0 {
            fighters += 1;
            costumes += reg.added[fk];
        }
    }
    if read > 0 || costumes > 0 {
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        reg.log.push(format!(
            "skins: {read} skin mod(s) read ({bad} with nothing usable), {costumes} costume(s) on {fighters} fighter(s), {} refused, {ms:.1} ms",
            reg.refused
        ));
    }
    reg
}

// ---- the process registry and the gw_Skins_* queries ----

static mut MODS: Vec<SkinMod> = Vec::new();
static mut REG: Option<Reg> = None;
static mut SUSPENDED: bool = false;

/// The mounted skin mods (main.rs, from the mods folder's active set, mount order). Before the first query.
pub fn set_mods(mods: Vec<SkinMod>) {
    unsafe {
        MODS = mods;
        REG = None;
    }
}

/// The engine's answers for a skin target (the Geno registry, the m-ex slots).
struct HostTargets;
impl Targets for HostTargets {
    fn define_resolve(&self, key: &str) -> i32 {
        crate::geno_host::DefineResolve(key)
    }
    fn define_base_kind(&self, kind: i32) -> i32 {
        crate::geno_host::DefineBaseKind(kind)
    }
    fn define_costume_count_native(&self, kind: i32) -> i32 {
        crate::geno_host::DefineCostumeCountNative(kind)
    }
    fn kind_for_file(&self, pl: &str) -> i32 {
        melee_script::geno::Registry::kind_for_file(pl, &crate::mex_host::GenoEnv { dir: std::path::PathBuf::new() })
    }
}

/// gw_mex_ftfunction_runtime.c `gw_costume_native_count`: a fighter's own costume count before any skin (the retail
/// table, or the disc's m-ex row when it gives a retail fighter more; an m-ex slot's row; a Geno define's declared
/// list, a donor-based one Mario's; the skins255 fixture). 0 = no costume list.
pub fn costume_native_count(fk: i32) -> i32 {
    const MEX0: i32 = 0x21;
    if fk < 0 {
        return 0;
    }
    if fk < MEX0 {
        let have = unsafe { melee_fighter::tu::ftdata::CostumeListsForeachCharacter[fk as usize].numCostumes } as i32;
        let n = crate::mex_host::FtCostumeCount(fk);
        let mut n = if n > have && fk < 27 { n } else { have };
        if fk == 0 {
            let t = melee_fighter::tu::ftdata::ext::Skins255_TestCount();
            if t > n {
                n = t;
            }
        }
        return n;
    }
    if fk < MEX0 + 94 {
        if crate::geno_host::DefineBaseKind(fk) >= 0 {
            let n = crate::geno_host::DefineCostumeCountNative(fk);
            return if n != 0 { n } else { costume_native_count(0) };
        }
        let k = crate::mex_host::SlotInternal(fk - MEX0);
        return if k >= 0 { crate::mex_host::FtCostumeCount(k) } else { 0 };
    }
    0
}

/// `sk_ensure`: the registry, built on first use. The retail costume table is the C's static initialiser; here it is
/// installed at engine start (melee_link), so a query before that answers empty without building.
fn reg() -> Option<&'static Reg> {
    unsafe {
        if REG.is_none() {
            if melee_fighter::tu::ftdata::CostumeListsForeachCharacter[0].numCostumes == 0 {
                return None;
            }
            // process-lifetime data: on the system heap, never in a match arena a rollback rewinds
            crate::snap_host::host_heap(|| {
                let r = build(&MODS, &costume_native_count, &HostTargets);
                for l in &r.log {
                    eprintln!("{l}");
                }
                // mex_data's part-visibility / Kirby hat readers answer from the same rows
                melee_runtime::tu::mex_data::SKINS = Some(melee_runtime::tu::mex_data::SkinRegistry {
                    base: r.base.to_vec(),
                    rows: r.c.iter().map(|c| melee_runtime::tu::mex_data::SkinRow { fk: c.fk, index: c.index, like: c.like, kirby_hat: c.kirby_hat }).collect(),
                });
                REG = Some(r);
            });
        }
        REG.as_ref()
    }
}
fn live() -> Option<&'static Reg> {
    if unsafe { SUSPENDED } { None } else { reg() }
}

/// gw_Skins_RebuildForTest's stand-in (tests only): install a finalized registry (None: back to "not built").
pub fn install_for_test(r: Option<Reg>) {
    unsafe { REG = r };
}

/// gw_Skins_Suspend: tests that install a throw-away define on a kind a mounted skin also targets.
pub fn Suspend(on: bool) {
    unsafe { SUSPENDED = on };
}
/// mex_data's `SKINS_ENSURE`: build before a part-visibility read.
pub fn Ensure() {
    reg();
}
pub fn Added(fk: i32) -> i32 { live().map_or(0, |r| r.added_of(fk)) }
pub fn Base(fk: i32) -> i32 { reg().map_or(0, |r| r.base_of(fk)) }
pub fn Total(fk: i32) -> i32 {
    match (reg(), unsafe { SUSPENDED }) {
        (Some(r), true) => r.base_of(fk),
        (Some(r), false) => r.total(fk),
        _ => 0,
    }
}
pub fn Count() -> i32 { reg().map_or(0, |r| r.added.iter().sum()) }
/// gw_Skins_String(fk, c, which): 0 file, 1 joint, 2 matanim; None for empty (a partner row without files).
pub fn String_(fk: i32, c: i32, which: i32) -> Option<&'static str> {
    let r = live()?.row(fk, c)?;
    let s = match which {
        0 => &r.file,
        1 => &r.joint,
        _ => &r.matanim,
    };
    (!s.is_empty()).then_some(s.as_str())
}
fn interned(s: Option<&'static str>) -> *const core::ffi::c_char {
    // the rows live for the process: hand out NUL-terminated copies that stay put
    static mut POOL: Vec<(usize, Box<[u8]>)> = Vec::new();
    let Some(s) = s else { return core::ptr::null() };
    unsafe {
        let key = s.as_ptr() as usize;
        if let Some(x) = POOL.iter().find(|x| x.0 == key) {
            return x.1.as_ptr() as *const _;
        }
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        let b = b.into_boxed_slice();
        let p = b.as_ptr();
        POOL.push((key, b));
        p as *const _
    }
}
pub fn StringPtr(fk: i32, c: i32, which: i32) -> *const core::ffi::c_char { interned(String_(fk, c, which)) }

/// gw_SceneLaunch_CKindToFKind.
pub fn ck_to_fk(ck: i32) -> i32 {
    const CK_TO_FK: [i8; 0x21] = [
        2, 3, 1, 24, 4, 5, 6, 17, 0, 18, 16, 8, 9, 12, 10, 15, 13, 14, 19, 7, 22, 20, 21, 26, 23, 25, 27, 29, 30, 31, 28, 32, 10,
    ];
    if (0x22..0x22 + 94).contains(&ck) {
        return 0x21 + (ck - 0x22);
    }
    if (0..0x21).contains(&ck) { CK_TO_FK[ck as usize] as i32 } else { -1 }
}
fn sk_fk(ck: i32) -> i32 {
    if ck >= 0 { ck_to_fk(ck) } else { -1 }
}

/// gw_Skins_CostumeCountCK: the count with skins, 0 when the fighter has none (the caller keeps its answer).
pub fn CostumeCountCK(ck: i32) -> i32 {
    let fk = sk_fk(ck);
    match live() {
        Some(r) if fk >= 0 && r.added_of(fk) > 0 => r.total(fk),
        _ => 0,
    }
}
pub fn TeamCostumeCK(ck: i32, team: i32) -> i32 {
    let fk = sk_fk(ck);
    match live() {
        Some(r) if fk >= 0 => r.team(fk, team),
        _ => -1,
    }
}
/// gw_Skins_NameCK: a skin costume's own name.
pub fn NameCK(ck: i32, costume: i32) -> Option<&'static str> {
    let fk = sk_fk(ck);
    if fk < 0 {
        return None;
    }
    live()?.row(fk, costume).map(|r| r.name.as_str())
}
pub fn ToWireCK(ck: i32, color: i32) -> i32 {
    let fk = sk_fk(ck);
    let Some(r) = reg() else { return color };
    if fk >= 0 { r.to_wire(fk, color) } else { color }
}
pub fn FromWireCK(ck: i32, wire: i32) -> i32 {
    let fk = sk_fk(ck);
    if fk < 0 || reg().is_none() {
        return if (0..SK_CAP).contains(&wire) { wire } else { 0 };
    }
    reg().unwrap().from_wire(fk, wire)
}
pub fn Like(fk: i32, c: i32) -> i32 { reg().map_or(-1, |r| r.like(fk, c)) }
pub fn KirbyRow(fk: i32, c: i32) -> i32 { reg().map_or(-1, |r| r.kirby_row(fk, c)) }

/// gw_SceneLaunch_PlayerColor's wire step: a color of 255 or more in port `n`'s match setup is a peer's wire costume.
pub fn scene_color(n: i32, ck: i32, color: i32) -> i32 {
    static mut SAID: [i32; 6] = [0; 6];
    if color < 255 || !(0..6).contains(&n) {
        return color;
    }
    let c = if ck >= 0 { FromWireCK(ck, color) } else { 0 };
    unsafe {
        if SAID[n as usize] != color {
            SAID[n as usize] = color;
            eprintln!(
                "skins: p{} wire costume {:08x} -> costume {c} here{}",
                n + 1,
                color as u32,
                if c == 0 { " (the default: this install lacks that skin)" } else { "" }
            );
        }
    }
    c
}

// ---- art ----

/// `sk_art_file`: a skin costume's own portrait (what 1) or stock icon (what 2), with its payload folder.
fn art_file(ck: i32, what: i32, costume: i32) -> Option<(&'static std::path::Path, &'static str)> {
    let fk = sk_fk(ck);
    if fk < 0 {
        return None;
    }
    let r = live()?.row(fk, costume)?;
    if r.partner_row {
        return None;
    }
    let f = match what {
        1 => &r.csp,
        2 => &r.stock,
        _ => return None,
    };
    (!f.is_empty()).then(|| (r.payload.as_path(), f.as_str()))
}
pub fn HasArtCK(ck: i32, what: i32, costume: i32) -> bool {
    art_file(ck, what, costume).is_some()
}
/// gw_Skins_ArtOpenCK: a gxtex handle for a skin costume's art, -1 when none or unreadable. The C reads the mounted
/// path (gw_DVDReadFileAlloc); the mod's own payload folder is that file unless another mod mounts the same path.
pub fn ArtOpenCK(ck: i32, what: i32, costume: i32) -> i32 {
    let Some((dir, file)) = art_file(ck, what, costume) else { return -1 };
    let Some(raw) = crate::snap_host::host_heap(|| std::fs::read(dir.join(file)).ok()) else { return -1 };
    crate::geno_host::GxTex_OpenBlob(raw, file)
}

/// Point the game's skin callees at the registry (before seams_gen's "no skins" defaults).
pub fn install() {
    unsafe {
        use melee_fighter::tu::ftdata::ext as fd;
        fd::SKINS_ADDED = Some(Added);
        fd::SKINS_BASE = Some(Base);
        fd::SKINS_STRING = Some(StringPtr);
        use melee_modes::tu::gm_1601::ext::hook as gm;
        gm::Skins_CostumeCountCK = Some(CostumeCountCK);
        gm::Skins_TeamCostumeCK = Some(TeamCostumeCK);
        melee_runtime::tu::mex_data::SKINS_ENSURE = Some(Ensure);
    }
}

#[cfg(test)]
mod tests {
    //! pc/tests/skins_core_test.c (the core) and gw_skins_tests.inc test_skins_parse (the mod.json reader).
    use super::*;

    fn base_fn(fk: i32) -> i32 {
        match fk {
            0 | 1 | 7 | 10 | 11 | 19 => 5,
            4 => 6,
            40 => 7,
            41 => 1,
            _ => 0,
        }
    }
    fn name_fn(fk: i32) -> Option<String> {
        match fk {
            0 => Some("Mario".into()),
            1 => Some("Fox".into()),
            _ => None,
        }
    }
    fn mk(fk: i32, m: &str, entry: i32, order: i32) -> Costume {
        Costume {
            fk,
            mod_id: m.into(),
            entry,
            order,
            name: format!("{m}-{entry}"),
            file: format!("skins/{m}/{entry}.dat"),
            joint: format!("J{entry}"),
            team: -1,
            like: 0,
            kirby_hat: -1,
            id: sk_identity(m, entry, "1.0.0"),
            ..Default::default()
        }
    }
    fn logged(r: &Reg, s: &str) -> bool {
        r.log.iter().any(|l| l.contains(s))
    }
    fn fin(r: &mut Reg) {
        r.finalize(&base_fn, &name_fn);
    }

    #[test]
    fn order_and_indices() {
        let mut r = Reg::default();
        for s in [mk(0, "zeta", 0, 0), mk(0, "Alpha", 1, 0), mk(0, "alpha", 0, 0), mk(0, "mid", 0, -5), mk(1, "fox-one", 0, 0)] {
            assert_eq!(r.add(&s), 0);
        }
        fin(&mut r);
        assert_eq!((r.total(0), r.total(1), r.added_of(0)), (9, 6, 4));
        assert_eq!(r.row(0, 5).unwrap().mod_id, "mid");
        assert_eq!(r.row(0, 6).unwrap().mod_id, "Alpha");
        assert_eq!(r.row(0, 8).unwrap().mod_id, "zeta");
        assert!(r.row(0, 4).is_none() && r.row(0, 9).is_none());
        assert_eq!(r.total(2), 0);
        let mut b = Reg::default();
        for s in [mk(0, "zeta", 0, 0), mk(0, "alpha", 0, 0), mk(0, "Alpha", 1, 0), mk(0, "mid", 0, -5)] {
            b.add(&s);
        }
        fin(&mut b);
        assert_eq!((b.row(0, 6).unwrap().index, b.row(0, 6).unwrap().mod_id.as_str()), (6, "Alpha"));
    }

    #[test]
    fn cap() {
        let mut r = Reg::default();
        for i in 0..300 {
            r.add(&mk(0, &format!("s{i:03}"), 0, 0));
        }
        fin(&mut r);
        assert_eq!((r.total(0), r.added_of(0), r.refused), (255, 250, 50));
        assert_eq!(r.row(0, 254).unwrap().mod_id, "s249");
        assert!(logged(&r, "is at 255 costumes - refused s250"));
        assert_eq!(r.from_wire(0, r.to_wire(0, 254)), 254);
        assert_eq!(r.from_wire(0, SK_WIRE_TAG | (sk_identity("s250", 0, "1.0.0") as i32 & SK_WIRE_MASK)), 0);
    }

    #[test]
    fn team_like_kirby() {
        let mut r = Reg::default();
        let mut a = mk(0, "a", 0, 0);
        a.team = 0;
        a.like = 2;
        let mut b = mk(0, "b", 0, 0);
        b.team = 0;
        let mut c = mk(0, "c", 0, 0);
        c.team = 2;
        c.like = 99;
        for s in [a, b, c] {
            r.add(&s);
        }
        fin(&mut r);
        assert_eq!((r.team(0, 0), r.team(0, 2), r.team(0, 1)), (5, 7, -1));
        assert!(logged(&r, "team red is already costume 5"));
        assert_eq!((r.like(0, 5), r.like(0, 7), r.like(0, 3), r.kirby_row(0, 5)), (2, 0, -1, 2));
        let mut k = Reg::default();
        let mut s = mk(4, "kb", 0, 0);
        s.kirby_hat = 3;
        s.like = 1;
        k.add(&s);
        fin(&mut k);
        assert_eq!((k.kirby_row(4, 6), k.total(4)), (3, 7));
    }

    #[test]
    fn partner() {
        let mut r = Reg::default();
        let p = mk(10, "ic-a", 0, 0);
        let mut q = mk(10, "ic-b", 0, 0);
        q.partner = true;
        q.pfile = "skins/ic-b/nana.dat".into();
        q.pjoint = "NJ".into();
        for s in [p, q, mk(11, "nana-direct", 0, 0)] {
            r.add(&s);
        }
        fin(&mut r);
        assert_eq!((r.total(10), r.total(11)), (7, 7));
        let r5 = r.row(11, 5).unwrap();
        assert!(r5.partner_row && r5.file.is_empty());
        let r6 = r.row(11, 6).unwrap();
        assert!(r6.partner_row && r6.file == "skins/ic-b/nana.dat" && r6.joint == "NJ");
        assert!(logged(&r, "takes its main fighter's skins"));
        assert_eq!(r.from_wire(11, r.to_wire(10, 6)), 0);
    }

    #[test]
    fn wire() {
        let mut r = Reg::default();
        for s in [mk(0, "one", 0, 0), mk(0, "two", 0, 0), mk(40, "one", 0, 0), mk(41, "geno-skin", 0, 0)] {
            r.add(&s);
        }
        fin(&mut r);
        for i in 0..5 {
            assert_eq!((r.to_wire(0, i), r.from_wire(0, i)), (i, i));
        }
        let w = r.to_wire(0, 5);
        assert!(w & SK_WIRE_TAG != 0 && w > 0);
        assert_eq!((r.from_wire(0, w), r.from_wire(0, r.to_wire(0, 6))), (5, 6));
        assert!(r.from_wire(40, w) == 7 || r.from_wire(40, w) == 0);
        assert_eq!((r.from_wire(1, w), r.from_wire(0, 17), r.from_wire(0, 254), r.from_wire(0, -1)), (0, 0, 0, 0));
        assert_eq!((r.from_wire(0, SK_WIRE_TAG | 0x12345), r.from_wire(99, 3)), (0, 3));
        assert_eq!(r.from_wire(41, r.to_wire(41, 1)), 1);
        assert!((5..7).all(|i| r.to_wire(0, i) >= SK_WIRE_TAG));
    }

    #[test]
    fn refusals_and_stress() {
        let mut r = Reg::default();
        r.add(&mk(2, "nolist", 0, 0));
        assert_ne!(r.add(&mk(999, "x", 0, 0)), 0);
        fin(&mut r);
        assert!(r.total(2) == 0 && logged(&r, "has no costume list"));
        let mut s = Reg::default();
        for fk in 0..2 {
            for i in 0..250 {
                s.add(&mk(fk, &format!("k{fk}-{i:03}"), i % 3, 0));
            }
        }
        fin(&mut s);
        assert_eq!((s.added_of(0), s.added_of(1)), (250, 250));
        for i in 5..255 {
            assert_eq!(s.row(1, i).unwrap().index, i);
            assert_eq!(s.from_wire(1, s.to_wire(1, i)), i);
        }
    }

    struct T;
    impl Targets for T {
        fn define_resolve(&self, key: &str) -> i32 {
            if key == "t9" { 127 } else { -1 }
        }
        fn define_base_kind(&self, _: i32) -> i32 {
            0
        }
        fn define_costume_count_native(&self, _: i32) -> i32 {
            2
        }
        fn kind_for_file(&self, pl: &str) -> i32 {
            melee_script::geno::Registry::kind_for_file(pl, &melee_script::geno::NoEnv)
        }
    }

    /// gw_skins_tests.inc test_skins_parse.
    #[test]
    fn skins_parse() {
        let root = std::env::temp_dir().join(format!("gw_skins_test_{}", std::process::id()));
        std::fs::create_dir_all(root.join("skins/a")).unwrap();
        std::fs::write(root.join("skins/a/x.dat"), "x").unwrap();
        std::fs::write(root.join("skins/a/csp.gxtex"), "x").unwrap();
        let bad: &[(&str, &str)] = &[
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/x.dat","joint":"J","bogus":1}]}}"#, "bogus"),
            (r#"{"skin":{"format":2,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "newer"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario","mex":"PlWf.dat"},"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "exactly one"),
            (r#"{"skin":{"format":1,"target":{"retail":"nobody"},"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "not a retail fighter"),
            (r#"{"skin":{"format":1,"target":{"mex":"PlZz.dat"},"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "not installed"),
            (r#"{"skin":{"format":1,"target":{"geno":"nope"},"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "not installed"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/missing.dat","joint":"J"}]}}"#, "not in the mod's files"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"../x.dat","joint":"J"}]}}"#, "plain relative path"),
            ("{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"name\":\"Caf\u{e9}\",\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "printable ASCII"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/x.dat","joint":"J","team":"purple"}]}}"#, "team must be"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/x.dat","joint":"J","kirby_hat":9}]}}"#, "kirby_hat"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[]}}"#, "at least one"),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"costumes":[{"file":"skins/a/x.dat"}]}}"#, "needs \"joint\""),
            (r#"{"skin":{"format":1,"target":{"retail":"mario"},"bogus":1,"costumes":[{"file":"skins/a/x.dat","joint":"J"}]}}"#, "key \"bogus\""),
            (r#"{"name":"no skin block"}"#, "needs a \"skin\" object"),
            ("{ not json", "not valid JSON"),
        ];
        for (i, (json, say)) in bad.iter().enumerate() {
            let mut r = Reg::default();
            let took = read_text(&mut r, "t-bad", &root, json.as_bytes(), &T);
            assert!(took == 0 && logged(&r, say), "refusal {i} ({say}): took {took}, log {:?}", r.log);
        }
        let mut r = Reg::default();
        let took = read_text(
            &mut r,
            "t-good",
            &root,
            br#"{"name":"Pack","version":"1.2.3","kind":"skin","skin":{"format":1,"target":{"retail":"mario"},"order":3,"costumes":[{"name":"Red One","file":"skins/a/x.dat","joint":"J1","matanim":"M1","team":"red","like":2,"csp":"skins/a/csp.gxtex","stock":"skins/a/gone.gxtex"},{"file":"skins/a/x.dat","joint":"J2"},{"name":"Blue","file":"skins/a/x.dat","joint":"J3","team":"blue"}]}}"#,
            &T,
        );
        assert_eq!(took, 3);
        r.finalize(&|fk| if fk == 0 { 5 } else { 0 }, &fighter_label);
        assert_eq!((r.total(0), r.added_of(0)), (8, 3));
        let (r5, r6, r7) = (r.row(0, 5).unwrap(), r.row(0, 6).unwrap(), r.row(0, 7).unwrap());
        assert_eq!((r5.name.as_str(), r6.name.as_str(), r7.name.as_str()), ("Red One", "Pack 2", "Blue"));
        assert!(r5.like == 2 && r5.matanim == "M1" && !r5.csp.is_empty() && r5.stock.is_empty() && r5.order == 3);
        assert_eq!((r.team(0, 0), r.team(0, 1), r.team(0, 2)), (5, 7, -1));
        assert_eq!(r5.id, sk_identity("t-good", 0, "1.2.3"));
        assert!(logged(&r, "gone.gxtex"));
        // a skin on a define (target {"geno": key}): the define's FighterKind, after its own costumes
        let mut g = Reg::default();
        let took = read_text(&mut g, "g", &root, br#"{"skin":{"format":1,"target":{"geno":"t9"},"costumes":[{"name":"Gold","file":"skins/a/x.dat","joint":"J","csp":"skins/a/csp.gxtex"}]}}"#, &T);
        assert_eq!(took, 1);
        g.finalize(&|fk| if fk == 126 { 2 } else { 0 }, &fighter_label);
        assert_eq!((g.total(126), g.row(126, 2).unwrap().name.as_str()), (3, "Gold"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ck_to_fk_table() {
        assert_eq!((ck_to_fk(0), ck_to_fk(8), ck_to_fk(0x20), ck_to_fk(0x21), ck_to_fk(0x22), ck_to_fk(0x22 + 93), ck_to_fk(0x22 + 94)), (2, 0, 10, -1, 0x21, 0x21 + 93, -1));
    }
}
