//! Port of src/melee/it/kinds/itsamusmissile.c: Samus's missiles (homing and smash; logic table 52). Callees
//! of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itsamusmissile.rs.
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::eflib::efLib_DestroyAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn ftSs_SpecialS_8012A068(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B634(a0: *mut Vec, a1: *mut Vec, a2: *mut HSD_GObj, a3: f32) -> *mut HSD_GObj => unsafe { crate::tu::it_26B1::it_8026B634(a0 as _, a1 as _, a2, a3) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB44(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BB44(a0) };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026BD24(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd24(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026C258(a0: *mut Vec, a1: f32) -> *mut HSD_GObj => unsafe { crate::tu::it_26B1::it_8026C258(a0 as _, a1) };
    fn it_8026E71C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E71C(a0, a1) };
    fn it_80272A60(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272A60(a0) };
    fn it_80272B40(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272B40(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_8027518C(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_8027518C(a0) };
    fn it_802751D8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_802751d8(&mut *((*a0).user_data as *mut Item)) as _ };
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_AngleXY(a0, a1) };
    fn lbVector_NormalizeXY(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_NormalizeXY(a0) };
    fn lbVector_Rotate(a0: *mut Vec, a1: i32, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_Rotate(a0, a1, a2) };
    fn lbVector_Sub(a0: *mut Vec, a1: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_Sub(a0, a1) };
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

/// `M_PI_2_F` (`M_PI_F / 2.0F`)
const M_PI_2_F: f32 = 3.14159265358979323846f32 / 2.0f32;

/// `ItemStateTable it_803F7340[]`
pub static mut it_803F7340: [ItemStateTable; 4] = [
    st_row(0, pred_b(itSamusmissile_UnkMotion0_Anim), Some(itSamusmissile_UnkMotion0_Phys), pred_b(itSamusmissile_UnkMotion0_Coll)),
    st_row(1, pred_b(itSamusmissile_UnkMotion1_Anim), Some(itSamusmissile_UnkMotion1_Phys), pred_b(itSamusmissile_UnkMotion1_Coll)),
    st_row(2, pred_b(itSamusmissile_UnkMotion3_Anim), None, None),
    st_row(3, pred_b(itSamusmissile_UnkMotion3_Anim), None, None),
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

/// `itGetJObjGrandchild` (it/inlines.h, static inline)
#[inline]
unsafe fn itGetJObjGrandchild(gobj: *mut Item_GObj) -> *mut HSD_JObj {
    let jobj = GET_JOBJ(gobj);
    let mut child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    child = if child.is_null() { null_mut() } else { (*child).child };
    child
}

pub unsafe extern "C" fn it_802B62D0(gobj: *mut Item_GObj, pos: *mut Vec3, is_smash_missile: i32, facing_dir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802B62D0");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Samus_Missile;
    Item_InitSpawnPositionFromParent(&mut spawn, gobj, pos);
    Item_InitSpawnCommonFields(&mut spawn, gobj, facing_dir, true);

    let new_gobj = ext::Item_80268B18(&mut spawn);

    if !new_gobj.is_null() {
        let ip = GET_ITEM(new_gobj);

        (*ip).xDD4_itemVar.samusmissile.is_smash_missile = is_smash_missile;
        (*ip).xDD4_itemVar.samusmissile.owner = (*ip).owner;

        if !(*ip).xDD4_itemVar.samusmissile.owner.is_null() {
            (*ip).xDD4_itemVar.samusmissile.x4 = ext::ftSs_SpecialS_8012A068((*ip).xDD4_itemVar.samusmissile.owner) as i32;
        }

        if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
            it_802B66A8(new_gobj);
        } else {
            it_802B6A60(new_gobj);
        }

        ext::db_80225DD8(new_gobj, gobj);
        return new_gobj;
    }

    null_mut()
}

pub unsafe extern "C" fn it_802B63F8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802B63F8");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;

    if (*ip).xD44_lifeTimer > (*attrs).x4 - (*attrs).x8 {
        (*ip).xDAC_itcmd_var0 = 1;
    } else {
        (*ip).xDAC_itcmd_var0 = 0;
    }

    if (*ip).xD44_lifeTimer == (*attrs).x4 - (*attrs).x8 && (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
        ext::efLib_DestroyAll(gobj);
    }

    if (*ip).xD44_lifeTimer <= ((*attrs).x24 - (*attrs).x28) {
        (*ip).xDB0_itcmd_var1 = 1;
    }

    if (*ip).xD44_lifeTimer <= 0.0 {
        if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
            it_802B701C(gobj);
        } else {
            it_802B70A0(gobj);
        }
    } else {
        (*ip).xD44_lifeTimer -= 1.0;
    }
}

/// `itSamusMissile_ClampTurn` (static inline)
#[inline]
unsafe fn itSamusMissile_ClampTurn(ip: *mut Item, sa: *mut itSamusMissileAttributes) {
    let turn = (*ip).xDD4_itemVar.samusmissile.x8;
    let abs_turn: f32;
    let max_turn: f32;

    if turn < 0.0 {
        abs_turn = -turn;
    } else {
        abs_turn = turn;
    }

    max_turn = (*sa).x1C;
    if abs_turn > max_turn {
        if turn > 0.0 {
            (*ip).xDD4_itemVar.samusmissile.x8 = max_turn;
        } else {
            (*ip).xDD4_itemVar.samusmissile.x8 = -max_turn;
        }
    }
}

pub unsafe extern "C" fn it_802B64FC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B64FC");
    let mut vec3: Vec3 = core::mem::zeroed();
    let mut vec2: Vec3 = core::mem::zeroed();
    let mut vec1: Vec3 = core::mem::zeroed();
    let mut vec0: Vec3 = core::mem::zeroed();
    let temp_r3: *mut HSD_GObj;
    let temp_ret: *mut HSD_GObj;
    let mut var_f1: f32;

    let ip = GET_ITEM(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;
    vec3.z = 0.0;
    vec3.y = 0.0;
    vec3.x = 0.0;
    if ext::it_8026B634(&mut (*ip).pos, &mut vec3, (*ip).owner, (*ip).facing_dir).is_null() {
        temp_ret = ext::it_8026C258(&mut (*ip).pos, (*ip).facing_dir);
        temp_r3 = temp_ret;
        if !temp_r3.is_null() {
            ext::it_8026BB88(temp_r3, &mut vec3);
        } else {
            return;
        }
    }
    var_f1 = 0.0;
    if vec3.x == 0.0 && vec3.y == 0.0 {
        return;
    }
    vec2.x = (*ip).x40_vel.x;
    vec2.y = (*ip).x40_vel.y;
    vec2.z = 0.0;
    ext::lbVector_NormalizeXY(&mut vec2);
    vec1.x = vec3.x - (*ip).pos.x;
    vec1.y = vec3.y - (*ip).pos.y;
    vec1.z = 0.0;
    ext::lbVector_NormalizeXY(&mut vec1);
    var_f1 = ext::lbVector_AngleXY(&mut vec2, &mut vec1);
    if var_f1 < (*sa).x20 {
        return;
    }
    vec0 = vec1;
    ext::lbVector_Sub(&mut vec0, &mut vec2);
    if vec0.y > 0.001 {
        (*ip).xDD4_itemVar.samusmissile.x8 -= (*sa).x18;
    } else if vec0.y < 0.001 {
        (*ip).xDD4_itemVar.samusmissile.x8 += (*sa).x18;
    }
    itSamusMissile_ClampTurn(ip, sa);
}

pub unsafe extern "C" fn it_802B66A8(gobj: *mut Item_GObj) -> *mut c_void {
    melee_core::ct!("it_802B66A8");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;

    (*ip).x40_vel.x = (*attrs).xC * (*ip).facing_dir;
    (*ip).x40_vel.y = 0.0;

    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDAC_itcmd_var0 = 0;

    ext::it_80275158(gobj, (*attrs).x4);

    (*ip).xDD4_itemVar.samusmissile.x8 = 0.0;

    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);

    ext::efSync_Spawn(0x485, gobj, itGetJObjGrandchild(gobj))
}

/// `isSamusmissile_MotionAnim` (static inline)
#[inline]
unsafe fn isSamusmissile_MotionAnim(gobj: *mut Item_GObj) {
    it_802B63F8(gobj);
    let ip = GET_ITEM(gobj);
    if (*ip).xDB8_itcmd_var3 as u32 == 0 {
        let owner = (*ip).xDD4_itemVar.samusmissile.owner;
        if !owner.is_null()
            && (ext::ftSs_SpecialS_8012A068(owner) as u32).wrapping_sub((*ip).xDD4_itemVar.samusmissile.x4 as u32) >= 2
        {
            it_2725_Logic52_DmgDealt(gobj);
        }
    }
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusmissile_UnkMotion0_Anim");
    isSamusmissile_MotionAnim(gobj);
    false
}

/// `inlineA0` (static inline)
#[inline]
unsafe fn inlineA0(gobj: *mut Item_GObj, vec: *mut Vec3) {
    let ip = GET_ITEM(gobj);
    let sa1 = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;
    (*vec).x = (*sa1).xC * (*ip).facing_dir;
    (*vec).y = 0.0;
    (*vec).z = 0.0;
    ext::lbVector_Rotate(vec, 4, (*ip).xDD4_itemVar.samusmissile.x8 * -(*ip).facing_dir);
    (*ip).x40_vel.x = (*vec).x;
    (*ip).x40_vel.y = (*vec).y;
}

/// `itSamusmissile_SetRotationX` (static inline)
#[inline]
unsafe fn itSamusmissile_SetRotationX(gobj: *mut Item_GObj) {
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let child = HSD_JObjGetChild(jobj);

    HSD_JObjSetRotationX(child, (*ip).xDD4_itemVar.samusmissile.x8);
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusmissile_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    let sa0 = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;
    let mut vec: Vec3 = core::mem::zeroed();
    if (*ip).xDAC_itcmd_var0 != 0 {
        it_802B64FC(gobj);
        inlineA0(gobj, &mut vec);
        itSamusmissile_SetRotationX(gobj);
    } else {
        // VEC2_SQ_LEN(v) > SQ(x14): `SQ(x) + SQ(y)` is contracted, `SQ(x14)` is a plain product
        let sq_len = fmadds((*ip).x40_vel.x, (*ip).x40_vel.x, (*ip).x40_vel.y * (*ip).x40_vel.y);
        if sq_len > (*sa0).x14 * (*sa0).x14 {
            (*ip).x40_vel.x *= (*sa0).x10;
            (*ip).x40_vel.y *= (*sa0).x10;
        }
        itSamusmissile_SetRotationX(gobj);
    }
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusmissile_UnkMotion0_Coll");
    ext::it_8026E71C(gobj, Some(it_802B701C));
    false
}

pub unsafe extern "C" fn it_802B6A60(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B6A60");
    let ip = GET_ITEM(gobj);

    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;

    (*ip).x40_vel.x = (*attrs).x2C * (*ip).facing_dir;
    (*ip).x40_vel.y = 0.0;

    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;

    ext::it_80275158(gobj, (*attrs).x24);
    ext::it_8026B3A8(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);

    ext::efSync_Spawn(1156, gobj, itGetJObjGrandchild(gobj));
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusmissile_UnkMotion1_Anim");
    isSamusmissile_MotionAnim(gobj);
    false
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itSamusmissile_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;

    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).x40_vel.x = fmadds((*attrs).x30, (*ip).facing_dir, (*ip).x40_vel.x);
        {
            let xvel: f32 = if (*ip).x40_vel.x < 0.0 { -(*ip).x40_vel.x } else { (*ip).x40_vel.x };
            let max_speed: f32 = (*attrs).x34;
            if xvel > max_speed {
                if (*ip).facing_dir == 1.0 {
                    (*ip).x40_vel.x = max_speed;
                } else {
                    (*ip).x40_vel.x = -max_speed;
                }
            }
        }
    }
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusmissile_UnkMotion1_Coll");
    ext::it_8026E71C(gobj, Some(it_802B70A0));
    false
}

pub unsafe extern "C" fn it_2725_Logic52_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic52_DmgDealt");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
        if (*ip).msid != 2 {
            it_802B701C(gobj);
        }
    } else if (*ip).msid != 3 {
        it_802B70A0(gobj);
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic52_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic52_Clanked");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
        if (*ip).msid != 2 {
            it_802B701C(gobj);
        }
    } else if (*ip).msid != 3 {
        it_802B70A0(gobj);
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic52_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic52_HitShield");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
        if (*ip).msid != 2 {
            it_802B701C(gobj);
        }
    } else if (*ip).msid != 3 {
        it_802B70A0(gobj);
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic52_ShieldBounced(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic52_ShieldBounced");
    let ip = GET_ITEM(arg0);
    let jobj = GET_JOBJ(arg0);
    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile != 0 {
        ext::itColl_BounceOffShield(arg0);
        {
            let temp_f1 = ext::atan2f((*ip).x40_vel.x, (*ip).x40_vel.y);
            let var_f31: f32;
            if (*ip).facing_dir == 1.0 {
                var_f31 = temp_f1 - M_PI_2_F;
            } else {
                // MTXDegToRad(270) - temp_f1: the product is a folded constant
                var_f31 = 270.0f32 * 0.01745329252f32 - temp_f1;
            }
            let child = HSD_JObjGetChild(jobj);
            HSD_JObjSetRotationX(child, var_f31);
        }
    }
    false
}

pub unsafe extern "C" fn it_2725_Logic52_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_2725_Logic52_Reflected");
    let ip = GET_ITEM(gobj);
    let sa = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itSamusMissileAttributes;
    let jobj = GET_JOBJ(gobj);
    (*ip).xDB8_itcmd_var3 = 1;
    ext::it_80273030(gobj);
    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile != 0 {
        (*ip).x40_vel.x = (*sa).x2C * (*ip).facing_dir;
    }
    if (*ip).x40_vel.y < 0.0 {
        (*ip).x40_vel.y = -(*ip).x40_vel.y * (*ip).xC70;
    }
    HSD_JObjSetRotationY(jobj, (core::f64::consts::FRAC_PI_2 as f32) * (*ip).facing_dir);
    ext::efLib_DestroyAll(gobj);
    if (*ip).xDD4_itemVar.samusmissile.is_smash_missile == 0 {
        ext::efSync_Spawn(1157, gobj, itGetJObjGrandchild(gobj));
    } else {
        ext::efSync_Spawn(1156, gobj, itGetJObjGrandchild(gobj));
    }
    false
}

pub unsafe extern "C" fn it_802B701C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B701C");
    ext::it_8026BB44(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_80273454(gobj);
    ext::efLib_DestroyAll(gobj);
    ext::it_80272B40(gobj);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itSamusmissile_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itSamusmissile_UnkMotion3_Anim");
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn it_802B70A0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802B70A0");
    ext::it_8026BB44(gobj);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_80273454(gobj);
    ext::efLib_DestroyAll(gobj);
    ext::it_80272A60(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_2725_Logic52_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic52_EvtUnk");
    let ip = GET_ITEM(gobj);
    ext::it_8026B894(gobj, arg1);
    if (*ip).xDD4_itemVar.samusmissile.owner == arg1 {
        (*ip).xDD4_itemVar.samusmissile.owner = null_mut();
    }
}
