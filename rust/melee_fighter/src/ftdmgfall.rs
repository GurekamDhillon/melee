
//! Hand port, round 14: the damage model-shift leaves of src/melee/ft/kinds/ftCommon/
//! ftCo_DamageFall.c (ftCo_80090574, static calcShift, ftCo_80090594, ftCo_80090690,
//! ftCo_80090718, ftCo_DamageFall_Anim, ftCo_DamageFall_Phys) and the parts-table lookups of
//! ftparts.c (ftParts_GetBoneIndex, ftPartsRemap, ftParts_8007506C).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): calcShift `x168 * hitlag + x16C` and
//! ftCo_80090690 `-x1900 * x + vec.y`, both -> `fmadds`. Checked against the C (cut verbatim,
//! shipping contraction) by tests/ftdmgfall_ref.rs.

use crate::ftcommon::calc_hitlag;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

/// `FTPART_INVALID` (ft/types.h).
pub const FTPART_INVALID: i32 = 0xFF;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// static `calcShift(hitlag_mul, msid, dmg)`: model-shift frames. The C returns `u16` from a
/// float: gwtool saturates to u32 first, then truncates.
pub fn calc_shift(hitlag_mul: f32, msid: i32, dmg: i32) -> u16 {
    let fcp = h::common_data();
    melee_core::ppc::fctiwz_f32(fmadds(fcp.x168, calc_hitlag(dmg, msid, hitlag_mul), fcp.x16C)) as u16
}

// ---- ftparts.c -------------------------------------------------------------------------------

/// `ftParts_GetBoneIndex(fp, part)`: `ftPartsTable[kind]->part_to_joint[part]`.
pub fn ftparts_get_bone_index(fp: &Fighter, part: i32) -> i32 {
    unsafe {
        let t = *h::ft_parts_table().offset(fp.kind as isize);
        *(*t).part_to_joint.offset(part as isize) as i32
    }
}

/// `ftPartsRemap(to_kind, from_kind, joint)` (`size_t` is the 32-bit `gc_size_t`). The TARGET_PC
/// missing-table branch logs its first 4 hits to stderr (C: OSReport), then returns FTPART_INVALID.
pub fn ftparts_remap(to_table_idx: u32, from_table_idx: u32, joint_idx: u32) -> i32 {
    static mut LOGGED: i32 = 0;
    unsafe {
        let tab = h::ft_parts_table();
        let from = *tab.offset(from_table_idx as isize);
        let to = *tab.offset(to_table_idx as isize);
        if from.is_null() || to.is_null() {
            let n = LOGGED;
            LOGGED = n.wrapping_add(1);
            if n < 4 {
                eprintln!(
                    "ftPartsRemap: no parts table (to kind {} {}, from kind {} {}, joint {})",
                    to_table_idx as i32,
                    if to.is_null() { "NULL" } else { "ok" },
                    from_table_idx as i32,
                    if from.is_null() { "NULL" } else { "ok" },
                    joint_idx as i32
                );
            }
            return FTPART_INVALID;
        }
        if joint_idx < (*from).parts_num {
            let part_idx = *(*from).joint_to_part.offset(joint_idx as isize) as u32;
            if part_idx != FTPART_INVALID as u32 {
                return *(*to).part_to_joint.offset(part_idx as isize) as i32;
            }
        }
    }
    FTPART_INVALID
}

pub use crate::tu::ftCo_DamageFall::ftco_80090574;
pub use crate::tu::ftCo_DamageFall::ftco_80090594;
pub use crate::tu::ftCo_DamageFall::ftco_80090690;
pub use crate::tu::ftCo_DamageFall::ftco_80090718;
pub use crate::tu::ftparts::ftparts_8007506c;
