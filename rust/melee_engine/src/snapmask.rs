//! The statics' host-only bytes: those that hold no game crate's data, read from the image's PDB (its section
//! contributions name the object file each byte of `.data` / `.bss` came from). They are std's, the runtime
//! libraries' and the platform crates' (allocator counters, once-flags, hash seeds, driver and render-thread state):
//! host state that other threads write at any time. A rollback load leaves them alone (it copies only game bytes back:
//! pages with no game byte are skipped, mixed pages are copied span by span), and MELEE_SNAP_CHECK compares only game
//! bytes. The spans come from `<exe>.snapmask` when present (written from the PDB at package time by
//! `melee --write-snapmask`, so a packaged exe needs no PDB), else from `<exe>.pdb`; with neither there is no mask and
//! loads restore every statics byte. The sidecar carries the exe's size and is ignored when it does not match.
//! MELEE_SNAP_CHECK_NOMASK turns the mask off.
//!
//! Linux has no PDB: the spans come from the executable's own symbol table (`.symtab`; release builds keep it), each
//! `.data*` / `.bss*` symbol classed by the crate its (v0 or legacy) mangled name names. Bytes of those sections that
//! no symbol of a host crate covers count as game (restored), so an unnamed or unparsed object is never left out of a
//! load. Without this mask a Linux load rewound std's thread bookkeeping (a thread that started or ended while a base
//! was alive left a later thread start panicking in a btree assertion). The sidecar works the same way there
//! (`melee --write-snapmask` reads the exe itself, so a stripped copy can carry it). Elsewhere `None`.

/// Crates whose objects are host-only even though their names match the game prefix.
const HOST_CRATES: &[&str] = &["melee_platform", "melee_audio", "melee_gx", "melee_fptrace"];

/// The host-only bytes of the static ranges, as a rollback load and MELEE_SNAP_CHECK see them: pages with no game
/// byte are skipped whole, pages that mix game and host objects are visited only in their game spans. Indexed like
/// `ranges`; ranges before `first` are never masked.
pub struct HostMask {
    /// One bitmask per range: bit set = the page holds no game byte.
    pub skip: Vec<Vec<u64>>,
    /// Per range, sorted by page: the pages that mix game and host bytes, with their game spans (page offsets, end
    /// exclusive, sorted, disjoint).
    pub mixed: Vec<Vec<(u32, Vec<(u16, u16)>)>>,
}

impl HostMask {
    /// The game spans of a mixed page, or None for a page that is all game (or all host: see `skip`).
    pub fn spans(&self, ri: usize, pi: u32) -> Option<&[(u16, u16)]> {
        let m = self.mixed.get(ri)?;
        m.binary_search_by_key(&pi, |e| e.0).ok().map(|k| &m[k].1[..])
    }
    pub fn skipped(&self, ri: usize, pi: u32) -> bool {
        self.skip.get(ri).and_then(|v| v.get(pi as usize / 64)).is_some_and(|w| w >> (pi % 64) & 1 != 0)
    }
}

/// The game spans (RVAs), read once per process: a 100+ MB PDB is not parsed again for every snapshot base. Kept on
/// the system heap (`SysStatic`), not in a plain static a rollback load would rewind.
static GAME_SPANS: crate::arena::SysStatic<Option<Option<std::sync::Arc<Vec<(usize, usize)>>>>> = crate::arena::SysStatic::new(|| None);

#[cfg(any(windows, target_os = "linux"))]
fn game_spans_cached() -> Option<std::sync::Arc<Vec<(usize, usize)>>> {
    GAME_SPANS.with(|c| {
        c.get_or_insert_with(|| {
            let exe = std::env::current_exe().ok()?;
            let mut g = match read_sidecar(&exe) {
                Some(g) => g,
                #[cfg(windows)]
                None => game_spans(&std::fs::read(exe.with_extension("pdb")).ok()?)?,
                #[cfg(target_os = "linux")]
                None => game_spans_elf(&std::fs::read(&exe).ok()?)?,
            };
            g.sort_unstable();
            Some(std::sync::Arc::new(g))
        })
        .clone()
    })
}

#[cfg(any(windows, target_os = "linux"))]
pub fn host_only(ranges: &[(usize, usize)], first: usize) -> Option<HostMask> {
    let game = game_spans_cached()?;
    let img = image_base(ranges, first)?;
    let mut skip: Vec<Vec<u64>> = ranges.iter().map(|r| vec![0u64; (r.1 / 4096).div_ceil(64)]).collect();
    let mut mixed: Vec<Vec<(u32, Vec<(u16, u16)>)>> = ranges.iter().map(|_| Vec::new()).collect();
    let (mut host, mut nmixed, mut total) = (0usize, 0usize, 0usize);
    for (ri, &(a, len)) in ranges.iter().enumerate().skip(first) {
        let np = len / 4096;
        // per page, its game spans (page offsets)
        let mut per: Vec<Vec<(u16, u16)>> = vec![Vec::new(); np];
        for &(lo, hi) in game.iter() {
            let (lo, hi) = (img + lo, img + hi);
            if hi <= a || lo >= a + len {
                continue;
            }
            let (lo, hi) = (lo.max(a) - a, hi.min(a + len) - a);
            for pi in lo / 4096..hi.div_ceil(4096) {
                let (s, e) = (lo.max(pi * 4096) - pi * 4096, hi.min(pi * 4096 + 4096) - pi * 4096);
                per[pi].push((s as u16, e as u16));
            }
        }
        for (pi, sp) in per.iter_mut().enumerate() {
            total += 1;
            if sp.is_empty() {
                host += 1;
                skip[ri][pi / 64] |= 1 << (pi % 64);
                continue;
            }
            // merged exactly (never widened: a host object can sit in the next byte)
            sp.sort_unstable();
            let mut m: Vec<(u16, u16)> = Vec::with_capacity(sp.len());
            for &(s, e) in sp.iter() {
                match m.last_mut() {
                    Some(l) if s <= l.1 => l.1 = l.1.max(e),
                    _ => m.push((s, e)),
                }
            }
            if !(m.len() == 1 && m[0] == (0, 4096)) {
                nmixed += 1;
                mixed[ri].push((pi as u32, m));
            }
        }
    }
    if std::env::var_os("MELEE_SNAP_CHECK").is_some() {
        eprintln!("MELEE_SNAP_CHECK: host-only mask {host} of {total} statics pages, {nmixed} mixed pages ({} game contributions)", game.len());
    }
    Some(HostMask { skip, mixed })
}

const SIDECAR_MAGIC: &[u8; 8] = b"MSNPMSK1";

/// `<exe>.snapmask`: magic, the exe's byte length (u64 LE), then (start, end) RVA pairs (u32 LE) of game spans.
fn read_sidecar(exe: &std::path::Path) -> Option<Vec<(usize, usize)>> {
    let b = std::fs::read(exe.with_extension("snapmask")).ok()?;
    let len = std::fs::metadata(exe).ok()?.len();
    if b.len() < 16 || &b[..8] != SIDECAR_MAGIC || u64::from_le_bytes(b[8..16].try_into().ok()?) != len {
        return None;
    }
    let u = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap()) as usize;
    Some((16..b.len() - 7).step_by(8).map(|o| (u(o), u(o + 4))).collect())
}

/// Writes `<exe>.snapmask` from `<exe>.pdb` (Linux: from the exe's own symbol table) (the packaging step;
/// `melee --write-snapmask`). Returns the span count.
pub fn write_sidecar(exe: &std::path::Path) -> Result<usize, String> {
    #[cfg(not(target_os = "linux"))]
    let game = {
        let pdb = std::fs::read(exe.with_extension("pdb")).map_err(|e| format!("no pdb beside {}: {e}", exe.display()))?;
        game_spans(&pdb).ok_or("pdb has no readable section contributions")?
    };
    #[cfg(target_os = "linux")]
    let game = game_spans_elf(&std::fs::read(exe).map_err(|e| e.to_string())?).ok_or("the exe has no symbol table (stripped?)")?;
    let len = std::fs::metadata(exe).map_err(|e| e.to_string())?.len();
    let mut out = SIDECAR_MAGIC.to_vec();
    out.extend_from_slice(&len.to_le_bytes());
    for &(lo, hi) in &game {
        out.extend_from_slice(&(lo as u32).to_le_bytes());
        out.extend_from_slice(&(hi as u32).to_le_bytes());
    }
    std::fs::write(exe.with_extension("snapmask"), out).map_err(|e| e.to_string())?;
    Ok(game.len())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn host_only(_ranges: &[(usize, usize)], _first: usize) -> Option<HostMask> {
    None
}

#[cfg(windows)]
fn image_base(ranges: &[(usize, usize)], first: usize) -> Option<usize> {
    let _ = (ranges, first);
    extern "system" {
        fn GetModuleHandleW(n: *const u16) -> usize;
    }
    let b = unsafe { GetModuleHandleW(core::ptr::null()) };
    (b != 0).then_some(b)
}

/// The main program's load bias (`dlpi_addr`: 0 for a non-PIE exe), the base the spans are relative to.
#[cfg(target_os = "linux")]
fn image_base(ranges: &[(usize, usize)], first: usize) -> Option<usize> {
    let _ = (ranges, first);
    unsafe extern "C" fn cb(info: *mut libc::dl_phdr_info, _size: usize, out: *mut core::ffi::c_void) -> i32 {
        *(out as *mut usize) = (*info).dlpi_addr as usize;
        1 // the first object is the main program
    }
    let mut b = usize::MAX;
    unsafe { libc::dl_iterate_phdr(Some(cb), &mut b as *mut usize as *mut core::ffi::c_void) };
    (b != usize::MAX).then_some(b)
}

/// A game crate's name (the PDB's modules and the ELF's symbols are classed alike).
fn is_game_crate(krate: &str) -> bool {
    (krate.starts_with("melee") || krate.starts_with("brawl")) && !HOST_CRATES.contains(&krate)
}

/// The crate a mangled Rust symbol belongs to: v0 (`_R`: the first crate root `C` of its path, through nested paths
/// and inherent / trait impls) or legacy (`_ZN`: the first path segment, or the first path inside a `<...>` one).
/// None: not Rust, or a form not followed (a `<T as Trait>` qualified v0 path, a back-reference, punycode).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn symbol_crate(n: &[u8]) -> Option<&[u8]> {
    let num = |r: &[u8], i: &mut usize| -> Option<usize> {
        let st = *i;
        while r.get(*i).is_some_and(|c| c.is_ascii_digit()) {
            *i += 1;
        }
        std::str::from_utf8(&r[st..*i]).ok()?.parse().ok()
    };
    if let Some(r) = n.strip_prefix(b"_R") {
        let mut i = 0usize;
        // an optional encoding version (decimal) first
        while r.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        let skip_dis = |i: &mut usize| -> Option<()> {
            if r.get(*i) == Some(&b's') {
                *i += r[*i..].iter().position(|&c| c == b'_')? + 1;
            }
            Some(())
        };
        loop {
            match *r.get(i)? {
                b'N' => i += 2,
                b'M' | b'X' => {
                    i += 1;
                    skip_dis(&mut i)?;
                }
                b'C' => {
                    i += 1;
                    skip_dis(&mut i)?;
                    if r.get(i) == Some(&b'u') {
                        return None;
                    }
                    let k = num(r, &mut i)?;
                    if r.get(i) == Some(&b'_') {
                        i += 1;
                    }
                    return r.get(i..i + k);
                }
                _ => return None,
            }
        }
    }
    let r = n.strip_prefix(b"_ZN")?;
    let mut i = 0usize;
    let k = num(r, &mut i)?;
    let seg = r.get(i..i + k)?;
    let seg = seg.strip_prefix(b"_$LT$").unwrap_or(seg);
    let end = seg.iter().position(|&c| !(c.is_ascii_alphanumeric() || c == b'_')).unwrap_or(seg.len());
    Some(&seg[..end])
}

/// Linux: the game spans (link-time addresses, relative to the load bias) of the exe's `.data*` / `.bss*` sections
/// from its `.symtab`: each section less the objects of non-game crates. A Rust symbol whose crate is not followed,
/// and bytes no symbol covers, count as game (a load restores them: the safe side). None without a symbol table.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn game_spans_elf(p: &[u8]) -> Option<Vec<(usize, usize)>> {
    if p.get(..4)? != b"\x7fELF" || p.get(4) != Some(&2) || p.get(5) != Some(&1) {
        return None;
    }
    let u64_ = |o: usize| -> Option<usize> { Some(u64::from_le_bytes(p.get(o..o + 8)?.try_into().ok()?) as usize) };
    let (shoff, shentsize, shnum, shstrndx) = (u64_(0x28)?, u16_at(p, 0x3A)?, u16_at(p, 0x3C)?, u16_at(p, 0x3E)?);
    struct Sh {
        name: usize,
        ty: usize,
        flags: usize,
        addr: usize,
        off: usize,
        size: usize,
        link: usize,
        entsize: usize,
    }
    let sh = |i: usize| -> Option<Sh> {
        let o = shoff + i * shentsize;
        Some(Sh {
            name: u32_at(p, o)?,
            ty: u32_at(p, o + 4)?,
            flags: u64_(o + 8)?,
            addr: u64_(o + 0x10)?,
            off: u64_(o + 0x18)?,
            size: u64_(o + 0x20)?,
            link: u32_at(p, o + 0x28)?,
            entsize: u64_(o + 0x38)?,
        })
    };
    let secs: Vec<Sh> = (0..shnum).map(sh).collect::<Option<_>>()?;
    let cstr = |o: usize| -> &[u8] {
        let b = p.get(o..).unwrap_or(&[]);
        &b[..b.iter().position(|&c| c == 0).unwrap_or(0)]
    };
    let shstr = secs.get(shstrndx)?.off;
    // SHF_WRITE | SHF_ALLOC, not SHF_TLS; .data.rel.ro is RELRO (read-only after relocation, never a range)
    let data: Vec<bool> = secs
        .iter()
        .map(|s| {
            let nm = cstr(shstr + s.name);
            s.flags & 3 == 3 && s.flags & 0x400 == 0 && s.size > 0 && (nm.starts_with(b".data") || nm.starts_with(b".bss")) && !nm.starts_with(b".data.rel.ro")
        })
        .collect();
    let symtab = secs.iter().find(|s| s.ty == 2)?;
    let strtab = secs.get(symtab.link)?.off;
    let es = if symtab.entsize == 0 { 24 } else { symtab.entsize };
    let mut host: Vec<Vec<(usize, usize)>> = secs.iter().map(|_| Vec::new()).collect();
    for k in 0..symtab.size / es {
        let o = symtab.off + k * es;
        let (name, info, shndx, value, size) = (u32_at(p, o)?, *p.get(o + 4)?, u16_at(p, o + 6)?, u64_(o + 8)?, u64_(o + 16)?);
        // STT_TLS (6) lives elsewhere at run time; a sizeless symbol covers nothing
        if size == 0 || info & 0xF == 6 || !data.get(shndx).copied().unwrap_or(false) {
            continue;
        }
        let nm = cstr(strtab + name);
        let game = match symbol_crate(nm) {
            Some(c) => is_game_crate(std::str::from_utf8(c).unwrap_or("")),
            // not a Rust path: the C runtime's and the linker's objects (completed.0, DW.ref.*) are host
            None => nm.starts_with(b"_R") || nm.starts_with(b"_ZN"),
        };
        if !game {
            host[shndx].push((value, value + size));
        }
    }
    let mut out = Vec::new();
    for (i, s) in secs.iter().enumerate() {
        if !data[i] {
            continue;
        }
        let h = &mut host[i];
        h.sort_unstable();
        let (mut at, end) = (s.addr, s.addr + s.size);
        for &(a, b) in h.iter() {
            let (a, b) = (a.max(s.addr), b.min(end));
            if a > at {
                out.push((at, a));
            }
            at = at.max(b);
        }
        if at < end {
            out.push((at, end));
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crates_of_mangled_names() {
        let c = |n: &str| symbol_crate(n.as_bytes()).map(|c| String::from_utf8_lossy(c).into_owned());
        assert_eq!(c("_RNvNtCs7xf0lne6Yu_12melee_engine8snapshot6REAPER").as_deref(), Some("melee_engine"));
        assert_eq!(c("_RNvNtNtCseJnJ9eLYtvS_3std6thread7current2ID").as_deref(), Some("std"));
        assert_eq!(c("_RNvNvMs5_NtCs7S1zK3abc_11melee_world3foo3new3BUF").as_deref(), Some("melee_world"));
        assert_eq!(c("_ZN12melee_engine8snapshot6REAPER17h0123456789abcdefE").as_deref(), Some("melee_engine"));
        assert_eq!(c("_ZN56_$LT$melee_core..Foo$u20$as$u20$core..Bar$GT$3baz17h0123456789abcdefE").as_deref(), Some("melee_core"));
        assert_eq!(c("completed.0"), None);
        assert!(is_game_crate("melee_world") && !is_game_crate("std") && !is_game_crate("melee_audio"));
    }
}

fn u16_at(b: &[u8], o: usize) -> Option<usize> {
    Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?) as usize)
}
fn u32_at(b: &[u8], o: usize) -> Option<usize> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?) as usize)
}

/// The RVA spans (start, end) of every writable-section contribution from a game crate's object file.
fn game_spans(p: &[u8]) -> Option<Vec<(usize, usize)>> {
    if !p.starts_with(b"Microsoft C/C++ MSF 7.00") {
        return None;
    }
    let bs = u32_at(p, 32)?;
    let ndir = u32_at(p, 44)?;
    let bmap = u32_at(p, 52)?;
    let blocks = |list: &mut dyn Iterator<Item = usize>, size: usize| -> Option<Vec<u8>> {
        let mut v = Vec::with_capacity(size);
        for b in list {
            v.extend_from_slice(p.get(b * bs..(b + 1) * bs)?);
        }
        v.truncate(size);
        Some(v)
    };
    let nb = ndir.div_ceil(bs);
    let d = blocks(&mut (0..nb).map(|i| u32_at(p, bmap * bs + 4 * i).unwrap_or(0)), ndir)?;
    let ns = u32_at(&d, 0)?;
    let sizes: Vec<usize> = (0..ns).map(|i| u32_at(&d, 4 + 4 * i).map(|s| if s == 0xFFFF_FFFF { 0 } else { s })).collect::<Option<_>>()?;
    let mut o = 4 + 4 * ns;
    let mut at = Vec::with_capacity(ns);
    for &s in &sizes {
        at.push(o);
        o += 4 * s.div_ceil(bs);
    }
    let stream = |i: usize| -> Option<Vec<u8>> {
        let s = *sizes.get(i)?;
        blocks(&mut (0..s.div_ceil(bs)).map(|k| u32_at(&d, at[i] + 4 * k).unwrap_or(0)), s)
    };
    let dbi = stream(3)?;
    let modsz = u32_at(&dbi, 24)?;
    let scsz = u32_at(&dbi, 28)?;
    let (smsz, sisz, tsmsz, ecsz) = (u32_at(&dbi, 32)?, u32_at(&dbi, 36)?, u32_at(&dbi, 40)?, u32_at(&dbi, 52)?);
    // module names (ModInfo: 64 fixed bytes, module name, object name, 4-aligned)
    let mut game_mod = Vec::new();
    let mut o = 64;
    while o < 64 + modsz {
        let n0 = o + 64;
        let n1 = n0 + dbi.get(n0..)?.iter().position(|&c| c == 0)?;
        let name = String::from_utf8_lossy(&dbi[n0..n1]);
        let n2 = n1 + 1 + dbi.get(n1 + 1..)?.iter().position(|&c| c == 0)?;
        let file = name.rsplit(['\\', '/']).next().unwrap_or("");
        let krate = file.split(['-', '.']).next().unwrap_or("");
        game_mod.push(is_game_crate(krate));
        o = (n2 + 1 + 3) & !3;
    }
    // section headers (optional debug header entry 5)
    let od = 64 + modsz + scsz + smsz + sisz + tsmsz + ecsz;
    let sh = stream(u16_at(&dbi, od + 10)?)?;
    let secs: Vec<(usize, bool)> = sh.chunks_exact(40).map(|c| (u32_at(c, 12).unwrap_or(0), u32_at(c, 36).unwrap_or(0) & 0x8000_0000 != 0)).collect();
    let sc = dbi.get(64 + modsz..64 + modsz + scsz)?;
    let esz = if u32_at(sc, 0)? == 0xF12E_BA2D { 28 } else { 32 };
    let mut out = Vec::new();
    for e in sc[4..].chunks_exact(esz) {
        let (sec, off, size, mi) = (u16_at(e, 0)?, u32_at(e, 4)?, u32_at(e, 8)?, u16_at(e, 16)?);
        if let Some(&(va, w)) = secs.get(sec.wrapping_sub(1)) {
            if w && size > 0 && game_mod.get(mi).copied().unwrap_or(true) {
                out.push((va + off, va + off + size));
            }
        }
    }
    Some(out)
}
