//! src/melee/ft/ftCo_800C703C.c: one field setter (reads `p_ftCommonData`).
use crate::hooks::common_data;
use crate::types::{Fighter, HSD_GObj};

/// `ftCo_800C703C`: seed `xA4_unk_vel` from the self velocity and common data, both
/// `dmg.x1948/x194C` from `x6CC`.
pub fn ftco_800c703c(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftco_800c703c");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    let cd = common_data();
    fp.xA4_unk_vel.x = fp.self_vel.x;
    fp.xA4_unk_vel.y = cd.x6D0;
    fp.xA4_unk_vel.z = 0.0;
    fp.dmg.x194C = cd.x6CC;
    fp.dmg.x1948 = fp.dmg.x194C;
}
