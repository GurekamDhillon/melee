//! Native objalloc.c. Integer address slots are widened by melee_types; pool
//! link accesses are unaligned because a C Vec pool has 12-byte cells/alignment 4.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use core::{ffi::c_void, ptr::{addr_of_mut, read_unaligned, write_unaligned}};
use melee_types::{objheap, HSD_ObjAllocData, HSD_ObjAllocLink};
use super::lbext::{OSReport, OsArg};

pub static mut obj_heap: objheap = objheap { top: 0, curr: 0, size: u32::MAX, remain: u32::MAX };
pub static mut alloc_datas: *mut HSD_ObjAllocData = core::ptr::null_mut();
pub static mut oa_trace: i32 = -1;
pub static mut oa_who: [*mut c_void; 8] = [core::ptr::null_mut(); 8];
pub static mut oa_hits: [i32; 8] = [0; 8];
pub static mut oa_size: [i32; 8] = [0; 8];
pub static mut oa_total: i32 = 0;
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
    fn HSD_GetHeap() -> i32 = crate::tu::memory::ext::HSD_GetHeap();
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = crate::tu::memory::HSD_MemAlloc(a0 as i32);
    fn OSCheckHeap(a0: i32) -> i32;
    fn PcTraceHeapEnabled() -> i32 = crate::tu::memory::ext::PcTraceHeapEnabled();
    fn Snap_InRender() -> i32;
    fn Snap_NoteArena(a0: *mut c_void, a1: *mut c_void, a2: u32);
    fn Snap_NoteObj(a0: *mut c_void, a1: *mut c_void, a2: i32);
    fn Snap_NoteRenderPool(a0: *mut c_void);
    fn Snap_RenderPoolAt(a0: i32) -> *mut c_void;
    fn Snap_RenderPoolCount() -> i32;
    fn Snap_Resimulating() -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Snap_NoteRenderPool(a0: *mut c_void) { super::Snap_NoteRenderPool(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT
pub unsafe fn snap_note_render(_data: *mut HSD_ObjAllocData, _delta: i32) {
    melee_core::ct!("snap_note_render");
    // The shipping C disabled registration after render writes became undoable.
    let _ = ext::Snap_InRender();
}
pub unsafe fn HSD_ObjAllocTopUp() {
    melee_core::ct!("HSD_ObjAllocTopUp");
    let n = ext::Snap_RenderPoolCount();
    for i in 0..n {
        let d = ext::Snap_RenderPoolAt(i).cast::<HSD_ObjAllocData>();
        if (*d).free as i32 >= 512 { continue; }
        if (*d).num_limit_flag() != 0 && ((*d).used.wrapping_add(512) as i32) > (*d).num_limit as i32 { continue; }
        let resim = ext::Snap_Resimulating();
        OSReport("snap: top-up pool %p from free=%d used=%d (resim %d)\n", &[OsArg::U(d as usize as u32), OsArg::I((*d).free as i32), OsArg::I((*d).used as i32), OsArg::I(resim)]);
        HSD_ObjAllocAddFree(d, 512u32.wrapping_sub((*d).free));
    }
}
pub unsafe fn HSD_ObjAllocReport() {
    melee_core::ct!("HSD_ObjAllocReport");
    let mut d = alloc_datas;
    while !d.is_null() {
        OSReport("objpool %p size=%d used=%d free=%d peak=%d\n", &[OsArg::U(d as usize as u32), OsArg::I((*d).size as i32), OsArg::I((*d).used as i32), OsArg::I((*d).free as i32), OsArg::I((*d).peak as i32)]);
        d = (*d).next;
    }
}
pub unsafe fn HSD_ObjSetHeap(size: u32, ptr: *mut c_void) {
    melee_core::ct!("HSD_ObjSetHeap");
    obj_heap.curr = ptr as usize; obj_heap.top = ptr as usize;
    obj_heap.remain = size; obj_heap.size = size;
}
pub unsafe fn HSD_ObjAllocAddFree(data: *mut HSD_ObjAllocData, mut num: u32) -> i32 {
    melee_core::ct!("HSD_ObjAllocAddFree");
    assert!(!data.is_null(), "HSD_ObjAllocAddFree: data");
    if oa_trace < 0 { oa_trace = ext::PcTraceHeapEnabled(); }
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
                OSReport("objalloc: %p size %d x%d\n", &[OsArg::U(oa_who[k] as usize as u32), OsArg::I(oa_size[k]), OsArg::I(oa_hits[k])]);
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
        if available < pool_size { pool_size = available - <u32 as melee_core::ppc::Ppc>::prem(available, (*data).size); }
        num = <u32 as melee_core::ppc::Ppc>::pdiv(pool_size, (*data).size);
        if num == 0 { return 0; }
        obj_heap.curr = computed_start.wrapping_add(pool_size as usize);
        obj_heap.remain = (pool_end - obj_heap.curr) as u32;
    } else {
        pool_start = ext::HSD_MemAlloc(pool_size as _).cast();
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
    melee_core::ct!("HSD_ObjAlloc");
    if (*data).num_limit_flag() != 0 && (*data).used >= (*data).num_limit { return core::ptr::null_mut(); }
    if (*data).heap_limit_flag() != 0 {
        let size = if obj_heap.top != 0 { obj_heap.remain } else { ext::OSCheckHeap(ext::HSD_GetHeap()) as u32 };
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
    melee_core::ct!("HSD_ObjFree");
    ext::Snap_NoteObj(data.cast(), obj, 1);
    snap_note_render(data, -1);
    write_unaligned(obj.cast::<*mut HSD_ObjAllocLink>(), (*data).freehead);
    (*data).freehead = obj.cast();
    (*data).free = (*data).free.wrapping_add(1);
    (*data).used = (*data).used.wrapping_sub(1);
}
pub unsafe fn removeAll(data: *mut HSD_ObjAllocData) {
    melee_core::ct!("removeAll");
    let mut cur = addr_of_mut!(alloc_datas);
    while !(*cur).is_null() {
        if *cur == data { *cur = (**cur).next; }
        else { cur = addr_of_mut!((**cur).next); }
    }
}
pub unsafe fn HSD_ObjAllocInit(data: *mut HSD_ObjAllocData, size: usize, align: u32) {
    melee_core::ct!("HSD_ObjAllocInit");
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
pub unsafe fn _HSD_ObjAllocForgetMemory(_low: *mut c_void, _high: *mut c_void) { melee_core::ct!("_HSD_ObjAllocForgetMemory"); alloc_datas = core::ptr::null_mut(); }
