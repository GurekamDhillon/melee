//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c: `ftCh_Init_80158B3C`, the finger-beam
//! windup state. No float math.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::*;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FIGHTER_CHANGEMOTIONSTATE, Fighter_ChangeMotionState, crate::adapt::Fighter_ChangeMotionState, (gobj: *mut HSD_GObj, msid: i32, flags: u32, anim_start: f32, anim_speed: f32, anim_blend: f32, arg3: *mut HSD_GObj));
    crate::port_hook!(FTANIM_SETANIMRATE, ftAnim_SetAnimRate, crate::adapt::ftAnim_SetAnimRate, (gobj: *mut HSD_GObj, rate: f32));
}

/// `ftCh_Init_80158B3C`.
pub fn ftCh_Init_80158B3C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_80158B3C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ext::Fighter_ChangeMotionState(gobj, 0x16A, 0, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_SetAnimRate(gobj, (*da).xCC_pos.x);
        (*fp).mv.ch.unk0.x8 = (*da).xC4_pos.y as i32; // C float -> s32 truncation
    }
}
