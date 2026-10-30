//! Native port of `src/sysdolphin/baselib/list.c`: pooled singly/doubly linked list nodes.
//! Pointer-for-pointer with the C (the arena `crate::list::SList` is a separate model).
#![allow(non_snake_case, non_upper_case_globals)]
use core::ffi::c_void;
use melee_types::{HSD_DList, HSD_ObjAllocData, HSD_SList};

pub static mut slist_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut dlist_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

pub unsafe fn HSD_ListInitAllocData() {
    melee_core::ct!("HSD_ListInitAllocData");
    ext::HSD_ObjAllocInit(&raw mut slist_alloc_data, core::mem::size_of::<HSD_SList>(), 4);
    ext::HSD_ObjAllocInit(&raw mut dlist_alloc_data, core::mem::size_of::<HSD_DList>(), 4);
}

pub unsafe fn HSD_SListGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_SListGetAllocData");
    &raw mut slist_alloc_data
}

pub unsafe fn HSD_DListGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_DListGetAllocData");
    &raw mut dlist_alloc_data
}

pub unsafe fn HSD_SListAlloc() -> *mut HSD_SList {
    melee_core::ct!("HSD_SListAlloc");
    let list = ext::HSD_ObjAlloc(HSD_SListGetAllocData()).cast::<HSD_SList>();
    assert!(!list.is_null(), "list.c:76 list");
    list.write(HSD_SList::zeroed());
    list
}

pub unsafe fn HSD_SListAllocAndAppend(next: *mut HSD_SList, data: *mut c_void) -> *mut HSD_SList {
    melee_core::ct!("HSD_SListAllocAndAppend");
    let list = HSD_SListAlloc();
    (*list).data = data;
    HSD_SListAppendList(next, list)
}

pub unsafe fn HSD_SListAllocAndPrepend(next: *mut HSD_SList, data: *mut c_void) -> *mut HSD_SList {
    melee_core::ct!("HSD_SListAllocAndPrepend");
    let list = HSD_SListAlloc();
    (*list).data = data;
    HSD_SListPrependList(next, list)
}

pub unsafe fn HSD_SListAppendList(list: *mut HSD_SList, next: *mut HSD_SList) -> *mut HSD_SList {
    melee_core::ct!("HSD_SListAppendList");
    assert!(!next.is_null(), "list.c:179 next");
    if !list.is_null() {
        (*next).next = (*list).next;
        (*list).next = next;
        list
    } else {
        (*next).next = core::ptr::null_mut();
        next
    }
}

pub unsafe fn HSD_SListPrependList(list: *mut HSD_SList, prev: *mut HSD_SList) -> *mut HSD_SList {
    melee_core::ct!("HSD_SListPrependList");
    assert!(!prev.is_null(), "list.c:202 prev");
    (*prev).next = list;
    prev
}

pub unsafe fn HSD_SListRemove(list: *mut HSD_SList) -> *mut HSD_SList {
    melee_core::ct!("HSD_SListRemove");
    if !list.is_null() {
        let next = (*list).next;
        ext::HSD_ObjFree(HSD_SListGetAllocData(), list.cast());
        return next;
    }
    core::ptr::null_mut()
}

/// Callees in unported TUs (baselib/objalloc.c).
pub mod ext {
    pub use super::super::id::ext::*;
}
