//! src/melee/ft/kinds/ftPopo/ftpopospecials.c: Ice Climbers' side-B (Squall Hammer). Two fused
//! sites, both `xE4_ground_accel_1 += x6C * floor.normal.x` (an fmadds) in ftPp_SpecialS1_Phys and
//! ftPp_SpecialS2_Phys (tools/rust-port/fma_sites.py). `ABS` is `x < 0 ? -x : x` (keeps -0.0).
//! Not smoke-tested yet (hooks-heavy; only compiled).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftpopo::mf;
use melee_core::fp::fmadds;
use melee_fighter::ftcommon as co;
use melee_types::*;

/// Already ported (rounds 3/5): re-exported under this TU.
use crate::chars::ftPopo::ftpopospecials as hand;

/// Gobj-signature wrappers over the hand ports in `chars::ftPopo` (those take the fighter).
pub fn ftPp_SpecialS1_IASA(gobj: G) {
    unsafe { hand::ftPp_SpecialS1_IASA(&mut *fpp(gobj)) }
}

pub fn ftPp_SpecialAirS1_IASA(gobj: G) {
    unsafe { hand::ftPp_SpecialAirS1_IASA(&mut *fpp(gobj)) }
}

pub fn ftPp_SpecialS_8011F964(gobj: G) -> i32 {
    unsafe { hand::ftPp_SpecialS_8011F964(&*fpp(gobj)) as i32 }
}

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
    pub static mut PLAYER_GETENTITYATINDEX: Option<fn(i32, i32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn Player_GetEntityAtIndex(slot: i32, index: i32) -> *mut HSD_GObj {
                match unsafe { PLAYER_GETENTITYATINDEX } {
                    Some(f) => f(slot, index),
                    None => { return unsafe { melee_fighter::tu::player::Player_GetEntityAtIndex(slot as _, index as _) as _ }; },
                }
            }
    pub static mut FTNN_INIT_80123954: Option<fn(*mut HSD_GObj, u32) -> u32> = None;
            #[allow(non_snake_case)]
            pub fn ftNn_Init_80123954(gobj: *mut HSD_GObj, ground_or_air: u32) -> u32 {
                match unsafe { FTNN_INIT_80123954 } {
                    Some(f) => f(gobj, ground_or_air),
                    None => { return unsafe { crate::tu::ftnanaspecials::ftNn_Init_80123954(gobj as _, ground_or_air as _) as _ }; },
                }
            }
    pub static mut FTNN_INIT_80123B10: Option<fn(*mut HSD_GObj) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ftNn_Init_80123B10(gobj: *mut HSD_GObj) -> i32 {
                match unsafe { FTNN_INIT_80123B10 } {
                    Some(f) => f(gobj),
                    None => { return unsafe { crate::tu::ftnanaspecials::ftNn_Init_80123B10(gobj as _) as _ }; },
                }
            }
    pub static mut FT_80082888: Option<fn(*mut HSD_GObj, *const ftCollisionBox) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ft_80082888(gobj: *mut HSD_GObj, cb: *const ftCollisionBox) -> i32 {
                match unsafe { FT_80082888 } {
                    Some(f) => f(gobj, cb),
                    None => { return unsafe { melee_fighter::tu::ft_081b::ft_80082888(gobj as _, cb as _) as _ }; },
                }
            }
    pub static mut FT_800824A0: Option<fn(*mut HSD_GObj, *const ftCollisionBox) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn ft_800824A0(gobj: *mut HSD_GObj, cb: *const ftCollisionBox) -> i32 {
                match unsafe { FT_800824A0 } {
                    Some(f) => f(gobj, cb),
                    None => { return unsafe { melee_fighter::tu::ft_081b::ft_800824a0(gobj as _, cb as _) as _ }; },
                }
            }
    pub static mut HSD_JOBJGETCHILD: Option<fn(*mut HSD_JObj) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJGETCHILD } {
                    Some(f) => f(jobj),
                    None => { return unsafe { melee_items::tu::kinds_inl::HSD_JObjGetChild(jobj as _) as _ }; },
                }
            }
    pub static mut HSD_JOBJGETNEXT: Option<fn(*mut HSD_JObj) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
        match unsafe { HSD_JOBJGETNEXT } {
            Some(f) => f(jobj),
            None => { unsafe { melee_world::tu::grinl::HSD_JObjGetNext(jobj as _) as _ } },
        }
    }
    hook!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (jobj: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (jobj: *mut HSD_JObj, flags: u32));
    hook!(HSD_JOBJSETROTATIONZ, HSD_JObjSetRotationZ, (jobj: *mut HSD_JObj, z: f32));
    /// Recording hook; unset, the native `melee_fighter::tu::ftparts` port runs.
    pub static mut FTPARTSETROTX: Option<fn(*mut Fighter, i32, f32)> = None;
    pub fn ftPartSetRotX(fp: *mut Fighter, part: i32, v: f32) {
        match unsafe { FTPARTSETROTX } {
            Some(f) => f(fp, part, v),
            None => melee_fighter::tu::ftparts::ftPartSetRotX(unsafe { &*fp }, part, v),
        }
    }
}

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpp(gobj: G) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftIceClimberAttributes {
    (*fp).dat_attrs as *const ftIceClimberAttributes
}
/// C `ABS` on floats: `x < 0 ? -x : x` (so -0.0 stays -0.0).
#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `Fighter_SetEffectHitlagCallbacks` (ft/inlines.h).
#[inline(always)]
unsafe fn set_effect_hitlag_callbacks(fp: *mut Fighter) {
    (*fp).pre_hitlag_cb = Some(crate::chars::cb::ext__efLib_PauseAll);
    (*fp).post_hitlag_cb = Some(crate::chars::cb::ext__efLib_ResumeAll);
}

/// `ftNn_Init_803CD820`: the Squall Hammer collision box.
pub static ftNn_Init_803CD820: ftCollisionBox = ftCollisionBox {
    top: 12.0,
    bottom: 0.0,
    left: Vec2 { x: -6.0, y: 6.0 },
    right: Vec2 { x: 6.0, y: 6.0 },
};

/// `setRefGObjFlagAndClear` (static inline).
unsafe fn setRefGObjFlagAndClear(fp: *mut Fighter) {
    let gobj = (*fp).x1A5C;
    if !gobj.is_null() {
        let ref_fp = fpp(gobj);
        st::Fighter_UnkSetFlag_8006CFBC(gobj);
        (*ref_fp).x1A5C = core::ptr::null_mut();
    }
    (*fp).x1A5C = core::ptr::null_mut();
}

pub fn ftPp_SpecialS_8011F68C(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftPartSetRotX(fp, 0, 0.0);
        st::Fighter_UnkSetFlag_8006CFBC(gobj);
        setRefGObjFlagAndClear(fp);
    }
}

pub fn ftPp_SpecialS_8011F6FC(gobj: G) -> i32 {
    unsafe {
        if !gobj.is_null() {
            let fp = fpp(gobj);
            if !fp.is_null() {
                return (*fp).cmd_vars[1] as i32;
            }
        }
        1
    }
}

pub fn ftPp_SpecialS_8011F720(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).mv.pp.specials.x8.is_null() {
            if (*fp).x2219_b0() != 0 && !(*fp).mv.pp.specials.x8.is_null() {
                let temp_r3_2 = (*(*fp).mv.pp.specials.x8).x4;
                if !temp_r3_2.is_null() {
                    let var_r0 = ext::HSD_JObjGetNext(ext::HSD_JObjGetChild((*temp_r3_2).hsd_obj as *mut HSD_JObj));
                    let temp_r30 = (*temp_r3_2).hsd_obj as *mut HSD_JObj;
                    if (*fp).motion_id == ftPp_MS_SpecialS1 as _ || (*fp).motion_id == ftPp_MS_SpecialS2 as _ {
                        ext::HSD_JObjClearFlagsAll(var_r0, JOBJ_HIDDEN as u32);
                    } else {
                        ext::HSD_JObjSetFlagsAll(var_r0, JOBJ_HIDDEN as u32);
                    }
                    if (*fp).cmd_vars[3] != 0 && (*fp).mv.pp.specials.xC != 0 {
                        let angle = -melee_runtime::lbtrigf::atan2f(
                            (*fp).coll_data.floor.normal.x,
                            (*fp).coll_data.floor.normal.y,
                        );
                        ext::HSD_JObjSetRotationZ(temp_r30, angle);
                    } else {
                        ext::HSD_JObjSetRotationZ(temp_r30, 0.0);
                    }
                    set_effect_hitlag_callbacks(fp);
                    return;
                }
            }
            (*fp).mv.pp.specials.x8 = core::ptr::null_mut();
            (*fp).set_x2219_b0(0);
            (*fp).pre_hitlag_cb = None;
            (*fp).post_hitlag_cb = None;
        }
    }
}

/// `inlineA0` (static inline).
unsafe fn inlineA0(gobj: G) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
    (*fp).death2_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
    set_effect_hitlag_callbacks(fp);
}

/// `inlineA1` (static inline).
unsafe fn inlineA1(gobj: G) {
    let fp = fpp(gobj);
    (*fp).mv.pp.specials.x8 = core::ptr::null_mut();
    (*fp).set_x2219_b0(1);
    set_effect_hitlag_callbacks(fp);
}

/// `inlineA2` (static inline).
unsafe fn inlineA2(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    st::Fighter_ClearCmdVars(&mut *fp);
    (*fp).mv.pp.specials.x0 = 0.0;
    (*fp).mv.pp.specials.x4 = 10;
    (*fp).mv.pp.specials.xC = 0;
    (*fp).mv.pp.specials.x8 = core::ptr::null_mut();
    (*fp).mv.pp.specials.x10 = 0;
    (*fp).mv.pp.specials.x14 = (*da).x68;
    (*fp).mv.pp.specials.x18 = 0;
    (*fp).mv.pp.specials.x1C = 0.0;
    if ext::ftNn_Init_80123954(ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1), (*fp).ground_or_air)
        == GA_Air
    {
        st::Fighter_ChangeMotionState(gobj, ftPp_MS_SpecialS1 as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x1A5C = core::ptr::null_mut();
    } else {
        st::Fighter_ChangeMotionState(gobj, ftPp_MS_SpecialS2 as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x1A5C = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);
    }
    (*fp).self_vel.y = 0.0;
    {
        let x_vel = (*da).x28 * (*fp).facing_dir;
        (*fp).self_vel.x = x_vel;
        (*fp).gr_vel = x_vel;
    }
}

pub fn ftPp_SpecialS_Enter(gobj: G) {
    unsafe {
        inlineA2(gobj);
        inlineA0(gobj);
        st::ftAnim_8006EBA4(gobj);
        inlineA1(gobj);
        inlineA0(gobj);
    }
}

/// `inlineB0` (static inline).
unsafe fn inlineB0(gobj: G) {
    let fp = fpp(gobj);
    let da = da(fp);
    st::Fighter_ClearCmdVars(&mut *fp);
    (*fp).mv.pp.specials.x0 = 0.0;
    (*fp).mv.pp.specials.x4 = 10;
    (*fp).mv.pp.specials.xC = 0;
    (*fp).mv.pp.specials.x8 = core::ptr::null_mut();
    (*fp).mv.pp.specials.x10 = 0;
    (*fp).mv.pp.specials.x14 = (*da).x68;
    (*fp).mv.pp.specials.x18 = 0;
    (*fp).mv.pp.specials.x1C = 0.0;
    if ext::ftNn_Init_80123954(ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1), (*fp).ground_or_air)
        == GA_Air
    {
        st::Fighter_ChangeMotionState(gobj, ftPp_MS_SpecialAirS1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x1A5C = core::ptr::null_mut();
        (*fp).self_vel.y = (*da).x20;
    } else {
        st::Fighter_ChangeMotionState(gobj, ftPp_MS_SpecialAirS2 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).x1A5C = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);
        (*fp).self_vel.y = (*da).x24;
    }
    (*fp).self_vel.x = (*da).x2C * (*fp).facing_dir;
}

pub fn ftPp_SpecialAirS_Enter(gobj: G) {
    unsafe {
        inlineB0(gobj);
        inlineA0(gobj);
        st::ftAnim_8006EBA4(gobj);
        inlineA1(gobj);
        inlineA0(gobj);
    }
}

/// `resetAnim` (static inline).
unsafe fn resetAnim(gobj: G) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = None;
    (*fp).death2_cb = None;
    (*fp).pre_hitlag_cb = None;
    (*fp).post_hitlag_cb = None;
    (*fp).x1A5C = core::ptr::null_mut();
    st::ft_8008A2BC(gobj);
}

pub fn ftPp_SpecialS1_Anim(gobj: G) {
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            resetAnim(gobj);
        }
    }
}

/// `inlineC0` (static inline).
unsafe fn inlineC0(gobj: G) {
    let fp = fpp(gobj);
    st::Fighter_UnkSetFlag_8006CFBC(gobj);
    (*fp).x1A5C = core::ptr::null_mut();
}

/// `inlineC1` (static inline).
unsafe fn inlineC1(gobj: G) {
    let fp = fpp(gobj);
    ext::ftPartSetRotX(fp, 0, 0.0);
    st::Fighter_UnkSetFlag_8006CFBC(gobj);

    if !(*fp).x1A5C.is_null() {
        inlineC0((*fp).x1A5C);
    }
    (*fp).x1A5C = core::ptr::null_mut();
    resetAnim(gobj);
}

pub fn ftPp_SpecialS2_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let nn_gobj = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 || ext::ftNn_Init_80123B10(nn_gobj) != 0 {
            inlineC1(gobj);
        }
    }
}

/// `inline0` (static inline).
unsafe fn inline0(gobj: G) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = None;
    (*fp).death2_cb = None;
    (*fp).pre_hitlag_cb = None;
    (*fp).post_hitlag_cb = None;
    (*fp).x1A5C = core::ptr::null_mut();
}

pub fn ftPp_SpecialAirS1_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            inline0(gobj);
            if (*da).x70 == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCommon_8007D60C(&mut *fp);
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x70);
            }
        }
    }
}

/// `inline1` (static inline).
unsafe fn inline1(gobj: G) {
    let fp = fpp(gobj);
    ext::ftPartSetRotX(fp, 0, 0.0);
    st::Fighter_UnkSetFlag_8006CFBC(gobj);
    setRefGObjFlagAndClear(fp);
}

pub fn ftPp_SpecialAirS2_Anim(gobj: G) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        let temp_r30 = ext::Player_GetEntityAtIndex((*fp).player_id as i32, 1);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 || ext::ftNn_Init_80123B10(temp_r30) != 0 {
            inline1(gobj);
            inline0(gobj);
            if (*da).x70 == 0.0 {
                st::ftCo_Fall_Enter(gobj);
            } else {
                st::ftCommon_8007D60C(&mut *fp);
                st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x70);
            }
        }
    }
}

pub fn ftPp_SpecialS2_IASA(gobj: G) {
    ftPp_SpecialS1_IASA(gobj);
}

pub fn ftPp_SpecialAirS2_IASA(gobj: G) {
    ftPp_SpecialAirS1_IASA(gobj);
}

/// The ground Phys body shared by S1 and S2; `vy` selects `x60` (S1) or `x64` (S2) and `air_msid`
/// the air state entered when the jump is buffered.
unsafe fn ground_phys(gobj: G, air_msid: u32, vy_of: fn(*const ftIceClimberAttributes) -> f32) {
    let fp = fpp(gobj);
    let temp_r30 = da(fp);
    if (*fp).cmd_vars[0] == 0 && (*fp).mv.pp.specials.x1C != 0.0 {
        let target_vel = if (*fp).mv.pp.specials.x1C > 0.0 { (*temp_r30).x38 } else { -(*temp_r30).x38 };
        co::calc_ground_accel_accel_to_vel(&mut *fp, (*fp).mv.pp.specials.x1C, target_vel, (*fp).co_attrs.ground_friction);
        let temp_r5 = fpp(gobj);
        {
            let da = da(temp_r5);
            (*temp_r5).xE4_ground_accel_1 =
                fmadds((*da).x6C, (*temp_r5).coll_data.floor.normal.x, (*temp_r5).xE4_ground_accel_1);
        }
        co::clamp_ground_vel(&mut *fp, (*temp_r30).x38);
        co::set_self_movement_from_grounded_movement_no_friction(&mut *fp);
    } else {
        let friction = (*fp).co_attrs.ground_friction;
        co::calc_ground_accel_deaccel(&mut *fp, friction);
        co::clamp_ground_vel(&mut *fp, (*temp_r30).x38);
        co::set_self_movement_from_grounded_movement_no_friction(&mut *fp);
    }
    (*fp).mv.pp.specials.x14 += 1;
    if (*fp).cmd_vars[2] != 0 && ((*fp).input.pressed_buttons & HSD_PAD_B as u32) != 0 {
        (*fp).mv.pp.specials.x10 += 1;
    }
    if (*fp).mv.pp.specials.x10 != 0 && (*fp).mv.pp.specials.x14 > (*temp_r30).x68 {
        let temp_r27 = fpp(gobj);
        let temp_r28 = da(temp_r27);
        st::ftCommon_8007D5D4(&mut *temp_r27);
        (*temp_r27).x74_self_accel.x = (*temp_r27).xE4_ground_accel_1;
        st::Fighter_ChangeMotionState(
            gobj,
            air_msid as i32,
            0x0C4C528A,
            (*temp_r27).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        let temp_r5_2 = fpp(gobj);
        (*temp_r5_2).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
        (*temp_r5_2).death2_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
        set_effect_hitlag_callbacks(temp_r5_2);
        co::clamp_self_vel_x(&mut *temp_r27, (*temp_r28).x3C);
        (*fp).self_vel.y += vy_of(temp_r30);
        (*fp).mv.pp.specials.x10 = 0;
        (*fp).mv.pp.specials.x14 = 0;
    }
    (*fp).mv.pp.specials.x18 += 1;
}

pub fn ftPp_SpecialS1_Phys(gobj: G) {
    unsafe { ground_phys(gobj, ftPp_MS_SpecialAirS1 as u32, |d| (*d).x60) }
}

pub fn ftPp_SpecialS2_Phys(gobj: G) {
    unsafe { ground_phys(gobj, ftPp_MS_SpecialAirS2 as u32, |d| (*d).x64) }
}

/// The air Phys body shared by AirS1 and AirS2 (S1 uses x60 / x4C_gravity / x54_terminal_vel,
/// S2 uses x64 / x50_gravity / x58_terminal_vel).
unsafe fn air_phys(gobj: G, jump_vy: f32, gravity: f32, terminal_vel: f32) {
    let fp = fpp(gobj);
    let da = da(fp);
    (*fp).mv.pp.specials.x14 += 1;
    if (*fp).cmd_vars[2] != 0 && ((*fp).input.pressed_buttons & HSD_PAD_B as u32) != 0 {
        (*fp).mv.pp.specials.x10 += 1;
    }
    if (*fp).mv.pp.specials.x10 != 0 && (*fp).mv.pp.specials.x14 > (*da).x68 {
        (*fp).self_vel.y += jump_vy;
        (*fp).mv.pp.specials.x10 = 0;
        (*fp).mv.pp.specials.x14 = 0;
    }
    (*fp).mv.pp.specials.x18 += 1;
    if ((*fp).mv.pp.specials.x18 as f32) < (*da).x5C {
        co::fall(&mut *fp, gravity, terminal_vel);
    } else {
        co::fall_basic(&mut *fp);
    }
    if (*fp).cmd_vars[0] == 0 && (*fp).mv.pp.specials.x1C != 0.0 {
        let var_f0 = if (*fp).mv.pp.specials.x1C > 0.0 { (*da).x3C } else { -(*da).x3C };
        co::calc_self_accel_accel_to_vel(&mut *fp, (*fp).mv.pp.specials.x1C, var_f0, 0.0);
    } else {
        co::calc_self_accel_deaccel_air(&mut *fp);
    }
}

pub fn ftPp_SpecialAirS1_Phys(gobj: G) {
    unsafe {
        let da = da(fpp(gobj));
        air_phys(gobj, (*da).x60, (*da).x4C_gravity, (*da).x54_terminal_vel)
    }
}

pub fn ftPp_SpecialAirS2_Phys(gobj: G) {
    unsafe {
        let da = da(fpp(gobj));
        air_phys(gobj, (*da).x64, (*da).x50_gravity, (*da).x58_terminal_vel)
    }
}

/// `inline2` (static inline).
unsafe fn inline2(gobj: G) {
    let fp = fpp(gobj);
    if (*fp).cmd_vars[3] != 0 && (*fp).mv.pp.specials.xC != 0 {
        ext::ftPartSetRotX(
            fp,
            0,
            (*fp).facing_dir * melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y),
        );
    } else {
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

/// `inline3` (static inline).
unsafe fn inline3(gobj: G, msid: u32) {
    let fp = fpp(gobj);
    let temp_r30 = da(fp);
    st::ftCommon_8007D5D4(&mut *fp);
    (*fp).x74_self_accel.x = (*fp).xE4_ground_accel_1;
    st::Fighter_ChangeMotionState(gobj, msid as i32, 0xC4C528A, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    inlineA0(gobj);
    co::clamp_self_vel_x(&mut *fp, (*temp_r30).x3C);
}

/// The ground Coll body shared by S1 and S2.
unsafe fn ground_coll(gobj: G, air_msid: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    let temp_r3 = ext::ft_80082888(gobj, &ftNn_Init_803CD820) != 0;
    let mut var_r0 = false;
    if (*fp).gr_vel != 0.0 {
        if (*fp).gr_vel > 0.0 {
            var_r0 = ((*fp).coll_data.env_flags & Collide_LeftWallMask) != 0;
        } else {
            var_r0 = ((*fp).coll_data.env_flags & Collide_RightWallMask) != 0;
        }
    }
    if var_r0 {
        if abs_c((*fp).gr_vel) < (*da).x48 {
            (*fp).gr_vel = if (*fp).gr_vel > 0.0 { -(*da).x48 } else { (*da).x48 };
        } else {
            (*fp).gr_vel *= -(*da).x44;
        }
    }
    if !temp_r3 {
        inline3(gobj, air_msid);
        (*fp).mv.pp.specials.xC = 0;
    } else {
        (*fp).mv.pp.specials.xC = 1;
    }
    inline2(gobj);
    ftPp_SpecialS_8011F720(gobj);
    inlineA0(gobj);
}

pub fn ftPp_SpecialS1_Coll(gobj: G) {
    unsafe { ground_coll(gobj, ftPp_MS_SpecialAirS1 as u32) }
}

pub fn ftPp_SpecialS2_Coll(gobj: G) {
    unsafe { ground_coll(gobj, ftPp_MS_SpecialAirS2 as u32) }
}

/// `inline4` (static inline).
unsafe fn inline4(gobj: G, msid: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, msid as i32, mf::SpecialS_Coll);
    inlineA0(gobj);
    (*fp).x74_self_accel.y = 0.0;
    (*fp).self_vel.y = 0.0;
    co::clamp_ground_vel(&mut *fp, (*da).x38);
}

/// The air Coll body shared by AirS1 and AirS2.
unsafe fn air_coll(gobj: G, ground_msid: u32) {
    let fp = fpp(gobj);
    let da = da(fp);
    let temp_r3 = ext::ft_800824A0(gobj, &ftNn_Init_803CD820) != 0;
    // `(env_flags & Collide_CeilingMask) == 1` can never hold (the mask is 0x6000); kept as written.
    if ((*fp).coll_data.env_flags & 0x6000) == 1 {
        (*fp).self_vel.y = 0.0;
    }
    let mut var_r0 = false;
    if (*fp).self_vel.x != 0.0 {
        if (*fp).self_vel.x > 0.0 {
            var_r0 = ((*fp).coll_data.env_flags & Collide_LeftWallMask) != 0;
        } else {
            var_r0 = ((*fp).coll_data.env_flags & Collide_RightWallMask) != 0;
        }
    }
    if var_r0 {
        if abs_c((*fp).self_vel.x) < (*da).x48 {
            (*fp).self_vel.x = if (*fp).self_vel.x > 0.0 { -(*da).x48 } else { (*da).x48 };
        } else {
            (*fp).self_vel.x *= -(*da).x44;
        }
    }
    if temp_r3 {
        inline4(gobj, ground_msid);
        (*fp).mv.pp.specials.xC = 1;
    } else {
        (*fp).mv.pp.specials.xC = 0;
    }
    inline2(gobj);
    ftPp_SpecialS_8011F720(gobj);
    inlineA0(gobj);
}

pub fn ftPp_SpecialAirS1_Coll(gobj: G) {
    unsafe { air_coll(gobj, ftPp_MS_SpecialS1 as u32) }
}

pub fn ftPp_SpecialAirS2_Coll(gobj: G) {
    unsafe { air_coll(gobj, ftPp_MS_SpecialS2 as u32) }
}
