//! Field-by-field native C oracle: colliding/64-bit keys, null values, lists, pools.
use core::ffi::c_void;
use melee_runtime::tu::{id, list};
use melee_types::{HSD_IDTable, HSD_ObjAllocData, HSD_SList};
use std::fmt::Write;

unsafe fn init(d: *mut HSD_ObjAllocData, size: usize, align: u32) {
    d.write(HSD_ObjAllocData::zeroed()); (*d).size = size as u32; (*d).align = align;
}
unsafe fn alloc(d: *mut HSD_ObjAllocData) -> *mut c_void {
    let p = std::alloc::alloc(std::alloc::Layout::from_size_align((*d).size as usize, 8).unwrap());
    p.write_bytes(0xa5, (*d).size as usize);
    (*d).used += 1; (*d).peak = (*d).peak.max((*d).used);
    p.cast()
}
unsafe fn free(d: *mut HSD_ObjAllocData, p: *mut c_void) {
    (*d).used -= 1; (*d).free += 1;
    std::alloc::dealloc(p.cast(), std::alloc::Layout::from_size_align((*d).size as usize, 8).unwrap());
}
fn next(s: &mut u32) -> u32 { *s = s.wrapping_mul(1664525).wrapping_add(1013904223); *s }

#[test]
fn native_collections_match_every_c_state_field() {
    unsafe {
        id::ext::INIT = Some(init); id::ext::ALLOC = Some(alloc); id::ext::FREE = Some(free);
        id::HSD_IDInitAllocData(); list::HSD_ListInitAllocData(); id::HSD_IDSetup();
        let mut a = HSD_IDTable::zeroed();
        let mut lists: [*mut HSD_SList; 2] = [std::ptr::null_mut(); 2];
        let mut rng = 0x945952; let mut got = String::new();
        let mut counts = [0; 10];
        writeln!(got, "EMPTY {}", list::HSD_SListRemove(std::ptr::null_mut()).is_null() as u8).unwrap();
        for i in 0..1200 {
            let r = next(&mut rng); let op = (r >> 5) % 10; let which = ((r >> 8) & 1) as usize;
            counts[op as usize] += 1;
            let mut key = ((r >> 16) % 16) as usize * 101 + ((r >> 12) & 3) as usize;
            if i % 97 == 0 { key |= 1usize << 40; }
            let data = (next(&mut rng) & 0xffff) as usize as *mut c_void;
            let tab = if which != 0 { std::ptr::null_mut() } else { &mut a };
            let mut success = -9; let mut value = std::ptr::null_mut();
            match op {
                0..=2 => id::HSD_IDInsertToTable(tab, key, data),
                3 => id::HSD_IDRemoveByIDFromTable(tab, key),
                4 => value = id::HSD_IDGetDataFromTable(tab, key, &mut success),
                5 => lists[which] = list::HSD_SListAllocAndAppend(lists[which], data),
                6 => lists[which] = list::HSD_SListAllocAndPrepend(lists[which], data),
                7 => lists[which] = list::HSD_SListRemove(lists[which]),
                8 => {
                    let node = list::HSD_SListAlloc(); (*node).data = data;
                    lists[which] = if r & 0x400 != 0 { list::HSD_SListAppendList(lists[which], node) }
                        else { list::HSD_SListPrependList(lists[which], node) };
                }
                _ => {
                    let e = id::IDEntryAlloc();
                    writeln!(got, "ZERO {} {} {}", (*e).next.is_null() as u8, ((*e).id == 0) as u8, (*e).data.is_null() as u8).unwrap();
                    id::IDEntryFree(e);
                    if r & 0x400 == 0 { id::_HSD_IDForgetMemory(std::ptr::null_mut(), std::ptr::null_mut()); } else { id::HSD_IDSetup(); }
                }
            }
            write!(got, "S {i} {} {success} {}", id::hash(key), value as usize).unwrap();
            for d in [id::HSD_IDGetAllocData(), list::HSD_SListGetAllocData(), list::HSD_DListGetAllocData()] {
                let d = &*d;
                write!(got, " {} {} {} {} {} {} {} {} {} {} {} {}", d.num_limit_flag(), d.heap_limit_flag(), !d.freehead.is_null() as u8,
                    d.used, d.free, d.peak, d.num_limit, d.heap_limit_size, d.heap_limit_num, d.size, d.align, !d.next.is_null() as u8).unwrap();
            }
            for (t, tab) in [&a, &*(&raw const id::default_table)].iter().enumerate() {
                for (b, head) in tab.table.iter().enumerate() {
                    let mut e = *head;
                    while !e.is_null() {
                        write!(got, " {t}:{b}:{}:{}", (*e).id, (*e).data as usize).unwrap(); e = (*e).next;
                    }
                }
            }
            write!(got, " L").unwrap();
            for (j, head) in lists.iter().enumerate() {
                write!(got, " {j}").unwrap(); let mut n = *head;
                while !n.is_null() { write!(got, ":{}", (*n).data as usize).unwrap(); n = (*n).next; }
            }
            got.push('\n');
        }
        id::HSD_IDInsertToTable(&mut a, usize::MAX, std::ptr::null_mut());
        let mut s = -9;
        id::HSD_IDGetDataFromTable(&mut a, usize::MAX, &mut s);
        let v = id::HSD_IDGetDataFromTable(&mut a, usize::MAX, std::ptr::null_mut());
        writeln!(got, "NULL {s} {}", v.is_null() as u8).unwrap();
        got.push_str("OPS");
        for n in counts { assert!(n > 0, "operation must be exercised"); write!(got, " {n}").unwrap(); }
        got.push('\n');
        let want = include_str!("data/tu_collections.txt");
        assert_eq!(got.lines().count(), want.lines().count());
        for (i, (got, want)) in got.lines().zip(want.lines()).enumerate() { assert_eq!(got, want, "C state row {i}"); }
    }
}
