//! Giga Bowser (src/melee/ft/kinds/ftGigaKoopa).

pub mod ftgkoopa {
    /// `ftGk_Init_UnkDemoCallbacks0(arg0, arg1, arg2)`: demo motion file indices.
    pub fn ftGk_Init_UnkDemoCallbacks0(arg0: i32, arg1: &mut i32, arg2: &mut i32) {
        if arg0 == 0xF {
            *arg2 = 0xE;
            *arg1 = 0xE;
        }
    }
}
