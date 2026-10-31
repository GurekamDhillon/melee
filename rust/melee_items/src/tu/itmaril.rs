//! Port of src/melee/it/kinds/itmaril.c: Marill (Pokeball Pokemon, logic table 28; `it_803F82B0`).
//! `maril.x60.x0` is the "turning around" flag (`x6C.x` is the turn step, `x6C.y` the frames left; before
//! the first bounce `x6C` is the normalised velocity), `x64` the horizontal speed, `x68` the roll timer.
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itmaril.rs.
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BD9C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026bd9c(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026D9A0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026D9A0(a0) };
    fn it_8026DB40(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DB40(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273130(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273130(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802754BC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754bc(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80276308(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80276308(&mut *((*a0).user_data as *mut Item)) };
    fn it_802763E0(a0: *mut HSD_GObj) -> i32 => unsafe { crate::helpers::it_802763e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279D38(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279D38(a0) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    fn it_802D61C8(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::itkinds10::it_802d61c8(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_802D6310(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::ithinoarashi::it_802D6310(a0 as _, a1) };
    fn lbVector_Normalize(a0: *mut Vec) -> f32 => unsafe { melee_runtime::tu::lbvector::lbVector_Normalize(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

kinds_jobj_inl!();

/// `ItemStateTable it_803F82B0[]`
pub static mut it_803F82B0: [ItemStateTable; 4] = [
    st_row(-1, pred_b(itMaril_UnkMotion0_Anim), Some(itMaril_UnkMotion0_Phys), pred_b(itMaril_UnkMotion0_Coll)),
    st_row(0, pred_b(itMaril_UnkMotion1_Anim), Some(itMaril_UnkMotion1_Phys), pred_b(itMaril_UnkMotion1_Coll)),
    st_row(-1, pred_b(itMaril_UnkMotion2_Anim), Some(itMaril_UnkMotion2_Phys), pred_b(itMaril_UnkMotion2_Coll)),
    st_row(-1, pred_b(itMaril_UnkMotion3_Anim), Some(itMaril_UnkMotion3_Phys), None),
];

/// `Item_SetEffectHitlagCallbacks` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_SetEffectHitlagCallbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterStateWithEffectHitlag` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_EnterStateWithEffectHitlag(gobj: *mut Item_GObj, msid: i32) {
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE);
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `itMaril_UnkMotion2_Anim_inline` (static)
pub unsafe extern "C" fn itMaril_UnkMotion2_Anim_inline(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion2_Anim_inline");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    (*ip).xD44_lifeTimer <= 0.0
}

/// 2D66F8
pub unsafe extern "C" fn it_802D66F8(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D66F8");
    if ext::it_80273130(gobj) == 1 {
        ext::efLib_DestroyAll(gobj);
        return true;
    }
    false
}

/// 2D6740
pub unsafe extern "C" fn it_802D6740(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D6740");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMarilAttributes;
    (*ip).x40_vel.y += (*attr).xC;
    (*ip).xDD4_itemVar.maril.x6C = (*ip).x40_vel;
    ext::lbVector_Normalize(&mut (*ip).xDD4_itemVar.maril.x6C);
}

/// 2D6798
pub unsafe extern "C" fn it_802D6798(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6798");
    let mut coll = ext::it_8026D9A0(gobj);
    coll |= ext::it_80276308(gobj);
    coll |= ext::it_802763E0(gobj);
    if coll != 0 {
        ext::it_802D61C8(gobj, coll);
        ext::it_802D6310(gobj, coll);
    }
    false
}

/// 2D6808
pub unsafe extern "C" fn it_802D6808(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6808");
    false
}

/// 2D6810
pub unsafe extern "C" fn it_802D6810(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6810");
    ext::it_80273030(gobj) != 0
}

/// 2D6830
pub unsafe extern "C" fn it_802D6830(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6830");
    true
}

/// 2D6838
pub unsafe extern "C" fn it_802D6838(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6838");
    false
}

/// 2D6840
pub unsafe extern "C" fn it_802D6840(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6840");
    true
}

/// 2D6848
pub unsafe extern "C" fn it_802D6848(_arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6848");
    true
}

/// 2D6850
pub unsafe extern "C" fn it_802D6850(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D6850");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn itMaril_Logic28_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_Logic28_Spawned");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMarilAttributes;
    (*ip).xDD4_itemVar.maril.x60.anon_0.set_x0(0);
    (*ip).x5D0_animFrameSpeed = (*attr).x10;
    ext::it_80279CDC(gobj, (*attr).x0);
    ext::Item_8026AE84(ip, 0x273D, 0x7F, 0x40);
    ext::it_80275158(gobj, (*attr).x4);
    it_802D68FC(gobj);
}

/// 2D68FC
pub unsafe extern "C" fn it_802D68FC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D68FC");
    Item_EnterStateWithEffectHitlag(gobj, 0);
    ext::it_80273670(gobj, 0, 0.0);
    ext::it_8026BDB4(gobj);
}

/// `itMaril_UnkMotion1_Coll_inline` (static)
pub unsafe extern "C" fn itMaril_UnkMotion1_Coll_inline(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_UnkMotion1_Coll_inline");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMarilAttributes;
    if (*ip).xDD4_itemVar.maril.x60.anon_0.x0() == 0 {
        (*ip).xDD4_itemVar.maril.x6C.y = (*attr).x8;
        (*ip).xDD4_itemVar.maril.x6C.x = ((core::f64::consts::PI * (-(*ip).facing_dir) as f64) / (*attr).x8 as f64) as f32;
        (*ip).xDD4_itemVar.maril.x60.anon_0.set_x0(1);
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).xDC8_word.flags.set_x19(0);
        (*ip).xDD4_itemVar.maril.x64 = -(*ip).xDD4_itemVar.maril.x64;
    }
}

pub unsafe extern "C" fn itMaril_UnkMotion0_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion0_Anim");
    ext::it_80279FF8(gobj);
    false
}

pub unsafe extern "C" fn itMaril_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_UnkMotion0_Phys");
    ext::it_8027A09C(gobj);
}

pub unsafe extern "C" fn itMaril_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion0_Coll");
    ext::it_8027A118(gobj, Some(it_802D69E4)) != 0
}

/// 2D69E4
pub unsafe extern "C" fn it_802D69E4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D69E4");
    let ip = GET_ITEM(gobj);
    ext::it_80273454(gobj);
    ext::it_802762BC(ip);
    it_802D6A54(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    Item_SetEffectHitlagCallbacks(ip);
    (*ip).xDD4_itemVar.maril.x68 = 8.0;
}

/// 2D6A54
pub unsafe extern "C" fn it_802D6A54(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D6A54");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMarilAttributes;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    (*ip).xDD4_itemVar.maril.x64 = (*attr).xC * (*ip).facing_dir;
    ext::it_8026BD9C(gobj);
    ext::it_802756E0(gobj);
    ext::it_802754D4(gobj);
    ext::it_802754BC(gobj);
}

pub unsafe extern "C" fn itMaril_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    if (*ip).xDD4_itemVar.maril.x60.anon_0.x0() != 0 {
        let rot = (*ip).xDD4_itemVar.maril.x6C.x;
        HSD_JObjAddRotationY(jobj, rot);
        (*ip).xDD4_itemVar.maril.x6C.y -= 1.0;
        if (*ip).xDD4_itemVar.maril.x6C.y <= 0.0 {
            (*ip).xDD4_itemVar.maril.x60.anon_0.set_x0(0);
            (*ip).xDC8_word.flags.set_x19(1);
        }
    }
    itMaril_UnkMotion2_Anim_inline(gobj)
}

pub unsafe extern "C" fn itMaril_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    ext::it_8027A344(gobj);
    (*ip).x40_vel.x = (*ip).xDD4_itemVar.maril.x64;
}

pub unsafe extern "C" fn itMaril_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion1_Coll");
    let coll = ext::it_8026DB40(gobj);
    if ext::it_80272C6C(gobj) == 0 {
        if coll & 1 != 0 {
            it_802D69E4(gobj);
        } else {
            it_802D6DDC(gobj);
        }
    } else if coll & 0xC != 0 {
        itMaril_UnkMotion1_Coll_inline(gobj);
    }
    false
}

/// 2D6DDC
pub unsafe extern "C" fn it_802D6DDC(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D6DDC");
    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1);
}

pub unsafe extern "C" fn itMaril_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion2_Anim");
    itMaril_UnkMotion2_Anim_inline(gobj)
}

pub unsafe extern "C" fn itMaril_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMaril_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion2_Coll");
    ext::it_8026E15C(gobj, Some(it_802D69E4));
    false
}

pub unsafe extern "C" fn itMaril_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMaril_UnkMotion3_Anim");
    itMaril_UnkMotion2_Anim(gobj)
}

pub unsafe extern "C" fn itMaril_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itMaril_UnkMotion3_Phys");
    itMaril_UnkMotion2_Phys(gobj);
}

/// `it_802D6F00_inline` (static inline)
#[inline]
unsafe fn it_802D6F00_inline(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    ext::it_802762BC(ip);
    ext::it_8026BDB4(gobj);
    ext::it_802756D0(gobj);
    ext::efLib_DestroyAll(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    ext::it_80273670(gobj, 0, 0.0);
}

/// 2D6F00
pub unsafe extern "C" fn it_802D6F00(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("it_802D6F00");
    let ip = GET_ITEM(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itMarilAttributes;
    if (*ip).xC9C as f32 >= (*attr).x14 {
        ext::it_80279D38(gobj);
        it_802D6F00_inline(gobj);
    }
    false
}

/// 2D6FB0
pub unsafe extern "C" fn it_802D6FB0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D6FB0");
    ext::it_8026B894(gobj, ref_gobj);
}
