//! Ness (src/melee/ft/kinds/ftNess).

pub mod ftness {
pub use crate::tu::ftness::{ftNs_Init_GetAbsorbHeal};

use crate::chars::da;

use melee_types::{ftNessAttributes, Fighter};

}

pub mod ftnessattackhi4 {
pub use crate::tu::ftnessattackhi4::{ftNs_AttackHi4_YoyoStartTimedRehit};

use crate::chars::da;

use melee_types::{ftNessAttributes, Fighter};

}

pub mod ftnessspeciallw {
pub use crate::tu::ftnessspeciallw::ftNs_SpecialAirLwStart_Phys_fp;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftNessAttributes, Fighter};

}

pub mod ftnessspecialn {
pub use crate::tu::ftnessspecialn::pk_flash_air_phys;
pub use crate::tu::ftnessspecialn::ftNs_SpecialAirNEnd_Phys_fp;
pub use crate::tu::ftnessspecialn::ftNs_SpecialAirNRelease_Phys_fp;
pub use crate::tu::ftnessspecialn::ftNs_SpecialAirNStart_Phys_fp;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftNessAttributes, Fighter};

}
