//! Native port of src/melee/it/kinds/itclinkmilk.c (Link's milk bottle, the Clink milk item): the
//! spawn helper, the attach/detach logic that follows the owning Ice-less climber reference, the
//! motion 1 callbacks (`it_803F7A28`) and the Logic80 hook. Callees in other TUs go through `ext`;
//! callees ported in this crate are called directly. `*_p` wrappers return `i32` for the state
//! table (`HSD_GObjPredicate`). Checked by tests/tu_itclinkmilk.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;


/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftCl_Init_8014920C(a0: *mut HSD_GObj) -> i32;
    fn ftCl_Init_801492C4(a0: *mut HSD_GObj);
    fn ftCl_Init_801492F4(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800869D4(a0: *mut HSD_GObj) -> f32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

jobj_inl!();
spawn_inl!();

/// `JOBJ_HIDDEN` (sysdolphin/baselib/jobj.h).
pub const JOBJ_HIDDEN: u32 = 1 << 4;

/// `predicate` form of a bool callback for the `int`-typed state table.
macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_ClearCmdVars` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `it_802C8B28` (itclinkmilk.c): spawn the milk bottle for `parent`, bound to `bone`.
pub unsafe extern "C" fn it_802C8B28(parent: *mut HSD_GObj, pos: *mut Vec3, bone: u32, facing_dir: f32) -> *mut HSD_GObj {
    if parent.is_null() {
        return null_mut();
    }
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_CLink_Milk;
    Item_InitSpawnPosition(&mut spawn, pos, true);
    Item_InitSpawnCommonFields(&mut spawn, parent, facing_dir, false);
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = item_of(item_gobj);
        Item_ClearCmdVars(ip);
        (*ip).xDCC_flag.set_b3(0);
        (*ip).xDD4_itemVar.clinkmilk.x0 = parent as *mut Item_GObj;
        ext::Item_8026AB54(item_gobj, parent, bone);
    }
    item_gobj
}

/// `it_802C8C34` (itclinkmilk.c): detach from the owner and destroy.
pub unsafe extern "C" fn it_802C8C34(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        let ip = item_of(gobj);
        if !ip.is_null() {
            (*ip).xDD4_itemVar.clinkmilk.x0 = null_mut();
            (*ip).owner = null_mut();
            ext::Item_8026A8EC(gobj);
        }
    }
}

/// `it_2725_Logic80_PickedUp`: pick the anim side from the facing direction.
pub unsafe extern "C" fn it_2725_Logic80_PickedUp(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).facing_dir == 1.0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    }
    ext::Item_802694CC(gobj);
}

/// `itCLinkMilk_NotifyParent` (static inline): release the owner's milk reference.
pub unsafe fn itCLinkMilk_NotifyParent(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        let ip = item_of(gobj);
        if !ip.is_null() {
            let parent = (*ip).xDD4_itemVar.clinkmilk.x0 as *mut HSD_GObj;
            if !parent.is_null() && (*ip).owner == parent {
                ext::ftCl_Init_801492C4(parent);
            }
            (*ip).xDD4_itemVar.clinkmilk.x0 = null_mut();
            (*ip).owner = null_mut();
        }
    }
}

/// `itCLinkMilk_Destroy` (static inline): clear the references; true if there was an item.
pub unsafe fn itCLinkMilk_Destroy(gobj: *mut HSD_GObj) -> bool {
    if !gobj.is_null() {
        let ip = item_of(gobj);
        if !ip.is_null() {
            (*ip).xDD4_itemVar.clinkmilk.x0 = null_mut();
            (*ip).owner = null_mut();
            return true;
        }
    }
    false
}

/// `itClinkmilk_UnkMotion1_Anim`: show the bottle while the flag is set; follow the owner's
/// state; destroy when the owner is gone.
pub unsafe extern "C" fn itClinkmilk_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let child = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);
    let grandchild = HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj);

    if (*ip).xDAC_itcmd_var0 == 1 {
        ext::HSD_JObjClearFlags(grandchild, JOBJ_HIDDEN);
    } else {
        ext::HSD_JObjSetFlags(grandchild, JOBJ_HIDDEN);
    }

    let parent = (*ip).xDD4_itemVar.clinkmilk.x0 as *mut HSD_GObj;
    if !parent.is_null() {
        if ext::ftCl_Init_8014920C(parent) == 1 {
            itCLinkMilk_NotifyParent(gobj);
            if itCLinkMilk_Destroy(gobj) {
                return true;
            }
        } else {
            (*ip).xDAC_itcmd_var0 = ext::ftCl_Init_801492F4(parent) as u32;
            let s = ext::ftLib_800869D4(parent);
            let sv = Vec3 { x: s, y: s, z: s };
            let mut sv = sv;
            HSD_JObjSetScale(child, &mut sv);
        }
    } else {
        itCLinkMilk_NotifyParent(gobj);
        if itCLinkMilk_Destroy(gobj) {
            return true;
        }
    }
    false
}

/// `itClinkmilk_UnkMotion1_Phys` (empty).
pub unsafe extern "C" fn itClinkmilk_UnkMotion1_Phys(_gobj: *mut HSD_GObj) {}

/// `itClinkmilk_UnkMotion1_Coll` (returns false).
pub unsafe extern "C" fn itClinkmilk_UnkMotion1_Coll(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itCLinkMilk_Logic80_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itCLinkMilk_Logic80_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pred_i32! {
    itClinkmilk_UnkMotion1_Anim_p => itClinkmilk_UnkMotion1_Anim;
    itClinkmilk_UnkMotion1_Coll_p => itClinkmilk_UnkMotion1_Coll;
}

/// `it_803F7A28[]` (itclinkmilk.c).
pub static it_803F7A28: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itClinkmilk_UnkMotion1_Anim_p), physics_updated: Some(itClinkmilk_UnkMotion1_Phys), collided: Some(itClinkmilk_UnkMotion1_Coll_p) },
    ItemStateTable { anim_id: 1, animated: Some(itClinkmilk_UnkMotion1_Anim_p), physics_updated: Some(itClinkmilk_UnkMotion1_Phys), collided: Some(itClinkmilk_UnkMotion1_Coll_p) },
];
