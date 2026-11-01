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

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon::calc_hitlag;
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::ftcommon2 as fc2;
#[allow(unused_imports)]
use crate::ftcommon3::ftco_get_cstick_angle;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ft0df::{G, fp};
use crate::types::*;

pub use crate::ftdmgfall::{ftparts_get_bone_index, ftparts_remap};
/// Callees outside this TU: settable hooks (the C oracle in tests/tu_ftparts_vs_c.rs installs recording
/// stubs); unset they run the runtime / world port.
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn HSD_DObjSetFlags(dobj: *mut HSD_DObj, flags: u32) = unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjSetFlags(dobj as _, flags) };
        fn HSD_DObjClearFlags(dobj: *mut HSD_DObj, flags: u32) = unsafe { melee_runtime::tu::dobj::hsd::HSD_DObjClearFlags(dobj as _, flags) };
        fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, q: *const Vec4) = unsafe { melee_world::tu::grinl::HSD_JObjSetRotation(jobj as _, q as _); };
        fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, flags: u32) = unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjClearFlags(jobj as _, flags) };
        fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, v: f32) = unsafe { melee_world::tu::grinl::HSD_JObjSetRotationX(jobj as _, v as _); };
        fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32) = unsafe { crate::tu::ftCo_Lift::HSD_JObjSetRotationY(jobj as _, v as _); };
        fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32) = unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetRotationZ(jobj as _, v) };
        fn Mex_CostumeVisIdx(fk: i32, costume: i32) -> i32 = melee_runtime::tu::mex_data::Mex_CostumeVisIdx(fk, costume);
        fn HSD_ObjAlloc(data: *mut c_void) -> *mut c_void = unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(data as _) as _ };
    }
    pub fn fighter_parts_alloc_data() -> *mut c_void { unsafe { (&raw mut crate::tu::fighter::fighter_parts_alloc_data) as *mut c_void } }
    pub fn fighter_dobj_list_alloc_data() -> *mut c_void { unsafe { (&raw mut crate::tu::fighter::fighter_dobj_list_alloc_data) as *mut c_void } }
}

#[inline]
fn bone<'a>(f: &Fighter, i: i32) -> &'a mut FighterBone {
    unsafe { &mut *f.parts.add(i as usize) }
}

static mut COMPLAINED: i32 = 0;

/// static `ftParts_DObjAt` (TARGET_PC `FT_DOBJ_AT`): bounds-checked `dobj_list->data[idx]`;
/// out of range logs (at most 8 times) and gives NULL.
pub fn ftParts_DObjAt(dobj_list: *const DObjList, idx: i32, _where: &str) -> *mut HSD_DObj {
    melee_core::ct!("ftParts_DObjAt");
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
    melee_core::ct!("ftParts_80074B6C");
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
    melee_core::ct!("ftParts_80074CA0");
    let lookup = vis.xC[idx as usize];
    if !lookup.is_null() && vis.cleared[idx as usize] == 0 {
        unsafe { each_entry(vis, lookup, |_, _, d| ext::HSD_DObjClearFlags(ftParts_DObjAt(dobj_list, d as i32, "ftParts_80074CA0"), 1)) };
        vis.cleared[idx as usize] = 1;
    }
}

/// `ftParts_80074D7C(vis, idx, dobj_list)`: hide every DObj of row `idx`.
pub fn ftParts_80074D7C(vis: &mut FtPartsVis, idx: i32, dobj_list: *const DObjList) {
    melee_core::ct!("ftParts_80074D7C");
    let lookup = vis.xC[idx as usize];
    if !lookup.is_null() && vis.cleared[idx as usize] != 0 {
        unsafe { each_entry(vis, lookup, |_, _, d| ext::HSD_DObjSetFlags(ftParts_DObjAt(dobj_list, d as i32, "ftParts_80074D7C"), 1)) };
        vis.cleared[idx as usize] = 0;
    }
}

/// `ftParts_80074B0C(gobj, model_idx, val)`: select a model variant.
pub fn ftParts_80074B0C(gobj: *mut HSD_GObj, model_idx: i32, val: i32) {
    melee_core::ct!("ftParts_80074B0C");
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
    melee_core::ct!("ftParts_80074E58");
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
    melee_core::ct!("ftParts_JObjSetRotation");
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

pub fn ftPartSetRotX(f: &Fighter, part_idx: i32, v: f32) { melee_core::ct!("ftPartSetRotX"); ext::HSD_JObjSetRotationX(rot_target(f, part_idx, "set fighter rot x"), v) }
pub fn ftPartSetRotY(f: &Fighter, part_idx: i32, v: f32) { melee_core::ct!("ftPartSetRotY"); ext::HSD_JObjSetRotationY(rot_target(f, part_idx, "set fighter rot y"), v) }
pub fn ftPartSetRotZ(f: &Fighter, part_idx: i32, v: f32) { melee_core::ct!("ftPartSetRotZ"); ext::HSD_JObjSetRotationZ(rot_target(f, part_idx, "set fighter rot z"), v) }

pub fn ftPartGetRotX(f: &Fighter, part_idx: i32) -> f32 {
    melee_core::ct!("ftPartGetRotX");
    melee_types::inline::HSD_JObjGetRotationX(unsafe { &*rot_target(f, part_idx, "get fighter rot x") })
}

/// `ftPartGetRotZ`: returns the Y rotation, as the C does.
pub fn ftPartGetRotZ(f: &Fighter, part_idx: i32) -> f32 {
    melee_core::ct!("ftPartGetRotZ");
    melee_types::inline::HSD_JObjGetRotationY(unsafe { &*rot_target(f, part_idx, "get fighter rot y") })
}

pub use super::ftparts_render::*;

// ---- the part setup (SetupParts, 8007462C), the visibility init (800749CC), ftParts_800750C8, 80075240

/// `ftParts_SetupParts(gobj)`: walk the model's joint tree in the fighter's part order, filling
/// `parts[]` (model joint, flags, tree depth, DObj ranges) and the DObj list.
pub unsafe fn ftParts_SetupParts(fighter_obj: *mut HSD_GObj) {
    melee_core::ct!("ftParts_SetupParts");
    let mut jobj = (*fighter_obj).hsd_obj as *mut HSD_JObj;
    let fp = (*fighter_obj).user_data as *mut Fighter;
    let mut part: u32 = 0;
    let mut tree_depth: u32 = 0;
    let mut dobj_count: i32 = 0;
    let parts_num = (**crate::hooks::ft_parts_table().add((*fp).kind as usize)).parts_num;

    if parts_num as i32 > MAX_FT_PARTS {
        panic!("fighter parts num over! player {}\n", (*fp).player_id);
    }
    while !jobj.is_null() {
        if crate::ftdmgfall::ftparts_8007506c((*fp).kind as i32, part as i32) != 0 {
            (*(*fp).parts.add(part as usize)).joint = core::ptr::null_mut();
            part += 1;
            continue;
        }
        crate::tu::ftparts_render::ftParts_80074194(fp, (*fp).parts.add(part as usize), jobj, &mut dobj_count, tree_depth);
        part += 1;
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !(*jobj).child.is_null() {
            // descend the left side of the tree
            jobj = (*jobj).child;
            tree_depth += 1;
            continue;
        }
        if !(*jobj).next.is_null() {
            // visit bottom nodes from left to right
            jobj = (*jobj).next;
            continue;
        }
        loop {
            if (*jobj).parent.is_null() {
                jobj = core::ptr::null_mut(); // finished
                break;
            }
            // go back up the tree until we can continue to the right
            if !(*(*jobj).parent).next.is_null() {
                jobj = (*(*jobj).parent).next;
                tree_depth = tree_depth.wrapping_sub(1);
                break;
            }
            jobj = (*jobj).parent;
            tree_depth = tree_depth.wrapping_sub(1);
        }
    }
    (*fp).dobj_list.count = dobj_count as u32;
    if part != parts_num {
        panic!("fighter parts num not match! player {}\n", (*fp).player_id);
    }
}

/// `ftParts_8007462C(gobj)`: the same walk over the animation skeleton, filling `parts[].x4_jobj2`.
pub unsafe fn ftParts_8007462C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftParts_8007462C");
    let fp = (*gobj).user_data as *mut Fighter;
    let mut i: u32 = 0;
    let mut jobj = (*fp).x8AC_animSkeleton;
    while !jobj.is_null() {
        if crate::ftdmgfall::ftparts_8007506c((*fp).kind as i32, i as i32) != 0 {
            (*(*fp).parts.add(i as usize)).x4_jobj2 = core::ptr::null_mut();
            i += 1;
            continue;
        }
        (*(*fp).parts.add(i as usize)).x4_jobj2 = jobj;
        i += 1;
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !(*jobj).child.is_null() {
            jobj = (*jobj).child;
            continue;
        }
        if !(*jobj).next.is_null() {
            jobj = (*jobj).next;
            continue;
        }
        loop {
            if (*jobj).parent.is_null() {
                jobj = core::ptr::null_mut();
                break;
            }
            if !(*(*jobj).parent).next.is_null() {
                jobj = (*(*jobj).parent).next;
                break;
            }
            jobj = (*jobj).parent;
        }
    }
    if i != (**crate::hooks::ft_parts_table().add((*fp).kind as usize)).parts_num {
        panic!("fighter parts num not match! player {}\n", (*fp).player_id);
    }
}

/// `ftParts_800749CC(gobj)`: initialise the fighter's part-visibility state (`x5AC`) for its costume.
pub unsafe fn ftParts_800749CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftParts_800749CC");
    let fp = (*gobj).user_data as *mut Fighter;
    // the visibility rows are indexed by the costume's visibility row (FT_COSTUME_VIS_IDX = Mex_CostumeVisIdx)
    crate::tu::ftparts_render::ftParts_8007487C(
        &mut (*(*(*fp).ft_data).x8).x0,
        &mut (*fp).x5AC,
        ext::Mex_CostumeVisIdx((*fp).kind as i32, (*fp).x619_costume_id as i32) as u32,
        &mut (*fp).dobj_list,
        &mut (*fp).x203C,
    );
    for i in 0..(*fp).x5AC.model_num as usize {
        (*fp).x5F4_arr[i].prev = -1;
    }
    crate::ft0df::ftparts_80074acc(gobj);
}

/// `ftParts_800750C8(fp, arg1, arg2)`: show (`arg2`) or hide model `arg1`, then the per-kind model event.
pub unsafe fn ftParts_800750C8(fp: *mut Fighter, arg1: u32, arg2: bool) {
    melee_core::ct!("ftParts_800750C8");
    let kind = (*fp).kind as usize;
    let vis: *mut FtPartsVis = &mut (*fp).x5AC;
    let ev = |model: u32| {
        if let Some(f) = crate::tu::ftdata::ftData_UnkIntBoolFunc0.model_events[kind] {
            f(fp, model as i32, arg2 as i32);
        }
    };
    if arg1 == 2 {
        if arg2 {
            ftParts_80074B6C(&*fp, &mut *vis, 2, &(*fp).x203C);
        } else {
            ftParts_80074D7C(&mut *vis, 2, &(*fp).x203C);
        }
        ev(2);
        if arg2 {
            ftParts_80074B6C(&*fp, &mut *vis, 3, &(*fp).dobj_list);
        } else {
            ftParts_80074D7C(&mut *vis, 3, &(*fp).dobj_list);
        }
        ev(3);
    } else {
        if arg2 {
            ftParts_80074B6C(&*fp, &mut *vis, arg1 as i32, &(*fp).dobj_list);
        } else {
            ftParts_80074D7C(&mut *vis, arg1 as i32, &(*fp).dobj_list);
        }
        ev(arg1);
    }
}

/// `ftParts_80075240(list, n)`: the n-th TObj over the DObj list's materials.
pub unsafe fn ftParts_80075240(list: *mut DObjList, n: i32) -> *mut HSD_TObj {
    melee_core::ct!("ftParts_80075240");
    let mut tobj_i = 0;
    for i in 0..(*list).count as usize {
        let dobj = *(*list).data.add(i);
        if !dobj.is_null() {
            let mobj = (*dobj).mobj;
            if !mobj.is_null() {
                let mut tobj = (*mobj).tobj;
                while !tobj.is_null() {
                    if tobj_i == n {
                        return tobj;
                    }
                    tobj_i += 1;
                    tobj = (*tobj).next;
                }
            }
        }
    }
    panic!("cannot find tobj!\n");
}

// ---- folded from ft0df.rs (consolidation steps 21/22) ----
// ---- ftparts.c leaves --------------------------------------------------------------------

pub fn ftparts_80074a4c(g: G, model_idx: i32, val: i32) {
    melee_core::ct!("ftparts_80074a4c");
    let f = fp(g);
    f.x5F4_arr[model_idx as usize].prev = val as i8;
    f.set_x221D_b2(1);
}

pub fn ftparts_80074a74(g: G, model_idx: i32) -> i32 { melee_core::ct!("ftparts_80074a74"); fp(g).x5F4_arr[model_idx as usize].prev as i32 }

pub fn ftparts_80074a8c(g: G) {
    melee_core::ct!("ftparts_80074a8c");
    let f = fp(g);
    let mut i = 0u32;
    while i < f.x5AC.model_num {
        f.x5F4_arr[i as usize].idx = f.x5F4_arr[i as usize].prev;
        i += 1;
    }
    f.set_x221D_b2(0);
}

pub fn ftparts_80074acc(g: G) {
    melee_core::ct!("ftparts_80074acc");
    let f = fp(g);
    let mut i = 0u32;
    while i < f.x5AC.model_num {
        f.x5F4_arr[i as usize].idx = -1;
        i += 1;
    }
    f.set_x221D_b2(0);
}

// ---- folded from ftdmgfall.rs (consolidation steps 21/22) ----
/// `ftParts_8007506C(kind, part)`: bit `i` of the first `Fighter_804D6540[kind]` entry whose
/// `x0 == part`, else 0.
pub fn ftparts_8007506c(ftkind: i32, part: i32) -> u32 {
    melee_core::ct!("ftparts_8007506c");
    unsafe {
        let t = *h::fighter_804d6540().offset(ftkind as isize);
        if !t.is_null() && (*t).x4 != 0 {
            let mut v = (*t).x0;
            let mut i = 0i32;
            while i < (*t).x4 {
                if (*v).x0 as i32 == part {
                    return 1u32.wrapping_shl(i as u32);
                }
                i += 1;
                v = v.add(1);
            }
        }
    }
    0
}
