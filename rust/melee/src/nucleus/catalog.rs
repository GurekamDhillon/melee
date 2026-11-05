//! The Nucleus catalog (port of gw_nucleus_core.h, gw_nucleus_kinds.h and gw_nucleus_catalog.h): mods and their
//! files parsed from the public API's JSON, upserted / removed by id, the JSON-lines cache (the same record shape the C
//! client writes, so one `.nucleus` folder serves both builds), search / filter / sort, and the sync algorithm over an
//! injected fetch: first a full keyset page-through noted at T0, then `updated_since = W - 15 min` and the removed
//! feed, at most one poll per 10 minutes, 429 / 5xx back-off with Retry-After, 700 ms between pages.
use crate::cosmetics_import::Json;

pub const API_BASE: &str = "https://ssbmnucleus.net/api/public/v1";
pub const MEDIA_BASE: &str = "https://media.ssbmnucleus.net/";
pub const SITE: &str = "ssbmnucleus.net";
pub const POLL_SECONDS: i64 = 600;
pub const OVERLAP_SECONDS: i64 = 900;
pub const PAGE_LIMIT: u32 = 100;
pub const MAX_FILES_PER_MOD: usize = 40;
pub const MAX_BODY: usize = 4 << 20;
pub const CACHE_SCHEMA: i64 = 4;
pub const SHOTS: usize = 3;

pub const T_COSTUME: usize = 0;
pub const T_STAGE_SKIN: usize = 1;
pub const T_EFFECTS: usize = 2;
pub const T_CUSTOM_STAGE: usize = 3;
pub const T_GAMEPLAY: usize = 4;
pub const T_PATCH: usize = 5;
pub const T_OTHER: usize = 6;
pub const TYPE_API: [&str; 7] = ["costume", "stage_skin", "effects", "custom_stage", "gameplay_mod", "patch", "other"];
pub const TYPE_LABEL: [&str; 7] = ["Costume", "Stage skin", "Effects", "Custom stage", "Gameplay", "Patch", "Other"];

pub const TAB_COSTUMES: usize = 0;
pub const TAB_STAGES: usize = 1;
pub const TAB_EFFECTS: usize = 2;
pub const TAB_MENUS: usize = 3;
pub const TAB_AUDIO: usize = 4;
pub const TAB_OTHER: usize = 5;
pub const TAB_LABEL: [&str; 6] = ["Costumes", "Stages", "Effects", "Menus", "Audio", "Other"];

pub const FC_OTHER: usize = 0;
pub const FC_COSTUME: usize = 1;
pub const FC_STAGE: usize = 2;
pub const FC_EFFECT: usize = 3;
pub const FC_FIGHTER: usize = 4;
pub const FC_MENU: usize = 5;
pub const FC_AUDIO: usize = 6;
pub const FC_IMAGE: usize = 7;
pub const FC_PATCH: usize = 8;
pub const FC_ARCHIVE: usize = 9;

#[allow(dead_code)]
pub const SORT_UPDATED: usize = 0;
pub const SORT_NEWEST: usize = 1;
pub const SORT_DOWNLOADS: usize = 2;
pub const SORT_LIKES: usize = 3;
pub const SORT_TITLE: usize = 4;
pub const SORT_LABEL: [&str; 5] = ["Recently updated", "Newest", "Most downloaded", "Most liked", "Title"];

pub fn type_of(s: &str) -> usize {
    TYPE_API.iter().position(|t| *t == s).unwrap_or(T_OTHER)
}

/// One row per retail fighter in FighterKind order: (symbol token, PlXx code, label, API names, costume colour order,
/// installable). The colour order is the game's own costume order (src/melee/ft/kinds/ft*), Nr = the default.
pub struct Fighter {
    pub token: &'static str,
    pub pl: &'static str,
    pub label: &'static str,
    pub names: &'static str,
    pub colours: &'static str,
    pub installable: bool,
}
macro_rules! f {
    ($t:expr, $pl:expr, $l:expr, $n:expr, $c:expr, $i:expr) => {
        Fighter { token: $t, pl: $pl, label: $l, names: $n, colours: $c, installable: $i }
    };
}
pub static FIGHTERS: [Fighter; 27] = [
    f!("Mario", "Mr", "Mario", "mario", "Nr,Ye,Bk,Bu,Gr", true),
    f!("Fox", "Fx", "Fox", "fox", "Nr,Or,La,Gr", true),
    f!("Captain", "Ca", "Captain Falcon", "captain falcon", "Nr,Gy,Re,Wh,Gr,Bu", true),
    f!("Donkey", "Dk", "Donkey Kong", "donkey kong", "Nr,Bk,Re,Bu,Gr", true),
    f!("Kirby", "Kb", "Kirby", "kirby", "Nr,Ye,Bu,Re,Gr,Wh", true),
    f!("Koopa", "Kp", "Bowser", "bowser", "Nr,Re,Bu,Bk", true),
    f!("Link", "Lk", "Link", "link", "Nr,Re,Bu,Bk,Wh", true),
    f!("Seak", "Sk", "Sheik", "sheik", "Nr,Re,Bu,Gr,Wh", false),
    f!("Ness", "Ns", "Ness", "ness", "Nr,Ye,Bu,Gr", true),
    f!("Peach", "Pe", "Peach", "peach", "Nr,Ye,Wh,Bu,Gr", true),
    f!("Popo", "Pp", "Ice Climbers", "ice climbers|popo", "Nr,Gr,Or,Re", true),
    f!("Nana", "Nn", "Nana", "nana", "Nr,Ye,Aq,Wh", false),
    f!("Pikachu", "Pk", "Pikachu", "pikachu", "Nr,Re,Bu,Gr", true),
    f!("Samus", "Ss", "Samus", "samus", "Nr,Pi,Bk,Gr,La", true),
    f!("Yoshi", "Ys", "Yoshi", "yoshi", "Nr,Re,Bu,Ye,Pi,Aq", true),
    f!("Purin", "Pr", "Jigglypuff", "jigglypuff", "Nr,Re,Bu,Gr,Ye", true),
    f!("Mewtwo", "Mt", "Mewtwo", "mewtwo", "Nr,Re,Bu,Gr", true),
    f!("Luigi", "Lg", "Luigi", "luigi", "Nr,Wh,Aq,Pi", true),
    f!("Mars", "Ms", "Marth", "marth", "Nr,Re,Gr,Bk,Wh", true),
    f!("Zelda", "Zd", "Zelda", "zelda", "Nr,Re,Bu,Gr,Wh", true),
    f!("Clink", "Cl", "Young Link", "young link", "Nr,Re,Bu,Wh,Bk", true),
    f!("Drmario", "Dr", "Dr. Mario", "dr. mario|dr mario|doctor mario", "Nr,Re,Bu,Gr,Bk", true),
    f!("Falco", "Fc", "Falco", "falco", "Nr,Re,Bu,Gr", true),
    f!("Pichu", "Pc", "Pichu", "pichu", "Nr,Re,Bu,Gr", true),
    f!("Gamewatch", "Gw", "Mr. Game & Watch", "mr. game & watch|game & watch|mr game & watch", "Nr", true),
    f!("Ganon", "Gn", "Ganondorf", "ganondorf|ganon", "Nr,Re,Bu,Gr,La", true),
    f!("Emblem", "Fe", "Roy", "roy", "Nr,Re,Bu,Gr,Ye", true),
];

pub fn fighter_by_token(t: &str) -> Option<usize> {
    FIGHTERS.iter().position(|f| f.token.eq_ignore_ascii_case(t))
}
pub fn fighter_by_pl(pl: &str) -> Option<usize> {
    FIGHTERS.iter().position(|f| f.pl == pl)
}
pub fn fighter_by_name(n: &str) -> Option<usize> {
    if n.is_empty() {
        return None;
    }
    FIGHTERS.iter().position(|f| f.names.split('|').any(|x| x.eq_ignore_ascii_case(n)))
}
/// The costume index a colour code is for a fighter (Nr = 0).
pub fn colour_index(fighter: usize, code: &str) -> Option<usize> {
    if code.is_empty() {
        return Some(0);
    }
    FIGHTERS[fighter].colours.split(',').position(|c| c == code)
}
fn colour_code(name: &str) -> Option<&'static str> {
    const T: [(&str, &str); 14] = [
        ("default", "Nr"), ("red", "Re"), ("blue", "Bu"), ("green", "Gr"), ("yellow", "Ye"), ("white", "Wh"), ("black", "Bk"), ("pink", "Pi"), ("aqua", "Aq"),
        ("lavender", "La"), ("lavendar", "La"), ("orange", "Or"), ("gray", "Gy"), ("grey", "Gy"),
    ];
    T.iter().find(|t| t.0.eq_ignore_ascii_case(name)).map(|t| t.1)
}

/// The UI font is Latin: printable ASCII, one '?' per other code point, control characters collapsed to one space;
/// at most `cap - 1` bytes (the C field sizes).
pub fn ascii(s: &str, cap: usize) -> String {
    let mut o = String::new();
    for c in s.chars() {
        if o.len() + 1 >= cap {
            break;
        }
        if (c as u32) >= 0x80 {
            o.push('?');
        } else if (c as u32) < 0x20 {
            if !o.is_empty() && !o.ends_with(' ') {
                o.push(' ');
            }
        } else {
            o.push(c);
        }
    }
    o
}

pub fn has_ext(f: &str, ext: &str) -> bool {
    f.len() >= ext.len() && f[f.len() - ext.len()..].eq_ignore_ascii_case(ext)
}
pub fn basename(p: &str) -> &str {
    p.rsplit(['/', '\\']).next().unwrap_or(p)
}

// ---------------------------------------------------------------- kinds (gw_nucleus_kinds.h)

/// The versus stages' files, longest first where one starts another (GrPs1..4 are Pokemon Stadium's forms).
pub const STAGE_STEMS: [&str; 33] = [
    "GrPs1", "GrPs2", "GrPs3", "GrPs4", "GrNBa", "GrNLa", "GrBb", "GrCn", "GrCs", "GrFs", "GrFz", "GrGb", "GrGd", "GrGr", "GrI1", "GrI2", "GrIm", "GrIz", "GrKg", "GrKr", "GrMc",
    "GrOk", "GrOp", "GrOt", "GrOy", "GrPs", "GrPu", "GrRc", "GrSh", "GrSt", "GrVe", "GrYt", "GrZe",
];
/// Stages the game asks for by prefix: it reads `<stem>.usd` on an American disc, so a skin goes there as well.
pub const STAGE_USD_STEMS: [&str; 4] = ["GrPs", "GrCn", "GrOt", "GrVe"];
pub const STAGE_NAMES: [(&str, &str); 33] = [
    ("GrNBa", "Battlefield"), ("GrNLa", "Final Destination"), ("GrSt", "Yoshi's Story"), ("GrOp", "Dream Land"), ("GrPs", "Pokemon Stadium"), ("GrIz", "Fountain of Dreams"),
    ("GrCs", "Peach's Castle"), ("GrKg", "Kongo Jungle"), ("GrZe", "Brinstar"), ("GrCn", "Corneria"), ("GrOt", "Onett"), ("GrMc", "Mute City"), ("GrRc", "Rainbow Cruise"),
    ("GrGd", "Jungle Japes"), ("GrGb", "Great Bay"), ("GrSh", "Temple"), ("GrKr", "Brinstar Depths"), ("GrYt", "Yoshi's Island"), ("GrGr", "Green Greens"), ("GrFs", "Fourside"),
    ("GrI1", "Mushroom Kingdom"), ("GrI2", "Mushroom Kingdom II"), ("GrVe", "Venom"), ("GrPu", "Poke Floats"), ("GrBb", "Big Blue"), ("GrIm", "Icicle Mountain"),
    ("GrFz", "Flat Zone"), ("GrOy", "Yoshi's Island (64)"), ("GrOk", "Kongo Jungle (64)"), ("GrPs1", "Pokemon Stadium (fire)"), ("GrPs2", "Pokemon Stadium (grass)"),
    ("GrPs3", "Pokemon Stadium (water)"), ("GrPs4", "Pokemon Stadium (rock)"),
];
pub fn stage_name(stem: &str) -> String {
    STAGE_NAMES.iter().find(|s| s.0 == stem).map_or_else(|| stem.to_string(), |s| s.1.to_string())
}
pub const MENU_FILES: [&str; 4] = ["MnSlChr", "MnSlMap", "IfAll", "GmPause"];

fn alnum(b: Option<&u8>) -> bool {
    b.is_some_and(|c| c.is_ascii_alphanumeric())
}
fn boundary(b: &[u8], p: usize) -> bool {
    p == 0 || !b[p - 1].is_ascii_alphanumeric()
}
fn dat_ext_at(b: &[u8], p: usize) -> bool {
    b.get(p) == Some(&b'.') && b.get(p + 1..p + 4).is_some_and(|e| e.eq_ignore_ascii_case(b"dat") || e.eq_ignore_ascii_case(b"usd")) && !alnum(b.get(p + 4))
}

pub fn stage_stem_in(fname: &str) -> Option<&'static str> {
    let b = fname.as_bytes();
    for p in 0..b.len() {
        if b[p] != b'G' || b.get(p + 1) != Some(&b'r') || !boundary(b, p) {
            continue;
        }
        for s in STAGE_STEMS {
            if b[p..].starts_with(s.as_bytes()) && !alnum(b.get(p + s.len())) {
                return Some(s);
            }
        }
    }
    None
}
fn effect_stem_in(fname: &str) -> Option<String> {
    let b = fname.as_bytes();
    for p in 0..b.len() {
        if b[p] != b'E' || b.get(p + 1) != Some(&b'f') || !b.get(p + 2).is_some_and(|c| c.is_ascii_uppercase()) || !b.get(p + 3).is_some_and(|c| c.is_ascii_lowercase()) || !boundary(b, p) {
            continue;
        }
        let n = if b[p + 4..].starts_with(b"Data") {
            8
        } else if b[p..].starts_with(b"EfKb") && b.get(p + 4).is_some_and(|c| c.is_ascii_uppercase()) && b.get(p + 5).is_some_and(|c| c.is_ascii_lowercase()) {
            6
        } else {
            continue;
        };
        if dat_ext_at(b, p + n) {
            return Some(fname[p..p + n].to_string());
        }
    }
    None
}
fn menu_stem_in(fname: &str) -> Option<(String, bool)> {
    let b = fname.as_bytes();
    for p in 0..b.len() {
        let two = &b[p..(p + 2).min(b.len())];
        if !(two == b"Mn" || two == b"If" || two == b"Gm") || !boundary(b, p) {
            continue;
        }
        let mut n = 2;
        while alnum(b.get(p + n)) && n < 12 {
            n += 1;
        }
        if n < 5 || !dat_ext_at(b, p + n) {
            continue;
        }
        let s = fname[p..p + n].to_string();
        let sup = MENU_FILES.contains(&s.as_str());
        return Some((s, sup));
    }
    None
}

/// (category, target stem, supported) from the API's file type and the file name.
fn file_category(file_type: &str, filename: &str) -> (usize, String, bool) {
    let bn = basename(filename);
    let ext = if has_ext(filename, ".usd") { ".usd" } else { ".dat" };
    if has_ext(filename, ".hps") || has_ext(filename, ".ssm") {
        return (FC_AUDIO, String::new(), true);
    }
    if [".png", ".jpg", ".tpl", ".webp", ".bmp"].iter().any(|e| has_ext(filename, e)) {
        return (FC_IMAGE, String::new(), false);
    }
    if [".xdelta", ".pat", ".ips", ".bps"].iter().any(|e| has_ext(filename, e)) {
        return (FC_PATCH, String::new(), false);
    }
    if [".zip", ".7z", ".rar"].iter().any(|e| has_ext(filename, e)) {
        return (FC_ARCHIVE, String::new(), false);
    }
    if let Some(s) = stage_stem_in(bn).or_else(|| if file_type == "stage_dat" { stage_stem_in(filename) } else { None }) {
        if has_ext(filename, ".dat") || has_ext(filename, ".usd") {
            return (FC_STAGE, format!("{s}{ext}"), true);
        }
        return (FC_STAGE, String::new(), false);
    }
    if file_type == "stage_dat" {
        return (FC_STAGE, String::new(), false);
    }
    if let Some(s) = effect_stem_in(bn) {
        return (FC_EFFECT, format!("{s}{ext}"), true);
    }
    if let Some((s, sup)) = menu_stem_in(bn) {
        return (FC_MENU, format!("{s}{ext}"), sup);
    }
    (FC_OTHER, String::new(), false)
}

pub fn cat_why_not(cat: usize, supported: bool) -> &'static str {
    match cat {
        FC_STAGE => if supported { "" } else { "Not a retail versus stage file name: needs conversion." },
        FC_EFFECT | FC_AUDIO | FC_COSTUME => "",
        FC_MENU => if supported { "" } else { "This menu file is not supported yet (only MnSlChr, MnSlMap, IfAll and GmPause)." },
        FC_FIGHTER => "Ships a whole fighter file (PlXx.dat): it would replace the fighter itself, not just its look.",
        FC_IMAGE => "Image / texture files: not installable yet.",
        FC_PATCH => "Patch file (xdelta / pat): not a mod, it patches the whole disc.",
        FC_ARCHIVE => "Packed archive (zip / 7z / rar): not unpacked yet.",
        _ => "Not a file GD can load as a loose override.",
    }
}

fn tab_of(t: usize, cmask: u32) -> usize {
    match t {
        T_COSTUME => TAB_COSTUMES,
        T_STAGE_SKIN | T_CUSTOM_STAGE => TAB_STAGES,
        T_EFFECTS => TAB_EFFECTS,
        _ if cmask & (1 << FC_STAGE) != 0 => TAB_STAGES,
        _ if cmask & (1 << FC_MENU) != 0 => TAB_MENUS,
        _ if cmask & (1 << FC_AUDIO) != 0 => TAB_AUDIO,
        _ if cmask & (1 << FC_EFFECT) != 0 => TAB_EFFECTS,
        _ => TAB_OTHER,
    }
}

// ---------------------------------------------------------------- records

#[derive(Clone, Debug, Default)]
pub struct NcFile {
    pub id: i64,
    pub filename: String,
    pub character: String,
    pub color: String,
    /// media URLs with MEDIA_BASE stripped
    pub csp: String,
    pub stock: String,
    pub fighter: Option<usize>,
    pub colour_idx: Option<usize>,
    pub costume: bool,
    pub mismatch: bool,
    pub cat: usize,
    pub supported: bool,
    pub ftype: String,
    pub stem: String,
    /// only reachable inside the post's own zip
    pub zip: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NcMod {
    pub id: i64,
    pub typ: usize,
    pub title: String,
    pub author: String,
    pub tags: String,
    pub desc: String,
    pub page: String,
    pub created: String,
    pub updated: String,
    pub stage: String,
    pub tab: usize,
    pub cmask: u32,
    pub shots: Vec<String>,
    pub downloads: i64,
    pub likes: i64,
    pub files: Vec<NcFile>,
    pub nf_total: i64,
    pub fmask: u32,
    /// "a,b": the GD mod folders (mods/<id>/) the post's zip carries
    pub bundle: String,
    pub bexe: bool,
    pub tries: u32,
}

fn slot_from_filename(fn_: &str) -> Option<(String, String)> {
    let b = fn_.as_bytes();
    for p in 0..b.len() {
        if b[p] != b'P' || b.get(p + 1) != Some(&b'l') || (p > 0 && b[p - 1].is_ascii_alphabetic()) {
            continue;
        }
        if !b.get(p + 2).is_some_and(|c| c.is_ascii_uppercase()) || !b.get(p + 3).is_some_and(|c| c.is_ascii_lowercase()) {
            continue;
        }
        let pl = fn_[p + 2..p + 4].to_string();
        let col = if b.get(p + 4).is_some_and(|c| c.is_ascii_uppercase()) && b.get(p + 5).is_some_and(|c| c.is_ascii_lowercase()) && !b.get(p + 6).is_some_and(|c| c.is_ascii_alphabetic()) {
            fn_[p + 4..p + 6].to_string()
        } else {
            String::new()
        };
        return Some((pl, col));
    }
    None
}

fn classify(f: &mut NcFile, file_type: &str) {
    let slot = slot_from_filename(&f.filename);
    let name_f = fighter_by_name(&f.character);
    let slot_f = slot.as_ref().and_then(|s| fighter_by_pl(&s.0));
    let want = colour_code(&f.color);
    let ext_ok = has_ext(&f.filename, ".dat") || has_ext(&f.filename, ".usd");
    f.fighter = name_f.or(slot_f);
    f.mismatch = name_f.is_some() && slot_f.is_some() && name_f != slot_f;
    f.costume = ext_ok && (file_type == "character_dat" || (file_type == "other" && slot.is_some()));
    f.ftype = ascii(file_type, 16);
    let (cat, stem, sup) = file_category(file_type, &f.filename);
    f.cat = cat;
    f.stem = stem;
    f.supported = sup;
    if f.costume {
        f.cat = FC_COSTUME;
        f.supported = true;
    } else if f.cat == FC_OTHER && slot.is_some() && ext_ok {
        f.cat = FC_FIGHTER;
    }
    let col = slot.as_ref().map(|s| s.1.clone()).filter(|c| !c.is_empty());
    f.colour_idx = f.fighter.and_then(|g| colour_index(g, col.as_deref().or(want).unwrap_or("")));
    if let (Some(c), Some(w), Some(g)) = (col.as_deref(), want, f.fighter) {
        if c != w && f.colour_idx.is_some() && colour_index(g, w).is_some() {
            f.mismatch = true;
        }
    }
}

pub fn media_strip(u: &str) -> &str {
    u.strip_prefix(MEDIA_BASE).unwrap_or(u)
}
pub fn media_full(s: &str) -> String {
    if s.is_empty() || s.starts_with("http") { s.to_string() } else { format!("{MEDIA_BASE}{s}") }
}
pub fn download_url(base: &str, m: i64, file: i64) -> String {
    if file > 0 { format!("{base}/mods/{m}/download?file={file}") } else { format!("{base}/mods/{m}/download") }
}

fn gstr<'a>(j: &'a Json, k: &str) -> &'a str {
    j.get(k).and_then(|v| v.str()).unwrap_or("")
}
fn gnum(j: &Json, k: &str, d: i64) -> i64 {
    match j.get(k) {
        Some(Json::Num(n)) => *n as i64,
        _ => d,
    }
}

/// A mod folder name in a GD mod zip: 1..40 of [A-Za-z0-9_-], not starting with '-'.
pub fn bundle_id_ok(s: &str) -> bool {
    !s.is_empty() && s.len() <= 40 && !s.starts_with('-') && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

#[derive(Default)]
pub struct Catalog {
    pub m: Vec<NcMod>,
    pub gone: Vec<i64>,
    index: std::collections::HashMap<i64, usize>,
    pub version: u64,
}

impl Catalog {
    pub fn find(&self, id: i64) -> Option<usize> {
        self.index.get(&id).copied()
    }
    fn reindex(&mut self) {
        self.index = self.m.iter().enumerate().map(|(i, m)| (m.id, i)).collect();
    }

    /// Insert or replace the mod in `node` (`nc_cat_apply`). A later copy of the SAME post (same updated_at) arriving
    /// in the bulk list's thin form keeps what a single-post refetch filled in. True when applied.
    pub fn apply(&mut self, node: &Json) -> bool {
        let id = gnum(node, "id", -1);
        if id <= 0 || !matches!(node, Json::Obj(_)) {
            return false;
        }
        let mut m = NcMod { id, typ: type_of(gstr(node, "type")), ..Default::default() };
        m.title = ascii(gstr(node, "title"), 80);
        m.author = ascii(gstr(node, "author"), 40);
        m.desc = ascii(gstr(node, "description"), 320);
        m.page = gstr(node, "page_url").chars().take(119).collect();
        m.created = gstr(node, "created_at").chars().take(23).collect();
        m.updated = gstr(node, "updated_at").chars().take(23).collect();
        if let Some(Json::Arr(a)) = node.get("screenshots") {
            for t in a.iter().filter_map(|t| t.str()) {
                if m.shots.len() >= SHOTS {
                    break;
                }
                let s = media_strip(t);
                if s.len() < 72 {
                    m.shots.push(s.to_string());
                }
            }
        }
        m.stage = ascii(gstr(node, "stage"), 28);
        m.downloads = gnum(node, "download_count", 0);
        m.likes = gnum(node, "like_count", 0);
        if let Some(Json::Arr(a)) = node.get("tags") {
            let mut t = String::new();
            for s in a.iter().filter_map(|s| s.str()) {
                if t.len() > 150 {
                    continue;
                }
                if !t.is_empty() {
                    t += ", ";
                }
                t += s;
            }
            m.tags = ascii(&t, 96);
        }
        let mut listed = 0;
        if let Some(Json::Arr(a)) = node.get("files") {
            for e in a {
                listed += 1;
                if m.files.len() >= MAX_FILES_PER_MOD || !matches!(e, Json::Obj(_)) {
                    continue;
                }
                let mut f = NcFile { id: gnum(e, "id", 0), filename: ascii(gstr(e, "filename"), 100), character: ascii(gstr(e, "character"), 24), color: ascii(gstr(e, "color"), 16), ..Default::default() };
                let csp = media_strip(gstr(e, "csp_url"));
                if csp.len() < 240 {
                    f.csp = csp.to_string();
                }
                let st = media_strip(gstr(e, "stock_url"));
                if st.len() < 240 {
                    f.stock = st.to_string();
                }
                classify(&mut f, gstr(e, "file_type"));
                // explicit nulls (not absent keys) mean the file is only reachable inside the mod's zip
                let nonstr = |k: &str| e.get(k).is_some_and(|v| v.str().is_none());
                f.zip = (nonstr("download_url") && nonstr("file_url")) || gnum(e, "zip", 0) == 1;
                if f.id <= 0 {
                    continue;
                }
                if f.costume {
                    if let Some(g) = f.fighter {
                        m.fmask |= 1 << g;
                    }
                }
                m.cmask |= 1 << f.cat;
                m.files.push(f);
            }
        }
        m.tab = tab_of(m.typ, m.cmask);
        m.nf_total = gnum(node, "files_total", listed);
        let mut have_af = false;
        if let Some(Json::Arr(af)) = node.get("archive_files") {
            have_af = true;
            let mut ids: Vec<String> = Vec::new();
            for e in af {
                let fname = match e {
                    Json::Obj(_) => gstr(e, "filename"),
                    Json::Str(s) => s.as_str(),
                    _ => "",
                };
                if fname == "melee-pc.exe" {
                    m.bexe = true;
                    continue;
                }
                let Some(rest) = fname.strip_prefix("mods/") else { continue };
                let Some((id, tail)) = rest.split_once('/') else { continue };
                if !bundle_id_ok(id) || tail.is_empty() || ids.iter().any(|x| x == id) || ids.iter().map(|x| x.len() + 1).sum::<usize>() + id.len() + 2 >= 96 {
                    continue;
                }
                ids.push(id.to_string());
            }
            m.bundle = ids.join(",");
        } else {
            m.bundle = gstr(node, "bundle").to_string();
            m.bexe = gnum(node, "bexe", 0) != 0;
        }
        let at = self.find(id);
        if let Some(at) = at {
            let old = &self.m[at];
            if old.updated == m.updated {
                m.tries = old.tries;
                if !have_af && m.bundle.is_empty() && !old.bundle.is_empty() {
                    m.bundle = old.bundle.clone();
                    m.bexe = old.bexe;
                }
                if m.shots.is_empty() && !old.shots.is_empty() {
                    m.shots = old.shots.clone();
                }
                if !have_af && m.files.is_empty() && !old.files.is_empty() {
                    m.files = old.files.clone();
                    m.nf_total = old.nf_total;
                    m.cmask = old.cmask;
                    m.fmask = old.fmask;
                    m.tab = old.tab;
                }
            }
            self.m[at] = m;
        } else {
            self.index.insert(id, self.m.len());
            self.m.push(m);
        }
        self.gone.retain(|&g| g != id);
        self.version += 1;
        true
    }

    pub fn remove_id(&mut self, id: i64) {
        if let Some(at) = self.find(id) {
            self.m.remove(at);
            self.reindex();
        }
        if !self.gone.contains(&id) {
            if self.gone.len() >= 4000 {
                self.gone.remove(0);
            }
            self.gone.push(id);
        }
        self.version += 1;
    }

    // ------------------------------------------------------------ cache (the C client's JSON-lines shape)

    pub fn save_text(&self) -> String {
        let mut b = String::with_capacity(self.m.len() * 600);
        b += "{\"gone\":[";
        b += &self.gone.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(",");
        b += "]}\n";
        for m in &self.m {
            b += &format!("{{\"id\":{},\"type\":{}", m.id, q(TYPE_API[m.typ]));
            b += &format!(",\"title\":{},\"author\":{},\"description\":{},\"stage\":{}", q(&m.title), q(&m.author), q(&m.desc), q(&m.stage));
            b += &format!(",\"tags\":[{}]", if m.tags.is_empty() { String::new() } else { q(&m.tags) });
            b += &format!(",\"page_url\":{},\"screenshots\":[{}]", q(&m.page), m.shots.iter().map(|s| q(&media_full(s))).collect::<Vec<_>>().join(","));
            b += &format!(",\"created_at\":{},\"updated_at\":{}", q(&m.created), q(&m.updated));
            if !m.bundle.is_empty() {
                b += &format!(",\"bundle\":{},\"bexe\":{}", q(&m.bundle), m.bexe as i32);
            }
            b += &format!(",\"download_count\":{},\"like_count\":{},\"files_total\":{},\"files\":[", m.downloads, m.likes, m.nf_total);
            for (i, f) in m.files.iter().enumerate() {
                let ft = if !f.ftype.is_empty() { f.ftype.as_str() } else if f.costume { "character_dat" } else { "other" };
                b += &format!("{}{{\"id\":{},\"file_type\":{},\"filename\":{}", if i > 0 { "," } else { "" }, f.id, q(ft), q(&f.filename));
                b += &format!(",\"character\":{},\"color\":{},\"csp_url\":{},\"stock_url\":{}", q(&f.character), q(&f.color), q(&media_full(&f.csp)), q(&media_full(&f.stock)));
                if f.zip {
                    b += ",\"zip\":1";
                }
                b += "}";
            }
            b += "]}\n";
        }
        b
    }

    pub fn load_text(text: &str) -> Catalog {
        let mut c = Catalog::default();
        for line in text.lines().filter(|l| !l.is_empty()) {
            let Ok(j) = Json::parse(line) else { continue };
            if let Some(Json::Arr(g)) = j.get("gone") {
                c.gone = g.iter().filter_map(|v| if let Json::Num(n) = v { Some(*n as i64) } else { None }).collect();
            } else {
                c.apply(&j);
            }
        }
        c
    }

    // ------------------------------------------------------------ search

    /// Catalog indices in the query's order: every word in the title, author, tags, stage or a file name.
    pub fn filter(&self, q: &Query) -> Vec<usize> {
        let words: Vec<String> = q.text.split(' ').filter(|w| !w.is_empty()).take(8).map(|w| w.to_ascii_lowercase()).collect();
        let mut out: Vec<usize> = (0..self.m.len())
            .filter(|&i| {
                let m = &self.m[i];
                if q.tab.is_some_and(|t| m.tab != t) || q.fighter.is_some_and(|f| m.fmask & (1 << f) == 0) {
                    return false;
                }
                if q.installable_only && !why_not(m).is_empty() {
                    return false;
                }
                let hay = [m.title.to_ascii_lowercase(), m.author.to_ascii_lowercase(), m.tags.to_ascii_lowercase(), m.stage.to_ascii_lowercase()];
                words.iter().all(|w| hay.iter().any(|h| h.contains(w.as_str())) || m.files.iter().any(|f| f.filename.to_ascii_lowercase().contains(w.as_str())))
            })
            .collect();
        let m = &self.m;
        out.sort_by(|&a, &b| {
            let (x, y) = (&m[a], &m[b]);
            let r = match q.sort {
                SORT_NEWEST => y.created.cmp(&x.created),
                SORT_DOWNLOADS => y.downloads.cmp(&x.downloads),
                SORT_LIKES => y.likes.cmp(&x.likes),
                SORT_TITLE => x.title.to_ascii_lowercase().cmp(&y.title.to_ascii_lowercase()),
                _ => y.updated.cmp(&x.updated),
            };
            r.then(x.id.cmp(&y.id)) // a total order: the list never reshuffles between two builds
        });
        out
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub text: String,
    pub tab: Option<usize>,
    pub fighter: Option<usize>,
    pub sort: usize,
    pub installable_only: bool,
}

pub fn q(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            '\n' => o += "\\n",
            '\r' => o += "\\r",
            '\t' => o += "\\t",
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn is_animelee(f: &NcFile) -> bool {
    f.filename.contains("Animelee") || f.filename.contains("animelee") || f.filename.contains("ANIMELEE")
}
/// A post shipping an Animelee set beside a vanilla one installs the vanilla files only.
pub fn variant_skip(files: &[NcFile], i: usize) -> bool {
    files[i].costume && is_animelee(&files[i]) && files.iter().enumerate().any(|(k, f)| k != i && f.costume && f.fighter == files[i].fighter && !is_animelee(f))
}

pub fn http_reason(status: u32) -> &'static str {
    match status {
        404 | 410 => "Nucleus offers no download for this file (the author may have removed it).",
        401 | 403 => "Nucleus does not allow this download.",
        429 => "Nucleus asked us to slow down. Try again in a minute.",
        s if s >= 500 => "Nucleus is not answering right now. Try again later.",
        0 => "Could not reach Nucleus.",
        _ => "The download did not work.",
    }
}

/// Why a mod cannot be installed here, or "" when it can (`nc_why_not_files`). Custom stages and MexManager fighter
/// zips need the m-ex runtime, which this build does not run yet (parity.md): they say so instead of installing.
pub fn why_not(m: &NcMod) -> &'static str {
    if m.typ == T_COSTUME {
        let costumes: Vec<&NcFile> = m.files.iter().filter(|f| f.costume).collect();
        if costumes.is_empty() {
            return "No costume file in this post.";
        }
        if !costumes.iter().any(|f| f.fighter.is_some()) {
            return "Not a retail fighter's costume.";
        }
        if costumes.iter().any(|f| f.fighter.is_some_and(|g| FIGHTERS[g].installable)) {
            return "";
        }
        return "Sheik and Nana costumes come with Zelda and Popo (not yet).";
    }
    if m.typ == T_CUSTOM_STAGE {
        return "Custom stage: needs the m-ex stage runtime, which the Rust build does not run yet.";
    }
    if m.typ == T_GAMEPLAY && m.bundle.is_empty() && m.files.iter().any(|f| has_ext(&f.filename, ".zip")) {
        return "Fighter / gameplay zip: needs the m-ex runtime, which the Rust build does not run yet.";
    }
    if m.typ == T_GAMEPLAY {
        let nz = m.files.iter().filter(|f| has_ext(&f.filename, ".zip") || has_ext(&f.filename, ".dat")).count();
        let ptr = m
            .files
            .iter()
            .filter(|f| (has_ext(&f.filename, ".txt") || has_ext(&f.filename, ".url")) && ["oogle", "rive", "ega", "ropbox", "ediafire", "ink", "ownload"].iter().any(|w| f.filename.contains(w)))
            .count();
        if nz == 0 && ptr > 0 {
            return "Download is an external link. GD does not follow external links.";
        }
    }
    if m.typ == T_PATCH {
        return "xdelta patch: not a mod (it patches the whole disc).";
    }
    if !m.bundle.is_empty() {
        return "";
    }
    if m.files.is_empty() {
        return "Nucleus lists no files for this post yet.";
    }
    let mut first = "";
    for f in &m.files {
        let why = cat_why_not(f.cat, f.supported);
        if why.is_empty() {
            return "";
        }
        if first.is_empty() || f.cat == FC_FIGHTER {
            first = why;
        }
    }
    if m.typ == T_GAMEPLAY && !first.is_empty() {
        return "Gameplay mod: needs conversion (code or game data, not a loose file).";
    }
    first
}

// ---------------------------------------------------------------- time

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
pub fn iso(t: i64) -> String {
    let (mut z, mut rem) = (t / 86400, t % 86400);
    if rem < 0 {
        rem += 86400;
        z -= 1;
    }
    z += 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}
#[allow(dead_code)]
pub fn parse_iso(s: &str) -> Option<i64> {
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    Some(days_from_civil(n(0, 4)?, n(5, 7)?, n(8, 10)?) * 86400 + n(11, 13)? * 3600 + n(14, 16)? * 60 + n(17, 19)?)
}
pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

pub fn urlenc(s: &str) -> String {
    let mut o = String::new();
    for c in s.bytes() {
        if c.is_ascii_alphanumeric() || b"-_.~".contains(&c) {
            o.push(c as char);
        } else {
            o += &format!("%{c:02X}");
        }
    }
    o
}

// ---------------------------------------------------------------- sync

#[derive(Clone, Debug, Default)]
pub struct SyncState {
    pub t0: i64,
    pub since: i64,
    pub last_poll: i64,
    pub next_ok: i64,
    pub full: bool,
    pub cursor: String,
    pub pages: u32,
    pub total: i64,
    pub stale: bool,
    pub msg: String,
}

impl SyncState {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema\":{CACHE_SCHEMA},\"t0\":{},\"since\":{},\"last_poll\":{},\"next_ok\":{},\"full\":{},\"cursor\":{}}}\n",
            self.t0, self.since, self.last_poll, self.next_ok, self.full as i32, q(&self.cursor)
        )
    }
    pub fn from_json(t: &str) -> SyncState {
        let Ok(j) = Json::parse(t.trim()) else { return SyncState { stale: true, ..Default::default() } };
        if gnum(&j, "schema", 1) < CACHE_SCHEMA {
            return SyncState { stale: true, ..Default::default() };
        }
        SyncState {
            t0: gnum(&j, "t0", 0),
            since: gnum(&j, "since", 0),
            last_poll: gnum(&j, "last_poll", 0),
            next_ok: gnum(&j, "next_ok", 0),
            full: gnum(&j, "full", 0) != 0,
            cursor: gstr(&j, "cursor").to_string(),
            ..Default::default()
        }
    }
}

/// What the sync needs from the engine: the transport, the clock, a sleep that ends early on stop, and the catalog
/// under its lock.
pub trait SyncIo {
    fn fetch(&self, url: &str) -> crate::nucleus::http::Resp;
    fn now(&self) -> i64;
    fn sleep_ms(&self, ms: u64);
    fn stop(&self) -> bool;
    fn progress(&self, what: &str, done: usize, total: i64);
    fn checkpoint(&self, s: &SyncState);
    fn with_cat<R>(&self, f: impl FnOnce(&mut Catalog) -> R) -> R;
}

/// GET with the API's back-off: 429 / 5xx / no answer wait (Retry-After when given, else 5, 15, 45, 135 s capped at
/// 120) and retry up to 5 times; another 4xx stops. Ok(body) on a 200.
fn get_backoff(io: &impl SyncIo, url: &str) -> Result<Vec<u8>, (String, u32)> {
    let mut wait = 5u64;
    let mut last = 0;
    for attempt in 0..5 {
        if io.stop() {
            return Err(("Stopped.".into(), 0));
        }
        let r = io.fetch(url);
        last = r.status;
        if r.status == 200 {
            return Ok(r.body);
        }
        if (400..500).contains(&r.status) && r.status != 429 {
            return Err((format!("SSBM Nucleus refused the request ({}).", r.status), r.status));
        }
        if attempt == 4 {
            break;
        }
        let ra = if r.retry_after > 0 { r.retry_after as u64 } else { wait }.min(120);
        io.sleep_ms(ra * 1000);
        wait *= 3;
    }
    Err(("SSBM Nucleus is not answering. Showing what is saved.".into(), last))
}

/// One page of /mods (or /mods/removed): (applied, next cursor, total).
fn apply_page(io: &impl SyncIo, body: &[u8], removed: bool) -> Option<(usize, String, Option<i64>)> {
    let j = Json::parse(std::str::from_utf8(body).ok()?).ok()?;
    let Some(Json::Arr(data)) = j.get("data") else { return None };
    let n = io.with_cat(|c| {
        let mut n = 0;
        for e in data {
            if removed {
                let id = gnum(e, "id", -1);
                if id > 0 {
                    c.remove_id(id);
                    n += 1;
                }
            } else if c.apply(e) {
                n += 1;
            }
        }
        n
    });
    let next = j.get("next_cursor").and_then(|v| v.str()).unwrap_or("").to_string();
    let total = j.get("total").map(|_| gnum(&j, "total", 0));
    Some((n, next, total))
}

/// One mod's record (the body of GET /mods/{id}) merged in.
pub fn apply_one(io: &impl SyncIo, body: &[u8]) -> bool {
    let Some(j) = std::str::from_utf8(body).ok().and_then(|t| Json::parse(t).ok()) else { return false };
    match j.get("data") {
        Some(d) => io.with_cat(|c| c.apply(d)),
        None => false,
    }
}

/// Recent posts the bulk list captured before Nucleus processed their archive (no files): asked for one by one, at
/// most 6 per run, each at most 3 times a session, only posts younger than 30 days.
fn recheck_empty(io: &impl SyncIo, base: &str) {
    let cutoff = iso(io.now() - 30 * 86400);
    let ids: Vec<i64> = io.with_cat(|c| c.m.iter().rev().filter(|m| m.files.is_empty() && m.nf_total == 0 && m.bundle.is_empty() && m.tries < 3 && m.created > cutoff).take(6).map(|m| m.id).collect());
    for id in ids {
        if io.stop() {
            break;
        }
        io.sleep_ms(700);
        match get_backoff(io, &format!("{base}/mods/{id}")) {
            Ok(b) => {
                apply_one(io, &b);
            }
            Err((_, st)) if !(400..500).contains(&st) => break,
            Err(_) => {}
        }
        io.with_cat(|c| {
            if let Some(at) = c.find(id) {
                if c.m[at].files.is_empty() && c.m[at].bundle.is_empty() {
                    c.m[at].tries += 1;
                }
            }
        });
    }
}

/// Run one sync (`nc_sync_run`). Ok(true) finished, Ok(false) too soon (nothing fetched), Err when it gave up (s.msg
/// says why; the catalog keeps what arrived). `force` (a press of Refresh) ignores the 10-minute rule but never
/// polls twice within a minute and never skips the back-off after a failure.
pub fn sync_run(s: &mut SyncState, io: &impl SyncIo, base: &str, force: bool) -> Result<bool, ()> {
    let now = io.now();
    s.msg.clear();
    if now < s.next_ok {
        s.msg = "Waiting before the next try.".into();
        return Ok(false);
    }
    if s.full && s.last_poll != 0 && ((!force && now - s.last_poll < POLL_SECONDS) || (force && now - s.last_poll < 60)) {
        return Ok(false);
    }
    let start = now;
    macro_rules! fail {
        ($m:expr) => {{
            s.msg = $m;
            s.next_ok = io.now() + POLL_SECONDS;
            io.checkpoint(s);
            return Err(());
        }};
    }
    if !s.full {
        if s.t0 == 0 {
            s.t0 = now;
            s.cursor.clear();
            s.pages = 0;
        }
        loop {
            let url = if s.cursor.is_empty() { format!("{base}/mods?limit={PAGE_LIMIT}") } else { format!("{base}/mods?limit={PAGE_LIMIT}&cursor={}", urlenc(&s.cursor)) };
            let body = match get_backoff(io, &url) {
                Ok(b) => b,
                Err((m, _)) => fail!(m),
            };
            let Some((_, next, total)) = apply_page(io, &body, false) else { fail!("SSBM Nucleus sent something unreadable.".into()) };
            s.pages += 1;
            s.total = total.unwrap_or(s.total);
            s.cursor = next.clone();
            let n = io.with_cat(|c| c.m.len());
            io.progress("Downloading the catalog", n, s.total);
            if s.pages % 10 == 0 {
                io.checkpoint(s);
            }
            if next.is_empty() {
                break;
            }
            io.sleep_ms(700);
        }
        s.full = true;
        s.since = s.t0;
        s.cursor.clear();
        io.checkpoint(s);
    }
    // the delta: everything updated since W - 15 minutes, then the removals
    let enc = urlenc(&iso(s.since - OVERLAP_SECONDS));
    let mut next = String::new();
    loop {
        let cur = if next.is_empty() { String::new() } else { format!("&cursor={}", urlenc(&next)) };
        let body = match get_backoff(io, &format!("{base}/mods?limit={PAGE_LIMIT}&updated_since={enc}{cur}")) {
            Ok(b) => b,
            Err((m, _)) => fail!(m),
        };
        let Some((_, nx, _)) = apply_page(io, &body, false) else { fail!("SSBM Nucleus sent something unreadable.".into()) };
        let n = io.with_cat(|c| c.m.len());
        io.progress("Checking for changes", n, 0);
        next = nx;
        if next.is_empty() {
            break;
        }
        io.sleep_ms(700);
    }
    loop {
        let cur = if next.is_empty() { String::new() } else { format!("&cursor={}", urlenc(&next)) };
        io.sleep_ms(700);
        let body = match get_backoff(io, &format!("{base}/mods/removed?limit=1000&since={enc}{cur}")) {
            Ok(b) => b,
            Err((m, _)) => fail!(m),
        };
        let Some((_, nx, _)) = apply_page(io, &body, true) else { fail!("SSBM Nucleus sent something unreadable.".into()) };
        next = nx;
        if next.is_empty() {
            break;
        }
    }
    recheck_empty(io, base);
    s.since = start;
    s.last_poll = start;
    s.next_ok = 0;
    io.checkpoint(s);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const PAGE1: &str = r#"{"data":[{"id":12816,"title":"Jessie - Princess Peach","author":"SSBM Textures","type":"costume","tags":["Peach"],"created_at":"2020-08-17T20:51:10.000000Z","updated_at":"2026-02-21T04:33:55.000000Z","download_count":42,"like_count":0,"page_url":"https://ssbmnucleus.net/post/12816/x","screenshots":[],"files":[{"id":7271,"filename":"PlPeNr Jessie Peach.dat.dat","file_type":"character_dat","character":"Peach","color":"Default","csp_url":"https://media.ssbmnucleus.net/posts/a_csp.png","stock_url":null,"download_url":"https://ssbmnucleus.net/api/public/v1/mods/12816/download?file=7271","file_url":"x"}]}],"next_cursor":"abc=","total":2}"#;
    const PAGE2: &str = r#"{"data":[{"id":12815,"title":"Pumpkin Hill - Dreamland","author":"SSBM Textures","type":"stage_skin","stage":"Dreamland","created_at":"2020-08-17T20:36:05Z","updated_at":"2026-02-21T04:33:50Z","download_count":8,"files":[{"id":7270,"filename":"Pumpkin Hill - GrOp.dat","file_type":"stage_dat","character":null,"color":null}]}],"next_cursor":null,"total":2}"#;

    struct Fake {
        cat: RefCell<Catalog>,
        urls: RefCell<Vec<String>>,
        slept: RefCell<u64>,
    }
    impl SyncIo for Fake {
        fn fetch(&self, url: &str) -> crate::nucleus::http::Resp {
            self.urls.borrow_mut().push(url.to_string());
            let body = if url.contains("cursor=abc%3D") && !url.contains("updated_since") {
                PAGE2
            } else if url.contains("/removed") {
                r#"{"data":[{"id":12815}],"next_cursor":null}"#
            } else if url.contains("updated_since") {
                r#"{"data":[],"next_cursor":null}"#
            } else {
                PAGE1
            };
            crate::nucleus::http::Resp { status: 200, body: body.as_bytes().to_vec(), retry_after: 0, err: String::new() }
        }
        fn now(&self) -> i64 {
            1_800_000_000
        }
        fn sleep_ms(&self, ms: u64) {
            *self.slept.borrow_mut() += ms;
        }
        fn stop(&self) -> bool {
            false
        }
        fn progress(&self, _: &str, _: usize, _: i64) {}
        fn checkpoint(&self, _: &SyncState) {}
        fn with_cat<R>(&self, f: impl FnOnce(&mut Catalog) -> R) -> R {
            f(&mut self.cat.borrow_mut())
        }
    }

    #[test]
    fn sync_pages_delta_removed_and_rules() {
        let io = Fake { cat: RefCell::new(Catalog::default()), urls: RefCell::new(vec![]), slept: RefCell::new(0) };
        let mut s = SyncState::default();
        assert_eq!(sync_run(&mut s, &io, API_BASE, false), Ok(true));
        let u = io.urls.borrow().clone();
        assert_eq!(u[0], format!("{API_BASE}/mods?limit=100"));
        assert_eq!(u[1], format!("{API_BASE}/mods?limit=100&cursor=abc%3D"));
        assert!(u[2].contains("updated_since=") && u[3].contains("/mods/removed?limit=1000&since="));
        assert!(s.full && s.last_poll == 1_800_000_000);
        assert!(*io.slept.borrow() >= 700 * 2, "pages are spaced");
        let c = io.cat.borrow();
        assert_eq!(c.m.len(), 1, "the removed feed took 12815 out");
        assert_eq!(c.gone, vec![12815]);
        let m = &c.m[0];
        assert_eq!((m.typ, m.tab, m.files.len()), (T_COSTUME, TAB_COSTUMES, 1));
        assert_eq!(m.files[0].fighter, fighter_by_pl("Pe"));
        assert_eq!(m.files[0].csp, "posts/a_csp.png");
        assert_eq!(why_not(m), "");
        drop(c);
        // the 10-minute rule: a second run fetches nothing
        let n = io.urls.borrow().len();
        assert_eq!(sync_run(&mut s, &io, API_BASE, false), Ok(false));
        assert_eq!(io.urls.borrow().len(), n);
        // the cache round-trips
        let t = io.cat.borrow().save_text();
        let back = Catalog::load_text(&t);
        assert_eq!(back.m.len(), 1);
        assert_eq!(back.gone, vec![12815]);
        assert_eq!(back.m[0].files[0].csp, "posts/a_csp.png");
        assert_eq!(back.filter(&Query { text: "jessie".into(), ..Default::default() }), vec![0]);
        assert!(back.filter(&Query { text: "zzz".into(), ..Default::default() }).is_empty());
        let st = SyncState::from_json(&s.to_json());
        assert!(st.full && !st.stale && st.since == s.since);
    }

    #[test]
    fn kinds_from_names() {
        assert_eq!(stage_stem_in("Pumpkin Hill - GrOp.dat"), Some("GrOp"));
        assert_eq!(stage_stem_in("GrPs3.dat"), Some("GrPs3"));
        assert_eq!(file_category("other", "EfFxData.dat").0, FC_EFFECT);
        assert_eq!(file_category("other", "MnSlChr.usd"), (FC_MENU, "MnSlChr.usd".into(), true));
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(parse_iso("2026-10-08T22:37:39Z").map(iso).as_deref(), Some("2026-10-08T22:37:39Z"));
    }
}
