//! Port of src/melee/it/it_2725.c: the item bone and hitbox helpers (hitbox enable/disable, the bone walk
//! `get_bone_by_id`, the scale/rotation/spin of a model), the throw/landing/hold transitions, the lifetime and
//! flag setters, and the sound/bounce callbacks. Functions already ported in `itlib`/`itlib3` (it_80272818,
//! it_80272860, it_80273030, it_80273130, it_80273484, it_80273500, it_80274658, it_80274990, it_80275158,
//! it_80275174, it_80275210, it_80275228, it_80275240, it_80275258, it_80275270, it_80275328, it_80275390,
//! it_80275870) are not repeated here. Callees of other TUs go through `ext` (unset hooks panic "TODO").
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use crate::tu::it_26B1::{it_8026B390, it_8026B3A8, it_8026BB88, it_8026BBCC, it_8026BDB4, it_8026B6C8};

// BEGIN GENERATED EXT (the callees of the C TU that are not ported in this crate)
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
    fn HSD_GObjObject_80390B0C(a0: *mut HSD_GObj);
    fn HSD_JObjAddRotationX(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjAddRotationY(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjAddRotationZ(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: i32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: i32);
    fn HSD_JObjAddScaleX(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjAddScaleY(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjAddScaleZ(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec3) -> *mut c_void;
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjGetRotationX(a0: *mut HSD_JObj) -> f32;
    fn HSD_JObjGetRotationY(a0: *mut HSD_JObj) -> f32;
    fn HSD_JObjGetRotationZ(a0: *mut HSD_JObj) -> f32;
    fn HSD_JObjGetTranslation(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetRotationX(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetRotationY(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetRotationZ(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetScale(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_JObjSetTranslate(a0: *mut HSD_JObj, a1: *mut Vec3);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_802680CC(a0: *mut HSD_GObj);
    fn Item_8026849C(a0: *mut HSD_GObj);
    fn Item_802697D4(a0: *mut HSD_GObj);
    fn Item_80269978(a0: *mut HSD_GObj);
    fn Item_8026A848(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026AFA0(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Item_8026B074(a0: *mut Item);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn db_ShowEnemyStompRange() -> u32;
    fn ftLib_80086630(a0: *mut HSD_GObj, a1: Fighter_Part) -> *mut HSD_JObj;
    fn ftLib_80086A0C(a0: *mut HSD_GObj) -> f32;
    fn ftLib_80086D40(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn ftLib_80086DC4(a0: i32, a1: i32);
    fn ftLib_80086E68(a0: *mut HSD_GObj);
    fn ftLib_80086EB4(a0: *mut HSD_GObj) -> i32;
    fn it_8027129C(a0: *mut HSD_GObj, a1: i32);
    fn it_802756D0(a0: *mut HSD_GObj);
    fn it_802756E0(a0: *mut HSD_GObj);
    fn it_802758D4(a0: *mut HSD_GObj) -> f32;
    fn it_802762BC(a0: *mut Item);
    fn it_80275BC8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_802787B4(a0: *mut HSD_GObj, a1: i32);
    fn it_80279BBC(a0: *mut Item);
    fn it_80274DFC(a0: *mut HSD_GObj);
    fn it_8027B070(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_8027B378(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: f32);
    fn it_8027B4A4(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_80274594(a0: *mut HSD_GObj);
    fn it_80275534(a0: *mut HSD_GObj, a1: f32);
    fn it_80274C60(a0: *mut HSD_GObj);
    fn it_80274F10(a0: *mut HSD_GObj);
    fn it_80275070(a0: *mut HSD_GObj, a1: i32);
    fn it_80274990(a0: *mut HSD_GObj) -> f32;
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    fn itIsHeavy(a0: *mut HSD_GObj) -> bool;
    fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
    fn lbAudioAx_80024DC4(a0: i32);
    fn lbColl_80008428(a0: *mut HitCapsule);
    fn lbColl_80008434(a0: *mut HitCapsule);
    fn lbVector_Add(a0: *mut Vec3, a1: *mut Vec3) -> *mut Vec3;
    fn lbVector_Mirror(a0: *mut Vec3, a1: *mut Vec3);
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3);
    fn lb_8000B09C(a0: *mut HSD_JObj) -> bool;
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint);
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32);
    fn lb_8000C1C0(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn lb_8000C2F8(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn lb_8000C390(a0: *mut HSD_JObj);
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32);
    fn ftLib_80086960(a0: *mut HSD_GObj) -> bool;
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8;
    }
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = core::ptr::null_mut();
    pub static mut it_804D6D28: *mut ItemCommonData = core::ptr::null_mut();
    pub static mut it_804D6D0C: i32 = 0;
    pub static mut it_804D6D08: i32 = 0;
}
// END GENERATED EXT

const M_PI_2: f64 = 1.5707963267948966;

#[inline(always)]
unsafe fn item_of(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `it_8027129C_by_4` (static inline): the four hitbox updates of an item.
unsafe fn it_8027129C_by_4(item_gobj: *mut HSD_GObj) {
    for cnt in 0..4 {
        ext::it_8027129C(item_gobj, cnt);
    }
}

/// `it_80272560`: clears hitbox `idx`'s hit and the item's flag 2 of `xDAA`, then re-runs the four updates.
pub unsafe extern "C" fn it_80272560(item_gobj: *mut HSD_GObj, idx: i32) {
    let item = item_of(item_gobj);
    ext::lbColl_80008428(&raw mut (*item).x5D4_hitboxes[idx as usize].hit);
    (*item).anon_daa.xDAA_flag.anon_0.set_b2(0);
    it_8027129C_by_4(item_gobj);
}

/// `it_802725D4`: disables all four hitboxes and clears the item's hitbox-active flags.
pub unsafe extern "C" fn it_802725D4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    for i in 0..4 {
        ext::lbColl_80008428(&raw mut (*item).x5D4_hitboxes[i].hit);
    }
    for i in 0..4 {
        ext::it_8027129C(item_gobj, i as i32);
    }
    (*item).xDC8_word.flags.set_x16(0);
    (*item).anon_daa.xDAA_flag.anon_0.set_b2(0);
}

/// `it_80272674`: advances hitbox `idx`'s position history (the enabled/unk2/unk3 states).
pub unsafe extern "C" fn it_80272674(item_gobj: *mut HSD_GObj, idx: i32) {
    let mut item = item_of(item_gobj);
    ext::lbColl_80008434(&raw mut (*item).x5D4_hitboxes[idx as usize].hit);
    (*item).xDC8_word.flags.set_x16(1);

    item = item_of(item_gobj);
    let hitbox = &raw mut (*item).x5D4_hitboxes[idx as usize].hit;
    let state = (*hitbox).state;
    match state {
        HitCapsule_Enabled => {
            ext::lb_8000B1CC((*hitbox).jobj, &raw mut (*hitbox).b_offset, &raw mut (*hitbox).x4C);
            (*hitbox).x58 = (*hitbox).x4C;
            (*hitbox).state = HitCapsule_Unk2;
            (*item).anon_daa.xDAA_flag.anon_0.set_b2(1);
        }
        HitCapsule_Unk2 => {
            (*hitbox).state = HitCapsule_Unk3;
            (*hitbox).x58 = (*hitbox).x4C;
            ext::lb_8000B1CC((*hitbox).jobj, &raw mut (*hitbox).b_offset, &raw mut (*hitbox).x4C);
        }
        HitCapsule_Unk3 => {
            (*hitbox).x58 = (*hitbox).x4C;
            ext::lb_8000B1CC((*hitbox).jobj, &raw mut (*hitbox).b_offset, &raw mut (*hitbox).x4C);
        }
        _ => {}
    }
}

/// `it_80272784_inline` (static inline).
unsafe fn it_80272784_inline(item_gobj: *mut HSD_GObj) {
    for i in 0..4 {
        ext::it_8027129C(item_gobj, i as i32);
    }
}

/// `it_80272784`: disables all hitboxes and sets the item's flag 0x16.
pub unsafe extern "C" fn it_80272784(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    for i in 0..4 {
        ext::lbColl_80008434(&raw mut (*item).x5D4_hitboxes[i].hit);
    }
    (*item).xDC8_word.flags.set_x16(1);
    it_80272784_inline(item_gobj);
}

/// `it_80272818`: the fighter stored in the item's special attributes (a `it_2728_DatAttrs`).
pub unsafe extern "C" fn it_80272818(item: *mut Item) -> *mut Fighter {
    let attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut it_2728_DatAttrs;
    (*attrs).fighter
}

/// `it_80275158` (itlib/helpers.rs has the same port; repeated here because it is a 2-line setter the
/// TU calls directly): both life timers are set, the half one scaled by `x4C_float`.
unsafe fn it_80275158_local(item_gobj: *mut HSD_GObj, lifetime: f32) {
    let item = item_of(item_gobj);
    (*item).xD44_lifeTimer = lifetime;
    (*item).xD48_halfLifeTimer = lifetime * (*ext::it_804D6D28).x4C_float;
}

/// `it_80272D1C`: true for a live item gobj of the item class.
pub unsafe extern "C" fn it_80272D1C(item_gobj: *mut HSD_GObj) -> bool {
    !item_gobj.is_null() && (*item_gobj).classifier == HSD_GOBJ_CLASS_ITEM as u16
}

/// `it_80272D40`: 0 for a fighter-owned item, 1 for an item with its own collision, 2 otherwise.
pub unsafe extern "C" fn it_80272D40(item_gobj: *mut HSD_GObj) -> i32 {
    if ext::ftLib_80086960(item_gobj) {
        return 0;
    }
    let chk = !item_gobj.is_null() && (*item_gobj).classifier == HSD_GOBJ_CLASS_ITEM as u16;
    if chk {
        return 1;
    }
    2
}

/// `itColl_BounceOffVictim`: bounces the item off its victim (scaled x velocity, offset y velocity).
pub unsafe extern "C" fn itColl_BounceOffVictim(gobj: *mut HSD_GObj) {
    let item = item_of(gobj);
    let d = ext::it_804D6D28;
    (*item).x40_vel.x *= (*d).x58_float;
    (*item).x40_vel.y = fmadds((*item).x40_vel.y, (*d).x5C_float, (*d).x60_float);
}

/// `it_80272DE4`: scales the jobj on all three axes by adding `scale` to each.
pub unsafe extern "C" fn it_80272DE4(jobj: *mut HSD_JObj, scale: f32) {
    ext::HSD_JObjAddScaleX(jobj, scale);
    ext::HSD_JObjAddScaleY(jobj, scale);
    ext::HSD_JObjAddScaleZ(jobj, scale);
}

/// `it_80272F7C`: sets the jobj's scale to `scale` on all axes.
pub unsafe extern "C" fn it_80272F7C(jobj: *mut HSD_JObj, scale: f32) {
    let mut temp = Vec3 { x: scale, y: scale, z: scale };
    ext::HSD_JObjSetScale(jobj, &raw mut temp);
}

/// `itColl_BounceOffShield`: mirrors the velocity off the shield and updates the facing direction.
pub unsafe extern "C" fn itColl_BounceOffShield(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    ext::lbVector_Mirror(&raw mut (*item).x40_vel, &raw mut (*item).xC58);
    let item = item_of(item_gobj);
    if !((*item).x40_vel.x.abs() < 0.00001f32) || (*item).facing_dir == 0.0 {
        let dir = if (*item).x40_vel.x >= 0.0 { 1.0f32 } else { -1.0f32 };
        (*item).facing_dir = dir;
    }
    let int_dir = if (*item).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&raw mut (*item).x378_itemColl, int_dir);
    false
}

/// `it_80273030`-neighbour: the item's life timer counts down; true when it reaches zero.
pub unsafe extern "C" fn it_80273130(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    (*item).xD44_lifeTimer -= 1.0;
    (*item).xD44_lifeTimer <= 0.0
}

/// `it_80273168`: plays the item's sound at `xD70` unless flag `xDCD` bit 2 is set.
pub unsafe extern "C" fn it_80273168(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        ext::Item_8026AE84(item, (*item).xD70, 0x7F, 0x40);
    }
}

/// `it_802731A4`: plays the item's sound at `xD78` unless flag `xDCD` bit 2 is set.
pub unsafe extern "C" fn it_802731A4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        ext::Item_8026AE84(item, (*item).xD78, 0x7F, 0x40);
    }
}

/// `it_802731E0`: plays the item's sound at `xD74` unless flag `xDCD` bit 2 is set.
pub unsafe extern "C" fn it_802731E0(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        ext::Item_8026AE84(item, (*item).xD74, 0x7F, 0x40);
    }
}

/// `it_8027321C`: the Unk4 kind plays its sound through the audio layer, others through `Item_8026AE84`.
pub unsafe extern "C" fn it_8027321C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        if (*item).kind == It_Kind_Unk4 {
            ext::lbAudioAx_80023870((*item).xD84, 0x7F, 0x40, 0x9A);
            return;
        }
        ext::Item_8026AE84(item, (*item).xD84, 0x7F, 0x40);
    }
}

/// `it_8027327C`: sets the item's motion `ID1` (when not -1) and plays `ID2` unless flag `xDCD` bit 2 is set.
pub unsafe extern "C" fn it_8027327C(item_gobj: *mut HSD_GObj, id1: i32, id2: i32) {
    let item = item_of(item_gobj);
    if id1 != -1 {
        ext::it_802787B4(item_gobj, id1);
    }
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        it_802732E4(item, id2);
        if id2 == 0x12F {
            ext::lbAudioAx_80024DC4(id2);
        }
    }
}

/// `it_802732E4`: plays sound `arg1` unless flag `xDCD` bit 2 is set.
pub unsafe extern "C" fn it_802732E4(item: *mut Item, arg1: i32) {
    if (*item).xDCD_flag.anon_0.b2() == 0 {
        ext::Item_8026AE84(item, arg1, 0x7F, 0x40);
    }
}

/// `it_80273318`: attaches the item to `joint`, resets its dynamics and places its jobj at its position.
pub unsafe extern "C" fn it_80273318(item_gobj: *mut HSD_GObj, joint: *mut HSD_Joint) {
    let item = item_of(item_gobj);
    (*item).xC8_joint = joint;
    ext::HSD_GObjObject_80390B0C(item_gobj);
    ext::Item_802680CC(item_gobj);
    ext::Item_8026849C(item_gobj);
    ext::HSD_JObjSetTranslate((*item_gobj).hsd_obj as *mut HSD_JObj, &raw mut (*item).pos);
}

/// `it_80273408`: clears the item's velocity, accumulators and nudges.
pub unsafe extern "C" fn it_80273408(item_gobj: *mut HSD_GObj) {
    it_80273454(item_gobj);
    it_8027346C(item_gobj);
    it_80273484(item_gobj);
    it_8027349C(item_gobj);
    it_802734B4(item_gobj);
}

/// `it_80273454`: `itResetVelocity` (static inline in the C header): zeroes the item velocity.
pub unsafe extern "C" fn it_80273454(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x40_vel.x = 0.0;
    (*item).x40_vel.y = 0.0;
    (*item).x40_vel.z = 0.0;
}

/// `it_8027346C`: clears `x64_vec_unk2`.
pub unsafe extern "C" fn it_8027346C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x64_vec_unk2.x = 0.0;
    (*item).x64_vec_unk2.y = 0.0;
    (*item).x64_vec_unk2.z = 0.0;
}

/// `it_80273484`: clears `x58_vec_unk`.
pub unsafe extern "C" fn it_80273484(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x58_vec_unk.x = 0.0;
    (*item).x58_vec_unk.y = 0.0;
    (*item).x58_vec_unk.z = 0.0;
}

/// `it_8027349C`: clears `x70_nudge`.
pub unsafe extern "C" fn it_8027349C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x70_nudge.x = 0.0;
    (*item).x70_nudge.y = 0.0;
    (*item).x70_nudge.z = 0.0;
}

/// `it_802734B4`: resets the item's vector set (`x7C`, `x88`, `x94`, `xA0`) and sets `xAC_unk` to up.
pub unsafe extern "C" fn it_802734B4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).x7C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*item).x88 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*item).x94 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*item).xA0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*item).xAC_unk.x = 0.0;
    (*item).xAC_unk.z = 0.0;
    (*item).xAC_unk.y = 1.0;
}

/// `it_80273598`: the owner's `ftLib_80086D40` with the two arguments, when the owner is a fighter.
pub unsafe extern "C" fn it_80273598(item_gobj: *mut HSD_GObj, arg1: i32, arg2: i32) {
    let item = item_of(item_gobj);
    if !(*item).owner.is_null() && ext::ftLib_80086960((*item).owner) {
        ext::ftLib_80086D40((*item).owner, arg1, arg2);
    }
}

/// `it_80273600`: `ftLib_80086E68` on the owner, when it is a fighter.
pub unsafe extern "C" fn it_80273600(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    if !(*item).owner.is_null() && ext::ftLib_80086960((*item).owner) {
        ext::ftLib_80086E68((*item).owner);
    }
}

/// `it_80273648`: `ftLib_80086DC4` with the two arguments.
pub unsafe extern "C" fn it_80273648(_item_gobj: *mut HSD_GObj, arg1: i32, arg2: i32) {
    ext::ftLib_80086DC4(arg1, arg2);
}

/// `it_80273670`: sets the item's state (`arg1` indexes the article's state table), starts its animation at
/// `arg8` and resets the command pointer.
pub unsafe extern "C" fn it_80273670(item_gobj: *mut HSD_GObj, arg1: i32, arg8: f32) {
    let item = item_of(item_gobj);
    let item_jobj1 = jobj_of(item_gobj);
    (*item).xD0_itemStateDesc = (*(*(*item).xC4_article_data).xC_itemStates).x0_itemStateDesc.as_mut_ptr().add(arg1 as usize);
    if !(*item).xD0_itemStateDesc.is_null() {
        ext::HSD_JObjRemoveAnimAll(item_jobj1);
        let joint = (*item).xC8_joint;
        if !joint.is_null() {
            let item_jobj2 = if item_jobj1.is_null() { null_mut() } else { (*item_jobj1).child };
            ext::lb_8000B804(item_jobj2, (*joint).child);
        }
        let desc = (*item).xD0_itemStateDesc;
        ext::HSD_JObjAddAnimAll(item_jobj1, (*desc).x0_anim_joint as *mut c_void, (*desc).x4_matanim_joint as *mut c_void, (*desc).x8_parameters as *mut c_void);
        ext::lb_8000BA0C(item_jobj1, (*item).x5D0_animFrameSpeed);
        ext::HSD_JObjReqAnimAll(item_jobj1, arg8);
    }
    ext::HSD_JObjAnimAll(item_jobj1);
    ext::HSD_JObjRemoveAnimAll(item_jobj1);
    (*item).x524_cmd.anon_8.u = null_mut();
}

/// `it_80273748`: launches the item from its holder with `vel` (the throw): owner, facing, rotation and the
/// holder-relative position.
pub unsafe extern "C" fn it_80273748(item_gobj: *mut HSD_GObj, pos: *mut Vec3, vel: *mut Vec3) {
    let mut sp54: Vec3 = core::mem::zeroed();
    let mut sp48: Vec3 = core::mem::zeroed();
    let mut sp3c: Vec3 = core::mem::zeroed();
    let item = item_of(item_gobj);
    let jobj = jobj_of(item_gobj);
    let mut owner = (*item).owner;
    it_80275070(item_gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id);
    if !it_8026B6C8(item_gobj) {
        it_8026B390(item_gobj);
    }
    ext::it_802756E0(item_gobj);
    let item_2 = item_of(item_gobj);
    let jobj_2 = jobj_of(item_gobj);
    let rotation = ext::it_80274990(item_gobj);
    ext::lb_8000B804(jobj_2, (*item_2).xC8_joint);
    ext::Item_8026849C(item_gobj);
    ext::it_80274658(item_gobj, rotation);
    if (*item).xDC8_word.flags.x0() != 0 {
        owner = (*item).x51C;
    }
    (*item).x40_vel = *vel;
    (*item).x40_vel.x *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    (*item).x40_vel.y *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    (*item).x40_vel.z *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    {
        let item = item_of(item_gobj);
        let vel_x = (*item).x40_vel.x.abs();
        if !(vel_x < 0.00001f32) || (*item).facing_dir == 0.0 {
            let dir = if (*item).x40_vel.x >= 0.0 { 1.0f32 } else { -1.0f32 };
            (*item).facing_dir = dir;
        }
        let int_dir = if (*item).facing_dir == -1.0 { -1 } else { 1 };
        ext::mpCollSetFacingDir(&raw mut (*item).x378_itemColl, int_dir);
    }
    if (*item).xDC8_word.flags.x19() == 1 {
        let dir_radians = M_PI_2 as f32 * (*item).facing_dir;
        ext::HSD_JObjSetRotationY(jobj, dir_radians);
    }
    let hold_kind = (*item).hold_kind;
    if (ext::itIsHeavy(item_gobj) && hold_kind != 4 && hold_kind != 6) || (*item).hold_kind == 8 {
        let jobj2 = it_80272C90(item_gobj);
        ext::HSD_JObjGetTranslation(jobj2, &raw mut sp3c);
        sp3c.x = -sp3c.x;
        sp3c.y = -sp3c.y;
        sp3c.z = -sp3c.z;
        ext::lb_8000B1CC(ext::ftLib_80086630(owner, (*item).xDC4), &raw mut sp3c, &raw mut sp54);
        ext::lb_8000B1CC(ext::ftLib_80086630(owner, (*item).xDC4), null_mut(), &raw mut sp48);
        sp3c.x = sp54.x - sp48.x;
        sp3c.y = sp54.y - sp48.y;
        sp3c.z = 0.0;
    } else {
        sp3c.x = 0.0;
        sp3c.y = 0.0;
        sp3c.z = 0.0;
    }
    (*item).pos.x = (*pos).x + sp3c.x;
    (*item).pos.y = (*pos).y + sp3c.y;
    (*item).pos.z = 0.0;
    ext::HSD_JObjSetTranslate(jobj, &raw mut (*item).pos);
}

/// `getOwnerJointPosition` (static inline): the owner's part position, through `offset` when given.
unsafe fn getOwnerJointPosition(item: *mut Item, owner_gobj: *mut HSD_GObj, offset: *mut Vec3, world_pos: *mut Vec3) {
    if !offset.is_null() {
        ext::lb_8000B1CC(ext::ftLib_80086630(owner_gobj, (*item).xDC4), offset, world_pos);
    } else {
        ext::lb_8000B1CC(ext::ftLib_80086630(owner_gobj, (*item).xDC4), null_mut(), world_pos);
    }
}

/// `it_80273B50`: the holder-relative throw (the owner's joint position and the velocity).
pub unsafe extern "C" fn it_80273B50(item_gobj: *mut HSD_GObj, vel: *mut Vec3) {
    let mut sp40: Vec3 = core::mem::zeroed();
    let mut sp34: Vec3 = core::mem::zeroed();
    let item = item_of(item_gobj);
    let item_jobj1 = jobj_of(item_gobj);
    let mut owner_gobj = (*item).owner;
    it_80275070(item_gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id);
    if !it_8026B6C8(item_gobj) {
        it_8026B390(item_gobj);
    }
    ext::it_802756E0(item_gobj);
    let rotation;
    {
        let item = item_of(item_gobj);
        let item_jobj2 = jobj_of(item_gobj);
        rotation = ext::it_80274990(item_gobj);
        ext::lb_8000B804(item_jobj2, (*item).xC8_joint);
        ext::Item_8026849C(item_gobj);
    }
    ext::it_80274658(item_gobj, rotation);
    if (*item).xDC8_word.flags.x0() != 0 {
        owner_gobj = (*item).x51C;
    }
    (*item).x40_vel = *vel;
    (*item).x40_vel.x *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    (*item).x40_vel.y *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    (*item).x40_vel.z *= (*(*item).xCC_item_attr).x4_throw_speed_mul;
    {
        let item2 = item_of(item_gobj);
        let vel_x = (*item2).x40_vel.x.abs();
        if !(vel_x < 0.00001f32) || (*item2).facing_dir == 0.0 {
            let dir = if (*item2).x40_vel.x >= 0.0 { 1.0f32 } else { -1.0f32 };
            (*item2).facing_dir = dir;
        }
        let int_dir = if (*item2).facing_dir == -1.0 { -1 } else { 1 };
        ext::mpCollSetFacingDir(&raw mut (*item2).x378_itemColl, int_dir);
    }
    if (*item).xDC8_word.flags.x19() == 1 {
        let dir_radians = M_PI_2 as f32 * (*item).facing_dir;
        ext::HSD_JObjSetRotationY(item_jobj1, dir_radians);
    }
    let hold_kind = (*item).hold_kind;
    if (ext::itIsHeavy(item_gobj) && hold_kind != 4 && hold_kind != 6) || (*item).hold_kind == 8 {
        let jobj = it_80272C90(item_gobj);
        ext::HSD_JObjGetTranslation(jobj, &raw mut sp40);
        sp40.x = -sp40.x;
        sp40.y = -sp40.y;
        sp40.z = -sp40.z;
    } else {
        sp40.x = 0.0;
        sp40.y = 0.0;
        sp40.z = 0.0;
    }
    {
        let item3 = item_of(item_gobj);
        let item_jobj3 = jobj_of(item_gobj);
        getOwnerJointPosition(item3, owner_gobj, &raw mut sp40, &raw mut sp34);
        let pos = &raw mut (*item3).pos;
        (*item3).pos.x = sp34.x;
        (*item3).pos.y = sp34.y;
        (*item3).pos.z = 0.0;
        ext::HSD_JObjSetTranslate(item_jobj3, pos);
    }
}

/// `it_80273F34`: the common landing/throw-out setup (flags, counters, owner-relative state and attach).
pub unsafe extern "C" fn it_80273F34(item_gobj: *mut HSD_GObj, arg_gobj2: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let item_jobj = jobj_of(item_gobj);
    (*item).xDC8_word.flags.set_x13(0);
    (*item).xDC8_word.flags.set_x14(1);
    (*item).xDC4 = 0;
    (*item).xD54_throwNum += 1;
    (*item).xD50_landNum = 0;

    ext::it_80275BC8(item_gobj, arg_gobj2);
    it_80274DAC(item_gobj);

    if (*item).xDC8_word.flags.x19() == 1 {
        let rotate = M_PI_2 as f32 * (*item).facing_dir;
        ext::HSD_JObjSetRotationY(item_jobj, rotate);
    }

    if ext::ftLib_80086960(arg_gobj2) {
        ext::Item_8026A848(item_gobj, arg_gobj2);
        ext::it_8027B070(item_gobj, arg_gobj2);
    }

    let int_dir = if (*item).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&raw mut (*item).x378_itemColl, int_dir);
    ext::Item_8026B074(item);
    ext::it_802762BC(item);
    ext::HSD_JObjSetTranslate(item_jobj, &raw mut (*item).pos);

    ext::it_8027B4A4(arg_gobj2, item_gobj);
    ext::it_8027B378(arg_gobj2, item_gobj, ext::it_802758D4(item_gobj));
}

/// `it_80274198`: `it_80273F34` on the owner, or on the secondary owner when the item is flagged `x0`.
pub unsafe extern "C" fn it_80274198(item_gobj: *mut HSD_GObj, chk: bool) {
    let item = item_of(item_gobj);
    let mut owner_gobj = (*item).owner;
    if (*item).xDC8_word.flags.x0() != 0 {
        if chk {
            let x51c_gobj = (*item).x51C;
            (*item).owner = x51c_gobj;
            owner_gobj = x51c_gobj;
            (*item).x51C = null_mut();
        } else {
            owner_gobj = (*item).x51C;
        }
    }
    it_80273F34(item_gobj, owner_gobj);
}

/// `it_802741F4`: same as `it_80274198`.
pub unsafe extern "C" fn it_802741F4(item_gobj: *mut HSD_GObj, chk: bool) {
    let item = item_of(item_gobj);
    let mut owner_gobj = (*item).owner;
    if (*item).xDC8_word.flags.x0() != 0 {
        if chk {
            let x51c_gobj = (*item).x51C;
            (*item).owner = x51c_gobj;
            owner_gobj = x51c_gobj;
            (*item).x51C = null_mut();
        } else {
            owner_gobj = (*item).x51C;
        }
    }
    it_80273F34(item_gobj, owner_gobj);
}

/// `it_80274250`: `it_80273B50` then `it_80273F34` on the owner (or the secondary owner when flagged).
pub unsafe extern "C" fn it_80274250(item_gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    it_80273B50(item_gobj, arg1);
    let item = item_of(item_gobj);
    let mut owner_gobj = (*item).owner;
    if (*item).xDC8_word.flags.x0() != 0 {
        owner_gobj = (*item).x51C;
    }
    it_80273F34(item_gobj, owner_gobj);
}

/// `it_8027429C`: `it_80273B50` then `it_80273F34`, moving the secondary owner into the primary slot.
pub unsafe extern "C" fn it_8027429C(item_gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    it_80273B50(item_gobj, arg1);
    let item = item_of(item_gobj);
    let mut owner_gobj = (*item).owner;
    if (*item).xDC8_word.flags.x0() != 0 {
        let x51c_gobj = (*item).x51C;
        owner_gobj = x51c_gobj;
        (*item).owner = x51c_gobj;
        (*item).x51C = null_mut();
    }
    it_80273F34(item_gobj, owner_gobj);
}

/// `it_802742F4`: the grab/hold attach of the item to `gobj`'s part `ftpart` (the holder's bone).
pub unsafe extern "C" fn it_802742F4(item_gobj: *mut HSD_GObj, gobj: *mut HSD_GObj, ftpart: Fighter_Part) {
    let item = item_of(item_gobj);
    ext::lb_8000B804(jobj_of(item_gobj), (*item).xC8_joint);
    ext::Item_8026849C(item_gobj);
    if ext::ftLib_80086960(gobj) {
        let bit_idx = ext::ftLib_80086BE0(gobj) as u32;
        let cur = (*item).xDC8_word.flags.xF();
        (*item).xDC8_word.flags.set_xF((cur as u32 | 1u32.wrapping_shl(bit_idx)) as _);
        if (*item).xDC8_word.flags.x0() == 0 {
            (*item).owner = gobj;
        } else {
            (*item).x51C = gobj;
        }
        (*item).xDC4 = ftpart;
        (*item).x20_team_id = ext::ftLib_80086EB4(gobj) as u8;
        (*item).xD50_landNum = 0;
        (*item).xDC8_word.flags.set_x13(1);
        ext::HSD_JObjClearFlagsAll((*jobj_of(item_gobj)).child, JOBJ_HIDDEN);
        ext::it_802756D0(item_gobj);
        it_8026B3A8(item_gobj);
        ext::db_80225DD8(item_gobj, gobj);
    }
    if (*item).xDD0_flag.anon_0.b6() != 0 {
        (*item).xDD0_flag.anon_0.set_b6(0);
        (*item).xD40 = 0.0;
        ext::it_80279BBC(item);
    }
    if (*item).kind < It_Kind_L_Gun_Ray {
        // A common item.
        it_80275158_local(item_gobj, (*ext::it_804D6D28).x30_lifetime as f32);
    }
    it_80274F48(item_gobj, (*(*(*item).xC4_article_data).x10_modelDesc).x8_bone_attach_id, gobj, ftpart);
    it_80274C88(item_gobj);
}

/// `it_80274484`: sets the item and its jobj to `scale` and re-derives the scale-dependent sizes.
pub unsafe extern "C" fn it_80274484(item_gobj: *mut HSD_GObj, jobj: *mut HSD_JObj, scale: f32) {
    let item = item_of(item_gobj);
    (*item).scl = scale;
    let mut sp1c = Vec3 { x: (*item).scl, y: (*item).scl, z: (*item).scl };
    ext::HSD_JObjSetScale(jobj, &raw mut sp1c);
    ext::it_80275534(item_gobj, (*item).x3C);
    it_80274DFC(item_gobj);
    it_80274E44(item_gobj);
}

/// `it_80274574`: `it_80274594`.
pub unsafe extern "C" fn it_80274574(item_gobj: *mut HSD_GObj) {
    it_80274594(item_gobj);
}

/// `HSD_JObjSetScale_2` (static inline): the jobj scale write with its matrix-dirty bit.
unsafe fn HSD_JObjSetScale_2(jobj: *mut HSD_JObj, scale: *mut Vec3) {
    (*jobj).scale = *scale;
    if (*jobj).flags & (1 << 25) == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

/// `it_80274594`: multiplies the item's scale by the owner's scale and applies it.
pub unsafe extern "C" fn it_80274594(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let item_jobj = jobj_of(item_gobj);
    (*item).scl *= ext::ftLib_80086A0C((*item).owner);
    let mut sp18 = Vec3 { x: (*item).scl, y: (*item).scl, z: (*item).scl };
    HSD_JObjSetScale_2(item_jobj, &raw mut sp18);
    ext::it_80275534(item_gobj, (*item).x3C);
    it_80274DFC(item_gobj);
    it_80274E44(item_gobj);
}

/// `it_802746F8`: the jobj of the item's bone (walks `bit_chk` children down the model).
pub unsafe extern "C" fn it_802746F8(item_gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let mut item_jobj1 = item_gobj_jobj(item_gobj);
    let bits = (*(*(*item_of(item_gobj)).xC4_article_data).x10_modelDesc).xC_bit_field;
    let bit_chk = (bits >> 6) & 3;
    if bit_chk != 0 {
        for _ in 0..bit_chk {
            item_jobj1 = if item_jobj1.is_null() { null_mut() } else { (*item_jobj1).child };
        }
    }
    item_jobj1
}

#[inline(always)]
unsafe fn item_gobj_jobj(item_gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    jobj_of(item_gobj)
}

/// `it_80274740`: zeroes the spin speed and sets the spin on the axis selected by `xDC8` bits 0x17.
pub unsafe extern "C" fn it_80274740(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let mut item_jobj = jobj_of(item_gobj);
    let bits = (*(*(*item).xC4_article_data).x10_modelDesc).xC_bit_field;
    let bit_chk = (bits >> 6) & 3;
    if bit_chk != 0 {
        for _ in 0..bit_chk {
            item_jobj = if item_jobj.is_null() { null_mut() } else { (*item_jobj).child };
        }
    }
    (*item).xD3C_spinSpeed = 0.0;
    match (*item).xDC8_word.flags.x17() {
        0 => ext::HSD_JObjSetRotationZ(item_jobj, (*item).xD3C_spinSpeed),
        1 => ext::HSD_JObjSetRotationX(item_jobj, (*item).xD3C_spinSpeed),
        _ => ext::HSD_JObjSetRotationY(item_jobj, (*item).xD3C_spinSpeed),
    }
}

/// `it_80274A64`: adds the item's spin to the rotation on the axis selected by `xDC8` bits 0x17.
pub unsafe extern "C" fn it_80274A64(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let mut item_jobj = jobj_of(item_gobj);
    let bits = (*(*(*item).xC4_article_data).x10_modelDesc).xC_bit_field;
    let bit_chk = (bits >> 6) & 3;
    if bit_chk != 0 {
        for _ in 0..bit_chk {
            item_jobj = if item_jobj.is_null() { null_mut() } else { (*item_jobj).child };
        }
    }
    match (*item).xDC8_word.flags.x17() {
        0 => ext::HSD_JObjAddRotationZ(item_jobj, (*item).xD3C_spinSpeed),
        1 => ext::HSD_JObjAddRotationX(item_jobj, (*item).xD3C_spinSpeed),
        _ => ext::HSD_JObjAddRotationY(item_jobj, (*item).xD3C_spinSpeed),
    }
}

/// `it_80274C60`: clears `xDC8` bit 0xC.
pub unsafe extern "C" fn it_80274C60(item_gobj: *mut HSD_GObj) {
    (*item_of(item_gobj)).xDC8_word.flags.set_xC(0);
}

/// `it_80274C78`: `xDC8` bit 0xC.
pub unsafe extern "C" fn it_80274C78(item_gobj: *mut HSD_GObj) -> bool {
    (*item_of(item_gobj)).xDC8_word.flags.xC() != 0
}

/// `it_80274C88`: clears `xDD0` bit 0 and `xDAA` bit 3.
pub unsafe extern "C" fn it_80274C88(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xDD0_flag.anon_0.set_b0(0);
    (*item).anon_daa.xDAA_flag.anon_0.set_b3(0);
}

/// `it_80274CAC`: sets `xDD0` bit 0, and `xDAA` bit 3 when the stomp-range debug view is on.
pub unsafe extern "C" fn it_80274CAC(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xDD0_flag.anon_0.set_b0(1);
    if ext::db_ShowEnemyStompRange() != 0 {
        (*item).anon_daa.xDAA_flag.anon_0.set_b3(1);
    }
}

/// `it_80274D04`: the item's position offset from its last position (the `lb_80014638` arg).
pub unsafe extern "C" fn it_80274D04(item_gobj: *mut HSD_GObj, arg1: *mut lb_80014638_arg1_t) {
    let mut sp20: Vec3 = core::mem::zeroed();
    let mut sp14: Vec3 = core::mem::zeroed();
    it_8026BB88(item_gobj, &raw mut sp20);
    it_8026BBCC(item_gobj, &raw mut sp14);
    (*arg1).unk_x = sp20.x - sp14.x;
    (*arg1).unk_y = sp20.y - sp14.y;
}

/// `it_80274D6C`: the item's ECB right/left/top corner (x8 vector) placed at its position.
pub unsafe extern "C" fn it_80274D6C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let right_x = (*ip).x378_itemColl.ecb.right.x;
    let right_y = (*ip).x378_itemColl.ecb.right.y;
    let left_x = (*ip).x378_itemColl.ecb.left.x;
    let top_y = (*ip).x378_itemColl.ecb.top.y;
    (*ip).xB54.x8.y = right_x + (*ip).pos.x;
    (*ip).xB54.x8.x = left_x + (*ip).pos.x;
    (*ip).xB54.x8.z = right_y + (*ip).pos.y + top_y;
}

/// `it_80274DAC`: resets the item's `xB54` offsets and recomputes its x8 corner.
pub unsafe extern "C" fn it_80274DAC(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xB54.unk_x = 0.0;
    (*ip).xB54.unk_y = 0.0;
    let cd = &raw mut (*ip).x378_itemColl;
    let right_x = (*cd).ecb.right.x;
    let right_y = (*cd).ecb.right.y;
    let left_x = (*cd).ecb.left.x;
    let top_y = (*cd).ecb.top.y;
    (*ip).xB54.x8.y = right_x + (*ip).pos.x;
    (*ip).xB54.x8.x = left_x + (*ip).pos.x;
    (*ip).xB54.x8.z = right_y + (*ip).pos.y + top_y;
}

/// `it_80274DFC`: scales the item's offset and grab-range vectors by its scale.
pub unsafe extern "C" fn it_80274DFC(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xBCC_unk.x *= (*item).scl;
    (*item).xBCC_unk.y *= (*item).scl;
    (*item).xBD4_grabRange.x *= (*item).scl;
    (*item).xBD4_grabRange.y *= (*item).scl;
}

/// `it_80274E44`: scales the item's two ECB-derived boxes by its scale.
pub unsafe extern "C" fn it_80274E44(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let s = (*item).scl;
    (*item).xBEC.top *= s;
    (*item).xBEC.bottom *= s;
    (*item).xBEC.right *= s;
    (*item).xBEC.left *= s;
    (*item).xBDC.top *= s;
    (*item).xBDC.bottom *= s;
    (*item).xBDC.right *= s;
    (*item).xBDC.left *= s;
}

/// `it_80274ECC`: sets the item's `xD09` flag.
pub unsafe extern "C" fn it_80274ECC(item_gobj: *mut HSD_GObj, arg1: bool) {
    (*item_of(item_gobj)).xD09 = arg1 as _;
}

/// `it_80274ED8`: counts one item spawn.
pub unsafe extern "C" fn it_80274ED8() {
    ext::it_804D6D0C += 1;
}

/// `it_80274EE8`: counts one item event.
pub unsafe extern "C" fn it_80274EE8() {
    ext::it_804D6D08 += 1;
}

/// `it_80274EF8`: clears `xDC8` bit 2.
pub unsafe extern "C" fn it_80274EF8(item_gobj: *mut HSD_GObj) {
    (*item_of(item_gobj)).xDC8_word.flags.set_x2(0);
}

/// `it_80274F10`: sets `xDC8` bit 2.
pub unsafe extern "C" fn it_80274F10(item_gobj: *mut HSD_GObj) {
    (*item_of(item_gobj)).xDC8_word.flags.set_x2(1);
}

/// `it_80274F28`: records the grab callbacks and the victim-hold `arg1` on the item.
pub unsafe extern "C" fn it_80274F28(item: *mut Item, arg1: i8, arg2: HSD_GObjEvent, arg3: HSD_GObjInteraction) {
    (*item).xDD0_flag.anon_0.set_b5(1);
    (*item).xD08 = arg1 as _;
    (*item).grab_dealt = arg2;
    (*item).grabbed_for_victim = arg3;
}

/// `get_bone_by_id` (static inline): the jobj of bone `bone_id` (the dynamic bone table, else the child walk).
unsafe fn get_bone_by_id(item_gobj: *mut HSD_GObj, bone_id: i32) -> *mut HSD_JObj {
    let item = item_of(item_gobj);
    let mut jobj = jobj_of(item_gobj);
    if (*(*(*item).xC4_article_data).x10_modelDesc).x4_bone_count != 0 {
        jobj = (*(*item).xBBC_dynamicBoneTable).bones[bone_id as usize];
    } else if bone_id != 0 {
        let mut b = bone_id;
        while b > 0 {
            b -= 1;
            jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
        }
    }
    jobj
}

/// `it_80274F48`: attaches bone `bone_id`'s jobj to the fighter part (constraint `lb_8000C2F8`).
pub unsafe extern "C" fn it_80274F48(item_gobj: *mut HSD_GObj, bone_id: i32, arg2_gobj: *mut HSD_GObj, part_idx: Fighter_Part) {
    let jobj = get_bone_by_id(item_gobj, bone_id);
    ext::lb_8000C2F8(jobj, ext::ftLib_80086630(arg2_gobj, part_idx));
}

/// `it_80274FDC`: as `it_80274F48` with the other constraint (`lb_8000C1C0`).
pub unsafe extern "C" fn it_80274FDC(item_gobj: *mut HSD_GObj, bone_id: i32, arg2_gobj: *mut HSD_GObj, part_idx: Fighter_Part) {
    let jobj = get_bone_by_id(item_gobj, bone_id);
    ext::lb_8000C1C0(jobj, ext::ftLib_80086630(arg2_gobj, part_idx));
}

/// `it_80275070`: releases bone `bone_id`'s constraint.
pub unsafe extern "C" fn it_80275070(item_gobj: *mut HSD_GObj, bone_id: i32) {
    let jobj = get_bone_by_id(item_gobj, bone_id);
    ext::lb_8000C390(jobj);
}

/// `it_802750E8`: true when any bit of `mask` is set in `xDC0`.
pub unsafe extern "C" fn it_802750E8(item_gobj: *mut HSD_GObj, mask: i32) -> bool {
    ((*item_of(item_gobj)).xDC0 as i32) & mask != 0
}

/// `it_802750F8`: re-derives the item's model and state with the `xDCC` bit 3 held clear.
pub unsafe extern "C" fn it_802750F8(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xDCC_flag.set_b3(0);
    ext::Item_802697D4(item_gobj);
    ext::Item_80269978(item_gobj);
    (*item).xDCC_flag.set_b3(1);
}

/// `it_80275288`: sound `arg2 + rand(arg1)` through `Item_8026AE84`.
pub unsafe extern "C" fn it_80275288(item_gobj: *mut HSD_GObj, arg1: i32, arg2: i32) {
    let item = item_of(item_gobj);
    let val = arg2 + ext::HSD_Randi(arg1);
    ext::Item_8026AE84(item, val, 127, 64);
}

/// `it_802752D8`: sound `arg2 + rand(arg1)` through `Item_8026AFA0`.
pub unsafe extern "C" fn it_802752D8(item_gobj: *mut HSD_GObj, arg1: i32, arg2: i32) {
    let item = item_of(item_gobj);
    let val = arg2 + ext::HSD_Randi(arg1);
    ext::Item_8026AFA0(item, val, 127, 64);
}

/// `it_802753BC`: clears the item's `xC94`, `xCA0`, `xCA4` and stores `arg1` in `xC9C`.
pub unsafe extern "C" fn it_802753BC(item_gobj: *mut HSD_GObj, arg1: i16) {
    let item = item_of(item_gobj);
    (*item).xC94 = 0;
    (*item).xC9C = arg1 as _;
    (*item).xCA0 = 0;
    (*item).xCA4 = 0;
}

/// `it_802728C8`: shows the item's child jobj on the frames where its life timer is a multiple of `x44_float`.
pub unsafe extern "C" fn it_802728C8(item_gobj: *mut HSD_GObj) {
    let item_jobj1 = jobj_of(item_gobj);
    let item_jobj2 = if item_jobj1.is_null() { null_mut() } else { (*item_jobj1).child };
    let d = (*ext::it_804D6D28).x44_float as u32;
    let life = (*item_of(item_gobj)).xD44_lifeTimer as i32 as u32;
    let rem = if d == 0 { 0 } else { life % d };
    if rem != 0 {
        ext::HSD_JObjClearFlagsAll(item_jobj2, JOBJ_HIDDEN);
        return;
    }
    ext::HSD_JObjSetFlagsAll(item_jobj2, JOBJ_HIDDEN);
}

/// `it_80272940`: shows the item's child jobj.
pub unsafe extern "C" fn it_80272940(item_gobj: *mut HSD_GObj) {
    let item_jobj1 = jobj_of(item_gobj);
    let item_jobj2 = if item_jobj1.is_null() { null_mut() } else { (*item_jobj1).child };
    ext::HSD_JObjClearFlagsAll(item_jobj2, JOBJ_HIDDEN);
}

/// `it_80272980`: sets the facing direction from the x velocity and passes it to the collision.
pub unsafe extern "C" fn it_80272980(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    let mut vel_x = (*item).x40_vel.x;
    if vel_x < 0.0 {
        vel_x = -vel_x;
    }
    if !(vel_x < 0.00001f32) || (*item).facing_dir == 0.0 {
        let dir = if (*item).x40_vel.x >= 0.0 { 1.0f32 } else { -1.0f32 };
        (*item).facing_dir = dir;
    }
    let int_dir = if (*item).facing_dir == -1.0 { -1 } else { 1 };
    ext::mpCollSetFacingDir(&raw mut (*item).x378_itemColl, int_dir);
}

/// `it_80272A18`: shows the jobj (clears its hidden flag).
pub unsafe extern "C" fn it_80272A18(item_jobj: *mut HSD_JObj) {
    ext::HSD_JObjClearFlagsAll(item_jobj, JOBJ_HIDDEN);
}

/// `it_80272A3C`: hides the jobj.
pub unsafe extern "C" fn it_80272A3C(item_jobj: *mut HSD_JObj) {
    ext::HSD_JObjSetFlagsAll(item_jobj, JOBJ_HIDDEN);
}

/// `it_80272A60`: the item's effect 0x40E and sound 0x74, then the flag reset.
pub unsafe extern "C" fn it_80272A60(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::efSync_Spawn(0x40E, item_gobj, &raw mut (*item).pos);
    ext::Item_8026AE84(item, 0x74, 0x7F, 0x40);
    it_80274C60(item_gobj);
}

/// `it_80272AC4`: effect 0x40C at `arg1` offset from the item, then the sound and the flag reset.
pub unsafe extern "C" fn it_80272AC4(item_gobj: *mut HSD_GObj, arg1: *mut Vec3) {
    let item = item_of(item_gobj);
    ext::lbVector_Add(arg1, &raw mut (*item).pos);
    ext::efSync_Spawn(0x40C, item_gobj, arg1);
    ext::Item_8026AE84(item, 0x74, 0x7F, 0x40);
    it_80274C60(item_gobj);
}

/// `it_80272B40`: effect 0x40C at the item's position, then the sound and the flag reset.
pub unsafe extern "C" fn it_80272B40(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::efSync_Spawn(0x40C, item_gobj, &raw mut (*item).pos);
    ext::Item_8026AE84(item, 0x74, 0x7F, 0x40);
    it_80274C60(item_gobj);
}

/// `it_80272BA4`: effect 0x411 at the item's position, sound 0x73.
pub unsafe extern "C" fn it_80272BA4(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::efSync_Spawn(0x411, item_gobj, &raw mut (*item).pos);
    ext::Item_8026AE84(item, 0x73, 0x7F, 0x40);
    it_80274C60(item_gobj);
}

/// `it_80272C08`: effect 0x410 at the item's position, sound 0x74.
pub unsafe extern "C" fn it_80272C08(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    ext::efSync_Spawn(0x410, item_gobj, &raw mut (*item).pos);
    ext::Item_8026AE84(item, 0x74, 0x7F, 0x40);
    it_80274C60(item_gobj);
}

/// `it_80272C6C`: whether the item's jobj has the `lb_8000B09C` property.
pub unsafe extern "C" fn it_80272C6C(item_gobj: *mut HSD_GObj) -> bool {
    ext::lb_8000B09C(jobj_of(item_gobj))
}

/// `it_80272C90`: the bone jobj for the item's attach id.
pub unsafe extern "C" fn it_80272C90(item_gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let idx = (*(*(*item_of(item_gobj)).xC4_article_data).x10_modelDesc).x8_bone_attach_id;
    it_80272CC0(item_gobj, idx)
}

/// `it_80272CC0`: the jobj `idx` levels below the item's jobj (or the dynamic bone table entry).
pub unsafe extern "C" fn it_80272CC0(item_gobj: *mut HSD_GObj, mut idx: i32) -> *mut HSD_JObj {
    let item = item_of(item_gobj);
    let mut jobj = jobj_of(item_gobj);
    if (*(*(*item).xC4_article_data).x10_modelDesc).x4_bone_count != 0 {
        return (*(*item).xBBC_dynamicBoneTable).bones[idx as usize];
    }
    if idx != 0 {
        while idx > 0 {
            jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
            idx -= 1;
        }
    }
    jobj
}

/// `it_8027518C`: sets the life timer to the common `xF8` value, flags the item (`xDCF` bit 2, `xDD1` bit 0) and
/// clears its `0x1A` flag.
pub unsafe extern "C" fn it_8027518C(item_gobj: *mut HSD_GObj) {
    let item = item_of(item_gobj);
    (*item).xD44_lifeTimer = (*ext::it_804D6D28).xF8 as f32;
    (*item).xDCF_flag.anon_0.set_b2(1);
    (*item).xDD1_flag.anon_0.set_b0(1);
    it_8026BDB4(item_gobj);
}

/// `it_802751D8`: counts the life timer down; true when it has run out.
pub unsafe extern "C" fn it_802751D8(item_gobj: *mut HSD_GObj) -> bool {
    let item = item_of(item_gobj);
    (*item).xD44_lifeTimer -= 1.0;
    (*item).xD44_lifeTimer <= 0.0
}
