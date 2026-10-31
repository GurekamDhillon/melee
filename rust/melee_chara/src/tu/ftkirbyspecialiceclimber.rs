//! src/melee/ft/kinds/ftKirby/ftkirbyspecialiceclimber.c: Kirby's Ice Climbers copy ability
//! (Ice Shot). One fused site (fn_801090D4: `pos.x += x_spawn * facing_dir` is an fmadds, found by
//! tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftkirbyspecialiceclimber.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/11): re-exported under this TU.
pub use crate::chars::r2::ftKirby::ftkirbyspecialiceclimber::{
    ftKb_PpSpecialAirN_Anim, ftKb_PpSpecialAirN_Phys, ftKb_PpSpecialN_Anim, ftKb_PpSpecialN_Phys,
};

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
    /// Like `hook!`, but an unset hook falls back to an existing native port.
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
    crate::port_hook!(IT_802C17DC, it_802C17DC, crate::adapt::it_802C17DC, (item: *mut HSD_GObj));
    crate::port_hook!(IT_802C16F8, it_802C16F8, crate::adapt::it_802C16F8, (item: *mut HSD_GObj));
    crate::port_hook!(IT_802C1590, it_802C1590, crate::adapt::it_802C1590, (gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32) -> *mut HSD_GObj);
    crate::port_hook!(FT_PLAYSFX, ft_PlaySFX, crate::adapt::ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    hook_or!(FT_800881D8, ft_800881D8, ft_800881d8_port, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    fn ft_800881d8_port(fp: *mut Fighter, sfx: i32, vol: u8, pan: u8) {
        melee_fighter::ft0881::ft_800881d8(unsafe { &mut *fp }, sfx, vol, pan)
    }
    crate::port_hook!(FTCOMMON_SETACCESSORY, ftCommon_SetAccessory, crate::adapt::ftCommon_SetAccessory, (fp: *mut Fighter, joint: *mut HSD_Joint));
    pub static mut LB_8000C2F8: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C2F8(jobj: *mut HSD_JObj, parent: *mut HSD_JObj)  {
                match unsafe { LB_8000C2F8 } {
                    Some(f) => f(jobj, parent),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000C2F8(jobj as _, parent as _); } return; },
                }
            }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    // `ft_80459B88.hats[kind]`, or NULL when `hats` is NULL.
    crate::port_hook!(FT_80459B88_HAT, ft_80459B88_hat, crate::adapt::ft_80459B88_hat, (kind: i32) -> *mut KirbyHatStruct);
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftKb_Init_800EE74C` / `ftKb_Init_800EE7B8` stored as death2/take-damage callbacks.
#[inline(always)]
unsafe fn set_cbs(fp: *mut Fighter) {
    (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
}

/// Accessory callback entry for `fn_801090D4`.
pub unsafe extern "C" fn cb_fn_801090D4(g: G) {
    melee_core::ct!("cb_fn_801090D4");
    fn_801090D4(g);
}

pub fn ftKb_SpecialNIc_80108CE8(gobj: G, it_gobj: G) {
    melee_core::ct!("ftKb_SpecialNIc_80108CE8");
    unsafe {
        let ft = fpp(gobj);
        if it_gobj == (*ft).u.kb.xC0 {
            (*ft).u.kb.xC0 = core::ptr::null_mut();
        }
    }
}

/// Callback entry for `ftKb_SpecialNIc_80108D04`.
pub unsafe extern "C" fn cb_ftKb_SpecialNIc_80108D04(g: G) {
    melee_core::ct!("cb_ftKb_SpecialNIc_80108D04");
    ftKb_SpecialNIc_80108D04(g);
}

pub fn ftKb_SpecialNIc_80108D04(gobj: G) {
    melee_core::ct!("ftKb_SpecialNIc_80108D04");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xC0.is_null() {
            ext::it_802C17DC((*fp).u.kb.xC0);
            let fp2 = fpp(gobj);
            if (*fp).u.kb.xC0 == (*fp2).u.kb.xC0 {
                (*fp2).u.kb.xC0 = core::ptr::null_mut();
            }
        }
    }
}

/// Attach the Peach-hat accessory (the block shared by both Enter functions); does nothing when
/// Kirby's hat table has no Peach hat (the C `return`s without attaching).
unsafe fn attach_hat_accessory(gobj: G) {
    let fp2 = fpp(gobj);
    let hat = ext::ft_80459B88_hat(Ft_Kind_Peach as i32);
    if hat.is_null() {
        return;
    }
    ext::ftCommon_SetAccessory(fp2, (*hat).hat_dynamics[1] as *mut HSD_Joint);
    let bone = melee_fighter::hooks::ftparts_get_bone_index(&*fp2, FtPart_LThumbNb as i32);
    ext::lb_8000C2F8((*fp2).x20A0_accessory, (*(*fp2).parts.add(bone as usize)).joint);
}

pub fn ftKb_SpecialNIc_80108D64(gobj: G) {
    melee_core::ct!("ftKb_SpecialNIc_80108D64");
    unsafe {
        let fp = fpp(gobj);
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).u.kb.xC0 = core::ptr::null_mut();
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_PpSpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_fn_801090D4);
        attach_hat_accessory(gobj);
    }
}

pub fn ftKb_SpecialNIc_80108E14(gobj: G) {
    melee_core::ct!("ftKb_SpecialNIc_80108E14");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        (*fp).anon_2210.throw_flags = 0;
        (*fp).cmd_vars[0] = 0;
        (*fp).u.kb.xC0 = core::ptr::null_mut();
        if (*fp).u.kb.xC4 == 0 {
            (*fp).self_vel.y = (*da).specialn_pp_air_vertical_momentum;
            (*fp).u.kb.xC4 = 1;
            (*fp).u.kb.xC8 = 0.0;
        } else {
            (*fp).u.kb.xC8 = -10.0;
        }
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_PpSpecialAirN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(cb_fn_801090D4);
        attach_hat_accessory(gobj);
    }
}

pub fn ftKb_PpSpecialN_IASA(_gobj: G) { melee_core::ct!("ftKb_PpSpecialN_IASA");}
pub fn ftKb_PpSpecialAirN_IASA(_gobj: G) { melee_core::ct!("ftKb_PpSpecialAirN_IASA");}

pub fn ftKb_PpSpecialN_Coll(gobj: G) {
    melee_core::ct!("ftKb_PpSpecialN_Coll");
    unsafe {
        if st::ft_80082708(gobj) == 0 {
            let fp = fpp(gobj);
            if !(*fp).u.kb.xC0.is_null() {
                ext::it_802C17DC((*fp).u.kb.xC0);
                let fp2 = fpp(gobj);
                if (*fp).u.kb.xC0 == (*fp2).u.kb.xC0 {
                    (*fp2).u.kb.xC0 = core::ptr::null_mut();
                }
            }
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_PpSpecialAirN_Coll(gobj: G) {
    melee_core::ct!("ftKb_PpSpecialAirN_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            let fp1 = fpp(gobj);
            if !(*fp1).u.kb.xC0.is_null() {
                ext::it_802C17DC((*fp1).u.kb.xC0);
                let fp2 = fpp(gobj);
                if (*fp1).u.kb.xC0 == (*fp2).u.kb.xC0 {
                    (*fp2).u.kb.xC0 = core::ptr::null_mut();
                }
            }
            (*fp).u.kb.xC4 = 0;
            (*fp).u.kb.xC8 = 0.0;
            st::ftCo_LandingFallSpecial_Enter(gobj, 0, (*da).specialn_pp_landing_lag);
        }
    }
}

pub fn fn_801090D4(gobj: G) {
    melee_core::ct!("fn_801090D4");
    unsafe {
        let fp = fpp(gobj);
        let cmd = (*fp).cmd_vars[0];
        if cmd == 0 {
            return;
        }
        if cmd == 1 {
            let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
            let mut pos: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts).joint, core::ptr::null(), &mut pos);
            // `pos.x += x_spawn * facing_dir`: fmuladd in the shipping build.
            pos.x = fmadds((*da).specialn_pp_x_spawn, (*fp).facing_dir, pos.x);
            pos.y += (*da).specialn_pp_y_spawn + (*fp).u.kb.xC8;
            (*fp).u.kb.xC0 = ext::it_802C1590(gobj, &mut pos, It_Kind_Kirby_IceClimberIce as i32, (*fp).facing_dir);
            ext::ft_PlaySFX(fp, 0x1FBE5, 127, 64);
            if !(*fp).u.kb.xC0.is_null() {
                set_cbs(fp);
            }
            (*fp).cmd_vars[0] = 0;
            return;
        }
        if cmd == 2 {
            if !(*fp).u.kb.xC0.is_null() {
                ext::it_802C16F8((*fp).u.kb.xC0);
                (*fp).cmd_vars[0] = 0;
                ext::ft_800881D8(fp, 0x2233B, 127, 64);
                ext::ft_PlaySFX(fp, 0x1FBE8, 127, 64);
                let fp2 = fpp(gobj);
                if (*fp).u.kb.xC0 == (*fp2).u.kb.xC0 {
                    (*fp2).u.kb.xC0 = core::ptr::null_mut();
                }
            }
        }
    }
}

pub fn ftKb_SpecialNYs_80109238(gobj: G) -> Fighter_Part {
    melee_core::ct!("ftKb_SpecialNYs_80109238");
    unsafe { melee_fighter::hooks::ftparts_get_bone_index(&*fpp(gobj), FtPart_TransN2 as i32) as Fighter_Part }
}
