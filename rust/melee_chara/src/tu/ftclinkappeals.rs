//! src/melee/ft/kinds/ftCLink/ftclinkappeals.c: Young Link's side taunt (the milk bottle).
//! No float math (no fused sites). Smoke-tested by tests/tu_ftclinkappeals.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftclink::{checkFighter2244, ftCl_Init_80149268};
use melee_types::*;

/// Already ported (round 2): re-exported under this TU.
pub use crate::chars::r2::ftCLink::ftclinkappeals::ftCl_AppealS_Phys;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
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
    hook!(IT_802C8B28, it_802C8B28, (gobj: *mut HSD_GObj, pos: *mut Vec3, bone: u32, facing: f32) -> *mut HSD_GObj);
    pub static mut FTCO_APPEALS_IASA: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ftCo_AppealS_IASA(gobj: *mut HSD_GObj)  {
                match unsafe { FTCO_APPEALS_IASA } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ftCo_AppealS::ftCo_AppealS_IASA(gobj as _); } return; },
                }
            }
    pub static mut FT_80084280: Option<fn(*mut HSD_GObj) > = None;
            #[allow(non_snake_case)]
            pub fn ft_80084280(gobj: *mut HSD_GObj)  {
                match unsafe { FT_80084280 } {
                    Some(f) => f(gobj),
                    None => { unsafe { let _ = melee_fighter::tu::ft_081b::ft_80084280(gobj as _); } return; },
                }
            }
}

pub fn ftCl_AppealS_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[1] == 1 && (*fp).u.lk.x18.is_null() {
            let bone = melee_fighter::hooks::ftparts_get_bone_index(&*fp, FtPart_LThumbNb as i32);
            let item_gobj = ext::it_802C8B28(gobj, &mut (*fp).cur_pos, bone as u32, (*fp).facing_dir);
            (*fp).u.lk.x18 = item_gobj as *mut Item_GObj;
            if !item_gobj.is_null() {
                (*fp).death2_cb = Some(crate::chars::cb::ftlink__ftLk_800EAF58);
                (*fp).take_dmg_cb = Some(crate::chars::cb::ftlink__ftLk_800EAF58);
            }
        } else if (*fp).cmd_vars[1] == 2 {
            ftCl_Init_80149268(gobj);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            checkFighter2244(gobj);
            st::ft_8008A2BC(gobj);
        }
    }
}

pub fn ftCl_AppealS_IASA(gobj: *mut HSD_GObj) {
    ext::ftCo_AppealS_IASA(gobj);
}

pub fn ftCl_AppealS_Coll(gobj: *mut HSD_GObj) {
    ext::ft_80084280(gobj);
}
