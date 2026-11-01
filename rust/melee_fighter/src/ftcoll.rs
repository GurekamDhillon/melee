
//! Hand port, round 9: leaf functions of ft/ftcoll.c (damage-log index reset, environment
//! damage rounding, two inlines). Checked against the C (cut verbatim) by tests/ftleaf_ref.rs.
#![allow(static_mut_refs)]

use crate::types::*;

/// `dmg_log0_idx` / `dmg_log1_idx` (ftcoll.c statics).
pub static mut DMG_LOG0_IDX: i32 = 0;

pub static mut DMG_LOG1_IDX: i32 = 0;

/// `getEnvDmg` (static): nonzero damage is at least 1, else truncated. C `(int) f` is the
/// shipping lowering's saturating conversion (lower.py), which is Rust's `as`.
pub fn get_env_dmg(dmg: f32) -> i32 {
    if dmg != 0.0 {
        if dmg as i32 != 0 {
            return dmg as i32;
        }
        return 1;
    }
    0
}

/// `inlineB1` (static inline): `hit0->coll_distance < p_ftCommonData->x7A8`.
pub fn inline_b1(hit0: &HitCapsule) -> bool {
    hit0.coll_distance < crate::hooks::common_data().x7A8
}

/// `getUnkVal` (static inline).
pub fn get_unk_val(f: &Fighter, i: i32) -> i32 {
    i.wrapping_add((f.player_id as i32) * 2).wrapping_add(f.is_sub_fighter() as i32)
}

pub use crate::tu::ftcoll::ftcoll_800765e0;
