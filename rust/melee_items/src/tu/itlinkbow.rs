//! Native port of src/melee/it/kinds/itlinkbow.c: Link's Bow (and Kirby's copy) item: spawn, the per-frame
//! scale / state follow of the owner's SpecialN index, and the six states. Callees in other TUs go through
//! `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction) by
//! tests/tu_itlinkbow.rs. No contracted float sites (copies and compares only).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268DD4(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::item::Item_80268DD4(a0, a1) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftKb_SpecialNLk800FB394(a0: *mut HSD_GObj) -> u32;
    fn ftKb_SpecialNLk800FB444(a0: *mut HSD_GObj);
    fn ftKb_SpecialNLk800FB470(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32;
    fn ftLk_SpecialN_GetIndex(a0: *mut HSD_GObj) -> u32;
    fn ftLk_SpecialN_IsActiveAnd2071b6(a0: *mut HSD_GObj) -> i32;
    fn ftLk_SpecialN_UnsetFv14(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::inl_it_core!();
crate::inl_it_jobj!();

pub static mut it_803F6E98: [ItemStateTable; 7] = [
    ItemStateTable { anim_id: 0, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: 5, animated: Some(itLinkbow_UnkMotion5_Anim), physics_updated: Some(itLinkbow_UnkMotion5_Phys), collided: Some(itLinkbow_UnkMotion5_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itLinkbow_UnkMotion6_Anim), physics_updated: Some(itLinkbow_UnkMotion6_Phys), collided: Some(itLinkbow_UnkMotion6_Coll) },
];

static it_803F6F08: [i32; 7] = [0, 1, 2, 3, 4, 5, 6];
static it_803F6F24: [i32; 7] = [3, 4, 5, 0, 1, 2, 6];

pub unsafe fn it_802AF1A4(facing_dir: f32, owner_gobj: *mut Fighter_GObj, vec: *mut Vec3, arg3: Fighter_Part, arg4: i32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = arg4;
    Item_InitSpawnPosition(&mut spawn, vec, true);
    Item_InitSpawnCommonFields(&mut spawn, owner_gobj, facing_dir, true);

    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let item = GET_ITEM(gobj);
        Item_ClearCmdVars(item);
        (*item).xDCC_flag.set_b3(0);
        (*item).xDD4_itemVar.linkbow.x0 = ext::ftLib_800869D4((*item).owner);
        (*item).xDD4_itemVar.linkbow.x4 = owner_gobj;
        ext::Item_8026AB54(gobj, owner_gobj, arg3);
    }
    gobj
}

pub unsafe fn itLinkBow_Logic100_Destroyed(arg0: *mut Item_GObj) {
    if !arg0.is_null() {
        let item = GET_ITEM(arg0);
        if !(*item).owner.is_null() {
            match (*item).kind {
                It_Kind_Link_Bow | It_Kind_CLink_Bow => ext::ftLk_SpecialN_UnsetFv14((*item).owner),
                It_Kind_Kirby_CLinkBow | It_Kind_Kirby_LinkBow => ext::ftKb_SpecialNLk800FB444((*item).owner),
                _ => {}
            }
        }
    }
}

pub unsafe fn it_802AF304(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe fn it_802AF32C(arg0: *mut HSD_GObj) {
    let item = GET_ITEM(arg0);
    let mut arg1: u32 = 0;

    if !(*item).xDD4_itemVar.linkbow.x4.is_null() {
        if (*item).owner == (*item).xDD4_itemVar.linkbow.x4 {
            match (*item).kind {
                It_Kind_Link_Bow | It_Kind_CLink_Bow => arg1 = ext::ftLk_SpecialN_GetIndex((*item).owner),
                It_Kind_Kirby_CLinkBow | It_Kind_Kirby_LinkBow => arg1 = ext::ftKb_SpecialNLk800FB394((*item).owner),
                _ => {}
            }
            if (*item).msid != it_803F6F08[arg1 as usize] {
                if (*item).msid == it_803F6F24[arg1 as usize] {
                    let temp_f31 = (*item).x5CC_currentAnimFrame;
                    ext::Item_80268E5C(arg0, arg1 as i32, ITEM_ANIM_UPDATE as u32);
                    ext::Item_802694CC(arg0);
                    ext::Item_80268DD4(arg0, temp_f31);
                    return;
                }
                ext::Item_80268E5C(arg0, arg1 as i32, ITEM_ANIM_UPDATE as u32);
                ext::Item_802694CC(arg0);
            }
        }
    }
}

pub unsafe fn itLinkBow_Logic100_PickedUp(arg0: *mut Item_GObj) {
    let item = GET_ITEM(arg0);
    let jobj = HSD_GObjGetHSDObj(arg0) as *mut HSD_JObj;
    let mut scale: Vec3 = core::mem::zeroed();
    let mut idx: u32 = 0;

    if !(*item).xDD4_itemVar.linkbow.x4.is_null() {
        if (*item).owner == (*item).xDD4_itemVar.linkbow.x4 {
            match (*item).kind {
                It_Kind_Link_Bow | It_Kind_CLink_Bow => idx = ext::ftLk_SpecialN_GetIndex((*item).owner),
                It_Kind_Kirby_CLinkBow | It_Kind_Kirby_LinkBow => idx = ext::ftKb_SpecialNLk800FB394((*item).owner),
                _ => {}
            }
            match idx {
                ftLk_SpecialNIndex_Start => {
                    ext::Item_80268E5C(arg0, ftLk_SpecialNIndex_Start as i32, ITEM_ANIM_UPDATE as u32);
                    ext::Item_802694CC(arg0);
                    scale.x = (*item).xDD4_itemVar.linkbow.x0;
                    scale.y = scale.x;
                    scale.z = scale.y;
                    HSD_JObjSetScale(jobj, &mut scale);
                }
                ftLk_SpecialNIndex_AirStart | ftLk_SpecialNIndex_None => {
                    ext::Item_80268E5C(arg0, ftLk_SpecialNIndex_AirStart as i32, ITEM_ANIM_UPDATE as u32);
                    ext::Item_802694CC(arg0);
                    scale.x = (*item).xDD4_itemVar.linkbow.x0;
                    scale.y = scale.x;
                    scale.z = scale.y;
                    HSD_JObjSetScale(jobj, &mut scale);
                }
                _ => {}
            }
        }
    }
}

pub unsafe extern "C" fn itLinkbow_UnkMotion5_Anim(arg0: *mut Item_GObj) -> i32 {
    let item = GET_ITEM(arg0);
    let jobj = HSD_GObjGetHSDObj(arg0) as *mut HSD_JObj;
    let mut scale: Vec3 = core::mem::zeroed();

    scale.x = (*item).xDD4_itemVar.linkbow.x0;
    scale.y = scale.x;
    scale.z = scale.y;
    HSD_JObjSetScale(jobj, &mut scale);

    match (*item).msid {
        5 | 2 => {
            if (*item).x5CC_currentAnimFrame == 0.0 || (*item).x5CC_currentAnimFrame >= 24.0 {
                itLinkBow_Logic100_Destroyed(arg0);
                return 1;
            }
        }
        _ => it_802AF32C(arg0),
    }

    if (*item).xDD4_itemVar.linkbow.x4.is_null() {
        return 1;
    }
    if (*item).owner != (*item).xDD4_itemVar.linkbow.x4 {
        return 1;
    }
    if !(*item).owner.is_null() {
        match (*item).kind {
            It_Kind_Link_Bow | It_Kind_CLink_Bow => {
                if ext::ftLk_SpecialN_IsActiveAnd2071b6((*item).owner) != 0 {
                    return 1;
                }
            }
            It_Kind_Kirby_CLinkBow | It_Kind_Kirby_LinkBow => {
                if ext::ftKb_SpecialNLk800FB470((*item).owner) != 0 {
                    return 1;
                }
            }
            _ => {}
        }
    }
    0
}

pub unsafe extern "C" fn itLinkbow_UnkMotion5_Phys(_arg0: *mut HSD_GObj) {}

pub unsafe extern "C" fn itLinkbow_UnkMotion5_Coll(arg0: *mut Item_GObj) -> i32 {
    let item = GET_ITEM(arg0);
    let jobj = HSD_GObjGetHSDObj(arg0) as *mut HSD_JObj;
    let mut scale: Vec3 = core::mem::zeroed();
    scale.x = (*item).xDD4_itemVar.linkbow.x0;
    scale.y = scale.x;
    scale.z = scale.y;
    HSD_JObjSetScale(jobj, &mut scale);
    0
}

pub unsafe extern "C" fn itLinkbow_UnkMotion6_Anim(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe extern "C" fn itLinkbow_UnkMotion6_Phys(_arg0: *mut HSD_GObj) {}

pub unsafe extern "C" fn itLinkbow_UnkMotion6_Coll(_arg0: *mut Item_GObj) -> i32 {
    1
}

pub unsafe fn itLinkBow_Logic100_EvtUnk(arg0: *mut Item_GObj, arg1: *mut Item_GObj) {
    ext::it_8026B894(arg0, arg1);
}
