//! Port of src/melee/gm/gmgover.c (the game-over / regular-end scene tables, the challenger
//! hand-off on leaving the congratulations scene, and the 16-byte `gm_8049C178` record the ending
//! scenes read: character, costume, stock count, ...). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_gmgover.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use super::support::*;
use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn gm_80164430(a0: u16) -> i32 = crate::tu::adapt::gm_80164430__8808be;
    fn gm_80164504(a0: u16) = crate::tu::adapt::gm_80164504__95f551;
    fn gm_80172898(a0: u16) -> i32 = crate::tu::adapt::gm_80172898__ee73a3;
    fn gm_80173224(a0: i32, a1: i32) -> u8 = crate::tu::adapt::gm_80173224__600622;
    fn gm_80173754(a0: u8, a1: u8) -> i32 = crate::tu::adapt::gm_80173754__f83272;
    fn gm_8017390C(a0: i32, a1: i32) = crate::tu::adapt::gm_8017390C__4e0f3c;
    fn gm_80173EEC() = crate::tu::adapt::gm_80173EEC__58a22d;
    fn gm_InitChallengerData(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) = crate::tu::adapt::gm_InitChallengerData__c5db86;
    fn gm_SetNewGameModePending() = crate::tu::adapt::gm_SetNewGameModePending__1e5e48;
    fn gm_SetPendingGameMode(a0: u8) = crate::tu::adapt::gm_SetPendingGameMode__90d26d;
    fn lbMthp_8001F800() = crate::tu::adapt::lbMthp_8001F800__fe22a6;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `static u8 gm_8049C178[16]` (49C178).
pub static mut gm_8049C178: [u8; 16] = [0; 16];
/// `static UNK_T gm_804D6920[2]` (4D6920).
pub static mut gm_804D6920: [*mut c_void; 2] = [core::ptr::null_mut(); 2];

/// `gm_Mode_GOver_States`
pub static mut gm_Mode_GOver_States: [GameModeState; 5] = [
    state(0, lbDvdPreload_2, None, None, GS_REGEND_TOYFALL, NULL, unsafe { &raw mut gm_804D6920 as *mut c_void }),
    state(1, lbDvdPreload_2, None, None, GS_STAFFROLL, NULL, NULL),
    state(2, lbDvdPreload_2, None, Some(gm_801BEF84), GS_MOVIE_END, NULL, NULL),
    state(3, lbDvdPreload_2, None, Some(gm_801BEE9C), GS_REGEND_CONGRATS, NULL, unsafe { &raw mut gm_804D6920 as *mut c_void }),
    end(),
];

/// `gm_Mode_DebugGOver_States`
pub static mut gm_Mode_DebugGOver_States: [GameModeState; 3] = [
    state(0, lbDvdPreload_2, None, None, GS_REGEND_TOYFALL, NULL, unsafe { &raw mut gm_804D6920 as *mut c_void }),
    state(1, lbDvdPreload_2, None, Some(gm_801BEE9C), GS_REGEND_CONGRATS, NULL, unsafe { &raw mut gm_804D6920 as *mut c_void }),
    end(),
];

/// 1BEE9C
pub unsafe extern "C" fn gm_801BEE9C(arg0: *mut GameModeState) {
    let game_mode = (*arg0).info.exit_data as *mut i8;
    let ckind = ext::gm_80173224(gm_801BF030(), 1);
    if gm_801BEFB0() == CKind_GameWatch && ext::gm_80164430(0x1B) == 0 {
        ext::gm_80164504(0x1B);
    }
    ext::gm_8017390C(gm_801BF030(), 1);
    ext::gm_80173EEC();
    ext::gm_80172898(0x40);
    if ckind as u32 == ChKind_None {
        if ext::gm_80173754(1, gm_801BEFD0() as u8) == 0 {
            ext::gm_SetPendingGameMode(*game_mode as u8);
        }
    } else {
        ext::gm_InitChallengerData(
            gm_801BEFB0() as u8,
            gm_801BEFD0() as u8,
            gm_801BF010() as u8,
            gm_801BEFF0() as u8,
            ckind,
            *game_mode as u8,
        );
        ext::gm_SetPendingGameMode(GM_CHALLENGER_APPROACH as u8);
    }
    ext::gm_SetNewGameModePending();
}

/// 1BEF84
pub unsafe extern "C" fn gm_801BEF84(_arg: *mut GameModeState) {
    ext::lbMthp_8001F800();
}

/// 1BEFA4
pub fn gm_801BEFA4(ckind: i32) {
    unsafe { gm_8049C178[0] = ckind as u8 }
}

/// 1BEFB0
pub fn gm_801BEFB0() -> CharacterKind {
    unsafe { gm_8049C178[0] as CharacterKind }
}

/// 1BEFC0
pub fn gm_801BEFC0(arg0: i32) {
    unsafe { gm_8049C178[1] = arg0 as u8 }
}

/// 1BEFD0
pub fn gm_801BEFD0() -> i32 {
    unsafe { gm_8049C178[1] as i32 }
}

/// 1BEFE0
pub fn gm_801BEFE0(arg0: i8) {
    unsafe { gm_8049C178[0xA] = arg0 as u8 }
}

/// 1BEFF0
pub fn gm_801BEFF0() -> i32 {
    unsafe { gm_8049C178[0xA] as i32 }
}

/// 1BF000
pub fn gm_801BF000(arg0: i8) {
    unsafe { gm_8049C178[9] = arg0 as u8 }
}

/// 1BF010
pub fn gm_801BF010() -> i32 {
    unsafe { gm_8049C178[9] as i32 }
}

/// 1BF020
pub fn gm_801BF020(arg0: i8) {
    unsafe { gm_8049C178[8] = arg0 as u8 }
}

/// 1BF030
pub fn gm_801BF030() -> i32 {
    unsafe { gm_8049C178[8] as i32 }
}

/// 1BF040
pub fn gm_801BF040(arg0: i8) {
    unsafe { gm_8049C178[2] = arg0 as u8 }
}

/// 1BF050
pub fn gm_801BF050() -> i32 {
    unsafe { gm_8049C178[2] as i32 }
}
