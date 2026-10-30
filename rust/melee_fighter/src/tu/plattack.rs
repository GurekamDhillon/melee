//! Port of src/melee/pl/plattack.c: the per-player attack-stat counters and the global attack
//! instance counter (`unk_804D6480`, shared with `crate::stale::ATTACK_COUNTER`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use super::player::Player_GetActionStats;
use melee_types::*;

pub use crate::stale::ATTACK_COUNTER as unk_804D6480;

/// `plAttack_80037590`
pub fn plAttack_80037590() {
    melee_core::ct!("plAttack_80037590");
    unsafe { unk_804D6480 = 1 };
}

/// `clearAttackStats` (static inline)
fn clearAttackStats(s: &mut plAttackStats) {
    for i in 0..StatsAttack_Count as usize {
        s.by_attack_counts[i] = 0;
    }
    s.total = 0;
    s.thrown_item_count = 0;
    s.aerials_count = 0;
    s.specials_count = 0;
    s.x1A0_count = 0;
    s.x1A4_count = 0;
    s.x1A8 = 0;
}

/// `plAttack_8003759C`: resets the player's stats
pub fn plAttack_8003759C(slot: u32) {
    melee_core::ct!("plAttack_8003759C");
    let stats = unsafe { &mut *Player_GetActionStats(slot as i32) };

    clearAttackStats(&mut stats.attacks);
    clearAttackStats(&mut stats.hits);
    clearAttackStats(unsafe { &mut stats.anon_358.x358_hits });

    for i in 0..StatsAttack_Count as usize {
        stats.x504[i] = 0;
    }
    stats.x568 = 0;
    stats.x570 = 0;
    stats.x56C = 0;
    stats.x574 = 0;
    stats.x578 = 0;
    stats.x57C = 0;
    stats.x580 = 0;
    stats.x584 = 0;
    stats.x588 = 0;
    stats.x58C = 0;
    stats.x590 = 0;
    stats.x594 = 0;

    for i in 0..8 {
        stats.x598[i] = 0;
    }
    stats.set_x5BC_b0(0);
    stats.set_x5BC_b1(0);
    stats.set_x5BC_b2(0);
    stats.set_x5BC_b3(0);
}

/// `plAttack_80037B08`: returns the old counter, skipping 0 on wrap
pub fn plAttack_80037B08() -> u16 {
    melee_core::ct!("plAttack_80037B08");
    unsafe {
        let before = unk_804D6480;
        unk_804D6480 = unk_804D6480.wrapping_add(1);
        if unk_804D6480 == 0 {
            unk_804D6480 = 1;
        }
        before
    }
}
