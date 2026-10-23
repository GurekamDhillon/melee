//! Native port of src/melee/it/kinds/itgamewatchjudge.c (Game & Watch's Judgment item). Callees in other
//! TUs go through `ext` (TODO hooks). `bool` is the C `int`: callee results are tested as ints and
//! `true` is 1. Checked against the whole C TU (shipping contraction) by tests/tu_itgamewatchjudge.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut)]
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32);
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj);
    fn ftGw_SpecialS_ItemCheckJudgementRemove(a0: *mut HSD_GObj) -> i32;
    fn ftGw_SpecialS_ItemJudgementSetFlag(a0: *mut HSD_GObj);
    fn ftLib_800865C0(a0: *mut HSD_GObj) -> f32;
    fn it_8026B724(a0: *mut HSD_GObj);
    fn it_8026B73C(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32);
    fn it_8027CE64(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: *mut c_void);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

#[inline]
unsafe fn GET_ITEM(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_InitSpawnPosition` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnPosition(spawn: *mut SpawnItem, pos: *mut Vec, on_plane: bool) {
    (*spawn).prev_pos = *pos;
    if on_plane {
        (*spawn).prev_pos.z = 0.0;
    }
    (*spawn).pos = (*spawn).prev_pos;
}

/// `Item_InitSpawnCommonFields` (it/kinds/inlines.h, static inline).
#[inline]
unsafe fn Item_InitSpawnCommonFields(spawn: *mut SpawnItem, parent: *mut HSD_GObj, facing_dir: f32, initial_collision: bool) {
    (*spawn).facing_dir = facing_dir;
    (*spawn).x3C_damage = 0;
    (*spawn).vel.x = 0.0;
    (*spawn).vel.y = 0.0;
    (*spawn).vel.z = 0.0;
    (*spawn).x0_parent_gobj = parent;
    (*spawn).x4_parent_gobj2 = (*spawn).x0_parent_gobj;
    (*spawn).x44_flag.anon_0.set_b0(initial_collision as u8);
    (*spawn).x40 = 0;
}

/// `void** attr = item->xC4_article_data->x4_specialAttributes`.
#[inline]
unsafe fn special_attributes(item: *mut Item) -> *mut *mut c_void {
    (*(*item).xC4_article_data).x4_specialAttributes as *mut *mut c_void
}

const JOBJ_USE_QUATERNION_: u32 = JOBJ_USE_QUATERNION as u32;
const JOBJ_MTX_INDEP_SRT_: u32 = JOBJ_MTX_INDEP_SRT as u32;
const JOBJ_USER_DEF_MTX_: u32 = JOBJ_USER_DEF_MTX as u32;
const JOBJ_MTX_DIRTY_: u32 = JOBJ_MTX_DIRTY as u32;

/// `HSD_ASSERT(line, cond)` (baselib/debug.h) when it fails: `__assert` never returns.
#[cold]
fn hsd_assert_fail(line: u32, cond: &str) -> ! {
    panic!("__assert(jobj.h, {line}, {cond})")
}

/// `HSD_JObjGetChild` (baselib/jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        null_mut()
    } else {
        (*jobj).child
    }
}

/// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    if jobj.is_null() {
        hsd_assert_fail(564, "jobj");
    }
    (*jobj).flags & JOBJ_USER_DEF_MTX_ == 0 && (*jobj).flags & JOBJ_MTX_DIRTY_ != 0
}

/// `HSD_JObjSetRotationY` (baselib/jobj.h, static inline) with the `HSD_JObjSetMtxDirty` macro.
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    if jobj.is_null() {
        hsd_assert_fail(660, "jobj");
    }
    if (*jobj).flags & JOBJ_USE_QUATERNION_ != 0 {
        hsd_assert_fail(661, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    }
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT_ == 0 && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `ItemStateTable it_803F7968[]`
pub static mut it_803F7968: [ItemStateTable; 1] = [
    ItemStateTable { anim_id: 0, animated: Some(itGamewatchjudge_UnkMotion0_Anim), physics_updated: None, collided: None },
];

/// `HSD_GObj* it_802C7774(float facing_dir, HSD_GObj* parent_gobj, Vec3* pos, s32 part, u32 arg4)`
pub unsafe extern "C" fn it_802C7774(facing_dir: f32, parent_gobj: *mut HSD_GObj, pos: *mut Vec, part: i32, arg4: u32) -> *mut HSD_GObj {
    let mut si: SpawnItem = core::mem::zeroed();
    si.kind = It_Kind_GameWatch_Judge;
    Item_InitSpawnPosition(&mut si, pos, false);
    Item_InitSpawnCommonFields(&mut si, parent_gobj, facing_dir, true);

    let new_gobj = ext::Item_80268B18(&mut si);
    if !new_gobj.is_null() {
        let special_attrs = special_attributes(GET_ITEM(new_gobj));
        ext::Item_8026AB54(new_gobj, parent_gobj, part as u32);
        ext::it_80273670(new_gobj, 0, arg4.wrapping_add(1) as i32 as f32);
        it_802C78B8(new_gobj);
        ext::db_80225DD8(new_gobj, parent_gobj);
        ext::it_8027CE64(new_gobj, parent_gobj, *special_attrs);
        return new_gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn it_802C78B8(item_gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(item_gobj);
    let jobj = (*item_gobj).hsd_obj as *mut HSD_JObj;
    if !(*ip).owner.is_null() {
        let facing = ext::ftLib_800865C0((*ip).owner);
        let child = HSD_JObjGetChild(HSD_JObjGetChild(jobj));
        if facing == 1.0f32 {
            HSD_JObjSetRotationY(child, core::f64::consts::PI as f32);
        } else {
            HSD_JObjSetRotationY(child, 0.0f32);
        }
    }
}

pub unsafe extern "C" fn itGameWatchJudge_Logic77_Destroyed(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    if !(*ip).owner.is_null() {
        ext::ftGw_SpecialS_ItemJudgementSetFlag((*ip).owner);
    }
}

pub unsafe extern "C" fn it_802C7A84(item_gobj: *mut HSD_GObj) {
    let item = GET_ITEM(item_gobj);
    if !item.is_null() {
        itGameWatchJudge_Logic77_Destroyed(item_gobj);
        ext::Item_8026A8EC(item_gobj);
    }
}

pub unsafe extern "C" fn it_802C7AD0(gobj: *mut HSD_GObj) {
    ext::it_8026B724(gobj);
}

pub unsafe extern "C" fn it_802C7AF0(gobj: *mut HSD_GObj) {
    ext::it_8026B73C(gobj);
}

pub unsafe extern "C" fn it_2725_Logic77_PickedUp(item_gobj: *mut HSD_GObj) {
    let it = GET_ITEM(item_gobj);
    (*it).xDB0_itcmd_var1 = 0;
    (*it).xDAC_itcmd_var0 = 0;
    if !(*it).owner.is_null() {
        ext::Item_80268E5C(item_gobj, 0, ITEM_ANIM_UPDATE as u32);
    }
}

pub unsafe extern "C" fn itGamewatchjudge_UnkMotion0_Anim(item_gobj: *mut HSD_GObj) -> i32 {
    it_802C78B8(item_gobj);
    let remove: i32 = {
        let ip = GET_ITEM(item_gobj);
        if !(*ip).owner.is_null() {
            ext::ftGw_SpecialS_ItemCheckJudgementRemove((*ip).owner)
        } else {
            1
        }
    };
    if remove != 0 {
        itGameWatchJudge_Logic77_Destroyed(item_gobj);
        return 1;
    }
    0
}

pub unsafe extern "C" fn itGameWatchJudge_Logic77_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
