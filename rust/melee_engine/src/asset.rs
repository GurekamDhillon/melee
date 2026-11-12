//! Background asset service, slice 1 (notes/asset-service-design.md): archive decodes prepared ahead on a worker as a
//! relocatable decode plan, applied on the game thread where the synchronous load happens.
//!
//! A *plan* is what [`prepare`] records while `archive::decode` runs on a copy of the file in a recording-heap slot
//! (recheap.rs): every block reachable from the `Decoded` result, in allocation order, with its bytes, plus a fixup
//! for every word that points at another block or into the raw file image. [`apply`] allocates the same blocks in the
//! same order from the game arena, copies them and patches the fixups (raw image words point at the C heap block the
//! game read the file into). Every load goes through the same apply, whether its plan was prepared ahead on the worker
//! or just now on the game thread, so the arena sees identical allocations with the service on or off: the timing of
//! the worker is never visible to the game. The plan stays cached (content-addressed: same length + same bytes), so a
//! file loaded again (commons, the stage and fighters of a rematch) is an apply only.
//!
//! Files under [`MIN_BYTES`] (the per-action animation archives, small menus) keep the old in-place decode in both
//! modes: a plan buys nothing there.
//!
//! `MELEE_ASSETS=off` turns the worker and the hints off (every load is prepared on the spot: the byte-identity test's
//! reference mode); `MELEE_ASSET_LOG=1` logs hints, hits and misses.

use crate::archive::{self, Decoded};
use crate::recheap::{self, Rec};
use core::alloc::Layout;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub const MIN_BYTES: usize = 64 << 10;
/// Plans kept (each holds a slot); the least recently used goes first.
const KEEP: usize = recheap::NSLOT - 6;
/// Raw image + block bytes kept over all plans.
const KEEP_BYTES: usize = 1536 << 20;

#[derive(Clone, Copy)]
enum Tgt {
    Blk(u32, u32),
    Raw(u32),
    Null,
}
struct Fix {
    blk: u32,
    off: u32,
    tgt: Tgt,
}

/// A prepared decode (see the module doc).
pub struct Plan {
    slot: usize,
    raw: usize,
    len: usize,
    /// `FTDATA_LOADING_KIND` it ran with, and the fighter kind `ftdata` resolved (None: no fighter data inside).
    loading_in: i32,
    resolved: Option<i32>,
    blocks: Vec<(usize, usize, usize)>,
    fixes: Vec<Fix>,
    out: u32,
    /// Live blocks the result does not reach (a lazily built global?): the slot is never given back.
    escaped: usize,
    /// Pointer words to dead blocks (written as null).
    pub dead_refs: usize,
    pub bytes: usize,
    pub prep_ms: f64,
}

impl Drop for Plan {
    fn drop(&mut self) {
        if self.escaped == 0 {
            recheap::release_slot(self.slot);
        }
    }
}

impl Plan {
    fn file(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.raw as *const u8, self.len) }
    }
    fn fits(&self, loading: i32) -> bool {
        match self.resolved {
            None => true,
            Some(k) => self.loading_in == loading || (self.loading_in == -1 && loading == k),
        }
    }
    /// Where `o` differs from this plan (diagnostics for the byte-identity test).
    pub fn first_difference(&self, o: &Plan) -> String {
        if self.blocks.len() != o.blocks.len() {
            return format!("block count {} vs {}", self.blocks.len(), o.blocks.len());
        }
        let fa: std::collections::HashMap<(u32, u32), u8> = self.fixes.iter().map(|f| ((f.blk, f.off), 0)).collect();
        let fb: std::collections::HashMap<(u32, u32), u8> = o.fixes.iter().map(|f| ((f.blk, f.off), 0)).collect();
        let mut n = 0;
        let mut first = String::new();
        let out = unsafe { &*(self.blocks[self.out as usize].0 as *const Decoded) };
        let mut tys: std::collections::BTreeMap<String, usize> = Default::default();
        for (k, (a, b)) in self.blocks.iter().zip(&o.blocks).enumerate() {
            if a.1 != b.1 || a.2 != b.2 {
                return format!("block {k}: size/align {}/{} vs {}/{}", a.1, a.2, b.1, b.2);
            }
            let x = unsafe { core::slice::from_raw_parts(a.0 as *const u8, a.1) };
            let y = unsafe { core::slice::from_raw_parts(b.0 as *const u8, b.1) };
            let mut off = 0;
            while off < a.1 {
                let w = off & !7;
                let fixed = fa.contains_key(&(k as u32, w as u32)) || fb.contains_key(&(k as u32, w as u32));
                if !fixed && x[off] != y[off] {
                    n += 1;
                    *tys.entry(format!("{}+{:#x}", out.arena.type_at(a.0).unwrap_or("?"), off % 0x100)).or_default() += 1;
                    if first.is_empty() {
                        let lo = off.saturating_sub(8);
                        let hi = (off + 8).min(a.1);
                        let ty = out.arena.type_at(a.0).unwrap_or("(not a decoded object)");
                        first = format!("first at block {k} ({ty}, size {} align {}) +{off:#x}: {:02x?} vs {:02x?}", a.1, a.2, &x[lo..hi], &y[lo..hi]);
                    }
                }
                off += 1;
            }
        }
        format!("{n} byte(s) differ; {first}; by type+offset {tys:?}; fixups {} vs {}", self.fixes.len(), o.fixes.len())
    }

    /// A digest of the plan (block sizes, alignments, bytes with pointers as their targets): equal digests mean equal
    /// applies. The byte-identity test compares worker and game-thread plans by it.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |x: u64| {
            h ^= x;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        let mut fx = std::collections::HashMap::new();
        for f in &self.fixes {
            fx.insert((f.blk, f.off), f.tgt);
        }
        for (k, &(at, size, align)) in self.blocks.iter().enumerate() {
            mix(size as u64);
            mix(align as u64);
            let b = unsafe { core::slice::from_raw_parts(at as *const u8, size) };
            let mut off = 0;
            while off < size {
                if off + 8 <= size {
                    if let Some(t) = fx.get(&(k as u32, off as u32)) {
                        match *t {
                            Tgt::Blk(j, d) => { mix(1); mix(j as u64); mix(d as u64) }
                            Tgt::Raw(d) => { mix(2); mix(d as u64) }
                            Tgt::Null => mix(3),
                        }
                        off += 8;
                        continue;
                    }
                }
                mix(b[off] as u64);
                off += 1;
            }
        }
        h
    }
}

struct RecGuard;
impl Drop for RecGuard {
    fn drop(&mut self) {
        recheap::end();
    }
}

/// Record a decode of `file` (any thread). None: the recording heap is not available (no slot, or the process
/// allocator does not route to it).
pub fn prepare(file: &[u8], loading: i32) -> Option<Plan> {
    let t0 = std::time::Instant::now();
    let _sys = crate::arena::SysScope::enter();
    let slot = recheap::take_slot()?;
    let lo = recheap::slot_base(slot);
    let raw = lo + 0x1000;
    let len = file.len();
    if !recheap::commit(raw, len) {
        recheap::release_slot(slot);
        return None;
    }
    unsafe { core::ptr::copy_nonoverlapping(file.as_ptr(), raw as *mut u8, len) };
    let mut rec = Box::new(Rec::new(slot, raw + len + 0x1000));
    let outp;
    let resolved;
    {
        unsafe { recheap::begin(&mut *rec) };
        let _g = RecGuard;
        let probe = Box::new(0u64);
        let ok = recheap::in_range(&*probe as *const u64 as *const u8);
        drop(probe);
        if !ok {
            drop(_g);
            recheap::release_slot(slot);
            return None;
        }
        crate::dat_custom::begin_prepare(loading);
        let mut d = unsafe { archive::decode(raw as *mut u8, len) };
        let (eff, r) = crate::dat_custom::end_prepare();
        d.effects = eff;
        resolved = r;
        outp = Box::into_raw(Box::new(d)) as usize;
    }
    // reachability from the result, and the fixups
    let bl = rec.blocks();
    let root = rec.find(outp).filter(|&i| bl[i].at == outp).expect("asset: the result is not a recorded block");
    let mut seen = vec![false; bl.len()];
    let mut stack = vec![root];
    seen[root] = true;
    let mut raw_fix: Vec<(usize, usize, Tgt)> = Vec::new(); // (block, off, target with ORIGINAL block numbers)
    let mut dead_refs = 0;
    while let Some(i) = stack.pop() {
        let b = bl[i];
        let mut off = 0;
        while off + 8 <= b.size {
            let w = unsafe { ((b.at + off) as *const usize).read_unaligned() };
            if w >= raw && w <= raw + len {
                raw_fix.push((i, off, Tgt::Raw((w - raw) as u32)));
            } else if w >= rec.data_lo && w <= rec.bump {
                match rec.find(w) {
                    Some(j) if bl[j].live == 1 => {
                        raw_fix.push((i, off, Tgt::Blk(j as u32, (w - bl[j].at) as u32)));
                        if !seen[j] {
                            seen[j] = true;
                            stack.push(j);
                        }
                    }
                    _ => {
                        raw_fix.push((i, off, Tgt::Null));
                        dead_refs += 1;
                    }
                }
            } else if w >= rec.lo && w < rec.lo + recheap::SLOT {
                // a stale word that points into this slot but at nothing (uninitialised padding holding a
                // recording address): written as null so the plan does not depend on which slot it was made in
                raw_fix.push((i, off, Tgt::Null));
            }
            off += 8;
        }
    }
    let mut remap = vec![u32::MAX; bl.len()];
    let mut blocks = Vec::new();
    let mut bytes = len;
    for (i, b) in bl.iter().enumerate() {
        if seen[i] {
            remap[i] = blocks.len() as u32;
            blocks.push((b.at, b.size, b.align));
            bytes += b.size;
        }
    }
    let escaped = bl.iter().enumerate().filter(|(i, b)| b.live == 1 && !seen[*i]).count();
    let fixes = raw_fix
        .into_iter()
        .map(|(i, off, t)| Fix {
            blk: remap[i],
            off: off as u32,
            tgt: match t {
                Tgt::Blk(j, d) => Tgt::Blk(remap[j as usize], d),
                t => t,
            },
        })
        .collect();
    if escaped > 0 {
        eprintln!("asset: {escaped} live block(s) the decode result does not reach (slot {slot} kept)");
    }
    Some(Plan {
        slot,
        raw,
        len,
        loading_in: loading,
        resolved,
        blocks,
        fixes,
        out: remap[root],
        escaped,
        dead_refs,
        bytes,
        prep_ms: t0.elapsed().as_secs_f64() * 1e3,
    })
}

/// Replay `p` into the game arena (game thread, inside the game scope): the `Decoded` for the file at `src`.
///
/// # Safety
/// `src` holds the same bytes the plan was prepared from (not yet relocated).
pub unsafe fn apply(p: &Plan, src: *mut u8) -> Box<Decoded> {
    let mut newp: Vec<usize> = {
        let _s = crate::arena::SysScope::enter();
        Vec::with_capacity(p.blocks.len())
    };
    for &(at, size, align) in &p.blocks {
        let l = Layout::from_size_align_unchecked(size, align);
        // straight to the arena when the game scope is on (what the process allocator would do, minus its per-call
        // diagnostics hooks); the process allocator otherwise
        let mut q = if crate::arena::active() { crate::arena::alloc(l) } else { core::ptr::null_mut() };
        if q.is_null() {
            q = std::alloc::alloc(l);
        }
        assert!(!q.is_null(), "asset: apply out of memory ({size} bytes)");
        core::ptr::copy_nonoverlapping(at as *const u8, q, size);
        newp.push(q as usize);
    }
    for f in &p.fixes {
        let v = match f.tgt {
            Tgt::Blk(k, d) => newp[k as usize] + d as usize,
            Tgt::Raw(d) => src as usize + d as usize,
            Tgt::Null => 0,
        };
        ((newp[f.blk as usize] + f.off as usize) as *mut usize).write_unaligned(v);
    }
    let out = Box::from_raw(newp[p.out as usize] as *mut Decoded);
    {
        let _s = crate::arena::SysScope::enter();
        drop(newp);
    }
    out
}

// ---- the service: cache, worker, hints ----

enum Src {
    Disc(u64, usize),
}
struct Job {
    entry: i32,
    src: Src,
    loading: i32,
}
struct Entry {
    /// Disc entry it was hinted for (-1: prepared on the spot).
    entry: i32,
    len: usize,
    plan: Option<Arc<Plan>>,
    /// Queued or being prepared.
    pending: bool,
    used: u64,
}
#[derive(Default)]
pub struct Stats {
    pub hints: u64,
    pub hits: u64,
    pub waits: u64,
    pub inline: u64,
    pub prepared_ahead: u64,
    pub refused: u64,
    pub self_ok: u64,
    pub self_bad: u64,
}
struct Inner {
    entries: Vec<Entry>,
    jobs: VecDeque<Job>,
    clock: u64,
    stats: Stats,
}
struct Svc {
    m: Mutex<Inner>,
    cv: Condvar,
    iso: Option<String>,
    worker: bool,
}
static SVC: core::sync::atomic::AtomicPtr<Svc> = core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

fn svc() -> Option<&'static Svc> {
    let p = SVC.load(core::sync::atomic::Ordering::Acquire);
    (!p.is_null()).then(|| unsafe { &*p })
}

fn log_on() -> bool {
    static L: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *L.get_or_init(|| std::env::var_os("MELEE_ASSET_LOG").is_some())
}

/// Boot (game thread, before any snapshot): reserve the recording range, start the worker. `iso`: the disc image the
/// worker reads hinted files from.
pub fn init(iso: Option<String>) {
    if svc().is_some() {
        return;
    }
    let _s = crate::arena::SysScope::enter();
    if !recheap::init() {
        return;
    }
    let worker = std::env::var("MELEE_ASSETS").map_or(true, |v| v != "off") && iso.is_some();
    let s = Box::leak(Box::new(Svc {
        m: Mutex::new(Inner { entries: Vec::new(), jobs: VecDeque::new(), clock: 0, stats: Stats::default() }),
        cv: Condvar::new(),
        iso,
        worker,
    }));
    SVC.store(s, core::sync::atomic::Ordering::Release);
    if worker {
        std::thread::Builder::new().name("asset-worker".into()).spawn(worker_main).expect("asset worker");
    }
}

fn worker_main() {
    crate::arena::mark_foreign_thread();
    let s = svc().unwrap();
    let mut f = s.iso.as_ref().and_then(|p| std::fs::File::open(p).ok());
    loop {
        let job = {
            let mut g = s.m.lock().unwrap();
            loop {
                if let Some(j) = g.jobs.pop_front() {
                    break j;
                }
                g = s.cv.wait(g).unwrap();
            }
        };
        let Src::Disc(off, len) = job.src;
        let bytes = f.as_mut().and_then(|f| {
            use std::io::{Read, Seek, SeekFrom};
            let mut v = vec![0u8; len];
            f.seek(SeekFrom::Start(off)).ok()?;
            f.read_exact(&mut v).ok()?;
            Some(v)
        });
        let bytes = bytes.filter(|b| b.len() >= 0x20 && u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize == b.len());
        let plan = bytes.and_then(|b| std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prepare(&b, job.loading))).ok().flatten());
        let mut g = s.m.lock().unwrap();
        if let Some(e) = g.entries.iter_mut().find(|e| e.pending && e.entry == job.entry) {
            e.pending = false;
            e.plan = plan.map(Arc::new);
        }
        if g.entries.iter().any(|e| e.entry == job.entry && e.plan.is_some()) {
            g.stats.prepared_ahead += 1;
        }
        g.entries.retain(|e| e.pending || e.plan.is_some());
        drop(g);
        s.cv.notify_all();
    }
}

/// A hint: disc entry `entry` will be loaded soon (never sim-visible: the load is the same whether this ran or not).
/// `heap`: the lbHeap the game will read it into. Refused at once (counted, logged) when that heap cannot take the
/// file now, or when the service's own budget (slots, kept bytes) is spent.
pub fn hint_entry(entry: i32, heap: i32) {
    let Some(s) = svc() else { return };
    if !s.worker {
        return;
    }
    let Some((off, len)) = crate::host::Host::try_get().and_then(|h| h.disc.plain_location(entry)) else { return };
    if len < MIN_BYTES {
        return;
    }
    let largest = melee_runtime::tu::lbheap::lbHeap_LargestFree(heap) as usize;
    let _sys = crate::arena::SysScope::enter();
    let mut g = s.m.lock().unwrap();
    if g.entries.iter().any(|e| e.entry == entry) {
        return;
    }
    let held: usize = g.entries.iter().filter_map(|e| e.plan.as_ref()).map(|p| p.bytes).sum();
    let busy = g.entries.len();
    if (largest != 0 && largest < len) || busy >= KEEP || held + 8 * len > KEEP_BYTES {
        g.stats.refused += 1;
        if log_on() {
            eprintln!("asset: hint entry {entry:#x} ({} KB) refused: heap {heap} largest free {largest}, {busy} plans, {} MB held", len >> 10, held >> 20);
        }
        return;
    }
    g.stats.hints += 1;
    g.entries.push(Entry { entry, len, plan: None, pending: true, used: 0 });
    g.jobs.push_back(Job { entry, src: Src::Disc(off, len), loading: -1 });
    drop(g);
    if log_on() {
        eprintln!("asset: hint entry {entry:#x} ({} KB)", len >> 10);
    }
    s.cv.notify_all();
}

/// The load seam (`archive::host_parse`): the decode of the file at `src`, applied from a plan. None: use the old
/// in-place decode (small file, or no recording heap).
///
/// # Safety
/// `src` holds `len` bytes of an archive whose header was checked.
pub unsafe fn load(src: *mut u8, len: usize) -> Option<Box<Decoded>> {
    let s = svc()?;
    if len < MIN_BYTES {
        return None;
    }
    let loading = melee_fighter::tu::ftdata::FTDATA_LOADING_KIND;
    let file = core::slice::from_raw_parts(src as *const u8, len);
    let t0 = std::time::Instant::now();
    let _sys = crate::arena::SysScope::enter();
    let mut waited = false;
    let found: Option<Arc<Plan>> = {
        let mut g = s.m.lock().unwrap();
        loop {
            let hit = g.entries.iter().filter_map(|e| e.plan.as_ref()).find(|p| p.len == len && p.fits(loading) && p.file() == file).cloned();
            if hit.is_some() {
                break hit;
            }
            // a hinted file of this length still being prepared: wait for it (a readiness miss, counted)
            if g.entries.iter().any(|e| e.pending && e.len == len) {
                waited = true;
                g = s.cv.wait(g).unwrap();
                continue;
            }
            break None;
        }
    };
    let (plan, how) = match found {
        Some(p) => (p, if waited { "wait" } else { "hit" }),
        None => {
            let p = Arc::new(prepare(file, loading)?);
            (p, "inline")
        }
    };
    if std::env::var_os("MELEE_ASSET_SELFTEST").is_some() {
        // byte-identity check: the same file prepared on another (foreign) thread must give the same plan
        let copy: Vec<u8> = file.to_vec();
        let other = std::thread::spawn(move || {
            crate::arena::mark_foreign_thread();
            prepare(&copy, loading)
        })
        .join()
        .ok()
        .flatten();
        let ok = other.as_ref().is_some_and(|o| o.digest() == plan.digest());
        let why = match &other {
            Some(o) if !ok => plan.first_difference(o),
            _ => String::new(),
        };
        let mut g = s.m.lock().unwrap();
        if ok { g.stats.self_ok += 1 } else { g.stats.self_bad += 1 }
        eprintln!("asset selftest: {} KB {how}: {} {why} [ok {} bad {}]", len >> 10, if ok { "SAME" } else { "DIFFERENT" }, g.stats.self_ok, g.stats.self_bad);
        drop(g);
        drop(other);
    }
    let t1 = std::time::Instant::now();
    let out = {
        drop(_sys);
        let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || {
            melee_core::frameprof::Text::new().s("asset apply ").s(how).s(" ").n((len >> 10) as i64).s(" KB")
        });
        apply(&plan, src)
    };
    let _sys = crate::arena::SysScope::enter();
    let mut g = s.m.lock().unwrap();
    g.clock += 1;
    let now = g.clock;
    match how {
        "hit" => g.stats.hits += 1,
        "wait" => {
            g.stats.hits += 1;
            g.stats.waits += 1
        }
        _ => g.stats.inline += 1,
    }
    if let Some(e) = g.entries.iter_mut().find(|e| e.plan.as_ref().is_some_and(|p| Arc::ptr_eq(p, &plan))) {
        e.used = now;
    } else {
        g.entries.push(Entry { entry: -1, len, plan: Some(plan.clone()), pending: false, used: now });
    }
    // keep the cache in budget (least recently used first; pending entries stay)
    loop {
        let kept: Vec<usize> = (0..g.entries.len()).filter(|&i| g.entries[i].plan.is_some()).collect();
        let bytes: usize = kept.iter().map(|&i| g.entries[i].plan.as_ref().unwrap().bytes).sum();
        if kept.len() <= KEEP && bytes <= KEEP_BYTES {
            break;
        }
        let old = *kept.iter().min_by_key(|&&i| g.entries[i].used).unwrap();
        g.entries.remove(old);
    }
    if log_on() {
        let st = &g.stats;
        eprintln!(
            "asset: load {} KB {how}: wait/prepare {:.2} ms, apply {:.2} ms ({} blocks, {} fixups, {} dead refs; prepared in {:.2} ms) [hints {} ahead {} hits {} waits {} inline {}]",
            len >> 10,
            (t1 - t0).as_secs_f64() * 1e3,
            t1.elapsed().as_secs_f64() * 1e3,
            plan.blocks.len(),
            plan.fixes.len(),
            plan.dead_refs,
            plan.prep_ms,
            st.hints,
            st.prepared_ahead,
            st.hits,
            st.waits,
            st.inline
        );
    }
    Some(out)
}
