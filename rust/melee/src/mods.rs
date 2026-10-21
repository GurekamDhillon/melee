//! The mods folder (gdmods) mounted over the vanilla disc: the native port of the C build's
//! pc/platform/gw_mods.h resolution, file side only.
//!
//! Layout (`MELEE_MODS_DIR`, or `--mods DIR`): `<dir>/<id>/mod.json` (flat object, every field
//! optional) + `<dir>/<id>/files/<disc path>` (no `files/` = the folder itself is the payload).
//! `enabled.txt` (one id per line, `#` comments) is the enabled set; absent = every mod.
//! `MELEE_MODS=0` mounts nothing. Resolution as gw_mods.h: drop a mod whose `requires` are not
//! all mounting, drop a conflict (an explicit `conflicts` entry either way, or a second `base`),
//! repeat until stable. Mount order: requirements first, then base < misc < fighter/stage, then
//! id; a later mod wins a disc path both provide. `targettest/` is not a mod.
//!
//! What gw_mods.c also does and this does not: zips, skins, scripts, menus, the toggle API.
//! Nothing read here is written anywhere: the mods are the user's disc-derived files.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ModInfo {
    pub id: String,
    pub kind: String,
    pub requires: Vec<String>,
    pub conflicts: Vec<String>,
    root: PathBuf,
}

/// The mounted set and the disc-path overlay it produces.
#[derive(Default)]
pub struct Mods {
    /// Mounted mods, in mount order.
    pub active: Vec<ModInfo>,
    /// Lowercased disc path ("plwf.dat", "audio/us/wolf.ssm") -> file on disk.
    by_path: HashMap<String, PathBuf>,
    /// Lowercased bare file name -> file on disk (Disc::read looks files up by bare name).
    by_name: HashMap<String, PathBuf>,
}

/// A flat JSON object: string values and arrays of strings (other values are skipped).
fn parse_flat_json(s: &str) -> BTreeMap<String, Vec<String>> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut out = BTreeMap::new();
    let ws = |i: &mut usize| {
        while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
            *i += 1
        }
    };
    let string = |i: &mut usize| -> Option<String> {
        if b.get(*i) != Some(&b'"') {
            return None;
        }
        *i += 1;
        let mut v = String::new();
        while *i < b.len() && b[*i] != b'"' {
            if b[*i] == b'\\' && *i + 1 < b.len() {
                *i += 1;
                v.push(match b[*i] {
                    b'n' => '\n',
                    b't' => '\t',
                    c => c as char,
                });
            } else {
                v.push(b[*i] as char);
            }
            *i += 1;
        }
        *i += 1;
        Some(v)
    };
    ws(&mut i);
    if b.get(i) != Some(&b'{') {
        return out;
    }
    i += 1;
    loop {
        ws(&mut i);
        let Some(k) = string(&mut i) else { break };
        ws(&mut i);
        if b.get(i) != Some(&b':') {
            break;
        }
        i += 1;
        ws(&mut i);
        let mut vals = vec![];
        if b.get(i) == Some(&b'[') {
            i += 1;
            loop {
                ws(&mut i);
                if let Some(v) = string(&mut i) {
                    vals.push(v);
                } else {
                    while i < b.len() && b[i] != b',' && b[i] != b']' {
                        i += 1
                    }
                }
                ws(&mut i);
                match b.get(i) {
                    Some(b',') => i += 1,
                    _ => {
                        i += 1;
                        break;
                    }
                }
            }
        } else if let Some(v) = string(&mut i) {
            vals.push(v);
        } else {
            while i < b.len() && b[i] != b',' && b[i] != b'}' {
                i += 1
            }
        }
        out.insert(k, vals);
        ws(&mut i);
        if b.get(i) == Some(&b',') {
            i += 1;
        } else {
            break;
        }
    }
    out
}

fn kind_rank(k: &str) -> u8 {
    match k {
        "base" => 0,
        "misc" | "" => 1,
        _ => 2,
    }
}

impl Mods {
    /// `MELEE_MODS_DIR` (None when unset or `MELEE_MODS=0`).
    pub fn from_env() -> Result<Option<Mods>, String> {
        if std::env::var("MELEE_MODS").is_ok_and(|v| v == "0") {
            return Ok(None);
        }
        match std::env::var_os("MELEE_MODS_DIR") {
            Some(d) => Mods::open(Path::new(&d), None).map(Some),
            None => Ok(None),
        }
    }

    /// Scan `dir`; `enabled` overrides enabled.txt when given.
    pub fn open(dir: &Path, enabled: Option<&[String]>) -> Result<Mods, String> {
        let mut all: BTreeMap<String, ModInfo> = BTreeMap::new();
        let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for e in rd.flatten() {
            let p = e.path();
            let id = e.file_name().to_string_lossy().into_owned();
            if !p.is_dir() || id.eq_ignore_ascii_case("targettest") {
                continue;
            }
            let j = std::fs::read_to_string(p.join("mod.json")).map(|s| parse_flat_json(&s)).unwrap_or_default();
            let one = |k: &str| j.get(k).and_then(|v| v.first().cloned()).unwrap_or_default();
            let root = if p.join("files").is_dir() { p.join("files") } else { p.clone() };
            all.insert(
                id.to_ascii_lowercase(),
                ModInfo {
                    id: id.clone(),
                    kind: one("kind"),
                    requires: j.get("requires").cloned().unwrap_or_default(),
                    conflicts: j.get("conflicts").cloned().unwrap_or_default(),
                    root,
                },
            );
        }
        let enabled: Vec<String> = match enabled {
            Some(v) => v.iter().map(|s| s.to_ascii_lowercase()).collect(),
            None => match std::fs::read_to_string(dir.join("enabled.txt")) {
                Ok(s) => s.lines().map(|l| l.split('#').next().unwrap().trim().to_ascii_lowercase()).filter(|l| !l.is_empty()).collect(),
                Err(_) => all.keys().cloned().collect(),
            },
        };
        let mut keep: Vec<String> = enabled.into_iter().filter(|k| all.contains_key(k)).collect();
        keep.sort();
        keep.dedup();
        // Drop missing deps and conflicts until stable.
        loop {
            let before = keep.len();
            let snap = keep.clone();
            keep.retain(|k| all[k].requires.iter().all(|r| snap.contains(&r.to_ascii_lowercase())));
            let mut kept: Vec<String> = vec![];
            let mut base = false;
            for k in &keep {
                let m = &all[k];
                let clash = kept.iter().any(|o| {
                    let om = &all[o];
                    om.conflicts.iter().any(|c| c.eq_ignore_ascii_case(&m.id)) || m.conflicts.iter().any(|c| c.eq_ignore_ascii_case(&om.id))
                });
                if clash || (m.kind == "base" && base) {
                    continue;
                }
                base |= m.kind == "base";
                kept.push(k.clone());
            }
            keep = kept;
            if keep.len() == before {
                break;
            }
        }
        // Mount order: requirements first, then kind rank, then id.
        let mut order: Vec<String> = vec![];
        let mut sorted = keep.clone();
        sorted.sort_by(|a, b| (kind_rank(&all[a].kind), a).cmp(&(kind_rank(&all[b].kind), b)));
        fn visit(k: &str, all: &BTreeMap<String, ModInfo>, keep: &[String], order: &mut Vec<String>, depth: u32) {
            if order.iter().any(|o| o == k) || depth > 64 {
                return;
            }
            for r in &all[k].requires {
                let r = r.to_ascii_lowercase();
                if keep.contains(&r) {
                    visit(&r, all, keep, order, depth + 1);
                }
            }
            order.push(k.to_string());
        }
        for k in &sorted {
            visit(k, &all, &keep, &mut order, 0);
        }
        let mut m = Mods::default();
        for k in order {
            let info = all[&k].clone();
            let mut stack = vec![info.root.clone()];
            while let Some(d) = stack.pop() {
                let Ok(rd) = std::fs::read_dir(&d) else { continue };
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        if !(d == info.root && e.file_name().eq_ignore_ascii_case("scripts")) {
                            stack.push(p);
                        }
                        continue;
                    }
                    if d == info.root && e.file_name().eq_ignore_ascii_case("mod.json") {
                        continue;
                    }
                    let rel = p.strip_prefix(&info.root).unwrap().to_string_lossy().replace('\\', "/").to_ascii_lowercase();
                    m.by_name.insert(e.file_name().to_string_lossy().to_ascii_lowercase(), p.clone());
                    m.by_path.insert(rel, p);
                }
            }
            m.active.push(info);
        }
        Ok(m)
    }

    /// The mounted file answering disc path `path` ("/PlWf.dat", "audio/us/wolf.ssm") or bare name.
    pub fn lookup(&self, path: &str) -> Option<&Path> {
        let p = path.trim_start_matches('/').replace('\\', "/").to_ascii_lowercase();
        self.by_path.get(&p).or_else(|| if p.contains('/') { None } else { self.by_name.get(&p) }).map(|p| p.as_path())
    }

    pub fn files(&self) -> usize {
        self.by_path.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_json() {
        let j = parse_flat_json(r#"{ "id": "akaneia-wolf", "kind": "fighter", "requires": ["akaneia-base", "x"], "files": 20, "conflicts": [] }"#);
        assert_eq!(j["id"], vec!["akaneia-wolf"]);
        assert_eq!(j["requires"], vec!["akaneia-base", "x"]);
        assert!(j["conflicts"].is_empty());
        assert!(j["files"].is_empty());
    }

    #[test]
    fn resolve_and_overlay() {
        let d = std::env::temp_dir().join(format!("melee-mods-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let mk = |id: &str, json: &str, files: &[(&str, &str)]| {
            let r = d.join(id);
            std::fs::create_dir_all(r.join("files/audio/us")).unwrap();
            std::fs::write(r.join("mod.json"), json).unwrap();
            for (p, c) in files {
                std::fs::write(r.join("files").join(p), c).unwrap();
            }
        };
        mk("p-base", r#"{"kind":"base"}"#, &[("PlCo.dat", "base"), ("PlWf.dat", "base")]);
        mk("p-wolf", r#"{"kind":"fighter","requires":["p-base"]}"#, &[("PlWf.dat", "wolf"), ("audio/us/wolf.ssm", "s")]);
        mk("q-base", r#"{"kind":"base"}"#, &[("PlCo.dat", "q")]);
        mk("orphan", r#"{"kind":"fighter","requires":["nope"]}"#, &[("PlZz.dat", "z")]);
        let m = Mods::open(&d, None).unwrap();
        let ids: Vec<&str> = m.active.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["p-base", "p-wolf"]); // q-base is a second base; orphan misses its dep
        let read = |p: &str| std::fs::read_to_string(m.lookup(p).unwrap()).unwrap();
        assert_eq!(read("/PlWf.dat"), "wolf"); // the later (fighter) mod wins
        assert_eq!(read("plco.dat"), "base");
        assert_eq!(read("audio/us/wolf.ssm"), "s");
        assert_eq!(read("wolf.ssm"), "s");
        assert!(m.lookup("PlZz.dat").is_none());
        let m = Mods::open(&d, Some(&["p-wolf".into()])).unwrap();
        assert!(m.active.is_empty()); // its requirement is not enabled
        let _ = std::fs::remove_dir_all(&d);
    }
}
