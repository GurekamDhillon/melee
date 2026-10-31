//! Archive roots that are tables of differently typed pointers (`void**` in the C, split by index at the call
//! site). The generated `dat_types.rs` hands a `custom:<name>` symbol here. Seams only: this decodes the file, the
//! game code that reads the result is the tu port.
//!
//! An object whose size the bytes do not state (a float table, a record array) is decoded up to the next
//! offset that something points at (or the next public, or the end of the data block): `Bounds::extent`.

use core::ffi::c_void;
use melee_core::gcdecode::{GcDecode, GcDecoder, GcPointee};
use melee_types::{
    FighterPartsTable, Fighter_804D64FC_t, Fighter_804D651C_t, Fighter_804D6518_t, Fighter_804D6520_t, Fighter_804D6524_t,
    Fighter_804D653C_t, Fighter_804D6540_t, Fighter_ShakeTable_t, HSD_Joint, Vec2, ftCommonData, Article, ItemCommonData,
    it_804D6D20_t, it_804D6D40_t,
};

/// Sorted data offsets where something starts: every relocation target, every public, the end of the data.
pub struct Bounds(pub Vec<u32>, pub usize);

impl Bounds {
    /// Native address of data offset `off` in the RAW (big-endian, relocated) image of the archive.
    pub fn raw_addr(&self, off: u32) -> *mut c_void {
        (self.1 + off as usize) as *mut c_void
    }
    /// Bytes from `off` to the next boundary above it (at least 1).
    pub fn extent(&self, off: u32) -> u32 {
        match self.0.binary_search(&off) {
            Ok(i) => self.0.get(i + 1).map_or(1, |n| n - off),
            Err(i) => self.0.get(i).map_or(1, |n| n - off),
        }
        .max(1)
    }
}

/// `T[]` at `off`, sized by the next boundary.
fn arr<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut T {
    let n = (b.extent(off) / T::GC_BYTES).max(1) as usize;
    d.array::<T>(off, n)
}

/// The `T*` stored in the pointer word at `at`, as a `T[]` sized by the next boundary (null if not a pointer).
fn parr<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut c_void {
    match d.target(at) {
        Some(t) => arr::<T>(d, b, t) as *mut c_void,
        None => core::ptr::null_mut(),
    }
}

/// The table of pointers at `at` (`T**`): a leaked `[*mut T]`, each entry a `T[]` sized by the next boundary.
fn ptable<T: GcDecode>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut c_void {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut c_void> = (0..n).map(|i| parr::<T>(d, b, t + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// Pointer words that are themselves pointees (no array): `T*` where `T: GcPointee`.
fn one<T: GcPointee>(d: &mut GcDecoder<'_>, at: u32) -> *mut c_void {
    d.ptr::<T>(at) as *mut c_void
}

/// Decode custom root `name` at data offset `off`; the result is the native address of the root.
pub fn decode(name: &str, d: &mut GcDecoder<'_>, off: u32, _extent: u32, b: &Bounds, sym: &str) -> *mut c_void {
    match name {
        "plco" => plco(d, b, off),
        "itco" => itco(d, b, off),
        "ftdata" => ftdata(d, b, off, sym),
        "ptrtable_itemdata" => ptrtable::<melee_types::GroundItemData>(d, b, off),
        "ptrtable_dynmodel" => ptrtable::<melee_types::DynamicModelDesc>(d, b, off),
        "ptrtable_lightlist" => ptrtable::<melee_types::LightList>(d, b, off),
        "rawptrs" => {
            // a null terminated table of pointers to raw (big-endian) data, kept as native pointers into the raw image
            let n = (b.extent(off) / 4).max(1);
            let mut v: Vec<*mut c_void> = Vec::new();
            for i in 0..n {
                match d.target(off + 4 * i) {
                    Some(t) => v.push(b.raw_addr(t)),
                    None => {
                        v.push(core::ptr::null_mut());
                        break;
                    }
                }
            }
            Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
        }
        "efdat" => efdat(d, b, off),
        "ptclcmd" => ptcl_cmd(d, b, off),
        "ptcltex" => ptcl_tex(d, b, off),
        "lbrefdata" => {
            // lbrefract.c `static struct { u8 x0; f32* x4; }* refract_data`
            let r = melee_runtime::tu::lbrefract::RefractData { x0: d.u8_at(off), x4: parr::<f32>(d, b, off + 4) as *mut f32 };
            Box::leak(Box::new(r)) as *mut _ as *mut c_void
        }
        _ => panic!("dat_custom: no decoder for `{name}`"),
    }
}

/// `ftLoadCommonData` (PlCo.dat): the 23 pointers `Fighter_LoadCommonData` distributes to its globals.
fn plco(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let at = |i: u32| off + 4 * i;
    let mut out: Vec<*mut c_void> = vec![core::ptr::null_mut(); 23];
    out[0] = parr::<ftCommonData>(d, b, at(0));
    out[1] = ptable::<i32>(d, b, at(1)); // int**
    out[2] = parr::<f32>(d, b, at(2)); // float (*)[5]
    out[3] = parr::<f32>(d, b, at(3)); // float*
    out[4] = ptable::<FighterPartsTable>(d, b, at(4));
    out[5] = ptable::<Fighter_804D6540_t>(d, b, at(5));
    out[6] = parr::<Fighter_804D653C_t>(d, b, at(6));
    out[7] = parr::<Fighter_804D653C_t>(d, b, at(7));
    // [17] UNK_T: no type in the C either; read as null
    out[9] = ptable::<Vec2>(d, b, at(9)); // Vec2**
    out[10] = parr::<Fighter_ShakeTable_t>(d, b, at(10));
    out[11] = parr::<Fighter_ShakeTable_t>(d, b, at(11));
    out[12] = parr::<Fighter_804D6524_t>(d, b, at(12));
    out[13] = parr::<Fighter_804D6520_t>(d, b, at(13));
    out[14] = parr::<Fighter_804D651C_t>(d, b, at(14));
    out[15] = parr::<Fighter_804D6518_t>(d, b, at(15));
    out[16] = one::<HSD_Joint>(d, at(16));
    out[18] = parr::<u8>(d, b, at(18));
    out[19] = parr::<u8>(d, b, at(19));
    out[20] = one::<HSD_Joint>(d, at(20));
    out[21] = one::<melee_types::CrowdConfig>(d, at(21)); // gCrowdConfig
    {
        // Fighter_804D6534: { respawn platform HSD_Joint*, its HSD_AnimJoint* } (ftCo_800D4FF4 reads tbl[0], tbl[1])
        if let Some(t) = d.target(at(8)) {
            let j = d.ptr::<HSD_Joint>(t) as *mut c_void;
            let a = d.ptr::<melee_types::HSD_AnimJoint>(t + 4) as *mut c_void;
            out[8] = Box::leak(vec![j, a].into_boxed_slice()).as_mut_ptr() as *mut c_void;
        }
    }
    out[22] = {
        // x4..x1C are kind-indexed `T**` tables (Fighter_LoadCommonData indexes them by kind): the generated decode of a
        // `T**` field would give one entry, and ftCommonData_ExtendCpuTables reads up to Ft_Kind_Mex0 of them.
        let p = parr::<Fighter_804D64FC_t>(d, b, at(22)) as *mut Fighter_804D64FC_t;
        if let Some(t) = d.target(at(22)) {
            let tables = [4u32, 8, 0xC, 0x10, 0x14, 0x18, 0x1C].map(|f| rtable_min::<c_void>(d, b, t + f, melee_types::Ft_Kind_Mex0 as u32));
            unsafe {
                (*p).x4 = tables[0];
                (*p).x8 = tables[1];
                (*p).xC = tables[2];
                (*p).x10 = tables[3];
                (*p).x14 = tables[4];
                (*p).x18 = tables[5];
                (*p).x1C = tables[6];
            }
        }
        p as *mut c_void
    };
    // the root is a pointer to the table (`void** pData` after `lbArchive_LoadSymbols`)
    Box::leak(out.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// A table of `T*` (`T**`), one record per entry (null entry = null pointer), sized by the next boundary.
fn rtable<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut T {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut T> = (0..n).map(|i| d.ptr::<T>(t + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}


/// Article tables whose `x4_specialAttributes` blocks still point into the big-endian image: (table, entries, image
/// base, boundaries). Filled while decoding (an object's fields exist only after the decoder ran), applied by
/// [`fix_articles`] right after `finish()`.
static mut ARTICLE_TABLES: Vec<(usize, usize, usize, Vec<u32>)> = Vec::new();

/// `rtable::<Article>`; see [`fix_articles`].
fn rtable_articles(d: &mut GcDecoder<'_>, b: &Bounds, at: u32) -> *mut *mut melee_types::Article {
    let tbl = rtable::<melee_types::Article>(d, b, at);
    if let Some(t) = d.target(at) {
        let n = (b.extent(t) / 4).max(1) as usize;
        unsafe { (*core::ptr::addr_of_mut!(ARTICLE_TABLES)).push((tbl as usize, n, b.1, b.0.clone())) };
    }
    tbl
}

/// The C reads an article's special attributes as `s32` / `f32` fields of the kind's attribute struct, but the pointer
/// is a `void*` into the big-endian image. Each block (up to the next boundary) is copied with its 32-bit words
/// swapped; articles sharing a block share the copy.
pub fn fix_articles() {
    use std::collections::HashMap;
    let tables = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(ARTICLE_TABLES)) };
    for (tbl, n, base, bounds) in tables {
        let b = Bounds(bounds, base);
        let end = base + b.0.last().copied().unwrap_or(0) as usize;
        let tbl = tbl as *mut *mut melee_types::Article;
        let mut done: HashMap<usize, usize> = HashMap::new();
        for i in 0..n {
            unsafe {
                let a = *tbl.add(i);
                if a.is_null() || (*a).x4_specialAttributes.is_null() {
                    continue;
                }
                let raw = (*a).x4_specialAttributes as usize;
                if raw < base || raw >= end {
                    continue; // already rebuilt (a shared article)
                }
                let copy = *done.entry(raw).or_insert_with(|| {
                    let off = (raw - base) as u32;
                    let len = (b.extent(off) as usize + 3) / 4 * 4;
                    let src = core::slice::from_raw_parts(raw as *const u8, len);
                    let mut v: Vec<u8> = Vec::with_capacity(len);
                    for w in src.chunks(4) {
                        let mut x = [0u8; 4];
                        x[..w.len()].copy_from_slice(w);
                        v.extend_from_slice(&u32::from_be_bytes(x).to_ne_bytes());
                    }
                    Box::leak(v.into_boxed_slice()).as_ptr() as usize
                });
                (*a).x4_specialAttributes = copy as *mut c_void;
            }
        }
    }
}

/// `rtable` with at least `min` entries (missing ones are null), for tables the C indexes by a count the bytes do not state.
fn rtable_min<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, at: u32, min: u32) -> *mut *mut T {
    let Some(t) = d.target(at) else { return core::ptr::null_mut() };
    let n = (b.extent(t) / 4).max(1);
    let v: Vec<*mut T> = (0..n.max(min))
        .map(|i| if i < n { d.ptr::<T>(t + 4 * i) } else { core::ptr::null_mut() })
        .collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// `itPublicData` (ItCo.dat / ItCoUS.dat): `it_804D6D20_t`, whose `Article**` tables run to the next boundary.
fn itco(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let at = |i: u32| off + 4 * i;
    let mut r = it_804D6D20_t::zeroed();
    r.x0 = d.ptr::<ItemCommonData>(at(0));
    r.x4 = rtable_articles(d, b, at(1));
    r.x8 = rtable_articles(d, b, at(2));
    r.xC = rtable_articles(d, b, at(3));
    r.x10 = d.ptr::<it_804D6D40_t>(at(4));
    r.x14 = parr::<Fighter_804D653C_t>(d, b, at(5)) as *mut Fighter_804D653C_t;
    Box::leak(Box::new(r)) as *mut _ as *mut c_void
}

/// A root that is a table of `T*` (C `T**`), one record per entry, running to the next boundary. The root is the
/// table itself (`HSD_ArchiveGetPublicAddress` returns the address of the first slot).
fn ptrtable<T: GcPointee>(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    let n = (b.extent(off) / 4).max(1);
    let v: Vec<*mut T> = (0..n).map(|i| d.ptr::<T>(off + 4 * i)).collect();
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
}

/// A fighter's public data (`ftDataCaptain`, ...): the record plus the per-animation arrays whose length the C takes
/// from the retail tables (`ftData_Table_Unk0` rows for `xC` / `x10`, `ftData_UnkIntPairs` rows for `x14` / `x18`).
fn ftdata(d: &mut GcDecoder<'_>, b: &Bounds, off: u32, sym: &str) -> *mut c_void {
    use melee_core::gc::GcLayout;
    use melee_fighter::tu::ftdata as fd;
    use melee_types::{ftData, Fighter_WaitAnimData};
    let kind = unsafe {
        (0..fd::ftData_803C1F40.len())
            .find(|&k| {
                let b = fd::ftData_803C1F40[k].b;
                !b.is_null() && core::ffi::CStr::from_ptr(b).to_bytes() == sym.as_bytes()
            })
            .unwrap_or_else(|| panic!("dat_custom: `{sym}` is not a fighter data symbol of ftData_803C1F40"))
    };
    let (n, m) = unsafe { (fd::ftData_Table_Unk0[kind].count as usize, fd::ftData_UnkIntPairs[kind].count as usize) };
    let at = |f: &str| off + ftData::gc_field(f).expect("ftData field").offset;
    let mut v = <ftData as GcDecode>::gc_decode(d, off);
    v.xC = d.ptr_array::<Fighter_WaitAnimData>(at("xC"), n);
    v.x10 = d.ptr_array::<[u8; 2]>(at("x10"), n);
    v.x14 = d.ptr_array::<Fighter_WaitAnimData>(at("x14"), m);
    v.x18 = d.ptr_array::<[u8; 2]>(at("x18"), m);
    v.x48_items = rtable_articles(d, b, at("x48_items")) as *mut *mut c_void;
    let lower = sym.trim_start_matches("ftData").to_lowercase();
    if let Some(p) = crate::extattr::decode(&lower, d, at("ext_attr")) {
        v.ext_attr = p;
    }
    Box::leak(Box::new(v)) as *mut ftData as *mut c_void
}

// ---- particle banks (baselib/particle.c: psInitDataBankLocate / psInitDataBankLoad) ----------------------------
//
// The cmd and tex banks are blobs of GameCube-layout records addressed by 32-bit offsets relative to the bank start
// (NOT archive relocations: `psInitDataBankLocate` adds the bank address to them). The native `tu::particle` reads
// pointer-sized slots and native records, so each bank is rebuilt into one native image whose slots hold byte
// offsets relative to the image start, which `psInitDataBankLocate` then relocates exactly as the C does. What
// stays raw: the command STREAM of each record (opcodes and big-endian operands, read byte-wise;
// `particle::CMD_STREAM_BIG_ENDIAN`) and the texture/palette image bytes (pointers into the raw archive image).

use std::collections::HashMap;

static mut BANKS: Option<HashMap<(usize, u32), usize>> = None;

#[allow(static_mut_refs)]
fn memo(key: (usize, u32), make: impl FnOnce() -> *mut c_void) -> *mut c_void {
    let m = unsafe { BANKS.get_or_insert_with(HashMap::new) };
    if let Some(&p) = m.get(&key) {
        return p as *mut c_void;
    }
    let p = make();
    unsafe { BANKS.as_mut().unwrap().insert(key, p as usize) };
    p
}

/// A zeroed 8-aligned native buffer of `n` bytes that lives for the rest of the process.
fn native_buf(n: usize) -> *mut u8 {
    Box::leak(vec![0u64; (n + 7) / 8].into_boxed_slice()).as_mut_ptr() as *mut u8
}

/// The particle command bank starting at data offset `off`: `{ version, count.., offsets }` + `HSD_PSCmdList` records.
pub fn ptcl_cmd(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    use melee_types::HSD_PSCmdList;
    memo((b.1, off), || {
        let version = d.u16_at(off);
        // (slot count of the header, index of the first record slot, record count)
        let (hdr, first, count) = match version {
            0 => (2usize, 2usize, d.u32_at(off + 4) as usize),
            // versions 0x40..0x43: `num` is the index base (generator ids run from `num`; the C points the list `3 - num` words
            // before the table), `count` the number of records
            0x40..=0x43 => (3, 3, d.u32_at(off + 8) as usize),
            v => panic!("particle cmd bank {off:#x}: unknown version {v:#x}"),
        };
        let slots = hdr + count;
        let rel: Vec<u32> = (0..count).map(|i| d.u32_at(off + 4 * (first + i) as u32)).collect();
        let mut starts: Vec<u32> = rel.iter().copied().filter(|&r| r != 0).collect();
        starts.sort_unstable();
        starts.dedup();
        let end_of_bank = off + b.extent(off);
        const HDR: usize = 0x3C; // HSD_PSCmdList up to `cmdList`
        let rec_len = |r: u32| -> usize {
            let at = off + r;
            let stop = match starts.binary_search(&r) {
                Ok(i) => starts.get(i + 1).map_or(end_of_bank, |n| off + n),
                Err(_) => end_of_bank,
            };
            (stop.max(at + HDR as u32) - at) as usize
        };
        let total = slots * 8 + starts.iter().map(|&r| (rec_len(r) + 7) & !7).sum::<usize>();
        let img = native_buf(total);
        let w = |at: usize, v: isize| unsafe { (img.add(at) as *mut isize).write(v) };
        w(0, version as isize);
        if version == 0 {
            w(8, count as isize);
        } else {
            w(8, d.u32_at(off + 4) as isize);
            w(16, count as isize);
        }
        let mut cur = slots * 8;
        let mut placed: HashMap<u32, usize> = HashMap::new();
        for &r in &starts {
            let len = rec_len(r);
            let rec = <HSD_PSCmdList as GcDecode>::gc_decode(d, off + r);
            unsafe {
                core::ptr::copy_nonoverlapping(&rec as *const HSD_PSCmdList as *const u8, img.add(cur), HDR);
                for k in HDR..len {
                    *img.add(cur + k) = d.u8_at(off + r + k as u32);
                }
            }
            placed.insert(r, cur);
            cur += (len + 7) & !7;
        }
        for (i, &r) in rel.iter().enumerate() {
            w(8 * (first + i), if r == 0 { 0 } else { placed[&r] as isize });
        }
        img as *mut c_void
    })
}

/// The particle texture bank at data offset `off`: `{ group count, group offsets }` + `HSD_PSTexGroup` records whose
/// texture / palette slots become offsets from the image start to the (raw, big-endian) image bytes.
pub fn ptcl_tex(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    memo((b.1, off), || {
        let n = d.u32_at(off) as usize;
        let rel: Vec<u32> = (0..n).map(|i| d.u32_at(off + 4 + 4 * i as u32)).collect();
        // slot count of group record `g`: `num`, plus the palette entries `psInitDataBankLocate` relocates
        let slots_of = |d: &mut GcDecoder<'_>, g: u32| -> usize {
            let num = d.u32_at(off + g);
            let fmt = d.u32_at(off + g + 4);
            let palnum = d.u16_at(off + g + 0x14) as u32;
            let palflag = d.u16_at(off + g + 0x16) as u32;
            let end = if fmt != 8 && fmt.wrapping_sub(9) > 1 {
                num
            } else if palflag & 1 != 0 {
                num + 1
            } else if palnum != 0 {
                num + palnum
            } else {
                num.wrapping_mul(2)
            };
            end as usize
        };
        let sizes: Vec<usize> = rel.iter().map(|&g| if g == 0 { 0 } else { 24 + 8 * slots_of(d, g).max(1) }).collect();
        let total = 8 + 8 * n + sizes.iter().sum::<usize>();
        let img = native_buf(total);
        let base = img as usize;
        unsafe { (img as *mut isize).write(n as isize) };
        let mut cur = 8 + 8 * n;
        for (i, &g) in rel.iter().enumerate() {
            if g == 0 {
                continue;
            }
            unsafe { (img.add(8 + 8 * i) as *mut isize).write(cur as isize) };
            let slots = slots_of(d, g);
            let at = off + g;
            unsafe {
                for (o, src) in [(0usize, 0u32), (4, 4), (8, 8), (12, 12), (16, 16)] {
                    (img.add(cur + o) as *mut u32).write(d.u32_at(at + src));
                }
                (img.add(cur + 20) as *mut u16).write(d.u16_at(at + 0x14));
                (img.add(cur + 22) as *mut u16).write(d.u16_at(at + 0x16));
                for k in 0..slots {
                    let t = d.u32_at(at + 0x18 + 4 * k as u32);
                    // `psInitDataBankLocate` adds the image base: store (raw image address - base)
                    let v: isize = if t == 0 { 0 } else { (b.raw_addr(off + t) as usize).wrapping_sub(base) as isize };
                    (img.add(cur + 24 + 8 * k) as *mut isize).write(v);
                }
            }
            cur += sizes[i];
        }
        img as *mut c_void
    })
}

/// An effect bank root (`effCommonDataTable`, `effFoxDataTable`, ...): C `struct { cmd bank*, tex bank*, EF_EffectDesc descs[] }`
/// (`EF_DAT_Entry` for the first two words; `efAsync_LoadSync` keeps `&entry->data` as the `EF_EffectDesc` table). Native:
/// two converted bank pointers, then the native `EF_EffectDesc` array, contiguous from byte 16 (= `offsetof(EF_DAT_Entry, data)`).
fn efdat(d: &mut GcDecoder<'_>, b: &Bounds, off: u32) -> *mut c_void {
    use melee_types::EF_EffectDesc;
    let cmd = d.target(off).map_or(core::ptr::null_mut(), |t| ptcl_cmd(d, b, t));
    let tex = d.target(off + 4).map_or(core::ptr::null_mut(), |t| ptcl_tex(d, b, t));
    let n = (b.extent(off + 8) / <EF_EffectDesc as GcDecode>::GC_BYTES) as usize;
    let sz = core::mem::size_of::<EF_EffectDesc>();
    let img = native_buf(16 + n.max(1) * sz);
    unsafe {
        (img as *mut *mut c_void).write(cmd);
        (img.add(8) as *mut *mut c_void).write(tex);
        if n > 0 {
            let descs = d.array::<EF_EffectDesc>(off + 8, n);
            core::ptr::copy_nonoverlapping(descs as *const u8, img.add(16), n * sz);
        }
    }
    img as *mut c_void
}
