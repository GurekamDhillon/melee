//! Native port of src/melee/it/kinds/itgreatfoxlaser.c: the Great Fox laser item (spawn, per-frame
//! motion, logic callbacks). Callees in other TUs go through `ext`. Checked against the C (the whole
//! TU, shipping contraction, item/jobj state hashed field by field) by tests/tu_itgreatfoxlaser.rs.
//!
//! No fused sites: only single multiplies (`attr * facing_dir`) and a subtract of `ABS(..)`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
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
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::helpers::decrease_lifetimer;
#[allow(unused_imports)]
use crate::itkinds::it_802d5884;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_chicorita_phys;
#[allow(unused_imports)]
use crate::itkinds::it_harisen_logic24_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lipstick_logic23_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lizardon_unk_motion1_phys;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_anim_inline;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_80275870;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_8027b730;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086644(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B3A8(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::tu::ithitbox::it_8027542C(a0) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::tu::itzako::it_8027B730(a0) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_world::tu_ext::lb_8000B1CC(a0, a1 as _, a2 as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_jobj_set!();
crate::inl_HSD_JObjGetRotation!();

/// `ItemStateTable it_803F8FD0[2]`
pub static it_803F8FD0: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itGreatfoxlaser_UnkMotion1_Anim), physics_updated: Some(itGreatfoxlaser_UnkMotion1_Phys), collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itGreatfoxlaser_UnkMotion1_Anim), physics_updated: Some(itGreatfoxlaser_UnkMotion1_Phys), collided: None },
];

/// `ABS(x)`: `x < 0 ? -x : x` (keeps `-0.0` and NaN as they are).
#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `Item_GObj* it_802EAF34(HSD_GObj* owner, Vec3* offset, int type)`: spawn a laser at `offset` of `owner`'s joint.
pub unsafe extern "C" fn it_802EAF34(owner: *mut HSD_GObj, offset: *mut Vec3, type_: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_802EAF34");
    let jobj = (*owner).hsd_obj as *mut HSD_JObj;
    let mut unused: Quaternion = core::mem::zeroed();
    let mut spawn: SpawnItem = core::mem::zeroed();
    ext::lb_8000B1CC(jobj, offset, &raw mut spawn.prev_pos);
    spawn.kind = It_Kind_GreatFox_Laser as _;
    spawn.pos = spawn.prev_pos;
    spawn.facing_dir = -1.0;
    spawn.x3C_damage = 0;
    spawn.vel.x = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.z = 0.0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = null_mut();
    spawn.x44_flag.anon_0.set_b0(0);
    spawn.x40 = 0;
    HSD_JObjGetRotation(jobj, &raw mut unused);
    let item_gobj = ext::Item_80268B18(&raw mut spawn);
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGreatFoxLaser_Attrs;
        (*ip).scl = ext::Ground_801C0498();
        HSD_JObjSetScaleX((*item_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        HSD_JObjSetScaleY((*item_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        HSD_JObjSetScaleZ((*item_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        (*ip).xDD4_itemVar.greatfoxlaser.x20 = owner;
        (*ip).xDD4_itemVar.greatfoxlaser.x28 = *offset;
        (*ip).xDD4_itemVar.greatfoxlaser.x38 = type_ as i16;
        if (*ip).xDD4_itemVar.greatfoxlaser.x38 == 0 {
            ext::it_80272460(&raw mut (*ip).x5D4_hitboxes[0].hit, (*attr).x0 as u32, item_gobj);
        } else {
            ext::it_80272460(&raw mut (*ip).x5D4_hitboxes[0].hit, (*attr).x8 as u32, item_gobj);
        }
        it_802EB268(item_gobj);
    }
    item_gobj
}

/// `void it_802EB1EC(Item_GObj*)`
pub unsafe extern "C" fn it_802EB1EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802EB1EC");
    let ip = GET_ITEM(gobj);
    ext::it_8027B730(gobj);
    ext::it_8027C56C(gobj, (*ip).facing_dir);
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);
    (*ip).xDD4_itemVar.greatfoxlaser.x36 = 1;
    it_802EB268(gobj);
}

/// `void it_802EB268(Item_GObj*)`
pub unsafe extern "C" fn it_802EB268(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802EB268");
    let ip = GET_ITEM(gobj);
    ext::it_8026B3A8(gobj);
    (*ip).xDD4_itemVar.greatfoxlaser.x36 = 60;
    (*ip).xDD4_itemVar.greatfoxlaser.x34 = 0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itGreatfoxlaser_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGreatfoxlaser_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itGreatFoxLaser_Attrs;
    if ext::it_80272C6C(gobj) == 0 {
        if (*ip).xDD4_itemVar.greatfoxlaser.x38 == 0 {
            (*ip).x40_vel.x = (*attr).x4 * (*ip).facing_dir;
            ext::it_80272460(&raw mut (*ip).x5D4_hitboxes[0].hit, (*attr).x0 as u32, gobj);
        } else {
            (*ip).x40_vel.x = (*attr).xC * (*ip).facing_dir;
            ext::it_80272460(&raw mut (*ip).x5D4_hitboxes[0].hit, (*attr).x8 as u32, gobj);
        }
    } else {
        let jobj = (*(*ip).xDD4_itemVar.greatfoxlaser.x20).hsd_obj as *mut HSD_JObj;
        let mut offset: Vec3 = (*ip).xDD4_itemVar.greatfoxlaser.x28;
        ext::lb_8000B1CC(jobj, &raw mut offset, &raw mut (*ip).pos);
    }
    if (*ip).xDD4_itemVar.greatfoxlaser.x36 == 0 {
        return 1;
    }
    (*ip).xDD4_itemVar.greatfoxlaser.x36 -= 1;
    0
}

pub unsafe extern "C" fn itGreatfoxlaser_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itGreatfoxlaser_UnkMotion1_Phys");
    let mut ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.greatfoxlaser.x34 != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        ext::it_8027C56C(gobj, (*ip).facing_dir);
        ip = GET_ITEM(gobj);
        ext::it_8026B3A8(gobj);
        (*ip).xDD4_itemVar.greatfoxlaser.x36 = 60;
        (*ip).xDD4_itemVar.greatfoxlaser.x34 = 0;
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    }
}

pub unsafe extern "C" fn itGreatFoxLaser_Logic27_DmgDealt(_item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGreatFoxLaser_Logic27_DmgDealt");
    0
}

pub unsafe extern "C" fn itGreatFoxLaser_Logic27_Clanked(_item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGreatFoxLaser_Logic27_Clanked");
    0
}

pub unsafe extern "C" fn itGreatFoxLaser_Logic27_Absorbed(_item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itGreatFoxLaser_Logic27_Absorbed");
    0
}

pub unsafe extern "C" fn it_2725_Logic27_Reflected(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_2725_Logic27_Reflected");
    let ip = GET_ITEM(gobj);
    let mut reflect_pos: Vec3 = core::mem::zeroed();
    let mut laser_pos: Vec3 = core::mem::zeroed();
    if (*ip).msid == 0 {
        ext::ftLib_80086644((*ip).xC64_reflectGObj, &raw mut reflect_pos);
        let jobj = (*(*ip).xDD4_itemVar.greatfoxlaser.x20).hsd_obj as *mut HSD_JObj;
        ext::lb_8000B1CC(jobj, &raw mut (*ip).xDD4_itemVar.greatfoxlaser.x28, &raw mut laser_pos);
        (*ip).xDD4_itemVar.greatfoxlaser.x28.x -= abs_c(laser_pos.x - reflect_pos.x);
        ext::lb_8000B1CC(jobj, &raw mut (*ip).xDD4_itemVar.greatfoxlaser.x28, &raw mut (*ip).pos);
        HSD_JObjSetTranslate((*(*ip).xBBC_dynamicBoneTable).bones[2], &raw mut (*ip).pos);
        (*ip).xDD4_itemVar.greatfoxlaser.x34 = 1;
    }
    0
}

pub unsafe extern "C" fn it_802EB5A8(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802EB5A8");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802EB5A8` (src/melee/it/kinds/itgreatfoxlaser.c): `it_8026B894(..)`.
pub fn it_802eb5a8(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802eb5a8");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----

// Split into src/melee/it/kinds/itleadead.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::itleadead::{it_802EAF28, it_802eaf28};
