//! Port of src/melee/it/kinds/itarwinglaser.c: the Arwing's laser (Corneria / Venom). Types 0 and 2 are
//! the main beam (type 0 also spawns two accessory beams, 4 and 5, that follow it through the parent's
//! bone 0), 1 and 3 are the aimed shots (`it_802E7654`). Callees of unported TUs go through `ext`
//! (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itarwinglaser.rs.
//!
//! `it_802E7654` has no `return` in the C ("NOTE: no return"); the port returns the spawned gobj,
//! which is what the PPC code leaves in r3 for its callers, and the oracle does not compare it.
//! The state-table predicates are stored through the same `(HSD_GObjPredicate) (Event)` cast the C uses.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Ground_801C0498() -> f32 => unsafe { melee_world::tu::ground::Ground_801C0498() };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_80225194() -> u32 => unsafe { melee_world::tu::stage::Stage_80225194() };
    fn Stage_8022519C(a0: u32) -> u32 => unsafe { melee_world::tu::stage::Stage_8022519C(a0) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32 => unsafe { crate::tu::ftlib::ftLib_800865C0(a0) };
    fn grCorneria_801DDCF0(a0: *mut Vec) => unsafe { melee_world::tu::grcorneria::grCorneria_801DDCF0(a0 as _) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026EA20(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut i32, a5: *mut u32) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026EA20(a0, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn it_8026EA9C(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut i32, a5: *mut u32, a6: i32) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026EA9C(a0, a1 as _, a2 as _, a3 as _, a4, a5, a6) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_802750F8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802750F8(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    fn mpGetSpeed(a0: i32, a1: *mut Vec, a2: *mut Vec) -> i32 => unsafe { melee_world::tu::mplib::mpGetSpeed(a0, a1, a2) };
    fn mpLineGetFlags(a0: i32) -> u32 => unsafe { melee_world::tu::mplib::mpLineGetFlags(a0) };
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

/// `struct ArwingLaserAttr` (the `.c` file's own record of `Article.x4_specialAttributes`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArwingLaserAttr {
    pub x0: *mut ItemAttr,
    pub x4: f32,
    pub x8: f32,
}
melee_core::gc_layout!(ArwingLaserAttr, "ArwingLaserAttr", size 0xC, align 4, [
    x0 @ 0x0: 4 "ItemAttr *",
    x4 @ 0x4: 4 "f32",
    x8 @ 0x8: 4 "f32",
]);

/// `static Vec2 it_803F8E48[]`
pub static it_803F8E48: [Vec2; 4] = [
    Vec2 { x: 0.0, y: 200.0 },
    Vec2 { x: 0.0, y: -200.0 },
    Vec2 { x: -200.0, y: 0.0 },
    Vec2 { x: 200.0, y: 0.0 },
];

/// `ItemStateTable it_803F8DE8[]`
pub static mut it_803F8DE8: [ItemStateTable; 6] = [
    st_row(0, pred_v(itArwinglaser_UnkMotion2_Anim), Some(itArwinglaser_UnkMotion2_Phys), pred_b!(itArwinglaser_UnkMotion2_Coll)),
    st_row(1, pred_v(itArwinglaser_UnkMotion3_Anim), Some(itArwinglaser_UnkMotion3_Phys), pred_b!(itArwinglaser_UnkMotion3_Coll)),
    st_row(2, pred_v(itArwinglaser_UnkMotion2_Anim), Some(itArwinglaser_UnkMotion2_Phys), pred_b!(itArwinglaser_UnkMotion2_Coll)),
    st_row(3, pred_v(itArwinglaser_UnkMotion3_Anim), Some(itArwinglaser_UnkMotion3_Phys), pred_b!(itArwinglaser_UnkMotion3_Coll)),
    st_row(4, None, None, pred_b!(itArwinglaser_UnkMotion5_Coll)),
    st_row(5, None, None, pred_b!(itArwinglaser_UnkMotion5_Coll)),
];

/// `itArwinglaser_GetCollArg` (static inline)
#[inline]
unsafe fn itArwinglaser_GetCollArg(ip: *mut Item) -> i32 {
    (*ip).xDD4_itemVar.arwinglaser.xE48
}

/// 2E70BC: the beam's tip segment against the lines, then four 200-unit probes around it.
pub unsafe extern "C" fn it_802E70BC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802E70BC");
    let mut sp44: i32 = 0;
    let mut sp40: u32 = 0;
    let mut sp34: Vec3 = core::mem::zeroed();
    let mut sp28: Vec3 = core::mem::zeroed();
    let mut sp1C: Vec3 = core::mem::zeroed();
    let mut sp10: Vec3 = core::mem::zeroed();

    let ip = GET_ITEM(gobj);
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp10);
    sp28 = sp10;
    sp1C = (*ip).xDD4_itemVar.arwinglaser.xE18;
    sp1C.z = 0.0;
    sp28.z = 0.0;
    if ext::it_8026EA9C(gobj, &mut sp1C, &mut sp28, &mut sp34, &mut sp44, &mut sp40, itArwinglaser_GetCollArg(ip)) != 0 {
        ext::mpGetSpeed(sp44, &mut (*ip).pos, &mut (*ip).x40_vel);
        return 1;
    }
    let mut hit_count = 0;
    let mut i = 0usize;
    while i < 4 {
        sp1C = sp10;
        sp28 = sp1C;
        sp28.x += it_803F8E48[i].x;
        sp28.y += it_803F8E48[i].y;
        sp1C.z = 0.0;
        sp28.z = 0.0;
        if ext::it_8026EA9C(gobj, &mut sp1C, &mut sp28, &mut sp34, &mut sp44, &mut sp40, itArwinglaser_GetCollArg(ip)) != 0 {
            sp1C = sp10;
            sp1C.z = 0.0;
            sp28.z = 0.0;
            if ext::it_8026EA9C(gobj, &mut sp28, &mut sp1C, &mut sp34, &mut sp44, &mut sp40, itArwinglaser_GetCollArg(ip)) != 0
                && ext::mpLineGetFlags(sp44) & 0x100 == 0
            {
                hit_count += 1;
            }
        } else if ext::it_8026EA9C(gobj, &mut sp28, &mut sp1C, &mut sp34, &mut sp44, &mut sp40, itArwinglaser_GetCollArg(ip)) != 0
            && ext::mpLineGetFlags(sp44) & 0x100 == 0
        {
            hit_count += 1;
        }
        i += 1;
    }
    if hit_count != 0 {
        return 1;
    }
    0
}

/// 2E72E0: spawns a beam of `type_` (0/2 main, 4/5 accessory) at `bone`.
pub unsafe extern "C" fn it_802E72E0(parent: *mut Item_GObj, bone: *mut HSD_JObj, type_: i32, scale: f32, scale_mult: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802E72E0");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut sp24: Vec3 = core::mem::zeroed();
    let mut z: f32;

    ext::lb_8000B1CC(bone, null_mut(), &mut sp24);
    match type_ {
        2 | 0 => {
            sp24.z = 0.0;
        }
        4 | 5 => {
            if type_ == 4 {
                z = sp24.z;
                if z < 0.0 {
                    z = -z;
                }
            } else {
                z = sp24.z;
                if z < 0.0 {
                    z = -z;
                }
                z = -z;
            }
            sp24.z = z;
        }
        _ => {}
    }
    spawn.pos = sp24;
    spawn.prev_pos = sp24;
    spawn.kind = It_Kind_Arwing_Laser;
    spawn.facing_dir = scale;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x44_flag.anon_0.set_b0(0);
    spawn.x40 = 0;
    let new_gobj = ext::Item_80268B18(&mut spawn);
    if !new_gobj.is_null() {
        let ip = GET_ITEM(new_gobj);
        let jobj = GET_JOBJ(new_gobj);
        ext::it_80275158(new_gobj, 300.0);
        (*ip).xDD4_itemVar.arwinglaser.xE24 = sp24;
        (*ip).xDD4_itemVar.arwinglaser.xE18 = (*ip).xDD4_itemVar.arwinglaser.xE24;
        (*ip).xDD4_itemVar.arwinglaser.xE30 = 0;
        (*ip).xDD4_itemVar.arwinglaser.xDFC = ext::Ground_801C0498();
        (*ip).xDD4_itemVar.arwinglaser.xDF4 = parent;
        (*ip).xDD4_itemVar.arwinglaser.xDF8 = bone;
        (*ip).xDD4_itemVar.arwinglaser.xE34 = scale_mult;
        (*ip).xDD4_itemVar.arwinglaser.xE38 = type_;
        (*ip).xDD4_itemVar.arwinglaser.xE3C = 0;
        (*ip).xDD4_itemVar.arwinglaser.xE4C = 0;
        HSD_JObjSetRotationY(jobj, (1.5707963267948966f64 * (*ip).facing_dir as f64) as f32);
        match (*ip).xDD4_itemVar.arwinglaser.xE38 {
            0 | 2 => {
                (*ip).xDD4_itemVar.arwinglaser.xE00 = 0.0;
                (*ip).xDD4_itemVar.arwinglaser.xE04 = 0.0;
                (*ip).xDD4_itemVar.arwinglaser.xE08 = 0.0;
                it_802E7A4C(new_gobj);
                match (*ip).xDD4_itemVar.arwinglaser.xE38 {
                    0 => {
                        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        (*ip).xDD4_itemVar.arwinglaser.xE40 = it_802E72E0(new_gobj, bone, 4, scale, scale_mult);
                        (*ip).xDD4_itemVar.arwinglaser.xE44 = it_802E72E0(new_gobj, bone, 5, scale, scale_mult);
                    }
                    2 => {
                        (*ip).xDD4_itemVar.arwinglaser.xE40 = null_mut();
                        (*ip).xDD4_itemVar.arwinglaser.xE44 = null_mut();
                    }
                    _ => {}
                }
            }
            4 | 5 => {
                (*ip).xDD4_itemVar.arwinglaser.xE40 = null_mut();
                (*ip).xDD4_itemVar.arwinglaser.xE44 = null_mut();
                (*ip).xDD4_itemVar.arwinglaser.xE00 = 0.0;
                (*ip).xDD4_itemVar.arwinglaser.xE04 = 0.0;
                if (*ip).xDD4_itemVar.arwinglaser.xE38 == 4 {
                    z = (*ip).pos.z;
                    if z < 0.0 {
                        z = -z;
                    }
                } else {
                    z = (*ip).pos.z;
                    if z < 0.0 {
                        z = -z;
                    }
                    z = -z;
                }
                (*ip).xDD4_itemVar.arwinglaser.xE08 = z;
                it_802E7A4C(new_gobj);
            }
            _ => {}
        }
    }
    new_gobj
}

/// 2E7654: spawns an aimed shot from `bone` toward `target`. (No `return` in the C.)
pub unsafe extern "C" fn it_802E7654(owner: *mut Item_GObj, bone: *mut HSD_JObj, target: *mut Vec3, type_: i32, arg4: i32, scale: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802E7654");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut sp28: Vec3 = core::mem::zeroed();

    ext::lb_8000B1CC(bone, null_mut(), &mut sp28);
    spawn.kind = It_Kind_Arwing_Laser;
    spawn.prev_pos = sp28;
    spawn.facing_dir = 0.0;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x44_flag.anon_0.set_b0(0);
    spawn.x40 = 0;
    let new_gobj = ext::Item_80268B18(&mut spawn);
    if !new_gobj.is_null() {
        let ip = GET_ITEM(new_gobj);
        let jobj = GET_JOBJ(new_gobj);
        ext::it_80275158(new_gobj, 400.0);
        (*ip).xDD4_itemVar.arwinglaser.xE24 = sp28;
        (*ip).xDD4_itemVar.arwinglaser.xE18 = (*ip).xDD4_itemVar.arwinglaser.xE24;
        (*ip).xDD4_itemVar.arwinglaser.xE30 = 0;
        (*ip).xDD4_itemVar.arwinglaser.xDFC = ext::Ground_801C0498();
        (*ip).xDD4_itemVar.arwinglaser.xDF4 = owner;
        (*ip).xDD4_itemVar.arwinglaser.xDF8 = bone;
        (*ip).xDD4_itemVar.arwinglaser.xE34 = scale;
        (*ip).xDD4_itemVar.arwinglaser.xE38 = type_;
        (*ip).xDD4_itemVar.arwinglaser.xE3C = 0;
        (*ip).xDD4_itemVar.arwinglaser.xE48 = arg4;
        (*ip).xDD4_itemVar.arwinglaser.xE4C = 0;
        (*ip).xDD4_itemVar.arwinglaser.xE0C.x = (*target).x - sp28.x;
        (*ip).xDD4_itemVar.arwinglaser.xE0C.y = (*target).y - sp28.y;
        (*ip).xDD4_itemVar.arwinglaser.xE0C.z = (*target).z - sp28.z;
        ext::lbVector_Normalize(&mut (*ip).xDD4_itemVar.arwinglaser.xE0C);
        (*ip).facing_dir = if (*ip).xDD4_itemVar.arwinglaser.xE0C.x >= 0.0 { 1.0 } else { -1.0 };
        let mut rot = -ext::atan2f((*ip).xDD4_itemVar.arwinglaser.xE0C.y, (*ip).xDD4_itemVar.arwinglaser.xE0C.z);
        HSD_JObjSetRotationX(jobj, rot);
        rot = ext::atan2f((*ip).xDD4_itemVar.arwinglaser.xE0C.x, (*ip).xDD4_itemVar.arwinglaser.xE0C.z);
        HSD_JObjSetRotationY(jobj, rot);
        HSD_JObjSetRotationZ(jobj, 0.0);
        it_802E7A4C(new_gobj);
        ext::it_802750F8(new_gobj);
        (*ip).xDCC_flag.set_b3(0);
    }
    new_gobj
}

/// 2E79C8: detaches the accessory beams of a main beam.
pub unsafe extern "C" fn it_802E79C8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E79C8");
    let ip = GET_ITEM(gobj);
    match (*ip).xDD4_itemVar.arwinglaser.xE38 {
        0 | 2 => {
            (*ip).xDD4_itemVar.arwinglaser.xDF4 = null_mut();
            if !(*ip).xDD4_itemVar.arwinglaser.xE40.is_null() {
                let ip2 = GET_ITEM((*ip).xDD4_itemVar.arwinglaser.xE40);
                if (*ip2).xDD4_itemVar.arwinglaser.xDF4 == gobj {
                    (*ip2).xDD4_itemVar.arwinglaser.xDF4 = null_mut();
                }
            }
            (*ip).xDD4_itemVar.arwinglaser.xE40 = null_mut();
            if !(*ip).xDD4_itemVar.arwinglaser.xE44.is_null() {
                let ip3 = GET_ITEM((*ip).xDD4_itemVar.arwinglaser.xE44);
                if (*ip3).xDD4_itemVar.arwinglaser.xDF4 == gobj {
                    (*ip3).xDD4_itemVar.arwinglaser.xDF4 = null_mut();
                }
            }
            (*ip).xDD4_itemVar.arwinglaser.xE44 = null_mut();
        }
        _ => {}
    }
}

/// 2E7A4C
pub unsafe extern "C" fn it_802E7A4C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E7A4C");
    let item = GET_ITEM(gobj);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, (*item).xDD4_itemVar.arwinglaser.xE38, 2);
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion2_Anim(gobj: *mut Item_GObj) {
    melee_core::ct!("itArwinglaser_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.arwinglaser.xE24 = (*ip).xDD4_itemVar.arwinglaser.xE18;
    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.arwinglaser.xE30 = 1;
    }
    ext::it_80273130(gobj);
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion3_Anim(gobj: *mut Item_GObj) {
    melee_core::ct!("itArwinglaser_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.arwinglaser.xE24 = (*ip).xDD4_itemVar.arwinglaser.xE18;
    (*ip).xDD4_itemVar.arwinglaser.xE30 = 1;
    ext::it_80273130(gobj);
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itArwinglaser_UnkMotion2_Phys");
    let mut scale_vec: Vec3 = core::mem::zeroed();
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*((*gobj).user_data as *mut Item)).xC4_article_data).x4_specialAttributes as *mut ArwingLaserAttr;
    let ip = (*gobj).user_data as *mut Item;

    match (*ip).xDD4_itemVar.arwinglaser.xE38 {
        0 | 2 => {
            if (*ip).xDD4_itemVar.arwinglaser.xE30 == 1 {
                (*ip).x40_vel.x = (*(*attrs).x0).x4_throw_speed_mul * (*ip).facing_dir;
                (*ip).x40_vel.z = 0.0;
                (*ip).x40_vel.y = 0.0;
            } else {
                ext::lb_8000B1CC((*ip).xDD4_itemVar.arwinglaser.xDF8, null_mut(), &mut (*ip).pos);
                (*ip).pos.z = 0.0;
                (*ip).x40_vel.z = 0.0;
                (*ip).x40_vel.y = 0.0;
                (*ip).x40_vel.x = 0.0;
            }
        }
        _ => {}
    }
    let s = (*ip).xDD4_itemVar.arwinglaser.xDFC * (*ip).xDD4_itemVar.arwinglaser.xE34;
    scale_vec.z = s;
    scale_vec.y = s;
    scale_vec.x = s;
    HSD_JObjSetScale(jobj, &mut scale_vec);
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itArwinglaser_UnkMotion3_Phys");
    let mut scale_vec: Vec3 = core::mem::zeroed();
    let mut corneria_offset: Vec3 = core::mem::zeroed();
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*((*gobj).user_data as *mut Item)).xC4_article_data).x4_specialAttributes as *mut ArwingLaserAttr;
    let ip = (*gobj).user_data as *mut Item;
    match (*ip).xDD4_itemVar.arwinglaser.xE38 {
        2 => {}
        1 | 3 => {
            (*ip).x40_vel.x = (*ip).xDD4_itemVar.arwinglaser.xE0C.x * ((*(*attrs).x0).x4_throw_speed_mul * (*attrs).x4);
            (*ip).x40_vel.y = (*ip).xDD4_itemVar.arwinglaser.xE0C.y * ((*(*attrs).x0).x4_throw_speed_mul * (*attrs).x4);
            (*ip).x40_vel.z = (*ip).xDD4_itemVar.arwinglaser.xE0C.z * ((*(*attrs).x0).x4_throw_speed_mul * (*attrs).x4);
        }
        _ => {}
    }
    let s = (*ip).xDD4_itemVar.arwinglaser.xDFC * (*ip).xDD4_itemVar.arwinglaser.xE34;
    scale_vec.z = s;
    scale_vec.y = s;
    scale_vec.x = s;
    scale_vec.y *= (*attrs).x8;
    scale_vec.z *= (*attrs).x8;
    HSD_JObjSetScale(jobj, &mut scale_vec);

    match ext::Stage_8022519C(ext::Stage_80225194()) {
        Gr_Kind_Corneria => {
            ext::grCorneria_801DDCF0(&mut corneria_offset);
            (*ip).pos.x += corneria_offset.x;
            (*ip).pos.y += corneria_offset.y;
        }
        _ => {}
    }
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itArwinglaser_UnkMotion2_Coll");
    let mut sp40: Vec3 = core::mem::zeroed();
    let mut prev_pos: Vec3;
    let mut curr_pos: Vec3;
    let mut sp1C: Vec3 = core::mem::zeroed();
    let mut sp18: u32 = 0;
    let mut sp14: i32 = 0;
    let ip = GET_ITEM(gobj);
    let hit: i32;

    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp40);
    (*ip).xDD4_itemVar.arwinglaser.xE18 = sp40;
    let ip2 = GET_ITEM(gobj);
    curr_pos = (*ip2).xDD4_itemVar.arwinglaser.xE18;
    prev_pos = (*ip2).xDD4_itemVar.arwinglaser.xE24;
    prev_pos.z = 0.0;
    curr_pos.z = 0.0;
    if ext::it_8026EA20(gobj, &mut prev_pos, &mut curr_pos, &mut sp1C, &mut sp14, &mut sp18) != 0 {
        ext::mpGetSpeed(sp14, &mut (*ip2).pos, &mut (*ip2).x40_vel);
        (*ip2).pos.x += curr_pos.x - (*ip2).xDD4_itemVar.arwinglaser.xE18.x;
        hit = 1;
    } else {
        hit = 0;
    }
    if hit == 1 {
        ext::efSync_Spawn(0x40E, gobj, &mut sp40);
        ext::Item_8026AE84(ip, 0x74, 0x7F, 0x40);
        return true;
    }
    false
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itArwinglaser_UnkMotion3_Coll");
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);

    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp10);
    (*ip).xDD4_itemVar.arwinglaser.xE18 = sp10;
    if (*ip).xDD4_itemVar.arwinglaser.xE24.z > -30.0 && (*ip).xDD4_itemVar.arwinglaser.xE24.z < 30.0 {
        (*ip).xDD4_itemVar.arwinglaser.xE4C = (*ip).xDD4_itemVar.arwinglaser.xE4C.wrapping_add(1);
        if (*ip).xDD4_itemVar.arwinglaser.xE4C == 1 {
            if (*ip).xDD4_itemVar.arwinglaser.xE48 == 3 {
                ext::Item_8026AE84(ip, 0x5573B, 0x7F, 0x40);
            } else {
                ext::Item_8026AE84(ip, 0x6B6CA, 0x7F, 0x40);
            }
            (*ip).xDD4_itemVar.arwinglaser.xE4C = (*ip).xDD4_itemVar.arwinglaser.xE4C.wrapping_add(1);
        }
        if it_802E70BC(gobj) == 1 {
            ext::efSync_Spawn(0x40E, gobj, &mut sp10);
            ext::Item_8026AE84(ip, 0x74, 0x7F, 0x40);
            return true;
        }
    }
    false
}

pub unsafe extern "C" fn itArwinglaser_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itArwinglaser_UnkMotion5_Coll");
    let mut trans: Vec3 = core::mem::zeroed();
    let mut scale_vec: Vec3 = core::mem::zeroed();
    let mut rot: Quaternion = core::mem::zeroed();
    let ip = GET_ITEM(gobj);

    if !(*((*gobj).user_data as *mut Item)).xDD4_itemVar.arwinglaser.xDF4.is_null() {
        let parent_ip = GET_ITEM((*((*gobj).user_data as *mut Item)).xDD4_itemVar.arwinglaser.xDF4);
        if (*parent_ip).xDD4_itemVar.arwinglaser.xE40 == gobj || (*parent_ip).xDD4_itemVar.arwinglaser.xE44 == gobj {
            HSD_JObjGetTranslation((*(*parent_ip).xBBC_dynamicBoneTable).bones[0], &mut trans);
            trans.z = (*ip).xDD4_itemVar.arwinglaser.xE08;
            HSD_JObjSetTranslate((*(*ip).xBBC_dynamicBoneTable).bones[0], &mut trans);
            (*ip).pos = trans;

            HSD_JObjGetScale((*(*parent_ip).xBBC_dynamicBoneTable).bones[0], &mut scale_vec);
            HSD_JObjSetScale((*(*ip).xBBC_dynamicBoneTable).bones[0], &mut scale_vec);

            HSD_JObjGetRotation((*(*parent_ip).xBBC_dynamicBoneTable).bones[0], &mut rot);
            HSD_JObjSetRotation((*(*ip).xBBC_dynamicBoneTable).bones[0], &mut rot);

            return false;
        }
    }
    true
}

/// 2E838C
pub unsafe extern "C" fn it_802E838C(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E838C");
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp10);
    (*ip).xDD4_itemVar.arwinglaser.xE18 = sp10;
    ext::efSync_Spawn(0x40E, gobj, &mut sp10);
    ext::Item_8026AE84(ip, 0x74, 0x7F, 0x40);
    true
}

/// 2E8418
pub unsafe extern "C" fn it_802E8418(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E8418");
    true
}

/// 2E8420: reflected.
pub unsafe extern "C" fn it_802E8420(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E8420");
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = (*gobj).user_data as *mut Item;
    let jobj = GET_JOBJ(gobj);

    let gobj = if (*ip).xDCC_flag.b1() == 0 { (*ip).owner } else { (*ip).xC64_reflectGObj };
    (*ip).xDD4_itemVar.arwinglaser.xE3C = 1;
    match (*ip).xDD4_itemVar.arwinglaser.xE38 {
        0 | 2 => {
            ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp10);
            (*ip).pos = sp10;
            (*ip).xDD4_itemVar.arwinglaser.xE24 = (*ip).pos;
            (*ip).xDD4_itemVar.arwinglaser.xE18 = (*ip).xDD4_itemVar.arwinglaser.xE24;
            (*ip).facing_dir = -(*ip).facing_dir;
            (*ip).x40_vel.x = -(*ip).x40_vel.x;
            (*ip).x40_vel.z = 0.0;
            (*ip).x40_vel.y = 0.0;
            HSD_JObjSetRotationY(jobj, (1.5707963267948966f64 * (*ip).facing_dir as f64) as f32);
        }
        1 | 3 => {
            if (*ip).x40_vel.x * ext::ftLib_800865C0(gobj) < 0.0 {
                (*ip).facing_dir = -(*ip).facing_dir;
            }
        }
        _ => {}
    }
    false
}

/// 2E85F4: shield-bounced.
pub unsafe extern "C" fn it_802E85F4(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E85F4");
    let mut sp10: Vec3 = core::mem::zeroed();
    let ip = (*gobj).user_data as *mut Item;
    let jobj = GET_JOBJ(gobj);

    (*ip).xDD4_itemVar.arwinglaser.xE3C = 1;
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[4], null_mut(), &mut sp10);
    (*ip).pos = sp10;
    (*ip).xDD4_itemVar.arwinglaser.xE24 = (*ip).pos;
    (*ip).xDD4_itemVar.arwinglaser.xE18 = (*ip).xDD4_itemVar.arwinglaser.xE24;
    match (*ip).xDD4_itemVar.arwinglaser.xE38 {
        2 | 0 => {
            (*ip).facing_dir = if (*ip).x40_vel.x < 0.0 { -1.0 } else { 1.0 };
            HSD_JObjSetRotationY(jobj, (1.5707963267948966f64 * (*ip).facing_dir as f64) as f32);
        }
        _ => {}
    }
    false
}

/// 2E8784
pub unsafe extern "C" fn it_802E8784(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802E8784");
    true
}
