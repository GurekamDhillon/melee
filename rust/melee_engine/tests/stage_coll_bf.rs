//! Step 9: BF platforms on the engine (needs GW_ISO_VANILLA; vacuous without).
mod stage_common;
use stage_common::{boot, drop_box};

#[test]
fn battlefield_platforms_catch_a_falling_box() {
    let Some(_e) = boot("bf") else { return };
    unsafe {
        // main stage top
        let (y, line, _) = drop_box(0.0, 3.0, -2.0);
        assert!(line >= 0 && y.abs() < 1e-3, "main floor, y = {y}");
        // side platform (about y = 27) at x = -40, from above
        let (y, line, _) = drop_box(-40.0, 33.0, 22.0);
        assert!(line >= 0 && y > 20.0 && y < 32.0, "side platform, y = {y}");
        // top platform (about y = 54) at x = 0, from above
        let (y, line, _) = drop_box(0.0, 60.0, 48.0);
        assert!(line >= 0 && y > 48.0 && y < 60.0, "top platform, y = {y}");
        // between the side platforms and the top one nothing is hit (x = 0, y 40 -> 38)
        let (y, line, _) = drop_box(0.0, 40.0, 38.0);
        assert_eq!((y, line), (38.0, -1), "free fall between platforms");
    }
}
