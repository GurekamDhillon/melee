//! src/melee/lb/lbmemory.c: the handle allocator behind the fixed heaps of lbheap.c (ARAM and the
//! persistent main-RAM heaps): 131 child handles and 6 heap handles in one static `Allocator`, free lists
//! threaded through them, a best-fit allocator over the gaps of a heap's sorted child list, and the
//! compaction pass (`lbMemory_8001529C`) that slides every block down, by DevCom (ARAM) or by a timer
//! driven memcpy (RAM).
//!
//! Addresses in the handles are 32-bit quantities (`(u32) iter->lo`), as on the GameCube; the port keeps
//! those casts. Checked against the C (compiled natively with the allocator at a fixed address below 4 GB)
//! by a state oracle: tests/tu_lbheap.rs over gen/turef/tu_lbheap_ref.c.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Callees in TUs that are not ported (dolphin ar/os, baselib devcom); the heap diagnostics default to
/// this crate's lbheap port.
pub mod ext {
    use super::*;
    use crate::tu::lbext::lb_hook;
    pub use crate::tu::lbext::{OSReport, OsArg, __assert};
    // BEGIN GENERATED ext
    lb_hook!(H_ARAlloc, ARAlloc, (a: u32) -> u32);
    lb_hook!(H_ARFree, ARFree, (a: *mut u32) -> u32);
    lb_hook!(H_ARGetSize, ARGetSize, () -> u32);
    lb_hook!(H_OSCreateAlarm, OSCreateAlarm, (a: *mut OSAlarm));
    lb_hook!(H_OSSetAlarm, OSSetAlarm, (a: *mut OSAlarm, b: i64, c: OSAlarmHandler));
    lb_hook!(H_HSD_DevComRequest, HSD_DevComRequest, (a: i32, b: usize, c: usize, d: usize, e: i32, f: i32, g: HSD_DevComCallback, h: usize) -> i32);
    lb_hook!(H_OSDisableInterrupts, OSDisableInterrupts, () -> i32);
    lb_hook!(H_OSRestoreInterrupts, OSRestoreInterrupts, (a: i32) -> i32);
    // END GENERATED ext
}

const FILE: &str = "src/melee/lb/lbmemory.c";

/// The heap compaction's one-shot RAM-copy alarm (`start_ram_copy` / `fn_80015184`: `OSSetAlarm(3 ms)`). There is no
/// timer interrupt here, so the armed alarm waits in `COPY_ALARM` until `lbMemory_FireCopyAlarm` runs it: the engine's
/// per-frame alarm tick, and the spins that wait on the compaction (lbDvd_800189EC waiting for a preload on the heap
/// being compacted) where the console's alarm interrupt would fire. Without it the compaction never finishes, the
/// heap stays `persistent_heap` and the preload wait spins forever (Training's stage load).
pub static mut COPY_ALARM: (*mut OSAlarm, OSAlarmHandler) = (core::ptr::null_mut(), None);

/// `OSSetAlarm` for this TU's copy alarm: arm it (see `COPY_ALARM`).
pub fn lbMemory_ArmCopyAlarm(alarm: *mut OSAlarm, _ticks: i64, handler: OSAlarmHandler) {
    unsafe { COPY_ALARM = (alarm, handler); }
}

/// Fires the armed copy alarm, if any (one 3 ms period elapsed). Returns whether one fired.
pub fn lbMemory_FireCopyAlarm() -> bool {
    unsafe {
        let (alarm, handler) = COPY_ALARM;
        let Some(h) = handler else { return false };
        COPY_ALARM = (core::ptr::null_mut(), None);
        h(alarm, core::ptr::null_mut());
        true
    }
}

// TU-local records are generated from the C: `Handle` is a heap (lbmemory.h), `HSD_AllocEntry` one of its blocks
// (upstream #3547 split the two; before it both were the port's `Handle`).
pub use melee_types::{HSD_AllocEntry, LBMgr, Allocator};

/// `struct Allocator lbMemory_804318B0` (zero initialised).
pub static mut lbMemory_804318B0: Allocator = unsafe { core::mem::zeroed() };

#[inline]
fn a() -> &'static mut Allocator {
    unsafe { &mut *addr_of_mut!(lbMemory_804318B0) }
}

/// `PUSH_HANDLE(list, handle)`.
#[inline]
unsafe fn push_handle(list: *mut *mut Handle, handle: *mut Handle) {
    (*handle).next = *list;
    *list = handle;
}
/// `PUSH_HANDLE` over the block free list (`HSD_AllocEntry`).
#[inline]
unsafe fn push_block(list: *mut *mut HSD_AllocEntry, block: *mut HSD_AllocEntry) {
    (*block).next = *list;
    *list = block;
}
/// `POP_HANDLE(list, handle)`.
#[inline]
unsafe fn pop_handle(list: *mut *mut Handle) -> *mut Handle {
    let handle = *list;
    *list = (*handle).next;
    handle
}
/// `POP_HANDLE` over the block free list.
#[inline]
unsafe fn pop_block(list: *mut *mut HSD_AllocEntry) -> *mut HSD_AllocEntry {
    let block = *list;
    *list = (*block).next;
    block
}
#[inline]
fn u32_of(p: *mut c_void) -> u32 {
    p as usize as u32
}
/// `(void*) (u32) v`.
#[inline]
fn ptr_of(v: u32) -> *mut c_void {
    v as usize as *mut c_void
}
/// `OSMillisecondsToTicks(msec)`: `msec * (OS_TIMER_CLOCK / 1000)` in u32 arithmetic, widened to OSTime.
#[inline]
fn os_milliseconds_to_ticks(msec: i32) -> OSTime {
    (msec as u32).wrapping_mul(crate::tu::lbext::OS_BUS_CLOCK / 4 / 1000) as OSTime
}
/// `OSRoundUp32B`.
#[inline]
fn round_up_32b(x: u32) -> u32 {
    x.wrapping_add(31) & !31
}
/// `(HSD_AllocEntry*) &h->blocks`: the list head seen as a block whose `next` is the first child.
#[inline]
unsafe fn head_of(h: *mut Handle) -> *mut HSD_AllocEntry {
    addr_of_mut!((*h).blocks) as *mut HSD_AllocEntry
}

/// `lbMemory_80014E24`: a new heap handle over `[arena_lo, arena_hi)`.
///
/// # Safety
/// The allocator must have been set up by `lbMemory_8001564C`.
pub unsafe fn lbMemory_80014E24(arena_lo: *mut c_void, arena_hi: *mut c_void) -> *mut Handle {
    melee_core::ct!("lbMemory_80014E24");
    if a().free_heap.is_null() {
        ext::__assert(FILE, 66, "_p(free_heap)");
    }
    if u32_of(arena_lo) < 0x8000_0000 && u32_of(arena_hi) < 0x8000_0000 {
        if !(u32_of(arena_lo) >= a().a_arenaLo as u32 && u32_of(arena_hi) <= a().a_arenaHi as u32) {
            ext::__assert(FILE, 69, "(u32)arenaLo >= (u32)_p(a_arenaLo) && (u32)arenaHi <= (u32)_p(a_arenaHi)");
        }
    }
    let h = pop_handle(addr_of_mut!(a().free_heap));
    (*h).next = null_mut();
    (*h).lo = arena_lo;
    (*h).hi = arena_hi;
    (*h).blocks = null_mut();
    h
}

/// `lbMemory_80014EEC`: releases a heap handle and all its children.
///
/// # Safety
/// `handle` is a heap handle of this allocator (or null, which asserts).
pub unsafe fn lbMemory_80014EEC(handle: *mut Handle) {
    melee_core::ct!("lbMemory_80014EEC");
    if handle.is_null() {
        ext::__assert(FILE, 84, "handle");
    }
    let mut iter = (*handle).blocks;
    while !iter.is_null() {
        let tmp_next = (*iter).next;
        push_block(addr_of_mut!(a().free_mem), iter);
        iter = tmp_next;
        a().num_allocs -= 1;
    }
    push_handle(addr_of_mut!(a().free_heap), handle);
}

/// `lbMemory_80014F7C`: the free bytes of a heap (the sum of the gaps between its blocks).
///
/// # Safety
/// `h` is a heap handle of this allocator.
pub unsafe fn lbMemory_80014F7C(h: *mut Handle) -> u32 {
    melee_core::ct!("lbMemory_80014F7C");
    let mut r4 = u32_of((*h).lo);
    let mut iter = head_of(h);
    let mut sum: u32 = 0;
    loop {
        iter = (*iter).next;
        let r0 = u32_of(if !iter.is_null() { (*iter).addr } else { (*h).hi });
        sum = sum.wrapping_add(r0.wrapping_sub(r4));
        if iter.is_null() {
            break;
        }
        r4 = u32_of((*iter).addr).wrapping_add((*iter).size as u32);
    }
    sum
}

/// TARGET_PC: the largest free gap of a heap (what one allocation can get).
///
/// # Safety
/// `h` is a heap handle of this allocator.
pub unsafe fn lbMemory_LargestFree(h: *mut Handle) -> u32 {
    let mut largest: u32 = 0;
    let mut it = head_of(h);
    let mut from = (*h).lo;
    loop {
        let to = if !(*it).next.is_null() { (*(*it).next).addr } else { (*h).hi };
        largest = largest.max(u32_of(to).wrapping_sub(u32_of(from)));
        if (*it).next.is_null() {
            break;
        }
        it = (*it).next;
        from = ptr_of(u32_of((*it).addr).wrapping_add((*it).size as u32));
    }
    largest
}

/// `lbMemory_80014FC8`: best-fit allocation of `size` bytes (rounded up to 32) from heap `arg0`; returns the
/// block entry (its `addr` is the block).
///
/// # Safety
/// `arg0` is a heap handle of this allocator.
pub unsafe fn lbMemory_80014FC8(arg0: *mut Handle, size: usize) -> *mut HSD_AllocEntry {
    melee_core::ct!("lbMemory_80014FC8");
    let mut least_leftover: u32 = 0x4000_0000;
    let mut lo: *mut c_void = null_mut();
    if a().free_mem.is_null() {
        ext::__assert(FILE, 124, "_p(free_mem)");
    }
    let size = round_up_32b(size as u32);
    let mut start = (*arg0).lo;
    let mut iter = head_of(arg0);
    let mut memp_kouho: *mut HSD_AllocEntry = null_mut();

    loop {
        let end = if !(*iter).next.is_null() { (*(*iter).next).addr } else { (*arg0).hi };
        let available_space = u32_of(end).wrapping_sub(u32_of(start));
        if available_space >= size {
            let leftover = available_space.wrapping_sub(size);
            if leftover <= least_leftover {
                least_leftover = leftover;
                lo = start;
                memp_kouho = iter;
            }
        }
        if (*iter).next.is_null() {
            break;
        }
        iter = (*iter).next;
        start = ptr_of(u32_of((*iter).addr).wrapping_add((*iter).size as u32));
    }
    // TARGET_PC failure-path diagnostic: only runs when the allocation is about to assert.
    if memp_kouho.is_null() {
        let mut name = [0u8; 16];
        let heap = super::lbheap::lbHeap_IdOfHandle(arg0 as *mut c_void, name.as_mut_ptr() as *mut core::ffi::c_char, 16);
        let mut largest: u32 = 0;
        let mut it = head_of(arg0);
        let mut from = (*arg0).lo;
        loop {
            let to = if !(*it).next.is_null() { (*(*it).next).addr } else { (*arg0).hi };
            if u32_of(to).wrapping_sub(u32_of(from)) > largest {
                largest = u32_of(to).wrapping_sub(u32_of(from));
            }
            if (*it).next.is_null() {
                break;
            }
            it = (*it).next;
            from = ptr_of(u32_of((*it).addr).wrapping_add((*it).size as u32));
        }
        static mut REPORTED: i32 = 0;
        let not_lb = b"not an lbHeap\0";
        ext::OSReport(
            "lbMemory_80014FC8: ALLOC_FAIL heap %d (%s) size=0x%X largest free=0x%X lo=0x%X hi=0x%X\n",
            &[
                ext::OsArg::I(heap),
                ext::OsArg::S(if heap >= 0 { name.as_ptr() as *const core::ffi::c_char } else { not_lb.as_ptr() as *const core::ffi::c_char }),
                ext::OsArg::U(size),
                ext::OsArg::U(largest),
                ext::OsArg::U(u32_of((*arg0).lo)),
                ext::OsArg::U(u32_of((*arg0).hi)),
            ],
        );
        if REPORTED == 0 {
            REPORTED = 1;
            super::lbheap::lbHeap_80015DF8();
        }
    }
    if memp_kouho.is_null() {
        ext::__assert(FILE, 186, "memp_kouho");
    }
    let result = pop_block(addr_of_mut!(a().free_mem));
    (*result).size = size as usize;
    (*result).addr = lo;
    (*result).next = (*memp_kouho).next;
    (*memp_kouho).next = result;

    a().num_allocs += 1;
    if a().num_allocs > a().max_num_allocs {
        a().max_num_allocs = a().num_allocs;
    }
    result
}

/// `lbMemFreeToHeap`: frees the child of heap `h` whose block starts at `arg1`.
///
/// # Safety
/// `h` is a heap handle of this allocator.
pub unsafe fn lbMemFreeToHeap(h: *mut Handle, arg1: *mut c_void) {
    melee_core::ct!("lbMemFreeToHeap");
    let mut handle = (*h).blocks;
    let mut r6 = head_of(h);

    while !handle.is_null() {
        if (*handle).addr == arg1 {
            (*r6).next = (*handle).next;
            push_block(addr_of_mut!(a().free_mem), handle);
            a().num_allocs -= 1;
            return;
        }
        r6 = handle;
        handle = (*handle).next;
    }
    ext::OSReport("[LbMem] Error: lbMemFreeToHeap %x.\n", &[ext::OsArg::U(arg1 as usize as u32)]);
    ext::__assert(FILE, 219, "0");
}

/// `fn_80015184` (the timer handler) minus the entry: one step of the RAM copy.
pub fn fn_80015184_body() {
    melee_core::ct!("fn_80015184_body");
    unsafe {
        let p = addr_of_mut!(a().mgr);
        if (*p).size == 0 {
            ext::__assert(FILE, 230, "p->size");
        }
        let temp_r6 = (*p).offset;
        let temp_r3_2 = (*p).size.wrapping_sub(temp_r6);
        let mut var_r30 = temp_r3_2;
        if temp_r3_2 > 0x19000 {
            var_r30 = 0x19000;
        }
        core::ptr::copy((*p).src.add(temp_r6 as usize), (*p).dst.add(temp_r6 as usize), var_r30 as usize);
        (*p).offset = (*p).offset.wrapping_add(var_r30);
        if (*p).offset == (*p).size {
            (*p).size = 0;
            // the callback is (or may be) one of the `C-unwind` entry points above
            let cb: unsafe extern "C-unwind" fn(i32, usize, *mut c_void, i32) = core::mem::transmute((*p).cb.unwrap());
            cb(0, (*p).cb_arg, null_mut(), 0);
            return;
        }
        ext::OSCreateAlarm(addr_of_mut!((*p).alarm));
        ext::OSSetAlarm(addr_of_mut!((*p).alarm), os_milliseconds_to_ticks(3), fn_80015184_handler());
    }
}

/// `fn_80015184`: the OSAlarm handler of the RAM copy.
/// `fn_80015184` as the `OSAlarmHandler` it is stored as. The entry points are `extern "C-unwind"` (same calling
/// convention): an `__assert` inside them unwinds to the caller, the way the C driver's longjmp does.
fn fn_80015184_handler() -> OSAlarmHandler {
    Some(unsafe { core::mem::transmute::<unsafe extern "C-unwind" fn(*mut OSAlarm, *mut OSContext), unsafe extern "C" fn(*mut OSAlarm, *mut OSContext)>(fn_80015184) })
}

/// `lbMemory_80015320` as the DevCom callback it is stored as (see `fn_80015184_handler`).
fn lbMemory_80015320_callback() -> HSD_DevComCallback {
    Some(unsafe { core::mem::transmute::<unsafe extern "C-unwind" fn(i32, usize, *mut c_void, i32), unsafe extern "C" fn(i32, usize, *mut c_void, i32)>(lbMemory_80015320) })
}

pub unsafe extern "C-unwind" fn fn_80015184(_alarm: *mut OSAlarm, _context: *mut OSContext) {
    fn_80015184_body();
}

/// `lbMemory_8001529C`: starts the compaction of heap `h`; `arg1(arg2)` runs when every block has been moved.
/// Returns 1 when a move was started (the continuation is `lbMemory_80015320`), 0 when nothing moves.
///
/// # Safety
/// `h` is a heap handle of this allocator.
pub unsafe fn lbMemory_8001529C(h: *mut Handle, arg1: Option<unsafe extern "C" fn(u32)>, arg2: u32) -> u32 {
    melee_core::ct!("lbMemory_8001529C");
    a().compact_cb = arg1;
    a().compact_arg = arg2;
    a().compact_cursor = (*h).lo as usize;

    let r7 = addr_of_mut!(a().compact_cursor);
    let mut iter = (*h).blocks;
    while !iter.is_null() {
        let lo = (*iter).addr;
        if lo as usize != *r7 {
            lbMemory_80015320_body(0, iter as usize, null_mut(), 0);
            return 1;
        }
        *r7 = u32_of(lo).wrapping_add((*iter).size as u32) as usize;
        iter = (*iter).next;
    }
    0
}

/// `start_ram_copy`: queue the timer driven copy of `size` bytes from `old` to `current`; `next` is the
/// continuation's argument.
unsafe fn start_ram_copy(old: u32, current: u32, size: u32, next: *mut HSD_AllocEntry) {
    let p = addr_of_mut!(a().mgr);
    let enabled = ext::OSDisableInterrupts();
    if (*p).size != 0 {
        ext::__assert(FILE, 276, "!p->size");
    }
    (*p).src = old as usize as *mut u8;
    (*p).dst = current as usize as *mut u8;
    (*p).size = size;
    (*p).offset = 0;
    (*p).cb_arg = next as usize;
    (*p).cb = lbMemory_80015320_callback();
    ext::OSRestoreInterrupts(enabled);
    ext::OSCreateAlarm(addr_of_mut!((*p).alarm));
    ext::OSSetAlarm(addr_of_mut!((*p).alarm), os_milliseconds_to_ticks(3), fn_80015184_handler());
}

/// `lbMemory_80015320`: the compaction state machine, one block per step. `_handle` is the next block to move
/// (an `HSD_AllocEntry*`), 0 when done.
pub fn lbMemory_80015320_body(_arg0: i32, _handle: usize, _arg2: *mut c_void, cancelflag: i32) {
    melee_core::ct!("lbMemory_80015320_body");
    unsafe {
        let handle = _handle as *mut HSD_AllocEntry;
        let currentp = addr_of_mut!(a().compact_cursor);
        let current = a().compact_cursor as u32;
        let null_or_old: *mut c_void = null_mut();

        if cancelflag != 0 {
            ext::__assert(FILE, 303, "!cancelflag");
        }

        if !handle.is_null() {
            let loaded_old = (*handle).addr;
            let old = loaded_old;
            if old != ptr_of(current) {
                let null_or_old = old;
                (*handle).addr = ptr_of(current);
                *currentp = u32_of((*handle).addr).wrapping_add((*handle).size as u32) as usize;
                let copy_src = null_or_old;

                if u32_of((*handle).addr) < 0x8000_0000 {
                    ext::HSD_DevComRequest(
                        0,
                        u32_of(copy_src) as usize,
                        current as usize,
                        round_up_32b((*handle).size as u32) as usize,
                        0x1B,
                        1,
                        lbMemory_80015320_callback(),
                        (*handle).next as usize,
                    );
                    return;
                } else {
                    start_ram_copy(u32_of(copy_src), current, round_up_32b((*handle).size as u32), (*handle).next);
                    return;
                }
            }
            *currentp = u32_of(old).wrapping_add((*handle).size as u32) as usize;
            lbMemory_80015320_body(0, (*handle).next as usize, null_or_old, 0);
            return;
        }

        (a().compact_cb.unwrap())(a().compact_arg);
    }
}

/// `lbMemory_80015320`: the DevCom completion callback of the compaction.
pub unsafe extern "C-unwind" fn lbMemory_80015320(arg0: i32, handle: usize, arg2: *mut c_void, cancelflag: i32) {
    lbMemory_80015320_body(arg0, handle, arg2, cancelflag);
}

/// `lbMemory_800154BC`: the ARAM arena bounds.
///
/// # Safety
/// Both pointers valid for writes.
pub unsafe fn lbMemory_800154BC(arena_lo: *mut usize, arena_hi: *mut usize) {
    melee_core::ct!("lbMemory_800154BC");
    *arena_lo = a().a_arenaLo;
    *arena_hi = a().a_arenaHi;
}

/// `lbMemory_800154D4`: creates the ARAM heap handle.
///
/// # Safety
/// As `lbMemory_80014E24`.
pub unsafe fn lbMemory_800154D4(arena_lo: *mut c_void, arena_hi: *mut c_void) -> *mut Handle {
    melee_core::ct!("lbMemory_800154D4");
    a().aram_heap = lbMemory_80014E24(arena_lo, arena_hi);
    a().aram_heap
}

/// `lbMemory_800155A4`: releases the ARAM heap handle.
///
/// # Safety
/// The ARAM heap handle must exist.
pub unsafe fn lbMemory_800155A4() {
    melee_core::ct!("lbMemory_800155A4");
    let handle = a().aram_heap;
    if handle.is_null() {
        ext::__assert(FILE, 352, "handle");
    }
    let r5 = addr_of_mut!(a().free_mem);
    let mut iter = (*handle).blocks;
    while !iter.is_null() {
        let tmp_next = (*iter).next;
        push_block(r5, iter);
        iter = tmp_next;
        a().num_allocs -= 1;
    }
    push_handle(addr_of_mut!(a().free_heap), handle);
    a().aram_heap = null_mut();
}

/// `lbMemory_8001564C`: initialises the allocator (ARAM bounds, the free lists, the ARAM heap handle).
///
/// # Safety
/// Not reentrant.
pub unsafe fn lbMemory_8001564C() {
    melee_core::ct!("lbMemory_8001564C");
    let mut freed_size: u32 = 0;

    a().a_arenaLo = ext::ARAlloc(0x20) as usize;
    ext::ARFree(&mut freed_size);
    // TARGET_PC: the port's ARAM is 32 MB; no 16 MB cap
    a().a_arenaHi = ext::ARGetSize() as usize;

    a().free_mem = addr_of_mut!(a().mem[0]);
    let n = a().mem.len();
    for i in 0..n - 1 {
        a().mem[i].next = addr_of_mut!(a().mem[i + 1]);
    }
    a().mem[n - 1].next = null_mut();

    a().max_num_allocs = 0;
    a().num_allocs = 0;
    a().free_heap = addr_of_mut!(a().heap[0]);
    let n = a().heap.len();
    for i in 0..n - 1 {
        a().heap[i].next = addr_of_mut!(a().heap[i + 1]);
    }
    a().heap[n - 1].next = null_mut();
    a().aram_heap = null_mut();
    {
        let hi = a().a_arenaHi as *mut c_void;
        let lo = a().a_arenaLo as *mut c_void;
        lbMemory_800154D4(lo, hi);
    }
    a().mgr.size = 0;
}
