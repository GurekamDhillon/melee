//! src/melee/ft/kinds/ftKirby/ftkirbyspecialgamewatch.c: Kirby with Mr. Game & Watch's hat, neutral B
//! (Chef). No fused sites (tools/rust-port/fma_sites.py). Stateful only: smoke-tested through
//! recording hooks by tests/tu_ftkirbyspecialgamewatch.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2, 3, 4): re-exported under this TU.
pub use crate::chars::ftKirby::ftkirbyspecialgamewatch::ftKb_SpecialNGw_8010D160;
pub use crate::chars::r2::ftKirby::ftkirbyspecialgamewatch::{ftKb_GwSpecialAirN_Phys, ftKb_GwSpecialN_Phys};
pub use crate::chars::r3::ftKirby::ftkirbyspecialgamewatch::{
    ftKb_GwSpecialAirN_Coll, ftKb_GwSpecialN_Coll, ftKb_SpecialNGw_8010D580, ftKb_SpecialNGw_8010D5F0,
};
pub use crate::chars::r4::ftKirby::ftkirbyspecialgamewatch::inline1;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::{HSD_GObj, HSD_JObj, Item_GObj, Vec3};
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000B1CC(jobj: *mut HSD_JObj, offset: *const Vec3, out: *mut Vec3)  {
                match unsafe { LB_8000B1CC } {
                    Some(f) => f(jobj, offset, out),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, offset as _, out as _); } return; },
                }
            }
    hook!(HSD_RANDI, HSD_Randi, (max: i32) -> i32);
    pub static mut IT_802C837C: Option<fn(*mut HSD_GObj, *mut Vec3, i32, i32, f32) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C837C(gobj: *mut HSD_GObj, pos: *mut Vec3, kind: i32, sausage: i32, dir: f32)  {
                match unsafe { IT_802C837C } {
                    Some(f) => f(gobj, pos, kind, sausage, dir),
                    None => { unsafe { let _ = melee_items::tu::itgamewatchchef::it_802C837C(gobj as _, pos as _, kind as _, sausage as _, dir as _); } return; },
                }
            }
    pub static mut IT_802C74D8: Option<fn(*mut HSD_GObj, *mut Vec3, i32, f32) -> *mut Item_GObj> = None;
            #[allow(non_snake_case)]
            pub fn it_802C74D8(gobj: *mut HSD_GObj, pos: *mut Vec3, bone: i32, dir: f32) -> *mut Item_GObj {
                match unsafe { IT_802C74D8 } {
                    Some(f) => f(gobj, pos, bone, dir),
                    None => { return unsafe { melee_items::tu::itkirbygamewatchchefpan::it_802C74D8(gobj as _, pos as _, bone as _, dir as _) as _ }; },
                }
            }
    pub static mut IT_802C760C: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C760C(item: *mut Item_GObj)  {
                match unsafe { IT_802C760C } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itkirbygamewatchchefpan::it_802C760C(item as _); } return; },
                }
            }
    pub static mut IT_802C7658: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7658(item: *mut Item_GObj)  {
                match unsafe { IT_802C7658 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itkirbygamewatchchefpan::it_802C7658(item as _); } return; },
                }
            }
    pub static mut IT_802C7678: Option<fn(*mut Item_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn it_802C7678(item: *mut Item_GObj)  {
                match unsafe { IT_802C7678 } {
                    Some(f) => f(item),
                    None => { unsafe { let _ = melee_items::tu::itkirbygamewatchchefpan::it_802C7678(item as _); } return; },
                }
            }
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `ftParts_GetBoneIndex(fp, FtPart_LThumbNb)`.
#[inline(always)]
unsafe fn thumb_bone(fp: *mut Fighter) -> i32 {
    melee_fighter::ftdmgfall::ftparts_get_bone_index(&*fp, FtPart_LThumbNb as i32)
}

pub unsafe extern "C" fn cb_fn_8010CE5C(g: *mut HSD_GObj) {
    fn_8010CE5C(g);
}
pub unsafe extern "C" fn cb_fn_8010CFB0(g: *mut HSD_GObj) {
    fn_8010CFB0(g);
}
pub unsafe extern "C" fn cb_fn_8010D100(g: *mut HSD_GObj) {
    fn_8010D100(g);
}
pub unsafe extern "C" fn cb_ftKb_SpecialNGw_8010D130(g: *mut HSD_GObj) {
    ftKb_SpecialNGw_8010D130(g);
}

/// `fn_8010CE5C` (static, accessory4_cb): on the script's cmd_vars[0] pulse throw one sausage,
/// picking its slot among the five not used by the last two throws.
pub fn fn_8010CE5C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp2 = fpp(gobj);
        let da = (*fp2).dat_attrs as *const ftKb_DatAttrs;
        let mut sausage_count = [0i32; 6];
        if (*fp2).cmd_vars[0] != 0 {
            let mut var_r28: i32 = 0;
            (*fp2).cmd_vars[0] = 0;
            if ((*fp2).mv.gw.SpecialN.maxSausage as f32) < (*da).specialn_gw_max_sausages_per_use {
                (*fp2).mv.gw.SpecialN.maxSausage += 1;
                let vec1 = Vec3 { x: 2.5, y: 6.5, z: 0.0 };
                let mut vec0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::lb_8000B1CC((*(*fp2).parts.add(thumb_bone(fp2) as usize)).joint, &vec1, &mut vec0);
                let fp = fpp(gobj);
                let mut p_sausage = 0usize;
                for i in 0..5 {
                    if i != (*fp).u.kb.xD4 && i != (*fp).u.kb.xD8 {
                        sausage_count[p_sausage] = i;
                        p_sausage += 1;
                        var_r28 += 1;
                    }
                }
                let temp_r5 = sausage_count[ext::HSD_Randi(var_r28) as usize];
                (*fp).u.kb.xD8 = (*fp).u.kb.xD4;
                (*fp).u.kb.xD4 = temp_r5;
                ext::it_802C837C(gobj, &mut vec0, It_Kind_Kirby_GameWatchChef as i32, temp_r5, (*fp2).facing_dir);
            }
            (*fp2).accessory4_cb = None;
        }
    }
}

/// `fn_8010CFB0` (static, accessory4_cb): spawn the chef item at the thumb and arm its callbacks.
pub fn fn_8010CFB0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC((*(*fp).parts.add(thumb_bone(fp) as usize)).joint, core::ptr::null(), &mut pos);
        (*fp).u.kb.xDC = ext::it_802C74D8(gobj, &mut pos, thumb_bone(fp), (*fp).facing_dir);
        if !(*fp).u.kb.xDC.is_null() {
            (*fp).death2_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE74C);
            (*fp).take_dmg_cb = Some(crate::chars::cb::ftkirby__ftKb_Init_800EE7B8);
        }
        (*fp).pre_hitlag_cb = Some(cb_fn_8010D100);
        (*fp).post_hitlag_cb = Some(cb_ftKb_SpecialNGw_8010D130);
        (*fp).accessory4_cb = Some(cb_fn_8010CE5C);
    }
}

pub fn ftKb_SpecialNGw_8010D074(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ftKb_SpecialNGw_8010D130(gobj);
        (*fp).u.kb.xDC = core::ptr::null_mut();
    }
}

pub fn ftKb_SpecialNGw_8010D0A8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xDC.is_null() {
            ext::it_802C760C((*fp).u.kb.xDC);
            ftKb_SpecialNGw_8010D074(gobj);
        }
    }
}

pub fn fn_8010D100(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xDC.is_null() {
            ext::it_802C7658((*fp).u.kb.xDC);
        }
    }
}

pub fn ftKb_SpecialNGw_8010D130(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if !(*fp).u.kb.xDC.is_null() {
            ext::it_802C7678((*fp).u.kb.xDC);
        }
    }
}

/// `setGwVars` (static inline): `ftGameWatch_SpecialN_SetVars` with the Chef callback.
pub fn setGwVars(fighter_gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(fighter_gobj);
        (*fp).cmd_vars[0] = 0;
        (*fp).cmd_vars[1] = 0;
        (*fp).cmd_vars[2] = 0;
        (*fp).mv.gw.SpecialN.isChefLoopDisable = 0;
        (*fp).mv.gw.SpecialN.maxSausage = 0;
        (*fp).accessory4_cb = Some(cb_fn_8010CFB0);
    }
}

pub fn ftKb_SpecialNGw_8010D188(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_GwSpecialN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        setGwVars(gobj);
    }
}

pub fn ftKb_SpecialNGw_8010D204(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).self_vel.y = 0.0;
        st::Fighter_ChangeMotionState(gobj, ftKb_MS_GwSpecialAirN as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        setGwVars(gobj);
    }
}

pub fn ftKb_GwSpecialN_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*da).specialn_gw_max_sausages_per_use
                && (*fp).mv.gw.SpecialN.isChefLoopDisable == 0
            {
                ftKb_SpecialNGw_8010D660(gobj, (*da).specialn_gw_frame_on_repeat);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftKb_GwSpecialAirN_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).cmd_vars[2] != 0 {
            (*fp).cmd_vars[2] = 0;
            if (*fp).mv.gw.SpecialN.isChefLoopDisable == 0
                && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*da).specialn_gw_max_sausages_per_use
            {
                ftKb_SpecialNGw_8010D6D0(gobj, (*da).specialn_gw_frame_on_repeat);
            }
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ftCo_Fall_Enter(gobj);
        }
    }
}

pub fn ftKb_GwSpecialN_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).mv.gw.SpecialN.isChefLoopDisable = 1;
        }
        if (*fp).cmd_vars[1] != 0
            && ((*fp).input.pressed_buttons & (HSD_PAD_B as u32)) != 0
            && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*da).specialn_gw_max_sausages_per_use
        {
            ftKb_SpecialNGw_8010D660(gobj, (*da).specialn_gw_frame_on_repeat);
        }
    }
}

pub fn ftKb_GwSpecialAirN_IASA(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftKb_DatAttrs;
        if (*fp).input.held_buttons[0] & (HSD_PAD_B as u32) == 0 {
            (*fp).mv.gw.SpecialN.isChefLoopDisable = 1;
        }
        if (*fp).cmd_vars[1] != 0
            && ((*fp).input.pressed_buttons & (HSD_PAD_B as u32)) != 0
            && ((*fp).mv.gw.SpecialN.maxSausage as f32) < (*da).specialn_gw_max_sausages_per_use
        {
            ftKb_SpecialNGw_8010D6D0(gobj, (*da).specialn_gw_frame_on_repeat);
        }
    }
}

/// `ftKb_SpecialNGw_8010D660`: restart the ground Chef swing at `arg1 - 1`.
pub fn ftKb_SpecialNGw_8010D660(gobj: *mut HSD_GObj, arg1: f32) {
    st::Fighter_ChangeMotionState(gobj, ftKb_MS_GwSpecialN as i32, 0x0C4C508C, arg1 - 1.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    inline1(gobj);
}

/// `ftKb_SpecialNGw_8010D6D0`: the aerial restart.
pub fn ftKb_SpecialNGw_8010D6D0(gobj: *mut HSD_GObj, arg1: f32) {
    st::Fighter_ChangeMotionState(gobj, ftKb_MS_GwSpecialAirN as i32, 0x0C4C508C, arg1 - 1.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    inline1(gobj);
}
