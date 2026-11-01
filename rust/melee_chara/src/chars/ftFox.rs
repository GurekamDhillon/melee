//! Fox (src/melee/ft/kinds/ftFox).

pub mod ftfoxspecialhi {
pub use crate::tu::ftfoxspecialhi::ftFx_SpecialHiHoldAir_Phys_fp as ftFx_SpecialHiHoldAir_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftFox_DatAttrs, Fighter};

}

pub mod ftfoxspecials {
pub use crate::tu::ftfoxspecials::ftFx_SpecialAirSStart_Phys_fp as ftFx_SpecialAirSStart_Phys;
pub use crate::tu::ftfoxspecials::ftFx_SpecialS_CheckGhostRemove_fp as ftFx_SpecialS_CheckGhostRemove;
pub use crate::tu::ftfoxspecials::ftFx_SpecialS_CopyGhostPosIndexed_fp as ftFx_SpecialS_CopyGhostPosIndexed;
pub use crate::tu::ftfoxspecials::ftFx_SpecialS_GetCmdVar2_fp as ftFx_SpecialS_GetCmdVar2;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftFox_DatAttrs, ftFx_MS_SpecialAirSEnd, ftFx_MS_SpecialSStart, Fighter, Vec3};

}
