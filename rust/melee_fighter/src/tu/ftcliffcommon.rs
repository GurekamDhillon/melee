//! src/melee/ft/ftcliffcommon.c: ledge grab (CliffCatch) entry and its state callbacks.
//! One fused site (`fma_sites.py`): ftCo_CliffCatch_Phys `x68C.z * facing + vec.x` -> `fmadds`.
//! Already-ported callees are called directly (ftcommon2, ft0881, plbonus, ftcamera); the rest
//! are `ext` hooks that panic while unset.
#![allow(non_snake_case, static_mut_refs)]

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        FT_80082E3C ft_80082E3C(g: G) -> G;
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: MotionFlags, anim_start: f32, rate: f32, blend: f32, obj: *mut HSD_GObj);
        FTANIM_8006EBA4 ftAnim_8006EBA4(g: G);
        MPLIB_80053ECC_FLOOR mpLib_80053ECC_Floor(line: i32, v: &mut Vec3);
        MPLIB_80053DA4_FLOOR mpLib_80053DA4_Floor(line: i32, v: &mut Vec3);
        MPLIB_80054ED8 mpLib_80054ED8(line: i32) -> bool;
        MPLIB_8005811C mpLib_8005811C(cd: *mut CollData, line: i32);
        EFASYNC_SPAWN efAsync_Spawn(g: G, slot: *mut core::ffi::c_void, kind: u32, id: u32, jobj: *mut HSD_JObj, v: *const Vec3);
        FTCO_8009A804 ftCo_8009A804(g: G);
        FT_800821DC ft_800821DC(g: G) -> bool;
        FT_80082B1C ft_80082B1C(g: G);
        FTCO_8009EF68 ftCo_8009EF68(g: G) -> bool;
    }
    /// Shared hooks in `crate::hooks`.
    pub fn ftAnim_IsFramesRemaining(g: G) -> bool {
        match unsafe { crate::hooks::FTANIM_IS_FRAMES_REMAINING } {
            Some(f) => f(g),
            None => unsafe { crate::tu::ftanim::ftAnim_IsFramesRemaining(g as _) as _ },
        }
    }
    pub fn ftCo_Fall_Enter(g: G) {
        crate::hooks::ftco_fall_enter(g)
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

const LEDGE_GRAB_MASK: i32 = Collide_LeftLedgeGrab | Collide_RightLedgeGrab;

/// `ftCliffCommon_80081298`: try to grab a ledge; true if CliffCatch was entered.
pub fn ftcliffcommon_80081298(g: G) -> bool {
    melee_core::ct!("ftcliffcommon_80081298");
    let f = fp(g);
    if f.input.lstick[0].y <= -crate::hooks::common_data().x480 {
        return false;
    }
    if f.coll_data.env_flags & LEDGE_GRAB_MASK != 0 && (f.is_sandbag() & 1) == 0 {
        let mut og = ext::ft_80082E3C(g);
        // 20XX TE hand-warmers: no edge-hogging, both can hang on
        if !og.is_null() && melee_core::codes::handwarmers_active() {
            og = core::ptr::null_mut();
        }
        if og.is_null() {
            // Slippi Ledge Grab Limit: count the grab
            if f.is_sub_fighter() == 0 {
                melee_core::codes::note_ledge_grab(f.player_idx as i32);
            }
            crate::plbonus::pl_80040048(f.player_idx as i32, f.is_sub_fighter() as i32);
            ftcliffcommon_80081370(g);
            return true;
        }
        let of = fp(og);
        let lid = unsafe { of.mv.co.cliff.ledge_id };
        crate::plbonus::pl_8003ffdc(of.player_idx as i32, of.is_sub_fighter() as i32, f.player_idx as i32, f.is_sub_fighter() as i32, lid);
        f.x213C = lid;
        return false;
    }
    false
}

/// `ftCliffCommon_80081370`: enter CliffCatch (motion 252).
pub fn ftcliffcommon_80081370(g: G) {
    melee_core::ct!("ftcliffcommon_80081370");
    use crate::ftcommon2 as c2;
    let f = fp(g);
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let ledge_dir: f32 = if f.coll_data.env_flags & Collide_LeftLedgeGrab != 0 { 1.0 } else { -1.0 };
    let facing = f.facing_dir;
    if facing != ledge_dir {
        f.facing_dir = -facing;
    }
    c2::ftcommon_8007d780(f);
    c2::ftcommon_8007d5d4(f);
    ext::Fighter_ChangeMotionState(g, 252, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    c2::ftcommon_8007d5d4(f);
    c2::ftcommon_8007efc0(f, crate::hooks::common_data().x5F0 as u32);
    c2::ftcommon_8007e2fc(f);
    f.set_x221D_b7(1);
    unsafe {
        f.mv.co.cliff.ledge_id = if f.facing_dir > 0.0 { f.coll_data.ledge_id_left } else { f.coll_data.ledge_id_right };
    }
    ftco_cliff_catch_phys(g);
    let f = fp(g);
    let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x28 };
    crate::ft0881::ft_800881d8(f, sfx, 127, 64);
    c2::ftcommon_8007e2f4(f, 511);
    c2::ftcommon_8007ebac(f, 12, 0);
    let lid = unsafe { f.mv.co.cliff.ledge_id };
    if f.facing_dir > 0.0 {
        ext::mpLib_80053ECC_Floor(lid, &mut vec);
    } else {
        ext::mpLib_80053DA4_Floor(lid, &mut vec);
    }
    let f = fp(g);
    ext::efAsync_Spawn(g, &mut f.x60C as *mut _ as *mut core::ffi::c_void, 2, 1052, core::ptr::null_mut(), &vec);
    crate::hooks::play_sfx(fp(g), 4, 127, 64);
}

/// `ftCo_CliffCatch_Anim`.
pub fn ftco_cliff_catch_anim(g: G) {
    melee_core::ct!("ftco_cliff_catch_anim");
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_8009A804(g);
    }
}

/// `ftCo_CliffCatch_IASA`: empty.
pub fn ftco_cliff_catch_iasa(_g: G) { melee_core::ct!("ftco_cliff_catch_iasa");}

/// `ftCo_CliffCatch_Phys`: snap to the ledge, or fall if it is gone.
pub fn ftco_cliff_catch_phys(g: G) {
    melee_core::ct!("ftco_cliff_catch_phys");
    let f = fp(g);
    let lid = unsafe { f.mv.co.cliff.ledge_id };
    if ext::mpLib_80054ED8(lid) {
        let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if f.facing_dir > 0.0 {
            ext::mpLib_80053ECC_Floor(lid, &mut vec);
        } else {
            ext::mpLib_80053DA4_Floor(lid, &mut vec);
        }
        let f = fp(g);
        f.cur_pos.x = fmadds(f.x68C_transNPos.z, f.facing_dir, vec.x);
        f.cur_pos.y = vec.y + f.x68C_transNPos.y;
    } else {
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ftCo_CliffCatch_Coll`.
pub fn ftco_cliff_catch_coll(g: G) {
    melee_core::ct!("ftco_cliff_catch_coll");
    if ext::ft_800821DC(g) {
        ext::ft_80082B1C(g);
        return;
    }
    if ext::ftCo_8009EF68(g) {
        fp(g).x2064_ledgeCooldown = crate::hooks::common_data().ledge_cooldown;
    }
}

/// `ftCo_Cliff_Cam`.
pub fn ftco_cliff_cam(g: G) {
    melee_core::ct!("ftco_cliff_cam");
    super::ftcamera::ftcamera_update_camera_box(g);
    let f = fp(g);
    if f.ground_or_air as i32 == GA_Air as i32 {
        let lid = unsafe { f.mv.co.cliff.ledge_id };
        ext::mpLib_8005811C(&mut f.coll_data, lid);
        unsafe { (*f.x890_cameraBox).set_on_ledge(1) };
    }
}
