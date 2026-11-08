//! Port of src/melee/pl/player.c: the player table (`player_slots`), per-slot accessors, fighter
//! (entity) creation / teardown for a slot, and the TARGET_PC m-ex mapping setters.
//!
//! Data: `player_slots` and `ftMapping_list` are the crate's existing statics
//! (`crate::stale::PLAYER_SLOTS`, `crate::player::FT_MAPPING_LIST`), re-exported under the C names so
//! there is a single copy. `Player_CheckSlot` is HSD_ASSERTREPORT + `__assert`: an out-of-range
//! slot panics. Callees of unported TUs are `ext` hooks named like the C (unset: panics "TODO").
//! No float arithmetic (so no fused sites). `Player_SetFlagsAEBit1` is declared `u8` in C but has
//! no `return` (garbage in r3): it returns nothing here.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::stale::{player};
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_world::pl::ftMapping_list as FT_MAPPING_LIST;
#[allow(unused_imports)]
use melee_world::pl::FT_MAPPING_INIT;
#[allow(unused_imports)]
use crate::player::{ent as ent_player, get_player_state, has_extra_fighter_id, human_or_cpu, mapping, poses};
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use crate::stale::PLAYER_NUM_MAX;
use melee_types::*;

pub use crate::player::FT_MAPPING_LIST as ftMapping_list;
pub use crate::stale::FtMapping;
/// The plain table accessors live in `melee_world::pl` (the world TUs call them too).
pub use melee_world::pl::{
    Player_GetPlayerSlotType,
    Player_GetEntity,
    Player_LoadPlayerCoords,
    Player_GetTeam,
    Player_GetStocks,
    Player_GetEntityAtIndex,
    Player_GetPlayerCharacter,
    Player_GetPlayerId,
    Player_GetNametagSlotID,
    Player_GetPlayerState,
    Player_GetDamage,
    Player_GetMoreFlagsBit2,
    Player_GetRemainingHP,
    Player_800325C8,
    Player_GetCoins,
};
pub use crate::stale::PLAYER_SLOTS as player_slots;

// BEGIN EXT (callees of the C TU that live in unported TUs)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? #[allow(unreachable_code)] { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) } }
                }
            } )*
        };
    }
    ext_fns! {
    fn Fighter_Create(a0: *mut plAllocInfo) -> *mut HSD_GObj => unsafe { crate::tu::fighter::Fighter_Create(&*(a0 as *const _)) as _ };
    fn Fighter_FirstInitialize_80067A84() => unsafe { crate::tu::fighter::Fighter_FirstInitialize_80067A84(); };
    fn GenoDefine_InitKinds() => unsafe { let _ = crate::platform::GenoDefine_InitKinds(); };
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_ObjAllocInit(a0: *mut HSD_ObjAllocData, a1: usize, a2: u32) => unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2 as _); };
    fn ft_80087AC0(a0: *mut HSD_GObj, a1: i32) => unsafe { let _ = crate::ft0877::ft_80087ac0(a0 as _, a1 as _); };
    fn ft_80087BAC(a0: *mut HSD_GObj, a1: i32) => unsafe { let _ = crate::ft0877::ft_80087bac(a0 as _, a1 as _); };
    fn ft_80087BC0(a0: *mut HSD_GObj, a1: i32) => unsafe { let _ = crate::ft0877::ft_80087bc0(a0 as _, a1 as _); };
    fn ft_80087BEC(a0: *mut HSD_GObj, a1: i32) => unsafe { let _ = crate::ft0877::ft_80087bec(a0 as _, a1 as _); };
    fn ftCo_800D4F24(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::ft_0d4d::ftCo_800D4F24(a0 as _, a1 as _); };
    fn ftCo_800D4FF4(a0: *mut HSD_GObj) => unsafe { crate::tu::ft_0d4d::ftCo_800D4FF4(a0 as _); };
    fn ftData_800855C8(a0: FighterKind, a1: u8) => unsafe { crate::tu::ftdata::ftData_800855C8(a0 as _, a1 as _); };
    fn ftData_8008578C(a0: i32, a1: u8) => unsafe { crate::tu::ftdata::ftData_8008578C(a0 as _, a1 as _); };
    fn ftDemo_CreateFighter(a0: *mut plAllocInfo2) -> *mut HSD_GObj => unsafe { crate::tu::ftdemo::ftdemo_create_fighter(&*(a0 as *const _)) as _ };
    fn ftDemo_SetArchiveData(a0: i32, a1: *mut HSD_Archive, a2: i32) => unsafe { crate::tu::ftdemo::ftdemo_set_archive_data(a0 as _, a1 as _, a2 as _); };
    fn ftDemo_SetFacingDirection(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::ftdemo::ftdemo_set_facing_direction(a0 as _, a1 as _); };
    fn ftKb_SpecialN_800EED50(a0: i32, a1: i32);
    fn ftLib_GetHipJObj(a0: *mut HSD_GObj) -> *mut HSD_JObj => unsafe { crate::ftlib::ftlib_get_hip_j_obj(a0 as _) as _ };
    fn ftLib_SetPos(a0: *mut HSD_GObj, a1: *mut Vec3) => unsafe { let _ = crate::ftlib::ftlib_set_pos(a0 as _, &*(a1 as *const _)); };
    fn ftLib_DisableInput(a0: *mut HSD_GObj) => unsafe { let _ = crate::ftlib::ftlib_disable_input(a0 as _); };
    fn ftLib_EnableInput(a0: *mut HSD_GObj) => unsafe { let _ = crate::ftlib::ftlib_enable_input(a0 as _); };
    fn ftLib_800869F8(a0: *mut HSD_GObj) -> f32 => unsafe { crate::ftlib::ftlib_800869f8(a0 as _) };
    fn ftLib_80086A4C(a0: *mut HSD_GObj, a1: f32) => unsafe { let _ = crate::ftlib::ftlib_80086a4c(a0 as _, a1 as _); };
    fn ftLib_GetCameraSubjectBonePos(a0: *mut HSD_GObj, a1: *mut Vec3) => unsafe { let _ = crate::ftlib::ftlib_get_camera_subject_bone_pos(a0 as _, &mut *(a1 as *mut _)); };
    fn ftLib_IsCameraSubjectInBounds(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_camera_subject_in_bounds(a0 as _) };
    fn ftLib_80086F4C(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_80086f4c(a0 as _) };
    fn ftLib_GetNameTagHeight(a0: *mut HSD_GObj) -> f32 => unsafe { crate::ftlib::ftlib_get_name_tag_height(a0 as _) };
    fn ftLib_IsSleeping(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_sleeping(a0 as _) };
    fn ftLib_SetPercent(a0: *mut HSD_GObj, a1: i32) => unsafe { let _ = crate::ftlib::ftlib_set_percent(a0 as _, a1 as _); };
    fn ftLib_80087140(a0: *mut HSD_GObj) => unsafe { let _ = crate::ftlib::ftlib_80087140(a0 as _); };
    fn ftLib_GetLastAttackerSlot(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_last_attacker_slot(a0 as _) };
    fn ftLib_IsRebirth(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_rebirth(a0 as _) };
    fn ftLib_LoadKirbyHats(a0: u8) => unsafe { crate::tu::ftlib::ftlib_load_kirby_hats(a0 as _); };
    fn ftLib_GetKind(a0: *mut HSD_GObj) -> FighterKind => unsafe { crate::ftlib::get_kind(a0 as _) };
    fn ftLib_SetScale(a0: *mut HSD_GObj, a1: f32) => unsafe { let _ = crate::ftlib::set_scale(a0 as _, a1 as _); };
    fn gm_8016B168() -> bool;
    fn gm_8016C5C0(a0: i32) -> i32;
    fn gm_GetFrameCount() -> u32;
    fn ifStatus_802F69C0(a0: i32, a1: i32) => unsafe { melee_world::tu::ifstatus::ifStatus_802F69C0(a0 as _, a1 as _); };
    fn ifStatus_802F6AF8(a0: i32) => unsafe { melee_world::tu::ifstatus::ifStatus_802F6AF8(a0 as _); };
    fn ifStatus_802F6C04(a0: i32) => unsafe { melee_world::tu::ifstatus::ifStatus_802F6C04(a0 as _); };
    fn ifStatus_802F6D10(a0: i32) => unsafe { melee_world::tu::ifstatus::ifStatus_802F6D10(a0 as _); };
    fn ifStatus_802F6E1C(a0: i32) => unsafe { melee_world::tu::ifstatus::ifStatus_802F6E1C(a0 as _); };
    fn lbArchive_LoadSymbols(a0: *const c_char, a1: *mut c_void, a2: *const c_char, a3: i32) -> *mut HSD_Archive;
    fn lbDvd_IsPlanningSixSlot() -> i32 => unsafe { melee_runtime::tu::lbdvd::lbDvd_IsPlanningSixSlot() };
    fn pl_8003891C(a0: i32) => unsafe { crate::tu::plbonus::pl_8003891C(a0 as _); };
    fn pl_80038F10(a0: i32) => unsafe { let _ = crate::plbonus3::pl_80038f10(a0 as _); };
    fn plAttack_80037590() => unsafe { crate::tu::plattack::plAttack_80037590(); };
    fn plStale_InitAttackInstance() => { crate::tu::plstale::plStale_InitAttackInstance() };
    }
    /// `ScriptGame_ModValue(slot, field, base)` (TARGET_PC): the crate's `SCRIPT_MOD_VALUE` hook,
    /// vanilla-neutral (returns `base`) when none is installed.
    pub fn ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 {
        match unsafe { crate::hooks::SCRIPT_MOD_VALUE } {
            Some(f) => f(slot, field, base),
            None => base,
        }
    }
}
// END EXT

// ---- .data / .bss -------------------------------------------------------------------------------

pub static mut str_PdPmdat_start_of_data: [u8; 9] = *b"PdPm.dat\0";
pub static mut str_plLoadCommonData: [u8; 17] = *b"plLoadCommonData\0";

pub static mut Player_AllocData: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut pl_804D6470: *mut pl_804D6470_t = null_mut();

const SUB: usize = 2;

/// `hasExtraFighterId`
#[inline]
pub fn hasExtraFighterId(data: &FtMapping) -> bool {
    melee_core::ct!("hasExtraFighterId");
    data.extra_internal_id != -1
}

/// `Player_CheckSlot`: `HSD_ASSERTREPORT(102, 0, "cant get player struct! %d\n", slot)`
#[inline]
pub fn Player_CheckSlot(slot: i32) {
    melee_core::ct!("Player_CheckSlot");
    if slot < 0 || !(slot < Gm_Player_NumMax as i32) {
        panic!("src/melee/pl/player.c:102: cant get player struct! {}", slot);
    }
}

#[inline]
fn map(ckind: CharacterKind) -> &'static mut FtMapping {
    unsafe { &mut ftMapping_list[ckind as usize] }
}

/// `&player_slots[slot]` after `Player_CheckSlot(slot)`.
#[inline]
fn sp(slot: i32) -> &'static mut StaticPlayer {
    Player_CheckSlot(slot);
    unsafe { &mut player_slots[slot as usize] }
}

#[inline]
fn ent(p: &StaticPlayer, i: usize) -> *mut HSD_GObj {
    p.player_entity[p.transformed[i] as usize]
}

/// `Player_GetPtrForSlot`
pub fn Player_GetPtrForSlot(slot: i32) -> *mut StaticPlayer {
    melee_core::ct!("Player_GetPtrForSlot");
    Player_CheckSlot(slot);
    unsafe { &mut player_slots[slot as usize] as *mut StaticPlayer }
}

pub fn Player_80031790(slot: i32) {
    melee_core::ct!("Player_80031790");
    let player = sp(slot);
    if player.pkind == Gm_PKind_Human || player.pkind == Gm_PKind_Cpu {
        for i in 0..SUB {
            if !ent(player, i).is_null() {
                ext::ftLib_DisableInput(ent(player, i));
            }
        }
    }
}

pub fn Player_80031848(slot: i32) {
    melee_core::ct!("Player_80031848");
    let player = sp(slot);
    if player.pkind == Gm_PKind_Human || player.pkind == Gm_PKind_Cpu {
        for i in 0..SUB {
            if !ent(player, i).is_null() {
                ext::ftLib_EnableInput(ent(player, i));
            }
        }
    }
}

/// `func_8008688C_wrapper` (static)
fn func_8008688C_wrapper(player: &mut StaticPlayer) {
    if player.pkind == Gm_PKind_Human || player.pkind == Gm_PKind_Cpu {
        for i in 0..SUB {
            if !ent(player, i).is_null() {
                ext::ftLib_EnableInput(ent(player, i));
            }
        }
    }
}

pub fn Player_80031900() {
    melee_core::ct!("Player_80031900");
    for slot in 0..Gm_Player_NumMax as i32 {
        let player = unsafe { &mut player_slots[slot as usize] };
        Player_CheckSlot(slot);
        func_8008688C_wrapper(player);
    }
}

pub fn Player_800319C4(slot: i32, arg1: bool) -> bool {
    melee_core::ct!("Player_800319C4");
    let player = sp(slot);
    if arg1 {
        for i in 0..SUB {
            if ent(player, i).is_null() {
                return false;
            }
        }
        return true;
    }
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            return false;
        }
    }
    true
}

pub fn Player_80031AD0(slot: i32) {
    melee_core::ct!("Player_80031AD0");
    let mut first_struct: plAllocInfo = unsafe { core::mem::zeroed() };
    let mut second_struct: plAllocInfo = unsafe { core::mem::zeroed() };
    let player = sp(slot);

    // `offset_arr[ckind * sizeof(ftMapping)]` is `ftMapping_list[ckind].extra_internal_id`
    if map(player.ckind).extra_internal_id != -1 {
        player.flags.set_b2(1);
    }

    let mut internal_id: i32 = map(player.ckind).internal_id as i32;
    Player_CheckSlot(slot);

    first_struct.internal_id = internal_id as FighterKind;
    first_struct.slot = slot as u8;
    first_struct.anon_6.set_b0(0);
    first_struct.anon_6.set_has_transformation(0);
    first_struct.x5 = -1;

    player.player_entity[0] = ext::Fighter_Create(&mut first_struct);
    player.player_state = 2;

    let byte_check: i8 = map(player.ckind).extra_internal_id;
    internal_id = byte_check as i32;

    if byte_check != -1 {
        let has_transformation: i32 = map(player.ckind).has_transformation as i32;

        Player_CheckSlot(slot);

        second_struct.internal_id = internal_id as FighterKind;
        second_struct.slot = slot as u8;
        second_struct.anon_6.set_b0(1);
        second_struct.anon_6.set_has_transformation(has_transformation as u8);
        second_struct.x5 = -1;

        player.player_entity[1] = ext::Fighter_Create(&mut second_struct);

        if player.player_state != 1 {
            player.player_state = 2;
        }
    }

    if player.player_state == 2 {
        if let Some(f) = player.struct_func {
            unsafe { f(slot) };
        }
    }
}

pub fn Player_80031CB0(kind: CharacterKind, color: u8) {
    melee_core::ct!("Player_80031CB0");
    // TARGET_PC: make the Geno define's character-kind mapping before the preload asks for files
    ext::GenoDefine_InitKinds();
    if map(kind).internal_id != -1 {
        ext::ftData_800855C8(map(kind).internal_id as i32 as FighterKind, color);
    }
    if hasExtraFighterId(map(kind)) {
        ext::ftData_800855C8(map(kind).extra_internal_id as i32 as FighterKind, color);
    }
}

static mut hats_preloaded: bool = false;

pub fn Player_80031D2C(kind: CharacterKind, color: u8) {
    melee_core::ct!("Player_80031D2C");
    // TARGET_PC: preload Kirby's hat archives once, before any fighter is set up
    unsafe {
        if !hats_preloaded && ext::lbDvd_IsPlanningSixSlot() == 0 {
            hats_preloaded = true;
            ext::ftLib_LoadKirbyHats(0);
        }
    }
    if map(kind).internal_id != -1 {
        ext::ftData_8008578C(map(kind).internal_id as i32, color);
    }
    if hasExtraFighterId(map(kind)) {
        ext::ftData_8008578C(map(kind).extra_internal_id as i32, color);
    }
}

pub fn Player_80031DA8(param_1: i32, param_2: i32) {
    melee_core::ct!("Player_80031DA8");
    ext::ftKb_SpecialN_800EED50(param_1, param_2);
}

pub fn Player_80031DC8(func_arg: unsafe extern "C" fn(i32, i32)) {
    melee_core::ct!("Player_80031DC8");
    for slot in 0..Gm_Player_NumMax as i32 {
        Player_CheckSlot(slot);
        let p = unsafe { &mut player_slots[slot as usize] };
        if p.player_state != 0 {
            unsafe { func_arg(map(p.ckind).internal_id as i32, 0) };
            if hasExtraFighterId(map(p.ckind)) {
                unsafe { func_arg(map(p.ckind).extra_internal_id as i32, 0) };
            }
        }
    }
}

pub fn Player_80031EBC(slot: i32) {
    melee_core::ct!("Player_80031EBC");
    Player_CheckSlot(slot);
    for i in 0..SUB {
        let player = unsafe { &mut player_slots[slot as usize] };
        if !ent(player, i).is_null() {
            if !ext::ftLib_IsSleeping(ent(player, i)) {
                ext::ftCo_800D4F24(ent(player, i), 1);
            }
            ext::HSD_GObjFree(ent(player, i));
        }
    }
}

pub fn Player_80031FB0(slot: i32, entity_index: i32) {
    melee_core::ct!("Player_80031FB0");
    let player = sp(slot);
    let t = player.transformed[entity_index as usize] as usize;
    player.player_entity[t] = null_mut();

    if player.flags.b2() == 0 || Player_800319C4(slot, false) {
        player.player_state = 0;
    }
}

pub fn Player_80032070(slot: i32, bool_arg: bool) {
    melee_core::ct!("Player_80032070");
    let player = sp(slot);

    if !bool_arg {
        ext::ftCo_800D4FF4(ent(player, 0));

        if player.flags.b2() != 0
            && map(player.ckind).has_transformation == 0
            && ext::ftLib_IsSleeping(ent(player, 1))
        {
            ext::ftCo_800D4FF4(ent(player, 1));
        }

        ext::ifStatus_802F6E1C(slot);
        return;
    }

    if ext::ftLib_IsRebirth(ent(player, 0)) {
        ext::ftCo_800D4FF4(ent(player, 1));
    }
}

pub fn Player_8003219C(slot: i32) -> bool {
    melee_core::ct!("Player_8003219C");
    let player = sp(slot);
    ext::ftLib_IsSleeping(ent(player, 0))
}

pub fn Player_8003221C(slot: i32) -> bool {
    melee_core::ct!("Player_8003221C");
    if Player_GetPlayerState(slot) == 2 {
        let player = sp(slot);
        if !ext::ftLib_IsSleeping(ent(player, 0)) {
            return true;
        }
    }
    false
}



pub fn Player_SetPlayerCharacter(slot: i32, value: CharacterKind) {
    melee_core::ct!("Player_SetPlayerCharacter");
    sp(slot).ckind = value;
}


pub fn Player_8003248C(slot: i32, arg1: i32) -> Gm_PKind {
    melee_core::ct!("Player_8003248C");
    let player = sp(slot);

    if arg1 == 1 {
        if map(player.ckind).has_transformation == 0 {
            if player.pkind == Gm_PKind_Human || player.pkind == Gm_PKind_Cpu {
                return 1;
            }
        }
    }
    player.pkind
}

pub fn Player_SetSlottype(slot: i32, value: Gm_PKind) {
    melee_core::ct!("Player_SetSlottype");
    sp(slot).pkind = value;
}


pub fn Player_80032610(slot: i32, arg1: i32) -> i8 {
    melee_core::ct!("Player_80032610");
    let error_value: i32 = -1;
    let player = sp(slot);

    if arg1 == 0 {
        return map(player.ckind).internal_id;
    }
    if arg1 == 1 {
        return map(player.ckind).extra_internal_id;
    }
    error_value as i8
}


pub fn Player_80032768(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_80032768");
    let player = sp(slot);
    unsafe {
        player.player_poses.byIndex[player.transformed[0] as usize] = *arg_vec;
        player.player_poses.byIndex[player.transformed[1] as usize] = *arg_vec;
    }
}

pub fn Player_80032828(slot: i32, index: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_80032828");
    let player = sp(slot);
    unsafe { player.player_poses.byIndex[player.transformed[index as usize] as usize] = *arg_vec };
}

pub fn Player_800328D4(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_800328D4");
    let player = sp(slot);
    Player_CheckSlot(slot);

    unsafe {
        player.player_poses.byIndex[player.transformed[0] as usize] = *arg_vec;
        player.player_poses.byIndex[player.transformed[1] as usize] = *arg_vec;
    }

    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ftLib_80087140(ent(player, i));
        }
    }
}

pub fn Player_80032A04(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_80032A04");
    Player_CheckSlot(slot);

    for i in 0..SUB {
        let player = unsafe { &mut player_slots[slot as usize] };
        unsafe { player.player_poses.byIndex[player.transformed[i] as usize] = *arg_vec };

        if !ent(player, i).is_null() {
            ext::ftLib_SetPos(ent(player, i), arg_vec);
        }
    }
}

pub fn Player_SetPlayerAndEntityFacingDirection(slot: i32, facing_dir: f32) {
    melee_core::ct!("Player_SetPlayerAndEntityFacingDirection");
    let player = sp(slot);
    for i in 0..SUB {
        player.facing_direction = facing_dir;
        if !ent(player, i).is_null() {
            ext::ftDemo_SetFacingDirection(ent(player, i), facing_dir);
        }
    }
}

pub fn Player_80032BB0(slot: i32) -> f32 {
    melee_core::ct!("Player_80032BB0");
    let player = sp(slot);
    let temp = ext::ftLib_800869F8(ent(player, 0));
    temp / player.model_scale
}

pub fn Player_SetScale(slot: i32, scale: f32) {
    melee_core::ct!("Player_SetScale");
    let player = sp(slot);
    player.model_scale = scale;
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ftLib_SetScale(ent(player, i), scale);
        }
    }
}

pub fn Player_GetSpawnPlatformPos(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_GetSpawnPlatformPos");
    let player = sp(slot);
    unsafe { *arg_vec = player.player_poses.byVecName.spawn_platform_final_pos };
}

pub fn Player_SetSpawnPlatformPos(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_SetSpawnPlatformPos");
    let player = sp(slot);
    unsafe { player.player_poses.byVecName.spawn_platform_final_pos = *arg_vec };
}

pub fn Player_GetSomePos(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_GetSomePos");
    let player = sp(slot);
    unsafe { *arg_vec = player.player_poses.byVecName.some_other_player_pos };
}

pub fn Player_SetSomePos(slot: i32, arg_vec: *mut Vec3) {
    melee_core::ct!("Player_SetSomePos");
    let player = sp(slot);
    unsafe { player.player_poses.byVecName.some_other_player_pos = *arg_vec };
}

pub fn Player_80032F30(slot: i32) -> i32 {
    melee_core::ct!("Player_80032F30");
    sp(slot).unk4E as i32
}

pub fn Player_80032FA4(slot: i32, arg: i32) {
    melee_core::ct!("Player_80032FA4");
    sp(slot).unk4E = arg as i8;
}

pub fn Player_GetFacingDirection(slot: i32) -> f32 {
    melee_core::ct!("Player_GetFacingDirection");
    sp(slot).facing_direction
}

pub fn Player_SetFacingDirection(slot: i32, direction: f32) {
    melee_core::ct!("Player_SetFacingDirection");
    sp(slot).facing_direction = direction;
}

pub fn Player_SetFacingDirectionConditional(slot: i32, b: bool, direction: f32) {
    melee_core::ct!("Player_SetFacingDirectionConditional");
    if !b {
        sp(slot).facing_direction = direction;
    }
}

pub fn Player_GetCostumeId(slot: i32) -> u32 {
    melee_core::ct!("Player_GetCostumeId");
    sp(slot).costume_id as u32
}

pub fn Player_SetCostumeId(slot: i32, costume_id: i32) {
    melee_core::ct!("Player_SetCostumeId");
    sp(slot).costume_id = costume_id as u8;
}

pub fn Player_GetControllerIndex(slot: i32) -> u8 {
    melee_core::ct!("Player_GetControllerIndex");
    sp(slot).controller_index as u8
}

pub fn Player_SetControllerIndex(slot: i32, controller_index: i8) {
    melee_core::ct!("Player_SetControllerIndex");
    sp(slot).controller_index = controller_index;
}


pub fn Player_SetTeam(slot: i32, team: i8) {
    melee_core::ct!("Player_SetTeam");
    sp(slot).team = team as u8;
}


pub fn Player_SetPlayerId(slot: i32, player_id: i32) {
    melee_core::ct!("Player_SetPlayerId");
    sp(slot).player_id = player_id as u8;
}

pub fn Player_GetCpuLevel(slot: i32) -> i32 {
    melee_core::ct!("Player_GetCpuLevel");
    sp(slot).cpu_level as i32
}

pub fn Player_SetPlayerAndEntityCpuLevel(slot: i32, cpu_level: i32) {
    melee_core::ct!("Player_SetPlayerAndEntityCpuLevel");
    let player = sp(slot);
    player.cpu_level = cpu_level as u8;
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ft_80087BEC(ent(player, i), cpu_level);
        }
    }
}

pub fn Player_GetCpuType(slot: i32) -> i32 {
    melee_core::ct!("Player_GetCpuType");
    sp(slot).cpu_type as i32
}

pub fn Player_SetPlayerAndEntityCpuType(slot: i32, cpu_type: i32) {
    melee_core::ct!("Player_SetPlayerAndEntityCpuType");
    let player = sp(slot);
    player.cpu_type = cpu_type as u8;
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ft_80087BC0(ent(player, i), cpu_type);
        }
    }
}

pub fn Player_GetHandicap(slot: i32) -> i32 {
    melee_core::ct!("Player_GetHandicap");
    sp(slot).handicap as i32
}

pub fn Player_SetHandicap(slot: i32, handicap: i8) {
    melee_core::ct!("Player_SetHandicap");
    sp(slot).handicap = handicap as u8;
}

pub fn Player_GetUnk50(slot: i32) -> f32 {
    melee_core::ct!("Player_GetUnk50");
    sp(slot).unk50
}

pub fn Player_GetAttackRatio(slot: i32) -> f32 {
    melee_core::ct!("Player_GetAttackRatio");
    let mut attack_ratio = sp(slot).attack_ratio;
    attack_ratio = ext::ScriptGame_ModValue(slot, 0, attack_ratio);
    attack_ratio
}

pub fn Player_SetAttackRatio(slot: i32, attack_ratio: f32) {
    melee_core::ct!("Player_SetAttackRatio");
    sp(slot).attack_ratio = attack_ratio;
}

pub fn Player_GetDefenseRatio(slot: i32) -> f32 {
    melee_core::ct!("Player_GetDefenseRatio");
    let mut defense_ratio = sp(slot).defense_ratio;
    defense_ratio = ext::ScriptGame_ModValue(slot, 1, defense_ratio);
    defense_ratio
}

pub fn Player_SetDefenseRatio(slot: i32, defense_ratio: f32) {
    melee_core::ct!("Player_SetDefenseRatio");
    sp(slot).defense_ratio = defense_ratio;
}

pub fn Player_GetModelScale(slot: i32) -> f32 {
    melee_core::ct!("Player_GetModelScale");
    sp(slot).model_scale
}

pub fn Player_SetModelScale(slot: i32, model_scale: f32) {
    melee_core::ct!("Player_SetModelScale");
    sp(slot).model_scale = model_scale;
}

pub fn Player_80033BB8(slot: i32) -> i32 {
    melee_core::ct!("Player_80033BB8");
    ext::gm_8016C5C0(slot)
}


pub fn Player_GetP1Stock() -> i32 {
    melee_core::ct!("Player_GetP1Stock");
    unsafe { player_slots[0].stocks as i32 }
}

pub fn Player_SetStocks(slot: i32, stocks: i32) {
    melee_core::ct!("Player_SetStocks");
    sp(slot).stocks = stocks as i8;
}

pub fn Player_LoseStock(slot: i32) {
    melee_core::ct!("Player_LoseStock");
    let player = sp(slot);
    if player.stocks > 0 {
        player.stocks = player.stocks - 1;
    }
}


pub fn Player_SetCoins(slot: i32, current_coins: i32) {
    melee_core::ct!("Player_SetCoins");
    sp(slot).current_coins = current_coins;
}

pub fn Player_GetTotalCoins(slot: i32) -> i32 {
    melee_core::ct!("Player_GetTotalCoins");
    sp(slot).total_coins
}

pub fn Player_SetTotalCoins(slot: i32, coins: i32) {
    melee_core::ct!("Player_SetTotalCoins");
    sp(slot).total_coins = coins;
}

pub fn Player_GetUnk98(slot: i32) -> i32 {
    melee_core::ct!("Player_GetUnk98");
    sp(slot).unk98
}

pub fn Player_SetUnk98(slot: i32, unk98: i32) {
    melee_core::ct!("Player_SetUnk98");
    sp(slot).unk98 = unk98;
}

pub fn Player_GetUnk9C(slot: i32) -> i32 {
    melee_core::ct!("Player_GetUnk9C");
    sp(slot).unk9C
}

pub fn Player_SetUnk9C(slot: i32, unk9C: i32) {
    melee_core::ct!("Player_SetUnk9C");
    sp(slot).unk9C = unk9C;
}



pub fn Player_SwapTransformedStates(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("Player_SwapTransformedStates");
    let player = sp(slot);
    let temp_r5 = player.transformed[arg1 as usize];
    player.transformed[arg1 as usize] = player.transformed[arg2 as usize];
    player.transformed[arg2 as usize] = temp_r5;
}


pub fn Player_SetHUDDamage(slot: i32, arg1: i32) {
    melee_core::ct!("Player_SetHUDDamage");
    Player_CheckSlot(slot);
    for i in 0..SUB {
        let player = unsafe { &mut player_slots[slot as usize] };
        unsafe { player.staminas.byIndex[player.transformed[i] as usize] = arg1 as i16 };
        if !ent(player, i).is_null() {
            ext::ftLib_SetPercent(ent(player, i), arg1);
        }
    }
}

pub fn Player_SetHPByIndex(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("Player_SetHPByIndex");
    let player = sp(slot);
    unsafe { player.staminas.byIndex[player.transformed[arg1 as usize] as usize] = arg2 as i16 };
}

pub fn Player_GetOtherStamina(slot: i32) -> i32 {
    melee_core::ct!("Player_GetOtherStamina");
    let player = sp(slot);
    unsafe { player.staminas.byName.stamina as i32 }
}

pub fn Player_SetOtherStamina(slot: i32, stamina: i32) {
    melee_core::ct!("Player_SetOtherStamina");
    let player = sp(slot);
    unsafe { player.staminas.byName.stamina = stamina as i16 };
}



pub fn Player_SetMoreFlagsBit2(slot: i32, bit2: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit2");
    sp(slot).more_flags.set_b2(bit2);
}

pub fn Player_GetMoreFlagsBit3(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit3");
    sp(slot).more_flags.b3() != 0
}

pub fn Player_SetMoreFlagsBit3(slot: i32, bit3: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit3");
    sp(slot).more_flags.set_b3(bit3);
}

pub fn Player_SetMoreFlagsBit4(slot: i32, bit4: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit4");
    sp(slot).more_flags.set_b4(bit4);
}

pub fn Player_GetMoreFlagsBit4(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit4");
    sp(slot).more_flags.b4() != 0
}

pub fn Player_GetMoreFlagsBit5(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit5");
    sp(slot).more_flags.b5() != 0
}

pub fn Player_SetMoreFlagsBit5(slot: i32, bit5: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit5");
    sp(slot).more_flags.set_b5(bit5);
}

/// `bool` is MSL `int`: the 2-bit field comes back as 0..=3
pub fn Player_GetMoreFlagsBit6(slot: i32) -> i32 {
    melee_core::ct!("Player_GetMoreFlagsBit6");
    sp(slot).more_flags.b6() as i32
}

pub fn Player_SetMoreFlagsBit6(slot: i32, bit6: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit6");
    sp(slot).more_flags.set_b6(bit6);
}

pub fn Player_GetFlagsAEBit0(slot: i32) -> u8 {
    melee_core::ct!("Player_GetFlagsAEBit0");
    sp(slot).flagsAE.b0()
}

pub fn Player_SetFlagsAEBit0(slot: i32, bit0: u8) {
    melee_core::ct!("Player_SetFlagsAEBit0");
    sp(slot).flagsAE.set_b0(bit0);
}

pub fn Player_GetRemainingHPByIndex(slot: i32, index: i32) -> i32 {
    melee_core::ct!("Player_GetRemainingHPByIndex");
    let player = sp(slot);
    let stamina: u32 = unsafe { player.staminas.byName.stamina as i32 as u32 };
    let result: i32 = unsafe {
        stamina.wrapping_sub(player.staminas.byIndex[player.transformed[index as usize] as usize] as i32 as u32) as i32
    };
    let mut result2 = result;
    if result < 0 {
        result2 = 0;
    }
    result2
}

pub fn Player_GetFalls(slot: i32) -> i32 {
    melee_core::ct!("Player_GetFalls");
    let player = sp(slot);

    if map(player.ckind).extra_internal_id != -1 && map(player.ckind).has_transformation != 0 {
        return player.falls[player.transformed[0] as usize].wrapping_add(player.falls[player.transformed[1] as usize]);
    }

    player.falls[player.transformed[0] as usize]
}

pub fn Player_GetFallsByIndex(slot: i32, arg1: i32) -> i32 {
    melee_core::ct!("Player_GetFallsByIndex");
    let player = sp(slot);
    player.falls[player.transformed[arg1 as usize] as usize]
}

pub fn Player_SetFalls(slot: i32, falls: i32) {
    melee_core::ct!("Player_SetFalls");
    let player = sp(slot);
    player.falls[player.transformed[0] as usize] = falls;
}

pub fn Player_SetFallsByIndex(slot: i32, index: i32, falls: i32) {
    melee_core::ct!("Player_SetFallsByIndex");
    let player = sp(slot);
    player.falls[player.transformed[index as usize] as usize] = falls;
}

pub fn Player_GetKOsByPlayerIndex(slot: i32, idx: i32) -> i32 {
    melee_core::ct!("Player_GetKOsByPlayerIndex");
    sp(slot).kos_by_player[idx as usize] as i32
}

pub fn Player_UpdateKOsBySlot(slot: i32, bool_arg: bool, other_slot: i32) {
    melee_core::ct!("Player_UpdateKOsBySlot");
    let player = sp(slot);

    if !bool_arg {
        let kos = player.kos_by_player[other_slot as usize];

        if kos < u32::MAX {
            player.kos_by_player[other_slot as usize] = kos + 1;
        }

        if slot == other_slot {
            ext::ifStatus_802F6D10(other_slot);
        } else {
            if ext::gm_8016B168() {
                Player_CheckSlot(slot);
                Player_CheckSlot(other_slot);
                let other_player = unsafe { &mut player_slots[other_slot as usize] };

                let player_team: i32 = player.team as i32;
                let other_player_team: i32 = other_player.team as i32;
                if player_team == other_player_team {
                    ext::ifStatus_802F69C0(other_slot, slot);
                    return;
                }
            }

            ext::ifStatus_802F6AF8(other_slot);
        }
    }
}

pub fn Player_GetMatchFrameCount(slot: i32) -> u32 {
    melee_core::ct!("Player_GetMatchFrameCount");
    sp(slot).match_frame_count
}

pub fn Player_UpdateMatchFrameCount(slot: i32, condition: bool) {
    melee_core::ct!("Player_UpdateMatchFrameCount");
    let player = sp(slot);
    if !condition && player.match_frame_count.wrapping_add(0x10000) == 0xffff {
        player.match_frame_count = ext::gm_GetFrameCount();
    }
}

pub fn Player_GetSelfDestructs(slot: i32) -> u32 {
    melee_core::ct!("Player_GetSelfDestructs");
    sp(slot).self_destructs as u32
}

pub fn Player_SetSelfDestructs(slot: i32, self_destructs: u32) {
    melee_core::ct!("Player_SetSelfDestructs");
    if self_destructs > 0xffff || 0 > self_destructs as i32 {
        return;
    }
    let player = sp(slot);
    player.self_destructs = self_destructs as u16;
}

pub fn Player_IncSelfDestructs(slot: i32, condition: i32) {
    melee_core::ct!("Player_IncSelfDestructs");
    let player = sp(slot);
    if condition == 0 {
        let self_destructs = player.self_destructs;
        if self_destructs < 0xffff {
            player.self_destructs = player.self_destructs.wrapping_add(1);
            ext::ifStatus_802F6C04(slot);
        }
    }
}

pub fn Player_800353BC(slot: i32) -> bool {
    melee_core::ct!("Player_800353BC");
    let player = sp(slot);
    player.player_state == 2 && player.flags.b0() != 0
}

pub fn Player_8003544C(slot: i32, condition: bool) -> bool {
    melee_core::ct!("Player_8003544C");
    if !condition {
        let player = sp(slot);
        if player.player_state == 2 && player.flags.b0() != 0 {
            return true;
        }
    }
    false
}

/// `bool` is `int` in the C build: the whole value goes through the 1-bit store.
pub fn Player_SetFlagsBit0(slot: i32, bit0: i32) {
    melee_core::ct!("Player_SetFlagsBit0");
    sp(slot).flags.set_b0((bit0 & 1) as u8);
}


pub fn Player_SetNametagSlotID(slot: i32, nametag_slot_id: i32) {
    melee_core::ct!("Player_SetNametagSlotID");
    sp(slot).nametag_slot_id = nametag_slot_id;
}

pub fn Player_GetFlagsBit1(slot: i32) -> i32 {
    melee_core::ct!("Player_GetFlagsBit1");
    sp(slot).flags.b1() as i32
}

pub fn Player_SetFlagsBit1(slot: i32) {
    melee_core::ct!("Player_SetFlagsBit1");
    sp(slot).flags.set_b1(1);
}

pub fn Player_UnsetFlagsBit1(slot: i32) {
    melee_core::ct!("Player_UnsetFlagsBit1");
    sp(slot).flags.set_b1(0);
}

pub fn Player_GetFlagsBit3(slot: i32) -> i32 {
    melee_core::ct!("Player_GetFlagsBit3");
    sp(slot).flags.b3() as i32
}

pub fn Player_SetFlagsBit3(slot: i32, bit3: u8) {
    melee_core::ct!("Player_SetFlagsBit3");
    sp(slot).flags.set_b3(bit3);
}

pub fn Player_GetFlagsBit4(slot: i32) -> i32 {
    melee_core::ct!("Player_GetFlagsBit4");
    sp(slot).flags.b4() as i32
}

pub fn Player_GetFlagsBit5(slot: i32) -> u8 {
    melee_core::ct!("Player_GetFlagsBit5");
    sp(slot).flags.is_metal()
}

pub fn Player_SetFlagsBit5(slot: i32, is_metal: u8) {
    melee_core::ct!("Player_SetFlagsBit5");
    sp(slot).flags.set_is_metal(is_metal);
}

pub fn Player_GetFlagsBit6(slot: i32) -> u8 {
    melee_core::ct!("Player_GetFlagsBit6");
    sp(slot).flags.b6()
}

pub fn Player_SetFlagsBit6(slot: i32, bit6: u8) {
    melee_core::ct!("Player_SetFlagsBit6");
    sp(slot).flags.set_b6(bit6);
}

pub fn Player_GetFlagsBit7(slot: i32) -> u8 {
    melee_core::ct!("Player_GetFlagsBit7");
    sp(slot).flags.b7()
}

pub fn Player_SetFlagsBit7(slot: i32, bit7: u8) {
    melee_core::ct!("Player_SetFlagsBit7");
    sp(slot).flags.set_b7(bit7);
}

pub fn Player_GetMoreFlagsBit0(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit0");
    sp(slot).more_flags.b0() != 0
}

pub fn Player_GetMoreFlagsBit1(slot: i32) -> bool {
    melee_core::ct!("Player_GetMoreFlagsBit1");
    sp(slot).more_flags.b1() != 0
}

pub fn Player_SetMoreFlagsBit1(slot: i32, bit1: u8) {
    melee_core::ct!("Player_SetMoreFlagsBit1");
    sp(slot).more_flags.set_b1(bit1);
}

pub fn Player_GetUnk4D(slot: i32) -> i32 {
    melee_core::ct!("Player_GetUnk4D");
    sp(slot).unk4D as u8 as i32
}

pub fn Player_SetUnk4D(slot: i32, unk4D: i8) {
    melee_core::ct!("Player_SetUnk4D");
    sp(slot).unk4D = unk4D;
}

pub fn Player_GetFlagsAEBit1(slot: i32) -> u8 {
    melee_core::ct!("Player_GetFlagsAEBit1");
    sp(slot).flagsAE.b1()
}

/// Declared `u8` in C but falls off the end (no `return`).
pub fn Player_SetFlagsAEBit1(slot: i32, bit1: u8) {
    melee_core::ct!("Player_SetFlagsAEBit1");
    sp(slot).flagsAE.set_b1(bit1);
}

pub fn Player_GetUnk4C(slot: i32) -> i32 {
    melee_core::ct!("Player_GetUnk4C");
    sp(slot).unk4C as u8 as i32
}

pub fn Player_SetUnk4C(slot: i32, unk4C: u8) {
    melee_core::ct!("Player_SetUnk4C");
    sp(slot).unk4C = unk4C as i8;
}

pub fn Player_80036058(slot: i32) -> bool {
    melee_core::ct!("Player_80036058");
    let player = sp(slot);
    ext::ftLib_80086F4C(ent(player, 0))
}

pub fn Player_800360D8(slot: i32) -> f32 {
    melee_core::ct!("Player_800360D8");
    let player = sp(slot);
    ext::ftLib_GetNameTagHeight(ent(player, 0))
}

pub fn Player_SetStructFunc(slot: i32, arg_func: *mut c_void) {
    melee_core::ct!("Player_SetStructFunc");
    let player = sp(slot);
    player.struct_func = unsafe { core::mem::transmute::<*mut c_void, Option<unsafe extern "C" fn(i32)>>(arg_func) };
}

pub fn Player_GetActionStats(slot: i32) -> *mut plActionStats {
    melee_core::ct!("Player_GetActionStats");
    Player_CheckSlot(slot);
    unsafe { core::ptr::addr_of_mut!(player_slots[slot as usize].stale_moves.total_attack_count_struct) }
}

pub fn Player_GetStaleMoveTableIndexPtr(slot: i32) -> *mut StaleMoveTable {
    melee_core::ct!("Player_GetStaleMoveTableIndexPtr");
    Player_CheckSlot(slot);
    unsafe { core::ptr::addr_of_mut!(player_slots[slot as usize].stale_moves) }
}

pub fn Player_GetUnk6A8Ptr(slot: i32) -> *mut pl_x5EC_t {
    melee_core::ct!("Player_GetUnk6A8Ptr");
    Player_CheckSlot(slot);
    unsafe { core::ptr::addr_of_mut!(player_slots[slot as usize].stale_moves.x5EC) }
}

pub fn Player_GetStaleMoveTableIndexPtr2(slot: i32) -> *mut pl_StaleMoveTableExt_t {
    melee_core::ct!("Player_GetStaleMoveTableIndexPtr2");
    Player_CheckSlot(slot);
    unsafe { core::ptr::addr_of_mut!(player_slots[slot as usize].stale_moves) as *mut pl_StaleMoveTableExt_t }
}

pub fn Player_80036394(slot: i32) -> FighterKind {
    melee_core::ct!("Player_80036394");
    let player = sp(slot);
    let entity = ent(player, 0);
    if !entity.is_null() {
        return ext::ftLib_GetKind(entity);
    }
    -1i32 as FighterKind
}

pub fn Player_80036428(slot: i32) -> i32 {
    melee_core::ct!("Player_80036428");
    let player = sp(slot);
    let entity = ent(player, 0);
    if !entity.is_null() {
        return ext::ftLib_GetLastAttackerSlot(entity);
    }
    6
}

pub fn Player_SetUnk45(slot: i32, unk45: i32) {
    melee_core::ct!("Player_SetUnk45");
    sp(slot).unk45 = unk45 as u8;
}

pub fn Player_GetUnk45(slot: i32) -> u32 {
    melee_core::ct!("Player_GetUnk45");
    let player = sp(slot);
    if player.pkind == Gm_PKind_Human {
        return player.unk45 as u32;
    }
    4
}

pub fn Player_UpdateJoystickCountByIndex(slot: i32, index: i32) {
    melee_core::ct!("Player_UpdateJoystickCountByIndex");
    let player = sp(slot);
    let transformed = player.transformed[index as usize] as usize;
    player.joystick_direction_input_count[transformed] =
        player.joystick_direction_input_count[transformed].wrapping_add(1);
}

pub fn Player_GetJoystickCountByIndex(slot: i32, index: i32) -> i32 {
    melee_core::ct!("Player_GetJoystickCountByIndex");
    let player = sp(slot);
    let transformed = player.transformed[index as usize] as usize;
    player.joystick_direction_input_count[transformed]
}

pub fn Player_800366DC(slot: i32, arg1: i32) {
    melee_core::ct!("Player_800366DC");
    let player = sp(slot);
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ft_80087AC0(ent(player, i), arg1);
        }
    }
}

pub fn Player_80036790(slot: i32, arg1: f32) {
    melee_core::ct!("Player_80036790");
    let player = sp(slot);
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ftLib_80086A4C(ent(player, i), arg1);
        }
    }
}

pub fn Player_80036844(slot: i32, arg1: i32) {
    melee_core::ct!("Player_80036844");
    let player = sp(slot);
    for i in 0..SUB {
        if !ent(player, i).is_null() {
            ext::ft_80087BAC(ent(player, i), arg1);
        }
    }
}

pub fn Player_800368F8(slot: i32) -> bool {
    melee_core::ct!("Player_800368F8");
    let player = sp(slot);
    ext::ftLib_IsCameraSubjectInBounds(ent(player, 0))
}

pub fn Player_80036978(slot: i32, pos: *mut Vec3) {
    melee_core::ct!("Player_80036978");
    let player = sp(slot);
    ext::ftLib_GetCameraSubjectBonePos(ent(player, 0), pos);
}

pub fn Player_InitOrResetPlayer(slot: i32) {
    melee_core::ct!("Player_InitOrResetPlayer");
    let player = sp(slot);

    player.player_state = 0;
    player.ckind = CKind_Mario;
    let t0 = player.transformed[0] as usize;
    let t1 = player.transformed[1] as usize;

    let zerofloat: f32 = 0.0;
    unsafe {
        let poses = &mut player.player_poses.byIndex;
        poses[t0].z = zerofloat;
        poses[t0].y = zerofloat;
        poses[t0].x = zerofloat;

        poses[t1].z = zerofloat;
        poses[t1].y = zerofloat;
        poses[t1].x = zerofloat;

        poses[2].z = zerofloat;
        poses[2].y = zerofloat;
        poses[2].x = zerofloat;
        poses[3].z = zerofloat;
        poses[3].y = zerofloat;
        poses[3].x = zerofloat;
    }

    player.costume_id = 0;
    player.pkind = Gm_PKind_NA;
    player.transformed[0] = 0;
    player.transformed[1] = 1;

    player.unk45 = 0;
    player.controller_index = 0;
    player.team = 0;
    player.player_id = 0;
    player.cpu_level = 0;

    player.cpu_type = 4;

    player.handicap = 0;
    player.unk4C = 0;

    player.unk4D = 4;
    player.unk4E = -1;

    let onefloat: f32 = 1.0;
    player.unk50 = onefloat;
    player.attack_ratio = onefloat;
    player.defense_ratio = onefloat;
    player.model_scale = onefloat;

    unsafe {
        player.staminas.byIndex[player.transformed[0] as usize] = 0;
        player.staminas.byIndex[player.transformed[1] as usize] = 0;
        player.staminas.byIndex[2] = 0;
    }

    player.falls[player.transformed[0] as usize] = 0;
    player.falls[player.transformed[1] as usize] = 0;

    player.kos_by_player[0] = 0;
    player.kos_by_player[1] = 0;
    player.kos_by_player[2] = 0;
    player.kos_by_player[3] = 0;
    player.kos_by_player[4] = 0;
    player.kos_by_player[5] = 0;

    player.match_frame_count = u32::MAX;
    player.self_destructs = 0;
    player.stocks = 0;

    player.current_coins = 0;
    player.total_coins = 0;
    player.unk98 = 0;
    player.unk9C = 0;

    // `*transformed0` / `*transformed1` were read before the reset above (pointers to the bytes,
    // so these use the freshly reset 0 / 1)
    player.joystick_direction_input_count[player.transformed[0] as usize] = 0;
    player.joystick_direction_input_count[player.transformed[1] as usize] = 0;

    player.flags.set_b0(0);
    player.nametag_slot_id = 0;
    player.flags.set_b1(0);
    player.flags.set_b2(0);
    player.flags.set_b3(0);
    player.flags.set_b4(0);
    player.flags.set_is_metal(0);
    player.flags.set_b6(0);
    player.flags.set_b7(0);

    player.more_flags.set_b0(0);
    player.more_flags.set_b1(0);
    player.more_flags.set_b2(0);
    player.more_flags.set_b3(0);
    player.more_flags.set_b4(0);
    player.more_flags.set_b5(0);
    player.more_flags.set_b6(0);

    player.flagsAE.set_b0(0);
    player.flagsAE.set_b1(0);

    player.player_entity[player.transformed[0] as usize] = null_mut();
    player.player_entity[player.transformed[1] as usize] = null_mut();

    player.struct_func = None;
}

pub fn Player_80036CF0(slot: i32) {
    melee_core::ct!("Player_80036CF0");
    Player_InitOrResetPlayer(slot);
    ext::pl_8003891C(slot);
}

pub fn Player_80036D24(slot: i32) {
    melee_core::ct!("Player_80036D24");
    Player_InitOrResetPlayer(slot);
    ext::pl_80038F10(slot);
}

pub fn Player_InitAllPlayers() {
    melee_core::ct!("Player_InitAllPlayers");
    ext::plStale_InitAttackInstance();
    ext::plAttack_80037590();

    for i in 0..Gm_Player_NumMax as i32 {
        Player_InitOrResetPlayer(i);
        ext::pl_8003891C(i);
    }
}

pub fn Player_80036DA4() {
    melee_core::ct!("Player_80036DA4");
    unsafe { ext::HSD_ObjAllocInit(&mut Player_AllocData, 8, 4) };
    ext::Fighter_FirstInitialize_80067A84();
}

pub fn Player_80036DD8() {
    melee_core::ct!("Player_80036DD8");
    let mut sp8: *mut *mut c_void = null_mut();
    unsafe {
        ext::lbArchive_LoadSymbols(
            str_PdPmdat_start_of_data.as_ptr() as *const c_char,
            &mut sp8 as *mut *mut *mut c_void as *mut c_void,
            str_plLoadCommonData.as_ptr() as *const c_char,
            0,
        );
        pl_804D6470 = *sp8 as *mut pl_804D6470_t;
    }
}

pub fn Player_80036E20(ckind: CharacterKind, archive: *mut HSD_Archive, arg2: i32) {
    melee_core::ct!("Player_80036E20");
    ext::ftDemo_SetArchiveData(map(ckind).internal_id as i32, archive, arg2);
    if map(ckind).extra_internal_id != -1 && map(ckind).has_transformation == 0 {
        ext::ftDemo_SetArchiveData(map(ckind).extra_internal_id as i32, archive, arg2);
    }
}

pub fn Player_80036EA0(slot: i32) -> *mut HSD_JObj {
    melee_core::ct!("Player_80036EA0");
    let player = sp(slot);
    let entity = ent(player, 0);
    if !entity.is_null() {
        return ext::ftLib_GetHipJObj(entity);
    }
    null_mut()
}

pub fn Player_80036F34(slot: i32, arg1: i32) {
    melee_core::ct!("Player_80036F34");
    let mut some_struct: plAllocInfo2 = unsafe { core::mem::zeroed() };
    let player = sp(slot);
    some_struct.internal_id = map(player.ckind).internal_id as i32 as FighterKind;
    some_struct.slot = slot as u8;
    some_struct.anon_c.set_has_transformation(0);
    some_struct.unk8 = arg1;
    some_struct.anon_c.set_b0(0);

    player.pkind = Gm_PKind_Demo;
    player.player_entity[0] = ext::ftDemo_CreateFighter(&mut some_struct);
    if map(player.ckind).extra_internal_id != -1 && map(player.ckind).has_transformation == 0 {
        some_struct.internal_id = map(player.ckind).extra_internal_id as i32 as FighterKind;
        some_struct.anon_c.set_has_transformation(1);
        player.player_entity[1] = ext::ftDemo_CreateFighter(&mut some_struct);
    }
}

pub fn Player_80037054(slot: i32, arg1: i32) {
    melee_core::ct!("Player_80037054");
    let mut some_struct: plAllocInfo2 = unsafe { core::mem::zeroed() };
    let player = sp(slot);
    some_struct.internal_id = map(player.ckind).internal_id as i32 as FighterKind;
    some_struct.slot = slot as u8;
    some_struct.anon_c.set_has_transformation(0);
    some_struct.unk8 = arg1;
    some_struct.anon_c.set_b0(1);

    player.pkind = Gm_PKind_Demo;
    player.player_entity[0] = ext::ftDemo_CreateFighter(&mut some_struct);
    if map(player.ckind).extra_internal_id != -1 && map(player.ckind).has_transformation == 0 {
        some_struct.internal_id = map(player.ckind).extra_internal_id as i32 as FighterKind;
        some_struct.anon_c.set_has_transformation(1);
        player.player_entity[1] = ext::ftDemo_CreateFighter(&mut some_struct);
    }
}

/// TARGET_PC: m-ex character kind `ckind` plays fighter kind `fkind`.
pub fn Player_MexSetMapping(ckind: i32, fkind: i32) {
    melee_core::ct!("Player_MexSetMapping");
    let m = map(ckind as CharacterKind);
    m.internal_id = fkind as i8;
    m.extra_internal_id = -1;
    m.has_transformation = 0;
}

/// TARGET_PC: a Zelda / Sheik template define pair: `ckind` also owns the partner sub-fighter.
pub fn Player_MexSetPartner(ckind: i32, partner_fkind: i32) {
    melee_core::ct!("Player_MexSetPartner");
    let m = map(ckind as CharacterKind);
    m.extra_internal_id = partner_fkind as i8;
    m.has_transformation = 1;
}

/// TARGET_PC: every scene begins without fighters.
pub fn Player_ForgetEntities() {
    melee_core::ct!("Player_ForgetEntities");
    for slot in 0..Gm_Player_NumMax as usize {
        for i in 0..SUB {
            unsafe { player_slots[slot].player_entity[i] = null_mut() };
        }
    }
}

// ---- folded from player.rs (consolidation steps 21/22) ----
/// `Player_80031790`.
pub fn player_80031790(slot: i32) {
    melee_core::ct!("player_80031790");
    let p = player(slot);
    if human_or_cpu(p) {
        for i in 0..2 {
            let e = ent_player(p, i);
            if !e.is_null() {
                h::ftlib_disable_input(e);
            }
        }
    }
}

/// `func_8008688C_wrapper` (static).
pub fn func_8008688c_wrapper(p: &StaticPlayer) {
    melee_core::ct!("func_8008688c_wrapper");
    if human_or_cpu(p) {
        for i in 0..2 {
            let e = ent_player(p, i);
            if !e.is_null() {
                h::ftlib_enable_input(e);
            }
        }
    }
}

/// `Player_80031848`.
pub fn player_80031848(slot: i32) {
    melee_core::ct!("player_80031848");
    func_8008688c_wrapper(player(slot));
}

/// `Player_80031900`.
pub fn player_80031900() {
    melee_core::ct!("player_80031900");
    for slot in 0..PLAYER_NUM_MAX as i32 {
        func_8008688c_wrapper(player(slot));
    }
}

/// `Player_800319C4(slot, arg1)`: arg1 -> both entities exist; !arg1 -> neither does.
pub fn player_800319c4(slot: i32, arg1: i32) -> bool {
    melee_core::ct!("player_800319c4");
    let p = player(slot);
    if arg1 != 0 {
        return (0..2).all(|i| !ent_player(p, i).is_null());
    }
    (0..2).all(|i| ent_player(p, i).is_null())
}

/// `Player_80031DA8`.
pub fn player_80031da8(a: i32, b: i32) {
    melee_core::ct!("player_80031da8");
    h::ftkb_specialn_800eed50(a, b);
}

/// `Player_80031DC8(func)`.
pub fn player_80031dc8(mut func: impl FnMut(i32, i32)) {
    melee_core::ct!("player_80031dc8");
    for slot in 0..PLAYER_NUM_MAX as i32 {
        let p = player(slot);
        if p.player_state != 0 {
            let mp = *mapping(p.ckind);
            func(mp.internal_id as i32, 0);
            if has_extra_fighter_id(&mp) {
                func(mp.extra_internal_id as i32, 0);
            }
        }
    }
}

/// `Player_80031EBC`.
pub fn player_80031ebc(slot: i32) {
    melee_core::ct!("player_80031ebc");
    for i in 0..2 {
        let p = player(slot);
        let e = ent_player(p, i);
        if !e.is_null() {
            if !h::ftlib_is_sleeping(e) {
                h::ftco_800d4f24(ent_player(p, i), 1);
            }
            h::hsd_gobj_free(ent_player(p, i));
        }
    }
}

/// `Player_80031FB0`.
pub fn player_80031fb0(slot: i32, entity_index: i32) {
    melee_core::ct!("player_80031fb0");
    let p = player(slot);
    let t = p.transformed[entity_index as usize] as usize;
    p.player_entity[t] = core::ptr::null_mut();
    if p.flags.b2() == 0 || player_800319c4(slot, 0) {
        player(slot).player_state = 0;
    }
}

/// `Player_80032070`.
pub fn player_80032070(slot: i32, bool_arg: i32) {
    melee_core::ct!("player_80032070");
    let p = player(slot);
    if bool_arg == 0 {
        h::ftco_800d4ff4(ent_player(p, 0));
        if p.flags.b2() != 0 && mapping(p.ckind).has_transformation == 0 && h::ftlib_is_sleeping(ent_player(p, 1)) {
            h::ftco_800d4ff4(ent_player(p, 1));
        }
        h::ifstatus_802f6e1c(slot);
        return;
    }
    if h::ftlib_is_rebirth(ent_player(p, 0)) {
        h::ftco_800d4ff4(ent_player(p, 1));
    }
}

/// `Player_8003219C`.
pub fn player_8003219c(slot: i32) -> bool {
    melee_core::ct!("player_8003219c");
    h::ftlib_is_sleeping(ent_player(player(slot), 0))
}

/// `Player_8003221C`.
pub fn player_8003221c(slot: i32) -> bool {
    melee_core::ct!("player_8003221c");
    get_player_state(slot) == 2 && !h::ftlib_is_sleeping(ent_player(player(slot), 0))
}

/// `Player_8003248C` over the real mapping table (the night-1 form is `stale::slot_type_for`).
pub fn player_8003248c(slot: i32, arg1: i32) -> Gm_PKind {
    melee_core::ct!("player_8003248c");
    let p = player(slot);
    crate::stale::slot_type_for(slot, arg1 == 1, mapping(p.ckind).has_transformation != 0)
}

/// `Player_800325C8(kind, b)`.
pub fn player_800325c8(kind: CharacterKind, b: i32) -> i8 {
    melee_core::ct!("player_800325c8");
    if b == 0 {
        mapping(kind).internal_id
    } else if b == 1 {
        mapping(kind).extra_internal_id
    } else {
        -1
    }
}

/// `Player_80032610(slot, arg1)`.
pub fn player_80032610(slot: i32, arg1: i32) -> i8 {
    melee_core::ct!("player_80032610");
    player_800325c8(player(slot).ckind, arg1)
}

/// `Player_80032768`.
pub fn player_80032768(slot: i32, v: &Vec3) {
    melee_core::ct!("player_80032768");
    let p = player(slot);
    let (a, b) = (p.transformed[0] as usize, p.transformed[1] as usize);
    poses(p)[a] = *v;
    poses(p)[b] = *v;
}

/// `Player_80032828`.
pub fn player_80032828(slot: i32, index: i32, v: &Vec3) {
    melee_core::ct!("player_80032828");
    let p = player(slot);
    let t = p.transformed[index as usize] as usize;
    poses(p)[t] = *v;
}

/// `Player_800328D4`.
pub fn player_800328d4(slot: i32, v: &Vec3) {
    melee_core::ct!("player_800328d4");
    player_80032768(slot, v);
    let p = player(slot);
    for i in 0..2 {
        let e = ent_player(p, i);
        if !e.is_null() {
            h::ftlib_80087140(e);
        }
    }
}

/// `Player_80032A04`.
pub fn player_80032a04(slot: i32, v: &Vec3) {
    melee_core::ct!("player_80032a04");
    for i in 0..2 {
        let p = player(slot);
        let t = p.transformed[i] as usize;
        poses(p)[t] = *v;
        let e = ent_player(p, i);
        if !e.is_null() {
            h::ftlib_set_pos(e, v);
        }
    }
}

/// `Player_80032BB0`.
pub fn player_80032bb0(slot: i32) -> f32 {
    melee_core::ct!("player_80032bb0");
    let p = player(slot);
    let t = h::ftlib_800869f8(ent_player(p, 0));
    t / p.model_scale
}

/// `Player_80032F30` (unk4E, s8 sign-extended).
pub fn player_80032f30(slot: i32) -> i32 {
    melee_core::ct!("player_80032f30");
    player(slot).unk4E as i32
}

/// `Player_80032FA4`.
pub fn player_80032fa4(slot: i32, arg: i32) {
    melee_core::ct!("player_80032fa4");
    player(slot).unk4E = arg as i8;
}

/// `Player_80033BB8`.
pub fn player_80033bb8(slot: i32) -> i32 {
    melee_core::ct!("player_80033bb8");
    h::gm_8016c5c0(slot)
}
