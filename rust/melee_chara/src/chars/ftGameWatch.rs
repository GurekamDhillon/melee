//! Mr. Game & Watch (src/melee/ft/kinds/ftGameWatch). The `Item*Remove` checks return true when
//! the fighter has left the motion range that owns the item (judgement hammer, torch, ...).

#[inline]
fn outside(fp: &melee_types::Fighter, lo: u32, hi: u32) -> bool {
    let msid = fp.motion_id as i32;
    !(msid >= lo as i32 && msid <= hi as i32)
}

pub mod ftgamewatchattack11 {
    use melee_types::{ftGw_MS_Attack100End, ftGw_MS_Attack11, Fighter};
    pub fn ftGw_Attack11_ItemGreenhouse_CheckAll(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_Attack11 as u32, ftGw_MS_Attack100End as u32)
    }
    pub fn ftGw_Attack11_ItemGreenhouse_CheckAttack11(fp: &Fighter) -> bool {
        fp.motion_id as u32 != ftGw_MS_Attack11 as u32
    }
}

pub mod ftgamewatchattackair {
    use melee_types::*;
    pub fn ftGw_AttackAirN_ItemCheckParachuteRemove(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_AttackAirN as u32, ftGw_MS_LandingAirN as u32)
    }
    pub fn ftGw_AttackAirN_ItemCheckSparkyRemove(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_AttackAirHi as u32, ftGw_MS_LandingAirHi as u32)
    }
    pub fn ftGw_AttackAirN_ItemCheckTurtleRemove(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_AttackAirB as u32, ftGw_MS_LandingAirB as u32)
    }
}

pub mod ftgamewatchattacklw3 {
    use melee_types::{ftGw_MS_AttackLw3, Fighter};
    pub fn ftGw_AttackLw3_ItemCheckManholeRemove(fp: &Fighter) -> bool {
        fp.motion_id as u32 != ftGw_MS_AttackLw3 as u32
    }
}

pub mod ftgamewatchattacks4 {
    use melee_types::{ftGw_MS_AttackS4, Fighter};
    pub fn ftGw_AttackS4_ItemCheckTorchRemove(fp: &Fighter) -> bool {
        fp.motion_id as u32 != ftGw_MS_AttackS4 as u32
    }
}

pub mod ftgamewatchspeciallw {
    use melee_types::{ftGw_MS_SpecialAirLwShoot, ftGw_MS_SpecialLwShoot, Fighter};
    pub fn ftGw_SpecialLw_ItemCheckPanicRemove(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_SpecialLwShoot as u32, ftGw_MS_SpecialAirLwShoot as u32)
    }
}

pub mod ftgamewatchspecials {
    use melee_types::{ftGw_MS_SpecialAirS9, ftGw_MS_SpecialS1, Fighter};
    pub fn ftGw_SpecialS_ItemCheckJudgementRemove(fp: &Fighter) -> bool {
        super::outside(fp, ftGw_MS_SpecialS1 as u32, ftGw_MS_SpecialAirS9 as u32)
    }
}
