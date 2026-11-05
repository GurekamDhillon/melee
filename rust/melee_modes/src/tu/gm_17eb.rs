//! Native port of src/melee/gm/gm_17EB.c: the All-Star stage table readers and the All-Star
//! result-flag setter. Verified against the C by tests/tu_gm_17eb.rs (state oracle).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::ext_fns;
use melee_types::*;

mod data;
pub use data::lbl_803D85F0;

/// `AllstarStageEntry` (gm_17EB.c, size 0x1A).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AllstarStageEntry {
    pub stage_kind: u8,
    pub pad_1: u8,
    /// Divided by 100 to get a ratio.
    pub scale0_pct: u16,
    /// Divided by 100 to get a ratio.
    pub scale1_pct: u16,
    pub pad_6: [u8; 10],
    pub scale2_pct: u16,
    pub scale3_pct: u16,
    pub pad_14: [u8; 6],
}

/// `static u8 lbl_80472CB0[0x78]` (the C casts it to `UnkAllstarData*`; the native record is bigger
/// than 0x78 bytes because of its function pointers, so the storage is the record itself).
pub static mut lbl_80472CB0: UnkAllstarData = unsafe { core::mem::zeroed() };

/// Callees in TUs that are not ported yet.
pub mod ext {
    use super::*;
    ext_fns! {
        fn Player_GetPlayerCharacter(a0: i32) -> CharacterKind = crate::tu::adapt::Player_GetPlayerCharacter__96861a;
        fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = crate::tu::adapt::Player_GetPlayerSlotType__2d05e5;
        fn fn_8017E318() -> i32 = crate::tu::gm_17c0::fn_8017E318;
        fn gmMainLib_8015CDC8() -> *mut gmm_x0_528_t = crate::tu::adapt::gmMainLib_8015CDC8__ddba5a;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gm_GetFrameCount() -> u32 = crate::tu::adapt::gm_GetFrameCount__aa8e1a;
        fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = crate::tu::adapt::gm_GetStartMeleeRules__b73473;
    }
}

#[inline]
unsafe fn entry(difficulty: u8, stage_slot: u8) -> &'static AllstarStageEntry {
    &lbl_803D85F0[stage_slot as usize + difficulty as usize * 5]
}

pub unsafe fn gm_GetAllStarData() -> *mut UnkAllstarData {
    melee_core::ct!("gm_GetAllStarData");
    core::ptr::addr_of_mut!(lbl_80472CB0)
}

pub unsafe fn gm_8017EB3C(difficulty: u8, stage_slot: u8) -> u8 {
    melee_core::ct!("gm_8017EB3C");
    entry(difficulty, stage_slot).stage_kind
}

pub unsafe fn gm_8017EB64(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017EB64");
    entry(difficulty, stage_slot).pad_6[arg2 as usize * 3]
}

pub unsafe fn gm_8017EB98(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017EB98");
    entry(difficulty, stage_slot).pad_6[arg2 as usize * 3 + 1]
}

pub unsafe fn gm_8017EBCC(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017EBCC");
    entry(difficulty, stage_slot).pad_6[arg2 as usize * 3 + 2]
}

pub unsafe fn gm_8017EC00(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8017EC00");
    entry(difficulty, stage_slot).scale2_pct as f32 / 100.0
}

pub unsafe fn gm_8017EC50(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8017EC50");
    entry(difficulty, stage_slot).scale3_pct as f32 / 100.0
}

pub unsafe fn gm_8017ECA0(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017ECA0");
    entry(difficulty, stage_slot).pad_14[arg2 as usize * 3]
}

pub unsafe fn gm_8017ECD4(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017ECD4");
    entry(difficulty, stage_slot).pad_14[arg2 as usize * 3 + 1]
}

pub unsafe fn gm_8017ED08(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    melee_core::ct!("gm_8017ED08");
    entry(difficulty, stage_slot).pad_14[arg2 as usize * 3 + 2]
}

pub unsafe fn gm_8017ED3C(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8017ED3C");
    entry(difficulty, stage_slot).scale0_pct as f32 / 100.0
}

pub unsafe fn gm_8017ED8C(difficulty: u8, stage_slot: u8) -> f32 {
    melee_core::ct!("gm_8017ED8C");
    entry(difficulty, stage_slot).scale1_pct as f32 / 100.0
}

pub unsafe fn fn_8017EDDC() -> i32 {
    melee_core::ct!("fn_8017EDDC");
    if ext::gm_GetCurrentGameMode() as i32 == GM_CLASSIC as i32 {
        let p = &*gm_GetAllStarData();
        if p.x0.xC.xD == 0 && p.x0.x0.cpu_level >= 2 && p.x0.xC.x20 < 0x5208 {
            return 1;
        }
    }
    0
}

/// `((u8_bits*) &arg0->_x448[i])->bN = 1` with the GameCube (MSB-first) bit order: bN is mask 1<<N.
#[inline]
unsafe fn set_flag(arg0: *mut MatchEnd, byte: usize, bit: u32) {
    (*arg0)._x448[byte] |= <u8 as melee_core::ppc::Ppc>::pshl(1, (bit) as i64);
}

/// The C takes the `MatchEnd*` as an `int` (a GameCube pointer); native code takes the pointer.
pub unsafe fn fn_8017EE40(arg0: *mut MatchEnd) {
    melee_core::ct!("fn_8017EE40");
    let rules = ext::gm_GetStartMeleeRules();
    let allstar = gm_GetAllStarData();

    if ext::fn_8017E318() > 0 {
        set_flag(arg0, 2, 3);
    }

    if (*rules).x4_5() != 0 {
        let main_data = ext::gmMainLib_8015CDC8();
        let total_time = (*allstar).x0.xC.x20.wrapping_add(ext::gm_GetFrameCount());
        set_flag(arg0, 0, 7);

        if (*allstar).x0.x0.cpu_level == 4 {
            set_flag(arg0, 0, 4);
        }

        if total_time < 0x3840 {
            set_flag(arg0, 2, 6);
        } else if total_time < 0x5460 {
            set_flag(arg0, 2, 7);
        }

        if (*allstar).x0.xC.x1C as u32 == 0 && (*arg0).player_standings[0].x44 == 0 {
            set_flag(arg0, 1, 1);
        }

        if (*allstar).x0.xC.xE != 0 {
            set_flag(arg0, 2, 5);
        }

        if (*allstar).x0.xC.xF != 0 {
            set_flag(arg0, 2, 4);
        }

        if (*allstar).x0.xC.xD != 0 {
            set_flag(arg0, 1, 0);
        } else if (*arg0).player_standings[0].stocks as i32 == (*main_data).stocks as i32 {
            set_flag(arg0, 1, 2);
        }
    }

    for i in 0..6 {
        if ext::Player_GetPlayerSlotType(i) == Gm_PKind_Cpu
            && ext::Player_GetPlayerCharacter(i) == CKind_CrezyH
        {
            set_flag(arg0, 0, 0);
            return;
        }
    }
}
