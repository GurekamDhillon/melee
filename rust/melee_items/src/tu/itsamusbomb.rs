//! Port of src/melee/it/kinds/itsamusbomb.c: Samus's morph-ball bomb (logic table 50). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itsamusbomb.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftSs_Init_80128944(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn ftSs_Init_80128A1C(a0: *mut HSD_GObj, a1: *mut c_void, a2: f32) -> i32;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E248(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E248(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80272B40(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272B40(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn lbVector_NormalizeXY(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_NormalizeXY(a0) };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
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

/// `ItemStateTable it_803F7220[]`
pub static mut it_803F7220: [ItemStateTable; 4] = [
    st_row(0, pred_b!(itSamusbomb_UnkMotion0_Anim), Some(itSamusbomb_UnkMotion0_Phys), pred_b!(itSamusbomb_UnkMotion0_Coll)),
    st_row(0, pred_b!(itSamusbomb_UnkMotion1_Anim), Some(itSamusbomb_UnkMotion1_Phys), pred_b!(itSamusbomb_UnkMotion1_Coll)),
    st_row(0, pred_b!(itSamusbomb_UnkMotion2_Anim), Some(itSamusbomb_UnkMotion2_Phys), pred_b!(itSamusbomb_UnkMotion2_Coll)),
    st_row(1, pred_b!(itSamusbomb_UnkMotion3_Anim), None, None),
];

/// `Item_InitSpawnPositionFromParent` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.z = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.x = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as _);
    (*spawn).x40 = 0;
}

pub unsafe extern "C" fn it_802B4AC8(parent_gobj: *mut Fighter_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802B4AC8");
    let mut si: SpawnItem = core::mem::zeroed();
    si.kind = It_Kind_Samus_Bomb;
    Item_InitSpawnPositionFromParent(&mut si, parent_gobj, pos);
    Item_InitSpawnCommonFields(&mut si, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&mut si);
    if !gobj.is_null() {
        it_802B4BA0(gobj);
        ext::db_80225DD8(gobj, parent_gobj);
        gobj
    } else {
        null_mut()
    }
}

pub unsafe extern "C" fn it_802B4BA0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B4BA0");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusBombAttributes;

    (*ip).x40_vel.y = (*(*ip).xCC_item_attr).x18;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDD4_itemVar.samusbomb.owner = (*ip).owner;
    ext::it_80275158(gobj, (*attr).x0);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_802B4C10(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B4C10");
    ext::Item_80268E5C(gobj, 0, 0x11);
}

/// `itSamusBomb_UnkMotion_Process` (static inline)
#[inline]
unsafe fn itSamusBomb_UnkMotion_Process(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let vx = (*ip).x40_vel.x;
    let abs = if vx < 0.0 { -vx } else { vx };
    if !(abs < 0.0001f32) {
        ext::it_80272980(gobj);
        if (*ip).facing_dir == 1.0 {
            HSD_JObjSetRotationY(jobj, 0.0);
        } else {
            HSD_JObjSetRotationY(jobj, core::f64::consts::PI as f32);
        }
    }
}

/// `itSamusBomb_UnkMotion_PreProcess` (static inline)
#[inline]
unsafe fn itSamusBomb_UnkMotion_PreProcess(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusBombAttributes;

    if (*ip).xD44_lifeTimer <= (*attr).x4 {
        ext::lb_8000BA0C(jobj, (*attr).x8);
    }

    if (*ip).xD44_lifeTimer <= 0.0 {
        it_802B53CC(gobj);
    } else {
        (*ip).xD44_lifeTimer -= 1.0;
    }
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion0_Anim");
    itSamusBomb_UnkMotion_PreProcess(gobj);
    false
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itSamusbomb_UnkMotion0_Phys");}

pub unsafe extern "C" fn itSamusbomb_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_802B4CF4));
    false
}

pub unsafe extern "C" fn it_802B4CF4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B4CF4");
    ext::Item_80268E5C(gobj, 1, 0x11);
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion1_Anim");
    itSamusBomb_UnkMotion_PreProcess(gobj);
    itSamusBomb_UnkMotion_Process(gobj);
    false
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusbomb_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion1_Coll");
    ext::it_8026E248(gobj, Some(it_802B4C10));
    false
}

pub unsafe extern "C" fn itSamusBomb_Logic50_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusBomb_Logic50_EnteredAir");
    ext::Item_80268E5C(gobj, 2, 0x11);
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion2_Anim");
    itSamusBomb_UnkMotion_PreProcess(gobj);
    itSamusBomb_UnkMotion_Process(gobj);
    false
}

/// `my_sqrtf` (static inline; 4 iterations instead of the usual 3). `_three - g*g*x` contracts to an fma.
#[inline]
fn my_sqrtf(x: f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = melee_core::fp::frsqrte(xd);
        for _ in 0..4 {
            guess = 0.5 * guess * ((-(guess * guess)).mul_add(xd, 3.0));
        }
        return (xd * guess) as f32;
    }
    x
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusbomb_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusBombAttributes;

    // VEC2_SQ_LEN(x7C) = SQ(x) + SQ(y), contracted
    let sq = fmadds((*ip).x7C.x, (*ip).x7C.x, (*ip).x7C.y * (*ip).x7C.y);
    if my_sqrtf(sq) > (*attr).xC {
        ext::lbVector_NormalizeXY(&mut (*ip).x7C);
        let mult = (*attr).xC;
        (*ip).x7C.x *= mult;
        (*ip).x7C.y *= mult;
    }
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion2_Coll");
    ext::it_8026E8C4(gobj, Some(it_802B4C10), Some(it_802B4CF4));
    false
}

pub unsafe extern "C" fn itSamusBomb_Logic50_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusBomb_Logic50_DmgDealt");
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 3 {
        it_802B53CC(gobj);
    }
    false
}

pub unsafe extern "C" fn itSamusBomb_Logic50_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusBomb_Logic50_Clanked");
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 3 {
        it_802B53CC(gobj);
    }
    false
}

pub unsafe extern "C" fn itSamusBomb_Logic50_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusBomb_Logic50_HitShield");
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 3 {
        it_802B53CC(gobj);
    }
    false
}

pub unsafe extern "C" fn itSamusBomb_Logic50_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusBomb_Logic50_ShieldBounced");
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic50_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic50_Reflected");
    let ip = GET_ITEM(gobj);

    ext::it_80273030(gobj);
    let yvel = (*ip).x40_vel.y;
    if yvel < 0.0 {
        (*ip).x40_vel.y = -yvel * (*ip).xC70;
    }
    (*ip).xDD4_itemVar.samusbomb.owner = null_mut();
    false
}

pub unsafe extern "C" fn it_802B53CC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B53CC");
    let ip = GET_ITEM(gobj);

    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::it_80273454(gobj);
    ext::it_8026BB44(gobj);
    ext::it_80272B40(gobj);
    ext::it_8026B3A8(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    (*ip).on_accessory = Some(it_802B5478);
    (*ip).xDD4_itemVar.samusbomb.x0 = 1;
}

pub unsafe extern "C" fn itSamusbomb_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusbomb_UnkMotion3_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn it_802B5478(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B5478");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.samusbomb.x0 != 0 {
        (*ip).xDD4_itemVar.samusbomb.x0 = 0;

        if !(*ip).xDD4_itemVar.samusbomb.owner.is_null()
            && ext::ftSs_Init_80128A1C((*ip).xDD4_itemVar.samusbomb.owner, (*ip).x5D4_hitboxes.as_mut_ptr() as *mut c_void, (*ip).scl) != 0
        {
            ext::ftSs_Init_80128944((*ip).xDD4_itemVar.samusbomb.owner, (*ip).pos.x, (*ip).x5D4_hitboxes[0].hit.scale);
        }
    }
}

pub unsafe extern "C" fn itSamusBomb_Logic50_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusBomb_Logic50_EvtUnk");
    let ip = GET_ITEM(gobj);
    ext::it_8026B894(gobj, ref_gobj);
    (*ip).xDD4_itemVar.samusbomb.owner = null_mut();
}
