//! src/melee/lb/lbheap.c: the table of the game's fixed heaps (HSD main heap, ARAM, Seq, Stay, AllM, AllA and,
//! on the PC port, m-ex's Scene heap): where each lives, whether it is transient, creating and destroying
//! them around a scene, allocating from and freeing to one, and the report of what is left.
//!
//! Heap 0 is the HSD main heap (an OSHeap), the others are handle managed by `lbmemory`. Checked against the
//! C by a state oracle (tests/tu_lbheap.rs over gen/turef/tu_lbheap_ref.c, which compiles lbheap.c and
//! lbmemory.c together over a seeded sequence of scene set-ups, allocations, frees and compactions; the
//! callee log, the results and the whole of both modules' state are compared).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Callees in TUs that are not ported (dolphin os/OSAlloc, baselib initialize/memory).
pub mod ext {
    use super::*;
    use crate::tu::lbext::lb_hook;
    pub use crate::tu::lbext::{OSReport, OsArg, __assert};
    // BEGIN GENERATED ext
    lb_hook!(H_OSDestroyHeap, OSDestroyHeap, (a: i32));
    lb_hook!(H_OSCreateHeap, OSCreateHeap, (a: *mut c_void, b: *mut c_void) -> i32);
    lb_hook!(H_HSD_CreateMainHeap, HSD_CreateMainHeap, (a: *mut c_void, b: *mut c_void) -> i32);
    lb_hook!(H_HSD_GetHeap, HSD_GetHeap, () -> i32);
    lb_hook!(H_HSD_SetHeap, HSD_SetHeap, (a: i32));
    lb_hook!(H_HSD_MemAlloc, HSD_MemAlloc, (a: i32) -> *mut c_void = |a| unsafe { crate::tu::memory::HSD_MemAlloc(a) });
    lb_hook!(H_HSD_Free, HSD_Free, (a: *mut c_void) = |a| unsafe { crate::tu::memory::HSD_Free(a) });
    lb_hook!(H_OSCheckHeap, OSCheckHeap, (a: i32) -> i32);
    lb_hook!(H_HSD_GetNextArena, HSD_GetNextArena, (a: *mut *mut c_void, b: *mut *mut c_void) = |a, b| unsafe { crate::tu::initialize::HSD_GetNextArena(a, b) });
    lb_hook!(H_OSAllocFromHeap, OSAllocFromHeap, (a: i32, b: u32) -> *mut c_void);
    lb_hook!(H_OSDumpHeap, OSDumpHeap, (a: i32));
    lb_hook!(H_HSD_ObjAllocReport, HSD_ObjAllocReport, () = || unsafe { crate::tu::objalloc::HSD_ObjAllocReport() });
    lb_hook!(H_OSAllocTrack, OSAllocTrack, (a: i32));
    lb_hook!(H_Env1, Env1, (a: *const c_char) -> i32);
    lb_hook!(H_OSDisableInterrupts, OSDisableInterrupts, () -> i32);
    lb_hook!(H_OSRestoreInterrupts, OSRestoreInterrupts, (a: i32) -> i32);
    // END GENERATED ext
}

/// `LBHEAP_HEAP_COUNT` on TARGET_PC.
pub const LBHEAP_HEAP_COUNT: usize = 7;

pub use melee_types::lbHeap_HeapDesc;

const fn hd(idx: u32, ty: u32, prev_idx: u32, size: u32) -> lbHeap_HeapDesc {
    lbHeap_HeapDesc { idx, r#type: ty, prev_idx, size }
}
/// `lbHeap_803BA380` (TARGET_PC variant: heap 3 +8 MiB, heap 4 16 MiB bigger, AllA bigger, m-ex's Scene heap).
pub static lbHeap_803BA380: [lbHeap_HeapDesc; 6] = [
    hd(2, 1, 6, 0x800),
    hd(3, 1, 2, 0xD1A690),
    hd(4, 2, 6, 0x164B400),
    hd(5, 4, 6, 0x196C800),
    hd(6, 1, 3, 0x20),
    hd(LBHEAP_HEAP_COUNT as u32, 0, 0, 0),
];

/// `lbHeap_803BA448`: the report names (NUL terminated).
pub static lbHeap_803BA448: [&[u8]; LBHEAP_HEAP_COUNT] =
    [b"     Hsd\0", b"    ARAM\0", b"     Seq\0", b"    Stay\0", b"    AllM\0", b"    AllA\0", b"   Scene\0"];

/// `static struct lbHeap_HeapState lbHeap_80431FA0` (zero initialised).
pub static mut lbHeap_80431FA0: lbHeap_HeapState = unsafe { core::mem::zeroed() };

const FILE: &str = "src/melee/lb/lbheap.c";

#[inline]
fn st() -> &'static mut lbHeap_HeapState {
    unsafe { &mut *addr_of_mut!(lbHeap_80431FA0) }
}
/// `&lbHeap_80431FA0.heap_array[i]` (no bounds check, like the C address arithmetic).
#[inline]
unsafe fn heap_ptr(i: usize) -> *mut Heap {
    (addr_of_mut!(lbHeap_80431FA0.heap_array) as *mut Heap).add(i)
}
/// `(Handle*) -1`.
#[inline]
fn minus1() -> *mut Handle {
    usize::MAX as *mut Handle
}

/// `lbHeap_ResetHeap`.
unsafe fn lbHeap_ResetHeap(heap: *mut Heap) {
    (*heap).id = -1;
    (*heap).handle = minus1();
    (*heap).start = 0;
    (*heap).size = 0;
    (*heap).r#type = 1;
    (*heap).transient = 1;
    (*heap).status = LbHeapStatus_Destroy;
}

/// `lbHeap_DestroyOffsetViewIfCreated` on heap `i`.
unsafe fn destroy_if_created(i: usize) {
    let heap = heap_ptr(i);
    if (*heap).status == LbHeapStatus_Create {
        if (*heap).r#type == 0 {
            ext::OSDestroyHeap((*heap).id);
            (*heap).id = -1;
        } else {
            super::lbmemory::lbMemory_80014EEC((*heap).handle);
            (*heap).handle = minus1();
        }
        (*heap).status = LbHeapStatus_Destroy;
    }
}

/// `lbHeap_CreateOffsetViewIfDestroyed` on heap `i`.
unsafe fn create_if_destroyed(i: usize) {
    let heap = heap_ptr(i);
    if (*heap).status == LbHeapStatus_Destroy {
        let lo = (*heap).start as *mut c_void;
        let hi = (*heap).start.wrapping_add((*heap).size as usize) as *mut c_void;
        if (*heap).r#type == 0 {
            (*heap).id = ext::OSCreateHeap(lo, hi);
        } else {
            (*heap).handle = super::lbmemory::lbMemory_80014E24(lo, hi);
        }
        (*heap).status = LbHeapStatus_Create;
    }
}

/// `lbHeap_800158D0`: sets the transient flag of heap `arg0`.
///
/// # Safety
/// `arg0` is a heap index.
pub unsafe fn lbHeap_800158D0(arg0: i32, arg1: i32) {
    melee_core::ct!("lbHeap_800158D0");
    (*heap_ptr(arg0 as usize)).transient = arg1;
}

/// `lbHeap_800158E8`: the transient flag of heap `arg0`.
///
/// # Safety
/// `arg0` is a heap index.
pub unsafe fn lbHeap_800158E8(arg0: i32) -> i32 {
    melee_core::ct!("lbHeap_800158E8");
    (*heap_ptr(arg0 as usize)).transient
}

/// `lbHeap_80015900`: re-lays out the heaps: destroys the transient ones, recomputes the main heap and the
/// ARAM heap around the persistent ones, and recreates the heaps that are not transient.
///
/// # Safety
/// The heaps were set up by `lbHeap_80015F3C`.
pub unsafe fn lbHeap_80015900() {
    melee_core::ct!("lbHeap_80015900");
    // 0 and 1 are reserved for HSD and ARAM
    for i in 2..LBHEAP_HEAP_COUNT {
        if (*heap_ptr(i)).transient == 1 {
            destroy_if_created(i);
        }
    }

    let mut arena_lo = st().arena_lo as usize;
    let mut arena_hi = st().arena_hi as usize;
    let mut aram_lo = st().aram_lo;
    let aram_hi = st().aram_hi;

    for i in 2..LBHEAP_HEAP_COUNT {
        let bounds_heap = heap_ptr(i);
        if (*bounds_heap).transient == 0 {
            match (*bounds_heap).r#type {
                1 => {
                    // `s32 temp_r0 = start + size;` compared/assigned as a sign extended uintptr_t
                    let temp_r0 = (*bounds_heap).start.wrapping_add((*bounds_heap).size as usize) as u32 as i32 as isize as usize;
                    if arena_lo < temp_r0 {
                        arena_lo = temp_r0;
                    }
                }
                2 => {
                    if arena_hi > (*bounds_heap).start {
                        arena_hi = (*bounds_heap).start;
                    }
                }
                4 => {
                    let temp_r0 = (*bounds_heap).start.wrapping_add((*bounds_heap).size as usize) as u32 as i32 as isize as usize;
                    if aram_lo < temp_r0 {
                        aram_lo = temp_r0;
                    }
                }
                _ => {}
            }
        }
    }

    let main_heap = heap_ptr(0);
    (*main_heap).id = ext::HSD_CreateMainHeap(arena_lo as *mut c_void, arena_hi as *mut c_void);
    (*main_heap).start = arena_lo;
    let aram_heap = heap_ptr(1);
    (*main_heap).size = arena_hi.wrapping_sub(arena_lo) as u32;
    (*main_heap).status = LbHeapStatus_Create;
    (*main_heap).r#type = 0;

    super::lbmemory::lbMemory_800155A4();

    (*aram_heap).handle = super::lbmemory::lbMemory_800154D4(aram_lo as *mut c_void, aram_hi as *mut c_void);
    (*aram_heap).start = aram_lo;
    (*aram_heap).size = aram_hi.wrapping_sub(aram_lo) as u32;
    (*aram_heap).status = LbHeapStatus_Create;
    (*aram_heap).r#type = 3;

    for i in 2..LBHEAP_HEAP_COUNT {
        if (*heap_ptr(i)).transient == 0 {
            create_if_destroyed(i);
        }
    }
}

/// `lbHeap_80015BB8`: the status of heap `arg0`.
///
/// # Safety
/// `arg0` is a heap index.
pub unsafe fn lbHeap_80015BB8(arg0: i32) -> LbHeapStatus {
    melee_core::ct!("lbHeap_80015BB8");
    (*heap_ptr(arg0 as usize)).status
}

/// `lbHeap_80015BD0`: allocates `size` bytes from heap `heap_id` (null when the heap does not exist).
///
/// # Safety
/// `heap_id` is a heap index.
pub unsafe fn lbHeap_80015BD0(heap_id: i32, size: usize) -> *mut c_void {
    melee_core::ct!("lbHeap_80015BD0");
    let enabled = ext::OSDisableInterrupts();
    let p = heap_ptr(heap_id as usize);
    let result: *mut c_void;

    if (*p).status == LbHeapStatus_Create {
        if (*p).r#type == 0 {
            let cur_heap = ext::HSD_GetHeap();
            ext::HSD_SetHeap((*p).id);
            result = ext::HSD_MemAlloc(size as i32);
            ext::HSD_SetHeap(cur_heap);
        } else {
            // MELEE_HEAP_TRACE=1: log each allocation from the handle-managed heaps
            static mut TRACE: i32 = -1;
            if TRACE < 0 {
                TRACE = ext::Env1(b"MELEE_HEAP_TRACE\0".as_ptr() as *const c_char);
            }
            if TRACE != 0 {
                ext::OSReport("lbHeap: heap %d type %d alloc 0x%X\n", &[ext::OsArg::I(heap_id), ext::OsArg::I((*p).r#type), ext::OsArg::U(size as u32)]);
            }
            let h = super::lbmemory::lbMemory_80014FC8((*p).handle, size);
            result = if (*p).r#type == 3 { (*h).x4_lo } else { h as *mut c_void };
        }
    } else {
        result = null_mut();
    }
    ext::OSRestoreInterrupts(enabled);
    result
}

/// `lbHeap_80015CA8`: frees `arg1` to heap `arg0`.
///
/// # Safety
/// `arg0` is a heap index.
pub unsafe fn lbHeap_80015CA8(arg0: i32, arg1: *mut c_void) {
    melee_core::ct!("lbHeap_80015CA8");
    let enabled = ext::OSDisableInterrupts();
    let p = heap_ptr(arg0 as usize);

    if (*p).status != LbHeapStatus_Create {
        ext::__assert(FILE, 259, "p->status == LbHeapStatus_Create");
    }
    if (*p).r#type == 0 {
        let cur_heap = ext::HSD_GetHeap();
        ext::HSD_SetHeap((*p).id);
        ext::HSD_Free(arg1);
        ext::HSD_SetHeap(cur_heap);
    } else {
        super::lbmemory::lbMemFreeToHeap((*p).handle, arg1);
    }
    ext::OSRestoreInterrupts(enabled);
}

/// `lbHeap_80015D6C`: starts the compaction of heap `heap0` (not for the HSD and ARAM heaps); `cb(heap1)` runs
/// when done.
///
/// # Safety
/// `heap0` is a heap index.
pub unsafe fn lbHeap_80015D6C(heap0: u32, cb: Option<unsafe extern "C" fn(u32)>, heap1: u32) -> i32 {
    melee_core::ct!("lbHeap_80015D6C");
    let enabled = ext::OSDisableInterrupts();
    let p = heap_ptr(heap0 as usize);
    let var_r30: i32;

    if heap0 <= 1 {
        var_r30 = 0;
    } else {
        var_r30 = super::lbmemory::lbMemory_8001529C((*p).handle, cb, heap1) as i32;
    }
    ext::OSRestoreInterrupts(enabled);
    var_r30
}

/// `lbHeap_IdOfHandle` (PC): which heap owns the handle managed arena `handle` (-1 if none); writes the heap's
/// trimmed report name into `name`.
///
/// # Safety
/// `name` is valid for `cap` bytes when `cap > 0`.
pub unsafe fn lbHeap_IdOfHandle(handle: *mut c_void, name: *mut c_char, cap: i32) -> i32 {
    melee_core::ct!("lbHeap_IdOfHandle");
    for i in 0..LBHEAP_HEAP_COUNT {
        let h = heap_ptr(i);
        if (*h).status == LbHeapStatus_Create && (*h).r#type != 0 && (*h).handle as *mut c_void == handle {
            let s = lbHeap_803BA448[i];
            let mut si = 0;
            let mut k: i32 = 0;
            while s[si] == b' ' {
                si += 1;
            }
            while s[si + k as usize] != 0 && k < cap - 1 {
                *name.add(k as usize) = s[si + k as usize] as c_char;
                k += 1;
            }
            *name.add(k as usize) = 0;
            return i as i32;
        }
    }
    if cap > 0 {
        *name = 0;
    }
    -1
}

/// `lbHeap_80015DF8`: the usage report of every heap (through `OSReport`).
///
/// # Safety
/// The heaps were set up by `lbHeap_80015F3C`.
pub unsafe fn lbHeap_80015DF8() {
    melee_core::ct!("lbHeap_80015DF8");
    use ext::OsArg::{I, S, U};
    let nm = |i: usize| S(lbHeap_803BA448[i].as_ptr() as *const c_char);
    ext::OSReport("[lbHeap] -- Report --\n", &[]);

    for i in 0..LBHEAP_HEAP_COUNT {
        ext::OSReport("%s :", &[nm(i)]);
        let p = heap_ptr(i);
        if (*p).status == LbHeapStatus_Create {
            let var_r25: i32;
            if (*p).r#type == 0 {
                var_r25 = ext::OSCheckHeap((*p).id);
            } else {
                // m-ex's 0x20-byte "Scene" heap has no handle to walk
                if (*p).handle.is_null() || (*p).handle == minus1() {
                    ext::OSReport("                    no arena", &[]);
                    ext::OSReport(" / %5d KB\n", &[U((*p).size / 1024), U((*p).size)]);
                    continue;
                }
                var_r25 = super::lbmemory::lbMemory_80014F7C((*p).handle) as i32;
            }
            ext::OSReport(" %5d KB + ", &[U((*p).size.wrapping_sub(var_r25 as u32) / 1024)]);
            ext::OSReport(" %5d KB( %8d)", &[I(var_r25 / 1024), I(var_r25)]);
        } else {
            ext::OSReport("                         destroy", &[]);
        }
        ext::OSReport(" / %5d KB\n", &[U((*p).size / 1024), U((*p).size)]);
    }

    let mut bytes: i32 = (st().arena_hi as usize).wrapping_sub(st().arena_lo as usize) as i32;
    ext::OSReport("MainRAM Total : %5d KB( %8d)\n", &[I(bytes / 1024), I(bytes)]);
    bytes = st().aram_hi.wrapping_sub(st().aram_lo) as i32;
    ext::OSReport("   ARAM Total : %5d KB( %8d)\n", &[I(bytes / 1024), I(bytes)]);
}

/// `lbHeap_80015F3C`: lays the heaps out from the descriptor table, the main RAM arena and the ARAM bounds.
///
/// # Safety
/// Not reentrant.
pub unsafe fn lbHeap_80015F3C() {
    melee_core::ct!("lbHeap_80015F3C");
    ext::HSD_GetNextArena(addr_of_mut!(lbHeap_80431FA0.arena_lo), addr_of_mut!(lbHeap_80431FA0.arena_hi));
    super::lbmemory::lbMemory_800154BC(addr_of_mut!(lbHeap_80431FA0.aram_lo), addr_of_mut!(lbHeap_80431FA0.aram_hi));

    for i in 0..6 {
        lbHeap_ResetHeap(heap_ptr(i));
    }

    let mut k = 0;
    loop {
        let desc = &lbHeap_803BA380[k];
        let curr_idx = desc.idx as usize;
        if curr_idx == LBHEAP_HEAP_COUNT {
            break;
        }
        let curr_heap = heap_ptr(curr_idx);

        (*curr_heap).r#type = desc.r#type as i32;
        (*curr_heap).size = desc.size;
        let prev_idx = desc.prev_idx as usize;
        if prev_idx == 6 {
            match (*curr_heap).r#type {
                3 => {}
                1 => (*curr_heap).start = st().arena_lo as usize,
                2 => (*curr_heap).start = (st().arena_hi as usize).wrapping_sub((*curr_heap).size as usize),
                4 => (*curr_heap).start = st().aram_lo,
                _ => {}
            }
        } else {
            let prev_heap = heap_ptr(prev_idx);
            match (*curr_heap).r#type {
                3 => {}
                1 => (*curr_heap).start = (*prev_heap).start.wrapping_add((*prev_heap).size as usize),
                2 => (*curr_heap).start = (*prev_heap).start.wrapping_sub((*curr_heap).size as usize),
                4 => (*curr_heap).start = (*prev_heap).start.wrapping_add((*prev_heap).size as usize),
                _ => {}
            }
        }
        k += 1;
    }
    ext::OSReport("lbHeap: PC stage heap 3 expanded capacity=%u (+8 MiB MEM1)\n", &[ext::OsArg::U((*heap_ptr(3)).size)]);
}

/// `lbHeap_Capacity` (PC): the size of heap `heap` (0 out of range).
pub fn lbHeap_Capacity(heap: i32) -> u32 {
    melee_core::ct!("lbHeap_Capacity");
    if heap >= 0 && (heap as usize) < LBHEAP_HEAP_COUNT {
        unsafe { (*heap_ptr(heap as usize)).size }
    } else {
        0
    }
}

/// `lbHeap_Free` (PC): the free bytes of heap `heap` (0 out of range, destroyed or without a handle).
pub fn lbHeap_Free(heap: i32) -> u32 {
    melee_core::ct!("lbHeap_Free");
    unsafe {
        if heap < 0 || heap as usize >= LBHEAP_HEAP_COUNT {
            return 0;
        }
        let p = heap_ptr(heap as usize);
        if (*p).status != LbHeapStatus_Create {
            return 0;
        }
        if (*p).r#type == 0 {
            let free_bytes = ext::OSCheckHeap((*p).id);
            return if free_bytes > 0 { free_bytes as u32 } else { 0 };
        }
        if (*p).handle.is_null() || (*p).handle == minus1() {
            return 0;
        }
        super::lbmemory::lbMemory_80014F7C((*p).handle)
    }
}

/// `lbHeap_StageSlotTryAlloc` (lbheap_stage_slots.inc): a non-asserting allocation from the main heap.
pub unsafe fn lbHeap_StageSlotTryAlloc(size: u32) -> *mut c_void {
    melee_core::ct!("lbHeap_StageSlotTryAlloc");
    let p = heap_ptr(0);
    if size == 0 || (*p).status != LbHeapStatus_Create || (*p).r#type != 0 {
        return null_mut();
    }
    ext::OSAllocFromHeap((*p).id, size)
}

/// `lbHeap_StageSlotDump` (lbheap_stage_slots.inc): diagnostic dump of the main heap's cells.
pub unsafe fn lbHeap_StageSlotDump(mode: i32) {
    melee_core::ct!("lbHeap_StageSlotDump");
    let p = heap_ptr(0);
    if (*p).status != LbHeapStatus_Create || (*p).r#type != 0 {
        return;
    }
    if mode == 2 {
        ext::OSAllocTrack(1);
        return;
    }
    ext::OSDumpHeap((*p).id);
    ext::HSD_ObjAllocReport();
}
