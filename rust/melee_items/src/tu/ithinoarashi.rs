//! Native port of src/melee/it/kinds/ithinoarashi.c (Moltres / "Hinoarashi" Pokemon and its flame
//! item). Callees in other TUs go through `ext` (settable hooks, unset panics "TODO"); the pure
//! math (sinf, cosf, atan2f, lbVector_*) is the native port in melee_runtime. Checked against the
//! C (the whole TU, shipping contraction) by tests/tu_ithinoarashi.rs.
//!
//! Fused sites (`fmadds`): `it_802D64B8` 2 (`(x14 - x10) * rand + x10`, `(x8 - x4) * rand + x4`).
//! Everything else is f32 arithmetic with M_PI / 2*M_PI in double (the C promotes).
//!
//! `it_802D61C8` stores a `Vec3` over the flame's item-var bytes +0x60..+0x6C and `it_802D6310`
//! reads them back as `f32` / `Vec3*` (a byte-level pun on the GameCube); both are done here with
//! unaligned raw reads / writes of the same bytes.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use crate::helpers::{M_PI, M_PI_2};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_core::vec::Vec3 as RVec;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE60() -> u32 => unsafe { crate::tu::item::Item_8026AE60() };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_DestroyAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_DestroyAll(a0) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026BDB4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026BDB4(a0) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802752D8(a0: *mut HSD_GObj, a1: i32, a2: i32) => unsafe { crate::tu::it_2725::it_802752D8(a0, a1, a2) };
    fn it_802754BC(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754bc(&mut *((*a0).user_data as *mut Item)) };
    fn it_802754D4(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802754d4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279D38(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279D38(a0) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    fn it_802D66F8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaril::it_802D66F8(a0 as _) as _ };
    fn it_802D6740(a0: *mut HSD_GObj) => unsafe { crate::itkinds10::it_802d6740(&mut *((*a0).user_data as *mut Item)) };
    fn it_802D6798(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaril::it_802D6798(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
        pub unsafe extern "C" fn it_802D66F8(a0: *mut HSD_GObj) -> i32 { super::it_802D66F8(a0) }
        pub unsafe extern "C" fn it_802D6740(a0: *mut HSD_GObj) { super::it_802D6740(a0) }
        pub unsafe extern "C" fn it_802D6798(a0: *mut HSD_GObj) -> i32 { super::it_802D6798(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `M_TAU` (src/Runtime/platform.h) as the double the C uses for `2 * M_PI`.
const TWO_PI: f64 = 2.0 * M_PI;

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `Item_SetEffectHitlagCallbacks` (src/melee/it/inlines.h).
#[inline]
unsafe fn set_effect_hitlag_callbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

/// `Item_EnterStateWithEffectHitlag` (src/melee/it/inlines.h).
#[inline]
unsafe fn enter_state_with_effect_hitlag(gobj: *mut Item_GObj, msid: i32) {
    let ip = ip_of(gobj);
    ext::it_802762BC(ip);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `Item_ClampAngle` (src/melee/it/kinds/inlines.h): wrap into [-pi, pi] by `M_TAU` steps in double.
#[inline]
unsafe fn item_clamp_angle(angle: *mut f32) {
    while (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + crate::helpers::M_TAU) as f32;
    }
    while *angle as f64 > M_PI {
        *angle = (*angle as f64 - crate::helpers::M_TAU) as f32;
    }
}

/// `Item_ClampAngleReverse` (src/melee/it/kinds/inlines.h): the same wrap, upper bound first.
#[inline]
unsafe fn item_clamp_angle_reverse(angle: *mut f32) {
    while *angle as f64 > M_PI {
        *angle = (*angle as f64 - crate::helpers::M_TAU) as f32;
    }
    while (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + crate::helpers::M_TAU) as f32;
    }
}

/// The C `ABS(x)` macro, `((x) < 0 ? -(x) : (x))` (keeps the sign of a zero).
#[inline]
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `ItemStateTable it_803F8270[3]` (3F8270)
pub static it_803F8270: [ItemStateTable; 3] = [
    ItemStateTable {
        anim_id: -1,
        animated: Some(it_802D5EC8),
        physics_updated: Some(it_802D5EEC),
        collided: Some(it_802D5F0C),
    },
    ItemStateTable {
        anim_id: 1,
        animated: Some(itHinoarashi_UnkMotion1_Anim),
        physics_updated: Some(itHinoarashi_UnkMotion1_Phys),
        collided: Some(itHinoarashi_UnkMotion1_Coll),
    },
    ItemStateTable {
        anim_id: -1,
        animated: Some(itHinoarashi_UnkMotion2_Anim),
        physics_updated: Some(itHinoarashi_UnkMotion2_Phys),
        collided: None,
    },
];

/// `ItemStateTable it_803F82A0[1]` (3F82A0): the flame's state, defined in another TU.
pub static it_803F82A0: [ItemStateTable; 1] = [ItemStateTable {
    anim_id: 0,
    animated: Some(ext::cb::it_802D66F8),
    physics_updated: Some(ext::cb::it_802D6740),
    collided: Some(ext::cb::it_802D6798),
}];

/// `void it_802D5CF8(Item_GObj*)` (2D5CF8)
pub unsafe extern "C" fn it_802D5CF8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5CF8");
    let ip = ip_of(gobj);
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;
    (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.set_b0(0);
    (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.set_b1(0);
    (*ip).xDD4_itemVar.hinoarashi.x64 = 0.0;
    ext::it_80279CDC(gobj, (*ip).scl);
    ext::Item_8026AE84(ip, 0x2734, 0x7F, 0x40);
    it_802D5E4C(gobj);
}

/// `void it_802D5D7C(HSD_GObj*)` (2D5D7C): the item's `on_accessory` callback.
pub unsafe extern "C" fn it_802D5D7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D5D7C");
    let ip = ip_of(gobj);
    let mut pos = Vec { x: 0.0, y: 0.0, z: 0.0 };

    if (*ip).xDAC_itcmd_var0 != 0 {
        if (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.b0() == 0 {
            (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.set_b0(1);
            ext::efSync_Spawn(
                0x473,
                gobj,
                &[(*(*ip).xBBC_dynamicBoneTable).bones[2] as *const c_void],
            );
        }
        ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[2], null_mut(), &mut pos);
        it_802D64B8(gobj, &mut pos, ext::Item_8026AE60(), (*ip).facing_dir);
        (*ip).xDAC_itcmd_var0 = 0;
    }
    if (*ip).xDB0_itcmd_var1 != 0 {
        ext::efLib_DestroyAll(gobj);
        (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.set_b1(1);
        (*ip).xDB0_itcmd_var1 = 0;
    }
}

/// `void it_802D5E4C(Item_GObj*)` (2D5E4C)
pub unsafe extern "C" fn it_802D5E4C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5E4C");
    enter_state_with_effect_hitlag(gobj, 0);
    ext::it_80273670(gobj, 0, 0.0);
    ext::it_8026BDB4(gobj);
}

/// `bool it_802D5EC8(Item_GObj*)` (2D5EC8)
pub unsafe extern "C" fn it_802D5EC8(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5EC8");
    ext::it_80279FF8(gobj);
    0
}

/// `void it_802D5EEC(Item_GObj*)` (2D5EEC)
pub unsafe extern "C" fn it_802D5EEC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D5EEC");
    ext::it_8027A09C(gobj);
}

/// `bool it_802D5F0C(Item_GObj*)` (2D5F0C)
pub unsafe extern "C" fn it_802D5F0C(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5F0C");
    ext::it_8027A118(gobj, Some(it_802D5F34))
}

/// `void it_802D5F34(HSD_GObj*)` (2D5F34)
pub unsafe extern "C" fn it_802D5F34(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802D5F34");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    ext::it_802756E0(gobj);
    ext::it_802754D4(gobj);
    ext::it_802754BC(gobj);
    ext::it_80273454(gobj);
    (*ip).on_accessory = Some(it_802D5D7C);
    (*ip).xDD1_flag.anon_0.set_b1(1);
}

/// `bool itHinoarashi_UnkMotion1_Anim(Item_GObj*)` (2D5FC8)
pub unsafe extern "C" fn itHinoarashi_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHinoarashi_UnkMotion1_Anim");
    let ip = ip_of(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHinoarashiAttributes;
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    if (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.b0() != 0
        && (*ip).xDD4_itemVar.hinoarashi.x60.anon_0.b1() == 0
    {
        if (*ip).xDD4_itemVar.hinoarashi.x64 == 0.0 {
            (*ip).xDD4_itemVar.hinoarashi.x64 = (*attr).x8;
            ext::it_802752D8(gobj, 3, 0x2735);
        }
        (*ip).xDD4_itemVar.hinoarashi.x64 -= 1.0;
    }
    ext::it_8027A160((*(*ip).xBBC_dynamicBoneTable).bones[1], ip);
    0
}

/// `void itHinoarashi_UnkMotion1_Phys(Item_GObj*)` (2D6084)
pub unsafe extern "C" fn itHinoarashi_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHinoarashi_UnkMotion1_Phys");
    ext::it_8027A344(gobj);
}

/// `bool itHinoarashi_UnkMotion1_Coll(Item_GObj*)` (2D60A4)
pub unsafe extern "C" fn itHinoarashi_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHinoarashi_UnkMotion1_Coll");
    ext::it_8026DA08(gobj);
    0
}

/// `bool itHinoarashi_UnkMotion2_Anim(Item_GObj*)` (2D60C8, static in the C)
pub unsafe extern "C" fn itHinoarashi_UnkMotion2_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHinoarashi_UnkMotion2_Anim");
    0
}

/// `void itHinoarashi_UnkMotion2_Phys(Item_GObj*)` (2D60D0)
pub unsafe extern "C" fn itHinoarashi_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHinoarashi_UnkMotion2_Phys");
    let ip = ip_of(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

/// `bool it_2725_Logic27_DmgReceived(Item_GObj*)` (2D6100)
pub unsafe extern "C" fn it_2725_Logic27_DmgReceived(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_2725_Logic27_DmgReceived");
    let ip = ip_of(gobj);
    let attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHinoarashiAttributes;
    if (*ip).xC9C as f32 >= (*attr).x4 {
        ext::it_80279D38(gobj);
        ext::it_802762BC((*gobj).user_data as *mut Item);
        ext::it_802756D0(gobj);
        ext::efLib_DestroyAll(gobj);
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
        ext::it_80273670(gobj, 0, 0.0);
    }
    0
}

/// `void it_802D61A8(Item_GObj*, Item_GObj*)` (2D61A8)
pub unsafe extern "C" fn it_802D61A8(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D61A8");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802D61C8(Item_GObj*, s32 coll)` (2D61C8): the surface normal the flame bounces off, stored
/// over the 12 item-var bytes at +0x60.
pub unsafe extern "C" fn it_802D61C8(gobj: *mut Item_GObj, coll: i32) {
    melee_core::ct!("it_802D61C8");
    let ip = ip_of(gobj);
    let mut normal = Vec { x: 0.0, y: 0.0, z: 0.0 };

    while ((*ip).xDD4_itemVar.hinoarashi.x78 as f64) < -M_PI {
        (*ip).xDD4_itemVar.hinoarashi.x78 = ((*ip).xDD4_itemVar.hinoarashi.x78 as f64 + TWO_PI) as f32;
    }
    while (*ip).xDD4_itemVar.hinoarashi.x78 as f64 > M_PI {
        (*ip).xDD4_itemVar.hinoarashi.x78 = ((*ip).xDD4_itemVar.hinoarashi.x78 as f64 - TWO_PI) as f32;
    }

    if coll & 1 != 0 {
        normal.x += (*ip).x378_itemColl.floor.normal.x;
        normal.y += (*ip).x378_itemColl.floor.normal.y;
    }
    if coll & 2 != 0 {
        normal.x += (*ip).x378_itemColl.ceiling.normal.x;
        normal.y += (*ip).x378_itemColl.ceiling.normal.y;
    }
    if coll & 8 != 0 {
        normal.x += (*ip).x378_itemColl.left_facing_wall.normal.x;
        normal.y += (*ip).x378_itemColl.left_facing_wall.normal.y;
    }
    if coll & 4 != 0 {
        normal.x += (*ip).x378_itemColl.right_facing_wall.normal.x;
        normal.y += (*ip).x378_itemColl.right_facing_wall.normal.y;
    }

    lbvector::normalize(&mut *(&mut normal as *mut Vec as *mut RVec));
    core::ptr::write_unaligned(
        &raw mut (*ip).xDD4_itemVar.hinoarashi.x60 as *mut Vec,
        normal,
    );
}

/// `void it_802D6310(Item_GObj*, s32 coll)` (2D6310): steer `x78` towards the stored normal.
pub unsafe extern "C" fn it_802D6310(gobj: *mut Item_GObj, coll: i32) {
    melee_core::ct!("it_802D6310");
    let ip = ip_of(gobj);

    if coll != 0 {
        let hv = &raw mut (*ip).xDD4_itemVar.hinoarashi;
        let x60 = &raw mut (*hv).x60 as *mut u8;
        let target_angle = atan2f(core::ptr::read_unaligned(x60 as *const f32), (*hv).x64);
        let mut delta = atan2f((*hv).x6C.x, (*hv).x6C.y) - target_angle;
        let angle_factor: f32;

        item_clamp_angle_reverse(&mut delta);

        if delta == 0.0 {
            angle_factor = 0.0;
        } else {
            let angle = lbvector::angle(&*(x60 as *const RVec), &*(&raw const (*hv).x6C as *const RVec));
            let scale: f32;

            if (c_abs(delta) as f64) < M_PI_2 {
                scale = (0.02f32 as f64 * (c_abs(delta) as f64 / M_PI)) as f32;
            } else {
                scale = (0.5f32 as f64 * (c_abs(delta) as f64 / M_PI)) as f32;
            }
            let signed_scale = if delta < 0.0 { -scale } else { scale };
            angle_factor = angle * signed_scale;
        }

        (*hv).x78 -= angle_factor;

        item_clamp_angle(&mut (*hv).x78);
    }
}

/// `void it_802D64B8(HSD_GObj*, Vec3*, u32, f32)` (2D64B8): spawn one flame.
pub unsafe extern "C" fn it_802D64B8(gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: u32, facing_dir: f32) {
    melee_core::ct!("it_802D64B8");
    let ip = ip_of(gobj);
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Hinoarashi_Flame;
    spawn.prev_pos = *pos;
    spawn.prev_pos.z = 0.0;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = arg2 as i32;
    {
        let new_gobj = ext::Item_80268B18(&mut spawn);
        if !new_gobj.is_null() {
            let new_ip = ip_of(new_gobj);
            let attr = (*(*new_ip).xC4_article_data).x4_specialAttributes as *mut itHinoarashiAttributes;
            let speed: f32;
            let mut angle: f32;

            angle = (*attr).x0;
            ext::it_80275158(new_gobj, angle);
            {
                let rand = ext::HSD_Randf();
                let a = fmadds((*attr).x14 - (*attr).x10, rand, (*attr).x10);
                angle = ((if (*new_ip).facing_dir == 1.0 { a } else { -a }) as f64 - M_PI_2) as f32;
            }

            while angle < 0.0 {
                angle = (angle as f64 + TWO_PI) as f32;
            }
            while angle as f64 > TWO_PI {
                angle = (angle as f64 - TWO_PI) as f32;
            }
            (*new_ip).xDD4_itemVar.hinoarashi.x78 = angle;

            {
                let rand = ext::HSD_Randf();
                speed = fmadds((*attr).x8 - (*attr).x4, rand, (*attr).x4);
            }
            (*new_ip).x40_vel.x = speed * cosf(angle);
            (*new_ip).x40_vel.y = speed * sinf(angle);
            (*new_ip).x40_vel.z = 0.0;
            (*new_ip).xDCC_flag.set_b3(0);
            it_802D6674(new_gobj, ext::HSD_Randi(4));
        }
    }
}

/// `void it_802D6674(Item_GObj*, s32)` (2D6674)
pub unsafe extern "C" fn it_802D6674(gobj: *mut Item_GObj, arg1: i32) {
    melee_core::ct!("it_802D6674");
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut scale: f32 = 1.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    ext::efSync_Spawn(
        arg1.wrapping_add(0x44E),
        gobj,
        &[
            jobj as *const c_void,
            &raw const (*ip).facing_dir as *const c_void,
            &raw const scale as *const c_void,
        ],
    );
    ext::it_8026B3A8(gobj);
}
