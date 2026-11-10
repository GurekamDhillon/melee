//! src/melee/ft/kinds/ftPopo/ftpopospeciallw.c: Ice Climbers' down-B (Blizzard). One fused site:
//! fn_80122D2C (`pos.x += xBC * facing_dir` is an fmadds; tools/rust-port/fma_sites.py).
//! Not smoke-tested yet (hooks-heavy; only compiled).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use crate::tu::ftpopo::mf;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_types::*;

/// Already ported (rounds 2/5): re-exported under this TU.



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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, off, out),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, off as _, out as _); } return; },
                }
            }
    pub static mut ITCLIMBERSBLIZZARD_SPAWN: Option<fn(*mut HSD_GObj, *mut Vec3, f32) > = None;
            #[allow(non_snake_case)]
            pub fn itClimbersBlizzard_Spawn(parent: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32)  {
                match unsafe { ITCLIMBERSBLIZZARD_SPAWN } {
                    Some(f) => f(parent, pos, facing_dir),
                    None => { unsafe { melee_items::tu::itclimbersblizzard::itClimbersBlizzard_Spawn(parent as _, pos as _, facing_dir as _); } return; },
                }
            }
    hook_or!(FT_800881D8, ft_800881D8, ft_800881d8_port, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    fn ft_800881d8_port(fp: *mut Fighter, sfx: i32, vol: u8, pan: u8) {
        melee_fighter::ft0881::ft_800881d8(unsafe { &mut *fp }, sfx, vol, pan)
    }
    hook_or!(FT_80088510, ft_80088510, ft_80088510_port, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    fn ft_80088510_port(fp: *mut Fighter, sfx: i32, vol: u8, pan: u8) {
        melee_fighter::ft0881::ft_80088510(unsafe { &mut *fp }, sfx, vol, pan)
    }
    /// `efSync_Spawn(gfx_id, gobj, ...)` (variadic in C; `va` are the extra word arguments).
    pub static mut EFSYNC_SPAWN: Option<fn(i32, *mut HSD_GObj, &[usize]) -> *mut core::ffi::c_void> = None;
    pub fn efSync_Spawn(id: i32, gobj: *mut HSD_GObj, va: &[usize]) -> *mut core::ffi::c_void {
        match unsafe { EFSYNC_SPAWN } {
            Some(f) => f(id, gobj, va),
            None => unsafe { crate::adapt::efSync_Spawn_usize(id, gobj, va) },
        }
    }
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

/// The block shared by the Anim functions and `ftPp_SpecialHi_80122898` (the effect teardown).
unsafe fn teardown_blizzard(gobj: G) {
    let mut fp = fpp(gobj);
    if (*fp).u.pp.x2230_b0() != 0 {
        st::efLib_DestroyAll(gobj);
        fp = fpp(gobj);
        (*fp).u.pp.set_x2230_b0(0);
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
        ext::ftPartSetRotX(fpp(gobj), 0, 0.0);
    }
}

pub fn ftPp_SpecialHi_80122898(gobj: G) {
    melee_core::ct!("ftPp_SpecialHi_80122898");
    unsafe { teardown_blizzard(gobj) }
}

unsafe fn enter(gobj: G, msid: u32) {
    let fp = fpp(gobj);
    (*fp).x2210.throw_flags = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).cmd_vars[3] = 0;
    (*fp).mv.pp.speciallw.x0 = 0;
    (*fp).mv.pp.speciallw.set_x4_b0(0);
    st::Fighter_ChangeMotionState(gobj, msid as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    (*fp).accessory4_cb = Some(crate::chars::cb::ftpopospeciallw__fn_80122D2C);
}

pub fn ftPp_SpecialLw_Enter(gobj: G) {
    melee_core::ct!("ftPp_SpecialLw_Enter");
    unsafe { enter(gobj, ftPp_MS_SpecialLw as u32) }
}

pub fn ftPp_SpecialAirLw_Enter(gobj: G) {
    melee_core::ct!("ftPp_SpecialAirLw_Enter");
    unsafe { enter(gobj, ftPp_MS_SpecialAirLw as u32) }
}

pub fn ftPp_SpecialLw_Anim(gobj: G) {
    melee_core::ct!("ftPp_SpecialLw_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            teardown_blizzard(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftPp_SpecialAirLw_Anim(gobj: G) {
    melee_core::ct!("ftPp_SpecialAirLw_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            teardown_blizzard(gobj);
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftPp_SpecialLw_IASA(_gobj: G) { melee_core::ct!("ftPp_SpecialLw_IASA");}
pub fn ftPp_SpecialAirLw_IASA(_gobj: G) { melee_core::ct!("ftPp_SpecialAirLw_IASA");}

/// `ftPp_set_cbs` (static inline).
#[inline(always)]
unsafe fn ftPp_set_cbs(gobj: G) {
    let fp = fpp(gobj);
    (*fp).death2_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
    (*fp).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
}

/// `ftPp_SpecialLw_Coll_inline` (static inline).
unsafe fn ftPp_SpecialLw_Coll_inline(gobj: G) {
    let fp = fpp(gobj);
    if (*fp).cmd_vars[3] != 0 {
        ext::ftPartSetRotX(
            fp,
            0,
            (*fp).facing_dir
                * melee_runtime::lbtrigf::atan2f((*fp).coll_data.floor.normal.x, (*fp).coll_data.floor.normal.y),
        );
    } else {
        ext::ftPartSetRotX(fp, 0, 0.0);
    }
}

pub fn fn_80122B54(gobj: G) {
    melee_core::ct!("fn_80122B54");
    unsafe {
        let fp = fpp(gobj);
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(
            gobj,
            ftPp_MS_SpecialLw as i32,
            mf::SpecialLw_Coll,
            (*fp).cur_anim_frame,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
        ftPp_set_cbs(gobj);
        ftPp_SpecialLw_Coll_inline(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftpopospeciallw__fn_80122D2C);
    }
}

/// `ftPp_SpecialLw_Coll_Land` (static inline).
unsafe fn ftPp_SpecialLw_Coll_Land(gobj: G) {
    let fp = fpp(gobj);
    st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, ftPp_MS_SpecialAirLw as i32, mf::SpecialLw_Coll);
    ftPp_set_cbs(gobj);
    (*fp).accessory4_cb = Some(crate::chars::cb::ftpopospeciallw__fn_80122D2C);
    st::ftCommon_ClampAirDrift(&mut *fp);
}

pub fn ftPp_SpecialLw_Coll(gobj: G) {
    melee_core::ct!("ftPp_SpecialLw_Coll");
    unsafe {
        if st::ft_80082708(gobj) == GA_Ground as i32 {
            let fp = fpp(gobj);
            ext::ftPartSetRotX(fp, 0, 0.0);
            ftPp_SpecialLw_Coll_Land(gobj);
        } else {
            ftPp_SpecialLw_Coll_inline(gobj);
        }
    }
}

pub fn fn_80122D2C(gobj: G) {
    melee_core::ct!("fn_80122D2C");
    unsafe {
        let fp = fpp(gobj);
        let mut pos: Vec3 = core::mem::zeroed();
        if (*fp).mv.pp.speciallw.x4_b0() != 0 {
            if (*fp).mv.pp.speciallw.x0 == 0 {
                let da = (*fp).dat_attrs as *const ftIceClimberAttributes;
                ext::lb_8000B1CC((*(*fp).parts.add(FtPart_L3rdNa as usize)).joint, core::ptr::null(), &mut pos);
                pos.x = fmadds((*da).xBC, (*fp).facing_dir, pos.x);
                pos.y += (*da).xC0;
                ext::itClimbersBlizzard_Spawn(gobj, &mut pos, (*fp).facing_dir);
                (*fp).mv.pp.speciallw.x0 = f2i32((*da).xB8 as f64);
            }
            (*fp).mv.pp.speciallw.x0 -= 1;
        }
        match (*fp).cmd_vars[0] {
            1 => {
                ext::efSync_Spawn(0x4EC, gobj, &[(*(*fp).parts.add(FtPart_L4thNb as usize)).joint as usize]);
                (*fp).mv.pp.speciallw.set_x4_b0(1);
                (*fp).u.pp.set_x2230_b0(1);
                ftPp_set_cbs(gobj);
                (*fp).cmd_vars[0] = 0;
                if (*fp).kind == Ft_Kind_Popo as _ {
                    ext::ft_800881D8(fp, 0x1FC42, 0x7F, 0x40);
                    ext::ft_80088510(fp, 0x1FBEB, 0x7F, 0x40);
                } else {
                    ext::ft_800881D8(fp, 0x1FC0F, 0x7F, 0x40);
                    ext::ft_80088510(fp, 0x1FBEE, 0x7F, 0x40);
                }
            }
            2 => {
                (*fp).mv.pp.speciallw.set_x4_b0(0);
                (*fp).cmd_vars[0] = 0;
            }
            _ => {}
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftPp_SpecialLw_Phys` (src/melee/ft/kinds/ftPopo/ftpopospeciallw.c).
pub fn ftPp_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialLw_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftPp_SpecialAirLw_Phys` (src/melee/ft/kinds/ftPopo/ftpopospeciallw.c).
pub fn ftPp_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialAirLw_Phys");
    st::ft_80084EEC(gobj);
}

/// `ftPp_SpecialAirLw_Coll` (src/melee/ft/kinds/ftPopo/ftpopospeciallw.c).
pub fn ftPp_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialAirLw_Coll");
    unsafe {
        st::ft_80082C74(gobj, Some(crate::chars::cb::ftpopospeciallw__fn_80122B54));
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPp_set_cbs_fp` (static) (src/melee/ft/kinds/ftPopo/ftpopospeciallw.c).
pub fn ftPp_set_cbs_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_set_cbs_fp");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).death2_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
        (*fp).take_dmg_cb = Some(crate::chars::cb::ftpopo__ftPp_Init_8011F060);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPp_SpecialLw_Coll_Land_fp` (static) (src/melee/ft/kinds/ftPopo/ftpopospeciallw.c).
pub fn ftPp_SpecialLw_Coll_Land_fp(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPp_SpecialLw_Coll_Land_fp");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 358 /* ftPp_MS_SpecialAirLw */, 0xC4C5282u32 /* ftPp_MF_SpecialLw_Coll */);
        crate::chars::r3::ftPopo::ftpopospeciallw::ftPp_set_cbs(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftpopospeciallw__fn_80122D2C);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}
