//! TARGET_PC (Rust only): per-clip animation overrides (docs/rust-port/notes/anim-overrides.md).
//!
//! A mod's `mod.json` maps (fighter, clip) to a loose file holding ONE standalone FigaTree archive. The table is filled
//! before the engine boots (`install`, from `melee::mods`); `ftdata::load_figatree` consults it and copies the override's
//! bytes into the fighter's animation buffer instead of the bank slice. With no entry installed `active()` is false and
//! `load_figatree` runs exactly as before (one test of a static).
//!
//! Rules:
//! - Later entries win: entries are installed in mod mount order, file order inside a mod, and looked up from the end.
//! - `Symbol(name)` matches every motion row (of that fighter kind's table) whose FigaTree symbol is `name`, and the
//!   demo / result rows with that symbol; `Row(n)` matches only row `n` of the fighter's own motion table.
//! - The file's public symbol: if the archive has exactly one public symbol it is used whatever its name; with several,
//!   the one named like the clip's symbol (Symbol entries only) is used; anything else is an error at load.
//! - Nana: when she plays Popo's rows (outside demos, as retail does) the rows are Popo's, so `popo` entries apply to
//!   her; `nana` entries apply only to rows of Nana's own table.
//! - Size: a file over `FT_ANIM_BUF_SIZE` (the fixed per-fighter buffer) is rejected at load with both numbers.

use crate::types::FT_ANIM_BUF_SIZE;
use std::path::PathBuf;

/// Which motion rows an entry replaces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clip {
    /// Every row whose FigaTree symbol is this name.
    Symbol(Vec<u8>),
    /// One row of the fighter's own motion table.
    Row(i32),
}

/// One installed override.
#[derive(Clone, Debug)]
pub struct Entry {
    /// FighterKind (internal kind) whose rows it replaces.
    pub kind: i32,
    pub clip: Clip,
    /// The file on disk (re-read by `reload`).
    pub path: PathBuf,
    /// `<mod id>/<file>` for messages.
    pub label: String,
    /// The archive.
    pub bytes: Vec<u8>,
    /// The public symbol to bind, NUL-terminated.
    pub public: Vec<u8>,
}

static mut TABLE: Vec<Entry> = Vec::new();
static mut ACTIVE: bool = false;
static mut GEN: u32 = 0;

/// True when at least one override is installed (the only test `load_figatree` makes without one).
#[inline]
pub fn active() -> bool {
    unsafe { ACTIVE }
}

/// How many times the table was installed or reloaded (a reload's observers compare it).
pub fn generation() -> u32 {
    unsafe { GEN }
}

/// Replace the whole table (mount order: later entries win). An empty vector turns overrides off.
pub fn install(v: Vec<Entry>) {
    unsafe {
        let t = &mut *core::ptr::addr_of_mut!(TABLE);
        *t = v;
        ACTIVE = !t.is_empty();
        GEN = GEN.wrapping_add(1);
    }
}

/// The installed entries (read only).
pub fn entries() -> &'static [Entry] {
    unsafe { &*core::ptr::addr_of!(TABLE) }
}

/// Re-read every installed entry's file from disk and validate it; all or nothing (on any error the old data stays and
/// the error names the mod and file). Answers how many entries changed bytes. Allocation: each changed entry's buffer
/// is replaced (the old one is freed), so repeated reloads do not grow memory.
pub fn reload() -> Result<usize, String> {
    let t = unsafe { &mut *core::ptr::addr_of_mut!(TABLE) };
    let mut next: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(t.len());
    for e in t.iter() {
        let bytes = std::fs::read(&e.path).map_err(|er| format!("anim override {}: cannot read {}: {er}", e.label, e.path.display()))?;
        let public = check(&bytes, &e.clip).map_err(|m| format!("anim override {}: {m}", e.label))?;
        next.push((bytes, public));
    }
    let mut changed = 0;
    for (e, (b, p)) in t.iter_mut().zip(next) {
        if e.bytes != b || e.public != p {
            changed += 1;
            e.bytes = b;
            e.public = p;
        }
    }
    unsafe { GEN = GEN.wrapping_add(1) };
    Ok(changed)
}

/// Bytes held by the table's buffers (the reload test's memory measure).
pub fn held_bytes() -> usize {
    entries().iter().map(|e| e.bytes.capacity() + e.public.capacity()).sum()
}

/// Build one entry from a file: read, validate (size, archive structure, public symbol), resolve the symbol.
pub fn load(kind: i32, clip: Clip, path: PathBuf, label: String) -> Result<Entry, String> {
    let bytes = std::fs::read(&path).map_err(|er| format!("anim override {label}: cannot read {}: {er}", path.display()))?;
    let public = check(&bytes, &clip).map_err(|m| format!("anim override {label}: {m}"))?;
    Ok(Entry { kind, clip, path, label, bytes, public })
}

/// The HSD archive's public symbols: (data offset, name).
pub fn publics(b: &[u8]) -> Result<Vec<(u32, Vec<u8>)>, String> {
    let be = |o: usize| -> u32 { u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) };
    if b.len() < 0x20 {
        return Err(format!("{} bytes is too short for an HSD archive (32-byte header)", b.len()));
    }
    let (file_size, data_size, nb_reloc, nb_public, nb_extern) = (be(0), be(4), be(8), be(12), be(16));
    if file_size as usize != b.len() {
        return Err(format!("not an HSD archive: the header says {file_size} bytes, the file has {}", b.len()));
    }
    let data = 0x20usize;
    let reloc = data.checked_add(data_size as usize);
    let public = reloc.and_then(|r| r.checked_add(nb_reloc as usize * 4));
    let ext = public.and_then(|p| p.checked_add(nb_public as usize * 8));
    let strings = ext.and_then(|e| e.checked_add(nb_extern as usize * 8));
    let (Some(reloc), Some(public), Some(strings)) = (reloc, public, strings) else {
        return Err("not an HSD archive: table sizes overflow".into());
    };
    if strings > b.len() {
        return Err(format!("not an HSD archive: its tables need {strings} bytes, the file has {}", b.len()));
    }
    if nb_public == 0 {
        return Err("the archive has no public symbol".into());
    }
    // every relocation must point inside the data and hold an offset inside it (HSD_ArchiveParse trusts them)
    for i in 0..nb_reloc as usize {
        let at = be(reloc + 4 * i) as usize;
        if at + 4 > data_size as usize || be(data + at) > data_size {
            return Err(format!("relocation {i} points outside the archive's data"));
        }
    }
    let mut out = Vec::new();
    for i in 0..nb_public as usize {
        let (off, s) = (be(public + 8 * i), be(public + 8 * i + 4) as usize);
        let at = strings.checked_add(s).filter(|&a| a < b.len()).ok_or_else(|| format!("public symbol {i}'s name is outside the file"))?;
        let end = b[at..].iter().position(|&c| c == 0).ok_or_else(|| format!("public symbol {i}'s name is not terminated"))?;
        out.push((off, b[at..at + end].to_vec()));
    }
    Ok(out)
}

/// Validate an override file for `clip`; answers the public symbol to bind (NUL-terminated).
pub fn check(b: &[u8], clip: &Clip) -> Result<Vec<u8>, String> {
    let limit = FT_ANIM_BUF_SIZE as usize;
    if b.len() > limit {
        return Err(format!(
            "{} bytes (0x{:X}) is over the fighter animation buffer limit of {} bytes (0x{:X}); the clip must be smaller",
            b.len(),
            b.len(),
            limit,
            limit
        ));
    }
    let pubs = publics(b)?;
    let pick = if pubs.len() == 1 {
        &pubs[0]
    } else {
        let want = match clip {
            Clip::Symbol(s) => Some(s.as_slice()),
            Clip::Row(_) => None,
        };
        let names: Vec<String> = pubs.iter().map(|p| String::from_utf8_lossy(&p.1).into_owned()).collect();
        match want.and_then(|w| pubs.iter().find(|p| p.1 == w)) {
            Some(p) => p,
            None => return Err(format!("the archive has {} public symbols ({}) and none is the clip's: keep one", pubs.len(), names.join(", "))),
        }
    };
    let data_size = u32::from_be_bytes([b[4], b[5], b[6], b[7]]);
    if pick.0.checked_add(0x14).is_none_or(|e| e > data_size) {
        return Err(format!("public symbol {} does not point at a FigaTree inside the data", String::from_utf8_lossy(&pick.1)));
    }
    let mut p = pick.1.clone();
    p.push(0);
    Ok(p)
}

/// The entry that replaces a motion row, if any. `owner` is the FighterKind whose motion table holds the row and
/// `row` its index there (None for rows of another table: the demo / result rows), `sym` the row's FigaTree symbol.
pub fn find(owner: i32, row: Option<i32>, sym: Option<&[u8]>) -> Option<&'static Entry> {
    entries().iter().rev().find(|e| {
        e.kind == owner
            && match &e.clip {
                Clip::Row(r) => row == Some(*r),
                Clip::Symbol(s) => sym == Some(s.as_slice()),
            }
    })
}

/// The fighter names a manifest may use (FighterKind; the scenario names plus the internal ones).
pub fn kind_by_name(name: &str) -> Option<i32> {
    use crate::types::*;
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "mario" => Ft_Kind_Mario,
        "fox" => Ft_Kind_Fox,
        "falcon" | "captain" => Ft_Kind_Captain,
        "dk" | "donkey" => Ft_Kind_Donkey,
        "kirby" => Ft_Kind_Kirby,
        "bowser" | "koopa" => Ft_Kind_Koopa,
        "link" => Ft_Kind_Link,
        "sheik" | "seak" => Ft_Kind_Seak,
        "ness" => Ft_Kind_Ness,
        "peach" => Ft_Kind_Peach,
        "popo" | "ics" | "iceclimbers" => Ft_Kind_Popo,
        "nana" => Ft_Kind_Nana,
        "pikachu" => Ft_Kind_Pikachu,
        "samus" => Ft_Kind_Samus,
        "yoshi" => Ft_Kind_Yoshi,
        "jigglypuff" | "purin" => Ft_Kind_Purin,
        "mewtwo" => Ft_Kind_Mewtwo,
        "luigi" => Ft_Kind_Luigi,
        "marth" | "mars" => Ft_Kind_Mars,
        "zelda" => Ft_Kind_Zelda,
        "younglink" | "clink" => Ft_Kind_CLink,
        "drmario" => Ft_Kind_DrMario,
        "falco" => Ft_Kind_Falco,
        "pichu" => Ft_Kind_Pichu,
        "gw" | "gameandwatch" | "gamewatch" => Ft_Kind_GameWatch,
        "ganon" | "ganondorf" => Ft_Kind_Ganon,
        "roy" | "emblem" => Ft_Kind_Emblem,
        _ => return None,
    } as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic standalone archive: one FigaTree-sized blob, two relocated pointer words, the given public names.
    pub fn archive(names: &[&str], pad: usize) -> Vec<u8> {
        let data_size = 0x20u32 + pad as u32;
        let relocs = [0xCu32, 0x10];
        let mut data = vec![0u8; data_size as usize];
        data[0xC..0x10].copy_from_slice(&0x14u32.to_be_bytes());
        data[0x10..0x14].copy_from_slice(&0x18u32.to_be_bytes());
        let mut strings = Vec::new();
        let mut pubs = Vec::new();
        for n in names {
            pubs.push((0u32, strings.len() as u32));
            strings.extend_from_slice(n.as_bytes());
            strings.push(0);
        }
        let total = 0x20 + data.len() + relocs.len() * 4 + pubs.len() * 8 + strings.len();
        let mut b = Vec::new();
        for w in [total as u32, data_size, relocs.len() as u32, pubs.len() as u32, 0, 0, 0, 0] {
            b.extend_from_slice(&w.to_be_bytes());
        }
        b.extend_from_slice(&data);
        for r in relocs {
            b.extend_from_slice(&r.to_be_bytes());
        }
        for (o, s) in pubs {
            b.extend_from_slice(&o.to_be_bytes());
            b.extend_from_slice(&s.to_be_bytes());
        }
        b.extend_from_slice(&strings);
        b
    }

    /// The table is process-global: the tests that install serialize on this.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    const SYM: &[u8] = b"PlyMars5K_Share_ACTION_AttackS4_figatree";

    #[test]
    fn symbol_rule() {
        // one public: used whatever its name
        let a = archive(&["Renamed_figatree"], 0);
        assert_eq!(check(&a, &Clip::Symbol(SYM.to_vec())).unwrap(), b"Renamed_figatree\0");
        assert_eq!(check(&a, &Clip::Row(62)).unwrap(), b"Renamed_figatree\0");
        // several: the clip's own name, else an error naming them
        let a = archive(&["X_figatree", "PlyMars5K_Share_ACTION_AttackS4_figatree"], 0);
        assert_eq!(check(&a, &Clip::Symbol(SYM.to_vec())).unwrap(), [SYM, b"\0"].concat());
        let e = check(&a, &Clip::Row(62)).unwrap_err();
        assert!(e.contains("2 public symbols") && e.contains("X_figatree"), "{e}");
    }

    #[test]
    fn rejects_bad_files() {
        assert!(check(b"short", &Clip::Row(0)).unwrap_err().contains("too short"));
        let mut a = archive(&["A"], 0);
        a.push(0);
        assert!(check(&a, &Clip::Row(0)).unwrap_err().contains("header says"));
        let a = archive(&[], 0);
        assert!(check(&a, &Clip::Row(0)).unwrap_err().contains("no public symbol"));
        let mut a = archive(&["A"], 0);
        // relocation 0 now points past the data
        let r = 0x20 + 0x20;
        a[r..r + 4].copy_from_slice(&0x100u32.to_be_bytes());
        assert!(check(&a, &Clip::Row(0)).unwrap_err().contains("relocation 0"));
    }

    #[test]
    fn over_the_buffer_is_rejected_with_size_and_limit() {
        let a = archive(&["A"], FT_ANIM_BUF_SIZE as usize);
        let e = check(&a, &Clip::Row(0)).unwrap_err();
        assert!(e.contains(&format!("{} bytes", a.len())) && e.contains("131072 bytes (0x20000)"), "{e}");
        // exactly at the limit is fine
        let pad = FT_ANIM_BUF_SIZE as usize - archive(&["A"], 0).len();
        let a = archive(&["A"], pad);
        assert_eq!(a.len(), FT_ANIM_BUF_SIZE as usize);
        assert!(check(&a, &Clip::Row(0)).is_ok());
    }

    #[test]
    fn find_later_wins_and_row_vs_symbol() {
        let _g = LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let e = |kind, clip, label: &str| Entry { kind, clip, path: PathBuf::new(), label: label.into(), bytes: vec![], public: vec![] };
        let mars = Ft_Kind_Mars_i();
        install(vec![
            e(mars, Clip::Symbol(SYM.to_vec()), "a"),
            e(mars, Clip::Symbol(SYM.to_vec()), "b"),
            e(mars, Clip::Row(5), "row5"),
        ]);
        assert!(active());
        assert_eq!(find(mars, Some(62), Some(SYM)).unwrap().label, "b"); // the later mod wins
        assert_eq!(find(mars, Some(63), Some(SYM)).unwrap().label, "b"); // every row sharing the clip follows it
        assert_eq!(find(mars, None, Some(SYM)).unwrap().label, "b"); // a demo row with the symbol too
        assert_eq!(find(mars, Some(5), Some(b"other")).unwrap().label, "row5");
        assert!(find(mars, Some(6), Some(b"other")).is_none()); // a row entry affects only its row
        assert!(find(mars, None, Some(b"other")).is_none());
        assert!(find(mars + 1, Some(62), Some(SYM)).is_none());
        install(Vec::new());
        assert!(!active());
    }

    #[test]
    fn reload_replaces_and_does_not_grow() {
        let _g = LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let d = std::env::temp_dir().join(format!("anim-override-test-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("clip.dat");
        std::fs::write(&f, archive(&["A"], 0x40)).unwrap();
        let e = load(Ft_Kind_Mars_i(), Clip::Row(1), f.clone(), "m/clip.dat".into()).unwrap();
        install(vec![e]);
        assert_eq!(reload().unwrap(), 0);
        std::fs::write(&f, archive(&["B"], 0x40)).unwrap();
        assert_eq!(reload().unwrap(), 1);
        assert_eq!(entries()[0].public, b"B\0");
        // a broken file keeps the old data and names the file
        std::fs::write(&f, b"junk").unwrap();
        let er = reload().unwrap_err();
        assert!(er.contains("m/clip.dat") && er.contains("too short"), "{er}");
        assert_eq!(entries()[0].public, b"B\0");
        // 100 reloads alternating two files: the held bytes stay flat
        let (a1, a2) = (archive(&["A"], 0x40), archive(&["C"], 0x40));
        std::fs::write(&f, &a1).unwrap();
        reload().unwrap();
        let held = held_bytes();
        for i in 0..100 {
            std::fs::write(&f, if i % 2 == 0 { &a2 } else { &a1 }).unwrap();
            assert_eq!(reload().unwrap(), 1);
            assert!(held_bytes() <= held, "reload {i}: {} > {held}", held_bytes());
        }
        install(Vec::new());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[allow(non_snake_case)]
    fn Ft_Kind_Mars_i() -> i32 {
        crate::types::Ft_Kind_Mars as i32
    }

    #[test]
    fn names() {
        assert_eq!(kind_by_name("Marth"), Some(crate::types::Ft_Kind_Mars as i32));
        assert_eq!(kind_by_name("nana"), Some(crate::types::Ft_Kind_Nana as i32));
        assert_eq!(kind_by_name("ics"), Some(crate::types::Ft_Kind_Popo as i32));
        assert!(kind_by_name("wolf").is_none());
    }
}
