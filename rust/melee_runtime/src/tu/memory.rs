//! Native memory.c, including the PC heap census. Host allocation and diagnostics
//! remain explicit hooks; addresses are native pointers and only caller log IDs
//! retain the C u32 truncation.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use core::ffi::c_void;

pub static mut hsd_trace: i32 = -1;
pub static mut hsd_allocs: i32 = 0;
pub static mut hsd_frees: i32 = 0;
pub static mut hsd_live: i32 = 0;
pub static mut hsd_big_size: i32 = 0;
pub static mut hsd_big_count: i32 = 0;
pub static mut hist_size: [i32; 12] = [0; 12];
pub static mut hist_hits: [i32; 12] = [0; 12];
pub static mut caller_pc: [u32; 8] = [0; 8];
pub static mut caller_hits: [i32; 8] = [0; 8];

pub mod ext {
    use super::*;
    macro_rules! hook {
        ($name:ident, $slot:ident, ($($arg:ident:$ty:ty),*) -> $ret:ty) => {
            pub static mut $slot: Option<unsafe fn($($ty),*) -> $ret> = None;
            pub unsafe fn $name($($arg:$ty),*) -> $ret {
                $slot.expect(concat!("TODO ", stringify!($name)))($($arg),*)
            }
        };
    }
    hook!(HSD_GetHeap, HSD_GetHeap_hook, () -> i32);
    hook!(OSAllocFromHeap, OSAllocFromHeap_hook, (heap:i32, size:u32) -> *mut c_void);
    hook!(OSFreeToHeap, OSFreeToHeap_hook, (heap:i32, ptr:*mut c_void) -> ());
    hook!(PcTraceHeapEnabled, PcTraceHeapEnabled_hook, () -> i32);
    hook!(Snap_NoteMem, Snap_NoteMem_hook, (size:i32, ptr:*mut c_void, caller:u32, freeing:i32) -> ());
    // The host supplies the caller token: stable Rust cannot portably read a
    // machine return address. It only enters PC diagnostic/snapshot tracing.
    hook!(__builtin_return_address, return_address_hook, (depth:i32) -> *mut c_void);
    hook!(OSReport, OSReport_hook, (format:&str, args:&[i64]) -> ());
}

pub unsafe fn caller_note(pc: u32) {
    let mut worst = 0;
    for i in 0..8 {
        if caller_pc[i] == pc {
            caller_hits[i] = caller_hits[i].wrapping_add(1);
            return;
        }
        if caller_hits[i] < caller_hits[worst] { worst = i; }
    }
    caller_pc[worst] = pc;
    caller_hits[worst] = 1;
}
pub unsafe fn caller_dump() {
    for i in 0..8 {
        if caller_hits[i] > 64 {
            ext::OSReport("heap:   from 0x%08X x%d\n", &[caller_pc[i] as i64, caller_hits[i] as i64]);
        }
    }
}
pub unsafe fn hist_note(size: i32) {
    let mut worst = 0;
    for i in 0..12 {
        if hist_size[i] == size {
            hist_hits[i] = hist_hits[i].wrapping_add(1);
            return;
        }
        if hist_hits[i] < hist_hits[worst] { worst = i; }
    }
    hist_size[worst] = size;
    hist_hits[worst] = 1;
}
pub unsafe fn hist_dump() {
    for i in 0..12 {
        if hist_hits[i] > 64 {
            ext::OSReport("heap:   size %5d x %d\n", &[hist_size[i] as i64, hist_hits[i] as i64]);
        }
    }
}
pub unsafe fn hsd_note(size: i32, freeing: i32) {
    if hsd_trace < 0 { hsd_trace = ext::PcTraceHeapEnabled(); }
    if hsd_trace == 0 { return; }
    if freeing != 0 {
        hsd_frees = hsd_frees.wrapping_add(1);
        return;
    }
    hsd_allocs = hsd_allocs.wrapping_add(1);
    hist_note(size);
    hsd_live = hsd_live.wrapping_add(size);
    if size >= 2048 {
        if size == hsd_big_size { hsd_big_count = hsd_big_count.wrapping_add(1); }
        else { hsd_big_size = size; hsd_big_count = 1; }
    }
    if hsd_allocs % 512 == 0 {
        ext::OSReport("heap: %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
            &[hsd_allocs as i64, hsd_frees as i64, (hsd_live / 1024) as i64, hsd_big_size as i64, hsd_big_count as i64]);
    }
}
pub unsafe fn HSD_Free(ptr: *mut c_void) {
    let caller = ext::__builtin_return_address(0) as usize as u32;
    ext::Snap_NoteMem(0, ptr, caller, 1);
    hsd_note(0, 1);
    let heap = ext::HSD_GetHeap();
    ext::OSFreeToHeap(heap, ptr);
}
pub unsafe fn HSD_MemAlloc(size: i32) -> *mut c_void {
    if size <= 0 { return core::ptr::null_mut(); }
    let heap = ext::HSD_GetHeap();
    let adr = ext::OSAllocFromHeap(heap, size as u32);
    let caller = ext::__builtin_return_address(0) as usize as u32;
    ext::Snap_NoteMem(size, adr, caller, 0);
    hsd_note(size, 0);
    caller_note(ext::__builtin_return_address(0) as usize as u32);
    if adr.is_null() {
        ext::OSReport("heap: EXHAUSTED on %d bytes after %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
            &[size as i64, hsd_allocs as i64, hsd_frees as i64, (hsd_live / 1024) as i64, hsd_big_size as i64, hsd_big_count as i64]);
        hist_dump(); caller_dump();
    }
    assert!(!adr.is_null(), "HSD_MemAlloc: adr");
    adr
}
