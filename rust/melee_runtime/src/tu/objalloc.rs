//! Native objalloc.c. Integer address slots are widened by melee_types; pool
//! link accesses are unaligned because a C Vec pool has 12-byte cells/alignment 4.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use core::{ffi::c_void, ptr::{addr_of_mut, read_unaligned, write_unaligned}};
use melee_types::{objheap, HSD_ObjAllocData, HSD_ObjAllocLink};
use super::memory::{HSD_MemAlloc, ext::{HSD_GetHeap, OSReport, PcTraceHeapEnabled}};

pub static mut obj_heap: objheap = objheap { top: 0, curr: 0, size: u32::MAX, remain: u32::MAX };
pub static mut alloc_datas: *mut HSD_ObjAllocData = core::ptr::null_mut();
pub static mut oa_trace: i32 = -1;
pub static mut oa_who: [*mut c_void; 8] = [core::ptr::null_mut(); 8];
pub static mut oa_hits: [i32; 8] = [0; 8];
pub static mut oa_size: [i32; 8] = [0; 8];
pub static mut oa_total: i32 = 0;

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
    hook!(OSCheckHeap, OSCheckHeap_hook, (heap:i32) -> i32);
    hook!(Snap_InRender, Snap_InRender_hook, () -> i32);
    hook!(Snap_NoteObj, Snap_NoteObj_hook, (data:*mut c_void, obj:*mut c_void, freeing:i32) -> ());
    hook!(Snap_NoteArena, Snap_NoteArena_hook, (data:*mut c_void, start:*mut c_void, size:u32) -> ());
    hook!(Snap_RenderPoolCount, Snap_RenderPoolCount_hook, () -> i32);
    hook!(Snap_RenderPoolAt, Snap_RenderPoolAt_hook, (index:i32) -> *mut c_void);
    hook!(Snap_Resimulating, Snap_Resimulating_hook, () -> i32);
}
pub unsafe fn snap_note_render(_data: *mut HSD_ObjAllocData, _delta: i32) {
    // The shipping C disabled registration after render writes became undoable.
    let _ = ext::Snap_InRender();
}
pub unsafe fn HSD_ObjAllocTopUp() {
    let n = ext::Snap_RenderPoolCount();
    for i in 0..n {
        let d = ext::Snap_RenderPoolAt(i).cast::<HSD_ObjAllocData>();
        if (*d).free as i32 >= 512 { continue; }
        if (*d).num_limit_flag() != 0 && ((*d).used.wrapping_add(512) as i32) > (*d).num_limit as i32 { continue; }
        let resim = ext::Snap_Resimulating();
        OSReport("snap: top-up pool %p from free=%d used=%d (resim %d)\n", &[d as usize as i64, (*d).free as i32 as i64, (*d).used as i32 as i64, resim as i64]);
        HSD_ObjAllocAddFree(d, 512u32.wrapping_sub((*d).free));
    }
}
pub unsafe fn HSD_ObjAllocReport() {
    let mut d = alloc_datas;
    while !d.is_null() {
        OSReport("objpool %p size=%d used=%d free=%d peak=%d\n", &[d as usize as i64, (*d).size as i32 as i64, (*d).used as i32 as i64, (*d).free as i32 as i64, (*d).peak as i32 as i64]);
        d = (*d).next;
    }
}
pub unsafe fn HSD_ObjSetHeap(size: u32, ptr: *mut c_void) {
    obj_heap.curr = ptr as usize; obj_heap.top = ptr as usize;
    obj_heap.remain = size; obj_heap.size = size;
}
pub unsafe fn HSD_ObjAllocAddFree(data: *mut HSD_ObjAllocData, mut num: u32) -> i32 {
    assert!(!data.is_null(), "HSD_ObjAllocAddFree: data");
    if oa_trace < 0 { oa_trace = PcTraceHeapEnabled(); }
    if oa_trace != 0 {
        let mut worst = 0;
        let mut found = false;
        for k in 0..8 {
            if oa_who[k] == data.cast() { oa_hits[k] = oa_hits[k].wrapping_add(1); found = true; break; }
            if oa_hits[k] < oa_hits[worst] { worst = k; }
        }
        if !found { oa_who[worst] = data.cast(); oa_hits[worst] = 1; oa_size[worst] = (*data).size as i32; }
        oa_total = oa_total.wrapping_add(1);
        if oa_total % 8192 == 0 {
            for k in 0..8 { if oa_hits[k] > 256 {
                OSReport("objalloc: %p size %d x%d\n", &[oa_who[k] as usize as i64, oa_size[k] as i64, oa_hits[k] as i64]);
            } }
        }
    }
    let mut pool_size = (*data).size.wrapping_mul(num);
    let pool_start: *mut u8;
    if obj_heap.top != 0 {
        let pool_end = obj_heap.top.wrapping_add(obj_heap.size as usize);
        let computed_start = obj_heap.curr.wrapping_add((*data).align as usize) & !((*data).align as usize);
        if computed_start > pool_end { return 0; }
        pool_start = computed_start as *mut u8;
        let available = (pool_end - computed_start) as u32;
        if available < pool_size { pool_size = available - available % (*data).size; }
        num = pool_size / (*data).size;
        if num == 0 { return 0; }
        obj_heap.curr = computed_start.wrapping_add(pool_size as usize);
        obj_heap.remain = (pool_end - obj_heap.curr) as u32;
    } else {
        pool_start = HSD_MemAlloc(pool_size as i32).cast();
        if pool_start.is_null() { return 0; }
        obj_heap.remain = obj_heap.remain.wrapping_sub(pool_size);
    }
    // Valid C pools have at least one pointer-sized cell. Their alignment need
    // not be the native pointer alignment (HSD_VecInitAllocData uses 4).
    for i in 0..num {
        let cell = pool_start.add((*data).size.wrapping_mul(i) as usize).cast::<*mut HSD_ObjAllocLink>();
        let next = if i + 1 == num { (*data).freehead } else { pool_start.add((*data).size.wrapping_mul(i + 1) as usize).cast() };
        write_unaligned(cell, next);
    }
    (*data).freehead = pool_start.cast();
    (*data).free = (*data).free.wrapping_add(num);
    ext::Snap_NoteArena(data.cast(), pool_start.cast(), num.wrapping_mul((*data).size));
    num as i32
}
pub unsafe fn HSD_ObjAlloc(data: *mut HSD_ObjAllocData) -> *mut c_void {
    if (*data).num_limit_flag() != 0 && (*data).used >= (*data).num_limit { return core::ptr::null_mut(); }
    if (*data).heap_limit_flag() != 0 {
        let size = if obj_heap.top != 0 { obj_heap.remain } else { ext::OSCheckHeap(HSD_GetHeap()) as u32 };
        if (*data).heap_limit_num == u32::MAX {
            if size <= (*data).heap_limit_size { (*data).heap_limit_num = (*data).used.wrapping_add((*data).free); }
        } else if size > (*data).heap_limit_size { (*data).heap_limit_num = u32::MAX; }
        if (*data).used >= (*data).heap_limit_num { return core::ptr::null_mut(); }
    }
    if (*data).free == 0 {
        HSD_ObjAllocAddFree(data, 1);
        if (*data).free == 0 { return core::ptr::null_mut(); }
    }
    let cur = (*data).freehead;
    ext::Snap_NoteObj(data.cast(), cur.cast(), 0);
    snap_note_render(data, 1);
    (*data).freehead = read_unaligned(cur.cast::<*mut HSD_ObjAllocLink>());
    (*data).used = (*data).used.wrapping_add(1);
    (*data).free = (*data).free.wrapping_sub(1);
    if (*data).used > (*data).peak { (*data).peak = (*data).used; }
    cur.cast()
}
pub unsafe fn HSD_ObjFree(data: *mut HSD_ObjAllocData, obj: *mut c_void) {
    ext::Snap_NoteObj(data.cast(), obj, 1);
    snap_note_render(data, -1);
    write_unaligned(obj.cast::<*mut HSD_ObjAllocLink>(), (*data).freehead);
    (*data).freehead = obj.cast();
    (*data).free = (*data).free.wrapping_add(1);
    (*data).used = (*data).used.wrapping_sub(1);
}
pub unsafe fn removeAll(data: *mut HSD_ObjAllocData) {
    let mut cur = addr_of_mut!(alloc_datas);
    while !(*cur).is_null() {
        if *cur == data { *cur = (**cur).next; }
        else { cur = addr_of_mut!((**cur).next); }
    }
}
pub unsafe fn HSD_ObjAllocInit(data: *mut HSD_ObjAllocData, size: usize, align: u32) {
    assert!(!data.is_null(), "HSD_ObjAllocInit: data");
    removeAll(data);
    data.write(HSD_ObjAllocData::zeroed());
    (*data).num_limit = u32::MAX;
    (*data).heap_limit_num = u32::MAX;
    (*data).align = align.wrapping_sub(1);
    (*data).size = (size.wrapping_add((*data).align as usize) & !((*data).align as usize)) as u32;
    (*data).next = alloc_datas;
    alloc_datas = data;
}
pub unsafe fn _HSD_ObjAllocForgetMemory(_low: *mut c_void, _high: *mut c_void) { alloc_datas = core::ptr::null_mut(); }
