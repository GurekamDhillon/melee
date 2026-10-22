//! src/melee/ft/kinds/ftMasterHand/ftmasterhandtagrockpaper.c. Float math: `facing_dir_1 *= neg1`
//! (a plain multiply by -1.0f, no fused site). Smoke-tested by tests/tu_ftmasterhandtagrockpaper.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D1C;
use crate::tu::ftmasterhandthrownmasterhand::ftMh_CaptureWaitMasterHand_80155D6C;
use crate::tu::ftmasterhandwait12::{ftMh_MS_389_80151018, ftMh_UpdateBossMotion};
use melee_fighter::tu::ftbosslib as bl;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandtagrockpaper::{
    ftMh_MS_386_80155818, ftMh_TagFail_Phys, ftMh_TagGrab_Phys, ftMh_TagRockPaper_Phys,
    ftMh_TagSqueeze_Phys,
};
pub use crate::chars::r8::ftMasterHand::ftmasterhandtagrockpaper::{
    ftMh_TagCancel_IASA, ftMh_TagFail_IASA, ftMh_TagGrab_IASA, ftMh_TagRockPaper_IASA,
    ftMh_TagSqueeze_IASA,
};

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
    unsafe {
        if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_TagRockPaper_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_TagGrab_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        if bl::ftBossLib_8015C31C() || bl::ftBossLib_8015C3A0() || st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            st::Fighter_UnkSetFlag_8006CFBC(gobj);
            (*fp).x1A5C = core::ptr::null_mut();
            ftMh_MS_386_80155818(gobj);
        }
    }
}

pub fn ftMh_TagGrab_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_TagSqueeze_Anim(gobj: *mut HSD_GObj) {
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

pub fn ftMh_TagSqueeze_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_TagFail_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_TagFail_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftMh_TagCancel_Anim(gobj: *mut HSD_GObj) {
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
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        ftMh_UpdateBossMotion(gobj, fp, da);
    }
}

pub fn ftMh_TagCancel_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).mv.mh.unk0.x18 == 0.0 {
            (*fp).self_vel.z = 0.0;
            (*fp).self_vel.y = 0.0;
            (*fp).self_vel.x = 0.0;
        }
    }
}
