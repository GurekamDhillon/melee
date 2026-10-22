//! Port of src/melee/it/ithitbox.c: the item hitbox / hurtbox flag and scale setters (`xDCD`/`xDCE` flag
//! bytes, the four `x5D4_hitboxes`), plus the per-frame hitbox position bookkeeping that calls into
//! the unported collision code. Callees of other TUs go through `ext`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn ScriptGame_HitRuleBaseDamage(a0: *mut HitCapsule) -> f32;
    fn it_802714C0(a0: *mut HSD_GObj);
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj);
    fn it_80274D04(a0: *mut HSD_GObj, a1: *mut lb_80014638_arg1_t);
    fn it_80274D6C(a0: *mut HSD_GObj);
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
    let item = item(item_gobj);
    (*item).xC34_damageDealt = 0;
    (*item).xC40 = 1.0;
    (*item).xC44 = 1.0;
    (*item).xC3C = 1.0;
    (*item).xC48 = 0;
    (*item).xC4C = 0;
    (*item).xC50 = 0;
    (*item).xDCE_flag.anon_0.set_b6(0);
}

pub unsafe extern "C" fn it_80275414(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xDCD_flag.anon_0.set_b7(1);
}

pub unsafe extern "C" fn it_8027542C(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xDCD_flag.anon_0.set_b7(0);
}

pub unsafe extern "C" fn it_80275444(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    (*item).xDCD_flag.anon_0.set_b5(1);
    (*item).xDCD_flag.anon_0.set_b7(1);
    (*item).xDCD_flag.anon_0.set_b6(1);
}

pub unsafe extern "C" fn it_80275474(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    (*item).xDCD_flag.anon_0.set_b5(0);
    (*item).xDCD_flag.anon_0.set_b7(0);
    (*item).xDCD_flag.anon_0.set_b6(0);
}

pub unsafe extern "C" fn it_802754A4(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xDCE_flag.anon_0.set_b2(1);
}

pub unsafe extern "C" fn it_802754BC(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xDCE_flag.anon_0.set_b2(0);
}

pub unsafe extern "C" fn it_802754D4(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    (*item).xDCE_flag.anon_0.set_b0(1);
    (*item).xDCE_flag.anon_0.set_b2(1);
    (*item).xDCE_flag.anon_0.set_b1(1);
}

pub unsafe extern "C" fn it_80275504(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    (*item).xDCE_flag.anon_0.set_b0(0);
    (*item).xDCE_flag.anon_0.set_b2(0);
    (*item).xDCE_flag.anon_0.set_b1(0);
}

pub unsafe extern "C" fn it_80275534(item_gobj: *mut HSD_GObj, scale: f32) {
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
    let item = item(item_gobj);
    let hitcapsule = &mut (*item).x5D4_hitboxes[idx as usize].hit;
    if hitcapsule.state != HitCapsule_Disabled {
        hitcapsule.scale *= mult;
    }
}

pub unsafe extern "C" fn it_802755C0(item_gobj: *mut HSD_GObj, scale: f32) {
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
    let item = item(item_gobj);
    let mut i = 0usize;
    while i < 4 {
        let hitcapsule: *mut HitCapsule = &mut (*item).x5D4_hitboxes[i].hit;
        if (*hitcapsule).state != HitCapsule_Disabled {
            // ScriptGame_HitRuleBaseDamage(hit) * arg1, converted to u32 (the C (u32) cast of a float: saturating)
            let d = ext::ScriptGame_HitRuleBaseDamage(hitcapsule) * arg1;
            ext::it_80272460(hitcapsule, d as u32, item_gobj);
        }
        i += 1;
    }
}

pub unsafe extern "C" fn it_802756D0(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xD0C = 2;
}

pub unsafe extern "C" fn it_802756E0(item_gobj: *mut HSD_GObj) {
    (*item(item_gobj)).xD0C = 0;
    ext::it_802714C0(item_gobj);
}

pub unsafe extern "C" fn it_8027570C(item_gobj: *mut HSD_GObj, idx: i32) {
    (*item(item_gobj)).x5D4_hitboxes[idx as usize].hit.set_x40_b0(0);
}

pub unsafe extern "C" fn it_8027572C(item_gobj: *mut HSD_GObj, idx: i32) {
    (*item(item_gobj)).x5D4_hitboxes[idx as usize].hit.set_x40_b0(1);
}

pub unsafe extern "C" fn it_8027574C(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    ext::it_80274D04(item_gobj, &raw mut (*item).xB54);
    ext::it_80274D6C(item_gobj);
}

pub unsafe extern "C" fn it_80275788(item_gobj: *mut HSD_GObj) {
    let item = item(item_gobj);
    let state = HitCapsule_Unk4;
    let mut var_ctr = 0usize;
    while var_ctr < 4 {
        let hitcapsule = &mut (*item).x5D4_hitboxes[var_ctr].hit;
        if hitcapsule.state != HitCapsule_Disabled {
            hitcapsule.state = state;
            (*item).anon_daa.xDAA_flag.anon_0.set_b2(1);
        }
        var_ctr += 1;
    }
}

pub unsafe extern "C" fn it_80275820(item_gobj: *mut HSD_GObj, arg1: *mut Vec3, arg2: *mut Vec3, idx: i32) {
    let item = item(item_gobj);
    let hitcapsule = &mut (*item).x5D4_hitboxes[idx as usize].hit;
    if hitcapsule.state != HitCapsule_Disabled {
        hitcapsule.x58 = *arg2;
        hitcapsule.x4C = *arg1;
    }
}

pub unsafe extern "C" fn it_80275870(item_gobj: *mut HSD_GObj) -> i32 {
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
