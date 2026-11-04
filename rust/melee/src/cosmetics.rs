//! Cosmetics: costume skins (a replacement `Pl<Xx><Yy>.dat` for one costume slot) picked per slot and mounted over the
//! disc, after the gdmods (a skin wins its slot).
//!
//! * Catalog: `cosmetics_dir=` in settings.cfg / `MELEE_COSMETICS_DIR`, else `Cosmetics/` beside the executable.
//!   `Cosmetics/skins/<slot>/<name>.dat`, `<slot>` the lowercased costume file stem (`plfxnr`). `melee cosmetics
//!   import FILE [--slot PlFxNr.dat] [--name N]` copies a file in; `list`, `select SLOT NAME|none`, `check`.
//! * Choice: `skin_<slot>=<name>` in settings.cfg (`skin_plfxnr=green`), written by the settings overlay or `select`.
//! * Sim safety (the rule: no setting may change a hash). A costume file is a model, but the fighter's skeleton comes
//!   from it: the bones drive the ECB, the hurt bubbles and every bone-relative hitbox. So a skin is mounted only when
//!   its `*_Share_joint` tree equals the vanilla costume's: same shape, same transform-relevant flags, bit-identical
//!   rest pose (rotation, scale, translation). Meshes, textures and material / shape animations may differ freely. A
//!   skin that fails is never mounted (offline or online), and the reason is logged and shown in the overlay. With
//!   that rule a skin is a local choice: netplay peers need not agree on skins (nothing of it enters the handshake).
//! * Live: offline (no rollback session), a changed choice applies to the next match (the slot's cached file is
//!   dropped from the disc and the overlay asked again). Under a rollback session the choices are fixed at boot.
//!
//! Reference (behaviour only, nothing copied): Hero's melee-unlocked `port/runtime/host/cosmetic_mods.cpp` (catalog,
//! per-slot choice, "joints match / rest pose differs" online verdict). Its archive / vault importers are not here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The catalog directory.
pub fn dir() -> PathBuf {
    if let Some(d) = crate::cfg::text("cosmetics_dir") {
        return PathBuf::from(d);
    }
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("Cosmetics"))).unwrap_or_else(|| PathBuf::from("Cosmetics"))
}

/// `PlFxNr.dat` -> `plfxnr`; None for a name that is not a costume slot (`Pl` + 2-letter fighter + 2-letter colour).
pub fn slot_of(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let (stem, ext) = base.rsplit_once('.')?;
    if !ext.eq_ignore_ascii_case("dat") || stem.len() != 6 || !stem[..2].eq_ignore_ascii_case("pl") || !stem.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some(stem.to_ascii_lowercase())
}

/// The disc file name of a slot (`plfxnr` -> `PlFxNr.dat`).
pub fn slot_file(slot: &str) -> String {
    let b = slot.as_bytes();
    let up = |c: u8| (c as char).to_ascii_uppercase();
    let lo = |c: u8| (c as char).to_ascii_lowercase();
    if b.len() != 6 {
        return format!("{slot}.dat");
    }
    format!("Pl{}{}{}{}.dat", up(b[2]), lo(b[3]), up(b[4]), lo(b[5]))
}

fn be32(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4).map(|s| u32::from_be_bytes(s.try_into().unwrap()))
}

/// One joint of a `*_Share_joint` tree: (depth, flags that matter to the sim, rotation / scale / translation bits).
type Joint = (u16, u32, [u32; 9]);

/// HSD_JObj flags that only change how the model draws (hidden, particles, lighting, texgen, specular, OPA / XLU /
/// TEXEDGE and their ROOT_ forms). Everything else (skeleton / envelope / billboard / quaternion / matrix
/// independence) shapes the bone matrices and must match.
const RENDER_FLAGS: u32 = 1 << 4 | 1 << 5 | 1 << 7 | 1 << 8 | 1 << 16 | 1 << 18 | 1 << 19 | 1 << 20 | 1 << 28 | 1 << 29 | 1 << 30;

/// The skeleton of a costume DAT (an HSD archive): its `*_Share_joint` root walked depth first.
pub fn skeleton(dat: &[u8]) -> Result<Vec<Joint>, String> {
    let file = be32(dat, 0).ok_or("too short for a DAT header")? as usize;
    let data = be32(dat, 4).ok_or("bad header")? as usize;
    let nrel = be32(dat, 8).ok_or("bad header")? as usize;
    let nroot = be32(dat, 12).ok_or("bad header")? as usize;
    let nref = be32(dat, 16).ok_or("bad header")? as usize;
    if file != dat.len() || 0x20 + data > dat.len() {
        return Err(format!("not an HSD archive (header size {file:#x}, file {:#x})", dat.len()));
    }
    let roots = 0x20 + data + nrel * 4;
    let strings = roots + (nroot + nref) * 8;
    if strings > dat.len() {
        return Err("root table past the end".into());
    }
    let body = &dat[0x20..0x20 + data];
    let mut top = None;
    for i in 0..nroot {
        let off = be32(dat, roots + i * 8).unwrap() as usize;
        let no = be32(dat, roots + i * 8 + 4).unwrap() as usize;
        let s = dat.get(strings + no..).ok_or("root name past the end")?;
        let name = &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())];
        if name.ends_with(b"_Share_joint") {
            top = Some(off);
            break;
        }
    }
    let top = top.ok_or("no *_Share_joint root (not a costume file)")?;
    let mut out = Vec::new();
    // (offset, depth), explicit stack: child first, then next
    let mut stack = vec![(top, 0u16)];
    while let Some((o, d)) = stack.pop() {
        if out.len() > 4096 {
            return Err("joint tree too large or cyclic".into());
        }
        if o + 0x40 > body.len() {
            return Err(format!("joint at {o:#x} past the data"));
        }
        let w = |k: usize| be32(body, o + k).unwrap();
        let mut srt = [0u32; 9];
        for (k, v) in srt.iter_mut().enumerate() {
            *v = w(0x14 + k * 4);
        }
        out.push((d, w(4) & !RENDER_FLAGS, srt));
        let (child, next) = (w(8) as usize, w(0xC) as usize);
        if next != 0 {
            stack.push((next, d));
        }
        if child != 0 {
            stack.push((child, d + 1));
        }
    }
    Ok(out)
}

/// The archive's root symbol names, sorted (the game finds the costume's parts by these exact names).
pub fn root_names(dat: &[u8]) -> Vec<String> {
    let (Some(data), Some(nrel), Some(nroot), Some(nref)) = (be32(dat, 4), be32(dat, 8), be32(dat, 12), be32(dat, 16)) else { return vec![] };
    let roots = 0x20 + data as usize + nrel as usize * 4;
    let strings = roots + (nroot + nref) as usize * 8;
    let mut v: Vec<String> = (0..nroot as usize)
        .filter_map(|i| {
            let no = be32(dat, roots + i * 8 + 4)? as usize;
            let s = dat.get(strings + no..)?;
            Some(String::from_utf8_lossy(&s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())]).into_owned())
        })
        .collect();
    v.sort();
    v
}

/// Whether `skin` may replace `vanilla`: Ok(joints) when the skeletons are identical, else why not.
pub fn check(vanilla: &[u8], skin: &[u8]) -> Result<usize, String> {
    let a = skeleton(vanilla).map_err(|e| format!("vanilla: {e}"))?;
    let b = skeleton(skin)?;
    let (ra, rb) = (root_names(vanilla), root_names(skin));
    if ra != rb {
        let miss = ra.iter().find(|n| !rb.contains(n)).cloned().unwrap_or_default();
        return Err(format!("its symbols differ from the costume's (missing {miss}): a skin for another slot?"));
    }
    if a.len() != b.len() {
        return Err(format!("{} joints, the costume has {}", b.len(), a.len()));
    }
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        if x.0 != y.0 {
            return Err(format!("joint {i}: tree shape differs"));
        }
        if x.1 != y.1 {
            return Err(format!("joint {i}: flags {:#x} vs {:#x}", y.1, x.1));
        }
        if x.2 != y.2 {
            return Err(format!("joint {i}: rest pose differs"));
        }
    }
    Ok(a.len())
}

#[derive(Clone, Debug)]
pub struct Skin {
    pub slot: String,
    pub name: String,
    pub path: PathBuf,
}

/// Every skin in the catalog, by slot then name.
pub fn list(root: &Path) -> Vec<Skin> {
    let mut v = Vec::new();
    let Ok(rd) = std::fs::read_dir(root.join("skins")) else { return v };
    for e in rd.flatten() {
        let slot = e.file_name().to_string_lossy().to_ascii_lowercase();
        if slot_of(&format!("{slot}.dat")).is_none() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(e.path()) else { continue };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dat")) {
                let name = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                v.push(Skin { slot: slot.clone(), name, path: p });
            }
        }
    }
    v.sort_by(|a, b| (&a.slot, a.name.to_ascii_lowercase()).cmp(&(&b.slot, b.name.to_ascii_lowercase())));
    v
}

/// The chosen skin name per slot, from settings.cfg (`skin_<slot>=name`).
pub fn chosen(skins: &[Skin]) -> HashMap<String, String> {
    let mut s = crate::cfg::open();
    let mut m = HashMap::new();
    let mut slots: Vec<&str> = skins.iter().map(|k| k.slot.as_str()).collect();
    slots.dedup();
    for slot in slots {
        let v = s.str_or(&format!("skin_{slot}"), "");
        if !v.is_empty() && v != "none" {
            m.insert(slot.to_string(), v);
        }
    }
    m
}

/// The live set: slot -> the validated skin's bytes. Shared by the disc overlay and the settings overlay.
#[derive(Default)]
pub struct Mounted {
    pub files: HashMap<String, Arc<[u8]>>,
    /// Per slot: the verdict of the last check (Ok(joints) or the refusal), for the overlay and the log.
    pub verdict: HashMap<String, Result<usize, String>>,
}

pub type Shared = Arc<Mutex<Mounted>>;

/// Validate `skin` against the disc's costume and mount it (or unmount the slot with None). Returns the verdict.
pub fn mount(sh: &Shared, vanilla: &mut dyn FnMut(&str) -> Option<Vec<u8>>, slot: &str, skin: Option<&Path>) -> Result<usize, String> {
    let mut m = sh.lock().unwrap();
    let Some(p) = skin else {
        m.files.remove(slot);
        m.verdict.remove(slot);
        return Ok(0);
    };
    let r = (|| {
        let b = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        let v = vanilla(&slot_file(slot)).ok_or_else(|| format!("{} is not on the disc", slot_file(slot)))?;
        let n = check(&v, &b)?;
        Ok((n, b))
    })();
    let verdict = match r {
        Ok((n, b)) => {
            m.files.insert(slot.to_string(), b.into());
            Ok(n)
        }
        Err(e) => {
            m.files.remove(slot);
            Err(e)
        }
    };
    m.verdict.insert(slot.to_string(), verdict.clone());
    verdict
}

/// Boot: the catalog's chosen skins, validated against the disc at `iso` (read once each). None when nothing is chosen.
pub fn boot(iso: &str) -> Option<Shared> {
    let root = dir();
    let skins = list(&root);
    let pick = chosen(&skins);
    if pick.is_empty() {
        return None;
    }
    let sh: Shared = Default::default();
    let mut disc = melee_engine::gcm::Gcm::open(iso).ok()?;
    for (slot, name) in &pick {
        let Some(s) = skins.iter().find(|s| &s.slot == slot && s.name.eq_ignore_ascii_case(name)) else {
            eprintln!("cosmetics: {}: skin '{name}' is not in the catalog", slot_file(slot));
            continue;
        };
        match mount(&sh, &mut |f| disc.read(f).ok(), slot, Some(&s.path)) {
            Ok(n) => eprintln!("cosmetics: {} <- '{}' ({n} joints match)", slot_file(slot), s.name),
            Err(e) => eprintln!("cosmetics: {} '{}' refused: {e} (the costume stays vanilla)", slot_file(slot), s.name),
        }
    }
    Some(sh)
}

/// The disc overlay answering the mounted slots (layered over the gdmods' overlay, if any).
pub fn overlay(sh: Shared, under: Option<melee_engine::gcm::Overlay>) -> melee_engine::gcm::Overlay {
    Box::new(move |path: &str| {
        if let Some(slot) = slot_of(path) {
            if let Some(b) = sh.lock().unwrap().files.get(&slot) {
                return Some(b.to_vec());
            }
        }
        under.as_ref().and_then(|u| u(path))
    })
}

/// The skins mounted at this boot (the overlay's live set), for the settings overlay.
static BOOTED: std::sync::OnceLock<Mutex<Option<Shared>>> = std::sync::OnceLock::new();

pub fn set_booted(sh: Option<Shared>) {
    *BOOTED.get_or_init(Default::default).lock().unwrap() = sh;
}

pub fn booted() -> Option<Shared> {
    BOOTED.get().and_then(|m| m.lock().unwrap().clone())
}

/// `melee cosmetics list|import|select|check`.
pub fn cli(args: &[String]) -> std::process::ExitCode {
    use std::process::ExitCode;
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let root = dir();
    let fail = |e: String| {
        eprintln!("cosmetics: {e}");
        ExitCode::FAILURE
    };
    match args.first().map(String::as_str) {
        Some("list") | None => {
            let skins = list(&root);
            let pick = chosen(&skins);
            println!("catalog {} ({} skins)", root.display(), skins.len());
            for s in &skins {
                let on = pick.get(&s.slot).is_some_and(|n| n.eq_ignore_ascii_case(&s.name));
                println!("  {} {:<12} {}", if on { "*" } else { " " }, slot_file(&s.slot), s.name);
            }
            ExitCode::SUCCESS
        }
        Some("import") => {
            let Some(src) = args.get(1) else { return fail("import FILE [--slot PlXxYy.dat] [--name NAME]".into()) };
            let src = PathBuf::from(src);
            let slot = match get("--slot").as_deref().map(slot_of).unwrap_or_else(|| slot_of(&src.to_string_lossy())) {
                Some(s) => s,
                None => return fail("cannot tell the costume slot from the file name: pass --slot PlXxYy.dat".into()),
            };
            let name = get("--name").unwrap_or_else(|| src.file_stem().unwrap_or_default().to_string_lossy().to_string());
            if name.is_empty() || name.contains(['/', '\\', '.', '=']) {
                return fail(format!("bad skin name '{name}'"));
            }
            let b = match std::fs::read(&src) {
                Ok(b) => b,
                Err(e) => return fail(format!("{}: {e}", src.display())),
            };
            if let Err(e) = skeleton(&b) {
                return fail(format!("{}: {e}", src.display()));
            }
            let d = root.join("skins").join(&slot);
            let to = d.join(format!("{name}.dat"));
            if let Err(e) = std::fs::create_dir_all(&d).and_then(|_| std::fs::write(&to, &b)) {
                return fail(format!("{}: {e}", to.display()));
            }
            println!("imported {} as {} '{name}'", src.display(), slot_file(&slot));
            ExitCode::SUCCESS
        }
        Some("select") => {
            let (Some(slot), Some(name)) = (args.get(1).and_then(|s| slot_of(s).or_else(|| slot_of(&format!("{s}.dat")))), args.get(2)) else {
                return fail("select PlXxYy.dat NAME|none".into());
            };
            if name != "none" && !list(&root).iter().any(|s| s.slot == slot && s.name.eq_ignore_ascii_case(name)) {
                return fail(format!("no skin '{name}' for {}", slot_file(&slot)));
            }
            let mut s = crate::cfg::open();
            s.set_str(&format!("skin_{slot}"), name);
            println!("{} -> {name}", slot_file(&slot));
            ExitCode::SUCCESS
        }
        Some("check") => {
            let iso = match crate::iso_path(args) {
                Ok(i) => i,
                Err(e) => return fail(e),
            };
            let mut disc = match melee_engine::gcm::Gcm::open(&iso) {
                Ok(d) => d,
                Err(_) => return fail("cannot open the disc image".into()),
            };
            let mut bad = 0;
            for s in list(&root) {
                let r = std::fs::read(&s.path).map_err(|e| e.to_string()).and_then(|b| {
                    let v = disc.read(&slot_file(&s.slot)).map_err(|_| format!("{} is not on the disc", slot_file(&s.slot)))?;
                    check(&v, &b)
                });
                match r {
                    Ok(n) => println!("  ok   {:<12} {} ({n} joints match)", slot_file(&s.slot), s.name),
                    Err(e) => {
                        bad += 1;
                        println!("  REFUSED {:<12} {}: {e}", slot_file(&s.slot), s.name)
                    }
                }
            }
            if bad > 0 { ExitCode::from(2) } else { ExitCode::SUCCESS }
        }
        Some(o) => fail(format!("unknown command '{o}' (list, import, select, check)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal HSD archive: one `_Share_joint` root over `joints` (depth-first: (child?, srt)).
    fn dat(trans: f32, three: bool) -> Vec<u8> {
        let mut body = Vec::new();
        let n = if three { 3 } else { 2 };
        for i in 0..n {
            let mut j = [0u8; 0x40];
            let child = if i + 1 < n { (i + 1) * 0x40 } else { 0 };
            j[8..12].copy_from_slice(&(child as u32).to_be_bytes());
            j[4..8].copy_from_slice(&(1u32 | 1 << 18).to_be_bytes());
            j[0x2C..0x30].copy_from_slice(&(trans * i as f32).to_bits().to_be_bytes());
            body.extend_from_slice(&j);
        }
        let name = b"PlyTest5K_Share_joint\0";
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

    #[test]
    fn skeleton_rule() {
        let v = dat(1.0, true);
        assert_eq!(check(&v, &v), Ok(3));
        // a render-only flag difference is allowed
        let mut x = v.clone();
        x[0x20 + 5] |= 0x08; // bit 19 (XLU)
        assert_eq!(check(&v, &x), Ok(3));
        assert!(check(&v, &dat(1.5, true)).unwrap_err().contains("rest pose"));
        assert!(check(&v, &dat(1.0, false)).unwrap_err().contains("joints"));
        assert!(skeleton(&v[..0x30]).is_err());
    }

    #[test]
    fn slot_names() {
        assert_eq!(slot_of("PlFxNr.dat").as_deref(), Some("plfxnr"));
        assert_eq!(slot_of("/x/PLCAGR.DAT").as_deref(), Some("plcagr"));
        assert_eq!(slot_of("PlFx.dat"), None);
        assert_eq!(slot_of("GrNBa.dat"), None);
        assert_eq!(slot_file("plfxnr"), "PlFxNr.dat");
    }
}
