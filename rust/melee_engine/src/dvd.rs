//! The dolphin DVD seam on the disc image: path -> entry number, file info, and the baselib `host_dev_com_request`
//! the `lbFile` loaders queue their reads on. Reads complete synchronously (the C pumps the queue until the
//! completion callback has run, so an immediate callback is the same observable order).

use crate::host::Host;
use core::ffi::{c_char, c_void, CStr};
use melee_types::{DVDFileInfo, HSD_DevComCallback};

/// `DVDConvertPathToEntrynum`.
pub fn DVDConvertPathToEntrynum(path: *const c_char) -> i32 {
    let p = unsafe { CStr::from_ptr(path) }.to_string_lossy();
    let n = Host::get().disc.entry_for_path(&p);
    if n < 0 {
        eprintln!("engine: disc has no file {p}");
    } else if std::env::var_os("MELEE_ENGINE_LOG").is_some() {
        eprintln!("engine: path {p} = entry {n}");
    }
    n
}

/// `DVDFastOpen`: fills `startAddr` (the entry number stands in for the disc offset) and `length`.
pub fn DVDFastOpen(entry: i32, info: *mut DVDFileInfo) -> i32 {
    match Host::get().disc.entry_len(entry) {
        Some(len) => {
            unsafe {
                (*info).startAddr = entry as u32;
                (*info).length = len;
            }
            1
        }
        None => 0,
    }
}

/// `DVDClose`.
pub fn DVDClose(_info: *mut DVDFileInfo) -> i32 {
    1
}

pub fn OSDisableInterrupts() -> i32 {
    1
}
pub fn OSRestoreInterrupts(_level: i32) -> i32 {
    0
}
/// A completion `host_dev_com_deferred` holds back: (callback, args, buffer).
type Completion = (unsafe extern "C" fn(i32, usize, *mut c_void, i32), usize, *mut c_void);
static mut DEFERRED: Vec<Completion> = Vec::new();

/// `host_dev_com_request` for the movie player (lb/lbmthp.c), whose reads must complete *after* the request returns, as
/// the disc interrupt does on hardware: `fn_8001F13C` sets `unk_110` (a read in flight) after `HSD_DevComRequest`, and the
/// completion `fn_8001E910` clears it and refills the frame ring. A synchronous callback ran before the flag was set (the
/// movie stalled on its first frame) and nested one call per buffered frame. The copy is immediate; the callback runs in
/// [`run_deferred_completions`], once per frame after the alarms.
pub fn host_dev_com_deferred(file: i32, src: usize, dst: usize, size: usize, ty: i32, pri: i32, cb: HSD_DevComCallback, args: usize) -> i32 {
    let r = host_dev_com_request(file, src, dst, size, ty, pri, None, args);
    if let Some(f) = cb {
        // the buffer host_dev_com_request hands its callback: the main-RAM destination, or null for ARAM
        let h = Host::get();
        let in_ram = dst >= h.os.mem.base as usize && dst < h.os.mem.end_ptr() as usize;
        let buf = if in_ram || ty == 0x21 { dst as *mut c_void } else { core::ptr::null_mut() };
        unsafe { (*core::ptr::addr_of_mut!(DEFERRED)).push((f, args, buf)) };
    }
    r
}

/// Completes the reads [`host_dev_com_deferred`] queued (the ones a completion issues wait for the next frame).
pub fn run_deferred_completions() {
    let batch = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(DEFERRED)) };
    for (f, a, b) in batch {
        unsafe { f(0, a, b, 0) };
    }
}

/// `host_pump_disc`: pumps the disc queue; requests already completed.
pub fn host_pump_disc() {}

/// `host_dev_com_request(file, src, dst, size, type, pri, callback, args)`: copy `size` bytes from offset `src`
/// of disc entry `file` to `dst` (main RAM address for `dst >= 0x8000_0000`-style types 0x21; else an ARAM
/// offset), zero-filling past the end of the file (the C rounds `size` up to 32), then run the callback.
pub fn host_dev_com_request(file: i32, src: usize, dst: usize, size: usize, ty: i32, _pri: i32, cb: HSD_DevComCallback, args: usize) -> i32 {
    let h = Host::get();
    if std::env::var_os("MELEE_ENGINE_LOG").is_some() {
        eprintln!("engine: dev_com file {file} src {src:#x} dst {dst:#x} size {size:#x} type {ty:#x}");
        h.disc.read_entry(file, 0, 16).map(|b| eprintln!("engine:   head {:02x?} flen {:#x}", b, h.disc.entry_len(file).unwrap_or(0))).ok();
    }
    let flen = h.disc.entry_len(file).unwrap_or_else(|| panic!("host_dev_com_request: entry {file} is not a file")) as usize;
    let want = size.min(flen.saturating_sub(src));
    let bytes = h.disc.read_entry(file, src as u32, want).unwrap_or_else(|e| panic!("host_dev_com_request: {e}"));
    // lbFile picks the type by `dst >= 0x80000000` (a GC main-RAM address). A native pointer never is, so a
    // destination inside the host's RAM block is main RAM whatever the type says; anything else is an ARAM offset.
    let in_ram = dst >= h.os.mem.base as usize && dst < h.os.mem.end_ptr() as usize;
    let ty = if in_ram { 0x21 } else { ty };
    let buf: *mut c_void = match ty {
        0x21 => {
            // main RAM: `dst` is the native address of a block the caller owns
            unsafe {
                core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst as *mut u8, bytes.len());
                core::ptr::write_bytes((dst as *mut u8).add(bytes.len()), 0, size - bytes.len());
            }
            dst as *mut c_void
        }
        0x23 => {
            if h.aram.len() < dst + size {
                h.aram.resize(dst + size, 0);
            }
            h.aram[dst..dst + bytes.len()].copy_from_slice(&bytes);
            h.aram[dst + bytes.len()..dst + size].fill(0);
            core::ptr::null_mut()
        }
        _ => panic!("host_dev_com_request: unsupported request type {ty:#x}"),
    };
    if let Some(f) = cb {
        unsafe { f(0, args, buf, 0) };
    }
    0
}
