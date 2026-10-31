//! Native port of src/melee/it/kinds/itfoods.c: the food items (maxim tomato, heart container and
//! co.: spawn position from the attribute table, the random pick, the state callbacks and the
//! state table `it_803F5DB0`). Callees in other TUs go through `ext` (hooks; unset panics "TODO").
//! Checked against the C (the whole TU, shipping contraction) by tests/tu_itfoods.rs.
//!
//! Header inlines compiled into the TU: `itResetVelocity`, `HSD_JObjSetTranslate` (jobj.h, with
//! its `HSD_ASSERT`s and the `HSD_JObjSetMtxDirty` macro), `HSD_JObjGetChild`, `GET_ITEM`.
//!
//! Float math: `pos.x = arg1->x + arg8 * attr[i].w` is the one contracted site (a Gekko `fmadds`:
//! f64 multiply, f64 add, one rounding to f32), see `melee_core::fp::fmadds`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D324(a0: i32) -> i32 => unsafe { crate::tu::itspawn::it_8026D324(a0) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273318(a0: *mut HSD_GObj, a1: *mut HSD_Joint) => unsafe { crate::tu::it_2725::it_80273318(a0, a1) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn get_item(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `HSD_ASSERT(line, cond)`: the failing path is never taken by well-formed state.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT failed: {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "jobj");
    let mut result = false;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0 {
        result = true;
    }
    result
}

/// `HSD_JObjSetTranslate` (jobj.h) with the `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!jobj.is_null(), "jobj");
    hsd_assert(!translate.is_null(), "translate");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjGetChild` (jobj.h).
#[inline]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}

/// `itResetVelocity`.
#[inline]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

pub static mut it_803F5DB0: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: -1, animated: Some(itFoods_UnkMotion0_Anim), physics_updated: Some(itFoods_UnkMotion0_Phys), collided: Some(itFoods_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFoods_UnkMotion1_Anim), physics_updated: Some(itFoods_UnkMotion1_Phys), collided: Some(itFoods_UnkMotion1_Coll) },
    ItemStateTable { anim_id: -1, animated: Some(itFoods_UnkMotion2_Anim), physics_updated: Some(itFoods_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: -1, animated: Some(itFoods_UnkMotion3_Anim), physics_updated: Some(itFoods_UnkMotion3_Phys), collided: Some(itFoods_UnkMotion3_Coll) },
];

pub unsafe extern "C" fn it_8028F9D8(arg0: *mut Item_GObj, arg1: *mut Vec, arg8: f32) {
    melee_core::ct!("it_8028F9D8");
    let temp_r30 = get_item(arg0);
    let temp_r6 = (*(*temp_r30).xC4_article_data).x4_specialAttributes as *mut Vec4;
    let x0 = (*temp_r30).xDD4_itemVar.foods.x0 as usize;
    (*temp_r30).pos.x = fmadds(arg8, (*temp_r6.add(x0)).w, (*arg1).x);
    let var_2 = (*temp_r6.add(x0 + 1)).x;
    (*temp_r30).pos.y = var_2 + (*arg1).y;
    (*temp_r30).pos.z = (*arg1).z;
    HSD_JObjSetTranslate((*arg0).hsd_obj as *mut HSD_JObj, &mut (*temp_r30).pos);
}

pub unsafe extern "C" fn it_8028FAF4(arg0: *mut Item_GObj, arg1: *mut Vec) -> *mut HSD_GObj {
    melee_core::ct!("it_8028FAF4");
    let mut spawn: SpawnItem = core::mem::zeroed();
    if ext::it_8026D324(It_Kind_Foods as _) == 0 {
        return null_mut();
    }
    spawn.kind = It_Kind_Foods;
    spawn.prev_pos = *arg1;
    if !arg0.is_null() {
        ext::it_8026BB68(arg0, &mut spawn.pos);
    } else {
        spawn.pos = spawn.prev_pos;
    }
    spawn.facing_dir = -1.0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x3C_damage = 0;
    spawn.x0_parent_gobj = null_mut();
    spawn.x4_parent_gobj2 = null_mut();
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        ext::it_80274ED8();
    }
    gobj
}

pub unsafe extern "C" fn getRandMax(article: *mut Article) -> u32 {
    melee_core::ct!("getRandMax");
    let attr = (*article).x4_specialAttributes as *mut itFoodsAttributes;
    (*attr).x0 as u32
}

/// `FOODS_JOINTS[i]`: the native joint of food variant `i`. The attribute block keeps the GameCube row layout
/// (`{s32 x0; HSD_Joint* x4; s32 x8; s32 xC}`, 16 bytes: `it_8028F9D8` reads the same rows as `Vec4`), so a pointer
/// cannot sit in `x4`; the engine's archive decoder stores the variant index there and the pointer here.
pub static mut FOODS_JOINTS: [*mut HSD_Joint; 32] = [core::ptr::null_mut(); 32];

/// One 16-byte row of the Foods attribute block (GameCube layout; `x4` is a [`FOODS_JOINTS`] index).
#[repr(C)]
struct FoodsRow {
    x0: i32,
    x4: u32,
    x8: i32,
    xc: i32,
}

pub unsafe extern "C" fn itFoods_Logic18_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itFoods_Logic18_Spawned");
    let ip = get_item(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut FoodsRow;
    let rand = ext::HSD_Randi(getRandMax((*ip).xC4_article_data) as i32);
    (*ip).xDD4_itemVar.foods.heal_amount = (*attr.offset(rand as isize)).x8 as u32;
    (*ip).xDD4_itemVar.foods.x0 = rand as u32;
    let temp = rand;
    ext::it_80273318(gobj, FOODS_JOINTS[(*attr.offset(temp as isize)).x4 as usize]);
    it_8028FC5C(gobj);
}

pub unsafe extern "C" fn it_8028FC5C(arg0: *mut HSD_GObj) {
    melee_core::ct!("it_8028FC5C");
    ext::Item_80268E5C(arg0, 0, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFoods_UnkMotion0_Anim(_arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itFoods_UnkMotion0_Phys(arg0: *mut Item_GObj) {
    melee_core::ct!("itFoods_UnkMotion0_Phys");
    let item = get_item(arg0);
    let temp_r4 = (*item).xCC_item_attr;
    ext::it_80272860(arg0, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFoods_UnkMotion0_Coll(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion0_Coll");
    ext::it_8026E414(arg0, Some(it_8028FCE8));
    0
}

pub unsafe extern "C" fn it_8028FCE8(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_8028FCE8");
    let item = get_item(gobj);
    itResetVelocity(item);
    ext::Item_8026AE84(item, 0x107, 0x7F, 0x40);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFoods_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion1_Anim");
    0
}

pub unsafe extern "C" fn itFoods_UnkMotion1_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFoods_UnkMotion1_Phys");}

pub unsafe extern "C" fn itFoods_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion1_Coll");
    ext::it_8026D62C(gobj, Some(it_8028FC5C));
    0
}

pub unsafe extern "C" fn itFoods_Logic18_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itFoods_Logic18_PickedUp");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child = HSD_JObjGetChild(jobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as _);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as _);
}

pub unsafe extern "C" fn itFoods_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion2_Anim");
    0
}

pub unsafe extern "C" fn itFoods_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itFoods_UnkMotion2_Phys");}

pub unsafe extern "C" fn itFoods_Logic18_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itFoods_Logic18_Dropped");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itFoods_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion3_Anim");
    0
}

pub unsafe extern "C" fn itFoods_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itFoods_UnkMotion3_Phys");
    let item = get_item(gobj);
    let temp_r4 = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*temp_r4).x10_fall_speed, (*temp_r4).x14_fall_speed_max);
}

pub unsafe extern "C" fn itFoods_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itFoods_UnkMotion3_Coll");
    ext::it_8026E414(gobj, Some(it_8028FCE8));
    0
}

pub unsafe extern "C" fn itFoods_Logic18_EvtUnk(gobj: *mut Item_GObj, arg1: *mut Item_GObj) {
    melee_core::ct!("itFoods_Logic18_EvtUnk");
    ext::it_8026B894(gobj, arg1);
}
