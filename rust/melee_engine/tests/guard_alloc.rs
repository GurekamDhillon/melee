//! Debug global allocator for hunting heap corruption in the engine tests: 16-byte canaries around every block,
//! a registry of live blocks, and a sweep every `SWEEP` allocations that names the first damaged block (size,
//! allocation sequence number, the last `ct!` function that was running when it was allocated).
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

const SLOTS: usize = 1 << 22;
const PAD: usize = 32;
const SWEEP: usize = 400000;

#[derive(Clone, Copy)]
struct Ent {
    ptr: usize,
    size: usize,
    seq: usize,
    tag: usize,
}
static mut TAB: [Ent; SLOTS] = [Ent { ptr: 0, size: 0, seq: 0, tag: 0 }; SLOTS];
static SEQ: AtomicUsize = AtomicUsize::new(0);
static BUSY: AtomicUsize = AtomicUsize::new(0);
pub static TAG: AtomicUsize = AtomicUsize::new(0);

pub struct Guard;

/// Index of `ptr`'s entry, or (when absent) of the first free / tombstone slot of its 64-slot probe window
/// (usize::MAX when the window is full: the block is then simply not tracked).
unsafe fn slot(ptr: usize) -> usize {
    let h = (ptr >> 4).wrapping_mul(0x9E37_79B9) % SLOTS;
    let mut first_free = usize::MAX;
    for k in 0..64 {
        let i = (h + k) % SLOTS;
        let e = (&raw mut TAB as *mut Ent).add(i);
        if (*e).ptr == ptr {
            return i;
        }
        if ((*e).ptr == usize::MAX || (*e).ptr == 0) && first_free == usize::MAX {
            first_free = i;
        }
    }
    first_free
}

unsafe fn sweep() {
    for i in 0..SLOTS {
        let e = (&raw mut TAB as *mut Ent).add(i).read();
        if e.ptr != 0 && e.ptr != usize::MAX {
            let base = e.ptr as *const u8;
            for k in 0..PAD {
                if *base.add(k) != 0xA5 || *base.add(PAD + e.size + k) != 0xA5 {
                    let tag = e.tag as *const u8;
                    let _ = tag;
                    eprintln!("GUARD damaged block: user ptr {:#x} size {} seq {} (pad byte {k})", e.ptr + PAD, e.size, e.seq);
                    std::process::abort();
                }
            }
        }
    }
}

unsafe impl GlobalAlloc for Guard {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let align = l.align().max(16);
        let total = PAD + l.size() + PAD;
        let lay = Layout::from_size_align_unchecked((total + align - 1) / align * align, align);
        // keep the user pointer aligned: PAD is a multiple of 32
        let p = System.alloc(lay);
        if p.is_null() {
            return p;
        }
        core::ptr::write_bytes(p, 0xA5, PAD);
        core::ptr::write_bytes(p.add(PAD + l.size()), 0xA5, PAD);
        if BUSY.fetch_add(1, Relaxed) == 0 {
            let i = slot(p as usize);
            let s = SEQ.fetch_add(1, Relaxed);
            if i != usize::MAX { (&raw mut TAB as *mut Ent).add(i).write(Ent { ptr: p as usize, size: l.size(), seq: s, tag: TAG.load(Relaxed) }); }
            if s % SWEEP == SWEEP - 1 {
                sweep();
            }
        }
        BUSY.fetch_sub(1, Relaxed);
        p.add(PAD)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        let base = p.sub(PAD);
        if BUSY.fetch_add(1, Relaxed) == 0 {
            let i = slot(base as usize);
            let e = (&raw mut TAB as *mut Ent).add(if i == usize::MAX { 0 } else { i });
            if i != usize::MAX && (*e).ptr == base as usize {
                for k in 0..PAD {
                    if *base.add(k) != 0xA5 || *base.add(PAD + l.size() + k) != 0xA5 {
                        eprintln!("GUARD damaged on free: user ptr {:#x} size {} seq {}", p as usize, l.size(), (*e).seq);
                        std::process::abort();
                    }
                }
                (*e).ptr = usize::MAX; // tombstone keeps probe chains intact; skipped by sweep (never equals a live ptr)
            }
        }
        BUSY.fetch_sub(1, Relaxed);
        let align = l.align().max(16);
        let total = PAD + l.size() + PAD;
        System.dealloc(base, Layout::from_size_align_unchecked((total + align - 1) / align * align, align));
    }
}
