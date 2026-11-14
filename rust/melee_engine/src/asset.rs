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
//! `MELEE_ASSET_SERVICE=off` disables the service (the in-place decode of platforms without the recording heap).
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
    /// (image + offset) - the start of the block holding the word (`dat_custom` effect texture banks).
    RawRel(u32),
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
    /// The `stage_info.grkind` the decode read (None: it did not read it). A plan fits only that stage.
    grkind: Option<i32>,
    blocks: Vec<(usize, usize, usize)>,
    fixes: Vec<Fix>,
    /// `fixes[fix_at[k]..fix_at[k + 1]]` are block k's.
    fix_at: Vec<u32>,
    /// The recorded allocator calls, in order (recheap::ev).
    events: Vec<u64>,
    /// Blocks a REALLOC made (their start holds the old block's bytes after the replay's realloc: copied whole).
    realloc_target: Vec<bool>,
    live_mask: Vec<bool>,
    /// (file offset | byte count << 28, word, pointer target) the decode wrote into the file image.
    raw_patch: Vec<(u32, u64, Option<Tgt>)>,
    /// For blocks of at least [`SPARSE_MIN`] bytes: (block, offset, length) of their non-zero 4 KB runs, in order.
    sparse: Vec<(u32, u32, u32)>,
    out: u32,
    /// Live blocks the result does not reach (a lazily built global?): the slot is never given back.
    escaped: usize,
    /// Pointer words to dead blocks (written as null).
    pub dead_refs: usize,
    pub bytes: usize,
    /// Bytes of the live blocks (what [`apply`] writes at the end).
    live_bytes: usize,
    pub prep_ms: f64,
    /// (copy-in + decode, reachability scan, plan build incl. sparse runs) ms.
    pub prep_split: [f64; 3],
}

impl Drop for Plan {
    fn drop(&mut self) {
        if self.escaped == 0 {
            recheap::release_slot(self.slot);
        }
    }
}

impl Plan {
    fn live(&self, k: usize) -> bool {
        self.live_mask[k]
    }
    fn file(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.raw as *const u8, self.len) }
    }
    fn fits(&self, loading: i32, grkind: i32) -> bool {
        if self.grkind.is_some_and(|g| g != grkind) {
            return false;
        }
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
        for &e in &self.events {
            mix(e);
        }
        for &(o, w, t) in &self.raw_patch {
            mix(o as u64);
            match t {
                Some(Tgt::Blk(j, d)) => { mix(1); mix(j as u64); mix(d as u64) }
                Some(Tgt::Raw(d)) => { mix(2); mix(d as u64) }
                _ => mix(w),
            }
        }
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
                            Tgt::RawRel(d) => { mix(4); mix(d as u64) }
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
        recheap::end_recording();
    }
}

/// Why [`prepare`] gave no plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoPlan {
    /// Transient: no recording slot free (or its memory could not be committed). Worth retrying later.
    NoSlot,
    /// The recording heap does not route this thread's allocations (never changes in a run).
    NotRouted,
    /// The decode cannot be replayed (an image-relative word outside the image, or a block held from outside the
    /// decode result). Retrying gives the same answer: the file is not prepared again.
    Unsafe,
}

/// Record a decode of `file` (any thread).
pub fn prepare(file: &[u8], loading: i32, grkind: i32) -> Result<Plan, NoPlan> {
    let t0 = std::time::Instant::now();
    let _sys = crate::arena::SysScope::enter();
    let slot = recheap::take_slot().ok_or(NoPlan::NoSlot)?;
    let lo = recheap::slot_base(slot);
    let raw = lo + 0x1000;
    let len = file.len();
    if !recheap::commit(raw, len) {
        recheap::release_slot(slot);
        return Err(NoPlan::NoSlot);
    }
    unsafe { core::ptr::copy_nonoverlapping(file.as_ptr(), raw as *mut u8, len) };
    let mut rec = Box::new(Rec::new(slot, raw + len + 0x1000));
    let outp;
    let resolved;
    let grkind_used;
    {
        unsafe { recheap::begin_recording(&mut *rec) };
        let _g = RecGuard;
        let probe = Box::new(0u64);
        let ok = recheap::in_range(&*probe as *const u64 as *const u8);
        drop(probe);
        rec.clear_events();
        if !ok {
            drop(_g);
            recheap::release_slot(slot);
            return Err(NoPlan::NotRouted);
        }
        crate::dat_custom::begin_prepare(loading, grkind);
        let _ = crate::dat_custom::take_image_relative();
        let mut d = unsafe { archive::decode(raw as *mut u8, len) };
        let (eff, r, g) = crate::dat_custom::end_prepare();
        d.effects = eff;
        resolved = r;
        grkind_used = g;
        outp = Box::into_raw(Box::new(d)) as usize;
    }
    // the fixups of every block (live or freed: a replay writes each one's last contents), then reachability
    let t_dec = std::time::Instant::now();
    if recheap::poison_on() {
        let w = recheap::poisoned_writes(&rec);
        eprintln!("asset poison: {len} bytes: {} word(s) written after their block's free{}", w.len(),
            w.iter().take(8).map(|(i, n, o, x)| format!("; block #{i} size {n} +{o:#x} {x:02x?}")).collect::<String>());
    }
    let bl = rec.blocks();
    let root = rec.find(outp).filter(|&i| bl[i].at == outp).expect("asset: the result is not a recorded block");
    // pages the decode never wrote are still zero: neither scanned nor copied (the shared tables are mostly untouched)
    let pg0 = rec.data_lo & !0xFFF;
    let wrote: Option<Vec<bool>> = recheap::written_pages(rec.data_lo, rec.bump - rec.data_lo).map(|v| {
        let mut m = vec![false; (rec.bump - pg0) / 4096 + 1];
        for p in v {
            if p >= pg0 && (p - pg0) / 4096 < m.len() {
                m[(p - pg0) / 4096] = true;
            }
        }
        m
    });
    let page_written = |a: usize| wrote.as_ref().map_or(true, |m| m.get((a - pg0) / 4096).copied().unwrap_or(true));
    let mut fixes: Vec<Fix> = Vec::new();
    let mut fix_at: Vec<u32> = Vec::with_capacity(bl.len() + 1);
    // image-relative words (differences, invisible to the scan below), by (block, offset)
    let mut rel: Vec<(u32, u32, u32)> = Vec::new();
    for (wa, ta) in crate::dat_custom::take_image_relative() {
        let Some(j) = rec.find(wa) else { return Err(NoPlan::Unsafe) };
        if ta < raw || ta > raw + len {
            return Err(NoPlan::Unsafe);
        }
        rel.push((j as u32, (wa - bl[j].at) as u32, (ta - raw) as u32));
    }
    rel.sort_unstable();
    let mut dead_refs = 0;
    for (i, b) in bl.iter().enumerate() {
        fix_at.push(fixes.len() as u32);
        let mut off = 0;
        while off + 8 <= b.size {
            if (b.at + off) & 0xFFF < 8 || off == 0 {
                if !page_written(b.at + off) {
                    off = (((b.at + off) | 0xFFF) + 1 - b.at + 7) & !7; // the next page
                    continue;
                }
            }
            let w = unsafe { ((b.at + off) as *const usize).read_unaligned() };
            let r = rel.binary_search_by(|x| (x.0, x.1).cmp(&(i as u32, off as u32)));
            let tgt = if let Ok(r) = r {
                Some(Tgt::RawRel(rel[r].2))
            } else if w >= raw && w <= raw + len {
                Some(Tgt::Raw((w - raw) as u32))
            } else if w >= rec.data_lo && w <= rec.bump {
                match rec.find(w) {
                    Some(j) => Some(Tgt::Blk(j as u32, (w - bl[j].at) as u32)),
                    None => {
                        dead_refs += 1;
                        Some(Tgt::Null)
                    }
                }
            } else if w >= rec.lo && w < rec.lo + recheap::SLOT {
                Some(Tgt::Null) // a stale recording address outside every block
            } else {
                None
            };
            if let Some(tgt) = tgt {
                fixes.push(Fix { blk: i as u32, off: off as u32, tgt });
            }
            off += 8;
        }
    }
    fix_at.push(fixes.len() as u32);
    // words the decode wrote into the file image itself (native pointers some tables keep there): kept as patches the
    // apply writes into the game's copy, and the image put back as read (the plan is matched by these bytes)
    let mut raw_patch: Vec<(u32, u64, Option<Tgt>)> = Vec::new();
    {
        let img = unsafe { core::slice::from_raw_parts_mut(raw as *mut u8, len) };
        let mut off = 0;
        while off < len {
            let end = (off + 8).min(len);
            if img[off..end] != file[off..end] {
                let mut w = [0u8; 8];
                w[..end - off].copy_from_slice(&img[off..end]);
                let v = u64::from_le_bytes(w) as usize;
                let tgt = if end - off < 8 {
                    None
                } else if v >= rec.data_lo && v <= rec.bump {
                    rec.find(v).map(|j| Tgt::Blk(j as u32, (v - bl[j].at) as u32))
                } else if v >= raw && v <= raw + len {
                    Some(Tgt::Raw((v - raw) as u32))
                } else {
                    None
                };
                raw_patch.push((off as u32 | ((end - off) as u32) << 28, u64::from_le_bytes(w), tgt));
                img[off..end].copy_from_slice(&file[off..end]);
            }
            off = end;
        }
    }
    let t_scan = std::time::Instant::now();
    // a live block the result does not reach was allocated for something else (a lazily built global): a replay would
    // allocate it but the in-place decode on the game thread would not; the plan is not usable
    let mut seen = vec![false; bl.len()];
    let mut stack = vec![root];
    seen[root] = true;
    for &(_, _, t) in &raw_patch {
        if let Some(Tgt::Blk(j, _)) = t {
            if !seen[j as usize] {
                seen[j as usize] = true;
                stack.push(j as usize);
            }
        }
    }
    while let Some(i) = stack.pop() {
        for f in &fixes[fix_at[i] as usize..fix_at[i + 1] as usize] {
            if let Tgt::Blk(j, _) = f.tgt {
                if bl[j as usize].live == 1 && !seen[j as usize] {
                    seen[j as usize] = true;
                    stack.push(j as usize);
                }
            }
        }
    }
    let escaped = bl.iter().enumerate().filter(|(i, b)| b.live == 1 && !seen[*i]).count();
    // with dat_custom's side lists all empty again nothing outside the result holds a block of this decode: an
    // unreached live block is then a leak of the decode itself (an article's dynamics/hurt-bone list or special
    // attributes written twice: the first block loses its only pointer). The in-place decode leaks it the same way and
    // the replay reproduces it (every recorded call is replayed and every live block written), so the plan is usable.
    let held_outside = escaped > 0 && !crate::dat_custom::side_lists_empty();
    let blocks: Vec<(usize, usize, usize)> = bl.iter().map(|b| (b.at, b.size, b.align)).collect();
    let bytes = len + blocks.iter().map(|b| b.1).sum::<usize>();
    let events: Vec<u64> = rec.events().to_vec();
    let mut realloc_target = vec![false; bl.len()];
    for &e in &events {
        let (k, _, n) = recheap::ev::unpack(e);
        if k == recheap::ev::REALLOC {
            realloc_target[n] = true;
        }
    }
    if escaped > 0 && log_on() {
        eprintln!("asset: {escaped} live block(s) leaked by the decode (no outside holder): kept in the plan");
    }
    if held_outside {
        // the slot is kept (something outside the result may still point into it)
        let what: Vec<String> = bl
            .iter()
            .enumerate()
            .filter(|(i, b)| b.live == 1 && !seen[*i])
            .take(4)
            .map(|(i, b)| format!("#{i}/{} size {} align {} words {:x?}", bl.len(), b.size, b.align, unsafe { core::slice::from_raw_parts(b.at as *const u64, (b.size / 8).min(4)) }))
            .collect();
        eprintln!("asset: {escaped} live block(s) the decode result does not reach and a side list is not empty: plan dropped (the load decodes in place): {}", what.join("; "));
        return Err(NoPlan::Unsafe);
    }
    Ok(Plan {
        slot,
        raw,
        len,
        loading_in: loading,
        resolved,
        grkind: grkind_used.then_some(grkind),
        sparse: sparse_runs(&blocks, &page_written),
        blocks,
        fixes,
        fix_at,
        events,
        realloc_target,
        live_mask: bl.iter().map(|b| b.live == 1).collect(),
        raw_patch,
        out: root as u32,
        escaped: 0,
        dead_refs,
        bytes,
        live_bytes: bl.iter().filter(|b| b.live == 1).map(|b| b.size).sum(),
        prep_ms: t0.elapsed().as_secs_f64() * 1e3,
        prep_split: [(t_dec - t0).as_secs_f64() * 1e3, (t_scan - t_dec).as_secs_f64() * 1e3, t_scan.elapsed().as_secs_f64() * 1e3],
    })
}

const SPARSE_MIN: usize = 64 << 10;
/// Live bytes from which [`apply`] splits its block writes over [`PAR_THREADS`] threads.
const PAR_MIN: usize = 8 << 20;
const PAR_THREADS: usize = 4;
/// ... or fixups from which it does.
const PAR_MIN_FIXES: usize = 100_000;

/// `MELEE_ASSET_SPARSE=off`: always copy whole blocks (diagnostics).
fn sparse_on() -> bool {
    static S: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *S.get_or_init(|| std::env::var("MELEE_ASSET_SPARSE").map_or(true, |v| v != "off"))
}

fn sparse_runs(blocks: &[(usize, usize, usize)], page_written: &dyn Fn(usize) -> bool) -> Vec<(u32, u32, u32)> {
    let mut v = Vec::new();
    for (k, &(at, size, _)) in blocks.iter().enumerate() {
        if size < SPARSE_MIN {
            continue;
        }
        let b = unsafe { core::slice::from_raw_parts(at as *const u8, size) };
        let mut off = 0;
        while off < size {
            let end = (off + 4096).min(size);
            let touched = page_written(at + off) || page_written(at + end - 1);
            if touched && b[off..end].iter().any(|&x| x != 0) {
                match v.last_mut() {
                    Some((kk, o, n)) if *kk == k as u32 && (*o + *n) as usize == off => *n += (end - off) as u32,
                    _ => v.push((k as u32, off as u32, (end - off) as u32)),
                }
            }
            off = end;
        }
    }
    v
}

/// Replay `p` into the game arena (game thread, inside the game scope): the `Decoded` for the file at `src`.
///
/// # Safety
/// `src` holds the same bytes the plan was prepared from (not yet relocated).
pub unsafe fn apply(p: &Plan, src: *mut u8) -> Box<Decoded> {
    let t0 = std::time::Instant::now();
    let (mut addr, sparse_at): (Vec<usize>, Vec<u32>) = {
        let _s = crate::arena::SysScope::enter();
        let _ = sparse_on(); // a first read allocates: never on the asset path
        let mut sa = vec![0u32; p.blocks.len() + 1];
        let mut j = 0;
        for (k, slot) in sa.iter_mut().enumerate() {
            while j < p.sparse.len() && (p.sparse[j].0 as usize) < k {
                j += 1;
            }
            *slot = j as u32;
        }
        (vec![0usize; p.blocks.len()], sa)
    };
    // block k's last contents at its address (a fresh replay block is zero: the asset path zeroes what it takes, so
    // only the non-zero pages are copied, except into a realloc's block, which holds the old bytes)
    let srcu = src as usize; // (a usize: the closure is shared with the copy threads below)
    let write = |k: usize, addr: &Vec<usize>| {
        let src = srcu;
        let (at, size, _) = p.blocks[k];
        let dst = addr[k];
        if size >= SPARSE_MIN && !p.realloc_target[k] && sparse_on() {
            for &(_, off, n) in &p.sparse[sparse_at[k] as usize..sparse_at[k + 1] as usize] {
                core::ptr::copy_nonoverlapping((at + off as usize) as *const u8, (dst + off as usize) as *mut u8, n as usize);
            }
        } else {
            core::ptr::copy_nonoverlapping(at as *const u8, dst as *mut u8, size);
        }
        for f in &p.fixes[p.fix_at[k] as usize..p.fix_at[k + 1] as usize] {
            let v = match f.tgt {
                Tgt::Blk(j, d) => addr[j as usize] + d as usize,
                Tgt::Raw(d) => src as usize + d as usize,
                Tgt::Null => 0,
                Tgt::RawRel(d) => (src as usize + d as usize).wrapping_sub(dst),
            };
            ((dst + f.off as usize) as *mut usize).write_unaligned(v);
        }
    };
    let lay = |k: usize| Layout::from_size_align_unchecked(p.blocks[k].1, p.blocks[k].2);
    // a block's last contents just before its free: all of it, unless the free drops them anyway (a trimmed big
    // block: its pages are decommitted, as after the in-place decode's free)
    let last_write = |a: usize, addr: &Vec<usize>| {
        if !crate::arena::free_drops_contents(p.blocks[a].1) {
            write(a, addr);
            return;
        }
        // only the bytes the decommit keeps: up to the first page boundary past +16, and the last partial page
        let (at, size, _) = p.blocks[a];
        let dst = addr[a];
        let head = ((dst + 16 + 0xFFF) & !0xFFF).min(dst + size) - dst;
        let tail = (((dst + size) & !0xFFF).max(dst + head)) - dst;
        for (lo, hi) in [(0, head), (tail, size)] {
            if hi > lo {
                core::ptr::copy_nonoverlapping((at + lo) as *const u8, (dst + lo) as *mut u8, hi - lo);
            }
        }
        // (a block's fixups are in offset order: the ones wholly inside the dropped middle are skipped at once)
        let fx = &p.fixes[p.fix_at[a] as usize..p.fix_at[a + 1] as usize];
        let lo = fx.partition_point(|f| (f.off as usize) < head);
        let hi = fx.partition_point(|f| f.off as usize + 8 <= tail).max(lo);
        for f in fx[..lo].iter().chain(fx[hi..].iter()) {
            let o = f.off as usize;
            let v = match f.tgt {
                Tgt::Blk(j, d) => addr[j as usize] + d as usize,
                Tgt::Raw(d) => srcu + d as usize,
                Tgt::Null => 0,
                Tgt::RawRel(d) => (srcu + d as usize).wrapping_sub(dst),
            }
            .to_le_bytes();
            // the bytes of the word inside the kept ranges (a word may straddle a range's edge)
            for (k, &byte) in v.iter().enumerate() {
                let b = o + k;
                if b < head || (b >= tail && b < size) {
                    ((dst + b) as *mut u8).write(byte);
                }
            }
        }
    };
    // the recorded calls, in order, through the process allocator (the arena decides exactly as it did for the
    // in-place decode: same blocks, same free lists)
    crate::arena::set_asset_path(true, digest_on());
    let mut t_temp = 0.0f64;
    let diag = log_on();
    let mut cat = [(0u32, 0u64, 0u64); 4];
    let (mut nfree, mut nrealloc) = (0u32, 0u32);
    for &e in &p.events {
        let (kind, a, b) = recheap::ev::unpack(e);
        match kind {
            recheap::ev::ALLOC => {
                if diag {
                    // MELEE_ASSET_LOG: where the allocation time goes (fresh bump / recycled, small / 64 KB and up)
                    let t = std::time::Instant::now();
                    let q = std::alloc::alloc(lay(a));
                    let ns = t.elapsed().as_nanos() as u64;
                    let c = (!crate::arena::last_fresh()) as usize * 2 + (p.blocks[a].1 >= SPARSE_MIN) as usize;
                    cat[c].0 += 1;
                    cat[c].1 += ns;
                    cat[c].2 += p.blocks[a].1 as u64;
                    assert!(!q.is_null(), "asset: replay out of memory");
                    addr[a] = q as usize;
                    continue;
                }
                let q = std::alloc::alloc(lay(a));
                assert!(!q.is_null(), "asset: replay out of memory");
                addr[a] = q as usize;
            }
            recheap::ev::FREE => {
                let t = diag.then(std::time::Instant::now);
                last_write(a, &addr);
                std::alloc::dealloc(addr[a] as *mut u8, lay(a));
                if let Some(t) = t {
                    t_temp += t.elapsed().as_secs_f64();
                    nfree += 1;
                }
            }
            _ => {
                let t = diag.then(std::time::Instant::now);
                let (sa, sb) = (p.blocks[a].1, p.blocks[b].1);
                if !crate::arena::contains(addr[a] as *const u8) {
                    // not an arena block (the arena refused the request): the process allocator's own realloc
                    write(a, &addr);
                    let q = std::alloc::realloc(addr[a] as *mut u8, lay(a), sb);
                    assert!(!q.is_null(), "asset: replay out of memory");
                    addr[b] = q as usize;
                } else if crate::arena::same_class(sa, sb) {
                    // the realloc stays in place. Growing: every byte of the block up to its new size is written
                    // again with the new block's last contents (a realloc's block is written whole, at its free or
                    // at the end), so the old contents and the zeroed growth need not be written first. Shrinking:
                    // the old block's bytes past the new size stay in memory, so they are written.
                    if sb < sa {
                        write(a, &addr);
                    }
                    addr[b] = addr[a];
                } else {
                    // what the process allocator's realloc does for an arena block of another class: allocate the
                    // new block, copy, free the old one. The copy is left out: the new block is written whole with
                    // its own last contents (at its free, or at the end), and nothing reads it before. The old
                    // block is written as any freed temporary is (only what its free keeps).
                    let q = std::alloc::alloc(Layout::from_size_align_unchecked(sb, p.blocks[a].2));
                    assert!(!q.is_null(), "asset: replay out of memory");
                    addr[b] = q as usize;
                    last_write(a, &addr);
                    std::alloc::dealloc(addr[a] as *mut u8, lay(a));
                }
                if let Some(t) = t {
                    t_temp += t.elapsed().as_secs_f64();
                    nrealloc += 1;
                }
            }
        }
    }
    let ta = std::time::Instant::now();
    // the live blocks' last contents (in index order: pointers only name blocks that exist by now)
    // big plans: split over a few threads (plain copies and fixups into blocks that are all allocated by now; the
    // bytes are the same whoever writes them). The spawns allocate: kept off the arena.
    let live_bytes: usize = p.live_bytes;
    // (many small blocks count too: ItCo's 58k blocks / 351k fixups are ~3 MB but 5 ms of single-threaded writes)
    let nt = if live_bytes >= PAR_MIN || p.fixes.len() >= PAR_MIN_FIXES { PAR_THREADS } else { 1 };
    if nt == 1 {
        for k in 0..p.blocks.len() {
            if p.live(k) {
                write(k, &addr);
            }
        }
    } else {
        let _s = crate::arena::SysScope::enter();
        let per = live_bytes / nt + 1;
        let mut cuts = vec![0usize];
        let mut acc = 0;
        for (k, b) in p.blocks.iter().enumerate() {
            if p.live(k) {
                acc += b.1 + 64;
                if acc >= per * cuts.len() && cuts.len() < nt {
                    cuts.push(k + 1);
                }
            }
        }
        cuts.push(p.blocks.len());
        let (w, a) = (&write, &addr);
        std::thread::scope(|sc| {
            for c in cuts.windows(2).skip(1) {
                let (lo, hi) = (c[0], c[1]);
                sc.spawn(move || {
                    for k in lo..hi {
                        if p.live(k) {
                            w(k, a);
                        }
                    }
                });
            }
            for k in cuts[0]..cuts[1] {
                if p.live(k) {
                    w(k, a);
                }
            }
        });
    }
    for &(o, w, t) in &p.raw_patch {
        let (off, n) = ((o & 0x0FFF_FFFF) as usize, (o >> 28) as usize);
        let v = match t {
            Some(Tgt::Blk(j, d)) => (addr[j as usize] + d as usize) as u64,
            Some(Tgt::Raw(d)) => (src as usize + d as usize) as u64,
            _ => w,
        };
        core::ptr::copy_nonoverlapping(v.to_le_bytes().as_ptr(), src.add(off), n);
    }
    APPLY_SPLIT.with(|c| c.set(((ta - t0).as_secs_f64() * 1e3, ta.elapsed().as_secs_f64() * 1e3, t_temp * 1e3)));
    if diag {
        let d = crate::arena::diag_take();
        let g = |c: (u64, u64, u64)| format!("{} x {:.2} ms {} KB", c.0, c.1 as f64 / 1e6, c.2 >> 10);
        eprintln!("asset: arena big blocks in this apply: decommit at free {}; recommit {}; kept-block take (zeroing) {}; tail decommit {}; frees kept {}", g(d[0]), g(d[1]), g(d[2]), g(d[3]), g(d[4]));
        let f = |c: (u32, u64, u64)| format!("{} allocs {:.2} ms {} KB", c.0, c.1 as f64 / 1e6, c.2 >> 10);
        eprintln!("asset: apply allocs: fresh small {}; fresh big {}; recycled small {}; recycled big {}; {nfree} frees, {nrealloc} reallocs", f(cat[0]), f(cat[1]), f(cat[2]), f(cat[3]));
    }
    let out = Box::from_raw(addr[p.out as usize] as *mut Decoded);
    {
        let _s = crate::arena::SysScope::enter();
        addr = Vec::new();
        drop(addr);
    }
    out
}

/// `MELEE_ASSET_DIGEST=1`: log a digest of every block a load took (address, size, bytes) to compare runs.
fn digest_on() -> bool {
    static D: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *D.get_or_init(|| std::env::var_os("MELEE_ASSET_DIGEST").is_some())
}

/// The digest of the asset path's blocks since [`crate::arena::set_asset_path`] (and turn the path off).
/// The executable's mapped range (Windows; empty elsewhere).
fn exe_image() -> (usize, usize) {
    #[cfg(windows)]
    unsafe {
        extern "system" {
            fn GetModuleHandleW(name: *const u16) -> *const u8;
        }
        let b = GetModuleHandleW(core::ptr::null());
        if b.is_null() {
            return (0, 0);
        }
        let pe = b.add((b.add(0x3C) as *const u32).read_unaligned() as usize);
        let size = (pe.add(0x18 + 56) as *const u32).read_unaligned() as usize; // OptionalHeader.SizeOfImage
        (b as usize, b as usize + size)
    }
    #[cfg(not(windows))]
    (0, 0)
}

/// Is `p` in committed host memory (digest only).
fn committed(p: usize) -> bool {
    #[cfg(windows)]
    unsafe {
        #[repr(C)]
        struct Mbi {
            base: usize,
            alloc_base: usize,
            alloc_protect: u32,
            region_size: usize,
            state: u32,
            protect: u32,
            ty: u32,
        }
        extern "system" {
            fn VirtualQuery(p: *const u8, m: *mut Mbi, n: usize) -> usize;
        }
        if p >= 0x8000_0000_0000 {
            return false;
        }
        let mut m: Mbi = core::mem::zeroed();
        VirtualQuery(p as *const u8, &mut m, core::mem::size_of::<Mbi>()) != 0 && m.state == 0x1000
    }
    #[cfg(not(windows))]
    {
        let _ = p;
        false
    }
}

thread_local! {
    /// The file image of the load being digested.
    static SRC_FOR_DIGEST: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
}

fn end_asset_path(len: usize, how: &str) {
    crate::arena::set_asset_path(false, false);
    if !digest_on() {
        return;
    }
    // the calls (allocations and frees, by size, in order), then the contents of the blocks still live, with every
    // word that points into one of this load's blocks written as (block number, offset) and into the file image as
    // its offset: comparable across processes (the arena's addresses differ from run to run; the bytes must not)
    let v = crate::arena::take_asset_blocks();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    // MELEE_ASSET_DIGEST_DUMP=<dir>: also write each load's normalised token stream (u64 LE) for word-level diffs
    let dump = std::env::var_os("MELEE_ASSET_DIGEST_DUMP");
    let mut toks: Vec<u64> = Vec::new();
    let mut raw: Vec<u64> = Vec::new();
    let mut mix = |x: u64| {
        h = (h ^ x).wrapping_mul(0x0100_0000_01b3);
        if dump.is_some() {
            toks.push(x);
        }
    };
    let mut live: Vec<(usize, usize, usize)> = Vec::new(); // (addr, size, number)
    mix(v.len() as u64);
    for (k, &(a, n)) in v.iter().enumerate() {
        let freed = n >> 63 != 0;
        mix(freed as u64);
        mix((n & !(1 << 63)) as u64);
        if freed {
            live.retain(|b| b.0 != a);
        } else {
            live.push((a, n, k));
        }
    }
    let mut by_addr = live.clone();
    by_addr.sort_unstable();
    let src = SRC_FOR_DIGEST.with(|c| c.get());
    let img = exe_image();
    let arena0 = crate::arena::used_range().0;
    // per live block: its size, then each 8-byte word as (class, value): 0 raw, 1 this load's block (number,
    // offset), 2 file image offset, 3 exe image offset, 4 other arena object offset, 5 host pointer (Rust heap,
    // stack: per run, never game data, so its value is not compared; 1: self-relative), 6 file image - block start; the tail raw
    for &(a, n, _) in &live {
        let b = unsafe { core::slice::from_raw_parts(a as *const u8, n) };
        mix(n as u64);
        let mut off = 0;
        while off + 8 <= n {
            let w = u64::from_le_bytes(b[off..off + 8].try_into().unwrap()) as usize;
            let i = by_addr.partition_point(|x| x.0 <= w);
            let t = (a + off).wrapping_add(w);
            let (c, x) = if i > 0 && w <= by_addr[i - 1].0 + by_addr[i - 1].1 {
                (1, (by_addr[i - 1].2 as u64) << 32 | (w - by_addr[i - 1].0) as u64)
            } else if w >= src && w <= src + len {
                (2, (w - src) as u64)
            } else if w >= img.0 && w < img.1 {
                (3, (w - img.0) as u64)
            } else if crate::arena::contains(w as *const u8) {
                (4, (w - arena0) as u64)
            } else if w >> 47 != 0 && a.wrapping_add(w) >= src && a.wrapping_add(w) <= src + len {
                (6, (a.wrapping_add(w) - src) as u64) // file image address - this block's start (dat_custom banks)
            } else if w >> 47 != 0 && crate::arena::contains(a.wrapping_add(w) as *const u8) {
                (6, 1 << 40 | (a.wrapping_add(w) - arena0) as u64) // another image in the arena - this block's start
            } else if w >= 0x1_0000 && committed(w) {
                (5, 0)
            } else if w >> 47 != 0 && ((t >= img.0 && t < img.1) || (t >= 0x1_0000 && committed(t))) {
                (5, 1)
            } else {
                (0, w as u64)
            };
            mix(c);
            mix(x);
            if c >= 3 {
                raw.push(w as u64); // dump only: a data word that merely looks like a host pointer in one run
            }
            off += 8;
        }
        for &y in &b[off..] {
            mix(7);
            mix(y as u64);
        }
    }
    drop(mix);
    if let Some(d) = &dump {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let k = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut bytes = Vec::with_capacity(toks.len() * 8);
        for t in &toks {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        let _ = std::fs::write(std::path::Path::new(d).join(format!("{k:04}_{len}.bin")), bytes);
        let r: Vec<u8> = raw.iter().flat_map(|x| x.to_le_bytes()).collect();
        let _ = std::fs::write(std::path::Path::new(d).join(format!("{k:04}_{len}.raw")), r);
    }
    eprintln!("asset digest: {len} bytes {how}: {} calls {h:016x}", v.len());
}

thread_local! {
    /// The last apply's (event replay ms, live-block writes ms, of the replay: temporaries' writes + frees ms).
    static APPLY_SPLIT: core::cell::Cell<(f64, f64, f64)> = const { core::cell::Cell::new((0.0, 0.0, 0.0)) };
}

// ---- the service: cache, worker, hints ----

enum Src {
    Disc(u64, usize),
    /// A copy of a file the game decoded in place (kept for its next load).
    Bytes(Vec<u8>),
}
struct Job {
    entry: i32,
    src: Src,
    loading: i32,
    /// The game's `stage_info.grkind` at the hint (a stage file decoded for another stage is not used).
    grkind: i32,
}
struct Entry {
    /// Disc entry it was hinted for (-1: prepared on the spot).
    entry: i32,
    len: usize,
    plan: Option<Arc<Plan>>,
    /// Queued or being prepared.
    pending: bool,
    /// The worker has taken it (a load waits only for a started one; a queued one is prepared on the spot instead).
    started: bool,
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
    /// File lengths whose prepare gave no plan (the decode leaves blocks the result does not reach): never prepared
    /// again (each retry is a whole decode on the worker, ItCo's ~30 ms, competing with the game for nothing).
    unplannable: Vec<usize>,
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
    let _ = crate::archive::reclaim_flags(); // before any decode (see there)
    crate::arena::start_prefault();
    let _ = crate::archive::decode_prof_on(); // its first read allocates: never on the asset path (a decode)
    // MELEE_ASSET_SERVICE=off: no service, every load is the old in-place decode (what a platform without the recording
    // heap runs, e.g. Linux): lets that path be tested on Windows
    if std::env::var("MELEE_ASSET_SERVICE").is_ok_and(|v| v == "off") || !recheap::init() {
        return;
    }
    let worker = !std::env::var("MELEE_ASSETS").is_ok_and(|v| v == "off") && iso.is_some(); // on by default; MELEE_ASSETS=off
    let s = Box::leak(Box::new(Svc {
        m: Mutex::new(Inner { entries: Vec::new(), jobs: VecDeque::new(), clock: 0, stats: Stats::default(), unplannable: Vec::new() }),
        cv: Condvar::new(),
        iso,
        worker,
    }));
    SVC.store(s, core::sync::atomic::Ordering::Release);
    game_io(); // boot hints wait for the first load burst (the title's first frame) to end
    if worker {
        std::thread::Builder::new().name("asset-worker".into()).spawn(worker_main).expect("asset worker");
    }
}

/// When the game thread last decoded a file in place (ms since the service clock's start): a kept-copy prepare
/// ([`Src::Bytes`], for the NEXT load of a file) waits until the game has been quiet for [`BYTES_QUIET_MS`], so it
/// never competes with the decodes of the same load burst (Envoy's no-lead fight start decodes ~10 files in a frame).
static GAME_DECODE_AT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
const BYTES_QUIET_MS: u64 = 150;
/// The longest a load waits for the worker to finish preparing the same file (see [`load`]).
const WAIT_MAX: std::time::Duration = std::time::Duration::from_micros(1500);
/// A disc hint (a coming load, or a boot file) waits until the game thread has neither decoded nor read a file for
/// this long: it must never compete with a load burst in progress (the title's first frame reads the demo match's
/// files; a no-lead fight start decodes ~10 files in a frame). The service is started with the clock stamped, so boot
/// hints also wait for the first burst to end.
const DISC_QUIET_MS: u64 = 40;
/// The game thread read a disc file (`host_dev_com_request`): a load burst is in progress.
pub fn game_io() {
    GAME_DECODE_AT.store(now_ms(), core::sync::atomic::Ordering::Relaxed);
}
fn now_ms() -> u64 {
    static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    T0.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64 + 1
}

fn worker_main() {
    crate::arena::mark_foreign_thread();
    let s = svc().unwrap();
    let mut f = s.iso.as_ref().and_then(|p| std::fs::File::open(p).ok());
    loop {
        let job = {
            let mut g = s.m.lock().unwrap();
            loop {
                // disc hints first (a load is coming); a kept-copy job only once the game's decodes went quiet
                let since = now_ms().saturating_sub(GAME_DECODE_AT.load(core::sync::atomic::Ordering::Relaxed));
                let pick = if since < DISC_QUIET_MS {
                    None
                } else {
                    g.jobs.iter().position(|j| matches!(j.src, Src::Disc(..))).or(if since >= BYTES_QUIET_MS { (!g.jobs.is_empty()).then_some(0) } else { None })
                };
                if pick.is_none() && !g.jobs.is_empty() {
                    g = s.cv.wait_timeout(g, std::time::Duration::from_millis(DISC_QUIET_MS / 2)).unwrap().0;
                    continue;
                }
                if let Some(j) = pick.and_then(|i| g.jobs.remove(i)) {
                    if let Some(e) = g.entries.iter_mut().find(|e| e.pending && e.entry == j.entry) {
                        e.started = true;
                    } else {
                        continue; // dropped meanwhile (its load came first)
                    }
                    break j;
                }
                g = s.cv.wait(g).unwrap();
            }
        };
        let bytes = match job.src {
            Src::Bytes(b) => Some(b),
            Src::Disc(off, len) => f.as_mut().and_then(|f| {
            use std::io::{Read, Seek, SeekFrom};
            let mut v = vec![0u8; len];
            f.seek(SeekFrom::Start(off)).ok()?;
            f.read_exact(&mut v).ok()?;
            Some(v)
            }),
        };
        let bytes = bytes.filter(|b| b.len() >= 0x20 && u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize == b.len());
        let res = match bytes {
            None => Err(NoPlan::NoSlot), // the read failed: not the file's fault
            Some(b) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| prepare(&b, job.loading, job.grkind))).unwrap_or(Err(NoPlan::Unsafe)),
        };
        let (plan, why) = match res { Ok(p) => (Some(p), None), Err(e) => (None, Some(e)) };
        let mut g = s.m.lock().unwrap();
        if matches!(why, Some(NoPlan::Unsafe | NoPlan::NotRouted)) {
            let n = g.entries.iter().find(|e| e.entry == job.entry).map_or(0, |e| e.len);
            if n != 0 && !g.unplannable.contains(&n) {
                g.unplannable.push(n);
            }
        }
        if why.is_some() && log_on() {
            eprintln!("asset: entry {:#x}: no plan ({why:?}){}", job.entry, if why == Some(NoPlan::NoSlot) { ": may be hinted again" } else { "" });
        }
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
    hint_entry_gr(entry, heap, None)
}

fn hint_entry_gr(entry: i32, heap: i32, gr: Option<i32>) {
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
    if g.entries.iter().any(|e| e.entry == entry) || g.unplannable.contains(&len) {
        return;
    }
    // a hint outranks a cached plan nobody asked for lately: make room (least recently used first)
    while g.entries.len() >= KEEP || g.entries.iter().filter_map(|e| e.plan.as_ref()).map(|p| p.bytes).sum::<usize>() + 8 * len > KEEP_BYTES {
        let Some(old) = (0..g.entries.len()).filter(|&i| g.entries[i].plan.is_some()).min_by_key(|&i| g.entries[i].used) else { break };
        g.entries.remove(old);
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
    // a hinted plan counts as just used: the eviction must not take the files the game is about to load first
    g.clock += 1;
    let now = g.clock;
    g.entries.push(Entry { entry, len, plan: None, pending: true, started: false, used: now });
    g.jobs.push_back(Job { entry, src: Src::Disc(off, len), loading: -1, grkind: gr.unwrap_or_else(hint_grkind) });
    drop(g);
    if log_on() {
        eprintln!("asset: hint entry {entry:#x} ({} KB)", len >> 10);
    }
    s.cv.notify_all();
}

/// A disc entry for a game file name (`PlFx.dat`; a name without an extension tries the locale's `.usd`, then `.dat`).
/// Pure: unlike `lbFileGetFullName` it writes no game static.
fn entry_of(name: *const i8) -> i32 {
    if name.is_null() {
        return -1;
    }
    let Some(h) = crate::host::Host::try_get() else { return -1 };
    let n = unsafe { core::ffi::CStr::from_ptr(name) }.to_string_lossy();
    if n.contains('.') {
        return h.disc.disc_entry(&n);
    }
    for ext in [".usd", ".dat"] {
        let e = h.disc.disc_entry(&format!("{n}{ext}"));
        if e > 0 {
            return e;
        }
    }
    -1
}

/// Hints for a fight decided ahead (Envoy: the sim publishes the coming room's plan at the camp / stop screens): the
/// stage file (prepared for that stage's grkind) and every fighter's data, costume (color 0, the one Envoy fights
/// use unless the CSS pick says otherwise) and animation bank. Reads only immutable name tables.
/// Hint ground kind `gr`'s stage file (no-op for 0 / an unknown kind).
pub fn hint_stage(gr: i32) {
    unsafe {
        let sd = &*core::ptr::addr_of!(melee_world::tu::ground::stage_datas);
        if gr > 0 && (gr as usize) < sd.len() && !sd[gr as usize].is_null() {
            let e = entry_of((*sd[gr as usize]).data1 as *const i8);
            if e > 0 {
                hint_entry_gr(e, -1, Some(gr));
            }
        }
    }
}

/// The stage select, once per tick (the play loop): `stkind` is the stage of the icon under the cursor (-1: none or
/// Random). Once the cursor has rested on a stage for [`HOVER_TICKS`] its file is hinted, so the worker has the
/// plan before the player confirms (the retail flow registers the stage with the preload cache only at the confirm,
/// in the tick that loads it: no lead). Host state only; never sim-visible.
pub fn sss_hover(stkind: i32) {
    use core::sync::atomic::{AtomicI32, Ordering::Relaxed};
    const HOVER_TICKS: i32 = 12;
    static LAST: AtomicI32 = AtomicI32::new(-1);
    static SINCE: AtomicI32 = AtomicI32::new(0);
    if LAST.swap(stkind, Relaxed) != stkind {
        SINCE.store(0, Relaxed);
        return;
    }
    if stkind < 0 || SINCE.fetch_add(1, Relaxed) + 1 != HOVER_TICKS {
        return;
    }
    // a GD stage (explore.rs: a custom map) plays on the Final Destination carrier: that is the file its match loads
    let stkind = if melee_modes::tu::explore::is_gd_stage(stkind) { melee_modes::tu::explore::FD_STKIND } else { stkind };
    let gr = unsafe {
        let map = &*core::ptr::addr_of!(melee_world::tu::stage::stage_id_map);
        if stkind as usize >= map.len() {
            return;
        }
        map[stkind as usize].grkind as i32
    };
    hint_stage(gr);
}

pub fn hint_envoy_plan(plan: &[i32; melee_modes::tu::envoy::PLAN_WORDS]) {
    use melee_modes::tu::envoy as ev;
    // not only an armed plan: the sim publishes the coming fight's plan from mod load on, and the camp door to that
    // fight can be a few ticks; prefetching it before the arm gives the first fight its lead (never sim-visible)
    if svc().is_none_or(|s| !s.worker) {
        return;
    }
    // once per distinct plan (the sim may publish the same plan every frame)
    static LAST: Mutex<Vec<i32>> = Mutex::new(Vec::new());
    {
        let _s = crate::arena::SysScope::enter();
        let mut l = LAST.lock().unwrap();
        if l.as_slice() == &plan[..] {
            return;
        }
        *l = plan.to_vec();
    }
    let mut cks: Vec<i32> = Vec::new();
    let _s = crate::arena::SysScope::enter();
    cks.extend([plan[ev::P_MAIN], plan[ev::P_PARTNER], plan[ev::P_MAIN2], plan[ev::P_PARTNER2]]);
    // an unset main / partner becomes Fox / Marth at match_begin (envoy.rs `pair`): hint those defaults too
    if plan[ev::P_MAIN] < 0 || plan[ev::P_PARTNER] < 0 {
        cks.extend([0x02, 0x09]);
    }
    let ncpu = plan[ev::P_NCPU].clamp(0, 3) as usize;
    cks.extend(&plan[ev::P_CPU_CK..ev::P_CPU_CK + ncpu]);
    cks.extend(&plan[ev::P_WAVE_CK..ev::P_WAVE_CK + 3]);
    cks.extend(&plan[ev::P_GAUNT_CK..ev::P_GAUNT_CK + 6]);
    let gr = plan[ev::P_STAGE];
    unsafe {
        // the fight's stage and the one a live stage transition goes to (its file in memory before the hand-off)
        hint_stage(gr);
        hint_stage(plan[ev::P_STAGE_NEXT]);
        let mapping = &*core::ptr::addr_of!(melee_world::pl::ftMapping_list);
        let names = &*core::ptr::addr_of!(melee_fighter::tu::ftdata::ftData_803C1F40);
        let costumes = &*core::ptr::addr_of!(melee_fighter::tu::ftdata::ftData_803C2360);
        let banks = &*core::ptr::addr_of!(melee_fighter::tu::ftdata::ftData_803C23E4);
        let mut done: Vec<i32> = Vec::new();
        for ck in cks {
            if ck < 0 || ck as usize >= mapping.len() {
                continue;
            }
            let m = &mapping[ck as usize];
            for k in [m.internal_id as i32, m.extra_internal_id as i32] {
                if k < 0 || k as usize >= names.len() || done.contains(&k) {
                    continue;
                }
                done.push(k);
                let k = k as usize;
                let mut files = vec![names[k].a as *const i8, banks[k] as *const i8];
                if !costumes[k].is_null() {
                    files.push((*costumes[k]).dat_filename as *const i8);
                }
                for f in files {
                    let e = entry_of(f);
                    if e > 0 {
                        hint_entry_gr(e, -1, None);
                    }
                }
            }
        }
    }
    if log_on() {
        eprintln!("asset: envoy plan hinted (stage {gr})");
    }
}

/// Files every session loads early, hinted at boot so the worker starts on them at once: the title's, the menus' and
/// character select's archives, then the match commons (in the order the game needs them). Either language's name.
const BOOT_FILES: &[&str] = &[
    "GmTtAll.usd", "GmTtAll.dat",
    // Kirby's copy hats: every match (the title demo's too) with Kirby decodes the whole set at its load (~22 ms in
    // place at the title's first frame when the demo picked Kirby); small, so prepared early (files < MIN_BYTES skip)
    "PlKbCpDk.dat", "PlKbNrCpDk.dat", "PlKbCpFx.dat", "PlKbCpGw.dat", "PlKbNrCpGw.dat", "PlKbCpMs.dat", "PlKbCpMt.dat",
    "PlKbNrCpMt.dat", "PlKbCpNs.dat", "PlKbCpPk.dat", "PlKbCpPp.dat", "PlKbCpPr.dat", "PlKbNrCpPr.dat", "PlKbCpYs.dat",
    "PlKbCpSk.dat", "PlKbCpFc.dat", "PlKbNrCpFc.dat", "PlKbCpFe.dat", "PlKbCpPc.dat", "PlKbCpLg.dat", "PlKbCpCl.dat",
    "PlKbCpMr.dat", "PlKbCpZd.dat", "PlKbCpGn.dat", "PlKbCpCa.dat", "PlKbCpKp.dat", "PlKbCpLk.dat", "PlKbCpDr.dat",
    "PlKbCpPe.dat", "PlKbCpSs.dat",
    "MnMaAll.usd", "MnMaAll.dat", "MnSlChr.usd", "MnSlChr.dat", "MnExtAll.usd", "MnExtAll.dat", "MnSlMap.usd", "MnSlMap.dat",
    "EfCoData.dat", "ItCo.usd", "ItCo.dat", "IfAll.usd", "IfAll.dat", "PlCo.dat",
];

/// Disc warm-up (boot, once; `MELEE_DISC_WARM=off` keeps it off): a below-normal thread reads every game data file
/// of the image once through its own handle and throws the bytes away, so the operating system's file cache holds
/// them when the game thread asks. A scene change reads its files inside one tick (the port completes a disc read
/// at once): from the cache that is a memory copy (~0.3 ms per 1.5 MB measured), from the drive it was 1.2-1.4 ms
/// per 1.5 MB (5.7 ms of reads in the VS scene change's tick). Music streams and movies are left out (never read
/// inside a tick). The game's reads, their bytes and their order are unchanged: never sim-visible. It waits while the
/// game thread is reading ([`game_io`]) so it never competes with a load burst.
fn disc_warm(files: Vec<(u64, usize)>) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    let Some(iso) = svc().and_then(|s| s.iso.clone()) else { return };
    if files.is_empty() || std::env::var("MELEE_DISC_WARM").is_ok_and(|v| v == "off") {
        return;
    }
    ONCE.call_once(|| {
        let _s = crate::arena::SysScope::enter();
        let _ = std::thread::Builder::new().name("disc-warm".into()).spawn(move || {
            crate::arena::mark_foreign_thread();
            #[cfg(windows)]
            unsafe {
                extern "system" {
                    fn GetCurrentThread() -> *mut core::ffi::c_void;
                    fn SetThreadPriority(h: *mut core::ffi::c_void, p: i32) -> i32;
                }
                SetThreadPriority(GetCurrentThread(), -1);
            }
            use std::io::{Read, Seek, SeekFrom};
            let Ok(mut f) = std::fs::File::open(&iso) else { return };
            let t0 = std::time::Instant::now();
            let mut buf = vec![0u8; 1 << 20];
            let (mut bytes, mut n) = (0usize, 0usize);
            for (off, len) in files {
                if f.seek(SeekFrom::Start(off)).is_err() {
                    return;
                }
                let mut left = len;
                while left > 0 {
                    // never while the game thread is in a load burst
                    while now_ms().saturating_sub(GAME_DECODE_AT.load(core::sync::atomic::Ordering::Relaxed)) < DISC_QUIET_MS {
                        std::thread::sleep(std::time::Duration::from_millis(DISC_QUIET_MS / 2));
                    }
                    let k = left.min(buf.len());
                    if f.read_exact(&mut buf[..k]).is_err() {
                        return;
                    }
                    left -= k;
                    bytes += k;
                }
                n += 1;
            }
            if log_on() || std::env::var_os("MELEE_BOOT_TIMES").is_some() {
                eprintln!("asset: disc warm-up: {n} files, {} MB read in {:.0} ms", bytes >> 20, t0.elapsed().as_secs_f64() * 1e3);
            }
        });
    });
}

/// Boot hints ([`BOOT_FILES`]); the heap test is skipped (the game's heaps are not set up yet; the service never
/// allocates them anyway).
pub fn boot_hints() {
    let Some(h) = crate::host::Host::try_get() else { return };
    // `melee play` offline only (trim mode is its mark: the tests' golden / synctest / netplay runs do not warm)
    if crate::arena::trim() {
        let _s = crate::arena::SysScope::enter();
        disc_warm(h.disc.files_with_ext(&[".dat", ".usd", ".ssm", ".sem"]));
    }
    for f in BOOT_FILES {
        let e = h.disc.disc_entry(f);
        if e > 0 {
            hint_entry(e, -1);
        }
    }
}

/// The `stage_info.grkind` a hinted file will be loaded under: the coming scene's stage when the preload cache names
/// one (the stage-select choice: its file is hinted from there), else the current one.
fn hint_grkind() -> i32 {
    unsafe {
        let st = (*core::ptr::addr_of!(melee_runtime::tu::lbdvd::preloadCache)).new_scene.game_cache.stkind as usize;
        let map = &*core::ptr::addr_of!(melee_world::tu::stage::stage_id_map);
        if st != 0x148 && st < map.len() {
            return map[st].grkind as i32;
        }
        (*core::ptr::addr_of!(melee_world::tu::ground::stage_info)).grkind as i32
    }
}

/// The load seam (`archive::host_parse`): the decode of the file at `src`, applied from a plan. None: use the old
/// in-place decode (small file, or no recording heap).
///
/// # Safety
/// `src` holds `len` bytes of an archive whose header was checked.
pub unsafe fn load(src: *mut u8, len: usize) -> Option<Box<Decoded>> {
    let s = svc()?;
    let loading = melee_fighter::tu::ftdata::FTDATA_LOADING_KIND;
    let grkind = (*core::ptr::addr_of!(melee_world::tu::ground::stage_info)).grkind as i32;
    let file = core::slice::from_raw_parts(src as *const u8, len);
    let t0 = std::time::Instant::now();
    let _sys = crate::arena::SysScope::enter();
    let mut waited = false;
    let found: Option<Arc<Plan>> = if len < MIN_BYTES {
        None
    } else {
        let mut g = s.m.lock().unwrap();
        loop {
            let hit = g.entries.iter().filter_map(|e| e.plan.as_ref()).find(|p| p.len == len && p.fits(loading, grkind) && p.file() == file).cloned();
            if hit.is_some() {
                break hit;
            }
            // a hinted file of this length being prepared right now: wait for it (a readiness miss, counted), but only
            // for a moment. The wait had no bound: a stage picked on the stage select is hinted in the tick that loads
            // it, and the game thread then sat out the worker's whole prepare (35 ms of the VS match's setup tick on
            // Final Destination, a file that decodes in place in a few ms). Past the bound the file is decoded in
            // place, which gives the same arena sequence as the replay (the service's rule), and the worker's plan
            // serves the next load.
            let left = WAIT_MAX.saturating_sub(t0.elapsed());
            if !left.is_zero() && g.entries.iter().any(|e| e.pending && e.started && e.len == len) {
                waited = true;
                g = s.cv.wait_timeout(g, left).unwrap().0;
                continue;
            }
            break None;
        }
    };
    let Some(plan) = found else {
        // no plan: decode in place, on the asset path (the sequence a plan replays). A hint of this file still queued
        // (not started) is dropped first: the worker would otherwise prepare the same file now, competing with this
        // decode and the frame's next ones for nothing (the Bytes job below keeps a plan for the next load)
        if len >= MIN_BYTES {
            let mut g = s.m.lock().unwrap();
            g.entries.retain(|e| !(e.pending && !e.started && e.len == len));
        }
        GAME_DECODE_AT.store(now_ms(), core::sync::atomic::Ordering::Relaxed);
        drop(_sys);
        let t = std::time::Instant::now();
        let out = {
            let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || {
                melee_core::frameprof::Text::new().s("asset decode in place ").n((len >> 10) as i64).s(" KB")
            });
            SRC_FOR_DIGEST.with(|c| c.set(src as usize));
            crate::arena::set_asset_path(true, digest_on());
            crate::dat_custom::begin_prepare(loading, grkind);
            let mut d = archive::decode(src, len);
            let (eff, _, _) = crate::dat_custom::end_prepare();
            d.effects = eff;
            let b = Box::new(d);
            end_asset_path(len, "direct");
            b
        };
        let ms = t.elapsed().as_secs_f64() * 1e3;
        GAME_DECODE_AT.store(now_ms(), core::sync::atomic::Ordering::Relaxed);
        if len >= MIN_BYTES {
            // keep a plan of it for the next load of the same bytes (the worker prepares it from a copy)
            let _sys = crate::arena::SysScope::enter();
            let mut g = s.m.lock().unwrap();
            g.stats.inline += 1;
            if s.worker && !g.entries.iter().any(|e| e.pending && e.len == len) && !g.unplannable.contains(&len) {
                g.clock += 1;
                let now = g.clock;
                let id = -2 - now as i32;
                g.entries.push(Entry { entry: id, len, plan: None, pending: true, started: false, used: now });

                g.jobs.push_back(Job { entry: id, src: Src::Bytes(file.to_vec()), loading, grkind });
                drop(g);
                s.cv.notify_all();
            }
            if log_on() {
                eprintln!("asset: load {} KB in place: {ms:.2} ms (no plan ready)", len >> 10);
            }
        }
        return Some(out);
    };
    let how = if waited { "wait" } else { "hit" };
    if std::env::var_os("MELEE_ASSET_SELFTEST").is_some() {
        // byte-identity check: the same file prepared on another (foreign) thread must give the same plan
        let copy: Vec<u8> = file.to_vec();
        let other = std::thread::spawn(move || {
            crate::arena::mark_foreign_thread();
            prepare(&copy, loading, grkind).ok()
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
    if log_on() {
        crate::arena::DIAG_ON.store(true, core::sync::atomic::Ordering::Relaxed);
        let d = crate::arena::diag_take();
        let f = |c: (u64, u64, u64)| format!("{} x {:.2} ms {} KB", c.0, c.1 as f64 / 1e6, c.2 >> 10);
        eprintln!("asset: arena big blocks since the last apply: decommit at free {}; recommit {}; kept-block take (zeroing) {}; tail decommit {}; frees kept {}", f(d[0]), f(d[1]), f(d[2]), f(d[3]), f(d[4]));
    }
    let t1 = std::time::Instant::now();
    let pf0 = if log_on() { archive::page_faults() } else { 0 };
    let out = {
        drop(_sys);
        let _zh = melee_core::frameprof::Span::new(melee_core::frameprof::Kind::Note, || {
            melee_core::frameprof::Text::new().s("asset apply ").s(how).s(" ").n((len >> 10) as i64).s(" KB")
        });
        SRC_FOR_DIGEST.with(|c| c.set(src as usize));
        let o = apply(&plan, src);
        end_asset_path(len, "replay");
        o
    };
    let _sys = crate::arena::SysScope::enter();
    let mut g = s.m.lock().unwrap();
    g.clock += 1;
    let now = g.clock;
    g.stats.hits += 1;
    if waited {
        g.stats.waits += 1;
    }
    if let Some(e) = g.entries.iter_mut().find(|e| e.plan.as_ref().is_some_and(|p| Arc::ptr_eq(p, &plan))) {
        e.used = now;
    }
    if log_on() {
        let st = &g.stats;
        let (rep, livew, temp) = APPLY_SPLIT.with(|c| c.get());
        eprintln!(
            "asset: load {} KB {how}: wait {:.2} ms, apply {:.2} ms ({} page faults) [events {:.2} (temporaries {:.2}) live writes {:.2}; {} events, {} KB of blocks] ({} blocks, {} fixups, {} dead refs; prepared in {:.2} ms = decode {:.2} scan {:.2} build {:.2}) [hints {} ahead {} hits {} waits {} in place {}]",
            len >> 10,
            (t1 - t0).as_secs_f64() * 1e3,
            t1.elapsed().as_secs_f64() * 1e3,
            archive::page_faults().wrapping_sub(pf0),
            rep,
            temp,
            livew,
            plan.events.len(),
            (plan.bytes - plan.len) >> 10,
            plan.blocks.len(),
            plan.fixes.len(),
            plan.dead_refs,
            plan.prep_ms,
            plan.prep_split[0],
            plan.prep_split[1],
            plan.prep_split[2],
            st.hints,
            st.prepared_ahead,
            st.hits,
            st.waits,
            st.inline
        );
    }
    Some(out)
}
