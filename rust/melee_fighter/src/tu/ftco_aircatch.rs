//! src/melee/ft/kinds/ftCommon/ftCo_AirCatch.c: tether recovery (Z-air grab) for Link / Young
//! Link / Samus: ledge probe, entry check (incl. the TARGET_PC caps gate and m-ex onZair hook),
//! state enters, the grapple timeline in AirCatch_Anim, and Phys / Coll.
//!
//! Fused sites: AirCatch_Anim's wall-probe ends `8.0 * facing * scale.y + mtx[0][3]` (Link) and
//! `2.0 * facing * scale.y + mtx[0][3]` (Samus): one f64 fma each, then f32. No others
//! (`gravity * 0.2` is a lone f64 multiply). Callees outside the ported set are `ext` stubs with
//! override slots. Smoke-tested by tests/tu_ftco_aircatch.rs.
#![allow(non_snake_case)]

use super::ftco_0d8e::hsd_jobj_setup_matrix;
use super::ftco_0d95::ext as ss;
use crate::ftcommon as co;
use crate::ftcommon2 as co2;
use crate::hooks as h;
use crate::types::*;

pub type G = *mut Fighter_GObj;

macro_rules! stubs {
    ($($slot:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
        pub static mut $slot: Option<fn($($t),*) $(-> $r)?> = None;
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $slot } {
                Some(f) => f($($a),*),
                None => melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported")),
            }
        }
    )*};
}

pub mod ext {
    #![allow(non_snake_case)]
    use crate::types::*;
    type G = *mut HSD_GObj;
    stubs! {
        MPCOLL_80044164 mpColl_80044164(c: &mut CollData, id: &mut i32) -> bool;
        MPCOLL_800443C4 mpColl_800443C4(c: &mut CollData, id: &mut i32) -> bool;
        SCRIPTGAME_CAPS ScriptGame_CapsActionAllowed(fp: &Fighter, flag: i32) -> bool;
        MEX_HAS_ZAIR Mex_HasZairHook(kind: i32) -> bool;
        MEX_ON_ZAIR Mex_OnZairDispatch(kind: i32, g: G);
        CHANGE_MOTION Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G);
        FTCO_800968C8 ftCo_800968C8(g: G);
        IT_802A2BA4 it_802A2BA4(g: G, pos: &mut Vec3, facing: f32, x: i32) -> *mut Item_GObj;
        IT_802A2B10 it_802A2B10(item: *mut Item_GObj);
        IT_802A78B8 it_802A78B8(item: *mut Item_GObj, pos: &mut Vec3);
        IT_802A77DC it_802A77DC(item: *mut Item_GObj);
        FT_80081D0C ft_80081D0C(g: G) -> GroundOrAir;
        LANDING_ENTER ftCo_Landing_Enter(g: G, msid: i32, a: bool, flags: u32, f0: f32, f1: f32);
    }
    /// TODO(group5 itlinkhookshot.c) callbacks stored in the fighter's accessory / death slots.
    pub unsafe extern "C" fn it_802A7AF0(_g: *mut HSD_GObj) { { unsafe { melee_items::tu::itlinkhookshot::it_802A7AF0(_g as _); } return; } }
    pub unsafe extern "C" fn it_802A7AAC(_g: *mut HSD_GObj) { { unsafe { melee_items::tu::itlinkhookshot::it_802A7AAC(_g as _); } return; } }
    pub unsafe extern "C" fn it_802A7B34(_g: *mut HSD_GObj) { { unsafe { melee_items::tu::itlinkhookshot::it_802A7B34(_g as _); } return; } }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

fn is_link(f: &Fighter) -> bool {
    f.kind == Ft_Kind_Link || f.kind == Ft_Kind_CLink
}

/// `ftCo_800C3A14`: ledge probe on a copy of the collision data raised by 5.
pub fn ftco_800c3a14(g: G) -> bool {
    melee_core::ct!("ftco_800c3a14");
    let f = fp(g);
    let mut coll = f.coll_data;
    coll.ledge_snap_y = (coll.ledge_snap_y as f64 + 5.0) as f32;
    coll.ledge_snap_height = (coll.ledge_snap_height as f64 + 5.0) as f32;
    let hit = if f.facing_dir as f64 > 0.0 {
        if !ext::mpColl_80044164(&mut coll, &mut f.coll_data.ledge_id_left) {
            return false;
        }
        Collide_LeftLedgeGrab
    } else {
        if !ext::mpColl_800443C4(&mut coll, &mut f.coll_data.ledge_id_right) {
            return false;
        }
        Collide_RightLedgeGrab
    };
    f.coll_data.env_flags |= hit as i32;
    f.self_vel.x = 0.0;
    f.self_vel.y = 0.0;
    true
}

fn zair_input(f: &Fighter) -> bool {
    f.input.held_buttons[0] & HSD_PAD_LR as HSD_Pad != 0
        && f.input.pressed_buttons & HSD_PAD_A as HSD_Pad != 0
}

/// `ftCo_800C3B10`: Z-air tether check.
pub fn ftco_800c3b10(g: G) -> bool {
    melee_core::ct!("ftco_800c3b10");
    let f = fp(g);
    if !ext::ScriptGame_CapsActionAllowed(f, 8) {
        return false;
    }
    if f.used_tether() != 0 {
        return false;
    }
    if !is_link(f) && f.kind != Ft_Kind_Samus {
        if ext::Mex_HasZairHook(f.kind as i32) && zair_input(f) {
            ext::Mex_OnZairDispatch(f.kind as i32, g);
            f.set_used_tether(1);
            return true;
        }
        return false;
    }
    if f.accessory2_cb.is_some() || f.death1_cb.is_some() || f.accessory3_cb.is_some() {
        return false;
    }
    if !f.item_gobj.is_null() {
        return false;
    }
    if zair_input(f) {
        ftco_800c3be8(g);
        f.set_used_tether(1);
        return true;
    }
    false
}

/// `ftCo_800C3BE8`: enter AirCatch (air only).
pub fn ftco_800c3be8(g: G) {
    melee_core::ct!("ftco_800c3be8");
    let f = fp(g);
    if f.ground_or_air == GA_Air {
        unsafe { f.mv.co.aircatch.x0 = 0.0 };
        f.x74_self_accel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let nul = core::ptr::null_mut();
        if is_link(f) {
            ext::Fighter_ChangeMotionState(g, ftLk_MS_AirCatch as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, nul);
        } else if f.kind == Ft_Kind_Samus {
            ext::Fighter_ChangeMotionState(g, ftSs_MS_AirCatch as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, nul);
        }
        co2::ftcommon_8007e2f4(f, 0x1FF);
    }
}

/// `ftCo_800C3CC0`: enter AirCatchHit.
pub fn ftco_800c3cc0(g: G) {
    melee_core::ct!("ftco_800c3cc0");
    let f = fp(g);
    let ms = if is_link(f) { ftLk_MS_AirCatchHit as i32 } else { ftSs_MS_AirCatchHit as i32 };
    ext::Fighter_ChangeMotionState(g, ms, Ft_MF_KeepFastFall as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let drift = f.co_attrs.air_drift_max;
    co::clamp_self_vel_x(f, drift);
    unsafe {
        f.mv.co.aircatchhit.x0 = 20;
        f.mv.co.aircatchhit.x4 = 0.0;
    }
    if f.ground_or_air == GA_Ground {
        co2::ftcommon_8007d5d4(f);
    }
}

#[inline(always)]
fn tick(f: &mut Fighter) -> f32 {
    unsafe {
        f.mv.co.aircatch.x0 = (f.mv.co.aircatch.x0 as f64 + 1.0) as f32;
        f.mv.co.aircatch.x0
    }
}

fn tether_attrs(item: *mut Item_GObj) -> *mut TetherAttributes {
    unsafe {
        let ip = (*item).user_data as *mut Item;
        (*(*ip).xC4_article_data).x4_specialAttributes as *mut TetherAttributes
    }
}

fn link_sfx(f: &mut Fighter, clink: i32, link: i32) {
    let id = if f.kind == Ft_Kind_CLink { clink } else { link };
    h::play_sfx(f, id, 0x7F, 0x40);
}

/// `ftCo_AirCatch_Anim`.
pub fn ftco_aircatch_anim(g: G) {
    melee_core::ct!("ftco_aircatch_anim");
    let f = fp(g);
    if is_link(f) {
        let da = unsafe { &*(f.dat_attrs as *const ftLk_DatAttrs) };
        let t = tick(f);
        if t == da.xA4 as f32 {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let b = crate::ftdmgfall::ftparts_get_bone_index(f, FtPart_RThumbNb as i32);
            ss::lb_8000B1CC(unsafe { (*f.parts.offset(b as isize)).joint }, None, &mut pos);
            let it = ext::it_802A2BA4(g, &mut pos, f.facing_dir, da.xBC as i32);
            f.u.lk.xC = it;
            if it.is_null() {
                ext::ftCo_800968C8(g);
            } else {
                f.accessory2_cb = Some(ext::it_802A7AF0);
                f.death1_cb = Some(ext::it_802A7AAC);
                f.accessory3_cb = Some(ext::it_802A7B34);
            }
        } else if t > da.xA4 as f32 && t <= da.xB0 as f32 {
            let tg = unsafe { f.u.lk.xC };
            let td = tether_attrs(tg);
            if t == da.xA8 as f32 {
                let mut pos = Vec3 { x: 1.8, y: 0.0, z: 0.0 };
                let b = crate::ftdmgfall::ftparts_get_bone_index(f, FtPart_RThumbNb as i32);
                let jobj = unsafe { (*f.parts.offset(b as isize)).joint };
                hsd_jobj_setup_matrix(jobj);
                let m = unsafe { &(*jobj).mtx };
                let x1 = (8.0 * f.facing_dir as f64).mul_add(f.x34_scale.y as f64, m[0][3] as f64) as f32;
                if ss::mpCheckAllRemap(-1, -1, f.coll_data.cur_pos.x, m[1][3], x1, m[1][3]) {
                    ext::it_802A2B10(unsafe { f.u.lk.xC });
                    ext::ftCo_800968C8(g);
                    return;
                }
                pos.x = unsafe { (*td).pos_x_0 };
                pos.x *= f.facing_dir;
                pos.x += f.pos_delta.x;
                ext::it_802A78B8(tg, &mut pos);
                link_sfx(f, 0x111B9, 0x27149);
            } else if t == da.xAC as f32 {
                ext::it_802A77DC(tg);
                link_sfx(f, 0x111BC, 0x2714C);
            } else if t == da.xB0 as f32 {
                ext::it_802A2B10(unsafe { f.u.lk.xC });
            }
        }
    } else if f.kind == Ft_Kind_Samus {
        let da = unsafe { &*(f.dat_attrs as *const ftSs_DatAttrs) };
        let t = tick(f);
        if t == da.xBC as f32 {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ss::lb_8000B1CC(unsafe { (*f.parts.add(FtPart_ThrowN as usize)).joint }, None, &mut pos);
            let it = ss::it_802B7C18(g, &mut pos, f.facing_dir);
            f.u.ss.x223C = it;
            if it.is_null() {
                ext::ftCo_800968C8(g);
            } else {
                f.accessory2_cb = Some(ss::it_802BAC80);
                f.death1_cb = Some(ss::it_802BAC3C);
                f.accessory3_cb = Some(ss::it_802BACC4);
            }
        } else if t > da.xBC as f32 && t <= da.xC8 as f32 {
            let tg = unsafe { f.u.ss.x223C };
            let td = tether_attrs(tg);
            if t == da.xC0 as f32 {
                let mut pos = Vec3 { x: 1.8, y: 0.0, z: 0.0 };
                let jobj = unsafe { (*f.parts.add(FtPart_ThrowN as usize)).joint };
                hsd_jobj_setup_matrix(jobj);
                let m = unsafe { &(*jobj).mtx };
                let x1 = (2.0 * f.facing_dir as f64).mul_add(f.x34_scale.y as f64, m[0][3] as f64) as f32;
                if ss::mpCheckAllRemap(-1, -1, f.coll_data.cur_pos.x, m[1][3], x1, m[1][3]) {
                    ss::it_802B7B84(unsafe { f.u.ss.x223C });
                    ext::ftCo_800968C8(g);
                    return;
                }
                pos.x = unsafe { (*td).pos_x_1 };
                pos.x *= f.facing_dir;
                pos.x += f.pos_delta.x;
                ss::it_802BAAE4(tg, &mut pos);
            } else if t == da.xC4 as f32 {
                ss::it_802BAA58(tg);
            } else if t == da.xC8 as f32 {
                ss::it_802B7B84(unsafe { f.u.ss.x223C });
            }
        }
    }
    if !h::ftanim_is_frames_remaining(g) {
        ext::ftCo_800968C8(g);
    }
}

/// `ftCo_AirCatchHit_Anim` (empty).
pub fn ftco_aircatchhit_anim(_g: G) { melee_core::ct!("ftco_aircatchhit_anim");}
/// `ftCo_AirCatch_IASA` (empty).
pub fn ftco_aircatch_iasa(_g: G) { melee_core::ct!("ftco_aircatch_iasa");}
/// `ftCo_AirCatchHit_IASA` (empty).
pub fn ftco_aircatchhit_iasa(_g: G) { melee_core::ct!("ftco_aircatchhit_iasa");}

/// `ftCo_AirCatch_Phys`.
pub fn ftco_aircatch_phys(g: G) {
    melee_core::ct!("ftco_aircatch_phys");
    let f = fp(g);
    co::check_fall_fast(f);
    let (gr, tv) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    if f.fall_fast() != 0 {
        co::fall_fast(f);
    } else if unsafe { f.mv.co.aircatch.x0 } < 20.0 && (f.pos_delta.y as f64) < 0.0 {
        co::fall(f, (gr as f64 * 0.2) as f32, tv);
    } else {
        co::fall(f, gr, tv);
    }
    co::calc_self_accel_drift(f);
}

/// `ftCo_AirCatchHit_Phys`.
pub fn ftco_aircatchhit_phys(g: G) {
    melee_core::ct!("ftco_aircatchhit_phys");
    let f = fp(g);
    f.self_vel = f.pos_delta;
    let (gr, tv) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
    co::fall(f, gr, tv);
}

/// `ftCo_AirCatch_Coll`.
pub fn ftco_aircatch_coll(g: G) {
    melee_core::ct!("ftco_aircatch_coll");
    if ext::ft_80081D0C(g) != GA_Ground {
        ext::ftCo_Landing_Enter(g, ftCo_MS_Landing as i32, false, Ft_MF_None as u32, 0.0, 1.0);
    }
}
