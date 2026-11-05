//! Port of src/melee/it/kinds/itpikachuthunder.c: Pikachu's Down-B thunder bolt (logic table 39), a
//! chain of bolts linked through `x34`; `x0 == 0` is the head. Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itpikachuthunder.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itPikachuthunderAttributes;
#[allow(unused_imports)]
use crate::layout::itRshellAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itThunderPokemonAttributes;
#[allow(unused_imports)]
use crate::layout::itUnknownAttributes;
#[allow(unused_imports)]
use crate::layout::ItLGunAttr;
#[allow(unused_imports)]
use crate::layout::MaximTomatoSpecialAttr;
#[allow(unused_imports)]
use crate::layout::ScopeBeamAttrs;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
#[allow(unused_imports)]
use crate::itkinds5::{sa};
#[allow(unused_imports)]
use crate::helpers::pika_scale as pika_scale_itkinds14;
#[allow(unused_imports)]
use crate::itkinds11::it_80272cc0;
#[allow(unused_imports)]
use crate::itkinds13::swap_velocity;
#[allow(unused_imports)]
use crate::itkinds5::it_802ac5d8;
#[allow(unused_imports)]
use crate::itkinds5::it_802b3368;
#[allow(unused_imports)]
use crate::itkinds8::get_min_life;
#[allow(unused_imports)]
use crate::itkinds9::my_sqrtf_accurate;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_80275594;
#[allow(unused_imports)]
use crate::itlib3::it_80272828;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::layout::Article;
#[allow(unused_imports)]
use crate::layout::CommandInfo;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HSD_JObj;
#[allow(unused_imports)]
use crate::layout::HSD_Joint;
#[allow(unused_imports)]
use crate::layout::ItemStateTable;
#[allow(unused_imports)]
use crate::layout::it_8026C47C_arg0_t;
#[allow(unused_imports)]
use crate::layout::itBombHeiAttributes;
#[allow(unused_imports)]
use crate::layout::itClimbersStringAttributes;
#[allow(unused_imports)]
use crate::layout::itLikelikeAttributes;
#[allow(unused_imports)]
use crate::layout::itNessPKFirepillarAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusBombAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::ItemKind;
#[allow(unused_imports)]
use crate::layout::It_Kind_Capsule;
#[allow(unused_imports)]
use crate::layout::It_Kind_RabbitC;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_core::vec::Vec3 as RVec;
#[allow(unused_imports)]
use melee_runtime::lbvector;
#[allow(unused_imports)]
use crate::itkinds14::{item_of};
#[allow(unused_imports)]
use crate::layout::ftCommonData;
#[allow(unused_imports)]
use crate::layout::itECB;
#[allow(unused_imports)]
use crate::layout::itSeakChain_Attrs;
#[allow(unused_imports)]
use crate::layout::Collide_CeilingHug;
#[allow(unused_imports)]
use crate::layout::Collide_CeilingPush;
#[allow(unused_imports)]
use crate::layout::GroundOrAir;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::cabi::f2i32;
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
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE60() -> u32 => unsafe { crate::tu::item::Item_8026AE60() };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftPk_SpecialLw_80127608(a0: *mut HSD_GObj);
    fn ftPk_SpecialLw_CheckProperty(a0: *mut HSD_GObj) -> i32;
    fn ftPk_SpecialLw_SetState_Unk0(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB20(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB20(a0) };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026DA70(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA70(a0) };
    fn it_80272AC4(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_80272AC4(a0, a1 as _) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80275594(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::itlib::it_80275594(&mut *((*a0).user_data as *mut Item), a1, a2) };
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

/// `ItemStateTable it_803F70C8[]`
pub static mut it_803F70C8: [ItemStateTable; 3] = [
    st_row(-1, pred_b!(itPikachuthunder_UnkMotion0_Anim), None, None),
    st_row(0, pred_b!(itPikachuthunder_UnkMotion1_Anim), None, pred_b!(itPikachuthunder_UnkMotion1_Coll)),
    st_row(0, pred_b!(itPikachuthunder_UnkMotion2_Anim), None, None),
];

/// 2B1DEC
pub unsafe extern "C" fn it_802B1DEC(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802B1DEC");
    (*GET_ITEM(arg0)).xDD4_itemVar.pikachuthunder.x4
}

/// 2B1DF8
pub unsafe extern "C" fn it_802B1DF8(owner: *mut Item_GObj, pos: *mut Vec3, vel: *mut Vec3, count: i32, delay: i32, kind: i32) -> *mut Item_GObj {
    melee_core::ct!("it_802B1DF8");
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
    melee_core::ct!("it_802B1FC8");
    it_802B22B8(gobj);
}

/// 2B1FE8
pub unsafe extern "C" fn it_802B1FE8(gobj: *mut Item_GObj, pos: *mut Vec3) {
    melee_core::ct!("it_802B1FE8");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    let scale = (*ip).xDD4_itemVar.pikachuthunder.x18;
    let mut offset = -(*attrs).x8;

    offset *= scale;
    *pos = (*ip).pos;
    (*pos).y += offset;
}

pub unsafe extern "C" fn it_2725_Logic39_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic39_Destroyed");
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
    melee_core::ct!("it_802B2080");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
    ext::it_8026BB44(gobj);
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuthunder_UnkMotion0_Anim");
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
    melee_core::ct!("it_802B211C");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPikachuthunderAttributes;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).x40_vel = (*ip).xDD4_itemVar.pikachuthunder.x1C;
    (*ip).xDCC_flag.set_b3(1);
    ext::it_8026BB20(gobj);
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuthunder_UnkMotion1_Anim");
    let ip = (*gobj).user_data as *mut Item;
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPikachuthunder_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuthunder_UnkMotion1_Coll");
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
    melee_core::ct!("it_802B22B8");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1 | ITEM_HIT_PRESERVE);
    (*ip).xD44_lifeTimer = (*ip).xDD4_itemVar.pikachuthunder.x14;
    (*ip).xDD4_itemVar.pikachuthunder.x4 = 1;
    it_802B22B8_inline(gobj);
}

pub unsafe extern "C" fn pika_scale(a: f32, b: f32) -> f32 {
    melee_core::ct!("pika_scale");
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
    melee_core::ct!("itPikachuthunder_UnkMotion2_Anim");
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
    melee_core::ct!("itPikachuThunder_Logic39_DmgDealt");
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_HitShield(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuThunder_Logic39_HitShield");
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_Clanked(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuThunder_Logic39_Clanked");
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_Absorbed(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itPikachuThunder_Logic39_Absorbed");
    false
}

pub unsafe extern "C" fn itPikachuThunder_Logic39_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itPikachuThunder_Logic39_EvtUnk");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.pikachuthunder.x38 == ref_gobj {
        (*ip).xDD4_itemVar.pikachuthunder.x38 = null_mut();
    }
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from helpers.rs (consolidation steps 21/22) ----
/// `pika_scale_fp` (itpikachuthunder.c, static): `(s32)` is gwtool's saturating conversion
/// (NaN -> 0), which is what Rust's `as i32` does.
pub fn pika_scale_fp(a: f32, b: f32) -> f32 {
    melee_core::ct!("pika_scale_fp");
    let f = 10000.0f32 * a / b;
    (melee_core::ppc::fctiwz_f32(f + 1.0)) as f32 / 10000.0
}

// ---- folded from itkinds14.rs (consolidation steps 21/22) ----
/// itpikachuthunder.c `it_802B22B8_inline`: copy `pos` to the child's `x28`, set its `x4`.
/// # Safety
/// `x34` NULL or a GObj whose user_data is NULL or an Item.
pub unsafe fn it_802b22b8_inline(ip: &mut Item) {
    melee_core::ct!("it_802b22b8_inline");
    let c = ip.xDD4_itemVar.pikachuthunder.x34;
    if !c.is_null() {
        if let Some(ch) = item_of(c) {
            ch.xDD4_itemVar.pikachuthunder.x28 = ip.pos;
            ch.xDD4_itemVar.pikachuthunder.x4 = 1;
        }
    }
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `it_802B1DEC` (itpikachuthunder.c): `pikachuthunder.x4`.
pub fn it_802b1dec(ip: &Item) -> i32 {
    melee_core::ct!("it_802b1dec");
    unsafe { ip.xDD4_itemVar.pikachuthunder.x4 }
}

/// `it_802B1FE8` (itpikachuthunder.c): `*pos = pos - (0, sa->x8 * pikachuthunder.x18, 0)`.
/// # Safety
/// The special attributes must be a valid `itPikachuthunderAttributes`.
pub unsafe fn it_802b1fe8(ip: &Item, pos: &mut GcVec) {
    melee_core::ct!("it_802b1fe8");
    let a: &itPikachuthunderAttributes = sa(ip);
    let scale = ip.xDD4_itemVar.pikachuthunder.x18;
    let mut offset = -a.x8;
    offset *= scale;
    *pos = ip.pos;
    pos.y += offset;
}
