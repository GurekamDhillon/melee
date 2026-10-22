//! Native port of src/melee/gm/gm_1736.c: the challenger-approach data and the special-message
//! unlock bookkeeping. Verified against the C by tests/tu_gm_1736.rs (state oracle).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::ext_hook::memzero;
use crate::ext_fns;
use core::ffi::c_void;
use melee_types::*;

mod data;
pub use data::{lbl_803D6450, lbl_803D646C};

/// `static struct lbl_803D6450_t` (gm_1736.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803D6450_t {
    pub x0: u8,
    pub x1: u8,
    pub x2: u16,
}

/// `static struct lbl_803D646C_t` (gm_1736.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_803D646C_t {
    pub x0: u16,
    pub x2: u16,
}

/// `static ChallengerData challenger_data`.
pub static mut challenger_data: ChallengerData = unsafe { core::mem::zeroed() };

/// Callees in TUs that are not ported yet.
pub mod ext {
    use super::*;
    ext_fns! {
        fn Mex_Enabled(a0: *const i8) -> i32;
        fn Toy_80305918(a0: i8, a1: i32, a2: i32);
        fn fn_80162CCC() -> i32;
        fn fn_80164B48() -> i32;
        fn fn_80172C78(a0: i32) -> i32 = crate::tu::gm_16f1::fn_80172C78;
        fn fn_8017DEC8(a0: i32) -> *mut Unk1PData = crate::tu::gm_17c0::fn_8017DEC8;
        fn gmMainLib_8015D084(a0: u8) -> *mut i32;
        fn gmMainLib_8015D450(a0: u8) -> *mut i32;
        fn gmMainLib_8015D640() -> i32;
        fn gmMainLib_8015D94C(a0: u32) -> i32;
        fn gmMainLib_8015DA40(a0: u32);
        fn gmMainLib_8015DA68(a0: u32);
        fn gmMainLib_8015DADC(a0: u32) -> i32;
        fn gmMainLib_8015EDBC() -> *mut gmm_retval_EDBC;
        fn gmMainLib_8015EE90() -> i32;
        fn gm_80160474(a0: CharacterKind, a1: GameModeKind) -> i32;
        fn gm_80162EC8() -> i32;
        fn gm_80164ABC() -> i32;
        fn gm_801721EC() -> i32 = crate::tu::gm_16f1::gm_801721EC;
        fn gm_801BEBC0(a0: u8) -> u8;
        fn gm_CKindToSelKind(a0: u8) -> u8;
        fn gm_GetAdventureData() -> *mut UnkAdventureData = crate::tu::gm_17e4::gm_GetAdventureData;
        fn gm_GetCurrentGameMode() -> u8;
        fn gm_GetPreviousGameMode() -> u8;
        fn gm_GetVsPlayMatchTotal() -> u32;
        fn gm_SelKindToCKind(a0: u8) -> u8;
        fn gm_SetGameModeStateId(a0: u8);
        fn gm_SetNewGameModePending();
        fn gm_SetPendingGameMode(a0: u8);
        fn lbLang_GetLanguageSetting() -> i32;
        fn lbTime_8000AEC8(a0: u32, a1: u32) -> u32;
        fn lbTime_8000AEE4(a0: u32, a1: i32) -> u32;
        fn memzero(a0: *mut c_void, a1: i32) = super::memzero;
        fn un_803045A0() -> i32;
        fn un_80304690() -> i32;
        fn un_80304780() -> i32;
    }
}

const LANG_JP: i32 = 0;

pub unsafe fn gm_GetChallengerData() -> *mut ChallengerData {
    core::ptr::addr_of_mut!(challenger_data)
}

pub unsafe fn gm_InitChallengerData(
    human_ckind: u8,
    human_color: u8,
    human_slot: u8,
    human_nametag: u8,
    cpu_ckind: u8,
    curr_mode: u8,
) {
    let tmp = core::ptr::addr_of_mut!(challenger_data);
    ext::memzero(tmp as *mut c_void, core::mem::size_of::<ChallengerData>() as i32);
    (*tmp).human_ckind = human_ckind;
    (*tmp).human_color = human_color;
    (*tmp).human_slot = human_slot;
    (*tmp).human_nametag = human_nametag;
    (*tmp).cpu_ckind = cpu_ckind;
    (*tmp).curr_mode = curr_mode;
}

pub unsafe fn gm_80173754(game_mode: u8, arg1: u8) -> i32 {
    // m-ex "Spoof No Pending Messages 2" (opt-in MELEE_MEX=no_pending_messages_2)
    if ext::Mex_Enabled(b"no_pending_messages_2\0".as_ptr() as *const i8) != 0 {
        return 0;
    }
    if ext::gm_801721EC() != 0 {
        ext::memzero(
            core::ptr::addr_of_mut!(challenger_data) as *mut c_void,
            core::mem::size_of::<ChallengerData>() as i32,
        );
        challenger_data.human_ckind = ChKind_None as u8;
        challenger_data.human_slot = arg1;
        challenger_data.curr_mode = game_mode;
        ext::gm_SetPendingGameMode(GM_CHALLENGER_APPROACH as u8);
        ext::gm_SetNewGameModePending();
        return 1;
    }
    0
}

pub unsafe fn gm_801737D8() -> u8 {
    challenger_data.prev_mode
}

pub unsafe fn gm_Mode_ChallengerApproach_OnLoad() {
    challenger_data.prev_mode = ext::gm_GetPreviousGameMode();
    if challenger_data.human_ckind == ChKind_None as u8 {
        ext::gm_SetGameModeStateId(2);
    } else {
        ext::gm_SetGameModeStateId(0);
    }
}

pub unsafe fn fn_80173834(ckind: u8, major: u8, arg2: i32) {
    // the C keeps the raw int result of gm_80160474 in a `bool` (int) and passes it on unchanged
    let temp_r30 = ext::gm_80160474(ckind as CharacterKind, major as GameModeKind);

    if arg2 != 0 {
        ext::gmMainLib_8015DA68(temp_r30 as u32);
    }
    ext::fn_80172C78(temp_r30);
    if ckind as i32 == CKind_Zelda as i32 {
        let temp_r31 = ext::gm_80160474(CKind_Seak, major as GameModeKind);
        if arg2 != 0 {
            ext::gmMainLib_8015DA68(temp_r31 as u32);
        }
        ext::fn_80172C78(temp_r31);
    }
    if ckind as i32 == CKind_Seak as i32 {
        let temp_r31 = ext::gm_80160474(CKind_Zelda, major as GameModeKind);
        if arg2 != 0 {
            ext::gmMainLib_8015DA68(temp_r31 as u32);
        }
        ext::fn_80172C78(temp_r31);
    }
}

pub unsafe fn gm_8017390C(arg0: i32, arg1: i32) {
    match arg0 {
        0 => {
            let temp_r3 = ext::gm_GetAdventureData();
            if (*temp_r3).x76 != 0 {
                ext::fn_80172C78(0xE7);
            }
            if arg1 != 0 {
                fn_80173834((*temp_r3).x0.x0.ckind as u8, 4, 1);
                if (*temp_r3).x0.xC.xD == 0 && (*temp_r3).x77 != 0 {
                    ext::fn_80172C78(0x51);
                }
                if (*temp_r3).x0.xC.xD == 0 && (*temp_r3).x0.x0.cpu_level >= 3 {
                    ext::fn_80172C78(0x53);
                }
                if (*temp_r3).x0.xC.x20 < 0x101D0 {
                    ext::fn_80172C78(0xE3);
                }
            }
        }
        1 => {
            if arg1 != 0 {
                let temp_r3_2 = ext::fn_8017DEC8(1);
                fn_80173834((*temp_r3_2).x0.ckind as u8, 3, 1);
                if (*temp_r3_2).xC.xD == 0 && (*temp_r3_2).x0.cpu_level >= 3 {
                    ext::fn_80172C78(0x52);
                }
                if (*temp_r3_2).xC.x20 < 0x4650 {
                    ext::fn_80172C78(0x11A);
                }
            }
        }
        2 => {
            if arg1 != 0 {
                let temp_r3_3 = ext::fn_8017DEC8(2);
                fn_80173834((*temp_r3_3).x0.ckind as u8, 5, 1);
                ext::fn_80172C78(0xBE);
                if (*temp_r3_3).xC.xD == 0 {
                    ext::fn_80172C78(0xCC);
                }
                if (*temp_r3_3).x0.cpu_level >= 3 {
                    ext::fn_80172C78(0x86);
                }
            }
        }
        _ => {}
    }
}

pub unsafe fn gm_80173AA4() {
    let mut var_r30: u32 = 0;

    if ext::gmMainLib_8015D640() != 0 {
        ext::fn_80172C78(0xBC);
        for i in 0..0x19 {
            var_r30 = var_r30.wrapping_add(*ext::gmMainLib_8015D450(i as u8) as u32);
        }
        if var_r30 < 0x15F90 {
            ext::fn_80172C78(0x9B);
        }
        if var_r30 < 0xAFC8 {
            ext::fn_80172C78(0x122);
        }
    }
}

pub unsafe fn gm_80173B30(arg0: u32) {
    let mut var_r31: u32 = 0;

    if arg0 >= 0x7530 {
        ext::fn_80172C78(0xBD);
    }
    if arg0 >= 0xAFC8 {
        ext::fn_80172C78(0xCB);
    }
    for i in 0..0x19 {
        var_r31 = ext::lbTime_8000AEE4(var_r31, *ext::gmMainLib_8015D084(i as u8));
    }
    if var_r31 >= 0x7A120 {
        ext::fn_80172C78(0x8D);
    }
}

pub unsafe fn gm_80173BC4(ckind: i8) {
    let mut var_r29: i32 = 0;
    let temp_r28 = (*ext::gmMainLib_8015EDBC()).x114[ext::gm_CKindToSelKind(ckind as u8) as usize];
    for selkind in 0..SELKIND_COUNT as usize {
        var_r29 = var_r29.wrapping_add((*ext::gmMainLib_8015EDBC()).x114[selkind]);
    }
    if temp_r28 >= 0xA {
        ext::fn_80172C78(0x6D);
    }
    if temp_r28 >= 0x14 {
        ext::fn_80172C78(0x5B);
    }
    if var_r29 >= 0x7D {
        ext::fn_80172C78(0x6E);
    }
}

pub unsafe fn gm_80173C70(_c_kind: i8, arg1: u32, arg2: u32, arg3: i32) {
    if arg3 != 0 {
        if ext::gm_GetCurrentGameMode() as i32 == GM_15MIN_VS as i32 {
            ext::fn_80172C78(0x50);
        }
        if ext::gm_GetCurrentGameMode() as i32 == GM_100MAN_VS as i32 && arg2 <= 0x3840 {
            ext::fn_80172C78(0x4E);
        }
        if ext::gm_GetCurrentGameMode() as i32 == GM_15MIN_VS as i32 {
            ext::fn_80172C78(0x50);
        }
    }
    if ext::gm_GetCurrentGameMode() as i32 == GM_ENDLESS_VS as i32 && arg1 >= 100 {
        ext::fn_80172C78(0x4F);
    }
    if ext::gm_GetCurrentGameMode() as i32 == GM_CRUEL_VS as i32 && arg1 >= 5 {
        ext::fn_80172C78(0x10E);
    }
}

/// `static inline int inline0(int arg0)`: unlock id for the stage-language table row matching `arg0`.
pub unsafe fn inline0(arg0: i32) -> i32 {
    let r30 = &lbl_803D6450;
    for i in 0..r30.len() {
        if r30[i].x1 == 2 || r30[i].x1 as i32 == ext::lbLang_GetLanguageSetting() {
            if ext::gm_801BEBC0(r30[i].x0) == ext::gm_801BEBC0(arg0 as u8) {
                return r30[i].x2 as i32;
            }
        }
    }
    -1
}

pub unsafe fn gm_80173D3C(arg0: i32) {
    let var_r0 = inline0(arg0);
    if var_r0 != -1 {
        ext::fn_80172C78(var_r0);
    }
}

pub unsafe fn gm_80173DE4(arg0: *mut MatchEnd) {
    let temp_r30 = (*ext::gmMainLib_8015EDBC()).x0 as u32;

    for i in 0..lbl_803D646C.len() {
        if temp_r30 >= lbl_803D646C[i].x0 as u32 {
            ext::fn_80172C78(lbl_803D646C[i].x2 as i32);
        }
    }
    if (*ext::gmMainLib_8015EDBC()).x4 >= 100 {
        ext::fn_80172C78(0x76);
    }
    if (*ext::gmMainLib_8015EDBC()).xC >= 1000 {
        ext::fn_80172C78(0xE2);
    }
    let mut var_r29_2: u32 = 0;

    for i in 0..6 {
        if (*arg0).player_standings[i].pkind == 0 {
            var_r29_2 = ext::lbTime_8000AEC8(var_r29_2, (*arg0).player_standings[i].x50);
        }
    }
    if ext::lbTime_8000AEC8(var_r29_2, (*ext::gmMainLib_8015EDBC()).x10 as u32) >= 0xF4240 {
        ext::fn_80172C78(0xB0);
    }
}

#[inline]
pub unsafe fn gm_80173EEC_inline(arg0: u8, arg1: i32) -> i32 {
    ext::gm_80160474(arg0 as CharacterKind, arg1 as GameModeKind)
}

pub unsafe fn gm_80173EEC() {
    // m-ex "Disable Special Messages" (opt-in MELEE_MEX=no_special_messages)
    if ext::Mex_Enabled(b"no_special_messages\0".as_ptr() as *const i8) != 0 {
        return;
    }

    for selkind in 0..SELKIND_COUNT as usize {
        let temp_r29 = (*ext::gmMainLib_8015EDBC()).x18[selkind];
        if temp_r29 >= 100 {
            let ckind = ext::gm_SelKindToCKind(selkind as u8);
            ext::fn_80172C78(gm_80173EEC_inline(ckind, GM_CLASSIC as i32));
            if ckind as i32 == CKind_Zelda as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Seak as u8, GM_CLASSIC as i32));
            }
            if ckind as i32 == CKind_Seak as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Zelda as u8, GM_CLASSIC as i32));
            }
        }
        if temp_r29 >= 200 {
            let ckind = ext::gm_SelKindToCKind(selkind as u8);
            ext::fn_80172C78(gm_80173EEC_inline(ckind, GM_ADVENTURE as i32));
            if ckind as i32 == CKind_Zelda as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Seak as u8, GM_ADVENTURE as i32));
            }
            if ckind as i32 == CKind_Seak as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Zelda as u8, GM_ADVENTURE as i32));
            }
        }
        if temp_r29 >= 300 {
            let ckind = ext::gm_SelKindToCKind(selkind as u8);
            ext::fn_80172C78(gm_80173EEC_inline(ckind, GM_ALLSTAR as i32));
            if ckind as i32 == CKind_Zelda as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Seak as u8, GM_ALLSTAR as i32));
            }
            if ckind as i32 == CKind_Seak as i32 {
                ext::fn_80172C78(gm_80173EEC_inline(CKind_Zelda as u8, GM_ALLSTAR as i32));
            }
        }
    }

    if ext::fn_80164B48() != 0 {
        ext::fn_80172C78(0xA0);
    }
    if ext::gm_80164ABC() != 0 {
        ext::fn_80172C78(0x9F);
    }
    if ext::gmMainLib_8015EE90() != 0 {
        ext::fn_80172C78(0xDC);
    }
    if (*ext::gmMainLib_8015EDBC()).x14 >= 0x2710 {
        ext::fn_80172C78(0x10C);
    }
    if ext::gmMainLib_8015D94C(0x1A) != 0 {
        ext::fn_80172C78(0x96);
    }
    if ext::un_803045A0() != 0 {
        ext::fn_80172C78(0x116);
    }
    if ext::un_80304690() != 0 {
        ext::fn_80172C78(0xAF);
    }
    if ext::un_80304780() != 0 {
        ext::fn_80172C78(0x100);
    }

    let mut result = true;
    for j in 0..0x100u32 {
        if j == 0x29 {
            continue;
        }
        if j == 0x42 || j == 0x43 {
            continue;
        }
        if j == 0xB9 {
            continue;
        }
        if j == 0xC9 || j == 0xCA {
            continue;
        }
        if j == 9 {
            continue;
        }
        if ext::gmMainLib_8015DADC(j) == 0 {
            result = false;
            break;
        }
    }
    if result {
        ext::fn_80172C78(0x123);
    }
}

pub unsafe fn gm_80174180() {
    if ext::fn_80162CCC() != 0 && ext::gm_80162EC8() != 0 {
        ext::Toy_80305918(4, 0, 0);
    }
    if ext::gm_GetVsPlayMatchTotal() >= 0xC8 {
        ext::Toy_80305918(5, 0, 0);
    }
    if ext::gm_80164ABC() != 0 {
        ext::Toy_80305918(6, 0, 0);
    }
}

pub unsafe fn gm_801741FC() {
    for i in 0..0x12Cu32 {
        ext::gmMainLib_8015DA40(i);
    }
}

pub unsafe fn gm_80174238() {
    for i in 0..0x12Cu32 {
        ext::gmMainLib_8015DA68(i);
    }
}
