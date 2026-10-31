//! Mechanical native port of src/melee/ef/efalt.c (
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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn hsd_8039EFAC(a0: i32, a1: i32, a2: i32, a3: *mut HSD_JObj) -> *mut HSD_Generator;
    }
}



/// `efAlt_Spawn`
pub unsafe extern "C" fn efAlt_Spawn(mut gfx_id: i32, mut gobj: *mut HSD_GObj, mut vlist_arg: *mut crate::tu::va::Va) -> *mut c_void {
    let mut effect: *mut EF_Effect = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut rotate_y: f32 = 0.0;
    let mut scale: Vec3 = unsafe { core::mem::zeroed() };
    let mut value_ptr: *mut f32 = null_mut();
    let mut ret_obj: *mut c_void = null_mut();
    ret_obj = null_mut();
    super::efdata::efLib_LoadKind = 1 as u32;
    's4: {
        let mut jl5 = false;
        match gfx_id {
            1145 => {
                ret_obj = super::eflib::efLib_CreateGenerator(1010, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1146 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = super::eflib::efLib_Create_Attach(1000_u32, gobj, jobj) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    let _ = ext::hsd_8039EFAC(0, 1, 1001, jobj);
                    break 's4;
                }
            }
            1147 => {
                ret_obj = super::eflib::efLib_CreateGenerator(1003, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1148 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale(1001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_ftMr_SpecialLw);
                }
            }
            1149 => {
                ret_obj = ext::hsd_8039EFAC(0, 1, 1008, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1150 => {
                ret_obj = ext::hsd_8039EFAC(0, 1, 1009, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1151 => {
                ret_obj = ext::hsd_8039EFAC(0, 2, 2004, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1152 => {
                ret_obj = ext::hsd_8039EFAC(0, 2, 2002, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1153 => {
                ret_obj = ext::hsd_8039EFAC(0, 2, 2003, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1154 => {
                ret_obj = super::eflib::efLib_Create_Attach(2000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetScaleRotY_FromFighter);
                }
            }
            1155 => {
                ret_obj = super::eflib::efLib_CreateGenerator(2007, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1156 => {
                ret_obj = ext::hsd_8039EFAC(0, 2, 2011, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1157 => {
                ret_obj = ext::hsd_8039EFAC(0, 2, 2014, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1158 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(2001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    break 's4;
                }
            }
            1159 => {
                ret_obj = super::eflib::efLib_Create_AttachChild_Scale(2002_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1160 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale(3000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1161 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale(3001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1162 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale(3002_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1163 => {
                ret_obj = super::eflib::efLib_Create_Attach(3003_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1164 => {
                ret_obj = super::eflib::efLib_Create_Attach(3004_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetRotYZ_FromFighter);
                }
            }
            1165 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(3008, vlist_arg) as *mut c_void;
            }
            1166 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(3005_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect = ret_obj as *mut EF_Effect;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, (3.141592653589793f64 / (2 as f64)) as f32);
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, *(*((*vlist_arg).arg(8_u32) as *mut *mut f32)));
                    }
                    break 's4;
                }
            }
            1167 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(4000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_Attach_Scale_FacingDir(4001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void) as *mut EF_Effect;
                    }
                    break 's4;
                }
            }
            1168 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach(4002_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).params.z = *(*((*vlist_arg).arg(8_u32) as *mut *mut f32));
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetRotYZ_FromParamZ_FighterDir);
                    }
                    break 's4;
                }
            }
            1169 => {
                ret_obj = super::eflib::efLib_Create_Attach(4004_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1170 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(4003_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    break 's4;
                }
            }
            1171 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(4005_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    break 's4;
                }
            }
            1172 => {
                {
                    let mut fp: *mut Fighter = null_mut();
                    let mut jobj: *mut HSD_JObj = null_mut();
                    let mut jobj_2: *mut HSD_JObj = null_mut();
                    let mut effect_1: *mut EF_Effect = null_mut();
                    let mut effect_2: *mut EF_Effect = null_mut();
                    let mut next_eff: *mut EF_Effect = null_mut();
                    let mut effect_flags: u16 = 0;
                    effect_flags = 65 as u16;
                    fp = (*gobj).user_data as *mut Fighter;
                    jobj = (*(*fp).parts.offset((FtPart_R3rdNa as i32) as isize)).joint;
                    jobj_2 = (*(*fp).parts.offset((FtPart_TransN as i32) as isize)).joint;
                    ret_obj = super::eflib::efLib_Create_AttachChild(5000_u32, gobj, jobj) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect_1 = ret_obj as *mut EF_Effect;
                        (*effect_1).update = Some(super::eflib::efLib_Cb_SetRotYAndTransition);
                        (*effect_1).lifetime = effect_flags;
                        (*effect_1).user_data = null_mut();
                        next_eff = (super::eflib::efLib_Create_Attach(5001_u32, gobj, jobj) as *mut c_void) as *mut EF_Effect;
                        (*effect_1).next = next_eff;
                        if !next_eff.is_null() {
                            effect_2 = ((*effect_1).next as *mut c_void) as *mut EF_Effect;
                            (*effect_2).update = Some(super::eflib::efLib_Cb_SetRotYAndTransition);
                            (*effect_2).lifetime = effect_flags;
                            (*effect_2).user_data = fp as *mut c_void;
                            next_eff = (super::eflib::efLib_Create_Attach(5002_u32, gobj, jobj_2) as *mut c_void) as *mut EF_Effect;
                            (*effect_2).next = next_eff;
                            if !next_eff.is_null() {
                                effect_1 = ((*effect_2).next as *mut c_void) as *mut EF_Effect;
                                (*effect_1).update = Some(super::eflib::efLib_Cb_SetRotYAndTransition);
                                (*effect_1).lifetime = effect_flags;
                                (*effect_1).user_data = fp as *mut c_void;
                            }
                        }
                    }
                    break 's4;
                }
            }
            1173 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(5003_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    break 's4;
                }
            }
            1174 => {
                ret_obj = super::eflib::efLib_Create_Attach(5004_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1175 => {
                ret_obj = super::eflib::efLib_Create_Attach(5005_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                if !ret_obj.is_null() {
                    (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetRotY_FromFighterDir);
                }
            }
            1176 => {
                ret_obj = super::eflib::efLib_Create_AttachChild(5006_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1177 => {
                ret_obj = super::eflib::efLib_Create_AttachChild(5007_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1178 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(5003, vlist_arg) as *mut c_void;
            }
            1179 => {
                ret_obj = ext::hsd_8039EFAC(0, 5, 5007, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1180 => {
                ret_obj = ext::hsd_8039EFAC(0, 5, 5013, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1181 => {
                {
                    let mut jobj: *mut HSD_JObj = null_mut();
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(5008_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        let mut gobj_jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
                        jobj = HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj;
                        HSD_JObjSetScaleX(jobj, HSD_JObjGetScaleY(gobj_jobj));
                        HSD_JObjSetScaleY(jobj, HSD_JObjGetScaleY(gobj_jobj));
                        HSD_JObjSetScaleZ(jobj, HSD_JObjGetScaleY(gobj_jobj));
                        HSD_JObjSetTranslateZ(HSD_JObjGetChild(jobj), HSD_JObjGetTranslationZ(jobj) - 6.0f32);
                        if (*(*((*vlist_arg).arg(8_u32) as *mut *mut f32))) < 0.0f32 {
                            HSD_JObjSetRotationY(jobj, (-(3.141592653589793f64 / (2 as f64))) as f32);
                            HSD_JObjSetRotationZ(jobj, -(*(*((*vlist_arg).arg(8_u32) as *mut *mut f32))));
                        } else {
                            HSD_JObjSetRotationY(jobj, (3.141592653589793f64 / (2 as f64)) as f32);
                            HSD_JObjSetRotationZ(jobj, *(*((*vlist_arg).arg(8_u32) as *mut *mut f32)));
                        }
                        (*(ret_obj as *mut EF_Effect)).update = Some(super::eflib::efLib_Cb_SetJObjOffsetZ);
                    }
                    break 's4;
                }
            }
            1182 => {
                ret_obj = super::eflib::efLib_CreateGenerator(518, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1183 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    let mut jobj_2: *mut HSD_JObj = null_mut();
                    jobj_2 = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = super::eflib::efLib_Create_Attach(32000_u32, gobj, jobj_2) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect = ret_obj as *mut EF_Effect;
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    let _ = ext::hsd_8039EFAC(0, 32, 32000, jobj_2);
                    break 's4;
                }
            }
            1184 => {
                ret_obj = super::eflib::efLib_CreateGenerator(32002, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1189 => {
                ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(41000, vlist_arg, gobj) as *mut c_void;
            }
            1190 => {
                ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(41001, vlist_arg, gobj) as *mut c_void;
            }
            1191 => {
                ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(41002, vlist_arg, gobj) as *mut c_void;
            }
            1192 => {
                ret_obj = super::eflib::efLib_CreateGenerator_Attach_Scale(41003, vlist_arg, gobj) as *mut c_void;
            }
            1185 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 34, 34004, jobj) as *mut c_void;
            }
            1186 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 34, 34002, jobj) as *mut c_void;
            }
            1187 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 34, 34003, jobj) as *mut c_void;
            }
            1188 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(34000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*(ret_obj as *mut EF_Effect)).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    break 's4;
                }
            }
            1193 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(39000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect = ret_obj as *mut EF_Effect;
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, rotate_y_2);
                        ext::HSD_JObjAnimAll(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj);
                    }
                    break 's4;
                }
            }
            1194 => {
                {
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    ret_obj = super::eflib::efLib_Create_Attach_Scale(39001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect = ret_obj as *mut EF_Effect;
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, rotate_y_2);
                        ext::HSD_JObjAnimAll(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj);
                    }
                    break 's4;
                }
            }
            1195 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(38001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_Attach_Scale_FacingDir(38000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void) as *mut EF_Effect;
                    }
                    break 's4;
                }
            }
            1196 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach_Pos(33000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
                    if !ret_obj.is_null() {
                        effect = ret_obj as *mut EF_Effect;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, (3.141592653589793f64 / (2 as f64)) as f32);
                        HSD_JObjSetRotationZ(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, *(*((*vlist_arg).arg(8_u32) as *mut *mut f32)));
                    }
                    break 's4;
                }
            }
            1197 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 36, 36000, jobj) as *mut c_void;
            }
            1198 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 46, 46000, jobj) as *mut c_void;
            }
            1199 => {
                jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                ret_obj = ext::hsd_8039EFAC(0, 46, 46001, jobj) as *mut c_void;
            }
            1200 => {
                ret_obj = super::eflib::efLib_CreateGenerator_AppSRT_SetFacingDir(46006, vlist_arg) as *mut c_void;
            }
            1201 => {
                {
                    let mut input_jobj: *mut HSD_JObj = null_mut();
                    let mut temp_d: f64 = 0.0;
                    let mut rotate_y_2: f32 = 0.0;
                    input_jobj = *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj);
                    ret_obj = super::eflib::efLib_Create_Attach(37000_u32, gobj, input_jobj) as *mut c_void;
                    if !ret_obj.is_null() {
                        value_ptr = *((*vlist_arg).arg(8_u32) as *mut *mut f32);
                        if (*value_ptr) < 0.0f32 {
                            temp_d = -(3.141592653589793f64 / (2 as f64));
                        } else {
                            temp_d = 3.141592653589793f64 / (2 as f64);
                        }
                        effect = ret_obj as *mut EF_Effect;
                        rotate_y_2 = temp_d as f32;
                        HSD_JObjSetRotationY(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, rotate_y_2);
                    }
                    let _ = ext::hsd_8039EFAC(0, 37, 37000, input_jobj);
                    break 's4;
                }
            }
            1202 => {
                ret_obj = super::eflib::efLib_CreateGenerator(37002, *((*vlist_arg).arg(8_u32) as *mut *mut Vec)) as *mut c_void;
            }
            1203 => {
                {
                    ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(47001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                    if !ret_obj.is_null() {
                        (*(ret_obj as *mut EF_Effect)).next = (super::eflib::efLib_Create_Attach_Scale_FacingDir(47000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void) as *mut EF_Effect;
                    }
                    break 's4;
                }
            }
            1204 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(20000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1205 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(20001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1206 => {
                ret_obj = super::eflib::efLib_Create_AttachChild(21000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                jl5 = true;
            }
            1207 => {
                ret_obj = super::eflib::efLib_Create_AttachChild(21001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
                jl5 = true;
            }
            1208 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(48000_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            1209 => {
                ret_obj = super::eflib::efLib_Create_Attach_Scale_FacingDir(48001_u32, gobj, *((*vlist_arg).arg(8_u32) as *mut *mut HSD_JObj)) as *mut c_void;
            }
            _ => {}
        }
        if jl5 {
            if !ret_obj.is_null() {
                effect = ret_obj as *mut EF_Effect;
                HSD_JObjGetScale(HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
                if (HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj).is_null() {
                    panic!("efAlt_Spawn: assertion failed");
                }
                HSD_JObjSetScale(HSD_GObjGetHSDObj((*effect).gobj) as *mut HSD_JObj, core::ptr::addr_of_mut!(scale) as *mut Vec);
            }
        }
    }
    'l6: while super::efdata::efLib_AnimCount != 0 {
        let mut cnt: i32 = 0;
        cnt = super::efdata::efLib_AnimCount - 1;
        super::efdata::efLib_AnimCount = cnt;
        ext::HSD_JObjAnimAll(*((super::eflib::efLib_AnimQueue.as_mut_ptr()) as *mut *mut HSD_JObj).offset(cnt as isize));
    }
    return ret_obj;
}

/// `HSD_JObjSetRotationY`
pub unsafe extern "C" fn HSD_JObjSetRotationY(mut jobj: *mut HSD_JObj, mut y: f32) {
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

/// `HSD_GObjGetHSDObj`
pub unsafe extern "C" fn HSD_GObjGetHSDObj(mut gobj: *mut HSD_GObj) -> *mut c_void {
    return (*gobj).hsd_obj;
}

/// `HSD_JObjSetRotationZ`
pub unsafe extern "C" fn HSD_JObjSetRotationZ(mut jobj: *mut HSD_JObj, mut z: f32) {
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

/// `HSD_JObjSetScaleX`
pub unsafe extern "C" fn HSD_JObjSetScaleX(mut jobj: *mut HSD_JObj, mut x: f32) {
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
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScaleY: assertion failed"); }
    return (*jobj).scale.y;
}

/// `HSD_JObjSetScaleY`
pub unsafe extern "C" fn HSD_JObjSetScaleY(mut jobj: *mut HSD_JObj, mut y: f32) {
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

/// `HSD_JObjSetTranslateZ`
pub unsafe extern "C" fn HSD_JObjSetTranslateZ(mut jobj: *mut HSD_JObj, mut z: f32) {
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

/// `HSD_JObjGetChild`
pub unsafe extern "C" fn HSD_JObjGetChild(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return null_mut();
    } else {
        return (*jobj).child;
    }
}

/// `HSD_JObjGetTranslationZ`
pub unsafe extern "C" fn HSD_JObjGetTranslationZ(mut jobj: *mut HSD_JObj) -> f32 {
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetTranslationZ: assertion failed"); }
    return (*jobj).translate.z;
}

/// `HSD_JObjGetScale`
pub unsafe extern "C" fn HSD_JObjGetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
    if !jobj.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    if !scale.is_null() { (); } else { panic!("HSD_JObjGetScale: assertion failed"); }
    *scale = (*jobj).scale;
}

/// `HSD_JObjSetScale`
pub unsafe extern "C" fn HSD_JObjSetScale(mut jobj: *mut HSD_JObj, mut scale: *mut Vec) {
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

/// `HSD_JObjMtxIsDirty`
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    let mut result: i32 = 0;
    if !jobj.is_null() { (); } else { panic!("HSD_JObjMtxIsDirty: assertion failed"); }
    result = 0;
    if (!(((*jobj).flags & ((1 << 23) as u32)) != 0)) && (((*jobj).flags & ((1 << 6) as u32)) != 0) {
        result = 1;
    }
    return result;
}
