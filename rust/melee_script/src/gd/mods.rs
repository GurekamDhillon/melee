//! Read-only mission/mod file access: `gd.mod_read`, `gd.mod_list`, `gd.mod_stamp`
//! (`gw_script_mission.inc`). A path is `missions/...` relative to the calling script's own mod folder; the
//! final canonical path must stay inside that folder, so a junction or symlink cannot escape it.
use super::{reg, TableExt};
use crate::args::Args;
use crate::engine::{fnv, Engine, Shared};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::path::{Path, PathBuf};

const TEXT_MAX: u64 = 1 << 20;

/// `gs_mission_path_valid`
fn path_valid(path: &str) -> bool {
    let b = path.as_bytes();
    let len = b.len();
    if len < 9 || len >= 260 || !path.starts_with("missions/") || path.contains("..") {
        return false;
    }
    let mut start = 0;
    for (i, &c) in b.iter().enumerate() {
        if c < 32 || c == 127 || b"\\:*?\"<>|".contains(&c) {
            return false;
        }
        if c == b'/' {
            if i == start || b[i - 1] == b'.' || b[i - 1] == b' ' {
                return false;
            }
            start = i + 1;
        }
    }
    start == len || (b[len - 1] != b'.' && b[len - 1] != b' ')
}

/// `gs_mission_script_root`: a mounted (active) mod whose folder holds the script's entry wins; the nearest
/// ancestor holding a mod.json is the fallback for standalone manifests.
fn script_root(st: &Shared) -> Option<PathBuf> {
    let s = st.borrow();
    let i = s.cur?;
    if i == s.console {
        return None;
    }
    let sc = s.scripts.get(i).filter(|x| x.used)?;
    let entry = std::fs::canonicalize(&sc.entry).ok()?;
    for id in &s.active_mods {
        if let Ok(root) = std::fs::canonicalize(s.mods_dir.join(id)) {
            if entry.starts_with(&root) {
                return Some(root);
            }
        }
    }
    let mut dir = entry.clone();
    while dir.pop() {
        if dir.join("mod.json").is_file() {
            return Some(dir);
        }
    }
    None
}

/// Open-and-contain: the canonical final path of `root/rel`, which must be inside the canonical root.
fn contained(st: &Shared, rel: &str) -> Result<(PathBuf, PathBuf), &'static str> {
    if !path_valid(rel) {
        return Err("path must start with missions/ and contain no traversal, drive or backslash");
    }
    let Some(root) = script_root(st) else { return Err("calling script has no mod folder") };
    let full = root.join(rel.trim_end_matches('/'));
    let Ok(canon_root) = std::fs::canonicalize(&root) else { return Err("mod folder unavailable") };
    let Ok(canon) = std::fs::canonicalize(&full) else { return Err("mission path unavailable") };
    if !canon.starts_with(&canon_root) || canon == canon_root {
        return Err("mission path escapes the mod folder or is too long");
    }
    Ok((canon_root, canon))
}

fn err2(lua: &Lua, why: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_iter([Value::Nil, Value::String(lua.create_string(why)?)]))
}

/// FILETIME (100 ns since 1601) halves of a modification time, as `GetFileInformationByHandle` reports them.
fn filetime(md: &std::fs::Metadata) -> (u32, u32) {
    let t = md
        .modified()
        .ok()
        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| (d.as_secs() * 10_000_000 + d.subsec_nanos() as u64 / 100) + 116_444_736_000_000_000)
        .unwrap_or(0);
    ((t >> 32) as u32, t as u32)
}

fn stamp(md: &std::fs::Metadata) -> i64 {
    let (hi, lo) = filetime(md);
    let mut h = fnv(14695981039346656037, &(md.len() as u32).to_le_bytes());
    h = fnv(h, &hi.to_le_bytes());
    h = fnv(h, &lo.to_le_bytes());
    (h & 0x7FFF_FFFF_FFFF_FFFF) as i64
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "mod_read", |lua, st, a: Args| {
        let rel = a.check_string(1)?;
        let (_, p) = match contained(st, &rel) {
            Ok(x) => x,
            Err(why) => return err2(lua, why),
        };
        let Ok(md) = std::fs::metadata(&p) else { return err2(lua, "mission path unavailable") };
        if md.is_dir() || md.len() > TEXT_MAX {
            return err2(lua, "mission text must be a file of at most 1 MiB");
        }
        match std::fs::read(&p) {
            Ok(b) if b.len() as u64 == md.len() => Ok(MultiValue::from_iter([Value::String(lua.create_string(&b)?)])),
            _ => err2(lua, "mission file changed or could not be read completely"),
        }
    })?;
    reg(e, gd, "mod_stamp", |_, st, a: Args| {
        let rel = a.check_string(1)?;
        let Ok((_, p)) = contained(st, &rel) else { return Ok(Value::Nil) };
        let Ok(md) = std::fs::metadata(&p) else { return Ok(Value::Nil) };
        if md.is_dir() || md.len() > TEXT_MAX {
            return Ok(Value::Nil);
        }
        Ok(Value::Integer(stamp(&md)))
    })?;
    reg(e, gd, "mod_list", |lua, st, a: Args| {
        let rel = a.check_string(1)?;
        let (_, p) = match contained(st, &rel) {
            Ok(x) => x,
            Err(why) => return err2(lua, why),
        };
        if !Path::new(&p).is_dir() {
            return err2(lua, "mission path is not a directory");
        }
        let Ok(rd) = std::fs::read_dir(&p) else { return err2(lua, "mission directory enumeration failed") };
        let mut rows: Vec<(String, bool)> = rd
            .flatten()
            .map(|d| (d.file_name().to_string_lossy().into_owned(), d.path().is_dir()))
            .collect();
        rows.sort_by(|x, y| x.0.as_bytes().cmp(y.0.as_bytes()));
        let out = lua.create_table()?;
        for (i, (name, dir)) in rows.iter().enumerate() {
            let t = lua.create_table()?;
            t.string("name", name)?;
            t.boolean("dir", *dir)?;
            out.raw_set(i + 1, t)?;
        }
        Ok(MultiValue::from_iter([Value::Table(out)]))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::path_valid;
    #[test]
    fn mission_paths() {
        assert!(path_valid("missions/a/b.txt"));
        assert!(path_valid("missions/dir/"));
        assert!(!path_valid("missions/../x"));
        assert!(!path_valid("data/x.txt"));
        assert!(!path_valid("missions/a\\b"));
        assert!(!path_valid("missions/a//b"));
        assert!(!path_valid("missions/a./b"));
        assert!(!path_valid("missions/x:y"));
        assert!(path_valid("missions/")); // start == len is accepted by the C check
    }
}
