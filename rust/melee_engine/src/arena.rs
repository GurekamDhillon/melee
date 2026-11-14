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
/// Live blocks per size class (diagnostics).
static mut LIVE: [i64; NCLASS] = [0; NCLASS];

#[cfg(windows)]
mod sys {
    extern "system" {
        pub fn VirtualAlloc(addr: *mut core::ffi::c_void, size: usize, ty: u32, prot: u32) -> *mut core::ffi::c_void;
        pub fn VirtualFree(addr: *mut core::ffi::c_void, size: usize, ty: u32) -> i32;
    }
    pub const MEM_DECOMMIT: u32 = 0x4000;
    pub const MEM_COMMIT: u32 = 0x1000;
    pub const MEM_RESERVE: u32 = 0x2000;
    pub const PAGE_READWRITE: u32 = 0x04;
    /// Reserve the arena's address space (`MEM_WRITE_WATCH`: a rollback asks which pages were written).
    pub unsafe fn reserve(cap: usize) -> *mut core::ffi::c_void {
        VirtualAlloc(core::ptr::null_mut(), cap, MEM_RESERVE | 0x200000, PAGE_READWRITE)
    }
    pub unsafe fn commit(at: usize, len: usize) -> bool {
        !VirtualAlloc(at as *mut _, len, MEM_COMMIT, PAGE_READWRITE).is_null()
    }
}

/// Unix: the reserve is a PROT_NONE, MAP_NORESERVE mapping; committing makes pages read-write (the kernel backs them
/// with zero pages on first touch, as Windows' MEM_COMMIT does). There is no write watch: the snapshot falls back to
/// treating every page in the range as written.
#[cfg(unix)]
mod sys {
    pub unsafe fn reserve(cap: usize) -> *mut core::ffi::c_void {
        let p = libc::mmap(core::ptr::null_mut(), cap, libc::PROT_NONE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, -1, 0);
        if p == libc::MAP_FAILED { core::ptr::null_mut() } else { p }
    }
    pub unsafe fn commit(at: usize, len: usize) -> bool {
        libc::mprotect(at as *mut _, len, libc::PROT_READ | libc::PROT_WRITE) == 0
    }
    /// Drop the pages' contents (they read back as zero), like MEM_DECOMMIT followed by a recommit.
    pub unsafe fn decommit(at: usize, len: usize) {
        libc::madvise(at as *mut _, len, libc::MADV_DONTNEED);
    }
}

/// Trim mode ([`set_trim`]): big free blocks give their pages back, and a big request with no block of its class
/// free takes the smallest free block of a larger class. Off by default: a snapshot reads the whole used range, so
/// only a run that never snapshots the arena (`melee play` offline) turns it on.
static mut TRIM: bool = false;
/// Blocks this big or bigger are trimmed.
const TRIM_MIN: usize = 64 << 10;

pub fn set_trim(on: bool) {
    unsafe { TRIM = on }
}
/// Whether [`set_trim`] is on (the run never snapshots the arena).
pub fn trim() -> bool {
    unsafe { TRIM }
}

/// Decommit the whole pages inside `[a, b)` (their contents are dropped; the range stays reserved).
#[cfg(windows)]
unsafe fn decommit(a: usize, b: usize) {
    let (a, b) = ((a + 0xFFF) & !0xFFF, b & !0xFFF);
    if b > a {
        sys::VirtualFree(a as *mut _, b - a, sys::MEM_DECOMMIT);
    }
}
#[cfg(unix)]
unsafe fn decommit(a: usize, b: usize) {
    let (a, b) = ((a + 0xFFF) & !0xFFF, b & !0xFFF);
    if b > a {
        sys::decommit(a, b - a);
    }
}
/// Unix: a decommitted page stays mapped read-write (it reads back zero), so there is nothing to do.
#[cfg(unix)]
unsafe fn recommit(_a: usize, _b: usize) -> bool {
    true
}
/// Commit the pages under `[a, b)` again (decommitted ones come back zeroed, committed ones are unchanged).
#[cfg(windows)]
unsafe fn recommit(a: usize, b: usize) -> bool {
    let (a, b) = (a & !0xFFF, (b + 0xFFF) & !0xFFF);
    !sys::VirtualAlloc(a as *mut _, b - a, sys::MEM_COMMIT, sys::PAGE_READWRITE).is_null()
}

thread_local! {
    /// Set on threads that are never game code (the GX decode thread): their allocations stay on the system heap even
    /// while the game thread is in a [`GameScope`].
    static FOREIGN: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    /// Set on the thread that entered a [`GameScope`] (the game thread). Other threads (audio callbacks, the GPU
    /// driver's workers) allocating while the game thread is in its scope stay on the system heap: a rollback load
    /// rewrites the arena under them otherwise.
    static GAME_THREAD: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

/// Test support: turn the calling thread's foreign mark on or off (the reclaim write trap's disc reads).
pub fn set_foreign(on: bool) {
    FOREIGN.with(|f| f.set(on));
}

/// Mark the calling thread as one whose allocations never go to the arena.
pub fn mark_foreign_thread() {
    FOREIGN.with(|f| f.set(true));
}

/// True while the engine code runs (allocations go to the arena), on the game thread.
#[inline]
pub fn active() -> bool {
    IN_GAME.load(Relaxed) && GAME_THREAD.with(|g| g.get()) && !FOREIGN.with(|f| f.get())
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
        GAME_THREAD.with(|g| g.set(true));
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
#[cfg(any(windows, unix))]
pub unsafe fn alloc(l: Layout) -> *mut u8 {
    alloc_fresh(l).0
}

/// Set once the arena was ever snapshotted (a load may then have left bytes past the bump pointer).
static SNAPSHOTTED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Called by `snapshot::Base::capture`: from now on fresh bump memory is not known to be zero.
pub fn note_snapshot() {
    SNAPSHOTTED.store(true, Relaxed);
}

/// [`alloc`] for a zeroed block: zeroes it unless it is fresh bump memory and the arena was never snapshotted (pages
/// past the bump pointer were never handed out: still the OS's zero pages). A multi-MB zeroed table then costs only
/// the pages that are touched (the archive decoder's shared tables, `gcdecode::shared_array`).
#[cfg(any(windows, unix))]
pub unsafe fn alloc_zeroed(l: Layout) -> *mut u8 {
    let (p, fresh) = alloc_fresh(l);
    if !p.is_null() && !(fresh && !SNAPSHOTTED.load(Relaxed)) {
        core::ptr::write_bytes(p, 0, l.size());
    }
    p
}

/// (block, whether it came from the bump pointer).
#[cfg(any(windows, unix))]
unsafe fn alloc_fresh(l: Layout) -> (*mut u8, bool) {
    let r = alloc_inner(l);
    (r, FRESH)
}

#[cfg(any(windows, unix))]
static mut FRESH: bool = false;

#[cfg(any(windows, unix))]
unsafe fn alloc_inner(l: Layout) -> *mut u8 {
    FRESH = false;
    let Some(ab) = bucket(l.align()) else { return core::ptr::null_mut() };
    let Some((c, size)) = class_of(l.size()) else { return core::ptr::null_mut() };
    let _g = Lock::take();
    if BASE == 0 {
        let p = sys::reserve(CAP);
        if p.is_null() {
            return core::ptr::null_mut();
        }
        BASE = (p as usize + 0xFFFF) & !0xFFFF;
        BUMP = BASE;
        COMMITTED = BASE;
    }
    NALLOC += 1;
    LIVE[c] += 1;
    let head = FREE[ab][c];
    if head != 0 {
        if *((head + 8) as *const u64) != FREE_MARK {
            report_misuse("a free block was written after it was freed (its free-list mark is gone)", head, size);
        }
        *((head + 8) as *mut u64) = 0;
        FREE[ab][c] = *(head as *const usize);
        if TRIM && size >= TRIM_MIN && !recommit(head, head + size) {
            return core::ptr::null_mut();
        }
        return head as *mut u8;
    }
    if TRIM && size >= TRIM_MIN {
        // the smallest free block of a larger class (up to about 4x): its tail past `size` is given back for good
        for c2 in c + 1..(c + 17).min(NCLASS) {
            let h2 = FREE[ab][c2];
            if h2 != 0 {
                FREE[ab][c2] = *(h2 as *const usize);
                *((h2 + 8) as *mut u64) = 0;
                if !recommit(h2, h2 + size) {
                    return core::ptr::null_mut();
                }
                decommit(h2 + size, h2 + class_size(c2));
                return h2 as *mut u8;
            }
        }
    }
    let al = l.align().max(16);
    let p = (BUMP + al - 1) & !(al - 1);
    let end = p + size;
    if end > BASE + CAP - 0x10000 {
        return core::ptr::null_mut();
    }
    while end > COMMITTED {
        if !sys::commit(COMMITTED, COMMIT_STEP) {
            return core::ptr::null_mut();
        }
        COMMITTED += COMMIT_STEP;
    }
    BUMP = end;
    FRESH = true;
    p as *mut u8
}

#[cfg(not(any(windows, unix)))]
pub unsafe fn alloc(_l: Layout) -> *mut u8 {
    core::ptr::null_mut()
}
#[cfg(not(any(windows, unix)))]
pub unsafe fn alloc_zeroed(_l: Layout) -> *mut u8 {
    core::ptr::null_mut()
}

/// Misuse check: a free block carries this mark in its second word (the first is the free-list link) until it is
/// handed out again. A block freed while marked was freed twice; a block popped without the mark was written after its
/// free. Both are what a harness value holding arena memory across a rollback load does (the load puts the free lists
/// back to a state where the block is free or someone else's: the headless audio capture, grown inside the game scope,
/// corrupted decoded archive objects that way and a later reclaim called a null `free` pointer). The mark lives in the
/// arena, so a load restores it together with the lists.
const FREE_MARK: u64 = 0xF4EE_B10C_F4EE_B10C;
static MISUSE_N: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static mut MISUSE_HOOK: Option<fn(&str, usize, usize)> = None;
/// The binary's reporter for [`FREE_MARK`] misuse. It runs under the arena lock: it must not allocate.
pub fn set_misuse_hook(f: fn(&str, usize, usize)) {
    unsafe { MISUSE_HOOK = Some(f) }
}
/// Misuse reports so far (tests).
pub fn misuse_count() -> usize {
    MISUSE_N.load(Relaxed)
}
unsafe fn report_misuse(what: &str, p: usize, n: usize) {
    if MISUSE_N.fetch_add(1, Relaxed) < 4 {
        if let Some(f) = MISUSE_HOOK {
            f(what, p, n);
        }
    }
}

/// Return an arena block (the caller checked [`contains`]).
pub unsafe fn free(p: *mut u8, l: Layout) {
    let (Some((c, _)), Some(ab)) = (class_of(l.size()), bucket(l.align())) else { return };
    // Read the knobs before the lock: a first env read allocates, and allocating under the lock deadlocks.
    let q = quarantine_on() && quarantine_from() < u64::MAX;
    let _g = Lock::take();
    LIVE[c] -= 1;
    if q {
        quarantine(p as usize, class_size(c));
        return;
    }
    if *((p as usize + 8) as *const u64) == FREE_MARK {
        report_misuse("a block that is already free was freed again (double free)", p as usize, l.size());
    }
    *((p as usize + 8) as *mut u64) = FREE_MARK;
    *(p as *mut usize) = FREE[ab][c];
    FREE[ab][c] = p as usize;
    #[cfg(any(windows, unix))]
    if TRIM {
        let size = class_size(c);
        if size >= TRIM_MIN {
            decommit(p as usize + 16, p as usize + size);
        }
    }
}

/// Block size of class `c` (inverse of [`class_of`]).
fn class_size(c: usize) -> usize {
    if c < 8 {
        (c + 1) * 16
    } else {
        let k = 7 + (c - 8) / 8;
        let sub = (c - 8) % 8 + 1;
        (1usize << k) + sub * (1usize << (k - 3))
    }
}

/// The size class a `realloc` stays in (the block can be reused in place).
pub fn same_class(a: usize, b: usize) -> bool {
    class_of(a).map(|x| x.0) == class_of(b).map(|x| x.0)
}

/// The writable sections of the image this code is in (`.data`, `.bss`, ...): every Rust / C static of the game. That
/// is the executable for `melee.exe`, and the library when the engine is a game library a host loaded (core v1): the
/// process's main module is then the host, whose statics are not the game's.
#[cfg(windows)]
pub fn static_sections() -> Vec<(usize, usize)> {
    extern "system" {
        fn GetModuleHandleExW(flags: u32, name: *const u16, module: *mut *const u8) -> i32;
    }
    // GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT
    const FROM_ADDRESS: u32 = 0x4 | 0x2;
    static ANCHOR: u8 = 0;
    unsafe {
        let mut base: *const u8 = core::ptr::null();
        if GetModuleHandleExW(FROM_ADDRESS, (&ANCHOR as *const u8).cast(), &mut base) == 0 || base.is_null() {
            // An image the loader never saw (a host mapping it itself, as Hero's Source Port does inside an
            // AppContainer): the start of the allocation holding it is the image's base.
            base = allocation_base((&ANCHOR as *const u8).cast());
            if base.is_null() || core::ptr::read_unaligned(base as *const u16) != 0x5A4D {
                return Vec::new();
            }
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

#[cfg(windows)]
unsafe fn allocation_base(p: *const u8) -> *const u8 {
    #[repr(C)]
    struct MemoryBasicInformation { base: usize, allocation_base: usize, allocation_protect: u32, partition: u16, size: usize, state: u32, protect: u32, kind: u32 }
    extern "system" {
        fn VirtualQuery(address: *const u8, info: *mut MemoryBasicInformation, length: usize) -> usize;
    }
    let mut info: MemoryBasicInformation = core::mem::zeroed();
    if VirtualQuery(p, &mut info, core::mem::size_of::<MemoryBasicInformation>()) == 0 {
        return core::ptr::null();
    }
    info.allocation_base as *const u8
}

/// Linux: the main program's writable PT_LOAD segments (`.data`, `.bss`, ...), less its PT_GNU_RELRO part (made
/// read-only after relocation) and the TLS template (each thread's live TLS is elsewhere, as on Windows).
#[cfg(target_os = "linux")]
pub fn static_sections() -> Vec<(usize, usize)> {
    unsafe extern "C" fn cb(info: *mut libc::dl_phdr_info, _size: usize, out: *mut core::ffi::c_void) -> i32 {
        let out = &mut *(out as *mut Vec<(usize, usize)>);
        let info = &*info;
        let base = info.dlpi_addr as usize;
        let ph = core::slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize);
        let relro = ph.iter().find(|p| p.p_type == libc::PT_GNU_RELRO).map(|p| (base + p.p_vaddr as usize, base + (p.p_vaddr + p.p_memsz) as usize));
        for p in ph.iter().filter(|p| p.p_type == libc::PT_LOAD && p.p_flags & libc::PF_W != 0 && p.p_memsz != 0) {
            let mut a = (base + p.p_vaddr as usize) & !0xFFF;
            let e = (base + (p.p_vaddr + p.p_memsz) as usize + 0xFFF) & !0xFFF;
            if let Some((ra, re)) = relro {
                // RELRO covers the segment's start (the loader page-rounds its end down)
                if ra <= a && re > a {
                    a = re & !0xFFF;
                }
            }
            if e > a {
                out.push((a, e - a));
            }
        }
        1 // the first object is the main program
    }
    let mut out: Vec<(usize, usize)> = Vec::new();
    unsafe { libc::dl_iterate_phdr(Some(cb), &mut out as *mut _ as *mut core::ffi::c_void) };
    out
}

#[cfg(not(any(windows, target_os = "linux")))]
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

/// Diagnostics: live bytes in blocks handed out (by size class, so rounded up).
pub fn live_bytes() -> usize {
    unsafe {
        (0..NCLASS)
            .filter(|&c| LIVE[c] > 0)
            .map(|c| {
                let sz = if c < 8 { (c + 1) * 16 } else { let k = 7 + (c - 8) / 8; let sub = (c - 8) % 8 + 1; (1usize << k) + sub * (1usize << (k - 3)) };
                sz * LIVE[c] as usize
            })
            .sum()
    }
}

/// Diagnostics: bytes bumped, bytes sitting on the free lists, and the five classes holding most free bytes (class size, blocks).
pub fn stats() -> (usize, usize, Vec<(usize, usize)>) {
    unsafe {
        let mut free = 0usize;
        let mut per: Vec<(usize, usize)> = Vec::new();
        for c in 0..NCLASS {
            let mut n = 0usize;
            let mut sz = 0usize;
            for ab in 0..NALIGN {
                let mut p = FREE[ab][c];
                while p != 0 {
                    n += 1;
                    p = *(p as *const usize);
                }
            }
            if n > 0 {
                // block size of class c (inverse of class_of)
                sz = if c < 8 { (c + 1) * 16 } else { let k = 7 + (c - 8) / 8; let sub = (c - 8) % 8 + 1; (1usize << k) + sub * (1usize << (k - 3)) };
                free += n * sz;
                per.push((sz * n, sz));
            }
        }
        per.sort_unstable_by(|a, b| b.cmp(a));
        // live bytes by class: replace the free-list report by the live one
        let mut live: Vec<(usize, usize)> = (0..NCLASS).filter(|&c| LIVE[c] > 0).map(|c| {
            let sz = if c < 8 { (c + 1) * 16 } else { let k = 7 + (c - 8) / 8; let sub = (c - 8) % 8 + 1; (1usize << k) + sub * (1usize << (k - 3)) };
            (sz * LIVE[c] as usize, sz)
        }).collect();
        live.sort_unstable_by(|a, b| b.cmp(a));
        if std::env::var_os("MELEE_ARENA_LIVE").is_some() {
            return (BUMP - BASE, free, live.into_iter().take(8).map(|(t, s)| (s, t / s)).collect());
        }
        (BUMP - BASE, free, per.into_iter().take(6).map(|(t, s)| (s, t / s)).collect())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// The free-list mark catches a double free and a write after free (the class of bug a harness value holding
    /// arena memory across a rollback load produces: the headless audio capture, net.md).
    #[test]
    fn free_mark_reports_double_free_and_write_after_free() {
        unsafe {
            let l = Layout::from_size_align(200, 16).unwrap();
            let n0 = misuse_count();
            let p = alloc(l);
            assert!(!p.is_null());
            free(p, l);
            assert_eq!(misuse_count(), n0, "a plain free is not misuse");
            free(p, l); // double free
            assert_eq!(misuse_count(), n0 + 1);
            // (that class's list now links the block to itself: leave it) a write after free, in another class
            let l2 = Layout::from_size_align(3000, 16).unwrap();
            let q = alloc(l2);
            free(q, l2);
            *(q.add(8) as *mut u64) = 0x1234;
            let r = alloc(l2);
            assert_eq!(r, q);
            assert_eq!(misuse_count(), n0 + 2);
        }
    }
}

// ---- quarantine (`MELEE_ARENA_QUARANTINE=1`, test only): a freed block is filled with 0xEE and never handed out
// again; its address, size and the return addresses of the free are kept, and [`quarantine_check`] (once a frame)
// reports the first block something wrote into after its free, with the stack that freed it.
const QN: usize = 1 << 20;
const QS: usize = 10;
static mut Q: [(usize, usize, [usize; QS]); QN] = [(0, 0, [0; QS]); QN];
static mut QLEN: usize = 0;
static mut QDONE: usize = 0;
/// The frame `quarantine_check` last saw, and the first frame frees are recorded from (MELEE_ARENA_QUARANTINE_FROM,
/// default 0): blocks freed earlier are still poisoned and never reused, but not tracked. The record is a ring, so
/// late frees are always tracked (the old `QLEN < QN` cap stopped recording after 1 << 20 frees).
static mut QFRAME: u64 = 0;
fn quarantine_from() -> u64 {
    static F: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *F.get_or_init(|| std::env::var("MELEE_ARENA_QUARANTINE_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0))
}
fn quarantine_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("MELEE_ARENA_QUARANTINE").is_some())
}
unsafe fn quarantine(p: usize, size: usize) {
    core::ptr::write_bytes(p as *mut u8, 0xEE, size);
    if QFRAME >= quarantine_from() {
        let mut st = [0usize; QS];
        #[cfg(windows)]
        {
            extern "system" {
                fn RtlCaptureStackBackTrace(skip: u32, n: u32, f: *mut *mut core::ffi::c_void, h: *mut u32) -> u16;
            }
            RtlCaptureStackBackTrace(2, QS as u32, st.as_mut_ptr().cast(), core::ptr::null_mut());
        }
        Q[QLEN % QN] = (p, size, st);
        QLEN += 1;
    }
}
/// Report (to stderr, once) the first quarantined block written after its free: address, offset, bytes and the
/// freeing stack as exe offsets. Returns true when one was found.
pub fn quarantine_check(frame: u64) -> bool {
    if !quarantine_on() {
        return false;
    }
    let _ = quarantine_from();
    let found = {
        let _g = Lock::take();
        let mut f = None;
        unsafe {
            QFRAME = frame;
            if frame == 1278 {
                eprintln!("arena: quarantine at frame 1278: {QLEN} frees recorded (ring {QN}), {} unchecked", QLEN - QDONE);
            }
            for i in QDONE.max(QLEN.saturating_sub(QN))..QLEN {
                let (p, n, st) = Q[i % QN];
                let b = core::slice::from_raw_parts(p as *const u8, n);
                if let Some(o) = b.iter().position(|&x| x != 0xEE) {
                    let mut w = [0u8; 16];
                    let k = (n - o).min(16);
                    w[..k].copy_from_slice(&b[o..o + k]);
                    f = Some((p, n, o, w, st));
                    QDONE = QLEN;
                    break;
                }
            }
        }
        f
    };
    let Some((p, n, o, w, st)) = found else { return false };
    unsafe {
        let base = {
            #[cfg(windows)]
            {
                extern "system" {
                    fn GetModuleHandleW(n: *const u16) -> *mut core::ffi::c_void;
                }
                GetModuleHandleW(core::ptr::null()) as usize
            }
            #[cfg(not(windows))]
            0usize
        };
        let mut s = String::new();
        for a in st.iter().filter(|&&a| a > base && a < base + 0x400_0000) {
            s.push_str(&format!(" melee.exe+{:#X}", a - base));
        }
        eprintln!("arena: QUARANTINE WRITTEN at frame {frame}: block {p:#x} ({n} bytes) +{o:#x} now {w:02x?}; freed at{s}");
    }
    true
}
