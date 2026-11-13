//! The m-ex slot runtime on the host: the native form of the C build's `pc/platform/gw_mex_ftfunction_runtime.c`
//! (mexData readers, fighter slots, costumes, sound banks, music, effects, Kirby copy rows), `gw_mex_grfunction.c`'s
//! stage table readers and `gw_mex_graudio.c`.
//!
//! An m-ex disc (ACE, Akaneia) ships its content tables as `MxDt.dat`, an HSD archive whose one public is `mexData`.
//! The C copies the data section into guest memory, relocates it and reads it big-endian with `gw_r32`; here the
//! section goes into a block below 4 GiB (`lowmem`) and is relocated the same way, so every pointer word in it is a
//! 32-bit address of the block and the C readers translate one for one (`r32` = `gw_r32`). Strings handed to the game
//! point into the block (bytes have no byte order); records the game reads as structs (`Fighter_DemoStrings`,
//! `Fighter_CostumeStrings`) are decoded into native copies once.
//!
//! Callback rows (`fighter_function`, `kirby_function`) hold vanilla guest addresses. The C resolves them through the
//! generated bridge (guest address -> native function). The retail rows (internal 0..26) of the same table hold the
//! vanilla fighters' own addresses, so an address is resolved here to the native entry of the retail row (and slot)
//! that carries it: `ftData_MexRetailSlot` / the Kirby tables. An address no retail row carries is left empty, as the
//! C does for an unbridged one (the clone base's callback then runs), and logged.
//!
//! Guest code in a fighter's `ftFunction` (PPC overrides) is not run: the three installed fighters (Halberd, Ultimate
//! Kirby, Sora) carry none the C dispatches (Halberd's one override, slot 45 GetTrailData, is not dispatched by the C
//! either). A fighter whose file does carry dispatched overrides is logged at install.
//!
//! Nothing loaded is the C's "no mexData" state: every reader answers as on a vanilla disc.
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_void};
use std::ffi::CStr;

const OFF_FIGHTER: u32 = 0x08;
const OFF_FTFUNC: u32 = 0x0C;
const OFF_SSM: u32 = 0x10;
const OFF_MUSIC: u32 = 0x14;
const OFF_EFFECT: u32 = 0x18;
const OFF_ITEM: u32 = 0x1C;
const OFF_KIRBY_DATA: u32 = 0x20;
const OFF_KIRBY_FUNC: u32 = 0x24;
const OFF_MAP: u32 = 0x28;
/// metadata stage counts and MexData.grFunction (gw_mex_grfunction.c GW_MEXDT_*).
const META_INT_STAGES: u32 = 4 + 3 * 4;
const META_EXT_STAGES: u32 = 4 + 4 * 4;
const OFF_GRDESC: u32 = 0x2C;
const GR_STAGEIDS_STRIDE: u32 = 0x0C;
const GR_SLOT_COUNT: u32 = 13;
const GR_SLOT_FILE: u32 = 2;
const GR_SLOT_FLAGS2: u32 = 10;
const GR_FIRST_NEW: i32 = 71;
const GR_MAX: i32 = 160;
const GR_EXT_MAX: i32 = 384;

/// `GW_MEX_SLOTS` (gw.h GW_MEX_SLOT_COUNT): port fighter kinds Ft_Kind_Mex0 (0x21) .. +94.
const MEX_SLOTS: i32 = 94;
const FT_MEX0: i32 = 0x21;
/// `GW_PORT_CK_MEX0`: CharacterKind 0x22 + slot.
const CK_MEX0: i32 = 0x22;
const FIRST_NEW: i32 = 27;
const CSS_ICON_MAX: i32 = 128;
const EFFECT_BANK_MAX: u32 = 50;
const KIRBY_FUNC_SLOTS: i32 = 9;

pub struct Mex {
    base: u32,
    size: u32,
    root: u32,
    slots: Vec<i32>,
    /// `gw_Mex_FtDemoStrings` as native records, by internal id.
    demo: Vec<*mut melee_types::Fighter_DemoStrings>,
    kirby_costumes: Vec<*mut c_void>,
    /// `gw_Mex_CssIconTable` as native records (mexData.menu.css +0xDC, 0x1C a row; `char_kind` is the m-ex EXTERNAL id).
    css: *mut melee_types::CSSIcon,
    /// stage table state (gr_tables)
    gr_int: i32,
    gr_ext: i32,
    gr_map: u32,
    gr_desc: u32,
}

static mut MEX: Option<Mex> = None;

fn m() -> Option<&'static Mex> {
    unsafe { MEX.as_ref() }
}

impl Mex {
    fn inr(&self, a: u32, len: u32) -> bool {
        a >= self.base && (a as u64) + len as u64 <= self.base as u64 + self.size as u64
    }
    fn r32(&self, a: u32) -> u32 {
        if self.inr(a, 4) { unsafe { u32::from_be_bytes(*(a as usize as *const [u8; 4])) } } else { 0 }
    }
    fn r16(&self, a: u32) -> u32 {
        if self.inr(a, 2) { unsafe { u16::from_be_bytes(*(a as usize as *const [u8; 2])) as u32 } } else { 0 }
    }
    fn r8(&self, a: u32) -> Option<u8> {
        if self.inr(a, 1) { Some(unsafe { *(a as usize as *const u8) }) } else { None }
    }
    /// `gw_mex_word(tbl, index, stride)`.
    fn word(&self, tbl: u32, index: u32, stride: u32) -> u32 {
        if tbl == 0 { 0 } else { self.r32(tbl.wrapping_add(index.wrapping_mul(stride))) }
    }
    fn cstr(&self, p: u32) -> *const c_char {
        if p != 0 && self.inr(p, 1) { p as usize as *const c_char } else { core::ptr::null() }
    }
    fn meta(&self) -> u32 {
        self.r32(self.root)
    }
    fn ftfield(&self, off: u32) -> u32 {
        let f = self.r32(self.root + OFF_FIGHTER);
        self.r32(f + off)
    }
    fn kirbyfield(&self, off: u32) -> u32 {
        let k = self.r32(self.root + OFF_KIRBY_DATA);
        self.r32(k + off)
    }
    /// metadata +0x0C, the CSS icon count (the C's gate for everything m-ex: `gw_Mex_CssIconCount`).
    fn css_icons(&self) -> i32 {
        let md = self.meta();
        if !self.inr(md, 0x10) {
            return 0;
        }
        let n = self.r32(md + 0x0C) as i32;
        if n > CSS_ICON_MAX { 0 } else { n.max(0) }
    }
    fn internal_count(&self) -> i32 {
        let md = self.meta();
        if self.inr(md, 8) { self.r32(md + 4) as i32 } else { 0 }
    }
}

fn file_exists(name: &CStr) -> bool {
    crate::host::Host::is_booted() && crate::host::Host::get().disc.entry_for_path(&name.to_string_lossy()) >= 0
}

fn str_of(p: *const c_char) -> String {
    if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
}

/// Load `MxDt.dat` from the mounted disc (and mods), relocate it, build the slot table. Returns false on a vanilla disc.
pub fn load() -> bool {
    unsafe { MEX = None };
    let disc = &mut crate::host::Host::get().disc;
    let Ok(dat) = disc.read("MxDt.dat") else { return false };
    let be = |o: usize| u32::from_be_bytes(dat[o..o + 4].try_into().unwrap());
    if dat.len() < 0x20 {
        return false;
    }
    let (file_size, data_size, nb_reloc, nb_pub) = (be(0) as usize, be(4) as usize, be(8) as usize, be(12) as usize);
    if file_size != dat.len() || 0x20 + data_size + nb_reloc * 4 > dat.len() {
        eprintln!("mexdata: MxDt.dat header inconsistent");
        return false;
    }
    // the public symbol `mexData`
    let pubs = 0x20 + data_size + nb_reloc * 4;
    let strs = pubs + (nb_pub + be(16) as usize) * 8;
    let mut sym = None;
    for i in 0..nb_pub {
        let (off, name) = (be(pubs + i * 8), be(pubs + i * 8 + 4) as usize);
        let s = &dat[strs + name..];
        let e = s.iter().position(|&c| c == 0).unwrap_or(0);
        if &s[..e] == b"mexData" {
            sym = Some(off);
        }
    }
    let Some(sym) = sym else {
        eprintln!("mexdata: MxDt.dat has no public symbol mexData");
        return false;
    };
    let block = match crate::lowmem::alloc_low(data_size) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("mexdata: {e}");
            return false;
        }
    };
    let base = block.addr32();
    let img = unsafe { core::slice::from_raw_parts_mut(block.base, data_size) };
    img.copy_from_slice(&dat[0x20..0x20 + data_size]);
    for i in 0..nb_reloc {
        let off = be(0x20 + data_size + i * 4) as usize;
        if off + 4 > data_size {
            continue;
        }
        let w = u32::from_be_bytes(img[off..off + 4].try_into().unwrap()).wrapping_add(base);
        img[off..off + 4].copy_from_slice(&w.to_be_bytes());
    }
    let mut mx = Mex { base, size: data_size as u32, root: base + sym, slots: Vec::new(), demo: Vec::new(), kirby_costumes: Vec::new(), css: core::ptr::null_mut(), gr_int: 0, gr_ext: 0, gr_map: 0, gr_desc: 0 };
    let fighter = mx.r32(mx.root + OFF_FIGHTER);
    let item = mx.r32(mx.root + OFF_ITEM);
    if !mx.inr(fighter, 0x50) || !mx.inr(item, 0x18) {
        eprintln!("mexdata: MxDt.dat loaded but its layout does not match - leaving mexData unloaded");
        return false;
    }
    eprintln!("mexdata: MxDt.dat loaded, mexData @ {:#010X} (data {base:#010X}, {data_size} bytes)", mx.root);
    if mx.css_icons() == 0 {
        // the C's fighter / sound / effect readers all gate on the CSS icon count
        eprintln!("mexdata: the CSS lists no icons (or more than the port holds) - m-ex content is OFF");
        return false;
    }
    // gw_mex_slots_build
    let n = mx.internal_count();
    let mut k = FIRST_NEW;
    while n > 6 && k < n - 6 {
        let pl = mx.cstr(mx.word(mx.ftfield(0x04), k as u32, 8));
        if !pl.is_null() && unsafe { *pl } != 0 {
            if !file_exists(unsafe { CStr::from_ptr(pl) }) {
                eprintln!("mexdata: m-ex fighter {k}'s {} is not on the disc or in a mod - skipped", str_of(pl));
            } else if mx.slots.len() as i32 == MEX_SLOTS {
                eprintln!("mexdata: ERROR no m-ex fighter slot left (the port holds {MEX_SLOTS}): m-ex fighter {k} ({}) is left out", str_of(pl));
            } else {
                mx.slots.push(k);
            }
        }
        k += 1;
    }
    eprintln!("mexdata: {} m-ex fighter slots (of {MEX_SLOTS})", mx.slots.len());
    // native copies of the struct rows the game reads
    let n = n.max(0) as usize;
    for k in 0..n {
        let p = mx.word(mx.ftfield(0x18), k as u32, 4);
        mx.demo.push(if p != 0 && mx.inr(p, 16) {
            let s = |i: u32| mx.cstr(mx.r32(p + 4 * i)) as *mut i8;
            Box::leak(Box::new(melee_types::Fighter_DemoStrings { result_filename: s(0), intro_filename: s(1), ending_filename: s(2), vi_wait_filename: s(3) }))
        } else {
            core::ptr::null_mut()
        });
        let p = mx.word(mx.kirbyfield(0x08), k as u32, 4);
        mx.kirby_costumes.push(if p != 0 && mx.inr(p, 16) {
            // a Fighter_CostumeStrings[] (stride 12 in the guest), as many rows as the fighter has costumes
            let e = ext_for_internal(&mx, k as i32);
            let t = mx.ftfield(0x10);
            let cn = if e >= 0 { mx.r8(t + e as u32 * 4).unwrap_or(0) as usize } else { 0 }.max(1);
            let v: Vec<melee_types::Fighter_CostumeStrings> = (0..cn)
                .map(|c| {
                    let r = p + 12 * c as u32;
                    melee_types::Fighter_CostumeStrings { dat_filename: mx.cstr(mx.r32(r)) as *mut i8, joint_name: mx.cstr(mx.r32(r + 4)) as *mut i8, matanim_joint_name: mx.cstr(mx.r32(r + 8)) as *mut i8 }
                })
                .collect();
            Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut c_void
        } else {
            core::ptr::null_mut()
        });
    }
    gr_tables(&mut mx);
    css_table(&mut mx);
    unsafe { MEX = Some(mx) };
    true
}

/// The character select's icon rows as native records: the C hands the game the guest rows themselves (the same
/// layout as the port's `CSSIcon`); here the big-endian words are read once. One spare row past the end.
fn css_table(mx: &mut Mex) {
    let n = mx.css_icons() as u32;
    let menu = mx.r32(mx.root + 0x04);
    let css = if mx.inr(menu, 8) { mx.r32(menu + 0x04) } else { 0 };
    let tbl = css.wrapping_add(0xDC);
    if n == 0 || css == 0 || !mx.inr(tbl, n * 0x1C) {
        return;
    }
    let f = |a: u32| f32::from_bits(mx.r32(a));
    let mut v: Vec<melee_types::CSSIcon> = (0..n)
        .map(|i| {
            let r = tbl + i * 0x1C;
            let b = |o: u32| mx.r8(r + o).unwrap_or(0);
            melee_types::CSSIcon {
                ft_hudindex: b(0),
                char_kind: b(1),
                state: b(2),
                anim_timer: b(3),
                joint_id_vs: b(4),
                joint_id_1p: b(5),
                sfx: mx.r32(r + 0x08) as i32,
                bound_l: f(r + 0x0C),
                bound_r: f(r + 0x10),
                bound_u: f(r + 0x14),
                bound_d: f(r + 0x18),
            }
        })
        .collect();
    v.push(melee_types::CSSIcon::zeroed());
    mx.css = Box::leak(v.into_boxed_slice()).as_mut_ptr();
}

fn gr_tables(mx: &mut Mex) {
    let meta = mx.meta();
    let map = mx.r32(mx.root + OFF_MAP);
    let desc = mx.r32(mx.root + OFF_GRDESC);
    if !mx.inr(meta, 0x3C) || !mx.inr(map, 0x18) || !mx.inr(desc, 4) {
        eprintln!("grfunction: mexData has no usable stage tables - m-ex stages stay off");
        return;
    }
    let (ni, ne) = (mx.r32(meta + META_INT_STAGES) as i32, mx.r32(meta + META_EXT_STAGES) as i32);
    if ni <= 0 || ni > GR_MAX || ne <= 0 || ne > GR_EXT_MAX {
        eprintln!("grfunction: implausible stage counts (internal {ni}, external {ne}) - m-ex stages stay off");
        return;
    }
    let ids = mx.r32(map);
    if !mx.inr(ids, ne as u32 * GR_STAGEIDS_STRIDE) {
        return;
    }
    (mx.gr_int, mx.gr_ext, mx.gr_map, mx.gr_desc) = (ni, ne, map, desc);
    eprintln!("grfunction: mexData stage tables ready: {ni} internal, {ne} external");
}

// ---- the readers (gw_Mex_*) ---------------------------------------------------------------------------------------

fn ext_for_internal(mx: &Mex, k: i32) -> i32 {
    let desc = mx.ftfield(0x0C);
    let n = mx.internal_count();
    for e in 0..n {
        if desc != 0 && mx.r8(desc + e as u32 * 3) == Some(k as u8) {
            return e;
        }
    }
    -1
}
pub fn SlotInternal(slot: i32) -> i32 {
    m().and_then(|m| m.slots.get(usize::try_from(slot).ok()?).copied()).unwrap_or(-1)
}
fn slot_of_internal(k: i32) -> i32 {
    m().and_then(|m| m.slots.iter().position(|&s| s == k)).map_or(-1, |i| i as i32)
}
fn slot_of_port(fk: i32) -> i32 {
    if (FT_MEX0..FT_MEX0 + MEX_SLOTS).contains(&fk) { fk - FT_MEX0 } else { -1 }
}
pub fn InternalCount() -> i32 {
    m().map_or(0, |m| m.internal_count())
}
pub fn InternalForPortKind(fk: i32) -> i32 {
    let n = InternalCount();
    if (0..=26).contains(&fk) {
        return fk;
    }
    if (27..=32).contains(&fk) {
        return if n > 6 { n - 6 + (fk - 27) } else { fk };
    }
    let s = slot_of_port(fk);
    if s >= 0 { SlotInternal(s) } else { -1 }
}
pub fn ExtForInternal(k: i32) -> i32 {
    m().map_or(-1, |m| ext_for_internal(m, k))
}
pub fn InternalForExt(e: i32) -> i32 {
    let Some(m) = m() else { return -1 };
    let desc = m.ftfield(0x0C);
    if e < 0 || desc == 0 {
        return -1;
    }
    m.r8(desc + e as u32 * 3).map_or(-1, |v| v as i32)
}
pub fn PortCKindToExt(ck: i32) -> i32 {
    if (0..=25).contains(&ck) {
        return ck;
    }
    if (CK_MEX0..CK_MEX0 + MEX_SLOTS).contains(&ck) {
        let k = SlotInternal(ck - CK_MEX0);
        return if k >= 0 { ExtForInternal(k) } else { -1 };
    }
    -1
}
pub fn ExtToPortCKind(ext: i32) -> i32 {
    if (0..=25).contains(&ext) {
        return ext;
    }
    let k = InternalForExt(ext);
    let s = if k >= 0 { slot_of_internal(k) } else { -1 };
    if s >= 0 { CK_MEX0 + s } else { -1 }
}
/// `gw_Mex_CssIconCount`: the character select's icon rows (0 without mexData).
pub fn CssIconCount() -> i32 {
    m().map_or(0, |m| if m.css.is_null() { 0 } else { m.css_icons() })
}
/// `gw_Mex_CssIconTable`: `CssIconCount()` native `CSSIcon` rows whose `char_kind` is the m-ex external id.
pub fn CssIconTable() -> *mut c_void {
    m().map_or(core::ptr::null_mut(), |m| m.css as *mut c_void)
}
pub fn FighterName(ext: i32) -> *const c_char {
    match m() {
        Some(m) if ext >= 0 => m.cstr(m.word(m.ftfield(0x00), ext as u32, 4)),
        _ => core::ptr::null(),
    }
}
pub fn FtPlFile(k: i32) -> *mut c_char {
    m().map_or(core::ptr::null(), |m| m.cstr(m.word(m.ftfield(0x04), k as u32, 8))) as *mut c_char
}
pub fn FtPlSymbol(k: i32) -> *mut c_char {
    m().map_or(core::ptr::null(), |m| {
        let t = m.ftfield(0x04);
        if t == 0 { core::ptr::null() } else { m.cstr(m.word(t + 4, k as u32, 8)) }
    }) as *mut c_char
}
pub fn FtAnimFile(k: i32) -> *mut c_char {
    m().map_or(core::ptr::null(), |m| m.cstr(m.word(m.ftfield(0x1C), k as u32, 4))) as *mut c_char
}
pub fn FtAnimCount(k: i32) -> i32 {
    m().map_or(0, |m| {
        let t = m.ftfield(0x20);
        if t == 0 { 0 } else { m.word(t + 4, k as u32, 8) as i32 }
    })
}
pub fn FtEffectIndex(k: i32) -> i32 {
    m().map_or(-1, |m| {
        let t = m.ftfield(0x24);
        if t == 0 { -1 } else { m.r8(t + k as u32).map_or(-1, |v| v as i32) }
    })
}
pub fn FtCostumeCount(k: i32) -> i32 {
    m().map_or(0, |m| {
        let t = m.ftfield(0x10);
        let e = ext_for_internal(m, k);
        if e >= 0 && t != 0 { m.r8(t + e as u32 * 4).map_or(0, |v| v as i32) } else { 0 }
    })
}
pub fn FtCostumeString(k: i32, c: i32, which: i32) -> *const c_char {
    m().map_or(core::ptr::null(), |m| {
        let tbl = m.word(m.ftfield(0x14), k as u32, 4);
        if tbl == 0 { core::ptr::null() } else { m.cstr(m.word(tbl + which as u32 * 4, c as u32, 16)) }
    })
}
pub fn FtDemoStrings(k: i32) -> *mut melee_types::Fighter_DemoStrings {
    m().and_then(|m| m.demo.get(usize::try_from(k).ok()?).copied()).unwrap_or(core::ptr::null_mut())
}
pub fn CostumeInfo(ck: i32, field: i32) -> i32 {
    let Some(m) = m() else { return 0 };
    let t = m.ftfield(0x10);
    let e = PortCKindToExt(ck);
    if e < 0 || t == 0 || !(0..=3).contains(&field) || !m.inr(t + e as u32 * 4, 4) {
        return 0;
    }
    m.r8(t + e as u32 * 4 + field as u32).map_or(0, |v| v as i32)
}
pub fn CostumeVisIdx(k: i32, costume: i32) -> i32 {
    // the per-internal `costume_file[k][c].visibility_lookup_idx` (gw_Mex_CostumeVisIdx after the kind mapping)
    let Some(m) = m() else { return costume };
    let tbl = m.word(m.ftfield(0x14), k as u32, 4);
    let a = tbl + costume as u32 * 16 + 12;
    if tbl == 0 || !m.inr(a, 4) {
        return costume;
    }
    let v = m.r32(a) as i32;
    if (0..255).contains(&v) { v } else { 0 }
}
pub fn FtBaseKind(k: i32) -> i32 {
    let Some(m) = m() else { return 0 };
    let tbl = m.word(m.r32(m.root + OFF_FTFUNC), 0, 4);
    let g = m.word(tbl, k as u32, 4);
    if g == 0 {
        return 0;
    }
    (0..=26).find(|&r| m.word(tbl, r, 4) == g).map_or(0, |r| r as i32)
}
pub fn InsigniaForExt(ext: i32) -> i32 {
    let Some(m) = m() else { return -1 };
    let tbl = m.ftfield(0x08);
    if ext < 0 || tbl == 0 { -1 } else { m.r8(tbl + ext as u32).map_or(-1, |v| v as i32) }
}

/// `fighter_function[slot][k]` resolved to the native callback of the retail row that carries the same guest address.
pub fn FtFunc(slot: i32, k: i32) -> *mut c_void {
    let Some(m) = m() else { return core::ptr::null_mut() };
    let ff = m.r32(m.root + OFF_FTFUNC);
    let g = m.word(m.word(ff, slot as u32, 4), k as u32, 4);
    if g == 0 {
        return core::ptr::null_mut();
    }
    // the retail rows: internal 0..26 are port kinds 0..26, the six bosses (the last six internal ids) port kinds 27..32
    let n = m.internal_count();
    let retail: Vec<(u32, usize)> = (0..=26u32).map(|r| (r, r as usize)).chain((0..6).filter(|_| n > 6).map(|j| ((n - 6 + j) as u32, 27 + j as usize))).collect();
    for s in [slot].into_iter().chain(0..46) {
        let tbl = m.word(ff, s as u32, 4);
        if let Some(&(_, fk)) = retail.iter().find(|&&(r, _)| m.word(tbl, r, 4) == g) {
            let p = melee_fighter::tu::ftdata::ftData_MexRetailSlot(s, fk);
            if !p.is_null() {
                return p;
            }
        }
    }
    eprintln!("mexdata: fighter_function[{slot}][{k}] = {g:#010X} has no native counterpart - left empty");
    core::ptr::null_mut()
}

/// `kirby_function[slot][k]`, resolved the same way against the retail Kirby copy rows.
pub fn KirbyFunc(slot: i32, k: i32) -> *mut c_void {
    let Some(m) = m() else { return core::ptr::null_mut() };
    if k < 0 || !(0..KIRBY_FUNC_SLOTS).contains(&slot) {
        return core::ptr::null_mut();
    }
    let kf = m.r32(m.root + OFF_KIRBY_FUNC);
    let g = m.word(m.word(kf, slot as u32, 4), k as u32, 4);
    if g == 0 {
        return core::ptr::null_mut();
    }
    use melee_chara::tu::ftkirby as kb;
    for s in [slot].into_iter().chain(0..4) {
        let tbl = m.word(kf, s as u32, 4);
        if let Some(r) = (0..=26usize).find(|&r| m.word(tbl, r as u32, 4) == g) {
            let f = unsafe {
                match s {
                    0 => kb::ftKb_Init_803C9CC8[r * 2],
                    1 => kb::ftKb_Init_803C9CC8[r * 2 + 1],
                    2 => kb::ftKb_Init_803C9DD0[r],
                    3 => kb::ftKb_Init_803C9E54[r],
                    _ => None,
                }
            };
            if let Some(f) = f {
                return f as *mut c_void;
            }
        }
    }
    eprintln!("mexdata: kirby_function[{slot}][{k}] = {g:#010X} has no native counterpart - the clone base's callback is kept");
    core::ptr::null_mut()
}
fn kirby_cap_file(k: i32) -> *const c_char {
    let Some(m) = m() else { return core::ptr::null() };
    if k < 0 { core::ptr::null() } else { m.cstr(m.word(m.kirbyfield(0x00), k as u32, 8)) }
}
/// The Kirby hat file of internal kind `k` as the m-ex table names it (gw_Mex_KirbyCapFile without its disc check: the
/// identity table's only caller hashes a missing file as 0, which is what the C's NULL gives).
pub fn KirbyCapFile(k: i32) -> *const c_char {
    kirby_cap_file(k)
}
fn kirby_cap_symbol(k: i32) -> *const c_char {
    let Some(m) = m() else { return core::ptr::null() };
    let t = m.kirbyfield(0x00);
    if k < 0 || t == 0 { core::ptr::null() } else { m.cstr(m.word(t + 4, k as u32, 8)) }
}

// sound banks / music / effects
pub fn SsmCount() -> i32 {
    m().map_or(0, |m| {
        let md = m.meta();
        if m.inr(md, 0x20) { m.r32(md + 0x1C) as i32 } else { 0 }
    })
}
fn ssm_table(m: &Mex, which: u32) -> u32 {
    let s = m.r32(m.root + OFF_SSM);
    m.r32(s + which)
}
pub fn SsmFile(i: i32) -> *const c_char {
    m().map_or(core::ptr::null(), |m| {
        let t = ssm_table(m, 0);
        if t == 0 { core::ptr::null() } else { m.r32(t + 4 * i as u32) as usize as *const c_char }
    })
}
pub fn SsmSize(i: i32) -> u32 {
    let Some(m) = m() else { return 0 };
    let t = ssm_table(m, 4);
    let table = if t != 0 { m.r32(t + 8 * i as u32) } else { 0 };
    let name = SsmFile(i);
    if name.is_null() || unsafe { *name } == 0 {
        return table;
    }
    let path = format!("/audio/us/{}", str_of(name));
    let disc = &mut crate::host::Host::get().disc;
    let e = disc.entry_for_path(&path);
    if e < 0 {
        return 0;
    }
    match disc.read_entry(e, 0, 8) {
        Ok(h) => u32::from_be_bytes(h[4..8].try_into().unwrap()),
        Err(_) => table,
    }
}
pub fn SsmLookup(i: i32, k: i32) -> i32 {
    m().map_or(0, |m| {
        let t = ssm_table(m, 8);
        if t == 0 { 0 } else { m.r8(t + 4 * i as u32 + k as u32).map_or(0, |v| v as i8 as i32) }
    })
}
fn ssm_for_ext(ext: i32) -> i32 {
    let Some(m) = m() else { return -1 };
    if ext < 0 || SsmCount() == 0 {
        return -1;
    }
    let tbl = m.ftfield(0x38);
    if tbl == 0 {
        return -1;
    }
    match m.r8(tbl + ext as u32 * 0x10) {
        Some(e) if e != 0xFF && (e as i32) < SsmCount() => e as i32,
        _ => -1,
    }
}
pub fn SsmForPortCKind(ck: i32) -> i32 {
    ssm_for_ext(PortCKindToExt(ck))
}
pub fn SsmForPortKind(fk: i32) -> i32 {
    let s = slot_of_port(fk);
    let k = if s >= 0 { SlotInternal(s) } else { -1 };
    if k >= 0 { ssm_for_ext(ExtForInternal(k)) } else { -1 }
}
pub fn BgmCount() -> i32 {
    m().map_or(0, |m| {
        let md = m.meta();
        if m.inr(md, 0x24) { m.r32(md + 0x20) as i32 } else { 0 }
    })
}
pub fn BgmFile(i: i32) -> *const c_char {
    let Some(m) = m() else { return core::ptr::null() };
    if i < 0 || i >= BgmCount() {
        return core::ptr::null();
    }
    let names = m.r32(m.r32(m.root + OFF_MUSIC));
    if names == 0 { core::ptr::null() } else { m.r32(names + 4 * i as u32) as usize as *const c_char }
}
pub fn EffectCount() -> i32 {
    m().map_or(0, |m| {
        let md = m.meta();
        let n = if m.inr(md, 0x28) { m.r32(md + 0x24) as i32 } else { 0 };
        if n > 0 && n <= 256 { n } else { 0 }
    })
}
pub fn EffectString(i: i32, which: i32) -> *const c_char {
    let Some(m) = m() else { return core::ptr::null() };
    if i < 0 || i >= EffectCount() || !(0..=1).contains(&which) {
        return core::ptr::null();
    }
    let tbl = m.r32(m.r32(m.root + OFF_EFFECT));
    m.cstr(m.word(tbl, i as u32 * 3 + which as u32, 4))
}
fn fighter_s32_for_ck(ck: i32, off: u32, fallback: i32) -> i32 {
    let Some(m) = m() else { return fallback };
    let ext = PortCKindToExt(ck);
    let tbl = m.ftfield(off);
    if ext < 0 || tbl == 0 || !m.inr(tbl + 4 * ext as u32, 4) { fallback } else { m.r32(tbl + 4 * ext as u32) as i32 }
}
pub fn VictoryThemeForPortCKind(ck: i32) -> i32 {
    fighter_s32_for_ck(ck, 0x30, -1)
}
pub fn AnnouncerForPortCKind(ck: i32) -> i32 {
    fighter_s32_for_ck(ck, 0x34, -1)
}
pub fn ResultFileForPortCKind(ck: i32) -> *const c_char {
    let p = fighter_s32_for_ck(ck, 0x28, 0) as u32;
    match m() {
        Some(m) if p != 0 && m.inr(p, 1) && m.r8(p) != Some(0) => p as usize as *const c_char,
        _ => core::ptr::null(),
    }
}
pub fn ResultScaleForPortCKind(ck: i32) -> f32 {
    f32::from_bits(fighter_s32_for_ck(ck, 0x2C, 0x3F80_0000) as u32)
}
pub fn FighterBgmForPortCKind(ck: i32, which: i32) -> i32 {
    if !(0..=1).contains(&which) {
        return -1;
    }
    let w = fighter_s32_for_ck(ck, 0x54, -1);
    let bgm = if which == 0 { (w >> 16) & 0xFFFF } else { w & 0xFFFF };
    if bgm == 0xFFFF || bgm >= BgmCount() { -1 } else { bgm }
}
fn music_field(off: u32) -> u32 {
    m().map_or(0, |m| m.r32(m.r32(m.root + OFF_MUSIC) + off))
}
pub fn MenuPlaylistCount() -> i32 {
    let (tbl, n) = (music_field(4), music_field(8) as i32);
    match m() {
        Some(m) if tbl != 0 && n > 0 && n <= 256 && m.inr(tbl, n as u32 * 4) => n,
        _ => 0,
    }
}
pub fn MenuPlaylistBgm(i: i32) -> i32 {
    if i < 0 || i >= MenuPlaylistCount() {
        return -1;
    }
    let b = m().map_or(-1, |m| m.r16(music_field(4) + 4 * i as u32) as i32);
    if b >= 0 && b < BgmCount() { b } else { -1 }
}
pub fn MenuPlaylistChance(i: i32) -> i32 {
    if i < 0 || i >= MenuPlaylistCount() {
        return 0;
    }
    m().map_or(0, |m| m.r16(music_field(4) + 4 * i as u32 + 2) as i32)
}
pub fn MenuParamF(i: i32) -> f32 {
    let Some(m) = m() else { return 1.0 };
    if !(0..=7).contains(&i) {
        return 1.0;
    }
    let params = m.r32(m.r32(m.root + 4));
    let p = params + 4 * i as u32;
    if params == 0 || !m.inr(p, 4) { 1.0 } else { f32::from_bits(m.r32(p)) }
}

// stages (gw_mex_grfunction.c)
fn gr_row(grkind: i32) -> u32 {
    let Some(m) = m() else { return 0 };
    if grkind < 0 || grkind >= m.gr_int {
        return 0;
    }
    let p = m.r32(m.gr_desc + 4 * grkind as u32);
    if m.inr(p, GR_SLOT_COUNT * 4) { p } else { 0 }
}
pub fn GrInternalCount() -> i32 {
    m().map_or(0, |m| m.gr_int)
}
pub fn GrExternalCount() -> i32 {
    m().map_or(0, |m| m.gr_ext)
}
pub fn GrKindForExt(ext: i32) -> i32 {
    let Some(m) = m() else { return -1 };
    if ext < 0 || ext >= m.gr_ext {
        return -1;
    }
    let k = m.r32(m.r32(m.gr_map) + ext as u32 * GR_STAGEIDS_STRIDE) as i32;
    if k >= 0 && k < m.gr_int { k } else { -1 }
}
pub fn GrFile(grkind: i32) -> *const c_char {
    let Some(m) = m() else { return core::ptr::null() };
    let row = gr_row(grkind);
    let p = if row != 0 { m.r32(row + GR_SLOT_FILE * 4) } else { 0 };
    if p == 0 || !m.inr(p, 2) {
        return core::ptr::null();
    }
    for i in 0..64u32 {
        match m.r8(p + i) {
            None => return core::ptr::null(),
            Some(0) => {
                if i <= 1 {
                    return core::ptr::null();
                }
                let s = p as usize as *const c_char;
                if grkind >= GR_FIRST_NEW && !file_exists(unsafe { CStr::from_ptr(s) }) {
                    return core::ptr::null();
                }
                return s;
            }
            _ => {}
        }
    }
    core::ptr::null()
}
pub fn GrIsMex(grkind: i32) -> i32 {
    (grkind >= GR_FIRST_NEW && !GrFile(grkind).is_null()) as i32
}
pub fn GrFlags2(grkind: i32) -> u32 {
    let row = gr_row(grkind);
    m().map_or(0, |m| if row != 0 { m.r32(row + GR_SLOT_FLAGS2 * 4) } else { 0 })
}
fn gr_bgm_word(grkind: i32, i: i32, chance: bool) -> i32 {
    let none = if chance { 0 } else { -1 };
    let Some(m) = m() else { return none };
    if grkind < 0 || grkind >= m.gr_int {
        return none;
    }
    let tbl = m.r32(m.gr_map + 0x14);
    let row = tbl + grkind as u32 * 8;
    if tbl == 0 || !m.inr(row, 8) {
        return none;
    }
    let count = m.r32(row) as i32;
    if count <= 0 || count > 64 {
        return none;
    }
    if !chance && i == -1 {
        return count;
    }
    if i < 0 || i >= count {
        return none;
    }
    let e = m.r32(row + 4) + 4 * i as u32;
    if !m.inr(e, 4) {
        return none;
    }
    let (id, ch) = (m.r16(e) as i32, m.r16(e + 2) as i32);
    if id <= 0 || id >= 1024 {
        return none;
    }
    if chance { if (0..=10000).contains(&ch) { ch } else { 0 } } else { id }
}
pub fn GrBgmCount(grkind: i32) -> i32 {
    gr_bgm_word(grkind, -1, false).max(0)
}
pub fn GrBgmId(grkind: i32, i: i32) -> i32 {
    gr_bgm_word(grkind, i, false)
}
pub fn GrBgmChance(grkind: i32, i: i32) -> i32 {
    gr_bgm_word(grkind, i, true)
}

/// Is an m-ex disc mounted (MxDt.dat loaded with its CSS)?
pub fn on() -> bool {
    m().is_some()
}

/// Fill `melee_runtime`'s MexData (the readers linked into the fighter and Kirby code) from the loaded image.
fn install_mex_data() {
    use melee_runtime::tu::mex_data as md;
    let Some(mx) = m() else {
        unsafe { md::MEX = None };
        return;
    };
    let n = mx.internal_count().max(0);
    let cstring = |p: *const c_char| if p.is_null() { None } else { Some(unsafe { CStr::from_ptr(p) }.to_owned()) };
    let data = md::MexData {
        on: true,
        internal_count: n,
        slots: mx.slots.iter().map(|&k| md::MexSlot { internal: k }).collect(),
        base_kind: (0..n).map(FtBaseKind).collect(),
        costume_vis: (0..n).map(|k| (0..FtCostumeCount(k)).map(|c| CostumeVisIdx(k, c)).collect()).collect(),
        kirby_cap_file: (0..n).map(|k| cstring(kirby_cap_file(k))).collect(),
        kirby_cap_symbol: (0..n).map(|k| cstring(kirby_cap_symbol(k))).collect(),
        kirby_costumes: mx.kirby_costumes.clone(),
        kirby_effect_id: (0..n)
            .map(|k| {
                let t = mx.kirbyfield(0x10);
                let v = if t == 0 { 0xFF } else { mx.r8(t + k as u32).unwrap_or(0xFF) };
                if v != 0xFF && v as u32 >= EFFECT_BANK_MAX {
                    eprintln!("mexdata: kirby effect id {v} for internal kind {k} is past this build's {EFFECT_BANK_MAX}-entry effect bank table - treated as none");
                    0xFF
                } else {
                    v
                }
            })
            .collect(),
        kirby_func: (0..KIRBY_FUNC_SLOTS).map(|s| (0..n).map(|k| KirbyFunc(s, k)).collect()).collect(),
        effect_bank_max: EFFECT_BANK_MAX as i32,
        file_exists: Some(file_exists),
    };
    unsafe { md::MEX = Some(data) };
}

/// Load the disc's MxDt.dat (when it has one) and route the m-ex seams to the readers. Run after the link table and
/// before the fallback seams (which only fill hooks still unset).
pub fn install() {
    if !load() {
        return;
    }
    install_mex_data();
    unsafe {
        use melee_fighter::tu::ftdata::ext as fd;
        fd::MEX_SLOT_INTERNAL = Some(SlotInternal);
        fd::MEX_FT_PL_FILE = Some(FtPlFile);
        fd::MEX_FT_PL_SYMBOL = Some(FtPlSymbol);
        fd::MEX_FT_ANIM_FILE = Some(FtAnimFile);
        fd::MEX_FT_ANIM_COUNT = Some(FtAnimCount);
        fd::MEX_FT_COSTUME_COUNT = Some(FtCostumeCount);
        fd::MEX_FT_COSTUME_STRING = Some(FtCostumeString);
        fd::MEX_FT_DEMO_STRINGS = Some(FtDemoStrings);
        fd::MEX_FT_FUNC = Some(FtFunc);
        fd::MEX_FT_BASE_KIND = Some(FtBaseKind);
        fd::MEX_FT_EFFECT_INDEX = Some(FtEffectIndex);
        fd::MEX_EFFECT_STRING = Some(EffectString);
        fd::MEX_FT_FUNCTION_INSTALL = Some(ft_function_install);
        // gw_PcCostumeAlloc: the persistent descriptor pool (allocated once at boot, never freed)
        fd::PC_COSTUME_ALLOC = Some(|n| Box::leak(vec![0u64; n.div_ceil(8).max(1)].into_boxed_slice()).as_mut_ptr() as *mut c_void);

        use melee_modes::tu::gm_1601::ext::hook as g16;
        g16::Mex_AnnouncerForPortCKind = Some(AnnouncerForPortCKind);
        g16::Mex_CostumeInfo = Some(CostumeInfo);
        g16::Mex_FighterName = Some(|e| FighterName(e) as _);
        g16::Mex_PortCKindToExt = Some(PortCKindToExt);
        g16::Mex_ResultFileForPortCKind = Some(|c| ResultFileForPortCKind(c) as _);
        g16::Mex_VictoryThemeForPortCKind = Some(VictoryThemeForPortCKind);

        use melee_runtime::tu::lbaudio_ax::ext::hook as au;
        au::Mex_BgmCount = Some(BgmCount);
        au::Mex_BgmFile = Some(|i| BgmFile(i) as _);
        au::Mex_FighterBgmForPortCKind = Some(FighterBgmForPortCKind);
        au::Mex_SsmCount = Some(SsmCount);
        au::Mex_SsmFile = Some(|i| SsmFile(i) as _);
        au::Mex_SsmForPortCKind = Some(SsmForPortCKind);
        au::Mex_SsmLookup = Some(SsmLookup);
        au::Mex_SsmSize = Some(|i| SsmSize(i) as _);
        melee_fighter::hooks::MEX_SSM_FOR_PORT_KIND = Some(SsmForPortKind);
        melee_items::tu::item::ext::hook::Mex_SsmForPortKind = Some(SsmForPortKind);

        use melee_world::tu::efn::efasync::ext::hook as ef;
        ef::Mex_EffectCount = Some(EffectCount);
        ef::Mex_EffectString = Some(|i, w| EffectString(i, w) as _);

        use melee_world::tu::ground::ext::hook as gr;
        gr::Mex_GrBgmCount = Some(GrBgmCount);
        gr::Mex_GrBgmId = Some(GrBgmId);
        gr::Mex_GrBgmChance = Some(GrBgmChance);
        gr::Mex_GrFile = Some(|k| GrFile(k) as _);
        gr::Mex_GrFlags2 = Some(|k| GrFlags2(k) as _);
        gr::Mex_GrInternalCount = Some(GrInternalCount);
        gr::Mex_GrIsMex = Some(GrIsMex);
        use melee_world::tu::stage::ext as st;
        {
            unsafe extern "C" fn ec() -> i32 { GrExternalCount() }
            unsafe extern "C" fn kfe(e: i32) -> i32 { GrKindForExt(e) }
            st::H_Mex_GrExternalCount = Some(ec);
            st::H_Mex_GrKindForExt = Some(kfe);
        }

        use melee_world::tu::ifstatus::ext::hook as ifs;
        ifs::Mex_InsigniaForExt = Some(InsigniaForExt);
        ifs::Mex_PortCKindToExt = Some(PortCKindToExt);

        use melee_modes::tu::gmmain_lib::ext as gml;
        gml::MEX_MENUPLAYLISTCOUNT = Some(MenuPlaylistCount);
        gml::MEX_MENUPLAYLISTBGM = Some(MenuPlaylistBgm);
        gml::MEX_MENUPLAYLISTCHANCE = Some(MenuPlaylistChance);
    }
}

/// `gw_Mex_FtFunctionInstall(kind, archive data, size)`: the C relocates the fighter file's `ftFunction` PPC code and
/// wires its overrides into the interpreter. Here mex_guest reads the fighter's file (the raw archive, big-endian, as
/// the guest sees it) and registers the overrides with the dispatch tables.
fn ft_function_install(kind: i32, data: *mut u8, size: u32) {
    let _ = (data, size);
    let k = InternalForPortKind(kind);
    let (file, sym) = (str_of(FtPlFile(k)), str_of(FtPlSymbol(k)));
    if file.is_empty() || sym.is_empty() {
        eprintln!("mexdata: kind {kind} (m-ex internal {k}): no PlXx.dat / ftData name - the clone base's handlers run");
        return;
    }
    crate::mex_guest::install(kind, &file, &sym);
    unsafe {
        melee_fighter::platform::hook::Mex_MoveLogicTable = Some(move_logic_table);
    }
}

/// `gw_Mex_MoveLogicTable(kind, vanilla)`: an m-ex fighter's own MoveLogic table, else the vanilla one.
fn move_logic_table(kind: i32) -> *mut melee_types::MotionState {
    match crate::mex_guest::move_logic_table(kind) {
        Some(t) => t,
        None if (0..melee_fighter::platform::MEX_KIND_MAX as i32).contains(&kind) => unsafe { melee_fighter::tu::ftdata::ftData_CharacterStateTables[kind as usize] },
        None => core::ptr::null_mut(),
    }
}

/// The relocated MxDt.dat image (big-endian, at its own low address): (base, size, mexData root).
pub fn mexdata_image() -> Option<(u32, u32, u32)> {
    m().map(|m| (m.base, m.size, m.root))
}

/// `gw_mex_ft_item_global(port kind, n)`: the global item kind of fighter article `n` (MxDt.dat fighter item_lookup,
/// +0x4C: per internal id {count, u16 ids*}).
pub fn ft_item_global(kind: i32, n: u32) -> Option<u32> {
    let m = m()?;
    let k = InternalForPortKind(kind);
    if k < 0 {
        return None;
    }
    let entry = m.ftfield(0x4C).wrapping_add(k as u32 * 8);
    if !m.inr(entry, 8) {
        return None;
    }
    let (count, ids) = (m.r32(entry) as i32, m.r32(entry + 4));
    if (n as i32) < 0 || n as i32 >= count || !m.inr(ids + n * 2, 2) {
        return None;
    }
    Some(m.r16(ids + n * 2))
}

/// The answers `ext_real` gives by name (callers the link table reaches only through `seams`).
pub fn by_name(name: &str, a: &[u64]) -> Option<u64> {
    if !on() {
        return None;
    }
    let i = |n: usize| a.get(n).copied().unwrap_or(0) as i32;
    Some(match name {
        "Mex_InternalCount" => InternalCount() as i64 as u64,
        "Mex_InternalForPortKind" => InternalForPortKind(i(0)) as i64 as u64,
        "Mex_PortCKindToExt" => PortCKindToExt(i(0)) as i64 as u64,
        "Mex_ExtToPortCKind" => ExtToPortCKind(i(0)) as i64 as u64,
        "Mex_FighterName" => FighterName(i(0)) as usize as u64,
        "Mex_CssIconCount" => CssIconCount() as i64 as u64,
        "Mex_CssIconTable" => CssIconTable() as usize as u64,
        // the m-ex CSS's denser grid: the hand and the puck scale with it (the C's default; MELEE_CSS_CURSOR_SCALE=0 off)
        "Mex_CssCursorScaleEnabled" => (std::env::var("MELEE_CSS_CURSOR_SCALE").map_or(true, |v| !v.starts_with('0')) && CssIconCount() > 0) as u64,
        "Mex_InsigniaForExt" => InsigniaForExt(i(0)) as i64 as u64,
        "Mex_GrFile" => GrFile(i(0)) as usize as u64,
        "Mex_GrKindForExt" => GrKindForExt(i(0)) as i64 as u64,
        "Mex_ResultScaleForPortCKind" => ResultScaleForPortCKind(i(0)).to_bits() as u64,
        "Mex_MenuParamF" => MenuParamF(i(0)).to_bits() as u64,
        _ => return None,
    })
}

/// Is `tok` an explicit m-ex reference (`mexint:N`, `mex:N`, `mexext:N`)?
pub fn is_reference(tok: &str) -> bool {
    ["mexint:", "mex:", "mexext:"].iter().any(|p| tok.starts_with(p))
}

/// gw_sl_norm: letters and digits only, lower case.
fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect()
}

/// A scene's fighter token on an m-ex disc (gw_runtime.c gw_sl_parse_char): `mexext:N`, `mexint:N` / `mex:N`, or a
/// name the disc's fighter table carries ("Ultimate Kirby" answers `ultimatekirby`). None: not an m-ex fighter here.
pub fn resolve_fighter(tok: &str) -> Option<i32> {
    if !on() {
        return None;
    }
    let num = |p: &str| tok.strip_prefix(p).filter(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit())).map(|r| r.parse::<i32>().unwrap());
    if let Some(e) = num("mexext:") {
        let ck = ExtToPortCKind(e);
        return (ck >= 0).then_some(ck);
    }
    if let Some(k) = num("mexint:").or_else(|| num("mex:")) {
        if (0..=25).contains(&k) {
            return Some(k);
        }
        let s = slot_of_internal(k);
        return (s >= 0).then_some(CK_MEX0 + s);
    }
    let want = norm(tok);
    let n = InternalCount();
    for e in 0..n {
        let have = norm(&str_of(FighterName(e)));
        if !have.is_empty() && have == want {
            let ck = ExtToPortCKind(e);
            if ck >= 0 {
                return Some(ck);
            }
        }
    }
    None
}

/// What the Geno registry asks the host (melee_script `geno::registry::Env`): the m-ex fighter in each dense slot (so a
/// geno.json entry for `PlBm.dat` finds kind 57) and a mounted mod's own files (its subaction words files).
pub struct GenoEnv {
    pub dir: std::path::PathBuf,
}
impl melee_script::geno::registry::Env for GenoEnv {
    fn fighter_wasm(&self, module: &[u8], slot_names: &[String], slot_types: &[i32], targets: &[String]) -> Result<Box<dyn melee_script::geno::code::FighterCode>, String> {
        crate::geno_wasm::load(module, slot_names, slot_types, targets).map(|g| Box::new(g) as Box<dyn melee_script::geno::code::FighterCode>)
    }
    fn mex_pl_file(&self, slot: usize) -> Option<String> {
        let k = SlotInternal(slot as i32);
        if k < 0 {
            return None;
        }
        Some(str_of(FtPlFile(k)))
    }
    fn read_mod_file(&self, mod_id: &str, rel: &str) -> Option<Vec<u8>> {
        if self.dir.as_os_str().is_empty() {
            return None;
        }
        std::fs::read(self.dir.join(mod_id).join(rel)).ok()
    }
}
