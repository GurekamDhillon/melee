//! src/melee/gm/gmmenu.c: the OnLoad hooks of the Classic, Adventure, All-Star and Opening
//! game-over/opening modes. They copy the saved character (kind, colour, slot, nametag) into the
//! menu state. Verification: none yet (the callees and the save-record globals are unported).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use melee_types::*;

/// C `gm_Mode_ClassicGOver_OnLoad`.
pub unsafe fn gm_Mode_ClassicGOver_OnLoad() {
    melee_core::ct!("gm_Mode_ClassicGOver_OnLoad");
    let temp = ext::gm_GetAllStarData();
    let var_r0 = if (*temp).x0.x0.ckind == CKind_Zelda as _ && (*temp).x0.xC.x12 != 0 {
        CKind_Seak as u8
    } else {
        (*temp).x0.x0.ckind as u8
    };
    ext::gm_801BEFA4(var_r0 & 0xFF);
    ext::gm_801BEFC0((*temp).x0.x0.color);
    ext::gm_801BF000((*temp).x0.x0.slot);
    ext::gm_801BEFE0((*temp).x0.x0.nametag);
    ext::gm_801BF020(1);
}

/// C `gm_Mode_AdventureGOver_OnLoad`.
pub unsafe fn gm_Mode_AdventureGOver_OnLoad() {
    melee_core::ct!("gm_Mode_AdventureGOver_OnLoad");
    let temp = ext::gm_GetAdventureData();
    let var_r0 = if (*temp).x0.x0.ckind == CKind_Zelda as _ && (*temp).x0.xC.x12 != 0 {
        CKind_Seak as u8
    } else {
        (*temp).x0.x0.ckind as u8
    };
    ext::gm_801BEFA4(var_r0 & 0xFF);
    ext::gm_801BEFC0((*temp).x0.x0.color);
    ext::gm_801BF000((*temp).x0.x0.slot);
    ext::gm_801BEFE0((*temp).x0.x0.nametag);
    ext::gm_801BF020(0);
}

/// C `gm_Mode_AllstarGOver_OnLoad` (the All-Star record is the file-scope `gm_80473A18`).
pub unsafe fn gm_Mode_AllstarGOver_OnLoad() {
    melee_core::ct!("gm_Mode_AllstarGOver_OnLoad");
    let tmp = ext::gm_80473A18();
    let var_r0 = if (*tmp).x0.x0.ckind == CKind_Zelda as _ && (*tmp).x0.xC.x12 != 0 {
        CKind_Seak as u8
    } else {
        (*tmp).x0.x0.ckind as u8
    };
    ext::gm_801BEFA4(var_r0 & 0xFF);
    ext::gm_801BEFC0((*tmp).x0.x0.color);
    ext::gm_801BF000((*tmp).x0.x0.slot);
    ext::gm_801BEFE0((*tmp).x0.x0.nametag);
    ext::gm_801BF020(2);
}

/// C `gm_Mode_Opening_OnLoad`.
pub unsafe fn gm_Mode_Opening_OnLoad() {
    melee_core::ct!("gm_Mode_Opening_OnLoad");
    ext::gm_SetGameModeStateId(ext::gm_801BF718());
}

/// Callees in TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut GM_GETALLSTARDATA: Option<fn() -> *mut UnkAllstarData> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetAllStarData() -> *mut UnkAllstarData {
                match unsafe { GM_GETALLSTARDATA } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_17eb::gm_GetAllStarData() as _ }; },
                }
            }
    pub static mut GM_GETADVENTUREDATA: Option<fn() -> *mut UnkAdventureData> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetAdventureData() -> *mut UnkAdventureData {
                match unsafe { GM_GETADVENTUREDATA } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_17e4::gm_GetAdventureData() as _ }; },
                }
            }
    hook!(GM_80473A18, gm_80473A18, () -> *mut UnkAllstarData);
    pub static mut GM_801BEFA4: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BEFA4(v: u8)  {
                match unsafe { GM_801BEFA4 } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { crate::tu::gmgover::gm_801BEFA4(v as _); } return; },
                }
            }
    pub static mut GM_801BEFC0: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BEFC0(v: u8)  {
                match unsafe { GM_801BEFC0 } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { crate::tu::gmgover::gm_801BEFC0(v as _); } return; },
                }
            }
    pub static mut GM_801BF000: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF000(v: u8)  {
                match unsafe { GM_801BF000 } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { crate::tu::gmgover::gm_801BF000(v as _); } return; },
                }
            }
    pub static mut GM_801BEFE0: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BEFE0(v: u8)  {
                match unsafe { GM_801BEFE0 } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { crate::tu::gmgover::gm_801BEFE0(v as _); } return; },
                }
            }
    pub static mut GM_801BF020: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF020(v: i32)  {
                match unsafe { GM_801BF020 } {
                    Some(f) => unsafe { f(v) },
                    None => { unsafe { crate::tu::gmgover::gm_801BF020(v as _); } return; },
                }
            }
    pub static mut GM_801BF718: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF718() -> i32 {
                match unsafe { GM_801BF718 } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gmopeningmode::gm_801BF718() as _ }; },
                }
            }
    pub static mut GM_SETGAMEMODESTATEID: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetGameModeStateId(id: i32)  {
                match unsafe { GM_SETGAMEMODESTATEID } {
                    Some(f) => unsafe { f(id) },
                    None => { unsafe { crate::tu::gm_1A3F::gm_SetGameModeStateId(id as _); } return; },
                }
            }
}
