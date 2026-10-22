//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattack100.c: Mr. Game & Watch's rapid jab (start,
//! loop, end). No float arithmetic (0 fused sites). Smoke-tested by
//! tests/tu_ftgamewatchattack100.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/12): re-exported under this TU.
pub use crate::chars::r12::ftGameWatch::ftgamewatchattack100::{ftGw_Attack100End_IASA, ftGw_Attack100Start_IASA};
pub use crate::chars::r2::ftGameWatch::ftgamewatchattack100::{
    ftGw_Attack100End_Phys, ftGw_Attack100Loop_Phys, ftGw_Attack100Start_Phys,
};
pub use crate::chars::r3::ftGameWatch::ftgamewatchattack100::{
    ftGw_Attack100End_Enter, ftGw_Attack100Loop_Enter, ftGw_Attack100Start_Anim,
};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook_or!(FTCO_800D6B00, ftCo_800D6B00, (gobj: *mut HSD_GObj, msid: i32), melee_fighter::tu::ftCo_Attack100::ftCo_800D6B00);
    hook_or!(FTCO_800D6C60, ftCo_800D6C60, (gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)), melee_fighter::tu::ftCo_Attack100::ftCo_800D6C60);
    hook_or!(FTCO_ATTACK100LOOP_IASA, ftCo_Attack100Loop_IASA, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Attack100::ftCo_Attack100Loop_IASA);
    hook_or!(FTCO_ATTACK100END_ANIM, ftCo_Attack100End_Anim, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Attack100::ftCo_Attack100End_Anim);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftGw_Attack100Start_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_800D6B00(gobj, ftGw_MS_Attack100Start as i32);
        (*fp).accessory4_cb = Some(crate::tu::ftgamewatchattack11::cb_ftGw_Attack11_ItemGreenhouseSetup);
    }
}

pub fn ftGw_Attack100Start_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    crate::tu::ftgamewatch::ftGw_Init_8014A538(gobj);
}

pub fn ftGw_Attack100Loop_Anim(gobj: *mut HSD_GObj) {
    ext::ftCo_800D6C60(gobj, ftGw_Attack100End_Enter);
}

pub fn ftGw_Attack100Loop_IASA(gobj: *mut HSD_GObj) {
    ext::ftCo_Attack100Loop_IASA(gobj);
}

pub fn ftGw_Attack100Loop_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    crate::tu::ftgamewatch::ftGw_Init_8014A538(gobj);
}

pub fn ftGw_Attack100End_Anim(gobj: *mut HSD_GObj) {
    ext::ftCo_Attack100End_Anim(gobj);
}

pub fn ftGw_Attack100End_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
    crate::tu::ftgamewatch::ftGw_Init_8014A538(gobj);
}
