//! Port of src/melee/it/kinds/itunknown.c: the Pokemon Stadium "unknown" item pair (logic tables 37/38).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itunknown.rs.
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
    fn Camera_GetTransformInterest(a0: *mut Vec);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_GetBlastZoneBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneBottomOffset() };
    fn Stage_GetBlastZoneLeftOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneLeftOffset() };
    fn Stage_GetBlastZoneRightOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneRightOffset() };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn Stage_GetCamBoundsBottomOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetCamBoundsBottomOffset() };
    fn Stage_GetCamBoundsTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetCamBoundsTopOffset() };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn efLib_PauseAll(a0: *mut HSD_GObj);
    fn efLib_ResumeAll(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80273318(a0: *mut HSD_GObj, a1: *mut HSD_Joint) => unsafe { crate::tu::it_2725::it_80273318(a0, a1) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0, a1, a2) };
    fn lbVector_Rotate(a0: *mut Vec, a1: i32, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_Rotate(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `ItemStateTable it_803F7D60[]`
pub static mut it_803F7D60: [ItemStateTable; 4] = [
    st_row(0, pred_b(itUnknown_UnkMotion0_Anim), Some(itUnknown_UnkMotion0_Phys), pred_b(itUnknown_UnkMotion0_Coll)),
    st_row(1, pred_b(itUnknown_UnkMotion1_Anim), Some(itUnknown_UnkMotion1_Phys), pred_b(itUnknown_UnkMotion1_Coll)),
    st_row(-1, pred_b(itUnknown_UnkMotion2_Anim), Some(itUnknown_UnkMotion2_Phys), pred_b(itUnknown_UnkMotion2_Coll)),
    st_row(0, pred_b(it_802CF120), Some(it_802CF154), pred_b(it_802CF3D8)),
];

/// `ABS(x)` (Runtime/platform.h) for f32.
#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterAirStateWithHitlag` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_EnterAirStateWithHitlag(gobj: *mut Item_GObj, msid: enum_t) {
    let item = GET_ITEM(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `randi_perm` (static inline): `HSD_Randi(f32)`, the float converted to int at the call
#[inline]
unsafe fn randi_perm(f: f32) -> i32 {
    ext::HSD_Randi(f as i32)
}

/// `randi_perm_int` (static inline)
#[inline]
unsafe fn randi_perm_int(i: i32) -> i32 {
    ext::HSD_Randi(i)
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itUnknownAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itUnknownAttributes
}

pub unsafe extern "C" fn it_802CE710(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let r = ext::HSD_Randi(26);
    ext::it_80273318(gobj, (*attr).x24[r as usize]);
    if ext::HSD_Randi(2) != 0 {
        (*ip).xDD4_itemVar.unknown.x60 = 1.0;
    } else {
        (*ip).xDD4_itemVar.unknown.x60 = -1.0;
    }
    (*ip).xDD4_itemVar.unknown.x64.i = 0;
    (*ip).xDD4_itemVar.unknown.x68.i = (*attr).x18.i;
    (*ip).xDCC_flag.set_b3(0);
    (*ip).facing_dir = 0.0;
    ext::it_80279CDC(gobj, (*attr).x0.f);
    it_802CEC24(gobj);
}

pub unsafe extern "C" fn it_802CE7CC(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn it_802CE7D0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itUnknown_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).pos.x > ext::Stage_GetBlastZoneRightOffset()
        || (*ip).pos.x < ext::Stage_GetBlastZoneLeftOffset()
        || (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset()
        || (*ip).pos.y < ext::Stage_GetBlastZoneBottomOffset()
    {
        it_802CE8D0(gobj);
    }
    false
}

pub unsafe extern "C" fn itUnknown_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    ext::it_80272860(gobj, (*attr).xC, (*attr).x10);
    // `vel.x += x4 * x60`: one fused multiply-add (shipping contraction; f32 = f64 product + sum, then f32)
    (*ip).x40_vel.x = ((*attr).x4 as f64 * (*ip).xDD4_itemVar.unknown.x60 as f64 + (*ip).x40_vel.x as f64) as f32;
}

pub unsafe extern "C" fn itUnknown_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn it_802CE8D0(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let mut cam_pos: Vec3 = core::mem::zeroed();
    let mut dir: Vec3 = core::mem::zeroed();

    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    match ext::HSD_Randi(4) {
        0 => {
            (*ip).xDD4_itemVar.unknown.x84 = 0;
            (*ip).xDD4_itemVar.unknown.x6C.x = ext::Stage_GetBlastZoneRightOffset();
            let top = ext::Stage_GetBlastZoneTopOffset();
            let bottom = ext::Stage_GetBlastZoneBottomOffset();
            let r = randi_perm(top - bottom) as f32;
            (*ip).xDD4_itemVar.unknown.x6C.y = r + ext::Stage_GetBlastZoneBottomOffset();
        }
        1 => {
            (*ip).xDD4_itemVar.unknown.x84 = 1;
            (*ip).xDD4_itemVar.unknown.x6C.x = ext::Stage_GetBlastZoneLeftOffset();
            let top = ext::Stage_GetBlastZoneTopOffset();
            let bottom = ext::Stage_GetBlastZoneBottomOffset();
            let r = randi_perm(top - bottom) as f32;
            (*ip).xDD4_itemVar.unknown.x6C.y = r + ext::Stage_GetBlastZoneBottomOffset();
        }
        2 => {
            (*ip).xDD4_itemVar.unknown.x84 = 2;
            let right = ext::Stage_GetBlastZoneRightOffset();
            let left = ext::Stage_GetBlastZoneLeftOffset();
            let r = randi_perm(right - left) as f32;
            (*ip).xDD4_itemVar.unknown.x6C.x = r + ext::Stage_GetBlastZoneLeftOffset();
            (*ip).xDD4_itemVar.unknown.x6C.y = ext::Stage_GetBlastZoneTopOffset();
        }
        _ => {
            (*ip).xDD4_itemVar.unknown.x84 = 3;
            let right = ext::Stage_GetBlastZoneRightOffset();
            let left = ext::Stage_GetBlastZoneLeftOffset();
            let r = randi_perm(right - left) as f32;
            (*ip).xDD4_itemVar.unknown.x6C.x = r + ext::Stage_GetBlastZoneLeftOffset();
            (*ip).xDD4_itemVar.unknown.x6C.y = ext::Stage_GetBlastZoneBottomOffset();
        }
    }
    {
        let zero: f32 = 0.0;
        (*ip).xDD4_itemVar.unknown.x6C.z = zero;
    }
    ext::Camera_GetTransformInterest(&mut cam_pos);
    ext::lbVector_Diff(&mut cam_pos, &mut (*ip).xDD4_itemVar.unknown.x6C, &mut dir);
    (*ip).xDD4_itemVar.unknown.x78.vec.x = (*attr).x14;
    (*ip).xDD4_itemVar.unknown.x78.vec.y = 0.0;
    (*ip).xDD4_itemVar.unknown.x78.vec.z = 0.0;
    let ang = ext::atan2f(dir.y, dir.x);
    ext::lbVector_Rotate(&mut (*ip).xDD4_itemVar.unknown.x78.vec, 4, ang);
    ext::it_80272980(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    if ext::HSD_Randi(2) != 0 {
        ext::Item_8026AE84(ip, 0x271A, 127, 64);
    } else {
        ext::Item_8026AE84(ip, 0x271B, 127, 64);
    }
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itUnknown_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.unknown.x64.i -= 1;
    if (*ip).xDD4_itemVar.unknown.x64.i < 0 {
        it_802CED54(gobj);
        (*ip).xDD4_itemVar.unknown.x68.i -= 1;
        if (*ip).xDD4_itemVar.unknown.x68.i == 0 {
            return true;
        }
        let attr = attrs_of(ip);
        let r = ext::HSD_Randi((*attr).x1C.i);
        (*ip).xDD4_itemVar.unknown.x64.i = (*attr).x20.i + r;
    }
    false
}

pub unsafe extern "C" fn itUnknown_UnkMotion1_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itUnknown_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn it_802CEC24(gobj: *mut Item_GObj) {
    Item_EnterAirStateWithHitlag(gobj, 2);
}

pub unsafe extern "C" fn itUnknown_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itUnknown_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    if ext::it_8027A09C(gobj) != 0 {
        ext::it_80273454(gobj);
        {
            let ip = GET_ITEM(gobj);
            let attr = attrs_of(ip);
            (*ip).x40_vel.y = (*attr).x8;
            ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
            Item_SetEffectHitlagCallbacks(ip);
        }
    }
}

pub unsafe extern "C" fn itUnknown_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802CE7CC)) != 0
}

pub unsafe extern "C" fn it_802CED54(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let mut spawn: SpawnItem = core::mem::zeroed();
    let bottom = ext::Stage_GetCamBoundsBottomOffset();
    let range = (ext::Stage_GetCamBoundsTopOffset() - bottom) / 3.0;

    match (*ip).xDD4_itemVar.unknown.x84 {
        0 | 1 => {
            spawn.prev_pos.x = (*ip).xDD4_itemVar.unknown.x6C.x;
            let r = randi_perm(range) as f32;
            spawn.prev_pos.y = (*ip).xDD4_itemVar.unknown.x6C.y + (r - range / 2.0);
        }
        2 | 3 => {
            let r = randi_perm(range) as f32;
            spawn.prev_pos.x = (*ip).xDD4_itemVar.unknown.x6C.x + (r - range / 2.0);
            spawn.prev_pos.y = (*ip).xDD4_itemVar.unknown.x6C.y;
        }
        _ => {}
    }
    spawn.prev_pos.z = (*ip).xDD4_itemVar.unknown.x6C.z;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = (*ip).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel = (*ip).xDD4_itemVar.unknown.x78.vec;
    spawn.x40 = 0;
    spawn.kind = It_Kind_Unknown_Swarm;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(0);
    ext::Item_80268B18(&mut spawn);
}

pub unsafe extern "C" fn it_2725_Logic38_Spawned(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    let r = randi_perm_int(26);
    ext::it_80273318(gobj, (*attr).x24[r as usize]);
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xD44_lifeTimer = (*attr).x0.i as f32;
    ext::it_80274740(gobj);

    if ABS((*ip).x40_vel.x) > ABS((*ip).x40_vel.y) {
        (*ip).x40_vel.y *= (*attr).x1C.f;
        (*ip).xDD4_itemVar.unknown.x78.dir = 0;
    } else {
        (*ip).x40_vel.x *= (*attr).x1C.f;
        (*ip).xDD4_itemVar.unknown.x78.dir = 1;
    }
    (*ip).xDD4_itemVar.unknown.x60 = 0.0;
    (*ip).xDD4_itemVar.unknown.x64.f = 0.0;
    (*ip).xDD4_itemVar.unknown.x68.f = 0.0;
    let r = randi_perm_int(((*attr).x4 + (*attr).x8) as i32);
    (*ip).xDD4_itemVar.unknown.x6C.x = r as f32 - (*attr).x8;
    let r = randi_perm_int(((*attr).xC + (*attr).x10) as i32);
    (*ip).xDD4_itemVar.unknown.x6C.y = r as f32 - (*attr).x10;
    let r = randi_perm_int(((*attr).x14 + (*attr).x18.f) as i32);
    (*ip).xDD4_itemVar.unknown.x6C.z = r as f32 - (*attr).x18.f;
    it_802CF0D4(gobj);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn itUnknown_Logic38_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802CF0D4(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn it_802CF120(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn it_802CF154(gobj: *mut Item_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    (*ip).xDD4_itemVar.unknown.x60 += (*ip).xDD4_itemVar.unknown.x6C.x;
    (*ip).xDD4_itemVar.unknown.x64.f += (*ip).xDD4_itemVar.unknown.x6C.y;
    (*ip).xDD4_itemVar.unknown.x68.f += (*ip).xDD4_itemVar.unknown.x6C.z;
    HSD_JObjSetRotationX(jobj, 0.017453292f32 * (*ip).xDD4_itemVar.unknown.x60);
    HSD_JObjSetRotationY(jobj, 0.017453292f32 * (*ip).xDD4_itemVar.unknown.x64.f);
    HSD_JObjSetRotationZ(jobj, 0.017453292f32 * (*ip).xDD4_itemVar.unknown.x68.f);
    if (*ip).xDD4_itemVar.unknown.x78.dir != 0 {
        (*ip).x40_vel.x *= (*attr).x20.f;
    } else {
        (*ip).x40_vel.y *= (*attr).x20.f;
    }
}

pub unsafe extern "C" fn it_802CF3D8(_gobj: *mut Item_GObj) -> bool {
    false
}
