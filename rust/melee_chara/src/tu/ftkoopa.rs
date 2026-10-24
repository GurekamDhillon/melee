//! src/melee/ft/kinds/ftKoopa/ftkoopa.c: Bowser's init callbacks (shared with Giga Bowser
//! through `ftKp_Init_OnLoadForGKoopa`) and the SpecialS attribute getters.
//! No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py). Smoke-tested by
//! tests/tu_ftkoopa.rs.
//!
//! Data: the file-name strings are below; `ftKp_Init_MotionStateTable` (the `ftKp_MS_SelfCount`
//! rows of Anim/IASA/Phys/Coll pointers into the ftKoopa TUs) belongs to the motion-state
//! dispatcher and is TODO until the SpecialS callbacks are all native. `ftKp_CostumeList` is
//! zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};
use melee_types::*;

/// Already ported (rounds 5/6 and hand-written): re-exported under this TU. The SpecialS getters
/// take the `Fighter` the C reads from the gobj (pure helpers).
pub use crate::chars::ftKoopa::ftkoopa::{
    ftKp_SpecialS_80132DC0, ftKp_SpecialS_80132DD0, ftKp_SpecialS_80132DE0, ftKp_SpecialS_80132DF0,
    ftKp_SpecialS_80132E00, ftKp_SpecialS_80132E10, ftKp_SpecialS_80132E20,
};
pub use crate::chars::r5::ftKoopa::ftkoopa::{ftKp_Init_OnKnockbackEnter, ftKp_Init_OnKnockbackExit};
pub use crate::chars::r6::ftKoopa::ftkoopa::{ftKp_Init_OnItemInvisible, ftKp_Init_OnItemVisible};

pub const ftKp_Init_DatFilename: &str = "PlKp.dat";
pub const ftKp_Init_DataName: &str = "ftDataKoopa";
pub const ftKp_Init_AnimDatFilename: &str = "PlKpAJ.dat";
/// `ftKp_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftKp_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileKoopa",
    "ftDemoIntroMotionFileKoopa",
    "ftDemoEndingMotionFileKoopa",
    "ftDemoViWaitMotionFileKoopa",
];
/// `ftKp_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftKp_Init_CostumeStrings: [[&str; 3]; 4] = [
    ["PlKpNr.dat", "PlyKoopa5K_Share_joint", "PlyKoopa5K_Share_matanim_joint"],
    ["PlKpRe.dat", "PlyKoopa5KRe_Share_joint", "PlyKoopa5KRe_Share_matanim_joint"],
    ["PlKpBu.dat", "PlyKoopa5KBu_Share_joint", "PlyKoopa5KBu_Share_matanim_joint"],
    ["PlKpBk.dat", "PlyKoopa5KBk_Share_joint", "PlyKoopa5KBk_Share_matanim_joint"],
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
    pub static mut IT_8026B3F8: Option<fn(*mut c_void, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B3F8(article: *mut c_void, kind: i32)  {
                match unsafe { IT_8026B3F8 } {
                    Some(f) => f(article, kind),
                    None => { unsafe { let _ = melee_items::tu::it_26B1::it_8026B3F8(article as _, kind as _); } return; },
                }
            }
    pub static mut FTCO_800DE2A8: Option<fn(*mut HSD_GObj, *mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_800DE2A8(gobj: *mut HSD_GObj, vic: *mut HSD_GObj)  {
                match unsafe { FTCO_800DE2A8 } {
                    Some(f) => f(gobj, vic),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8(gobj as _, vic as _); } return; },
                }
            }
    hook!(FTCO_800DE7C0, ftCo_800DE7C0, (gobj: *mut HSD_GObj, a: i32, b: i32));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftKp_Init_OnDeath(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let koopa_attr = (*fp).dat_attrs as *const ftKoopaAttributes;
        st::ftParts_80074A4C(gobj, 0, 0);
        (*fp).dmg.armor0 = (*koopa_attr).x0;
        (*fp).u.kp.x222C = (*koopa_attr).x10;
        (*fp).u.kp.x2230 = (*koopa_attr).x18;
    }
}

pub fn ftKp_Init_UnkMotionStates3(gobj: *mut HSD_GObj) {
    crate::tu::ftkoopaspecialn::ftKp_SpecialLw_80134D78(gobj);
}

/// `PUSH_ATTRS(fp, ftKoopaAttributes)`: copy ext_attr into the backup, point dat_attrs at it.
pub fn ftKp_Init_OnLoadForGKoopa(fp: *mut Fighter) {
    unsafe {
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftKoopaAttributes;
        *((*fp).dat_attrs_backup as *mut ftKoopaAttributes) = *src;
        (*fp).dat_attrs = backup;
    }
}

pub fn ftKp_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;
        ftKp_Init_OnLoadForGKoopa(fp);
        ext::it_8026B3F8(*items.add(0), It_Kind_Koopa_Flame as i32);
        (*fp).set_x2226_b1(1);
    }
}

/// `ftKp_Init_80132B38`: the empty damage callback.
pub fn ftKp_Init_80132B38(_gobj: *mut HSD_GObj) {}

pub fn ftKp_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftKp_Init_OnItemDrop(gobj: *mut HSD_GObj, bool1: i32) {
    Fighter_OnItemDrop(gobj, bool1, 1, 1);
}

/// `COPY_ATTRS(gobj, ftKoopaAttributes)`.
pub fn ftKp_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftKoopaAttributes) = *((*(*fp).ft_data).ext_attr as *const ftKoopaAttributes);
    }
}

/// `ftCheckThrowB4` (ft/inlines.h): test and clear the script's throw flag.
fn ftCheckThrowB4(fp: &mut Fighter) -> bool {
    unsafe {
        if fp.anon_2210.anon_0.throw_flags_b4() != 0 {
            fp.anon_2210.anon_0.set_throw_flags_b4(0);
            true
        } else {
            false
        }
    }
}

pub fn ftKp_SpecialS_80132E30(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if ftCheckThrowB4(&mut *fp) {
            (*fp).facing_dir = -(*fp).facing_dir;
            (*fp).mv.kp.unk1.xC = 1;
        }
        if (*fp).cmd_vars[0] == 0 {
            return;
        }
        if (*fp).victim_gobj.is_null() {
            return;
        }
        let victim_gobj = (*fp).victim_gobj;
        st::ftCommon_8007E2F4(&mut *fp, 0);
        ext::ftCo_800DE2A8(gobj, victim_gobj);
        ext::ftCo_800DE7C0(victim_gobj, 0, 0);
        (*fp).cmd_vars[0] = 0;
    }
}
