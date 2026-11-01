//! Master Hand (src/melee/ft/kinds/ftMasterHand).

#[inline]
pub(crate) fn stop_if_x18_zero(fp: &mut melee_types::Fighter) {
    if unsafe { fp.mv.mh.unk0.x18 } == 0.0 {
        fp.self_vel.z = 0.0;
        fp.self_vel.y = 0.0;
        fp.self_vel.x = 0.0;
    }
}

pub mod ftmasterhandbackdisappear {
pub use crate::tu::ftmasterhandbackdisappear::ftMh_Cancel_Coll_fp as ftMh_Cancel_Coll;

use melee_types::Fighter;

}

pub mod ftmasterhandsqueeze {
pub use crate::tu::ftmasterhandsqueeze::ftMh_Squeeze_Coll_fp as ftMh_Squeeze_Coll;

use melee_types::Fighter;

}

pub mod ftmasterhandtagrockpaper {
pub use crate::tu::ftmasterhandtagrockpaper::ftMh_TagCancel_Coll_fp as ftMh_TagCancel_Coll;

use melee_types::Fighter;

}
