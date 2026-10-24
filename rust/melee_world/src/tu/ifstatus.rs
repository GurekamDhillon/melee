//! Port of src/melee/if/ifstatus.c (built `TARGET_PC`): the per-player damage percent plates and their
//! mark (series emblem) objects. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifstatus.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_mut, unused_assignments)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) = |a0: *mut HSD_AObj, a1: f32| { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetRate(a0 as _, a1 as _); } };
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void = |a0: *mut HSD_Archive, a1: *const i8| unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390CAC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_80390CAC(a0 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjRemoveAnim(a0: *mut HSD_JObj);
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32;
    fn HSD_TObjAddAnimAll(a0: *mut HSD_TObj, a1: *mut _HSD_TexAnim) = |a0: *mut HSD_TObj, a1: *mut _HSD_TexAnim| { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjAddAnimAll(a0 as _, a1 as _); } };
    fn HSD_TObjAnim(a0: *mut HSD_TObj) = |a0: *mut HSD_TObj| { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjAnim(a0 as _); } };
    fn HSD_TObjReqAnimAll(a0: *mut HSD_TObj, a1: f32) = |a0: *mut HSD_TObj, a1: f32| { unsafe { let _ = melee_runtime::tu::tobj::HSD_TObjReqAnimAll(a0 as _, a1 as _); } };
    fn Mex_InsigniaForExt(a0: i32) -> i32;
    fn Mex_PortCKindToExt(a0: i32) -> i32;
    fn Player_GetDamage(a0: i32) -> i32;
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj;
    fn Player_GetMoreFlagsBit2(a0: i32) -> i32;
    fn Player_GetPlayerCharacter(a0: i32) -> u32;
    fn Player_GetPlayerId(a0: i32) -> i32;
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind;
    fn Player_GetRemainingHP(a0: i32) -> i32;
    fn Player_GetStocks(a0: i32) -> i32;
    fn Player_GetTeam(a0: i32) -> i32;
    fn Script_StatusHUDVisible() -> i32;
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn gmVs_GetSceneController() -> *mut VsSceneController;
    fn gmVs_GetSceneState() -> *mut VsSceneState;
    fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32;
    fn gm_80160968(a0: u32) -> _GXColor;
    fn gm_80168B34(a0: u32, a1: i32, a2: i32) -> f32;
    fn gm_8016B168() -> i32;
    fn gm_8016B238() -> i32;
    fn gm_8016B8D4(a0: i32, a1: u8);
    fn gm_GetStartMeleeRules() -> *mut StartMeleeRules;
    fn ifAll_802F343C(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::ifall::ifAll_802F343C(a0 as _); } };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_GetPlayerHUDPosition(a0: i32) -> *mut Vec = |a0: i32| unsafe { crate::tu::ifall::ifAll_GetPlayerHUDPosition(a0 as _) as _ };
    fn ifStock_802F98E8(a0: u8, a1: u8) = |a0: u8, a1: u8| { unsafe { let _ = crate::tu::ifstock::ifStock_802F98E8(a0 as _, a1 as _); } };
    fn ifStock_802FB650(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::ifstock::ifStock_802FB650(a0 as _); } };
    fn ifStock_802FB6AC(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::ifstock::ifStock_802FB6AC(a0 as _); } };
    fn if_802F7AF8(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::if_2F72::if_802F7AF8(a0 as _); } };
    fn if_802F7BB4(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::if_2F72::if_802F7BB4(a0 as _); } };
    fn if_802F7C30(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::if_2F72::if_802F7C30(a0 as _); } };
    fn if_802F7D08(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::if_2F72::if_802F7D08(a0 as _); } };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]);
    fn lbDvd_8001819C(a0: *const i8) -> *mut HSD_Archive = |a0: *const i8| unsafe { melee_runtime::tu::lbdvd::lbDvd_8001819C(a0 as _) as _ };
    fn lb_8000B09C(a0: *mut HSD_JObj) -> i32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::lb_00B0::lb_8000B09C(a0 as _) as _ };
    fn lb_8000C07C(a0: *mut HSD_JObj, a1: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint) = |a0: *mut HSD_JObj, a1: i32, a2: *mut *mut HSD_AnimJoint, a3: *mut *mut HSD_MatAnimJoint, a4: *mut *mut HSD_ShapeAnimJoint| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C07C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn memzero(a0: *mut c_void, a1: i32) = super::memzero_default;
    fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: u32);
    fn un_802FF364(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::if_2FF2::un_802FF364(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn if_802F7AF8(a0: i32) { super::if_802F7AF8(a0) }
        pub unsafe extern "C" fn if_802F7BB4(a0: i32) { super::if_802F7BB4(a0) }
        pub unsafe extern "C" fn if_802F7C30(a0: i32) { super::if_802F7C30(a0) }
        pub unsafe extern "C" fn if_802F7D08(a0: i32) { super::if_802F7D08(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT


crate::jobj_inlines!();

use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i8;

/// `AT_RE_HUD_DAMAGE` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_DAMAGE: i32 = 0;

/// `UnkX` (ifstatus.c): the death-animation view of an `IfDamageState` (flags at 0x10, the two
/// float vectors at 0x34 / 0x44, the joints at 0x54 in the GameCube layout). Same record here.
pub type UnkX = IfDamageState;

const HUNDREDS: usize = 0;
const TENS: usize = 1;
const ONES: usize = 2;
const PERCENT: usize = 3;

/// Color endpoints for damage percentage interpolation: start (low damage).
pub static mut ifStatus_804D57A8: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
/// End color (high damage).
pub static mut ifStatus_804D57AC: [u8; 4] = [0x50, 0x00, 0x00, 0xFF];
/// 4D6D60
pub static mut ifStatus_804D6D60: u8 = 0;
/// 4D6D61
pub static mut ifStatus_804D6D61: i8 = 0;

pub static num_models_name: [u8; 20] = *b"DmgNum_scene_models\0";
pub static mrk_models_name: [u8; 20] = *b"DmgMrk_scene_models\0";

/// 4A10C8
pub static mut ifStatus_HudInfo: HudIndex = unsafe { core::mem::zeroed() };

/// `memzero` (lb_00B0.c): the real behaviour, used unless a test installs a hook.
pub fn memzero_default(mem: *mut c_void, size: i32) {
    unsafe { core::ptr::write_bytes(mem as *mut u8, 0, size as usize) }
}

pub unsafe extern "C" fn ifStatus_GetHUDInfo() -> *mut HudIndex {
    &raw mut ifStatus_HudInfo
}

/// `HSD_ASSERT`-style `__assert("jobj.h", line, "jobj")` on a null joint.
#[inline]
pub fn assert_not_null(j: *mut HSD_JObj, line: i32) {
    if j.is_null() {
        panic!("__assert jobj.h:{line} jobj");
    }
}

pub unsafe fn foo(a: f32, b: f32) -> f32 {
    let result;
    if ifStatus_804D6D61 != 0 {
        result = -a - b;
        ifStatus_804D6D61 = 0;
    } else {
        result = a + b;
        ifStatus_804D6D61 = 1;
    }
    result
}

pub unsafe fn jobj_flagCheckSetMtxDirtySub(jobj: *mut HSD_JObj) {
    if (*jobj).flags & 0x02000000 == 0 && !jobj.is_null() {
        assert_not_null(jobj, 0x234);
        if (((*jobj).flags & 0x800000 == 0 && (*jobj).flags & 0x40 != 0) as i32) == 0 {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe fn jobj_get(_jobj: *mut HSD_JObj, value: *mut UnkX, i: usize) -> *mut HSD_JObj {
    (*value).jobjs[i]
}

pub unsafe extern "C" fn ifStatus_PercentOnDeathAnimationThink(value: *mut UnkX, _arg1: i32, _arg2: i32) {
    if (*value).flags.randomize_velocity() != 0 {
        for i in 0..4usize {
            let a = 0.6083f32 * ext::HSD_Randf();
            (*value).velocity_x[i] = foo(a, 0.3041);
            (*value).velocity_y[i] = fmadds(0.811, ext::HSD_Randf(), 1.2165);
        }
        (*value).flags.set_randomize_velocity(0);
        return;
    }

    for i in 0..4usize {
        let mut jobj_r30 = (*value).jobjs[i];
        assert_not_null(jobj_r30, 993);
        if melee_types::inline::fabsf_bitwise((*jobj_r30).translate.x) < 100.0 {
            let f = (*value).velocity_x[i];
            jobj_r30 = jobj_get(jobj_r30, value, i);
            assert_not_null(jobj_r30, 1102);
            (*jobj_r30).translate.x += f;
            jobj_flagCheckSetMtxDirtySub(jobj_r30);
        }
        jobj_r30 = jobj_get(jobj_r30, value, i);
        assert_not_null(jobj_r30, 1006);

        if (*jobj_r30).translate.y > -100.0 {
            let f = (*value).velocity_y[i];
            jobj_r30 = jobj_get(jobj_r30, value, i);
            jobj_r30 = jobj_get(jobj_r30, value, i);
            assert_not_null(jobj_r30, 1114);
            (*jobj_r30).translate.y += f;
            jobj_flagCheckSetMtxDirtySub(jobj_r30);
            (*value).velocity_y[i] -= 0.2028;
        }
    }
}

pub unsafe fn offset_rand() -> f32 {
    ext::HSD_Randf() - 0.5
}

pub unsafe extern "C" fn ifStatus_802F4B84(state: *mut IfDamageState, _is_stamina: i32) {
    let frames = (*state).frames_of_shake_remaining;
    if frames != 0 {
        if frames == 1 {
            for i in 0..4usize {
                HSD_JObjSetTranslateX((*state).jobjs[i], (*state).translation_x[i]);
                HSD_JObjSetTranslateY((*state).jobjs[i], (*state).translation_y[i]);
            }
            (*state).frames_of_shake_remaining = 0;
            return;
        }
        let new_var = (*state).damage_from_last_attack;
        let mut mag: f32 = 0.1014 * (new_var as f32);
        if mag < 0.1014 {
            mag = 0.1014;
        } else if mag > 1.5207 {
            mag = 1.5207;
        }
        for i in 0..4usize {
            let t = 2.0f32 * offset_rand();
            let mut ox = mag * t;
            let t = 2.0f32 * offset_rand();
            let mut oy = mag * t;
            if ox < 0.0 {
                ox -= 0.2028;
            } else {
                ox += 0.2028;
            }
            if oy < 0.0 {
                oy -= 0.2028;
            } else {
                oy += 0.2028;
            }
            HSD_JObjSetTranslateX((*state).jobjs[i], (*state).translation_x[i] + ox);
            HSD_JObjSetTranslateY((*state).jobjs[i], (*state).translation_y[i] + oy);
        }
        (*state).frames_of_shake_remaining = (*state).frames_of_shake_remaining.wrapping_sub(1);
    }
}

/// `damage_percent % m / d` as the C int arithmetic.
#[inline]
fn dig(damage: i16, m: i32, d: i32) -> i32 {
    (damage as i32 % m) / d
}


type AnimBase = *mut *mut *mut HSD_MatAnimJoint;
/// `(*anim_base)[0]->child->matanim->texanim`
pub unsafe fn tex0(anim_base: AnimBase) -> *mut _HSD_TexAnim {
    (*(*(*(**anim_base)).child).matanim).texanim
}
/// `(*anim_base)[0]->child->next->next->next->matanim->texanim`
pub unsafe fn tex3(anim_base: AnimBase) -> *mut _HSD_TexAnim {
    (*(*(*(*(*(*(**anim_base)).child).next).next).next).matanim).texanim
}

pub unsafe fn digit_anim(state: *mut IfDamageState, place: usize, anim_base: *mut *mut *mut HSD_MatAnimJoint, digit: u8) {
    let j = (*state).jobjs[place];
    let tobj = (*(*(*j).u.dobj).mobj).tobj;
    ext::HSD_TObjAddAnimAll(tobj, tex0(anim_base));
    ext::HSD_TObjReqAnimAll((*(*(*(*state).jobjs[place]).u.dobj).mobj).tobj, 2.0f32 * digit as f32);
    ext::HSD_AObjSetRate((*(*(*(*(*state).jobjs[place]).u.dobj).mobj).tobj).aobj, 0.0);
}

pub unsafe fn ifStatus_InitDamageDigits(state: *mut IfDamageState, anim_base: *mut *mut *mut HSD_MatAnimJoint) {
    let dp = (*state).damage_percent;
    digit_anim(state, ONES, anim_base, dig(dp, 10, 1) as u8);
    digit_anim(state, TENS, anim_base, dig(dp, 100, 10) as u8);
    digit_anim(state, HUNDREDS, anim_base, dig(dp, 1000, 100) as u8);
}

pub unsafe fn ifStatus_GetStoredTranslationX(state: *mut IfDamageState, index: i32) -> f32 {
    (*state).translation_x[index as usize]
}

pub unsafe fn set_diffuse(state: *mut IfDamageState, place: usize, color: *mut _GXColor) {
    let mobj = (*(*(*state).jobjs[place]).u.dobj).mobj;
    (*(*mobj).mat).diffuse.r = (*color).r;
    (*(*mobj).mat).diffuse.g = (*color).g;
    (*(*mobj).mat).diffuse.b = (*color).b;
}

pub unsafe fn lerp_channel(factor: f32, k: usize) -> u8 {
    let a = ifStatus_804D57A8[k];
    let b = ifStatus_804D57AC[k];
    let v = fmadds(factor, (b as i32 - a as i32) as f32, a as f32);
    f2i8(v as f64) as u8
}

pub unsafe fn ifStatus_UpdateDamageDisplay(
    state: *mut IfDamageState,
    jobj: *mut HSD_JObj,
    color: *mut _GXColor,
    stamina_color: *mut _GXColor,
    normal_color: *mut _GXColor,
) {
    if dig((*state).damage_percent, 1000, 100) == 0 {
        ext::HSD_JObjSetFlagsAll((*state).jobjs[HUNDREDS], JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjClearFlagsAll((*state).jobjs[HUNDREDS], JOBJ_HIDDEN as u32);
    }

    if (*state).old_damage != (*state).damage_percent {
        if ext::Player_GetMoreFlagsBit2((*state).player_slot as i8 as i32) != 0 {
            let mut clamped_damage = (*state).damage_percent;
            if clamped_damage > 100 {
                clamped_damage = 100;
            } else if clamped_damage < 0 {
                clamped_damage = 0;
            }
            let factor = 1.0f32 - (clamped_damage as f32 / 100.0f32);
            (*stamina_color).r = lerp_channel(factor, 0);
            (*stamina_color).g = lerp_channel(factor, 1);
            (*stamina_color).b = lerp_channel(factor, 2);
            (*stamina_color).a = 255;
            *color = *stamina_color;
        } else {
            let mut clamped_damage = (*state).damage_percent;
            if clamped_damage > 300 {
                clamped_damage = 300;
            } else if clamped_damage < 0 {
                clamped_damage = 0;
            }
            let factor = clamped_damage as f32 / 300.0f32;
            (*normal_color).r = lerp_channel(factor, 0);
            (*normal_color).g = lerp_channel(factor, 1);
            (*normal_color).b = lerp_channel(factor, 2);
            (*normal_color).a = 255;
            *color = *normal_color;
        }

        set_diffuse(state, HUNDREDS, color);
        set_diffuse(state, TENS, color);
        set_diffuse(state, ONES, color);
        set_diffuse(state, PERCENT, color);
    }

    if ext::lb_8000B09C(jobj) != 0 {
        for i in 0..4usize {
            let digit_jobj = (*state).jobjs[i];
            assert_not_null(digit_jobj, 993);
            (*state).translation_x[i] = (*digit_jobj).translate.x;

            let digit_jobj = (*state).jobjs[i];
            assert_not_null(digit_jobj, 1006);
            (*state).translation_y[i] = (*digit_jobj).translate.y;
        }
    }
}

pub unsafe extern "C" fn ifStatus_802F4EDC(gobj: *mut HSD_GObj) {
    let hud = ifStatus_GetHUDInfo();
    let jobj: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut state: *mut IfDamageState = null_mut();
    let mut ptr = (*hud).players.as_mut_ptr();
    let mut i = 0i32;
    while i < 6 {
        if (*ptr).HUD_parent_entity == gobj {
            state = (*hud).players.as_mut_ptr().add(i as usize);
            break;
        }
        ptr = ptr.add(1);
        i += 1;
    }

    // Check for death animation flag
    if (*state).flags.explode_animation() != 0 {
        ifStatus_PercentOnDeathAnimationThink(state, i, ptr as usize as u32 as i32);
        return;
    }

    let is_stamina = ext::lb_8000B09C(jobj);
    let mut color = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut stamina_color = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut normal_color = _GXColor { r: 0, g: 0, b: 0, a: 0 };

    // Check if we need to start a new animation
    if (*state).flags.unk10() != 0 && ((*state).flags.animation_status_id() != 1 || is_stamina == 0) {
        (*state).flags.set_animation_status_id(1);

        ext::HSD_JObjRemoveAnim(jobj);

        let anim_base = &raw mut (*hud).damage_num_matanims;
        ext::lb_8000C07C(jobj, 1, (*hud).damage_num_anims, *anim_base, (*hud).damage_num_shapeanims);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);

        let digit_jobj = (*state).jobjs[PERCENT];
        if !digit_jobj.is_null() {
            let tobj = (*(*(*digit_jobj).u.dobj).mobj).tobj;
            ext::HSD_TObjAddAnimAll(tobj, tex3(anim_base));
            if ext::Player_GetMoreFlagsBit2((*state).player_slot as i8 as i32) != 0 {
                ext::HSD_TObjReqAnimAll(tobj, 1.0);
            } else {
                ext::HSD_TObjReqAnimAll(tobj, 0.0);
            }
            ext::HSD_AObjSetRate((*tobj).aobj, 0.0);
            ext::HSD_TObjAnim(tobj);
        }

        ext::mn_8022F3D8(jobj, 1, 0x400);
        ifStatus_InitDamageDigits(state, anim_base);
    }

    ext::HSD_JObjAnimAll(jobj);

    {
        let anim_joints = &raw mut (*hud).damage_num_matanims;
        let dp = (*state).damage_percent;

        digit_anim(state, ONES, anim_joints, dig(dp, 10, 1) as u8);
        digit_anim(state, TENS, anim_joints, dig(dp, 100, 10) as u8);

        if dig(dp, 1000, 100) == 0 && dig(dp, 100, 10) == 0 {
            ext::HSD_JObjSetFlagsAll((*state).jobjs[TENS], JOBJ_HIDDEN as u32);
        } else {
            ext::HSD_JObjClearFlagsAll((*state).jobjs[TENS], JOBJ_HIDDEN as u32);
        }

        digit_anim(state, HUNDREDS, anim_joints, dig(dp, 1000, 100) as u8);
    }

    ifStatus_UpdateDamageDisplay(state, jobj, &mut color, &mut stamina_color, &mut normal_color);

    {
        let dp = (*state).damage_percent;
        // Calculate digit spacing offsets based on which digit is "1"
        let ones_offset: f32 = if dp as i32 % 10 == 1 { 0.5069 } else { 0.0 };
        let tens_offset: f32 = if dig(dp, 100, 10) == 1 { 0.5069 } else { 0.0 };

        // Position percent sign
        let mut digit_jobj = (*state).jobjs[PERCENT];
        let mut pos = (*state).translation_x[PERCENT] - ones_offset;
        if digit_jobj.is_null() {
            panic!("__assert jobj.h:932 jobj");
        }
        (*digit_jobj).translate.x = pos;
        jobj_flagCheckSetMtxDirtySub(digit_jobj);

        // Position tens digit
        let mut digit_offset = ones_offset;
        digit_offset += tens_offset;
        digit_jobj = (*state).jobjs[TENS];
        pos = (*state).translation_x[TENS] + digit_offset;
        if digit_jobj.is_null() {
            panic!("__assert jobj.h:932 jobj");
        }
        (*digit_jobj).translate.x = pos;
        jobj_flagCheckSetMtxDirtySub(digit_jobj);

        // Position hundreds digit
        let hundreds_offset: f32 = if dig(dp, 1000, 100) == 1 { 0.5069 } else { 0.0 };
        digit_jobj = (*state).jobjs[HUNDREDS];
        digit_offset += tens_offset + hundreds_offset;
        pos = ifStatus_GetStoredTranslationX(state, HUNDREDS as i32) - -digit_offset;
        if digit_jobj.is_null() {
            panic!("__assert jobj.h:932 jobj");
        }
        (*digit_jobj).translate.x = pos;
        jobj_flagCheckSetMtxDirtySub(digit_jobj);
    }

    // Handle shake animation
    if (*state).flags.force_digit_shake() != 0 {
        (*state).frames_of_shake_remaining = 10;
    }

    ifStatus_802F4B84(state, is_stamina);

    // Apply final scale
    if ifStatus_804D6D60 >= 5 {
        (*jobj).scale.x = 0.65;
    }
}

pub unsafe fn find_player_by_entity(gobj: *mut HSD_GObj) -> *mut IfDamageState {
    for i in 0..6usize {
        if (*ifStatus_GetHUDInfo()).players[i].HUD_parent_entity == gobj {
            return &raw mut (*ifStatus_GetHUDInfo()).players[i];
        }
    }
    null_mut()
}

pub unsafe extern "C" fn ifStatus_802F5B48(gobj: *mut HSD_GObj) {
    let p = find_player_by_entity(gobj);
    let mut t = (*p).unk9;
    if t != 0 {
        t = t.wrapping_add(1);
        (*p).unk9 = t;
        if t > 0xC8 {
            ifStatus_802F6788((*p).player_slot);
            return;
        }
    }
    let slot = (*p).player_slot as i8 as i32;
    if ext::Player_GetEntity(slot).is_null() {
        return;
    }
    (*p).old_damage = (*p).damage_percent;
    if ext::Player_GetMoreFlagsBit2((*p).player_slot as i8 as i32) != 0 {
        (*p).damage_percent = ext::Player_GetRemainingHP((*p).player_slot as i8 as i32) as i16;
        if (*p).damage_percent > 0x3E7 {
            (*p).damage_percent = 0x3E7;
        } else if (*p).damage_percent < 0 {
            (*p).damage_percent = 0;
        }
        if (*p).old_damage != -1 && (*p).damage_percent < (*p).old_damage {
            (*p).flags.set_force_digit_shake(1);
            let mut diff = (*p).old_damage as i32 - (*p).damage_percent as i32;
            if diff < 0 {
                diff = -diff;
            }
            (*p).damage_from_last_attack = diff as u8;
        } else {
            (*p).flags.set_force_digit_shake(0);
        }
        if (*p).old_damage != -1 && (*p).damage_percent > (*p).old_damage {
            (*p).flags.set_unk10(1);
        } else {
            (*p).flags.set_unk10(0);
        }
        if (*p).old_damage == 0 {
            if (*ext::gmVs_GetSceneState()).fighters[(*p).player_slot as i8 as usize].anon_4.anon_0.x4_b5() != 0 {
                ifStatus_802F6948((*p).player_slot as i8 as i32);
            }
        }
    } else {
        (*p).damage_percent = ext::Player_GetDamage((*p).player_slot as i8 as i32) as i16;
        if (*p).damage_percent > 0x3E7 {
            (*p).damage_percent = 0x3E7;
        } else if (*p).damage_percent < 0 {
            (*p).damage_percent = 0;
        }
        if (*p).old_damage != -1 && (*p).damage_percent < (*p).old_damage {
            (*p).flags.set_unk10(1);
        } else {
            (*p).flags.set_unk10(0);
        }
        if (*p).old_damage != -1 && (*p).damage_percent > (*p).old_damage {
            (*p).flags.set_force_digit_shake(1);
            let mut diff = (*p).old_damage as i32 - (*p).damage_percent as i32;
            if diff < 0 {
                diff = -diff;
            }
            (*p).damage_from_last_attack = diff as u8;
            return;
        }
        (*p).flags.set_force_digit_shake(0);
    }
}

pub unsafe fn getPlayerByHUDParent(parent: *mut HSD_GObj) -> *mut IfDamageState {
    for i in 0..6usize {
        if (*ifStatus_GetHUDInfo()).players[i].HUD_parent_entity == parent {
            return &raw mut (*ifStatus_GetHUDInfo()).players[i];
        }
    }
    null_mut()
}

pub unsafe extern "C" fn ifStatus_802F5DE0(player: *mut HSD_GObj, arg1: i32) {
    // TARGET_PC: the script's HUD switch and the Atlas retail mask
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_DAMAGE) != 0 {
        return;
    }
    if (*getPlayerByHUDParent(player)).flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(player, arg1);
    }
}

pub unsafe fn getPlayerByNext(gobj: *mut HSD_GObj) -> *mut IfDamageState {
    for i in 0..6usize {
        if (*ifStatus_GetHUDInfo()).players[i].next == gobj {
            return &raw mut (*ifStatus_GetHUDInfo()).players[i];
        }
    }
    null_mut()
}

pub unsafe extern "C" fn ifStatus_802F5E50(gobj: *mut HSD_GObj, arg1: i32) {
    if ext::Script_StatusHUDVisible() == 0 || ext::Ui_RetailHidden(AT_RE_HUD_DAMAGE) != 0 {
        return;
    }
    let player = getPlayerByNext(gobj);
    if (*player).flags.hide_all_digits() == 0 {
        ext::HSD_GObj_JObjCallback(gobj, arg1);
    }
}

pub unsafe fn ifStatus_GetDamageJObj(jobj: *mut HSD_JObj, i: i32) -> *mut HSD_JObj {
    ifStatus_802F6194(jobj, i)
}

pub unsafe fn ifStatus_CreateMarkGObj(gobj: *mut *mut HSD_GObj) {
    *gobj = ext::GObj_Create(0xE, 0xF, 0);
}

pub unsafe fn ifStatus_GetPlayerCharacter(arg0: i32, chara: *mut u32) {
    let idx = arg0 as u8;
    *chara = ext::Player_GetPlayerCharacter(idx as i32);
}

pub unsafe fn ifStatus_LoadDamageJObj(hud: *mut HudIndex) -> *mut HSD_JObj {
    ext::HSD_JObjLoadJoint((*hud).damage_num_joint)
}

pub unsafe extern "C" fn ifStatus_802F5EC0(state: *mut IfDamageState, player_idx: i32) -> *mut HSD_GObj {
    let hud = ifStatus_GetHUDInfo();
    let jobj: *mut HSD_JObj;

    if (*state).HUD_parent_entity.is_null() {
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        jobj = ifStatus_LoadDamageJObj(hud);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ifStatus_802F5DE0), 0xB, 0);
        ext::HSD_GObj_SetupProc(gobj, Some(ifStatus_802F5B48), 0x11);
        ext::HSD_GObj_SetupProc(gobj, Some(ifStatus_802F4EDC), 0x11);
        (*state).HUD_parent_entity = gobj;
    } else {
        jobj = (*(*state).HUD_parent_entity).hsd_obj as *mut HSD_JObj;
    }
    (*state).flags.set_animation_status_id(0);
    ext::HSD_JObjRemoveAnim(jobj);
    let anim_base = &raw mut (*hud).damage_num_matanims;
    ext::lb_8000C07C(jobj, 0, (*hud).damage_num_anims, *anim_base, (*hud).damage_num_shapeanims);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    let vec = ext::ifAll_GetPlayerHUDPosition(player_idx as u8 as i32);
    HSD_JObjSetTranslate(jobj, vec as *mut Vec3);
    for i in 0..4usize {
        (*state).jobjs[i] = ifStatus_GetDamageJObj(jobj, i as i32);
        (*state).translation_x[i] = HSD_JObjGetTranslationX((*state).jobjs[i]);
        (*state).translation_y[i] = HSD_JObjGetTranslationY((*state).jobjs[i]);
    }
    if !(*state).jobjs[3].is_null() {
        let tobj = (*(*(*(*state).jobjs[3]).u.dobj).mobj).tobj;
        ext::HSD_TObjAddAnimAll(tobj, tex3(anim_base));
        if ext::Player_GetMoreFlagsBit2((*state).player_slot as i8 as i32) != 0 {
            ext::HSD_TObjReqAnimAll(tobj, 1.0);
        } else {
            ext::HSD_TObjReqAnimAll(tobj, 0.0);
        }
        ext::HSD_AObjSetRate((*tobj).aobj, 0.0);
        ext::HSD_TObjAnim(tobj);
    }
    ifStatus_802F5B48((*state).HUD_parent_entity);
    (*state).old_damage = ((*state).damage_percent == 0) as i16;
    ifStatus_802F4EDC((*state).HUD_parent_entity);
    (*state).HUD_parent_entity
}

pub unsafe extern "C" fn ifStatus_802F6194(node: *mut HSD_JObj, n: i32) -> *mut HSD_JObj {
    if node.is_null() || n < 0 {
        return null_mut();
    }
    let gx_head = (*node).child;
    let mut gx_cur = gx_head;
    let mut i = 0;
    while i < n && !gx_cur.is_null() {
        let gx_next = if gx_cur.is_null() { null_mut() } else { (*gx_cur).next };
        gx_cur = gx_next;
        i += 1;
    }
    gx_cur
}

pub unsafe extern "C" fn ifStatus_802F61FC(state: *mut IfDamageState, player_idx: i32) -> *mut HSD_GObj {
    let hud = ifStatus_GetHUDInfo();
    let idx = player_idx as u8;
    let mut chara: u32 = 0;
    ifStatus_GetPlayerCharacter(player_idx, &mut chara);
    let jobj: *mut HSD_JObj;
    let mut tobj: *mut HSD_TObj;

    if (*state).next.is_null() {
        ext::ifAll_GetArchive();
        let mut gobj: *mut HSD_GObj = null_mut();
        ifStatus_CreateMarkGObj(&mut gobj);
        if gobj.is_null() {
            panic!("__assert Error : gobj dont't get (ifAddMark)");
        }
        jobj = ext::HSD_JObjLoadJoint((*hud).damage_mark_joint);
        if jobj.is_null() {
            panic!("__assert Error : jobj dont't get (ifAddMark)");
        }
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, Some(ifStatus_802F5E50), 0xB, 0);
        (*state).next = gobj;
    } else {
        jobj = (*(*state).next).hsd_obj as *mut HSD_JObj;
    }
    tobj = (*(*(*(*jobj).child).u.dobj).mobj).tobj;
    ext::lb_8000C07C(jobj, 0, (*hud).damage_mark_anims, (*hud).damage_mark_matanims, (*hud).damage_mark_shapeanims);
    // TARGET_PC: ported from m-ex (Replace Match Emblem Matanim): with an m-ex IfAll the series
    // emblem comes from Eblm_matanim_joint at the frame insignia_idx[external id].
    let ifall = ext::lbDvd_8001819C(c"IfAll".as_ptr().cast());
    let eblm = if !ifall.is_null() {
        ext::HSD_ArchiveGetPublicAddress(ifall, c"Eblm_matanim_joint".as_ptr().cast()) as *mut HSD_MatAnimJoint
    } else {
        null_mut()
    };
    let emblem = if !eblm.is_null() { ext::Mex_InsigniaForExt(ext::Mex_PortCKindToExt(chara as i32)) } else { -1 };
    if emblem >= 0 {
        ext::HSD_JObjAddAnimAll((*jobj).child, null_mut(), eblm, null_mut());
        tobj = (*(*(*(*jobj).child).u.dobj).mobj).tobj;
        ext::HSD_TObjReqAnimAll(tobj, 0.5f32 + emblem as f32);
    } else {
        if chara == CKind_MasterH as u32 || chara.wrapping_sub(CKind_GKoops as u32) <= 1 {
            chara = CKind_Boy as u32;
        }
        ext::HSD_TObjReqAnimAll(tobj, 0.5f32 + ext::gm_80168B34(chara, 0, 0));
    }
    ext::HSD_AObjSetRate((*tobj).aobj, 0.1);
    ext::HSD_TObjAnim(tobj);
    let vec = ext::ifAll_GetPlayerHUDPosition(idx as i32);
    HSD_JObjSetTranslate(jobj, vec as *mut Vec3);
    HSD_JObjAddTranslationX(jobj, 0.25);
    let a = ext::Player_GetPlayerId(idx as i32);
    let b = ext::Player_GetTeam(idx as i32);
    let c = ext::gm_8016B168();
    let d = ext::Player_GetPlayerSlotType(idx as i32);
    let color = ext::gm_80160968(ext::gm_80160854(a as u8, b as u8, c as u8, d as u8));
    let mobj = (*(*HSD_JObjGetChild(jobj)).u.dobj).mobj;
    (*(*mobj).mat).diffuse.r = color.r;
    (*(*mobj).mat).diffuse.g = color.g;
    (*(*mobj).mat).diffuse.b = color.b;
    (*state).next
}

pub unsafe extern "C" fn ifStatus_802F6508(arg0: i32) {
    if ext::Player_GetPlayerSlotType(arg0) != Gm_PKind_NA && (ifStatus_804D6D60 as i32) > arg0 {
        let rules = ext::gm_GetStartMeleeRules();
        if (*rules).x2_6() != 0 {
            let hud_player = &raw mut (*ifStatus_GetHUDInfo()).players[arg0 as u8 as usize];
            (*hud_player).damage_percent = -1;
            (*hud_player).old_damage = -1;
            (*hud_player).frames_of_shake_remaining = 0;
            (*hud_player).flags.set_explode_animation(0);
            (*hud_player).flags.set_randomize_velocity(0);
            (*hud_player).flags.set_force_digit_shake(0);
            (*hud_player).flags.set_unk10(0);
            (*hud_player).player_slot = arg0 as u8;
            (*hud_player).unk9 = 0;
            ifStatus_802F61FC(hud_player, arg0 as u8 as i32);
            ifStatus_802F5EC0(hud_player, arg0 as u8 as i32);
            if (*rules).x3_0() != 0 && ext::gm_8016B238() == 0 {
                ext::un_802FF364(arg0);
            }
            let mode = (*rules).match_kind();
            if mode == 1 || (*rules).x4_2() != 0 {
                ext::ifStock_802F98E8(arg0 as u8, 0);
                return;
            }
            if mode == 2 {
                ext::ifStock_802F98E8(arg0 as u8, 2);
                return;
            }
            ext::ifStock_802F98E8(arg0 as u8, 1);
        }
    }
}

pub unsafe extern "C" fn ifStatus_802F665C(arg0: i32) {
    ext::ifAll_802F343C(arg0);
    ifStatus_804D6D60 = arg0 as u8;
    for i in 0..6 {
        ifStatus_802F6508(i);
    }
}

pub unsafe extern "C" fn ifStatus_802F66A4() {
    let hud = ifStatus_GetHUDInfo();
    let mut mrk: *mut *mut DynamicModelDesc = null_mut();
    let mut num: *mut *mut DynamicModelDesc = null_mut();
    let arch = ext::ifAll_GetArchive();
    ext::lbArchive_LoadSections(
        *arch,
        (&raw mut num).cast(),
        num_models_name.as_ptr(),
        &[((&raw mut mrk).cast(), mrk_models_name.as_ptr())],
    );
    (*hud).damage_num_joint = (**num).joint;
    (*hud).damage_num_anims = (**num).anims;
    (*hud).damage_num_matanims = (**num).matanims;
    (*hud).damage_num_shapeanims = (**num).shapeanims;
    (*hud).damage_mark_joint = (**mrk).joint;
    (*hud).damage_mark_anims = (**mrk).anims;
    (*hud).damage_mark_matanims = (**mrk).matanims;
    (*hud).damage_mark_shapeanims = (**mrk).shapeanims;
    ifStatus_804D6D60 = 0;
    ext::memzero(hud as *mut c_void, core::mem::size_of_val(&(*hud).players) as i32);
}

pub unsafe extern "C" fn ifStatus_802F6788(player_idx: u8) {
    let p_idx = player_idx as i8;
    let player_hud = &raw mut (*ifStatus_GetHUDInfo()).players[(p_idx as i32 & 0xFF) as usize];
    if !(*player_hud).HUD_parent_entity.is_null() {
        ext::HSD_GObjFree((*player_hud).HUD_parent_entity);
        (*player_hud).HUD_parent_entity = null_mut();
    }
    if !(*player_hud).next.is_null() {
        ext::HSD_GObjFree((*player_hud).next);
        (*player_hud).next = null_mut();
    }
    ext::ifStock_802FB650(player_idx as i32 & 0xFF);
}

pub unsafe extern "C" fn ifStatus_802F6804() {
    for i in 0..6i32 {
        let v = &raw mut (*ifStatus_GetHUDInfo()).players[(i & 0xFF) as usize];
        if !(*v).HUD_parent_entity.is_null() {
            ext::HSD_GObjFree((*v).HUD_parent_entity);
            (*v).HUD_parent_entity = null_mut();
        }
        if !(*v).next.is_null() {
            ext::HSD_GObjFree((*v).next);
            (*v).next = null_mut();
        }
        ext::ifStock_802FB650((i as i8 as i32) & 0xFF);
    }
}

/// Hide Percentage Digits
pub unsafe extern "C" fn ifStatus_802F6898() {
    let v = ifStatus_GetHUDInfo();
    for i in 0..6usize {
        (*v).players[i].flags.set_hide_all_digits(1);
    }
}

/// Show All Percentage Digits
pub unsafe extern "C" fn ifStatus_802F68F0() {
    let v = ifStatus_GetHUDInfo();
    for i in 0..6usize {
        (*v).players[i].flags.set_hide_all_digits(0);
    }
}

pub unsafe extern "C" fn ifStatus_802F6948(player_idx: i32) {
    let small_thing = ext::gm_GetStartMeleeRules();
    let hud_player = &raw mut (*ifStatus_GetHUDInfo()).players[player_idx as usize];
    let flags = &raw mut (*hud_player).flags;
    if (*flags).explode_animation() != 1 {
        (*flags).set_explode_animation(1);
        (*flags).set_randomize_velocity(1);
        if (*small_thing).x2_7() != 0 {
            (*hud_player).unk9 = 1;
        }
    }
}

pub unsafe fn ifStatus_TriggerStockLoss(player_idx: i32, callback: Option<unsafe extern "C" fn(i32)>) {
    let big_thing = ext::gmVs_GetSceneController();
    (*big_thing).state.unk_D = player_idx as u8;
    let small_thing = ext::gm_GetStartMeleeRules();
    let hud_player = &raw mut (*ifStatus_GetHUDInfo()).players[player_idx as usize];
    let flags = &raw mut (*hud_player).flags;
    if (*flags).explode_animation() != 1 {
        (*flags).set_explode_animation(1);
        (*flags).set_randomize_velocity(1);
        if (*small_thing).x2_7() != 0 {
            (*hud_player).unk9 = 1;
        }
    }
    if (*big_thing).start.is_stock() != 0
        && (ext::Player_GetPlayerSlotType(player_idx) == Gm_PKind_Human || ext::Player_GetPlayerSlotType(player_idx) == Gm_PKind_Cpu)
        && ext::Player_GetStocks(player_idx) == 0
    {
        ext::gm_8016B8D4(player_idx, ext::Player_GetPlayerSlotType(player_idx) as u8);
    }

    if (*big_thing).start.match_kind() != 1 && (*big_thing).start.x2_5() != 0 {
        if let Some(cb) = callback {
            cb(player_idx);
        }
    }
}

pub unsafe extern "C" fn ifStatus_802F69C0(player_idx: i32, arg1: i32) {
    let big_thing = ext::gmVs_GetSceneController();
    if (*big_thing).start.match_kind() != 1 && (*big_thing).start.x2_5() != 0 {
        ext::if_802F7C30(arg1);
    }
    ifStatus_TriggerStockLoss(player_idx, Some(ext::cb::if_802F7BB4));
}

pub unsafe extern "C" fn ifStatus_802F6AF8(player_idx: i32) {
    ifStatus_TriggerStockLoss(player_idx, Some(ext::cb::if_802F7AF8));
}

pub unsafe extern "C" fn ifStatus_802F6C04(player_idx: i32) {
    ifStatus_TriggerStockLoss(player_idx, Some(ext::cb::if_802F7C30));
}

pub unsafe extern "C" fn ifStatus_802F6D10(player_idx: i32) {
    ifStatus_TriggerStockLoss(player_idx, Some(ext::cb::if_802F7D08));
}

pub unsafe extern "C" fn ifStatus_802F6E1C(slot: i32) {
    ifStatus_802F6508(slot);
}

pub unsafe extern "C" fn ifStatus_802F6E3C(player_num: i32) {
    let player = &raw mut (*ifStatus_GetHUDInfo()).players[player_num as usize];
    if !(*player).HUD_parent_entity.is_null() {
        ext::HSD_GObj_80390CAC((*player).HUD_parent_entity);
    }
    if !(*player).next.is_null() {
        ext::HSD_GObj_80390CAC((*player).next);
    }
    ext::ifStock_802FB6AC(player_num);
}
