//! Peach (src/melee/ft/kinds/ftPeach).

pub mod ftpeachfloat {
pub use crate::tu::ftpeachfloat::ftPe_Float_Phys_fp as ftPe_Float_Phys;

use melee_fighter::ftcommon as co;

use melee_types::Fighter;

}

pub mod ftpeachfloatfall {
pub use crate::tu::ftpeachfloatfall::{getFloatDir};

use melee_fighter::hooks::common_data;

use melee_types::{ftPe_MS_FloatFallB, ftPe_MS_FloatFallF, ftPeach_MotionState, Fighter};

}

pub mod ftpeachspecialhi {
pub use crate::tu::ftpeachspecialhi::{checkCmdVar2};

use melee_types::Fighter;

}

pub mod ftpeachspecialn {
pub use crate::tu::ftpeachspecialn::ftPe_SpecialAirNHit_Phys_fp;

pub use crate::tu::ftpeachspecialn::{ftPe_SpecialN_IsActive};

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftPe_DatAttrs, ftPe_MS_SpecialAirNHit, ftPe_MS_SpecialN, Fighter};

}

pub mod ftpeachspecials {
pub use crate::tu::ftpeachspecials::ftPe_SpecialAirSJump_Phys_fp;
pub use crate::tu::ftpeachspecials::ftPe_SpecialSStart_Phys_fp;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftPe_DatAttrs, Fighter};

}
