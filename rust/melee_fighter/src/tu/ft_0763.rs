//! Port of src/melee/ft/ft_0763.c, split from ftcoll.c upstream (doldecomp 598987cf3): ftcoll_800763c0, ftcoll_80076444, ftcoll_8007646c, ftcoll_800764dc, ftcoll_80076528, ftcoll_800765ac.
#![allow(non_snake_case, static_mut_refs, non_upper_case_globals, clippy::missing_safety_doc)]

use crate::ftcoll::{DMG_LOG0_IDX, DMG_LOG1_IDX};
use crate::ftcommon::abs_c;
use crate::ftcommon2 as fc2;
use crate::ftcommon3::ftco_get_cstick_angle;
use crate::hooks as h;
use crate::ft0df::{G as G_ft0df, fp};
use crate::types::*;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use super::ftcoll::*;
use super::ftcoll::ext;

/// `ftColl_800763C0`: combo count logic.
pub fn ftcoll_800763c0(attacker: G, victim: G, attack_id: i32) {
    melee_core::ct!("ftcoll_800763c0");
    if attacker == victim {
        return;
    }
    unsafe {
        let fp = &mut *fpg(attacker);
        let cur = fp.x2094;
        if cur.is_null() {
            fp.x208C = attack_id as _;
            fp.x2090 = 1;
            fp.x2094 = victim;
        } else if cur == victim {
            if attack_id != 1 && fp.x208C as i32 == attack_id {
                fp.x2090 = fp.x2090.wrapping_add(1);
                if fp.x2090 as i32 >= cd().x4C4 as i32 {
                    fp.x2092 = cd().x4D8 as _;
                }
            } else {
                fp.x2090 = 0;
                fp.x208C = attack_id as _;
            }
        }
    }
}

/// `ftColl_80076444`.
pub fn ftcoll_80076444(attacker: G, victim: G) {
    melee_core::ct!("ftcoll_80076444");
    let id = unsafe { (*fpg(attacker)).x2068_attackID } as i32;
    ftcoll_800763c0(attacker, victim, id);
}

/// `ftColl_8007646C`: combo count from an item's owner.
pub fn ftcoll_8007646c(item: G, victim: G) {
    melee_core::ct!("ftcoll_8007646c");
    let owner = ext::itGetOwner(item);
    let msid = ext::itGetAttackId(item);
    if ext::ftLib_IsFighter(owner) {
        ftcoll_800763c0(owner, victim, msid);
    }
}

/// `ftColl_800764DC`: check to end the victim's combo.
pub fn ftcoll_800764dc(g: G) {
    melee_core::ct!("ftcoll_800764dc");
    unsafe {
        let fp = &mut *fpg(g);
        if fp.x2098 != 0 {
            fp.x2098 -= 1;
        }
        if !fp.x2094.is_null() {
            let fp1 = &*fpg(fp.x2094);
            if fp1.x221C_b6() == 0 && fp1.x2098 == 0 {
                fp.x2094 = core::ptr::null_mut();
            }
        }
    }
}

/// `ftColl_80076528`.
pub fn ftcoll_80076528(g: G) {
    melee_core::ct!("ftcoll_80076528");
    unsafe {
        let fp = &mut *fpg(g);
        let v = fp.x2092;
        if v != 0 {
            fp.x2092 = v.wrapping_sub(1);
            if fp.victim_gobj.is_null() && ga(fp.ground_or_air) == GROUND {
                combo_count_push(fp);
            }
        }
    }
}

/// `ftColl_800765AC`: clear `victim` from every fighter's combo target.
pub fn ftcoll_800765ac(victim: G) {
    melee_core::ct!("ftcoll_800765ac");
    let mut cur = crate::hooks::plink_head(PLINK_FIGHTER);
    while !cur.is_null() {
        unsafe {
            let fp = &mut *fpg(cur);
            if victim == fp.x2094 {
                fp.x2094 = core::ptr::null_mut();
            }
            cur = (*cur).next;
        }
    }
}
