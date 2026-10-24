//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattackair.c: Mr. Game & Watch's neutral / back / up
//! aerials (parachute, turtle and sparky items) and their landings. No float arithmetic, so no
//! fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatchattackair.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::ftgamewatch::{cb_ftGw_Init_OnDamage, ext as gw_ext, ftGw_Init_OnDamage};
use melee_fighter::tu::{ftCo_AttackAir as aa, ftCo_Landing as land, ftCo_LandingAir as landair};
use melee_types::*;

/// Already ported (rounds 2/5/12 and the item-check helpers): re-exported under this TU.
pub use crate::chars::ftGameWatch::ftgamewatchattackair::{
    ftGw_AttackAirN_ItemCheckParachuteRemove, ftGw_AttackAirN_ItemCheckSparkyRemove,
    ftGw_AttackAirN_ItemCheckTurtleRemove,
};
pub use crate::chars::r12::ftGameWatch::ftgamewatchattackair::{
    ftGw_LandingAirB_IASA, ftGw_LandingAirHi_IASA, ftGw_LandingAirN_IASA,
};
pub use crate::chars::r2::ftGameWatch::ftgamewatchattackair::{
    ftGw_AttackAirB_Phys, ftGw_AttackAirHi_Phys, ftGw_AttackAirN_Phys,
};
pub use crate::chars::r5::ftGameWatch::ftgamewatchattackair::{
    ftGw_AttackAirB_Coll, ftGw_AttackAirHi_Coll, ftGw_AttackAirN_Coll,
};

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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // itgamewatchparachute.c
    pub static mut IT_802C6C38: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C6C38(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C6C38 } {
                    Some(f) => f(owner, pos, part, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchparachute::it_802C6C38(owner as _, pos as _, part as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802C6D6C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6D6C(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6D6C } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchparachute::it_802C6D6C(item as _); } return; },
                }
            }
    pub static mut IT_802C6DB8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6DB8(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6DB8 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchparachute::it_802C6DB8(item as _); } return; },
                }
            }
    pub static mut IT_802C6DD8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6DD8(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6DD8 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchparachute::it_802C6DD8(item as _); } return; },
                }
            }
    pub static mut IT_802C6E50: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C6E50(item: *mut HSD_GObj)  {
                match unsafe { IT_802C6E50 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchparachute::it_802C6E50(item as _); } return; },
                }
            }
    // itgamewatchturtle.c
    pub static mut IT_802C6F40: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C6F40(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C6F40 } {
                    Some(f) => f(owner, pos, part, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchturtle::it_802C6F40(owner as _, pos as _, part as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802C7074: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7074(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7074 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchturtle::it_802C7074(item as _); } return; },
                }
            }
    pub static mut IT_802C70C0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C70C0(item: *mut HSD_GObj)  {
                match unsafe { IT_802C70C0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchturtle::it_802C70C0(item as _); } return; },
                }
            }
    pub static mut IT_802C70E0: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C70E0(item: *mut HSD_GObj)  {
                match unsafe { IT_802C70E0 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchturtle::it_802C70E0(item as _); } return; },
                }
            }
    pub static mut IT_802C7158: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7158(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7158 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchturtle::it_802C7158(item as _); } return; },
                }
            }
    // itgamewatchbreath.c (sparky)
    pub static mut IT_802C720C: Option<fn(*mut HSD_GObj, *const Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C720C(owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802C720C } {
                    Some(f) => f(owner, pos, part, facing),
                    None => { return unsafe { melee_items::tu::itgamewatchbreath::it_802C720C(owner as _, pos as _, part as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802C7340: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7340(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7340 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchbreath::it_802C7340(item as _); } return; },
                }
            }
    pub static mut IT_802C738C: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C738C(item: *mut HSD_GObj)  {
                match unsafe { IT_802C738C } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchbreath::it_802C738C(item as _); } return; },
                }
            }
    pub static mut IT_802C73AC: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C73AC(item: *mut HSD_GObj)  {
                match unsafe { IT_802C73AC } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchbreath::it_802C73AC(item as _); } return; },
                }
            }
    pub static mut IT_802C7424: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7424(item: *mut HSD_GObj)  {
                match unsafe { IT_802C7424 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchbreath::it_802C7424(item as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

macro_rules! cb_entry {
    ($($(#[$m:meta])* $cb:ident => $f:ident;)*) => {
        $( $(#[$m])* pub unsafe extern "C" fn $cb(g: *mut HSD_GObj) { $f(g); } )*
    };
}
cb_entry! {
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemParachuteSetup => ftGw_AttackAirN_ItemParachuteSetup;
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemParachuteOnLand => ftGw_AttackAirN_ItemParachuteOnLand;
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemTurtleSetup => ftGw_AttackAirN_ItemTurtleSetup;
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemTurtleOnLand => ftGw_AttackAirN_ItemTurtleOnLand;
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemSparkySetup => ftGw_AttackAirN_ItemSparkySetup;
    /// accessory4_cb entry.
    cb_ftGw_AttackAirN_ItemSparkyOnLand => ftGw_AttackAirN_ItemSparkyOnLand;
    /// pre_hitlag_cb entry.
    cb_ftGw_AttackAirN_EnterItemHitlag => ftGw_AttackAirN_EnterItemHitlag;
    /// post_hitlag_cb entry.
    cb_ftGw_AttackAirN_ExitItemHitlag => ftGw_AttackAirN_ExitItemHitlag;
    /// ft_80082C74 landing callback.
    cb_ftGw_LandingAirN_Enter => ftGw_LandingAirN_Enter;
    /// ft_80082C74 landing callback.
    cb_ftGw_LandingAirB_Enter => ftGw_LandingAirB_Enter;
    /// ft_80082C74 landing callback.
    cb_ftGw_LandingAirHi_Enter => ftGw_LandingAirHi_Enter;
}

/// The callbacks every aerial item installs.
#[inline(always)]
unsafe fn set_item_callbacks(fp: *mut Fighter) {
    (*fp).death2_cb = Some(cb_ftGw_Init_OnDamage);
    (*fp).take_dmg_cb = Some(cb_ftGw_Init_OnDamage);
    (*fp).pre_hitlag_cb = Some(cb_ftGw_AttackAirN_EnterItemHitlag);
    (*fp).post_hitlag_cb = Some(cb_ftGw_AttackAirN_ExitItemHitlag);
}

/// Create the Parachute item.
pub fn ftGw_AttackAirN_ItemParachuteSetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ftGw_AttackAirN_ItemParachuteOnLand(gobj);
        } else {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TransN as usize)).joint, core::ptr::null(), &mut sp10);
            (*fp).u.gw.x2258_parachuteGObj = ext::it_802C6C38(gobj, &sp10, FtPart_TransN as i32, (*fp).facing_dir);
            if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
                set_item_callbacks(fp);
            }
        }
        (*fp).accessory4_cb = None;
    }
}

/// Parachute accessory callback: landing lag.
pub fn ftGw_AttackAirN_ItemParachuteOnLand(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ftGw_AttackAirN_ExitItemHitlag(gobj);
            if (*fp).motion_id == ftGw_MS_LandingAirN as FtMotionId {
                ext::it_802C6E50((*fp).u.gw.x2258_parachuteGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Parachute item flag update on land + clear pointers.
pub fn ftGw_AttackAirN_ItemParachuteSetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x2258_parachuteGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove the Parachute item.
pub fn ftGw_AttackAirN_ItemParachuteRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ext::it_802C6D6C((*fp).u.gw.x2258_parachuteGObj);
            ftGw_AttackAirN_ItemParachuteSetFlag(gobj);
        }
    }
}

/// Create the Turtle item.
pub fn ftGw_AttackAirN_ItemTurtleSetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ftGw_AttackAirN_ItemTurtleOnLand(gobj);
        } else {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LShoulderN as usize)).joint, core::ptr::null(), &mut sp10);
            (*fp).u.gw.x225C_turtleGObj = ext::it_802C6F40(gobj, &sp10, FtPart_LShoulderN as i32, (*fp).facing_dir);
            if !(*fp).u.gw.x225C_turtleGObj.is_null() {
                set_item_callbacks(fp);
            }
        }
        (*fp).accessory4_cb = None;
    }
}

/// Turtle accessory callback: landing lag. The motion id compared is LandingAirN's on purpose
/// (the C notes a deliberate mismatch to keep the item animation from freezing).
pub fn ftGw_AttackAirN_ItemTurtleOnLand(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ftGw_AttackAirN_ExitItemHitlag(gobj);
            if (*fp).motion_id == ftGw_MS_LandingAirN as FtMotionId {
                ext::it_802C7158((*fp).u.gw.x225C_turtleGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Turtle item flag update on land + clear pointers.
pub fn ftGw_AttackAirN_ItemTurtleSetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x225C_turtleGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove the Turtle item.
pub fn ftGw_AttackAirN_ItemTurtleRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ext::it_802C7074((*fp).u.gw.x225C_turtleGObj);
            ftGw_AttackAirN_ItemTurtleSetFlag(gobj);
        }
    }
}

/// Create the Sparky item.
pub fn ftGw_AttackAirN_ItemSparkySetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ftGw_AttackAirN_ItemSparkyOnLand(gobj);
        } else {
            gw_ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandN as usize)).joint, core::ptr::null(), &mut sp10);
            (*fp).u.gw.x2260_sparkyGObj = ext::it_802C720C(gobj, &sp10, FtPart_LHandN as i32, (*fp).facing_dir);
            if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
                set_item_callbacks(fp);
            }
        }
        (*fp).accessory4_cb = None;
    }
}

/// Sparky accessory callback: landing lag.
pub fn ftGw_AttackAirN_ItemSparkyOnLand(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ftGw_AttackAirN_ExitItemHitlag(gobj);
            if (*fp).motion_id == ftGw_MS_LandingAirN as FtMotionId {
                ext::it_802C7424((*fp).u.gw.x2260_sparkyGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Sparky item flag update on land + clear pointers.
pub fn ftGw_AttackAirN_ItemSparkySetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x2260_sparkyGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove the Sparky item.
pub fn ftGw_AttackAirN_ItemSparkyRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ext::it_802C7340((*fp).u.gw.x2260_sparkyGObj);
            ftGw_AttackAirN_ItemSparkySetFlag(gobj);
        }
    }
}

/// Apply hitlag for the available aerial items.
pub fn ftGw_AttackAirN_EnterItemHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ext::it_802C6DB8((*fp).u.gw.x2258_parachuteGObj);
        }
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ext::it_802C70C0((*fp).u.gw.x225C_turtleGObj);
        }
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ext::it_802C738C((*fp).u.gw.x2260_sparkyGObj);
        }
    }
}

/// Remove hitlag for the available aerial items (static in the C).
pub fn ftGw_AttackAirN_ExitItemHitlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ext::it_802C6DD8((*fp).u.gw.x2258_parachuteGObj);
        }
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ext::it_802C70E0((*fp).u.gw.x225C_turtleGObj);
        }
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ext::it_802C73AC((*fp).u.gw.x2260_sparkyGObj);
        }
    }
}

/// Decide G&W's aerial attack motion state from the C stick.
pub fn ftGw_AttackAirN_DecideAction(gobj: *mut HSD_GObj) {
    unsafe {
        let msid = aa::ftCo_AttackAir_GetMsidFromCStick(&*fpp(gobj));
        match msid {
            m if m == ftCo_MS_AttackAirN as i32 => ftGw_AttackAirN_Enter(gobj),
            m if m == ftCo_MS_AttackAirF as i32 => aa::ftCo_AttackAir_EnterFromMsid(gobj, msid),
            m if m == ftCo_MS_AttackAirB as i32 => ftGw_AttackAirB_Enter(gobj),
            m if m == ftCo_MS_AttackAirHi as i32 => ftGw_AttackAirHi_Enter(gobj),
            m if m == ftCo_MS_AttackAirLw as i32 => aa::ftCo_AttackAir_EnterFromMsid(gobj, msid),
            _ => {}
        }
    }
}

/// Neutral aerial Action State handler.
pub fn ftGw_AttackAirN_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        aa::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirN as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemParachuteSetup);
    }
}

/// Neutral aerial Animation callback.
pub fn ftGw_AttackAirN_Anim(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAir_Anim(gobj);
}

/// Neutral aerial IASA callback.
pub fn ftGw_AttackAirN_IASA(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAirN_IASA(gobj);
}

/// The three LandingAir*_Enter bodies differ only in the item OnLand callback and the Init.
#[inline(always)]
unsafe fn landing_enter(gobj: *mut HSD_GObj, init: fn(*mut HSD_GObj), on_land: unsafe extern "C" fn(*mut HSD_GObj)) {
    let fp = fpp(gobj);
    if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
        ext::it_802C6DD8((*fp).u.gw.x2258_parachuteGObj);
    }
    if !(*fp).u.gw.x225C_turtleGObj.is_null() {
        ext::it_802C70E0((*fp).u.gw.x225C_turtleGObj);
    }
    if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
        ext::it_802C73AC((*fp).u.gw.x2260_sparkyGObj);
    }
    if (*fp).cmd_vars[0] != 0 {
        init(gobj);
        (*fp).accessory4_cb = Some(on_land);
        return;
    }
    land::ftCo_Landing_Enter_Basic(gobj);
    ftGw_Init_OnDamage(gobj);
}

/// Neutral aerial landing think.
pub fn ftGw_LandingAirN_Enter(gobj: *mut HSD_GObj) {
    unsafe { landing_enter(gobj, ftGw_LandingAirN_Init, cb_ftGw_AttackAirN_ItemParachuteOnLand) }
}

/// Back aerial Action State handler.
pub fn ftGw_AttackAirB_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        aa::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirB as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemTurtleSetup);
    }
}

/// Back aerial Animation callback.
pub fn ftGw_AttackAirB_Anim(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAir_Anim(gobj);
}

/// Back aerial IASA callback.
pub fn ftGw_AttackAirB_IASA(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAirB_IASA(gobj);
}

/// Back aerial landing think.
pub fn ftGw_LandingAirB_Enter(gobj: *mut HSD_GObj) {
    unsafe { landing_enter(gobj, ftGw_LandingAirB_Init, cb_ftGw_AttackAirN_ItemTurtleOnLand) }
}

/// Up aerial Motion State handler.
pub fn ftGw_AttackAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        aa::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirHi as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemSparkySetup);
    }
}

/// Up aerial Animation callback.
pub fn ftGw_AttackAirHi_Anim(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAir_Anim(gobj);
}

/// Up aerial IASA callback.
pub fn ftGw_AttackAirHi_IASA(gobj: *mut HSD_GObj) {
    aa::ftCo_AttackAirHi_IASA(gobj);
}

/// Up aerial landing think.
pub fn ftGw_LandingAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe { landing_enter(gobj, ftGw_LandingAirHi_Init, cb_ftGw_AttackAirN_ItemSparkyOnLand) }
}

/// Neutral aerial landing prefunction (static in the C).
pub fn ftGw_LandingAirN_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        landair::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirN as i32, (*fp).co_attrs.landingairn_lag);
    }
}

/// Run the common landing callback, then drop the items if the landing state was left.
#[inline(always)]
unsafe fn landing_check(gobj: *mut HSD_GObj, msid: u32) {
    let fp = fpp(gobj);
    if (*fp).motion_id != msid as FtMotionId {
        ftGw_Init_OnDamage(gobj);
    }
}

/// Neutral aerial landing Animation callback.
pub fn ftGw_LandingAirN_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Anim(gobj);
        landing_check(gobj, ftGw_MS_LandingAirN);
    }
}

/// Neutral aerial landing Physics callback.
pub fn ftGw_LandingAirN_Phys(gobj: *mut HSD_GObj) {
    land::ftCo_Landing_Phys(gobj);
}

/// Neutral aerial landing Collision callback.
pub fn ftGw_LandingAirN_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Coll(gobj);
        landing_check(gobj, ftGw_MS_LandingAirN);
    }
}

/// Back aerial landing prefunction.
pub fn ftGw_LandingAirB_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        landair::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirB as i32, (*fp).co_attrs.landingairb_lag);
    }
}

/// Back aerial landing Animation callback.
pub fn ftGw_LandingAirB_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Anim(gobj);
        landing_check(gobj, ftGw_MS_LandingAirB);
    }
}

/// Back aerial landing Physics callback.
pub fn ftGw_LandingAirB_Phys(gobj: *mut HSD_GObj) {
    land::ftCo_Landing_Phys(gobj);
}

/// Back aerial landing Collision callback.
pub fn ftGw_LandingAirB_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Coll(gobj);
        landing_check(gobj, ftGw_MS_LandingAirB);
    }
}

/// Up aerial landing prefunction. Erroneously uses the back aerial's landing lag (as the C).
pub fn ftGw_LandingAirHi_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        landair::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirHi as i32, (*fp).co_attrs.landingairb_lag);
    }
}

/// Up aerial landing Animation callback.
pub fn ftGw_LandingAirHi_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Anim(gobj);
        landing_check(gobj, ftGw_MS_LandingAirHi);
    }
}

/// Up aerial landing Physics callback.
pub fn ftGw_LandingAirHi_Phys(gobj: *mut HSD_GObj) {
    land::ftCo_Landing_Phys(gobj);
}

/// Up aerial landing Collision callback.
pub fn ftGw_LandingAirHi_Coll(gobj: *mut HSD_GObj) {
    unsafe {
        land::ftCo_Landing_Coll(gobj);
        landing_check(gobj, ftGw_MS_LandingAirHi);
    }
}
