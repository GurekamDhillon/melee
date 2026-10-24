//! src/melee/ft/kinds/ftDonkey/ftdonkeyheavylanding.c: DK carrying a heavy item, landing/damage.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftdonkeyheavylanding.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r2::ftDonkey::ftdonkeyheavylanding::ftDk_HeavyLanding_Phys;
pub use crate::chars::r4::ftDonkey::ftdonkeyheavylanding::ftDk_HeavyWait2_Coll;
pub use crate::chars::r5::ftDonkey::ftdonkeyheavylanding::ftDk_HeavyLanding_Coll;

/// Calls into TUs that are not ported yet (settable hooks; unset they panic).
pub mod ext {
    use melee_types::*;
    type G = *mut HSD_GObj;
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
    hook!(ITEM_8026ABD8, Item_8026ABD8, (gobj: *mut Item_GObj, pos: *mut Vec3, f: f32));
    hook!(FTCO_8008E908, ftCo_8008E908, (gobj: G, f: f32));
    hook!(FTCO_8008E984, ftCo_8008E984, (fp: *mut Fighter) -> bool);
    hook!(FTCO_8008D8E8, ftCo_8008D8E8, (f: f32) -> i32);
    hook!(FTCO_SCALEBY154, ftCo_ScaleBy154, (f: f32) -> f32);
    hook!(FTCO_8008DCE0, ftCo_8008DCE0, (gobj: G, a: i32, facing: f32));
    pub static mut FTCO_8008F744: Option<fn(G) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_8008F744(gobj: G)  {
                match unsafe { FTCO_8008F744 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_Damage::ftCo_8008F744(gobj as _); } return; },
                }
            }
    hook!(FTCO_DAMAGE_PHYS, ftCo_Damage_Phys, (gobj: G));
}

/// `checkSomething` (static).
pub fn checkSomething(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if ext::ftCo_8008E984(fp) {
            return true;
        }
        if ext::ftCo_8008D8E8(ext::ftCo_ScaleBy154((*fp).dmg.kb_applied)) < 3 {
            return true;
        }
        false
    }
}

/// `doSomething` (static).
pub fn doSomething(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let a = (*fp).x2CC;
        ext::ftCo_8008DCE0(gobj, (*a).motion_state + 9, 0.0);
    }
}

pub fn ftDk_MS_349_800E06D8(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if checkSomething(gobj) {
            doSomething(gobj);
        } else {
            let mut vec: Vec3 = core::mem::zeroed();
            vec.x = 0.0;
            vec.y = 0.0;
            vec.z = 0.0;
            ext::Item_8026ABD8((*fp).item_gobj, &mut vec, 1.0);
            ext::ftCo_8008E908(gobj, 0.0);
        }
    }
}

pub fn ftDk_HeavyWait2_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        ext::ftCo_8008F744(gobj);
        if (*fp).x221C_b6() == 0 {
            if (*fp).ground_or_air == GA_Air {
                super::ftdonkeyheavyturn::ftDk_MS_345_800E0294(gobj);
            } else {
                super::ftdonkeyheavywait0::ftDk_MS_341_800DF980(gobj);
            }
        }
    }
}

pub fn ftDk_HeavyWait2_Phys(gobj: *mut HSD_GObj) {
    ext::ftCo_Damage_Phys(gobj);
}
