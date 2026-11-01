//! Crazy Hand (src/melee/ft/kinds/ftCrazyHand).

/// `if (fp->mv.ch.unk0.x18 == 0) fp->self_vel.x = fp->self_vel.y = fp->self_vel.z = 0;`
/// (the C chain stores z, then y, then x; all three are 0.0f).
#[inline]
pub(crate) fn stop_if_x18_zero(fp: &mut melee_types::Fighter) {
    if unsafe { fp.mv.ch.unk0.x18 } == 0.0 {
        fp.self_vel.z = 0.0;
        fp.self_vel.y = 0.0;
        fp.self_vel.x = 0.0;
    }
}

pub mod ftcrazyhandbackairplane1 {
pub use crate::tu::ftcrazyhandbackairplane1::ftCh_BackAirplane1_Coll_fp as ftCh_BackAirplane1_Coll;

use melee_types::Fighter;

}

pub mod ftcrazyhandfingerbeam {
pub use crate::tu::ftcrazyhandfingerbeam::ftCh_FingerGun1_Coll;

use melee_types::Fighter;

}

pub mod ftcrazyhandslam {
pub use crate::tu::ftcrazyhandslam::ftCh_TagApplaud_Coll_fp as ftCh_TagApplaud_Coll;

use melee_types::Fighter;

}
