//! Round 9 (night 3 round 5): lb functions verified bit for bit against the C by
//! `tests/oracle_r9.rs`. TODO(group1): lb leftovers; move into melee_runtime if group 1 takes
//! these TUs.
use crate::lb_ptr::checkJObjFlags;
use crate::lb_r8::lb_80011B74;
use core::ffi::c_void;
use melee_types::{it_8026C47C_arg0_t, HSD_JObj};

pub use melee_runtime::tu::lbspdisplay::lb_80011C18;

/// lb_0219.c `fn_80021C18`: an empty command callback.
pub fn fn_80021C18(_gobj: *mut c_void, _cmd: *mut c_void, _arg2: i32) {}

/// lbsnap.c `struct Unk80433380_0` (defined in the .c): a snapshot save header.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Unk80433380_0 {
    pub x0: i32,
    pub width: u16,
    pub height: u16,
    pub x8: u8,
    pub xC: i32,
    pub stkind: u16,
    pub x14: it_8026C47C_arg0_t,
    pub x34: i32,
    pub x38: [u8; 4],
}

melee_core::gc_layout!(Unk80433380_0, "struct Unk80433380_0", size 0x3C, align 4, [
    x0 @ 0x0: 4 "s32",
    width @ 0x4: 2 "u16",
    height @ 0x6: 2 "u16",
    x8 @ 0x8: 1 "u8",
    xC @ 0xC: 4 "s32",
    stkind @ 0x10: 2 "u16",
    x14 @ 0x14: 0x20 "struct it_8026C47C_arg0_t",
    x34 @ 0x34: 4 "s32",
    x38 @ 0x38: 4 "char[4]",
]);

/// lbsnap.c `lbSnap_GetSaveDataOffset`: `xC` plus the offset of `x38` (0x38 on both the GC and
/// native layouts: the record holds no pointers).
pub fn lbSnap_GetSaveDataOffset(snap: &Unk80433380_0) -> i32 {
    snap.xC.wrapping_add(core::mem::offset_of!(Unk80433380_0, x38) as i32)
}
