//! src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c: Kirby's Toad (Peach copy ability).
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftkirbyspecialpeach.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{fpp, ftKb_SpecialN_set_cbs, G};
use melee_types::*;

/// Already ported (rounds 2, 4, 11, 13): re-exported under this TU.
pub use crate::chars::r11::ftKirby::ftkirbyspecialpeach::{
    ftKb_PeSpecialAirLwHit_Coll, ftKb_PeSpecialLwHit_Coll, ftKb_SpecialNGw_8010CC6C, ftKb_SpecialNGw_8010CCD4,
};
pub use crate::chars::r13::ftKirby::ftkirbyspecialpeach::ftKb_PeSpecialLw_Phys;
pub use crate::chars::r2::ftKirby::ftkirbyspecialpeach::ftKb_PeSpecialLwHit_Phys;
pub use crate::chars::r4::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNGw_8010CD44;

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
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:expr, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => ($port)($($a),*),
                }
            }
        };
    }
    type G = *mut HSD_GObj;
    pub use crate::tu::ftkirbyspecialn::ext::{lb_8000B1CC, LB_8000B1CC};
    hook!(IT_802BDE18, it_802BDE18, (g: G, pos: *const Vec3, part: i32, kind: i32, dir: f32) -> *mut Item_GObj);
    hook!(IT_802BE214, it_802BE214, (g: G, pos: *const Vec3, kind: i32, dir: f32));
    hook!(IT_802BDF40, it_802BDF40, (item: *mut Item_GObj));
    hook!(IT_802BDFA0, it_802BDFA0, (item: *mut Item_GObj));
    hook!(IT_802BDFC0, it_802BDFC0, (item: *mut Item_GObj));
    hook!(IT_802BE100, it_802BE100, (item: *mut Item_GObj));
    hook!(FTCOLL_8007B1B8, ftColl_8007B1B8, (g: G, desc: *mut ShieldDesc, cb: HSD_GObjEvent));
    hook_or!(FTCOMMON_FALLBASIC, ftCommon_FallBasic, |fp: *mut Fighter| melee_fighter::ftcommon::fall_basic(unsafe { &mut *fp }), (fp: *mut Fighter));
    hook_or!(FTCOMMON_CALCSELFACCEL_DEACCEL, ftCommon_CalcSelfAccel_Deaccel, |fp: *mut Fighter, f| melee_fighter::ftcommon::calc_self_accel_deaccel(unsafe { &mut *fp }, f), (fp: *mut Fighter, friction: f32));
}

/// `mf` of the Lw / LwHit ground<->air changes (`(1 << 2) | (1 << 3) | ...`, named-flag todo in the C).
const MF: u32 = (1 << 2) | (1 << 3) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

// Callback entries (extern "C") stored in Fighter fields.
pub unsafe extern "C" fn cb_fn_8010C288(g: G) {
    fn_8010C288(g)
}
pub unsafe extern "C" fn cb_fn_8010C344(g: G) {
    fn_8010C344(g)
}
pub unsafe extern "C" fn cb_fn_8010C44C(g: G) {
    fn_8010C44C(g)
}
pub unsafe extern "C" fn cb_ftKb_SpecialNPe_8010C47C(g: G) {
    ftKb_SpecialNPe_8010C47C(g)
}
pub unsafe extern "C" fn cb_fn_8010CD88(g: G) {
    fn_8010CD88(g)
}

/// `fn_8010C288`: accessory4 callback; spawns the Toad item once.
pub fn fn_8010C288(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[2] == 0 {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[2] = 1;
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint as *mut _, core::ptr::null(), &mut pos);
            (*fp).u.kb.xD0 = ext::it_802BDE18(gobj, &pos, FtPart_R3rdNa as i32, It_Kind_Kirby_PeachToad as i32, (*fp).facing_dir);
            if !(*fp).u.kb.xD0.is_null() {
                ftKb_SpecialN_set_cbs(gobj);
            }
            (*fp).accessory4_cb = None;
            (*fp).pre_hitlag_cb = Some(cb_fn_8010C44C);
            (*fp).post_hitlag_cb = Some(cb_ftKb_SpecialNPe_8010C47C);
        }
    }
}

/// `fn_8010C344`: accessory4 callback; spawns the Toad's spore.
pub fn fn_8010C344(arg0: G) {
    unsafe {
        let fp = (*arg0).user_data as *mut Fighter;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_R3rdNa as usize)).joint as *mut _, core::ptr::null(), &mut pos);
        pos.y += 2.5;
        pos.z = 0.0;
        ext::it_802BE214(arg0, &pos, It_Kind_Kirby_PeachToadSpore as i32, (*fp).facing_dir);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftKb_SpecialNPe_8010C3C0(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        ftKb_SpecialNPe_8010C47C(gobj);
        (*fp).u.kb.xD0 = core::ptr::null_mut();
    }
}

pub fn ftKb_SpecialNPe_8010C3F4(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDF40((*fp).u.kb.xD0);
            ftKb_SpecialNPe_8010C3C0(gobj);
        }
    }
}

pub fn fn_8010C44C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDFA0((*fp).u.kb.xD0);
        }
    }
}

pub fn ftKb_SpecialNPe_8010C47C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDFC0((*fp).u.kb.xD0);
        }
    }
}

/// True unless Kirby is in one of the Toad states (the root module's Fighter-only half).
pub fn ftKb_SpecialNPe_8010C4AC(gobj: G) -> bool {
    unsafe { crate::chars::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C4AC(&*fpp(gobj)) }
}

/// `inlineA0` (static).
pub fn inlineA0(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).mv.kb.specialn_pe.facing_dir = (*fp).facing_dir as i32;
        (*fp).accessory4_cb = Some(cb_fn_8010C288);
    }
}

/// `inlineA1` (static).
pub fn inlineA1(gobj: G, msid: i32) {
    unsafe {
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        inlineA0(gobj);
    }
}

pub fn ftKb_SpecialNPe_8010C4D4(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        inlineA1(gobj, ftKb_MS_PeSpecialLw as i32);
    }
}

pub fn ftKb_SpecialNPe_8010C560(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        (*fp).self_vel.x /= da.specialn_pe_friction;
        inlineA1(gobj, ftKb_MS_PeSpecialAirLw as i32);
    }
}

/// `doPeAnim` (static inline): arm the absorb shield on cmd 1, then `cb` when the anim ends.
pub fn doPeAnim(gobj: G, cb: fn(G)) {
    unsafe {
        let fp = fpp(gobj);
        let cmd_var1 = (*fp).cmd_vars[1];
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        if cmd_var1 == 1 {
            (*fp).cmd_vars[1] = 2;
            ext::ftColl_8007B1B8(gobj, &da.specialn_pe_absorbdesc as *const AbsorbDesc as *mut ShieldDesc, Some(cb_fn_8010CD88));
            (*fp).anon_221b.set_x221B_b3(1);
            (*fp).shield_unk0 = da.specialn_pe_unk3;
            (*fp).shield_unk1 = da.specialn_pe_unk3;
        } else if cmd_var1 == 0 {
            (*fp).anon_221b.set_x221B_b0(0);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftKb_PeSpecialLw_Anim(gobj: G) {
    doPeAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKb_PeSpecialAirLw_Anim(gobj: G) {
    doPeAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftKb_PeSpecialLw_IASA(_gobj: G) {}
pub fn ftKb_PeSpecialAirLw_IASA(_gobj: G) {}

pub fn ftKb_PeSpecialAirLw_Phys(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let cmd_var0 = (*fp).cmd_vars[0];
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        if cmd_var0 >= 1 {
            if cmd_var0 == 1 {
                (*fp).cmd_vars[0] = 2;
                if (*fp).u.kb.xCC == 0 {
                    (*fp).u.kb.xCC = 1;
                    (*fp).self_vel.y = da.specialn_pe_air_initial_vertical_momentum;
                } else {
                    (*fp).self_vel.y = 0.0;
                }
            }
            st::ftCommon_Fall(&mut *fp, da.specialn_pe_fall_acceleration, da.specialn_pe_unk2);
        } else {
            ext::ftCommon_FallBasic(fp);
        }
        ext::ftCommon_CalcSelfAccel_Deaccel(fp, da.specialn_pe_air_horizontal_momentum_preservation);
        st::ftColl_8007AEE0(gobj);
    }
}

pub fn ftKb_PeSpecialLw_Coll(gobj: G) {
    if st::ft_800827A0(gobj) == 0 {
        ftKb_SpecialNPe_8010C8D8(gobj);
    }
}

pub fn ftKb_PeSpecialAirLw_Coll(gobj: G) {
    if st::ft_80081D0C(gobj) != 0 {
        ftKb_SpecialNPe_8010C9CC(gobj);
    }
}

/// Re-arm the Toad callbacks and the absorb shield after a ground/air change (shared tail).
unsafe fn rearm(gobj: G) {
    let fp = fpp(gobj);
    let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
    if !(*fp).u.kb.xD0.is_null() {
        ftKb_SpecialN_set_cbs(gobj);
    }
    (*fp).accessory4_cb = Some(cb_fn_8010C288);
    (*fp).pre_hitlag_cb = Some(cb_fn_8010C44C);
    (*fp).post_hitlag_cb = Some(cb_ftKb_SpecialNPe_8010C47C);
    if (*fp).cmd_vars[1] == 2 {
        ext::ftColl_8007B1B8(gobj, &da.specialn_pe_absorbdesc as *const AbsorbDesc as *mut ShieldDesc, Some(cb_fn_8010CD88));
        (*fp).anon_221b.set_x221B_b3(1);
    }
}

pub fn ftKb_SpecialNPe_8010C8D8(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_PeSpecialAirLw as i32, MF);
        if (*fp).cmd_vars[0] == 1 {
            (*fp).cmd_vars[0] = 2;
        }
        rearm(gobj);
    }
}

pub fn ftKb_SpecialNPe_8010C9CC(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.kb.xCC = 0;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_PeSpecialLw as i32, MF);
        rearm(gobj);
    }
}

/// `doPeHitAnim` (static inline).
pub fn doPeHitAnim(gobj: G, cb: fn(G)) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[3] != 0 {
            (*fp).cmd_vars[3] = 0;
            (*fp).accessory4_cb = Some(cb_fn_8010C344);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftKb_PeSpecialLwHit_Anim(gobj: G) {
    doPeHitAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKb_PeSpecialAirLwHit_Anim(gobj: G) {
    doPeHitAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftKb_PeSpecialLwHit_IASA(_gobj: G) {}
pub fn ftKb_PeSpecialAirLwHit_IASA(_gobj: G) {}

/// The root module's Fighter-only half, as the C-signature entry.
pub fn ftKb_PeSpecialAirLwHit_Phys(gobj: G) {
    unsafe { crate::chars::ftKirby::ftkirbyspecialpeach::ftKb_PeSpecialAirLwHit_Phys(&mut *fpp(gobj)) }
}

/// `fn_8010CD88`: the absorb shield's hit callback; goes to the Hit state.
pub fn fn_8010CD88(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.kb.specialn_pe.facing_dir = (*fp).specialn_facing_dir as i32;
        let msid = if (*fp).ground_or_air == GA_Ground { ftKb_MS_PeSpecialLwHit } else { ftKb_MS_PeSpecialAirLwHit };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 9.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BE100((*fp).u.kb.xD0);
        }
        ftKb_SpecialNGw_8010CD44(gobj);
    }
}
