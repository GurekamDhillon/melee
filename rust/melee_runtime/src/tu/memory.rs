//! Native memory.c, including the PC heap census. Host allocation and diagnostics
//! remain explicit hooks; addresses are native pointers and only caller log IDs
//! retain the C u32 truncation.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use core::ffi::c_void;
use super::lbext::{OSReport, OsArg};

/// `__builtin_return_address(depth)`: the host supplies the caller token (stable Rust cannot portably read a
/// machine return address). It only enters PC diagnostic / snapshot tracing.
pub static mut return_address_hook: Option<fn(i32) -> *mut c_void> = None;
unsafe fn return_address(depth: i32) -> *mut c_void {
    match return_address_hook { Some(f) => f(depth), None => core::ptr::null_mut() }
}

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
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_GetHeap() -> i32;
    fn OSAllocFromHeap(a0: i32, a1: u32) -> *mut c_void;
    fn OSFreeToHeap(a0: i32, a1: *mut c_void);
    fn PcTraceHeapEnabled() -> i32;
    fn Snap_NoteMem(a0: i32, a1: *mut c_void, a2: u32, a3: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

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
            OSReport("heap:   from 0x%08X x%d\n", &[OsArg::U(caller_pc[i]), OsArg::I(caller_hits[i])]);
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
            OSReport("heap:   size %5d x %d\n", &[OsArg::I(hist_size[i]), OsArg::I(hist_hits[i])]);
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
        OSReport("heap: %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
            &[OsArg::I(hsd_allocs), OsArg::I(hsd_frees), OsArg::I(hsd_live / 1024), OsArg::I(hsd_big_size), OsArg::I(hsd_big_count)]);
    }
}
pub unsafe fn HSD_Free(ptr: *mut c_void) {
    let caller = return_address(0) as usize as u32;
    ext::Snap_NoteMem(0, ptr, caller, 1);
    hsd_note(0, 1);
    let heap = ext::HSD_GetHeap();
    ext::OSFreeToHeap(heap, ptr);
}
pub unsafe fn HSD_MemAlloc(size: i32) -> *mut c_void {
    if size <= 0 { return core::ptr::null_mut(); }
    let heap = ext::HSD_GetHeap();
    let adr = ext::OSAllocFromHeap(heap, size as u32);
    let caller = return_address(0) as usize as u32;
    ext::Snap_NoteMem(size, adr, caller, 0);
    hsd_note(size, 0);
    caller_note(return_address(0) as usize as u32);
    if adr.is_null() {
        OSReport("heap: EXHAUSTED on %d bytes after %d allocs %d frees, live %d KB, top repeat %d bytes x%d\n",
            &[OsArg::I(size), OsArg::I(hsd_allocs), OsArg::I(hsd_frees), OsArg::I(hsd_live / 1024), OsArg::I(hsd_big_size), OsArg::I(hsd_big_count)]);
        hist_dump(); caller_dump();
    }
    assert!(!adr.is_null(), "HSD_MemAlloc: adr");
    adr
}
