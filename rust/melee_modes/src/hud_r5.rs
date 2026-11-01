
//! Round 5: if/ty leaf functions over nested pointers (textdraw.c, ifstock.c, toy.c), verified
//! bit for bit against the C by `tests/oracle_r5.rs`.
use melee_types as mt;

use mt::{DevText, HSD_MObj, TyDisplayData, _GXColor};

pub use crate::tu::textdraw::DevText_AddToList_fp as DevText_AddToList;
pub use crate::tu::textdraw::DevText_NumToStr_fp as DevText_NumToStr;
pub use crate::tu::textdraw::DevText_StrLen_fp as DevText_StrLen;
pub use crate::tu::toy::_Toy_8030FE48_link_entries_fp as _Toy_8030FE48_link_entries;
pub use crate::tu::ifstock::copyColor_fp as copyColor;
