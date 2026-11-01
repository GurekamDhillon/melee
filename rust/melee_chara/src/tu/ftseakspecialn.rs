//! src/melee/ft/kinds/ftSeak/ftseakspecialn.c: Sheik's Needle Storm (neutral B): charge, throw,
//! cancel, and the shared needle-release helper. Fused sites (tools/rust-port/fma_sites.py):
//! `ftSk_SpecialN_80111FBC` 1 (`pos.y += scale.y * y_scale`), `shootNeedles` 5 (x and y offsets of
//! the grounded and aerial branches, and the aerial `2.0f * n + xC`); each is `fmadds`. Not yet tested: no C oracle and no smoke test (unverified).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3): re-exported under this TU.



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
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
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
    crate::port_hook!(HSD_RANDI, HSD_Randi, crate::adapt::HSD_Randi, (max: i32) -> i32);
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: i32, b: i32) {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => { unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *(fp as *mut _), a as _, b as _); } },
        }
    }
    hook_or!(FTCO_LANDING_ENTER_BASIC, ftCo_Landing_Enter_Basic, (gobj: *mut HSD_GObj),
        melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter_Basic);
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: i32, pan: i32),
        |fp: *mut Fighter, sfx: i32, vol: i32, pan: i32| unsafe { melee_fighter::hooks::play_sfx(&mut *fp, sfx, vol, pan) });
    crate::port_hook!(EFSYNC_SPAWN, efSync_Spawn, crate::adapt::efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut core::ffi::c_void]) -> *mut core::ffi::c_void);
    // Items (it/kinds/itseakneedlethrown.c, itseakneedleheld.c)
    pub static mut IT_802AFD8C: Option<fn(*mut HSD_GObj, *mut Vec3, ItemKind, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AFD8C(owner: *mut HSD_GObj, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) -> *mut HSD_GObj {
                match unsafe { IT_802AFD8C } {
                    Some(f) => f(owner, pos, kind, facing_dir),
                    None => { return unsafe { melee_items::tu::itseakneedlethrown::it_802AFD8C(owner as _, pos as _, kind as _, facing_dir as _) as _ }; },
                }
            }
    pub static mut IT_802AFEA8: Option<fn(*mut HSD_GObj, *mut HSD_GObj, i32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AFEA8(item: *mut HSD_GObj, owner: *mut HSD_GObj, arg: i32)  {
                match unsafe { IT_802AFEA8 } {
                    Some(f) => f(item, owner, arg),
                    None => { unsafe { melee_items::tu::itseakneedlethrown::it_802AFEA8(item as _, owner as _, arg as _); } return; },
                }
            }
    pub static mut IT_802B19AC: Option<fn(*mut HSD_GObj, *mut Vec3, i32, ItemKind, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802B19AC(owner: *mut HSD_GObj, pos: *mut Vec3, bone: i32, kind: ItemKind, facing_dir: f32) -> *mut HSD_GObj {
                match unsafe { IT_802B19AC } {
                    Some(f) => f(owner, pos, bone, kind, facing_dir),
                    None => { return unsafe { melee_items::tu::itseakneedleheld::it_802B19AC(owner as _, pos as _, bone as _, kind as _, facing_dir as _) as _ }; },
                }
            }
}

/// `needleYPosScale`.
pub const needleYPosScale: [f32; 9] = [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];

/// The `mf` flags of the Coll functions: `(1<<7)|(1<<12)|(1<<14)|(1<<18)|(1<<19)|(1<<22)|(1<<26)|(1<<27)`.
const MF: u32 = (1 << 7) | (1 << 12) | (1 << 14) | (1 << 18) | (1 << 19) | (1 << 22) | (1 << 26) | (1 << 27);

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *const ftSeakAttributes {
    (*fp).dat_attrs as *const ftSeakAttributes
}

/// `Fighter_SetDamageCallback` (ft/inlines.h).
unsafe fn set_damage_callback(gobj: *mut HSD_GObj, cb: HSD_GObjEvent) {
    let fp = fpp(gobj);
    (*fp).take_dmg_cb = cb;
    (*fp).death2_cb = cb;
}

pub unsafe extern "C" fn cb_shootNeedles(g: *mut HSD_GObj) {
    melee_core::ct!("cb_shootNeedles");
    shootNeedles(g);
}

/// Sheik_CheckAndDestroyNeedles
pub fn ftSk_SpecialN_80111FBC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialN_80111FBC");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if !(*fp).u.sk.x4.is_null() {
            (*fp).u.sk.x4 = core::ptr::null_mut();
            while (*fp).u.sk.x0 != 0 {
                let mut pos = (*fp).cur_pos;
                let y_scale = if (*fp).ground_or_air == GA_Ground { (*da).x4 } else { (*da).xC };
                pos.y = fmadds((*fp).x34_scale.y, y_scale, pos.y);
                pos.z = 0.0;
                let item_gobj = ext::it_802AFD8C(gobj, &mut pos, It_Kind_Seak_NeedleThrow, (*fp).facing_dir);
                if !item_gobj.is_null() {
                    ext::it_802AFEA8(item_gobj, gobj, 1);
                }
                (*fp).u.sk.x0 -= 1;
            }
            let cb = crate::tu::ftseak::cb_ftSk_Init_80110198 as unsafe extern "C" fn(*mut HSD_GObj);
            if (*fp).death2_cb == Some(cb) || (*fp).take_dmg_cb == Some(cb) {
                ftSk_SpecialN_801120D4(gobj);
            }
        }
        (*fp).u.sk.x0 = 0;
    }
}

/// `doEnter` (static inline).
fn doEnter(gobj: *mut HSD_GObj, msid: i32) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
        st::Fighter_ClearCmdVars(&mut *fp);
        (*fp).mv.sk.specialn.x0 = 0;
        if (*fp).u.sk.x0 == 0 {
            (*fp).u.sk.x0 = 1;
        }
        (*fp).mv.sk.specialn.x4 = 0;
        (*fp).mv.sk.specialn.x8 = 0;
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftSk_SpecialN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialN_Enter");
    doEnter(gobj, ftSk_MS_SpecialNStart as i32);
}

pub fn ftSk_SpecialAirN_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirN_Enter");
    doEnter(gobj, ftSk_MS_SpecialAirNStart as i32);
}

/// The shared tail of both Start_Anim functions.
unsafe fn start_anim(gobj: *mut HSD_GObj, loop_msid: i32) {
    let fp = fpp(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        (*fp).u.sk.x4 = ext::it_802B19AC(gobj, &mut (*fp).cur_pos, 23, It_Kind_Seak_NeedleHeld, (*fp).facing_dir);
        st::Fighter_ChangeMotionState(gobj, loop_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
    }
}

pub fn ftSk_SpecialNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNStart_Anim");
    unsafe { start_anim(gobj, ftSk_MS_SpecialNLoop as i32) }
}

/// The shared body of both Loop_Anim functions (Sheik_ChargeNeedlesIncrementer).
unsafe fn loop_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).mv.sk.specialn.x8 == 0 {
        ext::ft_PlaySFX(fp, 270134, 127, 64);
    }
    (*fp).mv.sk.specialn.x8 += 1;
    if (*fp).cur_anim_frame == 0.0 {
        (*fp).u.sk.x0 += 1;
        (*fp).mv.sk.specialn.x8 = 0;
        if (*fp).u.sk.x0 > 6 {
            (*fp).u.sk.x0 = 6;
            (*fp).mv.sk.specialn.x8 = 100;
            ext::ftCo_800BFFD0(fp, 86, 0);
        }
    }
}

pub fn ftSk_SpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNLoop_Anim");
    unsafe { loop_anim(gobj) }
}

/// The shared prologue of both End_Anim functions: on throw frames 2, 5, 8, 11, 14, 17 release
/// the needles; the counter always advances.
unsafe fn end_anim_counter(fp: *mut Fighter) {
    match (*fp).mv.sk.specialn.x0 {
        2 | 5 | 8 | 11 | 14 | 17 => {
            (*fp).mv.sk.specialn.x4 = 1;
            (*fp).u.sk.x4 = core::ptr::null_mut();
        }
        _ => {}
    }
    (*fp).mv.sk.specialn.x0 += 1;
}

pub fn ftSk_SpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        end_anim_counter(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftSk_SpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNStart_Anim");
    unsafe { start_anim(gobj, ftSk_MS_SpecialAirNLoop as i32) }
}

pub fn ftSk_SpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNLoop_Anim");
    unsafe { loop_anim(gobj) }
}

/// `if (da->x10 == 0) ftCo_Fall_Enter(gobj); else ftCo_80096900(gobj, 1, 0, true, 1, da->x10);`
unsafe fn fall_or_land_lag(gobj: *mut HSD_GObj, da: *const ftSeakAttributes) {
    if (*da).x10 == 0.0 {
        st::ftCo_Fall_Enter(gobj);
    } else {
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).x10);
    }
}

pub fn ftSk_SpecialAirNCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNCancel_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).u.sk.x4 = core::ptr::null_mut();
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_land_lag(gobj, da);
        }
    }
}

pub fn ftSk_SpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNEnd_Anim");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        end_anim_counter(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_land_lag(gobj, da);
        }
    }
}

pub fn ftSk_SpecialNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialNStart_IASA");}

/// `doIasa` (static): B released -> End (needles go out at the accessory callback); L/R -> Cancel.
unsafe fn doIasa(gobj: *mut HSD_GObj, end_msid: i32, cancel_msid: i32) {
    let fp = fpp(gobj);
    if ((*fp).input.held_buttons[0] as i32 & HSD_PAD_B) == 0 {
        (*fp).mv.sk.specialn.x0 = 0;
        st::Fighter_ChangeMotionState(gobj, end_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
        (*fp).accessory4_cb = Some(cb_shootNeedles);
    } else if ((*fp).input.pressed_buttons as u32 & HSD_PAD_LR as u32) != 0 {
        st::Fighter_ChangeMotionState(gobj, cancel_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
    }
}

pub fn ftSk_SpecialNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNLoop_IASA");
    unsafe { doIasa(gobj, ftSk_MS_SpecialNEnd as i32, ftSk_MS_SpecialNCancel as i32) }
}

pub fn ftSk_SpecialNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialNCancel_IASA");}

pub fn ftSk_SpecialNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialNEnd_IASA");}

pub fn ftSk_SpecialAirNStart_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialAirNStart_IASA");}

pub fn ftSk_SpecialAirNLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNLoop_IASA");
    unsafe { doIasa(gobj, ftSk_MS_SpecialAirNEnd as i32, ftSk_MS_SpecialAirNCancel as i32) }
}

pub fn ftSk_SpecialAirNCancel_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialAirNCancel_IASA");}

pub fn ftSk_SpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftSk_SpecialAirNEnd_IASA");}

/// Ground Start/Loop Coll: walked off the edge -> the matching aerial motion.
unsafe fn ground_coll(gobj: *mut HSD_GObj, air_msid: i32) {
    let fp = fpp(gobj);
    if st::ft_80082708(gobj) == GA_Ground as i32 {
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, air_msid, MF);
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
    }
}

pub fn ftSk_SpecialNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNStart_Coll");
    unsafe { ground_coll(gobj, ftSk_MS_SpecialAirNStart as i32) }
}

pub fn ftSk_SpecialNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNLoop_Coll");
    unsafe { ground_coll(gobj, ftSk_MS_SpecialAirNLoop as i32) }
}

pub fn ftSk_SpecialNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNCancel_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            set_damage_callback(gobj, None);
            fall_or_land_lag(gobj, da);
        }
    }
}

pub fn ftSk_SpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            set_damage_callback(gobj, None);
            (*fp).u.sk.x0 = 0;
            (*fp).mv.sk.specialn.x4 = 0;
            fall_or_land_lag(gobj, da);
        }
    }
}

/// `doColl` (static inline).
unsafe fn doColl(gobj: *mut HSD_GObj, msid: i32) {
    let fp = fpp(gobj);
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, msid, MF);
        set_damage_callback(gobj, Some(crate::tu::ftseak::cb_ftSk_Init_80110198));
    }
}

pub fn ftSk_SpecialAirNStart_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNStart_Coll");
    unsafe { doColl(gobj, ftSk_MS_SpecialNStart as i32) }
}

pub fn ftSk_SpecialAirNLoop_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNLoop_Coll");
    unsafe { doColl(gobj, ftSk_MS_SpecialNLoop as i32) }
}

pub fn ftSk_SpecialAirNCancel_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNCancel_Coll");
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        ext::ftCo_Landing_Enter_Basic(gobj);
        unsafe { set_damage_callback(gobj, None) };
    }
}

pub fn ftSk_SpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNEnd_Coll");
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != GA_Ground as i32 {
            set_damage_callback(gobj, None);
            (*fp).u.sk.x0 = 0;
            (*fp).mv.sk.specialn.x4 = 0;
            ext::ftCo_Landing_Enter_Basic(gobj);
        }
    }
}

/// Accessory callback: release one needle volley when the throw frame set `specialn.x4`.
pub fn shootNeedles(gobj: *mut HSD_GObj) {
    melee_core::ct!("shootNeedles");
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);

        if (*fp).mv.sk.specialn.x4 != 0 {
            (*fp).mv.sk.specialn.x4 = 0;

            if (*fp).u.sk.x0 > 0 {
                let mut pos = (*fp).cur_pos;
                let x_scale: f32;
                let y_scale: f32;
                if (*fp).ground_or_air == GA_Ground {
                    x_scale = (*da).x0 * (*fp).facing_dir;
                    pos.x = fmadds((*fp).x34_scale.y, x_scale, pos.x);
                    let rand = ext::HSD_Randi(9);
                    y_scale = (*da).x4 + needleYPosScale[rand as usize];
                    pos.y = fmadds((*fp).x34_scale.y, y_scale, pos.y);
                } else {
                    x_scale = (*da).x8 * (*fp).facing_dir;
                    pos.x = fmadds((*fp).x34_scale.y, x_scale, pos.x);
                    let rand = ext::HSD_Randi(9);
                    y_scale = fmadds(2.0, needleYPosScale[rand as usize], (*da).xC);
                    pos.y = fmadds((*fp).x34_scale.y, y_scale, pos.y);
                }
                pos.z = 0.0;

                let item_gobj = ext::it_802AFD8C(gobj, &mut pos, It_Kind_Seak_NeedleThrow, (*fp).facing_dir);
                if !item_gobj.is_null() {
                    ext::it_802AFEA8(item_gobj, gobj, 0);
                }

                (*fp).u.sk.x0 -= 1;

                ext::efSync_Spawn(1283, gobj, &[&mut pos as *mut Vec3 as *mut core::ffi::c_void]);
                ext::ft_PlaySFX(fp, 270140, 127, 64);
            }
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftSk_SpecialNStart_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNStart_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialNLoop_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNLoop_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialNCancel_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNCancel_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialNEnd_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNEnd_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSk_SpecialAirNStart_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialAirNStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNStart_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftSk_SpecialAirNLoop_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialAirNLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNLoop_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftSk_SpecialAirNCancel_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialAirNCancel_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNCancel_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftSk_SpecialAirNEnd_Phys` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialAirNEnd_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialAirNEnd_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftSk_SpecialN_801120D4` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialN_801120D4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialN_801120D4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).take_dmg_cb = core::mem::zeroed(); /* NULL */
        (*fp).death2_cb = core::mem::zeroed(); /* NULL */
    }
}

/// `ftSk_SpecialNCancel_Anim` (src/melee/ft/kinds/ftSeak/ftseakspecialn.c).
pub fn ftSk_SpecialNCancel_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSk_SpecialNCancel_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).u.sk.x4 = core::mem::zeroed(); /* 0 */
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}
