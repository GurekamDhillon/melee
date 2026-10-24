//! src/melee/gr/grdatfiles.c (+ grdatfiles_stage_slot_retail.inc): the stage archive slots
//! (`grDatFiles_8049EE10[4]`), publishing a stage DAT's public symbols into `stage_info`, and the default
//! `grDatFiles_803E0848` / `grDatFiles_803E0924` tables used when a stage has no DAT. TARGET_PC bodies
//! (`grDatFiles_GetArchive` / `grDatFiles_801C6330` ask `ScriptGame_StageSlotArchive` first; the
//! `grDatFiles_801C62B4` slot finder has external linkage). No float arithmetic.
//! State-oracle checked against the C (tests/gro_grdatfiles.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::assert_;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::stage_info;
    pub static mut HSD_ARCHIVEGETPUBLICADDRESS: Option<fn(*mut HSD_Archive, *const i8) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void {
        match unsafe { HSD_ARCHIVEGETPUBLICADDRESS } { Some(f) => f(a0, a1), None => { return unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ }; } }
    }
    hook!(SCRIPTGAME_STAGESLOTARCHIVE, ScriptGame_StageSlotArchive, (slot: i32) -> *mut UnkArchiveStruct);
    hook_or!(LBARCHIVE_80016DBC, lbArchive_80016DBC, (filename: *const i8, symbols: *mut c_void, va: &[usize]) -> *mut HSD_Archive, |filename: *const i8, symbols: *mut c_void, va: &[usize]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![symbols as *mut core::ffi::c_void]; v.extend(va.iter().map(|x| *x as *mut core::ffi::c_void)); if v.last().map_or(true, |p| !p.is_null()) { v.push(core::ptr::null_mut()); } melee_runtime::tu::lbarchive::lbArchive_80016DBC(filename as _, &v) as _ } });
    hook_or!(LBARCHIVE_800171CC, lbArchive_800171CC, (dst: *mut *mut HSD_Archive, filename: *const i8, symbols: *mut c_void, va: &[usize]) -> i32, |dst: *mut *mut HSD_Archive, filename: *const i8, symbols: *mut c_void, va: &[usize]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![symbols as *mut core::ffi::c_void]; v.extend(va.iter().map(|x| *x as *mut core::ffi::c_void)); if v.last().map_or(true, |p| !p.is_null()) { v.push(core::ptr::null_mut()); } melee_runtime::tu::lbarchive::lbArchive_800171CC(dst as _, filename as _, &v) as i32 } });
    pub static mut LBARCHIVE_INITIALIZEDAT: Option<fn(*mut HSD_Archive, *mut c_void, u32)> = None;
    #[allow(non_snake_case)]
    pub fn lbArchive_InitializeDAT(archive: *mut HSD_Archive, data: *mut c_void, length: u32) {
        match unsafe { LBARCHIVE_INITIALIZEDAT } { Some(f) => f(archive, data, length), None => { unsafe { let _ = melee_runtime::tu::lbarchive::lbArchive_InitializeDAT(archive as _, data as _, length as _); } return; } }
    }
    pub static mut LBHEAP_80015BD0: Option<fn(i32, u32) -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn lbHeap_80015BD0(heap_id: i32, size: u32) -> *mut c_void {
        match unsafe { LBHEAP_80015BD0 } { Some(f) => f(heap_id, size), None => { return unsafe { melee_runtime::tu::lbheap::lbHeap_80015BD0(heap_id as _, size as _) as _ }; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }
    pub static mut PSINITDATABANK: Option<fn(i32, *mut c_void, *mut c_void, *mut u32, *mut i32)> = None;
    #[allow(non_snake_case)]
    pub fn psInitDataBank(bank: i32, cmdBank: *mut c_void, texBank: *mut c_void, r#ref: *mut u32, formBank: *mut i32) {
        match unsafe { PSINITDATABANK } {
            Some(f) => f(bank, cmdBank, texBank, r#ref, formBank),
            None => { unsafe { let _ = melee_runtime::tu::particle::psInitDataBank(bank as _, cmdBank as _, texBank as _, r#ref as _, formBank as _); } },
        }
    }
    pub static mut PSINITDATABANKLOAD: Option<fn(i32, *mut c_void, *mut c_void, *mut u32, *mut i32)> = None;
    #[allow(non_snake_case)]
    pub fn psInitDataBankLoad(bank: i32, cmdBank: *mut c_void, texBank: *mut c_void, r#ref: *mut u32, formBank: *mut i32) {
        match unsafe { PSINITDATABANKLOAD } {
            Some(f) => f(bank, cmdBank, texBank, r#ref, formBank),
            None => { unsafe { let _ = melee_runtime::tu::particle::psInitDataBankLoad(bank as _, cmdBank as _, texBank as _, r#ref as _, formBank as _); } },
        }
    }
    pub static mut PSINITDATABANKLOCATE: Option<fn(*mut HSD_Archive, *mut HSD_Archive, *mut i32)> = None;
    #[allow(non_snake_case)]
    pub fn psInitDataBankLocate(cmdBank: *mut HSD_Archive, texBank: *mut HSD_Archive, formBank: *mut i32) {
        match unsafe { PSINITDATABANKLOCATE } { Some(f) => f(cmdBank, texBank, formBank), None => { unsafe { let _ = melee_runtime::tu::particle::psInitDataBankLocate(cmdBank as _, texBank as _, formBank as _); } return; } }
    }
}

#[inline(always)]
fn cs(s: &'static [u8]) -> *const i8 {
    s.as_ptr() as *const i8
}

/// `static UnkArchiveStruct grDatFiles_8049EE10[4]` (49EE10): the four stage archive slots.
pub static mut grDatFiles_8049EE10: [UnkArchiveStruct; 4] = unsafe { core::mem::zeroed() };

/// `static StageParam grDatFiles_803E07E4`.
pub static mut grDatFiles_803E07E4: StageParam =
    StageParam { stkind: 0, x4: -1, x8: -1, xC: 0, x10: 0, x14: 0, x16: 0, x18: 0, x1A: [0; 37] };

/// `GroundParam grDatFiles_803E0848`: the default ground parameters (no stage DAT).
pub static mut grDatFiles_803E0848: GroundParam = GroundParam {
    y: 1.0,
    x4: 0x80,
    x6_pad: [0; 2],
    x8: 0x1E,
    xA: 0,
    xC: 1,
    x10: 0x8000,
    x14: 10,
    x18: 0.0,
    x1C: 0.0,
    x20: 1.0,
    x24: 1.0,
    x28: 1.0,
    x2C_pad: [0; 2],
    x2E: 40,
    x30: 10,
    x34: 50,
    x38: 100,
    x3C: 10.0,
    x40: 10.0,
    x44: 10.0,
    x48: 10.0,
    x4C_fixed_cam: 0,
    x50: 0.0,
    x54: 0.0,
    x58: 0.0,
    x5C: 30.0,
    x60: 10.0,
    x64: 0.0,
    x68: 0,
    x6A: [0; 35],
    stage_params: unsafe { addr_of_mut!(grDatFiles_803E07E4) },
    stage_param_count: 1,
    xB8: unsafe { core::mem::zeroed() },
    xBC: unsafe { core::mem::zeroed() },
    xC0: unsafe { core::mem::zeroed() },
    xC4: unsafe { core::mem::zeroed() },
    xC8: unsafe { core::mem::zeroed() },
    xCC: unsafe { core::mem::zeroed() },
    xD0: unsafe { core::mem::zeroed() },
    xD4: unsafe { core::mem::zeroed() },
    xD8: unsafe { core::mem::zeroed() },
};

/// `UnkStageDat grDatFiles_803E0924 = { 0 }`.
pub static mut grDatFiles_803E0924: UnkStageDat = unsafe { core::mem::zeroed() };

pub unsafe fn grDatFiles_801C5FC0(archive: *mut HSD_Archive, data: *mut c_void, length: u32) {
    ext::lbArchive_InitializeDAT(archive, data, length);
    let map_ptcl = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_ptcl\0")) as *mut HSD_Archive;
    let map_texg = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_texg\0")) as *mut HSD_Archive;

    if !map_ptcl.is_null() && !map_texg.is_null() {
        ext::psInitDataBankLocate(map_ptcl, map_texg, null_mut());
    }
}

pub unsafe fn grDatFiles_801C6038(arg0: *mut c_void, arg1: s32, arg2: s32) {
    let temp_r3 = grDatFiles_801C62B4();
    let si = ext::stage_info();
    if !arg0.is_null() {
        let mut sp14: *mut HSD_Archive = null_mut();
        let phi_r28: s32;
        let r4 = arg0;
        let sym = addr_of_mut!((*temp_r3).unk4) as *mut c_void;
        if arg2 != 0 {
            phi_r28 = ext::lbArchive_800171CC(&mut sp14, r4 as *const i8, sym, &[cs(b"map_head\0") as usize, 0]);
        } else {
            sp14 = ext::lbArchive_80016DBC(r4 as *const i8, sym, &[cs(b"map_head\0") as usize, 0]);
            phi_r28 = 0;
        }
        (*temp_r3).unk8 = 0;
        if arg1 == 0 {
            (*si).coll_data = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"coll_data\0")) as *mut MapCollData;
            (*si).param = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"grGroundParam\0")) as *mut GroundParam;
            (*si).itemdata = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"itemdata\0")) as *mut *mut GroundItemData;
            (*si).ald_yaku_all = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"ALDYakuAll\0")) as *mut *mut *mut c_void;
            (*si).map_ptcl = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"map_ptcl\0"));
            (*si).map_texg = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"map_texg\0"));
            (*si).yakumono_param = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"yakumono_param\0"));
            (*si).map_plit = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"map_plit\0")) as *mut *mut LightList;
            (*si).quake_model_set = ext::HSD_ArchiveGetPublicAddress(sp14, cs(b"quake_model_set\0")) as *mut DynamicModelDesc;
        }
        (*temp_r3).unk0 = sp14;
        if !(*si).map_ptcl.is_null() && !(*si).map_texg.is_null() {
            if phi_r28 != 0 {
                ext::psInitDataBankLoad(0x40, (*si).map_ptcl, (*si).map_texg, null_mut(), null_mut());
            } else {
                ext::psInitDataBank(0x40, (*si).map_ptcl, (*si).map_texg, null_mut(), null_mut());
            }
        }
        grDatFiles_801C6228((*temp_r3).unk4);
    } else {
        (*temp_r3).unk4 = addr_of_mut!(grDatFiles_803E0924);
        if arg1 == 0 {
            (*si).coll_data = null_mut();
            (*si).param = addr_of_mut!(grDatFiles_803E0848);
            (*si).itemdata = null_mut();
            (*si).ald_yaku_all = null_mut();
            (*si).map_ptcl = null_mut();
            (*si).map_texg = null_mut();
            (*si).yakumono_param = null_mut();
            (*si).map_plit = null_mut();
            (*si).x6C8 = null_mut();
        }
        (*temp_r3).unk0 = usize::MAX as *mut HSD_Archive;
    }
}

pub unsafe fn grDatFiles_801C6228(arg0: *mut UnkStageDat) {
    if !arg0.is_null() && !(*arg0).unk28.is_null() && (*arg0).unk2C != 0 {
        let mut i = 0;
        while i < (*arg0).unk2C {
            let temp_r4 = *(*arg0).unk28.offset(i as isize);
            if !temp_r4.is_null() {
                (*temp_r4).unk4 |= 0x4000000;
            }
            i += 1;
        }
    }
}

pub unsafe fn grDatFiles_801C6288() {
    ext::memzero(addr_of_mut!(grDatFiles_8049EE10) as *mut c_void, core::mem::size_of::<[UnkArchiveStruct; 4]>() as ssize_t);
}

/// External linkage under TARGET_PC (an m-ex stage blob calls it by address).
pub unsafe fn grDatFiles_801C62B4() -> *mut UnkArchiveStruct {
    let mut i = 0;
    while i < 4 {
        if grDatFiles_8049EE10[i].unk0.is_null() {
            return addr_of_mut!(grDatFiles_8049EE10[i]);
        }
        i += 1;
    }
    assert_(false, 229, "0");
    unreachable!()
}

pub unsafe fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct {
    let slot = ext::ScriptGame_StageSlotArchive(0);
    if !slot.is_null() {
        return slot;
    }
    addr_of_mut!(grDatFiles_8049EE10) as *mut UnkArchiveStruct
}

pub unsafe fn grDatFiles_801C6330(arg0: s32) -> *mut UnkArchiveStruct {
    let slot_archive = ext::ScriptGame_StageSlotArchive(arg0);
    if !slot_archive.is_null() {
        return slot_archive;
    }
    if arg0 >= 0 {
        let mut i = 0;
        while i < 4 {
            if !grDatFiles_8049EE10[i].unk0.is_null() {
                let temp_r7 = grDatFiles_8049EE10[i].unk4;
                if !temp_r7.is_null()
                    && (*temp_r7).unkC > arg0
                    && !(*(*temp_r7).unk8.offset(arg0 as isize)).unk0.is_null()
                {
                    return addr_of_mut!(grDatFiles_8049EE10[i]);
                }
            }
            i += 1;
        }
    }
    null_mut()
}

pub unsafe fn grDatFiles_801C6478(data: *mut c_void, length: s32) -> *mut UnkArchiveStruct {
    let archive = ext::lbHeap_80015BD0(0, core::mem::size_of::<HSD_Archive>() as u32) as *mut HSD_Archive;
    ext::lbArchive_InitializeDAT(archive, data, length as u32);
    let arc = grDatFiles_801C62B4();
    assert_(!arc.is_null(), 290, "arc");
    (*arc).unk0 = archive;
    (*arc).unk4 = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_head\0")) as *mut UnkStageDat;
    (*arc).unk8 = 1;

    grDatFiles_801C6228((*arc).unk4);

    arc
}

/// grdatfiles_stage_slot_retail.inc: the same archive publication as `grDatFiles_801C6038` from a cached visit.
pub unsafe fn grDatFiles_StageSlotInstall(archive: *mut HSD_Archive) {
    let dst = grDatFiles_801C62B4();
    let si = ext::stage_info();
    (*dst).unk0 = archive;
    (*dst).unk4 = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_head\0")) as *mut UnkStageDat;
    (*dst).unk8 = 0;
    (*si).coll_data = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"coll_data\0")) as *mut MapCollData;
    (*si).param = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"grGroundParam\0")) as *mut GroundParam;
    (*si).itemdata = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"itemdata\0")) as *mut *mut GroundItemData;
    (*si).ald_yaku_all = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"ALDYakuAll\0")) as *mut *mut *mut c_void;
    (*si).map_ptcl = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_ptcl\0"));
    (*si).map_texg = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_texg\0"));
    (*si).yakumono_param = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"yakumono_param\0"));
    (*si).map_plit = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"map_plit\0")) as *mut *mut LightList;
    (*si).quake_model_set = ext::HSD_ArchiveGetPublicAddress(archive, cs(b"quake_model_set\0")) as *mut DynamicModelDesc;
    if !(*si).map_ptcl.is_null() && !(*si).map_texg.is_null() {
        ext::psInitDataBank(0x40, (*si).map_ptcl, (*si).map_texg, null_mut(), null_mut());
    }
    grDatFiles_801C6228((*dst).unk4);
}
