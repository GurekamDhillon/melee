//! src/melee/ft/ftstarrodswing.c: Peach's Star Rod swing state (setup, callbacks, the
//! accessory callback that plays the swing SFX and fires the star).
#![allow(non_snake_case)]

use crate::types::*;
use core::ffi::c_void;

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    pub fn ftCo_800CD140(gobj: *mut HSD_GObj, a: i32, b: i32, c: i32, f: f32) { { unsafe { let _ = crate::tu::ft_0cd1::ftco_800cd140(gobj as _, a as _, b as _, c as _, f as _); } return; } }
    pub fn ftCo_800CD1BC(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ft_0cd1::ftco_800cd1bc(gobj as _); } return; } }
    pub fn ftCo_800CD204(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ft_0cd1::ftco_800cd204(gobj as _); } return; } }
    pub fn ftCo_800CD278(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ft_0cd1::ftco_800cd278(gobj as _); } return; } }
    pub fn ftCo_800CD2C4(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) { panic!("TODO(ft_0CD1) ftCo_800CD2C4") }
    pub unsafe extern "C" fn ft_800CD31C(gobj: *mut HSD_GObj) { { unsafe { let _ = crate::tu::ft_0cd1::ft_800cd31c(gobj as _); } return; } }
    pub fn it_80292394(item: *mut Item_GObj, v: &mut Vec3) { { unsafe { let _ = melee_items::tu::itstarrod::it_80292394(item as _, v as _); } return; } }
    pub fn it_802923BC(item: *mut Item_GObj, v: &Vec3, dir: f32, arg: u32) { panic!("TODO(group5 itstarrod) it_802923BC") }
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, a: *const Vec3, b: &mut Vec3) { { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, a as _, b as _); } return; } }
    pub fn efAsync_Spawn(gobj: *mut HSD_GObj, q: *mut c_void, kind: u32, gfx: u32, jobj: *mut HSD_JObj, pos: &Vec3, f: *const f32) {
        panic!("TODO(group2 efasync) efAsync_Spawn")
    }
}

use super::ftlipstickswing::{fp, item, ft_check_throw_b0};

/// `ftCo_800CD82C(gobj, anim, swing, f)`: enter the Star Rod swing.
pub fn ftCo_800CD82C(gobj: *mut HSD_GObj, arg1: i32, arg2: i32, arg3: f32) {
    let f = fp(gobj);
    f.cmd_vars[0] = 540000;
    f.cmd_vars[1] = 0;
    ext::ftCo_800CD140(gobj, arg1, 0, arg2, arg3);
    f.accessory4_cb = Some(ft_800CD914_cb);
    f.take_dmg_cb = Some(ext::ft_800CD31C);
}

pub fn ftCo_StarRodSwing_Anim(gobj: *mut HSD_GObj) { ext::ftCo_800CD1BC(gobj) }
pub fn ftCo_StarRodSwing_IASA(gobj: *mut HSD_GObj) { ext::ftCo_800CD204(gobj) }
pub fn ftCo_StarRodSwing_Phys(gobj: *mut HSD_GObj) { ext::ftCo_800CD278(gobj) }
pub fn ftCo_StarRodSwing_Coll(gobj: *mut HSD_GObj) { ext::ftCo_800CD2C4(gobj, Some(ext::ft_800CD31C)) }

unsafe extern "C" fn ft_800CD914_cb(gobj: *mut HSD_GObj) { ft_800CD914(gobj) }

/// `ft_800CD914(gobj)`: accessory callback. The C asserts the item kind before its NULL check
/// (debug-only assert: here it is checked only when the item exists).
pub fn ft_800CD914(gobj: *mut HSD_GObj) {
    let f = fp(gobj);
    let x = unsafe { (*(*f.ft_data).x8).x10 } as usize;
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut v2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let it = f.item_gobj;
    if it.is_null() {
        return;
    }
    debug_assert!(melee_items::itlib::it_get_kind(item(it)) == It_Kind_StarRod);
    if f.cmd_vars[0] != 540000 {
        if !melee_items::itlib::it_8026b594(item(it)) {
            crate::hooks::play_sfx(f, f.cmd_vars[0] as i32, 127, 64);
        } else {
            crate::hooks::play_sfx(f, 234, 127, 64);
        }
        f.cmd_vars[0] = 540000;
    }
    if ft_check_throw_b0(f) {
        ext::it_80292394(it, &mut v);
        let joint = unsafe { (*f.parts.add(x)).joint };
        ext::lb_8000B1CC(joint, &v, &mut v2);
        if !melee_items::itlib::it_8026b594(item(it)) {
            let q = &mut f.x60C as *mut *mut c_void as *mut c_void;
            let dir = f.facing_dir;
            ext::efAsync_Spawn(gobj, q, 5, 1070, core::ptr::null_mut(), &v2, &dir);
            ext::it_802923BC(it, &v2, f.facing_dir, f.cmd_vars[1]);
        } else {
            let z = 0.0f32;
            let q = &mut f.x60C as *mut *mut c_void as *mut c_void;
            ext::efAsync_Spawn(gobj, q, 5, 1032, core::ptr::null_mut(), &v2, &z);
        }
    }
}
