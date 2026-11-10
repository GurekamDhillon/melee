//! In-match tag swap and the sim transition record (rust/mods/tag_fighter, host imports `gd_tag_*` /
//! `gd_transition_*` in melee_modhost).
//!
//! A tag pair rides the retail Zelda / Sheik transform path, which already swaps a fighter in place and is plain
//! game state (so it rolls back and resimulates like any other frame):
//! * spawn: `Player_80031AD0` (pl/player.c) creates the partner as the player's sub-fighter with
//!   `has_transformation` set, so `Fighter_Create` puts it to sleep (`ftCo_800BFD04`) beside the main one. The
//!   partner kind comes from `ftMapping_list[ckind].extra_internal_id`, set by `Player_MexSetPartner` from
//!   `MELEE_TAG_PARTNER` (melee_fighter `tu::player::tag_apply_partner`).
//! * swap: `ftCommon_8007EFC8` (ft/ftcommon.c, what `ftZd_SpecialLw` / `ftSk_SpecialLw` call) swaps
//!   `Player.transformed`, wakes the partner (`Fighter_UnkInitReset_80067C98`), copies position, facing, velocity,
//!   percent, items and camera, sleeps the outgoing fighter and runs the entry callback on the incoming one (here
//!   `ft_8008A2BC`, Wait). Afterwards the incoming fighter's OWN percent is restored (each character keeps its
//!   percent); stocks are the player's, so the team shares them.
//!
//! The transition (shared shape with the stage switch, port/stagesw; envoy-plan.md section 3): REQUESTED (the mod
//! calls `begin`) -> FREEZE (game procs masked for `len` frames, the retail pause mechanism: `gm_SetDbPauseFlag` with
//! the mask row `gm_803DA888[TRANSITION_PAUSE_BIT]`, the same mask as the Start pause) -> SWAP (applied between
//! logic frames, after the `frame.run_procs` post chain) -> RELEASE (mask cleared; the next frame runs). Everything
//! here is a static, so the snapshot covers it; the cinematic is drawn by a presentation mod from this record and
//! never writes back.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_fighter::tu::player as pl;
use melee_types::*;

pub const TAG_NOT_GAMEPLAY: i32 = -1;
pub const TAG_NO_PARTNER: i32 = -2;
pub const TAG_NOT_ACTIONABLE: i32 = -3;
pub const TAG_BUSY: i32 = -9;

/// The pause-mask row the transition freeze uses (`gm_803DA888[7]`, unused by retail; TARGET_PC sets it to the
/// Start pause's row so a player's own pause never clears the freeze).
pub const TRANSITION_PAUSE_BIT: i32 = 7;

pub const PHASE_IDLE: u8 = 0;
pub const PHASE_FREEZE: u8 = 1;
pub const KIND_TAG: u8 = 1;
/// Reserved for the stage switch (rust/mods/stage_switch) when it adopts the freeze.
pub const KIND_STAGE: u8 = 2;
/// A plain hold (the Envoy reward pick): no change at the release, released by the mod (`hold_release`).
pub const KIND_HOLD: u8 = 3;

/// The transition record (sim state; read-only for presentation).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Transition {
    pub phase: u8,
    pub kind: u8,
    pub slot: i8,
    pub skipped: u8,
    /// Frames spent frozen so far, and the planned length.
    pub frame: u16,
    pub len: u16,
    /// Fighter kinds of the outgoing and incoming fighter.
    pub from_kind: i16,
    pub to_kind: i16,
    /// `gm_match_frame`-like logic frame count at the last release (presentation: the entry reveal).
    pub released_at: u32,
    /// Releases so far this boot (presentation edge detect).
    pub count: u32,
}

pub static mut TRANSITION: Transition =
    Transition { phase: 0, kind: 0, slot: -1, skipped: 0, frame: 0, len: 0, from_kind: -1, to_kind: -1, released_at: 0, count: 0 };
/// Logic frames since boot (counted in `tick`), the clock the record's `released_at` uses.
pub static mut LOGIC_FRAMES: u32 = 0;
/// A swap queued by the mod (`gd_tag_swap`), applied after the frame's post chain.
pub static mut PENDING_SWAP: i8 = -1;
/// A stage transition's destination (ground kind) and the freeze frame its switch is applied at (KIND_STAGE).
pub static mut STAGE_TO: i32 = -1;
pub static mut STAGE_AT: u16 = 0;
/// A queued release of a KIND_HOLD freeze.
pub static mut PENDING_RELEASE: u8 = 0;

unsafe fn entity(slot: i32, i: i32) -> *mut HSD_GObj {
    let p = &*pl::Player_GetPtrForSlot(slot);
    p.player_entity[p.transformed[i as usize] as usize]
}

unsafe fn fighter(g: *mut HSD_GObj) -> *mut Fighter {
    if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

/// The partner's fighter kind of `slot` (the sleeping sub-fighter), or -1 if the slot has no tag pair.
pub unsafe fn partner_kind(slot: i32) -> i32 {
    if !(0..4).contains(&slot) || pl::TAG_SLOT[slot as usize] == 0 || crate::tu::petey::partner_caged() {
        return -1;
    }
    if pl::tag_inplace() {
        // the in-place prototype: the benched character, which has no fighter object
        return pl::tag_internal_kind(pl::TAG_BENCH_CK[slot as usize] as i32);
    }
    let f = fighter(entity(slot, 1));
    if f.is_null() { -1 } else { (*f).kind as i32 }
}

/// Whether `slot` could tag now (0) or why not: grounded, actionable (Wait / walk / dash / run / turn / squat),
/// not grabbed or grabbing, no item held, no transition running.
pub unsafe fn check(slot: i32) -> i32 {
    check_ex(slot, false)
}

/// `check`, optionally also in the air (actionable airborne: jump, aerial jump, fall).
pub unsafe fn check_ex(slot: i32, air: bool) -> i32 {
    if TRANSITION.phase != PHASE_IDLE || PENDING_SWAP >= 0 {
        return TAG_BUSY;
    }
    if partner_kind(slot) < 0 {
        return TAG_NO_PARTNER;
    }
    let f = fighter(entity(slot, 0));
    if f.is_null() {
        return TAG_NOT_GAMEPLAY;
    }
    let f = &*f;
    let m = f.motion_id as i32;
    let ok = matches!(m as ftCommon_MotionState,
        ftCo_MS_Wait | ftCo_MS_WalkSlow | ftCo_MS_WalkMiddle | ftCo_MS_WalkFast | ftCo_MS_Turn | ftCo_MS_Dash
        | ftCo_MS_Run | ftCo_MS_RunBrake | ftCo_MS_Squat | ftCo_MS_SquatWait | ftCo_MS_SquatRv);
    let air_ok = air && f.ground_or_air != GA_Ground && matches!(m as ftCommon_MotionState,
        ftCo_MS_JumpF | ftCo_MS_JumpB | ftCo_MS_JumpAerialF | ftCo_MS_JumpAerialB | ftCo_MS_Fall | ftCo_MS_FallF
        | ftCo_MS_FallB | ftCo_MS_FallAerial | ftCo_MS_FallAerialF | ftCo_MS_FallAerialB);
    if air_ok && f.victim_gobj.is_null() && f.x1A5C.is_null() && f.item_gobj.is_null() {
        return 0;
    }
    if !ok || f.ground_or_air != GA_Ground || !f.victim_gobj.is_null() || !f.x1A5C.is_null() || !f.item_gobj.is_null() {
        return TAG_NOT_ACTIONABLE;
    }
    0
}

/// Start the freeze for a tag of `slot` lasting `len` frames. Returns `check`'s code; on 0 the next logic frame is
/// frozen.
pub unsafe fn begin(slot: i32, len: i32) -> i32 {
    begin_ex(slot, len, false)
}

/// `begin` with air tags allowed (`air`): the incoming fighter enters Fall instead of Wait.
pub unsafe fn begin_ex(slot: i32, len: i32, air: bool) -> i32 {
    let rc = check_ex(slot, air);
    if rc != 0 {
        return rc;
    }
    let from = fighter(entity(slot, 0));
    freeze_begin(KIND_TAG, slot, len, (*from).kind as i32, partner_kind(slot));
    0
}

/// The kind-neutral freeze (also meant for the stage switch, port/stagesw: kind 2, from / to = ground kinds, its
/// change applied at the release): freeze the procs from the next logic frame on and fill the record.
pub unsafe fn freeze_begin(kind: u8, slot: i32, len: i32, from: i32, to: i32) {
    TRANSITION.phase = PHASE_FREEZE;
    TRANSITION.kind = kind;
    TRANSITION.slot = slot as i8;
    TRANSITION.skipped = 0;
    TRANSITION.frame = 0;
    TRANSITION.len = len.clamp(1, 3600) as u16;
    TRANSITION.from_kind = from as i16;
    TRANSITION.to_kind = to as i16;
    crate::tu::gmscene::gm_SetDbPauseFlag(TRANSITION_PAUSE_BIT);
}

/// Queue the swap + release (`skip` != 0: the player skipped the cinematic). Applied by `apply_pending`.
pub unsafe fn request_swap(skip: bool) -> i32 {
    if TRANSITION.phase != PHASE_FREEZE {
        return TAG_BUSY;
    }
    TRANSITION.skipped = skip as u8;
    PENDING_SWAP = TRANSITION.slot;
    0
}

/// Per logic frame, after the `frame.run_procs` post chain (outside `HSD_GObj_RunProcs`).
pub unsafe fn tick() {
    LOGIC_FRAMES = LOGIC_FRAMES.wrapping_add(1);
    if PENDING_SWAP >= 0 {
        let slot = PENDING_SWAP as i32;
        PENDING_SWAP = -1;
        swap(slot);
        TRANSITION.phase = PHASE_IDLE;
        TRANSITION.released_at = LOGIC_FRAMES;
        TRANSITION.count = TRANSITION.count.wrapping_add(1);
        crate::tu::gmscene::gm_ClearDbPauseFlag(TRANSITION_PAUSE_BIT);
    } else if PENDING_RELEASE != 0 {
        PENDING_RELEASE = 0;
        release();
    } else if TRANSITION.phase == PHASE_FREEZE {
        TRANSITION.frame = TRANSITION.frame.saturating_add(1);
        if TRANSITION.kind == KIND_STAGE {
            if TRANSITION.frame == STAGE_AT && STAGE_TO >= 0 {
                let rc = crate::tu::stage_hop::switch_to(STAGE_TO);
                if rc != 0 {
                    eprintln!("transition: stage switch to {STAGE_TO} refused at apply ({rc})");
                }
                STAGE_TO = -1;
            }
            if TRANSITION.frame >= TRANSITION.len {
                release();
            }
        }
    }
}

unsafe fn release() {
    TRANSITION.phase = PHASE_IDLE;
    TRANSITION.released_at = LOGIC_FRAMES;
    TRANSITION.count = TRANSITION.count.wrapping_add(1);
    crate::tu::gmscene::gm_ClearDbPauseFlag(TRANSITION_PAUSE_BIT);
}

/// A stage switch under the transition freeze: frozen for `len` frames, the switch applied halfway (under the cover),
/// released at the end. Returns `stage_hop::check`'s code, or TAG_BUSY while another transition runs.
pub unsafe fn stage_begin(grkind: i32, len: i32) -> i32 {
    if TRANSITION.phase != PHASE_IDLE || PENDING_SWAP >= 0 || PENDING_RELEASE != 0 {
        return TAG_BUSY;
    }
    if crate::tu::petey::active() {
        return crate::tu::stage_hop::HOP_UNSUPPORTED; // the boss arena stays put
    }
    let rc = crate::tu::stage_hop::check(grkind);
    if rc != 0 {
        return rc;
    }
    let len = len.clamp(2, 600);
    STAGE_TO = grkind;
    STAGE_AT = (len / 2) as u16;
    freeze_begin(KIND_STAGE, -1, len, crate::tu::stage_hop::current(), grkind);
    0
}

/// A hold freeze (KIND_HOLD) of at most `len` frames; the mod releases it (`hold_release`).
pub unsafe fn hold_begin(len: i32) -> i32 {
    if TRANSITION.phase != PHASE_IDLE || PENDING_SWAP >= 0 || PENDING_RELEASE != 0 {
        return TAG_BUSY;
    }
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return TAG_NOT_GAMEPLAY;
    }
    freeze_begin(KIND_HOLD, -1, len, -1, -1);
    0
}

pub unsafe fn hold_release() -> i32 {
    if TRANSITION.phase != PHASE_FREEZE || TRANSITION.kind != KIND_HOLD {
        return TAG_BUSY;
    }
    PENDING_RELEASE = 1;
    0
}

/// The swap itself (`ftCommon_8007EFC8` with Wait as the entry), keeping each character's own percent.
unsafe fn swap(slot: i32) {
    if pl::tag_inplace() {
        return swap_inplace(slot);
    }
    let out_g = entity(slot, 0);
    let in_g = entity(slot, 1);
    if fighter(out_g).is_null() || fighter(in_g).is_null() {
        return;
    }
    let own = (*fighter(in_g)).dmg.x1830_percent;
    if (*fighter(out_g)).ground_or_air == GA_Ground {
        melee_fighter::tu::ftcommon::ftcommon_8007efc8(out_g, |g| melee_fighter::tu::ft_08a1::ft_8008A2BC(g));
    } else {
        melee_fighter::tu::ftcommon::ftcommon_8007efc8(out_g, |g| melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(g));
    }
    let f = &mut *fighter(in_g);
    f.dmg.x1830_percent = own;
    pl::Player_SetHPByIndex(slot, f.is_sub_fighter() as i32, melee_core::ppc::fctiwz_f32(own));
    if std::env::var_os("MELEE_TAG_QUIET").is_none() {
        eprintln!("tag: slot {slot} {} -> {} at frame {}", TRANSITION.from_kind, TRANSITION.to_kind, LOGIC_FRAMES);
    }
}

/// The in-place prototype (`MELEE_TAG_INPLACE=1`, notes/envoy-tag-swap.md): spawn the benched character on this slot
/// as a sleeping sub-fighter, run the retail transform swap onto it (position, velocity, input, damage state...), then
/// free the outgoing fighter and bench its character and percent in plain statics. The slot owns one fighter object
/// between tags, so a character with its own sub-fighter (Ice Climbers, Zelda / Sheik) is not blocked by the tag.
unsafe fn swap_inplace(slot: i32) {
    let s = slot as usize;
    let p = &mut *pl::Player_GetPtrForSlot(slot);
    let out_idx = p.transformed[0] as usize;
    let free_idx = 1 - out_idx;
    let out_g = p.player_entity[out_idx];
    let ck_in = pl::TAG_BENCH_CK[s] as i32;
    if fighter(out_g).is_null() || ck_in < 0 {
        return;
    }
    if !p.player_entity[free_idx].is_null() {
        // the active character owns a sub-fighter (Nana): the prototype does not carry pairs yet
        eprintln!("tag: slot {slot} in-place swap refused: the entity slot is taken");
        return;
    }
    let t0 = std::time::Instant::now();
    let in_g = pl::tag_inplace_spawn(slot, free_idx, ck_in as CharacterKind);
    let t1 = std::time::Instant::now();
    if fighter(in_g).is_null() {
        return;
    }
    let own = pl::TAG_BENCH_PCT[s];
    let out_pct = (*fighter(out_g)).dmg.x1830_percent;
    let ck_out = p.ckind as i32;
    if (*fighter(out_g)).ground_or_air == GA_Ground {
        melee_fighter::tu::ftcommon::ftcommon_8007efc8(out_g, |g| melee_fighter::tu::ft_08a1::ft_8008A2BC(g));
    } else {
        melee_fighter::tu::ftcommon::ftcommon_8007efc8(out_g, |g| melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(g));
    }
    let f = &mut *fighter(in_g);
    f.dmg.x1830_percent = own;
    pl::Player_SetHPByIndex(slot, f.is_sub_fighter() as i32, melee_core::ppc::fctiwz_f32(own));
    let t2 = std::time::Instant::now();
    pl::tag_inplace_free(slot, out_idx);
    let t3 = std::time::Instant::now();
    p.ckind = ck_in as CharacterKind;
    pl::TAG_BENCH_CK[s] = ck_out as i8;
    pl::TAG_BENCH_PCT[s] = out_pct;
    // wall-clock diagnostics stay local (never in a static the snapshot would compare)
    let us = [(t1 - t0).as_micros(), (t3 - t2).as_micros()];
    if std::env::var_os("MELEE_TAG_QUIET").is_none() {
        eprintln!("tag: slot {slot} {} -> {} at frame {} (in place: spawn {} us, despawn {} us)", TRANSITION.from_kind,
            TRANSITION.to_kind, LOGIC_FRAMES, us[0], us[1]);
    }
}

/// A new match: no transition carried over (the mask bit is the scene's; clear it if a match ended frozen).
pub unsafe fn reset() {
    if TRANSITION.phase != PHASE_IDLE {
        crate::tu::gmscene::gm_ClearDbPauseFlag(TRANSITION_PAUSE_BIT);
    }
    TRANSITION.phase = PHASE_IDLE;
    TRANSITION.slot = -1;
    PENDING_SWAP = -1;
    PENDING_RELEASE = 0;
    STAGE_TO = -1;
}
