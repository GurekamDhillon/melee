//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcancel.c: `ftCh_Init_8015A030`, the state after a
//! cancelled grab. No float math.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::*;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTANIM_8006EBA4, ftAnim_8006EBA4, crate::adapt::ftAnim_8006EBA4, (gobj: *mut HSD_GObj));
    crate::port_hook!(FIGHTER_CHANGEMOTIONSTATE, Fighter_ChangeMotionState, crate::adapt::Fighter_ChangeMotionState, (gobj: *mut HSD_GObj, msid: i32, flags: u32, anim_start: f32, anim_speed: f32, anim_blend: f32, arg3: *mut HSD_GObj));
    crate::port_hook!(FTBOSSLIB_8015C208, ftBossLib_8015C208, crate::adapt::ftBossLib_8015C208, (gobj: *mut HSD_GObj, out: *mut Vec3));
}

/// `ftCh_Init_8015A030`.
pub fn ftCh_Init_8015A030(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Init_8015A030");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        ext::Fighter_ChangeMotionState(gobj, 0x177, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(gobj);
        {
            let mut pos: Vec3 = core::mem::zeroed();
            ext::ftBossLib_8015C208(gobj, &mut pos);
            (*fp).cur_pos.x = pos.x;
        }
        (*fp).cur_pos.y = (*da).xE0;
        (*fp).self_vel.x = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.z = 0.0;
    }
}
