//! m-ex disc -> gdmods.
//!
//! A gdmod is what the C build's overlay (`pc/platform/shim_dvd.c` + `gw_mods.c`, layout in
//! `gw_mods.h`) mounts over a vanilla disc: `mods/<id>/mod.json` + `mods/<id>/files/<disc path>`.
//! This module is the Rust port of the workspace's two Python tools, fused:
//!   1. `tools/mex_port/make_mod_from_disc.py`: the delta. A file is packed when it is NEW on the
//!      mod disc or its CONTENT differs from vanilla (same-size pairs are byte-compared).
//!   2. `tools/mex_port/split_modpack.py`: when the delta has `MxDt.dat`, one gdmod per added
//!      fighter and per added stage (claims read from MxDt.dat), plus `<name>-base` with
//!      everything else; a file two entities claim stays in the base.
//! Ids, kinds, file sets, mod.json text and the "hash" digest are the Python tools' exactly
//! (checked by tests/python_parity.rs against their output on the user's discs).

use crate::disc::{same_content, Disc, Loc};
use crate::hsd::{Tables, FIRST_NEW_FIGHTER, FIRST_NEW_STAGE_EXT, FIRST_NEW_STAGE_INT};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    /// Disc paths (mod disc spelling), sorted.
    pub new: Vec<String>,
    pub modified: Vec<String>,
    pub same: usize,
    /// Vanilla paths the mod disc lacks (the overlay cannot delete them).
    pub gone: Vec<String>,
}

pub fn delta(vanilla: &Disc, m: &Disc, mut progress: impl FnMut(usize, usize)) -> io::Result<Delta> {
    let base: HashMap<String, &Loc> = vanilla.files.iter().map(|(k, v)| (k.to_ascii_lowercase(), v)).collect();
    let (mut new, mut modified, mut same) = (vec![], vec![], 0);
    let total = m.files.len();
    for (i, (path, loc)) in m.files.iter().enumerate() {
        progress(i, total);
        match base.get(&path.to_ascii_lowercase()) {
            None => new.push(path.clone()),
            Some(v) if v.size() != loc.size() => modified.push(path.clone()),
            Some(v) => {
                if same_content(m, loc, vanilla, v)? {
                    same += 1
                } else {
                    modified.push(path.clone())
                }
            }
        }
    }
    let have: BTreeSet<String> = m.files.keys().map(|k| k.to_ascii_lowercase()).collect();
    let gone = vanilla.files.keys().filter(|k| !have.contains(&k.to_ascii_lowercase())).cloned().collect();
    Ok(Delta { new, modified, same, gone })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModPlan {
    pub id: String,
    pub kind: &'static str,
    pub name: String,
    /// Disc paths, sorted (Python `sorted()` order: byte order).
    pub files: Vec<String>,
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c);
        } else {
            dash = true;
        }
    }
    if out.is_empty() {
        "unnamed".into()
    } else {
        out
    }
}

/// split_modpack.py resolve: a claimed path -> pack paths. A name with no extension is a prefix.
fn resolve(idx: &BTreeMap<String, String>, path: &str) -> Vec<String> {
    let p = path.trim_start_matches('/').to_lowercase();
    if let Some(r) = idx.get(&p) {
        return vec![r.clone()];
    }
    let basename = p.rsplit('/').next().unwrap_or(&p);
    if !basename.contains('.') {
        let mut c = vec![format!("{p}.dat"), format!("{p}.usd")];
        c.extend((1..10).map(|i| format!("{p}{i}.dat")));
        return c.iter().filter_map(|c| idx.get(c).cloned()).collect();
    }
    vec![]
}

pub struct Split {
    pub mods: Vec<ModPlan>,
    pub shared: usize,
    pub entities: usize,
    pub notes: Vec<String>,
}

/// Port of split_modpack.py main(), over the delta's paths instead of a pack folder.
pub fn split(t: &Tables, pack_files: &[String], name: &str) -> Split {
    let idx: BTreeMap<String, String> = pack_files.iter().map(|p| (p.to_lowercase(), p.clone())).collect();
    let mut notes = vec![];
    let mut entities: Vec<(String, &'static str, String, Vec<String>)> = vec![];
    let ext_of = t.ext_of_internal();
    let n_int = t.meta["internal_id_count"];
    for k in FIRST_NEW_FIGHTER..(n_int - 6) {
        let (fname, claims) = t.fighter_claims(k, ext_of.get(&(k as u8)).copied());
        if claims.is_empty() {
            continue;
        }
        let fname = fname.unwrap();
        let files: BTreeSet<String> = claims.iter().flat_map(|c| resolve(&idx, c)).collect();
        if !files.iter().any(|f| f.eq_ignore_ascii_case(&claims[0])) {
            notes.push(format!("skip fighter {k} {fname}: its {} is not in the pack", claims[0]));
            continue;
        }
        entities.push((format!("{name}-{}", slug(&fname)), "fighter", fname, files.into_iter().collect()));
    }
    let mut seen_ext = BTreeSet::new();
    for ext in t.sss_exts() {
        if ext < FIRST_NEW_STAGE_EXT || !seen_ext.insert(ext) {
            continue;
        }
        let (g, gr) = t.stage_file_for_ext(ext);
        let (Some(g), Some(gr)) = (g, gr) else { continue };
        if g < FIRST_NEW_STAGE_INT {
            continue;
        }
        let files: BTreeSet<String> = resolve(&idx, &gr).into_iter().collect();
        if files.is_empty() {
            notes.push(format!("skip stage ext {ext} {gr}: not in the pack"));
            continue;
        }
        let base = gr.trim_start_matches('/').rsplit('/').next().unwrap().to_string();
        let stem = match base.rfind('.') {
            Some(i) if i > 0 => base[..i].to_string(),
            _ => base.clone(),
        };
        let title = t.stage_name(g).unwrap_or(stem);
        entities.push((format!("{name}-stage-{}", slug(&title)), "stage", title, files.into_iter().collect()));
    }
    let mut seen: HashMap<String, usize> = HashMap::new();
    for e in entities.iter_mut() {
        if let Some(n) = seen.get_mut(&e.0) {
            *n += 1;
            e.0 = format!("{}-{}", e.0, n);
        } else {
            seen.insert(e.0.clone(), 1);
        }
    }
    let mut owners: HashMap<&str, BTreeSet<&str>> = HashMap::new();
    for (eid, _, _, files) in &entities {
        for f in files {
            owners.entry(f.as_str()).or_default().insert(eid.as_str());
        }
    }
    let shared: BTreeSet<String> = owners.iter().filter(|(_, o)| o.len() > 1).map(|(f, _)| f.to_string()).collect();
    let claimed: BTreeSet<String> = owners.keys().filter(|f| !shared.contains(**f)).map(|f| f.to_string()).collect();
    let all: BTreeSet<String> = idx.values().cloned().collect();
    let mut mods = vec![ModPlan {
        id: format!("{name}-base"),
        kind: "base",
        name: format!("{} base", name.to_uppercase()),
        files: all.difference(&claimed).cloned().collect(),
    }];
    let nent = entities.len();
    for (id, kind, ename, files) in entities {
        mods.push(ModPlan { id, kind, name: ename, files: files.into_iter().filter(|f| !shared.contains(f)).collect() });
    }
    Split { mods, shared: shared.len(), entities: nent, notes }
}

// ---- mod.json ------------------------------------------------------------------------------

/// A JSON value written exactly as Python's json.dump(indent=2) writes it (ensure_ascii).
pub enum J {
    S(String),
    I(i64),
    L(Vec<String>),
}

pub fn jstr(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            '\u{8}' => o.push_str("\\b"),
            '\u{c}' => o.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7E => {
                let mut b = [0u16; 2];
                for u in c.encode_utf16(&mut b) {
                    o.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub fn json_object(fields: &[(&str, J)]) -> String {
    let mut o = String::from("{\n");
    for (i, (k, v)) in fields.iter().enumerate() {
        o.push_str("  ");
        o.push_str(&jstr(k));
        o.push_str(": ");
        match v {
            J::S(s) => o.push_str(&jstr(s)),
            J::I(n) => o.push_str(&n.to_string()),
            J::L(l) if l.is_empty() => o.push_str("[]"),
            J::L(l) => {
                o.push_str("[\n");
                for (j, s) in l.iter().enumerate() {
                    o.push_str("    ");
                    o.push_str(&jstr(s));
                    o.push_str(if j + 1 < l.len() { ",\n" } else { "\n" });
                }
                o.push_str("  ]");
            }
        }
        o.push_str(if i + 1 < fields.len() { ",\n" } else { "\n" });
    }
    o.push('}');
    o
}

/// split_modpack.py's mod.json (plus its trailing newline).
pub fn mod_json(m: &ModPlan, pack: &str, version: &str, base_id: Option<&str>, hash: &str) -> String {
    let up = pack.to_uppercase();
    let desc = if m.kind == "base" {
        format!(
            "{up} shared data: m-ex tables (MxDt.dat), menus, common fighter data, shared banks and music. \
             Every {pack} fighter and stage mod requires it."
        )
    } else {
        format!("{} from {up} {version}", m.name).trim().to_string()
    };
    let requires = match (m.kind, base_id) {
        ("base", _) | (_, None) => vec![],
        (_, Some(b)) => vec![b.to_string()],
    };
    json_object(&[
        ("id", J::S(m.id.clone())),
        ("name", J::S(m.name.clone())),
        ("version", J::S(version.to_string())),
        ("kind", J::S(m.kind.to_string())),
        ("pack", J::S(pack.to_string())),
        ("description", J::S(desc)),
        ("requires", J::L(requires)),
        ("conflicts", J::L(vec![])),
        ("hash", J::S(hash.to_string())),
        ("files", J::I(m.files.len() as i64)),
    ]) + "\n"
}

/// split_modpack.py's mod digest: sha256 over "lower(path):sha256(file)\n" in lower-path order, 32 hex.
pub fn mod_hash(files: &[String], digest: &BTreeMap<String, String>) -> String {
    let mut fs: Vec<&String> = files.iter().collect();
    fs.sort_by_key(|f| f.to_lowercase());
    let mut h = crate::sha256::Sha256::new();
    for f in fs {
        h.update(format!("{}:{}\n", f.to_lowercase(), digest[f]).as_bytes());
    }
    h.hex()[..32].to_string()
}

/// Write one gdmod. Files go to `.part` and are renamed into place. Returns bytes written.
pub fn write_mod(out: &Path, m: &ModPlan, src: &Disc, json: &str) -> io::Result<u64> {
    let dir = out.join(&m.id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    let mut bytes = 0;
    for f in &m.files {
        let (_, loc) = src.find(f).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, f.clone()))?;
        let dst = dir.join("files").join(f.replace('/', std::path::MAIN_SEPARATOR_STR));
        std::fs::create_dir_all(dst.parent().unwrap())?;
        let part = dst.with_extension(format!("{}part", dst.extension().map(|e| format!("{}.", e.to_string_lossy())).unwrap_or_default()));
        {
            use std::io::Write;
            let mut w = std::io::BufWriter::new(std::fs::File::create(&part)?);
            src.stream(loc, |c| w.write_all(c))?;
            w.flush()?;
        }
        std::fs::rename(&part, &dst)?;
        bytes += loc.size();
    }
    std::fs::write(dir.join("mod.json"), json)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_like_python() {
        assert_eq!(slug("Luigi's Mansion"), "luigi-s-mansion");
        assert_eq!(slug("  Wolf (SSBU) "), "wolf-ssbu");
        assert_eq!(slug("!!!"), "unnamed");
        assert_eq!(slug("King Dedede"), "king-dedede");
    }

    #[test]
    fn json_like_python() {
        let s = json_object(&[("a", J::S("x\"é".into())), ("b", J::L(vec![])), ("c", J::L(vec!["p".into()])), ("d", J::I(3))]);
        assert_eq!(s, "{\n  \"a\": \"x\\\"\\u00e9\",\n  \"b\": [],\n  \"c\": [\n    \"p\"\n  ],\n  \"d\": 3\n}");
    }

    #[test]
    fn resolve_prefix() {
        let idx: BTreeMap<String, String> =
            ["GrXx.dat", "GrXx1.dat", "grxx.usd", "Other.dat"].iter().map(|p| (p.to_lowercase(), p.to_string())).collect();
        assert_eq!(resolve(&idx, "/GrXx"), ["GrXx.dat", "grxx.usd", "GrXx1.dat"]);
        assert_eq!(resolve(&idx, "/other.dat"), ["Other.dat"]);
        assert!(resolve(&idx, "nope.dat").is_empty());
    }
}

/// Opt-in, on the user's own discs (nothing is written):
/// `GW_ISO_VANILLA=... GW_ISO_AKANEIA=... cargo test -p mex_import -- --ignored`
#[cfg(test)]
mod disc_tests {
    use super::*;

    #[test]
    #[ignore = "needs GW_ISO_VANILLA and GW_ISO_AKANEIA (the user's discs)"]
    fn akaneia_split_partitions_the_delta() {
        let (Ok(v), Ok(m)) = (std::env::var("GW_ISO_VANILLA"), std::env::var("GW_ISO_AKANEIA")) else {
            eprintln!("skipped: GW_ISO_VANILLA / GW_ISO_AKANEIA unset");
            return;
        };
        let v = Disc::open(Path::new(&v)).unwrap();
        let m = Disc::open(Path::new(&m)).unwrap();
        let d = delta(&v, &m, |_, _| {}).unwrap();
        let mut files: Vec<String> = d.new.iter().chain(&d.modified).cloned().collect();
        files.sort();
        let t = Tables::parse(&m.read_path("MxDt.dat").unwrap()).unwrap();
        let s = split(&t, &files, "akaneia");
        // every delta file lands in exactly one gdmod
        let mut all: Vec<&String> = s.mods.iter().flat_map(|p| &p.files).collect();
        all.sort();
        assert_eq!(all.len(), files.len());
        assert!(all.iter().zip(&files).all(|(a, b)| *a == b));
        let sonic = s.mods.iter().find(|p| p.id == "akaneia-sonic").expect("sonic gdmod");
        assert!(sonic.files.iter().any(|f| f == "PlSn.dat"));
        assert!(s.mods[0].files.iter().any(|f| f == "MxDt.dat"), "MxDt.dat stays in the base");
    }
}
