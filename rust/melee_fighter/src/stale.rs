//! Move staling and per-player attack bookkeeping: hand port of src/melee/pl/plstale.c,
//! src/melee/pl/plattack.c, the staling half of src/melee/ft/ft_0881.c (ft_800890BC..
//! ft_800892A0) and the player-table accessors they use (src/melee/pl/player.c).
//!
//! The C keeps these as globals (`staleAttackInstance`, `unk_804D6480`, `player_slots`,
//! `Fighter_804D6548`); here they are the same globals, behind safe accessors. Game state is
//! single-threaded. Checked against the C by tests/stale_ref.rs.
#![allow(static_mut_refs)]

use crate::types::{
    plActionStats, plAttackStats, Fighter, StaleMoveTable, StaticPlayer, Gm_PKind,
    Gm_PKind_Cpu, Gm_PKind_Human, StatsAttack_Count,
};

/// `Gm_Player_NumMax`.
pub const PLAYER_NUM_MAX: usize = 6;
/// Entries in `StaleMoveTable::StaleMoves`.
const STALE_LEN: i32 = 10;

/// `player_slots` (player.c .bss).
pub static mut PLAYER_SLOTS: [StaticPlayer; PLAYER_NUM_MAX] = unsafe { core::mem::zeroed() };
/// `staleAttackInstance` (plstale.c).
pub static mut STALE_ATTACK_INSTANCE: u16 = 0;
/// `unk_804D6480` (plattack.c).
pub static mut ATTACK_COUNTER: u16 = 0;
/// `Fighter_804D6548`: the 9 staling decrements, from PlCo (`pData[3]`, fighter.c). Installed by
/// the fighter loader. TODO(move-to-core): owned by ftdata/fighter init once ported.
pub static mut STALE_DECREMENTS: *const f32 = core::ptr::null();
/// `DbLevel` (db.c, group 1); at `DbLKind_DebugRom` (3) and above staling is off.
pub static mut DB_LEVEL: i32 = 0;
pub const DBL_KIND_DEBUG_ROM: i32 = 3;

/// `ftMapping` (player.c), one row per CharacterKind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FtMapping {
    pub internal_id: i8,
    pub extra_internal_id: i8,
    pub has_transformation: i8,
}

/// `Player_CheckSlot`: the C reports through HSD_ASSERTREPORT and carries on; an out-of-range
/// slot is a bug either way, so this panics.
#[inline]
fn check_slot(slot: i32) -> usize {
    assert!((0..PLAYER_NUM_MAX as i32).contains(&slot), "cant get player struct! {slot}");
    slot as usize
}

/// `Player_GetPtrForSlot`.
pub fn player(slot: i32) -> &'static mut StaticPlayer {
    let i = check_slot(slot);
    unsafe { &mut PLAYER_SLOTS[i] }
}

/// `Player_GetStaleMoveTableIndexPtr`.
pub fn stale_table(slot: i32) -> &'static mut StaleMoveTable {
    &mut player(slot).stale_moves
}

/// `Player_GetActionStats`.
pub fn action_stats(slot: i32) -> &'static mut plActionStats {
    &mut stale_table(slot).total_attack_count_struct
}

/// `Player_GetStocks` (stocks is an s8 in the table).
pub fn stocks(slot: i32) -> i32 {
    player(slot).stocks as i32
}

/// `Player_SetStocks`.
pub fn set_stocks(slot: i32, stocks: i32) {
    player(slot).stocks = stocks as i8;
}

/// `Player_GetPlayerSlotType`.
pub fn slot_type(slot: i32) -> Gm_PKind {
    player(slot).pkind
}

/// `Player_SetSlottype`.
pub fn set_slot_type(slot: i32, v: Gm_PKind) {
    player(slot).pkind = v;
}

/// `Player_8003248C(slot, is_follower)`: the slot type, except a follower of a character
/// without a transformation (Nana) reads as CPU (1) when the leader is human or CPU.
/// `has_transformation` is `ftMapping_list[ckind].has_transformation`.
pub fn slot_type_for(slot: i32, follower: bool, has_transformation: bool) -> Gm_PKind {
    let p = player(slot);
    if follower && !has_transformation && (p.pkind == Gm_PKind_Human || p.pkind == Gm_PKind_Cpu) {
        return 1;
    }
    p.pkind
}

// ---- plstale.c

/// `plStale_InitAttackInstance`.
pub fn init_attack_instance() {
    unsafe { STALE_ATTACK_INSTANCE = 1 };
}

/// `plStale_ResetStaleMoveTableForPlayer`.
pub fn reset_stale_move_table(slot: i32) {
    let t = stale_table(slot);
    t.current_index = 0;
    for m in t.StaleMoves.iter_mut() {
        m.move_id = 0;
        m.attack_instance = 0;
    }
}

/// `plStale_IncrementAttackInstance`: returns the old value; skips 0 on wrap.
pub fn increment_attack_instance() -> u16 {
    unsafe {
        let before = STALE_ATTACK_INSTANCE;
        STALE_ATTACK_INSTANCE = STALE_ATTACK_INSTANCE.wrapping_add(1);
        if STALE_ATTACK_INSTANCE == 0 {
            STALE_ATTACK_INSTANCE = 1;
        }
        before
    }
}

/// The shared body of plStale_UpdateStaleMovesFrom{Fighter,Item}.
fn push_stale(t: &mut StaleMoveTable, attack_id: i32, instance: u16) {
    if attack_id == 1 {
        return;
    }
    for m in t.StaleMoves.iter() {
        if attack_id == m.move_id as i32 && instance == m.attack_instance {
            return;
        }
    }
    let i = t.current_index as usize;
    t.StaleMoves[i].move_id = attack_id as u16;
    t.StaleMoves[i].attack_instance = instance;
    t.current_index = if t.current_index == STALE_LEN - 1 { 0 } else { t.current_index + 1 };
}

/// `plStale_UpdateStaleMovesFromFighter(gobj, gobj_2)`: `same_gobj` is `gobj == gobj_2`.
pub fn update_stale_moves_from_fighter(ft: &Fighter, same_gobj: bool) {
    if !same_gobj {
        let (instance, attack_id) = (ft.x206C_attack_instance, ft.x2068_attackID);
        push_stale(stale_table(ft.player_id as i32), attack_id, instance);
    }
}

/// `plStale_UpdateStaleMovesFromItem(gobj, gobj_2)`: the item's `xD88_attackID` /
/// `xD8C_attack_instance`; `owner` is the item's owner when `ftLib_80086960(owner)` (it is a
/// fighter) and `owner != gobj_2`, else None.
pub fn update_stale_moves_from_item(owner: Option<&Fighter>, attack_id: i32, instance: u16) {
    if let Some(o) = owner {
        push_stale(stale_table(o.player_id as i32), attack_id, instance);
    }
}

// ---- plattack.c

/// `plAttack_80037590`.
pub fn attack_counter_init() {
    unsafe { ATTACK_COUNTER = 1 };
}

fn clear_attack_stats(s: &mut plAttackStats) {
    for c in s.by_attack_counts.iter_mut().take(StatsAttack_Count as usize) {
        *c = 0;
    }
    s.total = 0;
    s.thrown_item_count = 0;
    s.aerials_count = 0;
    s.specials_count = 0;
    s.x1A0_count = 0;
    s.x1A4_count = 0;
    s.x1A8 = 0;
}

/// `plAttack_8003759C`: reset a player's action stats.
pub fn reset_action_stats(slot: u32) {
    let st = action_stats(slot as i32);
    clear_attack_stats(&mut st.attacks);
    clear_attack_stats(&mut st.hits);
    clear_attack_stats(unsafe { &mut st.anon_358.x358_hits });
    for v in st.x504.iter_mut().take(StatsAttack_Count as usize) {
        *v = 0;
    }
    st.x568 = 0;
    st.x570 = 0;
    st.x56C = 0;
    st.x574 = 0;
    st.x578 = 0;
    st.x57C = 0;
    st.x580 = 0;
    st.x584 = 0;
    st.x588 = 0;
    st.x58C = 0;
    st.x590 = 0;
    st.x594 = 0;
    for v in st.x598.iter_mut() {
        *v = 0;
    }
    st.set_x5BC_b0(0);
    st.set_x5BC_b1(0);
    st.set_x5BC_b2(0);
    st.set_x5BC_b3(0);
}

/// `plAttack_80037B08`: returns the old counter; skips 0 on wrap.
pub fn attack_counter_next() -> u16 {
    unsafe {
        let before = ATTACK_COUNTER;
        ATTACK_COUNTER = ATTACK_COUNTER.wrapping_add(1);
        if ATTACK_COUNTER == 0 {
            ATTACK_COUNTER = 1;
        }
        before
    }
}

// ---- ft_0881.c (staling)

/// `ft_800890BC`.
pub fn clear_attack_id(fp: &mut Fighter) {
    fp.x2068_attackID = 1;
    fp.x206C_attack_instance = 0;
}

fn set_attack_id(fp: &mut Fighter, move_id: i32) {
    fp.x2068_attackID = move_id;
    fp.x206C_attack_instance = increment_attack_instance();
}

/// `ft_800890D0`: a new attack instance unless the same (non-1) move is still running.
pub fn new_attack(fp: &mut Fighter, move_id: u32) {
    if move_id == 1 || move_id != fp.x2068_attackID as u32 {
        set_attack_id(fp, move_id as i32);
    }
}

/// `ft_800892A0`: restart the current attack's instance (the move hits again).
pub fn renew_attack_instance(fp: &mut Fighter) {
    // inlineC0(fp, fp->x2068_attackID): compares the id with itself, so always true
    let id = fp.x2068_attackID;
    set_attack_id(fp, id);
}

/// `ft_80089118`: the staling multiplier of `move_id` in `table` (1.0 when fresh).
/// `decrements` is `Fighter_804D6548` (at least 9 entries).
pub fn stale_multiplier(table: &StaleMoveTable, move_id: i32, decrements: &[f32]) -> f32 {
    let mut m = 1.0f32;
    let mut k = if table.current_index != 0 { table.current_index - 1 } else { 9 };
    if move_id == 1 {
        return 1.0;
    }
    for d in decrements.iter().take(9) {
        let e = &table.StaleMoves[k as usize];
        if e.move_id == 0 {
            return m;
        }
        if move_id == e.move_id as i32 {
            m -= *d;
        }
        k = if k != 0 { k - 1 } else { 9 };
    }
    m
}

/// `ft_80089228(fp, attack_id, arg2, damage)`: damage after staling.
pub fn apply_staling(fp: &Fighter, attack_id: i32, _arg2: i32, damage: f32) -> f32 {
    if unsafe { DB_LEVEL } >= DBL_KIND_DEBUG_ROM {
        return damage;
    }
    let dec = unsafe {
        assert!(!STALE_DECREMENTS.is_null(), "Fighter_804D6548 not loaded");
        core::slice::from_raw_parts(STALE_DECREMENTS, 9)
    };
    let mut d = damage;
    let m = stale_multiplier(stale_table(fp.player_id as i32), attack_id, dec);
    if m != 1.0 {
        d *= m;
    }
    d
}
