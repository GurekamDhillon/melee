
//! Round 3: `src/melee/mn/` leaf functions that take pointers to game structs. Conventions and
//! verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
#![allow(non_snake_case)]

use melee_core::fp::fmadds;

use melee_types as mt;

use mt::{AnimLoopSettings, HSD_Text, Vec as Vec3};

/// The two bytes of the C's `"　"` (U+3000, the full-width space) as the shipping build
/// compiles them: the source file is UTF-8 and clang's execution charset is UTF-8, so they
/// are E3 80 (the GameCube's Shift-JIS build had 81 40).
pub const FULLWIDTH_SPACE: [u8; 2] = [0xE3, 0x80];

pub use crate::tu::mnmain::GetAnimEndFrame_fp as GetAnimEndFrame;
pub use crate::tu::mnmain::GetAnimStartFrame_fp as GetAnimStartFrame;
pub use crate::tu::mnnamenew::GlyphVariantCount_fp as GlyphVariantCount;
pub use crate::tu::mnname::checkStringRest_fp as checkStringRest;
pub use crate::tu::mngallery::float_order_helper;
pub use crate::tu::mnsound::getVolumePosition_fp as getVolumePosition;
pub use crate::tu::mncharsel::loadStickValue;
pub use crate::tu::mncharsel::mnCharSel_MexPackRows;
pub use crate::tu::mndiagram3::mnDiagram3_GetRowStat;
pub use crate::tu::mndiagram::mnDiagram_TextSetPos;
pub use crate::tu::mninfobonus::mnInfoBonus_80252ADC_inline_fp as mnInfoBonus_80252ADC_inline;
pub use crate::tu::mninfo::mnInfo_80251AFC_inline_3_fp as mnInfo_80251AFC_inline_3;
pub use crate::tu::mninfo::mnInfo_80252720_fp as mnInfo_80252720;
pub use crate::tu::mnnamenew::mnNameNew_8023BAA8_fp as mnNameNew_8023BAA8;
pub use crate::tu::mnname::mnName_80238C34_inline_fp as mnName_80238C34_inline;
pub use crate::tu::mnstagesw::mnStageSw_CountEnabled_fp as mnStageSw_CountEnabled;
pub use crate::tu::mn_22EC::mn_8022F410_fp as mn_8022F410;
pub use crate::tu::mn_22EC::mn_8022F470_fp as mn_8022F470;
pub use crate::tu::mnname::readNameTerminator_fp as readNameTerminator;
pub use crate::tu::mninfobonus::textSize_fp as textSize;
