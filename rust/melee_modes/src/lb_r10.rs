
//! Round 10 (night 3 round 6): the lbrefract texel callbacks and lbaudio_ax `fn_80025E38`,
//! verified bit for bit against the C by `tests/oracle_r10.rs`.
use melee_types::{lbRefract_CallbackData, HSD_GObj};

use crate::lb_r4::lbAudioAx_UserData;

/// `(u8*) data->buffer + c * data->row_stride + r * tile` in the C's u32 arithmetic.
pub(crate) unsafe fn tile_base(data: &lbRefract_CallbackData, c: u32, r_off: u32) -> *mut u8 {
    (data.buffer as *mut u8).add(c.wrapping_mul(data.row_stride).wrapping_add(r_off) as usize)
}

pub use crate::tu::lbrefract::fn_80021F70_fp as fn_80021F70;
pub use crate::tu::lbrefract::fn_80021FB4_fp as fn_80021FB4;
pub use crate::tu::lbrefract::fn_80021FF8_fp as fn_80021FF8;
pub use crate::tu::lbrefract::fn_8002206C_fp as fn_8002206C;
pub use crate::tu::lbaudio_ax::fn_80025E38_fp as fn_80025E38;
