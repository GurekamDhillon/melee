//! Native port of src/melee/it/kinds/itgamewatchrescue.c (Game & Watch's Fire Rescue item). Callees in
//! other TUs go through `ext` (TODO hooks). `bool` is the C `int`: callee results are tested as ints and
//! `true` is 1. Checked against the whole C TU (shipping contraction) by tests/tu_itgamewatchrescue.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut)]
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn ftGw_SpecialHi_ItemCheckRescueRemove(a0: *mut HSD_GObj) -> i32;
    fn ftGw_SpecialHi_ItemRescueSetNULL(a0: *mut HSD_GObj);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8027CE64(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut c_void) => unsafe { crate::tu::itzako::it_8027CE64(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline]
unsafe fn GET_ITEM(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.x = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.z = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `void** attr = item->xC4_article_data->x4_specialAttributes`.
#[inline]
unsafe fn special_attributes(item: *mut Item) -> *mut *mut c_void {
    (*(*item).xC4_article_data).x4_specialAttributes as *mut *mut c_void
}

/// `Item_ClearCmdVars` (it/inlines.h, static inline).
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `ItemStateTable it_803F79C0[]`
pub static mut it_803F79C0: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchrescue_UnkMotion1_Anim), physics_updated: Some(itGamewatchrescue_UnkMotion1_Phys), collided: Some(itGamewatchrescue_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchrescue_UnkMotion1_Anim), physics_updated: Some(itGamewatchrescue_UnkMotion1_Phys), collided: Some(itGamewatchrescue_UnkMotion1_Coll) },
];

/// `Item_GObj* it_802C8038(Item_GObj* parent, Vec3* arg1, s32 arg2, s32 arg3, f32 farg0, f32 farg1)`
pub unsafe extern "C" fn it_802C8038(parent: *mut HSD_GObj, arg1: *mut Vec, arg2: i32, arg3: i32, farg0: f32, farg1: f32) -> *mut HSD_GObj {
    melee_core::ct!("it_802C8038");
    let result: *mut HSD_GObj;
    if !parent.is_null() {
        let mut spawn: SpawnItem = core::mem::zeroed();
        spawn.kind = It_Kind_GameWatch_Rescue;
        Item_InitSpawnPosition(&mut spawn, arg1, true);
        Item_InitSpawnCommonFields(&mut spawn, parent, farg0, false);
        result = ext::Item_80268B18(&mut spawn);
        if !result.is_null() {
            let temp_r6 = (*result).user_data as *mut Item;
            let tmp = special_attributes(temp_r6);
            Item_ClearCmdVars(temp_r6);
            (*temp_r6).xDCC_flag.set_b3(0);
            (*temp_r6).xDD4_itemVar.gamewatchrescue.xDD8 = parent;
            ext::it_8027CE64(result, parent, *tmp);
            it_802C8208(result, arg3);
        }
    } else {
        result = null_mut();
    }
    result
}

pub unsafe extern "C" fn it_802C8158(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C8158");
    let item = GET_ITEM(item_gobj);
    if !item_gobj.is_null() && !item.is_null() {
        let temp_r3 = (*item).xDD4_itemVar.gamewatchrescue.xDD8;
        if !temp_r3.is_null() && (*item).owner == temp_r3 {
            ext::ftGw_SpecialHi_ItemRescueSetNULL(temp_r3);
        }
        (*item).xDD4_itemVar.gamewatchrescue.xDD8 = null_mut();
        (*item).owner = null_mut();
        ext::Item_8026A8EC(item_gobj);
    }
}

pub unsafe extern "C" fn it_802C81C8(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C81C8");
    ext::it_8026B724(item_gobj);
}

pub unsafe extern "C" fn it_802C81E8(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C81E8");
    ext::it_8026B73C(item_gobj);
}

pub unsafe extern "C" fn it_802C8208(hsd_gobj: *mut HSD_GObj, msid: enum_t) {
    melee_core::ct!("it_802C8208");
    ext::Item_80268E5C(hsd_gobj, msid, ITEM_ANIM_UPDATE as u32);
    ext::Item_802694CC(hsd_gobj);
}

pub unsafe extern "C" fn itGamewatchrescue_UnkMotion1_Anim(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGamewatchrescue_UnkMotion1_Anim");
    let item1 = GET_ITEM(item_gobj);
    let temp_r3 = (*item1).xDD4_itemVar.gamewatchrescue.xDD8;
    if !temp_r3.is_null() {
        if ext::ftGw_SpecialHi_ItemCheckRescueRemove(temp_r3) == 1 {
            let item2 = GET_ITEM(item_gobj);
            if !item_gobj.is_null() && !item2.is_null() {
                let temp_r3_2 = (*item2).xDD4_itemVar.gamewatchrescue.xDD8;
                if !temp_r3_2.is_null() && (*item2).owner == temp_r3_2 {
                    ext::ftGw_SpecialHi_ItemRescueSetNULL(temp_r3_2);
                }
                (*item2).xDD4_itemVar.gamewatchrescue.xDD8 = null_mut();
                (*item2).owner = null_mut();
                ext::Item_8026A8EC(item_gobj);
            }
            if !item_gobj.is_null() {
                let temp_r3_3 = GET_ITEM(item_gobj);
                (*temp_r3_3).xDD4_itemVar.gamewatchrescue.xDD8 = null_mut();
                (*temp_r3_3).owner = null_mut();
            }
            return 1;
        }
        return 0;
    }
    if !item_gobj.is_null() && !item1.is_null() {
        // the C also tests `temp_r3 != NULL && item1->owner == temp_r3` here; temp_r3 is NULL on this path
        (*item1).xDD4_itemVar.gamewatchrescue.xDD8 = null_mut();
        (*item1).owner = null_mut();
        ext::Item_8026A8EC(item_gobj);
    }
    if !item_gobj.is_null() {
        let temp_r3_4 = GET_ITEM(item_gobj);
        (*temp_r3_4).xDD4_itemVar.gamewatchrescue.xDD8 = null_mut();
        (*temp_r3_4).owner = null_mut();
    }
    1
}

pub unsafe extern "C" fn itGamewatchrescue_UnkMotion1_Phys(gobj: *mut HSD_GObj) { melee_core::ct!("itGamewatchrescue_UnkMotion1_Phys");}

pub unsafe extern "C" fn itGamewatchrescue_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGamewatchrescue_UnkMotion1_Coll");
    0
}

pub unsafe extern "C" fn itGameWatchRescue_Logic81_EvtUnk(item_gobj1: *mut HSD_GObj, item_gobj2: *mut HSD_GObj) {
    melee_core::ct!("itGameWatchRescue_Logic81_EvtUnk");
    ext::it_8026B894(item_gobj1, item_gobj2);
}
