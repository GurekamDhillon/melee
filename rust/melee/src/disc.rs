//! Read one file from the user's GameCube disc image (GCM/ISO). The FST reader is `melee_engine::gcm`; this wraps it
//! with the gdmods mounted over the disc. Local only: nothing read here is written anywhere or committed.

pub struct Disc {
    gcm: melee_engine::gcm::Gcm,
    /// gdmods mounted over the disc (`crate::mods`): a mounted file answers before the disc.
    pub mods: Option<crate::mods::Mods>,
    /// Files answered by a mod, in read order (for the run log).
    pub from_mods: Vec<String>,
}

impl Disc {
    pub fn open(path: &str) -> Result<Self, String> {
        Ok(Self { gcm: melee_engine::gcm::Gcm::open(path)?, mods: None, from_mods: vec![] })
    }

    /// `len` bytes of the game's main.dol at its RAM address `addr`. Used for the SIS font atlas.
    pub fn dol_bytes(&mut self, addr: u32, len: usize) -> Result<Vec<u8>, String> {
        self.gcm.dol_bytes(addr, len)
    }

    /// A root-level or nested file by bare name (case-insensitive), as `iso_file` in tests/dat.rs.
    pub fn read(&mut self, want: &str) -> Result<Vec<u8>, String> {
        if let Some(p) = self.mods.as_ref().and_then(|m| m.lookup(want)) {
            let v = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            self.from_mods.push(want.to_string());
            return Ok(v);
        }
        self.read_vanilla(want)
    }

    /// The vanilla disc's own copy, ignoring mounted mods.
    pub fn read_vanilla(&mut self, want: &str) -> Result<Vec<u8>, String> {
        self.gcm.read(want)
    }
}

/// The one disc the Rust engine plays: Super Smash Bros. Melee, NTSC-U, revision 2 ("1.02"), as a plain GCM/ISO.
pub const WANT: &str = "Super Smash Bros. Melee (USA) (Rev 2), NTSC 1.02, as a plain .iso or .gcm";

/// Check a disc image's header before anything boots from it: game id GALE01, disc 1, revision 2, the GameCube
/// magic, and an uncompressed image (the FST reader reads raw sectors only). The message never names the path
/// (it is shown to the player and goes to the log).
pub fn identify(path: &str) -> Result<(), String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| format!("the disc image could not be opened ({})", e.kind()))?;
    let mut h = [0u8; 0x20];
    f.read_exact(&mut h).map_err(|_| "the disc image is empty or cut short".to_string())?;
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    identify_header(&h, len)
}

/// [`identify`] on the first 0x20 bytes and the file's length.
pub fn identify_header(h: &[u8; 0x20], len: u64) -> Result<(), String> {
    let compressed = match &h[..4] {
        b"RVZ\x01" => Some("RVZ"),
        b"WIA\x01" => Some("WIA"),
        b"CISO" => Some("CISO"),
        b"WBFS" => Some("WBFS"),
        [0x01, 0xC0, 0x0B, 0xB1] => Some("GCZ"),
        _ => None,
    };
    if let Some(k) = compressed {
        return Err(format!("this is a compressed {k} disc image: the game needs the plain .iso of {WANT}"));
    }
    if h[0x1C..0x20] != [0xC2, 0x33, 0x9F, 0x3D] {
        return Err(format!("this file is not a GameCube disc image: the game needs {WANT}"));
    }
    let id = &h[..6];
    if &id[..4] != b"GALE" {
        let shown: String = id.iter().map(|&b| if b.is_ascii_alphanumeric() { b as char } else { '?' }).collect();
        return Err(format!("this is a different GameCube game (id {shown}): the game needs {WANT}"));
    }
    if &id[4..6] != b"01" {
        return Err(format!("this disc is not the original Nintendo release (maker {}): the game needs {WANT}", String::from_utf8_lossy(&id[4..6])));
    }
    if h[6] != 0 {
        return Err(format!("this is not disc 1: the game needs {WANT}"));
    }
    match h[7] {
        2 => {}
        r => return Err(format!("this is Melee NTSC 1.0{r} (revision {r}): the game plays only revision 2, {WANT}")),
    }
    // A full image is 1,459,978,240 bytes; a trimmed one still reads (the FST names every file's offset), but one
    // shorter than the FST's start cannot be the game.
    if len != 0 && len < 0x0010_0000 {
        return Err(format!("the disc image is cut short: the game needs a full copy of {WANT}"));
    }
    Ok(())
}

#[cfg(test)]
mod identify_tests {
    use super::*;
    fn hdr(id: &[u8; 6], disc: u8, rev: u8) -> [u8; 0x20] {
        let mut h = [0u8; 0x20];
        h[..6].copy_from_slice(id);
        h[6] = disc;
        h[7] = rev;
        h[0x1C..].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);
        h
    }
    #[test]
    fn header_checks() {
        assert!(identify_header(&hdr(b"GALE01", 0, 2), 1_459_978_240).is_ok());
        assert!(identify_header(&hdr(b"GALE01", 0, 1), 1_459_978_240).unwrap_err().contains("1.01"));
        assert!(identify_header(&hdr(b"GALE01", 0, 0), 1_459_978_240).unwrap_err().contains("1.00"));
        assert!(identify_header(&hdr(b"GALP01", 0, 0), 1).unwrap_err().contains("GALP01"));
        assert!(identify_header(&hdr(b"GALE01", 1, 2), 1_459_978_240).unwrap_err().contains("disc 1"));
        let mut z = [0u8; 0x20];
        z[..4].copy_from_slice(b"RVZ\x01");
        assert!(identify_header(&z, 1).unwrap_err().contains("RVZ"));
        assert!(identify_header(&[0u8; 0x20], 1).unwrap_err().contains("not a GameCube"));
        assert!(identify_header(&hdr(b"GALE01", 0, 2), 4096).unwrap_err().contains("cut short"));
    }
}

/// A message for the player: on stderr (with the bridge on, as the `crash:` line the YouGame app shows when the
/// game stops) and, in a window session on Windows, a message box. `MELEE_NO_DIALOG=1` (tests) skips the box.
pub fn tell_player(msg: &str) {
    eprintln!("play: {msg}");
    if melee_platform::yougame::active() {
        eprintln!("{}", melee_platform::yougame::crash_line(msg));
    }
    #[cfg(windows)]
    if std::env::var_os("MELEE_NO_DIALOG").is_none() && std::env::var_os("MELEE_PAD_SCRIPT").is_none() {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(h: *mut core::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
        }
        let w = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let (t, c) = (w(msg), w("Melee (Rust engine)"));
        unsafe { MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), 0x30) };
    }
}
