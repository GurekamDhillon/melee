//! Mr. Game & Watch (src/melee/ft/kinds/ftGameWatch). The `Item*Remove` checks return true when
//! the fighter has left the motion range that owns the item (judgement hammer, torch, ...).

#[inline]
pub(crate) fn outside(fp: &melee_types::Fighter, lo: u32, hi: u32) -> bool {
    let msid = fp.motion_id as i32;
    !(msid >= lo as i32 && msid <= hi as i32)
}

pub mod ftgamewatchattack11 {
pub use crate::tu::ftgamewatchattack11::{ftGw_Attack11_ItemGreenhouse_CheckAll, ftGw_Attack11_ItemGreenhouse_CheckAttack11};

use melee_types::{ftGw_MS_Attack100End, ftGw_MS_Attack11, Fighter};

}

pub mod ftgamewatchattackair {
pub use crate::tu::ftgamewatchattackair::{ftGw_AttackAirN_ItemCheckParachuteRemove, ftGw_AttackAirN_ItemCheckSparkyRemove, ftGw_AttackAirN_ItemCheckTurtleRemove};

use melee_types::*;

}

pub mod ftgamewatchattacklw3 {
pub use crate::tu::ftgamewatchattacklw3::{ftGw_AttackLw3_ItemCheckManholeRemove};

use melee_types::{ftGw_MS_AttackLw3, Fighter};

}

pub mod ftgamewatchattacks4 {
pub use crate::tu::ftgamewatchattacks4::{ftGw_AttackS4_ItemCheckTorchRemove};

use melee_types::{ftGw_MS_AttackS4, Fighter};

}

pub mod ftgamewatchspeciallw {
pub use crate::tu::ftgamewatchspeciallw::{ftGw_SpecialLw_ItemCheckPanicRemove};

use melee_types::{ftGw_MS_SpecialAirLwShoot, ftGw_MS_SpecialLwShoot, Fighter};

}

pub mod ftgamewatchspecials {
pub use crate::tu::ftgamewatchspecials::{ftGw_SpecialS_ItemCheckJudgementRemove};

use melee_types::{ftGw_MS_SpecialAirS9, ftGw_MS_SpecialS1, Fighter};

}
