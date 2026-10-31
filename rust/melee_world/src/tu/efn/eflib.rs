//! Mechanical native port of src/melee/ef/eflib.c (effect objects, particle generator helpers, effect
//! update callbacks, TARGET_PC variant), generated from the C by `gen/mtu/c2rs.py` (libclang AST: every
//! implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`). Verified against
//! the C compiled natively by the state oracle `tests/tu_ef.rs` (`python rust/melee_world/gen/mtu/mk_ef.py`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO", or the native port
//! `gen/rewire/` wired). `bool` is MSL `int`. The hand port `tu::eflib` (EfHost-free, ext stubs) predates this.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetDObj(a0 as _) as _ };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetDPtclCallback(a0: Option<unsafe extern "C" fn(i32, i32, i32, *mut HSD_JObj)>);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_JObjSetSPtclCallback(a0: Option<unsafe extern "C" fn(i32, i32, i32, *mut HSD_JObj)>);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_JObjWalkTree(a0: *mut HSD_JObj, a1: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut f32, i32)>, a2: *mut *mut f32);
    fn HSD_MObjGetTObj(a0: *mut HSD_MObj) -> *mut HSD_TObj = |a0: *mut HSD_MObj| unsafe { melee_runtime::tu::mobj::HSD_MObjGetTObj(a0 as _) as _ };
    fn HSD_MtxGetScale(a0: *mut [f32; 4], a1: *mut Vec) = |a0: *mut [f32; 4], a1: *mut Vec| { unsafe { melee_runtime::tu::mtx_world::HSD_MtxGetScale(a0 as _, a1 as _); } };
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = |a0: *mut _HSD_ObjAllocData| unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) as _ };
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = |a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32| { unsafe { melee_runtime::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2 as _); } };
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = |a0: *mut _HSD_ObjAllocData, a1: *mut c_void| { unsafe { melee_runtime::tu::objalloc::HSD_ObjFree(a0 as _, a1 as _); } };
    fn HSD_StateSetColorUpdate(a0: i32);
    fn atan2f(a0: f32, a1: f32) -> f32 = |a0: f32, a1: f32| unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ };
    fn ftCo_800C0FCC(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn grLib_801C99C0(a0: i32, a1: i32, a2: *mut HSD_JObj, a3: i32) = |a0: i32, a1: i32, a2: *mut HSD_JObj, a3: i32| { unsafe { crate::tu::grlib::grLib_801C99C0(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn hsd_80398A08(a0: u32) = |a0: u32| { unsafe { melee_runtime::tu::particle::hsd_80398A08(a0 as _); } };
    fn hsd_8039CEAC(a0: u32) = |a0: u32| { unsafe { melee_runtime::tu::particle::hsd_8039CEAC(a0 as _); } };
    fn hsd_8039D1E4(a0: *mut HSD_Generator, a1: *mut c_void);
    fn hsd_8039D354(a0: u32);
    fn hsd_8039D4DC(a0: *mut HSD_Generator);
    fn hsd_8039D688(a0: *mut HSD_JObj, a1: *mut *mut f32, a2: i32);
    fn hsd_8039EE24(a0: u32);
    fn hsd_8039EFAC(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) -> *mut HSD_Generator;
    fn hsd_8039F05C(a0: i32, a1: i32, a2: i32) -> *mut HSD_Generator;
    fn hsd_8039F6CC(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) -> *mut HSD_Generator;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_8000C1C0(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(a0 as _, a1 as _); } };
    fn lb_8000C290(a0: *mut HSD_JObj, a1: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(a0 as _, a1 as _); } };
    fn lb_80011C18(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::lbspdisplay::lb_80011C18(a0 as _, a1 as _); } };
    fn psAddGeneratorAppSRT_begin(a0: *mut HSD_Generator, a1: i32) -> *mut HSD_psAppSRT;
    fn psAddParticleAppSRT_begin(a0: *mut HSD_Particle, a1: i32) -> *mut HSD_psAppSRT;
    fn psDispParticles(a0: u32, a1: u32) = |a0: u32, a1: u32| { unsafe { melee_runtime::tu::psdisp::psDispParticles(a0 as _, a1 as _); } };
    fn psInitAppSRT(a0: i32, a1: i32) -> i32;
    fn psRemoveParticleAppSRT(a0: *mut HSD_Particle) -> i32;
    fn OSReport(a0: &str) = |a0: &str| melee_runtime::tu::os_error::OSReport_str(a0);
    }
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn hsd_8039D688(a0: *mut HSD_JObj, a1: *mut *mut f32, a2: i32) { super::hsd_8039D688(a0, a1, a2) }
    }
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::generator::hsd_804D7900;
    pub use melee_runtime::tu::particle::ptclref_804D0E5C;
}

pub static mut lbl_803BF810: [Option<unsafe extern "C" fn(*mut HSD_Particle)>; 3] = [Some(efLib_Cb_ParticleRender), None, None];
pub static mut efLib_AnimQueue: [EF_ParamEntry; 16] = unsafe { core::mem::zeroed() };
pub static mut efLib_ParamTable: [EF_ParamEntry; 8] = unsafe { core::mem::zeroed() };

/// `eflib_create_generator_add_appsrt`
pub unsafe extern "C" fn eflib_create_generator_add_appsrt(mut generator: *mut *mut HSD_Generator, mut gfx_id: i32, mut jobj: *mut HSD_JObj) {
    melee_core::ct!("eflib_create_generator_add_appsrt");
    *generator = ext::hsd_8039EFAC(0, gfx_id / 1000, gfx_id, jobj);
    if !(*generator).is_null() {
        let mut appsrt: *mut HSD_psAppSRT = null_mut();
        if { appsrt = (*(*generator)).appsrt; appsrt }.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(*generator, 0);
        }
        if !appsrt.is_null() {
            (*appsrt).gp = *generator;
        } else {
            ext::hsd_8039D4DC(*generator);
            *generator = null_mut();
            return;
        }
        (*(*generator)).r#type = (((*(*generator)).r#type as u32) & 4294965759_u32) as u16;
        (*(*generator)).r#type = (((*(*generator)).r#type as i32) | (1 << 11)) as u16;
    }
}

/// `eflib_create_effect_and_attach`
pub unsafe extern "C" fn eflib_create_effect_and_attach(mut gfx_id: i32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("eflib_create_effect_and_attach");
    let mut effect: *mut EF_Effect = efLib_Create(gfx_id, gobj);
    if !effect.is_null() {
        let mut effect_jobj: *mut HSD_JObj = null_mut();
        if { effect_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj; effect_jobj }.is_null() {
            ext::HSD_GObjFree((*effect).gobj);
            return null_mut();
        } else {
            let mut translate: Vec3 = unsafe { core::mem::zeroed() };
            ext::lb_8000C1C0(effect_jobj, jobj);
            ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
            HSD_JObjSetTranslate(effect_jobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
            (*effect).attach_jobj = jobj;
        }
    }
    return effect;
}

/// `eflib_generator_add_appsrt`
pub unsafe extern "C" fn eflib_generator_add_appsrt(mut generator: *mut HSD_Generator, mut status: i32) -> *mut HSD_Generator {
    melee_core::ct!("eflib_generator_add_appsrt");
    let mut psAppSRT: *mut HSD_psAppSRT = null_mut();
    if !generator.is_null() {
        if { psAppSRT = (*generator).appsrt; psAppSRT }.is_null() {
            psAppSRT = ext::psAddGeneratorAppSRT_begin(generator, status);
        }
        if psAppSRT.is_null() {
            ext::hsd_8039D4DC(generator);
            generator = null_mut();
        }
    }
    return generator;
}

/// `efLib_Init`
pub unsafe extern "C" fn efLib_Init() {
    melee_core::ct!("efLib_Init");
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut i: i32 = 0;
    ext::HSD_ObjAllocInit(core::ptr::addr_of_mut!(super::efdata::efLib_AllocData) as *mut _HSD_ObjAllocData, 72_u64 as u32, 4_u32);
    super::efdata::efLib_EffectCount = 0 as u32;
    {
        i = 0;
        'l35: while i < (65_u64 as i32) {
            super::efasync::efAsync_DatEntries[i as usize].data = null_mut();
            i = i + 1;
        }
    }
    ext::hsd_8039D354(0 as u32);
    ext::hsd_80398A08(0 as u32);
    ext::hsd_804D7900 = Some(efLib_Cb_PtclAppSRTHook);
    gobj = ext::GObj_Create(8_u32 as u16, 11_u32 as u8, 1_u32 as u8);
    ext::GObj_SetupGXLink(gobj, Some(efLib_render_callback), 7_u32 as u8, 2_u32);
    let _ = ext::HSD_GObj_SetupProc(gobj, Some(efLib_particles_proc_main), 15_u32 as u8);
    gobj = ext::GObj_Create(8_u32 as u16, 12_u32 as u8, 1_u32 as u8);
    ext::GObj_SetupGXLink(gobj, Some(efLib_render_callback), 8_u32 as u8, 2_u32);
    let _ = ext::HSD_GObj_SetupProc(gobj, Some(efLib_particles_proc_aux), 15_u32 as u8);
    ext::HSD_JObjSetSPtclCallback(Some(efLib_Cb_SPtcl));
    ext::HSD_JObjSetDPtclCallback(Some(efLib_Cb_DPtcl));
    let _ = ext::psInitAppSRT(0, 164);
    super::efasync::efAsync_QueueInit();
    {
        i = 0;
        'l36: while i < 8 {
            efLib_ParamTable[i as usize].gobj = null_mut();
            efLib_ParamTable[i as usize].alpha = 255 as u16;
            i = i + 1;
        }
    }
}

/// `efLib_SetFlags`
pub unsafe extern "C" fn efLib_SetFlags(mut gobj: *mut HSD_GObj, mut expire_flags: i32) {
    melee_core::ct!("efLib_SetFlags");
    let mut gobj_1: *mut HSD_GObj = null_mut();
    let mut gobj_2: *mut HSD_GObj = null_mut();
    let mut effect_1: *mut EF_Effect = null_mut();
    let mut effect_2: *mut EF_Effect = null_mut();
    gobj_1 = *ext::HSD_GObjPLinkHead.offset(11 as isize);
    'l37: while !gobj_1.is_null() {
        effect_1 = HSD_GObjGetUserData(gobj_1) as *mut EF_Effect;
        if (!effect_1.is_null()) && ((*effect_1).parent_gobj == gobj) {
            (*effect_1).expire_flags = (((*effect_1).expire_flags as i32) | expire_flags) as i8;
        }
        gobj_1 = (*gobj_1).next;
    }
    gobj_2 = *ext::HSD_GObjPLinkHead.offset(12 as isize);
    'l38: while !gobj_2.is_null() {
        effect_2 = HSD_GObjGetUserData(gobj_2) as *mut EF_Effect;
        if (!effect_2.is_null()) && ((*effect_2).parent_gobj == gobj) {
            (*effect_2).expire_flags = (((*effect_2).expire_flags as i32) | expire_flags) as i8;
        }
        gobj_2 = (*gobj_2).next;
    }
}

/// `efLib_Destroy`
pub unsafe extern "C" fn efLib_Destroy(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_Destroy");
    let mut effect: *mut EF_Effect = (*gobj).user_data as *mut EF_Effect;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    let mut obj_kind: u8 = 0;
    if ((!effect.is_null()) && (((*effect).is_async as i32) == 0)) && (!gobj.is_null()) {
        {
            i = 0;
            'l39: while i < 8 {
                let mut efLib_gobj: *mut HSD_GObj = efLib_ParamTable[i as usize].gobj as *mut HSD_GObj;
                if efLib_gobj == gobj {
                    efLib_ParamTable[i as usize].gobj = null_mut();
                }
                i = i + 1;
            }
        }
        obj_kind = (*gobj).obj_kind;
        if (obj_kind as i32) == (ext::HSD_GObj_JObjKind as i32) {
            jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            ext::HSD_JObjWalkTree(jobj, Some(ext::cb::hsd_8039D688), null_mut());
        }
        ext::HSD_GObjFree(gobj);
    }
}

/// `efLib_DestroyAll`
pub unsafe extern "C" fn efLib_DestroyAll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_DestroyAll");
    let mut gobj_1: *mut HSD_GObj = null_mut();
    let mut gobj_2: *mut HSD_GObj = null_mut();
    let mut i: i32 = 0;
    {
        i = 0;
        'l40: while i < 8 {
            if efLib_ParamTable[i as usize].gobj == (gobj as *mut c_void) {
                efLib_ParamTable[i as usize].gobj = null_mut();
            }
            i = i + 1;
        }
    }
    gobj_1 = *ext::HSD_GObjPLinkHead.offset(11 as isize);
    'l41: while !gobj_1.is_null() {
        let mut gobj_2: *mut HSD_GObj = null_mut();
        let mut effect_1: *mut EF_Effect = null_mut();
        gobj_2 = (*gobj_1).next;
        if (!{ effect_1 = HSD_GObjGetUserData(gobj_1) as *mut EF_Effect; effect_1 }.is_null()) && ((*effect_1).parent_gobj == gobj) {
            if ((*(*effect_1).gobj).obj_kind as i32) == (ext::HSD_GObj_JObjKind as i32) {
                ext::HSD_JObjWalkTree((*(*effect_1).gobj).hsd_obj as *mut HSD_JObj, Some(ext::cb::hsd_8039D688), null_mut());
            }
            ext::HSD_GObjFree((*effect_1).gobj);
        }
        gobj_1 = gobj_2;
    }
    gobj_2 = *ext::HSD_GObjPLinkHead.offset(12 as isize);
    'l42: while !gobj_2.is_null() {
        let mut gobj_3: *mut HSD_GObj = null_mut();
        let mut effect_2: *mut EF_Effect = null_mut();
        effect_2 = HSD_GObjGetUserData(gobj_2) as *mut EF_Effect;
        gobj_2 = (*gobj_2).next;
        if (!effect_2.is_null()) && ((*effect_2).parent_gobj == gobj) {
            gobj_3 = (*effect_2).gobj;
            if ((*gobj_3).obj_kind as i32) == (ext::HSD_GObj_JObjKind as i32) {
                ext::HSD_JObjWalkTree((*gobj_3).hsd_obj as *mut HSD_JObj, Some(ext::cb::hsd_8039D688), null_mut());
            }
            ext::HSD_GObjFree((*effect_2).gobj);
        }
    }
    if ((*gobj).obj_kind as i32) == (ext::HSD_GObj_JObjKind as i32) {
        ext::HSD_JObjWalkTree((*gobj).hsd_obj as *mut HSD_JObj, Some(ext::cb::hsd_8039D688), null_mut());
    }
}

/// `efLib_PauseAll`
pub unsafe extern "C" fn efLib_PauseAll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_PauseAll");
    let mut gobj_1: *mut HSD_GObj = null_mut();
    let mut gobj_2: *mut HSD_GObj = null_mut();
    let mut effect_1: *mut EF_Effect = null_mut();
    let mut effect_2: *mut EF_Effect = null_mut();
    gobj_1 = *ext::HSD_GObjPLinkHead.offset(11 as isize);
    'l43: while !gobj_1.is_null() {
        effect_1 = HSD_GObjGetUserData(gobj_1) as *mut EF_Effect;
        if (!effect_1.is_null()) && ((*effect_1).parent_gobj == gobj) {
            (*effect_1).state_flags = ((((*effect_1).state_flags as i32) & 128) | 1) as u8;
        }
        gobj_1 = (*gobj_1).next;
    }
    gobj_2 = *ext::HSD_GObjPLinkHead.offset(12 as isize);
    'l44: while !gobj_2.is_null() {
        effect_2 = HSD_GObjGetUserData(gobj_2) as *mut EF_Effect;
        if (!effect_2.is_null()) && ((*effect_2).parent_gobj == gobj) {
            (*effect_2).state_flags = ((((*effect_2).state_flags as i32) & 128) | 1) as u8;
        }
        gobj_2 = (*gobj_2).next;
    }
}

/// `efLib_ResumeAll`
pub unsafe extern "C" fn efLib_ResumeAll(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_ResumeAll");
    let mut gobj_1: *mut HSD_GObj = null_mut();
    let mut gobj_2: *mut HSD_GObj = null_mut();
    let mut effect_1: *mut EF_Effect = null_mut();
    let mut effect_2: *mut EF_Effect = null_mut();
    gobj_1 = *ext::HSD_GObjPLinkHead.offset(11 as isize);
    'l45: while !gobj_1.is_null() {
        effect_1 = HSD_GObjGetUserData(gobj_1) as *mut EF_Effect;
        if (!effect_1.is_null()) && ((*effect_1).parent_gobj == gobj) {
            (*effect_1).state_flags = (((*effect_1).state_flags as i32) & 128) as u8;
        }
        gobj_1 = (*gobj_1).next;
    }
    gobj_2 = *ext::HSD_GObjPLinkHead.offset(12 as isize);
    'l46: while !gobj_2.is_null() {
        effect_2 = HSD_GObjGetUserData(gobj_2) as *mut EF_Effect;
        if (!effect_2.is_null()) && ((*effect_2).parent_gobj == gobj) {
            (*effect_2).state_flags = (((*effect_2).state_flags as i32) & 128) as u8;
        }
        gobj_2 = (*gobj_2).next;
    }
}

/// `efLib_remove_user_data`
pub unsafe extern "C" fn efLib_remove_user_data(mut user_data: *mut c_void) {
    melee_core::ct!("efLib_remove_user_data");
    let mut effect: *mut EF_Effect = null_mut();
    let mut effect_2: *mut EF_Effect = null_mut();
    effect = { effect_2 = user_data as *mut EF_Effect; effect_2 };
    if !(*effect_2).gobj.is_null() {
        if ((*effect).is_async as i32) == 0 {
            super::efdata::efLib_EffectCount = super::efdata::efLib_EffectCount - 1;
        }
        ext::HSD_ObjFree(core::ptr::addr_of_mut!(super::efdata::efLib_AllocData) as *mut _HSD_ObjAllocData, effect as *mut c_void);
    } else {
        ext::OSReport(&format!("Duplicate Free {:08X}\n", ((effect) as usize as u32)));
        if 0 != 0 { (); } else { panic!("efLib_remove_user_data: assertion failed"); }
    }
}

/// `efLib_RemoveLast`
pub unsafe extern "C" fn efLib_RemoveLast() {
    melee_core::ct!("efLib_RemoveLast");
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut next: *mut HSD_GObj = null_mut();
    gobj = *ext::HSD_GObjPLinkHead.offset(11 as isize);
    'l47: while !gobj.is_null() {
        next = (*gobj).next;
        efLib_Destroy(gobj);
        {
            let mut check: u32 = super::efdata::efLib_EffectCount;
            let mut value: u32 = 64_u32;
            if check < value {
                return;
            }
        }
        gobj = next;
    }
    gobj = *ext::HSD_GObjPLinkHead.offset(12 as isize);
    'l48: while !gobj.is_null() {
        next = (*gobj).next;
        efLib_Destroy(gobj);
        {
            let mut check: u32 = super::efdata::efLib_EffectCount;
            let mut value: u32 = 64_u32;
            if check < value {
                return;
            }
        }
        gobj = next;
    }
    if 0 != 0 { (); } else { ext::OSReport(&format!("can't remove no force effect!\n")); panic!("efLib_RemoveLast: assertion failed"); }
}

/// `efLib_Update`
pub unsafe extern "C" fn efLib_Update(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_Update");
    let mut effect: *mut EF_Effect = null_mut();
    let mut jobj: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    's49: {
        match (*{ effect = HSD_GObjGetUserData(gobj) as *mut EF_Effect; effect }).state_flags as i32 {
            1 => {
                (*effect).state_flags = (*effect).state_flags + 1;
            }
            2 => {
                return;
            }
            _ => {}
        }
    }
    if ((*effect).lifetime as i32) != 0 {
        let mut param: u16 = (((*effect).lifetime as i32) - 1) as u16;
        (*effect).lifetime = param;
        if (param as i32) == 0 {
            ext::HSD_GObjFree(gobj);
            return;
        }
    }
    if ((*effect).scale_flags as i32) != 0 {
        if !(*effect).attach_jobj.is_null() {
            let mut scale: Vec3 = unsafe { core::mem::zeroed() };
            if !HSD_JObjGetParent((*effect).attach_jobj).is_null() {
                let mut attach_jobj: *mut HSD_JObj = (*effect).attach_jobj;
                if (!attach_jobj.is_null()) && (HSD_JObjMtxIsDirty(attach_jobj) != 0) {
                    ext::HSD_JObjSetupMatrixSub(attach_jobj);
                }
                if (((*effect).scale_flags as i32) & 1) != 0 {
                    ext::HSD_MtxGetScale(HSD_JObjGetMtxPtr((*effect).attach_jobj), core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            } else if (((*effect).scale_flags as i32) & 1) != 0 {
                HSD_JObjGetScale((*effect).attach_jobj, core::ptr::addr_of_mut!(scale) as *mut Vec);
            }
            if (((*effect).scale_flags as i32) & 1) != 0 {
                scale.x = { scale.z = scale.y; scale.z };
                HSD_JObjSetScale(jobj, core::ptr::addr_of_mut!(scale) as *mut Vec);
            }
        }
    }
    ext::HSD_JObjAnimAll(jobj);
    if (*effect).update.is_some() {
        (*effect).update.unwrap()(effect);
    }
}

/// `efLib_Create`
pub unsafe extern "C" fn efLib_Create(mut gfx_id: i32, mut parent_gobj: *mut HSD_GObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create");
    let mut effect: *mut EF_Effect = null_mut();
    let mut desc: *mut EF_EffectDesc = null_mut();
    let mut p_link: u8 = 0;
    {
        let mut bank: i32 = gfx_id / 1000;
        if ((bank < 0) || (bank >= (65_u64 as i32))) || (super::efasync::efAsync_DatEntries[bank as usize].data.is_null()) {
            return null_mut();
        }
    }
    desc = core::ptr::addr_of_mut!(*(super::efasync::efAsync_DatEntries[(gfx_id / 1000) as usize].data as *mut EF_EffectDesc).offset((gfx_id % 1000) as isize)) as *mut EF_EffectDesc;
    if super::efdata::efLib_LoadKind == (0 as u32) {
        if super::efdata::efLib_EffectCount >= (64 as u32) {
            efLib_RemoveLast();
        }
        super::efdata::efLib_EffectCount = super::efdata::efLib_EffectCount + (1 as u32);
    }
    {
        let mut new_effect: *mut EF_Effect = ext::HSD_ObjAlloc(core::ptr::addr_of_mut!(super::efdata::efLib_AllocData) as *mut _HSD_ObjAllocData) as *mut EF_Effect;
        if new_effect.is_null() {
            effect = null_mut();
        } else {
            (*new_effect).next = null_mut();
            effect = new_effect;
            (*new_effect).gobj = null_mut();
            (*new_effect).attach_jobj = null_mut();
            (*new_effect).update = None;
            (*new_effect).state_flags = 0 as u8;
            (*new_effect).expire_flags = 0 as i8;
            (*new_effect).scale_flags = 0 as u8;
            (*new_effect).is_async = super::efdata::efLib_LoadKind as u8;
        }
    }
    if effect.is_null() {
        return null_mut();
    }
    if (parent_gobj.is_null()) && (((*effect).is_async as i32) != 0) {
        if 0 != 0 { (); } else { ext::OSReport(&format!("error no parent gobj!\n")); panic!("efLib_Create: assertion failed"); }
    }
    if (gfx_id >= 27) || (gfx_id < 25) {
        p_link = 11 as u8;
    } else {
        p_link = 12 as u8;
    }
    (*effect).parent_gobj = parent_gobj;
    {
        let mut gobj: *mut HSD_GObj = ext::GObj_Create(8 as u16, p_link, 0 as u8);
        (*effect).gobj = gobj;
        if gobj.is_null() {
            if !(*effect).gobj.is_null() {
                if ((*effect).is_async as i32) == 0 {
                    super::efdata::efLib_EffectCount = super::efdata::efLib_EffectCount - (1 as u32);
                }
                ext::HSD_ObjFree(core::ptr::addr_of_mut!(super::efdata::efLib_AllocData) as *mut _HSD_ObjAllocData, effect as *mut c_void);
            } else {
                if 0 != 0 { (); } else { ext::OSReport(&format!("Duplicate Free {:08X}\n", ((effect) as usize as u32))); panic!("efLib_Create: assertion failed"); }
            }
            return null_mut();
        }
    }
    ext::GObj_SetupGXLink((*effect).gobj, Some(ext::cb::HSD_GObj_JObjCallback), 7 as u8, 1 as u32);
    ext::GObj_InitUserData((*effect).gobj, 8 as u8, Some(efLib_remove_user_data), effect as *mut c_void);
    {
        let mut jobj: *mut HSD_JObj = ext::HSD_JObjLoadJoint((*desc).model_desc.joint);
        if jobj.is_null() {
            ext::HSD_GObjFree((*effect).gobj);
            return null_mut();
        }
        {
            let mut kind: u8 = ext::HSD_GObj_JObjKind;
            ext::HSD_GObjObject_80390A70((*effect).gobj, kind, jobj as *mut c_void);
        }
        if melee_runtime::cabi::urem((1e+01f32 * (*desc).lifetime) as u32, 10 as u32) != (0 as u32) {
            ext::lb_80011C18(jobj, 134217728 as u32);
        }
        (*effect).lifetime = ((*desc).lifetime as u32) as u16;
        if ((*effect).lifetime as i32) != 0 {
            (*effect).lifetime = (*effect).lifetime + 1;
        }
        {
            if ((((!(*desc).model_desc.animjoint.is_null()) as i32) | ((!(*desc).model_desc.matanim_joint.is_null()) as i32)) | ((!(*desc).model_desc.shapeanim_joint.is_null()) as i32)) != 0 {
                ext::HSD_JObjAddAnimAll(jobj, (*desc).model_desc.animjoint, (*desc).model_desc.matanim_joint, (*desc).model_desc.shapeanim_joint);
                ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
                {
                    let mut temp_r5_2: i32 = super::efdata::efLib_AnimCount;
                    super::efdata::efLib_AnimCount = super::efdata::efLib_AnimCount + 1;
                    *((efLib_AnimQueue.as_mut_ptr()) as *mut *mut HSD_JObj).offset(temp_r5_2 as isize) = jobj;
                    if super::efdata::efLib_AnimCount >= 32 {
                        if 0 != 0 { (); } else { ext::OSReport(&format!("Over Anime Call\n")); panic!("efLib_Create: assertion failed"); }
                    }
                }
            }
        }
    }
    let _ = ext::HSD_GObj_SetupProc((*effect).gobj, Some(efLib_Update), 15 as u8);
    return effect;
}

/// `efLib_Create_Attach`
pub unsafe extern "C" fn efLib_Create_Attach(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach");
    let mut effect: *mut EF_Effect = efLib_Create(gfx_id as i32, gobj);
    if !effect.is_null() {
        let mut effect_jobj: *mut HSD_JObj = null_mut();
        if { effect_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj; effect_jobj }.is_null() {
            ext::HSD_GObjFree((*effect).gobj);
            return null_mut();
        } else {
            let mut translate: Vec3 = unsafe { core::mem::zeroed() };
            ext::lb_8000C1C0(effect_jobj, jobj);
            ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
            HSD_JObjSetTranslate(effect_jobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
            (*effect).attach_jobj = jobj;
        }
    }
    return effect;
}

/// `efLib_Create_AttachChild`
pub unsafe extern "C" fn efLib_Create_AttachChild(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_AttachChild");
    let mut effect: *mut EF_Effect = null_mut();
    if !{ effect = eflib_create_effect_and_attach(gfx_id as i32, gobj, jobj); effect }.is_null() {
        ext::lb_8000C290(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, jobj);
    }
    return effect;
}

/// `efLib_Create_Attach_Scale`
pub unsafe extern "C" fn efLib_Create_Attach_Scale(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Scale");
    let mut effect: *mut EF_Effect = null_mut();
    if !{ effect = eflib_create_effect_and_attach(gfx_id as i32, gobj, jobj); effect }.is_null() {
        let mut scale: Vec3 = unsafe { core::mem::zeroed() };
        HSD_JObjGetScale(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
        scale.x = { scale.z = scale.y; scale.z };
        HSD_JObjSetScale(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
    }
    return effect;
}

/// `efLib_Create_AttachChild_Scale`
pub unsafe extern "C" fn efLib_Create_AttachChild_Scale(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_AttachChild_Scale");
    let mut effect: *mut EF_Effect = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !effect.is_null() {
        ext::lb_8000C290(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, jobj);
    }
    if !effect.is_null() {
        let mut scale: Vec3 = unsafe { core::mem::zeroed() };
        HSD_JObjGetScale(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
        scale.x = { scale.z = scale.y; scale.z };
        HSD_JObjSetScale(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
    }
    return effect;
}

/// `efLib_Create_Attach_Scale_FacingDir`
pub unsafe extern "C" fn efLib_Create_Attach_Scale_FacingDir(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Scale_FacingDir");
    let mut effect: *mut EF_Effect = efLib_Create_Attach(gfx_id, gobj, jobj);
    if !effect.is_null() {
        let mut scale: Vec3 = unsafe { core::mem::zeroed() };
        HSD_JObjGetScale(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
        scale.x = { scale.z = scale.y; scale.z };
        HSD_JObjSetScale(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
    }
    if !effect.is_null() {
        (*effect).update = Some(efLib_Cb_SetRotY_FromFighterDir);
    }
    return effect;
}

/// `efLib_Create_Attach_Pos`
pub unsafe extern "C" fn efLib_Create_Attach_Pos(mut gfx_id: u32, mut gobj: *mut HSD_GObj, mut position: *mut Vec) -> *mut EF_Effect {
    melee_core::ct!("efLib_Create_Attach_Pos");
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut effect: *mut EF_Effect = null_mut();
    effect = efLib_Create(gfx_id as i32, gobj);
    if !effect.is_null() {
        jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
        ();
        HSD_JObjSetTranslate(jobj, position);
    }
    return effect;
}

/// `efLib_render_callback`
pub unsafe extern "C" fn efLib_render_callback(mut gobj: *mut HSD_GObj, mut code: i32) {
    melee_core::ct!("efLib_render_callback");
    let mut particles_code: u32 = 0;
    's50: {
        match code {
            0 => {
                particles_code = 0 as u32;
            }
            1 => {
                particles_code = 1 as u32;
            }
            2 => {
                particles_code = 2 as u32;
            }
            _ => {
                return;
            }
        }
    }
    ext::HSD_StateSetColorUpdate(1);
    if ((*gobj).gx_link as i32) == 7 {
        ext::psDispParticles(((1 | 4) | 8) as u32, particles_code);
        return;
    }
    ext::psDispParticles(2 as u32, particles_code);
}

/// `efLib_particles_proc_main`
pub unsafe extern "C" fn efLib_particles_proc_main(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_particles_proc_main");
    ext::hsd_8039CEAC((131072 | 262144) as u32);
    ext::hsd_8039EE24((131072 | 262144) as u32);
}

/// `efLib_particles_proc_aux`
pub unsafe extern "C" fn efLib_particles_proc_aux(mut gobj: *mut HSD_GObj) {
    melee_core::ct!("efLib_particles_proc_aux");
    ext::hsd_8039CEAC((65536 | 524288) as u32);
    ext::hsd_8039EE24((65536 | 524288) as u32);
}

/// `efLib_CreateGenerator`
pub unsafe extern "C" fn efLib_CreateGenerator(mut gfx_id: i32, mut pos: *mut Vec) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut linkNo: i32 = 0;
    linkNo = 0;
    's51: {
        match gfx_id {
            289 => {
                linkNo = 2;
            }
            255 | 31000 | 252 | 247 => {
                linkNo = 1;
            }
            _ => {}
        }
    }
    {
        let mut bank: u32 = (gfx_id / 1000) as u32;
        generator = ext::hsd_8039F05C(linkNo, bank as i32, gfx_id);
        if !generator.is_null() {
            (*generator).pos.x = (*pos).x;
            (*generator).pos.y = (*pos).y;
            (*generator).pos.z = (*pos).z;
        }
    }
    return generator;
}

/// `efLib_CreateGenerator_AddAppSRT`
pub unsafe extern "C" fn efLib_CreateGenerator_AddAppSRT(mut gfx_id: i32) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_AddAppSRT");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut appsrt: *mut HSD_psAppSRT = null_mut();
    generator = ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id);
    if !generator.is_null() {
        if { appsrt = (*generator).appsrt; appsrt }.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(generator, 1);
        }
        if appsrt.is_null() {
            ext::hsd_8039D4DC(generator);
            return null_mut();
        }
    }
    return generator;
}

/// `efLib_CreateGenerator_Translate_FacingDir`
pub unsafe extern "C" fn efLib_CreateGenerator_Translate_FacingDir(mut gfx_id: i32, mut translation: *mut Vec, mut direction: f32) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_Translate_FacingDir");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut appsrt: *mut HSD_psAppSRT = null_mut();
    generator = ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id);
    if !generator.is_null() {
        if { appsrt = (*generator).appsrt; appsrt }.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(generator, 1);
        }
        if appsrt.is_null() {
            ext::hsd_8039D4DC(generator);
            generator = null_mut();
        }
    }
    if !generator.is_null() {
        (*(*generator).appsrt).translate.x = (*translation).x;
        (*(*generator).appsrt).translate.y = (*translation).y;
        (*(*generator).appsrt).translate.z = (*translation).z;
        if direction < 0.0f32 {
            (*generator).kind = (*generator).kind | (262144 as u32);
            (*(*generator).appsrt).rot.y = -(3.1415927f32 / 2.0f32);
        } else {
            (*(*generator).appsrt).rot.y = 3.1415927f32 / 2.0f32;
        }
    }
    return generator;
}

/// `efLib_CreateGenerator_Attach`
pub unsafe extern "C" fn efLib_CreateGenerator_Attach(mut gfx_id: i32, mut jobj: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_Attach");
    let mut generator: *mut HSD_Generator = ext::hsd_8039EFAC(0, gfx_id / 1000, gfx_id, jobj);
    if !generator.is_null() {
        (*generator).r#type = (((*generator).r#type as i32) & (!(1 << 10))) as u16;
    }
    return generator;
}

/// `efLib_CreateGenerator_Attach_AddAppSRT`
pub unsafe extern "C" fn efLib_CreateGenerator_Attach_AddAppSRT(mut gfx_id: i32, mut jobj: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_Attach_AddAppSRT");
    let mut generator: *mut HSD_Generator = ext::hsd_8039EFAC(0, gfx_id / 1000, gfx_id, jobj);
    if !generator.is_null() {
        let mut appsrt: *mut HSD_psAppSRT = null_mut();
        if { appsrt = (*generator).appsrt; appsrt }.is_null() {
            appsrt = ext::psAddGeneratorAppSRT_begin(generator, 0);
        }
        if !appsrt.is_null() {
            (*appsrt).gp = generator;
        } else {
            ext::hsd_8039D4DC(generator);
            return null_mut();
        }
        (*generator).r#type = (((*generator).r#type as u32) & 4294965759_u32) as u16;
        (*generator).r#type = (((*generator).r#type as i32) | (1 << 11)) as u16;
    }
    return generator;
}

/// `efLib_CreateGenerator_Attach_Scale`
pub unsafe extern "C" fn efLib_CreateGenerator_Attach_Scale(mut gfx_id: i32, mut vlist: *mut crate::tu::va::Va, mut gobj: *mut HSD_GObj) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_Attach_Scale");
    let mut scale: Vec3 = unsafe { core::mem::zeroed() };
    let mut generator: *mut HSD_Generator = null_mut();
    let mut id: i32 = 0;
    id = gfx_id;
    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, id, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
    if !generator.is_null() {
        HSD_JObjGetScale(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
        (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = scale.y; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
    }
    return generator;
}

/// `efLib_CreateGenerator_AppSRT_SetScale`
pub unsafe extern "C" fn efLib_CreateGenerator_AppSRT_SetScale(mut gfx_id: i32, mut vlist: *mut crate::tu::va::Va) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetScale");
    let mut generator: *mut HSD_Generator = null_mut();
    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
    if !generator.is_null() {
        (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = *(*((*vlist).arg(8_u32) as *mut *mut f32)); (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
    }
    return generator;
}

/// `efLib_CreateGenerator_AppSRT_SetFacingDir`
pub unsafe extern "C" fn efLib_CreateGenerator_AppSRT_SetFacingDir(mut gfx_id: i32, mut vlist: *mut crate::tu::va::Va) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetFacingDir");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut direction: f32 = 0.0;
    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
    if !generator.is_null() {
        direction = *(*((*vlist).arg(8_u32) as *mut *mut f32));
        (*(*generator).appsrt).rot.y = (if direction < 0.0f32 { -(3.141592653589793f64 / (2 as f64)) } else { 3.141592653589793f64 / (2 as f64) }) as f32;
    }
    return generator;
}

/// `efLib_CreateGenerator_AppSRT_SetFacingDirScale`
pub unsafe extern "C" fn efLib_CreateGenerator_AppSRT_SetFacingDirScale(mut gfx_id: i32, mut vlist: *mut crate::tu::va::Va) -> *mut HSD_Generator {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetFacingDirScale");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut direction: f32 = 0.0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut id: i32 = 0;
    id = gfx_id;
    jobj = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, id, jobj);
    if !generator.is_null() {
        direction = *(*((*vlist).arg(8_u32) as *mut *mut f32));
        (*(*generator).appsrt).rot.y = (if direction < 0.0f32 { -(3.141592653589793f64 / (2 as f64)) } else { 3.141592653589793f64 / (2 as f64) }) as f32;
        (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = *(*((*vlist).arg(8_u32) as *mut *mut f32)); (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
    }
    return generator;
}

/// `efLib_SpawnParticleEffect`
pub unsafe extern "C" fn efLib_SpawnParticleEffect(mut bank: i32, mut gfx_id: i32, mut jobj: *mut HSD_JObj, mut flag: i32) {
    melee_core::ct!("efLib_SpawnParticleEffect");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut root: *mut HSD_JObj = jobj;
    let mut chk: i32 = 0;
    's52: {
        match gfx_id {
            19000 | 19001 | 19002 | 47000 | 47001 | 47002 => {
                {
                    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, jobj);
                    if !generator.is_null() {
                        'l53: while !HSD_JObjGetParent(root).is_null() {
                            root = HSD_JObjGetParent(root);
                        }
                        (*(*generator).appsrt).rot.y = HSD_JObjGetRotationY(root);
                    }
                    return;
                }
            }
            45 | 46 | 49 => {
                {
                    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, jobj);
                    if !generator.is_null() {
                        (*generator).r#type = (((*generator).r#type as i32) | 4096) as u16;
                    }
                    return;
                }
            }
            295 => {
                {
                    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, jobj);
                    if !generator.is_null() {
                        'l54: while !HSD_JObjGetParent(root).is_null() {
                            root = HSD_JObjGetParent(root);
                        }
                        (*(*generator).appsrt).rot.y = HSD_JObjGetRotationY(root);
                        HSD_JObjGetScale(root, core::ptr::addr_of_mut!((*(*generator).appsrt).scale) as *mut Vec);
                    }
                    return;
                }
            }
            2 | 6 | 163 | 167 | 170 | 242 | 298 | 306 | 307 => {
                {
                    eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, jobj);
                    if !generator.is_null() {
                        'l55: while !HSD_JObjGetParent(root).is_null() {
                            root = HSD_JObjGetParent(root);
                        }
                        HSD_JObjGetScale(root, core::ptr::addr_of_mut!((*(*generator).appsrt).scale) as *mut Vec);
                    }
                    return;
                }
            }
            365 | 366 | 367 | 368 | 2018 => {
                {
                    if !{ generator = eflib_generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1); generator }.is_null() {
                        'l56: while !HSD_JObjGetParent(root).is_null() {
                            root = HSD_JObjGetParent(root);
                        }
                        (*(*generator).appsrt).rot.y = HSD_JObjGetRotationY(root);
                        HSD_JObjGetTranslation(root, core::ptr::addr_of_mut!((*(*generator).appsrt).translate) as *mut Vec);
                    }
                    return;
                }
            }
            212 => {
                {
                    if !{ generator = ext::hsd_8039F05C(2, bank, gfx_id); generator }.is_null() {
                        let mut psAppSRT: *mut HSD_psAppSRT = null_mut();
                        if { psAppSRT = (*generator).appsrt; psAppSRT }.is_null() {
                            psAppSRT = ext::psAddGeneratorAppSRT_begin(generator, 1);
                        }
                        if !psAppSRT.is_null() {
                            'l57: while !HSD_JObjGetParent(root).is_null() {
                                root = HSD_JObjGetParent(root);
                            }
                            (*psAppSRT).rot.z = HSD_JObjGetRotationZ(root);
                            HSD_JObjGetTranslation(root, core::ptr::addr_of_mut!((*psAppSRT).translate) as *mut Vec);
                            HSD_JObjGetScale(root, core::ptr::addr_of_mut!((*psAppSRT).scale) as *mut Vec);
                            return;
                        }
                        ext::hsd_8039D4DC(generator);
                    }
                    return;
                }
            }
            579 => {
                {
                    if !{ generator = ext::hsd_8039F05C(2, bank, gfx_id); generator }.is_null() {
                        let mut psAppSRT: *mut HSD_psAppSRT = null_mut();
                        if { psAppSRT = (*generator).appsrt; psAppSRT }.is_null() {
                            psAppSRT = ext::psAddGeneratorAppSRT_begin(generator, 1);
                        }
                        if !psAppSRT.is_null() {
                            'l58: while !HSD_JObjGetParent(root).is_null() {
                                root = HSD_JObjGetParent(root);
                            }
                            (*psAppSRT).rot.z = HSD_JObjGetRotationZ(root);
                            HSD_JObjGetTranslation(root, core::ptr::addr_of_mut!((*psAppSRT).translate) as *mut Vec);
                            return;
                        }
                        ext::hsd_8039D4DC(generator);
                    }
                    return;
                }
            }
            227 => {
                {
                    if !{ generator = eflib_generator_add_appsrt(ext::hsd_8039F05C(0, gfx_id / 1000, gfx_id), 1); generator }.is_null() {
                        ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!((*(*generator).appsrt).translate) as *mut Vec);
                        'l59: while !HSD_JObjGetParent(root).is_null() {
                            root = HSD_JObjGetParent(root);
                        }
                        HSD_JObjGetScale(root, core::ptr::addr_of_mut!((*(*generator).appsrt).scale) as *mut Vec);
                    }
                    return;
                }
            }
            252 | 255 | 247 | 31000 => {
                chk = 1;
            }
            _ => {}
        }
    }
    if flag != 0 {
        let _ = ext::hsd_8039EFAC(chk, bank, gfx_id, root);
    } else {
        let _ = ext::hsd_8039F6CC(chk, bank, gfx_id, root);
    }
}

/// `efLib_Cb_SPtcl`
pub unsafe extern "C" fn efLib_Cb_SPtcl(mut linkNo: i32, mut bank: i32, mut gfx_id: i32, mut jobj: *mut HSD_JObj) {
    melee_core::ct!("efLib_Cb_SPtcl");
    if bank == 30 {
        ext::grLib_801C99C0(bank, gfx_id, jobj, 0);
        return;
    }
    efLib_SpawnParticleEffect(bank, gfx_id, jobj, 0);
}

/// `efLib_Cb_DPtcl`
pub unsafe extern "C" fn efLib_Cb_DPtcl(mut linkNo: i32, mut bank: i32, mut gfx_id: i32, mut jobj: *mut HSD_JObj) {
    melee_core::ct!("efLib_Cb_DPtcl");
    if bank == 30 {
        ext::grLib_801C99C0(bank, gfx_id, jobj, 1);
        return;
    }
    efLib_SpawnParticleEffect(bank, gfx_id, jobj, 1);
}

/// `efLib_Cb_ParticleRender`
pub unsafe extern "C" fn efLib_Cb_ParticleRender(mut particle: *mut HSD_Particle) {
    melee_core::ct!("efLib_Cb_ParticleRender");
    let mut translate: Vec3 = unsafe { core::mem::zeroed() };
    let mut scale: Vec3 = unsafe { core::mem::zeroed() };
    let mut rot: Quaternion = unsafe { core::mem::zeroed() };
    let mut appsrt: *mut HSD_psAppSRT = null_mut();
    if (!particle.is_null()) && (!(*particle).appsrt.is_null()) {
        translate = (*(*particle).appsrt).translate;
        scale = (*(*particle).appsrt).scale;
        rot = (*(*particle).appsrt).rot;
        let _ = ext::psRemoveParticleAppSRT(particle);
        if !{ appsrt = ext::psAddParticleAppSRT_begin(particle, 1); appsrt }.is_null() {
            (*appsrt).translate = translate;
            (*appsrt).rot = rot;
            (*appsrt).scale = scale;
        } else {
            (*particle).cmdWait = 0 as u16;
            (*particle).size = 0.0f32;
            (*particle).life = 1 as u16;
        }
    }
}

/// `efLib_Cb_PtclAppSRTHook`
pub unsafe extern "C" fn efLib_Cb_PtclAppSRTHook(mut r#gen: *mut HSD_Generator) {
    melee_core::ct!("efLib_Cb_PtclAppSRTHook");
    if (*r#gen).cmdList == ((*(*ext::ptclref_804D0E5C[0 as usize].offset(150 as isize))).cmdList.as_mut_ptr()) {
        ext::hsd_8039D1E4(r#gen, (lbl_803BF810.as_mut_ptr()) as *mut c_void);
    }
    if (*r#gen).cmdList == ((*(*ext::ptclref_804D0E5C[0 as usize].offset(151 as isize))).cmdList.as_mut_ptr()) {
        ext::hsd_8039D1E4(r#gen, (lbl_803BF810.as_mut_ptr()) as *mut c_void);
    }
    if (*r#gen).cmdList == ((*(*ext::ptclref_804D0E5C[0 as usize].offset(152 as isize))).cmdList.as_mut_ptr()) {
        ext::hsd_8039D1E4(r#gen, (lbl_803BF810.as_mut_ptr()) as *mut c_void);
    }
    if (*r#gen).cmdList == ((*(*ext::ptclref_804D0E5C[0 as usize].offset(539 as isize))).cmdList.as_mut_ptr()) {
        ext::hsd_8039D1E4(r#gen, (lbl_803BF810.as_mut_ptr()) as *mut c_void);
    }
}

/// `efLib_Cb_SetOffsetY_FromParamY`
pub unsafe extern "C" fn efLib_Cb_SetOffsetY_FromParamY(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetOffsetY_FromParamY");
    let mut sp14: Vec3 = unsafe { core::mem::zeroed() };
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    ext::lb_8000B1CC((*effect).attach_jobj, null_mut(), core::ptr::addr_of_mut!(sp14) as *mut Vec);
    sp14.y = sp14.y + (*effect).params.y;
    HSD_JObjSetTranslate(eff_jobj, core::ptr::addr_of_mut!(sp14) as *mut Vec);
}

/// `efLib_Cb_SetScale_FromParamX`
pub unsafe extern "C" fn efLib_Cb_SetScale_FromParamX(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetScale_FromParamX");
    let mut scale_x: f32 = 0.0;
    let mut scale_y: f32 = 0.0;
    let mut scale_z: f32 = 0.0;
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    scale_x = (*effect).params.x * HSD_JObjGetScaleX({ eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj; eff_jobj });
    HSD_JObjSetScaleX(eff_jobj, scale_x);
    scale_y = (*effect).params.x * HSD_JObjGetScaleY(eff_jobj);
    HSD_JObjSetScaleY(eff_jobj, scale_y);
    scale_z = (*effect).params.x * HSD_JObjGetScaleZ(eff_jobj);
    HSD_JObjSetScaleZ(eff_jobj, scale_z);
}

/// `efLib_Cb_SetRotYAndTransition`
pub unsafe extern "C" fn efLib_Cb_SetRotYAndTransition(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYAndTransition");
    let mut temp_d: f64 = 0.0;
    let mut rotate_y: f32 = 0.0;
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut HSD_JObj = null_mut();
    user_data = (*effect).user_data as *mut HSD_JObj;
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    ();
    if !user_data.is_null() {
        if (*user_data).scale.x < 0.0f32 {
            temp_d = -(3.141592653589793f64 / (2 as f64));
        } else {
            temp_d = 3.141592653589793f64 / (2 as f64);
        }
        rotate_y = temp_d as f32;
        HSD_JObjSetRotationY(eff_jobj, rotate_y);
    }
    if (((*effect).expire_flags as u8) as i32) != 0 {
        (*effect).lifetime = 11_u32 as u16;
        (*effect).update = None;
        ext::HSD_JObjReqAnimAll(eff_jobj, 65.0f32);
        return;
    }
    if ((*effect).lifetime as i32) == 1 {
        (*effect).lifetime = 6_u32 as u16;
        ext::HSD_JObjReqAnimAll(eff_jobj, 6e+01f32);
    }
}

/// `efLib_Cb_SetJObjOffsetZ`
pub unsafe extern "C" fn efLib_Cb_SetJObjOffsetZ(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetJObjOffsetZ");
    let mut eff_jobj_child: *mut HSD_JObj = null_mut();
    let mut translate_z: f32 = 0.0;
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    if eff_jobj.is_null() {
        eff_jobj_child = null_mut();
    } else {
        eff_jobj_child = (*eff_jobj).child;
    }
    translate_z = 2.0f32 + HSD_JObjGetTranslationZ(eff_jobj_child);
    HSD_JObjSetTranslateZ(eff_jobj_child, translate_z);
}

/// `efLib_Cb_SetRotY_FromFighterDir`
pub unsafe extern "C" fn efLib_Cb_SetRotY_FromFighterDir(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotY_FromFighterDir");
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    let mut half_pi: f64 = 0.0;
    let mut rotate_y: f32 = 0.0;
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    if (*(HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter)).facing_dir < 0.0f32 {
        half_pi = -(3.141592653589793f64 / (2 as f64));
    } else {
        half_pi = 3.141592653589793f64 / (2 as f64);
    }
    rotate_y = half_pi as f32;
    HSD_JObjSetRotationY(eff_jobj, rotate_y);
}

/// `efLib_Cb_SetRotYZ_FromFighter`
pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromFighter(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYZ_FromFighter");
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut half_pi: f32 = 0.0;
    let mut fighter: *mut Fighter = null_mut();
    fighter = HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter;
    jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    half_pi = (3.1415927f32 / 2.0f32) - (*fighter).mv.co.common.x4.x;
    if (*fighter).facing_dir < 0.0f32 {
        HSD_JObjSetRotationY(jobj_1, -(3.1415927f32 / 2.0f32));
        HSD_JObjSetRotationZ(jobj_1, half_pi);
    } else {
        HSD_JObjSetRotationY(jobj_1, 3.1415927f32 / 2.0f32);
        HSD_JObjSetRotationZ(jobj_1, -half_pi);
    }
}

/// `efLib_Cb_Fall_FromParamY`
pub unsafe extern "C" fn efLib_Cb_Fall_FromParamY(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_Fall_FromParamY");
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut rotate_y: f32 = 0.0;
    let mut rotate_x: f32 = 0.0;
    let mut translate_z: f32 = 0.0;
    let mut translate_y: f32 = 0.0;
    let mut translate_x: f32 = 0.0;
    let mut translate_x_adjusted: f32 = 0.0;
    jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    translate_x = HSD_JObjGetTranslationX(jobj_1);
    translate_y = HSD_JObjGetTranslationY(jobj_1);
    translate_z = HSD_JObjGetTranslationZ(jobj_1);
    rotate_x = HSD_JObjGetRotationX(jobj_1);
    rotate_y = HSD_JObjGetRotationY(jobj_1);
    (*effect).params.y = (*effect).params.y - 0.2f32;
    translate_x_adjusted = translate_x + melee_runtime::gekko::sinf(rotate_y);
    HSD_JObjSetTranslateX(jobj_1, translate_x_adjusted);
    HSD_JObjSetTranslateY(jobj_1, translate_y + (*effect).params.y);
    HSD_JObjSetTranslateZ(jobj_1, translate_z + melee_runtime::gekko::cosf(rotate_y));
    HSD_JObjSetRotationX(jobj_1, 0.5f32 + rotate_x);
}

/// `efLib_Cb_SetOffset_FromParams`
pub unsafe extern "C" fn efLib_Cb_SetOffset_FromParams(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetOffset_FromParams");
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut translate_z: f32 = 0.0;
    let mut translate_y: f32 = 0.0;
    let mut translate_x: f32 = 0.0;
    let mut translate_x_adjusted: f32 = 0.0;
    jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    translate_x = HSD_JObjGetTranslationX(jobj_1);
    translate_y = HSD_JObjGetTranslationY(jobj_1);
    translate_z = HSD_JObjGetTranslationZ(jobj_1);
    (*effect).params.y = (*effect).params.y - 0.1f32;
    translate_x_adjusted = translate_x + (*effect).params.x;
    HSD_JObjSetTranslateX(jobj_1, translate_x_adjusted);
    HSD_JObjSetTranslateY(jobj_1, translate_y + (*effect).params.y);
    HSD_JObjSetTranslateZ(jobj_1, translate_z + (*effect).params.z);
}

/// `efLib_Cb_LifetimeEndSpawn`
pub unsafe extern "C" fn efLib_Cb_LifetimeEndSpawn(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_LifetimeEndSpawn");
    if ((*effect).lifetime as i32) == 1 {
        let _ = ext::hsd_8039EFAC(0, 0, 427, (*effect).attach_jobj);
        (*effect).update = None;
        (*effect).lifetime = 39_u32 as u16;
    }
}

/// `efLib_Cb_SetScaleRotY_FromFighter`
pub unsafe extern "C" fn efLib_Cb_SetScaleRotY_FromFighter(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetScaleRotY_FromFighter");
    let mut half_pi: f64 = 0.0;
    let mut scale_1: Vec3 = unsafe { core::mem::zeroed() };
    let mut scale_2: Vec3 = unsafe { core::mem::zeroed() };
    let mut jobj_2: *mut HSD_JObj = null_mut();
    let mut rotate_y: f32 = 0.0;
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut user_data: *mut c_void = null_mut();
    let mut gobj_1: *mut HSD_GObj = null_mut();
    gobj_1 = (*effect).parent_gobj;
    jobj_1 = HSD_GObjGetHSDObj(gobj_1) as *mut HSD_JObj;
    jobj_2 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    user_data = (*gobj_1).user_data;
    user_data = (HSD_GObjGetUserData(gobj_1) as *mut Fighter) as *mut c_void;
    HSD_JObjGetScale(jobj_1, core::ptr::addr_of_mut!(scale_1) as *mut Vec);
    HSD_JObjGetScale(jobj_2, core::ptr::addr_of_mut!(scale_2) as *mut Vec);
    scale_1.x = scale_1.x * scale_2.x;
    scale_1.y = scale_1.y * scale_2.y;
    scale_1.z = scale_1.z * scale_2.z;
    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale_1) as *mut Vec);
    if (*(user_data as *mut Fighter)).facing_dir < 0.0f32 {
        half_pi = -(3.141592653589793f64 / (2 as f64));
    } else {
        half_pi = 3.141592653589793f64 / (2 as f64);
    }
    rotate_y = half_pi as f32;
    HSD_JObjSetRotationY(jobj_2, rotate_y);
}

/// `efLib_Cb_SetRotYZ_FromParamZ_FighterDir`
pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromParamZ_FighterDir(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_SetRotYZ_FromParamZ_FighterDir");
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut rotate_z: f32 = 0.0;
    jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    if (*(HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter)).facing_dir < 0.0f32 {
        HSD_JObjSetRotationY(jobj_1, -(3.1415927f32 / 2.0f32));
        rotate_z = (*effect).params.z;
        HSD_JObjSetRotationZ(jobj_1, rotate_z);
    } else {
        HSD_JObjSetRotationY(jobj_1, 3.1415927f32 / 2.0f32);
        rotate_z = -(*effect).params.z;
        HSD_JObjSetRotationZ(jobj_1, rotate_z);
    }
}

/// `efLib_Cb_ftMr_SpecialLw`
pub unsafe extern "C" fn efLib_Cb_ftMr_SpecialLw(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ftMr_SpecialLw");
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    let mut eff_child_jobj: *mut HSD_JObj = null_mut();
    let mut eff_child_nxt_jobj: *mut HSD_JObj = null_mut();
    let mut fighter: *mut Fighter = null_mut();
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    eff_child_jobj = if eff_jobj.is_null() { null_mut() } else { (*eff_jobj).child };
    eff_child_nxt_jobj = if eff_child_jobj.is_null() { null_mut() } else { (*eff_child_jobj).next };
    fighter = HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter;
    if (*fighter).motion_id == 349 {
        ext::HSD_JObjClearFlagsAll(eff_child_nxt_jobj, (1 << 4) as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(eff_child_nxt_jobj, (1 << 4) as u32);
    }
    if ((*fighter).cmd_vars[3 as usize] != 0_u32) && ((*((core::ptr::addr_of_mut!((*fighter).mv.co.common.x4.z) as *mut f32) as *mut i32)) != 0) {
        let mut rotate_z: f32 = -ext::atan2f((*fighter).coll_data.floor.normal.x, (*fighter).coll_data.floor.normal.y);
        HSD_JObjSetRotationZ(eff_jobj, rotate_z);
    } else {
        HSD_JObjSetRotationZ(eff_jobj, 0.0f32);
    }
}

/// `efLib_Cb_ftLg_SpecialLw`
pub unsafe extern "C" fn efLib_Cb_ftLg_SpecialLw(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ftLg_SpecialLw");
    let mut eff_jobj: *mut HSD_JObj = null_mut();
    let mut eff_child_jobj: *mut HSD_JObj = null_mut();
    let mut eff_child_nxt_jobj: *mut HSD_JObj = null_mut();
    let mut fighter: *mut Fighter = null_mut();
    eff_jobj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    eff_child_jobj = if eff_jobj.is_null() { null_mut() } else { (*eff_jobj).child };
    eff_child_nxt_jobj = if eff_child_jobj.is_null() { null_mut() } else { (*eff_child_jobj).next };
    fighter = HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter;
    if (*fighter).motion_id == 357 {
        ext::HSD_JObjClearFlagsAll(eff_child_nxt_jobj, (1 << 4) as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(eff_child_nxt_jobj, (1 << 4) as u32);
    }
    if ((*fighter).cmd_vars[3 as usize] != 0_u32) && ((*((core::ptr::addr_of_mut!((*fighter).mv.co.common.x4.z) as *mut f32) as *mut i32)) != 0) {
        let mut rotate_z: f32 = -ext::atan2f((*fighter).coll_data.floor.normal.x, (*fighter).coll_data.floor.normal.y);
        HSD_JObjSetRotationZ(eff_jobj, rotate_z);
    } else {
        HSD_JObjSetRotationZ(eff_jobj, 0.0f32);
    }
}

/// `efLib_Cb_ftKp_SpecialHi`
pub unsafe extern "C" fn efLib_Cb_ftKp_SpecialHi(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ftKp_SpecialHi");
    let mut rotate_z: f32 = 0.0;
    let mut fighter: *mut Fighter = null_mut();
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut jobj_2: *mut HSD_JObj = null_mut();
    jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    jobj_2 = HSD_GObjGetHSDObj((*(*effect).next).gobj) as *mut HSD_JObj;
    if (*{ fighter = HSD_GObjGetUserData((*effect).parent_gobj) as *mut Fighter; fighter }).motion_id == 359 {
        ext::HSD_JObjClearFlagsAll(jobj_2, (1 << 4) as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(jobj_2, (1 << 4) as u32);
    }
    if (((*fighter).cmd_vars[2 as usize] & (1 as u32)) != 0) && (((*fighter).mv.co.common.x10 as usize as i32) != 0) {
        rotate_z = -ext::atan2f((*fighter).coll_data.floor.normal.x, (*fighter).coll_data.floor.normal.y);
        HSD_JObjSetRotationZ(jobj_1, rotate_z);
        HSD_JObjSetRotationZ(jobj_2, rotate_z);
    } else {
        HSD_JObjSetRotationZ(jobj_1, 0.0f32);
        HSD_JObjSetRotationZ(jobj_2, 0.0f32);
    }
}

/// `efLib_Cb_ftCo_Bury`
pub unsafe extern "C" fn efLib_Cb_ftCo_Bury(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ftCo_Bury");
    ext::ftCo_800C0FCC((*effect).gobj, (*effect).parent_gobj);
}

/// `efLib_SetTevKonstColor`
pub unsafe extern "C" fn efLib_SetTevKonstColor(mut jobj: *mut HSD_JObj, mut count: i32, mut konst: u32, mut tev0: u32) {
    melee_core::ct!("efLib_SetTevKonstColor");
    let mut dobj: *mut HSD_DObj = ext::HSD_JObjGetDObj(jobj);
    let mut mobj: *mut HSD_MObj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
    let mut tobj: *mut HSD_TObj = ext::HSD_MObjGetTObj(mobj);
    'l60: while count != 0 {
        let mut next: *mut HSD_TObj = (*tobj).next;
        tobj = next;
        count = count - 1;
    }
    (*(*tobj).tev).konst.r = ((konst >> 16) & (255 as u32)) as u8;
    (*(*tobj).tev).konst.g = ((konst >> 8) & (255 as u32)) as u8;
    (*(*tobj).tev).konst.b = (konst & (255 as u32)) as u8;
    (*(*tobj).tev).tev0.r = ((tev0 >> 16) & (255 as u32)) as u8;
    (*(*tobj).tev).tev0.g = ((tev0 >> 8) & (255 as u32)) as u8;
    (*(*tobj).tev).tev0.b = (tev0 & (255 as u32)) as u8;
}

/// `efLib_SetParamAlpha`
pub unsafe extern "C" fn efLib_SetParamAlpha(mut gobj: *mut HSD_GObj, mut alpha: u8) {
    melee_core::ct!("efLib_SetParamAlpha");
    let mut idx: i32 = 0;
    let mut base: *mut EF_ParamEntry = efLib_ParamTable.as_mut_ptr();
    'b61: {
        {
            idx = 0;
            'l62: while idx < 8 {
                if (*base.offset(idx as isize)).gobj == (gobj as *mut c_void) {
                    break 'b61;
                }
                idx = idx + 1;
            }
        }
        {
            idx = 0;
            'l63: while idx < 8 {
                if (*base.offset(idx as isize)).gobj.is_null() {
                    break 'b61;
                }
                idx = idx + 1;
            }
        }
        return;
    }
    efLib_ParamTable[idx as usize].gobj = gobj as *mut c_void;
    efLib_ParamTable[idx as usize].alpha = alpha as u16;
}

/// `efLib_SetParamGfxId`
pub unsafe extern "C" fn efLib_SetParamGfxId(mut gobj: *mut HSD_GObj, mut gfx_id: i32) {
    melee_core::ct!("efLib_SetParamGfxId");
    let mut idx: i32 = 0;
    let mut base: *mut EF_ParamEntry = efLib_ParamTable.as_mut_ptr();
    'b64: {
        {
            idx = 0;
            'l65: while idx < 8 {
                if (*base.offset(idx as isize)).gobj == (gobj as *mut c_void) {
                    break 'b64;
                }
                idx = idx + 1;
            }
        }
        {
            idx = 0;
            'l66: while idx < 8 {
                if (*base.offset(idx as isize)).gobj.is_null() {
                    break 'b64;
                }
                idx = idx + 1;
            }
        }
        return;
    }
    efLib_ParamTable[idx as usize].gobj = gobj as *mut c_void;
    efLib_ParamTable[idx as usize].gfx_id = gfx_id as u16;
}

/// `efLib_Cb_ApplyStoredAlpha`
pub unsafe extern "C" fn efLib_Cb_ApplyStoredAlpha(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_ApplyStoredAlpha");
    let mut dobj: *mut HSD_DObj = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut mobj: *mut HSD_MObj = null_mut();
    let mut tobj: *mut HSD_TObj = null_mut();
    let mut hsd_obj: *mut HSD_JObj = null_mut();
    let mut entry: *mut EF_ParamEntry = null_mut();
    hsd_obj = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
    jobj = if hsd_obj.is_null() { null_mut() } else { (*hsd_obj).child };
    dobj = ext::HSD_JObjGetDObj(jobj);
    mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
    tobj = ext::HSD_MObjGetTObj(mobj);
    {
        let mut i: i32 = 0;
        entry = core::ptr::addr_of_mut!(efLib_ParamTable).cast::<EF_ParamEntry>().wrapping_offset(0 as isize) as *mut EF_ParamEntry;
        {
            i = 0;
            'l67: while i != 8 {
                if (*entry).gobj == ((*effect).parent_gobj as *mut c_void) {
                    if (efLib_ParamTable[i as usize].gfx_id as i32) == 1047 {
                        tobj = (*tobj).next;
                    }
                    (*(*tobj).tev).konst.a = efLib_ParamTable[i as usize].alpha as u8;
                    if (efLib_ParamTable[i as usize].gfx_id as i32) == 1049 {
                        let mut j: i32 = 0;
                        {
                            j = 0;
                            'l68: while j < 6 {
                                jobj = (*jobj).next;
                                dobj = ext::HSD_JObjGetDObj(jobj);
                                mobj = if !dobj.is_null() { (*dobj).mobj } else { null_mut() };
                                tobj = ext::HSD_MObjGetTObj(mobj);
                                (*(*tobj).tev).konst.a = efLib_ParamTable[i as usize].alpha as u8;
                                j = j + 1;
                            }
                        }
                    }
                    break 'l67;
                }
                entry = entry.offset(1);
                i = i + 1;
            }
        }
    }
}

/// `efLib_Cb_AccumOffset_FromParams`
pub unsafe extern "C" fn efLib_Cb_AccumOffset_FromParams(mut effect: *mut EF_Effect) {
    melee_core::ct!("efLib_Cb_AccumOffset_FromParams");
    let mut translate: Vec3 = unsafe { core::mem::zeroed() };
    ext::lb_8000B1CC((*effect).attach_jobj, null_mut(), core::ptr::addr_of_mut!(translate) as *mut Vec);
    translate.x = translate.x + (*effect).params.x;
    translate.y = translate.y + (*effect).params.y;
    translate.z = translate.z + (*effect).params.z;
    HSD_JObjSetTranslate(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(translate) as *mut Vec);
}

/// `efLib_CreateGenerator_AppSRT_SetPos`
pub unsafe extern "C" fn efLib_CreateGenerator_AppSRT_SetPos(mut gfx_id: i32, mut gobj: *mut HSD_GObj, mut jobj: *mut HSD_JObj, mut vec: *mut Vec) -> *mut EF_Effect {
    melee_core::ct!("efLib_CreateGenerator_AppSRT_SetPos");
    let mut generator: *mut HSD_Generator = null_mut();
    let mut effect: *mut EF_Effect = efLib_Create(0, gobj);
    if !effect.is_null() {
        (*effect).update = Some(efLib_Cb_AccumOffset_FromParams);
        (*effect).attach_jobj = jobj;
        (*effect).params = *vec;
        eflib_create_generator_add_appsrt(core::ptr::addr_of_mut!(generator) as *mut *mut HSD_Generator, gfx_id, HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj);
    }
    return effect;
}

/// `HSD_GObjGetHSDObj`
pub unsafe extern "C" fn HSD_GObjGetHSDObj(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
}

/// `HSD_JObjSetTranslate`
pub unsafe extern "C" fn HSD_JObjSetTranslate(mut jobj: *mut HSD_JObj, mut translate: *mut Vec) {
    melee_core::ct!("HSD_JObjSetTranslate");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetTranslate: assertion failed"); }
    if !translate.is_null() { (); } else { panic!("HSD_JObjSetTranslate: assertion failed"); }
    (*jobj).translate = *translate;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `HSD_JObjGetParent`
pub unsafe extern "C" fn HSD_JObjGetParent(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    melee_core::ct!("HSD_JObjGetParent");
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).parent;
    }
}

/// `HSD_JObjMtxIsDirty`
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    let mut result: i32 = 0;
    if !jobj.is_null() { (); } else { panic!("HSD_JObjMtxIsDirty: assertion failed"); }
    result = 0;
    if (!(((*jobj).flags & ((1 << 23) as u32)) != 0)) && (((*jobj).flags & ((1 << 6) as u32)) != 0) {
        result = 1;
    }
    return result;
}

/// `HSD_JObjGetMtxPtr`
pub unsafe extern "C" fn HSD_JObjGetMtxPtr(mut jobj: *mut HSD_JObj) -> *mut [f32; 4] {
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetMtxPtr: assertion failed"); }
    HSD_JObjSetupMatrix(jobj);
    return (*jobj).mtx.as_mut_ptr();
}

/// `HSD_JObjGetScale`
pub unsafe extern "C" fn HSD_JObjGetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    melee_core::ct!("HSD_JObjGetScale");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    *scale = (*jobj).scale;
}

/// `HSD_JObjSetScale`
pub unsafe extern "C" fn HSD_JObjSetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    melee_core::ct!("HSD_JObjSetScale");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjSetScale: assertion failed"); }
    (*jobj).scale = *scale;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetRotationY`
pub unsafe extern "C" fn HSD_JObjGetRotationY(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetRotationY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetRotationY: assertion failed"); }
    return (*jobj).rotate.y;
}

/// `HSD_JObjGetTranslation`
pub unsafe extern "C" fn HSD_JObjGetTranslation(mut jobj: *mut HSD_JObj, mut translate: *mut Vec) {
    melee_core::ct!("HSD_JObjGetTranslation");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetTranslation: assertion failed"); }
    if !translate.is_null() { (); } else { panic!("HSD_JObjGetTranslation: assertion failed"); }
    *translate = (*jobj).translate;
}

/// `HSD_JObjGetRotationZ`
pub unsafe extern "C" fn HSD_JObjGetRotationZ(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetRotationZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetRotationZ: assertion failed"); }
    return (*jobj).rotate.z;
}

/// `HSD_JObjGetScaleX`
pub unsafe extern "C" fn HSD_JObjGetScaleX(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetScaleX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScaleX: assertion failed"); }
    return (*jobj).scale.x;
}

/// `HSD_JObjSetScaleX`
pub unsafe extern "C" fn HSD_JObjSetScaleX(mut jobj: *mut HSD_JObj, mut x: f32) {
    melee_core::ct!("HSD_JObjSetScaleX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleX: assertion failed"); }
    (*jobj).scale.x = x;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetScaleY`
pub unsafe extern "C" fn HSD_JObjGetScaleY(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetScaleY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScaleY: assertion failed"); }
    return (*jobj).scale.y;
}

/// `HSD_JObjSetScaleY`
pub unsafe extern "C" fn HSD_JObjSetScaleY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetScaleY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleY: assertion failed"); }
    (*jobj).scale.y = y;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetScaleZ`
pub unsafe extern "C" fn HSD_JObjGetScaleZ(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetScaleZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScaleZ: assertion failed"); }
    return (*jobj).scale.z;
}

/// `HSD_JObjSetScaleZ`
pub unsafe extern "C" fn HSD_JObjSetScaleZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetScaleZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetScaleZ: assertion failed"); }
    (*jobj).scale.z = z;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetRotationY`
pub unsafe extern "C" fn HSD_JObjSetRotationY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetRotationY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetRotationY: assertion failed"); }
    if !(((*jobj).flags & ((1 << 17) as u32)) != 0) { (); } else { panic!("HSD_JObjSetRotationY: assertion failed"); }
    (*jobj).rotate.y = y;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetTranslationZ`
pub unsafe extern "C" fn HSD_JObjGetTranslationZ(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetTranslationZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetTranslationZ: assertion failed"); }
    return (*jobj).translate.z;
}

/// `HSD_JObjSetTranslateZ`
pub unsafe extern "C" fn HSD_JObjSetTranslateZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetTranslateZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetTranslateZ: assertion failed"); }
    (*jobj).translate.z = z;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetRotationZ`
pub unsafe extern "C" fn HSD_JObjSetRotationZ(mut jobj: *mut HSD_JObj, mut z: f32) {
    melee_core::ct!("HSD_JObjSetRotationZ");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetRotationZ: assertion failed"); }
    if !(((*jobj).flags & ((1 << 17) as u32)) != 0) { (); } else { panic!("HSD_JObjSetRotationZ: assertion failed"); }
    (*jobj).rotate.z = z;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjGetTranslationX`
pub unsafe extern "C" fn HSD_JObjGetTranslationX(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetTranslationX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetTranslationX: assertion failed"); }
    return (*jobj).translate.x;
}

/// `HSD_JObjGetTranslationY`
pub unsafe extern "C" fn HSD_JObjGetTranslationY(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetTranslationY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetTranslationY: assertion failed"); }
    return (*jobj).translate.y;
}

/// `HSD_JObjGetRotationX`
pub unsafe extern "C" fn HSD_JObjGetRotationX(mut jobj: *mut HSD_JObj) -> f32 {
    melee_core::ct!("HSD_JObjGetRotationX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetRotationX: assertion failed"); }
    return (*jobj).rotate.x;
}

/// `HSD_JObjSetTranslateX`
pub unsafe extern "C" fn HSD_JObjSetTranslateX(mut jobj: *mut HSD_JObj, mut x: f32) {
    melee_core::ct!("HSD_JObjSetTranslateX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetTranslateX: assertion failed"); }
    (*jobj).translate.x = x;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetTranslateY`
pub unsafe extern "C" fn HSD_JObjSetTranslateY(mut jobj: *mut HSD_JObj, mut y: f32) {
    melee_core::ct!("HSD_JObjSetTranslateY");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetTranslateY: assertion failed"); }
    (*jobj).translate.y = y;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetRotationX`
pub unsafe extern "C" fn HSD_JObjSetRotationX(mut jobj: *mut HSD_JObj, mut x: f32) {
    melee_core::ct!("HSD_JObjSetRotationX");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjSetRotationX: assertion failed"); }
    if !(((*jobj).flags & ((1 << 17) as u32)) != 0) { (); } else { panic!("HSD_JObjSetRotationX: assertion failed"); }
    (*jobj).rotate.x = x;
    if !(((*jobj).flags & ((1 << 25) as u32)) != 0) {
        {
            if (!jobj.is_null()) && (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
                ext::HSD_JObjSetMtxDirtySub(jobj);
            }
        }
    }
}

/// `HSD_JObjSetupMatrix`
pub unsafe extern "C" fn HSD_JObjSetupMatrix(mut jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetupMatrix");
    if (!(!jobj.is_null())) || (!(HSD_JObjMtxIsDirty(jobj) != 0)) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}
