//! Port of src/melee/it/kinds/itpeachturnip.c: Peach's vegetables (logic table 56), including the
//! TARGET_PC branch of the destroyed callback for m-ex custom articles. The weighted-odds table is
//! read through raw offsets because the C walks one entry past `x8[length - 1]`. Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping
//! contraction) by tests/kinds_itpeachturnip.rs.
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
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::item::Item_8026AB54(a0, a1, a2) };
    fn ftPe_SpecialLw_UnsetVeg(a0: *mut HSD_GObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BCF4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bcf4(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BD0C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd0c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80274484(a0: *mut HSD_GObj, a1: *mut HSD_JObj, a2: f32) => unsafe { crate::tu::it_2725::it_80274484(a0, a1, a2) };
    fn it_80274594(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274594(a0) };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_80276FC4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itmaplib::it_80276FC4(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `ItemStateTable it_803F74A8[]`
pub static mut it_803F74A8: [ItemStateTable; 5] = [
    st_row(1, pred_b(itPeachturnip_UnkMotion4_Anim), Some(itPeachturnip_UnkMotion4_Phys), None),
    st_row(-1, pred_b(itPeachturnip_UnkMotion1_Anim), Some(itPeachturnip_UnkMotion1_Phys), None),
    st_row(2, pred_b(itPeachturnip_UnkMotion3_Anim), Some(itPeachturnip_UnkMotion3_Phys), pred_b(itPeachturnip_UnkMotion3_Coll)),
    st_row(2, pred_b(itPeachturnip_UnkMotion3_Anim), Some(itPeachturnip_UnkMotion3_Phys), pred_b(itPeachturnip_UnkMotion3_Coll)),
    st_row(-1, pred_b(itPeachturnip_UnkMotion4_Anim), Some(itPeachturnip_UnkMotion4_Phys), None),
];

/// `attr->x8[i].x0_odds` (x8 starts at +8, 8 bytes per entry; `i` may run one past the array).
#[inline]
unsafe fn odds(attr: *mut itPeachTurnipAttributes, i: isize) -> i32 {
    *((attr as *const u8).offset(8 + 8 * i) as *const i32)
}

/// `attr->x8[i].x4_damage`
#[inline]
unsafe fn damage(attr: *mut itPeachTurnipAttributes, i: isize) -> i32 {
    *((attr as *const u8).offset(8 + 8 * i + 4) as *const i32)
}

pub unsafe extern "C" fn decrease_lifetimer(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("decrease_lifetimer");
    let item = GET_ITEM(item_gobj);
    (*item).xD44_lifeTimer -= 1.0;
    if (*item).xD44_lifeTimer <= 0.0 {
        return true;
    }
    false
}

/// Peach_TurnipTypeDecide - weighted random selection (2BD32C)
pub unsafe extern "C" fn it_802BD32C(item_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802BD32C");
    let item = GET_ITEM(item_gobj);
    let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPeachTurnipAttributes;

    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    while i < (*attr).x4_length {
        sum = sum.wrapping_add(odds(attr, i as isize));
        i += 1;
    }

    let rand = ext::HSD_Randi(sum);

    let mut cum_odds = odds(attr, 0);
    let tmp: i32 = -1;
    let mut ret = tmp;
    i = 0;
    while i < (*attr).x4_length && ret == tmp {
        if rand < cum_odds {
            ret = i;
        }
        cum_odds = cum_odds.wrapping_add(odds(attr, (i + 1) as isize));
        i += 1;
    }

    ret
}

/// 2BD45C
pub unsafe extern "C" fn it_802BD45C(item_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802BD45C");
    ext::Item_8026A8EC(item_gobj);
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_Destroyed(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachTurnip_Logic56_Destroyed");
    let item = GET_ITEM(item_gobj);
    // TARGET_PC: an m-ex custom article on Peach's turnip logic keeps other data in xDE4; the
    // thrower is item->owner.
    if (*item).kind != It_Kind_Peach_Turnip {
        if !(*item).owner.is_null() {
            ext::ftPe_SpecialLw_UnsetVeg((*item).owner);
        }
        return;
    }
    if !(*item).xDD4_itemVar.peachturnip.xDE4_owner.is_null() {
        ext::ftPe_SpecialLw_UnsetVeg((*item).xDD4_itemVar.peachturnip.xDE4_owner);
    }
}

/// Peach_SpawnAndGiveTurnip (2BD4AC)
pub unsafe extern "C" fn it_802BD4AC(item_gobj: *mut Item_GObj, pos: *mut Vec3, part: u32, kind: ItemKind, fdir: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802BD4AC");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    // Item_InitSpawnPosition(&spawn, pos, false)
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    // Item_InitSpawnCommonFields(&spawn, item_gobj, fdir, true)
    spawn.facing_dir = fdir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = item_gobj;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    let spawn_gobj = ext::Item_80268B18(&mut spawn);
    if !spawn_gobj.is_null() {
        let mut item = GET_ITEM(spawn_gobj);
        if kind == It_Kind_Peach_Turnip {
            let attr = (*(*item).xC4_article_data).x4_specialAttributes as *mut itPeachTurnipAttributes;
            ext::it_80275158(spawn_gobj, (*attr).x0_lifetime);
            (*item).xDD4_itemVar.peachturnip.xDD8 = it_802BD32C(spawn_gobj);
            (*item).xDD4_itemVar.peachturnip.xDDC_damage = damage(attr, (*item).xDD4_itemVar.peachturnip.xDD8 as isize);
            (*item).xDD4_itemVar.peachturnip.xDD4.anon_0.set_b0(0);
            (*item).xDD4_itemVar.peachturnip.xDE4_owner = (*item).owner;
            ext::it_8026BD0C(spawn_gobj);
        }

        ext::Item_8026AB54(spawn_gobj, item_gobj, part);
        if kind != It_Kind_Sword {
            ext::it_80274594(spawn_gobj);
            item = GET_ITEM(spawn_gobj);
            match (*item).kind {
                k if k == It_Kind_BombHei => {
                    (*item).xDD4_itemVar.peachturnip.xDE8_scl = (*item).scl;
                }
                k if k == It_Kind_Dosei => {
                    (*item).xDD4_itemVar.peachturnip.xDE0_scl = (*item).scl;
                }
                k if k == It_Kind_Peach_Turnip => {
                    (*item).xDD4_itemVar.peachturnip.xDE0_scl = (*item).scl;
                }
                _ => {}
            }
        }
    }
    spawn_gobj
}

/// ItemOnSpawn_Turnip
pub unsafe extern "C" fn itPeachTurnip_Logic56_PickedUp(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachTurnip_Logic56_PickedUp");
    let item = GET_ITEM(item_gobj);

    ext::it_80273670(item_gobj, 0, (*item).xDD4_itemVar.peachturnip.xDD8 as f32);
    if (*item).xDD4_itemVar.peachturnip.xDD4.anon_0.b0() != 0 {
        ext::Item_80268E5C(item_gobj, 4, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE);
        ext::it_8026BCF4(item_gobj);
    }
    (*item).xDD4_itemVar.peachturnip.xDD4.anon_0.set_b0(1);
    ext::Item_802694CC(item_gobj);
}

pub unsafe extern "C" fn itPeachturnip_UnkMotion4_Anim(_item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachturnip_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itPeachturnip_UnkMotion4_Phys(_item_gobj: *mut Item_GObj) { melee_core::ct!("itPeachturnip_UnkMotion4_Phys");}

pub unsafe extern "C" fn itPeachturnip_UnkMotion1_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachturnip_UnkMotion1_Anim");
    decrease_lifetimer(item_gobj)
}

pub unsafe extern "C" fn itPeachturnip_UnkMotion1_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachturnip_UnkMotion1_Phys");
    let temp_r4 = (*GET_ITEM(item_gobj)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_Thrown(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachTurnip_Logic56_Thrown");
    let item = GET_ITEM(item_gobj);
    let jobj = GET_JOBJ(item_gobj);

    ext::Item_80268E5C(item_gobj, 2, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
    ext::it_80273670(item_gobj, 0, (*item).xDD4_itemVar.peachturnip.xDD8 as f32);

    let hit = &raw mut (*item).x5D4_hitboxes[0].hit;
    ext::it_80272460(hit, (*item).xDD4_itemVar.peachturnip.xDDC_damage as u32, item_gobj);
    ext::it_80274484(item_gobj, jobj, (*item).xDD4_itemVar.peachturnip.xDE0_scl);
}

pub unsafe extern "C" fn itPeachturnip_UnkMotion3_Anim(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachturnip_UnkMotion3_Anim");
    decrease_lifetimer(item_gobj)
}

/// ProjectilePhysics_TurnipVelocity
pub unsafe extern "C" fn itPeachturnip_UnkMotion3_Phys(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachturnip_UnkMotion3_Phys");
    // Item_ApplyFallingPhysics(item_gobj)
    let attrs = (*GET_ITEM(item_gobj)).xCC_item_attr;
    ext::it_80272860(item_gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(item_gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itPeachturnip_UnkMotion3_Coll(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachturnip_UnkMotion3_Coll");
    let first_check: i32 = 0xE;
    let second_check: i32 = 0x3;

    let chk: i32 = ext::it_8026DAA8(item_gobj);
    if chk & first_check != 0 {
        ext::it_80276FC4(item_gobj, chk);
        ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    } else if chk & second_check != 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_Dropped(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itPeachTurnip_Logic56_Dropped");
    let item = GET_ITEM(item_gobj);
    ext::Item_80268E5C(item_gobj, 3, ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE);
    ext::it_80273670(item_gobj, 0, (*item).xDD4_itemVar.peachturnip.xDD8 as f32);
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_DmgDealt(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachTurnip_Logic56_DmgDealt");
    ext::itColl_BounceOffVictim(item_gobj);
    ext::Item_80268E5C(item_gobj, 1, ITEM_ANIM_UPDATE);
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_Clanked(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachTurnip_Logic56_Clanked");
    ext::itColl_BounceOffVictim(item_gobj);
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_Reflected(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachTurnip_Logic56_Reflected");
    ext::it_80273030(item_gobj);
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_HitShield(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachTurnip_Logic56_HitShield");
    ext::itColl_BounceOffVictim(item_gobj);
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_ShieldBounced(item_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itPeachTurnip_Logic56_ShieldBounced");
    ext::itColl_BounceOffShield(item_gobj);
    false
}

pub unsafe extern "C" fn itPeachTurnip_Logic56_EvtUnk(item_gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itPeachTurnip_Logic56_EvtUnk");
    let item = GET_ITEM(item_gobj);
    if (*item).xDD4_itemVar.peachturnip.xDE4_owner == ref_gobj {
        (*item).xDD4_itemVar.peachturnip.xDE4_owner = null_mut();
    }
    ext::it_8026B894(item_gobj, ref_gobj);
}
