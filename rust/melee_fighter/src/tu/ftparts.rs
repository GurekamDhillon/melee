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
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use core::ffi::c_char;
#[allow(unused_imports)]
use core::ffi::c_void as c_void_ftparts_render;
#[allow(unused_imports)]
use core::mem::size_of;
#[allow(unused_imports)]
use core::ptr::null_mut as null_mut_ftparts_render;
#[allow(unused_imports)]
use crate::tu::ftparts_render::{GX_TEXMTX0, GX_TEXMTX1, HSD_MTX_ENVELOPE, HSD_MTX_RIGID, M34, MP, RENDER_SHADOW, SETUP_HIGHLIGHT, SETUP_JOINT0, SETUP_JOINT1, SETUP_NORMAL, SETUP_NORMAL_PROJECTION, SETUP_REFLECTION, TEX_COORD_HILIGHT, TEX_COORD_REFLECTION, ext as ext_ftparts_render, ftIntpJObj, ftJObj, ftPObj, ft_jobj_scale, gx, mtx_at};
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
        b.x8.flags8 = 0;
        b.xC_u.flagsC = 0;
    }
    unsafe {
        bone(f, 0).x8.x0.set_flags_b3(1);
        for p in [FtPart_TransN, FtPart_XRotN, FtPart_YRotN, FtPart_HipN, FtPart_TransN2] {
            bone(f, ftparts_get_bone_index(f, p as i32)).x8.x0.set_flags_b3(1);
        }
        bone(f, ftparts_get_bone_index(f, FtPart_TransN as i32)).x8.x0.set_flags_b4(1);
        bone(f, ftparts_get_bone_index(f, 0x35)).x8.x0.set_flags_b4(1);
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
        panic!("fighter parts num over! player {}\n", (*fp).player_idx);
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
        panic!("fighter parts num not match! player {}\n", (*fp).player_idx);
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
        panic!("fighter parts num not match! player {}\n", (*fp).player_idx);
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
        ext::Mex_CostumeVisIdx((*fp).kind as i32, (*fp).costume_id as i32) as u32,
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
    // a kind past the table (an m-ex guest left fp->kind spoofed: MexCPU_ProcSpoof swaps it around the CPU calls) has
    // no model event; logged once instead of indexing past the table
    let ev = |model: u32| {
        match crate::tu::ftdata::ftData_UnkIntBoolFunc0.model_events.get(kind) {
            Some(Some(f)) => f(fp, model as i32, arg2 as i32),
            Some(None) => {}
            None => {
                static SEEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !SEEN.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    eprintln!("ftParts_800750C8: fighter kind {kind:#x} past the model-event table (fp {fp:p}): event skipped");
                }
            }
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

// ---- folded from ftparts_render.rs (consolidation steps 21/22) ----
/// `ftParts_JObjMakePositionMtx(jobj, mtx, rmtx)`.
pub unsafe extern "C" fn ftParts_JObjMakePositionMtx(jobj: *mut HSD_JObj, mtx: MP, rmtx: MP) {
    melee_core::ct!("ftParts_JObjMakePositionMtx");
    let fighter = (*gx::HSD_GObj_804D7814).user_data as *mut Fighter;
    (melee_runtime::tu::jobj::hsdJObj.make_pmtx.unwrap())(jobj, mtx, rmtx);
    if (*fighter).x34_scale.z != 1.0 {
        let scale_z = (*jobj).scale.z;
        let scale_y = (*jobj).scale.y;
        let scale_x = (*jobj).scale.x;
        let temp = melee_runtime::mtx::scale(1.0 / scale_x, 1.0 / scale_y, 1.0 / scale_z);
        let r = mtx_at(rmtx);
        let m = melee_runtime::mtx::concat(&r, &temp);
        let inv = melee_runtime::hsd_mtx::inverse(&r);
        ft_jobj_scale.mtx = melee_runtime::mtx::concat(&m, &inv);
        ft_jobj_scale.has_z_scale = true;
    } else {
        ft_jobj_scale.has_z_scale = false;
    }
}

pub unsafe extern "C" fn ftParts_JObjInfoInit() {
    melee_core::ct!("ftParts_JObjInfoInit");
    melee_runtime::tu::class::hsdInitClassInfo(
        (&raw mut ftJObj).cast(),
        (&raw mut melee_runtime::tu::jobj::hsdJObj).cast(),
        c"fighter_class_library".as_ptr() as *mut c_char,
        c"ft_jobj".as_ptr() as *mut c_char,
        size_of::<HSD_JObjInfo>() as i32,
        size_of::<HSD_JObj>() as i32,
    );
    ftJObj.make_pmtx = Some(ftParts_JObjMakePositionMtx);
}

pub unsafe extern "C" fn ftParts_IntpJObjLoad(jobj: *mut HSD_JObj, joint: *mut HSD_Joint, parent: *mut HSD_JObj) -> i32 {
    melee_core::ct!("ftParts_IntpJObjLoad");
    let dobjdesc = (*joint).u.dobjdesc;
    (*joint).u.dobjdesc = null_mut_ftparts_render();
    let ret = (melee_runtime::tu::jobj::hsdJObj.load.unwrap())(jobj, joint, parent);
    (*joint).u.dobjdesc = dobjdesc;
    ret
}

pub unsafe extern "C" fn ftParts_IntpJObjInfoInit() {
    melee_core::ct!("ftParts_IntpJObjInfoInit");
    melee_runtime::tu::class::hsdInitClassInfo(
        (&raw mut ftIntpJObj).cast(),
        (&raw mut melee_runtime::tu::jobj::hsdJObj).cast(),
        c"fighter_class_library".as_ptr() as *mut c_char,
        c"ft_intp_jobj".as_ptr() as *mut c_char,
        size_of::<HSD_JObjInfo>() as i32,
        size_of::<HSD_JObj>() as i32,
    );
    ftIntpJObj.load = Some(ftParts_IntpJObjLoad);
}

pub unsafe fn ftPartsGetSetupFlags(jobj: *mut HSD_JObj, rendermode: u32) -> u32 {
    let mut flags = 0;
    if rendermode & RENDER_SHADOW == 0 {
        if (*jobj).flags & JOBJ_LIGHTING as u32 != 0 {
            flags |= SETUP_NORMAL;
        }
        if !gx::_HSD_TObjGetCurrentByType(null_mut_ftparts_render(), TEX_COORD_REFLECTION).is_null() {
            flags |= SETUP_NORMAL | SETUP_REFLECTION;
        }
        if !gx::_HSD_TObjGetCurrentByType(null_mut_ftparts_render(), TEX_COORD_HILIGHT).is_null() {
            flags |= SETUP_NORMAL | SETUP_HIGHLIGHT;
        }
    }
    flags
}

pub unsafe fn ftPartsSetupZScaleMtx(src: M34) -> M34 {
    if ft_jobj_scale.has_z_scale {
        let scale_mtx = melee_runtime::mtx::concat(&ft_jobj_scale.mtx, &src);
        melee_runtime::hsd_mtx::inverse_transpose(&scale_mtx)
    } else {
        melee_runtime::hsd_mtx::inverse_transpose(&src)
    }
}

pub unsafe fn ftPartsSetupNrmMtx(jobj: *mut HSD_JObj, mut mtx: M34, id: u32) {
    if (*jobj).flags & JOBJ_LIGHTING as u32 != 0 {
        gx::GXLoadNrmMtxImm(mtx.as_mut_ptr(), id);
        gx::HSD_PerfCountMtxLoad();
    }
}

pub unsafe fn ftPartsSetupTexMtx(mut mtx: M34, id: u32) {
    gx::GXLoadTexMtxImm(mtx.as_mut_ptr(), id, 0);
    gx::HSD_PerfCountMtxLoad();
}

pub unsafe extern "C" fn ftPartsSetupRigidMtx(_pobj: *mut HSD_PObj, _vmtx: MP, pmtx: MP, rendermode: u32) {
    melee_core::ct!("ftPartsSetupRigidMtx");
    let jobj = gx::HSD_JObjGetCurrent();
    let mut mark_jobj: *mut c_void_ftparts_render = null_mut_ftparts_render();
    let mut mark: u32 = 0;
    melee_runtime::tu::pobj::HSD_PObjGetMtxMark(0, &mut mark_jobj, &mut mark);
    if mark_jobj != jobj.cast() || mark != HSD_MTX_RIGID {
        melee_runtime::tu::pobj::HSD_PObjSetMtxMark(0, jobj.cast(), HSD_MTX_RIGID);
        gx::GXSetCurrentMtx(0);
        gx::GXLoadPosMtxImm(pmtx, 0);
        gx::HSD_PerfCountMtxLoad();
        let flags = ftPartsGetSetupFlags(jobj, rendermode);
        if flags & SETUP_NORMAL != 0 {
            let mtx = ftPartsSetupZScaleMtx(mtx_at(pmtx));
            ftPartsSetupNrmMtx(jobj, mtx, 0);
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                ftPartsSetupTexMtx(mtx, GX_TEXMTX0);
            }
        }
    }
}

pub unsafe extern "C" fn ftPartsSetupSharedVtxMtx(pobj: *mut HSD_PObj, vmtx: MP, pmtx: MP, rendermode: u32) {
    melee_core::ct!("ftPartsSetupSharedVtxMtx");
    let mut flags = 0u32;
    let jobj = gx::HSD_JObjGetCurrent();
    let mut mark_jobj: *mut c_void_ftparts_render = null_mut_ftparts_render();
    let mut mark: u32 = 0;
    melee_runtime::tu::pobj::HSD_PObjGetMtxMark(0, &mut mark_jobj, &mut mark);
    if mark_jobj != jobj.cast() && mark != HSD_MTX_RIGID {
        flags |= SETUP_JOINT0;
    }
    melee_runtime::tu::pobj::HSD_PObjSetMtxMark(0, jobj.cast(), HSD_MTX_RIGID);
    melee_runtime::tu::pobj::HSD_PObjGetMtxMark(1, &mut mark_jobj, &mut mark);
    if mark_jobj != (*pobj).u.jobj.cast() && mark != HSD_MTX_RIGID {
        flags |= SETUP_JOINT1;
    }
    melee_runtime::tu::pobj::HSD_PObjSetMtxMark(1, (*pobj).u.jobj.cast(), HSD_MTX_RIGID);
    if flags == 0 {
        return;
    }
    flags |= ftPartsGetSetupFlags(jobj, rendermode);
    {
        gx::GXSetCurrentMtx(0);
        gx::GXLoadPosMtxImm(pmtx, 0);
        gx::HSD_PerfCountMtxLoad();
        if flags & SETUP_NORMAL != 0 {
            let mtx0 = ftPartsSetupZScaleMtx(mtx_at(pmtx));
            ftPartsSetupNrmMtx(jobj, mtx0, 0);
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                ftPartsSetupTexMtx(mtx0, GX_TEXMTX0);
            }
        }
    }
    {
        gx::HSD_JObjSetupMatrix((*pobj).u.jobj);
        let mut tmp = melee_runtime::mtx::concat(&mtx_at(vmtx), &(*(*pobj).u.jobj).mtx);
        gx::GXLoadPosMtxImm(tmp.as_mut_ptr(), 3);
        gx::HSD_PerfCountMtxLoad();
        if flags & SETUP_NORMAL != 0 {
            let mtx1 = ftPartsSetupZScaleMtx(tmp);
            ftPartsSetupNrmMtx(jobj, mtx1, 3);
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                ftPartsSetupTexMtx(mtx1, GX_TEXMTX1);
            }
        }
    }
}

pub unsafe extern "C" fn ftPartsSetupEnvelopeMtx(pobj: *mut HSD_PObj, vmtx: MP, _pmtx: MP, rendermode: u32) {
    melee_core::ct!("ftPartsSetupEnvelopeMtx");
    let jobj = gx::HSD_JObjGetCurrent();
    melee_runtime::tu::pobj::HSD_PObjClearMtxMark(null_mut_ftparts_render(), HSD_MTX_ENVELOPE);
    let flags = ftPartsGetSetupFlags(jobj, rendermode);
    let mut sp_ac = [[0.0f32; 4]; 3];
    let node_mtxp = gx::_HSD_mkEnvelopeModelNodeMtx(jobj, sp_ac.as_mut_ptr());
    let mut envelope_list = (*pobj).u.envelope_list;
    let mut i = 0u32;
    while i < 10 && !envelope_list.is_null() {
        let mut envelope = (*envelope_list).data as *mut HSD_Envelope;
        let mtx_id = 3 * i; // HSD_Index2PosNrmMtx(i)
        let mut envelope_count = 0;
        assert!(!envelope.is_null(), "ftparts.c:328 envelope");
        let mtxp: M34;
        if (*envelope).weight >= 1.0 {
            gx::HSD_JObjSetupMatrix((*envelope).jobj);
            if !node_mtxp.is_null() {
                mtxp = melee_runtime::mtx::concat(&(*(*envelope).jobj).mtx, &mtx_at((*(*envelope).jobj).envelopemtx));
            } else {
                mtxp = (*(*envelope).jobj).mtx;
            }
        } else {
            let mut mtx = [[0.0f32; 4]; 3];
            while !envelope.is_null() {
                let jp = (*envelope).jobj;
                assert!(!jp.is_null(), "ftparts.c:348 envelope->jobj");
                gx::HSD_JObjSetupMatrix(jp);
                assert!(!(*jp).envelopemtx.is_null(), "ftparts.c:352 jp->envelopemtx");
                let tmp = melee_runtime::mtx::concat(&(*jp).mtx, &mtx_at((*jp).envelopemtx));
                mtx = melee_runtime::hsd_mtx::scaled_add(&tmp, &mtx, (*envelope).weight);
                envelope = (*envelope).next;
                envelope_count += 1;
            }
            mtxp = mtx;
        }
        gx::HSD_PerfCountEnvelopeBlending(envelope_count);
        let mtxp = if !node_mtxp.is_null() { melee_runtime::mtx::concat(&mtxp, &mtx_at(node_mtxp)) } else { mtxp };
        let mut tmp = melee_runtime::mtx::concat(&mtx_at(vmtx), &mtxp);
        gx::GXLoadPosMtxImm(tmp.as_mut_ptr(), mtx_id);
        gx::HSD_PerfCountMtxLoad();
        if flags & SETUP_NORMAL != 0 {
            let mtx = ftPartsSetupZScaleMtx(tmp);
            ftPartsSetupNrmMtx(jobj, mtx, mtx_id);
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                ftPartsSetupTexMtx(mtx, GX_TEXMTX0 + 3 * i); // HSD_Index2TexMtx(i)
            }
        }
        envelope_list = (*envelope_list).next;
        i += 1;
    }
}

pub unsafe extern "C" fn ftParts_PObjSetupMtx(pobj: *mut HSD_PObj, vmtx: MP, pmtx: MP, rendermode: u32) {
    melee_core::ct!("ftParts_PObjSetupMtx");
    if !ft_jobj_scale.has_z_scale {
        let m = melee_runtime::tu::pobj::hsdPObj.setup_mtx.unwrap();
        m(pobj, vmtx, pmtx, rendermode);
        return;
    }
    // `pobj_type(pobj)` is `flags & POBJ_TYPE_MASK`
    match (*pobj).flags & 0x3000 {
        0 => {
            if (*pobj).u.jobj.is_null() {
                ftPartsSetupRigidMtx(pobj, vmtx, pmtx, rendermode);
            } else {
                ftPartsSetupSharedVtxMtx(pobj, vmtx, pmtx, rendermode);
            }
        }
        0x1000 => ftPartsSetupRigidMtx(pobj, vmtx, pmtx, rendermode),
        0x2000 => ftPartsSetupEnvelopeMtx(pobj, vmtx, pmtx, rendermode),
        _ => {}
    }
}

pub unsafe extern "C" fn ftParts_PObjInfoInit() {
    melee_core::ct!("ftParts_PObjInfoInit");
    melee_runtime::tu::class::hsdInitClassInfo(
        (&raw mut ftPObj).cast(),
        (&raw mut melee_runtime::tu::pobj::hsdPObj).cast(),
        c"fighter_class_library".as_ptr() as *mut c_char,
        c"ft_pobj".as_ptr() as *mut c_char,
        size_of::<HSD_PObjInfo>() as i32,
        size_of::<HSD_PObj>() as i32,
    );
    ftPObj.load = Some(melee_runtime::tu::lbrefract::lbRefract_PObjLoad);
    ftPObj.setup_mtx = Some(ftParts_PObjSetupMtx);
}

/// `ftParts_8007482C(joint)`: load a joint with the interpolation JObj class as the default.
pub unsafe fn ftParts_8007482C(joint: *mut HSD_Joint) -> *mut HSD_JObj {
    melee_core::ct!("ftParts_8007482C");
    ext_ftparts_render::HSD_JObjSetDefaultClass((&raw mut ftIntpJObj).cast());
    let jobj = ext_ftparts_render::HSD_JObjLoadJoint(joint);
    ext_ftparts_render::HSD_JObjSetDefaultClass(null_mut_ftparts_render());
    jobj
}

/// `ftParts_80075304(type, root, new_jobj)`: graft `new_jobj` into the tree at `root`:
/// 0 child of root, taking root's children; 1 child alongside root's children; 2 sibling taking
/// root's siblings as children; 3 sibling alongside root's siblings.
pub unsafe fn ftParts_80075304(ty: u8, root: *mut HSD_JObj, new_jobj: *mut HSD_JObj) {
    melee_core::ct!("ftParts_80075304");
    match ty {
        0 => {
            let child = if root.is_null() { null_mut_ftparts_render() } else { (*root).child };
            (*new_jobj).child = child;
            (*new_jobj).parent = root;
            (*root).child = new_jobj;
            if !child.is_null() {
                (*child).parent = new_jobj;
            }
        }
        1 => {
            let child = if root.is_null() { null_mut_ftparts_render() } else { (*root).child };
            (*new_jobj).next = child;
            (*new_jobj).parent = root;
            (*root).child = new_jobj;
        }
        2 => {
            let next = if root.is_null() { null_mut_ftparts_render() } else { (*root).next };
            (*new_jobj).child = next;
            (*new_jobj).parent = (*root).parent;
            (*root).next = new_jobj;
        }
        3 => {
            let next = if root.is_null() { null_mut_ftparts_render() } else { (*root).next };
            (*new_jobj).next = next;
            (*new_jobj).parent = (*root).parent;
            (*root).next = new_jobj;
        }
        _ => {}
    }
}

/// `ftParts_80074194(fighter, bone, jobj, &dobj_index, tree_depth)`: register a bone's JObj and
/// its DObjs in the fighter's DObj list.
pub unsafe fn ftParts_80074194(fighter: *mut Fighter, bone: *mut FighterBone, jobj: *mut HSD_JObj, dobj_index: *mut i32, tree_depth: u32) {
    melee_core::ct!("ftParts_80074194");
    let mut dobj = melee_runtime::tu::jobj_ptr::HSD_JObjGetDObj(jobj);
    let jobj_flags = (*jobj).flags;
    let mut dobj_count = 0;
    (*bone).joint = jobj;
    let mut b = (*bone).x8.x0;
    b.set_flags_b1(1);
    b.set_flags_b7((jobj_flags & JOBJ_LIGHTING as u32 != 0) as u8);
    b.set_flags2_b0((jobj_flags & JOBJ_TEXGEN as u32 != 0) as u8);
    b.set_flags2_b1((jobj_flags & JOBJ_SPECULAR as u32 != 0) as u8);
    b.set_flags2_b2((jobj_flags & JOBJ_UNK_B18 as u32 != 0) as u8);
    b.set_flags2_b3((jobj_flags & JOBJ_UNK_B20 as u32 != 0) as u8);
    b.set_flags2_b4((jobj_flags & JOBJ_UNK_B19 as u32 != 0) as u8);
    let mut c = (*bone).xC_u.x0;
    c.xC = tree_depth as u8;
    while !dobj.is_null() {
        if *dobj_index >= 124 {
            panic!("ftparts.c:466 fighter parts model dobj num over! player {}", (*fighter).player_idx);
        }
        *(*fighter).dobj_list.data.add(*dobj_index as usize) = dobj;
        let mobj = (*dobj).mobj;
        if !mobj.is_null() {
            melee_runtime::tu::class::hsdChangeClass(mobj.cast(), crate::tu::ftmaterial::ftMObj_ptr().cast());
        }
        dobj = (*dobj).next;
        dobj_count += 1;
        *dobj_index += 1;
    }
    if dobj_count >= 128 {
        panic!("ftparts.c:480 fighter dobj num over! player {}", (*fighter).player_idx);
    }
    c.set_xD(if *dobj_index != 0 { (*dobj_index - 1) as u8 } else { 0 });
    (*bone).xC_u.x0 = c;
    b.set_flags2_b6((dobj_count != 0) as u8);
    (*bone).x8.x0 = b;
}

/// `ftParts_8007487C(desc, vis, costume_id, arg3, arg4)`: set up a part-visibility table.
pub unsafe fn ftParts_8007487C(desc: *mut FtPartsDesc, vis: *mut FtPartsVis, costume_id: u32, arg3: *mut DObjList, arg4: *mut DObjList) {
    melee_core::ct!("ftParts_8007487C");
    let vis_table = (*desc).vis_table as *mut [*mut c_void_ftparts_render; 4];
    (*vis).model_num = (*desc).model_num;
    if (*vis).model_num > 11 {
        panic!("ftparts.c:627 fighter parts model num over!");
    }
    let row = |c: u32, k: usize| -> *mut c_void_ftparts_render {
        let v = (*vis_table.add(c as usize))[k];
        if !v.is_null() { v } else { (*vis_table)[k] }
    };
    for k in 0..4 {
        (*vis).xC[k] = row(costume_id, k) as _;
    }
    (*vis).xC[4] = core::ptr::null_mut();
    for k in 0..5 {
        (*vis).cleared[k] = 1;
    }
    super::ftparts::ftParts_80074D7C(&mut *vis, 0, arg3);
    super::ftparts::ftParts_80074D7C(&mut *vis, 1, arg3);
    super::ftparts::ftParts_80074D7C(&mut *vis, 2, arg4);
    super::ftparts::ftParts_80074D7C(&mut *vis, 3, arg3);
    super::ftparts::ftParts_80074D7C(&mut *vis, 4, arg3);
}

/// `ftParts_800753D4(fp, desc, joint)`: replace a bone's model with a copy of a joint of `joint`'s tree.
pub unsafe fn ftParts_800753D4(fp: *mut Fighter, arg1: *mut Fighter_804D6540_x0_t, arg2: *mut HSD_Joint) {
    melee_core::ct!("ftParts_800753D4");
    let mut sp6c = arg2;
    if (*arg1).x3 != 0xFF {
        let mut depth = 0i32;
        for _ in 0..(*arg1).x3 {
            super::ftanim::ftAnim_GetNextJointInTree(&mut sp6c, &mut depth);
        }
    }
    let mut sp2c: HSD_Joint = *sp6c;
    sp2c.next = null_mut_ftparts_render();
    sp2c.child = null_mut_ftparts_render();
    let temp_r30 = ftParts_8007482C(&mut sp2c);
    let temp_r31 = ftParts_8007482C(&mut sp2c);
    ftParts_80075304((*arg1).x2, (*(*fp).parts.add((*arg1).x1 as usize)).joint, temp_r30);
    ftParts_80075304((*arg1).x2, (*(*fp).parts.add((*arg1).x1 as usize)).x4_jobj2, temp_r31);
    let mut tree_depth = (*(*fp).parts.add((*arg1).x1 as usize)).xC_u.x0.xC as u32;
    if (*arg1).x2 == 0 || (*arg1).x2 == 1 {
        tree_depth += 1;
    }
    let mut dobj_index = 0i32;
    ftParts_80074194(fp, (*fp).parts.add((*arg1).x0 as usize), temp_r30, &mut dobj_index, tree_depth);
    (*(*fp).parts.add((*arg1).x0 as usize)).x4_jobj2 = temp_r31;
    let mut b = (*(*fp).parts.add((*arg1).x0 as usize)).x8.x0;
    b.set_flags_b2(1);
    (*(*fp).parts.add((*arg1).x0 as usize)).x8.x0 = b;
}

/// `ftParts_800755E8(fp, desc)`: remove a bone's model.
pub unsafe fn ftParts_800755E8(fp: *mut Fighter, arg1: *mut Fighter_804D6540_x0_t) {
    melee_core::ct!("ftParts_800755E8");
    let bone = (*fp).parts.add((*arg1).x0 as usize);
    melee_runtime::tu::jobj::HSD_JObjRemove_ptr((*bone).joint);
    melee_runtime::tu::jobj::HSD_JObjRemove_ptr((*bone).x4_jobj2);
    (*bone).joint = null_mut_ftparts_render();
    (*bone).x4_jobj2 = null_mut_ftparts_render();
    let mut b = (*bone).x8.x0;
    b.set_flags_b1(0);
    b.set_flags_b2(0);
    (*bone).x8.x0 = b;
}

/// `ftParts_80075650(gobj, jobj, list)`: collect every DObj of a JObj tree into `list` (making
/// their MObjs fighter MObjs).
pub unsafe fn ftParts_80075650(_gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj, arg2: *mut DObjList) {
    melee_core::ct!("ftParts_80075650");
    let mut var_r30 = 0usize;
    while !jobj.is_null() {
        let mut dobj = melee_runtime::tu::jobj_ptr::HSD_JObjGetDObj(jobj);
        while !dobj.is_null() {
            if var_r30 >= 0x20 {
                panic!("ftparts.c:1063 fighter parts model dobj num over!");
            }
            *(*arg2).data.add(var_r30) = dobj;
            let mobj = (*dobj).mobj;
            if !mobj.is_null() {
                melee_runtime::tu::class::hsdChangeClass(mobj.cast(), crate::tu::ftmaterial::ftMObj_ptr().cast());
            }
            dobj = (*dobj).next;
            var_r30 += 1;
        }
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !(*jobj).child.is_null() {
            jobj = (*jobj).child;
        } else if !(*jobj).next.is_null() {
            jobj = (*jobj).next;
        } else {
            loop {
                if (*jobj).parent.is_null() {
                    jobj = null_mut_ftparts_render();
                    break;
                }
                if !(*(*jobj).parent).next.is_null() {
                    jobj = (*(*jobj).parent).next;
                    break;
                }
                jobj = (*jobj).parent;
            }
        }
    }
}
