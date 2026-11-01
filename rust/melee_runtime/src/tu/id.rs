//! Native port of `src/sysdolphin/baselib/id.c`: the HSD object-ID hash table.
//! Chained buckets of `IDEntry` nodes, exactly as the C (insert at head, remove unlinks).
#![allow(non_snake_case, non_upper_case_globals)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::id::{IdKey};
use core::ffi::c_void;
use melee_types::{HSD_IDKey, HSD_IDTable, HSD_ObjAllocData, IDEntry};

pub static mut hsd_iddata: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut default_table: HSD_IDTable = unsafe { core::mem::zeroed() };

pub unsafe fn HSD_IDGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_IDGetAllocData");
    &raw mut hsd_iddata
}

pub unsafe fn HSD_IDInitAllocData() {
    melee_core::ct!("HSD_IDInitAllocData");
    ext::alloc_init(HSD_IDGetAllocData(), core::mem::size_of::<IDEntry>(), 4);
}

pub unsafe fn HSD_IDSetup() {
    melee_core::ct!("HSD_IDSetup");
    (&raw mut default_table).write(HSD_IDTable::zeroed());
}

#[inline]
pub fn hash(id: HSD_IDKey) -> u32 {
    melee_core::ct!("hash");
    (id % 0x65) as u32
}

pub unsafe fn IDEntryAlloc() -> *mut IDEntry {
    melee_core::ct!("IDEntryAlloc");
    let entry = ext::alloc(HSD_IDGetAllocData()).cast::<IDEntry>();
    assert!(!entry.is_null(), "id.c:67 entry");
    entry.write(IDEntry::zeroed());
    entry
}

pub unsafe fn IDEntryFree(entry: *mut IDEntry) {
    melee_core::ct!("IDEntryFree");
    ext::free(HSD_IDGetAllocData(), entry.cast());
}

unsafe fn table_or_default(table: *mut HSD_IDTable) -> *mut HSD_IDTable {
    if table.is_null() { &raw mut default_table } else { table }
}

pub unsafe fn HSD_IDInsertToTable(table: *mut HSD_IDTable, id: HSD_IDKey, data: *mut c_void) {
    melee_core::ct!("HSD_IDInsertToTable");
    let table = table_or_default(table);
    let h = hash(id) as usize;
    let mut entry = (*table).table[h];
    while !entry.is_null() {
        if (*entry).id == id {
            break;
        }
        entry = (*entry).next;
    }
    if !entry.is_null() {
        (*entry).id = id;
        (*entry).data = data;
    } else {
        entry = IDEntryAlloc();
        (*entry).id = id;
        (*entry).data = data;
        (*entry).next = (*table).table[h];
        (*table).table[h] = entry;
    }
}

pub unsafe fn HSD_IDRemoveByIDFromTable(table: *mut HSD_IDTable, id: HSD_IDKey) {
    melee_core::ct!("HSD_IDRemoveByIDFromTable");
    let table = table_or_default(table);
    let h = hash(id) as usize;
    let mut prev: *mut IDEntry = core::ptr::null_mut();
    let mut entry = (*table).table[h];
    while !entry.is_null() {
        if (*entry).id == id {
            if !prev.is_null() {
                (*prev).next = (*entry).next;
            } else {
                (*table).table[h] = (*entry).next;
            }
            IDEntryFree(entry);
            return;
        }
        prev = entry;
        entry = (*entry).next;
    }
}

pub unsafe fn HSD_IDGetDataFromTable(table: *mut HSD_IDTable, id: HSD_IDKey, success: *mut i32) -> *mut c_void {
    melee_core::ct!("HSD_IDGetDataFromTable");
    let table = table_or_default(table);
    let mut entry = (*table).table[hash(id) as usize];
    while !entry.is_null() {
        if (*entry).id == id {
            if !success.is_null() {
                *success = 1;
            }
            return (*entry).data;
        }
        entry = (*entry).next;
    }
    if !success.is_null() {
        *success = 0;
    }
    core::ptr::null_mut()
}

pub unsafe fn _HSD_IDForgetMemory(_low: *mut c_void, _high: *mut c_void) {
    melee_core::ct!("_HSD_IDForgetMemory");
    (&raw mut default_table).write(HSD_IDTable::zeroed());
}

/// Callees in unported TUs (baselib/objalloc.c). Hosts install the real allocator through the
/// hook slots; until then each call is a TODO stub named like the C function.
pub mod ext {
    use super::*;
    pub static mut INIT: Option<unsafe fn(*mut HSD_ObjAllocData, usize, u32)> = None;
    pub static mut ALLOC: Option<unsafe fn(*mut HSD_ObjAllocData) -> *mut c_void> = None;
    pub static mut FREE: Option<unsafe fn(*mut HSD_ObjAllocData, *mut c_void)> = None;
    pub unsafe fn alloc_init(d: *mut HSD_ObjAllocData, size: usize, align: u32) {
        if let Some(f) = INIT { f(d, size, align) } else { crate::tu::objalloc::HSD_ObjAllocInit(d, size, align) }
    }
    pub unsafe fn alloc(d: *mut HSD_ObjAllocData) -> *mut c_void {
        if let Some(f) = ALLOC { f(d) } else { crate::tu::objalloc::HSD_ObjAlloc(d) }
    }
    pub unsafe fn free(d: *mut HSD_ObjAllocData, p: *mut c_void) {
        if let Some(f) = FREE { f(d, p) } else { crate::tu::objalloc::HSD_ObjFree(d, p) }
    }
}

// ---- folded from id.rs (consolidation steps 21/22) ----
#[inline]
pub fn hash_fp(id: IdKey) -> usize {
    melee_core::ct!("hash_fp");
    (id % 0x65) as usize
}
