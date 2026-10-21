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
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

// ---- ftCo_DamageFall.c -----------------------------------------------------------------------

/// `ftCo_80090574(gobj)`: `ft_80081DD4(gobj)`.
pub fn ftco_80090574(g: G) {
    h::ft_80081dd4(g)
}

/// static `calcShift(hitlag_mul, msid, dmg)`: model-shift frames. The C returns `u16` from a
/// float: gwtool saturates to u32 first, then truncates.
pub fn calc_shift(hitlag_mul: f32, msid: i32, dmg: i32) -> u16 {
    let fcp = h::common_data();
    fmadds(fcp.x168, calc_hitlag(dmg, msid, hitlag_mul), fcp.x16C) as u32 as u16
}

/// `ftCo_80090594(fp, element, dmg, msid, ground_or_air, hitlag_mul)`: arm the hit model shift.
pub fn ftco_80090594(fp: &mut Fighter, element: i32, dmg: i32, msid: i32, ground_or_air: i32, hitlag_mul: f32) {
    if element == HitElement_Cape as i32
        || element == HitElement_Disable as i32
        || element == HitElement_Nap as i32
        || element == HitElement_Sleep as i32
    {
        return;
    }
    if fp.motion_id as i32 == ftCo_MS_DamageIce as i32 {
        return;
    }
    fp.dmg.x18fa_model_shift_frames = calc_shift(hitlag_mul, msid, dmg);
    fp.dmg.x18FC = 0;
    if element == HitElement_Electric as i32 {
        fp.dmg.x18F8 = 2;
    } else if ground_or_air == GA_Air as i32 {
        fp.dmg.x18F8 = 0;
    } else {
        fp.dmg.x18F8 = 1;
        fp.dmg.x1900 = fp.coll_data.floor.normal.x;
        fp.dmg.x1904 = fp.coll_data.floor.normal.y;
    }
    // `(u8) (u32) Fighter_804D6530[x18F8 * 2 + 1]`: the odd entries hold a count, not a pointer.
    let e = unsafe { *h::fighter_804d6530().offset(fp.dmg.x18F8 as isize * 2 + 1) };
    fp.dmg.x18FD = e as usize as u32 as u8;
}

/// `ftCo_80090690(fp, &shift)`: the current model shift (C returns `shift`, or NULL).
pub fn ftco_80090690(fp: &mut Fighter) -> Option<Vec2> {
    if fp.dmg.x18fa_model_shift_frames == 0 {
        return None;
    }
    let k = fp.dmg.x18F8;
    let vec = unsafe { *(*h::fighter_804d6530().offset(k as isize * 2)).offset(fp.dmg.x18FC as isize) };
    if k == 1 {
        let x = vec.x * fp.facing_dir;
        Some(Vec2 { x: fp.dmg.x1904 * x, y: fmadds(-fp.dmg.x1900, x, vec.y) })
    } else {
        Some(Vec2 { x: vec.x * fp.facing_dir, y: vec.y })
    }
}

/// `ftCo_80090718(fp)`: play and clear the queued hit SFX / voice line.
pub fn ftco_80090718(fp: &mut Fighter) {
    if fp.dmg.x1908 != -1 {
        let id = fp.dmg.x1908;
        h::play_sfx(fp, id, 127, 64);
        fp.dmg.x1908 = -1;
    }
    if !fp.dmg.x190C.is_null() {
        let p = fp.dmg.x190C as *const FtSFXArr;
        h::ft_800889f4(fp, p);
        fp.dmg.x190C = core::ptr::null_mut();
    }
}

/// `ftCo_DamageFall_Anim`: empty.
pub fn ftco_damagefall_anim(_g: G) {}

/// `ftCo_DamageFall_Phys(gobj)`: `ft_80084DB0(gobj)`.
pub fn ftco_damagefall_phys(g: G) {
    h::ft_80084db0(g)
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

/// `ftParts_8007506C(kind, part)`: bit `i` of the first `Fighter_804D6540[kind]` entry whose
/// `x0 == part`, else 0.
pub fn ftparts_8007506c(ftkind: i32, part: i32) -> u32 {
    unsafe {
        let t = *h::fighter_804d6540().offset(ftkind as isize);
        if !t.is_null() && (*t).x4 != 0 {
            let mut v = (*t).x0;
            let mut i = 0i32;
            while i < (*t).x4 {
                if (*v).x0 as i32 == part {
                    return 1u32.wrapping_shl(i as u32);
                }
                i += 1;
                v = v.add(1);
            }
        }
    }
    0
}
