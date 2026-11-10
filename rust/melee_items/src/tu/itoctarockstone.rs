//! Port of src/melee/it/kinds/itoctarockstone.c: the stone an Octorok spits. Callees of unported TUs go through
//! `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itoctarockstone.rs.
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
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn ftLib_FindNearestOpponent(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_GetCameraBonePos(a0: *mut HSD_GObj, a1: *mut Vec);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026DFB0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DFB0(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80275474(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80275474(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027781C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_8027781C(a0) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `ItemStateTable it_803F8E90[]`
pub static mut it_803F8E90: [ItemStateTable; 2] = [
    st_row(0, pred_b!(itOctarockstone_UnkMotion0_Anim), Some(itOctarockstone_UnkMotion0_Phys), pred_b!(itOctarockstone_UnkMotion0_Coll)),
    st_row(0, pred_b!(itOctarockstone_UnkMotion1_Anim), Some(itOctarockstone_UnkMotion1_Phys), pred_b!(itOctarockstone_UnkMotion1_Coll)),
];

pub unsafe extern "C" fn itOctarockstone_802E878C(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarockstone_802E878C");
    let ip = GET_ITEM(gobj);
    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_80275474(gobj);
    itOctarockstone_802E883C(gobj);
}

pub unsafe extern "C" fn itOctarockstone_Logic4_DmgDealt(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_DmgDealt");
    true
}

pub unsafe extern "C" fn itOctarockstone_Logic4_Clanked(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_Clanked");
    true
}

pub unsafe extern "C" fn itOctarockstone_Logic4_HitShield(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_HitShield");
    true
}

pub unsafe extern "C" fn itOctarockstone_Logic4_Absorbed(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_Absorbed");
    true
}

pub unsafe extern "C" fn itOctarockstone_Logic4_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_Reflected");
    ext::it_80273030(gobj);
    false
}

pub unsafe extern "C" fn itOctarockstone_Logic4_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_Logic4_ShieldBounced");
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn itOctarockstone_802E883C(gobj: *mut HSD_GObj) {
    melee_core::ct!("itOctarockstone_802E883C");
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itOctarockstone_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_UnkMotion0_Anim");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    false
}

pub unsafe extern "C" fn itOctarockstone_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itOctarockstone_UnkMotion0_Phys");}

pub unsafe extern "C" fn itOctarockstone_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_UnkMotion0_Coll");
    let ip = GET_ITEM(gobj);

    let coll = ext::it_8026DAA8(gobj);
    if coll & 0xF != 0 {
        ext::it_8027781C(gobj);
        if coll & 0x1 != 0 {
            ext::it_802762B0(ip);
        }
        itOctarockstone_802E890C(gobj);
    }
    false
}

pub unsafe extern "C" fn itOctarockstone_802E890C(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarockstone_802E890C");
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::Item_8026AE84(ip, 0x138, 0x7F, 0x40);
}

pub unsafe extern "C" fn itOctarockstone_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itOctarockstone_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarockstone_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.y -= (*(*ip).xCC_item_attr).x10_fall_speed;
}

pub unsafe extern "C" fn itOctarockstone_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itOctarockstone_UnkMotion1_Coll");
    ext::it_8026DFB0(gobj) != 0
}

pub unsafe extern "C" fn itOctarockstone_802E89B0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itOctarockstone_802E89B0");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `getX` (static inline)
#[inline]
fn getX(x1: f32, x2: f32) -> f32 {
    let res = x1 - x2;
    if res < 0.0 {
        return -res;
    }
    res
}

pub unsafe extern "C" fn it_802E89D0(gobj: *mut Item_GObj, horiz_speed: f32, min_vy: f32, max_vy: f32) {
    melee_core::ct!("it_802E89D0");
    let ip = GET_ITEM(gobj);
    let mut pos: Vec3 = core::mem::zeroed();

    (*ip).x40_vel.x = horiz_speed * (*ip).facing_dir;
    (*ip).x40_vel.z = 0.0;

    let gp = ext::ftLib_FindNearestOpponent(&mut (*ip).pos, null_mut());
    if !gp.is_null() {
        ext::ftLib_GetCameraBonePos(gp, &mut pos);
        let mut x = getX((*ip).pos.x, pos.x);
        let speed = x / horiz_speed;
        let new_var = (*(*ip).xCC_item_attr).x10_fall_speed * (speed * speed);
        x = new_var;
        x *= 0.5;
        (*ip).x40_vel.y = x / speed;
        let r = ext::HSD_Randf();
        (*ip).x40_vel.y = fmadds(r, (pos.y - (*ip).pos.y) / 10.0, (*ip).x40_vel.y);

        if (*ip).x40_vel.y > max_vy {
            (*ip).x40_vel.y = max_vy;
        }
        if (*ip).x40_vel.y < min_vy {
            (*ip).x40_vel.y = min_vy;
        }
        return;
    }
    (*ip).x40_vel.y = 0.0;
}

/// Spawns an Octarock stone projectile.
pub unsafe extern "C" fn it_802E8ADC(gobj: *mut Item_GObj, pos: *mut Vec3, dir: i32, horiz_speed: f32, min_vy: f32, max_vy: f32) {
    melee_core::ct!("it_802E8ADC");
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Octarock_Stone;
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = dir as f32;
    spawn.x3C_damage = 0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.x0.set_b0(1);
    spawn.x40 = 0;
    let stone_gobj = ext::Item_80268B18(&mut spawn);
    if !stone_gobj.is_null() {
        it_802E89D0(stone_gobj, horiz_speed, min_vy, max_vy);
    }
}
