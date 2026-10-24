//! src/melee/ft/kinds/ftLink/ftlinkspecials.c: Link's Boomerang (side B).
//! Fused sites (tools/rust-port/fma_sites.py): none. `calcAnglePos` is checked bit-exact against
//! the C by tests/tu_ftlinkspecials.rs (shipping contraction; `cosf`/`sinf`/`atan2f` are group 1's
//! MSL ports, the `M_PI`/`M_TAU` arithmetic is done in double like the C); the rest is smoke-tested
//! through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftlink::{ftLk_Init_BoomerangExists, cb_ftLk_800EAF38, cb_ftLk_800EAF58};
use crate::tu::ftlinkspeciallw::ftCheckThrowB0;
use melee_fighter::hooks as fh;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

/// Already ported (rounds 2, 3, 4 and the hand-written `chars::ftLink`): re-exported under this TU.
pub use crate::chars::r2::ftLink::ftlinkspecials::{
    ftLk_SpecialAirS1Empty_Phys, ftLk_SpecialAirS1_Anim, ftLk_SpecialAirS1_Phys, ftLk_SpecialAirS2_Phys,
    ftLk_SpecialAirS1Empty_Coll, ftLk_SpecialS1Empty_Anim, ftLk_SpecialS1Empty_Coll, ftLk_SpecialS1Empty_Phys,
    ftLk_SpecialS1_Anim, ftLk_SpecialS1_Phys, ftLk_SpecialS2_Phys,
};
pub use crate::chars::r3::ftLink::ftlinkspecials::{ftLk_SpecialAirS1Empty_Anim, ftLk_SpecialAirS1_Coll, ftLk_SpecialS1_Coll};
pub use crate::chars::r4::ftLink::ftlinkspecials::ftLk_SpecialS2_Enter;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftlinkspeciallw::ext::{lb_8000B1CC, LB_8000B1CC};
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
    // it/kinds/itlinkboomerang.c
    pub static mut IT_802A07B4: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802A07B4(boomerang: *mut Item_GObj)  {
                match unsafe { IT_802A07B4 } {
                    Some(f) => f(boomerang),
                    None => { unsafe { let _ = melee_items::tu::itlinkboomerang::it_802A07B4(boomerang as _); } return; },
                }
            }
    hook!(IT_802A013C, it_802A013C, (facing_dir: f32, owner: *mut HSD_GObj, pos: &Vec3, part: i32, x2c: i32) -> *mut Item_GObj);
    pub static mut IT_8029FDBC: Option<fn(*mut Item_GObj) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn it_8029FDBC(boomerang: *mut Item_GObj) -> bool {
                match unsafe { IT_8029FDBC } {
                    Some(f) => f(boomerang),
                    None => { return unsafe { melee_items::tu::itlinkboomerang::it_8029FDBC(boomerang as _) != 0 }; },
                }
            }
    hook!(IT_802A0534, it_802A0534, (boomerang: *mut Item_GObj, pos: &Vec3, angle: f32));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
unsafe fn da<'a>(fp: *mut Fighter) -> &'a ftLk_DatAttrs {
    &*((*fp).dat_attrs as *const ftLk_DatAttrs)
}

/// `on21EC` (static): decide whether the throw was a smash (dash-smash window) input.
pub fn on21EC(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let cd = fh::common_data();
        if melee_fighter::ftcommon::abs_c((*fp).input.lstick[0].x) >= cd.dash_smash_stick_threshold
            && ((*fp).active_sticky.lstick.x as f32) < (cd.dash_smash_window as f32 + cd.x44)
        {
            (*fp).x2070.anon_0.set_count_thrown_items(1);
            (*fp).u.lk.x4 = 1;
        } else {
            (*fp).u.lk.x4 = 0;
        }
    }
}

/// `on21EC` as a Fighter callback entry.
pub unsafe extern "C" fn cb_on21EC(g: *mut HSD_GObj) {
    on21EC(g);
}

/// `ftLk_SepcialS_Get2219b5(gobj)`: the fighter-only body is `chars::ftLink`'s.
pub fn ftLk_SepcialS_Get2219b5(gobj: *mut HSD_GObj) -> bool {
    unsafe { crate::chars::ftLink::ftlinkspecials::ftLk_SepcialS_Get2219b5(&*fpp(gobj)) }
}

pub fn ftLk_SpecialS_Is2071b0_5to13(gobj: *mut HSD_GObj) -> bool {
    unsafe { matches!((*fpp(gobj)).x2070.anon_0.x2071_b0_3(), 5..=13) }
}

pub fn ftLk_SpecialS_Is2071b0_1to13(gobj: *mut HSD_GObj) -> bool {
    unsafe { matches!((*fpp(gobj)).x2070.anon_0.x2071_b0_3(), 1..=13) }
}

pub fn ftLk_SpecialS_RemoveBoomerang0(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.lk.used_boomerang = 0;
        (*fp).u.lk.boomerang_gobj = core::ptr::null_mut();
        ftLk_Init_BoomerangExists(gobj)
    }
}

pub fn ftLk_SpecialS_RemoveBoomerang1(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.lk.boomerang_gobj.is_null() {
            ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
            (*fp).u.lk.used_boomerang = 0;
            (*fp).u.lk.boomerang_gobj = core::ptr::null_mut();
            ftLk_Init_BoomerangExists(gobj);
        }
    }
}

/// `calcAnglePos` (static): the throw angle from the stick (clamped to +-x18*x1C) and the spawn
/// offset `dist` along it, mirrored by the facing. Returns the angle in [0, tau).
pub fn calcAnglePos(fp: &Fighter, a: &ftLk_DatAttrs, pos: &mut Vec3, dist: f32) -> f32 {
    let mut angle: f32 = 0.0;
    let lstick_y = fp.input.lstick[0].y;
    let da_x18_mul_x1c = a.x18 * a.x1C;
    if melee_fighter::ftcommon::abs_c(lstick_y) > a.x14 {
        let mut lstick_x = fp.input.lstick[0].x;
        if lstick_x < 0.0 {
            lstick_x = -lstick_x;
        }
        angle = melee_runtime::lbtrigf::atan2f(lstick_y, lstick_x);
        if angle > da_x18_mul_x1c {
            angle = da_x18_mul_x1c;
        } else {
            let neg = -da_x18_mul_x1c;
            if angle < neg {
                angle = neg;
            }
        }
    }
    pos.x = fp.facing_dir * (dist * cosf(angle));
    pos.y = fp.facing_dir * (dist * sinf(angle));
    pos.z = 0.0;
    if fp.facing_dir == -1.0 {
        // `-M_PI - angle` / `+M_PI - angle` are double expressions stored back into the float.
        if angle < 0.0 {
            angle = (-core::f64::consts::PI - angle as f64) as f32;
        } else {
            angle = (core::f64::consts::PI - angle as f64) as f32;
        }
    }
    if angle < 0.0 {
        // `M_TAU` is the double 6.283185307179586.
        angle = (angle as f64 + core::f64::consts::TAU) as f32;
    }
    angle
}

/// `onAccessory4` (static): spawn the boomerang on the script's throw pulse, then aim it.
pub fn onAccessory4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let a = da(fp);
        if ftCheckThrowB0(&mut *fp) {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let idx = fh::ftparts_get_bone_index(&*fp, FtPart_LThumbNb as i32);
            ext::lb_8000B1CC((*(*fp).parts.offset(idx as isize)).joint as *mut _, None, &mut pos);
            let boomerang_gobj = ext::it_802A013C((*fp).facing_dir, gobj, &pos, idx, a.x2C);
            (*fp).x1984_heldItemSpec = boomerang_gobj;
            (*fp).u.lk.boomerang_gobj = boomerang_gobj;
            if !boomerang_gobj.is_null() {
                (*fp).u.lk.used_boomerang = 1;
                (*fp).death3_cb = Some(cb_ftLk_800EAF38);
                (*fp).take_dmg_cb = Some(cb_ftLk_800EAF58);
            }
        }
        if (*fp).cmd_vars[0] != 0 && !(*fp).u.lk.boomerang_gobj.is_null() && ext::it_8029FDBC((*fp).u.lk.boomerang_gobj) {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let dist = if (*fp).u.lk.x4 != 0 { a.x20 } else { a.x24 };
            let angle = calcAnglePos(&*fp, a, &mut pos, dist);
            ext::it_802A0534((*fp).u.lk.boomerang_gobj, &pos, angle);
            (*fp).cmd_vars[0] = 0;
            (*fp).cmd_vars[1] = 0;
        }
    }
}

/// `onAccessory4` as a Fighter callback entry.
pub unsafe extern "C" fn cb_onAccessory4(g: *mut HSD_GObj) {
    onAccessory4(g);
}

pub fn ftLk_SpecialS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).x21EC = Some(cb_on21EC);
        if (*fp).u.lk.used_boomerang != 0 {
            st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialS1Empty as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialS1 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_onAccessory4);
    }
}

pub fn ftLk_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).x21EC = Some(cb_on21EC);
        if (*fp).u.lk.used_boomerang != 0 {
            st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirS1Empty as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        } else {
            st::Fighter_ChangeMotionState(gobj, ftLk_MS_SpecialAirS1 as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
        }
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_onAccessory4);
    }
}

/// `doS2Anim` (static): drop the boomerang on the script's cmd_vars[1] pulse, then `cb` at the end.
fn doS2Anim(gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj)) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[1] != 0 {
            if !(*fp).u.lk.boomerang_gobj.is_null() {
                ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
            }
            (*fp).cmd_vars[1] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftLk_SpecialS2_Anim(gobj: *mut HSD_GObj) {
    doS2Anim(gobj, st::ft_8008A2BC);
}

pub fn ftLk_SpecialAirS2_Anim(gobj: *mut HSD_GObj) {
    doS2Anim(gobj, st::ftCo_Fall_Enter);
}

/// `checkBoomerangSomething` (static inline): any grounded action input (short-circuits).
fn checkBoomerangSomething(gobj: *mut HSD_GObj) -> bool {
    use melee_fighter::tu as t;
    t::ftCo_SpecialS::ftCo_SpecialS_CheckInput(gobj)
        || t::ftCo_Attack100::ftCo_Attack100_CheckInput(gobj)
        || t::ftCo_Attack100::ftCo_800D6824(gobj)
        || t::ftCo_Attack100::ftCo_800D68C0(gobj)
        || t::ftCo_Guard::ftCo_80091A4C(gobj)
        || t::ftCo_Jump::ftCo_Jump_CheckInput(gobj)
        || t::ftCo_Dash::ftCo_Dash_CheckInput(gobj)
}

pub fn ftLk_SpecialS2_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if checkBoomerangSomething(gobj) && !(*fp).u.lk.boomerang_gobj.is_null() {
            ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
        }
    }
}

/// `checkAirBoomerangSomething` (static): any aerial action input (short-circuits).
fn checkAirBoomerangSomething(gobj: *mut HSD_GObj) -> bool {
    use melee_fighter::tu as t;
    t::ftCo_SpecialAir::ftCo_SpecialAir_CheckInput(gobj) || t::ftCo_JumpAerial::ftCo_800CB870(gobj)
}

pub fn ftLk_SpecialAirS2_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if checkAirBoomerangSomething(gobj) && !(*fp).u.lk.boomerang_gobj.is_null() {
            ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
        }
    }
}

pub fn ftLk_SpecialS2_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80082708(gobj) == 0 {
            if !(*fp).u.lk.boomerang_gobj.is_null() {
                ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
            }
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftLk_SpecialAirS2_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            if !(*fp).u.lk.boomerang_gobj.is_null() {
                ext::it_802A07B4((*fp).u.lk.boomerang_gobj);
            }
            melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter_Basic(gobj);
        }
    }
}
