//! src/melee/ft/kinds/ftYoshi/ftyoshi.c: Yoshi's init callbacks (the matanim rate setup of the
//! egg shield, on-load and on-death, the item and knockback hooks) and the file-name strings.
//! No fused sites (tools/rust-port/fma_sites.py: the only float products are `xC * (1 - a / b)`
//! and `(1 - ...)`, which contract to nothing). `HSD_MObjReqAnim`, `HSD_MObjAnim`,
//! `HSD_AObjSetRate`, `HSD_AObjGetEndFrame` and the item / parts helpers are `ext` hooks named as
//! the C; unset, they panic. `HSD_ASSERTREPORT` reports nothing here (its message goes to the
//! debug console on the GameCube, which the port does not model).
//!
//! Data: the file-name strings are below. `ftYs_Init_MotionStateTable` (the 29-row motion-state
//! table, pointers into the other ftYoshi TUs) belongs to the motion-state dispatcher and is not
//! here; `ftYs_CostumeList` is an uninitialised BSS array.
//! Smoke-tested by tests/tu_ftyoshi.rs (ftYs_Init_8012BAC0, ftYs_Init_8012B918); the rest is unverified.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use crate::stubs as st;
use melee_types::*;

/// Calls into TUs that are not ported yet (or that have no native port reachable from here).
/// Each is a settable hook; an unset hook panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut HSD_AOBJSETRATE: Option<fn(*mut HSD_AObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32)  {
                match unsafe { HSD_AOBJSETRATE } {
                    Some(f) => f(aobj, rate),
                    None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetRate(aobj as _, rate as _); } return; },
                }
            }
    crate::port_hook!(HSD_AOBJGETENDFRAME, HSD_AObjGetEndFrame, crate::adapt::HSD_AObjGetEndFrame, (aobj: *mut HSD_AObj) -> f32);
    // rp3 HSD_MObjReqAnim
    pub static mut HSD_MOBJREQANIM: Option<fn(*mut HSD_MObj, f32)> = None;
    #[allow(non_snake_case, unused_unsafe)]
    pub fn HSD_MObjReqAnim(mobj: *mut HSD_MObj, frame: f32) {
        match unsafe { HSD_MOBJREQANIM } {
            Some(f) => f(mobj, frame),
            None => unsafe { melee_runtime::tu::mobj::HSD_MObjReqAnim(mobj, frame); },
        }
    }
    // rp3-end

    // rp3 HSD_MObjAnim
    pub static mut HSD_MOBJANIM: Option<fn(*mut HSD_MObj)> = None;
    #[allow(non_snake_case, unused_unsafe)]
    pub fn HSD_MObjAnim(mobj: *mut HSD_MObj) {
        match unsafe { HSD_MOBJANIM } {
            Some(f) => f(mobj),
            None => unsafe { melee_runtime::tu::mobj::HSD_MObjAnim(mobj); },
        }
    }
    // rp3-end

    crate::port_hook!(HSD_JOBJSETTRANSLATE, HSD_JObjSetTranslate, crate::adapt::HSD_JObjSetTranslate, (jobj: *mut HSD_JObj, v: *const Vec3));

    pub static mut IT_8026B3F8: Option<fn(*mut core::ffi::c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut core::ffi::c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    pub static mut FTANIM_8006FA58: Option<fn(*mut Fighter, i32, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FA58(fp: *mut Fighter, part: i32, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_8006FA58 } {
            Some(f) => f(fp, part, joint),
            None => { unsafe { let _ = melee_fighter::tu::ftanim::ftAnim_8006FA58(&mut *(fp as *mut _), part as _, joint as _); } },
        }
    }
    pub static mut FTYS_SPECIALS_8012E270: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftYs_SpecialS_8012E270(gobj: *mut HSD_GObj)  {
                match unsafe { FTYS_SPECIALS_8012E270 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftyoshispecialhi::ftYs_SpecialS_8012E270(gobj as _); } return; },
                }
            }
    pub static mut FTYS_SPECIALS_8012DF18: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftYs_SpecialS_8012DF18(gobj: *mut HSD_GObj)  {
                match unsafe { FTYS_SPECIALS_8012DF18 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftyoshispecialhi::ftYs_SpecialS_8012DF18(gobj as _); } return; },
                }
            }
}

/// `HSD_ASSERTREPORT(line, cond, msg)`: the GameCube's assert report; the port does not model
/// the debug console, so nothing is reported.
#[inline(always)]
fn hsd_assert_report(_line: i32, _cond: i32, _msg: &str) {}

/// `ftYs_Unk1_803B75C0`: `Vec3 const = { 0 }`.
pub const ftYs_Unk1_803B75C0: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

pub const ftYs_Init_DatFilename: &str = "PlYs.dat";
pub const ftYs_Init_DataName: &str = "ftDataYoshi";
pub const ftYs_Init_AnimDatFilename: &str = "PlYsAJ.dat";

/// `ftYs_Init_DemoMotionFilenames`.
pub const ftYs_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileYoshi",
    "ftDemoIntroMotionFileYoshi",
    "ftDemoEndingMotionFileYoshi",
    "ftDemoViWaitMotionFileYoshi",
];

/// `ftYs_Init_CostumeStrings`: (dat, joint, matanim joint) for the six costumes.
pub const ftYs_Init_CostumeStrings: [[&str; 3]; 6] = [
    ["PlYsNr.dat", "PlyYoshi5K_Share_joint", "PlyYoshi5K_Share_matanim_joint"],
    ["PlYsRe.dat", "PlyYoshi5KRe_Share_joint", "PlyYoshi5KRe_Share_matanim_joint"],
    ["PlYsBu.dat", "PlyYoshi5KBu_Share_joint", "PlyYoshi5KBu_Share_matanim_joint"],
    ["PlYsYe.dat", "PlyYoshi5KYe_Share_joint", "PlyYoshi5KYe_Share_matanim_joint"],
    ["PlYsPi.dat", "PlyYoshi5KPi_Share_joint", "PlyYoshi5KPi_Share_matanim_joint"],
    ["PlYsAq.dat", "PlyYoshi5KAq_Share_joint", "PlyYoshi5KAq_Share_matanim_joint"],
];

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn attrs(fp: *mut Fighter) -> *mut ftYoshiAttributes {
    (*fp).dat_attrs as *mut ftYoshiAttributes
}

/// `ftYs_Init_8012B6E8` (static): stop the matanim of one shield set and latch its end frame.
pub fn ftYs_Init_8012B6E8(fp: *mut Fighter, unk_struct_arg: *mut S_UNK_YOSHI1) {
    melee_core::ct!("ftYs_Init_8012B6E8");
    unsafe {
        let attr_r26 = (*(*fp).ft_data).ext_attr as *mut ftYoshiAttributes;
        let unk_struct = (*unk_struct_arg).unk_struct;
        let index = (*unk_struct).xC_start_index;
        let end_index = (*unk_struct).x8_end_index;
        let zero_float = 0.0_f32;

        let mut i = 0;
        while i < end_index {
            let dobj = *(*fp).dobj_list.data.add(*index.add(i as usize) as usize);
            let mobj = if !dobj.is_null() { (*dobj).mobj } else { core::ptr::null_mut() };
            let aobj = (*mobj).aobj;
            ext::HSD_AObjSetRate(aobj, 0.0);

            if zero_float == (*attr_r26).xC {
                (*attr_r26).xC = ext::HSD_AObjGetEndFrame(aobj);
            } else if (*attr_r26).xC != ext::HSD_AObjGetEndFrame(aobj) {
                hsd_assert_report(97, 0, "yoshi matanim frame not same\n");
            }
            i += 1;
        }
    }
}

/// `ftYs_Init_8012B804` (static): request the shield matanim at `start_frame` and play it.
pub fn ftYs_Init_8012B804(fp: *mut Fighter, unk_struct_arg: *mut S_UNK_YOSHI1, start_frame: f32) {
    melee_core::ct!("ftYs_Init_8012B804");
    unsafe {
        if !unk_struct_arg.is_null() {
            let unk_struct = (*unk_struct_arg).unk_struct;
            let index = (*unk_struct).xC_start_index;
            let end_index = (*unk_struct).x8_end_index;
            let mut i = 0;
            while i < end_index {
                let dobj = *(*fp).dobj_list.data.add(*index.add(i as usize) as usize);
                let mobj = if !dobj.is_null() { (*dobj).mobj } else { core::ptr::null_mut() };
                ext::HSD_MObjReqAnim(mobj, start_frame);
                ext::HSD_MObjAnim(mobj);
                i += 1;
            }
        }
    }
}

/// `ftYs_Init_8012B8A4`: the shield matanim at the damaged fraction of the shield.
pub fn ftYs_Init_8012B8A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_8012B8A4");
    unsafe {
        let fp = fpp(gobj);
        let da = attrs(fp);
        let shield_base = melee_fighter::hooks::script_value(
            &*fp,
            4,
            melee_fighter::hooks::common_data().x260_startShieldHealth,
        );
        let tempf = (*da).xC * (1.0_f32 - ((*fp).shield_health / shield_base));
        ftYs_Init_8012B804(fp, (*fp).x5AC.xC[0] as *mut S_UNK_YOSHI1, tempf);
        ftYs_Init_8012B804(fp, (*fp).x5AC.xC[1] as *mut S_UNK_YOSHI1, tempf);
    }
}

/// `ftYs_Init_8012B918`: the shield matanim at frame 0.
pub fn ftYs_Init_8012B918(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_8012B918");
    unsafe {
        let fp = fpp(gobj);
        ftYs_Init_8012B804(fp, (*fp).x5AC.xC[0] as *mut S_UNK_YOSHI1, 0.0);
        ftYs_Init_8012B804(fp, (*fp).x5AC.xC[1] as *mut S_UNK_YOSHI1, 0.0);
    }
}

pub fn ftYs_Init_OnDeath(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnDeath");
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074A4C(gobj, 0, 0);
        (*fp).u.ys.x2238 = core::ptr::null_mut();
    }
}

/// `ftYs_Init_OnLoad`: the shield latch, the attribute push, and the three item kinds.
pub fn ftYs_Init_OnLoad(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnLoad");
    unsafe {
        let fp = fpp(gobj);
        let temp_r27 = (*fp).x5AC.xC[0] as *mut S_UNK_YOSHI1;
        let temp_r28 = (*fp).x5AC.xC[1] as *mut S_UNK_YOSHI1;
        let ft = (*fp).ft_data;
        let item_list = (*ft).x48_items;
        let other_attr = (*ft).ext_attr as *mut ftYoshiAttributes;

        if temp_r27.is_null() {
            hsd_assert_report(113, 0, "yoshi parts_model NULL!!\n");
        }

        (*other_attr).xC = 0.0;
        ftYs_Init_8012B6E8(fp, temp_r27);
        ftYs_Init_8012B6E8(fp, temp_r28);
        crate::tu::ftfox::push_attrs::<ftYoshiAttributes>(fp);
        ext::it_8026B3F8(*item_list.add(0), It_Kind_Yoshi_EggThrow as i32);
        ext::it_8026B3F8(*item_list.add(1), It_Kind_Yoshi_Star as i32);
        ext::it_8026B3F8(*item_list.add(2), It_Kind_Yoshi_EggLay as i32);
        (*fp).set_x2226_b1(1);
    }
}

/// `ftYs_Init_8012BA8C`: the two Special S setups.
pub fn ftYs_Init_8012BA8C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_8012BA8C");
    ext::ftYs_SpecialS_8012E270(gobj);
    ext::ftYs_SpecialS_8012DF18(gobj);
}

/// `ftYs_Init_8012BAC0`: the attribute at `+0x120`.
pub fn ftYs_Init_8012BAC0(fp: &Fighter) -> f32 {
    melee_core::ct!("ftYs_Init_8012BAC0");
    unsafe {
        let da = fp.dat_attrs as *const ftYoshiAttributes;
        (*da).x120
    }
}

pub fn ftYs_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: bool) {
    melee_core::ct!("ftYs_Init_OnItemPickup");
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag as i32, 1, 1);
}

pub fn ftYs_Init_OnItemInvisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnItemInvisible");
    st::Fighter_OnItemInvisible(gobj, 1);
}

pub fn ftYs_Init_OnItemVisible(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnItemVisible");
    st::Fighter_OnItemVisible(gobj, 1);
}

pub fn ftYs_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: bool) {
    melee_core::ct!("ftYs_Init_OnItemDrop");
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, bool1 as i32, 1, 1);
}

pub fn ftYs_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_LoadSpecialAttrs");
    unsafe { crate::tu::ftfox::copy_attrs::<ftYoshiAttributes>(gobj) }
}

pub fn ftYs_Init_OnKnockbackEnter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnKnockbackEnter");
    st::Fighter_OnKnockbackEnter(gobj, 1);
    st::ftAnim_ApplyPartAnim(gobj, 3, 3, 0.0);
    st::ftAnim_ApplyPartAnim(gobj, 4, 3, 0.0);
}

pub fn ftYs_Init_OnKnockbackExit(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_Init_OnKnockbackExit");
    st::Fighter_OnKnockbackExit(gobj, 1);
    st::ftAnim_ApplyPartAnim(gobj, 3, 2, 0.0);
    st::ftAnim_ApplyPartAnim(gobj, 4, 2, 0.0);
}
