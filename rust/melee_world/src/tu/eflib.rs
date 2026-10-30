//! Native port of src/melee/ef/eflib.c (effect objects, particle generator helpers, effect
//! update callbacks), TARGET_PC variant.
//!
//! The TU's own globals (`efLib_AnimQueue`, `efLib_ParamTable`) are module statics; efdata.c's
//! globals live in [`EF_DATA`] (efdata.rs defines their record, not an instance). The
//! `efSync_*`-style va_list entry points take their varargs as explicit parameters.
//! The three plink walkers already ported in `crate::ef::eflib` are reused.
//! Float math: only plain mul/add/sub (no `a*b+c` in one expression), `sinf`/`cosf`/`atan2f`
//! are MSL's (melee_runtime). Calls into baselib/lb/gr/ft TUs that are not ported go through
//! `ext` (panicking stubs named as the C functions).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::efdata::EfData;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::*;
use melee_types::inline::*;

/// `M_PI_2` (f64), as the C writes `-M_PI_2 : M_PI_2` then stores to f32.
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
const M_PI_2_F: f32 = M_PI_2 as f32;

// TARGET_PC particle link masks.
const PTCL_SKIP_LINKNO_0: u32 = 0x10000;
const PTCL_SKIP_LINKNO_1: u32 = 0x20000;
const PTCL_SKIP_LINKNO_2: u32 = 0x40000;
const PTCL_STAGE_SKIP: u32 = 0x80000;
const PTCL_STAGE_RENDER: u32 = 0x8;
const PTCL_RENDER_LINKNO_0: u32 = 0x1;
const PTCL_RENDER_LINKNO_1: u32 = 0x2;
const PTCL_RENDER_LINKNO_2: u32 = 0x4;

/// `efLib_AnimQueue` (0x80458EE0): used as `HSD_JObj*[32]`.
pub static mut efLib_AnimQueue: [*mut HSD_JObj; 0x20] = [core::ptr::null_mut(); 0x20];
/// `efLib_ParamTable` (0x80458F60).
pub static mut efLib_ParamTable: [EF_ParamEntry; 8] =
    [EF_ParamEntry { gobj: core::ptr::null_mut(), gfx_id: 0, alpha: 0 }; 8];
/// efdata.c's globals (`efLib_AllocData`, `efLib_AnimCount`, `efLib_EffectCount`, `efLib_LoadKind`).
pub static mut EF_DATA: EfData = EfData {
    alloc_data: unsafe { core::mem::zeroed() },
    anim_count: 0,
    effect_count: 0,
    load_kind: 0,
};

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use melee_types::*;
    pub type G = *mut HSD_GObj;
    pub type J = *mut HSD_JObj;
    pub type Gen = *mut HSD_Generator;
    pub fn HSD_GObjPLinkHead(_i: usize) -> G { unsafe { *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(_i) } }
    pub fn HSD_GObj_JObjKind() -> u8 { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_JObjKind } }
    pub fn efAsync_DatEntries_data(_bank: i32) -> *mut core::ffi::c_void { todo!("efAsync_DatEntries (ef/efasync.c)") }
    pub fn efAsync_DatEntries_clear() { todo!("efAsync_DatEntries (ef/efasync.c)") }
    pub fn efAsync_QueueInit() { todo!("efAsync_QueueInit (ef/efasync.c)") }
    pub fn HSD_ObjAllocInit(_d: *mut HSD_ObjAllocData, _size: u32, _align: u32) { unsafe { let _ = melee_runtime::tu::objalloc::HSD_ObjAllocInit(_d as _, _size as _, _align as _); } }
    pub fn HSD_ObjAlloc(_d: *mut HSD_ObjAllocData) -> *mut core::ffi::c_void { unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(_d as _) as _ } }
    pub fn HSD_ObjFree(_d: *mut HSD_ObjAllocData, _p: *mut core::ffi::c_void) { unsafe { let _ = melee_runtime::tu::objalloc::HSD_ObjFree(_d as _, _p as _); } }
    pub fn hsd_8039D354(_a: i32) { todo!("hsd_8039D354 (baselib/particle.c)") }
    pub fn hsd_80398A08(_a: i32) { unsafe { let _ = melee_runtime::tu::particle::hsd_80398A08(_a as _); } }
    pub fn set_hsd_804D7900(_f: unsafe extern "C" fn(Gen)) { todo!("hsd_804D7900 (baselib/particle.c)") }
    pub fn GObj_Create(_class: u16, _p_link: u8, _prio: u8) -> G { unsafe { melee_runtime::tu::gobj_native::GObj_Create(_class as _, _p_link as _, _prio as _) as _ } }
    pub fn GObj_SetupGXLink(_g: G, _f: unsafe extern "C" fn(G, i32), _link: u8, _prio: u32) { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(_g as _, Some(_f), _link as _, _prio as _); } }
    pub fn GObj_SetupGXLink_JObjCallback(_g: G, _link: u8, _prio: u32) { todo!("GObj_SetupGXLink(HSD_GObj_JObjCallback) (baselib/gobjgxlink.c)") }
    pub fn HSD_GObj_SetupProc(_g: G, _f: unsafe extern "C" fn(G), _prio: u8) { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(_g as _, Some(_f), _prio as _); } }
    pub fn GObj_InitUserData(_g: G, _kind: u8, _free: unsafe extern "C" fn(*mut core::ffi::c_void), _ud: *mut core::ffi::c_void) { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(_g as _, _kind as _, Some(_free), _ud as _); } }
    pub fn HSD_GObjObject_80390A70(_g: G, _kind: u8, _obj: *mut core::ffi::c_void) { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(_g as _, _kind as _, _obj as _); } }
    pub fn HSD_GObjFree(_g: G) { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(_g as _); } }
    pub fn HSD_JObjSetSPtclCallback(_f: unsafe extern "C" fn(i32, i32, i32, J)) { todo!("HSD_JObjSetSPtclCallback (baselib/jobj.c)") }
    pub fn HSD_JObjSetDPtclCallback(_f: unsafe extern "C" fn(i32, i32, i32, J)) { todo!("HSD_JObjSetDPtclCallback (baselib/jobj.c)") }
    pub fn psInitAppSRT(_a: i32, _b: i32) { todo!("psInitAppSRT (baselib/psappsrt.c)") }
    pub fn HSD_JObjWalkTree_8039D688(_j: J) { todo!("HSD_JObjWalkTree(jobj, hsd_8039D688, NULL) (baselib/jobj.c)") }
    pub fn HSD_JObjSetMtxDirtySub(_j: J) { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(_j as _); } }
    pub fn HSD_JObjSetupMatrixSub(_j: J) { todo!("HSD_JObjSetupMatrixSub (baselib/jobj.c)") }
    pub fn HSD_MtxGetScale(_m: *mut Mtx, _s: *mut Vec3) { unsafe { let _ = melee_runtime::tu::mtx_world::HSD_MtxGetScale(_m as _, _s as _); } }
    pub fn HSD_JObjAnimAll(_j: J) { todo!("HSD_JObjAnimAll (baselib/jobj.c)") }
    pub fn HSD_JObjLoadJoint(_j: *mut HSD_Joint) -> J { unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(_j as _) as _ } }
    pub fn HSD_JObjAddAnimAll(_j: J, _a: *mut HSD_AnimJoint, _m: *mut HSD_MatAnimJoint, _s: *mut HSD_ShapeAnimJoint) { todo!("HSD_JObjAddAnimAll (baselib/jobj.c)") }
    pub fn HSD_JObjReqAnimAll(_j: J, _f: f32) { todo!("HSD_JObjReqAnimAll (baselib/jobj.c)") }
    pub fn HSD_JObjSetFlagsAll(_j: J, _f: u32) { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(_j as _, _f as _); } }
    pub fn HSD_JObjClearFlagsAll(_j: J, _f: u32) { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(_j as _, _f as _); } }
    pub fn HSD_JObjGetDObj(_j: J) -> *mut HSD_DObj { unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetDObj(_j as _) as _ } }
    pub fn HSD_MObjGetTObj(_m: *mut HSD_MObj) -> *mut HSD_TObj { unsafe { melee_runtime::tu::mobj::HSD_MObjGetTObj(_m as _) as _ } }
    pub fn lb_80011C18(_j: J, _f: u32) { unsafe { let _ = melee_runtime::tu::lbspdisplay::lb_80011C18(_j as _, _f as _); } }
    pub fn lb_8000C1C0(_a: J, _b: J) { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C1C0(_a as _, _b as _); } }
    pub fn lb_8000C290(_a: J, _b: J) { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C290(_a as _, _b as _); } }
    pub fn lb_8000B1CC(_j: J, _off: *const Vec3, _out: *mut Vec3) { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(_j as _, _off as _, _out as _); } }
    pub fn HSD_StateSetColorUpdate(_a: i32) { todo!("HSD_StateSetColorUpdate (baselib/state.c)") }
    pub fn psDispParticles(_mask: u32, _code: u32) { unsafe { let _ = melee_runtime::tu::psdisp::psDispParticles(_mask as _, _code as _); } }
    pub fn hsd_8039CEAC(_mask: u32) { unsafe { let _ = melee_runtime::tu::particle::hsd_8039CEAC(_mask as _); } }
    pub fn hsd_8039EE24(_mask: u32) { todo!("hsd_8039EE24 (baselib/particle.c)") }
    pub fn hsd_8039F05C(_link: i32, _bank: i32, _id: i32) -> Gen { todo!("hsd_8039F05C (baselib/particle.c)") }
    pub fn hsd_8039EFAC(_link: i32, _bank: i32, _id: i32, _j: J) -> Gen { todo!("hsd_8039EFAC (baselib/particle.c)") }
    pub fn hsd_8039F6CC(_link: i32, _bank: i32, _id: i32, _j: J) -> Gen { todo!("hsd_8039F6CC (baselib/particle.c)") }
    pub fn hsd_8039D4DC(_g: Gen) { todo!("hsd_8039D4DC (baselib/particle.c)") }
    pub fn hsd_8039D1E4(_g: Gen, _cbs: *mut Option<unsafe extern "C" fn(*mut HSD_Particle)>) { todo!("hsd_8039D1E4 (baselib/particle.c)") }
    pub fn psAddGeneratorAppSRT_begin(_g: Gen, _s: i32) -> *mut HSD_psAppSRT { todo!("psAddGeneratorAppSRT_begin (baselib/psappsrt.c)") }
    pub fn psAddParticleAppSRT_begin(_p: *mut HSD_Particle, _s: i32) -> *mut HSD_psAppSRT { todo!("psAddParticleAppSRT_begin (baselib/psappsrt.c)") }
    pub fn psRemoveParticleAppSRT(_p: *mut HSD_Particle) { todo!("psRemoveParticleAppSRT (baselib/psappsrt.c)") }
    pub fn ptclref_cmdList(_bank: usize, _idx: usize) -> *mut u8 { todo!("ptclref_804D0E5C (baselib/particle.c)") }
    pub fn grLib_801C99C0(_bank: i32, _id: i32, _j: J, _f: i32) { unsafe { let _ = crate::tu::grlib::grLib_801C99C0(_bank as _, _id as _, _j as _, _f as _); } }
    pub fn ftCo_800C0FCC(_a: G, _b: G) { todo!("ftCo_800C0FCC (ftCommon/ftCo_Bury.c)") }
    pub fn OSReport(_s: &str) {}
}

type G = *mut HSD_GObj;
type J = *mut HSD_JObj;
type Gen = *mut HSD_Generator;

/// Routes the jobj.h setter inlines' dirty marking to baselib.
struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

#[inline(always)]
unsafe fn jobj_of(g: G) -> J { (*g).hsd_obj as J }
#[inline(always)]
unsafe fn effect_of(g: G) -> *mut EF_Effect { (*g).user_data as *mut EF_Effect }
#[inline(always)]
unsafe fn fighter_of(g: G) -> *mut Fighter { (*g).user_data as *mut Fighter }
#[inline(always)]
unsafe fn root_of(mut j: J) -> J {
    while !HSD_JObjGetParent(j.as_ref()).is_null() {
        j = HSD_JObjGetParent(j.as_ref());
    }
    j
}
unsafe fn set_rot_y(j: J, v: f32) { HSD_JObjSetRotationY(&mut *j, v, &mut Dirty) }
unsafe fn set_rot_z(j: J, v: f32) { HSD_JObjSetRotationZ(&mut *j, v, &mut Dirty) }
unsafe fn set_scale(j: J, v: &Vec3) { HSD_JObjSetScale(&mut *j, v, &mut Dirty) }
unsafe fn set_translate(j: J, v: &Vec3) { HSD_JObjSetTranslate(&mut *j, v, &mut Dirty) }
unsafe fn get_scale(j: J) -> Vec3 {
    let mut s = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    HSD_JObjGetScale(&*j, &mut s);
    s
}

/// `eflib_create_generator_add_appsrt` (static inline).
unsafe fn create_generator_add_appsrt(gfx_id: i32, jobj: J) -> Gen {
    let mut generator = ext::hsd_8039EFAC(0, gfx_id / 1000, gfx_id, jobj);
    if !generator.is_null() {
        let mut appsrt = (*generator).appsrt;
        if appsrt.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(generator, 0);
        }
        if !appsrt.is_null() {
            (*appsrt).gp = generator;
        } else {
            ext::hsd_8039D4DC(generator);
            generator = core::ptr::null_mut();
            return generator;
        }
        (*generator).r#type &= 0xF9FF;
        (*generator).r#type |= PSAPPSRT_UNK_B11 as u16;
    }
    generator
}

/// `eflib_generator_add_appsrt` (static inline).
unsafe fn generator_add_appsrt(mut generator: Gen, status: i32) -> Gen {
    if !generator.is_null() {
        let mut a = (*generator).appsrt;
        if a.is_null() {
            a = ext::psAddGeneratorAppSRT_begin(generator, status);
        }
        if a.is_null() {
            ext::hsd_8039D4DC(generator);
            generator = core::ptr::null_mut();
        }
    }
    generator
}

/// `efLib_Init`.
pub unsafe fn efLib_Init() {
    melee_core::ct!("efLib_Init");
    ext::HSD_ObjAllocInit(&mut EF_DATA.alloc_data, core::mem::size_of::<EF_Effect>() as u32, 4);
    EF_DATA.effect_count = 0;
    ext::efAsync_DatEntries_clear();
    ext::hsd_8039D354(0);
    ext::hsd_80398A08(0);
    ext::set_hsd_804D7900(efLib_Cb_PtclAppSRTHook);
    let g = ext::GObj_Create(8, 0xB, 1);
    ext::GObj_SetupGXLink(g, efLib_render_callback, 7, 2);
    ext::HSD_GObj_SetupProc(g, efLib_particles_proc_main, 0xF);
    let g = ext::GObj_Create(8, 0xC, 1);
    ext::GObj_SetupGXLink(g, efLib_render_callback, 8, 2);
    ext::HSD_GObj_SetupProc(g, efLib_particles_proc_aux, 0xF);
    ext::HSD_JObjSetSPtclCallback(efLib_Cb_SPtcl);
    ext::HSD_JObjSetDPtclCallback(efLib_Cb_DPtcl);
    ext::psInitAppSRT(0, 0xA4);
    ext::efAsync_QueueInit();
    for e in efLib_ParamTable.iter_mut() {
        e.gobj = core::ptr::null_mut();
        e.alpha = 0xFF;
    }
}

fn plink_heads() -> [G; 2] {
    [ext::HSD_GObjPLinkHead(crate::ef::eflib::HSD_GOBJ_PLINK_EFFECT_UNK0), ext::HSD_GObjPLinkHead(crate::ef::eflib::HSD_GOBJ_PLINK_EFFECT_UNK1)]
}
fn plink_table() -> [G; 13] {
    let [a, b] = plink_heads();
    let mut t = [core::ptr::null_mut(); 13];
    t[crate::ef::eflib::HSD_GOBJ_PLINK_EFFECT_UNK0] = a;
    t[crate::ef::eflib::HSD_GOBJ_PLINK_EFFECT_UNK1] = b;
    t
}

/// `efLib_SetFlags` (walk ported in `crate::ef::eflib`).
pub unsafe fn efLib_SetFlags(gobj: G, expire_flags: i32) { melee_core::ct!("efLib_SetFlags"); crate::ef::eflib::eflib_set_flags(&plink_table(), gobj, expire_flags) }
/// `efLib_PauseAll`.
pub unsafe fn efLib_PauseAll(gobj: G) { melee_core::ct!("efLib_PauseAll"); crate::ef::eflib::eflib_pause_all(&plink_table(), gobj) }
/// `efLib_ResumeAll`.
pub unsafe fn efLib_ResumeAll(gobj: G) { melee_core::ct!("efLib_ResumeAll"); crate::ef::eflib::eflib_resume_all(&plink_table(), gobj) }

/// `efLib_Destroy`.
pub unsafe fn efLib_Destroy(gobj: G) {
    melee_core::ct!("efLib_Destroy");
    let effect = (*gobj).user_data as *mut EF_Effect;
    if !effect.is_null() && (*effect).is_async == 0 && !gobj.is_null() {
        for e in efLib_ParamTable.iter_mut() {
            if e.gobj as G == gobj {
                e.gobj = core::ptr::null_mut();
            }
        }
        if (*gobj).obj_kind == ext::HSD_GObj_JObjKind() {
            ext::HSD_JObjWalkTree_8039D688((*gobj).hsd_obj as J);
        }
        ext::HSD_GObjFree(gobj);
    }
}

/// `efLib_DestroyAll`.
pub unsafe fn efLib_DestroyAll(gobj: G) {
    melee_core::ct!("efLib_DestroyAll");
    for e in efLib_ParamTable.iter_mut() {
        if e.gobj as G == gobj {
            e.gobj = core::ptr::null_mut();
        }
    }
    for mut cur in plink_heads() {
        while !cur.is_null() {
            let e = effect_of(cur);
            cur = (*cur).next;
            if !e.is_null() && (*e).parent_gobj == gobj {
                let eg = (*e).gobj;
                if (*eg).obj_kind == ext::HSD_GObj_JObjKind() {
                    ext::HSD_JObjWalkTree_8039D688((*eg).hsd_obj as J);
                }
                ext::HSD_GObjFree((*e).gobj);
            }
        }
    }
    if (*gobj).obj_kind == ext::HSD_GObj_JObjKind() {
        ext::HSD_JObjWalkTree_8039D688((*gobj).hsd_obj as J);
    }
}

/// `efLib_remove_user_data`.
pub unsafe extern "C" fn efLib_remove_user_data(user_data: *mut core::ffi::c_void) {
    melee_core::ct!("efLib_remove_user_data");
    let effect = user_data as *mut EF_Effect;
    if !(*effect).gobj.is_null() {
        if (*effect).is_async == 0 {
            EF_DATA.effect_count = EF_DATA.effect_count.wrapping_sub(1);
        }
        ext::HSD_ObjFree(&mut EF_DATA.alloc_data, user_data);
    } else {
        ext::OSReport("Duplicate Free\n");
        panic!("eflib.c:53: 0");
    }
}

/// `efLib_RemoveLast`.
pub unsafe fn efLib_RemoveLast() {
    melee_core::ct!("efLib_RemoveLast");
    for mut g in plink_heads() {
        while !g.is_null() {
            let next = (*g).next;
            efLib_Destroy(g);
            if EF_DATA.effect_count < 0x40 {
                return;
            }
            g = next;
        }
    }
    panic!("eflib.c:77: can't remove no force effect!");
}

/// `efLib_Update` (GObj proc).
pub unsafe extern "C" fn efLib_Update(gobj: G) {
    melee_core::ct!("efLib_Update");
    let jobj = jobj_of(gobj);
    let effect = effect_of(gobj);
    match (*effect).state_flags as i32 {
        EF_STATE_ENTERING_PAUSE => (*effect).state_flags = (*effect).state_flags.wrapping_add(1),
        EF_STATE_PAUSED => return,
        _ => {}
    }
    if (*effect).lifetime != 0 {
        let p = (*effect).lifetime - 1;
        (*effect).lifetime = p;
        if p == 0 {
            ext::HSD_GObjFree(gobj);
            return;
        }
    }
    if (*effect).scale_flags as i32 != EF_SCALE_NO_INHERIT && !(*effect).attach_jobj.is_null() {
        let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let inherit = (*effect).scale_flags as i32 & EF_SCALE_INHERIT != 0;
        if !HSD_JObjGetParent((*effect).attach_jobj.as_ref()).is_null() {
            let a = (*effect).attach_jobj;
            if !a.is_null() && HSD_JObjMtxIsDirty(&*a) {
                ext::HSD_JObjSetupMatrixSub(a);
            }
            if inherit {
                ext::HSD_MtxGetScale(&mut (*(*effect).attach_jobj).mtx, &mut scale);
            }
        } else if inherit {
            HSD_JObjGetScale(&*(*effect).attach_jobj, &mut scale);
        }
        if inherit {
            scale.z = scale.y;
            scale.x = scale.y;
            set_scale(jobj, &scale);
        }
    }
    ext::HSD_JObjAnimAll(jobj);
    if let Some(u) = (*effect).update {
        u(effect);
    }
}

/// `efLib_Create` (TARGET_PC: unloaded/out-of-range bank returns NULL).
pub unsafe fn efLib_Create(gfx_id: i32, parent_gobj: G) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create");
    let bank = gfx_id / 1000;
    let data = ext::efAsync_DatEntries_data(bank);
    if data.is_null() {
        return core::ptr::null_mut();
    }
    let desc = (data as *mut EF_EffectDesc).offset((gfx_id % 1000) as isize);
    if EF_DATA.load_kind == EF_LOADKIND_ASYNC as u32 {
        if EF_DATA.effect_count >= 64 {
            efLib_RemoveLast();
        }
        EF_DATA.effect_count = EF_DATA.effect_count.wrapping_add(1);
    }
    let effect = ext::HSD_ObjAlloc(&mut EF_DATA.alloc_data) as *mut EF_Effect;
    if effect.is_null() {
        return core::ptr::null_mut();
    }
    (*effect).next = core::ptr::null_mut();
    (*effect).gobj = core::ptr::null_mut();
    (*effect).attach_jobj = core::ptr::null_mut();
    (*effect).update = None;
    (*effect).state_flags = EF_STATE_ACTIVE as u8;
    (*effect).expire_flags = EF_DOES_NOT_EXPIRE as i8;
    (*effect).scale_flags = EF_SCALE_NO_INHERIT as u8;
    (*effect).is_async = EF_DATA.load_kind as u8;
    if parent_gobj.is_null() && (*effect).is_async != 0 {
        panic!("eflib.c:177: error no parent gobj!");
    }
    let p_link: u8 = if gfx_id >= 27 || gfx_id < 25 { 11 } else { 12 };
    (*effect).parent_gobj = parent_gobj;
    let g = ext::GObj_Create(HSD_GOBJ_CLASS_EFFECT as u16, p_link, 0);
    (*effect).gobj = g;
    if g.is_null() {
        // (*effect).gobj was just set NULL: the C takes its "Duplicate Free" branch.
        panic!("eflib.c:53: Duplicate Free");
    }
    ext::GObj_SetupGXLink_JObjCallback((*effect).gobj, 7, 1);
    ext::GObj_InitUserData((*effect).gobj, 8, efLib_remove_user_data, effect as *mut _);
    let jobj = ext::HSD_JObjLoadJoint((*desc).model_desc.joint);
    if jobj.is_null() {
        ext::HSD_GObjFree((*effect).gobj);
        return core::ptr::null_mut();
    }
    ext::HSD_GObjObject_80390A70((*effect).gobj, ext::HSD_GObj_JObjKind(), jobj as *mut _);
    if lifetime_tenths(&*desc) % 10 != 0 {
        ext::lb_80011C18(jobj, 0x0800_0000);
    }
    (*effect).lifetime = f2u32((*desc).lifetime) as u16;
    if (*effect).lifetime != 0 {
        (*effect).lifetime = (*effect).lifetime.wrapping_add(1);
    }
    let md = &(*desc).model_desc;
    if !md.animjoint.is_null() | !md.matanim_joint.is_null() | !md.shapeanim_joint.is_null() {
        ext::HSD_JObjAddAnimAll(jobj, md.animjoint, md.matanim_joint, md.shapeanim_joint);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        let i = EF_DATA.anim_count;
        EF_DATA.anim_count += 1;
        efLib_AnimQueue[i as usize] = jobj;
        if EF_DATA.anim_count >= 32 {
            panic!("eflib.c:224: Over Anime Call");
        }
    }
    ext::HSD_GObj_SetupProc((*effect).gobj, efLib_Update, 15);
    effect
}

/// `(u32) f`: gwtool's `fptoui.sat.i32` (saturating, negative / NaN -> 0), `melee_runtime::cabi::f2u32`.
#[inline]
pub fn f2u32(f: f32) -> u32 { melee_core::ct!("f2u32"); melee_runtime::cabi::f2u32(f as f64) }
/// `(u32) (10.0F * desc->lifetime)`.
pub fn lifetime_tenths(desc: &EF_EffectDesc) -> u32 { melee_core::ct!("lifetime_tenths"); f2u32(10.0f32 * desc.lifetime) }

/// `efLib_Create_Attach`.
pub unsafe fn efLib_Create_Attach(gfx_id: u32, gobj: G, jobj: J) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach");
    let effect = efLib_Create(gfx_id as i32, gobj);
    if !effect.is_null() {
        let ej = jobj_of((*effect).gobj);
        if ej.is_null() {
            ext::HSD_GObjFree((*effect).gobj);
            return core::ptr::null_mut();
        }
        let mut t = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000C1C0(ej, jobj);
        ext::lb_8000B1CC(jobj, core::ptr::null(), &mut t);
        set_translate(ej, &t);
        (*effect).attach_jobj = jobj;
    }
    effect
}

/// `efLib_Create_AttachChild`.
pub unsafe fn efLib_Create_AttachChild(gfx_id: u32, gobj: G, jobj: J) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_AttachChild");
    let e = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !e.is_null() {
        ext::lb_8000C290(jobj_of((*e).gobj), jobj);
    }
    e
}

unsafe fn copy_scale_y(from: G, e: *mut EF_Effect) {
    let mut s = get_scale(jobj_of(from));
    s.z = s.y;
    s.x = s.y;
    set_scale(jobj_of((*e).gobj), &s);
}

/// `efLib_Create_Attach_Scale`.
pub unsafe fn efLib_Create_Attach_Scale(gfx_id: u32, gobj: G, jobj: J) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Scale");
    let e = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !e.is_null() {
        copy_scale_y(gobj, e);
    }
    e
}

/// `efLib_Create_AttachChild_Scale`.
pub unsafe fn efLib_Create_AttachChild_Scale(gfx_id: u32, gobj: G, jobj: J) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_AttachChild_Scale");
    let e = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !e.is_null() {
        ext::lb_8000C290(jobj_of((*e).gobj), jobj);
        copy_scale_y(gobj, e);
    }
    e
}

/// `efLib_Create_Attach_Scale_FacingDir`.
pub unsafe fn efLib_Create_Attach_Scale_FacingDir(gfx_id: u32, gobj: G, jobj: J) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Scale_FacingDir");
    let e = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !e.is_null() {
        copy_scale_y(gobj, e);
        (*e).update = Some(efLib_Cb_SetRotY_FromFighterDir);
    }
    e
}

/// `efLib_Create_Attach_Pos`.
pub unsafe fn efLib_Create_Attach_Pos(gfx_id: u32, gobj: G, position: &Vec3) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Pos");
    let e = efLib_Create(gfx_id as i32, gobj);
    if !e.is_null() {
        set_translate(jobj_of((*e).gobj), position);
    }
    e
}

/// `efLib_render_callback` (GX link).
pub unsafe extern "C" fn efLib_render_callback(gobj: G, code: i32) {
    melee_core::ct!("efLib_render_callback");
    let pc = match code {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => return,
    };
    ext::HSD_StateSetColorUpdate(1);
    if (*gobj).gx_link == 7 {
        ext::psDispParticles(PTCL_RENDER_LINKNO_0 | PTCL_RENDER_LINKNO_2 | PTCL_STAGE_RENDER, pc);
        return;
    }
    ext::psDispParticles(PTCL_RENDER_LINKNO_1, pc);
}

/// `efLib_particles_proc_main`.
pub unsafe extern "C" fn efLib_particles_proc_main(_gobj: G) {
    melee_core::ct!("efLib_particles_proc_main");
    ext::hsd_8039CEAC(PTCL_SKIP_LINKNO_1 | PTCL_SKIP_LINKNO_2);
    ext::hsd_8039EE24(PTCL_SKIP_LINKNO_1 | PTCL_SKIP_LINKNO_2);
}

/// `efLib_particles_proc_aux`.
pub unsafe extern "C" fn efLib_particles_proc_aux(_gobj: G) {
    melee_core::ct!("efLib_particles_proc_aux");
    ext::hsd_8039CEAC(PTCL_SKIP_LINKNO_0 | PTCL_STAGE_SKIP);
    ext::hsd_8039EE24(PTCL_SKIP_LINKNO_0 | PTCL_STAGE_SKIP);
}

/// linkNo picked by `efLib_CreateGenerator`.
pub fn create_generator_link(gfx_id: i32) -> i32 {
    melee_core::ct!("create_generator_link");
    match gfx_id {
        0x121 => 2,
        0xFF | 0x7918 | 0xFC | 0xF7 => 1,
        _ => 0,
    }
}

/// `efLib_CreateGenerator`.
pub unsafe fn efLib_CreateGenerator(gfx_id: i32, pos: &Vec3) -> Gen {
    melee_core::ct!("efLib_CreateGenerator");
    let g = ext::hsd_8039F05C(create_generator_link(gfx_id), gfx_id / 1000, gfx_id);
    if !g.is_null() {
        (*g).pos = *pos;
    }
    g
}

/// `efLib_CreateGenerator_AddAppSRT`.
pub unsafe fn efLib_CreateGenerator_AddAppSRT(gfx_id: i32) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_AddAppSRT");
    generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1)
}

/// `efLib_CreateGenerator_Translate_FacingDir`.
pub unsafe fn efLib_CreateGenerator_Translate_FacingDir(gfx_id: i32, t: &Vec3, direction: f32) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_Translate_FacingDir");
    let g = generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1);
    if !g.is_null() {
        (*(*g).appsrt).translate = *t;
        if direction < 0.0 {
            (*g).kind |= 0x40000;
            (*(*g).appsrt).rot.y = -M_PI_2_F;
        } else {
            (*(*g).appsrt).rot.y = M_PI_2_F;
        }
    }
    g
}

/// `efLib_CreateGenerator_Attach`.
pub unsafe fn efLib_CreateGenerator_Attach(gfx_id: i32, jobj: J) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_Attach");
    let g = ext::hsd_8039EFAC(0, gfx_id / 1000, gfx_id, jobj);
    if !g.is_null() {
        (*g).r#type &= !(PSAPPSRT_UNK_B10 as u16);
    }
    g
}

/// `efLib_CreateGenerator_Attach_AddAppSRT`.
pub unsafe fn efLib_CreateGenerator_Attach_AddAppSRT(gfx_id: i32, jobj: J) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_Attach_AddAppSRT");
    create_generator_add_appsrt(gfx_id, jobj)
}

/// `efLib_CreateGenerator_Attach_Scale` (va_list: the jobj).
pub unsafe fn efLib_CreateGenerator_Attach_Scale(gfx_id: i32, jobj: J, gobj: G) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_Attach_Scale");
    let g = create_generator_add_appsrt(gfx_id, jobj);
    if !g.is_null() {
        let s = get_scale(jobj_of(gobj));
        let a = &mut *(*g).appsrt;
        a.scale.z = s.y;
        a.scale.y = s.y;
        a.scale.x = s.y;
    }
    g
}

/// `efLib_CreateGenerator_AppSRT_SetScale` (va_list: jobj, f32*).
pub unsafe fn efLib_CreateGenerator_AppSRT_SetScale(gfx_id: i32, jobj: J, scale: f32) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetScale");
    let g = create_generator_add_appsrt(gfx_id, jobj);
    if !g.is_null() {
        let a = &mut *(*g).appsrt;
        a.scale.z = scale;
        a.scale.y = scale;
        a.scale.x = scale;
    }
    g
}

/// `direction < 0.0F ? -M_PI_2 : M_PI_2` stored to f32.
#[inline]
pub fn facing_rot(direction: f32) -> f32 { melee_core::ct!("facing_rot"); (if direction < 0.0 { -M_PI_2 } else { M_PI_2 }) as f32 }

/// `efLib_CreateGenerator_AppSRT_SetFacingDir` (va_list: jobj, f32*).
pub unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDir(gfx_id: i32, jobj: J, direction: f32) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetFacingDir");
    let g = create_generator_add_appsrt(gfx_id, jobj);
    if !g.is_null() {
        (*(*g).appsrt).rot.y = facing_rot(direction);
    }
    g
}

/// `efLib_CreateGenerator_AppSRT_SetFacingDirScale` (va_list: jobj, f32*, f32*).
pub unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDirScale(gfx_id: i32, jobj: J, direction: f32, scale: f32) -> Gen {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetFacingDirScale");
    let g = create_generator_add_appsrt(gfx_id, jobj);
    if !g.is_null() {
        let a = &mut *(*g).appsrt;
        a.rot.y = facing_rot(direction);
        a.scale.z = scale;
        a.scale.y = scale;
        a.scale.x = scale;
    }
    g
}

/// `efLib_SpawnParticleEffect`.
pub unsafe fn efLib_SpawnParticleEffect(bank: i32, gfx_id: i32, jobj: J, flag: bool) {
    melee_core::ct!("efLib_SpawnParticleEffect");
    let mut chk = 0;
    match gfx_id {
        0x4A38 | 0x4A39 | 0x4A3A | 0xB798 | 0xB799 | 0xB79A => {
            let g = create_generator_add_appsrt(gfx_id, jobj);
            if !g.is_null() {
                (*(*g).appsrt).rot.y = HSD_JObjGetRotationY(&*root_of(jobj));
            }
            return;
        }
        0x2D | 0x2E | 0x31 => {
            let g = create_generator_add_appsrt(gfx_id, jobj);
            if !g.is_null() {
                (*g).r#type |= 0x1000;
            }
            return;
        }
        0x127 => {
            let g = create_generator_add_appsrt(gfx_id, jobj);
            if !g.is_null() {
                let r = root_of(jobj);
                (*(*g).appsrt).rot.y = HSD_JObjGetRotationY(&*r);
                HSD_JObjGetScale(&*r, &mut (*(*g).appsrt).scale);
            }
            return;
        }
        0x2 | 0x6 | 0xA3 | 0xA7 | 0xAA | 0xF2 | 0x12A | 0x132 | 0x133 => {
            let g = create_generator_add_appsrt(gfx_id, jobj);
            if !g.is_null() {
                HSD_JObjGetScale(&*root_of(jobj), &mut (*(*g).appsrt).scale);
            }
            return;
        }
        0x16D | 0x16E | 0x16F | 0x170 | 0x7E2 => {
            let g = generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1);
            if !g.is_null() {
                let r = root_of(jobj);
                (*(*g).appsrt).rot.y = HSD_JObjGetRotationY(&*r);
                HSD_JObjGetTranslation(&*r, &mut (*(*g).appsrt).translate);
            }
            return;
        }
        0xD4 | 0x243 => {
            let g = ext::hsd_8039F05C(2, bank, gfx_id);
            if !g.is_null() {
                let mut a = (*g).appsrt;
                if a.is_null() {
                    a = ext::psAddGeneratorAppSRT_begin(g, 1);
                }
                if !a.is_null() {
                    let r = root_of(jobj);
                    (*a).rot.z = HSD_JObjGetRotationZ(&*r);
                    HSD_JObjGetTranslation(&*r, &mut (*a).translate);
                    if gfx_id == 0xD4 {
                        HSD_JObjGetScale(&*r, &mut (*a).scale);
                    }
                    return;
                }
                ext::hsd_8039D4DC(g);
            }
            return;
        }
        0xE3 => {
            let g = generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1);
            if !g.is_null() {
                ext::lb_8000B1CC(jobj, core::ptr::null(), &mut (*(*g).appsrt).translate);
                HSD_JObjGetScale(&*root_of(jobj), &mut (*(*g).appsrt).scale);
            }
            return;
        }
        0xFC | 0xFF | 0xF7 | 0x7918 => chk = 1,
        _ => {}
    }
    if flag {
        ext::hsd_8039EFAC(chk, bank, gfx_id, jobj);
    } else {
        ext::hsd_8039F6CC(chk, bank, gfx_id, jobj);
    }
}

/// `efLib_Cb_SPtcl`.
pub unsafe extern "C" fn efLib_Cb_SPtcl(_link: i32, bank: i32, gfx_id: i32, jobj: J) {
    melee_core::ct!("efLib_Cb_SPtcl");
    if bank == 0x1E {
        ext::grLib_801C99C0(bank, gfx_id, jobj, 0);
        return;
    }
    efLib_SpawnParticleEffect(bank, gfx_id, jobj, false);
}

/// `efLib_Cb_DPtcl`.
pub unsafe extern "C" fn efLib_Cb_DPtcl(_link: i32, bank: i32, gfx_id: i32, jobj: J) {
    melee_core::ct!("efLib_Cb_DPtcl");
    if bank == 0x1E {
        ext::grLib_801C99C0(bank, gfx_id, jobj, 1);
        return;
    }
    efLib_SpawnParticleEffect(bank, gfx_id, jobj, true);
}

/// `efLib_Cb_ParticleRender`.
pub unsafe extern "C" fn efLib_Cb_ParticleRender(p: *mut HSD_Particle) {
    melee_core::ct!("efLib_Cb_ParticleRender");
    if !p.is_null() && !(*p).appsrt.is_null() {
        let t = (*(*p).appsrt).translate;
        let s = (*(*p).appsrt).scale;
        let r = (*(*p).appsrt).rot;
        ext::psRemoveParticleAppSRT(p);
        let a = ext::psAddParticleAppSRT_begin(p, 1);
        if !a.is_null() {
            (*a).translate = t;
            (*a).rot = r;
            (*a).scale = s;
        } else {
            (*p).cmdWait = 0;
            (*p).size = 0.0;
            (*p).life = 1;
        }
    }
}

/// `lbl_803BF810`.
pub static mut lbl_803BF810: [Option<unsafe extern "C" fn(*mut HSD_Particle)>; 3] = [Some(efLib_Cb_ParticleRender), None, None];

/// `efLib_Cb_PtclAppSRTHook`.
pub unsafe extern "C" fn efLib_Cb_PtclAppSRTHook(gen: Gen) {
    melee_core::ct!("efLib_Cb_PtclAppSRTHook");
    for idx in [0x96, 0x97, 0x98, 0x21B] {
        if (*gen).cmdList == ext::ptclref_cmdList(0, idx) {
            ext::hsd_8039D1E4(gen, lbl_803BF810.as_mut_ptr());
        }
    }
}

/// `efLib_Cb_SetOffsetY_FromParamY`.
pub unsafe extern "C" fn efLib_Cb_SetOffsetY_FromParamY(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetOffsetY_FromParamY");
    let ej = jobj_of((*e).gobj);
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC((*e).attach_jobj, core::ptr::null(), &mut v);
    v.y += (*e).params.y;
    set_translate(ej, &v);
}

/// `efLib_Cb_SetScale_FromParamX`.
pub unsafe extern "C" fn efLib_Cb_SetScale_FromParamX(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetScale_FromParamX");
    let j = jobj_of((*e).gobj);
    let x = (*e).params.x * HSD_JObjGetScaleX(&*j);
    HSD_JObjSetScaleX(&mut *j, x, &mut Dirty);
    let y = (*e).params.x * HSD_JObjGetScaleY(&*j);
    HSD_JObjSetScaleY(&mut *j, y, &mut Dirty);
    let z = (*e).params.x * HSD_JObjGetScaleZ(&*j);
    HSD_JObjSetScaleZ(&mut *j, z, &mut Dirty);
}

/// `efLib_Cb_SetRotYAndTransition`.
pub unsafe extern "C" fn efLib_Cb_SetRotYAndTransition(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYAndTransition");
    let ud = (*e).user_data as J;
    let ej = jobj_of((*e).gobj);
    if !ud.is_null() {
        set_rot_y(ej, facing_rot((*ud).scale.x));
    }
    if (*e).expire_flags as u8 as i32 != EF_DOES_NOT_EXPIRE {
        (*e).lifetime = 0xB;
        (*e).update = None;
        ext::HSD_JObjReqAnimAll(ej, 65.0);
        return;
    }
    if (*e).lifetime == 1 {
        (*e).lifetime = 6;
        ext::HSD_JObjReqAnimAll(ej, 60.0);
    }
}

/// `efLib_Cb_SetJObjOffsetZ`.
pub unsafe extern "C" fn efLib_Cb_SetJObjOffsetZ(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetJObjOffsetZ");
    let ej = jobj_of((*e).gobj);
    let c = if ej.is_null() { core::ptr::null_mut() } else { (*ej).child };
    let z = 2.0f32 + HSD_JObjGetTranslationZ(&*c);
    HSD_JObjSetTranslateZ(&mut *c, z, &mut Dirty);
}

/// `efLib_Cb_SetRotY_FromFighterDir`.
pub unsafe extern "C" fn efLib_Cb_SetRotY_FromFighterDir(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotY_FromFighterDir");
    let ej = jobj_of((*e).gobj);
    set_rot_y(ej, facing_rot((*fighter_of((*e).parent_gobj)).facing_dir));
}

/// `efLib_Cb_SetRotYZ_FromFighter`.
pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromFighter(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYZ_FromFighter");
    let f = fighter_of((*e).parent_gobj);
    let j = jobj_of((*e).gobj);
    let hp = M_PI_2_F - (*f).mv.co.common.x4.x;
    if (*f).facing_dir < 0.0 {
        set_rot_y(j, -M_PI_2_F);
        set_rot_z(j, hp);
    } else {
        set_rot_y(j, M_PI_2_F);
        set_rot_z(j, -hp);
    }
}

/// `efLib_Cb_Fall_FromParamY`.
pub unsafe extern "C" fn efLib_Cb_Fall_FromParamY(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_Fall_FromParamY");
    let j = jobj_of((*e).gobj);
    let tx = HSD_JObjGetTranslationX(&*j);
    let ty = HSD_JObjGetTranslationY(&*j);
    let tz = HSD_JObjGetTranslationZ(&*j);
    let rx = HSD_JObjGetRotationX(&*j);
    let ry = HSD_JObjGetRotationY(&*j);
    (*e).params.y -= 0.2f32;
    HSD_JObjSetTranslateX(&mut *j, tx + sinf(ry), &mut Dirty);
    HSD_JObjSetTranslateY(&mut *j, ty + (*e).params.y, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *j, tz + cosf(ry), &mut Dirty);
    HSD_JObjSetRotationX(&mut *j, 0.5f32 + rx, &mut Dirty);
}

/// `efLib_Cb_SetOffset_FromParams`.
pub unsafe extern "C" fn efLib_Cb_SetOffset_FromParams(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetOffset_FromParams");
    let j = jobj_of((*e).gobj);
    let tx = HSD_JObjGetTranslationX(&*j);
    let ty = HSD_JObjGetTranslationY(&*j);
    let tz = HSD_JObjGetTranslationZ(&*j);
    (*e).params.y -= 0.1f32;
    HSD_JObjSetTranslateX(&mut *j, tx + (*e).params.x, &mut Dirty);
    HSD_JObjSetTranslateY(&mut *j, ty + (*e).params.y, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *j, tz + (*e).params.z, &mut Dirty);
}

/// `efLib_Cb_LifetimeEndSpawn`.
pub unsafe extern "C" fn efLib_Cb_LifetimeEndSpawn(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_LifetimeEndSpawn");
    if (*e).lifetime == 1 {
        ext::hsd_8039EFAC(0, 0, 0x1AB, (*e).attach_jobj);
        (*e).update = None;
        (*e).lifetime = 0x27;
    }
}

/// `efLib_Cb_SetScaleRotY_FromFighter`.
pub unsafe extern "C" fn efLib_Cb_SetScaleRotY_FromFighter(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetScaleRotY_FromFighter");
    let pg = (*e).parent_gobj;
    let j1 = jobj_of(pg);
    let j2 = jobj_of((*e).gobj);
    let f = fighter_of(pg);
    let mut s1 = get_scale(j1);
    let s2 = get_scale(j2);
    s1.x *= s2.x;
    s1.y *= s2.y;
    s1.z *= s2.z;
    set_scale(j2, &s1);
    set_rot_y(j2, facing_rot((*f).facing_dir));
}

/// `efLib_Cb_SetRotYZ_FromParamZ_FighterDir`.
pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromParamZ_FighterDir(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYZ_FromParamZ_FighterDir");
    let j = jobj_of((*e).gobj);
    if (*fighter_of((*e).parent_gobj)).facing_dir < 0.0 {
        set_rot_y(j, -M_PI_2_F);
        set_rot_z(j, (*e).params.z);
    } else {
        set_rot_y(j, M_PI_2_F);
        set_rot_z(j, -(*e).params.z);
    }
}

/// Shared body of the Mario/Luigi SpecialLw callbacks.
unsafe fn special_lw_cb(e: *mut EF_Effect, motion: i32) {
    let ej = jobj_of((*e).gobj);
    let c = if ej.is_null() { core::ptr::null_mut() } else { (*ej).child };
    let cn = if c.is_null() { core::ptr::null_mut() } else { (*c).next };
    let f = fighter_of((*e).parent_gobj);
    if (*f).motion_id as i32 == motion {
        ext::HSD_JObjClearFlagsAll(cn, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(cn, JOBJ_HIDDEN as u32);
    }
    if (*f).cmd_vars[3] != 0 && (*f).mv.co.common.x4.z.to_bits() != 0 {
        let rz = -atan2f((*f).coll_data.floor.normal.x, (*f).coll_data.floor.normal.y);
        set_rot_z(ej, rz);
    } else {
        set_rot_z(ej, 0.0);
    }
}

/// `efLib_Cb_ftMr_SpecialLw`.
pub unsafe extern "C" fn efLib_Cb_ftMr_SpecialLw(e: *mut EF_Effect) { melee_core::ct!("efLib_Cb_ftMr_SpecialLw"); special_lw_cb(e, 349) }
/// `efLib_Cb_ftLg_SpecialLw`.
pub unsafe extern "C" fn efLib_Cb_ftLg_SpecialLw(e: *mut EF_Effect) { melee_core::ct!("efLib_Cb_ftLg_SpecialLw"); special_lw_cb(e, 357) }

/// `efLib_Cb_ftKp_SpecialHi`.
pub unsafe extern "C" fn efLib_Cb_ftKp_SpecialHi(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ftKp_SpecialHi");
    let j1 = jobj_of((*e).gobj);
    let j2 = jobj_of((*(*e).next).gobj);
    let f = fighter_of((*e).parent_gobj);
    if (*f).motion_id as i32 == 359 {
        ext::HSD_JObjClearFlagsAll(j2, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(j2, JOBJ_HIDDEN as u32);
    }
    if (*f).cmd_vars[2] & 1 != 0 && ((*f).mv.co.common.x10 as usize as i32) != 0 {
        let rz = -atan2f((*f).coll_data.floor.normal.x, (*f).coll_data.floor.normal.y);
        set_rot_z(j1, rz);
        set_rot_z(j2, rz);
    } else {
        set_rot_z(j1, 0.0);
        set_rot_z(j2, 0.0);
    }
}

/// `efLib_Cb_ftCo_Bury`.
pub unsafe extern "C" fn efLib_Cb_ftCo_Bury(e: *mut EF_Effect) { melee_core::ct!("efLib_Cb_ftCo_Bury"); ext::ftCo_800C0FCC((*e).gobj, (*e).parent_gobj) }

unsafe fn tobj_of(j: J) -> *mut HSD_TObj {
    let d = ext::HSD_JObjGetDObj(j);
    let m = if d.is_null() { core::ptr::null_mut() } else { (*d).mobj };
    ext::HSD_MObjGetTObj(m)
}

/// `efLib_SetTevKonstColor`.
pub unsafe fn efLib_SetTevKonstColor(jobj: J, mut count: i32, konst: u32, tev0: u32) {
    melee_core::ct!("efLib_SetTevKonstColor");
    let mut t = tobj_of(jobj);
    while count != 0 {
        t = (*t).next;
        count -= 1;
    }
    let tev = &mut *(*t).tev;
    tev.konst.r = (konst >> 16) as u8;
    tev.konst.g = (konst >> 8) as u8;
    tev.konst.b = konst as u8;
    tev.tev0.r = (tev0 >> 16) as u8;
    tev.tev0.g = (tev0 >> 8) as u8;
    tev.tev0.b = tev0 as u8;
}

fn param_slot(gobj: G) -> Option<usize> {
    unsafe {
        (0..8)
            .find(|&i| efLib_ParamTable[i].gobj as G == gobj)
            .or_else(|| (0..8).find(|&i| efLib_ParamTable[i].gobj.is_null()))
    }
}

/// `efLib_SetParamAlpha` (TARGET_PC: indexes the table itself).
pub unsafe fn efLib_SetParamAlpha(gobj: G, alpha: u8) {
    melee_core::ct!("efLib_SetParamAlpha");
    if let Some(i) = param_slot(gobj) {
        efLib_ParamTable[i].gobj = gobj as *mut _;
        efLib_ParamTable[i].alpha = alpha as _;
    }
}

/// `efLib_SetParamGfxId`.
pub unsafe fn efLib_SetParamGfxId(gobj: G, gfx_id: i32) {
    melee_core::ct!("efLib_SetParamGfxId");
    if let Some(i) = param_slot(gobj) {
        efLib_ParamTable[i].gobj = gobj as *mut _;
        efLib_ParamTable[i].gfx_id = gfx_id as _;
    }
}

/// `efLib_Cb_ApplyStoredAlpha`.
pub unsafe extern "C" fn efLib_Cb_ApplyStoredAlpha(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ApplyStoredAlpha");
    let h = jobj_of((*e).gobj);
    let mut jobj = if h.is_null() { core::ptr::null_mut() } else { (*h).child };
    let mut t = tobj_of(jobj);
    for i in 0..8 {
        if efLib_ParamTable[i].gobj as G == (*e).parent_gobj {
            if efLib_ParamTable[i].gfx_id as i32 == 0x417 {
                t = (*t).next;
            }
            (*(*t).tev).konst.a = efLib_ParamTable[i].alpha as u8;
            if efLib_ParamTable[i].gfx_id as i32 == 0x419 {
                for _ in 0..6 {
                    jobj = (*jobj).next;
                    t = tobj_of(jobj);
                    (*(*t).tev).konst.a = efLib_ParamTable[i].alpha as u8;
                }
            }
            break;
        }
    }
}

/// `efLib_Cb_AccumOffset_FromParams`.
pub unsafe extern "C" fn efLib_Cb_AccumOffset_FromParams(e: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_AccumOffset_FromParams");
    let mut t = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC((*e).attach_jobj, core::ptr::null(), &mut t);
    t.x += (*e).params.x;
    t.y += (*e).params.y;
    t.z += (*e).params.z;
    set_translate(jobj_of((*e).gobj), &t);
}

/// `efLib_CreateGenerator_AppSRT_SetPos`.
pub unsafe fn efLib_CreateGenerator_AppSRT_SetPos(gfx_id: i32, gobj: G, jobj: J, vec: &Vec3) -> *mut EF_Effect {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetPos");
    let e = efLib_Create(0, gobj);
    if !e.is_null() {
        (*e).update = Some(efLib_Cb_AccumOffset_FromParams);
        (*e).attach_jobj = jobj;
        (*e).params = *vec;
        create_generator_add_appsrt(gfx_id, jobj_of((*e).gobj));
    }
    e
}
