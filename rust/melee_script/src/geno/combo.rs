//! geno_combo_registry.inc: geno 11, the "combo" block of a define (docs/geno.md section 25) and its HUD "meter".
//! Strict like the rest of a define: an attach entry with the key, a file older than "geno": 11, an unknown name or a
//! value out of range refuses the entry. A route with a list "from" becomes one parsed route per source; parsed routes
//! are stably sorted by priority (highest first). The block is part of the entry's content id.
use super::consts::*;
use super::json::{fmt_g, Doc, Ty};
use super::registry::{eq_ic, target, target_p, Profile, Registry};

const MAX_ROUTES: usize = GENO_COMBO_MAX_ROUTES as usize;
const CR: usize = GENO_CR_COUNT as usize;
const CP: usize = GENO_CP_COUNT as usize;

pub struct Combo {
    pub w: Vec<[i32; CR]>,
    pub name: Vec<String>,
    pub param: [i32; CP],
    pub has_meter: bool,
    pub mname: String,
    pub mrgba: u32,
    pub mx: f32,
    pub my: f32,
    pub mw: f32,
    pub mh: f32,
    pub mmax: i32,
}

/// Melee's common moves by name: the motion ids a "from" covers (first = what a "to" enters).
const MOVES: [(&str, u32, u32); 24] = [
    ("jab1", 44, 1), ("jab2", 45, 1), ("jab3", 46, 1), ("rapid_jab", 47, 3), ("dash_attack", 50, 1),
    ("ftilt", 53, 1), ("ftilt_any", 51, 5), ("utilt", 56, 1), ("dtilt", 57, 1),
    ("fsmash", 60, 1), ("fsmash_any", 58, 5), ("usmash", 63, 1), ("dsmash", 64, 1),
    ("nair", 65, 1), ("fair", 66, 1), ("bair", 67, 1), ("uair", 68, 1), ("dair", 69, 1),
    ("grab", 212, 1), ("dash_grab", 214, 1), ("fthrow", 219, 1), ("bthrow", 220, 1), ("uthrow", 221, 1), ("dthrow", 222, 1),
];

fn pick(s: &str, names: &[&str]) -> i32 {
    names.iter().position(|n| eq_ic(s, n)).map_or(-1, |i| i as i32)
}

/// A "from" item: up to 8 target words, empty when invalid.
fn from_item(d: &Doc, x: i32, p: &Profile) -> Vec<u32> {
    if d.is(x, Ty::Str) {
        let s = d.text(x);
        if s == "*" {
            return vec![target(GENO_TGT_MOTION, GENO_COMBO_FROM_ANY)];
        }
        if let Some(m) = MOVES.iter().find(|m| eq_ic(&s, m.0)) {
            return (0..m.2).map(|k| target(GENO_TGT_MOTION, m.1 + k)).collect();
        }
    }
    match target_p(d, x, Some(p)) {
        Some(t) if (t >> 28) == GENO_TGT_MOTION || (t >> 28) == GENO_TGT_GENO => vec![t],
        _ => Vec::new(),
    }
}

fn to_item(d: &Doc, x: i32, p: &Profile) -> Option<u32> {
    const ACTS: [&str; 15] = [
        "special_n", "special_s", "special_hi", "special_lw", "", "", "", "", "jump", "nair", "fair", "bair", "uair", "dair", "jump_short",
    ];
    if d.is(x, Ty::Str) {
        let s = d.text(x);
        let i = pick(&s, &ACTS);
        if i >= 0 && !ACTS[i as usize].is_empty() {
            return Some(target(GENO_COMBO_TGT_ACT, i as u32));
        }
        if let Some(m) = MOVES.iter().find(|m| eq_ic(&s, m.0)) {
            return Some(target(GENO_TGT_MOTION, m.1));
        }
    }
    target_p(d, x, Some(p)).filter(|&t| t != GENO_TGT_STAY)
}

impl Registry {
    fn combo_int(&mut self, d: &Doc, obj: i32, key: &str, def: i32, lo: i32, hi: i32, where_: &str, route: &str) -> Option<i32> {
        let v = d.get(obj, key);
        if v < 0 {
            return Some(def);
        }
        if d.is(v, Ty::Bool) {
            return Some((d.n[v as usize].num != 0.0) as i32);
        }
        let n = if d.is(v, Ty::Num) { d.n[v as usize].num } else { f64::NAN };
        if !d.is(v, Ty::Num) || n < lo as f64 || n > hi as f64 || n != (n as i32) as f64 {
            let r = if route.is_empty() { String::new() } else { format!(" route {route}") };
            self.logf(format!("geno: {where_}: combo{r}: \"{key}\" must be an integer {lo}..{hi} - refused"));
            return None;
        }
        Some(n as i32)
    }

    /// Ki amounts are written in meter units (0..1000, two decimals) and kept in hundredths.
    fn combo_ki(&mut self, d: &Doc, obj: i32, key: &str, def: i32, where_: &str, route: &str) -> Option<i32> {
        let v = d.get(obj, key);
        if v < 0 {
            return Some(def);
        }
        let n = if d.is(v, Ty::Num) { d.n[v as usize].num } else { f64::NAN };
        if !(n >= 0.0 && n <= 1000.0) {
            let r = if route.is_empty() { String::new() } else { format!(" route {route}") };
            self.logf(format!("geno: {where_}: combo{r}: \"{key}\" must be a number 0..1000 - refused"));
            return None;
        }
        Some((n * 100.0 + 0.5) as i32)
    }

    fn combo_enum(&mut self, d: &Doc, obj: i32, key: &str, def: i32, names: &[&str], where_: &str, route: &str) -> Option<i32> {
        let v = d.get(obj, key);
        if v < 0 {
            return Some(def);
        }
        let i = if d.is(v, Ty::Str) { pick(&d.text(v), names) } else { -1 };
        if i < 0 {
            self.logf(format!("geno: {where_}: combo route {route}: \"{key}\" is not one of the known names - refused"));
            return None;
        }
        Some(i)
    }

    /// gn_combo_parse: true when the entry is fine (with or without a combo block); false refuses it.
    pub(super) fn combo_parse(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str, version: i32) -> bool {
        let cb = d.get(e, "combo");
        if cb < 0 {
            return true;
        }
        if !p.defined {
            self.logf(format!("geno: {where_}: a combo graph is for a define; an attach entry cannot carry \"combo\" - refused"));
            return false;
        }
        if version < 11 {
            self.logf(format!("geno: {where_}: \"combo\" needs \"geno\": 11 - refused"));
            return false;
        }
        if !d.is(cb, Ty::Obj) {
            self.logf(format!("geno: {where_}: \"combo\" must be an object - refused"));
            return false;
        }
        match self.combo_build(p, d, cb, where_) {
            Some(g) => {
                self.logf(format!(
                    "geno: {where_}: combo graph: {} routes, max {} links, ki max {}",
                    g.w.len(),
                    g.param[GENO_CP_MAX_LINKS as usize],
                    g.param[GENO_CP_KI_MAX as usize] / 100
                ));
                p.def.combo = Some(Box::new(g));
                true
            }
            None => false,
        }
    }

    fn combo_build(&mut self, p: &Profile, d: &Doc, cb: i32, where_: &str) -> Option<Combo> {
        const INPUTS: [&str; 7] = ["light", "heavy", "special", "jump", "grab", "shield", "blast"];
        const DIRS: [&str; 7] = ["any", "neutral", "forward", "back", "up", "down", "side"];
        const ONS: [&str; 5] = ["any", "hit", "whiff", "block", "contact"];
        const WHERES: [&str; 3] = ["any", "ground", "air"];
        let mut g = Combo {
            w: Vec::new(),
            name: Vec::new(),
            param: [0; CP],
            has_meter: false,
            mname: String::new(),
            mrgba: 0,
            mx: 0.0,
            my: 0.0,
            mw: 0.0,
            mh: 0.0,
            mmax: 0,
        };
        let buffer = self.combo_int(d, cb, "buffer", 5, 0, 30, where_, "")?;
        let max_links = self.combo_int(d, cb, "max_links", 5, 1, 32, where_, "")?;
        g.param[GENO_CP_MAX_LINKS as usize] = max_links;
        let ki = d.get(cb, "ki");
        if ki >= 0 {
            if !d.is(ki, Ty::Obj) {
                self.logf(format!("geno: {where_}: combo.ki must be an object - refused"));
                return None;
            }
            g.param[GENO_CP_KI_MAX as usize] = self.combo_ki(d, ki, "max", 100, where_, "")?;
            g.param[GENO_CP_KI_START as usize] = self.combo_ki(d, ki, "start", 0, where_, "")?;
            g.param[GENO_CP_KI_ON_HIT as usize] = self.combo_ki(d, ki, "on_hit", 0, where_, "")?;
            g.param[GENO_CP_KI_ON_TAKEN as usize] = self.combo_ki(d, ki, "on_taken", 0, where_, "")?;
            g.param[GENO_CP_KI_ON_BLOCK as usize] = self.combo_ki(d, ki, "on_block", 0, where_, "")?;
            if g.param[GENO_CP_KI_START as usize] > g.param[GENO_CP_KI_MAX as usize] {
                self.logf(format!("geno: {where_}: combo.ki.start is above max - refused"));
                return None;
            }
        }
        let rl = d.get(cb, "routes");
        if !d.is(rl, Ty::Arr) {
            self.logf(format!("geno: {where_}: combo needs a \"routes\" list - refused"));
            return None;
        }
        for (k, c) in d.kids(rl).into_iter().enumerate() {
            let nm = d.get(c, "name");
            if !d.is(c, Ty::Obj) {
                self.logf(format!("geno: {where_}: combo route {k} is not an object - refused"));
                return None;
            }
            let rname = if d.is(nm, Ty::Str) && !d.n[nm as usize].str.is_empty() {
                super::registry::cap(&d.text(nm), GENO_COMBO_MAX_NAME as usize)
            } else {
                format!("route{k}")
            };
            let mut w = [0i32; CR];
            let mut from: Vec<u32> = Vec::new();
            let x = d.get(c, "from");
            if d.is(x, Ty::Arr) {
                for f in d.kids(x) {
                    let got = if from.len() < 56 { from_item(d, f, p) } else { Vec::new() };
                    if got.is_empty() {
                        self.logf(format!("geno: {where_}: combo route {rname}: a \"from\" item is not a move, a state or \"*\" (or there are too many) - refused"));
                        return None;
                    }
                    from.extend(got);
                }
            } else if x >= 0 {
                from = from_item(d, x, p);
            }
            if from.is_empty() {
                self.logf(format!("geno: {where_}: combo route {rname}: \"from\" is missing or names no move or state - refused"));
                return None;
            }
            let x = d.get(c, "to");
            match if x >= 0 { to_item(d, x, p) } else { None } {
                Some(t) => w[GENO_CR_TO as usize] = t as i32,
                None => {
                    self.logf(format!("geno: {where_}: combo route {rname}: \"to\" is missing or is not a target (geno:<state>, a move name, special_n/s/hi/lw, jump, nair..dair, auto) - refused"));
                    return None;
                }
            }
            let v = d.get(c, "input");
            if d.is(v, Ty::Str) && eq_ic(&d.text(v), "none") {
                w[GENO_CR_INPUT as usize] = GENO_CI_NONE as i32;
            } else if v < 0 {
                self.logf(format!("geno: {where_}: combo route {rname}: \"input\" is required (light, heavy, special, jump, grab, shield, none) - refused"));
                return None;
            } else {
                w[GENO_CR_INPUT as usize] = self.combo_enum(d, c, "input", 0, &INPUTS[..GENO_CI_COUNT as usize], where_, &rname)?;
            }
            w[GENO_CR_DIR as usize] = self.combo_enum(d, c, "dir", GENO_CD_ANY as i32, &DIRS, where_, &rname)?;
            w[GENO_CR_ON as usize] = self.combo_enum(d, c, "on", GENO_CO_HIT as i32, &ONS, where_, &rname)?;
            w[GENO_CR_WHERE as usize] = self.combo_enum(d, c, "where", GENO_CW_ANY as i32, &WHERES, where_, &rname)?;
            w[GENO_CR_BUFFER as usize] = self.combo_int(d, c, "buffer", buffer, 0, 30, where_, &rname)?;
            w[GENO_CR_HOLD as usize] = self.combo_int(d, c, "hold", 0, 0, 600, where_, &rname)?;
            w[GENO_CR_PRIORITY as usize] = self.combo_int(d, c, "priority", 0, -1000, 1000, where_, &rname)?;
            w[GENO_CR_KI_COST as usize] = self.combo_ki(d, c, "ki_cost", 0, where_, &rname)?;
            w[GENO_CR_KI_MIN as usize] = self.combo_ki(d, c, "ki_min", 0, where_, &rname)?;
            w[GENO_CR_DEPTH as usize] = self.combo_int(d, c, "depth", max_links, 1, 32, where_, &rname)?;
            w[GENO_CR_ONCE as usize] = self.combo_int(d, c, "once", 0, 0, 1, where_, &rname)?;
            w[GENO_CR_STARTS as usize] = self.combo_int(d, c, "starts", 0, 0, 1, where_, &rname)?;
            let (mut lo, mut hi) = (1, 0);
            let x = d.get(c, "window");
            if x >= 0 {
                let ok = d.is(x, Ty::Arr) && d.n[x as usize].count == 2 && {
                    let k = d.kids(x);
                    let (a, b) = (&d.n[k[0] as usize], &d.n[k[1] as usize]);
                    let good = a.ty == Ty::Num
                        && b.ty == Ty::Num
                        && (1.0..=9999.0).contains(&a.num)
                        && (0.0..=9999.0).contains(&b.num)
                        && !(b.num != 0.0 && b.num < a.num);
                    if good {
                        lo = a.num as i32;
                        hi = b.num as i32;
                    }
                    good
                };
                if !ok {
                    self.logf(format!("geno: {where_}: combo route {rname}: \"window\" must be [first, last] action frames (1-based; last 0 = to the end) - refused"));
                    return None;
                }
            }
            w[GENO_CR_WIN_LO as usize] = lo;
            w[GENO_CR_WIN_HI as usize] = hi;
            w[GENO_CR_SLOT as usize] = -1;
            let x = d.get(c, "when");
            if x >= 0 {
                let sl = if d.is(x, Ty::Obj) { d.get(x, "slot") } else { -1 };
                let mut idx = -1;
                if d.is(sl, Ty::Str) {
                    if let Some(g) = p.def.lua.as_ref() {
                        let s = d.text(sl);
                        for i in 0..g.nslot() {
                            if g.slot_name(i) == s {
                                idx = i as i32;
                            }
                        }
                    }
                }
                if idx < 0 {
                    self.logf(format!("geno: {where_}: combo route {rname}: \"when\".slot must name a slot of lua.state - refused"));
                    return None;
                }
                w[GENO_CR_SLOT_MIN as usize] = self.combo_int(d, x, "min", 1, -1000000, 1000000, where_, &rname)?;
                w[GENO_CR_SLOT as usize] = idx;
            }
            if w[GENO_CR_INPUT as usize] == GENO_CI_NONE as i32 && w[GENO_CR_ON as usize] == GENO_CO_ANY as i32 && hi == 0 {
                self.logf(format!("geno: {where_}: combo route {rname}: an input-less route needs a condition or a closed window - refused"));
                return None;
            }
            for f in from {
                if g.w.len() >= MAX_ROUTES {
                    self.logf(format!("geno: {where_}: combo: more than {MAX_ROUTES} routes (after expanding \"from\" lists) - refused"));
                    return None;
                }
                let mut r = w;
                r[GENO_CR_FROM as usize] = f as i32;
                g.w.push(r);
                g.name.push(rname.clone());
            }
        }
        // stable sort by priority, highest first (the C's insertion sort)
        let mut idx: Vec<usize> = (0..g.w.len()).collect();
        idx.sort_by(|&a, &b| g.w[b][GENO_CR_PRIORITY as usize].cmp(&g.w[a][GENO_CR_PRIORITY as usize]));
        g.w = idx.iter().map(|&i| g.w[i]).collect();
        g.name = idx.iter().map(|&i| g.name[i].clone()).collect();
        g.param[GENO_CP_ROUTES as usize] = g.w.len() as i32;
        Some(g)
    }

    /// gn_meter_parse: "meter", a HUD bar for the combo ki value. true = fine, false = refuse the entry.
    pub(super) fn meter_parse(&mut self, p: &mut Profile, d: &Doc, e: i32, where_: &str) -> bool {
        let m = d.get(e, "meter");
        if m < 0 {
            return true;
        }
        if p.def.combo.is_none() || !p.defined {
            self.logf(format!("geno: {where_}: \"meter\" shows the combo ki value: it needs a define with a combo block - refused"));
            return false;
        }
        if !d.is(m, Ty::Obj) {
            self.logf(format!("geno: {where_}: \"meter\" must be an object - refused"));
            return false;
        }
        let g = p.def.combo.as_mut().unwrap();
        if g.param[GENO_CP_KI_MAX as usize] <= 0 {
            self.log.push(format!("geno: {where_}: \"meter\" needs combo.ki (max above 0) - refused"));
            return false;
        }
        g.has_meter = true;
        g.mname = "KI".into();
        g.mrgba = 0xF2C1_4EFF;
        g.mw = 110.0;
        g.mh = 7.0;
        g.mx = -1.0;
        g.my = 438.0;
        let v = d.get(m, "name");
        if v >= 0 {
            if !d.is(v, Ty::Str) || d.n[v as usize].str.len() > 11 {
                self.log.push(format!("geno: {where_}: meter.name must be a string of at most 11 characters - refused"));
                return false;
            }
            g.mname = d.text(v);
        }
        let v = d.get(m, "color");
        if v >= 0 {
            let c = if d.is(v, Ty::Str) { meter_color(&d.text(v)) } else { None };
            match c {
                Some(c) => g.mrgba = c,
                None => {
                    self.log.push(format!("geno: {where_}: meter.color must be \"#RRGGBB\" - refused"));
                    return false;
                }
            }
        }
        for (key, lo, hi, pos) in [("position", 0.0f32, 640.0f32, true), ("size", 4.0, 400.0, false)] {
            let v = d.get(m, key);
            if v < 0 {
                continue;
            }
            let pair = if d.is(v, Ty::Arr) && d.n[v as usize].count == 2 {
                let k = d.kids(v);
                let (a, b) = (&d.n[k[0] as usize], &d.n[k[1] as usize]);
                (a.ty == Ty::Num && b.ty == Ty::Num && a.num >= lo as f64 && a.num <= hi as f64 && b.num >= lo as f64 && b.num <= hi as f64)
                    .then_some((a.num as f32, b.num as f32))
            } else {
                None
            };
            let Some((a, b)) = pair else {
                self.log.push(format!("geno: {where_}: meter.{key} must be [a, b] numbers in {}..{} - refused", fmt_g(lo as f64, 6), fmt_g(hi as f64, 6)));
                return false;
            };
            if pos {
                g.mx = a;
                g.my = b;
            } else {
                g.mw = a;
                g.mh = b;
            }
        }
        let v = d.get(m, "max");
        if v >= 0 {
            let n = if d.is(v, Ty::Num) { d.n[v as usize].num } else { -1.0 };
            if !d.is(v, Ty::Num) || n <= 0.0 || n > 1000.0 {
                self.log.push(format!("geno: {where_}: meter.max must be a number in 0..1000 - refused"));
                return false;
            }
            g.mmax = (n * 100.0 + 0.5) as i32;
        }
        let shown = (if g.mmax != 0 { g.mmax } else { g.param[GENO_CP_KI_MAX as usize] }) / 100;
        let m = format!("geno: {where_}: meter \"{}\" (ki, max {shown})", g.mname);
        self.log.push(m);
        true
    }

    fn combo_of(&self, p: i32) -> Option<&Combo> {
        if p < 0 {
            return None;
        }
        self.p.get(p as usize).and_then(|x| x.def.combo.as_deref())
    }
    pub fn combo_param(&self, p: i32, what: i32) -> i32 {
        match self.combo_of(p) {
            Some(g) if (0..CP as i32).contains(&what) => g.param[what as usize],
            _ => 0,
        }
    }
    pub fn combo_word(&self, p: i32, route: i32, word: i32) -> i32 {
        match self.combo_of(p) {
            Some(g) if route >= 0 && (route as usize) < g.w.len() && (0..CR as i32).contains(&word) => g.w[route as usize][word as usize],
            _ => 0,
        }
    }
    pub fn combo_name(&self, p: i32, route: i32) -> &str {
        match self.combo_of(p) {
            Some(g) if route >= 0 && (route as usize) < g.w.len() => &g.name[route as usize],
            _ => "",
        }
    }
    /// gw_Geno_MeterParam: 0 has a meter, 1 rgba, 2..5 x y w h in hundredths, 6 max, 7 + i the name's character i.
    pub fn meter_param(&self, p: i32, what: i32) -> i32 {
        let Some(g) = self.combo_of(p).filter(|g| g.has_meter) else { return 0 };
        let h = |v: f32| (v * 100.0 + 0.5) as i32;
        match what {
            0 => 1,
            1 => g.mrgba as i32,
            2 => h(g.mx),
            3 => h(g.my),
            4 => h(g.mw),
            5 => h(g.mh),
            6 => if g.mmax != 0 { g.mmax } else { g.param[GENO_CP_KI_MAX as usize] },
            7..=18 => g.mname.as_bytes().get((what - 7) as usize).copied().unwrap_or(0) as i32,
            _ => 0,
        }
    }
}

fn meter_color(s: &str) -> Option<u32> {
    let b = s.as_bytes();
    if b.len() != 7 || b[0] != b'#' {
        return None;
    }
    let v = u32::from_str_radix(&s[1..], 16).ok()?;
    if !s[1..].bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some((v << 8) | 0xFF)
}
