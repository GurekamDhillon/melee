//! src/melee/ft/kinds/ftCommon/ftCo_09C4.c: Donkey Kong's cargo hold (ShoulderedWait) entry.
//! No float math (no fused sites). Smoke-tested by tests/tu_ftco_09c4.rs.
#![allow(non_snake_case, static_mut_refs)]

use crate::hooks as h;
use crate::types::*;

pub type G = *mut Fighter_GObj;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use crate::types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(FTCO_80099F1C, ftCo_80099F1C, (g: *mut Fighter_GObj) -> bool);
    hook!(FTCO_8009A184, ftCo_8009A184, (g: *mut Fighter_GObj, msid: i32, flags: MotionFlags, anim_start: f32));
    hook!(FTCO_8009C5A4, ftCo_8009C5A4, (g: *mut Fighter_GObj, msid: i32));
}

/// `ftCo_8009C4F8`.
pub fn ftCo_8009C4F8(gobj: G) -> bool {
    if ext::ftCo_80099F1C(gobj) {
        ftCo_8009C540(gobj);
        return true;
    }
    false
}

/// `ftCo_8009C540` (static).
pub fn ftCo_8009C540(gobj: G) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ext::ftCo_8009A184(gobj, (*(*fp).x2CC).x4_motion_state + 6, Ft_MF_None as MotionFlags, 0.0);
        h::ftanim_set_anim_rate(gobj, 0.0);
        ext::ftCo_8009C5A4((*fp).victim_gobj, ftCo_MS_ShoulderedWait as i32);
    }
}
