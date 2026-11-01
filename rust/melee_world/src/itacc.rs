
//! The plain `Item` field accessors of `src/melee/it/**` (`itGetKind`, `it_8026B294`, the Tincle /
//! coin / hitbox flag pokes, the stage article table `it_804A0F60`, ...) in the C shape (an item
//! `HSD_GObj*`, `gobj->user_data` is the `Item`), for the stage TUs that call them. The item crate
//! ports the same functions over `&Item` (its `itlib`) and over its own `ext` layer; these are the
//! raw-pointer C-named ones the world TUs need, and read the same records. No float arithmetic.
//! `Mex_IndexStageItem` (the TARGET_PC m-ex stage article registration) is the [`MEX_INDEX_STAGE_ITEM`]
//! hook: unset, a custom-stage article index (`kind >= 5000`) is not registered.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::*;

/// `it_804A0F60`: the stage article table, indexed by `kind - It_Kind_Old_Kuri`.
pub static mut it_804A0F60: [*mut Article; 30] = [core::ptr::null_mut(); 30];

/// `Mex_IndexStageItem(article, index)`: register a custom stage's article (platform layer).
pub static mut MEX_INDEX_STAGE_ITEM: Option<fn(*mut Article, i32)> = None;

#[inline(always)]
pub(crate) unsafe fn ip<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}

pub use crate::tu::it_26b1::itGetKind_fp as itGetKind;
pub use crate::tu::it_26b1::it_8026B294_fp as it_8026B294;
pub use crate::tu::it_26b1::it_8026B3C0_fp as it_8026B3C0;
pub use crate::tu::it_26b1::it_8026B40C_fp as it_8026B40C;
pub use crate::tu::it_26b1::it_8026B718_fp as it_8026B718;
pub use crate::tu::it_26b1::it_8026C1E8_fp as it_8026C1E8;
pub use crate::tu::itcoll::it_802714C0_fp as it_802714C0;
pub use crate::tu::it_2725::it_80274C60_fp as it_80274C60;
pub use crate::tu::ithitbox::it_80275414_fp as it_80275414;
pub use crate::tu::ithitbox::it_802756D0_fp as it_802756D0;
pub use crate::tu::ithitbox::it_802756E0_fp as it_802756E0;
pub use crate::tu::ittincle::it_802ECC8C_fp as it_802ECC8C;
pub use crate::tu::ittincle::it_802ECC98_fp as it_802ECC98;
pub use crate::tu::ittincle::it_802ECCA4_fp as it_802ECCA4;
pub use crate::tu::itkyasarin::it_802ECD1C_fp as it_802ECD1C;
pub use crate::tu::itcoin::it_802F2014_fp as it_802F2014;
pub use crate::tu::itcoin::it_802F2020_fp as it_802F2020;
pub use crate::tu::it_26b1::it_ResetStageArticleTable_fp as it_ResetStageArticleTable;
