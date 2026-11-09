//! Port of src/melee/it/ithitbox.c: the item hitbox / hurtbox flag and scale setters (`xDCD`/`xDCE` flag
//! bytes, the four `x5D4_hitboxes`), plus the per-frame hitbox position bookkeeping that calls into
//! the unported collision code. Callees of other TUs go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026bb88;
#[allow(unused_imports)]
use crate::itlib::it_8026bbcc;
#[allow(unused_imports)]
use crate::itlib::it_802714c0;
#[allow(unused_imports)]
use crate::itlib::it_8027346c;
#[allow(unused_imports)]
use crate::itlib::it_80273484;
#[allow(unused_imports)]
use crate::itlib::it_8027349c;
#[allow(unused_imports)]
use crate::itlib::it_802734b4;
#[allow(unused_imports)]
use crate::itlib::it_80275228;
#[allow(unused_imports)]
use crate::itlib::it_80275240;
#[allow(unused_imports)]
use crate::itlib::it_80275594 as it_80275594_itlib3;
#[allow(unused_imports)]
use crate::itlib::COLLIDE_FLOOR_MASK;
#[allow(unused_imports)]
use crate::layout::it_2728_DatAttrs;
#[allow(unused_imports)]
use crate::layout::it_804D6D40_t;
#[allow(unused_imports)]
use crate::layout::itPokemonSpawn_DatAttrs;
#[allow(unused_imports)]
use crate::layout::CommandInfo;
#[allow(unused_imports)]
use crate::layout::Fighter;
#[allow(unused_imports)]
use crate::layout::HitCapsule;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use crate::layout::ItemStateDesc;
#[allow(unused_imports)]
use crate::layout::PokemonSelectionState;
#[allow(unused_imports)]
use crate::layout::lb_80014638_arg1_t;
#[allow(unused_imports)]
use crate::layout::HSD_GOBJ_CLASS_ITEM;
#[allow(unused_imports)]
use crate::layout::It_PKind_Marumine;
#[allow(unused_imports)]
use crate::layout::It_PKind_Start;
#[allow(unused_imports)]
use crate::layout::It_PKind_Terminate;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itlib3;
#[allow(unused_imports)]
use crate::layout::CollData;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Disabled;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Unk4;
#[allow(unused_imports)]
use crate::layout::HurtCapsuleState;
#[allow(unused_imports)]
use crate::layout::ItemKind;
#[allow(unused_imports)]
use crate::layout::Collide_FloorHug;
#[allow(unused_imports)]
use crate::layout::Collide_FloorPush;
#[allow(unused_imports)]
use crate::layout::Collide_LeftEdge;
#[allow(unused_imports)]
use crate::layout::Collide_RightEdge;
#[allow(unused_imports)]
use crate::layout::It_Kind_Arwing_Laser;
#[allow(unused_imports)]
use crate::layout::It_Kind_BombHei;
#[allow(unused_imports)]
use crate::layout::It_Kind_Coin;
#[allow(unused_imports)]
use crate::layout::It_Kind_F_Flower;
#[allow(unused_imports)]
use crate::layout::It_Kind_Foods;
#[allow(unused_imports)]
use crate::layout::It_Kind_Heart;
#[allow(unused_imports)]
use crate::layout::It_Kind_Kuriboh;
#[allow(unused_imports)]
use crate::layout::It_Kind_Kyasarin_Egg;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun;
#[allow(unused_imports)]
use crate::layout::It_Kind_Link_Bomb;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lucky_Egg;
#[allow(unused_imports)]
use crate::layout::It_Kind_Octarock_Stone;
#[allow(unused_imports)]
use crate::layout::It_Kind_Old_Kuri;
#[allow(unused_imports)]
use crate::layout::It_Kind_S_Scope;
#[allow(unused_imports)]
use crate::layout::It_Kind_StarRod;
#[allow(unused_imports)]
use crate::layout::It_Kind_Tomato;
#[allow(unused_imports)]
use crate::layout::It_Kind_WhispyApple;
#[allow(unused_imports)]
use crate::layout::It_Kind_WhispyHealApple;
#[allow(unused_imports)]
use crate::helpers::COLLIDE_CEILING_MASK;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itlib;
#[allow(unused_imports)]
use crate::helpers::it_link_lerp;
#[allow(unused_imports)]
use crate::itlib::it_80271534;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80274d6c;
#[allow(unused_imports)]
use crate::itlib::it_80274ecc;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_80274d04;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itkinds::itGShell_Attrs;
#[allow(unused_imports)]
use crate::layout::ftCollisionBox;
#[allow(unused_imports)]
use crate::layout::itLinkHookshotAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::itSword_UnkArticle1;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itToolsAttributes;
#[allow(unused_imports)]
use crate::layout::itUnkAttributes_legacy;
#[allow(unused_imports)]
use crate::layout::itWstarAttributes;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HurtCapsule;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds6;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

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
    fn ScriptGame_HitRuleBaseDamage(a0: *mut HitCapsule) -> f32;
    fn it_802714C0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802714c0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_80274D04(a0: *mut HSD_GObj, a1: *mut lb_80014638_arg1_t) => unsafe { crate::itlib3::it_80274d04(&*((*a0).user_data as *mut Item), &mut *a1) };
    fn it_80274D6C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274d6c(&mut *((*a0).user_data as *mut Item)) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline(always)]
unsafe fn item(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

pub unsafe extern "C" fn it_802753DC(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802753DC");
    let item = item(item_gobj);
    (*item).xC34_damageDealt = 0;
    (*item).xC40 = 1.0;
    (*item).xC44 = 1.0;
    (*item).xC3C = 1.0;
    (*item).xC48 = 0;
    (*item).xC4C = 0;
    (*item).xC50 = 0;
    (*item).xDCE_flag.x0.set_b6(0);
}

pub unsafe extern "C" fn it_80275414(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275414");
    (*item(item_gobj)).xDCD_flag.x0.set_b7(1);
}

pub unsafe extern "C" fn it_8027542C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8027542C");
    (*item(item_gobj)).xDCD_flag.x0.set_b7(0);
}

pub unsafe extern "C" fn it_80275444(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275444");
    let item = item(item_gobj);
    (*item).xDCD_flag.x0.set_b5(1);
    (*item).xDCD_flag.x0.set_b7(1);
    (*item).xDCD_flag.x0.set_b6(1);
}

pub unsafe extern "C" fn it_80275474(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275474");
    let item = item(item_gobj);
    (*item).xDCD_flag.x0.set_b5(0);
    (*item).xDCD_flag.x0.set_b7(0);
    (*item).xDCD_flag.x0.set_b6(0);
}

pub unsafe extern "C" fn it_802754A4(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802754A4");
    (*item(item_gobj)).xDCE_flag.x0.set_b2(1);
}

pub unsafe extern "C" fn it_802754BC(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802754BC");
    (*item(item_gobj)).xDCE_flag.x0.set_b2(0);
}

pub unsafe extern "C" fn it_802754D4(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802754D4");
    let item = item(item_gobj);
    (*item).xDCE_flag.x0.set_b0(1);
    (*item).xDCE_flag.x0.set_b2(1);
    (*item).xDCE_flag.x0.set_b1(1);
}

pub unsafe extern "C" fn it_80275504(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275504");
    let item = item(item_gobj);
    (*item).xDCE_flag.x0.set_b0(0);
    (*item).xDCE_flag.x0.set_b2(0);
    (*item).xDCE_flag.x0.set_b1(0);
}

pub unsafe extern "C" fn it_80275534(item_gobj: *mut HSD_GObj, scale: f32) {
    melee_core::ct!("it_80275534");
    let item = item(item_gobj);
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        let hitcapsule = &mut (*item).x5D4_hitboxes[var_ctr].hit;
        if hitcapsule.state != HitCapsule_Disabled {
            hitcapsule.scale = scale;
        }
        var_ctr += 1;
    }
}

pub unsafe extern "C" fn it_80275594(item_gobj: *mut HSD_GObj, idx: i32, mult: f32) {
    melee_core::ct!("it_80275594");
    let item = item(item_gobj);
    let hitcapsule = &mut (*item).x5D4_hitboxes[idx as usize].hit;
    if hitcapsule.state != HitCapsule_Disabled {
        hitcapsule.scale *= mult;
    }
}

pub unsafe extern "C" fn it_802755C0(item_gobj: *mut HSD_GObj, scale: f32) {
    melee_core::ct!("it_802755C0");
    let item = item(item_gobj);
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        let hitcapsule = &mut (*item).x5D4_hitboxes[var_ctr].hit;
        if hitcapsule.state != HitCapsule_Disabled {
            hitcapsule.scale *= scale;
        }
        var_ctr += 1;
    }
}

pub unsafe extern "C" fn it_80275640(item_gobj: *mut HSD_GObj, arg1: f32) {
    melee_core::ct!("it_80275640");
    let item = item(item_gobj);
    let mut i = 0usize;
    while i < 4 {
        let hitcapsule: *mut HitCapsule = &mut (*item).x5D4_hitboxes[i].hit;
        if (*hitcapsule).state != HitCapsule_Disabled {
            // ScriptGame_HitRuleBaseDamage(hit) * arg1, converted to u32 (the C (u32) cast of a float: saturating)
            let d = ext::ScriptGame_HitRuleBaseDamage(hitcapsule) * arg1;
            ext::it_80272460(hitcapsule, melee_core::ppc::fcvt_u32(d), item_gobj);
        }
        i += 1;
    }
}

pub unsafe extern "C" fn it_802756D0(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802756D0");
    (*item(item_gobj)).xD0C = 2;
}

pub unsafe extern "C" fn it_802756E0(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802756E0");
    (*item(item_gobj)).xD0C = 0;
    ext::it_802714C0(item_gobj);
}

pub unsafe extern "C" fn it_8027570C(item_gobj: *mut HSD_GObj, idx: i32) {
    melee_core::ct!("it_8027570C");
    (*item(item_gobj)).x5D4_hitboxes[idx as usize].hit.set_x40_b0(0);
}

pub unsafe extern "C" fn it_8027572C(item_gobj: *mut HSD_GObj, idx: i32) {
    melee_core::ct!("it_8027572C");
    (*item(item_gobj)).x5D4_hitboxes[idx as usize].hit.set_x40_b0(1);
}

pub unsafe extern "C" fn it_8027574C(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8027574C");
    let item = item(item_gobj);
    ext::it_80274D04(item_gobj, &raw mut (*item).xB54);
    ext::it_80274D6C(item_gobj);
}

pub unsafe extern "C" fn it_80275788(item_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80275788");
    let item = item(item_gobj);
    let state = HitCapsule_Unk4;
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        let hitcapsule = &mut (*item).x5D4_hitboxes[var_ctr].hit;
        if hitcapsule.state != HitCapsule_Disabled {
            hitcapsule.state = state;
            (*item).xDAA.xDAA_flag.x0.set_b2(1);
        }
        var_ctr += 1;
    }
}

pub unsafe extern "C" fn it_80275820(item_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, idx: i32) {
    melee_core::ct!("it_80275820");
    let item = item(item_gobj);
    let hitcapsule = &mut (*item).x5D4_hitboxes[idx as usize].hit;
    if hitcapsule.state != HitCapsule_Disabled {
        hitcapsule.x58 = *arg2;
        hitcapsule.x4C = *arg1;
    }
}

pub unsafe extern "C" fn it_80275870(item_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("it_80275870");
    let item = item(item_gobj);
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        if (*item).x5D4_hitboxes[var_ctr].hit.state != HitCapsule_Disabled {
            return 1;
        }
        var_ctr += 1;
    }
    0
}

pub unsafe extern "C" fn it_802758D4(item_gobj: *mut HSD_GObj) -> f32 {
    melee_core::ct!("it_802758D4");
    let item = item(item_gobj);
    let mut damage = 0.0f32;
    let mut disable_chk = false;
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        let hitcapsule = &(*item).x5D4_hitboxes[var_ctr].hit;
        if hitcapsule.state != HitCapsule_Disabled {
            disable_chk = true;
            break;
        } else {
            disable_chk = false;
        }
        var_ctr += 1;
    }
    if disable_chk {
        var_ctr = 0;
        while var_ctr < 4 {
            let hitcapsule = &(*item).x5D4_hitboxes[var_ctr].hit;
            if hitcapsule.state != HitCapsule_Disabled {
                if damage <= hitcapsule.damage {
                    damage = hitcapsule.damage;
                }
            }
            var_ctr += 1;
        }
    }
    damage
}

// ---- folded from itkinds6.rs (consolidation steps 21/22) ----
// ---- item.c / ithitbox.c / maplib -----------------------------------------------------------

/// `it_8027574C` (ithitbox.c): `it_80274D04(gobj, &ip->xB54); it_80274D6C(gobj)`.
pub fn it_8027574c(ip: &mut Item) {
    melee_core::ct!("it_8027574c");
    let mut out = ip.xB54;
    it_80274d04(ip, &mut out);
    ip.xB54 = out;
    it_80274d6c(ip);
}

// ---- folded from itlib.rs (consolidation steps 21/22) ----
// ---- ithitbox.c ------------------------------------------------------------------------

/// `it_802753DC`.
pub fn it_802753dc(ip: &mut Item) {
    melee_core::ct!("it_802753dc");
    ip.xC34_damageDealt = 0;
    ip.xC40 = 1.0;
    ip.xC44 = 1.0;
    ip.xC3C = 1.0;
    ip.xC48 = 0;
    ip.xC4C = 0;
    ip.xC50 = 0;
    unsafe { ip.xDCE_flag.x0.set_b6(0) }
}

/// `it_80275414_fp`: `xDCD_flag.b7 = 1`.
pub fn it_80275414_fp(ip: &mut Item) {
    melee_core::ct!("it_80275414_fp");
    ip.xDCD_flag.x0.set_b7(1);
}

/// `it_8027542C`: `xDCD_flag.b7 = 0`.
pub fn it_8027542c(ip: &mut Item) {
    melee_core::ct!("it_8027542c");
    ip.xDCD_flag.x0.set_b7(0);
}

/// `it_80275444_fp`: `xDCD_flag.b5 = b7 = b6 = 1`.
pub fn it_80275444_fp(ip: &mut Item) {
    melee_core::ct!("it_80275444_fp");
    let f = &mut ip.xDCD_flag.x0;
    f.set_b5(1);
    f.set_b7(1);
    f.set_b6(1);
}

/// `it_80275474_fp`: `xDCD_flag.b5 = b7 = b6 = 0`.
pub fn it_80275474_fp(ip: &mut Item) {
    melee_core::ct!("it_80275474_fp");
    let f = &mut ip.xDCD_flag.x0;
    f.set_b5(0);
    f.set_b7(0);
    f.set_b6(0);
}

/// `it_802754A4`: `xDCE_flag.b2 = 1`.
pub fn it_802754a4(ip: &mut Item) {
    melee_core::ct!("it_802754a4");
    unsafe { ip.xDCE_flag.x0.set_b2(1) }
}

/// `it_802754BC`: `xDCE_flag.b2 = 0`.
pub fn it_802754bc(ip: &mut Item) {
    melee_core::ct!("it_802754bc");
    unsafe { ip.xDCE_flag.x0.set_b2(0) }
}

/// `it_802754D4`: `xDCE_flag.b0 = b2 = b1 = 1`.
pub fn it_802754d4(ip: &mut Item) {
    melee_core::ct!("it_802754d4");
    unsafe {
        let f = &mut ip.xDCE_flag.x0;
        f.set_b0(1);
        f.set_b2(1);
        f.set_b1(1);
    }
}

/// `it_80275504_fp`: `xDCE_flag.b0 = b2 = b1 = 0`.
pub fn it_80275504_fp(ip: &mut Item) {
    melee_core::ct!("it_80275504_fp");
    unsafe {
        let f = &mut ip.xDCE_flag.x0;
        f.set_b0(0);
        f.set_b2(0);
        f.set_b1(0);
    }
}

/// `it_80275534_fp`: set the scale of every enabled hitbox.
pub fn it_80275534_fp(ip: &mut Item, scale: f32) {
    melee_core::ct!("it_80275534_fp");
    for hb in ip.x5D4_hitboxes.iter_mut() {
        if hb.hit.state != HitCapsule_Disabled {
            hb.hit.scale = scale;
        }
    }
}

/// `it_80275594_fp`: multiply one hitbox's scale. Panics on an `idx` outside 0..4 (the C would
/// index past the array).
pub fn it_80275594_fp(ip: &mut Item, idx: i32, mult: f32) {
    melee_core::ct!("it_80275594_fp");
    let hit = &mut ip.x5D4_hitboxes[idx as usize].hit;
    if hit.state != HitCapsule_Disabled {
        hit.scale *= mult;
    }
}

/// `it_802755C0`: multiply the scale of every enabled hitbox.
pub fn it_802755c0(ip: &mut Item, scale: f32) {
    melee_core::ct!("it_802755c0");
    for hb in ip.x5D4_hitboxes.iter_mut() {
        if hb.hit.state != HitCapsule_Disabled {
            hb.hit.scale *= scale;
        }
    }
}

/// `it_802756D0`: `xD0C = 2`.
pub fn it_802756d0(ip: &mut Item) {
    melee_core::ct!("it_802756d0");
    ip.xD0C = 2;
}

/// `it_8027570C`: `x5D4_hitboxes[idx].hit.x40_b0 = 0`.
pub fn it_8027570c(ip: &mut Item, idx: i32) {
    melee_core::ct!("it_8027570c");
    ip.x5D4_hitboxes[idx as usize].hit.set_x40_b0(0);
}

/// `it_8027572C`: `x5D4_hitboxes[idx].hit.x40_b0 = 1`.
pub fn it_8027572c(ip: &mut Item, idx: i32) {
    melee_core::ct!("it_8027572c");
    ip.x5D4_hitboxes[idx as usize].hit.set_x40_b0(1);
}

/// `it_80275788_fp`: every enabled hitbox goes to `HitCapsule_Unk4`, and sets `xDAA_flag.b2`.
pub fn it_80275788_fp(ip: &mut Item) {
    melee_core::ct!("it_80275788_fp");
    for k in 0..4 {
        if ip.x5D4_hitboxes[k].hit.state != HitCapsule_Disabled {
            ip.x5D4_hitboxes[k].hit.state = HitCapsule_Unk4;
            unsafe { ip.xDAA.xDAA_flag.x0.set_b2(1) };
        }
    }
}

/// `it_80275820_fp`: set one enabled hitbox's `x58 = *arg2`, `x4C = *arg1`.
pub fn it_80275820_fp(ip: &mut Item, arg1: &GcVec, arg2: &GcVec, idx: i32) {
    melee_core::ct!("it_80275820_fp");
    let hit = &mut ip.x5D4_hitboxes[idx as usize].hit;
    if hit.state != HitCapsule_Disabled {
        hit.x58 = *arg2;
        hit.x4C = *arg1;
    }
}

/// `it_80275870_fp`: any hitbox enabled.
pub fn it_80275870_fp(ip: &Item) -> bool {
    melee_core::ct!("it_80275870_fp");
    ip.x5D4_hitboxes.iter().any(|hb| hb.hit.state != HitCapsule_Disabled)
}

/// `it_802758D4`: the largest damage of the enabled hitboxes (0.0 when none; `<=` keeps the
/// later of equal values, which only matters for signed zeros).
pub fn it_802758d4(ip: &Item) -> f32 {
    melee_core::ct!("it_802758d4");
    let mut damage = 0.0f32;
    if it_80275870_fp(ip) {
        for hb in ip.x5D4_hitboxes.iter() {
            if hb.hit.state != HitCapsule_Disabled && damage <= hb.hit.damage {
                damage = hb.hit.damage;
            }
        }
    }
    damage
}

// ---- folded from itlib3.rs (consolidation steps 21/22) ----
// ---- ithitbox.c ------------------------------------------------------------------------

/// `it_802756E0`: `xD0C = 0`, then `it_802714C0`.
pub fn it_802756e0(ip: &mut Item) {
    melee_core::ct!("it_802756e0");
    ip.xD0C = 0;
    it_802714c0(ip);
}
