//! src/melee/ft/kinds/ftCommon/ftCo_0D95.c: Samus grab grapple (spawn, sparkle effects along
//! the beam, fire, retract, destroy on fixed frames of ftSs_DatAttrs), for both grab states.
//!
//! Fused sites (tools/rust-port/fma_sites.py): fn_800D9558 has four f64 fmuladd (the three
//! `4.0 * (r - 0.5f) + effPos.c` sparkle jitters and the wall-probe end
//! `2.0 * facing_dir * scale.y + mtx[0][3]`), fn_800D9930 three (the jitters); each is one f64
//! fma, then rounded to f32. The rest have none.
//! Callees outside the ported set are `ext` stubs with override slots. Checked against the C by
//! tests/tu_ftco_0d95.rs.
#![allow(non_snake_case)]

use super::ftco_0d8e::hsd_jobj_setup_matrix;
use crate::hooks as h;
use crate::types::*;

pub type G = *mut Fighter_GObj;

macro_rules! stubs {
    ($($slot:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
        pub static mut $slot: Option<fn($($t),*) $(-> $r)?> = None;
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $slot } {
                Some(f) => f($($a),*),
                None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
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
        IT_802B7C18 it_802B7C18(g: G, pos: &mut Vec3, facing: f32) -> *mut Item_GObj;
        IT_802B7B84 it_802B7B84(item: *mut Item_GObj);
        IT_802BAAE4 it_802BAAE4(item: *mut Item_GObj, vel: &mut Vec3);
        IT_802BAA58 it_802BAA58(item: *mut Item_GObj);
        MP_CHECK_ALL_REMAP mpCheckAllRemap(skip: i32, only: i32, x0: f32, y0: f32, x1: f32, y1: f32) -> bool;
        HSD_RANDF HSD_Randf() -> f32;
        EF_SYNC_SPAWN efSync_Spawn(id: i32, g: G, pos: &mut Vec3);
    }
    /// TODO(group5 itsamusgrapple.c) callbacks stored in the fighter's accessory / death slots.
    pub unsafe extern "C" fn it_802BAC80(_g: *mut HSD_GObj) { { unsafe { let _ = melee_items::tu::itsamusgrapple::it_802BAC80(_g as _); } return; } }
    pub unsafe extern "C" fn it_802BAC3C(_g: *mut HSD_GObj) { { unsafe { let _ = melee_items::tu::itsamusgrapple::it_802BAC3C(_g as _); } return; } }
    pub unsafe extern "C" fn it_802BACC4(_g: *mut HSD_GObj) { { unsafe { let _ = melee_items::tu::itsamusgrapple::it_802BACC4(_g as _); } return; } }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `4.0 * d + c` in double (one fma), rounded to f32.
#[inline(always)]
fn jitter(d: f32, c: f32) -> f32 {
    4.0f64.mul_add(d as f64, c as f64) as f32
}

/// Shared body of fn_800D9558 / fn_800D9930. `a` = (spawn, fire, retract, destroy) frames,
/// `sparks` = sparkle count, `probe` = fn_800D9558's wall probe before firing.
fn grapple(g: G, a: [i32; 4], sparks: i32, probe: bool) -> bool {
    let f = fp(g);
    if f.kind != Ft_Kind_Samus {
        return false;
    }
    let grav = unsafe {
        f.mv.ca.specials.grav = (f.mv.ca.specials.grav as f64 + 1.0) as f32;
        f.mv.ca.specials.grav
    };
    if grav == a[0] as f32 {
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(unsafe { (*f.parts.add(51)).joint }, None, &mut pos);
        let it = ext::it_802B7C18(g, &mut pos, f.facing_dir);
        f.u.ss.x223C = it;
        if it.is_null() {
            h::ft_8008a2bc(g);
            return true;
        }
        f.accessory2_cb = Some(ext::it_802BAC80);
        f.death1_cb = Some(ext::it_802BAC3C);
        f.accessory3_cb = Some(ext::it_802BACC4);
    } else if grav > a[0] as f32 && grav <= a[3] as f32 {
        let item = unsafe { f.u.ss.x223C };
        let ip = unsafe { (*item).user_data as *mut Item };
        let ga = unsafe { (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusGrappleAttributes };
        if !item.is_null() {
            for i in 0..sparks {
                if unsafe { f.mv.ca.specials.grav } == (i * 3 + 0x14) as f32 {
                    let seg = unsafe { (*(*ip).xDD4_itemVar.samusgrapple.x0).gobj };
                    let jobj = unsafe { (*seg).hsd_obj as *mut HSD_JObj };
                    hsd_jobj_setup_matrix(jobj);
                    let m = unsafe { &(*jobj).mtx };
                    let mut e = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
                    let r = ext::HSD_Randf();
                    e.x = jitter(r - 0.5, e.x);
                    e.y = jitter(ext::HSD_Randf() - 0.5, e.y);
                    let r = ext::HSD_Randf() - 0.5;
                    e.z = jitter(r, e.z);
                    ext::efSync_Spawn(0x3F3, seg, &mut e);
                }
            }
        }
        let grav = unsafe { f.mv.ca.specials.grav };
        if grav == a[1] as f32 {
            if probe {
                let joint = unsafe { (*f.parts.add(51)).joint };
                hsd_jobj_setup_matrix(joint);
                let m = unsafe { &(*joint).mtx };
                let my = m[1][3];
                let x1 = (2.0 * f.facing_dir as f64).mul_add(f.x34_scale.y as f64, m[0][3] as f64) as f32;
                if ext::mpCheckAllRemap(-1, -1, f.coll_data.cur_pos.x, my, x1, my) {
                    ext::it_802B7B84(unsafe { f.u.ss.x223C });
                    h::ft_8008a2bc(g);
                    return true;
                }
            }
            let mut vel = Vec3 { x: unsafe { (*ga).x40 } * f.facing_dir, y: 0.0, z: 0.0 };
            ext::it_802BAAE4(item, &mut vel);
        } else if grav == a[2] as f32 {
            ext::it_802BAA58(item);
        } else if grav == a[3] as f32 {
            ext::it_802B7B84(unsafe { f.u.ss.x223C });
        }
    }
    false
}

fn da(f: &Fighter) -> &ftSs_DatAttrs {
    unsafe { &*(f.dat_attrs as *const ftSs_DatAttrs) }
}

/// `fn_800D9558` (frames x9C..xA8, six sparkles, wall probe).
pub fn fn_800d9558(g: G) -> bool {
    melee_core::ct!("fn_800d9558");
    let f = fp(g);
    if f.kind != Ft_Kind_Samus {
        return false;
    }
    let a = da(f);
    grapple(g, [a.x9C, a.xA0, a.xA4, a.xA8], 6, true)
}

/// `fn_800D9930` (frames xAC..xB8, four sparkles, no probe).
pub fn fn_800d9930(g: G) -> bool {
    melee_core::ct!("fn_800d9930");
    let f = fp(g);
    if f.kind != Ft_Kind_Samus {
        return false;
    }
    let a = da(f);
    grapple(g, [a.xAC, a.xB0, a.xB4, a.xB8], 4, false)
}

/// `fn_800D9C64`.
pub fn fn_800d9c64(g: G) {
    melee_core::ct!("fn_800d9c64");
    let f = fp(g);
    if f.kind == Ft_Kind_Samus {
        ext::it_802B7B84(unsafe { f.u.ss.x223C });
    }
}

/// `ftCo_800D9C98`.
pub fn ftco_800d9c98(g: G) {
    melee_core::ct!("ftco_800d9c98");
    let f = fp(g);
    unsafe {
        if !f.u.ss.x223C.is_null() {
            ext::it_802B7B84(f.u.ss.x223C);
            f.u.ss.x223C = core::ptr::null_mut();
        }
    }
    f.death2_cb = None;
    f.take_dmg_2_cb = None;
    f.take_dmg_cb = None;
}
