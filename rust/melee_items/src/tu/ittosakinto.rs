//! Port of src/melee/it/kinds/ittosakinto.c: Goldeen (Pokeball Pokemon, logic table 0). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_ittosakinto.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn Stage_GetBlastZoneBottomOffset() -> f32;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32;
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273454(a0: *mut HSD_GObj);
    fn it_802762BC(a0: *mut Item);
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32);
    fn it_80279FF8(a0: *mut HSD_GObj);
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32;
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32;
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item);
    fn it_8027A344(a0: *mut HSD_GObj);
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32);
    fn lbAudioAx_800263E8(a0: f32, a1: *mut HSD_GObj, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, a8: i32, a9: i32, a10: i32) -> *mut HSD_GObj;
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

/// `SFX_NONE` (lb/lbaudio_ax.h)
const SFX_NONE: i32 = -1;

/// `ItemStateTable it_803F7A48[]`
pub static mut it_803F7A48: [ItemStateTable; 5] = [
    st_row(0, pred_b(itTosakinto_UnkMotion2_Anim), Some(itTosakinto_UnkMotion2_Phys), pred_b(itTosakinto_UnkMotion2_Coll)),
    st_row(1, pred_b(itTosakinto_UnkMotion2_Anim), Some(itTosakinto_UnkMotion2_Phys), pred_b(itTosakinto_UnkMotion2_Coll)),
    st_row(2, pred_b(itTosakinto_UnkMotion2_Anim), Some(itTosakinto_UnkMotion2_Phys), pred_b(itTosakinto_UnkMotion2_Coll)),
    st_row(-1, None, Some(itTosakinto_UnkMotion3_Phys), pred_b(itTosakinto_UnkMotion3_Coll)),
    st_row(-1, pred_b(itTosakinto_UnkMotion4_Anim), Some(itTosakinto_UnkMotion4_Phys), pred_b(itTosakinto_UnkMotion4_Coll)),
];

/// `Item_TickLifetime` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

/// `Item_SetEffectHitlagCallbacks` is not used here; `GET_ATTRS(it)`
#[inline]
unsafe fn GET_ATTRS(it: *mut Item) -> *mut itTosakinto_Attrs {
    (*(*it).xC4_article_data).x4_specialAttributes as *mut itTosakinto_Attrs
}

pub unsafe extern "C" fn it_802C8F4C(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    let attrs = GET_ATTRS(it);
    (*it).xDD4_itemVar.tosakinto.x60_msid = 0;
    (*it).xD44_lifeTimer = (*attrs).x4_life_timer;
    (*it).xDD4_itemVar.tosakinto.x68_sfx_id = SFX_NONE;
    ext::it_80279CDC(gobj, (*attrs).x0_scale);
    ext::Item_8026AE84(it, 0x272E, 0x7F, 0x40);
    it_802C93BC(gobj);
}

pub unsafe extern "C" fn it_802C8FC4(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802C8FE4(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    let mut scale_vec: Vec3 = core::mem::zeroed();

    let jobj = GET_JOBJ(gobj);
    let child = HSD_JObjGetChild(jobj);
    let scale = (*(*it).xCC_item_attr).x60_scale;
    scale_vec.z = scale;
    scale_vec.y = scale;
    scale_vec.x = scale;
    HSD_JObjSetScale(child, &mut scale_vec);
    ext::it_80273454(gobj);
    it_802C90E8(gobj);
    (*it).xDD4_itemVar.tosakinto.x68_sfx_id = (*it).xD6C as _;
}

pub unsafe extern "C" fn it_802C90E8(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    let attrs = GET_ATTRS(it);

    ext::it_8027ADEC(0x46E, gobj, GET_JOBJ(gobj), 2.0);
    ext::it_802762BC(it);
    let mut msid = (*it).xDD4_itemVar.tosakinto.x60_msid;
    if msid >= 3 {
        msid = 0;
    }
    (*it).xDD4_itemVar.tosakinto.x60_msid = msid;
    ext::Item_80268E5C(gobj, (*it).xDD4_itemVar.tosakinto.x60_msid, ITEM_ANIM_UPDATE);
    (*it).xDD4_itemVar.tosakinto.x64_vel_x = (*attrs).x8_vel_x;
    if ext::HSD_Randi(2) != 0 {
        (*it).xDD4_itemVar.tosakinto.x64_vel_x = -(*it).xDD4_itemVar.tosakinto.x64_vel_x;
    }
}

pub unsafe extern "C" fn itTosakinto_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::it_8027A160(HSD_JObjGetChild(jobj), it);
    Item_TickLifetime(it)
}

pub unsafe extern "C" fn itTosakinto_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    ext::it_8027A344(gobj);
    (*it).x40_vel.x += (*it).xDD4_itemVar.tosakinto.x64_vel_x;
}

pub unsafe extern "C" fn itTosakinto_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);

    let temp = ext::it_8026DA08(gobj) != 0;
    if ext::it_80272C6C(gobj) == 0 {
        if temp {
            (*it).xDD4_itemVar.tosakinto.x60_msid += 1;
            it_802C90E8(gobj);
        } else {
            (*it).xDD4_itemVar.tosakinto.x60_msid += 1;
            it_802C9468(gobj);
        }
    } else if temp {
        (*it).xDD4_itemVar.tosakinto.x60_msid += 1;
        it_802C90E8(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802C93BC(gobj: *mut Item_GObj) {
    ext::it_802762BC(GET_ITEM(gobj));
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTosakinto_UnkMotion4_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itTosakinto_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itTosakinto_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8027A118(gobj, Some(it_802C8FE4)) != 0
}

pub unsafe extern "C" fn it_802C9468(gobj: *mut Item_GObj) {
    ext::it_802762BC(GET_ITEM(gobj));
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTosakinto_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTosakinto_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(it_802C90E8));
    false
}

pub unsafe extern "C" fn itTosakinto_Logic0_Destroyed(gobj: *mut Item_GObj) {
    let it_r31 = GET_ITEM(gobj);
    if (*it_r31).pos.y - 50.0 < ext::Stage_GetBlastZoneBottomOffset() {
        ext::lbAudioAx_800263E8(3.0, gobj, 9, 0x272E, 0x7F, 0x7F, 0x7F, 0, 0x3C, 0, (*it_r31).xDD4_itemVar.tosakinto.x68_sfx_id);
    }
}
