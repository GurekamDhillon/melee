
//! Port round 12 (night 3, round 4): hand ports of 15 item functions whose callees are all
//! ported (this crate's `it_80272CC0` / `it_80277040`) or native elsewhere in the workspace
//! (MSL `sqrtf` / `sqrtf_accurate` in `melee_runtime::gekko`; `PSVECAdd`, mtx_pc.c's plain
//! component adds, written out). Verified against verbatim C copies
//! (tests/c_oracle/round12_bodies.h, extracted by gen_round12.py) in tests/oracle_round12.rs.
//!
//! Conventions as in round 3 / round 10 / round 11: a followed C pointer makes the function
//! `unsafe` (precondition: valid, as in C); globals are parameters: `it_804D6D28` ->
//! `&ItemCommonData`, `Item_804A0C64` -> `&mut HSD_ObjAllocUnk`, `it_804D6D24` -> the article
//! table pointer, the `it_804D6D08` / `it_804D6D0C` / `it_804D6D18` counters -> `&mut`,
//! `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head (TODO(group1), as itlib3.rs).
//!
//! `it_8026B1D4`'s `my_sqrtf` is MSL `sqrtf` (the PC build has no MUST_MATCH, it_26B1.c:59);
//! itmaplib.c's `sqrtf_accurate_sp18` is the same four Newton steps as MSL `sqrtf_accurate`.
//!
//! Fused sites (shipping-build contraction): `it_8026B1D4` 3 f32 (`vx*vx + vy*vy + vz*vz` ->
//! `fmadds(vz, vz, fmadds(vx, vx, vy*vy))`, `ret += speed * k5` -> `fmadds(speed, k5, ret)`),
//! `it_8026C258` 1, `it_802775F0` 1, `it_80277544` 1 (each `SQ(x) + SQ(y)` -> `fmadds(x, x, y*y)`).
#![allow(non_snake_case)]

use crate::itkinds10::it_80277040;

use crate::itkinds11::it_80272cc0;

use crate::layout::{
    Article, Fighter, HSD_GObj, HSD_GObjEvent, HSD_GObjInteraction, HSD_JObj, HSD_ObjAllocUnk,
    HitCapsule, Item, ItemCommonData, Vec as GcVec, itLGun_ItemVars, It_Kind_L_Gun,
};

use melee_core::fp::fmadds;

use melee_runtime::gekko::{sqrtf, sqrtf_accurate};

#[inline(always)]
pub(crate) unsafe fn item<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}

pub use crate::tu::it_26B1::it_8026b1d4;
pub use crate::tu::it_26B1::it_8026b7d8;
pub use crate::tu::it_26B1::it_8026b7e0;
pub use crate::tu::it_26B1::it_8026c1d4;
pub use crate::tu::it_26B1::it_8026c258;
pub use crate::tu::itcoll::it_8026f9a0;
pub use crate::tu::it_2725::it_80272c90;
pub use crate::tu::it_2725::it_802746f8;
pub use crate::tu::it_2725::it_80274ed8;
pub use crate::tu::it_2725::it_80274ee8;
pub use crate::tu::it_2725::it_80274f28_fp as it_80274f28;
pub use crate::tu::itmaplib::it_80277544_fp as it_80277544;
pub use crate::tu::itmaplib::it_802775f0;
pub use crate::tu::it_279C::it_8027b070;
pub use crate::tu::item::item_8026784c;
