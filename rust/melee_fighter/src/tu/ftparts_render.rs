
//! src/melee/ft/ftparts.c, the model-class half (re-exported from `tu::ftparts`): the fighter
//! JObj / IntpJObj / PObj classes (`hsdChangeClass`ed onto every fighter model so a fighter's Z
//! scale reaches the matrices), their matrix setup, `ftParts_8007482C` (an interpolation-JObj
//! load), `ftParts_80075304` (graft a JObj into a bone tree) and the part setup / removal
//! (`ftParts_8007487C`, `ftParts_800753D4`, `ftParts_800755E8`, `ftParts_80075650`).
//!
//! The GX matrix loads are the hooks of `melee_runtime::tu::pobj::ext` (unset they panic with the
//! C name). No fused sites (fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;

use core::ffi::{c_char, c_void};

use core::mem::size_of;

use core::ptr::null_mut;

/// The pobj hooks plus the typed matrix wrappers and inlines melee_runtime now provides natively.
pub(crate) mod gx {
    pub use melee_runtime::tu::pobj::ext::*;
    pub use melee_runtime::tu::pobj::mx::{GXLoadNrmMtxImm, GXLoadPosMtxImm, GXLoadTexMtxImm, _HSD_mkEnvelopeModelNodeMtx};
    pub use melee_runtime::tu::pobj::HSD_PerfCountMtxLoad;
    pub use melee_runtime::tu::pobj::jobj_setup_matrix as HSD_JObjSetupMatrix;
}

pub(crate) type MP = *mut [f32; 4];

pub(crate) type M34 = [[f32; 4]; 3];

/// `ft_jobj_scale` (ftparts.static.h): the inverse-scale matrix for a fighter with a Z scale.
pub struct FtJObjScale {
    pub mtx: M34,
    pub has_z_scale: bool,
}

pub static mut ft_jobj_scale: FtJObjScale = FtJObjScale { mtx: [[0.0; 4]; 3], has_z_scale: false };

pub static mut ftJObj: HSD_JObjInfo = {
    let mut i: HSD_JObjInfo = unsafe { core::mem::zeroed() };
    i.parent.parent.head.info_init = Some(ftParts_JObjInfoInit);
    i
};

pub static mut ftIntpJObj: HSD_JObjInfo = {
    let mut i: HSD_JObjInfo = unsafe { core::mem::zeroed() };
    i.parent.parent.head.info_init = Some(ftParts_IntpJObjInfoInit);
    i
};

pub static mut ftPObj: HSD_PObjInfo = {
    let mut i: HSD_PObjInfo = unsafe { core::mem::zeroed() };
    i.parent.head.info_init = Some(ftParts_PObjInfoInit);
    i
};

pub(crate) unsafe fn mtx_at(p: MP) -> M34 {
    p.cast::<M34>().read()
}

/// `ftParts_80073758(jobj)`: make `jobj` a fighter JObj.
pub unsafe fn ftParts_80073758(jobj: *mut HSD_JObj) {
    melee_core::ct!("ftParts_80073758");
    melee_runtime::tu::class::hsdChangeClass(jobj.cast(), (&raw mut ftJObj).cast());
}

pub(crate) const SETUP_NORMAL: u32 = 1;

pub(crate) const SETUP_REFLECTION: u32 = 2;

pub(crate) const SETUP_HIGHLIGHT: u32 = 4;

pub(crate) const SETUP_NORMAL_PROJECTION: u32 = 6;

pub(crate) const SETUP_JOINT0: u32 = 1;

pub(crate) const SETUP_JOINT1: u32 = 2;

pub(crate) const RENDER_SHADOW: u32 = 0x400_0000;

pub(crate) const HSD_MTX_RIGID: u32 = 1;

pub(crate) const HSD_MTX_ENVELOPE: u32 = 2;

pub(crate) const TEX_COORD_REFLECTION: u32 = 1;

pub(crate) const TEX_COORD_HILIGHT: u32 = 2;

pub(crate) const GX_TEXMTX0: u32 = 30;

pub(crate) const GX_TEXMTX1: u32 = 33;

pub unsafe fn ftPartsPObjSetDefaultClass() {
    melee_core::ct!("ftPartsPObjSetDefaultClass");
    melee_runtime::tu::pobj::HSD_PObjSetDefaultClass(&raw mut ftPObj);
}

pub unsafe fn ftPartsPObjClearDefaultClass() {
    melee_core::ct!("ftPartsPObjClearDefaultClass");
    melee_runtime::tu::pobj::HSD_PObjSetDefaultClass(null_mut());
}

/// Calls out of `ftParts_8007482C`: settable hooks (the class oracle's recording stand-ins); unset they run the ported callees.
pub mod ext {
    use super::*;
    pub static mut HOOK_SET_DEFAULT_CLASS: Option<unsafe fn(*mut c_void)> = None;
    pub static mut HOOK_LOAD_JOINT: Option<unsafe fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
    pub unsafe fn HSD_JObjSetDefaultClass(info: *mut c_void) {
        match HOOK_SET_DEFAULT_CLASS {
            Some(f) => f(info),
            None => melee_runtime::tu::jobj::HSD_JObjSetDefaultClass(info.cast()),
        }
    }
    pub unsafe fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
        match HOOK_LOAD_JOINT {
            Some(f) => f(joint),
            None => melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint),
        }
    }
}

pub use crate::tu::ftparts::ftPartsGetSetupFlags;
pub use crate::tu::ftparts::ftPartsSetupEnvelopeMtx;
pub use crate::tu::ftparts::ftPartsSetupNrmMtx;
pub use crate::tu::ftparts::ftPartsSetupRigidMtx;
pub use crate::tu::ftparts::ftPartsSetupSharedVtxMtx;
pub use crate::tu::ftparts::ftPartsSetupTexMtx;
pub use crate::tu::ftparts::ftPartsSetupZScaleMtx;
pub use crate::tu::ftparts::ftParts_80074194;
pub use crate::tu::ftparts::ftParts_8007482C;
pub use crate::tu::ftparts::ftParts_8007487C;
pub use crate::tu::ftparts::ftParts_80075304;
pub use crate::tu::ftparts::ftParts_800753D4;
pub use crate::tu::ftparts::ftParts_800755E8;
pub use crate::tu::ftparts::ftParts_80075650;
pub use crate::tu::ftparts::ftParts_IntpJObjInfoInit;
pub use crate::tu::ftparts::ftParts_IntpJObjLoad;
pub use crate::tu::ftparts::ftParts_JObjInfoInit;
pub use crate::tu::ftparts::ftParts_JObjMakePositionMtx;
pub use crate::tu::ftparts::ftParts_PObjInfoInit;
pub use crate::tu::ftparts::ftParts_PObjSetupMtx;
