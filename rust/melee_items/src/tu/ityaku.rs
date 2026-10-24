//! Port of src/melee/it/kinds/ityaku.c: the stage-driven "yakumono" item (logic table 117), spawned by a ground
//! with its own callbacks. Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_ityaku.rs.
//!
//! `it_2E6A_UnkMotion19_Phys` ends in `OSReport(...); while (true);` when the item is neither bone-following (1)
//! nor fixed (2): the port panics there (the C hangs).
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027B798(a0: *mut HSD_GObj, a1: *mut Vec) -> i32 => unsafe { crate::tu::itzako::it_8027B798(a0, a1 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D38: *mut *mut Article = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `static ItemAttr it_803F8C08` (the Random Pokemon slot's common attributes, installed on spawn);
/// the first two bytes are the packed bitfields (`0x3108`: x0_78 = 6, hold_kind = 1, x1_5 = 1).
pub static mut it_803F8C08: ItemAttr = ItemAttr {
    bf_0: [0x31, 0x08],
    x3: 0,
    x4_throw_speed_mul: 0.0,
    x8: 0,
    xC_spin_speed: 0.0,
    x10_fall_speed: 0.0,
    x14_fall_speed_max: 0.0,
    x18: 0.0,
    x1C_damage_mul: 0.0,
    x20: _itECB { top: 0.0, bottom: 0.0, right: 0.0, left: 0.0 },
    x30_unk: Vec2 { x: 0.0, y: 0.0 },
    x38_grab_range: Vec2 { x: 0.0, y: 0.0 },
    x40: _itECB { top: 2.0, bottom: 2.0, right: 2.0, left: 2.0 },
    x50: 0.0,
    x54: 0.0,
    x58: 0.0,
    x5c: 0.0,
    x60_scale: 1.0,
    destroy_gfx: -1,
    x68: -1,
    x6C: 0x00083D60,
    x70: 0x00083D60,
    x74: 0x00083D60,
    destroy_sfx: 0x00083D60,
    x7C: 0x00083D60,
    x80: 0x00083D60,
};

/// `static const Vec3 it_803B8730`
static it_803B8730: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };

/// `ItemStateTable it_803F8C8C[]`
pub static mut it_803F8C8C: [ItemStateTable; 20] = {
    const fn row(i: i32) -> ItemStateTable {
        st_row(i, None, Some(it_2E6A_UnkMotion19_Phys), None)
    }
    [
        row(0), row(1), row(2), row(3), row(4), row(5), row(6), row(7), row(8), row(9),
        row(10), row(11), row(12), row(13), row(14), row(15), row(16), row(17), row(18), row(19),
    ]
};

pub unsafe extern "C" fn it_802E6AEC(
    arg0: *mut Ground,
    arg1: i32,
    arg2: i32,
    arg3: *mut HSD_JObj,
    pos: *mut Vec3,
    arg5: i32,
    arg6: Option<unsafe extern "C" fn(*mut Item_GObj, *mut Ground)>,
    arg7: Option<unsafe extern "C" fn(*mut Item_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>,
    arg8: Option<unsafe extern "C" fn(*mut Item_GObj, *mut Ground, *mut HSD_GObj)>,
) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_PKind_Random;
    if !arg3.is_null() {
        ext::lb_8000B1CC(arg3, null_mut(), &mut spawn.prev_pos);
    } else if !pos.is_null() {
        spawn.prev_pos = *pos;
    } else {
        return null_mut();
    }
    spawn.pos = spawn.prev_pos;
    spawn.facing_dir = 0.0;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(if arg5 != 0 { 1 } else { 0 });
    spawn.x40 = 0;
    (**ext::it_804D6D38.offset((spawn.kind - It_Kind_Kuriboh) as isize)).x0_common_attr = &raw mut it_803F8C08;

    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let item = GET_ITEM(item_gobj);
        if !arg3.is_null() {
            (*item).xDD4_itemVar.yaku.x2 = 1;
            (*item).xDD4_itemVar.yaku.anon_4.x4 = arg3;
        } else if !pos.is_null() {
            (*item).xDD4_itemVar.yaku.x2 = 2;
            (*item).xDD4_itemVar.yaku.anon_4.x4_vec = *pos;
        }

        (*item).xDD4_itemVar.yaku.x0 = arg1 as i16;
        (*item).xDD4_itemVar.yaku.x10 = arg0;
        (*item).xDD4_itemVar.yaku.x14 = arg6;
        (*item).xDD4_itemVar.yaku.x18 = arg7;
        (*item).xDD4_itemVar.yaku.x1C = arg8;
        let item_jobj = GET_JOBJ(item_gobj);
        let mut sp30 = it_803B8730;
        HSD_JObjSetScale(item_jobj, &mut sp30);
        (*item).xDCC_flag.set_b3(0);
        ext::it_802756D0(item_gobj);
        ext::Item_80268E5C(item_gobj, arg2, ITEM_ANIM_UPDATE);
        (*item).touched = pred_b(it_802E7054);
        (*item).x378_itemColl.x34_flags.set_b1234(5);
    }
    item_gobj
}

pub unsafe extern "C" fn it_2E6A_UnkMotion19_Phys(item_gobj: *mut HSD_GObj) {
    let mut sp24: Vec3 = core::mem::zeroed();
    let mut sp14: Quaternion = core::mem::zeroed();

    let item = GET_ITEM(item_gobj);
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    if (*item).xDD4_itemVar.yaku.x2 == 1 {
        ext::lb_8000B1CC((*item).xDD4_itemVar.yaku.anon_4.x4, null_mut(), &mut sp24);
        (*item).pos = sp24;
        let jobj1 = (*item).xDD4_itemVar.yaku.anon_4.x4;
        HSD_JObjGetScale(jobj1, &mut sp24);
        HSD_JObjSetScale(item_jobj, &mut sp24);
        let jobj2 = (*item).xDD4_itemVar.yaku.anon_4.x4;
        HSD_JObjGetRotation(jobj2, &mut sp14);
        HSD_JObjSetRotation(item_jobj, &mut sp14);
    } else if (*item).xDD4_itemVar.yaku.x2 != 2 {
        // OSReport("%s:%d: oioi...\n", __FILE__, 215); while (true);
        panic!("ityaku.c:215: oioi...");
    }
}

pub unsafe extern "C" fn it_2E6A_Logic117_DmgDealt(item_gobj: *mut Item_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    if let (Some(f), false) = ((*item).xDD4_itemVar.yaku.x14, (*item).xDD4_itemVar.yaku.x10.is_null()) {
        f(item_gobj, (*item).xDD4_itemVar.yaku.x10);
    }
    false
}

pub unsafe extern "C" fn it_2E6A_Logic117_DmgReceived(item_gobj: *mut Item_GObj) -> bool {
    let mut sp10: Vec3 = core::mem::zeroed();
    let item = GET_ITEM(item_gobj);

    (*item).xC9C = 0;
    ext::it_8027B798(item_gobj, &mut sp10);
    let fighter_gobj = (*item).xCEC_fighterGObj;
    if let (Some(f), false) = ((*item).xDD4_itemVar.yaku.x18, (*item).xDD4_itemVar.yaku.x10.is_null()) {
        f(item_gobj, (*item).xDD4_itemVar.yaku.x10, &mut sp10, fighter_gobj, (*item).xCA0 as f32);
    }
    false
}

pub unsafe extern "C" fn it_802E7054(item_gobj: *mut HSD_GObj) -> bool {
    let item = GET_ITEM(item_gobj);
    if let (Some(f), false) = ((*item).xDD4_itemVar.yaku.x1C, (*item).xDD4_itemVar.yaku.x10.is_null()) {
        f(item_gobj, (*item).xDD4_itemVar.yaku.x10, (*item).toucher);
    }
    false
}

pub unsafe extern "C" fn it_2E6A_Logic117_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(item_gobj, ref_gobj);
}
