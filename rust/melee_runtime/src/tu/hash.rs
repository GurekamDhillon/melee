//! Port of src/sysdolphin/baselib/hash.c (`HashSearchEntry`, `HSD_HashSearch`) over the generated
//! native `HSD_Hash` / `HSD_HashEntry` records. The class vtable (`getidx`, `keycheck`) is called
//! through the record's native function pointers, exactly as the C does.
#![allow(non_snake_case)]

use core::ffi::c_void;
use melee_types::{HSD_Hash, HSD_HashEntry};

/// `HashSearchEntry`. With `ptr` non-null the chain is walked by slot and `*ptr` receives the
/// address of the slot (`HSD_HashEntry**` smuggled in an `HSD_HashEntry*`) holding the match.
///
/// # Safety
/// `hash` must be a valid hash whose class info has `keycheck` set and whose `table` has more
/// than `idx` slots; every chain must be NULL terminated.
pub unsafe fn HashSearchEntry(
    hash: *mut HSD_Hash,
    idx: i32,
    key: *mut c_void,
    ptr: *mut *mut HSD_HashEntry,
) -> *mut HSD_HashEntry {
    let table = (*hash).table;
    if (*table.offset(idx as isize)).is_null() {
        return core::ptr::null_mut();
    }
    let keycheck = (*(*hash).parent.class_info).keycheck.expect("HSD_HashClassInfo.keycheck");
    if !ptr.is_null() {
        let mut entry: *mut *mut HSD_HashEntry = table.offset(idx as isize);
        while !(*entry).is_null() {
            if keycheck(hash, (**entry).key, key) == 0 {
                *ptr = entry as *mut HSD_HashEntry;
                return *entry;
            }
            entry = &mut (**entry).next;
        }
    } else {
        let mut entry: *mut HSD_HashEntry = *table.offset(idx as isize);
        while !entry.is_null() {
            if keycheck(hash, (*entry).key, key) == 0 {
                return entry;
            }
            entry = (*entry).next;
        }
    }
    core::ptr::null_mut()
}

/// `HSD_HashSearch`: returns the entry's value (NULL when absent); `*success` (if non-null) is
/// set to 1/0.
///
/// # Safety
/// As [`HashSearchEntry`]; `success` is null or writable.
pub unsafe fn HSD_HashSearch(hash: *mut HSD_Hash, key: *mut c_void, success: *mut i32) -> *mut c_void {
    let getidx = (*(*hash).parent.class_info).getidx.expect("HSD_HashClassInfo.getidx");
    let idx = getidx(hash) as u32;
    assert!(idx < (*hash).table_size, "HSD_ASSERT(113, idx < hash->table_size)");
    let entry = HashSearchEntry(hash, idx as i32, key, core::ptr::null_mut());
    if !success.is_null() {
        *success = !entry.is_null() as i32;
    }
    if !entry.is_null() {
        return (*entry).value;
    }
    core::ptr::null_mut()
}
