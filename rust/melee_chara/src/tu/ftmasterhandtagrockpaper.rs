//! src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c. Float math: `facing_dir_1 *= neg1`
//! (a plain multiply by -1.0f, no fused site). Smoke-tested by tests/tu_ftmasterhandtagrockpaper.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D1C;
use crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D6C;
use crate::tu::ftmasterhandwait12::{ftMh_MS_389_80151018, ftMh_UpdateBossMotion};
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.



/// Calls into TUs outside this file. Each is a settable hook; unset it falls back to the
/// native port in `melee_fighter`.
pub mod ext {
    use melee_types::*;
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook_or!(FTCO_800DE2A8, ftCo_800DE2A8, melee_fighter::tu::ftCo_Throw::ftCo_800DE2A8, (gobj: *mut HSD_GObj, victim: *mut HSD_GObj));
    hook_or!(FTCO_800DE7C0, ftCo_800DE7C0, melee_fighter::tu::ftCo_Thrown::ftCo_800DE7C0, (gobj: *mut HSD_GObj, victim: *mut HSD_GObj, up: bool));
}

/// `const float neg1 = -1;`
pub const neg1: f32 = -1.0;

pub fn ftMh_TagRockPaper_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagRockPaper_Anim");
    unsafe {
        if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_TagRockPaper_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_TagRockPaper_Coll");}

pub fn ftMh_TagGrab_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagGrab_Anim");
    unsafe {
        if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            ftMh_MS_386_80155818(gobj);
        }
    }
}

pub fn ftMh_TagGrab_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_TagGrab_Coll");}

pub fn ftMh_TagSqueeze_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagSqueeze_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;

        if (*fp).cmd_vars[0] != 0 {
            (*fp).cmd_vars[0] = 0;
            ftMh_CaptureWaitMasterHand_80155D6C((*fp).victim_gobj, ftCo_MS_ThrownMasterHand as i32);
            if !(*fp).victim_gobj.is_null() {
                let victim = (*fp).victim_gobj;
                let victim_fp = (*victim).user_data as *mut Fighter;
                st::ftCommon_8007E2F4(&mut *fp, 0);
                ext::ftCo_800DE2A8(gobj, victim);
                (*victim_fp).dmg.facing_dir_1 *= neg1;
                ext::ftCo_800DE7C0(victim, core::ptr::null_mut(), false);
            }
        }

        if bl::ftBossLib_8015C31C() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let ft_3 = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*ft_3).x1A5C = core::ptr::null_mut();
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_TagSqueeze_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_TagSqueeze_Coll");}

pub fn ftMh_TagFail_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagFail_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_TagFail_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_TagFail_Coll");}

pub fn ftMh_TagCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCancel_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.mh.unk0.x24 -= 1.0;
        if (*fp).mv.mh.unk0.x24 <= 0.0 && (*fp).cmd_vars[0] != 0 {
            ftMh_CaptureWaitMasterHand_80155D1C((*fp).victim_gobj);
            (*fp).cmd_vars[0] = 0;
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_TagCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCancel_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_TagCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCancel_Coll");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.mh.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_TagRockPaper_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagRockPaper_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagRockPaper_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_TagGrab_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagGrab_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagGrab_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_TagSqueeze_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagSqueeze_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagSqueeze_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_386_80155818` (static) (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_MS_386_80155818(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_386_80155818");
    st::Fighter_ChangeMotionState(gobj, 387 /* ftMh_MS_TagFail */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
}

/// `ftMh_TagFail_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagFail_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagFail_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_TagRockPaper_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagRockPaper_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagRockPaper_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_TagGrab_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagGrab_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagGrab_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_TagSqueeze_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagSqueeze_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagSqueeze_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_TagFail_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagFail_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagFail_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_TagCancel_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c).
pub fn ftMh_TagCancel_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_TagCancel_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftMh_TagCancel_Coll_fp(fp: &mut Fighter) {
    melee_core::ct!("ftMh_TagCancel_Coll_fp");
    crate::chars::ftMasterHand::stop_if_x18_zero(fp)
}
