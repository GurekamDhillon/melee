//! Native port of src/melee/it/kinds/itclimbersstring.c (Ice Climbers' gum strings): the chain of
//! `ItemLink`s from the owner's hand, the chain solvers (`it_802C2CA8`, `it_802C2DB0`,
//! `it_802C2EC4`, `it_802C30E8`), the retract (`it_802C32D4`) and the four on-accessory states
//! (`it_803F76B8`). Callees in other TUs go through `ext`; callees ported in this crate are called
//! directly (`it_802a3c98`, `it_802a4420`, `it_80275328`, `it_8026b894`, `it_802c248c_joint24/28`).
//! `Item_InitLinkMtx` and `Item_RetractChain` are the C static inlines of kinds/inlines.h, ported
//! here as private helpers. Float sites `a*b + c` are fused (`fmadds`) as the shipping build does.
//! Checked by tests/tu_itclimbersstring.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::fmadds;

use crate::itlib::it_8026b894;
use crate::itkinds6::it_802a4420;
use crate::itkinds7::it_802a3c98;
use crate::itkinds14::{it_802c248c_joint24, it_802c248c_joint28};
use crate::itkinds6::it_2725_logic70_evt_unk;
use crate::itlib3::it_80275328;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;
/// `HSD_GObj_JObjKind` (sysdolphin/baselib/gobj.h).
pub const HSD_GObj_JObjKind: u8 = 4;

// BEGIN GENERATED EXT (callees of the C TU that are not ported in this crate yet)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
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
    fn GObj_Create(class: u16, p_link: u8, prio: u8) -> *mut HSD_GObj;
    fn HSD_GObjFree(gobj: *mut HSD_GObj);
    fn HSD_ObjAlloc(d: *mut HSD_ObjAllocData) -> *mut c_void;
    fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, free: unsafe extern "C" fn(*mut c_void), ud: *mut c_void);
    fn GObj_SetupGXLink(gobj: *mut HSD_GObj, cb: unsafe extern "C" fn(*mut HSD_GObj), gx_link: u8, priority: u32);
    fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, o: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj);
    fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, m: *const Mtx);
    fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj);
    fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *const Vec3);
    fn PSMTXIdentity(m: *mut Mtx);
    fn PSMTXConcat(a: *const Mtx, b: *const Mtx, ab: *mut Mtx);
    fn PSMTXCopy(src: *const Mtx, dst: *mut Mtx);
    fn lbVector_Normalize(v: *mut Vec3) -> f32;
    fn it_802A6DC8(jobj: *mut HSD_JObj, pos: *mut Vec3, dir: *mut Vec3);
    fn it_8026BB20(gobj: *mut HSD_GObj);
    fn it_8026BB44(gobj: *mut HSD_GObj);
    fn Item_80268E5C(gobj: *mut HSD_GObj, msid: i32, flags: u32);
    fn Item_8026A8EC(gobj: *mut HSD_GObj);
    fn Item_80268B18(spawn: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, flag: bool);
    fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing: f32, flag: bool);
    fn Item_8026AB54(gobj: *mut HSD_GObj, parent: *mut HSD_GObj, bone: u32);
    fn ftPp_SpecialS_8012114C(gobj: *mut HSD_GObj);
    fn ftLib_GetMotionId(gobj: *mut HSD_GObj) -> enum_t;
    }
    // Data defined in other TUs.
    pub static mut item_link_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

    // Callbacks of other TUs whose address this TU stores.
    pub unsafe extern "C" fn it_802A2474(_p: *mut c_void) {
        panic!("TODO: it_802A2474 is not ported")
    }
    pub unsafe extern "C" fn it_802A24A0(_g: *mut HSD_GObj) {
        panic!("TODO: it_802A24A0 is not ported")
    }
    pub unsafe extern "C" fn HSD_GObj_JObjCallback(_g: *mut HSD_GObj) {
        panic!("TODO: HSD_GObj_JObjCallback is not ported")
    }
}
// END GENERATED EXT

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ip->xC4_article_data->x4_specialAttributes` as a `T`.
#[inline]
unsafe fn sa<'a, T>(ip: *mut Item) -> &'a T {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const T)
}

/// `GET_FIGHTER(owner)` (the owner's user data).
#[inline]
unsafe fn fighter_of(owner: *mut HSD_GObj) -> *mut Fighter {
    (*owner).user_data as *mut Fighter
}

/// `Item_InitLinkMtx(m, z)` (src/melee/it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitLinkMtx(m: *mut Mtx, z: f32) {
    ext::PSMTXIdentity(m);
    (*m)[0][3] = 0.0;
    (*m)[1][3] = 0.0;
    (*m)[2][3] = z;
}

/// `Item_RetractChain` (src/melee/it/kinds/inlines.h, static inline): walk back from `link` until
/// the chain reaches `target_dist`, clearing the bend flags on the way; the remainder is capped
/// by `*max_dist`.
unsafe fn Item_RetractChain(link: *mut ItemLink, pos: *mut Vec3, target_dist: f32, max_dist: *const f32,
    next_out: *mut *mut ItemLink, cur_out: *mut *mut ItemLink, remaining_out: *mut f32)
{
    let mut cur = link;
    let mut next = (*link).prev;
    let mut dir: Vec3 = core::mem::zeroed();

    while !next.is_null() && (*cur).x2C_b0() == 0 {
        cur = next;
        next = (*next).prev;
    }

    let mut distance = it_802a3c98(&(*cur).pos, &*pos, &mut dir);

    while !next.is_null() && target_dist > distance {
        (*cur).set_x2C_b0(0);
        distance = it_802a3c98(&(*next).pos, &*pos, &mut dir);
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

/// `it_802C248C_setupGObj` (static inline): give a link gobj its joint and GX callback.
unsafe fn it_802C248C_setupGObj(link_gobj: *mut HSD_GObj, loaded_jobj: *mut HSD_JObj, callback: unsafe extern "C" fn(*mut HSD_GObj)) {
    ext::HSD_GObjObject_80390A70(link_gobj, HSD_GObj_JObjKind, loaded_jobj);
    ext::GObj_SetupGXLink(link_gobj, callback, 6, 0);
}

/// `it_802C248C` (itclimbersstring.c, static): build the `x0_count` links of the gum string
/// from the owner's hand joint `jobj`. Returns the tail link's gobj's hsd_obj, or NULL.
unsafe fn it_802C248C(ip: *mut Item, jobj: *mut HSD_JObj) -> *mut HSD_GObj {
    let attrs = sa::<itClimbersStringAttributes>(ip);
    let mut prev_link: *mut ItemLink = null_mut();
    let mut head_link: *mut ItemLink = null_mut();
    let mut tail_link: *mut ItemLink = null_mut();
    let mut result: *mut HSD_GObj = null_mut();
    let zero_vel = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let mut i: i32 = 0;
    while i < attrs.x0_count {
        let link_gobj = ext::GObj_Create(7, 0xA, 0);

        if link_gobj.is_null() {
            while !prev_link.is_null() {
                ext::HSD_GObjFree((*prev_link).gobj);
                prev_link = (*prev_link).next;
            }
            return null_mut();
        }

        let link = ext::HSD_ObjAlloc(&mut ext::item_link_alloc_data) as *mut ItemLink;
        it_80275328(link_gobj, &mut *link);
        ext::GObj_InitUserData(link_gobj, 6, ext::it_802A2474, link as *mut c_void);

        if i == 0 {
            (*link).next = null_mut();
            head_link = link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(0);
            it_802C248C_setupGObj(link_gobj, ext::HSD_JObjLoadJoint(it_802c248c_joint24(&*ip)), ext::it_802A24A0);
        } else if i == attrs.x0_count - 1 {
            (*prev_link).prev = link;
            (*link).prev = null_mut();
            tail_link = link;
            (*link).next = prev_link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(1);

            let joint = ext::HSD_JObjLoadJoint(it_802c248c_joint28(&*ip));
            ext::HSD_GObjObject_80390A70(link_gobj, HSD_GObj_JObjKind, joint);
            ext::GObj_SetupGXLink(link_gobj, ext::HSD_GObj_JObjCallback, 6, 0);
            (*ip).xDD4_itemVar.climbersstring.x14 = joint;
            result = (*link_gobj).hsd_obj as *mut HSD_GObj;
        } else {
            (*prev_link).prev = link;
            (*link).next = prev_link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(0);
            it_802C248C_setupGObj(link_gobj, ext::HSD_JObjLoadJoint(it_802c248c_joint24(&*ip)), ext::it_802A24A0);
        }
        prev_link = link;
        i += 1;
    }
    (*ip).xDD4_itemVar.climbersstring.x4 = tail_link;
    (*ip).xDD4_itemVar.climbersstring.x8 = head_link;
    result
}

/// `itClimbersstring_Cleanup` (static inline): drop the owner's grip and free the links.
unsafe fn itClimbersstring_Cleanup(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        let ip = item_of(gobj);
        if !(*ip).owner.is_null() {
            ext::ftPp_SpecialS_8012114C((*ip).owner);
        }
        (*ip).owner = null_mut();
        (*ip).xDD4_itemVar.climbersstring.xC = null_mut();
        let mut cur = (*ip).xDD4_itemVar.climbersstring.x4;
        while !cur.is_null() {
            let cur_gobj = (*cur).gobj;
            cur = (*cur).next;
            ext::HSD_GObjFree(cur_gobj);
        }
    }
}

/// `it_802C2750`: the item's destruction; cleanup, then `Item_8026A8EC`.
pub unsafe extern "C" fn it_802C2750(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if !ip.is_null() {
        if !gobj.is_null() {
            if !(*ip).owner.is_null() {
                ext::ftPp_SpecialS_8012114C((*ip).owner);
            }
            (*ip).owner = null_mut();
            (*ip).xDD4_itemVar.climbersstring.xC = null_mut();
            let mut cur = (*ip).xDD4_itemVar.climbersstring.x4;
            while !cur.is_null() {
                let cur_gobj = (*cur).gobj;
                cur = (*cur).next;
                ext::HSD_GObjFree(cur_gobj);
            }
        }
        ext::Item_8026A8EC(gobj);
    }
}

/// `it_802C27D4`: spawn the gum string from `owner`'s hand (`FtPart_L4thNb`).
pub unsafe extern "C" fn it_802C27D4(owner: *mut HSD_GObj, pos: *mut Vec3, _msid: i32, dir: f32) -> *mut HSD_GObj {
    let fp = fighter_of(owner);
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_IceClimber_GumStrings;
    ext::Item_InitSpawnPosition(&mut spawn, pos, false);
    ext::Item_InitSpawnCommonFields(&mut spawn, owner, dir, false);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = item_of(item_gobj);
        (*ip).xDD4_itemVar.climbersstring.xC = owner;
        it_802C248C(ip, (*(*fp).parts.add(FtPart_L4thNb as usize)).joint);
        ext::Item_8026AB54(item_gobj, owner, FtPart_L4thNb as u32);
    }
    item_gobj
}

/// `fn_802C28B8`: the on-accessory callback that runs the link-chain solver for the head.
pub unsafe extern "C" fn fn_802C28B8(gobj: *mut HSD_GObj) {
    it_802C33B8(item_of(gobj));
}

/// `fn_802C28DC` (static): the first chain solver (`it_802C2EC4`) for the tail link.
pub unsafe extern "C" fn fn_802C28DC(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = sa::<itClimbersStringAttributes>(ip);
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    (*link).set_x2C_b0(1);
    Item_InitLinkMtx(&mut m, 0.0);
    ext::HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &m, &mut m);
    pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    it_802C2EC4(link, &mut pos, attrs, ip);
    it_802C3520(ip, &mut pos);
}

/// `fn_802C29E8` (static): the second chain solver (`it_802C30E8`) for the tail link.
pub unsafe extern "C" fn fn_802C29E8(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = sa::<itClimbersStringAttributes>(ip);
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    (*link).set_x2C_b0(1);
    Item_InitLinkMtx(&mut m, 0.0);
    ext::HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &m, &mut m);
    pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    it_802C30E8(link, &mut pos, attrs, ip);
    it_802C3520(ip, &mut pos);
}

/// `fn_802C2AF4` (static): the retract (`it_802C32D4`) for the head link; picks up the item when
/// the chain is fully retracted.
pub unsafe extern "C" fn fn_802C2AF4(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = sa::<itClimbersStringAttributes>(ip);
    let link = (*ip).xDD4_itemVar.climbersstring.x8;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    Item_InitLinkMtx(&mut m, 0.0);
    ext::HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &m, &mut m);
    pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    if it_802C32D4(link, &mut pos, attrs, ip, (*ip).xDD4_itemVar.climbersstring.x0) {
        it_2725_Logic70_PickedUp(gobj);
    }
    it_802C3520(ip, &mut pos);
}

/// `itClimbersstring_UnkMotion3_Anim`: clean up once the owner leaves the grip motions
/// (0x15B..=0x164).
pub unsafe extern "C" fn itClimbersstring_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let should_cleanup: bool = if !(*ip).owner.is_null() {
        let action = ext::ftLib_GetMotionId((*ip).owner);
        !(action >= 0x15B && action <= 0x164)
    } else {
        true
    };
    if should_cleanup {
        itClimbersstring_Cleanup(gobj);
        return true;
    }
    false
}

/// `it_802C2CA8`: pull the chain toward `target`, keeping each bent link `attrs->x8` from the
/// next one.
pub unsafe fn it_802C2CA8(link: *mut ItemLink, target: *mut Vec3, attrs: &itClimbersStringAttributes, length: f32) {
    let mut link = link;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;

    it_802a3c98(&(*link).pos, &*target, &mut dir);
    (*link).pos.x = fmadds(dir.x, length, (*target).x);
    (*link).pos.y = fmadds(dir.y, length, (*target).y);
    (*link).pos.z = fmadds(dir.z, length, (*target).z);
    while !prev.is_null() {
        if (*prev).x2C_b0() != 0 {
            if it_802a3c98(&(*prev).pos, &(*link).pos, &mut dir) > attrs.x8 {
                (*prev).pos.x = fmadds(dir.x, attrs.x8, (*link).pos.x);
                (*prev).pos.y = fmadds(dir.y, attrs.x8, (*link).pos.y);
                (*prev).pos.z = fmadds(dir.z, attrs.x8, (*link).pos.z);
            }
        }
        link = prev;
        prev = (*prev).prev;
    }
}

/// `it_802C2DB0`: the falling version of the chain pull, applying gravity `attrs->x14` to each
/// link before solving.
pub unsafe fn it_802C2DB0(cur: *mut ItemLink, target: *mut Vec3, attrs: &itClimbersStringAttributes, length: f32) {
    let mut cur = cur;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*cur).prev;

    it_802a3c98(&(*cur).pos, &*target, &mut dir);
    (*cur).pos.x = fmadds(dir.x, length, (*target).x);
    (*cur).pos.y = fmadds(dir.y, length, (*target).y);
    (*cur).pos.z = fmadds(dir.z, length, (*target).z);
    while !prev.is_null() {
        (*prev).vel.y -= attrs.x14;
        it_802_a4420_call(prev);
        if it_802a3c98(&(*prev).pos, &(*cur).pos, &mut dir) > attrs.x8 {
            (*prev).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
            (*prev).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
            (*prev).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
        }
        cur = prev;
        prev = (*prev).prev;
    }
}

#[inline]
unsafe fn it_802_a4420_call(l: *mut ItemLink) {
    it_802a4420(&mut *l);
}

/// `it_802C2EC4`: solve the chain against the owner's hand position while the owner holds it.
/// Returns 2 when the chain is settled, 0 when it stopped short.
pub unsafe fn it_802C2EC4(link: *mut ItemLink, target: *mut Vec3, attrs: &itClimbersStringAttributes, ip: *mut Item) -> i32 {
    let mut cur = link;
    let mut iter = (*link).next;
    let fp = fighter_of((*ip).xDD4_itemVar.climbersstring.xC);
    let mut dir: Vec3 = core::mem::zeroed();
    let mut dir2: Vec3 = core::mem::zeroed();

    if (*fp).u.pp.x2240.x != 0.0 || (*fp).u.pp.x2240.y != 0.0 {
        (*link).pos = (*fp).u.pp.x2240;
    }
    (*link).vel.x = 0.0;
    (*link).vel.y = 0.0;
    (*link).vel.z = 0.0;

    while !iter.is_null() {
        if (*iter).x2C_b0() != 0 {
            (*iter).pos = *target;
            let dist = it_802a3c98(&(*iter).pos, &(*cur).pos, &mut dir);
            if dist > attrs.x8 {
                (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
            } else if dist < attrs.xC {
                if it_802a3c98(&(*iter).pos, &*target, &mut dir2) <= 0.1 {
                    (*iter).set_x2C_b0(0);
                } else {
                    (*iter).pos.x = fmadds(dir.x, attrs.xC, (*cur).pos.x);
                    (*iter).pos.y = fmadds(dir.y, attrs.xC, (*cur).pos.y);
                    (*iter).pos.z = fmadds(dir.z, attrs.xC, (*cur).pos.z);
                }
            }
        } else {
            if it_802a3c98(&*target, &(*cur).pos, &mut dir) > attrs.x8 {
                (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
                (*iter).set_x2C_b0(1);
            } else {
                return 0;
            }
        }
        cur = iter;
        iter = (*iter).next;
    }
    2
}

/// `it_802C2CA8_outline` (static inline): forwards to `it_802C2CA8`.
unsafe fn it_802C2CA8_outline(link: *mut ItemLink, target: *mut Vec3, attrs: &itClimbersStringAttributes, length: f32) {
    it_802C2CA8(link, target, attrs, length);
}

/// `it_802C30E8`: the swinging solver: gravity on each link, then the chain pull; 2 when settled,
/// 0 when a link is still too close to be moved.
pub unsafe fn it_802C30E8(link: *mut ItemLink, target: *mut Vec3, attrs: &itClimbersStringAttributes, ip: *mut Item) -> i32 {
    let mut cur = link;
    let mut iter = (*link).next;
    let mut dir: Vec3 = core::mem::zeroed();
    let fp = fighter_of((*ip).xDD4_itemVar.climbersstring.xC);

    (*link).vel.y = fmadds(-0.9, attrs.x14, (*link).vel.y);

    if (*fp).u.pp.x2240.x != 0.0 || (*fp).u.pp.x2240.y != 0.0 {
        (*link).pos = (*fp).u.pp.x2240;
    } else {
        it_802_a4420_call(cur);
    }

    let mut counter: i32 = 0;
    while !iter.is_null() {
        counter += 1;
        if (*iter).x2C_b0() != 0 {
            (*iter).vel.y -= attrs.x14;
            it_802_a4420_call(iter);
            if it_802a3c98(&(*iter).pos, &(*cur).pos, &mut dir) > attrs.x8 {
                (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
            }
        } else {
            if counter > attrs.x4 {
                (*iter).pos = *target;
            } else {
                if it_802a3c98(&*target, &(*cur).pos, &mut dir) > attrs.x8 {
                    (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                    (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                    (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
                    (*iter).set_x2C_b0(1);
                } else {
                    return 0;
                }
            }
        }
        cur = iter;
        iter = (*iter).next;
    }

    it_802C2CA8_outline(cur, target, attrs, attrs.x8);
    (*link).vel.x *= 0.9;
    2
}

/// `it_802C32D4`: retract the chain to `dist` from `pos`; true once the head is reached.
pub unsafe fn it_802C32D4(link: *mut ItemLink, pos: *mut Vec3, attrs: &itClimbersStringAttributes, _ip: *mut Item, dist: f32) -> bool {
    let mut prev: *mut ItemLink = null_mut();
    let mut cur: *mut ItemLink = null_mut();
    let mut step: f32 = 0.0;

    Item_RetractChain(link, pos, dist, &attrs.x8, &mut prev, &mut cur, &mut step);
    it_802C2DB0(cur, pos, attrs, step);

    if !prev.is_null() {
        return false;
    }
    true
}

/// `it_802C33B8`: pin the head link's gobj to the owner's hand, with no offset.
pub unsafe fn it_802C33B8(ip: *mut Item) {
    let mut m: Mtx = [[0.0; 4]; 3];
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let jobj = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
    let zero = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    (*link).set_x2C_b0(0);
    ext::HSD_JObjSetupMatrix((*link).jobj);

    ext::PSMTXIdentity(&mut m);
    m[0][3] = zero.x;
    m[1][3] = zero.y;
    m[2][3] = zero.z;
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &m, &mut m);

    ext::HSD_JObjCopyMtx(jobj, &m);
    (*jobj).flags |= 0x03800000;
    ext::HSD_JObjSetMtxDirty(jobj);
}

/// `it_802C3520`: place every link of the chain from the tail's bend point to the head, setting
/// each link's joint translation and scaling its matrix by `attrs->x8`.
pub unsafe fn it_802C3520(ip: *mut Item, target: *mut Vec3) {
    let mut link = (*ip).xDD4_itemVar.climbersstring.x8;
    let attrs = sa::<itClimbersStringAttributes>(ip);
    let mut m: Mtx = [[0.0; 4]; 3];

    while (*link).x2C_b0() == 0 {
        link = (*link).prev;
    }

    while !link.is_null() {
        let next_pos: Vec3 = if !(*link).next.is_null() {
            if (*(*link).next).x2C_b0() != 0 {
                (*(*link).next).pos
            } else {
                *target
            }
        } else {
            *target
        };

        let prev_pos: Vec3 = if !(*link).prev.is_null() {
            (*(*link).prev).pos
        } else {
            (*link).pos
        };

        let jobj = (*(*link).gobj).hsd_obj as *mut HSD_JObj;
        let mut link_pos: Vec3 = (*link).pos;
        ext::HSD_JObjSetTranslate(jobj, &link_pos);

        let mut dir = Vec3 {
            x: prev_pos.x - next_pos.x,
            y: prev_pos.y - next_pos.y,
            z: prev_pos.z - next_pos.z,
        };
        ext::lbVector_Normalize(&mut dir);

        ext::it_802A6DC8(jobj, &mut link_pos, &mut dir);

        ext::PSMTXCopy(&(*jobj).mtx, &mut m);
        m[0][0] *= attrs.x8;
        m[1][0] *= attrs.x8;
        m[2][0] *= attrs.x8;
        m[0][1] *= attrs.x8;
        m[1][1] *= attrs.x8;
        m[2][1] *= attrs.x8;
        m[0][2] *= attrs.x8;
        m[1][2] *= attrs.x8;
        m[2][2] *= attrs.x8;
        ext::PSMTXCopy(&m, &mut (*jobj).mtx);

        link = (*link).prev;
    }
}

/// `it_2725_Logic70_PickedUp`: show the gum string held (motion 0), then its on-accessory is
/// `fn_802C28B8`.
pub unsafe extern "C" fn it_2725_Logic70_PickedUp(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_8026BB44(gobj);
    (*ip).on_accessory = Some(fn_802C28B8);
}

/// `it_802C3810`: motion 2 (on-accessory `fn_802C29E8`).
pub unsafe extern "C" fn it_802C3810(gobj: *mut HSD_GObj) {
    let item = item_of(gobj);
    ext::Item_80268E5C(gobj, 2, 2);
    ext::it_8026BB20(gobj);
    (*item).on_accessory = Some(fn_802C29E8);
}

/// `it_802C3864`: motion 3 with the retract length set to 2.25 (on-accessory `fn_802C2AF4`).
pub unsafe extern "C" fn it_802C3864(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let link = (*ip).xDD4_itemVar.climbersstring.x8;
    let mut mtx: Mtx = [[0.0; 4]; 3];

    Item_InitLinkMtx(&mut mtx, 0.0);
    ext::HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &mtx, &mut mtx);
    (*ip).xDD4_itemVar.climbersstring.x0 = 2.25;
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    ext::it_8026BB20(gobj);
    (*ip).on_accessory = Some(fn_802C2AF4);
}

/// `it_802C3950`: motion 1 with the tail link placed at the owner's hand (on-accessory
/// `fn_802C28DC`).
pub unsafe extern "C" fn it_802C3950(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let mut mtx: Mtx = [[0.0; 4]; 3];

    Item_InitLinkMtx(&mut mtx, 0.0);
    ext::HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&(*(*link).jobj).mtx, &mtx, &mut mtx);
    let result = Vec3 { x: mtx[0][3], y: mtx[1][3], z: mtx[2][3] };
    (*link).pos = result;
    (*link).coll_data.cur_pos = (*link).pos;
    (*link).coll_data.last_pos = (*link).coll_data.cur_pos;
    (*link).set_x2C_b0(1);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_8026BB20(gobj);
    (*ip).on_accessory = Some(fn_802C28DC);
}

/// `it_2725_Logic70_EvtUnk`: `it_8026B894`, then forget the owner if it is the reference.
pub unsafe extern "C" fn it_2725_Logic70_EvtUnk(arg0: *mut HSD_GObj, arg1: *mut HSD_GObj) {
    let ip = item_of(arg0);
    it_8026b894(&mut *ip, arg1);
    if (*ip).xDD4_itemVar.climbersstring.xC == arg1 {
        (*ip).xDD4_itemVar.climbersstring.xC = null_mut();
    }
}

/// `it_803F76B8[]` (itclimbersstring.c): four entries, all `itClimbersstring_UnkMotion3_Anim`.
pub static it_803F76B8: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: -1, animated: Some(itClimbersstring_UnkMotion3_Anim_p), physics_updated: None, collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itClimbersstring_UnkMotion3_Anim_p), physics_updated: None, collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itClimbersstring_UnkMotion3_Anim_p), physics_updated: None, collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itClimbersstring_UnkMotion3_Anim_p), physics_updated: None, collided: None },
];

/// `itClimbersstring_UnkMotion3_Anim` as the `int`-returning predicate of the state table.
pub unsafe extern "C" fn itClimbersstring_UnkMotion3_Anim_p(g: *mut HSD_GObj) -> i32 {
    itClimbersstring_UnkMotion3_Anim(g) as i32
}
