//! Layout check, part B: every field the native archive image holds, over every archive on the disc, against the
//! bytes the C build holds for the same file.
//!
//! The C build keeps an archive as the file's big-endian bytes in game memory, relocated in place (`Locate`), and
//! reads a field at its GC offset with its GC width; so the C value of `R.f` is the big-endian word at
//! `record + gc_offset(f)` of the file (a pointer: the relocated target). Part A (`c_layout_check.py`) proves those
//! GC offsets / widths / signedness equal the C compiler's for every record. This walker loads each archive through
//! the game's own path (`lbFile_LoadInto` -> `lbArchive_InitializeDAT` -> the engine's `host_parse`), takes every
//! public root the root table types (`dat_types`, `extattr`), and walks the native graph: every scalar compared bit
//! for bit, every pointer checked against the relocation table (relocated <-> non-null) and followed, arrays to the
//! count the decode uses, bitfield bytes compared raw. Output: per-file counts and every mismatch (path + values).
//! Nothing read from the disc is written outside the scratch output the caller names.
use melee_core::gcdecode::GcDecoder;
use melee_engine::{dat_types::RootType, gcm::Gcm, Host};
use melee_runtime::dat::DatArchive;
use melee_runtime::tu::{archive::HSD_ArchiveGetPublicAddress, lbarchive, lbfile};
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
pub enum Ty {
    Prim { w: u8, signed: bool, float: bool },
    Usize,
    Fn,
    Ptr(usize),
    Arr(usize, u32),
    Rec(usize),
    Void,
}
#[allow(dead_code)]
pub struct Fd {
    pub name: &'static str,
    pub noff: usize,
    pub goff: u32,
    pub ty: usize,
    pub count: Option<fn(&mut GcDecoder<'_>, u32) -> usize>,
    pub until: Option<u32>,
    pub decoded: bool,
}
#[allow(dead_code)]
pub struct Rd {
    pub name: &'static str,
    pub gsize: u32,
    pub nsize: usize,
    pub union: bool,
    pub fields: &'static [Fd],
}

mod table {
    use super::{Fd, Rd, Ty};
    include!(env!("LAYOUT_WALK_TABLE"));
}
use table::{ALIASES, RECS, TYS};

thread_local! {
    /// Every record type the walk compared (the layout-audit table lists them).
    static VISITED: std::cell::RefCell<std::collections::BTreeSet<usize>> = Default::default();
}

/// `LAYOUT_WATCH=Record.field`: print every value of that (4-byte) field the walk meets, and the maximum.
static WATCH: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
static WATCH_MAX: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(i64::MIN);

#[derive(Default)]
struct Stats {
    scalars: u64,
    ptrs: u64,
    followed: u64,
    bf: u64,
    unreloc_words: u64,
    void_ptrs: u64,
    swapped_blobs: u64,
    mism: u64,
}

struct W<'a> {
    data: &'a [u8],
    relocs: &'a HashSet<u32>,
    raw_base: usize,
    dec: GcDecoder<'a>,
    seen: HashSet<(usize, u32)>,
    st: Stats,
    out: Vec<String>,
    names: &'a HashMap<&'static str, usize>,
    budget: usize,
    /// The next array visited stops at this many elements (a C count the Rust type does not carry).
    limit: Option<u32>,
}

fn gstride(t: usize) -> u32 {
    match &TYS[t] {
        Ty::Prim { w, .. } => *w as u32,
        Ty::Usize | Ty::Fn | Ty::Ptr(_) | Ty::Void => 4,
        Ty::Arr(e, n) => gstride(*e) * n,
        Ty::Rec(r) => RECS[*r].gsize,
    }
}
fn nstride(t: usize) -> usize {
    match &TYS[t] {
        Ty::Prim { w, .. } => *w as usize,
        Ty::Usize | Ty::Fn | Ty::Ptr(_) | Ty::Void => 8,
        Ty::Arr(e, n) => nstride(*e) * *n as usize,
        Ty::Rec(r) => RECS[*r].nsize,
    }
}

impl<'a> W<'a> {
    fn bad(&mut self, s: String) {
        self.st.mism += 1;
        if self.out.len() < self.budget {
            self.out.push(s);
        }
    }
    fn be(&self, g: u32, w: usize) -> Option<u64> {
        let g = g as usize;
        let b = self.data.get(g..g + w)?;
        Some(b.iter().fold(0u64, |a, &x| (a << 8) | x as u64))
    }
    unsafe fn ne(n: *const u8, w: usize) -> u64 {
        let mut v = 0u64;
        for i in 0..w {
            v |= (*n.add(i) as u64) << (8 * i);
        }
        v
    }

    /// The union member the decoder fills (hand decoders choose by a tag in the parent).
    fn member_ok(&self, parent: &str, pg: u32, u: &Rd, f: &Fd) -> bool {
        let w = |o: u32| self.be(pg + o, 4).unwrap_or(0) as u32;
        let pick = match (parent, u.name) {
            ("HSD_Joint", _) => Some(if w(4) & (1 << 14) != 0 { "spline" } else if w(4) & (1 << 5) != 0 { "ptcl" } else { "dobjdesc" }),
            ("HSD_RObjDesc", _) => Some(match w(4) & 0x7000_0000 {
                0 => "exp",
                0x1000_0000 => "joint",
                0x2000_0000 => "limit",
                0x3000_0000 => "bcexp",
                0x4000_0000 => "ik_hint",
                _ => "i",
            }),
            ("HSD_PObjDesc", _) => {
                let fl = self.be(pg + 0xC, 2).unwrap_or(0) as u32;
                Some(match fl & 0x3000 { 0x1000 => "shape_set", 0x2000 => "envelope_p", _ => "joint" })
            }
            _ => None,
        };
        match pick {
            Some(p) => f.name == p,
            None => f.decoded,
        }
    }

    unsafe fn rec(&mut self, r: usize, n: *const u8, g: u32, path: &str) {
        VISITED.with(|v| v.borrow_mut().insert(r));
        let rd = &RECS[r];
        if rd.gsize as usize + g as usize > self.data.len() {
            // a record that runs past the data section: the decoder reads zeros there, as the C would read the next file's bytes
            return;
        }
        for f in rd.fields {
            if rd.union && !self.member_ok("", g, rd, f) {
                continue;
            }
            if let Some(wf) = WATCH.get() {
                if wf.0 == rd.name && wf.1 == f.name {
                    if let Some(v) = self.be(g + f.goff, 4) {
                        let m = WATCH_MAX.load(std::sync::atomic::Ordering::Relaxed);
                        if v as i64 as i32 as i64 > m { WATCH_MAX.store(v as i32 as i64, std::sync::atomic::Ordering::Relaxed); }
                        println!("  WATCH {}.{} = {} at {path}", rd.name, f.name, v as i32);
                    }
                }
            }
            let fp = format!("{path}.{}", f.name);
            self.val(f.ty, n.add(f.noff), g + f.goff, &fp, Some((r, g, f)));
        }
    }

    unsafe fn val(&mut self, t: usize, n: *const u8, g: u32, path: &str, fld: Option<(usize, u32, &Fd)>) {
        match &TYS[t] {
            Ty::Prim { w, float, .. } => {
                let w = *w as usize;
                let Some(gv) = self.be(g, w) else { return };
                let nv = Self::ne(n, w);
                self.st.scalars += 1;
                if gv != nv {
                    let s = if *float && w == 4 {
                        format!("{path} @{g:#x}: C {} vs Rust {}", f32::from_bits(gv as u32), f32::from_bits(nv as u32))
                    } else {
                        format!("{path} @{g:#x}: C {gv:#x} vs Rust {nv:#x}")
                    };
                    self.bad(s);
                }
            }
            Ty::Usize => {
                let Some(gv) = self.be(g, 4) else { return };
                let nv = Self::ne(n, 8);
                self.st.scalars += 1;
                if self.relocs.contains(&g) {
                    if nv == 0 {
                        self.bad(format!("{path} @{g:#x}: relocated key word decoded as 0"));
                    }
                } else if nv != gv && nv != (gv as u32 as i32 as i64 as u64) {
                    self.bad(format!("{path} @{g:#x}: C {gv:#x} vs Rust {nv:#x}"));
                }
            }
            Ty::Fn => {
                let Some(gv) = self.be(g, 4) else { return };
                if gv != 0 {
                    self.st.unreloc_words += 1;
                }
            }
            Ty::Void => {}
            Ty::Arr(e, cnt) => {
                let (gs, ns) = (gstride(*e), nstride(*e));
                let cnt = &match self.limit.take() { Some(l) => l.min(*cnt), None => *cnt };
                if matches!(TYS[*e], Ty::Prim { w: 1, .. }) && *cnt >= 4 {
                    // a byte blob: equal, or a hand decoder's native copy (32-bit words swapped, `fix_dynamics`-style)
                    let len = *cnt as usize;
                    let gb = match self.data.get(g as usize..g as usize + len) { Some(b) => b, None => return };
                    let nat = std::slice::from_raw_parts(n, len);
                    self.st.scalars += len as u64;
                    if gb == nat {
                        return;
                    }
                    if len % 4 == 0 && gb.chunks(4).zip(nat.chunks(4)).all(|(a, b)| a.iter().rev().eq(b.iter())) {
                        self.st.swapped_blobs += 1;
                        return;
                    }
                }
                for i in 0..*cnt {
                    self.val(*e, n.add(i as usize * ns), g + i * gs, &format!("{path}[{i}]"), None);
                }
            }
            Ty::Rec(r) => {
                let rd = &RECS[*r];
                if rd.union {
                    // which member: by the parent's tag for the hand-decoded unions, else the decoded one
                    let parent = fld.map(|(p, _, _)| RECS[p].name).unwrap_or("");
                    let pg = fld.map(|(_, pg, _)| pg).unwrap_or(g);
                    for f in rd.fields {
                        if self.member_ok(parent, pg, rd, f) {
                            self.val(f.ty, n.add(f.noff), g + f.goff, &format!("{path}.{}", f.name), Some((*r, g, f)));
                        }
                    }
                } else {
                    self.rec(*r, n, g, path);
                }
            }
            Ty::Ptr(inner) => {
                let Some(gv) = self.be(g, 4) else { return };
                let np = Self::ne(n, 8) as usize;
                self.st.ptrs += 1;
                if !self.relocs.contains(&g) {
                    if gv != 0 {
                        self.st.unreloc_words += 1; // a raw word in a pointer slot: the decoder's rule is null (or the hand decode's value)
                        if np != 0 && np != gv as usize {
                            self.bad(format!("{path} @{g:#x}: unrelocated word {gv:#x} decoded as {np:#x}"));
                        }
                    } else if np != 0 {
                        self.bad(format!("{path} @{g:#x}: null in the file, {np:#x} natively"));
                    }
                    return;
                }
                let tgt = gv as u32;
                if np == 0 {
                    if matches!(TYS[*inner], Ty::Void) {
                        self.st.void_ptrs += 1;
                    } else {
                        self.bad(format!("{path} @{g:#x}: relocated pointer (-> {tgt:#x}) decoded as null"));
                    }
                    return;
                }
                if np == self.raw_base + tgt as usize {
                    return; // points into the raw image (RAW_STREAM_TYPES, byte readers): the C's own bytes
                }
                if matches!(TYS[*inner], Ty::Void) {
                    return;
                }
                let mut cnt = 1usize;
                let mut inner = *inner;
                if let Some((r, base, f)) = fld {
                    // hand decoders that retype a pointee (dat_custom): `DynamicsDesc.data` on the disc is the template's
                    // `count` `lb_00F9_UnkDesc1Inner` records (fix_dynamics for fighters, dyn_template for stages)
                    if RECS[r].name == "DynamicsDesc" && f.name == "data" {
                        if let (Some(&ri), Some(c)) = (self.names.get("lb_00F9_UnkDesc1Inner"), self.be(base + 4, 4)) {
                            if let Some(t) = TYS.iter().position(|t| matches!(t, Ty::Rec(x) if *x == ri)) {
                                inner = t;
                                cnt = (c as usize).clamp(1, 64);
                            }
                        }
                    }
                }
                let inner = &inner;
                if let Some((r, base, f)) = fld {
                    // `ftDynamics.ftDynamicBones`: `BoneDynamicsTemplate[dynamicsNum]` (lb/types.h DAT_COUNT); the Rust type is a fixed
                    // `ArticleDynamicBones { array: [_; 10] }`, of which the C reads `dynamicsNum` entries
                    if f.name == "ftDynamicBones" {
                        self.limit = self.be(base, 4).map(|v| v as u32);
                        let _ = r;
                    }
                }
                if let Some((_, base, f)) = fld.filter(|_| cnt == 1) {
                    if let Some(c) = f.count {
                        cnt = c(&mut self.dec, base);
                    } else if let Some(word) = f.until {
                        let gs = gstride(*inner);
                        cnt = 0;
                        while let Some(v) = self.be(tgt + cnt as u32 * gs, 4) {
                            if v as u32 == word {
                                break;
                            }
                            cnt += 1;
                        }
                    }
                }
                if !self.seen.insert((*inner, tgt)) {
                    return;
                }
                self.st.followed += 1;
                let (gs, ns) = (gstride(*inner), nstride(*inner));
                for i in 0..cnt.min(1 << 16) {
                    self.val(*inner, (np as *const u8).add(i * ns), tgt + i as u32 * gs, &format!("{path}->[{i}]"), None);
                }
            }
        }
    }
}

/// lbFile_LoadInto -> lbArchive_InitializeDAT, as `lbArchive_LoadSymbols` does (tests/archive_host.rs).
unsafe fn load(name: &str) -> Option<(*mut melee_types::HSD_Archive, Vec<u8>)> {
    let h = Host::get();
    let c = std::ffi::CString::new(name).unwrap();
    let want = Gcm::entry_len(&h.disc, h.disc.entry_for_path(name))? as usize;
    let cap = (want + 31) & !31;
    let lo = h.os.init_alloc(h.os.arena_lo(), h.os.arena_hi(), 4);
    let heap = h.os.create_heap(lo, h.os.arena_hi());
    let buf = h.os.alloc_from_heap(heap, cap as u32);
    let mut size = 0usize;
    if lbfile::lbFile_LoadInto(c.as_ptr(), buf, cap, &mut size) != 1 {
        return None;
    }
    let orig = std::slice::from_raw_parts(buf as *const u8, size).to_vec();
    let rec = h.os.alloc_from_heap(heap, std::mem::size_of::<melee_types::HSD_Archive>() as u32) as *mut melee_types::HSD_Archive;
    lbarchive::lbArchive_InitializeDAT(rec, buf, size);
    h.os.destroy_heap(heap);
    Some((rec, orig))
}

fn custom_root(name: &str) -> Option<&'static str> {
    Some(match name {
        "ftdata" => "ftData",
        "itco" => "it_804D6D20_t",
        "dyntemplate" => "DynamicsDesc",
        _ => return None,
    })
}

/// True when the native struct has the GameCube layout byte for byte (same offsets, widths and size): code that
/// indexes it by GC byte offsets, or copies GC byte counts over it, is then exact.
fn identical(t: usize, memo: &mut HashMap<usize, bool>) -> bool {
    match &TYS[t] {
        Ty::Prim { .. } => true,
        Ty::Usize | Ty::Fn | Ty::Ptr(_) | Ty::Void => false,
        Ty::Arr(e, _) => identical(*e, memo),
        Ty::Rec(r) => {
            if let Some(&v) = memo.get(r) {
                return v;
            }
            memo.insert(*r, true);
            let rd = &RECS[*r];
            let v = rd.nsize == rd.gsize as usize && rd.fields.iter().all(|f| f.noff == f.goff as usize && identical(f.ty, memo));
            memo.insert(*r, v);
            v
        }
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--identical") {
        let mut memo = HashMap::new();
        for (i, r) in RECS.iter().enumerate() {
            let t = TYS.iter().position(|t| matches!(t, Ty::Rec(x) if *x == i));
            let same = t.map(|t| identical(t, &mut memo)).unwrap_or(false);
            println!("{} {} gc {:#x} native {:#x}", if same { "IDENTICAL" } else { "DIFFERS" }, r.name, r.gsize, r.nsize);
        }
        return;
    }
    let iso = std::env::var("GW_ISO_VANILLA").expect("GW_ISO_VANILLA");
    if let Ok(w) = std::env::var("LAYOUT_WATCH") {
        if let Some((a, b)) = w.split_once('.') {
            let _ = WATCH.set((a.to_string(), b.to_string()));
        }
    }
    let only: Vec<String> = std::env::args().skip(1).collect();
    let mut names: HashMap<&'static str, usize> = RECS.iter().enumerate().map(|(i, r)| (r.name, i)).collect();
    for (a, t) in ALIASES {
        if let Some(&i) = names.get(t) {
            names.insert(a, i);
        }
    }
    let files: Vec<String> = Gcm::open(&iso).unwrap().file_paths();
    let mut tot = Stats::default();
    let (mut nfiles, mut nroots, mut skipped) = (0u64, 0u64, HashMap::<String, u64>::new());
    for path in &files {
        let lower = path.to_ascii_lowercase();
        if !(lower.ends_with(".dat") || lower.ends_with(".usd")) {
            continue;
        }
        if !only.is_empty() && !only.iter().any(|o| path.contains(o.as_str())) {
            continue;
        }
        eprintln!("BEGIN {path}");
        // one host; each file re-inits the arena (load) and the previous file's images are dropped below
        if !Host::is_booted() {
            Host::boot(Gcm::open(&iso).unwrap(), 256 << 20).unwrap();
            melee_engine::install();
            melee_link::retail::install_retail(); // the fighter tables `ftdata` looks the symbol up in
        }
        Host::get().archives.clear();
        Host::get().retired.clear();
        let r = std::panic::catch_unwind(|| unsafe { load(path) });
        let Ok(Some((rec, orig))) = r else {
            println!("FILE {path}: not loaded (not an archive, or the loader stopped)");
            continue;
        };
        let Ok(dat) = DatArchive::parse(&orig) else { continue };
        let Some(img) = Host::get().archives.get(&(rec as usize)) else { continue };
        nfiles += 1;
        let relocs: HashSet<u32> = dat.relocs.iter().copied().collect();
        let data = &dat.data;
        let mut w = W {
            data,
            relocs: &relocs,
            raw_base: img.raw + 0x20,
            dec: dat.decoder(),
            seen: HashSet::new(),
            st: Stats::default(),
            out: Vec::new(),
            names: &names,
            budget: 40,
            limit: None,
        };
        for p in &dat.publics {
            let Some(ty) = melee_engine::dat_types::type_of(&p.name) else {
                *skipped.entry("not in the root table".into()).or_default() += 1;
                continue;
            };
            let cname = std::ffi::CString::new(p.name.as_str()).unwrap();
            let root = unsafe { HSD_ArchiveGetPublicAddress(rec, cname.as_ptr()) } as *const u8;
            if root.is_null() {
                continue;
            }
            let next = dat.publics.iter().map(|q| q.offset).filter(|&o| o > p.offset).min().unwrap_or(data.len() as u32);
            let (rname, arr, ptr) = match ty {
                RootType::Record(n) => (n, false, false),
                RootType::Array(n) => (n, true, false),
                RootType::PtrTo(n) => (n, false, true),
                // PlCo's `ftLoadCommonData`: slot 0 of the native table is the `ftCommonData*` (the other slots are typed by hand)
                RootType::Custom("plco") => ("ftCommonData", false, true),
                RootType::Custom(c) => match custom_root(c) {
                    Some(n) => (n, false, false),
                    None => {
                        *skipped.entry(format!("custom {c}")).or_default() += 1;
                        continue;
                    }
                },
                RootType::Raw => {
                    *skipped.entry("raw".into()).or_default() += 1;
                    continue;
                }
            };
            let Some(&ri) = names.get(rname) else {
                *skipped.entry(format!("no table record {rname}")).or_default() += 1;
                continue;
            };
            nroots += 1;
            unsafe {
                if ptr {
                    let np = *(root as *const *const u8);
                    if !np.is_null() && relocs.contains(&p.offset) {
                        let t = u32::from_be_bytes(data[p.offset as usize..p.offset as usize + 4].try_into().unwrap());
                        w.rec(ri, np, t, &p.name);
                    }
                } else if arr {
                    let n = ((next - p.offset) / RECS[ri].gsize.max(1)) as usize;
                    for i in 0..n {
                        w.rec(ri, root.add(i * RECS[ri].nsize), p.offset + i as u32 * RECS[ri].gsize, &format!("{}[{i}]", p.name));
                    }
                } else {
                    w.seen.insert((usize::MAX - ri, p.offset));
                    w.rec(ri, root, p.offset, &p.name);
                    if rname == "ftData" {
                        // the fighter's special attributes, as the character code types them (extattr.rs)
                        let fd = &*(root as *const melee_types::ftData);
                        let lower = p.name.trim_start_matches("ftData").to_lowercase();
                        if let Some(an) = extattr_record(&lower) {
                            if let (Some(&ai), true) = (names.get(an), relocs.contains(&(p.offset + 4))) {
                                let t = u32::from_be_bytes(data[p.offset as usize + 4..p.offset as usize + 8].try_into().unwrap());
                                if !fd.ext_attr.is_null() {
                                    w.rec(ai, fd.ext_attr as *const u8, t, &format!("{}.ext_attr({an})", p.name));
                                }
                            }
                        }
                    }
                }
            }
        }
        let s = &w.st;
        println!(
            "FILE {path}: scalars {} ptrs {} followed {} unrelocated-words {} void* {} swapped-blobs {} MISMATCH {}",
            s.scalars, s.ptrs, s.followed, s.unreloc_words, s.void_ptrs, s.swapped_blobs, s.mism
        );
        for m in &w.out {
            println!("  {m}");
        }
        tot.scalars += s.scalars;
        tot.ptrs += s.ptrs;
        tot.followed += s.followed;
        tot.unreloc_words += s.unreloc_words;
        tot.void_ptrs += s.void_ptrs;
        tot.mism += s.mism;
        tot.swapped_blobs += s.swapped_blobs;
        tot.bf += s.bf;
    }
    println!(
        "TOTAL files {nfiles} roots {nroots} scalars {} ptrs {} followed {} unrelocated-words {} void* {} swapped-blobs {} MISMATCH {}",
        tot.scalars, tot.ptrs, tot.followed, tot.unreloc_words, tot.void_ptrs, tot.swapped_blobs, tot.mism
    );
    if WATCH.get().is_some() {
        println!("WATCH max {}", WATCH_MAX.load(std::sync::atomic::Ordering::Relaxed));
    }
    VISITED.with(|v| {
        for &r in v.borrow().iter() {
            println!("VISITED {}", RECS[r].name);
        }
    });
    let mut sk: Vec<_> = skipped.into_iter().collect();
    sk.sort();
    for (k, v) in sk {
        println!("SKIPPED {k}: {v}");
    }
}

/// `extattr.rs`'s fighter -> record table, read back from that generated file at build time.
fn extattr_record(sym: &str) -> Option<&'static str> {
    const SRC: &str = include_str!("../../../rust/melee_engine/src/extattr.rs");
    for ln in SRC.lines() {
        let ln = ln.trim();
        if let Some(rest) = ln.strip_prefix(&format!("\"{sym}\" => d.ptr::<")) {
            return rest.split('>').next().map(|s| &*Box::leak(s.to_string().into_boxed_str()));
        }
    }
    None
}
