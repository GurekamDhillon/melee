//! Port of src/melee/it/kinds/itnesspkthundertrail.c: the PK Thunder trail segments. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnesspkthundertrail.rs.
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_802AB3F0(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32) => unsafe { crate::tu::itnesspkthunderball::it_802AB3F0(a0 as _, a1 as _, a2) };
    fn it_802AB468(a0: *mut HSD_GObj, a1: *mut f32, a2: i32) => unsafe { crate::tu::itnesspkthunderball::it_802AB468(a0 as _, a1, a2) };
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

const M_PI: f64 = 3.14159265358979323846;

/// `ItemStateTable it_803F6C08[]`
pub static mut it_803F6C08: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itNesspkthundertrail_UnkMotion0_Anim),
    Some(itNesspkthundertrail_UnkMotion0_Phys),
    pred_b(itNesspkthundertrail_UnkMotion0_Coll),
)];

/// `s32 it_803F6C18[]`
pub static it_803F6C18: [i32; 6] = [
    It_Kind_Ness_PKThunder1 as i32,
    It_Kind_Ness_PKThunder1 as i32,
    It_Kind_Ness_PKThunder1 as i32,
    It_Kind_Ness_PKThunder2 as i32,
    It_Kind_Ness_PKThunder3 as i32,
    It_Kind_Ness_PKThunder4 as i32,
];

pub unsafe extern "C" fn it_802AC43C(gobj: *mut Fighter_GObj, ball: *mut Item_GObj, pos: *mut Vec3, trail_idx: i32, arg4: i32, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802AC43C");
    let mut spawn: SpawnItem = core::mem::zeroed();

    if trail_idx < 0 || trail_idx > 5 {
        return null_mut();
    }

    spawn.kind = it_803F6C18[trail_idx as usize] as _;
    Item_InitSpawnPosition(&mut spawn, pos, true);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = gobj;
    spawn.x4_parent_gobj2 = ball;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = arg4;

    let trail = ext::Item_80268B18(&mut spawn);
    if !trail.is_null() {
        let ip = GET_ITEM(trail);
        Item_ClearCmdVars(ip);
        (*ip).xDD4_itemVar.nesspkthundertrail.x0 = ball;
        (*ip).xDD4_itemVar.nesspkthundertrail.x4 = trail_idx;
        (*ip).xDD4_itemVar.nesspkthundertrail.x8 = 0;
        ext::it_8026B3A8(trail);
        (*ip).xDC8_word.flags.set_x13(0);
        ext::it_80272940(trail);
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.x = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).xDCC_flag.set_b3(0);
        if !gobj.is_null() {
            ext::db_80225DD8(trail, gobj);
        }
        it_802AC604(trail);
    }
    trail
}

pub unsafe extern "C" fn it_802AC58C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AC58C");
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        ext::it_802725D4(gobj);
        (*ip).xDD4_itemVar.nesspkthundertrail.x0 = null_mut();
        (*ip).owner = null_mut();
        (*ip).xDC8_word.flags.set_x13(0);
    }
}

pub unsafe extern "C" fn it_802AC5D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AC5D8");
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            (*ip).owner = null_mut();
            (*ip).xDC8_word.flags.set_x13(0);
        }
    }
}

pub unsafe extern "C" fn it_802AC604(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802AC604");
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itNesspkthundertrail_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itNesspkthundertrail_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);

    if (*ip).xDD4_itemVar.nesspkthundertrail.x8 & 1 != 0 {
        ext::HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    }
    (*ip).xDD4_itemVar.nesspkthundertrail.x8 += 1;

    if (*ip).xDD4_itemVar.nesspkthundertrail.x0.is_null() {
        return true;
    }
    false
}

pub unsafe extern "C" fn itNesspkthundertrail_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itNesspkthundertrail_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    if !(*ip).xDD4_itemVar.nesspkthundertrail.x0.is_null() {
        let mut radius: f32 = 0.0;
        let mut angle: f32;
        let mut next_pos: Vec3 = core::mem::zeroed();

        ext::it_802AB3F0((*ip).xDD4_itemVar.nesspkthundertrail.x0, &mut (*ip).pos, (*ip).xDD4_itemVar.nesspkthundertrail.x4);
        ext::it_802AB468((*ip).xDD4_itemVar.nesspkthundertrail.x0, &mut radius, (*ip).xDD4_itemVar.nesspkthundertrail.x4);
        HSD_JObjSetScaleZ(jobj, 2.0 * radius / 4.0);

        ext::it_802AB3F0((*ip).xDD4_itemVar.nesspkthundertrail.x0, &mut next_pos, (*ip).xDD4_itemVar.nesspkthundertrail.x4 + 1);

        if (*ip).facing_dir == 1.0 {
            angle = ext::atan2f((*ip).pos.y - next_pos.y, (*ip).pos.x - next_pos.x);
        } else {
            angle = (M_PI + ext::atan2f((*ip).pos.y - next_pos.y, (*ip).pos.x - next_pos.x) as f64) as f32;
        }

        HSD_JObjSetRotationX(jobj, angle * -(*ip).facing_dir);
    }
}

pub unsafe extern "C" fn itNesspkthundertrail_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itNesspkthundertrail_UnkMotion0_Coll");
    false
}
