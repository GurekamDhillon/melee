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
#[cfg(not(windows))]
fn written_pages(addr: usize, len: usize, base: usize, out: &mut Vec<u64>) {
    for p in (addr - base) / PAGE..(addr - base + len) / PAGE {
        set_bit(out, p);
    }
}

/// The arena's copy-on-write part is write-watched by the OS too (it is reserved MEM_WRITE_WATCH), but `refresh` never
/// reads that part (the cow bits say which pages were ever written). The incremental ring save reads (and resets) the
/// watch over just the runs of pages ever written, to learn which of them were written since the previous snapshot.
fn written_runs(cum: &[u64], limit: usize, addr: usize, out: &mut Vec<u64>) {
    let bit = |i: usize| cum.get(i / 64).copied().unwrap_or(0) >> (i % 64) & 1 != 0;
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
        while i < limit && bit(i) {
            i += 1;
        }
        written_pages(addr + s * PAGE, (i - s) * PAGE, addr, out);
    }
}

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

const WATCHED: usize = 2;

/// The reference every snapshot is a delta of (the state when the first snapshot was taken).
pub struct Base {
    ranges: Vec<(usize, usize)>,
    /// Full copies of every range but the arena (range 0), whose base is kept copy-on-first-write (`cow`).
    data: Vec<Vec<u8>>,
    /// Bytes of the arena that existed at capture (pages beyond were zero then).
    arena_len: usize,
    /// Watched ranges: every page written since the base (cumulative).
    dirty: std::cell::RefCell<Vec<Vec<u64>>>,
    /// Watched ranges: every page written since the snapshot `last` was saved or loaded (the incremental ring save).
    since: std::cell::RefCell<Vec<Vec<u64>>>,
    /// Id of the snapshot the live state was last saved as or loaded from (0: unknown).
    last: std::cell::Cell<u64>,
    /// Set by the first `save_after` with a previous snapshot (the Lab's ring): only then do saves and loads pay for
    /// the arena's since-last watch (the rollback's per-frame saves never do).
    track: std::cell::Cell<bool>,
    /// Page buffers of dropped snapshots, reused by the next saves (no allocation per page in steady state).
    pool: Pool,
    /// The next snapshot id. Here, not a static: a load puts the statics back, and a static counter would hand out
    /// an id again (the rollback statics rule).
    next_id: std::cell::Cell<u64>,
    /// MELEE_SNAP_NOINC: compare every candidate page (the measurement baseline).
    no_inc: bool,
}

type Pool = Rc<std::cell::RefCell<Vec<Rc<[u8; PAGE]>>>>;
/// Most page buffers kept for reuse (64 MB, the Lab ring's churn; more are freed).
const POOL_MAX: usize = 16384;

#[derive(Clone)]
struct RangeSnap {
    addr: usize,
    len: usize,
    /// Shared: a snapshot taken with [`Base::save_after`] reuses the previous snapshot's unchanged pages.
    pages: Vec<(u32, Rc<[u8; PAGE]>)>,
}

/// One saved state.
#[derive(Clone)]
pub struct Snapshot {
    ranges: Vec<RangeSnap>,
    id: u64,
    pool: Pool,
}

impl Drop for Snapshot {
    /// Pages no other snapshot shares go back to the pool.
    fn drop(&mut self) {
        let Ok(mut pool) = self.pool.try_borrow_mut() else { return };
        for r in &mut self.ranges {
            for (_, p) in r.pages.drain(..) {
                if pool.len() < POOL_MAX && Rc::strong_count(&p) == 1 && Rc::weak_count(&p) == 0 {
                    pool.push(p);
                }
            }
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
        unsafe { crate::cow::release() };
    }
}

impl Base {
    /// Capture the current state as the base (the system heap holds the copy).
    pub fn capture() -> Base {
        crate::arena::note_snapshot();
        let ranges = ranges();
        let data: Vec<Vec<u8>> = ranges.iter().enumerate().map(|(i, &(a, l))| if i == 0 { Vec::new() } else { unsafe { bytes(a, l) }.to_vec() }).collect();
        let arena_len = ranges[0].1;
        let b = Base {
            ranges, data, arena_len, dirty: std::cell::RefCell::new(vec![Default::default(); WATCHED]),
            since: std::cell::RefCell::new(vec![Default::default(); WATCHED]), last: std::cell::Cell::new(0),
            track: std::cell::Cell::new(false),
            // reserved now: a push never reallocates, wherever a snapshot is dropped
            pool: Rc::new(std::cell::RefCell::new(Vec::with_capacity(POOL_MAX))), next_id: std::cell::Cell::new(1),
            no_inc: std::env::var_os("MELEE_SNAP_NOINC").is_some(),
        };
        b.refresh(); // consumes the watch state up to now (the copy above only read)
        b.dirty.borrow_mut().iter_mut().for_each(|d| d.iter_mut().for_each(|w| *w = 0));
        // the arena's base is not copied: the first write to each page saves it (cow.rs)
        unsafe { crate::cow::protect(b.ranges[0].0, arena_len).expect("arena copy-on-write") };
        b
    }

    /// Pull the pages written since the last call into the dirty sets. The arena part that existed at the capture is
    /// copy-on-first-write (`cow`): a page ever written there has its bit in the cow store, so the (expensive, it walks
    /// ~5 GB) write watch only covers what the arena grew by since; the host block is watched in full.
    fn refresh(&self) {
        let cur = ranges();
        let mut d = self.dirty.borrow_mut();
        let mut since = self.since.borrow_mut();
        let (a0, l0) = cur[0];
        let cow_len = self.arena_len.min(l0);
        crate::cow::or_written(&mut d[0]);
        // what the watch reports was written since the previous refresh: into the since-last sets, then the cumulative
        if l0 > cow_len {
            written_pages(a0 + cow_len, l0 - cow_len, a0, &mut since[0]);
        }
        for ri in 1..WATCHED {
            written_pages(cur[ri].0, cur[ri].1.max(PAGE), cur[ri].0, &mut since[ri]);
        }
        for ri in 0..WATCHED {
            let (s, dd) = (&since[ri], &mut d[ri]);
            if dd.len() < s.len() {
                dd.resize(s.len(), 0);
            }
            for (a, b) in dd.iter_mut().zip(s.iter()) {
                *a |= *b;
            }
        }
    }

    /// The cow part of the arena: pages written since the last call, into `since` (the incremental save and the
    /// loads need it once tracking is on; the rollback's per-frame refresh never pays for it).
    fn refresh_cow_since(&self) {
        let (a0, l0) = arena::used_range();
        let cow_len = self.arena_len.min(l0);
        let d = self.dirty.borrow();
        let mut since = self.since.borrow_mut();
        written_runs(&d[0], cow_len / PAGE, a0, &mut since[0]);
    }

    /// The live state is snapshot `id` now: forget what was written so far.
    fn mark_last(&self, id: u64) {
        if self.track.get() {
            self.refresh_cow_since();
            self.last.set(id);
        } else {
            self.last.set(0);
        }
        self.since.borrow_mut().iter_mut().for_each(|v| v.iter_mut().for_each(|w| *w = 0));
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
        let d = &self.data[ri];
        let s = pi * PAGE;
        if s + PAGE <= d.len() { &d[s..s + PAGE] } else { &ZERO }
    }

    /// The page indexes of range `ri` that can differ from the base.
    fn candidates(&self, ri: usize, len: usize) -> Vec<u32> {
        if ri < WATCHED {
            bits(&self.dirty.borrow()[ri], len / PAGE)
        } else {
            (0..(len / PAGE) as u32).collect()
        }
    }

    /// Save the current state.
    pub fn save(&self) -> SnapRef {
        self.save_after(None).0
    }

    /// Save the current state, sharing every page that is unchanged since `prev` (the Lab's sparse rewind ring: a
    /// snapshot every few frames then costs only the pages written in between). Returns the bytes newly copied.
    pub fn save_after(&self, prev: Option<&Snapshot>) -> (SnapRef, usize) {
        let mut fresh = 0usize;
        self.refresh();
        if prev.is_some() && !self.track.get() {
            self.track.set(true);
        }
        // incremental when `prev` is the state the live one evolved from: a watched page not written since then is
        // `prev`'s page (or the base's) without comparing a byte
        let inc = prev.is_some_and(|p| p.id == self.last.get() && self.last.get() != 0) && !self.no_inc;
        if inc {
            self.refresh_cow_since();
        }
        let since = self.since.borrow();
        let cur = ranges();
        assert_eq!(cur.len(), self.ranges.len(), "snapshot ranges changed");
        let mut out = Vec::with_capacity(cur.len());
        let mut pool = self.pool.borrow_mut();
        for (ri, &(addr, len)) in cur.iter().enumerate() {
            if ri > 0 {
                assert_eq!(addr, self.ranges[ri].0, "range {ri} moved");
            }
            let live = unsafe { bytes(addr, len) };
            let old = prev.and_then(|p| p.ranges.get(ri)).map_or(&[][..], |r| &r.pages[..]);
            let mut pages = Vec::with_capacity(old.len() + 16);
            let sb = if inc && ri < WATCHED { Some(&since[ri][..]) } else { None };
            let mut k = 0usize;
            for pi in self.candidates(ri, len) {
                let pi = pi as usize;
                while k < old.len() && (old[k].0 as usize) < pi {
                    k += 1;
                }
                let in_old = k < old.len() && old[k].0 as usize == pi;
                if let Some(sb) = sb {
                    if sb.get(pi / 64).map_or(true, |w| w >> (pi % 64) & 1 == 0) {
                        // not written since prev: prev's page, or (absent there) still the base's
                        if in_old {
                            pages.push((pi as u32, old[k].1.clone()));
                        }
                        continue;
                    }
                }
                let p = &live[pi * PAGE..(pi + 1) * PAGE];
                // a page of prev differs from the base: equal to it is shared; else new when it differs from the base
                if in_old && &old[k].1[..] == p {
                    pages.push((pi as u32, old[k].1.clone()));
                } else if p != self.base_page(ri, pi) {
                    let b = match pool.pop() {
                        Some(mut b) => {
                            Rc::get_mut(&mut b).expect("pooled page is unshared").copy_from_slice(p);
                            b
                        }
                        None => Rc::new(p.try_into().unwrap()),
                    };
                    pages.push((pi as u32, b));
                    fresh += PAGE;
                }
            }
            out.push(RangeSnap { addr, len, pages });
        }
        drop(pool);
        drop(since);
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        self.mark_last(id);
        (Rc::new(Snapshot { ranges: out, id, pool: self.pool.clone() }), fresh)
    }

    /// Put a saved state back. The caller restores the arena mode flag afterwards (it is a static too).
    pub fn load(&self, s: &Snapshot) {
        self.refresh();
        let was = arena::active();
        let (abase, alen_now) = arena::used_range();
        for (ri, r) in s.ranges.iter().enumerate() {
            let live = unsafe { core::slice::from_raw_parts_mut(r.addr as *mut u8, r.len) };
            // pages that may differ now: the dirty ones plus the snapshot's own (both ascending)
            let cand = self.candidates(ri, r.len);
            let (mut i, mut k) = (0, 0);
            while i < cand.len() || k < r.pages.len() {
                let pi = match (cand.get(i), r.pages.get(k)) {
                    (Some(&c), Some(p)) => c.min(p.0),
                    (Some(&c), None) => c,
                    (None, Some(p)) => p.0,
                    (None, None) => unreachable!(),
                };
                if cand.get(i) == Some(&pi) {
                    i += 1;
                }
                let own = if r.pages.get(k).map(|p| p.0) == Some(pi) { k += 1; Some(k - 1) } else { None };
                let pi = pi as usize;
                let dst = &mut live[pi * PAGE..(pi + 1) * PAGE];
                let src: &[u8] = match own {
                    Some(n) => &r.pages[n].1[..],
                    None => self.base_page(ri, pi),
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
        // the live state is `s` now (the load's own writes show up as written later: compared once, conservatively)
        self.mark_last(s.id);
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
                    // four independent multiply lanes: the former single chain was latency bound (a page is 512 words)
                    let mut l = [h, h ^ 0x9E37_79B9_7F4A_7C15, h ^ 0xC2B2_AE3D_27D4_EB4F, h ^ 0x1656_67B1_9E37_79F9];
                    for q in p.chunks_exact(32) {
                        for k in 0..4 {
                            let v = u64::from_le_bytes(q[k * 8..k * 8 + 8].try_into().unwrap());
                            l[k] = (l[k] ^ v).wrapping_mul(0x100000001b3).rotate_left(23);
                        }
                    }
                    h = (l[0] ^ l[1].rotate_left(13) ^ l[2].rotate_left(29) ^ l[3].rotate_left(43)).wrapping_mul(0x100000001b3);
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
