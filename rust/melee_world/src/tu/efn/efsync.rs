//! Mechanical native port of src/melee/ef/efsync.c (
//! TARGET_PC variant), generated from the C by `gen/mtu/c2rs.py` (libclang AST: every
//! implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds`). Verified against
//! the C compiled natively by the state oracle `tests/tu_ef.rs` (`python rust/melee_world/gen/mtu/mk_ef.py`).
//! Callees of other TUs are `ext` hooks named like the C (unset: panic "TODO", or the native port
//! `gen/rewire/` wired). `bool` is MSL `int`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randf() -> f32;
    fn hsd_8039EFAC(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) -> *mut HSD_Generator;
    fn hsd_8039F05C(a0: i32, a1: i32, a2: i32) -> *mut HSD_Generator;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    }
}



/// `efSync_GetEffect`
pub unsafe extern "C" fn efSync_GetEffect(mut ret_obj: *mut c_void) -> *mut EF_Effect {
    melee_core::ct!("efSync_GetEffect");
    return ret_obj as *mut EF_Effect;
}

/// `efSync_GetGenerator`
pub unsafe extern "C" fn efSync_GetGenerator(mut generator: *mut HSD_Generator) -> *mut HSD_Generator {
    melee_core::ct!("efSync_GetGenerator");
    return generator;
}

/// `efSync_MexSpawn`
pub unsafe extern "C" fn efSync_MexSpawn(mut gfx_id: i32, mut gobj: *mut HSD_GObj, mut vlist: *mut crate::tu::va::Va) -> *mut c_void {
    melee_core::ct!("efSync_MexSpawn");
    let mut id: i32 = 0;
    let mut is_ptcl: i32 = 0;
    let mut owner: *mut HSD_GObj = null_mut();
    let mut r#type: i32 = super::efasync::efAsync_MexResolve(gobj, gfx_id, core::ptr::addr_of_mut!(id) as *mut i32, core::ptr::addr_of_mut!(is_ptcl) as *mut i32, core::ptr::addr_of_mut!(owner) as *mut *mut HSD_GObj);
    let mut eff: *mut EF_Effect = null_mut();
    let mut r#gen: *mut HSD_Generator = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut pos: *mut Vec = null_mut();
    let mut wpos: Vec3 = unsafe { core::mem::zeroed() };
    let mut f: f32 = 0.0;
    if r#type < 0 {
        return null_mut();
    }
    if !(is_ptcl != 0) {
        's12: {
            match r#type {
                0 => {
                    eff = super::eflib::efLib_Create_Attach_Pos(id as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec));
                    if !eff.is_null() {
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    }
                }
                1 => {
                    eff = super::eflib::efLib_Create_Attach_Pos(id as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec));
                    if !eff.is_null() {
                        HSD_JObjSetScale(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!((*(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj)).scale) as *mut Vec);
                    }
                }
                2 => {
                    eff = super::eflib::efLib_Create_Attach_Pos(id as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec));
                    if !eff.is_null() {
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, *(*((*vlist).arg(8_u32) as *mut *mut f32)));
                    }
                }
                3 => {
                    jobj = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                    ext::lb_8000B1CC(jobj, null_mut(), core::ptr::addr_of_mut!(wpos) as *mut Vec);
                    eff = super::eflib::efLib_Create_Attach_Pos(id as u32, gobj, core::ptr::addr_of_mut!(wpos) as *mut Vec);
                    if !eff.is_null() {
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, 0.0f32);
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, (*jobj).rotate.z);
                    }
                }
                4 => {
                    pos = *((*vlist).arg(8_u32) as *mut *mut Vec);
                    eff = super::eflib::efLib_Create_Attach_Pos(id as u32, gobj, pos);
                    if !eff.is_null() {
                        f = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, (if f < 0.0f32 { -(3.141592653589793f64 / (2 as f64)) } else { 3.141592653589793f64 / (2 as f64) }) as f32);
                        HSD_JObjSetScale(HSD_GObjGetHSDObj((*eff).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!((*(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj)).scale) as *mut Vec);
                    }
                }
                5 => {
                    eff = super::eflib::efLib_Create_Attach_Scale(id as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
                }
                6 => {
                    eff = super::eflib::efLib_Create_AttachChild_Scale(id as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
                }
                _ => {
                    return null_mut();
                }
            }
        }
        let __t13 = core::ptr::addr_of_mut!((*(HSD_GObjGetUserData(owner) as *mut Fighter)).pre_hitlag_cb); *__t13 = Some(super::eflib::efLib_PauseAll);
        let __t14 = core::ptr::addr_of_mut!((*(HSD_GObjGetUserData(owner) as *mut Fighter)).post_hitlag_cb); *__t14 = Some(super::eflib::efLib_ResumeAll);
        return eff as *mut c_void;
    }
    's15: {
        match r#type {
            0 => {
                pos = *((*vlist).arg(8_u32) as *mut *mut Vec);
                r#gen = ext::hsd_8039F05C(0, id / 1000, id);
                if !r#gen.is_null() {
                    (*r#gen).pos = *pos;
                }
                return r#gen as *mut c_void;
            }
            1 | 2 | 3 => {
                pos = *((*vlist).arg(8_u32) as *mut *mut Vec);
                f = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                return super::eflib::efLib_CreateGenerator_Translate_FacingDir(id, pos, f) as *mut c_void;
            }
            5 => {
                r#gen = ext::hsd_8039EFAC(0, id / 1000, id, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj));
                if !r#gen.is_null() {
                    (*r#gen).size = (*r#gen).size * (*(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj)).scale.x;
                }
                return r#gen as *mut c_void;
            }
            6 => {
                return super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(id, vlist) as *mut c_void;
            }
            7 => {
                return super::eflib::efLib_CreateGenerator_Attach_Scale(id, vlist, gobj) as *mut c_void;
            }
            _ => {
                return null_mut();
            }
        }
    }
}

/// `efSync_Spawn`
pub unsafe fn efSync_Spawn(mut gfx_id: i32, mut gobj: *mut HSD_GObj, args: &[*const c_void]) -> *mut c_void {
    melee_core::ct!("efSync_Spawn");
    let mut va_store = crate::tu::va::Va::new(args);
    let mut vlist: *mut crate::tu::va::Va = null_mut();
    let mut translate: Vec3 = unsafe { core::mem::zeroed() };
    let mut scale: Vec3 = unsafe { core::mem::zeroed() };
    let mut psAppSRT: *mut HSD_psAppSRT = null_mut();
    let mut ret_obj: *mut c_void = null_mut();
    let mut ret_eff: *mut EF_Effect = null_mut();
    let mut jobj_2: *mut HSD_JObj = null_mut();
    let mut jobj_1: *mut HSD_JObj = null_mut();
    let mut va_vec3: *mut Vec = null_mut();
    let mut half_pi: f64 = 0.0;
    let mut va_f32_1: f32 = 0.0;
    let mut rand_f32: f32 = 0.0;
    let mut rand_param_x: f32 = 0.0;
    let mut rand_param_y: f32 = 0.0;
    let mut rand_rot_y: f32 = 0.0;
    let mut fp: *mut Fighter = null_mut();
    let mut cnt_2: i32 = 0;
    let mut rand_rot_x: f32 = 0.0;
    super::efdata::efLib_AnimCount = { super::efdata::efLib_LoadKind = { ret_obj = null_mut(); ret_obj } as usize as u32; super::efdata::efLib_LoadKind } as i32;
    vlist = &raw mut va_store;
    if (gfx_id == 1145) && (super::efasync::efAsync_DatEntries[1 as usize].data.is_null()) {
        gfx_id = 1286;
    }
    if gfx_id < 592 {
        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
        return super::eflib::efLib_CreateGenerator(gfx_id, va_vec3) as *mut c_void;
    }
    if (gfx_id / 1000) == 30 {
        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
        return super::eflib::efLib_CreateGenerator(gfx_id, va_vec3) as *mut c_void;
    }
    if gfx_id < 1144 {
        return super::efasync::efAsync_Dispatch(gfx_id, gobj, vlist);
    }
    if gfx_id < 1210 {
        return super::efalt::efAlt_Spawn(gfx_id, gobj, vlist);
    }
    super::efdata::efLib_LoadKind = 1 as u32;
    'b16: {
        if (gfx_id >= 5000) && (gfx_id < 9000) {
            ret_obj = efSync_MexSpawn(gfx_id, gobj, vlist);
            break 'b16;
        }
        's17: {
            let mut jl18 = false;
            match gfx_id {
                1211 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(6000 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_AttachChild_Scale(6002 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void) as *mut EF_Effect;
                    }
                }
                1212 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(6001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_AttachChild_Scale(6003 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void) as *mut EF_Effect;
                    }
                }
                1213 => {
                    ret_obj = ext::hsd_8039EFAC(0, 7, 7000, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1214 => {
                    ret_obj = ext::hsd_8039EFAC(0, 7, 7004, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1215 => {
                    jobj_1 = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = ext::hsd_8039EFAC(0, 7, 7005, jobj_1) as *mut c_void;
                    let _ = ext::hsd_8039EFAC(0, 0, 95, jobj_1);
                }
                1216 => {
                    ret_obj = super::eflib::efLib_Create_Attach(7000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1217 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(7001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        va_f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, va_f32_1);
                    }
                }
                1218 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(7002 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        va_f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, va_f32_1);
                    }
                }
                1219 => {
                    {
                        let mut r#gen: *mut HSD_Generator = super::eflib::efLib_CreateGenerator_AddAppSRT(588_u32 as i32);
                        if !r#gen.is_null() {
                            va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                            ret_obj = r#gen as *mut c_void;
                            psAppSRT = (*r#gen).appsrt;
                            (*psAppSRT).translate = *va_vec3;
                        }
                        break 's17;
                    }
                }
                1220 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(7003 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        va_f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, va_f32_1);
                    }
                }
                1221 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(7004 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        va_f32_1 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, va_f32_1);
                    }
                }
                1222 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(8000 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                1223 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(8001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                1224 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(8002 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    ret_eff = ret_obj as *mut EF_Effect;
                    if !ret_eff.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj, va_f32_1);
                        jobj_2 = HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj;
                        ext::HSD_JObjAnimAll(jobj_2);
                    }
                }
                1225 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(8003 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    ret_eff = ret_obj as *mut EF_Effect;
                    if !ret_eff.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj, va_f32_1);
                        jobj_2 = HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj;
                        ext::HSD_JObjAnimAll(jobj_2);
                    }
                }
                1226 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(8004 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1227 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(8005 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1228 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(8006_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        jobj_1 = (*gobj).hsd_obj as *mut HSD_JObj;
                        ();
                        HSD_JObjGetScale(jobj_1, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        HSD_JObjSetScale(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetRotY_FromFighterDir);
                    }
                }
                1229 => {
                    ret_obj = super::eflib::efLib_Create_Attach(9000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1230 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(9000_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1231 | 1232 => {
                    {
                        let mut effect: *mut EF_Effect = null_mut();
                        let mut eff_1: *mut EF_Effect = null_mut();
                        let mut cnt_1: i32 = 0;
                        let mut scale_f32: f32 = 0.0;
                        scale_f32 = 1.0f32;
                        va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                        translate = *va_vec3;
                        if gfx_id == 1231 {
                            scale_f32 = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        }
                        cnt_1 = 0;
                        'w19: loop {
                            rand_f32 = ext::HSD_Randf();
                            if rand_f32 < 0.5f32 {
                                if gfx_id == 1232 {
                                    effect = super::eflib::efLib_Create_Attach_Pos(28_u32, gobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
                                } else {
                                    effect = super::eflib::efLib_Create_Attach_Pos(30_u32, gobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
                                }
                            } else if gfx_id == 1232 {
                                effect = super::eflib::efLib_Create_Attach_Pos(29_u32, gobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
                            } else {
                                effect = super::eflib::efLib_Create_Attach_Pos(31_u32, gobj, core::ptr::addr_of_mut!(translate) as *mut Vec);
                            }
                            if !effect.is_null() {
                                (*effect).update = Some(super::eflib::efLib_Cb_SetOffset_FromParams);
                                (*effect).lifetime = 50 as u16;
                                rand_rot_y = (6.283185307179586f64 * (ext::HSD_Randf() as f64)) as f32;
                                rand_rot_x = (6.283185307179586f64 * (ext::HSD_Randf() as f64)) as f32;
                                jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
                                HSD_JObjSetScaleX(jobj_1, scale_f32);
                                jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
                                HSD_JObjSetScaleY(jobj_1, scale_f32);
                                jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
                                HSD_JObjSetScaleZ(jobj_1, scale_f32);
                                jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
                                HSD_JObjSetRotationY(jobj_1, rand_rot_y);
                                jobj_1 = HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj;
                                HSD_JObjSetRotationX(jobj_1, rand_rot_x);
                                rand_param_x = melee_runtime::gekko::sinf(rand_rot_y);
                                {
                                    let mut tmp: f32 = 2.0f32 * melee_runtime::gekko::cosf(rand_rot_x);
                                    (*effect).params.x = tmp * rand_param_x;
                                }
                                (*effect).params.y = 2.0f32 * melee_runtime::gekko::sinf(rand_rot_x);
                                rand_param_y = melee_runtime::gekko::cosf(rand_rot_y);
                                {
                                    let mut tmp: f32 = 2.0f32 * melee_runtime::gekko::cosf(rand_rot_x);
                                    (*effect).params.z = tmp * rand_param_y;
                                }
                                if cnt_1 != 0 {
                                    (*eff_1).next = effect;
                                    eff_1 = ((*eff_1).next as *mut c_void) as *mut EF_Effect;
                                } else {
                                    ret_obj = { eff_1 = effect; eff_1 } as *mut c_void;
                                }
                                if { cnt_1 = cnt_1 + 1; cnt_1 } < 12 {
                                    continue 'w19;
                                }
                            }
                            break;
                        }
                        break 's17;
                    }
                }
                1262 => {
                    ret_obj = super::eflib::efLib_Create_Attach(10000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1263 => {
                    ret_obj = super::eflib::efLib_Create_Attach(10001_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1264 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(10002 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1237 => {
                    ret_obj = ext::hsd_8039EFAC(0, 11, 11000, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1238 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(11000 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1239 => {
                    ret_obj = ext::hsd_8039EFAC(0, 11, 11001, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1240 => {
                    {
                        let mut generator: *mut HSD_Generator = efSync_GetGenerator(super::eflib::efLib_CreateGenerator_AddAppSRT(97_u32 as i32));
                        if !generator.is_null() {
                            va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                            psAppSRT = (*generator).appsrt;
                            (*psAppSRT).translate = *va_vec3;
                            HSD_JObjGetScale({ jobj_2 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj; jobj_2 }, core::ptr::addr_of_mut!(scale) as *mut Vec);
                            ret_obj = generator as *mut c_void;
                            (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = scale.y; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
                            (*(*generator).appsrt).rot.y = (3.141592653589793f64 / (2 as f64)) as f32;
                        }
                        break 's17;
                    }
                }
                1241 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(12000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        ret_eff = ret_obj as *mut EF_Effect;
                        jobj_2 = HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj;
                        HSD_JObjSetRotationY(jobj_2, va_f32_1);
                        HSD_JObjGetScale({ jobj_2 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj; jobj_2 }, core::ptr::addr_of_mut!(scale) as *mut Vec);
                        jobj_2 = HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj;
                        HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                    }
                }
                1242 => {
                    {
                        let mut eff_1: *mut EF_Effect = null_mut();
                        jobj_2 = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                        ret_obj = super::eflib::efLib_Create_Attach_Scale(12001 as u32, gobj, jobj_2) as *mut c_void;
                        if !ret_obj.is_null() {
                            eff_1 = super::eflib::efLib_Create_Attach_Scale(12002 as u32, gobj, jobj_2);
                            (*(ret_obj as *mut EF_Effect)).next = (eff_1 as *mut c_void) as *mut EF_Effect;
                            if !eff_1.is_null() {
                                (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ftKp_SpecialHi);
                            }
                        }
                        break 's17;
                    }
                }
                1243 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(12005, vlist, gobj) as *mut c_void;
                }
                1244 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(12006, vlist, gobj) as *mut c_void;
                }
                1245 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(12007, vlist, gobj) as *mut c_void;
                }
                1246 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(12008, vlist, gobj) as *mut c_void;
                }
                1247 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(323, vlist, gobj) as *mut c_void;
                }
                1248 => {
                    ret_obj = super::eflib::efLib_Create_Attach(13000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1249 => {
                    {
                        let mut cursor: *mut HSD_JObj = null_mut();
                        ret_obj = super::eflib::efLib_Create(13001, gobj) as *mut c_void;
                        if !ret_obj.is_null() {
                            jobj_1 = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                            (*(ret_obj as *mut EF_Effect)).attach_jobj = jobj_1;
                            (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetOffsetY_FromParamY);
                            cursor = jobj_1;
                            'l20: while !HSD_JObjGetParent(cursor).is_null() {
                                jobj_1 = HSD_JObjGetParent(cursor);
                                cursor = jobj_1;
                            }
                            HSD_JObjGetScale(cursor, core::ptr::addr_of_mut!(scale) as *mut Vec);
                            (*(ret_obj as *mut EF_Effect)).params = *(*((*vlist).arg(8_u32) as *mut *mut Vec));
                            let __t21 = core::ptr::addr_of_mut!((*efSync_GetEffect(ret_obj)).params.y); *__t21 = *__t21 * scale.y;
                        }
                        break 's17;
                    }
                }
                1250 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(13002_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1251 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(394_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1252 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(404_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1253 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 381, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1254 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 382, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1255 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(406_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1256 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(13003_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1257 => {
                    ret_obj = ext::hsd_8039EFAC(0, 14, 14000, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1258 => {
                    ret_obj = ext::hsd_8039EFAC(0, 14, 14001, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1259 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(14006, vlist) as *mut c_void;
                }
                1260 => {
                    ret_obj = ext::hsd_8039EFAC(0, 14, 14007, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1261 => {
                    ret_obj = super::eflib::efLib_Create_AttachChild(14000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1233 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 100, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1234 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(15000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1235 => {
                    jobj_2 = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_AddAppSRT(370, jobj_2) as *mut c_void;
                    if !ret_obj.is_null() {
                        let _ = super::eflib::efLib_CreateGenerator_Attach_AddAppSRT(371, jobj_2);
                    }
                }
                1236 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach(286, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1265 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(16000, vlist) as *mut c_void;
                }
                1266 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(16000 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                1267 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(16001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                1268 => {
                    ret_obj = super::eflib::efLib_Create_AttachChild(17000_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    jl18 = true;
                }
                1269 => {
                    ret_obj = super::eflib::efLib_Create_AttachChild(17001_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    jl18 = true;
                }
                1270 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(17002 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1271 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(17003 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1272 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(110, vlist, gobj) as *mut c_void;
                }
                1273 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(456, vlist, gobj) as *mut c_void;
                }
                1274 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(358, vlist, gobj) as *mut c_void;
                }
                1275 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 113, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1278 => {
                    ret_obj = super::eflib::efLib_Create_Attach(17004_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1279 | 1280 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(17005 as u32, gobj, (*(*((*gobj).user_data as *mut Fighter)).parts.offset(1 as isize)).joint) as *mut c_void;
                }
                1281 => {
                    fp = HSD_GObjGetUserData(gobj) as *mut Fighter;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(17006 as u32, gobj, (*(*((*gobj).user_data as *mut Fighter)).parts.offset(1 as isize)).joint) as *mut c_void;
                    if !ret_obj.is_null() {
                        if (*fp).facing_dir < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetRotationY(jobj_2, va_f32_1);
                        (*(ret_obj as *mut EF_Effect)).attach_jobj = (*(*fp).parts.offset(85 as isize)).joint;
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_LifetimeEndSpawn);
                        (*(ret_obj as *mut EF_Effect)).lifetime = 6 as u16;
                    }
                }
                1282 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 422, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1276 => {
                    ret_obj = ext::hsd_8039EFAC(0, 17, 17005, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1277 => {
                    ret_obj = ext::hsd_8039EFAC(0, 17, 17009, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1283 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(106_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1284 => {
                    {
                        let mut generator: *mut HSD_Generator = efSync_GetGenerator(super::eflib::efLib_CreateGenerator_AddAppSRT(109_u32 as i32));
                        if !generator.is_null() {
                            va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                            psAppSRT = (*generator).appsrt;
                            (*psAppSRT).translate = *va_vec3;
                            HSD_JObjGetScale({ jobj_2 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj; jobj_2 }, core::ptr::addr_of_mut!(scale) as *mut Vec);
                            ret_obj = generator as *mut c_void;
                            (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = scale.y; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
                        }
                        break 's17;
                    }
                }
                1285 => {
                    {
                        let mut generator: *mut HSD_Generator = efSync_GetGenerator(super::eflib::efLib_CreateGenerator_AddAppSRT(121_u32 as i32));
                        if !generator.is_null() {
                            va_vec3 = *((*vlist).arg(8_u32) as *mut *mut Vec);
                            psAppSRT = (*generator).appsrt;
                            (*psAppSRT).translate = *va_vec3;
                            HSD_JObjGetScale({ jobj_2 = (*gobj).hsd_obj as *mut HSD_JObj; jobj_2 }, core::ptr::addr_of_mut!(scale) as *mut Vec);
                            ret_obj = generator as *mut c_void;
                            (*(*generator).appsrt).scale.x = { (*(*generator).appsrt).scale.y = { (*(*generator).appsrt).scale.z = scale.y; (*(*generator).appsrt).scale.z }; (*(*generator).appsrt).scale.y };
                        }
                        break 's17;
                    }
                }
                1286 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(18008_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1287 => {
                    jobj_1 = *((*vlist).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = super::eflib::efLib_Create_Attach(18000_u32, gobj, jobj_1) as *mut c_void;
                    if !ret_obj.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetRotationY(jobj_2, va_f32_1);
                    }
                    let _ = ext::hsd_8039EFAC(0, 18, 18000, jobj_1);
                }
                1288 => {
                    ret_obj = super::eflib::efLib_CreateGenerator(18002_u32 as i32, *((*vlist).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                }
                1289 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(18001 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ftLg_SpecialLw);
                    }
                }
                1290 => {
                    ret_obj = ext::hsd_8039EFAC(0, 0, 95, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1291 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(19000 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_Attach_Scale_FacingDir(19001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void) as *mut EF_Effect;
                    }
                }
                1292 => {
                    ret_obj = super::eflib::efLib_Create_Attach(19002_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).params.z = *(*((*vlist).arg(8_u32) as *mut *mut f32));
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetRotYZ_FromParamZ_FighterDir);
                    }
                }
                1293 => {
                    ret_obj = super::eflib::efLib_Create_Attach(19003_u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                }
                1294 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(19004 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetRotationY(jobj_2, va_f32_1);
                    }
                }
                1295 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(19005 as u32, gobj, *((*vlist).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        if (*(*((*vlist).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            half_pi = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            half_pi = 3.141592653589793f64 / (2 as f64);
                        }
                        va_f32_1 = half_pi as f32;
                        jobj_2 = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetRotationY(jobj_2, va_f32_1);
                    }
                }
                1296 => {
                    ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(49000, vlist) as *mut c_void;
                }
                1297 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(49000 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                1298 => {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(49001 as u32, gobj, (*((*vlist).arg(8_u32) as *mut *mut c_void)) as *mut HSD_JObj) as *mut c_void;
                }
                _ => {}
            }
            if jl18 {
                if !ret_obj.is_null() {
                    HSD_JObjGetScale({ jobj_2 = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj; jobj_2 }, core::ptr::addr_of_mut!(scale) as *mut Vec);
                    ret_eff = ret_obj as *mut EF_Effect;
                    if (HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj).is_null() {
                        panic!("efSync_Spawn: assertion failed");
                    }
                    jobj_2 = HSD_GObjGetHSDObj((*ret_eff).gobj) as *mut HSD_JObj;
                    HSD_JObjSetScale(jobj_2, core::ptr::addr_of_mut!(scale) as *mut Vec);
                }
            }
        }
    }
    'l22: while super::efdata::efLib_AnimCount != 0 {
        cnt_2 = super::efdata::efLib_AnimCount - 1;
        super::efdata::efLib_AnimCount = cnt_2;
        ext::HSD_JObjAnimAll(*((super::eflib::efLib_AnimQueue.as_mut_ptr()) as *mut *mut HSD_JObj).offset(cnt_2 as isize));
    }
    ();
    return ret_obj;
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

/// `HSD_GObjGetHSDObj`
pub unsafe extern "C" fn HSD_GObjGetHSDObj(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetHSDObj");
    return (*gobj).hsd_obj;
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

/// `HSD_GObjGetUserData`
pub unsafe extern "C" fn HSD_GObjGetUserData(mut gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjGetUserData");
    return (*gobj).user_data;
}

/// `HSD_JObjGetScale`
pub unsafe extern "C" fn HSD_JObjGetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    melee_core::ct!("HSD_JObjGetScale");
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    *scale = (*jobj).scale;
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
