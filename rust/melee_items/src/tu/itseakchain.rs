//! Port of src/melee/it/kinds/itseakchain.c (+ itseakchain.static.h): Sheik's chain (logic table 54).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itseakchain.rs.
//!
//! Fused sites (`fmadds`, f64 multiply + f64 add + f32 round): every `a * b + c` / `a * b - c` of the
//! link physics (`pos = dir * len + origin`, `vel += history * gain`, `-x18 * scale - x28`, the
//! `-(x18 * s - vel.y)` of `it_802BBED0`, and `0.5f * x0 - 1.0f`). Uninitialised locals are 0 (the
//! shipping build's auto-var-init); `it_802BAF2C` returns the low 32 bits of a pointer, as the 64-bit C does.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::kinds_inl::*;

// // BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void => unsafe { melee_runtime::tu::objalloc::HSD_ObjAlloc(a0 as _) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) => unsafe { *(a2 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXConcat(&*(a0 as *const _), &*(a1 as *const _)) };
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void) => unsafe { *(a1 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXCopy(&*(a0 as *const _)) };
    fn PSMTXIdentity(a0: *mut c_void) => unsafe { *(a0 as *mut melee_types::Mtx) = melee_runtime::tu::mtx_pc::PSMTXIdentity() };
    fn ftColl_8007AFF8(a0: *mut HSD_GObj);
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_GetMotionId(a0) };
    fn ftSk_SpecialS_80110E4C(a0: *mut HSD_GObj);
    fn ftSk_SpecialS_80110F58(a0: *mut HSD_GObj) -> f32;
    fn ftSk_SpecialS_80110F64(a0: *mut HSD_GObj) -> f32;
    fn ftSk_SpecialS_UpdateHitboxes(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80275328(a0: *mut HSD_GObj, a1: *mut ItemLink) => unsafe { crate::itlib3::it_80275328(a0, &mut *a1) };
    fn it_802A2474(a0: *mut c_void) => unsafe { crate::tu::itlinkhookshot::it_802A2474(a0) };
    fn it_802A24A0(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itlinkhookshot::it_802A24A0(a0, a1) };
    fn it_802A24D0(a0: *mut ItemLink, a1: f32) => unsafe { crate::tu::itlinkhookshot::it_802A24D0(a0, a1) };
    fn it_802A3C98(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> f32 => unsafe { crate::tu::itlinkhookshot::it_802A3C98(a0 as _, a1 as _, a2 as _) };
    fn it_802A43B8(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43b8(&mut *a0) };
    fn it_802A43EC(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43ec(&mut *a0) };
    fn it_802A4420(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a4420(&mut *a0) };
    fn it_802A6DC8(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { crate::tu::itlinkhookshot::it_802A6DC8(a0, a1 as _, a2 as _) };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn mpColl_80048844(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_80048844(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn it_802A2474(a0: *mut c_void) { super::it_802A2474(a0) }
        pub unsafe extern "C" fn it_802A24A0(a0: *mut HSD_GObj, a1: i32) { super::it_802A24A0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::item::item_link_alloc_data;
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `HSD_JObjSetupMatrix` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_JObjCopyMtx` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *mut Mtx) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!mtx.is_null(), "mtx");
    ext::PSMTXCopy(mtx as *mut c_void, (*jobj).mtx.as_mut_ptr() as *mut c_void);
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `Item_InitLinkMtx` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitLinkMtx(m: *mut Mtx, z: f32) {
    ext::PSMTXIdentity(m as *mut c_void);
    (*m)[0][3] = 0.0;
    (*m)[1][3] = 0.0;
    (*m)[2][3] = z;
}

/// `Item_RetractChain` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_RetractChain(
    link: *mut ItemLink,
    pos: *mut Vec3,
    target_dist: f32,
    max_dist: *const f32,
    next_out: *mut *mut ItemLink,
    cur_out: *mut *mut ItemLink,
    remaining_out: *mut f32,
) {
    let mut dir: Vec3 = core::mem::zeroed();

    let mut cur = link;
    let mut next = (*link).prev;

    while !next.is_null() && (*cur).x2C_b0() == 0 {
        cur = next;
        next = (*next).prev;
    }

    let mut distance = ext::it_802A3C98(&mut (*cur).pos, pos, &mut dir);

    while !next.is_null() && target_dist > distance {
        (*cur).set_x2C_b0(0);
        distance = ext::it_802A3C98(&mut (*next).pos, pos, &mut dir);
        cur = next;
        next = (*next).prev;
    }

    let mut remaining = distance - target_dist;
    let limit = *max_dist;
    if remaining > limit {
        remaining = limit;
    }
    *next_out = next;
    *cur_out = cur;
    *remaining_out = remaining;
}

/// `ItemStateTable it_803F7438[]`
pub static mut it_803F7438: [ItemStateTable; 5] = [
    st_row(-1, pred_b!(itSeakchain_UnkMotion4_Anim), None, None),
    st_row(-1, pred_b!(itSeakchain_UnkMotion4_Anim), None, None),
    st_row(-1, pred_b!(itSeakchain_UnkMotion4_Anim), None, None),
    st_row(-1, pred_b!(itSeakchain_UnkMotion4_Anim), None, None),
    st_row(-1, pred_b!(itSeakchain_UnkMotion4_Anim), None, None),
];

/// `const Vec3 it_803B8680 = { 0 }`
pub static it_803B8680: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
/// `const Vec3 it_803B868C = { 0.0f, 0.0f, 0.1f }`
pub static it_803B868C: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.1 };

pub unsafe extern "C" fn it_802BAEEC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAEEC");
    ext::it_8026B724(gobj);
}

pub unsafe extern "C" fn it_802BAF0C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BAF0C");
    ext::it_8026B73C(gobj);
}

#[inline]
unsafe fn it_802BAF2C_Load_x64(ip: *mut Item) -> *mut HSD_JObj {
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    ext::HSD_JObjLoadJoint((*attrs).x64_joint)
}

#[inline]
unsafe fn it_802BAF2C_Load_x68(ip: *mut Item) -> *mut HSD_JObj {
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    ext::HSD_JObjLoadJoint((*attrs).x68_joint)
}

pub unsafe extern "C" fn it_802BAF2C(ip: *mut Item, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("it_802BAF2C");
    let mut prev_link: *mut ItemLink;
    let mut head_link: *mut ItemLink = null_mut();
    let mut tail_link: *mut ItemLink = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    let zero: Vec3 = it_803B8680;

    prev_link = null_mut();
    let mut i: i32 = 0;
    while i < (*attrs).x0 {
        let link_gobj = ext::GObj_Create(7, 0xA, 0);

        if link_gobj.is_null() {
            while !prev_link.is_null() {
                ext::HSD_GObjFree((*prev_link).gobj);
                prev_link = (*prev_link).next;
            }
            return 0;
        }

        let link = ext::HSD_ObjAlloc(&raw mut ext::item_link_alloc_data) as *mut ItemLink;
        ext::it_80275328(link_gobj, link);
        ext::GObj_InitUserData(link_gobj, 6, Some(ext::cb::it_802A2474), link as *mut c_void);

        if i == 0 {
            (*link).next = null_mut();
            head_link = link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero;
            (*link).pos = zero;
            (*link).set_x2C_b0(0);
            ext::it_802A24D0(link, 1.0);
            let joint = it_802BAF2C_Load_x64(ip);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, joint as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::it_802A24A0), 6, 0);
        } else if i == (*attrs).x0 - 1 {
            (*prev_link).prev = link;
            (*link).prev = null_mut();
            tail_link = link;
            (*link).next = prev_link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero;
            (*link).pos = zero;
            (*link).set_x2C_b0(0);
            ext::it_802A24D0(link, 1.0);
            {
                let joint = it_802BAF2C_Load_x68(ip);
                ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, joint as *mut c_void);
            }
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::HSD_GObj_JObjCallback), 6, 0);
            result = (*link_gobj).hsd_obj as *mut HSD_GObj;
        } else {
            (*prev_link).prev = link;
            (*link).next = prev_link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero;
            (*link).pos = zero;
            (*link).set_x2C_b0(0);
            ext::it_802A24D0(link, 1.0);
            let joint = it_802BAF2C_Load_x64(ip);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, joint as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::it_802A24A0), 6, 0);
        }
        prev_link = link;
        i += 1;
    }
    (*ip).xDD4_itemVar.seakchain.x0 = tail_link;
    (*ip).xDD4_itemVar.seakchain.x4 = head_link;
    result as usize as i32
}

/// `inlineA0` (static inline)
#[inline]
pub unsafe fn inlineA0(gobj: *mut Item_GObj) {
    melee_core::ct!("inlineA0");
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !(*ip).owner.is_null() {
            ext::ftSk_SpecialS_80110E4C((*ip).owner);
        }
        (*ip).owner = null_mut();
        (*ip).xDD4_itemVar.seakchain.parent_gobj = null_mut();
        {
            let mut cur = (*ip).xDD4_itemVar.seakchain.x0;
            while !cur.is_null() {
                let cur_gobj = (*cur).gobj;
                cur = (*cur).next;
                ext::HSD_GObjFree(cur_gobj);
            }
        }
    }
}

pub unsafe extern "C" fn it_802BB20C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BB20C");
    if !GET_ITEM(gobj).is_null() {
        inlineA0(gobj);
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn itSeakChain_Spawn(parent_gobj: *mut Fighter_GObj, arg1: *mut Point3d, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("itSeakChain_Spawn");
    let mut spawn: SpawnItem = core::mem::zeroed();

    let fp = (*parent_gobj).user_data as *mut Fighter;
    spawn.kind = It_Kind_Seak_Chain;
    Item_InitSpawnPosition(&mut spawn, arg1, false);
    Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).xDD4_itemVar.seakchain.parent_gobj = parent_gobj;
        (*ip).xDD4_itemVar.seakchain.x10 = 0;
        (*ip).xDD4_itemVar.seakchain.x14 = 0;
        let mut i = 0;
        while i < 0xF {
            (*ip).xDD4_itemVar.seakchain.history[i].z = 0.0;
            (*ip).xDD4_itemVar.seakchain.history[i].y = 0.0;
            (*ip).xDD4_itemVar.seakchain.history[i].x = 0.0;
            i += 1;
        }
        ext::it_80272A3C(GET_JOBJ(gobj));
        it_802BAF2C(ip, (*(*fp).parts.add(FtPart_L3rdNa as usize)).joint);
        ext::Item_8026AB54(gobj, parent_gobj, FtPart_L3rdNa as u32);
    }
    gobj
}

pub unsafe extern "C" fn fn_802BB428(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802BB428");
    it_802BCA30(GET_ITEM(gobj));
}

pub unsafe extern "C" fn fn_802BB44C(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802BB44C");
    let mut vec: Vec3 = core::mem::zeroed();
    let mut mtx: Mtx = core::mem::zeroed();

    let temp_r30 = GET_ITEM(gobj);
    let link = (*temp_r30).xDD4_itemVar.seakchain.x0;
    let sa = (*(*temp_r30).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    Item_InitLinkMtx(&mut mtx, 0.1);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    vec.x = mtx[0][3];
    vec.y = mtx[1][3];
    vec.z = mtx[2][3];
    match it_802BBD64(link, &mut vec, sa) {
        1 => {
            (*link).vel.x *= -(*sa).x58;
            it_802BCF2C(gobj);
        }
        2 => {
            it_802BCED4(gobj);
        }
        _ => {}
    }
    it_802BCB88(temp_r30, &mut vec);
}

pub unsafe extern "C" fn fn_802BB574(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802BB574");
    let mut vec: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    let mut mtx: Mtx = core::mem::zeroed();
    let link = (*ip).xDD4_itemVar.seakchain.x0;

    Item_InitLinkMtx(&mut mtx, 0.1);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    vec.x = mtx[0][3];
    vec.y = mtx[1][3];
    vec.z = mtx[2][3];
    match it_802BBED0(link, &mut vec, sa) {
        1 => {
            (*link).vel.x *= -(*sa).x58;
        }
        2 => {
            it_802BCED4(gobj);
        }
        _ => {}
    }
    it_802BCB88(ip, &mut vec);
}

pub unsafe extern "C" fn fn_802BB694(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802BB694");
    let mut vec: Vec3 = core::mem::zeroed();
    let mut mtx: Mtx = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let link = (*ip).xDD4_itemVar.seakchain.x4;
    Item_InitLinkMtx(&mut mtx, 0.1);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    vec.x = mtx[0][3];
    vec.y = mtx[1][3];
    vec.z = mtx[2][3];
    it_802BC080(link, &mut vec, ip);
    it_802BCB88(ip, &mut vec);
}

pub unsafe extern "C" fn fn_802BB784(gobj: *mut Item_GObj) {
    melee_core::ct!("fn_802BB784");
    let mut vec: Vec3 = core::mem::zeroed();
    let mut mtx: Mtx = core::mem::zeroed();

    let ip = GET_ITEM(gobj);
    let link = (*ip).xDD4_itemVar.seakchain.x4;
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    Item_InitLinkMtx(&mut mtx, 0.1);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    vec.x = mtx[0][3];
    vec.y = mtx[1][3];
    vec.z = mtx[2][3];
    if it_802BC94C(link, &mut vec, sa, (*sa).x54) {
        it_2725_Logic54_PickedUp(gobj);
    }
    it_802BCB88(ip, &mut vec);
}

/// `notInSpecialS` (static)
pub unsafe fn notInSpecialS(gobj: *mut Fighter_GObj) -> bool {
    melee_core::ct!("notInSpecialS");
    if !gobj.is_null() {
        let msid = ext::ftLib_GetMotionId(gobj);
        if msid >= ftSk_MS_SpecialSStart as i32 && msid <= ftSk_MS_SpecialAirSEnd as i32 {
            return false;
        }
        return true;
    }
    true
}

pub unsafe extern "C" fn itSeakchain_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSeakchain_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    if notInSpecialS((*ip).owner) {
        inlineA0(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn it_802BB938(link: *mut ItemLink, arg1: i32, arg2: f32) -> i32 {
    melee_core::ct!("it_802BB938");
    let coll: *mut CollData = &mut (*link).coll_data;

    if !(*link).next.is_null() && (*(*link).next).x2C_b0() != 0 {
        (*coll).last_pos = (*(*link).next).pos;
    } else {
        (*coll).last_pos = (*coll).cur_pos;
    }
    (*coll).cur_pos = (*link).pos;
    if ext::mpColl_80048844(coll) != 0 {
        if (*link).x2C_b1() == 0 && arg1 != 0 && (*link).x2C_b2() == 0 {
            ext::lbAudioAx_800237A8(0x41F45, 0x7F, 0x40);
            (*link).set_x2C_b2(0xF); // weird; gets truncated to 0x1
        }
        (*link).set_x2C_b1(1);
    } else {
        (*link).set_x2C_b1(0);
        if (*link).x2C_b2() != 0 {
            // `x2C_b2--` on a 1-bit field
            let v = (*link).x2C_b2();
            (*link).set_x2C_b2(v.wrapping_sub(1));
        }
    }
    if (*coll).env_flags & 0x18000 != 0 {
        (*coll).cur_pos.x = (*link).pos.x;
    } else if (*coll).env_flags & 0xFFF != 0 {
        if !(*link).next.is_null() && (*(*link).next).x2C_b0() != 0 && !(*link).prev.is_null() {
            (*coll).cur_pos.y += arg2;
        }
    }
    (*link).pos = (*coll).cur_pos;
    ((*coll).env_flags & 0x18FFF) as i32
}

pub unsafe extern "C" fn it_802BBAEC(link: *mut ItemLink, arg1: i32, arg2: f32) -> i32 {
    melee_core::ct!("it_802BBAEC");
    it_802BB938(link, arg1, arg2)
}

pub unsafe extern "C" fn it_802BBB0C(mut link: *mut ItemLink, offset: *mut Vec3, sa: *mut itSeakChain_Attrs, scale: f32) {
    melee_core::ct!("it_802BBB0C");
    let mut pos: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;

    ext::it_802A3C98(&mut (*link).pos, offset, &mut pos);
    (*link).pos.x = fmadds(pos.x, scale, (*offset).x);
    (*link).pos.y = fmadds(pos.y, scale, (*offset).y);
    (*link).pos.z = fmadds(pos.z, scale, (*offset).z);
    while !prev.is_null() {
        (*prev).vel.y -= (*sa).x18;
        ext::it_802A4420(prev);
        ext::it_802A43EC(prev);
        it_802BB938(prev, 0, (*sa).x4);
        if ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut pos) > (*sa).x4 {
            (*prev).pos.x = fmadds(pos.x, (*sa).x4, (*link).pos.x);
            (*prev).pos.y = fmadds(pos.y, (*sa).x4, (*link).pos.y);
            (*prev).pos.z = fmadds(pos.z, (*sa).x4, (*link).pos.z);
        }
        link = prev;
        prev = (*prev).prev;
    }
}

pub unsafe extern "C" fn it_802BBC38(mut link: *mut ItemLink, offset: *mut Vec3, sa: *mut itSeakChain_Attrs, scale: f32) {
    melee_core::ct!("it_802BBC38");
    let mut origin: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;

    ext::it_802A3C98(&mut (*link).pos, offset, &mut origin);
    (*link).pos.x = fmadds(origin.x, scale, (*offset).x);
    (*link).pos.y = fmadds(origin.y, scale, (*offset).y);
    (*link).pos.z = fmadds(origin.z, scale, (*offset).z);
    while !prev.is_null() {
        (*prev).vel.y -= (*sa).x18;
        ext::it_802A4420(prev);
        ext::it_802A43EC(prev);
        it_802BB938(prev, 0, (*sa).x4);
        if ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut origin) > (*sa).x4 {
            (*prev).pos.x = fmadds(origin.x, (*sa).x4, (*link).pos.x);
            (*prev).pos.y = fmadds(origin.y, (*sa).x4, (*link).pos.y);
            (*prev).pos.z = fmadds(origin.z, (*sa).x4, (*link).pos.z);
        }
        link = prev;
        prev = (*prev).prev;
    }
}

pub unsafe extern "C" fn it_802BBD64(link: *mut ItemLink, arg1: *mut Vec3, sa: *mut itSeakChain_Attrs) -> i32 {
    melee_core::ct!("it_802BBD64");
    let mut vec: Vec3 = core::mem::zeroed();
    let mut cur = link;
    let mut prev = (*link).next;

    ext::it_802A4420(link);
    let temp_r29 = it_802BB938(cur, 1, (*sa).x4) & 0xFFF;
    (*cur).pos.z = (*cur).pos.z;
    while !prev.is_null() {
        if (*prev).x2C_b0() != 0 {
            if ext::it_802A3C98(&mut (*prev).pos, &mut (*cur).pos, &mut vec) > (*sa).x4 {
                (*prev).pos.x = fmadds(vec.x, (*sa).x4, (*cur).pos.x);
                (*prev).pos.y = fmadds(vec.y, (*sa).x4, (*cur).pos.y);
                (*prev).pos.z = fmadds(vec.z, (*sa).x4, (*cur).pos.z);
            }
            ext::it_802A43EC(prev);
        } else {
            if ext::it_802A3C98(arg1, &mut (*cur).pos, &mut vec) > (*sa).x4 {
                (*prev).pos.x = fmadds(vec.x, (*sa).x4, (*cur).pos.x);
                (*prev).pos.y = fmadds(vec.y, (*sa).x4, (*cur).pos.y);
                (*prev).pos.z = fmadds(vec.z, (*sa).x4, (*cur).pos.z);
                (*prev).set_x2C_b0(1);
                ext::it_802A43B8(prev);
            } else {
                if temp_r29 != 0 {
                    return 1;
                }
                return 0;
            }
        }
        cur = prev;
        prev = (*prev).next;
    }
    it_802BBB0C(cur, arg1, sa, (*sa).x4);
    2
}

pub unsafe extern "C" fn it_802BBED0(link: *mut ItemLink, arg1: *mut Point3d, sa: *mut itSeakChain_Attrs) -> i32 {
    melee_core::ct!("it_802BBED0");
    let mut pos: Point3d = core::mem::zeroed();
    let mut cur = link;
    let mut prev = (*link).next;

    (*link).vel.y -= (*sa).x18;
    ext::it_802A4420(cur);

    {
        let temp_r29 = it_802BB938(cur, 1, (*sa).x4) & 0xFFF;
        let mut vel_scale: f32 = 1.0;
        while !prev.is_null() {
            if (*prev).x2C_b0() != 0 {
                (*prev).vel.y = -fmadds((*sa).x18, vel_scale, -(*prev).vel.y);
                vel_scale *= (*sa).x34;
                ext::it_802A4420(prev);
                if ext::it_802A3C98(&mut (*prev).pos, &mut (*cur).pos, &mut pos) > (*sa).x4 {
                    (*prev).pos.x = fmadds(pos.x, (*sa).x4, (*cur).pos.x);
                    (*prev).pos.y = fmadds(pos.y, (*sa).x4, (*cur).pos.y);
                    (*prev).pos.z = fmadds(pos.z, (*sa).x4, (*cur).pos.z);
                }
                ext::it_802A43EC(prev);
            } else {
                if ext::it_802A3C98(arg1, &mut (*cur).pos, &mut pos) > (*sa).x4 {
                    (*prev).pos.x = fmadds(pos.x, (*sa).x4, (*cur).pos.x);
                    (*prev).pos.y = fmadds(pos.y, (*sa).x4, (*cur).pos.y);
                    (*prev).pos.z = fmadds(pos.z, (*sa).x4, (*cur).pos.z);
                    (*prev).set_x2C_b0(1);
                    ext::it_802A43B8(prev);
                } else {
                    if temp_r29 != 0 {
                        return 1;
                    }
                    return 0;
                }
            }
            cur = prev;
            prev = (*prev).next;
        }
        it_802BBB0C(cur, arg1, sa, (*sa).x4);
        2
    }
}

/// `itSeakChain_clamp_x10` (static inline)
#[inline]
unsafe fn itSeakChain_clamp_x10(link: *mut ItemLink, attrs: *mut itSeakChain_Attrs) {
    let value = (*link).vel.x;
    let limit = (*attrs).x10;

    if value > limit {
        (*link).vel.x = value - limit;
    } else if value < -limit {
        (*link).vel.x = value + limit;
    } else {
        (*link).vel.x = 0.0;
    }
}

/// `itSeakChain_clamp_x14` (static inline)
#[inline]
unsafe fn itSeakChain_clamp_x14(link: *mut ItemLink, attrs: *mut itSeakChain_Attrs) {
    let value = (*link).vel.x;
    let limit = (*attrs).x14;

    if value > limit {
        (*link).vel.x = value - limit;
    } else if value < -limit {
        (*link).vel.x = value + limit;
    } else {
        (*link).vel.x = 0.0;
    }
}

pub unsafe extern "C" fn it_802BC080(link: *mut ItemLink, target: *mut Vec3, ip: *mut Item) {
    melee_core::ct!("it_802BC080");
    let mut dir: Vec3 = core::mem::zeroed();
    let mut iter = (*link).prev;
    let mut cur = link;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    let last_idx: i32 = fmadds(0.5, (*attrs).x0 as f32, -1.0) as i32;
    let mut env_flags: i32 = 0;

    while !iter.is_null() && (*cur).x2C_b0() == 0 {
        cur = iter;
        iter = (*iter).prev;
    }

    let coll_arg: i32 = if (*ip).xDD4_itemVar.seakchain.x18 != 0 { 1 } else { 0 };
    let fp = (*(*ip).xDD4_itemVar.seakchain.parent_gobj).user_data as *mut Fighter;

    let mut i: i32 = 0;
    while last_idx > i {
        (*ip).xDD4_itemVar.seakchain.history[(last_idx - i) as usize].x =
            (*ip).xDD4_itemVar.seakchain.history[(last_idx - 1 - i) as usize].x;
        (*ip).xDD4_itemVar.seakchain.history[(last_idx - i) as usize].y =
            (*ip).xDD4_itemVar.seakchain.history[(last_idx - 1 - i) as usize].y;
        i += 1;
    }

    let lsd = (*fp).u.sk.lstick_delta;
    if ABS(lsd.x) > (*attrs).x48 {
        let sign: i32 = if lsd.x < 0.0 { -1 } else { 1 };
        if (*ip).facing_dir == sign as f32 {
            (*ip).xDD4_itemVar.seakchain.history[0].x = lsd.x * (*attrs).x38;
        } else {
            (*ip).xDD4_itemVar.seakchain.history[0].x = lsd.x * (*attrs).x3C;
        }
    } else {
        (*ip).xDD4_itemVar.seakchain.history[0].x = 0.0;
    }

    if ABS(lsd.y) > (*attrs).x48 {
        let sign: i32 = if lsd.y < 0.0 { -1 } else { 1 };
        if sign as f32 > 0.0 {
            (*ip).xDD4_itemVar.seakchain.history[0].y = lsd.y * (*attrs).x40;
        } else {
            (*ip).xDD4_itemVar.seakchain.history[0].y = lsd.y * (*attrs).x44;
        }
    } else {
        (*ip).xDD4_itemVar.seakchain.history[0].y = 0.0;
    }

    (*ip).xDD4_itemVar.seakchain.x18 = (*fp).mv.co.common.x1C.to_bits() as i32;

    (*cur).vel.x = fmadds((*ip).xDD4_itemVar.seakchain.history[0].x, (*attrs).x1C, (*cur).vel.x);
    (*cur).vel.y = fmadds((*ip).xDD4_itemVar.seakchain.history[0].y, (*attrs).x20, (*cur).vel.y);
    itSeakChain_clamp_x10(cur, attrs);
    if ABS((*cur).vel.x) > (*attrs).x24 {
        if (*cur).vel.x > 0.0 {
            (*cur).vel.x = (*attrs).x24;
        } else {
            (*cur).vel.x = -(*attrs).x24;
        }
    }
    if (*cur).vel.y > (*attrs).x18 - (*attrs).x28 {
        (*cur).vel.y -= (*attrs).x18;
    } else if (*cur).vel.y < -(*attrs).x18 - (*attrs).x28 {
        (*cur).vel.y += (*attrs).x18;
    }
    if ABS((*cur).vel.y) > (*attrs).x2C {
        if (*cur).vel.y > 0.0 {
            (*cur).vel.y = (*attrs).x2C;
        } else {
            (*cur).vel.y = -(*attrs).x2C;
        }
    }

    ext::it_802A4420(cur);

    let mut scale: f32 = 1.0;
    scale *= (*attrs).x34;
    if ext::it_802A3C98(&mut (*cur).pos, target, &mut dir) > (*attrs).x4 {
        (*cur).pos.x = fmadds(dir.x, (*attrs).x4, (*target).x);
        (*cur).pos.y = fmadds(dir.y, (*attrs).x4, (*target).y);
        (*cur).pos.z = fmadds(dir.z, (*attrs).x4, (*target).z);
    }

    let use_arg = (*ip).xDD4_itemVar.seakchain.x10;
    let lstick_x = ext::ftSk_SpecialS_80110F58((*ip).xDD4_itemVar.seakchain.parent_gobj);
    let lstick_y = ext::ftSk_SpecialS_80110F64((*ip).xDD4_itemVar.seakchain.parent_gobj);
    let mode: i32 = if ABS(lstick_x) < (*attrs).x48 && ABS(lstick_y) < (*attrs).x48 {
        3
    } else if lstick_y < -0.5 {
        1
    } else {
        2
    };

    let mut counter: i32 = 0;
    while !iter.is_null() {
        let idx = (0.5f32 * ((counter + 1) as f32)) as i32;
        let fx = (*attrs).x1C;
        let fy = (*attrs).x20;
        let mut dx = scale * ((*ip).xDD4_itemVar.seakchain.history[idx as usize].x * fx);
        let dy = scale * ((*ip).xDD4_itemVar.seakchain.history[idx as usize].y * fy);
        if use_arg != 0 {
            dx *= (*attrs).x30;
        }
        (*iter).vel.x += dx;
        (*iter).vel.y += dy;

        {
            let lim = (*attrs).x10 * scale;
            if (*iter).vel.x > lim {
                (*iter).vel.x -= lim;
            } else if (*iter).vel.x < -(*attrs).x10 * scale {
                (*iter).vel.x += lim;
            } else {
                (*iter).vel.x = 0.0;
            }
        }
        {
            if ABS((*iter).vel.x) > (*attrs).x24 * scale {
                if (*iter).vel.x > 0.0 {
                    (*iter).vel.x = (*attrs).x24 * scale;
                } else {
                    (*iter).vel.x = -(*attrs).x24 * scale;
                }
            }
        }
        {
            let vy_lim = (*attrs).x18 * scale;
            if (*iter).vel.y > vy_lim - (*attrs).x28 {
                (*iter).vel.y -= vy_lim;
            } else if (*iter).vel.y < fmadds(-(*attrs).x18, scale, -(*attrs).x28) {
                (*iter).vel.y += vy_lim;
            }
        }
        {
            if ABS((*iter).vel.y) > (*attrs).x2C * scale {
                if (*iter).vel.y > 0.0 {
                    (*iter).vel.y = (*attrs).x2C * scale;
                } else {
                    (*iter).vel.y = -(*attrs).x2C * scale;
                }
            }
        }

        counter += 1;
        scale *= (*attrs).x34;
        ext::it_802A4420(iter);

        if counter > mode {
            if counter % 5 == 0 {
                env_flags = it_802BBAEC(iter, coll_arg, (*attrs).x4);
            } else {
                env_flags = it_802BBAEC(iter, 0, (*attrs).x4);
            }
        } else {
            env_flags = 0;
        }
        env_flags &= 0x18000;
        if env_flags != 0 {
            if ABS((*iter).vel.y) > (*attrs).x60 {
                (*iter).vel.y *= -(*attrs).x5C;
            } else {
                itSeakChain_clamp_x14(iter, attrs);
                (*iter).vel.y = 0.0;
            }
        }

        if ext::it_802A3C98(&mut (*iter).pos, &mut (*cur).pos, &mut dir) > (*attrs).x4 {
            (*iter).pos.x = fmadds(dir.x, (*attrs).x4, (*cur).pos.x);
            (*iter).pos.y = fmadds(dir.y, (*attrs).x4, (*cur).pos.y);
            (*iter).pos.z = fmadds(dir.z, (*attrs).x4, (*cur).pos.z);
        }

        cur = iter;
        iter = (*iter).prev;
    }
    (*ip).xDD4_itemVar.seakchain.x10 = env_flags;
}

pub unsafe extern "C" fn it_802BC94C(arg0: *mut ItemLink, arg1: *mut Vec3, sa: *mut itSeakChain_Attrs, farg0: f32) -> bool {
    melee_core::ct!("it_802BC94C");
    let mut var_r30: *mut ItemLink = null_mut();
    let mut var_r29: *mut ItemLink = null_mut();
    let mut var_f1: f32 = 0.0;

    Item_RetractChain(arg0, arg1, farg0, &(*sa).x4, &mut var_r30, &mut var_r29, &mut var_f1);
    it_802BBC38(var_r29, arg1, sa, var_f1);

    if !var_r30.is_null() {
        return false;
    }
    true
}

pub unsafe extern "C" fn it_802BCA30(ip: *mut Item) {
    melee_core::ct!("it_802BCA30");
    let link = (*ip).xDD4_itemVar.seakchain.x0;
    let jobj0 = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
    let mut mtx: Mtx = core::mem::zeroed();
    let mut vec: Vec3 = it_803B868C;
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXIdentity(mtx.as_mut_ptr() as *mut c_void);
    mtx[0][3] = vec.x;
    mtx[1][3] = vec.y;
    mtx[2][3] = vec.z;
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    HSD_JObjCopyMtx(jobj0, &mut mtx);
    (*jobj0).flags |= (JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT) as u32;
    HSD_JObjSetMtxDirty(jobj0);
}

#[inline]
unsafe fn it_802BCB88_prev(link: *mut ItemLink) -> *mut ItemLink {
    (*link).prev
}

pub unsafe extern "C" fn it_802BCB88(ip: *mut Item, vec: *mut Vec3) {
    melee_core::ct!("it_802BCB88");
    let mut link = (*ip).xDD4_itemVar.seakchain.x4;
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSeakChain_Attrs;
    let stride: i32 = (*sa).x0 / 3;
    while (*link).x2C_b0() == 0 {
        link = (*link).prev;
    }
    {
        let mut count: i32 = 0;
        while !link.is_null() {
            let mut cur = (*link).next;
            let pos0: Vec3;
            if !cur.is_null() {
                if (*cur).x2C_b0() != 0 {
                    pos0 = (*cur).pos;
                } else {
                    pos0 = *vec;
                }
            } else {
                pos0 = *vec;
            }
            cur = it_802BCB88_prev(link);
            {
                let pos1: Vec3;
                if !cur.is_null() {
                    pos1 = (*cur).pos;
                } else {
                    pos1 = (*link).pos;
                }
                {
                    let jobj = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
                    {
                        let mut translate: Vec3 = (*link).pos;
                        HSD_JObjSetTranslate(jobj, &mut translate);
                        if melee_runtime::cabi::srem(count, stride) == 0 {
                            let mut i: i32 = 3;
                            while i > melee_runtime::cabi::sdiv(count, stride) {
                                ext::ftSk_SpecialS_UpdateHitboxes((*ip).xDD4_itemVar.seakchain.parent_gobj, &mut translate, i - 1);
                                i -= 1;
                            }
                        }
                        if (*link).prev.is_null() {
                            ext::ftSk_SpecialS_UpdateHitboxes((*ip).xDD4_itemVar.seakchain.parent_gobj, &mut translate, 3);
                        }
                        count += 1;
                        {
                            let mut dir: Vec3 = core::mem::zeroed();
                            dir.x = pos1.x - pos0.x;
                            dir.y = pos1.y - pos0.y;
                            dir.z = pos1.z - pos0.z;
                            ext::lbVector_Normalize(&mut dir);
                            ext::it_802A6DC8(jobj, &mut translate, &mut dir);
                        }
                    }
                    {
                        let mut mtx: Mtx = core::mem::zeroed();
                        ext::PSMTXCopy((*jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
                        mtx[0][0] *= (*sa).x4;
                        mtx[1][0] *= (*sa).x4;
                        mtx[2][0] *= (*sa).x4;
                        mtx[0][1] *= (*sa).x4;
                        mtx[1][1] *= (*sa).x4;
                        mtx[2][1] *= (*sa).x4;
                        mtx[0][2] *= (*sa).x4;
                        mtx[1][2] *= (*sa).x4;
                        mtx[2][2] *= (*sa).x4;
                        ext::PSMTXCopy(mtx.as_mut_ptr() as *mut c_void, (*jobj).mtx.as_mut_ptr() as *mut c_void);
                    }
                }
            }
            link = (*link).prev;
        }
    }
}

pub unsafe extern "C" fn it_2725_Logic54_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic54_PickedUp");
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*item).on_accessory = Some(fn_802BB428);
}

pub unsafe extern "C" fn it_802BCED4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BCED4");
    let ip = GET_ITEM(gobj);
    ext::ftColl_8007AFF8((*ip).xDD4_itemVar.seakchain.parent_gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(fn_802BB694);
}

pub unsafe extern "C" fn it_802BCF2C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BCF2C");
    let ip = GET_ITEM(gobj);
    ext::ftColl_8007AFF8((*ip).xDD4_itemVar.seakchain.parent_gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(fn_802BB574);
}

pub unsafe extern "C" fn it_802BCF84(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BCF84");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(fn_802BB784);
}

pub unsafe extern "C" fn it_802BCFC4(gobj: *mut Item_GObj, vel: *mut Vec3) {
    melee_core::ct!("it_802BCFC4");
    let ip = GET_ITEM(gobj);
    let link = (*ip).xDD4_itemVar.seakchain.x0;
    let mut pos: Vec3 = core::mem::zeroed();
    let mut mtx: Mtx = core::mem::zeroed();
    Item_InitLinkMtx(&mut mtx, 0.1);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat((*(*link).jobj).mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void, mtx.as_mut_ptr() as *mut c_void);
    {
        pos.x = mtx[0][3];
        pos.y = mtx[1][3];
        pos.z = mtx[2][3];
        (*link).pos = pos;
    }
    ext::it_8026BB68((*ip).xDD4_itemVar.seakchain.parent_gobj, &mut (*link).coll_data.cur_pos);
    (*link).coll_data.last_pos = (*link).coll_data.cur_pos;
    (*link).set_x2C_b0(1);
    (*link).vel = *vel;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(fn_802BB44C);
}

pub unsafe extern "C" fn itSeakChain_Logic54_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Fighter_GObj) {
    melee_core::ct!("itSeakChain_Logic54_EvtUnk");
    let item = GET_ITEM(gobj);
    ext::it_8026B894(gobj, ref_gobj);
    if (*item).xDD4_itemVar.seakchain.parent_gobj == ref_gobj {
        (*item).xDD4_itemVar.seakchain.parent_gobj = null_mut();
    }
}
