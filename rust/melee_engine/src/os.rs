//! The dolphin SDK's OSAlloc heaps and arena, on a block below 4 GiB (`crate::lowmem`).
//!
//! Semantics follow `OSAlloc.c`: `OSInitAlloc(lo, hi, maxHeaps)` reserves a heap table at `lo` and returns the
//! first free byte, `OSCreateHeap(start, end)` rounds to 32 bytes, an allocation takes `round32(size + 0x20)`
//! bytes (a 32-byte cell header, the returned pointer is after it), first fit over the address-ordered free
//! list, a remainder under 64 bytes goes to the allocation, and a free coalesces with both neighbours. The cell
//! bookkeeping lives beside the arena (not in it) so that a stray write cannot corrupt the allocator.

use crate::lowmem::{alloc_low, LowBlock};
use core::ffi::c_void;
use std::collections::BTreeMap;

const CELL_HDR: usize = 0x20;
const ALIGN: usize = 0x20;
const MIN_SPLIT: usize = 0x40;

fn up(x: usize) -> usize {
    (x + ALIGN - 1) & !(ALIGN - 1)
}
fn down(x: usize) -> usize {
    x & !(ALIGN - 1)
}

#[derive(Default)]
pub struct OsHeap {
    lo: usize,
    hi: usize,
    /// cell start -> cell size
    free: BTreeMap<usize, usize>,
    /// cell start -> cell size
    used: BTreeMap<usize, usize>,
}

impl OsHeap {
    fn new(lo: usize, hi: usize) -> Self {
        let (lo, hi) = (up(lo), down(hi));
        let mut free = BTreeMap::new();
        if hi > lo {
            free.insert(lo, hi - lo);
        }
        Self { lo, hi, free, used: BTreeMap::new() }
    }

    fn alloc(&mut self, size: usize) -> *mut c_void {
        let need = up(size + CELL_HDR);
        let Some((&at, &have)) = self.free.iter().find(|(_, &sz)| sz >= need) else { return core::ptr::null_mut() };
        self.free.remove(&at);
        let take = if have - need < MIN_SPLIT {
            have
        } else {
            self.free.insert(at + need, have - need);
            need
        };
        self.used.insert(at, take);
        (at + CELL_HDR) as *mut c_void
    }

    fn free(&mut self, ptr: usize) -> bool {
        let cell = ptr - CELL_HDR;
        let Some(sz) = self.used.remove(&cell) else { return false };
        let (mut start, mut size) = (cell, sz);
        if let Some((&n, &nsz)) = self.free.range(start + size..).next() {
            if n == start + size {
                self.free.remove(&n);
                size += nsz;
            }
        }
        if let Some((&p, &psz)) = self.free.range(..start).next_back() {
            if p + psz == start {
                self.free.remove(&p);
                start = p;
                size += psz;
            }
        }
        self.free.insert(start, size);
        true
    }

    /// `OSCheckHeap`: the free byte count (the header size is not counted as free).
    pub fn free_bytes(&self) -> usize {
        self.free.values().map(|&s| s.saturating_sub(CELL_HDR).max(0)).sum()
    }
    pub fn range(&self) -> (usize, usize) {
        (self.lo, self.hi)
    }
}

/// The OS: the main-RAM arena, the heap table and the current heap.
pub struct Os {
    pub mem: LowBlock,
    arena_lo: usize,
    arena_hi: usize,
    heaps: Vec<Option<OsHeap>>,
    max_heaps: usize,
    current: i32,
}

impl Os {
    /// A fresh OS over `mem_size` bytes of low memory (the GameCube has 24 MiB; the PC build gives the game more).
    pub fn new(mem_size: usize) -> Result<Self, String> {
        let mem = alloc_low(mem_size)?;
        let (lo, hi) = (mem.base as usize, mem.base as usize + mem.len);
        Ok(Self { mem, arena_lo: lo, arena_hi: hi, heaps: Vec::new(), max_heaps: 0, current: -1 })
    }
    pub fn physical_mem_size(&self) -> u32 {
        self.mem.len as u32
    }
    pub fn arena_lo(&self) -> *mut c_void {
        self.arena_lo as *mut c_void
    }
    pub fn arena_hi(&self) -> *mut c_void {
        self.arena_hi as *mut c_void
    }
    pub fn set_arena_lo(&mut self, p: *mut c_void) {
        self.arena_lo = p as usize;
    }
    pub fn set_arena_hi(&mut self, p: *mut c_void) {
        self.arena_hi = p as usize;
    }

    /// `OSInitAlloc`: the heap table takes `max_heaps * 12` bytes at `lo`; returns the next free address.
    pub fn init_alloc(&mut self, lo: *mut c_void, hi: *mut c_void, max_heaps: i32) -> *mut c_void {
        let _ = hi;
        self.max_heaps = max_heaps.max(0) as usize;
        self.heaps = (0..self.max_heaps).map(|_| None).collect();
        self.current = -1;
        up(lo as usize + self.max_heaps * 12) as *mut c_void
    }

    /// `OSCreateHeap`: the new heap's id, or -1 when the table is full.
    pub fn create_heap(&mut self, start: *mut c_void, end: *mut c_void) -> i32 {
        match self.heaps.iter().position(Option::is_none) {
            Some(i) => {
                self.heaps[i] = Some(OsHeap::new(start as usize, end as usize));
                i as i32
            }
            None => -1,
        }
    }
    pub fn destroy_heap(&mut self, id: i32) {
        if let Some(h) = self.heaps.get_mut(id as usize) {
            *h = None;
        }
        if self.current == id {
            self.current = -1;
        }
    }
    pub fn set_current_heap(&mut self, id: i32) -> i32 {
        std::mem::replace(&mut self.current, id)
    }
    pub fn current_heap(&self) -> i32 {
        self.current
    }
    pub fn alloc_from_heap(&mut self, id: i32, size: u32) -> *mut c_void {
        match self.heaps.get_mut(id as usize).and_then(Option::as_mut) {
            Some(h) => h.alloc(size as usize),
            None => core::ptr::null_mut(),
        }
    }
    pub fn free_to_heap(&mut self, id: i32, ptr: *mut c_void) {
        if ptr.is_null() {
            return;
        }
        let ok = self.heaps.get_mut(id as usize).and_then(Option::as_mut).map_or(false, |h| h.free(ptr as usize));
        assert!(ok, "OSFreeToHeap: {ptr:p} is not a live block of heap {id}");
    }
    pub fn check_heap(&self, id: i32) -> i32 {
        match self.heaps.get(id as usize).and_then(Option::as_ref) {
            Some(h) => h.free_bytes() as i32,
            None => -1,
        }
    }
    pub fn heap(&self, id: i32) -> Option<&OsHeap> {
        self.heaps.get(id as usize).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_free_coalesce() {
        let mut os = Os::new(1 << 20).unwrap();
        let lo = os.init_alloc(os.arena_lo(), os.arena_hi(), 2);
        let h = os.create_heap(lo, os.arena_hi());
        assert_eq!(h, 0);
        let total = os.check_heap(h);
        let a = os.alloc_from_heap(h, 100);
        let b = os.alloc_from_heap(h, 5000);
        let c = os.alloc_from_heap(h, 1);
        assert!(!a.is_null() && (a as usize) % 32 == 0 && (b as usize) > (a as usize));
        assert_eq!(b as usize - a as usize, 160); // round32(100 + 0x20)
        os.free_to_heap(h, b);
        os.free_to_heap(h, a);
        os.free_to_heap(h, c);
        assert_eq!(os.check_heap(h), total, "everything coalesced back into one cell");
        assert!(os.alloc_from_heap(h, u32::MAX / 2).is_null());
        assert_eq!(os.create_heap(lo, os.arena_hi()), 1);
        assert_eq!(os.create_heap(lo, os.arena_hi()), -1);
    }
}
