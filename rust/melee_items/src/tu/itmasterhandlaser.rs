//! Port of src/melee/it/kinds/itmasterhandlaser.c: Master Hand's laser (logic table 84; `it_803F9378`).
//! `masterhandlaser`: `x0` = "finished" flag set by the event script, `x4` = the fighter part the beam is
//! attached to, `x8` = frames until the next spark, `x10` = the beam's root joint. Callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_itmasterhandlaser.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::{fmadds, frsqrte};
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_802694CC(a0: *mut HSD_GObj);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_802750F8(a0: *mut HSD_GObj);
    fn it_80275788(a0: *mut HSD_GObj);
    fn it_80275820(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut Vec, a3: i32);
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec);
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint);
    fn lb_8000C2F8(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn lb_8000C390(a0: *mut HSD_JObj);
    fn mpCheckMultiple(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: u32, a9: i32, a10: i32) -> i32;
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

/// `ItemStateTable it_803F9378[]`
pub static mut it_803F9378: [ItemStateTable; 1] = [st_row(
    0,
    pred_b(itMasterhandlaser_UnkMotion0_Anim),
    Some(itMasterhandlaser_UnkMotion0_Phys),
    pred_b(itMasterhandlaser_UnkMotion0_Coll),
)];

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
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `0.5 * guess * (3.0 - guess * guess * x)`: the shipping build contracts `3.0 - (guess*guess)*x` into one
/// double fma.
#[inline(always)]
fn nr_step(guess: f64, x: f64) -> f64 {
    (0.5 * guess) * (-(guess * guess)).mul_add(x, 3.0)
}

/// `sqrtf_store` (static inline): `__frsqrte` plus three Newton steps in double; the result goes through the
/// volatile slot `*y`.
#[inline]
pub unsafe fn sqrtf_store(x: f32, y: *mut f32) -> f32 {
    if x > 0.0 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        guess = nr_step(guess, xd);
        core::ptr::write_volatile(y, (xd * guess) as f32);
        return core::ptr::read_volatile(y);
    }
    x
}

/// 2F0340
pub unsafe extern "C" fn it_802F0340(
    gobj: *mut Item_GObj,
    prev_pos: *mut Vec3,
    pos: *mut Vec3,
    part: Fighter_Part,
    kind: ItemKind,
    facing_dir: f32,
) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();
    let fp = (*gobj).user_data as *mut Fighter;
    spawn.kind = kind;
    spawn.prev_pos = *prev_pos;
    spawn.pos = *pos;
    Item_InitSpawnCommonFields(&mut spawn, gobj, facing_dir, true);
    let spawned = ext::Item_80268B18(&mut spawn);
    it_802F0484(spawned);
    ext::db_80225DD8(spawned, gobj);
    ext::it_802750F8(spawned);
    let ip = GET_ITEM(spawned);
    (*ip).xDD4_itemVar.masterhandlaser.x4 = part as i32;
    (*ip).xDD4_itemVar.masterhandlaser.x10 = (*spawned).hsd_obj as *mut HSD_JObj;
    ext::lb_8000C2F8((*ip).xDD4_itemVar.masterhandlaser.x10, (*(*fp).parts.add(part as usize)).joint);
    ext::Item_802694CC(spawned);
    spawned
}

pub unsafe extern "C" fn itMasterHandLaser_Logic84_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    ext::it_8026B894(gobj, arg1);
}

/// 2F046C
pub unsafe extern "C" fn it_802F046C(gobj: *mut Item_GObj) {
    if !gobj.is_null() {
        (*GET_ITEM(gobj)).xDD4_itemVar.masterhandlaser.x0 = 1;
    }
}

/// 2F0484
pub unsafe extern "C" fn it_802F0484(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMasterHandLaserAttributes;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    (*ip).on_accessory = Some(it_802F05A8);
    (*ip).xDD4_itemVar.masterhandlaser.x0 = 0;
    (*ip).xDD4_itemVar.masterhandlaser.x8 = (*attrs).x4 as i32;
}

pub unsafe extern "C" fn itMasterhandlaser_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        it_802F0484(gobj);
    }
    if (*ip).xDD4_itemVar.masterhandlaser.x0 != 0 {
        ext::lb_8000C390((*ip).xDD4_itemVar.masterhandlaser.x10);
        return true;
    }
    false
}

pub unsafe extern "C" fn itMasterhandlaser_UnkMotion0_Phys(_gobj: *mut Item_GObj) {}

pub unsafe extern "C" fn itMasterhandlaser_UnkMotion0_Coll(_gobj: *mut Item_GObj) -> bool {
    false
}

/// 2F05A8
pub unsafe extern "C" fn it_802F05A8(gobj: *mut Item_GObj) {
    let mut sp28: Vec3 = core::mem::zeroed();
    let mut sp1C: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);
    let owner = (*ip).owner;
    let fp = (*owner).user_data as *mut Fighter;
    it_802F063C(gobj, owner);
    ext::lb_8000B1CC((*(*fp).parts.add((*ip).xDD4_itemVar.masterhandlaser.x4 as usize)).joint, null_mut(), &mut sp28);
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[2], null_mut(), &mut sp1C);
    ext::it_80275788(gobj);
    ext::it_80275820(gobj, &mut sp28, &mut sp1C, 0);
}

/// 2F063C
pub unsafe extern "C" fn it_802F063C(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    let mut pos_0: Vec3 = core::mem::zeroed();
    let mut pos_1: Vec3 = core::mem::zeroed();
    let mut pos_2: Vec3 = core::mem::zeroed();
    let mut translate: Vec3 = core::mem::zeroed();
    let mut sqrt_0: f32 = 0.0;
    let mut sqrt_1: f32 = 0.0;
    let mut sqrt_2: f32 = 0.0;

    let ip = GET_ITEM(gobj);
    let fp = (*(*ip).owner).user_data as *mut Fighter;
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMasterHandLaserAttributes;

    ext::lb_8000B804(HSD_JObjGetChild((*gobj).hsd_obj as *mut HSD_JObj), (*(*ip).xC8_joint).child);
    ext::lb_8000B1CC((*(*fp).parts.add((*ip).xDD4_itemVar.masterhandlaser.x4 as usize)).joint, null_mut(), &mut pos_0);
    let y0 = pos_0.y;
    let x0 = pos_0.x;

    translate.x = 0.0;
    translate.y = -(*attrs).x0;
    translate.z = 0.0;

    let bone2 = (*(*ip).xBBC_dynamicBoneTable).bones[2];
    HSD_JObjSetTranslate(bone2, &mut translate);
    ext::lb_8000B1CC(bone2, null_mut(), &mut pos_2);
    let x1 = pos_2.x;
    let y1 = pos_2.y;

    if ext::mpCheckMultiple(x0, y0, x1, y1, &mut pos_1, null_mut(), null_mut(), null_mut(), 1, -1, -1) != 0 {
        translate.x = 0.0;
        translate.z = 0.0;
        // `a*a + b*b + c*c` contracts to fmadds(c, c, fmadds(a, a, b*b)); `a*a + b*b` to fmadds(a, a, b*b)
        let dx = pos_0.x - pos_2.x;
        let dy = pos_0.y - pos_2.y;
        let dz = pos_0.z - pos_2.z;
        let s0 = sqrtf_store(fmadds(dz, dz, fmadds(dx, dx, dy * dy)), &mut sqrt_0);
        let dx1 = pos_0.x - pos_1.x;
        let dy1 = pos_0.y - pos_1.y;
        let s1 = sqrtf_store(fmadds(dx1, dx1, dy1 * dy1), &mut sqrt_1);
        let s2 = sqrtf_store(fmadds(dx, dx, dy * dy), &mut sqrt_2);
        translate.y = s0 * -(s1 / s2);
        HSD_JObjSetTranslate((*(*ip).xBBC_dynamicBoneTable).bones[2], &mut translate);

        (*ip).xDD4_itemVar.masterhandlaser.x8 = (*ip).xDD4_itemVar.masterhandlaser.x8.wrapping_sub(1);
        if (*ip).xDD4_itemVar.masterhandlaser.x8 < 0 {
            ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[2], null_mut(), &mut pos_2);
            ext::efSync_Spawn(0x405, gobj, &mut pos_2);
            (*ip).xDD4_itemVar.masterhandlaser.x8 = (*attrs).x4 as i32;
        }
    } else {
        HSD_JObjSetTranslate((*(*ip).xBBC_dynamicBoneTable).bones[2], &mut translate);
    }
}
