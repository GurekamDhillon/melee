//! Port of src/melee/it/kinds/itpikachuthunder.c: Pikachu's Down-B thunder bolt (logic table 39), a
//! chain of bolts linked through `x34`; `x0 == 0` is the head. Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itpikachuthunder.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::cabi::f2i32;
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE60() -> u32;
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftPk_SpecialLw_80127608(a0: *mut HSD_GObj);
    fn ftPk_SpecialLw_CheckProperty(a0: *mut HSD_GObj) -> i32;
    fn ftPk_SpecialLw_SetState_Unk0(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB20(a0: *mut HSD_GObj);
    fn it_8026BB44(a0: *mut HSD_GObj);
    fn it_8026DA70(a0: *mut HSD_GObj) -> i32;
    fn it_80272AC4(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_80275594(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn it_802756D0(a0: *mut HSD_GObj);
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

/// `HSD_JObjSetScaleY` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetScaleY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).scale.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `ItemStateTable it_803F70C8[]`
pub static mut it_803F70C8: [ItemStateTable; 3] = [
    st_row(-1, pred_b(itPikachuthunder_UnkMotion0_Anim), None, None),
    st_row(0, pred_b(itPikachuthunder_UnkMotion1_Anim), None, pred_b(itPikachuthunder_UnkMotion1_Coll)),
    st_row(0, pred_b(itPikachuthunder_UnkMotion2_Anim), None, None),
];

/// 2B1DEC
pub unsafe extern "C" fn it_802B1DEC(arg0: *mut Item_GObj) -> i32 {
    (*GET_ITEM(arg0)).xDD4_itemVar.pikachuthunder.x4
}

/// 2B1DF8
pub unsafe extern "C" fn it_802B1DF8(owner: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, count: i32, delay: i32, kind: i32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let x40: u32 = ext::Item_8026AE60();
    let mut cur_delay: i32;
    let mut prev: *mut Item_GObj = null_mut();

    spawn.kind = kind;
    // Item_InitSpawnPosition(&spawn, pos, true)
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    spawn.pos = spawn.prev_pos;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.facing_dir = -1.0;
    cur_delay = 0;
    spawn.x3C_damage = cur_delay as i16;
    cur_delay = 0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = x40 as i32;

    let mut first: *mut Item_GObj = null_mut();
    let mut i: i32 = 0;
    while i < count {
        let item_gobj = ext::Item_80268B18(&mut spawn);
        if i == 0 {
            first = item_gobj;
        }
        if !item_gobj.is_null() {
            let ip = HSD_GObjGetUserData(item_gobj) as *mut Item;
            let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
            if !prev.is_null() {
                (*GET_ITEM(prev)).xDD4_itemVar.pikachuthunder.x34 = item_gobj;
            }
            ext::it_802756D0(item_gobj);
            (*ip).xDC8_word.flags.set_xD(0);
            ext::db_80225DD8(item_gobj, owner);
            it_802B2080(item_gobj);
            (*ip).xDCC_flag.set_b3(0);
            (*ip).xDCC_flag.set_b4567(14);
            (*ip).xDD4_itemVar.pikachuthunder.x0 = i;
            (*ip).xDD4_itemVar.pikachuthunder.x38 = owner;
            (*ip).xDD4_itemVar.pikachuthunder.x8 = cur_delay;
            // ABS(attrs->x4 / vel->y)
            let q = (*attrs).x4 / (*vel).y;
            (*ip).xDD4_itemVar.pikachuthunder.x14 = if q < 0.0 { -q } else { q };
            (*ip).xDD4_itemVar.pikachuthunder.x1C = *vel;
            (*ip).xDD4_itemVar.pikachuthunder.x34 = null_mut();
            (*ip).xDD4_itemVar.pikachuthunder.x4 = 0;
            (*ip).xDD4_itemVar.pikachuthunder.xC = (*attrs).x4;
            (*ip).xDD4_itemVar.pikachuthunder.x18 = 1.0;
        }
        prev = item_gobj;
        cur_delay = cur_delay.wrapping_add(delay);
        i += 1;
    }
    first
}

/// 2B1FC8
pub unsafe extern "C" fn it_802B1FC8(gobj: *mut Item_GObj) {
    it_802B22B8(gobj);
}

/// 2B1FE8
pub unsafe extern "C" fn it_802B1FE8(gobj: *mut Item_GObj, pos: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    let scale = (*ip).xDD4_itemVar.pikachuthunder.x18;
    let mut offset = -(*attrs).x8;

    offset *= scale;
    *pos = (*ip).pos;
    (*pos).y += offset;
}

pub unsafe extern "C" fn it_2725_Logic39_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachuthunder.x0 == 0
        && !(*ip).xDD4_itemVar.pikachuthunder.x38.is_null()
        && ext::ftPk_SpecialLw_CheckProperty((*ip).xDD4_itemVar.pikachuthunder.x38) == 0
    {
        ext::ftPk_SpecialLw_SetState_Unk0((*ip).xDD4_itemVar.pikachuthunder.x38);
    }
}

/// 2B2080
pub unsafe extern "C" fn it_802B2080(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
    ext::it_8026BB44(gobj);
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachuthunder.x8 <= 0 {
        it_802B211C(gobj);
    } else {
        (*ip).xDD4_itemVar.pikachuthunder.x8 = (*ip).xDD4_itemVar.pikachuthunder.x8.wrapping_sub(1);
    }
    false
}

/// 2B211C
pub unsafe extern "C" fn it_802B211C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).x40_vel = (*ip).xDD4_itemVar.pikachuthunder.x1C;
    (*ip).xDCC_flag.set_b3(1);
    ext::it_8026BB20(gobj);
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = (*gobj).user_data as *mut Item;
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachuthunder.x0 == 0 {
        if ext::it_8026DA70(gobj) != 0 {
            let ip = (*gobj).user_data as *mut Item;
            let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
            let mut pos: Vec3 = core::mem::zeroed();
            pos.z = 0.0;
            pos.x = 0.0;
            pos.y = -(*attrs).x8;
            ext::it_80272AC4(gobj, &mut pos);
            it_802B22B8(gobj);
            if !(*ip).xDD4_itemVar.pikachuthunder.x38.is_null() {
                if ext::ftPk_SpecialLw_CheckProperty((*ip).xDD4_itemVar.pikachuthunder.x38) == 0 {
                    ext::ftPk_SpecialLw_80127608((*ip).xDD4_itemVar.pikachuthunder.x38);
                }
            }
        }
    } else {
        let ip = (*gobj).user_data as *mut Item;
        if (*ip).xDD4_itemVar.pikachuthunder.x4 != 0 {
            if (*ip).pos.y <= (*ip).xDD4_itemVar.pikachuthunder.x28.y {
                it_802B22B8(gobj);
            }
        }
    }
    false
}

/// `static inline it_802B22B8_inline`
#[inline]
unsafe fn it_802B22B8_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).xDD4_itemVar.pikachuthunder.x34.is_null() {
        let child_ip = GET_ITEM((*ip).xDD4_itemVar.pikachuthunder.x34);
        if !child_ip.is_null() {
            (*child_ip).xDD4_itemVar.pikachuthunder.x28 = (*ip).pos;
            (*child_ip).xDD4_itemVar.pikachuthunder.x4 = 1;
        }
    }
}

/// 2B22B8
pub unsafe extern "C" fn it_802B22B8(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1 | ITEM_HIT_PRESERVE);
    (*ip).xD44_lifeTimer = (*ip).xDD4_itemVar.pikachuthunder.x14;
    (*ip).xDD4_itemVar.pikachuthunder.x4 = 1;
    it_802B22B8_inline(gobj);
}

pub unsafe extern "C" fn pika_scale(a: f32, b: f32) -> f32 {
    let mut f: f32 = 10000.0 * a / b;
    f = (f2i32((f + 1.0) as f64) as f32) / 10000.0;
    f
}

#[inline]
unsafe fn itPikachuthunder_UnkMotion2_ScaleCall(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_80275594(
        gobj,
        0,
        pika_scale((*ip).xDD4_itemVar.pikachuthunder.x10, (*ip).xDD4_itemVar.pikachuthunder.xC),
    );
}

#[inline]
unsafe fn itPikachuthunder_UnkMotion2_UpdateScale(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    (*ip).xDD4_itemVar.pikachuthunder.x10 = (*ip).xDD4_itemVar.pikachuthunder.xC + (*ip).xDD4_itemVar.pikachuthunder.x1C.y;
    if (*ip).xDD4_itemVar.pikachuthunder.x10 > 0.0 {
        (*ip).xDD4_itemVar.pikachuthunder.x18 = pika_scale((*ip).xDD4_itemVar.pikachuthunder.x10, (*attrs).x4);
        if (*ip).xDD4_itemVar.pikachuthunder.x18 <= 0.0 {
            (*ip).xDD4_itemVar.pikachuthunder.x18 = 0.01;
        }
        itPikachuthunder_UnkMotion2_ScaleCall(gobj);
        (*ip).xDD4_itemVar.pikachuthunder.xC = (*ip).xDD4_itemVar.pikachuthunder.x10;
    }
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);

    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    itPikachuthunder_UnkMotion2_UpdateScale(gobj);
    HSD_JObjSetScaleY(jobj, (*ip).xDD4_itemVar.pikachuthunder.x18);
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_HitShield(_arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_Clanked(_arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_Absorbed(_arg0: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachuthunder.x38 == ref_gobj {
        (*ip).xDD4_itemVar.pikachuthunder.x38 = null_mut();
    }
    ext::it_8026B894(gobj, ref_gobj);
}
