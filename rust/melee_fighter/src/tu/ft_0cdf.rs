//! src/melee/ft/ft_0CDF.c: Ray Gun (LGunShoot / LGunShootAir) states. No float math beyond
//! copies; no fused sites.
#![allow(non_snake_case)]

use super::ft_0cd1::{ext as e, fp, G};
use crate::hooks as h;
use crate::types::*;

/// `it_8026B594` through `gobj->user_data` (group 5's melee_items::itlib port, verified there).
pub fn it_8026b594(i: *mut HSD_GObj) -> bool {
    unsafe { melee_items::itlib::it_8026b594(&*((*i).user_data as *const Item)) }
}

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn it_8028E938(_i: G) { { unsafe { let _ = melee_items::tu::itlgun::it_8028E938(_i as _); } return; } }
    pub fn it_8028E774(_i: G, _v: &mut Vec3) { { unsafe { let _ = melee_items::tu::itlgun::it_8028E774(_i as _, _v as _); } return; } }
    pub fn it_8028E79C(_i: G, _v: &Vec3, _dir: f32) { unsafe { let _ = melee_items::tu::itlgun::it_8028E79C(_i as _, _v as *const _ as *mut _, _dir as _); } }
    pub fn ft_800841B8(_g: G, _cb: unsafe extern "C" fn(G)) { todo!("TODO(group3 ft_081B) ft_800841B8") }
    pub fn ft_80082C74(_g: G, _cb: unsafe extern "C" fn(G)) { todo!("TODO(group3 ft_081B) ft_80082C74") }
    /// `efSync_Spawn(id, gobj, pos)`.
    pub fn efSync_Spawn(_id: i32, _g: G, _pos: &Vec3) { todo!("TODO(group2 ef) efSync_Spawn") }
    /// `efSync_Spawn(id, gobj, pos, &facing_dir)`.
    pub fn efSync_Spawn_dir(_id: i32, _g: G, _pos: &Vec3, _dir: &f32) { todo!("TODO(group2 ef) efSync_Spawn") }
    /// `efSync_Spawn(id, gobj, pos, &facing_dir, &f)`.
    pub fn efSync_Spawn_dir_f(_id: i32, _g: G, _pos: &Vec3, _dir: &f32, _f: &f32) { todo!("TODO(group2 ef) efSync_Spawn") }
}

fn shoot_enter(g: G, full: i32, empty: i32, air: bool) {
    unsafe { fp(g).anon_2210.throw_flags = 0 };
    let ms = if !it_8026b594(fp(g).item_gobj) { full } else { empty };
    e::Fighter_ChangeMotionState(g, ms, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    e::ftAnim_8006EBA4(g);
    if air {
        crate::ftcommon::clamp_air_drift(fp(g));
    }
    let f = fp(g);
    f.accessory4_cb = Some(ftco_800ce1d4);
    if !f.item_gobj.is_null() {
        ext::it_8028E938(f.item_gobj);
    }
}

/// `ftCo_800CDF5C`: grounded LGunShoot enter.
pub fn ftco_800cdf5c(g: G) {
    shoot_enter(g, ftCo_MS_LGunShoot as i32, ftCo_MS_LGunShootEmpty as i32, false)
}

/// `ftCo_800CE010`: aerial LGunShootAir enter.
pub fn ftco_800ce010(g: G) {
    shoot_enter(g, ftCo_MS_LGunShootAir as i32, ftCo_MS_LGunShootAirEmpty as i32, true)
}

/// `ftCo_800CE0CC`: landed (air -> ground) keeping the frame.
pub extern "C" fn ftco_800ce0cc(g: G) {
    let f = fp(g);
    crate::ftcommon3::ftcommon_8007d7fc(f);
    let id = if f.motion_id as i32 == ftCo_MS_LGunShootAir as i32 { ftCo_MS_LGunShoot } else { ftCo_MS_LGunShootEmpty };
    let (f1, f2) = (f.cur_anim_frame, f.frame_speed_mul);
    e::Fighter_ChangeMotionState(g, id as i32, 0x0C4C5080, f1, f2, 0.0, core::ptr::null_mut());
    fp(g).accessory4_cb = Some(ftco_800ce1d4);
}

/// `ftCo_800CE14C`: left the ground keeping the frame.
pub extern "C" fn ftco_800ce14c(g: G) {
    let f = fp(g);
    crate::ftcommon2::ftcommon_8007d5d4(f);
    let id = if f.motion_id as i32 == ftCo_MS_LGunShoot as i32 { ftCo_MS_LGunShootAir } else { ftCo_MS_LGunShootAirEmpty };
    let (f1, f2) = (f.cur_anim_frame, f.frame_speed_mul);
    e::Fighter_ChangeMotionState(g, id as i32, 0x0C4C5080, f1, f2, 0.0, core::ptr::null_mut());
    crate::ftcommon::clamp_air_drift(fp(g));
    fp(g).accessory4_cb = Some(ftco_800ce1d4);
}

/// `ftCo_800CE1D4`: accessory4 callback, fires the shot when throw flag b0 is set.
pub extern "C" fn ftco_800ce1d4(g: G) {
    let f = fp(g);
    if f.item_gobj.is_null() {
        return;
    }
    if !super::ft_0cd1::ft_check_throw_b0(f) {
        return;
    }
    let mut pos1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = pos1;
    ext::it_8028E774(f.item_gobj, &mut pos1);
    super::ft_0cdd::ftco_800cde94(fp(g), &pos1, &mut pos2);
    let f = fp(g);
    if !it_8026b594(f.item_gobj) {
        ext::it_8028E79C(f.item_gobj, &pos2, f.facing_dir);
        pos2.z = 0.0;
        let dir = fp(g).facing_dir;
        ext::efSync_Spawn_dir(1073, g, &pos2, &dir);
        let f = fp(g);
        if f.facing_dir == 1.0 {
            h::play_sfx(f, 228, 127, 64);
        } else {
            h::play_sfx(f, 229, 127, 64);
        }
    } else {
        ext::efSync_Spawn(1029, g, &pos2);
        h::play_sfx(fp(g), 230, 127, 64);
    }
}

pub fn ftco_lgunshoot_anim(g: G) { super::ft_0cdd::ftco_800cde18(g) }
pub fn ftco_lgunshootair_anim(g: G) { super::ft_0cdd::ftco_800cde18(g) }
pub fn ftco_lgunshoot_iasa(_g: G) {}
pub fn ftco_lgunshootair_iasa(_g: G) {}
pub fn ftco_lgunshoot_phys(g: G) { super::ft_0cdd::ftco_800cde54(g) }
pub fn ftco_lgunshootair_phys(g: G) { super::ft_0cdd::ftco_800cde74(g) }
pub fn ftco_lgunshoot_coll(g: G) { ext::ft_800841B8(g, ftco_800ce14c) }
pub fn ftco_lgunshootair_coll(g: G) { ext::ft_80082C74(g, ftco_800ce0cc) }
