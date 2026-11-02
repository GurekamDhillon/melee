//! src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c: Yoshi's Down B (Yoshi Bomb). Fused site
//! (tools/rust-port/fma_sites.py): `-1.0f * star_offset.x + transn.x` in `fn_8012E644`, written with
//! `melee_core::fp::fmadds`. State-oracle checked against the C (tests/oracle_ftyoshispeciallw.rs,
//! gen/oracle); the earlier rounds' thin callbacks are re-exported and checked through the legacy
//! `crate::stubs` hooks.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftinl;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/4): re-exported under this TU.




/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut IT_802B2FC8: Option<fn(*mut HSD_GObj, *mut Vec3, f32) -> *mut Item_GObj> = None;
    #[allow(non_snake_case)]
    pub fn it_802B2FC8(parent: *mut HSD_GObj, pos: *mut Vec3, dir: f32) -> *mut Item_GObj {
        match unsafe { IT_802B2FC8 } { Some(f) => f(parent, pos, dir), None => { return unsafe { melee_items::tu::ityoshistar::it_802B2FC8(parent as _, pos as _, dir as _) as _ }; } }
    }
    pub use crate::tu::ftinl::ext::{
        ftAnim_8006EBA4, ftCliffCommon_80081298, ftCliffCommon_80081370, ftCommon_8007D5D4, ft_80085134,
        ft_CheckGroundAndLedge, Fighter_ChangeMotionState,
    };
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fn_8012E644` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_fn_8012E644(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012E644");
    fn_8012E644(g);
}

/// Float at GameCube offset `gc` of the attribute block. The block keeps the GC layout (4-byte `x1C`/`x20`
/// pointers), so the native `ftYs_DatAttrs` fields after `x20` sit 8 bytes too far; read by GC offset.
#[inline(always)]
pub(crate) unsafe fn gc_f32(da: *const ftYs_DatAttrs, gc: usize) -> f32 {
    core::ptr::read_unaligned((da as *const u8).add(gc) as *const f32)
}

/// Spawn the two Yoshi Bomb stars either side of Yoshi's TransN joint.
pub fn fn_8012E644(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012E644");
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftYs_DatAttrs;
        let (off_x, off_y) = (gc_f32(da, 0x118), gc_f32(da, 0x11C));
        let mut transn_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut star_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TransN as usize)).joint, core::ptr::null_mut(), &mut transn_pos);
        star_pos.x = fmadds(-1.0, off_x, transn_pos.x);
        star_pos.y = transn_pos.y + off_y;
        star_pos.z = transn_pos.z;
        ext::it_802B2FC8(gobj, &mut star_pos, -1.0);
        star_pos.x = transn_pos.x + off_x;
        ext::it_802B2FC8(gobj, &mut star_pos, 1.0);
        (*fp).accessory4_cb = None;
    }
}

/// `ftYoshi_SpecialLw_SetVars(gobj)` (static inline).
pub fn ftYoshi_SpecialLw_SetVars(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYoshi_SpecialLw_SetVars");
    unsafe { set_vars(gobj) }
}

/// `reset(gobj)` (static inline): the same stores as `ftYoshi_SpecialLw_SetVars`, in another order.
pub fn reset(gobj: *mut HSD_GObj) {
    melee_core::ct!("reset");
    unsafe { set_vars(gobj) }
}

unsafe fn set_vars(gobj: *mut HSD_GObj) {
    let fp = fpp(gobj);
    (*fp).cmd_vars[1] = 0;
    (*fp).cmd_vars[0] = 0;
    (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
    (*fp).self_vel.y = 0.0;
    (*fp).self_vel.x = 0.0;
    (*fp).x6A4_transNOffset.y = 0.0;
    (*fp).set_x2223_b4(1);
}

pub fn ftYs_SpecialLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLw_Enter");
    unsafe {
        let fp = fpp(gobj);
        ext::ftCommon_8007D5D4(fp);
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialLw as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        ftYoshi_SpecialLw_SetVars(gobj);
    }
}

pub fn ftYs_SpecialAirLw_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirLw_Enter");
    unsafe {
        ext::Fighter_ChangeMotionState(gobj, ftYs_MS_SpecialAirLw as i32, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        reset(gobj);
    }
}

pub fn ftYs_SpecialAirLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirLw_Phys");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const ftYoshiAttributes);
        ext::ft_80085134(gobj);
        if (*fp).self_vel.y < da.x114 || (*fp).cmd_vars[1] != 0 {
            (*fp).self_vel.y = da.x114;
        }
    }
}

pub fn ftYs_SpecialLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLw_Coll");
    ftYs_SpecialAirLw_Coll(gobj);
}

pub fn ftYs_SpecialAirLw_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirLw_Coll");
    unsafe {
        let fp = fpp(gobj);
        if ext::ft_CheckGroundAndLedge(gobj, ftinl::ftGetFacingDirInt(fp)) {
            if (*fp).cmd_vars[0] != 0 && (*fp).self_vel.y <= 0.0 {
                ftYs_SpecialS_8012EAD8(gobj);
            } else {
                ext::ftCommon_8007D5D4(fp);
            }
        } else if (*fp).cmd_vars[0] != 0 && ext::ftCliffCommon_80081298(gobj) {
            ext::ftCliffCommon_80081370(gobj);
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftYs_SpecialS_8012EA04` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialS_8012EA04(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialS_8012EA04");
    st::Fighter_ChangeMotionState(gobj, 368 /* ftYs_MS_SpecialAirLw */, 0xC4C5088u32 /* Ft_MF_SkipHit | Ft_MF_SkipMatAnim | Ft_MF_SkipColAnim | Ft_MF_UpdateCmd | Ft_MF_SkipItemVis | Ft_MF_Unk19 | Ft_MF_SkipModelPartVis | Ft_MF_SkipModelFlags | Ft_MF_Unk27 */, 20.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
}

/// `ftYs_SpecialLwLanding_Anim` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialLwLanding_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLwLanding_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftYs_SpecialLwLanding_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialLwLanding_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLwLanding_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialLwLanding_Coll` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialLwLanding_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLwLanding_Coll");
    if st::ft_80082708(gobj) == 0 /* GA_Ground */ {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftYs_SpecialLw_Anim` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLw_Anim");
    unsafe {
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            crate::chars::r2::ftYoshi::ftyoshispeciallw::ftYs_SpecialS_8012EA04(gobj);
        }
    }
}

/// `ftYs_SpecialS_8012EAD8` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialS_8012EAD8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialS_8012EAD8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).gr_vel = f32::from_bits(0x00000000) /* 0.0f */;
        st::ftCommon_8007D7FC(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 367 /* ftYs_MS_SpecialLwLanding */, 0x0u32 /* Ft_MF_None */, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftyoshispeciallw__fn_8012E644);
    }
}

/// `ftYs_SpecialAirLw_Anim` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialAirLw_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirLw_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            (*fp).cmd_vars[1] = (1_u32 /* true */);
        }
    }
}

/// `ftYs_SpecialLw_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c).
pub fn ftYs_SpecialLw_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialLw_Phys");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ft_80085134(gobj);
        if ((*fp).self_vel.y < f32::from_bits(0x00000000) /* 0.0f */) {
            (*fp).self_vel.y = f32::from_bits(0x00000000) /* 0.0f */;
        }
    }
}
