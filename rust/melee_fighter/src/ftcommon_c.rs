
//! C-named entry points of src/melee/ft/ftcommon.c's movement helpers. The bodies live in
//! `crate::ftcommon` (checked bit for bit against the C by tests/ftcommon_ref.rs, which calls
//! these names); this module only gives them the C names and the C parameter shapes.
#![allow(non_snake_case)]

use crate::ftcommon as fc;

use crate::types::*;

pub(crate) type G = *mut HSD_GObj;

#[inline(always)]
pub(crate) unsafe fn fpg<'a>(g: G) -> &'a mut Fighter {
    &mut *((*g).user_data as *mut Fighter)
}

pub use crate::tu::ftcommon::ftCo_GetLStickAngle;
pub use crate::tu::ftcommon::ftCommon_8007DA24;
pub use crate::tu::ftcommon::ftCommon_ApplyGroundedKnockbackFriction_fp as ftCommon_ApplyGroundedKnockbackFriction;
pub use crate::tu::ftcommon::ftCommon_ApplyShieldKnockbackFriction_fp as ftCommon_ApplyShieldKnockbackFriction;
pub use crate::tu::ftcommon::ftCommon_Ascend;
pub use crate::tu::ftcommon::ftCommon_CalcGroundAccel_AccelToLStickX_fp as ftCommon_CalcGroundAccel_AccelToLStickX;
pub use crate::tu::ftcommon::ftCommon_CalcGroundAccel_AccelToVel;
pub use crate::tu::ftcommon::ftCommon_CalcGroundAccel_DashRun;
pub use crate::tu::ftcommon::ftCommon_CalcGroundAccel_Deaccel;
pub use crate::tu::ftcommon::ftCommon_CalcHitlag_fp as ftCommon_CalcHitlag;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_AccelToVel;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_AccelToVelClamped;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_AccelToVelClampedFrom;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_Deaccel_fp as ftCommon_CalcSelfAccel_Deaccel;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DeaccelAir_fp as ftCommon_CalcSelfAccel_DeaccelAir;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DeaccelQuick;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DeaccelQuickAir;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_Drift_fp as ftCommon_CalcSelfAccel_Drift;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DriftFrom;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DriftSimple_fp as ftCommon_CalcSelfAccel_DriftSimple;
pub use crate::tu::ftcommon::ftCommon_CalcSelfAccel_DriftSimple_NoFriction_fp as ftCommon_CalcSelfAccel_DriftSimple_NoFriction;
pub use crate::tu::ftcommon::ftCommon_CheckFallFast;
pub use crate::tu::ftcommon::ftCommon_ClampAirDrift;
pub use crate::tu::ftcommon::ftCommon_ClampFallSpeed;
pub use crate::tu::ftcommon::ftCommon_ClampGroundVel_fp as ftCommon_ClampGroundVel;
pub use crate::tu::ftcommon::ftCommon_ClampSelfVelX_fp as ftCommon_ClampSelfVelX;
pub use crate::tu::ftcommon::ftCommon_Fall_fp as ftCommon_Fall;
pub use crate::tu::ftcommon::ftCommon_FallBasic_fp as ftCommon_FallBasic;
pub use crate::tu::ftcommon::ftCommon_FallFast;
pub use crate::tu::ftcommon::ftCommon_SandbagKnockbackDeaccel_fp as ftCommon_SandbagKnockbackDeaccel;
pub use crate::tu::ftcommon::ftCommon_SetGroundedKnockbackIfLanded;
pub use crate::tu::ftcommon::ftCommon_SetSelfMovementFromGroundedMovement_fp as ftCommon_SetSelfMovementFromGroundedMovement;
pub use crate::tu::ftcommon::ftCommon_SetSelfMovementFromGroundedMovement_NoFriction;
pub use crate::tu::ftcommon::ftCommon_UpdateFacing_fp as ftCommon_UpdateFacing;
