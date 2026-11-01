//! Sheik (src/melee/ft/kinds/ftSeak).

pub mod ftseakspecialhi {
pub use crate::tu::ftseakspecialhi::ftSk_SpecialAirHiStart_0_Phys_fp as ftSk_SpecialAirHiStart_0_Phys;
pub use crate::tu::ftseakspecialhi::ftSk_SpecialAirHi_Phys_fp as ftSk_SpecialAirHi_Phys;
pub use crate::tu::ftseakspecialhi::ftSk_SpecialHiStart_1_Phys_fp as ftSk_SpecialHiStart_1_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftSeakAttributes, Fighter};

}

pub mod ftseakspeciallw {
pub use crate::tu::ftseakspeciallw::ftSk_SpecialAirLw2_Phys_fp;
pub use crate::tu::ftseakspeciallw::ftSk_SpecialAirLw_Phys_fp;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftSeakAttributes, Fighter};

}

pub mod ftseakspecials {
pub use crate::tu::ftseakspecials::ftSk_SpecialAirSStart_Phys_fp;

pub use crate::tu::ftseakspecials::{ftSk_SpecialS_80110F58, ftSk_SpecialS_80110F64, ftSk_SpecialS_ZeroHitboxPositions};

use melee_fighter::ftcommon as co;

use melee_types::Fighter;

}
