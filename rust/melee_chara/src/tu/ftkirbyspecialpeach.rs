//! src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c: Kirby's Toad (Peach copy ability).
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftkirbyspecialpeach.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftKb_DatAttrs, ftKb_MS_PeSpecialAirLwHit, ftKb_MS_PeSpecialLw};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftkirbyspecialn::{fpp, ftKb_SpecialN_set_cbs, G};
use melee_types::*;

/// Already ported (rounds 2, 4, 11, 13): re-exported under this TU.





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
    pub static mut IT_802BDE18: Option<fn(G, *const Vec3, i32, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802BDE18(g: G, pos: *const Vec3, part: i32, kind: i32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802BDE18 } {
                    Some(f) => f(g, pos, part, kind, dir),
                    None => { return unsafe { melee_items::tu::itpeachtoad::it_802BDE18(g as _, pos as _, part as _, kind as _, dir as _) as _ }; },
                }
            }
    pub static mut IT_802BE214: Option<fn(G, *const Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE214(g: G, pos: *const Vec3, kind: i32, dir: f32)  {
                match unsafe { IT_802BE214 } {
                    Some(f) => f(g, pos, kind, dir),
                    None => { unsafe { melee_items::tu::itpeachtoadspore::it_802BE214(g as _, pos as _, kind as _, dir as _); } return; },
                }
            }
    pub static mut IT_802BDF40: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDF40(item: *mut Item_GObj)  {
                match unsafe { IT_802BDF40 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDF40(item as _); } return; },
                }
            }
    pub static mut IT_802BDFA0: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDFA0(item: *mut Item_GObj)  {
                match unsafe { IT_802BDFA0 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDFA0(item as _); } return; },
                }
            }
    pub static mut IT_802BDFC0: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDFC0(item: *mut Item_GObj)  {
                match unsafe { IT_802BDFC0 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDFC0(item as _); } return; },
                }
            }
    pub static mut IT_802BE100: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE100(item: *mut Item_GObj)  {
                match unsafe { IT_802BE100 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BE100(item as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_8007B1B8, ftColl_8007B1B8, crate::adapt::ftColl_8007B1B8, (g: G, desc: *mut ShieldDesc, cb: HSD_GObjEvent));
    hook_or!(FTCOMMON_FALLBASIC, ftCommon_FallBasic, |fp: *mut Fighter| melee_fighter::ftcommon::fall_basic(unsafe { &mut *fp }), (fp: *mut Fighter));
    hook_or!(FTCOMMON_CALCSELFACCEL_DEACCEL, ftCommon_CalcSelfAccel_Deaccel, |fp: *mut Fighter, f| melee_fighter::ftcommon::calc_self_accel_deaccel(unsafe { &mut *fp }, f), (fp: *mut Fighter, friction: f32));
}

/// `mf` of the Lw / LwHit ground<->air changes (`(1 << 2) | (1 << 3) | ...`, named-flag todo in the C).
const MF: u32 = (1 << 2) | (1 << 3) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

// Callback entries (extern "C") stored in Fighter fields.
pub unsafe extern "C" fn cb_fn_8010C288(g: G) {
    melee_core::ct!("cb_fn_8010C288");
    fn_8010C288(g)
}
pub unsafe extern "C" fn cb_fn_8010C344(g: G) {
    melee_core::ct!("cb_fn_8010C344");
    fn_8010C344(g)
}
pub unsafe extern "C" fn cb_fn_8010C44C(g: G) {
    melee_core::ct!("cb_fn_8010C44C");
    fn_8010C44C(g)
}
pub unsafe extern "C" fn cb_ftKb_SpecialNPe_8010C47C(g: G) {
    melee_core::ct!("cb_ftKb_SpecialNPe_8010C47C");
    ftKb_SpecialNPe_8010C47C(g)
}
pub unsafe extern "C" fn cb_fn_8010CD88(g: G) {
    melee_core::ct!("cb_fn_8010CD88");
    fn_8010CD88(g)
}

/// `fn_8010C288`: accessory4 callback; spawns the Toad item once.
pub fn fn_8010C288(gobj: G) {
    melee_core::ct!("fn_8010C288");
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
    melee_core::ct!("fn_8010C344");
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
    melee_core::ct!("ftKb_SpecialNPe_8010C3C0");
    unsafe {
        let fp = fpp(gobj);
        ftKb_SpecialNPe_8010C47C(gobj);
        (*fp).u.kb.xD0 = core::ptr::null_mut();
    }
}

pub fn ftKb_SpecialNPe_8010C3F4(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C3F4");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDF40((*fp).u.kb.xD0);
            ftKb_SpecialNPe_8010C3C0(gobj);
        }
    }
}

pub fn fn_8010C44C(gobj: G) {
    melee_core::ct!("fn_8010C44C");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDFA0((*fp).u.kb.xD0);
        }
    }
}

pub fn ftKb_SpecialNPe_8010C47C(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C47C");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BDFC0((*fp).u.kb.xD0);
        }
    }
}

/// True unless Kirby is in one of the Toad states (the root module's Fighter-only half).
pub fn ftKb_SpecialNPe_8010C4AC(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialNPe_8010C4AC");
    unsafe { crate::chars::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C4AC(&*fpp(gobj)) }
}

/// `inlineA0` (static).
pub fn inlineA0(gobj: G) {
    melee_core::ct!("inlineA0");
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).mv.kb.specialn_pe.facing_dir = melee_core::ppc::fctiwz_f32((*fp).facing_dir);
        (*fp).accessory4_cb = Some(cb_fn_8010C288);
    }
}

/// `inlineA1` (static).
pub fn inlineA1(gobj: G, msid: i32) {
    melee_core::ct!("inlineA1");
    unsafe {
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        inlineA0(gobj);
    }
}

pub fn ftKb_SpecialNPe_8010C4D4(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C4D4");
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        inlineA1(gobj, ftKb_MS_PeSpecialLw as i32);
    }
}

pub fn ftKb_SpecialNPe_8010C560(gobj: G) {
    melee_core::ct!("ftKb_SpecialNPe_8010C560");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftKb_DatAttrs);
        (*fp).self_vel.x /= da.specialn_pe_friction;
        inlineA1(gobj, ftKb_MS_PeSpecialAirLw as i32);
    }
}

/// `doPeAnim` (static inline): arm the absorb shield on cmd 1, then `cb` when the anim ends.
pub fn doPeAnim(gobj: G, cb: fn(G)) {
    melee_core::ct!("doPeAnim");
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
    melee_core::ct!("ftKb_PeSpecialLw_Anim");
    doPeAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKb_PeSpecialAirLw_Anim(gobj: G) {
    melee_core::ct!("ftKb_PeSpecialAirLw_Anim");
    doPeAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftKb_PeSpecialLw_IASA(_gobj: G) { melee_core::ct!("ftKb_PeSpecialLw_IASA");}
pub fn ftKb_PeSpecialAirLw_IASA(_gobj: G) { melee_core::ct!("ftKb_PeSpecialAirLw_IASA");}

pub fn ftKb_PeSpecialAirLw_Phys(gobj: G) {
    melee_core::ct!("ftKb_PeSpecialAirLw_Phys");
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
    melee_core::ct!("ftKb_PeSpecialLw_Coll");
    if st::ft_800827A0(gobj) == 0 {
        ftKb_SpecialNPe_8010C8D8(gobj);
    }
}

pub fn ftKb_PeSpecialAirLw_Coll(gobj: G) {
    melee_core::ct!("ftKb_PeSpecialAirLw_Coll");
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
    melee_core::ct!("ftKb_SpecialNPe_8010C8D8");
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
    melee_core::ct!("ftKb_SpecialNPe_8010C9CC");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.kb.xCC = 0;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_PeSpecialLw as i32, MF);
        rearm(gobj);
    }
}

/// `doPeHitAnim` (static inline).
pub fn doPeHitAnim(gobj: G, cb: fn(G)) {
    melee_core::ct!("doPeHitAnim");
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
    melee_core::ct!("ftKb_PeSpecialLwHit_Anim");
    doPeHitAnim(gobj, st::ft_8008A2BC);
}

pub fn ftKb_PeSpecialAirLwHit_Anim(gobj: G) {
    melee_core::ct!("ftKb_PeSpecialAirLwHit_Anim");
    doPeHitAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftKb_PeSpecialLwHit_IASA(_gobj: G) { melee_core::ct!("ftKb_PeSpecialLwHit_IASA");}
pub fn ftKb_PeSpecialAirLwHit_IASA(_gobj: G) { melee_core::ct!("ftKb_PeSpecialAirLwHit_IASA");}

/// The root module's Fighter-only half, as the C-signature entry.
pub fn ftKb_PeSpecialAirLwHit_Phys(gobj: G) {
    melee_core::ct!("ftKb_PeSpecialAirLwHit_Phys");
    unsafe { crate::chars::ftKirby::ftkirbyspecialpeach::ftKb_PeSpecialAirLwHit_Phys(&mut *fpp(gobj)) }
}

/// `fn_8010CD88`: the absorb shield's hit callback; goes to the Hit state.
pub fn fn_8010CD88(gobj: G) {
    melee_core::ct!("fn_8010CD88");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.kb.specialn_pe.facing_dir = melee_core::ppc::fctiwz_f32((*fp).specialn_facing_dir);
        let msid = if (*fp).ground_or_air == GA_Ground { ftKb_MS_PeSpecialLwHit } else { ftKb_MS_PeSpecialAirLwHit };
        st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 9.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        if !(*fp).u.kb.xD0.is_null() {
            ext::it_802BE100((*fp).u.kb.xD0);
        }
        ftKb_SpecialNGw_8010CD44(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftKb_PeSpecialLwHit_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_PeSpecialLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_PeSpecialLwHit_Coll");
    unsafe {
        if (st::ft_800827A0(gobj) == 0) {
            crate::chars::r11::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNGw_8010CC6C(gobj);
        }
    }
}

/// `ftKb_PeSpecialAirLwHit_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_PeSpecialAirLwHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_PeSpecialAirLwHit_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            crate::chars::r11::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNGw_8010CCD4(gobj);
        }
    }
}

/// `ftKb_SpecialNGw_8010CC6C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_SpecialNGw_8010CC6C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNGw_8010CC6C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 452 /* ftKb_MS_PeSpecialAirLwHit */, 0xC4C508Cu32 /* ((u32) ((1 << 2) | (1 << 3) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
        crate::chars::r4::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNGw_8010CD44(gobj);
    }
}

/// `ftKb_SpecialNGw_8010CCD4` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_SpecialNGw_8010CCD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNGw_8010CCD4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.kb.xCC = (0_i32 /* false */);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 450 /* ftKb_MS_PeSpecialLwHit */, 0xC4C508Cu32 /* ((u32) ((1 << 2) | (1 << 3) | (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27))) */);
        crate::chars::r4::ftKirby::ftkirbyspecialpeach::ftKb_SpecialNGw_8010CD44(gobj);
    }
}

/// `ftKb_PeSpecialLw_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_PeSpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_PeSpecialLw_Phys");
    unsafe {
        st::ft_80084F3C(gobj);
        st::ftColl_8007AEE0(gobj);
    }
}

/// `ftKb_PeSpecialLwHit_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_PeSpecialLwHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_PeSpecialLwHit_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNGw_8010CD44` (src/melee/ft/kinds/ftKirby/ftkirbyspecialpeach.c).
pub fn ftKb_SpecialNGw_8010CD44(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNGw_8010CD44");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (!(*fp).u.kb.xD0.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ftkirbyspecialpeach__fn_8010C44C);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ftkirbyspecialpeach__ftKb_SpecialNPe_8010C47C);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftKb_PeSpecialAirLwHit_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftKb_PeSpecialAirLwHit_Phys_fp");
    let a: &ftKb_DatAttrs = da(fp);
    co::fall(fp, a.specialn_pe_fall_acceleration, a.specialn_pe_unk2);
    co::calc_self_accel_deaccel(fp, a.specialn_pe_air_horizontal_momentum_preservation);
}

pub fn ftKb_SpecialNPe_8010C4AC_fp(fp: &Fighter) -> bool {
    melee_core::ct!("ftKb_SpecialNPe_8010C4AC_fp");
    let m = fp.motion_id as i32;
    !(m >= ftKb_MS_PeSpecialLw as i32 && m <= ftKb_MS_PeSpecialAirLwHit as i32)
}
