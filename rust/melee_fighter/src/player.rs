
//! The player table: hand port of the accessor half of src/melee/pl/player.c (port round 2).
//!
//! `player_slots` itself, `Player_CheckSlot`, the stocks / slot-type / stale-table accessors
//! live in `crate::stale` (night 1); this module ports the rest of player.c that is field logic
//! over the table, from `Player_80031790` to `Player_80036978`, plus the TARGET_PC m-ex mapping
//! setters and `Player_ForgetEntities`. Calls into fighters (ftLib, ftCo, ftDemo), the HUD
//! (ifStatus) and the match (gm) go through `crate::hooks` and panic until their owner is ported.
//!
//! Not ported here: `Player_80031AD0` (Fighter_Create), `Player_80031CB0` / `80031D2C`
//! (ftData preload + Geno), `Player_InitOrResetPlayer` and everything after it (allocators,
//! archives, JObj).
//!
//! C quirks kept: `transformed[i]` indexes the entity / falls / joystick arrays unchecked in C;
//! here an index >= 2 (>= 4 for poses and staminas) panics instead of reading the next field.
//! `Player_SetFlagsAEBit1` is declared `u8` and returns nothing in C (garbage); this returns ().
//! Checked against the C by tests/player_ref.rs (gen/cref/player_ref.c).
#![allow(static_mut_refs)]

use crate::hooks as h;

use crate::stale::{player, FtMapping, PLAYER_NUM_MAX};

use crate::types::*;

/// `ftMapping_list[ChKind_Cap]` (TARGET_PC size; m-ex rows are written by `mex_set_mapping`):
/// owned by `melee_world::pl`, which the world TUs read too.
pub use melee_world::pl::ftMapping_list as FT_MAPPING_LIST;

/// The retail initial value of `FT_MAPPING_LIST`.
pub use melee_world::pl::FT_MAPPING_INIT;

#[inline]
pub(crate) fn mapping(ckind: CharacterKind) -> &'static mut FtMapping {
    unsafe { &mut FT_MAPPING_LIST[ckind as usize] }
}

/// `hasExtraFighterId` (static inline).
#[inline]
pub fn has_extra_fighter_id(d: &FtMapping) -> bool {
    d.extra_internal_id != -1
}

#[inline]
pub(crate) fn ent(p: &StaticPlayer, i: usize) -> *mut HSD_GObj {
    p.player_entity[p.transformed[i] as usize]
}

#[inline]
pub(crate) fn human_or_cpu(p: &StaticPlayer) -> bool {
    p.pkind == Gm_PKind_Human || p.pkind == Gm_PKind_Cpu
}

/// `Player_GetPlayerState`.
pub fn get_player_state(slot: i32) -> i32 {
    player(slot).player_state
}

/// `Player_GetPlayerCharacter`.
pub fn get_player_character(slot: i32) -> CharacterKind {
    player(slot).ckind
}

/// `Player_SetPlayerCharacter`.
pub fn set_player_character(slot: i32, v: CharacterKind) {
    player(slot).ckind = v;
}

#[inline]
pub(crate) fn poses(p: &mut StaticPlayer) -> &mut [Vec3; 4] {
    unsafe { &mut p.player_poses.byIndex }
}

#[inline]
pub(crate) fn stam(p: &mut StaticPlayer) -> &mut [i16; 4] {
    unsafe { &mut p.staminas.byIndex }
}

/// `Player_LoadPlayerCoords`.
pub fn load_player_coords(slot: i32) -> Vec3 {
    let p = player(slot);
    let t = p.transformed[0] as usize;
    poses(p)[t]
}

/// `Player_SetPlayerAndEntityFacingDirection`.
pub fn set_player_and_entity_facing_direction(slot: i32, dir: f32) {
    let p = player(slot);
    for i in 0..2 {
        p.facing_direction = dir;
        let e = ent(p, i);
        if !e.is_null() {
            h::ftdemo_set_facing_direction(e, dir);
        }
    }
}

/// `Player_SetScale`.
pub fn set_scale(slot: i32, scale: f32) {
    let p = player(slot);
    p.model_scale = scale;
    for i in 0..2 {
        let e = ent(p, i);
        if !e.is_null() {
            h::ftlib_set_scale(e, scale);
        }
    }
}

/// `Player_GetSpawnPlatformPos`.
pub fn get_spawn_platform_pos(slot: i32) -> Vec3 {
    unsafe { player(slot).player_poses.byVecName.spawn_platform_final_pos }
}

/// `Player_SetSpawnPlatformPos`.
pub fn set_spawn_platform_pos(slot: i32, v: &Vec3) {
    player(slot).player_poses.byVecName.spawn_platform_final_pos = *v;
}

/// `Player_GetSomePos`.
pub fn get_some_pos(slot: i32) -> Vec3 {
    unsafe { player(slot).player_poses.byVecName.some_other_player_pos }
}

/// `Player_SetSomePos`.
pub fn set_some_pos(slot: i32, v: &Vec3) {
    player(slot).player_poses.byVecName.some_other_player_pos = *v;
}

/// `Player_GetFacingDirection`.
pub fn get_facing_direction(slot: i32) -> f32 {
    player(slot).facing_direction
}

/// `Player_SetFacingDirection`.
pub fn set_facing_direction(slot: i32, dir: f32) {
    player(slot).facing_direction = dir;
}

/// `Player_SetFacingDirectionConditional`.
pub fn set_facing_direction_conditional(slot: i32, b: i32, dir: f32) {
    if b == 0 {
        player(slot).facing_direction = dir;
    }
}

/// `Player_GetCostumeId`.
pub fn get_costume_id(slot: i32) -> u32 {
    player(slot).costume_id as u32
}

/// `Player_SetCostumeId`.
pub fn set_costume_id(slot: i32, v: i32) {
    player(slot).costume_id = v as u8;
}

/// `Player_GetControllerIndex` (s8 field returned as u8).
pub fn get_controller_index(slot: i32) -> u8 {
    player(slot).controller_index as u8
}

/// `Player_SetControllerIndex`.
pub fn set_controller_index(slot: i32, v: i8) {
    player(slot).controller_index = v;
}

/// `Player_GetTeam`.
pub fn get_team(slot: i32) -> i32 {
    player(slot).team as i32
}

/// `Player_SetTeam`.
pub fn set_team(slot: i32, v: i8) {
    player(slot).team = v as u8;
}

/// `Player_GetPlayerId`.
pub fn get_player_id(slot: i32) -> i32 {
    player(slot).player_id as i32
}

/// `Player_SetPlayerId`.
pub fn set_player_id(slot: i32, v: i32) {
    player(slot).player_id = v as u8;
}

/// `Player_GetCpuLevel`.
pub fn get_cpu_level(slot: i32) -> i32 {
    player(slot).cpu_level as i32
}

/// `Player_SetPlayerAndEntityCpuLevel`.
pub fn set_player_and_entity_cpu_level(slot: i32, v: i32) {
    let p = player(slot);
    p.cpu_level = v as u8;
    for i in 0..2 {
        let e = ent(p, i);
        if !e.is_null() {
            h::ft_80087bec(e, v);
        }
    }
}

/// `Player_GetCpuType`.
pub fn get_cpu_type(slot: i32) -> i32 {
    player(slot).cpu_type as i32
}

/// `Player_SetPlayerAndEntityCpuType`.
pub fn set_player_and_entity_cpu_type(slot: i32, v: i32) {
    let p = player(slot);
    p.cpu_type = v as u8;
    for i in 0..2 {
        let e = ent(p, i);
        if !e.is_null() {
            h::ft_80087bc0(e, v);
        }
    }
}

/// `Player_GetHandicap`.
pub fn get_handicap(slot: i32) -> i32 {
    player(slot).handicap as i32
}

/// `Player_SetHandicap`.
pub fn set_handicap(slot: i32, v: i8) {
    player(slot).handicap = v as u8;
}

/// `Player_GetUnk50`.
pub fn get_unk50(slot: i32) -> f32 {
    player(slot).unk50
}

/// `Player_GetAttackRatio` (TARGET_PC: through `ScriptGame_ModValue(slot, 0, r)`).
pub fn get_attack_ratio(slot: i32) -> f32 {
    let r = player(slot).attack_ratio;
    h::script_mod_value(slot, 0, r)
}

/// `Player_SetAttackRatio`.
pub fn set_attack_ratio(slot: i32, v: f32) {
    player(slot).attack_ratio = v;
}

/// `Player_GetDefenseRatio` (TARGET_PC: through `ScriptGame_ModValue(slot, 1, r)`).
pub fn get_defense_ratio(slot: i32) -> f32 {
    let r = player(slot).defense_ratio;
    h::script_mod_value(slot, 1, r)
}

/// `Player_SetDefenseRatio`.
pub fn set_defense_ratio(slot: i32, v: f32) {
    player(slot).defense_ratio = v;
}

/// `Player_GetModelScale`.
pub fn get_model_scale(slot: i32) -> f32 {
    player(slot).model_scale
}

/// `Player_SetModelScale`.
pub fn set_model_scale(slot: i32, v: f32) {
    player(slot).model_scale = v;
}

/// `Player_GetP1Stock` (no slot check).
pub fn get_p1_stock() -> i32 {
    unsafe { crate::stale::PLAYER_SLOTS[0].stocks as i32 }
}

/// `Player_LoseStock`.
pub fn lose_stock(slot: i32) {
    let p = player(slot);
    if p.stocks > 0 {
        p.stocks -= 1;
    }
}

/// `Player_GetCoins`.
pub fn get_coins(slot: i32) -> i32 {
    player(slot).current_coins
}

/// `Player_SetCoins`.
pub fn set_coins(slot: i32, v: i32) {
    player(slot).current_coins = v;
}

/// `Player_GetTotalCoins`.
pub fn get_total_coins(slot: i32) -> i32 {
    player(slot).total_coins
}

/// `Player_SetTotalCoins`.
pub fn set_total_coins(slot: i32, v: i32) {
    player(slot).total_coins = v;
}

/// `Player_GetUnk98`.
pub fn get_unk98(slot: i32) -> i32 {
    player(slot).unk98
}

/// `Player_SetUnk98`.
pub fn set_unk98(slot: i32, v: i32) {
    player(slot).unk98 = v;
}

/// `Player_GetUnk9C`.
pub fn get_unk9c(slot: i32) -> i32 {
    player(slot).unk9C
}

/// `Player_SetUnk9C`.
pub fn set_unk9c(slot: i32, v: i32) {
    player(slot).unk9C = v;
}

/// `Player_GetEntity`.
pub fn get_entity(slot: i32) -> *mut HSD_GObj {
    ent(player(slot), 0)
}

/// `Player_GetEntityAtIndex`.
pub fn get_entity_at_index(slot: i32, index: i32) -> *mut HSD_GObj {
    ent(player(slot), index as usize)
}

/// `Player_SwapTransformedStates`.
pub fn swap_transformed_states(slot: i32, a: i32, b: i32) {
    player(slot).transformed.swap(a as usize, b as usize);
}

/// `Player_GetDamage`.
pub fn get_damage(slot: i32) -> i32 {
    let p = player(slot);
    let t = p.transformed[0] as usize;
    stam(p)[t] as i32
}

/// `Player_SetHUDDamage`.
pub fn set_hud_damage(slot: i32, v: i32) {
    for i in 0..2 {
        let p = player(slot);
        let t = p.transformed[i] as usize;
        stam(p)[t] = v as i16;
        let e = ent(p, i);
        if !e.is_null() {
            h::ftlib_800870f0(e, v);
        }
    }
}

/// `Player_SetHPByIndex`.
pub fn set_hp_by_index(slot: i32, index: i32, v: i32) {
    let p = player(slot);
    let t = p.transformed[index as usize] as usize;
    stam(p)[t] = v as i16;
}

/// `Player_GetOtherStamina`.
pub fn get_other_stamina(slot: i32) -> i32 {
    unsafe { player(slot).staminas.byName.stamina as i32 }
}

/// `Player_SetOtherStamina`.
pub fn set_other_stamina(slot: i32, v: i32) {
    player(slot).staminas.byName.stamina = v as i16;
}

/// `Player_GetRemainingHP`: max(stamina - damage, 0).
pub fn get_remaining_hp(slot: i32) -> i32 {
    get_remaining_hp_by_index(slot, 0)
}

/// `Player_GetRemainingHPByIndex`.
pub fn get_remaining_hp_by_index(slot: i32, index: i32) -> i32 {
    let p = player(slot);
    let t = p.transformed[index as usize] as usize;
    let s = unsafe { p.staminas.byName.stamina } as i32;
    let r = s.wrapping_sub(stam(p)[t] as i32);
    if r < 0 { 0 } else { r }
}

macro_rules! bit_get_set {
    ($($get:ident $set:ident: $field:ident . $g:ident / $s:ident;)*) => {$(
        #[doc = concat!("`", stringify!($field), ".", stringify!($g), "` getter.")]
        pub fn $get(slot: i32) -> u8 { player(slot).$field.$g() }
        #[doc = concat!("`", stringify!($field), ".", stringify!($g), "` setter (truncated to the field width).")]
        pub fn $set(slot: i32, v: u8) { player(slot).$field.$s(v) }
    )*};
}
bit_get_set! {
    get_more_flags_bit2 set_more_flags_bit2: more_flags.b2 / set_b2;
    get_more_flags_bit3 set_more_flags_bit3: more_flags.b3 / set_b3;
    get_more_flags_bit4 set_more_flags_bit4: more_flags.b4 / set_b4;
    get_more_flags_bit5 set_more_flags_bit5: more_flags.b5 / set_b5;
    get_more_flags_bit6 set_more_flags_bit6: more_flags.b6 / set_b6;
    get_flags_ae_bit0 set_flags_ae_bit0: flagsAE.b0 / set_b0;
    get_flags_ae_bit1 set_flags_ae_bit1: flagsAE.b1 / set_b1;
    get_flags_bit3 set_flags_bit3: flags.b3 / set_b3;
    get_flags_bit5 set_flags_bit5: flags.is_metal / set_is_metal;
    get_flags_bit6 set_flags_bit6: flags.b6 / set_b6;
    get_flags_bit7 set_flags_bit7: flags.b7 / set_b7;
    get_more_flags_bit1 set_more_flags_bit1: more_flags.b1 / set_b1;
}
/// `Player_GetFlagsBit1`.
pub fn get_flags_bit1(slot: i32) -> u8 {
    player(slot).flags.b1()
}
/// `Player_SetFlagsBit1`.
pub fn set_flags_bit1(slot: i32) {
    player(slot).flags.set_b1(1);
}
/// `Player_UnsetFlagsBit1`.
pub fn unset_flags_bit1(slot: i32) {
    player(slot).flags.set_b1(0);
}
/// `Player_GetFlagsBit4`.
pub fn get_flags_bit4(slot: i32) -> u8 {
    player(slot).flags.b4()
}
/// `Player_GetMoreFlagsBit0`.
pub fn get_more_flags_bit0(slot: i32) -> u8 {
    player(slot).more_flags.b0()
}
/// `Player_SetFlagsBit0(slot, bool)`.
pub fn set_flags_bit0(slot: i32, v: i32) {
    player(slot).flags.set_b0(v as u8);
}

/// `Player_GetFalls`: both entities' falls for a transforming character with a partner.
pub fn get_falls(slot: i32) -> i32 {
    let p = player(slot);
    let mp = mapping(p.ckind);
    let a = p.falls[p.transformed[0] as usize];
    if mp.extra_internal_id != -1 && mp.has_transformation != 0 {
        return a.wrapping_add(p.falls[p.transformed[1] as usize]);
    }
    a
}
/// `Player_GetFallsByIndex`.
pub fn get_falls_by_index(slot: i32, index: i32) -> i32 {
    let p = player(slot);
    p.falls[p.transformed[index as usize] as usize]
}
/// `Player_SetFalls`.
pub fn set_falls(slot: i32, v: i32) {
    set_falls_by_index(slot, 0, v);
}
/// `Player_SetFallsByIndex`.
pub fn set_falls_by_index(slot: i32, index: i32, v: i32) {
    let p = player(slot);
    let t = p.transformed[index as usize] as usize;
    p.falls[t] = v;
}
/// `Player_GetKOsByPlayerIndex` (returns the u32 count as s32).
pub fn get_kos_by_player_index(slot: i32, idx: i32) -> i32 {
    player(slot).kos_by_player[idx as usize] as i32
}
/// `Player_UpdateKOsBySlot(slot, bool_arg, other_slot)`.
pub fn update_kos_by_slot(slot: i32, bool_arg: i32, other: i32) {
    let p = player(slot);
    if bool_arg != 0 {
        return;
    }
    let k = &mut p.kos_by_player[other as usize];
    if *k < u32::MAX {
        *k += 1;
    }
    if slot == other {
        h::ifstatus_802f6d10(other);
    } else {
        if h::gm_8016b168() {
            let team = player(slot).team as i32;
            let other_team = player(other).team as i32;
            if team == other_team {
                h::ifstatus_802f69c0(other, slot);
                return;
            }
        }
        h::ifstatus_802f6af8(other);
    }
}
/// `Player_GetMatchFrameCount`.
pub fn get_match_frame_count(slot: i32) -> u32 {
    player(slot).match_frame_count
}
/// `Player_UpdateMatchFrameCount`: `(count + 0x10000) == 0xFFFF` in u32, i.e. count is still
/// the 0xFFFFFFFF "unset" value.
pub fn update_match_frame_count(slot: i32, condition: i32) {
    let p = player(slot);
    if condition == 0 && p.match_frame_count.wrapping_add(0x10000) == 0xFFFF {
        p.match_frame_count = h::gm_get_frame_count();
    }
}
/// `Player_GetSelfDestructs`.
pub fn get_self_destructs(slot: i32) -> u32 {
    player(slot).self_destructs as u32
}
/// `Player_SetSelfDestructs` (values above 0xFFFF are ignored; the slot is not checked then).
pub fn set_self_destructs(slot: i32, v: u32) {
    if v > 0xFFFF || (v as i32) < 0 {
        return;
    }
    player(slot).self_destructs = v as u16;
}
/// `Player_IncSelfDestructs`.
pub fn inc_self_destructs(slot: i32, condition: i32) {
    let p = player(slot);
    if condition == 0 && p.self_destructs < 0xFFFF {
        p.self_destructs += 1;
        h::ifstatus_802f6c04(slot);
    }
}
/// `Player_800353BC`.
pub fn player_800353bc(slot: i32) -> bool {
    let p = player(slot);
    p.player_state == 2 && p.flags.b0() != 0
}
/// `Player_8003544C`.
pub fn player_8003544c(slot: i32, condition: i32) -> bool {
    condition == 0 && player_800353bc(slot)
}
/// `Player_GetNametagSlotID` (the int field truncated to u8).
pub fn get_nametag_slot_id(slot: i32) -> u8 {
    player(slot).nametag_slot_id as u8
}
/// `Player_SetNametagSlotID`.
pub fn set_nametag_slot_id(slot: i32, v: i32) {
    player(slot).nametag_slot_id = v;
}
/// `Player_GetUnk4D` (s8 read as u8).
pub fn get_unk4d(slot: i32) -> i32 {
    player(slot).unk4D as u8 as i32
}
/// `Player_SetUnk4D`.
pub fn set_unk4d(slot: i32, v: i8) {
    player(slot).unk4D = v;
}
/// `Player_GetUnk4C` (s8 read as u8).
pub fn get_unk4c(slot: i32) -> i32 {
    player(slot).unk4C as u8 as i32
}
/// `Player_SetUnk4C`.
pub fn set_unk4c(slot: i32, v: u8) {
    player(slot).unk4C = v as i8;
}
/// `Player_80036058`.
pub fn player_80036058(slot: i32) -> bool {
    h::ftlib_80086f4c(ent(player(slot), 0))
}
/// `Player_800360D8`.
pub fn player_800360d8(slot: i32) -> f32 {
    h::ftlib_80086f80(ent(player(slot), 0))
}
/// `Player_SetStructFunc`.
pub fn set_struct_func(slot: i32, f: Option<unsafe extern "C" fn(i32)>) {
    player(slot).struct_func = f;
}
/// `Player_GetUnk6A8Ptr`.
pub fn get_unk6a8_ptr(slot: i32) -> &'static mut pl_x5EC_t {
    &mut player(slot).stale_moves.x5EC
}
/// `Player_GetStaleMoveTableIndexPtr2` (the same table, seen as `pl_StaleMoveTableExt_t`).
pub fn get_stale_move_table_index_ptr2(slot: i32) -> *mut pl_StaleMoveTableExt_t {
    &mut player(slot).stale_moves as *mut StaleMoveTable as *mut pl_StaleMoveTableExt_t
}
/// `Player_80036394`: the main entity's fighter kind, or -1.
pub fn player_80036394(slot: i32) -> FighterKind {
    let e = ent(player(slot), 0);
    if e.is_null() { (-1i32) as FighterKind } else { h::ftlib_get_kind(e) }
}
/// `Player_80036428`.
pub fn player_80036428(slot: i32) -> i32 {
    let e = ent(player(slot), 0);
    if e.is_null() { 6 } else { h::ftlib_80087300(e) }
}
/// `Player_SetUnk45`.
pub fn set_unk45(slot: i32, v: i32) {
    player(slot).unk45 = v as u8;
}
/// `Player_GetUnk45`: 4 unless the slot is human.
pub fn get_unk45(slot: i32) -> u32 {
    let p = player(slot);
    if p.pkind == Gm_PKind_Human { p.unk45 as u32 } else { 4 }
}
/// `Player_UpdateJoystickCountByIndex`.
pub fn update_joystick_count_by_index(slot: i32, index: i32) {
    let p = player(slot);
    let t = p.transformed[index as usize] as usize;
    p.joystick_direction_input_count[t] = p.joystick_direction_input_count[t].wrapping_add(1);
}
/// `Player_GetJoystickCountByIndex`.
pub fn get_joystick_count_by_index(slot: i32, index: i32) -> i32 {
    let p = player(slot);
    p.joystick_direction_input_count[p.transformed[index as usize] as usize]
}

fn each_entity(slot: i32, mut f: impl FnMut(*mut HSD_GObj)) {
    let p = player(slot);
    for i in 0..2 {
        let e = ent(p, i);
        if !e.is_null() {
            f(e);
        }
    }
}
/// `Player_800366DC`.
pub fn player_800366dc(slot: i32, arg: i32) {
    each_entity(slot, |e| h::ft_80087ac0(e, arg));
}
/// `Player_80036790`.
pub fn player_80036790(slot: i32, arg: f32) {
    each_entity(slot, |e| h::ftlib_80086a4c(e, arg));
}
/// `Player_80036844`.
pub fn player_80036844(slot: i32, arg: i32) {
    each_entity(slot, |e| h::ft_80087bac(e, arg));
}
/// `Player_800368F8`.
pub fn player_800368f8(slot: i32) -> bool {
    h::ftlib_80086bb4(ent(player(slot), 0))
}
/// `Player_80036978`.
pub fn player_80036978(slot: i32, pos: &mut Vec3) {
    h::ftlib_80086b90(ent(player(slot), 0), pos);
}

/// `Player_MexSetMapping(ckind, fkind)` (TARGET_PC).
pub fn mex_set_mapping(ckind: i32, fkind: i32) {
    let r = mapping(ckind as CharacterKind);
    r.internal_id = fkind as i8;
    r.extra_internal_id = -1;
    r.has_transformation = 0;
}
/// `Player_MexSetPartner(ckind, partner_fkind)` (TARGET_PC).
pub fn mex_set_partner(ckind: i32, partner: i32) {
    let r = mapping(ckind as CharacterKind);
    r.extra_internal_id = partner as i8;
    r.has_transformation = 1;
}
/// `Player_ForgetEntities` (TARGET_PC).
pub fn forget_entities() {
    for slot in 0..PLAYER_NUM_MAX {
        unsafe { crate::stale::PLAYER_SLOTS[slot].player_entity = [core::ptr::null_mut(); 2] };
    }
}

pub use crate::tu::player::func_8008688c_wrapper;
pub use crate::tu::player::player_80031790;
pub use crate::tu::player::player_80031848;
pub use crate::tu::player::player_80031900;
pub use crate::tu::player::player_800319c4;
pub use crate::tu::player::player_80031da8;
pub use crate::tu::player::player_80031dc8;
pub use crate::tu::player::player_80031ebc;
pub use crate::tu::player::player_80031fb0;
pub use crate::tu::player::player_80032070;
pub use crate::tu::player::player_8003219c;
pub use crate::tu::player::player_8003221c;
pub use crate::tu::player::player_8003248c;
pub use crate::tu::player::player_800325c8;
pub use crate::tu::player::player_80032610;
pub use crate::tu::player::player_80032768;
pub use crate::tu::player::player_80032828;
pub use crate::tu::player::player_800328d4;
pub use crate::tu::player::player_80032a04;
pub use crate::tu::player::player_80032bb0;
pub use crate::tu::player::player_80032f30;
pub use crate::tu::player::player_80032fa4;
pub use crate::tu::player::player_80033bb8;
