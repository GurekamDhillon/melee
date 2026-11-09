//! geno_registry.c: Geno's native half. Reads every mounting mod's `geno.json`, gives each fighter entry a STABLE
//! content-derived id, resolves what it attaches to, and answers the game half's questions with scalars.
//!
//! Ported: attach entries with attributes, scale, toon, jumps, hooks, v1 (special_attributes, on_land, motion_anims,
//! subaction overlays), v2 (states, parameter blocks, specials), v5 (counter windows, sounds, articles) and the
//! stable ids / kind table / hot-reload layout check. Native `define` entries (geno_define_*.inc), fighter Lua, the
//! combo graph and the meter are NOT ported yet: a `define` entry is skipped with a log line, and a lua / combo / meter
//! block on an attach entry is refused exactly as the C refuses it.
use super::consts::*;
use super::json::{fmt_g, Doc, Ty};
use super::tables;

pub const GN_NONE: u32 = 0xFFFF_FFFF;
pub const GW_MEX_SLOT_COUNT: usize = 94;
pub const GN_MEX_KIND0: i32 = 0x21;
pub const GN_KINDS: usize = 0x21 + GW_MEX_SLOT_COUNT;

const MAX_ATTRS: usize = GENO_MAX_ATTRS as usize;
const MAX_JUMP_VY: usize = GENO_MAX_JUMP_VY as usize;
const EV_COUNT: usize = GENO_EV_COUNT as usize;
const EV_MAX_HOOKS: usize = GENO_EV_MAX_HOOKS as usize;
const MAX_SPECIAL: usize = GENO_MAX_SPECIAL as usize;
const MAX_ONLAND: usize = GENO_MAX_ONLAND as usize;
const MAX_MANIM: usize = GENO_MAX_MOTION_ANIM as usize;
const MAX_OVERLAYS: usize = GENO_MAX_OVERLAYS as usize;
pub const MAX_STATES: usize = GENO_MAX_STATES as usize;
const CB_SLOTS: usize = GENO_CB_SLOTS as usize;
const PARAMS: usize = GENO_PARAMS as usize;
const SP_COUNT: usize = GENO_SP_COUNT as usize;
const SP_SELECT: usize = GENO_SP_SELECT as usize;
const MAX_SOUNDS: usize = GENO_MAX_SOUNDS as usize;
pub const MAX_ARTICLES: usize = GENO_MAX_ARTICLES as usize;
const AP_COUNT: usize = GENO_AP_COUNT as usize;
const ART_HIT_ENTRIES: usize = GENO_ART_HIT_ENTRIES as usize;
const AH_COUNT: usize = GENO_AH_COUNT as usize;
pub const POOL_WORDS: usize = GENO_POOL_WORDS as usize;
pub const MAX_SLOTS: usize = GENO_MAX_SLOTS as usize;

/// `GENO_TARGET(kind, id)`.
pub fn target(kind: u32, id: u32) -> u32 {
    ((kind & 15) << 28) | (id & 0xFFFF)
}

/// What the registry needs from the host: the dense m-ex slot table and the mods' files.
pub trait Env {
    /// Pl file of the m-ex fighter in dense slot `s` (None when the slot is absent: the scan stops there).
    fn mex_pl_file(&self, _slot: usize) -> Option<String> {
        None
    }
    /// A file of a mounted mod (geno.json, a words file). None when missing.
    fn read_mod_file(&self, _mod_id: &str, _rel: &str) -> Option<Vec<u8>> {
        None
    }
    /// geno 12: a define's `wasm` fighter module, ready to call (slot layout and the `go` target names given). Err is
    /// the refusal text; a host without a wasm runtime refuses every module.
    fn fighter_wasm(&self, _module: &[u8], _slot_names: &[String], _slot_types: &[i32], _targets: &[String]) -> Result<Box<dyn super::lua::FighterCode>, String> {
        Err("this host runs no wasm fighter modules".into())
    }
    /// gw_Fx_BindLoad: the fx binding set for a path in the mod, -1 none.
    fn fx_bind_load(&self, _mod_id: &str, _rel: &str) -> i32 {
        -1
    }
    /// gw_Fx_Find: a Geno effect package by name, -1 none.
    fn fx_find(&self, _name: &str) -> i32 {
        -1
    }
}

/// No m-ex fighters, no files.
pub struct NoEnv;
impl Env for NoEnv {}

pub const VANILLA: [(&str, &str, Option<&str>); 27] = [
    ("Mr", "mario", None),
    ("Fx", "fox", None),
    ("Ca", "captain", Some("falcon")),
    ("Dk", "donkey", Some("dk")),
    ("Kb", "kirby", None),
    ("Kp", "koopa", Some("bowser")),
    ("Lk", "link", None),
    ("Sk", "seak", Some("sheik")),
    ("Ns", "ness", None),
    ("Pe", "peach", None),
    ("Pp", "popo", None),
    ("Nn", "nana", None),
    ("Pk", "pikachu", None),
    ("Ss", "samus", None),
    ("Ys", "yoshi", None),
    ("Pr", "purin", Some("jigglypuff")),
    ("Mt", "mewtwo", None),
    ("Lg", "luigi", None),
    ("Ms", "mars", Some("marth")),
    ("Zd", "zelda", None),
    ("Cl", "clink", Some("younglink")),
    ("Dr", "drmario", None),
    ("Fc", "falco", None),
    ("Pc", "pichu", None),
    ("Gw", "gamewatch", Some("gnw")),
    ("Gn", "ganon", Some("ganondorf")),
    ("Fe", "emblem", Some("roy")),
];

/// snprintf into a char[cap]: at most cap-1 bytes.
pub(super) fn cap(s: &str, n: usize) -> String {
    if s.len() < n {
        return s.to_string();
    }
    let mut e = n - 1;
    while e > 0 && !s.is_char_boundary(e) {
        e -= 1;
    }
    s[..e].to_string()
}

pub(super) fn eq_ic(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn starts_ic(s: &str, p: &str) -> bool {
    s.len() >= p.len() && s.as_bytes()[..p.len()].eq_ignore_ascii_case(p.as_bytes())
}

/// The Pl file a target names ("kirby" -> "PlKb.dat"; "PlSh.dat" stays). "" when unknown.
pub fn target_file(t: &str) -> String {
    if t.len() > 4 && eq_ic(&t[t.len() - 4..], ".dat") {
        return cap(t, 32);
    }
    for (code, name, alias) in VANILLA {
        if eq_ic(t, name) || alias.is_some_and(|a| eq_ic(t, a)) {
            return format!("Pl{code}.dat");
        }
    }
    String::new()
}

pub(super) fn fbits(v: f64) -> u32 {
    (v as f32).to_bits()
}

/// C `strtol(s, &end, 0)` on ASCII: (value, bytes consumed; 0 = no conversion).
fn strtol0(s: &str) -> (i64, usize) {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || (9..=13).contains(&b[i])) {
        i += 1;
    }
    let mut neg = false;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        neg = b[i] == b'-';
        i += 1;
    }
    let (radix, start) = if i + 2 < b.len() + 0 && b[i] == b'0' && (b[i + 1] | 0x20) == b'x' && b[i + 2].is_ascii_hexdigit() {
        (16, i + 2)
    } else if i < b.len() && b[i] == b'0' {
        (8, i)
    } else {
        (10, i)
    };
    let mut j = start;
    let mut v: i64 = 0;
    while j < b.len() && (b[j] as char).to_digit(radix).is_some() {
        v = v.saturating_mul(radix as i64).saturating_add((b[j] as char).to_digit(radix).unwrap() as i64);
        j += 1;
    }
    if j == start {
        if radix == 8 && start < b.len() {
            return (0, start + 1); // a lone "0" digit consumed? (octal loop took none: cannot happen)
        }
        return (0, 0);
    }
    (if neg { -v } else { v }, j)
}

/// C `strtoull(s, &end, 0)`: (value, consumed). Negative values wrap like C.
fn strtoull0(s: &str) -> (u64, usize) {
    let (v, n) = strtol0(s);
    if n == 0 {
        return (0, 0);
    }
    // values past i64 saturate in strtol0; strtoull would give ULLONG_MAX on overflow: close enough for a range check
    (v as u64, n)
}

/// atoi.
fn atoi(s: &str) -> i32 {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || (9..=13).contains(&b[i])) {
        i += 1;
    }
    let mut neg = false;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        neg = b[i] == b'-';
        i += 1;
    }
    let mut v: i32 = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        v = v.wrapping_mul(10).wrapping_add((b[i] - b'0') as i32);
        i += 1;
    }
    if neg {
        v.wrapping_neg()
    } else {
        v
    }
}

// ---- name lookups (the game half's tables, pc/geno/geno_game*.c) ----
pub fn attr_find(name: &str) -> i32 {
    tables::ATTRS.iter().position(|a| a.0 == name).map_or(-1, |i| i as i32)
}
pub fn attr_is_int(i: i32) -> bool {
    i >= 0 && (i as usize) < tables::ATTRS.len() && tables::ATTRS[i as usize].1
}
pub fn hook_find(name: &str) -> i32 {
    tables::HOOKS.iter().find(|h| h.1 == name).map_or(-1, |h| h.0 as i32)
}
pub fn behavior_find(name: &str) -> i32 {
    tables::BEHAVIORS.iter().skip(1).find(|b| b.1 == name).map_or(-1, |b| b.0 as i32)
}
pub fn callback_find(slot: u32, name: &str) -> i32 {
    if name == "like" {
        return GENO_CB_LIKE as i32;
    }
    tables::CALLBACKS.iter().find(|c| c.1 == slot && c.2 == name).map_or(-1, |c| c.0 as i32)
}
pub fn param_find(family: &str, name: &str) -> i32 {
    if let Some(p) = tables::PARAMS.iter().find(|p| p.0 == family && p.1 == name) {
        return p.2 as i32;
    }
    -1
}

// ---- hashing ----
pub(super) fn mix(mut h: u64, v: u64) -> u64 {
    h ^= v;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    h
}

fn hash_bytes(mut h: u64, s: &[u8]) -> u64 {
    h = mix(h, s.len() as u64);
    for &b in s {
        h = mix(h, b as u64);
    }
    h
}

/// The canonical form of a value: type tags, keys in file order, numbers as %.9g, no whitespace.
fn hash_node(d: &Doc, x: i32, mut h: u64) -> u64 {
    let n = &d.n[x as usize];
    h = mix(h, 0x100 + n.ty as u64);
    match n.ty {
        Ty::Bool => h = mix(h, (n.num != 0.0) as u64),
        Ty::Num => h = hash_bytes(h, fmt_g(n.num, 9).as_bytes()),
        Ty::Str => h = hash_bytes(h, &n.str),
        Ty::Arr | Ty::Obj => {
            h = mix(h, n.count as u64);
            for c in d.kids(x) {
                if n.ty == Ty::Obj {
                    h = hash_bytes(h, &d.n[c as usize].key);
                }
                h = hash_node(d, c, h);
            }
        }
        Ty::Null => {}
    }
    h
}

/// Words of one ftcmd command (geno.h geno_cmd_words).
pub fn cmd_words(w0: u32) -> usize {
    const LEN: [u8; 59] = [
        1, 1, 1, 1, 1, 2, 1, 2, 1, 1, 5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7,
        4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 2, 1, 4,
    ];
    let op = w0 >> 26;
    if op < 59 {
        LEN[op as usize] as usize
    } else if op == 59 {
        let l = (w0 >> 16) & 15;
        if l != 0 {
            l as usize
        } else {
            1
        }
    } else {
        1
    }
}

pub struct Profile {
    pub id: u64,
    pub hex: String,
    pub mod_id: String,
    pub target: String,
    pub plfile: String,
    pub name: String,
    pub defined: bool,
    pub kind: i32,
    pub max_jumps: i32,
    pub scale: f32,
    pub fx_set: i32,
    pub njvy: usize,
    pub jvy: [u32; MAX_JUMP_VY],
    pub nattr: usize,
    pub attr_index: [i32; MAX_ATTRS],
    pub attr_bits: [u32; MAX_ATTRS],
    pub nhook: [usize; EV_COUNT],
    pub hook: [[i32; EV_MAX_HOOKS]; EV_COUNT],
    pub hook_arg: [[i32; EV_MAX_HOOKS]; EV_COUNT],
    pub nspec: usize,
    pub spec_index: [i32; MAX_SPECIAL],
    pub spec_bits: [u32; MAX_SPECIAL],
    pub nland: usize,
    pub land_from: [u32; MAX_ONLAND],
    pub land_to: [u32; MAX_ONLAND],
    pub nmanim: usize,
    pub manim_motion: [i32; MAX_MANIM],
    pub manim_row: [i32; MAX_MANIM],
    pub nov: usize,
    pub ov_anim: Vec<i32>,
    pub ov_slot: Vec<i32>,
    pub ov_tag: Vec<i32>,
    pub st_tag: [i32; MAX_STATES],
    pub nstate: usize,
    pub st_name: Vec<String>,
    pub st_bhv: [i32; MAX_STATES],
    pub st_cb: [[i32; CB_SLOTS]; MAX_STATES],
    pub st_anim: [i32; MAX_STATES],
    pub st_anim_from: [u32; MAX_STATES],
    pub st_like: [u32; MAX_STATES],
    pub st_flags_set: [bool; MAX_STATES],
    pub st_flags: [u32; MAX_STATES],
    pub st_move_id: [i32; MAX_STATES],
    pub st_next: [u32; MAX_STATES],
    pub st_land: [u32; MAX_STATES],
    pub st_lag: [u32; MAX_STATES],
    pub st_motion: [u32; MAX_STATES],
    pub st_grav: [u32; MAX_STATES],
    pub nparam: usize,
    pub param_id: [i32; PARAMS],
    pub param_bits: [u32; PARAMS],
    pub special: [u32; SP_COUNT],
    pub st_ctr: [[u32; 4]; MAX_STATES],
    pub st_ctr_on: [bool; MAX_STATES],
    pub nsnd: usize,
    pub snd_name: Vec<String>,
    pub snd_id: [u32; MAX_SOUNDS],
    pub snd_vol: [u32; MAX_SOUNDS],
    pub nart: usize,
    pub art_name: Vec<String>,
    pub art_param: [[u32; AP_COUNT]; MAX_ARTICLES],
    pub art_nhit: [usize; MAX_ARTICLES],
    pub art_hit: Box<[[[u32; AH_COUNT]; ART_HIT_ENTRIES]; MAX_ARTICLES]>,
    pub sp_sel_var: [i32; SP_COUNT],
    pub sp_sel_n: [usize; SP_COUNT],
    pub sp_sel_t: [[u32; SP_SELECT]; SP_COUNT],
    pub art_file: Vec<String>,
    pub art_sym: Vec<String>,
    pub toon_on: bool,
    pub toon: [f32; 16],
    /// The native define half (geno_define_registry.inc and its slices); `define::Def::default()` for an attach entry.
    pub def: super::define::Def,
    /// A define's `controls` (the menus' schema, copied into the frozen match mapping); empty when absent or malformed.
    pub controls: super::controls::Schema,
}

impl Profile {
    fn new() -> Box<Profile> {
        Box::new(Profile {
            id: 0,
            hex: String::new(),
            mod_id: String::new(),
            target: String::new(),
            plfile: String::new(),
            name: String::new(),
            defined: false,
            kind: -1,
            max_jumps: -1,
            scale: 0.0,
            fx_set: -1,
            njvy: 0,
            jvy: [0; MAX_JUMP_VY],
            nattr: 0,
            attr_index: [0; MAX_ATTRS],
            attr_bits: [0; MAX_ATTRS],
            nhook: [0; EV_COUNT],
            hook: [[0; EV_MAX_HOOKS]; EV_COUNT],
            hook_arg: [[0; EV_MAX_HOOKS]; EV_COUNT],
            nspec: 0,
            spec_index: [0; MAX_SPECIAL],
            spec_bits: [0; MAX_SPECIAL],
            nland: 0,
            land_from: [0; MAX_ONLAND],
            land_to: [0; MAX_ONLAND],
            nmanim: 0,
            manim_motion: [0; MAX_MANIM],
            manim_row: [0; MAX_MANIM],
            nov: 0,
            ov_anim: Vec::new(),
            ov_slot: Vec::new(),
            ov_tag: Vec::new(),
            st_tag: [0; MAX_STATES],
            nstate: 0,
            st_name: vec![String::new(); MAX_STATES],
            st_bhv: [0; MAX_STATES],
            st_cb: [[0; CB_SLOTS]; MAX_STATES],
            st_anim: [0; MAX_STATES],
            st_anim_from: [0; MAX_STATES],
            st_like: [0; MAX_STATES],
            st_flags_set: [false; MAX_STATES],
            st_flags: [0; MAX_STATES],
            st_move_id: [0; MAX_STATES],
            st_next: [0; MAX_STATES],
            st_land: [0; MAX_STATES],
            st_lag: [0; MAX_STATES],
            st_motion: [0; MAX_STATES],
            st_grav: [0; MAX_STATES],
            nparam: 0,
            param_id: [0; PARAMS],
            param_bits: [0; PARAMS],
            special: [0; SP_COUNT],
            st_ctr: [[0; 4]; MAX_STATES],
            st_ctr_on: [false; MAX_STATES],
            nsnd: 0,
            snd_name: vec![String::new(); MAX_SOUNDS],
            snd_id: [0; MAX_SOUNDS],
            snd_vol: [0; MAX_SOUNDS],
            nart: 0,
            art_name: vec![String::new(); MAX_ARTICLES],
            art_param: [[0; AP_COUNT]; MAX_ARTICLES],
            art_nhit: [0; MAX_ARTICLES],
            art_hit: Box::new([[[0; AH_COUNT]; ART_HIT_ENTRIES]; MAX_ARTICLES]),
            sp_sel_var: [-1; SP_COUNT],
            sp_sel_n: [0; SP_COUNT],
            sp_sel_t: [[0; SP_SELECT]; SP_COUNT],
            art_file: vec![String::new(); MAX_ARTICLES],
            art_sym: vec![String::new(); MAX_ARTICLES],
            toon_on: false,
            toon: [0.0; 16],
            def: super::define::Def::default(),
            controls: Default::default(),
        })
    }
}

pub struct Registry {
    pub p: Vec<Box<Profile>>,
    /// The define catalogue (GdfCatalog): rows by (mod, key), the profile and resident alias of each.
    pub definitions: super::define::Catalog,
    pub files: i32,
    pub kind_profile: Vec<i32>,
    pub kinds_built: bool,
    pub pool: Vec<u32>,
    pub slot_off: Vec<usize>,
    pub slot_len: Vec<usize>,
    pub slot_rel: Vec<bool>,
    /// What gw_log would have printed, in order.
    pub log: Vec<String>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            p: Vec::new(),
            definitions: Default::default(),
            files: 0,
            kind_profile: vec![-1; GN_KINDS],
            kinds_built: false,
            pool: Vec::new(),
            slot_off: Vec::new(),
            slot_len: Vec::new(),
            slot_rel: Vec::new(),
            log: Vec::new(),
        }
    }
}

// ---- target / word helpers (take the doc) ----

/// A geno.json target: a number (motion id), "motion:N", "special:N", "geno:N" or (v2) "geno:<state name>".
pub(super) fn target_p(d: &Doc, x: i32, p: Option<&Profile>) -> Option<u32> {
    if x < 0 {
        return None;
    }
    let n = &d.n[x as usize];
    if n.ty == Ty::Num {
        if n.num < 0.0 || n.num > 65535.0 {
            return None;
        }
        return Some(target(GENO_TGT_MOTION, n.num as u32));
    }
    if n.ty != Ty::Str {
        return None;
    }
    let s = d.text(x);
    if eq_ic(&s, "auto") {
        return Some(GENO_TGT_AUTO);
    }
    if eq_ic(&s, "helpless") {
        return Some(GENO_TGT_HELPLESS);
    }
    if eq_ic(&s, "stay") {
        return Some(GENO_TGT_STAY);
    }
    let (kind, rest) = if starts_ic(&s, "motion:") {
        (GENO_TGT_MOTION, &s[7..])
    } else if starts_ic(&s, "special:") {
        (GENO_TGT_SPECIAL, &s[8..])
    } else if starts_ic(&s, "geno:") {
        (GENO_TGT_GENO, &s[5..])
    } else {
        return None;
    };
    let (id, used) = strtol0(rest);
    if used == 0 && kind == GENO_TGT_GENO {
        if let Some(p) = p {
            for i in 0..p.nstate {
                if eq_ic(rest, &p.st_name[i]) {
                    return Some(target(GENO_TGT_GENO, i as u32));
                }
            }
        }
        return None;
    }
    if used == 0 || used != rest.len() || !(0..=0xFFFF).contains(&id) {
        return None;
    }
    Some(target(kind, id as u32))
}

pub(super) fn move_tag(d: &Doc, obj: i32) -> i32 {
    const NAMES: [&str; 10] = ["", "jab", "dash_attack", "tilt", "smash", "aerial", "grab", "throw", "special", "projectile"];
    let v = d.get(obj, "move_tag");
    if !d.is(v, Ty::Str) {
        return 0;
    }
    let s = d.text(v);
    (1..10).find(|&i| s == NAMES[i]).unwrap_or(0) as i32
}

/// One script word: a number (0..2^32-1, or a negative int) or a "0x..." / decimal string.
fn word(d: &Doc, x: i32) -> Option<u32> {
    let n = &d.n[x as usize];
    if n.ty == Ty::Num {
        let v = n.num;
        if !(-2147483648.0..=4294967295.0).contains(&v) || v != (v as i64) as f64 {
            return None;
        }
        return Some(v as i64 as u32);
    }
    if n.ty == Ty::Str {
        let s = d.text(x);
        let (v, used) = strtoull0(&s);
        if used == 0 || used != s.len() || v > 0xFFFF_FFFF {
            return None;
        }
        return Some(v as u32);
    }
    None
}

fn num(d: &Doc, obj: i32, key: &str) -> Option<f64> {
    let v = d.get(obj, key);
    if v < 0 || (d.n[v as usize].ty != Ty::Num && d.n[v as usize].ty != Ty::Bool) {
        return None;
    }
    Some(d.n[v as usize].num)
}

fn vec_n(d: &Doc, obj: i32, key: &str, out: &mut [f64]) {
    let v = d.get(obj, key);
    if !d.is(v, Ty::Arr) {
        return;
    }
    for (i, c) in d.kids(v).into_iter().enumerate() {
        if i >= out.len() {
            break;
        }
        if d.n[c as usize].ty == Ty::Num {
            out[i] = d.n[c as usize].num;
        }
    }
}

fn int(d: &Doc, obj: i32, key: &str, def: i32) -> u32 {
    num(d, obj, key).map_or(def as u32, |v| v as i32 as u32)
}

fn jump_max(requested: f64, where_: &str, key: &str, log: &mut Vec<String>) -> i32 {
    let value = if requested < 1.0 {
        1
    } else if requested > 250.0 {
        250
    } else {
        requested as i32
    };
    if requested != value as f64 {
        log.push(format!(
            "geno: {where_}: {key} {} clamped to {value} (integer supported range 1..250)",
            fmt_g(requested, 6)
        ));
    }
    value
}

impl Registry {
    pub fn n(&self) -> usize {
        self.p.len()
    }

    pub(super) fn logf(&mut self, s: String) {
        self.log.push(s);
    }

    fn hook_ref(s: &str) -> (i32, i32) {
        let (name, arg) = match s.find(':') {
            Some(i) => (&s[..i], atoi(&s[i + 1..])),
            None => (s, 0),
        };
        if name.len() >= 64 {
            return (-1, arg);
        }
        (hook_find(name), arg)
    }

    // ---- v1 ----
    fn add_v1(&mut self, p: &mut Profile, d: &Doc, e: i32, env: &dyn Env, mod_id: &str, where_: &str) {
        let x = d.get(e, "special_attributes");
        if d.is(x, Ty::Arr) {
            for c in d.kids(x) {
                let ix = d.get(c, "index");
                let off = d.get(c, "offset");
                let vf = d.get(c, "float");
                let vi = d.get(c, "int");
                let mut idx: i64 = -1;
                if d.is(ix, Ty::Num) {
                    idx = d.n[ix as usize].num as i64;
                } else if off >= 0 {
                    if let Some(w) = word(d, off) {
                        if w & 3 == 0 {
                            idx = (w / 4) as i64;
                        }
                    }
                }
                if idx < 0 || idx >= GENO_SPECIAL_WORDS as i64 || (!d.is(vf, Ty::Num) && !d.is(vi, Ty::Num)) {
                    self.logf(format!(
                        "geno: {where_}: a special_attributes entry needs \"index\" (0-{}) or a 4-aligned \"offset\", and a \"float\" or \"int\" value - ignored",
                        GENO_SPECIAL_WORDS - 1
                    ));
                    continue;
                }
                if p.nspec >= MAX_SPECIAL {
                    break;
                }
                p.spec_index[p.nspec] = idx as i32;
                p.spec_bits[p.nspec] =
                    if d.is(vf, Ty::Num) { fbits(d.n[vf as usize].num) } else { d.n[vi as usize].num as i64 as i32 as u32 };
                p.nspec += 1;
            }
        }
        let x = d.get(e, "on_land");
        if d.is(x, Ty::Arr) {
            for c in d.kids(x) {
                let kf = d.get(c, "keep_frame");
                let from = target_p(d, d.get(c, "from"), Some(p));
                let to = target_p(d, d.get(c, "to"), Some(p));
                let (Some(from), Some(mut to)) = (from, to) else {
                    self.logf(format!("geno: {where_}: an on_land entry needs \"from\" and \"to\" (a motion id, \"special:N\", \"motion:N\" or \"geno:N\") - ignored"));
                    continue;
                };
                if d.is(kf, Ty::Bool) && d.n[kf as usize].num != 0.0 {
                    to |= GENO_TGT_KEEP_FRAME;
                }
                if p.nland >= MAX_ONLAND {
                    break;
                }
                p.land_from[p.nland] = from;
                p.land_to[p.nland] = to;
                p.nland += 1;
                let _ = &mut to;
            }
        }
        let x = d.get(e, "motion_anims");
        if d.is(x, Ty::Arr) {
            for c in d.kids(x) {
                let mo = d.get(c, "motion");
                let sa = d.get(c, "subaction");
                if !d.is(mo, Ty::Num) || !d.is(sa, Ty::Num) {
                    self.logf(format!("geno: {where_}: a motion_anims entry needs \"motion\" and \"subaction\" 0-1023 - ignored"));
                    continue;
                }
                let (m, s) = (d.n[mo as usize].num, d.n[sa as usize].num);
                if m < 0.0 || m > 1023.0 || s < 0.0 || s > 1023.0 {
                    self.logf(format!("geno: {where_}: a motion_anims entry needs \"motion\" and \"subaction\" 0-1023 - ignored"));
                    continue;
                }
                if p.nmanim >= MAX_MANIM {
                    break;
                }
                p.manim_motion[p.nmanim] = m as i32;
                p.manim_row[p.nmanim] = s as i32;
                p.nmanim += 1;
            }
        }
        let x = d.get(e, "subactions");
        if d.is(x, Ty::Arr) {
            for c in d.kids(x) {
                let ix = d.get(c, "index");
                let wl = d.get(c, "words");
                let fl = d.get(c, "file");
                let start = self.pool.len();
                let mut n: usize = 0;
                let mut ok = true;
                if !d.is(ix, Ty::Num) || d.n[ix as usize].num < 0.0 || d.n[ix as usize].num > 1023.0 {
                    self.logf(format!("geno: {where_}: a subactions entry needs \"index\" 0-1023 - ignored"));
                    continue;
                }
                if p.nov >= MAX_OVERLAYS || self.slot_off.len() >= MAX_SLOTS {
                    self.logf(format!("geno: {where_}: too many subaction overlays (limit {MAX_OVERLAYS} per profile, {MAX_SLOTS} slots over all profiles) - the rest are ignored"));
                    break;
                }
                if d.is(wl, Ty::Arr) {
                    for w in d.kids(wl) {
                        match word(d, w) {
                            Some(v) if self.pool.len() < POOL_WORDS => {
                                self.pool.push(v);
                                n += 1;
                            }
                            _ => {
                                ok = false;
                                break;
                            }
                        }
                    }
                } else if d.is(fl, Ty::Str) {
                    match self.words_file(env, mod_id, &d.text(fl), where_) {
                        Some(k) => n = k,
                        None => ok = false,
                    }
                } else {
                    ok = false;
                }
                if !ok || n == 0 {
                    self.logf(format!("geno: {where_}: subaction {}: no usable \"words\" / \"file\" - ignored", d.n[ix as usize].num as i32));
                    self.pool.truncate(start);
                    continue;
                }
                if self.pool.len() >= POOL_WORDS {
                    self.pool.truncate(start);
                    continue;
                }
                self.pool.push(0);
                n += 1;
                if p.defined {
                    // A define's overlay has no absolute pointers: a Subroutine (5) or Goto (7) operand is the word index,
                    // inside this overlay, of its target, and every target must be a command boundary.
                    let mut isb = vec![false; n + 1];
                    let mut q = 0;
                    while q < n {
                        isb[q] = true;
                        q += cmd_words(self.pool[start + q]);
                    }
                    let mut bad = false;
                    q = 0;
                    while q < n {
                        let op = self.pool[start + q] >> 26;
                        if (op == 5 || op == 7)
                            && (q + 1 >= n || self.pool[start + q + 1] as usize >= n || !isb[self.pool[start + q + 1] as usize])
                        {
                            bad = true;
                        }
                        q += cmd_words(self.pool[start + q]);
                    }
                    if bad {
                        self.logf(format!(
                            "geno: {where_}: subaction {}: a Subroutine or Goto operand is not a command inside the overlay - ignored",
                            d.n[ix as usize].num as i32
                        ));
                        self.pool.truncate(start);
                        continue;
                    }
                }
                self.slot_rel.push(p.defined);
                p.ov_anim.push(d.n[ix as usize].num as i32);
                p.ov_tag.push(move_tag(d, c));
                p.ov_slot.push(self.slot_off.len() as i32);
                self.slot_off.push(start);
                self.slot_len.push(n);
                p.nov += 1;
            }
        }
    }

    /// A words file: whitespace/comma separated numbers (0x.. or decimal), '#' comments. Appends to the pool.
    fn words_file(&mut self, env: &dyn Env, mod_id: &str, file: &str, where_: &str) -> Option<usize> {
        if file.contains("..") {
            self.logf(format!("geno: {where_}: script file \"{file}\" refused"));
            return None;
        }
        let Some(bytes) = env.read_mod_file(mod_id, file) else {
            self.logf(format!("geno: {where_}: cannot open script file {mod_id}/{file}"));
            return None;
        };
        let mut n = 0;
        let mut i = 0;
        let b = &bytes;
        while i < b.len() {
            let c = b[i];
            if c == b'#' {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if matches!(c, b' ' | b'\t' | b'\r' | b'\n' | b',') {
                i += 1;
                continue;
            }
            let s = i;
            while i < b.len() && !matches!(b[i], b' ' | b'\t' | b'\r' | b'\n' | b',' | b'#') {
                i += 1;
            }
            let tok = String::from_utf8_lossy(&b[s..i.min(s + 63)]).into_owned();
            let (v, used) = strtoull0(&tok);
            if used == 0 || used != tok.len() || v > 0xFFFF_FFFF || self.pool.len() >= POOL_WORDS {
                self.logf(format!("geno: {where_}: {file}: bad word \"{tok}\" (or the script pool is full)"));
                return None;
            }
            self.pool.push(v as u32);
            n += 1;
        }
        Some(n)
    }

    // ---- v2 ----
    fn special_select(&mut self, p: &mut Profile, d: &Doc, v: i32, k: usize, where_: &str, key: &str) {
        let sel = d.get(v, "select");
        let tl = d.get(v, "targets");
        let s = if d.is(sel, Ty::Str) { d.text(sel) } else { String::new() };
        let bank = if starts_ic(&s, "la_i:") {
            GENO_BANK_LA_INT as i32
        } else if starts_ic(&s, "ra_i:") {
            GENO_BANK_RA_INT as i32
        } else {
            -1
        };
        let idx = if s.len() >= 5 { atoi(&s[5..]) } else { 0 };
        if bank < 0 || idx < 0 || idx >= GENO_VARS_PER_BANK as i32 || !d.is(tl, Ty::Arr) {
            self.logf(format!("geno: {where_}: specials.{key}: a select needs \"select\": \"la_i:N\" / \"ra_i:N\" and a \"targets\" list"));
            return;
        }
        p.sp_sel_var[k] = ((bank & 3) << 6) | (idx & 63);
        p.sp_sel_n[k] = 0;
        for c in d.kids(tl) {
            if p.sp_sel_n[k] >= SP_SELECT {
                break;
            }
            let t = match target_p(d, c, Some(p)) {
                Some(t) => t,
                None => {
                    self.logf(format!("geno: {where_}: specials.{key}: bad target in \"targets\""));
                    GN_NONE
                }
            };
            p.sp_sel_t[k][p.sp_sel_n[k]] = t;
            p.sp_sel_n[k] += 1;
        }
    }

    fn add_v2(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str) {
        const FAMS: [&str; 4] = ["glide", "tornado", "drill", "cape"];
        const CB_KEYS: [&str; 4] = ["anim", "iasa", "phys", "coll"];
        const SP_KEYS: [&str; 8] = ["n", "s", "hi", "lw", "air_n", "air_s", "air_hi", "air_lw"];
        for k in 0..SP_COUNT {
            p.special[k] = GN_NONE;
            p.sp_sel_var[k] = -1;
        }
        let x = d.get(e, "states");
        if d.is(x, Ty::Arr) {
            let kids = d.kids(x);
            for &c in kids.iter().take(MAX_STATES) {
                let nm = d.get(c, "name");
                p.st_name[p.nstate] = if d.is(nm, Ty::Str) { cap(&d.text(nm), 32) } else { String::new() };
                p.nstate += 1;
            }
            if kids.len() > MAX_STATES {
                self.logf(format!("geno: {where_}: more than {MAX_STATES} states - the rest are ignored"));
            }
            for (k, &c) in kids.iter().enumerate().take(p.nstate) {
                let b = d.get(c, "behavior");
                let sa = d.get(c, "subaction");
                p.st_bhv[k] = GENO_BHV_NONE as i32;
                p.st_anim[k] = -1;
                p.st_anim_from[k] = GN_NONE;
                p.st_like[k] = GN_NONE;
                p.st_move_id[k] = -1;
                p.st_tag[k] = move_tag(d, c);
                p.st_next[k] = GN_NONE;
                p.st_land[k] = GN_NONE;
                p.st_cb[k] = [-1; CB_SLOTS];
                let stn = p.st_name[k].clone();
                if d.is(b, Ty::Str) {
                    let id = behavior_find(&d.text(b));
                    if id < 0 {
                        self.logf(format!("geno: {where_}: state {stn}: behavior \"{}\" is unknown", d.text(b)));
                    } else {
                        p.st_bhv[k] = id;
                    }
                }
                if d.is(sa, Ty::Num) && d.n[sa as usize].num >= 0.0 && d.n[sa as usize].num <= 1023.0 {
                    p.st_anim[k] = d.n[sa as usize].num as i32;
                } else if sa >= 0 {
                    match target_p(d, sa, None) {
                        Some(t) => p.st_anim_from[k] = t,
                        None => self.logf(format!("geno: {where_}: state {stn}: \"subaction\" must be an index or \"motion:N\" / \"special:N\" (that motion's animation) - none used")),
                    }
                }
                let v = d.get(c, "like");
                if v >= 0 {
                    match target_p(d, v, None) {
                        Some(t) => p.st_like[k] = t,
                        None => self.logf(format!("geno: {where_}: state {stn}: bad \"like\"")),
                    }
                }
                let v = d.get(c, "flags");
                if v >= 0 {
                    if let Some(w) = word(d, v) {
                        p.st_flags[k] = w;
                        p.st_flags_set[k] = true;
                    }
                }
                let v = d.get(c, "move_id");
                if d.is(v, Ty::Num) {
                    p.st_move_id[k] = (d.n[v as usize].num as i32) & 0xFF;
                }
                let v = d.get(c, "next");
                if v >= 0 {
                    match target_p(d, v, Some(p)) {
                        Some(t) => p.st_next[k] = t,
                        None => self.logf(format!("geno: {where_}: state {stn}: bad \"next\"")),
                    }
                }
                let v = d.get(c, "land");
                if v >= 0 {
                    match target_p(d, v, Some(p)) {
                        Some(t) => p.st_land[k] = t,
                        None => self.logf(format!("geno: {where_}: state {stn}: bad \"land\"")),
                    }
                }
                let v = d.get(c, "landing_lag");
                if d.is(v, Ty::Num) {
                    p.st_lag[k] = fbits(d.n[v as usize].num);
                }
                p.st_motion[k] = GENO_MOTION_LIFTOFF;
                let v = d.get(c, "ledge");
                if v >= 0 {
                    let n = &d.n[v as usize];
                    let mut m: i32 = -1;
                    if n.ty == Ty::Num && n.num >= 0.0 && n.num <= 2.0 {
                        m = n.num as i32;
                    } else if n.ty == Ty::Str {
                        m = match d.text(v).as_str() {
                            "none" => 0,
                            "front" => 1,
                            "both" => 2,
                            _ => -1,
                        };
                    }
                    if m < 0 {
                        self.logf(format!("geno: {where_}: state {stn}: \"ledge\" must be none / front / both"));
                    } else {
                        p.st_motion[k] = (p.st_motion[k] & !GENO_MOTION_LEDGE_MASK) | m as u32;
                    }
                }
                let v = d.get(c, "liftoff");
                if d.is(v, Ty::Bool) || d.is(v, Ty::Num) {
                    if d.n[v as usize].num != 0.0 {
                        p.st_motion[k] |= GENO_MOTION_LIFTOFF;
                    } else {
                        p.st_motion[k] &= !GENO_MOTION_LIFTOFF;
                    }
                }
                let v = d.get(c, "origin");
                if (d.is(v, Ty::Bool) || d.is(v, Ty::Num)) && d.n[v as usize].num != 0.0 {
                    p.st_motion[k] |= GENO_MOTION_ORIGIN;
                }
                let v = d.get(c, "gravity");
                if d.is(v, Ty::Num) {
                    p.st_grav[k] = fbits(d.n[v as usize].num);
                }
                let v = d.get(c, "facing");
                if d.is(v, Ty::Str) && d.text(v) == "entry" {
                    p.st_motion[k] |= GENO_MOTION_ENTRY_FACING;
                }
                for slot in 0..CB_SLOTS {
                    let v = d.get(c, CB_KEYS[slot]);
                    if !d.is(v, Ty::Str) {
                        continue;
                    }
                    p.st_cb[k][slot] = callback_find(slot as u32, &d.text(v));
                    if p.st_cb[k][slot] < 0 {
                        self.logf(format!(
                            "geno: {where_}: state {stn}: {} callback \"{}\" is unknown - the behavior's is used",
                            CB_KEYS[slot],
                            d.text(v)
                        ));
                    }
                }
            }
        }
        for fam in FAMS {
            let x = d.get(e, fam);
            if !d.is(x, Ty::Obj) {
                continue;
            }
            for c in d.kids(x) {
                let key = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                let id = param_find(fam, &key);
                let n = &d.n[c as usize];
                if n.ty != Ty::Bool && n.ty != Ty::Num {
                    continue;
                }
                let v = n.num;
                if id < 0 {
                    self.logf(format!("geno: {where_}: {fam}.{key} is not a parameter Geno knows - ignored"));
                    continue;
                }
                let mut j = 0;
                while j < p.nparam && p.param_id[j] != id {
                    j += 1;
                }
                if j >= PARAMS {
                    break;
                }
                p.param_id[j] = id;
                p.param_bits[j] = fbits(v);
                if j == p.nparam {
                    p.nparam += 1;
                }
            }
        }
        let x = d.get(e, "specials");
        if d.is(x, Ty::Obj) {
            for k in 0..SP_COUNT {
                let v = d.get(x, SP_KEYS[k]);
                if d.is(v, Ty::Obj) {
                    self.special_select(p, d, v, k, where_, SP_KEYS[k]);
                    continue;
                }
                if v >= 0 {
                    match target_p(d, v, Some(p)) {
                        Some(t) => p.special[k] = t,
                        None => {
                            self.logf(format!("geno: {where_}: specials.{}: bad target", SP_KEYS[k]));
                            p.special[k] = GN_NONE;
                        }
                    }
                }
            }
            for k in GENO_SP_AIR_N as usize..=GENO_SP_AIR_LW as usize {
                if p.special[k] == GN_NONE && p.sp_sel_var[k] < 0 {
                    let s = k - GENO_SP_AIR_N as usize;
                    p.special[k] = p.special[s];
                    p.sp_sel_var[k] = p.sp_sel_var[s];
                    p.sp_sel_n[k] = p.sp_sel_n[s];
                    p.sp_sel_t[k] = p.sp_sel_t[s];
                }
            }
        }
    }

    // ---- v5 ----
    fn add_v5(&mut self, p: &mut Profile, d: &Doc, e: i32, env: &dyn Env, where_: &str) {
        const DESPAWN_KEYS: [&str; 4] = ["hit", "shield", "stage", "clank"];
        const HITS_KEYS: [&str; 5] = ["ground", "air", "reflect", "absorb", "counter"];
        let x = d.get(e, "states");
        if d.is(x, Ty::Arr) {
            for (k, c) in d.kids(x).into_iter().enumerate() {
                if k >= p.nstate {
                    break;
                }
                let w = d.get(c, "counter");
                if !d.is(w, Ty::Obj) {
                    continue;
                }
                p.st_ctr_on[k] = true;
                p.st_ctr[k][GENO_CTR_FROM as usize] = int(d, w, "from", 1);
                p.st_ctr[k][GENO_CTR_TO as usize] = int(d, w, "to", 0x7FFF_FFFF);
                p.st_ctr[k][GENO_CTR_NEGATE as usize] = (int(d, w, "negate", 1) != 0) as u32;
                p.st_ctr[k][GENO_CTR_TARGET as usize] = GN_NONE;
                let t = d.get(w, "target");
                if t >= 0 {
                    match target_p(d, t, Some(p)) {
                        Some(v) => p.st_ctr[k][GENO_CTR_TARGET as usize] = v,
                        None => {
                            let stn = p.st_name[k].clone();
                            self.logf(format!("geno: {where_}: state {stn}: bad counter \"target\" - a countered hit only stays flagged"));
                            p.st_ctr[k][GENO_CTR_TARGET as usize] = GN_NONE;
                        }
                    }
                }
            }
        }
        let s = d.get(e, "sounds");
        if d.is(s, Ty::Arr) {
            for sc in d.kids(s) {
                if p.nsnd >= MAX_SOUNDS {
                    break;
                }
                let nm = d.get(sc, "name");
                if !d.is(sc, Ty::Obj) || !d.is(nm, Ty::Str) {
                    continue;
                }
                let Some(sv) = num(d, sc, "retail_sfx") else { continue };
                p.snd_name[p.nsnd] = cap(&d.text(nm), 32);
                p.snd_id[p.nsnd] = sv as i32 as u32;
                p.snd_vol[p.nsnd] = match num(d, sc, "volume") {
                    Some(v) if (0.0..=127.0).contains(&v) => v as i32 as u32,
                    _ => 127,
                };
                p.nsnd += 1;
            }
        }
        let x = d.get(e, "articles");
        if !d.is(x, Ty::Arr) {
            return;
        }
        let kids = d.kids(x);
        for &c in &kids {
            let a = p.nart;
            if a >= MAX_ARTICLES {
                self.logf(format!("geno: {where_}: more than {MAX_ARTICLES} articles - the rest are ignored"));
                break;
            }
            if d.n[c as usize].ty != Ty::Obj {
                continue;
            }
            let m = d.get(c, "name");
            p.art_name[a] = if d.is(m, Ty::Str) { cap(&d.text(m), 32) } else { String::new() };
            p.art_param[a] = [0; AP_COUNT];
            let ap = &mut p.art_param[a];
            ap[GENO_AP_LIFETIME as usize] = fbits(match num(d, c, "lifetime") {
                Some(v) if v > 0.0 => v,
                _ => 60.0,
            });
            let mut vel = [0.0; 2];
            vec_n(d, c, "velocity", &mut vel);
            ap[GENO_AP_VEL_FWD as usize] = fbits(vel[0]);
            ap[GENO_AP_VEL_UP as usize] = fbits(vel[1]);
            let mut sp = [0.0; 2];
            vec_n(d, c, "spawn", &mut sp);
            ap[GENO_AP_SPAWN_FWD as usize] = fbits(sp[0]);
            ap[GENO_AP_SPAWN_UP as usize] = fbits(sp[1]);
            for (key, id) in [("gravity", GENO_AP_GRAVITY), ("max_fall", GENO_AP_MAX_FALL), ("accel", GENO_AP_ACCEL), ("max_speed", GENO_AP_MAX_SPEED)] {
                if let Some(v) = num(d, c, key) {
                    ap[id as usize] = fbits(v);
                }
            }
            ap[GENO_AP_SCALE as usize] = fbits(match num(d, c, "scale") {
                Some(v) if v > 0.0 => v,
                _ => 1.0,
            });
            if let Some(v) = num(d, c, "spin") {
                ap[GENO_AP_SPIN as usize] = fbits(v);
            }
            ap[GENO_AP_MAX_LIVE as usize] = match num(d, c, "max_live") {
                Some(v) if v >= 1.0 => v as i32 as u32,
                _ => 4,
            };
            let m = d.get(c, "homing");
            if d.is(m, Ty::Obj) {
                for (key, id) in [("turn", GENO_AP_HOMING_TURN), ("range", GENO_AP_HOMING_RANGE), ("delay", GENO_AP_HOMING_DELAY)] {
                    if let Some(v) = num(d, m, key) {
                        ap[id as usize] = fbits(v);
                    }
                }
            }
            let mut desp = GENO_ART_DESPAWN_DEFAULT;
            let m = d.get(c, "despawn");
            if d.is(m, Ty::Obj) {
                for (k, key) in DESPAWN_KEYS.iter().enumerate() {
                    if let Some(v) = num(d, m, key) {
                        desp = if v != 0.0 { desp | (1 << k) } else { desp & !(1 << k) };
                    }
                }
            }
            ap[GENO_AP_DESPAWN as usize] = desp;
            let m = d.get(c, "spins");
            if d.is(m, Ty::Arr) {
                let mut n2 = 0;
                for e2 in d.kids(m) {
                    if n2 >= 4 {
                        break;
                    }
                    if d.n[e2 as usize].ty != Ty::Obj {
                        continue;
                    }
                    ap[GENO_AP_SPINS as usize + 2 * n2] = int(d, e2, "joint", 0);
                    ap[GENO_AP_SPINS as usize + 2 * n2 + 1] = fbits(num(d, e2, "z").unwrap_or(0.0));
                    n2 += 1;
                }
            }
            if let Some(v) = num(d, c, "min_speed") {
                ap[GENO_AP_MIN_SPEED as usize] = fbits(v);
            }
            if let Some(v) = num(d, c, "angle") {
                ap[GENO_AP_ANGLE as usize] = fbits(v);
            }
            ap[GENO_AP_BONE as usize] = int(d, c, "bone", -1);
            for (si, key) in ["spawn_sound", "end_sound"].iter().enumerate() {
                let r = d.get(c, key);
                if !d.is(r, Ty::Str) {
                    continue;
                }
                let want = d.text(r);
                let q = (0..p.nsnd).find(|&q| p.snd_name[q] == want);
                let Some(q) = q else {
                    let an = p.art_name[a].clone();
                    self.log.push(format!("geno: {where_}: article {an}: {key} \"{want}\" is not in \"sounds\" - no sound"));
                    continue;
                };
                ap[if si == 1 { GENO_AP_END_SFX } else { GENO_AP_SPAWN_SFX } as usize] = p.snd_id[q];
                ap[if si == 1 { GENO_AP_END_VOL } else { GENO_AP_SPAWN_VOL } as usize] = p.snd_vol[q];
            }
            ap[GENO_AP_EFFECT as usize] = int(d, c, "effect", 0);
            let m = d.get(c, "fx");
            if d.is(m, Ty::Str) {
                let pk = env.fx_find(&d.text(m));
                ap[GENO_AP_FX as usize] = if pk >= 0 { (pk + 1) as u32 } else { 0 };
            }
            ap[GENO_AP_SHOW_MODEL as usize] = matches!(num(d, c, "show_model"), Some(v) if v != 0.0) as u32;
            let m = d.get(c, "effects");
            if d.is(m, Ty::Arr) {
                let mut n2 = 0usize;
                for e2 in d.kids(m) {
                    if n2 >= 8 {
                        break;
                    }
                    let n = &d.n[e2 as usize];
                    if n.ty == Ty::Num {
                        ap[GENO_AP_EFFECTS as usize + n2] = n.num as i32 as u32;
                        n2 += 1;
                    } else if n.ty == Ty::Obj {
                        ap[GENO_AP_EFFECTS as usize + n2] = int(d, e2, "id", 0);
                        ap[GENO_AP_EFFECT_AT as usize + n2] = (int(d, e2, "frame", 0) & 0xFFFF)
                            | ((int(d, e2, "joint", 0) & 0xFF) << 16)
                            | (((int(d, e2, "count", 1).wrapping_sub(1)) & 0xFF) << 24);
                        n2 += 1;
                    }
                }
            }
            let m = d.get(c, "spawns");
            if d.is(m, Ty::Arr) {
                let mut n2 = 0usize;
                for e2 in d.kids(m) {
                    if n2 >= GENO_ART_SPAWNS as usize {
                        break;
                    }
                    if d.n[e2 as usize].ty != Ty::Arr {
                        continue;
                    }
                    let mut o = [0.0; 2];
                    for (k2, q) in d.kids(e2).into_iter().take(2).enumerate() {
                        if d.n[q as usize].ty == Ty::Num {
                            o[k2] = d.n[q as usize].num;
                        }
                    }
                    ap[GENO_AP_SPAWN_V as usize + 2 * n2] = fbits(o[0]);
                    ap[GENO_AP_SPAWN_V as usize + 2 * n2 + 1] = fbits(o[1]);
                    n2 += 1;
                }
                ap[GENO_AP_SPAWN_N as usize] = n2 as u32;
            }
            let m = d.get(c, "model");
            if d.is(m, Ty::Obj) {
                let f = d.get(m, "file");
                let y = d.get(m, "symbol");
                if d.is(f, Ty::Str) {
                    p.art_file[a] = cap(&d.text(f), 64);
                }
                if d.is(y, Ty::Str) {
                    p.art_sym[a] = cap(&d.text(y), 64);
                }
            }
            let hl = d.get(c, "hitboxes");
            let mut hc = 0usize;
            let hits = if d.is(hl, Ty::Arr) { d.kids(hl) } else { Vec::new() };
            for h in hits {
                if hc >= ART_HIT_ENTRIES {
                    break;
                }
                if d.n[h as usize].ty != Ty::Obj {
                    continue;
                }
                let hb = &mut p.art_hit[a][hc];
                hb[GENO_AH_DAMAGE as usize] = fbits(num(d, h, "damage").unwrap_or(1.0));
                hb[GENO_AH_SIZE as usize] = fbits(num(d, h, "size").unwrap_or(3.0));
                let mut off = [0.0; 3];
                vec_n(d, h, "offset", &mut off);
                hb[GENO_AH_OFF_X as usize] = fbits(off[0]);
                hb[GENO_AH_OFF_Y as usize] = fbits(off[1]);
                hb[GENO_AH_OFF_Z as usize] = fbits(off[2]);
                hb[GENO_AH_ANGLE as usize] = int(d, h, "angle", 361);
                hb[GENO_AH_KBG as usize] = int(d, h, "kbg", 100);
                hb[GENO_AH_WKB as usize] = int(d, h, "wkb", 0);
                hb[GENO_AH_BKB as usize] = int(d, h, "bkb", 0);
                hb[GENO_AH_ELEMENT as usize] = int(d, h, "element", 0);
                hb[GENO_AH_SHIELD_DAMAGE as usize] = int(d, h, "shield_damage", 0);
                let stun = int(d, h, "stun", 0) as i32;
                hb[GENO_AH_STUN as usize] = stun.clamp(0, 255) as u32;
                hb[GENO_AH_SFX_SEVERITY as usize] = int(d, h, "sfx_severity", 1);
                hb[GENO_AH_SFX_KIND as usize] = int(d, h, "sfx_kind", 0);
                hb[GENO_AH_START as usize] = int(d, h, "start", 1);
                hb[GENO_AH_END as usize] = int(d, h, "end", 0);
                hb[GENO_AH_FLAGS as usize] = GENO_AHF_DEFAULT;
                hb[GENO_AH_SLOT as usize] = int(d, h, "slot", (hc % GENO_ART_HITBOXES as usize) as i32) & (GENO_ART_HITBOXES - 1);
                let m = d.get(h, "hits");
                if d.is(m, Ty::Obj) {
                    for (k, key) in HITS_KEYS.iter().enumerate() {
                        if let Some(v) = num(d, m, key) {
                            let f = hb[GENO_AH_FLAGS as usize];
                            hb[GENO_AH_FLAGS as usize] = if v != 0.0 { f | (1 << k) } else { f & !(1 << k) };
                        }
                    }
                }
                hc += 1;
            }
            p.art_nhit[a] = hc;
            if hc == 0 {
                let an = p.art_name[a].clone();
                self.log.push(format!("geno: {where_}: article {an} has no hitboxes (it only flies)"));
            }
            p.nart += 1;
        }
        let mut k = 0usize;
        for &c in &kids {
            if k >= p.nart {
                break;
            }
            if d.n[c as usize].ty != Ty::Obj {
                continue;
            }
            for e2 in 0..GENO_ART_CHILDREN as usize {
                p.art_param[k][GENO_AP_CHILD as usize + 5 * e2] = u32::MAX;
            }
            let cl = d.get(c, "children");
            let mut n2 = 0usize;
            let ch = if d.is(cl, Ty::Arr) { d.kids(cl) } else { Vec::new() };
            for e2 in ch {
                if n2 >= GENO_ART_CHILDREN as usize {
                    break;
                }
                let an = d.get(e2, "article");
                let mut idx: i32 = -1;
                if d.is(an, Ty::Num) {
                    idx = d.n[an as usize].num as i32;
                } else if d.is(an, Ty::Str) {
                    let want = d.text(an);
                    for q in 0..p.nart {
                        if eq_ic(&p.art_name[q], &want) {
                            idx = q as i32;
                        }
                    }
                }
                if idx < 0 || idx >= p.nart as i32 {
                    let an = p.art_name[k].clone();
                    self.log.push(format!("geno: {where_}: article {an}: a child names no article - ignored"));
                    continue;
                }
                let b = GENO_AP_CHILD as usize + 5 * n2;
                p.art_param[k][b] = idx as u32;
                p.art_param[k][b + 1] = int(d, e2, "frame", 1);
                p.art_param[k][b + 2] = int(d, e2, "every", 0);
                p.art_param[k][b + 3] = int(d, e2, "count", 1);
                p.art_param[k][b + 4] = int(d, e2, "spawn", 0);
                n2 += 1;
            }
            k += 1;
        }
    }

    // ---- one fighters[] entry ----
    fn add_fighter(&mut self, d: &Doc, e: i32, mod_id: &str, where_: &str, version: i32, env: &dyn Env) {
        const EV_NAMES: [&str; 5] = ["on_init", "on_frame", "on_action", "on_land", "on_hit"];
        let pool_before = self.pool.len();
        let slots_before = self.slot_off.len();
        if d.n[e as usize].ty != Ty::Obj {
            self.logf(format!("geno: {where_}: a fighters[] entry is not an object - skipped"));
            return;
        }
        let is_define = d.get(e, "define") >= 0;
        let x = d.get(e, "attach");
        if !is_define && !d.is(x, Ty::Str) {
            self.logf(format!("geno: {where_}: entry has no attach or define - refused"));
            return;
        }
        if self.p.len() >= GENO_MAX_PROFILES as usize {
            self.logf(format!("geno: {where_}: profile allocation/id space exhausted - refused"));
            return;
        }
        let mut p = Profile::new();
        p.mod_id = cap(mod_id, 64);
        if is_define {
            if !self.define_parse(&mut p, d, e, version, where_) {
                return;
            }
            p.target = cap(&p.def.define_key, 64);
            p.plfile = "PlMr.dat".into();
        } else {
            p.target = cap(&d.text(x), 64);
            p.plfile = target_file(&p.target);
            if p.plfile.is_empty() {
                self.logf(format!("geno: {where_}: unknown attach target - refused"));
                return;
            }
            let nm = d.get(e, "name");
            p.name = if d.is(nm, Ty::Str) { cap(&d.text(nm), 64) } else { p.plfile.clone() };
        }

        // gn_add_fighter: a define's `controls` (docs/geno.md 26); malformed falls back atomically, an attach's is ignored
        let x = d.get(e, "controls");
        if x >= 0 {
            if !p.defined {
                self.logf(format!("geno: {where_}: controls requires a define - ignored"));
            } else if let Some(c) = super::controls::parse(d, x) {
                self.logf(format!("geno: {}: controls {} layouts, {} opt-in extras", p.name, c.layout.len(), c.extra.len()));
                p.controls = c;
            } else {
                self.logf(format!("geno: {where_}: malformed controls - using Both/Default/no extras"));
            }
        }

        let x = d.get(e, "attributes");
        if d.is(x, Ty::Obj) {
            for c in d.kids(x) {
                let key = String::from_utf8_lossy(&d.n[c as usize].key).into_owned();
                let idx = attr_find(&key);
                if idx < 0 || d.n[c as usize].ty != Ty::Num {
                    self.logf(format!(
                        "geno: {where_}: attribute \"{key}\" {} - ignored",
                        if idx < 0 { "is not a common attribute Geno knows" } else { "is not a number" }
                    ));
                    continue;
                }
                if p.nattr >= MAX_ATTRS {
                    break;
                }
                let v = d.n[c as usize].num;
                p.attr_index[p.nattr] = idx;
                p.attr_bits[p.nattr] = if key == "max_jumps" {
                    jump_max(v, where_, "attributes.max_jumps", &mut self.log) as u32
                } else if attr_is_int(idx) {
                    v as i32 as u32
                } else {
                    fbits(v)
                };
                p.nattr += 1;
            }
        }
        p.fx_set = -1;
        let x = d.get(e, "fx_bindings");
        if d.is(x, Ty::Str) {
            p.fx_set = env.fx_bind_load(mod_id, &d.text(x));
        }
        let x = d.get(e, "toon");
        if d.is(x, Ty::Obj) || (d.is(x, Ty::Bool) && d.n[x as usize].num != 0.0) {
            const TK: [(&str, usize, f32, f32, f32); 7] = [
                ("bands", 0, 3.0, 2.0, 8.0),
                ("shadow", 1, 0.58, 0.0, 1.0),
                ("rim", 2, 0.18, 0.0, 1.0),
                ("outline", 3, 0.28, 0.0, 1.0),
                ("outline_dark", 7, 0.75, 0.0, 1.0),
                ("softness", 11, 0.04, 0.0, 0.5),
                ("unlit", 15, 1.0, 0.0, 1.0),
            ];
            p.toon_on = true;
            p.toon[4] = 0.45;
            p.toon[5] = 0.65;
            p.toon[6] = 0.6;
            p.toon[8] = 0.62;
            p.toon[9] = 0.66;
            p.toon[10] = 0.86;
            let is_obj = d.is(x, Ty::Obj);
            for (key, slot, def, lo, hi) in TK {
                let v = if is_obj { d.get(x, key) } else { -1 };
                let mut f = if d.is(v, Ty::Num) {
                    d.n[v as usize].num as f32
                } else if d.is(v, Ty::Bool) {
                    if d.n[v as usize].num != 0.0 { 1.0 } else { 0.0 }
                } else {
                    def
                };
                if !(f >= lo && f <= hi) {
                    self.logf(format!("geno: {where_}: toon.{key} out of range [{}, {}] - default used", fmt_g(lo as f64, 6), fmt_g(hi as f64, 6)));
                    f = def;
                }
                p.toon[slot] = f;
            }
            if is_obj {
                for a in 0..2 {
                    let v = d.get(x, if a == 1 { "shadow_tint" } else { "light" });
                    if !d.is(v, Ty::Arr) || d.n[v as usize].count != 3 {
                        continue;
                    }
                    for (k, c3) in d.kids(v).into_iter().enumerate().take(3) {
                        let n = &d.n[c3 as usize];
                        if n.ty == Ty::Num && n.num >= if a == 1 { 0.0 } else { -1.0 } && n.num <= 1.0 {
                            p.toon[(if a == 1 { 8 } else { 4 }) + k] = n.num as f32;
                        }
                    }
                }
            }
        }
        let x = d.get(e, "scale");
        if d.is(x, Ty::Num) {
            let v = d.n[x as usize].num;
            if (0.5..=2.0).contains(&v) {
                p.scale = v as f32;
            } else {
                self.logf(format!("geno: {where_}: scale {} is outside 0.5..2.0 - ignored", fmt_g(v, 6)));
            }
        }
        let x = d.get(e, "jumps");
        if d.is(x, Ty::Obj) {
            let m = d.get(x, "max");
            let v = d.get(x, "air_vy");
            if d.is(m, Ty::Num) {
                p.max_jumps = jump_max(d.n[m as usize].num, where_, "jumps.max", &mut self.log);
            }
            if d.is(v, Ty::Arr) {
                for c in d.kids(v) {
                    if p.njvy >= MAX_JUMP_VY {
                        break;
                    }
                    if d.n[c as usize].ty == Ty::Num {
                        p.jvy[p.njvy] = fbits(d.n[c as usize].num);
                        p.njvy += 1;
                    }
                }
            }
        }
        let x = d.get(e, "hooks");
        for ev in 0..EV_COUNT {
            if x < 0 {
                break;
            }
            let l = d.get(x, EV_NAMES[ev]);
            if !d.is(l, Ty::Arr) {
                continue;
            }
            for c in d.kids(l) {
                let (id, arg) = if d.n[c as usize].ty == Ty::Str { Self::hook_ref(&d.text(c)) } else { (-1, 0) };
                if id <= 0 {
                    let shown = if d.n[c as usize].ty == Ty::Str { d.text(c) } else { "?".into() };
                    self.logf(format!("geno: {where_}: {} hook \"{shown}\" is unknown - ignored", EV_NAMES[ev]));
                    continue;
                }
                if p.nhook[ev] >= EV_MAX_HOOKS {
                    break;
                }
                p.hook[ev][p.nhook[ev]] = id;
                p.hook_arg[ev][p.nhook[ev]] = arg;
                p.nhook[ev] += 1;
            }
        }
        self.add_v2(&mut p, d, e, where_);
        if !self.define_audio_states(&mut p, d, e, where_) {
            return;
        }
        self.add_v5(&mut p, d, e, env, where_);
        self.add_v1(&mut p, d, e, env, mod_id, where_);
        let undo = |r: &mut Registry| {
            r.pool.truncate(pool_before);
            r.slot_off.truncate(slots_before);
            r.slot_len.truncate(slots_before);
            r.slot_rel.truncate(slots_before);
        };
        // slice 5 / geno 11: the lua, combo and meter blocks (each refuses an attach entry that carries one)
        if !self.lua_parse(&mut p, d, e, env, mod_id, where_, version)
            || !self.combo_parse(&mut p, d, e, where_, version)
            || !self.meter_parse(&mut p, d, e, where_)
        {
            undo(self);
            return;
        }
        if p.defined {
            let overlays = d.get(e, "subactions");
            if overlays >= 0 && (!d.is(overlays, Ty::Arr) || d.n[overlays as usize].count as usize != p.nov) {
                undo(self);
                self.logf(format!("geno: {where_}: native definition refused: invalid overlay or script budget exhausted"));
                return;
            }
        }
        // the id covers the whole entry as written, including keys this version ignores
        let mut id = hash_node(d, e, mix(0x47454E4F00000000, GENO_ID_VERSION as u64));
        for o in 0..p.nov {
            let s = p.ov_slot[o] as usize;
            id = mix(id, 0x4F56_0000_0000_0000 | p.ov_anim[o] as u64);
            for w in 0..self.slot_len[s] {
                id = mix(id, self.pool[self.slot_off[s] + w] as u64);
            }
        }
        id = super::define::fold_lua(id, &p);
        if id == 0 {
            id = 1;
        }
        p.id = id;
        p.hex = format!("{id:016x}");
        self.p.push(p);
    }

    /// Parse one geno.json text into the registry. The number of fighter entries taken, -1 on bad JSON.
    pub fn parse_text(&mut self, text: &[u8], mod_id: &str, env: &dyn Env) -> i32 {
        let where_ = format!("{mod_id}/geno.json");
        let before = self.p.len();
        let (d, root) = Doc::parse(text);
        if root < 0 {
            self.logf(format!("geno: {where_} is not valid JSON ({} near \"{}\") - ignored", d.err, d.near()));
            return -1;
        }
        let v = d.get(root, "geno");
        if !d.is(v, Ty::Num) || (d.n[v as usize].num as i32) < 1 {
            self.logf(format!("geno: {where_} has no \"geno\": <version> - ignored"));
            return -1;
        }
        let version = d.n[v as usize].num as i32;
        if version > super::GENO_FORMAT as i32 {
            self.logf(format!("geno: {where_} is Geno v{version}, this build is v{} - reading what it knows", super::GENO_FORMAT));
        }
        let f = d.get(root, "fighters");
        if d.is(f, Ty::Arr) {
            for e in d.kids(f) {
                self.add_fighter(&d, e, mod_id, &where_, version, env);
            }
        }
        self.files += 1;
        (self.p.len() - before) as i32
    }

    /// This install's FighterKind for a Pl file: retail by code, m-ex by scanning the dense slots. -1 = not installed.
    pub fn kind_for_file(pl: &str, env: &dyn Env) -> i32 {
        for (i, v) in VANILLA.iter().enumerate() {
            if eq_ic(&format!("Pl{}.dat", v.0), pl) {
                return i as i32;
            }
        }
        for s in 0..GW_MEX_SLOT_COUNT {
            let Some(f) = env.mex_pl_file(s) else { break };
            if eq_ic(f.trim_start_matches('/'), pl) {
                return GN_MEX_KIND0 + s as i32;
            }
        }
        -1
    }

    /// FighterKind -> profile, once the m-ex slots exist (and the defines are bound to their aliases). Two entries for
    /// one fighter are a REFUSAL, never "later wins" (docs/specs/geno-v5.md 3): neither applies, one log line per extra
    /// claimant names both mod ids, and the result does not depend on load order.
    pub fn build_kinds(&mut self, env: &dyn Env) {
        // gw_Mex_SlotInternal: the dense slots up to the first absent one
        let mut occupied = vec![false; GW_MEX_SLOT_COUNT];
        for (s, o) in occupied.iter_mut().enumerate() {
            if env.mex_pl_file(s).is_none() {
                break;
            }
            *o = true;
        }
        self.define_bind(&occupied);
        for k in self.kind_profile.iter_mut() {
            *k = -1;
        }
        let mut conflict = vec![false; GN_KINDS];
        let mut first = vec![-1i32; GN_KINDS];
        for i in 0..self.p.len() {
            if !self.p[i].defined {
                self.p[i].kind = Self::kind_for_file(&self.p[i].plfile, env);
            }
            let kind = self.p[i].kind;
            if kind < 0 || kind as usize >= GN_KINDS {
                let p = &self.p[i];
                let m = format!("geno: {}/{} attaches to {}, which this install does not have - inactive", p.mod_id, p.name, p.plfile);
                self.p[i].kind = -1;
                self.log.push(m);
                continue;
            }
            let k = kind as usize;
            if first[k] < 0 {
                first[k] = i as i32;
            } else {
                let m = format!(
                    "geno: REFUSED: mods {} and {} both attach to {} (fighter kind {}); neither entry applies - remove one of them",
                    self.p[first[k] as usize].mod_id, self.p[i].mod_id, self.p[i].plfile, kind
                );
                self.log.push(m);
                conflict[k] = true;
            }
        }
        for i in 0..self.p.len() {
            let kind = self.p[i].kind;
            if kind < 0 {
                continue;
            }
            if conflict[kind as usize] {
                let p = &self.p[i];
                let m = format!("geno: {}/{} refused: another mod attaches to {} too - inactive", p.mod_id, p.name, p.plfile);
                self.p[i].kind = -1;
                self.log.push(m);
                continue;
            }
            self.kind_profile[kind as usize] = i as i32;
            let p = &self.p[i];
            let m = format!(
                "geno: fighter {} ({}) -> kind {}: id {}, {} attribute(s), max jumps {}, {} air vy, hooks {}/{}/{}/{}, {} special attribute(s), {} on_land, {} subaction overlay(s), {} Geno state(s), {} behaviour parameter(s), {} article(s), {} on_hit hook(s)",
                p.name, p.plfile, p.kind, p.hex, p.nattr, p.max_jumps, p.njvy, p.nhook[0], p.nhook[1], p.nhook[2], p.nhook[3], p.nspec,
                p.nland, p.nov, p.nstate, p.nparam, p.nart, p.nhook[GENO_EV_HIT as usize]
            );
            self.log.push(m);
        }
        self.kinds_built = true;
    }

    /// Load every mounted mod's geno.json, in mount order.
    pub fn load_mods(&mut self, mods: &[String], env: &dyn Env) {
        for m in mods {
            let Some(text) = env.read_mod_file(m, "geno.json") else { continue };
            if text.len() > 1 << 20 {
                continue;
            }
            self.parse_text(&text, m, env);
        }
        if self.files > 0 {
            let n = self.p.len();
            self.log.push(format!("geno: {} geno.json file(s), {} fighter entr{}", self.files, n, if n == 1 { "y" } else { "ies" }));
        }
    }

    // ---- the game half's scalar API ----
    pub fn profile_for_kind(&mut self, kind: i32, env: &dyn Env) -> i32 {
        if self.p.is_empty() || kind < 0 || kind as usize >= GN_KINDS {
            return -1;
        }
        if !self.kinds_built {
            self.build_kinds(env);
        }
        self.kind_profile[kind as usize]
    }

    /// The id of the profile attached to the fighter whose main Pl file is `pl`, 0 when none (netplay identity salt).
    pub fn salt_for_pl_file(&self, pl: &str) -> u64 {
        let pl = pl.trim_start_matches('/');
        let mut best = None;
        for (i, p) in self.p.iter().enumerate() {
            if !p.defined && eq_ic(&p.plfile, pl) {
                best = Some(i);
            }
        }
        best.map_or(0, |i| self.p[i].id)
    }

    /// geno hot reload's layout test: would live fighters / items / command cursors change meaning?
    pub fn reload_layout_changed(cur: &Registry, next: &Registry) -> Option<String> {
        if next.p.len() != cur.p.len() {
            return Some(format!("fighter entries {} -> {}", cur.p.len(), next.p.len()));
        }
        for (p, q) in cur.p.iter().zip(next.p.iter()) {
            let why = if p.kind != q.kind || !eq_ic(&p.plfile, &q.plfile) {
                format!("{} now attaches to {}", q.name, q.plfile)
            } else if p.nstate != q.nstate {
                format!("{}: Geno states {} -> {}", q.name, p.nstate, q.nstate)
            } else if p.st_bhv != q.st_bhv || p.st_cb != q.st_cb {
                format!("{}: state behavior/callbacks changed (active fighter callbacks are copied)", q.name)
            } else if p.st_name != q.st_name
                || p.st_anim != q.st_anim
                || p.st_anim_from != q.st_anim_from
                || p.st_like != q.st_like
                || p.st_flags_set != q.st_flags_set
                || p.st_flags != q.st_flags
                || p.st_move_id != q.st_move_id
            {
                format!("{}: state motion rows changed (active fighter row is copied)", q.name)
            } else if p.nart != q.nart || p.art_name[..p.nart] != q.art_name[..p.nart] {
                format!("{}: the article list changed", q.name)
            } else if p.nov != q.nov || p.ov_anim != q.ov_anim || p.ov_slot != q.ov_slot {
                format!("{}: the subaction overlay list changed", q.name)
            } else {
                continue;
            };
            return Some(why);
        }
        if next.slot_off.len() != cur.slot_off.len() || cur.slot_off != next.slot_off || cur.slot_len != next.slot_len {
            return Some("overlay pool layout changed (live command pointers would move)".into());
        }
        None
    }
}
