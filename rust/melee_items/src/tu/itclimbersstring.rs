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

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void;
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXIdentity(a0: *mut c_void);
    fn ftLib_GetMotionId(a0: *mut HSD_GObj) -> i32;
    fn ftPp_SpecialS_8012114C(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB20(a0: *mut HSD_GObj);
    fn it_8026BB44(a0: *mut HSD_GObj);
    fn it_80275328(a0: *mut HSD_GObj, a1: *mut ItemLink);
    fn it_802A2474(a0: *mut c_void);
    fn it_802A24A0(a0: *mut HSD_GObj, a1: i32);
    fn it_802A3C98(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> f32;
    fn it_802A4420(a0: *mut ItemLink);
    fn it_802A6DC8(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    fn lbVector_Normalize(a0: *mut Vec) -> f32;
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
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut cur: *mut ItemLink = unsafe { core::mem::zeroed() };
    pub static mut dir: Vec3 = unsafe { core::mem::zeroed() };
    pub static mut distance: f32 = unsafe { core::mem::zeroed() };
    pub static mut item_link_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
    pub static mut limit: f32 = unsafe { core::mem::zeroed() };
    pub static mut next: *mut ItemLink = unsafe { core::mem::zeroed() };
    pub static mut remaining: f32 = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

jobj_inl!();
spawn_inl!();

/// `HSD_JObjSetupMatrix` (jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

/// `HSD_JObjCopyMtx` (jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *mut Mtx) {
    assert!(!jobj.is_null(), "HSD_ASSERT(1170, jobj)");
    assert!(!mtx.is_null(), "HSD_ASSERT(1171, mtx)");
    ext::PSMTXCopy(mtx as *mut c_void, &mut (*jobj).mtx as *mut Mtx as *mut c_void);
}

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
pub unsafe fn Item_InitLinkMtx(m: *mut Mtx, z: f32) {
    ext::PSMTXIdentity(m as *mut c_void);
    (*m)[0][3] = 0.0;
    (*m)[1][3] = 0.0;
    (*m)[2][3] = z;
}

/// `Item_RetractChain` (src/melee/it/kinds/inlines.h, static inline): walk back from `link` until
/// the chain reaches `target_dist`, clearing the bend flags on the way; the remainder is capped
/// by `*max_dist`.
pub unsafe fn Item_RetractChain(link: *mut ItemLink, pos: *mut Vec3, target_dist: f32, max_dist: *const f32,
    next_out: *mut *mut ItemLink, cur_out: *mut *mut ItemLink, remaining_out: *mut f32)
{
    let mut cur = link;
    let mut next = (*link).prev;
    let mut dir: Vec3 = core::mem::zeroed();

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

/// `it_802C248C_setupGObj` (static inline): give a link gobj its joint and GX callback.
pub unsafe fn it_802C248C_setupGObj(link_gobj: *mut HSD_GObj, loaded_jobj: *mut HSD_JObj, callback: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>) {
    ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, loaded_jobj as *mut c_void);
    ext::GObj_SetupGXLink(link_gobj, callback, 6, 0);
}

/// `it_802C248C_joint24` (static inline).
pub unsafe fn it_802C248C_joint24(ip: *mut Item) -> *mut HSD_Joint {
    (*sa::<itClimbersStringAttributes>(ip)).x24_joint
}

/// `it_802C248C_joint28` (static inline).
pub unsafe fn it_802C248C_joint28(ip: *mut Item) -> *mut HSD_Joint {
    (*sa::<itClimbersStringAttributes>(ip)).x28_joint
}

/// `it_802C248C` (itclimbersstring.c, static): build the `x0_count` links of the gum string
/// from the owner's hand joint `jobj`. Returns the tail link's gobj's hsd_obj, or NULL.
pub unsafe fn it_802C248C(ip: *mut Item, jobj: *mut HSD_JObj) -> *mut HSD_GObj {
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
        ext::it_80275328(link_gobj, link);
        ext::GObj_InitUserData(link_gobj, 6, Some(ext::cb::it_802A2474), link as *mut c_void);

        if i == 0 {
            (*link).next = null_mut();
            head_link = link;
            (*link).jobj = jobj;
            (*link).gobj = link_gobj;
            (*link).vel = zero_vel;
            (*link).pos = (*ip).pos;
            (*link).set_x2C_b0(0);
            it_802C248C_setupGObj(link_gobj, ext::HSD_JObjLoadJoint(it_802C248C_joint24(ip)), Some(ext::cb::it_802A24A0));
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

            let joint = ext::HSD_JObjLoadJoint(it_802C248C_joint28(ip));
            ext::HSD_GObjObject_80390A70(link_gobj, ext::HSD_GObj_JObjKind, joint as *mut c_void);
            ext::GObj_SetupGXLink(link_gobj, Some(ext::cb::HSD_GObj_JObjCallback), 6, 0);
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
            it_802C248C_setupGObj(link_gobj, ext::HSD_JObjLoadJoint(it_802C248C_joint24(ip)), Some(ext::cb::it_802A24A0));
        }
        prev_link = link;
        i += 1;
    }
    (*ip).xDD4_itemVar.climbersstring.x4 = tail_link;
    (*ip).xDD4_itemVar.climbersstring.x8 = head_link;
    result
}

/// `itClimbersstring_Cleanup` (static inline): drop the owner's grip and free the links.
pub unsafe fn itClimbersstring_Cleanup(gobj: *mut HSD_GObj) {
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
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, owner, dir, false);
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
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itClimbersStringAttributes;
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    (*link).set_x2C_b0(1);
    Item_InitLinkMtx(&mut m, 0.0);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void);
    pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    it_802C2EC4(link, &mut pos, attrs, ip);
    it_802C3520(ip, &mut pos);
}

/// `fn_802C29E8` (static): the second chain solver (`it_802C30E8`) for the tail link.
pub unsafe extern "C" fn fn_802C29E8(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itClimbersStringAttributes;
    let link = (*ip).xDD4_itemVar.climbersstring.x4;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    (*link).set_x2C_b0(1);
    Item_InitLinkMtx(&mut m, 0.0);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void);
    pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    it_802C30E8(link, &mut pos, attrs, ip);
    it_802C3520(ip, &mut pos);
}

/// `fn_802C2AF4` (static): the retract (`it_802C32D4`) for the head link; picks up the item when
/// the chain is fully retracted.
pub unsafe extern "C" fn fn_802C2AF4(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itClimbersStringAttributes;
    let link = (*ip).xDD4_itemVar.climbersstring.x8;
    let mut m: Mtx = [[0.0; 4]; 3];
    let mut pos: Vec3;

    Item_InitLinkMtx(&mut m, 0.0);
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void);
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
pub unsafe fn it_802C2CA8(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itClimbersStringAttributes, length: f32) {
    let attrs = &*attrs;
    let mut link = link;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*link).prev;

    ext::it_802A3C98(&mut (*link).pos, target, &mut dir);
    (*link).pos.x = fmadds(dir.x, length, (*target).x);
    (*link).pos.y = fmadds(dir.y, length, (*target).y);
    (*link).pos.z = fmadds(dir.z, length, (*target).z);
    while !prev.is_null() {
        if (*prev).x2C_b0() != 0 {
            if ext::it_802A3C98(&mut (*prev).pos, &mut (*link).pos, &mut dir) > attrs.x8 {
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
pub unsafe fn it_802C2DB0(cur: *mut ItemLink, target: *mut Vec3, attrs: *mut itClimbersStringAttributes, length: f32) {
    let attrs = &*attrs;
    let mut cur = cur;
    let mut dir: Vec3 = core::mem::zeroed();
    let mut prev = (*cur).prev;

    ext::it_802A3C98(&mut (*cur).pos, target, &mut dir);
    (*cur).pos.x = fmadds(dir.x, length, (*target).x);
    (*cur).pos.y = fmadds(dir.y, length, (*target).y);
    (*cur).pos.z = fmadds(dir.z, length, (*target).z);
    while !prev.is_null() {
        (*prev).vel.y -= attrs.x14;
        ext::it_802A4420(prev);
        if ext::it_802A3C98(&mut (*prev).pos, &mut (*cur).pos, &mut dir) > attrs.x8 {
            (*prev).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
            (*prev).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
            (*prev).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
        }
        cur = prev;
        prev = (*prev).prev;
    }
}

/// `it_802C2EC4`: solve the chain against the owner's hand position while the owner holds it.
/// Returns 2 when the chain is settled, 0 when it stopped short.
pub unsafe fn it_802C2EC4(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itClimbersStringAttributes, ip: *mut Item) -> i32 {
    let attrs = &*attrs;
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
            let dist = ext::it_802A3C98(&mut (*iter).pos, &mut (*cur).pos, &mut dir);
            if dist > attrs.x8 {
                (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
            } else if dist < attrs.xC {
                if ext::it_802A3C98(&mut (*iter).pos, target, &mut dir2) <= 0.1 {
                    (*iter).set_x2C_b0(0);
                } else {
                    (*iter).pos.x = fmadds(dir.x, attrs.xC, (*cur).pos.x);
                    (*iter).pos.y = fmadds(dir.y, attrs.xC, (*cur).pos.y);
                    (*iter).pos.z = fmadds(dir.z, attrs.xC, (*cur).pos.z);
                }
            }
        } else {
            if ext::it_802A3C98(target, &mut (*cur).pos, &mut dir) > attrs.x8 {
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
pub unsafe fn it_802C2CA8_outline(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itClimbersStringAttributes, length: f32) {
    it_802C2CA8(link, target, attrs, length);
}

/// `it_802C30E8`: the swinging solver: gravity on each link, then the chain pull; 2 when settled,
/// 0 when a link is still too close to be moved.
pub unsafe fn it_802C30E8(link: *mut ItemLink, target: *mut Vec3, attrs: *mut itClimbersStringAttributes, ip: *mut Item) -> i32 {
    let attrs = &*attrs;
    let mut cur = link;
    let mut iter = (*link).next;
    let mut dir: Vec3 = core::mem::zeroed();
    let fp = fighter_of((*ip).xDD4_itemVar.climbersstring.xC);

    (*link).vel.y = fmadds(-0.9, attrs.x14, (*link).vel.y);

    if (*fp).u.pp.x2240.x != 0.0 || (*fp).u.pp.x2240.y != 0.0 {
        (*link).pos = (*fp).u.pp.x2240;
    } else {
        ext::it_802A4420(cur);
    }

    let mut counter: i32 = 0;
    while !iter.is_null() {
        counter += 1;
        if (*iter).x2C_b0() != 0 {
            (*iter).vel.y -= attrs.x14;
            ext::it_802A4420(iter);
            if ext::it_802A3C98(&mut (*iter).pos, &mut (*cur).pos, &mut dir) > attrs.x8 {
                (*iter).pos.x = fmadds(dir.x, attrs.x8, (*cur).pos.x);
                (*iter).pos.y = fmadds(dir.y, attrs.x8, (*cur).pos.y);
                (*iter).pos.z = fmadds(dir.z, attrs.x8, (*cur).pos.z);
            }
        } else {
            if counter > attrs.x4 {
                (*iter).pos = *target;
            } else {
                if ext::it_802A3C98(target, &mut (*cur).pos, &mut dir) > attrs.x8 {
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

    it_802C2CA8(cur, target, attrs as *const _ as *mut _, attrs.x8);
    (*link).vel.x *= 0.9;
    2
}

/// `it_802C32D4`: retract the chain to `dist` from `pos`; true once the head is reached.
pub unsafe fn it_802C32D4(link: *mut ItemLink, pos: *mut Vec3, attrs: *mut itClimbersStringAttributes, _ip: *mut Item, dist: f32) -> bool {
    let attrs = &*attrs;
    let mut prev: *mut ItemLink = null_mut();
    let mut cur: *mut ItemLink = null_mut();
    let mut step: f32 = 0.0;

    Item_RetractChain(link, pos, dist, &attrs.x8, &mut prev, &mut cur, &mut step);
    it_802C2DB0(cur, pos, attrs as *const _ as *mut _, step);

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
    HSD_JObjSetupMatrix((*link).jobj);

    ext::PSMTXIdentity(&mut m as *mut Mtx as *mut c_void);
    m[0][3] = zero.x;
    m[1][3] = zero.y;
    m[2][3] = zero.z;
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void);

    HSD_JObjCopyMtx(jobj, &mut m);
    (*jobj).flags |= 0x03800000;
    HSD_JObjSetMtxDirty(jobj);
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
        HSD_JObjSetTranslate(jobj, &mut link_pos);

        let mut dir = Vec3 {
            x: prev_pos.x - next_pos.x,
            y: prev_pos.y - next_pos.y,
            z: prev_pos.z - next_pos.z,
        };
        ext::lbVector_Normalize(&mut dir);

        ext::it_802A6DC8(jobj, &mut link_pos, &mut dir);

        ext::PSMTXCopy(&mut (*jobj).mtx as *mut Mtx as *mut c_void, &mut m as *mut Mtx as *mut c_void);
        m[0][0] *= attrs.x8;
        m[1][0] *= attrs.x8;
        m[2][0] *= attrs.x8;
        m[0][1] *= attrs.x8;
        m[1][1] *= attrs.x8;
        m[2][1] *= attrs.x8;
        m[0][2] *= attrs.x8;
        m[1][2] *= attrs.x8;
        m[2][2] *= attrs.x8;
        ext::PSMTXCopy(&mut m as *mut Mtx as *mut c_void, &mut (*jobj).mtx as *mut Mtx as *mut c_void);

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
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut mtx as *mut Mtx as *mut c_void, &mut mtx as *mut Mtx as *mut c_void);
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
    HSD_JObjSetupMatrix((*link).jobj);
    ext::PSMTXConcat(&mut (*(*link).jobj).mtx as *mut Mtx as *mut c_void, &mut mtx as *mut Mtx as *mut c_void, &mut mtx as *mut Mtx as *mut c_void);
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
    ext::it_8026B894(arg0, arg1);
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
