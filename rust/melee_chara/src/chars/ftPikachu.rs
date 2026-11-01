//! Pikachu (src/melee/ft/kinds/ftPikachu).

pub mod ftpikachuspecialhi {
pub use crate::tu::ftpikachuspecialhi::ftPk_SpecialAirHiEnd_Phys_fp as ftPk_SpecialAirHiEnd_Phys;
pub use crate::tu::ftpikachuspecialhi::ftPk_SpecialAirHiStart0_Phys_fp as ftPk_SpecialAirHiStart0_Phys;
pub use crate::tu::ftpikachuspecialhi::ftPk_SpecialHiStart1_Phys_fp as ftPk_SpecialHiStart1_Phys;
pub use crate::tu::ftpikachuspecialhi::ftPk_SpecialHi_UpdateVel_fp as ftPk_SpecialHi_UpdateVel;

use crate::chars::{abs_c, da};

use melee_fighter::ftcommon as co;

use melee_types::{ftPikachuAttributes, Fighter};

}

pub mod ftpikachuspeciallw {
pub use crate::tu::ftpikachuspeciallw::ftPk_SpecialAirLwLoop1_Phys_fp as ftPk_SpecialAirLwLoop1_Phys;
pub use crate::tu::ftpikachuspeciallw::ftPk_SpecialLw_SetState_Unk0_fp as ftPk_SpecialLw_SetState_Unk0;
pub use crate::tu::ftpikachuspeciallw::ftPk_SpecialLw_SetState_Unk1_fp as ftPk_SpecialLw_SetState_Unk1;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftPikachuAttributes, Fighter};

}

pub mod ftpikachuspecials {
pub use crate::tu::ftpikachuspecials::ftPk_SpecialAirS1_Phys_fp as ftPk_SpecialAirS1_Phys;
pub use crate::tu::ftpikachuspecials::ftPk_SpecialAirSEnd_Phys_fp as ftPk_SpecialAirSEnd_Phys;
pub use crate::tu::ftpikachuspecials::ftPk_SpecialSEnd_Phys_fp as ftPk_SpecialSEnd_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftPikachuAttributes, Fighter};

}
