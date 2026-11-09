
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
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[inline(always)]
pub(crate) fn ids(f: &Fighter) -> (i32, i32) {
    (f.player_idx as i32, f.is_sub_fighter() as i32)
}

#[inline(always)]
pub(crate) fn x594_b0(f: &Fighter) -> u8 {
    unsafe { f.x594.x0.x594_b0() }
}

/// `ftCommon_PcFloorMisses[6]` (static, TARGET_PC): never reset, as in the C.
pub(crate) static PC_FLOOR_MISSES: [AtomicU32; 6] = [const { AtomicU32::new(0) }; 6];

/// `ftCommon_PcFloorGuard(fp, valid, fall)`: false when the floor is valid; otherwise report
/// (every 120th miss per player), call `fall` and return true.
pub(crate) fn pc_floor_guard(f: &mut Fighter, valid: bool, fall: fn(G)) -> bool {
    if valid {
        return false;
    }
    let mut n = 0u32;
    if f.player_idx < 6 {
        n = PC_FLOOR_MISSES[f.player_idx as usize].fetch_add(1, Ordering::Relaxed);
    }
    if n % 120 == 0 {
        h::pc_floor_lost_report(f);
    }
    fall(f.gobj);
    true
}

/// `ftCommon_PcFloorValid`: the map exists, the floor line is in range and ft_80084A18 holds.
pub(crate) fn pc_floor_valid(f: &Fighter) -> bool {
    let map = h::mplib_8004d164_line_count();
    let line = f.coll_data.floor.index;
    match map {
        Some(count) => line >= 0 && line < count && h::ft_80084a18(f.gobj),
        None => false,
    }
}

/// `(a.x * a_dir + v.x) - (b_dir * b.x + p.x)`, both products fused in the shipping build.
#[inline(always)]
pub(crate) fn nudge_dx(a: &Vec2, a_dir: f32, v: &Vec3, b_dir: f32, b: &Vec2, p: &Vec3) -> f32 {
    fmadds(a.x, a_dir, v.x) - fmadds(b_dir, b.x, p.x)
}

#[inline(always)]
pub(crate) fn sgn_c(neg: bool, x: f32) -> f32 {
    if neg { -x } else { x }
}

#[inline(always)]
pub(crate) fn same_or_adjacent_floor(a: i32, b: i32) -> bool {
    a == b || b == h::mp_line_get_next(a) || b == h::mp_line_get_prev(a)
}

#[inline(always)]
pub(crate) fn bone4(f: &Fighter) -> *mut HSD_JObj {
    let i = h::ftparts_get_bone_index(f, 4);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// `ftCommon_SetAccessory`: panics (HSD_ASSERT, ftcommon.c:0x486) if one already exists.
pub fn set_accessory(f: &mut Fighter, joint: *mut HSD_Joint) {
    assert!(f.x20A0_accessory.is_null(), "fighter accessory already exsist! (ftcommon.c:1158)");
    f.x20A0_accessory = h::hsd_jobj_load_joint(joint);
}

pub(crate) const PARASOL_TABLE_2: [i32; 7] = [7, 8, 9, 10, 5, 6, 4];

pub(crate) const PARASOL_TABLE_3_SET: [bool; 7] = [false, false, false, false, true, false, true];

pub(crate) const PARASOL_TABLE_4: [i32; 7] = [-1, -1, -1, -1, 1, -1, 2];

pub(crate) const FT_PARASOL_NONE: i32 = -1;

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

#[inline(always)]
pub(crate) fn holds_sword(f: &Fighter) -> bool {
    !f.item_gobj.is_null()
        && unsafe { (*f.item_gobj).classifier } as i32 == HSD_GOBJ_CLASS_ITEM
        && h::it_get_kind(f.item_gobj) == It_Kind_Sword as i32
}

/// `_func_8007F948_inline`.
pub(crate) fn apply_head_item_cb(g: G) {
    let f = fp(g);
    if f.x197C.is_null() || f.x1980.is_null() {
        h::mex_on_apply_head_item(f.kind as i32, g);
    }
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

pub use crate::tu::ftcommon::ftcommon_8007d6a4;
pub use crate::tu::ftcommon::ftcommon_8007d7fc;
pub use crate::tu::ftcommon::ftcommon_8007d92c;
pub use crate::tu::ftcommon::ftcommon_8007d9d4;
pub use crate::tu::ftcommon::ftcommon_8007db24;
pub use crate::tu::ftcommon::ftcommon_8007db58;
pub use crate::tu::ftcommon::ftcommon_8007dd7c;
pub use crate::tu::ftcommon::ftcommon_8007dfd0;
pub use crate::tu::ftcommon::ftcommon_8007e0e4;
pub use crate::tu::ftcommon::ftcommon_8007e358;
pub use crate::tu::ftcommon::ftcommon_8007e3ec;
pub use crate::tu::ftcommon::ftcommon_8007e690;
pub use crate::tu::ftcommon::ftcommon_8007e6dc;
pub use crate::tu::ftcommon::ftcommon_8007e79c;
pub use crate::tu::ftcommon::ftcommon_8007e7e4;
pub use crate::tu::ftcommon::ftcommon_8007e82c;
pub use crate::tu::ftcommon::ftcommon_8007e83c;
pub use crate::tu::ftcommon::ftcommon_8007ea90;
pub use crate::tu::ftcommon::ftcommon_8007ec30;
pub use crate::tu::ftcommon::ftcommon_8007eec8;
pub use crate::tu::ftcommon::ftcommon_8007ef5c;
pub use crate::tu::ftcommon::ftcommon_8007f578;
pub use crate::tu::ftcommon::ftcommon_8007f5cc;
pub use crate::tu::ftcommon::ftcommon_8007f6a4;
pub use crate::tu::ftcommon::ftcommon_8007f824;
pub use crate::tu::ftcommon::ftcommon_8007f86c;
pub use crate::tu::ftcommon::ftcommon_8007f8e8;
pub use crate::tu::ftcommon::ftcommon_8007f948;
pub use crate::tu::ftcommon::ftcommon_8007f9b4;
pub use crate::tu::ftcommon::ftcommon_8007fa00;
pub use crate::tu::ftcommon::ftcommon_8007fa58;
pub use crate::tu::ftcommon::ftcommon_8007fc7c;
pub use crate::tu::ftcommon::ftcommon_8007fe84;
