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
    pub fn Fighter_ResetInputData_80068854(_g: *mut HSD_GObj) { todo!("TODO(group3 fighter) Fighter_ResetInputData_80068854") }
    pub fn lbBgFlash_80021C48(_a: u32, _b: u32) { todo!("TODO(group1 lb) lbBgFlash_80021C48") }
    pub fn gm_80167470(_slot: i32, _sub: i32) { todo!("TODO(group6 gm) gm_80167470") }
    pub fn ftCo_8009750C(_g: *mut HSD_GObj) { todo!("TODO(group3 ftCo) ftCo_8009750C") }
    pub fn ftCo_CaptureCut_Enter(_g: *mut HSD_GObj) { todo!("TODO(group3 ftCo) ftCo_CaptureCut_Enter") }
    pub fn ftCo_800DA698(_g: *mut HSD_GObj, _b: bool) { todo!("TODO(group3 ftCo) ftCo_800DA698") }
    pub fn ftCo_80090780(_g: *mut HSD_GObj) { todo!("TODO(group3 ftCo) ftCo_80090780") }
    pub fn ftCo_DownSpot_Enter(_g: *mut HSD_GObj) { todo!("TODO(group3 ftCo) ftCo_DownSpot_Enter") }
    pub fn ftMh_MS_343_80151484(_g: *mut HSD_GObj) { todo!("TODO(group4 ftMh) ftMh_MS_343_80151484") }
    pub fn ftCh_GrabUnk1_8015ADD0(_g: *mut HSD_GObj) { todo!("TODO(group4 ftCh) ftCh_GrabUnk1_8015ADD0") }
    /// `HSD_JObjGetDObj(HSD_JObjLoadJoint(Fighter_804D6504))`.
    pub fn HSD_JObjGetDObj_LoadJoint_Fighter_804D6504() -> *mut HSD_DObj {
        todo!("TODO(group1 HSD) HSD_JObjLoadJoint(Fighter_804D6504) / HSD_JObjGetDObj")
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
    let f = fp(g);
    if f.x2224_b2() == 0
        && f.x2225_b7() != 0
        && f.dmg.x18F0 == 0
        && crate::player::get_remaining_hp_by_index(f.player_id as i32, f.is_sub_fighter() as i32) == 0
    {
        fn_800c8e74(g);
        return true;
    }
    false
}

/// `ftCo_800C8D00`.
pub fn ftco_800c8d00(g: G) {
    if fp(g).x2224_b3() == 0 {
        return;
    }
    if fp(g).x2224_b2() == 0 {
        inline_ko(g);
    }
    crate::ftcommon3::ftcommon_8007db58(g);
    ext::ftCo_8009750C(g);
    let victim = fp(g).victim_gobj;
    if !victim.is_null() {
        if fp(g).anon_221b.x221B_b5() != 0 {
            ext::ftCo_CaptureCut_Enter(victim);
        } else {
            ext::ftCo_CaptureCut_Enter(g);
            ext::ftCo_800DA698(victim, false);
        }
        crate::ftcommon3::ftcommon_8007db58(victim);
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
    let id = unsafe { (*(*f.ft_data).x4C_sfx).xC };
    h::play_sfx(f, id, 127, 64);
}

/// `fn_800C8E74`.
pub fn fn_800c8e74(g: G) {
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
    let dobj = ext::HSD_JObjGetDObj_LoadJoint_Fighter_804D6504();
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
    let f = fp(g);
    if crate::player::get_more_flags_bit0(f.player_id as i32) != 0 {
        let cd = h::common_data();
        f.x2034 = cd.x7DC;
        f.x2038 = cd.x7E0;
        f.set_x2227_b3(1);
        crate::tu::ft_0c88::ext::ft_800C80A4(fp(g));
        crate::ftcommon2::ftcommon_80080460(fp(g));
    }
}

/// `ftCo_800C9034`.
pub fn ftco_800c9034(g: G) {
    h::ftco_800d4f24(g, 0);
}
