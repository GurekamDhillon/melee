//! Port of src/melee/it/kinds/itpikachutjoltair.c: the air half of Pikachu's Thunder Jolt (logic
//! table 107), which crawls along the surface the ground half left. `xDD8` links back to the ground
//! half. `atan2f` is an `ext` hook, `sinf` / `cosf` the gekko ports. Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itpikachutjoltair.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf};
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026AF0C(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AF0C(a0, a1, a2, a3) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_802B3368(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itpikachutjoltground::it_802B3368(a0 as _) as _ };
    fn it_802B3544(a0: *mut HSD_GObj) => unsafe { crate::itkinds5::it_802b3544(&mut *((*a0).user_data as *mut Item)) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
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

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
const M_TAU: f64 = 6.283185307179586;

/// `ItemStateTable it_803F71D8[]`
pub static mut it_803F71D8: [ItemStateTable; 1] = [
    st_row(
        0,
        pred_b(itPikachutjoltair_UnkMotion0_Anim),
        Some(itPikachutjoltair_UnkMotion0_Phys),
        pred_b(itPikachutjoltair_UnkMotion0_Coll),
    ),
];

/// 2B3EFC
pub unsafe extern "C" fn it_802B3EFC(gobj: *mut Item_GObj) -> *mut Item_GObj {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            return (*ip).xDD4_itemVar.pikachujoltair.xDD8;
        }
    }
    null_mut()
}

/// 2B3F20
pub unsafe extern "C" fn it_802B3F20(gobj: *mut Item_GObj, v: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.pikachujoltair.xDD8.is_null() {
        ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[6], null_mut(), v);
        (*v).z = 0.0;
        return;
    }
    (*v).z = 0.0;
    (*v).y = 0.0;
    (*v).x = 0.0;
}

/// 2B3F88
pub unsafe extern "C" fn it_802B3F88(gobj: *mut Item_GObj, pos: *mut Vec3, coll: *mut CollData, vel: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    if !gobj.is_null() && !ip.is_null() {
        (*ip).pos = *pos;
        (*ip).x378_itemColl = *coll;
        (*ip).xDD4_itemVar.pikachujoltair.xDE8 = *vel;
        (*ip).xDD4_itemVar.pikachujoltair.xDF4 = 0;
        HSD_JObjSetTranslate(jobj, &mut (*ip).pos);
        let angle = ext::atan2f(
            (*ip).xDD4_itemVar.pikachujoltair.xDE8.x,
            (*ip).xDD4_itemVar.pikachujoltair.xDE8.y,
        );
        HSD_JObjSetRotationX(jobj, if (*ip).facing_dir == 1.0 { angle } else { -angle });
        ext::Item_80268E5C(gobj, 0, 0x12);
        ext::Item_802694CC(gobj);
        match (*ip).kind {
            k if k == It_Kind_Kirby_PikachuTJolt_Air || k == It_Kind_Pikachu_TJolt_Air => {
                ext::Item_8026AF0C(ip, 0x3A9CF, 127, 64);
            }
            k if k == It_Kind_Kirby_PichuTJolt_Air || k == It_Kind_Pichu_TJolt_Air => {
                ext::Item_8026AF0C(ip, 0x382B6, 127, 64);
            }
            _ => {}
        }
    }
}

/// 2B4224
pub unsafe extern "C" fn it_802B4224(owner: *mut HSD_GObj, gobj: *mut Item_GObj, pos: *mut Vec3, kind: i32, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    // Item_InitSpawnPosition(&spawn, pos, true)
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, owner, facing_dir, true)
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        // Item_ClearCmdVars(ip)
        (*ip).xDB8_itcmd_var3 = 0;
        (*ip).xDB4_itcmd_var2 = 0;
        (*ip).xDB0_itcmd_var1 = 0;
        (*ip).xDAC_itcmd_var0 = 0;
        (*ip).xDD4_itemVar.pikachujoltair.xDD8 = gobj;
        (*ip).xDD4_itemVar.pikachujoltair.xDF4 = 0;
        HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
        it_802B43D0(item_gobj, owner);
        (*ip).xDD4_itemVar.pikachujoltair.xDD4 = owner;
    }
    item_gobj
}

/// 2B43B0
pub unsafe extern "C" fn it_802B43B0(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            (*ip).xDD4_itemVar.pikachujoltair.xDD8 = null_mut();
        }
    }
}

/// 2B43D0
pub unsafe extern "C" fn it_802B43D0(gobj: *mut Item_GObj, owner: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, 0, 2);
    ext::Item_802694CC(gobj);
    ext::db_80225DD8(gobj, owner);
}

/// `static inline itPikachuTJoltAir_Anim_Destroy`
#[inline]
unsafe fn itPikachuTJoltAir_Anim_Destroy(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802725D4(gobj);
    let linked = (*ip).xDD4_itemVar.pikachujoltair.xDD8;
    if !linked.is_null() {
        ext::it_802B3544(linked);
        (*ip).xDD4_itemVar.pikachujoltair.xDD8 = null_mut();
    }
}

/// `static inline itPikachutjoltair_UnkMotion0_Anim_inline`
#[inline]
unsafe fn itPikachutjoltair_UnkMotion0_Anim_inline(gobj: *mut Item_GObj) -> bool {
    let linked = (*GET_ITEM(gobj)).xDD4_itemVar.pikachujoltair.xDD8;
    if linked.is_null() {
        return true;
    }
    if ext::it_802B3368(linked) != gobj {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPikachutjoltair_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    let angle = ext::atan2f(
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.x,
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.y,
    );
    HSD_JObjSetRotationX(jobj, if (*ip).facing_dir == 1.0 { angle } else { -angle });
    (*ip).xDD4_itemVar.pikachujoltair.xDF4 = (*ip).xDD4_itemVar.pikachujoltair.xDF4.wrapping_add(1);
    if !(((*ip).xDD4_itemVar.pikachujoltair.xDF4 <= 24) && itPikachutjoltair_UnkMotion0_Anim_inline(gobj) == false) {
        itPikachuTJoltAir_Anim_Destroy(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn itPikachutjoltair_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
}

/// 2B45E8 (static)
pub unsafe extern "C" fn itPikachutjoltair_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

#[inline]
unsafe fn destroy_linked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::it_802725D4(gobj);
    if !(*ip).xDD4_itemVar.pikachujoltair.xDD8.is_null() {
        ext::it_802B3544((*ip).xDD4_itemVar.pikachujoltair.xDD8);
        (*ip).xDD4_itemVar.pikachujoltair.xDD8 = null_mut();
    }
    true
}

pub unsafe extern "C" fn it_2725_Logic107_DmgDealt(gobj: *mut Item_GObj) -> bool {
    destroy_linked(gobj)
}

pub unsafe extern "C" fn it_2725_Logic107_Clanked(gobj: *mut Item_GObj) -> bool {
    destroy_linked(gobj)
}

pub unsafe extern "C" fn it_2725_Logic107_Absorbed(gobj: *mut Item_GObj) -> bool {
    destroy_linked(gobj)
}

pub unsafe extern "C" fn it_2725_Logic107_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[6], null_mut(), &mut (*ip).pos);
    (*ip).facing_dir = -(*ip).facing_dir;

    let rot: f32 = if (*ip).facing_dir == 1.0 {
        HSD_JObjGetRotationX(jobj)
    } else {
        -HSD_JObjGetRotationX(jobj)
    };

    let mut angle: f32 = (rot as f64 + (M_PI - HSD_JObjGetRotationX((*(*ip).xBBC_dynamicBoneTable).bones[4]) as f64)) as f32;

    while angle < 0.0 {
        angle = (angle as f64 + M_TAU) as f32;
    }
    while angle as f64 > M_TAU {
        angle = (angle as f64 - M_TAU) as f32;
    }

    HSD_JObjSetRotationX(jobj, angle);
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    HSD_JObjSetTranslate(jobj, &mut (*ip).pos);

    if (*ip).facing_dir == 1.0 {
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.x = sinf(angle);
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.y = cosf(angle);
    } else {
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.x = sinf(-angle);
        (*ip).xDD4_itemVar.pikachujoltair.xDE8.y = cosf(-angle);
    }
    (*ip).xDD4_itemVar.pikachujoltair.xDE8.z = 0.0;
    (*ip).xDD4_itemVar.pikachujoltair.xDF4 = 0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    false
}

pub unsafe extern "C" fn it_2725_Logic107_HitShield(gobj: *mut Item_GObj) -> bool {
    destroy_linked(gobj)
}

pub unsafe extern "C" fn it_2725_Logic107_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    destroy_linked(gobj)
}

pub unsafe extern "C" fn itPikachuTJoltAir_Logic107_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
