//! The capsule-metadata table of the LAB's hit rules (`pc/gameworld/script_hit_rules.inc`): the
//! part that the fighter/character code calls from below `melee_modes`
//! (`ScriptGame_HitRuleDamage` from `ftColl_8007ABD0`, `ScriptGame_HitRuleSpecial` from the Purin
//! rollout). It was in `melee_modes::tu::script_game` (still re-exported from there); the table
//! and its accessors live here so every crate shares the one table.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::{HRMeta, HitCapsule};

pub const HR_META_MAX: i32 = 1024;

pub static mut script_hit_meta: [HRMeta; 1024] = unsafe { core::mem::zeroed() };
pub static mut script_hit_meta_count: i32 = 0;

/// `hr_ordinary(element)`: the element ids the rules apply to.
pub fn hr_ordinary(e: i32) -> i32 {
    melee_core::ct!("hr_ordinary");
    (e == 0 || e == 1 || e == 2 || e == 5 || e == 13) as i32
}

/// `script_hit_metadata(hit, create)`: the capsule's row (open addressing by pointer), or a fresh
/// one when `create`.
pub unsafe extern "C" fn script_hit_metadata(h: *mut HitCapsule, create: i32) -> *mut HRMeta {
    melee_core::ct!("script_hit_metadata");
    // The C hashes the 32-bit address; the cast keeps the low 32 bits.
    let start: u32 = ((h as usize as u32) >> 2) % (HR_META_MAX as u32);
    if create == 0 && script_hit_meta_count == 0 {
        return core::ptr::null_mut();
    }
    let base = (&raw mut script_hit_meta) as *mut HRMeta;
    for i in 0..HR_META_MAX {
        let m = base.add(((start.wrapping_add(i as u32)) % (HR_META_MAX as u32)) as usize);
        if (*m).hit == h {
            return m;
        }
    }
    if create != 0 {
        for i in 0..HR_META_MAX {
            let m = base.add(((start.wrapping_add(i as u32)) % (HR_META_MAX as u32)) as usize);
            if (*m).hit.is_null() {
                core::ptr::write_bytes(m, 0, 1);
                (*m).hit = h;
                script_hit_meta_count += 1;
                return m;
            }
        }
    }
    core::ptr::null_mut()
}

pub unsafe extern "C" fn ScriptGame_HitRuleForget(hit: *mut HitCapsule) {
    melee_core::ct!("ScriptGame_HitRuleForget");
    let m = script_hit_metadata(hit, 0);
    if !m.is_null() {
        core::ptr::write_bytes(m, 0, 1);
        script_hit_meta_count -= 1;
    }
}

pub unsafe extern "C" fn ScriptGame_HitRuleSpecial(hit: *mut HitCapsule) {
    melee_core::ct!("ScriptGame_HitRuleSpecial");
    let m = script_hit_metadata(hit, 0);
    if !m.is_null() {
        (*hit).damage = (*m).original_damage;
        (*hit).unk_count = (*m).original_count;
        (*hit).x24 = (*m).original_growth;
        (*hit).x2C = (*m).original_base;
        (*hit).x34 = (*m).original_shield;
        ScriptGame_HitRuleForget(hit);
    }
}

pub unsafe extern "C" fn ScriptGame_HitRuleDamage(hit: *mut HitCapsule) {
    melee_core::ct!("ScriptGame_HitRuleDamage");
    let m = script_hit_metadata(hit, 0);
    if m.is_null() {
        return;
    }
    (*m).original_damage = (*hit).damage;
    (*m).original_count = (*hit).unk_count;
    if hr_ordinary((*hit).element as i32) == 0 {
        ScriptGame_HitRuleForget(hit);
        return;
    }
    if (*m).damage_scale != 1.0 {
        (*hit).damage *= (*m).damage_scale;
        if (*hit).damage > 500.0 {
            (*hit).damage = 500.0;
        }
    }
    (*hit).unk_count = ((*hit).unk_count as f32 * (*m).damage_scale) as u32;
}
