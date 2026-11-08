//! Port of src/melee/sfx/crowdsfx_1.c, split from crowdsfx.c upstream (doldecomp 598987cf3): un_80322258, un_80322298, un_803222EC, un_80322314, un_8032233C, un_80322598.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

use crate::types::*;
use super::crowdsfx::*;
use super::crowdsfx::ext;

pub unsafe fn un_80322258(arg: f32) -> bool {
    melee_core::ct!("un_80322258");
    let val2c = cfg().horiz_margin;
    let bx = ext::mpLib_80458868();

    arg < val2c + (*bx.add(1)).left || arg > (*bx.add(1)).right - val2c
}

pub unsafe fn un_80322298(arg: f32) -> i32 {
    melee_core::ct!("un_80322298");
    let v = cfg();
    if arg >= v.kb_threshold_high {
        return 3;
    }
    if arg >= v.kb_threshold_mid {
        return 2;
    }
    if arg >= v.kb_threshold_low {
        return 1;
    }
    0
}

pub unsafe fn un_803222EC(arg1: f32, arg2: f32) -> f32 {
    melee_core::ct!("un_803222EC");
    let v = cfg();
    if !(arg2 > v.angle_min) {
        return arg1;
    }
    if !(arg2 < v.angle_max) {
        return arg1;
    }
    arg1 * v.angle_mult
}

pub unsafe fn un_80322314() {
    melee_core::ct!("un_80322314");
    let d = data();
    let v = cfg();
    if d.x18 >= v.max_gasp_count {
        return;
    }
    d.x1C = 1;
    d.x20 = 1;
}

pub unsafe fn un_8032233C(arg0: u32, arg1: u32) {
    melee_core::ct!("un_8032233C");
    let d = data();

    let gobj = ext::ftLib_FindBySpawnNum(arg0);
    let kb_mag = ext::ftLib_GetCrowdKnockback(ext::ftLib_FindBySpawnNum(arg1));

    let cat = if kb_mag >= cfg().kb_threshold_high {
        3
    } else if kb_mag >= cfg().kb_threshold_mid {
        2
    } else if kb_mag >= cfg().kb_threshold_low {
        1
    } else {
        0
    };

    if cat == 0 {
        return;
    }

    if !gobj.is_null() && ext::ftLib_GetCrowdKnockback(gobj) >= 3.0 {
        un_80321D30(arg0, kb_mag);
    } else if d.x0 as u32 == arg0 && (d.x4 as f32) < cfg().x18 {
        un_80321D30(arg0, if kb_mag > d.x8 { kb_mag } else { d.x8 });
    } else {
        match cat {
            3 => un_80321CA4(0x144),
            2 => un_80321CA4(0x145),
            1 => un_80321CA4(0x146),
            _ => {}
        }

        if cat == 3 || (cat == 2 && d.xC == arg1) {
            un_80321C70();
        }
    }
    d.x4 = 0;
    d.x0 = arg0 as i32;
    d.x8 = kb_mag;
}

pub unsafe fn un_80322598(arg0: i32, arg1: f32) -> i32 {
    melee_core::ct!("un_80322598");
    let val14 = (*ext::mpLib_80458868().add(1)).bottom;
    if arg1 >= val14 || arg1 < cfg().recovery_y_low + val14 {
        return 0;
    }
    let v = cfg();
    let cat = if arg1 > v.recovery_y_high + val14 {
        3
    } else if arg1 > v.recovery_y_mid + val14 {
        2
    } else {
        1
    };
    // No `return` in the C after this call: r3 is un_8032201C's result.
    un_8032201C(arg0 as u32, cat) as i32
}
