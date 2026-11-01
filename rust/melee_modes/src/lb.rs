
//! `src/melee/lb/` leaf functions group 1 did not port (lbtime, lb_00CE, lbsnap, lbaudio_ax,
//! lbcardnew, lbcollision, lbdvd, lb_00F9 helpers). Hand ports, same names as the C, checked
//! by `tests/oracle.rs`. TODO(group1): move into melee_runtime when group 1 ports these TUs.
#![allow(non_snake_case)]

use melee_core::vec::Vec3;

use melee_runtime::cabi;

pub use crate::tu::lbsnap::RGB565_TO_RGB5A3_fp as RGB565_TO_RGB5A3;
pub use crate::tu::lb_00f9::absf_fp as absf;
pub use crate::tu::lb_00f9::approximatelyZeroVec3_fp as approximatelyZeroVec3;
pub use crate::tu::lbaudio_ax::calcPan_fp as calcPan;
pub use crate::tu::lbcardnew::convertHsdResult_fp as convertHsdResult;
pub use crate::tu::lbcardnew::convertSdkResult_fp as convertSdkResult;
pub use crate::tu::lb_00ce::expf;
pub use crate::tu::lbcollision::isSmall_fp as isSmall;
pub use crate::tu::lbsnap::lbSnap_8001E204_fp as lbSnap_8001E204;
pub use crate::tu::lbsnap::lbSnap_8001E210_fp as lbSnap_8001E210;
pub use crate::tu::lbtime::lbTime_8000AEC8_fp as lbTime_8000AEC8;
pub use crate::tu::lbtime::lbTime_8000AEE4_fp as lbTime_8000AEE4;
pub use crate::tu::lbtime::lbTime_8000AF24_fp as lbTime_8000AF24;
pub use crate::tu::lbtime::lbTime_8000AF74_fp as lbTime_8000AF74;
pub use crate::tu::lb_00ce::lb_8000D008_fp as lb_8000D008;
pub use crate::tu::lb_00ce::powf_fp as powf;
pub use crate::tu::lb_00ce::powi_fp as powi;
pub use crate::tu::lbdvd::same;
