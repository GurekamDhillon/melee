//! src/melee/ft/kinds/ftPeach/ftpeachspecialn.c: Peach's Toad (neutral B, ground and air).
//! No fused sites (tools/rust-port/fma_sites.py: 0 fmuladd in every function); the only float
//! math is `self_vel.x /= specialairn_vel_x_div`. `mv.pe.specialn.facing_dir` is a C `int`, so
//! the float stores into it truncate (`as i32`). Smoke-tested by tests/tu_ftpeachspecialn.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
#[allow(unused_imports)]
use crate::chars::da as da_g;
#[allow(unused_imports)]
use melee_fighter::ftcommon as co;
#[allow(unused_imports)]
use melee_types::{ftPe_DatAttrs, ftPe_MS_SpecialAirNHit, ftPe_MS_SpecialN, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/4/13 and the original pass): re-exported under this TU.





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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut IT_802BDE18: Option<fn(*mut HSD_GObj, *mut Vec3, Fighter_Part, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802BDE18(gobj: *mut HSD_GObj, pos: *mut Vec3, part: Fighter_Part, kind: i32, facing_dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802BDE18 } {
                    Some(f) => f(gobj, pos, part, kind, facing_dir),
                    None => { return unsafe { melee_items::tu::itpeachtoad::it_802BDE18(gobj as _, pos as _, part as _, kind as _, facing_dir as _) as _ }; },
                }
            }
    pub static mut IT_802BE214: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE214(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32)  {
                match unsafe { IT_802BE214 } {
                    Some(f) => f(gobj, pos, kind, facing_dir),
                    None => { unsafe { melee_items::tu::itpeachtoadspore::it_802BE214(gobj as _, pos as _, kind as _, facing_dir as _); } return; },
                }
            }
    pub static mut IT_802BDF40: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDF40(toad: *mut Item_GObj)  {
                match unsafe { IT_802BDF40 } {
                    Some(f) => f(toad),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDF40(toad as _); } return; },
                }
            }
    pub static mut IT_802BDFA0: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDFA0(toad: *mut Item_GObj)  {
                match unsafe { IT_802BDFA0 } {
                    Some(f) => f(toad),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDFA0(toad as _); } return; },
                }
            }
    pub static mut IT_802BDFC0: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BDFC0(toad: *mut Item_GObj)  {
                match unsafe { IT_802BDFC0 } {
                    Some(f) => f(toad),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BDFC0(toad as _); } return; },
                }
            }
    pub static mut IT_802BE100: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802BE100(toad: *mut Item_GObj)  {
                match unsafe { IT_802BE100 } {
                    Some(f) => f(toad),
                    None => { unsafe { melee_items::tu::itpeachtoad::it_802BE100(toad as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_8007B1B8, ftColl_8007B1B8, crate::adapt::ftColl_8007B1B8,
        (gobj: *mut HSD_GObj, shield: *mut core::ffi::c_void, cb: HSD_GObjEvent));
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftPe_DatAttrs {
    (*fp).dat_attrs as *mut ftPe_DatAttrs
}

const cmd_phys_state: usize = 0;
const cmd_anim_state: usize = 1;
const cmd_accessory4: usize = 2;
const cmd_hit_anim_state: usize = 3;

/// `coll_mf` (static const MotionFlags).
pub const coll_mf: u32 = Ft_MF_KeepColAnimHitStatus
    | Ft_MF_SkipHit
    | Ft_MF_SkipMatAnim
    | Ft_MF_SkipColAnim
    | Ft_MF_UpdateCmd
    | Ft_MF_SkipItemVis
    | Ft_MF_Unk19
    | Ft_MF_SkipModelPartVis
    | Ft_MF_SkipModelFlags
    | Ft_MF_Unk27;

/// Callback entries stored into Fighter (C function pointers).
pub unsafe extern "C" fn cb_onAccessory4(g: G) {
    melee_core::ct!("cb_onAccessory4");
    onAccessory4(g);
}
pub unsafe extern "C" fn cb_onHitAccessory4(g: G) {
    melee_core::ct!("cb_onHitAccessory4");
    onHitAccessory4(g);
}
pub unsafe extern "C" fn cb_onEnterHitlag(g: G) {
    melee_core::ct!("cb_onEnterHitlag");
    onEnterHitlag(g);
}
pub unsafe extern "C" fn cb_onExitHitlag(g: G) {
    melee_core::ct!("cb_onExitHitlag");
    onExitHitlag(g);
}
pub unsafe extern "C" fn cb_onUnkHit(g: G) {
    melee_core::ct!("cb_onUnkHit");
    onUnkHit(g);
}

/// `onAccessory4` (static): spawns Toad on the script's accessory pulse.
pub fn onAccessory4(gobj: G) {
    melee_core::ct!("onAccessory4");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[cmd_accessory4] != 0 {
            return;
        }
        (*fp).cmd_vars[cmd_accessory4] = 1;
        {
            let mut pos: Vec3 = core::mem::zeroed();
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_109 as usize)).joint, core::ptr::null(), &mut pos);
            (*fp).u.pe.toad_gobj =
                ext::it_802BDE18(gobj, &mut pos, FtPart_109, It_Kind_Peach_Toad as i32, (*fp).facing_dir);
        }
        (*fp).x1984_heldItemSpec = (*fp).u.pe.toad_gobj;
        if !(*fp).u.pe.toad_gobj.is_null() {
            (*fp).death2_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
        }
        (*fp).accessory4_cb = None;
        (*fp).pre_hitlag_cb = Some(cb_onEnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_onExitHitlag);
    }
}

/// `doHitAccessory4` (static): spore burst above Toad.
pub fn doHitAccessory4(gobj: G) {
    melee_core::ct!("doHitAccessory4");
    unsafe {
        let fp = fpp(gobj);
        let mut pos: Vec3 = core::mem::zeroed();
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_109 as usize)).joint, core::ptr::null(), &mut pos);
        pos.y += 2.5;
        pos.z = 0.0;
        ext::it_802BE214(gobj, &mut pos, It_Kind_Peach_ToadSpore as i32, (*fp).facing_dir);
    }
}

pub fn onHitAccessory4(gobj: G) {
    melee_core::ct!("onHitAccessory4");
    unsafe {
        let fp = fpp(gobj);
        doHitAccessory4(gobj);
        (*fp).accessory4_cb = None;
    }
}

pub fn ftPe_SpecialN_DoDeath2(gobj: G) {
    melee_core::ct!("ftPe_SpecialN_DoDeath2");
    unsafe {
        let fp = fpp(gobj);
        onExitHitlag(gobj);
        (*fp).u.pe.toad_gobj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

pub fn ftPe_SpecialN_OnDeath2(gobj: G) {
    melee_core::ct!("ftPe_SpecialN_OnDeath2");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pe.toad_gobj.is_null() {
            ext::it_802BDF40((*fp).u.pe.toad_gobj);
            ftPe_SpecialN_DoDeath2(gobj);
        }
    }
}

pub fn onEnterHitlag(gobj: G) {
    melee_core::ct!("onEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pe.toad_gobj.is_null() {
            ext::it_802BDFA0((*fp).u.pe.toad_gobj);
        }
    }
}

pub fn onExitHitlag(gobj: G) {
    melee_core::ct!("onExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.pe.toad_gobj.is_null() {
            ext::it_802BDFC0((*fp).u.pe.toad_gobj);
        }
    }
}

/// `reset` (static).
pub fn reset(gobj: G) {
    melee_core::ct!("reset");
    unsafe {
        let fp = fpp(gobj);
        (*fp).cmd_vars[cmd_phys_state] = 0;
        (*fp).cmd_vars[cmd_anim_state] = 0;
        (*fp).cmd_vars[cmd_accessory4] = 0;
        (*fp).cmd_vars[cmd_hit_anim_state] = 0;
        (*fp).mv.pe.specialn.facing_dir = melee_core::ppc::fctiwz_f32((*fp).facing_dir);
        (*fp).accessory4_cb = Some(cb_onAccessory4);
    }
}

pub fn ftPe_SpecialN_Enter(gobj: G) {
    melee_core::ct!("ftPe_SpecialN_Enter");
    unsafe {
        (*fpp(gobj)).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialN as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        reset(gobj);
    }
}

pub fn ftPe_SpecialAirN_Enter(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirN_Enter");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).self_vel.x /= (*da).specialairn_vel_x_div;
        st::Fighter_ChangeMotionState(gobj, ftPe_MS_SpecialAirN as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        reset(gobj);
    }
}

/// `doAnim` (static): absorb-shield window, then `cb` once the animation ends.
pub fn doAnim(gobj: G, cb: fn(G)) {
    melee_core::ct!("doAnim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[cmd_anim_state] == 1 {
            (*fp).cmd_vars[cmd_anim_state] = 2;
            ext::ftColl_8007B1B8(gobj, &raw mut (*da).xAC as *mut _, Some(cb_onUnkHit));
            (*fp).anon_221b.set_x221B_b3(1);
            (*fp).shield_unk0 = (*da).xA8;
            (*fp).shield_unk1 = (*da).xA8;
        } else if (*fp).cmd_vars[cmd_anim_state] == 0 {
            (*fp).anon_221b.set_x221B_b0(0);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftPe_SpecialN_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialN_Anim");
    doAnim(gobj, st::ft_8008A2BC);
}

pub fn ftPe_SpecialAirN_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirN_Anim");
    doAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftPe_SpecialN_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialN_IASA");}

pub fn ftPe_SpecialAirN_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialAirN_IASA");}

pub fn ftPe_SpecialAirN_Phys(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirN_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).cmd_vars[cmd_phys_state] >= 1 {
            if (*fp).cmd_vars[cmd_phys_state] == 1 {
                (*fp).cmd_vars[cmd_phys_state] = 2;
                if (*fp).u.pe.specialairn_used == 0 {
                    (*fp).u.pe.specialairn_used = 1;
                    (*fp).self_vel.y = (*da).specialairn_vel_y;
                } else {
                    (*fp).self_vel.y = 0.0;
                }
            }
            melee_fighter::ftcommon::fall(&mut *fp, (*da).xA0, (*da).xA4);
        } else {
            melee_fighter::ftcommon::fall_basic(&mut *fp);
        }
        melee_fighter::ftcommon::calc_self_accel_deaccel(&mut *fp, (*da).x98);
        st::ftColl_8007AEE0(gobj);
    }
}

pub fn ftPe_SpecialN_Coll(gobj: G) {
    melee_core::ct!("ftPe_SpecialN_Coll");
    if st::ft_800827A0(gobj) == 0 {
        doColl(gobj);
    }
}

pub fn ftPe_SpecialAirN_Coll(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirN_Coll");
    if st::ft_80081D0C(gobj) != 0 {
        doAirColl(gobj);
    }
}

/// `setupColl` (static).
pub fn setupColl(gobj: G) {
    melee_core::ct!("setupColl");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if !(*fp).u.pe.toad_gobj.is_null() {
            (*fp).death2_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
        }
        (*fp).accessory4_cb = Some(cb_onAccessory4);
        (*fp).pre_hitlag_cb = Some(cb_onEnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_onExitHitlag);
        if (*fp).cmd_vars[cmd_anim_state] == 2 {
            ext::ftColl_8007B1B8(gobj, &raw mut (*da).xAC as *mut _, Some(cb_onUnkHit));
            (*fp).anon_221b.set_x221B_b3(1);
        }
    }
}

/// `doColl` (static).
pub fn doColl(gobj: G) {
    melee_core::ct!("doColl");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftPe_MS_SpecialAirN as i32, coll_mf);
        if (*fp).cmd_vars[cmd_phys_state] == 1 {
            (*fp).cmd_vars[cmd_phys_state] = 2;
        }
        setupColl(gobj);
    }
}

/// `doAirColl` (static).
pub fn doAirColl(gobj: G) {
    melee_core::ct!("doAirColl");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.pe.specialairn_used = 0;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftPe_MS_SpecialN as i32, coll_mf);
        setupColl(gobj);
    }
}

/// `doHitAnim` (static).
pub fn doHitAnim(gobj: G, cb: fn(G)) {
    melee_core::ct!("doHitAnim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[cmd_hit_anim_state] != 0 {
            (*fp).cmd_vars[cmd_hit_anim_state] = 0;
            (*fp).accessory4_cb = Some(cb_onHitAccessory4);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            cb(gobj);
        }
    }
}

pub fn ftPe_SpecialNHit_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialNHit_Anim");
    doHitAnim(gobj, st::ft_8008A2BC);
}

pub fn ftPe_SpecialAirNHit_Anim(gobj: G) {
    melee_core::ct!("ftPe_SpecialAirNHit_Anim");
    doHitAnim(gobj, st::ftCo_Fall_Enter);
}

pub fn ftPe_SpecialNHit_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialNHit_IASA");}

pub fn ftPe_SpecialAirNHit_IASA(_gobj: G) { melee_core::ct!("ftPe_SpecialAirNHit_IASA");}

/// `onUnkHit` (static): the absorb shield was hit; switch to the Hit motion.
pub fn onUnkHit(gobj: G) {
    melee_core::ct!("onUnkHit");
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.pe.specialn.facing_dir = melee_core::ppc::fctiwz_f32((*fp).specialn_facing_dir);
        let msid = if (*fp).ground_or_air == GA_Ground {
            ftPe_MS_SpecialNHit
        } else {
            ftPe_MS_SpecialAirNHit
        };
        st::Fighter_ChangeMotionState(gobj, msid as i32, Ft_MF_None, 9.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        if !(*fp).u.pe.toad_gobj.is_null() {
            ext::it_802BE100((*fp).u.pe.toad_gobj);
        }
        setupHitColl(gobj);
    }
}

/// `ftPe_SpecialAirNHit_Phys(gobj)`: the retail motion-state callback; the body is the fighter-only port `chars::ftPeach::ftpeachspecialn::ftPe_SpecialAirNHit_Phys_fp`.
pub fn ftPe_SpecialAirNHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_SpecialAirNHit_Phys");
    unsafe { crate::chars::ftPeach::ftpeachspecialn::ftPe_SpecialAirNHit_Phys_fp(&mut *((*gobj).user_data as *mut Fighter)) }
}

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftPe_SpecialN_IsActive(gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("ftPe_SpecialN_IsActive");
    let fp = unsafe { &*((*gobj).user_data as *const Fighter) };
    let m = fp.motion_id as i32;
    !(m >= ftPe_MS_SpecialN as i32 && m <= ftPe_MS_SpecialAirNHit as i32)
}

/// `ftPe_SpecialN_Phys` (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn ftPe_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_SpecialN_Phys");
    unsafe {
        st::ft_80084F3C(gobj);
        st::ftColl_8007AEE0(gobj);
    }
}

/// `ftPe_SpecialNHit_Phys` (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn ftPe_SpecialNHit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_SpecialNHit_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftPe_SpecialNHit_Coll` (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn ftPe_SpecialNHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_SpecialNHit_Coll");
    unsafe {
        if (st::ft_800827A0(gobj) == 0) {
            crate::chars::r4::ftPeach::ftpeachspecialn::doHitColl(gobj);
        }
    }
}

/// `ftPe_SpecialAirNHit_Coll` (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn ftPe_SpecialAirNHit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_SpecialAirNHit_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0) {
            crate::chars::r4::ftPeach::ftpeachspecialn::doAirHitColl(gobj);
        }
    }
}

/// `doHitColl` (static) (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn doHitColl(gobj: *mut HSD_GObj) {
    melee_core::ct!("doHitColl");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 368 /* ftPe_MS_SpecialAirNHit */, 0xC4C508Cu32 /* coll_mf */);
        crate::chars::r4::ftPeach::ftpeachspecialn::setupHitColl(gobj);
    }
}

/// `doAirHitColl` (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn doAirHitColl(gobj: *mut HSD_GObj) {
    melee_core::ct!("doAirHitColl");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.pe.specialairn_used = (0_i32 /* false */);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 366 /* ftPe_MS_SpecialNHit */, 0xC4C508Cu32 /* coll_mf */);
        crate::chars::r4::ftPeach::ftpeachspecialn::setupHitColl(gobj);
    }
}

/// `setupHitColl` (static) (src/melee/ft/kinds/ftPeach/ftpeachspecialn.c).
pub fn setupHitColl(gobj: *mut HSD_GObj) {
    melee_core::ct!("setupHitColl");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (!(*fp).u.pe.toad_gobj.is_null()) {
            (*fp).death2_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftpeach__ftPe_Init_OnDeath2);
        }
        (*fp).pre_hitlag_cb = Some(crate::chars::cb::ftpeachspecialn__onEnterHitlag);
        (*fp).post_hitlag_cb = Some(crate::chars::cb::ftpeachspecialn__onExitHitlag);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
pub fn ftPe_SpecialAirNHit_Phys_fp(fp: &mut Fighter) {
    melee_core::ct!("ftPe_SpecialAirNHit_Phys_fp");
    let a: &ftPe_DatAttrs = da_g(fp);
    co::fall(fp, a.xA0, a.xA4);
    co::calc_self_accel_deaccel(fp, a.x98);
}
