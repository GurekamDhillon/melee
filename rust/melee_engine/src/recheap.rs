//! The asset service's recording heap (notes/asset-service-design.md, "decode plan").
//!
//! While a thread records ([`begin`] .. [`end`]), the process allocator (the binary's `#[global_allocator]`) serves
//! that thread's allocations from one *slot* of a reserved address range instead of the game arena or the system heap:
//! a bump allocator that never reuses memory, so every block starts zeroed and the bytes a decode leaves are a pure
//! function of its input. A free only marks the block dead. The asset service then copies the blocks that are
//! reachable from the decode's result into the game arena at the apply tick and rewrites the pointers between them
//! (asset.rs), so the arena sees the same allocations whichever thread decoded and whenever it did.
//!
//! The range is reserved once at boot ([`init`]) and its bookkeeping lives in a heap cell whose address the static
//! holds (statics are snapshot state; the cell is not, and a rollback load must not rewind which slots are in use).
//! A free of an address in the range by a thread that is not recording it is a no-op (the slot owns the memory).

use core::alloc::Layout;
use core::cell::Cell;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering::*};

/// Bytes per slot (reserved; committed on demand).
pub const SLOT: usize = 1 << 30;
/// Slots in the range (one per prepared file kept; at most 64, the mask width).
pub const NSLOT: usize = 48;
/// The slot's tail holds the block records (not recorded allocations).
const META: usize = 64 << 20;
const STEP: usize = 1 << 20;

struct State {
    base: usize,
    used: AtomicU64,
}
static STATE: AtomicPtr<State> = AtomicPtr::new(core::ptr::null_mut());

/// One recorded allocation.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Blk {
    pub at: usize,
    pub size: usize,
    pub align: usize,
    pub live: usize,
}

/// A thread's recording into one slot.
pub struct Rec {
    pub slot: usize,
    pub lo: usize,
    /// First byte of the block area (after the raw image).
    pub data_lo: usize,
    pub bump: usize,
    committed: usize,
    meta: *mut Blk,
    pub n: usize,
    meta_committed: usize,
}

thread_local! {
    static CUR: Cell<*mut Rec> = const { Cell::new(core::ptr::null_mut()) };
}

#[cfg(windows)]
mod sys {
    extern "system" {
        pub fn VirtualAlloc(addr: *mut core::ffi::c_void, size: usize, ty: u32, prot: u32) -> *mut core::ffi::c_void;
        pub fn VirtualFree(addr: *mut core::ffi::c_void, size: usize, ty: u32) -> i32;
    }
    pub unsafe fn reserve(n: usize) -> usize {
        VirtualAlloc(core::ptr::null_mut(), n, 0x2000, 0x04) as usize
    }
    pub unsafe fn reserve_at(at: usize, n: usize) -> usize {
        VirtualAlloc(at as *mut _, n, 0x2000, 0x04) as usize
    }
    pub unsafe fn commit(at: usize, n: usize) -> bool {
        !VirtualAlloc(at as *mut _, n, 0x1000, 0x04).is_null()
    }
    pub unsafe fn decommit(at: usize, n: usize) {
        VirtualFree(at as *mut _, n, 0x4000);
    }
}
#[cfg(not(windows))]
mod sys {
    pub unsafe fn reserve(_n: usize) -> usize {
        0
    }
    pub unsafe fn reserve_at(_at: usize, _n: usize) -> usize {
        0
    }
    pub unsafe fn commit(_at: usize, _n: usize) -> bool {
        false
    }
    pub unsafe fn decommit(_at: usize, _n: usize) {}
}

/// Reserve the range (once, at boot, before any snapshot). False: not supported here (the service stays off).
pub fn init() -> bool {
    if !STATE.load(Acquire).is_null() {
        return true;
    }
    // a fixed, odd-looking base first: the plan builder takes any word inside the range for a pointer, so the range
    // should be one that plain data (big-endian words, floats, text) is unlikely to spell
    let mut base = unsafe { sys::reserve_at(0x5C7A_0000_0000, SLOT * NSLOT + SLOT) };
    if base == 0 {
        base = unsafe { sys::reserve(SLOT * NSLOT + SLOT) };
    }
    if base == 0 {
        return false;
    }
    let base = (base + SLOT - 1) & !(SLOT - 1);
    let st = unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, Layout::new::<State>()) } as *mut State;
    unsafe { st.write(State { base, used: AtomicU64::new(0) }) };
    STATE.store(st, Release);
    true
}

fn state() -> Option<&'static State> {
    let p = STATE.load(Acquire);
    (!p.is_null()).then(|| unsafe { &*p })
}

/// True when `p` lies in the recording range (any slot).
#[inline]
pub fn in_range(p: *const u8) -> bool {
    let p = p as usize;
    match state() {
        Some(s) => p >= s.base && p < s.base + SLOT * NSLOT,
        None => false,
    }
}

/// True while this thread records.
#[inline]
pub fn active() -> bool {
    CUR.with(|c| !c.get().is_null())
}

/// Claim a free slot; None when all are in use.
pub fn take_slot() -> Option<usize> {
    let s = state()?;
    loop {
        let m = s.used.load(Acquire);
        let free = (!m).trailing_zeros() as usize;
        if free >= NSLOT {
            return None;
        }
        if s.used.compare_exchange(m, m | (1 << free), AcqRel, Acquire).is_ok() {
            return Some(free);
        }
    }
}

/// Give a slot back: its pages are dropped (they read zero when committed again).
pub fn release_slot(slot: usize) {
    let Some(s) = state() else { return };
    unsafe { sys::decommit(s.base + slot * SLOT, SLOT) };
    s.used.fetch_and(!(1u64 << slot), AcqRel);
}

/// Start of slot `slot`.
pub fn slot_base(slot: usize) -> usize {
    state().map_or(0, |s| s.base + slot * SLOT)
}

/// Commit `[at, at + n)` inside a slot (for the raw image the caller places before the blocks).
pub fn commit(at: usize, n: usize) -> bool {
    let a = at & !(STEP - 1);
    let b = (at + n + STEP - 1) & !(STEP - 1);
    unsafe { sys::commit(a, b - a) }
}

impl Rec {
    /// A recording into `slot` whose blocks start at `data_lo` (inside the slot, already past whatever the caller put there).
    pub fn new(slot: usize, data_lo: usize) -> Rec {
        let lo = slot_base(slot);
        let data_lo = (data_lo + 0xFFF) & !0xFFF;
        Rec { slot, lo, data_lo, bump: data_lo, committed: data_lo, meta: (lo + SLOT - META) as *mut Blk, n: 0, meta_committed: lo + SLOT - META }
    }
    /// The block records, in allocation (= address) order.
    pub fn blocks(&self) -> &[Blk] {
        unsafe { core::slice::from_raw_parts(self.meta, self.n) }
    }
    /// The index of the block that contains `p` (its one-past-end address included), live or dead.
    pub fn find(&self, p: usize) -> Option<usize> {
        let b = self.blocks();
        let i = b.partition_point(|x| x.at <= p);
        if i == 0 {
            return None;
        }
        let x = &b[i - 1];
        (p <= x.at + x.size).then_some(i - 1)
    }
}

/// Make `r` this thread's recording.
///
/// # Safety
/// `r` stays alive and in place until [`end`].
pub unsafe fn begin(r: *mut Rec) {
    CUR.with(|c| c.set(r));
}

pub fn end() {
    CUR.with(|c| c.set(core::ptr::null_mut()));
}

/// Serve `l` from this thread's recording (null when the slot is full).
///
/// # Safety
/// Called by the process allocator while [`active`].
pub unsafe fn alloc(l: Layout) -> *mut u8 {
    let r = &mut *CUR.with(|c| c.get());
    let al = l.align().max(16);
    let p = (r.bump + al - 1) & !(al - 1);
    // a 16-byte gap after each block keeps one-past-end pointers unambiguous
    let end = p + l.size().max(1) + 16;
    if end > r.lo + SLOT - META {
        return core::ptr::null_mut();
    }
    if end > r.committed {
        let to = (end + STEP - 1) & !(STEP - 1);
        if !sys::commit(r.committed, to - r.committed) {
            return core::ptr::null_mut();
        }
        r.committed = to;
    }
    let mend = r.meta as usize + (r.n + 1) * core::mem::size_of::<Blk>();
    if mend > r.meta_committed {
        if mend > r.lo + SLOT {
            return core::ptr::null_mut();
        }
        if !sys::commit(r.meta_committed, STEP) {
            return core::ptr::null_mut();
        }
        r.meta_committed += STEP;
    }
    r.meta.add(r.n).write(Blk { at: p, size: l.size(), align: l.align(), live: 1 });
    r.n += 1;
    r.bump = end;
    p as *mut u8
}

/// A free of a recorded address: marks the block dead when this thread records its slot, else nothing (the slot owns it).
///
/// # Safety
/// Called by the process allocator for an address [`in_range`].
pub unsafe fn free(p: *mut u8) {
    let r = CUR.with(|c| c.get());
    if r.is_null() {
        return;
    }
    let r = &mut *r;
    if (p as usize) < r.data_lo || (p as usize) >= r.bump {
        return;
    }
    if let Some(i) = r.find(p as usize) {
        if r.blocks()[i].at == p as usize {
            (*r.meta.add(i)).live = 0;
        }
    }
}
