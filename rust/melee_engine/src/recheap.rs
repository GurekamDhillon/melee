//! The asset service's recording heap (notes/asset-service-design.md, "decode plan").
//!
//! While a thread records ([`begin_recording`] .. [`end_recording`]), the process allocator (the binary's `#[global_allocator]`) serves
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
pub const NSLOT: usize = 64;
/// The slot's tail holds the block records (not recorded allocations).
const META: usize = 64 << 20;
const STEP: usize = 1 << 20;

struct State {
    base: usize,
    used: AtomicU64,
    /// Per slot: bytes from its start that the last user committed (and wrote). A released slot keeps its pages
    /// (a fresh page costs a fault per 4 KB on first touch, which made a prepare ~1.8x the old decode); the next
    /// taker zeroes this much instead, and pages past [`KEEP_COMMITTED`] are given back.
    dirty: [core::sync::atomic::AtomicUsize; NSLOT],
}
const KEEP_COMMITTED: usize = 256 << 20;
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
    /// The allocation events in order ([`Ev`]), in the slot's last [`META`] / 2 bytes.
    ev: *mut u64,
    pub nev: usize,
    ev_committed: usize,
}

/// One recorded allocator call: `kind << 62 | a << 31 | b` (block indices).
pub mod ev {
    pub const ALLOC: u64 = 0;
    pub const FREE: u64 = 1;
    pub const REALLOC: u64 = 2;
    pub fn pack(kind: u64, a: usize, b: usize) -> u64 {
        kind << 62 | (a as u64) << 31 | b as u64
    }
    pub fn unpack(e: u64) -> (u64, usize, usize) {
        (e >> 62, ((e >> 31) & 0x7FFF_FFFF) as usize, (e & 0x7FFF_FFFF) as usize)
    }
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
        VirtualAlloc(at as *mut _, n, 0x2000 | 0x200000, 0x04) as usize
    }
    extern "system" {
        fn GetWriteWatch(flags: u32, base: *mut core::ffi::c_void, size: usize, addrs: *mut *mut core::ffi::c_void, count: *mut usize, gran: *mut u32) -> u32;
    }
    /// Pages of `[a, a + n)` written since the last reset (and reset them): page addresses, ascending. None: unknown.
    pub unsafe fn written(a: usize, n: usize) -> Option<Vec<usize>> {
        let mut out = Vec::new();
        let mut buf = vec![core::ptr::null_mut::<core::ffi::c_void>(); 1 << 16];
        let mut at = a;
        while at < a + n {
            let mut count = buf.len();
            let mut gran = 0u32;
            if GetWriteWatch(1, at as *mut _, a + n - at, buf.as_mut_ptr(), &mut count, &mut gran) != 0 {
                return None;
            }
            out.extend(buf[..count].iter().map(|&p| p as usize));
            if count < buf.len() {
                break;
            }
            at = out.last().unwrap() + 4096;
        }
        Some(out)
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
    pub unsafe fn written(_a: usize, _n: usize) -> Option<Vec<usize>> {
        None
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
    // (with write watch: the plan builder reads only the pages a decode wrote; see [`written_pages`])
    let mut base = unsafe { sys::reserve_at(0x5C7A_0000_0000, SLOT * NSLOT + SLOT) };
    if base == 0 {
        base = unsafe { sys::reserve_at(0, SLOT * NSLOT + SLOT) };
    }
    if base == 0 {
        return false;
    }
    let base = (base + SLOT - 1) & !(SLOT - 1);
    let st = unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, Layout::new::<State>()) } as *mut State;
    unsafe { st.write(State { base, used: AtomicU64::new(0), dirty: [const { core::sync::atomic::AtomicUsize::new(0) }; NSLOT] }) };
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
        let free = if m == u64::MAX { 64 } else { (!m).trailing_zeros() as usize };
        if free >= NSLOT {
            return None;
        }
        if s.used.compare_exchange(m, m | (1 << free), AcqRel, Acquire).is_ok() {
            let d = s.dirty[free].swap(0, AcqRel);
            if d > 0 {
                unsafe { core::ptr::write_bytes((s.base + free * SLOT) as *mut u8, 0, d) };
            }
            // the zeroing (and the last user's writes) are not this user's writes
            let _ = unsafe { sys::written(s.base + free * SLOT, SLOT - META) };
            return Some(free);
        }
    }
}

/// Give a slot back: its pages are dropped (they read zero when committed again).
pub fn release_slot(slot: usize) {
    let Some(s) = state() else { return };
    let d = s.dirty[slot].load(Acquire);
    if d > KEEP_COMMITTED {
        unsafe { sys::decommit(s.base + slot * SLOT + KEEP_COMMITTED, SLOT - META - KEEP_COMMITTED) };
        s.dirty[slot].store(KEEP_COMMITTED, Release);
    }
    s.used.fetch_and(!(1u64 << slot), AcqRel);
}

/// Note how far slot `slot` was written (its taker zeroes that much on reuse).
pub fn note_dirty(slot: usize, upto: usize) {
    if let Some(s) = state() {
        s.dirty[slot].fetch_max(upto, AcqRel);
    }
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
        let ev = lo + SLOT - META / 2;
        Rec { slot, lo, data_lo, bump: data_lo, committed: data_lo, meta: (lo + SLOT - META) as *mut Blk, n: 0, meta_committed: lo + SLOT - META, ev: ev as *mut u64, nev: 0, ev_committed: ev }
    }
    /// The block records, in allocation (= address) order.
    pub fn events(&self) -> &[u64] {
        unsafe { core::slice::from_raw_parts(self.ev, self.nev) }
    }
    /// Forget the events so far (the probe allocation before a decode).
    pub fn clear_events(&mut self) {
        self.nev = 0;
    }
    unsafe fn push_ev(&mut self, e: u64) -> bool {
        let end = self.ev as usize + (self.nev + 1) * 8;
        if end > self.lo + SLOT {
            return false;
        }
        if end > self.ev_committed {
            if !sys::commit(self.ev_committed, STEP) {
                return false;
            }
            self.ev_committed += STEP;
        }
        self.ev.add(self.nev).write(e);
        self.nev += 1;
        true
    }
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
/// `r` stays alive and in place until [`end_recording`].
pub unsafe fn begin_recording(r: *mut Rec) {
    CUR.with(|c| c.set(r));
}

pub fn end_recording() {
    let r = CUR.with(|c| c.replace(core::ptr::null_mut()));
    if !r.is_null() {
        let r = unsafe { &*r };
        note_dirty(r.slot, r.committed - r.lo);
    }
}

/// Serve `l` from this thread's recording (null when the slot is full).
///
/// # Safety
/// Called by the process allocator while [`active`].
pub unsafe fn alloc(l: Layout) -> *mut u8 {
    let p = alloc_block(l);
    if !p.is_null() {
        let r = &mut *CUR.with(|c| c.get());
        if !r.push_ev(ev::pack(ev::ALLOC, r.n - 1, 0)) {
            return core::ptr::null_mut();
        }
    }
    p
}

/// A recorded `realloc` of the block at `p` (`old` bytes) to `n` bytes: always a new block (memory is never reused),
/// recorded as one REALLOC event so a replay makes the same `realloc` call (which the arena may do in place).
///
/// # Safety
/// `p` is a live block of this thread's recording.
pub unsafe fn realloc(p: *mut u8, old: Layout, n: usize) -> *mut u8 {
    let r = &mut *CUR.with(|c| c.get());
    let Some(i) = r.find(p as usize).filter(|&i| r.blocks()[i].at == p as usize) else { return core::ptr::null_mut() };
    let q = alloc_block(Layout::from_size_align_unchecked(n, old.align()));
    if q.is_null() {
        return q;
    }
    core::ptr::copy_nonoverlapping(p, q, old.size().min(n));
    (*r.meta.add(i)).live = 0;
    if !r.push_ev(ev::pack(ev::REALLOC, i, r.n - 1)) {
        return core::ptr::null_mut();
    }
    q
}

unsafe fn alloc_block(l: Layout) -> *mut u8 {
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
        if mend > r.lo + SLOT - META / 2 {
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
        if r.blocks()[i].at == p as usize && (*r.meta.add(i)).live == 1 {
            (*r.meta.add(i)).live = 0;
            r.push_ev(ev::pack(ev::FREE, i, 0));
        }
    }
}

/// The pages of `[a, a + n)` (inside a slot) written since the slot was taken, ascending; None: not known (treat all
/// as written).
pub fn written_pages(a: usize, n: usize) -> Option<Vec<usize>> {
    unsafe { sys::written(a & !0xFFF, ((a + n + 0xFFF) & !0xFFF) - (a & !0xFFF)) }
}
