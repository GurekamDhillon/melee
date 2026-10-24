//! Port of src/melee/it/kinds/itnessbat.c: Ness's bat (logic table 58). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnessbat.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftCo_800BF228(a0: *mut HSD_GObj) -> i32;
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftNs_AttackS4_CheckNessBatRemove(a0: *mut HSD_GObj) -> i32;
    fn ftNs_AttackS4_ItemNessBatSetNULL(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `Item_CopyJObjScale` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_CopyJObjScale(dst: *mut HSD_JObj, src: *mut HSD_JObj, scale: *mut Vec3) {
    let s = HSD_JObjGetScaleY(src);
    (*scale).z = s;
    (*scale).y = s;
    (*scale).x = s;
    HSD_JObjSetScale(dst, scale);
}

/// `ItemStateTable it_803F6C68[]`
pub static mut it_803F6C68: [ItemStateTable; 2] = [
    st_row(0, pred_b(itNessbat_UnkMotion0_Anim), Some(itNessbat_UnkMotion0_Phys), pred_b(itNessbat_UnkMotion0_Coll)),
    st_row(-1, pred_b(itNessbat_UnkMotion1_Anim), Some(itNessbat_UnkMotion1_Phys), pred_b(itNessbat_UnkMotion1_Coll)),
];

/// `itNessbat_ClearOwnerRef` (static inline)
unsafe fn itNessbat_ClearOwnerRef(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !gobj.is_null() && !ip.is_null() {
        let x0 = (*ip).xDD4_itemVar.nessbat.x0;
        if !x0.is_null() && (*ip).owner == x0 {
            ext::ftNs_AttackS4_ItemNessBatSetNULL(x0);
        }
        (*ip).xDD4_itemVar.nessbat.x0 = null_mut();
        (*ip).owner = null_mut();
    }
}

/// `itNessbat_RemoveItem` (static inline)
unsafe fn itNessbat_RemoveItem(gobj: *mut Item_GObj) -> bool {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).xDD4_itemVar.nessbat.x0 = null_mut();
        (*ip).owner = null_mut();
        return true;
    }
    false
}

/// 2AD478: spawns the bat on a fighter's bone.
pub unsafe extern "C" fn it_802AD478(gobj: *mut Item_GObj, pos: *mut Vec3, part: Fighter_Part, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let bat: *mut Item_GObj;

    if !gobj.is_null() {
        spawn.kind = It_Kind_Ness_Bat;
        Item_InitSpawnPosition(&mut spawn, pos, true);
        Item_InitSpawnCommonFields(&mut spawn, gobj, facing_dir, false);

        bat = ext::Item_80268B18(&mut spawn);
        if !bat.is_null() {
            let ip = GET_ITEM(bat);
            Item_ClearCmdVars(ip);
            (*ip).xDCC_flag.set_b3(0);
            (*ip).xDD4_itemVar.nessbat.x0 = gobj;
            ext::it_80275158(bat, 1200.0);
            ext::Item_8026AB54(bat, gobj, part as _);
        }
    } else {
        bat = null_mut();
    }
    bat
}

/// 2AD590: spawns the bat at the fighter's hand position.
pub unsafe extern "C" fn it_802AD590(gobj: *mut Fighter_GObj, part: Fighter_Part) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let bat: *mut Item_GObj;
    let mut pos: Vec3 = core::mem::zeroed();

    if !gobj.is_null() {
        spawn.kind = It_Kind_Ness_Bat;
        ext::ftLib_80086644(gobj, &mut pos);
        Item_InitSpawnPosition(&mut spawn, &mut pos, true);
        Item_InitSpawnCommonFields(&mut spawn, gobj, ext::ftLib_800865C0(gobj), false);

        bat = ext::Item_80268B18(&mut spawn);
        if !bat.is_null() {
            let ip = GET_ITEM(bat);
            (*ip).xDB8_itcmd_var3 = 1;
            (*ip).xDB4_itcmd_var2 = 1;
            (*ip).xDB0_itcmd_var1 = 1;
            (*ip).xDAC_itcmd_var0 = 1;
            (*ip).xDCC_flag.set_b3(0);
            (*ip).xDD4_itemVar.nessbat.x0 = gobj;
            ext::Item_8026AB54(bat, gobj, part as _);
        }
    } else {
        bat = null_mut();
    }
    bat
}

pub unsafe extern "C" fn it_802AD6B8(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        (*ip).xDD4_itemVar.nessbat.x0 = null_mut();
        (*ip).owner = null_mut();
        ext::Item_8026A8EC(gobj);
    }
}

pub unsafe extern "C" fn it_2725_Logic58_PickedUp(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if ((*ip).xDAC_itcmd_var0 | (*ip).xDB0_itcmd_var1 | (*ip).xDB4_itcmd_var2 | (*ip).xDB8_itcmd_var3) != 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    } else {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    }
    ext::Item_802694CC(gobj);
}

pub unsafe extern "C" fn itNessbat_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);

    if (*ip).xDAC_itcmd_var0 == 0 {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    }

    if !(*ip).xDD4_itemVar.nessbat.x0.is_null() {
        if ext::ftNs_AttackS4_CheckNessBatRemove((*ip).xDD4_itemVar.nessbat.x0) == 1 {
            itNessbat_ClearOwnerRef(gobj);
            if itNessbat_RemoveItem(gobj) {
                return true;
            }
        }
    } else {
        itNessbat_ClearOwnerRef(gobj);
        if itNessbat_RemoveItem(gobj) {
            return true;
        }
    }
    false
}

pub unsafe extern "C" fn itNessbat_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itNessbat_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNessbat_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            if !(*ip).xDD4_itemVar.nessbat.x0.is_null() {
                if ext::ftCo_800BF228((*ip).xDD4_itemVar.nessbat.x0) == 1 {
                    let mut scale: Vec3 = core::mem::zeroed();
                    let owner = (*ip).xDD4_itemVar.nessbat.x0;
                    Item_CopyJObjScale(GET_JOBJ(gobj), GET_JOBJ(owner), &mut scale);
                }
            }
        }
    }
    false
}

pub unsafe extern "C" fn itNessbat_UnkMotion1_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itNessbat_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNessBat_Logic58_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
