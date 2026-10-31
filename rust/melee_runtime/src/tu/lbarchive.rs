//! src/melee/lb/lbarchive.c: loading a `.dat` archive from the disc (heap block for the data, one for the
//! `HSD_Archive`, parse, locate externs) and filling symbol pointers from the archive's public table;
//! plus `lbArchiveRelocate`, the native-pointer relocation of an already loaded archive. All 11 functions
//! with a body (and the 5 static inlines folded into them) are checked against the C by a state oracle
//! (tests/tu_lbarchive.rs over gen/turef/tu_lbarchive_ref.c): callee log, return value, symbol outputs, heap
//! use and the relocated blob for every call.
//!
//! Variadics: the C functions take `(..., void** dst0, const char* name0, void** dst1, const char* name1,
//! ..., NULL)`. Here the whole run is one slice `rest = [dst0, name0, dst1, name1, ..., NULL]` (the first
//! `dst` is the C's fixed `symbols` parameter, as in `tu_ext`'s hooks of these functions); a slice that ends
//! without the NULL ends the list (the C would read garbage).
//!
//! `HSD_ASSERT` is not under `MUST_MATCH` on the PC build, so `__assert` gets `__LINE__` (24, 87, 149, 150),
//! not the macro's `line` argument; `file` is the path the build passes to the compiler.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::*;

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut HSD_ARCHIVEPARSE: Option<fn(*mut HSD_Archive, *mut u8, usize) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn HSD_ArchiveParse(archive: *mut HSD_Archive, data: *mut u8, file_size: usize) -> i32 {
                match unsafe { HSD_ARCHIVEPARSE } {
                    Some(f) => f(archive, data, file_size),
                    None => { return unsafe { crate::tu::archive::HSD_ArchiveParse(archive as _, data as _, file_size as _) as _ }; },
                }
            }
    pub static mut HSD_ARCHIVEGETEXTERN: Option<fn(*mut HSD_Archive, i32) -> *mut c_char> = None;
            #[allow(non_snake_case)]
            pub fn HSD_ArchiveGetExtern(archive: *mut HSD_Archive, i: i32) -> *mut c_char {
                match unsafe { HSD_ARCHIVEGETEXTERN } {
                    Some(f) => f(archive, i),
                    None => { return unsafe { crate::tu::archive::HSD_ArchiveGetExtern(archive as _, i as _) as _ }; },
                }
            }
    pub static mut HSD_ARCHIVELOCATEEXTERN: Option<fn(*mut HSD_Archive, *const c_char, *mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_ArchiveLocateExtern(archive: *mut HSD_Archive, name: *const c_char, addr: *mut c_void)  {
                match unsafe { HSD_ARCHIVELOCATEEXTERN } {
                    Some(f) => f(archive, name, addr),
                    None => { unsafe { crate::tu::archive::HSD_ArchiveLocateExtern(archive as _, name as _, addr as _); } return; },
                }
            }
    pub static mut HSD_ARCHIVEGETPUBLICADDRESS: Option<fn(*mut HSD_Archive, *const c_char) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn HSD_ArchiveGetPublicAddress(archive: *mut HSD_Archive, name: *const c_char) -> *mut c_void {
                match unsafe { HSD_ARCHIVEGETPUBLICADDRESS } {
                    Some(f) => f(archive, name),
                    None => { return unsafe { crate::tu::archive::HSD_ArchiveGetPublicAddress(archive as _, name as _) as _ }; },
                }
            }
    pub static mut LBHEAP_80015BD0: Option<fn(i32, usize) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn lbHeap_80015BD0(heap_id: i32, size: usize) -> *mut c_void {
                match unsafe { LBHEAP_80015BD0 } {
                    Some(f) => f(heap_id, size),
                    None => { return unsafe { crate::tu::lbheap::lbHeap_80015BD0(heap_id as _, size as _) as _ }; },
                }
            }
    pub static mut LBHEAP_80015CA8: Option<fn(i32, *mut c_void) > = None;
            #[allow(non_snake_case)]
            pub fn lbHeap_80015CA8(heap_id: i32, ptr: *mut c_void)  {
                match unsafe { LBHEAP_80015CA8 } {
                    Some(f) => f(heap_id, ptr),
                    None => { unsafe { crate::tu::lbheap::lbHeap_80015CA8(heap_id as _, ptr as _); } return; },
                }
            }
    pub static mut LBFILEGETSIZE: Option<fn(*const c_char) -> usize> = None;
            #[allow(non_snake_case)]
            pub fn lbFileGetSize(basename: *const c_char) -> usize {
                match unsafe { LBFILEGETSIZE } {
                    Some(f) => f(basename),
                    None => { return unsafe { crate::tu::lbfile::lbFileGetSize(basename as _) as _ }; },
                }
            }
    pub static mut LBFILE_8001668C: Option<fn(*const c_char, *mut c_void, *mut usize) > = None;
            #[allow(non_snake_case)]
            pub fn lbFile_8001668C(basename: *const c_char, dst: *mut c_void, size: *mut usize)  {
                match unsafe { LBFILE_8001668C } {
                    Some(f) => f(basename, dst, size),
                    None => { unsafe { crate::tu::lbfile::lbFile_8001668C(basename as _, dst as _, size as _); } return; },
                }
            }
    pub static mut LBDVD_8001819C: Option<fn(*const c_char) -> *mut HSD_Archive> = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_8001819C(basename: *const c_char) -> *mut HSD_Archive {
                match unsafe { LBDVD_8001819C } {
                    Some(f) => f(basename),
                    None => { return unsafe { crate::tu::lbdvd::lbDvd_8001819C(basename as _) as _ }; },
                }
            }
    /// `OSReport(fmt, ...)`: the variadic arguments as one slice of register-width values.
    hook!(OSREPORT, OSReport, (fmt: *const c_char, args: &[u64]));
    pub static mut ASSERT: Option<fn(*const c_char, u32, *const c_char)> = None;
    /// `__assert(file, line, text)` (baselib/debug.c): never returns.
    pub fn __assert(file: *const c_char, line: u32, text: *const c_char) -> ! {
        if let Some(f) = unsafe { ASSERT } {
            f(file, line, text);
        }
        panic!("TODO: __assert is not ported / returned")
    }
}

const FILE: &[u8] = b"src/melee/lb/lbarchive.c\0";

/// `HSD_ASSERT(_, cond)` with `__LINE__ == line`.
#[inline]
unsafe fn hsd_assert(line: u32, cond: bool, text: &'static [u8]) {
    if !cond {
        ext::__assert(FILE.as_ptr() as *const c_char, line, text.as_ptr() as *const c_char);
    }
}

/// `OSRoundUp32B(x)`: the C does the arithmetic in u32.
#[inline]
fn os_round_up_32b(x: usize) -> usize {
    ((x as u32).wrapping_add(32 - 1) & !(32 - 1)) as usize
}

/// Cursor over a `[dst, name, dst, name, ..., NULL]` slice; an exhausted slice reads as NULL.
struct VaList<'a> {
    rest: &'a [*mut c_void],
    at: usize,
}
impl VaList<'_> {
    fn next(&mut self) -> *mut c_void {
        let v = self.rest.get(self.at).copied().unwrap_or(null_mut());
        self.at += 1;
        v
    }
}

/// `lbArchive_InitializeDAT(archive, data, length)`: parse, then locate every extern symbol to NULL.
pub unsafe fn lbArchive_InitializeDAT(archive: *mut HSD_Archive, data: *mut c_void, length: usize) {
    melee_core::ct!("lbArchive_InitializeDAT");
    if ext::HSD_ArchiveParse(archive, data as *mut u8, length) == -1 {
        ext::OSReport(b"HSD_ArchiveParse error!\n\0".as_ptr() as *const c_char, &[]);
        hsd_assert(24, false, b"0\0");
    }
    let mut i: i32 = 0;
    loop {
        let symbol = ext::HSD_ArchiveGetExtern(archive, i);
        i += 1;
        if !symbol.is_null() {
            ext::HSD_ArchiveLocateExtern(archive, symbol, null_mut());
        }
        if symbol.is_null() {
            return;
        }
    }
}

/// The `[dst, name]*` loop shared by `lbArchive_LoadSections`, `vLoadSections` and `vLoadSectionsFatal`.
unsafe fn load_sections(archive: *mut HSD_Archive, va: &mut VaList, fatal: bool) {
    let mut symbol = va.next() as *mut *mut c_void;
    while !symbol.is_null() {
        let symbol_name = va.next() as *const c_char;
        *symbol = null_mut();
        *symbol = ext::HSD_ArchiveGetPublicAddress(archive, symbol_name);
        if (*symbol).is_null() {
            ext::OSReport(b"Cannot find symbol %s.\n\0".as_ptr() as *const c_char, &[symbol_name as u64]);
            if fatal {
                hsd_assert(87, false, b"0\0");
            }
        }
        symbol = va.next() as *mut *mut c_void;
    }
}

/// `lbArchive_LoadSections(archive, symbol, ...)`: fill the symbol pointers; a missing one is reported only.
/// `rest` = `[symbol, name, symbol, name, ..., NULL]`.
pub unsafe fn lbArchive_LoadSections(archive: *mut HSD_Archive, rest: &[*mut c_void]) {
    melee_core::ct!("lbArchive_LoadSections");
    load_sections(archive, &mut VaList { rest, at: 0 }, false);
}

/// `lbArchive_LoadArchive_inline(filename)` (also `lbArchive_LoadArchive` and the open-coded copies in
/// `LoadSymbols`/`80016DBC`/`80016F80`/`80017040`/`800171CC`): heap blocks, read the file, parse.
unsafe fn load_archive(filename: *const c_char) -> *mut HSD_Archive {
    let data = ext::lbHeap_80015BD0(0, os_round_up_32b(ext::lbFileGetSize(filename)));
    let archive = ext::lbHeap_80015BD0(0, core::mem::size_of::<HSD_Archive>()) as *mut HSD_Archive;
    let mut length: usize = 0;
    ext::lbFile_8001668C(filename, data, &mut length);
    lbArchive_InitializeDAT(archive, data, length);
    archive
}

/// `lbArchive_LoadArchive(filename)`.
pub unsafe fn lbArchive_LoadArchive(filename: *const c_char) -> *mut HSD_Archive {
    melee_core::ct!("lbArchive_LoadArchive");
    load_archive(filename)
}

/// `lbArchive_LoadSymbols(filename, symbols, ...)`: load, then fill the symbols; a missing one asserts.
/// `rest` = `[symbols, name, ..., NULL]`.
pub unsafe fn lbArchive_LoadSymbols(filename: *const c_char, rest: &[*mut c_void]) -> *mut HSD_Archive {
    melee_core::ct!("lbArchive_LoadSymbols");
    let archive = load_archive(filename);
    load_sections(archive, &mut VaList { rest, at: 0 }, true);
    archive
}

/// `lbArchive_80016DBC(filename, symbols, ...)`: as `LoadSymbols` but a missing symbol is only reported.
pub unsafe fn lbArchive_80016DBC(filename: *const c_char, rest: &[*mut c_void]) -> *mut HSD_Archive {
    melee_core::ct!("lbArchive_80016DBC");
    let archive = load_archive(filename);
    load_sections(archive, &mut VaList { rest, at: 0 }, false);
    archive
}

/// `lbArchive_80016EFC(archive)`: free a loaded archive's data block (at `data - 0x20`) and its header block.
pub unsafe fn lbArchive_80016EFC(archive: *mut HSD_Archive) {
    melee_core::ct!("lbArchive_80016EFC");
    hsd_assert(149, !archive.is_null(), b"archive\0");
    hsd_assert(150, (*archive).flags & HSD_ARCHIVE_DONT_FREE as u32 != 0, b"archive->flags & HSD_ARCHIVE_DONT_FREE\0");
    ext::lbHeap_80015CA8(0, (*archive).data.wrapping_sub(0x20) as *mut c_void);
    ext::lbHeap_80015CA8(0, archive as *mut c_void);
}

/// Preloaded copy from the DVD cache, else a fresh load; `true` when preloaded.
unsafe fn dvd_or_load(filename: *const c_char) -> (*mut HSD_Archive, bool) {
    let archive = ext::lbDvd_8001819C(filename);
    if !archive.is_null() {
        (archive, true)
    } else {
        (load_archive(filename), false)
    }
}

/// `lbArchive_80016F80(archive, filename)`: the archive from the DVD cache or loaded; returns whether it was cached.
pub unsafe fn lbArchive_80016F80(archive: *mut *mut HSD_Archive, filename: *const c_char) -> bool {
    melee_core::ct!("lbArchive_80016F80");
    let (var_r3, result) = dvd_or_load(filename);
    if !archive.is_null() {
        *archive = var_r3;
    }
    result
}

/// `lbArchive_80017040(dst, filename, symbols, ...)`: `80016F80` plus the symbols (a missing one asserts).
/// `rest` = `[symbols, name, ..., NULL]`.
pub unsafe fn lbArchive_80017040(dst: *mut *mut HSD_Archive, filename: *const c_char, rest: &[*mut c_void]) -> bool {
    melee_core::ct!("lbArchive_80017040");
    let (archive, preloaded) = dvd_or_load(filename);
    load_sections(archive, &mut VaList { rest, at: 0 }, true);
    if !dst.is_null() {
        *dst = archive;
    }
    preloaded
}

/// `lbArchive_800171CC(dst, filename, symbols, ...)`: as `80017040` but a missing symbol is only reported.
pub unsafe fn lbArchive_800171CC(dst: *mut *mut HSD_Archive, filename: *const c_char, rest: &[*mut c_void]) -> bool {
    melee_core::ct!("lbArchive_800171CC");
    let (archive, preloaded) = dvd_or_load(filename);
    load_sections(archive, &mut VaList { rest, at: 0 }, false);
    if !dst.is_null() {
        *dst = archive;
    }
    preloaded
}

/// `static Locate(archive, base_addr)`: add `base_addr` to the pointer-sized word at every relocation offset.
unsafe fn Locate(archive: *mut HSD_Archive, base_addr: isize) {
    let mut i: u32 = 0;
    while i < (*archive).header.nb_reloc {
        let ptr = (*(*archive).reloc_info.add(i as usize)).offset;
        let at = (*archive).data.add(ptr as usize) as *mut isize;
        at.write_unaligned(at.read_unaligned().wrapping_add(base_addr));
        i += 1;
    }
}

/// `lbArchiveRelocate(archive, src, file_size, base_addr)`: set `archive` up over the loaded file `src` and
/// relocate it; -1 for a null archive or a `file_size` that is not the header's.
pub unsafe fn lbArchiveRelocate(archive: *mut HSD_Archive, src: *mut u8, file_size: usize, base_addr: isize) -> i32 {
    melee_core::ct!("lbArchiveRelocate");
    if archive.is_null() {
        return -1;
    }
    core::ptr::write_bytes(archive as *mut u8, 0, core::mem::size_of::<HSD_Archive>());
    (*archive).flags |= 1;
    core::ptr::copy_nonoverlapping(src as *const u8, archive as *mut u8, core::mem::size_of::<HSD_ArchiveHeader>());

    if (*archive).header.file_size as usize != file_size {
        ext::OSReport(
            b"lbArchiveRelocate: byte-order mismatch! Please check data format %x %x\n\0".as_ptr() as *const c_char,
            &[(*archive).header.file_size as u64, file_size as u64],
        );
        return -1;
    }

    let mut file_offset: usize = core::mem::size_of::<HSD_ArchiveHeader>();
    if (*archive).header.data_size != 0 {
        (*archive).data = src.add(file_offset);
        file_offset = (*archive).header.data_size as usize + core::mem::size_of::<HSD_ArchiveHeader>();
    }
    if (*archive).header.nb_reloc != 0 {
        (*archive).reloc_info = src.add(file_offset) as *mut HSD_ArchiveRelocationInfo;
        file_offset += (*archive).header.nb_reloc as usize * core::mem::size_of::<HSD_ArchiveRelocationInfo>();
    }
    if (*archive).header.nb_public != 0 {
        (*archive).public_info = src.add(file_offset) as *mut HSD_ArchivePublicInfo;
        file_offset += (*archive).header.nb_public as usize * core::mem::size_of::<HSD_ArchivePublicInfo>();
    }
    if (*archive).header.nb_extern != 0 {
        (*archive).extern_info = src.add(file_offset) as *mut HSD_ArchiveExternInfo;
        file_offset += (*archive).header.nb_extern as usize * core::mem::size_of::<HSD_ArchiveExternInfo>();
    }
    if file_offset < (*archive).header.file_size as usize {
        (*archive).symbols = src.add(file_offset) as *mut i8;
    }

    Locate(archive, base_addr);

    0
}
