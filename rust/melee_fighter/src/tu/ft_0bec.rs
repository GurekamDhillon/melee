//! src/melee/ft/ft_0BEC.c: the "dead" poses DeadDown, DeadRight and DeadUpStarIce, which hand
//! Fox his blaster, Ness his bat and Doc two pills. No fused sites
//! (`0.71428f * (scale_y * model_scaling)` is two plain f32 products).
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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported (ft_0BEC ext)")),
                }
            }
        )*};
    }
    ext_fns! {
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, frame: f32, rate: f32, blend: f32, other: G);
        IT_802AE994 it_802AE994(g: G, part: i32, kind: i32) -> G;
        IT_802AD590 it_802AD590(g: G, part: i32) -> G;
        IT_8026BAE8 it_8026BAE8(i: G, scale: f32);
        ITDRMARIOPILL_802C09C4 itDrMarioPill_802C09C4(g: G, pos: &Vec3, color: i32, kind: i32, a: i32, part: i32, facing: f32) -> G;
        FTMR_SPECIALN_VITAMINRANDOM ftMr_SpecialN_VitaminRandom(g: G) -> i32;
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `setupInitialState` (static inline).
fn setup_initial_state(f: &mut Fighter) {
    f.set_x2219_b2(1);
    f.set_x2219_b1(1);
    f.set_x2228_b1(1);
    f.item_gobj = core::ptr::null_mut();
    f.x1984_heldItemSpec = core::ptr::null_mut();
}

/// `FoxHelper` (static inline): the blaster in the right thumb, scaled to the model.
fn fox_helper(g: G) {
    let kind = unsafe { (*(fp(g).dat_attrs as *const ftFox_DatAttrs)).x20_FOX_BLASTER_GUN_ITKIND };
    let bone = crate::hooks::ftparts_get_bone_index(fp(g), FtPart_RThumbNb as i32);
    let it = ext::it_802AE994(g, bone, kind as i32);
    let f = fp(g);
    f.item_gobj = it;
    ext::it_8026BAE8(f.item_gobj, f.x34_scale.y * f.co_attrs.model_scaling);
}

/// `DocHelper` (static inline): a pill in hand and a held spare, both at 0.71428 scale.
fn doc_helper(g: G) {
    let color = ext::ftMr_SpecialN_VitaminRandom(g);
    let f = fp(g);
    let it = ext::itDrMarioPill_802C09C4(g, &f.cur_pos, color, It_Kind_DrMario_Vitamin as i32, 1, 0, f.facing_dir);
    let f = fp(g);
    f.item_gobj = it;
    ext::it_8026BAE8(f.item_gobj, 0.71428f32 * (f.x34_scale.y * f.co_attrs.model_scaling));
    let color = ext::ftMr_SpecialN_VitaminRandom(g);
    let f = fp(g);
    let it = ext::itDrMarioPill_802C09C4(g, &f.cur_pos, color, It_Kind_DrMario_Vitamin as i32, 0, 0, f.facing_dir);
    let f = fp(g);
    f.x1984_heldItemSpec = it;
    ext::it_8026BAE8(f.x1984_heldItemSpec, 0.71428f32 * (f.x34_scale.y * f.co_attrs.model_scaling));
}

fn enter(g: G, msid: ftCommon_MotionState) {
    ext::Fighter_ChangeMotionState(g, msid as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    setup_initial_state(fp(g));
}

/// `ftCo_800BECB0`: DeadDown (Fox holds his blaster).
pub fn ftCo_800BECB0(g: G) {
    enter(g, ftCo_MS_DeadDown);
    if crate::ft0892::template_kind(fp(g)) == Ft_Kind_Fox as i32 {
        fox_helper(g);
    }
}

/// `ftCo_800BED84`: empty.
pub fn ftCo_800BED84(_g: G) {}

/// `ftCo_800BED88`: DeadRight (Fox: blaster; Doc: pills).
pub fn ftCo_800BED88(g: G) {
    enter(g, ftCo_MS_DeadRight);
    let k = crate::ft0892::template_kind(fp(g));
    if k == Ft_Kind_Fox as i32 {
        fox_helper(g);
    } else if k == Ft_Kind_DrMario as i32 {
        doc_helper(g);
    }
}

/// `ftCo_800BEF00`: empty.
pub fn ftCo_800BEF00(_g: G) {}

/// `ftCo_800BEF04`: DeadUpStarIce (Ness holds his bat at 0.8 scale).
pub fn ftCo_800BEF04(g: G) {
    enter(g, ftCo_MS_DeadUpStarIce);
    if crate::ft0892::template_kind(fp(g)) == Ft_Kind_Ness as i32 {
        let bone = crate::hooks::ftparts_get_bone_index(fp(g), FtPart_RThumbNb as i32);
        let it = ext::it_802AD590(g, bone);
        let f = fp(g);
        f.item_gobj = it;
        ext::it_8026BAE8(f.item_gobj, 0.8f32 * (f.x34_scale.y * f.co_attrs.model_scaling));
    }
}

/// `ftCo_800BEFD0`: empty.
pub fn ftCo_800BEFD0(_g: G) {}
