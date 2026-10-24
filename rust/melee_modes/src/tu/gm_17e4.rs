//! Native port of src/melee/gm/gm_17E4.c: the Adventure-mode data (stage table readers, rumble
//! slot, scene counting, match-end bonus flags). Verified against the C by tests/tu_gm_17e4.rs.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::ext_fns;
use melee_types::*;

mod data;
pub use data::lbl_803D7AC0;

/// `AdventureStageEntry` (gm_17E4.c, size 0x1A).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AdventureStageEntry {
    pub stage_kind: u8,
    pub pad_1: u8,
    /// Divided by 100 to get a ratio.
    pub scale0_pct: u16,
    /// Divided by 100 to get a ratio.
    pub scale1_pct: u16,
    pub pad_6: [u8; 20],
}

/// `static UnkAdventureData lbl_80472C30`.
pub static mut lbl_80472C30: UnkAdventureData = unsafe { core::mem::zeroed() };

/// Callees (and data) in TUs that are not ported yet.
pub mod ext {
    use super::*;
    /// gm_unsplit data: the Adventure scene-state table (49 entries).
    pub static mut gm_Mode_Adventure_States: [GameModeState; 49] = unsafe { core::mem::zeroed() };
    /// gm_unsplit data: per-scene records, terminated by `x0 == 0xFF` (46 entries).
    pub static mut gm_803DE650: [gm_803DE650_t; 46] = unsafe { core::mem::zeroed() };
    ext_fns! {
        fn Player_GetPlayerCharacter(a0: i32) -> CharacterKind = crate::tu::adapt::Player_GetPlayerCharacter__96861a;
        fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = crate::tu::adapt::Player_GetPlayerSlotType__2d05e5;
        fn Player_LoseStock(a0: i32) = crate::tu::adapt::Player_LoseStock__6785b0;
        fn fn_8017E318() -> i32 = crate::tu::gm_17c0::fn_8017E318;
        fn gmMainLib_8015CDD4() -> *mut gmm_x0_528_t = crate::tu::adapt::gmMainLib_8015CDD4__51396a;
        fn gmVs_GetSceneController() -> *mut VsSceneController = crate::tu::adapt::gmVs_GetSceneController__400cec;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gm_GetCurrentSceneIndex() -> u8 = crate::tu::adapt::gm_GetCurrentSceneIndex__5f49a4;
        fn gm_GetFrameCount() -> u32 = crate::tu::adapt::gm_GetFrameCount__aa8e1a;
        fn gm_GetStartMeleeRules() -> *mut StartMeleeRules = crate::tu::adapt::gm_GetStartMeleeRules__b73473;
        fn gm_RumbleEnabledForPlayer(a0: i32, a1: i32) -> i32 = crate::tu::adapt::gm_RumbleEnabledForPlayer__aa5085;
        fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
        fn ifStatus_802F6948(a0: i32) = crate::tu::adapt::ifStatus_802F6948__0cc861;
        fn ifStatus_802F6E3C(a0: i32) = crate::tu::adapt::ifStatus_802F6E3C__cdcf00;
    }
}

pub unsafe fn gm_GetAdventureData() -> *mut UnkAdventureData {
    core::ptr::addr_of_mut!(lbl_80472C30)
}

pub unsafe fn gm_8017E430() -> u8 {
    (*gm_GetAdventureData()).x0.x0.slot
}

pub unsafe fn gm_8017E440() -> u8 {
    let r31 = gm_GetAdventureData();
    if ext::gm_RumbleEnabledForPlayer((*r31).x0.x0.slot as i32, (*r31).x0.x0.nametag as i32) == 0 {
        return 4;
    }
    (*r31).x0.x0.slot
}

/// Number of VS scenes before `scene` in the Adventure state table.
pub unsafe fn gm_8017E48C(scene: *mut GameModeState) -> u8 {
    let mut count: u8 = 0;
    let mut i = 0usize;
    while (*scene).id != ext::gm_Mode_Adventure_States[i].id {
        if ext::gm_Mode_Adventure_States[i].info.scene_kind as i32 == GS_VS as i32 {
            count = count.wrapping_add(1);
        }
        i += 1;
    }
    count
}

/// The `gm_803DE650` record for scene index `arg0`, or NULL.
pub unsafe fn gm_8017E4C4(arg0: u8) -> *mut gm_803DE650_t {
    let mut ptr = core::ptr::addr_of_mut!(ext::gm_803DE650) as *mut gm_803DE650_t;
    while (*ptr).x0 != 0xFF {
        if (*ptr).x0 == arg0 {
            return ptr;
        }
        ptr = ptr.add(1);
    }
    core::ptr::null_mut()
}

#[inline]
unsafe fn entry(difficulty: u8, stage_slot: u8) -> &'static AdventureStageEntry {
    &lbl_803D7AC0[stage_slot as usize + difficulty as usize * 5]
}

/// Adventure stage kind for the given difficulty and stage slot.
pub unsafe fn gm_8017E500(difficulty: u8, stage_slot: u8) -> u8 {
    entry(difficulty, stage_slot).stage_kind
}

pub unsafe fn gm_8017E528(difficulty: u8, stage_slot: u8) -> f32 {
    entry(difficulty, stage_slot).scale0_pct as f32 / 100.0
}

pub unsafe fn gm_8017E578(difficulty: u8, stage_slot: u8) -> f32 {
    entry(difficulty, stage_slot).scale1_pct as f32 / 100.0
}

pub unsafe fn gm_8017E5C8(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[arg2 as usize * 3]
}

pub unsafe fn gm_8017E5FC(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[1 + arg2 as usize * 3]
}

pub unsafe fn gm_8017E630(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[arg2 as usize * 3 + 2]
}

pub unsafe fn gm_8017E664(difficulty: u8, stage_slot: u8) -> f32 {
    entry(difficulty, stage_slot).scale0_pct as f32 / 100.0
}

pub unsafe fn gm_8017E6B4(difficulty: u8, stage_slot: u8) -> f32 {
    entry(difficulty, stage_slot).scale1_pct as f32 / 100.0
}

pub unsafe fn gm_8017E704(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[0x0E + arg2 as usize * 3]
}

pub unsafe fn gm_8017E738(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[0xF + arg2 as usize * 3]
}

pub unsafe fn gm_8017E76C(difficulty: u8, stage_slot: u8, arg2: u8) -> u8 {
    entry(difficulty, stage_slot).pad_6[0x10 + arg2 as usize * 3]
}

pub unsafe fn gm_8017E7A0(match_result: u8) {
    if match_result as i32 == OUTCOME_TIMEOUT as i32 {
        ext::Player_LoseStock(0);
        ext::ifStatus_802F6948(0);
        ext::ifStatus_802F6E3C(0);
    }
}

pub unsafe fn gm_8017E7E0() -> i32 {
    ((*gm_GetAdventureData()).x7C == 0x14) as i32
}

pub unsafe fn gm_8017E7FC(match_result: u8) {
    let r31 = gm_GetAdventureData();
    let cond = ext::gm_GetCurrentGameMode() as i32 == GM_ADVENTURE as i32
        && (*r31).x0.x0.cpu_level >= 2
        && (*r31).x0.xC.x20.wrapping_add(ext::gm_GetFrameCount()) < 0xFD20;

    if !cond {
        let rules = ext::gm_GetStartMeleeRules();
        (*rules).set_x4_5(1);
        (*r31).x77 = 0;
        ext::gm_SetNextGameModeStateId(0x5A);
    } else {
        (*r31).x77 = 1;
        ext::gm_SetNextGameModeStateId(0x5A);
    }
}

/// `((u8_bits*) &flags[i])->bN = 1` with the GameCube (MSB-first) bit order: bN is mask 1<<N.
#[inline]
unsafe fn set_flag(flags: *mut u8, byte: usize, bit: u32) {
    *flags.add(byte) |= 1 << bit;
}

/// The C takes the `MatchEnd*` as an `int` (a GameCube pointer); native code takes the pointer.
pub unsafe fn fn_8017E8A4(arg0: *mut MatchEnd) {
    let gm = ext::gmVs_GetSceneController();
    let stage = gm_8017E4C4(ext::gm_GetCurrentSceneIndex());
    let adv = core::ptr::addr_of_mut!(lbl_80472C30);
    let flags = (*arg0)._x448.as_mut_ptr();

    if ext::fn_8017E318() > 0 {
        set_flag(flags, 2, 3);
    }

    if (*gm).start.x4_5() != 0 {
        let main_data = ext::gmMainLib_8015CDD4();
        let total_time = (*adv).x0.xC.x20.wrapping_add(ext::gm_GetFrameCount());
        set_flag(flags, 0, 6);

        if (*adv).x0.x0.cpu_level == 4 {
            set_flag(flags, 0, 3);
        }

        if total_time < 0xC4E0 {
            set_flag(flags, 2, 6);
        } else if total_time < 0x13560 {
            set_flag(flags, 2, 7);
        }

        if (*adv).x0.xC.x1C as u32 == 0 && (*arg0).player_standings[0].x44 == 0 {
            set_flag(flags, 1, 1);
        }

        if (*adv).x0.xC.xE != 0 {
            set_flag(flags, 2, 5);
        }

        if (*adv).x0.xC.xF != 0 {
            set_flag(flags, 2, 4);
        }

        if (*adv).x0.xC.xD != 0 {
            set_flag(flags, 1, 0);
        } else if (*arg0).player_standings[0].stocks as i32 == (*main_data).stocks as i32 {
            set_flag(flags, 1, 2);
        }
    }

    if (*stage).x0 == 3 {
        for i in 0..6 {
            if ext::Player_GetPlayerSlotType(i) == Gm_PKind_Cpu
                && ext::Player_GetPlayerCharacter(i) == CKind_Luigi
            {
                set_flag(flags, 1, 7);
                return;
            }
        }
    }

    if (*stage).x0 == 0x11 && (*gm).state.unk_C >= 5 {
        set_flag(flags, 1, 6);
        return;
    }

    if (*stage).x0 == 0x25 {
        set_flag(flags, 1, 5);
        return;
    } else if (*stage).x0 == 0x53 {
        for i in 0..6 {
            if ext::Player_GetPlayerSlotType(i) == Gm_PKind_Cpu
                && ext::Player_GetPlayerCharacter(i) == CKind_Luigi
            {
                set_flag(flags, 1, 4);
                return;
            }
        }
    }

    if (*stage).x0 == 0x5C {
        set_flag(flags, 1, 3);
    }
}
