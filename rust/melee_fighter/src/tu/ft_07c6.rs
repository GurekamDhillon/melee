//! Hand port of src/melee/ft/ft_07C6.c (all 3 functions: ft_8007C630 / ft_8007C6DC coin-pickup
//! probe positions, ft_8007C77C coin collection scan). No float arithmetic beyond copies, so no
//! fused sites; smoke-tested by tests/tu_ft_07C6.rs (callees stubbed).
#![allow(static_mut_refs)]

use crate::hooks;
use crate::types::*;

pub type G = *mut HSD_GObj;

/// Calls out of this TU: settable fn pointers, panicking with a TODO when reached unset
/// (except where a ported default exists).
pub mod ext {
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        LB_8000B1CC lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3);
        GM_8016B0B4 gm_8016B0B4() -> bool;
        GM_801A4BB8 gm_801A4BB8() -> u32;
        FTCOLL_8007B868 ftColl_8007B868(g: *mut HSD_GObj) -> i32;
        LBCOLL_80007B78 lbColl_80007B78(a: *mut Fighter_x1614_t, b: *mut Fighter_x1614_t, sa: f32, sb: f32) -> bool;
        IT_802E5EF4 it_802E5EF4(g: *mut HSD_GObj) -> i32;
        LBAUDIOAX_80023870 lbAudioAx_80023870(a: i32, b: i32, c: i32, d: i32);
        EFSYNC_SPAWN efSync_Spawn(id: i32, g: *mut HSD_GObj, pos: *mut Vec3, scale: *mut f32);
        ITEM_8026A8EC Item_8026A8EC(g: *mut HSD_GObj);
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ft_8007C630`: initialise both x1614 probes from `ft_data->x38`.
pub fn ft_8007c630(g: G) {
    melee_core::ct!("ft_8007c630");
    for i in 0..2usize {
        let f = fp(g);
        let d = unsafe { &mut *(*f.ft_data).x38.add(i) };
        let joint = unsafe { (*f.parts.add(d.x0 as usize)).joint };
        let t = &mut f.x1614[i] as *mut Fighter_x1614_t;
        unsafe {
            (*t).x4 = joint;
            (*t).x0 = d.x10;
            ext::lb_8000B1CC((*t).x4, &mut d.x4, &mut (*t).x8);
            (*t).x8.z = 0.0;
            (*t).x14 = (*t).x8;
        }
    }
}

/// `ft_8007C6DC`: per-frame probe update (previous position kept in x14).
pub fn ft_8007c6dc(g: G) {
    melee_core::ct!("ft_8007c6dc");
    if ext::gm_8016B0B4() {
        for i in 0..2usize {
            let f = fp(g);
            let d = unsafe { &mut *(*f.ft_data).x38.add(i) };
            let t = &mut f.x1614[i] as *mut Fighter_x1614_t;
            unsafe {
                (*t).x14 = (*t).x8;
                ext::lb_8000B1CC((*t).x4, &mut d.x4, &mut (*t).x8);
                (*t).x8.z = 0.0;
            }
        }
    }
}

/// `ft_8007C77C`: collect coin items (kind Unk4) overlapping either probe.
pub fn ft_8007c77c(g: G) {
    melee_core::ct!("ft_8007c77c");
    if !ext::gm_8016B0B4() {
        return;
    }
    let a = ext::gm_801A4BB8() & 1;
    let f = fp(g);
    let b = (f.player_id as u32) & 1;
    f.set_x2229_b4(if a == b { 0 } else { 1 });
    if f.x2229_b4() != 0 {
        return;
    }
    if ext::ftColl_8007B868(g) == 2 {
        return;
    }
    let mut cur = hooks::plink_head(HSD_GOBJ_PLINK_ITEM);
    while !cur.is_null() {
        let next = unsafe { (*cur).next };
        let ip = unsafe { &mut *((*cur).user_data as *mut Item) };
        if ip.kind as i32 == It_Kind_Unk4 as i32 {
            let v = unsafe { &mut ip.xDD4_itemVar.unk4 };
            let take = if crate::ftlib::ftlib_80086fd4(g, ip.owner) {
                v.xDEC_b1() != 0
            } else {
                v.xDEC_b0() != 0
            };
            if take {
                let t26 = &mut v.xDF0 as *mut Fighter_x1614_t;
                for i in 0..2usize {
                    let f = fp(g);
                    let tmp = &mut f.x1614[i] as *mut Fighter_x1614_t;
                    if ext::lbColl_80007B78(t26, tmp, ip.scl, f.x34_scale.y) {
                        let n = ext::it_802E5EF4(ip.entity);
                        let pid = fp(g).player_id as i32;
                        crate::player::set_coins(pid, n.wrapping_add(crate::player::get_coins(pid)));
                        crate::player::set_total_coins(pid, n.wrapping_add(crate::player::get_total_coins(pid)));
                        ext::lbAudioAx_80023870(0x93, 0x7F, 0x40, 0x1A);
                        let mut sp18 = 1.0f32;
                        unsafe { ext::efSync_Spawn(0x432, core::ptr::null_mut(), &mut (*t26).x20, &mut sp18) };
                        ext::Item_8026A8EC(ip.entity);
                        break;
                    }
                }
            }
        }
        cur = next;
    }
}
