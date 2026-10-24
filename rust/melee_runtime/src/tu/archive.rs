//! Port of src/sysdolphin/baselib/archive.c on the GameCube record `melee_types::HSD_Archive`:
//! `HSD_ArchiveParse`, `HSD_ArchiveGetPublicAddress`, `HSD_ArchiveGetExtern`,
//! `HSD_ArchiveLocateExtern`, and the static-inline `Locate`.
//!
//! Widths follow the C: relocation words and extern slots are 32-bit (`u32`); the pointer the
//! relocation produces is the low 32 bits of the data address (`(u32) archive->data`). A native
//! 64-bit run therefore only produces usable pointers when the buffer is below 4 GiB; the
//! loader in `crate::dat` (relocation set + `GcDecoder`) is the pointer-width-safe path.
//! The PC profiler wrapper (`HSD_ArchiveParse` -> `_profile_body`) is a no-op here.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, dead_code)]

use core::ffi::{c_void, CStr};
use core::ptr::null_mut;
use melee_types::*;

pub mod ext {
    #![allow(non_snake_case, unused_variables)]
    /// `OSReport` (dolphin/os): the one diagnostic of this TU; not ported.
    pub unsafe fn OSReport(msg: &str) {
        panic!("TODO: OSReport is not ported: {msg}")
    }
}

const HEADER_BYTES: usize = 0x20;

/// `static inline void Locate(HSD_Archive*)`.
unsafe fn Locate(archive: *mut HSD_Archive) {
    let mut i: u32 = 0;
    while i < (*archive).header.nb_reloc {
        let off = (*(*archive).reloc_info.add(i as usize)).offset;
        let ptr = (*archive).data.add(off as usize) as *mut u32;
        ptr.write_unaligned(ptr.read_unaligned().wrapping_add((*archive).data as usize as u32));
        i += 1;
    }
}

/// `HSD_ArchiveParse`
pub unsafe extern "C" fn HSD_ArchiveParse(archive: *mut HSD_Archive, src: *mut u8, file_size: usize) -> i32 {
    if archive.is_null() {
        return -1;
    }

    core::ptr::write_bytes(archive as *mut u8, 0, core::mem::size_of::<HSD_Archive>());
    (*archive).flags |= 1;
    core::ptr::copy_nonoverlapping(src, archive as *mut u8, HEADER_BYTES);

    if (*archive).header.file_size as usize != file_size {
        ext::OSReport(&format!(
            "HSD_ArchiveParse: byte-order mismatch! Please check data format {:x} {:x}\n",
            (*archive).header.file_size,
            file_size
        ));
        return -1;
    }

    let mut offset: u32 = HEADER_BYTES as u32;
    if (*archive).header.data_size != 0 {
        (*archive).data = src.add(HEADER_BYTES);
        offset = (*archive).header.data_size.wrapping_add(HEADER_BYTES as u32);
    }
    if (*archive).header.nb_reloc != 0 {
        (*archive).reloc_info = src.add(offset as usize) as *mut HSD_ArchiveRelocationInfo;
        offset = offset.wrapping_add((*archive).header.nb_reloc.wrapping_mul(core::mem::size_of::<HSD_ArchiveRelocationInfo>() as u32));
    }
    if (*archive).header.nb_public != 0 {
        (*archive).public_info = src.add(offset as usize) as *mut HSD_ArchivePublicInfo;
        offset = offset.wrapping_add((*archive).header.nb_public.wrapping_mul(core::mem::size_of::<HSD_ArchivePublicInfo>() as u32));
    }
    if (*archive).header.nb_extern != 0 {
        (*archive).extern_info = src.add(offset as usize) as *mut HSD_ArchiveExternInfo;
        offset = offset.wrapping_add((*archive).header.nb_extern.wrapping_mul(core::mem::size_of::<HSD_ArchiveExternInfo>() as u32));
    }
    if offset < (*archive).header.file_size {
        (*archive).symbols = src.add(offset as usize) as *mut i8;
    }

    (*archive).top_ptr = src as *mut c_void;
    Locate(archive);

    0
}

/// `HSD_ArchiveGetPublicAddress`
pub unsafe extern "C" fn HSD_ArchiveGetPublicAddress(archive: *mut HSD_Archive, symbols: *const i8) -> *mut c_void {
    let want = CStr::from_ptr(symbols);
    let mut i: u32 = 0;
    while i < (*archive).header.nb_public {
        let info = (*archive).public_info.add(i as usize);
        let name = CStr::from_ptr((*archive).symbols.add((*info).symbol as usize));
        if name == want {
            return (*archive).data.add((*info).offset as usize) as *mut c_void;
        }
        i += 1;
    }
    null_mut()
}

/// `HSD_ArchiveGetExtern`
pub unsafe extern "C" fn HSD_ArchiveGetExtern(archive: *mut HSD_Archive, offset: i32) -> *mut i8 {
    if offset < 0 || (*archive).header.nb_extern <= offset as u32 {
        return null_mut();
    }
    (*archive).symbols.add((*(*archive).extern_info.add(offset as usize)).symbol as usize)
}

/// `HSD_ArchiveLocateExtern`: patches every slot of the extern's chain with `addr` (low 32 bits).
pub unsafe extern "C" fn HSD_ArchiveLocateExtern(archive: *mut HSD_Archive, symbols: *const i8, addr: *mut c_void) {
    let want = CStr::from_ptr(symbols);
    let mut offset: u32 = u32::MAX;
    let mut i: u32 = 0;
    while i < (*archive).header.nb_extern {
        let info = (*archive).extern_info.add(i as usize);
        let name = CStr::from_ptr((*archive).symbols.add((*info).symbol as usize));
        if want == name {
            offset = (*info).offset;
            break;
        }
        i += 1;
    }

    if offset == u32::MAX {
        return;
    }

    while offset != u32::MAX && offset < (*archive).header.data_size {
        let slot = (*archive).data.add(offset as usize) as *mut u32;
        let next = slot.read_unaligned();
        slot.write_unaligned(addr as usize as u32);
        offset = next;
    }
}
