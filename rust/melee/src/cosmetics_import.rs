//! Cosmetics importers and pictures: costume ZIPs, saved Nucleus vault ZIPs, portraits (the character select
//! picture, "csp") and stock icons, into the `Cosmetics/` catalog of cosmetics.rs.
//!
//! * Mod ZIP: every member that parses as a costume archive (`*_Share_joint` root) and names a slot (its file name,
//!   or `mod.json` `"slot"`) is one skin; `mod.json` `"name"` names it, else the ZIP's stem. PNGs whose names say
//!   `stock` / `csp` / `portrait` are the skin's pictures (with several costumes, a PNG goes to the costume whose code
//!   its name carries). A ZIP of PNGs only is a portrait pack: each PNG whose name holds a slot code.
//! * Nucleus vault ZIP (`metadata.json` at the root): `characters.<group>.skins[]` records (`id`, `color`,
//!   `filename`, `costume_code`, `has_csp`, `has_stock`), each `filename` a nested ZIP holding one costume archive,
//!   its pictures beside it as `<stem>_csp.png` / `<stem>_stc.png`. The whole vault is planned and checked before
//!   anything is written: it goes in whole or not at all. Effects and stages in a vault are not imported (counted).
//! * Pictures: `skins/<slot>/<name>.csp.png|.stock.png` (the skin's own), `portraits/<slot>/csp.png|stock.png` (the
//!   costume's, switched by `portrait_<slot>=on|off`). The selected skin's own picture wins; the costume's fills in.
//!   They are drawn by texture override (melee_gx `tex_override`): the character select file's portrait and stock
//!   animations say which texture is a slot's (frame = fighter column + 30 * costume, read through the animation's
//!   keys); the renderer uploads the PNG in place of that payload. Render-only: nothing reaches the simulation, so a
//!   picture is a local choice even online.
//! * Safety: the skeleton rule of cosmetics.rs still decides mounting (an import only checks that a costume parses;
//!   `melee cosmetics check` and the boot compare it with the disc). Caps: 512 MB archive, 4096 entries, 64 MB per
//!   member, 16 MB metadata, pictures up to 1024x1024.
//!
//! Reference (behaviour only, reimplemented): Hero's melee-unlocked `cosmetic_mods.cpp` (mod ZIP / vault import,
//! portrait and stock companions, frame = column + 30 * costume).
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAX_ARCHIVE: u64 = 512 << 20;
const MAX_MEMBER: usize = 64 << 20;
const MAX_ENTRIES: usize = 4096;

// ---------------------------------------------------------------- ZIP

pub struct Entry {
    pub name: String,
    method: u16,
    csize: usize,
    size: usize,
    local: usize,
}

pub struct Zip {
    bytes: Vec<u8>,
    pub entries: Vec<Entry>,
}

fn le16(b: &[u8], o: usize) -> Option<u16> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}
fn le32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

impl Zip {
    pub fn parse(bytes: Vec<u8>) -> Result<Zip, String> {
        if bytes.len() as u64 > MAX_ARCHIVE {
            return Err("archive larger than 512 MB".into());
        }
        if bytes.len() < 22 {
            return Err("not a ZIP archive".into());
        }
        let lo = bytes.len().saturating_sub(22 + 0xFFFF);
        let eocd = (lo..=bytes.len() - 22).rev().find(|&o| le32(&bytes, o) == Some(0x0605_4b50)).ok_or("not a ZIP archive (no end record)")?;
        let n = le16(&bytes, eocd + 10).unwrap() as usize;
        let mut o = le32(&bytes, eocd + 16).unwrap() as usize;
        if n == 0xFFFF || o == 0xFFFF_FFFF {
            return Err("ZIP64 archives are not supported".into());
        }
        if n > MAX_ENTRIES {
            return Err(format!("{n} entries (at most {MAX_ENTRIES})"));
        }
        let mut entries = Vec::with_capacity(n);
        for _ in 0..n {
            if le32(&bytes, o) != Some(0x0201_4b50) {
                return Err("bad central directory".into());
            }
            let r = |k| le32(&bytes, o + k).map(|v| v as usize).ok_or("bad central directory");
            let (method, csize, size, local) = (le16(&bytes, o + 10).unwrap_or(0), r(20)?, r(24)?, r(42)?);
            let l16 = |k| le16(&bytes, o + k).map(|v| v as usize).ok_or("bad central directory");
            let (nl, el, cl) = (l16(28)?, l16(30)?, l16(32)?);
            let name = bytes.get(o + 46..o + 46 + nl).ok_or("bad entry name")?;
            let name = String::from_utf8_lossy(name).replace('\\', "/");
            o += 46 + nl + el + cl;
            if !name.ends_with('/') {
                entries.push(Entry { name, method, csize, size, local });
            }
        }
        Ok(Zip { bytes, entries })
    }

    pub fn read(&self, e: &Entry) -> Result<Vec<u8>, String> {
        if e.size > MAX_MEMBER {
            return Err(format!("{}: larger than 64 MB", e.name));
        }
        let b = &self.bytes;
        if le32(b, e.local) != Some(0x0403_4b50) {
            return Err(format!("{}: bad local header", e.name));
        }
        let start = e.local + 30 + le16(b, e.local + 26).unwrap_or(0) as usize + le16(b, e.local + 28).unwrap_or(0) as usize;
        let data = b.get(start..start + e.csize).ok_or_else(|| format!("{}: past the end", e.name))?;
        let out = match e.method {
            0 => data.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec_with_limit(data, MAX_MEMBER).map_err(|_| format!("{}: bad deflate data", e.name))?,
            m => return Err(format!("{}: compression method {m} not supported", e.name)),
        };
        if out.len() != e.size {
            return Err(format!("{}: size mismatch", e.name));
        }
        Ok(out)
    }

    pub fn find(&self, pred: impl Fn(&str) -> bool) -> Vec<&Entry> {
        self.entries.iter().filter(|e| pred(&e.name)).collect()
    }
}

fn base(name: &str) -> &str {
    name.rsplit(['/', '\\']).next().unwrap_or(name)
}

fn is_costume_name(n: &str) -> bool {
    let l = n.to_ascii_lowercase();
    l.ends_with(".dat") || l.ends_with(".usd")
}

// ---------------------------------------------------------------- JSON (the subset metadata files use)

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(v) => v.iter().find(|e| e.0 == k).map(|e| &e.1),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        if let Json::Str(s) = self { Some(s) } else { None }
    }
    pub fn truthy(&self) -> bool {
        matches!(self, Json::Bool(true))
    }

    pub fn parse(s: &str) -> Result<Json, String> {
        let b = s.as_bytes();
        let mut i = 0;
        let v = val(b, &mut i, 0)?;
        ws(b, &mut i);
        if i != b.len() {
            return Err("trailing data".into());
        }
        Ok(v)
    }
}

fn ws(b: &[u8], i: &mut usize) {
    while *i < b.len() && b[*i].is_ascii_whitespace() {
        *i += 1;
    }
}

fn sep(b: &[u8], i: &mut usize, first: bool) -> Result<(), String> {
    if !first {
        if b.get(*i) != Some(&b',') {
            return Err(format!("expected , at {}", *i));
        }
        *i += 1;
    }
    Ok(())
}

fn val(b: &[u8], i: &mut usize, depth: u32) -> Result<Json, String> {
    if depth > 64 {
        return Err("nested too deep".into());
    }
    ws(b, i);
    let c = *b.get(*i).ok_or("unexpected end")?;
    let lit = |i: &mut usize, w: &str, v: Json| -> Result<Json, String> {
        if b[*i..].starts_with(w.as_bytes()) {
            *i += w.len();
            Ok(v)
        } else {
            Err(format!("bad token at {}", *i))
        }
    };
    match c {
        b'{' => {
            *i += 1;
            let mut v = Vec::new();
            loop {
                ws(b, i);
                if b.get(*i) == Some(&b'}') {
                    *i += 1;
                    return Ok(Json::Obj(v));
                }
                sep(b, i, v.is_empty())?;
                let Json::Str(k) = val(b, i, depth + 1)? else { return Err("object key is not a string".into()) };
                ws(b, i);
                if b.get(*i) != Some(&b':') {
                    return Err(format!("expected : at {}", *i));
                }
                *i += 1;
                v.push((k, val(b, i, depth + 1)?));
            }
        }
        b'[' => {
            *i += 1;
            let mut v = Vec::new();
            loop {
                ws(b, i);
                if b.get(*i) == Some(&b']') {
                    *i += 1;
                    return Ok(Json::Arr(v));
                }
                sep(b, i, v.is_empty())?;
                v.push(val(b, i, depth + 1)?);
            }
        }
        b'"' => {
            *i += 1;
            let mut s = Vec::new();
            loop {
                let c = *b.get(*i).ok_or("unterminated string")?;
                *i += 1;
                match c {
                    b'"' => return Ok(Json::Str(String::from_utf8_lossy(&s).into_owned())),
                    b'\\' => {
                        let e = *b.get(*i).ok_or("bad escape")?;
                        *i += 1;
                        match e {
                            b'n' => s.push(b'\n'),
                            b't' => s.push(b'\t'),
                            b'r' => s.push(b'\r'),
                            b'b' | b'f' => {}
                            b'u' => {
                                let h = b.get(*i..*i + 4).and_then(|h| std::str::from_utf8(h).ok()).ok_or("bad escape")?;
                                let cp = u32::from_str_radix(h, 16).map_err(|_| "bad escape")?;
                                *i += 4;
                                let ch = char::from_u32(cp).unwrap_or('\u{FFFD}');
                                s.extend_from_slice(ch.to_string().as_bytes());
                            }
                            o => s.push(o),
                        }
                    }
                    o => s.push(o),
                }
            }
        }
        b't' => lit(i, "true", Json::Bool(true)),
        b'f' => lit(i, "false", Json::Bool(false)),
        b'n' => lit(i, "null", Json::Null),
        _ => {
            let st = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                *i += 1;
            }
            std::str::from_utf8(&b[st..*i]).ok().and_then(|t| t.parse().ok()).map(Json::Num).ok_or_else(|| format!("bad value at {st}"))
        }
    }
}

// ---------------------------------------------------------------- PNG

/// Decode a picture to RGBA8 (gray / gray+alpha / RGB / RGBA / indexed), at most 1024x1024.
pub fn decode_png(bytes: &[u8]) -> Result<melee_gx::tex_override::Picture, String> {
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = d.read_info().map_err(|e| format!("not a PNG: {e}"))?;
    let mut buf = vec![0; r.output_buffer_size().ok_or("PNG too large")?];
    let info = r.next_frame(&mut buf).map_err(|e| format!("bad PNG: {e}"))?;
    let (w, h) = (info.width, info.height);
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return Err(format!("picture {w}x{h} (at most 1024x1024)"));
    }
    let n = (w * h) as usize;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..n * 4].to_vec(),
        png::ColorType::Rgb => buf[..n * 3].chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf[..n * 2].chunks(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf[..n].iter().flat_map(|&g| [g, g, g, 255]).collect(),
        c => return Err(format!("PNG colour type {c:?} not supported")),
    };
    Ok(melee_gx::tex_override::Picture { w, h, rgba })
}

// ---------------------------------------------------------------- names

/// The first `pl` + fighter + colour code in a name that is a slot with a cell (`fox PlFxGr stock.png` -> plfxgr).
pub fn slot_in_name(name: &str) -> Option<String> {
    let l = base(name).to_ascii_lowercase();
    let b = l.as_bytes();
    for i in 0..(b.len() + 1).saturating_sub(6) {
        if &b[i..i + 2] == b"pl" && (i == 0 || !b[i - 1].is_ascii_alphanumeric()) && (i + 6 == b.len() || !b[i + 6].is_ascii_alphanumeric()) {
            let s = &l[i..i + 6];
            if portrait_frame(s).is_some() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// "csp" or "stock" from a picture's name (Nucleus `_stc` included); None when it says neither.
pub fn picture_kind(name: &str) -> Option<&'static str> {
    let l = base(name).to_ascii_lowercase();
    if !l.ends_with(".png") {
        return None;
    }
    if l.contains("stock") || l.contains("_stc") {
        Some("stock")
    } else if l.contains("csp") || l.contains("portrait") {
        Some("csp")
    } else {
        None
    }
}

/// A catalog-safe skin name.
pub fn clean_name(n: &str) -> String {
    let s: String = n.chars().map(|c| if c.is_control() || "/\\.=:*?\"<>|".contains(c) { '-' } else { c }).collect();
    let s = s.trim().trim_matches('-').trim().to_string();
    if s.is_empty() { "skin".into() } else { s.chars().take(64).collect() }
}

/// The character select screen's fighter columns, each with its costumes in the game's order. The portrait of slot
/// `Pl<f><c>` is frame `column + 30 * costume` of the portrait (and stock) animation. Nana and Sheik show Popo's /
/// Zelda's cells and have none of their own.
const COLUMNS: [(&str, &[&str]); 25] = [
    ("ca", &["nr", "gy", "re", "wh", "gr", "bu"]),
    ("dk", &["nr", "bk", "re", "bu", "gr"]),
    ("fx", &["nr", "or", "la", "gr"]),
    ("gw", &["nr"]),
    ("kb", &["nr", "ye", "bu", "re", "gr", "wh"]),
    ("kp", &["nr", "re", "bu", "bk"]),
    ("lk", &["nr", "re", "bu", "bk", "wh"]),
    ("lg", &["nr", "wh", "aq", "pi"]),
    ("mr", &["nr", "ye", "bk", "bu", "gr"]),
    ("ms", &["nr", "re", "gr", "bk", "wh"]),
    ("mt", &["nr", "re", "bu", "gr"]),
    ("ns", &["nr", "ye", "bu", "gr"]),
    ("pe", &["nr", "ye", "wh", "bu", "gr"]),
    ("pk", &["nr", "re", "bu", "gr"]),
    ("pp", &["nr", "gr", "or", "re"]),
    ("pr", &["nr", "re", "bu", "gr", "ye"]),
    ("ss", &["nr", "pi", "bk", "gr", "la"]),
    ("ys", &["nr", "re", "bu", "ye", "pi", "aq"]),
    ("zd", &["nr", "re", "bu", "gr", "wh"]),
    ("fc", &["nr", "re", "bu", "gr"]),
    ("cl", &["nr", "re", "bu", "wh", "bk"]),
    ("dr", &["nr", "re", "bu", "gr", "bk"]),
    ("fe", &["nr", "re", "bu", "gr", "wh"]),
    ("pc", &["nr", "re", "bu", "gr"]),
    ("gn", &["nr", "re", "bu", "gr", "la"]),
];

/// The portrait / stock frame of a slot (`plfxgr` -> 2 + 30 * 3), None for a slot with no cell of its own.
pub fn portrait_frame(slot: &str) -> Option<u32> {
    if !slot.starts_with("pl") || slot.len() != 6 {
        return None;
    }
    let (f, c) = (slot.get(2..4)?, slot.get(4..6)?);
    let col = COLUMNS.iter().position(|x| x.0 == f)?;
    let cos = COLUMNS[col].1.iter().position(|&x| x == c)?;
    Some(col as u32 + 30 * cos as u32)
}

// ---------------------------------------------------------------- import

/// One planned catalog write (an empty `to` is a note line only).
struct Write {
    to: PathBuf,
    bytes: Vec<u8>,
    what: String,
}

fn commit(plan: Vec<Write>) -> Result<Vec<String>, String> {
    for w in plan.iter().filter(|w| !w.to.as_os_str().is_empty()) {
        if let Some(d) = w.to.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        std::fs::write(&w.to, &w.bytes).map_err(|e| format!("{}: {e}", w.to.display()))?;
    }
    Ok(plan.into_iter().map(|w| w.what).filter(|w| !w.is_empty()).collect())
}

/// A free skin name in `slot` (`name`, `name 2`, ...), also not taken by an earlier write of the same plan.
fn free_name(root: &Path, slot: &str, name: &str, plan: &[Write]) -> String {
    let d = root.join("skins").join(slot);
    let taken = |n: &str| {
        let p = d.join(format!("{n}.dat"));
        p.exists() || plan.iter().any(|w| w.to == p)
    };
    let mut n = name.to_string();
    let mut k = 2;
    while taken(&n) {
        n = format!("{name} {k}");
        k += 1;
    }
    n
}

fn plan_skin(root: &Path, plan: &mut Vec<Write>, slot: &str, name: &str, dat: Vec<u8>, pics: Vec<(&'static str, Vec<u8>)>) -> Result<(), String> {
    crate::cosmetics::skeleton(&dat).map_err(|e| format!("{}: {e}", crate::cosmetics::slot_file(slot)))?;
    let name = free_name(root, slot, &clean_name(name), plan);
    let d = root.join("skins").join(slot);
    let mut what = format!("{} '{name}'", crate::cosmetics::slot_file(slot));
    for (kind, b) in pics {
        decode_png(&b).map_err(|e| format!("{what} {kind}: {e}"))?;
        what += &format!(" +{kind}");
        plan.push(Write { to: d.join(format!("{name}.{kind}.png")), bytes: b, what: String::new() });
    }
    plan.push(Write { to: d.join(format!("{name}.dat")), bytes: dat, what });
    Ok(())
}

/// Import any supported file: `.dat` / `.usd` costume, `.zip` (mod or Nucleus vault), `.png` picture.
/// `slot` / `name` / `kind` override what the file names say; `skin` gives a picture to that installed skin.
/// Returns one line per thing imported.
pub fn import(root: &Path, src: &Path, slot: Option<String>, name: Option<String>, kind: Option<String>, skin: Option<String>) -> Result<Vec<String>, String> {
    let fname = src.file_name().unwrap_or_default().to_string_lossy().to_string();
    let ext = src.extension().unwrap_or_default().to_string_lossy().to_ascii_lowercase();
    let meta = std::fs::metadata(src).map_err(|e| format!("{}: {e}", src.display()))?;
    if meta.len() > MAX_ARCHIVE {
        return Err("file larger than 512 MB".into());
    }
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let stem = src.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let mut plan = Vec::new();
    match ext.as_str() {
        "dat" | "usd" => {
            if bytes.len() > MAX_MEMBER {
                return Err("costume larger than 64 MB".into());
            }
            let s = slot.or_else(|| crate::cosmetics::slot_of(&format!("{stem}.dat"))).ok_or("cannot tell the costume slot from the file name: pass --slot PlXxYy.dat")?;
            plan_skin(root, &mut plan, &s, &name.unwrap_or(stem), bytes, vec![])?;
        }
        "png" => {
            let s = slot.or_else(|| slot_in_name(&fname)).ok_or("the name holds no costume code (PlFxGr...): pass --slot PlXxYy.dat")?;
            let k: &'static str = match kind.as_deref().or(picture_kind(&fname)) {
                Some("stock") => "stock",
                _ => "csp",
            };
            decode_png(&bytes)?;
            let to = match skin {
                Some(n) => {
                    if !root.join("skins").join(&s).join(format!("{n}.dat")).exists() {
                        return Err(format!("no skin '{n}' for {}", crate::cosmetics::slot_file(&s)));
                    }
                    root.join("skins").join(&s).join(format!("{n}.{k}.png"))
                }
                None => root.join("portraits").join(&s).join(format!("{k}.png")),
            };
            plan.push(Write { to, bytes, what: format!("{} {k} picture", crate::cosmetics::slot_file(&s)) });
        }
        "zip" => {
            let z = Zip::parse(bytes)?;
            if let Some(m) = z.entries.iter().find(|e| e.name.eq_ignore_ascii_case("metadata.json")) {
                return commit(plan_vault(root, &z, m)?);
            }
            plan_mod_zip(root, &z, &stem, slot, name, &mut plan)?;
        }
        _ => return Err("choose a costume .dat / .usd, a mod or Nucleus vault .zip, or a portrait / stock icon .png".into()),
    }
    commit(plan)
}

fn plan_mod_zip(root: &Path, z: &Zip, stem: &str, slot: Option<String>, name: Option<String>, plan: &mut Vec<Write>) -> Result<(), String> {
    let mut mname = None;
    let mut mslot = slot;
    if let Some(m) = z.entries.iter().find(|e| base(&e.name).eq_ignore_ascii_case("mod.json")) {
        if m.size > 64 << 10 {
            return Err("mod.json larger than 64 KB".into());
        }
        let j = Json::parse(&String::from_utf8_lossy(&z.read(m)?)).map_err(|e| format!("mod.json: {e}"))?;
        mname = j.get("name").and_then(Json::str).map(str::to_string);
        if mslot.is_none() {
            mslot = j.get("slot").and_then(Json::str).and_then(|s| slot_in_name(s).or_else(|| crate::cosmetics::slot_of(&format!("{s}.dat"))));
        }
    }
    let mut costumes: Vec<(String, Vec<u8>)> = Vec::new();
    for e in z.find(is_costume_name) {
        let b = z.read(e)?;
        if crate::cosmetics::skeleton(&b).is_err() {
            continue;
        }
        let st = base(&e.name);
        let st = &st[..st.len() - 4];
        let Some(s) = crate::cosmetics::slot_of(&format!("{st}.dat")).or_else(|| mslot.clone()) else {
            return Err(format!("{}: cannot tell its costume slot (name it PlXxYy.dat or set mod.json \"slot\")", e.name));
        };
        if let Some(o) = costumes.iter().find(|c| c.0 == s) {
            if o.1 != b {
                return Err(format!("two different costume files for {}", crate::cosmetics::slot_file(&s)));
            }
            continue;
        }
        costumes.push((s, b));
    }
    let pngs = z.find(|n| n.to_ascii_lowercase().ends_with(".png"));
    if costumes.is_empty() {
        // a portrait pack
        let mut n = 0;
        for e in &pngs {
            let Some(s) = slot_in_name(&e.name) else { continue };
            let k = picture_kind(&e.name).unwrap_or("csp");
            let b = z.read(e)?;
            decode_png(&b).map_err(|x| format!("{}: {x}", e.name))?;
            plan.push(Write { to: root.join("portraits").join(&s).join(format!("{k}.png")), bytes: b, what: format!("{} {k} picture", crate::cosmetics::slot_file(&s)) });
            n += 1;
        }
        if n == 0 {
            return Err("the ZIP holds no costume archive and no picture named for a costume".into());
        }
        return Ok(());
    }
    let one = costumes.len() == 1;
    let label = name.or(mname).unwrap_or_else(|| stem.to_string());
    for (s, dat) in costumes {
        let mut pics: Vec<(&'static str, Vec<u8>)> = Vec::new();
        for e in &pngs {
            let Some(k) = picture_kind(&e.name) else { continue };
            let named = slot_in_name(&e.name);
            if named.as_deref() == Some(s.as_str()) || (one && named.is_none()) {
                if pics.iter().any(|p| p.0 == k) {
                    return Err(format!("several {k} pictures for {}", crate::cosmetics::slot_file(&s)));
                }
                pics.push((k, z.read(e)?));
            }
        }
        let n = if one { label.clone() } else { format!("{label} {}", crate::cosmetics::slot_file(&s).trim_end_matches(".dat")) };
        plan_skin(root, plan, &s, &n, dat, pics)?;
    }
    Ok(())
}

fn plan_vault(root: &Path, z: &Zip, meta: &Entry) -> Result<Vec<Write>, String> {
    if meta.size > 16 << 20 {
        return Err("metadata.json larger than 16 MB".into());
    }
    let j = Json::parse(&String::from_utf8_lossy(&z.read(meta)?)).map_err(|e| format!("Nucleus metadata.json: {e}"))?;
    let Some(Json::Obj(chars)) = j.get("characters") else { return Err("Nucleus metadata.json has no character catalog".into()) };
    let mut plan = Vec::new();
    let mut ids = std::collections::HashSet::new();
    let (mut skipped, mut notes) = (0usize, Vec::new());
    for (group, g) in chars {
        if let Some(Json::Obj(x)) = g.get("extras") {
            skipped += x.iter().map(|e| if let Json::Arr(a) = &e.1 { a.len() } else { 0 }).sum::<usize>();
        }
        let Some(skins) = g.get("skins") else { continue };
        let Json::Arr(skins) = skins else { return Err(format!("Nucleus group {group}: malformed skin list")) };
        for sk in skins {
            let f = |k: &str| sk.get(k).and_then(Json::str).map(str::to_string).ok_or_else(|| format!("Nucleus {group} skin: missing \"{k}\""));
            let (id, color, file, code) = (f("id")?, f("color")?, f("filename")?, f("costume_code")?);
            if !ids.insert(id.to_ascii_lowercase()) {
                return Err(format!("Nucleus metadata: duplicate skin id {id}"));
            }
            if file.contains(['/', '\\']) || !file.to_ascii_lowercase().ends_with(".zip") {
                return Err(format!("Nucleus skin {id}: filename '{file}' is not a ZIP base name"));
            }
            let m = z.find(|n| base(n).eq_ignore_ascii_case(&file));
            let [m] = m[..] else { return Err(format!("Nucleus skin {id}: '{file}' is {} in the vault", if m.is_empty() { "missing" } else { "ambiguous" })) };
            let inner = Zip::parse(z.read(m)?).map_err(|e| format!("{file}: {e}"))?;
            let mut dat = None;
            for e in inner.find(is_costume_name) {
                let b = inner.read(e)?;
                if crate::cosmetics::skeleton(&b).is_ok() {
                    if dat.is_some() {
                        return Err(format!("{file}: more than one costume archive"));
                    }
                    dat = Some((base(&e.name).to_string(), b));
                }
            }
            let (member, dat) = dat.ok_or_else(|| format!("{file}: no costume archive"))?;
            let slot = crate::cosmetics::slot_of(&format!("{code}.dat")).ok_or_else(|| format!("Nucleus skin {id}: costume_code '{code}' is not a slot"))?;
            if let Some(ms) = crate::cosmetics::slot_of(&format!("{}.dat", &member[..member.len() - 4])) {
                if ms != slot {
                    return Err(format!("Nucleus skin {id}: metadata says {code}, the archive is {member}"));
                }
            }
            let parent = m.name.rfind('/').map_or("", |i| &m.name[..=i]);
            let st = &file[..file.len() - 4];
            let mut pics = Vec::new();
            for (k, suf) in [("csp", "_csp.png"), ("stock", "_stc.png")] {
                let want = format!("{parent}{st}{suf}").to_ascii_lowercase();
                match z.entries.iter().find(|e| e.name.to_ascii_lowercase() == want) {
                    Some(e) => pics.push((k, z.read(e)?)),
                    None if sk.get(&format!("has_{k}")).is_some_and(Json::truthy) => notes.push(format!("note: {id}: {k} picture missing (the game's is shown)")),
                    None => {}
                }
            }
            plan_skin(root, &mut plan, &slot, &color, dat, pics).map_err(|e| format!("Nucleus skin {id}: {e}"))?;
        }
    }
    if plan.is_empty() {
        return Err("the vault holds no costume skins".into());
    }
    if skipped > 0 {
        notes.push(format!("note: {skipped} effect records not imported (costumes only)"));
    }
    plan.extend(notes.into_iter().map(|what| Write { to: PathBuf::new(), bytes: Vec::new(), what }));
    Ok(plan)
}

// ---------------------------------------------------------------- pictures at run time

/// The pictures to show: (slot, kind) -> PNG path. The selected skin's own picture, else the costume's (unless
/// `portrait_<slot>=off`).
pub fn chosen_pictures(root: &Path) -> HashMap<(String, &'static str), PathBuf> {
    let skins = crate::cosmetics::list(root);
    let pick = crate::cosmetics::chosen(&skins);
    let mut out = HashMap::new();
    for (slot, k, p) in slot_pictures(root) {
        if crate::cfg::text(&format!("portrait_{slot}")).is_none_or(|v| v != "off" && v != "0") {
            out.insert((slot, k), p);
        }
    }
    for (slot, name) in pick {
        if let Some(sk) = skins.iter().find(|s| s.slot == slot && s.name.eq_ignore_ascii_case(&name)) {
            for k in ["csp", "stock"] {
                let p = sk.path.with_extension(format!("{k}.png"));
                if p.is_file() {
                    out.insert((slot.clone(), k), p);
                }
            }
        }
    }
    out
}

/// Every costume picture in `portraits/` (slot, kind, path), by slot.
pub fn slot_pictures(root: &Path) -> Vec<(String, &'static str, PathBuf)> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(root.join("portraits")) {
        for e in rd.flatten() {
            let slot = e.file_name().to_string_lossy().to_ascii_lowercase();
            if portrait_frame(&slot).is_none() {
                continue;
            }
            for k in ["csp", "stock"] {
                let p = e.path().join(format!("{k}.png"));
                if p.is_file() {
                    v.push((slot.clone(), k, p));
                }
            }
        }
    }
    v.sort();
    v
}

fn be32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_be_bytes(s.try_into().unwrap()))
}
fn be16(b: &[u8], o: usize) -> Option<u16> {
    b.get(o..o + 2).map(|s| u16::from_be_bytes([s[0], s[1]]))
}

/// The keys of an HSD FObj track: (frame, value), frames accumulated from the waits.
fn fobj_keys(body: &[u8], ad: usize, len: usize, frac: u8, frac_slope: u8) -> Option<Vec<(u32, f32)>> {
    let end = ad.checked_add(len)?.min(body.len());
    let mut p = ad;
    let rd = |p: &mut usize, f: u8| -> Option<f32> {
        let d = (1u32 << (f & 0x1F).min(31)) as f32;
        let v = match f & 0xE0 {
            0x00 => {
                let x = f32::from_bits(be32(body, *p)?);
                *p += 4;
                return Some(x);
            }
            0x20 => {
                let x = be16(body, *p)? as i16 as f32;
                *p += 2;
                x
            }
            0x40 => {
                let x = be16(body, *p)? as f32;
                *p += 2;
                x
            }
            0x60 => {
                let x = *body.get(*p)? as i8 as f32;
                *p += 1;
                x
            }
            0x80 => {
                let x = *body.get(*p)? as f32;
                *p += 1;
                x
            }
            _ => return None,
        };
        Some(v / d)
    };
    let varint = |p: &mut usize| -> Option<u32> {
        let (mut v, mut sh) = (0u32, 0);
        loop {
            let b = *body.get(*p)?;
            *p += 1;
            v |= ((b & 0x7F) as u32) << sh;
            sh += 7;
            if b & 0x80 == 0 || sh > 28 {
                return Some(v);
            }
        }
    };
    let mut out = Vec::new();
    let mut frame = 0u32;
    while p < end && out.len() < 4096 {
        let mut b = body[p];
        p += 1;
        let op = b & 0x0F;
        let mut n = ((b >> 4) & 7) as u32;
        let mut sh = 3;
        while b & 0x80 != 0 && sh < 28 {
            b = *body.get(p)?;
            p += 1;
            n |= ((b & 0x7F) as u32) << sh;
            sh += 7;
        }
        n += 1;
        for _ in 0..n {
            let v = match op {
                1 | 2 | 3 | 6 => rd(&mut p, frac)?,
                4 => {
                    let v = rd(&mut p, frac)?;
                    rd(&mut p, frac_slope)?;
                    v
                }
                5 => {
                    rd(&mut p, frac_slope)?;
                    continue;
                }
                _ => return if out.is_empty() { None } else { Some(out) },
            };
            out.push((frame, v));
            frame = frame.saturating_add(varint(&mut p)?);
        }
    }
    Some(out)
}

/// The character select file's portrait (136x188 C8) and stock (square C4) texture animations: kind -> frame ->
/// texture identity.
pub fn select_textures(dat: &[u8]) -> HashMap<&'static str, HashMap<u32, melee_gx::tex_override::Key>> {
    let mut out: HashMap<&'static str, HashMap<u32, melee_gx::tex_override::Key>> = HashMap::new();
    let (Some(data), Some(nrel)) = (be32(dat, 4), be32(dat, 8)) else { return out };
    let Some(body) = dat.get(0x20..0x20 + data as usize) else { return out };
    let relocs: std::collections::HashSet<u32> = (0..nrel as usize).filter_map(|i| be32(dat, 0x20 + data as usize + i * 4)).collect();
    let w = |o: usize| be32(body, o).unwrap_or(0) as usize;
    let desc = |id: usize| (be16(body, id + 4).unwrap_or(0) as u32, be16(body, id + 6).unwrap_or(0) as u32, be32(body, id + 8).unwrap_or(99));
    for o in (0..body.len().saturating_sub(0x18)).step_by(4) {
        if !relocs.contains(&(o as u32 + 0xC)) || !relocs.contains(&(o as u32 + 0x10)) || !relocs.contains(&(o as u32 + 8)) {
            continue;
        }
        let (n, np) = (be16(body, o + 0x14).unwrap() as usize, be16(body, o + 0x16).unwrap() as usize);
        if !(20..=400).contains(&n) || n != np {
            continue;
        }
        let (imgs, pals) = (w(o + 0xC), w(o + 0x10));
        if imgs + 4 * n > body.len() || pals + 4 * n > body.len() {
            continue;
        }
        let kind = match desc(w(imgs)) {
            (136, 188, 9) => "csp",
            (a, b, 8) if a == b && (16..=64).contains(&a) => "stock",
            _ => continue,
        };
        // the animation's image index track (type 1)
        let aobj = w(o + 8);
        let mut track = w(aobj + 8);
        let mut keys = None;
        for _ in 0..64 {
            if track == 0 || track + 0x14 > body.len() {
                break;
            }
            if body[track + 0xC] == 1 {
                keys = fobj_keys(body, w(track + 0x10), w(track + 4), body[track + 0xD], body[track + 0xE]);
            }
            track = w(track);
        }
        let Some(keys) = keys else { continue };
        let mut map = HashMap::new();
        for f in 0..(25 + 30 * 5) as u32 {
            let Some(&(_, v)) = keys.iter().rev().find(|k| k.0 <= f) else { continue };
            let i = v as usize;
            if i >= n {
                continue;
            }
            let (id, pd) = (w(imgs + 4 * i), w(pals + 4 * i));
            let (iw, ih, fmt) = desc(id);
            let Some(sz) = melee_gx::tex_override::level0_size(fmt, iw, ih) else { continue };
            let (tex, lut, ne) = (w(id), w(pd), be16(body, pd + 12).unwrap_or(0) as usize);
            let (Some(t), Some(l)) = (body.get(tex..tex + sz), body.get(lut..lut + ne * 2)) else { continue };
            map.insert(f, melee_gx::tex_override::key(fmt, iw, ih, t, Some(l)));
        }
        if out.get(kind).is_none_or(|e| e.len() < map.len()) {
            out.insert(kind, map);
        }
    }
    out
}

/// Register the chosen pictures as texture overrides, reading the select file through `read`. Returns a summary.
pub fn apply_pictures(read: &mut dyn FnMut(&str) -> Option<Vec<u8>>) -> String {
    let pics = chosen_pictures(&crate::cosmetics::dir());
    if pics.is_empty() {
        if melee_gx::tex_override::active() {
            melee_gx::tex_override::set(None);
        }
        return "no pictures".into();
    }
    let Some(sel) = read("MnSlChr.usd") else { return "MnSlChr.usd is not on the disc".into() };
    let tex = select_textures(&sel);
    let mut map = HashMap::new();
    let mut lines = Vec::new();
    let mut keys: Vec<_> = pics.iter().collect();
    keys.sort();
    for ((slot, kind), path) in keys {
        let Some(f) = portrait_frame(slot) else { continue };
        let Some(k) = tex.get(kind).and_then(|m| m.get(&f)) else {
            lines.push(format!("{} {kind}: no cell", crate::cosmetics::slot_file(slot)));
            continue;
        };
        match std::fs::read(path).map_err(|e| e.to_string()).and_then(|b| decode_png(&b)) {
            Ok(p) => {
                map.insert(*k, Arc::new(p));
            }
            Err(e) => lines.push(format!("{} {kind}: {e}", crate::cosmetics::slot_file(slot))),
        }
    }
    let n = map.len();
    melee_gx::tex_override::set(Some(map));
    let s = format!(
        "{n} pictures over the game's (portrait cells {}, stock cells {}){}",
        tex.get("csp").map_or(0, |m| m.len()),
        tex.get("stock").map_or(0, |m| m.len()),
        if lines.is_empty() { String::new() } else { format!("; {}", lines.join("; ")) }
    );
    eprintln!("cosmetics: {s}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stored (method 0) ZIP of `files`.
    pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let (mut out, mut cd) = (Vec::new(), Vec::new());
        for (n, b) in files {
            let off = out.len() as u32;
            out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&(b.len() as u32).to_le_bytes());
            out.extend_from_slice(&(b.len() as u32).to_le_bytes());
            out.extend_from_slice(&(n.len() as u16).to_le_bytes());
            out.extend_from_slice(&[0, 0]);
            out.extend_from_slice(n.as_bytes());
            out.extend_from_slice(b);
            cd.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            cd.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            cd.extend_from_slice(&(b.len() as u32).to_le_bytes());
            cd.extend_from_slice(&(b.len() as u32).to_le_bytes());
            cd.extend_from_slice(&(n.len() as u16).to_le_bytes());
            cd.extend_from_slice(&[0; 12]);
            cd.extend_from_slice(&off.to_le_bytes());
            cd.extend_from_slice(n.as_bytes());
        }
        let at = out.len() as u32;
        out.extend_from_slice(&cd);
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(cd.len() as u32).to_le_bytes());
        out.extend_from_slice(&at.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    fn png1() -> Vec<u8> {
        let mut v = Vec::new();
        {
            let mut e = png::Encoder::new(&mut v, 2, 2);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(&[255; 16]).unwrap();
        }
        v
    }

    fn costume() -> Vec<u8> {
        let mut body = vec![0u8; 0x40];
        body[4..8].copy_from_slice(&1u32.to_be_bytes());
        let name = b"PlyFox5K_Share_joint\0";
        let mut f = vec![0u8; 0x20];
        let total = 0x20 + body.len() + 8 + name.len();
        f[0..4].copy_from_slice(&(total as u32).to_be_bytes());
        f[4..8].copy_from_slice(&(body.len() as u32).to_be_bytes());
        f[12..16].copy_from_slice(&1u32.to_be_bytes());
        f.extend_from_slice(&body);
        f.extend_from_slice(&[0; 8]);
        f.extend_from_slice(name);
        f
    }

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("melee_cosm_{n}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn json_subset() {
        let j = Json::parse(r#"{"a": [1, 2.5, true, null], "b": {"c": "x\"yA"}}"#).unwrap();
        assert_eq!(j.get("b").and_then(|b| b.get("c")).and_then(Json::str), Some("x\"yA"));
        assert!(Json::parse("{\"a\":}").is_err());
    }

    #[test]
    fn names_and_frames() {
        assert_eq!(slot_in_name("Fox PlFxGr stock.png").as_deref(), Some("plfxgr"));
        assert_eq!(slot_in_name("plfxzz.png"), None);
        assert_eq!(picture_kind("a/x_stc.png"), Some("stock"));
        assert_eq!(picture_kind("csp.png"), Some("csp"));
        assert_eq!(portrait_frame("plfxgr"), Some(2 + 90));
        assert_eq!(portrait_frame("plcanr"), Some(0));
        assert_eq!(portrait_frame("plnnnr"), None);
        assert_eq!(clean_name(" a/b.c "), "a-b-c");
    }

    #[test]
    fn fobj_constant_keys() {
        // op CON (1), 3 keys (count - 1 = 2 in bits 4..6), u8 values, waits of 1
        let body = [0x21u8, 5, 1, 6, 1, 7, 1];
        let k = fobj_keys(&body, 0, body.len(), 0x80, 0).unwrap();
        assert_eq!(k, vec![(0, 5.0), (1, 6.0), (2, 7.0)]);
    }

    #[test]
    fn mod_zip_and_vault() {
        let root = tmp("root");
        let src = tmp("src");
        let c = costume();
        let p = png1();
        // a single-costume mod ZIP with a manifest and both pictures
        let z = zip(&[("mod.json", br#"{"name": "Tom"}"#), ("PlFxGr.dat", &c), ("stock.png", &p), ("Tom csp.png", &p)]);
        std::fs::write(src.join("m.zip"), z).unwrap();
        let r = import(&root, &src.join("m.zip"), None, None, None, None).unwrap();
        assert!(r[0].contains("PlFxGr.dat 'Tom'"), "{r:?}");
        assert!(root.join("skins/plfxgr/Tom.dat").is_file() && root.join("skins/plfxgr/Tom.stock.png").is_file() && root.join("skins/plfxgr/Tom.csp.png").is_file());
        assert_eq!(crate::cosmetics::list(&root).len(), 1, "pictures are not skins");
        // a vault: one good skin
        let inner = zip(&[("PlFxOr.dat", &c)]);
        let meta = br#"{"characters": {"Fox": {"skins": [{"id": "a1", "color": "Orange Gold", "filename": "fox_or.zip", "costume_code": "PlFxOr", "has_csp": true, "has_stock": true}]}}}"#;
        std::fs::write(src.join("v.zip"), zip(&[("metadata.json", meta), ("Fox/fox_or.zip", &inner), ("Fox/fox_or_csp.png", &p)])).unwrap();
        let r = import(&root, &src.join("v.zip"), None, None, None, None).unwrap();
        assert!(r.iter().any(|l| l.contains("PlFxOr.dat 'Orange Gold' +csp")), "{r:?}");
        assert!(r.iter().any(|l| l.contains("stock picture missing")), "{r:?}");
        // a broken second record makes the whole vault fail and write nothing
        let bad = br#"{"characters": {"Fox": {"skins": [{"id": "b", "color": "X", "filename": "fox_or.zip", "costume_code": "PlFxOr"}, {"id": "c", "color": "Y", "filename": "gone.zip", "costume_code": "PlFxLa"}]}}}"#;
        std::fs::write(src.join("b.zip"), zip(&[("metadata.json", bad), ("fox_or.zip", &inner)])).unwrap();
        assert!(import(&root, &src.join("b.zip"), None, None, None, None).unwrap_err().contains("missing"));
        assert!(!root.join("skins/plfxor/X.dat").exists(), "all or nothing");
        // a metadata target that contradicts the archive is refused
        let lie = br#"{"characters": {"Fox": {"skins": [{"id": "d", "color": "Z", "filename": "fox_or.zip", "costume_code": "PlFxLa"}]}}}"#;
        std::fs::write(src.join("l.zip"), zip(&[("metadata.json", lie), ("fox_or.zip", &inner)])).unwrap();
        assert!(import(&root, &src.join("l.zip"), None, None, None, None).is_err());
        // a portrait on its own, by the code in its name
        std::fs::write(src.join("PlMsNr stock.png"), &p).unwrap();
        import(&root, &src.join("PlMsNr stock.png"), None, None, None, None).unwrap();
        assert!(root.join("portraits/plmsnr/stock.png").is_file());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&src);
    }
}
