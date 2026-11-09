//! The script host's game side on the engine: `melee_script::game::ScriptGame` for the running match, each method the
//! call pc/platform/gw_script*.c makes into pc/gameworld/script_game.c (the transpiled TU, melee_modes::tu::script_game).
//! Arguments and results are the C's ints and floats; a C `int` answering a yes/no is tested `!= 0` as the C callers do.
#![allow(clippy::missing_safety_doc)]

use melee_modes::tu::script_game as sg;
use melee_script::game::ScriptGame;

/// The engine's game, as the script host sees it (stateless: everything lives in the game's own globals).
#[derive(Default)]
pub struct EngineGame;

impl ScriptGame for EngineGame {
    /// `ScriptGame_FighterI`
    fn fighter_i(&self, slot: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterI(slot as _, field as _) as _ }
    }
    /// `ScriptGame_FighterF`
    fn fighter_f(&self, slot: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_FighterF(slot as _, field as _) as _ }
    }
    /// `ScriptGame_LabI`
    fn lab_i(&self, slot: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_LabI(slot as _, field as _) as _ }
    }
    /// `ScriptGame_LabF`
    fn lab_f(&self, slot: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_LabF(slot as _, field as _) as _ }
    }
    /// `ScriptGame_HitI`
    fn hit_i(&self, slot: i32, i: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_HitI(slot as _, i as _, field as _) as _ }
    }
    /// `ScriptGame_HitF`
    fn hit_f(&self, slot: i32, i: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_HitF(slot as _, i as _, field as _) as _ }
    }
    /// `ScriptGame_HurtI`
    fn hurt_i(&self, slot: i32, i: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_HurtI(slot as _, i as _, field as _) as _ }
    }
    /// `ScriptGame_HurtF`
    fn hurt_f(&self, slot: i32, i: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_HurtF(slot as _, i as _, field as _) as _ }
    }
    /// `ScriptGame_JointF`
    fn joint_f(&self, slot: i32, i: i32, comp: i32) -> f32 {
        unsafe { sg::ScriptGame_JointF(slot as _, i as _, comp as _) as _ }
    }
    /// `ScriptGame_JointsSetup`
    fn joints_setup(&mut self, slot: i32) {
        unsafe { sg::ScriptGame_JointsSetup(slot as _) };
    }
    /// `ScriptGame_JointParent`
    fn joint_parent(&self, slot: i32, i: i32) -> i32 {
        unsafe { sg::ScriptGame_JointParent(slot as _, i as _) as _ }
    }
    /// `ScriptGame_LabDObjI`
    fn lab_dobj_i(&mut self, slot: i32, d: i32, field: u32) -> i32 {
        unsafe { sg::ScriptGame_LabDObjI(slot as _, d as _, field as _) as _ }
    }
    /// `ScriptGame_LabTObjF`
    fn lab_tobj_f(&self, slot: i32, d: i32, t: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_LabTObjF(slot as _, d as _, t as _, field as _) as _ }
    }
    /// `ScriptGame_LabDebugDraw`
    fn lab_debug_draw(&mut self, slot: i32, write: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_LabDebugDraw(slot as _, write as _, flags as _) as _ }
    }
    /// `ScriptGame_LabStageDraw`
    fn lab_stage_draw(&mut self, mask: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_LabStageDraw(mask as _, flags as _) as _ }
    }
    /// `ScriptGame_CameraF`
    fn camera_f(&self, field: i32) -> f32 {
        unsafe { sg::ScriptGame_CameraF(field as _) as _ }
    }
    /// `ScriptGame_LabAnimEnd`
    fn lab_anim_end(&self, slot: i32) -> f32 {
        unsafe { sg::ScriptGame_LabAnimEnd(slot as _) as _ }
    }
    /// `ScriptGame_LabAttrCount`
    fn lab_attr_count(&self) -> i32 {
        unsafe { sg::ScriptGame_LabAttrCount() as _ }
    }
    /// `ScriptGame_LabAttrF`
    fn lab_attr_f(&self, slot: i32, i: i32) -> f32 {
        unsafe { sg::ScriptGame_LabAttrF(slot as _, i as _) as _ }
    }
    /// `ScriptGame_VirtualPadField`
    fn virtual_pad_field(&mut self, slot: i32, k: i32, v: i32, set: bool) -> i32 {
        unsafe { sg::ScriptGame_VirtualPadField(slot as _, k as _, v as _, set as _) as _ }
    }
    /// `ScriptGame_VirtualPadSeen`
    fn virtual_pad_seen(&self, slot: i32, k: i32) -> i32 {
        unsafe { sg::ScriptGame_VirtualPadSeen(slot as _, k as _) as _ }
    }
    /// `ScriptGame_VirtualPadRelease`
    fn virtual_pad_release(&mut self, slot: i32, owner: i32) {
        unsafe { sg::ScriptGame_VirtualPadRelease(slot as _, owner as _) };
    }
    /// `ScriptGame_SetStocks`
    fn set_stocks(&mut self, slot: i32, n: i32) {
        unsafe { sg::ScriptGame_SetStocks(slot as _, n as _) };
    }
    /// `ScriptGame_Impulse`
    fn impulse(&mut self, slot: i32, x_bits: i32, y_bits: i32) -> bool {
        unsafe { sg::ScriptGame_Impulse(slot as _, x_bits as _, y_bits as _) != 0 }
    }
    /// `ScriptGame_Hit`
    fn hit(&mut self, slot: i32, from: i32, damage: i32, angle: i32, kbg: i32, bkb: i32) -> bool {
        unsafe { sg::ScriptGame_Hit(slot as _, from as _, damage as _, angle as _, kbg as _, bkb as _) != 0 }
    }
    /// `ScriptGame_CpuMode`
    fn cpu_mode(&mut self, slot: i32, fight: i32) -> bool {
        unsafe { sg::ScriptGame_CpuMode(slot as _, fight as _) != 0 }
    }
    /// `ScriptGame_CpuScriptMode`
    fn cpu_script_mode(&mut self, slot: i32) -> bool {
        unsafe { sg::ScriptGame_CpuScriptMode(slot as _) != 0 }
    }
    /// `ScriptGame_CpuModeInfo`
    fn cpu_mode_info(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_CpuModeInfo(slot as _) as _ }
    }
    /// `ScriptGame_PlaySoundOptions`
    fn play_sound_options(&mut self, id: i32, volume: i32, pitch: i32) -> i32 {
        unsafe { sg::ScriptGame_PlaySoundOptions(id as _, volume as _, pitch as _) as _ }
    }
    /// `ScriptGame_LabCommonCount`
    fn lab_common_count(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_LabCommonCount(slot as _) as _ }
    }
    /// `ScriptGame_LabMotionAnim`
    fn lab_motion_anim(&self, slot: i32, motion: i32) -> i32 {
        unsafe { sg::ScriptGame_LabMotionAnim(slot as _, motion as _) as _ }
    }
    /// `ScriptGame_LabRowCount`
    fn lab_row_count(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_LabRowCount(slot as _) as _ }
    }
    /// `ScriptGame_LabSetPreviewRow`
    fn lab_set_preview_row(&mut self, slot: i32, row: i32, facing: i32) {
        unsafe { sg::ScriptGame_LabSetPreviewRow(slot as _, row as _, facing as _) };
    }
    /// `ScriptGame_LabSetMotion`
    fn lab_set_motion(&mut self, slot: i32, motion: i32, rate_bits: i32, lift_bits: i32) {
        unsafe { sg::ScriptGame_LabSetMotion(slot as _, motion as _, rate_bits as _, lift_bits as _) };
    }
    /// `ScriptGame_StageSlotsActive`
    fn stage_slots_active(&self) -> bool {
        unsafe { sg::ScriptGame_StageSlotsActive() != 0 }
    }
    /// `ScriptGame_LabCommonF`
    fn lab_common_f(&self, which: i32) -> f32 {
        unsafe { sg::ScriptGame_LabCommonF(which as _) as _ }
    }
    /// `ScriptGame_LabFloorY`
    fn lab_floor_y(&self, x: f32, y: f32, depth: f32) -> f32 {
        unsafe { sg::ScriptGame_LabFloorY(x.to_bits() as i32, y.to_bits() as i32, depth.to_bits() as i32) as _ } // the C passes gs_fbits
    }
    /// `ScriptGame_LabSetShield` (the value as gs_fbits, as gw_script.c passes it)
    fn lab_set_shield(&mut self, slot: i32, health: f32) {
        unsafe { sg::ScriptGame_LabSetShield(slot as _, health.to_bits() as i32) };
    }
    /// `ScriptGame_LabSetScale` (the value as gs_fbits, as gw_script.c passes it)
    fn lab_set_scale(&mut self, slot: i32, scale: f32) {
        unsafe { sg::ScriptGame_LabSetScale(slot as _, scale.to_bits() as i32) };
    }
    /// `ScriptGame_StageKind`
    fn stage_kind(&self) -> i32 {
        unsafe { sg::ScriptGame_StageKind() as _ }
    }
    /// `ScriptGame_GameMode`
    fn game_mode(&self) -> i32 {
        unsafe { sg::ScriptGame_GameMode() as _ }
    }
    /// `ScriptGame_RngSeed`
    fn rng_seed(&self) -> u32 {
        unsafe { sg::ScriptGame_RngSeed() as _ }
    }
    /// `ScriptGame_SetPercent`
    fn set_percent(&mut self, slot: i32, percent: i32) {
        unsafe { sg::ScriptGame_SetPercent(slot as _, percent as _) };
    }
    /// `ScriptGame_ModeActive`
    fn mode_active(&self) -> bool {
        unsafe { sg::ScriptGame_ModeActive() != 0 }
    }
    /// `ScriptGame_LabKnockback`
    fn lab_knockback(&self, slot: i32, attacker: i32, dmg_bits: i32, kbg: i32, wbk: i32, bkb: i32, pct_bits: i32, post: i32) -> f32 {
        unsafe { sg::ScriptGame_LabKnockback(slot as _, attacker as _, dmg_bits as _, kbg as _, wbk as _, bkb as _, pct_bits as _, post as _) as _ }
    }
    /// `ScriptGame_LabKbLevel`
    fn lab_kb_level(&self, kb_bits: i32) -> i32 {
        unsafe { sg::ScriptGame_LabKbLevel(kb_bits as _) as _ }
    }
    /// `ScriptGame_LabFlightF`
    fn lab_flight_f(&self, slot: i32, which: i32) -> f32 {
        unsafe { sg::ScriptGame_LabFlightF(slot as _, which as _) as _ }
    }
    /// `ScriptGame_ArenaOwner`
    fn arena_owner(&self) -> i32 {
        unsafe { sg::ScriptGame_ArenaOwner() as _ }
    }
    /// `ScriptGame_ArenaOrigin`
    fn arena_origin(&mut self, owner: i32, x_bits: i32, y_bits: i32, frames: i32) {
        unsafe { sg::ScriptGame_ArenaOrigin(owner as _, x_bits as _, y_bits as _, frames as _) };
    }
    /// `ScriptGame_ArenaSpawn`
    fn arena_spawn(&mut self, owner: i32, slot: i32, x_bits: i32, y_bits: i32) -> i32 {
        unsafe { sg::ScriptGame_ArenaSpawn(owner as _, slot as _, x_bits as _, y_bits as _) as _ }
    }
    /// `ScriptGame_ArenaSpawnPos`
    fn arena_spawn_pos(&self, slot: i32, axis: i32, fallback: f32) -> f32 {
        unsafe { sg::ScriptGame_ArenaSpawnPos(slot as _, axis as _, fallback as _) as _ }
    }
    /// `ScriptGame_ArenaBounds`
    fn arena_bounds(&mut self, owner: i32, blast: bool, l: i32, r: i32, t: i32, b: i32) {
        unsafe { sg::ScriptGame_ArenaBounds(owner as _, blast as _, l as _, r as _, t as _, b as _) };
    }
    /// `ScriptGame_ArenaRestore`
    fn arena_restore(&mut self) {
        unsafe { sg::ScriptGame_ArenaRestore() };
    }
    /// `ScriptGame_ArenaRead`
    fn arena_read(&self, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ArenaRead(field as _) as _ }
    }
    /// `ScriptGame_ArenaGroup`
    fn arena_group(&self, id: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ArenaGroup(id as _, field as _) as _ }
    }
    /// `ScriptGame_ArenaGroupSet`
    fn arena_group_set(&mut self, owner: i32, id: i32, enabled: bool) {
        unsafe { sg::ScriptGame_ArenaGroupSet(owner as _, id as _, enabled as _) };
    }
    /// `ScriptGame_ArenaRelease`
    fn arena_release(&mut self, owner: i32) {
        unsafe { sg::ScriptGame_ArenaRelease(owner as _) };
    }
    /// `ScriptGame_MapBounds`
    fn map_bounds(&mut self, which: i32, l: i32, r: i32, b: i32, t: i32) {
        unsafe { sg::ScriptGame_MapBounds(which as _, l as _, r as _, b as _, t as _) };
    }
    /// `ScriptGame_MapBound`
    fn map_bound(&self, which: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_MapBound(which as _, field as _) as _ }
    }
    /// `ScriptGame_CapsMovement`
    fn caps_movement(&mut self, entity: i32, owner: i32, jumps: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsMovement(entity as _, owner as _, jumps as _, flags as _) as _ }
    }
    /// `ScriptGame_CapsMovementRead`
    fn caps_movement_read(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsMovementRead(entity as _, field as _) as _ }
    }
    /// `ScriptGame_CapsEffect`
    fn caps_effect(&mut self, entity: i32, owner: i32, kind: i32, value_bits: i32, frames: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsEffect(entity as _, owner as _, kind as _, value_bits as _, frames as _) as _ }
    }
    /// `ScriptGame_CapsEffectRead`
    fn caps_effect_read(&self, entity: i32, kind: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsEffectRead(entity as _, kind as _, field as _) as _ }
    }
    /// `ScriptGame_CapsArmour`
    fn caps_armour(&mut self, entity: i32, owner: i32, damage_bits: i32, kb_bits: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsArmour(entity as _, owner as _, damage_bits as _, kb_bits as _) as _ }
    }
    /// `ScriptGame_CapsArmourRead`
    fn caps_armour_read(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsArmourRead(entity as _, field as _) as _ }
    }
    /// `ScriptGame_CapsGiveItem`
    fn caps_give_item(&mut self, entity: i32, owner: i32, kind: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsGiveItem(entity as _, owner as _, kind as _) as _ }
    }
    /// `ScriptGame_CapsOpponent`
    fn caps_opponent(&self, entity: i32, radius_bits: i32, nth: i32, exclude: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsOpponent(entity as _, radius_bits as _, nth as _, exclude as _) as _ }
    }
    /// `ScriptGame_CapsStatus`
    fn caps_status(&mut self, entity: i32, channel: i32, owner: i32, value_bits: i32, frames: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsStatus(entity as _, channel as _, owner as _, value_bits as _, frames as _) as _ }
    }
    /// `ScriptGame_CapsStatusRead`
    fn caps_status_read(&self, entity: i32, channel: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CapsStatusRead(entity as _, channel as _, field as _) as _ }
    }
    /// `ScriptGame_IntrWinSet`
    fn intr_win_set(&mut self, entity: i32, owner: i32, frames: i32, exits: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_IntrWinSet(entity as _, owner as _, frames as _, exits as _, flags as _) as _ }
    }
    /// `ScriptGame_IntrWinRead`
    fn intr_win_read(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_IntrWinRead(entity as _, field as _) as _ }
    }
    /// `ScriptGame_ArmorSet`
    fn armor_set(&mut self, e: i32, owner: i32, ty: i32, value_bits: i32, frames: i32, state: i32, from: i32, to: i32, dir: i32) -> i32 {
        unsafe { sg::ScriptGame_ArmorSet(e as _, owner as _, ty as _, value_bits as _, frames as _, state as _, from as _, to as _, dir as _) as _ }
    }
    /// `ScriptGame_ArmorRead`
    fn armor_read(&self, e: i32, ty: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ArmorRead(e as _, ty as _, field as _) as _ }
    }
    /// `ScriptGame_ArmorClear`
    fn armor_clear(&mut self, e: i32, owner: i32, ty: i32) -> i32 {
        unsafe { sg::ScriptGame_ArmorClear(e as _, owner as _, ty as _) as _ }
    }
    /// `ScriptGame_CritRead`
    fn crit_read(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CritRead(entity as _, field as _) as _ }
    }
    /// `ScriptGame_CritSet`
    fn crit_set(&mut self, e: i32, owner: i32, slot: i32, chance: i32, mult: i32, mult_max: i32, launch: i32, min_pct: i32, force: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_CritSet(e as _, owner as _, slot as _, chance as _, mult as _, mult_max as _, launch as _, min_pct as _, force as _, flags as _) as _ }
    }
    /// `ScriptGame_CritSeed`
    fn crit_seed(&mut self, seed: i32) {
        unsafe { sg::ScriptGame_CritSeed(seed as _) };
    }
    /// `ScriptGame_FighterBenchCheck`
    fn fighter_bench_check(&self, slot: i32, owner: i32, calling: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterBenchCheck(slot as _, owner as _, calling as _) as _ }
    }
    /// `ScriptGame_FighterBench`
    fn fighter_bench(&mut self, slot: i32, owner: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterBench(slot as _, owner as _) as _ }
    }
    /// `ScriptGame_FighterCall`
    fn fighter_call(&mut self, slot: i32, owner: i32, x: i32, y: i32, facing: i32, intangible: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterCall(slot as _, owner as _, x as _, y as _, facing as _, intangible as _) as _ }
    }
    /// `ScriptGame_FighterBenched`
    fn fighter_benched(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterBenched(slot as _) as _ }
    }
    /// `ScriptGame_FighterReserveRead`
    fn fighter_reserve_read(&self, slot: i32, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterReserveRead(slot as _, entity as _, field as _) as _ }
    }
    /// `ScriptGame_ActionCategory`
    fn action_category(&self, motion: i32) -> i32 {
        unsafe { sg::ScriptGame_ActionCategory(motion as _) as _ }
    }
    /// `ScriptGame_ZoneField`
    fn zone_field(&self, slot: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneField(slot as _, field as _) as _ }
    }
    /// `ScriptGame_ZoneText`
    fn zone_text(&self, slot: i32, text: i32, byte: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneText(slot as _, text as _, byte as _) as _ }
    }
    /// `ScriptGame_ZoneDraft`
    fn zone_draft(&mut self, field: i32, value: i32) {
        unsafe { sg::ScriptGame_ZoneDraft(field as _, value as _) };
    }
    /// `ScriptGame_ZoneDraftText`
    fn zone_draft_text(&mut self, text: i32, byte: i32, value: i32) {
        unsafe { sg::ScriptGame_ZoneDraftText(text as _, byte as _, value as _) };
    }
    /// `ScriptGame_ZoneCommit`
    fn zone_commit(&mut self, handle: i32, owner: i32, serial_floor: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneCommit(handle as _, owner as _, serial_floor as _) as _ }
    }
    /// `ScriptGame_ZoneRemove`
    fn zone_remove(&mut self, handle: i32, owner: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneRemove(handle as _, owner as _) as _ }
    }
    /// `ScriptGame_ZonesRelease`
    fn zones_release(&mut self, owner: i32) {
        unsafe { sg::ScriptGame_ZonesRelease(owner as _) };
    }
    /// `ScriptGame_ZonesArm`
    fn zones_arm(&mut self, mask: i32) {
        unsafe { sg::ScriptGame_ZonesArm(mask as _) };
    }
    /// `ScriptGame_ZonesFrame`
    fn zones_frame(&mut self, terminal: i32) {
        unsafe { sg::ScriptGame_ZonesFrame(terminal as _) };
    }
    /// `ScriptGame_ZoneEntity`
    fn zone_entity(&self, entity: i32, field: i32, zone: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneEntity(entity as _, field as _, zone as _) as _ }
    }
    /// `ScriptGame_ZoneEvent`
    fn zone_event(&self, event: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneEvent(event as _, field as _) as _ }
    }
    /// `ScriptGame_ZoneEventText`
    fn zone_event_text(&self, event: i32, text: i32, byte: i32) -> i32 {
        unsafe { sg::ScriptGame_ZoneEventText(event as _, text as _, byte as _) as _ }
    }
    /// `ScriptGame_ZoneEventCount`
    fn zone_event_count(&self) -> i32 {
        unsafe { sg::ScriptGame_ZoneEventCount() as _ }
    }
    /// `ScriptGame_SkillRowCount`
    fn skill_row_count(&self) -> i32 {
        unsafe { sg::ScriptGame_SkillRowCount() as _ }
    }
    /// `ScriptGame_SkillRow`
    fn skill_row(&self, idx: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_SkillRow(idx as _, field as _) as _ }
    }
    /// `ScriptGame_SkillState`
    fn skill_state(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_SkillState(entity as _, field as _) as _ }
    }
    /// `ScriptGame_SkillConst`
    fn skill_const(&self, idx: i32) -> i32 {
        unsafe { sg::ScriptGame_SkillConst(idx as _) as _ }
    }
    /// `ScriptGame_ShockSet`
    fn shock_set(&mut self, entity: i32, owner: i32, frames: i32, charges: i32, mult_bits: i32, bonus: i32, flags: i32) -> i32 {
        unsafe { sg::ScriptGame_ShockSet(entity as _, owner as _, frames as _, charges as _, mult_bits as _, bonus as _, flags as _) as _ }
    }
    /// `ScriptGame_ShockRead`
    fn shock_read(&self, entity: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_ShockRead(entity as _, field as _) as _ }
    }
    /// `ScriptGame_ContactField`
    fn contact_field(&self, slot: i32, surface: i32, field: i32, line: i32) -> i32 {
        unsafe { sg::ScriptGame_ContactField(slot as _, surface as _, field as _, line as _) as _ }
    }
    /// `ScriptGame_ContactFlags`
    fn contact_flags(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_ContactFlags(slot as _) as _ }
    }
    /// `ScriptGame_ContactCross`
    fn contact_cross(&self, slot: i32, after: i32) -> i32 {
        unsafe { sg::ScriptGame_ContactCross(slot as _, after as _) as _ }
    }
    /// `ScriptGame_CameraParamRead`
    fn camera_param_read(&self, field: i32) -> i32 {
        unsafe { sg::ScriptGame_CameraParamRead(field as _) as _ }
    }
    /// `ScriptGame_CameraParamOwner`
    fn camera_param_owner(&self) -> i32 {
        unsafe { sg::ScriptGame_CameraParamOwner() as _ }
    }
    /// `ScriptGame_FighterHistoryDepth`
    fn fighter_history_depth(&self) -> i32 {
        unsafe { sg::ScriptGame_FighterHistoryDepth() as _ }
    }
    /// `ScriptGame_FighterHistoryRead`
    fn fighter_history_read(&self, entity: i32, age: i32, hitbox: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterHistoryRead(entity as _, age as _, hitbox as _, field as _) as _ }
    }
    /// `ScriptGame_EchoRemove`
    fn echo_remove(&mut self, handle: i32, owner: i32) -> i32 {
        unsafe { sg::ScriptGame_EchoRemove(handle as _, owner as _) as _ }
    }
    /// `ScriptGame_EchoRead`
    fn echo_read(&self, entity: i32, index: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_EchoRead(entity as _, index as _, field as _) as _ }
    }
    /// `ScriptGame_EchoOwner`
    fn echo_owner(&self, entity: i32) -> i32 {
        unsafe { sg::ScriptGame_EchoOwner(entity as _) as _ }
    }
    /// `ScriptGame_EchoNextHandle`
    fn echo_next_handle(&self) -> i32 {
        unsafe { sg::ScriptGame_EchoNextHandle() as _ }
    }
    /// `ScriptGame_EchoState`
    fn echo_state(&self, entity: i32, handle: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_EchoState(entity as _, handle as _, field as _) as _ }
    }
    /// `ScriptGame_CpuCtl`
    fn cpu_ctl(&mut self, slot: i32, sub: i32, op: i32, a: i32) -> i32 {
        unsafe { sg::ScriptGame_CpuCtl(slot as _, sub as _, op as _, a as _) as _ }
    }
    /// `ScriptGame_CpuAttr`
    fn cpu_attr(&self, slot: i32, id: i32) -> i32 {
        unsafe { sg::ScriptGame_CpuAttr(slot as _, id as _) as _ }
    }
    /// `ScriptGame_PartsI`
    fn parts_i(&self, slot: i32, index: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_PartsI(slot as _, index as _, field as _) as _ }
    }
    /// `ScriptGame_PartsF`
    fn parts_f(&self, slot: i32, index: i32, field: i32) -> f32 {
        unsafe { sg::ScriptGame_PartsF(slot as _, index as _, field as _) as _ }
    }
    /// `ScriptGame_PartsControl`
    fn parts_control(&mut self, slot: i32, part: i32, ctl: i32, value: i32, owner: i32) -> i32 {
        unsafe { sg::ScriptGame_PartsControl(slot as _, part as _, ctl as _, value as _, owner as _) as _ }
    }
    /// `ScriptGame_PartsJoint`
    fn parts_joint(&self, slot: i32, joint: i32) -> i32 {
        unsafe { sg::ScriptGame_PartsJoint(slot as _, joint as _) as _ }
    }
    /// `ScriptGame_PartsReset`
    fn parts_reset(&mut self, owner: i32) {
        unsafe { sg::ScriptGame_PartsReset(owner as _) };
    }
    /// `ScriptGame_LaunchScene`
    fn launch_scene(&mut self, mode: i32) {
        unsafe { sg::ScriptGame_LaunchScene(mode as _) };
    }
    /// `ScriptGame_FighterModOwner`
    fn fighter_mod_owner(&self, slot: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterModOwner(slot as _) as _ }
    }
    /// `ScriptGame_FighterModRead`
    fn fighter_mod_read(&self, slot: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterModRead(slot as _, field as _) as _ }
    }
    /// `ScriptGame_FighterModExtra`
    fn fighter_mod_extra(&mut self, slot: i32, owner: i32, fall: i32, weight: i32, regen: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterModExtra(slot as _, owner as _, fall as _, weight as _, regen as _) as _ }
    }
    /// `ScriptGame_EntityField`
    fn entity_field(&self, kind: i32, id: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_EntityField(kind as _, id as _, field as _) as _ }
    }
    /// `ScriptGame_EntityEpoch`
    fn entity_epoch(&self) -> i32 {
        unsafe { sg::ScriptGame_EntityEpoch() as _ }
    }
    /// `ScriptGame_EntityAddressField`
    fn entity_address_field(&self, kind: i32, address: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_EntityAddressField(kind as _, address as _, field as _) as _ }
    }
    /// `ScriptGame_HitRulesOwner`
    fn hit_rules_owner(&self, entity: i32) -> i32 {
        unsafe { sg::ScriptGame_HitRulesOwner(entity as _) as _ }
    }
    /// `ScriptGame_HitRulesCount`
    fn hit_rules_count(&self, entity: i32) -> i32 {
        unsafe { sg::ScriptGame_HitRulesCount(entity as _) as _ }
    }
    /// `ScriptGame_FighterStatus`
    fn fighter_status(&self, entity: i32) -> i32 {
        unsafe { sg::ScriptGame_FighterStatus(entity as _) as _ }
    }
    /// `ScriptGame_HitRuleRead`
    fn hit_rule_read(&self, entity: i32, index: i32, field: i32) -> i32 {
        unsafe { sg::ScriptGame_HitRuleRead(entity as _, index as _, field as _) as _ }
    }

    // ---- the C-string and slice calls (gw_script.c's conversions)
    /// `ScriptGame_LabAnimSymbol`
    fn lab_anim_symbol(&self, slot: i32) -> Option<String> {
        unsafe { cstr(sg::ScriptGame_LabAnimSymbol(slot as _)) }
    }
    /// `ScriptGame_LabAttrName`
    fn lab_attr_name(&self, i: i32) -> Option<String> {
        unsafe { cstr(sg::ScriptGame_LabAttrName(i as _)) }
    }
    /// `ScriptGame_LabAnimSymbolFor`
    fn lab_anim_symbol_for(&self, slot: i32, anim: i32) -> Option<String> {
        unsafe { cstr(sg::ScriptGame_LabAnimSymbolFor(slot as _, anim as _)) }
    }
    /// `ScriptGame_CapsGiveItemReason` (the C host prints the string as is; NULL is the empty string)
    fn caps_give_item_reason(&self, reason: i32) -> String {
        unsafe { cstr(sg::ScriptGame_CapsGiveItemReason(reason as _)) }.unwrap_or_default()
    }
    /// `ScriptGame_LabScript`: the animation's subaction script. Raw archive images live in the OS arena below 4 GiB
    /// (lowmem.rs), so its address is the 32-bit guest word the C host hands Lua.
    fn lab_script(&self, slot: i32, anim: i32) -> Option<u32> {
        let p = unsafe { sg::ScriptGame_LabScript(slot as _, anim as _) } as usize;
        (p != 0 && p <= u32::MAX as usize).then_some(p as u32)
    }
    /// gw_script.c's `gs_guest_ok`: `[addr, addr + len)` inside game memory (the OS arena: heaps and archive images).
    fn guest_ok(&self, addr: u32, len: u32) -> bool {
        if !crate::host::Host::is_booted() {
            return false;
        }
        let h = crate::host::Host::get();
        let (lo, hi) = (h.os.mem.base as usize, h.os.mem.base as usize + h.os.mem.len);
        let (a, n) = (addr as usize, len as usize);
        a >= lo && a.checked_add(n).is_some_and(|e| e <= hi)
    }
    /// gw_script.c's `gs_guest_word`: the big-endian word at a guest address (0 outside game memory).
    fn guest_word(&self, addr: u32) -> u32 {
        if !self.guest_ok(addr, 4) {
            return 0;
        }
        u32::from_be_bytes(unsafe { *(addr as usize as *const [u8; 4]) })
    }
    /// `ScriptGame_CameraParams(owner, mask, a..k)`: the eleven float bit patterns (gw_script_camera_params.inc).
    fn camera_params(&mut self, owner: i32, mask: i32, bits: &[i32]) -> i32 {
        let b = |i: usize| bits.get(i).copied().unwrap_or(0);
        unsafe {
            sg::ScriptGame_CameraParams(owner, mask, b(0), b(1), b(2), b(3), b(4), b(5), b(6), b(7), b(8), b(9)) as _
        }
    }
    /// `ScriptGame_EchoAddWithHandle(entity, owner, delay, move, match_element, airborne, damage, knockback, element,
    /// once, handle)`: `rest` is the eight fields after the owner (gw_script_echo.inc passes row[1..9]).
    fn echo_add_with_handle(&mut self, entity: i32, owner: i32, rest: &[i32], handle: i32) -> i32 {
        let b = |i: usize| rest.get(i).copied().unwrap_or(0);
        unsafe { sg::ScriptGame_EchoAddWithHandle(entity, owner, b(0), b(1), b(2), b(3), b(4), b(5), b(6), b(7), handle) as _ }
    }
    /// `ScriptGame_EchoReplace(entity, owner, count)`: the rules are read back through `Script_EchoInput(i)` during the
    /// call, as the C host points `gs_echo_input` at them (gs_echo_apply).
    fn echo_replace(&mut self, entity: i32, owner: i32, count: i32, rules: &[i32]) -> i32 {
        with_input(&ECHO_INPUT, rules, || unsafe { sg::ScriptGame_EchoReplace(entity, owner, count) as _ })
    }
    /// `ScriptGame_FighterMod(slot, owner, enabled, a..h)`: the eight bit patterns (gw_script_sim_state.inc).
    fn fighter_mod(&mut self, slot: i32, owner: i32, enabled: bool, bits: &[i32]) -> i32 {
        let b = |i: usize| bits.get(i).copied().unwrap_or(0);
        unsafe { sg::ScriptGame_FighterMod(slot, owner, enabled as _, b(0), b(1), b(2), b(3), b(4), b(5), b(6), b(7)) as _ }
    }
    /// `ScriptGame_HitRulesReplace(port, owner, count, status)`: the rules through `Script_HitRulesInput(i)` (gs_hr_apply).
    fn hit_rules_replace(&mut self, entity: i32, owner: i32, count: i32, status: i32, rules: &[i32]) -> i32 {
        with_input(&HIT_RULES_INPUT, rules, || unsafe { sg::ScriptGame_HitRulesReplace(entity, owner, count, status) as _ })
    }
}

/// A C string the game side returned (NUL-terminated, static or per-call buffer), None for NULL.
unsafe fn cstr(p: *const core::ffi::c_char) -> Option<String> {
    (!p.is_null()).then(|| unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

/// The rules a replace call reads back by index while it runs (the C host's `gs_echo_input` / `gs_hit_rules_input`).
static ECHO_INPUT: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());
static HIT_RULES_INPUT: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());

fn with_input<R>(slot: &std::sync::Mutex<Vec<i32>>, rules: &[i32], f: impl FnOnce() -> R) -> R {
    *slot.lock().unwrap_or_else(|e| e.into_inner()) = rules.to_vec();
    let r = f();
    slot.lock().unwrap_or_else(|e| e.into_inner()).clear();
    r
}

fn input_at(slot: &std::sync::Mutex<Vec<i32>>, i: u64) -> u64 {
    slot.lock().unwrap_or_else(|e| e.into_inner()).get(i as usize).copied().unwrap_or(0) as u32 as u64
}

/// The game-side callees into the host (`Script_*`, script_game's `ext_fns!`) reached by name, answered by the host
/// the C links them to (pc/platform/gw_script*.c). None: not answered here (the seams' "feature off" defaults apply).
pub fn by_name(name: &str, args: &[u64]) -> Option<u64> {
    let a = |i: usize| args.get(i).copied().unwrap_or(0);
    match name {
        // gw_Script_EchoInput / gw_Script_HitRulesInput: the staged rules of the replace call in progress
        "Script_EchoInput" => Some(input_at(&ECHO_INPUT, a(0))),
        "Script_HitRulesInput" => Some(input_at(&HIT_RULES_INPUT, a(0))),
        _ => None,
    }
}
