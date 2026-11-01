
//! Hand port, round 5: all of src/melee/ft/ft_0877.c (34 functions): fighter input / flag
//! queries, the CPU kind/level setters, the fighter-kind mask, sfx id adjustment (size
//! modifier, metal, Ice Climbers costume banks, m-ex relative ids) and `ft_PlaySFX`.
//! Signatures keep the C shape: `G` is the fighter gobj (`gobj->user_data` is the Fighter).
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0877.c`): only ft_80087B34, five
//! f32 fmuladd, each `x*x + y*y` -> `fmadds(x, x, y*y)`. Calls out of the ported set go through
//! `crate::hooks` (lbAudioAx: group 1; itIsHeavy / itGetKind: group 5; ftCo: group 3 states).
//! TARGET_PC paths ported: m-ex 5000..9999 sfx ids in 80087D0C, Geno voice owner in PlaySFX.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ft0877_ref.rs.

use crate::ftcommon2 as c2;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ft_GetFtKindMask`: one bit per fighter kind below Master Hand in the fighter list.
pub fn get_ft_kind_mask() -> i32 {
    let mut r: u32 = 0;
    let mut cur = h::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        let k = fp(cur).kind;
        if k < Ft_Kind_MasterH {
            r |= 1u32.wrapping_shl(k as u32);
        }
        cur = unsafe { (*cur).next };
    }
    r as i32
}

pub(crate) const SFX_METAL: [i32; 25] = [
    407, 467, 503, 515, 290051, 509, 485, 491, 479, 473, 437, 455, 461, 449, 443, 425, 431, 419,
    413, 377, 395, 401, 389, 383, 371,
];

pub(crate) const SFX_SIZE: [i32; 13] = [359, 368, 365, 362, 344, 353, 356, 350, 347, 338, 341, 335, 332];

/// `ft_PlaySFX(fp, sfx_id, vol, pan)`.
pub fn play_sfx(f: &mut Fighter, sfx_id: i32, vol: u8, pan: u8) {
    if f.kind >= Ft_Kind_Mex0 && h::geno_define_voice_play_owner(f.kind as i32, sfx_id, f as *mut Fighter) != 0 {
        f.x2160 = -1;
        return;
    }
    let sfx_id = ft_80087d0c(f, sfx_id);
    f.x2160 = h::lbaudioax_800237a8(sfx_id, vol as i32, pan as i32);
    let rand = if (332..=370).contains(&sfx_id) { h::hsd_randi(200) - 100 } else { 0 };
    h::lbaudioax_80024b94(f.x2160, rand);
}

pub use crate::tu::ft_0877::ft_800877f8;
pub use crate::tu::ft_0877::ft_80087818_fp as ft_80087818;
pub use crate::tu::ft_0877::ft_80087838_fp as ft_80087838;
pub use crate::tu::ft_0877::ft_80087858_fp as ft_80087858;
pub use crate::tu::ft_0877::ft_80087878_fp as ft_80087878;
pub use crate::tu::ft_0877::ft_800878bc;
pub use crate::tu::ft_0877::ft_80087900_fp as ft_80087900;
pub use crate::tu::ft_0877::ft_80087944_fp as ft_80087944;
pub use crate::tu::ft_0877::ft_80087988_fp as ft_80087988;
pub use crate::tu::ft_0877::ft_800879d8;
pub use crate::tu::ft_0877::ft_800879f8;
pub use crate::tu::ft_0877::ft_80087a18;
pub use crate::tu::ft_0877::ft_80087a80;
pub use crate::tu::ft_0877::ft_80087a8c;
pub use crate::tu::ft_0877::ft_80087a98;
pub use crate::tu::ft_0877::ft_80087aa4;
pub use crate::tu::ft_0877::ft_80087ab4;
pub use crate::tu::ft_0877::ft_80087ac0;
pub use crate::tu::ft_0877::ft_80087aec;
pub use crate::tu::ft_0877::ft_80087b34;
pub use crate::tu::ft_0877::ft_80087bac;
pub use crate::tu::ft_0877::ft_80087bc0;
pub use crate::tu::ft_0877::ft_80087bec;
pub use crate::tu::ft_0877::ft_80087c58;
pub use crate::tu::ft_0877::ft_80087c64;
pub use crate::tu::ft_0877::ft_80087c70;
pub use crate::tu::ft_0877::ft_80087d0c;
pub use crate::tu::ft_0877::ft_8008805c;
pub use crate::tu::ft_0877::ft_80088080;
pub use crate::tu::ft_0877::ft_800880ac;
pub use crate::tu::ft_0877::ft_800880d8;
pub use crate::tu::ft_0877::ft_80088110;
