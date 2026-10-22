//! src/melee/ft/kinds/ftPeach/ftpeachspeciallw.c: Peach's down B (pull a vegetable / throw it).
//! No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested through
//! recording hooks by tests/tu_ftpeachspeciallw.rs.
//!
//! `pickVeg` reads `speciallw_item_table[idx + 1]` one entry past the 3-entry table on its last
//! iteration, and `table[-1].kind` if no entry matched; both are GC memory reads of the
//! neighbouring `ftPe_DatAttrs` fields (`x30` / `x14`), reproduced here with raw offset reads.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/4/5): re-exported under this TU.
pub use crate::chars::r2::ftPeach::ftpeachspeciallw::{ftPe_SpecialAirLw_Phys, ftPe_SpecialLw_Phys};
pub use crate::chars::r4::ftPeach::ftpeachspeciallw::ftPe_SpecialLw_UnsetVeg;
pub use crate::chars::r5::ftPeach::ftpeachspeciallw::{ftPe_SpecialAirLw_Coll, ftPe_SpecialLw_Coll};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// `HSD_Randi(max)` (baselib/random.c); unset, group 3's `hooks::hsd_randi`.
    pub static mut HSD_RANDI: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_Randi(max: i32) -> i32 {
        match unsafe { HSD_RANDI } {
            Some(f) => f(max),
            None => melee_fighter::hooks::hsd_randi(max),
        }
    }
    hook!(IT_8026B300, itGetKind, (item: *mut Item_GObj) -> i32);
    hook!(IT_802BD45C, it_802BD45C, (item: *mut Item_GObj));
    hook!(IT_802BD4AC, it_802BD4AC, (gobj: *mut HSD_GObj, pos: *mut Vec3, part: u32, kind: i32, dir: f32) -> *mut Item_GObj);
    hook!(FTPICKUPITEM_80094818, ftpickupitem_80094818, (gobj: *mut HSD_GObj, flag: bool));
    hook!(LB_8000B1CC, lb_8000B1CC, (jobj: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3));
    hook!(FTCO_800957F4, ftCo_800957F4, (gobj: *mut HSD_GObj, msid: i32));
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCheckThrowB0(fp)` (ft/inlines.h): test and clear `throw_flags_b0`.
unsafe fn ftCheckThrowB0(fp: *mut Fighter) -> bool {
    if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
        true
    } else {
        false
    }
}

/// `table[i].randi_max`, read at GC offset `0x18 + 8 * i` of the attribute record (so
/// `i == 3` is `x30`, exactly as the C out-of-bounds read).
unsafe fn table_randi_max(a: *const ftPe_DatAttrs, i: isize) -> i32 {
    *((a as *const u8).offset(0x18 + 8 * i) as *const i32)
}
/// `table[i].kind` at GC offset `0x1C + 8 * i` (`i == -1` is `x14`).
unsafe fn table_kind(a: *const ftPe_DatAttrs, i: isize) -> i32 {
    *((a as *const u8).offset(0x1C + 8 * i) as *const i32)
}

/// `pickVeg` (non-static in C): weighted pick from the item table.
pub fn pickVeg(gobj: *mut HSD_GObj) -> ItemKind {
    unsafe {
        let a = (*fpp(gobj)).dat_attrs as *const ftPe_DatAttrs;
        let count = (*a).speciallw_item_table_count as isize;
        let mut odds: i32 = 0;
        let mut idx: isize = 0;
        while idx < count {
            odds = odds.wrapping_add(table_randi_max(a, idx));
            idx += 1;
        }
        let randi = ext::HSD_Randi(odds);
        let mut result: isize = -1;
        idx = 0;
        let mut randi_max = table_randi_max(a, 0);
        while idx < count && result == -1 {
            if randi < randi_max {
                result = idx;
            }
            randi_max = randi_max.wrapping_add(table_randi_max(a, idx + 1));
            idx += 1;
        }
        table_kind(a, result)
    }
}

pub fn ftPe_SpecialLw_8011CFA0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let veg_gobj = (*fp).u.pe.veg_gobj;
        if !veg_gobj.is_null() {
            let item_gobj = (*fp).item_gobj;
            if veg_gobj == item_gobj && ext::itGetKind(item_gobj) == It_Kind_Peach_Turnip {
                ext::it_802BD45C((*fp).u.pe.veg_gobj);
                ftPe_SpecialLw_UnsetVeg(gobj);
            }
        }
    }
}

/// `getVeg` (static): turnip, unless a `1 in x14` roll picks from the item table.
fn getVeg(gobj: *mut HSD_GObj) -> ItemKind {
    unsafe {
        let da = (*fpp(gobj)).dat_attrs as *const ftPe_DatAttrs;
        let da_x14 = (*da).x14;
        let mut kind = It_Kind_Peach_Turnip;
        if ext::HSD_Randi(da_x14) == 0 {
            kind = pickVeg(gobj);
        }
        kind
    }
}

/// `setupVeg` (static).
fn setupVeg(kind: ItemKind, gobj: *mut HSD_GObj, fp: *mut Fighter, pos: *mut Vec3) {
    unsafe {
        let part = (*(*(*fp).ft_data).x8).x10 as u32;
        let veg_gobj = ext::it_802BD4AC(gobj, pos, part, kind, (*fp).facing_dir);
        (*fp).item_gobj = veg_gobj;
        (*fp).u.pe.veg_gobj = veg_gobj;
        if !veg_gobj.is_null() {
            ext::ftpickupitem_80094818(gobj, false);
            ext::efSync_Spawn(1234, gobj, &[&raw mut (*fp).cur_pos as *mut core::ffi::c_void]);
            (*fp).death2_cb = Some(crate::tu::ftpeach::cb_ftPe_Init_OnDeath2);
            (*fp).take_dmg_cb = Some(crate::tu::ftpeach::cb_ftPe_Init_OnDeath2);
        }
    }
}

/// `spawnVeg` (static): the Accessory4 callback that pulls the vegetable.
pub fn spawnVeg(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut pos: Vec3 = core::mem::zeroed();
        if ftCheckThrowB0(fp) {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_109 as usize)).joint, core::ptr::null(), &mut pos);
            let kind = getVeg(gobj);
            setupVeg(kind, gobj, fp, &mut pos);
        }
    }
}
/// `spawnVeg` as the Accessory4 callback entry.
pub unsafe extern "C" fn cb_spawnVeg(g: *mut HSD_GObj) {
    spawnVeg(g);
}

/// `throwVegIfHeld` (static): holding any item means the move is a throw; a turnip is thrown.
fn throwVegIfHeld(gobj: *mut HSD_GObj, msid: i32) -> bool {
    unsafe {
        let igobj = (*fpp(gobj)).item_gobj;
        if !igobj.is_null() {
            if ext::itGetKind(igobj) == It_Kind_Peach_Turnip {
                ext::ftCo_800957F4(gobj, msid);
            }
            true
        } else {
            false
        }
    }
}

pub fn ftPe_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !throwVegIfHeld(gobj, ftCo_MS_LightThrowF4 as i32) {
            (*fp).anon_2210.throw_flags = 0;
            st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialLw as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).accessory4_cb = Some(cb_spawnVeg);
        }
    }
}

pub fn ftPe_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    throwVegIfHeld(gobj, ftCo_MS_LightThrowAirF4 as i32);
}

/// `coll_mf` (static const).
pub const coll_mf: MotionFlags = Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

/// `handleAirColl` (static).
pub fn handleAirColl(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftPe_MS_SpecialAirLw as i32, coll_mf);
        (*fp).accessory4_cb = Some(cb_spawnVeg);
    }
}

/// `handleColl` (static).
pub fn handleColl(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftPe_MS_SpecialLw as i32, coll_mf);
        (*fp).accessory4_cb = Some(cb_spawnVeg);
    }
}

/// `doAnim` (static).
fn doAnim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        cb(gobj);
        ftPe_SpecialLw_UnsetVeg(gobj);
    }
}

pub fn ftPe_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    doAnim(gobj, st::ft_8008A2BC);
}

pub fn ftPe_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    doAnim(gobj, st::ftCo_Fall_Enter);
}
