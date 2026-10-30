//! The m-ex / skins / replay *data services* the fighter code asks the platform for
//! (`pc/platform/gw_mex_ftfunction_runtime.c`: `Mex_InternalForPortKind`, `Mex_FtBaseKind`,
//! `Mex_CostumeVisIdx`, `Mex_Kirby*`; `gw_skins_core.h`: `Skins_Like`, `Skins_KirbyRow`;
//! `gw_replay.c`: `Slippi_Version`; `gw_runtime.c`: `Mex_OnFloatDispatch`).
//!
//! The C reads them out of the loaded `MxDt.dat` image; natively the host (the `melee` binary,
//! through `mex_import`) parses the image and installs the rows into [`MexData`] / [`SkinRegistry`].
//! Nothing installed is the C's "no mexData / no skins" state (`gw_mex_ft_on() == 0`): the
//! vanilla answers below, which are the ones a vanilla disc gets.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::{c_void, CStr};
use std::ffi::CString;
use std::vec::Vec;

/// One port fighter slot (`gw_mex_slot_internal[slot]`): the m-ex internal id it holds.
#[derive(Clone, Debug)]
pub struct MexSlot {
    pub internal: i32,
}

/// The rows of `MxDt.dat` the ported code reads. Tables are indexed by INTERNAL id except where
/// noted.
#[derive(Default)]
pub struct MexData {
    /// `gw_mex_ft_on()`: a real `MxDt.dat` with CSS icons (or the synthetic fighter image).
    pub on: bool,
    /// metadata.internal_id_count (41 on Akaneia).
    pub internal_count: i32,
    /// `Ft_Kind_Mex0 + s` -> internal id.
    pub slots: Vec<MexSlot>,
    /// `gw_Mex_FtBaseKind(k)`: the retail kind a fighter was cloned from.
    pub base_kind: Vec<i32>,
    /// `gw_Mex_FtCostumeCount(k)` and `costume_file[k][c].visibility_lookup_idx`.
    pub costume_vis: Vec<Vec<i32>>,
    /// kirby_data.capfiles[k].filename (None: no row).
    pub kirby_cap_file: Vec<Option<CString>>,
    /// kirby_data.capfiles[k].symbol.
    pub kirby_cap_symbol: Vec<Option<CString>>,
    /// kirby_data.costumes[k] (a `Fighter_CostumeStrings*`).
    pub kirby_costumes: Vec<*mut c_void>,
    /// kirby_data.effectids[k] (255 = none).
    pub kirby_effect_id: Vec<u8>,
    /// kirby_function[slot][k].
    pub kirby_func: Vec<Vec<*mut c_void>>,
    /// `GW_MEX_EFFECT_BANK_MAX`.
    pub effect_bank_max: i32,
    /// `gw_DVDConvertPathToEntrynum(path) >= 0`.
    pub file_exists: Option<fn(&CStr) -> bool>,
}

/// One skin costume row (`sk_costume`).
#[derive(Clone, Copy, Debug)]
pub struct SkinRow {
    pub fk: i32,
    pub index: i32,
    pub like: i32,
    pub kirby_hat: i32,
}

/// The skins registry (`sk_reg`): rows sorted by fighter, then costume index.
#[derive(Default)]
pub struct SkinRegistry {
    /// `base[fk]`: the original costume count.
    pub base: Vec<i32>,
    pub rows: Vec<SkinRow>,
}

pub static mut MEX: Option<MexData> = None;
pub static mut SKINS: Option<SkinRegistry> = None;
/// The replay's Slippi version (`major*10000 + minor*100 + build`) while a replay plays.
pub static mut SLIPPI_REPLAY_VERSION: Option<i32> = None;
/// `gw_Mex_GObjPredDispatch(GW_MEX_EVENT_ON_FLOAT, kind, gobj, NULL)`: the m-ex float handler of a
/// kind, or `None` (the C returns 0 when no hook is registered).
pub static mut ON_FLOAT_HOOK: Option<fn(i32, *mut c_void) -> i32> = None;

fn mex() -> Option<&'static MexData> {
    unsafe { (*(&raw const MEX)).as_ref().filter(|m| m.on) }
}

/// `gw_Mex_InternalCount()`.
pub fn Mex_InternalCount() -> i32 {
    melee_core::ct!("Mex_InternalCount");
    mex().map_or(0, |m| m.internal_count)
}

/// `gw_Mex_SlotInternal(slot)`.
fn slot_internal(slot: i32) -> i32 {
    mex().and_then(|m| m.slots.get(slot as usize)).map_or(-1, |s| s.internal)
}

/// `gw_mex_slot_of_port(fk)`: `Ft_Kind_Mex0` (0x21) .. + 94.
fn slot_of_port(fk: i32) -> i32 {
    if (0x21..0x21 + 94).contains(&fk) {
        fk - 0x21
    } else {
        -1
    }
}

/// `Mex_InternalForPortKind(fk)`.
pub fn Mex_InternalForPortKind(fk: i32) -> i32 {
    melee_core::ct!("Mex_InternalForPortKind");
    let n = Mex_InternalCount();
    let s = slot_of_port(fk);
    if (0..=26).contains(&fk) {
        return fk;
    }
    if (27..=32).contains(&fk) {
        // the six bosses: m-ex moves them to the last six internal ids
        return if n > 6 { n - 6 + (fk - 27) } else { fk };
    }
    if s >= 0 {
        slot_internal(s)
    } else {
        -1
    }
}

/// `Mex_FtBaseKind(k)`: 0 (Mario) without mexData.
pub fn Mex_FtBaseKind(k: i32) -> i32 {
    melee_core::ct!("Mex_FtBaseKind");
    match mex() {
        None => 0,
        Some(m) => usize::try_from(k).ok().and_then(|k| m.base_kind.get(k)).copied().unwrap_or(0),
    }
}

fn skin_row(fk: i32, idx: i32) -> Option<SkinRow> {
    let reg = unsafe { (*(&raw const SKINS)).as_ref()? };
    let base = *reg.base.get(usize::try_from(fk).ok()?)?;
    if idx < base {
        return None;
    }
    reg.rows.iter().copied().find(|r| r.fk == fk && r.index == idx)
}

/// `Skins_Like(fk, idx)`: the original costume a skin's part visibility copies, or -1.
pub fn Skins_Like(fk: i32, idx: i32) -> i32 {
    melee_core::ct!("Skins_Like");
    let Some(r) = skin_row(fk, idx) else { return -1 };
    let base = unsafe { (*(&raw const SKINS)).as_ref().map_or(0, |s| s.base[fk as usize]) };
    if r.like >= 0 && r.like < base {
        r.like
    } else {
        0
    }
}

/// `Skins_KirbyRow(fk, idx)`: the Kirby copy-hat row (0..5) of a Kirby skin costume, or -1.
pub fn Skins_KirbyRow(fk: i32, idx: i32) -> i32 {
    melee_core::ct!("Skins_KirbyRow");
    let Some(r) = skin_row(fk, idx) else { return -1 };
    if (0..=5).contains(&r.kirby_hat) {
        return r.kirby_hat;
    }
    Skins_Like(fk, idx) % 6
}

/// `Mex_CostumeVisIdx(fk, costume)`: the part-visibility table a costume uses; the costume id
/// itself when there is no mexData or no row.
pub fn Mex_CostumeVisIdx(fk: i32, costume: i32) -> i32 {
    melee_core::ct!("Mex_CostumeVisIdx");
    let mut costume = costume;
    let like = Skins_Like(fk, costume); // a skin costume copies an original costume's part visibility
    if like >= 0 {
        costume = like;
    }
    let k = Mex_InternalForPortKind(fk);
    let Some(m) = mex() else { return costume };
    if k < 0 || costume < 0 {
        return costume;
    }
    let Some(row) = m.costume_vis.get(k as usize) else { return costume };
    if costume as usize >= row.len() {
        return costume;
    }
    let v = row[costume as usize];
    if (0..255).contains(&v) {
        v
    } else {
        0
    }
}

/// `Mex_KirbyCapFile(k)`: the hat's archive name, NULL when there is no row or the file is not on
/// the disc (or in a mod).
pub fn Mex_KirbyCapFile(k: i32) -> *mut i8 {
    melee_core::ct!("Mex_KirbyCapFile");
    let Some(m) = mex() else { return core::ptr::null_mut() };
    let Some(Some(name)) = usize::try_from(k).ok().and_then(|k| m.kirby_cap_file.get(k)) else {
        return core::ptr::null_mut();
    };
    if name.as_bytes().is_empty() {
        return core::ptr::null_mut();
    }
    if let Some(exists) = m.file_exists {
        if !exists(name.as_c_str()) {
            return core::ptr::null_mut();
        }
    }
    name.as_ptr() as *mut i8
}

/// `Mex_KirbyCapSymbol(k)`.
pub fn Mex_KirbyCapSymbol(k: i32) -> *mut i8 {
    melee_core::ct!("Mex_KirbyCapSymbol");
    mex()
        .and_then(|m| usize::try_from(k).ok().and_then(|k| m.kirby_cap_symbol.get(k)))
        .and_then(|s| s.as_ref())
        .map_or(core::ptr::null_mut(), |s| s.as_ptr() as *mut i8)
}

/// `Mex_KirbyCostumes(k)`.
pub fn Mex_KirbyCostumes(k: i32) -> *mut c_void {
    melee_core::ct!("Mex_KirbyCostumes");
    mex()
        .and_then(|m| usize::try_from(k).ok().and_then(|k| m.kirby_costumes.get(k)))
        .copied()
        .unwrap_or(core::ptr::null_mut())
}

/// `Mex_KirbyEffectId(k)`: the effect bank, -1 when the hat loads none (255 is m-ex's "none").
pub fn Mex_KirbyEffectId(k: i32) -> i32 {
    melee_core::ct!("Mex_KirbyEffectId");
    let Some(m) = mex() else { return -1 };
    let Some(&v) = usize::try_from(k).ok().and_then(|k| m.kirby_effect_id.get(k)) else { return -1 };
    if v == 0xFF {
        return -1;
    }
    if v as i32 >= m.effect_bank_max {
        return -1;
    }
    v as i32
}

/// `Mex_KirbyFunc(slot, k)`: the hat callback row, NULL when empty.
pub fn Mex_KirbyFunc(slot: i32, k: i32) -> *mut c_void {
    melee_core::ct!("Mex_KirbyFunc");
    if k < 0 || slot < 0 {
        return core::ptr::null_mut();
    }
    mex()
        .and_then(|m| m.kirby_func.get(slot as usize))
        .and_then(|row| row.get(k as usize))
        .copied()
        .unwrap_or(core::ptr::null_mut())
}

/// `Slippi_Version()`: the replay's version while one plays, else a large number (live play gets
/// the current codes).
pub fn Slippi_Version() -> i32 {
    melee_core::ct!("Slippi_Version");
    unsafe { SLIPPI_REPLAY_VERSION }.unwrap_or(999_999)
}

/// `Mex_OnFloatDispatch(kind, gobj)`.
pub fn Mex_OnFloatDispatch(kind: i32, gobj: *mut c_void) -> i32 {
    melee_core::ct!("Mex_OnFloatDispatch");
    match unsafe { ON_FLOAT_HOOK } {
        Some(f) => f(kind, gobj),
        None => 0,
    }
}
