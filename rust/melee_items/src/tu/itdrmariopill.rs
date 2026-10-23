//! Native port of src/melee/it/kinds/itdrmariopill.c (Dr. Mario's pill): the spawn variants, the
//! flight (`it_803F75D0`), the stick-to-Dr. Mario motion 2 and the hit/shield responses. Callees
//! in other TUs go through `ext`; callees ported in this crate are called directly. The C
//! `__frsqrte` based `my_sqrtf` (4 Newton steps) is ported with `melee_core::fp::frsqrte`.
//! `*_p` wrappers return `i32` for the state table (`HSD_GObjPredicate`). Checked by
//! tests/tu_itdrmariopill.rs.
//!
//! Fused sites (shipping contraction): `VEC2_SQ_LEN(v)` = `fmadds(x, x, y*y)`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::{fmadds, frsqrte};

use crate::itlib::{it_8026b3a8, it_8026b894, it_80273030, it_802756d0};
use crate::helpers::it_80275158;
use crate::itlib3::it_80272860;
use crate::itlib3::it_80274658;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;
/// `JOBJ_HIDDEN` (sysdolphin/baselib/jobj.h).
pub const JOBJ_HIDDEN: u32 = 1 << 4;
/// `M_PI_2` (src/MSL/math.h): a double literal.
pub const M_PI_2: f64 = 1.5707963267948966;

// BEGIN GENERATED EXT (callees of the C TU that are not ported in this crate yet)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(spawn: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3);
    fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing: f32, flag: bool);
    fn Item_80268E5C(gobj: *mut HSD_GObj, msid: i32, flags: u32);
    fn Item_8026A8EC(gobj: *mut HSD_GObj);
    fn Item_8026AE84(ip: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026AB54(gobj: *mut HSD_GObj, parent: *mut HSD_GObj, bone: u32);
    fn Item_CopyJObjScale(dst: *mut HSD_JObj, src: *mut HSD_JObj, scale: *mut Vec3);
    fn it_80273670(gobj: *mut HSD_GObj, a: i32, f: f32);
    fn it_802750F8(gobj: *mut HSD_GObj);
    fn it_80274740(gobj: *mut HSD_GObj);
    fn it_8026D9A0(gobj: *mut HSD_GObj);
    fn it_8027781C(gobj: *mut HSD_GObj) -> i32;
    fn db_80225DD8(item: *mut HSD_GObj, owner: *mut HSD_GObj);
    fn ftLib_80086644(gobj: *mut HSD_GObj, v: *mut Vec3);
    fn ftLib_800869D4(gobj: *mut HSD_GObj) -> f32;
    fn ftDr_Init_80149844(gobj: *mut HSD_GObj) -> bool;
    fn ftDr_Init_801498A0(gobj: *mut HSD_GObj);
    fn ftDr_Init_801498EC(gobj: *mut HSD_GObj) -> u32;
    fn ftCo_800BF228(gobj: *mut HSD_GObj) -> bool;
    fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *const Vec3);
    fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, v: f32);
    fn HSD_JObjSetScale(jobj: *mut HSD_JObj, scale: *const Vec3);
    fn HSD_JObjSetFlags(jobj: *mut HSD_JObj, flags: u32);
    fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, flags: u32);
    }
    // Data defined in other TUs.
    pub static mut it_804D6D28: *const ItemCommonData = core::ptr::null();
}
// END GENERATED EXT

macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
pub unsafe extern "C" fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `GET_ATTRS(ip)`: `ip->xC4_article_data->x4_specialAttributes` as `itDrMarioPillAttributes`.
#[inline]
pub unsafe extern "C" fn attrs_of<'a>(ip: *mut Item) -> &'a itDrMarioPillAttributes {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const itDrMarioPillAttributes)
}

/// `HSD_JObjGetChild(jobj)` (NULL-safe).
#[inline]
pub unsafe extern "C" fn jobj_child(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() {
        null_mut()
    } else {
        (*j).child
    }
}

/// `Item_ClearCmdVars` (src/melee/it/inlines.h, static inline).
#[inline]
pub unsafe extern "C" fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `my_sqrtf` (itdrmariopill.c, static inline): `__frsqrte` refined by 4 Newton steps, then
/// `x * guess` rounded to f32. Returns `x` for `x <= 0`.
#[inline]
fn my_sqrtf(x: f32) -> f32 {
    const HALF: f64 = 0.5;
    const THREE: f64 = 3.0;
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        for _ in 0..4 {
            guess = HALF * guess * (THREE - guess * guess * xd);
        }
        return (xd * guess) as f32;
    }
    x
}

/// `itDrMarioPill_Spawn` (itdrmariopill.c): spawn a pill from `parent_gobj`, flown with
/// `attrs->x0` / `x4` (via `itDrMarioPill_802C0B5C`).
pub unsafe extern "C" fn itDrMarioPill_Spawn(parent_gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: i32, kind: ItemKind, facing_dir: f32) {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    ext::Item_InitSpawnPositionFromParent(&mut spawn, parent_gobj, pos);
    ext::Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        itDrMarioPill_802C0B5C(gobj);
        ext::it_80273670(gobj, 0, arg2 as f32);
        ext::db_80225DD8(gobj, parent_gobj);
        ext::it_802750F8(gobj);
        it_8026b3a8(&mut *item_of(gobj));
    }
}

/// `itDrMarioPill_802C061C` (static): follow the held Dr. Mario's position while attached.
pub unsafe extern "C" fn itDrMarioPill_802C061C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    if !(*ip).xDD4_itemVar.drmariopill.x4.is_null() {
        let mut v: Vec3 = core::mem::zeroed();
        ext::ftLib_80086644((*ip).xDD4_itemVar.drmariopill.x4, &mut v);
        (*ip).pos = v;
        ext::HSD_JObjSetTranslate(jobj, &v);
        ext::HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).xDD4_itemVar.drmariopill.x0 as f64) as f32);
        ext::HSD_JObjSetRotationZ(jobj, 0.0);
    }
    it_8026b3a8(&mut *ip);
}

/// `itDrMarioPill_Appeal_Spawn`: the Dr. Mario appeal pill, attached to its owner.
pub unsafe extern "C" fn itDrMarioPill_Appeal_Spawn(parent_gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: i32, kind: ItemKind, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    ext::Item_InitSpawnPositionFromParent(&mut spawn, parent_gobj, pos);
    ext::Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = item_of(gobj);
        itDrMarioPill_802C0DF8(gobj);
        ext::it_80273670(gobj, 1, arg2 as f32);
        it_8026b3a8(&mut *ip);
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
        Item_ClearCmdVars(ip);
        (*ip).xDC8_word.flags.set_x19(0);
        (*ip).on_accessory = Some(itDrMarioPill_802C061C);
        (*ip).xDC8_word.flags.set_x13(1);
        (*ip).owner = parent_gobj;
        (*ip).xDD4_itemVar.drmariopill.x4 = parent_gobj as *mut Item_GObj;
        (*ip).xDD4_itemVar.drmariopill.x0 = facing_dir;
        ext::it_80274740(gobj);
        it_802756d0(&mut *ip);
        it_8026b3a8(&mut *ip);
        ext::db_80225DD8(gobj, parent_gobj);
        ext::it_802750F8(gobj);
        it_8026b3a8(&mut *ip);
    }
    gobj
}

/// `itDrMarioPill_802C09C4`: the Dr. Mario pill spawned for a fighter part `part` (`msid` picks
/// the motion; msid 2 attaches it to `part`).
pub unsafe extern "C" fn itDrMarioPill_802C09C4(parent_gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: i32, kind: ItemKind, msid: i32, part: Fighter_Part, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = kind;
    ext::Item_InitSpawnPositionFromParent(&mut spawn, parent_gobj, pos);
    ext::Item_InitSpawnCommonFields(&mut spawn, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = item_of(gobj);
        itDrMarioPill_802C1180(gobj, msid);
        ext::it_80273670(gobj, 1, arg2 as f32);
        it_802756d0(&mut *ip);
        it_8026b3a8(&mut *ip);
        ext::Item_80268E5C(gobj, msid + 4, ITEM_ANIM_UPDATE);
        Item_ClearCmdVars(ip);
        (*ip).xDC8_word.flags.set_x19(0);
        (*ip).on_accessory = Some(itDrMarioPill_802C061C);
        (*ip).xDC8_word.flags.set_x13(1);
        (*ip).owner = parent_gobj;
        (*ip).xDD4_itemVar.drmariopill.x4 = parent_gobj as *mut Item_GObj;
        (*ip).xDD4_itemVar.drmariopill.x0 = facing_dir;
        ext::it_80274740(gobj);
        it_802756d0(&mut *ip);
        it_8026b3a8(&mut *ip);
        ext::it_802750F8(gobj);
        if msid == 2 {
            ext::Item_8026AB54(gobj, parent_gobj, part as u32);
        }
        it_8026b3a8(&mut *ip);
    }
    gobj
}

/// `itDrMarioPill_802C0B5C`: launch the pill along the attribute angle (`x0` speed, `x4` angle).
pub unsafe extern "C" fn itDrMarioPill_802C0B5C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = attrs_of(ip);
    let temp_f31 = attrs.x0;
    let temp_f30 = attrs.x4;
    (*ip).x40_vel.x = (*ip).facing_dir * (temp_f31 * melee_runtime::gekko::cosf(temp_f30));
    (*ip).x40_vel.y = temp_f31 * melee_runtime::gekko::sinf(temp_f30);
    (*ip).x40_vel.z = 0.0;
    it_80275158(&mut *ip, attrs.x8, &*ext::it_804D6D28);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    it_8026b3a8(&mut *ip);
}

/// `itDrMarioPill_UnkMotion0_Anim` (static): tick the lifetime.
pub unsafe extern "C" fn itDrMarioPill_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    (*ip).xD44_lifeTimer -= 1.0;
    (*ip).xD44_lifeTimer <= 0.0
}

/// `itDrMarioPill_UnkMotion0_Phys` (static): fall, then the common physics.
pub unsafe extern "C" fn itDrMarioPill_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    let it_attr = &*(*ip).xCC_item_attr;
    it_80272860(&mut *ip, it_attr.x10_fall_speed, it_attr.x14_fall_speed_max);
    it_80274658(&mut *ip, (*ext::it_804D6D28).x68_float);
}

/// `itDrmariopill_UnkMotion0_Coll` (static): settle once slow enough, else play the bounce sound.
pub unsafe extern "C" fn itDrmariopill_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D9A0(gobj);
    it_8026b3a8(&mut *item_of(gobj));
    if ext::it_8027781C(gobj) != 0 {
        let ip = item_of(gobj);
        let attrs = attrs_of(ip);
        let v = (*ip).x40_vel;
        let sq = fmadds(v.x, v.x, v.y * v.y);
        if my_sqrtf(sq) < attrs.x10 {
            return true;
        }
        ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    }
    false
}

/// `itDrMarioPill_802C0DBC`: drop the held reference and free the item.
pub unsafe extern "C" fn itDrMarioPill_802C0DBC(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        let ip = item_of(gobj);
        if !ip.is_null() {
            (*ip).xDD4_itemVar.drmariopill.x4 = null_mut();
            ext::Item_8026A8EC(gobj);
        }
    }
}

/// `itDrMarioPill_802C0DF8`: the appeal pill's flight (motion 1, 1200 units of lifetime).
pub unsafe extern "C" fn itDrMarioPill_802C0DF8(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    it_80275158(&mut *ip, 1200.0, &*ext::it_804D6D28);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    it_8026b3a8(&mut *ip);
    it_8026b3a8(&mut *ip);
}

/// `itDrMarioPill_Motion2_Anim_sub` (static inline): forget the held reference and clear the
/// on-accessory; true.
pub unsafe extern "C" fn itDrMarioPill_Motion2_Anim_sub(arg_gobj: *mut HSD_GObj) -> bool {
    if !arg_gobj.is_null() {
        let ip = item_of(arg_gobj);
        if !ip.is_null() {
            let gobj = (*ip).xDD4_itemVar.drmariopill.x4 as *mut HSD_GObj;
            if !gobj.is_null() && (*ip).owner == gobj {
                ext::ftDr_Init_801498A0(gobj);
            }
            (*ip).xDD4_itemVar.drmariopill.x4 = null_mut();
            (*ip).on_accessory = None;
        }
    }
    let ip = if !arg_gobj.is_null() { item_of(arg_gobj) } else { null_mut() };
    if !ip.is_null() {
        (*ip).xDD4_itemVar.drmariopill.x4 = null_mut();
        (*ip).on_accessory = None;
        true
    } else {
        false
    }
}

/// `itDrMarioPill_Motion2_Anim_flags` (static inline): show the child joint while the flag is set.
pub unsafe extern "C" fn itDrMarioPill_Motion2_Anim_flags(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = jobj_child((*gobj).hsd_obj as *mut HSD_JObj);
    it_8026b3a8(&mut *ip);
    if (*ip).xDAC_itcmd_var0 == 1 {
        ext::HSD_JObjClearFlags(jobj, JOBJ_HIDDEN);
    } else {
        ext::HSD_JObjSetFlags(jobj, JOBJ_HIDDEN);
    }
}

/// `itDrMarioPill_Motion2_Anim` (static): stick to Dr. Mario; release when he lets go.
pub unsafe extern "C" fn itDrMarioPill_Motion2_Anim(gobj: *mut HSD_GObj) -> bool {
    let hobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = item_of(gobj);
    it_802756d0(&mut *ip);
    it_8026b3a8(&mut *ip);
    itDrMarioPill_Motion2_Anim_flags(gobj);
    if !(*ip).xDD4_itemVar.drmariopill.x4.is_null() {
        let held = (*ip).xDD4_itemVar.drmariopill.x4 as *mut HSD_GObj;
        if ext::ftDr_Init_80149844(held) == true {
            if itDrMarioPill_Motion2_Anim_sub(gobj) {
                return true;
            }
        } else {
            (*ip).xDAC_itcmd_var0 = ext::ftDr_Init_801498EC(held);
            if (*ip).owner == held {
                let mag = ext::ftLib_800869D4(held);
                let scale = Vec3 { x: mag, y: mag, z: mag };
                ext::HSD_JObjSetScale(hobj, &scale);
            } else if itDrMarioPill_Motion2_Anim_sub(gobj) {
                return true;
            }
        }
    } else if itDrMarioPill_Motion2_Anim_sub(gobj) {
        return true;
    }
    false
}

/// `itDrMarioPill_Motion2_Phys` (static): the common physics.
pub unsafe extern "C" fn itDrMarioPill_Motion2_Phys(gobj: *mut HSD_GObj) {
    it_802756d0(&mut *item_of(gobj));
    it_8026b3a8(&mut *item_of(gobj));
}

/// `itDrMariopill_Motion2_Coll` (static): nothing to collide with (false).
pub unsafe extern "C" fn itDrMariopill_Motion2_Coll(gobj: *mut HSD_GObj) -> bool {
    it_802756d0(&mut *item_of(gobj));
    it_8026b3a8(&mut *item_of(gobj));
    false
}

/// `itDrMarioPill_802C1180`: set the motion for `msid` and its speed attribute `x8`.
pub unsafe extern "C" fn itDrMarioPill_802C1180(gobj: *mut HSD_GObj, msid: enum_t) {
    let ip = item_of(gobj);
    it_80275158(&mut *ip, attrs_of(ip).x8, &*ext::it_804D6D28);
    ext::Item_80268E5C(gobj, msid + 4, ITEM_ANIM_UPDATE);
    it_8026b3a8(&mut *ip);
}

/// `itDrMarioPill_PickedUp`: motion 6 with the attribute lifetime.
pub unsafe extern "C" fn itDrMarioPill_PickedUp(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = attrs_of(ip);
    it_80275158(&mut *ip, attrs.x8, &*ext::it_804D6D28);
    ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    it_8026b3a8(&mut *ip);
}

/// `itDrMarioPill_Motion6_Anim` (static): copy the held Dr. Mario's scale onto the pill's joint.
pub unsafe extern "C" fn itDrMarioPill_Motion6_Anim(orig: *mut HSD_GObj) -> bool {
    let ip = item_of(orig);
    it_8026b3a8(&mut *ip);
    let gobj = (*ip).xDD4_itemVar.drmariopill.x4 as *mut HSD_GObj;
    if !gobj.is_null() && ext::ftCo_800BF228(gobj) == true {
        let mut scale: Vec3 = core::mem::zeroed();
        ext::Item_CopyJObjScale(
            (*orig).hsd_obj as *mut HSD_JObj,
            (*gobj).hsd_obj as *mut HSD_JObj,
            &mut scale,
        );
    }
    false
}

/// `itDrMarioPill_Motion6_Phys` (static): the common physics.
pub unsafe extern "C" fn itDrMarioPill_Motion6_Phys(gobj: *mut HSD_GObj) {
    it_8026b3a8(&mut *item_of(gobj));
}

/// `itDrMarioPill_Motion6_Coll` (static): false.
pub unsafe extern "C" fn itDrMarioPill_Motion6_Coll(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    false
}

/// `itDrMarioPill_DmgDealt`: the hit sound; true.
pub unsafe extern "C" fn itDrMarioPill_DmgDealt(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    true
}

/// `itDrMarioPill_Reflected`: `it_80273030`.
pub unsafe extern "C" fn itDrMarioPill_Reflected(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    it_80273030(&mut *item_of(gobj))
}

/// `itDrMarioPill_Clanked` (identical to `itDrMarioPill_DmgDealt`).
pub unsafe extern "C" fn itDrMarioPill_Clanked(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    true
}

/// `itDrMarioPill_HitShield` (identical to `itDrMarioPill_DmgDealt`).
pub unsafe extern "C" fn itDrMarioPill_HitShield(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    true
}

/// `itDrMarioPill_Absorbed` (identical to `itDrMarioPill_DmgDealt`).
pub unsafe extern "C" fn itDrMarioPill_Absorbed(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    true
}

/// `itDrMarioPill_ShieldBounced` (identical to `itDrMarioPill_DmgDealt`).
pub unsafe extern "C" fn itDrMarioPill_ShieldBounced(gobj: *mut HSD_GObj) -> bool {
    it_8026b3a8(&mut *item_of(gobj));
    let ip = item_of(gobj);
    it_8026b3a8(&mut *ip);
    ext::Item_8026AE84(ip, 0x15FAE, 0x7F, 0x40);
    true
}

/// `itDrMarioPill_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itDrMarioPill_EvtUnk(gobj: *mut HSD_GObj, hobj: *mut HSD_GObj) {
    it_8026b894(&mut *item_of(gobj), hobj);
}

pred_i32! {
    itDrMarioPill_UnkMotion0_Anim_p => itDrMarioPill_UnkMotion0_Anim;
    itDrmariopill_UnkMotion0_Coll_p => itDrmariopill_UnkMotion0_Coll;
    itDrMarioPill_Motion2_Anim_p => itDrMarioPill_Motion2_Anim;
    itDrMariopill_Motion2_Coll_p => itDrMariopill_Motion2_Coll;
    itDrMarioPill_Motion6_Anim_p => itDrMarioPill_Motion6_Anim;
    itDrMarioPill_Motion6_Coll_p => itDrMarioPill_Motion6_Coll;
}

/// `it_803F75D0[]` (itdrmariopill.c).
pub static it_803F75D0: [ItemStateTable; 7] = [
    ItemStateTable { anim_id: 0, animated: Some(itDrMarioPill_UnkMotion0_Anim_p), physics_updated: Some(itDrMarioPill_UnkMotion0_Phys), collided: Some(itDrmariopill_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: 1, animated: Some(itDrMarioPill_Motion2_Anim_p), physics_updated: Some(itDrMarioPill_Motion2_Phys), collided: Some(itDrMariopill_Motion2_Coll_p) },
    ItemStateTable { anim_id: 2, animated: Some(itDrMarioPill_Motion2_Anim_p), physics_updated: Some(itDrMarioPill_Motion2_Phys), collided: Some(itDrMariopill_Motion2_Coll_p) },
    ItemStateTable { anim_id: 1, animated: Some(itDrMarioPill_Motion6_Anim_p), physics_updated: Some(itDrMarioPill_Motion6_Phys), collided: Some(itDrMarioPill_Motion6_Coll_p) },
    ItemStateTable { anim_id: 3, animated: Some(itDrMarioPill_Motion6_Anim_p), physics_updated: Some(itDrMarioPill_Motion6_Phys), collided: Some(itDrMarioPill_Motion6_Coll_p) },
    ItemStateTable { anim_id: 4, animated: Some(itDrMarioPill_Motion6_Anim_p), physics_updated: Some(itDrMarioPill_Motion6_Phys), collided: Some(itDrMarioPill_Motion6_Coll_p) },
    ItemStateTable { anim_id: 5, animated: Some(itDrMarioPill_Motion6_Anim_p), physics_updated: Some(itDrMarioPill_Motion6_Phys), collided: Some(itDrMarioPill_Motion6_Coll_p) },
];
