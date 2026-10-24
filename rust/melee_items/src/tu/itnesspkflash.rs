//! Port of src/melee/it/kinds/itnesspkflash.c: Ness's PK Flash (logic table 102). Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itnesspkflash.rs.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Item_8026B074(a0: *mut Item) => unsafe { crate::tu::item::Item_8026B074(a0) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn ftKb_SpecialNNs_800FEC78(a0: *mut HSD_GObj) -> i32;
    fn ftKb_SpecialNNs_800FECBC(a0: *mut HSD_GObj);
    fn ftLib_800865D8(a0: *mut HSD_GObj, a1: *mut f32, a2: *mut f32) => unsafe { crate::tu::ftlib::ftLib_800865D8(a0, a1, a2) };
    fn ftNs_SpecialN_CheckSpecialNHold(a0: *mut HSD_GObj) -> i32;
    fn ftNs_SpecialN_SetNULL(a0: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802AF940(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut Vec, a3: i32, a4: f32, a5: f32) -> *mut HSD_GObj => unsafe { crate::tu::itnesspkflashexplode::it_802AF940(a0 as _, a1 as _, a2 as _, a3, a4, a5) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

const M_PI: f64 = 3.14159265358979323846;
const M_PI_2: f64 = 1.57079632679489661923;

/// `ItemStateTable it_803F6B70[]`
pub static mut it_803F6B70: [ItemStateTable; 3] = [
    st_row(0, pred_b(itNesspkflash_UnkMotion0_Anim), Some(itNesspkflash_UnkMotion0_Phys), pred_b(itNesspkflash_UnkMotion0_Coll)),
    st_row(1, pred_b(itNesspkflash_UnkMotion1_Anim), Some(itNesspkflash_UnkMotion1_Phys), pred_b(itNesspkflash_UnkMotion1_Coll)),
    st_row(2, pred_b(itNesspkflash_UnkMotion2_Anim), Some(itNesspkflash_UnkMotion2_Phys), pred_b(itNesspkflash_UnkMotion2_Coll)),
];

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

/// `itNesspkflash_SetScale` (static inline). One fused site.
#[inline]
unsafe fn itNesspkflash_SetScale(jobj: *mut HSD_JObj, ip: *mut Item, attr: *mut itFlashAttributes) {
    let mut scale: Vec3 = core::mem::zeroed();
    let v = fmadds(
        (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash,
        ((*attr).xC_FLASH_GRAPHIC_SIZE_GROWTH_MUL - (*attr).x8_FLASH_GRAPHIC_SIZE_INIT_MUL) / (*attr).x4_FLASH_HITBOX_SIZE_MUL,
        (*attr).x8_FLASH_GRAPHIC_SIZE_INIT_MUL,
    );
    scale.z = v;
    scale.y = v;
    scale.x = v;
    HSD_JObjSetScale(jobj, &mut scale);
}

#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itFlashAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itFlashAttributes
}

pub unsafe extern "C" fn it_802AA7E4(gobj: *mut Item_GObj) -> *mut Fighter_GObj {
    (*GET_ITEM(gobj)).owner
}

pub unsafe extern "C" fn it_802AA7F0(gobj: *mut Item_GObj) -> bool {
    let it = GET_ITEM(gobj);
    (*it).msid == 0x2
}

pub unsafe extern "C" fn it_802AA810(gobj: *mut Item_GObj, cd: *mut CollData) -> bool {
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

pub unsafe extern "C" fn it_802AA8C0(gobj: *mut Item_GObj, pos: *mut Vec3, kind: ItemKind, facing_dir: f32) -> *mut HSD_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&mut spawn, gobj, pos);
    Item_InitSpawnCommonFields(&mut spawn, gobj, facing_dir, true);

    let flash = ext::Item_80268B18(&mut spawn);
    if !flash.is_null() {
        let ip = GET_ITEM(flash);
        let attr = attrs_of(ip);
        Item_ClearCmdVars(ip);
        ext::it_80275158(flash, (*attr).x0_FLASH_LIFETIMER);
        (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash = 0.0;
        (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash = 0;
        (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = gobj;
        (*ip).xDD4_itemVar.pkflush.xDE4_PKFlash = 0;
        it_802AAA80(flash);
    }
    flash
}

pub unsafe extern "C" fn it_2725_Logic102_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802725D4(gobj);

    if !(*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner.is_null() {
        if (*ip).owner == (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner {
            let o = (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner;
            if (*ip).kind == It_Kind_Ness_PKFlush {
                ext::ftNs_SpecialN_SetNULL(o);
            } else if (*ip).kind == It_Kind_Kirby_NessPKFlush {
                ext::ftKb_SpecialNNs_800FECBC(o);
            }
        }
        (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = null_mut();
    }
    (*ip).xDC8_word.flags.set_x13(0);
}

pub unsafe extern "C" fn it_802AAA50(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        if !ip.is_null() {
            (*ip).owner = null_mut();
            (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = null_mut();
            (*ip).xDC8_word.flags.set_x13(0);
        }
    }
}

pub unsafe extern "C" fn it_802AAA80(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    Item_ClearFlagsAndEnterState(gobj, ip, 0);
    ext::it_80275158(gobj, (*attr).x0_FLASH_LIFETIMER);
    (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash = 0.0;
    (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash = 0;
    // (0.017453292f * x10 * facing_dir) + (f32) M_PI_2: fused site, fmadds(0.017453292f * x10, facing_dir, M_PI_2)
    let angle = fmadds(0.017453292f32 * (*attr).x10_FLASH_UNK1, (*ip).facing_dir, M_PI_2 as f32);
    (*ip).x40_vel.x = -(*attr).x14_FLASH_PEAK_RISE_HEIGHT * cosf(angle);
    (*ip).x40_vel.y = (*attr).x14_FLASH_PEAK_RISE_HEIGHT * sinf(angle);
    (*ip).x40_vel.z = 0.0;
    ext::db_80225DD8(gobj, (*ip).owner);
}

/// The `switch (ip->kind)` hold query of Anim/Phys (the `default` leaves `holding` at its zero init).
#[inline]
unsafe fn holding_of(ip: *mut Item) -> i32 {
    let o = (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner;
    if (*ip).kind == It_Kind_Ness_PKFlush {
        ext::ftNs_SpecialN_CheckSpecialNHold(o)
    } else if (*ip).kind == It_Kind_Kirby_NessPKFlush {
        ext::ftKb_SpecialNNs_800FEC78(o)
    } else {
        0
    }
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let jobj = GET_JOBJ(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    }

    if (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash == 0 {
        (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash += 1.0;
        if (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash >= (*attr).x4_FLASH_HITBOX_SIZE_MUL {
            (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash = (*attr).x4_FLASH_HITBOX_SIZE_MUL;
        }

        if !(*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner.is_null() && (*ip).owner == (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner {
            let holding = holding_of(ip);
            if holding != 1 {
                ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
                (*ip).xD44_lifeTimer = (*attr).x28_FLASH_EXPLOSION_DELAY;
                (*ip).xDAC_itcmd_var0 = 0;
            }
        }
    }

    if ext::it_80273130(gobj) == 1 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        (*ip).xD44_lifeTimer = (*attr).x28_FLASH_EXPLOSION_DELAY;
        (*ip).xDAC_itcmd_var0 = 0;
    }

    itNesspkflash_SetScale(jobj, ip, attr);
    false
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attr = attrs_of(ip);

    (*ip).xDAC_itcmd_var0 += 1;

    if (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash == 0 {
        if !(*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner.is_null() {
            let owner = (*ip).owner;
            if owner == (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner
                && (*ip).xDAC_itcmd_var0 >= (melee_runtime::cabi::f2i32((*attr).x28_FLASH_EXPLOSION_DELAY as f64).wrapping_sub(2)) as u32
            {
                if (*ip).kind == It_Kind_Ness_PKFlush {
                    ext::it_802AF940(owner, gobj, &mut (*ip).pos, It_Kind_Ness_PKFlush_Explode as i32, (*ip).facing_dir, (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash);
                } else if (*ip).kind == It_Kind_Kirby_NessPKFlush {
                    ext::it_802AF940(owner, gobj, &mut (*ip).pos, It_Kind_Kirby_NessPKFlush_Explode as i32, (*ip).facing_dir, (*ip).xDD4_itemVar.pkflush.xDD8_PKFlash);
                }
                return true;
            }
        }
    }

    itNesspkflash_SetScale(jobj, ip, attr);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attr = attrs_of(ip);
    itNesspkflash_SetScale(jobj, ip, attr);
    ext::it_80273130(gobj) != 0
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);

    if (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash == 0 {
        if !(*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner.is_null() && (*ip).owner == (*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner {
            let holding = holding_of(ip);
            if holding == 1 {
                let mut stick_y: f32 = 0.0;
                let mut stick_x: f32 = 0.0;
                ext::ftLib_800865D8((*ip).xDD4_itemVar.pkflush.xDE0_PKFlash_Owner, &mut stick_x, &mut stick_y);
                if ABS(stick_x) > 0.2 {
                    (*ip).x40_vel.x = fmadds(stick_x, (*attr).x18_FLASH_CONTROL, (*ip).x40_vel.x);
                    if ABS((*ip).x40_vel.x) > (*attr).x20_FLASH_UNK2 {
                        (*ip).x40_vel.x = if (*ip).x40_vel.x > 0.0 { (*attr).x20_FLASH_UNK2 } else { -(*attr).x20_FLASH_UNK2 };
                    }
                }
            }
        }
    }

    (*ip).x40_vel.y -= (*attr).x1C_FLASH_GRAVITY;
    if (*ip).x40_vel.y < -(*attr).x24_FLASH_UNK3 {
        (*ip).x40_vel.y = -(*attr).x24_FLASH_UNK3;
    }
    (*ip).x40_vel.z = 0.0;
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    itResetVelocity(GET_ITEM(gobj));
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    itResetVelocity(GET_ITEM(gobj));
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = attrs_of(ip);
    let jobj = GET_JOBJ(gobj);

    if it_802AA810(gobj, &mut (*ip).x378_itemColl) {
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
        (*ip).xD44_lifeTimer = (*attr).x28_FLASH_EXPLOSION_DELAY;
        ext::Item_8026B074(ip);
        ext::Item_8026AE84(ip, 0x86, 0x7F, 0x40);
    }

    itNesspkflash_SetScale(jobj, ip, attr);
    false
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNesspkflash_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNesspkflash_Logic102_Reflected(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    (*ip).xDD4_itemVar.pkflush.xDDC_PKFlash = 1;
    (*ip).facing_dir = -(*ip).facing_dir;
    HSD_JObjSetRotationY(jobj, (M_PI_2 * (*ip).facing_dir as f64) as f32);
    (*ip).x40_vel.x = -(*ip).x40_vel.x;
    (*ip).x40_vel.y = -(*ip).x40_vel.y;
    (*ip).x40_vel.z = 0.0;
    false
}

pub unsafe extern "C" fn itNesspkflash_Logic102_Clanked(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNesspkflash_Logic102_Absorbed(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNesspkflash_Logic102_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
