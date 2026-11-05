//! src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c. Fused sites (tools/rust-port/fma_sites.py):
//! `my_lbVector_Len` (`x*x + y*y + z*z` = fmadds(z,z, fmadds(x,x, y*y))) and `my_sqrtf` (3 f64
//! fma, in `crate::pure::my_sqrtf`). Smoke-tested by tests/tu_ftmasterhandwalk.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/8/9): re-exported under this TU.





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
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut FTMH_MS_389_80151018: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_389_80151018(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_389_80151018 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandwait12::ftMh_MS_389_80151018(gobj as _); } return; },
                }
            }
    pub static mut FTMH_MS_349_80151CA8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftMh_MS_349_80151CA8(gobj: *mut HSD_GObj)  {
                match unsafe { FTMH_MS_349_80151CA8 } {
                    Some(f) => f(gobj),
                    None => { unsafe { crate::tu::ftmasterhandslap::ftMh_MS_349_80151CA8(gobj as _); } return; },
                }
            }
    pub static mut FTBOSSLIB_8015C244: Option<fn(*mut HSD_GObj, *const Vec3) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn ftBossLib_8015C244(gobj: *mut HSD_GObj, pos: *const Vec3) -> *mut HSD_GObj {
        match unsafe { FTBOSSLIB_8015C244 } {
            Some(f) => f(gobj, pos),
            None => { unsafe { melee_fighter::tu::ftbosslib::ftBossLib_8015C244(gobj as _, &*(pos as *const _)) as _ } },
        }
    }
    pub static mut FTLIB_800866DC: Option<fn(*mut HSD_GObj, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn ftLib_800866DC(gobj: *mut HSD_GObj, out: *mut Vec3) {
        match unsafe { FTLIB_800866DC } {
            Some(f) => f(gobj, out),
            None => { unsafe { let _ = melee_fighter::ftlib::ftlib_800866dc(gobj as _, &mut *(out as *mut _)); } },
        }
    }
    pub static mut MPFLOORGETLEFT: Option<fn(i32, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn mpFloorGetLeft(line: i32, out: *mut Vec3) {
        match unsafe { MPFLOORGETLEFT } {
            Some(f) => f(line, out),
            None => { unsafe { melee_world::tu::mplib::mpFloorGetLeft(line as _, out as _); } },
        }
    }
}

pub fn ftMh_WalkShoot_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkShoot_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ext::ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_WalkShoot_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_WalkShoot_Coll");}
pub fn ftMh_Walk2_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Walk2_Coll");}
pub fn ftMh_WalkLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_WalkLoop_Coll");}

/// `my_lbVector_Len` (static inline): `x*x + y*y + z*z` contracts to two fmadds.
fn my_lbVector_Len(v: &Vec3) -> f32 {
    let s = v.x * v.x + v.y * v.y + v.z * v.z; // through a pointer: not fused on the console
    crate::pure::my_sqrtf(s)
}

pub fn ftMh_WalkLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkLoop_Anim");
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

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_WalkShoot_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_WalkShoot_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkShoot_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_353_80151DC4` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_MS_353_80151DC4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_353_80151DC4");
    st::Fighter_ChangeMotionState(gobj, 350 /* ftMh_MS_Walk2 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_Walk2_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_Walk2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Walk2_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_350_80151EB4` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_MS_350_80151EB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_350_80151EB4");
    st::Fighter_ChangeMotionState(gobj, 351 /* ftMh_MS_WalkLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_Walk2_Anim` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_Walk2_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Walk2_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r2::ftMasterHand::ftmasterhandwalk::ftMh_MS_350_80151EB4(gobj);
        }
    }
}

/// `ftMh_MS_351_801520D8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_MS_351_801520D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_351_801520D8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 352 /* ftMh_MS_WalkWait */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).self_vel.x = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftMh_WalkShoot_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_WalkShoot_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkShoot_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_Walk2_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_Walk2_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_Walk2_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_WalkLoop_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_WalkLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_WalkLoop_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c).
pub fn ftMh_WalkLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_WalkLoop_Phys");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        (*fp).self_vel.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x40_pos.z;
    }
}
