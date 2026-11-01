
//! Character attribute accessors the common fighter states call (`ftKb_SpecialN_*`, `ftKp_SpecialS_*`,
//! `ftYs_SpecialN_*`, `ftYs_Init_8012BAC0`): reads of the fighter's `dat_attrs` block / the loaded
//! character data, moved down from `melee_chara` so the fighter TUs need no upward dependency.
//! The bodies are the ones `melee_chara` checks against its C oracle.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::types::*;

use melee_core::fp::fmadds;

/// `(T*) fp->dat_attrs`: the character's special attributes block (owned by the fighter data).
#[inline(always)]
pub fn da<T>(fp: &Fighter) -> &'static T {
    unsafe { &*(fp.dat_attrs as *const T) }
}

/// Yoshi's loaded extra-attribute block (`gFtDataList[Ft_Kind_Yoshi]->ext_attr`).
pub(crate) unsafe fn ext_attr() -> &'static ftYoshiAttributes {
    &*((*crate::tu::ftdata::gFtDataList[Ft_Kind_Yoshi as usize]).ext_attr as *const ftYoshiAttributes)
}

pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F58AC_fp as ftKb_SpecialN_800F58AC;
pub use crate::tu::ftkirbyspecialn::ftKb_SpecialN_800F597C_fp as ftKb_SpecialN_800F597C;
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
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132DC0_fp as ftKp_SpecialS_80132DC0;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132DD0_fp as ftKp_SpecialS_80132DD0;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132DE0_fp as ftKp_SpecialS_80132DE0;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132DF0_fp as ftKp_SpecialS_80132DF0;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132E00_fp as ftKp_SpecialS_80132E00;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132E10_fp as ftKp_SpecialS_80132E10;
pub use crate::tu::ftkoopa::ftKp_SpecialS_80132E20_fp as ftKp_SpecialS_80132E20;
pub use crate::tu::ftyoshi::ftYs_Init_8012BAC0_fp as ftYs_Init_8012BAC0;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_8012CD40_fp as ftYs_SpecialN_8012CD40;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_8012CDB4_fp as ftYs_SpecialN_8012CDB4;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetDatAttr18_fp as ftYs_SpecialN_GetDatAttr18;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetDatAttr1C_fp as ftYs_SpecialN_GetDatAttr1C;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetDatAttr20_fp as ftYs_SpecialN_GetDatAttr20;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetDatAttr24_fp as ftYs_SpecialN_GetDatAttr24;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetExtAttr28_fp as ftYs_SpecialN_GetExtAttr28;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetExtAttr2C_fp as ftYs_SpecialN_GetExtAttr2C;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetExtAttr34_fp as ftYs_SpecialN_GetExtAttr34;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetExtAttr38_fp as ftYs_SpecialN_GetExtAttr38;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_GetFacingDir_fp as ftYs_SpecialN_GetFacingDir;
pub use crate::tu::ftyoshispecialn::ftYs_SpecialN_SetupItemVel_fp as ftYs_SpecialN_SetupItemVel;
