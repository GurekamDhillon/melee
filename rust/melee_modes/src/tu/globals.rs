//! Game globals that several gm TUs share (defined in TUs not ported yet). Native pointers/values,
//! set by whoever owns the simulation (the oracle tests set them to synthetic state).
#![allow(non_upper_case_globals, static_mut_refs)]
use core::ptr::null_mut;
use melee_types::*;

/// gmmain_lib.c `/* 4D3EE0 */ struct gmm_x0* gmMainLib_804D3EE0` (the save data).
pub static mut gmMainLib_804D3EE0: *mut gmm_x0 = null_mut();
/// gm_1884.c `/* 473814 */ CssSubStruct gm_80473814`.
pub static mut gm_80473814: CssSubStruct = unsafe { core::mem::zeroed() };

/// gmapproach.c `UNK_T gmVsMelee_ApproachData[2]` (scene data of the approach screen).
pub static mut gmVsMelee_ApproachData: [*mut core::ffi::c_void; 2] = [null_mut(); 2];
/// if_2FD9.c `/* 4A1F48 */ struct un_804A1F48_t if_Scene_Prize_EnterData`.
pub static mut if_Scene_Prize_EnterData: un_804A1F48_t = unsafe { core::mem::zeroed() };

/// controller.c `HSD_PadStatus HSD_PadMasterStatus[4]`.
pub static mut HSD_PadMasterStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
/// ground.c `/* 49E6C8 */ StageInfo stage_info`.
pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
/// video.c `HSD_VIInfo HSD_VIData`.
pub static mut HSD_VIData: HSD_VIInfo = unsafe { core::mem::zeroed() };
/// ftdata.c `/* 3C0EC0 */ struct UnkCostumeList CostumeListsForeachCharacter[Ft_Kind_Max]`.
pub static mut CostumeListsForeachCharacter: [UnkCostumeList; Ft_Kind_Max as usize] = unsafe { core::mem::zeroed() };

/// controller.c `HSD_PadStatus HSD_PadCopyStatus[4]` (the pads as the frame's logic reads them).
pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
/// db.c `/* 4D4A08 */ DbLKind DbLevel`.
pub static mut DbLevel: DbLKind = 0;
/// random.c `u32* HSD_RandSeedPtr`.
pub static mut HSD_RandSeedPtr: *mut u32 = null_mut();
/// os `u32 __OSBusClock` (the bus clock in Hz; `OS_TIMER_CLOCK` is a quarter of it).
pub static mut __OSBusClock: u32 = 0;
