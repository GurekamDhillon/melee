//! Engine state snapshots (step 17): the whole game state as memory ranges, saved as page deltas against a base.
//!
//! The ranges are the game arena (every Rust heap object the engine owns, `arena`), the OS-heap block of the
//! host (the C heaps: fighters, items, archives) and the executable's writable sections (all statics). A load
//! puts all of them back, so the engine continues exactly where it was; nothing here knows what the bytes mean.
//! Statics of the harness that must survive a load do not exist: the harness keeps its state in values it owns on
//! the system heap (see `arena`), and the few flags the load itself overwrites are put back by the caller.
//! The one exception is the statics' host-only bytes (std's, the runtime libraries' and the platform crates' objects,
//! which other threads write at any time): a load leaves them alone when the image's PDB or snapmask sidecar names
//! them (`snapmask`).
//!
//! Cost: every page carries the epoch it was last written in (the OS write watch for the arena and the host block,
//! `pagewatch` for the statics), so a save after the previous save or load reads only the pages written since and a
//! load restores only the pages written since its snapshot was taken (notes/perf.md "Rollback snapshots").
//! MELEE_SNAP_STATS=1 prints the page counters, MELEE_SNAP_CHECK=N checks every Nth save and load against the full
//! state, MELEE_SNAP_NOINC=1 compares every dirty page (the old way), MELEE_SNAP_STATIC_SCAN=1 compares the statics
//! in full instead of watching them. Linux tracks the writes with userfaultfd's async write protection (`wpwatch`),
//! soft-dirty when that is missing or MELEE_SNAP_NO_UFFD=1 is set.

use crate::arena;
use crate::host::Host;
use std::rc::Rc;

const PAGE: usize = 4096;

fn ranges() -> Vec<(usize, usize)> {
    let mut r = Vec::new();
    r.push(arena::used_range());
    let m = Host::get().os.mem;
    r.push((m.base as usize, m.len));
    r.extend(arena::static_sections());
    if std::env::var_os("MELEE_SNAP_DBG").is_some() { for x in &r { eprintln!("snap range {:#x} {:#x}", x.0, x.1); } }
    r
}

/// Pages of the first two ranges (arena, host block) are write-watched: only pages written since the base can differ.
/// Sets the bit of every page of `[addr, addr+len)` written since the last call (and resets the watch).
#[cfg(windows)]
fn written_pages(addr: usize, len: usize, base: usize, out: &mut Vec<u64>) {
    extern "system" {
        fn GetWriteWatch(flags: u32, base: *mut core::ffi::c_void, size: usize, addrs: *mut *mut core::ffi::c_void, count: *mut usize, gran: *mut u32) -> u32;
    }
    if len == 0 {
        return;
    }
    let mut buf = [core::ptr::null_mut::<core::ffi::c_void>(); 4096];
    // GetWriteWatch resumes where the reset left off: a full buffer means more pages follow in the same range
    loop {
        let mut n = buf.len();
        let mut g = 0u32;
        let r = unsafe { GetWriteWatch(1, addr as *mut _, len, buf.as_mut_ptr(), &mut n, &mut g) };
        assert_eq!(r, 0, "GetWriteWatch failed");
        for a in &buf[..n] {
            set_bit(out, (*a as usize - base) / PAGE);
        }
        if n < buf.len() {
            break;
        }
    }
}
/// Linux: the kernel's soft-dirty bits (/proc/self/pagemap bit 55, cleared by writing 4 to /proc/self/clear_refs)
/// stand in for the write watch. The clear is process-wide, so it happens once at the end of `refresh`, after every
/// range was read; a read between clears (`page_hash`'s host refresh) sees a superset, which is always safe. Without
/// soft-dirty support (or under arena trim, whose page drops leave no dirty bit) every page counts as written.
#[cfg(not(windows))]
fn written_pages(addr: usize, len: usize, base: usize, out: &mut Vec<u64>) {
    #[cfg(target_os = "linux")]
    if wpwatch::read(addr, len, base, out) || softdirty::read(addr, len, base, out) {
        return;
    }
    for p in (addr - base) / PAGE..(addr - base + len) / PAGE {
        set_bit(out, p);
    }
}

#[cfg(target_os = "linux")]
mod softdirty {
    use super::{set_bit, PAGE};
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::fs::FileExt;
    use std::sync::OnceLock;

    const SOFT_DIRTY: u64 = 1 << 55;
    static PAGEMAP: OnceLock<Option<File>> = OnceLock::new();
    /// Set by the first clear at the end of a `refresh`: only from then on do the bits cover everything written since
    /// the previous refresh (the self-test's own clear dropped the bits of earlier writes).
    static mut ARMED: bool = false;
    /// A range was read through soft-dirty since the last [`clear`] (or tried to: the first clear arms it). With every
    /// range on the userfaultfd watch (`wpwatch`) nothing is, and the process-wide clear (a walk of every page table
    /// of the process, ~1.6 ms a save on the laptop, plus a write fault on every page any thread writes next) is skipped.
    static mut NEEDED: bool = false;

    fn clear_refs() -> bool {
        std::fs::OpenOptions::new().write(true).open("/proc/self/clear_refs").and_then(|mut f| f.write_all(b"4")).is_ok()
    }

    fn entries(f: &File, addr: usize, n: usize) -> Option<Vec<u64>> {
        let mut b = vec![0u8; n * 8];
        f.read_exact_at(&mut b, (addr / PAGE * 8) as u64).ok()?;
        Some(b.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect())
    }

    /// Soft-dirty works here: after a clear, a written page reads dirty and an untouched one clean.
    fn self_test(f: &File) -> bool {
        unsafe {
            let p = libc::mmap(core::ptr::null_mut(), 2 * PAGE, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0);
            if p == libc::MAP_FAILED {
                return false;
            }
            let a = p as usize;
            core::ptr::write_volatile(a as *mut u8, 1);
            core::ptr::write_volatile((a + PAGE) as *mut u8, 1);
            let ok = clear_refs() && {
                core::ptr::write_volatile(a as *mut u8, 2);
                matches!(entries(f, a, 2), Some(e) if e[0] & SOFT_DIRTY != 0 && e[1] & SOFT_DIRTY == 0)
            };
            libc::munmap(p, 2 * PAGE);
            ok
        }
    }

    fn pagemap() -> Option<&'static File> {
        PAGEMAP
            .get_or_init(|| {
                if std::env::var_os("MELEE_SNAP_NO_SOFTDIRTY").is_some() {
                    return None;
                }
                let f = File::open("/proc/self/pagemap").ok()?;
                self_test(&f).then_some(f)
            })
            .as_ref()
    }

    /// Set the bits of the pages of `[addr, addr+len)` the kernel saw written since the last [`clear`]; false when
    /// soft-dirty cannot be used (the caller then counts every page).
    pub fn read(addr: usize, len: usize, base: usize, out: &mut Vec<u64>) -> bool {
        let Some(f) = pagemap() else { return false };
        unsafe { NEEDED = true };
        if unsafe { !ARMED } || crate::arena::trim() {
            return false;
        }
        let n = len / PAGE;
        let first = (addr - base) / PAGE;
        let mut i = 0;
        while i < n {
            let k = (n - i).min(1 << 16);
            let Some(e) = entries(f, addr + i * PAGE, k) else { return false };
            for (j, &x) in e.iter().enumerate() {
                if x & SOFT_DIRTY != 0 {
                    set_bit(out, first + i + j);
                }
            }
            i += k;
        }
        true
    }

    /// End of a refresh: every range was read, start the next interval.
    pub fn clear() {
        if unsafe { NEEDED } && pagemap().is_some() {
            unsafe {
                NEEDED = false;
                ARMED = clear_refs();
            }
        }
    }
}

/// Linux: the snapshot ranges' write tracking through userfaultfd's asynchronous write protection (kernel 6.7+:
/// UFFD_FEATURE_WP_ASYNC + the PAGEMAP_SCAN ioctl). Each range is registered once (write-protect mode, resolved in
/// the kernel: no fault thread, nothing blocks); `read` asks which of its pages were written and write-protects them
/// again in one ioctl, for that range only. That is the Windows write watch's semantics (GetWriteWatch with reset),
/// and unlike soft-dirty it needs no process-wide clear: `/proc/self/clear_refs` walks every page table of the
/// process (GPU driver, audio, heaps; ~1.6 ms a save measured) and makes every thread's next write to any page fault.
/// A page dropped and refaulted, and anything written before the registration, reads as written (a superset, safe).
/// Unprivileged (UFFD_USER_MODE_ONLY; a kernel-mode write, e.g. a read(2) into the range, still resolves and marks
/// the page). Without the kernel support, or with MELEE_SNAP_NO_UFFD, `read` says no and soft-dirty takes over.
#[cfg(target_os = "linux")]
mod wpwatch {
    use super::{set_bit, PAGE};
    use std::os::fd::AsRawFd;

    const UFFD_USER_MODE_ONLY: i32 = 1;
    const UFFD_API: u64 = 0xAA;
    const UFFD_FEATURE_WP_UNPOPULATED: u64 = 1 << 13;
    const UFFD_FEATURE_WP_ASYNC: u64 = 1 << 15;
    const UFFDIO_API: u64 = 0xC018_AA3F;
    const UFFDIO_REGISTER: u64 = 0xC020_AA00;
    const UFFDIO_REGISTER_MODE_WP: u64 = 2;
    const PAGEMAP_SCAN: u64 = 0xC060_6610;
    const PM_SCAN_WP_MATCHING: u64 = 1;
    const PM_SCAN_CHECK_WPASYNC: u64 = 2;
    const PAGE_IS_WRITTEN: u64 = 1 << 1;

    #[repr(C)]
    struct UffdioApi {
        api: u64,
        features: u64,
        ioctls: u64,
    }
    #[repr(C)]
    struct UffdioRegister {
        start: u64,
        len: u64,
        mode: u64,
        ioctls: u64,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct PageRegion {
        start: u64,
        end: u64,
        categories: u64,
    }
    #[repr(C)]
    #[derive(Default)]
    struct PmScanArg {
        size: u64,
        flags: u64,
        start: u64,
        end: u64,
        walk_end: u64,
        vec: u64,
        vec_len: u64,
        max_pages: u64,
        category_inverted: u64,
        category_mask: u64,
        category_anyof_mask: u64,
        return_mask: u64,
    }

    struct Watch {
        /// None: no userfaultfd here (old kernel, no WP_ASYNC, disabled)
        fds: Option<(std::fs::File, std::fs::File)>,
        /// the registered ranges
        regs: Vec<(usize, usize)>,
        /// the scan's output buffer (kept: a read allocates nothing)
        buf: Vec<PageRegion>,
    }

    // on the system heap (SysStatic): a rollback load rewinds the statics, and this one owns heap memory
    static W: crate::arena::SysStatic<Watch> = crate::arena::SysStatic::new(|| Watch { fds: open(), regs: Vec::new(), buf: vec![PageRegion::default(); 512] });

    fn open() -> Option<(std::fs::File, std::fs::File)> {
        use std::os::fd::FromRawFd;
        if std::env::var_os("MELEE_SNAP_NO_UFFD").is_some() {
            return None;
        }
        unsafe {
            let fd = libc::syscall(libc::SYS_userfaultfd, libc::O_CLOEXEC | libc::O_NONBLOCK | UFFD_USER_MODE_ONLY) as i32;
            if fd < 0 {
                return None;
            }
            let u = std::fs::File::from_raw_fd(fd);
            let mut api = UffdioApi { api: UFFD_API, features: UFFD_FEATURE_WP_ASYNC | UFFD_FEATURE_WP_UNPOPULATED, ioctls: 0 };
            if libc::ioctl(fd, UFFDIO_API as _, &mut api) != 0 {
                return None;
            }
            let pm = std::fs::File::open("/proc/self/pagemap").ok()?;
            Some((u, pm))
        }
    }

    /// Register the ranges not registered yet (page aligned; a range that fails stays on soft-dirty).
    pub fn arm(ranges: &[(usize, usize)]) {
        W.with(|w| {
            let Some((u, _)) = &w.fds else { return };
            let fd = u.as_raw_fd();
            for &(a, l) in ranges {
                if l == 0 || a % PAGE != 0 || l % PAGE != 0 || w.regs.iter().any(|&(ra, rl)| a >= ra && a + l <= ra + rl) {
                    continue;
                }
                let mut r = UffdioRegister { start: a as u64, len: l as u64, mode: UFFDIO_REGISTER_MODE_WP, ioctls: 0 };
                if unsafe { libc::ioctl(fd, UFFDIO_REGISTER as _, &mut r) } == 0 {
                    w.regs.push((a, l));
                } else if std::env::var_os("MELEE_SNAP_STATS").is_some() {
                    eprintln!("snapshot: write watch: range {:#x}+{:#x} not registered ({}), soft-dirty there", a, l, std::io::Error::last_os_error());
                }
            }
        })
    }

    /// Set the bits of the pages of `[addr, addr+len)` written since the last read of them, and protect them again;
    /// false when the range is not on the watch (the caller then asks soft-dirty).
    pub fn read(addr: usize, len: usize, base: usize, out: &mut Vec<u64>) -> bool {
        if len == 0 {
            return true;
        }
        if crate::arena::trim() {
            return false;
        }
        W.with(|w| {
            let Some((_, pm)) = &w.fds else { return false };
            if !w.regs.iter().any(|&(ra, rl)| addr >= ra && addr + len <= ra + rl) {
                return false;
            }
            let end = (addr + len) as u64;
            let mut at = addr as u64;
            while at < end {
                let mut arg = PmScanArg {
                    size: core::mem::size_of::<PmScanArg>() as u64,
                    flags: PM_SCAN_WP_MATCHING | PM_SCAN_CHECK_WPASYNC,
                    start: at,
                    end,
                    vec: w.buf.as_mut_ptr() as u64,
                    vec_len: w.buf.len() as u64,
                    category_mask: PAGE_IS_WRITTEN,
                    return_mask: PAGE_IS_WRITTEN,
                    ..Default::default()
                };
                let n = unsafe { libc::ioctl(pm.as_raw_fd(), PAGEMAP_SCAN as _, &mut arg) };
                if n < 0 {
                    // a failed scan mid-range: every page of the rest counts as written (the pages before were set)
                    for p in (at as usize - base) / PAGE..(end as usize - base) / PAGE {
                        set_bit(out, p);
                    }
                    return true;
                }
                for r in &w.buf[..n as usize] {
                    for p in (r.start as usize - base) / PAGE..(r.end as usize - base) / PAGE {
                        set_bit(out, p);
                    }
                }
                if arg.walk_end <= at {
                    break;
                }
                at = arg.walk_end;
            }
            true
        })
    }
}

/// Linux: keep the snapshot ranges on 4 KB pages. With transparent huge pages on (`always`, a common default), a 2 MB
/// huge page is one soft-dirty unit: one write in it marks all 512 of its pages written, so each save copied and each
/// state hash digested ~8x the pages the game wrote (measured: a two-peer netplay match, 2470 pages copied per save,
/// ~1000 of them in the statics' .data/.bss alone; 300 on 4 KB pages). MADV_NOHUGEPAGE stops new huge pages there (the
/// arena reserve and the host block are advised so at their mmap, before the first touch); a huge page already in
/// place (the statics are touched long before a match) is split by MADV_COLD on 4 KB of it (the kernel splits a huge
/// page a cold hint covers only in part; the contents and the mapping stay, only the page's LRU age changes). Best
/// effort: an unsplit page only costs time, never correctness (soft-dirty then reports a superset).
#[cfg(target_os = "linux")]
fn small_pages(ranges: &[(usize, usize)]) {
    const HUGE: usize = 2 << 20;
    for &(a, l) in ranges {
        let (s, e) = (a & !(PAGE - 1), (a + l + PAGE - 1) & !(PAGE - 1));
        if e <= s {
            continue;
        }
        unsafe {
            libc::madvise(s as *mut _, e - s, libc::MADV_NOHUGEPAGE);
            let mut h = s & !(HUGE - 1);
            while h < e {
                let p = h.max(s);
                libc::madvise(p as *mut _, PAGE, libc::MADV_COLD);
                h += HUGE;
            }
        }
    }
}

/// The arena's copy-on-write part is write-watched by the OS too (it is reserved MEM_WRITE_WATCH). The cow bits say
/// which pages were ever written; `refresh` reads (and resets) the watch over just the runs of those (the reserve is
/// ~5 GB), to learn which were written since the previous save or load.
fn written_runs(cum: &[u64], limit: usize, addr: usize, out: &mut Vec<u64>) {
    let bit = |i: usize| cum.get(i / 64).copied().unwrap_or(0) >> (i % 64) & 1 != 0;
    // Linux: one PAGEMAP_SCAN over the span from the first to the last run instead of one per run (a refresh read
    // ~20-40 short runs, each an ioctl; the kernel skips the unpopulated page tables between them). The span reports
    // only pages written since the last read, and outside the runs nothing can have been written (a write to an arena
    // page the cow store does not hold yet faults into it first and sets its bit), so the bits are the same.
    #[cfg(target_os = "linux")]
    {
        let words = limit.div_ceil(64).min(cum.len());
        let first = cum[..words].iter().position(|&w| w != 0);
        let last = cum[..words].iter().rposition(|&w| w != 0);
        let (Some(fw), Some(lw)) = (first, last) else { return };
        let lo = fw * 64 + cum[fw].trailing_zeros() as usize;
        let hi = (lw * 64 + 64 - cum[lw].leading_zeros() as usize).min(limit);
        if lo < hi {
            let mut span = Vec::new();
            written_pages(addr + lo * PAGE, (hi - lo) * PAGE, addr, &mut span);
            // keep the result to the runs, as the per-run reads did
            for (i, w) in span.iter().enumerate() {
                let m = w & cum.get(i).copied().unwrap_or(0);
                if m != 0 {
                    if out.len() <= i {
                        out.resize(i + 1, 0);
                    }
                    out[i] |= m;
                }
            }
        }
        return;
    }
    #[allow(unreachable_code)]
    let mut i = 0usize;
    while i < limit {
        if i % 64 == 0 && cum.get(i / 64).copied().unwrap_or(0) == 0 {
            i += 64;
            continue;
        }
        if !bit(i) {
            i += 1;
            continue;
        }
        let s = i;
        // one call over runs a short gap apart: a page of the gap was never written (no cow bit), so its watch has
        // nothing to report or reset, and the result is the same with fewer GetWriteWatch calls (each scans little
        // but costs a kernel entry: the arena's written pages lie in many short runs)
        loop {
            while i < limit && bit(i) {
                i += 1;
            }
            let mut j = i;
            while j < limit && j < i + RUN_GAP && !bit(j) {
                j += 1;
            }
            if j < limit && j < i + RUN_GAP {
                i = j;
                continue;
            }
            break;
        }
        written_pages(addr + s * PAGE, (i - s) * PAGE, addr, out);
    }
}

/// [`written_runs`]: runs of written pages at most this many pages apart are read in one call (Windows; elsewhere a
/// fallback marks a whole range written, so the runs stay apart there).
#[cfg(windows)]
const RUN_GAP: usize = 64;
#[cfg(not(windows))]
const RUN_GAP: usize = 0;

fn set_bit(v: &mut Vec<u64>, i: usize) {
    if v.len() <= i / 64 {
        v.resize(i / 64 + 1, 0);
    }
    v[i / 64] |= 1u64 << (i % 64);
}

/// Indexes of the set bits, ascending, below `limit`.
fn bits(v: &[u64], limit: usize) -> Vec<u32> {
    let mut out = Vec::new();
    for (w, &x) in v.iter().enumerate() {
        let mut x = x;
        while x != 0 {
            let i = w * 64 + x.trailing_zeros() as usize;
            if i >= limit {
                return out;
            }
            out.push(i as u32);
            x &= x - 1;
        }
    }
    out
}

/// Ranges 0 and 1 are write-watched by the OS (the arena's copy-on-write part through `written_runs`); the static
/// ranges (2..) through `pagewatch` (read-only pages, first write faults), or compared in full under
/// MELEE_SNAP_STATIC_SCAN.
const HOST: usize = 1;
const STATIC0: usize = 2;
/// A game-crate static only MELEE_SNAP_CHECK_POKE writes (see `load`).
static CHECK_POKE: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// A static page that faulted stays writable ("hot", counted as written in every interval) for this many intervals,
/// then is made read-only again: a page written every frame faults once per HOT_AGE saves/loads, not every frame.
const HOT_AGE: u8 = 32;

/// MELEE_SNAP_STATS counters.
#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    pub saves: u64,
    pub loads: u64,
    /// Pages a save looked at (written since the previous snapshot, or every dirty page when not incremental).
    pub save_cand: u64,
    /// Pages a save copied into a fresh buffer.
    pub save_copied: u64,
    /// Pages a save shared with the previous snapshot without reading them.
    pub save_shared: u64,
    pub save_full: u64,
    pub load_cand: u64,
    pub load_copied: u64,
    pub load_full: u64,
    /// Static pages that faulted (made hot) / were made read-only again.
    pub static_faults: u64,
    pub static_reprotects: u64,
    /// MELEE_SNAP_CHECK: operations checked, pages checked.
    pub checks: u64,
    pub checked_pages: u64,
    /// page_hash calls, those answered from the previous result, pages digested.
    pub hashes: u64,
    pub hashes_cached: u64,
    pub hash_digests: u64,
}

/// The reference every snapshot is a delta of (the state when the first snapshot was taken).
///
/// Every page of every range carries the epoch it was last written in (`stamp`; an epoch ends at each save and
/// load). A snapshot remembers the epoch it was taken at, so a load of it only visits the pages written since then
/// (an untouched page still holds the snapshot's contents), and a save after a save or load of `last` only visits the
/// pages written since (the rest are `last`'s pages, shared). Neither depends on how much was ever written.
pub struct Base {
    ranges: Vec<(usize, usize)>,
    /// Full copies of every range but the arena (range 0), whose base is kept copy-on-first-write (`cow`).
    data: Vec<Vec<u8>>,
    /// Bytes of the arena that existed at capture (pages beyond were zero then).
    arena_len: usize,
    /// Every page written since the base (cumulative), per range.
    dirty: std::cell::RefCell<Vec<Vec<u64>>>,
    /// Pages written in the current epoch (since the last save or load), per range.
    since: std::cell::RefCell<Vec<Vec<u64>>>,
    /// Per range and page: the last epoch the page was written in (0: not since the base); and per 64 pages the
    /// latest of theirs (a load scans the ~5 GB arena reserve by blocks).
    stamp: std::cell::RefCell<Vec<(Vec<u32>, Vec<u32>)>>,
    /// The current (open) epoch.
    epoch: std::cell::Cell<u32>,
    /// The snapshot the live state was last saved as or loaded from (its id; 0: none), and the snapshot itself while
    /// someone still holds it (a plain `save` shares its unchanged pages).
    last: std::cell::Cell<u64>,
    last_snap: std::cell::RefCell<std::rc::Weak<Snapshot>>,
    /// Static ranges: writable pages (bits) and their age in intervals.
    hot: std::cell::RefCell<Vec<(Vec<u64>, Vec<u8>)>>,
    /// The statics are fault-watched (else compared in full).
    static_watch: bool,
    /// Page buffers of dropped snapshots, reused by the next saves (no allocation per page in steady state).
    pool: Pool,
    /// The next snapshot id. Here, not a static: a load puts the statics back, and a static counter would hand out
    /// an id again (the rollback statics rule).
    next_id: std::cell::Cell<u64>,
    /// MELEE_SNAP_NOINC: no epochs (every save compares every dirty page, every load visits every dirty page): the
    /// full-compare reference.
    no_inc: bool,
    /// MELEE_SNAP_CHECK=N: every Nth save and load is checked page by page against the full state (see `check`).
    check_every: u64,
    /// The statics' host-only bytes (snapmask.rs): a load leaves them alone and the load check skips them.
    host_mask: Option<crate::snapmask::HostMask>,
    /// MELEE_SNAP_CHECK_POKE: write [`CHECK_POKE`] before each load check (the mask's self-test).
    poke: bool,
    /// Per-page digests of the hashed range (the host block), so `page_hash` only rehashes pages written since its
    /// previous call.
    hcache: std::cell::RefCell<HashCache>,
    /// MELEE_HASH_VERIFY: recompute every digest on each `page_hash` and panic if a cached one is stale.
    hash_verify: bool,
    /// [`Base::save_after_hash`]: the next `refresh` leaves the host block's write watch alone (`page_hash` read and
    /// reset it just before, with nothing run in between).
    skip_host: core::cell::Cell<bool>,
    pub stats: std::cell::RefCell<Stats>,
}

/// Digest cache for the hashed range. `stale` collects every page the write watch reported since the last
/// `page_hash`; a page whose digest was never computed counts as stale too.
#[derive(Default)]
struct HashCache {
    /// None: equal to the base page (not mixed in). Indexed by page; `known` says which entries are filled.
    digest: Vec<Option<u64>>,
    known: Vec<u64>,
    stale: Vec<u64>,
    /// Digests of the base pages (computed on first use; the base never changes): a live page whose digest equals
    /// its base page's is not mixed in, without reading the base copy.
    base: Vec<Option<u64>>,
    /// The previous result: no page written since means the same hash.
    last: Option<u32>,
}

/// Range whose pages `page_hash` covers (the OS-heap block).
const HASHED: usize = HOST;

fn or_into(dst: &mut Vec<u64>, src: &[u64]) {
    if dst.len() < src.len() {
        dst.resize(src.len(), 0);
    }
    for (a, b) in dst.iter_mut().zip(src) {
        *a |= *b;
    }
}

/// Buffers for the next [`Base::capture`]'s copies (index = range), allocated and touched ahead by [`prewarm`].
/// (Off the snapshot: the helper thread can finish after a capture, and a load must not rewind these buffers.)
static PREWARM: crate::arena::SysStatic<Vec<Vec<u8>>> = crate::arena::SysStatic::new(Vec::new);

/// Make the next capture's copy buffers ahead, on a helper thread (system heap), every page touched: a capture at a
/// netplay match start otherwise spends ~40 ms in the first-touch faults of ~its fresh buffers, which more threads do
/// not speed up. Presentation-neutral: the buffers hold nothing until the capture copies the state into them.
pub fn prewarm() {
    let sizes: Vec<usize> = ranges().iter().map(|r| r.1).collect();
    if PREWARM.with(|g| g.len() == sizes.len() && g.iter().zip(&sizes).skip(1).all(|(v, &l)| v.len() == l)) {
        return;
    }
    let _s = crate::arena::SysScope::enter();
    let _ = std::thread::Builder::new().name("snapshot-prewarm".into()).spawn(move || {
        let bufs: Vec<Vec<u8>> = sizes
            .iter()
            .enumerate()
            .map(|(i, &l)| {
                if i == 0 {
                    return Vec::new(); // the arena is copy-on-write, never copied
                }
                let mut v = Vec::<u8>::with_capacity(l);
                // SAFETY: every byte is written before the length covers it
                unsafe {
                    core::ptr::write_bytes(v.as_mut_ptr(), 0, l);
                    v.set_len(l);
                }
                v
            })
            .collect();
        PREWARM.with(|g| *g = bufs);
    });
}

/// The host block's base is copied by a helper thread behind a write fault (cow::bg; MELEE_SNAP_BGCOPY=0: on the game
/// thread at capture, as before).
#[cfg(windows)]
fn bg_copy() -> bool {
    std::env::var("MELEE_SNAP_BGCOPY").map_or(true, |v| v != "0")
}

/// The host block's write faults while its base was being copied in the background (stats line).
fn bg_faults_note() -> String {
    #[cfg(windows)]
    if crate::cow::bg::active() || crate::cow::bg::faults() > 0 {
        return format!("; host base copy: {} write faults", crate::cow::bg::faults());
    }
    String::new()
}

/// Copy `src` into `dst` (same length), split over a few threads when big.
fn par_fill(dst: &mut [u8], src: &[u8]) {
    const THREADS: usize = 4;
    if src.len() < (4 << 20) {
        dst.copy_from_slice(src);
        return;
    }
    let per = src.len().div_ceil(THREADS).next_multiple_of(PAGE);
    let _s = crate::arena::SysScope::enter();
    std::thread::scope(|sc| {
        for (d, s) in dst.chunks_mut(per).zip(src.chunks(per)) {
            sc.spawn(move || d.copy_from_slice(s));
        }
    });
}

/// `src.to_vec()`, the copy split over a few threads when it is big (the base capture copies the host block and the
/// statics at a netplay match start: ~37 ms on one thread, most of it the fresh pages' first-touch faults). The buffer
/// is allocated where `to_vec` would allocate it (the caller's heap scope); only the threads' own spawns are kept off
/// the arena.
fn par_copy(src: &[u8]) -> Vec<u8> {
    const MIN: usize = 4 << 20;
    const THREADS: usize = 6;
    if src.len() < MIN {
        return src.to_vec();
    }
    let mut v: Vec<u8> = Vec::with_capacity(src.len());
    {
        let dst = &mut v.spare_capacity_mut()[..src.len()];
        let per = src.len().div_ceil(THREADS).next_multiple_of(PAGE);
        let _s = crate::arena::SysScope::enter();
        std::thread::scope(|sc| {
            for (d, s) in dst.chunks_mut(per).zip(src.chunks(per)) {
                sc.spawn(move || {
                    // SAFETY: MaybeUninit<u8> and u8 have the same layout; every byte of `d` is written here
                    unsafe { core::ptr::copy_nonoverlapping(s.as_ptr(), d.as_mut_ptr() as *mut u8, s.len()) };
                });
            }
        });
    }
    // SAFETY: the threads above wrote all `src.len()` bytes
    unsafe { v.set_len(src.len()) };
    v
}

fn clear_bit(v: &mut [u64], i: usize) {
    if let Some(w) = v.get_mut(i / 64) {
        *w &= !(1u64 << (i % 64));
    }
}

/// [`Base::page_hash`] digests on [`PAR_DIGEST_THREADS`] threads from this many pages to digest.
const PAR_DIGEST_MIN: usize = 2048;
const PAR_DIGEST_THREADS: usize = 6;

fn bit_of(v: &[u64], i: usize) -> bool {
    v.get(i / 64).is_some_and(|w| w >> (i % 64) & 1 != 0)
}

/// Digest of one page: the xxh3 accumulate step (eight 64-bit lanes, a 32x32->64 multiply each, which the compiler
/// vectorises) over 64-byte stripes, then a multiply-fold of the lanes. Only compared within one run.
fn page_digest(pi: usize, p: &[u8]) -> u64 {
    const KEY: [u64; 8] = [
        0xbe4ba423396cfeb8, 0x1cad21f72c81017c, 0xdb979083e96dd4de, 0x1f67b3b7a4a44072,
        0x78e5c0cc4ee679cb, 0x2172ffcc7dd05a82, 0x8e2443f7744608b8, 0x4c263a81e69035e0,
    ];
    let seed = (pi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut acc = [0x9E37_79B1u64, 0xC2B2_AE3D_27D4_EB4F, 0x1656_67B1_9E37_79F9, 0x85EB_CA77_C2B2_AE63,
        0x27D4_EB2F_1656_67C5, 0x9E37_79B9_7F4A_7C15, 0xFF51_AFD7_ED55_8CCD, 0xC4CE_B9FE_1A85_EC53];
    for q in p.chunks_exact(64) {
        let mut v = [0u64; 8];
        for k in 0..8 {
            v[k] = u64::from_le_bytes(q[k * 8..k * 8 + 8].try_into().unwrap());
        }
        for k in 0..8 {
            let d = v[k] ^ (KEY[k] ^ seed);
            acc[k ^ 1] = acc[k ^ 1].wrapping_add(v[k]);
            acc[k] = acc[k].wrapping_add((d & 0xFFFF_FFFF).wrapping_mul(d >> 32));
        }
    }
    let mut h = seed ^ 0x2D35_8DCC_AA6C_78A5;
    for k in 0..8 {
        h = (h ^ acc[k]).wrapping_mul(0x100000001b3).rotate_left(29) ^ (acc[k] >> 31);
    }
    h ^ (h >> 33)
}

type Pool = Rc<PagePool>;
/// Most page buffers kept for reuse (64 MB, the Lab ring's churn; more are freed).
const POOL_MAX: usize = 16384;

/// The page buffers kept for reuse. When the last snapshot of a session goes (the session's end: the online loop's
/// match -> select), its buffers are freed on the page reaper thread ([`reap_pages`]), not the game thread.
pub struct PagePool(std::cell::RefCell<Vec<Rc<[u8; PAGE]>>>);
impl std::ops::Deref for PagePool {
    type Target = std::cell::RefCell<Vec<Rc<[u8; PAGE]>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for PagePool {
    fn drop(&mut self) {
        reap_pages(std::mem::take(self.0.get_mut()));
    }
}

/// Unshared page buffers on their way to the reaper thread. Each `Rc` in it has strong count 1 and no weak
/// reference ([`reap_pages`] checks), so nothing on the sending thread can touch its counts again: moving it to
/// another thread and dropping it there is sound although `Rc` is not `Send`.
struct Reaped(Vec<Rc<[u8; PAGE]>>);
unsafe impl Send for Reaped {}

/// The page reaper's channel. Host state off the snapshot (`arena::SysStatic`): a rollback load rewinds only the
/// pointer to it (the same value, or null before its first use: a second reaper is started then, the old one idles).
static REAPER: arena::SysStatic<Option<std::sync::mpsc::Sender<Reaped>>> = arena::SysStatic::new(|| None);

/// `MELEE_SNAP_REAPER=off`: free on the calling thread, as before (A/B). Read once (under SysScope by the caller).
fn reaper_off() -> bool {
    static S: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);
    match S.load(core::sync::atomic::Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let off = std::env::var("MELEE_SNAP_REAPER").is_ok_and(|v| v == "off");
            S.store(if off { 2 } else { 1 }, core::sync::atomic::Ordering::Relaxed);
            off
        }
    }
}

/// Free page buffers on a background thread. A session's snapshots hold tens of thousands of 4 KB buffers; freeing
/// them at its end took 25-40 ms of the game thread on the in-process online loop's match -> select tick (the laptop,
/// power-saver). They are system-heap buffers no game state refers to (snapshot copies), so when they are freed
/// changes nothing the game can see. Shared buffers (another snapshot still holds them) are only released here.
fn reap_pages(pages: Vec<Rc<[u8; PAGE]>>) {
    let t0 = std::time::Instant::now();
    let n = pages.len();
    if n < 256 {
        drop(pages); // a few: freed here (a steady-state ring's churn)
        if n > 0 && arena::rb_diag() {
            arena::rb_diag_log("reap: freed in place", n, t0.elapsed().as_micros() as u64);
        }
        return;
    }
    let _s = arena::SysScope::enter();
    if reaper_off() {
        drop(pages);
        return;
    }
    let (unique, shared): (Vec<_>, Vec<_>) = pages.into_iter().partition(|p| Rc::strong_count(p) == 1 && Rc::weak_count(p) == 0);
    drop(shared);
    let mut job = Some(Reaped(unique));
    REAPER.with(|r| {
        if r.is_none() {
            let (tx, rx) = std::sync::mpsc::channel::<Reaped>();
            let ok = std::thread::Builder::new()
                .name("snapshot-reaper".into())
                .spawn(move || {
                    arena::mark_foreign_thread();
                    for Reaped(v) in rx {
                        // never during a fight (recheap::in_fight): the frees wait for its end, then go in steps,
                        // yielding: never a long hold of the allocator's lock against the game thread
                        while crate::recheap::in_fight() {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        let mut v = v;
                        while !v.is_empty() {
                            let n = v.len().saturating_sub(512);
                            drop(v.split_off(n));
                            std::thread::yield_now();
                        }
                    }
                })
                .is_ok();
            if ok {
                *r = Some(tx);
            }
        }
        if let Some(tx) = r.as_ref() {
            if let Err(e) = tx.send(job.take().unwrap()) {
                job = Some(e.0);
            }
        }
    });
    let inline = job.is_some();
    drop(job); // no reaper: freed here, as before
    if arena::rb_diag() {
        arena::rb_diag_log(if inline { "reap: no reaper, freed in place" } else { "reap: sent to the reaper" }, n, t0.elapsed().as_micros() as u64);
    }
}

#[derive(Clone)]
struct RangeSnap {
    addr: usize,
    len: usize,
    /// Shared: a snapshot reuses the previous snapshot's unchanged pages.
    pages: Vec<(u32, Rc<[u8; PAGE]>)>,
}

impl RangeSnap {
    fn page(&self, pi: u32) -> Option<&[u8]> {
        self.pages.binary_search_by_key(&pi, |p| p.0).ok().map(|i| &self.pages[i].1[..])
    }
}

/// One saved state.
#[derive(Clone)]
pub struct Snapshot {
    ranges: Vec<RangeSnap>,
    id: u64,
    /// The epoch it was taken in: pages stamped later may differ from it.
    epoch: u32,
    me: std::rc::Weak<Snapshot>,
    pool: Pool,
}

impl Drop for Snapshot {
    /// Pages no other snapshot shares go back to the pool.
    fn drop(&mut self) {
        let t0 = std::time::Instant::now();
        let Ok(mut pool) = self.pool.try_borrow_mut() else { return };
        // unshared pages past the pool's cap go to the reaper (in bulk: a session's end) instead of being freed here
        let mut spill = Vec::new();
        for r in &mut self.ranges {
            for (_, p) in r.pages.drain(..) {
                if Rc::strong_count(&p) == 1 && Rc::weak_count(&p) == 0 {
                    if pool.len() < POOL_MAX {
                        pool.push(p);
                    } else {
                        let _s = arena::SysScope::enter();
                        spill.push(p);
                    }
                }
            }
        }
        let np = pool.len();
        drop(pool);
        let _s = arena::SysScope::enter();
        let ns = spill.len();
        {
            let _z = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || {
                melee_core::frameprof::Text::new().s("snapshot drop spill ").n(ns as i64)
            });
            reap_pages(spill);
        }
        if ns > 0 && arena::rb_diag() {
            arena::rb_diag_log(&format!("snapshot drop: spill (pool {np})"), ns, t0.elapsed().as_micros() as u64);
        }
    }
}

pub type SnapRef = Rc<Snapshot>;

impl Snapshot {
    /// Bytes of page data the snapshot refers to (its delta against the base, shared pages included).
    pub fn bytes(&self) -> usize {
        self.ranges.iter().map(|r| r.pages.len() * PAGE).sum()
    }

    /// The addresses of the page buffers it refers to (to count the distinct bytes a set of snapshots holds).
    pub fn page_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.ranges.iter().flat_map(|r| r.pages.iter().map(|p| Rc::as_ptr(&p.1) as usize))
    }
}

unsafe fn bytes(addr: usize, len: usize) -> &'static [u8] {
    core::slice::from_raw_parts(addr as *const u8, len)
}

const ZERO: [u8; PAGE] = [0; PAGE];

impl Drop for Base {
    fn drop(&mut self) {
        if std::env::var_os("MELEE_SNAP_STATS").is_some() {
            eprintln!("snapshot: {}", self.stats_line());
        }
        unsafe {
            if self.static_watch {
                crate::pagewatch::release();
            }
            #[cfg(windows)]
            crate::cow::bg::release_copy();
            crate::cow::release()
        };
    }
}

/// Merge two ascending index lists into their union.
fn union(a: &[u32], b: impl Iterator<Item = u32>) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len() + 16);
    let mut i = 0;
    for x in b {
        while i < a.len() && a[i] < x {
            out.push(a[i]);
            i += 1;
        }
        if i < a.len() && a[i] == x {
            i += 1;
        }
        out.push(x);
    }
    out.extend_from_slice(&a[i..]);
    out
}

impl Base {
    /// Capture the current state as the base (the system heap holds the copy).
    pub fn capture() -> Base {
        let _bg = crate::arena::BgPause::enter();
        crate::arena::note_snapshot();
        let ranges = ranges();
        #[cfg(target_os = "linux")]
        {
            small_pages(&ranges);
            // the arena by its whole reservation (it grows inside it); the refresh below takes the first, full report
            let mut w = vec![arena::reserved_range()];
            w.extend(ranges[HOST..].iter().copied());
            wpwatch::arm(&w);
        }
        // the copies are taken last, once every write is tracked (a write between a copy and the start of tracking
        // would leave a page that differs from its base copy without being dirty)
        let data: Vec<Vec<u8>> = vec![Vec::new(); ranges.len()];
        let arena_len = ranges[0].1;
        let n = ranges.len();
        let scan = std::env::var_os("MELEE_SNAP_STATIC_SCAN").is_some();
        let host_mask = if std::env::var_os("MELEE_SNAP_CHECK_NOMASK").is_none() { crate::snapmask::host_only(&ranges, STATIC0) } else { None };
        let b = Base {
            dirty: std::cell::RefCell::new(vec![Default::default(); n]),
            since: std::cell::RefCell::new(vec![Default::default(); n]),
            stamp: std::cell::RefCell::new(ranges.iter().map(|r| (vec![0u32; r.1 / PAGE], vec![0u32; (r.1 / PAGE).div_ceil(64)])).collect()),
            epoch: std::cell::Cell::new(1),
            last: std::cell::Cell::new(0),
            last_snap: Default::default(),
            hot: std::cell::RefCell::new(ranges[STATIC0.min(n)..].iter().map(|r| (vec![0u64; (r.1 / PAGE).div_ceil(64)], vec![0u8; r.1 / PAGE])).collect()),
            static_watch: false,
            ranges, data, arena_len,
            // reserved now: a push never reallocates, wherever a snapshot is dropped
            pool: Rc::new(PagePool(std::cell::RefCell::new(Vec::with_capacity(POOL_MAX)))), next_id: std::cell::Cell::new(1),
            no_inc: std::env::var_os("MELEE_SNAP_NOINC").is_some(),
            check_every: std::env::var("MELEE_SNAP_CHECK").ok().map(|v| v.parse().unwrap_or(1).max(1)).unwrap_or(0),
            host_mask,
            poke: std::env::var_os("MELEE_SNAP_CHECK_POKE").is_some(),
            hcache: Default::default(),
            hash_verify: std::env::var_os("MELEE_HASH_VERIFY").is_some(),
            skip_host: core::cell::Cell::new(false),
            stats: Default::default(),
        };
        let mut b = b;
        let _z1 = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 500, "snapshot capture: refresh");
        b.refresh(); // consumes the watch state up to now (the copy above only read)
        // both sets: a since-last set left full here is OR-ed into `dirty` by every later refresh (the whole host block
        // stayed a candidate forever: f82738ca3 made save and load ~7 ms)
        b.dirty.borrow_mut().iter_mut().for_each(|d| d.iter_mut().for_each(|w| *w = 0));
        b.since.borrow_mut().iter_mut().for_each(|d| d.iter_mut().for_each(|w| *w = 0));
        drop(_z1);
        let _z2 = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 500, "snapshot capture: arena protect");
        // the arena's base is not copied: the first write to each page saves it (cow.rs)
        unsafe { crate::cow::protect(b.ranges[0].0, arena_len).expect("arena copy-on-write") };
        drop(_z2);
        let _z3 = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 500, "snapshot capture: statics watch");
        if !scan {
            match unsafe { crate::pagewatch::watch(&b.ranges[STATIC0.min(n)..]) } {
                Ok(()) => b.static_watch = true,
                Err(e) => eprintln!("snapshot: statics compared in full ({e})"),
            }
        }
        drop(_z3);
        let _z4 = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 500, "snapshot capture: copies");
        let mut warm = PREWARM.with(std::mem::take);
        for i in 1..n {
            let (a, l) = b.ranges[i];
            // the host block (64 MB, most of the copy): read-only now, copied by a helper thread (cow::bg)
            #[cfg(windows)]
            if i == HOST && bg_copy() && l % PAGE == 0 {
                let mut v = match warm.get_mut(i).map(std::mem::take) {
                    Some(v) if v.len() == l => v,
                    _ => vec![0u8; l],
                };
                match unsafe { crate::cow::bg::begin(a, l, v.as_mut_ptr()) } {
                    Ok(()) => {
                        b.data[i] = v;
                        continue;
                    }
                    Err(e) => eprintln!("snapshot: host block copied in place ({e})"),
                }
            }
            let src = unsafe { bytes(a, l) };
            b.data[i] = match warm.get_mut(i).map(std::mem::take) {
                Some(mut v) if v.len() == l => {
                    par_fill(&mut v, src);
                    v
                }
                _ => par_copy(src),
            };
        }
        drop(_z4);
        b
    }

    /// Pull the pages written since the last call into `since` (and the cumulative `dirty`). The arena part that
    /// existed at the capture is copy-on-first-write (`cow`): a page ever written there has its bit in the cow store,
    /// and the OS write watch is read over just the runs of those (the full reserve is ~5 GB); the arena's growth and
    /// the host block are watched in full; the statics come from `pagewatch` plus the hot pages.
    fn refresh(&self) {
        let cur = self.cur_ranges();
        let mut d = self.dirty.borrow_mut();
        let mut since = self.since.borrow_mut();
        let (a0, l0) = cur[0];
        let cow_len = self.arena_len.min(l0);
        crate::cow::or_written(&mut d[0]);
        written_runs(&d[0], cow_len / PAGE, a0, &mut since[0]);
        if l0 > cow_len {
            written_pages(a0 + cow_len, l0 - cow_len, a0, &mut since[0]);
        }
        drop((d, since));
        if !self.skip_host.replace(false) {
            self.refresh_host();
        }
        let mut d = self.dirty.borrow_mut();
        let mut since = self.since.borrow_mut();
        let mut hot = self.hot.borrow_mut();
        for ri in STATIC0..cur.len() {
            let np = cur[ri].1 / PAGE;
            if self.static_watch {
                let mut f = Vec::new();
                crate::pagewatch::take(ri - STATIC0, &mut f);
                let (hb, age) = &mut hot[ri - STATIC0];
                for pi in bits(&f, np) {
                    if !bit_of(hb, pi as usize) {
                        self.stats.borrow_mut().static_faults += 1;
                    }
                    set_bit(hb, pi as usize);
                    age[pi as usize] = 0;
                }
                or_into(&mut since[ri], hb);
            } else {
                // Linux: no fault-based static watch, but the kernel's soft-dirty bits cover the statics too
                #[cfg(target_os = "linux")]
                if std::env::var_os("MELEE_SNAP_STATIC_SCAN").is_none()
                    && (wpwatch::read(cur[ri].0, cur[ri].1, cur[ri].0, &mut since[ri]) || softdirty::read(cur[ri].0, cur[ri].1, cur[ri].0, &mut since[ri]))
                {
                    continue;
                }
                for pi in 0..np {
                    set_bit(&mut since[ri], pi);
                }
            }
        }
        for ri in 0..cur.len() {
            let (s, dd) = (&since[ri], &mut d[ri]);
            or_into(dd, s);
        }
        #[cfg(target_os = "linux")]
        softdirty::clear();
    }

    /// The ranges now: the arena's used part moves, the others are fixed at capture.
    fn cur_ranges(&self) -> Vec<(usize, usize)> {
        let mut r = self.ranges.clone();
        r[0] = arena::used_range();
        r
    }

    /// The host block's part of `refresh` (all `page_hash` needs: the other ranges' watches keep their state until
    /// the next save or load reads them).
    fn refresh_host(&self) {
        let (a, l) = { let m = Host::get().os.mem; (m.base as usize, m.len) };
        let mut delta = Vec::new();
        written_pages(a, l.max(PAGE), a, &mut delta);
        or_into(&mut self.since.borrow_mut()[HOST], &delta);
        or_into(&mut self.dirty.borrow_mut()[HOST], &delta);
        or_into(&mut self.hcache.borrow_mut().stale, &delta);
    }

    /// End the current epoch (after a save or load that made the live state snapshot `id`): stamp what was written in
    /// it, start the next, and age the hot static pages.
    fn close(&self, id: u64, snap: std::rc::Weak<Snapshot>) {
        let e = self.epoch.get();
        {
            let mut since = self.since.borrow_mut();
            let mut stamp = self.stamp.borrow_mut();
            for (ri, s) in since.iter_mut().enumerate() {
                let (st, blk) = &mut stamp[ri];
                for pi in bits(s, usize::MAX) {
                    let pi = pi as usize;
                    if st.len() <= pi {
                        st.resize(pi + 1, 0);
                        blk.resize(st.len().div_ceil(64), 0);
                    }
                    st[pi] = e;
                    blk[pi / 64] = e;
                }
                s.iter_mut().for_each(|w| *w = 0);
            }
            if self.static_watch {
                // a hot page goes read-only again after HOT_AGE intervals; a write between the refresh and here is
                // covered by marking it written in the new epoch too
                let mut hot = self.hot.borrow_mut();
                for (k, (hb, age)) in hot.iter_mut().enumerate() {
                    let mut back = Vec::new();
                    for pi in bits(hb, age.len()) {
                        let a = &mut age[pi as usize];
                        *a = a.saturating_add(1);
                        if *a >= HOT_AGE {
                            back.push(pi);
                        }
                    }
                    if !back.is_empty() {
                        unsafe { crate::pagewatch::protect(k, &back) };
                        for &pi in &back {
                            hb[pi as usize / 64] &= !(1u64 << (pi % 64));
                            set_bit(&mut since[STATIC0 + k], pi as usize);
                        }
                        self.stats.borrow_mut().static_reprotects += back.len() as u64;
                    }
                }
            }
        }
        self.epoch.set(e + 1);
        self.last.set(id);
        *self.last_snap.borrow_mut() = snap;
    }

    fn base_page(&self, ri: usize, pi: usize) -> &[u8] {
        if ri == 0 {
            if pi * PAGE >= self.arena_len {
                return &ZERO;
            }
            let at = self.ranges[0].0 + pi * PAGE;
            // never written since the capture: the live page IS the base page
            return unsafe { crate::cow::saved(at).unwrap_or_else(|| bytes(at, PAGE)) };
        }
        #[cfg(windows)]
        if ri == HOST && crate::cow::bg::active() && !crate::cow::bg::copied(pi) {
            // not copied yet, so not written since the capture (a write copies it first): the live page is the base
            let (a, l) = self.ranges[HOST];
            return if (pi + 1) * PAGE <= l { unsafe { bytes(a + pi * PAGE, PAGE) } } else { &ZERO };
        }
        let d = &self.data[ri];
        let s = pi * PAGE;
        if s + PAGE <= d.len() { &d[s..s + PAGE] } else { &ZERO }
    }

    /// The page indexes of range `ri` that can differ from the base.
    fn candidates(&self, ri: usize, len: usize) -> Vec<u32> {
        bits(&self.dirty.borrow()[ri], len / PAGE)
    }

    /// Save the current state.
    pub fn save(&self) -> SnapRef {
        self.save_after(None).0
    }

    /// [`Base::save`] right after [`Base::page_hash`] with nothing run in between (the rollback session hashes then
    /// saves each frame): the hash already read and reset the host block's write watch, so the save does not scan
    /// it again (a full-block `GetWriteWatch`, ~4% of a rolling-back peer's game thread). Only for that sequence.
    pub fn save_after_hash(&self) -> SnapRef {
        self.skip_host.set(true);
        let s = self.save();
        self.skip_host.set(false);
        s
    }

    /// Save the current state, sharing every page that is unchanged since `prev` (default: the snapshot the live
    /// state was last saved as or loaded from, while it is alive). When `prev` is that snapshot only the pages written
    /// since are read; otherwise every dirty page is compared. Returns the bytes newly copied.
    pub fn save_after(&self, prev: Option<&Snapshot>) -> (SnapRef, usize) {
        let _bg = crate::arena::BgPause::enter();
        let mut fresh = 0usize;
        self.refresh();
        let auto = if prev.is_none() && !self.no_inc { self.last_snap.borrow().upgrade() } else { None };
        let prev = prev.or(auto.as_deref());
        // incremental when `prev` is the state the live one evolved from: a page not written since then is `prev`'s
        // page (or the base's) without comparing a byte
        let inc = prev.is_some_and(|p| p.id == self.last.get() && self.last.get() != 0) && !self.no_inc;
        let blind = inc && auto.is_some();
        let since = self.since.borrow();
        let cur = self.cur_ranges();
        assert_eq!(cur.len(), self.ranges.len(), "snapshot ranges changed");
        let mut out = Vec::with_capacity(cur.len());
        let mut pool = self.pool.borrow_mut();
        let mut st = self.stats.borrow_mut();
        st.saves += 1;
        if !inc {
            st.save_full += 1;
        }
        for (ri, &(addr, len)) in cur.iter().enumerate() {
            if ri > 0 {
                assert_eq!(addr, self.ranges[ri].0, "range {ri} moved");
            }
            let np = len / PAGE;
            let live = unsafe { bytes(addr, len) };
            let old = prev.and_then(|p| p.ranges.get(ri)).map_or(&[][..], |r| &r.pages[..]);
            let mut pages = Vec::with_capacity(old.len() + 16);
            let cand = if inc { bits(&since[ri], np) } else { self.candidates(ri, len) };
            st.save_cand += cand.len() as u64;
            // walk the candidates and (incremental) prev's pages together, ascending
            let (mut i, mut k) = (0usize, 0usize);
            loop {
                let c = cand.get(i).map(|&x| x as usize);
                let o = if inc { old.get(k).map(|p| p.0 as usize) } else { None };
                let pi = match (c, o) {
                    (None, None) => break,
                    (Some(c), Some(o)) => c.min(o),
                    (Some(c), None) => c,
                    (None, Some(o)) => o,
                };
                if pi >= np {
                    break;
                }
                if !inc {
                    while k < old.len() && (old[k].0 as usize) < pi {
                        k += 1;
                    }
                }
                let in_old = k < old.len() && old[k].0 as usize == pi;
                let written = c == Some(pi);
                if written {
                    i += 1;
                    let p = &live[pi * PAGE..(pi + 1) * PAGE];
                    // a page of prev differs from the base: equal to it is shared; else new when it differs from the base
                    // (a plain save after its own previous one copies a written page without comparing: the
                    // rollback's snapshots live a few frames, sharing would save memory, not time)
                    if !blind && in_old && &old[k].1[..] == p {
                        pages.push((pi as u32, old[k].1.clone()));
                    } else if blind || p != self.base_page(ri, pi) {
                        let b = match pool.pop() {
                            Some(mut b) => {
                                Rc::get_mut(&mut b).expect("pooled page is unshared").copy_from_slice(p);
                                b
                            }
                            None => Rc::new(p.try_into().unwrap()),
                        };
                        pages.push((pi as u32, b));
                        fresh += PAGE;
                        st.save_copied += 1;
                    }
                } else {
                    // not written since prev: prev's page
                    pages.push((pi as u32, old[k].1.clone()));
                    st.save_shared += 1;
                }
                if inc && in_old {
                    k += 1;
                }
            }
            out.push(RangeSnap { addr, len, pages });
        }
        drop(st);
        drop(pool);
        drop(since);
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let epoch = self.epoch.get();
        let s = Rc::new_cyclic(|me| Snapshot { ranges: out, id, epoch, me: me.clone(), pool: self.pool.clone() });
        if self.check_every > 0 && self.stats.borrow().saves % self.check_every == 0 {
            // what was written after the save read it (the save's own allocations bump a static counter) is not `s`
            self.refresh();
            let since = self.since.borrow();
            self.check(&s, "save", Some(&since[..]), self.host_mask.as_ref());
        }
        self.close(id, Rc::downgrade(&s));
        (s, fresh)
    }

    /// Put a saved state back. The caller restores the arena mode flag afterwards (it is a static too).
    pub fn load(&self, s: &Snapshot) {
        self.load_with(s, !self.no_inc)
    }

    /// Put back a snapshot this base did not save ([`Base::import`]: a state file): the full way, every page that can
    /// differ from the base and every page of `s` (its pages carry no write history the incremental way could use).
    pub fn load_full(&self, s: &Snapshot) {
        self.load_with(s, false)
    }

    fn load_with(&self, s: &Snapshot, inc: bool) {
        let _bg = crate::arena::BgPause::enter();
        // core v1: the transcoder lets go of the game buffers it holds first (the load rewinds their refcounts)
        #[cfg(feature = "corev1")]
        melee_gx::corev1::snapshot_loaded();
        // the render state is outside the snapshot: its copies of game memory are dropped (render only)
        crate::gxbridge::on_state_load();
        self.refresh();
        let was = arena::active();
        let (abase, alen_now) = arena::used_range();
        let mut zeroed: Option<(usize, usize)> = None;
        let mut changed: Vec<usize> = Vec::new();
        {
            let since = self.since.borrow();
            let stamp = self.stamp.borrow();
            let mut st = self.stats.borrow_mut();
            st.loads += 1;
            if !inc {
                st.load_full += 1;
            }
            // every list first: nothing allocates between the copies and the check (an allocation writes the
            // system-heap counter, a static)
            let mut cands = Vec::with_capacity(s.ranges.len());
            for (ri, r) in s.ranges.iter().enumerate() {
                let np = r.len / PAGE;
                // the pages that may differ from `s`: written since it was taken (or, the full way, the dirty ones
                // plus its own)
                let cand: Vec<u32> = if inc {
                    let (sp, blk) = &stamp[ri];
                    let mut later = Vec::new();
                    for (b, _) in blk.iter().enumerate().filter(|x| *x.1 > s.epoch) {
                        for pi in b * 64..(b * 64 + 64).min(sp.len()).min(np) {
                            if sp[pi] > s.epoch {
                                later.push(pi as u32);
                            }
                        }
                    }
                    union(&bits(&since[ri], np), later.into_iter())
                } else {
                    union(&self.candidates(ri, r.len), r.pages.iter().map(|p| p.0))
                };
                st.load_cand += cand.len() as u64;
                cands.push(cand);
            }
            // the pages this load rewrites, for the render cache (gxbridge::on_state_loaded); sized before the copies
            // and on the system heap: the arena is what the copies rewrite
            {
                let _sys = crate::arena::SysScope::enter();
                changed.reserve(cands.iter().map(|c| c.len()).sum::<usize>());
            }
            for (ri, r) in s.ranges.iter().enumerate() {
                let live = unsafe { core::slice::from_raw_parts_mut(r.addr as *mut u8, r.len) };
                let mut k = 0usize;
                for &pi in &cands[ri] {
                    while k < r.pages.len() && r.pages[k].0 < pi {
                        k += 1;
                    }
                    // host-only statics bytes stay as they are: other threads write them at any time (snapmask.rs)
                    let spans = match &self.host_mask {
                        Some(m) if ri >= STATIC0 => {
                            if m.skipped(ri, pi) {
                                continue;
                            }
                            m.spans(ri, pi)
                        }
                        _ => None,
                    };
                    let pi = pi as usize;
                    let dst = &mut live[pi * PAGE..(pi + 1) * PAGE];
                    let src: &[u8] = if k < r.pages.len() && r.pages[k].0 as usize == pi { &r.pages[k].1[..] } else { self.base_page(ri, pi) };
                    match spans {
                        None => {
                            if dst != src {
                                dst.copy_from_slice(src);
                                st.load_copied += 1;
                                changed.push(r.addr + pi * PAGE);
                            }
                        }
                        Some(sp) => {
                            let mut any = false;
                            for &(a, b) in sp {
                                let (a, b) = (a as usize, b as usize);
                                if dst[a..b] != src[a..b] {
                                    dst[a..b].copy_from_slice(&src[a..b]);
                                    any = true;
                                }
                            }
                            st.load_copied += any as u64;
                            if any {
                                changed.push(r.addr + pi * PAGE);
                            }
                        }
                    }
                }
            }
            // arena memory the discarded timeline bumped into is fresh again: zero it, as an untouched page is
            if let Some(a) = s.ranges.first() {
                if alen_now > a.len {
                    unsafe { core::ptr::write_bytes((abase + a.len) as *mut u8, 0, alen_now - a.len) };
                    // these writes lie past the arena's new used range, which the next refresh does not scan: record
                    // them as written in the coming epoch, or a later load of a snapshot that does reach that far (a
                    // newer one: the lab's lookahead resume point, a slot saved later) would skip them as unchanged
                    zeroed = Some((a.len / PAGE, alen_now.div_ceil(PAGE)));
                }
            }
            arena::set_active(was);
            drop(st);
            if self.check_every > 0 && self.stats.borrow().loads % self.check_every == 0 {
                if self.poke {
                    // MELEE_SNAP_CHECK_POKE: a game-crate static written behind the load's back; the check must catch it
                    CHECK_POKE.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                }
                self.check(s, "load", None, self.host_mask.as_ref());
            }
        }
        // the live state is `s` now (the load's own writes show up as written in the next epoch: compared once)
        self.close(s.id, s.me.clone());
        {
            // render cache: copies the load did not touch stay valid (the zeroed arena tail counts as rewritten)
            let _sys = crate::arena::SysScope::enter();
            if let Some((p0, p1)) = zeroed {
                changed.extend((p0..p1).map(|pi| abase + pi * PAGE));
            }
            changed.sort_unstable();
            crate::gxbridge::on_state_loaded(&changed, PAGE);
            drop(changed);
        }
        if let Some((p0, p1)) = zeroed {
            let mut since = self.since.borrow_mut();
            for pi in p0..p1 {
                set_bit(&mut since[0], pi);
            }
        }
    }

    /// MELEE_SNAP_CHECK: the live state must equal `s` page for page, as a full-copy restore would leave it: every page
    /// of the host block and the statics, and every arena page ever written (an arena page never written is its base
    /// page by the copy-on-write store's construction, and `s` holds only pages that differ from the base).
    fn check(&self, s: &Snapshot, what: &str, skip: Option<&[Vec<u64>]>, mask: Option<&crate::snapmask::HostMask>) {
        // allocation free until the verdict (see `load`)
        let mut bad = [(0usize, 0u32); 8];
        let mut nbad = 0usize;
        let mut n = 0u64;
        // the live bytes of the first bad pages as the compare saw them (another thread may rewrite them before the report)
        let mut seen = [[0u8; PAGE]; 2];
        let mut quick = [[(0u16, 0u8, 0u8); 8]; 2];
        let dirty = self.dirty.borrow();
        for (ri, r) in s.ranges.iter().enumerate() {
            let live = unsafe { bytes(r.addr, r.len) };
            let np = (r.len / PAGE) as u32;
            let mut one = |pi: u32| {
                let sk = mask.filter(|_| ri >= STATIC0);
                if skip.is_some_and(|sk| bit_of(&sk[ri], pi as usize)) || sk.is_some_and(|m| m.skipped(ri, pi)) {
                    return;
                }
                n += 1;
                let want = r.page(pi).unwrap_or_else(|| self.base_page(ri, pi as usize));
                let have = &live[pi as usize * PAGE..(pi as usize + 1) * PAGE];
                let differs = match sk.and_then(|m| m.spans(ri, pi)) {
                    Some(sp) => sp.iter().any(|&(a, b)| have[a as usize..b as usize] != want[a as usize..b as usize]),
                    None => have != want,
                };
                if differs {
                    if nbad < seen.len() {
                        seen[nbad].copy_from_slice(have);
                        // a value that flips back at once (a busy flag, a lock word): note its offsets straight away
                        let mut k = 0;
                        for i in 0..PAGE {
                            let h = unsafe { core::ptr::read_volatile(have.as_ptr().add(i)) };
                            if h != want[i] && k < 8 {
                                quick[nbad][k] = (i as u16, h, want[i]);
                                k += 1;
                            }
                        }
                    }
                    if nbad < bad.len() {
                        bad[nbad] = (ri, pi);
                    }
                    nbad += 1;
                }
            };
            if ri == 0 {
                for (w, &x) in dirty[0].iter().enumerate() {
                    let mut x = x;
                    while x != 0 {
                        let pi = (w * 64) as u32 + x.trailing_zeros();
                        x &= x - 1;
                        if pi < np {
                            one(pi);
                        }
                    }
                }
                for p in &r.pages {
                    one(p.0);
                }
            } else {
                for pi in 0..np {
                    one(pi);
                }
            }
        }
        drop(dirty);
        let mut st = self.stats.borrow_mut();
        st.checks += 1;
        st.checked_pages += n;
        drop(st);
        for (bi, &(ri, pi)) in bad[..nbad.min(bad.len())].iter().enumerate() {
            let r = &s.ranges[ri];
            let now = unsafe { bytes(r.addr + pi as usize * PAGE, PAGE) };
            let live: &[u8] = if bi < seen.len() { &seen[bi][..] } else { now };
            if bi < quick.len() {
                let img = { #[cfg(windows)] { extern "system" { fn GetModuleHandleW(n: *const u16) -> usize; } unsafe { GetModuleHandleW(core::ptr::null()) } } #[cfg(not(windows))] { 0usize } };
                let at = r.addr + pi as usize * PAGE - img;
                eprintln!("MELEE_SNAP_CHECK: at compare range {ri} page {pi} (rva {at:#x}): (offset, live, want) {:x?}", &quick[bi]);
            }
            let want = r.page(pi).unwrap_or_else(|| self.base_page(ri, pi as usize));
            let offs: Vec<usize> = (0..PAGE).filter(|&i| live[i] != want[i]).collect();
            if let (Some(&a), Some(&b)) = (offs.first(), offs.last()) {
                let at = r.addr + pi as usize * PAGE;
                #[cfg(windows)]
                let img = { extern "system" { fn GetModuleHandleW(n: *const u16) -> usize; } unsafe { GetModuleHandleW(core::ptr::null()) } };
                #[cfg(not(windows))]
                let img = 0usize;
                eprintln!("MELEE_SNAP_CHECK: first differing byte at rva {:#x} (image base {img:#x})", at + a - img);
                eprintln!("MELEE_SNAP_CHECK: range {ri} page {pi}: {} bytes differ in {:#x}..={:#x} (range offset {:#x}); live {:02x?} want {:02x?}",
                    offs.len(), at + a, at + b, pi as usize * PAGE + a, &live[a..=b.min(a + 31)], &want[a..=b.min(a + 31)]);
                eprintln!("MELEE_SNAP_CHECK: now {:02x?}; differing offsets {:?}", &now[a..=b.min(a + 31)], &offs[..offs.len().min(32)]);
            }
        }
        assert!(nbad == 0, "MELEE_SNAP_CHECK: after {what} of snapshot {} the live state differs from it in {nbad} pages, first (range, page) {:?}", s.id, &bad[..nbad.min(8)]);
    }

    /// The counters, one line (MELEE_SNAP_STATS).
    pub fn stats_line(&self) -> String {
        let s = *self.stats.borrow();
        let per = |x: u64, n: u64| x as f64 / n.max(1) as f64;
        format!(
            "{} saves ({} full): {:.1} pages looked at, {:.1} copied, {:.1} shared each; {} loads ({} full): {:.1} looked at, {:.1} copied each; statics {}: {} faults, {} made read-only again; {} checks ({} pages); {} hashes ({} unchanged), {:.1} pages digested each",
            s.saves, s.save_full, per(s.save_cand, s.saves), per(s.save_copied, s.saves), per(s.save_shared, s.saves),
            s.loads, s.load_full, per(s.load_cand, s.loads), per(s.load_copied, s.loads),
            if self.static_watch { "watched" } else { "scanned" }, s.static_faults, s.static_reprotects, s.checks, s.checked_pages, s.hashes, s.hashes_cached, per(s.hash_digests, s.hashes)
        )
        + &bg_faults_note()
    }

    /// A hash of the OS-heap block (the C-layout game state: fighters, items, gobjs, jobjs). The arena holds Rust
    /// bookkeeping whose bytes depend on std's per-process hasher seeds, and the statics hold counters, so neither is
    /// hashed. Only pages that differ from the base are mixed in, each through a cached per-page digest that is
    /// recomputed only when the write watch reports the page written since the previous call (a load's restores are
    /// writes too). The value is compared only within one run (rollback snapshots, synctest), never across builds.
    pub fn page_hash(&self) -> u32 {
        {
            let _z = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 1000, "state hash: refresh host");
            self.refresh_host();
        }
        let (addr, len) = self.ranges[HASHED];
        let live = unsafe { bytes(addr, len) };
        let mut hc = self.hcache.borrow_mut();
        let hc = &mut *hc;
        self.stats.borrow_mut().hashes += 1;
        if let Some(h) = hc.last {
            if hc.stale.iter().all(|&w| w == 0) && !self.hash_verify {
                self.stats.borrow_mut().hashes_cached += 1;
                return h;
            }
        }
        let npages = len / PAGE;
        if hc.digest.len() < npages {
            hc.digest.resize(npages, None);
            hc.base.resize(npages, None);
        }
        let mut h: u64 = 0xcbf29ce484222325;
        let cands = {
            let _z = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 1000, "state hash: candidates");
            self.candidates(HASHED, len)
        };
        let _zd = melee_core::frameprof::SlowSpan::new(melee_core::frameprof::Kind::Note, 1000, "state hash: digests");
        // Many pages to digest at once (the first hash of a match: every page, ~50 ms on one thread at a netplay
        // match start): digest them on a few threads first. Digests are pure functions of the bytes, so the result
        // is the same; the fold below then finds them known.
        let todo: Vec<usize> = cands.iter().map(|&p| p as usize).filter(|&pi| pi < npages && (bit_of(&hc.stale, pi) || !bit_of(&hc.known, pi))).collect();
        if todo.len() >= PAR_DIGEST_MIN && !self.hash_verify {
            let _s = crate::arena::SysScope::enter();
            let per = todo.len().div_ceil(PAR_DIGEST_THREADS);
            // (page, its base bytes, the base digest if known): plain data the threads can share
            let items: Vec<(usize, &[u8], Option<u64>)> = todo.iter().map(|&pi| (pi, self.base_page(HASHED, pi), hc.base[pi])).collect();
            let res: Vec<Vec<(usize, Option<u64>, u64)>> = std::thread::scope(|sc| {
                let hs: Vec<_> = items
                    .chunks(per)
                    .map(|ch| {
                        sc.spawn(move || {
                            ch.iter()
                                .map(|&(pi, bp, bd)| {
                                    let d = page_digest(pi, &live[pi * PAGE..(pi + 1) * PAGE]);
                                    let b = bd.unwrap_or_else(|| page_digest(pi, bp));
                                    (pi, (d != b).then_some(d), b)
                                })
                                .collect()
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().expect("page digest thread")).collect()
            });
            let mut st = self.stats.borrow_mut();
            for (pi, f, b) in res.into_iter().flatten() {
                hc.base[pi] = Some(b);
                hc.digest[pi] = f;
                set_bit(&mut hc.known, pi);
                clear_bit(&mut hc.stale, pi);
                st.hash_digests += 1;
            }
        }
        for pi in cands {
            let pi = pi as usize;
            let mut fresh = |pi: usize| {
                let d = page_digest(pi, &live[pi * PAGE..(pi + 1) * PAGE]);
                let b = *hc.base[pi].get_or_insert_with(|| page_digest(pi, self.base_page(HASHED, pi)));
                (d != b).then_some(d)
            };
            if bit_of(&hc.stale, pi) || !bit_of(&hc.known, pi) {
                let f = fresh(pi);
                self.stats.borrow_mut().hash_digests += 1;
                hc.digest[pi] = f;
                set_bit(&mut hc.known, pi);
            } else if self.hash_verify {
                let f = fresh(pi);
                assert_eq!(hc.digest[pi], f, "page_hash: cached digest of page {pi} is stale");
            }
            if let Some(d) = hc.digest[pi] {
                h = ((h ^ (HASHED as u64) << 32 ^ pi as u64).wrapping_mul(0x100000001b3) ^ d).wrapping_mul(0x100000001b3);
            }
        }
        hc.stale.iter_mut().for_each(|w| *w = 0);
        let h = (h ^ (h >> 32)) as u32;
        hc.last = Some(h);
        h
    }
}

/// One range of a snapshot as whole pages ([`Base::export`] / [`Base::import`]): its address, its length, and every
/// page that is not all zero.
pub struct RangeImage {
    pub addr: usize,
    pub len: usize,
    pub pages: Vec<(u32, Vec<u8>)>,
}

impl Base {
    /// The ranges a snapshot covers now: (address, length); the arena's length is its used range.
    pub fn layout(&self) -> Vec<(usize, usize)> {
        self.cur_ranges()
    }

    /// Bytes of the whole state (every range): what one full copy of it costs (gw_snap_slot_bytes' MEM1 + globals).
    pub fn state_bytes(&self) -> usize {
        self.cur_ranges().iter().map(|r| r.1).sum()
    }

    /// The counters (MELEE_SNAP_STATS).
    pub fn stats(&self) -> Stats {
        *self.stats.borrow()
    }

    /// Page `pi` of range `ri` as snapshot `s` holds it (its own page, else the base's, zero past either).
    fn snap_page<'a>(&'a self, s: &'a Snapshot, ri: usize, pi: usize) -> &'a [u8] {
        let r = &s.ranges[ri];
        match r.pages.binary_search_by_key(&(pi as u32), |p| p.0) {
            Ok(i) => &r.pages[i].1[..],
            Err(_) if pi * PAGE < r.len => self.base_page(ri, pi),
            Err(_) => &ZERO,
        }
    }

    /// Snapshot `s` as whole pages: every non-zero page of every range (a state file's contents).
    pub fn export(&self, s: &Snapshot) -> Vec<RangeImage> {
        (0..s.ranges.len())
            .map(|ri| {
                let r = &s.ranges[ri];
                let pages = (0..r.len / PAGE)
                    .filter_map(|pi| {
                        let p = self.snap_page(s, ri, pi);
                        (p != &ZERO[..]).then(|| (pi as u32, p.to_vec()))
                    })
                    .collect();
                RangeImage { addr: r.addr, len: r.len, pages }
            })
            .collect()
    }

    /// A snapshot holding the state [`Base::export`] wrote (a page not listed is zero), for [`Base::load_full`]. None when
    /// the ranges are not this base's: another count, a range at another address, a fixed range of another length.
    pub fn import(&self, img: &[RangeImage]) -> Option<SnapRef> {
        let cur = self.cur_ranges();
        if img.len() != cur.len() {
            return None;
        }
        for (ri, (r, c)) in img.iter().zip(&cur).enumerate() {
            // the arena's used length moves (a load handles that); every other range is fixed
            if r.addr != c.0 || (ri > 0 && r.len != c.1) || r.len % PAGE != 0 || r.pages.iter().any(|p| p.1.len() != PAGE || p.0 as usize >= r.len / PAGE) {
                return None;
            }
        }
        let mut out = Vec::with_capacity(img.len());
        for (ri, r) in img.iter().enumerate() {
            let mut pages = Vec::new();
            let mut k = 0usize;
            for pi in 0..r.len / PAGE {
                while k < r.pages.len() && (r.pages[k].0 as usize) < pi {
                    k += 1;
                }
                let want: &[u8] = if k < r.pages.len() && r.pages[k].0 as usize == pi { &r.pages[k].1 } else { &ZERO };
                if want != self.base_page(ri, pi) {
                    pages.push((pi as u32, Rc::new(<[u8; PAGE]>::try_from(want).unwrap())));
                }
            }
            out.push(RangeSnap { addr: r.addr, len: r.len, pages });
        }
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        // epoch 0: older than every write, so nothing is assumed unchanged since it
        Some(Rc::new_cyclic(|me| Snapshot { ranges: out, id, epoch: 0, me: me.clone(), pool: self.pool.clone() }))
    }

    /// The rewind self-test's compare of two snapshots: (bytes that differ in any range, bytes that differ in the
    /// C-layout game state (the OS heap, the range `page_hash` covers: the arena and the statics hold Rust
    /// bookkeeping), a 64-bit FNV-1a hash of each side's game-state range).
    pub fn compare(&self, a: &Snapshot, b: &Snapshot) -> (u64, u64, u64, u64) {
        let (mut all, mut sim) = (0u64, 0u64);
        let (mut ha, mut hb) = (0xcbf29ce484222325u64, 0xcbf29ce484222325u64);
        for ri in 0..a.ranges.len().min(b.ranges.len()) {
            let n = a.ranges[ri].len.max(b.ranges[ri].len) / PAGE;
            for pi in 0..n {
                let (pa, pb) = (self.snap_page(a, ri, pi), self.snap_page(b, ri, pi));
                if ri == HASHED {
                    for i in 0..PAGE {
                        ha = (ha ^ pa[i] as u64).wrapping_mul(0x100000001b3);
                        hb = (hb ^ pb[i] as u64).wrapping_mul(0x100000001b3);
                    }
                }
                if pa != pb {
                    let d = (0..PAGE).filter(|&i| pa[i] != pb[i]).count() as u64;
                    all += d;
                    if ri == HASHED {
                        sim += d;
                    }
                }
            }
        }
        (all, sim, ha, hb)
    }

    /// Debug: describe where two snapshots differ ("range R page P: first byte offsets ...").
    pub fn diff(&self, a: &Snapshot, b: &Snapshot, max: usize) -> Vec<String> {
        let mut out = Vec::new();
        for (ri, (ra, rb)) in a.ranges.iter().zip(b.ranges.iter()).enumerate() {
            let n = ra.len.max(rb.len) / PAGE;
            let page = |r: &RangeSnap, pi: usize| -> Vec<u8> {
                match r.pages.binary_search_by_key(&(pi as u32), |p| p.0) {
                    Ok(i) => r.pages[i].1.to_vec(),
                    Err(_) => if pi * PAGE < r.len { self.base_page(ri, pi).to_vec() } else { vec![0; PAGE] },
                }
            };
            for pi in 0..n {
                let (pa, pb) = (page(ra, pi), page(rb, pi));
                if pa != pb {
                    let offs: Vec<usize> = (0..PAGE).filter(|&i| pa[i] != pb[i]).take(6).collect();
                    out.push(format!("range {ri} addr {:#x} page {pi}: {} bytes differ, first at {:?}", ra.addr + pi * PAGE, (0..PAGE).filter(|&i| pa[i] != pb[i]).count(), offs));
                    if let Some(&o) = offs.first() {
                        let o8 = o & !7;
                        out.push(format!("    a {:02x?}
    b {:02x?}", &pa[o8..o8 + 32.min(PAGE - o8)], &pb[o8..o8 + 32.min(PAGE - o8)]));
                    }
                    if out.len() >= max { return out; }
                }
            }
        }
        out
    }
}
