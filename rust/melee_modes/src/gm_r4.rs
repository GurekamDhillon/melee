
//! Round 4: gm (and textlib) callers of rounds 1-3. Verified bit for bit against the C
//! (`tests/oracle_r4.rs`, oracle/gen_oracle_r4.py: the callees run as the game's own C there).
//! No fused site in any of these (`tools/rust-port/fma_sites.py`).
#![allow(non_snake_case)]

use crate::gm::bitset_mask;

use crate::gm_ptr::{fn_80165418, fn_801654A0, fn_80167194, gm_801B05F4, gm_801B07E8};

use crate::hud::DevText_Clamp;

use crate::lb::lbTime_8000AF74;

use melee_types as mt;

use mt::{CSSData, DevText, MatchEnd, PlayerInitData, ResultsData};

pub use crate::tu::textlib::DevText_SetCursorX_fp as DevText_SetCursorX;
pub use crate::tu::gmmain_lib::bitset_clear_fp as bitset_clear;
pub use crate::tu::gmmain_lib::bitset_set_fp as bitset_set;
pub use crate::tu::gmmain_lib::bitset_test_fp as bitset_test;
pub use crate::tu::gmmain_lib::bitset_test_word_fp as bitset_test_word;
pub use crate::tu::gmresult::fn_801771C0_fp as fn_801771C0;
pub use crate::tu::gmvsmelee::gmVsMelee_UpdateKOCounts_fp as gmVsMelee_UpdateKOCounts;
pub use crate::tu::gmtrainingmode::gm_801B07E8_layer_fp as gm_801B07E8_layer;
pub use crate::tu::gm_1601::gm_MatchHasMultipleWinners_fp as gm_MatchHasMultipleWinners;
pub use crate::tu::gm_1B03::gm_SetupCpuPlayer_fp as gm_SetupCpuPlayer;
pub use crate::tu::gm_1B03::gm_SetupHumanPlayer_fp as gm_SetupHumanPlayer;
