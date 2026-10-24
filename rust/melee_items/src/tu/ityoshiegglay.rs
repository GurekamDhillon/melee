//! Port of src/melee/it/kinds/ityoshiegglay.c: the egg Yoshi lays (logic table 114). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_ityoshiegglay.rs.
//!
//! `efAsync_Spawn` is variadic in the C; its extra argument (the scale, spawn kind 4) is the sixth `ext` parameter.
//! Fused site (checked in the IR): `xD44_lifeTimer -= xCA0 * x0` is `fmadds(-(float) xCA0, x0, timer)`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut f32);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F94A0[]`
pub static mut it_803F94A0: [ItemStateTable; 3] = [
    st_row(-1, pred_b(it_27CF_UnkMotion0_Anim), Some(it_27CF_UnkMotion0_Phys), pred_b(it_27CF_UnkMotion0_Coll)),
    st_row(-1, pred_b(it_27CF_UnkMotion1_Anim), Some(it_27CF_UnkMotion1_Phys), pred_b(it_27CF_UnkMotion1_Coll)),
    st_row(-1, pred_b(it_27CF_UnkMotion2_Anim), None, None),
];

pub unsafe extern "C" fn it_802F2F34(gobj: *mut HSD_GObj, attr: *mut itYoshiEggLay_DatAttrs) {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = (*attr).kind;
    // Item_InitSpawnPosition(&spawn, &attr->pos, true)
    spawn.prev_pos = (*attr).pos;
    spawn.prev_pos.z = 0.0;
    spawn.pos = spawn.prev_pos;
    spawn.facing_dir = (*attr).float2;
    spawn.x3C_damage = 0;
    spawn.vel = (*attr).vel;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = null_mut();
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let item = GET_ITEM(item_gobj);
        (*item).xDD4_itemVar.yoshiegglay.x0 = (*attr).float3;
        (*item).xC98 = (*attr).x24;
        ext::it_80275158(item_gobj, (*attr).lifetime);
        it_802F3020(item_gobj);
    }
}

pub unsafe extern "C" fn it_802F3020(item_gobj: *mut HSD_GObj) {
    let item = GET_ITEM(item_gobj);
    ext::it_8026B3A8(item_gobj);
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    (*item).xDCE_flag.anon_0.set_b7(0);
}

pub unsafe extern "C" fn it_27CF_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    let mut ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        ip = (*gobj).user_data as *mut Item;
        let item_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let mut sp1c: f32 = 1.0;
        ext::Item_8026AE84(ip, 280088, 127, 64);
        let item_gobj = gobj;
        ext::efAsync_Spawn(gobj, &raw mut (*((*gobj).user_data as *mut Item)).xBC0 as *mut c_void, 4, 1231, item_jobj, &mut sp1c);
        it_802F3290(item_gobj);
    }
    false
}

pub unsafe extern "C" fn it_27CF_UnkMotion1_Phys(item_gobj: *mut HSD_GObj) {
    let attr = (*GET_ITEM(item_gobj)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn it_27CF_UnkMotion1_Coll(item_gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(item_gobj, Some(it_802F317C));
    false
}

pub unsafe extern "C" fn it_802F317C(item_gobj: *mut HSD_GObj) {
    let item = GET_ITEM(item_gobj);
    itResetVelocity(item);
    ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_27CF_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    let mut ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        ip = (*gobj).user_data as *mut Item;
        let item_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        let mut sp1c: f32 = 1.0;
        ext::Item_8026AE84(ip, 280088, 127, 64);
        let item_gobj = gobj;
        ext::efAsync_Spawn(gobj, &raw mut (*((*gobj).user_data as *mut Item)).xBC0 as *mut c_void, 4, 1231, item_jobj, &mut sp1c);
        it_802F3290(item_gobj);
    }
    false
}

pub unsafe extern "C" fn it_27CF_UnkMotion0_Phys(_item_gobj: *mut HSD_GObj) {}

pub unsafe extern "C" fn it_27CF_UnkMotion0_Coll(item_gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(item_gobj, Some(it_802F3020));
    false
}

pub unsafe extern "C" fn it_802F3290(item_gobj: *mut Item_GObj) {
    let item_jobj = GET_JOBJ(item_gobj);
    let item = GET_ITEM(item_gobj);
    itResetVelocity(item);
    (*item).xD44_lifeTimer = 40.0;
    ext::HSD_JObjSetFlagsAll(item_jobj, JOBJ_HIDDEN as u32);
    ext::it_802756D0(item_gobj);
    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_27CF_UnkMotion2_Anim(item_gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    (*item).xD44_lifeTimer -= 1.0;
    if (*item).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn it_27CF_Logic114_DmgReceived(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    (*item).xD44_lifeTimer = fmadds(-((*item).xCA0 as f32), (*item).xDD4_itemVar.yoshiegglay.x0, (*item).xD44_lifeTimer);
    false
}

pub unsafe extern "C" fn it_27CF_Logic114_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
