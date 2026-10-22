//! src/melee/ft/kinds/ftGigaKoopa/ftgkoopa.c: Giga Bowser's init callbacks (most forward to
//! Bowser's, ftKoopa). No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftgkoopa.rs.
//!
//! Data: file-name strings below. `ftGk_CostumeList` does not exist in this TU.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6 and the demo-callback helper): re-exported under this TU.
pub use crate::chars::ftGigaKoopa::ftgkoopa::ftGk_Init_UnkDemoCallbacks0;
pub use crate::chars::r5::ftGigaKoopa::ftgkoopa::{ftGk_Init_OnDeath, ftGk_Init_OnKnockbackEnter, ftGk_Init_OnKnockbackExit};
pub use crate::chars::r6::ftGigaKoopa::ftgkoopa::{ftGk_Init_OnItemInvisible, ftGk_Init_OnItemVisible};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

pub const ftGk_Init_DatFilename: &str = "PlGk.dat";
pub const ftGk_Init_DataName: &str = "ftDataGkoopa";
pub const ftGk_Init_803D3904: &str = "PlGkNr.dat";
pub const ftGk_Init_803D3910: &str = "PlyGkoopa5K_Share_joint";
pub const ftGk_Init_803D3928: &str = "PlyGkoopa5K_Share_matanim_joint";
pub const ftGk_Init_AnimDatFilename: &str = "PlGkAJ.dat";
/// `ftGk_Init_DemoMotionFilenames` is `{ 0 }`: no demo motion files.
pub const ftGk_Init_DemoMotionFilenames: [Option<&str>; 4] = [None; 4];
/// `ftGk_Init_803D3984`.
pub const ftGk_Init_803D3984: [&str; 1] = ["ftDemoVi1201V2MotionFileGkoopa"];
/// `ftGk_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftGk_Init_CostumeStrings: [[&str; 3]; 1] =
    [[ftGk_Init_803D3904, ftGk_Init_803D3910, ftGk_Init_803D3928]];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    // ftKoopa/ftkoopa.c
    hook!(FTKP_INIT_UNKMOTIONSTATES3, ftKp_Init_UnkMotionStates3, (gobj: *mut HSD_GObj));
    hook!(FTKP_INIT_ONLOADFORGKOOPA, ftKp_Init_OnLoadForGKoopa, (fp: *mut Fighter));
    hook!(FTKP_INIT_LOADSPECIALATTRS, ftKp_Init_LoadSpecialAttrs, (gobj: *mut HSD_GObj));
    // it/it_26B1.c
    hook!(IT_8026B3F8, it_8026B3F8, (article: *mut c_void, kind: i32));
}

pub fn ftGk_Init_UnkMotionStates3(gobj: *mut HSD_GObj) {
    ext::ftKp_Init_UnkMotionStates3(gobj);
}

pub fn ftGk_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let ftdata = (*fp).ft_data;
        let items = (*ftdata).x48_items;

        ext::ftKp_Init_OnLoadForGKoopa(fp);
        ext::it_8026B3F8(*items.add(0), It_Kind_Koopa_Flame as i32);

        (*fp).set_x2226_b1(1);
        (*fp).set_x222A_b0(1);
    }
}

pub fn ftGk_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftGk_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

pub fn ftGk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    ext::ftKp_Init_LoadSpecialAttrs(gobj);
}

/// `ftGk_Init_GetMotionFileString(arg0)`: the Giga Bowser vi-wait demo motion file name.
///
/// The C leaves `offset` uninitialised for every `arg0` other than 15 (its remarks flag the UB);
/// the shipping build compiles with `-ftrivial-auto-var-init=zero`, so `offset - 14` is -14 and
/// the lookup reads before the table. That is not representable: `None`.
pub fn ftGk_Init_GetMotionFileString(arg0: i32) -> Option<&'static str> {
    let offset: i32 = match arg0 {
        15 => 14,
        _ => return None,
    };
    Some(ftGk_Init_803D3984[(offset - 14) as usize])
}
