
//! Round 3: `src/melee/lb/` leaf functions that take pointers to game structs, from the lb TUs
//! group 1 did not port. TODO(group1): move into melee_runtime when group 1 ports these TUs.
//! Conventions and verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
//!
//! FMA: `tools/rust-port/fma_sites.py` lists the shipping build's fused sites; every
//! `fmadds` below is one of them and every other `a*b+c` is left unfused.
#![allow(non_snake_case)]

use core::ffi::c_void;

use melee_core::fp::fmadds;

use melee_types as mt;

use mt::{CommandInfo, HSD_DObj, HSD_JObj, HitCapsule, Vec as Vec3};

pub use melee_runtime::tu::lbanim::{FigaTree, lbAnim_8001E8F8};

/// lbcollision.c `lbColl_80008428`.
pub fn lbColl_80008428(arg0: &mut HitCapsule) {
    // one port per C function: forwards to melee_runtime's TU port
    unsafe { melee_runtime::tu::lbcollision::lbColl_80008428(arg0) }
}

/// lbcollision.c `lbColl_80008434`.
pub fn lbColl_80008434(arg0: &mut HitCapsule) {
    // one port per C function: forwards to melee_runtime's TU port
    unsafe { melee_runtime::tu::lbcollision::lbColl_80008434(arg0) }
}

/// lbcollision.c `lbColl_80008440`: forget every victim.
pub fn lbColl_80008440(hit: &mut HitCapsule) {
    // one port per C function: forwards to melee_runtime's TU port
    unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(hit) }
}

// Compatibility exports: lbshadow belongs to runtime (inventory group 1).
pub use melee_runtime::tu::lbshadow::{splGetCardinalTangent, splGetBSplineTangent, splGetBezierTangent};

/// lbspdisplay.c and lb_00F9.c `checkJObjFlags` (identical static copies): neither a
/// particle nor a spline joint.
pub fn checkJObjFlags(jobj: &HSD_JObj) -> bool {
    jobj.flags & (mt::JOBJ_PTCL | mt::JOBJ_SPLINE) as u32 == 0
}

/// lb_00B0.c `lb_8000CE30`.
pub fn lb_8000CE30(dobj: Option<&mut HSD_DObj>, next: *mut HSD_DObj) {
    // one port per C function: forwards to melee_runtime's TU port
    unsafe { melee_runtime::tu::lb_00B0::lb_8000CE30(dobj.map_or(core::ptr::null_mut(), |d| d as *mut HSD_DObj), next) }
}

/// lb_00B0.c `lb_8000CE40`.
pub fn lb_8000CE40(jobj: Option<&mut HSD_JObj>, dobj: *mut HSD_DObj) {
    // one port per C function: forwards to melee_runtime's TU port
    unsafe { melee_runtime::tu::lb_00B0::lb_8000CE40(jobj.map_or(core::ptr::null_mut(), |j| j as *mut HSD_JObj), dobj) }
}

pub use crate::tu::lbcommand::Command_00_fp as Command_00;
pub use crate::tu::lbcommand::Command_06_fp as Command_06;
pub use crate::tu::lbcommand::Command_08_fp as Command_08;
pub use crate::tu::lbcollision::end_fp as end;
pub use crate::tu::lbcollision::lbColl_80005EBC_fp as lbColl_80005EBC;
pub use crate::tu::lbcollision::lbColl_80005FC0_fp as lbColl_80005FC0;
pub use crate::tu::lbcollision::lbColl_800089B8_fp as lbColl_800089B8;
pub use crate::tu::lbcollision::lbColl_80008A5C_fp as lbColl_80008A5C;
pub use crate::tu::lbcollision::lbColl_80008D30_fp as lbColl_80008D30;
pub use crate::tu::lbcollision::lbColl_DifferenceY;
pub use crate::tu::lbcollision::lbColl_GetY;
pub use crate::tu::lbsnap::lbSnap_ClearText_fp as lbSnap_ClearText;
pub use crate::tu::lbvector::lbVector_Mirror_fp as lbVector_Mirror;
pub use crate::tu::lb_00f9::lb_800103D8_fp as lb_800103D8;
pub use crate::tu::lb_013B::lb_80013BB0;
pub use crate::tu::lb_0146::lb_80014638;
pub use crate::tu::lbcardnew::readCardFileSize_fp as readCardFileSize;
pub use crate::tu::lbcollision::sqrDistance;
