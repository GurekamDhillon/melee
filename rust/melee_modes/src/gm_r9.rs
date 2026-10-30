//! Round 9 (night 3 round 5): gm setters/getters, several over records the .c files define,
//! verified bit for bit against the C by `tests/oracle_r9.rs`.
use core::ffi::c_void;
use melee_types::HSD_GObj;

/// gmtoulib.c `fn_8018FBD8`: `gobj->user_data = (void*) arg1` (the s32 sign-extends, as the
/// native C's int-to-pointer cast does).
pub fn fn_8018FBD8(gobj: &mut HSD_GObj, arg1: i32) {
    gobj.user_data = arg1 as isize as *mut c_void;
}

/// gm_1879.c `gm_80187F48_EnterData`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct gm_80187F48_EnterData {
    pub x0: u8,
    pub stage_index: u8,
    pub stkind: u16,
}

melee_core::gc_layout!(gm_80187F48_EnterData, "gm_80187F48_EnterData", size 0x4, align 2, [
    x0 @ 0x0: 1 "u8",
    stage_index @ 0x1: 1 "u8",
    stkind @ 0x2: 2 "u16",
]);

/// gm_1879.c `getStKind`.
pub fn getStKind(data: &gm_80187F48_EnterData) -> i32 {
    data.stkind as i32
}

/// gmstaffroll.c `StaffEntryData`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StaffEntryData {
    pub x0: u8,
    pub pad_01: [u8; 3],
    /// `StaffEntryCheck`: `bool (*)(s16)`
    pub check: Option<unsafe extern "C" fn(i16) -> bool>,
    pub check_arg: i16,
    pub pad_0A: [u8; 2],
    pub alt_page: i32,
    pub has_check: u8,
    pub x11: [u8; 2],
    pub pad_13: u8,
}

melee_core::gc_layout!(StaffEntryData, "StaffEntryData", size 0x14, align 4, [
    x0 @ 0x0: 1 "u8",
    pad_01 @ 0x1: 3 "u8[3]",
    check @ 0x4: 4 "StaffEntryCheck",
    check_arg @ 0x8: 2 "s16",
    pad_0A @ 0xA: 2 "u8[2]",
    alt_page @ 0xC: 4 "int",
    has_check @ 0x10: 1 "u8",
    x11 @ 0x11: 2 "u8[2]",
    pad_13 @ 0x13: 1 "u8",
]);

pub use crate::tu::gmstaffroll::gm_801AB200_HasCheck;

/// gmevent.c `gm_801BAB40_src`: one event match's player entry.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct gm_801BAB40_src {
    pub c_kind: i8,
    pub slot_type: u8,
    pub stocks: u8,
    pub color: u8,
    pub x5: u8,
    pub sub_color: u8,
    pub team: u8,
    pub xB: u8,
    pub flags: u8,
    pub xE: u8,
    pub cpu_level: u8,
    pub pad: u8,
    pub x12: u16,
    pub hp: u16,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
}

melee_core::gc_layout!(gm_801BAB40_src, "gm_801BAB40_src", size 0x1C, align 4, [
    c_kind @ 0x0: 1 "s8",
    slot_type @ 0x1: 1 "u8",
    stocks @ 0x2: 1 "u8",
    color @ 0x3: 1 "u8",
    x5 @ 0x4: 1 "u8",
    sub_color @ 0x5: 1 "u8",
    team @ 0x6: 1 "u8",
    xB @ 0x7: 1 "u8",
    flags @ 0x8: 1 "u8",
    xE @ 0x9: 1 "u8",
    cpu_level @ 0xA: 1 "u8",
    pad @ 0xB: 1 "u8",
    x12 @ 0xC: 2 "u16",
    hp @ 0xE: 2 "u16",
    x18 @ 0x10: 4 "f32",
    x1C @ 0x14: 4 "f32",
    x20 @ 0x18: 4 "f32",
]);

/// gmevent.c `gm_801BC00C_GetCharacterKind`: the s8 sign-extended to CharacterKind.
pub fn gm_801BC00C_GetCharacterKind(event_entry: &gm_801BAB40_src) -> i32 {
    event_entry.c_kind as i32
}
