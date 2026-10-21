//! Master Hand (src/melee/ft/kinds/ftMasterHand).

#[inline]
fn stop_if_x18_zero(fp: &mut melee_types::Fighter) {
    if unsafe { fp.mv.mh.unk0.x18 } == 0.0 {
        fp.self_vel.z = 0.0;
        fp.self_vel.y = 0.0;
        fp.self_vel.x = 0.0;
    }
}

pub mod ftmasterhandbackdisappear {
    use melee_types::Fighter;
    pub fn ftMh_Cancel_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}

pub mod ftmasterhandsqueeze {
    use melee_types::Fighter;
    pub fn ftMh_Squeeze_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}

pub mod ftmasterhandtagrockpaper {
    use melee_types::Fighter;
    pub fn ftMh_TagCancel_Coll(fp: &mut Fighter) {
        super::stop_if_x18_zero(fp)
    }
}
