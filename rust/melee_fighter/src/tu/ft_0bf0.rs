//! src/melee/ft/ft_0BF0.c: the camera-hit ice KO and Sleep poses (Fox: blaster, Ness: bat,
//! Doc: a pill; no scaling) and their motion test `ftCo_800BF228`. No float math.
//! TARGET_PC: the m-ex onIntroL dispatch after the per-kind work (`Mex_OnIntroLDispatch`).
#![allow(non_snake_case)]

use crate::types::*;

pub type G = *mut HSD_GObj;

/// Callees outside this TU with no port yet: settable, panic while unset.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    macro_rules! ext_fns {
        ($($hook:ident $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported (ft_0BF0 ext)")),
                }
            }
        )*};
    }
    ext_fns! {
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, frame: f32, rate: f32, blend: f32, other: G);
        IT_802AE994 it_802AE994(g: G, part: i32, kind: i32) -> G;
        IT_802AD590 it_802AD590(g: G, part: i32) -> G;
        ITDRMARIOPILL_802C09C4 itDrMarioPill_802C09C4(g: G, pos: &Vec3, color: i32, kind: i32, a: i32, part: i32, facing: f32) -> G;
        FTMR_SPECIALN_VITAMINRANDOM ftMr_SpecialN_VitaminRandom(g: G) -> i32;
        MEX_ON_INTRO_L_DISPATCH Mex_OnIntroLDispatch(kind: i32, g: G, vanilla: *mut core::ffi::c_void);
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

fn enter(g: G, msid: ftCommon_MotionState) {
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
}

fn bone(g: G) -> i32 {
    crate::hooks::ftparts_get_bone_index(fp(g), FtPart_RThumbNb as i32)
}

/// The Fox / Ness cases; true when taken (the C returns from them).
fn fox_or_ness(g: G, kind: i32) -> bool {
    if kind == Ft_Kind_Fox as i32 {
        let ik = unsafe { (*(fp(g).dat_attrs as *const ftFox_DatAttrs)).x20_FOX_BLASTER_GUN_ITKIND };
        let b = bone(g);
        let it = ext::it_802AE994(g, b, ik as i32);
        fp(g).item_gobj = it;
        return true;
    }
    if kind == Ft_Kind_Ness as i32 {
        let b = bone(g);
        let it = ext::it_802AD590(g, b);
        fp(g).item_gobj = it;
        return true;
    }
    false
}

/// `ftCo_800BF034`: DeadUpFallHitCameraIce.
pub fn ftCo_800BF034(g: G) {
    melee_core::ct!("ftCo_800BF034");
    enter(g, ftCo_MS_DeadUpFallHitCameraIce);
    if fox_or_ness(g, crate::ft0892::template_kind(fp(g))) {
        return;
    }
    // m-ex onIntroL (slot 41), at the epilogue (TARGET_PC).
    ext::Mex_OnIntroLDispatch(fp(g).kind as i32, g, core::ptr::null_mut());
}

/// `ftCo_800BF108`: Sleep.
pub fn ftCo_800BF108(g: G) {
    melee_core::ct!("ftCo_800BF108");
    enter(g, ftCo_MS_Sleep);
    let kind = crate::ft0892::template_kind(fp(g));
    if fox_or_ness(g, kind) {
        return;
    }
    if kind == Ft_Kind_DrMario as i32 {
        let color = ext::ftMr_SpecialN_VitaminRandom(g);
        let b = bone(g);
        let f = fp(g);
        let it = ext::itDrMarioPill_802C09C4(g, &f.cur_pos, color, It_Kind_DrMario_Vitamin as i32, 2, b, f.facing_dir);
        fp(g).item_gobj = it;
    }
}

/// `ftCo_800BF228`: is the fighter in one of those two poses.
pub fn ftCo_800BF228(g: G) -> bool {
    melee_core::ct!("ftCo_800BF228");
    if g.is_null() {
        return false;
    }
    let p = unsafe { (*g).user_data as *const Fighter };
    if p.is_null() {
        return false;
    }
    let m = unsafe { (*p).motion_id };
    m == ftCo_MS_DeadUpFallHitCameraIce as i32 || m == ftCo_MS_Sleep as i32
}
