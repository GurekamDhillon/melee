//! geno_define_registry.inc, geno_define_core.h (GdfCatalog), geno_lua_registry.inc, geno_define_records.inc and
//! geno_define_online.inc: a native `define` entry (a brand-new fighter on a resident alias), its fighter-Lua block, its
//! records file and what netplay must agree on for it. The combo graph and the meter are combo.rs.
//!
//! A define is boot metadata: parsed once from the mods, bound to the aliases 127, 126, ... (CharacterKind; the
//! FighterKind is one less) in (mod, key) order, never changed during a match. Nothing here is simulation state; the
//! game half reads it through scalars (melee_engine::geno_host).
use super::consts::*;
use super::json::{Doc, Ty};
use super::lua::{GLua, GLUA_BOOL, GLUA_FLOAT, GLUA_INT, GLUA_NAME, GLUA_SLOT_NAME};
use super::registry::{callback_find, cap, eq_ic, mix, Env, Profile, Registry, MAX_STATES, VANILLA};

pub const GN_PRES_ICON: usize = 0;
pub const GN_PRES_PORTRAIT: usize = 1;
pub const GN_PRES_STOCK: usize = 2;
pub const GN_PRES_EMBLEM: usize = 3;
pub const GN_PRES_RESULTS: usize = 4;
pub const GN_PRES_KINDS: usize = 5;
pub const GN_MAX_VOICE: usize = 32;
pub const GPL_MAX_OWN_ROWS: usize = super::plan::GPL_MAX_OWN_ROWS;

/// slice 4: the files a base "none" define names (entry-level "fighter" block).
#[derive(Default)]
pub struct Fm {
    pub plan_file: String,
    pub anim_file: String,
    pub cos_file: Vec<String>,
    pub cos_joint: Vec<String>,
    pub cos_matanim: Vec<String>,
    pub cos_name: Vec<String>,
    /// -1 none, 0 red, 1 blue, 2 green.
    pub cos_team: Vec<i8>,
    pub own_clip: Vec<String>,
    pub plan_tried: bool,
    pub plan: Option<Box<super::plan::Plan>>,
}

/// slice 6: a define's own menu and HUD art (file names of the mod's files/ folder).
#[derive(Default)]
pub struct Pres {
    pub file: [Vec<String>; GN_PRES_KINDS],
}

/// slice 6 (geno 10): a define's own announcer call and voice clips.
#[derive(Default)]
pub struct Aud {
    pub announcer: String,
    pub voice_sfx: Vec<i32>,
    pub voice_vol: Vec<i32>,
    pub voice_stop: Vec<bool>,
    pub voice_states: Vec<u64>,
    pub voice_file: Vec<String>,
}

/// The define half of a profile (gn_profile's define / slice 4-6 / slice 5 fields).
pub struct Def {
    pub define_key: String,
    pub partner_key: String,
    pub none: bool,
    pub tmpl: usize,
    pub art_base_ck: i32,
    pub common_row: Vec<[i32; 9]>,
    pub cs_tag: Vec<i32>,
    pub fm: Option<Box<Fm>>,
    pub pres: Option<Box<Pres>>,
    pub aud: Option<Box<Aud>>,
    pub ai_like: i32,
    pub ai_set: bool,
    pub kb_copy: i32,
    pub thrown_row: [i32; 4],
    pub thrown_as: i32,
    pub lua: Option<Box<GLua>>,
    pub st_lua: [[i32; 2]; MAX_STATES],
    pub lua_hash: u64,
    pub combo: Option<Box<super::combo::Combo>>,
}

impl Default for Def {
    fn default() -> Self {
        Def {
            define_key: String::new(),
            partner_key: String::new(),
            none: false,
            tmpl: 0,
            art_base_ck: 0,
            common_row: Vec::new(),
            cs_tag: Vec::new(),
            fm: None,
            pres: None,
            aud: None,
            ai_like: 0,
            ai_set: false,
            kb_copy: 0,
            thrown_row: [0; 4],
            thrown_as: 0,
            lua: None,
            st_lua: [[-1; 2]; MAX_STATES],
            lua_hash: 0,
            combo: None,
        }
    }
}

/// define.template: the retail fighter a define's native code and tables come from (FighterKind, CharacterKind,
/// animation-row count of its ftData, highest motion number).
pub struct Tmpl {
    pub name: &'static str,
    pub kind: i32,
    pub ck: i32,
    pub rows: i32,
    pub maxmotion: i32,
}
pub const TMPLS: [Tmpl; 8] = [
    Tmpl { name: "mario", kind: 0, ck: 8, rows: 303, maxmotion: 350 },
    Tmpl { name: "captain", kind: 2, ck: 0, rows: 318, maxmotion: 363 },
    Tmpl { name: "fox", kind: 1, ck: 1, rows: 327, maxmotion: 375 },
    Tmpl { name: "falco", kind: 22, ck: 20, rows: 327, maxmotion: 375 },
    Tmpl { name: "marth", kind: 18, ck: 9, rows: 327, maxmotion: 372 },
    Tmpl { name: "puff", kind: 15, ck: 15, rows: 327, maxmotion: 372 },
    Tmpl { name: "sheik", kind: 7, ck: 19, rows: 317, maxmotion: 364 },
    Tmpl { name: "zelda", kind: 19, ck: 18, rows: 311, maxmotion: 358 },
];
fn tmpl_is_pair(t: usize) -> bool {
    TMPLS[t].name == "sheik" || TMPLS[t].name == "zelda"
}
fn tmpl_find(name: &str) -> Option<usize> {
    TMPLS.iter().position(|t| t.name == name)
}

/// GdfCatalog: the definition rows by (mod, key) (strcmp order), each row's profile and resident alias.
#[derive(Default)]
pub struct Catalog {
    pub rows: Vec<(String, String, String)>,
    pub profiles: Vec<i32>,
    pub aliases: Vec<i32>,
}

fn text_ok(s: &str, cap: usize) -> bool {
    !s.is_empty() && s.len() < cap && s.bytes().all(|b| (32..=126).contains(&b))
}

impl Catalog {
    fn clear(&mut self) {
        *self = Catalog::default();
    }
    pub fn count(&self) -> usize {
        self.rows.len()
    }
    /// gdf_add (with gwr_catalog_import's checks): 0 on a bad or duplicate identity.
    fn add(&mut self, m: &str, key: &str, name: &str, profile: i32) -> bool {
        if key.is_empty() || key.len() >= 40 || name.is_empty() || name.len() >= 48 || profile < 0 {
            return false;
        }
        if m.is_empty() || m.len() >= 64 || !text_ok(key, 40) || !text_ok(name, 48) {
            return false;
        }
        let row = (m.to_string(), key.to_string(), name.to_string());
        let pos = self.rows.partition_point(|r| (r.0.as_bytes(), r.1.as_bytes()) < (m.as_bytes(), key.as_bytes()));
        if pos < self.rows.len() && self.rows[pos].0 == m && self.rows[pos].1 == key {
            return false;
        }
        self.rows.insert(pos, row);
        self.profiles.insert(pos, profile);
        self.aliases.insert(pos, -1);
        true
    }
    /// gdf_bind: aliases 127 downward, in catalogue order, skipping the occupied ones (the m-ex slots).
    fn bind(&mut self, occupied: &[bool; 128]) -> bool {
        if self.rows.len() > 94 {
            return false;
        }
        let mut next = Vec::with_capacity(self.rows.len());
        let mut alias: i32 = 127;
        for _ in 0..self.rows.len() {
            while alias >= 34 && occupied[alias as usize] {
                alias -= 1;
            }
            if alias < 34 {
                return false;
            }
            next.push(alias);
            alias -= 1;
        }
        self.aliases = next;
        true
    }
    pub fn alias_for_profile(&self, p: i32) -> i32 {
        self.profiles.iter().position(|&x| x == p).map_or(-1, |i| self.aliases[i])
    }
    pub fn profile_for_kind(&self, kind: i32) -> i32 {
        (0..self.rows.len()).find(|&i| self.aliases[i] >= 34 && self.aliases[i] - 1 == kind).map_or(-1, |i| self.profiles[i])
    }
}

/// gn_retail_kind: a retail fighter by name (Nana excluded), -1 none.
fn retail_kind(name: &str) -> i32 {
    for (i, v) in VANILLA.iter().enumerate() {
        if i == 11 {
            continue;
        }
        if eq_ic(name, v.1) || v.2.is_some_and(|a| eq_ic(name, a)) {
            return i as i32;
        }
    }
    -1
}

fn plain_name(s: &[u8], cap: usize) -> bool {
    !s.is_empty() && s.len() < cap && !s.iter().any(|&c| c == b'\\' || c == b'/' || c == b':') && !s.windows(2).any(|w| w == b"..")
}

fn file_name_ok(s: &str, ext: &str) -> bool {
    let n = s.len();
    if n < ext.len() + 1 || n >= 64 || !eq_ic(&s[n - ext.len()..], ext) || s.contains(['/', '\\', ':']) || s.contains("..") {
        return false;
    }
    s.bytes().all(|b| (33..=126).contains(&b))
}

fn key_ok(s: &str) -> bool {
    s.bytes().enumerate().all(|(i, ch)| ch.is_ascii_lowercase() || ch.is_ascii_digit() || (i > 0 && (ch == b'_' || ch == b'.' || ch == b'-')))
}

fn is_int(v: f64) -> bool {
    v == (v as i32) as f64
}

impl Registry {
    /// gn_fm_str: 1 when the key is absent (and not required) or a plain file name shorter than `cap`.
    fn fm_str(d: &Doc, o: i32, key: &str, cap: usize, required: bool) -> Option<String> {
        let v = d.get(o, key);
        if v < 0 {
            return if required { None } else { Some(String::new()) };
        }
        if !d.is(v, Ty::Str) || !plain_name(&d.n[v as usize].str, cap) {
            return None;
        }
        Some(d.text(v))
    }

    /// gn_define_fighter: the entry-level "fighter" block of a base "none" define.
    fn define_fighter(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str) -> bool {
        let f = d.get(e, "fighter");
        if !d.is(f, Ty::Obj) {
            self.logf(format!("geno: {where_}: a base \"none\" define needs a \"fighter\" block (plan, animation, costumes)"));
            return false;
        }
        let mut fm = Fm::default();
        match (Self::fm_str(d, f, "plan", 64, true), Self::fm_str(d, f, "animation", 64, true)) {
            (Some(a), Some(b)) => {
                fm.plan_file = a;
                fm.anim_file = b;
            }
            _ => {
                self.logf(format!("geno: {where_}: fighter.plan and fighter.animation must be plain file names"));
                return false;
            }
        }
        let cs = d.get(f, "costumes");
        if !d.is(cs, Ty::Arr) || d.n[cs as usize].count < 1 || d.n[cs as usize].count > 255 {
            self.logf(format!("geno: {where_}: fighter.costumes must list 1..255 costumes"));
            return false;
        }
        for (i, c) in d.kids(cs).into_iter().enumerate() {
            let (file, joint, mat) = if d.is(c, Ty::Obj) {
                (Self::fm_str(d, c, "file", 64, true), Self::fm_str(d, c, "joint", 48, true), Self::fm_str(d, c, "matanim", 48, false))
            } else {
                (None, None, None)
            };
            let (Some(file), Some(joint), Some(mat)) = (file, joint, mat) else {
                self.logf(format!("geno: {where_}: fighter.costumes[{i}] needs file and joint (matanim optional)"));
                return false;
            };
            fm.cos_file.push(file);
            fm.cos_joint.push(joint);
            fm.cos_matanim.push(mat);
            let mut team: i8 = -1;
            let mut name = String::new();
            let nv = d.get(c, "name");
            let tv = d.get(c, "team");
            if nv >= 0 {
                if !d.is(nv, Ty::Str) || d.n[nv as usize].str.is_empty() || d.n[nv as usize].str.len() >= 24 {
                    self.logf(format!("geno: {where_}: fighter.costumes[{i}].name must be a string of 1..23 characters"));
                    return false;
                }
                if d.n[nv as usize].str.iter().any(|&b| !(32..=126).contains(&b)) {
                    self.logf(format!("geno: {where_}: fighter.costumes[{i}].name must be printable ASCII"));
                    return false;
                }
                name = d.text(nv);
            }
            if tv >= 0 {
                let t = if d.is(tv, Ty::Str) { d.text(tv) } else { String::new() };
                team = match t.as_str() {
                    "red" => 0,
                    "blue" => 1,
                    "green" => 2,
                    _ => {
                        self.logf(format!("geno: {where_}: fighter.costumes[{i}].team must be \"red\", \"blue\" or \"green\""));
                        return false;
                    }
                };
            }
            fm.cos_name.push(name);
            fm.cos_team.push(team);
        }
        let rv = d.get(f, "rows");
        if rv >= 0 {
            if !d.is(rv, Ty::Arr) || d.n[rv as usize].count as usize > GPL_MAX_OWN_ROWS {
                self.logf(format!("geno: {where_}: fighter.rows must be a list of at most {GPL_MAX_OWN_ROWS} clip names"));
                return false;
            }
            for c in d.kids(rv) {
                if !d.is(c, Ty::Str) || d.n[c as usize].str.is_empty() || d.n[c as usize].str.len() >= 32 {
                    self.logf(format!("geno: {where_}: fighter.rows[{}] must be a clip name of 1..31 characters", fm.own_clip.len()));
                    return false;
                }
                fm.own_clip.push(d.text(c));
            }
        }
        p.def.fm = Some(Box::new(fm));
        true
    }

    /// gn_define_presentation: the optional entry-level "presentation".
    fn define_presentation(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str) -> bool {
        const KEYS: [&str; GN_PRES_KINDS] = ["icon", "portrait", "stock", "emblem", "results"];
        let pv = d.get(e, "presentation");
        if pv < 0 {
            return true;
        }
        if !d.is(pv, Ty::Obj) {
            self.logf(format!("geno: {where_}: presentation must be an object"));
            return false;
        }
        for c in d.kids(pv) {
            let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
            if !KEYS.contains(&k.as_str()) {
                self.logf(format!("geno: {where_}: presentation key \"{k}\" is not known (icon, portrait, stock, emblem, results)"));
                return false;
            }
        }
        for k in KEYS {
            let v = d.get(pv, k);
            if v < 0 {
                continue;
            }
            let ok = d.is(v, Ty::Str) || (d.is(v, Ty::Arr) && (1..=255).contains(&d.n[v as usize].count));
            if !ok {
                self.logf(format!("geno: {where_}: presentation list must have 1..255 entries"));
                return false;
            }
        }
        let pres_ok = |s: &str| file_name_ok(s, ".gxtex") && s.len() >= 7;
        let mut pr = Pres::default();
        for (k, key) in KEYS.iter().enumerate() {
            let v = d.get(pv, key);
            if v < 0 {
                continue;
            }
            let mut good = true;
            if d.is(v, Ty::Str) {
                let s = d.text(v);
                if pres_ok(&s) {
                    pr.file[k].push(s);
                } else {
                    good = false;
                }
            } else if d.is(v, Ty::Arr) && k != GN_PRES_ICON && k != GN_PRES_EMBLEM {
                for q in d.kids(v) {
                    let s = if d.is(q, Ty::Str) { d.text(q) } else { String::new() };
                    if !d.is(q, Ty::Str) || !pres_ok(&s) {
                        good = false;
                        break;
                    }
                    pr.file[k].push(s);
                }
            } else {
                good = false;
            }
            if !good {
                self.logf(format!("geno: {where_}: presentation.{key} must be one plain *.gxtex file name (portrait, stock and results: or 1..255 of them; icon and emblem: one)"));
                return false;
            }
        }
        p.def.pres = Some(Box::new(pr));
        true
    }

    /// gn_define_s6_keys: ai, kirby_copy, thrown_as and audio.
    fn define_s6_keys(&mut self, p: &mut Profile, d: &Doc, e: i32, version: i32, where_: &str) -> bool {
        let df = &mut p.def;
        df.ai_like = -1;
        df.ai_set = false;
        df.kb_copy = -1;
        df.thrown_as = 2;
        df.thrown_row = [-1; 4];
        if version < 10 && (d.get(e, "ai") >= 0 || d.get(e, "kirby_copy") >= 0 || d.get(e, "thrown_as") >= 0) {
            self.logf(format!("geno: {where_}: ai, kirby_copy and thrown_as need \"geno\": 10"));
            return false;
        }
        let v = d.get(e, "thrown_as");
        if v >= 0 {
            if !p.def.none {
                self.logf(format!("geno: {where_}: thrown_as is for a base \"none\" define (a base \"mario\" define is thrown with Mario's rows)"));
                p.def.thrown_as = 2;
                return false;
            }
            if d.is(v, Ty::Obj) {
                const KEYS: [&str; 4] = ["f", "b", "hi", "lw"];
                for c2 in d.kids(v) {
                    let k = String::from_utf8_lossy(&d.n[c2 as usize].key).into_owned();
                    if k != "default" && !KEYS.contains(&k.as_str()) {
                        self.logf(format!("geno: {where_}: thrown_as key \"{k}\" is not known (default, f, b, hi, lw)"));
                        p.def.thrown_as = 2;
                        return false;
                    }
                }
                let c2 = d.get(v, "default");
                if c2 >= 0 {
                    let k = if d.is(c2, Ty::Str) { retail_kind(&d.text(c2)) } else { -1 };
                    p.def.thrown_as = k;
                    if !d.is(c2, Ty::Str) || k < 0 {
                        self.logf(format!("geno: {where_}: thrown_as.default must name a retail fighter"));
                        p.def.thrown_as = 2;
                        return false;
                    }
                }
                for (k2, key) in KEYS.iter().enumerate() {
                    let c2 = d.get(v, key);
                    if c2 < 0 {
                        continue;
                    }
                    let k = if d.is(c2, Ty::Str) { retail_kind(&d.text(c2)) } else { -1 };
                    p.def.thrown_row[k2] = k;
                    if k < 0 {
                        self.logf(format!("geno: {where_}: thrown_as.{key} must name a retail fighter"));
                        p.def.thrown_as = 2;
                        p.def.thrown_row = [-1; 4];
                        return false;
                    }
                }
            } else {
                let k = if d.is(v, Ty::Str) { retail_kind(&d.text(v)) } else { -1 };
                p.def.thrown_as = k;
                if k < 0 {
                    self.logf(format!("geno: {where_}: thrown_as must name a retail fighter (mario, fox, captain, donkey, kirby, koopa, link, seak, ness, peach, popo, pikachu, samus, yoshi, purin, mewtwo, luigi, mars, zelda, clink, drmario, falco, pichu, gamewatch, ganon, emblem) or an object {{default, f, b, hi, lw}}"));
                    p.def.thrown_as = 2;
                    return false;
                }
            }
        }
        if version < 9 && d.get(e, "audio") >= 0 {
            self.logf(format!("geno: {where_}: audio needs \"geno\": 9"));
            return false;
        }
        let v = d.get(e, "ai");
        if v >= 0 {
            if !d.is(v, Ty::Obj) {
                self.logf(format!("geno: {where_}: ai must be an object {{\"like\": <retail fighter>}}"));
                return false;
            }
            for c in d.kids(v) {
                let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                if k != "like" {
                    self.logf(format!("geno: {where_}: ai key \"{k}\" is not known (like)"));
                    return false;
                }
            }
            let lk = d.get(v, "like");
            let k = if d.is(lk, Ty::Str) { retail_kind(&d.text(lk)) } else { -1 };
            p.def.ai_like = k;
            if k < 0 {
                self.logf(format!("geno: {where_}: ai.like must name a retail fighter (mario, fox, captain, donkey, kirby, koopa, link, seak, ness, peach, popo, pikachu, samus, yoshi, purin, mewtwo, luigi, mars, zelda, clink, drmario, falco, pichu, gamewatch, ganon, emblem)"));
                p.def.ai_like = -1;
                return false;
            }
            p.def.ai_set = true;
        }
        let v = d.get(e, "kirby_copy");
        if v >= 0 {
            let t = if d.is(v, Ty::Str) { d.text(v) } else { String::new() };
            if t == "none" {
                p.def.kb_copy = -1;
            } else if let Some(r) = t.strip_prefix("retail:").map(retail_kind).filter(|&k| k >= 0 && k != 4) {
                p.def.kb_copy = r;
            } else {
                self.logf(format!("geno: {where_}: kirby_copy must be \"none\" or \"retail:<fighter>\" (a retail fighter other than Kirby, e.g. \"retail:mario\")"));
                p.def.kb_copy = -1;
                return false;
            }
        }
        let v = d.get(e, "audio");
        if v >= 0 {
            if !d.is(v, Ty::Obj) {
                self.logf(format!("geno: {where_}: audio must be an object"));
                return false;
            }
            for c in d.kids(v) {
                let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                if k != "announcer" && k != "voice" {
                    self.logf(format!("geno: {where_}: audio key \"{k}\" is not known (announcer, voice)"));
                    return false;
                }
            }
            let mut a = Aud::default();
            let c = d.get(v, "announcer");
            if c >= 0 {
                if !d.is(c, Ty::Str) || !file_name_ok(&d.text(c), ".gnsnd") {
                    self.logf(format!("geno: {where_}: audio.announcer must be one plain *.gnsnd file name"));
                    return false;
                }
                a.announcer = d.text(c);
            }
            let vo = d.get(v, "voice");
            if vo >= 0 {
                if !d.is(vo, Ty::Arr) || d.n[vo as usize].count < 1 || d.n[vo as usize].count as usize > GN_MAX_VOICE {
                    self.logf(format!("geno: {where_}: audio.voice must list 1..{GN_MAX_VOICE} rows"));
                    return false;
                }
                let bad = |r: &mut Registry| {
                    r.logf(format!("geno: {where_}: audio.voice rows are {{\"sfx\": 1..999999, \"file\": \"x.gnsnd\", \"volume\": 0..127 (optional)}}, each sound once"));
                    false
                };
                for c in d.kids(vo) {
                    if !d.is(c, Ty::Obj) {
                        return bad(self);
                    }
                    for x in d.kids(c) {
                        let k = String::from_utf8_lossy(&d.n[x as usize].key).into_owned();
                        if !["sfx", "file", "volume", "stoppable", "while_states"].contains(&k.as_str()) {
                            return bad(self);
                        }
                    }
                    let (sx, fl, vl) = (d.get(c, "sfx"), d.get(c, "file"), d.get(c, "volume"));
                    let mut stop = false;
                    let x = d.get(c, "stoppable");
                    if x >= 0 {
                        if !d.is(x, Ty::Bool) {
                            return bad(self);
                        }
                        stop = d.n[x as usize].num != 0.0;
                    }
                    if !d.is(sx, Ty::Num) || !(1.0..=999999.0).contains(&d.n[sx as usize].num) || !is_int(d.n[sx as usize].num) {
                        return bad(self);
                    }
                    if !d.is(fl, Ty::Str) || !file_name_ok(&d.text(fl), ".gnsnd") {
                        return bad(self);
                    }
                    let mut vol = 127;
                    if vl >= 0 {
                        if !d.is(vl, Ty::Num) || !(0.0..=127.0).contains(&d.n[vl as usize].num) || !is_int(d.n[vl as usize].num) {
                            return bad(self);
                        }
                        vol = d.n[vl as usize].num as i32;
                    }
                    let sfx = d.n[sx as usize].num as i32;
                    if a.voice_sfx.contains(&sfx) {
                        self.logf(format!("geno: {where_}: audio.voice names sound {sfx} twice"));
                        return false;
                    }
                    a.voice_sfx.push(sfx);
                    a.voice_vol.push(vol);
                    a.voice_stop.push(stop);
                    a.voice_states.push(0);
                    a.voice_file.push(d.text(fl));
                }
            }
            p.def.aud = Some(Box::new(a));
        }
        true
    }

    /// gn_define_audio_states: a stoppable voice row's while_states, once every state name is known.
    pub(super) fn define_audio_states(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str) -> bool {
        if p.def.aud.is_none() {
            return true;
        }
        let a = d.get(e, "audio");
        let v = d.get(a, "voice");
        if v < 0 {
            return true;
        }
        for (n, c) in d.kids(v).into_iter().enumerate() {
            let list = d.get(c, "while_states");
            if list < 0 {
                continue;
            }
            let mut mask = 0u64;
            let stop = p.def.aud.as_ref().unwrap().voice_stop.get(n).copied().unwrap_or(false);
            let mut ok = stop && d.is(list, Ty::Arr) && (1..=MAX_STATES as i32).contains(&d.n[list as usize].count);
            if ok {
                for x in d.kids(list) {
                    if !d.is(x, Ty::Str) {
                        ok = false;
                        break;
                    }
                    let name = d.text(x);
                    let s = (0..p.nstate).find(|&s| p.st_name[s] == name);
                    match s {
                        Some(s) if mask & (1u64 << s) == 0 => mask |= 1u64 << s,
                        _ => {
                            ok = false;
                            break;
                        }
                    }
                }
            }
            if !ok {
                self.logf(format!("geno: {where_}: audio.voice[{n}].while_states requires stoppable: true and 1..{MAX_STATES} distinct Geno state names"));
                return false;
            }
            p.def.aud.as_mut().unwrap().voice_states[n] = mask;
        }
        true
    }

    /// gn_sounds_check: strict validation of the sound table and of every article reference (a define's load error).
    fn sounds_check(&mut self, d: &Doc, e: i32, where_: &str) -> bool {
        let s = d.get(e, "sounds");
        let a = d.get(e, "articles");
        let names: Vec<String> = if d.is(s, Ty::Arr) {
            d.kids(s).into_iter().map(|c| { let n = d.get(c, "name"); if d.is(n, Ty::Str) { d.text(n) } else { String::new() } }).collect()
        } else {
            Vec::new()
        };
        if s >= 0 {
            if !d.is(s, Ty::Arr) {
                self.logf(format!("geno: {where_}: \"sounds\" must be an array"));
                return false;
            }
            for (n, c) in d.kids(s).into_iter().enumerate() {
                let (nm, id, vol) = (d.get(c, "name"), d.get(c, "retail_sfx"), d.get(c, "volume"));
                if n >= GENO_MAX_SOUNDS as usize {
                    self.logf(format!("geno: {where_}: more than {GENO_MAX_SOUNDS} sounds"));
                    return false;
                }
                if !d.is(c, Ty::Obj) || !d.is(nm, Ty::Str) || d.n[nm as usize].str.is_empty() || d.n[nm as usize].str.len() >= 32 {
                    self.logf(format!("geno: {where_}: sound {n} needs a \"name\" of 1..31 characters"));
                    return false;
                }
                let nms = d.text(nm);
                if !d.is(id, Ty::Num) || !is_int(d.n[id as usize].num) || !(1.0..=999999.0).contains(&d.n[id as usize].num) {
                    self.logf(format!("geno: {where_}: sound \"{nms}\": \"retail_sfx\" must be an engine sound id, an integer 1..999999"));
                    return false;
                }
                if vol >= 0 && (!d.is(vol, Ty::Num) || !is_int(d.n[vol as usize].num) || !(0.0..=127.0).contains(&d.n[vol as usize].num)) {
                    self.logf(format!("geno: {where_}: sound \"{nms}\": \"volume\" must be an integer 0..127"));
                    return false;
                }
                if names[..n].iter().any(|o| *o == nms) {
                    self.logf(format!("geno: {where_}: duplicate sound name \"{nms}\""));
                    return false;
                }
            }
        }
        if d.is(a, Ty::Arr) {
            for (n, c) in d.kids(a).into_iter().enumerate() {
                for key in ["spawn_sound", "end_sound"] {
                    let r = d.get(c, key);
                    if r < 0 {
                        continue;
                    }
                    let found = d.is(r, Ty::Str) && d.is(s, Ty::Arr) && names.iter().any(|x| *x == d.text(r));
                    if !found {
                        let shown = if d.is(r, Ty::Str) { d.text(r) } else { "a value that".into() };
                        self.logf(format!("geno: {where_}: article {n} \"{key}\": {shown} is not a name in \"sounds\" (no donor fallback)"));
                        return false;
                    }
                }
            }
        }
        true
    }

    /// gn_define_parse: false refuses the entry (the reason is logged).
    pub(super) fn define_parse(&mut self, p: &mut Profile, d: &Doc, e: i32, version: i32, where_: &str) -> bool {
        if self.define_parse_inner(p, d, e, version, where_) {
            p.defined = true;
            return true;
        }
        self.logf(format!("geno: {where_}: define refused (requires v6, v7 or v8 native Mario/common v1/retail references; no attach; v7 for special_attributes, fx_bindings and attributes past the first 40; v8 for articles and sounds)"));
        false
    }

    fn define_parse_inner(&mut self, p: &mut Profile, d: &Doc, e: i32, version: i32, where_: &str) -> bool {
        const KEYS: [&str; 5] = ["key", "name", "base", "common", "resources"];
        let def = d.get(e, "define");
        if def < 0 {
            return false;
        }
        let has = |k: &str| d.get(e, k) >= 0;
        if version < 6
            || version > GENO_VERSION as i32
            || has("attach")
            || !d.is(def, Ty::Obj)
            || (version < 8 && (has("articles") || has("sounds")))
            || (version < 7 && (has("fx_bindings") || has("special_attributes")))
        {
            if version < 8 && (has("articles") || has("sounds")) {
                self.logf(format!("geno: {where_}: define articles and sounds need \"geno\": 8"));
            }
            return false;
        }
        for c in d.kids(def) {
            let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
            let known = KEYS.contains(&k.as_str());
            if !known && (k == "template" || k == "transform_partner") {
                continue;
            }
            if !known || !d.is(c, Ty::Str) {
                return false;
            }
        }
        for k in KEYS {
            if !d.is(d.get(def, k), Ty::Str) {
                return false;
            }
        }
        let base = d.text(d.get(def, "base"));
        if base == "none" {
            if version < 9 {
                self.logf(format!("geno: {where_}: base \"none\" needs \"geno\": 9"));
                return false;
            }
            p.def.none = true;
        } else if base != "mario" {
            return false;
        }
        p.def.tmpl = 0;
        let v = d.get(def, "template");
        if v >= 0 {
            let t = if d.is(v, Ty::Str) { tmpl_find(&d.text(v)) } else { None };
            match t {
                Some(t) if p.def.none && version >= 10 => p.def.tmpl = t,
                _ => {
                    self.logf(format!("geno: {where_}: define.template must be \"mario\", \"captain\", \"fox\", \"falco\", \"marth\", \"puff\", \"sheik\" or \"zelda\", on a base \"none\" define of \"geno\": 10"));
                    return false;
                }
            }
        }
        p.def.partner_key.clear();
        let pv = d.get(def, "transform_partner");
        if pv >= 0 {
            let s = if d.is(pv, Ty::Str) { d.text(pv) } else { String::new() };
            if !p.def.none || version < 10 || !tmpl_is_pair(p.def.tmpl) || s.is_empty() || s.len() >= 40 {
                self.logf(format!("geno: {where_}: define.transform_partner is the key of the other define of a pair, on a base \"none\" define whose template is \"zelda\" or \"sheik\" (\"geno\": 10)"));
                return false;
            }
            if !key_ok(&s) {
                return false;
            }
            p.def.partner_key = s;
        } else if p.def.none && tmpl_is_pair(p.def.tmpl) {
            self.logf(format!("geno: {where_}: a \"zelda\" or \"sheik\" template needs define.transform_partner (Down B changes into the partner; with none the fighter would crash)"));
            return false;
        }
        // A native-code default is not a visual identity: only a retail base or a named template lends its disc art.
        p.def.art_base_ck = if p.def.none && v < 0 { -1 } else { TMPLS[p.def.tmpl].ck };
        if d.text(d.get(def, "common")) != "melee.common.v1" {
            return false;
        }
        if d.text(d.get(def, "resources")) != if p.def.none { "mod:files" } else { "retail:mario" } {
            return false;
        }
        if p.def.none && !self.define_fighter(p, d, e, where_) {
            return false;
        }
        let key = d.text(d.get(def, "key"));
        if key.is_empty() || key.len() >= 40 || key.contains("..") || !key_ok(&key) {
            return false;
        }
        p.def.define_key = key;
        let name = d.text(d.get(def, "name"));
        if name.is_empty() || name.len() >= 48 || name.bytes().any(|b| !(32..=126).contains(&b)) {
            return false;
        }
        p.name = name;
        let v = d.get(e, "attributes");
        if v >= 0 {
            if !d.is(v, Ty::Obj) || d.n[v as usize].count > GENO_MAX_ATTRS as i32 {
                return false;
            }
            for c in d.kids(v) {
                let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                let ai = super::registry::attr_find(&k);
                if !d.is(c, Ty::Num) || ai < 0 || (version < 7 && ai >= 40) {
                    let why = if !d.is(c, Ty::Num) {
                        "is not a number"
                    } else if ai < 0 {
                        "is not a common attribute Geno knows"
                    } else {
                        "needs \"geno\": 7"
                    };
                    self.logf(format!("geno: {where_}: define attribute \"{k}\" {why}"));
                    return false;
                }
            }
        }
        if !self.sounds_check(d, e, where_) {
            return false;
        }
        if version < 9 && d.get(e, "presentation") >= 0 {
            self.logf(format!("geno: {where_}: presentation needs \"geno\": 9"));
            return false;
        }
        if !self.define_presentation(p, d, e, where_) {
            return false;
        }
        if !self.define_s6_keys(p, d, e, version, where_) {
            return false;
        }
        let own = p.def.fm.as_ref().map_or(0, |f| f.own_clip.len()) as i32;
        let rows = TMPLS[p.def.tmpl].rows;
        let v = d.get(e, "subactions");
        if v >= 0 {
            if !d.is(v, Ty::Arr) {
                return false;
            }
            for c in d.kids(v) {
                let ix = d.get(c, "index");
                if !d.is(ix, Ty::Num) {
                    return false;
                }
                let n = d.n[ix as usize].num;
                if n < 0.0 || n >= (rows + own) as f64 || !is_int(n) {
                    return false;
                }
            }
        }
        let v = d.get(e, "common_states");
        if v >= 0 {
            const FIELDS: [&str; 9] = ["motion", "like", "subaction", "flags", "move_id", "anim", "iasa", "phys", "coll"];
            if !d.is(v, Ty::Arr) || d.n[v as usize].count > 64 {
                return false;
            }
            for row in d.kids(v) {
                if !d.is(row, Ty::Obj) {
                    return false;
                }
                let mut r = [-1i32; 9];
                let tag = super::registry::move_tag(d, row);
                for x in d.kids(row) {
                    let k = String::from_utf8_lossy(&d.n[x as usize].key).into_owned();
                    if k == "move_tag" {
                        continue;
                    }
                    let Some(j) = FIELDS.iter().position(|f| *f == k) else { return false };
                    let kv: i32 = if j < 5 {
                        let hi: f64 = match j {
                            0 | 1 => TMPLS[p.def.tmpl].maxmotion as f64,
                            2 => (rows - 1 + own) as f64,
                            4 => 255.0,
                            _ => 2147483647.0,
                        };
                        if !d.is(x, Ty::Num) || d.n[x as usize].num < 0.0 || d.n[x as usize].num > hi || !is_int(d.n[x as usize].num) {
                            return false;
                        }
                        d.n[x as usize].num as i32
                    } else {
                        if !d.is(x, Ty::Str) {
                            return false;
                        }
                        let k = callback_find((j - 5) as u32, &d.text(x));
                        if k < 0 && k != GENO_CB_LIKE as i32 {
                            return false;
                        }
                        k
                    };
                    r[j] = kv;
                }
                if r[0] < 0 || p.def.common_row.iter().any(|o| o[0] == r[0]) {
                    return false;
                }
                p.def.common_row.push(r);
                p.def.cs_tag.push(tag);
            }
        }
        for key in ["states", "motion_anims"] {
            let v = d.get(e, key);
            if v < 0 {
                continue;
            }
            if !d.is(v, Ty::Arr) {
                return false;
            }
            for c in d.kids(v) {
                let sub = d.get(c, "subaction");
                if d.is(sub, Ty::Num) {
                    let n = d.n[sub as usize].num;
                    if n < 0.0 || n >= (rows + own) as f64 || !is_int(n) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// gn_define_bind: the separate, source-independent catalogue of the admitted defines and their resident aliases.
    pub(super) fn define_bind(&mut self, occupied_mex: &[bool]) {
        self.definitions.clear();
        if !self.p.iter().any(|p| p.defined) {
            return;
        }
        let n = self.p.len();
        let mut drop = vec![false; n];
        // a Zelda / Sheik pair is admitted together: each member's partner must name it back
        loop {
            let mut changed = false;
            for i in 0..n {
                if !self.p[i].defined || drop[i] || self.p[i].def.partner_key.is_empty() {
                    continue;
                }
                let pk = self.p[i].def.partner_key.clone();
                let found = (0..n).find(|&j| j != i && self.p[j].defined && !drop[j] && self.p[j].def.define_key == pk);
                let ok = found.is_some_and(|j| {
                    let q = &self.p[j].def;
                    q.tmpl != self.p[i].def.tmpl && tmpl_is_pair(q.tmpl) && q.partner_key == self.p[i].def.define_key
                });
                if !ok {
                    let m = format!(
                        "geno: {}: define refused: its transform partner \"{}\" is not a loaded define of the other template (zelda / sheik) naming this one back",
                        self.p[i].name, pk
                    );
                    self.logf(m);
                    drop[i] = true;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut occupied = [false; 128];
        for (s, &o) in occupied_mex.iter().enumerate() {
            if o && 34 + s < 128 {
                occupied[34 + s] = true;
            }
        }
        let mut ok = true;
        for i in 0..n {
            if !self.p[i].defined || drop[i] {
                continue;
            }
            let (m, k, nm) = (self.p[i].mod_id.clone(), self.p[i].def.define_key.clone(), self.p[i].name.clone());
            if !self.definitions.add(&m, &k, &nm, i as i32) {
                self.logf(format!("geno: {nm}: native catalogue refused duplicate identity/allocation/id exhaustion"));
                ok = false;
                break;
            }
        }
        if ok {
            ok = self.definitions.bind(&occupied);
        }
        if !ok {
            self.definitions.clear();
            self.logf("geno: define admission refused: no unoccupied resident aliases or invalid catalogue; definitions inactive".into());
        }
        for i in 0..n {
            if self.p[i].defined {
                let ck = self.definitions.alias_for_profile(i as i32);
                self.p[i].kind = if ck >= 34 { ck - 1 } else { -1 };
            }
        }
    }

    // ---- slice 5: the "lua" block (geno_lua_registry.inc) ----

    /// gn_lua_parse: true when the entry is fine (with or without Lua); false refuses it.
    pub(super) fn lua_parse(&mut self, p: &mut Profile, d: &Doc, e: i32, env: &dyn Env, mod_id: &str, where_: &str, version: i32) -> bool {
        let lb = d.get(e, "lua");
        let st = d.get(e, "states");
        let any = d.is(st, Ty::Arr) && d.kids(st).into_iter().any(|c| d.get(c, "lua") >= 0);
        if lb < 0 && !any {
            return true;
        }
        if !p.defined {
            self.logf(format!("geno: {where_}: fighter Lua is for a define; an attach entry cannot carry a \"lua\" block - refused"));
            return false;
        }
        if version < 10 {
            self.logf(format!("geno: {where_}: \"lua\" needs \"geno\": 10 - refused"));
            return false;
        }
        if !d.is(lb, Ty::Obj) {
            self.logf(format!("geno: {where_}: a state names Lua functions but the entry has no \"lua\" block - refused"));
            return false;
        }
        let (sc, so, sl) = (d.get(lb, "script"), d.get(lb, "source"), d.get(lb, "state"));
        if (sc < 0) == (so < 0) || (sc >= 0 && !d.is(sc, Ty::Str)) || (so >= 0 && !d.is(so, Ty::Str)) {
            self.logf(format!("geno: {where_}: lua needs exactly one of \"script\" (a file of the mod) and \"source\" (inline text) - refused"));
            return false;
        }
        let src: Vec<u8> = if sc >= 0 {
            let rel = d.text(sc);
            if rel.is_empty() || rel.contains("..") || rel.contains(':') || rel.starts_with('/') || rel.starts_with('\\') {
                self.logf(format!("geno: {where_}: lua.script \"{rel}\" must be a relative path inside the mod - refused"));
                return false;
            }
            match env.read_mod_file(mod_id, &rel).filter(|t| t.len() <= 1 << 20) {
                // gn_read_file + strlen: the text up to its first NUL
                Some(t) => t.split(|&b| b == 0).next().unwrap_or(&[]).to_vec(),
                None => {
                    self.logf(format!("geno: {where_}: cannot read lua.script {mod_id}/{rel} - refused"));
                    return false;
                }
            }
        } else {
            d.n[so as usize].str.split(|&b| b == 0).next().unwrap_or(&[]).to_vec()
        };
        const MAX_SOURCE: usize = 64 * 1024;
        if src.is_empty() || src.len() > MAX_SOURCE {
            self.logf(format!("geno: {where_}: the Lua module must be 1..{MAX_SOURCE} bytes - refused"));
            return false;
        }
        let mut names: Vec<String> = Vec::new();
        let mut types: Vec<i32> = Vec::new();
        if sl >= 0 {
            if !d.is(sl, Ty::Obj) {
                self.logf(format!("geno: {where_}: lua.state must be an object of name: type - refused"));
                return false;
            }
            for c in d.kids(sl) {
                let ty = if d.is(c, Ty::Str) { d.text(c) } else { String::new() };
                let t = match ty.as_str() {
                    "int" => GLUA_INT,
                    "float" => GLUA_FLOAT,
                    "bool" => GLUA_BOOL,
                    _ => -1,
                };
                let k = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                if names.len() >= GENO_LUA_STATE_SLOTS as usize {
                    self.logf(format!("geno: {where_}: lua.state has more than {GENO_LUA_STATE_SLOTS} slots - refused"));
                    return false;
                }
                let name_ok = !k.is_empty()
                    && k.len() < GLUA_SLOT_NAME
                    && k.bytes().enumerate().all(|(i, c)| c.is_ascii_alphabetic() || c == b'_' || (i > 0 && c.is_ascii_digit()));
                if t < 0 || !name_ok {
                    self.logf(format!("geno: {where_}: lua.state \"{k}\": a name of 1..{} letters, digits and underscores, and a type of int / float / bool - refused", GLUA_SLOT_NAME - 1));
                    return false;
                }
                if names.contains(&k) {
                    self.logf(format!("geno: {where_}: lua.state \"{k}\" is declared twice - refused"));
                    return false;
                }
                names.push(k);
                types.push(t);
            }
        }
        let Some(mut g) = GLua::new() else {
            self.logf(format!("geno: {where_}: Lua could not allocate - refused"));
            return false;
        };
        let tnames: Vec<String> = (0..p.nstate.min(MAX_STATES)).map(|k| cap(&p.st_name[k], GLUA_NAME)).collect();
        g.set_layout(&names, &types);
        g.set_targets(&tnames);
        let chunk = format!("={}", if sc >= 0 { d.text(sc) } else { "lua.source".into() });
        let chunk = cap(&chunk, 96);
        if !g.load(&src, &chunk) {
            self.logf(format!("geno: {where_}: the Lua module was refused: {}", g.err));
            return false;
        }
        p.def.st_lua = [[-1; 2]; MAX_STATES];
        if d.is(st, Ty::Arr) {
            const PHASES: [&str; 2] = ["enter", "frame"];
            for (k, c) in d.kids(st).into_iter().enumerate().take(p.nstate) {
                let lk = d.get(c, "lua");
                if lk < 0 {
                    continue;
                }
                if !d.is(lk, Ty::Obj) {
                    self.logf(format!("geno: {where_}: state {}: \"lua\" must be an object {{enter, frame}} - refused", p.st_name[k]));
                    return false;
                }
                for (ph, phase) in PHASES.iter().enumerate() {
                    let v = d.get(lk, phase);
                    if v < 0 {
                        continue;
                    }
                    let f = if d.is(v, Ty::Str) { g.find(&d.text(v)) } else { -1 };
                    p.def.st_lua[k][ph] = f;
                    if f < 0 {
                        let shown = if d.is(v, Ty::Str) { d.text(v) } else { "?".into() };
                        self.logf(format!("geno: {where_}: state {}: lua.{phase} \"{shown}\" is not a function of the module - refused", p.st_name[k]));
                        return false;
                    }
                }
                if p.def.st_lua[k][GENO_LUA_PHASE_FRAME as usize] >= 0 && p.st_cb[k][GENO_CB_ANIM as usize] < 0 {
                    p.st_cb[k][GENO_CB_ANIM as usize] = callback_find(GENO_CB_ANIM, "lua");
                    if p.st_cb[k][GENO_CB_ANIM as usize] < 0 {
                        self.logf(format!("geno: {where_}: this build has no \"lua\" anim callback - refused"));
                        return false;
                    }
                }
            }
        }
        let mut h = lua_mix_text(0x4C55_412D_5631_0000, &src);
        for (k, nm) in names.iter().enumerate() {
            let mut b = nm.as_bytes().to_vec();
            b.push(0);
            h = lua_mix_text(h, &b) ^ types[k] as u64;
        }
        p.def.lua_hash = h;
        p.def.lua = Some(g);
        true
    }
}

fn lua_mix_text(mut h: u64, s: &[u8]) -> u64 {
    for &b in s {
        h = (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The define's content id folded with its Lua module (gn_add_fighter's last step).
pub(super) fn fold_lua(id: u64, p: &Profile) -> u64 {
    if p.def.lua.is_some() { mix(id, p.def.lua_hash) } else { id }
}

// ---- the scalar API (gw_Geno_Define*), on an installed registry ----

impl Registry {
    fn def_profile_for_kind(&self, kind: i32) -> Option<&Profile> {
        let p = self.definitions.profile_for_kind(kind);
        if p >= 0 { Some(&self.p[p as usize]) } else { None }
    }
    pub fn define_count(&self) -> i32 {
        self.definitions.count() as i32
    }
    pub fn define_ck_at(&self, i: i32) -> i32 {
        if i >= 0 && (i as usize) < self.definitions.count() { self.definitions.aliases[i as usize] } else { -1 }
    }
    pub fn common_move_tag(&self, p: i32, motion: i32) -> i32 {
        if p < 0 || p as usize >= self.p.len() {
            return 0;
        }
        let df = &self.p[p as usize].def;
        df.common_row.iter().position(|r| r[0] == motion).map_or(0, |i| df.cs_tag[i])
    }
    pub fn define_base_kind(&self, kind: i32) -> i32 {
        self.def_profile_for_kind(kind).map_or(-1, |p| TMPLS[p.def.tmpl].kind)
    }
    pub fn define_partner_kind(&self, kind: i32) -> i32 {
        let Some(p) = self.def_profile_for_kind(kind) else { return -1 };
        if p.def.partner_key.is_empty() {
            return -1;
        }
        self.p.iter().find(|q| q.defined && q.def.define_key == p.def.partner_key).map_or(-1, |q| if q.kind >= 0 { q.kind } else { -1 })
    }
    pub fn define_base_ck(&self, ck: i32) -> i32 {
        let b = self.define_base_kind(ck - 1);
        if b < 0 {
            return -1;
        }
        TMPLS.iter().find(|t| t.kind == b).map_or(-1, |t| t.ck)
    }
    pub fn define_name(&self, ck: i32) -> Option<&str> {
        self.def_profile_for_kind(ck - 1).map(|p| p.name.as_str())
    }
    pub fn define_resolve(&self, token: &str) -> i32 {
        self.p
            .iter()
            .find(|p| p.defined && p.kind >= 0 && (eq_ic(token, &p.def.define_key) || eq_ic(token, &p.name)))
            .map_or(-1, |p| p.kind + 1)
    }
    pub fn define_identity_word(&self, kind: i32, high: bool) -> i32 {
        self.def_profile_for_kind(kind).map_or(0, |p| if high { (p.id >> 32) as i32 } else { p.id as i32 })
    }
    pub fn define_override_count(&self, kind: i32) -> i32 {
        self.def_profile_for_kind(kind).map_or(0, |p| p.def.common_row.len() as i32)
    }
    pub fn define_override_field(&self, kind: i32, row: i32, field: i32) -> i32 {
        match self.def_profile_for_kind(kind) {
            Some(p) if row >= 0 && (row as usize) < p.def.common_row.len() && (0..9).contains(&field) => p.def.common_row[row as usize][field as usize],
            _ => -1,
        }
    }
    fn fm_for_kind(&self, kind: i32) -> Option<&Fm> {
        self.def_profile_for_kind(kind).and_then(|p| p.def.fm.as_deref())
    }
    pub fn define_is_none(&self, kind: i32) -> i32 {
        self.fm_for_kind(kind).is_some() as i32
    }
    pub fn define_template_skeleton(&self, kind: i32) -> i32 {
        (self.fm_for_kind(kind).is_some() && self.define_base_kind(kind) > 0) as i32
    }
    /// gw_Geno_DefineCostumeCount without installed skins (gw_skins_boot.inc is not hosted).
    pub fn define_costume_count(&self, kind: i32) -> i32 {
        self.fm_for_kind(kind).map_or(0, |f| f.cos_file.len() as i32)
    }
    pub fn define_ai_like(&self, kind: i32) -> i32 {
        if kind < 0x21 {
            return -1;
        }
        match self.def_profile_for_kind(kind) {
            Some(p) if p.def.ai_set => p.def.ai_like,
            _ => -1,
        }
    }
    pub fn define_kirby_copy(&self, kind: i32) -> i32 {
        if kind < 0x21 {
            return -2;
        }
        self.def_profile_for_kind(kind).map_or(-2, |p| p.def.kb_copy)
    }
    pub fn define_thrown_as(&self, kind: i32) -> i32 {
        if kind < 0x21 || self.fm_for_kind(kind).is_none() {
            return -1;
        }
        self.def_profile_for_kind(kind).map_or(-1, |p| p.def.thrown_as)
    }
    pub fn define_thrown_as_row(&self, kind: i32, row: i32) -> i32 {
        if kind < 0x21 || self.fm_for_kind(kind).is_none() {
            return -1;
        }
        let Some(p) = self.def_profile_for_kind(kind) else { return -1 };
        if (0..4).contains(&row) && p.def.thrown_row[row as usize] >= 0 { p.def.thrown_row[row as usize] } else { p.def.thrown_as }
    }
    /// gn_throw_banks_needed: the most distinct stand-in banks one define needs.
    pub fn throw_banks_needed(&self) -> i32 {
        let mut most = 1;
        for p in self.p.iter().filter(|p| p.def.none) {
            let mut ks = vec![p.def.thrown_as];
            for &k in &p.def.thrown_row {
                if k >= 0 && !ks.contains(&k) {
                    ks.push(k);
                }
            }
            most = most.max(ks.len() as i32);
        }
        most
    }
    /// gw_Geno_DefineString: 0 the anim file, 1 the plan file, 2/3/4 a costume's file / joint / matanim.
    pub fn define_string(&self, kind: i32, which: i32, costume: i32) -> Option<&str> {
        let fm = self.fm_for_kind(kind)?;
        let c = costume as usize;
        match which {
            0 => Some(&fm.anim_file),
            1 => Some(&fm.plan_file),
            2 if costume >= 0 && c < fm.cos_file.len() => Some(&fm.cos_file[c]),
            3 if costume >= 0 && c < fm.cos_joint.len() => Some(&fm.cos_joint[c]),
            4 if costume >= 0 && c < fm.cos_matanim.len() => Some(fm.cos_matanim[c].as_str()).filter(|s| !s.is_empty()),
            _ => None,
        }
    }
    pub fn define_art_base_ck(&self, ck: i32) -> i32 {
        self.def_profile_for_kind(ck - 1).map_or(-1, |p| p.def.art_base_ck)
    }
    /// gn_fm_team_costume: the costume a team colour (0 red, 1 blue, 2 green) uses, -1 when not a base "none" define.
    pub fn define_team_costume(&self, ck: i32, team: i32) -> i32 {
        let Some(fm) = self.def_profile_for_kind(ck - 1).and_then(|p| p.def.fm.as_deref()) else { return -1 };
        let n = fm.cos_file.len() as i32;
        if n < 1 || !(0..=2).contains(&team) {
            return -1;
        }
        if let Some(i) = fm.cos_team.iter().position(|&t| t as i32 == team) {
            return i as i32;
        }
        if team < n { team } else { 0 }
    }
    pub fn define_costume_count_ck(&self, ck: i32) -> i32 {
        self.def_profile_for_kind(ck - 1).and_then(|p| p.def.fm.as_deref()).map_or(0, |f| f.cos_file.len() as i32)
    }
    pub fn define_costume_name(&self, ck: i32, costume: i32) -> Option<&str> {
        let fm = self.def_profile_for_kind(ck - 1).and_then(|p| p.def.fm.as_deref())?;
        fm.cos_name.get(costume as usize).filter(|s| costume >= 0 && !s.is_empty()).map(|s| s.as_str())
    }
    /// gn_pres_file: the art file a define declares for `what` and `costume`.
    pub fn define_pres_file(&self, ck: i32, what: i32, costume: i32) -> Option<&str> {
        let pr = self.def_profile_for_kind(ck - 1)?.def.pres.as_deref()?;
        if !(0..GN_PRES_KINDS as i32).contains(&what) {
            return None;
        }
        let mut w = what as usize;
        if w == GN_PRES_RESULTS && pr.file[w].is_empty() {
            w = GN_PRES_PORTRAIT;
        }
        if pr.file[w].is_empty() {
            return None;
        }
        let c = if w == GN_PRES_ICON || w == GN_PRES_EMBLEM || costume < 0 || costume as usize >= pr.file[w].len() { 0 } else { costume as usize };
        Some(&pr.file[w][c])
    }
    /// The define profile behind a CharacterKind (gn_define_profile_ck).
    pub fn define_profile_ck(&self, ck: i32) -> i32 {
        self.definitions.profile_for_kind(ck - 1)
    }

    /// gw_Geno_ControlsSchema(ck): a define's controls, None when ck is not a define or it declares none.
    pub fn controls_schema(&self, ck: i32) -> Option<&super::controls::Schema> {
        let p = self.define_profile_ck(ck);
        self.p.get(p as usize).filter(|_| p >= 0).map(|x| &x.controls).filter(|c| c.any())
    }

    // ---- slice 5: the Lua scalars ----
    pub fn lua_fn(&self, p: i32, s: i32, phase: i32) -> bool {
        let Some(x) = self.p.get(p as usize).filter(|_| p >= 0) else { return false };
        x.def.lua.is_some() && s >= 0 && (s as usize) < MAX_STATES && (0..2).contains(&phase) && x.def.st_lua[s as usize][phase as usize] >= 0
    }
    pub fn lua_slots(&self, p: i32) -> i32 {
        self.p.get(p as usize).filter(|_| p >= 0).and_then(|x| x.def.lua.as_ref()).map_or(0, |g| g.nslot as i32)
    }
    pub fn lua_slot_type(&self, p: i32, i: i32) -> i32 {
        self.p
            .get(p as usize)
            .filter(|_| p >= 0)
            .and_then(|x| x.def.lua.as_ref())
            .filter(|g| i >= 0 && (i as usize) < g.nslot)
            .map_or(-1, |g| g.slot_type[i as usize])
    }
    pub fn lua_slot_name(&self, p: i32, i: i32) -> &str {
        self.p
            .get(p as usize)
            .filter(|_| p >= 0)
            .and_then(|x| x.def.lua.as_ref())
            .filter(|g| i >= 0 && (i as usize) < g.nslot)
            .map_or("", |g| g.slot_name[i as usize].as_str())
    }
    /// gw_Geno_LuaCall: GENO_LUA_NONE when the state has no function for the phase; 0 ok; a GENO_LUA_FAULT_* otherwise.
    /// The fault log line (at most 8 per profile) goes to the registry log.
    pub unsafe fn lua_call(&mut self, p: i32, s: i32, phase: i32, io: *mut u32) -> i32 {
        if !self.lua_fn(p, s, phase) || io.is_null() {
            return GENO_LUA_NONE;
        }
        let x = &mut self.p[p as usize];
        let f = x.def.st_lua[s as usize][phase as usize];
        let g = x.def.lua.as_mut().unwrap();
        let rc = g.call(f, io);
        if rc != 0 && g.faults <= 8 {
            let m = format!(
                "geno: lua fault {rc} in profile {p} state {} ({}): {}{}",
                x.st_name[s as usize],
                if phase == GENO_LUA_PHASE_ENTER as i32 { "enter" } else { "frame" },
                g.err,
                if g.faults == 8 { " (further faults of this profile are not logged)" } else { "" }
            );
            self.log.push(m);
        }
        rc
    }

    /// gw_Geno_DefineOnlineInfo: what two peers must agree on for define number `index` (CK, content id, key, name and
    /// the resource files whose bytes join the identity, in a fixed order).
    pub fn define_online_info(&self, index: i32) -> Option<OnlineInfo> {
        if index < 0 || index as usize >= self.definitions.count() {
            return None;
        }
        let ck = self.definitions.aliases[index as usize];
        let p = self.definitions.profile_for_kind(ck - 1);
        if p < 0 {
            return None;
        }
        let q = &self.p[p as usize];
        let mut files = Vec::new();
        let none = q.def.fm.is_some();
        if let Some(fm) = q.def.fm.as_deref() {
            files.push(fm.anim_file.clone());
            files.push(fm.plan_file.clone());
            for c in &fm.cos_file {
                if files.len() >= 24 {
                    break;
                }
                files.push(c.clone());
            }
        } else {
            files.push("PlMr.dat".into());
            files.push("PlMrAJ.dat".into());
        }
        Some(OnlineInfo { ck, id: q.id, key: q.def.define_key.clone(), name: q.name.clone(), none, files })
    }
}

/// GenoDefineOnline (geno_define_online.h).
pub struct OnlineInfo {
    pub ck: i32,
    pub id: u64,
    pub key: String,
    pub name: String,
    pub none: bool,
    pub files: Vec<String>,
}

// ---- slice 6: a define's own records (geno_define_records.inc) ----

pub const GN_REC_FIELDS: usize = 12;
const GN_REC_MAXFIELD: usize = 11;
pub const GN_REC_NAMES: [&str; GN_REC_FIELDS] = [
    "matches", "wins", "losses", "kos", "falls", "sds", "play_seconds", "damage_dealt", "damage_taken", "attacks_hit",
    "attacks_total", "peak_damage",
];

/// The records table: one row per define key, kept in a file of its own (never the retail save).
#[derive(Default)]
pub struct Records {
    pub recs: Vec<(String, String, [u32; GN_REC_FIELDS])>,
    loaded: bool,
}

impl Records {
    fn find(&mut self, key: &str, create: bool) -> Option<usize> {
        if let Some(i) = self.recs.iter().position(|r| r.0 == key) {
            return Some(i);
        }
        if !create || self.recs.len() >= 64 {
            return None;
        }
        self.recs.push((key.to_string(), String::new(), [0; GN_REC_FIELDS]));
        Some(self.recs.len() - 1)
    }
    /// gn_rec_load_file: a missing file is an empty table; a malformed one is moved aside as <file>.bad.
    pub fn load_file(&mut self, path: &std::path::Path, log: &mut Vec<String>) -> bool {
        self.recs.clear();
        let Ok(text) = std::fs::read(path) else { return true };
        if text.len() > 1 << 20 {
            log.push(format!("geno: records file {} is too large - ignored", path.display()));
            return false;
        }
        let (d, root) = Doc::parse(&text);
        let recs = if root >= 0 { d.get(0, "records") } else { -1 };
        if !d.is(recs, Ty::Obj) {
            log.push(format!("geno: records file {} is not a records file - kept as .bad, starting empty", path.display()));
            let bad = std::path::PathBuf::from(format!("{}.bad", path.display()));
            let _ = std::fs::remove_file(&bad);
            let _ = std::fs::rename(path, &bad);
            self.recs.clear();
            return false;
        }
        for c in d.kids(recs) {
            if !d.is(c, Ty::Obj) {
                continue;
            }
            let key = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
            let Some(i) = self.find(&key, true) else { continue };
            let nm = d.get(c, "name");
            if d.is(nm, Ty::Str) {
                self.recs[i].1 = cap(&d.text(nm), 48);
            }
            for (k, f) in GN_REC_NAMES.iter().enumerate() {
                let v = d.get(c, f);
                if d.is(v, Ty::Num) && (0.0..=4294967295.0).contains(&d.n[v as usize].num) {
                    self.recs[i].2[k] = d.n[v as usize].num as u32;
                }
            }
        }
        true
    }
    pub fn save_file(&self, path: &std::path::Path, log: &mut Vec<String>) -> bool {
        let mut s = String::from("{\"format\": 1, \"records\": {\n");
        for (i, r) in self.recs.iter().enumerate() {
            s += &format!("  \"{}\": {{\"name\": \"{}\"", r.0, r.1);
            for (k, f) in GN_REC_NAMES.iter().enumerate() {
                s += &format!(", \"{f}\": {}", r.2[k]);
            }
            s += &format!("}}{}\n", if i + 1 < self.recs.len() { "," } else { "" });
        }
        s += "}}\n";
        let tmp = std::path::PathBuf::from(format!("{}.tmp", path.display()));
        if std::fs::write(&tmp, s).is_err() {
            log.push(format!("geno: cannot write the records file {}", path.display()));
            return false;
        }
        let _ = std::fs::remove_file(path);
        if std::fs::rename(&tmp, path).is_err() {
            log.push(format!("geno: cannot replace the records file {}", path.display()));
            return false;
        }
        true
    }
    /// The row of define `key` (named `name`), the file read on first use. None: no place to keep records.
    pub fn row(&mut self, path: &std::path::Path, key: &str, name: &str, log: &mut Vec<String>) -> Option<usize> {
        if !self.loaded {
            self.load_file(path, log);
            self.loaded = true;
        }
        let i = self.find(key, true)?;
        self.recs[i].1 = cap(name, 48);
        Some(i)
    }
    /// gn_rec_add: peak_damage is a maximum, the rest saturating sums.
    pub fn add(&mut self, i: usize, field: usize, value: u32) {
        let v = &mut self.recs[i].2[field];
        if field == GN_REC_MAXFIELD {
            *v = (*v).max(value);
        } else {
            *v = v.saturating_add(value);
        }
    }
}
