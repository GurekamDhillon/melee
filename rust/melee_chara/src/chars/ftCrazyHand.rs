//! Crazy Hand (src/melee/ft/kinds/ftCrazyHand).

/// `if (fp->mv.ch.unk0.x18 == 0) fp->self_vel.x = fp->self_vel.y = fp->self_vel.z = 0;`
/// (the C chain stores z, then y, then x; all three are 0.0f).
#[inline]
fn stop_if_x18_zero(fp: &mut melee_types::Fighter) {
    if unsafe { fp.mv.ch.unk0.x18 } == 0.0 {
        fp.self_vel.z = 0.0;
        fp.self_vel.y = 0.0;
        fp.self_vel.x = 0.0;
    }
}

pub mod ftcrazyhandbackairplane1 {
    use melee_types::Fighter;
    pub fn ftCh_BackAirplane1_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}

pub mod ftcrazyhandfingerbeam {
    use melee_types::Fighter;
    pub fn ftCh_FingerGun1_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}

pub mod ftcrazyhandslam {
    use melee_types::Fighter;
    pub fn ftCh_TagApplaud_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}
