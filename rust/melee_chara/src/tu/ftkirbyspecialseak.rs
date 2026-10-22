//! src/melee/ft/kinds/ftKirby/ftkirbyspecialseak.c: Kirby's Sheik copy ability (needle storm).
//! Fused sites (tools/rust-port/fma_sites.py): `ftKb_SpecialNSk_8010603C` 1 (`pos.y += scale.y *
//! y_offset`) and `fn_80106DB0` 5 (the four `pos.x/y += scale.y * (..)` and the inner
//! `2.0f * rand + y_offset`), all `fmadds` here. Smoke-tested through recording hooks by
//! tests/tu_ftkirbyspecialseak.rs (the needle positions are checked against hand-computed fmadds).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::hooks as fh;
use melee_types::*;

/// Already ported (rounds 2, 3, 11): re-exported under this TU.
pub use crate::chars::r11::ftKirby::ftkirbyspecialseak::{
    ftKb_SkSpecialAirNLoop_Coll, ftKb_SkSpecialAirNLoop_IASA, ftKb_SkSpecialAirNStart_Coll,
    ftKb_SkSpecialNLoop_Coll, ftKb_SkSpecialNLoop_IASA, ftKb_SkSpecialNStart_Coll,
};
pub use crate::chars::r2::ftKirby::ftkirbyspecialseak::{
    ftKb_SkSpecialAirNCancel_Phys, ftKb_SkSpecialAirNEnd_Phys, ftKb_SkSpecialAirNLoop_Phys,
    ftKb_SkSpecialAirNStart_Phys, ftKb_SkSpecialNCancel_Phys, ftKb_SkSpecialNEnd_Phys,
    ftKb_SkSpecialNLoop_Phys, ftKb_SkSpecialNStart_Phys,
};
pub use crate::chars::r3::ftKirby::ftkirbyspecialseak::ftKb_SkSpecialNCancel_Anim;

/// `ftKb_Init_803CB770`: the needle spread offsets.
pub const ftKb_Init_803CB770: [f32; 9] = [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75, 1.0];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // `efSync_Spawn(gfx_id, gobj, ...)` (ef/efsync.c, variadic): the trailing pointers as a slice.
    hook!(EFSYNC_SPAWN, efSync_Spawn, (gfx_id: i32, gobj: *mut HSD_GObj, args: &[*mut c_void]) -> *mut c_void);
    hook!(IT_802AFD8C, it_802AFD8C, (gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32) -> *mut Item_GObj);
    hook!(IT_802AFEA8, it_802AFEA8, (item: *mut Item_GObj, owner: *mut HSD_GObj, flag: i32));
    hook!(IT_802B19AC, it_802B19AC, (gobj: *mut HSD_GObj, pos: *mut Vec3, bone: i32, kind: i32, facing_dir: f32) -> *mut Item_GObj);
    /// Recording hook; unset, the native `melee_fighter::tu::ftcolanim` port runs.
    pub static mut FTCO_800BFFD0: Option<fn(*mut Fighter, i32, i32) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_800BFFD0(fp: *mut Fighter, a: i32, b: i32) -> bool {
        match unsafe { FTCO_800BFFD0 } {
            Some(f) => f(fp, a, b),
            None => melee_fighter::tu::ftcolanim::ftco_800bffd0(unsafe { &mut *fp }, a, b),
        }
    }
    /// Recording hook; unset, the native `melee_fighter::tu::ftCo_Landing` port runs.
    pub static mut FTCO_LANDING_ENTER_BASIC: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_Landing_Enter_Basic(gobj: *mut HSD_GObj) {
        match unsafe { FTCO_LANDING_ENTER_BASIC } {
            Some(f) => f(gobj),
            None => melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter_Basic(gobj),
        }
    }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}
#[inline(always)]
unsafe fn da(fp: *mut Fighter) -> *mut ftKb_DatAttrs {
    (*fp).dat_attrs as *mut ftKb_DatAttrs
}

/// `ftKb_SpecialN_set_cbs` (ftKirby/inlines.h, static inline): Kirby's damage callbacks.
#[inline(always)]
pub fn ftKb_SpecialN_set_cbs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
    }
}

/// `fn_80105FEC`: the empty reflect callback of Kirby's Zelda ability.
pub fn fn_80105FEC(_gobj: *mut HSD_GObj) {}

/// `fn_80105FEC` as a Fighter / reflect callback entry.
pub unsafe extern "C" fn cb_fn_80105FEC(_g: *mut HSD_GObj) {}

/// `fn_80106DB0` as a Fighter callback entry (`accessory4_cb`).
pub unsafe extern "C" fn cb_fn_80106DB0(g: *mut HSD_GObj) {
    fn_80106DB0(g);
}

/// Returns `false` when a needle is held (`xB8 != 0`), else `true`.
pub fn ftKb_SpecialNSk_80105FF0(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        // The C loads `GET_FIGHTER(gobj)` before the NULL test (a harmless GC low-memory read).
        if !gobj.is_null() {
            let ft = fpp(gobj);
            if !(*ft).u.kb.xB8.is_null() {
                return false;
            }
            return true;
        }
        true
    }
}

pub fn ftKb_SpecialNSk_80106020(gobj: *mut HSD_GObj) -> i32 {
    unsafe {
        if !gobj.is_null() {
            let ft = fpp(gobj);
            return (*ft).u.kb.xB4;
        }
        0
    }
}

/// Release every stored needle (one thrown per stored charge) when the held one is dropped.
pub fn ftKb_SpecialNSk_8010603C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if !(*fp).u.kb.xB8.is_null() {
            (*fp).u.kb.xB8 = core::ptr::null_mut();
            while (*fp).u.kb.xB4 != 0 {
                let mut pos = (*fp).cur_pos;
                let y_offset = if (*fp).ground_or_air == GA_Ground {
                    (*da).specialn_sk_graphic_y_offset_ground
                } else {
                    (*da).specialn_sk_graphic_y_offset_air
                };
                pos.y = fmadds((*fp).x34_scale.y, y_offset, pos.y);
                pos.z = 0.0;
                let needle = ext::it_802AFD8C(gobj, &mut pos, It_Kind_Kirby_SeakNeedleThrow as i32, (*fp).facing_dir);
                if !needle.is_null() {
                    ext::it_802AFEA8(needle, gobj, 1);
                }
                (*fp).u.kb.xB4 -= 1;
            }
        }
        (*fp).u.kb.xB4 = 0;
    }
}

/// Shared start of the two Enters after the motion change.
unsafe fn enter_tail(gobj: *mut HSD_GObj, fp: *mut Fighter) {
    (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
    (*fp).cmd_vars = [0; 4];
    (*fp).mv.kb.specialhi.x0 = 0;
    if (*fp).u.kb.xB4 == 0 {
        (*fp).u.kb.xB4 = 1;
    }
    (*fp).mv.kb.specialhi.x4 = 0;
    (*fp).mv.kb.specialhi.x8.i = 0;
    ftKb_SpecialN_set_cbs(gobj);
    st::ftAnim_8006EBA4(gobj);
}

pub fn ftKb_SpecialNSk_8010612C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_SkSpecialNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        enter_tail(gobj, fp);
    }
}

pub fn ftKb_SpecialNSk_801061E4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_SkSpecialAirNStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        enter_tail(gobj, fp);
    }
}

/// Start-up done: pick up the held needle and enter the loop state `loop_msid`.
unsafe fn start_anim(gobj: *mut HSD_GObj, loop_msid: i32) {
    let fp = fpp(gobj);
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        (*fp).u.kb.xB8 = ext::it_802B19AC(gobj, &mut (*fp).cur_pos, 39, It_Kind_Kirby_SeakNeedleHeld as i32, (*fp).facing_dir);
        st::Fighter_ChangeMotionState(gobj, loop_msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ftKb_SpecialN_set_cbs(gobj);
    }
}

pub fn ftKb_SkSpecialNStart_Anim(gobj: *mut HSD_GObj) {
    unsafe { start_anim(gobj, ftKb_MS_SkSpecialNLoop as i32) }
}

pub fn ftKb_SkSpecialAirNStart_Anim(gobj: *mut HSD_GObj) {
    unsafe { start_anim(gobj, ftKb_MS_SkSpecialAirNLoop as i32) }
}

/// Shared body of the two Loop Anims: charge sfx on frame 0, one needle stored per loop.
unsafe fn loop_anim(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    if (*fp).mv.kb.specialhi.x8.i == 0 {
        melee_fighter::hooks::play_sfx(&mut *fp, 270134, 127, 64);
    }
    (*fp).mv.kb.specialhi.x8.i += 1;
    if (*fp).cur_anim_frame == 0.0 {
        (*fp).u.kb.xB4 += 1;
        (*fp).mv.kb.specialhi.x8.i = 0;
        if (*fp).u.kb.xB4 > 6 {
            (*fp).u.kb.xB4 = 6;
            (*fp).mv.kb.specialhi.x8.i = 100;
            ext::ftCo_800BFFD0(fp, 87, 0);
        }
    }
}

pub fn ftKb_SkSpecialNLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj) }
}

pub fn ftKb_SkSpecialAirNLoop_Anim(gobj: *mut HSD_GObj) {
    unsafe { loop_anim(gobj) }
}

/// The `x0` switch both End Anims share: throw a needle on frames 2, 5, 8, 11, 14, 17.
unsafe fn end_anim_step(fp: *mut Fighter) {
    match (*fp).mv.kb.specialhi.x0 {
        2 | 5 | 8 | 11 | 14 | 17 => {
            (*fp).mv.kb.specialhi.x4 = 1;
            (*fp).u.kb.xB8 = core::ptr::null_mut();
        }
        _ => {}
    }
    (*fp).mv.kb.specialhi.x0 += 1;
}

pub fn ftKb_SkSpecialNEnd_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        end_anim_step(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// Leave the move into Fall, or FallSpecial when the freefall toggle is set.
unsafe fn fall_or_special(gobj: *mut HSD_GObj, da: *mut ftKb_DatAttrs) {
    if (*da).specialn_sk_freefall_toggle == 0.0 {
        st::ftCo_Fall_Enter(gobj);
    } else {
        st::ftCo_80096900(gobj, 1, 0, 1, 1.0, (*da).specialn_sk_freefall_toggle);
    }
}

pub fn ftKb_SkSpecialAirNCancel_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        (*fp).u.kb.xB8 = core::ptr::null_mut();
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_special(gobj, da);
        }
    }
}

pub fn ftKb_SkSpecialAirNEnd_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        end_anim_step(fp);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            fall_or_special(gobj, da);
        }
    }
}

pub fn ftKb_SkSpecialNStart_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_SkSpecialNCancel_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_SkSpecialNEnd_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_SkSpecialAirNStart_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_SkSpecialAirNCancel_IASA(_gobj: *mut HSD_GObj) {}
pub fn ftKb_SkSpecialAirNEnd_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftKb_SkSpecialNCancel_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            fall_or_special(gobj, da);
        }
    }
}

pub fn ftKb_SkSpecialNEnd_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            (*fp).u.kb.xB4 = 0;
            (*fp).mv.kb.specialhi.x4 = 0;
            fall_or_special(gobj, da);
        }
    }
}

pub fn ftKb_SkSpecialAirNCancel_Coll(gobj: *mut HSD_GObj) {
    if st::ft_80081D0C(gobj) != GA_Ground as i32 {
        ext::ftCo_Landing_Enter_Basic(gobj);
    }
}

pub fn ftKb_SkSpecialAirNEnd_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if st::ft_80081D0C(gobj) != 0 {
            (*fp).u.kb.xB4 = 0;
            (*fp).mv.kb.specialhi.x4 = 0;
            ext::ftCo_Landing_Enter_Basic(gobj);
        }
    }
}

/// `perm_randi` (static inline): `HSD_Randi(n)`.
pub fn perm_randi(n: i32) -> i32 {
    fh::hsd_randi(n)
}

/// The scripted needle throw (accessory4 callback of the End states).
pub fn fn_80106DB0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = da(fp);
        if (*fp).mv.kb.specialhi.x4 != 0 {
            (*fp).mv.kb.specialhi.x4 = 0;
            if (*fp).u.kb.xB4 > 0 {
                let mut pos = (*fp).cur_pos;
                let scale = (*fp).x34_scale.y;
                if (*fp).ground_or_air == GA_Ground {
                    pos.x = fmadds(scale, (*da).specialn_sk_graphic_x_offset_ground * (*fp).facing_dir, pos.x);
                    pos.y = fmadds(scale, (*da).specialn_sk_graphic_y_offset_ground + ftKb_Init_803CB770[perm_randi(9) as usize], pos.y);
                } else {
                    pos.x = fmadds(scale, (*da).specialn_sk_graphic_x_offset_air * (*fp).facing_dir, pos.x);
                    let inner = fmadds(2.0, ftKb_Init_803CB770[perm_randi(9) as usize], (*da).specialn_sk_graphic_y_offset_air);
                    pos.y = fmadds(scale, inner, pos.y);
                }
                pos.z = 0.0;
                let needle = ext::it_802AFD8C(gobj, &mut pos, It_Kind_Kirby_SeakNeedleThrow as i32, (*fp).facing_dir);
                if !needle.is_null() {
                    ext::it_802AFEA8(needle, gobj, 0);
                }
                (*fp).u.kb.xB4 -= 1;
                ext::efSync_Spawn(0x503, gobj, &[&mut pos as *mut Vec3 as *mut _]);
                melee_fighter::hooks::play_sfx(&mut *fp, 0x41F3C, 127, 64);
            }
        }
    }
}
