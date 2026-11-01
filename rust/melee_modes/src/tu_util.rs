
//! Small helpers shared by the per-TU ports of `src/melee/gm`.
#![allow(non_snake_case, dead_code)]

use melee_types::inline::JObjDirty;

use melee_types::*;

/// `JObjDirty` for the inline JObj setters of melee_types::inline: the macro form calls the
/// extern `HSD_JObjSetMtxDirtySub` through its hook.
pub struct Dirty;

impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _jobj: &mut HSD_JObj) {
        unreachable!("the out-of-line (HSD_JObjSetMtxDirty) form is not used by the gm TUs")
    }
    fn set_mtx_dirty_sub(&mut self, jobj: &mut HSD_JObj) {
        crate::tu_ext::HSD_JObjSetMtxDirtySub(jobj)
    }
}

/// `HSD_JObjGetNext`: NULL-safe.
#[inline]
pub fn jobj_next(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { core::ptr::null_mut() } else { unsafe { (*j).next } }
}

/// A PPC (MSB-first) bitfield run held in one byte: `width` bits whose first (most significant)
/// bit is `msb` (0 = the top bit of the byte).
#[inline]
pub fn bits8_get(v: u8, msb: u32, width: u32) -> u8 {
    (v >> (8 - msb - width)) & (((1u32 << width) - 1) as u8)
}

#[inline]
pub fn bits8_set(v: &mut u8, msb: u32, width: u32, val: u8) {
    let shift = 8 - msb - width;
    let mask = (((1u32 << width) - 1) as u8) << shift;
    *v = (*v & !mask) | ((val << shift) & mask);
}

// dolphin/pad.h and baselib/controller.h button masks
pub const PAD_BUTTON_A: u32 = 1 << 8;

pub const PAD_BUTTON_START: u32 = 1 << 12;

pub const PAD_MAX_CONTROLLERS: i32 = 4;

pub const PAD_ANY_UP: u64 = 1 << 36;

pub const PAD_ANY_DOWN: u64 = 1 << 37;

pub const PAD_ANY_LEFT: u64 = 1 << 38;

pub const PAD_ANY_RIGHT: u64 = 1 << 39;

pub use crate::tu::itparasol::jobj_child_fp as jobj_child;
