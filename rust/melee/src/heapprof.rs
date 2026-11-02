//! Opt-in live-heap sampler (`MELEE_HEAP_SAMPLE=<bytes between samples>`): a sampled allocation's call stack is kept
//! in a fixed table (no allocation inside the allocator) until the block is freed; [`dump`] writes what is still live,
//! grouped by call stack, with bytes scaled by the sampling rate. Windows only; a no-op elsewhere.
//! `play` dumps at each `mem after` line to `MELEE_HEAP_OUT` (default `heap.txt`), one section per call.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

const SLOTS: usize = 1 << 18;
const DEPTH: usize = 20;

#[derive(Clone, Copy)]
struct Slot {
    p: usize,
    size: usize,
    stack: [usize; DEPTH],
}

/// `SLOTS` slots, allocated by [`init`] only when sampling is on (a static table would be a 46 MB `.bss` that every
/// arena snapshot copies).
static mut TABLE: *mut Slot = core::ptr::null_mut();
unsafe fn slot(i: usize) -> *mut Slot {
    TABLE.add(i)
}
static EVERY: AtomicUsize = AtomicUsize::new(0);
static N: AtomicUsize = AtomicUsize::new(0);
static LOCK: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if let Some(n) = std::env::var("MELEE_HEAP_SAMPLE").ok().and_then(|v| v.parse::<usize>().ok()).filter(|&n| n > 0) {
        let l = std::alloc::Layout::array::<Slot>(SLOTS).unwrap();
        // SAFETY: the system heap directly (the game arena is not active yet); zeroed = every slot free.
        let t = unsafe { std::alloc::GlobalAlloc::alloc_zeroed(&std::alloc::System, l) } as *mut Slot;
        if !t.is_null() {
            unsafe { TABLE = t };
            EVERY.store(n, Relaxed);
        }
    }
}

#[inline]
pub fn enabled() -> bool {
    EVERY.load(Relaxed) != 0
}

fn lock() {
    while LOCK.compare_exchange_weak(false, true, std::sync::atomic::Ordering::Acquire, Relaxed).is_err() {
        core::hint::spin_loop();
    }
}
fn unlock() {
    LOCK.store(false, std::sync::atomic::Ordering::Release);
}
fn slot_of(p: usize) -> usize {
    (p >> 4).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - 18)
}

#[cfg(windows)]
extern "system" {
    fn RtlCaptureStackBackTrace(skip: u32, n: u32, out: *mut *mut core::ffi::c_void, hash: *mut u32) -> u16;
}

/// Note a new block (called by the process allocator).
pub fn on_alloc(p: *mut u8, size: usize) {
    let every = EVERY.load(Relaxed);
    // byte sampling: a block is kept when the running byte count crosses a multiple of `every` (always when it is
    // that big); it stands for max(size, every) bytes
    if every == 0 || p.is_null() {
        return;
    }
    let before = N.fetch_add(size, Relaxed);
    if size < every && before / every == (before + size) / every {
        return;
    }
    let size = size.max(every);
    let mut st = [0usize; DEPTH];
    #[cfg(windows)]
    unsafe {
        RtlCaptureStackBackTrace(2, DEPTH as u32, st.as_mut_ptr() as *mut _, core::ptr::null_mut());
    }
    lock();
    unsafe {
        let mut i = slot_of(p as usize);
        for _ in 0..64 {
            let s = &mut *slot(i);
            if s.p == 0 {
                *s = Slot { p: p as usize, size, stack: st };
                break;
            }
            i = (i + 1) & (SLOTS - 1);
        }
    }
    unlock();
}

/// Note a freed block.
pub fn on_free(p: *mut u8) {
    if EVERY.load(Relaxed) == 0 {
        return;
    }
    lock();
    unsafe {
        let mut i = slot_of(p as usize);
        for _ in 0..64 {
            let s = &mut *slot(i);
            if s.p == p as usize {
                s.p = 0;
                // keep the probe chain intact: re-insert the rest of the cluster
                let mut j = (i + 1) & (SLOTS - 1);
                while (*slot(j)).p != 0 {
                    let e = *slot(j);
                    (*slot(j)).p = 0;
                    let mut k = slot_of(e.p);
                    while (*slot(k)).p != 0 {
                        k = (k + 1) & (SLOTS - 1);
                    }
                    *slot(k) = e;
                    j = (j + 1) & (SLOTS - 1);
                }
                break;
            }
            if s.p == 0 {
                break;
            }
            i = (i + 1) & (SLOTS - 1);
        }
    }
    unlock();
}

/// Append the live sampled blocks, grouped by their first frames in this program, to `MELEE_HEAP_OUT`.
pub fn dump(title: &str) {
    let every = EVERY.load(Relaxed);
    if every == 0 || BUSY.swap(true, Relaxed) {
        return;
    }
    let saved = EVERY.swap(0, Relaxed); // the dump's own allocations are not sampled
    lock();
    let live: Vec<Slot> = unsafe { core::slice::from_raw_parts(TABLE, SLOTS).iter().filter(|s| s.p != 0).copied().collect() };
    unlock();
    #[cfg(windows)]
    let names = crate::prof::symbolize(live.iter().flat_map(|s| s.stack.iter().map(|&a| a as u64)).filter(|&a| a != 0));
    #[cfg(not(windows))]
    let names: std::collections::HashMap<u64, String> = Default::default();
    let skip = |n: &str| {
        n.starts_with("alloc::") || n.starts_with("core::") || n.starts_with("std::") || n.starts_with("hashbrown::") || n.starts_with("melee::heapprof") || n.starts_with("melee::impl$") || n.starts_with("melee::_::") || n.starts_with("__rust") || n.starts_with("0x")
    };
    let mut by: std::collections::HashMap<String, (usize, usize)> = Default::default();
    for s in &live {
        let fr: Vec<&str> = s.stack.iter().filter(|&&a| a != 0).map(|&a| names.get(&(a as u64)).map(String::as_str).unwrap_or("?")).filter(|n| !skip(n)).take(4).collect();
        let e = by.entry(fr.join(" < ")).or_default();
        e.0 += 1;
        e.1 += s.size;
    }
    let mut v: Vec<_> = by.into_iter().collect();
    v.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
    let total: usize = v.iter().map(|x| x.1 .1).sum();
    let mut out = format!("== {title}: ~{} KB live in sampled blocks (x{saved}) ==\n", total * saved >> 10);
    for (k, (n, b)) in v.iter().take(40) {
        out += &format!("{:9} KB {:6} blocks  {k}\n", b * saved >> 10, n * saved);
    }
    let path = std::env::var("MELEE_HEAP_OUT").unwrap_or_else(|_| "heap.txt".into());
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(out.as_bytes());
    }
    EVERY.store(saved, Relaxed);
    BUSY.store(false, Relaxed);
}
