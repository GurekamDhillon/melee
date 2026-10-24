//! Port of src/melee/it/kinds/itnesspkthunderball.c: the PK Thunder head (logic table 26). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnesspkthunderball.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf};
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
    fn Item_80267454(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_80267454(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE60() -> u32 => unsafe { crate::tu::item::Item_8026AE60() };
    fn Stage_80225194() -> u32 => unsafe { melee_world::tu::stage::Stage_80225194() };
    fn atan2f(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::lbtrigf::atan2f(a0, a1) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftLib_800865D8(a0: *mut HSD_GObj, a1: *mut f32, a2: *mut f32);
    fn ftNs_SpecialHi_CheckSpecialHiHold(a0: *mut HSD_GObj) -> i32;
    fn ftNs_SpecialHi_ItemPKThunderCheckOwn(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftNs_SpecialHi_ItemPKThunderRemove(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802AC43C(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut Vec, a3: i32, a4: i32, a5: f32) -> *mut HSD_GObj => unsafe { crate::tu::itnesspkthundertrail::it_802AC43C(a0 as _, a1 as _, a2 as _, a3, a4, a5) as _ };
    fn it_802AC58C(a0: *mut HSD_GObj) => unsafe { crate::tu::itnesspkthundertrail::it_802AC58C(a0 as _) };
    fn it_802AC5D8(a0: *mut HSD_GObj) => unsafe { crate::tu::itnesspkthundertrail::it_802AC5D8(a0 as _) };
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Angle(a0, a1) };
    fn lbVector_CrossprodNormalized(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_CrossprodNormalized(a0, a1, a2) };
    fn lbVector_Mirror(a0: *mut Vec, a1: *mut Vec) => unsafe { melee_runtime::tu::lbvector::lbVector_Mirror(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const M_PI: f64 = 3.14159265358979323846;

/// `MTXDegToRad(a)`
#[inline]
fn MTXDegToRad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ItemStateTable it_803F6BC8[]`
pub static mut it_803F6BC8: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itNesspkthunderball_UnkMotion0_Anim),
    Some(itNesspkthunderball_UnkMotion0_Phys),
    pred_b(itNesspkthunderball_UnkMotion0_Coll),
)];

#[inline]
unsafe fn Item_InitSpawnPositionFromParent(spawn: *mut SpawnItem, parent: *mut HSD_GObj, pos: *mut Vec3) {
    (*spawn).prev_pos = *pos;
    (*spawn).prev_pos.z = 0.0;
    ext::it_8026BB68(parent, &mut (*spawn).pos);
}

#[inline]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802AB3F0(gobj: *mut Item_GObj, out: *mut Vec3, idx: i32) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            if !out.is_null() {
                *out = (*ip).xDD4_itemVar.pkthunder.positions[(idx * 2) as usize];
            }
        } else if !out.is_null() {
            (*out).z = 0.0;
            (*out).y = 0.0;
            (*out).x = 0.0;
        }
    } else if !out.is_null() {
        (*out).z = 0.0;
        (*out).y = 0.0;
        (*out).x = 0.0;
    }
}

pub unsafe extern "C" fn it_802AB468(gobj: *mut Item_GObj, out: *mut f32, _unused: i32) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            if out.is_null() {
                return;
            }
            *out = (*ip).xDD4_itemVar.pkthunder.xEEC;
            return;
        }
        if out.is_null() {
            return;
        }
        *out = 0.0;
        return;
    }
    if out.is_null() {
        return;
    }
    *out = 0.0;
}

pub unsafe extern "C" fn it_802AB4B8(gobj: *mut Item_GObj, cd: *mut CollData) -> bool {
    let ip = GET_ITEM(gobj);
    let mut ret = false;
    ext::it_8026DA08(gobj);
    if (*ip).x40_vel.y > 0.0 {
        if (*cd).env_flags & (Collide_CeilingPush | Collide_CeilingHug) != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & (Collide_FloorPush | Collide_FloorHug) != 0 {
        ret = true;
    }
    if (*ip).x40_vel.x > 0.0 {
        if (*cd).env_flags & Collide_LeftWallMask != 0 {
            ret = true;
        }
    } else if (*cd).env_flags & Collide_RightWallMask != 0 {
        ret = true;
    }
    ret
}

pub unsafe extern "C" fn it_802AB568(gobj: *mut Item_GObj) -> *mut HSD_GObj {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            return (*ip).owner;
        }
    }
    null_mut()
}

pub unsafe extern "C" fn it_802AB58C(owner: *mut Item_GObj, pos: *mut Vec3, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Ness_PKThunder;
    Item_InitSpawnPositionFromParent(&mut spawn, owner, pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = ext::Item_8026AE60() as i32;

    let ball = ext::Item_80268B18(&mut spawn);
    if !ball.is_null() {
        let ip = GET_ITEM(ball);
        let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPKThunderAttributes;

        Item_ClearCmdVars(ip);
        ext::it_80275158(ball, (*attr).x0_PKTHUNDER_LIFETIME);
        (*ip).xDD4_itemVar.pkthunder.xEF4 = null_mut();
        for i in 0..16 {
            (*ip).xDD4_itemVar.pkthunder.positions[i] = *pos;
        }
        for i in 0..16 {
            (*ip).xDD4_itemVar.pkthunder.angles[i] = MTXDegToRad((*attr).x8_PKTHUNDER_SPAWN_ANGLE);
        }
        for i in 0..6 {
            (*ip).xDD4_itemVar.pkthunder.xDD4[i] = null_mut();
        }
        (*ip).xDD4_itemVar.pkthunder.xEFC = 0;
        (*ip).xDD4_itemVar.pkthunder.xEF8 = 0;
        (*ip).xDD4_itemVar.pkthunder.xF00 = 0;
        (*ip).xDD4_itemVar.pkthunder.xF04 = owner;
        (*ip).xDD4_itemVar.pkthunder.xF08 = null_mut();
        it_802ABA4C(ball);
        (*ip).xDCC_flag.set_b3(0);
        if ext::Stage_80225194() == 0x54 {
            ext::Item_80267454(ball);
        }
    }
    ball
}

pub unsafe extern "C" fn it_802AB90C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);

    ext::it_802725D4(gobj);

    if !(*ip).xDD4_itemVar.pkthunder.xF04.is_null() {
        if (*ip).xDD4_itemVar.pkthunder.xF00 == 0 && (*ip).xDD4_itemVar.pkthunder.xF04 == (*ip).owner {
            ext::ftNs_SpecialHi_ItemPKThunderRemove((*ip).xDD4_itemVar.pkthunder.xF04);
        }
        (*ip).xDD4_itemVar.pkthunder.xF04 = null_mut();
    }

    for i in 0..6 {
        if !(*ip).xDD4_itemVar.pkthunder.xDD4[i].is_null() {
            it_802AC58C_ext((*ip).xDD4_itemVar.pkthunder.xDD4[i]);
            (*ip).xDD4_itemVar.pkthunder.xDD4[i] = null_mut();
        }
    }
    (*ip).xDC8_word.flags.set_x13(0);
}

pub unsafe extern "C" fn it_802AB9C0(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() && (*ip).xDD4_itemVar.pkthunder.xF00 == 0 {
            (*ip).owner = null_mut();
            (*ip).xDC8_word.flags.set_x13(0);
            for i in 0..6 {
                if !(*ip).xDD4_itemVar.pkthunder.xDD4[i].is_null() {
                    ext::it_802AC5D8((*ip).xDD4_itemVar.pkthunder.xDD4[i]);
                }
            }
        }
    }
}

#[inline]
unsafe fn it_802AC58C_ext(g: *mut Item_GObj) {
    ext::it_802AC58C(g)
}

pub unsafe extern "C" fn it_802ABA4C(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPKThunderAttributes;

    Item_ClearFlagsAndEnterState(gobj, ip, 0);
    ext::it_80275158(gobj, (*attr).x0_PKTHUNDER_LIFETIME);
    for i in 0..16 {
        (*ip).xDD4_itemVar.pkthunder.angles[i] = MTXDegToRad((*attr).x8_PKTHUNDER_SPAWN_ANGLE);
    }
    {
        let speed = (*attr).x4_PKTHUNDER_SPEED;
        (*ip).xDD4_itemVar.pkthunder.xEEC = speed;
        (*ip).x40_vel.y = speed;
    }
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::db_80225DD8(gobj, (*ip).xDD4_itemVar.pkthunder.xF04);
}

pub unsafe extern "C" fn itNesspkthunderball_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    match (*ip).xDD4_itemVar.pkthunder.xEF8 {
        0 | 2 | 4 | 6 | 8 | 10 => {
            if (*ip).xDD4_itemVar.pkthunder.xEFC <= 5 {
                let idx = (*ip).xDD4_itemVar.pkthunder.xEFC as usize;
                if (*ip).owner == (*ip).xDD4_itemVar.pkthunder.xF04 {
                    let dir = (*ip).facing_dir;
                    (*ip).xDD4_itemVar.pkthunder.xDD4[idx] = ext::it_802AC43C(
                        (*ip).xDD4_itemVar.pkthunder.xF04,
                        gobj,
                        &mut (*ip).pos,
                        (*ip).xDD4_itemVar.pkthunder.xEFC,
                        (*ip).xAC4_ignoreItemID as i32,
                        dir,
                    );
                } else {
                    let dir = (*ip).facing_dir;
                    (*ip).xDD4_itemVar.pkthunder.xDD4[idx] = ext::it_802AC43C(
                        null_mut(),
                        gobj,
                        &mut (*ip).pos,
                        (*ip).xDD4_itemVar.pkthunder.xEFC,
                        (*ip).xAC4_ignoreItemID as i32,
                        dir,
                    );
                }
                (*ip).xDD4_itemVar.pkthunder.xEFC += 1;
            }
        }
        _ => {}
    }

    (*ip).xDD4_itemVar.pkthunder.xEF8 += 1;

    if (*ip).xDD4_itemVar.pkthunder.xF00 == 0 && !(*ip).xDD4_itemVar.pkthunder.xF04.is_null() {
        if (*ip).owner == (*ip).xDD4_itemVar.pkthunder.xF04 {
            if ext::ftNs_SpecialHi_CheckSpecialHiHold((*ip).xDD4_itemVar.pkthunder.xF04) != 1 {
                return true;
            }
        } else {
            it_802AB90C(gobj);
            return true;
        }
    }
    ext::it_80273130(gobj) != 0
}

#[inline]
unsafe fn itNesspkthunderball_ShiftAngles(ip: *mut Item) {
    let mut i: i32 = 14;
    while i >= 0 {
        (*ip).xDD4_itemVar.pkthunder.angles[(i + 1) as usize] = (*ip).xDD4_itemVar.pkthunder.angles[i as usize];
        i -= 1;
    }
}

#[inline]
unsafe fn itNesspkthunderball_ShiftPositions(ip: *mut Item) {
    let mut i: i32 = 14;
    while i >= 0 {
        (*ip).xDD4_itemVar.pkthunder.positions[(i + 1) as usize] = (*ip).xDD4_itemVar.pkthunder.positions[i as usize];
        i -= 1;
    }
}

pub unsafe extern "C" fn itNesspkthunderball_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itPKThunderAttributes;
    let mut stick: Vec3 = core::mem::zeroed();
    let mut angle: f32;
    let mut cross: Vec3 = core::mem::zeroed();

    if (*ip).xDD4_itemVar.pkthunder.xF00 == 0
        && !(*ip).xDD4_itemVar.pkthunder.xF04.is_null()
        && (*ip).owner == (*ip).xDD4_itemVar.pkthunder.xF04
        && ext::ftNs_SpecialHi_CheckSpecialHiHold((*ip).xDD4_itemVar.pkthunder.xF04) == 1
    {
        itNesspkthunderball_ShiftAngles(ip);

        ext::ftLib_800865D8((*ip).xDD4_itemVar.pkthunder.xF04, &mut stick.x, &mut stick.y);
        stick.z = 0.0;

        if ABS(stick.x) > (*attr).xC_PKTHUNDER_STICK_THRESHOLD || ABS(stick.y) > (*attr).xC_PKTHUNDER_STICK_THRESHOLD {
            angle = ext::lbVector_Angle(&mut (*ip).x40_vel, &mut stick);
            ext::lbVector_CrossprodNormalized(&mut (*ip).x40_vel, &mut stick, &mut cross);

            if angle >= MTXDegToRad(45.0) {
                if cross.z > 0.0 {
                    (*ip).xDD4_itemVar.pkthunder.angles[0] = fmadds((*attr).x10_PKTHUNDER_TURN_RADIUS, 0.01745329252f32, (*ip).xDD4_itemVar.pkthunder.angles[0]);
                } else if cross.z < 0.0 {
                    (*ip).xDD4_itemVar.pkthunder.angles[0] = fmadds(-(*attr).x10_PKTHUNDER_TURN_RADIUS, 0.01745329252f32, (*ip).xDD4_itemVar.pkthunder.angles[0]);
                }
            }
            if angle < MTXDegToRad(45.0) {
                if cross.z > 0.0 {
                    (*ip).xDD4_itemVar.pkthunder.angles[0] += angle / (45.0 / (*attr).x10_PKTHUNDER_TURN_RADIUS);
                } else if cross.z < 0.0 {
                    (*ip).xDD4_itemVar.pkthunder.angles[0] -= angle / (45.0 / (*attr).x10_PKTHUNDER_TURN_RADIUS);
                }
            }

            while ((*ip).xDD4_itemVar.pkthunder.angles[0] as f64) < -M_TAU {
                (*ip).xDD4_itemVar.pkthunder.angles[0] = ((*ip).xDD4_itemVar.pkthunder.angles[0] as f64 + M_TAU) as f32;
            }
            while (*ip).xDD4_itemVar.pkthunder.angles[0] as f64 > M_TAU {
                (*ip).xDD4_itemVar.pkthunder.angles[0] = ((*ip).xDD4_itemVar.pkthunder.angles[0] as f64 - M_TAU) as f32;
            }

            (*ip).x40_vel.x = (*ip).xDD4_itemVar.pkthunder.xEEC * cosf((*ip).xDD4_itemVar.pkthunder.angles[0]);
            (*ip).x40_vel.y = (*ip).xDD4_itemVar.pkthunder.xEEC * sinf((*ip).xDD4_itemVar.pkthunder.angles[0]);
            (*ip).x40_vel.z = 0.0;
        }
    } else {
        itNesspkthunderball_ShiftAngles(ip);
    }

    itNesspkthunderball_ShiftPositions(ip);
    (*ip).xDD4_itemVar.pkthunder.positions[0] = (*ip).pos;
}

pub unsafe extern "C" fn itNesspkthunderball_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if it_802AB4B8(gobj, &mut (*ip).x378_itemColl) {
        ext::it_802725D4(gobj);
        return true;
    }
    false
}

pub unsafe extern "C" fn itNessPKThunderball_Logic26_DmgDealt(gobj: *mut Item_GObj) -> bool {
    it_802AB90C(gobj);
    true
}

pub unsafe extern "C" fn it_802AC074(gobj: *mut Item_GObj) -> bool {
    it_802AB90C(gobj);
    true
}

pub unsafe extern "C" fn it_802AC098(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if !(*ip).xDD4_itemVar.pkthunder.xF04.is_null() {
        if (*ip).owner == (*ip).xDD4_itemVar.pkthunder.xF04 {
            ext::ftNs_SpecialHi_ItemPKThunderCheckOwn((*ip).xDD4_itemVar.pkthunder.xF04, gobj);
        }
        (*ip).xDD4_itemVar.pkthunder.xF04 = null_mut();
    }

    (*ip).xDD4_itemVar.pkthunder.xF00 = 1;
    (*ip).xDD4_itemVar.pkthunder.angles[0] = ((*ip).xDD4_itemVar.pkthunder.angles[0] as f64 + M_PI) as f32;
    Item_NormalizeAngle(&mut (*ip).xDD4_itemVar.pkthunder.angles[0]);

    for i in 0..6 {
        if !(*ip).xDD4_itemVar.pkthunder.xDD4[i].is_null() {
            ext::it_802AC58C((*ip).xDD4_itemVar.pkthunder.xDD4[i]);
            (*ip).xDD4_itemVar.pkthunder.xDD4[i] = null_mut();
        }
    }

    (*ip).xDD4_itemVar.pkthunder.xEF8 = 0;
    (*ip).xDD4_itemVar.pkthunder.xEFC = 0;

    for i in 1..16 {
        (*ip).xDD4_itemVar.pkthunder.positions[i] = (*ip).xDD4_itemVar.pkthunder.positions[0];
        (*ip).xDD4_itemVar.pkthunder.angles[i] = (*ip).xDD4_itemVar.pkthunder.angles[0];
    }

    (*ip).x40_vel.x = (*ip).xDD4_itemVar.pkthunder.xEEC * cosf((*ip).xDD4_itemVar.pkthunder.angles[0]);
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.pkthunder.xEEC * sinf((*ip).xDD4_itemVar.pkthunder.angles[0]);
    (*ip).x40_vel.z = 0.0;
    false
}

pub unsafe extern "C" fn it_802AC338(gobj: *mut Item_GObj) -> bool {
    it_802AB90C(gobj);
    true
}

pub unsafe extern "C" fn it_802AC35C(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    ext::lbVector_Mirror(&mut (*ip).x40_vel, &mut (*ip).xC58);
    (*ip).x40_vel.z = 0.0;
    (*ip).xDD4_itemVar.pkthunder.angles[0] = ext::atan2f((*ip).x40_vel.y, (*ip).x40_vel.x);
    Item_NormalizeAngle(&mut (*ip).xDD4_itemVar.pkthunder.angles[0]);
    false
}

pub unsafe extern "C" fn it_802AC3F8(gobj: *mut Item_GObj) -> bool {
    it_802AB90C(gobj);
    true
}

pub unsafe extern "C" fn it_802AC41C(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
