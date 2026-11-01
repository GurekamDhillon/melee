//! Native port of src/melee/it/kinds/itgamewatchpanic.c (Game & Watch's Oil Panic item). Callees in other
//! TUs go through `ext` (TODO hooks). `bool` is the C `int`: callee results are tested as ints and
//! `true` is 1. Checked against the whole C TU (shipping contraction) by tests/tu_itgamewatchpanic.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
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
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftGw_SpecialLw_ItemCheckPanicRemove(a0: *mut HSD_GObj) -> i32;
    fn ftGw_SpecialLw_ItemPanicSetFlag(a0: *mut HSD_GObj);
    fn ftLib_800865CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_800865CC(a0) };
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

/// `Item_AttachToParent` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_AttachToParent(item_gobj: *mut HSD_GObj, parent: *mut HSD_GObj, part: Fighter_Part) -> *mut HSD_GObj {
    ext::Item_8026AB54(item_gobj, parent, part);
    ext::db_80225DD8(item_gobj, parent);
    item_gobj
}

/// `Item_AttachGameWatchArticle` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_AttachGameWatchArticle(parent: *mut HSD_GObj, part: Fighter_Part, item_gobj: *mut HSD_GObj, attributes: *mut *mut c_void) {
    Item_AttachToParent(item_gobj, parent, part);
    ext::it_8027CE64(item_gobj, parent, *attributes);
}

/// `ItemStateTable it_803F79A0[]`
pub static mut it_803F79A0: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchpanic_UnkMotion1_Anim), physics_updated: None, collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itGamewatchpanic_UnkMotion1_Anim), physics_updated: None, collided: None },
];

/// `HSD_GObj* it_802C7D60(Item_GObj* parent, Vec3* pos, Fighter_Part arg2, f32 dir)`
pub unsafe extern "C" fn it_802C7D60(parent: *mut HSD_GObj, pos: *mut Vec, arg2: Fighter_Part, dir: f32) -> *mut HSD_GObj {
    melee_core::ct!("it_802C7D60");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_GameWatch_Panic;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, parent, dir, true);
    let result = ext::Item_80268B18(&mut spawn);
    if !result.is_null() {
        let item = GET_ITEM(result);
        let attr = special_attributes(item);
        Item_AttachGameWatchArticle(parent, arg2, result, attr);
        return result;
    }
    null_mut()
}

pub unsafe extern "C" fn itGameWatchPanic_Logic78_Destroyed(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGameWatchPanic_Logic78_Destroyed");
    let ip = GET_ITEM(gobj);
    if !(*ip).owner.is_null() {
        ext::ftGw_SpecialLw_ItemPanicSetFlag((*ip).owner);
    }
}

pub unsafe extern "C" fn it_802C7E94(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C7E94");
    let ip = GET_ITEM(gobj);
    if !ip.is_null() {
        itGameWatchPanic_Logic78_Destroyed(gobj);
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn it_802C7EE0(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C7EE0");
    ext::it_8026B724(gobj);
}

pub unsafe extern "C" fn it_802C7F00(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802C7F00");
    ext::it_8026B73C(gobj);
}

pub unsafe extern "C" fn itGameWatchPanic_Logic78_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGameWatchPanic_Logic78_PickedUp");
    let ip = GET_ITEM(gobj);
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
    if !(*ip).owner.is_null() {
        (*ip).ground_or_air = ext::ftLib_800865CC((*ip).owner) as GroundOrAir;
        if (*ip).ground_or_air == GA_Ground {
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
        } else {
            ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        }
        ext::Item_802694CC(gobj);
    }
}

pub unsafe extern "C" fn itGamewatchpanic_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGamewatchpanic_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let cond = if !(*ip).owner.is_null() { ext::ftGw_SpecialLw_ItemCheckPanicRemove((*ip).owner) } else { 1 };
    if cond != 0 {
        let ip = GET_ITEM(gobj);
        if !(*ip).owner.is_null() {
            ext::ftGw_SpecialLw_ItemPanicSetFlag((*ip).owner);
        }
        return 1;
    }
    0
}

pub unsafe extern "C" fn itGameWatchPanic_Logic78_EvtUnk(arg0: *mut HSD_GObj, arg1: *mut HSD_GObj) {
    melee_core::ct!("itGameWatchPanic_Logic78_EvtUnk");
    ext::it_8026B894(arg0, arg1);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802C7EE0` (src/melee/it/kinds/itgamewatchpanic.c): `it_8026B724(..)`.
pub fn it_802c7ee0(ip: &mut Item) {
    melee_core::ct!("it_802c7ee0");
    it_8026b724(ip);
}

/// `it_802C7F00` (src/melee/it/kinds/itgamewatchpanic.c): `it_8026B73C(..)`.
pub fn it_802c7f00(ip: &mut Item) {
    melee_core::ct!("it_802c7f00");
    it_8026b73c(ip);
}
