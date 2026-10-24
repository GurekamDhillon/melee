//! src/melee/ft/kinds/ftCLink/ftclink.c: Young Link's init callbacks and Hylian-shield helpers.
//! No fused sites (tools/rust-port/fma_sites.py). Checked by tests/tu_ftclink.rs (the
//! 80149114 slide speed against the C cut verbatim; the rest smoke-tested through hooks).
//!
//! Data: the file-name strings are below; `ftCl_Init_MotionStateTable` (22 rows of
//! Anim/IASA/Phys/Coll pointers into ftLink/ftCommon TUs) belongs to the motion-state
//! dispatcher and is TODO until those callbacks are all native.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftCLink::ftclink::{ftCl_Init_OnDeath, ftCl_Init_OnKnockbackEnter, ftCl_Init_OnKnockbackExit};
pub use crate::chars::r6::ftCLink::ftclink::{ftCl_Init_OnItemInvisible, ftCl_Init_OnItemVisible};

pub const ftCl_Init_DatFilename: &str = "PlCl.dat";
pub const ftCl_Init_DataName: &str = "ftDataClink";
pub const ftCl_Init_AnimDatFilename: &str = "PlClAJ.dat";
/// `ftCl_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftCl_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileClink",
    "ftDemoIntroMotionFileClink",
    "ftDemoEndingMotionFileClink",
    "ftDemoViWaitMotionFileClink",
];
/// `ftCl_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftCl_Init_CostumeStrings: [[&str; 3]; 5] = [
    ["PlClNr.dat", "PlyClink5K_Share_joint", "PlyClink5K_Share_matanim_joint"],
    ["PlClRe.dat", "PlyClink5KRe_Share_joint", "PlyClink5KRe_Share_matanim_joint"],
    ["PlClBu.dat", "PlyClink5KBu_Share_joint", "PlyClink5KBu_Share_matanim_joint"],
    ["PlClWh.dat", "PlyClink5KWh_Share_joint", "PlyClink5KWh_Share_matanim_joint"],
    ["PlClBk.dat", "PlyClink5KBk_Share_joint", "PlyClink5KBk_Share_matanim_joint"],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    crate::port_hook!(LBANIM_8001E8F8, lbAnim_8001E8F8, crate::adapt::lbAnim_8001E8F8, (tree: *mut FigaTree) -> f32);
    pub static mut FTDATA_80085E50: Option<fn(*mut Fighter, i32) -> *mut FigaTree> = None;
            #[allow(non_snake_case)]
            pub fn ftData_80085E50(fp: *mut Fighter, msid: i32) -> *mut FigaTree {
                match unsafe { FTDATA_80085E50 } {
                    Some(f) => f(fp, msid),
                    None => { return unsafe { melee_fighter::tu::ftdata::ftData_80085E50(fp as _, msid as _) as _ }; },
                }
            }
    pub static mut FTLK_INIT_ONLOADFORCLINK: Option<fn(*mut Fighter) > = None;
            #[allow(non_snake_case)]
            pub fn ftLk_Init_OnLoadForCLink(fp: *mut Fighter)  {
                match unsafe { FTLK_INIT_ONLOADFORCLINK } {
                    Some(f) => f(fp),
                    None => { unsafe { let _ = crate::tu::ftlink::ftLk_Init_OnLoadForCLink(fp as _); } return; },
                }
            }
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    hook!(FTPARTS_800753D4, ftParts_800753D4, (fp: *mut Fighter, x0: *mut Fighter_804D6540_x0_t, item: *mut c_void));
    pub static mut FTLK_INIT_LOADSPECIALATTRS: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftLk_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj)  {
                match unsafe { FTLK_INIT_LOADSPECIALATTRS } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = crate::tu::ftlink::ftLk_Init_LoadSpecialAttrs(gobj as _); } return; },
                }
            }
    pub static mut FTCO_80092ED8: Option<fn(i32, f32) -> f32> = None;
            #[allow(non_snake_case)]
            pub fn ftCo_80092ED8(a: i32, b: f32) -> f32 {
                match unsafe { FTCO_80092ED8 } {
                    Some(f) => f(a, b),
                    None => { return unsafe { melee_fighter::tu::ftCo_Guard::ftCo_80092ED8(a as _, b as _) as _ }; },
                }
            }
    crate::port_hook!(FTCOLL_8007B1B8, ftColl_8007B1B8, crate::adapt::ftColl_8007B1B8, (gobj: *mut HSD_GObj, shield: *mut c_void, cb: HSD_GObjEvent));
    pub static mut FTCO_800DEAE8: Option<fn(*mut HSD_GObj, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800DEAE8(gobj: *mut HSD_GObj, msid0: i32, msid1: i32)  {
                match unsafe { FTCO_800DEAE8 } {
                    Some(f) => f(gobj, msid0, msid1),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_AppealS::ftCo_800DEAE8(gobj as _, msid0 as _, msid1 as _); } return; },
                }
            }
    crate::port_hook!(IT_802C8C34, it_802C8C34, crate::adapt::it_802C8C34, (item: *mut Item_GObj));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftCl_Init_80149114` as a Fighter callback entry (the shield-hit callback).
pub unsafe extern "C" fn cb_ftCl_Init_80149114(g: *mut HSD_GObj) {
    ftCl_Init_80149114(g);
}

/// `Fighter_OnItemPickup(gobj, catch, bool2, bool3)` (ft/inlines.h).
pub fn Fighter_OnItemPickup(gobj: *mut HSD_GObj, catch_item: i32, bool2: i32, bool3: i32) {
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 0 {
            match melee_fighter::tu::ftwaitanim::ext::itGetHoldKind((*fp).item_gobj) {
                1 => melee_fighter::ftanim::ftanim_80070fb4(gobj, bool2, 1),
                2 => melee_fighter::ftanim::ftanim_80070fb4(gobj, bool2, 0),
                3 => melee_fighter::ftanim::ftanim_80070fb4(gobj, bool2, 2),
                4 => melee_fighter::ftanim::ftanim_80070fb4(gobj, bool2, 3),
                _ => {}
            }
            if catch_item != 0 {
                st::ftAnim_80070C48(gobj, bool3);
            }
        }
    }
}

/// `Fighter_OnItemDrop(gobj, drop, bool2, bool3)` (ft/inlines.h).
pub fn Fighter_OnItemDrop(gobj: *mut HSD_GObj, drop_item: i32, bool2: i32, bool3: i32) {
    melee_fighter::ftanim::ftanim_80070fb4(gobj, bool2, -1);
    if drop_item != 0 {
        st::ftAnim_80070CC4(gobj, bool3);
    }
}

/// `checkFighter2244` (ftCLink/inlines.h): drop the held milk/item of the taunt.
pub fn checkFighter2244(gobj: *mut HSD_GObj) {
    if gobj.is_null() {
        return;
    }
    unsafe {
        let fp = fpp(gobj);
        if !fp.is_null() && !(*fp).u.lk.x18.is_null() {
            ext::it_802C8C34((*fp).u.lk.x18);
            (*fp).u.lk.x18 = core::ptr::null_mut();
        }
    }
}

pub fn ftCl_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let ftdata = (*fp).ft_data;
        let mut ea = (*ftdata).ext_attr as *mut ftLk_DatAttrs;
        let items = (*ftdata).x48_items;
        (*fp).set_can_walljump(1);
        (*ea).attackairlw_hit_anim_frame_end = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, 72));
        ext::ftLk_Init_OnLoadForCLink(fp);
        ea = (*fp).dat_attrs as *mut ftLk_DatAttrs;
        ext::it_8026B3F8(*items.add(0), (*ea).x48);
        ext::it_8026B3F8(*items.add(1), (*ea).x2C);
        ext::it_8026B3F8(*items.add(2), (*ea).xBC);
        ext::it_8026B3F8(*items.add(3), (*ea).xC);
        ext::it_8026B3F8(*items.add(4), (*ea).x10);
        ext::it_8026B3F8(*items.add(5), It_Kind_CLink_Milk as i32);
        let t = *fh::fighter_804d6540().add((*fp).kind as usize);
        ext::ftParts_800753D4(fp, (*t).x0, *items.add(6));
    }
}

pub fn ftCl_Init_OnItemPickupExt(gobj: *mut HSD_GObj, flag: i32) {
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 1 {
            st::ftParts_80074A4C(gobj, 1, 1);
        }
    }
    st::ftParts_80074A4C(gobj, 2, 1);
    ftCl_Init_OnItemPickup(gobj, flag);
}

pub fn ftCl_Init_OnItemDropExt(gobj: *mut HSD_GObj, flag: i32) {
    unsafe {
        let fp = fpp(gobj);
        if st::itIsHeavy((*fp).item_gobj) == 1 {
            st::ftParts_80074A4C(gobj, 1, 0);
        }
    }
    st::ftParts_80074A4C(gobj, 2, 0);
    ftCl_Init_OnItemDrop(gobj, flag);
}

pub fn ftCl_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftCl_Init_OnItemDrop(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemDrop(gobj, flag, 1, 1);
}

pub fn ftCl_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    ext::ftLk_Init_LoadSpecialAttrs(gobj);
}

/// `ftCl_Init_80149114`: the shield-hit push-back (fighter-only body: [`shield_push`]).
pub fn ftCl_Init_80149114(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = &mut *fpp(gobj);
        shield_push(fp);
        fh::play_sfx(fp, 70106, 127, 64);
    }
}

/// Body of `ftCl_Init_80149114` up to its sound call.
pub fn shield_push(fp: &mut Fighter) {
    let ea = unsafe { &*((*fp.ft_data).ext_attr as *const ftLk_DatAttrs) };
    let mut ftmp = ext::ftCo_80092ED8(fp.x19A4, ea.xD8);
    fp.gr_vel = ftmp * fh::common_data().x294;
    if fp.specialn_facing_dir < 0.0 {
        ftmp = fp.gr_vel;
    } else {
        ftmp = -fp.gr_vel;
    }
    fp.gr_vel = ftmp;
}

pub fn ftCl_Init_8014919C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).x5F4_arr[2].prev == 0 {
            let da = (*fp).dat_attrs as *mut ftLk_DatAttrs;
            ext::ftColl_8007B1B8(gobj, &mut (*da).xC4 as *mut AbsorbDesc as *mut _, Some(cb_ftCl_Init_80149114));
            (*fp).anon_221b.set_x221B_b3(1);
            (*fp).anon_221b.set_x221B_b4(1);
            (*fp).anon_221b.set_x221B_b2(1);
        }
    }
}

pub fn ftCl_Init_8014920C(gobj: *mut HSD_GObj) -> bool {
    if gobj.is_null() {
        return true;
    }
    unsafe {
        let fp = fpp(gobj);
        if fp.is_null() {
            return true;
        }
        let m = (*fp).motion_id as i32;
        if m != 342 && m != 343 {
            return true;
        }
        if (*fp).u.lk.x18.is_null() {
            return true;
        }
    }
    false
}

pub fn ftCl_Init_80149268(gobj: *mut HSD_GObj) {
    checkFighter2244(gobj);
}

pub fn ftCl_Init_801492C4(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() && !(*fp).u.lk.x18.is_null() {
                (*fp).u.lk.x18 = core::ptr::null_mut();
            }
        }
    }
}

/// C `bool` (int) return of the u32 `cmd_vars[1]`, unnormalised.
pub fn ftCl_Init_801492F4(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        unsafe {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return (*fp).cmd_vars[1] as i32;
            }
        }
    }
    0
}

pub fn ftCl_Init_80149318(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_800DEAE8(gobj, 342, 343);
        (*fp).cmd_vars[1] = 0;
    }
}
