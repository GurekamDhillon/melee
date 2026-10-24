//! Port of src/melee/it/kinds/itmewtwodisable.c: Mewtwo's Disable projectile (logic table 67). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itmewtwodisable.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftLib_80086A0C(a0: *mut HSD_GObj) -> f32;
    fn ftMt_SpecialLw_ClearDisableGObj(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_802755C0(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::itlib::it_802755c0(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F7750[1]`
pub static mut it_803F7750: [ItemStateTable; 1] = [
    st_row(0, pred_b(itMewtwodisable_UnkMotion0_Anim), Some(itMewtwodisable_UnkMotion0_Phys), pred_b_i(itMewtwodisable_UnkMotion0_Coll)),
];

/// A state-table predicate slot from an `int`-returning function.
const fn pred_b_i(f: unsafe extern "C" fn(*mut HSD_GObj) -> i32) -> HSD_GObjPredicate {
    unsafe { core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj) -> i32, HSD_GObjPredicate>(f) }
}

/// `Item_TickLifetime` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_Destroy(item_gobj: *mut Item_GObj) {
    ext::Item_8026A8EC(item_gobj);
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_Destroyed(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    if !(*item).xDD4_itemVar.mdisable.owner.is_null() {
        ext::ftMt_SpecialLw_ClearDisableGObj((*item).xDD4_itemVar.mdisable.owner);
    }
    (*item).xDD4_itemVar.mdisable.owner = null_mut();
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_SpawnMewtwoDisable(
    fighter_gobj: *mut Fighter_GObj,
    offset: *mut Vec3,
    facing_dir: f32,
) -> *mut Item_GObj {
    let mut spawnitem: SpawnItem = core::mem::zeroed();

    spawnitem.kind = It_Kind_Mewtwo_Disable;
    spawnitem.prev_pos = *offset;
    // set item position relative to fp bone
    ext::it_8026BB68(fighter_gobj, &mut spawnitem.pos);
    spawnitem.facing_dir = facing_dir;
    spawnitem.x3C_damage = 0;
    spawnitem.vel.z = 0.0;
    spawnitem.vel.y = 0.0;
    spawnitem.vel.x = 0.0;
    spawnitem.x0_parent_gobj = fighter_gobj;
    spawnitem.x4_parent_gobj2 = spawnitem.x0_parent_gobj;
    spawnitem.x44_flag.anon_0.set_b0(1);
    spawnitem.x40 = 0;

    let item_gobj = ext::Item_80268B18(&mut spawnitem);
    if !item_gobj.is_null() {
        let item = GET_ITEM(item_gobj);
        let vars = &mut (*item).xDD4_itemVar.mdisable as *mut itMDisable_ItemVars;
        let owner = &mut (*vars).owner as *mut *mut Fighter_GObj;
        *owner = (*item).owner;
        it_802C4B38(item_gobj);
        ext::it_8026B3A8(item_gobj);
        ext::db_80225DD8(item_gobj, fighter_gobj);
        (*item).xDCC_flag.set_b3(1);
    }
    item_gobj
}

pub unsafe extern "C" fn it_802C4B38(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itMDisableAttributes;
    let fighter_gobj = (*item).xDD4_itemVar.mdisable.owner;

    // the last parameter gets the y scaling of mewtwo
    (*item).x40_vel.x = (*attrs).x_vel * (*item).facing_dir * ext::ftLib_80086A0C(fighter_gobj);
    (*item).x40_vel.z = 0.0;
    (*item).x40_vel.y = 0.0;
    // set mdisable lifetime timers
    ext::it_80275158(item_gobj, (*attrs).lifetime);
    it_802C4BB8(item_gobj);
}

pub unsafe extern "C" fn it_802C4BB8(item_gobj: *mut Item_GObj) {
    let item = GET_ITEM(item_gobj);

    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
    let y_scale = ext::ftLib_80086A0C((*item).xDD4_itemVar.mdisable.owner);
    ext::it_802755C0(item_gobj, y_scale);
}

pub unsafe extern "C" fn itMewtwodisable_UnkMotion0_Anim(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    Item_TickLifetime(item)
}

pub unsafe extern "C" fn itMewtwodisable_UnkMotion0_Phys(_item_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMewtwodisable_UnkMotion0_Coll(item_gobj: *mut Item_GObj) -> i32 {
    let mut coll_flags: u32 = 0;

    // coll processing
    ext::it_8026D9A0(item_gobj);
    // report if collided with wall
    coll_flags |= ext::it_80276308(item_gobj) as u32;
    // combine with collided with ceiling report
    coll_flags |= ext::it_802763E0(item_gobj) as u32;
    coll_flags as i32
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_DmgDealt(_item_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_Reflected(item_gobj: *mut Item_GObj) -> bool {
    // process being reflected
    ext::it_80273030(item_gobj) != 0
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_Clanked(_item_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_HitShield(_item_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_Absorbed(_item_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_ShieldBounced(_item_gobj: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itMewtwoDisable_Logic67_EvtUnk(item_gobj: *mut Item_GObj, ref_owner: *mut Fighter_GObj) {
    let item = GET_ITEM(item_gobj);
    if (*item).xDD4_itemVar.mdisable.owner == ref_owner {
        (*item).xDD4_itemVar.mdisable.owner = null_mut();
    }
    // remove all gobj interaction references from item
    ext::it_8026B894(item_gobj, ref_owner);
}
