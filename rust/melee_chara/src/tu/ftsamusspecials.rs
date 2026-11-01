//! Mechanical native port of the C file named below, generated from the C by `rust/melee_world/gen/mtu/c2rs.py`
//! (libclang AST: every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO"; `melee_link` wires them to the
//! canonical TU function). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
#[allow(unused_imports)]
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Fighter_ChangeMotionState(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32, a4: f32, a5: f32, a6: *mut HSD_GObj);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn ftAnim_8006EBA4(a0: *mut HSD_GObj);
    fn ftAnim_IsFramesRemaining(a0: *mut HSD_GObj) -> i32;
    fn ftCo_Fall_Enter(a0: *mut HSD_GObj);
    fn ftCommon_CalcSelfAccel_Deaccel(a0: *mut Fighter, a1: f32);
    fn ftCommon_FallBasic(a0: *mut Fighter);
    fn ft_80081D0C(a0: *mut HSD_GObj) -> GroundOrAir;
    fn ft_80082708(a0: *mut HSD_GObj) -> GroundOrAir;
    fn ft_800827A0(a0: *mut HSD_GObj) -> i32;
    fn ft_80082B1C(a0: *mut HSD_GObj);
    fn ft_80084F3C(a0: *mut HSD_GObj);
    fn ft_8008A2BC(a0: *mut HSD_GObj);
    }
}





/// `ftSs_SpecialS_Enter`
pub unsafe extern "C" fn ftSs_SpecialS_Enter(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_Enter");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    (*fp).gr_vel = (*fp).gr_vel / (*samus_attr).x2C;
    (*fp).self_vel.y = 0.0f32;
    if (((*fp).active_sticky.lstick.x as i32) as f32) < (*samus_attr).x28 {
        ext::Fighter_ChangeMotionState(gobj, 350, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
        ext::ftAnim_8006EBA4(gobj);
    } else {
        ext::Fighter_ChangeMotionState(gobj, 349, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
        ext::ftAnim_8006EBA4(gobj);
    }
    ftSamus_ClearThrowFlagsUnk(gobj);
}

/// `ftSs_SpecialAirS_Enter`
pub unsafe extern "C" fn ftSs_SpecialAirS_Enter(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirS_Enter");
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    (*fp).self_vel.x = (*fp).self_vel.x / (*samus_attr).x2C;
    if (((*fp).active_sticky.lstick.x as i32) as f32) < (*samus_attr).x28 {
        ext::Fighter_ChangeMotionState(gobj, 352, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
        ext::ftAnim_8006EBA4(gobj);
    } else {
        ext::Fighter_ChangeMotionState(gobj, 351, 0 as u32, 0.0f32, 1.0f32, 0.0f32, null_mut());
        ext::ftAnim_8006EBA4(gobj);
    }
    ftSamus_ClearThrowFlagsUnk(gobj);
}





/// `ftSs_SpecialS_IASA`
pub unsafe extern "C" fn ftSs_SpecialS_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_IASA");
}

/// `ftSs_SpecialAirS_IASA`
pub unsafe extern "C" fn ftSs_SpecialAirS_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirS_IASA");
}



/// `ftSs_SpecialAirS_Phys`
pub unsafe extern "C" fn ftSs_SpecialAirS_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirS_Phys");
    let mut __: [u8; 24] = [0; 24];
    let mut fp: *mut Fighter = HSD_GObjGetUserData(gobj) as *mut Fighter;
    let mut samus_attr: *mut _ftSamusAttributes = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    let mut fighter2: *mut Fighter = null_mut();
    ext::ftCommon_FallBasic(fp);
    fighter2 = fp;
    ext::ftCommon_CalcSelfAccel_Deaccel(fighter2, (*samus_attr).x30);
}



/// `ftSs_SpecialAirS_Coll`
pub unsafe extern "C" fn ftSs_SpecialAirS_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirS_Coll");
    if ext::ft_80081D0C(gobj) != 0 {
        ext::ft_80082B1C(gobj);
    }
}





/// `ftSs_SpecialSSmash_IASA`
pub unsafe extern "C" fn ftSs_SpecialSSmash_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialSSmash_IASA");
}

/// `ftSs_SpecialAirSSmash_IASA`
pub unsafe extern "C" fn ftSs_SpecialAirSSmash_IASA(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirSSmash_IASA");
}



/// `ftSs_SpecialAirSSmash_Phys`
pub unsafe extern "C" fn ftSs_SpecialAirSSmash_Phys(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirSSmash_Phys");
    let mut __: [u8; 40] = [0; 40];
    let mut fighter2: *mut Fighter = null_mut();
    let mut fp: *mut Fighter = null_mut();
    let mut samus_attr: *mut _ftSamusAttributes = null_mut();
    let mut samus_attr2: *mut _ftSamusAttributes = null_mut();
    fp = { fighter2 = HSD_GObjGetUserData(gobj) as *mut Fighter; fighter2 };
    samus_attr = getFtSpecialAttrs(fp) as *mut _ftSamusAttributes;
    ext::ftCommon_FallBasic(fp);
    samus_attr2 = samus_attr;
    ext::ftCommon_CalcSelfAccel_Deaccel(fp, (*samus_attr2).x30);
}



/// `ftSs_SpecialAirSSmash_Coll`
pub unsafe extern "C" fn ftSs_SpecialAirSSmash_Coll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirSSmash_Coll");
    if ext::ft_80081D0C(gobj) != 0 {
        ext::ft_80082B1C(gobj);
    }
}



/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `getFtSpecialAttrs`
pub unsafe extern "C" fn getFtSpecialAttrs(mut fp: *mut Fighter) -> *mut c_void {
    melee_core::ct!("getFtSpecialAttrs");
    let mut fighter_attr: *mut c_void = (*fp).dat_attrs;
    return fighter_attr;
}

/// Oracle-checked hand ports (rounds 2-12, tests/chars_dispatch.rs), re-exported under this TU.

// ---- folded from chars (consolidation step 20/21) ----
/// `ftSs_SpecialS_Anim` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftSs_SpecialAirS_Anim` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftSs_SpecialS_Phys` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSs_SpecialS_Coll` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_Coll");
    if st::ft_800827A0(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftSs_SpecialSSmash_Anim` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialSSmash_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialSSmash_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftSs_SpecialAirSSmash_Anim` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialAirSSmash_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialAirSSmash_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftSs_SpecialSSmash_Phys` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialSSmash_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialSSmash_Phys");
    st::ft_80084F3C(gobj);
}

/// `ftSs_SpecialSSmash_Coll` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialSSmash_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialSSmash_Coll");
    if st::ft_80082708(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftSs_SpecialS_8012A640` (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSs_SpecialS_8012A640(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSs_SpecialS_8012A640");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::efLib_DestroyAll(gobj);
        (*fp).u.ss.x2244 = 0_u32;
    }
}

/// `ftSamus_ClearThrowFlagsUnk` (static) (src/melee/ft/kinds/ftSamus/ftsamusspecials.c).
pub fn ftSamus_ClearThrowFlagsUnk(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftSamus_ClearThrowFlagsUnk");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        (*fp).anon_2210.throw_flags = 0_u32;
        (*fp).accessory4_cb = Some(crate::chars::cb::ftsamusspecialn__ftSs_SpecialS_8012A074);
    }
}
