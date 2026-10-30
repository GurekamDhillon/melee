//! src/melee/ft/ftswing.c: battering-item swings (bat, fan, beam sword, star rod, lip's stick,
//! ...): the per-item motion table, item kind -> swing type, and the dispatch to each swing's
//! setup.
#![allow(non_snake_case, non_upper_case_globals)]

use crate::types::*;

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    pub fn ft_800831CC(gobj: *mut HSD_GObj, a: fn(*mut HSD_GObj, i32) -> bool, b: fn(*mut HSD_GObj)) { { unsafe { let _ = crate::tu::ft_081b::ft_800831cc(gobj as _, a as _, b as _); } return; } }
    pub fn ftCo_80096CC8(gobj: *mut HSD_GObj, line_id: i32) -> bool { { return unsafe { crate::tu::ftCo_FallSpecial::ftCo_80096CC8(gobj as _, line_id as _) as _ }; } }
    pub fn ft_80082B1C(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ft_081b::ft_80082b1c(gobj as _); } return; } }
    pub fn ftpickupitem_80094790(gobj: *mut HSD_GObj) -> bool { { return unsafe { crate::tu::ftpickupitem::ftpickupitem_80094790(gobj as _) as _ }; } }
    pub fn ftCo_800CD350(gobj: *mut HSD_GObj, a: i32, b: i32, f: f32) { { unsafe { let _ = crate::tu::ft_0cd3::ftco_800cd350(gobj as _, a as _, b as _, f as _); } return; } }
    pub fn ftCo_800CD418(gobj: *mut HSD_GObj, a: i32, b: i32, f: f32) { { unsafe { let _ = crate::tu::ft_0cd3::ftco_800cd418(gobj as _, a as _, b as _, f as _); } return; } }
    pub fn ftCo_800CD4E0(gobj: *mut HSD_GObj, a: i32, b: i32, f: f32) { { unsafe { let _ = crate::tu::ft_0cd3::ftco_800cd4e0(gobj as _, a as _, b as _, f as _); } return; } }
    pub fn ftCo_800CD5A8(gobj: *mut HSD_GObj, a: i32, b: i32, f: f32) { { unsafe { let _ = crate::tu::ft_0cd3::ftco_800cd5a8(gobj as _, a as _, b as _, f as _); } return; } }
    /// `Fighter_804D654C` (fighter.c, `float (*)[5]`, set from the common data at load).
    pub static mut Fighter_804D654C: *mut [f32; 5] = core::ptr::null_mut();
}

/// `lbl_803C6D70[6][5]`: swing motion per swing type x swing (`-1`: smash 4 is per character).
pub static lbl_803C6D70: [[i32; 5]; 6] = [
    [120, 121, 122, -1, 123],
    [124, 125, 126, -1, 127],
    [128, 129, 130, -1, 131],
    [132, 133, 134, -1, 135],
    [136, 137, 138, -1, 139],
    [140, 141, 142, -1, 143],
];

pub fn ftCo_FallAerial_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCo_FallAerial_Coll");
    ext::ft_800831CC(gobj, ext::ftCo_80096CC8, ext::ft_80082B1C)
}

/// `fn_800CCEC4(item_kind)`: swing type. Any other kind is the C's fatal assert.
pub fn fn_800CCEC4(x: i32) -> i32 {
    melee_core::ct!("fn_800CCEC4");
    match x {
        12 => 0,
        11 => 1,
        13 => 2,
        24 => 3,
        22 => 4,
        23 => 5,
        _ => panic!("ellegal swing item!!!"),
    }
}

/// `get_anim_id(fp, swing_type, swing)` with the fighter's kind passed in. Under TARGET_PC the
/// kind is `Geno_DefineBaseKind(kind)` when that is >= 0. Only Captain Falcon has swing 3.
pub fn get_anim_id_kind(kind: i32, swing_type: i32, arg1: i32) -> i32 {
    melee_core::ct!("get_anim_id_kind");
    if arg1 != 3 {
        return lbl_803C6D70[swing_type as usize][arg1 as usize];
    }
    let b = crate::hooks::geno_define_base_kind(kind);
    let swing_kind = if b >= 0 { b } else { kind };
    if swing_kind == Ft_Kind_Captain as i32 {
        return swing_type + 341;
    }
    panic!("don't have swing42 motion!!!");
}

pub fn get_anim_id(fp: &Fighter, swing_type: i32, arg1: i32) -> i32 {
    melee_core::ct!("get_anim_id");
    get_anim_id_kind(fp.kind as i32, swing_type, arg1)
}

pub fn ftCo_Attack_800CCF58(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("ftCo_Attack_800CCF58");
    let f = super::ftlipstickswing::fp(gobj);
    if arg1 == 0 || arg1 == 1 {
        if ext::ftpickupitem_80094790(gobj) {
            return;
        }
    }
    assert!(!f.item_gobj.is_null());
    let swing_type = fn_800CCEC4(melee_items::itlib::it_get_kind(super::ftlipstickswing::item(f.item_gobj)) as i32);
    let anim_id = get_anim_id(f, swing_type, arg1);
    let fl = unsafe { (*ext::Fighter_804D654C.add(swing_type as usize))[arg1 as usize] };
    match swing_type {
        0 => ext::ftCo_800CD350(gobj, anim_id, arg1, fl),
        1 => ext::ftCo_800CD418(gobj, anim_id, arg1, fl),
        2 => ext::ftCo_800CD4E0(gobj, anim_id, arg1, fl),
        3 => ext::ftCo_800CD5A8(gobj, anim_id, arg1, fl),
        4 => super::ftstarrodswing::ftCo_800CD82C(gobj, anim_id, arg1, fl),
        5 => super::ftlipstickswing::ftCo_800CDAB4(gobj, anim_id, arg1, fl),
        _ => {}
    }
}
