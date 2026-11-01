//! Engine state snapshots (step 17): the whole game state as memory ranges, saved as page deltas against a base.
//!
//! The ranges are the game arena (every Rust heap object the engine owns, `arena`), the OS-heap block of the
//! host (the C heaps: fighters, items, archives) and the executable's writable sections (all statics). A load
//! puts all of them back, so the engine continues exactly where it was; nothing here knows what the bytes mean.
//! Statics of the harness that must survive a load do not exist: the harness keeps its state in values it owns on
//! the system heap (see `arena`), and the few flags the load itself overwrites are put back by the caller.

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
#[cfg(windows)]
fn written_pages(addr: usize, len: usize, out: &mut std::collections::BTreeSet<u32>) {
    extern "system" {
        fn GetWriteWatch(flags: u32, base: *mut core::ffi::c_void, size: usize, addrs: *mut *mut core::ffi::c_void, count: *mut usize, gran: *mut u32) -> u32;
    }
    let mut buf = vec![core::ptr::null_mut::<core::ffi::c_void>(); 1 << 16];
    loop {
        let mut n = buf.len();
        let mut g = 0u32;
        let r = unsafe { GetWriteWatch(1, addr as *mut _, len, buf.as_mut_ptr(), &mut n, &mut g) };
        assert_eq!(r, 0, "GetWriteWatch failed");
        for a in &buf[..n] {
            out.insert(((*a as usize - addr) / PAGE) as u32);
        }
        if n < buf.len() {
            break;
        }
    }
}
#[cfg(not(windows))]
fn written_pages(_: usize, len: usize, out: &mut std::collections::BTreeSet<u32>) {
    out.extend(0..(len / PAGE) as u32);
}

const WATCHED: usize = 2;

/// The reference every snapshot is a delta of (the state when the first snapshot was taken).
pub struct Base {
    ranges: Vec<(usize, usize)>,
    data: Vec<Vec<u8>>,
    /// Watched ranges: every page written since the base (cumulative).
    dirty: std::cell::RefCell<Vec<std::collections::BTreeSet<u32>>>,
}

#[derive(Clone)]
struct RangeSnap {
    addr: usize,
    len: usize,
    pages: Vec<(u32, Box<[u8; PAGE]>)>,
}

/// One saved state.
#[derive(Clone)]
pub struct Snapshot {
    ranges: Vec<RangeSnap>,
}

pub type SnapRef = Rc<Snapshot>;

unsafe fn bytes(addr: usize, len: usize) -> &'static [u8] {
    core::slice::from_raw_parts(addr as *const u8, len)
}

const ZERO: [u8; PAGE] = [0; PAGE];

impl Base {
    /// Capture the current state as the base (the system heap holds the copy).
    pub fn capture() -> Base {
        let ranges = ranges();
        let data: Vec<Vec<u8>> = ranges.iter().map(|&(a, l)| unsafe { bytes(a, l) }.to_vec()).collect();
        let b = Base { ranges, data, dirty: std::cell::RefCell::new(vec![Default::default(); WATCHED]) };
        b.refresh(); // consumes the watch state up to now (the copy above only read)
        b.dirty.borrow_mut().iter_mut().for_each(|d| d.clear());
        b
    }

    /// Pull the pages written since the last call into the dirty sets.
    fn refresh(&self) {
        let cur = ranges();
        let mut d = self.dirty.borrow_mut();
        for ri in 0..WATCHED {
            written_pages(cur[ri].0, cur[ri].1.max(PAGE), &mut d[ri]);
        }
    }

    fn base_page(&self, ri: usize, pi: usize) -> &[u8] {
        let d = &self.data[ri];
        let s = pi * PAGE;
        if s + PAGE <= d.len() { &d[s..s + PAGE] } else { &ZERO }
    }

    /// The page indexes of range `ri` that can differ from the base.
    fn candidates(&self, ri: usize, len: usize) -> Vec<u32> {
        if ri < WATCHED {
            self.dirty.borrow()[ri].iter().copied().filter(|&p| (p as usize) < len / PAGE).collect()
        } else {
            (0..(len / PAGE) as u32).collect()
        }
    }

    /// Save the current state.
    pub fn save(&self) -> SnapRef {
        self.refresh();
        let cur = ranges();
        assert_eq!(cur.len(), self.ranges.len(), "snapshot ranges changed");
        let mut out = Vec::new();
        for (ri, &(addr, len)) in cur.iter().enumerate() {
            if ri > 0 {
                assert_eq!(addr, self.ranges[ri].0, "range {ri} moved");
            }
            let live = unsafe { bytes(addr, len) };
            let mut pages = Vec::new();
            for pi in self.candidates(ri, len) {
                let pi = pi as usize;
                let p = &live[pi * PAGE..(pi + 1) * PAGE];
                if p != self.base_page(ri, pi) {
                    let mut b = Box::new([0u8; PAGE]);
                    b.copy_from_slice(p);
                    pages.push((pi as u32, b));
                }
            }
            out.push(RangeSnap { addr, len, pages });
        }
        Rc::new(Snapshot { ranges: out })
    }

    /// Put a saved state back. The caller restores the arena mode flag afterwards (it is a static too).
    pub fn load(&self, s: &Snapshot) {
        self.refresh();
        let was = arena::active();
        let (abase, alen_now) = arena::used_range();
        for (ri, r) in s.ranges.iter().enumerate() {
            let live = unsafe { core::slice::from_raw_parts_mut(r.addr as *mut u8, r.len) };
            // pages that may differ now: the dirty ones plus the snapshot's own
            let mut todo: std::collections::BTreeSet<u32> = self.candidates(ri, r.len).into_iter().collect();
            todo.extend(r.pages.iter().map(|p| p.0));
            for pi in todo {
                let pi = pi as usize;
                let dst = &mut live[pi * PAGE..(pi + 1) * PAGE];
                let src: &[u8] = match r.pages.binary_search_by_key(&(pi as u32), |p| p.0) {
                    Ok(i) => &r.pages[i].1[..],
                    Err(_) => self.base_page(ri, pi),
                };
                if dst != src {
                    dst.copy_from_slice(src);
                }
            }
        }
        // arena memory the discarded timeline bumped into is fresh again: zero it, as an untouched page is
        if let Some(a) = s.ranges.first() {
            if alen_now > a.len {
                unsafe { core::ptr::write_bytes((abase + a.len) as *mut u8, 0, alen_now - a.len) };
            }
        }
        arena::set_active(was);
    }

    /// A hash of the OS-heap block (the C-layout game state: fighters, items, gobjs, jobjs). The arena holds Rust
    /// bookkeeping whose bytes depend on std's per-process hasher seeds, and the statics hold counters, so neither is hashed. Only pages that differ from the base are mixed in.
    pub fn page_hash(&self) -> u32 {
        self.refresh();
        let cur = ranges();
        let mut h: u64 = 0xcbf29ce484222325;
        for (ri, &(addr, len)) in cur.iter().enumerate().skip(1).take(1) {
            let live = unsafe { bytes(addr, len) };
            for pi in self.candidates(ri, len) {
                let pi = pi as usize;
                let p = &live[pi * PAGE..(pi + 1) * PAGE];
                if p != self.base_page(ri, pi) {
                    h = (h ^ (ri as u64) << 32 ^ pi as u64).wrapping_mul(0x100000001b3);
                    for w in p.chunks_exact(8) {
                        let v = u64::from_le_bytes(w.try_into().unwrap());
                        h = (h ^ v).wrapping_mul(0x100000001b3).rotate_left(23);
                    }
                }
            }
        }
        (h ^ (h >> 32)) as u32
    }
}

impl Base {
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
