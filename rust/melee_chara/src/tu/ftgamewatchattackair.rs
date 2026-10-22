//! src/melee/ft/kinds/ftGameWatch/ftgamewatchattackair.c: Mr. Game & Watch's aerials (parachute,
//! turtle and sparky items) and their landing lag. No float arithmetic (0 fused sites).
//! Smoke-tested by tests/tu_ftgamewatchattackair.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 2/5/12): re-exported under this TU.
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
    hook!(LB_8000B1CC, lb_8000B1CC, (joint: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3));
    // Parachute (itgamewatchparachute.c)
    hook!(IT_802C6C38, it_802C6C38, (owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing_dir: f32) -> *mut HSD_GObj);
    hook!(IT_802C6E50, it_802C6E50, (item: *mut HSD_GObj));
    hook!(IT_802C6D6C, it_802C6D6C, (item: *mut HSD_GObj));
    hook!(IT_802C6DB8, it_802C6DB8, (item: *mut HSD_GObj));
    hook!(IT_802C6DD8, it_802C6DD8, (item: *mut HSD_GObj));
    // Turtle (itgamewatchturtle.c)
    hook!(IT_802C6F40, it_802C6F40, (owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing_dir: f32) -> *mut HSD_GObj);
    hook!(IT_802C7158, it_802C7158, (item: *mut HSD_GObj));
    hook!(IT_802C7074, it_802C7074, (item: *mut HSD_GObj));
    hook!(IT_802C70C0, it_802C70C0, (item: *mut HSD_GObj));
    hook!(IT_802C70E0, it_802C70E0, (item: *mut HSD_GObj));
    // Sparky (itgamewatchbreath.c)
    hook!(IT_802C720C, it_802C720C, (owner: *mut HSD_GObj, pos: *const Vec3, part: i32, facing_dir: f32) -> *mut HSD_GObj);
    hook!(IT_802C7424, it_802C7424, (item: *mut HSD_GObj));
    hook!(IT_802C7340, it_802C7340, (item: *mut HSD_GObj));
    hook!(IT_802C738C, it_802C738C, (item: *mut HSD_GObj));
    hook!(IT_802C73AC, it_802C73AC, (item: *mut HSD_GObj));
    // ftCommon
    hook_or!(FTCO_ATTACKAIR_GETMSIDFROMCSTICK, ftCo_AttackAir_GetMsidFromCStick, (fp: *mut Fighter) -> i32,
        |fp: *mut Fighter| unsafe { melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_GetMsidFromCStick(&*fp) });
    hook_or!(FTCO_ATTACKAIR_ENTERFROMMSID, ftCo_AttackAir_EnterFromMsid, (gobj: *mut HSD_GObj, msid: i32),
        melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_EnterFromMsid);
    hook_or!(FTCO_ATTACKAIR_ANIM, ftCo_AttackAir_Anim, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAir_Anim);
    hook_or!(FTCO_ATTACKAIRN_IASA, ftCo_AttackAirN_IASA, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAirN_IASA);
    hook_or!(FTCO_ATTACKAIRB_IASA, ftCo_AttackAirB_IASA, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAirB_IASA);
    hook_or!(FTCO_ATTACKAIRHI_IASA, ftCo_AttackAirHi_IASA, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_AttackAir::ftCo_AttackAirHi_IASA);
    hook_or!(FTCO_LANDING_ENTER_BASIC, ftCo_Landing_Enter_Basic, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Landing::ftCo_Landing_Enter_Basic);
    hook_or!(FTCO_LANDINGAIR_ENTERWITHMSIDLAG, ftCo_LandingAir_EnterWithMsidLag, (gobj: *mut HSD_GObj, msid: i32, lag: f32),
        melee_fighter::tu::ftCo_LandingAir::ftCo_LandingAir_EnterWithMsidLag);
    hook_or!(FTCO_LANDING_ANIM, ftCo_Landing_Anim, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Landing::ftCo_Landing_Anim);
    hook_or!(FTCO_LANDING_PHYS, ftCo_Landing_Phys, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Landing::ftCo_Landing_Phys);
    hook_or!(FTCO_LANDING_COLL, ftCo_Landing_Coll, (gobj: *mut HSD_GObj), melee_fighter::tu::ftCo_Landing::ftCo_Landing_Coll);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// Fighter callback entries.
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemParachuteSetup(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemParachuteSetup(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemParachuteOnLand(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemParachuteOnLand(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemTurtleSetup(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemTurtleSetup(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemTurtleOnLand(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemTurtleOnLand(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemSparkySetup(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemSparkySetup(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ItemSparkyOnLand(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ItemSparkyOnLand(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_EnterItemHitlag(g: *mut HSD_GObj) {
    ftGw_AttackAirN_EnterItemHitlag(g);
}
pub unsafe extern "C" fn cb_ftGw_AttackAirN_ExitItemHitlag(g: *mut HSD_GObj) {
    ftGw_AttackAirN_ExitItemHitlag(g);
}
pub unsafe extern "C" fn cb_ftGw_LandingAirN_Enter(g: *mut HSD_GObj) {
    ftGw_LandingAirN_Enter(g);
}
pub unsafe extern "C" fn cb_ftGw_LandingAirB_Enter(g: *mut HSD_GObj) {
    ftGw_LandingAirB_Enter(g);
}
pub unsafe extern "C" fn cb_ftGw_LandingAirHi_Enter(g: *mut HSD_GObj) {
    ftGw_LandingAirHi_Enter(g);
}

/// The damage / hitlag callbacks shared by every live aerial item.
unsafe fn set_item_callbacks(fp: *mut Fighter) {
    (*fp).death2_cb = Some(crate::tu::ftgamewatch::cb_ftGw_Init_OnDamage);
    (*fp).take_dmg_cb = Some(crate::tu::ftgamewatch::cb_ftGw_Init_OnDamage);
    (*fp).pre_hitlag_cb = Some(cb_ftGw_AttackAirN_EnterItemHitlag);
    (*fp).post_hitlag_cb = Some(cb_ftGw_AttackAirN_ExitItemHitlag);
}

// ---- parachute ----

/// Create Parachute item.
pub fn ftGw_AttackAirN_ItemParachuteSetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);

        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ftGw_AttackAirN_ItemParachuteOnLand(gobj);
        } else {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_TransN as usize)).joint, core::ptr::null(), &mut sp10);
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
            if (*fp).motion_id as i32 == ftGw_MS_LandingAirN as i32 {
                ext::it_802C6E50((*fp).u.gw.x2258_parachuteGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Parachute flag update on land, clear pointers.
pub fn ftGw_AttackAirN_ItemParachuteSetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x2258_parachuteGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove Parachute item.
pub fn ftGw_AttackAirN_ItemParachuteRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2258_parachuteGObj.is_null() {
            ext::it_802C6D6C((*fp).u.gw.x2258_parachuteGObj);
            ftGw_AttackAirN_ItemParachuteSetFlag(gobj);
        }
    }
}

/// True when G&W is NOT in neutral air / its landing.
pub fn ftGw_AttackAirN_ItemCheckParachuteRemove(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let msid = (*fpp(gobj)).motion_id as i32;
        !(msid >= ftGw_MS_AttackAirN as i32 && msid <= ftGw_MS_LandingAirN as i32)
    }
}

// ---- turtle ----

/// Create Turtle item.
pub fn ftGw_AttackAirN_ItemTurtleSetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);

        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ftGw_AttackAirN_ItemTurtleOnLand(gobj);
        } else {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LShoulderN as usize)).joint, core::ptr::null(), &mut sp10);
            (*fp).u.gw.x225C_turtleGObj = ext::it_802C6F40(gobj, &sp10, FtPart_LShoulderN as i32, (*fp).facing_dir);
            if !(*fp).u.gw.x225C_turtleGObj.is_null() {
                set_item_callbacks(fp);
            }
        }
        (*fp).accessory4_cb = None;
    }
}

/// Turtle accessory callback: landing lag.
pub fn ftGw_AttackAirN_ItemTurtleOnLand(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ftGw_AttackAirN_ExitItemHitlag(gobj);
            // Deliberate msid mismatch in the C (LandingAirN, not LandingAirB).
            if (*fp).motion_id as i32 == ftGw_MS_LandingAirN as i32 {
                ext::it_802C7158((*fp).u.gw.x225C_turtleGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Turtle flag update on land, clear pointers.
pub fn ftGw_AttackAirN_ItemTurtleSetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x225C_turtleGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove Turtle item.
pub fn ftGw_AttackAirN_ItemTurtleRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x225C_turtleGObj.is_null() {
            ext::it_802C7074((*fp).u.gw.x225C_turtleGObj);
            ftGw_AttackAirN_ItemTurtleSetFlag(gobj);
        }
    }
}

/// True when G&W is NOT in back air / its landing.
pub fn ftGw_AttackAirN_ItemCheckTurtleRemove(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let msid = (*fpp(gobj)).motion_id as i32;
        !(msid >= ftGw_MS_AttackAirB as i32 && msid <= ftGw_MS_LandingAirB as i32)
    }
}

// ---- sparky ----

/// Create Sparky item.
pub fn ftGw_AttackAirN_ItemSparkySetup(gobj: *mut HSD_GObj) {
    unsafe {
        let mut sp10 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let fp = fpp(gobj);

        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ftGw_AttackAirN_ItemSparkyOnLand(gobj);
        } else {
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_LHandN as usize)).joint, core::ptr::null(), &mut sp10);
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
            if (*fp).motion_id as i32 == ftGw_MS_LandingAirN as i32 {
                ext::it_802C7424((*fp).u.gw.x2260_sparkyGObj);
            }
            set_item_callbacks(fp);
            (*fp).accessory4_cb = None;
        }
    }
}

/// Sparky flag update on land, clear pointers.
pub fn ftGw_AttackAirN_ItemSparkySetFlag(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftGw_AttackAirN_ExitItemHitlag(gobj);
        (*fp).u.gw.x2260_sparkyGObj = core::ptr::null_mut();
        (*fp).death2_cb = None;
        (*fp).take_dmg_cb = None;
    }
}

/// Remove Sparky item.
pub fn ftGw_AttackAirN_ItemSparkyRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.gw.x2260_sparkyGObj.is_null() {
            ext::it_802C7340((*fp).u.gw.x2260_sparkyGObj);
            ftGw_AttackAirN_ItemSparkySetFlag(gobj);
        }
    }
}

/// Apply hitlag to whichever aerial items are out.
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

/// Remove hitlag from whichever aerial items are out.
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

/// True when G&W is NOT in up air / its landing.
pub fn ftGw_AttackAirN_ItemCheckSparkyRemove(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let msid = (*fpp(gobj)).motion_id as i32;
        !(msid >= ftGw_MS_AttackAirHi as i32 && msid <= ftGw_MS_LandingAirHi as i32)
    }
}

// ---- aerial attacks ----

/// Pick the aerial from the C-stick.
pub fn ftGw_AttackAirN_DecideAction(gobj: *mut HSD_GObj) {
    unsafe {
        let msid = ext::ftCo_AttackAir_GetMsidFromCStick(fpp(gobj));
        match msid {
            x if x == ftCo_MS_AttackAirN as i32 => ftGw_AttackAirN_Enter(gobj),
            x if x == ftCo_MS_AttackAirF as i32 => ext::ftCo_AttackAir_EnterFromMsid(gobj, msid),
            x if x == ftCo_MS_AttackAirB as i32 => ftGw_AttackAirB_Enter(gobj),
            x if x == ftCo_MS_AttackAirHi as i32 => ftGw_AttackAirHi_Enter(gobj),
            x if x == ftCo_MS_AttackAirLw as i32 => ext::ftCo_AttackAir_EnterFromMsid(gobj, msid),
            _ => {}
        }
    }
}

pub fn ftGw_AttackAirN_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirN as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemParachuteSetup);
    }
}

pub fn ftGw_AttackAirN_Anim(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAir_Anim(gobj);
}

pub fn ftGw_AttackAirN_IASA(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAirN_IASA(gobj);
}

/// Shared by the three landing-enter functions: release the item hitlag.
unsafe fn exit_all_item_hitlag(fp: *mut Fighter) {
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

/// Neutral air landing (the `ft_80082C74` landing callback).
pub fn ftGw_LandingAirN_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        exit_all_item_hitlag(fp);
        if (*fp).cmd_vars[0] != 0 {
            ftGw_LandingAirN_Init(gobj);
            (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemParachuteOnLand);
            return;
        }
        ext::ftCo_Landing_Enter_Basic(gobj);
        crate::tu::ftgamewatch::ftGw_Init_OnDamage(gobj);
    }
}

pub fn ftGw_AttackAirB_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirB as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemTurtleSetup);
    }
}

pub fn ftGw_AttackAirB_Anim(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAir_Anim(gobj);
}

pub fn ftGw_AttackAirB_IASA(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAirB_IASA(gobj);
}

/// Back air landing.
pub fn ftGw_LandingAirB_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        exit_all_item_hitlag(fp);
        if (*fp).cmd_vars[0] != 0 {
            ftGw_LandingAirB_Init(gobj);
            (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemTurtleOnLand);
            return;
        }
        ext::ftCo_Landing_Enter_Basic(gobj);
        crate::tu::ftgamewatch::ftGw_Init_OnDamage(gobj);
    }
}

pub fn ftGw_AttackAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_AttackAir_EnterFromMsid(gobj, ftGw_MS_AttackAirHi as i32);
        (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemSparkySetup);
    }
}

pub fn ftGw_AttackAirHi_Anim(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAir_Anim(gobj);
}

pub fn ftGw_AttackAirHi_IASA(gobj: *mut HSD_GObj) {
    ext::ftCo_AttackAirHi_IASA(gobj);
}

/// Up air landing.
pub fn ftGw_LandingAirHi_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        exit_all_item_hitlag(fp);
        if (*fp).cmd_vars[0] != 0 {
            ftGw_LandingAirHi_Init(gobj);
            (*fp).accessory4_cb = Some(cb_ftGw_AttackAirN_ItemSparkyOnLand);
            return;
        }
        ext::ftCo_Landing_Enter_Basic(gobj);
        crate::tu::ftgamewatch::ftGw_Init_OnDamage(gobj);
    }
}

// ---- landings ----

/// `ftGw_LandingAirN_Init` (static): neutral air landing prefunction.
pub fn ftGw_LandingAirN_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirN as i32, (*fp).co_attrs.landingairn_lag);
    }
}

/// Runs `ftCo_Landing_Anim`/`Coll`, then drops the items if the landing state ended.
unsafe fn landing_then_drop(gobj: *mut HSD_GObj, msid: u32, f: fn(*mut HSD_GObj)) {
    let fp = fpp(gobj);
    f(gobj);
    if (*fp).motion_id as i32 != msid as i32 {
        crate::tu::ftgamewatch::ftGw_Init_OnDamage(gobj);
    }
}

pub fn ftGw_LandingAirN_Anim(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirN as u32, ext::ftCo_Landing_Anim) }
}

pub fn ftGw_LandingAirN_Phys(gobj: *mut HSD_GObj) {
    ext::ftCo_Landing_Phys(gobj);
}

pub fn ftGw_LandingAirN_Coll(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirN as u32, ext::ftCo_Landing_Coll) }
}

/// Back air landing prefunction.
pub fn ftGw_LandingAirB_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirB as i32, (*fp).co_attrs.landingairb_lag);
    }
}

pub fn ftGw_LandingAirB_Anim(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirB as u32, ext::ftCo_Landing_Anim) }
}

pub fn ftGw_LandingAirB_Phys(gobj: *mut HSD_GObj) {
    ext::ftCo_Landing_Phys(gobj);
}

pub fn ftGw_LandingAirB_Coll(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirB as u32, ext::ftCo_Landing_Coll) }
}

/// Up air landing prefunction (erroneously uses back air's landing lag, as the C does).
pub fn ftGw_LandingAirHi_Init(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftCo_LandingAir_EnterWithMsidLag(gobj, ftGw_MS_LandingAirHi as i32, (*fp).co_attrs.landingairb_lag);
    }
}

pub fn ftGw_LandingAirHi_Anim(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirHi as u32, ext::ftCo_Landing_Anim) }
}

pub fn ftGw_LandingAirHi_Phys(gobj: *mut HSD_GObj) {
    ext::ftCo_Landing_Phys(gobj);
}

pub fn ftGw_LandingAirHi_Coll(gobj: *mut HSD_GObj) {
    unsafe { landing_then_drop(gobj, ftGw_MS_LandingAirHi as u32, ext::ftCo_Landing_Coll) }
}
