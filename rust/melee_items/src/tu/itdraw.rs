//! Port of src/melee/it/itdraw.c: drawing an item (`it_8026EB18` model draw with an optional translated view
//! matrix, the dynamic-bone flag toggles, the per-hitbox debug-draw predicate `it_8026ECE0`, and the item draw
//! entry `it_8026EECC` with the Camera mode switch). Callees of other TUs go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_80031060() -> i32;
    fn GXSetPipelineWaitAURORA(a0: u8);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj => unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() };
    fn HSD_ClearVtxDesc() => unsafe { melee_runtime::tu::pobj::HSD_ClearVtxDesc() };
    fn HSD_GObj_80390EB8(a0: i32) -> u32 => unsafe { melee_runtime::tu::gobj::HSD_GObj_80390EB8(a0) };
    fn HSD_JObjDispAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 => unsafe { crate::tu::jobj::HSD_JObjGetFlags(a0) as _ };
    fn HSD_StateInitTev();
    fn HSD_StateInvalidate(a0: i32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) => unsafe { *(a2 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXConcat(&*(a0 as *const _), &*(a1 as *const _)) };
    fn PSMTXIdentity(a0: *mut c_void) => unsafe { *(a0 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXIdentity() };
    fn ftLib_IsItemVisible(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn ftLib_IsFighter(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_IsFighter(a0) as _ };
    fn ftLib_GetShakeOffset(a0: *mut HSD_GObj, a1: *mut Vec) -> i32;
    fn it_80272A18(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A18(a0) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn lbColl_80009F54(a0: *mut HitCapsule, a1: u32, a2: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80009F54(a0, a1, a2) as _ };
    fn lbColl_8000A10C(a0: *mut lbColl_8000A10C_arg0_t, a1: u32, a2: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_8000A10C(a0, a1, a2) as _ };
    fn lbColl_8000A244(a0: *mut HurtCapsule, a1: u32, a2: *mut c_void, a3: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_8000A244(a0, a1, a2 as _, a3) as _ };
    fn lbColl_8000A584(a0: *mut HurtCapsule, a1: u32, a2: u32, a3: *mut c_void, a4: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_8000A584(a0, a1, a2, a3 as _, a4) as _ };
    fn lbGx_8001E2F8(a0: *mut Quaternion, a1: *mut Vec, a2: *mut U8Vec4, a3: u32, a4: f32) -> i32 => unsafe { melee_runtime::tu::lbgx::lbGx_8001E2F8(a0 as _, a1 as _, a2 as _, a3, a4) as _ };
    fn lb_800149E0(a0: *mut c_void, a1: u32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub static mut it_804D5168: U8Vec4 = U8Vec4 { x: 0xFF, y: 0x40, z: 0x80, w: 0x80 };

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
unsafe fn HSD_JObjGetParent(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).parent }
}

pub unsafe extern "C" fn it_8026EB18(gobj: *mut HSD_GObj, arg1: i32, arg2: *mut Vec3) {
    melee_core::ct!("it_8026EB18");
    let mut m2: Mtx = core::mem::zeroed();
    let mptr: *mut c_void;
    if !arg2.is_null() {
        let cobj = ext::HSD_CObjGetCurrent();
        let vmtx = (&raw mut (*cobj).view_mtx) as *mut c_void;
        let mut m: Mtx = core::mem::zeroed();
        ext::PSMTXIdentity((&raw mut m) as *mut c_void);
        m[0][3] = (*arg2).x;
        m[1][3] = (*arg2).y;
        m[2][3] = (*arg2).z;
        ext::PSMTXConcat(vmtx, (&raw mut m) as *mut c_void, (&raw mut m2) as *mut c_void);
        mptr = (&raw mut m2) as *mut c_void;
    } else {
        mptr = null_mut();
    }
    // TARGET_PC: the draw waits for its GPU pipeline
    ext::GXSetPipelineWaitAURORA(1);
    ext::HSD_JObjDispAll((*gobj).hsd_obj as *mut HSD_JObj, mptr, ext::HSD_GObj_80390EB8(arg1), 0);
    ext::GXSetPipelineWaitAURORA(0);
}

pub unsafe extern "C" fn it_8026EBC8(gobj: *mut HSD_GObj, arg1: u16, arg2: *mut u8) {
    melee_core::ct!("it_8026EBC8");
    let ip = item(gobj);
    let mut cnt: u16 = 0;
    let mut index = arg2;
    while cnt < arg1 {
        let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[*index as usize];
        let jobj_parent = HSD_JObjGetParent(jobj);
        if ext::HSD_JObjGetFlags(jobj_parent) & 0x10 == 0 {
            ext::it_80272A18(jobj);
        }
        cnt = cnt.wrapping_add(1);
        index = index.add(1);
    }
}

pub unsafe extern "C" fn it_8026EC54(gobj: *mut HSD_GObj, arg1: u16, arg2: *mut u8) {
    melee_core::ct!("it_8026EC54");
    let ip = item(gobj);
    let mut cnt: u16 = 0;
    let mut index = arg2;
    while cnt < arg1 {
        let jobj = (*(*ip).xBBC_dynamicBoneTable).bones[*index as usize];
        let jobj_parent = HSD_JObjGetParent(jobj);
        if ext::HSD_JObjGetFlags(jobj_parent) & 0x10 == 0 {
            ext::it_80272A3C(jobj);
        }
        cnt = cnt.wrapping_add(1);
        index = index.add(1);
    }
}

pub unsafe extern "C" fn it_8026ECE0(gobj: *mut HSD_GObj, arg1: u32) -> u32 {
    melee_core::ct!("it_8026ECE0");
    let mut ret = 0u32;
    let ip = item(gobj);
    if (*ip).kind == It_Kind_Unk4 {
        let v = &raw mut (*ip).xDD4_itemVar.it_266F;
        if (*ip).xDAA.xDAA_flag.x0.b0() != 0
            && ((*v).x18.x0.b0() != 0 || (*v).x18.x0.b1() != 0)
            && ext::lbColl_8000A10C(&raw mut (*v).x1C, arg1, (*ip).scl) != 0
        {
            ret = 1;
        }
    } else {
        if (*ip).xDAA.xDAA_flag.x0.b6() != 0 {
            if (*ip).xDAA.xDAA_flag.x0.b2() != 0 {
                let mut idx = 0usize;
                while idx < 4 {
                    if ext::lbColl_80009F54(&raw mut (*ip).x5D4_hitboxes[idx].hit, arg1, (*ip).scl) != 0 {
                        ret = 1;
                    }
                    idx += 1;
                }
            }
            if (*ip).xDC8_word.flags.x13() == 0 && (*ip).xDAA.xDAA_flag.x0.b1() != 0 {
                if (*ip).xD0C == 0 {
                    let mut idx = 0usize;
                    while idx < (*ip).xAC8_hurtboxNum as usize {
                        if ext::lbColl_8000A244(&raw mut (*ip).xACC_itemHurtbox[idx], arg1, null_mut(), 0.0) != 0 {
                            ret = 1;
                        }
                        idx += 1;
                    }
                } else {
                    let mut idx = 0usize;
                    while idx < (*ip).xAC8_hurtboxNum as usize {
                        if ext::lbColl_8000A584(&raw mut (*ip).xACC_itemHurtbox[idx], (*ip).xD0C as u32, arg1, null_mut(), 0.0) != 0 {
                            ret = 1;
                        }
                        idx += 1;
                    }
                }
            }
        }
        if (*ip).xDAA.xDAA_flag.x0.b4() != 0
            && (*ip).xDC8_word.flags.x15() != 0
            && ext::lbGx_8001E2F8((&raw mut (*ip).xBCC_unk) as *mut Quaternion, &raw mut (*ip).pos, &raw mut it_804D5168, arg1, (*ip).facing_dir) != 0
        {
            ret = 1;
        }
        if (*ip).xDAA.xDAA_flag.x0.b3() != 0
            && (*ip).xDD0_flag.x0.b0() != 0
            && ext::lb_800149E0((&raw mut (*ip).xB54) as *mut c_void, arg1) != 0
        {
            ret = 1;
        }
    }
    ret
}

#[inline(always)]
unsafe fn vars(ip: *mut Item) -> *mut it_266F_ItemVars {
    *((&raw mut (*ip).xDD4_itemVar.it_266F) as *mut *mut it_266F_ItemVars)
}

pub unsafe extern "C" fn it_8026EECC_inline_1(gobj: *mut HSD_GObj, arg1: i32, pos: *mut Vec3) {
    melee_core::ct!("it_8026EECC_inline_1");
    let ip = item(gobj);
    (*ip).xDCF_flag.x0.set_b4(1);
    (*ip).xDCF_flag.x0.set_b5(0);
    it_8026EC54(gobj, (*vars(ip)).x0, (*vars(ip)).x4);
    it_8026EBC8(gobj, (*vars(ip)).x8, (*vars(ip)).xC);
    it_8026EB18(gobj, arg1, if (*ip).xDCF_flag.x0.b7() != 0 { pos } else { null_mut() });
    it_8026EBC8(gobj, (*vars(ip)).x0, (*vars(ip)).x4);
    it_8026EC54(gobj, (*vars(ip)).x8, (*vars(ip)).xC);
}

pub unsafe extern "C" fn it_8026EECC_inline_2(gobj: *mut HSD_GObj, arg1: i32, pos: *mut Vec3) {
    melee_core::ct!("it_8026EECC_inline_2");
    let ip = item(gobj);
    (*ip).xDCF_flag.x0.set_b4(0);
    (*ip).xDCF_flag.x0.set_b5(0);
    it_8026EB18(gobj, arg1, if (*ip).xDCF_flag.x0.b7() != 0 { pos } else { null_mut() });
}

pub unsafe extern "C" fn it_8026EECC_inline_3(gobj: *mut HSD_GObj, arg1: i32, pos: *mut Vec3) {
    melee_core::ct!("it_8026EECC_inline_3");
    let ip = item(gobj);
    (*ip).xDCF_flag.x0.set_b4(1);
    (*ip).xDCF_flag.x0.set_b5(1);
    it_8026EC54(gobj, (*vars(ip)).x0, (*vars(ip)).x4);
    it_8026EBC8(gobj, (*vars(ip)).x8, (*vars(ip)).xC);
    it_8026EB18(gobj, arg1, if (*ip).xDCF_flag.x0.b7() != 0 { pos } else { null_mut() });
    it_8026EBC8(gobj, (*vars(ip)).x0, (*vars(ip)).x4);
    it_8026EC54(gobj, (*vars(ip)).x8, (*vars(ip)).xC);
}

pub unsafe extern "C" fn it_8026EECC_inline_0(gobj: *mut HSD_GObj, pos: *mut Vec3) -> *mut Item {
    melee_core::ct!("it_8026EECC_inline_0");
    let ip = item(gobj);
    (*ip).xDCF_flag.x0.set_b7(0);
    if !(*ip).owner.is_null() && ext::ftLib_IsFighter((*ip).owner) != 0 {
        if ext::ftLib_GetShakeOffset((*ip).owner, pos) != 0 {
            (*ip).xDCF_flag.x0.set_b7(1);
        }
    } else {
        (*pos).z = 0.0;
        (*pos).y = 0.0;
        (*pos).x = 0.0;
    }
    item(gobj)
}

pub unsafe extern "C" fn it_8026EECC_inline_sw(gobj: *mut HSD_GObj, arg1: i32, pos: *mut Vec3) {
    melee_core::ct!("it_8026EECC_inline_sw");
    let ip = item(gobj);
    match ext::Camera_80031060() {
        1 => {
            if (*ip).xDCF_flag.x0.b3() != 0 {
                it_8026EECC_inline_1(gobj, arg1, pos);
                it_8026EECC_inline_2(gobj, arg1, pos);
                it_8026EECC_inline_3(gobj, arg1, pos);
            }
        }
        0 => {
            if (*ip).xDCF_flag.x0.b3() == 0 {
                it_8026EECC_inline_2(gobj, arg1, pos);
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn it_8026EECC(gobj: *mut HSD_GObj, arg1: isize) {
    melee_core::ct!("it_8026EECC");
    let ip = item(gobj);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if (*ip).xDAA.xDAA_flag.x0.b7() != 0 {
        pos.z = 0.0;
        pos.y = 0.0;
        pos.x = 0.0;
        if (*ip).xDC8_word.flags.x13() != 0 {
            if (*ip).owner.is_null() || ext::ftLib_IsFighter((*ip).owner) == 0 || ext::ftLib_IsItemVisible((*ip).owner, gobj) != 0 {
                it_8026EECC_inline_0(gobj, &raw mut pos);
                it_8026EECC_inline_sw(gobj, arg1 as i32, &raw mut pos);
            }
        } else {
            it_8026EECC_inline_sw(gobj, arg1 as i32, &raw mut pos);
        }
    }
    if it_8026ECE0(gobj, arg1 as u32) != 0 {
        ext::HSD_StateInvalidate(-1);
        ext::HSD_StateInitTev();
        ext::HSD_ClearVtxDesc();
    }
}
