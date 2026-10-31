//! Port of src/melee/it/itcoll.c: item collision against fighters and other items (hitbox vs hurtbox, the
//! damage log and its resolution, per-frame hitbox position updates, ECB nudging between items, the item
//! hurtbox setup). Callees of other TUs go through `ext`; `efSync_Spawn` is C varargs, so the extra arguments are
//! typed [`EfArg`]s.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_assignments, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

pub use super::iteffect::EfArg;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn ScriptGame_FighterDealtDamage(a0: *mut Fighter, a1: f32) -> f32;
    fn ScriptGame_HitRuleDamage(a0: *mut HitCapsule);
    fn ScriptGame_ItemModDamage(a0: *mut Item, a1: f32) -> f32;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[EfArg]) -> *mut c_void;
    fn ftCo_80098634(a0: *mut Item_FtTrack);
    fn ftCo_CalcYScaledKnockback(a0: f32, a1: f32, a2: f32) -> f32;
    fn ftColl_80076808(a0: *mut Fighter, a1: *mut HitCapsule, a2: i32, a3: *mut c_void, a4: i32);
    fn ftCommon_8007F804(a0: *mut Fighter) -> *mut c_void;
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_80086EB4(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086EB4(a0) };
    fn ftLib_80086FD4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn ft_80089228(a0: *mut Fighter, a1: i32, a2: i32, a3: f32) -> f32;
    fn gm_8016B0D4() -> i32;
    fn gm_8016B168() -> i32;
    fn itIsHeavy(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::itIsHeavy(a0) as _ };
    fn it_802746F8(a0: *mut HSD_GObj) -> *mut HSD_JObj => unsafe { crate::itkinds12::it_802746f8(a0) };
    fn it_8027B1F4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B1F4(a0) };
    fn it_8027B378(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: f32) => unsafe { crate::tu::it_279C::it_8027B378(a0, a1, a2) };
    fn it_8027B408(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: f32) => unsafe { crate::tu::it_279C::it_8027B408(a0, a1, a2) };
    fn it_8027B4A4(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B4A4(a0, a1) };
    fn it_8027B508(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_8027B508(a0, a1) };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0, a1, a2) };
    fn lbColl_80005BB0(a0: *mut HitCapsule, a1: i32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80005BB0(a0, a1) };
    fn lbColl_80007AFC(a0: *mut HitCapsule, a1: *mut HitCapsule, a2: f32, a3: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80007AFC(a0, a1, a2, a3) as _ };
    fn lbColl_80007ECC(a0: *mut HitCapsule, a1: *mut HurtCapsule, a2: *mut c_void, a3: f32, a4: f32, a5: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80007ECC(a0, a1, a2 as _, a3, a4, a5) as _ };
    fn lbColl_8000805C(a0: *mut HitCapsule, a1: *mut HurtCapsule, a2: *mut c_void, a3: i32, a4: f32, a5: f32, a6: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_8000805C(a0, a1, a2 as _, a3, a4, a5, a6) as _ };
    fn lbColl_80008248(a0: *mut HitCapsule, a1: *mut HurtCapsule, a2: *mut c_void, a3: f32, a4: f32, a5: f32) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80008248(a0, a1, a2 as _, a3, a4, a5) as _ };
    fn lbColl_80008440(a0: *mut HitCapsule) => unsafe { melee_runtime::tu::lbcollision::lbColl_80008440(a0) };
    fn lbColl_80008688(a0: *mut HitCapsule, a1: i32, a2: *mut c_void) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80008688(a0, a1, a2) as _ };
    fn lbColl_80008820(a0: *mut HitCapsule, a1: i32, a2: *mut c_void) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_80008820(a0, a1, a2) as _ };
    fn lbColl_80008A5C(a0: *mut HitCapsule) => unsafe { melee_runtime::tu::lbcollision::lbColl_80008A5C(a0) };
    fn lbColl_8000ACFC(a0: *mut c_void, a1: *mut HitCapsule) -> i32 => unsafe { melee_runtime::tu::lbcollision::lbColl_8000ACFC(a0, a1 as _) as _ };
    fn lbColl_CopyHitCapsule(a0: *mut HitCapsule, a1: *mut HitCapsule) => unsafe { melee_runtime::tu::lbcollision::lbColl_CopyHitCapsule(a0 as _, a1) };
    fn lbVector_RotateAboutUnitAxis(a0: *mut Vec, a1: *mut Vec, a2: f32) => unsafe { melee_runtime::tu::lbvector::lbVector_RotateAboutUnitAxis(a0, a1, a2) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::shared::Fighter_804D6524;
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use crate::tu::item::Item_804A0CCC;
    pub static mut it_804A0E70: [DamageLogEntry; 15] = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D18: u32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D1C: [u8; 4] = unsafe { core::mem::zeroed() };
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `M_TAU` is a `double` macro (src/Runtime/platform.h).
const M_TAU: f64 = 6.283185307179586;

/// `it_803B8560`
pub static it_803B8560: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 1.0, w: 0.0 };

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ItCollDynamicsDesc {
    pub bone_id: i32,
    pub offset: Vec3,
    pub size: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ItCollDynamics {
    pub _pad: [u8; 8],
    pub count: i32,
    pub descs: *mut ItCollDynamicsDesc,
}

/// EF IDs spawned on hit, indexed by HitElement (-1 = no effect)
static hit_effect_ids: [i32; 17] = [
    Ef_Id_Unk1000 as i32, Ef_Id_Unk1002 as i32, Ef_Id_Unk1001 as i32, Ef_Id_Unk1004 as i32, Ef_Id_Unk1145 as i32,
    Ef_Id_Unk1005 as i32, -1, -1, -1, Ef_Id_Unk1000 as i32, Ef_Id_Unk1000 as i32, -1, -1, Ef_Id_Unk1046 as i32, -1, -1, 0,
];

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

#[inline(always)]
unsafe fn HSD_JObjGetRotationX(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(715, jobj)");
    (*jobj).rotate.x
}
#[inline(always)]
unsafe fn HSD_JObjGetRotationY(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(730, jobj)");
    (*jobj).rotate.y
}
#[inline(always)]
unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
    assert!(!jobj.is_null(), "HSD_ASSERT(745, jobj)");
    (*jobj).rotate.z
}
#[inline(always)]
unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    assert!(!jobj.is_null(), "HSD_ASSERT(979, jobj)");
    assert!(!translate.is_null(), "HSD_ASSERT(980, translate)");
    *translate = (*jobj).translate;
}

/// `itColl_chkECBOverlap` (static): `true` if (pos_x, pos_y), expanded by the sum of two ECBs, encloses target.
pub unsafe extern "C" fn itColl_chkECBOverlap(pos_x: f32, pos_y: f32, ecb_a: *mut itECB, ecb_b: *mut itECB, target: *mut Vec3) -> i32 {
    melee_core::ct!("itColl_chkECBOverlap");
    let top = pos_y + ((*ecb_a).top + (*ecb_b).top);
    let bottom = pos_y + ((*ecb_a).bottom + (*ecb_b).bottom);
    let right = pos_x + ((*ecb_a).right + (*ecb_b).right);
    let left = pos_x + ((*ecb_a).left + (*ecb_b).left);
    if top >= (*target).y && bottom <= (*target).y && right <= (*target).x && left >= (*target).x { 1 } else { 0 }
}

pub unsafe extern "C" fn it_8026F9A0() {
    melee_core::ct!("it_8026F9A0");
    ext::it_804D6D18 = 0;
}

pub unsafe extern "C" fn it_8026F9AC(arg0: i32, fighter: *mut c_void, hit: *mut HitCapsule, _arg_item: *mut Item, hurt: *mut HurtCapsule) {
    melee_core::ct!("it_8026F9AC");
    let log_size = 15u32;
    if ext::it_804D6D18 < log_size {
        let i = ext::it_804D6D18 as usize;
        ext::it_804A0E70[i].x0 = arg0;
        ext::it_804A0E70[i].x4 = fighter;
        ext::it_804A0E70[i].x8 = hit;
        ext::it_804A0E70[i].xC = hurt;
        ext::it_804D6D18 += 1;
        return;
    }
    panic!("damage log over {}!! (HSD_ASSERT(105, 0))", log_size);
}

pub unsafe extern "C" fn it_8026FA2C(arg_item0: *mut Item, arg_hit: *mut HitCapsule, arg2: i32, arg_item3: *mut Item, arg_chk: i32) {
    melee_core::ct!("it_8026FA2C");
    for i in 0..4usize {
        let hit = &raw mut (*arg_item0).x5D4_hitboxes[i].hit;
        if (*hit).state != HitCapsule_Disabled && (*hit).x4 == (*arg_hit).x4 && ext::lbColl_80008688(hit, arg2, arg_item3 as *mut c_void) != 0 && arg_chk != 0 {
            ext::it_804D6D1C[i] = 0;
        }
    }
}

pub unsafe extern "C" fn it_8026FAC4(arg_item0: *mut Item, arg_hit: *mut HitCapsule, arg2: i32, arg3: *mut c_void, chk: i32) {
    melee_core::ct!("it_8026FAC4");
    if (*arg_item0).xAC4_ignoreItemID != 0 {
        let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
        while !item_gobj.is_null() {
            let it = item(item_gobj);
            if (*it).xAC4_ignoreItemID == (*arg_item0).xAC4_ignoreItemID {
                it_8026FA2C(it, arg_hit, arg2, arg3 as *mut Item, chk);
            }
            item_gobj = (*item_gobj).next;
        }
    } else {
        it_8026FA2C(arg_item0, arg_hit, arg2, arg3 as *mut Item, chk);
    }
}

/// `it_8026FC00_inline` (static inline)
pub unsafe extern "C" fn it_8026FC00_inline(arg_item: *mut Item, arg_hit: *mut HitCapsule, arg2: i32, arg3: *mut Fighter) {
    melee_core::ct!("it_8026FC00_inline");
    for i in 0..4usize {
        let hit = &raw mut (*arg_item).x5D4_hitboxes[i].hit;
        if (*hit).state != HitCapsule_Disabled && (*hit).x4 == (*arg_hit).x4 {
            ext::lbColl_80008820(hit, arg2, arg3 as *mut c_void);
        }
    }
}

pub unsafe extern "C" fn it_8026FC00(arg_item: *mut Item, arg_hit: *mut HitCapsule, arg2: i32, arg3: *mut Fighter) {
    melee_core::ct!("it_8026FC00");
    if (*arg_item).xAC4_ignoreItemID != 0 {
        let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
        // the C reads `item_gobj->next` here (an empty `if`)
        let _ = (*item_gobj).next;
        while !item_gobj.is_null() {
            let it = item(item_gobj);
            if (*it).xAC4_ignoreItemID == (*arg_item).xAC4_ignoreItemID {
                it_8026FC00_inline(it, arg_hit, arg2, arg3);
            }
            item_gobj = (*item_gobj).next;
        }
    } else {
        it_8026FC00_inline(arg_item, arg_hit, arg2, arg3);
    }
}

pub unsafe extern "C" fn it_8026FCF8(arg_item: *mut Item, arg_hit: *mut HitCapsule) {
    melee_core::ct!("it_8026FCF8");
    if (*arg_item).xAC4_ignoreItemID != 0 {
        let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
        while !item_gobj.is_null() {
            let it = item(item_gobj);
            if (*it).xAC4_ignoreItemID == (*arg_item).xAC4_ignoreItemID {
                let mut chk = false;
                let mut i = 0usize;
                while i < 4 {
                    let hit = &raw mut (*it).x5D4_hitboxes[i].hit;
                    if hit != arg_hit && (*hit).state != HitCapsule_Disabled && (*hit).x4 == (*arg_hit).x4 {
                        ext::lbColl_CopyHitCapsule(hit, arg_hit);
                        chk = true;
                        break;
                    }
                    chk = false;
                    i += 1;
                }
                if chk {
                    return;
                }
            }
            item_gobj = (*item_gobj).next;
        }
    }
    ext::lbColl_80008440(arg_hit);
}

pub unsafe extern "C" fn it_8026FE68(arg_item0: *mut Item, hit1: *mut HitCapsule, arg_item2: *mut Item, hit3: *mut HitCapsule) {
    melee_core::ct!("it_8026FE68");
    let mut sp48 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    sp48.x = 0.5 * ((*hit1).hurt_coll_pos.x + (*hit3).hurt_coll_pos.x);
    sp48.y = 0.5 * ((*hit1).hurt_coll_pos.y + (*hit3).hurt_coll_pos.y);
    sp48.z = 0.5 * ((*hit1).hurt_coll_pos.z + (*hit3).hurt_coll_pos.z);
    let dmg3 = (*hit3).damage;
    if ((dmg3 as i32).wrapping_sub((*ext::it_804D6D28).xB4)) < ((*hit1).damage as i32) {
        let dmg_int = dmg3 as i32;
        let var_r26 = if (*hit3).x41_b5() != 0 { 4 } else { 3 };
        it_8026FAC4(arg_item2, hit3, var_r26, arg_item0 as *mut c_void, 1);
        if dmg_int > (*arg_item2).xC48 {
            (*arg_item2).xC48 = dmg_int;
            (*arg_item2).xCF4_fighterGObjUnk = null_mut();
            (*arg_item2).xC38 = (*arg_item0).kind;
            (*arg_item2).xCD4 = (*arg_item0).pos;
            let vel_x = (*arg_item2).x40_vel.x;
            let vel_x_mag = if vel_x < 0.0 { -vel_x } else { vel_x };
            let dir;
            if vel_x_mag < (*ext::it_804D6D28).xD4 {
                let pos_x = (*arg_item2).pos.x;
                dir = if pos_x > (*arg_item0).pos.x { -1.0 } else { 1.0 };
            } else if vel_x < 0.0 {
                dir = -1.0;
            } else {
                dir = 1.0;
            }
            (*arg_item2).xCB8_outDamageDirection = dir;
        }
        ext::efSync_Spawn(0x41C, (*arg_item2).entity, &[EfArg::Vec3P(&raw const sp48)]);
    }
    let dmg1 = (*hit1).damage;
    if ((dmg1 as i32).wrapping_sub((*ext::it_804D6D28).xB4)) < ((*hit3).damage as i32) {
        let dmg_int = dmg1 as i32;
        let var_r26 = if (*hit1).x41_b5() != 0 { 4 } else { 3 };
        it_8026FAC4(arg_item0, hit1, var_r26, arg_item2 as *mut c_void, 0);
        if dmg_int > (*arg_item0).xC48 {
            (*arg_item0).xC48 = dmg_int;
            (*arg_item0).xCF4_fighterGObjUnk = null_mut();
            (*arg_item0).xC38 = (*arg_item2).kind;
            (*arg_item0).xCD4 = (*arg_item2).pos;
            let vel_x = (*arg_item0).x40_vel.x;
            let vel_x_mag = if vel_x < 0.0 { -vel_x } else { vel_x };
            let dir;
            if vel_x_mag < (*ext::it_804D6D28).xD4 {
                let pos_x = (*arg_item0).pos.x;
                dir = if pos_x > (*arg_item2).pos.x { -1.0 } else { 1.0 };
            } else if vel_x < 0.0 {
                dir = -1.0;
            } else {
                dir = 1.0;
            }
            (*arg_item0).xCB8_outDamageDirection = dir;
        }
        ext::efSync_Spawn(0x41C, (*arg_item0).entity, &[EfArg::Vec3P(&raw const sp48)]);
    }
}

/// `it_8026FAC4_noinline` (static)
pub unsafe extern "C" fn it_8026FAC4_noinline(ip: *mut Item, hit: *mut HitCapsule, arg2: i32, arg3: *mut c_void, chk: i32) {
    melee_core::ct!("it_8026FAC4_noinline");
    it_8026FAC4(ip, hit, arg2, arg3, chk);
}

pub unsafe extern "C" fn it_802701BC(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802701BC");
    let ip = item(gobj);
    (*ip).grab_victim = null_mut();
    (*ip).xD10 = 3.4028235e38;
    let mut fighter_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !fighter_gobj.is_null() {
        let fp = (*fighter_gobj).user_data as *mut Fighter;
        if (ext::ftLib_80086FD4(fighter_gobj, (*ip).owner) == 0 || (*ip).xDCD_flag.anon_0.b5() != 0)
            && (ext::gm_8016B168() == 0 || ext::gm_8016B0D4() != 0 || (*ip).xDCD_flag.anon_0.b6() != 0 || (*ip).x20_team_id != (*fp).team)
            && (*fp).x2219_b1() == 0
            && (*fp).x222A_b0() == 0
            && (*fp).x1988 == 0
            && (*fp).x198C == 0
            && (*fp).x221D_b6() == 0
            && ((*fp).x1A6A & ((*ip).xD08 as u16)) == 0
        {
            let mut it_hit_index = 0usize;
            while it_hit_index < 4 {
                let new_var = &raw mut (*ip).x5D4_hitboxes[it_hit_index].hit;
                if (*new_var).state != HitCapsule_Disabled
                    && (*new_var).element == HitElement_Catch as u32
                    && (((*new_var).x40_b2() != 0 && (*fp).ground_or_air == GA_Air) || ((*new_var).x40_b3() != 0 && (*fp).ground_or_air == GA_Ground))
                    && ext::lbColl_8000ACFC(fp as *mut c_void, new_var) == 0
                {
                    let mut ft_hit_index = 0usize;
                    while ft_hit_index < (*fp).hurt_capsules_len as usize {
                        if (*fp).hurt_capsules[ft_hit_index].is_grabbable != 0
                            && ext::lbColl_80007ECC(
                                new_var,
                                &raw mut (*fp).hurt_capsules[ft_hit_index].capsule,
                                ext::ftCommon_8007F804(fp),
                                (*ip).scl,
                                (*fp).x34_scale.y,
                                (*fp).cur_pos.z,
                            ) != 0
                        {
                            it_8026FAC4_noinline(ip, new_var, 0, fp as *mut c_void, 0);
                            let pos_x = abs_c((*fp).cur_pos.x - (*ip).pos.x);
                            if pos_x < (*ip).xD10 {
                                (*ip).atk_victim = (*fp).gobj;
                                (*ip).grab_victim = (*ip).atk_victim;
                                (*ip).xDD0_flag.anon_0.set_b1(1);
                                (*ip).xD10 = pos_x;
                            }
                            return;
                        }
                        ft_hit_index += 1;
                    }
                }
                it_hit_index += 1;
            }
        }
        fighter_gobj = (*fighter_gobj).next;
    }
}

/// `it_8026F9AC_noinline` (static)
pub unsafe extern "C" fn it_8026F9AC_noinline(arg0: i32, fighter: *mut c_void, hit: *mut HitCapsule, arg_item: *mut Item, hurt: *mut HurtCapsule) {
    melee_core::ct!("it_8026F9AC_noinline");
    it_8026F9AC(arg0, fighter, hit, arg_item, hurt);
}

pub unsafe extern "C" fn it_802703E8(arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802703E8");
    let arg_item = (*arg_item_gobj).user_data as *mut Item;
    if (*arg_item).xAC8_hurtboxNum == 0 {
        return;
    }
    let mut fighter_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !fighter_gobj.is_null() {
        'next: {
            if (*arg_item).owner == fighter_gobj && (*arg_item).xDCE_flag.anon_0.b0() == 0 {
                break 'next;
            }
            let fighter = (*fighter_gobj).user_data as *mut Fighter;
            if !((*fighter).x1064_thrownHitbox.anon_134.owner.is_null()
                || (*fighter).x1064_thrownHitbox.anon_134.owner != (*arg_item).owner
                || (*arg_item).xDCE_flag.anon_0.b0() != 0)
            {
                break 'next;
            }
            let ft_team: u32 = if !(*fighter).x1064_thrownHitbox.anon_134.owner.is_null() {
                (ext::ftLib_80086EB4((*fighter).x1064_thrownHitbox.anon_134.owner) as u32) & 0xFF
            } else {
                (*fighter).team as u32
            };
            if ext::gm_8016B168() != 0 && ext::gm_8016B0D4() == 0 && (*arg_item).xDCE_flag.anon_0.b1() == 0 && ft_team == (*arg_item).x20_team_id as u32 {
                break 'next;
            }
            for hit_index in 0..4usize {
                let hit = &raw mut (*fighter).x914[hit_index];
                if (*hit).state == HitCapsule_Disabled
                    || (*hit).element == HitElement_Catch as u32
                    || (*hit).x42_b7() != 1
                    || (((*hit).x40_b2() == 0 || (*arg_item).ground_or_air != GA_Air) && ((*hit).x40_b3() == 0 || (*arg_item).ground_or_air != GA_Ground))
                    || ext::lbColl_8000ACFC(arg_item as *mut c_void, hit) != 0
                    || (*arg_item).xD0C == 2
                {
                    continue;
                }
                let mut hurt_index = 0usize;
                while hurt_index < (*arg_item).xAC8_hurtboxNum as usize {
                    if ext::lbColl_8000805C(hit, &raw mut (*arg_item).xACC_itemHurtbox[hurt_index], null_mut(), 0, (*fighter).x34_scale.y, (*arg_item).scl, 0.0) == 0 {
                        hurt_index += 1;
                        continue;
                    }
                    if (*hit).element != HitElement_Inert as u32 {
                        ext::ftColl_80076808(fighter, hit, 0, arg_item as *mut c_void, 0);
                        let mut dmg = (*hit).damage as i32;
                        dmg = ext::ScriptGame_FighterDealtDamage(fighter, dmg as f32) as i32;
                        (*fighter).dmg.x1914 = dmg;
                        (*arg_item).xCA0 = (*arg_item).xCA0.wrapping_add(dmg);
                        if dmg > (*arg_item).xCA4 {
                            (*arg_item).xCA4 = dmg;
                        }
                        it_8026F9AC_noinline(1, fighter as *mut c_void, hit, arg_item, &raw mut (*arg_item).xACC_itemHurtbox[hurt_index]);
                        ext::it_8027B378((*fighter).gobj, (*arg_item).entity, dmg as f32);
                        let kind = (*arg_item).kind;
                        if kind == It_PKind_Random
                            && (*arg_item).xDD4_itemVar.pokemon.x0 == 7
                            && ((*hit).sfx_kind == 1 || (*hit).sfx_kind == 2)
                        {
                            ext::lbAudioAx_800237A8(0x61A87, 0x7F, 0x40);
                        } else if kind != It_PKind_Random
                            || (*arg_item).xDD4_itemVar.pokemon.x0 != 8
                            || ((*hit).sfx_kind != 1 && (*hit).sfx_kind != 2)
                        {
                            ext::lbColl_80005BB0(hit, -1);
                        }
                    } else {
                        (*fighter).unk_gobj = arg_item_gobj;
                    }
                    break;
                }
            }
        }
        fighter_gobj = (*fighter_gobj).next;
    }
}

/// `it_802706D0_sub3` (static inline)
pub unsafe extern "C" fn it_802706D0_sub3(it: *mut Item, arg_item: *mut Item, hit: *mut HitCapsule, arg_hurt: *mut HurtCapsule) {
    melee_core::ct!("it_802706D0_sub3");
    it_8026FAC4_noinline(it, hit, if (*hit).x41_b4() != 0 { 8 } else { 0 }, arg_item as *mut c_void, 0);
    let dir;
    if abs_c((*it).x40_vel.x) < (*ext::it_804D6D28).xD4 {
        dir = if (*it).pos.x > (*arg_item).pos.x { -1.0 } else { 1.0 };
    } else if (*it).x40_vel.x < 0.0 {
        dir = -1.0;
    } else {
        dir = 1.0;
    }
    (*it).xCB8_outDamageDirection = dir;
    let mut dmg = (*hit).damage as i32;
    (*it).xC34_damageDealt = dmg;
    dmg = ext::ScriptGame_ItemModDamage(it, dmg as f32) as i32;
    (*it).xC34_damageDealt = dmg;
    (*it).xCF4_fighterGObjUnk = null_mut();
    (*it).xC38 = (*arg_item).kind;
    (*it).xCD4 = (*arg_item).pos;
    (*arg_item).xCA0 = (*arg_item).xCA0.wrapping_add(dmg);
    if dmg > (*arg_item).xCA4 {
        (*arg_item).xCA4 = dmg;
    }
    it_8026F9AC_noinline(2, it as *mut c_void, hit, arg_item, arg_hurt);
    ext::it_8027B408((*it).entity, (*arg_item).entity, dmg as f32);
    let kind = (*arg_item).kind;
    if kind == It_PKind_Random && (*arg_item).xDD4_itemVar.pokemon.x0 == 7 && ((*hit).sfx_kind == 1 || (*hit).sfx_kind == 2) {
        ext::lbAudioAx_800237A8(0x61A87, 0x7F, 0x40);
    } else if kind != It_PKind_Random || (*arg_item).xDD4_itemVar.pokemon.x0 != 8 || ((*hit).sfx_kind != 1 && (*hit).sfx_kind != 2) {
        ext::lbColl_80005BB0(hit, -1);
    }
}

pub unsafe extern "C" fn it_802706D0(arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802706D0");
    let mut chk = false;
    let arg_item = item(arg_item_gobj);
    let mut count = 0i32;
    let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !item_gobj.is_null() {
        'next: {
            let it = (*item_gobj).user_data as *mut Item;
            if arg_item_gobj == item_gobj {
                chk = true;
                break 'next;
            } else if (((*arg_item).owner.is_null() && (*it).owner.is_null() && (*it).xDCD_flag.anon_0.b7() == 0 && (*arg_item).xDCE_flag.anon_0.b2() == 0)
                || (ext::ftLib_80086960((*it).owner) != 0
                    && ext::ftLib_80086960((*arg_item).owner) != 0
                    && ext::ftLib_80086FD4((*arg_item).owner, (*it).owner) != 0
                    && (*it).xDCD_flag.anon_0.b7() == 0
                    && (*arg_item).xDCE_flag.anon_0.b2() == 0)
                || (ext::gm_8016B168() != 0
                    && ext::gm_8016B0D4() == 0
                    && (*it).xDCD_flag.anon_0.b6() == 0
                    && (*arg_item).xDCE_flag.anon_0.b1() == 0
                    && (*it).x20_team_id == (*arg_item).x20_team_id))
            {
                break 'next;
            }
            if chk && (*arg_item).xDD0_flag.anon_0.b1() == 0 {
                count = 0;
                for hit_index in 0..4usize {
                    let arg_hit = &raw mut (*arg_item).x5D4_hitboxes[hit_index].hit;
                    if (*arg_hit).state != HitCapsule_Disabled
                        && (*arg_hit).element != HitElement_Catch as u32
                        && (((*arg_hit).x40_b2() != 0 && (*it).ground_or_air == GA_Air) || ((*arg_hit).x40_b3() != 0 && (*it).ground_or_air == GA_Ground))
                        && ext::lbColl_8000ACFC(it as *mut c_void, arg_hit) == 0
                    {
                        ext::it_804D6D1C[hit_index] = 1;
                        count += 1;
                    } else {
                        ext::it_804D6D1C[hit_index] = 0;
                    }
                }
            }
            for hit_index in 0..4usize {
                let hit = &raw mut (*it).x5D4_hitboxes[hit_index].hit;
                if (*hit).state == HitCapsule_Disabled
                    || (*hit).x42_b7() != 1
                    || (((*hit).x40_b2() == 0 || (*arg_item).ground_or_air != GA_Air) && ((*hit).x40_b3() == 0 || (*arg_item).ground_or_air != GA_Ground))
                    || ext::lbColl_8000ACFC(arg_item as *mut c_void, hit) != 0
                {
                    continue;
                }
                if chk && (*arg_item).xDD0_flag.anon_0.b1() == 0 && count != 0 {
                    let mut chk2 = false;
                    for i in 0..4usize {
                        if ext::it_804D6D1C[i] != 0 {
                            let arg_hit = &raw mut (*arg_item).x5D4_hitboxes[i].hit;
                            if (*hit).element == HitElement_Inert as u32 || (*arg_hit).element == HitElement_Inert as u32 {
                                if (*hit).element != (*arg_hit).element && ext::lbColl_80007AFC(hit, arg_hit, (*it).scl, (*arg_item).scl) != 0 {
                                    if (*hit).element == HitElement_Inert as u32 {
                                        (*it).xDCE_flag.anon_0.set_b6(1);
                                        (*it).toucher = arg_item_gobj;
                                    } else {
                                        (*arg_item).xDCE_flag.anon_0.set_b6(1);
                                        (*arg_item).toucher = (*it).entity;
                                    }
                                    chk2 = true;
                                    break;
                                }
                            } else if (*hit).x40_b0() == 1 && (*arg_hit).x40_b0() == 1 && ext::lbColl_80007AFC(hit, arg_hit, (*it).scl, (*arg_item).scl) != 0 {
                                it_8026FE68(it, hit, arg_item, arg_hit);
                                chk2 = true;
                                break;
                            }
                        }
                    }
                    if chk2 {
                        continue;
                    }
                }
                if (*arg_item).xAC8_hurtboxNum == 0 {
                    continue;
                }
                if (*hit).element == HitElement_Inert as u32 {
                    for i in 0..(*arg_item).xAC8_hurtboxNum as usize {
                        if ext::lbColl_80008248(hit, &raw mut (*arg_item).xACC_itemHurtbox[i], null_mut(), (*it).scl, (*arg_item).scl, 0.0) != 0 {
                            (*it).xDCE_flag.anon_0.set_b6(1);
                            (*it).toucher = arg_item_gobj;
                        }
                    }
                } else if (*arg_item).xD0C != 2 {
                    let mut hurt_index = 0usize;
                    while hurt_index < (*arg_item).xAC8_hurtboxNum as usize {
                        if ext::lbColl_8000805C(hit, &raw mut (*arg_item).xACC_itemHurtbox[hurt_index], null_mut(), 0, (*it).scl, (*arg_item).scl, 0.0) != 0 {
                            it_802706D0_sub3(it, arg_item, hit, &raw mut (*arg_item).xACC_itemHurtbox[hurt_index]);
                            break;
                        }
                        hurt_index += 1;
                    }
                }
            }
        }
        item_gobj = (*item_gobj).next;
    }
}

pub unsafe extern "C" fn it_80270CD8(ip: *mut Item, hit: *mut HitCapsule) -> f32 {
    melee_core::ct!("it_80270CD8");
    let attr = (*ip).xCC_item_attr;
    let c = &(*ext::it_804D6D28).x80_float;
    let mut f1: f32;
    if (*hit).x28 != 0 {
        // 0.01f * x24 * ((c11 * (mul * ((c10 * c8) + (c9 * (c10 * x28))))) + c12) + x2C
        let inner = fmadds(c[10], c[8], c[9] * (c[10] * (*hit).x28 as f32));
        let t = fmadds(c[11], (*attr).x1C_damage_mul * inner, c[12]);
        f1 = fmadds(0.01 * (*hit).x24 as f32, t, (*hit).x2C as f32);
    } else {
        let s = (*ip).xC9C as f32 + (*ip).xCA0 as f32;
        let inner = fmadds(c[8], s, c[9] * ((*hit).damage * s));
        let t = fmadds(c[11], (*attr).x1C_damage_mul * inner, c[12]);
        f1 = fmadds(0.01 * (*hit).x24 as f32, t, (*hit).x2C as f32);
    }
    if f1 >= c[7] {
        f1 = c[7];
    }
    f1
}

pub unsafe extern "C" fn it_80270E30(arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80270E30");
    if ext::it_804D6D18 == 0 {
        return;
    }
    let arg_item = (*arg_item_gobj).user_data as *mut Item;
    let mut max_knockback: f32 = -1.0;
    let mut hurt_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut index2: u32 = 0;
    {
        let mut index: u32 = 0;
        while index < ext::it_804D6D18 {
            let dl = &raw mut ext::it_804A0E70[index as usize];
            let hit = (*dl).x8;
            let attr = (*arg_item).xCC_item_attr;
            let c = &(*ext::it_804D6D28).x80_float;
            let mut knockback: f32;
            if (*hit).x28 != 0 {
                let inner = fmadds(c[10], c[8], c[9] * (c[10] * (*hit).x28 as f32));
                let t = fmadds(c[11], (*attr).x1C_damage_mul * inner, c[12]);
                knockback = fmadds(0.01 * (*hit).x24 as f32, t, (*hit).x2C as f32);
            } else {
                let s = (*arg_item).xC9C as f32 + (*arg_item).xCA0 as f32;
                let inner = fmadds(c[8], s, c[9] * ((*hit).damage * s));
                let t = fmadds(c[11], (*attr).x1C_damage_mul * inner, c[12]);
                knockback = fmadds(0.01 * (*hit).x24 as f32, t, (*hit).x2C as f32);
            }
            let knockback_cap = c[7];
            if knockback >= knockback_cap {
                knockback = knockback_cap;
            }
            if (*arg_item).xDCF_flag.anon_0.b1() == 0 {
                if (*arg_item).hold_kind == 4 || (*arg_item).hold_kind == 6 {
                    let mut sp18 = (*hit).damage;
                    let hit2 = (*dl).x8;
                    let arg_item2 = (*arg_item_gobj).user_data as *mut Item;
                    let hurt_coll_pos = &raw mut (*hit2).hurt_coll_pos;
                    let element = hit_effect_ids[(*hit2).element as usize];
                    match element {
                        1000 => {
                            ext::efSync_Spawn(1000, arg_item_gobj, &[EfArg::Vec3P(hurt_coll_pos), EfArg::F32P(&raw const sp18)]);
                        }
                        1001 | 1002 | 1004 | 1046 | 1145 => {
                            ext::efSync_Spawn(
                                hit_effect_ids[(*hit2).element as usize],
                                arg_item_gobj,
                                &[EfArg::Vec3P(hurt_coll_pos), EfArg::Ptr(arg_item2 as *mut c_void)],
                            );
                        }
                        1005 => {
                            ext::efSync_Spawn(
                                hit_effect_ids[(*hit2).element as usize],
                                arg_item_gobj,
                                &[EfArg::Vec3P(hurt_coll_pos), EfArg::F32P(&raw const (*arg_item2).facing_dir)],
                            );
                        }
                        _ => {}
                    }
                } else {
                    hurt_pos = (*hit).hurt_coll_pos;
                    ext::efSync_Spawn(1000, arg_item_gobj, &[EfArg::Vec3P(&raw const hurt_pos), EfArg::F32P(&raw const (*(*dl).x8).damage)]);
                }
            }
            if knockback > max_knockback {
                max_knockback = knockback;
                index2 = index;
            }
            index += 1;
        }
    }
    let temp_r29 = &raw mut ext::it_804A0E70[index2 as usize];
    match (*temp_r29).x0 {
        1 => {
            let fighter = (*temp_r29).x4 as *mut Fighter;
            (*arg_item).xCB0_source_ply = (*fighter).player_id as i32;
            (*arg_item).xCEC_fighterGObj = (*fighter).gobj;
            (*arg_item).xCF0_itemGObj = null_mut();
            let dir = if (*arg_item).pos.x > (*fighter).cur_pos.x { -1.0 } else { 1.0 };
            (*arg_item).xCCC_incDamageDirection = dir;
            ext::it_8027B4A4((*fighter).gobj, arg_item_gobj);
        }
        2 => {
            let it = (*temp_r29).x4 as *mut Item;
            let item_owner_gobj = (*it).owner;
            if !item_owner_gobj.is_null() && ext::ftLib_80086960(item_owner_gobj) != 0 {
                (*arg_item).xCB0_source_ply = (*((*item_owner_gobj).user_data as *mut Fighter)).player_id as i32;
                (*arg_item).xCEC_fighterGObj = null_mut();
                (*arg_item).xCF0_itemGObj = item_owner_gobj;
            } else {
                (*arg_item).xCB0_source_ply = 6;
                (*arg_item).xCEC_fighterGObj = null_mut();
                (*arg_item).xCF0_itemGObj = null_mut();
            }
            (*arg_item).xCB4 = (*it).kind;
            (*arg_item).xCE0 = (*it).pos;
            let dir;
            if abs_c((*it).x40_vel.x) < (*ext::it_804D6D28).x78_float {
                dir = if (*arg_item).pos.x > (*it).pos.x { -1.0 } else { 1.0 };
            } else if (*it).x40_vel.x < 0.0 {
                dir = 1.0;
            } else {
                dir = -1.0;
            }
            (*arg_item).xCCC_incDamageDirection = dir;
            ext::it_8027B508((*it).entity, arg_item_gobj);
        }
        _ => {}
    }
    (*arg_item).xCAC_angle = (*(*temp_r29).x8).kb_angle;
    (*arg_item).xCC8_knockback = max_knockback;
    (*arg_item).xCC4 = (*(*temp_r29).x8).element as i32;
    (*arg_item).xDCF_flag.anon_0.set_b6((*(*temp_r29).x8).x43_b0());
}

pub unsafe extern "C" fn it_8027129C(arg_item_gobj: *mut HSD_GObj, index: i32) {
    melee_core::ct!("it_8027129C");
    let it = item(arg_item_gobj);
    let hit = &raw mut (*it).x5D4_hitboxes[index as usize].hit;
    let state = (*hit).state;
    match state {
        HitCapsule_Enabled => {
            ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
            (*hit).x58 = (*hit).x4C;
            (*hit).state = HitCapsule_Unk2;
            (*it).anon_daa.xDAA_flag.anon_0.set_b2(1);
        }
        HitCapsule_Unk2 => {
            (*hit).state = HitCapsule_Unk3;
            (*hit).x58 = (*hit).x4C;
            ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
        }
        HitCapsule_Unk3 => {
            (*hit).x58 = (*hit).x4C;
            ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
        }
        _ => {}
    }
}

pub unsafe extern "C" fn it_8027137C(arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8027137C");
    let mut index = 0usize;
    while index < 4 {
        let arg_item = (*arg_item_gobj).user_data as *mut Item;
        let hit = &raw mut (*arg_item).x5D4_hitboxes[index].hit;
        match (*hit).state {
            HitCapsule_Enabled => {
                ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
                (*hit).x58 = (*hit).x4C;
                (*hit).state = HitCapsule_Unk2;
                (*arg_item).anon_daa.xDAA_flag.anon_0.set_b2(1);
            }
            HitCapsule_Unk2 => {
                (*hit).state = HitCapsule_Unk3;
                (*hit).x58 = (*hit).x4C;
                ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
            }
            HitCapsule_Unk3 => {
                (*hit).x58 = (*hit).x4C;
                ext::lb_8000B1CC((*hit).jobj, &raw mut (*hit).b_offset, &raw mut (*hit).x4C);
            }
            _ => {}
        }
        index += 1;
    }
}

pub unsafe extern "C" fn it_8027146C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8027146C");
    let it = item(item_gobj);
    for i in 0..4usize {
        ext::lbColl_80008A5C(&raw mut (*it).x5D4_hitboxes[i].hit);
    }
}

pub unsafe extern "C" fn it_802714C0(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802714C0");
    let it = item(item_gobj);
    let mut index = 0usize;
    while index < (*it).xAC8_hurtboxNum as usize {
        (*it).xACC_itemHurtbox[index].set_skip_update_pos(0);
        index += 1;
    }
    (*it).anon_daa.xDAA_flag.anon_0.set_b1(1);
}

pub unsafe extern "C" fn it_80271508(item_gobj: *mut HSD_GObj, state: HurtCapsuleState) {
    melee_core::ct!("it_80271508");
    let it = item(item_gobj);
    let mut index = 0usize;
    while index < (*it).xAC8_hurtboxNum as usize {
        (*it).xACC_itemHurtbox[index].state = state;
        index += 1;
    }
}

pub unsafe extern "C" fn it_80271534(item_gobj: *mut HSD_GObj, index: i32, arg_hurt: *mut HurtCapsule) {
    melee_core::ct!("it_80271534");
    let it = item(item_gobj);
    if !(*(*it).xC4_article_data).x8_hurtbones.is_null() {
        let hurt = &raw mut (*it).xACC_itemHurtbox[index as usize];
        (*hurt).a_offset = (*arg_hurt).a_offset;
        (*hurt).b_offset = (*arg_hurt).b_offset;
        (*hurt).scale = (*arg_hurt).scale;
    }
}

pub unsafe extern "C" fn it_80271590(item_gobj: *mut HSD_GObj, index: i32, arg_hurt: *mut HurtCapsule) {
    melee_core::ct!("it_80271590");
    let it = item(item_gobj);
    if !(*(*it).xC4_article_data).x8_hurtbones.is_null() {
        let hurt = &raw mut (*it).xACC_itemHurtbox[index as usize];
        (*arg_hurt).a_offset = (*hurt).a_offset;
        (*arg_hurt).b_offset = (*hurt).b_offset;
        (*arg_hurt).scale = (*hurt).scale;
        return;
    }
    let sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*arg_hurt).a_offset = sp18;
    (*arg_hurt).b_offset = sp18;
    (*arg_hurt).scale = 0.0;
}

pub unsafe extern "C" fn it_8027163C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8027163C");
    let it = (*item_gobj).user_data as *mut Item;
    let article = (*it).xC4_article_data;
    let it_hurtbox = (*article).x8_hurtbones;
    let it_dynams = (*article).x14_dynamics as *mut ItCollDynamics;
    if !it_hurtbox.is_null() {
        if (*it_hurtbox).count > 2 {
            panic!("item hit num over! (HSD_ASSERTREPORT(0x3F4))");
        }
        let mut cnt: u32 = 0;
        (*it).xAC8_hurtboxNum = (*it_hurtbox).count as u8;
        let mut index = 0usize;
        while cnt < (*it_hurtbox).count as u32 {
            let hurt = &raw mut (*it).xACC_itemHurtbox[index];
            let hurt_dyn_desc = (*it_hurtbox).descs.add(index);
            (*it).xACC_itemHurtbox[index].state = HurtCapsule_Enabled;
            if (*hurt_dyn_desc).bone_id != 0 {
                if (*it).xBBC_dynamicBoneTable.is_null() {
                    panic!("item can't init hit! (HSD_ASSERTREPORT(0x402))");
                }
                (*hurt).bone = (*(*it).xBBC_dynamicBoneTable).bones[(*hurt_dyn_desc).bone_id as usize];
            } else {
                (*hurt).bone = (*item_gobj).hsd_obj as *mut HSD_JObj;
            }
            index += 1;
            cnt += 1;
            (*hurt).a_offset = (*hurt_dyn_desc).a_offset;
            (*hurt).b_offset = (*hurt_dyn_desc).b_offset;
            (*hurt).scale = (*hurt_dyn_desc).scale;
        }
    } else {
        (*it).xAC8_hurtboxNum = 0;
    }
    if !it_dynams.is_null() {
        if (*it_dynams).count > 2 {
            panic!("item dynamics hit num over! (HSD_ASSERTREPORT(0x415)) kind {} count {} ptr {:p}", (*it).kind, (*it_dynams).count, it_dynams);
        }
        let mut cnt: u32 = 0;
        (*it).xB68 = (*it_dynams).count as u8;
        let mut index = 0usize;
        while cnt < (*it_dynams).count as u32 {
            let vars = &raw mut (*it).xB6C_vars[cnt as usize];
            let bone_dyn_desc = (*it_dynams).descs.add(index);
            (*vars).xB90 = (*bone_dyn_desc).bone_id as u32;
            (*vars).xB7C = (*(*it).xBBC_dynamicBoneTable).bones[(*bone_dyn_desc).bone_id as usize];
            (*vars).xB6C = (*bone_dyn_desc).offset;
            (*vars).xB78 = (*bone_dyn_desc).size;
            index += 1;
            cnt += 1;
        }
    }
}

pub unsafe extern "C" fn it_80271830(it: *mut Item, mut angle: f32) {
    melee_core::ct!("it_80271830");
    let mut sp68 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp5C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp50 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp44 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp20 = Vec3 { x: it_803B8560.x, y: it_803B8560.y, z: it_803B8560.z };
    while angle < 0.0 {
        angle = (angle as f64 + M_TAU) as f32;
    }
    while (angle as f64) > M_TAU {
        angle = (angle as f64 - M_TAU) as f32;
    }
    sp68.y = (*it).xBEC.top;
    sp68.z = 0.0;
    sp68.x = 0.0;
    ext::lbVector_RotateAboutUnitAxis(&raw mut sp68, &raw mut sp20, angle);
    sp5C.y = (*it).xBEC.bottom;
    sp5C.z = 0.0;
    sp5C.x = 0.0;
    ext::lbVector_RotateAboutUnitAxis(&raw mut sp5C, &raw mut sp20, angle);
    sp50.x = (*it).xBEC.right;
    sp50.z = 0.0;
    sp50.y = 0.0;
    ext::lbVector_RotateAboutUnitAxis(&raw mut sp50, &raw mut sp20, angle);
    sp44.x = (*it).xBEC.left;
    sp44.z = 0.0;
    sp44.y = 0.0;
    ext::lbVector_RotateAboutUnitAxis(&raw mut sp44, &raw mut sp20, angle);
    let mut left_pos = if sp68.x > sp5C.x { sp68.x } else { sp5C.x };
    left_pos = if left_pos > sp50.x { left_pos } else { sp50.x };
    left_pos = if left_pos > sp44.x { left_pos } else { sp44.x };
    (*it).xBEC.left = left_pos;
    let mut top_pos = if sp68.y > sp5C.y { sp68.y } else { sp5C.y };
    top_pos = if top_pos > sp50.y { top_pos } else { sp50.y };
    top_pos = if top_pos > sp44.y { top_pos } else { sp44.y };
    (*it).xBEC.top = top_pos;
    let mut right_pos = if sp68.x < sp5C.x { sp68.x } else { sp5C.x };
    right_pos = if right_pos < sp50.x { right_pos } else { sp50.x };
    right_pos = if right_pos < sp44.x { right_pos } else { sp44.x };
    (*it).xBEC.right = right_pos;
    let mut bottom_pos = if sp68.y < sp5C.y { sp68.y } else { sp5C.y };
    bottom_pos = if bottom_pos < sp50.y { bottom_pos } else { sp50.y };
    bottom_pos = if bottom_pos < sp44.y { bottom_pos } else { sp44.y };
    (*it).xBEC.bottom = bottom_pos;
}

pub unsafe extern "C" fn it_80271A58(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80271A58");
    let it = item(item_gobj);
    (*it).xBEC = (*it).xBDC;
    if (*it).facing_dir == 1.0 && (*it).xDC8_word.flags.x19() == 1 {
        let temp = -(*it).xBEC.right;
        (*it).xBEC.right = -(*it).xBEC.left;
        (*it).xBEC.left = temp;
    }
    let jobj = ext::it_802746F8(item_gobj);
    let rotate = if (*it).xDC8_word.flags.x17() == 0 {
        HSD_JObjGetRotationZ(jobj)
    } else if (*it).xDC8_word.flags.x17() == 1 {
        HSD_JObjGetRotationX(jobj)
    } else {
        HSD_JObjGetRotationY(jobj)
    };
    it_80271830(it, rotate);
}

pub unsafe extern "C" fn it_80271B60(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80271B60");
    let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    let it = item(item_gobj);
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::Item_804A0CCC.x154.anon_0.b0() != 1 {
        HSD_JObjGetTranslation(item_jobj, &raw mut sp24);
        let mut cnt = 0usize;
        while cnt < ext::Item_804A0CCC.count {
            let ecb = &raw mut ext::Item_804A0CCC.ecb_offset_arr[cnt];
            let y_pos = ext::Item_804A0CCC.ft_pos_arr[cnt].y;
            let x_pos = ext::Item_804A0CCC.ft_pos_arr[cnt].x;
            if itColl_chkECBOverlap(x_pos, y_pos, &raw mut (*it).xBEC, ecb, &raw mut sp24) != 0 {
                let dir;
                if abs_c(sp24.x - x_pos) < 0.001 {
                    dir = if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
                } else if sp24.x - x_pos < 0.0 {
                    dir = -1.0;
                } else {
                    dir = 1.0;
                }
                (*it).x70_nudge.x = (*ext::it_804D6D28).x7C_float * dir;
                (*it).xDC0 |= 1;
            }
            cnt += 1;
        }
    }
}

pub unsafe extern "C" fn it_80271D2C(arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80271D2C");
    let arg_item_jobj = (*arg_item_gobj).hsd_obj as *mut HSD_JObj;
    let arg_item = item(arg_item_gobj);
    let mut sp34 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    HSD_JObjGetTranslation(arg_item_jobj, &raw mut sp34);
    let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !item_gobj.is_null() {
        let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        let it = item(item_gobj);
        if arg_item_gobj != item_gobj
            && (*it).xDC8_word.flags.x13() == 0
            && (*it).ground_or_air == GA_Ground
            && (*it).xDD1_flag.anon_0.b0() == 0
            && ((*it).hold_kind != 3 || ((*it).hold_kind == 3 && (*arg_item).xDC8_word.flags.x1E() != 0))
        {
            HSD_JObjGetTranslation(item_jobj, &raw mut sp28);
            if itColl_chkECBOverlap(sp28.x, sp28.y, &raw mut (*arg_item).xBEC, &raw mut (*it).xBEC, &raw mut sp34) != 0 {
                let dir;
                if abs_c(sp34.x - sp28.x) < 0.001 {
                    if (*it).xDC8_word.flags.x1B() == 0 {
                        dir = if (*it).x70_nudge.x < 0.0 { 1.0 } else { -1.0 };
                    } else if ext::HSD_Randi(2) != 0 {
                        dir = 1.0;
                    } else {
                        dir = -1.0;
                    }
                } else if sp34.x - sp28.x < 0.0 {
                    dir = -1.0;
                } else {
                    dir = 1.0;
                }
                (*arg_item).x70_nudge.x = (*ext::it_804D6D28).x7C_float * dir;
                (*arg_item).xDC0 |= 2;
            }
        }
        item_gobj = (*item_gobj).next;
    }
}

pub unsafe extern "C" fn it_80271F78(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80271F78");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let arg_item = item(gobj);
    let mut sp34 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    HSD_JObjGetTranslation(jobj, &raw mut sp34);
    let mut item_gobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !item_gobj.is_null() {
        let item_jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
        let it = item(item_gobj);
        if gobj != item_gobj
            && (*it).xDC8_word.flags.x13() == 0
            && (*it).ground_or_air == GA_Ground
            && (*it).xDD1_flag.anon_0.b0() == 0
            && ext::itIsHeavy(item_gobj) == 1
        {
            HSD_JObjGetTranslation(item_jobj, &raw mut sp28);
            if itColl_chkECBOverlap(sp28.x, sp28.y, &raw mut (*arg_item).xBEC, &raw mut (*it).xBEC, &raw mut sp34) != 0 {
                let dir;
                if abs_c(sp34.x - sp28.x) < 0.001 {
                    if (*it).xDC8_word.flags.x1B() == 0 {
                        dir = if (*it).x70_nudge.x < 0.0 { 1.0 } else { -1.0 };
                    } else if ext::HSD_Randi(2) != 0 {
                        dir = 1.0;
                    } else {
                        dir = -1.0;
                    }
                } else if sp34.x - sp28.x < 0.0 {
                    dir = -1.0;
                } else {
                    dir = 1.0;
                }
                (*arg_item).x70_nudge.x = (*ext::it_804D6D28).x7C_float * dir;
                (*arg_item).xDC0 |= 2;
            }
        }
        item_gobj = (*item_gobj).next;
    }
}

pub unsafe extern "C" fn it_802721B8(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802721B8");
    let it = (*item_gobj).user_data as *mut Item;
    (*it).x70_nudge.z = 0.0;
    (*it).x70_nudge.y = 0.0;
    (*it).x70_nudge.x = 0.0;
    (*it).xDC0 = 0;
    if (*it).xDC8_word.flags.x13() == 0 {
        if (*it).xDC8_word.flags.x1A() == 1 && (*it).ground_or_air == GA_Ground {
            if (*it).xDC8_word.flags.x1C() != 0 {
                it_80271B60(item_gobj);
            }
            if (*it).xDC8_word.flags.x1D() != 0 {
                it_80271D2C(item_gobj);
            }
        }
    }
    if (*it).xDC8_word.flags.x13() == 0 && (*it).ground_or_air == GA_Ground && ext::itIsHeavy(item_gobj) == 1 {
        it_80271F78(item_gobj);
    }
}

pub unsafe extern "C" fn it_80272280(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80272280");
    (*((*item_gobj).user_data as *mut Item)).xDC8_word.flags.set_x1B(0);
}

pub unsafe extern "C" fn it_80272298(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80272298");
    (*((*item_gobj).user_data as *mut Item)).xDC8_word.flags.set_x1B(1);
}

pub unsafe extern "C" fn it_802722B0(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802722B0");
    if item_gobj == *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize) {
        ext::ftCo_80098634(&raw mut ext::Item_804A0CCC);
        ext::Item_804A0CCC.x154.anon_0.set_b0(0);
    }
}

pub unsafe extern "C" fn it_80272304(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80272304");
    let it = item(item_gobj);
    let mut i = 0u32;
    while i < (*it).xB68 as u32 {
        let tmp = &raw mut (*it).xB6C_vars[i as usize];
        ext::lb_8000B1CC((*tmp).xB7C, &raw mut (*tmp).xB6C, &raw mut (*tmp).xB84);
        i += 1;
    }
}

pub unsafe extern "C" fn it_8027236C(arg_item_gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("it_8027236C");
    let arg_item = (*arg_item_gobj).user_data as *mut Item;
    let fighter_gobj = (*arg_item).xCEC_fighterGObj;
    if !fighter_gobj.is_null() {
        (*arg_item).owner = fighter_gobj;
        (*arg_item).x20_team_id = ext::ftLib_80086EB4((*arg_item).xCEC_fighterGObj) as u8;
    } else {
        let unk_owner_gobj = (*arg_item).xCF0_itemGObj;
        if !unk_owner_gobj.is_null() {
            (*arg_item).owner = unk_owner_gobj;
            (*arg_item).x20_team_id = ext::ftLib_80086EB4((*arg_item).xCF0_itemGObj) as u8;
        } else {
            (*arg_item).owner = null_mut();
            (*arg_item).x20_team_id = u8::MAX;
        }
    }
    ext::it_8027B1F4(arg_item_gobj);
    (*arg_item).owner
}

pub unsafe extern "C" fn it_802723FC(arg_item_gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("it_802723FC");
    let arg_item = (*arg_item_gobj).user_data as *mut Item;
    if ext::ftLib_80086960((*arg_item).xCFC) != 0 {
        (*arg_item).owner = (*arg_item).xCFC;
        (*arg_item).x20_team_id = ext::ftLib_80086EB4((*arg_item).owner) as u8;
    } else {
        (*arg_item).owner = null_mut();
        (*arg_item).x20_team_id = u8::MAX;
    }
    (*arg_item).owner
}

pub unsafe extern "C" fn it_80272460(hitbox: *mut HitCapsule, damage: u32, arg_item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80272460");
    let mut dmg = damage;
    let arg_item = item(arg_item_gobj);
    let owner_gobj = (*arg_item).owner;
    if ext::ftLib_80086960(owner_gobj) != 0 {
        let owner = (*owner_gobj).user_data as *mut Fighter;
        if (*owner).x34_scale.y != 1.0 {
            let k = ext::ftCo_CalcYScaledKnockback(dmg as f32, (*owner).x34_scale.y, (*ext::Fighter_804D6524).x4);
            dmg = melee_runtime::cabi::f2u32((0.999f32 + k) as f64);
        }
        (*hitbox).unk_count = dmg;
        (*hitbox).damage = ext::ft_80089228(owner, (*arg_item).xD88_attackID, (*arg_item).xD8C_attack_instance as i32, (*hitbox).unk_count as f32);
        ext::ScriptGame_HitRuleDamage(hitbox);
        return;
    }
    (*hitbox).unk_count = dmg;
    (*hitbox).damage = dmg as f32;
    ext::ScriptGame_HitRuleDamage(hitbox);
}
