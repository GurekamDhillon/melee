//! src/melee/ft/kinds/ftMario/ftmariospecials.c: Cape (side B), also Dr. Mario's.
//! Fused sites (tools/rust-port/fma_sites.py): `ftMr_SpecialS_Phys` and `ftMr_SpecialAirS_Phys`
//! each have one f32 site, `coords.x += 3 * fp->facing_dir` -> `fmadds(3.0, facing_dir,
//! coords.x)`. Smoke-tested through recording hooks by tests/tu_ftmariospecials.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as fc;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (round 2 and the pure helper): re-exported under this TU.
pub use crate::chars::r2::ftMario::ftmariospecials::{ftMr_SpecialAirS_Anim, ftMr_SpecialS_Anim};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftcaptainspecialn::ext::{lb_800119DC, LB_800119DC};
    pub use crate::tu::ftcrazyhandfingerbeam0::ext::{lb_8000B1CC, LB_8000B1CC};
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
    pub static mut IT_802B2560: Option<fn(*mut HSD_GObj, f32, *mut Vec3, i32, ItemKind) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802B2560(gobj: *mut HSD_GObj, facing_dir: f32, pos: *mut Vec3, bone: i32, kind: ItemKind) -> *mut HSD_GObj {
                match unsafe { IT_802B2560 } {
                    Some(f) => f(gobj, facing_dir, pos, bone, kind),
                    None => { return unsafe { melee_items::tu::itmariocape::it_802B2560(gobj as _, facing_dir as _, pos as _, bone as _, kind as _) as _ }; },
                }
            }
    pub static mut IT_802B2674: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B2674(cape: *mut HSD_GObj)  {
                match unsafe { IT_802B2674 } {
                    Some(f) => f(cape),
                    None => { unsafe { melee_items::tu::itmariocape::it_802B2674(cape as _); } return; },
                }
            }
    pub static mut IT_802B26C0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B26C0(cape: *mut HSD_GObj)  {
                match unsafe { IT_802B26C0 } {
                    Some(f) => f(cape),
                    None => { unsafe { melee_items::tu::itmariocape::it_802B26C0(cape as _); } return; },
                }
            }
    pub static mut IT_802B26E0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802B26E0(cape: *mut HSD_GObj)  {
                match unsafe { IT_802B26E0 } {
                    Some(f) => f(cape),
                    None => { unsafe { melee_items::tu::itmariocape::it_802B26E0(cape as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit, (gobj: *mut HSD_GObj, desc: *mut ReflectDesc, arg: *mut core::ffi::c_void));

    /// `ftMr_Init_OnTakeDamage` (ftmario.c, not ported yet): installed as `death2_cb` /
    /// `take_dmg_cb` while the cape is out.
    pub unsafe extern "C" fn ftMr_Init_OnTakeDamage(_g: *mut HSD_GObj) {
        { unsafe { crate::tu::ftmario::ftMr_Init_OnTakeDamage(_g as _); } return; };
    }
}

/// Ft_MF_KeepColAnimHitStatus | SkipHit | SkipMatAnim | UpdateCmd | SkipColAnim | SkipItemVis |
/// Unk19 | SkipModelPartVis | SkipModelFlags | Unk27.
const transition_flags: u32 = (Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipColAnim
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27) as u32;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftMario_DatAttrs {
    (*fp).dat_attrs as *mut ftMario_DatAttrs
}
#[inline(always)]
unsafe fn joint_of(fp: *mut Fighter, part: Fighter_Part) -> *mut HSD_JObj {
    let idx = fh::ftparts_get_bone_index(&*fp, part as i32);
    (*(*fp).parts.offset(idx as isize)).joint
}

pub unsafe extern "C" fn cb_ftMr_SpecialS_CreateCape(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMr_SpecialS_CreateCape");
    ftMr_SpecialS_CreateCape(g);
}
pub unsafe extern "C" fn cb_ftMr_SpecialS_EnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMr_SpecialS_EnterHitlag");
    ftMr_SpecialS_EnterHitlag(g);
}
pub unsafe extern "C" fn cb_ftMr_SpecialS_ExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftMr_SpecialS_ExitHitlag");
    ftMr_SpecialS_ExitHitlag(g);
}

/// `setCallbacks` (static).
pub fn setCallbacks(fp: *mut Fighter) {
    melee_core::ct!("setCallbacks");
    unsafe {
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            (*fp).death2_cb = Some(ext::ftMr_Init_OnTakeDamage);
            (*fp).take_dmg_cb = Some(ext::ftMr_Init_OnTakeDamage);
        }
        (*fp).pre_hitlag_cb = Some(cb_ftMr_SpecialS_EnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_ftMr_SpecialS_ExitHitlag);
    }
}

/// Accessory-4 callback: spawn the cape item on the script's command.
pub fn ftMr_SpecialS_CreateCape(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_CreateCape");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if (*fp).cmd_vars[2] == 0 {
            let mut coords: Vec3 = core::mem::zeroed();
            (*fp).cmd_vars[2] = 1;
            ext::lb_8000B1CC(joint_of(fp, FtPart_RThumbNb), core::ptr::null(), &mut coords);
            let bone = fh::ftparts_get_bone_index(&*fp, FtPart_RThumbNb as i32);
            let cape = ext::it_802B2560(gobj, (*fp).facing_dir, &mut coords, bone, (*sa).specials.cape_kind);
            (*fp).u.mr.x223C_capeGObj = cape;
            (*fp).x1984_heldItemSpec = (*fp).u.mr.x223C_capeGObj as *mut Item_GObj;
            setCallbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

pub fn ftMr_SpecialS_Reset(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_Reset");
    unsafe {
        let fp = fpp(gobj);
        ftMr_SpecialS_ExitHitlag(gobj);
        (*fp).u.mr.x223C_capeGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftMr_SpecialS_RemoveCape(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_RemoveCape");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B2674((*fp).u.mr.x223C_capeGObj);
            ftMr_SpecialS_Reset(gobj);
        }
    }
}

pub fn ftMr_SpecialS_EnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_EnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B26C0((*fp).u.mr.x223C_capeGObj);
        }
    }
}

pub fn ftMr_SpecialS_ExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_ExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.mr.x223C_capeGObj.is_null() {
            ext::it_802B26E0((*fp).u.mr.x223C_capeGObj);
        }
    }
}

/// True unless the fighter is in SpecialS / SpecialAirS (the pure check is
/// `chars::ftMario::ftmariospecials`).
pub fn ftMr_SpecialS_CheckItemCapeRemove(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftMr_SpecialS_CheckItemCapeRemove");
    unsafe { crate::chars::ftMario::ftmariospecials::ftMr_SpecialS_CheckItemCapeRemove(&*fpp(gobj)) }
}

/// `changeAction` (static).
pub fn changeAction(gobj: *mut HSD_GObj, msid: i32) {
    melee_core::ct!("changeAction");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        let fp = fpp(gobj);
        (*fp).cmd_vars[2] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).mv.mr.SpecialS.reflecting = 0;
        (*fp).accessory4_cb = Some(cb_ftMr_SpecialS_CreateCape);
    }
}

pub fn ftMr_SpecialS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_Enter");
    unsafe {
        (*fpp(gobj)).self_vel.y = 0.0;
        changeAction(gobj, ftMr_MS_SpecialS as i32);
    }
}

pub fn ftMr_SpecialAirS_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirS_Enter");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        (*fp).self_vel.x /= (*sa).specials.vel_x_decay;
        changeAction(gobj, ftMr_MS_SpecialAirS as i32);
    }
}

pub fn ftMr_SpecialS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMr_SpecialS_IASA");}

pub fn ftMr_SpecialAirS_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMr_SpecialAirS_IASA");}

/// `reflect` (static): open and close the cape's reflect window from the script flag.
pub fn reflect(gobj: *mut HSD_GObj) {
    melee_core::ct!("reflect");
    unsafe {
        let fp = fpp(gobj);
        let sa = da(fp);
        if (*fp).cmd_vars[1] == 1 && (*fp).mv.mr.SpecialS.reflecting == 0 {
            (*fp).mv.mr.SpecialS.reflecting = 1;
            ext::ftColl_CreateReflectHit(gobj, &mut (*sa).cape_reflection, core::ptr::null_mut());
        } else if (*fp).cmd_vars[1] == 0 && (*fp).mv.mr.SpecialS.reflecting == 1 {
            (*fp).mv.mr.SpecialS.reflecting = 0;
            (*fp).set_reflecting(0);
        }
        st::ftColl_8007AEF8(gobj);
    }
}

pub fn ftMr_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_Phys");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] == 1 {
            (*fp).cmd_vars[0] = 2;
            let mut coords: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC(joint_of(fp, FtPart_HipN), core::ptr::null(), &mut coords);
            coords.x = fmadds(3.0, (*fp).facing_dir, coords.x);
            ext::lb_800119DC(&mut coords, 120, 0.9f64 as f32, 0.02f64 as f32, (core::f64::consts::PI / 3.0) as f32);
        }
        st::ft_80084F3C(gobj);
        reflect(gobj);
    }
}

pub fn ftMr_SpecialAirS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirS_Phys");
    unsafe {
        let fp = fpp(gobj);
        let v0 = (*fp).cmd_vars[0];
        let sa = da(fp);
        if v0 >= 1 {
            if v0 == 1 {
                (*fp).cmd_vars[0] = 2;
                if (*fp).u.mr.x2238_isCapeBoost == 0 {
                    (*fp).u.mr.x2238_isCapeBoost = 1;
                    (*fp).self_vel.y = (*sa).specials.vel.y;
                } else {
                    (*fp).self_vel.y = 0.0;
                }
                let mut coords: Vec3 = core::mem::zeroed();
                ext::lb_8000B1CC(joint_of(fp, FtPart_HipN), core::ptr::null(), &mut coords);
                coords.x = fmadds(3.0, (*fp).facing_dir, coords.x);
                ext::lb_800119DC(&mut coords, 120, 3.0, 0.1f64 as f32, (core::f64::consts::PI / 3.0) as f32);
            }
            fc::fall(&mut *fp, (*sa).specials.grav, (*sa).specials.terminal_vel);
        } else {
            fc::fall_basic(&mut *fp);
        }
        fc::calc_self_accel_deaccel(&mut *fp, (*sa).specials.vel.x);
        reflect(gobj);
    }
}

pub fn ftMr_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftMr_SpecialS_GroundToAir(gobj);
    }
}

pub fn ftMr_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirS_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        ftMr_SpecialAirS_AirToGround(gobj);
    }
}

/// `collUpdateVars` (static): carry the reflect flag over a ground/air transition (the state
/// change clears it) and re-arm the callbacks.
pub fn collUpdateVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("collUpdateVars");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.mr.SpecialS.reflecting != 0 {
            (*fp).set_reflecting(1);
        }
        setCallbacks(fp);
        (*fp).accessory4_cb = Some(cb_ftMr_SpecialS_CreateCape);
    }
}

pub fn ftMr_SpecialS_GroundToAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialS_GroundToAir");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftMr_MS_SpecialAirS as i32, transition_flags);
        if (*fp).cmd_vars[0] as i32 == 1 {
            (*fp).cmd_vars[0] = 2;
        }
        collUpdateVars(gobj);
    }
}

pub fn ftMr_SpecialAirS_AirToGround(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMr_SpecialAirS_AirToGround");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.mr.x2238_isCapeBoost = 0;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftMr_MS_SpecialS as i32, transition_flags);
        collUpdateVars(gobj);
    }
}
