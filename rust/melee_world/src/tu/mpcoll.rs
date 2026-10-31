//! Port of src/melee/mp/mpcoll.c: environment collision for fighters / items (the `CollData` update
//! loop: ECB sources, interpolation, floor / ceiling / wall passes, ledge grabs and pushout, the
//! per-substep callbacks). Generated from the C by `gen/mtu/c2rs.py` (see `mplib.rs`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// MSL `__fpclassifyf` / `__fpclassifyd` (math.h; FP_NAN 1, FP_INFINITE 2, FP_ZERO 3, FP_NORMAL 4, FP_SUBNORMAL 5)
fn __fpclassifyf(x: f32) -> i32 {
    let b = x.to_bits() as i32;
    match b & 0x7F800000 {
        0x7F800000 => if b & 0x007FFFFF != 0 { 1 } else { 2 },
        0 => if b & 0x007FFFFF != 0 { 5 } else { 3 },
        _ => 4,
    }
}

fn __fpclassifyd(x: f64) -> i32 {
    let hi = (x.to_bits() >> 32) as u32 as i32;
    let lo = x.to_bits() as u32;
    match hi & 0x7ff00000 {
        0x7ff00000 => if (hi & 0x000fffff) != 0 || lo != 0 { 1 } else { 2 },
        0 => if (hi & 0x000fffff) != 0 || lo != 0 { 5 } else { 3 },
        _ => 4,
    }
}

/// `lbVector_Add` (lb/lbvector.c; ported in melee_runtime)
unsafe fn lbVector_Add(a: *mut Vec, b: *mut Vec) -> *mut Vec {
    let mut x = melee_core::vec::Vec3 { x: (*a).x, y: (*a).y, z: (*a).z };
    melee_runtime::lbvector::add(&mut x, &melee_core::vec::Vec3 { x: (*b).x, y: (*b).y, z: (*b).z });
    (*a).x = x.x;
    (*a).y = x.y;
    (*a).z = x.z;
    a
}

/// `lbVector_Diff` (lb/lbvector.c; ported in melee_runtime): `*out = *a - *b`
pub unsafe fn lbVector_Diff(a: *mut Vec, b: *mut Vec, out: *mut Vec) -> *mut Vec {
    melee_core::ct!("lbVector_Diff");
    let r = melee_runtime::lbvector::diff(
        &melee_core::vec::Vec3 { x: (*a).x, y: (*a).y, z: (*a).z },
        &melee_core::vec::Vec3 { x: (*b).x, y: (*b).y, z: (*b).z },
    );
    (*out).x = r.x;
    (*out).y = r.y;
    (*out).z = r.z;
    out
}

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn ScriptGame_StageSlotSameJoint(a0: i32, a1: i32) -> i32;
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086960(a0 as _) as _ };
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086BE0(a0 as _) as _ };
    fn ftLib_800874BC(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_800874BC(a0 as _) as _ };
    fn grDynamicAttr_801CA284(a0: *mut Vec, a1: i32) -> i32 = |a0: *mut Vec, a1: i32| unsafe { crate::tu::grdynamicattr::grDynamicAttr_801CA284(a0 as _, a1 as _) as _ };
    fn itGetKind(a0: *mut HSD_GObj) -> ItemKind = |a0: *mut HSD_GObj| unsafe { crate::itacc::itGetKind(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn OSReport(a0: &str) = |a0: &str| melee_runtime::tu::os_error::OSReport_str(a0);
    }
    pub use melee_runtime::shared::DbLevel;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mpColl_80458810_t {
    pub right: [i32; 9],
    pub left: [i32; 9],
    pub normal: Vec3,
    pub x54_pad: [u8; 4],
}

melee_core::gc_layout!(mpColl_80458810_t, "mpColl_80458810_t", size 0x58, align 4, [
    right @ 0x0: 36 "int[9]", left @ 0x24: 36 "int[9]", normal @ 0x48: 12 "Vec3", x54_pad @ 0x54: 4 "u8[4]",
]);

pub static mut mpColl_80458810: mpColl_80458810_t = unsafe { core::mem::zeroed() };
pub static mut mpColl_804D6488: i32 = 0;
pub static mut mpColl_804D648C: i32 = 0;
pub static mut mpColl_804D6490_max_x: f32 = 0.0;
pub static mut mpColl_804D6494_line_id: i32 = 0;
pub static mut mpColl_804D6498_flags: u32 = 0;
pub static mut mpColl_IsEcbTiny: i32 = 0;
pub static mut mpColl_804D64A0: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32> = None;
pub static mut mpColl_804D64A4: *mut HSD_GObj = null_mut();
pub static mut mpColl_804D64A8: Option<unsafe extern "C" fn()> = None;
pub static mut mpColl_804D64AC: i32 = 0;
pub static mut mpColl_LedgeLevelOK: i32 = 0;

/// `mpColl_80041C78`
pub unsafe fn mpColl_80041C78() {
    melee_core::ct!("mpColl_80041C78");
    mpColl_804D64A0 = None;
    mpColl_804D64A4 = null_mut();
    mpColl_804D64A8 = None;
}

/// `mpCollPrev`
pub unsafe fn mpCollPrev(mut cd: *mut CollData) {
    melee_core::ct!("mpCollPrev");
    let mut __: [u8; 8] = [0; 8];
    mpColl_804D64A0 = None;
    mpColl_804D64A4 = null_mut();
    if ext::DbLevel >= ((DbLKind_DebugRom as i32) as u32) {
        if (((!((*cd).cur_pos.x < 4.5e+04f32)) || (!((*cd).cur_pos.x > (-4.5e+04f32)))) || (!((*cd).cur_pos.y < 4.5e+04f32))) || (!((*cd).cur_pos.y > (-4.5e+04f32))) {
            if ext::ftLib_80086960((*cd).x0_gobj) != 0 {
                ext::OSReport(&format!("{}:{}: Error: mpCollPrev() pos({:.6},{:.6}) player={} ms={}\n", core::ffi::CStr::from_ptr((c"mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 203, (*cd).cur_pos.x as f64, (*cd).cur_pos.y as f64, ext::ftLib_80086BE0((*cd).x0_gobj) as i32, ext::ftLib_800874BC((*cd).x0_gobj)));
            } else {
                ext::OSReport(&format!("{}:{}: Error: mpCollPrev() pos({:.6},{:.6}) gobj_id={}\n", core::ffi::CStr::from_ptr((c"mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 212, (*(*cd).x0_gobj).classifier as i32, (*cd).cur_pos.x as f64, (*cd).cur_pos.y as f64));
                if ((*(*cd).x0_gobj).p_link as i32) == 9 {
                    ext::OSReport(&format!("itkind={}\n", ext::itGetKind((*cd).x0_gobj)));
                }
            }
            if 0 != 0 { (); } else { panic!("mpCollPrev: assertion failed"); }
        }
    }
    (*cd).x28_vec = (*cd).cur_pos;
}

/// `clamp_above`
unsafe fn clamp_above(mut value: *mut f32, mut min: f32) {
    if (*value) < min {
        *value = min;
    }
}

/// `clamp_below`
unsafe fn clamp_below(mut value: *mut f32, mut max: f32) {
    if (*value) > max {
        *value = max;
    }
}

/// `mpCollCheckBounding`
pub unsafe fn mpCollCheckBounding(mut cd: *mut CollData, mut flags: u32) {
    melee_core::ct!("mpCollCheckBounding");
    let mut left: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut top: f32 = 0.0;
    left = (*cd).ecb.left.x + (*cd).cur_pos.x;
    clamp_below(core::ptr::addr_of_mut!(left) as *mut f32, (*cd).prev_ecb.left.x + (*cd).prev_pos.x);
    right = (*cd).ecb.right.x + (*cd).cur_pos.x;
    clamp_above(core::ptr::addr_of_mut!(right) as *mut f32, (*cd).prev_ecb.right.x + (*cd).prev_pos.x);
    bottom = (*cd).ecb.bottom.y + (*cd).cur_pos.y;
    clamp_below(core::ptr::addr_of_mut!(bottom) as *mut f32, (*cd).prev_ecb.bottom.y + (*cd).prev_pos.y);
    top = (*cd).ecb.top.y + (*cd).cur_pos.y;
    clamp_above(core::ptr::addr_of_mut!(top) as *mut f32, (*cd).prev_ecb.top.y + (*cd).prev_pos.y);
    if (flags & (4 as u32)) != 0 {
        let mut ledge_snap_x: f32 = (*cd).ledge_snap_x;
        let mut half_height: f32 = 0.5f32 * (*cd).ledge_snap_height;
        let mut offset: f32 = 0.0;
        right = right + ledge_snap_x;
        left = left - ledge_snap_x;
        clamp_below(core::ptr::addr_of_mut!(bottom) as *mut f32, ((*cd).ledge_snap_y - half_height) + (*cd).cur_pos.y);
        clamp_below(core::ptr::addr_of_mut!(bottom) as *mut f32, ((*cd).ledge_snap_y - half_height) + (*cd).prev_pos.y);
        offset = (*cd).ledge_snap_y + half_height;
        clamp_above(core::ptr::addr_of_mut!(top) as *mut f32, (*cd).cur_pos.y + offset);
        clamp_above(core::ptr::addr_of_mut!(top) as *mut f32, (*cd).prev_pos.y + offset);
    }
    super::mplib::mpBoundingCheck(left, bottom, right, top);
}

/// `mpColl_80041EE4`
pub unsafe fn mpColl_80041EE4(mut cd: *mut CollData) {
    melee_core::ct!("mpColl_80041EE4");
    (*cd).x0_gobj = null_mut();
    (*cd).x34_flags.set_b0((1 as u8) as u8);
    (*cd).x34_flags.set_b6((0 as u8) as u8);
    (*cd).x34_flags.set_b7((0 as u8) as u8);
    (*cd).x35_flags.set_b0((1 as u8) as u8);
    (*cd).x34_flags.set_b1234((0 as u8) as u8);
    (*cd).env_flags = 0;
    (*cd).x130_flags = 0 as u32;
    (*cd).prev_pos = (*cd).cur_pos;
    (*cd).last_pos = (*cd).cur_pos;
    (*cd).x28_vec = (*cd).cur_pos;
    (*cd).floor_skip = (-1_i32);
    (*cd).ledge_id_right = (-1_i32);
    (*cd).ledge_id_left = (-1_i32);
    (*cd).floor.index = (-1_i32);
    (*cd).floor.flags = 0 as u32;
    (*cd).floor.normal.x = 0.0f32;
    (*cd).floor.normal.y = 1.0f32;
    (*cd).floor.normal.z = 0.0f32;
    (*cd).ceiling.index = (-1_i32);
    (*cd).ceiling.flags = 0 as u32;
    (*cd).ceiling.normal.x = 0.0f32;
    (*cd).ceiling.normal.y = -1.0f32;
    (*cd).ceiling.normal.z = 0.0f32;
    (*cd).right_facing_wall.index = (-1_i32);
    (*cd).right_facing_wall.flags = 0 as u32;
    (*cd).right_facing_wall.normal.x = 0.0f32;
    (*cd).right_facing_wall.normal.y = 1.0f32;
    (*cd).right_facing_wall.normal.z = 0.0f32;
    (*cd).left_facing_wall.index = (-1_i32);
    (*cd).left_facing_wall.flags = 0 as u32;
    (*cd).left_facing_wall.normal.x = 0.0f32;
    (*cd).left_facing_wall.normal.y = -1.0f32;
    (*cd).left_facing_wall.normal.z = 0.0f32;
    (*cd).x38 = mpColl_804D64AC;
    (*cd).x50 = 0.0f32;
    (*cd).joint_id_skip = (-1_i32);
    (*cd).joint_id_only = (-1_i32);
    (*cd).ledge_snap_x = 0.0f32;
    (*cd).ledge_snap_y = 0.0f32;
    (*cd).ledge_snap_height = 0.0f32;
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).ecb) as *mut _ftECB) as *mut c_void) as *mut u8, 0, (32_u64 as i32) as usize);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).prev_ecb) as *mut _ftECB) as *mut c_void) as *mut u8, 0, (32_u64 as i32) as usize);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).xE4_ecb) as *mut _ftECB) as *mut c_void) as *mut u8, 0, (32_u64 as i32) as usize);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).ecb_source) as *mut ECBSource) as *mut c_void) as *mut u8, 0, (80_u64 as i32) as usize);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).desired_ecb) as *mut _ftECB) as *mut c_void) as *mut u8, 0, (32_u64 as i32) as usize);
    core::ptr::write_bytes(((core::ptr::addr_of_mut!((*cd).x64_ecb) as *mut _ftECB) as *mut c_void) as *mut u8, 0, (32_u64 as i32) as usize);
}

/// `mpColl_SetECBSource_JObj`
pub unsafe fn mpColl_SetECBSource_JObj(mut cd: *mut CollData, mut gobj: *mut HSD_GObj, mut arg1: *mut HSD_JObj, mut arg2: *mut HSD_JObj, mut arg3: *mut HSD_JObj, mut arg4: *mut HSD_JObj, mut arg5: *mut HSD_JObj, mut arg6: *mut HSD_JObj, mut arg7: *mut HSD_JObj, mut arg9: f32) {
    melee_core::ct!("mpColl_SetECBSource_JObj");
    (*cd).x0_gobj = gobj;
    (*cd).ecb_source.kind = (ECBSource_JObj as i32) as u32;
    (*cd).ecb_source.anon_4.anon_0.x108_joint = arg1;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[0 as usize] = arg2;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[1 as usize] = arg3;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[2 as usize] = arg4;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[3 as usize] = arg5;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[4 as usize] = arg6;
    (*cd).ecb_source.anon_4.anon_0.x10C_joint[5 as usize] = arg7;
    (*cd).ecb_source.x124 = arg9;
    (*cd).ecb_source.x128 = 1e+01f32;
    (*cd).ecb_source.x12C = 1e+01f32;
    if (*cd).x34_flags.b0() != 0 {
        (*cd).ecb.top.x = 0.0f32;
        (*cd).ecb.top.y = 8.0f32;
        (*cd).ecb.bottom.x = 0.0f32;
        (*cd).ecb.bottom.y = 0.0f32;
        (*cd).ecb.right.x = 4.0f32;
        (*cd).ecb.right.y = 4.0f32;
        (*cd).ecb.left.x = -4.0f32;
        (*cd).ecb.left.y = 4.0f32;
        (*cd).ecb = (*cd).ecb;
        (*cd).prev_ecb = (*cd).ecb;
        (*cd).xE4_ecb = (*cd).ecb;
        (*cd).x64_ecb = (*cd).ecb;
        (*cd).desired_ecb = (*cd).ecb;
    }
    (*cd).facing_dir = (-1_i32) as i16;
    (*cd).x50 = 0.0f32;
}

/// `mpColl_SetECBSource_Fixed`
pub unsafe fn mpColl_SetECBSource_Fixed(mut cd: *mut CollData, mut gobj: *mut HSD_GObj, mut arg1: f32, mut arg2: f32, mut arg3: f32, mut arg4: f32) {
    melee_core::ct!("mpColl_SetECBSource_Fixed");
    (*cd).x0_gobj = gobj;
    (*cd).ecb_source.kind = (ECBSource_Fixed as i32) as u32;
    (*cd).ecb_source.anon_4.anon_0_0.up = arg1;
    (*cd).ecb_source.anon_4.anon_0_0.down = arg2;
    (*cd).ecb_source.anon_4.anon_0_0.front = arg3;
    (*cd).ecb_source.anon_4.anon_0_0.back = arg4;
    (*cd).ecb_source.anon_4.anon_0_0.angle = 0.0f32;
    if (*cd).x34_flags.b0() != 0 {
        (*cd).ecb.top.x = 0.0f32;
        (*cd).ecb.top.y = 8.0f32;
        (*cd).ecb.bottom.x = 0.0f32;
        (*cd).ecb.bottom.y = 0.0f32;
        (*cd).ecb.right.x = 4.0f32;
        (*cd).ecb.right.y = 4.0f32;
        (*cd).ecb.left.x = -4.0f32;
        (*cd).ecb.left.y = 4.0f32;
        (*cd).prev_ecb = (*cd).ecb;
        (*cd).xE4_ecb = (*cd).ecb;
        (*cd).x64_ecb = (*cd).ecb;
        (*cd).desired_ecb = (*cd).ecb;
    }
    (*cd).facing_dir = (-1_i32) as i16;
}

/// `mpColl_SetLedgeSnap`
pub unsafe fn mpColl_SetLedgeSnap(mut coll: *mut CollData, mut ledge_snap_x: f32, mut ledge_snap_y: f32, mut ledge_snap_height: f32) {
    melee_core::ct!("mpColl_SetLedgeSnap");
    (*coll).ledge_snap_x = ledge_snap_x;
    (*coll).ledge_snap_y = ledge_snap_y;
    (*coll).ledge_snap_height = ledge_snap_height;
}

/// `mpColl_80042384`
pub unsafe fn mpColl_80042384(mut cd: *mut CollData) {
    melee_core::ct!("mpColl_80042384");
    if (if ((*cd).desired_ecb.top.y - (*cd).desired_ecb.bottom.y) < (0 as f32) { -((*cd).desired_ecb.top.y - (*cd).desired_ecb.bottom.y) } else { (*cd).desired_ecb.top.y - (*cd).desired_ecb.bottom.y }) < 1.0f32 {
        let mut mid: f32 = 0.0;
        (*cd).desired_ecb.top.y = (*cd).desired_ecb.top.y + 1.0f32;
        mid = 0.5f32 * ((*cd).desired_ecb.top.y + (*cd).desired_ecb.bottom.y);
        (*cd).desired_ecb.left.y = mid;
        (*cd).desired_ecb.right.y = mid;
    }
    clamp_above(core::ptr::addr_of_mut!((*cd).desired_ecb.top.y) as *mut f32, 1.0f32);
    clamp_below(core::ptr::addr_of_mut!((*cd).desired_ecb.left.x) as *mut f32, -1.0f32);
    clamp_above(core::ptr::addr_of_mut!((*cd).desired_ecb.right.x) as *mut f32, 1.0f32);
    if (*cd).desired_ecb.top.y < (*cd).desired_ecb.bottom.y {
        (*cd).desired_ecb.top.y = 1.0f32 + (*cd).desired_ecb.bottom.y;
    }
    if ((*cd).desired_ecb.right.y > (*cd).desired_ecb.top.y) || ((*cd).desired_ecb.right.y < (*cd).desired_ecb.bottom.y) {
        let mut mid: f32 = 0.5f32 * ((*cd).desired_ecb.top.y + (*cd).desired_ecb.bottom.y);
        (*cd).desired_ecb.left.y = mid;
        (*cd).desired_ecb.right.y = mid;
    }
    if (((*cd).desired_ecb.top.y - (*cd).desired_ecb.right.y) < 0.001f32) || (((*cd).desired_ecb.right.y - (*cd).desired_ecb.bottom.y) < 0.001f32) {
        (*cd).desired_ecb.right.y = 0.5f32 * ((*cd).desired_ecb.top.y + (*cd).desired_ecb.bottom.y);
    }
    if (((*cd).desired_ecb.top.y - (*cd).desired_ecb.left.y) < 0.001f32) || (((*cd).desired_ecb.left.y - (*cd).desired_ecb.bottom.y) < 0.001f32) {
        (*cd).desired_ecb.left.y = 0.5f32 * ((*cd).desired_ecb.top.y + (*cd).desired_ecb.bottom.y);
    }
}

/// `update_min_max`
unsafe fn update_min_max(mut min: *mut f32, mut max: *mut f32, mut val: f32) {
    if (*min) > val {
        *min = val;
    } else if (*max) < val {
        *max = val;
    }
}

/// `mpColl_LoadECB_JObj`
pub unsafe fn mpColl_LoadECB_JObj(mut coll: *mut CollData, mut flags: u32) {
    melee_core::ct!("mpColl_LoadECB_JObj");
    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
    let mut left_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut right_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut phi_f1: f32 = 0.0;
    let mut phi_f2: f32 = 0.0;
    let mut mid_y: f32 = 0.0;
    let mut tmpval: f32 = 0.0;
    if ((*coll).x130_flags & ((CollData_X130_Clear as i32) as u32)) != 0 {
        (*coll).ecb.top.x = 0.0f32;
        (*coll).ecb.top.y = 0.0f32;
        (*coll).ecb.bottom.x = 0.0f32;
        (*coll).ecb.bottom.y = 0.0f32;
        (*coll).ecb.right.x = 0.0f32;
        (*coll).ecb.right.y = 0.0f32;
        (*coll).ecb.left.x = 0.0f32;
        (*coll).ecb.left.y = 0.0f32;
        (*coll).x130_flags = (*coll).x130_flags & ((!(CollData_X130_Clear as i32)) as u32);
    }
    (*coll).xE4_ecb = (*coll).ecb;
    {
        let mut temp_x: f32 = (*coll).cur_pos.x;
        let mut temp_y: f32 = (*coll).cur_pos.y;
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[0 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        left_x = { right_x = vec.x - temp_x; right_x };
        bottom_y = { top_y = vec.y - temp_y; top_y };
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[1 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        dx = vec.x - temp_x;
        dy = vec.y - temp_y;
        update_min_max(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, dx);
        update_min_max(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, dy);
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[2 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        dx = vec.x - temp_x;
        dy = vec.y - temp_y;
        update_min_max(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, dx);
        update_min_max(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, dy);
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[3 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        dx = vec.x - temp_x;
        dy = vec.y - temp_y;
        update_min_max(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, dx);
        update_min_max(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, dy);
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[4 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        dx = vec.x - temp_x;
        dy = vec.y - temp_y;
        update_min_max(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, dx);
        update_min_max(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, dy);
        ext::lb_8000B1CC((*coll).ecb_source.anon_4.anon_0.x10C_joint[5 as usize], null_mut(), core::ptr::addr_of_mut!(vec) as *mut Vec);
        dx = vec.x - temp_x;
        dy = vec.y - temp_y;
        update_min_max(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, dx);
        update_min_max(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, dy);
    }
    if !((flags & (4 as u32)) != 0) {
        left_x = left_x - 2.0f32;
        right_x = right_x + 2.0f32;
        bottom_y = bottom_y - 2.0f32;
        top_y = top_y + 2.0f32;
    }
    phi_f1 = if 4.0f32 > (*coll).ecb_source.x12C { 4.0f32 } else { (*coll).ecb_source.x12C };
    phi_f2 = if (right_x - left_x) < (0 as f32) { -(right_x - left_x) } else { right_x - left_x };
    if phi_f2 < phi_f1 {
        right_x = 0.5f32 * phi_f2;
        left_x = -right_x;
    }
    phi_f1 = if 4.0f32 > (*coll).ecb_source.x128 { 4.0f32 } else { (*coll).ecb_source.x128 };
    phi_f2 = if (top_y - bottom_y) < (0 as f32) { -(top_y - bottom_y) } else { top_y - bottom_y };
    if phi_f2 < phi_f1 {
        tmpval = 0.5f32 * phi_f2;
        mid_y = 0.5f32 * (top_y + bottom_y);
        top_y = mid_y + tmpval;
        bottom_y = mid_y - tmpval;
    }
    if (flags & (8 as u32)) != 0 {
        left_x = -1.0f32;
        right_x = 1.0f32;
    } else {
        right_x = if right_x < 2.0f32 { 2.0f32 } else { right_x };
        left_x = if left_x > (-2.0f32) { -2.0f32 } else { left_x };
    }
    if (flags & (1 as u32)) != 0 {
        bottom_y = 0.0f32;
        if (flags & (16 as u32)) != 0 {
            top_y = 2.0f32;
        }
    } else {
        if bottom_y < 0.0f32 {
            bottom_y = 0.0f32;
        }
        if (flags & (16 as u32)) != 0 {
            mid_y = 0.5f32 * (bottom_y + top_y);
            bottom_y = mid_y - 1.0f32;
            top_y = mid_y + 1.0f32;
            if bottom_y < 0.0f32 {
                bottom_y = 0.0f32;
                top_y = 2.0f32;
            }
        }
    }
    (*coll).desired_ecb.top.x = 0.0f32;
    (*coll).desired_ecb.top.y = top_y;
    (*coll).desired_ecb.bottom.x = 0.0f32;
    (*coll).desired_ecb.bottom.y = bottom_y;
    (*coll).desired_ecb.right.x = right_x;
    {
        let mut half: f32 = 0.5f32 * (bottom_y + top_y);
        (*coll).desired_ecb.right.y = (*coll).ecb_source.x124 + half;
        (*coll).desired_ecb.left.x = left_x;
        (*coll).desired_ecb.left.y = (*coll).ecb_source.x124 + half;
    }
    (*coll).x34_flags.set_b0((0 as u8) as u8);
}

/// `update_min_max_2`
unsafe fn update_min_max_2(mut min: *mut f32, mut max: *mut f32, mut val: f32) {
    if (*max) < val {
        *max = val;
    } else if (*min) > val {
        *min = val;
    }
}

/// `clamp_above_2`
unsafe fn clamp_above_2(mut value: *mut f32, mut min: f32) {
    if (*value) < min {
        *value = min;
    }
}

/// `clamp_below_2`
unsafe fn clamp_below_2(mut value: *mut f32, mut max: f32) {
    if (*value) > max {
        *value = max;
    }
}

/// `mpColl_LoadECB_Fixed`
pub unsafe fn mpColl_LoadECB_Fixed(mut coll: *mut CollData) {
    melee_core::ct!("mpColl_LoadECB_Fixed");
    let mut angle: f32 = 0.0;
    let mut sin: f32 = 0.0;
    let mut cos: f32 = 0.0;
    let mut midpoint_x: f32 = 0.0;
    let mut midpoint_y: f32 = 0.0;
    let mut orig_top_y: f32 = 0.0;
    let mut orig_bottom_y: f32 = 0.0;
    let mut orig_right_x: f32 = 0.0;
    let mut orig_left_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut right_x: f32 = 0.0;
    let mut left_x: f32 = 0.0;
    let mut rot_top_x: f32 = 0.0;
    let mut rot_top_y: f32 = 0.0;
    let mut rot_bot_x: f32 = 0.0;
    let mut rot_bot_y: f32 = 0.0;
    let mut rot_right_x: f32 = 0.0;
    let mut rot_right_y: f32 = 0.0;
    let mut rot_left_y: f32 = 0.0;
    let mut rot_left_x: f32 = 0.0;
    angle = (*coll).ecb_source.anon_4.anon_0_0.angle;
    if ((*coll).x130_flags & ((CollData_X130_Clear as i32) as u32)) != 0 {
        (*coll).ecb.top.x = 0.0f32;
        (*coll).ecb.top.y = 0.0f32;
        (*coll).ecb.bottom.x = 0.0f32;
        (*coll).ecb.bottom.y = 0.0f32;
        (*coll).ecb.right.x = 0.0f32;
        (*coll).ecb.right.y = 0.0f32;
        (*coll).ecb.left.x = 0.0f32;
        (*coll).ecb.left.y = 0.0f32;
        (*coll).x130_flags = (*coll).x130_flags & ((!(CollData_X130_Clear as i32)) as u32);
    }
    (*coll).xE4_ecb = (*coll).ecb;
    bottom_y = -(*coll).ecb_source.anon_4.anon_0_0.down;
    top_y = (*coll).ecb_source.anon_4.anon_0_0.up;
    if ((*coll).facing_dir as i32) == 1 {
        right_x = (*coll).ecb_source.anon_4.anon_0_0.front;
        left_x = -(*coll).ecb_source.anon_4.anon_0_0.back;
    } else {
        right_x = (*coll).ecb_source.anon_4.anon_0_0.back;
        left_x = -(*coll).ecb_source.anon_4.anon_0_0.front;
    }
    if angle != 0.0f32 {
        sin = melee_runtime::gekko::sinf(angle);
        cos = melee_runtime::gekko::cosf(angle);
        orig_top_y = top_y;
        orig_bottom_y = bottom_y;
        orig_right_x = right_x;
        orig_left_x = left_x;
        rot_right_y = 0.5f32 * (orig_right_x + orig_left_x);
        midpoint_x = rot_right_y;
        top_y = { right_x = { bottom_y = { left_x = 0.0f32; left_x }; bottom_y }; right_x };
        rot_top_x = (-orig_top_y) * sin;
        rot_top_y = orig_top_y * cos;
        update_min_max_2(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, rot_top_x);
        update_min_max_2(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, rot_top_y);
        rot_bot_x = (-orig_bottom_y) * sin;
        rot_bot_y = orig_bottom_y * cos;
        update_min_max_2(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, rot_bot_x);
        update_min_max_2(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, rot_bot_y);
        rot_right_x = fmadds(orig_right_x, cos, -(midpoint_x * sin));
        rot_right_y = fmadds(orig_right_x, sin, midpoint_x * cos);
        update_min_max_2(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, rot_right_x);
        update_min_max_2(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, rot_right_y);
        rot_left_x = fmadds(orig_left_x, cos, -(midpoint_x * sin));
        rot_left_y = fmadds(orig_left_x, sin, midpoint_x * cos);
        update_min_max_2(core::ptr::addr_of_mut!(left_x) as *mut f32, core::ptr::addr_of_mut!(right_x) as *mut f32, rot_left_x);
        update_min_max_2(core::ptr::addr_of_mut!(bottom_y) as *mut f32, core::ptr::addr_of_mut!(top_y) as *mut f32, rot_left_y);
    }
    clamp_above_2(core::ptr::addr_of_mut!(top_y) as *mut f32, 0.0f32);
    clamp_below_2(core::ptr::addr_of_mut!(bottom_y) as *mut f32, -0.0f32);
    clamp_above_2(core::ptr::addr_of_mut!(right_x) as *mut f32, 0.0f32);
    clamp_below_2(core::ptr::addr_of_mut!(left_x) as *mut f32, -0.0f32);
    if (top_y - bottom_y) < 3.0f32 {
        top_y = 1.5f32;
        bottom_y = -top_y;
    }
    if (right_x - left_x) < 3.0f32 {
        right_x = 1.5f32;
        left_x = -right_x;
    }
    (*coll).desired_ecb.top.x = 0.0f32;
    (*coll).desired_ecb.top.y = top_y;
    (*coll).desired_ecb.bottom.x = 0.0f32;
    (*coll).desired_ecb.bottom.y = bottom_y;
    midpoint_y = 0.5f32 * (top_y + bottom_y);
    (*coll).desired_ecb.right.x = right_x;
    (*coll).desired_ecb.right.y = midpoint_y;
    (*coll).desired_ecb.left.x = left_x;
    (*coll).desired_ecb.left.y = midpoint_y;
    (*coll).x34_flags.set_b0((0 as u8) as u8);
}

/// `mpColl_80042C58`
pub unsafe fn mpColl_80042C58(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) {
    melee_core::ct!("mpColl_80042C58");
    if ((*coll).x130_flags & ((CollData_X130_Clear as i32) as u32)) != 0 {
        (*coll).ecb.top.x = 0.0f32;
        (*coll).ecb.top.y = 0.0f32;
        (*coll).ecb.bottom.x = 0.0f32;
        (*coll).ecb.bottom.y = 0.0f32;
        (*coll).ecb.right.x = 0.0f32;
        (*coll).ecb.right.y = 0.0f32;
        (*coll).ecb.left.x = 0.0f32;
        (*coll).ecb.left.y = 0.0f32;
        (*coll).x130_flags = (*coll).x130_flags & ((!(CollData_X130_Clear as i32)) as u32);
    }
    (*coll).xE4_ecb = (*coll).ecb;
    (*coll).desired_ecb.top.x = 0.0f32;
    (*coll).desired_ecb.top.y = (*arg1).top;
    (*coll).desired_ecb.bottom.x = 0.0f32;
    (*coll).desired_ecb.bottom.y = (*arg1).bottom;
    (*coll).desired_ecb.right.x = (*arg1).right.x;
    (*coll).desired_ecb.right.y = (*arg1).right.y;
    (*coll).desired_ecb.left.x = (*arg1).left.x;
    (*coll).desired_ecb.left.y = (*arg1).left.y;
    (*coll).x34_flags.set_b0((0 as u8) as u8);
}

/// `mpColl_LoadECB_inline`
unsafe fn mpColl_LoadECB_inline(mut coll: *mut CollData, mut i: i32) {
    let mut saved_bottom_x: f32 = 0.0;
    let mut saved_bottom_y: f32 = 0.0;
    if ((*coll).x130_flags & ((CollData_X130_Locked as i32) as u32)) != 0 {
        saved_bottom_x = (*coll).desired_ecb.bottom.x;
        saved_bottom_y = (*coll).desired_ecb.bottom.y;
    }
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, i as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    if ((*coll).x130_flags & ((CollData_X130_Locked as i32) as u32)) != 0 {
        (*coll).desired_ecb.bottom.x = saved_bottom_x;
        (*coll).desired_ecb.bottom.y = saved_bottom_y;
    }
    mpColl_80042384(coll);
}

/// `mpColl_LoadECB`
pub unsafe fn mpColl_LoadECB(mut coll: *mut CollData) {
    melee_core::ct!("mpColl_LoadECB");
    let mut saved_bottom_x: f32 = 0.0;
    let mut saved_bottom_y: f32 = 0.0;
    if ((*coll).x130_flags & ((CollData_X130_Locked as i32) as u32)) != 0 {
        saved_bottom_x = (*coll).desired_ecb.bottom.x;
        saved_bottom_y = (*coll).desired_ecb.bottom.y;
    }
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 6 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    if ((*coll).x130_flags & ((CollData_X130_Locked as i32) as u32)) != 0 {
        (*coll).desired_ecb.bottom.x = saved_bottom_x;
        (*coll).desired_ecb.bottom.y = saved_bottom_y;
    }
    mpColl_80042384(coll);
}

/// `Vec2_Interpolate`
unsafe fn Vec2_Interpolate(mut time: f32, mut dest: *mut Vec2, mut src: *mut Vec2) {
    (*dest).x = fmadds(time, (*src).x - (*dest).x, (*dest).x);
    (*dest).y = fmadds(time, (*src).y - (*dest).y, (*dest).y);
}

/// `mpCollInterpolateECB`
pub unsafe fn mpCollInterpolateECB(mut coll: *mut CollData, mut time: f32) {
    melee_core::ct!("mpCollInterpolateECB");
    (*coll).prev_ecb = (*coll).ecb;
    if (*coll).x34_flags.b6() != 0 {
        (*coll).ecb = (*coll).x64_ecb;
        (*coll).x34_flags.set_b6((0 as u8) as u8);
    }
    Vec2_Interpolate(time, core::ptr::addr_of_mut!((*coll).ecb.top) as *mut Vec2, core::ptr::addr_of_mut!((*coll).desired_ecb.top) as *mut Vec2);
    Vec2_Interpolate(time, core::ptr::addr_of_mut!((*coll).ecb.bottom) as *mut Vec2, core::ptr::addr_of_mut!((*coll).desired_ecb.bottom) as *mut Vec2);
    Vec2_Interpolate(time, core::ptr::addr_of_mut!((*coll).ecb.left) as *mut Vec2, core::ptr::addr_of_mut!((*coll).desired_ecb.left) as *mut Vec2);
    Vec2_Interpolate(time, core::ptr::addr_of_mut!((*coll).ecb.right) as *mut Vec2, core::ptr::addr_of_mut!((*coll).desired_ecb.right) as *mut Vec2);
    if ((((((((if 1 != 0 { __fpclassifyf((*coll).ecb.top.x) } else { __fpclassifyd((*coll).ecb.top.x as f64) }) == (FP_NAN as i32)) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.top.y) } else { __fpclassifyd((*coll).ecb.top.y as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.bottom.x) } else { __fpclassifyd((*coll).ecb.bottom.x as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.bottom.y) } else { __fpclassifyd((*coll).ecb.bottom.y as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.left.x) } else { __fpclassifyd((*coll).ecb.left.x as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.left.y) } else { __fpclassifyd((*coll).ecb.left.y as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.right.x) } else { __fpclassifyd((*coll).ecb.right.x as f64) }) == (FP_NAN as i32))) || ((if 1 != 0 { __fpclassifyf((*coll).ecb.right.y) } else { __fpclassifyd((*coll).ecb.right.y as f64) }) == (FP_NAN as i32)) {
        ext::OSReport(&format!("mpcoll: ECB NaN: gobj={:p} classifier={} p_link={} itkind={} time={:.6}\n", (*coll).x0_gobj as *mut c_void, if !(*coll).x0_gobj.is_null() { (*(*coll).x0_gobj).classifier as i32 } else { (-1_i32) }, if !(*coll).x0_gobj.is_null() { (*(*coll).x0_gobj).p_link as i32 } else { (-1_i32) }, if (!(*coll).x0_gobj.is_null()) && (((*(*coll).x0_gobj).p_link as i32) == 9) { ext::itGetKind((*coll).x0_gobj) } else { (-1_i32) }, time as f64));
        ext::OSReport(&format!("mpcoll:   pos({:.6},{:.6}) ecb T({:.6},{:.6}) B({:.6},{:.6}) L({:.6},{:.6}) R({:.6},{:.6})\n", (*coll).cur_pos.x as f64, (*coll).cur_pos.y as f64, (*coll).ecb.top.x as f64, (*coll).ecb.top.y as f64, (*coll).ecb.bottom.x as f64, (*coll).ecb.bottom.y as f64, (*coll).ecb.left.x as f64, (*coll).ecb.left.y as f64, (*coll).ecb.right.x as f64, (*coll).ecb.right.y as f64));
        ext::OSReport(&format!("mpcoll:   desired T({:.6},{:.6}) B({:.6},{:.6}) L({:.6},{:.6}) R({:.6},{:.6})\n", (*coll).desired_ecb.top.x as f64, (*coll).desired_ecb.top.y as f64, (*coll).desired_ecb.bottom.x as f64, (*coll).desired_ecb.bottom.y as f64, (*coll).desired_ecb.left.x as f64, (*coll).desired_ecb.left.y as f64, (*coll).desired_ecb.right.x as f64, (*coll).desired_ecb.right.y as f64));
        ext::OSReport(&format!("mpcoll:   prev    T({:.6},{:.6}) B({:.6},{:.6}) L({:.6},{:.6}) R({:.6},{:.6})\n", (*coll).prev_ecb.top.x as f64, (*coll).prev_ecb.top.y as f64, (*coll).prev_ecb.bottom.x as f64, (*coll).prev_ecb.bottom.y as f64, (*coll).prev_ecb.left.x as f64, (*coll).prev_ecb.left.y as f64, (*coll).prev_ecb.right.x as f64, (*coll).prev_ecb.right.y as f64));
        if 0 != 0 { (); } else { ext::OSReport(&format!("error\n")); panic!("mpCollInterpolateECB: assertion failed"); }
    }
}

/// `mpColl_RightWall_inline`
unsafe fn mpColl_RightWall_inline(mut line_id: i32) {
    let mut i: i32 = 0;
    {
        i = 0;
        'l20: while i < mpColl_804D6488 {
            let mut start_id: i32 = mpColl_80458810.right[i as usize];
            if (line_id == start_id) || (super::mplib::mpLinesConnected(start_id, line_id) != 0) {
                return;
            }
            i = i + 1;
        }
    }
    if i < 9 { (); } else { panic!("mpColl_RightWall_inline: assertion failed"); }
    mpColl_80458810.right[i as usize] = line_id;
    mpColl_804D6488 = mpColl_804D6488 + 1;
}

/// `mpColl_LeftWall_inline`
unsafe fn mpColl_LeftWall_inline(mut line_id: i32) {
    let mut i: i32 = 0;
    let mut arr: *mut i32 = mpColl_80458810.left.as_mut_ptr();
    'l21: while i < mpColl_804D648C {
        if (line_id == (*arr)) || (super::mplib::mpLinesConnected(*arr, line_id) != 0) {
            return;
        }
        arr = arr.offset(1); i = i + 1;
    }
    if i < 9 { (); } else { panic!("mpColl_LeftWall_inline: assertion failed"); }
    mpColl_80458810.left[i as usize] = line_id;
    mpColl_804D648C = mpColl_804D648C + 1;
}

/// `mpColl_LeftWall_inline3`
unsafe fn mpColl_LeftWall_inline3(mut line_id: i32, mut arr: *mut i32) {
    let mut i: i32 = 0;
    {
        i = 0;
        'l22: while i < mpColl_804D648C {
            if (line_id == (*arr)) || (super::mplib::mpLinesConnected(*arr, line_id) != 0) {
                return;
            }
            arr = arr.offset(1); i = i + 1;
        }
    }
    if i < 9 { (); } else { panic!("mpColl_LeftWall_inline3: assertion failed"); }
    mpColl_80458810.left[i as usize] = line_id;
    mpColl_804D648C = mpColl_804D648C + 1;
}

/// `mpColl_80043268`
pub unsafe fn mpColl_80043268(mut coll: *mut CollData, mut line_id: i32, mut arg2: i32, mut dy: f32) {
    melee_core::ct!("mpColl_80043268");
    let mut joint_id: i32 = 0;
    joint_id = super::mplib::mpJointFromLine(line_id);
    if joint_id != (-1_i32) {
        let mut callback: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)> = None;
        let mut user_data: *mut c_void = null_mut();
        super::mplib::mpJointGetCb1(joint_id, core::ptr::addr_of_mut!(callback) as *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, core::ptr::addr_of_mut!(user_data) as *mut *mut c_void);
        if callback.is_some() {
            let mut thing: i32 = 0;
            if !(arg2 != 0) {
                thing = 2;
            } else {
                thing = 1;
            }
            callback.unwrap()(user_data, joint_id, coll, (*coll).x50 as i32, thing as u32, dy);
        }
    }
}

/// `mpCollEnd_inline2`
unsafe fn mpCollEnd_inline2(mut coll: *mut CollData, mut line_id: i32, mut arg2: i32, mut dy: f32) {
    let mut dummy: i32 = 0;
    let mut joint_id: i32 = 0;
    joint_id = super::mplib::mpJointFromLine(line_id);
    if joint_id != (-1_i32) {
        let mut callback: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)> = None;
        let mut user_data: *mut c_void = null_mut();
        super::mplib::mpJointGetCb2(joint_id, core::ptr::addr_of_mut!(callback) as *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, core::ptr::addr_of_mut!(user_data) as *mut *mut c_void);
        if callback.is_some() {
            callback.unwrap()(user_data, joint_id, coll, (*coll).x50 as i32, 0 as u32, dy);
        }
    }
}

/// `mpCollEnd_inline`
unsafe fn mpCollEnd_inline(mut coll: *mut CollData, mut line_id: i32, mut arg2: i32, mut dy: f32) {
    mpColl_80043268(coll, line_id, arg2, dy);
}

/// `mpCollEnd`
pub unsafe fn mpCollEnd(mut coll: *mut CollData, mut arg1: i32, mut arg2: i32) {
    melee_core::ct!("mpCollEnd");
    if (*coll).floor.index != (-1_i32) {
        let mut temp_r3: i32 = ext::grDynamicAttr_801CA284(core::ptr::addr_of_mut!((*coll).cur_pos) as *mut Vec, (*coll).floor.index);
        if temp_r3 != 0 {
            (*coll).floor.flags = ((*coll).floor.flags & ((!255) as u32)) | ((temp_r3 & 255) as u32);
        }
    }
    if (((arg1 != 0) || (((*coll).env_flags & 8388608) != 0)) || (((*coll).env_flags & 1048576) != 0)) || (((*coll).env_flags & 2097152) != 0) {
        mpCollEnd_inline(coll, (*coll).floor.index, arg2, (*coll).cur_pos.y - (*coll).last_pos.y);
    }
    if ((*coll).env_flags & (16384 | 8192)) != 0 {
        mpCollEnd_inline2(coll, (*coll).ceiling.index, arg2, (*coll).cur_pos.y - (*coll).last_pos.y);
    }
    if ext::DbLevel >= ((DbLKind_DebugRom as i32) as u32) {
        if !(((((*coll).cur_pos.x < 4.5e+04f32) && ((*coll).cur_pos.x > (-4.5e+04f32))) && ((*coll).cur_pos.y < 4.5e+04f32)) && ((*coll).cur_pos.y > (-4.5e+04f32))) {
            if ext::ftLib_80086960((*coll).x0_gobj) != 0 {
                ext::OSReport(&format!("{}:{}: Error: mpCollEnd() last({:.6},{:.6}) pos({:.6},{:.6}) ply={} ms={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 1350, (*coll).last_pos.x as f64, (*coll).last_pos.y as f64, (*coll).cur_pos.x as f64, (*coll).cur_pos.y as f64, ext::ftLib_80086BE0((*coll).x0_gobj) as i32, ext::ftLib_800874BC((*coll).x0_gobj)));
            } else {
                let mut gobjid: i32 = (*(*coll).x0_gobj).classifier as i32;
                ext::OSReport(&format!("{}:{}: Error: mpCollEnd() last({:.6},{:.6}) pos({:.6},{:.6}) gobjid={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 1358, (*coll).last_pos.x as f64, (*coll).last_pos.y as f64, (*coll).cur_pos.x as f64, (*coll).cur_pos.y as f64, gobjid));
                if ((*(*coll).x0_gobj).p_link as i32) == 9 {
                    ext::OSReport(&format!("itkind={}\n", ext::itGetKind((*coll).x0_gobj)));
                }
            }
            if 0 != 0 { (); } else { panic!("mpCollEnd: assertion failed"); }
        }
    }
}

/// `mpColl_80043558`
pub unsafe fn mpColl_80043558(mut coll: *mut CollData, mut line_id: i32) {
    melee_core::ct!("mpColl_80043558");
    let mut kind: i32 = 0;
    let mut joint_id: i32 = 0;
    kind = super::mplib::mpLineGetKind(line_id);
    if kind == (CollLine_Floor as i32) {
        joint_id = super::mplib::mpJointFromLine(line_id);
        if joint_id != (-1_i32) {
            let mut user_data: *mut c_void = null_mut();
            let mut callback: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)> = None;
            super::mplib::mpJointGetCb1(joint_id, core::ptr::addr_of_mut!(callback) as *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, core::ptr::addr_of_mut!(user_data) as *mut *mut c_void);
            if callback.is_some() {
                callback.unwrap()(user_data, joint_id, coll, (*coll).x50 as i32, 2 as u32, 0.0f32);
            }
        }
    } else if kind == (CollLine_Ceiling as i32) {
        joint_id = super::mplib::mpJointFromLine(line_id);
        if joint_id != (-1_i32) {
            let mut user_data: *mut c_void = null_mut();
            let mut callback: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)> = None;
            super::mplib::mpJointGetCb2(joint_id, core::ptr::addr_of_mut!(callback) as *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, core::ptr::addr_of_mut!(user_data) as *mut *mut c_void);
            if callback.is_some() {
                callback.unwrap()(user_data, joint_id, coll, (*coll).x50 as i32, 0 as u32, 0.0f32);
            }
        }
    }
}

/// `mpColl_80043670`
pub unsafe fn mpColl_80043670(mut coll: *mut CollData) {
    melee_core::ct!("mpColl_80043670");
    (*coll).x130_flags = (*coll).x130_flags | ((CollData_X130_Clear as i32) as u32);
}

/// `mpColl_80043680`
pub unsafe fn mpColl_80043680(mut coll: *mut CollData, mut arg1: *mut Vec) {
    melee_core::ct!("mpColl_80043680");
    (*coll).cur_pos = *arg1;
    (*coll).prev_pos = (*coll).cur_pos;
    (*coll).last_pos = (*coll).prev_pos;
    (*coll).x130_flags = (*coll).x130_flags | ((CollData_X130_Clear as i32) as u32);
}

/// `mpCollSetFacingDir`
pub unsafe fn mpCollSetFacingDir(mut coll: *mut CollData, mut facing_dir: i32) {
    melee_core::ct!("mpCollSetFacingDir");
    (*coll).facing_dir = facing_dir as i16;
}

/// `mpColl_800436E4`
pub unsafe fn mpColl_800436E4(mut coll: *mut CollData, mut arg1: f32) {
    melee_core::ct!("mpColl_800436E4");
    let mut var_f1: f32 = 0.0;
    var_f1 = arg1;
    if (*coll).ecb_source.kind == ((ECBSource_Fixed as i32) as u32) {
        if (var_f1 as f64) > 6.283185307179586f64 {
            var_f1 = ((var_f1 as f64) - 6.283185307179586f64) as f32;
        } else if (var_f1 as f64) < (-6.283185307179586f64) {
            var_f1 = ((var_f1 as f64) + 6.283185307179586f64) as f32;
        }
        (*coll).ecb_source.anon_4.anon_0_0.angle = var_f1;
    } else {
        ext::OSReport(&format!("not support rotate at JObj type coll\n"));
        'l23: while 1 != 0 {
        }
    }
}

/// `max_inline`
unsafe fn max_inline(mut a: f32, mut b: f32) -> f32 {
    return if a > b { a } else { b };
}

/// `mpColl_80043754`
pub unsafe fn mpColl_80043754(mut cb: Option<unsafe extern "C" fn(*mut CollData, u32) -> i32>, mut coll: *mut CollData, mut flags: u32) -> i32 {
    melee_core::ct!("mpColl_80043754");
    let mut vel: Vec3 = unsafe { core::mem::zeroed() };
    let mut __: [u8; 4] = [0; 4];
    let mut dist_right_x: f32 = 0.0;
    let mut dist_right_y: f32 = 0.0;
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut dist_left_x: f32 = 0.0;
    let mut dist_top_y: f32 = 0.0;
    let mut step: i32 = 0;
    let mut steps: i32 = 0;
    let mut ret: i32 = 0;
    let _ = lbVector_Diff(core::ptr::addr_of_mut!((*coll).cur_pos) as *mut Vec, core::ptr::addr_of_mut!((*coll).last_pos) as *mut Vec, core::ptr::addr_of_mut!(vel) as *mut Vec);
    x = if vel.x < (0 as f32) { -vel.x } else { vel.x };
    y = if vel.y < (0 as f32) { -vel.y } else { vel.y };
    dist_left_x = (*coll).desired_ecb.left.x - (*coll).ecb.left.x;
    dist_left_x = if dist_left_x < (0 as f32) { -dist_left_x } else { dist_left_x };
    dist_right_x = (*coll).desired_ecb.right.x - (*coll).ecb.right.x;
    if dist_right_x < (0 as f32) {
        dist_right_x = -dist_right_x;
    }
    if dist_left_x < dist_right_x {
        dist_left_x = dist_right_x;
    }
    dist_top_y = (*coll).desired_ecb.top.y - (*coll).ecb.top.y;
    dist_top_y = if dist_top_y < (0 as f32) { -dist_top_y } else { dist_top_y };
    dist_right_y = (*coll).desired_ecb.right.y - (*coll).ecb.right.y;
    if dist_right_y < (0 as f32) {
        dist_right_y = -dist_right_y;
    }
    if dist_top_y < dist_right_y {
        dist_top_y = dist_right_y;
    }
    x = max_inline(x, dist_left_x);
    y = max_inline(y, dist_top_y);
    x = max_inline(x, y);
    if x > 6.0f32 {
        steps = (x / 6.0f32) as i32;
        steps = steps + 1;
        vel.x = vel.x / (steps as f32);
        vel.y = vel.y / (steps as f32);
        vel.z = vel.z / (steps as f32);
    } else {
        steps = 1;
    }
    step = 0;
    (*coll).cur_pos = (*coll).last_pos;
    (*coll).x34_flags.set_b5((0 as u8) as u8);
    'l24: while (step < steps) && (!((*coll).x34_flags.b5() != 0)) {
        mpCollInterpolateECB(coll, 1.0f32 / ((steps - step) as f32));
        (*coll).prev_pos = (*coll).cur_pos;
        let _ = lbVector_Add(core::ptr::addr_of_mut!((*coll).cur_pos) as *mut Vec, core::ptr::addr_of_mut!(vel) as *mut Vec);
        mpCollCheckBounding(coll, flags);
        ret = cb.unwrap()(coll, flags);
        super::mplib::mpUncheckBounding();
        step = step + 1;
        (*coll).x38 = mpColl_804D64AC;
    }
    return ret;
}

/// `mpColl_800439FC`
pub unsafe fn mpColl_800439FC(mut coll: *mut CollData) {
    melee_core::ct!("mpColl_800439FC");
    let mut right_dx: f32 = 0.0;
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut right_x: f32 = 0.0;
    let mut right_y: f32 = 0.0;
    let mut sp10: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    right_x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    right_y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    right_dx = if (*coll).ecb.right.x < (0 as f32) { -(*coll).ecb.right.x } else { (*coll).ecb.right.x };
    f1 = fmadds((*coll).ceiling.normal.y, right_dx, right_x);
    f2 = fmadds(-(*coll).ceiling.normal.x, right_dx, right_y);
    if !(super::mplib::mpCheckLeftWall(f1, f2, right_x, right_y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
        return;
    }
    sp10.x = (*coll).contact.x - right_dx;
    sp10.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(sp10) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec) == (-1_i32) {
        return;
    }
    (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    (*coll).cur_pos.x = sp10.x;
}

/// `mpColl_80043ADC`
pub unsafe fn mpColl_80043ADC(mut coll: *mut CollData) {
    melee_core::ct!("mpColl_80043ADC");
    let mut left_dx: f32 = 0.0;
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut left_x: f32 = 0.0;
    let mut left_y: f32 = 0.0;
    let mut sp10: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    left_x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    left_y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    left_dx = if (*coll).ecb.left.x < (0 as f32) { -(*coll).ecb.left.x } else { (*coll).ecb.left.x };
    f1 = fmadds(-(*coll).ceiling.normal.y, left_dx, left_x);
    f2 = fmadds((*coll).ceiling.normal.x, left_dx, left_y);
    if !(super::mplib::mpCheckRightWall(f1, f2, left_x, left_y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
        return;
    }
    sp10.x = (*coll).contact.x + left_dx;
    sp10.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(sp10) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec) == (-1_i32) {
        return;
    }
    (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    (*coll).cur_pos.x = sp10.x;
}

/// `mpColl_80043BBC`
pub unsafe fn mpColl_80043BBC(mut coll: *mut CollData, mut line_id_out: *mut i32) -> i32 {
    melee_core::ct!("mpColl_80043BBC");
    let mut line_id: i32 = super::mplib::mpLinePrevNonFloor((*coll).floor.index);
    let mut bottom_x: f32 = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    let mut bottom_y: f32 = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    let mut right_x: f32 = (*coll).cur_pos.x + (*coll).ecb.right.x;
    let mut right_y: f32 = (*coll).cur_pos.y + (*coll).ecb.right.y;
    let mut wall_id: i32 = 0;
    if (super::mplib::mpCheckLeftWall(bottom_x, bottom_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (wall_id != line_id) {
        *line_id_out = wall_id;
        return 1;
    }
    return 0;
}

/// `mpColl_80043C6C`
pub unsafe fn mpColl_80043C6C(mut coll: *mut CollData, mut line_id: i32, mut ignore_bottom: i32) {
    melee_core::ct!("mpColl_80043C6C");
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut right_dx: f32 = 0.0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut wall_id: i32 = 0;
    right_dx = if (*coll).ecb.right.x < (0 as f32) { -(*coll).ecb.right.x } else { (*coll).ecb.right.x };
    pos.x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    pos.y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    if super::mplib::mpLib_8004E398_LeftWall(line_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), null_mut(), null_mut()) != (-1_i32) {
        let mut floor_x: f32 = fmadds(-(*coll).floor.normal.y, right_dx, pos.x);
        let mut floor_y: f32 = fmadds((*coll).floor.normal.x, right_dx, pos.y);
        if super::mplib::mpCheckLeftWall(floor_x, floor_y, pos.x, pos.y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            pos.x = (*coll).contact.x - right_dx;
            if ignore_bottom != 0 {
                pos.y = (*coll).cur_pos.y;
            } else {
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
            }
            if super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec) != (-1_i32) {
                (*coll).cur_pos.y = (*coll).cur_pos.y + y;
                (*coll).cur_pos.x = pos.x;
            }
        }
    } else {
        super::mplib::mpLeftWallGetTop(line_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
        f1 = pos.x - 2.0f32;
        f2 = pos.y;
        pos.x = -fmadds(2.0f32, right_dx, -f1);
        pos.y = -fmadds(2.0f32, (*coll).ecb.right.y - (*coll).ecb.bottom.y, -f2);
        if super::mplib::mpCheckFloor(f1, f2, pos.x, pos.y, 0.0f32, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, null_mut(), null_mut(), null_mut(), (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut()) != 0 {
            pos.x = (*coll).contact.x;
            if ignore_bottom != 0 {
                pos.y = (*coll).cur_pos.y;
            } else {
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
            }
            if super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec) != (-1_i32) {
                (*coll).cur_pos.y = (*coll).cur_pos.y + y;
                (*coll).cur_pos.x = pos.x;
            }
        }
    }
}

/// `mpColl_80043E90`
pub unsafe fn mpColl_80043E90(mut coll: *mut CollData, mut line_id_out: *mut i32) -> i32 {
    melee_core::ct!("mpColl_80043E90");
    let mut line_id: i32 = super::mplib::mpLineNextNonFloor((*coll).floor.index);
    let mut bottom_x: f32 = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    let mut bottom_y: f32 = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    let mut left_x: f32 = (*coll).cur_pos.x + (*coll).ecb.left.x;
    let mut left_y: f32 = (*coll).cur_pos.y + (*coll).ecb.left.y;
    let mut wall_id: i32 = 0;
    if (super::mplib::mpCheckRightWall(bottom_x, bottom_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (wall_id != line_id) {
        *line_id_out = wall_id;
        return 1;
    }
    return 0;
}

/// `mpColl_80043F40`
pub unsafe fn mpColl_80043F40(mut coll: *mut CollData, mut line_id: i32, mut ignore_bottom: i32) {
    melee_core::ct!("mpColl_80043F40");
    let mut f1: f32 = 0.0;
    let mut f2: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut left_dx: f32 = 0.0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut wall_id: i32 = 0;
    left_dx = if (*coll).ecb.left.x < (0 as f32) { -(*coll).ecb.left.x } else { (*coll).ecb.left.x };
    pos.x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    pos.y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    if super::mplib::mpLib_8004E684_RightWall(line_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), null_mut(), null_mut()) != (-1_i32) {
        let mut floor_x: f32 = fmadds((*coll).floor.normal.y, left_dx, pos.x);
        let mut floor_y: f32 = fmadds(-(*coll).floor.normal.x, left_dx, pos.y);
        if super::mplib::mpCheckRightWall(floor_x, floor_y, pos.x, pos.y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            pos.x = (*coll).contact.x + left_dx;
            if ignore_bottom != 0 {
                pos.y = (*coll).cur_pos.y;
            } else {
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
            }
            if super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec) != (-1_i32) {
                (*coll).cur_pos.y = (*coll).cur_pos.y + y;
                (*coll).cur_pos.x = pos.x;
            }
        }
    } else {
        super::mplib::mpRightWallGetTop(line_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
        f1 = 2.0f32 + pos.x;
        f2 = pos.y;
        pos.x = fmadds(2.0f32, left_dx, f1);
        pos.y = fmadds(-2.0f32, (*coll).ecb.left.y - (*coll).ecb.bottom.y, f2);
        if super::mplib::mpCheckFloor(f1, f2, pos.x, pos.y, 0.0f32, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, null_mut(), null_mut(), null_mut(), (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut()) != 0 {
            pos.x = (*coll).contact.x;
            if ignore_bottom != 0 {
                pos.y = (*coll).cur_pos.y;
            } else {
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
            }
            if super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec) != (-1_i32) {
                (*coll).cur_pos.x = pos.x;
                (*coll).cur_pos.y = (*coll).cur_pos.y + y;
            }
        }
    }
}

/// `mpColl_80044164`
pub unsafe fn mpColl_80044164(mut cd: *mut CollData, mut p_ledge_id: *mut i32) -> i32 {
    melee_core::ct!("mpColl_80044164");
    let mut left: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut top: f32 = 0.0;
    let mut grabbed_ledge: i32 = 0;
    let mut already_checked: i32 = 0;
    let mut ledge_id: i32 = 0;
    let mut __: [u8; 8] = [0; 8];
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut half_height: f32 = 0.5f32 * (*cd).ledge_snap_height;
    let mut snap_x: f32 = (*cd).ledge_snap_x;
    let mut snap_y: f32 = (*cd).ledge_snap_y;
    if (*cd).prev_pos.x < (*cd).cur_pos.x {
        left = (*cd).prev_pos.x;
        right = snap_x + ((*cd).cur_pos.x + (*cd).ecb.right.x);
    } else {
        left = (*cd).cur_pos.x;
        right = snap_x + ((*cd).prev_pos.x + (*cd).ecb.right.x);
    }
    if (*cd).prev_pos.y < (*cd).cur_pos.y {
        bottom = ((*cd).prev_pos.y + snap_y) - half_height;
        top = half_height + ((*cd).cur_pos.y + snap_y);
    } else {
        bottom = ((*cd).cur_pos.y + snap_y) - half_height;
        top = half_height + ((*cd).prev_pos.y + snap_y);
    }
    already_checked = super::mplib::mpCheckedBounding();
    if !(already_checked != 0) {
        super::mplib::mpBoundingCheck(left, bottom, right, top);
    }
    ledge_id = super::mplib::mpLib_80051BA8_Floor(core::ptr::addr_of_mut!((*cd).contact) as *mut Vec, (*cd).floor_skip, (*cd).joint_id_skip, (*cd).joint_id_only, 1, left, bottom, right, top);
    if ((((ledge_id != (-1_i32)) && ({ super::mplib::mpFloorGetLeft(ledge_id, core::ptr::addr_of_mut!(edge) as *mut Vec); ((*cd).contact.x - edge.x) < 5.0f32 })) && (((*cd).cur_pos.x + (*cd).ecb.bottom.x) < edge.x)) && (((*cd).cur_pos.y + (*cd).ecb.bottom.y) < edge.y)) && ((((*cd).cur_pos.y + (*cd).ecb.bottom.y) > (*cd).contact.y) || (((!(super::mplib::mpCheckMultiple((*cd).cur_pos.x + (*cd).ecb.top.x, (*cd).cur_pos.y + (*cd).ecb.top.y, (*cd).contact.x, (*cd).contact.y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), 6 as u32, (*cd).joint_id_skip, (*cd).joint_id_only) != 0)) || (mpColl_stage_same_joint(ledge_id, line_id) != 0)) && ((!(super::mplib::mpCheckMultiple((*cd).cur_pos.x + (*cd).ecb.bottom.x, (-2.0f32) + ((*cd).cur_pos.y + (*cd).ecb.bottom.y), (*cd).contact.x, (*cd).contact.y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), 6 as u32, (*cd).joint_id_skip, (*cd).joint_id_only) != 0)) || (mpColl_stage_same_joint(ledge_id, line_id) != 0)))) {
        if !p_ledge_id.is_null() {
            *p_ledge_id = ledge_id;
        }
        grabbed_ledge = 1;
    } else {
        grabbed_ledge = 0;
    }
    if !(already_checked != 0) {
        super::mplib::mpUncheckBounding();
    }
    return grabbed_ledge;
}

/// `mpColl_800443C4`
pub unsafe fn mpColl_800443C4(mut cd: *mut CollData, mut p_ledge_id: *mut i32) -> i32 {
    melee_core::ct!("mpColl_800443C4");
    let mut left: f32 = 0.0;
    let mut bottom: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut top: f32 = 0.0;
    let mut grabbed_ledge: i32 = 0;
    let mut already_checked: i32 = 0;
    let mut ledge_id: i32 = 0;
    let mut __: [u8; 8] = [0; 8];
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut half_height: f32 = 0.5f32 * (*cd).ledge_snap_height;
    let mut snap_x: f32 = (*cd).ledge_snap_x;
    let mut snap_y: f32 = (*cd).ledge_snap_y;
    snap_x = -snap_x;
    if (*cd).prev_pos.x > (*cd).cur_pos.x {
        right = (*cd).prev_pos.x;
        left = snap_x + ((*cd).cur_pos.x + (*cd).ecb.left.x);
    } else {
        right = (*cd).cur_pos.x;
        left = snap_x + ((*cd).prev_pos.x + (*cd).ecb.left.x);
    }
    if (*cd).prev_pos.y < (*cd).cur_pos.y {
        bottom = ((*cd).prev_pos.y + snap_y) - half_height;
        top = half_height + ((*cd).cur_pos.y + snap_y);
    } else {
        bottom = ((*cd).cur_pos.y + snap_y) - half_height;
        top = half_height + ((*cd).prev_pos.y + snap_y);
    }
    already_checked = super::mplib::mpCheckedBounding();
    if !(already_checked != 0) {
        super::mplib::mpBoundingCheck(left, bottom, right, top);
    }
    ledge_id = super::mplib::mpLib_80051BA8_Floor(core::ptr::addr_of_mut!((*cd).contact) as *mut Vec, (*cd).floor_skip, (*cd).joint_id_skip, (*cd).joint_id_only, (-1_i32), left, bottom, right, top);
    if ((((ledge_id != (-1_i32)) && ({ super::mplib::mpFloorGetRight(ledge_id, core::ptr::addr_of_mut!(edge) as *mut Vec); (edge.x - (*cd).contact.x) < 5.0f32 })) && (((*cd).cur_pos.x + (*cd).ecb.bottom.x) > edge.x)) && (((*cd).cur_pos.y + (*cd).ecb.bottom.y) < edge.y)) && ((((*cd).cur_pos.y + (*cd).ecb.bottom.y) > (*cd).contact.y) || (((!(super::mplib::mpCheckMultiple((*cd).cur_pos.x + (*cd).ecb.top.x, (*cd).cur_pos.y + (*cd).ecb.top.y, (*cd).contact.x, (*cd).contact.y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), 10 as u32, (*cd).joint_id_skip, (*cd).joint_id_only) != 0)) || (mpColl_stage_same_joint(ledge_id, line_id) != 0)) && ((!(super::mplib::mpCheckMultiple((*cd).cur_pos.x + (*cd).ecb.bottom.x, (-2.0f32) + ((*cd).cur_pos.y + (*cd).ecb.bottom.y), (*cd).contact.x, (*cd).contact.y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), 10 as u32, (*cd).joint_id_skip, (*cd).joint_id_only) != 0)) || (mpColl_stage_same_joint(ledge_id, line_id) != 0)))) {
        if !p_ledge_id.is_null() {
            *p_ledge_id = ledge_id;
        }
        grabbed_ledge = 1;
    } else {
        grabbed_ledge = 0;
    }
    if !(already_checked != 0) {
        super::mplib::mpUncheckBounding();
    }
    return grabbed_ledge;
}

/// `mpColl_80044628_Floor`
pub unsafe fn mpColl_80044628_Floor(mut coll: *mut CollData, mut cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut gobj: *mut HSD_GObj, mut left_right: i32) -> i32 {
    melee_core::ct!("mpColl_80044628_Floor");
    let mut y: f32 = 0.0;
    let mut bottom: Vec3 = unsafe { core::mem::zeroed() };
    let mut prev_bottom: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut hit_floor: i32 = 0;
    prev_bottom.x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_bottom.y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if (*coll).x38 != mpColl_804D64AC {
        hit_floor = super::mplib::mpCheckFloorRemap(prev_bottom.x, prev_bottom.y, bottom.x, bottom.y, 0.0f32, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!((*coll).floor.index) as *mut i32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, cb, gobj);
    } else {
        hit_floor = super::mplib::mpCheckFloor(prev_bottom.x, prev_bottom.y, bottom.x, bottom.y, 0.0f32, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!((*coll).floor.index) as *mut i32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, cb, gobj);
    }
    if hit_floor != 0 {
        if (!(((*coll).floor.flags & ((1 << 8) as u32)) != 0)) || ((*coll).floor.index != (*coll).floor_skip) {
            (*coll).env_flags = (*coll).env_flags | 32768;
            (*coll).env_flags = (*coll).env_flags | 65536;
            return 1;
        }
    }
    if (((left_right & 1) != 0) && ({ { line_id = super::mplib::mpLinePrevNonLeftWall((*coll).left_facing_wall.index); line_id }; line_id != (-1_i32) })) || (((left_right & 2) != 0) && ({ { line_id = super::mplib::mpLineNextNonRightWall((*coll).right_facing_wall.index); line_id }; line_id != (-1_i32) })) {
        if (super::mplib::mpLib_80054ED8(line_id) != 0) && (super::mplib::mpLineGetKind(line_id) == (CollLine_Floor as i32)) {
            let mut floor_id: i32 = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!(bottom) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
            if (floor_id != (-1_i32)) && (y > 0.0f32) {
                (*coll).floor.index = floor_id;
                if (!(((*coll).floor.flags & ((1 << 8) as u32)) != 0)) || ((*coll).floor.index != (*coll).floor_skip) {
                    if (cb.is_none()) || (cb.unwrap()(gobj, (*coll).floor.index) != 0) {
                        (*coll).env_flags = (*coll).env_flags | 32768;
                        return 1;
                    }
                }
            }
        }
    }
    return 0;
}

/// `mpColl_80044838_Floor`
pub unsafe fn mpColl_80044838_Floor(mut coll: *mut CollData, mut ignore_bottom: i32) -> i32 {
    melee_core::ct!("mpColl_80044838_Floor");
    let mut bottom: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    if ignore_bottom != 0 {
        bottom.x = (*coll).cur_pos.x;
        bottom.y = (*coll).cur_pos.y;
    } else {
        bottom.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
        bottom.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    }
    line_id = super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(bottom) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
    if line_id != (-1_i32) {
        (*coll).floor.index = line_id;
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    } else {
        super::mplib::mpFloorGetLeft((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if edge.x <= bottom.x {
            super::mplib::mpFloorGetRight((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec);
        }
        (*coll).cur_pos.x = edge.x - (*coll).ecb.bottom.x;
        (*coll).cur_pos.y = edge.y - (*coll).ecb.bottom.y;
        line_id = super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec, null_mut(), core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
        if line_id != (-1_i32) {
            (*coll).floor.index = line_id;
        }
    }
    return 1;
}

/// `mpColl_80044948_Floor`
pub unsafe fn mpColl_80044948_Floor(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80044948_Floor");
    let mut bottom: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut hit_wall: i32 = 0;
    let mut line_id: i32 = 0;
    hit_wall = 0;
    if (*coll).ecb.bottom.y <= 0.0f32 {
        bottom.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
        bottom.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    } else {
        bottom.x = (*coll).cur_pos.x;
        bottom.y = (*coll).cur_pos.y;
    }
    line_id = super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(bottom) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
    if line_id != (-1_i32) {
        (*coll).floor.index = line_id;
        hit_wall = 1;
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    } else {
        super::mplib::mpFloorGetLeft((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if bottom.x < edge.x {
            line_id = super::mplib::mpLinePrevNonFloor((*coll).floor.index);
            if (line_id != (-1_i32)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_RightWall as i32)) {
                hit_wall = 1;
            }
        } else {
            super::mplib::mpFloorGetRight((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec);
            line_id = super::mplib::mpLineNextNonFloor((*coll).floor.index);
            if (line_id != (-1_i32)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_LeftWall as i32)) {
                hit_wall = 1;
            }
        }
        (*coll).cur_pos.y = edge.y - (*coll).ecb.bottom.y;
        if hit_wall != 0 {
            (*coll).cur_pos.x = edge.x;
            line_id = super::mplib::mpLib_8004DD90_Floor((*coll).floor.index, core::ptr::addr_of_mut!(edge) as *mut Vec, null_mut(), core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).floor.index = line_id;
            } else {
                ext::OSReport(&format!("{}:{}: Error:oioi... id={}\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 2488, (*coll).floor.index));
            }
        }
    }
    return hit_wall;
}

/// `mpColl_80044AD8_Ceiling`
pub unsafe fn mpColl_80044AD8_Ceiling(mut coll: *mut CollData, mut left_right: i32) -> i32 {
    melee_core::ct!("mpColl_80044AD8_Ceiling");
    let mut y: f32 = 0.0;
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    let mut prev_top: Vec3 = unsafe { core::mem::zeroed() };
    let mut ceiling_id: i32 = 0;
    let mut line_id: i32 = 0;
    let mut hit_ceiling: i32 = 0;
    prev_top.x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_top.y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if (*coll).x38 != mpColl_804D64AC {
        hit_ceiling = super::mplib::mpCheckCeilingRemap(prev_top.x, prev_top.y, top.x, top.y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!((*coll).ceiling.index) as *mut i32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec, (*coll).joint_id_skip, (*coll).joint_id_only);
    } else {
        hit_ceiling = super::mplib::mpCheckCeiling(prev_top.x, prev_top.y, top.x, top.y, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!((*coll).ceiling.index) as *mut i32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec, (*coll).joint_id_skip, (*coll).joint_id_only);
    }
    if hit_ceiling != 0 {
        (*coll).env_flags = (*coll).env_flags | 8192;
        (*coll).env_flags = (*coll).env_flags | 16384;
        return 1;
    }
    if (((((left_right & 1) != 0) && ({ line_id = super::mplib::mpLineNextNonLeftWall((*coll).left_facing_wall.index); line_id } != (-1_i32))) || (((left_right & 2) != 0) && ({ line_id = super::mplib::mpLinePrevNonRightWall((*coll).right_facing_wall.index); line_id } != (-1_i32)))) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_Ceiling as i32)) {
        ceiling_id = super::mplib::mpLib_8004E090_Ceiling(line_id, core::ptr::addr_of_mut!(top) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
        if (ceiling_id != (-1_i32)) && (y < 0.0f32) {
            (*coll).ceiling.index = ceiling_id;
            (*coll).env_flags = (*coll).env_flags | 8192;
            return 1;
        }
    }
    return 0;
}

/// `mpColl_80044C74_Ceiling`
pub unsafe fn mpColl_80044C74_Ceiling(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80044C74_Ceiling");
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    let mut ceiling_end: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut hit_wall: i32 = 0;
    top.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    line_id = super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(top) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
    if line_id != (-1_i32) {
        (*coll).ceiling.index = line_id;
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    } else {
        hit_wall = 0;
        super::mplib::mpCeilingGetLeft((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
        if top.x <= ceiling_end.x {
            line_id = super::mplib::mpLineNextNonCeiling((*coll).ceiling.index);
            if ((line_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_RightWall as i32)) {
                hit_wall = 1;
            }
        } else {
            super::mplib::mpCeilingGetRight((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
            line_id = super::mplib::mpLinePrevNonCeiling((*coll).ceiling.index);
            if ((line_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_LeftWall as i32)) {
                hit_wall = 1;
            }
        }
        (*coll).cur_pos.y = ceiling_end.y - (*coll).ecb.top.y;
        if hit_wall != 0 {
            (*coll).cur_pos.x = ceiling_end.x;
            line_id = super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec, null_mut(), core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).ceiling.index = line_id;
            } else {
                ext::OSReport(&format!("{}:{}: oioi...\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 2620));
            }
        }
    }
    return 1;
}

/// `mpColl_RightWall_inline2`
unsafe fn mpColl_RightWall_inline2(mut coll: *mut CollData, mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut line_id_out: *mut i32) -> i32 {
    if (*coll).x38 != mpColl_804D64AC {
        return super::mplib::mpCheckRightWallRemap(ax, ay, bx, by, null_mut(), line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only);
    }
    return super::mplib::mpCheckRightWall(ax, ay, bx, by, null_mut(), line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only);
}

/// `mpColl_80044E10_RightWall`
pub unsafe fn mpColl_80044E10_RightWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80044E10_RightWall");
    let mut line_id: i32 = 0;
    let mut left_x: f32 = 0.0;
    let mut left_y: f32 = 0.0;
    let mut prev_left_x: f32 = 0.0;
    let mut prev_left_y: f32 = 0.0;
    let mut vertical_y: f32 = 0.0;
    let mut vertical_x: f32 = 0.0;
    let mut prev_x: f32 = 0.0;
    let mut prev_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    hit_wall = 0;
    mpColl_804D6488 = 0;
    left_x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    left_y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.left.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.left.y;
    if mpColl_RightWall_inline2(coll, prev_x, prev_y, left_x, left_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 2048;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if mpColl_RightWall_inline2(coll, prev_x, prev_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if mpColl_RightWall_inline2(coll, prev_x, prev_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if super::mplib::mpCheckRightWall(vertical_x, vertical_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
        prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
        prev_left_x = (*coll).prev_pos.x + (*coll).prev_ecb.left.x;
        prev_left_y = (*coll).prev_pos.y + (*coll).prev_ecb.left.y;
        if super::mplib::mpLib_800511A4_RightWall(prev_x, prev_y, prev_left_x, prev_left_y, vertical_x, vertical_y, left_x, left_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_RightWall_inline(line_id);
            hit_wall = 1;
        }
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpCheckRightWall(vertical_x, vertical_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_top_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
        let mut prev_top_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
        if super::mplib::mpLib_800511A4_RightWall(prev_left_x, prev_left_y, prev_top_x, prev_top_y, left_x, left_y, vertical_x, vertical_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_RightWall_inline(line_id);
            hit_wall = 1;
        }
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 64;
    }
    return hit_wall;
}

/// `mpColl_800454A4_RightWall`
pub unsafe fn mpColl_800454A4_RightWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800454A4_RightWall");
    let mut flags: u32 = 0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut i: i32 = 0;
    mpColl_804D6490_max_x = -f32::MAX;
    {
        i = 0;
        'l25: while i < mpColl_804D6488 {
            'c25: {
                let mut top: f32 = 0.0;
                let mut mid: f32 = 0.0;
                let mut bot: f32 = 0.0;
                let mut f27: f32 = 0.0;
                let mut f26: f32 = 0.0;
                let mut f2: f32 = 0.0;
                let mut f3: f32 = 0.0;
                let mut f4: f32 = 0.0;
                let mut f5: f32 = 0.0;
                let mut f6: f32 = 0.0;
                let mut wall_id: i32 = 0;
                let mut j: i32 = 0;
                let mut x: f32 = 0.0;
                wall_id = mpColl_80458810.right[i as usize];
                super::mplib::mpRightWallGetTop(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y < ((*coll).cur_pos.y + (*coll).ecb.bottom.y) {
                    if mpColl_804D6490_max_x < pos.x {
                        line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if line_id != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = line_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c25;
                }
                super::mplib::mpRightWallGetBottom(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y > ((*coll).cur_pos.y + (*coll).ecb.top.y) {
                    if mpColl_804D6490_max_x < pos.x {
                        line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if line_id != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = line_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c25;
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.left.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.left.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                line_id = super::mplib::mpLinePrevNonRightWall(wall_id);
                if ((line_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && ((super::mplib::mpLineGetKind(line_id) & (CollLine_Ceiling as i32)) != 0) {
                    let mut top: Vec3 = unsafe { core::mem::zeroed() };
                    let mut nrm: Vec3 = unsafe { core::mem::zeroed() };
                    super::mplib::mpRightWallGetTop(wall_id, core::ptr::addr_of_mut!(top) as *mut Vec);
                    if pos.y > top.y {
                        line_id = super::mplib::mpLineNextNonCeiling(line_id);
                        if ((line_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && ((super::mplib::mpLineGetKind(line_id) & (CollLine_RightWall as i32)) != 0) {
                            let _ = super::mplib::mpLineGetNormal(line_id, core::ptr::addr_of_mut!(nrm) as *mut Vec);
                            x = (fmadds((pos.y - top.y) / nrm.x, -nrm.y, top.x) - pos.x) + 0.5f32;
                            if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                                let mut temp: u32 = super::mplib::mpLineGetFlags(line_id);
                                mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                                mpColl_804D6494_line_id = line_id;
                                mpColl_804D6498_flags = temp;
                                mpColl_80458810.normal = nrm;
                            }
                        }
                    }
                }
                f6 = (*coll).ecb.left.x;
                f5 = (*coll).ecb.top.y;
                f4 = (*coll).ecb.left.y;
                f3 = (*coll).ecb.bottom.y;
                f2 = (*coll).cur_pos.y;
                top = f2 + f5;
                mid = f2 + f4;
                bot = f2 + f3;
                f27 = f6 / (f4 - f3);
                f26 = f6 / (f4 - f5);
                {
                    j = wall_id;
                    'l26: while (j != (-1_i32)) && ((super::mplib::mpLineGetKind(j) & 15) == (CollLine_RightWall as i32)) {
                        'c26: {
                            super::mplib::mpLineGetV1Pos(j, core::ptr::addr_of_mut!(pos) as *mut Vec);
                            if (bot <= pos.y) && (pos.y <= mid) {
                                x = fmadds(f27, pos.y - bot, (*coll).ecb.bottom.x);
                            } else if (mid <= pos.y) && (pos.y <= top) {
                                x = fmadds(f26, pos.y - top, (*coll).ecb.top.x);
                            } else if pos.y < bot {
                                break 'l26;
                            } else {
                                break 'c26;
                            }
                            x = pos.x - x;
                            if mpColl_804D6490_max_x < x {
                                flags = super::mplib::mpLineGetFlags(j);
                                let _ = super::mplib::mpLineGetNormal(j, core::ptr::addr_of_mut!(normal) as *mut Vec);
                                mpColl_804D6490_max_x = x;
                                mpColl_804D6494_line_id = j;
                                mpColl_804D6498_flags = flags;
                                mpColl_80458810.normal = normal;
                            }
                        }
                        j = super::mplib::mpLineGetNext(j);
                    }
                }
                'l27: while (wall_id != (-1_i32)) && ((super::mplib::mpLineGetKind(wall_id) & 15) == (CollLine_RightWall as i32)) {
                    'c27: {
                        super::mplib::mpLineGetV0Pos(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                        if (bot <= pos.y) && (pos.y <= mid) {
                            x = fmadds(f27, pos.y - bot, (*coll).ecb.bottom.x);
                        } else if (mid <= pos.y) && (pos.y <= top) {
                            x = fmadds(f26, pos.y - top, (*coll).ecb.top.x);
                        } else if pos.y > top {
                            break 'l27;
                        } else {
                            break 'c27;
                        }
                        x = pos.x - x;
                        if mpColl_804D6490_max_x < x {
                            flags = super::mplib::mpLineGetFlags(wall_id);
                            let _ = super::mplib::mpLineGetNormal(wall_id, core::ptr::addr_of_mut!(normal) as *mut Vec);
                            mpColl_804D6490_max_x = x;
                            mpColl_804D6494_line_id = wall_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    wall_id = super::mplib::mpLineGetPrev(wall_id);
                }
            }
            i = i + 1;
        }
    }
    flags = mpColl_804D6498_flags;
    normal = mpColl_80458810.normal;
    line_id = mpColl_804D6494_line_id;
    if (*coll).cur_pos.x < mpColl_804D6490_max_x {
        (*coll).cur_pos.x = mpColl_804D6490_max_x;
        (*coll).right_facing_wall.index = line_id;
        (*coll).right_facing_wall.flags = flags;
        (*coll).right_facing_wall.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_LeftWall_inline2`
unsafe fn mpColl_LeftWall_inline2(mut coll: *mut CollData, mut ax: f32, mut ay: f32, mut bx: f32, mut by: f32, mut line_id_out: *mut i32) -> i32 {
    if (*coll).x38 != mpColl_804D64AC {
        return super::mplib::mpCheckLeftWallRemap(ax, ay, bx, by, null_mut(), line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only);
    }
    return super::mplib::mpCheckLeftWall(ax, ay, bx, by, null_mut(), line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only);
}

/// `mpColl_80045B74_LeftWall`
pub unsafe fn mpColl_80045B74_LeftWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80045B74_LeftWall");
    let mut line_id: i32 = 0;
    let mut right_x: f32 = 0.0;
    let mut right_y: f32 = 0.0;
    let mut prev_right_x: f32 = 0.0;
    let mut prev_right_y: f32 = 0.0;
    let mut vertical_y: f32 = 0.0;
    let mut vertical_x: f32 = 0.0;
    let mut prev_x: f32 = 0.0;
    let mut prev_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    hit_wall = 0;
    mpColl_804D648C = 0;
    right_x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    right_y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.right.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.right.y;
    if mpColl_LeftWall_inline2(coll, prev_x, prev_y, right_x, right_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline3(line_id, mpColl_80458810.left.as_mut_ptr());
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 32;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if mpColl_LeftWall_inline2(coll, prev_x, prev_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if mpColl_LeftWall_inline2(coll, prev_x, prev_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if super::mplib::mpCheckLeftWall(vertical_x, vertical_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
        prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
        prev_right_x = (*coll).prev_pos.x + (*coll).prev_ecb.right.x;
        prev_right_y = (*coll).prev_pos.y + (*coll).prev_ecb.right.y;
        if super::mplib::mpLib_800515A0_LeftWall(prev_right_x, prev_right_y, prev_x, prev_y, right_x, right_y, vertical_x, vertical_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_LeftWall_inline(line_id);
            hit_wall = 1;
        }
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpCheckLeftWall(vertical_x, vertical_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_top_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
        let mut prev_top_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
        if super::mplib::mpLib_800515A0_LeftWall(prev_top_x, prev_top_y, prev_right_x, prev_right_y, vertical_x, vertical_y, right_x, right_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_LeftWall_inline(line_id);
            hit_wall = 1;
        }
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 1;
    }
    return hit_wall;
}

/// `mpColl_80046224_LeftWall`
pub unsafe fn mpColl_80046224_LeftWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80046224_LeftWall");
    let mut flags: u32 = 0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut arr: *mut i32 = mpColl_80458810.left.as_mut_ptr();
    let mut i: i32 = 0;
    mpColl_804D6490_max_x = f32::MAX;
    {
        i = 0;
        'l28: while i < mpColl_804D648C {
            'c28: {
                let mut f30: f32 = 0.0;
                let mut f29: f32 = 0.0;
                let mut f28: f32 = 0.0;
                let mut f27: f32 = 0.0;
                let mut f26: f32 = 0.0;
                let mut f4: f32 = 0.0;
                let mut f5: f32 = 0.0;
                let mut f6: f32 = 0.0;
                let mut f7: f32 = 0.0;
                let mut wall_id: i32 = 0;
                let mut x: f32 = 0.0;
                wall_id = *arr;
                super::mplib::mpLeftWallGetTop(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y < ((*coll).cur_pos.y + (*coll).ecb.bottom.y) {
                    if mpColl_804D6490_max_x > pos.x {
                        let mut line_id: i32 = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if line_id != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = line_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c28;
                }
                super::mplib::mpLeftWallGetBottom(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y > ((*coll).cur_pos.y + (*coll).ecb.top.y) {
                    if mpColl_804D6490_max_x > pos.x {
                        let mut line_id: i32 = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if line_id != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = line_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c28;
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.right.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.right.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                line_id = super::mplib::mpLineNextNonLeftWall(wall_id);
                if ((line_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && ((super::mplib::mpLineGetKind(line_id) & (CollLine_Ceiling as i32)) != 0) {
                    let mut line_id2: i32 = 0;
                    let mut vec: Vec3 = unsafe { core::mem::zeroed() };
                    super::mplib::mpLeftWallGetTop(wall_id, core::ptr::addr_of_mut!(vec) as *mut Vec);
                    if pos.y > vec.y {
                        line_id2 = super::mplib::mpLinePrevNonCeiling(line_id);
                        if ((line_id2 != (-1_i32)) && (super::mplib::mpLib_80054ED8(line_id) != 0)) && ((super::mplib::mpLineGetKind(line_id) & (CollLine_LeftWall as i32)) != 0) {
                            let mut nrm: Vec3 = unsafe { core::mem::zeroed() };
                            let _ = super::mplib::mpLineGetNormal(line_id2, core::ptr::addr_of_mut!(nrm) as *mut Vec);
                            x = (fmadds((pos.y - vec.y) / (-nrm.x), nrm.y, vec.x) - pos.x) - 0.5f32;
                            if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                                let mut temp: u32 = super::mplib::mpLineGetFlags(line_id2);
                                mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                                mpColl_804D6494_line_id = line_id2;
                                mpColl_804D6498_flags = temp;
                                mpColl_80458810.normal = nrm;
                            }
                        }
                    }
                }
                f7 = (*coll).ecb.top.y;
                f6 = (*coll).ecb.right.y;
                f5 = (*coll).ecb.bottom.y;
                f4 = (*coll).ecb.right.x;
                f27 = (f4 - (*coll).ecb.bottom.x) / (f6 - f5);
                f26 = (f4 - (*coll).ecb.top.x) / (f6 - f7);
                f30 = (*coll).cur_pos.y + f7;
                f29 = (*coll).cur_pos.y + f6;
                f28 = (*coll).cur_pos.y + f5;
                {
                    let mut line_id2: i32 = 0;
                    {
                        line_id2 = wall_id;
                        'l29: while (line_id2 != (-1_i32)) && ((super::mplib::mpLineGetKind(line_id2) & 15) == (CollLine_LeftWall as i32)) {
                            'c29: {
                                super::mplib::mpLineGetV0Pos(line_id2, core::ptr::addr_of_mut!(pos) as *mut Vec);
                                if (f28 <= pos.y) && (pos.y <= f29) {
                                    x = fmadds(f27, pos.y - f28, (*coll).ecb.bottom.x);
                                } else if (f29 <= pos.y) && (pos.y <= f30) {
                                    x = fmadds(f26, pos.y - f30, (*coll).ecb.top.x);
                                } else if pos.y < f28 {
                                    break 'l29;
                                } else {
                                    break 'c29;
                                }
                                x = pos.x - x;
                                if mpColl_804D6490_max_x > x {
                                    flags = super::mplib::mpLineGetFlags(line_id2);
                                    let _ = super::mplib::mpLineGetNormal(line_id2, core::ptr::addr_of_mut!(normal) as *mut Vec);
                                    mpColl_804D6490_max_x = x;
                                    mpColl_804D6494_line_id = line_id2;
                                    mpColl_804D6498_flags = flags;
                                    mpColl_80458810.normal = normal;
                                }
                            }
                            line_id2 = super::mplib::mpLineGetPrev(line_id2);
                        }
                    }
                }
                'l30: while (wall_id != (-1_i32)) && ((super::mplib::mpLineGetKind(wall_id) & 15) == (CollLine_LeftWall as i32)) {
                    'c30: {
                        super::mplib::mpLineGetV1Pos(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                        if (f28 <= pos.y) && (pos.y <= f29) {
                            x = fmadds(f27, pos.y - f28, (*coll).ecb.bottom.x);
                        } else if (f29 <= pos.y) && (pos.y <= f30) {
                            x = fmadds(f26, pos.y - f30, (*coll).ecb.top.x);
                        } else if pos.y > f30 {
                            break 'l30;
                        } else {
                            break 'c30;
                        }
                        x = pos.x - x;
                        if mpColl_804D6490_max_x > x {
                            flags = super::mplib::mpLineGetFlags(wall_id);
                            let _ = super::mplib::mpLineGetNormal(wall_id, core::ptr::addr_of_mut!(normal) as *mut Vec);
                            mpColl_804D6490_max_x = x;
                            mpColl_804D6494_line_id = wall_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    wall_id = super::mplib::mpLineGetNext(wall_id);
                }
            }
            arr = arr.offset(1); i = i + 1;
        }
    }
    line_id = mpColl_804D6494_line_id;
    flags = mpColl_804D6498_flags;
    normal = mpColl_80458810.normal;
    if (*coll).cur_pos.x > mpColl_804D6490_max_x {
        (*coll).cur_pos.x = mpColl_804D6490_max_x;
        (*coll).left_facing_wall.index = line_id;
        (*coll).left_facing_wall.flags = flags;
        (*coll).left_facing_wall.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpCollCeilingInline`
unsafe fn mpCollCeilingInline(mut coll: *mut CollData) {
    let mut right_wall_id: i32 = 0;
    let mut left_wall_id: i32 = 0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut side_x: f32 = 0.0;
    let mut side_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    let mut non_ceiling_id: i32 = super::mplib::mpLineNextNonCeiling((*coll).ceiling.index);
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    side_x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    side_y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    if (super::mplib::mpCheckLeftWall(top_x, top_y, side_x, side_y, null_mut(), core::ptr::addr_of_mut!(left_wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (left_wall_id != non_ceiling_id) {
        hit_wall = 1;
    } else {
        hit_wall = 0;
    }
    if hit_wall != 0 {
        mpColl_800439FC(coll);
    }
    non_ceiling_id = super::mplib::mpLinePrevNonCeiling((*coll).ceiling.index);
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    side_x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    side_y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    if (super::mplib::mpCheckRightWall(top_x, top_y, side_x, side_y, null_mut(), core::ptr::addr_of_mut!(right_wall_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (right_wall_id != non_ceiling_id) {
        hit_wall = 1;
    } else {
        hit_wall = 0;
    }
    if hit_wall != 0 {
        mpColl_80043ADC(coll);
    }
}

/// `mpCollFloorInline`
unsafe fn mpCollFloorInline(mut coll: *mut CollData, mut ecb_unlocked: i32, mut squeeze_flags: u32) {
    let mut wall_id: i32 = 0;
    if mpColl_80043BBC(coll, core::ptr::addr_of_mut!(wall_id) as *mut i32) != 0 {
        mpColl_80043C6C(coll, wall_id, ((ecb_unlocked != 0) && (!((squeeze_flags & (1 as u32)) != 0))) as i32);
    }
    if mpColl_80043E90(coll, core::ptr::addr_of_mut!(wall_id) as *mut i32) != 0 {
        mpColl_80043F40(coll, wall_id, ((ecb_unlocked != 0) && (!((squeeze_flags & (1 as u32)) != 0))) as i32);
    }
}

/// `mpColl_80046904`
pub unsafe extern "C" fn mpColl_80046904(mut coll: *mut CollData, mut flags: u32) -> i32 {
    melee_core::ct!("mpColl_80046904");
    let mut prev_b6: i32 = 0;
    let mut squeeze_flags: i32 = 0;
    let mut old_squeeze_flags: i32 = 0;
    let mut squeeze_flags_all: i32 = 0;
    let mut dead: u32 = 0;
    let mut fl: u32 = flags | { dead = 0 as u32; dead };
    let mut c: *mut CollData = coll;
    let mut platform_pass: i32 = 0;
    let mut stay_airborne: i32 = 0;
    let mut left_right_flags: i32 = 0;
    let mut touched_floor: i32 = 0;
    let mut _wpad: [u8; 8] = [0; 8];
    let mut wid_floorA: i32 = 0;
    let mut wid_floorB: i32 = 0;
    platform_pass = (fl & (2 as u32)) as i32;
    stay_airborne = (fl & (1 as u32)) as i32;
    left_right_flags = 0;
    touched_floor = 0;
    squeeze_flags_all = 0;
    squeeze_flags = 0;
    'l31: loop {
        let mut r3: i32 = 0;
        let mut x_after_collide_right: f32 = 0.0;
        let mut x_after_collide_left: f32 = 0.0;
        let mut y_after_collide_floor: f32 = 0.0;
        let mut y_after_collide_ceiling: f32 = 0.0;
        x_after_collide_right = 0.0f32;
        old_squeeze_flags = squeeze_flags;
        x_after_collide_left = 0.0f32;
        prev_b6 = (*c).x34_flags.b6() as i32;
        squeeze_flags = 0;
        if mpColl_80045B74_LeftWall(c) != 0 {
            if mpColl_80046224_LeftWall(c) != 0 {
                left_right_flags = left_right_flags | 1;
                squeeze_flags = squeeze_flags | 8;
            }
            x_after_collide_left = (*c).cur_pos.x;
        }
        if mpColl_80044E10_RightWall(c) != 0 {
            if mpColl_800454A4_RightWall(c) != 0 {
                left_right_flags = left_right_flags | 2;
                squeeze_flags = squeeze_flags | 4;
            }
            x_after_collide_right = (*c).cur_pos.x;
        }
        if mpColl_80045B74_LeftWall(c) != 0 {
            if mpColl_80046224_LeftWall(c) != 0 {
                left_right_flags = left_right_flags | 1;
                squeeze_flags = squeeze_flags | 8;
            }
            x_after_collide_left = (*c).cur_pos.x;
        }
        if mpColl_80044E10_RightWall(c) != 0 {
            if mpColl_800454A4_RightWall(c) != 0 {
                left_right_flags = left_right_flags | 2;
                squeeze_flags = squeeze_flags | 4;
            }
            x_after_collide_right = (*c).cur_pos.x;
        }
        if (squeeze_flags & 12) == 12 {
            mpCollSqueezeHorizontal(c, 1, x_after_collide_right, x_after_collide_left);
        }
        y_after_collide_ceiling = 0.0f32;
        y_after_collide_floor = 0.0f32;
        if (mpColl_80044AD8_Ceiling(c, left_right_flags) != 0) && (mpColl_80044C74_Ceiling(c) != 0) {
            mpCollCeilingInline(c);
            squeeze_flags = squeeze_flags | 1;
            y_after_collide_ceiling = (*c).cur_pos.y;
        }
        if platform_pass != 0 {
            r3 = mpColl_80044628_Floor(c, mpColl_804D64A0, mpColl_804D64A4, left_right_flags);
        } else {
            r3 = mpColl_80044628_Floor(c, None, null_mut(), left_right_flags);
        }
        if r3 != 0 {
            if stay_airborne != 0 {
                if mpColl_80044948_Floor(c) != 0 {
                    if mpColl_80043BBC(c, core::ptr::addr_of_mut!(wid_floorA) as *mut i32) != 0 {
                        mpColl_80043C6C(c, wid_floorA, ((0 != 0) && (!((squeeze_flags & 1) != 0))) as i32);
                    }
                    if mpColl_80043E90(c, core::ptr::addr_of_mut!(wid_floorA) as *mut i32) != 0 {
                        mpColl_80043F40(c, wid_floorA, ((0 != 0) && (!((squeeze_flags & 1) != 0))) as i32);
                    }
                }
            } else {
                let mut ecb_unlocked: i32 = 0;
                if (*c).ecb.bottom.y > 0.0f32 {
                    ecb_unlocked = 1;
                }
                if mpColl_80044838_Floor(c, ((ecb_unlocked != 0) && (!((squeeze_flags & 1) != 0))) as i32) != 0 {
                    if mpColl_80043BBC(c, core::ptr::addr_of_mut!(wid_floorB) as *mut i32) != 0 {
                        mpColl_80043C6C(c, wid_floorB, ((ecb_unlocked != 0) && (!((squeeze_flags & 1) != 0))) as i32);
                    }
                    if mpColl_80043E90(c, core::ptr::addr_of_mut!(wid_floorB) as *mut i32) != 0 {
                        mpColl_80043F40(c, wid_floorB, ((ecb_unlocked != 0) && (!((squeeze_flags & 1) != 0))) as i32);
                    }
                    (*c).x34_flags.set_b5((1 as u8) as u8);
                    touched_floor = 1;
                }
            }
            y_after_collide_floor = (*c).cur_pos.y;
            squeeze_flags = squeeze_flags | 2;
            if (mpColl_80044AD8_Ceiling(c, left_right_flags) != 0) && (mpColl_80044C74_Ceiling(c) != 0) {
                mpCollCeilingInline(c);
                squeeze_flags = squeeze_flags | 1;
                y_after_collide_ceiling = (*c).cur_pos.y;
            }
        }
        if (squeeze_flags & 3) == 3 {
            let mut airborne: i32 = 0;
            if touched_floor != 0 {
                airborne = 0;
            } else {
                airborne = 1;
            }
            mpCollSqueezeVertical(c, airborne, y_after_collide_ceiling, y_after_collide_floor);
        }
        squeeze_flags_all = squeeze_flags_all | squeeze_flags;
        if !((prev_b6 != ((*c).x34_flags.b6() as i32)) || (squeeze_flags != old_squeeze_flags)) { break 'l31; }
    }
    if (!(touched_floor != 0)) && ((fl & (4 as u32)) != 0) {
        let mut on_edge: i32 = ((((*c).env_flags & 1048576) != 0) || (((*c).env_flags & 2097152) != 0)) as i32;
        if (!(on_edge != 0)) && (((*c).cur_pos.y < (*c).prev_pos.y) || ((mpColl_LedgeLevelOK != 0) && ((*c).cur_pos.y <= (*c).prev_pos.y))) {
            if (((*c).facing_dir as i32) == 1) || (((*c).facing_dir as i32) == 0) {
                if mpColl_80044164(c, core::ptr::addr_of_mut!((*c).ledge_id_left) as *mut i32) != 0 {
                    on_edge = 1;
                    (*c).env_flags = (*c).env_flags | 16777216;
                } else {
                    on_edge = 0;
                }
                if on_edge != 0 {
                    (*c).env_flags = (*c).env_flags | 16777216;
                }
            }
            if (((*c).facing_dir as i32) == (-1_i32)) || (((*c).facing_dir as i32) == 0) {
                if mpColl_800443C4(c, core::ptr::addr_of_mut!((*c).ledge_id_right) as *mut i32) != 0 {
                    on_edge = 1;
                    (*c).env_flags = (*c).env_flags | 33554432;
                } else {
                    on_edge = 0;
                }
                if on_edge != 0 {
                    (*c).env_flags = (*c).env_flags | 33554432;
                }
            }
        }
    }
    if !((squeeze_flags_all & 8) != 0) {
        (*c).env_flags = (*c).env_flags & (!63);
    }
    if !((squeeze_flags_all & 4) != 0) {
        (*c).env_flags = (*c).env_flags & (!4032);
    }
    return touched_floor;
}

/// `mpColl_80046F78_inline`
unsafe fn mpColl_80046F78_inline(mut coll: *mut CollData, mut line_id_out: *mut i32) -> i32 {
    if (*coll).x38 != mpColl_804D64AC {
        let mut prev_x: f32 = (*coll).prev_pos.x;
        let mut prev_y: f32 = (*coll).prev_pos.y;
        let mut x: f32 = (*coll).cur_pos.x;
        let mut y: f32 = (*coll).cur_pos.y;
        return super::mplib::mpCheckAllRemap(core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only, prev_x, prev_y, x, y);
    } else {
        let mut prev_x: f32 = (*coll).prev_pos.x;
        let mut prev_y: f32 = (*coll).prev_pos.y;
        let mut x: f32 = (*coll).cur_pos.x;
        let mut y: f32 = (*coll).cur_pos.y;
        return super::mplib::mpCheckAll(core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, line_id_out, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only, prev_x, prev_y, x, y);
    }
}

/// `mpColl_80046F78`
pub unsafe extern "C" fn mpColl_80046F78(mut coll: *mut CollData, mut __: u32) -> i32 {
    melee_core::ct!("mpColl_80046F78");
    let mut line_id: i32 = 0;
    let mut y: f32 = 0.0;
    if mpColl_80046F78_inline(coll, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        let mut kind: i32 = super::mplib::mpLineGetKind(line_id);
        if kind == (CollLine_Floor as i32) {
            line_id = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).floor.index = line_id;
                (*coll).cur_pos.x = (*coll).contact.x;
                (*coll).cur_pos.y = (*coll).contact.y + y;
                (*coll).cur_pos.z = (*coll).contact.z;
                (*coll).env_flags = (*coll).env_flags | 32768;
                return 1;
            }
            return 0;
        } else if kind == (CollLine_Ceiling as i32) {
            line_id = super::mplib::mpLib_8004E090_Ceiling(line_id, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).ceiling.index = line_id;
                (*coll).cur_pos.x = (*coll).contact.x;
                (*coll).cur_pos.y = (*coll).contact.y + y;
                (*coll).cur_pos.z = (*coll).contact.z;
                (*coll).env_flags = (*coll).env_flags | 8192;
                return 1;
            }
            return 0;
        } else if kind == (CollLine_LeftWall as i32) {
            line_id = super::mplib::mpLib_8004E398_LeftWall(line_id, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).left_facing_wall.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).left_facing_wall.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).left_facing_wall.index = line_id;
                (*coll).cur_pos.x = (*coll).contact.x + y;
                (*coll).cur_pos.y = (*coll).contact.y;
                (*coll).cur_pos.z = (*coll).contact.z;
                (*coll).env_flags = (*coll).env_flags | 1;
                return 1;
            }
            return 0;
        } else if kind == (CollLine_RightWall as i32) {
            line_id = super::mplib::mpLib_8004E684_RightWall(line_id, core::ptr::addr_of_mut!((*coll).contact) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).right_facing_wall.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).right_facing_wall.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).right_facing_wall.index = line_id;
                (*coll).cur_pos.x = (*coll).contact.x + y;
                (*coll).cur_pos.y = (*coll).contact.y;
                (*coll).cur_pos.z = (*coll).contact.z;
                (*coll).env_flags = (*coll).env_flags | 64;
                return 1;
            }
            return 0;
        } else {
            if 0 != 0 { (); } else { panic!("mpColl_80046F78: assertion failed"); }
        }
    }
    return 0;
}

/// `inline0`
unsafe fn inline0(mut coll: *mut CollData, mut i: i32, mut j: i32) -> i32 {
    let mut result: i32 = 0;
    (*coll).prev_env_flags = (*coll).env_flags;
    (*coll).env_flags = 0;
    if (((*coll).ecb.top.y - (*coll).ecb.bottom.y) < 6.0f32) && (((*coll).ecb.right.y - (*coll).ecb.left.y) < 6.0f32) {
        mpColl_IsEcbTiny = 1;
    } else {
        mpColl_IsEcbTiny = 0;
    }
    result = mpColl_80043754(Some(mpColl_80046904), coll, i as u32);
    mpCollEnd(coll, result, j);
    return result;
}

/// `inline4`
unsafe fn inline4(mut coll: *mut CollData, mut i: i32) -> i32 {
    let mut result: i32 = 0;
    (*coll).prev_env_flags = (*coll).env_flags;
    (*coll).env_flags = 0;
    if (((*coll).ecb.top.y - (*coll).ecb.bottom.y) < 6.0f32) && (((*coll).ecb.right.y - (*coll).ecb.left.y) < 6.0f32) {
        mpColl_IsEcbTiny = 1;
    } else {
        mpColl_IsEcbTiny = 0;
    }
    result = mpColl_80043754(Some(mpColl_80046F78), coll, i as u32);
    mpCollEnd(coll, result, 1);
    return result;
}

/// `inline2`
unsafe fn inline2(mut coll: *mut CollData, mut i: i32) -> i32 {
    let mut result: i32 = 0;
    (*coll).prev_env_flags = (*coll).env_flags;
    (*coll).env_flags = 0;
    if (((*coll).ecb.top.y - (*coll).ecb.bottom.y) < 6.0f32) && (((*coll).ecb.right.y - (*coll).ecb.left.y) < 6.0f32) {
        mpColl_IsEcbTiny = 1;
    } else {
        mpColl_IsEcbTiny = 0;
    }
    result = mpColl_80043754(core::mem::transmute::<*mut c_void, Option<unsafe extern "C" fn(*mut CollData, u32) -> i32>>(core::mem::transmute::<Option<unsafe extern "C" fn(*mut CollData, i32) -> i32>, *mut c_void>(Some(mpColl_8004ACE4))), coll, i as u32);
    mpCollEnd(coll, result, 0);
    return result;
}

/// `inline3`
unsafe fn inline3(mut coll: *mut CollData, mut i: i32) -> i32 {
    let mut result: i32 = 0;
    (*coll).prev_env_flags = (*coll).env_flags;
    (*coll).env_flags = 0;
    if (((*coll).ecb.top.y - (*coll).ecb.bottom.y) < 6.0f32) && (((*coll).ecb.right.y - (*coll).ecb.left.y) < 6.0f32) {
        mpColl_IsEcbTiny = 1;
    } else {
        mpColl_IsEcbTiny = 0;
    }
    result = mpColl_80043754(Some(mpColl_8004C534), coll, i as u32);
    mpCollEnd(coll, result, 0);
    return result;
}

/// `inline1`
unsafe fn inline1(mut coll: *mut CollData, mut i: i32, mut floor_cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut gobj: *mut HSD_GObj) -> i32 {
    let mut result: i32 = 0;
    (*coll).prev_env_flags = (*coll).env_flags;
    (*coll).env_flags = 0;
    if (((*coll).ecb.top.y - (*coll).ecb.bottom.y) < 6.0f32) && (((*coll).ecb.right.y - (*coll).ecb.left.y) < 6.0f32) {
        mpColl_IsEcbTiny = 1;
    } else {
        mpColl_IsEcbTiny = 0;
    }
    mpColl_804D64A0 = floor_cb;
    mpColl_804D64A4 = gobj;
    result = mpColl_80043754(Some(mpColl_80046904), coll, i as u32);
    mpCollEnd(coll, result, 1);
    return result;
}

/// `mpColl_800471F8`
pub unsafe fn mpColl_800471F8(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800471F8");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline0(coll, 0, 1);
}

/// `mpColl_8004730C`
pub unsafe fn mpColl_8004730C(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) -> i32 {
    melee_core::ct!("mpColl_8004730C");
    mpCollPrev(coll);
    mpColl_80042C58(coll, arg1);
    return inline0(coll, 0, 1);
}

/// `mpColl_800473CC`
pub unsafe fn mpColl_800473CC(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800473CC");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline0(coll, 4, 1);
}

/// `mpColl_800474E0`
pub unsafe fn mpColl_800474E0(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800474E0");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline0(coll, 4, 1);
}

/// `mpColl_800475F4`
pub unsafe fn mpColl_800475F4(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) -> i32 {
    melee_core::ct!("mpColl_800475F4");
    mpCollPrev(coll);
    mpColl_80042C58(coll, arg1);
    return inline0(coll, 4, 1);
}

/// `mpColl_800476B4`
pub unsafe fn mpColl_800476B4(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_800476B4");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline1(coll, 3, arg1, gobj);
}

/// `mpColl_800477E0`
pub unsafe fn mpColl_800477E0(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800477E0");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline0(coll, 1, 1);
}

/// `mpColl_800478F4`
pub unsafe fn mpColl_800478F4(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800478F4");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline0(coll, 1, 1);
}

/// `mpColl_80047A08`
pub unsafe fn mpColl_80047A08(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) -> i32 {
    melee_core::ct!("mpColl_80047A08");
    mpCollPrev(coll);
    mpColl_80042C58(coll, arg1);
    return inline0(coll, 1, 1);
}

/// `mpColl_80047AC8`
pub unsafe fn mpColl_80047AC8(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_80047AC8");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline1(coll, 2, arg1, arg2);
}

/// `mpColl_80047BF4`
pub unsafe fn mpColl_80047BF4(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_80047BF4");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 10);
    return inline1(coll, 2, arg1, arg2);
}

/// `mpColl_80047D20`
pub unsafe fn mpColl_80047D20(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_80047D20");
    mpCollPrev(coll);
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 18 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    mpColl_80042384(coll);
    return inline1(coll, 2, arg1, arg2);
}

/// `mpColl_80047E14`
pub unsafe fn mpColl_80047E14(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_80047E14");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 6);
    return inline1(coll, 6, arg1, arg2);
}

/// `mpColl_80047F40`
pub unsafe fn mpColl_80047F40(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_80047F40");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 10);
    return inline1(coll, 6, arg1, arg2);
}

/// `mpColl_8004806C`
pub unsafe fn mpColl_8004806C(mut coll: *mut CollData, mut arg1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, mut arg2: *mut HSD_GObj) -> i32 {
    melee_core::ct!("mpColl_8004806C");
    mpCollPrev(coll);
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 18 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    mpColl_80042384(coll);
    return inline1(coll, 6, arg1, arg2);
}

/// `mpColl_80048160`
pub unsafe fn mpColl_80048160(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048160");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 10);
    return inline0(coll, 0, 1);
}

/// `mpColl_80048274`
pub unsafe fn mpColl_80048274(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048274");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 10);
    return inline0(coll, 1, 1);
}

/// `mpColl_80048388`
pub unsafe fn mpColl_80048388(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048388");
    mpCollPrev(coll);
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 18 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    mpColl_80042384(coll);
    return inline0(coll, 1, 1);
}

/// `mpColl_80048464`
pub unsafe fn mpColl_80048464(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048464");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 10);
    return inline0(coll, 4, 1);
}

/// `mpColl_80048578`
pub unsafe fn mpColl_80048578(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048578");
    mpCollPrev(coll);
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 18 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    mpColl_80042384(coll);
    return inline0(coll, 4, 1);
}

/// `mpColl_80048654`
pub unsafe fn mpColl_80048654(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048654");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline0(coll, 0, 1);
}

/// `mpColl_80048768`
pub unsafe fn mpColl_80048768(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048768");
    mpCollPrev(coll);
    if (*coll).ecb_source.kind == ((ECBSource_JObj as i32) as u32) {
        mpColl_LoadECB_JObj(coll, 18 as u32);
    } else {
        mpColl_LoadECB_Fixed(coll);
    }
    mpColl_80042384(coll);
    return inline0(coll, 0, 1);
}

/// `mpColl_80048844`
pub unsafe fn mpColl_80048844(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048844");
    mpCollPrev(coll);
    return inline4(coll, 0);
}

/// `mpColl_800488F4`
pub unsafe fn mpColl_800488F4(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800488F4");
    let mut bottom: Vec3 = unsafe { core::mem::zeroed() };
    let mut __: [u8; 12] = [0; 12];
    let mut y: f32 = 0.0;
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_id: i32 = 0;
    let mut line_id: i32 = 0;
    let mut hit_wall: i32 = 0;
    floor_id = (*coll).floor.index;
    bottom.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if (!(super::mplib::mpLib_80054ED8((*coll).floor.index) != 0)) || (super::mplib::mpLineGetKind((*coll).floor.index) != (CollLine_Floor as i32)) {
        return 0;
    }
    line_id = super::mplib::mpLib_8004DD90_Floor(floor_id, core::ptr::addr_of_mut!(bottom) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).floor.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).floor.normal) as *mut Vec);
    if line_id != (-1_i32) {
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
        (*coll).floor.index = line_id;
        return 1;
    }
    hit_wall = 0;
    super::mplib::mpFloorGetLeft(floor_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
    if (*coll).cur_pos.x < edge.x {
        let mut non_floor_id: i32 = super::mplib::mpLinePrevNonFloor(floor_id);
        if ((non_floor_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(non_floor_id) != 0)) && ((super::mplib::mpLineGetKind(non_floor_id) & (CollLine_RightWall as i32)) != 0) {
            hit_wall = 1;
        } else {
            (*coll).env_flags = (*coll).env_flags | 268435456;
        }
    } else {
        super::mplib::mpFloorGetRight(floor_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if (*coll).cur_pos.x > edge.x {
            let mut non_floor_id: i32 = super::mplib::mpLineNextNonFloor(floor_id);
            if ((non_floor_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(non_floor_id) != 0)) && ((super::mplib::mpLineGetKind(non_floor_id) & (CollLine_LeftWall as i32)) != 0) {
                hit_wall = 1;
            } else {
                (*coll).env_flags = (*coll).env_flags | 536870912;
            }
        }
    }
    if hit_wall != 0 {
        (*coll).cur_pos = edge;
        return 1;
    }
    return 0;
}

/// `mpColl_80048AB0_RightWall`
pub unsafe fn mpColl_80048AB0_RightWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80048AB0_RightWall");
    let mut line_id: i32 = 0;
    let mut left_x: f32 = 0.0;
    let mut left_y: f32 = 0.0;
    let mut prev_left_x: f32 = 0.0;
    let mut prev_left_y: f32 = 0.0;
    let mut vertical_y: f32 = 0.0;
    let mut vertical_x: f32 = 0.0;
    let mut prev_x: f32 = 0.0;
    let mut prev_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    let mut line_id1: i32 = 0;
    let mut line_id2: i32 = 0;
    hit_wall = 0;
    mpColl_804D6488 = 0;
    if super::mplib::mpLib_80054ED8((*coll).floor.index) != 0 {
        let mut temp: i32 = 0;
        line_id1 = super::mplib::mpLib_80053394_Floor((*coll).floor.index);
        temp = super::mplib::mpLib_800536CC_Floor((*coll).floor.index);
        line_id = temp;
        line_id2 = if temp != (-1_i32) { super::mplib::mpLib_80053394_Floor(line_id) } else { (-1_i32) };
    } else {
        line_id2 = (-1_i32);
        line_id1 = (-1_i32);
    }
    left_x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    left_y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.left.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.left.y;
    if mpColl_RightWall_inline2(coll, prev_x, prev_y, left_x, left_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 2048;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if ((mpColl_RightWall_inline2(coll, prev_x, prev_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if mpColl_RightWall_inline2(coll, prev_x, prev_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if ((super::mplib::mpCheckRightWall(vertical_x, vertical_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_bottom_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
        let mut prev_bottom_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
        prev_left_x = (*coll).prev_pos.x + (*coll).prev_ecb.left.x;
        prev_left_y = (*coll).prev_pos.y + (*coll).prev_ecb.left.y;
        if ((super::mplib::mpLib_800511A4_RightWall(prev_bottom_x, prev_bottom_y, prev_left_x, prev_left_y, vertical_x, vertical_y, left_x, left_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
            mpColl_RightWall_inline(line_id);
            hit_wall = 1;
        }
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpCheckRightWall(vertical_x, vertical_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_top_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
        let mut prev_top_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
        if super::mplib::mpLib_800511A4_RightWall(prev_left_x, prev_left_y, prev_top_x, prev_top_y, left_x, left_y, vertical_x, vertical_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_RightWall_inline(line_id);
            hit_wall = 1;
        }
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 64;
    }
    return hit_wall;
}

/// `mpColl_800491C8_RightWall`
pub unsafe fn mpColl_800491C8_RightWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_800491C8_RightWall");
    let mut flags: u32 = 0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut i: i32 = 0;
    mpColl_804D6490_max_x = -f32::MAX;
    {
        i = 0;
        'l32: while i < mpColl_804D6488 {
            'c32: {
                let mut top: f32 = 0.0;
                let mut mid: f32 = 0.0;
                let mut bot: f32 = 0.0;
                let mut bot_y_to_x: f32 = 0.0;
                let mut top_y_to_x: f32 = 0.0;
                let mut bottom_y: f32 = 0.0;
                let mut left_y: f32 = 0.0;
                let mut top_y: f32 = 0.0;
                let mut left_x: f32 = 0.0;
                let mut j: i32 = 0;
                let mut wall_id: i32 = 0;
                let mut x: f32 = 0.0;
                wall_id = mpColl_80458810.right[i as usize];
                super::mplib::mpRightWallGetTop(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y < ((*coll).cur_pos.y + (*coll).ecb.bottom.y) {
                    if mpColl_804D6490_max_x < pos.x {
                        let mut r3: i32 = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if r3 != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = r3;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c32;
                }
                super::mplib::mpRightWallGetBottom(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y > ((*coll).cur_pos.y + (*coll).ecb.top.y) {
                    if mpColl_804D6490_max_x < pos.x {
                        let mut r3: i32 = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if r3 != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = r3;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c32;
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.left.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.left.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
                line_id = super::mplib::mpLib_8004E684_RightWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x < ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                left_x = (*coll).ecb.left.x;
                top_y = (*coll).ecb.top.y;
                left_y = (*coll).ecb.left.y;
                bottom_y = (*coll).ecb.bottom.y;
                top = (*coll).cur_pos.y + top_y;
                mid = (*coll).cur_pos.y + left_y;
                bot = (*coll).cur_pos.y + bottom_y;
                bot_y_to_x = left_x / (left_y - bottom_y);
                top_y_to_x = left_x / (left_y - top_y);
                {
                    j = wall_id;
                    'l33: while (j != (-1_i32)) && ((super::mplib::mpLineGetKind(j) & 15) == (CollLine_RightWall as i32)) {
                        'c33: {
                            super::mplib::mpLineGetV1Pos(j, core::ptr::addr_of_mut!(pos) as *mut Vec);
                            if (bot <= pos.y) && (pos.y <= mid) {
                                x = fmadds(bot_y_to_x, pos.y - bot, (*coll).ecb.bottom.x);
                            } else if (mid <= pos.y) && (pos.y <= top) {
                                x = fmadds(top_y_to_x, pos.y - top, (*coll).ecb.top.x);
                            } else if pos.y < bot {
                                break 'l33;
                            } else {
                                break 'c33;
                            }
                            x = pos.x - x;
                            if mpColl_804D6490_max_x < x {
                                flags = super::mplib::mpLineGetFlags(j);
                                let _ = super::mplib::mpLineGetNormal(j, core::ptr::addr_of_mut!(normal) as *mut Vec);
                                mpColl_804D6490_max_x = x;
                                mpColl_804D6494_line_id = j;
                                mpColl_804D6498_flags = flags;
                                mpColl_80458810.normal = normal;
                            }
                        }
                        j = super::mplib::mpLineGetNext(j);
                    }
                }
                'l34: while (wall_id != (-1_i32)) && ((super::mplib::mpLineGetKind(wall_id) & 15) == (CollLine_RightWall as i32)) {
                    'c34: {
                        super::mplib::mpLineGetV0Pos(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                        if (bot <= pos.y) && (pos.y <= mid) {
                            x = fmadds(bot_y_to_x, pos.y - bot, (*coll).ecb.bottom.x);
                        } else if (mid <= pos.y) && (pos.y <= top) {
                            x = fmadds(top_y_to_x, pos.y - top, (*coll).ecb.top.x);
                        } else if pos.y > top {
                            break 'l34;
                        } else {
                            break 'c34;
                        }
                        x = pos.x - x;
                        if mpColl_804D6490_max_x < x {
                            flags = super::mplib::mpLineGetFlags(wall_id);
                            let _ = super::mplib::mpLineGetNormal(wall_id, core::ptr::addr_of_mut!(normal) as *mut Vec);
                            mpColl_804D6490_max_x = x;
                            mpColl_804D6494_line_id = wall_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    wall_id = super::mplib::mpLineGetPrev(wall_id);
                }
            }
            i = i + 1;
        }
    }
    line_id = mpColl_804D6494_line_id;
    flags = mpColl_804D6498_flags;
    normal = mpColl_80458810.normal;
    if (*coll).cur_pos.x < mpColl_804D6490_max_x {
        (*coll).cur_pos.x = mpColl_804D6490_max_x;
        (*coll).right_facing_wall.index = line_id;
        (*coll).right_facing_wall.flags = flags;
        (*coll).right_facing_wall.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_80049778_LeftWall`
pub unsafe fn mpColl_80049778_LeftWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80049778_LeftWall");
    let mut line_id: i32 = 0;
    let mut right_x: f32 = 0.0;
    let mut right_y: f32 = 0.0;
    let mut prev_right_x: f32 = 0.0;
    let mut prev_right_y: f32 = 0.0;
    let mut vertical_y: f32 = 0.0;
    let mut vertical_x: f32 = 0.0;
    let mut prev_x: f32 = 0.0;
    let mut prev_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    let mut line_id1: i32 = 0;
    let mut line_id2: i32 = 0;
    hit_wall = 0;
    mpColl_804D648C = 0;
    if super::mplib::mpLib_80054ED8((*coll).floor.index) != 0 {
        let mut temp: i32 = 0;
        line_id1 = super::mplib::mpLib_80053448_Floor((*coll).floor.index);
        temp = super::mplib::mpLib_800534FC_Floor((*coll).floor.index);
        line_id = temp;
        line_id2 = if temp != (-1_i32) { super::mplib::mpLib_80053448_Floor(line_id) } else { (-1_i32) };
    } else {
        line_id2 = (-1_i32);
        line_id1 = (-1_i32);
    }
    right_x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    right_y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.right.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.right.y;
    if mpColl_LeftWall_inline2(coll, prev_x, prev_y, right_x, right_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline3(line_id, mpColl_80458810.left.as_mut_ptr());
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 32;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if ((mpColl_LeftWall_inline2(coll, prev_x, prev_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    prev_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if mpColl_LeftWall_inline2(coll, prev_x, prev_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if ((super::mplib::mpCheckLeftWall(vertical_x, vertical_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_bottom_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
        let mut prev_bottom_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
        prev_right_x = (*coll).prev_pos.x + (*coll).prev_ecb.right.x;
        prev_right_y = (*coll).prev_pos.y + (*coll).prev_ecb.right.y;
        if ((super::mplib::mpLib_800515A0_LeftWall(prev_right_x, prev_right_y, prev_bottom_x, prev_bottom_y, right_x, right_y, vertical_x, vertical_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
            mpColl_LeftWall_inline(line_id);
            hit_wall = 1;
        }
    }
    vertical_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    vertical_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if super::mplib::mpCheckLeftWall(vertical_x, vertical_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    if !(mpColl_IsEcbTiny != 0) {
        let mut prev_top_x: f32 = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
        let mut prev_top_y: f32 = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
        if super::mplib::mpLib_800515A0_LeftWall(prev_top_x, prev_top_y, prev_right_x, prev_right_y, vertical_x, vertical_y, right_x, right_y, core::ptr::addr_of_mut!(line_id) as *mut i32, (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
            mpColl_LeftWall_inline(line_id);
            hit_wall = 1;
        }
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 1;
    }
    return hit_wall;
}

/// `mpColl_80049EAC_LeftWall`
pub unsafe fn mpColl_80049EAC_LeftWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_80049EAC_LeftWall");
    let mut flags: u32 = 0;
    let mut pos: Vec3 = unsafe { core::mem::zeroed() };
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut arr: *mut i32 = mpColl_80458810.left.as_mut_ptr();
    let mut i: i32 = 0;
    mpColl_804D6490_max_x = f32::MAX;
    {
        i = 0;
        'l35: while i < mpColl_804D648C {
            'c35: {
                let mut top: f32 = 0.0;
                let mut mid: f32 = 0.0;
                let mut bot: f32 = 0.0;
                let mut bot_y_to_x: f32 = 0.0;
                let mut top_y_to_x: f32 = 0.0;
                let mut bottom_y: f32 = 0.0;
                let mut right_y: f32 = 0.0;
                let mut top_y: f32 = 0.0;
                let mut right_x: f32 = 0.0;
                let mut j: i32 = 0;
                let mut wall_id: i32 = 0;
                let mut x: f32 = 0.0;
                wall_id = *arr;
                super::mplib::mpLeftWallGetTop(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y < ((*coll).cur_pos.y + (*coll).ecb.bottom.y) {
                    if mpColl_804D6490_max_x > pos.x {
                        let mut r3: i32 = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if r3 != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = r3;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c35;
                }
                super::mplib::mpLeftWallGetBottom(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                if pos.y > ((*coll).cur_pos.y + (*coll).ecb.top.y) {
                    if mpColl_804D6490_max_x > pos.x {
                        let mut r3: i32 = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, null_mut(), core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                        if r3 != (-1_i32) {
                            mpColl_804D6490_max_x = pos.x;
                            mpColl_804D6494_line_id = r3;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    break 'c35;
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.right.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.right.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                pos.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
                pos.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
                line_id = super::mplib::mpLib_8004E398_LeftWall(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec, core::ptr::addr_of_mut!(x) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
                if line_id != (-1_i32) {
                    if mpColl_804D6490_max_x > ((*coll).cur_pos.x + x) {
                        mpColl_804D6490_max_x = (*coll).cur_pos.x + x;
                        mpColl_804D6494_line_id = line_id;
                        mpColl_804D6498_flags = flags;
                        mpColl_80458810.normal = normal;
                    }
                }
                right_x = (*coll).ecb.right.x;
                top_y = (*coll).ecb.top.y;
                right_y = (*coll).ecb.right.y;
                bottom_y = (*coll).ecb.bottom.y;
                top = (*coll).cur_pos.y + top_y;
                mid = (*coll).cur_pos.y + right_y;
                bot = (*coll).cur_pos.y + bottom_y;
                bot_y_to_x = right_x / (right_y - bottom_y);
                top_y_to_x = right_x / (right_y - top_y);
                {
                    j = wall_id;
                    'l36: while (j != (-1_i32)) && ((super::mplib::mpLineGetKind(j) & 15) == (CollLine_LeftWall as i32)) {
                        'c36: {
                            super::mplib::mpLineGetV0Pos(j, core::ptr::addr_of_mut!(pos) as *mut Vec);
                            if (bot <= pos.y) && (pos.y <= mid) {
                                x = fmadds(bot_y_to_x, pos.y - bot, (*coll).ecb.bottom.x);
                            } else if (mid <= pos.y) && (pos.y <= top) {
                                x = fmadds(top_y_to_x, pos.y - top, (*coll).ecb.top.x);
                            } else if pos.y < bot {
                                break 'l36;
                            } else {
                                break 'c36;
                            }
                            x = pos.x - x;
                            if mpColl_804D6490_max_x > x {
                                flags = super::mplib::mpLineGetFlags(j);
                                let _ = super::mplib::mpLineGetNormal(j, core::ptr::addr_of_mut!(normal) as *mut Vec);
                                mpColl_804D6490_max_x = x;
                                mpColl_804D6494_line_id = j;
                                mpColl_804D6498_flags = flags;
                                mpColl_80458810.normal = normal;
                            }
                        }
                        j = super::mplib::mpLineGetPrev(j);
                    }
                }
                'l37: while (wall_id != (-1_i32)) && ((super::mplib::mpLineGetKind(wall_id) & 15) == (CollLine_LeftWall as i32)) {
                    'c37: {
                        super::mplib::mpLineGetV1Pos(wall_id, core::ptr::addr_of_mut!(pos) as *mut Vec);
                        if (bot <= pos.y) && (pos.y <= mid) {
                            x = fmadds(bot_y_to_x, pos.y - bot, (*coll).ecb.bottom.x);
                        } else if (mid <= pos.y) && (pos.y <= top) {
                            x = fmadds(top_y_to_x, pos.y - top, (*coll).ecb.top.x);
                        } else if pos.y > top {
                            break 'l37;
                        } else {
                            break 'c37;
                        }
                        x = pos.x - x;
                        if mpColl_804D6490_max_x > x {
                            flags = super::mplib::mpLineGetFlags(wall_id);
                            let _ = super::mplib::mpLineGetNormal(wall_id, core::ptr::addr_of_mut!(normal) as *mut Vec);
                            mpColl_804D6490_max_x = x;
                            mpColl_804D6494_line_id = wall_id;
                            mpColl_804D6498_flags = flags;
                            mpColl_80458810.normal = normal;
                        }
                    }
                    wall_id = super::mplib::mpLineGetNext(wall_id);
                }
            }
            arr = arr.offset(1); i = i + 1;
        }
    }
    line_id = mpColl_804D6494_line_id;
    flags = mpColl_804D6498_flags;
    normal = mpColl_80458810.normal;
    if (*coll).cur_pos.x > mpColl_804D6490_max_x {
        (*coll).cur_pos.x = mpColl_804D6490_max_x;
        (*coll).left_facing_wall.index = line_id;
        (*coll).left_facing_wall.flags = flags;
        (*coll).left_facing_wall.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_8004A45C_Floor`
pub unsafe fn mpColl_8004A45C_Floor(mut coll: *mut CollData, mut line_id: i32) -> i32 {
    melee_core::ct!("mpColl_8004A45C_Floor");
    let mut flags: u32 = 0;
    let mut y: f32 = 0.0;
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_id: i32 = 0;
    let mut on_edge: i32 = 0;
    on_edge = 0;
    if (!(super::mplib::mpLib_80054ED8(line_id) != 0)) || (super::mplib::mpLineGetKind(line_id) != (CollLine_Floor as i32)) {
        return 0;
    }
    super::mplib::mpFloorGetLeft(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
    if (*coll).cur_pos.x <= edge.x {
        let mut edge_x: f32 = 0.0;
        let mut edge_y: f32 = 0.0;
        let mut right_x: f32 = 0.0;
        let mut right_y: f32 = 0.0;
        floor_id = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
        edge_x = edge.x + 1.0f32;
        edge_y = edge.y + 1.0f32;
        right_x = (edge.x + (*coll).ecb.right.x) - (*coll).ecb.bottom.x;
        right_y = (edge.y + (*coll).ecb.right.y) - (*coll).ecb.bottom.y;
        if !(super::mplib::mpCheckLeftWall(edge_x, edge_y, right_x, right_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
            on_edge = 1;
            (*coll).env_flags = (*coll).env_flags | 2097152;
        }
    } else {
        let mut edge_x: f32 = 0.0;
        let mut edge_y: f32 = 0.0;
        let mut left_x: f32 = 0.0;
        let mut left_y: f32 = 0.0;
        super::mplib::mpFloorGetRight(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if (*coll).cur_pos.x >= edge.x {
            floor_id = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
            edge_x = edge.x - 1.0f32;
            edge_y = edge.y + 1.0f32;
            left_x = (edge.x + (*coll).ecb.left.x) - (*coll).ecb.bottom.x;
            left_y = (edge.y + (*coll).ecb.left.y) - (*coll).ecb.bottom.y;
            if !(super::mplib::mpCheckRightWall(edge_x, edge_y, left_x, left_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
                on_edge = 1;
                (*coll).env_flags = (*coll).env_flags | 1048576;
            }
        }
    }
    if on_edge != 0 {
        (*coll).cur_pos.x = edge.x - (*coll).ecb.bottom.x;
        (*coll).cur_pos.y = edge.y - (*coll).ecb.bottom.y;
        (*coll).cur_pos.z = edge.z;
        (*coll).floor.index = floor_id;
        (*coll).floor.flags = flags;
        (*coll).floor.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_8004A678_Floor`
pub unsafe fn mpColl_8004A678_Floor(mut coll: *mut CollData, mut line_id: i32) -> i32 {
    melee_core::ct!("mpColl_8004A678_Floor");
    let mut flags: u32 = 0;
    let mut y: f32 = 0.0;
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut floor_id: i32 = 0;
    let mut on_edge: i32 = 0;
    if (!(super::mplib::mpLib_80054ED8(line_id) != 0)) || (super::mplib::mpLineGetKind(line_id) != (CollLine_Floor as i32)) {
        return 0;
    }
    super::mplib::mpFloorGetLeft(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
    if (*coll).cur_pos.x <= edge.x {
        if ((!(((*coll).env_flags & 63) != 0)) && (((*coll).facing_dir as i32) == (-1_i32))) && (((*coll).lstick_x as f64) > (-0.75f64)) {
            let mut edge_x: f32 = 0.0;
            let mut edge_y: f32 = 0.0;
            let mut right_x: f32 = 0.0;
            let mut right_y: f32 = 0.0;
            floor_id = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
            if floor_id != (-1_i32) {
                edge.y = edge.y + y;
            }
            edge_x = edge.x + 1.0f32;
            edge_y = edge.y + 1.0f32;
            right_x = (edge.x + (*coll).ecb.right.x) - (*coll).ecb.bottom.x;
            right_y = (edge.y + (*coll).ecb.right.y) - (*coll).ecb.bottom.y;
            if !(super::mplib::mpCheckLeftWall(edge_x, edge_y, right_x, right_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
                on_edge = 1;
            }
        }
    } else {
        super::mplib::mpFloorGetRight(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if ((((*coll).cur_pos.x >= edge.x) && (!(((*coll).env_flags & 4032) != 0))) && (((*coll).facing_dir as i32) == 1)) && (((*coll).lstick_x as f64) < 0.75f64) {
            let mut edge_x: f32 = 0.0;
            let mut edge_y: f32 = 0.0;
            let mut left_x: f32 = 0.0;
            let mut left_y: f32 = 0.0;
            floor_id = super::mplib::mpLib_8004DD90_Floor(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
            if floor_id != (-1_i32) {
                edge.y = edge.y + y;
            }
            edge_x = edge.x - 1.0f32;
            edge_y = edge.y + 1.0f32;
            left_x = (edge.x + (*coll).ecb.left.x) - (*coll).ecb.bottom.x;
            left_y = (edge.y + (*coll).ecb.left.y) - (*coll).ecb.bottom.y;
            if !(super::mplib::mpCheckRightWall(edge_x, edge_y, left_x, left_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
                on_edge = 1;
            }
        }
    }
    if on_edge != 0 {
        (*coll).cur_pos.x = edge.x - (*coll).ecb.bottom.x;
        (*coll).cur_pos.y = edge.y - (*coll).ecb.bottom.y;
        (*coll).cur_pos.z = edge.z;
        (*coll).floor.index = floor_id;
        (*coll).floor.flags = flags;
        (*coll).floor.normal = normal;
        (*coll).env_flags = (*coll).env_flags | 8388608;
        return 1;
    }
    return 0;
}

/// `mpColl_8004A908_Floor`
pub unsafe fn mpColl_8004A908_Floor(mut coll: *mut CollData, mut line_id: i32) -> i32 {
    melee_core::ct!("mpColl_8004A908_Floor");
    let mut floor_id: i32 = 0;
    let mut flags: u32 = 0;
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut prev_bottom_x: f32 = 0.0;
    let mut prev_bottom_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut hit_floor: i32 = 0;
    prev_bottom_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_bottom_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if (*coll).x38 != mpColl_804D64AC {
        hit_floor = super::mplib::mpCheckFloorRemap(prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, 0.0f32, null_mut(), core::ptr::addr_of_mut!(floor_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut());
    } else {
        hit_floor = super::mplib::mpCheckFloor(prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, 0.0f32, null_mut(), core::ptr::addr_of_mut!(floor_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut());
    }
    if (((hit_floor != 0) && (floor_id != (-1_i32))) && (floor_id != line_id)) && ((line_id == (-1_i32)) || (!(super::mplib::mpLinesConnected(floor_id, line_id) != 0))) {
        (*coll).floor.index = floor_id;
        (*coll).floor.flags = flags;
        (*coll).floor.normal = normal;
        return 1;
    }
    prev_bottom_y = fmadds(0.5f32, (*coll).prev_ecb.top.y + (*coll).prev_ecb.bottom.y, (*coll).prev_pos.y);
    if (*coll).x38 != mpColl_804D64AC {
        hit_floor = super::mplib::mpCheckFloorRemap(prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, 0.0f32, null_mut(), core::ptr::addr_of_mut!(floor_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut());
    } else {
        hit_floor = super::mplib::mpCheckFloor(prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, 0.0f32, null_mut(), core::ptr::addr_of_mut!(floor_id) as *mut i32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec, (*coll).floor_skip, (*coll).joint_id_skip, (*coll).joint_id_only, None, null_mut());
    }
    if (((hit_floor != 0) && (floor_id != (-1_i32))) && (floor_id != line_id)) && ((line_id == (-1_i32)) || (!(super::mplib::mpLinesConnected(floor_id, line_id) != 0))) {
        (*coll).floor.index = floor_id;
        (*coll).floor.flags = flags;
        (*coll).floor.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_8004AB80`
pub unsafe fn mpColl_8004AB80(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004AB80");
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    let mut y: f32 = 0.0;
    let mut ceiling_end: Vec3 = unsafe { core::mem::zeroed() };
    let mut line_id: i32 = 0;
    let mut hit_wall: i32 = 0;
    top.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    line_id = super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(top) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
    if line_id != (-1_i32) {
        (*coll).ceiling.index = line_id;
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
    } else {
        hit_wall = 0;
        super::mplib::mpCeilingGetLeft((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
        if top.x <= ceiling_end.x {
            let mut line_id: i32 = super::mplib::mpLineNextNonCeiling((*coll).ceiling.index);
            if (line_id != (-1_i32)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_RightWall as i32)) {
                hit_wall = 1;
            }
        } else {
            let mut line_id: i32 = 0;
            super::mplib::mpCeilingGetRight((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
            line_id = super::mplib::mpLinePrevNonCeiling((*coll).ceiling.index);
            if (line_id != (-1_i32)) && (super::mplib::mpLineGetKind(line_id) == (CollLine_LeftWall as i32)) {
                hit_wall = 1;
            }
        }
        (*coll).cur_pos.y = ceiling_end.y - (*coll).ecb.top.y;
        if hit_wall != 0 {
            let mut line_id: i32 = 0;
            (*coll).cur_pos.x = ceiling_end.x;
            line_id = super::mplib::mpLib_8004E090_Ceiling((*coll).ceiling.index, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec, null_mut(), core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
            if line_id != (-1_i32) {
                (*coll).ceiling.index = line_id;
            } else {
                ext::OSReport(&format!("{}:{}: oioi...\n", core::ffi::CStr::from_ptr((c"src/melee/mp/mpcoll.c".as_ptr() as *mut i8) as *const core::ffi::c_char).to_string_lossy(), 5745));
            }
        }
    }
    return 1;
}

/// `mpColl_8004ACE4`
pub unsafe extern "C" fn mpColl_8004ACE4(mut coll: *mut CollData, mut flags: i32) -> i32 {
    melee_core::ct!("mpColl_8004ACE4");
    let mut touching_floor: i32 = 0;
    let mut left_right: u32 = 0;
    let mut prev_b6: i32 = 0;
    touching_floor = 0;
    left_right = 0 as u32;
    'l38: loop {
        let mut x_after_right: f32 = 0.0;
        let mut x_after_left: f32 = 0.0;
        let mut y_after_ceiling: f32 = 0.0;
        let mut y_after_floor: f32 = 0.0;
        let mut hit_left: i32 = 0;
        let mut hit_right: i32 = 0;
        let mut hit_floor: i32 = 0;
        let mut hit_ceiling: i32 = 0;
        hit_right = 0;
        prev_b6 = (*coll).x34_flags.b6() as i32;
        hit_left = 0;
        if mpColl_80049778_LeftWall(coll) != 0 {
            hit_left = mpColl_80049EAC_LeftWall(coll);
            if hit_left != 0 {
                left_right = left_right | (1 as u32);
            } else {
                (*coll).env_flags = (*coll).env_flags & (!63);
            }
            x_after_left = (*coll).cur_pos.x;
            (*coll).x34_flags.set_b5((1 as u8) as u8);
        }
        if mpColl_80048AB0_RightWall(coll) != 0 {
            hit_right = mpColl_800491C8_RightWall(coll);
            if hit_right != 0 {
                left_right = left_right | (2 as u32);
            } else {
                (*coll).env_flags = (*coll).env_flags & (!4032);
            }
            x_after_right = (*coll).cur_pos.x;
            (*coll).x34_flags.set_b5((1 as u8) as u8);
        }
        if mpColl_80049778_LeftWall(coll) != 0 {
            hit_left = hit_left | mpColl_80049EAC_LeftWall(coll);
            if hit_left != 0 {
                left_right = left_right | (1 as u32);
            }
            x_after_left = (*coll).cur_pos.x;
            (*coll).x34_flags.set_b5((1 as u8) as u8);
        }
        if mpColl_80048AB0_RightWall(coll) != 0 {
            hit_right = hit_right | mpColl_800491C8_RightWall(coll);
            if hit_right != 0 {
                left_right = left_right | (2 as u32);
            }
            x_after_right = (*coll).cur_pos.x;
            (*coll).x34_flags.set_b5((1 as u8) as u8);
        }
        if (hit_left != 0) && (hit_right != 0) {
            mpCollSqueezeHorizontal(coll, 0, x_after_right, x_after_left);
        }
        hit_ceiling = 0;
        hit_floor = 0;
        if (mpColl_80044AD8_Ceiling(coll, left_right as i32) != 0) && (mpColl_8004AB80(coll) != 0) {
            hit_ceiling = 1;
            y_after_ceiling = (*coll).cur_pos.y;
        }
        if mpColl_800488F4(coll) != 0 {
            mpCollFloorInline(coll, 0, 0 as u32);
            y_after_floor = (*coll).cur_pos.y;
            touching_floor = 1;
            hit_floor = 1;
            if (mpColl_80044AD8_Ceiling(coll, left_right as i32) != 0) && (mpColl_8004AB80(coll) != 0) {
                y_after_ceiling = (*coll).cur_pos.y;
                hit_ceiling = 1;
            }
        } else {
            let mut old_skip: i32 = 0;
            let mut var_r23: i32 = 0;
            let mut floor_id: i32 = (*coll).floor.index;
            if (super::mplib::mpLib_80054ED8(floor_id) != 0) && (super::mplib::mpLineGetKind(floor_id) == (CollLine_Floor as i32)) {
                if (flags & 1) != 0 {
                    if mpColl_8004A678_Floor(coll, floor_id) != 0 {
                        (*coll).x34_flags.set_b5((1 as u8) as u8);
                        touching_floor = 0;
                        hit_floor = 1;
                        y_after_floor = (*coll).cur_pos.y;
                    } else {
                        var_r23 = 1;
                    }
                } else if (flags & 2) != 0 {
                    if mpColl_8004A45C_Floor(coll, floor_id) != 0 {
                        (*coll).x34_flags.set_b5((1 as u8) as u8);
                        touching_floor = 1;
                        hit_floor = 1;
                        y_after_floor = (*coll).cur_pos.y;
                    } else {
                        var_r23 = 1;
                    }
                } else {
                    var_r23 = 1;
                }
                if var_r23 != 0 {
                    old_skip = (*coll).floor_skip;
                    (*coll).floor_skip = floor_id;
                    if mpColl_80046904(coll, 0_u32) != 0 {
                        touching_floor = 1;
                    }
                    (*coll).floor_skip = old_skip;
                    (*coll).x34_flags.set_b5((1 as u8) as u8);
                }
            }
        }
        if (hit_floor != 0) && (hit_ceiling != 0) {
            let mut airborne: i32 = 0;
            if touching_floor != 0 {
                airborne = 0;
            } else {
                airborne = 1;
            }
            mpCollSqueezeVertical(coll, airborne, y_after_ceiling, y_after_floor);
        }
        if !(prev_b6 != ((*coll).x34_flags.b6() as i32)) { break 'l38; }
    }
    if mpColl_8004A908_Floor(coll, (*coll).floor.index) != 0 {
        if mpColl_80044838_Floor(coll, 0) != 0 {
            mpCollFloorInline(coll, 0, 0 as u32);
        }
        (*coll).x34_flags.set_b5((0 as u8) as u8);
        touching_floor = 1;
    }
    if touching_floor != 0 {
        (*coll).env_flags = (*coll).env_flags | 32768;
    }
    return touching_floor;
}

/// `mpColl_8004B108`
pub unsafe fn mpColl_8004B108(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B108");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline2(coll, 0);
}

/// `mpColl_8004B21C`
pub unsafe fn mpColl_8004B21C(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) -> i32 {
    melee_core::ct!("mpColl_8004B21C");
    mpCollPrev(coll);
    mpColl_80042C58(coll, arg1);
    return inline2(coll, 0);
}

/// `mpColl_8004B2DC`
pub unsafe fn mpColl_8004B2DC(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B2DC");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline2(coll, 2);
}

/// `mpColl_8004B3F0`
pub unsafe fn mpColl_8004B3F0(mut coll: *mut CollData, mut arg1: *mut ftCollisionBox) -> i32 {
    melee_core::ct!("mpColl_8004B3F0");
    mpCollPrev(coll);
    mpColl_80042C58(coll, arg1);
    return inline2(coll, 2);
}

/// `mpColl_8004B4B0`
pub unsafe fn mpColl_8004B4B0(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B4B0");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline2(coll, 1);
}

/// `mpColl_8004B5C4`
pub unsafe fn mpColl_8004B5C4(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B5C4");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 9);
    return inline2(coll, 1);
}

/// `mpColl_8004B6D8`
pub unsafe fn mpColl_8004B6D8(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B6D8");
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    let mut __: [u8; 12] = [0; 12];
    let mut y: f32 = 0.0;
    let mut ceiling_end: Vec3 = unsafe { core::mem::zeroed() };
    let mut ceiling_id: i32 = 0;
    let mut new_ceiling: i32 = 0;
    let mut hit_wall: i32 = 0;
    ceiling_id = (*coll).ceiling.index;
    top.x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top.y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if (!(super::mplib::mpLib_80054ED8((*coll).ceiling.index) != 0)) || (super::mplib::mpLineGetKind((*coll).ceiling.index) != (CollLine_Ceiling as i32)) {
        return 0;
    }
    new_ceiling = super::mplib::mpLib_8004E090_Ceiling(ceiling_id, core::ptr::addr_of_mut!(top) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!((*coll).ceiling.flags) as *mut u32, core::ptr::addr_of_mut!((*coll).ceiling.normal) as *mut Vec);
    if new_ceiling != (-1_i32) {
        (*coll).cur_pos.y = (*coll).cur_pos.y + y;
        (*coll).ceiling.index = new_ceiling;
        return 1;
    }
    hit_wall = 0;
    super::mplib::mpCeilingGetLeft(ceiling_id, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
    if (*coll).cur_pos.x < ceiling_end.x {
        let mut non_ceiling_id: i32 = super::mplib::mpLineNextNonCeiling(ceiling_id);
        if ((non_ceiling_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(non_ceiling_id) != 0)) && ((super::mplib::mpLineGetKind(non_ceiling_id) & (CollLine_RightWall as i32)) != 0) {
            hit_wall = 1;
        } else {
            (*coll).env_flags = (*coll).env_flags | 268435456;
        }
    } else {
        super::mplib::mpCeilingGetRight(ceiling_id, core::ptr::addr_of_mut!(ceiling_end) as *mut Vec);
        if (*coll).cur_pos.x > ceiling_end.x {
            let mut non_ceiling_id: i32 = super::mplib::mpLinePrevNonCeiling(ceiling_id);
            if ((non_ceiling_id != (-1_i32)) && (super::mplib::mpLib_80054ED8(non_ceiling_id) != 0)) && ((super::mplib::mpLineGetKind(non_ceiling_id) & (CollLine_LeftWall as i32)) != 0) {
                hit_wall = 1;
            } else {
                (*coll).env_flags = (*coll).env_flags | 536870912;
            }
        }
    }
    if hit_wall != 0 {
        (*coll).cur_pos = ceiling_end;
        return 1;
    }
    return 0;
}

/// `mpColl_8004B894_RightWall`
pub unsafe fn mpColl_8004B894_RightWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004B894_RightWall");
    let mut line_id: i32 = 0;
    let mut left_x: f32 = 0.0;
    let mut left_y: f32 = 0.0;
    let mut prev_left_x: f32 = 0.0;
    let mut prev_left_y: f32 = 0.0;
    let mut prev_bottom_x: f32 = 0.0;
    let mut prev_bottom_y: f32 = 0.0;
    let mut prev_top_x: f32 = 0.0;
    let mut prev_top_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    let mut line_id1: i32 = 0;
    let mut line_id2: i32 = 0;
    hit_wall = 0;
    mpColl_804D6488 = 0;
    if super::mplib::mpLib_80054ED8((*coll).ceiling.index) != 0 {
        let mut temp: i32 = 0;
        line_id1 = super::mplib::mpLib_8005389C_Ceiling((*coll).ceiling.index);
        temp = super::mplib::mpLib_80053BD4_Ceiling((*coll).ceiling.index);
        line_id = temp;
        line_id2 = if temp != (-1_i32) { super::mplib::mpLib_8005389C_Ceiling(line_id) } else { (-1_i32) };
    } else {
        line_id2 = (-1_i32);
        line_id1 = (-1_i32);
    }
    left_x = (*coll).cur_pos.x + (*coll).ecb.left.x;
    left_y = (*coll).cur_pos.y + (*coll).ecb.left.y;
    prev_left_x = (*coll).prev_pos.x + (*coll).prev_ecb.left.x;
    prev_left_y = (*coll).prev_pos.y + (*coll).prev_ecb.left.y;
    if mpColl_RightWall_inline2(coll, prev_left_x, prev_left_y, left_x, left_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 2048;
    }
    prev_bottom_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_bottom_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if mpColl_RightWall_inline2(coll, prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    prev_top_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_top_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if ((mpColl_RightWall_inline2(coll, prev_top_x, prev_top_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    prev_bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    prev_bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if super::mplib::mpCheckRightWall(prev_bottom_x, prev_bottom_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    prev_top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    prev_top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if ((super::mplib::mpCheckRightWall(prev_top_x, prev_top_y, left_x, left_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_RightWall_inline(line_id);
        hit_wall = 1;
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 64;
    }
    return hit_wall;
}

/// `mpColl_8004BDD4_LeftWall`
pub unsafe fn mpColl_8004BDD4_LeftWall(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004BDD4_LeftWall");
    let mut line_id: i32 = 0;
    let mut right_x: f32 = 0.0;
    let mut right_y: f32 = 0.0;
    let mut prev_right_x: f32 = 0.0;
    let mut prev_right_y: f32 = 0.0;
    let mut prev_bottom_x: f32 = 0.0;
    let mut prev_bottom_y: f32 = 0.0;
    let mut prev_top_x: f32 = 0.0;
    let mut prev_top_y: f32 = 0.0;
    let mut bottom_x: f32 = 0.0;
    let mut bottom_y: f32 = 0.0;
    let mut top_x: f32 = 0.0;
    let mut top_y: f32 = 0.0;
    let mut hit_wall: i32 = 0;
    let mut line_id1: i32 = 0;
    let mut line_id2: i32 = 0;
    hit_wall = 0;
    mpColl_804D648C = 0;
    if super::mplib::mpLib_80054ED8((*coll).ceiling.index) != 0 {
        let mut temp: i32 = 0;
        line_id1 = super::mplib::mpLib_80053950_Ceiling((*coll).ceiling.index);
        temp = super::mplib::mpLib_80053A04_Ceiling((*coll).ceiling.index);
        line_id = temp;
        line_id2 = if temp != (-1_i32) { super::mplib::mpLib_80053950_Ceiling(line_id) } else { (-1_i32) };
    } else {
        line_id2 = (-1_i32);
        line_id1 = (-1_i32);
    }
    right_x = (*coll).cur_pos.x + (*coll).ecb.right.x;
    right_y = (*coll).cur_pos.y + (*coll).ecb.right.y;
    prev_right_x = (*coll).prev_pos.x + (*coll).prev_ecb.right.x;
    prev_right_y = (*coll).prev_pos.y + (*coll).prev_ecb.right.y;
    if mpColl_LeftWall_inline2(coll, prev_right_x, prev_right_y, right_x, right_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline3(line_id, mpColl_80458810.left.as_mut_ptr());
        hit_wall = 1;
        (*coll).env_flags = (*coll).env_flags | 32;
    }
    prev_bottom_x = (*coll).prev_pos.x + (*coll).prev_ecb.bottom.x;
    prev_bottom_y = (*coll).prev_pos.y + (*coll).prev_ecb.bottom.y;
    bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if mpColl_LeftWall_inline2(coll, prev_bottom_x, prev_bottom_y, bottom_x, bottom_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    prev_top_x = (*coll).prev_pos.x + (*coll).prev_ecb.top.x;
    prev_top_y = (*coll).prev_pos.y + (*coll).prev_ecb.top.y;
    top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if ((mpColl_LeftWall_inline2(coll, prev_top_x, prev_top_y, top_x, top_y, core::ptr::addr_of_mut!(line_id) as *mut i32) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    prev_bottom_x = (*coll).cur_pos.x + (*coll).ecb.bottom.x;
    prev_bottom_y = (*coll).cur_pos.y + (*coll).ecb.bottom.y;
    if super::mplib::mpCheckLeftWall(prev_bottom_x, prev_bottom_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0 {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    prev_top_x = (*coll).cur_pos.x + (*coll).ecb.top.x;
    prev_top_y = (*coll).cur_pos.y + (*coll).ecb.top.y;
    if ((super::mplib::mpCheckLeftWall(prev_top_x, prev_top_y, right_x, right_y, null_mut(), core::ptr::addr_of_mut!(line_id) as *mut i32, null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) && (line_id != line_id1)) && (line_id != line_id2) {
        mpColl_LeftWall_inline(line_id);
        hit_wall = 1;
    }
    if hit_wall != 0 {
        (*coll).env_flags = (*coll).env_flags | 1;
    }
    return hit_wall;
}

/// `mpColl_8004C328_Ceiling`
pub unsafe fn mpColl_8004C328_Ceiling(mut coll: *mut CollData, mut line_id: i32) -> i32 {
    melee_core::ct!("mpColl_8004C328_Ceiling");
    let mut flags: u32 = 0;
    let mut y: f32 = 0.0;
    let mut normal: Vec3 = unsafe { core::mem::zeroed() };
    let mut edge: Vec3 = unsafe { core::mem::zeroed() };
    let mut ceiling_id: i32 = 0;
    let mut result: i32 = 0;
    let mut edge_x: f32 = 0.0;
    let mut edge_y: f32 = 0.0;
    let mut ecb_side_x: f32 = 0.0;
    let mut ecb_side_y: f32 = 0.0;
    result = 0;
    if (!(super::mplib::mpLib_80054ED8(line_id) != 0)) || (super::mplib::mpLineGetKind(line_id) != (CollLine_Ceiling as i32)) {
        return 0;
    }
    super::mplib::mpCeilingGetLeft(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
    if (*coll).cur_pos.x <= edge.x {
        ceiling_id = super::mplib::mpLib_8004E090_Ceiling(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
        edge_x = edge.x + 1.0f32;
        edge_y = edge.y - 1.0f32;
        ecb_side_x = (edge.x + (*coll).ecb.right.x) - (*coll).ecb.top.x;
        ecb_side_y = (edge.y + (*coll).ecb.right.y) - (*coll).ecb.top.y;
        if !(super::mplib::mpCheckLeftWall(edge_x, edge_y, ecb_side_x, ecb_side_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
            result = 1;
            (*coll).env_flags = (*coll).env_flags | 2097152;
        }
    } else {
        super::mplib::mpCeilingGetRight(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec);
        if (*coll).cur_pos.x >= edge.x {
            ceiling_id = super::mplib::mpLib_8004E090_Ceiling(line_id, core::ptr::addr_of_mut!(edge) as *mut Vec, core::ptr::addr_of_mut!(y) as *mut f32, core::ptr::addr_of_mut!(flags) as *mut u32, core::ptr::addr_of_mut!(normal) as *mut Vec);
            edge_x = edge.x - 1.0f32;
            edge_y = edge.y - 1.0f32;
            ecb_side_x = (edge.x + (*coll).ecb.left.x) - (*coll).ecb.top.x;
            ecb_side_y = (edge.y + (*coll).ecb.left.y) - (*coll).ecb.top.y;
            if !(super::mplib::mpCheckRightWall(edge_x, edge_y, ecb_side_x, ecb_side_y, null_mut(), null_mut(), null_mut(), null_mut(), (*coll).joint_id_skip, (*coll).joint_id_only) != 0) {
                result = 1;
                (*coll).env_flags = (*coll).env_flags | 1048576;
            }
        }
    }
    if result != 0 {
        (*coll).cur_pos = edge;
        (*coll).ceiling.index = ceiling_id;
        (*coll).ceiling.flags = flags;
        (*coll).ceiling.normal = normal;
        return 1;
    }
    return 0;
}

/// `mpColl_8004C534`
pub unsafe extern "C" fn mpColl_8004C534(mut coll: *mut CollData, mut flags: u32) -> i32 {
    melee_core::ct!("mpColl_8004C534");
    let mut hit_ceiling: i32 = 0;
    if mpColl_8004BDD4_LeftWall(coll) != 0 {
        let _ = mpColl_80049EAC_LeftWall(coll);
        (*coll).x34_flags.set_b5((1 as u8) as u8);
    }
    if mpColl_8004B894_RightWall(coll) != 0 {
        let _ = mpColl_800491C8_RightWall(coll);
        (*coll).x34_flags.set_b5((1 as u8) as u8);
    }
    if mpColl_8004B6D8(coll) != 0 {
        mpCollCeilingInline(coll);
        hit_ceiling = 1;
    } else if super::mplib::mpLib_80054ED8((*coll).ceiling.index) != 0 {
        let mut ceiling_id: i32 = (*coll).ceiling.index;
        if !((flags & (1 as u32)) != 0) {
            if (flags & (2 as u32)) != 0 {
                if mpColl_8004C328_Ceiling(coll, ceiling_id) != 0 {
                    hit_ceiling = 1;
                } else {
                    (*coll).x34_flags.set_b5((1 as u8) as u8);
                }
            } else {
                (*coll).x34_flags.set_b5((1 as u8) as u8);
            }
        }
    }
    if hit_ceiling != 0 {
        (*coll).env_flags = (*coll).env_flags | 8192;
    }
    return hit_ceiling;
}

/// `mpColl_8004C750`
pub unsafe fn mpColl_8004C750(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_8004C750");
    mpCollPrev(coll);
    mpColl_LoadECB_inline(coll, 5);
    return inline3(coll, 2);
}

/// `mpCollSqueezeHorizontal`
pub unsafe fn mpCollSqueezeHorizontal(mut coll: *mut CollData, mut airborne: i32, mut left: f32, mut right: f32) {
    melee_core::ct!("mpCollSqueezeHorizontal");
    let mut half_width: f32 = 0.5f32 * (((right - left) + (*coll).ecb.right.x) - (*coll).ecb.left.x);
    if !((*coll).x34_flags.b6() != 0) {
        (*coll).x64_ecb = (*coll).ecb;
    }
    (*coll).x34_flags.set_b6((1 as u8) as u8);
    (*coll).cur_pos.x = (right + (*coll).ecb.right.x) - half_width;
    (*coll).ecb.right.x = half_width;
    (*coll).ecb.left.x = -half_width;
    (*coll).desired_ecb.right.x = (*coll).ecb.right.x;
    (*coll).desired_ecb.left.x = (*coll).ecb.left.x;
    (*coll).x34_flags.set_b5((0 as u8) as u8);
}

/// `mpCollSqueezeVertical`
pub unsafe fn mpCollSqueezeVertical(mut coll: *mut CollData, mut airborne: i32, mut top: f32, mut bottom: f32) {
    melee_core::ct!("mpCollSqueezeVertical");
    let mut height: f32 = ((top - bottom) + (*coll).ecb.top.y) - (*coll).ecb.bottom.y;
    let mut mid_y: f32 = 0.0;
    if !((*coll).x34_flags.b6() != 0) {
        (*coll).x64_ecb = (*coll).ecb;
    }
    (*coll).x34_flags.set_b6((1 as u8) as u8);
    if height < 3.0f32 {
        let mut old_height: f32 = (*coll).ecb.top.y - (*coll).ecb.bottom.y;
        let mut new_height: f32 = ((*coll).ecb.top.y + top) - bottom;
        (*coll).ecb.top.y = if old_height < new_height { old_height } else { new_height };
        (*coll).ecb.bottom.y = 0.0f32;
        (*coll).cur_pos.y = bottom;
    } else if !(airborne != 0) {
        (*coll).cur_pos.y = bottom;
        (*coll).ecb.top.y = height + (*coll).ecb.bottom.y;
    } else {
        (*coll).cur_pos.y = 0.5f32 * (top + bottom);
        (*coll).ecb.top.y = 0.5f32 * (((*coll).ecb.top.y + (*coll).ecb.bottom.y) + height);
        (*coll).ecb.bottom.y = (*coll).ecb.top.y - height;
    }
    mid_y = 0.5f32 * ((*coll).ecb.top.y + (*coll).ecb.bottom.y);
    (*coll).ecb.right.y = mid_y;
    (*coll).ecb.left.y = mid_y;
    (*coll).desired_ecb.top.y = (*coll).ecb.top.y;
    (*coll).desired_ecb.bottom.y = (*coll).ecb.bottom.y;
    (*coll).desired_ecb.left.y = (*coll).ecb.left.y;
    (*coll).desired_ecb.right.y = (*coll).ecb.right.y;
    (*coll).x34_flags.set_b5((0 as u8) as u8);
}

/// `mpColl_8004CA6C`
pub unsafe fn mpColl_8004CA6C(mut coll: *mut CollData) -> f32 {
    melee_core::ct!("mpColl_8004CA6C");
    let mut result: f32 = 1.0f32;
    if (*coll).floor.index != (-1_i32) {
        result = super::mplib::mpLib_800569EC((*coll).floor.flags);
    }
    return result;
}

/// `mpCollGetSpeedCeiling`
pub unsafe fn mpCollGetSpeedCeiling(mut coll: *mut CollData, mut speed: *mut Vec) -> i32 {
    melee_core::ct!("mpCollGetSpeedCeiling");
    let mut index: i32 = (*coll).ceiling.index;
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    top.x = (*coll).ecb.top.x;
    top.y = (*coll).ecb.top.y;
    top.z = 0.0f32;
    return super::mplib::mpGetSpeed(index, core::ptr::addr_of_mut!(top) as *mut Vec, speed);
}

/// `mpCollGetSpeedLeftWall`
pub unsafe fn mpCollGetSpeedLeftWall(mut coll: *mut CollData, mut speed: *mut Vec) -> i32 {
    melee_core::ct!("mpCollGetSpeedLeftWall");
    let mut index: i32 = (*coll).left_facing_wall.index;
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    top.x = (*coll).ecb.top.x;
    top.y = (*coll).ecb.top.y;
    top.z = 0.0f32;
    return super::mplib::mpGetSpeed(index, core::ptr::addr_of_mut!(top) as *mut Vec, speed);
}

/// `mpCollGetSpeedRightWall`
pub unsafe fn mpCollGetSpeedRightWall(mut coll: *mut CollData, mut speed: *mut Vec) -> i32 {
    melee_core::ct!("mpCollGetSpeedRightWall");
    let mut index: i32 = (*coll).right_facing_wall.index;
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    top.x = (*coll).ecb.top.x;
    top.y = (*coll).ecb.top.y;
    top.z = 0.0f32;
    return super::mplib::mpGetSpeed(index, core::ptr::addr_of_mut!(top) as *mut Vec, speed);
}

/// `mpCollGetSpeedFloor`
pub unsafe fn mpCollGetSpeedFloor(mut coll: *mut CollData, mut speed: *mut Vec) -> i32 {
    melee_core::ct!("mpCollGetSpeedFloor");
    let mut index: i32 = (*coll).floor.index;
    let mut top: Vec3 = unsafe { core::mem::zeroed() };
    top.x = (*coll).ecb.top.x;
    top.y = (*coll).ecb.top.y;
    top.z = 0.0f32;
    return super::mplib::mpGetSpeed(index, core::ptr::addr_of_mut!(top) as *mut Vec, speed);
}

/// `mpColl_IsOnPlatform`
pub unsafe fn mpColl_IsOnPlatform(mut coll: *mut CollData) -> i32 {
    melee_core::ct!("mpColl_IsOnPlatform");
    return (super::mplib::mpLineGetFlags((*coll).floor.index) & ((1 << 8) as u32)) as i32;
}

/// `mpUpdateFloorSkip`
pub unsafe fn mpUpdateFloorSkip(mut coll: *mut CollData) {
    melee_core::ct!("mpUpdateFloorSkip");
    (*coll).floor_skip = (*coll).floor.index;
}

/// `mpClearFloorSkip`
pub unsafe fn mpClearFloorSkip(mut coll: *mut CollData) {
    melee_core::ct!("mpClearFloorSkip");
    (*coll).floor_skip = (-1_i32);
}

/// `mpCopyCollData`
pub unsafe fn mpCopyCollData(mut src: *mut CollData, mut dst: *mut CollData, mut arg2: i32) {
    melee_core::ct!("mpCopyCollData");
    if arg2 == 1 {
        (*dst).x64_ecb = (*src).x64_ecb;
        (*dst).desired_ecb = (*src).desired_ecb;
        (*dst).ecb = (*src).ecb;
        (*dst).prev_ecb = (*src).prev_ecb;
        (*dst).xE4_ecb = (*src).xE4_ecb;
    }
    (*dst).cur_pos = (*src).cur_pos;
    (*dst).prev_pos = (*src).prev_pos;
    (*dst).last_pos = (*src).last_pos;
    (*dst).x34_flags.set_b0(((*src).x34_flags.b0()) as u8);
    (*dst).x34_flags.set_b1234(((*src).x34_flags.b1234()) as u8);
    (*dst).x34_flags.set_b5(((*src).x34_flags.b5()) as u8);
    (*dst).x34_flags.set_b6(((*src).x34_flags.b6()) as u8);
    (*dst).facing_dir = (*src).facing_dir;
    (*dst).x38 = (*src).x38;
    (*dst).floor_skip = (*src).floor_skip;
    (*dst).ledge_id_left = (*src).ledge_id_left;
    (*dst).ledge_id_right = (*src).ledge_id_right;
    (*dst).joint_id_skip = (*src).joint_id_skip;
    (*dst).lstick_x = (*src).lstick_x;
    (*dst).x64_ecb = (*src).x64_ecb;
    (*dst).desired_ecb = (*src).desired_ecb;
    (*dst).ecb = (*src).ecb;
    (*dst).prev_ecb = (*src).prev_ecb;
    (*dst).xE4_ecb = (*src).xE4_ecb;
    (*dst).x130_flags = (*src).x130_flags;
    (*dst).env_flags = (*src).env_flags;
    (*dst).prev_env_flags = (*src).prev_env_flags;
    (*dst).x13C = (*src).x13C;
    (*dst).contact = (*src).contact;
    (*dst).floor.index = (*src).floor.index;
    (*dst).floor.flags = (*src).floor.flags;
    (*dst).floor.normal = (*src).floor.normal;
    (*dst).left_facing_wall.index = (*src).left_facing_wall.index;
    (*dst).left_facing_wall.flags = (*src).left_facing_wall.flags;
    (*dst).left_facing_wall.normal = (*src).left_facing_wall.normal;
    (*dst).right_facing_wall.index = (*src).right_facing_wall.index;
    (*dst).right_facing_wall.flags = (*src).right_facing_wall.flags;
    (*dst).right_facing_wall.normal = (*src).right_facing_wall.normal;
    (*dst).ceiling.index = (*src).ceiling.index;
    (*dst).ceiling.flags = (*src).ceiling.flags;
    (*dst).ceiling.normal = (*src).ceiling.normal;
}

/// `mpColl_8004D024`
pub unsafe fn mpColl_8004D024(mut arg0: *mut Vec) -> i32 {
    melee_core::ct!("mpColl_8004D024");
    let mut spC: CollData = unsafe { core::mem::zeroed() };
    mpColl_80041EE4(core::ptr::addr_of_mut!(spC) as *mut CollData);
    spC.x34_flags.set_b1234((0 as u8) as u8);
    mpColl_SetECBSource_Fixed(core::ptr::addr_of_mut!(spC) as *mut CollData, null_mut(), 1e+01f32, 1e+01f32, 1e+01f32, 1e+01f32);
    spC.last_pos.x = (*arg0).x;
    spC.last_pos.y = (-3.0f32) + (*arg0).y;
    spC.last_pos.z = (*arg0).z;
    spC.cur_pos = *arg0;
    spC.x130_flags = spC.x130_flags | ((CollData_X130_Clear as i32) as u32);
    mpCollPrev(core::ptr::addr_of_mut!(spC) as *mut CollData);
    mpColl_LoadECB(core::ptr::addr_of_mut!(spC) as *mut CollData);
    let _ = inline0(core::ptr::addr_of_mut!(spC) as *mut CollData, 0, 1);
    if spC.x34_flags.b6() != 0 {
        return 1;
    }
    return 0;
}

/// `mpColl_stage_same_joint`
unsafe fn mpColl_stage_same_joint(mut a: i32, mut b: i32) -> i32 {
    let mut same: i32 = ext::ScriptGame_StageSlotSameJoint(a, b);
    if same >= 0 {
        return (same != 0) as i32;
    }
    return (super::mplib::mpJointFromLine(a) == super::mplib::mpJointFromLine(b)) as i32;
}
