//! Port of src/melee/pl/plbonuslib.c: the event hooks that feed the bonus counters in
//! `pl_StaleMoveTableExt_t` (hits taken / dealt, item use, movement distance, camera distance...).
//!
//! Fused multiply-adds (tools/rust-port/fma_sites.py): `pl_8003ED0C` (1), `fn_8003F294` (1: the
//! sqrt argument `x*x + y*y`), `fn_8003F654` (3), `pl_8003FAA8` (1); each is `fmadds(a, b, other)`
//! with `a*b` the left product and `other` the separately rounded right operand. `my_sqrtf` is
//! MSL `sqrtf` (`melee_runtime::gekko::sqrtf`). `pl_8003E70C` / `pl_8003E7D4` index
//! `x598[It_PKind_Start..]` past the member (into `x848[]`), so they use byte-offset pointers.
//! `pl_8003E4A4`'s `seen[-1]` store for an unknown item kind (stack garbage in C) is skipped.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::plbonus::{pl_800419ac, pl_80040ed4};#[allow(unused_imports)]
use crate::plbonus2::{I, ext as ext_plbonus2};
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::player::{get_entity_at_index, get_team, get_unk6a8_ptr, player_8003221c};
#[allow(unused_imports)]
use crate::stale::action_stats;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use crate::plbonus::{calculate_average, check_if_same_team, ext, k, none_has};
use core::ffi::c_void;
use core::ptr::null_mut;

use super::player::*;
use super::plbonus::{
    pl_80038788, pl_80038824, pl_80038898, pl_8003906C, pl_80039238, pl_80039418, pl_800386D8, pl_800386E8,
};
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? #[allow(unreachable_code)] { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) } }
                }
            } )*
        };
    }
    ext_fns! {
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec3) => unsafe { melee_world::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); };
    fn ft_800877F8(a0: *mut HSD_GObj, a1: i32) -> bool => unsafe { crate::ft0877::ft_800877f8(a0 as _, a1 as _) };
    fn ft_80087818(a0: *mut HSD_GObj, a1: i32) -> bool => unsafe { crate::ft0877::ft_80087818(a0 as _, a1 as _) };
    fn ft_80087858(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087858(a0 as _) };
    fn ft_80087878(a0: *mut HSD_GObj, a1: i32) -> bool => unsafe { crate::ft0877::ft_80087878(a0 as _, a1 as _) };
    fn ft_800878BC(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_800878bc(a0 as _) };
    fn ft_80087900(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087900(a0 as _) };
    fn ft_80087944(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087944(a0 as _) };
    fn ft_80087A8C(a0: *mut HSD_GObj) -> f32 => unsafe { crate::ft0877::ft_80087a8c(a0 as _) };
    fn ft_80087A98(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ft0877::ft_80087a98(a0 as _) };
    fn ft_80087AA4(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087aa4(a0 as _) };
    fn ft_80087AB4(a0: *mut HSD_GObj) -> u8 => unsafe { crate::ft0877::ft_80087ab4(a0 as _) };
    fn ft_80087AEC(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087aec(a0 as _) };
    fn ft_80087B34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ft0877::ft_80087b34(a0 as _) };
    fn ft_80089884(a0: *mut HSD_GObj) -> *mut Struct2070 => unsafe { crate::ft0892::ft_80089884(a0 as _) as _ };
    fn ft_80089890(a0: *mut HSD_GObj) -> *mut UnkPlBonusBits => unsafe { crate::ft0892::ft_80089890(a0 as _) as _ };
    fn ft_8008989C(a0: *mut HSD_GObj) -> u16 => unsafe { crate::ft0892::ft_8008989c(a0 as _) };
    fn ft_800898A8(a0: *mut HSD_GObj) -> *mut Struct2074 => unsafe { crate::ft0892::ft_800898a8(a0 as _) as _ };
    fn ft_800898B4(a0: *mut HSD_GObj) -> *mut ft_800898B4_t => unsafe { crate::ft0892::ft_800898b4(a0 as _) as _ };
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec3) => unsafe { let _ = crate::ftlib::ftlib_get_pos(a0 as _, &mut *(a1 as *mut _)); };
    fn ftLib_GetFacingDir(a0: *mut HSD_GObj) -> f32 => unsafe { crate::ftlib::ftlib_get_facing_dir(a0 as _) };
    fn ftLib_GetGroundAir(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_ground_air(a0 as _) };
    fn ftLib_IsInputDisabled(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_input_disabled(a0 as _) };
    fn ftLib_GetItem(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::ftlib::get_item(a0 as _) as _ };
    fn ftLib_IsFighter(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_fighter(a0 as _) };
    fn ftLib_GetPlayerIndex(a0: *mut HSD_GObj) -> u8 => unsafe { crate::ftlib::ftlib_get_player_index(a0 as _) };
    fn ftLib_GetPercent(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_percent(a0 as _) };
    fn ftLib_IsLastAttackerSubFighter(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_is_last_attacker_sub_fighter(a0 as _) };
    fn ftLib_GetLastAttackerSlot(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_last_attacker_slot(a0 as _) };
    fn ftLib_IsDead(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_dead(a0 as _) };
    fn ftLib_IsDeadUp(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_dead_up(a0 as _) };
    fn ftLib_IsDeadUpStar(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_dead_up_star(a0 as _) };
    fn ftLib_IsDeadUpFall(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_dead_up_fall(a0 as _) };
    fn ftLib_IsSubFighter(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_is_sub_fighter(a0 as _) };
    fn ftLib_GetLastHitSourceType(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_last_hit_source_type(a0 as _) };
    fn ftLib_GetLastHitSourceKind(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::ftlib_get_last_hit_source_kind(a0 as _) };
    fn ftLib_GetGroundSlopeAngle(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftlib_get_ground_slope_angle(a0 as _) };
    fn ftLib_IsBeingNudged(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_is_being_nudged(a0 as _) };
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::get_motion_id(a0 as _) };
    fn gm_8016AEEC() -> u32;
    fn gm_8016B110() -> bool;
    fn gm_8016B168() -> bool;
    fn gm_8016B204() -> bool;
    fn gm_8016B6E8(a0: i32, a1: i32);
    fn gm_8016F2F8(a0: i32, a1: u8) -> i32;
    fn gm_GetFrameCount() -> u32;
    fn ifMagnify_802FB6E8(a0: i32) -> i32 => unsafe { melee_world::tu::ifmagnify::ifMagnify_802FB6E8(a0 as _) };
    fn itGetKind(a0: *mut HSD_GObj) -> i32 => unsafe { melee_items::tu::it_26B1::itGetKind(a0 as _) as _ };
    fn itGetOwner(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { melee_items::tu::it_26B1::itGetOwner(a0 as _) as _ };
    fn it_8026B30C(a0: *mut HSD_GObj) -> i32 => unsafe { melee_items::itlib::it_8026b30c(&*(a0 as *const _)) };
    fn it_8026B774(a0: *mut HSD_GObj, a1: u8) -> bool => unsafe { melee_items::itlib::it_8026b774(&*(a0 as *const _), a1 as _) };
    fn it_8026B7BC(a0: *mut HSD_GObj) -> i32 => unsafe { melee_items::itlib::it_8026b7bc(&*(a0 as *const _)) };
    fn it_8026B7CC(a0: *mut HSD_GObj) -> i32 => unsafe { melee_items::itlib::it_8026b7cc(&*(a0 as *const _)) };
    fn it_8026B7E8(a0: *mut HSD_GObj) -> i32 => unsafe { melee_items::itlib::it_8026b7e8(&*(a0 as *const _)) };
    fn it_8026B960(a0: *mut HSD_GObj) -> f32 => unsafe { melee_items::itlib::it_8026b960(&*(a0 as *const _)) };
    fn it_8026C1D4() -> u32;
    fn pl_80040ED4(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: f32) => unsafe { let _ = crate::plbonus::pl_80040ed4(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); };
    fn pl_80041720(a0: i32, a1: i32, a2: u16) => unsafe { let _ = crate::plbonus::pl_80041720(a0 as _, a1 as _, a2 as _); };
    fn pl_80041744(a0: i32, a1: i32, a2: u16) => unsafe { let _ = crate::plbonus::pl_80041744(a0 as _, a1 as _, a2 as _); };
    fn pl_800419AC(a0: i32, a1: i32, a2: u16) => unsafe { let _ = crate::plbonus::pl_800419ac(a0 as _, a1 as _, a2 as _); };
    }
}

/// `HSD_ASSERT(line, cond)`
#[inline]
fn hsd_assert(cond: bool, line: u32, expr: &str) {
    if !cond {
        panic!("src/melee/pl/plbonuslib.c:{}: {}", line, expr);
    }
}

/// `pl_CalculateAverage` (plbonusinline.h)
#[inline]
fn pl_CalculateAverage(val: f32, total: f32) -> f32 {
    hsd_assert(total != 0.0, 16, "total != 0.0F");
    val / total
}

#[inline]
fn tbl(slot: i32) -> *mut pl_StaleMoveTableExt_t {
    Player_GetStaleMoveTableIndexPtr2(slot)
}

#[inline]
fn c() -> &'static pl_804D6470_t {
    unsafe { &*pl_804D6470 }
}

/// `ABS(x)` (`(x) < 0 ? -(x) : (x)`)
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `pl_Verify_gm_8016AEDC` (static inline, pl/inlines.h)
#[inline]
fn pl_Verify_gm_8016AEDC() -> bool {
    ext::gm_GetFrameCount() != 0 && ext::gm_GetFrameCount() != (-2i32) as u32
}

/// `pl_CheckIfSameTeam` (static inline, pl/inlines.h)
#[inline]
fn pl_CheckIfSameTeam(slot_a: i32, slot_b: i32) -> bool {
    if ext::gm_8016B168() {
        let team_a = Player_GetTeam(slot_a);
        let team_b = Player_GetTeam(slot_b);
        if team_b == team_a {
            return true;
        }
    }
    false
}

/// `unk_cond` (static inline)
#[inline]
fn unk_cond(arg0: i32, temp_r23: i32) -> bool {
    temp_r23 == 6 || temp_r23 == arg0 || pl_CheckIfSameTeam(arg0, temp_r23)
}

/// `between_A1_D0` (static inline)
#[inline]
fn between_A1_D0(x: i32) -> bool {
    x >= 0xA1 && x < 0xD0
}

/// `plBonusLib_8003D514` (static)
fn plBonusLib_8003D514(arg0: i32) {
    let mut sp18: Vec3 = unsafe { core::mem::zeroed() };
    let mut sp_c: Vec3 = unsafe { core::mem::zeroed() };
    let temp_r31 = Player_GetEntity(arg0);

    if ext::ftLib_IsDeadUp(temp_r31) {
        return;
    }
    ext::ftLib_GetPos(temp_r31, &mut sp18);

    for var_r29 in 0..6 {
        if var_r29 == arg0 {
            continue;
        }

        let temp_r30 = Player_GetEntity(var_r29);
        if !Player_8003221C(var_r29) || ext::ftLib_IsDead(temp_r30) {
            continue;
        }

        let temp_r31_2 = tbl(var_r29);
        ext::ftLib_GetPos(temp_r30, &mut sp_c);
        let temp_f1 = ext::ftLib_GetFacingDir(temp_r30);
        unsafe {
            (*temp_r31_2).xDD1.anon_0.set_bit6(1);

            if (temp_f1 * sp18.x) > (temp_f1 * sp_c.x) {
                (*temp_r31_2).xDD1.anon_0.set_bit7(1);
            }
        }
    }
}

pub fn pl_8003D60C(arg0: i32) -> bool {
    melee_core::ct!("pl_8003D60C");
    ((arg0 >= 0xA0) && (arg0 < 0xA1)) || ((arg0 >= 0xEA) && (arg0 < 0xEE)) || (arg0 == 0xE1)
}

pub fn pl_8003D644(arg0: i32, arg1: i32, arg2: i32, arg3: i32) {
    melee_core::ct!("pl_8003D644");
    unsafe {
        let temp_r26 = tbl(arg0);
        let temp_r3 = Player_GetEntityAtIndex(arg0, arg1);
        let temp_r27: i32 = ext::ftLib_GetMotionId(temp_r3);
        let temp_r28: i32 = ext::ftLib_GetLastHitSourceType(temp_r3);
        let temp_r29: i32 = ext::ftLib_GetLastHitSourceKind(temp_r3);
        let temp_r30: *mut UnkPlBonusBits = ext::ft_80089890(temp_r3);
        let temp_r24: u16 = ext::ft_8008989C(temp_r3);
        let temp_r31: *mut ft_800898B4_t = ext::ft_800898B4(temp_r3);
        let temp_r23: i32 = ext::ftLib_GetLastAttackerSlot(temp_r3);
        let temp_r22: i32 = ext::ftLib_IsLastAttackerSubFighter(temp_r3);
        let pl = c();

        if arg1 != 1 {
            (*temp_r26).x0_staleMoveTable.xCB8 = temp_r23;
            (*temp_r26).x0_staleMoveTable.xCBC = *temp_r30;
            (*temp_r26).x0_staleMoveTable.xCC0 = *temp_r31;
            (*temp_r26).x0_staleMoveTable.xCD4 = temp_r24;
            pl_8003FED0(arg0, arg1);
            if ext::ftLib_IsDeadUpFall(temp_r3) {
                pl_80038824(arg0, 0x81);
            }
            if ext::ftLib_IsDeadUpStar(temp_r3) {
                (*temp_r26).x0_staleMoveTable.xC94 = (*temp_r26).x0_staleMoveTable.xC94.wrapping_add(1);
            }
            if ext::ftLib_IsDeadUp(temp_r3) {
                (*temp_r26).xDD1.anon_0.set_bit2(1);
            }
            if temp_r27 == 0 {
                (*temp_r26).xDD1.anon_0.set_bit3(1);
            }
            if temp_r27 == 2 {
                (*temp_r26).xDD1.anon_0.set_bit0(1);
            }
            if temp_r27 == 1 {
                (*temp_r26).xDD1.anon_0.set_bit1(1);
            }
            if ext::ft_80087858(temp_r3) {
                pl_80038788(arg0, 0x83, 1);
            }
            if ext::ft_80087878(temp_r3, arg2) {
                pl_80038824(arg0, 0x84);
            }
            plBonusLib_8003D514(arg0);
            if arg3 as u32 == 0x5F {
                pl_80038788(arg0, 0x9C, 1);
            }

            match temp_r28 {
                1 | 2 => {
                    if (*temp_r31).x4 <= pl.x128 && (*temp_r31).x0 >= pl.x12C {
                        pl_80038824(arg0, 0x85);
                    }
                    if temp_r28 == 2 {
                        if pl_8003D60C(temp_r29) {
                            pl_80038788(arg0, 0x88, 1);
                        } else if temp_r23 == arg0 {
                            pl_80038788(arg0, 0x9C, 1);
                        }
                        if temp_r29 == 0xEA {
                            (*temp_r26).xD00 = (*temp_r26).xD00.wrapping_add(1);
                        }
                    }
                }
                3 => match temp_r29 {
                    1 => pl_80038788(arg0, 0x87, 1),
                    _ => pl_80038788(arg0, 0x88, 1),
                },
                _ => {}
            }

            if temp_r23 == 6 && temp_r28 == 2 && temp_r29 == 6 {
                pl_80038788(arg0, 0xC7, 1);
            }
            if temp_r28 != 2 || !between_A1_D0(temp_r29) {
                if unk_cond(arg0, temp_r23) {
                    if temp_r23 == arg0 {
                        ext::pl_80041744(temp_r23, temp_r22, temp_r24);
                    }
                } else {
                    ext::pl_80041720(temp_r23, temp_r22, temp_r24);
                }
            }

            if unk_cond(arg0, temp_r23) {
                (*temp_r26).xD60 = 0;
                (*temp_r26).xD64 = 6;
            } else {
                let temp_r3_2 = tbl(temp_r23);
                if ((*temp_r3_2).xD60 != 0) && ((*temp_r3_2).xD64 == arg0) {
                    pl_80038824(temp_r23, 0x79);
                    (*temp_r3_2).xD60 = 0;
                    (*temp_r3_2).xD64 = 6;
                }
                (*temp_r26).xD60 = 0;
                (*temp_r26).xD64 = temp_r23;
            }

            if !unk_cond(arg0, temp_r23) {
                if ext::ftLib_IsDeadUpStar(temp_r3) {
                    pl_80038824(temp_r23, 0x6D);
                }
                if temp_r27 == 3 {
                    let temp_r3_3 = tbl(temp_r23);
                    (*temp_r3_3).x0_staleMoveTable.xCB4 = (*temp_r3_3).x0_staleMoveTable.xCB4.wrapping_add(1);
                }
                if pl.x120 == Player_GetDamage(arg0) {
                    pl_80038824(temp_r23, 0x76);
                }

                match temp_r28 {
                    1 | 2 => {
                        let x3 = (*temp_r30).x3;
                        if x3 >= 1 && x3 <= 3 {
                            pl_80038824(temp_r23, 0x6E);
                        }
                        if x3 >= 0x40 && x3 <= 0x43 {
                            pl_80038824(temp_r23, 0xC0);
                        }
                        if x3 == 0x46 {
                            pl_80038824(temp_r23, 0xC1);
                        }
                        if x3 == 0x5F {
                            pl_80038824(temp_r23, 0xAD);
                        }
                        if x3 == 0x63 {
                            pl_80038824(temp_r23, 0x70);
                        }
                        if x3 == 0x61 {
                            pl_80038824(temp_r23, 0xC6);
                        }
                        if (*temp_r30).x2_b7() != 0 {
                            pl_80038824(temp_r23, 0x6F);
                        }
                        if temp_r28 == 2 && (*temp_r30).x2_b3() != 0 {
                            pl_80038824(temp_r23, 0x72);
                        }
                        if (*temp_r31).x11_b0() != 0 {
                            pl_80038824(temp_r23, 0x71);
                        }
                        if (*temp_r31).x10_b0() != 0 {
                            pl_80038824(temp_r23, 6);
                        }
                        if (*temp_r31).kb_applied1 >= pl.x11C {
                            pl_80038824(temp_r23, 0x74);
                        }
                        if (*temp_r31).x11_b2() != 0 {
                            pl_80038824(temp_r23, 0x75);
                        }
                        if (*temp_r31).x11_b1() != 0 {
                            pl_80038824(temp_r23, 0x77);
                        }
                        if (*temp_r31).x10_b1() != 0 {
                            pl_80038824(temp_r23, 0xB0);
                        }
                        if (*temp_r31).x10_b2() != 0 {
                            pl_80038824(temp_r23, 0xBB);
                        }
                        if (*temp_r31).x10_b3() != 0 {
                            pl_80038824(temp_r23, 0xBF);
                        }
                        if (*temp_r31).x10_b4() != 0 {
                            pl_80038824(temp_r23, 0xBA);
                        }
                        if (*temp_r31).x10_b5() != 0 {
                            pl_80038824(temp_r23, 0xB7);
                        }
                        if (*temp_r31).x10_b6() != 0 {
                            pl_80038824(temp_r23, 0xB8);
                        }
                        if (*temp_r31).x11_b3() != 0 {
                            pl_80038824(temp_r23, 0xB);
                        }
                        if (*temp_r31).x11_b4() != 0 {
                            pl_80038824(temp_r23, 0x7E);
                        }
                        if !ext::gm_8016B204() && !ext::gm_8016B110() && ext::gm_8016AEEC().wrapping_add(1) <= pl.x118 {
                            pl_80038824(temp_r23, 0x73);
                        }
                        if temp_r28 == 2 {
                            if x3 == 0x62 {
                                match temp_r29 {
                                    0 | 3 => pl_80038824(temp_r23, 0x97),
                                    1 | 2 | 4 => pl_80038824(temp_r23, 0x98),
                                    30 => pl_80038824(temp_r23, 0xAC),
                                    20 => pl_80038824(temp_r23, 0xB2),
                                    17 => pl_80038824(temp_r23, 0xB1),
                                    _ => {}
                                }
                            }
                            if between_A1_D0(temp_r29) {
                                pl_80038824(temp_r23, 0xCB);
                            }
                            if pl_8003D60C(temp_r29) {
                                pl_80038824(temp_r23, 0x80);
                            }
                            if temp_r29 == 0xEA {
                                let temp_r3_4 = tbl(temp_r23);
                                (*temp_r3_4).xCFC = (*temp_r3_4).xCFC.wrapping_add(1);
                            }
                        }
                    }
                    3 => match temp_r29 {
                        1 => pl_80038824(temp_r23, 0x7F),
                        _ => pl_80038824(temp_r23, 0x80),
                    },
                    _ => {}
                }
            }
        }
    }
}

pub fn pl_8003DF44(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003DF44");
    let temp_r3: i32 = ext::ftLib_GetLastAttackerSlot(Player_GetEntityAtIndex(arg0, arg1));

    let var_r0_2 = temp_r3 == 6 || temp_r3 == arg0 || pl_CheckIfSameTeam(arg0, temp_r3);
    if var_r0_2 {
        return;
    }

    let smte = tbl(temp_r3);
    unsafe { (*smte).xD54 = c().xB4 };
}

pub fn pl_8003DFF4(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003DFF4");
    pl_8003FC88(arg0, arg1, arg2);
    pl_8003FF44(arg0, arg1, arg2);
    pl_8003EAAC(arg0, arg1, arg2);
}

pub fn pl_8003E058(arg0: i32, arg1: i32, arg2: i32, arg3: i32) {
    melee_core::ct!("pl_8003E058");
    pl_80038824(arg2, 0x3F);

    if arg0 == 6 {
        return;
    }
    if pl_CheckIfSameTeam(arg2, arg0) {
        return;
    }

    pl_80038824(arg0, 0x3E);
}

pub fn pl_8003E0E8(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003E0E8");
    if arg1 != 0 {
        return;
    }
    pl_80038824(arg0, 0x40);
}

pub fn pl_8003E114(arg0: i32, arg1: i32, arg2: f32) {
    melee_core::ct!("pl_8003E114");
    if arg2 <= c().x84 {
        pl_80038788(arg0, 0x41, 1);
    }
}

pub fn pl_8003E150(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003E150");
    let stale_moves = tbl(slot);
    unsafe {
        (*stale_moves).x0_staleMoveTable.xCD8 = (*stale_moves).x0_staleMoveTable.xCD8.wrapping_add(1);
    }
}

pub fn pl_8003E17C(player_id: i32, arg1: i32, item_gobj: *mut HSD_GObj) {
    melee_core::ct!("pl_8003E17C");
    let temp_r31 = tbl(player_id);
    let kind: ItemKind = ext::itGetKind(item_gobj);
    let kind2: ItemKind;

    if (kind >= It_Kind_Capsule) && (kind < It_Kind_L_Gun_Ray) {
        kind2 = kind;
    } else {
        // Not one of the common items
        kind2 = match kind {
            It_Kind_Lucky_Egg => It_Kind_L_Gun_Ray,
            It_Kind_WhispyApple => It_Kind_StarRod_Star,
            It_Kind_WhispyHealApple => It_Kind_LipStick_Spore,
            It_Kind_Hammer_Head => It_Kind_S_Scope_Beam,
            _ => -1,
        };
    }

    if kind2 != -1
        && (kind2 == It_Kind_L_Gun_Ray
            || kind2 == It_Kind_Egg
            || kind2 == It_Kind_S_Scope_Beam
            || ext::it_8026B7E8(item_gobj) == 1)
    {
        unsafe {
            if !ext::it_8026B774(item_gobj, player_id as u8) {
                let p = &mut (*temp_r31).x0_staleMoveTable.x674[kind2 as usize];
                *p = p.wrapping_add(1);
            }

            for cnt in 0..6 {
                if ext::it_8026B774(item_gobj, cnt as u8) {
                    return;
                }
            }

            let p = &mut (*temp_r31).x0_staleMoveTable.x710[kind2 as usize];
            *p = p.wrapping_add(1);
        }
    }
}

pub fn pl_8003E2CC(arg0: i32, pl_itemlog_kind: Pl_ItemLog) -> u32 {
    melee_core::ct!("pl_8003E2CC");
    hsd_assert(pl_itemlog_kind < Pl_ItemLog_Terminate, 555, "pl_itemlog_kind < Pl_ItemLog_Terminate");
    unsafe { (*tbl(arg0)).x0_staleMoveTable.x674[pl_itemlog_kind as usize] }
}

pub fn pl_8003E334(arg0: i32, pl_itemlog_kind: Pl_ItemLog) -> u32 {
    melee_core::ct!("pl_8003E334");
    hsd_assert(pl_itemlog_kind < Pl_ItemLog_Terminate, 564, "pl_itemlog_kind < Pl_ItemLog_Terminate");
    unsafe { (*tbl(arg0)).x0_staleMoveTable.x710[pl_itemlog_kind as usize] as u32 }
}

pub fn pl_8003E39C(arg0: i32) -> i32 {
    melee_core::ct!("pl_8003E39C");
    let mut sum: i32 = 0;
    let mut k: i32 = 0;
    while k < Pl_ItemLog_Terminate as i32 {
        hsd_assert((k as u32) < Pl_ItemLog_Terminate, 555, "pl_itemlog_kind < Pl_ItemLog_Terminate");
        sum = sum.wrapping_add(unsafe { (*tbl(arg0)).x0_staleMoveTable.x674[k as usize] } as i32);
        k += 1;
    }
    sum
}

pub fn pl_8003E420(arg0: i32) -> u32 {
    melee_core::ct!("pl_8003E420");
    let mut sum: i32 = 0;
    let mut k: i32 = 0;
    while k < Pl_ItemLog_Terminate as i32 {
        hsd_assert((k as u32) < Pl_ItemLog_Terminate, 564, "pl_itemlog_kind < Pl_ItemLog_Terminate");
        sum = sum.wrapping_add(unsafe { (*tbl(arg0)).x0_staleMoveTable.x710[k as usize] });
        k += 1;
    }
    sum as u32
}

/// `match_item_kind` (static inline)
pub fn match_item_kind(kind: i32) -> i32 {
    melee_core::ct!("match_item_kind");
    if kind >= It_Kind_Capsule && kind < It_Kind_L_Gun_Ray {
        kind
    } else {
        match kind {
            It_Kind_Lucky_Egg => Pl_ItemLog_Unk35 as i32,
            It_Kind_WhispyApple => Pl_ItemLog_Unk36 as i32,
            It_Kind_WhispyHealApple => Pl_ItemLog_Unk37 as i32,
            It_Kind_Hammer_Head => Pl_ItemLog_Unk38 as i32,
            _ => -1,
        }
    }
}

pub fn pl_8003E4A4(slot: i32, arg1: bool, arg2: *mut c_void, count: i32) {
    melee_core::ct!("pl_8003E4A4");
    let table = tbl(slot);
    let moves = arg2 as *const i32;
    let mut seen = [0u32; Pl_ItemLog_Terminate as usize];

    unsafe {
        for i in 0..count {
            let k = match_item_kind(*moves.offset(i as isize));
            if k < Pl_ItemLog_Terminate as i32 && k >= 0 {
                seen[k as usize] = 1;
            }
        }

        for i in 0..Pl_ItemLog_Terminate as usize {
            if seen[i] == 1 {
                (*table).x0_staleMoveTable.x7AC[i] = (*table).x0_staleMoveTable.x7AC[i].wrapping_add(1);
                if (*table).x0_staleMoveTable.x7AC[i] as u32 == c().x138 {
                    pl_80038788(slot, 0x9D, 1);
                    (*table).x0_staleMoveTable.x7AC[i] = 0;
                }
            } else {
                (*table).x0_staleMoveTable.x7AC[i] = 0;
            }
        }
    }
}

/// `&Player_GetStaleMoveTableIndexPtr2(slot)->x0_staleMoveTable.total_attack_count_struct.x598[kind]`
/// (unchecked: `kind` >= 8 indexes on into `x848[]`)
#[inline]
unsafe fn x598_ptr(slot: i32, kind: i32) -> *mut u32 {
    (tbl(slot) as *mut u8).add(0x2C + 0x598).offset(4 * kind as isize) as *mut u32
}

pub fn pl_8003E70C(igobj: *mut HSD_GObj) {
    melee_core::ct!("pl_8003E70C");
    let temp_r30 = ext::itGetOwner(igobj);

    hsd_assert(
        It_PKind_Start <= ext::itGetKind(igobj) && ext::itGetKind(igobj) < It_PKind_Terminate,
        634,
        "It_PKind_Start <= itGetKind(igobj) && itGetKind(igobj) < It_PKind_Terminate",
    );
    if !ext::ftLib_IsFighter(temp_r30) {
        return;
    }

    let slot = ext::ftLib_GetPlayerIndex(temp_r30) as i32;
    let temp_r3 = ext::itGetKind(igobj);
    unsafe {
        let p = x598_ptr(slot, temp_r3);
        *p = (*p).wrapping_add(1);
    }
}

pub fn pl_8003E7D4(arg0: i32, kind: i32) -> u32 {
    melee_core::ct!("pl_8003E7D4");
    hsd_assert(It_PKind_Start <= kind && kind < It_PKind_Terminate, 649, "It_PKind_Start <= kind && kind < It_PKind_Terminate");
    unsafe { *x598_ptr(arg0, kind) }
}

pub fn pl_8003E854(arg0: i32, arg1: i32, arg2: *mut HSD_GObj) {
    melee_core::ct!("pl_8003E854");
    let temp_r31 = Player_GetEntityAtIndex(arg0, arg1);
    let temp_r3 = ext::itGetOwner(arg2);

    if temp_r31 != temp_r3 && ext::it_8026B7BC(arg2) != 0 {
        pl_80038824(arg0, 0x9A);

        if ext::itGetKind(arg2) == It_Kind_Dosei {
            pl_80038788(arg0, 0xB6, 1);
        }

        if arg1 == 0 && ext::ftLib_IsFighter(temp_r3) && ext::ftLib_IsSubFighter(temp_r3) == 0 {
            let temp_r31_2: u8 = ext::ftLib_GetPlayerIndex(temp_r3);
            unsafe { (*tbl(arg0)).xD6C = temp_r31_2 as i32 };
        }
    }

    if ext::itGetKind(arg2) == It_Kind_BombHei {
        if ext::it_8026B960(arg2) <= c().x180 {
            ext::pl_800419AC(arg0, arg1, ext::it_8026B7CC(arg2) as u16);
        }
    }

    if arg1 != 0 {
        return;
    }
    unsafe { (*tbl(arg0)).x0_staleMoveTable.xCA8 = 0 };
}

pub fn pl_8003E978(slot: i32, fp_x221F_b4: bool, item_gobj: *mut HSD_GObj) {
    melee_core::ct!("pl_8003E978");
    fn_8003E998(slot, fp_x221F_b4 as i32);
}

pub fn fn_8003E998(arg0: i32, index: i32) {
    melee_core::ct!("fn_8003E998");
    let temp_r31 = tbl(arg0);
    unsafe {
        if index == 0 && (*temp_r31).xD6C != 6 {
            let gobj = Player_GetEntityAtIndex(arg0, index);
            (*ext::ft_800898A8(gobj)).x207C.y = (*temp_r31).xD6C;
        }
    }
}

pub fn pl_8003EA08(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003EA08");
    if arg1 != 0 {
        return;
    }
    unsafe { (*tbl(slot)).xD6C = 6 };
}

pub fn pl_8003EA40(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_8003EA40");
    if arg3 != 0 {
        return;
    }
    if arg2 != arg4 {
        return;
    }
    pl_80038824(arg0, 0x9B);
}

pub fn pl_8003EA74(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003EA74");
    if arg1 != 0 {
        return;
    }
    unsafe { (*tbl(arg0)).xD44 = 0 };
}

pub fn pl_8003EAAC(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003EAAC");
    let temp_r31 = tbl(arg0);

    if arg1 == 0 && arg2 == 0x59 {
        unsafe {
            (*temp_r31).x0_staleMoveTable.xCA8 = (*temp_r31).x0_staleMoveTable.xCA8.wrapping_add(1);

            if (*temp_r31).x0_staleMoveTable.xCA8 == ext::it_8026C1D4() {
                pl_80038788(arg0, 0xC2, 1);
            }
        }
    }
}

pub fn pl_8003EB30(arg0: f32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    melee_core::ct!("pl_8003EB30");
    if arg1 != 6 && arg1 != arg3 {
        unsafe {
            let temp_r31 = tbl(arg1);
            let var_r3: *mut pl_StaleMoveTableExt_t = if arg3 != 6 { tbl(arg3) } else { null_mut() };
            (*temp_r31).x0_staleMoveTable.xC6C += arg0;
            if arg5 == 0x34 {
                (*temp_r31).x0_staleMoveTable.xC70 += arg0;
            }
            if (arg5 as u32).wrapping_sub(0x5C) <= 1 {
                (*temp_r31).x0_staleMoveTable.xC74 += arg0;
            }
            if arg3 != 6 {
                if arg4 == 0 {
                    (*var_r3).x0_staleMoveTable.xC78[arg1 as usize] += arg0;
                }
                ext::pl_80040ED4(arg1, arg2, arg3, arg4, arg5, arg0);
            }
            let tmp = tbl(arg1);
            (*tmp).xD08 = 0;
        }
    }
}

pub fn pl_8003EC30(slot: i32, arg1: i32, arg2: i32, arg3: f32) {
    melee_core::ct!("pl_8003EC30");
    if arg1 != 0 {
        return;
    }

    let stale_moves = tbl(slot);
    match arg2 {
        1 | 3 => unsafe { (*stale_moves).x0_staleMoveTable.xC98 += arg3 },
        _ => {}
    }
}

pub fn pl_8003EC9C(arg0: i32, arg1: i32, arg2: f32, arg3: f32) {
    melee_core::ct!("pl_8003EC9C");
    let temp_r3 = tbl(arg0);
    if arg1 != 0 {
        return;
    }

    unsafe {
        (*temp_r3).x0_staleMoveTable.xC60 += arg3;
        if (*temp_r3).x0_staleMoveTable.xC64 < arg2 {
            (*temp_r3).x0_staleMoveTable.xC64 = arg2;
        }
        (*temp_r3).xD10 = 0;
    }
}

pub fn pl_8003ED0C(arg0: i32, arg1: i32, r5: i32, arg3: i32, arg2: f32) {
    melee_core::ct!("pl_8003ED0C");
    if arg0 == r5 {
        return;
    }
    if pl_CheckIfSameTeam(r5, arg0) {
        return;
    }

    unsafe {
        let temp_r3 = tbl(arg0);
        (*temp_r3).x0_staleMoveTable.xCA4 = (*temp_r3).x0_staleMoveTable.xCA4.wrapping_add(1);
        let temp_r4: i32 = (*temp_r3).x0_staleMoveTable.xCA4;
        let temp_f30: f32 = fmadds((*temp_r3).x0_staleMoveTable.xCA0, temp_r4.wrapping_sub(1) as f32, arg2);

        let total: f32 = temp_r4 as f32;
        (*temp_r3).x0_staleMoveTable.xCA0 = pl_CalculateAverage(temp_f30, total);
    }
}

pub fn fn_8003EE2C(arg0: i32, arg1: i32) {
    melee_core::ct!("fn_8003EE2C");
    unsafe {
        let temp_r31 = tbl(arg0);
        let temp_r30 = Player_GetEntityAtIndex(arg0, arg1);
        let pl = c();
        if arg1 == 1 {
            return;
        }
        if !pl_Verify_gm_8016AEDC() {
            return;
        }

        (*temp_r31).xD08 = (*temp_r31).xD08.wrapping_add(1);
        if (*temp_r31).xD08 > (*temp_r31).xD04 {
            (*temp_r31).xD04 = (*temp_r31).xD08;
        }
        (*temp_r31).xD10 = (*temp_r31).xD10.wrapping_add(1);
        if (*temp_r31).xD10 as u32 > (*temp_r31).xD0C {
            (*temp_r31).xD0C = (*temp_r31).xD10 as u32;
        }
        if ext::ftLib_GetGroundAir(temp_r30) == GA_Air as i32 {
            (*temp_r31).xD20 = (*temp_r31).xD20.wrapping_add(1);
        } else {
            (*temp_r31).xD24 = (*temp_r31).xD24.wrapping_add(1);
        }
        let temp_r28: i32 = (*ext::ft_80089884(temp_r30)).anon_0.x2073 as i32;
        if (*ext::ft_80089884(temp_r30)).anon_0.x2072_b1() != 0 {
            (*temp_r31).xD18 = (*temp_r31).xD18.wrapping_add(1);
            if ((*temp_r31).xD18 as f32) >= pl.x64 {
                (*temp_r31).xD14 = (*temp_r31).xD14.wrapping_add(1);
            }
        } else {
            (*temp_r31).xD18 = 0;
        }
        if temp_r28 == 0x68 {
            (*temp_r31).xD1C = (*temp_r31).xD1C.wrapping_add(1);
        }
        if ext::ftLib_GetGroundAir(temp_r30) == GA_Ground as i32 {
            let var_f1: f32 = if ext::ftLib_GetGroundSlopeAngle(temp_r30) < 0.0 {
                -ext::ftLib_GetGroundSlopeAngle(temp_r30)
            } else {
                ext::ftLib_GetGroundSlopeAngle(temp_r30)
            };
            if var_f1 >= pl.x58 {
                (*temp_r31).xD28 = (*temp_r31).xD28.wrapping_add(1);
            }
        }
        if ext::ftLib_IsBeingNudged(temp_r30) {
            (*temp_r31).xD2C = (*temp_r31).xD2C.wrapping_add(1);
        }
        let temp_r3_3: i32 = ext::ifMagnify_802FB6E8(arg0);
        if temp_r3_3 != 0 {
            (*temp_r31).xD30 = (*temp_r31).xD30.wrapping_add(1);
            (*temp_r31).xDD0.anon_0.set_bit7(1);
        } else {
            (*temp_r31).xDD0.anon_0.set_bit7(0);
        }
        match temp_r3_3 {
            1 => (*temp_r31).xDD0.anon_0.set_bit5(1),
            2 => (*temp_r31).xDD0.anon_0.set_bit4(1),
            3 => (*temp_r31).xDD0.anon_0.set_bit6(1),
            4 => (*temp_r31).xDD0.anon_0.set_bit3(1),
            _ => {}
        }
        if ext::ft_800878BC(temp_r30) {
            (*temp_r31).xD34 = (*temp_r31).xD34.wrapping_add(1);
        }
        if ext::ft_80087900(temp_r30) {
            (*temp_r31).xD38 = (*temp_r31).xD38.wrapping_add(1);
        }
        if ext::ft_80087944(temp_r30) {
            (*temp_r31).xD3C = (*temp_r31).xD3C.wrapping_add(1);
        }

        {
            let item_gobj = ext::ftLib_GetItem(temp_r30);
            if !item_gobj.is_null() {
                if match_item_kind(ext::itGetKind(item_gobj)) != -1 && ext::it_8026B30C(item_gobj) != 5 {
                    (*temp_r31).xD44 = (*temp_r31).xD44.wrapping_add(1);
                    if (*temp_r31).xD44 > (*temp_r31).xD40 {
                        (*temp_r31).xD40 = (*temp_r31).xD44;
                    }
                }
            } else {
                if (*temp_r31).xD44 != 0 {
                    if (*temp_r31).xD44 < (*temp_r31).xD48 || (*temp_r31).xD48 == 0 {
                        (*temp_r31).xD48 = (*temp_r31).xD44;
                    }
                    (*temp_r31).xD44 = 0;
                }
            }
        }

        if (*temp_r31).xD4C != 0 {
            (*temp_r31).xD4C -= 1;
        }
        if (*temp_r31).xD50 != 0 {
            (*temp_r31).xD50 -= 1;
        }
        if (*temp_r31).xD54 != 0 {
            (*temp_r31).xD54 -= 1;
        }
        (*temp_r31).xD58 = (*temp_r31).xD58.wrapping_add(1);
        if (*temp_r31).xD60 != 0 {
            (*temp_r31).xD60 -= 1;
        }
        if ext::ft_80087AEC(temp_r30) {
            (*temp_r31).xD68 = (*temp_r31).xD68.wrapping_add(1);
        }
    }
}

/// `plBonusLib_8003F294_inline` (static inline)
#[inline]
fn plBonusLib_8003F294_inline(stats: *mut plActionStats, attack: i32) -> u32 {
    pl_800386D8(stats, attack as isize)
}

pub fn fn_8003F294(slot: i32, index: i32) {
    melee_core::ct!("fn_8003F294");
    unsafe {
        let table = tbl(slot);
        let pl = c();

        Player_GetEntityAtIndex(slot, index);
        let stats = Player_GetActionStats(slot);

        if index == 1 {
            return;
        }

        pl_8003906C(slot, 0x3B, null_mut(), pl.x70, pl.x74, plBonusLib_8003F294_inline(stats, 0x6B), &mut (*table).xD9C);
        pl_8003906C(slot, 0x45, null_mut(), pl.x90, pl.x94, plBonusLib_8003F294_inline(stats, 0x70), &mut (*table).xDA0);
        {
            let e8 = pl_800386E8(stats as *mut pl_800386E8_arg0_t);
            pl_8003906C(slot, 0x49, null_mut(), pl.xA8, pl.xAC, e8 as u32, &mut (*table).xDA4);
        }
        pl_8003906C(slot, 0x54, null_mut(), pl.xC4, pl.xC8, plBonusLib_8003F294_inline(stats, 0x6F), &mut (*table).xDA8);
        pl_8003906C(
            slot,
            -1,
            &mut (*table).xDC8 as *mut i32 as *mut u32,
            pl.x18,
            pl.x1C,
            (*stats).hits.total,
            &mut (*table).xDAC,
        );
        pl_8003906C(slot, 0x13, null_mut(), pl.x2C, pl.x30, (*stats).attacks.total, &mut (*table).xDB0);
        pl_8003906C(slot, 0x5C, null_mut(), pl.xE0, pl.xE4, (*table).xD70 as u32, &mut (*table).xDB4);
        pl_8003906C(slot, 0x99, null_mut(), pl.x130, pl.x134, (*table).xD34 as u32, &mut (*table).xDB8);
        pl_8003906C(
            slot,
            0x3D,
            null_mut(),
            pl.x7C,
            pl.x80,
            (*table).x0_staleMoveTable.xCD8 as u32,
            &mut (*table).xDBC,
        );

        {
            let t2 = tbl(slot);
            // memory squares: not fused on the console (notes/fp-audit.md)
            let mag: f32 = sqrtf(
                (*t2).x0_staleMoveTable.xCDC * (*t2).x0_staleMoveTable.xCDC
                    + (*t2).x0_staleMoveTable.xCE0 * (*t2).x0_staleMoveTable.xCE0,
            );
            let x8c: f32 = f32::from_bits(pl.x8C);
            pl_80039238(slot, 0x44, null_mut(), pl.x88, &mut (*table).xDC0, x8c, mag);
        }

        if pl_8003906C(
            slot,
            -1,
            &mut (*table).xDCC as *mut i32 as *mut u32,
            pl.xB8,
            pl.xBC,
            (*table).xCF4 as u32,
            &mut (*table).xDC4,
        ) != 0
        {
            let v: u32 = ext::gm_GetFrameCount();
            let xb8: u32 = pl.xB8;
            if xb8 == v {
                if (*table).xD5C <= xb8 {
                    (*table).xDD1.anon_0.set_bit5(1);
                }
            } else if (*table).xDD1.anon_0.bit5() != 0 && (*table).xD5C > xb8 {
                (*table).xDD1.anon_0.set_bit5(0);
            }
            (*table).xD5C = u32::MAX;
        }
    }
}

pub fn fn_8003F53C(arg0: i32, arg1: i32) {
    melee_core::ct!("fn_8003F53C");
    unsafe {
        let temp_r31 = tbl(arg0);
        let temp_r30 = Player_GetEntityAtIndex(arg0, arg1);

        if arg1 != 1 && !ext::ftLib_IsInputDisabled(temp_r30) {
            if !ext::ft_800877F8(temp_r30, 0x100) && !ext::ft_800877F8(temp_r30, 0x200) {
                (*temp_r31).xDD0.anon_0.set_bit1(1);
            }
            if ext::ft_800877F8(temp_r30, 0x80000000u32 as i32) {
                (*temp_r31).xDD0.anon_0.set_bit2(1);
            }
            if ext::ft_80087818(temp_r30, 0x100) || ext::ft_80087818(temp_r30, 0x200) {
                (*temp_r31).xCF4 = (*temp_r31).xCF4.wrapping_add(1);
                let temp_r3: u32 = (*temp_r31).xD58;
                if temp_r3 < (*temp_r31).xD5C {
                    (*temp_r31).xD5C = temp_r3;
                }
                (*temp_r31).xD58 = 0;
            }
        }
    }
}

pub fn fn_8003F654(slot: i32, index: i32, pos: *mut Vec3, prev_pos: *mut Vec3) {
    melee_core::ct!("fn_8003F654");
    unsafe {
        let table = tbl(slot);
        let entity = Player_GetEntityAtIndex(slot, index);
        let mut other_pos: Vec3 = core::mem::zeroed();
        let mut cam_pos: Vec3 = core::mem::zeroed();
        let pl = c();

        if pl_Verify_gm_8016AEDC() && index != 1 {
            let prev_pos2 = prev_pos;
            if ext::ftLib_IsDead(entity) {
                return;
            }

            let b34: i32 = ext::ft_80087B34(entity);
            if b34 == 1 {
                // squares of computed expressions: not fused on the console (fn_8003F654: fmuls, fmuls, fadds)
                let dist = sqrtf(
                    ((*prev_pos).x - (*pos).x) * ((*prev_pos).x - (*pos).x)
                        + ((*prev_pos2).y - (*pos).y) * ((*prev_pos).y - (*pos).y),
                );
                let teammate_slot: i32 = ext::ftLib_GetLastAttackerSlot(entity);
                (*table).xD80 += dist;
                if dist > (*table).xD84 {
                    (*table).xD84 = dist;
                }
                if teammate_slot != 6 {
                    let teammate_table = tbl(teammate_slot);
                    if dist > (*teammate_table).xD88 {
                        (*teammate_table).xD88 = dist;
                    }
                }
            } else if b34 == 0 {
                if ext::ftLib_GetGroundAir(entity) == 0 {
                    let mut abs = (*pos).x - (*prev_pos).x;
                    if abs < 0.0 {
                        abs = -abs;
                    }
                    (*table).xD74 += abs;
                } else {
                    let y_delta = (*pos).y - (*prev_pos).y;
                    let mut abs = y_delta;
                    if y_delta > 0.0 {
                        if abs < 0.0 {
                            abs = -abs;
                        }
                        (*table).xD78 += abs;
                    } else {
                        if abs < 0.0 {
                            abs = -abs;
                        }
                        (*table).xD7C += abs;
                    }
                }
            }

            let mut sum: f32 = 0.0;
            let mut count: i32 = 0;
            for i in 0..6 {
                if ((i != slot) && !pl_CheckIfSameTeam(slot, i))
                    && Player_8003221C(i)
                    && !ext::ftLib_IsDead(Player_GetEntity(i))
                {
                    Player_LoadPlayerCoords(i, &mut other_pos);
                    let pos_x2 = (*pos).x;
                    let mut abs = other_pos.x;
                    abs = pos_x2 - abs;
                    if abs < 0.0 {
                        abs = -abs;
                    }
                    sum += abs;
                    count += 1;
                }
            }

            if count != 0 {
                (*table).xD90 = (*table).xD90.wrapping_add(1);
                let mut avg_sum = pl_CalculateAverage(sum, count as f32);
                avg_sum = fmadds((*table).xD8C, (*table).xD90.wrapping_sub(1) as f32, avg_sum);
                (*table).xD8C = pl_CalculateAverage(avg_sum, (*table).xD90 as f32);
            }

            (*table).xD98 = (*table).xD98.wrapping_add(1);
            ext::Stage_UnkSetVec3TCam_Offset(&mut cam_pos);
            let mut abs = (*pos).x - cam_pos.x;
            if abs < 0.0 {
                abs = -abs;
            }
            let dist = fmadds((*table).xD94, (*table).xD98.wrapping_sub(1) as f32, abs);
            (*table).xD94 = pl_CalculateAverage(dist, (*table).xD98 as f32);
        }
    }
}

pub fn pl_8003FAA8(slot: i32, index: i32, pos: *mut Vec3, prev_pos: *mut Vec3) {
    melee_core::ct!("pl_8003FAA8");
    unsafe {
        let temp_r30 = tbl(slot);
        fn_8003EE2C(slot, index);
        fn_8003F294(slot, index);
        fn_8003F53C(slot, index);
        fn_8003F654(slot, index, pos, prev_pos);

        if pl_Verify_gm_8016AEDC() && index == 0 {
            let temp_r31: i32 = ext::ftLib_GetPercent(Player_GetEntityAtIndex(slot, index));
            let temp_f30: f32 = fmadds(
                (*temp_r30).x0_staleMoveTable.xC9C,
                ext::gm_GetFrameCount().wrapping_sub(1) as f32,
                temp_r31 as f32,
            );

            (*temp_r30).x0_staleMoveTable.xC9C = pl_CalculateAverage(temp_f30, ext::gm_GetFrameCount() as f32);
        }
    }
}

pub fn pl_8003FBFC(arg0: i32) -> i32 {
    melee_core::ct!("pl_8003FBFC");
    unsafe { (*tbl(arg0)).xCFC }
}

pub fn pl_8003FC20(arg0: i32) -> i32 {
    melee_core::ct!("pl_8003FC20");
    unsafe { (*tbl(arg0)).xD00 }
}

pub fn pl_8003FC44(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003FC44");
    let temp_r3 = tbl(slot);
    if arg1 != 0 {
        return;
    }
    unsafe { (*temp_r3).xDD0.anon_0.set_bit0(1) };
}

pub fn pl_8003FC88(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003FC88");
    unsafe {
        let stale_moves = tbl(arg0);
        let mut var_r4 = false;

        if arg1 == 1 {
            return;
        }
        let temp_r0: u32 = (*stale_moves).x0_staleMoveTable.xC90;
        match temp_r0 {
            0 => {
                if arg2 >= 1 && arg2 <= 3 {
                    var_r4 = true;
                }
            }
            1 => {
                if arg2 >= 6 && arg2 <= 8 {
                    var_r4 = true;
                }
            }
            2 => {
                if arg2 >= 9 && arg2 <= 11 {
                    var_r4 = true;
                }
            }
            3 => {
                if arg2 >= 17 && arg2 <= 48 {
                    var_r4 = true;
                }
            }
            _ => {}
        }

        if var_r4 {
            if temp_r0 == 3 {
                (*stale_moves).x0_staleMoveTable.xC90 = 0;
                pl_80038824(arg0, 0x56);
            } else {
                (*stale_moves).x0_staleMoveTable.xC90 = (*stale_moves).x0_staleMoveTable.xC90.wrapping_add(1);
            }
        } else {
            (*stale_moves).x0_staleMoveTable.xC90 = 0;
        }
    }
}

pub fn pl_8003FDA0(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FDA0");
    pl_80038788(arg0, 0x5B, 1);
}

pub fn pl_8003FDC8(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FDC8");
    let temp_r3 = tbl(arg0);
    unsafe { (*temp_r3).xD70 = (*temp_r3).xD70.wrapping_add(1) };
}

pub fn pl_8003FDF4(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FDF4");
    pl_80038788(arg0, 0xC4, 1);
}

pub fn pl_8003FE1C(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003FE1C");
    pl_80038824(slot, 0xA);
}

pub fn pl_8003FE40(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FE40");
    pl_80038824(arg0, 0x4B);
}

pub fn pl_8003FE64(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FE64");
    let mut var_r31 = 0;
    while var_r31 < 6 {
        if pl_80039418(var_r31, 0x1A) != 0 {
            break;
        }
        var_r31 += 1;
    }
    if var_r31 != 6 {
        return;
    }
    pl_80038788(arg0, 0x1A, 1);
}

pub fn pl_8003FED0(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003FED0");
    if arg1 != 0 {
        return;
    }

    let mut var_r31 = 0;
    while var_r31 < 6 {
        if pl_80039418(var_r31, 0x82) != 0 {
            break;
        }
        var_r31 += 1;
    }
    if var_r31 != 6 {
        return;
    }
    pl_80038788(arg0, 0x82, 1);
}

pub fn pl_8003FF44(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003FF44");
    let temp_r31 = tbl(arg0);
    let temp_r3 = Player_GetEntityAtIndex(arg0, arg1);
    if arg1 == 0 && arg2 >= 1 && arg2 <= 0x10 && ext::ft_80087A8C(temp_r3) > c().x38 {
        unsafe { (*temp_r31).xDD1.anon_0.set_bit4(1) };
    }
}

pub fn pl_8003FFDC(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_8003FFDC");
    let temp_r3 = Player_GetEntityAtIndex(arg2, arg3);
    if arg0 != arg2 && arg4 != ext::ft_80087A98(temp_r3) {
        pl_80038824(arg0, 0x46);
    }
}

pub fn pl_80040048(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040048");
    unsafe {
        let temp_r3 = Player_GetEntityAtIndex(arg0, arg1);
        let temp_r3_2: i32 = ext::ftLib_GetLastAttackerSlot(temp_r3);
        if arg1 != 0 {
            return;
        }
        if temp_r3_2 == 6 {
            return;
        }
        let temp_f31: f32 = (*ext::ft_800898B4(temp_r3)).kb_applied1;
        let temp_r3_3 = tbl(arg0);
        let pl = c();
        if temp_f31 >= pl.x98 {
            (*temp_r3_3).x0_staleMoveTable.xCE8 = (*temp_r3_3).x0_staleMoveTable.xCE8.wrapping_add(1);
        }
        if temp_f31 >= pl.xA0 && !ext::ft_80087AA4(temp_r3) && ext::ft_80087AB4(temp_r3) <= 1 {
            pl_80038824(arg0, 0x48);
        }
    }
}

pub fn pl_80040120(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040120");
    unsafe {
        let temp_r30 = tbl(arg0);
        let pl = c();

        if arg1 == 1 {
            return;
        }

        if (*temp_r30).xD4C != 0 {
            pl_80038824(arg0, 0x4A);
        }
        if (*temp_r30).xD54 != 0 {
            pl_80038824(arg0, 0x4D);
        }

        for var_r29 in 0..6 {
            if !Player_8003221C(var_r29) {
                continue;
            }

            if var_r29 == arg0 {
                (*temp_r30).xD50 = pl.xB0;
            } else {
                (*tbl(var_r29)).xD4C = pl.xA4;
            }
        }
    }
}

pub fn pl_800401F0(arg0: i32, arg1: i32, arg2: f32, arg3: f32) {
    melee_core::ct!("pl_800401F0");
    let temp_r3 = tbl(arg0);
    unsafe {
        (*temp_r3).x0_staleMoveTable.xCDC += abs_c(arg2);
        (*temp_r3).x0_staleMoveTable.xCE0 += abs_c(arg3);
    }
}

pub fn pl_80040270(arg0: i32, arg1: i32, arg2: f32) {
    melee_core::ct!("pl_80040270");
    let temp_r3 = tbl(arg0);
    if arg1 != 0 {
        return;
    }

    if arg2 >= c().x98 {
        unsafe { (*temp_r3).x0_staleMoveTable.xCE4 = (*temp_r3).x0_staleMoveTable.xCE4.wrapping_add(1) };
    }
}

pub fn pl_800402D0(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_800402D0");
    let stale_moves = tbl(slot);
    if arg1 != 0 {
        return;
    }
    unsafe {
        (*stale_moves).x0_staleMoveTable.xCEC = (*stale_moves).x0_staleMoveTable.xCEC.wrapping_add(1);
        if arg2 == 0 {
            return;
        }
        (*stale_moves).x0_staleMoveTable.xCF0 = (*stale_moves).x0_staleMoveTable.xCF0.wrapping_add(1);
    }
}

pub fn pl_80040330(slot: i32, arg1: i32, arg2: f32) {
    melee_core::ct!("pl_80040330");
    if arg1 != 0 {
        return;
    }

    let temp_r3 = tbl(slot);
    unsafe {
        if arg2 < (*temp_r3).xCF8 {
            (*temp_r3).xCF8 = arg2;
        }
    }
}

pub fn pl_80040374(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040374");
    let temp_r3 = tbl(arg0);
    unsafe {
        if arg1 == 0 && (*temp_r3).xD64 != 6 {
            (*temp_r3).xD60 = c().x124;
        }
    }
}

pub fn pl_800403C0(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_800403C0");
    tbl(arg0);
    pl_80038788(arg0, 0xC5, 1);
}

pub fn pl_800403FC(arg0: i32, arg1: i32, slot: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_800403FC");
    let temp_r3 = tbl(slot);
    if arg3 == 0 && arg4 >= 0x54 && arg4 <= 0x57 {
        unsafe { (*temp_r3).x0_staleMoveTable.xCAC = arg0 as u32 };
    }
}

pub fn pl_80040460(slot: i32, arg1: i32) {
    melee_core::ct!("pl_80040460");
    // one port per C function: forwards to plbonus.rs
    crate::plbonus::pl_80040460(slot, arg1)
}

pub fn pl_8004049C(player: i32, arg1: ItemKind) {
    melee_core::ct!("pl_8004049C");
    let mut var_r30: i32 = -1;

    if player == Gm_Player_NumMax as i32 {
        return;
    }

    if !(0 <= player && player < Gm_Player_NumMax as i32) {
        panic!("src/melee/pl/plbonuslib.c:1559: zako ko player illegal ! :{}", player);
    }

    match arg1 {
        It_Kind_Old_Kuri | It_Kind_Kuriboh => var_r30 = 0xCE,
        It_Kind_Heiho => var_r30 = 0xD6,
        It_Kind_Nokonoko => var_r30 = 0xCF,
        It_Kind_Patapata => var_r30 = 0xD0,
        It_Kind_Likelike => var_r30 = 0xD2,
        It_Kind_Old_Lead | It_Kind_Leadead => var_r30 = 0xD1,
        It_Kind_Old_Octa | It_Kind_Octarock => var_r30 = 0xD3,
        It_Kind_Old_Otto | It_Kind_Ottosea => var_r30 = 0xD4,
        It_Kind_Whitebea => var_r30 = 0xD5,
        _ => {}
    }

    if var_r30 == -1 {
        return;
    }
    ext::gm_8016B6E8(player, ext::gm_8016F2F8(var_r30, player as u8));
    pl_80038824(player, var_r30);
}

pub fn pl_80040614(arg0: i32, arg1: bool, arg8: f32) {
    melee_core::ct!("pl_80040614");
    if arg8 == 0.0 {
        return;
    }

    if arg8 <= c().x4C {
        pl_80038788(arg0, 0x2D, 1);
    }
}

pub fn pl_8004065C(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8004065C");
    if arg1 != 0 {
        return;
    }
    pl_80038824(arg0, 0x78);
}

pub fn pl_80040688(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_80040688");
    unsafe {
        if ext::gm_GetFrameCount() <= c().xEC as u32 {
            pl_80038788(arg0, 0x5F, 1);
        }

        if arg2 != 0 && arg1 != 6 {
            let temp_r3 = tbl(arg1);

            let temp_r0: i32 = (*temp_r3).x0_staleMoveTable.xCB8;
            let bits = &mut (*temp_r3).x0_staleMoveTable.xCBC as *mut UnkPlBonusBits;
            let xcc0 = &mut (*temp_r3).x0_staleMoveTable.xCC0 as *mut ft_800898B4_t;

            if !unk_cond(arg1, temp_r0) {
                let temp_r0_2: u8 = (*bits).x3;
                if temp_r0_2 >= 0x33 && temp_r0_2 <= 0x3D {
                    pl_80038788(temp_r0, 0x2A, 1);
                }
                if (*xcc0).x10_b0() != 0 {
                    pl_80038788(temp_r0, 7, 1);
                    pl_80038898(temp_r0, 6);
                }
            }
        }
    }
}

pub fn pl_800407C8(arg0: i32) -> f32 {
    melee_core::ct!("pl_800407C8");
    unsafe {
        let temp_r3 = tbl(arg0);
        let temp_r5: u32 = (*temp_r3).x0_staleMoveTable.xCEC as u32;
        if temp_r5 != 0 {
            let tmp: f32 = (*temp_r3).x0_staleMoveTable.xCEC as u32 as f32;
            let temp_f31: f32 = (*temp_r3).x0_staleMoveTable.xCF0 as u32 as f32;
            return pl_CalculateAverage(temp_f31, tmp);
        }
    }
    0.0
}

pub fn pl_80040870(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040870");
    unsafe { (*tbl(arg0)).xCF8 }
}

pub fn pl_80040894(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040894");
    unsafe { (*tbl(arg0)).xD40 }
}

pub fn pl_800408B8(arg0: i32) -> u32 {
    melee_core::ct!("pl_800408B8");
    unsafe { (*tbl(arg0)).xD48 }
}

pub fn pl_800408DC(arg0: i32) -> i32 {
    melee_core::ct!("pl_800408DC");
    unsafe { (*tbl(arg0)).x0_staleMoveTable.xCB4 }
}

pub fn pl_80040900(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040900");
    unsafe { (*Player_GetActionStats(arg0)).anon_358.x358_hits.total as i32 }
}

pub fn pl_80040924(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040924");
    unsafe { (*Player_GetActionStats(arg0)).attacks.total as i32 }
}

pub fn pl_80040948(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040948");
    unsafe {
        let total: u32 = (*Player_GetActionStats(arg0)).attacks.total;
        let temp_r30: u32 = (*Player_GetActionStats(arg0)).anon_358.x358_hits.total;
        if total != 0 {
            return pl_CalculateAverage(temp_r30 as f32, total as f32);
        }
    }
    0.0
}

pub fn pl_80040A04(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040A04");
    unsafe {
        let temp_r3 = Player_GetActionStats(arg0);
        (*temp_r3).attacks.total.wrapping_sub((*temp_r3).attacks.aerials_count) as i32
    }
}

pub fn pl_80040A30(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040A30");
    unsafe { (*Player_GetActionStats(arg0)).attacks.aerials_count as i32 }
}

pub fn pl_80040A54(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040A54");
    unsafe { (*Player_GetActionStats(arg0)).attacks.thrown_item_count as i32 }
}

pub fn pl_80040A78(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040A78");
    unsafe { (*Player_GetActionStats(arg0)).attacks.by_attack_counts[StatsAttack_Catch as usize] as i32 }
}

pub fn pl_80040A9C(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040A9C");
    unsafe {
        let val = &(*Player_GetActionStats(arg0)).attacks.by_attack_counts;
        let mut idx = StatsAttack_DownAttackD as usize;
        let mut sum: i32 = 0;
        let mut cursor_i: u32 = 51;
        let mut ctr = 11;
        while ctr > 0 {
            if cursor_i - 51 > 1 {
                sum = sum.wrapping_add(val[idx + 1] as i32);
            }
            idx += 1;
            cursor_i += 1;
            ctr -= 1;
        }
        sum
    }
}

pub fn pl_80040AF0(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040AF0");
    pl_800386D8(Player_GetActionStats(arg0), 0x70)
}

pub fn pl_80040B18(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040B18");
    unsafe { (*Player_GetActionStats(arg0)).attacks.specials_count as i32 }
}

pub fn pl_80040B3C(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040B3C");
    unsafe { melee_runtime::cabi::f2u32((*tbl(arg0)).x0_staleMoveTable.xC6C as f64) }
}

pub fn pl_80040B64(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040B64");
    unsafe { melee_runtime::cabi::f2u32((*tbl(arg0)).x0_staleMoveTable.xC60 as f64) }
}

pub fn pl_80040B8C(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_80040B8C");
    let temp_r3 = tbl(slot);
    if arg1 != 0 {
        return;
    }
    unsafe { (*temp_r3).x0_staleMoveTable.xC68 = (*temp_r3).x0_staleMoveTable.xC68.wrapping_add(arg2) };
}

pub fn pl_80040BD8(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040BD8");
    unsafe { (*tbl(arg0)).x0_staleMoveTable.xC68 }
}

pub fn pl_80040BFC(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040BFC");
    unsafe { melee_runtime::cabi::f2u32((*tbl(arg0)).x0_staleMoveTable.xC64 as f64) }
}

pub fn pl_80040C24(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040C24");
    unsafe { (*tbl(arg0)).xD74 }
}

pub fn pl_80040C48(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040C48");
    unsafe { (*tbl(arg0)).xD78 }
}

pub fn pl_80040C6C(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040C6C");
    unsafe { (*tbl(arg0)).xD7C }
}

pub fn pl_80040C90(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040C90");
    unsafe { (*tbl(arg0)).xD80 }
}

pub fn pl_80040CB4(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040CB4");
    unsafe { (*tbl(arg0)).xD24 as i32 }
}

pub fn pl_80040CD8(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040CD8");
    unsafe { (*tbl(arg0)).xD20 as i32 }
}

pub fn pl_80040CFC(arg0: i32) -> u32 {
    melee_core::ct!("pl_80040CFC");
    unsafe { (*tbl(arg0)).xD04 }
}

pub fn pl_80040D20(arg0: i32) -> i32 {
    melee_core::ct!("pl_80040D20");
    unsafe { (*tbl(arg0)).xD68 }
}

pub fn pl_80040D44(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040D44");
    unsafe { (*tbl(arg0)).xD84 }
}

pub fn pl_80040D68(arg0: i32) -> f32 {
    melee_core::ct!("pl_80040D68");
    unsafe { (*tbl(arg0)).xD88 }
}

pub fn pl_80040D8C(slot: i32, arg1: i32) {
    melee_core::ct!("pl_80040D8C");
    let stale_moves = tbl(slot);
    unsafe {
        (*stale_moves).x0_staleMoveTable.xCB0 = (*stale_moves).x0_staleMoveTable.xCB0.wrapping_add(1);
    }
}

pub fn pl_80040DB8(slot: i32) -> i32 {
    melee_core::ct!("pl_80040DB8");
    unsafe { (*tbl(slot)).x0_staleMoveTable.xCB0 }
}

// ---- folded from plbonus.rs (consolidation steps 21/22) ----
/// plbonuslib.c static inline `unk_cond_fp`.
#[inline]
pub fn unk_cond_fp(arg0: i32, r23: i32) -> bool {
    melee_core::ct!("unk_cond_fp");
    r23 == 6 || r23 == arg0 || check_if_same_team(arg0, r23)
}

// ---- plbonuslib.c (pl_8003FBFC .. end of file) -------------------------------------------

/// `pl_8003FBFC`.
pub fn pl_8003fbfc(a: i32) -> i32 {
    melee_core::ct!("pl_8003fbfc");
    ext(a).xCFC
}

/// `pl_8003FC20`.
pub fn pl_8003fc20(a: i32) -> i32 {
    melee_core::ct!("pl_8003fc20");
    ext(a).xD00
}

/// `pl_8003FC44`.
pub fn pl_8003fc44(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003fc44");
    let t = ext(slot);
    if arg1 != 0 {
        return;
    }
    unsafe { t.xDD0.anon_0.set_bit0(1) };
}

/// `pl_8003FC88`.
pub fn pl_8003fc88(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003fc88");
    let s = &mut ext(arg0).x0_staleMoveTable;
    if arg1 == 1 {
        return;
    }
    let r0 = s.xC90;
    let hit = match r0 {
        0 => (1..=3).contains(&arg2),
        1 => (6..=8).contains(&arg2),
        2 => (9..=11).contains(&arg2),
        3 => (17..=48).contains(&arg2),
        _ => false,
    };
    if hit {
        if r0 == 3 {
            s.xC90 = 0;
            h::pl_80038824(arg0, 0x56);
        } else {
            s.xC90 = s.xC90.wrapping_add(1);
        }
    } else {
        s.xC90 = 0;
    }
}

/// `pl_8003FDA0`.
pub fn pl_8003fda0(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fda0");
    h::pl_80038788(arg0, 0x5B, 1);
}

/// `pl_8003FDC8`.
pub fn pl_8003fdc8(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fdc8");
    let t = ext(arg0);
    t.xD70 = t.xD70.wrapping_add(1);
}

/// `pl_8003FDF4`.
pub fn pl_8003fdf4(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fdf4");
    h::pl_80038788(arg0, 0xC4, 1);
}

/// `pl_8003FE1C`.
pub fn pl_8003fe1c(slot: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fe1c");
    h::pl_80038824(slot, 0xA);
}

/// `pl_8003FE40`.
pub fn pl_8003fe40(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fe40");
    h::pl_80038824(arg0, 0x4B);
}

/// `pl_8003FE64`.
pub fn pl_8003fe64(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_8003fe64");
    if none_has(0x1A) {
        h::pl_80038788(arg0, 0x1A, 1);
    }
}

/// `pl_8003FED0`.
pub fn pl_8003fed0(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003fed0");
    if arg1 != 0 {
        return;
    }
    if none_has(0x82) {
        h::pl_80038788(arg0, 0x82, 1);
    }
}

/// `pl_8003FF44` (the C evaluates ft_80087A8C only when the earlier tests pass).
pub fn pl_8003ff44(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003ff44");
    let t = ext(arg0);
    let g = get_entity_at_index(arg0, arg1);
    if arg1 == 0 && (1..=0x10).contains(&arg2) && h::ft_80087a8c(g) > k().x38 {
        unsafe { t.xDD1.anon_0.set_bit4(1) };
    }
}

/// `pl_8003FFDC`.
pub fn pl_8003ffdc(arg0: i32, _arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_8003ffdc");
    let g = get_entity_at_index(arg2, arg3);
    if arg0 != arg2 && arg4 != h::ft_80087a98(g) {
        h::pl_80038824(arg0, 0x46);
    }
}

/// `pl_80040048_fp`.
pub fn pl_80040048_fp(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040048_fp");
    let g = get_entity_at_index(arg0, arg1);
    let r3 = h::ftlib_get_last_attacker_slot(g);
    if arg1 != 0 || r3 == 6 {
        return;
    }
    let f31 = unsafe { (*h::ft_800898b4(g)).kb_applied1 };
    let t = ext(arg0);
    if f31 >= k().x98 {
        t.x0_staleMoveTable.xCE8 = t.x0_staleMoveTable.xCE8.wrapping_add(1);
    }
    if f31 >= k().xA0 && !h::ft_80087aa4(g) && h::ft_80087ab4(g) <= 1 {
        h::pl_80038824(arg0, 0x48);
    }
}

/// `pl_80040120_fp`.
pub fn pl_80040120_fp(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040120_fp");
    if arg1 == 1 {
        return;
    }
    if ext(arg0).xD4C != 0 {
        h::pl_80038824(arg0, 0x4A);
    }
    if ext(arg0).xD54 != 0 {
        h::pl_80038824(arg0, 0x4D);
    }
    for i in 0..6 {
        if !player_8003221c(i) {
            continue;
        }
        if i == arg0 {
            ext(arg0).xD50 = k().xB0;
        } else {
            ext(i).xD4C = k().xA4;
        }
    }
}

/// `pl_800401F0`.
pub fn pl_800401f0(arg0: i32, _arg1: i32, arg2: f32, arg3: f32) {
    melee_core::ct!("pl_800401f0");
    let s = &mut ext(arg0).x0_staleMoveTable;
    s.xCDC += abs_c(arg2);
    s.xCE0 += abs_c(arg3);
}

/// `pl_80040270_fp`.
pub fn pl_80040270_fp(arg0: i32, arg1: i32, arg2: f32) {
    melee_core::ct!("pl_80040270_fp");
    let s = &mut ext(arg0).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    if arg2 >= k().x98 {
        s.xCE4 = s.xCE4.wrapping_add(1);
    }
}

/// `pl_800402D0`.
pub fn pl_800402d0(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_800402d0");
    let s = &mut ext(slot).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    s.xCEC = s.xCEC.wrapping_add(1);
    if arg2 == 0 {
        return;
    }
    s.xCF0 = s.xCF0.wrapping_add(1);
}

/// `pl_80040330_fp`.
pub fn pl_80040330_fp(slot: i32, arg1: i32, arg2: f32) {
    melee_core::ct!("pl_80040330_fp");
    if arg1 != 0 {
        return;
    }
    let t = ext(slot);
    if arg2 < t.xCF8 {
        t.xCF8 = arg2;
    }
}

/// `pl_80040374_fp`.
pub fn pl_80040374_fp(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_80040374_fp");
    let t = ext(arg0);
    if arg1 == 0 && t.xD64 != 6 {
        t.xD60 = k().x124;
    }
}

/// `pl_800403C0`.
pub fn pl_800403c0(arg0: i32, _arg1: i32) {
    melee_core::ct!("pl_800403c0");
    h::pl_80038788(arg0, 0xC5, 1);
}

/// `pl_800403FC`.
pub fn pl_800403fc(arg0: i32, _arg1: i32, slot: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_800403fc");
    let t = ext(slot);
    if arg3 == 0 && (0x54..=0x57).contains(&arg4) {
        t.x0_staleMoveTable.xCAC = arg0 as u32;
    }
}

/// `pl_80040460_fp`.
pub fn pl_80040460_fp(slot: i32, arg1: i32) {
    melee_core::ct!("pl_80040460_fp");
    let t = ext(slot);
    if arg1 != 0 {
        return;
    }
    t.x0_staleMoveTable.xCAC = 6;
}

/// `pl_8004049C(player, item kind)`: KO of a stage enemy.
#[allow(non_upper_case_globals)]
pub fn pl_8004049c(player: i32, arg1: ItemKind) {
    melee_core::ct!("pl_8004049c");
    if player == 6 {
        return;
    }
    assert!((0..6).contains(&player), "zako ko player illegal ! :{player}");
    let v: i32 = match arg1 {
        It_Kind_Old_Kuri | It_Kind_Kuriboh => 0xCE,
        It_Kind_Heiho => 0xD6,
        It_Kind_Nokonoko => 0xCF,
        It_Kind_Patapata => 0xD0,
        It_Kind_Likelike => 0xD2,
        It_Kind_Old_Lead | It_Kind_Leadead => 0xD1,
        It_Kind_Old_Octa | It_Kind_Octarock => 0xD3,
        It_Kind_Old_Otto | It_Kind_Ottosea => 0xD4,
        It_Kind_Whitebea => 0xD5,
        _ => return,
    };
    let r = h::gm_8016f2f8(v, player as u8);
    h::gm_8016b6e8(player, r);
    h::pl_80038824(player, v);
}

/// `pl_80040614_fp` (`!arg8` is `arg8 == 0.0`).
pub fn pl_80040614_fp(arg0: i32, _arg1: bool, arg8: f32) {
    melee_core::ct!("pl_80040614_fp");
    if arg8 == 0.0 {
        return;
    }
    if arg8 <= k().x4C {
        h::pl_80038788(arg0, 0x2D, 1);
    }
}

/// `pl_8004065C`.
pub fn pl_8004065c(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8004065c");
    if arg1 != 0 {
        return;
    }
    h::pl_80038824(arg0, 0x78);
}

/// `pl_80040688_fp`.
pub fn pl_80040688_fp(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_80040688_fp");
    if h::gm_get_frame_count() <= k().xEC as u32 {
        h::pl_80038788(arg0, 0x5F, 1);
    }
    if arg2 != 0 && arg1 != 6 {
        let s = &ext(arg1).x0_staleMoveTable;
        let r0 = s.xCB8;
        let x3 = s.xCBC.x3;
        let b0 = s.xCC0.x10_b0();
        if !unk_cond_fp(arg1, r0) {
            if (0x33..=0x3D).contains(&x3) {
                h::pl_80038788(r0, 0x2A, 1);
            }
            if b0 != 0 {
                h::pl_80038788(r0, 7, 1);
                h::pl_80038898(r0, 6);
            }
        }
    }
}

/// `pl_800407C8`.
pub fn pl_800407c8(arg0: i32) -> f32 {
    melee_core::ct!("pl_800407c8");
    let s = &ext(arg0).x0_staleMoveTable;
    if s.xCEC != 0 {
        let tmp = s.xCEC as u32 as f32;
        let f31 = s.xCF0 as u32 as f32;
        return calculate_average(f31, tmp);
    }
    0.0
}

/// `pl_80040870_fp`.
pub fn pl_80040870_fp(a: i32) -> f32 {
    melee_core::ct!("pl_80040870_fp");
    ext(a).xCF8
}

/// `pl_80040894_fp`.
pub fn pl_80040894_fp(a: i32) -> u32 {
    melee_core::ct!("pl_80040894_fp");
    ext(a).xD40
}

/// `pl_800408B8`.
pub fn pl_800408b8(a: i32) -> u32 {
    melee_core::ct!("pl_800408b8");
    ext(a).xD48
}

/// `pl_800408DC`.
pub fn pl_800408dc(a: i32) -> i32 {
    melee_core::ct!("pl_800408dc");
    ext(a).x0_staleMoveTable.xCB4
}

/// `pl_80040900_fp`.
pub fn pl_80040900_fp(a: i32) -> i32 {
    melee_core::ct!("pl_80040900_fp");
    unsafe { action_stats(a).anon_358.x358_hits.total as i32 }
}

/// `pl_80040924_fp`.
pub fn pl_80040924_fp(a: i32) -> i32 {
    melee_core::ct!("pl_80040924_fp");
    action_stats(a).attacks.total as i32
}

/// `pl_80040948_fp`.
pub fn pl_80040948_fp(a: i32) -> f32 {
    melee_core::ct!("pl_80040948_fp");
    let total = action_stats(a).attacks.total;
    let r30 = unsafe { action_stats(a).anon_358.x358_hits.total };
    if total != 0 {
        return calculate_average(r30 as f32, total as f32);
    }
    0.0
}

/// `pl_80040A04`.
pub fn pl_80040a04(a: i32) -> i32 {
    melee_core::ct!("pl_80040a04");
    let s = action_stats(a);
    s.attacks.total.wrapping_sub(s.attacks.aerials_count) as i32
}

/// `pl_80040A30`.
pub fn pl_80040a30(a: i32) -> i32 {
    melee_core::ct!("pl_80040a30");
    action_stats(a).attacks.aerials_count as i32
}

/// `pl_80040A54`.
pub fn pl_80040a54(a: i32) -> i32 {
    melee_core::ct!("pl_80040a54");
    action_stats(a).attacks.thrown_item_count as i32
}

/// `pl_80040A78`.
pub fn pl_80040a78(a: i32) -> i32 {
    melee_core::ct!("pl_80040a78");
    action_stats(a).attacks.by_attack_counts[StatsAttack_Catch as usize] as i32
}

/// `pl_80040A9C`: the C loop sums `by_attack_counts[DownAttackD + 3 ..= DownAttackD + 11]`.
pub fn pl_80040a9c(a: i32) -> i32 {
    melee_core::ct!("pl_80040a9c");
    let c = &action_stats(a).attacks.by_attack_counts;
    let base = StatsAttack_DownAttackD as usize;
    let mut sum = 0i32;
    let mut cursor: u32 = 51;
    for k in 0..11usize {
        if cursor.wrapping_sub(51) > 1 {
            sum = sum.wrapping_add(c[base + k + 1] as i32);
        }
        cursor += 1;
    }
    sum
}

/// `pl_80040AF0`.
pub fn pl_80040af0(a: i32) -> u32 {
    melee_core::ct!("pl_80040af0");
    h::pl_800386d8(action_stats(a), 0x70)
}

/// `pl_80040B18`.
pub fn pl_80040b18(a: i32) -> i32 {
    melee_core::ct!("pl_80040b18");
    action_stats(a).attacks.specials_count as i32
}

/// `pl_80040B3C` (float -> u32, saturating).
pub fn pl_80040b3c(a: i32) -> u32 {
    melee_core::ct!("pl_80040b3c");
    melee_core::ppc::fcvt_u32(ext(a).x0_staleMoveTable.xC6C)
}

/// `pl_80040B64` (float -> u32, saturating).
pub fn pl_80040b64(a: i32) -> u32 {
    melee_core::ct!("pl_80040b64");
    melee_core::ppc::fcvt_u32(ext(a).x0_staleMoveTable.xC60)
}

/// `pl_80040B8C`.
pub fn pl_80040b8c(slot: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_80040b8c");
    let s = &mut ext(slot).x0_staleMoveTable;
    if arg1 != 0 {
        return;
    }
    s.xC68 = s.xC68.wrapping_add(arg2);
}

/// `pl_80040BD8`.
pub fn pl_80040bd8(a: i32) -> i32 {
    melee_core::ct!("pl_80040bd8");
    ext(a).x0_staleMoveTable.xC68
}

/// `pl_80040BFC` (float -> u32, saturating).
pub fn pl_80040bfc(a: i32) -> u32 {
    melee_core::ct!("pl_80040bfc");
    melee_core::ppc::fcvt_u32(ext(a).x0_staleMoveTable.xC64)
}

/// `pl_80040C24`.
pub fn pl_80040c24(a: i32) -> f32 {
    melee_core::ct!("pl_80040c24");
    ext(a).xD74
}

/// `pl_80040C48`.
pub fn pl_80040c48(a: i32) -> f32 {
    melee_core::ct!("pl_80040c48");
    ext(a).xD78
}

/// `pl_80040C6C`.
pub fn pl_80040c6c(a: i32) -> f32 {
    melee_core::ct!("pl_80040c6c");
    ext(a).xD7C
}

/// `pl_80040C90`.
pub fn pl_80040c90(a: i32) -> f32 {
    melee_core::ct!("pl_80040c90");
    ext(a).xD80
}

/// `pl_80040CB4`.
pub fn pl_80040cb4(a: i32) -> i32 {
    melee_core::ct!("pl_80040cb4");
    ext(a).xD24 as i32
}

/// `pl_80040CD8`.
pub fn pl_80040cd8(a: i32) -> i32 {
    melee_core::ct!("pl_80040cd8");
    ext(a).xD20 as i32
}

/// `pl_80040CFC`.
pub fn pl_80040cfc(a: i32) -> u32 {
    melee_core::ct!("pl_80040cfc");
    ext(a).xD04
}

/// `pl_80040D20`.
pub fn pl_80040d20(a: i32) -> i32 {
    melee_core::ct!("pl_80040d20");
    ext(a).xD68
}

/// `pl_80040D44`.
pub fn pl_80040d44(a: i32) -> f32 {
    melee_core::ct!("pl_80040d44");
    ext(a).xD84
}

/// `pl_80040D68`.
pub fn pl_80040d68(a: i32) -> f32 {
    melee_core::ct!("pl_80040d68");
    ext(a).xD88
}

/// `pl_80040D8C`.
pub fn pl_80040d8c(slot: i32, _arg1: i32) {
    melee_core::ct!("pl_80040d8c");
    let s = &mut ext(slot).x0_staleMoveTable;
    s.xCB0 = s.xCB0.wrapping_add(1);
}

/// `pl_80040DB8`.
pub fn pl_80040db8(slot: i32) -> i32 {
    melee_core::ct!("pl_80040db8");
    ext(slot).x0_staleMoveTable.xCB0
}

// ---- folded from plbonus2.rs (consolidation steps 21/22) ----
/// `pl_8003DF44`.
pub fn pl_8003df44(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003df44");
    let r3 = h::ftlib_get_last_attacker_slot(get_entity_at_index(arg0, arg1));
    if r3 == 6 || r3 == arg0 || check_if_same_team(arg0, r3) {
        return;
    }
    ext_plbonus2(r3).xD54 = h::pl_804d6470().xB4;
}

/// `pl_8003DFF4`.
pub fn pl_8003dff4(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003dff4");
    pl_8003fc88(arg0, arg1, arg2);
    pl_8003ff44(arg0, arg1, arg2);
    pl_8003eaac(arg0, arg1, arg2);
}

/// `pl_8003E058`.
pub fn pl_8003e058(arg0: i32, _arg1: i32, arg2: i32, _arg3: i32) {
    melee_core::ct!("pl_8003e058");
    h::pl_80038824(arg2, 0x3F);
    if arg0 == 6 || check_if_same_team(arg2, arg0) {
        return;
    }
    h::pl_80038824(arg0, 0x3E);
}

/// `pl_8003E0E8`.
pub fn pl_8003e0e8(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003e0e8");
    if arg1 != 0 {
        return;
    }
    h::pl_80038824(arg0, 0x40);
}

/// `pl_8003E114`.
pub fn pl_8003e114(arg0: i32, _arg1: i32, arg2: f32) {
    melee_core::ct!("pl_8003e114");
    if arg2 <= h::pl_804d6470().x84 {
        h::pl_80038788(arg0, 0x41, 1);
    }
}

/// `pl_8003E150`.
pub fn pl_8003e150(slot: i32, _arg1: i32) {
    melee_core::ct!("pl_8003e150");
    let s = &mut ext_plbonus2(slot).x0_staleMoveTable;
    s.xCD8 = s.xCD8.wrapping_add(1);
}

/// `pl_8003E17C`.
pub fn pl_8003e17c(player_id: i32, _arg1: i32, item: I) {
    melee_core::ct!("pl_8003e17c");
    let t = ext_plbonus2(player_id);
    let kind = h::it_get_kind(item);
    let kind2 = if kind >= It_Kind_Capsule as i32 && kind < It_Kind_L_Gun_Ray as i32 {
        kind
    } else {
        match kind {
            It_Kind_Lucky_Egg => It_Kind_L_Gun_Ray as i32,
            It_Kind_WhispyApple => It_Kind_StarRod_Star as i32,
            It_Kind_WhispyHealApple => It_Kind_LipStick_Spore as i32,
            It_Kind_Hammer_Head => It_Kind_S_Scope_Beam as i32,
            _ => -1,
        }
    };
    if kind2 != -1
        && (kind2 == It_Kind_L_Gun_Ray as i32
            || kind2 == It_Kind_Egg as i32
            || kind2 == It_Kind_S_Scope_Beam as i32
            || h::it_8026b7e8(item) == 1)
    {
        let s = &mut t.x0_staleMoveTable;
        if !h::it_8026b774(item, player_id as u8) {
            s.x674[kind2 as usize] = s.x674[kind2 as usize].wrapping_add(1);
        }
        for cnt in 0..6 {
            if h::it_8026b774(item, cnt) {
                return;
            }
        }
        s.x710[kind2 as usize] = s.x710[kind2 as usize].wrapping_add(1);
    }
}

/// `pl_8003E2CC` (HSD_ASSERT kind < Pl_ItemLog_Terminate -> panic).
pub fn pl_8003e2cc(arg0: i32, kind: u32) -> u32 {
    melee_core::ct!("pl_8003e2cc");
    assert!(kind < Pl_ItemLog_Terminate as u32, "plbonuslib.c:555");
    ext_plbonus2(arg0).x0_staleMoveTable.x674[kind as usize]
}

/// `pl_8003E334`.
pub fn pl_8003e334(arg0: i32, kind: u32) -> u32 {
    melee_core::ct!("pl_8003e334");
    assert!(kind < Pl_ItemLog_Terminate as u32, "plbonuslib.c:564");
    ext_plbonus2(arg0).x0_staleMoveTable.x710[kind as usize] as u32
}

/// `pl_8003E39C`.
pub fn pl_8003e39c(arg0: i32) -> i32 {
    melee_core::ct!("pl_8003e39c");
    let s = &ext_plbonus2(arg0).x0_staleMoveTable;
    s.x674.iter().fold(0i32, |a, &v| a.wrapping_add(v as i32))
}

/// `pl_8003E420`.
pub fn pl_8003e420(arg0: i32) -> u32 {
    melee_core::ct!("pl_8003e420");
    let s = &ext_plbonus2(arg0).x0_staleMoveTable;
    s.x710.iter().fold(0i32, |a, &v| a.wrapping_add(v)) as u32
}

/// plbonuslib.c static inline `match_item_kind_fp`.
#[inline]
pub fn match_item_kind_fp(kind: i32) -> i32 {
    melee_core::ct!("match_item_kind_fp");
    if kind >= It_Kind_Capsule as i32 && kind < It_Kind_L_Gun_Ray as i32 {
        return kind;
    }
    match kind {
        It_Kind_Lucky_Egg => Pl_ItemLog_Unk35 as i32,
        It_Kind_WhispyApple => Pl_ItemLog_Unk36 as i32,
        It_Kind_WhispyHealApple => Pl_ItemLog_Unk37 as i32,
        It_Kind_Hammer_Head => Pl_ItemLog_Unk38 as i32,
        _ => -1,
    }
}

/// `pl_8003E4A4`. C quirk not reproduced: an unknown kind gives -1, which passes the C's signed
/// `< Pl_ItemLog_Terminate` test and stores to `seen[-1]` (out of bounds on the stack); here it
/// is skipped. The oracle only drives kinds that map.
pub fn pl_8003e4a4(slot: i32, _arg1: bool, moves: &[i32]) {
    melee_core::ct!("pl_8003e4a4");
    let t = ext_plbonus2(slot);
    let mut seen = [0u32; Pl_ItemLog_Terminate as usize];
    for &m in moves {
        let k = match_item_kind_fp(m);
        if k >= 0 && k < Pl_ItemLog_Terminate as i32 {
            seen[k as usize] = 1;
        }
    }
    let lim = h::pl_804d6470().x138;
    for i in 0..Pl_ItemLog_Terminate as usize {
        let s = &mut t.x0_staleMoveTable;
        if seen[i] == 1 {
            s.x7AC[i] = s.x7AC[i].wrapping_add(1);
            if s.x7AC[i] as u32 == lim {
                h::pl_80038788(slot, 0x9D, 1);
                s.x7AC[i] = 0;
            }
        } else {
            s.x7AC[i] = 0;
        }
    }
}

/// `pl_8003E854`.
pub fn pl_8003e854_fp(arg0: i32, arg1: i32, arg2: I) {
    melee_core::ct!("pl_8003e854_fp");
    let r31 = get_entity_at_index(arg0, arg1);
    let r3 = h::it_get_owner(arg2);
    if r31 != r3 && h::it_8026b7bc(arg2) != 0 {
        h::pl_80038824(arg0, 0x9A);
        if h::it_get_kind(arg2) == It_Kind_Dosei as i32 {
            h::pl_80038788(arg0, 0xB6, 1);
        }
        if arg1 == 0 && h::ftlib_is_fighter(r3) && h::ftlib_is_sub_fighter(r3) == 0 {
            let id = h::ftlib_get_player_index(r3);
            ext_plbonus2(arg0).xD6C = id as i32;
        }
    }
    if h::it_get_kind(arg2) == It_Kind_BombHei as i32 && h::it_8026b960(arg2) <= h::pl_804d6470().x180 {
        pl_800419ac(arg0, arg1, h::it_8026b7cc(arg2) as u16);
    }
    if arg1 != 0 {
        return;
    }
    ext_plbonus2(arg0).x0_staleMoveTable.xCA8 = 0;
}

/// `pl_8003E978`.
pub fn pl_8003e978(slot: i32, b4: bool, _item: I) {
    melee_core::ct!("pl_8003e978");
    fn_8003e998(slot, b4 as i32);
}

/// `fn_8003E998`.
pub fn fn_8003e998(arg0: i32, index: i32) {
    melee_core::ct!("fn_8003e998");
    let t = ext_plbonus2(arg0);
    if index == 0 && t.xD6C != 6 {
        let g = get_entity_at_index(arg0, index);
        unsafe { (*h::ft_800898a8(g)).x207C.y = t.xD6C };
    }
}

/// `pl_8003EA08`.
pub fn pl_8003ea08(slot: i32, arg1: i32) {
    melee_core::ct!("pl_8003ea08");
    if arg1 != 0 {
        return;
    }
    ext_plbonus2(slot).xD6C = 6;
}

/// `pl_8003EA40`.
pub fn pl_8003ea40(arg0: i32, _arg1: i32, arg2: i32, arg3: i32, arg4: i32) {
    melee_core::ct!("pl_8003ea40");
    if arg3 != 0 || arg2 != arg4 {
        return;
    }
    h::pl_80038824(arg0, 0x9B);
}

/// `pl_8003EA74`.
pub fn pl_8003ea74(arg0: i32, arg1: i32) {
    melee_core::ct!("pl_8003ea74");
    if arg1 != 0 {
        return;
    }
    ext_plbonus2(arg0).xD44 = 0;
}

/// `pl_8003EAAC`.
pub fn pl_8003eaac(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("pl_8003eaac");
    let s = &mut ext_plbonus2(arg0).x0_staleMoveTable;
    if arg1 == 0 && arg2 == 0x59 {
        s.xCA8 = s.xCA8.wrapping_add(1);
        if s.xCA8 == h::it_8026c1d4() {
            h::pl_80038788(arg0, 0xC2, 1);
        }
    }
}

/// `pl_8003EB30`.
pub fn pl_8003eb30(arg0: f32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    melee_core::ct!("pl_8003eb30");
    if arg1 != 6 && arg1 != arg3 {
        {
            let s = &mut ext_plbonus2(arg1).x0_staleMoveTable;
            s.xC6C += arg0;
            if arg5 == 0x34 {
                s.xC70 += arg0;
            }
            if (arg5.wrapping_sub(0x5C) as u32) <= 1 {
                s.xC74 += arg0;
            }
        }
        if arg3 != 6 {
            if arg4 == 0 {
                let v = &mut ext_plbonus2(arg3).x0_staleMoveTable;
                v.xC78[arg1 as usize] += arg0;
            }
            pl_80040ed4(arg1, arg2, arg3, arg4, arg5, arg0);
        }
        ext_plbonus2(arg1).xD08 = 0;
    }
}

/// `pl_8003EC30`.
pub fn pl_8003ec30(slot: i32, arg1: i32, arg2: i32, arg3: f32) {
    melee_core::ct!("pl_8003ec30");
    if arg1 != 0 {
        return;
    }
    let s = &mut ext_plbonus2(slot).x0_staleMoveTable;
    if arg2 == 1 || arg2 == 3 {
        s.xC98 += arg3;
    }
}

/// `pl_8003EC9C`.
pub fn pl_8003ec9c(arg0: i32, arg1: i32, arg2: f32, arg3: f32) {
    melee_core::ct!("pl_8003ec9c");
    let t = ext_plbonus2(arg0);
    if arg1 != 0 {
        return;
    }
    t.x0_staleMoveTable.xC60 += arg3;
    if t.x0_staleMoveTable.xC64 < arg2 {
        t.x0_staleMoveTable.xC64 = arg2;
    }
    t.xD10 = 0;
}

/// `pl_8003ED0C`: `xCA0 * (n - 1) + arg2` is one fmadds in the shipping build.
pub fn pl_8003ed0c(arg0: i32, _arg1: i32, r5: i32, _arg3: i32, arg2: f32) {
    melee_core::ct!("pl_8003ed0c");
    if arg0 == r5 || check_if_same_team(r5, arg0) {
        return;
    }
    let s = &mut ext_plbonus2(arg0).x0_staleMoveTable;
    s.xCA4 = s.xCA4.wrapping_add(1);
    let n = s.xCA4;
    let f30 = fmadds(s.xCA0, n.wrapping_sub(1) as f32, arg2);
    s.xCA0 = calculate_average(f30, n as f32);
}
