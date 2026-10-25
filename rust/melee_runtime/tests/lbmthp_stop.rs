//! `lbMthp_8001F87C` (lbmthp.c: `MoviePlayer.power = 0`, the player is marked stopped without cleanup); checked against the C
//! source, no oracle build of the TU.
use melee_runtime::tu::lbmthp::{lbMthp_8001F87C, test_power, test_set_power};

#[test]
fn stop_without_cleanup_clears_power() {
    test_set_power(1);
    assert_eq!(test_power(), 1);
    lbMthp_8001F87C();
    assert_eq!(test_power(), 0);
    lbMthp_8001F87C();
    assert_eq!(test_power(), 0);
}
