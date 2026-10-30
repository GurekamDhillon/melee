//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam0.c: Crazy Hand's finger-beam start
//! (the bomb-spawning accessory callback). No float arithmetic beyond `+=` (no fused sites).
//! Smoke-tested by tests/tu_ftcrazyhandfingerbeam0.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/4/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandfingerbeam0::ftCh_FingerBeamStart_Phys;
pub use crate::chars::r4::ftCrazyHand::ftcrazyhandfingerbeam0::fn_801587B0;
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandfingerbeam0::ftCh_FingerBeamStart_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
/// (The hooks shared by the ftcrazyhand* TUs of this shard are defined here once.)
pub mod ext {
    use melee_types::*;
    #[macro_export]
    macro_rules! ch_hook {
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
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *const Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *const Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } {
            Some(f) => f(jobj, pos0, pos1),
            None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } },
        }
    }
    pub static mut FTCH_GRABUNK1_8015BC88: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCh_GrabUnk1_8015BC88(gobj: *mut HSD_GObj) {
        match unsafe { FTCH_GRABUNK1_8015BC88 } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = crate::tu::ftcrazyhandtagcancel::ftCh_GrabUnk1_8015BC88(gobj as _); } },
        }
    }
    crate::port_hook!(IT_802F0F6C, it_802F0F6C, crate::adapt::it_802F0F6C, (owner: *mut HSD_GObj, prev_pos: *mut Vec3, pos: *mut Vec3, kind: ItemKind, facing_dir: f32));
}

pub fn ftCh_FingerBeamStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_FingerBeamStart_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_FingerBeamStart_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_FingerBeamStart_Coll");}

/// Accessory-4 callback: spawns a CrazyHand bomb for each of the four cmd_vars flags set by the
/// motion script, at a bone plus a data offset.
pub unsafe extern "C" fn fn_801588B8(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801588B8");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        let parts = (*fp).parts;
        let sites: [(usize, usize, f32, f32); 4] = [
            (0, FtPart_RLegJA as usize, (*da).xA0, (*da).xA4),
            (1, FtPart_BustN as usize, (*da).xA8_pos.x, (*da).xA8_pos.y),
            (2, FtPart_LHandN as usize, (*da).xB0, (*da).xB4),
            (3, FtPart_L3rdNa as usize, (*da).xB8, (*da).xBC_pos.x),
        ];
        for (i, part, dx, dy) in sites {
            if (*fp).cmd_vars[i] != 0 {
                let mut pos: Vec3 = core::mem::zeroed();
                ext::lb_8000B1CC((*parts.add(part)).joint, core::ptr::null(), &mut pos);
                let mut prev = pos;
                pos.x += dx;
                pos.y += dy;
                ext::it_802F0F6C(gobj, &mut pos, &mut prev, It_Kind_CrazyHand_Bomb, (*fp).facing_dir);
                (*fp).cmd_vars[i] = 0;
                melee_fighter::hooks::play_sfx(&mut *fp, 0x4E203, 0x7F, 0x40);
            }
        }
    }
}
