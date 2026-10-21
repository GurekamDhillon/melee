//! src/melee/ft/ftlib.c: the functions left out of `crate::ftlib` (round 3), which holds the
//! other 93 (oracle-verified by tests/ftlib_ref.rs) and is re-exported here.
//! ftLib_8008777C has no fused site (fma_sites.py); it is a plain atan2f of the floor normal.
#![allow(non_snake_case)]

pub use crate::ftlib::*;

use crate::hooks as h;
use crate::types::*;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        FTCO_800C8348 ftCo_800C8348(g: G, a: i32, b: i32);
        IT_8026B560 it_8026B560(it: G) -> i32;
        IT_8026B574 it_8026B574(it: G) -> i32;
        CAMERA_REQUEST_QUAKE Camera_RequestQuake(k: CmQuakeKind, pos: &Vec3);
        FTDATA_8008572C ftData_8008572C(kind: i32);
        EFASYNC_LOAD_SYNC efAsync_LoadSync(id: i32);
        FTDATA_80085820 ftData_80085820(kind: i32, costume: u8);
        FTDATA_80085A14 ftData_80085A14(kind: i32);
        FTDATA_800857E0 ftData_800857E0(kind: i32);
        // `ftData_UnkBytePerCharacter[kind]` (ftdata.c table).
        FTDATA_UNK_BYTE ftData_UnkBytePerCharacter(kind: i32) -> i32;
        // `CostumeListsForeachCharacter[kind].numCostumes`.
        COSTUME_COUNT CostumeListsForeachCharacter_numCostumes(kind: i32) -> u8;
    }
}

/// `ftLib_800871A8`: fighter picks up a metal box.
pub fn ftlib_800871a8(g: G, item: G) {
    assert!(h::it_get_kind(item) == It_Kind_MetalB as i32, "ftlib.c:1117");
    ext::ftCo_800C8348(g, ext::it_8026B560(item), ext::it_8026B574(item));
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
    crate::ftcommon2::ftcommon_8007ebac(fp(g), 0xC, 0);
    let pos = fp(g).cur_pos;
    ext::Camera_RequestQuake(QuakeKind_Small, &pos);
    h::play_sfx(fp(g), 0x121, 0x7F, 0x40);
}

/// `ftLib_80087508`: load one costume's archives for `ft_kind` (C param is s8).
pub fn ftlib_80087508(ft_kind: i8, arg1: u8) {
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    ext::ftData_80085820(k, arg1);
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_80087574`: load every costume's archives for `ft_kind`.
pub fn ftlib_80087574(ft_kind: i8) {
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    let mut i: u8 = 0;
    while i < ext::CostumeListsForeachCharacter_numCostumes(k) {
        ext::ftData_80085820(k, i);
        i = i.wrapping_add(1);
    }
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_80087610` (TARGET_PC path): Kirby hat archives for every valid selectable kind.
pub fn ftlib_80087610(arg0: u8) {
    for i in 0..SELKIND_COUNT as i32 {
        h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 0) as i32, arg0 as i32);
        if i == CKind_Zelda as i32 || i == CKind_Seak as i32 {
            h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 1) as i32, arg0 as i32);
        }
    }
}

/// `ftLib_8008777C`: floor slope angle; asserts the fighter is grounded.
pub fn ftlib_8008777c(g: G) -> f32 {
    let f = fp(g);
    assert!(f.ground_or_air == GA_Ground, "ftlib.c:1517");
    let n = f.coll_data.floor.normal;
    melee_runtime::lbtrigf::atan2f(-n.x, n.y)
}
