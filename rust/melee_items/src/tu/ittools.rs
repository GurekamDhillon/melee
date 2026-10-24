//! Port of src/melee/it/kinds/ittools.c: the Pokemon Stadium "tools" item (logic table 22). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_ittools.rs.
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
    fn ftLib_800866DC(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB20(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB20(a0) };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
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

/// `ItemStateTable it_803F9260[]`
pub static mut it_803F9260: [ItemStateTable; 10] = [
    st_row(0, pred_b(itTools_UnkMotion4_Anim), Some(itTools_UnkMotion4_Phys), pred_b(itTools_UnkMotion4_Coll)),
    st_row(1, pred_b(itTools_UnkMotion4_Anim), Some(itTools_UnkMotion4_Phys), pred_b(itTools_UnkMotion4_Coll)),
    st_row(2, pred_b(itTools_UnkMotion4_Anim), Some(itTools_UnkMotion4_Phys), pred_b(itTools_UnkMotion4_Coll)),
    st_row(3, pred_b(itTools_UnkMotion4_Anim), Some(itTools_UnkMotion4_Phys), pred_b(itTools_UnkMotion4_Coll)),
    st_row(4, pred_b(itTools_UnkMotion4_Anim), Some(itTools_UnkMotion4_Phys), pred_b(itTools_UnkMotion4_Coll)),
    st_row(5, pred_b(itTools_UnkMotion9_Anim), Some(itTools_UnkMotion9_Phys), pred_b(itTools_UnkMotion9_Coll)),
    st_row(6, pred_b(itTools_UnkMotion9_Anim), Some(itTools_UnkMotion9_Phys), pred_b(itTools_UnkMotion9_Coll)),
    st_row(7, pred_b(itTools_UnkMotion9_Anim), Some(itTools_UnkMotion9_Phys), pred_b(itTools_UnkMotion9_Coll)),
    st_row(8, pred_b(itTools_UnkMotion9_Anim), Some(itTools_UnkMotion9_Phys), pred_b(itTools_UnkMotion9_Coll)),
    st_row(9, pred_b(itTools_UnkMotion9_Anim), Some(itTools_UnkMotion9_Phys), pred_b(itTools_UnkMotion9_Coll)),
];

/// `ABS(x)` (Runtime/platform.h) for f32.
#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `attrs->motions[idx]` (the C declares `motions[1]` and indexes past it)
#[inline]
unsafe fn motion(attrs: *mut itToolsAttributes, idx: i32) -> *mut itToolsMotionAttrs {
    (*attrs).motions.as_mut_ptr().offset(idx as isize)
}

#[inline]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itToolsAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itToolsAttributes
}

/// `itReflectItemAndUpdateRotation` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn itReflectItemAndUpdateRotation(gobj: *mut Item_GObj) -> bool {
    let PI_2: f32 = (core::f64::consts::PI / 2.0) as f32;
    let ip = HSD_GObjGetUserData(gobj) as *mut Item;
    let jobj = GET_JOBJ(gobj);

    ext::it_80273030(gobj);
    HSD_JObjSetRotationY(jobj, PI_2 * (*ip).facing_dir);
    false
}

pub unsafe extern "C" fn it_802EEFA8(pos: *mut Vec3, arg1: i32, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Tools;
    spawn.prev_pos = *pos;
    spawn.pos = *pos;
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        it_802EF098(gobj, arg1);
        return gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn itTools_Logic22_DmgDealt(gobj: *mut Item_GObj) -> bool {
    it_802EF320(gobj);
    false
}

pub unsafe extern "C" fn it_802EF098(gobj: *mut Item_GObj, arg1: i32) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    (*ip).xDD4_itemVar.tools.x0 = arg1;
    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).xD5C = 0;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = -(*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x0;
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTools_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = GET_ATTRS(ip);
    let mut rz = HSD_JObjGetRotationZ(jobj);
    // `rz += xC * facing_dir`: one fused multiply-add (shipping contraction; f32 = f64 product + sum, then f32)
    rz = ((*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).xC as f64 * (*ip).facing_dir as f64 + rz as f64) as f32;
    HSD_JObjSetRotationZ(jobj, rz);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        (*ip).xD44_lifeTimer = 0.0;
        it_802EF320(gobj);
    }
    false
}

pub unsafe extern "C" fn itTools_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_80272860(gobj, (*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x4, (*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x8);
}

pub unsafe extern "C" fn itTools_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).x40_vel.x != 0.0 || (*ip).x40_vel.y != 0.0 {
        if ext::it_8026DAA8(gobj) & 3 != 0 {
            it_802EF320(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn it_802EF320(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xD44_lifeTimer = (((*attrs).x4 as i32) & 0xFFFE) as f32;
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTools_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        (*ip).xD44_lifeTimer = 0.0;
        return true;
    }
    if ((*ip).xD44_lifeTimer as i32) & 2 != 0 {
        ext::it_8026BB20(gobj);
        ext::it_80273670(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, 0.0);
    } else {
        ext::it_8026BB44(gobj);
    }
    false
}

pub unsafe extern "C" fn itTools_UnkMotion9_Phys(_gobj: *mut Item_GObj) {}

#[inline]
unsafe fn itTools_UnkMotion9_Coll_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xD44_lifeTimer = (((*attrs).x4 as i32) & 0xFFFE) as f32;
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTools_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ((*ip).x40_vel.x != 0.0 || (*ip).x40_vel.y != 0.0) && (ext::it_8026DAA8(gobj) & 3) != 0 {
        itTools_UnkMotion9_Coll_inline(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802EF548(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    let mut fighter: *mut Fighter_GObj;
    let mut new_damage: f32;
    let mut fighter_pos: Vec3 = core::mem::zeroed();
    let mut dir: f32;

    if ABS((*ip).x40_vel.x) > (*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x14 {
        return;
    }

    fighter = null_mut();
    (*ip).xD44_lifeTimer = (*ip).xD48_halfLifeTimer;
    if !(*ip).xCEC_fighterGObj.is_null() {
        fighter = (*ip).xCEC_fighterGObj;
    }

    new_damage = (*ip).xCA0 as f32;
    // `-1.0f * x` is a real multiply (a NaN keeps its sign); black_box keeps LLVM from folding it into a negation
    dir = core::hint::black_box(-1.0f32) * (*ip).xCCC_incDamageDirection;
    if !fighter.is_null() {
        if ext::ftLib_80086960(fighter) != 0 {
            ext::ftLib_800866DC(fighter, &mut fighter_pos);
        } else {
            return;
        }
        if (*ip).pos.x > fighter_pos.x {
            dir = 1.0;
        } else {
            dir = -1.0;
        }
    }

    new_damage *= (*attrs).x8;
    new_damage += (*ip).x5D4_hitboxes[0].hit.damage;
    if new_damage > ((*attrs).xC as u32) as f32 {
        new_damage = ((*attrs).xC as u32) as f32;
    }
    ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, new_damage as u32, gobj);

    (*ip).x40_vel.x = dir * ((1.0 + ((*ip).x5D4_hitboxes[0].hit.damage - 10.0)) * (*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x18);
    (*ip).x40_vel.y = (*motion(attrs, (*ip).xDD4_itemVar.tools.x0)).x10;
}

pub unsafe extern "C" fn itTools_Logic22_DmgReceived(gobj: *mut Item_GObj) -> bool {
    it_802EF548(gobj);
    false
}

pub unsafe extern "C" fn it_2725_Logic22_Clanked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xD44_lifeTimer = (((*attrs).x4 as i32) & 0xFFFE) as f32;
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, ITEM_ANIM_UPDATE);
    false
}

pub unsafe extern "C" fn it_2725_Logic22_HitShield(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xD44_lifeTimer = (((*attrs).x4 as i32) & 0xFFFE) as f32;
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, ITEM_ANIM_UPDATE);
    false
}

pub unsafe extern "C" fn it_2725_Logic22_Absorbed(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attrs = GET_ATTRS(ip);
    ext::it_802756D0(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xD44_lifeTimer = (((*attrs).x4 as i32) & 0xFFFE) as f32;
    ext::Item_80268E5C(gobj, (*ip).xDD4_itemVar.tools.x0 + 5, ITEM_ANIM_UPDATE);
    (*ip).xD44_lifeTimer = 0.0;
    false
}

pub unsafe extern "C" fn itTools_Logic22_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itTools_Logic22_Reflected(gobj: *mut Item_GObj) -> bool {
    itReflectItemAndUpdateRotation(gobj)
}

pub unsafe extern "C" fn it_802EFA24(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
