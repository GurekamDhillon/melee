//! grcastle.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::clamp_tau;
#[allow(unused_imports)]
use crate::helpers::it_8027b798_calc_angle;
#[allow(unused_imports)]
use crate::helpers::COLLIDE_CEILING_MASK;
#[allow(unused_imports)]
use crate::helpers::M_PI;
#[allow(unused_imports)]
use crate::helpers::M_PI_2;
#[allow(unused_imports)]
use crate::helpers::M_TAU;
#[allow(unused_imports)]
use crate::item::norm_xf74;
#[allow(unused_imports)]
use crate::itlib::it_80276408;
#[allow(unused_imports)]
use crate::itlib::COLLIDE_FLOOR_MASK;
#[allow(unused_imports)]
use crate::layout::ftCommonData;
#[allow(unused_imports)]
use crate::layout::itBoxAttributes;
#[allow(unused_imports)]
use crate::layout::itHitodemanAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itPikachutJoltGroundAttributes;
#[allow(unused_imports)]
use crate::layout::itTincleAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use crate::layout::It_Kind_Flipper;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_core::vec::Vec3 as RVec;
#[allow(unused_imports)]
use melee_runtime::gekko::cosf;
#[allow(unused_imports)]
use melee_runtime::gekko::sinf;
#[allow(unused_imports)]
use melee_runtime::lbtrigf::atan2f;
#[allow(unused_imports)]
use melee_runtime::lbvector;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;

// ---- folded from itkinds10.rs (consolidation steps 21/22) ----
#[inline(always)]
pub fn zero_fp() -> GcVec {
    melee_core::ct!("zero_fp");
    GcVec { x: 0.0, y: 0.0, z: 0.0 }
}
