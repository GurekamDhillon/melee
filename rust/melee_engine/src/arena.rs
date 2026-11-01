//! The game arena: a global-allocator region for every Rust heap object the game code creates (step 17).
//!
//! The engine's game state is spread over three kinds of memory: the OS-heap block (`lowmem`), the Rust statics
//! (`.data` / `.bss`, the C globals of the TU ports) and the Rust heap objects those statics own (Box / Vec /
//! HashMap in `Host`, the decoded archive objects, ...). A rollback has to put all three back. The heap objects
//! can only be restored if they live in a range the snapshot knows, so while the engine runs ([`GameScope`]) the
//! process allocator ([`alloc`] / [`free`], called from the binary's `#[global_allocator]`) serves them from one
//! reserved range; everything else (the harness, the snapshots themselves) stays on the system heap.
//!
//! Size classes are powers of two (a block's class follows from its `Layout`, so there are no headers) and the
//! free lists live in this module's statics, which the snapshot restores together with the range.

use core::alloc::Layout;
use core::sync::atomic::{AtomicBool, Ordering::*};

/// Reserved address space (committed on demand).
const CAP: usize = 16 << 30;
const COMMIT_STEP: usize = 32 << 20;
const NCLASS: usize = 320;
const NALIGN: usize = 9;

fn bucket(a: usize) -> Option<usize> {
    let b = (a.max(16).trailing_zeros() - 4) as usize;
    (b < NALIGN).then_some(b)
}

static IN_GAME: AtomicBool = AtomicBool::new(false);
static LOCK: AtomicBool = AtomicBool::new(false);
static mut BASE: usize = 0;
static mut COMMITTED: usize = 0;
static mut BUMP: usize = 0;
/// Free lists by alignment bucket (16 << b) and size class.
static mut FREE: [[usize; NCLASS]; NALIGN] = [[0; NCLASS]; NALIGN];
/// Blocks served / bytes bumped (diagnostics).
static mut NALLOC: u64 = 0;

#[cfg(windows)]
mod sys {
    extern "system" {
        pub fn VirtualAlloc(addr: *mut core::ffi::c_void, size: usize, ty: u32, prot: u32) -> *mut core::ffi::c_void;
    }
    pub const MEM_COMMIT: u32 = 0x1000;
    pub const MEM_RESERVE: u32 = 0x2000;
    pub const PAGE_READWRITE: u32 = 0x04;
}

/// True while the engine code runs (allocations go to the arena).
pub fn active() -> bool {
    IN_GAME.load(Relaxed)
}

/// `(start, len)` of the part of the arena in use (page rounded), or `(0, 0)` before the first allocation.
pub fn used_range() -> (usize, usize) {
    unsafe {
        if BASE == 0 {
            return (0, 0);
        }
        (BASE, ((BUMP + 0xFFF) & !0xFFF) - BASE)
    }
}

pub fn contains(p: *const u8) -> bool {
    unsafe { BASE != 0 && (p as usize) >= BASE && (p as usize) < BASE + CAP }
}

/// Blocks allocated since the process started (diagnostics; restored by a rollback like the rest).
pub fn allocations() -> u64 {
    unsafe { NALLOC }
}

/// Allocations made while the guard lives are served from the arena.
pub struct GameScope(bool);

impl GameScope {
    pub fn enter() -> GameScope {
        GameScope(IN_GAME.swap(true, Relaxed))
    }
}

impl Drop for GameScope {
    fn drop(&mut self) {
        IN_GAME.store(self.0, Relaxed);
    }
}

/// Set the mode flag (a rollback restores the statics, including this one, so the caller puts its own value back).
pub fn set_active(on: bool) {
    IN_GAME.store(on, Relaxed);
}

struct Lock;
impl Lock {
    fn take() -> Lock {
        while LOCK.compare_exchange_weak(false, true, Acquire, Relaxed).is_err() {
            core::hint::spin_loop();
        }
        Lock
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        LOCK.store(false, Release);
    }
}

/// (class index, block size): 16-byte steps up to 128, then eight steps per power of two (at most 12.5% slack).
fn class_of(size: usize) -> Option<(usize, usize)> {
    let s = size.max(16);
    if s > CAP / 2 {
        return None;
    }
    if s <= 128 {
        let r = (s + 15) & !15;
        return Some((r / 16 - 1, r));
    }
    let k = (usize::BITS - 1 - (s - 1).leading_zeros()) as usize; // 2^k < s <= 2^(k+1)
    let step = 1usize << (k - 3);
    let r = (s + step - 1) & !(step - 1);
    let sub = (r - (1 << k)) / step; // 1..=8
    Some((8 + (k - 7) * 8 + sub - 1, r))
}

/// Serve `l` from the arena; null when the arena does not take it (alignment above 16, huge, no address space).
#[cfg(windows)]
pub unsafe fn alloc(l: Layout) -> *mut u8 {
    let Some(ab) = bucket(l.align()) else { return core::ptr::null_mut() };
    let Some((c, size)) = class_of(l.size()) else { return core::ptr::null_mut() };
    let _g = Lock::take();
    if BASE == 0 {
        let p = sys::VirtualAlloc(core::ptr::null_mut(), CAP, sys::MEM_RESERVE | 0x200000, sys::PAGE_READWRITE);
        if p.is_null() {
            return core::ptr::null_mut();
        }
        BASE = (p as usize + 0xFFFF) & !0xFFFF;
        BUMP = BASE;
        COMMITTED = BASE;
    }
    NALLOC += 1;
    let head = FREE[ab][c];
    if head != 0 {
        FREE[ab][c] = *(head as *const usize);
        return head as *mut u8;
    }
    let al = l.align().max(16);
    let p = (BUMP + al - 1) & !(al - 1);
    let end = p + size;
    if end > BASE + CAP - 0x10000 {
        return core::ptr::null_mut();
    }
    while end > COMMITTED {
        let q = sys::VirtualAlloc(COMMITTED as *mut _, COMMIT_STEP, sys::MEM_COMMIT, sys::PAGE_READWRITE);
        if q.is_null() {
            return core::ptr::null_mut();
        }
        COMMITTED += COMMIT_STEP;
    }
    BUMP = end;
    p as *mut u8
}

#[cfg(not(windows))]
pub unsafe fn alloc(_l: Layout) -> *mut u8 {
    core::ptr::null_mut()
}

/// Return an arena block (the caller checked [`contains`]).
pub unsafe fn free(p: *mut u8, l: Layout) {
    let (Some((c, _)), Some(ab)) = (class_of(l.size()), bucket(l.align())) else { return };
    let _g = Lock::take();
    *(p as *mut usize) = FREE[ab][c];
    FREE[ab][c] = p as usize;
}

/// The size class a `realloc` stays in (the block can be reused in place).
pub fn same_class(a: usize, b: usize) -> bool {
    class_of(a).map(|x| x.0) == class_of(b).map(|x| x.0)
}

/// The writable sections of the running executable (`.data`, `.bss`, ...): every Rust / C static of the game.
#[cfg(windows)]
pub fn static_sections() -> Vec<(usize, usize)> {
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *const u8;
    }
    unsafe {
        let base = GetModuleHandleW(core::ptr::null());
        if base.is_null() {
            return Vec::new();
        }
        let rd32 = |o: usize| core::ptr::read_unaligned(base.add(o) as *const u32);
        let rd16 = |o: usize| core::ptr::read_unaligned(base.add(o) as *const u16);
        let nt = rd32(0x3C) as usize;
        let nsec = rd16(nt + 6) as usize;
        let opt_size = rd16(nt + 20) as usize;
        let first = nt + 24 + opt_size;
        let mut out = Vec::new();
        for i in 0..nsec {
            let s = first + 40 * i;
            let name = core::slice::from_raw_parts(base.add(s), 8);
            let vsize = rd32(s + 8) as usize;
            let va = rd32(s + 12) as usize;
            let ch = rd32(s + 36);
            let writable = ch & 0x8000_0000 != 0;
            let discardable = ch & 0x0200_0000 != 0;
            if writable && !discardable && vsize != 0 && !name.starts_with(b".tls") {
                out.push((base as usize + va, (vsize + 0xFFF) & !0xFFF));
            }
        }
        out
    }
}

#[cfg(not(windows))]
pub fn static_sections() -> Vec<(usize, usize)> {
    Vec::new()
}

/// Diagnostics: the first requests made inside a [`GameScope`] that the arena did not take (they went to the system heap and a rollback cannot restore them).
pub static MISSED: [core::sync::atomic::AtomicUsize; 32] = [const { core::sync::atomic::AtomicUsize::new(0) }; 32];
pub static NMISSED: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
pub fn note_miss(l: Layout) {
    let n = NMISSED.fetch_add(1, Relaxed);
    if n < 16 {
        MISSED[2 * n].store(l.size(), Relaxed);
        MISSED[2 * n + 1].store(l.align(), Relaxed);
    }
}
