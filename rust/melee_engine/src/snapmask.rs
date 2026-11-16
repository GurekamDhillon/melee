//! MELEE_SNAP_CHECK's host-only page mask: the statics pages that hold no game crate's data, read from the image's
//! PDB (its section contributions name the object file each byte of `.data` / `.bss` came from). Those pages are
//! std's, the runtime libraries' and the platform crates' (allocator counters, once-flags, driver state): host state
//! a rollback load leaves alone by design, so the check skips them. Every page with any byte of a `melee*` / game crate
//! is still compared. Dev builds only (a shipped exe has no PDB beside it; the mask is then empty and nothing is
//! skipped). Windows; elsewhere `None`.

/// Crates whose objects are host-only even though their names match the game prefix.
const HOST_CRATES: &[&str] = &["melee_platform", "melee_audio", "melee_gx", "melee_fptrace"];

/// One bitmask per range (bit set = skip that page), aligned with `ranges`; ranges before `first` are never masked.
#[cfg(windows)]
pub fn host_only(ranges: &[(usize, usize)], first: usize) -> Option<Vec<Vec<u64>>> {
    let exe = std::env::current_exe().ok()?;
    let pdb = std::fs::read(exe.with_extension("pdb")).ok()?;
    let img = image_base(ranges, first)?;
    let game = game_spans(&pdb)?;
    let mut out: Vec<Vec<u64>> = ranges.iter().map(|r| vec![0u64; (r.1 / 4096).div_ceil(64)]).collect();
    let (mut host, mut total) = (0usize, 0usize);
    for (ri, &(a, len)) in ranges.iter().enumerate().skip(first) {
        let np = len / 4096;
        let mut keep = vec![false; np];
        for &(lo, hi) in &game {
            let (lo, hi) = (img + lo, img + hi);
            if hi <= a || lo >= a + len {
                continue;
            }
            let p0 = (lo.max(a) - a) / 4096;
            let p1 = (hi.min(a + len) - a).div_ceil(4096);
            keep[p0..p1].iter_mut().for_each(|k| *k = true);
        }
        for (pi, k) in keep.iter().enumerate() {
            total += 1;
            if !k {
                host += 1;
                out[ri][pi / 64] |= 1 << (pi % 64);
            }
        }
    }
    eprintln!("MELEE_SNAP_CHECK: host-only mask {host} of {total} statics pages ({} game contributions)", game.len());
    Some(out)
}

#[cfg(not(windows))]
pub fn host_only(_ranges: &[(usize, usize)], _first: usize) -> Option<Vec<Vec<u64>>> {
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
        game_mod.push((krate.starts_with("melee") || krate.starts_with("brawl")) && !HOST_CRATES.contains(&krate));
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
