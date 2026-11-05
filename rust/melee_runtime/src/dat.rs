//! HSD archive (`.dat`) reader: a big-endian relocated GameCube file -> native structs.
//!
//! Port of `HSD_ArchiveParse` / `HSD_ArchiveGetPublicAddress` / `HSD_ArchiveGetExtern`
//! (src/sysdolphin/baselib/archive.c). The C keeps the file bytes and patches each relocated
//! word in place (`Locate`: `*ptr += data`), so the game reads big-endian structs through real
//! pointers. Natively the file stays untouched; a root is decoded on demand into native
//! little-endian structs (`melee_types`, through `melee_core::gcdecode`), with pointers that
//! point at other decoded objects. The GC offset of every field comes from the generated
//! `GcLayout` tables, so the native layout never has to match the file.
//!
//! File layout (all big-endian u32): header (0x20: file_size, data_size, nb_reloc, nb_public,
//! nb_extern, version[4], pad[2]), data block, nb_reloc relocation offsets (data-relative
//! offsets of pointer words; each word holds a data-relative target), nb_public
//! (offset, symbol) roots, nb_extern (offset, symbol) external references, symbol strings.

use melee_core::gcdecode::{GcArena, GcDecode, GcDecoder};
use std::collections::HashSet;
use std::fmt;

/// `HSD_ArchiveHeader`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DatHeader {
    pub file_size: u32,
    pub data_size: u32,
    pub nb_reloc: u32,
    pub nb_public: u32,
    pub nb_extern: u32,
    pub version: [u8; 4],
}
pub const HEADER_SIZE: u32 = 0x20;

/// A public (root) or extern symbol: its data offset and name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatSymbol {
    pub offset: u32,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatError {
    /// Shorter than the header, or than the sections the header announces.
    Truncated { need: u64, have: usize },
    /// `header.file_size != file size`: HSD_ArchiveParse's "byte-order mismatch" check.
    SizeMismatch { header: u32, actual: usize },
    /// A relocation offset or a relocated word points outside the data block.
    BadReloc { at: u32, value: u32 },
    BadSymbol { index: usize },
    NoSuchSymbol(String),
}
impl fmt::Display for DatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for DatError {}

/// A parsed archive. Owns a copy of the data block; nothing is patched.
pub struct DatArchive {
    pub header: DatHeader,
    pub data: Vec<u8>,
    /// Relocation offsets in file order.
    pub relocs: Vec<u32>,
    reloc_set: melee_core::gcdecode::RelocSet,
    pub publics: Vec<DatSymbol>,
    pub externs: Vec<DatSymbol>,
}

/// A decoded root: the native object plus the arena that owns it and everything it points to.
pub struct Loaded<T> {
    pub root: *mut T,
    pub arena: GcArena,
    /// What the bytes could not say (untyped pointers, unrelocated words...). See gcdecode.
    pub issues: Vec<String>,
}
impl<T> Loaded<T> {
    /// The root. Pointers reachable from it stay valid while `self` lives.
    pub fn get(&self) -> &T {
        // SAFETY: root was allocated by the decoder into `arena`, which `self` owns.
        unsafe { &*self.root }
    }
}

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

impl DatArchive {
    /// `HSD_ArchiveParse`, minus the in-place relocation (validated instead).
    pub fn parse(file: &[u8]) -> Result<Self, DatError> {
        if file.len() < HEADER_SIZE as usize {
            return Err(DatError::Truncated { need: HEADER_SIZE as u64, have: file.len() });
        }
        let h = DatHeader {
            file_size: be32(file, 0),
            data_size: be32(file, 4),
            nb_reloc: be32(file, 8),
            nb_public: be32(file, 0xC),
            nb_extern: be32(file, 0x10),
            version: file[0x14..0x18].try_into().unwrap(),
        };
        if h.file_size as usize != file.len() {
            return Err(DatError::SizeMismatch { header: h.file_size, actual: file.len() });
        }
        let data_end = HEADER_SIZE as u64 + h.data_size as u64;
        let reloc_end = data_end + 4 * h.nb_reloc as u64;
        let pub_end = reloc_end + 8 * h.nb_public as u64;
        let ext_end = pub_end + 8 * h.nb_extern as u64;
        if ext_end > file.len() as u64 {
            return Err(DatError::Truncated { need: ext_end, have: file.len() });
        }
        let (data_end, reloc_end, pub_end, ext_end) =
            (data_end as usize, reloc_end as usize, pub_end as usize, ext_end as usize);
        let data = file[HEADER_SIZE as usize..data_end].to_vec();
        let relocs: Vec<u32> = (0..h.nb_reloc as usize).map(|i| be32(file, data_end + 4 * i)).collect();
        for &r in &relocs {
            if r as u64 + 4 > data.len() as u64 {
                return Err(DatError::BadReloc { at: r, value: 0 });
            }
            let v = be32(&data, r as usize);
            if v as usize > data.len() {
                return Err(DatError::BadReloc { at: r, value: v });
            }
        }
        // C: `symbols` exists only when the tables end before file_size.
        let symbols: &[u8] = if ext_end < file.len() { &file[ext_end..] } else { &[] };
        let sym = |tab: usize, n: u32| -> Result<Vec<DatSymbol>, DatError> {
            (0..n as usize)
                .map(|i| {
                    let offset = be32(file, tab + 8 * i);
                    let s = be32(file, tab + 8 * i + 4) as usize;
                    let rest = symbols.get(s..).ok_or(DatError::BadSymbol { index: i })?;
                    let end = rest.iter().position(|&c| c == 0).ok_or(DatError::BadSymbol { index: i })?;
                    Ok(DatSymbol { offset, name: String::from_utf8_lossy(&rest[..end]).into_owned() })
                })
                .collect()
        };
        let publics = sym(reloc_end, h.nb_public)?;
        let externs = sym(pub_end, h.nb_extern)?;
        let reloc_set = relocs.iter().copied().collect();
        Ok(Self { header: h, data, relocs, reloc_set, publics, externs })
    }

    /// `HSD_ArchiveGetPublicAddress`, as a data offset (first match, like the C's linear scan).
    pub fn public_offset(&self, name: &str) -> Option<u32> {
        self.publics.iter().find(|s| s.name == name).map(|s| s.offset)
    }

    /// `HSD_ArchiveGetExtern`.
    pub fn extern_name(&self, i: i32) -> Option<&str> {
        usize::try_from(i).ok().and_then(|i| self.externs.get(i)).map(|s| s.name.as_str())
    }

    /// Is the word at data offset `at` a pointer (in the relocation table)?
    pub fn is_reloc(&self, at: u32) -> bool {
        self.reloc_set.contains(&at)
    }

    /// A decoder over this archive, for loading several roots into one shared object graph.
    pub fn decoder(&self) -> GcDecoder<'_> {
        GcDecoder::new(&self.data, &self.reloc_set)
    }

    /// Decode the `T` at data offset `offset` and everything it reaches.
    pub fn load_at<T: GcDecode>(&self, offset: u32) -> Loaded<T> {
        let mut d = self.decoder();
        let root = d.object::<T>(offset);
        let (arena, issues) = d.finish();
        Loaded { root, arena, issues }
    }

    /// Decode the public root `name` as a `T` (the caller names the type, as the C casts the
    /// `void*` HSD_ArchiveGetPublicAddress returns).
    pub fn load<T: GcDecode>(&self, name: &str) -> Result<Loaded<T>, DatError> {
        let off = self.public_offset(name).ok_or_else(|| DatError::NoSuchSymbol(name.into()))?;
        Ok(self.load_at(off))
    }
}

/// Writes a synthetic archive in the GameCube format (tests and tools; no disc data needed).
#[derive(Default)]
pub struct DatBuilder {
    pub data: Vec<u8>,
    relocs: Vec<u32>,
    publics: Vec<(u32, String)>,
    externs: Vec<(u32, String)>,
}
impl DatBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Append `n` zero bytes aligned to `align`; returns their data offset.
    pub fn alloc(&mut self, n: usize, align: usize) -> u32 {
        while <usize as melee_core::ppc::Ppc>::prem(self.data.len(), align) != 0 {
            self.data.push(0);
        }
        let at = self.data.len() as u32;
        self.data.resize(self.data.len() + n, 0);
        at
    }
    pub fn put_u8(&mut self, at: u32, v: u8) {
        self.data[at as usize] = v;
    }
    pub fn put_u16(&mut self, at: u32, v: u16) {
        self.data[at as usize..at as usize + 2].copy_from_slice(&v.to_be_bytes());
    }
    pub fn put_u32(&mut self, at: u32, v: u32) {
        self.data[at as usize..at as usize + 4].copy_from_slice(&v.to_be_bytes());
    }
    pub fn put_f32(&mut self, at: u32, v: f32) {
        self.put_u32(at, v.to_bits());
    }
    /// Store a pointer at `at` to data offset `target` and add the relocation.
    pub fn put_ptr(&mut self, at: u32, target: u32) {
        self.put_u32(at, target);
        self.relocs.push(at);
    }
    /// Append a NUL-terminated string; returns its offset.
    pub fn string(&mut self, s: &str) -> u32 {
        let at = self.alloc(s.len() + 1, 1);
        self.data[at as usize..at as usize + s.len()].copy_from_slice(s.as_bytes());
        at
    }
    pub fn public(&mut self, name: &str, offset: u32) {
        self.publics.push((offset, name.into()));
    }
    pub fn extern_ref(&mut self, name: &str, offset: u32) {
        self.externs.push((offset, name.into()));
    }
    pub fn finish(mut self) -> Vec<u8> {
        self.alloc(0, 4);
        let mut syms = Vec::new();
        let tab = |list: &[(u32, String)], syms: &mut Vec<u8>| -> Vec<u8> {
            let mut t = Vec::new();
            for (off, name) in list {
                t.extend(off.to_be_bytes());
                t.extend((syms.len() as u32).to_be_bytes());
                syms.extend(name.as_bytes());
                syms.push(0);
            }
            t
        };
        let pubs = tab(&self.publics, &mut syms);
        let exts = tab(&self.externs, &mut syms);
        let mut out = vec![0u8; HEADER_SIZE as usize];
        out.extend(&self.data);
        for r in &self.relocs {
            out.extend(r.to_be_bytes());
        }
        out.extend(pubs);
        out.extend(exts);
        out.extend(syms);
        let n = out.len() as u32;
        for (i, v) in [n, self.data.len() as u32, self.relocs.len() as u32, self.publics.len() as u32,
                       self.externs.len() as u32].into_iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }
}
