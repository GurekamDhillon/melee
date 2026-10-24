//! Port of src/melee/it/kinds/itzeldadinfireexplode.c: Zelda's Din's Fire explosion (logic table 66). The
//! explosion grows by `zeldadinfireexplode.xDD4` (the charge, `scale` of the spawn); `xDD8` caches the hitbox's
//! first scale, `xDDC` the parent, `xDE0` the "alive" flag. Callees of unported TUs go through `ext` (unset:
//! panics "TODO"). Checked against the C (whole TU, shipping contraction) by tests/kinds_itzeldadinfireexplode.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2u32;
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_802694CC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_802694CC(a0) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efLib_DestroyAll(a0: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj) => unsafe { crate::tu::itcoll::it_80272460(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272940(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272940(a0) };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273598(a0: *mut HSD_GObj, a1: i32, a2: i32) => unsafe { crate::tu::it_2725::it_80273598(a0, a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
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

/// `ItemStateTable it_803F7740[]`
pub static mut it_803F7740: [ItemStateTable; 1] = [
    st_row(0, pred_b(itZeldadinfireexplode_UnkMotion0_Anim), Some(itZeldadinfireexplode_UnkMotion0_Phys), pred_b(itZeldadinfireexplode_UnkMotion0_Coll)),
];

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec3, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_ClearCmdVars` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearCmdVars(ip: *mut Item) {
    (*ip).xDB8_itcmd_var3 = 0;
    (*ip).xDB4_itcmd_var2 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDAC_itcmd_var0 = 0;
}

/// `Item_ClearFlagsAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
}

/// 2C4580: spawns the explosion at `pos` (z forced to 0).
pub unsafe extern "C" fn it_802C4580(parent_gobj: *mut Item_GObj, arg1: *mut HSD_GObj, pos: *mut Vec3, facing_dir: f32, scale: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Zelda_DinFire_Explode;
    Item_InitSpawnPosition(&mut spawn, pos, true);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = parent_gobj;
    spawn.x4_parent_gobj2 = arg1;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        let ip = GET_ITEM(gobj);
        Item_ClearCmdVars(ip);
        (*ip).xDD4_itemVar.zeldadinfireexplode.xDD4 = scale;
        (*ip).xDD4_itemVar.zeldadinfireexplode.xDDC = parent_gobj;
        it_802C46C4(gobj, ip);
    }
    gobj
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_Destroyed(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802725D4(gobj);
    (*ip).owner = null_mut();
    ext::efLib_DestroyAll(gobj);
    (*ip).xDD4_itemVar.zeldadinfireexplode.xDE0 = 0;
}

/// 2C46C4: starts the explosion; the camera quake depends on how charged it is.
pub unsafe extern "C" fn it_802C46C4(gobj: *mut Item_GObj, arg1: *mut Item) {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itZeldaDinFireExplodeAttributes;
    Item_ClearFlagsAndEnterState(gobj, ip, 0);
    ext::it_80275158(gobj, 60.0);
    (*ip).xDD4_itemVar.zeldadinfireexplode.xDD8 = 0.0;
    ext::efSync_Spawn(0x4FA, gobj, jobj);
    (*ip).xDD4_itemVar.zeldadinfireexplode.xDE0 = 1;
    ext::Item_802694CC(gobj);
    itZeldadinfireexplode_UnkMotion0_Anim(gobj);
    let temp_f1 = (*ip).xDD4_itemVar.zeldadinfireexplode.xDD4 / (*attrs).x0;
    if temp_f1 < 0.4375 {
        ext::it_80273598(gobj, 0xB, 0xA);
    } else if temp_f1 < 0.625 {
        ext::it_80273598(gobj, 0xC, 0xA);
    } else if temp_f1 < 0.8125 {
        ext::it_80273598(gobj, 0xD, 0xF);
        ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
    } else {
        ext::it_80273598(gobj, 0xE, 0x14);
        ext::Camera_RequestQuake(QuakeKind_Medium as u32, &mut (*ip).pos);
    }
    ext::db_80225DD8(gobj, (*ip).owner);
}

/// Scales the joint and the first hitbox with the charge.
pub unsafe extern "C" fn itZeldadinfireexplode_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut scale: Vec3 = core::mem::zeroed();

    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itZeldaDinFireExplodeAttributes;

    let mut temp_f1 = ((*attrs).x8 - (*attrs).x4) / (*attrs).x0;
    let s = fmadds((*ip).xDD4_itemVar.zeldadinfireexplode.xDD4, temp_f1, (*attrs).x4);
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(jobj, &mut scale);

    if (*ip).x5D4_hitboxes[0].hit.state != HitCapsule_Disabled {
        temp_f1 = fmadds((*ip).xDD4_itemVar.zeldadinfireexplode.xDD4, (*attrs).x10, (*attrs).xC);
        ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, f2u32(temp_f1 as f64), gobj);

        if (*ip).xDD4_itemVar.zeldadinfireexplode.xDD8 == 0.0 {
            (*ip).xDD4_itemVar.zeldadinfireexplode.xDD8 = (*ip).x5D4_hitboxes[0].hit.scale;
        }

        (*ip).x5D4_hitboxes[0].hit.scale = (*ip).xDD4_itemVar.zeldadinfireexplode.xDD8 * scale.x;
    }

    ext::it_80273130(gobj) != 0
}

/// @note Faithful to retail: only y and z are cleared (the doubled z is the original's typo); x velocity is
/// deliberately left untouched.
pub unsafe extern "C" fn itZeldadinfireexplode_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
}

pub unsafe extern "C" fn itZeldadinfireexplode_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_Clanked(arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_Absorbed(arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_ShieldBounced(arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_HitShield(arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itZeldaDinFireExplode_Logic66_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
