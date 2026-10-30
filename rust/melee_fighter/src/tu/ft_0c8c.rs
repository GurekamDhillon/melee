//! src/melee/ft/ft_0C8C.c: stamina-mode KO (HP reached 0) and the stamina-KO material colour.
//! No float math, no fused sites.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::hooks as h;
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ft_804D6588`: the stamina-KO MObj.
pub static mut ft_804D6588: *mut HSD_MObj = core::ptr::null_mut();

/// Callees outside this TU that have no port yet. Each panics until rewired.
pub mod ext {
    use crate::types::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(unreachable_code, unused_variables)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn HSD_JObjLoadJoint(_a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void = { todo!("TODO(group1 HSD) HSD_JObjLoadJoint") };
        fn HSD_JObjGetDObj(_a0: *mut core::ffi::c_void) -> *mut core::ffi::c_void = { todo!("TODO(group1 HSD) HSD_JObjGetDObj") };
        fn ft_800C80A4(_fp: *mut core::ffi::c_void) = { unsafe { crate::tu::ft_0c88::ext::ft_800C80A4(&mut *(_fp as *mut Fighter)) } };
        fn Player_GetMoreFlagsBit0(_a0: i32) -> i32 = { unsafe { (crate::player::get_more_flags_bit0(_a0 as _)) as _ } };
        fn Player_GetRemainingHPByIndex(_a0: i32, _a1: i32) -> i32 = { unsafe { (crate::player::get_remaining_hp_by_index(_a0 as _, _a1 as _)) as _ } };
        fn ftCommon_8007DB58(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon3::ftcommon_8007db58(_a0 as _); } };
        fn ftCommon_80080460(_a0: *mut core::ffi::c_void) = { unsafe { let _ = crate::ftcommon2::ftcommon_80080460(&mut *(_a0 as *mut Fighter)); } };
        fn ft_PlaySFX(_a0: *mut core::ffi::c_void, _a1: i32, _a2: u8, _a3: u8) = { unsafe { let _ = crate::hooks::play_sfx(&mut *(_a0 as *mut Fighter), _a1 as _, _a2 as _, _a3 as _); } };
        fn Fighter_ResetInputData_80068854(_g: *mut HSD_GObj) = { { unsafe { let _ = crate::tu::fighter::Fighter_ResetInputData_80068854(_g as _); } return; } };
        fn lbBgFlash_80021C48(_a: u32, _b: u32) = { todo!("TODO(group1 lb) lbBgFlash_80021C48") };
        fn gm_80167470(_slot: i32, _sub: i32) = { todo!("TODO(group6 gm) gm_80167470") };
        fn ftCo_8009750C(_g: *mut HSD_GObj) = { { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_8009750C(_g as _); } return; } };
        fn ftCo_CaptureCut_Enter(_g: *mut HSD_GObj) = { { unsafe { let _ = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter(_g as _); } return; } };
        fn ftCo_800DA698(_g: *mut HSD_GObj, _b: bool) = { { unsafe { let _ = crate::tu::ftCo_CatchCut::ftCo_800DA698(_g as _, _b as _); } return; } };
        fn ftCo_80090780(_g: *mut HSD_GObj) = { { unsafe { let _ = crate::tu::ftCo_DamageFall::ftCo_80090780(_g as _); } return; } };
        fn ftCo_DownSpot_Enter(_g: *mut HSD_GObj) = { { unsafe { let _ = crate::tu::ftCo_DownSpot::ftCo_DownSpot_Enter(_g as _); } return; } };
        fn ftMh_MS_343_80151484(_g: *mut HSD_GObj) = { todo!("TODO(group4 ftMh) ftMh_MS_343_80151484") };
        fn ftCh_GrabUnk1_8015ADD0(_g: *mut HSD_GObj) = { todo!("TODO(group4 ftCh) ftCh_GrabUnk1_8015ADD0") };
    }
}

fn inline_ko(g: G) {
    let f = fp(g);
    let sfx = unsafe { (*f.ft_data).x4C_sfx };
    f.set_x2224_b2(1);
    ext::Fighter_ResetInputData_80068854(g);
    h::ftco_800bffd0(fp(g), 0x7A, 0);
    fn_800c8e40(fp(g), sfx);
    fp(g).set_x2225_b6(1);
    ext::lbBgFlash_80021C48(0xE, 0);
    let f = fp(g);
    ext::gm_80167470(f.player_id as i32, f.is_sub_fighter() as i32);
}

/// `ftCo_800C8C84`: start the stamina KO when HP reached 0.
pub fn ftco_800c8c84(g: G) -> bool {
    melee_core::ct!("ftco_800c8c84");
    let f = fp(g);
    if f.x2224_b2() == 0
        && f.x2225_b7() != 0
        && f.dmg.x18F0 == 0
        && ext::Player_GetRemainingHPByIndex((f.player_id as i32) as i32, (f.is_sub_fighter() as i32) as i32) == 0
    {
        fn_800c8e74(g);
        return true;
    }
    false
}

/// `ftCo_800C8D00`.
pub fn ftco_800c8d00(g: G) {
    melee_core::ct!("ftco_800c8d00");
    if fp(g).x2224_b3() == 0 {
        return;
    }
    if fp(g).x2224_b2() == 0 {
        inline_ko(g);
    }
    ext::ftCommon_8007DB58(crate::mtu_ptr::ptr(&(g)));
    ext::ftCo_8009750C(g);
    let victim = fp(g).victim_gobj;
    if !victim.is_null() {
        if fp(g).anon_221b.x221B_b5() != 0 {
            ext::ftCo_CaptureCut_Enter(victim);
        } else {
            ext::ftCo_CaptureCut_Enter(g);
            ext::ftCo_800DA698(victim, false);
        }
        ext::ftCommon_8007DB58(crate::mtu_ptr::ptr(&(victim)));
    }
    if fp(g).ground_or_air == GA_Air {
        ext::ftCo_80090780(g);
    } else {
        ext::ftCo_DownSpot_Enter(g);
    }
    fp(g).set_x2224_b3(0);
}

/// `fn_800C8E40(fp, sfx)`: plays `ft_data->x4C_sfx->xC` (the argument is unused, as in C).
pub fn fn_800c8e40(f: &mut Fighter, _sfx: *mut FtSFX) {
    melee_core::ct!("fn_800c8e40");
    let id = unsafe { (*(*f.ft_data).x4C_sfx).xC };
    ext::ft_PlaySFX(crate::mtu_ptr::ptr(&(f)), (id) as i32, (127) as u8, (64) as u8);
}

/// `fn_800C8E74`.
pub fn fn_800c8e74(g: G) {
    melee_core::ct!("fn_800c8e74");
    let f = fp(g);
    if f.kind == Ft_Kind_MasterH || f.kind == Ft_Kind_CrezyH {
        if f.kind == Ft_Kind_MasterH {
            ext::ftMh_MS_343_80151484(g);
        } else {
            ext::ftCh_GrabUnk1_8015ADD0(g);
        }
        let f = fp(g);
        ext::gm_80167470(f.player_id as i32, f.is_sub_fighter() as i32);
        return;
    }
    f.set_x2224_b3(1);
    if f.dmg.kb_applied != 0.0 {
        inline_ko(g);
    }
}

/// `ftCo_800C8F6C`: cache the KO MObj and set its diffuse colour (PlCo x7D8).
pub fn ftco_800c8f6c() {
    melee_core::ct!("ftco_800c8f6c");
    let dobj = ext::HSD_JObjGetDObj(ext::HSD_JObjLoadJoint(unsafe { crate::tu::fighter::Fighter_804D6504 } as *mut core::ffi::c_void)) as *mut HSD_DObj;
    unsafe {
        let mobj = if !dobj.is_null() { (*dobj).mobj } else { core::ptr::null_mut() };
        ft_804D6588 = mobj;
        let mat = (*mobj).mat;
        if !mat.is_null() {
            (*mat).diffuse = h::common_data().x7D8;
        }
    }
}

/// `ftCo_800C8FC4`.
pub fn ftco_800c8fc4(g: G) {
    melee_core::ct!("ftco_800c8fc4");
    let f = fp(g);
    if (ext::Player_GetMoreFlagsBit0((f.player_id as i32) as i32) as u8) != 0 {
        let cd = h::common_data();
        f.x2034 = cd.x7DC;
        f.x2038 = cd.x7E0;
        f.set_x2227_b3(1);
        ext::ft_800C80A4(crate::mtu_ptr::ptr(&(fp(g))));
        ext::ftCommon_80080460(crate::mtu_ptr::ptr(&(fp(g))));
    }
}

/// `ftCo_800C9034`.
pub fn ftco_800c9034(g: G) {
    melee_core::ct!("ftco_800c9034");
    h::ftco_800d4f24(g, 0);
}
