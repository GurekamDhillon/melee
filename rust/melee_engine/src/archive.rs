//! The archive host (consolidation plan 1.4). An archive is kept as two images:
//!
//! 1. the **raw image**: the file bytes exactly as the DVD seam delivered them (big endian) in a block the
//!    caller owns below 4 GiB, relocated the way the C's `Locate` does it (`*ptr += (u32) data`, in big-endian
//!    words). Byte-level readers (`psInitDataBankLocate` on a nested archive, textures, `yakumono_param`...) use it;
//! 2. the **native image**: every public symbol the table (`dat_types.rs`) knows, decoded into native
//!    little-endian structs by `melee_core::gcdecode` over the same relocation set. Pointers inside it point into
//!    the same native graph.
//!
//! The native `HSD_Archive` record (GC layout via melee_types, native endianness) gets decoded header fields and
//! native copies of the relocation / public / extern tables, so the ported `HSD_ArchiveGetExtern` and friends read
//! it unchanged. `host_public_address(name)` answers the native root (or the raw address for a `raw` symbol)
//! and stops, naming the symbol, when the table does not know it.

use crate::dat_types::{self, RootType};
use crate::host::Host;
use core::ffi::{c_char, c_void, CStr};
use melee_core::gcdecode::GcArena;
use melee_runtime::dat::{DatArchive, DatSymbol};
use melee_types::{HSD_Archive, HSD_ArchiveExternInfo, HSD_ArchiveHeader, HSD_ArchivePublicInfo, HSD_ArchiveRelocationInfo};
use std::collections::HashMap;

const HEADER_BYTES: usize = 0x20;

#[derive(Clone, Copy)]
enum Root {
    Native(*mut c_void),
    Raw(u32),
}

/// What the host keeps per parsed archive.
pub struct ArchiveImage {
    /// Address of the raw image (the file bytes), also the C's `top_ptr`.
    pub raw: usize,
    pub raw_len: usize,
    /// The parsed file (data block copy, relocation set, symbols), built before relocation.
    pub dat: DatArchive,
    roots: HashMap<String, Root>,
    /// Owns every native object reachable from the roots.
    _arena: Option<GcArena>,
    /// What the bytes could not say while decoding (see gcdecode).
    pub issues: Vec<String>,
    // Native copies of the big-endian tables; `HSD_Archive.reloc_info` etc. point into these.
    _reloc: Vec<HSD_ArchiveRelocationInfo>,
    _public: Vec<HSD_ArchivePublicInfo>,
    _extern: Vec<HSD_ArchiveExternInfo>,
}

impl ArchiveImage {
    pub fn publics(&self) -> &[DatSymbol] {
        &self.dat.publics
    }
    /// The native address of the public symbol `name` (None: not a public, or not in the type table).
    pub fn native_root(&self, name: &str) -> Option<*mut c_void> {
        match self.roots.get(name)? {
            Root::Native(p) => Some(*p),
            Root::Raw(off) => Some((self.raw + HEADER_BYTES + *off as usize) as *mut c_void),
        }
    }
}

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

/// `host_parse`: header and table decode into `archive`, in-place big-endian relocation of the raw image
/// at `src`, and the native decode of the known roots. Returns -1 like the C on a size mismatch.
///
/// # Safety
/// `archive` is a writable `HSD_Archive`; `src` points at `file_size` bytes below 4 GiB that stay alive.
pub unsafe fn host_parse(archive: *mut HSD_Archive, src: *mut u8, file_size: usize) -> i32 {
    if archive.is_null() {
        return -1;
    }
    assert!((src as usize) + file_size <= 0x1_0000_0000, "host_parse: the raw image must be below 4 GiB ({src:p})");
    let file = core::slice::from_raw_parts(src, file_size);
    if file_size < HEADER_BYTES || be32(file, 0) as usize != file_size {
        melee_runtime::tu::lbext::OSReport(
            "host_parse: byte-order mismatch! Please check data format %x %x\n",
            &[
                melee_runtime::tu::lbext::OsArg::U(if file_size >= 4 { be32(file, 0) } else { 0 }),
                melee_runtime::tu::lbext::OsArg::U(file_size as u32),
            ],
        );
        return -1;
    }
    // Parse (and validate) the untouched file first; the relocation below rewrites it.
    let dat = DatArchive::parse(file).unwrap_or_else(|e| panic!("host_parse: {e}"));
    let h = dat.header;

    core::ptr::write_bytes(archive as *mut u8, 0, core::mem::size_of::<HSD_Archive>());
    (*archive).flags |= 1;
    (*archive).header = HSD_ArchiveHeader {
        file_size: h.file_size,
        data_size: h.data_size,
        nb_reloc: h.nb_reloc,
        nb_public: h.nb_public,
        nb_extern: h.nb_extern,
        version: h.version,
        pad: [be32(file, 0x18), be32(file, 0x1C)],
    };

    let mut offset = HEADER_BYTES as u32;
    if h.data_size != 0 {
        (*archive).data = src.add(HEADER_BYTES);
        offset = h.data_size + HEADER_BYTES as u32;
    }
    let reloc: Vec<HSD_ArchiveRelocationInfo> = dat.relocs.iter().map(|&o| HSD_ArchiveRelocationInfo { offset: o }).collect();
    let tab = |first: usize, n: u32| -> Vec<(u32, u32)> {
        (0..n as usize).map(|i| (be32(file, first + 8 * i), be32(file, first + 8 * i + 4))).collect()
    };
    let pub_at = offset as usize + 4 * h.nb_reloc as usize;
    let ext_at = pub_at + 8 * h.nb_public as usize;
    let sym_at = ext_at + 8 * h.nb_extern as usize;
    let public: Vec<HSD_ArchivePublicInfo> =
        tab(pub_at, h.nb_public).into_iter().map(|(o, s)| HSD_ArchivePublicInfo { offset: o, symbol: s }).collect();
    let externs: Vec<HSD_ArchiveExternInfo> =
        tab(ext_at, h.nb_extern).into_iter().map(|(o, s)| HSD_ArchiveExternInfo { offset: o, symbol: s }).collect();
    let mut image = ArchiveImage {
        raw: src as usize,
        raw_len: file_size,
        dat,
        roots: HashMap::new(),
        _arena: None,
        issues: Vec::new(),
        _reloc: reloc,
        _public: public,
        _extern: externs,
    };
    if h.nb_reloc != 0 {
        (*archive).reloc_info = image._reloc.as_mut_ptr();
    }
    if h.nb_public != 0 {
        (*archive).public_info = image._public.as_mut_ptr();
    }
    if h.nb_extern != 0 {
        (*archive).extern_info = image._extern.as_mut_ptr();
    }
    if sym_at < h.file_size as usize {
        (*archive).symbols = src.add(sym_at) as *mut i8;
    }
    (*archive).top_ptr = src as *mut c_void;

    // Native image: decode every public the table knows, in one shared graph.
    {
        let mut roots = HashMap::new();
        let mut d = image.dat.decoder();
        for p in &image.dat.publics {
            match dat_types::type_of(&p.name) {
                Some(RootType::Raw) => {
                    roots.insert(p.name.clone(), Root::Raw(p.offset));
                }
                Some(ty) => {
                    // an array root runs to the next public, or the end of the data section
                    let next = image.dat.publics.iter().map(|q| q.offset).filter(|&o| o > p.offset).min();
                    let extent = next.unwrap_or(image.dat.data.len() as u32) - p.offset;
                    let r = dat_types::decode_root(&mut d, ty, p.offset, extent);
                    roots.insert(p.name.clone(), Root::Native(r));
                }
                None => {}
            }
        }
        let (arena, issues) = d.finish();
        image.roots = roots;
        image._arena = Some(arena);
        image.issues = issues;
    }

    // Raw image: `Locate`, in big-endian words.
    let base = (*archive).data as usize as u32;
    for &r in &image.dat.relocs {
        let p = src.add(HEADER_BYTES + r as usize);
        let v = u32::from_be_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]).wrapping_add(base);
        for (i, x) in v.to_be_bytes().iter().enumerate() {
            *p.add(i) = *x;
        }
    }

    Host::get().archives.insert(archive as usize, image);
    0
}

/// `host_public_address`: the native root of public `symbols` (null when the archive has no such
/// public). Stops, naming the symbol, when the archive has it but `dat_symbols.tsv` has no type for it.
/// None: `archive` was not parsed by this host.
///
/// # Safety
/// `symbols` is a NUL-terminated string.
pub unsafe fn host_public_address(archive: *mut HSD_Archive, symbols: *const c_char) -> Option<*mut c_void> {
    let want = CStr::from_ptr(symbols).to_string_lossy();
    let image = Host::get().archives.get(&(archive as usize))?;
    if !image.dat.publics.iter().any(|s| s.name == want) {
        return Some(core::ptr::null_mut());
    }
    match image.native_root(&want) {
        Some(p) => Some(p),
        None => panic!(
            "melee_engine: archive symbol `{want}` has no type in tools/rust-port/engine/dat_symbols.tsv \
             (add the C type its call site casts it to, then run gen_dat_types.py)"
        ),
    }
}

/// `host_locate_extern` on the raw image: patches the chain of slots the extern's data offset starts, in
/// big-endian words, with the low 32 bits of `addr`. False when `archive` was not parsed by this host.
///
/// # Safety
/// `symbols` is a NUL-terminated string.
pub unsafe fn host_locate_extern(archive: *mut HSD_Archive, symbols: *const c_char, addr: *mut c_void) -> bool {
    let Some(image) = Host::get().archives.get(&(archive as usize)) else { return false };
    let want = CStr::from_ptr(symbols).to_string_lossy();
    let Some(ext) = image.dat.externs.iter().find(|s| s.name == want) else { return true };
    let data = (*archive).data;
    let mut offset = ext.offset;
    while offset != u32::MAX && offset < (*archive).header.data_size {
        let slot = data.add(offset as usize);
        let next = u32::from_be_bytes([*slot, *slot.add(1), *slot.add(2), *slot.add(3)]);
        for (i, x) in (addr as usize as u32).to_be_bytes().iter().enumerate() {
            *slot.add(i) = *x;
        }
        offset = next;
    }
    true
}
