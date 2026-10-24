//! Port of src/melee/it/kinds/itmariocape.c: Mario's cape (logic table 41; `it_803F70F8`). `xDAC_itcmd_var0` /
//! `xDB0_itcmd_var1` are the event-script "spawn effect" flags. Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itmariocape.rs.
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
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);
    fn ftLib_800865CC(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_800865CC(a0) };
    fn ftMr_SpecialS_CheckItemCapeRemove(a0: *mut HSD_GObj) -> i32;
    fn ftMr_SpecialS_Reset(a0: *mut HSD_GObj);
    fn it_8026B724(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b724(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B73C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b73c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80274574(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274574(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F70F8[]`
pub static mut it_803F70F8: [ItemStateTable; 2] = [
    st_row(0, pred_b(itMariocape_UnkMotion1_Anim), None, None),
    st_row(1, pred_b(itMariocape_UnkMotion1_Anim), None, None),
];

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

/// `Item_AttachToParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_AttachToParent(item_gobj: *mut Item_GObj, parent: *mut HSD_GObj, part: Fighter_Part) -> *mut Item_GObj {
    ext::Item_8026AB54(item_gobj, parent, part);
    ext::db_80225DD8(item_gobj, parent);
    item_gobj
}

/// 2B2560
pub unsafe extern "C" fn it_802B2560(
    parent_gobj: *mut Fighter_GObj,
    facing_dir: f32,
    pos: *mut Vec3,
    part: Fighter_Part,
    kind: ItemKind,
) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    Item_InitSpawnPosition(&mut spawn, pos, false);
    Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);

    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        return Item_AttachToParent(gobj, parent_gobj, part);
    }
    null_mut()
}

pub unsafe extern "C" fn itMarioCape_Logic41_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).owner.is_null() {
        ext::ftMr_SpecialS_Reset((*ip).owner);
    }
}

/// 2B2674
pub unsafe extern "C" fn it_802B2674(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !ip.is_null() {
        if !(*ip).owner.is_null() {
            ext::ftMr_SpecialS_Reset((*ip).owner);
        }
        ext::Item_8026A8EC(gobj);
    }
}

/// 2B26C0
pub unsafe extern "C" fn it_802B26C0(gobj: *mut Item_GObj) {
    ext::it_8026B724(gobj);
}

/// 2B26E0
pub unsafe extern "C" fn it_802B26E0(gobj: *mut Item_GObj) {
    ext::it_8026B73C(gobj);
}

pub unsafe extern "C" fn it_2725_Logic41_PickedUp(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
    if !(*ip).owner.is_null() {
        if ext::ftLib_800865CC(gobj) != 1 {
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
        } else {
            ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        }
        ext::Item_802694CC(gobj);
        ext::it_80274574(gobj);
    }
}

/// `inlineA0` (static)
pub unsafe extern "C" fn inlineA0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDAC_itcmd_var0 != 0 {
        (*ip).xDAC_itcmd_var0 = 0;
        ext::efAsync_Spawn(
            gobj,
            &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void,
            0,
            1149,
            (*(*ip).xBBC_dynamicBoneTable).bones[16],
        );
    }
    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).xDB0_itcmd_var1 = 0;
        ext::efAsync_Spawn(
            gobj,
            &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void,
            0,
            1150,
            (*(*ip).xBBC_dynamicBoneTable).bones[6],
        );
    }
}

/// `reset` (static inline)
#[inline]
unsafe fn reset(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).owner.is_null() {
        ext::ftMr_SpecialS_Reset((*ip).owner);
    }
}

pub unsafe extern "C" fn itMariocape_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    inlineA0(gobj);
    let ip = GET_ITEM(gobj);
    let removed = if !(*ip).owner.is_null() {
        ext::ftMr_SpecialS_CheckItemCapeRemove((*ip).owner) != 0
    } else {
        true
    };
    if removed {
        reset(gobj);
        return true;
    }
    false
}

/// 2B2870
pub unsafe extern "C" fn it_802B2870(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
