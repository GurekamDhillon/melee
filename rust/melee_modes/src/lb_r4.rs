
//! Round 4: lb callers of rounds 1-3 and of group 1's ported code (lbtrigf `acosf`, MSL
//! `sqrtf`). Verified bit for bit against the C (`tests/oracle_r4.rs`).
//! TODO(group1): these lb TUs are lb leftovers; move into melee_runtime if group 1 takes them.
//!
//! FMA (`tools/rust-port/fma_sites.py`): `lb_8000D148` has the shipping build's 5 fused sites,
//! each an `fmadds` below; nothing else here fuses.
#![allow(non_snake_case)]

use crate::lb::{absf, calcPan};

use melee_core::fp::fmadds;

use melee_runtime::gekko::sqrtf;

use melee_types as mt;

use mt::{CameraBlurData, DynamicsData, HSD_GObj, HSD_GObjEvent, HSD_JObj, Vec as Vec3};

/// `lbAudioAx_UserData` (lb/lbaudio_ax.static.h, 0x48 bytes on the GC). melee_types does not
/// scan `.static.h` headers. TODO(group0): generate it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct lbAudioAx_UserData {
    pub x0: i32,
    pub gobj: *mut HSD_GObj,
    pub owner: *mut HSD_GObj,
    pub xC: i32,
    pub x10: Option<unsafe extern "C" fn(*mut HSD_GObj) -> i32>,
    pub id: i32,
    pub start_vol: i32,
    pub end_vol: i32,
    pub vol: i32,
    pub pan_left: i32,
    pub pan_right: i32,
    pub pan: i32,
    pub voice_id: i32,
    pub current_frame: i32,
    pub end_frame: i32,
    pub x3C: f32,
    pub track: i32,
    /// MSL `bool` (int)
    pub x44: i32,
}

melee_core::gc_layout!(lbAudioAx_UserData, "lbAudioAx_UserData", size 0x48, align 4, [
    x0 @ 0x0: 4 "int",
    gobj @ 0x4: 4 "HSD_GObj*",
    owner @ 0x8: 4 "HSD_GObj*",
    xC @ 0xC: 4 "int",
    x10 @ 0x10: 4 "bool (*)(HSD_GObj*)",
    id @ 0x14: 4 "int",
    start_vol @ 0x18: 4 "int",
    end_vol @ 0x1C: 4 "int",
    vol @ 0x20: 4 "int",
    pan_left @ 0x24: 4 "int",
    pan_right @ 0x28: 4 "int",
    pan @ 0x2C: 4 "int",
    voice_id @ 0x30: 4 "int",
    current_frame @ 0x34: 4 "int",
    end_frame @ 0x38: 4 "int",
    x3C @ 0x3C: 4 "float",
    track @ 0x40: 4 "int",
    x44 @ 0x44: 4 "bool",
]);

/// `GET_SOUND(gobj)`: the gobj's user data as the sound object (may be NULL).
pub(crate) fn get_sound<'a>(gobj: &HSD_GObj) -> Option<&'a mut lbAudioAx_UserData> {
    // SAFETY: a sound gobj's user_data is its lbAudioAx_UserData (or NULL).
    unsafe { (mt::inline::HSD_GObjGetUserData(gobj) as *mut lbAudioAx_UserData).as_mut() }
}

pub(crate) fn pan_of(ud: &lbAudioAx_UserData) -> i32 {
    calcPan(ud.current_frame, ud.end_frame, ud.pan_left, ud.pan_right)
}

// Compatibility export: the implementation lives with its TU in runtime.
pub use melee_runtime::tu::lbshadow::lbShadow_8000E9F0;

pub use crate::tu::lb_020A::calc_acos_fp as calc_acos;
pub use crate::tu::lbaudio_ax::fn_800253D8_fp as fn_800253D8;
pub use crate::tu::lbaudio_ax::fn_800256BC_fp as fn_800256BC;
pub use crate::tu::lbaudio_ax::fn_800259EC_fp as fn_800259EC;
pub use crate::tu::lbaudio_ax::fn_80025A98_fp as fn_80025A98;
pub use crate::tu::lbaudio_ax::fn_80025B44_fp as fn_80025B44;
pub use crate::tu::lbaudio_ax::fn_80025CBC_fp as fn_80025CBC;
pub use crate::tu::lb_00f9::groundHeight_fp as groundHeight;
pub use crate::tu::lbaudio_ax::lbAudioAx_800264E4_fp as lbAudioAx_800264E4;
pub use crate::tu::lb_00ce::lb_8000D148_fp as lb_8000D148;
pub use crate::tu::lbspdisplay::lb_8001204C_fp as lb_8001204C;
pub use crate::tu::lbspdisplay::lb_800138CC_fp as lb_800138CC;
pub use crate::tu::lbspdisplay::lb_800138D8_fp as lb_800138D8;
