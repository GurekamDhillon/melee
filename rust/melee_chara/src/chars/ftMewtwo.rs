//! Mewtwo (src/melee/ft/kinds/ftMewtwo).

pub mod ftmewtwospecialhi {
    use melee_fighter::ftcommon as co;
    use melee_types::Fighter;

    pub fn ftMt_SpecialHiLost_Phys(fp: &mut Fighter) {
        co::set_self_movement_from_grounded_movement(fp);
    }
}
