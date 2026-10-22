//! src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c. Fused sites (tools/rust-port/fma_sites.py):
//! `my_lbVector_Len` (`x*x + y*y + z*z` = fmadds(z,z, fmadds(x,x, y*y))) and `my_sqrtf` (3 f64
//! fma, in `crate::pure::my_sqrtf`). Smoke-tested by tests/tu_ftmasterhandwalk.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/8/9): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandwalk::{
    ftMh_MS_350_80151EB4, ftMh_MS_353_80151DC4, ftMh_Walk2_Phys, ftMh_WalkShoot_Phys,
};
pub use crate::chars::r3::ftMasterHand::ftmasterhandwalk::{ftMh_MS_351_801520D8, ftMh_Walk2_Anim};
pub use crate::chars::r8::ftMasterHand::ftmasterhandwalk::{
    ftMh_Walk2_IASA, ftMh_WalkLoop_IASA, ftMh_WalkShoot_IASA,
};
pub use crate::chars::r9::ftMasterHand::ftmasterhandwalk::ftMh_WalkLoop_Phys;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
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
    hook!(FTMH_MS_389_80151018, ftMh_MS_389_80151018, (gobj: *mut HSD_GObj));
    hook!(FTMH_MS_349_80151CA8, ftMh_MS_349_80151CA8, (gobj: *mut HSD_GObj));
    hook!(FTBOSSLIB_8015C244, ftBossLib_8015C244, (gobj: *mut HSD_GObj, pos: *const Vec3) -> *mut HSD_GObj);
    hook!(FTLIB_800866DC, ftLib_800866DC, (gobj: *mut HSD_GObj, out: *mut Vec3));
    hook!(MPFLOORGETLEFT, mpFloorGetLeft, (line: i32, out: *mut Vec3));
}

pub fn ftMh_WalkShoot_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_WalkShoot_Coll(_gobj: *mut HSD_GObj) {}
pub fn ftMh_Walk2_Coll(_gobj: *mut HSD_GObj) {}
pub fn ftMh_WalkLoop_Coll(_gobj: *mut HSD_GObj) {}

/// `my_lbVector_Len` (static inline): `x*x + y*y + z*z` contracts to two fmadds.
fn my_lbVector_Len(v: &Vec3) -> f32 {
    let s = fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y));
    crate::pure::my_sqrtf(s)
}

pub fn ftMh_WalkLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        let target = ext::ftBossLib_8015C244(gobj, &(*fp).cur_pos);
        ext::ftLib_800866DC(target, &mut pos);
        let vel = Vec3 {
            x: pos.x - (*fp).cur_pos.x,
            y: pos.y - (*fp).cur_pos.y,
            z: pos.z - (*fp).cur_pos.z,
        };

        if my_lbVector_Len(&vel) < (*da).x4C {
            ext::ftMh_MS_349_80151CA8(gobj);
        }

        ext::mpFloorGetLeft(0, &mut pos);

        if (*fp).cur_pos.x < pos.x {
            ftMh_MS_351_801520D8(gobj);
        }

        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, ftMh_MS_WalkLoop as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
    }
}
