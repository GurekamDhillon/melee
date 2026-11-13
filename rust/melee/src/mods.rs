//! The mods folder (gdmods) mounted over the vanilla disc.
//!
//! The resolution and the overlay are `melee_data::modreg`, the port of the C build's
//! pc/platform/gw_mods.c + shim_dvd.c (checked against the shipping C by `mods_oracle`): the
//! `<dir>/<id>/mod.json` metadata, `enabled.txt` (absent = every mod), requirement / conflict /
//! base resolution, mount order, engine-requirement gating, and the disc-path overlay in which a
//! later mod wins a path. This file adapts it to `Disc::read`, which answers by disc path or by
//! bare file name. `MELEE_MODS=0` mounts nothing.
//!
//! Nothing read here is written anywhere: the mods are the user's disc-derived files.

use melee_data::modreg::{ModSet, Overlay};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ModInfo {
    pub id: String,
    pub kind: String,
}

/// The mounted set and the disc-path overlay it produces.
#[derive(Default)]
pub struct Mods {
    /// Mounted mods, in mount order.
    pub active: Vec<ModInfo>,
    /// The resolved set (every installed mod, its status, the fingerprint).
    pub set: ModSet,
    overlay: Overlay,
    /// Lowercased bare file name -> file on disk (Disc::read looks files up by bare name).
    by_name: HashMap<String, PathBuf>,
    /// The mods folder.
    pub dir: PathBuf,
}

impl Mods {
    /// `MELEE_MODS_DIR` or `mods_dir=` in settings.cfg (None when unset or `MELEE_MODS=0`).
    pub fn from_env() -> Result<Option<Mods>, String> {
        if !crate::cfg::flag("mods", true) {
            return Ok(None);
        }
        match crate::cfg::text("mods_dir") {
            Some(d) => Mods::open(Path::new(&d), None).map(Some),
            None => Ok(None),
        }
    }

    /// Scan `dir`; `enabled` overrides enabled.txt when given.
    pub fn open(dir: &Path, enabled: Option<&[String]>) -> Result<Mods, String> {
        std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut set = ModSet::load_with(dir, false, enabled);
        let overlay = Overlay::mount(&mut set);
        let active = set.order.iter().map(|&i| ModInfo { id: set.mods[i].id_str(), kind: set.mods[i].kind_str() }).collect();
        let mut by_name = HashMap::new();
        for f in &overlay.files {
            let name = f.disc.rsplit('/').next().unwrap_or(&f.disc).to_ascii_lowercase();
            by_name.insert(name, f.host.clone());
        }
        Ok(Mods { active, set, overlay, by_name, dir: dir.to_path_buf() })
    }

    /// The mounted file answering disc path `path` ("/PlWf.dat", "audio/us/wolf.ssm") or bare name.
    pub fn lookup(&self, path: &str) -> Option<&Path> {
        if let Some(f) = self.overlay.lookup(path) {
            return Some(f.host.as_path());
        }
        let p = path.trim_start_matches(['/', '\\']);
        if p.contains(['/', '\\']) {
            return None;
        }
        self.by_name.get(&p.to_ascii_lowercase()).map(|p| p.as_path())
    }

    pub fn files(&self) -> usize {
        self.overlay.files.len()
    }

    /// TARGET_PC (notes/anim-overrides.md): every mounted mod's `anim_overrides`, in mount order (a later mod wins), each
    /// file read and validated now. Any problem is an error naming the mod and the entry or file: the boot stops before a
    /// match instead of failing inside one.
    pub fn anim_overrides(&self) -> Result<Vec<melee_fighter::anim_override::Entry>, String> {
        use melee_data::modreg::AnimClip;
        use melee_fighter::anim_override as ao;
        let mut out = Vec::new();
        for &i in &self.set.order {
            let m = &self.set.mods[i];
            let id = m.id_str();
            if let Some(e) = m.anim_override_errors.first() {
                return Err(format!("mod {id}: mod.json {e}"));
            }
            for d in &m.anim_overrides {
                let kind = ao::kind_by_name(&d.fighter).ok_or_else(|| format!("mod {id}: anim_overrides: unknown fighter \"{}\"", d.fighter))?;
                let clip = match &d.clip {
                    AnimClip::Symbol(s) => ao::Clip::Symbol(s.as_bytes().to_vec()),
                    AnimClip::Row(r) if *r >= 0 => ao::Clip::Row(*r),
                    AnimClip::Row(r) => return Err(format!("mod {id}: anim_overrides: bad row {r}")),
                };
                let rel = std::path::Path::new(&d.file);
                if rel.is_absolute() || rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                    return Err(format!("mod {id}: anim_overrides: \"{}\" must be a path inside the mod folder", d.file));
                }
                out.push(ao::load(kind, clip, self.dir.join(&id).join(rel), format!("{id}/{}", d.file))?);
            }
        }
        Ok(out)
    }

    /// TARGET_PC (notes/anim-overrides.md): every mounted mod's `script_overrides` (replacement action scripts), in mount
    /// order (a later mod wins), each file read and validated now; an error names the mod and file.
    pub fn script_overrides(&self) -> Result<Vec<melee_fighter::script_override::Entry>, String> {
        use melee_data::modreg::AnimClip;
        use melee_fighter::{anim_override as ao, script_override as so};
        let mut out = Vec::new();
        for &i in &self.set.order {
            let m = &self.set.mods[i];
            let id = m.id_str();
            if let Some(e) = m.script_override_errors.first() {
                return Err(format!("mod {id}: mod.json {e}"));
            }
            for d in &m.script_overrides {
                let kind = ao::kind_by_name(&d.fighter).ok_or_else(|| format!("mod {id}: script_overrides: unknown fighter \"{}\" ({})", d.fighter, d.file))?;
                let clip = match &d.clip {
                    AnimClip::Symbol(s) => ao::Clip::Symbol(s.as_bytes().to_vec()),
                    AnimClip::Row(r) if *r >= 0 => ao::Clip::Row(*r),
                    AnimClip::Row(r) => return Err(format!("mod {id}: script_overrides: bad row {r} ({})", d.file)),
                };
                let rel = std::path::Path::new(&d.file);
                if rel.is_absolute() || rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
                    return Err(format!("mod {id}: script_overrides: \"{}\" must be a path inside the mod folder", d.file));
                }
                out.push(so::load(kind, clip, self.dir.join(&id).join(rel), format!("{id}/{}", d.file))?);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
