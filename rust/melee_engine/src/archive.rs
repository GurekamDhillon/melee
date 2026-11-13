//! The archive host (consolidation plan 1.4). An archive is kept as two images:
//!
//! 1. the **raw image**: the file bytes exactly as the DVD seam delivered them (big endian) in a block the
//!    caller owns below 4 GiB, relocated the way the C's `Locate` does it (`*ptr += (u32) data`, in big-endian
//!    words). Byte-level readers (`psInitDataBankLocate` on a nested archive, textures, `yakumono_param`...) use it;
//! 2. the **native image**: every public symbol the table (`dat_types.rs`) knows, decoded into native
//!    little-endian structs by `melee_core::gcdecode` over the same relocation set. Pointers inside it point into
//!    the same native graph.
//!
//! The native `HSD_Archive` record (GC layout via melee_types, native endianness) gets decoded header fields and
//! native copies of the relocation / public / extern tables, so the ported `HSD_ArchiveGetExtern` and friends read
//! it unchanged. `host_public_address(name)` answers the native root (or the raw address for a `raw` symbol)
//! and stops, naming the symbol, when the table does not know it.

use crate::dat_types::{self, RootType};
use crate::host::Host;
use core::ffi::{c_char, c_void, CStr};
use melee_core::gcdecode::GcArena;
use melee_runtime::dat::{DatArchive, DatSymbol};
use melee_types::{HSD_Archive, HSD_ArchiveExternInfo, HSD_ArchiveHeader, HSD_ArchivePublicInfo, HSD_ArchiveRelocationInfo};
use std::collections::HashMap;

const HEADER_BYTES: usize = 0x20;

#[derive(Clone, Copy)]
#[repr(u64)] // a full-word tag: no padding bytes
pub(crate) enum Root {
    Native(*mut c_void),
    Raw(usize), // a full word: a u32 payload would leave 4 uninitialised bytes in every root (the plan must be byte-exact)
}

/// What the host keeps per parsed archive.
pub struct ArchiveImage {
    /// Address of the raw image (the file bytes), also the C's `top_ptr`.
    pub raw: usize,
    pub raw_len: usize,
    /// The parsed file (data block copy, relocation set, symbols), built before relocation.
    pub dat: DatArchive,
    roots: HashMap<String, Root>,
    /// Owns every native object reachable from the roots.
    _arena: Option<GcArena>,
    /// What the bytes could not say while decoding (see gcdecode).
    pub issues: Vec<String>,
    // Native copies of the big-endian tables; `HSD_Archive.reloc_info` etc. point into these.
    _reloc: Vec<HSD_ArchiveRelocationInfo>,
    _public: Vec<HSD_ArchivePublicInfo>,
    _extern: Vec<HSD_ArchiveExternInfo>,
    /// Diagnostics: game-arena bytes the parse left live (data copy, tables, decoded objects).
    pub cost: usize,
}

impl ArchiveImage {
    pub fn publics(&self) -> &[DatSymbol] {
        &self.dat.publics
    }
    /// The native address of the public symbol `name` (None: not a public, or not in the type table).
    pub fn native_root(&self, name: &str) -> Option<*mut c_void> {
        match self.roots.get(name)? {
            Root::Native(p) => Some(*p),
            Root::Raw(off) => Some((self.raw + HEADER_BYTES + *off as usize) as *mut c_void),
        }
    }
}

fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}

/// `host_parse`: header and table decode into `archive`, in-place big-endian relocation of the raw image
/// at `src`, and the native decode of the known roots. Returns -1 like the C on a size mismatch.
///
/// # Safety
/// `archive` is a writable `HSD_Archive`; `src` points at `file_size` bytes below 4 GiB that stay alive.
/// Diagnostics: nanoseconds spent in [`host_parse`] / files parsed (the tick-spike breakdown reads and resets them).
pub static PARSE_NS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub static PARSE_N: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

pub unsafe fn host_parse(archive: *mut HSD_Archive, src: *mut u8, file_size: usize) -> i32 {
    let t0 = std::time::Instant::now();
    let r = host_parse_timed(archive, src, file_size);
    PARSE_NS.fetch_add(t0.elapsed().as_nanos() as u64, core::sync::atomic::Ordering::Relaxed);
    PARSE_N.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if melee_core::frameprof::on() {
        melee_core::frameprof::add(melee_core::frameprof::C::Parses, 1);
        melee_core::frameprof::add(melee_core::frameprof::C::ParseNs, t0.elapsed().as_nanos() as u64);
    }
    r
}

/// What a decode of one file produces (asset.rs moves it whole: built on any thread in the recording heap, copied
/// into the game arena at the apply). A pure function of the file bytes (and `FTDATA_LOADING_KIND`).
pub struct Decoded {
    pub dat: DatArchive,
    pub(crate) roots: Vec<(String, Root)>,
    pub(crate) arena: GcArena,
    pub issues: Vec<String>,
    pub(crate) reloc: Vec<HSD_ArchiveRelocationInfo>,
    pub(crate) public: Vec<HSD_ArchivePublicInfo>,
    pub(crate) externs: Vec<HSD_ArchiveExternInfo>,
    pub(crate) effects: Vec<crate::dat_custom::Effect>,
}

/// Parse and decode the (not yet relocated) file image at `src`, whose header was checked; the decoded roots point
/// into it. Touches no game state: dat_custom keeps its writes as effects while a prepare is on, else does them.
///
/// # Safety
/// `src` points at `file_size` readable bytes.
pub unsafe fn decode(src: *mut u8, file_size: usize) -> Decoded {
    let file = core::slice::from_raw_parts(src, file_size);
    let prof = decode_prof_on();
    if prof {
        crate::arena::PROF_NS.with(|c| c.set((0, 0, 0)));
        let main = std::thread::current().name() == Some("main");
        crate::arena::PROF_ALLOC.store(main, core::sync::atomic::Ordering::Relaxed);
        melee_core::gcdecode::PROF.store(main, core::sync::atomic::Ordering::Relaxed);
    }
    let pf0 = if prof { page_faults() } else { 0 };
    let tp0 = std::time::Instant::now();
    let dat = DatArchive::parse(file).unwrap_or_else(|e| panic!("host_parse: {e}"));
    let tp1 = std::time::Instant::now();
    let h = dat.header;
    let mut offset = HEADER_BYTES as u32;
    if h.data_size != 0 {
        offset = h.data_size + HEADER_BYTES as u32;
    }
    let reloc: Vec<HSD_ArchiveRelocationInfo> = dat.relocs.iter().map(|&o| HSD_ArchiveRelocationInfo { offset: o }).collect();
    let tab = |first: usize, n: u32| -> Vec<(u32, u32)> {
        (0..n as usize).map(|i| (be32(file, first + 8 * i), be32(file, first + 8 * i + 4))).collect()
    };
    let pub_at = offset as usize + 4 * h.nb_reloc as usize;
    let ext_at = pub_at + 8 * h.nb_public as usize;
    let public: Vec<HSD_ArchivePublicInfo> =
        tab(pub_at, h.nb_public).into_iter().map(|(o, s)| HSD_ArchivePublicInfo { offset: o, symbol: s }).collect();
    let externs: Vec<HSD_ArchiveExternInfo> =
        tab(ext_at, h.nb_extern).into_iter().map(|(o, s)| HSD_ArchiveExternInfo { offset: o, symbol: s }).collect();
    let mut roots = Vec::new();
    let bounds = {
        let data = &dat.data;
        let mut v: Vec<u32> = dat.publics.iter().map(|q| q.offset).collect();
        v.push(data.len() as u32);
        for &r in &dat.relocs {
            if let Some(w) = data.get(r as usize..r as usize + 4) {
                let t = u32::from_be_bytes(w.try_into().unwrap());
                if (t as usize) < data.len() {
                    v.push(t);
                }
            }
        }
        v.sort_unstable();
        v.dedup();
        crate::dat_custom::Bounds(v, src as usize + HEADER_BYTES)
    };
    let tp2 = std::time::Instant::now();
    let mut tp3 = tp2;
    let mut tp4 = tp2;
    let mut psum = (0, 0, 0, 0, 0);
    let mut per_root: Vec<(f64, &str)> = Vec::new();
    let (arena, issues) = {
        let mut d = dat.decoder();
        tp3 = std::time::Instant::now();
        d.set_bounds(bounds.0.clone());
        d.set_raw_base(src as usize + HEADER_BYTES);
        d.set_roots(dat.publics.iter().map(|q| q.offset).collect());
        for p in &dat.publics {
            match dat_types::type_of(&p.name) {
                Some(RootType::Raw) => roots.push((p.name.clone(), Root::Raw(p.offset as usize))),
                Some(ty) => {
                    // an array root runs to the next public, or the end of the data section
                    let next = dat.publics.iter().map(|q| q.offset).filter(|&o| o > p.offset).min();
                    let extent = next.unwrap_or(dat.data.len() as u32) - p.offset;
                    let tr = std::time::Instant::now();
                    let r = dat_types::decode_root(&mut d, ty, p.offset, extent, &bounds, &p.name);
                    if prof {
                        d.run(); // (finish runs the same queue; draining it here only moves the fills into this root's time)
                        per_root.push((tr.elapsed().as_secs_f64() * 1e3, p.name.as_str()));
                    }
                    roots.push((p.name.clone(), Root::Native(r)));
                }
                None => {}
            }
        }
        if prof {
            d.run();
            tp4 = std::time::Instant::now();
            psum = d.prof_summary();
        }
        d.finish()
    };
    let tp5 = std::time::Instant::now();
    crate::dat_custom::fix_articles();
    crate::dat_custom::fix_dynamics();
    if prof {
        let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1e3;
        let tp6 = std::time::Instant::now();
        per_root.sort_by(|a, b| b.0.total_cmp(&a.0));
        let top: Vec<String> = per_root.iter().take(4).map(|(t, n)| format!("{n} {t:.2}")).collect();
        let ps = melee_runtime::dat::PARSE_SPLIT.with(|c| c.get());
        crate::arena::PROF_ALLOC.store(false, core::sync::atomic::Ordering::Relaxed);
        let al = crate::arena::PROF_NS.with(|c| c.get());
        let types: Vec<String> = arena.bytes_by_type().iter().take(5).map(|(t, n, b)| format!("{} x{n} {}KB", t.rsplit("::").next().unwrap_or(t), b >> 10)).collect();
        melee_core::gcdecode::PROF.store(false, core::sync::atomic::Ordering::Relaxed);
        let loops: Vec<String> = melee_core::gcdecode::prof_take().iter().take(6).map(|(t, h, n, ns)| format!("{} {h} {n} el {:.2} ms", t.rsplit("::").next().unwrap_or(t), *ns as f64 / 1e6)).collect();
        eprintln!("decode prof loops: {} [page faults {}]", loops.join(", "), page_faults().wrapping_sub(pf0));
        eprintln!("decode prof alloc: {:.2} ms in {} zeroed allocs ({} KB); top types: {}", al.0 as f64 / 1e6, al.1, al.2 >> 10, types.join(", "));
        eprintln!(
            "decode prof [{}]: {} KB ({} relocs): total {:.2} ms = parse {:.2} [copy {:.2} relocs {:.2} syms {:.2} set {:.2}] bounds {:.2} decoder-new {:.2} roots {:.2} finish/drop {:.2} fix {:.2}; {} blocks {} KB, {} shared arrays {} KB-elems, {} memo; top roots: {}",
            std::thread::current().name().unwrap_or("?"),
            file_size >> 10,
            h.nb_reloc,
            ms(tp0, tp6),
            ms(tp0, tp1),
            ps.0, ps.1, ps.2, ps.3,
            ms(tp1, tp2),
            ms(tp2, tp3),
            ms(tp3, tp4),
            ms(tp4, tp5),
            ms(tp5, tp6),
            psum.0,
            psum.1 >> 10,
            psum.2,
            psum.3 >> 10,
            psum.4,
            top.join(", ")
        );
    }
    crate::hwwatch::flush("decode end");
    crate::hwwatch::check_issues(&issues);
    Decoded { dat, roots, arena, issues, reloc, public, externs, effects: Vec::new() }
}

unsafe fn host_parse_timed(archive: *mut HSD_Archive, src: *mut u8, file_size: usize) -> i32 {
    if archive.is_null() {
        return -1;
    }
    assert!((src as usize) + file_size <= 0x1_0000_0000, "host_parse: the raw image must be below 4 GiB ({src:p})");
    let file = core::slice::from_raw_parts(src, file_size);
    if file_size < HEADER_BYTES || be32(file, 0) as usize != file_size {
        eprintln!("engine: host_parse: header says {:#x} bytes, buffer {src:p} is {file_size:#x}", if file_size >= 4 { be32(file, 0) } else { 0 });
        melee_runtime::tu::lbext::OSReport(
            "host_parse: byte-order mismatch! Please check data format %x %x\n",
            &[
                melee_runtime::tu::lbext::OsArg::U(if file_size >= 4 { be32(file, 0) } else { 0 }),
                melee_runtime::tu::lbext::OsArg::U(file_size as u32),
            ],
        );
        return -1;
    }
    let live0 = crate::arena::live_bytes();
    // the decode: a prepared plan applied (asset.rs; prepared on the spot when nothing was prepared ahead), or done in
    // place when the recording heap is not available (tests with their own allocator)
    let dec: Box<Decoded> = match crate::asset::load(src, file_size) {
        Some(d) => d,
        None => {
            // the particle-bank memo lives for this one decode only (as on the asset path: begin_prepare /
            // end_prepare). Kept across frames, its table (a game-arena block) was rewound by a snapshot load while
            // the thread-local map kept pointing at it: lookups after a rollback probed a stale or rewritten table
            // (an endless probe after a stage switch) and skipped building banks the resimulation must build.
            crate::dat_custom::reset_banks();
            let mut d = decode(src, file_size);
            crate::dat_custom::reset_banks();
            d.effects = Vec::new();
            Box::new(d)
        }
    };
    let Decoded { dat, roots, arena, issues, reloc, public, externs, effects } = *dec;
    crate::dat_custom::apply_effects(effects);
    let h = dat.header;

    core::ptr::write_bytes(archive as *mut u8, 0, core::mem::size_of::<HSD_Archive>());
    (*archive).flags |= 1;
    (*archive).header = HSD_ArchiveHeader {
        file_size: h.file_size,
        data_size: h.data_size,
        nb_reloc: h.nb_reloc,
        nb_public: h.nb_public,
        nb_extern: h.nb_extern,
        version: h.version,
        pad: [be32(file, 0x18), be32(file, 0x1C)],
    };
    let mut offset = HEADER_BYTES as u32;
    if h.data_size != 0 {
        (*archive).data = src.add(HEADER_BYTES);
        offset = h.data_size + HEADER_BYTES as u32;
    }
    let sym_at = offset as usize + 4 * h.nb_reloc as usize + 8 * h.nb_public as usize + 8 * h.nb_extern as usize;
    let mut image = ArchiveImage {
        raw: src as usize,
        raw_len: file_size,
        dat,
        roots: roots.into_iter().collect(),
        _arena: Some(arena),
        issues,
        _reloc: reloc,
        _public: public,
        _extern: externs,
        cost: 0,
    };
    if h.nb_reloc != 0 {
        (*archive).reloc_info = image._reloc.as_mut_ptr();
    }
    if h.nb_public != 0 {
        (*archive).public_info = image._public.as_mut_ptr();
    }
    if h.nb_extern != 0 {
        (*archive).extern_info = image._extern.as_mut_ptr();
    }
    if sym_at < h.file_size as usize {
        (*archive).symbols = src.add(sym_at) as *mut i8;
    }
    (*archive).top_ptr = src as *mut c_void;
    if std::env::var_os("MELEE_DAT_LIST").is_some() {
        let names: Vec<&str> = image.dat.publics.iter().map(|p| p.name.as_str()).collect();
        eprintln!("dat publics: {}", names.join(" "));
    }
    if std::env::var_os("MELEE_DAT_ISSUES").is_some() {
        for i in image.issues.iter().take(40) {
            eprintln!("dat issue ({} public(s), {} bytes): {i}", image.dat.publics.len(), image.dat.data.len());
        }
    }

    // Raw image: `Locate`, in big-endian words.
    let base = (*archive).data as usize as u32;
    for &r in &image.dat.relocs {
        let p = src.add(HEADER_BYTES + r as usize);
        let v = u32::from_be_bytes([*p, *p.add(1), *p.add(2), *p.add(3)]).wrapping_add(base);
        for (i, x) in v.to_be_bytes().iter().enumerate() {
            *p.add(i) = *x;
        }
    }

    image.cost = crate::arena::live_bytes().saturating_sub(live0);
    if std::env::var_os("MELEE_ARCHIVE_LOG").is_some() {
        let name = image.dat.publics.iter().map(|p| p.name.as_str()).take(3).collect::<Vec<_>>().join(",");
        eprintln!("engine: parsed {name} raw {} KB, data copy {} KB, arena cost {} KB", file_size >> 10, image.dat.data.len() >> 10, image.cost >> 10);
    }
    if unsafe { RECLAIM } {
        reclaim_overlapping(archive as usize, src as usize, file_size);
    }
    if Host::get().archives.contains_key(&(archive as usize)) {
        // the C reused this HSD_Archive address: the old image's native graph stays alive (the game may still point into it)
        // once per record (fighter animations re-parse the same record on every action change)
        static mut LOGGED: Option<std::collections::HashSet<usize>> = None;
        #[allow(static_mut_refs)]
        let first = unsafe { LOGGED.get_or_insert_with(Default::default).insert(archive as usize) };
        if first || std::env::var_os("MELEE_RECLAIM_LOG").is_some() {
            eprintln!("engine: archive record {archive:p} re-parsed; keeping the previous native image alive (logged once per record)");
        }
        let old = Host::get().archives.remove(&(archive as usize)).unwrap();
        Host::get().retired.push(old);
    }
    Host::get().archives.insert(archive as usize, image);
    0
}

/// Set by `Engine::next_vs` (a later scene of the same run, `melee play`'s rematches): a file parsed over the raw
/// bytes of earlier images frees those images. Their bytes were overwritten, so in the C anything still pointing into
/// them reads the new file's bytes; the native graphs would only keep a dead copy alive, and every match re-loads the
/// fighters and the stage, so without this each match keeps hundreds of MB. Off for the single-scene runs (goldens).
pub static mut RECLAIM: bool = false;

/// Turn [`RECLAIM`] on from the first scene (`melee play`: no golden compare, and an action change re-parses a
/// fighter's animation over the same buffer, ~20 times a second in a 4-player match: kept, they grew the retired list
/// by ~20 MB a match and every GX array lookup scanned it).
pub fn set_reclaim(on: bool) {
    unsafe { RECLAIM = on }
}

fn reclaim_overlapping(_record: usize, raw: usize, len: usize) {
    let h = Host::get();
    let hit = |im: &ArchiveImage| im.raw < raw + len && raw < im.raw + im.raw_len;
    for im in h.archives.values().chain(h.retired.iter()).filter(|im| hit(im)) {
        crate::gxbridge::forget_range(im.raw, im.raw + im.raw_len);
    }
    crate::gxbridge::forget_range(raw, raw + len);
    let before = h.retired.len();
    // MELEE_RECLAIM_POISON=1 (test knob): a dropped image's native objects are filled with 0xDD and leaked instead
    // of freed, so a stale holder faults deterministically on its next read (the crash report names it)
    let poison = reclaim_flags().0;
    // MELEE_RECLAIM_LEAK=1 (mitigation): a dropped image is forgotten (never freed): the lists stay short, its memory
    // stays as it was, so a stale holder reads and writes its own old object instead of someone else's block
    let leak = reclaim_flags().1;
    let (mut dropped, keep): (Vec<ArchiveImage>, Vec<ArchiveImage>) = std::mem::take(&mut h.retired).into_iter().partition(|im| hit(im));
    h.retired = keep;
    let mut dead: Vec<usize> = h.archives.iter().filter(|(_, im)| hit(im)).map(|(&k, _)| k).collect();
    // free in address order, never the map's per-process random order: the arena's free lists (and so every later
    // block's address, and an asset plan's replay of a decode that reclaims) must not depend on the hasher's seed
    dead.sort_unstable();
    for k in &dead {
        if let Some(im) = h.archives.remove(k) {
            dropped.push(im);
        }
    }
    for mut im in dropped {
        if leak && !poison {
            core::mem::forget(im);
            continue;
        }
        if poison {
            if let Some(a) = im._arena.take() {
                let v = a.poison_and_leak(0xDD);
                unsafe { (*core::ptr::addr_of_mut!(POISONED)).extend(v) };
            }
        }
    }
    if std::env::var_os("MELEE_RECLAIM_LOG").is_some() && (before != h.retired.len() || !dead.is_empty()) {
        eprintln!("engine: reclaimed {} archive image(s) under {raw:#x}+{len:#x}", before - h.retired.len() + dead.len());
    }
}

/// Diagnostics: (live images, retired images, raw bytes, arena bytes of their parses).
pub fn stats() -> (usize, usize, usize, usize) {
    let h = Host::get();
    let all = || h.archives.values().chain(h.retired.iter());
    (h.archives.len(), h.retired.len(), all().map(|i| i.raw_len).sum(), all().map(|i| i.cost).sum())
}

/// `host_public_address`: the native root of public `symbols` (null when the archive has no such
/// public). Stops, naming the symbol, when the archive has it but `dat_symbols.tsv` has no type for it.
/// None: `archive` was not parsed by this host.
///
/// # Safety
/// `symbols` is a NUL-terminated string.
pub unsafe fn host_public_address(archive: *mut HSD_Archive, symbols: *const c_char) -> Option<*mut c_void> {
    let want = CStr::from_ptr(symbols).to_string_lossy();
    let image = Host::get().archives.get(&(archive as usize))?;
    if !image.dat.publics.iter().any(|s| s.name == want) {
        return Some(core::ptr::null_mut());
    }
    match image.native_root(&want) {
        Some(p) => Some(p),
        None => panic!(
            "melee_engine: archive symbol `{want}` has no type in tools/rust-port/engine/dat_symbols.tsv \
             (add the C type its call site casts it to, then run gen_dat_types.py)"
        ),
    }
}

/// `host_locate_extern` on the raw image: patches the chain of slots the extern's data offset starts, in
/// big-endian words, with the low 32 bits of `addr`. False when `archive` was not parsed by this host.
///
/// # Safety
/// `symbols` is a NUL-terminated string.
pub unsafe fn host_locate_extern(archive: *mut HSD_Archive, symbols: *const c_char, addr: *mut c_void) -> bool {
    let Some(image) = Host::get().archives.get(&(archive as usize)) else { return false };
    let want = CStr::from_ptr(symbols).to_string_lossy();
    let Some(ext) = image.dat.externs.iter().find(|s| s.name == want) else { return true };
    let data = (*archive).data;
    let mut offset = ext.offset;
    while offset != u32::MAX && offset < (*archive).header.data_size {
        let slot = data.add(offset as usize);
        let next = u32::from_be_bytes([*slot, *slot.add(1), *slot.add(2), *slot.add(3)]);
        for (i, x) in (addr as usize as u32).to_be_bytes().iter().enumerate() {
            *slot.add(i) = *x;
        }
        offset = next;
    }
    true
}

/// A public of a parsed archive the type table has no type for, as raw big-endian bytes: (the data block's address,
/// its length, the symbol's data offset). mex_guest answers an m-ex fighter's lookup of such a symbol (the mod's own
/// costume data, PlyTailsColor) with these bytes, which only the guest reads.
pub fn untyped_public(archive: *mut HSD_Archive, name: &str) -> Option<(usize, u32, u32)> {
    let image = Host::get().archives.get(&(archive as usize))?;
    let off = image.dat.publics.iter().find(|s| s.name == name)?.offset;
    if image.native_root(name).is_some() {
        return None;
    }
    Some((image.raw + HEADER_BYTES, image.dat.data.len() as u32, off))
}

/// The objects `MELEE_RECLAIM_POISON` filled with 0xDD: (address, bytes, type).
static mut POISONED: Vec<(usize, usize, &'static str)> = Vec::new();

/// `MELEE_RECLAIM_POISON`: report (once each) a poisoned object something wrote into after its image was dropped: the
/// type and the first changed offset name the stale holder's target. Call once per frame; a no-op without the knob.
pub fn check_poison(frame: u64) {
    let v = unsafe { &mut *core::ptr::addr_of_mut!(POISONED) };
    v.retain(|&(p, n, ty)| {
        // SAFETY: the block was leaked (never freed), so it stays readable.
        let b = unsafe { core::slice::from_raw_parts(p as *const u8, n) };
        match b.iter().position(|&x| x != 0xDD) {
            Some(o) => {
                let w = &b[o..(o + 16).min(n)];
                eprintln!("engine: POISON WRITTEN at frame {frame}: {ty} at {p:#x} (+{o:#x} of {n} bytes) now {w:02x?}");
                false
            }
            None => true,
        }
    });
}


/// The reclaim test knobs (poison, leak, sorted: now always on), read once. [`crate::asset::init`] reads them at boot: a first read
/// inside a decode would put its temporary allocation into whichever decode came first (a recorded plan or the
/// arena), so a replay and a direct decode would differ by it.
pub fn reclaim_flags() -> (bool, bool, bool) {
    static F: std::sync::OnceLock<(bool, bool, bool)> = std::sync::OnceLock::new();
    *F.get_or_init(|| {
        let _s = crate::arena::SysScope::enter();
        let on = |k: &str| std::env::var_os(k).is_some();
        (on("MELEE_RECLAIM_POISON"), on("MELEE_RECLAIM_LEAK"), on("MELEE_RECLAIM_SORTED"))
    })
}

/// `MELEE_DECODE_PROF=1`: one line per archive decode with its phase times (diagnostics; it drains the fill queue per
/// root, so the allocation ORDER differs from a normal run: never for digests or proofs).
pub(crate) fn decode_prof_on() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var_os("MELEE_DECODE_PROF").is_some())
}

/// The process's page-fault count (diagnostics; 0 off Windows).
pub fn page_faults() -> u32 {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Counters { cb: u32, page_faults: u32, rest: [usize; 9] }
        extern "system" {
            fn GetCurrentProcess() -> *mut core::ffi::c_void;
            fn K32GetProcessMemoryInfo(p: *mut core::ffi::c_void, c: *mut Counters, cb: u32) -> i32;
        }
        let mut c = Counters { cb: core::mem::size_of::<Counters>() as u32, page_faults: 0, rest: [0; 9] };
        if unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) } != 0 {
            return c.page_faults;
        }
    }
    0
}
