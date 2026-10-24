//! src/melee/ft/kinds/ftNess/ftnessattacks4.c: Ness forward smash (the baseball bat).
//! No fused sites (tools/rust-port/fma_sites.py), no float math. Smoke-tested through recording
//! hooks by tests/tu_ftnessattacks4.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

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
    crate::port_hook!(FT_PLAYSFX, ft_PlaySFX, crate::adapt::ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: u8, pan: u8));
    pub static mut IT_802AD478: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) -> *mut HSD_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802AD478(gobj: *mut HSD_GObj, pos: *mut Vec3, bone: i32, facing: f32) -> *mut HSD_GObj {
                match unsafe { IT_802AD478 } {
                    Some(f) => f(gobj, pos, bone, facing),
                    None => { return unsafe { melee_items::tu::itnessbat::it_802AD478(gobj as _, pos as _, bone as _, facing as _) as _ }; },
                }
            }
    pub static mut IT_802AD6B8: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802AD6B8(bat: *mut HSD_GObj)  {
                match unsafe { IT_802AD6B8 } {
                    Some(f) => f(bat),
                    None => { unsafe { let _ = melee_items::tu::itnessbat::it_802AD6B8(bat as _); } return; },
                }
            }
    crate::port_hook!(FTCOLL_CREATEREFLECTHIT, ftColl_CreateReflectHit, crate::adapt::ftColl_CreateReflectHit, (gobj: *mut HSD_GObj, desc: *mut ReflectDesc, cb: HSD_GObjEvent));
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// C-ABI entry for the reflect callback stored by `ftColl_CreateReflectHit`.
pub unsafe extern "C" fn cb_ftNs_AttackS4_OnReflect(g: *mut HSD_GObj) {
    ftNs_AttackS4_OnReflect(g)
}

pub fn ftNs_AttackS4_OnReflect(gobj: *mut HSD_GObj) {
    unsafe { ext::ft_PlaySFX(fpp(gobj), 0xE0, 0x7F, 0x40) }
}

pub fn ftNs_AttackS4_Enter(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).set_allow_interrupt(0);
        (*fp).cmd_vars[0] = 0;
        st::Fighter_ChangeMotionState(gobj, ftNs_MS_AttackS4 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        // 0x2A: Item Hold Bone.
        let bat = ext::it_802AD478(gobj, &mut (*fp).cur_pos, 0x2A, (*fp).facing_dir);
        (*fp).u.ns.bat_gobj = bat;
        if !bat.is_null() {
            (*fp).death2_cb = Some(crate::tu::ftnessattackhi4::cb_ftNs_Init_OnDamage);
            (*fp).take_dmg_cb = Some(crate::tu::ftnessattackhi4::cb_ftNs_Init_OnDamage);
        }
    }
}

/// C `bool`: true when Ness is not in the bat swing or has no bat.
pub fn ftNs_AttackS4_CheckNessBatRemove(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).motion_id as i32 != ftNs_MS_AttackS4 as i32 {
            return true;
        }
        (*fp).u.ns.bat_gobj.is_null()
    }
}

pub fn ftNs_AttackS4_ItemNessBatRemove(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ns.bat_gobj.is_null() {
            ext::it_802AD6B8((*fp).u.ns.bat_gobj);
            (*fp).u.ns.bat_gobj = core::ptr::null_mut();
        }
    }
}

pub fn ftNs_AttackS4_ItemNessBatSetNULL(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.ns.bat_gobj.is_null() {
            (*fp).u.ns.bat_gobj = core::ptr::null_mut();
        }
    }
}

pub fn ftNs_AttackS4_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).reflecting() == 0 {
            // The script raises cmd_vars[0] while the reflect bubble is out.
            if (*fp).cmd_vars[0] != 0 {
                let attrs = (*fp).dat_attrs as *mut ftNessAttributes;
                ext::ftColl_CreateReflectHit(gobj, &mut (*attrs).xB8_BASEBALL_BAT, Some(cb_ftNs_AttackS4_OnReflect));
            }
        } else if (*fp).cmd_vars[0] == 0 {
            (*fp).set_reflecting(0);
        }

        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp2 = fpp(gobj);
            if !(*fp2).u.ns.bat_gobj.is_null() {
                ext::it_802AD6B8((*fp2).u.ns.bat_gobj);
                (*fp2).u.ns.bat_gobj = core::ptr::null_mut();
            }
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftNs_AttackS4_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).allow_interrupt() != 0 {
            if !(*fp).u.ns.bat_gobj.is_null() {
                ext::it_802AD6B8((*fp).u.ns.bat_gobj);
                (*fp).u.ns.bat_gobj = core::ptr::null_mut();
            }
            st::ftCo_Wait_IASA(gobj);
        }
    }
}

pub fn ftNs_AttackS4_Phys(gobj: *mut HSD_GObj) {
    st::ft_80084FA8(gobj);
    st::ftColl_8007AEF8(gobj);
}

pub fn ftNs_AttackS4_Coll(gobj: *mut HSD_GObj) {
    st::ft_80084104(gobj);
}
