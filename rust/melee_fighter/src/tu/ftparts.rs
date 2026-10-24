//! src/melee/ft/ftparts.c (partial): part-visibility tables, part allocation, part rotation.
//!
//! Already ported elsewhere and re-exported: ftParts_80074A4C/A74/A8C/ACC (`crate::ft0df`),
//! ftParts_GetBoneIndex / ftPartsRemap / ftParts_8007506C (`crate::ftdmgfall`).
//! Not ported yet (render/matrix setup and model loading, HSD/GX heavy): JObjMakePositionMtx,
//! JObjInfoInit, 80073758, IntpJObjLoad, IntpJObjInfoInit, ftPartsSetup{ZScale,Nrm,Tex}Mtx,
//! ftPartsSetupRigidMtx, PObjSetupMtx, PObjInfoInit, PObj{Set,Clear}DefaultClass,
//! SetupParts, 8007462C, 8007482C, 8007487C, 800749CC, 800750C8, 80075240, 80075304,
//! 800755E8, 80075650.
#![allow(non_snake_case)]

use crate::types::*;

pub use crate::ft0df::{ftparts_80074a4c, ftparts_80074a74, ftparts_80074a8c, ftparts_80074acc};
pub use crate::ftdmgfall::{ftparts_8007506c, ftparts_get_bone_index, ftparts_remap};

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    pub fn HSD_DObjSetFlags(dobj: *mut HSD_DObj, flags: u32) { panic!("TODO(group1 dobj) HSD_DObjSetFlags") }
    pub fn HSD_DObjClearFlags(dobj: *mut HSD_DObj, flags: u32) { panic!("TODO(group1 dobj) HSD_DObjClearFlags") }
    pub fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, q: *const Vec4) { unsafe { let _ = melee_world::tu::grinl::HSD_JObjSetRotation(jobj as _, q as _); } }
    pub fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, flags: u32) { panic!("TODO(group1 jobj) HSD_JObjClearFlags") }
    pub fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, v: f32) { unsafe { let _ = melee_world::tu::grinl::HSD_JObjSetRotationX(jobj as _, v as _); } }
    pub fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32) { { unsafe { let _ = crate::tu::ftCo_Lift::HSD_JObjSetRotationY(jobj as _, v as _); } return; } }
    pub fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32) { panic!("TODO(group1 jobj) HSD_JObjSetRotationZ") }
    pub fn HSD_ObjAlloc(data: *mut c_void) -> *mut c_void { { return unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(data as _) as _ }; } }
    pub fn fighter_parts_alloc_data() -> *mut c_void { panic!("TODO(fighter.c) fighter_parts_alloc_data") }
    pub fn fighter_dobj_list_alloc_data() -> *mut c_void { panic!("TODO(fighter.c) fighter_dobj_list_alloc_data") }
}

#[inline]
fn bone<'a>(f: &Fighter, i: i32) -> &'a mut FighterBone {
    unsafe { &mut *f.parts.add(i as usize) }
}

static mut COMPLAINED: i32 = 0;

/// static `ftParts_DObjAt` (TARGET_PC `FT_DOBJ_AT`): bounds-checked `dobj_list->data[idx]`;
/// out of range logs (at most 8 times) and gives NULL.
pub fn ftParts_DObjAt(dobj_list: *const DObjList, idx: i32, _where: &str) -> *mut HSD_DObj {
    unsafe {
        if dobj_list.is_null() || (*dobj_list).data.is_null() || idx < 0 || idx as u32 >= (*dobj_list).count {
            if COMPLAINED < 8 {
                COMPLAINED += 1;
            }
            return core::ptr::null_mut();
        }
        *(*dobj_list).data.add(idx as usize)
    }
}

/// Visit every DObj index of every lookup entry of row `idx`: `f(model, entry, dobj_index)`.
unsafe fn each_entry(vis: &FtPartsVis, lookup: *mut FtPartsVisLookup, mut f: impl FnMut(u32, i32, u8)) {
    for i in 0..vis.model_num {
        let l = &*lookup.add(i as usize);
        for j in 0..l.x0 {
            let e = &*l.x4.add(j as usize);
            for k in 0..e.x0 {
                f(i, j, *e.x4.add(k as usize));
            }
        }
    }
}

/// `ftParts_80074B6C(fp, vis, idx, dobj_list)`: show the selected model variant of each part
/// model, hide the rest. (`cleared[idx] = 1` sits inside the model loop in the C.)
pub fn ftParts_80074B6C(f: &Fighter, vis: &mut FtPartsVis, idx: i32, dobj_list: *const DObjList) {
    let lookup = vis.xC[idx as usize];
    if lookup.is_null() || vis.cleared[idx as usize] != 0 {
        return;
    }
    for i in 0..vis.model_num {
        let r25 = f.x5F4_arr[i as usize].idx as i32;
        let l = unsafe { &*lookup.add(i as usize) };
        for j in 0..l.x0 {
            let e = unsafe { &*l.x4.add(j as usize) };
            for k in 0..e.x0 {
                let d = ftParts_DObjAt(dobj_list, unsafe { *e.x4.add(k as usize) } as i32, "ftParts_80074B6C");
                if j == r25 { ext::HSD_DObjClearFlags(d, 1) } else { ext::HSD_DObjSetFlags(d, 1) }
            }
        }
        vis.cleared[idx as usize] = 1;
    }
}

/// `ftParts_80074CA0(vis, idx, dobj_list)`: show every DObj of row `idx`.
pub fn ftParts_80074CA0(vis: &mut FtPartsVis, idx: i32, dobj_list: *const DObjList) {
    let lookup = vis.xC[idx as usize];
    if !lookup.is_null() && vis.cleared[idx as usize] == 0 {
        unsafe { each_entry(vis, lookup, |_, _, d| ext::HSD_DObjClearFlags(ftParts_DObjAt(dobj_list, d as i32, "ftParts_80074CA0"), 1)) };
        vis.cleared[idx as usize] = 1;
    }
}

/// `ftParts_80074D7C(vis, idx, dobj_list)`: hide every DObj of row `idx`.
pub fn ftParts_80074D7C(vis: &mut FtPartsVis, idx: i32, dobj_list: *const DObjList) {
    let lookup = vis.xC[idx as usize];
    if !lookup.is_null() && vis.cleared[idx as usize] != 0 {
        unsafe { each_entry(vis, lookup, |_, _, d| ext::HSD_DObjSetFlags(ftParts_DObjAt(dobj_list, d as i32, "ftParts_80074D7C"), 1)) };
        vis.cleared[idx as usize] = 0;
    }
}

/// `ftParts_80074B0C(gobj, model_idx, val)`: select a model variant.
pub fn ftParts_80074B0C(gobj: *mut HSD_GObj, model_idx: i32, val: i32) {
    let f = super::ftlipstickswing::fp(gobj);
    if val != f.x5F4_arr[model_idx as usize].idx as i32 {
        f.x5F4_arr[model_idx as usize].idx = val as _;
        f.set_x221D_b2(1);
        let dl: *const DObjList = &f.dobj_list;
        ftParts_80074D7C(&mut f.x5AC, 0, dl);
    }
}

/// `ftParts_80074E58(fp)`: allocate the part and DObj arrays and mark the root parts.
pub fn ftParts_80074E58(f: &mut Fighter) {
    f.parts = ext::HSD_ObjAlloc(ext::fighter_parts_alloc_data()) as *mut FighterBone;
    f.dobj_list.data = ext::HSD_ObjAlloc(ext::fighter_dobj_list_alloc_data()) as *mut *mut HSD_DObj;
    let n = unsafe { (**crate::hooks::ft_parts_table().add(f.kind as usize)).parts_num };
    for i in 0..n as i32 {
        let b = bone(f, i);
        b.anon_8.flags8 = 0;
        b.anon_c.flagsC = 0;
    }
    unsafe {
        bone(f, 0).anon_8.anon_0.set_flags_b3(1);
        for p in [FtPart_TransN, FtPart_XRotN, FtPart_YRotN, FtPart_HipN, FtPart_TransN2] {
            bone(f, ftparts_get_bone_index(f, p as i32)).anon_8.anon_0.set_flags_b3(1);
        }
        bone(f, ftparts_get_bone_index(f, FtPart_TransN as i32)).anon_8.anon_0.set_flags_b4(1);
        bone(f, ftparts_get_bone_index(f, 0x35)).anon_8.anon_0.set_flags_b4(1);
    }
}

pub fn ftParts_JObjSetRotation(jobj: *mut HSD_JObj, quat: *const Vec4) {
    ext::HSD_JObjSetRotation(jobj, quat);
    ext::HSD_JObjClearFlags(jobj, JOBJ_USE_QUATERNION as u32);
}

#[inline]
fn uses_quat(j: *mut HSD_JObj) -> bool {
    unsafe { (*j).flags & JOBJ_USE_QUATERNION as u32 != 0 }
}

/// The euler-rotation target of a part: its joint, or `x4_jobj2` when the joint is quaternion.
fn rot_target(f: &Fighter, part_idx: i32, what: &str) -> *mut HSD_JObj {
    let b = bone(f, part_idx);
    if uses_quat(b.joint) {
        assert!(!uses_quat(b.x4_jobj2), "cant {what}!");
        b.x4_jobj2
    } else {
        b.joint
    }
}

pub fn ftPartSetRotX(f: &Fighter, part_idx: i32, v: f32) { ext::HSD_JObjSetRotationX(rot_target(f, part_idx, "set fighter rot x"), v) }
pub fn ftPartSetRotY(f: &Fighter, part_idx: i32, v: f32) { ext::HSD_JObjSetRotationY(rot_target(f, part_idx, "set fighter rot y"), v) }
pub fn ftPartSetRotZ(f: &Fighter, part_idx: i32, v: f32) { ext::HSD_JObjSetRotationZ(rot_target(f, part_idx, "set fighter rot z"), v) }

pub fn ftPartGetRotX(f: &Fighter, part_idx: i32) -> f32 {
    melee_types::inline::HSD_JObjGetRotationX(unsafe { &*rot_target(f, part_idx, "get fighter rot x") })
}

/// `ftPartGetRotZ`: returns the Y rotation, as the C does.
pub fn ftPartGetRotZ(f: &Fighter, part_idx: i32) -> f32 {
    melee_types::inline::HSD_JObjGetRotationY(unsafe { &*rot_target(f, part_idx, "get fighter rot y") })
}

pub use super::ftparts_render::*;
