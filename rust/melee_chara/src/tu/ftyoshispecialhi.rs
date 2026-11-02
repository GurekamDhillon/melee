//! src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c: Yoshi's Egg Throw (up B). Fused sites
//! (tools/rust-port/fma_sites.py): the two `(x4 * x100 + xFC)` of `ftYs_SpecialS_8012DF8C` and the
//! `x4 * x110 + x10C` argument of `fn_8012E110`, written with `melee_core::fp::fmadds`. sinf/cosf are
//! hooks whose default is the MSL port (`melee_runtime::gekko`). State-oracle checked against the C
//! (tests/oracle_ftyoshispecialhi.rs, gen/oracle).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::tu::ftinl;
use melee_core::fp::fmadds;
use melee_types::*;

/// Already ported (rounds 2/3/4/6/7/11): re-exported under this TU.







/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    hook_or!(COSF, cosf, (x: f32) -> f32, melee_runtime::gekko::cosf);
    hook_or!(SINF, sinf, (x: f32) -> f32, melee_runtime::gekko::sinf);
    pub static mut IT_802F2CE0: Option<fn(*mut Item_GObj, i32)> = None;
    #[allow(non_snake_case)]
    pub fn it_802F2CE0(item: *mut Item_GObj, clear_destroy_type: i32) {
        match unsafe { IT_802F2CE0 } {
            Some(f) => f(item, clear_destroy_type),
            None => { unsafe { melee_items::tu::ityoshitongue::it_802F2CE0(item as _, (clear_destroy_type != 0)); } },
        }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    pub static mut IT_802B2A10: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) -> *mut Item_GObj> = None;
    #[allow(non_snake_case)]
    pub fn it_802B2A10(gobj: *mut HSD_GObj, pos: *mut Vec3, arg: i32, dir: f32) -> *mut Item_GObj {
        match unsafe { IT_802B2A10 } { Some(f) => f(gobj, pos, arg, dir), None => { return unsafe { melee_items::tu::ityoshieggthrow::it_802B2A10(gobj as _, pos as _, arg as _, dir as _) as _ }; } }
    }
    pub static mut IT_802B28C8: Option<fn(*mut Item_GObj, *mut Vec3, *mut Vec3, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn it_802B28C8(item: *mut Item_GObj, pos: *mut Vec3, offset: *mut Vec3, facing_dir: f32, unused: f32) {
        match unsafe { IT_802B28C8 } { Some(f) => f(item, pos, offset, facing_dir, unused), None => { unsafe { melee_items::tu::ityoshieggthrow::it_802B28C8(item as _, pos as _, offset as _, facing_dir as _, unused as _); } return; } }
    }
    pub static mut IT_802B2890: Option<fn(*mut Item_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn it_802B2890(item: *mut Item_GObj) {
        match unsafe { IT_802B2890 } { Some(f) => f(item), None => { unsafe { melee_items::tu::ityoshieggthrow::it_802B2890(item as _); } return; } }
    }
    pub use crate::tu::ftinl::ext::{ft_80082C74, ft_80083A48, ft_8008403C};
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `fn_8012E110` as a Fighter callback entry (accessory4_cb): the shared `chars::cb` entry, which calls it.
pub use crate::chars::cb::ftyoshispecialhi__fn_8012E110 as cb_fn_8012E110;
/// `fn_8012E3B4` as a callback entry.
pub unsafe extern "C" fn cb_fn_8012E3B4(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012E3B4");
    fn_8012E3B4(g);
}
/// `fn_8012E44C` as a callback entry.
pub unsafe extern "C" fn cb_fn_8012E44C(g: *mut HSD_GObj) {
    melee_core::ct!("cb_fn_8012E44C");
    fn_8012E44C(g);
}

pub fn ftYs_SpecialS_8012DF18(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialS_8012DF18");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).target_item_gobj.is_null() {
            let msid = (*fp).motion_id as i32;
            let mut clear_destroy_type = false;
            if msid == ftYs_MS_SpecialN1_1 as i32
                || msid == ftYs_MS_SpecialN2_1 as i32
                || msid == ftYs_MS_SpecialAirN1_2 as i32
                || msid == ftYs_MS_SpecialAirN2_1 as i32
            {
                clear_destroy_type = true;
            }
            ext::it_802F2CE0((*fp).target_item_gobj, clear_destroy_type as i32);
            (*fp).x1A64 = core::ptr::null_mut();
            (*fp).target_item_gobj = core::ptr::null_mut();
        }
    }
}

/// Yoshi's `dat_attrs` as the engine stores it. Offsets 0xEC..0x114 are `pad_xEC` of the decoded record: the raw big-endian bytes (swap on read). `ftYs_DatAttrs` types
/// `x1C` / `x20` as `void*`, which widens them natively and shifts every later field, so it must not be used on this block.
#[repr(C)]
pub struct YsDat {
    _p: [u8; 0xEC],
    pub xEC: u32,
    pub xF0: u32,
    pub xF4: u32,
    pub specialhi_base_angle: u32,
    pub xFC: u32,
    pub x100: u32,
    pub x104: u32,
    pub x108: u32,
    pub x10C: u32,
    pub x110: u32,
}

impl YsDat {
    #[inline] pub fn xEC(&self) -> f32 { f32::from_bits(self.xEC.swap_bytes()) }
    #[inline] pub fn xF0(&self) -> f32 { f32::from_bits(self.xF0.swap_bytes()) }
    #[inline] pub fn xF4(&self) -> f32 { f32::from_bits(self.xF4.swap_bytes()) }
    #[inline] pub fn specialhi_base_angle(&self) -> f32 { f32::from_bits(self.specialhi_base_angle.swap_bytes()) }
    #[inline] pub fn xFC(&self) -> f32 { f32::from_bits(self.xFC.swap_bytes()) }
    #[inline] pub fn x100(&self) -> f32 { f32::from_bits(self.x100.swap_bytes()) }
    #[inline] pub fn x104(&self) -> f32 { f32::from_bits(self.x104.swap_bytes()) }
    #[inline] pub fn x108(&self) -> f32 { f32::from_bits(self.x108.swap_bytes()) }
    #[inline] pub fn x10C(&self) -> f32 { f32::from_bits(self.x10C.swap_bytes()) }
    #[inline] pub fn x110(&self) -> f32 { f32::from_bits(self.x110.swap_bytes()) }
}

/// The egg's throw velocity from the stick: `arg1` receives the vector.
pub fn ftYs_SpecialS_8012DF8C(gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    melee_core::ct!("ftYs_SpecialS_8012DF8C");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const YsDat);
        let mut mag: f32;
        {
            let sx = (*fp).input.lstick[0].x;
            let ax = if sx < 0.0 { -sx } else { sx };
            let temp = ax / da.xEC();
            mag = temp;
            if mag > 1.0 {
                mag = 1.0;
            }
            mag *= da.xF0();
            if mag < da.xF4() {
                mag = 0.0;
            }
            mag *= ftinl::getStickDirX(fp) as f32;
        }
        {
            let angle: f32;
            if (*fp).facing_dir == 1.0 {
                angle = da.specialhi_base_angle() - mag;
            } else {
                // M_PI is a double: the subtractions happen in double, one rounding to float
                angle = (3.14159265358979323846_f64 - da.specialhi_base_angle() as f64 - mag as f64) as f32;
            }
            let x4 = (*fp).mv.ys.specialhi.x4 as f32;
            (*arg1).x = fmadds(x4, da.x100(), da.xFC()) * ext::cosf(angle);
            (*arg1).y = fmadds(x4, da.x100(), da.xFC()) * ext::sinf(angle);
            (*arg1).z = 0.0;
        }
    }
}

/// `ftYs_SpecialS_8012DF8C_outline(gobj, vec)` (static inline).
pub fn ftYs_SpecialS_8012DF8C_outline(gobj: *mut HSD_GObj, vec: *mut Vec3) {
    melee_core::ct!("ftYs_SpecialS_8012DF8C_outline");
    ftYs_SpecialS_8012DF8C(gobj, vec);
}

pub fn fn_8012E110(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012E110");
    unsafe {
        let fp = fpp(gobj);
        let da = &*((*fp).dat_attrs as *const YsDat);
        if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
            (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
            let mut sp30 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC((*(*fp).parts.add(31)).joint, core::ptr::null_mut(), &mut sp30);
            let item = ext::it_802B2A10(gobj, &mut sp30, 0x1F, (*fp).facing_dir);
            (*fp).u.ys.x2238 = item;
            (*fp).x1984_heldItemSpec = item;
            setDamageCallbacks(gobj);
        }
        if (*fp).cmd_vars[0] != 0 && !(*fp).u.ys.x2238.is_null() {
            let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (*fp).cmd_vars[0] = 0;
            (*fp).mv.ys.specialhi.x0 = 1;
            sp24.x = da.x104() * (*fp).facing_dir;
            sp24.y = da.x108();
            sp24.z = 0.0;
            ftYs_SpecialS_8012DF8C_outline(gobj, &mut sp18);
            {
                let x4 = (*fp).mv.ys.specialhi.x4 as f32;
                ext::it_802B28C8((*fp).u.ys.x2238, &mut sp18, &mut sp24, fmadds(x4, da.x110(), da.x10C()), x4);
            }
            (*fp).u.ys.x2238 = core::ptr::null_mut();
            (*fp).take_dmg_cb = None;
            (*fp).death2_cb = None;
        }
    }
}

pub fn ftYs_SpecialS_8012E270(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialS_8012E270");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ys.x2238.is_null() && (*fp).mv.ys.specials.x0 == 0 {
            ext::it_802B2890((*fp).u.ys.x2238);
        }
        (*fp).u.ys.x2238 = core::ptr::null_mut();
        (*fp).take_dmg_cb = None;
        (*fp).death2_cb = None;
    }
}

pub fn ftYs_SpecialHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialHi_Coll");
    ext::ft_8008403C(gobj, Some(cb_fn_8012E3B4));
}

pub fn ftYs_SpecialAirHi_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirHi_Coll");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).mv.ys.specialhi.x0 != 0 {
            ext::ft_80083A48(gobj, Some(cb_fn_8012E44C));
        } else {
            ext::ft_80082C74(gobj, Some(cb_fn_8012E44C));
        }
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftYs_SpecialHi_Anim` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialHi_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).input.held_buttons[0] as u32) & 0x200u32 /* HSD_PAD_B */) != 0) {
            (*fp).mv.ys.specialhi.x4 = (*fp).mv.ys.specialhi.x4.wrapping_add(1_i32);
        }
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftYs_SpecialAirHi_Anim` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialAirHi_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirHi_Anim");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if ((((*fp).input.held_buttons[0] as u32) & 0x200u32 /* HSD_PAD_B */) != 0) {
            (*fp).mv.ys.specialhi.x4 = (*fp).mv.ys.specialhi.x4.wrapping_add(1_i32);
        }
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

/// `ftYs_SpecialHi_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialHi_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftYs_SpecialAirHi_Phys` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialAirHi_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirHi_Phys");
    st::ft_80084EEC(gobj);
}

/// `setAccessory4Callback` (static) (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn setAccessory4Callback(gobj: *mut HSD_GObj) {
    melee_core::ct!("setAccessory4Callback");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).accessory4_cb = Some(crate::chars::cb::ftyoshispecialhi__fn_8012E110);
    }
}

/// `setDamageCallbacks` (static) (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn setDamageCallbacks(gobj: *mut HSD_GObj) {
    melee_core::ct!("setDamageCallbacks");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (!(*fp).u.ys.x2238.is_null()) {
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftyoshi__ftYs_Init_8012BA8C);
            (*fp).death2_cb = Some(crate::chars::cb::ftyoshi__ftYs_Init_8012BA8C);
        }
    }
}

/// `fn_8012E44C` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn fn_8012E44C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012E44C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 364 /* 0x16C */, 0xC4C5082u32 /* motion_flags */);
        crate::chars::r4::ftYoshi::ftyoshispecialhi::setDamageCallbacks(gobj);
        crate::chars::r3::ftYoshi::ftyoshispecialhi::setAccessory4Callback(gobj);
    }
}

/// `fn_8012E3B4` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn fn_8012E3B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8012E3B4");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_GroundToAirStateChange(gobj, &mut *fp, 365 /* ftYs_MS_SpecialAirHi */, 0xC4C5082u32 /* motion_flags */);
        crate::chars::r4::ftYoshi::ftyoshispecialhi::setDamageCallbacks(gobj);
        crate::chars::r3::ftYoshi::ftyoshispecialhi::setAccessory4Callback(gobj);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}

/// `ftYs_SpecialHi_Enter` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialHi_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ys.specialhi.x0 = 0_i32;
        (*fp).mv.ys.specialhi.x4 = 0_i32;
        (*fp).u.ys.x2238 = core::mem::zeroed(); /* NULL */
        (*fp).anon_2210.throw_flags = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 364 /* ftYs_MS_SpecialHi */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftyoshispecialhi__fn_8012E110);
        st::ftAnim_8006EBA4(gobj);
    }
}

/// `ftYs_SpecialAirHi_Enter` (src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c).
pub fn ftYs_SpecialAirHi_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftYs_SpecialAirHi_Enter");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ys.specialhi.x0 = 0_i32;
        (*fp).mv.ys.specialhi.x4 = 0_i32;
        (*fp).u.ys.x2238 = core::mem::zeroed(); /* NULL */
        (*fp).anon_2210.throw_flags = 0_u32;
        (*fp).cmd_vars[0] = 0_u32;
        st::Fighter_ChangeMotionState(gobj, 365 /* ftYs_MS_SpecialAirHi */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).accessory4_cb = Some(crate::chars::cb::ftyoshispecialhi__fn_8012E110);
        st::ftAnim_8006EBA4(gobj);
    }
}
