//! Bring a player's Melee save over from the Melee Unlocked engine's memory card (its `User\GC\CardA`
//! folder) into this engine's card (`<base>/USA/Card A`, melee_data::card), so unlocks, records,
//! settings and name tags survive a switch of engines.
//!
//! Both engines keep the card as a folder of GCI files: a 64 byte big-endian directory entry followed
//! by `blocks * 0x2000` bytes of file data. Only the file names differ (his `GALE01-<name>.gci`, ours
//! `01-GALE-<name>.gci`); the bytes of a save are interchangeable. The import copies the one file the
//! game itself keeps everything in, `SuperSmashBros0110290334` (GmSaveData plus the name-tag banks),
//! after checking it the way the game's card library reads it back:
//!
//! - the file data starts with a header: a 0x40 byte comment, the banner and the icons (sizes from the
//!   directory entry's banner/icon formats), then 0x30 bytes holding one 16 byte HSD checksum per
//!   header sector;
//! - the rest of that sector and every later sector is one HSD-encrypted block whose first 16 bytes
//!   are the checksum of its decrypted contents (`hsd_util::decrypt` returns 0 when they match).
//!
//! A sector still erased (all 0xFF, or all zero) is a spare the game never wrote and is skipped; any
//! other block that fails its checksum refuses the import. Our own save is never replaced unless asked
//! (`--force`), and is then first copied to `<Card A>/_import_backup/<unix seconds>/` (a sub-folder, so
//! the card's file scan never sees it as a duplicate save).
//!
//! `melee card-import --from <dir|file.gci> [--to <card base>] [--force] [--dry-run]`;
//! the launcher path calls [`on_launch`].

use std::path::{Path, PathBuf};

/// The game's own save file (lbcardgame `filename`).
pub const SAVE_NAME: &[u8] = b"SuperSmashBros0110290334";
/// The disc this engine runs (`CardSystem::init`'s `CARDInit("GALE", "01")`).
const GAME: &[u8; 4] = b"GALE";
const MAKER: &[u8; 2] = b"01";
const DIR_LEN: usize = 64;
const SECTOR: usize = 0x2000;
const MAX_BLOCKS: usize = 2043;
/// HSD's header digest: one 16 byte checksum per header sector, room for three.
const DIGEST_LEN: usize = 0x30;
const BACKUP_DIR: &str = "_import_backup";

#[derive(Debug)]
pub enum ImportError {
    /// No source folder/file, or no Melee save in it.
    Missing(String),
    /// A Melee save that fails its structure or checksum checks.
    Corrupt(String),
    /// Reading or writing failed.
    Io(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::Missing(m) => write!(f, "nothing to import: {m}"),
            ImportError::Corrupt(m) => write!(f, "refused, the save is damaged: {m}"),
            ImportError::Io(m) => write!(f, "failed: {m}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Imported { from: PathBuf, to: PathBuf, backup: Option<PathBuf> },
    /// Ours already has a save and `force` was not given: nothing written.
    Kept { existing: PathBuf },
    /// `dry_run`: the source checked out; this is where it would go.
    WouldImport { from: PathBuf, to: PathBuf, replaces: Option<PathBuf> },
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    /// His card folder (`...\User\GC\CardA`), a Dolphin-style base with `USA/Card A`, or one `.gci` file.
    pub from: PathBuf,
    /// Our card base (MELEE_CARD_PATH), or its `Card A` folder itself.
    pub to: PathBuf,
    pub force: bool,
    pub dry_run: bool,
}

/// What `verify` found in a save.
#[derive(Debug, PartialEq, Eq)]
pub struct Checked {
    pub blocks: usize,
    pub header_size: usize,
    pub data_blocks_ok: usize,
    pub spare_sectors: usize,
}

fn be16(b: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([b[o], b[o + 1]])
}

fn cname(b: &[u8]) -> &[u8] {
    &b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())]
}

/// The directory entry names Melee's save (any region).
fn is_save_name(dir: &[u8]) -> bool {
    dir.len() >= DIR_LEN && &dir[0..3] == b"GAL" && cname(&dir[8..40]) == SAVE_NAME
}

/// The directory entry is this engine's (GALE01) Melee save.
pub fn is_our_save(dir: &[u8]) -> bool {
    is_save_name(dir) && &dir[0..4] == GAME && &dir[4..6] == MAKER
}

/// Bytes of comment + banner + icons before the header digest (HSD's `hsd_803AC340` arithmetic, from
/// the directory entry: banner format at 0x07, icon formats at 0x30 and speeds at 0x32, two bits per
/// frame; the frames end at the first zero speed; CI8 icons share one 0x200 byte palette).
pub fn header_size(dir: &[u8]) -> usize {
    let banner = match dir[7] & 3 {
        2 => 0x1800,
        1 => 0xC00 + 0x200,
        _ => 0,
    };
    let (fmt, speed) = (be16(dir, 0x30), be16(dir, 0x32));
    let (mut icons, mut ci8) = (0, false);
    for i in 0..8 {
        if (speed >> (2 * i)) & 3 == 0 {
            break;
        }
        match (fmt >> (2 * i)) & 3 {
            2 => icons += 0x800,
            1 => {
                icons += 0x400;
                ci8 = true;
            }
            _ => {}
        }
    }
    0x40 + banner + icons + if ci8 { 0x200 } else { 0 }
}

/// GCI structure: a whole directory entry and exactly `blocks` sectors after it.
fn check_structure(bytes: &[u8]) -> Result<usize, ImportError> {
    if bytes.len() < DIR_LEN {
        return Err(ImportError::Corrupt(format!("{} bytes, shorter than a GCI directory entry", bytes.len())));
    }
    let blocks = be16(bytes, 0x38) as usize;
    if blocks == 0 || blocks > MAX_BLOCKS {
        return Err(ImportError::Corrupt(format!("block count {blocks} out of range")));
    }
    let want = DIR_LEN + blocks * SECTOR;
    if bytes.len() != want {
        return Err(ImportError::Corrupt(format!("{} bytes on disk, the directory entry says {blocks} blocks ({want} bytes)", bytes.len())));
    }
    Ok(blocks)
}

fn erased(b: &[u8]) -> bool {
    b.iter().all(|&x| x == 0xFF) || b.iter().all(|&x| x == 0)
}

/// Check a whole `.gci` (directory entry + data) the way the game's card library reads it.
pub fn verify(bytes: &[u8]) -> Result<Checked, ImportError> {
    let blocks = check_structure(bytes)?;
    let (dir, data) = bytes.split_at(DIR_LEN);
    let h = header_size(dir);
    let header_sectors = h.div_ceil(SECTOR);
    if header_sectors * 16 > DIGEST_LEN || h + DIGEST_LEN > data.len() {
        return Err(ImportError::Corrupt(format!("header of {h:#x} bytes does not fit the file")));
    }
    for i in 0..header_sectors {
        let sum = melee_runtime::hsd_util::checksum(&data[i * SECTOR..((i + 1) * SECTOR).min(h)]);
        if sum[..] != data[h + 16 * i..h + 16 * (i + 1)] {
            return Err(ImportError::Corrupt(format!("banner/icon header sector {i} fails its checksum")));
        }
    }
    let start = h + DIGEST_LEN;
    let first_end = (start / SECTOR + 1) * SECTOR;
    let mut units = vec![start..first_end.min(data.len())];
    units.extend((first_end..data.len()).step_by(SECTOR).map(|s| s..s + SECTOR));
    let (mut ok, mut spare) = (0, 0);
    for u in units {
        if u.len() <= 16 {
            continue;
        }
        if erased(&data[u.clone()]) {
            spare += 1;
            continue;
        }
        let mut b = data[u.clone()].to_vec();
        if melee_runtime::hsd_util::decrypt(&mut b) != 0 {
            return Err(ImportError::Corrupt(format!("data block at file offset {:#x} fails its checksum", DIR_LEN + u.start)));
        }
        ok += 1;
    }
    if ok == 0 {
        return Err(ImportError::Corrupt("no written data block (a save that was created but never saved)".into()));
    }
    Ok(Checked { blocks, header_size: h, data_blocks_ok: ok, spare_sectors: spare })
}

fn read_dir_entry(path: &Path) -> Option<[u8; DIR_LEN]> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut b = [0u8; DIR_LEN];
    f.read_exact(&mut b).ok()?;
    Some(b)
}

fn gci_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("gci")))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// The Melee save in `from`: the file itself, or the one in the folder (or its `USA/Card A`).
pub fn find_source(from: &Path) -> Result<PathBuf, ImportError> {
    if from.is_file() {
        return Ok(from.to_path_buf());
    }
    if !from.is_dir() {
        return Err(ImportError::Missing(format!("{} does not exist", from.display())));
    }
    let mut files = gci_files(from);
    if files.is_empty() {
        files = gci_files(&from.join("USA").join("Card A"));
    }
    let mut other_region = None;
    let mut found = Vec::new();
    for f in files {
        let Some(d) = read_dir_entry(&f) else { continue };
        if is_our_save(&d) {
            found.push(f);
        } else if is_save_name(&d) {
            other_region = Some(String::from_utf8_lossy(&d[0..6]).into_owned());
        }
    }
    if found.len() > 1 {
        eprintln!("card-import: {} Melee saves in {}; taking {}", found.len(), from.display(), found[0].display());
    }
    match (found.into_iter().next(), other_region) {
        (Some(f), _) => Ok(f),
        (None, Some(r)) => Err(ImportError::Missing(format!("the Melee save in {} is for {r}; this engine runs GALE01", from.display()))),
        (None, None) => Err(ImportError::Missing(format!("no Melee save ({}) in {}", String::from_utf8_lossy(SAVE_NAME), from.display()))),
    }
}

/// Our slot A folder for a card base (`<base>/USA/Card A`), or the base itself when it already is one.
pub fn card_folder(base: &Path) -> PathBuf {
    if base.file_name().is_some_and(|n| n == "Card A") {
        base.to_path_buf()
    } else {
        base.join("USA").join("Card A")
    }
}

/// Our existing Melee save files in a slot folder.
pub fn existing_saves(folder: &Path) -> Vec<PathBuf> {
    gci_files(folder).into_iter().filter(|p| read_dir_entry(p).is_some_and(|d| is_our_save(&d))).collect()
}

/// The disk name our card gives the save (melee_data::card `create_file`: `<maker>-<game>-<name>.gci`).
fn our_file_name() -> String {
    format!("{}-{}-{}.gci", "01", "GALE", String::from_utf8_lossy(SAVE_NAME))
}

fn io(what: &str, p: &Path, e: std::io::Error) -> ImportError {
    ImportError::Io(format!("{what} {}: {e}", p.display()))
}

/// Import Melee's save from `o.from` into the card at `o.to` (see the module notes).
pub fn import(o: &Options) -> Result<Outcome, ImportError> {
    let src = find_source(&o.from)?;
    let bytes = std::fs::read(&src).map_err(|e| io("cannot read", &src, e))?;
    if !is_our_save(&bytes[..bytes.len().min(DIR_LEN)]) {
        return Err(ImportError::Missing(format!("{} is not a GALE01 Melee save", src.display())));
    }
    let c = verify(&bytes)?;
    eprintln!(
        "card-import: {} checks out: {} blocks, header {:#x}, {} data blocks verified, {} spare sectors",
        src.display(), c.blocks, c.header_size, c.data_blocks_ok, c.spare_sectors
    );
    let folder = card_folder(&o.to);
    let existing = existing_saves(&folder);
    let target = folder.join(our_file_name());
    if let Some(e) = existing.first() {
        if !o.force {
            return Ok(Outcome::Kept { existing: e.clone() });
        }
    }
    if o.dry_run {
        return Ok(Outcome::WouldImport { from: src, to: target, replaces: existing.first().cloned() });
    }
    std::fs::create_dir_all(&folder).map_err(|e| io("cannot create", &folder, e))?;
    let mut backup = None;
    if !existing.is_empty() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let dir = folder.join(BACKUP_DIR).join(stamp.to_string());
        std::fs::create_dir_all(&dir).map_err(|e| io("cannot create", &dir, e))?;
        for e in &existing {
            let to = dir.join(e.file_name().unwrap_or_default());
            std::fs::copy(e, &to).map_err(|err| io("cannot back up", e, err))?;
        }
        backup = Some(dir);
    }
    let mut tmp = target.as_os_str().to_owned();
    tmp.push(".tmp-import");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, &bytes).map_err(|e| io("cannot write", &tmp, e))?;
    if let Err(e) = std::fs::rename(&tmp, &target) {
        let _ = std::fs::remove_file(&tmp);
        return Err(io("cannot write", &target, e));
    }
    // an old save under another disk name would now be a second copy, which the card refuses (BROKEN)
    for e in existing.iter().filter(|e| **e != target) {
        std::fs::remove_file(e).map_err(|err| io("cannot remove the replaced", e, err))?;
    }
    Ok(Outcome::Imported { from: src, to: target, backup })
}

/// One line for the log about an import's result.
pub fn describe(r: &Result<Outcome, ImportError>) -> String {
    match r {
        Ok(Outcome::Imported { from, to, backup: None }) => format!("card-import: imported {} -> {}", from.display(), to.display()),
        Ok(Outcome::Imported { from, to, backup: Some(b) }) => {
            format!("card-import: imported {} -> {} (the save it replaced is in {})", from.display(), to.display(), b.display())
        }
        Ok(Outcome::Kept { existing }) => format!("card-import: not imported, this card already has a save ({}); --force replaces it", existing.display()),
        Ok(Outcome::WouldImport { from, to, replaces }) => format!(
            "card-import: dry run: would import {} -> {}{}",
            from.display(),
            to.display(),
            replaces.as_ref().map(|r| format!(" (replacing {}, backed up first)", r.display())).unwrap_or_default()
        ),
        Err(e) => format!("card-import: {e}"),
    }
}

/// The launcher path: his card folder (`--card-dir`) into our card base, only while ours has no save
/// yet. `MELEE_CARD_IMPORT=0` turns it off. Never fails the launch; returns the line for the log.
pub fn on_launch(his_card: &Path, our_base: &Path) -> String {
    if std::env::var("MELEE_CARD_IMPORT").is_ok_and(|v| v.starts_with('0')) {
        return "card-import: off (MELEE_CARD_IMPORT=0)".into();
    }
    if let Some(e) = existing_saves(&card_folder(our_base)).first() {
        return format!("card-import: not needed, this card already has a save ({})", e.display());
    }
    if !his_card.exists() {
        return format!("card-import: no card at {} to import", his_card.display());
    }
    describe(&import(&Options { from: his_card.to_path_buf(), to: our_base.to_path_buf(), force: false, dry_run: false }))
}

/// `melee card-import --from <dir|file.gci> [--to <card base>] [--force] [--dry-run]`
pub fn cli(args: &[String]) -> std::process::ExitCode {
    use std::process::ExitCode;
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let usage = "usage: melee card-import --from <card folder | file.gci> [--to <card base>] [--force] [--dry-run]";
    let Some(from) = get("--from") else {
        eprintln!("{usage}");
        return ExitCode::from(2);
    };
    let to = match get("--to").map(PathBuf::from).or_else(melee_engine::card::configured_base) {
        Some(t) => t,
        None => {
            eprintln!("card-import: the card is off (MELEE_CARD=0); pass --to");
            return ExitCode::from(2);
        }
    };
    let o = Options { from: from.into(), to, force: args.iter().any(|a| a == "--force"), dry_run: args.iter().any(|a| a == "--dry-run") };
    let r = import(&o);
    eprintln!("{}", describe(&r));
    if r.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic save shaped like the game's: CI8 banner + one CI8 icon, the header digest, then
    /// encrypted blocks of made-up contents; `spare` trailing sectors left erased.
    fn synthetic_save(blocks: usize, spare: usize, seed: u8) -> Vec<u8> {
        let mut dir = [0u8; DIR_LEN];
        dir[0..4].copy_from_slice(GAME);
        dir[4..6].copy_from_slice(MAKER);
        dir[6] = 0xFF;
        dir[7] = 1; // CI8 banner
        dir[8..8 + SAVE_NAME.len()].copy_from_slice(SAVE_NAME);
        dir[0x2C..0x30].copy_from_slice(&0x40u32.to_be_bytes());
        dir[0x30..0x32].copy_from_slice(&1u16.to_be_bytes()); // icon 0: CI8
        dir[0x32..0x34].copy_from_slice(&3u16.to_be_bytes()); // icon 0 speed, then the end
        dir[0x34] = 4;
        dir[0x38..0x3A].copy_from_slice(&(blocks as u16).to_be_bytes());
        let mut data = vec![0xFFu8; blocks * SECTOR];
        let h = header_size(&dir);
        assert_eq!(h, 0x40 + 0xE00 + 0x400 + 0x200);
        for (i, b) in data[..h].iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(seed);
        }
        let sum = melee_runtime::hsd_util::checksum(&data[..h]);
        data[h..h + 16].copy_from_slice(&sum);
        data[h + 16..h + DIGEST_LEN].fill(0);
        let written = blocks - spare;
        let mut units = vec![h + DIGEST_LEN..SECTOR];
        units.extend((1..written).map(|s| s * SECTOR..(s + 1) * SECTOR));
        for (n, u) in units.into_iter().enumerate() {
            let b = &mut data[u];
            b[16] = 0;
            b[17] = n as u8; // block id
            b[18] = 1; // seq
            for (i, x) in b[0x20..].iter_mut().enumerate() {
                *x = (i as u8) ^ seed ^ (n as u8);
            }
            melee_runtime::hsd_util::encrypt(b);
        }
        let mut out = dir.to_vec();
        out.extend_from_slice(&data);
        out
    }

    struct Tmp(PathBuf);
    impl Tmp {
        fn new(tag: &str) -> Tmp {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let p = std::env::temp_dir().join(format!("melee-card-import-{}-{tag}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Tmp(p)
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// His card folder: flat, `GALE01-<name>.gci` naming, plus an unrelated game's file.
    fn his_card(t: &Tmp, save: &[u8]) -> PathBuf {
        let d = t.0.join("User").join("GC").join("CardA");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("GALE01-SuperSmashBros0110290334.gci"), save).unwrap();
        let mut other = synthetic_save(2, 0, 9);
        other[0..4].copy_from_slice(b"GXXE");
        std::fs::write(d.join("GXXE01-Other.gci"), other).unwrap();
        d
    }

    fn opts(from: &Path, to: &Path, force: bool) -> Options {
        Options { from: from.into(), to: to.into(), force, dry_run: false }
    }

    #[test]
    fn synthetic_save_verifies() {
        let s = synthetic_save(4, 1, 3);
        let c = verify(&s).unwrap();
        assert_eq!(c, Checked { blocks: 4, header_size: 0x1440, data_blocks_ok: 3, spare_sectors: 1 });
    }

    #[test]
    fn valid_import_lands_in_our_card() {
        let t = Tmp::new("valid");
        let save = synthetic_save(4, 0, 1);
        let from = his_card(&t, &save);
        let base = t.0.join("User").join("GC").join("RustPort");
        let r = import(&opts(&from, &base, false)).unwrap();
        let target = base.join("USA").join("Card A").join("01-GALE-SuperSmashBros0110290334.gci");
        assert_eq!(r, Outcome::Imported { from: from.join("GALE01-SuperSmashBros0110290334.gci"), to: target.clone(), backup: None });
        assert_eq!(std::fs::read(&target).unwrap(), save);
        // only the Melee save came over
        assert_eq!(gci_files(&card_folder(&base)).len(), 1);
        // and our card opens it by name
        let mut card = melee_data::card::CardSystem::with_system_clock();
        card.init(&base);
        assert_eq!(card.mount(0), melee_data::card::res::READY);
        let mut info = melee_data::card::FileInfo::default();
        assert_eq!(card.open(0, SAVE_NAME, &mut info), melee_data::card::res::READY);
        let mut buf = vec![0u8; SECTOR];
        assert_eq!(card.read(&info, &mut buf, SECTOR as i32, SECTOR as i32), melee_data::card::res::READY);
        assert_eq!(buf[..], save[DIR_LEN + SECTOR..DIR_LEN + 2 * SECTOR]);
    }

    #[test]
    fn missing_source_is_reported() {
        let t = Tmp::new("missing");
        let base = t.0.join("ours");
        match import(&opts(&t.0.join("nope"), &base, false)) {
            Err(ImportError::Missing(_)) => {}
            r => panic!("{r:?}"),
        }
        // a folder with no Melee save in it
        let empty = t.0.join("CardA");
        std::fs::create_dir_all(&empty).unwrap();
        match import(&opts(&empty, &base, false)) {
            Err(ImportError::Missing(_)) => {}
            r => panic!("{r:?}"),
        }
        assert!(!base.exists());
        assert!(on_launch(&t.0.join("nope"), &base).contains("no card"));
    }

    #[test]
    fn corrupt_checksum_is_refused() {
        let t = Tmp::new("corrupt");
        let base = t.0.join("ours");
        // a flipped byte in a data block
        let mut save = synthetic_save(4, 0, 2);
        save[DIR_LEN + 2 * SECTOR + 0x100] ^= 0x40;
        let from = his_card(&t, &save);
        match import(&opts(&from, &base, true)) {
            Err(ImportError::Corrupt(m)) => assert!(m.contains("checksum"), "{m}"),
            r => panic!("{r:?}"),
        }
        // a flipped byte in the banner (header digest)
        let mut save = synthetic_save(4, 0, 2);
        save[DIR_LEN + 0x80] ^= 1;
        std::fs::write(from.join("GALE01-SuperSmashBros0110290334.gci"), &save).unwrap();
        match import(&opts(&from, &base, true)) {
            Err(ImportError::Corrupt(m)) => assert!(m.contains("header"), "{m}"),
            r => panic!("{r:?}"),
        }
        // a truncated file
        std::fs::write(from.join("GALE01-SuperSmashBros0110290334.gci"), &synthetic_save(4, 0, 2)[..DIR_LEN + SECTOR]).unwrap();
        assert!(matches!(import(&opts(&from, &base, true)), Err(ImportError::Corrupt(_))));
        assert!(existing_saves(&card_folder(&base)).is_empty());
    }

    #[test]
    fn existing_save_is_kept_unless_forced() {
        let t = Tmp::new("existing");
        let theirs = synthetic_save(4, 0, 5);
        let from = his_card(&t, &theirs);
        let base = t.0.join("ours");
        let folder = card_folder(&base);
        std::fs::create_dir_all(&folder).unwrap();
        // ours, under a disk name of its own (a create that found the name taken)
        let ours = synthetic_save(4, 0, 6);
        let ours_path = folder.join("01-GALE-SuperSmashBros0110290334-1.gci");
        std::fs::write(&ours_path, &ours).unwrap();

        assert_eq!(import(&opts(&from, &base, false)).unwrap(), Outcome::Kept { existing: ours_path.clone() });
        assert!(on_launch(&from, &base).contains("already has a save"));
        let dry = import(&Options { dry_run: true, ..opts(&from, &base, true) }).unwrap();
        assert!(matches!(dry, Outcome::WouldImport { replaces: Some(_), .. }));
        assert_eq!(std::fs::read(&ours_path).unwrap(), ours);
        assert_eq!(gci_files(&folder).len(), 1);

        let Outcome::Imported { to, backup: Some(b), .. } = import(&opts(&from, &base, true)).unwrap() else { panic!() };
        assert_eq!(std::fs::read(&to).unwrap(), theirs);
        assert_eq!(std::fs::read(b.join("01-GALE-SuperSmashBros0110290334-1.gci")).unwrap(), ours);
        // exactly one save on the card afterwards (a second would make the card BROKEN)
        assert_eq!(existing_saves(&folder), vec![to]);
        let mut card = melee_data::card::CardSystem::with_system_clock();
        card.init(&base);
        assert_eq!(card.mount(0), melee_data::card::res::READY);
    }
}
