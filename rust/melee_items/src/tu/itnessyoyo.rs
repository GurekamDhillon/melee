//! Port of src/melee/it/kinds/itnessyoyo.c: Ness's yo-yo (logic table 59) and its string links. Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itnessyoyo.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
use melee_core::fp::Fmadd as _;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::helpers::decrease_lifetimer;
#[allow(unused_imports)]
use crate::itkinds::it_802d5884;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_chicorita_phys;
#[allow(unused_imports)]
use crate::itkinds::it_harisen_logic24_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lipstick_logic23_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lizardon_unk_motion1_phys;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_anim_inline;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_80275870;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_8027b730;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::{fmadds, frsqrte};
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1 as i32) };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
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
    fn ftLib_GetScale(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_GetScale(a0) };
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_GetMotionId(a0) };
    fn ftNs_AttackHi4_YoyoItemSetFlag(a0: *mut HSD_GObj);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80275328(a0: *mut HSD_GObj, a1: *mut ItemLink) => unsafe { crate::itlib3::it_80275328(a0, &mut *a1) };
    fn it_802A2474(a0: *mut c_void) => unsafe { crate::tu::itlinkhookshot::it_802A2474(a0) };
    fn it_802A24A0(a0: *mut HSD_GObj, a1: isize) => unsafe { crate::tu::itlinkhookshot::it_802A24A0(a0, a1) };
    fn it_802A24D0(a0: *mut ItemLink, a1: f32) => unsafe { crate::tu::itlinkhookshot::it_802A24D0(a0, a1) };
    fn it_802A3C98(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> f32 => unsafe { crate::tu::itlinkhookshot::it_802A3C98(a0 as _, a1 as _, a2 as _) };
    fn it_802A3D90(a0: *mut ItemLink) -> i32 => unsafe { crate::tu::itlinkhookshot::it_802A3D90(a0) };
    fn it_802A42F4(a0: *mut ItemLink, a1: f32) -> i32 => unsafe { crate::tu::itlinkhookshot::it_802A42F4(a0, a1) };
    fn it_802A43B8(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43b8(&mut *a0) };
    fn it_802A43EC(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a43ec(&mut *a0) };
    fn it_802A4420(a0: *mut ItemLink) => unsafe { crate::itkinds6::it_802a4420(&mut *a0) };
    fn it_802A6DC8(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { crate::tu::itlinkhookshot::it_802A6DC8(a0, a1 as _, a2 as _) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
    fn mpColl_800471F8(a0: *mut CollData) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_800471F8(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: isize) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn it_802A2474(a0: *mut c_void) { super::it_802A2474(a0) }
        pub unsafe extern "C" fn it_802A24A0(a0: *mut HSD_GObj, a1: isize) { super::it_802A24A0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use crate::tu::item::item_link_alloc_data;
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `ItemStateTable it_803F7558[]`
pub static mut it_803F7558: [ItemStateTable; 4] = [
    st_row(-1, pred_b!(itNessyoyo_UnkMotion3_Anim), Some(itNessyoyo_UnkMotion0_Phys), None),
    st_row(-1, pred_b!(itNessyoyo_UnkMotion3_Anim), Some(itNessyoyo_UnkMotion1_Phys), None),
    st_row(-1, pred_b!(itNessyoyo_UnkMotion3_Anim), Some(itNessyoyo_UnkMotion2_Phys), None),
    st_row(-1, pred_b!(itNessyoyo_UnkMotion3_Anim), Some(itNessyoyo_UnkMotion3_Phys), None),
];

/// `const Vec3 it_803B8698 = { 0, 0, 0 }`
pub static it_803B8698: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
/// `const Vec3 it_803B86A4 = { 0, 0, 1.8f }`
pub static it_803B86A4: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.8 };
/// `static const f32 it_zero = 0.0f`
static it_zero: f32 = 0.0;

/// `HSD_JObjSetupMatrix` (jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_JObjCopyMtx` (jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *mut c_void) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!mtx.is_null(), "(mtx)");
    ext::PSMTXCopy(mtx, &raw mut (*jobj).mtx as *mut c_void);
}

#[inline]
unsafe fn FIGHTER(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline]
unsafe fn ATTRS(ip: *mut Item) -> *mut itYoyoAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itYoyoAttributes
}

/// `m = I; m[i][3] = 0` prologue of the Phys functions plus `PSMTXConcat(jobj->mtx, m, m)`; returns m's translation.
#[inline]
unsafe fn bone_pos(jobj: *mut HSD_JObj) -> Vec3 {
    let mut m: Mtx = core::mem::zeroed();
    let mp = m.as_mut_ptr() as *mut c_void;
    ext::PSMTXIdentity(mp);
    m[0][3] = 0.0;
    m[1][3] = 0.0;
    m[2][3] = 0.0;
    HSD_JObjSetupMatrix(jobj);
    ext::PSMTXConcat(&raw mut (*jobj).mtx as *mut c_void, mp, mp);
    Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] }
}

pub unsafe extern "C" fn it_802BE598(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BE598");
    ext::it_8026B724(gobj);
}

pub unsafe extern "C" fn it_802BE5B8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BE5B8");
    ext::it_8026B73C(gobj);
}

pub unsafe extern "C" fn it_802BE5D8(arg: *mut c_void, frame: f32) {
    melee_core::ct!("it_802BE5D8");
    let gobj = arg as *mut HSD_GObj;
    let ip = (*gobj).user_data as *mut Item;
    let attrs = ATTRS(ip);
    let jobj = (*ip).xDD4_itemVar.nessyoyo.x18;
    ext::HSD_JObjRemoveAnimAll(jobj);
    ext::HSD_JObjAddAnimAll(jobj, null_mut(), (*attrs).x58_yoyo_matanim, null_mut());
    ext::lb_8000BA0C(jobj, 1.0);
    ext::HSD_JObjReqAnimAll(jobj, frame);
    ext::HSD_JObjAnimAll(jobj);
}

#[inline]
unsafe fn it_802BE65C_LoadString(ip: *mut Item) -> *mut HSD_JObj {
    let attrs = ATTRS(ip);
    ext::HSD_JObjLoadJoint((*attrs).x50_string_joint)
}

pub unsafe extern "C" fn it_802BE65C(ip: *mut Item, bone_jobj: *mut HSD_JObj) -> *mut HSD_GObj {
    melee_core::ct!("it_802BE65C");
    let mut prev_link: *mut ItemLink = null_mut();
    let mut head_link: *mut ItemLink = null_mut();
    let mut tail_link: *mut ItemLink = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let attrs = ATTRS(ip);
    let zero_vel: Vec3 = it_803B8698;

    let mut i: i32 = 0;
    while i < (*attrs).x0_CHARGE_SPAWN_POS {
        let link_gobj = ext::GObj_Create(7, 0xA, 0);
        if link_gobj.is_null() {
            while !prev_link.is_null() {
                ext::HSD_GObjFree((*prev_link).gobj);
                prev_link = (*prev_link).next;
            }
            return null_mut();
        }

        let link = ext::HSD_ObjAlloc(&mut ext::item_link_alloc_data) as *mut ItemLink;
        ext::it_80275328(link_gobj, link);
        ext::GObj_InitUserData(link_gobj, 6, Some(ext::cb::it_802A2474), link as *mut c_void);

        if i == 0 {
            (*link).next = null_mut();
            head_link = link;
            (*link).jobj = bone_jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(0);
            ext::it_802A24D0(link, 1.0);

            let s = it_802BE65C_LoadString(ip);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, s as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::it_802A24A0), 6, 0);
        } else if i == (*attrs).x0_CHARGE_SPAWN_POS - 1 {
            (*prev_link).prev = link;
            (*link).prev = null_mut();
            tail_link = link;
            (*link).next = prev_link;
            (*link).jobj = bone_jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(1);
            ext::it_802A24D0(link, 2.0 * ext::ftLib_GetScale((*ip).owner));
            {
                let attrs = ATTRS(ip);
                let joint = (*attrs).x54_yoyo_joint;
                let jobj = ext::HSD_JObjLoadJoint(joint);
                ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
                ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::HSD_GObj_JObjCallback), 6, 0);
                (*ip).xDD4_itemVar.nessyoyo.x18 = jobj;
                result = (*link_gobj).hsd_obj as *mut HSD_GObj;
            }
        } else {
            (*prev_link).prev = link;
            (*link).next = prev_link;
            (*link).jobj = bone_jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(0);
            ext::it_802A24D0(link, 1.0);
            let s = it_802BE65C_LoadString(ip);
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, s as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::it_802A24A0), 6, 0);
        }
        prev_link = link;
        i += 1;
    }
    (*ip).xDD4_itemVar.nessyoyo.x8 = tail_link;
    (*ip).xDD4_itemVar.nessyoyo.xC = head_link;
    result
}

/// `it_802BE958_inline` (static inline)
#[inline]
unsafe fn it_802BE958_inline(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !(*ip).owner.is_null() {
            ext::ftNs_AttackHi4_YoyoItemSetFlag((*ip).owner);
        }
        (*ip).owner = null_mut();
        {
            let mut link = (*ip).xDD4_itemVar.nessyoyo.x8;
            while !link.is_null() {
                let tmp = (*link).gobj;
                link = (*link).next;
                ext::HSD_GObjFree(tmp);
            }
        }
    }
}

pub unsafe extern "C" fn it_802BE958(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BE958");
    if !GET_ITEM(gobj).is_null() {
        it_802BE958_inline(gobj);
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn it_802BE9D8(owner: *mut HSD_GObj, facing_dir: f32, pos: *mut Vec3, action: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_802BE9D8");
    let fp = (*owner).user_data as *mut Fighter;
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Ness_Yoyo;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, owner, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let mut ip = GET_ITEM(gobj);

        (*ip).xDD4_itemVar.nessyoyo.x10 = owner;
        (*ip).xDD4_itemVar.nessyoyo.x0 = action;
        if (*ip).xDD4_itemVar.nessyoyo.x0 == 0x159 {
            (*ip).facing_dir *= -1.0;
        }
        it_802BE65C(ip, (*(*fp).parts.add(FtPart_R2ndNa as usize)).joint);
        ext::Item_8026AB54(gobj, owner, FtPart_R2ndNa as u32);
        ip = GET_ITEM(gobj);
        let yoyo_jobj = (*ip).xDD4_itemVar.nessyoyo.x18;
        let attrs = ATTRS(ip);
        ext::HSD_JObjRemoveAnimAll(yoyo_jobj);
        ext::HSD_JObjAddAnimAll(yoyo_jobj, null_mut(), (*attrs).x58_yoyo_matanim, null_mut());
        ext::lb_8000BA0C(yoyo_jobj, 1.0);
        ext::HSD_JObjReqAnimAll(yoyo_jobj, 0.0);
        ext::HSD_JObjAnimAll(yoyo_jobj);
    }
    gobj
}

pub unsafe extern "C" fn itNessyoyo_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itNessyoyo_UnkMotion0_Phys");
    it_802BF900(GET_ITEM(gobj));
}

pub unsafe extern "C" fn itNessyoyo_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itNessyoyo_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = ATTRS(ip);
    let link1 = (*ip).xDD4_itemVar.nessyoyo.x8;

    let mut pos = bone_pos((*link1).jobj);
    it_802BF28C(link1, &mut pos, attrs, ip);
    it_802BFAFC(ip, &mut pos);
}

pub unsafe extern "C" fn itNessyoyo_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itNessyoyo_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = ATTRS(ip);
    let link1 = (*ip).xDD4_itemVar.nessyoyo.x8;

    let mut pos = bone_pos((*link1).jobj);
    it_802BF4A0(link1, &mut pos, attrs, ip);
    it_802BFAFC(ip, &mut pos);
}

pub unsafe extern "C" fn itNessyoyo_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itNessyoyo_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = ATTRS(ip);
    let link2 = (*ip).xDD4_itemVar.nessyoyo.xC;

    let mut pos = bone_pos((*link2).jobj);
    if it_802BF800(link2, &mut pos, attrs, ip, (*ip).xDD4_itemVar.nessyoyo.x4) {
        let mut zero_vec: Vec3 = core::mem::zeroed();
        zero_vec.z = 0.0;
        zero_vec.y = 0.0;
        zero_vec.x = 0.0;
        it_802C0010(gobj, &mut zero_vec);
        {
            let owner = (*ip).xDD4_itemVar.nessyoyo.x10;
            let link1 = (*ip).xDD4_itemVar.nessyoyo.x8;
            let fp = (*owner).user_data as *mut Fighter;
            (*link1).pos = (*fp).u.ns.yoyo_hitbox_pos;
        }
    }
    it_802BFAFC(ip, &mut pos);
}

#[inline]
unsafe fn itNessyoyo_UnkMotion3_Anim_inline(gobj: *mut Item_GObj) -> bool {
    if !(*GET_ITEM(gobj)).owner.is_null() {
        let action = ext::ftLib_GetMotionId((*GET_ITEM(gobj)).owner);
        if action >= 0x156 && action <= 0x15B {
            return false;
        } else {
            return true;
        }
    }
    true
}

#[inline]
unsafe fn itNessyoyo_UnkMotion3_Anim_UpdateRotation(ip: *mut Item) {
    let fp = FIGHTER((*ip).xDD4_itemVar.nessyoyo.x10);
    let child: *mut HSD_JObj;
    if (*ip).xDD4_itemVar.nessyoyo.x18.is_null() {
        child = null_mut();
    } else {
        child = (*(*ip).xDD4_itemVar.nessyoyo.x18).child;
    }

    if !child.is_null() {
        let mut rot = HSD_JObjGetRotationX(child);
        rot += (*fp).u.ns.x223C;
        HSD_JObjSetRotationX(child, rot);
    }
}

pub unsafe extern "C" fn itNessyoyo_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itNessyoyo_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    itNessyoyo_UnkMotion3_Anim_UpdateRotation(ip);
    if itNessyoyo_UnkMotion3_Anim_inline(gobj) {
        it_802BE958_inline(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn it_802BF030(link: *mut ItemLink, arg1: i32, offset: f32) -> i32 {
    melee_core::ct!("it_802BF030");
    let coll = &raw mut (*link).coll_data;

    (*coll).last_pos = (*coll).cur_pos;
    (*coll).cur_pos = (*link).pos;
    if ext::mpColl_800471F8(coll) != 0 {
        if (*link).x2C_b1() == 0 && arg1 != 0 && (*link).x2C_b2() == 0 {
            (*link).set_x2C_b2(1);
        }
        (*link).set_x2C_b1(1);
    } else {
        (*link).set_x2C_b1(0);
    }
    {
        let env = (*coll).env_flags;
        if env & 0x18000 != 0 {
            (*coll).cur_pos.x = (*link).pos.x;
        } else if env & 0xFFF != 0 {
            if !(*link).next.is_null() && (*(*link).next).x2C_b0() != 0 && !(*link).prev.is_null() {
                (*coll).cur_pos.y += offset;
            }
        }
    }
    (*link).pos = (*coll).cur_pos;
    (*coll).env_flags & 0x18FFF
}

pub unsafe extern "C" fn it_802BF180(cur: *mut ItemLink, target: *mut Vec3, attrs: *mut itYoyoAttributes, length: f32) {
    melee_core::ct!("it_802BF180");
    let mut cur = cur;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*cur).prev;

    ext::it_802A3C98(&mut (*cur).pos, target, &mut dir);
    (*cur).pos.x = fmadds(dir.x, length, (*target).x);
    (*cur).pos.y = fmadds(dir.y, length, (*target).y);
    (*cur).pos.z = fmadds(dir.z, length, (*target).z);
    while !prev.is_null() {
        (*prev).vel.y = 0.0;
        ext::it_802A4420(prev);
        ext::it_802A43EC(prev);
        ext::it_802A3D90(prev);
        if ext::it_802A3C98(&mut (*prev).pos, &mut (*cur).pos, &mut dir) > length {
            (*prev).pos.x = fmadds(dir.x, length, (*cur).pos.x);
            (*prev).pos.y = fmadds(dir.y, length, (*cur).pos.y);
            (*prev).pos.z = fmadds(dir.z, length, (*cur).pos.z);
        }
        cur = prev;
        prev = (*prev).prev;
    }
}

pub unsafe extern "C" fn it_802BF28C(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itYoyoAttributes, ip: *mut Item) -> i32 {
    melee_core::ct!("it_802BF28C");
    let mut dir: Vec3 = core::mem::zeroed();
    let mut dir2: Vec3 = core::mem::zeroed();
    let min_len = (*attrs).x10_UNK1 * ext::ftLib_GetScale((*ip).owner);
    let max_len = (*attrs).xC_SIZE * ext::ftLib_GetScale((*ip).owner);
    let fp = FIGHTER((*ip).xDD4_itemVar.nessyoyo.x10);
    let mut cur = link;
    let mut next = (*cur).next;

    (*cur).pos = (*fp).u.ns.yoyo_hitbox_pos;
    (*cur).vel.z = 0.0;
    (*cur).vel.y = 0.0;
    (*cur).vel.x = 0.0;

    while !next.is_null() {
        if (*next).x2C_b0() != 0 {
            (*next).pos = *target;
            let dist = ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir);
            if dist > max_len {
                (*next).pos.x = fmadds(dir.x, max_len, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, max_len, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, max_len, (*cur).pos.z);
            } else if dist < min_len {
                if ext::it_802A3C98(&mut (*next).pos, target, &mut dir2) <= 0.1 {
                    (*next).set_x2C_b0(0);
                } else {
                    (*next).pos.x = fmadds(dir.x, min_len, (*cur).pos.x);
                    (*next).pos.y = fmadds(dir.y, min_len, (*cur).pos.y);
                    (*next).pos.z = fmadds(dir.z, min_len, (*cur).pos.z);
                }
            }
        } else {
            if ext::it_802A3C98(target, &mut (*cur).pos, &mut dir) > max_len {
                (*next).pos.x = fmadds(dir.x, max_len, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, max_len, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, max_len, (*cur).pos.z);
                (*next).set_x2C_b0(1);
            } else {
                return 0;
            }
        }
        cur = next;
        next = (*next).next;
    }
    2
}

#[inline]
unsafe fn it_802BF4A0_adjust_tail(cur: *mut ItemLink, target: *mut Vec3, dir2: *mut Vec3, size: f32) {
    let prev0 = (*cur).prev;
    let mut tail = cur;
    let mut prev = prev0;
    ext::it_802A3C98(&mut (*tail).pos, target, dir2);
    (*cur).pos.x = fmadds((*dir2).x, size, (*target).x);
    (*cur).pos.y = fmadds((*dir2).y, size, (*target).y);
    (*cur).pos.z = fmadds((*dir2).z, size, (*target).z);

    while !prev.is_null() {
        if ext::it_802A3C98(&mut (*prev).pos, &mut (*tail).pos, dir2) > size {
            (*prev).pos.x = fmadds((*dir2).x, size, (*tail).pos.x);
            (*prev).pos.y = fmadds((*dir2).y, size, (*tail).pos.y);
            (*prev).pos.z = fmadds((*dir2).z, size, (*tail).pos.z);
        }
        tail = prev;
        prev = (*prev).prev;
    }
}

pub unsafe extern "C" fn it_802BF4A0(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itYoyoAttributes, ip: *mut Item) -> i32 {
    melee_core::ct!("it_802BF4A0");
    let mut dir: Vec3 = core::mem::zeroed();
    let mut dir2: Vec3 = core::mem::zeroed();

    ext::ftLib_GetScale((*ip).owner);
    let size = (*attrs).xC_SIZE * ext::ftLib_GetScale((*ip).owner);
    let mut cur = link;
    let mut next = (*link).next;

    if ABS((*link).vel.x) < (*attrs).x2C_UNK3_MOD {
        (*link).vel.x = fmadds((*attrs).x28_YOYO_PULL_STRENGTH, (*ip).facing_dir, (*link).vel.x);
    } else {
        (*link).vel.x = (*attrs).x2C_UNK3_MOD * (*ip).facing_dir;
    }

    if ABS((*link).vel.y) < (*attrs).x38_GRAVITY_MOD {
        (*link).vel.y -= (*attrs).x34_GRAVITY;
    } else {
        (*link).vel.y = -(*attrs).x38_GRAVITY_MOD;
    }

    ext::it_802A4420(cur);
    let mut coll_flags = it_802BF030(cur, 0, size);
    if coll_flags & 0x18000 != 0 {
        (*link).coll_data.cur_pos.y += 0.001;
        (*link).pos.y = (*link).coll_data.cur_pos.y;
        if (*link).vel.y < 0.0 {
            (*link).vel.y = 0.0;
        }
    }
    coll_flags &= 0xFFF;

    let mut count: i32 = 0;
    while !next.is_null() {
        count += 1;
        if (*next).x2C_b0() != 0 {
            (*next).vel.y -= (*attrs).x3C_YOYO_PULL_STRENGTH_2;
            ext::it_802A4420(next);
            it_802BF030(cur, 0, size);
            if ext::it_802A3C98(&mut (*next).pos, &mut (*cur).pos, &mut dir) > size {
                (*next).pos.x = fmadds(dir.x, size, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, size, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, size, (*cur).pos.z);
            }
            ext::it_802A43EC(next);
            ext::it_802A42F4(next, (1.5 + (*target).y as f64) as f32);
        } else {
            let charge_len: i32;
            if (*ip).xDD4_itemVar.nessyoyo.x0 == 0x156 {
                charge_len = (*attrs).x4_UPSMASH_CHARGE_STRING_LENGTH;
            } else {
                charge_len = (*attrs).x8_DOWNSMASH_CHARGE_STRING_LENGTH;
            }
            if count > charge_len {
                (*next).pos = *target;
            } else if ext::it_802A3C98(target, &mut (*cur).pos, &mut dir) > size {
                (*next).pos.x = fmadds(dir.x, size, (*cur).pos.x);
                (*next).pos.y = fmadds(dir.y, size, (*cur).pos.y);
                (*next).pos.z = fmadds(dir.z, size, (*cur).pos.z);
                (*next).set_x2C_b0(1);
                ext::it_802A43B8(next);
            } else if coll_flags != 0 {
                return 1;
            } else {
                return 0;
            }
        }
        cur = next;
        next = (*next).next;
    }

    it_802BF4A0_adjust_tail(cur, target, &mut dir2, size);

    (*link).vel.x *= 0.9;
    2
}

pub unsafe extern "C" fn it_802BF800(item: *mut ItemLink, pos: *mut Vec3, attrs: *mut itYoyoAttributes, ip: *mut Item, dist: f32) -> bool {
    melee_core::ct!("it_802BF800");
    let mut dir: Vec3 = core::mem::zeroed();
    let size = (*attrs).xC_SIZE * ext::ftLib_GetScale((*ip).owner);
    let mut prev = (*item).prev;
    let mut cur = item;
    while !prev.is_null() && (*cur).x2C_b0() == 0 {
        cur = prev;
        prev = (*prev).prev;
    }
    let mut len = ext::it_802A3C98(&mut (*cur).pos, pos, &mut dir);
    while !prev.is_null() && dist > len {
        (*cur).set_x2C_b0(0);
        len = ext::it_802A3C98(&mut (*prev).pos, pos, &mut dir);
        cur = prev;
        prev = (*prev).prev;
    }
    let step: f32;
    if len - dist > size {
        step = size;
    } else {
        step = len - dist;
    }
    it_802BF180(cur, pos, attrs, step);
    if !prev.is_null() {
        return false;
    }
    true
}

pub unsafe extern "C" fn it_802BF900(ip: *mut Item) {
    melee_core::ct!("it_802BF900");
    let mut trans: Mtx = core::mem::zeroed();
    let mut scale: Mtx = core::mem::zeroed();
    let tp = trans.as_mut_ptr() as *mut c_void;
    let sp = scale.as_mut_ptr() as *mut c_void;
    let link1 = (*ip).xDD4_itemVar.nessyoyo.x8;
    let target = (*(*link1).gobj).hsd_obj as *mut HSD_JObj;

    let offset: Vec3 = it_803B86A4;
    let scl = ext::ftLib_GetScale((*ip).owner);

    HSD_JObjSetupMatrix((*link1).jobj);
    ext::PSMTXIdentity(tp);
    trans[0][3] = offset.x * scl;
    trans[1][3] = offset.y * scl;
    trans[2][3] = offset.z * scl;
    ext::PSMTXConcat(&raw mut (*(*link1).jobj).mtx as *mut c_void, tp, tp);
    ext::PSMTXIdentity(sp);
    scale[0][0] *= scl;
    scale[1][0] *= scl;
    scale[2][0] *= scl;
    scale[0][1] *= scl;
    scale[1][1] *= scl;
    scale[2][1] *= scl;
    scale[0][2] *= scl;
    scale[1][2] *= scl;
    scale[2][2] *= scl;
    ext::PSMTXConcat(tp, sp, tp);
    HSD_JObjCopyMtx(target, tp);
    (*target).flags |= (JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT) as u32;
    HSD_JObjSetMtxDirty(target);
}

pub unsafe extern "C" fn it_802BFAFC(ip: *mut Item, target: *mut Vec3) {
    melee_core::ct!("it_802BFAFC");
    let mut link = (*ip).xDD4_itemVar.nessyoyo.xC;
    let mut next_pos: Vec3;
    let mut prev_pos: Vec3;
    let mut link_pos: Vec3;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut m: Mtx = core::mem::zeroed();

    while (*link).x2C_b0() == 0 {
        link = (*link).prev;
    }

    while !link.is_null() {
        if !(*link).next.is_null() {
            if (*(*link).next).x2C_b0() != 0 {
                next_pos = (*(*link).next).pos;
            } else {
                next_pos = *target;
            }
        } else {
            next_pos = *target;
        }

        if !(*link).prev.is_null() {
            prev_pos = (*(*link).prev).pos;
        } else {
            prev_pos = (*link).pos;
        }

        let jobj = GET_JOBJ((*link).gobj);
        link_pos = (*link).pos;
        HSD_JObjSetTranslate(jobj, &mut link_pos);

        dir.x = prev_pos.x - next_pos.x;
        dir.y = prev_pos.y - next_pos.y;
        dir.z = prev_pos.z - next_pos.z;
        ext::lbVector_Normalize(&mut dir);

        if jobj == (*ip).xDD4_itemVar.nessyoyo.x18 {
            if (*ip).facing_dir == 1.0 {
                dir.x = 1.0;
            } else {
                dir.x = -1.0;
            }
            dir.z = 0.0;
            dir.y = 0.0;
        }

        if jobj == (*ip).xDD4_itemVar.nessyoyo.x18 {
            let fp = (*(*ip).xDD4_itemVar.nessyoyo.x10).user_data as *mut Fighter;
            (*fp).u.ns.yoyo_hitbox_pos = (*link).pos;
        }

        ext::it_802A6DC8(jobj, &mut link_pos, &mut dir);

        let attrs = ATTRS(ip);
        let mut scale: f32;
        if jobj != (*ip).xDD4_itemVar.nessyoyo.x18 {
            scale = (*attrs).xC_SIZE;
        } else {
            scale = 1.0;
        }
        scale *= ext::ftLib_GetScale((*ip).owner);
        let mp = m.as_mut_ptr() as *mut c_void;
        ext::PSMTXCopy(&raw mut (*jobj).mtx as *mut c_void, mp);
        m[0][0] *= scale;
        m[1][0] *= scale;
        m[2][0] *= scale;
        m[0][1] *= scale;
        m[1][1] *= scale;
        m[2][1] *= scale;
        m[0][2] *= scale;
        m[1][2] *= scale;
        m[2][2] *= scale;
        ext::PSMTXCopy(mp, &raw mut (*jobj).mtx as *mut c_void);

        link = (*link).prev;
    }
}

pub unsafe extern "C" fn itNessYoyo_Logic59_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itNessYoyo_Logic59_PickedUp");
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802BFE5C(gobj: *mut Item_GObj, pos: *mut Vec3, _unused: f32) {
    melee_core::ct!("it_802BFE5C");
    let ip = GET_ITEM(gobj);
    let link = (*ip).xDD4_itemVar.nessyoyo.x8;
    (*link).vel = *pos;
    let mut vel_x = (*link).vel.x;
    if vel_x < 0.0 {
        vel_x = -vel_x;
    }
    (*link).vel.x = (*ip).facing_dir * vel_x;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

/// `my_sqrtf` (static inline): `3.0 - g*g*x` is a true f64 fma (`fmuladd.f64`).
#[inline]
pub unsafe fn my_sqrtf(x: f32) -> f32 {
    melee_core::ct!("my_sqrtf");
    let half: f64 = 0.5;
    let three: f64 = 3.0;
    if x > 0.0 {
        let mut guess = frsqrte(x as f64);
        guess = half * guess * (-(guess * guess)).fmadd(x as f64, three);
        guess = half * guess * (-(guess * guess)).fmadd(x as f64, three);
        guess = half * guess * (-(guess * guess)).fmadd(x as f64, three);
        let y = (x as f64 * guess) as f32;
        return y;
    }
    x
}

pub unsafe extern "C" fn it_802BFEC4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BFEC4");
    let ip = GET_ITEM(gobj);
    let link2 = (*ip).xDD4_itemVar.nessyoyo.xC;
    let link1 = (*ip).xDD4_itemVar.nessyoyo.x8;

    let m = bone_pos((*link2).jobj);
    let dx = (*link1).pos.x - m.x;
    let dy = (*link1).pos.y - m.y;
    // HYPOT(link1->pos.x - m[0][3], ...): squares of computed expressions, not fused on the console
    (*ip).xDD4_itemVar.nessyoyo.x4 = my_sqrtf(dx * dx + dy * dy);
    (*ip).xDD4_itemVar.nessyoyo.x4 *= 0.1;
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802C0010(gobj: *mut Item_GObj, vel: *mut Vec3) {
    melee_core::ct!("it_802C0010");
    let link1 = (*GET_ITEM(gobj)).xDD4_itemVar.nessyoyo.x8;

    let pos = bone_pos((*link1).jobj);
    (*link1).pos = pos;
    (*link1).coll_data.cur_pos = (*link1).pos;
    (*link1).coll_data.last_pos = (*link1).coll_data.cur_pos;
    (*link1).set_x2C_b0(1);
    (*link1).vel = *vel;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_2725_Logic59_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic59_EvtUnk");
    let ip = GET_ITEM(gobj);
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802BE598` (src/melee/it/kinds/itnessyoyo.c): `it_8026B724(..)`.
pub fn it_802be598(ip: &mut Item) {
    melee_core::ct!("it_802be598");
    it_8026b724(ip);
}

/// `it_802BE5B8` (src/melee/it/kinds/itnessyoyo.c): `it_8026B73C(..)`.
pub fn it_802be5b8(ip: &mut Item) {
    melee_core::ct!("it_802be5b8");
    it_8026b73c(ip);
}
