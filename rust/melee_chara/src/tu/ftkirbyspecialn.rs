//! src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c: Kirby's Inhale / swallow (neutral B), the Eat*
//! states and the Hat helpers. Fused sites (tools/rust-port/fma_sites.py, f32 `fmadds`): the
//! `x + off * facing` adds of `ftKb_SpecialN_800F5B5C`, `800F5DE8`, `800F5EA8` and
//! `ftKb_SpecialNCapture0_Anim` / `ftKb_SpecialNCapture_Anim`. Pure parts checked against the C by
//! tests/tu_ftkirbyspecialn.rs; the states have a smoke test through recording hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::chars::da;
#[allow(unused_imports)]
use melee_types::{ftKb_DatAttrs, ftKb_MS_EatFall, ftKb_MS_EatWait, Vec2, Vec3};
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

/// The root module's Fighter-only halves of the one-line accessors; the C-signature (gobj) entries
/// below call them.
use crate::chars::ftKirby::ftkirbyspecialn as half;

/// Already ported (rounds 2, 3, 5): re-exported under this TU.




pub type G = *mut HSD_GObj;

/// Calls into TUs that are not ported yet (or ported in other crates: the default then calls the
/// port). Each is a settable hook; unset it panics, or runs the port.
pub mod ext {
    use core::ffi::c_void;
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
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    // Unported TUs.
    pub static mut MPUPDATEFLOORSKIP: Option<fn(*mut CollData)> = None;
    #[allow(non_snake_case)]
    pub fn mpUpdateFloorSkip(coll: *mut CollData) {
        match unsafe { MPUPDATEFLOORSKIP } {
            Some(f) => f(coll),
            None => { unsafe { melee_world::tu::mpcoll::mpUpdateFloorSkip(coll as _); } },
        }
    }
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (n: i32) -> i32);
    pub static mut KB_800F19AC: Option<fn(G) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F19AC(g: G)  {
                match unsafe { KB_800F19AC } {
                    Some(f) => f(g),
                    None => { unsafe { crate::tu::ftkirby::ftKb_SpecialN_800F19AC(g as _); } return; },
                }
            }
    pub static mut KB_800F190C: Option<fn(G, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F190C(g: G, kind: i32)  {
                match unsafe { KB_800F190C } {
                    Some(f) => f(g, kind),
                    None => { unsafe { crate::tu::ftkirby::ftKb_SpecialN_800F190C(g as _, kind as _); } return; },
                }
            }
    pub static mut KB_800EEEC4: Option<fn(G, i32) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800EEEC4(g: G, kind: i32)  {
                match unsafe { KB_800EEEC4 } {
                    Some(f) => f(g, kind),
                    None => { unsafe { crate::tu::ftkirby::ftKb_SpecialN_800EEEC4(g as _, kind as _); } return; },
                }
            }
    pub static mut KB_800F1BAC: Option<fn(G, i32, bool) > = None;
            #[allow(non_snake_case)]
            pub fn ftKb_SpecialN_800F1BAC(g: G, x: i32, b: bool)  {
                match unsafe { KB_800F1BAC } {
                    Some(f) => f(g, x, b),
                    None => { unsafe { crate::tu::ftkirby::ftKb_SpecialN_800F1BAC(g as _, x as _, b as _); } return; },
                }
            }
    pub static mut IT_802ADA1C: Option<fn(*mut Vec3, *mut Vec3, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802ADA1C(pos: *mut Vec3, vel: *mut Vec3, dir: f32)  {
                match unsafe { IT_802ADA1C } {
                    Some(f) => f(pos, vel, dir),
                    None => { unsafe { melee_items::tu::it_2ADA::it_802ADA1C(pos as _, vel as _, dir as _); } return; },
                }
            }
    pub static mut IT_802F23AC: Option<fn(*mut Item_GObj, *const Vec3) -> f64> = None;
            #[allow(non_snake_case)]
            pub fn it_802F23AC(item: *mut Item_GObj, pos: *const Vec3) -> f64 {
                match unsafe { IT_802F23AC } {
                    Some(f) => f(item, pos),
                    None => { return unsafe { melee_items::tu::itkirby_2F23::it_802F23AC(item as _, pos as _) as _ }; },
                }
            }
    pub static mut IT_802F2810: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802F2810(item: *mut Item_GObj)  {
                match unsafe { IT_802F2810 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itkirby_2F23::it_802F2810(item as _); } return; },
                }
            }
    pub static mut IT_802F23EC: Option<fn(*mut Item_GObj, G, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802F23EC(item: *mut Item_GObj, g: G, dir: f32)  {
                match unsafe { IT_802F23EC } {
                    Some(f) => f(item, g, dir),
                    None => { unsafe { melee_items::tu::itkirby_2F23::it_802F23EC(item as _, g as _, dir as _); } return; },
                }
            }
    crate::port_hook!(IT_802F28C8, it_802F28C8, crate::adapt::it_802F28C8, (item: *mut Item_GObj, flag: i32, f: f32));

    pub static mut IT_802F295C: Option<fn(G, *mut itUnk2_DatAttrs) -> f32> = None;
            #[allow(non_snake_case)]
            pub fn it_802F295C(g: G, attr: *mut itUnk2_DatAttrs) -> f32 {
                match unsafe { IT_802F295C } {
                    Some(f) => f(g, attr),
                    None => { return unsafe { melee_items::tu::it_2F28::it_802F295C(g as _, attr as _) as _ }; },
                }
            }
    crate::port_hook!(LBANIM_8001E8F8, lbAnim_8001E8F8, crate::adapt::lbAnim_8001E8F8, (tree: *mut FigaTree) -> f32);
    pub static mut LB_8000B1CC: Option<fn(*mut c_void, *const Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(joint: *mut c_void, off: *const Vec3, out: *mut Vec3) {
        match unsafe { LB_8000B1CC } {
            Some(f) => f(joint, off, out),
            None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(joint as _, off as _, out as _); } },
        }
    }
    // `efAsync_Spawn(gobj, queue, kind, gfx_id, ...)` (ef/efasync.c, variadic): trailing pointers.
    crate::port_hook!(EFASYNC_SPAWN, efAsync_Spawn, crate::adapt::efAsync_Spawn_slice, (gobj: G, queue: *mut c_void, kind: u32, gfx_id: u32, args: &[*mut c_void]));
    // Ported TUs (default = the port).
    hook_or!(FTCO_800BD1DC, ftCo_800BD1DC, melee_fighter::tu::ftCo_CaptureKirby::ftCo_800BD1DC, (g: G, victim: G));
    hook_or!(FTCO_800BD19C, ftCo_800BD19C, |g, p: *const Vec3| melee_fighter::tu::ftCo_CaptureKirby::ftCo_800BD19C(g, unsafe { &*p }), (g: G, pos: *const Vec3) -> f32);
    hook_or!(FTCO_800BD620, ftCo_800BD620, melee_fighter::tu::ftCo_CaptureWaitKirby::ftCo_800BD620, (g: G));
    hook_or!(FTCO_800BD9E0, ftCo_800BD9E0, melee_fighter::tu::ftCo_ThrownKirby::ftCo_800BD9E0, (g: G, victim: G) -> i32);
    hook_or!(FTCO_800BDB58, ftCo_800BDB58, melee_fighter::tu::ftCo_ThrownKirby::ftCo_800BDB58, (g: G, thrower: G));
    hook_or!(FTCO_800BE000, ftCo_800BE000, melee_fighter::tu::ftCo_ThrownKirby::ftCo_800BE000, (g: G, thrower: G));
    hook_or!(FTCO_800DE2CC, ftCo_800DE2CC, melee_fighter::tu::ftCo_Throw::ftCo_800DE2CC, (g: G, other: G));
    hook_or!(FTCO_8008EC90, ftCo_8008EC90, melee_fighter::tu::ftCo_Damage::ftCo_8008EC90, (g: G));
    hook_or!(FTCOLL_8007B8CC, ftColl_8007B8CC, |f: *mut Fighter, g| melee_fighter::ft0df::ftcoll_8007b8cc(unsafe { &mut *f }, g), (fp: *mut Fighter, grabber: G));
    hook_or!(FTDATA_80085E50, ftData_80085E50, |f, m| melee_fighter::tu::ftdata::ftData_80085E50(f, m), (fp: *mut Fighter, msid: i32) -> *mut FigaTree);
    hook_or!(FTCOMMON_8007E2D0, ftCommon_8007E2D0, |f: *mut Fighter, m, a, b, c| melee_fighter::ftcommon2::ftcommon_8007e2d0(unsafe { &mut *f }, m, a, b, c),
        (fp: *mut Fighter, mask: i16, cb0: HSD_GObjEvent, cb1: HSD_GObjEvent, throw_cb: HSD_GObjInteraction));
    hook_or!(FTWALK_800DFCA4, ftWalkCommon_800DFCA4, melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfca4,
        (g: G, msid: i32, flags: u32, anim_start: f32, f0: f32, f1: f32, f2: f32, r0: f32, r1: f32, r2: f32, accel: f32));
    hook_or!(FTWALK_800DFC70, ftWalkCommon_800DFC70, melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfc70, (g: G) -> bool);
    hook_or!(FTWALK_800DFDDC, ftWalkCommon_800DFDDC, melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfddc, (g: G));
    hook_or!(FTWALK_800DFEC8, ftWalkCommon_800DFEC8, melee_fighter::tu::ftwalkcommon::ftwalkcommon_800dfec8, (g: G, cb: fn(G, f32)));
    hook_or!(FTWALK_800E0060, ftWalkCommon_800E0060, melee_fighter::tu::ftwalkcommon::ftwalkcommon_800e0060, (g: G));
    hook_or!(FTCO_JUMP_GETINPUT, ftCo_Jump_GetInput, melee_fighter::tu::ftCo_Jump::ftCo_Jump_GetInput, (g: G) -> u32);
    hook_or!(FTCO_800CB110, ftCo_800CB110, melee_fighter::tu::ftCo_Jump::ftCo_800CB110, (g: G, b: bool, mul: f32));
    hook_or!(FTCO_JUMP_PHYS_INNER, ftCo_Jump_Phys_Inner, melee_fighter::tu::ftCo_Jump::ftCo_Jump_Phys_Inner, (g: G));
    hook_or!(FTCO_KNEEBEND_SHORTHOP, ftCo_KneeBend_Check_ShortHop, melee_fighter::tu::ftCo_KneeBend::ftCo_KneeBend_Check_ShortHop, (g: G));
    hook_or!(FT_8008A1FC, ft_8008A1FC, melee_fighter::ft084e::ft_8008a1fc, (g: G) -> bool);
}

/// `ftCommon_GroundAirColl_MF` (ftCommon/forward.h) and the Kirby motion-flag sets built on it.
pub const ftCommon_GroundAirColl_MF: u32 = 0xC4C5080;
pub const ftKb_MF_SpecialN_Coll: u32 = ftCommon_GroundAirColl_MF | Ft_MF_KeepGfx as u32;
pub const ftKb_MF_SpecialN_Capture_Coll: u32 = ftKb_MF_SpecialN_Coll | Ft_MF_SkipModel as u32;
pub const ftKb_MF_SpecialN_EatFall_Coll: u32 = ftCommon_GroundAirColl_MF | Ft_MF_SkipModel as u32;
pub const ftKb_MF_SpecialN_Loop_Coll: u32 =
    ftKb_MF_SpecialN_Capture_Coll | Ft_MF_SkipHit as u32 | Ft_MF_KeepSfx as u32;
pub const ftKb_MF_SpecialN_LoopRumble_Coll: u32 = ftKb_MF_SpecialN_Loop_Coll | Ft_MF_SkipRumble as u32;

#[inline(always)]
pub(crate) unsafe fn fpp(gobj: G) -> *mut Fighter {
    melee_core::ct!("fpp");
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
pub(crate) unsafe fn da_of(fp: *mut Fighter) -> *const ftKb_DatAttrs {
    melee_core::ct!("da_of");
    (*fp).dat_attrs as *const ftKb_DatAttrs
}

#[inline(always)]
pub(crate) unsafe fn change_ms(gobj: G, ms: i32, flags: u32) {
    melee_core::ct!("change_ms");
    st::Fighter_ChangeMotionState(gobj, ms, flags, 0.0, 1.0, 0.0, core::ptr::null_mut());
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
pub(crate) unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    melee_core::ct!("set_effect_hitlag_callbacks");
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h): death2 / take-damage callbacks of the swallow.
pub fn ftKb_SpecialN_set_cbs(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_set_cbs");
    unsafe {
        let fp = fpp(gobj);
        (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
    }
}

/// `FTKB_IS_KIRBY` (ft/forward.h, TARGET_PC).
#[allow(non_snake_case)]
pub fn FTKB_IS_KIRBY(kind: i32) -> bool {
    melee_core::ct!("FTKB_IS_KIRBY");
    melee_fighter::tu::ftco_800c7220::ftkb_is_kirby(kind as FighterKind)
}

// ---------------------------------------------------------------------------------------------
// Callbacks stored in Fighter fields (extern "C" entries of the Rust ports below).
// ---------------------------------------------------------------------------------------------
pub unsafe extern "C" fn cb_fn_800F6178(g: G) {
    melee_core::ct!("cb_fn_800F6178");
    fn_800F6178(g)
}
pub unsafe extern "C" fn cb_fn_800F6210(g: G) {
    melee_core::ct!("cb_fn_800F6210");
    fn_800F6210(g)
}
pub unsafe extern "C" fn cb_fn_800F6280(g: G) {
    melee_core::ct!("cb_fn_800F6280");
    fn_800F6280(g)
}
pub unsafe extern "C" fn cb_fn_800F6318(g: G) {
    melee_core::ct!("cb_fn_800F6318");
    fn_800F6318(g)
}
pub unsafe extern "C" fn cb_ftCo_800BD1DC(g: G, victim: G) {
    melee_core::ct!("cb_ftCo_800BD1DC");
    ext::ftCo_800BD1DC(g, victim)
}
pub unsafe extern "C" fn cb_fn_800F6588(g: G) {
    melee_core::ct!("cb_fn_800F6588");
    fn_800F6588(g)
}
pub unsafe extern "C" fn cb_fn_800F6638(g: G) {
    melee_core::ct!("cb_fn_800F6638");
    fn_800F6638(g)
}
pub unsafe extern "C" fn cb_fn_800F66E8(g: G) {
    melee_core::ct!("cb_fn_800F66E8");
    fn_800F66E8(g)
}
pub unsafe extern "C" fn cb_fn_800F6798(g: G) {
    melee_core::ct!("cb_fn_800F6798");
    fn_800F6798(g)
}

// ---------------------------------------------------------------------------------------------
// Pure-ish helpers
// ---------------------------------------------------------------------------------------------


// ---------------------------------------------------------------------------------------------
// C-signature entries over the root module's Fighter-only halves.
// ---------------------------------------------------------------------------------------------
pub fn ftKb_SpecialN_800F58AC(gobj: G, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F58AC");
    unsafe { half::ftKb_SpecialN_800F58AC(&*fpp(gobj), victim_self_vel, victim_facing_dir) }
}
/// Returns `floor.flags & 0x100` (an int: MSL `bool` is int).
pub fn ftKb_SpecialN_800F597C(gobj: G) -> i32 {
    melee_core::ct!("ftKb_SpecialN_800F597C");
    unsafe { half::ftKb_SpecialN_800F597C(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5A04(gobj: G, arg1: f32) {
    melee_core::ct!("ftKb_SpecialN_800F5A04");
    unsafe { half::ftKb_SpecialN_800F5A04(&mut *fpp(gobj), arg1) }
}
pub fn ftKb_SpecialN_800F5A38(gobj: G, victim_fp: *mut Fighter) -> bool {
    melee_core::ct!("ftKb_SpecialN_800F5A38");
    unsafe { half::ftKb_SpecialN_800F5A38(&*fpp(gobj), &*victim_fp) }
}
pub fn ftKb_SpecialN_800F5A60(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A60");
    unsafe { half::ftKb_SpecialN_800F5A60(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5A88(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A88");
    unsafe { half::ftKb_SpecialN_800F5A88(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5AB0(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AB0");
    unsafe { half::ftKb_SpecialN_800F5AB0(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5AF0(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AF0");
    unsafe { half::ftKb_SpecialN_800F5AF0(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5B00(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B00");
    unsafe { half::ftKb_SpecialN_800F5B00(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5B10(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B10");
    unsafe { half::ftKb_SpecialN_800F5B10(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5B20(gobj: G, out: &mut Vec2) {
    melee_core::ct!("ftKb_SpecialN_800F5B20");
    unsafe { half::ftKb_SpecialN_800F5B20(&*fpp(gobj), out) }
}
pub fn ftKb_SpecialN_800F5B3C(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B3C");
    unsafe { half::ftKb_SpecialN_800F5B3C(&*fpp(gobj)) }
}
pub fn ftKb_SpecialN_800F5B4C(gobj: G) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B4C");
    unsafe { half::ftKb_SpecialN_800F5B4C(&*fpp(gobj)) }
}

/// `ftKb_SpecialN_800F58D8`: spit-out velocity of a swallowed fighter (star); returns gravity.
pub fn ftKb_SpecialN_800F58D8(gobj: G, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F58D8");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let cos_val = cosf(da.specialn_spit_out_release_angle);
        let temp = da.specialn_swallow_star_vertical_velocity * cos_val;
        victim_self_vel.x = victim_facing_dir * temp;
        victim_self_vel.y = da.specialn_swallow_star_vertical_velocity * sinf(da.specialn_spit_out_release_angle);
        victim_self_vel.z = 0.0;
        da.specialn_swallow_star_gravity
    }
}

pub fn ftKb_SpecialN_800F5954(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F5954");
    unsafe {
        let fp = fpp(gobj);
        ext::mpUpdateFloorSkip(&mut (*fp).coll_data);
    }
}

pub fn ftKb_SpecialN_800F598C(gobj: G, arg1: i32) {
    melee_core::ct!("ftKb_SpecialN_800F598C");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        if (*fp).ground_or_air == GA_Ground {
            let new_var = arg1 as f32;
            ftKb_SpecialS_800F6BB0(gobj);
            (*fp).self_vel.y = da.specialn_stop_momentum * new_var;
        }
    }
}

/// `ftKb_KirbyFtData` (TARGET_PC): the Kirby ftData, or the first m-ex Kirby-based fighter's.
pub fn ftKb_KirbyFtData() -> *mut ftData {
    melee_core::ct!("ftKb_KirbyFtData");
    unsafe {
        let list = &melee_fighter::tu::ftdata::gFtDataList;
        if !list[Ft_Kind_Kirby as usize].is_null() {
            return list[Ft_Kind_Kirby as usize];
        }
        let mut k = Ft_Kind_Mex0 as usize;
        while k < Ft_Kind_Max as usize {
            if !list[k].is_null() && FTKB_IS_KIRBY(k as i32) {
                return list[k];
            }
            k += 1;
        }
        list[Ft_Kind_Kirby as usize]
    }
}

unsafe fn ext_attr() -> *const ftKb_DatAttrs {
    (*ftKb_KirbyFtData()).ext_attr as *const ftKb_DatAttrs
}

pub fn ftKb_SpecialN_800F5A70() -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A70");
    unsafe { (*ext_attr()).specialn_gravity_of_inhaled }
}
pub fn ftKb_SpecialN_800F5A98() -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A98");
    unsafe { (*ext_attr()).specialn_spit_spin }
}
pub fn ftKb_SpecialN_800F5AC0() -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AC0");
    unsafe { (*ext_attr()).specialn_star_duration_divisor }
}
pub fn ftKb_SpecialN_800F5AD8() -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AD8");
    unsafe { (*ext_attr()).specialn_star_deceleration_rate }
}

/// `ftKb_SpecialN_800F5B5C`: where the swallowed target is held (`cur_pos` + offset * facing).
/// The x add is a fused site in the shipping build (`fmadds(off, facing, cur_pos.x)`).
pub fn ftKb_SpecialN_800F5B5C(gobj: G, output: &mut Vec3) {
    melee_core::ct!("ftKb_SpecialN_800F5B5C");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        *output = (*fp).cur_pos;
        output.x = fmadds(da.specialn_x_offset_inhaled, (*fp).facing_dir, output.x);
        output.y += da.specialn_y_offset_inhaled;
    }
}

/// `ftKb_SpecialN_800F5BA4`: roll for losing the copy ability on hit.
pub fn ftKb_SpecialN_800F5BA4(fp: *mut Fighter) {
    melee_core::ct!("ftKb_SpecialN_800F5BA4");
    unsafe {
        let da = &*da_of(fp);
        if FTKB_IS_KIRBY((*fp).kind as i32)
            && (*fp).u.kb.hat.kind as i32 != Ft_Kind_Kirby as i32
            && (*fp).u.kb.hat.x8_b0() == 0
            && (*fp).victim_gobj as usize == 0
            && (*fp).dmg.x1860_element != HitElement_Cape as _
            && ext::HSD_Randi(melee_core::ppc::fctiwz_f32(da.specialn_odds_lose_ability_on_hit)) == 0
        {
            ftKb_SpecialN_800F5D04((*fp).gobj, true);
        }
    }
}

pub fn ftKb_SpecialN_800F5C34(fp: *mut Fighter) {
    melee_core::ct!("ftKb_SpecialN_800F5C34");
    unsafe {
        let da = &*da_of(fp);
        if FTKB_IS_KIRBY((*fp).kind as i32)
            && (*fp).u.kb.hat.kind as i32 != Ft_Kind_Kirby as i32
            && (*fp).u.kb.hat.x8_b0() == 0
            && (*fp).victim_gobj as usize == 0
            && ext::HSD_Randi(melee_core::ppc::fctiwz_f32(da.specialn_odds_lose_ability_on_hit)) == 0
        {
            let msid = (*fp).motion_id as i32;
            if (0x18F..0x220).contains(&msid) {
                ext::ftCo_8008EC90((*fp).gobj);
                ftKb_SpecialN_800F5D04((*fp).gobj, true);
                if (*fp).ground_or_air as i32 == GA_Ground as i32 {
                    st::ft_8008A2BC((*fp).gobj);
                } else {
                    st::ftCo_Fall_Enter((*fp).gobj);
                }
                return;
            }
            ftKb_SpecialN_800F5D04((*fp).gobj, true);
        }
    }
}

/// `ftKb_SpecialN_800F5D04`: lose the copy ability (and spawn the ability star when `arg1`).
pub fn ftKb_SpecialN_800F5D04(gobj: G, arg1: bool) {
    melee_core::ct!("ftKb_SpecialN_800F5D04");
    unsafe {
        let fp = fpp(gobj);
        ext::ftKb_SpecialN_800F19AC(gobj);
        ext::ftKb_SpecialN_800F190C(gobj, (*fp).u.kb.hat.kind as i32);
        ext::ftKb_SpecialN_800EEEC4(gobj, (*fp).u.kb.hat.kind as i32);
        if (*fp).u.kb.hat.kind as i32 != Ft_Kind_Kirby as i32 && arg1 {
            let fp2 = fpp(gobj);
            let da = &*da_of(fp2);
            let mut pos = Vec3 { x: (*fp2).cur_pos.x, y: (*fp2).cur_pos.y + da.specialn_ability_loss_star_x, z: 0.0 };
            let mut vel = Vec3 { x: da.specialn_ability_loss_star_y * (*fp2).facing_dir, y: da.specialn_ability_loss_star_z, z: 0.0 };
            ext::it_802ADA1C(&mut pos, &mut vel, (*fp2).facing_dir);
            fh::play_sfx(&mut *fp, 0x22305, 0x7F, 0x40);
        }
        (*fp).u.kb.hat.kind = Ft_Kind_Kirby;
    }
}

/// The "pull the swallowed target in" position: `cur_pos + (x_off * facing, y_off)`; the x add is
/// a fused site.
unsafe fn inhale_pos(fp: *mut Fighter, da: &ftKb_DatAttrs) -> Vec3 {
    let mut pos = (*fp).cur_pos;
    pos.x = fmadds(da.specialn_x_offset_inhaled, (*fp).facing_dir, pos.x);
    pos.y += da.specialn_y_offset_inhaled;
    pos
}

pub fn ftKb_SpecialN_800F5DE8(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F5DE8");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let pos = inhale_pos(fp, da);
        if ext::it_802F23AC((*fp).target_item_gobj, &pos) < (da.specialn_inhale_velocity * da.specialn_inhale_velocity) as f64 {
            ext::it_802F2810((*fp).target_item_gobj);
            if (*fp).ground_or_air == GA_Air {
                ftKb_SpecialN_800F63EC(gobj);
            } else {
                ftKb_SpecialN_800F6388(gobj);
            }
        }
    }
}

pub fn ftKb_SpecialN_800F5EA8(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F5EA8");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let pos = inhale_pos(fp, da);
        if ext::ftCo_800BD19C((*fp).victim_gobj, &pos) < da.specialn_inhale_velocity * da.specialn_inhale_velocity {
            ext::ftCo_800BD620((*fp).victim_gobj);
            if (*fp).ground_or_air == GA_Air {
                ftKb_SpecialN_800F63EC(gobj);
            } else {
                ftKb_SpecialN_800F6388(gobj);
            }
        }
    }
}

/// Shared body of the two Inhale entries: reset throw state, cache the walk anim frames, enter.
unsafe fn inhale_enter(gobj: G, ms: i32, capture_cb: unsafe extern "C" fn(G), release_cb: unsafe extern "C" fn(G)) {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    (*fp).x2210.throw_flags = 0;
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).u.kb.xE0 = 4;
    // Ends up being 00 28 according to data sheet
    (*fp).u.kb.xE4 = da.jumpaerial_unk;
    (*fp).u.kb.xE8 = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, ftCo_MS_HammerFall as i32));
    (*fp).u.kb.xEC = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, ftCo_MS_HammerJump as i32));
    (*fp).u.kb.xF0 = ext::lbAnim_8001E8F8(ext::ftData_80085E50(fp, ftCo_MS_HammerLanding as i32));
    change_ms(gobj, ms, 0);
    (*fp).set_x2222_b2(1);
    st::ftAnim_8006EBA4(gobj);
    let fp = fpp(gobj);
    ext::ftCommon_8007E2D0(fp, 16, Some(capture_cb), Some(release_cb), Some(cb_ftCo_800BD1DC));
    (*fp).set_x2225_b1(1);
}

pub fn ftKb_SpecialN_800F5F68(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F5F68");
    unsafe { inhale_enter(gobj, ftKb_MS_SpecialN as i32, cb_fn_800F6210, cb_fn_800F6178) }
}

pub fn ftKb_SpecialN_800F6070(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F6070");
    unsafe { inhale_enter(gobj, ftKb_MS_SpecialAirN as i32, cb_fn_800F6318, cb_fn_800F6280) }
}

pub fn fn_800F6178(gobj: G) {
    melee_core::ct!("fn_800F6178");
    unsafe {
        let fp = fpp(gobj);
        ext::it_802F23EC((*fp).target_item_gobj, gobj, -(*fp).facing_dir);
        change_ms(gobj, ftKb_MS_SpecialNCapture1 as i32, (Ft_MF_KeepGfx | Ft_MF_SkipModel | Ft_MF_KeepSfx) as u32);
        (*fp).set_x2222_b2(1);
        (*fp).u.kb.set_xF4_b0(1);
        ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn fn_800F6210(gobj: G) {
    melee_core::ct!("fn_800F6210");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ftKb_MS_SpecialNCapture0 as i32, 0x212);
        (*fp).set_x2222_b2(1);
        (*fp).u.kb.set_xF4_b0(0);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn fn_800F6280(gobj: G) {
    melee_core::ct!("fn_800F6280");
    unsafe {
        let fp = fpp(gobj);
        ext::it_802F23EC((*fp).target_item_gobj, gobj, -(*fp).facing_dir);
        change_ms(gobj, ftKb_MS_SpecialAirNCapture1 as i32, (Ft_MF_KeepGfx | Ft_MF_SkipModel | Ft_MF_KeepSfx) as u32);
        (*fp).set_x2222_b2(1);
        (*fp).u.kb.set_xF4_b0(1);
        ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn fn_800F6318(gobj: G) {
    melee_core::ct!("fn_800F6318");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ftKb_MS_SpecialAirNCapture0 as i32, 0x212);
        (*fp).set_x2222_b2(1);
        (*fp).u.kb.set_xF4_b0(0);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

/// `fn_800F64C8` (walk-anim callback of the Eat walk: `ftWalkCommon_800DFEC8`'s `arg_cb`).
pub fn fn_800F64C8(gobj: G, anim_start: f32) {
    melee_core::ct!("fn_800F64C8");
    unsafe {
        let fp = fpp(gobj);
        let ca = &(*fp).co_attrs;
        let da = &*da_of(fp);
        ext::ftWalkCommon_800DFCA4(
            gobj,
            ftKb_MS_EatWalkSlow as i32,
            Ft_MF_SkipModel as u32,
            anim_start,
            (*fp).u.kb.xE8,
            (*fp).u.kb.xEC,
            (*fp).u.kb.xF0,
            ca.slow_walk_max,
            ca.mid_walk_point,
            ca.fast_walk_min,
            da.specialn_walk_speed,
        );
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn fn_800F6588(gobj: G) {
    melee_core::ct!("fn_800F6588");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftKb_MS_SpecialAirN as i32, ftKb_MF_SpecialN_Loop_Coll);
        set_effect_hitlag_callbacks(fp);
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, 0x10, Some(cb_fn_800F6318), Some(cb_fn_800F6280), Some(cb_ftCo_800BD1DC));
        (*fp).set_x2225_b1(1);
    }
}

pub fn fn_800F6638(gobj: G) {
    melee_core::ct!("fn_800F6638");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_SpecialN as i32, 0x0C4C529A, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        set_effect_hitlag_callbacks(fp);
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, 0x10, Some(cb_fn_800F6210), Some(cb_fn_800F6178), Some(cb_ftCo_800BD1DC));
        (*fp).set_x2225_b1(1);
    }
}

pub fn fn_800F66E8(gobj: G) {
    melee_core::ct!("fn_800F66E8");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_SpecialAirNLoop as i32, 0x0C4C5A9A, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        set_effect_hitlag_callbacks(fp);
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, 0x10, Some(cb_fn_800F6318), Some(cb_fn_800F6280), Some(cb_ftCo_800BD1DC));
        (*fp).set_x2225_b1(1);
    }
}

pub fn fn_800F6798(gobj: G) {
    melee_core::ct!("fn_800F6798");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, ftKb_MS_SpecialNLoop as i32, ftKb_MF_SpecialN_LoopRumble_Coll);
        set_effect_hitlag_callbacks(fp);
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, 0x10, Some(cb_fn_800F6210), Some(cb_fn_800F6178), Some(cb_ftCo_800BD1DC));
        (*fp).set_x2225_b1(1);
    }
}

// ---------------------------------------------------------------------------------------------
// Anim callbacks
// ---------------------------------------------------------------------------------------------

/// `ftKb_SpecialN_SetHitlagCb` (static inline).
pub fn ftKb_SpecialN_SetHitlagCb(gobj: G, ms: i32, mf: u32) {
    melee_core::ct!("ftKb_SpecialN_SetHitlagCb");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ms, mf);
        (*fp).set_x2222_b2(1);
        set_effect_hitlag_callbacks(fp);
    }
}

/// `ftKb_SpecialN_SetThrowCb` (static inline).
pub fn ftKb_SpecialN_SetThrowCb(
    gobj: G,
    mask: i16,
    capture_cb: HSD_GObjEvent,
    release_cb: HSD_GObjEvent,
    throw_cb: HSD_GObjInteraction,
) {
    melee_core::ct!("ftKb_SpecialN_SetThrowCb");
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007E2D0(fp, mask, capture_cb, release_cb, throw_cb);
        (*fp).set_x2225_b1(1);
    }
}

unsafe fn inhale_anim(gobj: G, ms: i32, capture_cb: unsafe extern "C" fn(G), release_cb: unsafe extern "C" fn(G)) {
    let fp = fpp(gobj);
    let joint = (*(*fp).parts.add(FtPart_YRotN as usize)).joint as *mut core::ffi::c_void;
    if (*fp).cmd_vars[0] != 0 {
        let q = (&raw mut (*fp).x60C) as *mut *mut core::ffi::c_void as *mut core::ffi::c_void;
        let facing = &mut (*fp).facing_dir as *mut f32 as *mut core::ffi::c_void;
        ext::efAsync_Spawn(gobj, q, 3, 0x49A, &[joint, facing]);
        (*fp).set_x2219_b0(1);
        set_effect_hitlag_callbacks(fp);
        (*fp).cmd_vars[0] = 0;
    }
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftKb_SpecialN_SetHitlagCb(gobj, ms, 0x212);
        ftKb_SpecialN_SetThrowCb(gobj, 0x10, Some(capture_cb), Some(release_cb), Some(cb_ftCo_800BD1DC));
    }
}

pub fn ftKb_SpecialN_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_Anim");
    unsafe { inhale_anim(gobj, 0x162, cb_fn_800F6210, cb_fn_800F6178) }
}

pub fn ftKb_SpecialAirN_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirN_Anim");
    unsafe { inhale_anim(gobj, 0x174, cb_fn_800F6318, cb_fn_800F6280) }
}

pub fn ftKb_SpecialNLoop_Anim(_gobj: G) { melee_core::ct!("ftKb_SpecialNLoop_Anim");}
pub fn ftKb_SpecialAirNLoop_Anim(_gobj: G) { melee_core::ct!("ftKb_SpecialAirNLoop_Anim");}

/// `ftKb_SpecialNCapture_EnterState` (static inline).
pub fn ftKb_SpecialNCapture_EnterState(gobj: G, ms: i32) {
    melee_core::ct!("ftKb_SpecialNCapture_EnterState");
    unsafe {
        let fp = fpp(gobj);
        change_ms(gobj, ms, 0x10);
        ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    }
}

pub fn ftKb_SpecialNCapture0_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNCapture0_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let pos = inhale_pos(fp, da);
        if ext::it_802F23AC((*fp).target_item_gobj, &pos) < (da.specialn_inhale_velocity * da.specialn_inhale_velocity) as f64 {
            ext::it_802F2810((*fp).target_item_gobj);
            if (*fp).ground_or_air == GA_Air {
                ftKb_SpecialNCapture_EnterState(gobj, 0x178);
            } else {
                ftKb_SpecialNCapture_EnterState(gobj, 0x166);
            }
        }
    }
}

pub fn ftKb_SpecialNCapture_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNCapture_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = &*da_of(fp);
        let pos = inhale_pos(fp, da);
        if ext::ftCo_800BD19C((*fp).victim_gobj, &pos) < da.specialn_inhale_velocity * da.specialn_inhale_velocity {
            ext::ftCo_800BD620((*fp).victim_gobj);
            if (*fp).ground_or_air == GA_Air {
                ftKb_SpecialNCapture_EnterState(gobj, 0x178);
            } else {
                ftKb_SpecialNCapture_EnterState(gobj, 0x166);
            }
        }
    }
}

pub fn ftKb_SpecialNCapture1_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNCapture1_Anim");
    ftKb_SpecialN_800F5DE8(gobj);
}

pub fn ftKb_SpecialAirNCapture_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNCapture_Anim");
    ftKb_SpecialN_800F5EA8(gobj);
}

pub fn ftKb_SpecialAirNCaptured_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNCaptured_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            st::ftCommon_8007D5D4(&mut *fp);
            change_ms(gobj, ftKb_MS_EatFall as i32, 0x90);
            ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        }
    }
}

pub fn ftKb_EatWait_Anim(_gobj: G) { melee_core::ct!("ftKb_EatWait_Anim");}
pub fn ftKb_SpecialAirNCaptureWait_Anim(_gobj: G) { melee_core::ct!("ftKb_SpecialAirNCaptureWait_Anim");}

/// `ftKb_SpecialNSpit_Anim_inline` (static inline).
pub fn ftKb_SpecialNSpit_Anim_inline(gobj: G, victim: G) {
    melee_core::ct!("ftKb_SpecialNSpit_Anim_inline");
    unsafe {
        if !victim.is_null() {
            let fp = fpp(gobj);
            let victim_fp = (*victim).user_data as *mut Fighter;
            st::ftCommon_8007E2F4(&mut *fp, 0);
            ext::ftCo_800DE2CC(gobj, victim);
            ext::ftCo_800BDB58(victim, gobj);
            ext::ftColl_8007B8CC(victim_fp, gobj);
            (*fp).cmd_vars[0] = 0;
        }
    }
}

/// Item spit shared by `ftKb_SpecialNSpit0_Anim` / `Spit1_Anim`.
unsafe fn spit_item(gobj: G, fp: *mut Fighter) {
    let item_gobj = (*fp).target_item_gobj;
    if (*fp).cmd_vars[0] != 0 && !item_gobj.is_null() {
        st::ftCommon_8007E2F4(&mut *fp, 0);
        let mut attr: itUnk2_DatAttrs = core::mem::zeroed();
        let bone = melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp, FtPart_TransN2 as i32);
        ext::lb_8000B1CC((*(*fp).parts.add(bone as usize)).joint as *mut _, core::ptr::null(), &mut attr.pos);
        let neg_facing = -(*fp).facing_dir;
        let da = &*da_of(fpp(gobj));
        attr.vel.x = -neg_facing * da.specialn_ground_spit_initial_horizontal_velocity;
        attr.vel.z = 0.0;
        attr.vel.y = 0.0;
        attr.float2 = da.specialn_spit_deceleration_rate;
        attr.float1 = da.specialn_star_base_duration;
        let r = ext::it_802F295C(gobj, &mut attr);
        ext::it_802F28C8(item_gobj, 0, r);
        (*fp).x1A64 = core::ptr::null_mut();
        (*fp).target_item_gobj = core::ptr::null_mut();
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftKb_SpecialNSpit0_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSpit0_Anim");
    unsafe {
        let fp = fpp(gobj);
        spit_item(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_SpecialNSpit_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSpit_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            ftKb_SpecialNSpit_Anim_inline(gobj, (*fp).victim_gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_SpecialNSpit1_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNSpit1_Anim");
    unsafe {
        let fp = fpp(gobj);
        spit_item(gobj, fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_SpecialAirNSpit_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNSpit_Anim");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).cmd_vars[0] != 0 {
            ftKb_SpecialNSpit_Anim_inline(gobj, (*fp).victim_gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// Item swallow shared by `ftKb_SpecialNDrink0_Anim` / `Drink1_Anim`.
unsafe fn drink_item(fp: *mut Fighter) {
    let item_gobj = (*fp).target_item_gobj;
    if (*fp).cmd_vars[0] != 0 && !item_gobj.is_null() {
        st::ftCommon_8007E2F4(&mut *fp, 0);
        fh::play_sfx(&mut *fp, 0x222F6, 0x7F, 0x40);
        ext::it_802F28C8(item_gobj, 0, 0.0);
        (*fp).x1A64 = core::ptr::null_mut();
        (*fp).target_item_gobj = core::ptr::null_mut();
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftKb_SpecialNDrink0_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNDrink0_Anim");
    unsafe {
        drink_item(fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftKb_SpecialNDrink_Anim_inline` (static inline).
pub fn ftKb_SpecialNDrink_Anim_inline(gobj: G, victim_gobj: G) {
    melee_core::ct!("ftKb_SpecialNDrink_Anim_inline");
    unsafe {
        let fp = fpp(gobj);
        (*fp).u.kb.xE0 = ext::ftCo_800BD9E0(gobj, victim_gobj);
        ext::ftKb_SpecialN_800F1BAC(gobj, (*fp).u.kb.xE0, true);
    }
}

unsafe fn drink_fighter(gobj: G, fp: *mut Fighter) {
    if (*fp).cmd_vars[0] != 0 {
        let victim_gobj = (*fp).victim_gobj;
        if !victim_gobj.is_null() {
            st::ftCommon_8007E2F4(&mut *fp, 0);
            ext::ftCo_800DE2CC(gobj, victim_gobj);
            ext::ftCo_800BE000(victim_gobj, gobj);
            ftKb_SpecialNDrink_Anim_inline(gobj, victim_gobj);
            (*fp).cmd_vars[0] = 0;
        }
    }
}

pub fn ftKb_SpecialNDrink_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNDrink_Anim");
    unsafe {
        drink_fighter(gobj, fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_SpecialNDrink1_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialNDrink1_Anim");
    unsafe {
        drink_item(fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_SpecialAirNDrink_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNDrink_Anim");
    unsafe {
        drink_fighter(gobj, fpp(gobj));
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_EatTurn_Anim(gobj: G) {
    melee_core::ct!("ftKb_EatTurn_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).facing_dir = -(*fp).facing_dir;
            let fp = fpp(gobj);
            change_ms(gobj, ftKb_MS_EatWait as i32, Ft_MF_SkipModel as u32);
            ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        }
    }
}

pub fn ftKb_SpecialAirNCaptureTurn_Anim(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNCaptureTurn_Anim");
    unsafe {
        let fp = fpp(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            (*fp).facing_dir = -(*fp).facing_dir;
            st::ftCommon_8007D5D4(&mut *fp);
            change_ms(gobj, ftKb_MS_EatFall as i32, 0x90);
            ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        }
    }
}

pub fn ftKb_EatWalk_Anim(gobj: G) {
    melee_core::ct!("ftKb_EatWalk_Anim");
    ext::ftWalkCommon_800DFDDC(gobj);
}

pub fn ftKb_EatJump1_Anim(gobj: G) {
    melee_core::ct!("ftKb_EatJump1_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = fpp(gobj);
            let da = &*da_of(fp);
            st::ftCommon_8007D5D4(&mut *fp);
            change_ms(gobj, ftKb_MS_EatJump2 as i32, 0x92);
            ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            ext::ftCo_800CB110(gobj, true, da.specialn_jump_height);
        }
    }
}

pub fn ftKb_EatJump2_Anim(_gobj: G) { melee_core::ct!("ftKb_EatJump2_Anim");}

// ---------------------------------------------------------------------------------------------
// IASA callbacks
// ---------------------------------------------------------------------------------------------

pub fn ftKb_SpecialNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.xE4 != 0 {
            (*fp).u.kb.xE4 -= 1;
            return;
        }
        if ((*fp).input.held_buttons[0] & 0x200) == 0 {
            change_ms(gobj, ftKb_MS_SpecialNEnd as i32, 0);
        }
    }
}

pub fn ftKb_SpecialAirNLoop_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNLoop_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.xE4 != 0 {
            (*fp).u.kb.xE4 -= 1;
            return;
        }
        if ((*fp).input.held_buttons[0] & 0x200) == 0 {
            change_ms(gobj, ftKb_MS_SpecialAirNEnd as i32, 0);
        }
    }
}

/// Enter an Eat-state change from a Wait IASA: motion change, hitlag-less bookkeeping.
unsafe fn eat_wait_change(gobj: G, ms: i32, flags: u32, set_b2: bool) -> bool {
    let fp = fpp(gobj);
    change_ms(gobj, ms, flags);
    if set_b2 {
        (*fp).set_x2222_b2(1);
    }
    ftKb_SpecialN_800F9070(gobj);
    st::ftAnim_8006EBA4(gobj);
    st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
    true
}

unsafe fn wait_eat(gobj: G, target: *mut HSD_GObj, ms: i32) -> bool {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    if (((*fp).input.pressed_buttons & 0x200) != 0 && !target.is_null())
        || (((*fp).input.lstick[0].y < -da.specialn_y_axis_range_jump) && !target.is_null())
    {
        return eat_wait_change(gobj, ms, 2, true);
    }
    false
}

unsafe fn wait_spit(gobj: G, target: *mut HSD_GObj, ms: i32) -> bool {
    let fp = fpp(gobj);
    if ((*fp).input.pressed_buttons & 0x100) != 0 && !target.is_null() {
        return eat_wait_change(gobj, ms, 0x12, true);
    }
    false
}

unsafe fn wait_turn(gobj: G, ms: i32) -> bool {
    let fp = fpp(gobj);
    let da = &*da_of(fp);
    let mut stick_x = (*fp).input.lstick[0].x;
    let abs_x = if stick_x < 0.0 { -stick_x } else { stick_x };
    if abs_x < da.specialn_x_axis_range_walk {
        stick_x = 0.0;
    }
    if (stick_x < 0.0 && (*fp).facing_dir == 1.0) || (stick_x > 0.0 && (*fp).facing_dir == -1.0) {
        return eat_wait_change(gobj, ms, 0x92, false);
    }
    false
}

pub fn ftKb_EatWait_ItemEat(gobj: G) -> bool {
    melee_core::ct!("ftKb_EatWait_ItemEat");
    unsafe { wait_eat(gobj, (*fpp(gobj)).target_item_gobj, 0x170) }
}
pub fn ftKb_EatWait_ItemSpit(gobj: G) -> bool {
    melee_core::ct!("ftKb_EatWait_ItemSpit");
    unsafe { wait_spit(gobj, (*fpp(gobj)).target_item_gobj, 0x172) }
}
pub fn ftKb_EatWait_FighterEat(gobj: G) -> bool {
    melee_core::ct!("ftKb_EatWait_FighterEat");
    unsafe { wait_eat(gobj, (*fpp(gobj)).victim_gobj, 0x16F) }
}
pub fn ftKb_EatWait_FighterSpit(gobj: G) -> bool {
    melee_core::ct!("ftKb_EatWait_FighterSpit");
    unsafe { wait_spit(gobj, (*fpp(gobj)).victim_gobj, 0x171) }
}
pub fn ftKb_EatWait_Turn(gobj: G) -> bool {
    melee_core::ct!("ftKb_EatWait_Turn");
    unsafe { wait_turn(gobj, 0x16B) }
}

pub fn ftKb_EatWait_IASA(gobj: G) {
    melee_core::ct!("ftKb_EatWait_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.xF4_b0() != 0 {
            if ftKb_EatWait_ItemEat(gobj) {
                return;
            }
            if ftKb_EatWait_ItemSpit(gobj) {
                return;
            }
        } else {
            if ftKb_EatWait_FighterEat(gobj) {
                return;
            }
            if ftKb_EatWait_FighterSpit(gobj) {
                return;
            }
        }
        if !ftKb_EatWait_Turn(gobj) {
            let jump = ext::ftCo_Jump_GetInput(gobj);
            let r0_2;
            if jump != 0 {
                let fp3 = fpp(gobj);
                (*fp3).mv.kb.specialhi.x4 = jump as i32;
                (*fp3).mv.kb.specialhi.x0 = 0;
                change_ms(gobj, 0x16C, 0x92);
                ftKb_SpecialN_800F9070(gobj);
                st::ftAnim_8006EBA4(gobj);
                st::ftCommon_8007E2F4(&mut *fp3, 0x1FF);
                r0_2 = 1;
            } else {
                r0_2 = 0;
            }
            if r0_2 == 0 && ext::ftWalkCommon_800DFC70(gobj) {
                let fp4 = fpp(gobj);
                ext::ftWalkCommon_800DFCA4(
                    gobj,
                    0x168,
                    0x10,
                    0.0,
                    (*fp4).u.kb.xE8,
                    (*fp4).u.kb.xEC,
                    (*fp4).u.kb.xF0,
                    (*fp4).co_attrs.slow_walk_max,
                    (*fp4).co_attrs.mid_walk_point,
                    (*fp4).co_attrs.fast_walk_min,
                    (*da_of(fp4)).specialn_walk_speed,
                );
                st::ftCommon_8007E2F4(&mut *fp4, 0x1FF);
            }
        }
    }
}

pub fn ftKb_SpecialAirNCaptureWait_ItemEat(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_ItemEat");
    unsafe { wait_eat(gobj, (*fpp(gobj)).target_item_gobj, 0x17B) }
}
pub fn ftKb_SpecialAirNCaptureWait_ItemSpit(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_ItemSpit");
    unsafe { wait_spit(gobj, (*fpp(gobj)).target_item_gobj, 0x17D) }
}
pub fn ftKb_SpecialAirNCaptureWait_FighterEat(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_FighterEat");
    unsafe { wait_eat(gobj, (*fpp(gobj)).victim_gobj, 0x17A) }
}
pub fn ftKb_SpecialAirNCaptureWait_FighterSpit(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_FighterSpit");
    unsafe { wait_spit(gobj, (*fpp(gobj)).victim_gobj, 0x17C) }
}
pub fn ftKb_SpecialAirNCaptureWait_Turn(gobj: G) -> bool {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_Turn");
    unsafe { wait_turn(gobj, 0x17E) }
}

pub fn ftKb_SpecialAirNCaptureWait_IASA(gobj: G) {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.kb.xF4_b0() != 0 {
            if ftKb_SpecialAirNCaptureWait_ItemEat(gobj) {
                return;
            }
            if ftKb_SpecialAirNCaptureWait_ItemSpit(gobj) {
                return;
            }
        } else {
            if ftKb_SpecialAirNCaptureWait_FighterEat(gobj) {
                return;
            }
            if ftKb_SpecialAirNCaptureWait_FighterSpit(gobj) {
                return;
            }
        }
        let _ = ftKb_SpecialAirNCaptureWait_Turn(gobj);
    }
}

pub fn ftKb_EatWalk_IASA(gobj: G) {
    melee_core::ct!("ftKb_EatWalk_IASA");
    unsafe {
        let jump_input = ext::ftCo_Jump_GetInput(gobj);
        let var;
        if jump_input != 0 {
            let fp = fpp(gobj);
            (*fp).mv.kb.specialhi.x4 = jump_input as i32;
            (*fp).mv.kb.specialhi.x0 = 0;
            change_ms(gobj, ftKb_MS_EatJump1 as i32, (Ft_MF_KeepGfx | Ft_MF_SkipModel | Ft_MF_SkipMatAnim) as u32);
            ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
            var = 1;
        } else {
            var = 0;
        }
        if var == 0 {
            if ext::ft_8008A1FC(gobj) {
                let fp = fpp(gobj);
                change_ms(gobj, ftKb_MS_EatWait as i32, Ft_MF_SkipModel as u32);
                ftKb_SpecialN_800F9070(gobj);
                st::ftAnim_8006EBA4(gobj);
                st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
                return;
            }
            ext::ftWalkCommon_800DFEC8(gobj, fn_800F64C8);
        }
    }
}

pub fn ftKb_EatJump1_IASA(gobj: G) {
    melee_core::ct!("ftKb_EatJump1_IASA");
    ext::ftCo_KneeBend_Check_ShortHop(gobj);
}

// ---------------------------------------------------------------------------------------------
// Phys
// ---------------------------------------------------------------------------------------------

pub fn ftKb_EatWalk_Phys(gobj: G) {
    melee_core::ct!("ftKb_EatWalk_Phys");
    ext::ftWalkCommon_800E0060(gobj);
}

pub fn ftKb_EatJump2_Phys(gobj: G) {
    melee_core::ct!("ftKb_EatJump2_Phys");
    ext::ftCo_Jump_Phys_Inner(gobj);
}

/// `ftKb_SpecialN_800F9090`: release the swallowed item when Kirby (without a hat) is hit.
pub fn ftKb_SpecialN_800F9090(gobj: G) {
    melee_core::ct!("ftKb_SpecialN_800F9090");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).target_item_gobj.is_null()
            && FTKB_IS_KIRBY((*fp).kind as i32)
            && (*fp).u.kb.hat.kind as i32 == Ft_Kind_Kirby as i32
        {
            let mut capturing = 0;
            if (*fp).motion_id == ftKb_MS_SpecialNCapture1 as FtMotionId
                || (*fp).motion_id == ftKb_MS_SpecialAirNCapture1 as FtMotionId
            {
                capturing = 1;
            }
            ext::it_802F28C8((*fp).target_item_gobj, capturing, 0.0);
            (*fp).x1A64 = core::ptr::null_mut();
            (*fp).target_item_gobj = core::ptr::null_mut();
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `fn_800F6848` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6848(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6848");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 373 /* ftKb_MS_SpecialAirNEnd */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
}

/// `fn_800F68A8` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F68A8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F68A8");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_AirToGroundStateChange(gobj, fp, 355 /* ftKb_MS_SpecialNEnd */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
}

/// `ftKb_SpecialNEnd_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftKb_SpecialAirNEnd_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNEnd_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftKb_SpecialN_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirN_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirN_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirN_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirNLoop_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirNEnd_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialNCapture0_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNCapture_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNCapture1_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture1_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialAirNCapture_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCapture_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCapture_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_Eat_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_Eat_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_Eat_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirNCaptured_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptured_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptured_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_EatWait_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatWait_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirNCaptureWait_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptureWait_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialNSpit0_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNSpit_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNSpit1_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit1_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialAirNSpit_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNSpit_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNSpit_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialNDrink0_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink0_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink0_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNDrink_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialNDrink1_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink1_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_SpecialAirNDrink_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNDrink_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNDrink_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_EatTurn_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatTurn_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatTurn_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_SpecialAirNCaptureTurn_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptureTurn_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptureTurn_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftKb_EatJump1_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatJump1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatJump1_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftKb_EatLanding_Phys` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatLanding_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatLanding_Phys");
    st::ft_80084F3C(gobj);
}

/// `fn_800F6528` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6528(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6528");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 366 /* ftKb_MS_EatLanding */, 0x12u32 /* 0x12 */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `fn_800F6FC4` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6FC4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6FC4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 382 /* ftKb_MS_EatTurnAir */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
    }
}

/// `ftKb_SpecialN_800F9070` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialN_800F9070(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_800F9070");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).death3_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
        (*fp).death1_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
    }
}

/// `ftKb_SpecialN_800F6388` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialN_800F6388(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_800F6388");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 358 /* ftKb_MS_Eat */, 0x10u32 /* 0x10 */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `ftKb_SpecialN_800F63EC` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialN_800F63EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_800F63EC");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 376 /* ftKb_MS_EatAir */, 0x10u32 /* 0x10 */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6450` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6450(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6450");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 377 /* ftKb_MS_EatFall */, 0x90u32 /* 0x90 */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftAnim_8006EBA4(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6908` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6908(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6908");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 375 /* ftKb_MS_SpecialAirNCapture1 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F697C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F697C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F697C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 374 /* ftKb_MS_SpecialAirNCapture0 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F69E8` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F69E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F69E8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 357 /* ftKb_MS_SpecialNCapture1 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6A5C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6A5C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6A5C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 356 /* ftKb_MS_SpecialNCapture0 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6AC8` (static) (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6AC8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6AC8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 376 /* ftKb_MS_EatAir */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6B3C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6B3C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6B3C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 358 /* ftKb_MS_Eat */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `ftKb_SpecialS_800F6BB0` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialS_800F6BB0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialS_800F6BB0");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 377 /* ftKb_MS_EatFall */, 0xC4C5090u32 /* ftKb_MF_SpecialN_EatFall_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6C24` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6C24(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6C24");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 381 /* ftKb_MS_SpecialAirNSpit1 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6C98` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6C98(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6C98");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 380 /* ftKb_MS_SpecialAirNSpit0 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6D0C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6D0C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6D0C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 370 /* ftKb_MS_SpecialNSpit1 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6D80` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6D80(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6D80");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 369 /* ftKb_MS_SpecialNSpit0 */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6DF4` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6DF4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6DF4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 379 /* ftKb_MS_SpecialAirNDrink1 */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6E68` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6E68(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6E68");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 378 /* ftKb_MS_SpecialAirNDrink0 */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6EDC` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6EDC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6EDC");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 368 /* ftKb_MS_SpecialNDrink1 */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F6F50` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F6F50(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F6F50");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 367 /* ftKb_MS_SpecialNDrink0 */, 0xC4C5082u32 /* ftKb_MF_SpecialN_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `fn_800F702C` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn fn_800F702C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_800F702C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 363 /* ftKb_MS_EatTurn */, 0xC4C5092u32 /* ftKb_MF_SpecialN_Capture_Coll */);
        crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
    }
}

/// `ftKb_Eat_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_Eat_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_Eat_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::Fighter_ChangeMotionState(gobj, 359 /* ftKb_MS_EatWait */, 0x10u32 /* Ft_MF_SkipModel */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
        }
    }
}

/// `ftKb_EatLanding_Anim` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatLanding_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatLanding_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
            st::Fighter_ChangeMotionState(gobj, 359 /* ftKb_MS_EatWait */, 0x10u32 /* Ft_MF_SkipModel */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
            crate::chars::r3::ftKirby::ftkirbyspecialn::ftKb_SpecialN_800F9070(gobj);
            st::ftAnim_8006EBA4(gobj);
            st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (0x1FF) */ as i16));
        }
    }
}

/// `ftKb_SpecialN_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialN_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6588));
    }
}

/// `ftKb_SpecialAirN_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirN_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirN_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6638));
    }
}

/// `ftKb_SpecialNLoop_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNLoop_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F66E8));
    }
}

/// `ftKb_SpecialAirNLoop_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNLoop_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6798));
    }
}

/// `ftKb_SpecialNEnd_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNEnd_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6848));
    }
}

/// `ftKb_SpecialAirNEnd_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNEnd_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F68A8));
    }
}

/// `ftKb_SpecialNCapture0_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture0_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6908));
    }
}

/// `ftKb_SpecialNCapture_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F697C));
    }
}

/// `ftKb_SpecialNCapture1_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNCapture1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNCapture1_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F69E8));
    }
}

/// `ftKb_SpecialAirNCapture_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCapture_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCapture_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6A5C));
    }
}

/// `ftKb_Eat_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_Eat_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_Eat_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6AC8));
    }
}

/// `ftKb_SpecialAirNCaptured_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptured_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptured_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6B3C));
    }
}

/// `ftKb_EatWait_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatWait_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatWait_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__ftKb_SpecialS_800F6BB0));
    }
}

/// `ftKb_SpecialAirNCaptureWait_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptureWait_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptureWait_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6528));
    }
}

/// `ftKb_SpecialNSpit0_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit0_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6C24));
    }
}

/// `ftKb_SpecialNSpit_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6C98));
    }
}

/// `ftKb_SpecialNSpit1_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNSpit1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNSpit1_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6D0C));
    }
}

/// `ftKb_SpecialAirNSpit_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNSpit_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNSpit_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6D80));
    }
}

/// `ftKb_SpecialNDrink0_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink0_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink0_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6DF4));
    }
}

/// `ftKb_SpecialNDrink_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6E68));
    }
}

/// `ftKb_SpecialNDrink1_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialNDrink1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialNDrink1_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6EDC));
    }
}

/// `ftKb_SpecialAirNDrink_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNDrink_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNDrink_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6F50));
    }
}

/// `ftKb_EatTurn_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatTurn_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatTurn_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6FC4));
    }
}

/// `ftKb_SpecialAirNCaptureTurn_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_SpecialAirNCaptureTurn_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_SpecialAirNCaptureTurn_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F702C));
    }
}

/// `ftKb_EatWalk_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatWalk_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatWalk_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6450));
    }
}

/// `ftKb_EatJump1_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatJump1_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatJump1_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6450));
    }
}

/// `ftKb_EatJump2_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatJump2_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatJump2_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6528));
    }
}

/// `ftKb_EatLanding_Coll` (src/melee/ft/kinds/ftKirby/ftkirbyspecialn.c).
pub fn ftKb_EatLanding_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftKb_EatLanding_Coll");
    unsafe {
        st::ft_8008403C(gobj, Some(crate::chars::cb::ftkirbyspecialn__fn_800F6450));
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// Spit: the victim's launch velocity; returns the deceleration rate.
pub fn ftKb_SpecialN_800F58AC_fp(fp: &Fighter, victim_self_vel: &mut Vec3, victim_facing_dir: f32) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F58AC_fp");
    let a: &ftKb_DatAttrs = da(fp);
    victim_self_vel.x = -victim_facing_dir * a.specialn_ground_spit_initial_horizontal_velocity;
    victim_self_vel.z = 0.0;
    victim_self_vel.y = 0.0;
    a.specialn_spit_deceleration_rate
}

/// Returns `coll_data.floor.flags & 0x100` (an int, not 0/1: MSL bool is int).
pub fn ftKb_SpecialN_800F597C_fp(fp: &Fighter) -> i32 {
    melee_core::ct!("ftKb_SpecialN_800F597C_fp");
    (fp.coll_data.floor.flags & 0x100) as i32
}

pub fn ftKb_SpecialN_800F5A04_fp(fp: &mut Fighter, arg1: f32) {
    melee_core::ct!("ftKb_SpecialN_800F5A04_fp");
    let a: &ftKb_DatAttrs = da(fp);
    if fp.ground_or_air == 0 {
        fp.gr_vel = arg1 * a.specialn_stop_momentum;
        return;
    }
    fp.self_vel.x = arg1 * a.specialn_stop_momentum;
}

pub fn ftKb_SpecialN_800F5A38_fp(fp: &Fighter, _victim_fp: &Fighter) -> bool {
    melee_core::ct!("ftKb_SpecialN_800F5A38_fp");
    let id = fp.motion_id as u32;
    id == ftKb_MS_EatWait as u32 || id == ftKb_MS_EatFall as u32
}

pub fn ftKb_SpecialN_800F5A60_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A60_fp");
    -fp.facing_dir
}

pub fn ftKb_SpecialN_800F5A88_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5A88_fp");
    da::<ftKb_DatAttrs>(fp).specialn_star_base_duration
}

pub fn ftKb_SpecialN_800F5AB0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AB0_fp");
    da::<ftKb_DatAttrs>(fp).specialn_frames_in_swallow_star
}

pub fn ftKb_SpecialN_800F5AF0_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5AF0_fp");
    da::<ftKb_DatAttrs>(fp).specialn_base_duration
}

pub fn ftKb_SpecialN_800F5B00_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B00_fp");
    da::<ftKb_DatAttrs>(fp).specialn_duration_divisor
}

pub fn ftKb_SpecialN_800F5B10_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B10_fp");
    da::<ftKb_DatAttrs>(fp).specialn_inhale_resistance
}

pub fn ftKb_SpecialN_800F5B20_fp(fp: &Fighter, out: &mut Vec2) {
    melee_core::ct!("ftKb_SpecialN_800F5B20_fp");
    let a: &ftKb_DatAttrs = da(fp);
    out.x = a.specialn_velocity_outer_grab_box;
    out.y = a.specialn_velocity_inner_grab_box;
}

pub fn ftKb_SpecialN_800F5B3C_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B3C_fp");
    da::<ftKb_DatAttrs>(fp).specialn_gravity_of_inhaled
}

pub fn ftKb_SpecialN_800F5B4C_fp(fp: &Fighter) -> f32 {
    melee_core::ct!("ftKb_SpecialN_800F5B4C_fp");
    da::<ftKb_DatAttrs>(fp).specialn_z_offset_inhaled
}

/// Where an inhaled fighter sits. `output->x += off_x * facing_dir` is one fused site in
/// the shipping build (fmadds); the y add is a plain f32 add.
pub fn ftKb_SpecialN_800F5B5C_fp(fp: &Fighter, output: &mut Vec3) {
    melee_core::ct!("ftKb_SpecialN_800F5B5C_fp");
    let a: &ftKb_DatAttrs = da(fp);
    *output = fp.cur_pos;
    output.x = fmadds(a.specialn_x_offset_inhaled, fp.facing_dir, output.x);
    output.y += a.specialn_y_offset_inhaled;
}
