//! Port of src/melee/it/kinds/itnesspkflashexplode.c: the PK Flash explosion (logic table 103). Callees of
//! unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_itnesspkflashexplode.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_802694CC(a0: *mut HSD_GObj);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_80272460(a0: *mut HitCapsule, a1: u32, a2: *mut HSD_GObj);
    fn it_802725D4(a0: *mut HSD_GObj);
    fn it_80272940(a0: *mut HSD_GObj);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273598(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
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

/// `ItemStateTable it_803F6F40[]`
pub static mut it_803F6F40: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itNessPKFlashExplode_UnkMotion0_Anim),
    Some(itNessPKFlashExplode_UnkMotion0_Phys),
    pred_b(itNessPKFlashExplode_UnkMotion0_Coll),
)];

#[inline]
unsafe fn Item_ClearFlagsAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    ext::it_8026B3A8(gobj);
    (*ip).xDC8_word.flags.set_x13(0);
    ext::it_80272940(gobj);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn it_802AF940(owner: *mut Item_GObj, flash: *mut Item_GObj, pos: *mut Vec3, kind: ItemKind, facing_dir: f32, charge: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = kind;
    Item_InitSpawnPosition(&mut spawn, pos, true);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = owner;
    spawn.x4_parent_gobj2 = flash;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;

    let explode = ext::Item_80268B18(&mut spawn);
    if !explode.is_null() {
        let ip = GET_ITEM(explode);
        Item_ClearCmdVars(ip);
        (*ip).xDD4_itemVar.pkflushexplode.xDD4 = charge;
        (*ip).xDD4_itemVar.pkflushexplode.xDDC = owner;
        it_802AFA70(explode);
    }
    explode
}

pub unsafe extern "C" fn it_2725_Logic103_Destroyed(item_gobj: *mut Item_GObj) {
    if !item_gobj.is_null() {
        let item = GET_ITEM(item_gobj);
        if !item.is_null() {
            ext::it_802725D4(item_gobj);
            (*item).owner = null_mut();
        }
    }
}

pub unsafe extern "C" fn it_802AFA70(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itFlashExplAttributes;

    Item_ClearFlagsAndEnterState(gobj, ip, 0);
    ext::it_80275158(gobj, 1024.0);
    (*ip).xDD4_itemVar.pkflushexplode.xDD8 = 0.0;
    ext::Item_802694CC(gobj);
    itNessPKFlashExplode_UnkMotion0_Anim(gobj);

    let ratio = (*ip).xDD4_itemVar.pkflushexplode.xDD4 / (*attr).x0_FLASH_EXPL_HITBOX_SIZE_MUL;
    if ratio > 0.84999996f32 {
        ext::Camera_RequestQuake(QuakeKind_Medium as u32, &mut (*ip).pos);
        ext::it_80273598(gobj, 0xE, 0x14);
    } else if ratio > 0.65f32 {
        ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut (*ip).pos);
        ext::it_80273598(gobj, 0xD, 0xF);
    } else if ratio > 0.45f32 {
        ext::it_80273598(gobj, 0xC, 0xA);
    } else {
        ext::it_80273598(gobj, 0xB, 5);
    }
    ext::db_80225DD8(gobj, (*ip).owner);
}

pub unsafe extern "C" fn itNessPKFlashExplode_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itFlashExplAttributes;
    let jobj = GET_JOBJ(gobj);
    let mut scale: Vec3 = core::mem::zeroed();
    let v = fmadds(
        (*ip).xDD4_itemVar.pkflushexplode.xDD4,
        ((*attr).x8_FLASH_EXPL_GRAPHIC_SIZE_GROWTH - (*attr).x4_FLASH_EXPL_GRAPHIC_SIZE_INIT) / (*attr).x0_FLASH_EXPL_HITBOX_SIZE_MUL,
        (*attr).x4_FLASH_EXPL_GRAPHIC_SIZE_INIT,
    );
    scale.z = v;
    scale.y = v;
    scale.x = v;
    HSD_JObjSetScale(jobj, &mut scale);

    if (*ip).xDB4_itcmd_var2 == 0 && (*ip).x5D4_hitboxes[0].hit.state != HitCapsule_Disabled {
        let dmg = fmadds((*ip).xDD4_itemVar.pkflushexplode.xDD4, (*attr).x10_FLASH_EXPL_DAMAGE_MUL, (*attr).xC_FLASH_EXPL_BASE_DAMAGE);
        ext::it_80272460(&mut (*ip).x5D4_hitboxes[0].hit, melee_runtime::cabi::f2u32(dmg as f64), gobj);
        (*ip).xDB4_itcmd_var2 = 1;
    }

    if (*ip).xDD4_itemVar.pkflushexplode.xDD8 == 0.0 {
        (*ip).xDD4_itemVar.pkflushexplode.xDD8 = (*ip).x5D4_hitboxes[0].hit.scale;
    }
    (*ip).x5D4_hitboxes[0].hit.scale = (*ip).xDD4_itemVar.pkflushexplode.xDD8 * scale.x;

    if ext::it_80272C6C(gobj) == 0 {
        return true;
    }
    false
}

pub unsafe extern "C" fn itNessPKFlashExplode_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let it = GET_ITEM(gobj);
    (*it).x40_vel.z = 0.0;
    (*it).x40_vel.y = 0.0;
}

pub unsafe extern "C" fn itNessPKFlashExplode_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

pub unsafe extern "C" fn itNessPKFlashExplode_Logic103_Clanked(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFlashExplode_Logic103_Absorbed(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFlashExplode_Logic103_ShieldBounced(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFlashExplode_Logic103_HitShield(_arg0: *mut Item_GObj) -> bool {
    true
}

pub unsafe extern "C" fn itNessPKFlashExplode_Logic103_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
