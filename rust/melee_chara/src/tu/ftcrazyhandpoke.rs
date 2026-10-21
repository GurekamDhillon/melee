//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandpoke.c: Crazy Hand paper-crush / poke states (the
//! laser-spawning accessory callback `fn_80158534`). No float arithmetic (no fused sites).
//! Smoke-tested by tests/tu_ftcrazyhandpoke.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftcrazyhandfingerbeam0::ext::{ftCh_GrabUnk1_8015BC88, lb_8000B1CC};
use melee_types::*;

/// Already ported (rounds 2/3/4/8): re-exported under this TU.
pub use crate::chars::r2::ftCrazyHand::ftcrazyhandpoke::{ftCh_PaperCrush_Phys, ftCh_Poke1_Phys, ftCh_Poke2_Phys};
pub use crate::chars::r3::ftCrazyHand::ftcrazyhandpoke::fn_801582D8;
pub use crate::chars::r4::ftCrazyHand::ftcrazyhandpoke::{ftCh_Init_801583E4, ftCh_PaperCrush_Anim};
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandpoke::{ftCh_PaperCrush_IASA, ftCh_Poke1_IASA, ftCh_Poke2_IASA};

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    crate::ch_hook!(IT_802F0340, it_802F0340, (gobj: *mut HSD_GObj, prev_pos: *mut Vec3, pos: *mut Vec3, part: u32, kind: ItemKind, facing_dir: f32) -> *mut Item_GObj);
    crate::ch_hook!(IT_802F046C, it_802F046C, (gobj: *mut Item_GObj));
    crate::ch_hook!(LBAUDIOAX_800236B8, lbAudioAx_800236B8, (id: i32) -> i32);
}

pub fn ftCh_PaperCrush_Coll(_gobj: *mut HSD_GObj) {}

pub fn ftCh_Poke1_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            // Native layout caveat: `mv.ch.grab.x30/x34` are 8-byte pointers here but 4 bytes on
            // the GC, so they overlay `mv.ch.unk0.x38/x3C/x40`, which ftCh_Init_8015868C resets to
            // -1. On the GC the slots are disjoint, so the C's end state is: items freed, the four
            // slots NULL, x38/x3C/x40 = -1. Reading pointers and sound ids first, clearing the
            // slots, restoring the ids, then running Init (-1 stores last) gives that end state.
            // (fn_80158534 already lost grab.x30/x34 to the ids it stored; that needs the
            // melee_types layout fixed, not this TU.)
            let items = [(*fp).mv.ch.grab.x28, (*fp).mv.ch.grab.x2C, (*fp).mv.ch.grab.x30, (*fp).mv.ch.grab.x34];
            let ids = [(*fp).mv.ch.unk0.x38, (*fp).mv.ch.unk0.x3C, (*fp).mv.ch.unk0.x40];
            (*fp).mv.ch.grab.x28 = core::ptr::null_mut();
            (*fp).mv.ch.grab.x2C = core::ptr::null_mut();
            (*fp).mv.ch.grab.x30 = core::ptr::null_mut();
            (*fp).mv.ch.grab.x34 = core::ptr::null_mut();
            (*fp).mv.ch.unk0.x38 = ids[0];
            (*fp).mv.ch.unk0.x3C = ids[1];
            (*fp).mv.ch.unk0.x40 = ids[2];
            ftCh_Init_8015868C(gobj);
            for it in items {
                ext::it_802F046C(it);
            }
        }
    }
}

pub fn ftCh_Poke1_Coll(_gobj: *mut HSD_GObj) {}

/// Accessory-4 callback: once the script flags it, spawns four MasterHand lasers and starts their
/// three looping sounds.
pub unsafe extern "C" fn fn_80158534(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] == 0 {
            return;
        }
        let parts = (*fp).parts;
        let facing = (*fp).facing_dir;
        let mut vec: Vec3 = core::mem::zeroed();
        let v: *mut Vec3 = &mut vec;
        lb_8000B1CC((*parts.add(FtPart_RLegJA as usize)).joint, core::ptr::null(), v);
        (*fp).mv.ch.grab.x28 = ext::it_802F0340(gobj, v, v, FtPart_RLegJA as u32, It_Kind_MasterHand_Laser, facing);
        lb_8000B1CC((*parts.add(FtPart_BustN as usize)).joint, core::ptr::null(), v);
        (*fp).mv.ch.grab.x2C = ext::it_802F0340(gobj, v, v, FtPart_BustN as u32, It_Kind_MasterHand_Laser, facing);
        lb_8000B1CC((*parts.add(FtPart_LHandN as usize)).joint, core::ptr::null(), v);
        (*fp).mv.ch.grab.x30 = ext::it_802F0340(gobj, v, v, FtPart_LHandN as u32, It_Kind_MasterHand_Laser, facing);
        lb_8000B1CC((*parts.add(FtPart_L3rdNa as usize)).joint, core::ptr::null(), v);
        (*fp).mv.ch.grab.x34 = ext::it_802F0340(gobj, v, v, FtPart_L3rdNa as u32, It_Kind_MasterHand_Laser, facing);
        (*fp).mv.ch.unk0.x38 = melee_fighter::hooks::lbaudioax_800237a8(320004, 127, 64);
        (*fp).mv.ch.unk0.x3C = melee_fighter::hooks::lbaudioax_800237a8(320005, 127, 64);
        (*fp).mv.ch.unk0.x40 = melee_fighter::hooks::lbaudioax_800237a8(320006, 127, 64);
        (*fp).cmd_vars[0] = 0;
    }
}

pub fn ftCh_Init_8015868C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 0x168, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x38);
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x3C);
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x40);
        (*fp).mv.ch.unk0.x38 = -1;
        (*fp).mv.ch.unk0.x3C = -1;
        (*fp).mv.ch.unk0.x40 = -1;
    }
}

pub fn ftCh_Poke2_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ftCh_GrabUnk1_8015BC88(gobj);
    }
}

pub fn ftCh_Poke2_Coll(_gobj: *mut HSD_GObj) {}
