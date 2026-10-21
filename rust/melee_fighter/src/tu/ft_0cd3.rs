//! src/melee/ft/ft_0CD3.c: SwordSwing / BatSwing / ParasolSwing / HarisenSwing states, thin
//! wrappers over ft_0CD1. Float ops: Harisen timer `x8 -= 1` and compares (no fused site).
#![allow(non_snake_case)]

use super::ft_0cd1::{self as c, ext as e, fp, G};

/// Callees outside this TU that have no port yet (itharisen.c). Each panics until rewired.
pub mod ext {
    use crate::types::*;
    type I = *mut HSD_GObj;
    pub fn it_8029282C(_i: I) { todo!("TODO(group5 itharisen) it_8029282C") }
    pub fn it_80292A28(_i: I) { todo!("TODO(group5 itharisen) it_80292A28") }
    pub fn it_802927E8(_i: I) { todo!("TODO(group5 itharisen) it_802927E8") }
    pub fn it_80292998(_i: I, _f: f32) { todo!("TODO(group5 itharisen) it_80292998") }
    pub fn it_802929C8(_i: I, _f: f32) { todo!("TODO(group5 itharisen) it_802929C8") }
    pub fn it_802929F8(_i: I, _f: f32) { todo!("TODO(group5 itharisen) it_802929F8") }
}

fn enter(g: G, arg1: i32, flags: i32, arg2: i32, arg3: f32) {
    c::ftco_800cd140(g, arg1, flags, arg2, arg3);
    fp(g).take_dmg_cb = Some(c::ft_800cd31c);
}

/// `ftCo_800CD350`: SwordSwing enter.
pub fn ftco_800cd350(g: G, arg1: i32, arg2: i32, arg3: f32) { enter(g, arg1, 0, arg2, arg3) }
pub fn ftco_swordswing_anim(g: G) { c::ftco_800cd1bc(g) }
pub fn ftco_swordswing_iasa(g: G) { c::ftco_800cd204(g) }
pub fn ftco_swordswing_phys(g: G) { c::ftco_800cd278(g) }
pub fn ftco_swordswing_coll(g: G) { c::ftco_800cd2c4(g, c::ft_800cd31c) }

/// `ftCo_800CD418`: BatSwing enter.
pub fn ftco_800cd418(g: G, arg1: i32, arg2: i32, arg3: f32) { enter(g, arg1, 0, arg2, arg3) }
pub fn ftco_batswing_anim(g: G) { c::ftco_800cd1bc(g) }
pub fn ftco_batswing_iasa(g: G) { c::ftco_800cd204(g) }
pub fn ftco_batswing_phys(g: G) { c::ftco_800cd278(g) }
pub fn ftco_batswing_coll(g: G) { c::ftco_800cd2c4(g, c::ft_800cd31c) }

/// `ftCo_800CD4E0`: ParasolSwing enter (motion flags 0x400).
pub fn ftco_800cd4e0(g: G, arg1: i32, arg2: i32, arg3: f32) { enter(g, arg1, 0x400, arg2, arg3) }
pub fn ftco_parasolswing_anim(g: G) { c::ftco_800cd1bc(g) }
pub fn ftco_parasolswing_iasa(g: G) { c::ftco_800cd204(g) }
pub fn ftco_parasolswing_phys(g: G) { c::ftco_800cd278(g) }
pub fn ftco_parasolswing_coll(g: G) { c::ftco_800cd2c4(g, c::ft_800cd31c) }

/// `ftCo_800CD5A8`: HarisenSwing enter.
pub fn ftco_800cd5a8(g: G, arg1: i32, arg2: i32, arg3: f32) {
    let f = fp(g);
    f.cmd_vars[0] = 0;
    unsafe { f.mv.co.swing.x8 = 0.0 };
    c::ftco_800cd140(g, arg1, 0, arg2, arg3);
    let f = fp(g);
    f.deal_dmg_cb = Some(ft_800cd65c);
    f.take_dmg_cb = Some(ft_800cd604);
}

/// `ft_800CD604`: Harisen take-damage callback.
pub extern "C" fn ft_800cd604(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() {
        ext::it_8029282C(it);
        ext::it_80292A28(it);
        crate::ftcommon3::ftcommon_8007e7e4(g, 1);
    }
}

/// `ft_800CD65C`: Harisen deal-damage callback.
pub extern "C" fn ft_800cd65c(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() {
        ext::it_802927E8(it);
        unsafe { fp(g).mv.co.swing.x8 = crate::hooks::common_data().x5A0 };
    }
}

/// `ftCo_HarisenSwing_Anim`.
pub fn ftco_harisenswing_anim(g: G) {
    let it = fp(g).item_gobj;
    if !it.is_null() {
        let f = fp(g);
        unsafe {
            if f.mv.co.swing.x8 > 0.0 {
                f.mv.co.swing.x8 -= 1.0;
                if f.mv.co.swing.x8 <= 0.0 || !e::ftAnim_IsFramesRemaining(g) {
                    ext::it_8029282C(fp(g).item_gobj);
                }
            }
        }
        let f = fp(g);
        if f.cmd_vars[0] == 1 {
            let sp = f.frame_speed_mul;
            let it = f.item_gobj;
            match unsafe { f.mv.co.swing.x4 } {
                0 => ext::it_80292998(it, sp),
                1 => ext::it_802929C8(it, sp),
                2 | 3 => ext::it_802929F8(it, sp),
                4 => ext::it_802929C8(it, sp),
                _ => {}
            }
            fp(g).cmd_vars[0] = 0;
        }
        let f = fp(g);
        if f.cmd_vars[0] == 2 {
            ext::it_80292A28(f.item_gobj);
            fp(g).cmd_vars[0] = 0;
        }
    }
    c::ftco_800cd1bc(g);
}
pub fn ftco_harisenswing_iasa(g: G) { c::ftco_800cd204(g) }
pub fn ftco_harisenswing_phys(g: G) { c::ftco_800cd278(g) }
pub fn ftco_harisenswing_coll(g: G) { c::ftco_800cd2c4(g, ft_800cd604) }
