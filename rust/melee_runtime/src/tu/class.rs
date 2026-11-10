//! Port of src/sysdolphin/baselib/class.c: HSD's class system. A class is an `HSD_ClassInfo`
//! (name, sizes, parent / child / sibling links, live-object counts, and the alloc / init /
//! release / destroy / amnesia methods) and `hsdNew` builds an instance from one. Instances come
//! from a size-class pool of 32-byte-granular pieces (`hsdAllocMemPiece` / `hsdFreeMemPiece` over
//! `memory_list`), which grows through `HSD_MemAlloc`.
//!
//! Native shape: the records are `melee_types`' native `repr(C)` structs (pointers are host
//! pointers); `obj_size` / `info_size` are native `size_of`s, and the pool's bookkeeping is the
//! native C build's (`sizeof(*memory_list)` is the host pointer size), so size classes and
//! `nb_alloc` / `nb_free` evolve exactly as in the C; the state oracle checks this. The resize
//! path zeroes the new slots in full (the C zeroes `4 * n` bytes, which is the whole range only
//! with 4-byte pointers; the oracle's arena is zeroed, so both agree).
//!
//! `hsdClass` is the root class. Callees of unported TUs (HSD_MemAlloc, HSD_HashSearch, OSReport)
//! are the hooks in [`ext`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_void, CStr};
use core::ptr::null_mut;
use melee_types::{
    HSD_Class, HSD_ClassInfo, HSD_ClassInfoHead, HSD_FreeList, HSD_Hash, HSD_MemoryEntry, HSD_Obj,
    _HSD_ClassInfoHead,
};

use super::debug::__assert;
use super::lbext::OsArg;
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
    fn HSD_HashSearch(a0: *mut HSD_Hash, a1: *mut c_void, a2: *mut i32) -> *mut c_void = crate::tu::hash::HSD_HashSearch(a0, a1, a2);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = crate::tu::memory::HSD_MemAlloc(a0);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

macro_rules! hsd_assert {
    ($line:expr, $cond:expr, $text:literal) => {
        if !($cond) {
            __assert(c"class.c".as_ptr(), $line, concat!($text, "\0").as_ptr() as *const c_char);
        }
    };
}

pub static mut memory_list: *mut *mut HSD_MemoryEntry = null_mut();
pub static mut nb_memory_list: i32 = 0;
pub static mut current_hash: *mut HSD_Hash = null_mut();

/// `hsdClass`: the root of the class tree (`info_init` is `_hsdClassInfoInit`).
pub static mut hsdClass: HSD_ClassInfo = HSD_ClassInfo {
    head: _HSD_ClassInfoHead {
        info_init: Some(_hsdClassInfoInit),
        flags: 0,
        library_name: null_mut(),
        class_name: null_mut(),
        obj_size: 0,
        info_size: 0,
        parent: null_mut(),
        next: null_mut(),
        child: null_mut(),
        nb_exist: 0,
        nb_peak: 0,
    },
    alloc: None,
    init: None,
    release: None,
    destroy: None,
    amnesia: None,
};

#[inline]
fn round_down_32(x: u32) -> u32 {
    x & !31
}
#[inline]
fn round_up_32(x: u32) -> u32 {
    x.wrapping_add(31) & !31
}

/// `ClassInfoInit`.
pub unsafe fn ClassInfoInit(info: *mut HSD_ClassInfo) {
    melee_core::ct!("ClassInfoInit");
    if (*info).head.flags & 1 == 0 {
        ((*info).head.info_init.unwrap())();
    }
}

/// `hsdInitClassInfo`.
pub unsafe fn hsdInitClassInfo(
    class_info: *mut HSD_ClassInfo,
    parent_info: *mut HSD_ClassInfo,
    base_class_library: *mut c_char,
    type_: *mut c_char,
    info_size: i32,
    class_size: i32,
) {
    melee_core::ct!("hsdInitClassInfo");
    (*class_info).head.flags = 1;
    (*class_info).head.library_name = base_class_library;
    (*class_info).head.class_name = type_;
    (*class_info).head.obj_size = class_size as i16;
    (*class_info).head.info_size = info_size as i16;
    (*class_info).head.parent = parent_info;
    (*class_info).head.child = null_mut();
    (*class_info).head.next = null_mut();
    (*class_info).head.nb_exist = 0;
    (*class_info).head.nb_peak = 0;

    if !parent_info.is_null() {
        if (*parent_info).head.flags & 1 == 0 {
            ((*parent_info).head.info_init.unwrap())();
        }
        hsd_assert!(94, (*class_info).head.obj_size >= (*parent_info).head.obj_size, "class_info->head.obj_size >= parent_info->head.obj_size");
        hsd_assert!(95, (*class_info).head.info_size >= (*parent_info).head.info_size, "class_info->head.info_size >= parent_info->head.info_size");
        let n = (*parent_info).head.info_size as usize - core::mem::size_of::<HSD_ClassInfoHead>();
        core::ptr::copy(
            core::ptr::addr_of!((*parent_info).alloc) as *const u8,
            core::ptr::addr_of_mut!((*class_info).alloc) as *mut u8,
            n,
        );
        (*class_info).head.next = (*parent_info).head.child;
        (*parent_info).head.child = class_info;
    }
}

/// `OSReport_PrintSpaces`.
pub fn OSReport_PrintSpaces(count: i32) {
    melee_core::ct!("OSReport_PrintSpaces");
    for _ in 0..count.max(0) {
        super::lbext::OSReport(" ", &[]);
    }
}

/// `GetMemoryEntry`.
pub unsafe fn GetMemoryEntry(idx: i32) -> *mut HSD_MemoryEntry {
    melee_core::ct!("GetMemoryEntry");
    hsd_assert!(171, idx >= 0, "idx >= 0");
    let slot = core::mem::size_of::<*mut HSD_MemoryEntry>();

    if idx >= nb_memory_list {
        if nb_memory_list == 0 {
            // Uninitialised: allocate the array.
            let mut new_nb = 32;
            while idx >= new_nb {
                new_nb *= 2;
            }
            memory_list = ext::HSD_MemAlloc((new_nb as usize * slot) as i32) as *mut *mut HSD_MemoryEntry;
            if memory_list.is_null() {
                return null_mut();
            }
            core::ptr::write_bytes(memory_list as *mut u8, 0, new_nb as usize * slot);
            nb_memory_list = new_nb;
        } else {
            // Resize the array.
            let mut new_nb = nb_memory_list * 2;
            while idx >= new_nb {
                new_nb *= 2;
            }
            let new_list = ext::HSD_MemAlloc((new_nb as usize * slot) as i32) as *mut *mut HSD_MemoryEntry;
            if new_list.is_null() {
                return null_mut();
            }
            core::ptr::copy_nonoverlapping(memory_list, new_list, nb_memory_list as usize);
            core::ptr::write_bytes(new_list.add(nb_memory_list as usize), 0, (new_nb - nb_memory_list) as usize);
            let old_list = memory_list;
            let old_nb = round_down_32((nb_memory_list * slot as i32) as u32) as i32;
            memory_list = new_list;
            nb_memory_list = new_nb;

            hsdFreeMemPiece(old_list as *mut c_void, old_nb);
            let e = *memory_list.add((round_up_32(old_nb as u32) / 32 - 1) as usize);
            (*e).nb_alloc += 1;
        }
    }

    if (*memory_list.add(idx as usize)).is_null() {
        let entry = ext::HSD_MemAlloc(core::mem::size_of::<HSD_MemoryEntry>() as i32) as *mut HSD_MemoryEntry;
        if entry.is_null() {
            return null_mut();
        }
        core::ptr::write_bytes(entry as *mut u8, 0, core::mem::size_of::<HSD_MemoryEntry>());
        (*entry).size = (idx as u32 + 1) * 32;
        *memory_list.add(idx as usize) = entry;

        let mut found = false;
        let mut i = idx as isize - 1;
        while i >= 0 {
            let m = *memory_list.add(i as usize);
            if !m.is_null() {
                found = true;
                (*entry).next = (*m).next;
                (*m).next = entry;
                break;
            }
            i -= 1;
        }
        if !found {
            let mut i = idx + 1;
            while i < nb_memory_list {
                let m = *memory_list.add(i as usize);
                if !m.is_null() {
                    (*entry).next = m;
                    break;
                }
                i += 1;
            }
        }
    }
    *memory_list.add(idx as usize)
}

/// `hsdAllocMemPiece`.
/// `hsdAllocMemPiece` (with the MELEE_HEAP_LEAKS piece census around it).
pub unsafe fn hsdAllocMemPiece(size: i32) -> *mut c_void {
    let p = hsdAllocMemPiece_body(size);
    melee_core::heaptrace::piece_alloc(p as usize, size as u32);
    p
}
unsafe fn hsdAllocMemPiece_body(size: i32) -> *mut c_void {
    melee_core::ct!("hsdAllocMemPiece");
    let mut temp_r29 = (size + 0x1F) / 32 - 1;
    let temp_r3_3 = GetMemoryEntry((size + 0x1F) / 32 - 1);
    if temp_r3_3.is_null() {
        return null_mut();
    }
    let temp_r3_2 = (*temp_r3_3).free_list;
    if !temp_r3_2.is_null() {
        (*temp_r3_3).free_list = (*temp_r3_2).next;
        (*temp_r3_3).nb_free -= 1;
        return temp_r3_2 as *mut c_void;
    }
    let mut var_r28 = (*temp_r3_3).next;
    while !var_r28.is_null() {
        if !(*var_r28).free_list.is_null() {
            let temp_r3_4 = GetMemoryEntry(((*var_r28).size as i32 - (*temp_r3_3).size as i32 + 0x1F) / 32 - 1);
            if temp_r3_4.is_null() {
                return null_mut();
            }
            let temp_r5 = (*var_r28).free_list;
            (*var_r28).free_list = (*(*var_r28).free_list).next;
            (*var_r28).nb_free -= 1;
            (*var_r28).nb_alloc -= 1;
            let temp_r4_2 = (temp_r5 as *mut u8).add((*temp_r3_3).size as usize) as *mut HSD_FreeList;
            (*temp_r4_2).next = (*temp_r3_4).free_list;
            (*temp_r3_4).free_list = temp_r4_2;
            (*temp_r3_4).nb_alloc += 1;
            (*temp_r3_4).nb_free += 1;
            (*temp_r3_3).nb_alloc += 1;
            return temp_r5 as *mut c_void;
        }
        var_r28 = (*var_r28).next;
    }
    let temp_r28 = (nb_memory_list - temp_r29) - 2;
    temp_r29 = temp_r28;
    let mut var_r30: *mut HSD_MemoryEntry = null_mut();
    if temp_r29 >= 0 {
        var_r30 = GetMemoryEntry(temp_r29);
        if var_r30.is_null() {
            return null_mut();
        }
    }
    let temp_r3 = ext::HSD_MemAlloc(nb_memory_list * 32);
    if temp_r3.is_null() {
        return null_mut();
    }
    if temp_r28 >= 0 {
        let temp_r4 = (temp_r3 as *mut u8).add((*temp_r3_3).size as usize) as *mut HSD_FreeList;
        (*temp_r4).next = (*var_r30).free_list;
        (*var_r30).free_list = temp_r4;
        (*var_r30).nb_alloc += 1;
        (*var_r30).nb_free += 1;
    }
    (*temp_r3_3).nb_alloc += 1;
    temp_r3
}

/// `hsdFreeMemPiece`.
pub unsafe fn hsdFreeMemPiece(mem: *mut c_void, size: i32) {
    melee_core::ct!("hsdFreeMemPiece");
    let piece = mem as *mut HSD_FreeList;
    melee_core::heaptrace::piece_free(mem as usize);
    if !mem.is_null() {
        let entry = GetMemoryEntry((size + 31) / 32 - 1);
        (*piece).next = (*entry).free_list;
        (*entry).free_list = piece;
        (*entry).nb_free += 1;
    }
}

/// `_hsdClassAlloc`.
pub unsafe extern "C" fn _hsdClassAlloc(info: *mut HSD_ClassInfo) -> *mut HSD_Class {
    melee_core::ct!("_hsdClassAlloc");
    let mem_piece = hsdAllocMemPiece((*info).head.obj_size as i32) as *mut HSD_Class;
    if !mem_piece.is_null() {
        (*info).head.nb_exist += 1;
        if (*info).head.nb_exist > (*info).head.nb_peak {
            (*info).head.nb_peak = (*info).head.nb_exist;
        }
    }
    mem_piece
}

/// `_hsdClassInit`.
pub unsafe extern "C" fn _hsdClassInit(_arg0: *mut HSD_Class) -> i32 {
    melee_core::ct!("_hsdClassInit");
    0
}

/// `_hsdClassRelease`.
pub unsafe extern "C" fn _hsdClassRelease(_cls: *mut HSD_Class) { melee_core::ct!("_hsdClassRelease");}

/// `_hsdClassDestroy`.
pub unsafe extern "C" fn _hsdClassDestroy(cls: *mut HSD_Class) {
    melee_core::ct!("_hsdClassDestroy");
    let info = (*cls).class_info;
    (*info).head.nb_exist = (*info).head.nb_exist.wrapping_sub(1);
    hsdFreeMemPiece(cls as *mut c_void, (*info).head.obj_size as i32);
}

/// `_hsdClassAmnesia`.
pub unsafe extern "C" fn _hsdClassAmnesia(info: *mut HSD_ClassInfo) {
    melee_core::ct!("_hsdClassAmnesia");
    (*info).head.nb_exist = 0;
    (*info).head.nb_peak = 0;
    if info == core::ptr::addr_of_mut!(hsdClass) {
        nb_memory_list = 0;
        memory_list = null_mut();
        current_hash = null_mut();
    }
}

/// `_hsdClassInfoInit`.
pub unsafe extern "C" fn _hsdClassInfoInit() {
    melee_core::ct!("_hsdClassInfoInit");
    hsdInitClassInfo(
        core::ptr::addr_of_mut!(hsdClass),
        null_mut(),
        c"sysdolphin_base_library".as_ptr() as *mut c_char,
        c"hsd_class".as_ptr() as *mut c_char,
        core::mem::size_of::<HSD_ClassInfo>() as i32,
        core::mem::size_of::<HSD_Class>() as i32,
    );
    hsdClass.alloc = Some(_hsdClassAlloc);
    hsdClass.init = Some(_hsdClassInit);
    hsdClass.release = Some(_hsdClassRelease);
    hsdClass.destroy = Some(_hsdClassDestroy);
    hsdClass.amnesia = Some(_hsdClassAmnesia);
}

/// `hsdNew`.
pub unsafe fn hsdNew(i: *mut HSD_ClassInfo) -> *mut c_void {
    melee_core::ct!("hsdNew");
    let info = i;
    let info2 = info;
    if (*info2).head.flags & 1 == 0 {
        ((*info2).head.info_init.unwrap())();
    }
    let cls = ((*info).alloc.unwrap())(info2);
    if cls.is_null() {
        return null_mut();
    }
    ClassInfoInit(info);
    core::ptr::write_bytes(cls as *mut u8, 0, (*info).head.obj_size as usize);
    (*cls).class_info = info;
    if ((*info).init.unwrap())(cls) < 0 {
        ((*info).destroy.unwrap())(cls);
        return null_mut();
    }
    cls as *mut c_void
}

/// `hsdChangeClass_inline` (static inline).
unsafe fn hsdChangeClass_inline(object: *mut HSD_Obj, class_info: *mut HSD_ClassInfo) -> bool {
    hsd_assert!(0x249, !object.is_null(), "object");
    hsd_assert!(0x24A, !class_info.is_null(), "class_info");
    let mut var_r29 = (*object).parent.class_info;
    let mut var_r28 = class_info;
    if (*var_r28).head.flags & 1 == 0 {
        ((*var_r28).head.info_init.unwrap())();
    }
    if (*var_r29).head.obj_size != (*var_r28).head.obj_size {
        return false;
    }
    while !(*var_r29).head.parent.is_null() && (*(*var_r29).head.parent).head.obj_size == (*var_r29).head.obj_size {
        var_r29 = (*var_r29).head.parent;
    }
    while !(*var_r28).head.parent.is_null() && (*(*var_r28).head.parent).head.obj_size == (*var_r28).head.obj_size {
        var_r28 = (*var_r28).head.parent;
    }
    if var_r29 == var_r28 {
        (*var_r29).head.nb_exist = (*var_r29).head.nb_exist.wrapping_sub(1);
        (*class_info).head.nb_exist += 1;
        if (*class_info).head.nb_exist > (*class_info).head.nb_peak {
            (*class_info).head.nb_peak = (*class_info).head.nb_exist;
        }
        (*object).parent.class_info = class_info;
        return true;
    }
    false
}

/// `hsdChangeClass`.
pub unsafe fn hsdChangeClass(object: *mut c_void, class_info: *mut c_void) -> bool {
    melee_core::ct!("hsdChangeClass");
    hsdChangeClass_inline(object as *mut HSD_Obj, class_info as *mut HSD_ClassInfo)
}

/// `hsdIsDescendantOf`.
pub unsafe fn hsdIsDescendantOf(info: *mut c_void, p: *mut c_void) -> bool {
    melee_core::ct!("hsdIsDescendantOf");
    let cls = p as *mut HSD_ClassInfo;
    if info.is_null() || p.is_null() {
        return false;
    }
    let mut var_r31 = info as *mut HSD_ClassInfo;
    if (*var_r31).head.flags & 1 == 0 {
        ((*var_r31).head.info_init.unwrap())();
    }
    if (*cls).head.flags & 1 == 0 {
        ((*cls).head.info_init.unwrap())();
    }
    while !var_r31.is_null() {
        if var_r31 == cls {
            return true;
        }
        var_r31 = (*var_r31).head.parent;
    }
    false
}

/// `hsdObjIsDescendantOf`.
pub unsafe fn hsdObjIsDescendantOf(o: *mut HSD_Obj, p: *mut HSD_ClassInfo) -> bool {
    melee_core::ct!("hsdObjIsDescendantOf");
    if o.is_null() || p.is_null() {
        return false;
    }
    let mut info = (*o).parent.class_info;
    if (*p).head.flags & 1 == 0 {
        ((*p).head.info_init.unwrap())();
    }
    while !info.is_null() {
        if info == p {
            return true;
        }
        info = (*info).head.parent;
    }
    false
}

/// `class_set_flags`.
pub unsafe fn class_set_flags(class_info: *mut HSD_ClassInfo, set: i32, reset: i32) {
    melee_core::ct!("class_set_flags");
    (*class_info).head.flags = ((*class_info).head.flags & !(reset as u32)) | set as u32;
}

/// `ForgetClassLibraryReal`.
pub unsafe fn ForgetClassLibraryReal(class_info: *mut HSD_ClassInfo) {
    melee_core::ct!("ForgetClassLibraryReal");
    let mut cur = (*class_info).head.child;
    while !cur.is_null() {
        let next = (*cur).head.next;
        (*cur).head.next = null_mut();
        ForgetClassLibraryReal(cur);
        cur = next;
    }
    ((*class_info).amnesia.unwrap())(class_info);
    (*class_info).head.child = null_mut();
    (*class_info).head.parent = null_mut();
    class_set_flags(class_info, 0, 1);
}

/// `ForgetClassLibraryChild`.
pub unsafe fn ForgetClassLibraryChild(library_name: *const c_char, class_info: *mut HSD_ClassInfo) {
    melee_core::ct!("ForgetClassLibraryChild");
    let name = CStr::from_ptr(library_name);
    let mut cur: *mut *mut HSD_ClassInfo = core::ptr::addr_of_mut!((*class_info).head.child);
    while !(*cur).is_null() {
        if name == CStr::from_ptr((**cur).head.library_name) {
            ForgetClassLibraryReal(*cur);
            *cur = (**cur).head.next;
        } else {
            cur = core::ptr::addr_of_mut!((**cur).head.next);
        }
    }
}

/// `hsdForgetClassLibrary`.
pub unsafe fn hsdForgetClassLibrary(mut library_name: *const c_char) {
    melee_core::ct!("hsdForgetClassLibrary");
    if library_name.is_null() {
        library_name = c"sysdolphin_base_library".as_ptr();
    }
    if hsdClass.head.flags & 1 == 0 {
        return;
    }
    if CStr::from_ptr(library_name) == CStr::from_ptr(hsdClass.head.library_name) {
        current_hash = null_mut();
        ForgetClassLibraryReal(core::ptr::addr_of_mut!(hsdClass));
    } else {
        ForgetClassLibraryChild(library_name, core::ptr::addr_of_mut!(hsdClass));
    }
}

/// `hsdSearchClassInfo`.
pub unsafe fn hsdSearchClassInfo(class_name: *const c_char) -> *mut HSD_ClassInfo {
    melee_core::ct!("hsdSearchClassInfo");
    if !current_hash.is_null() {
        return ext::HSD_HashSearch(current_hash, class_name as *mut c_void, null_mut()) as *mut HSD_ClassInfo;
    }
    null_mut()
}

/// `DumpClassStat`.
pub unsafe fn DumpClassStat(info: *mut HSD_ClassInfo, level: i32) {
    melee_core::ct!("DumpClassStat");
    OSReport_PrintSpaces(level);
    super::lbext::OSReport("<class %s>\n", &[OsArg::S((*info).head.class_name)]);
    OSReport_PrintSpaces(level);
    super::lbext::OSReport(
        "    info %d object %d nb_exist %d nb_peak %d\n",
        &[OsArg::I((*info).head.info_size as i32), OsArg::I((*info).head.obj_size as i32), OsArg::I((*info).head.nb_exist as i32), OsArg::I((*info).head.nb_peak as i32)],
    );
}

/// `hsdDumpClassStat`.
pub unsafe fn hsdDumpClassStat(info: *mut HSD_ClassInfo, recursive: bool, mut level: i32) {
    melee_core::ct!("hsdDumpClassStat");
    if info.is_null() {
        hsdDumpClassStat(core::ptr::addr_of_mut!(hsdClass), true, level);
    } else if (*info).head.flags & 1 != 0 {
        DumpClassStat(info, level);
        if recursive {
            level += 2;
            let mut c = (*info).head.child;
            while !c.is_null() {
                hsdDumpClassStat(c, true, level);
                c = (*c).head.next;
            }
        }
    }
}
