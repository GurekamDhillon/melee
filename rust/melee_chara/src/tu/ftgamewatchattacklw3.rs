//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattacklw3.c: Mr. Game & Watch's down tilt and the
//! Manhole item it spawns. No float arithmetic, so no fused sites (tools/rust-port/fma_sites.py).
//! Smoke-tested by tests/tu_ftgamewatchattacklw3.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{ftGw_MS_AttackLw3, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext, ftGw_Init_8014A538};
use melee_fighter::tu::{
    ftCo_Attack1, ftCo_AttackHi3, ftCo_AttackHi4, ftCo_AttackLw3, ftCo_AttackLw4, ftCo_AttackS3, ftCo_AttackS4,
    ftCo_Dash, ftCo_Jump, ftCo_Squat, ftCo_SquatWait, ftCo_Turn, ftCo_Walk,
};
use melee_types::*;

/// Already ported (rounds 2 and the item-check helper): re-exported under this TU.



/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftgamewatch::ext::{lb_8000B1CC, LB_8000B1CC};
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
    // it/it_26B1.c
    pub static mut IT_8026BB44: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026BB44(item: *mut HSD_GObj)  {
                match unsafe { IT_8026BB44 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026BB44(item as _); } return; },
                }
            }
    pub static mut IT_8026B724: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B724(item: *mut HSD_GObj)  {
                match unsafe { IT_8026B724 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B724(item as _); } return; },
                }
            }
    pub static mut IT_8026BB20: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026BB20(item: *mut HSD_GObj)  {
                match unsafe { IT_8026BB20 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026BB20(item as _); } return; },
                }
            }
    pub static mut IT_8026B73C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_8026B73C(item: *mut HSD_GObj)  {
                match unsafe { IT_8026B73C } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::it_26B1::it_8026B73C(item as _); } return; },
                }
            }
    // itgamewatchmanhole.c
    pub static mut IT_802C65E4: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C65E4(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C65E4 } {
                    Some(f) => f(owner, pos, part, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchmanhole::it_802C65E4(owner as _, pos as _, part as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802C6718: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6718(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6718 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchmanhole::it_802C6718(item as _); } return; },
                }
            }
    pub static mut IT_802C6764: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6764(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6764 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchmanhole::it_802C6764(item as _); } return; },
                }
            }
    pub static mut IT_802C6784: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6784(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6784 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchmanhole::it_802C6784(item as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftGw_AttackLw3_ItemManholeSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_AttackLw3_ItemManholeSetup(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_AttackLw3_ItemManholeSetup");
    ftGw_AttackLw3_ItemManholeSetup(g);
}
/// `ftGw_AttackLw3_ItemManholeEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_AttackLw3_ItemManholeEnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_AttackLw3_ItemManholeEnterHitlag");
    ftGw_AttackLw3_ItemManholeEnterHitlag(g);
}
/// `ftGw_AttackLw3_ItemManholeExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_AttackLw3_ItemManholeExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_AttackLw3_ItemManholeExitHitlag");
    ftGw_AttackLw3_ItemManholeExitHitlag(g);
}

/// Create the Manhole item (swapping out a held item first).
pub fn ftGw_AttackLw3_ItemManholeSetup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_ItemManholeSetup");
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if (*fp).u.gw.x2250_manholeGObj2.is_null() {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandNb as usize)).joint, core::ptr::null(), &mut sp10);
            let manhole_gobj = (*fp).item_gobj;
            if !manhole_gobj.is_null() {
                (*fp).u.gw.x2248_manholeGObj = manhole_gobj;
                ext::it_8026BB44((*fp).item_gobj);
                ext::it_8026B724((*fp).item_gobj);
                melee_fighter::ftcommon3::ftcommon_8007e6dc(gobj, (*fp).item_gobj, 1);
            }
            (*fp).u.gw.x2250_manholeGObj2 =
                ext::it_802C65E4(gobj, &sp10, FtPart_LHandNb as i32, (*fp).facing_dir);
        }
        if !(*fp).u.gw.x2250_manholeGObj2.is_null() {
            if (*fp).death2_cb.is_none() {
                (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
            }
            if (*fp).take_dmg_cb.is_none() {
                (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
            }
        }
        (*fp).pre_hitlag_cb = Some(cb_ftGw_AttackLw3_ItemManholeEnterHitlag);
        (*fp).post_hitlag_cb = Some(cb_ftGw_AttackLw3_ItemManholeExitHitlag);
        (*fp).accessory4_cb = None;
    }
}

/// Swap the held item's GObj pointer back.
pub fn ftGw_AttackLw3_ItemManholeRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_ItemManholeRemove");
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackLw3_ItemManholeExitHitlag(gobj);
        (*fp).u.gw.x2250_manholeGObj2 = core::ptr::null_mut();
        let manhole_gobj = (*fp).u.gw.x2248_manholeGObj;
        if !manhole_gobj.is_null() {
            (*fp).item_gobj = manhole_gobj;
            (*fp).u.gw.x2248_manholeGObj = core::ptr::null_mut();
            ext::it_8026BB20((*fp).item_gobj);
            ext::it_8026B73C((*fp).item_gobj);
            melee_fighter::tu::ftpickupitem::ftpickupitem_80094818(gobj, true);
        }
    }
}

/// Remove the Manhole on damage.
pub fn ftGw_AttackLw3_ItemManholeOnDamage(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_ItemManholeOnDamage");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2250_manholeGObj2.is_null() {
            ext::it_802C6718((*fp).u.gw.x2250_manholeGObj2);
            ftGw_AttackLw3_ItemManholeRemove(gobj);
        }
    }
}

/// Apply hitlag to the Manhole item (static in the C).
pub fn ftGw_AttackLw3_ItemManholeEnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_ItemManholeEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2250_manholeGObj2.is_null() {
            ext::it_802C6764((*fp).u.gw.x2250_manholeGObj2);
        }
    }
}

/// Remove hitlag from the Manhole item (static in the C).
pub fn ftGw_AttackLw3_ItemManholeExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_ItemManholeExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2250_manholeGObj2.is_null() {
            ext::it_802C6784((*fp).u.gw.x2250_manholeGObj2);
        }
    }
}

/// Mr. Game & Watch's Down Tilt Motion State handler.
pub fn ftGw_AttackLw3_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_Enter");
    unsafe {
        let fp = fpp(gobj);
        if !melee_fighter::tu::ftpickupitem::ftpickupitem_80094790(gobj) {
            (*fp).set_allow_interrupt(0);
            st::Fighter_ChangeMotionState(gobj, ftGw_MS_AttackLw3 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).accessory4_cb = Some(cb_ftGw_AttackLw3_ItemManholeSetup);
        }
    }
}

/// Down Tilt Animation callback.
pub fn ftGw_AttackLw3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCo_SquatWait::ftCo_800D638C(gobj);
    }
}

/// Down Tilt IASA callback: the first input check that fires wins (`RETURN_IF`).
pub fn ftGw_AttackLw3_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_IASA");
    unsafe {
        let fp = fpp(gobj);
        if (*fp).allow_interrupt() != 0 {
            if ftCo_AttackS4::ftCo_AttackS4_CheckInput(gobj) {
                return;
            }
            if ftCo_AttackHi4::ftCo_AttackHi4_CheckInput(gobj) {
                return;
            }
            if ftCo_AttackLw4::ftCo_AttackLw4_CheckInput(gobj) {
                return;
            }
            if ftCo_AttackS3::ftCo_AttackS3_CheckInput(gobj) {
                return;
            }
            if ftCo_AttackHi3::ftCo_AttackHi3_CheckInput(gobj) {
                return;
            }
            if ftCo_AttackLw3::ftCo_AttackLw3_CheckInput(gobj) {
                return;
            }
            if ftCo_Attack1::ftCo_Attack1_CheckInput(gobj) {
                return;
            }
            if ftCo_Jump::ftCo_Jump_CheckInput(gobj) {
                return;
            }
            if ftCo_Dash::ftCo_Dash_CheckInput(gobj) {
                return;
            }
            if ftCo_Squat::ftCo_Squat_CheckInput(gobj) {
                return;
            }
            if ftCo_Turn::ftCo_Turn_CheckInput(gobj) {
                return;
            }
            if ftCo_Walk::ftCo_Walk_CheckInput(gobj) {
                return;
            }
        }
    }
}

/// Down Tilt Collision callback.
pub fn ftGw_AttackLw3_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_Coll");
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftGw_AttackLw3_ItemCheckManholeRemove(fp: &Fighter) -> bool {
    melee_core::ct!("ftGw_AttackLw3_ItemCheckManholeRemove");
    fp.motion_id as u32 != ftGw_MS_AttackLw3 as u32
}

/// `ftGw_AttackLw3_Phys` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattacklw3.c).
pub fn ftGw_AttackLw3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_AttackLw3_Phys");
    st::ft_80084F3C(gobj);
}
