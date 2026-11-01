//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattack11.c: Mr. Game & Watch's jab and the
//! Insecticide Sprayer (greenhouse) item it spawns. No float arithmetic, so no fused sites
//! (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatchattack11.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{ftGw_MS_Attack100End, ftGw_MS_Attack11, Fighter};
#[allow(unused_imports)]
use melee_types::{HSD_GObj};
use crate::stubs as st;
use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext, ftGw_Init_8014A538};
use melee_fighter::tu::ftCo_Attack1 as co;
use melee_types::*;

/// Already ported (round 1 helpers, rounds 2/ftGameWatch.rs): re-exported under this TU.



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
    pub static mut ITGAMEWATCHGREENHOUSE_SPAWN: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_Spawn(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { ITGAMEWATCHGREENHOUSE_SPAWN } {
                    Some(f) => f(owner, pos, part, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_Spawn(owner as _, pos as _, part as _, facing as _) as _ }; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6430: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6430(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6430 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6430(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6458: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6458(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6458 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6458(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6480: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6480(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6480 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6480(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C64A8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C64A8(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C64A8 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C64A8(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6328: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6328(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6328 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6328(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6374: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6374(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6374 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6374(item as _); } return; },
                }
            }
    pub static mut ITGAMEWATCHGREENHOUSE_802C6394: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn itGamewatchGreenhouse_802C6394(item: *mut HSD_GObj)  {
                match unsafe { ITGAMEWATCHGREENHOUSE_802C6394 } {
                    Some(f) => f(item),
                    None => { unsafe { melee_items::tu::itgamewatchgreenhouse::itGamewatchGreenhouse_802C6394(item as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftGw_Attack11_ItemGreenhouseSetup` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_Attack11_ItemGreenhouseSetup(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_Attack11_ItemGreenhouseSetup");
    ftGw_Attack11_ItemGreenhouseSetup(g);
}
/// `ftGw_Attack11_DecideAction` as a Fighter callback entry (accessory4_cb).
pub unsafe extern "C" fn cb_ftGw_Attack11_DecideAction(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_Attack11_DecideAction");
    ftGw_Attack11_DecideAction(g);
}
/// `ftGw_Attack11_ItemGreenhouseEnterHitlag` as a Fighter callback entry (pre_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_Attack11_ItemGreenhouseEnterHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_Attack11_ItemGreenhouseEnterHitlag");
    ftGw_Attack11_ItemGreenhouseEnterHitlag(g);
}
/// `ftGw_Attack11_ItemGreenhouseExitHitlag` as a Fighter callback entry (post_hitlag_cb).
pub unsafe extern "C" fn cb_ftGw_Attack11_ItemGreenhouseExitHitlag(g: *mut HSD_GObj) {
    melee_core::ct!("cb_ftGw_Attack11_ItemGreenhouseExitHitlag");
    ftGw_Attack11_ItemGreenhouseExitHitlag(g);
}

/// Create the Insecticide Spray item.
pub fn ftGw_Attack11_ItemGreenhouseSetup(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouseSetup");
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);

        if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
            ftGw_Attack11_DecideAction(gobj);
        } else {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandNb as usize)).joint, core::ptr::null(), &mut sp10);
            (*fp).u.gw.x224C_greenhouseGObj =
                ext::itGamewatchGreenhouse_Spawn(gobj, &sp10, FtPart_LHandNb as i32, (*fp).facing_dir);
            if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
                if (*fp).death2_cb.is_none() {
                    (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
                }
                if (*fp).take_dmg_cb.is_none() {
                    (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
                }
                (*fp).pre_hitlag_cb = Some(cb_ftGw_Attack11_ItemGreenhouseEnterHitlag);
                (*fp).post_hitlag_cb = Some(cb_ftGw_Attack11_ItemGreenhouseExitHitlag);
            }
        }
        (*fp).accessory4_cb = None;
    }
}

/// Decide the jab attack: tell the sprayer which jab state G&W is in.
pub fn ftGw_Attack11_DecideAction(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_DecideAction");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
            let msid = (*fp).motion_id;
            let item = (*fp).u.gw.x224C_greenhouseGObj;
            if msid == ftGw_MS_Attack11 as FtMotionId {
                ext::itGamewatchGreenhouse_802C6430(item);
            } else if msid == ftGw_MS_Attack100Start as FtMotionId {
                ext::itGamewatchGreenhouse_802C6458(item);
            } else if msid == ftGw_MS_Attack100Loop as FtMotionId {
                ext::itGamewatchGreenhouse_802C6480(item);
            } else if msid == ftGw_MS_Attack100End as FtMotionId {
                ext::itGamewatchGreenhouse_802C64A8(item);
            }

            if (*fp).death2_cb.is_none() {
                (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
            }
            if (*fp).take_dmg_cb.is_none() {
                (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
            }
            (*fp).pre_hitlag_cb = Some(cb_ftGw_Attack11_ItemGreenhouseEnterHitlag);
            (*fp).post_hitlag_cb = Some(cb_ftGw_Attack11_ItemGreenhouseExitHitlag);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Set flags on the Insecticide Sprayer (clear the pointer after ending its hitlag).
pub fn ftGw_Attack11_ItemGreenhouseSetFlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouseSetFlag");
    unsafe {
        let fp = fpp(gobj);
        ftGw_Attack11_ItemGreenhouseExitHitlag(gobj);
        (*fp).u.gw.x224C_greenhouseGObj = core::ptr::null_mut();
    }
}

/// Remove the Insecticide Sprayer.
pub fn ftGw_Attack11_ItemGreenhouseRemove(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouseRemove");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
            ext::itGamewatchGreenhouse_802C6328((*fp).u.gw.x224C_greenhouseGObj);
            ftGw_Attack11_ItemGreenhouseSetFlag(gobj);
        }
    }
}

/// Apply hitlag to the Insecticide Sprayer.
pub fn ftGw_Attack11_ItemGreenhouseEnterHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouseEnterHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
            ext::itGamewatchGreenhouse_802C6374((*fp).u.gw.x224C_greenhouseGObj);
        }
    }
}

/// Remove hitlag from the Insecticide Sprayer.
pub fn ftGw_Attack11_ItemGreenhouseExitHitlag(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouseExitHitlag");
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x224C_greenhouseGObj.is_null() {
            ext::itGamewatchGreenhouse_802C6394((*fp).u.gw.x224C_greenhouseGObj);
        }
    }
}

/// Mr. Game & Watch's Jab Motion State handler.
pub fn ftGw_Attack11_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_Enter");
    unsafe {
        let fp = fpp(gobj);
        if !melee_fighter::tu::ftpickupitem::ftpickupitem_80094790(gobj) {
            (*fp).set_allow_interrupt(0);
            (*fp).set_x2218_b1(0);
            st::Fighter_ChangeMotionState(gobj, ftGw_MS_Attack11 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
            (*fp).hitlag_mul = (*fp).co_attrs.jab_2_input_window;
            (*fp).unk_msid = 44;
            (*fp).set_x2218_b2(0);
            (*fp).mv.gw.Attack11.unk = 0;
            (*fp).x1A54 = 0;
            (*fp).accessory4_cb = Some(cb_ftGw_Attack11_ItemGreenhouseSetup);
        }
    }
}

/// Mr. Game & Watch's Jab IASA callback.
pub fn ftGw_Attack11_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_IASA");
    co::ftCo_Attack11_IASA(gobj);
}

/// Mr. Game & Watch's Jab Collision callback.
pub fn ftGw_Attack11_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_Coll");
    st::ft_80084104(gobj);
    ftGw_Init_8014A538(gobj);
}

// ---- folded from chars (consolidation step 20/21) ----
pub fn ftGw_Attack11_ItemGreenhouse_CheckAll(fp: &Fighter) -> bool {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouse_CheckAll");
    crate::chars::ftGameWatch::outside(fp, ftGw_MS_Attack11 as u32, ftGw_MS_Attack100End as u32)
}

pub fn ftGw_Attack11_ItemGreenhouse_CheckAttack11(fp: &Fighter) -> bool {
    melee_core::ct!("ftGw_Attack11_ItemGreenhouse_CheckAttack11");
    fp.motion_id as u32 != ftGw_MS_Attack11 as u32
}

/// `ftGw_Attack11_Anim` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack11.c).
pub fn ftGw_Attack11_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftGw_Attack11_Phys` (src/melee/ft/kinds/ftGameWatch/ftgamewatchattack11.c).
pub fn ftGw_Attack11_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftGw_Attack11_Phys");
    st::ft_80084FA8(gobj);
}
