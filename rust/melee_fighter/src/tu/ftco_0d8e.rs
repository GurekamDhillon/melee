//! src/melee/ft/kinds/ftCommon/ftCo_0D8E.c: Link / Young Link grab tether (the hookshot item
//! spawned, fired, pulled back and destroyed on fixed frames of the dat attributes), and the
//! Samus "has no grapple out" test.
//!
//! Fused sites (tools/rust-port/fma_sites.py): fn_800D8EC8 has two f64 fmuladd, the hookshot
//! wall-probe end `8.0 * facing_dir * scale.y + mtx[0][3]` (both kind branches) -> one f64 fma,
//! then rounded to f32. Everything else has none.
//! `ftCo_LinkCatchAttrs` is the C file's local view of `dat_attrs` (s32 frames at 0x84..0xA0,
//! 0xBC); it is read by offset here.
//! Callees outside the ported set are `ext` stubs with override slots (tests install traces).
//! Checked against the C by tests/tu_ftco_0d8e.rs.
#![allow(non_snake_case)]

use melee_core::fp::Fmadd as _;
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
        LB_8000B1CC lb_8000B1CC(j: *mut HSD_JObj, off: Option<&Vec3>, out: &mut Vec3);
        IT_802A2BA4 it_802A2BA4(g: G, pos: &mut Vec3, facing: f32, x: i32) -> *mut Item_GObj;
        IT_802A2B10 it_802A2B10(item: *mut Item_GObj);
        IT_802A78B8 it_802A78B8(item: *mut HSD_GObj, vel: &mut Vec3);
        IT_802A77DC it_802A77DC(item: *mut Item_GObj);
        MP_CHECK_ALL_REMAP mpCheckAllRemap(skip: i32, only: i32, x0: f32, y0: f32, x1: f32, y1: f32) -> bool;
        HSD_JOBJ_SETUP_MATRIX_SUB HSD_JObjSetupMatrixSub(j: *mut HSD_JObj);
        FTLK_INIT_BOOMERANG_EXISTS ftLk_Init_BoomerangExists(g: G) -> bool;
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

/// `HSD_JObjSetupMatrix` (jobj.h inline).
pub fn hsd_jobj_setup_matrix(j: *mut HSD_JObj) {
    melee_core::ct!("hsd_jobj_setup_matrix");
    if j.is_null() || !super::ftco_0c35::hsd_jobj_mtx_is_dirty(j) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(j);
}

/// `ftCo_LinkCatchAttrs` field at byte offset `off` of `dat_attrs`.
#[inline(always)]
fn la(f: &Fighter, off: usize) -> i32 {
    unsafe { *((f.dat_attrs as *const u8).add(off) as *const i32) }
}

fn joint(f: &Fighter, part: i32) -> *mut HSD_JObj {
    let i = h::ftparts_get_bone_index(f, part);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// The tether item's `xC4_article_data->x4_specialAttributes`.
fn special_attrs<T>(item: *mut Item_GObj) -> *mut T {
    unsafe { (*(*((*item).user_data as *mut Item)).xC4_article_data).x4_specialAttributes as *mut T }
}

fn is_link(f: &Fighter) -> bool {
    f.kind == Ft_Kind_Link || f.kind == Ft_Kind_CLink
}

fn spawn_hookshot(g: G, f: &mut Fighter) -> bool {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint(f, FtPart_RThumbNb as i32), None, &mut pos);
    let it = ext::it_802A2BA4(g, &mut pos, f.facing_dir, la(f, 0xBC));
    f.u.lk.xC = it;
    if it.is_null() {
        h::ft_8008a2bc(g);
        return true;
    }
    f.accessory2_cb = Some(ext::it_802A7AF0);
    f.death1_cb = Some(ext::it_802A7AAC);
    f.accessory3_cb = Some(ext::it_802A7B34);
    false
}

fn sfx_fire(f: &mut Fighter) {
    if f.kind == Ft_Kind_CLink { h::play_sfx(f, 0x111B9, 0x7F, 0x40) } else { h::play_sfx(f, 0x27149, 0x7F, 0x40) }
}
fn sfx_pull(f: &mut Fighter) {
    if f.kind == Ft_Kind_CLink { h::play_sfx(f, 0x111BC, 0x7F, 0x40) } else { h::play_sfx(f, 0x2714C, 0x7F, 0x40) }
}

/// `fn_800D8EC8`: the tether frame logic of one grab state (frame counter in mv.ca.specials.grav).
pub fn fn_800d8ec8(g: G) -> bool {
    melee_core::ct!("fn_800d8ec8");
    let f = fp(g);
    if !is_link(f) {
        return false;
    }
    let grav = unsafe {
        f.mv.ca.specials.grav = (f.mv.ca.specials.grav as f64 + 1.0) as f32;
        f.mv.ca.specials.grav
    };
    if grav == la(f, 0x84) as f32 {
        return spawn_hookshot(g, f);
    } else if grav > la(f, 0x84) as f32 && grav <= la(f, 0x90) as f32 {
        let item = unsafe { f.u.lk.xC };
        let hook_attrs: *mut itLinkHookshotAttributes = special_attrs(item);
        if grav == la(f, 0x88) as f32 {
            let jobj = joint(f, FtPart_RThumbNb as i32);
            hsd_jobj_setup_matrix(jobj);
            let m = unsafe { &(*jobj).mtx };
            // fmuladd(8.0 * facing, scale.y, mtx[0][3]) in f64 (both kind branches are the same)
            let var_f3 = (8.0 * f.facing_dir as f64).fmadd(f.x34_scale.y as f64, m[0][3] as f64) as f32;
            let my = m[1][3];
            if ext::mpCheckAllRemap(-1, -1, f.coll_data.cur_pos.x, my, var_f3, my) {
                ext::it_802A2B10(unsafe { f.u.lk.xC });
                h::ft_8008a2bc(g);
                return true;
            }
            let mut vel = Vec3 { x: unsafe { (*hook_attrs).x38 } * f.facing_dir, y: 0.0, z: 0.0 };
            ext::it_802A78B8(item, &mut vel);
            sfx_fire(f);
        } else if grav == la(f, 0x8C) as f32 {
            ext::it_802A77DC(item);
            sfx_pull(f);
        } else if grav == la(f, 0x90) as f32 {
            ext::it_802A2B10(unsafe { f.u.lk.xC });
        }
    }
    false
}

/// `fn_800D9228`: the same for the other grab state (counter in mv.co.catch_.x0, frames x94..xA0).
pub fn fn_800d9228(g: G) -> bool {
    melee_core::ct!("fn_800d9228");
    let f = fp(g);
    if !is_link(f) {
        return false;
    }
    let t = unsafe {
        f.mv.co.catch_.x0 = (f.mv.co.catch_.x0 as f64 + 1.0) as f32;
        f.mv.co.catch_.x0
    };
    if t == la(f, 0x94) as f32 {
        return spawn_hookshot(g, f);
    } else if t > la(f, 0x94) as f32 && t <= la(f, 0xA0) as f32 {
        let tether = unsafe { f.u.lk.xC };
        let td: *mut TetherAttributes = special_attrs(tether);
        if t == la(f, 0x98) as f32 {
            let mut pos = Vec3 { x: unsafe { (*td).pos_x_0 } * f.facing_dir, y: 0.0, z: 0.0 };
            ext::it_802A78B8(tether, &mut pos);
            sfx_fire(f);
        } else if t == la(f, 0x9C) as f32 {
            ext::it_802A77DC(tether);
            sfx_pull(f);
        } else if t == la(f, 0xA0) as f32 {
            ext::it_802A2B10(tether);
        }
    }
    false
}

/// `fn_800D949C`.
pub fn fn_800d949c(g: G) {
    melee_core::ct!("fn_800d949c");
    let f = fp(g);
    if is_link(f) {
        ext::it_802A2B10(unsafe { f.u.lk.xC });
    }
}

/// `ftCo_800D94D8`.
pub fn ftco_800d94d8(g: G) {
    melee_core::ct!("ftco_800d94d8");
    let f = fp(g);
    unsafe {
        if !f.u.lk.xC.is_null() {
            ext::it_802A2B10(f.u.lk.xC);
            f.u.lk.xC = core::ptr::null_mut();
        }
    }
    ext::ftLk_Init_BoomerangExists(g);
}

/// `fn_800D952C`: false only for a Samus with a grapple out.
pub fn fn_800d952c(g: G) -> bool {
    melee_core::ct!("fn_800d952c");
    let f = fp(g);
    !(f.kind == Ft_Kind_Samus && unsafe { !f.u.ss.x223C.is_null() })
}
