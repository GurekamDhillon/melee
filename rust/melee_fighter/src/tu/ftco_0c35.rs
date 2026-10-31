//! src/melee/ft/kinds/ftCommon/ftCo_0C35.c: the turn-around-on-hit knockback (inlineA0,
//! 800C3598 / 800C36DC) and the TransN Y-spin that plays it out (800C37A0).
//! No fused sites (tools/rust-port/fma_sites.py). `M_PI` / `M_PI_2` are doubles in MSL math.h,
//! so the spin angle is computed in f64 and rounded once, as in the C.
//! Callees outside the ported set are `ext` stubs. Checked against the C by tests/tu_ftco_0c35.rs.
#![allow(non_snake_case)]

use crate::ftcommon2::ftcommon_800804fc;
use crate::hooks as h;
use crate::types::*;
use melee_runtime::gekko::{cosf, sinf};

pub type G = *mut Fighter_GObj;

pub mod ext {
    use crate::types::*;
    /// Override slots (tests); unset = panic until the owning TU is ported.
    pub static mut FIGHTER_UNK_RECURSIVE_FUNC_8006D044: Option<fn(*mut Fighter_GObj)> = None;
    pub static mut FTCO_DAMAGE_CALC_VEL: Option<fn(&mut Fighter, f32, f32)> = None;
    pub static mut FT_PART_SET_ROT_Y: Option<fn(&mut Fighter, i32, f32)> = None;
    pub static mut HSD_JOBJ_SET_MTX_DIRTY_SUB: Option<fn(*mut HSD_JObj)> = None;

    /// TODO(fighter.c) `Fighter_UnkRecursiveFunc_8006D044`.
    pub fn Fighter_UnkRecursiveFunc_8006D044(g: *mut Fighter_GObj) {
        match unsafe { FIGHTER_UNK_RECURSIVE_FUNC_8006D044 } { Some(f) => f(g), None => { unsafe { crate::tu::fighter::Fighter_UnkRecursiveFunc_8006D044(g as _); } return; } }
    }
    /// TODO(ftCo_Damage.c) `ftCo_Damage_CalcVel(fp, x, y)`.
    pub fn ftCo_Damage_CalcVel(fp: &mut Fighter, x: f32, y: f32) {
        match unsafe { FTCO_DAMAGE_CALC_VEL } { Some(f) => f(fp, x, y), None => { unsafe { crate::tu::ftCo_Damage::ftCo_Damage_CalcVel(fp as _, x as _, y as _); } return; } }
    }
    /// TODO(ftparts.c) `ftPartSetRotY(fp, part, rot_y)`.
    pub fn ftPartSetRotY(fp: &mut Fighter, part: i32, r: f32) {
        match unsafe { FT_PART_SET_ROT_Y } { Some(f) => f(fp, part, r), None => { unsafe { crate::tu::ftparts::ftPartSetRotY(fp as _, part as _, r as _); } return; } }
    }
    /// TODO(group1 jobj.c) `HSD_JObjSetMtxDirtySub` (raw `HSD_JObj*` form).
    pub fn HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) {
        match unsafe { HSD_JOBJ_SET_MTX_DIRTY_SUB } { Some(f) => f(j), None => panic!("TODO: HSD_JObjSetMtxDirtySub not ported") }
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `HSD_JObjMtxIsDirty` (jobj.h inline).
pub fn hsd_jobj_mtx_is_dirty(j: *mut HSD_JObj) -> bool {
    melee_core::ct!("hsd_jobj_mtx_is_dirty");
    let fl = unsafe { (*j).flags };
    fl & JOBJ_USER_DEF_MTX as u32 == 0 && fl & JOBJ_MTX_DIRTY as u32 != 0
}

/// The `HSD_JObjSetMtxDirty(jobj)` macro (jobj.h).
pub fn hsd_jobj_set_mtx_dirty(j: *mut HSD_JObj) {
    melee_core::ct!("hsd_jobj_set_mtx_dirty");
    if !j.is_null() && !hsd_jobj_mtx_is_dirty(j) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

/// `HSD_JObjSetRotationY` (jobj.h inline; the asserts are debug-only).
pub fn hsd_jobj_set_rotation_y(j: *mut HSD_JObj, y: f32) {
    melee_core::ct!("hsd_jobj_set_rotation_y");
    unsafe {
        (*j).rotate.y = y;
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            hsd_jobj_set_mtx_dirty(j);
        }
    }
}

/// `inlineA0` (static).
fn inline_a0(g: G) {
    let f = fp(g);
    f.dmg.x18F4 = h::common_data().x648;
    if f.dmg.x18F4 as f32 > f.dmg.x1954 {
        f.dmg.x1954 = f.dmg.x18F4 as f32;
    }
    f.set_x2220_b4(1);
    ext::Fighter_UnkRecursiveFunc_8006D044(g);
}

/// `ftCo_800C3598`.
pub fn ftco_800c3598(g: G) {
    melee_core::ct!("ftco_800c3598");
    let f = fp(g);
    inline_a0(g);
    let cd = h::common_data();
    let mag = if f.ground_or_air == GA_Air { cd.x64C } else { cd.x650 };
    let kb = f.dmg.kb_applied;
    let angle = h::ftco_damage_calc_angle(f, kb);
    let x = mag * cosf(angle);
    let y = mag * sinf(angle);
    if f.ground_or_air == GA_Air {
        let vx = x * f.dmg.facing_dir_1;
        ext::ftCo_Damage_CalcVel(f, vx, y);
        f.xF0_ground_kb_vel = 0.0;
    } else {
        f.xF0_ground_kb_vel = x * f.dmg.facing_dir_1;
        let (a, b) = (f.coll_data.floor.normal.y * f.xF0_ground_kb_vel, -f.coll_data.floor.normal.x * f.xF0_ground_kb_vel);
        ext::ftCo_Damage_CalcVel(f, a, b);
    }
    ftcommon_800804fc(f);
}

/// `ftCo_800C36DC`.
pub fn ftco_800c36dc(g: G) {
    melee_core::ct!("ftco_800c36dc");
    let f = fp(g);
    inline_a0(g);
    f.xF0_ground_kb_vel = h::common_data().x654 * -f.specialn_facing_dir;
    let (a, b) = (f.coll_data.floor.normal.y * f.xF0_ground_kb_vel, -f.coll_data.floor.normal.x * f.xF0_ground_kb_vel);
    ext::ftCo_Damage_CalcVel(f, a, b);
}

/// `ftCo_800C37A0`.
pub fn ftco_800c37a0(g: G) {
    melee_core::ct!("ftco_800c37a0");
    let f = fp(g);
    if f.dmg.x18F4 == 0 {
        return;
    }
    let part = h::ftparts_get_bone_index(f, FtPart_TransN as i32);
    let jobj = unsafe { (*f.parts.offset(part as isize)).x4_jobj2 };
    let x648 = h::common_data().x648;
    let rot_y = ((core::f64::consts::PI * x648.wrapping_sub(f.dmg.x18F4) as f32 as f64) / x648 as f32 as f64) as f32;
    f.dmg.x18F4 = f.dmg.x18F4.wrapping_sub(1);
    ext::ftPartSetRotY(f, part, rot_y);
    hsd_jobj_set_rotation_y(jobj, rot_y);
    if f.dmg.x18F4 == 0 {
        f.set_x2220_b4(0);
        f.facing_dir = -f.facing_dir;
        let r = (core::f64::consts::PI / 2.0 * f.facing_dir as f64) as f32;
        ext::ftPartSetRotY(f, FtPart_TopN as i32, r);
        ext::ftPartSetRotY(f, part, 0.0);
        hsd_jobj_set_rotation_y(jobj, 0.0);
        if let Some(cb) = f.x21F8 {
            unsafe { cb(g) };
        }
    }
}
