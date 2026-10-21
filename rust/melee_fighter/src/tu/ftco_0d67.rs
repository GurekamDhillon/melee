//! src/melee/ft/kinds/ftCommon/ftCo_0D67.c: the neutral-B input test.
//! No fused sites (tools/rust-port/fma_sites.py). Checked against the C by tests/tu_ftco_0d67.rs.

use crate::ftcommon::abs_c;
use crate::hooks::common_data;
use crate::types::*;

/// `ftCo_800D67C4`: B pressed with the stick inside the x218 / x21C dead zone.
pub fn ftco_800d67c4(fp: &Fighter) -> bool {
    if fp.input.pressed_buttons & HSD_PAD_B as HSD_Pad != 0 {
        let cd = common_data();
        if abs_c(fp.input.lstick[0].x) < cd.x218 && abs_c(fp.input.lstick[0].y) < cd.x21C {
            return true;
        }
    }
    false
}
