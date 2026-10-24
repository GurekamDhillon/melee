//! Port of src/melee/it/it_26B1.c: the item accessor layer (kind, attributes, hold kind, owner, flags in the
//! `0xDC8`/`0xDCC`/`0xDCD`/`0xDD0` words, ECB and position probes), the article table (`it_804A0F60`,
//! `it_804D6D38`) and the bomb-rain spawn dispatch. Every function of the C file is here. Callees of other TUs go through `ext` (unset hooks panic
//! "TODO"). Checked against the C by the state oracle `tests/tu_it_26B1.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn Item_80267978(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_80267978(a0) };
    fn Item_80268E40(a0: *mut Item, a1: *mut ItemStateDesc) => unsafe { crate::itlib3::item_80268e40(&mut *a0, &*a1) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn Mex_IndexStageItem(a0: *mut c_void, a1: i32);
    fn Mex_ItRecord(a0: *mut Article, a1: i32) -> i32;
    fn ftLib_8008627C(a0: *mut Vec, a1: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_80086368(a0: *mut Vec, a1: *mut HSD_GObj, a2: f32) -> *mut HSD_GObj;
    fn ftLib_800864A8(a0: *mut Vec, a1: *mut HSD_GObj) -> f32;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_80086644(a0, a1) };
    fn ftLib_800866DC(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086960(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::ftlib::ftLib_80086960(a0) as _ };
    fn ftLib_80086984(a0: *mut HSD_GObj) -> *mut CollData;
    fn ftLib_80086990(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086A4C(a0: *mut HSD_GObj, a1: f32);
    fn ftLib_80086BE0(a0: *mut HSD_GObj) -> u8 => unsafe { crate::tu::ftlib::ftLib_80086BE0(a0) };
    fn gm_8016B238() -> i32;
    fn itFlipper_Spawn(a0: *mut HSD_JObj) -> *mut HSD_GObj => unsafe { crate::tu::itflipper::itFlipper_Spawn(a0) };
    fn itSword_Spawn(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itsword::itSword_Spawn(a0 as _) as _ };
    fn it_8026D324(a0: i32) -> i32 => unsafe { crate::tu::itspawn::it_8026D324(a0) as _ };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272784(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272784(a0) };
    fn it_80272A18(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A18(a0) };
    fn it_80272A3C(a0: *mut HSD_JObj) => unsafe { crate::tu::it_2725::it_80272A3C(a0) };
    fn it_80272D40(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272D40(a0) };
    fn it_80272F7C(a0: *mut HSD_JObj, a1: f32) => unsafe { crate::tu::it_2725::it_80272F7C(a0, a1) };
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_2725::it_8027429C(a0, a1 as _) };
    fn it_802742F4(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32) => unsafe { crate::tu::it_2725::it_802742F4(a0, a1, a2) };
    fn it_80274F10(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_80274f10(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275870(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80275870(&*((*a0).user_data as *mut Item)) as _ };
    fn it_802799E4(a0: *mut HSD_GObj) => unsafe { crate::tu::itanimlist::it_802799E4(a0 as _) };
    fn it_80279BE0(a0: *mut HSD_GObj) => unsafe { crate::tu::itanimlist::it_80279BE0(a0 as _) };
    fn it_8027D670(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itbombhei::it_8027D670(a0 as _) as _ };
    fn it_8027D730(a0: *mut HSD_GObj) => unsafe { crate::tu::itbombhei::it_8027D730(a0 as _) };
    fn it_80283AE4(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32) -> *mut HSD_GObj => unsafe { crate::tu::itheart::it_80283AE4(a0, a1 as _, a2) };
    fn it_80283BD4(a0: *mut HSD_GObj) => unsafe { crate::itkinds::it_80283bd4(&mut *((*a0).user_data as *mut Item)) };
    fn it_802841B4(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32) -> *mut HSD_GObj => unsafe { crate::tu::ittomato::it_802841B4(a0 as _, a1 as _, a2) as _ };
    fn it_8028428C(a0: *mut HSD_GObj) => unsafe { crate::itkinds5::it_8028428c(&mut *((*a0).user_data as *mut Item)) };
    fn it_80284854(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itbat::it_80284854(a0 as _) as _ };
    fn it_80286088(a0: *mut HSD_GObj) -> *mut HSD_GObj => unsafe { crate::tu::itbox::it_80286088(a0 as _) as _ };
    fn it_802896CC(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itkusudama::it_802896CC(a0 as _) };
    fn it_8028FE90(a0: *mut HSD_GObj) => unsafe { crate::tu::itmsbomb::it_8028FE90(a0 as _) };
    fn it_80291BE0(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itsscope::it_80291BE0(a0 as _) };
    fn it_80292D48(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itfflower::it_80292D48(a0) as _ };
    fn it_80294DC0(a0: *mut Vec) -> *mut HSD_GObj => unsafe { crate::tu::itrabbitc::it_80294DC0(a0 as _) };
    fn it_8029D968(a0: *mut HSD_GObj) => unsafe { crate::tu::itlinkbomb::it_8029D968(a0 as _) };
    fn it_802D09D0(a0: *mut HSD_GObj) => unsafe { crate::tu::itmarumine::it_802D09D0(a0 as _) };
    fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B804(a0, a1) };
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32) => unsafe { melee_runtime::tu::lb_00B0::lb_8000BA0C(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = unsafe { core::mem::zeroed() };
    pub static mut it_804A0F60: [*mut Article; 30] = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D08: i32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D0C: i32 = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D24: *mut *mut Article = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D38: *mut *mut Article = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline(always)]
unsafe fn item_of(gobj: *mut HSD_GObj) -> *mut Item {
    (*gobj).user_data as *mut Item
}

/// `it_8026B1D4`: applies the item's damage (adds the speed-scaled bonus when the flag is set).
pub unsafe extern "C" fn it_8026B1D4(gobj: *mut HSD_GObj, itemHitboxUnk: *mut HitCapsule) -> f32 {
    let mut ret = (*itemHitboxUnk).damage;
    let ip = item_of(gobj);
    if (*ip).xDC8_word.flags.x14() != 0 {
        let v = (*ip).x40_vel;
        // x*x + y*y + z*z: the two left-most products fuse in the C build (see RUST_PORT float contraction).
        let sum = fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y));
        let itemSpeed = sqrtf_native(sum);
        let cd = ext::it_804D6D28;
        ret = fmadds(itemSpeed, (*cd).x80_float[5], ret);
        ret += (*cd).x80_float[6];
        if (ret as f64) <= 1.0 {
            ret = 1.0;
            return ret;
        }
    }
    ret
}

/// The MSL `sqrtf` (math_ppc.h): the Newton-Raphson form the game uses; delegated to the native port.
fn sqrtf_native(x: f32) -> f32 {
    melee_runtime::gekko::sqrtf(x)
}

/// `it_8026B294`: copies the item's position.
pub unsafe extern "C" fn it_8026B294(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let temp_item = (*gobj).user_data as *mut Item;
    *pos = (*temp_item).pos;
}

/// `it_8026B2D8`: true when the item attribute marks it heavy (value 1).
pub unsafe extern "C" fn it_8026B2D8(gobj: *mut HSD_GObj) -> bool {
    let temp_item = (*gobj).user_data as *mut Item;
    (*(*temp_item).xCC_item_attr).x0_is_heavy() == 1
}

/// `itGetDamageMultiplier`: the item attribute's damage multiplier.
pub unsafe extern "C" fn itGetDamageMultiplier(gobj: *mut HSD_GObj) -> f32 {
    let temp_item = (*gobj).user_data as *mut Item;
    (*(*temp_item).xCC_item_attr).x1C_damage_mul
}

/// `it_8026B344`: the item's position offset by its `xBCC` vector (x flipped with facing).
pub unsafe extern "C" fn it_8026B344(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let ip = item_of(gobj);
    (*pos).x = fmadds((*ip).facing_dir, (*ip).xBCC_unk.x, (*ip).pos.x);
    (*pos).y = (*ip).pos.y + (*ip).xBCC_unk.y;
    (*pos).z = (*ip).pos.z;
}

/// `itGetGrabRangeX`.
pub unsafe extern "C" fn itGetGrabRangeX(gobj: *mut HSD_GObj) -> f32 {
    (*(*gobj).user_data.cast::<Item>()).xBD4_grabRange.x
}

/// `itGetGrabRangeY`.
pub unsafe extern "C" fn itGetGrabRangeY(gobj: *mut HSD_GObj) -> f32 {
    (*(*gobj).user_data.cast::<Item>()).xBD4_grabRange.y
}

/// `it_8026B390`: sets flag 0x15 of the `0xDC8` word.
pub unsafe extern "C" fn it_8026B390(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDC8_word.flags.set_x15(1);
}

/// `it_8026B3A8`: clears flag 0x15 of the `0xDC8` word.
pub unsafe extern "C" fn it_8026B3A8(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDC8_word.flags.set_x15(0);
}

/// `it_8026B3F8`: stores a fighter article in the article table (m-ex clones record instead).
pub unsafe extern "C" fn it_8026B3F8(article: *mut Article, kind: i32) {
    // TARGET_PC: while an m-ex clone's base onLoad runs in record-only mode the registration is captured.
    if ext::Mex_ItRecord(article, kind) != 0 {
        return;
    }
    *ext::it_804D6D38.offset((kind - It_Kind_Kuriboh) as isize) = article;
}

/// `it_Mex_RegisterIfEmpty`: registers a fighter article only where no loaded fighter already has one.
pub unsafe extern "C" fn it_Mex_RegisterIfEmpty(article: *mut Article, kind: i32) {
    let slot = ext::it_804D6D38.add((kind - It_Kind_Kuriboh) as usize);
    if kind >= It_Kind_Kuriboh && kind < It_PKind_Start && (*slot).is_null() {
        *slot = article;
    }
}

/// `it_ResetStageArticleTable`: clears the stage article descriptors (after a scene heap reset).
pub unsafe extern "C" fn it_ResetStageArticleTable() {
    for i in 0..30 {
        ext::it_804A0F60[i] = null_mut();
    }
}

/// `Item_StageArticleTableTest`: the disc-free regression of the stage article table (returns the number of
/// failed checks).
pub unsafe extern "C" fn Item_StageArticleTableTest() -> i32 {
    let mut saved: [*mut Article; 30] = [null_mut(); 30];
    let mut old: Article = core::mem::zeroed();
    let mut fresh: Article = core::mem::zeroed();
    let mut model: ItemModelDesc = core::mem::zeroed();
    let mut attr: ItemAttr = core::mem::zeroed();
    let mut joint: HSD_Joint = core::mem::zeroed();
    let mut ip: Item = core::mem::zeroed();
    let mut g: HSD_GObj = core::mem::zeroed();
    let mut failed = 0i32;
    for i in 0..30 {
        saved[i] = ext::it_804A0F60[i];
    }
    fresh.x0_common_attr = &raw mut attr;
    fresh.x10_modelDesc = &raw mut model;
    model.x0_joint = &raw mut joint;
    ip.kind = It_Kind_Nokonoko;
    g.user_data = &raw mut ip as *mut c_void;
    for phase in 0..2 {
        old.x10_modelDesc = if phase != 0 { 0xFF73FF73usize as *mut ItemModelDesc } else { null_mut() };
        ext::it_804A0F60[(It_Kind_Nokonoko - It_Kind_Old_Kuri) as usize] = &raw mut old;
        it_ResetStageArticleTable();
        for i in 0..30 {
            if !ext::it_804A0F60[i].is_null() {
                failed += 1;
            }
        }
        it_8026B40C(&raw mut fresh, It_Kind_Nokonoko);
        ext::Item_80267978(&raw mut g);
        if ip.xC4_article_data != &raw mut fresh {
            failed += 1;
        } else if (*(*ip.xC4_article_data).x10_modelDesc).x0_joint != &raw mut joint
            || (*ip.xC4_article_data).x0_common_attr != &raw mut attr
        {
            failed += 1;
        }
    }
    for i in 0..30 {
        ext::it_804A0F60[i] = saved[i];
    }
    failed
}

/// `it_8026B40C`: stores a stage item article (kinds >= 5000 index the stage's item lookup).
pub unsafe extern "C" fn it_8026B40C(article: *mut Article, kind: i32) {
    if kind >= 5000 {
        ext::Mex_IndexStageItem(article as *mut c_void, kind - 5000);
        return;
    }
    ext::it_804A0F60[(kind - It_Kind_Old_Kuri) as usize] = article;
}

/// `it_8026B424`: the item damage math: `(int)(damage * xB8 + xBC)`.
pub unsafe extern "C" fn it_8026B424(damage: i32) -> f32 {
    let itCommonData = ext::it_804D6D28;
    fmadds(damage as f32, (*itCommonData).xB8, (*itCommonData).xBC) as i32 as f32
}

/// `it_8026B47C`: the heal value of a healing item (0 for other kinds).
pub unsafe extern "C" fn it_8026B47C(gobj: *mut HSD_GObj) -> i32 {
    let temp_item = item_of(gobj);
    match (*temp_item).kind {
        It_Kind_Heart => (*temp_item).xDD4_itemVar.heart.xDD4_heal as i32,
        It_Kind_Tomato => (*temp_item).xDD4_itemVar.tomato.heal_amount as i32,
        It_Kind_Foods => (*temp_item).xDD4_itemVar.foods.heal_amount as i32,
        It_Kind_Lucky_Egg => (*temp_item).xDD4_itemVar.egg.heal_amount as i32,
        It_Kind_WhispyHealApple => (*temp_item).xDD4_itemVar.whispyapple.xDD4_heal as i32,
        _ => 0,
    }
}

/// `it_8026B4F0`: true for the healing item kinds.
pub unsafe extern "C" fn it_8026B4F0(gobj: *mut HSD_GObj) -> bool {
    let kind = (*item_of(gobj)).kind;
    matches!(
        kind,
        It_Kind_Heart | It_Kind_Tomato | It_Kind_Foods | It_Kind_Coin | It_Kind_Lucky_Egg | It_Kind_WhispyHealApple
    )
}

/// `it_8026B54C`: the float at 0x0 of the item's special attributes.
pub unsafe extern "C" fn it_8026B54C(gobj: *mut HSD_GObj) -> f32 {
    let temp_item = item_of(gobj);
    let unk_attr = (*(*temp_item).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
    (*unk_attr).x0_float
}

/// `it_8026B560`: identical to `it_8026B54C`.
pub unsafe extern "C" fn it_8026B560(gobj: *mut HSD_GObj) -> f32 {
    let temp_item = item_of(gobj);
    let unk_attr = (*(*temp_item).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
    (*unk_attr).x0_float
}

/// `it_8026B574`: the float at 0x4 of the item's special attributes.
pub unsafe extern "C" fn it_8026B574(gobj: *mut HSD_GObj) -> f32 {
    let temp_item = item_of(gobj);
    let unk_attr = (*(*temp_item).xC4_article_data).x4_specialAttributes as *mut itUnkAttributes;
    (*unk_attr).x4_float
}

/// `it_8026B5E4`: finds the fighter nearest `vector` and moves `vector2` to its point.
pub unsafe extern "C" fn it_8026B5E4(vector: *mut Vec3, vector2: *mut Vec3, gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    let unk_gobj = ext::ftLib_8008627C(vector, gobj);
    if !unk_gobj.is_null() {
        ext::ftLib_800866DC(unk_gobj, vector2);
    }
    unk_gobj
}

/// `it_8026B634`: the fighter gobj for `arg0` within `farg0`, and its point to `arg1`.
pub unsafe extern "C" fn it_8026B634(arg0: *mut Vec3, arg1: *mut Vec3, arg2: *mut HSD_GObj, farg0: f32) -> *mut HSD_GObj {
    let gobj = ext::ftLib_80086368(arg0, arg2, farg0);
    if !gobj.is_null() {
        ext::ftLib_800866DC(gobj, arg1);
    }
    gobj
}

/// `it_8026B684`: the facing direction at `pos` (no fighter).
pub unsafe extern "C" fn it_8026B684(pos: *mut Vec3) -> f32 {
    ext::ftLib_800864A8(pos, null_mut())
}

/// `it_8026B6A8`: facing direction for `pos` with the fighter `arg1`.
pub unsafe extern "C" fn it_8026B6A8(pos: *mut Vec3, arg1: *mut HSD_GObj) -> f32 {
    ext::ftLib_800864A8(pos, arg1)
}

/// `it_8026B6C8`: true for the stage-item kinds.
pub unsafe extern "C" fn it_8026B6C8(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let kind = (*ip).kind;
    if (kind >= It_Kind_Kuriboh && kind < It_Kind_Octarock_Stone)
        || (kind >= It_Kind_Old_Kuri && kind < It_Kind_Arwing_Laser)
    {
        let itemID_3 = (*ip).kind;
        if itemID_3 != It_Kind_Kyasarin_Egg && itemID_3 != It_Kind_WhispyApple {
            return true;
        }
    }
    false
}

/// `it_8026B718`: sets the item's hitlag frames.
pub unsafe extern "C" fn it_8026B718(gobj: *mut HSD_GObj, hitlagFrames: f32) {
    (*item_of(gobj)).xCBC_hitlagFrames = hitlagFrames;
}

/// `it_8026B724`: sets bit 3 of the `0xDC8` word.
pub unsafe extern "C" fn it_8026B724(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDC8_word.flags.set_x3(1);
}

/// `it_8026B73C`: sets 0x5 when 0x7 is set, clears 0x3.
pub unsafe extern "C" fn it_8026B73C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).xDC8_word.flags.x7() != 0 {
        (*ip).xDC8_word.flags.set_x5(1);
    }
    if (*ip).xDC8_word.flags.x3() != 0 {
        (*ip).xDC8_word.flags.set_x3(0);
    }
}

/// `it_8026B774`: true when bit `arg1` of the `0xF` field is set.
pub unsafe extern "C" fn it_8026B774(gobj: *mut HSD_GObj, arg1: u8) -> bool {
    let ip = item_of(gobj);
    ((*ip).xDC8_word.flags.xF() as i32) & (melee_runtime::cabi::shl(1, arg1 as u32) as i32) != 0
}

/// `itGetMotionId`.
pub unsafe extern "C" fn itGetMotionId(gobj: *mut HSD_GObj) -> i32 {
    (*item_of(gobj)).msid as i32
}

/// `itGetTeamId`.
pub unsafe extern "C" fn itGetTeamId(gobj: *mut HSD_GObj) -> u8 {
    (*item_of(gobj)).x20_team_id
}

/// `it_8026B7BC`: bit 0x14 of the `0xDC8` word.
pub unsafe extern "C" fn it_8026B7BC(gobj: *mut HSD_GObj) -> i32 {
    (*item_of(gobj)).xDC8_word.flags.x14() as i32
}

/// `it_8026B7CC`: the item's `x1C` word.
pub unsafe extern "C" fn it_8026B7CC(gobj: *mut HSD_GObj) -> i32 {
    (*item_of(gobj)).x1C
}

/// `it_8026B7D8`: common-data global `it_804D6D0C`.
pub unsafe extern "C" fn it_8026B7D8() -> i32 {
    ext::it_804D6D0C
}

/// `it_8026B7E0`: common-data global `it_804D6D08`.
pub unsafe extern "C" fn it_8026B7E0() -> i32 {
    ext::it_804D6D08
}

/// `it_8026B7E8`: bit 1 of the `0xDC8` word.
pub unsafe extern "C" fn it_8026B7E8(gobj: *mut HSD_GObj) -> i32 {
    (*item_of(gobj)).xDC8_word.flags.x1() as i32
}

/// `RunCallbackUnk` (static inline): calls the interaction proc when set.
unsafe fn RunCallbackUnk(proc_: HSD_GObjInteraction, gobj0: *mut HSD_GObj, gobj1: *mut HSD_GObj) {
    if let Some(f) = proc_ {
        f(gobj0, gobj1);
    }
}

/// `it_8026B894`: clears every reference to `ref_gobj` in the item; true when it was the owner.
pub unsafe extern "C" fn it_8026B894(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let mut result = false;
    if (*ip).owner == ref_gobj {
        (*ip).owner = null_mut();
        result = true;
    }
    if (*ip).xC64_reflectGObj == ref_gobj {
        (*ip).xC64_reflectGObj = null_mut();
    }
    if (*ip).xC90_absorbGObj == ref_gobj {
        (*ip).xC90_absorbGObj = null_mut();
    }
    if (*ip).xCEC_fighterGObj == ref_gobj {
        (*ip).xCEC_fighterGObj = null_mut();
        (*ip).xCB0_source_ply = 6;
    }
    if (*ip).xCF4_fighterGObjUnk == ref_gobj {
        (*ip).xCF4_fighterGObjUnk = null_mut();
    }
    if (*ip).toucher == ref_gobj {
        (*ip).toucher = null_mut();
    }
    result
}

/// `it_8026B924`: the item's timer for the kinds that have one, -1 otherwise.
pub unsafe extern "C" fn it_8026B924(gobj: *mut HSD_GObj) -> i32 {
    let ip = item_of(gobj);
    let mut result = -1;
    match (*ip).kind {
        It_Kind_Freeze | It_Kind_Foods | It_Kind_MSBomb | It_Kind_Flipper | It_Kind_LipStick | It_Kind_Harisen => {}
        It_Kind_S_Scope | It_Kind_StarRod | It_Kind_L_Gun | It_Kind_F_Flower => {
            result = (*ip).xD4C;
        }
        _ => {}
    }
    result
}

/// `it_8026B960`: the timer of a bomb-hei or link-bomb (-1 when not counting).
pub unsafe extern "C" fn it_8026B960(gobj: *mut HSD_GObj) -> f32 {
    let mut unk_timer = -1.0f32;
    let ip = item_of(gobj);
    match (*ip).kind {
        It_Kind_BombHei => {
            if (*ip).msid as i32 != 0xB {
                unk_timer = (*ip).xDD4_itemVar.bombhei.xDEC;
            }
        }
        It_Kind_Link_Bomb => {
            if (*ip).msid as i32 != 5 {
                unk_timer = (*ip).xD44_lifeTimer;
            }
        }
        _ => {}
    }
    unk_timer
}

/// `it_8026B9A8`: moves the item to another character's hold (Zelda <-> Sheik transformation).
pub unsafe extern "C" fn it_8026B9A8(gobj: *mut HSD_GObj, arg1: *mut HSD_GObj, arg2: Fighter_Part) {
    let mut vec: Vec3 = core::mem::zeroed();
    let ip = item_of(gobj);
    let states = (*(*ip).xC4_article_data).xC_itemStates;
    let jobj0 = (*gobj).hsd_obj as *mut HSD_JObj;
    let state_desc = (*states).x0_itemStateDesc.as_mut_ptr().add((*ip).anim_id as usize);
    vec.x = 0.0;
    vec.y = 0.0;
    vec.z = 0.0;
    ext::it_8027429C(gobj, &raw mut vec);
    ext::it_802742F4(gobj, arg1, arg2);

    (*ip).xD54_throwNum -= 1;
    (*ip).xDC8_word.flags.set_x14(0);

    if (*ip).anim_id != -1 {
        (*ip).xD0_itemStateDesc = state_desc;
        if !(*ip).xD0_itemStateDesc.is_null() {
            ext::HSD_JObjRemoveAnimAll(jobj0);
            if !(*ip).xC8_joint.is_null() {
                let jobj1 = if jobj0.is_null() { null_mut() } else { (*jobj0).child };
                ext::lb_8000B804(jobj1, (*(*ip).xC8_joint).child);
            }
            let temp_stateDesc = (*ip).xD0_itemStateDesc;
            ext::HSD_JObjAddAnimAll(
                jobj0,
                (*temp_stateDesc).x0_anim_joint,
                (*temp_stateDesc).x4_matanim_joint,
                (*temp_stateDesc).x8_parameters,
            );
            ext::lb_8000BA0C(jobj0, (*ip).x5D0_animFrameSpeed);
            ext::HSD_JObjReqAnimAll(jobj0, 0.0);
            ext::Item_80268E40(ip, state_desc);
        }
        ext::HSD_JObjAnimAll(jobj0);
        ext::it_80279BE0(gobj);
        ext::it_802799E4(gobj);
        return;
    }
    ext::HSD_JObjRemoveAnimAll(jobj0);
    (*ip).x524_cmd.anon_8.u = null_mut();
}

/// `it_8026BAE8`: sets the item's scale to `scl_mul` times its attribute scale.
pub unsafe extern "C" fn it_8026BAE8(gobj: *mut HSD_GObj, scl_mul: f32) {
    let ip = item_of(gobj);
    let item_jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let scl = scl_mul * (*(*ip).xCC_item_attr).x60_scale;
    (*ip).scl = scl;
    ext::it_80272F7C(item_jobj, scl);
}

/// `it_8026BB20`: clears the JObj flags on the item model.
pub unsafe extern "C" fn it_8026BB20(gobj: *mut HSD_GObj) {
    ext::it_80272A18((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `it_8026BB44`: sets the JObj flags on the item model.
pub unsafe extern "C" fn it_8026BB44(gobj: *mut HSD_GObj) {
    ext::it_80272A3C((*gobj).hsd_obj as *mut HSD_JObj);
}

/// `it_8026BB68`: adjusts the position to the fighter's bone.
pub unsafe extern "C" fn it_8026BB68(fighter_gobj: *mut HSD_GObj, pos: *mut Vec3) {
    ext::ftLib_80086990(fighter_gobj, pos);
}

/// `it_8026BB88`: the item's position plus half the ECB's vertical extent.
pub unsafe extern "C" fn it_8026BB88(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let ip = item_of(gobj);
    let ecb = &raw mut (*ip).x378_itemColl.ecb;
    let offset_xz = 0.0f32;
    let offset_y = 0.5f32 * ((*ecb).top.y + (*ecb).bottom.y);
    (*pos).x = (*ip).pos.x + offset_xz;
    (*pos).y = (*ip).pos.y + offset_y;
    (*pos).z = (*ip).pos.z + offset_xz;
}

/// `it_8026BBCC`: the last position plus half the ECB's vertical extent.
pub unsafe extern "C" fn it_8026BBCC(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let ip = item_of(gobj);
    let coll_data = &raw mut (*ip).x378_itemColl;
    let ecb = &raw mut (*coll_data).xE4_ecb;
    let offset_xz = 0.0f32;
    let offset_y = 0.5f32 * ((*ecb).top.y + (*ecb).bottom.y);
    (*pos).x = (*coll_data).last_pos.x + offset_xz;
    (*pos).y = (*coll_data).last_pos.y + offset_y;
    (*pos).z = (*coll_data).last_pos.z + offset_xz;
}

/// `it_8026BC14`: decrements the owner's hitlag while the item is held.
pub unsafe extern "C" fn it_8026BC14(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if !(*ip).owner.is_null() && ext::ftLib_80086960((*ip).owner) != 0 {
        ext::ftLib_80086A4C((*ip).owner, (*ip).xCBC_hitlagFrames - 1.0);
    }
}

/// `itGetOwner`.
pub unsafe extern "C" fn itGetOwner(gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    (*item_of(gobj)).owner
}

/// `itGetAttackId`.
pub unsafe extern "C" fn itGetAttackId(gobj: *mut HSD_GObj) -> i32 {
    (*(*gobj).user_data.cast::<Item>()).xD88_attackID
}

/// `it_8026BC90`: the ECB-adjusted position when the item's owner is a fighter.
pub unsafe extern "C" fn it_8026BC90(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    (*pos).z = 0.0;
    (*pos).y = 0.0;
    (*pos).x = 0.0;
    if !gobj.is_null() && ext::ftLib_80086960(gobj) != 0 {
        ext::ftLib_80086644(gobj, pos);
    }
}

/// `it_8026BCF4`: clears `xDCD` bit 2.
pub unsafe extern "C" fn it_8026BCF4(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCD_flag.anon_0.set_b2(0);
}

/// `it_8026BD0C`: sets `xDCD` bit 2.
pub unsafe extern "C" fn it_8026BD0C(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCD_flag.anon_0.set_b2(1);
}

/// `it_8026BD24`: sets `xDD0` bit 3.
pub unsafe extern "C" fn it_8026BD24(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDD0_flag.anon_0.set_b3(1);
}

/// `it_8026BD3C`: sets `xDCC` bit 3.
pub unsafe extern "C" fn it_8026BD3C(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCC_flag.set_b3(1);
}

/// `it_8026BD54`: clears `xDCC` bit 3.
pub unsafe extern "C" fn it_8026BD54(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCC_flag.set_b3(0);
}

/// `it_8026BD6C`: sets `xDCD` bit 3.
pub unsafe extern "C" fn it_8026BD6C(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCD_flag.anon_0.set_b3(1);
}

/// `it_8026BD84`: sets `xDCD` bit 4.
pub unsafe extern "C" fn it_8026BD84(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDCD_flag.anon_0.set_b4(1);
}

/// `it_8026BD9C`: sets bit 0x1A of the `0xDC8` word.
pub unsafe extern "C" fn it_8026BD9C(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDC8_word.flags.set_x1A(1);
}

/// `it_8026BDB4`: clears bit 0x1A of the `0xDC8` word.
pub unsafe extern "C" fn it_8026BDB4(gobj: *mut HSD_GObj) {
    (*item_of(gobj)).xDC8_word.flags.set_x1A(0);
}

/// `it_8026BDCC`: the flag set that makes an item a free projectile (several inlined setters).
pub unsafe extern "C" fn it_8026BDCC(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xDD0_flag.anon_0.set_b3(0);
    (*ip).xDCC_flag.set_b3(1);
    (*ip).xDCD_flag.anon_0.set_b3(0);
    (*ip).xDCD_flag.anon_0.set_b4(0);
    (*ip).xDC8_word.flags.set_x1A(1);
}

/// `it_8026BE28`: the inverse of `it_8026BDCC`.
pub unsafe extern "C" fn it_8026BE28(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xDD0_flag.anon_0.set_b3(1);
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xDCD_flag.anon_0.set_b3(1);
    (*ip).xDCD_flag.anon_0.set_b4(1);
    (*ip).xDC8_word.flags.set_x1A(0);
}

/// `it_8026BE84`: spawns the bomb-rain item for `bobOmbRain`'s kind and applies its flags.
pub unsafe extern "C" fn it_8026BE84(bobOmbRain: *mut BobOmbRain) -> *mut HSD_GObj {
    let mut gobj: *mut HSD_GObj = null_mut();
    if ext::gm_8016B238() != 0 && (*bobOmbRain).x14 != 6 {
        return null_mut();
    }
    let bobomb_id = (*bobOmbRain).x14;
    match bobomb_id {
        1 => {
            if ext::it_8026D324(It_Kind_Box) != 0 {
                gobj = ext::it_80286088((*bobOmbRain).x0);
            }
        }
        20 => {
            gobj = ext::itFlipper_Spawn((*bobOmbRain).x4);
        }
        8 => {
            gobj = ext::it_80283AE4((*bobOmbRain).x0, &raw mut (*bobOmbRain).x8_vec, (*bobOmbRain).x18);
            if !gobj.is_null() {
                let ip = item_of(gobj);
                match (*ip).kind {
                    It_Kind_Heart => ext::it_80283BD4(gobj),
                    It_Kind_Tomato => ext::it_8028428C(gobj),
                    _ => {}
                }
            }
        }
        9 => {
            gobj = ext::it_802841B4((*bobOmbRain).x0, &raw mut (*bobOmbRain).x8_vec, (*bobOmbRain).x18);
            if !gobj.is_null() {
                let ip = item_of(gobj);
                match (*ip).kind {
                    It_Kind_Heart => ext::it_80283BD4(gobj),
                    It_Kind_Tomato => ext::it_8028428C(gobj),
                    _ => {}
                }
            }
        }
        4 => {
            if ext::it_8026D324(It_Kind_Kusudama) != 0 {
                gobj = ext::it_802896CC(&raw mut (*bobOmbRain).x8_vec);
                if !gobj.is_null() {
                    ext::it_80274F10(gobj);
                }
            }
        }
        12 => {
            gobj = ext::itSword_Spawn(&raw mut (*bobOmbRain).x8_vec);
            if !gobj.is_null() {
                ext::it_80274F10(gobj);
            }
        }
        21 => {
            gobj = ext::it_80291BE0(&raw mut (*bobOmbRain).x8_vec);
            if !gobj.is_null() {
                ext::it_80274F10(gobj);
            }
        }
        25 => {
            gobj = ext::it_80292D48(&raw mut (*bobOmbRain).x8_vec);
            if !gobj.is_null() {
                ext::it_80274F10(gobj);
            }
        }
        31 => {
            gobj = ext::it_80294DC0(&raw mut (*bobOmbRain).x8_vec);
            if !gobj.is_null() {
                ext::it_80274F10(gobj);
            }
        }
        11 => {
            gobj = ext::it_80284854(&raw mut (*bobOmbRain).x8_vec);
            if !gobj.is_null() {
                ext::it_80274F10(gobj);
            }
        }
        6 => {
            // The C falls through into `default` here (gobj = NULL).
            ext::it_8027D670(&raw mut (*bobOmbRain).x8_vec);
            gobj = null_mut();
        }
        _ => {
            gobj = null_mut();
        }
    }

    if !gobj.is_null() && (*bobOmbRain).x1C.anon_0.b0() != 0 {
        let ip = item_of(gobj);
        (*ip).xDD0_flag.anon_0.set_b3(1);
        (*ip).xDCC_flag.set_b3(0);
        (*ip).xDCD_flag.anon_0.set_b3(1);
        (*ip).xDCD_flag.anon_0.set_b4(1);
        (*ip).xDC8_word.flags.set_x1A(0);
    }
    gobj
}

/// `it_8026C100`: the item's collision data (the fighter's for a fighter-owned item, else the item's own).
pub unsafe extern "C" fn it_8026C100(gobj: *mut HSD_GObj) -> *mut CollData {
    let mut collDataPtr: *mut CollData = null_mut();
    match ext::it_80272D40(gobj) {
        0 => {
            collDataPtr = ext::ftLib_80086984(gobj);
        }
        1 => {
            let ip = item_of(gobj);
            collDataPtr = &raw mut (*ip).x378_itemColl;
        }
        _ => {}
    }
    collDataPtr
}

/// `it_8026C16C`: a hammer's head breaks off when it is headless or out of durability.
pub unsafe extern "C" fn it_8026C16C(gobj: *mut HSD_GObj, is_headless: i32) {
    let ip = item_of(gobj);
    if (*ip).xD4C <= 0 {
        ext::it_802725D4(gobj);
    } else if is_headless != 0 {
        ext::it_80272784(gobj);
    } else {
        ext::it_802725D4(gobj);
    }
}

/// `it_8026C1B4`: true when the item has a hitbox.
pub unsafe extern "C" fn it_8026C1B4(gobj: *mut HSD_GObj) -> i32 {
    ext::it_80275870(gobj)
}

/// `it_8026C1D4`: the L-Gun's timer (from its special attributes).
pub unsafe extern "C" fn it_8026C1D4() -> u32 {
    let lgun = (**ext::it_804D6D24.add(It_Kind_L_Gun as usize)).x4_specialAttributes as *mut itLGun_ItemVars;
    (*lgun).timer as u32
}

/// `it_8026C1E8`: true when the item is free (no grab victim, and not attached to an owner).
pub unsafe extern "C" fn it_8026C1E8(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    if !(*ip).grab_victim.is_null() || ((*ip).xDC8_word.flags.x13() != 0 && !(*ip).owner.is_null()) {
        return false;
    }
    true
}

/// `it_8026C220`: records the owner's port number on the item.
pub unsafe extern "C" fn it_8026C220(gobj: *mut HSD_GObj, fighter_gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xCB0_source_ply = ext::ftLib_80086BE0(fighter_gobj) as _;
}

/// `it_8026C258`: the nearest item to `pos` that a Samus homing missile can lock onto.
pub unsafe extern "C" fn it_8026C258(pos: *mut Vec3, facing_dir: f32) -> *mut HSD_GObj {
    let mut min_sq_dist = f32::MAX;
    let mut result: *mut HSD_GObj = null_mut();
    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        let ip = item_of(cur);
        let hold_kind = (*ip).hold_kind as u32;
        if (hold_kind == ITEM_UNK_MATO || hold_kind == ITEM_UNK_LOCKON || (hold_kind == ITEM_UNK_ENEMY) as i32 != 0 || hold_kind == ITEM_UNK_7)
            && (*ip).grab_victim.is_null()
            && (!((*ip).xDC8_word.flags.x13() != 0) || (*ip).owner.is_null())
            && (facing_dir as f64 != -1.0 || !((*ip).pos.x > (*pos).x))
            && (facing_dir as f64 != 1.0 || !((*ip).pos.x < (*pos).x))
        {
            let dist_x = (*pos).x - (*ip).pos.x;
            let dist_y = (*pos).y - (*ip).pos.y;
            let sq_dist = fmadds(dist_x, dist_x, dist_y * dist_y);
            if sq_dist < min_sq_dist {
                min_sq_dist = sq_dist;
                result = cur;
            }
        }
        cur = (*cur).next;
    }
    result
}

/// `it_8026C334`: the item's position plus its ECB bottom offset.
pub unsafe extern "C" fn it_8026C334(gobj: *mut HSD_GObj, pos: *mut Vec3) {
    let ip = item_of(gobj);
    let offset_xz = 0.0f32;
    let offset_y = (*ip).x378_itemColl.ecb.bottom.y;
    (*pos).x = (*ip).pos.x + offset_xz;
    (*pos).y = (*ip).pos.y + offset_y;
    (*pos).z = (*ip).pos.z + offset_xz;
}

/// `it_8026C368`: runs the explosion callback of a bomb that was flagged by its collision (bit 7).
pub unsafe extern "C" fn it_8026C368(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).x378_itemColl.x34_flags.b7() != 0 {
        match (*ip).kind {
            It_Kind_BombHei => {
                ext::it_8027D730(gobj);
                return;
            }
            It_Kind_MSBomb => {
                ext::it_8028FE90(gobj);
                return;
            }
            It_Kind_Link_Bomb | It_Kind_CLink_Bomb => {
                ext::it_8029D968(gobj);
                return;
            }
            It_PKind_Marumine => {
                ext::it_802D09D0(gobj);
                return;
            }
            _ => {
                (*ip).destroy_type = 0;
                ext::Item_8026A8EC(gobj);
            }
        }
    }
}

// ---- functions of the C file that the first pass left to itlib (their C signatures, ported here) ----

/// `itIsHeavy`: the attribute's `x0_is_heavy` bit.
pub unsafe extern "C" fn itIsHeavy(gobj: *mut HSD_GObj) -> bool {
    let temp_item = (*gobj).user_data as *mut Item;
    (*(*temp_item).xCC_item_attr).x0_is_heavy() != 0
}

/// `itGetKind`.
pub unsafe extern "C" fn itGetKind(gobj: *mut HSD_GObj) -> ItemKind {
    (*item_of(gobj)).kind
}

/// `it_8026B30C`: the attribute's `x0_78` field.
pub unsafe extern "C" fn it_8026B30C(gobj: *mut HSD_GObj) -> enum_t {
    (*(*item_of(gobj)).xCC_item_attr).x0_78() as enum_t
}

/// `itGetHoldKind`: the attribute's `x0_hold_kind` field.
pub unsafe extern "C" fn itGetHoldKind(gobj: *mut HSD_GObj) -> Item_HoldKinds {
    (*(*item_of(gobj)).xCC_item_attr).x0_hold_kind() as Item_HoldKinds
}

/// `it_8026B3C0`: counts the item gobjs of `kind`.
pub unsafe extern "C" fn it_8026B3C0(kind: ItemKind) -> i32 {
    let mut i = 0;
    let mut unk = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !unk.is_null() {
        let temp_item = (*unk).user_data as *mut Item;
        if (*temp_item).kind == kind {
            i += 1;
        }
        unk = (*unk).next;
    }
    i
}

/// `it_8026B588`: `it_804D6D28->xDC`.
pub unsafe extern "C" fn it_8026B588() -> i32 {
    (*ext::it_804D6D28).xDC
}

/// `it_8026B594`: true for a projectile item with no ammo left.
pub unsafe extern "C" fn it_8026B594(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let kind = (*ip).kind;
    if kind == It_Kind_L_Gun || kind == It_Kind_S_Scope || kind == It_Kind_StarRod || kind == It_Kind_LipStick || kind == It_Kind_F_Flower {
        if (*ip).xD4C <= 0 {
            return true;
        }
    }
    false
}

/// `it_8026B7F8`: runs each item's `evt_unk` callback for `fighter_gobj` and releases the items it owned.
pub unsafe extern "C" fn it_8026B7F8(fighter_gobj: *mut HSD_GObj) {
    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        let ip = item_of(cur);
        let owner = (*ip).owner;
        if let Some(proc_) = (*(*ip).xB8_itemLogicTable).evt_unk {
            proc_(cur, fighter_gobj);
        }
        if (*ip).xDC8_word.flags.x13() != 0 && owner == fighter_gobj {
            ext::Item_8026A8EC(cur);
        }
        cur = (*cur).next;
    }
}

/// `it_8026BC68`: `xDD0_flag.b0`.
pub unsafe extern "C" fn it_8026BC68(gobj: *mut HSD_GObj) -> bool {
    (*item_of(gobj)).xDD0_flag.anon_0.b0() != 0
}

/// `it_8026C3FC`: sets bit 3 of the `0xDC8` word of every item gobj.
pub unsafe extern "C" fn it_8026C3FC() {
    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        it_8026B724(cur);
        cur = (*cur).next;
    }
}

/// `it_8026C42C`: the `it_8026B73C` flag toggle for every item gobj.
pub unsafe extern "C" fn it_8026C42C() {
    let mut cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !cur.is_null() {
        let ip = item_of(cur);
        if (*ip).xDC8_word.flags.x7() != 0 {
            (*ip).xDC8_word.flags.set_x5(1);
        }
        if (*ip).xDC8_word.flags.x3() != 0 {
            (*ip).xDC8_word.flags.set_x3(0);
        }
        cur = (*cur).next;
    }
}
