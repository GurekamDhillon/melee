//! Step 9: FD collision on the engine (needs GW_ISO_VANILLA; vacuous without).
mod stage_common;
use stage_common::{boot, drop_box};

#[test]
fn final_destination_floor_stops_a_falling_box() {
    let Some(_e) = boot("fd") else { return };
    unsafe {
        // well above the flat top (y = 0): nothing touched
        let (y, line, env) = drop_box(0.0, 20.0, 18.0);
        assert_eq!((y, line, env & 0xFFFF), (18.0, -1, 0), "free fall");
        // crossing y = 0 at x = 0: stopped on the floor with the ECB bottom (0) on it
        let (y, line, env) = drop_box(0.0, 3.0, -2.0);
        assert!(line >= 0, "floor line found, env {env:#x}");
        assert!(y.abs() < 1e-3, "stopped on the floor, y = {y}");
        // far outside the stage: no floor
        let (y, line, _) = drop_box(400.0, 3.0, -2.0);
        assert_eq!((y, line), (-2.0, -1), "off stage");
    }
}

