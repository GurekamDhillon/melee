//! Game globals that several gm TUs share (defined in TUs not ported yet). Native pointers/values,
//! set by whoever owns the simulation (the oracle tests set them to synthetic state).
#![allow(non_upper_case_globals, static_mut_refs)]
use core::ptr::null_mut;
use melee_types::*;

/// gmmain_lib.c `/* 4D3EE0 */ struct gmm_x0* gmMainLib_804D3EE0` (the save data).
/// One static for the world and mode crates (it used to be two copies; the engine sets the canonical one).
pub use melee_world::gmmain::gmMainLib_804D3EE0;
/// gm_1884.c `/* 473814 */ CssSubStruct gm_80473814`.
pub static mut gm_80473814: CssSubStruct = unsafe { core::mem::zeroed() };

/// gmapproach.c `UNK_T gmVsMelee_ApproachData[2]` (scene data of the approach screen).
pub static mut gmVsMelee_ApproachData: [*mut core::ffi::c_void; 2] = [null_mut(); 2];
/// if_2FD9.c `/* 4A1F48 */ struct un_804A1F48_t if_Scene_Prize_EnterData`.
pub use melee_world::tu::if_2FD9::if_Scene_Prize_EnterData;

/// controller.c `HSD_PadStatus HSD_PadMasterStatus[4]`.
pub use melee_runtime::tu::controller::HSD_PadMasterStatus;
/// ground.c `/* 49E6C8 */ StageInfo stage_info`.
pub use melee_world::tu::ground::stage_info;
/// video.c `HSD_VIInfo HSD_VIData`.
pub use melee_runtime::tu::video::HSD_VIData;
/// ftdata.c `/* 3C0EC0 */ struct UnkCostumeList CostumeListsForeachCharacter[Ft_Kind_Max]`.
pub use melee_fighter::tu::ftdata::CostumeListsForeachCharacter;

/// controller.c `HSD_PadStatus HSD_PadCopyStatus[4]` (the pads as the frame's logic reads them).
pub use melee_runtime::tu::controller::HSD_PadCopyStatus;
/// db.c `/* 4D4A08 */ DbLKind DbLevel`.
pub use melee_runtime::shared::DbLevel;
/// random.c `u32* HSD_RandSeedPtr`.
pub use melee_runtime::tu::random::global::HSD_RandSeedPtr;
/// os `u32 __OSBusClock` (the bus clock in Hz; `OS_TIMER_CLOCK` is a quarter of it).
pub use melee_runtime::tu::perf::__OSBusClock;
