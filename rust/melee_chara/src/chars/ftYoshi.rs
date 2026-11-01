//! Yoshi (src/melee/ft/kinds/ftYoshi).

pub mod ftyoshi {
pub use crate::tu::ftyoshi::ftYs_Init_8012BAC0_fp as ftYs_Init_8012BAC0;

use crate::chars::da;

use melee_types::{ftYoshiAttributes, Fighter};

}

pub mod ftyoshispecialn {
pub use crate::tu::ftyoshispecialn::{ftYs_SpecialN_GetDatAttr18, ftYs_SpecialN_GetDatAttr1C, ftYs_SpecialN_GetDatAttr20, ftYs_SpecialN_GetDatAttr24, ftYs_SpecialN_GetFacingDir, ftYs_SpecialN_SetupItemVel};

use crate::chars::da;

use melee_types::{ftYoshiAttributes, ftYs_DatAttrs, Fighter, Vec3};

}

pub mod ftyoshispecials {
pub use crate::tu::ftyoshispecials::fn_8012EFF4_fp as fn_8012EFF4;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSEnd_Phys_fp as ftYs_SpecialAirSEnd_Phys;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSLanding_IASA_fp;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSLanding_Phys_fp as ftYs_SpecialAirSLanding_Phys;
pub use crate::tu::ftyoshispecials::air_s_loop_phys_fp as air_s_loop_phys;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSLoop_2_Phys_fp as ftYs_SpecialAirSLoop_2_Phys;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSLoop_3_Phys_fp as ftYs_SpecialAirSLoop_3_Phys;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSStart_0_Phys_fp;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSStart_1_IASA_fp;
pub use crate::tu::ftyoshispecials::ftYs_SpecialAirSStart_1_Phys_fp as ftYs_SpecialAirSStart_1_Phys;

use crate::chars::{abs_c, da};

use melee_fighter::ftcommon as co;

use melee_types::{ftYoshiAttributes, Fighter};

}
