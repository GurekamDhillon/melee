
//! Hand port, round 9: leaf functions of ft/ftanim.c (x8B0 part-anim slots, part joint pick,
//! the inverse-scale helper). Checked against the C (cut verbatim) by tests/ftleaf_ref.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

pub use crate::tu::ftanim::ftanim_8007077c;
pub use crate::tu::ftanim::ftanim_80070fb4;
pub use crate::tu::ftanim::get_part_joint;
pub use crate::tu::ftanim::scale_inline;
