//! Kirby (src/melee/ft/kinds/ftKirby), including the copied-ability TUs.

pub mod ftkirby {
pub use crate::tu::ftkirby::{ftKb_Init_UnkDemoCallbacks0};

}

pub mod ftkirbyspecialgamewatch {
pub use crate::tu::ftkirbyspecialgamewatch::{ftKb_SpecialNGw_8010D160};

use melee_types::{ftKb_MS_GwSpecialAirN, ftKb_MS_GwSpecialN, Fighter};

}

pub mod ftkirbyspecialhi {
pub use crate::tu::ftkirbyspecialhi::ftKb_SpecialAirHi3_Phys_fp as ftKb_SpecialAirHi3_Phys;
pub use crate::tu::ftkirbyspecialhi::ftKb_SpecialHi3_Phys_fp as ftKb_SpecialHi3_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftKb_DatAttrs, Fighter};

}

pub mod ftkirbyspecialmars {
pub use crate::tu::ftkirbyspecialmars::fn_8010B2E8_fp as fn_8010B2E8;

use melee_types::Fighter;

}

pub mod ftkirbyspecialn {
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F58AC_fp as ftKb_SpecialN_800F58AC;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F597C_fp as ftKb_SpecialN_800F597C;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5A04_fp as ftKb_SpecialN_800F5A04;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5A38_fp as ftKb_SpecialN_800F5A38;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5A60_fp as ftKb_SpecialN_800F5A60;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5A88_fp as ftKb_SpecialN_800F5A88;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5AB0_fp as ftKb_SpecialN_800F5AB0;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5AF0_fp as ftKb_SpecialN_800F5AF0;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B00_fp as ftKb_SpecialN_800F5B00;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B10_fp as ftKb_SpecialN_800F5B10;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B20_fp as ftKb_SpecialN_800F5B20;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B3C_fp as ftKb_SpecialN_800F5B3C;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B4C_fp as ftKb_SpecialN_800F5B4C;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F5B5C_fp as ftKb_SpecialN_800F5B5C;

use crate::chars::da;

use melee_core::fp::fmadds;

use melee_types::{ftKb_DatAttrs, ftKb_MS_EatFall, ftKb_MS_EatWait, Fighter, Vec2, Vec3};

}

pub mod ftkirbyspecialness {
pub use crate::tu::ftkirbyspecialness::ftKb_NsSpecialAirNStart_Phys_fp as ftKb_NsSpecialAirNStart_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftKb_DatAttrs, Fighter};

}

pub mod ftkirbyspecialpeach {
pub use crate::tu::ftkirbyspecialpeach::ftKb_PeSpecialAirLwHit_Phys_fp as ftKb_PeSpecialAirLwHit_Phys;
pub use crate::tu::ftkirbyspecialpeach::ftKb_SpecialNPe_8010C4AC_fp as ftKb_SpecialNPe_8010C4AC;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftKb_DatAttrs, ftKb_MS_PeSpecialAirLwHit, ftKb_MS_PeSpecialLw, Fighter};

}

pub mod ftkirbyspecialpurin {
pub use crate::tu::ftkirbyspecialpurin::fall_cd;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialAirNEndR_Phys_fp as ftKb_PrSpecialAirNEndR_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialAirNFull_Phys_fp as ftKb_PrSpecialAirNFull_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialAirNLoop_Phys_fp as ftKb_PrSpecialAirNLoop_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialAirNStart_Phys_fp as ftKb_PrSpecialAirNStart_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialNEnd_Phys_fp as ftKb_PrSpecialNEnd_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialNHit_Phys_fp as ftKb_PrSpecialNHit_Phys;
pub use crate::tu::ftkirbyspecialpurin::ftKb_PrSpecialNStart_Phys_fp as ftKb_PrSpecialNStart_Phys;

use crate::chars::da;

use melee_fighter::ftcommon as co;

use melee_types::{ftCommonData, ftKb_DatAttrs, Fighter};

}

pub mod ftkirbyspecials {
pub use crate::tu::ftkirbyspecials::ftKb_SpecialN_800F5800_fp as ftKb_SpecialN_800F5800;

use melee_types::{Fighter, Vec3};

}

pub mod ftkirbyspecialyoshi {
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_80109260_fp as ftKb_SpecialNYs_80109260;
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_80109290_fp as ftKb_SpecialNYs_80109290;
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_8010929C_fp as ftKb_SpecialNYs_8010929C;
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_801092AC_fp as ftKb_SpecialNYs_801092AC;
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_801092BC_fp as ftKb_SpecialNYs_801092BC;
pub use crate::tu::ftkirbyspecialyoshi::ftKb_SpecialNYs_801092CC_fp as ftKb_SpecialNYs_801092CC;

use crate::chars::da;

use melee_types::{ftKb_DatAttrs, Fighter, Vec3};

}
