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

/// Whether freeing a block of `size` bytes drops its contents (trim mode decommits its pages past the first 16 bytes):
/// writing such a block's last contents just before its free is wasted work.
pub fn free_drops_contents(size: usize) -> bool {
    (unsafe { TRIM }) && class_of(size).is_some_and(|(_, cs)| cs >= TRIM_MIN)
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

/// Mark the calling thread as one whose allocations never go to the arena.
pub fn mark_foreign_thread() {
    FOREIGN.with(|f| f.set(true));
}

/// While it lives, this thread's allocations go to the system heap even inside a [`GameScope`] (the asset service's
/// own bookkeeping on the game thread: never game state).
pub struct SysScope(bool);
impl SysScope {
    pub fn enter() -> SysScope {
        SysScope(FOREIGN.with(|f| f.replace(true)))
    }
}
impl Drop for SysScope {
    fn drop(&mut self) {
        FOREIGN.with(|f| f.set(self.0));
    }
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
    if ZERO_ALL.with(|z| z.get()) {
        return alloc_zeroed(l);
    }
    alloc_fresh(l).0
}

thread_local! {
    /// The asset path (asset.rs: a decode in place, or a plan's replay): every block it takes is zeroed, recycled ones
    /// included, so bytes the decode leaves unwritten are 0 whichever way the load ran (not the previous occupant's).
    static ZERO_ALL: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
    /// `MELEE_ASSET_DIGEST`: (address, size) of every block the asset path took (system heap; diagnostics).
    static ASSET_BLOCKS: core::cell::RefCell<Option<Vec<(usize, usize)>>> = const { core::cell::RefCell::new(None) };
}

/// Turn the asset path's zeroing on/off for this thread (see `ZERO_ALL`); `log`: also list its blocks.
pub fn set_asset_path(on: bool, log: bool) {
    ZERO_ALL.with(|z| z.set(on));
    if on && log {
        let _s = SysScope::enter();
        ASSET_BLOCKS.with_borrow_mut(|v| *v = Some(Vec::new()));
    }
}

/// Whether this thread is on the asset path (see `ZERO_ALL`).
#[inline]
pub fn asset_path_on() -> bool {
    ZERO_ALL.with(|z| z.get())
}

/// The blocks listed since [`set_asset_path`]`(true, true)`.
pub fn take_asset_blocks() -> Vec<(usize, usize)> {
    ASSET_BLOCKS.with_borrow_mut(|v| v.take()).unwrap_or_default()
}

fn note_asset_block(p: *mut u8, n: usize) {
    if p.is_null() {
        return;
    }
    ASSET_BLOCKS.with_borrow_mut(|v| {
        if let Some(v) = v.as_mut() {
            let _s = SysScope::enter();
            v.push((p as usize, n));
        }
    });
}

/// `MELEE_DECODE_PROF` (archive.rs): time the asset path's allocations on the decoding thread (diagnostics).
pub static PROF_ALLOC: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
thread_local! {
    /// (ns in `alloc_zeroed`, calls, bytes) since the last take.
    pub static PROF_NS: core::cell::Cell<(u64, u64, u64)> = const { core::cell::Cell::new((0, 0, 0)) };
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
    if PROF_ALLOC.load(Relaxed) {
        PROF_ALLOC.store(false, Relaxed);
        let t = std::time::Instant::now();
        let p = alloc_zeroed(l);
        let ns = t.elapsed().as_nanos() as u64;
        PROF_ALLOC.store(true, Relaxed);
        PROF_NS.with(|c| { let (a, n, b) = c.get(); c.set((a + ns, n + 1, b + l.size() as u64)) });
        return p;
    }
    let (p, fresh) = alloc_fresh(l);
    if ZERO_ALL.with(|z| z.get()) {
        note_asset_block(p, l.size());
    }
    if !p.is_null() && !(fresh && !SNAPSHOTTED.load(Relaxed)) {
        let n = l.size();
        if !fresh && TRIM && class_of(n).is_some_and(|(_, cs)| cs >= TRIM_MIN) {
            // a recycled trimmed block: `free` decommitted its whole pages past the first 16 bytes, and the allocation
            // recommitted them, so they read back zero already. Zero only the bytes no decommit covered (up to the
            // first page boundary past +16, and the partial last page): writing the whole block (the decoder's 40 MB
            // shared tables) touched every page again for nothing
            let (a, e) = (p as usize, p as usize + n);
            let head_end = ((a + 16 + 0xFFF) & !0xFFF).min(e);
            core::ptr::write_bytes(p, 0, head_end - a);
            let tail = (e & !0xFFF).max(head_end);
            core::ptr::write_bytes(tail as *mut u8, 0, e - tail);
        } else {
            core::ptr::write_bytes(p, 0, n);
        }
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
    let _g = Lock::take();
    alloc_locked(l)
}

/// The asset service's apply: `ls` (size, align) served in order under one lock, each exactly as [`alloc`] would
/// (same blocks, same free-list and bump effects), addresses pushed to `out` (bit 0 set: the block is known zero, see the push). False at the first one the arena does
/// not take (`out` then holds the ones before it).
#[cfg(any(windows, unix))]
pub unsafe fn alloc_many(ls: &[(usize, usize, usize)], out: &mut Vec<usize>) -> bool {
    let _g = Lock::take();
    for &(_, size, align) in ls {
        let p = alloc_locked(Layout::from_size_align_unchecked(size, align));
        if p.is_null() {
            return false;
        }
        // bit 0: fresh bump memory never handed out and never snapshotted (still the OS's zero pages)
        out.push(p as usize | (FRESH && !SNAPSHOTTED.load(Relaxed)) as usize);
    }
    true
}

#[cfg(any(windows, unix))]
unsafe fn alloc_locked(l: Layout) -> *mut u8 {
    FRESH = false;
    let Some(ab) = bucket(l.align()) else { return core::ptr::null_mut() };
    let Some((c, size)) = class_of(l.size()) else { return core::ptr::null_mut() };
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
        if TRIM && size >= TRIM_MIN && !keep::take(head, size) && !recommit(head, head + size) {
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
                if !keep::take(h2, class_size(c2)) && !recommit(h2, h2 + size) {
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
    if ZERO_ALL.with(|z| z.get()) {
        note_asset_block(p, l.size() | 1 << 63);
    }
    let (Some((c, _)), Some(ab)) = (class_of(l.size()), bucket(l.align())) else { return };
    let _g = Lock::take();
    LIVE[c] -= 1;
    if *((p as usize + 8) as *const u64) == FREE_MARK {
        report_misuse("a block that is already free was freed again (double free)", p as usize, l.size());
    }
    *((p as usize + 8) as *mut u64) = FREE_MARK;
    *(p as *mut usize) = FREE[ab][c];
    FREE[ab][c] = p as usize;
    #[cfg(any(windows, unix))]
    if TRIM {
        let size = class_size(c);
        if size >= TRIM_MIN && !keep::put(p as usize, size) {
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

/// The background thread (pre-faulter / kept-block zeroer) against the snapshot. Its state lives in this module's
/// statics (`LOCK`, `COMMITTED`, the kept table), which a snapshot copies and a load writes back like every other
/// static. A save taken while the background thread held the arena lock recorded `LOCK = true`; loading it put the
/// lock back as held with no holder, and the game thread's next allocation spun on it forever (the fight-3 synctest
/// stall). A load while it held the lock also released it under it. So snapshot work runs under [`BgPause`]: it raises
/// `BG_PAUSE` and waits until the background thread is outside its critical part (`BG_BUSY` false, the lock not held
/// by it). Every save is then taken with `BG_PAUSE` true and `BG_BUSY` false, which is what a load writes back; the
/// pause's drop clears `BG_PAUSE` afterwards.
static BG_PAUSE: AtomicBool = AtomicBool::new(false);
static BG_BUSY: AtomicBool = AtomicBool::new(false);

fn bg_begin() -> bool {
    BG_BUSY.store(true, SeqCst);
    if BG_PAUSE.load(SeqCst) {
        BG_BUSY.store(false, SeqCst);
        return false;
    }
    true
}
fn bg_end() {
    BG_BUSY.store(false, SeqCst);
}
fn bg_paused() -> bool {
    BG_PAUSE.load(Relaxed)
}

/// Held around snapshot capture / save / load / hash (see [`BG_PAUSE`]). Not nested.
pub struct BgPause(());
impl BgPause {
    pub fn enter() -> BgPause {
        BG_PAUSE.store(true, SeqCst);
        while BG_BUSY.load(SeqCst) {
            std::thread::yield_now();
        }
        BgPause(())
    }
}
impl Drop for BgPause {
    fn drop(&mut self) {
        BG_PAUSE.store(false, SeqCst);
    }
}

/// Bytes past the bump pointer the pre-faulter keeps resident.
#[cfg(windows)]
const PREFAULT_AHEAD: usize = 96 << 20;

/// Start the pre-faulter (boot, once; `MELEE_ARENA_PREFAULT=off` keeps it off): a thread that keeps the pages just past
/// the bump pointer committed and resident by READING one byte of each. A fresh page's first touch is a demand-zero
/// fault (~1 us each: thousands per big archive decode or plan apply, most of their time); after the read the page is
/// already backed by a zeroed physical page, so the game thread's first write does not fault. Nothing is written: the
/// arena's bytes, blocks and write-watch state are exactly as without it (commits ahead are what `alloc_locked` would do
/// at its next step, made earlier, under the same lock).
#[cfg(windows)]
pub fn start_prefault() {
    {
        let _s = SysScope::enter();
        keep::init();
    }
    if std::env::var("MELEE_ARENA_PREFAULT").is_ok_and(|v| v == "off") {
        return;
    }
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Relaxed) {
        return;
    }
    let _s = SysScope::enter();
    let _ = std::thread::Builder::new().name("arena-prefault".into()).spawn(|| {
        mark_foreign_thread();
        extern "system" {
            fn GetCurrentThread() -> *mut core::ffi::c_void;
            fn SetThreadPriority(h: *mut core::ffi::c_void, p: i32) -> i32;
            fn GetCurrentProcess() -> *mut core::ffi::c_void;
            fn ReadProcessMemory(h: *mut core::ffi::c_void, base: *const core::ffi::c_void, buf: *mut core::ffi::c_void, n: usize, read: *mut usize) -> i32;
        }
        let me = unsafe { GetCurrentProcess() };
        unsafe { SetThreadPriority(GetCurrentThread(), -1) }; // below normal: never ahead of the game thread
        let mut touched = 0usize;
        loop {
            if !bg_begin() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            let (bump, committed) = unsafe { (core::ptr::read_volatile(&raw const BUMP), core::ptr::read_volatile(&raw const COMMITTED)) };
            if bump == 0 {
                bg_end();
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            }
            let want = bump + PREFAULT_AHEAD;
            if committed < want {
                let _g = Lock::take();
                unsafe {
                    while COMMITTED < want && COMMITTED + COMMIT_STEP <= BASE + CAP - 0x10000 {
                        if !sys::commit(COMMITTED, COMMIT_STEP) {
                            break;
                        }
                        COMMITTED += COMMIT_STEP;
                    }
                }
            }
            let committed = unsafe { core::ptr::read_volatile(&raw const COMMITTED) };
            let mut p = (touched.max(bump) + 0xFFF) & !0xFFF;
            let end = want.min(committed);
            let mut n = 0;
            while p < end && !bg_paused() {
                // Touch the page through the kernel, not with a plain load. `bump` above is a moment old: the game
                // thread can since have handed out a block here and freed it again, and a freed block of 64 KB or
                // more is decommitted when the keep list is full. A plain load of a decommitted page is an access
                // violation that ends the game (seen at match starts); this read just fails for that page and
                // faults the others in the same way.
                let mut b = 0u8;
                unsafe { ReadProcessMemory(me, p as *const _, (&raw mut b).cast(), 1, core::ptr::null_mut()) };
                p += 0x1000;
                n += 1;
                if n % 256 == 0 {
                    std::thread::yield_now();
                }
            }
            touched = touched.max(p.min(end));
            keep::zero_some();
            bg_end();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    });
}
#[cfg(not(windows))]
pub fn start_prefault() {}

/// Trim mode keeps big freed blocks RESIDENT instead of decommitting them (`MELEE_ARENA_KEEP=off` restores the
/// decommit; `MELEE_ARENA_KEEP=<MB>` sets the cap, default 256). A decommit made the block's whole pages past +16 read
/// zero at its next allocation, at the price of a demand-zero page fault per page on reuse (most of ItCo's apply time
/// every match). Here the same pages are zeroed instead: by the pre-faulter thread in 64 KB steps under the arena lock,
/// or, for what it has not reached yet, by the allocating thread when the block is handed out. Exactly the bytes a
/// decommit would drop are zeroed (whole pages inside `[p + 16, p + size)`); the bytes outside them keep their stale
/// contents exactly as before. All state changes happen under the arena lock, so a block is never written by the
/// zeroer once handed out. Over the cap (or the table full) a block is decommitted as before.
#[cfg(any(windows, unix))]
mod keep {
    use super::*;
    const N: usize = 512;
    const STEP: usize = 64 << 10;
    /// (block, class size, zeroed-up-to address; 0 = empty slot)
    static mut KEPT: [(usize, usize, usize); N] = [(0, 0, 0); N];
    static mut BYTES: usize = 0;
    /// 0: not read yet (off), 1: off, else the cap in bytes
    static mut CAP_BYTES: usize = 0;

    fn range(p: usize, size: usize) -> (usize, usize) {
        ((p + 16 + 0xFFF) & !0xFFF, (p + size) & !0xFFF)
    }
    /// Read the cap (boot, from `start_prefault`, outside the lock: an env read allocates). Until then, off.
    pub fn init() {
        let c = match std::env::var("MELEE_ARENA_KEEP") {
            Ok(v) if v == "off" => 1,
            Ok(v) => v.parse::<usize>().map_or(256 << 20, |mb| (mb << 20).max(2)),
            Err(_) => 256 << 20,
        };
        unsafe { CAP_BYTES = c };
    }
    unsafe fn cap() -> usize {
        CAP_BYTES
    }
    /// Under the lock, from `free`: keep `p` resident (true) or let the caller decommit it (false).
    pub unsafe fn put(p: usize, size: usize) -> bool {
        let c = cap();
        if c <= 1 || BYTES + size > c {
            return false;
        }
        let kept = &mut *core::ptr::addr_of_mut!(KEPT);
        let Some(slot) = kept.iter_mut().find(|e| e.0 == 0) else { return false };
        let (a, _) = range(p, size);
        *slot = (p, size, a);
        BYTES += size;
        true
    }
    /// Under the lock, from the allocation of free block `p` (its class size `size`): true when it was kept, its
    /// decommit range now zero (the pages stayed committed: no recommit needed).
    pub unsafe fn take(p: usize, size: usize) -> bool {
        if BYTES == 0 {
            return false;
        }
        let kept = &mut *core::ptr::addr_of_mut!(KEPT);
        let Some(e) = kept.iter_mut().find(|e| e.0 == p) else { return false };
        let (_, b) = range(p, size);
        if e.2 < b {
            core::ptr::write_bytes(e.2 as *mut u8, 0, b - e.2);
        }
        BYTES -= e.1;
        *e = (0, 0, 0);
        true
    }
    /// The zeroer (pre-faulter thread): zero kept blocks' ranges, one STEP per lock hold.
    pub fn zero_some() {
        let mut budget = 256usize; // at most 16 MB per call, yielding between steps
        while budget > 0 && !bg_paused() {
            budget -= 1;
            let _g = Lock::take();
            unsafe {
                if BYTES == 0 {
                    return;
                }
                let kept = &mut *core::ptr::addr_of_mut!(KEPT);
                let Some(e) = kept.iter_mut().find(|e| e.0 != 0 && e.2 < range(e.0, e.1).1) else { return };
                let b = range(e.0, e.1).1;
                let n = (b - e.2).min(STEP);
                core::ptr::write_bytes(e.2 as *mut u8, 0, n);
                e.2 += n;
            }
            drop(_g);
            std::thread::yield_now();
        }
    }
}
#[cfg(not(any(windows, unix)))]
mod keep {
    pub unsafe fn put(_: usize, _: usize) -> bool { false }
    pub unsafe fn take(_: usize, _: usize) -> bool { false }
    pub fn zero_some() {}
    pub fn init() {}
}
