//! Menu TU access to the shared jobj.h inlines; scalar translation getters supplement hsd.
#![allow(non_snake_case, clippy::missing_safety_doc)]
use melee_types::HSD_JObj;
pub use super::hsd::{
    HSD_JObjSetTranslate, HSD_JObjSetTranslateWithMtxDirty,
    HSD_JObjSetTranslateX, HSD_JObjSetTranslateY, HSD_JObjSetTranslateZ,
    HSD_JObjSetTranslateXWithMtxDirty, HSD_JObjSetTranslateYWithMtxDirty,
    HSD_JObjSetTranslateZWithMtxDirty, HSD_JObjGetTranslation,
};
pub unsafe fn HSD_JObjGetTranslationX(jobj: *mut HSD_JObj) -> f32 {
    super::hsd::hsd_assert(!jobj.is_null(), 582, c"jobj");
    (*jobj).translate.x
}
pub unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
    super::hsd::hsd_assert(!jobj.is_null(), 588, c"jobj");
    (*jobj).translate.y
}
pub unsafe fn HSD_JObjGetTranslationZ(jobj: *mut HSD_JObj) -> f32 {
    super::hsd::hsd_assert(!jobj.is_null(), 594, c"jobj");
    (*jobj).translate.z
}

/// `MenuInput` (mn/inlines.h).
pub const MenuInput_Up: i32 = 1 << 0;
pub const MenuInput_Down: i32 = 1 << 1;
pub const MenuInput_Left: i32 = 1 << 2;
pub const MenuInput_Right: i32 = 1 << 3;
pub const MenuInput_Confirm: i32 = 1 << 4;
pub const MenuInput_Back: i32 = 1 << 5;
pub const MenuInput_LTrigger: i32 = 1 << 6;
pub const MenuInput_RTrigger: i32 = 1 << 7;
pub const MenuInput_StartButton: i32 = 1 << 8;
pub const MenuInput_AButton: i32 = 1 << 9;
pub const MenuInput_XButton: i32 = 1 << 10;
pub const MenuInput_YButton: i32 = 1 << 11;
