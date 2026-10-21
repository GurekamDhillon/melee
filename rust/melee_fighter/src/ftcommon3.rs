//! Hand port, round 8: more of src/melee/ft/ftcommon.c: landing (8007D6A4 with the TARGET_PC
//! missing-floor guard, 8007D7FC, 8007D92C), death/effects (8007DB24, 8007DB58), player nudge
//! (8007DD7C, 8007DFD0, 8007E0E4), the bone-4 translation follow (8007E358, 8007E3EC),
//! accessory (SetAccessory, 8007E690), the per-kind item / knockback callbacks (8007E6DC,
//! 8007E79C, 8007E7E4, 8007E82C, 8007F578, 8007F5CC, 8007F824, 8007F86C, 8007F8E8), parasol
//! (8007E83C, ftGetParasolStatus), 8007EA90, rumble on every fighter (8007EC30), sword
//! (8007EEC8, 8007EF5C), model scale (8007F6A4) and the head / held items (8007F948 + its
//! inline, 8007F9B4, 8007FA00, 8007FA58, 8007FC7C, 8007FE84).
//!
//! TARGET_PC paths are the ones ported (m-ex dispatchers replace the ftData_* table calls; the
//! floor guard replaces the `HSD_ASSERT` of 8007D6A4; the parasol table-3 bounds check).
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ftcommon.c`): 8007DD7C and
//! 8007DFD0 two f32 fmuladd each (`a.x * dir + v.x`), 8007E3EC three (`x1A7C += d * x1A6C`);
//! all -> `fmadds`. The rest have none.
//! Calls out of the ported set go through `crate::hooks`. Checked against the C (cut
//! verbatim, shipping contraction) by tests/ftcommon3_ref.rs.

use crate::ftcommon::{abs_c, clamp_ground_vel};
use crate::ftcommon2 as c2;
use crate::hooks as h;
use crate::types::*;
use core::sync::atomic::{AtomicU32, Ordering};
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn ids(f: &Fighter) -> (i32, i32) {
    (f.player_id as i32, f.is_sub_fighter() as i32)
}
#[inline(always)]
fn x594_b0(f: &Fighter) -> u8 {
    unsafe { f.anon_594.anon_0.x594_b0() }
}

/// `ftCommon_PcFloorMisses[6]` (static, TARGET_PC): never reset, as in the C.
static PC_FLOOR_MISSES: [AtomicU32; 6] = [const { AtomicU32::new(0) }; 6];

/// `ftCommon_PcFloorGuard(fp, valid, fall)`: false when the floor is valid; otherwise report
/// (every 120th miss per player), call `fall` and return true.
fn pc_floor_guard(f: &mut Fighter, valid: bool, fall: fn(G)) -> bool {
    if valid {
        return false;
    }
    let mut n = 0u32;
    if f.player_id < 6 {
        n = PC_FLOOR_MISSES[f.player_id as usize].fetch_add(1, Ordering::Relaxed);
    }
    if n % 120 == 0 {
        h::pc_floor_lost_report(f);
    }
    fall(f.gobj);
    true
}

/// `ftCommon_PcFloorValid`: the map exists, the floor line is in range and ft_80084A18 holds.
fn pc_floor_valid(f: &Fighter) -> bool {
    let map = h::mplib_8004d164_line_count();
    let line = f.coll_data.floor.index;
    match map {
        Some(count) => line >= 0 && line < count && h::ft_80084a18(f.gobj),
        None => false,
    }
}

/// `ftCommon_8007D6A4`: land (TARGET_PC: fall instead when the floor is gone).
pub fn ftcommon_8007d6a4(f: &mut Fighter) {
    let was_air = f.ground_or_air == GA_Air;
    let valid = pc_floor_valid(f);
    if pc_floor_guard(f, valid, h::ftco_fall_enter) {
        return;
    }
    if x594_b0(f) != 0 {
        f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
    }
    let max = h::script_value(f, 2, f.co_attrs.ground_max_horizontal_velocity);
    clamp_ground_vel(f, max);
    if f.ground_or_air == GA_Air {
        h::script_game_event(4, f.player_id as i32, f.motion_id as i32, f.is_sub_fighter() as i32, 0);
    }
    f.ground_or_air = GA_Ground;
    f.gr_vel = f.self_vel.x;
    f.x1968_jumpsUsed = 0;
    f.x1969_walljumpUsed = 0;
    f.set_x2227_b0(0);
    c2::unlock_ecb(f);
    if was_air {
        h::geno_ground_edge(f, 1);
    }
}

/// `ftCommon_8007D7FC`: 8007D780's air checks, then land.
pub fn ftcommon_8007d7fc(f: &mut Fighter) {
    if f.ground_or_air == GA_Air {
        if h::un_803224dc(f.x8_spawnNum, f.cur_pos.x, f.dmg.x18A4_knockbackMagnitude) {
            f.dmg.x18A4_knockbackMagnitude = 0.0;
        }
        if f.x2227_b0() != 0 && f.x1968_jumpsUsed <= 1 {
            let (s, u) = ids(f);
            h::pl_8003fc44(s, u);
        }
    }
    ftcommon_8007d6a4(f);
}

pub fn ftcommon_8007d92c(g: G) {
    if fp(g).ground_or_air == GA_Air {
        h::ftco_fall_enter(g);
    } else {
        h::ft_8008a2bc(g);
    }
}

pub fn ftcommon_8007db24(g: G) {
    fp(g).set_x2219_b0(0);
    h::eflib_destroy_all(g);
}

pub fn ftcommon_8007db58(g: G) {
    let f = fp(g);
    crate::ft0881::ft_80088770(f);
    crate::ft0881::ft_800887cc(f);
    if let Some(cb) = f.take_dmg_cb {
        unsafe { cb(g) };
    }
    if let Some(cb) = fp(g).death1_cb {
        unsafe { cb(g) };
    }
}

/// `(a.x * a_dir + v.x) - (b_dir * b.x + p.x)`, both products fused in the shipping build.
#[inline(always)]
fn nudge_dx(a: &Vec2, a_dir: f32, v: &Vec3, b_dir: f32, b: &Vec2, p: &Vec3) -> f32 {
    fmadds(a.x, a_dir, v.x) - fmadds(b_dir, b.x, p.x)
}
#[inline(always)]
fn sgn_c(neg: bool, x: f32) -> f32 {
    if neg { -x } else { x }
}
#[inline(always)]
fn same_or_adjacent_floor(a: i32, b: i32) -> bool {
    a == b || b == h::mp_line_get_next(a) || b == h::mp_line_get_prev(a)
}

/// `ftCommon_8007DD7C`: push apart from grounded fighters on the same / adjacent floor line.
pub fn ftcommon_8007dd7c(g: G, v: &Vec3) {
    let arg = unsafe { (*g).user_data as *mut Fighter };
    let mut phi_r28 = false;
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if cur != g && !h::ftlib_80086fd4(cur, g) {
            let c = unsafe { &*((*cur).user_data as *const Fighter) };
            if !(c.x221F_b3() != 0 || c.ground_or_air != GA_Ground || !c.victim_gobj.is_null() || c.is_sub_fighter() != 0) {
                let (a2c4, a_dir, arg_gnd) = unsafe { ((*arg).x2C4, (*arg).facing_dir, (*arg).coll_data.floor.index) };
                let cur_gnd = c.coll_data.floor.index;
                if same_or_adjacent_floor(arg_gnd, cur_gnd) {
                    let sp24 = c2::ftcommon_8007f8b4(c);
                    let vt = c.x2C4;
                    let f0 = nudge_dx(&a2c4, a_dir, v, c.facing_dir, &vt, &sp24);
                    if abs_c(f0) < a2c4.y + vt.y {
                        let cd = h::common_data();
                        let a = unsafe { &mut *arg };
                        if f0 != 0.0 {
                            a.xF8_playerNudgeVel.x += sgn_c(f0 < 0.0, cd.x450);
                        } else {
                            a.xF8_playerNudgeVel.x += sgn_c(phi_r28, cd.x450);
                        }
                        if v.z - sp24.z != 0.0 {
                            a.xF8_playerNudgeVel.y += sgn_c(v.z - sp24.z < 0.0, cd.x454);
                        } else if f0 != 0.0 {
                            a.xF8_playerNudgeVel.y += sgn_c(f0 < 0.0, cd.x454);
                        } else {
                            a.xF8_playerNudgeVel.y += sgn_c(phi_r28, cd.x454);
                        }
                    }
                }
            }
        } else {
            phi_r28 = true;
        }
        cur = unsafe { (*cur).next };
    }
}

/// `ftCommon_8007DFD0`: the sub fighter (Nana) moves off its main fighter, then 8007DD7C.
pub fn ftcommon_8007dfd0(g: G, arg1: &Vec3) {
    let f = unsafe { (*g).user_data as *mut Fighter };
    let (pid, f2c4, f_dir, f_gnd) = unsafe { ((*f).player_id, (*f).x2C4, (*f).facing_dir, (*f).coll_data.floor.index) };
    let ng = h::player_get_entity(pid as i32);
    let t = unsafe { &*((*ng).user_data as *const Fighter) };
    if t.x221F_b3() == 0 && t.ground_or_air == GA_Ground {
        let t_gnd = t.coll_data.floor.index;
        if same_or_adjacent_floor(f_gnd, t_gnd) {
            let t = unsafe { &*((*ng).user_data as *const Fighter) };
            let sp1c = c2::ftcommon_8007f8b4(t);
            let tv = t.x2C4;
            let f1 = nudge_dx(&f2c4, f_dir, arg1, t.facing_dir, &tv, &sp1c);
            if abs_c(f1) < f2c4.y + tv.y {
                let d = h::common_data().x45C;
                unsafe { (*f).xF8_playerNudgeVel.y -= d };
            }
        }
    }
    ftcommon_8007dd7c(g, arg1);
}

/// `ftCommon_8007E0E4`: compute this frame's player nudge velocity.
pub fn ftcommon_8007e0e4(g: G) {
    let f = fp(g);
    f.xF8_playerNudgeVel.y = 0.0;
    f.xF8_playerNudgeVel.x = 0.0;
    if f.x2219_b1() == 0 && f.x2219_b5() == 0 && f.ground_or_air == GA_Ground {
        let cd = h::common_data();
        let (phi_f30, phi_f31) = if f.is_sub_fighter() == 0 { (cd.x454, cd.x458) } else { (cd.x45C, cd.x460) };
        let mut sp10 = c2::ftcommon_8007f8b4(f);
        if f.x221D_b5() == 0 {
            if f.is_sub_fighter() == 0 {
                ftcommon_8007dd7c(g, &sp10);
            } else {
                ftcommon_8007dfd0(g, &sp10);
            }
        }
        let f = fp(g);
        if f.xF8_playerNudgeVel.y == 0.0 && sp10.z != 0.0 {
            f.xF8_playerNudgeVel.y = if sp10.z < 0.0 { phi_f30 } else { -phi_f30 };
        }
        let ny = f.xF8_playerNudgeVel.y;
        if (ny > 0.0 && sp10.z < 0.0 && sp10.z + ny >= 0.0) || (ny < 0.0 && sp10.z > 0.0 && sp10.z + ny <= 0.0) {
            f.xF8_playerNudgeVel.y = -sp10.z;
            sp10.z = 0.0;
        }
        if sp10.z + f.xF8_playerNudgeVel.y > phi_f31 {
            f.xF8_playerNudgeVel.y = phi_f31 - sp10.z;
        } else if sp10.z + f.xF8_playerNudgeVel.y < -phi_f31 {
            f.xF8_playerNudgeVel.y = -phi_f31 - sp10.z;
        }
        if f.is_sub_fighter() == 1 {
            f.xF8_playerNudgeVel.x = 0.0;
        }
    }
}

#[inline(always)]
fn bone4(f: &Fighter) -> *mut HSD_JObj {
    let i = h::ftparts_get_bone_index(f, 4);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// `ftCommon_8007E358`: remember bone 4's translation in x1A7C.
pub fn ftcommon_8007e358(g: G) {
    let f = fp(g);
    let j = bone4(f);
    h::hsd_jobj_get_translation(j, &mut f.x1A7C);
}

/// `ftCommon_8007E3EC`: move x1A7C toward bone 4's translation by x1A6C (fused) and set it back.
pub fn ftcommon_8007e3ec(g: G) {
    let f = fp(g);
    let j = bone4(f);
    if h::hsd_jobj_mtx_is_dirty(j) {
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        h::hsd_jobj_get_translation(j, &mut sp10);
        sp10.x -= f.x1A7C.x;
        sp10.y -= f.x1A7C.y;
        sp10.z -= f.x1A7C.z;
        f.x1A7C.x = fmadds(sp10.x, f.x1A6C, f.x1A7C.x);
        f.x1A7C.y = fmadds(sp10.y, f.x1A6C, f.x1A7C.y);
        f.x1A7C.z = fmadds(sp10.z, f.x1A6C, f.x1A7C.z);
        h::hsd_jobj_set_translate(j, &f.x1A7C);
    }
}

/// `ftCommon_SetAccessory`: panics (HSD_ASSERT, ftcommon.c:0x486) if one already exists.
pub fn set_accessory(f: &mut Fighter, joint: *mut HSD_Joint) {
    assert!(f.x20A0_accessory.is_null(), "fighter accessory already exsist! (ftcommon.c:1158)");
    f.x20A0_accessory = h::hsd_jobj_load_joint(joint);
}

pub fn ftcommon_8007e690(f: &mut Fighter, arg1: *mut HSD_AnimJoint) {
    if !f.x20A0_accessory.is_null() {
        h::hsd_jobj_add_anim_all(f.x20A0_accessory, arg1);
        h::hsd_jobj_req_anim_all(f.x20A0_accessory, 0.0);
    }
}

/// `ftCommon_8007E6DC(gobj, item, arg2)`: drop the held item (TARGET_PC m-ex dispatch).
pub fn ftcommon_8007e6dc(g: G, item: G, arg2: i32) {
    let f = fp(g);
    if !f.x1978.is_null() && f.x1978 == item {
        f.x1978 = core::ptr::null_mut();
        return;
    }
    if h::ftco_800c5240(g) {
        h::ftco_800c5500(g);
    }
    let f = fp(g);
    h::mex_on_item_drop_ext(f.kind as i32, g, arg2);
    let f = fp(g);
    let (s, u) = ids(f);
    h::pl_8003ea08(s, u);
    fp(g).item_gobj = core::ptr::null_mut();
}

pub fn ftcommon_8007e79c(g: G, arg1: i32) {
    h::mex_on_item_drop(fp(g).kind as i32, g, arg1);
}

pub fn ftcommon_8007e7e4(g: G, arg1: i32) {
    h::mex_on_item_pickup2(fp(g).kind as i32, g, arg1);
}

pub fn ftcommon_8007e82c(g: G) {
    fp(g).x1984_heldItemSpec = core::ptr::null_mut();
}

const PARASOL_TABLE_2: [i32; 7] = [7, 8, 9, 10, 5, 6, 4];
const PARASOL_TABLE_3_SET: [bool; 7] = [false, false, false, false, true, false, true];
const PARASOL_TABLE_4: [i32; 7] = [-1, -1, -1, -1, 1, -1, 2];
const FT_PARASOL_NONE: i32 = -1;

/// `ftCommon_8007E83C(gobj, state, div)`: drive the held parasol's animation speed.
pub fn ftcommon_8007e83c(g: G, arg1: i32, div: f32) {
    assert!(ft_get_parasol_status(g) != FT_PARASOL_NONE, "ftcommon.c:1276");
    let f = fp(g);
    let val = if div == 0.0 {
        f.frame_speed_mul
    } else if h::it_get_kind(f.item_gobj) == It_Kind_Parasol as i32 {
        let r = h::it_8028b08c(f.item_gobj, PARASOL_TABLE_2[arg1 as usize]);
        fp(g).frame_speed_mul * (r as f32 / div)
    } else {
        let r = h::it_802bda40(f.item_gobj, PARASOL_TABLE_4[arg1 as usize]);
        fp(g).frame_speed_mul * (r as f32 / div)
    };
    let item = fp(g).item_gobj;
    if h::it_get_kind(item) == It_Kind_Parasol as i32 {
        assert!((0..7).contains(&arg1), "parasol_table_1[{arg1}] out of range");
        h::it_parasol_table_1(arg1, item, val);
    } else if arg1 >= 0 && arg1 < 7 && PARASOL_TABLE_3_SET[arg1 as usize] {
        h::it_parasol_table_3(arg1, item, val);
    }
}

/// `ftGetParasolStatus`.
pub fn ft_get_parasol_status(g: G) -> i32 {
    let item = fp(g).item_gobj;
    if !item.is_null() && h::it_get_kind(item) == It_Kind_Parasol as i32 {
        match h::it_get_motion_id(item) {
            7 => return 0,
            8 => return 1,
            9 => return 2,
            10 => return 3,
            5 => return 4,
            6 => return 5,
            4 => return 6,
            _ => {}
        }
    }
    let item = fp(g).item_gobj;
    if !item.is_null() && h::it_get_kind(item) == It_Kind_Peach_Parasol as i32 {
        match h::it_get_motion_id(item) {
            1 => return 4,
            0 | 2 => return 6,
            _ => {}
        }
    }
    FT_PARASOL_NONE
}

/// `ftCommon_8007EA90(fp, arg1)`: spawn hit particles (it_802E5F8C) and credit the attacker.
pub fn ftcommon_8007ea90(f: &mut Fighter, arg1: i32) {
    if !h::gm_8016b0b4() {
        return;
    }
    let (phi_f31, phi_f30, sp10);
    if f.dmg.kb_applied != 0.0 {
        let kb = f.dmg.kb_applied;
        let mut a = h::ftco_damage_calc_angle(f, kb);
        if f.dmg.facing_dir_1 > 0.0 {
            const PI: f64 = core::f64::consts::PI;
            a = if a as f64 > PI { (PI * 3.0 - a as f64) as f32 } else { (PI - a as f64) as f32 };
        }
        phi_f31 = a;
        phi_f30 = kb;
        sp10 = Vec3 { x: f.dmg.x1854_collpos.x, y: f.dmg.x1854_collpos.y, z: 0.0 };
    } else {
        phi_f31 = (core::f64::consts::PI / 2.0) as f32;
        phi_f30 = h::common_data().x5C0;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        h::ftlib_800866dc(f.gobj, &mut v);
        sp10 = v;
    }
    let r = h::it_802e5f8c(f.gobj, &sp10, arg1, 1, phi_f31, phi_f30);
    let src = f.dmg.x18c4_source_ply;
    if src != 6 {
        h::player_set_unk98(src, r.wrapping_add(h::player_get_unk98(src)));
    }
}

/// `ftCommon_8007EC30`: 8007EBAC on every fighter.
pub fn ftcommon_8007ec30(arg0: u32, arg1: u32) {
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        c2::ftcommon_8007ebac(fp(cur), arg0, arg1);
        cur = unsafe { (*cur).next };
    }
}

#[inline(always)]
fn holds_sword(f: &Fighter) -> bool {
    !f.item_gobj.is_null()
        && unsafe { (*f.item_gobj).classifier } as i32 == HSD_GOBJ_CLASS_ITEM
        && h::it_get_kind(f.item_gobj) == It_Kind_Sword as i32
}

pub fn ftcommon_8007eec8(f: &mut Fighter, arg1: i32, arg2: i32) {
    if holds_sword(f) {
        let multiplier = (1.0f64 / 256.0) as f32;
        h::it_80284fc4(f.item_gobj, arg1, arg2 as f32 * multiplier);
    }
}

pub fn ftcommon_8007ef5c(f: &mut Fighter, arg1: i32) {
    if holds_sword(f) {
        h::it_80285024(f.item_gobj, arg1);
    }
}

pub fn ftcommon_8007f578(g: G) {
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    h::mex_on_item_invisible(f.kind as i32, g);
}

/// `ftCommon_8007F5CC(gobj, visible)`; `bool` is int, stored into a `:1`.
pub fn ftcommon_8007f5cc(g: G, arg1: i32) {
    let f = fp(g);
    let item = f.item_gobj;
    if !item.is_null() && f.x221E_b3() as i32 != arg1 {
        if arg1 == 0 {
            ftcommon_8007f578(g);
        } else {
            h::mex_on_item_visible(f.kind as i32, g);
        }
    }
    fp(g).set_x221E_b3((arg1 & 1) as u8);
}

/// `ftCommon_8007F6A4(fp, jobj)`: scale `jobj` by 1 / model_scaling.
pub fn ftcommon_8007f6a4(f: &mut Fighter, jobj: *mut HSD_JObj) {
    let val = 1.0f32 / unsafe { (*(*f.ft_data).x0).model_scaling };
    let scale = Vec3 { x: val, y: val, z: val };
    h::hsd_jobj_set_scale(jobj, &scale);
}

pub fn ftcommon_8007f824(g: G) {
    h::mex_on_knockback_enter(fp(g).kind as i32, g);
}

pub fn ftcommon_8007f86c(g: G) {
    h::mex_on_knockback_exit(fp(g).kind as i32, g);
}

pub fn ftcommon_8007f8e8(g: G) {
    let f = fp(g);
    if f.x197C.is_null() || f.x1980.is_null() {
        h::mex_on_remove_head_item(f.kind as i32, g);
    }
}

/// `_func_8007F948_inline`.
fn apply_head_item_cb(g: G) {
    let f = fp(g);
    if f.x197C.is_null() || f.x1980.is_null() {
        h::mex_on_apply_head_item(f.kind as i32, g);
    }
}

pub fn ftcommon_8007f948(g: G, arg1: G, arg2: i32) {
    let f = fp(g);
    f.x197C = arg1;
    f.x2014 = arg2;
    apply_head_item_cb(g);
}

pub fn ftcommon_8007f9b4(g: G) {
    fp(g).x197C = core::ptr::null_mut();
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
}

/// `ftCommon_8007FA00`: place / scale the head item from `co_attrs.x114 / x120 / x12C`.
pub fn ftcommon_8007fa00(g: G) {
    let f = fp(g);
    let (a, b) = (f.co_attrs.x120, f.co_attrs.x114);
    h::it_80294eb0(f.x197C, &a, &b);
    let f = fp(g);
    h::it_80294e78(f.x197C, f.x34_scale.y * f.co_attrs.x12C);
}

/// `ftCommon_8007FA58(gobj, item)`: put `item` on the head (or just refresh the timer).
pub fn ftcommon_8007fa58(g: G, arg1: G) {
    if !fp(g).x197C.is_null() {
        fp(g).x2014 = h::it_8026b54c(arg1) as i32; // float -> s32 (saturating)
        h::play_sfx(fp(g), 0x117, 0x7F, 0x40);
        c2::ftcommon_8007ebac(fp(g), 0x10, 0);
        h::item_8026a8ec(arg1);
        return;
    }
    let vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 }; // ftCo_803B74A0
    h::it_8026bd0c(arg1);
    h::item_8026abd8(arg1, &vec, 0.0);
    let part = unsafe { (*(*fp(g).ft_data).x8).x12 } as i32;
    h::item_8026ab54(arg1, g, part);
    let t = h::it_8026b54c(arg1) as i32;
    ftcommon_8007f948(g, arg1, t);
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
    h::play_sfx(fp(g), 0x117, 0x7F, 0x40);
    c2::ftcommon_8007ebac(fp(g), 0x10, 0);
    h::it_8026bcf4(arg1);
    ftcommon_8007fa00(g);
}

/// `ftCommon_8007FC7C(gobj, arg8)`: grow the held (x1980) item, or spawn one.
/// Float -> int conversions saturate (gwtool lowering), int adds wrap (`-fwrapv`).
pub fn ftcommon_8007fc7c(g: G, arg8: f32) {
    let mut sp20 = (arg8 * h::common_data().x704) as i32;
    if !fp(g).x1980.is_null() {
        let f = fp(g);
        f.x2018 = f.x2018.wrapping_add(sp20);
        if f.x2018 > h::common_data().x700 {
            f.x2018 = h::common_data().x700;
        }
        sp20 = f.x2024;
        f.x2024 = (sp20 as f32 + arg8) as i32;
        c2::ftcommon_8007fda0(f);
    } else {
        let f = fp(g);
        let part = unsafe { (*(*f.ft_data).x8).x12 } as i32;
        let pos = f.cur_pos;
        let item = h::it_8029a748(g, &pos, part, f.facing_dir);
        if !item.is_null() {
            ftcommon_8007fe84(g, item, sp20, arg8);
            c2::ftcommon_8007fda0(fp(g));
        }
    }
    h::play_sfx(fp(g), 0x11F, 0x7F, 0x40);
}

/// `ftCommon_8007FE84(gobj, item, arg2, arg3)`: hold `item` in x1980.
pub fn ftcommon_8007fe84(g: G, item: G, arg2: i32, arg3: f32) {
    let f = fp(g);
    f.x1980 = item;
    f.x2018 = arg2;
    f.x201C = h::common_data().x6F8;
    f.x2024 = arg3 as i32;
    f.x2021 = 0;
    f.x2020 = 0;
    c2::ftcommon_8007ebac(f, 2, 0);
    apply_head_item_cb(g);
}

// ---- port round 9: the ftcommon.c atan2f stick angles (group 1's MSL atan2f) -------------

/// `ftCo_GetLStickAngle`: `atan2f(lstick.y, ABS(lstick.x))` (ABS keeps -0.0).
pub fn ftco_get_lstick_angle(fp: &Fighter) -> f32 {
    melee_runtime::lbtrigf::atan2f(fp.input.lstick[0].y, abs_c(fp.input.lstick[0].x))
}
/// `ftCo_GetCStickAngle`.
pub fn ftco_get_cstick_angle(fp: &Fighter) -> f32 {
    melee_runtime::lbtrigf::atan2f(fp.input.cstick[0].y, abs_c(fp.input.cstick[0].x))
}
/// `ftCommon_8007D9D4`: `atan2f(lstick.y, lstick.x)`.
pub fn ftcommon_8007d9d4(fp: &Fighter) -> f32 {
    melee_runtime::lbtrigf::atan2f(fp.input.lstick[0].y, fp.input.lstick[0].x)
}
