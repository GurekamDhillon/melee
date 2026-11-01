
//! Hand port, round 13: all of src/melee/ft/ft_0DF0.c (smash-attack charge state machine:
//! `smash_attrs`) and all of ft_0DF1.c (C-stick flick / hold tests against PlCo thresholds),
//! plus field leaves of ftcoll.c (`skip_update_pos` resets, ftColl_8007B8A8/B8CC), ftdevice.c
//! (`ftCo_800C0658` colour-overlay select) and ftparts.c (`x5F4_arr` model-part visibility).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): ftCo_800DEEB8 `(mul - 1) * (frames / hold) + 1`
//! and ftCo_800DF608 `SQ(x) + SQ(y)` (left product fused, the right one rounded first); both ->
//! `fmadds`. Checked against the C (cut verbatim, shipping contraction) by tests/ft0df_ref.rs.

use crate::ftcommon::abs_c;

use crate::ftcommon2 as fc2;

use crate::ftcommon3::ftco_get_cstick_angle;

use crate::hooks as h;

use crate::types::*;

use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

pub(crate) const HSD_PAD_A: u32 = 0x100;

pub(crate) const SMASH_STATE_4: i32 = 4;

#[inline(always)]
pub(crate) fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[inline(always)]
pub(crate) fn cd() -> &'static ftCommonData {
    unsafe { &*h::P_FT_COMMON_DATA }
}

pub(crate) fn tick(fp: &mut Fighter) {
    let a = &mut fp.smash_attrs;
    a.x2118_frames += 1.0;
    a.x212C = a.x212C.wrapping_add(1);
    if a.x212C >= a.x212D {
        a.x212C = 0;
    }
}

pub(crate) fn charge_sfx(fp: &mut Fighter) {
    if fp.smash_attrs.x2130_sfxBool == 0 && fp.smash_attrs.x2118_frames >= cd().x7C8 {
        h::play_sfx(fp, 0x7B, 0x7F, 0x40);
        fp.smash_attrs.x2130_sfxBool = 1;
    }
}

pub(crate) fn release(g: G) {
    let f = fp(g);
    f.smash_attrs.state = SmashState_Release as _;
    let rate = f.smash_attrs.x2124_frameSpeedMul;
    h::ftanim_set_anim_rate(g, rate);
    let k = fp(g).smash_attrs.x2128;
    h::ftco_800c0200(fp(g), k);
    fc2::ftcommon_8007ecd4(fp(g), 0x24);
}

// ---- ft_0DF1.c ---------------------------------------------------------------------------

/// `canUseCstick` (static inline): single-button mode off, or the held item allows the C-stick.
pub fn can_use_cstick(fp: &Fighter) -> bool {
    !h::gm_8016b0fc() || h::it_8026b30c(fp.item_gobj) == 0
}

pub(crate) fn cs(fp: &Fighter, i: usize) -> Vec2 {
    let v = &fp.input.cstick[i];
    Vec2 { x: v.x, y: v.y }
}

pub use crate::tu::ftdevice::ftco_800c0658;
pub use crate::tu::ftdevice::ftco_800c0674;
pub use crate::tu::ftdevice::ftco_800c0694;
pub use crate::tu::ft_0df0::ftco_800dee84;
pub use crate::tu::ft_0df0::ftco_800deea8;
pub use crate::tu::ft_0df0::ftco_800deeb8;
pub use crate::tu::ft_0df0::ftco_800deee8;
pub use crate::tu::ft_0df0::ftco_800def38;
pub use crate::tu::ft_0df0::ftco_800df0d0;
pub use crate::tu::ft_0df1::ftco_800df1c8;
pub use crate::tu::ft_0df1::ftco_800df21c;
pub use crate::tu::ft_0df1::ftco_800df2d8;
pub use crate::tu::ft_0df1::ftco_800df30c;
pub use crate::tu::ft_0df1::ftco_800df3a8;
pub use crate::tu::ft_0df1::ftco_800df3dc;
pub use crate::tu::ft_0df1::ftco_800df478;
pub use crate::tu::ft_0df1::ftco_800df50c;
pub use crate::tu::ft_0df1::ftco_800df608;
pub use crate::tu::ft_0df1::ftco_800df644;
pub use crate::tu::ft_0df1::ftco_800df678;
pub use crate::tu::ft_0df1::ftco_800df6f8;
pub use crate::tu::ft_0df1::ftco_800df72c;
pub use crate::tu::ft_0df1::ftco_800df79c;
pub use crate::tu::ft_0df1::ftco_800df7f4;
pub use crate::tu::ft_0df1::ftco_800df844;
pub use crate::tu::ft_0df1::ftco_800df878;
pub use crate::tu::ft_0df1::ftco_800df8b0;
pub use crate::tu::ft_0df1::ftco_800df8e8;
pub use crate::tu::ft_0df1::ftco_800df910;
pub use crate::tu::ftcoll::ftcoll_8007aee0;
pub use crate::tu::ftcoll::ftcoll_8007aef8;
pub use crate::tu::ftcoll::ftcoll_8007af10;
pub use crate::tu::ftcoll::ftcoll_8007af28;
pub use crate::tu::ftcoll::ftcoll_8007b8a8;
pub use crate::tu::ftcoll::ftcoll_8007b8cc;
pub use crate::tu::ftparts::ftparts_80074a4c;
pub use crate::tu::ftparts::ftparts_80074a74;
pub use crate::tu::ftparts::ftparts_80074a8c;
pub use crate::tu::ftparts::ftparts_80074acc;
