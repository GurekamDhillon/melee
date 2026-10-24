//! Port of src/melee/it/kinds/itseakneedleheld.c: Sheik's held needles (logic table 110). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itseakneedleheld.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftKb_SpecialNSk_80105FF0(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNSk_80106020(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800869D4(a0) };
    fn ftSk_SpecialS_80111F70(a0: *mut HSD_GObj) -> i32;
    fn ftSk_SpecialS_80111FA0(a0: *mut HSD_GObj) -> i32;
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

crate::kinds_jobj_inl!();

/// `Item_ClearCmdVars` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `HSD_JObjGetNext` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).next }
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



/// `ItemStateTable it_803F70A8[]`
pub static mut it_803F70A8: [ItemStateTable; 1] = [
    st_row(-1, pred_b(itSeakneedleheld_UnkMotion0_Anim), Some(itSeakneedleheld_UnkMotion0_Phys), pred_b(itSeakneedleheld_UnkMotion0_Coll)),
];

pub unsafe extern "C" fn it_802B18B0(gobj: *mut Item_GObj) {
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild(jobj);
    let mut needle = HSD_JObjGetChild(child);
    let owner = (*ip).xDD4_itemVar.seakneedleheld.owner;
    // uninitialised on the default path (the shipping build's auto-var-init zeroes it)
    let mut count: i32 = 0;

    if owner.is_null() {
        return;
    }
    if (*ip).owner != owner {
        return;
    }

    match (*ip).kind {
        It_Kind_Seak_NeedleHeld => count = ext::ftSk_SpecialS_80111FA0(owner),
        It_Kind_Kirby_SeakNeedleHeld => count = ext::ftKb_SpecialNSk_80106020(owner),
        _ => {}
    }

    let mut i = 0;
    while i < 6 {
        if i < count {
            ext::HSD_JObjClearFlagsAll(needle, JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjSetFlagsAll(needle, JOBJ_HIDDEN as u32);
        }
        needle = HSD_JObjGetNext(needle);
        i += 1;
    }
}

pub unsafe extern "C" fn it_802B19AC(
    parent_gobj: *mut Fighter_GObj,
    cur_pos: *mut Vec3,
    part: u32,
    kind: ItemKind,
    facing_dir: f32,
) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPosition(&mut spawn, cur_pos, true);
    Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);

    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        Item_ClearCmdVars(ip);
        (*ip).xDCC_flag.set_b3(0);
        (*ip).xDD4_itemVar.seakneedleheld.owner = parent_gobj;
        ext::Item_8026AB54(gobj, parent_gobj, part);
    }
    gobj
}

pub unsafe extern "C" fn itSeakNeedleHeld_Logic110_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSeakneedleheld_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let needle = HSD_JObjGetChild(jobj);
    let mut scale: Vec3 = core::mem::zeroed();

    if (*ip).xDD4_itemVar.seakneedleheld.owner.is_null() {
        return true;
    }
    if (*ip).owner != (*ip).xDD4_itemVar.seakneedleheld.owner {
        return false;
    }

    match (*ip).kind {
        It_Kind_Seak_NeedleHeld => {
            if ext::ftSk_SpecialS_80111F70((*ip).xDD4_itemVar.seakneedleheld.owner) == 1 {
                return true;
            }
        }
        It_Kind_Kirby_SeakNeedleHeld => {
            if ext::ftKb_SpecialNSk_80105FF0((*ip).xDD4_itemVar.seakneedleheld.owner) == 1 {
                return true;
            }
        }
        _ => {}
    }

    it_802B18B0(gobj);

    let s = ext::ftLib_800869D4((*ip).xDD4_itemVar.seakneedleheld.owner);
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(needle, &mut scale);
    false
}

pub unsafe extern "C" fn itSeakneedleheld_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itSeakneedleheld_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itSeakNeedleHeld_Logic110_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
