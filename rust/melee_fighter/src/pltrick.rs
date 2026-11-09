
//! Hand port, round 7: the attack-statistics half of src/melee/pl/pltrick.c (7 of its 10
//! definitions): pl_80037B2C, 80037BC0, 80037C60, 80037DF4, 80037ECC, 800384DC, 80038628 plus the
//! table `pl_803BCE70`. Not ported: fn_80037F00 / pl_80038144 (hit bookkeeping; need
//! ifMagnify, gm and the rest of plbonuslib).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/pl/pltrick.c`): none in the ported set.
//! `union Struct2070` int arguments are taken as the `Struct2070` they pun (see `ft0892`).
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0892_ref.rs.

use crate::ft0892::G;

use crate::hooks as h;

use crate::stale::action_stats;

use crate::types::*;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

pub static PL_803BCE70: [i32; 16] = [-1, 0x2F, 0x30, 0x31, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];

/// `acp->by_attack_hi[id]++` for id >= 100. The C indexes past the 65-entry array: the u32 is
/// at plActionStats +0x3E8 + id*4. id 100..=115 is a u32 field (x578..x594, x598[0..8]) and is
/// incremented as one. 116/117 land on `x5B8[4]` / the `x5BC` bit byte: the game does a
/// big-endian u32 increment over those bytes, done here byte-wise (the native C differs).
/// id >= 118 runs past plActionStats into the rest of the GameCube StaticPlayer, whose native
/// layout differs: not ported (panics).
pub(crate) fn by_attack_hi_inc(acp: &mut plActionStats, id: u8) {
    match id {
        100..=115 => {
            let hi = unsafe { acp.x358.x0.by_attack_hi.as_mut_ptr().add(id as usize) };
            unsafe { *hi = (*hi).wrapping_add(1) };
        }
        116 | 117 => {
            let base = acp as *mut plActionStats as *mut u8;
            let p = unsafe { base.add(0x3E8 + id as usize * 4) };
            let mut b = [0u8; 4];
            unsafe { core::ptr::copy_nonoverlapping(p, b.as_mut_ptr(), 4) };
            b = u32::from_be_bytes(b).wrapping_add(1).to_be_bytes();
            unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), p, 4) };
        }
        _ => panic!("TODO(group3 pltrick) pl_80037C60: by_attack_hi[{id}] is past plActionStats (GC StaticPlayer layout)"),
    }
}

pub use crate::tu::pltrick::pl_80037b2c;
pub use crate::tu::pltrick::pl_80037bc0;
pub use crate::tu::pltrick::pl_80037c60;
pub use crate::tu::pltrick::pl_80037df4;
pub use crate::tu::pltrick::pl_80037ecc;
pub use crate::tu::pltrick::pl_800384dc;
pub use crate::tu::pltrick::pl_80038628_fp as pl_80038628;
