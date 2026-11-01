//! src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c: Master Hand finger-beam states (the
//! laser-spawning accessory callback `ftMh_MS_362_80152E28`). No float arithmetic (no fused sites).
//! Smoke-tested by tests/tu_ftmasterhandfingerbeam.rs.
//!
//! Native layout defect (melee_types, not this TU): `unk0.x4` is an 8-byte `HSD_GObjEvent` here, so
//! the `unk0` fields after it sit 4 bytes later than on the GC, and `fingerbeam.x34..x40` are
//! 8-byte pointers at native 0x38/0x40/0x48/0x50. On the GC the sound handles (`unk0.x28..x30`)
//! and the four item slots are disjoint; natively the third handle (native 0x38) overlays the
//! first item pointer, so that pointer is lost. The port reads and writes every field by name, as
//! the C does, so it is right as soon as the generated layout keeps the GC offsets; until then the
//! first laser is not freed (it_802F046C gets a garbage pointer). Tests pin the overlay.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/4/8): re-exported under this TU.





/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftmasterhanddamage0::ext::{it_802F046C, lbAudioAx_800236B8, lb_8000B1CC};
    pub static mut IT_802F0340: Option<fn(*mut HSD_GObj, *mut Vec3, *mut Vec3, u32, ItemKind, f32) -> *mut Item_GObj> = None;
    #[allow(non_snake_case)]
    pub fn it_802F0340(gobj: *mut HSD_GObj, prev_pos: *mut Vec3, pos: *mut Vec3, part: u32, kind: ItemKind, facing_dir: f32) -> *mut Item_GObj {
        match unsafe { IT_802F0340 } {
            Some(f) => f(gobj, prev_pos, pos, part, kind, facing_dir),
            None => { unsafe { melee_items::tu::itmasterhandlaser::it_802F0340(gobj as _, prev_pos as _, pos as _, part as _, kind, facing_dir as _) as _ } },
        }
    }
    /// `lbAudioAx_800237A8` is the melee_fighter hook `hooks::lbaudioax_800237a8`.
    pub fn lbAudioAx_800237A8(sfx_id: i32, vol: i32, pan: i32) -> i32 {
        melee_fighter::hooks::lbaudioax_800237a8(sfx_id, vol, pan)
    }
}

pub fn ftMh_Poke1_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_Poke1_Coll");}

pub fn ftMh_FingerBeamStart_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerBeamStart_Coll");}

pub fn ftMh_FingerBeamLoop_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamLoop_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        unsafe {
            let fp = (*gobj).user_data as *mut Fighter;
            ftMh_MS_362_80152F80(gobj);
            let items = [
                (*fp).mv.mh.fingerbeam.x34,
                (*fp).mv.mh.fingerbeam.x38,
                (*fp).mv.mh.fingerbeam.x3C,
                (*fp).mv.mh.fingerbeam.x40,
            ];
            for item in items {
                ext::it_802F046C(item);
            }
            (*fp).mv.mh.unk0.x34 = 0;
            (*fp).mv.mh.unk0.x38 = 0;
            (*fp).mv.mh.unk0.x3C = 0;
            (*fp).mv.mh.unk0.x40 = 0;
        }
    }
}

pub fn ftMh_FingerBeamLoop_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerBeamLoop_Coll");}

/// Accessory-4 callback: once the script flags it, spawns four MasterHand lasers and starts their
/// three looping sounds.
pub unsafe extern "C" fn ftMh_MS_362_80152E28(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_362_80152E28");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] == 0 {
            return;
        }
        let parts = (*fp).parts;
        let facing = (*fp).facing_dir;
        let mut vec: Vec3 = core::mem::zeroed();
        let v: *mut Vec3 = &mut vec;
        ext::lb_8000B1CC((*parts.add(FtPart_RLegJA as usize)).joint, core::ptr::null(), v);
        (*fp).mv.mh.fingerbeam.x34 = ext::it_802F0340(gobj, v, v, FtPart_RLegJA as u32, It_Kind_MasterHand_Laser, facing);
        ext::lb_8000B1CC((*parts.add(FtPart_BustN as usize)).joint, core::ptr::null(), v);
        (*fp).mv.mh.fingerbeam.x38 = ext::it_802F0340(gobj, v, v, FtPart_BustN as u32, It_Kind_MasterHand_Laser, facing);
        ext::lb_8000B1CC((*parts.add(FtPart_LHandN as usize)).joint, core::ptr::null(), v);
        (*fp).mv.mh.fingerbeam.x3C = ext::it_802F0340(gobj, v, v, FtPart_LHandN as u32, It_Kind_MasterHand_Laser, facing);
        ext::lb_8000B1CC((*parts.add(FtPart_L3rdNa as usize)).joint, core::ptr::null(), v);
        (*fp).mv.mh.fingerbeam.x40 = ext::it_802F0340(gobj, v, v, FtPart_L3rdNa as u32, It_Kind_MasterHand_Laser, facing);
        (*fp).mv.mh.unk0.x28 = ext::lbAudioAx_800237A8(320004, 127, 64);
        (*fp).mv.mh.unk0.x2C = ext::lbAudioAx_800237A8(320005, 127, 64);
        (*fp).mv.mh.unk0.x30 = ext::lbAudioAx_800237A8(320006, 127, 64);
        (*fp).cmd_vars[0] = 0;
    }
}

/// `ftMh_MS_362_80152F80` (static in the C): enter FingerBeamEnd and stop the three beam sounds.
pub fn ftMh_MS_362_80152F80(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_362_80152F80");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, ftMh_MS_FingerBeamEnd as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        ext::lbAudioAx_800236B8((*fp).mv.mh.unk0.x28);
        ext::lbAudioAx_800236B8((*fp).mv.mh.unk0.x2C);
        ext::lbAudioAx_800236B8((*fp).mv.mh.unk0.x30);
        (*fp).mv.mh.unk0.x28 = -1;
        (*fp).mv.mh.unk0.x2C = -1;
        (*fp).mv.mh.unk0.x30 = -1;
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_FingerBeamStart_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_FingerBeamStart_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamStart_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_FingerBeamLoop_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_FingerBeamLoop_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamLoop_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_359_80152BCC` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_MS_359_80152BCC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_359_80152BCC");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 361 /* ftMh_MS_FingerBeamStart */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x28 = -1_i32;
        (*fp).mv.mh.unk0.x2C = -1_i32;
        (*fp).mv.mh.unk0.x30 = -1_i32;
    }
}

/// `ftMh_FingerBeamStart_Anim` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_FingerBeamStart_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamStart_Anim");
    unsafe {
        if (st::ftAnim_IsFramesRemaining(gobj) == 0) {
            crate::chars::r4::ftMasterHand::ftmasterhandfingerbeam::ftMh_MS_361_80152CD8(gobj);
        }
    }
}

/// `ftMh_MS_361_80152CD8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_MS_361_80152CD8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_361_80152CD8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 362 /* ftMh_MS_FingerBeamLoop */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).accessory4_cb = Some(crate::chars::cb::ftmasterhandfingerbeam__ftMh_MS_362_80152E28);
        (*fp).cmd_vars[0] = 1_u32;
    }
}

/// `ftMh_FingerBeamStart_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_FingerBeamStart_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamStart_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_FingerBeamLoop_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandfingerbeam.c).
pub fn ftMh_FingerBeamLoop_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerBeamLoop_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
