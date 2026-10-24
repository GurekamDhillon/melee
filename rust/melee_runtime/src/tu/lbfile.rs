//! src/melee/lb/lbfile.c: file loading from the disc through the DevCom queue (`lbFile_800161C4`,
//! `lbFile_80016760`, ...) and the `.dat`/`.usd` file name builder. Every function is checked against the C
//! by a state oracle (tests/tu_lbfile.rs over gen/turef/tu_lbfile_ref.c): the same seeded stream drives
//! the callee results, the callees log their arguments, and the log, the results and the module state
//! are compared.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::{c_char, c_void};
use core::ptr::addr_of_mut;
use melee_types::*;

/// Callees in TUs that are not ported (baselib devcom, lb_0195, lbdvd, dolphin dvd/os); the language
/// getters and the heap allocator default to this crate's ports.
pub mod ext {
    use super::*;
    use crate::tu::lbext::lb_hook;
    pub use crate::tu::lbext::{OSReport, OsArg, __assert};
    // BEGIN GENERATED ext
    lb_hook!(H_lb_800195D0, lb_800195D0, ());
    lb_hook!(H_HSD_DevComRequest, HSD_DevComRequest, (a: i32, b: usize, c: usize, d: usize, e: i32, f: i32, g: HSD_DevComCallback, h: usize) -> i32);
    lb_hook!(H_DVDFastOpen, DVDFastOpen, (a: i32, b: *mut DVDFileInfo) -> i32);
    lb_hook!(H_DVDClose, DVDClose, (a: *mut DVDFileInfo) -> i32);
    lb_hook!(H_OSDisableInterrupts, OSDisableInterrupts, () -> i32);
    lb_hook!(H_OSRestoreInterrupts, OSRestoreInterrupts, (a: i32) -> i32);
    lb_hook!(H_DVDConvertPathToEntrynum, DVDConvertPathToEntrynum, (a: *const c_char) -> i32);
    lb_hook!(H_lbDvd_8001819C, lbDvd_8001819C, (a: *const c_char) -> *mut HSD_Archive);
    lb_hook!(H_lbHeap_80015BD0, lbHeap_80015BD0, (a: i32, b: usize) -> *mut c_void = |a, b| unsafe { crate::tu::lbheap::lbHeap_80015BD0(a, b) });
    lb_hook!(H_lbLang_IsSettingUS, lbLang_IsSettingUS, () -> bool = crate::tu::lblanguage::lbLang_IsSettingUS);
    lb_hook!(H_lbLang_IsSavedLanguageUS, lbLang_IsSavedLanguageUS, () -> bool = crate::tu::lblanguage::lbLang_IsSavedLanguageUS);
    // END GENERATED ext
}

const FILE: &str = "lb/lbfile.c";

/// `static bool cancel`: set by the DevCom completion callback ::lbFile_8001615C.
static mut cancel: bool = false;

/// The DevCom completion callback of the blocking loads: marks the request done (the body, so that the
/// assert path can unwind in tests; the `extern "C"` entry below is what DevCom is handed).
pub fn lbFile_8001615C_body(cancelflag: i32) {
    if cancelflag != 0 {
        ext::__assert(FILE, 19, "!cancelflag");
    }
    unsafe { cancel = true };
}

/// `lbFile_8001615C`: the DevCom completion callback of the blocking loads.
pub unsafe extern "C" fn lbFile_8001615C(_dcreq: i32, _args: usize, _buf: *mut c_void, cancelflag: i32) {
    lbFile_8001615C_body(cancelflag);
}

/// `discIsDone`: pumps the disc queue once, then reports whether the request completed.
fn discIsDone() -> bool {
    ext::lb_800195D0();
    unsafe { cancel }
}

/// `waitForDisc`: spins until the completion callback ran.
fn waitForDisc() {
    while !discIsDone() {}
}

/// `lbFile_800161C4`: blocking DevCom request of `size` bytes.
pub fn lbFile_800161C4(file: i32, src: usize, dst: usize, size: usize, ty: i32, pri: i32) {
    unsafe { cancel = false };
    ext::HSD_DevComRequest(file, src, dst, size, ty, pri, Some(lbFile_8001615C), 0);
    waitForDisc();
}

const MAX_FILENAME_LENGTH: usize = 0x20;
const FILE_EXTENSION_LENGTH: i32 = 4; // ".usd" or ".dat"
const MAX_BASENAME_LENGTH: i32 = MAX_FILENAME_LENGTH as i32 - FILE_EXTENSION_LENGTH;

/// `static char result[MAX_FILENAME_LENGTH]` of `lbFileGetFullName`.
static mut RESULT: [u8; MAX_FILENAME_LENGTH] = [0; MAX_FILENAME_LENGTH];

/// `strcpy`.
unsafe fn strcpy(dst: *mut u8, src: *const u8) {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

/// `lbFileGetFullName`: `basename` with the locale's extension appended when it has none; the result is
/// a static buffer.
///
/// # Safety
/// `basename` must be a NUL-terminated string.
pub unsafe fn lbFileGetFullName(basename: *const c_char) -> *mut c_char {
    let result = addr_of_mut!(RESULT) as *mut u8;
    let mut cur = basename as *const u8;
    let mut pos: i32 = 0;

    while *cur != 0 && *cur != b'.' {
        // no room for file extension?
        if pos > MAX_BASENAME_LENGTH {
            ext::OSReport("Error : file name too long %s.", &[ext::OsArg::S(basename)]);
            ext::__assert(FILE, 67, "NULL");
        }
        *result.offset(pos as isize) = *cur;
        pos += 1;
        cur = cur.add(1);
    }
    // keep any existing file extension
    if *cur != 0 && *cur.add(1) != 0 {
        strcpy(result, basename as *const u8);
    // otherwise, append the appropriate extension for the locale
    } else if *cur == b'.' {
        *result.offset(pos as isize) = b'.';
        pos += 1;
        strcpy(result.offset(pos as isize), if ext::lbLang_IsSettingUS() { b"usd\0".as_ptr() } else { b"dat\0".as_ptr() });
    } else {
        *result.offset(pos as isize) = b'.';
        pos += 1;
        strcpy(result.offset(pos as isize), if ext::lbLang_IsSavedLanguageUS() { b"usd\0".as_ptr() } else { b"dat\0".as_ptr() });
    }
    result as *mut c_char
}

/// `lbFile_8001634C`: the length of the disc file number `fileno`.
pub fn lbFile_8001634C(fileno: i32) -> usize {
    let mut info: DVDFileInfo = DVDFileInfo::zeroed();
    let intr = ext::OSDisableInterrupts();
    if ext::DVDFastOpen(fileno, &mut info) == 0 {
        ext::OSReport("Cannot open file no=%d.", &[ext::OsArg::I(fileno)]);
        ext::__assert(FILE, 101, "0");
    }
    let length = info.length as usize;
    ext::DVDClose(&mut info);
    ext::OSRestoreInterrupts(intr);
    length
}

/// `lbFileGetSize`: the length of the disc file `basename` (extension added by the locale).
///
/// # Safety
/// `basename` must be a NUL-terminated string.
pub unsafe fn lbFileGetSize(basename: *const c_char) -> usize {
    let filename = lbFileGetFullName(basename);
    let entry_num = ext::DVDConvertPathToEntrynum(filename);
    if entry_num == -1 {
        ext::OSReport("file isn't exist %s = %d\n", &[ext::OsArg::S(filename), ext::OsArg::I(entry_num)]);
        ext::__assert(FILE, 116, "entry_num != -1");
    }
    lbFile_8001634C(entry_num)
}

#[inline]
fn round_up_32(x: usize) -> usize {
    // the C's size_t is 32 bits wide
    ((x as u32).wrapping_add(31) & !31u32) as usize
}

/// `lbFile_800164A4`: queues the read of disc file `file` to `dst` (main RAM when `dst >= 0x80000000`, else
/// ARAM) and stores the file length in `*size`.
///
/// # Safety
/// `size` must be valid for writes.
pub unsafe fn lbFile_800164A4(file: i32, dst: usize, size: *mut usize, pri: i32, callback: HSD_DevComCallback, args: usize) {
    *size = lbFile_8001634C(file);
    let ty = if dst >= 0x8000_0000 { 0x21 } else { 0x23 };
    ext::HSD_DevComRequest(file, 0, dst, round_up_32(*size), ty, pri, callback, args);
}

/// `lbFile_80016580`: queues the read of `basename` into `dst` (length to `*size`); `callback` runs when
/// the transfer is done.
///
/// # Safety
/// `basename` is a NUL-terminated string, `size` valid for writes.
pub unsafe fn lbFile_80016580(basename: *const c_char, dst: *mut c_void, size: *mut usize, callback: HSD_DevComCallback, args: usize) {
    let filename = lbFileGetFullName(basename);
    let entry_num = ext::DVDConvertPathToEntrynum(filename);
    if entry_num == -1 {
        ext::OSReport("file isn't exist %s = %d\n", &[ext::OsArg::S(filename), ext::OsArg::I(entry_num)]);
        ext::__assert(FILE, 140, "entry_num != -1");
    }
    lbFile_800164A4(entry_num, dst as usize, size, 1, callback, args);
}

/// `lbFile_8001668C`: blocking read of `basename` into `dst`.
///
/// # Safety
/// As `lbFile_80016580`.
pub unsafe fn lbFile_8001668C(basename: *const c_char, dst: *mut c_void, size: *mut usize) {
    cancel = false;
    lbFile_80016580(basename, dst, size, Some(lbFile_8001615C), 0);
    waitForDisc();
}

/// `lbFile_80016760_inline` (static inline): the blocking read of `basename` into a new block of heap
/// `heap_id`.
unsafe fn lbFile_80016760_inline(heap_id: i32, basename: *const c_char, dst: *mut *mut c_void, size: *mut usize) {
    *size = lbFileGetSize(basename);
    *dst = ext::lbHeap_80015BD0(heap_id, round_up_32(*size));
    lbFile_80016580(basename, *dst, size, Some(lbFile_8001615C), 0);
    waitForDisc();
}

/// `lbFile_80016760`: blocking read of `basename` into a new block of heap 0.
///
/// # Safety
/// `basename` is a NUL-terminated string, `dst`/`size` valid for writes.
pub unsafe fn lbFile_80016760(basename: *const c_char, dst: *mut *mut c_void, size: *mut usize) {
    cancel = false;
    lbFile_80016760_inline(0, basename, dst, size);
}

/// `lbFile_LoadToHeap` (TARGET_PC): reads a disc file into a new block of the given heap and returns it.
///
/// # Safety
/// As `lbFile_80016760`.
pub unsafe fn lbFile_LoadToHeap(heap_id: i32, basename: *const c_char, size: *mut usize) -> *mut c_void {
    let mut dst: *mut c_void = core::ptr::null_mut();
    cancel = false;
    lbFile_80016760_inline(heap_id, basename, &mut dst, size);
    dst
}

/// `lbFile_LoadInto` (TARGET_PC): reads a disc file into the caller's buffer (`cap` bytes, at least the file
/// size rounded up to 32); 0 when it does not fit.
///
/// # Safety
/// `basename` is a NUL-terminated string, `size` valid for writes.
pub unsafe fn lbFile_LoadInto(basename: *const c_char, dst: *mut c_void, cap: usize, size: *mut usize) -> i32 {
    let n = lbFileGetSize(basename);
    if dst.is_null() || round_up_32(n) > cap {
        return 0;
    }
    *size = n;
    cancel = false;
    lbFile_80016580(basename, dst, size, Some(lbFile_8001615C), 0);
    waitForDisc();
    1
}

/// `lbFile_800168A0`: true when `basename` was already loaded by lbdvd's preload cache (`*dst` set to it),
/// else loads it into a new block of heap `heap_id`.
///
/// # Safety
/// As `lbFile_80016760`.
pub unsafe fn lbFile_800168A0(heap_id: i32, basename: *const c_char, dst: *mut *mut c_void, size: *mut usize) -> bool {
    *dst = ext::lbDvd_8001819C(basename) as *mut c_void;
    if !(*dst).is_null() {
        *size = lbFileGetSize(basename);
        true
    } else {
        cancel = false;
        lbFile_80016760_inline(heap_id, basename, dst, size);
        false
    }
}

/// Current value of `cancel` (for the state oracle).
pub fn cancel_flag() -> bool {
    unsafe { cancel }
}
/// The static name buffer (for the state oracle).
pub fn result_buffer() -> [u8; MAX_FILENAME_LENGTH] {
    unsafe { RESULT }
}
/// Sets `cancel` (for the state oracle).
pub fn set_cancel(v: bool) {
    unsafe { cancel = v }
}
