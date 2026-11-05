//! Port of src/melee/pl/plbonus.c: the post-match bonus bookkeeping. `pl_StaleMoveTableExt_t`
//! (the per-player stale-move table plus the bonus counters, `x904[]` flags / point values) is
//! read and written through raw pointers (`Player_GetStaleMoveTableIndexPtr2`), as the C does.
//!
//! `pl_800386D8` / `fn_80038700` index past the `plActionStats` members they name (a `by_attack_hi`
//! index of 0x65.. lands in `x504..x594`), so they use byte-offset pointer reads. No fused sites
//! (tools/rust-port/fma_sites.py: 0 fmuladd). Callees of unported TUs are `ext` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use crate::plbonus3::{GM_DEC_TYPE_POINT, ext, hsd_assert as hsd_assert_plbonus3, set_point_value, verify_gm_8016aedc};
use core::ffi::c_void;

use super::player::*;
use super::plbonuslib::{pl_8003E2CC, pl_8003E334, pl_8003E39C, pl_8003E420, pl_8003E7D4};
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
    fn Camera_80030E10() -> f32;
    fn ft_80087988(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_80087988(a0 as _) };
    fn ft_800879D8(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_800879d8(a0 as _) };
    fn ft_800879F8(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0877::ft_800879f8(a0 as _) };
    fn ft_800898C0(a0: *mut HSD_GObj) -> bool => unsafe { crate::ft0892::ft_800898c0(a0 as _) };
    fn ftLib_8008732C(a0: *mut HSD_GObj) -> bool => unsafe { crate::ftlib::ftlib_8008732c(a0 as _) };
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32 => unsafe { crate::ftlib::get_motion_id(a0 as _) };
    fn gmDecisionGetType(a0: i32) -> u32;
    fn gm_8016AEEC() -> u32;
    fn gm_8016B0FC() -> bool;
    fn gm_8016B110() -> bool;
    fn gm_8016B1EC() -> bool;
    fn gm_8016B204() -> bool;
    fn gm_8016B3A0() -> bool;
    fn gm_8016B3D8() -> bool;
    fn gm_8016B558() -> i32;
    fn gm_801720B4() -> bool;
    fn gm_801720F8() -> bool;
    fn gm_80172140() -> i32;
    fn gm_GetFrameCount() -> u32;
    fn it_8026B7D8() -> i32;
    fn it_8026B7E0() -> i32;
    fn pl_80037B2C(a0: *mut plActionStats, a1: i32, a2: i32) -> i32 => unsafe { crate::pltrick::pl_80037b2c(&*(a0 as *const _), a1 as _, a2 as _) };
    fn pl_80038628(a0: *mut HSD_GObj, a1: i32) -> bool => unsafe { crate::pltrick::pl_80038628(a0 as _, a1 as _) };
    fn pl_800414C0(a0: i32) -> f32 => unsafe { crate::plbonus::pl_800414c0(a0 as _) };
    fn pl_80040DDC(a0: i32) => unsafe { let _ = crate::plbonus::pl_80040ddc(a0 as _); };
    fn pl_80041524(a0: i32) => unsafe { let _ = crate::plbonus::pl_80041524(a0 as _); };
    fn pl_8004182C(a0: i32) => unsafe { let _ = crate::plbonus::pl_8004182c(a0 as _); };
    fn pl_800418F4(a0: i32) => unsafe { let _ = crate::plbonus::pl_800418f4(a0 as _); };
    fn pl_80041BFC(a0: i32) => unsafe { let _ = crate::plbonus::pl_80041bfc(a0 as _); };
    fn plStale_ResetStaleMoveTableForPlayer(a0: i32) => { crate::tu::plstale::plStale_ResetStaleMoveTableForPlayer(a0) };
    }
}

const Gm_DecType_Flag: u32 = 0;
const Gm_DecType_Point: u32 = 1;

/// `HSD_ASSERT(line, cond)`
#[inline]
fn hsd_assert(cond: bool, line: u32, expr: &str) {
    if !cond {
        panic!("src/melee/pl/plbonus.c:{}: {}", line, expr);
    }
}

/// `pl_CalculateAverage` (plbonusinline.h)
#[inline]
fn pl_CalculateAverage(val: f32, total: f32) -> f32 {
    hsd_assert(total != 0.0, 16, "total != 0.0F");
    val / total
}

#[inline]
fn tbl(player: i32) -> *mut pl_StaleMoveTableExt_t {
    Player_GetStaleMoveTableIndexPtr2(player)
}

#[inline]
fn c() -> &'static pl_804D6470_t {
    unsafe { &*pl_804D6470 }
}

/// `pl_Verify_gm_8016AEDC` (static inline, pl/inlines.h)
#[inline]
fn pl_Verify_gm_8016AEDC() -> bool {
    ext::gm_GetFrameCount() != 0 && ext::gm_GetFrameCount() != (-2i32) as u32
}

/// `setFlag` (static): sets a bonus flag boolean to true
fn setFlag(player: i32, kind: i32) {
    let table = tbl(player);
    hsd_assert(player != Gm_Player_Other as i32, 0x38, "player != Gm_Player_Other");
    hsd_assert(ext::gmDecisionGetType(kind) == Gm_DecType_Flag, 0x39, "gmDecisionGetType(kind) == Gm_DecType_Flag");
    unsafe { (*table).x0_staleMoveTable.x904[kind as usize] = 1 };
}

/// `setPointValue` (static inline): sets a bonus value to a specified number of points
fn setPointValue(player: i32, kind: i32, val: u32) {
    let table = tbl(player);
    hsd_assert(player != Gm_Player_Other as i32, 0x50, "player != Gm_Player_Other");
    hsd_assert(ext::gmDecisionGetType(kind) == Gm_DecType_Point, 0x51, "gmDecisionGetType(kind) == Gm_DecType_Point");
    unsafe { (*table).x0_staleMoveTable.x904[kind as usize] = val };
}

pub fn pl_80038788(player: i32, kind: i32, arg2: i32) {
    melee_core::ct!("pl_80038788");
    setPointValue(player, kind, arg2 as u32);
}

pub fn pl_80038824(player: i32, kind: i32) {
    melee_core::ct!("pl_80038824");
    let temp_r31 = tbl(player);
    hsd_assert(ext::gmDecisionGetType(kind) == Gm_DecType_Point, 92, "gmDecisionGetType(kind) == Gm_DecType_Point");
    unsafe {
        let p = &mut (*temp_r31).x0_staleMoveTable.x904[kind as usize];
        *p = p.wrapping_add(1);
    }
}

pub fn pl_80038898(player: i32, kind: i32) {
    melee_core::ct!("pl_80038898");
    let temp_r31 = tbl(player);
    hsd_assert(ext::gmDecisionGetType(kind) == Gm_DecType_Point, 103, "gmDecisionGetType(kind) == Gm_DecType_Point");
    unsafe {
        let current_value = (*temp_r31).x0_staleMoveTable.x904[kind as usize];
        if current_value == 0 {
            return;
        }
        (*temp_r31).x0_staleMoveTable.x904[kind as usize] = current_value - 1;
    }
}

pub fn pl_80038914() -> *mut pl_804D6470_t {
    melee_core::ct!("pl_80038914");
    unsafe { pl_804D6470 }
}

pub fn pl_8003891C(player: i32) {
    melee_core::ct!("pl_8003891C");
    let t = tbl(player);
    ext::plStale_ResetStaleMoveTableForPlayer(player);
    super::plattack::plAttack_8003759C(player as u32);
    ext::pl_80040DDC(player);
    ext::pl_80041524(player);
    ext::pl_800418F4(player);

    unsafe {
        for i in 0..Pl_ItemLog_Terminate as usize {
            (*t).x0_staleMoveTable.x674[i] = 0;
        }
        for i in 0..Pl_ItemLog_Terminate as usize {
            (*t).x0_staleMoveTable.x710[i] = 0;
        }
        for i in 0..Pl_ItemLog_Terminate as usize {
            (*t).x0_staleMoveTable.x7AC[i] = 0;
        }
        for i in 0..30 {
            (*t).x0_staleMoveTable.x848[i] = 0;
        }

        fn_80038FB8(player);
        let m = &mut (*t).x0_staleMoveTable;
        m.xC60 = 0.0;
        m.xC64 = 0.0;
        m.xC68 = 0;
        m.xC6C = 0.0;
        m.xC70 = 0.0;
        m.xC74 = 0.0;
        for i in 0..6 {
            m.xC78[i] = 0.0;
        }
        m.xC90 = 0;
        m.xC94 = 0;
        m.xC98 = 0.0;
        m.xC9C = 0.0;
        m.xCA0 = 0.0;
        m.xCA4 = 0;
        m.xCAC = 6;
        m.xCB4 = 0;

        // `volatile int temp_zero = 0;` stored through an int* over UnkPlBonusBits.x0..x3
        m.xCB8 = 6;
        core::ptr::write_volatile(&mut m.xCBC as *mut UnkPlBonusBits as *mut i32, 0);

        m.xCC0.x0 = 0;
        m.xCC0.x4 = 0;
        m.xCC0.kb_applied1 = 0.0;
        m.xCC0.xC = 6;
        m.xCC0.set_x10_b0(0);
        m.xCC0.set_x11_b3(0);
        m.xCC0.set_x10_b1(0);
        m.xCC0.set_x10_b2(0);
        m.xCC0.set_x10_b3(0);
        m.xCC0.set_x10_b4(0);
        m.xCC0.set_x10_b5(0);
        m.xCC0.set_x10_b6(0);
        m.xCC0.set_x10_b7(0);
        m.xCC0.set_x11_b0(0);
        m.xCC0.set_x11_b1(0);
        m.xCC0.set_x11_b2(0);
        m.xCC0.set_x11_b4(0);
        m.xCD4 = 0;
        m.xCD8 = 0;
        m.xCE0 = 0.0;
        m.xCDC = 0.0;
        m.xCE4 = 0;
        m.xCE8 = 0;
        m.xCEC = 0;
        m.xCF0 = 0;
        (*t).xCF4 = 0;
        (*t).xCF8 = 0.0;
        (*t).x0_staleMoveTable.xCA8 = 0;
        (*t).xD04 = 0;
        (*t).xD08 = 0;
        (*t).xD0C = 0;
        (*t).xD10 = 0;
        (*t).xD14 = 0;
        (*t).xD18 = 0;
        (*t).xD1C = 0;
        (*t).xD20 = 0;
        (*t).xD24 = 0;
        (*t).xD28 = 0;
        (*t).xD2C = 0;
        (*t).xD30 = 0;
        (*t).xD34 = 0;
        (*t).xD38 = 0;
        (*t).xD3C = 0;
        (*t).xD40 = 0;
        (*t).xD44 = 0;
        (*t).xD48 = 0;
        (*t).xD4C = 0;
        (*t).xD50 = 0;
        (*t).xD54 = 0;
        (*t).xDC0 = 0.0;
        (*t).xD58 = 0;
        (*t).xD5C = u32::MAX;
        (*t).xD60 = 0;
        (*t).xD64 = 6;
        (*t).xD68 = 0;
        (*t).xD6C = 6;
        (*t).xCFC = 0;
        (*t).xD00 = 0;
        (*t).xD70 = 0;
        (*t).xD74 = 0.0;
        (*t).xD78 = 0.0;
        (*t).xD7C = 0.0;
        (*t).xD80 = 0.0;
        (*t).xD84 = 0.0;
        (*t).xD88 = 0.0;
        (*t).xD94 = 0.0;
        (*t).xD98 = 0;
        (*t).xD8C = 0.0;
        (*t).xD90 = 0;
        (*t).xD9C = 0;
        (*t).xDA0 = 0;
        (*t).xDA4 = 0;
        (*t).xDA8 = 0;
        (*t).xDAC = 0;
        (*t).xDB4 = 0;
        (*t).xDBC = 0;
        (*t).xDC4 = 0;
        (*t).xDC8 = 0;
        (*t).xDCC = 0;
        (*t).xDD0.anon_0.set_bit0(0);
        (*t).xDD0.anon_0.set_bit1(0);
        (*t).xDD0.anon_0.set_bit2(0);
        (*t).xDD0.anon_0.set_bit3(0);
        (*t).xDD0.anon_0.set_bit4(0);
        (*t).xDD0.anon_0.set_bit5(0);
        (*t).xDD0.anon_0.set_bit6(0);
        (*t).xDD0.anon_0.set_bit7(0);
        (*t).xDD1.anon_0.set_bit0(0);
        (*t).xDD1.anon_0.set_bit1(0);
        (*t).xDD1.anon_0.set_bit2(0);
        (*t).xDD1.anon_0.set_bit3(0);
        (*t).xDB8 = 0;
        (*t).xDD1.anon_0.set_bit4(0);
        (*t).xDD1.anon_0.set_bit5(0);
        (*t).xDD1.anon_0.set_bit6(0);
        (*t).xDD1.anon_0.set_bit7(0);
    }
}

pub fn pl_80038F10(player: i32) {
    melee_core::ct!("pl_80038F10");
    unsafe {
        let temp_r3 = tbl(player);
        (*temp_r3).xD50 = 0;
        (*temp_r3).xD54 = 0;
        (*temp_r3).xD60 = 0;
        (*temp_r3).xD64 = 6;

        for i in 0..6 {
            let temp_r3_2 = tbl(i);

            if player == (*temp_r3_2).xD64 {
                (*temp_r3_2).xD60 = 0;
                (*temp_r3_2).xD64 = 6;
            }

            if player as u32 == (*temp_r3_2).x0_staleMoveTable.xCAC {
                (*temp_r3_2).x0_staleMoveTable.xCAC = 6;
            }

            if player == (*temp_r3_2).xD6C {
                (*temp_r3_2).xD6C = 6;
            }
        }
    }
}

pub fn fn_80038FB8(player: i32) {
    melee_core::ct!("fn_80038FB8");
    let temp_r3 = tbl(player);
    for var_ctr in 0..215 {
        unsafe { (*temp_r3).x0_staleMoveTable.x904[var_ctr] = 0 };
    }
}

pub fn pl_8003906C(player: i32, kind: i32, arg2: *mut u32, arg3: u32, arg4: u32, arg5: u32, arg6: *mut i32) -> i32 {
    melee_core::ct!("pl_8003906C");
    unsafe {
        if pl_Verify_gm_8016AEDC() {
            if ext::gm_GetFrameCount() <= arg3 {
                if arg5 >= arg4 {
                    if kind != -1 {
                        setPointValue(player, kind, 1);
                    } else {
                        *arg2 = 1;
                    }
                }

                if arg3 == ext::gm_GetFrameCount() {
                    *arg6 = arg5 as i32;
                    return 1;
                }
            } else if arg3 != 0 && <u32 as melee_core::ppc::Ppc>::prem(ext::gm_GetFrameCount(), arg3) == 0 {
                let var_r3: u32 = if kind != -1 { pl_80039418(player, kind) } else { *arg2 };

                if (var_r3 as i32 != 0) && (arg5.wrapping_sub(*arg6 as u32) < arg4) {
                    if kind != -1 {
                        setPointValue(player, kind, 0);
                    } else {
                        *arg2 = 0;
                    }
                }

                *arg6 = arg5 as i32;
                return 1;
            }
        }
    }
    0
}

pub fn pl_80039238(player: i32, kind: i32, arg2: *mut i32, arg3: u32, arg4: *mut f32, arg6: f32, arg7: f32) -> bool {
    melee_core::ct!("pl_80039238");
    unsafe {
        let var_r0 = ext::gm_GetFrameCount() != 0 && ext::gm_GetFrameCount() != (-2i32) as u32;
        if var_r0 {
            if ext::gm_GetFrameCount() <= arg3 {
                if arg7 >= arg6 {
                    if kind != -1 {
                        setPointValue(player, kind, 1);
                    } else {
                        *arg2 = 1;
                    }
                }
                if arg3 == ext::gm_GetFrameCount() {
                    *arg4 = arg7;
                    return true;
                }
            } else if arg3 != 0 && <u32 as melee_core::ppc::Ppc>::prem(ext::gm_GetFrameCount(), arg3) == 0 {
                let var_r3: i32 = if kind != -1 { pl_80039418(player, kind) as i32 } else { *arg2 };
                if var_r3 != 0 && arg7 - *arg4 < arg6 {
                    if kind != -1 {
                        setPointValue(player, kind, 0);
                    } else {
                        *arg2 = 0;
                    }
                }
                *arg4 = arg7;
                return true;
            }
        }
    }
    false
}

pub fn pl_80039418(player: i32, kind: i32) -> u32 {
    melee_core::ct!("pl_80039418");
    let table = tbl(player);
    unsafe { (*table).x0_staleMoveTable.x904[kind as usize] }
}

/// `resetBonuses` (static inline)
fn resetBonuses(player: i32) {
    let table = tbl(player);
    for i in 0..215i32 {
        if ext::gmDecisionGetType(i) == Gm_DecType_Flag {
            unsafe { (*table).x0_staleMoveTable.x904[i as usize] = 0 };
        }
    }
}

pub fn pl_80039450(player: i32) {
    melee_core::ct!("pl_80039450");
    let table = tbl(player);

    resetBonuses(player);

    fn_80039618(player);
    fn_8003B044(player);
    fn_8003B9A4(player);
    fn_8003BD60(player);
    fn_8003C340(player);
    fn_8003CC84(player);
    fn_8003D2EC(player);

    unsafe {
        if (*table).xDD0.anon_0.bit1() == 0 {
            setFlag(player, 0x4F);
        }

        if !ext::gm_8016B0FC() && (*table).xDD0.anon_0.bit2() == 0 {
            setFlag(player, 0x55);
        }

        if (*table).xDCC != 0 && (*table).xDD1.anon_0.bit5() != 0 {
            setFlag(player, 0x4E);
        }
    }
}

pub fn fn_80039618(player: i32) {
    melee_core::ct!("fn_80039618");
    let table = tbl(player);
    let stats_p = Player_GetActionStats(player);
    let gobj = Player_GetEntity(player);
    unsafe {
        let stats = &mut *stats_p;
        let aerials: u32 = stats.hits.aerials_count;
        let atk_thrown: u32 = stats.attacks.thrown_item_count;
        let atk_counts: [u32; 100] = stats.attacks.by_attack_counts;
        let atk_x1A8: u32 = stats.attacks.x1A8;
        let hit_counts: [u32; 100] = stats.hits.by_attack_counts;
        let hit_thrown: u32 = stats.hits.thrown_item_count;
        let x358_counts: [u32; 100] = stats.anon_358.x358_hits.by_attack_counts;
        let hit_specials: u32 = stats.hits.specials_count;
        let hit_x1A0: u32 = stats.hits.x1A0_count;
        let hit_x1A8: u32 = stats.hits.x1A8;
        let x358_x1A8: u32 = stats.anon_358.x358_hits.x1A8;
        let attacks_total: u32 = stats.attacks.total;
        let hits_total: u32 = stats.hits.total;
        let x358_total: u32 = stats.anon_358.x358_hits.total;
        let pl = c();

        if aerials != 0 && aerials == hits_total {
            setFlag(player, 0);
        }
        if hit_specials != 0 && hit_specials == hits_total {
            setFlag(player, 0x21);
        }
        if hit_x1A0 != 0 && hit_x1A0 == hits_total {
            setFlag(player, 0x22);
        }
        if hits_total != 0 {
            'skip_only_low_moves: {
                for i in 1..100usize {
                    if !((i >= 0x33 && i <= 0x3D) || hit_counts[i] == 0) {
                        break 'skip_only_low_moves;
                    }
                }
                setFlag(player, 0x24);
            }
        }
        {
            let mut sum: u32 = 0;
            let mut sub_sum: u32 = 0;
            for i in 0x40..=0x62usize {
                let count = hit_counts[i];
                sum = sum.wrapping_add(count);
                match i {
                    0x42 | 0x46 | 0x4A | 0x4E | 0x56 => sub_sum = sub_sum.wrapping_add(count),
                    _ => {}
                }
            }
            if sum != 0 {
                if hits_total == hit_counts[0x62] {
                    setFlag(player, 0x95);
                } else if hits_total == sub_sum {
                    setFlag(player, 0x96);
                } else if hits_total == sum {
                    setFlag(player, 0x94);
                }
            }
        }
        for j in 0..6 {
            let mut k = 1;
            while k <= 0x10 {
                if !(ext::pl_80037B2C(stats_p, j, k) != 0 || !ext::pl_80038628(gobj, k)) {
                    break;
                }
                k += 1;
            }
            if k == 0x11 {
                setFlag(player, 0x11);
                break;
            }
        }
        if (*Player_GetStaleMoveTableIndexPtr2(player)).x0_staleMoveTable.x904[0x11] == 0 {
            let mut found_low = 0;
            let mut found_high = 0;
            let mut k = 1;
            while k <= 0xB {
                if !(hit_counts[k as usize] != 0 || !ext::pl_80038628(gobj, k)) {
                    break;
                }
                k += 1;
            }
            if k == 0xC {
                found_low = 1;
            }
            k = 0xC;
            while k <= 0x10 {
                if !(hit_counts[k as usize] != 0 || !ext::pl_80038628(gobj, k)) {
                    break;
                }
                k += 1;
            }
            if k == 0x11 {
                found_high = 1;
            }
            if found_low != 0 && found_high != 0 {
                setFlag(player, 0x10);
            } else if found_low != 0 {
                setFlag(player, 0xE);
            } else if found_high != 0 {
                setFlag(player, 0xF);
            }
        }
        'skip_combo_count: {
            let mut p: usize = 1;
            let mut max: i32 = 0;
            let mut min: i32 = 0;
            let mut count: i32 = 0;
            let mut i = 99;
            while i != 0 {
                let cc = x358_counts[p];
                if cc != 0 {
                    count += 1;
                    if cc > max as u32 {
                        max = cc as i32;
                    }
                    if min == 0 || cc < min as u32 {
                        min = cc as i32;
                    }
                }
                if max != 0 && min != 0 && (max.wrapping_sub(min) as f32) >= (max as f32) * pl.x28 {
                    break 'skip_combo_count;
                }
                p += 1;
                i -= 1;
            }
            if count >= pl.x24 {
                setFlag(player, 0x12);
            }
        }
        if hits_total != 0 {
            if hit_thrown == hits_total {
                setFlag(player, 0x15);
            } else if pl_CalculateAverage(hit_thrown as f32, hits_total as f32) >= pl.x34 {
                setFlag(player, 0x14);
            }
        }
        if atk_thrown == 0 {
            setFlag(player, 0x16);
        }
        'skip_hit_count: {
            let mut hit_count = 0;
            for i in 1..100usize {
                if hit_counts[i] != 0 {
                    if i < 0x11 || i > 0x30 {
                        break 'skip_hit_count;
                    }
                    hit_count += 1;
                }
            }
            if hit_count != 0 {
                if hit_count == 1 {
                    setFlag(player, 0x18);
                } else {
                    setFlag(player, 0x17);
                }
            }
        }
        if hits_total != 0 {
            if pl_CalculateAverage(stats.x56C as f32, hits_total as f32) >= pl.x8 {
                setFlag(player, 3);
            }
        }
        if hits_total != 0 && stats.x574 == hits_total {
            setFlag(player, 5);
        } else if hits_total >= pl.xC {
            if pl_CalculateAverage(stats.x574 as f32, hits_total as f32) >= pl.x10 {
                setFlag(player, 4);
            }
        }
        if attacks_total == 0 && (*table).x0_staleMoveTable.xC60 == 0.0 {
            setFlag(player, 0x64);
        } else {
            if attacks_total == 0 {
                setFlag(player, 0x60);
            } else if (*table).xD04 >= pl.x54 {
                setFlag(player, 0x2F);
            } else if (*table).xD04 >= pl.x50 {
                setFlag(player, 0x2E);
            }
            if (*table).x0_staleMoveTable.xC60 == 0.0 {
                setFlag(player, 0x62);
            } else if (*table).xD0C >= pl.xF0 {
                setFlag(player, 0x61);
            }
        }
        {
            let falls: u32 = (Player_GetFalls(player).wrapping_add(ext::gm_80172140())) as u32;
            if falls == 0 {
                setFlag(player, 0x63);
            } else if falls == (*table).x0_staleMoveTable.xC94 as u32 {
                setFlag(player, 0x66);
            }
        }
        if (*table).xDD1.anon_0.bit0() != 0
            && (*table).xDD1.anon_0.bit1() != 0
            && (*table).xDD1.anon_0.bit2() != 0
            && (*table).xDD1.anon_0.bit3() != 0
        {
            setFlag(player, 0x86);
        }
        if atk_x1A8 != 0
            && atk_x1A8 == x358_x1A8
            && hit_x1A8 == (*Player_GetStaleMoveTableIndexPtr2(player)).x0_staleMoveTable.x904[6]
        {
            setFlag(player, 8);
        }
        if !ext::gm_801720B4() {
            let dmg: i32 = melee_core::ppc::fctiwz_f32((*table).x0_staleMoveTable.xC64);
            if (dmg as f32) >= 400.0 {
                setFlag(player, 0x20);
            } else if (dmg as f32) >= 350.0 {
                setFlag(player, 0x1F);
            } else if (dmg as f32) >= 300.0 {
                if !ext::gm_8016B3A0() {
                    setFlag(player, 0x1E);
                }
            } else if (dmg as f32) >= 250.0 {
                setFlag(player, 0x1D);
            } else if (dmg as f32) >= 200.0 {
                setFlag(player, 0x1C);
            } else if (dmg as f32) >= 150.0 {
                setFlag(player, 0x1B);
            }
        }
        if (*table).xDC8 as u32 != 0 {
            if x358_total == attacks_total {
                setFlag(player, 0xD);
            } else if pl_CalculateAverage(x358_total as f32, attacks_total as f32) >= pl.x20 {
                setFlag(player, 0xC);
            }
        }
        {
            let sum_attacks: u32 = fn_80038700(stats_p as *const i32, 1, 0x10) as u32;
            let sum_x358: u32 =
                fn_80038700((&stats.anon_358.x358_hits as *const plAttackStats) as *const i32, 1, 0x10) as u32;
            if sum_x358 != 0 && sum_attacks == sum_x358 && (*table).xDD1.anon_0.bit4() == 0 {
                setFlag(player, 0x23);
            }
        }
        if (*table).x0_staleMoveTable.xC74 >= pl.x160 {
            setFlag(player, 0xAA);
        }
        if ext::pl_800414C0(player) >= pl.x0 {
            setFlag(player, 1);
        }
        if ext::pl_80038628(gobj, 2) {
            let atk1 = atk_counts[1];
            if atk1 != 0 && atk1 == atk_counts[2] {
                let hit1 = x358_counts[1];
                if hit1 == x358_counts[2] && atk1 == hit1 {
                    if ext::pl_80038628(gobj, 3) {
                        let atk1b = atk_counts[1];
                        if atk1b == atk_counts[3] && atk1b == x358_counts[3] {
                            setFlag(player, 0x19);
                        }
                    } else {
                        setFlag(player, 0x19);
                    }
                }
            }
        }
        {
            let mut max_ratio: f32 = 0.0;
            let mut sum_ratio: f32 = 0.0;
            for i in 0..6 {
                if i != player {
                    let val = (*Player_GetStaleMoveTableIndexPtr2(i)).x0_staleMoveTable.xC78[player as usize];
                    if max_ratio < val {
                        max_ratio = val;
                    }
                    sum_ratio += val;
                }
            }
            if sum_ratio != 0.0 {
                if ext::gm_8016B558() == 3 && pl_CalculateAverage(max_ratio, sum_ratio) >= pl.xF8 {
                    setFlag(player, 0x67);
                } else if ext::gm_8016B558() >= 4 && pl_CalculateAverage(max_ratio, sum_ratio) >= pl.xFC {
                    setFlag(player, 0x67);
                }
            }
        }
        if ext::gm_8016B558() >= 3 {
            let mut streak = 0;
            for i in 0..6 {
                if i != player {
                    let kos: i32 = Player_GetKOsByPlayerIndex(player, i);
                    if kos != 0 {
                        if (kos as f32) >= pl.x100 && streak == 0 {
                            streak = 1;
                        } else {
                            streak = 0;
                            break;
                        }
                    }
                }
            }
            if streak != 0 {
                setFlag(player, 0x68);
            }
        }
    }
}

pub fn fn_8003B044(player: i32) {
    melee_core::ct!("fn_8003B044");
    unsafe {
        let temp_r31 = tbl(player);
        let temp_r3 = Player_GetActionStats(player);
        let pl = c();
        if (*temp_r3).x5BC_b0() == 0 {
            setFlag(player, 0x31);
        }
        if pl_800386D8(temp_r3, 0x65) == 0 {
            setFlag(player, 0x32);
        }
        if (*temp_r31).xD14 == 0 {
            setFlag(player, 0x33);
        }
        if pl_800386D8(temp_r3, 0x68) == 0 && pl_800386D8(temp_r3, 0x67) == 0 {
            setFlag(player, 0x35);
        }
        if (*temp_r3).x5BC_b2() == 0 {
            if (*temp_r3).x5BC_b3() != 0 {
                setFlag(player, 0x38);
            } else {
                setFlag(player, 0x3A);
            }
        }
        if (*temp_r31).xDD0.anon_0.bit0() == 0 && (*temp_r3).x5BC_b3() != 0 {
            setFlag(player, 0x39);
        }
        if pl_800386D8(temp_r3, 0x6C) != 0 && pl_800386D8(temp_r3, 0x6D) == 0 {
            setFlag(player, 0x51);
        }
        if pl_800386D8(temp_r3, 0x6C) == 0 && pl_800386D8(temp_r3, 0x6D) != 0 {
            setFlag(player, 0x52);
        }
        if pl_800386D8(temp_r3, 0x6C) == 0 && pl_800386D8(temp_r3, 0x6D) == 0 {
            setFlag(player, 0x50);
            if pl_800386D8(temp_r3, 0x6E) != 0 {
                setFlag(player, 0x53);
            }
        }
        if (*temp_r31).x0_staleMoveTable.xCE4 != 0 {
            if pl_CalculateAverage(
                (*temp_r31).x0_staleMoveTable.xCE8 as f32,
                (*temp_r31).x0_staleMoveTable.xCE4 as f32,
            ) >= pl.x9C
            {
                setFlag(player, 0x47);
            }
        }
        if (*temp_r31).x0_staleMoveTable.xC60 >= pl.xD0 {
            if pl_CalculateAverage((*temp_r31).x0_staleMoveTable.xC98, (*temp_r31).x0_staleMoveTable.xC60) >= pl.xCC {
                setFlag(player, 0x57);
            }
        }
        if !ext::gm_801720B4() && (*temp_r31).xD2C == 0 {
            setFlag(player, 0x6B);
        }
        if !ext::gm_8016B3A0() && ext::gm_8016B558() >= 3 {
            let mut var_f30: f32 = 0.0;
            let mut var_r27: i32 = 0;
            for i in 0..6 {
                if Player_8003221C(i) {
                    var_f30 += (*Player_GetStaleMoveTableIndexPtr2(i)).x0_staleMoveTable.xC9C;
                    var_r27 += 1;
                }
            }
            if var_f30 != 0.0 && var_r27 != 0 {
                let tmp: f32 = pl_CalculateAverage(var_f30, var_r27 as f32);
                if (*temp_r31).x0_staleMoveTable.xCA0 >= tmp * pl.xF4 {
                    setFlag(player, 0x65);
                }
            }
        }
        if ext::gm_GetFrameCount() >= pl.x108 as u32 {
            let tmp: f32 = 0.5 * ext::Camera_80030E10();
            if (*temp_r31).xD8C >= pl.x104 * tmp {
                setFlag(player, 0x69);
            }
        }
        if ext::gm_GetFrameCount() >= pl.x110 as u32 && (*temp_r31).xD8C < pl.x10C {
            setFlag(player, 0x6A);
        }
        {
            let tmp: f32 = 0.5 * ext::Camera_80030E10();
            if (*temp_r31).xD94 <= pl.x114 * tmp {
                setFlag(player, 0x6C);
            }
        }
        if (*temp_r31).xDD1.anon_0.bit6() != 0 && (*temp_r31).xDD1.anon_0.bit7() == 0 {
            setFlag(player, 0x5A);
        }
    }
}

pub fn fn_8003B9A4(player: i32) {
    melee_core::ct!("fn_8003B9A4");
    unsafe {
        let temp_r31 = tbl(player);
        let action_stats = Player_GetActionStats(player);
        let attack_counts = &(*action_stats).attacks.by_attack_counts;
        let hit_counts = &(*action_stats).hits.by_attack_counts;
        let pl = c();
        let catch = StatsAttack_Catch as usize;
        let catch_attack = StatsAttack_CatchAttack as usize;

        if attack_counts[catch] != 0 {
            if attack_counts[catch] >= pl.x3C && hit_counts[catch] == 0 {
                setFlag(player, 0x26);
            } else if pl_CalculateAverage(hit_counts[catch] as f32, attack_counts[catch] as f32) <= pl.x44 {
                setFlag(player, 0x25);
            }
        }
        if hit_counts[catch] != 0 {
            let mut i = catch;
            while i < StatsAttack_LedgeAttackSlow as usize {
                if !(i == catch || i == catch_attack || attack_counts[i] == 0) {
                    break;
                }
                i += 1;
            }
            if i == StatsAttack_LedgeAttackSlow as usize {
                if hit_counts[catch_attack] == 0 {
                    if hit_counts[catch] >= pl.x40 {
                        setFlag(player, 0x27);
                    }
                } else {
                    setFlag(player, 0x2C);
                }
            }
        }

        let mut i = StatsAttack_ThrowF as usize;
        while i < StatsAttack_CargoThrowF as usize {
            if attack_counts[i] == StatsAttack_None {
                break;
            }
            i += 1;
        }

        if i == StatsAttack_CargoThrowF as usize {
            setFlag(player, 0x28);
        }
        if (*temp_r31).x0_staleMoveTable.xC6C > 0.0 {
            if pl_CalculateAverage((*temp_r31).x0_staleMoveTable.xC70, (*temp_r31).x0_staleMoveTable.xC6C) >= pl.x48 {
                setFlag(player, 0x2B);
            }
        }
    }
}

pub fn fn_8003BD60(player: i32) {
    melee_core::ct!("fn_8003BD60");
    unsafe {
        let temp_r31 = tbl(player);
        let pl = c();
        if ext::gm_GetFrameCount() != 0 {
            let fc = ext::gm_GetFrameCount() as f32;
            if pl_CalculateAverage((*temp_r31).xD14 as f32, fc) >= pl.x60 {
                setFlag(player, 0x34);
            }
            let fc = ext::gm_GetFrameCount() as f32;
            if pl_CalculateAverage((*temp_r31).xD1C as f32, fc) >= pl.x68 {
                setFlag(player, 0x36);
            }
            let fc = ext::gm_GetFrameCount() as f32;
            if pl_CalculateAverage((*temp_r31).xD20 as f32, fc) >= pl.x6C {
                setFlag(player, 0x37);
            }
            if (*temp_r31).xD24 != 0 && pl_CalculateAverage((*temp_r31).xD28 as f32, (*temp_r31).xD24 as f32) >= pl.x5C {
                setFlag(player, 0x30);
            }
            if !ext::gm_801720B4() {
                let fc = ext::gm_GetFrameCount() as f32;
                if pl_CalculateAverage((*temp_r31).xD2C as f32, fc) >= pl.x78 {
                    setFlag(player, 0x3C);
                }
            }
            let fc = ext::gm_GetFrameCount() as f32;
            if pl_CalculateAverage((*temp_r31).xD30 as f32, fc) >= pl.xE8 {
                setFlag(player, 0x5D);
            }
        }
        if (*temp_r31).xDD0.anon_0.bit3() != 0
            && (*temp_r31).xDD0.anon_0.bit4() != 0
            && (*temp_r31).xDD0.anon_0.bit5() != 0
            && (*temp_r31).xDD0.anon_0.bit6() != 0
        {
            setFlag(player, 0x5E);
        }
        if (*temp_r31).xD38 >= pl.x164 {
            setFlag(player, 0xAB);
        }
        if (*temp_r31).xD3C >= pl.x16C {
            setFlag(player, 0xB4);
        }
    }
}

pub fn fn_8003C340(player: i32) {
    melee_core::ct!("fn_8003C340");
    let pl = c();
    let temp_r3: i32 = pl_8003E39C(player);
    if temp_r3 == 0 {
        setFlag(player, 0x93);
        return;
    }
    if temp_r3 as u32 == pl_8003E2CC(player, Pl_ItemLog_Unk07) {
        setFlag(player, 0xB3);
    }
    if pl_8003E420(player) as i32 == ext::it_8026B7D8() {
        setFlag(player, 0x9F);
    }

    let temp_r3_2: i32 = ext::it_8026B7E0();
    if temp_r3_2 != 0 && temp_r3_2 as u32 == pl_8003E334(player, Pl_ItemLog_Unk18) {
        setFlag(player, 0xA0);
    }

    let mut count: u32 = 0;
    for i in 0..Pl_ItemLog_Terminate {
        if i >= Pl_ItemLog_Unk06 && i != Pl_ItemLog_Unk18 && pl_8003E2CC(player, i) != 0 {
            count += 1;
        }
    }
    if count >= pl.x144 {
        setFlag(player, 0x9E);
    }

    let mut var_r24: u32 = 0;
    let mut var_r25: u32 = 0;
    let mut var_r26: u32 = 0;
    let mut var_r27: u32 = 0;
    let mut var_r28: u32 = 0;
    let mut var_r29: u32 = 0;
    let mut var_r30: u32 = 0;

    for i in 0..Pl_ItemLog_Terminate {
        let temp_r3_3: u32 = pl_8003E2CC(player, i);
        match i {
            Pl_ItemLog_Unk11 | Pl_ItemLog_Unk12 | Pl_ItemLog_Unk13 | Pl_ItemLog_Unk22 | Pl_ItemLog_Unk23
            | Pl_ItemLog_Unk24 | Pl_ItemLog_Unk28 => var_r30 = var_r30.wrapping_add(temp_r3_3),
            Pl_ItemLog_Unk16 | Pl_ItemLog_Unk21 | Pl_ItemLog_Unk25 => var_r29 = var_r29.wrapping_add(temp_r3_3),
            Pl_ItemLog_Unk08 | Pl_ItemLog_Unk09 | Pl_ItemLog_Unk35 | Pl_ItemLog_Unk37 => {
                var_r28 = var_r28.wrapping_add(temp_r3_3)
            }
            Pl_ItemLog_Unk18 => var_r27 = var_r27.wrapping_add(temp_r3_3),
            Pl_ItemLog_Unk26 | Pl_ItemLog_Unk27 | Pl_ItemLog_Unk31 | Pl_ItemLog_Unk32 | Pl_ItemLog_Unk33 => {
                var_r26 = var_r26.wrapping_add(temp_r3_3)
            }
            Pl_ItemLog_Unk00 | Pl_ItemLog_Unk01 | Pl_ItemLog_Unk02 | Pl_ItemLog_Unk03 | Pl_ItemLog_Unk04
            | Pl_ItemLog_Unk05 | Pl_ItemLog_Unk06 | Pl_ItemLog_Unk07 | Pl_ItemLog_Unk14 | Pl_ItemLog_Unk15
            | Pl_ItemLog_Unk17 | Pl_ItemLog_Unk19 | Pl_ItemLog_Unk20 | Pl_ItemLog_Unk30 | Pl_ItemLog_Unk34
            | Pl_ItemLog_Unk36 | Pl_ItemLog_Unk38 => var_r25 = var_r25.wrapping_add(temp_r3_3),
            Pl_ItemLog_Unk10 | Pl_ItemLog_Unk29 => var_r24 = var_r24.wrapping_add(temp_r3_3),
            _ => {}
        }
    }

    if var_r30 >= pl.x148 && var_r29 == 0 && var_r28 == 0 && var_r27 == 0 && var_r26 == 0 && var_r25 == 0 && var_r24 == 0 {
        setFlag(player, 0xA2);
    }
    if var_r30 == 0 && var_r29 >= pl.x14C && var_r28 == 0 && var_r27 == 0 && var_r26 == 0 && var_r25 == 0 && var_r24 == 0 {
        setFlag(player, 0xA3);
    }
    if var_r30 == 0 && var_r29 == 0 && var_r28 >= pl.x150 && var_r27 == 0 && var_r26 == 0 && var_r25 == 0 && var_r24 == 0 {
        setFlag(player, 0xA4);
    }
    if var_r30 == 0 && var_r29 == 0 && var_r28 == 0 && var_r27 >= pl.x154 && var_r26 == 0 && var_r25 == 0 && var_r24 == 0 {
        setFlag(player, 0xA1);
    }
    if var_r30 == 0 && var_r29 == 0 && var_r28 == 0 && var_r27 == 0 && var_r26 >= pl.x158 && var_r25 == 0 && var_r24 == 0 {
        setFlag(player, 0xA5);
    }
    if var_r30 == 0 && var_r29 == 0 && var_r28 == 0 && var_r27 == 0 && var_r26 == 0 && var_r25 >= pl.x15C && var_r24 == 0 {
        setFlag(player, 0xA6);
    }
    if pl_8003E2CC(player, Pl_ItemLog_Unk26).wrapping_add(pl_8003E2CC(player, Pl_ItemLog_Unk27)) >= pl.x168 {
        setFlag(player, 0xAE);
    }

    'skip: {
        for i in 0..Pl_ItemLog_Terminate {
            match i {
                Pl_ItemLog_Unk10 | Pl_ItemLog_Unk25 | Pl_ItemLog_Unk26 => {
                    if pl_8003E2CC(player, i) == 0 {
                        break 'skip;
                    }
                }
                _ => {
                    if pl_8003E2CC(player, i) != 0 {
                        break 'skip;
                    }
                }
            }
        }
        setFlag(player, 0xAF);
    }

    if pl_8003E2CC(player, Pl_ItemLog_Unk07) >= pl.x170 {
        setFlag(player, 0xB5);
    }
    if pl_8003E2CC(player, Pl_ItemLog_Unk09) >= pl.x174 {
        setFlag(player, 0xBC);
    }
    if pl_8003E2CC(player, Pl_ItemLog_Unk08) >= pl.x178 {
        setFlag(player, 0xBD);
    }
    if pl_8003E7D4(player, 0xB7) != 0 {
        setFlag(player, 0xCC);
    }
    if pl_8003E7D4(player, 0xB8) != 0 {
        setFlag(player, 0xCD);
    }
}

pub fn fn_8003CC84(player: i32) {
    melee_core::ct!("fn_8003CC84");
    unsafe {
        let temp_r31 = tbl(player);
        Player_GetActionStats(player);
        let temp_r29 = Player_GetEntity(player);
        if !ext::gm_8016B1EC() {
            ext::pl_8004182C(player);
            ext::pl_80041524(player);
            ext::pl_80041BFC(player);
            ext::pl_800418F4(player);
            if Player_8003221C(player) {
                match ext::ftLib_GetMotionId(temp_r29) {
                    0xC | 0xD => setFlag(player, 0x89),
                    0 | 1 | 2 => {
                        if ext::gm_8016B3D8() || !ext::gm_801720F8() {
                            setFlag(player, 0x8C);
                        }
                    }
                    4 => {
                        if ext::gm_8016B3D8() || !ext::gm_801720F8() {
                            setFlag(player, 0x8D);
                        }
                    }
                    _ => {}
                }
                if !ext::gm_8016B204() && !ext::gm_8016B110() {
                    match ext::gm_8016AEEC() {
                        0x0 => setFlag(player, 0x8F),
                        0x7 => setFlag(player, 0x8E),
                        0xD5 => setFlag(player, 0x90),
                        _ => {}
                    }
                }
                if !ext::ftLib_8008732C(temp_r29) {
                    if ext::ft_800898C0(temp_r29) {
                        setFlag(player, 0x8B);
                    }
                    if (*temp_r31).xDD0.xDD0_value & 1 != 0 {
                        setFlag(player, 0x8A);
                    }
                    if ext::ft_80087988(temp_r29) {
                        setFlag(player, 0xA7);
                    }
                    for i in 0..6 {
                        if player as u32 == (*Player_GetStaleMoveTableIndexPtr2(i)).x0_staleMoveTable.xCAC {
                            setFlag(player, 0xA8);
                        }
                    }
                    if ext::ft_800879D8(temp_r29) {
                        setFlag(player, 0xA9);
                    }
                    if ext::ft_800879F8(temp_r29) {
                        setFlag(player, 0xBE);
                    }
                    let temp_ret: i32 = Player_GetDamage(player);
                    if temp_ret == 0 {
                        setFlag(player, 0x92);
                    }
                    let temp_r5 = temp_ret / 100;
                    let temp_r3_3 = (temp_ret - (temp_r5 * 100)) / 10;
                    let temp_r0 = (temp_ret - (temp_r5 * 100)) - (temp_r3_3 * 10);
                    if (temp_r5 != 0 && temp_r5 == temp_r3_3 && temp_r3_3 == temp_r0)
                        || (temp_r5 == 0 && temp_r3_3 != 0 && temp_r3_3 == temp_r0)
                    {
                        setFlag(player, 0x91);
                    }
                }
            }
        }
    }
}

pub fn fn_8003D2EC(player: i32) {
    melee_core::ct!("fn_8003D2EC");
    unsafe {
        let temp_r3 = Player_GetActionStats(player);
        let temp_r27 = &(*temp_r3).attacks.by_attack_counts;
        let temp_r30: u32 = (*temp_r3).attacks.total;
        let pl = c();

        let x = |idx: usize| (*Player_GetStaleMoveTableIndexPtr2(player)).x0_staleMoveTable.x904[idx];
        if ext::gm_8016B0FC()
            || x(0) != 0
            || x(5) != 0
            || x(0x15) != 0
            || x(0x17) != 0
            || x(0x18) != 0
            || x(0x21) != 0
            || x(0x22) != 0
            || x(0x24) != 0
            || x(0x96) != 0
            || x(0x95) != 0
            || x(0x94) != 0
        {
            return;
        }

        if temp_r30 >= pl.xDC {
            for i in 1..100usize {
                if pl_CalculateAverage(temp_r27[i] as f32, temp_r30 as f32) >= pl.xD8 {
                    setFlag(player, 0x59);
                    break;
                }
            }
        }
    }
}

// ---- folded from plbonus3.rs (consolidation steps 21/22) ----

/// `fn_80038700_fp`: sum of `arg0[i + 1]` for i in arg1..=arg2.
pub fn fn_80038700_fp(arg0: &[i32], arg1: i32, arg2: i32) -> i32 {
    melee_core::ct!("fn_80038700_fp");
    let mut sum: i32 = 0;
    let mut i = arg1;
    while i <= arg2 {
        sum = sum.wrapping_add(arg0[(i + 1) as usize]);
        i += 1;
    }
    sum
}

/// `pl_80038788_fp`.
pub fn pl_80038788_fp(player: i32, kind: i32, arg2: i32) {
    melee_core::ct!("pl_80038788_fp");
    set_point_value(player, kind, arg2 as u32);
}

/// `pl_80038824_fp`.
pub fn pl_80038824_fp(player: i32, kind: i32) {
    melee_core::ct!("pl_80038824_fp");
    let t = ext(player);
    hsd_assert_plbonus3(92, h::gm_decision_get_type(kind) == GM_DEC_TYPE_POINT, "gmDecisionGetType(kind) == Gm_DecType_Point");
    let v = &mut t.x0_staleMoveTable.x904[kind as usize];
    *v = v.wrapping_add(1);
}

/// `pl_80038898_fp`.
pub fn pl_80038898_fp(player: i32, kind: i32) {
    melee_core::ct!("pl_80038898_fp");
    let t = ext(player);
    hsd_assert_plbonus3(103, h::gm_decision_get_type(kind) == GM_DEC_TYPE_POINT, "gmDecisionGetType(kind) == Gm_DecType_Point");
    let cur = t.x0_staleMoveTable.x904[kind as usize];
    if cur == 0 {
        return;
    }
    t.x0_staleMoveTable.x904[kind as usize] = cur - 1;
}

/// `pl_80038914_fp`: the `pl_804D6470` constants pointer.
pub fn pl_80038914_fp() -> *const pl_804D6470_t {
    melee_core::ct!("pl_80038914_fp");
    // player.c's `pl_804D6470` (set by the PlCo loader, `tu::player`); the hook only where that loader has not run (tests)
    let p = unsafe { crate::tu::player::pl_804D6470 } as *const pl_804D6470_t;
    if p.is_null() { unsafe { h::PL_804D6470 } } else { p }
}

/// `pl_80038F10`.
pub fn pl_80038f10(player: i32) {
    melee_core::ct!("pl_80038f10");
    let t = ext(player);
    t.xD50 = 0;
    t.xD54 = 0;
    t.xD60 = 0;
    t.xD64 = 6;
    for i in 0..6 {
        let o = ext(i);
        if player == o.xD64 {
            o.xD60 = 0;
            o.xD64 = 6;
        }
        if player as u32 == o.x0_staleMoveTable.xCAC {
            o.x0_staleMoveTable.xCAC = 6;
        }
        if player == o.xD6C {
            o.xD6C = 6;
        }
    }
}

/// `fn_80038FB8`.
pub fn fn_80038fb8(player: i32) {
    melee_core::ct!("fn_80038fb8");
    let t = ext(player);
    for v in t.x0_staleMoveTable.x904.iter_mut() {
        *v = 0;
    }
}

/// `pl_8003906C`. `arg2` is used when `kind == -1`.
#[allow(clippy::too_many_arguments)]
pub fn pl_8003906c(player: i32, kind: i32, arg2: &mut u32, arg3: u32, arg4: u32, arg5: u32, arg6: &mut i32) -> i32 {
    melee_core::ct!("pl_8003906c");
    if verify_gm_8016aedc() {
        if h::gm_get_frame_count() <= arg3 {
            if arg5 >= arg4 {
                if kind != -1 {
                    set_point_value(player, kind, 1);
                } else {
                    *arg2 = 1;
                }
            }
            if arg3 == h::gm_get_frame_count() {
                *arg6 = arg5 as i32;
                return 1;
            }
        } else if <u32 as melee_core::ppc::Ppc>::prem(h::gm_get_frame_count(), arg3) == 0 {
            let r3 = if kind != -1 { h::pl_80039418(player, kind) } else { *arg2 };
            if r3 as i32 != 0 && arg5.wrapping_sub(*arg6 as u32) < arg4 {
                if kind != -1 {
                    set_point_value(player, kind, 0);
                } else {
                    *arg2 = 0;
                }
            }
            *arg6 = arg5 as i32;
            return 1;
        }
    }
    0
}

/// `pl_80039238_fp`.
#[allow(clippy::too_many_arguments)]
pub fn pl_80039238_fp(player: i32, kind: i32, arg2: &mut i32, arg3: u32, arg4: &mut f32, arg6: f32, arg7: f32) -> bool {
    melee_core::ct!("pl_80039238_fp");
    let r0 = h::gm_get_frame_count() != 0 && h::gm_get_frame_count() != (-2i32) as u32;
    if r0 {
        if h::gm_get_frame_count() <= arg3 {
            if arg7 >= arg6 {
                if kind != -1 {
                    set_point_value(player, kind, 1);
                } else {
                    *arg2 = 1;
                }
            }
            if arg3 == h::gm_get_frame_count() {
                *arg4 = arg7;
                return true;
            }
        } else if <u32 as melee_core::ppc::Ppc>::prem(h::gm_get_frame_count(), arg3) == 0 {
            let r3 = if kind != -1 { h::pl_80039418(player, kind) as i32 } else { *arg2 };
            if r3 != 0 && arg7 - *arg4 < arg6 {
                if kind != -1 {
                    set_point_value(player, kind, 0);
                } else {
                    *arg2 = 0;
                }
            }
            *arg4 = arg7;
            return true;
        }
    }
    false
}

/// `pl_80039418_fp`.
pub fn pl_80039418_fp(player: i32, kind: i32) -> u32 {
    melee_core::ct!("pl_80039418_fp");
    ext(player).x0_staleMoveTable.x904[kind as usize]
}

// Split into src/melee/pl/pltrick.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::pltrick::{fn_80038700, pl_800386D8, pl_800386E8, pl_800386d8, pl_800386e8};
