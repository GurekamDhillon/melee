//! Diagnostics: `MELEE_HEAP_LEAKS=1` records every live OS-heap block with the call stack that allocated it, tagged with
//! the stage-switch epoch it was born in ([`mark`], called at the end of each switch). At each mark it reports the blocks
//! born two epochs back (during the visit before the previous one, install included) that are still live: a block that
//! outlived the stage it was allocated for. Off (every normal run) it costs one relaxed load per allocation.
//! Not game state: the table lives in a std Mutex and is never snapshotted (use it with goldens, not synctest).
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::sync::Mutex;

static STATE: AtomicU8 = AtomicU8::new(0); // 0 unknown, 1 off, 2 on

struct Block {
    heap: i32,
    size: u32,
    epoch: u32,
    bt: std::backtrace::Backtrace,
}

struct Table {
    live: HashMap<usize, Block>,
    /// Pool pieces (hsdAllocMemPiece / HSD_ObjAlloc): objects, which go back to a free list rather than the heap.
    pieces: HashMap<usize, Block>,
    epoch: u32,
}

static TABLE: Mutex<Option<Table>> = Mutex::new(None);

#[inline]
pub fn on() -> bool {
    match STATE.load(Relaxed) {
        2 => true,
        1 => false,
        _ => {
            let v = std::env::var_os("MELEE_HEAP_LEAKS").is_some();
            STATE.store(if v { 2 } else { 1 }, Relaxed);
            v
        }
    }
}

pub fn alloc(heap: i32, p: usize, size: u32) {
    if !on() || p == 0 {
        return;
    }
    let bt = std::backtrace::Backtrace::force_capture();
    let mut t = TABLE.lock().unwrap();
    let t = t.get_or_insert_with(|| Table { live: HashMap::new(), pieces: HashMap::new(), epoch: 0 });
    let epoch = t.epoch;
    t.live.insert(p, Block { heap, size, epoch, bt });
}

pub fn free(p: usize) {
    if !on() || p == 0 {
        return;
    }
    if let Some(t) = TABLE.lock().unwrap().as_mut() {
        t.live.remove(&p);
    }
}

pub fn piece_alloc(p: usize, size: u32) {
    if !on() || p == 0 {
        return;
    }
    let bt = std::backtrace::Backtrace::force_capture();
    let mut t = TABLE.lock().unwrap();
    let t = t.get_or_insert_with(|| Table { live: HashMap::new(), pieces: HashMap::new(), epoch: 0 });
    let epoch = t.epoch;
    t.pieces.insert(p, Block { heap: -1, size, epoch, bt });
}

pub fn piece_free(p: usize) {
    if !on() || p == 0 {
        return;
    }
    if let Some(t) = TABLE.lock().unwrap().as_mut() {
        t.pieces.remove(&p);
    }
}

/// The interesting frames of a captured stack: game / engine functions, without the allocator and std plumbing.
fn frames(bt: &std::backtrace::Backtrace) -> String {
    let s = format!("{bt}");
    let mut out = Vec::new();
    for l in s.lines() {
        let l = l.trim();
        let Some((_, name)) = l.split_once(": ") else { continue };
        if l.starts_with("at ") || name.starts_with("std::") || name.starts_with("core::") || name.contains("heaptrace")
            || name.contains("alloc_from") || name.contains("OSAllocFromHeap") || name.contains("backtrace")
        {
            continue;
        }
        let short = name.rsplit("::").take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("::");
        out.push(short);
        if ["class::", "jobj::", "jobj_ptr::", "hsd::", "native::", "texp::", "objalloc::", "fobj::", "aobj::", "memory::", "id::", "lbarchive::"].iter().any(|m| name.contains(m)) && std::env::var_os("MELEE_HEAP_LEAKS_DEEP").is_some() {
            continue;
        }
        if name.contains("hsdAllocMemPiece") || name.contains("HSD_ObjAlloc") || name.contains("HSD_MemAlloc") || name.contains("hsdNew") || name.contains("_hsdClassAlloc") || name.starts_with("ext::") {
            continue;
        }
        if out.len() == 9 {
            break;
        }
    }
    out.join(" < ")
}

/// End of a stage switch: report the blocks of epoch `now - 2` still live, then start a new epoch.
pub fn mark(label: &str) {
    if !on() {
        return;
    }
    let mut g = TABLE.lock().unwrap();
    let t = g.get_or_insert_with(|| Table { live: HashMap::new(), pieces: HashMap::new(), epoch: 0 });
    if t.epoch >= 2 {
        let want = t.epoch - 2;
        let mut by: HashMap<String, (u32, u64, i32)> = HashMap::new();
        for b in t.live.values().chain(t.pieces.values()).filter(|b| b.epoch == want) {
            let e = by.entry(frames(&b.bt)).or_insert((0, 0, b.heap));
            e.0 += 1;
            e.1 += b.size as u64;
        }
        let mut v: Vec<_> = by.into_iter().collect();
        v.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
        let total: u64 = v.iter().map(|x| x.1 .1).sum();
        eprintln!("heaptrace: {label}: epoch {want} left {total} bytes live in {} sites", v.len());
        for (k, (n, bytes, heap)) in v.iter().take(12) {
            eprintln!("heaptrace:   heap {heap} {bytes:>8} B x{n:<4} {k}");
        }
    }
    t.epoch += 1;
}
