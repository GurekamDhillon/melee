//! Round 9 (night 3 round 5): lb functions verified bit for bit against the C by
//! `tests/oracle_r9.rs`. TODO(group1): lb leftovers; move into melee_runtime if group 1 takes
//! these TUs.
use crate::lb_ptr::checkJObjFlags;
use crate::lb_r8::lb_80011B74;
use core::ffi::c_void;
use melee_types::{it_8026C47C_arg0_t, HSD_JObj};

/// lbspdisplay.c `lb_80011C18`: OR `flags` into the render mode of every MObj under `jobj`
/// (its child and next subtrees, the child/next joints' DObj chains, the joint's own DObjs).
///
/// # Safety
/// `jobj` is a live JObj tree; every joint `checkJObjFlags` accepts holds a NULL or live DObj
/// chain whose DObjs all have a live MObj (as in the C, which does not check `mobj`).
pub unsafe fn lb_80011C18(jobj: &mut HSD_JObj, flags: u32) {
    for cur in [jobj.child, jobj.next] {
        if let Some(cur) = cur.as_mut() {
            if let Some(c) = cur.child.as_mut() {
                lb_80011C18(c, flags);
            }
            if let Some(n) = cur.next.as_mut() {
                lb_80011C18(n, flags);
            }
            if checkJObjFlags(cur) {
                if let Some(d) = cur.u.dobj.as_mut() {
                    lb_80011B74(d, flags);
                }
            }
        }
    }
    if checkJObjFlags(jobj) {
        if let Some(d) = jobj.u.dobj.as_mut() {
            if let Some(n) = d.next.as_mut() {
                lb_80011B74(n, flags);
            }
            (*d.mobj).rendermode |= flags;
        }
    }
}

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
