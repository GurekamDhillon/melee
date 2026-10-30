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
