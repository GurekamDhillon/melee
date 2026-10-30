//! src/melee/ft/ftparts.c, the model-class half (re-exported from `tu::ftparts`): the fighter
//! JObj / IntpJObj / PObj classes (`hsdChangeClass`ed onto every fighter model so a fighter's Z
//! scale reaches the matrices), their matrix setup, `ftParts_8007482C` (an interpolation-JObj
//! load), `ftParts_80075304` (graft a JObj into a bone tree) and the part setup / removal
//! (`ftParts_8007487C`, `ftParts_800753D4`, `ftParts_800755E8`, `ftParts_80075650`).
//!
//! The GX matrix loads are the hooks of `melee_runtime::tu::pobj::ext` (unset they panic with the
//! C name). No fused sites (fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::{c_char, c_void};
use core::mem::size_of;
use core::ptr::null_mut;
/// The pobj hooks plus the typed matrix wrappers and inlines melee_runtime now provides natively.
mod gx {
    pub use melee_runtime::tu::pobj::ext::*;
    pub use melee_runtime::tu::pobj::mx::{GXLoadNrmMtxImm, GXLoadPosMtxImm, GXLoadTexMtxImm, _HSD_mkEnvelopeModelNodeMtx};
    pub use melee_runtime::tu::pobj::HSD_PerfCountMtxLoad;
    pub use melee_runtime::tu::pobj::jobj_setup_matrix as HSD_JObjSetupMatrix;
}

type MP = *mut [f32; 4];
type M34 = [[f32; 4]; 3];

/// `ft_jobj_scale` (ftparts.static.h): the inverse-scale matrix for a fighter with a Z scale.
pub struct FtJObjScale {
    pub mtx: M34,
    pub has_z_scale: bool,
}
pub static mut ft_jobj_scale: FtJObjScale = FtJObjScale { mtx: [[0.0; 4]; 3], has_z_scale: false };

pub static mut ftJObj: HSD_JObjInfo = {
    let mut i: HSD_JObjInfo = unsafe { core::mem::zeroed() };
    i.parent.parent.head.info_init = Some(ftParts_JObjInfoInit);
    i
};
pub static mut ftIntpJObj: HSD_JObjInfo = {
    let mut i: HSD_JObjInfo = unsafe { core::mem::zeroed() };
    i.parent.parent.head.info_init = Some(ftParts_IntpJObjInfoInit);
    i
};
pub static mut ftPObj: HSD_PObjInfo = {
    let mut i: HSD_PObjInfo = unsafe { core::mem::zeroed() };
    i.parent.head.info_init = Some(ftParts_PObjInfoInit);
    i
};

unsafe fn mtx_at(p: MP) -> M34 {
    p.cast::<M34>().read()
}

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

/// `ftParts_80073758(jobj)`: make `jobj` a fighter JObj.
pub unsafe fn ftParts_80073758(jobj: *mut HSD_JObj) {
    melee_core::ct!("ftParts_80073758");
    melee_runtime::tu::class::hsdChangeClass(jobj.cast(), (&raw mut ftJObj).cast());
}

pub unsafe extern "C" fn ftParts_IntpJObjLoad(jobj: *mut HSD_JObj, joint: *mut HSD_Joint, parent: *mut HSD_JObj) -> i32 {
    melee_core::ct!("ftParts_IntpJObjLoad");
    let dobjdesc = (*joint).u.dobjdesc;
    (*joint).u.dobjdesc = null_mut();
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

const SETUP_NORMAL: u32 = 1;
const SETUP_REFLECTION: u32 = 2;
const SETUP_HIGHLIGHT: u32 = 4;
const SETUP_NORMAL_PROJECTION: u32 = 6;
const SETUP_JOINT0: u32 = 1;
const SETUP_JOINT1: u32 = 2;
const RENDER_SHADOW: u32 = 0x400_0000;
const HSD_MTX_RIGID: u32 = 1;
const HSD_MTX_ENVELOPE: u32 = 2;
const TEX_COORD_REFLECTION: u32 = 1;
const TEX_COORD_HILIGHT: u32 = 2;
const GX_TEXMTX0: u32 = 30;
const GX_TEXMTX1: u32 = 33;

unsafe fn ftPartsGetSetupFlags(jobj: *mut HSD_JObj, rendermode: u32) -> u32 {
    let mut flags = 0;
    if rendermode & RENDER_SHADOW == 0 {
        if (*jobj).flags & JOBJ_LIGHTING as u32 != 0 {
            flags |= SETUP_NORMAL;
        }
        if !gx::_HSD_TObjGetCurrentByType(null_mut(), TEX_COORD_REFLECTION).is_null() {
            flags |= SETUP_NORMAL | SETUP_REFLECTION;
        }
        if !gx::_HSD_TObjGetCurrentByType(null_mut(), TEX_COORD_HILIGHT).is_null() {
            flags |= SETUP_NORMAL | SETUP_HIGHLIGHT;
        }
    }
    flags
}

unsafe fn ftPartsSetupZScaleMtx(src: M34) -> M34 {
    if ft_jobj_scale.has_z_scale {
        let scale_mtx = melee_runtime::mtx::concat(&ft_jobj_scale.mtx, &src);
        melee_runtime::hsd_mtx::inverse_transpose(&scale_mtx)
    } else {
        melee_runtime::hsd_mtx::inverse_transpose(&src)
    }
}

unsafe fn ftPartsSetupNrmMtx(jobj: *mut HSD_JObj, mut mtx: M34, id: u32) {
    if (*jobj).flags & JOBJ_LIGHTING as u32 != 0 {
        gx::GXLoadNrmMtxImm(mtx.as_mut_ptr(), id);
        gx::HSD_PerfCountMtxLoad();
    }
}

unsafe fn ftPartsSetupTexMtx(mut mtx: M34, id: u32) {
    gx::GXLoadTexMtxImm(mtx.as_mut_ptr(), id, 0);
    gx::HSD_PerfCountMtxLoad();
}

pub unsafe extern "C" fn ftPartsSetupRigidMtx(_pobj: *mut HSD_PObj, _vmtx: MP, pmtx: MP, rendermode: u32) {
    melee_core::ct!("ftPartsSetupRigidMtx");
    let jobj = gx::HSD_JObjGetCurrent();
    let mut mark_jobj: *mut c_void = null_mut();
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
    let mut mark_jobj: *mut c_void = null_mut();
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
    melee_runtime::tu::pobj::HSD_PObjClearMtxMark(null_mut(), HSD_MTX_ENVELOPE);
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

pub unsafe fn ftPartsPObjSetDefaultClass() {
    melee_core::ct!("ftPartsPObjSetDefaultClass");
    melee_runtime::tu::pobj::HSD_PObjSetDefaultClass(&raw mut ftPObj);
}

pub unsafe fn ftPartsPObjClearDefaultClass() {
    melee_core::ct!("ftPartsPObjClearDefaultClass");
    melee_runtime::tu::pobj::HSD_PObjSetDefaultClass(null_mut());
}

/// Calls out of `ftParts_8007482C`: settable hooks (the class oracle's recording stand-ins); unset they run the ported callees.
pub mod ext {
    use super::*;
    pub static mut HOOK_SET_DEFAULT_CLASS: Option<unsafe fn(*mut c_void)> = None;
    pub static mut HOOK_LOAD_JOINT: Option<unsafe fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
    pub unsafe fn HSD_JObjSetDefaultClass(info: *mut c_void) {
        match HOOK_SET_DEFAULT_CLASS {
            Some(f) => f(info),
            None => melee_runtime::tu::jobj::HSD_JObjSetDefaultClass(info.cast()),
        }
    }
    pub unsafe fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
        match HOOK_LOAD_JOINT {
            Some(f) => f(joint),
            None => melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint),
        }
    }
}

/// `ftParts_8007482C(joint)`: load a joint with the interpolation JObj class as the default.
pub unsafe fn ftParts_8007482C(joint: *mut HSD_Joint) -> *mut HSD_JObj {
    melee_core::ct!("ftParts_8007482C");
    ext::HSD_JObjSetDefaultClass((&raw mut ftIntpJObj).cast());
    let jobj = ext::HSD_JObjLoadJoint(joint);
    ext::HSD_JObjSetDefaultClass(null_mut());
    jobj
}

/// `ftParts_80075304(type, root, new_jobj)`: graft `new_jobj` into the tree at `root`:
/// 0 child of root, taking root's children; 1 child alongside root's children; 2 sibling taking
/// root's siblings as children; 3 sibling alongside root's siblings.
pub unsafe fn ftParts_80075304(ty: u8, root: *mut HSD_JObj, new_jobj: *mut HSD_JObj) {
    melee_core::ct!("ftParts_80075304");
    match ty {
        0 => {
            let child = if root.is_null() { null_mut() } else { (*root).child };
            (*new_jobj).child = child;
            (*new_jobj).parent = root;
            (*root).child = new_jobj;
            if !child.is_null() {
                (*child).parent = new_jobj;
            }
        }
        1 => {
            let child = if root.is_null() { null_mut() } else { (*root).child };
            (*new_jobj).next = child;
            (*new_jobj).parent = root;
            (*root).child = new_jobj;
        }
        2 => {
            let next = if root.is_null() { null_mut() } else { (*root).next };
            (*new_jobj).child = next;
            (*new_jobj).parent = (*root).parent;
            (*root).next = new_jobj;
        }
        3 => {
            let next = if root.is_null() { null_mut() } else { (*root).next };
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
    let mut b = (*bone).anon_8.anon_0;
    b.set_flags_b1(1);
    b.set_flags_b7((jobj_flags & JOBJ_LIGHTING as u32 != 0) as u8);
    b.set_flags2_b0((jobj_flags & JOBJ_TEXGEN as u32 != 0) as u8);
    b.set_flags2_b1((jobj_flags & JOBJ_SPECULAR as u32 != 0) as u8);
    b.set_flags2_b2((jobj_flags & JOBJ_UNK_B18 as u32 != 0) as u8);
    b.set_flags2_b3((jobj_flags & JOBJ_UNK_B20 as u32 != 0) as u8);
    b.set_flags2_b4((jobj_flags & JOBJ_UNK_B19 as u32 != 0) as u8);
    let mut c = (*bone).anon_c.anon_0;
    c.xC = tree_depth as u8;
    while !dobj.is_null() {
        if *dobj_index >= 124 {
            panic!("ftparts.c:466 fighter parts model dobj num over! player {}", (*fighter).player_id);
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
        panic!("ftparts.c:480 fighter dobj num over! player {}", (*fighter).player_id);
    }
    c.set_xD(if *dobj_index != 0 { (*dobj_index - 1) as u8 } else { 0 });
    (*bone).anon_c.anon_0 = c;
    b.set_flags2_b6((dobj_count != 0) as u8);
    (*bone).anon_8.anon_0 = b;
}

/// `ftParts_8007487C(desc, vis, costume_id, arg3, arg4)`: set up a part-visibility table.
pub unsafe fn ftParts_8007487C(desc: *mut FtPartsDesc, vis: *mut FtPartsVis, costume_id: u32, arg3: *mut DObjList, arg4: *mut DObjList) {
    melee_core::ct!("ftParts_8007487C");
    let vis_table = (*desc).vis_table as *mut [*mut c_void; 4];
    (*vis).model_num = (*desc).model_num;
    if (*vis).model_num > 11 {
        panic!("ftparts.c:627 fighter parts model num over!");
    }
    let row = |c: u32, k: usize| -> *mut c_void {
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
    sp2c.next = null_mut();
    sp2c.child = null_mut();
    let temp_r30 = ftParts_8007482C(&mut sp2c);
    let temp_r31 = ftParts_8007482C(&mut sp2c);
    ftParts_80075304((*arg1).x2, (*(*fp).parts.add((*arg1).x1 as usize)).joint, temp_r30);
    ftParts_80075304((*arg1).x2, (*(*fp).parts.add((*arg1).x1 as usize)).x4_jobj2, temp_r31);
    let mut tree_depth = (*(*fp).parts.add((*arg1).x1 as usize)).anon_c.anon_0.xC as u32;
    if (*arg1).x2 == 0 || (*arg1).x2 == 1 {
        tree_depth += 1;
    }
    let mut dobj_index = 0i32;
    ftParts_80074194(fp, (*fp).parts.add((*arg1).x0 as usize), temp_r30, &mut dobj_index, tree_depth);
    (*(*fp).parts.add((*arg1).x0 as usize)).x4_jobj2 = temp_r31;
    let mut b = (*(*fp).parts.add((*arg1).x0 as usize)).anon_8.anon_0;
    b.set_flags_b2(1);
    (*(*fp).parts.add((*arg1).x0 as usize)).anon_8.anon_0 = b;
}

/// `ftParts_800755E8(fp, desc)`: remove a bone's model.
pub unsafe fn ftParts_800755E8(fp: *mut Fighter, arg1: *mut Fighter_804D6540_x0_t) {
    melee_core::ct!("ftParts_800755E8");
    let bone = (*fp).parts.add((*arg1).x0 as usize);
    melee_runtime::tu::jobj::HSD_JObjRemove_ptr((*bone).joint);
    melee_runtime::tu::jobj::HSD_JObjRemove_ptr((*bone).x4_jobj2);
    (*bone).joint = null_mut();
    (*bone).x4_jobj2 = null_mut();
    let mut b = (*bone).anon_8.anon_0;
    b.set_flags_b1(0);
    b.set_flags_b2(0);
    (*bone).anon_8.anon_0 = b;
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
                    jobj = null_mut();
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
