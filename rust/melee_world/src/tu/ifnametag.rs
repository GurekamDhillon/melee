//! Port of src/melee/if/ifnametag.c (built `TARGET_PC`): the in-match name tags. Every call out of
//! the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifnametag.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

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
    fn Camera_80030A50() -> *mut HSD_GObj;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_InitUserData(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn GetNameText(a0: i32) -> *mut i8;
    fn HSD_CObjEndCurrent() = || { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlags(a0 as _, a1 as _); } };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = |a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ };
    fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8) = |a0: i32, a1: *mut i8, a2: *mut i8| { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0 as _, a1 as _, a2 as _); } };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = |a0: i32, a1: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ };
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const u8) -> i32;
    fn HSD_SisLib_803A746C(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) = |a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32| { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A746C(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_SisLib_803A7548(a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32) = |a0: *mut HSD_Text, a1: i32, a2: f32, a3: f32| { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7548(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_SisLib_803A75E0(a0: *mut HSD_Text, a1: i32) = |a0: *mut HSD_Text, a1: i32| { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A75E0(a0 as _, a1 as _); } };
    fn Mex_Enabled(a0: *const i8) -> i32;
    fn Player_80036058(a0: i32) -> i32;
    fn Player_800360D8(a0: i32) -> f32;
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::pl::Player_GetEntity(a0 as _) as _ };
    fn Player_GetNametagSlotID(a0: i32) -> u8 = |a0: i32| unsafe { crate::pl::Player_GetNametagSlotID(a0 as _) as _ };
    fn Player_GetPlayerId(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetPlayerId(a0 as _) as _ };
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = |a0: i32| unsafe { crate::pl::Player_GetPlayerSlotType(a0 as _) as _ };
    fn Player_GetPlayerState(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetPlayerState(a0 as _) as _ };
    fn Player_GetStocks(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetStocks(a0 as _) as _ };
    fn Player_GetTeam(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetTeam(a0 as _) as _ };
    fn Player_LoadPlayerCoords(a0: i32, a1: *mut Vec) = |a0: i32, a1: *mut Vec| { unsafe { let _ = crate::pl::Player_LoadPlayerCoords(a0 as _, a1 as _); } };
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn Wb_RenderActive() -> i32;
    fn gmVs_GetSceneController() -> *mut VsSceneController = || unsafe { crate::gm::gmVs_GetSceneController() as _ };
    fn gm_8016B0E8() -> i32 = || unsafe { crate::gm::gm_8016B0E8() as _ };
    fn gm_8016B168() -> i32 = || unsafe { crate::gm::gm_8016B168() as _ };
    fn gm_8016B258(a0: i32) -> i32 = |a0: i32| unsafe { crate::gm::gm_8016B258(a0 as _) as _ };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_IsHUDHidden() -> i32 = || unsafe { crate::tu::ifall::ifAll_IsHUDHidden() as _ };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]);
    fn lbVector_WorldToScreen(a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32) -> *mut Vec = |a0: *mut HSD_CObj, a1: *const Vec, a2: *mut Vec, a3: i32| unsafe { melee_runtime::tu::lbvector::lbVector_WorldToScreen(a0 as _, a1 as _, a2 as _, a3 as _) as _ };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn memzero(a0: *mut c_void, a1: i32) = super::memzero_default;
    fn mn_8022EB04(a0: *mut c_void);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn mn_8022EB04(a0: *mut c_void) { super::mn_8022EB04(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::jobj_inlines!();

/// `AT_RE_HUD_NAMETAG` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_NAMETAG: i32 = 3;
/// `NAMETAG_DISABLED` (ifnametag.h).
const NAMETAG_DISABLED: u8 = 120;
const GM_PLAYER_NUM_MAX: usize = Gm_Player_NumMax as usize;

/// `HSD_GObjGetUserData` (baselib/gobj.h, static inline).
#[inline]
unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).user_data
}

/// 3F98B8: texture-frame table of the HUD plate models, `[player id][team colour]`.
pub static mut un_803F98B8: [[f32; 5]; 5] = [
    [0.0, 1.0, 3.0, 0.0, 16.0],
    [4.0, 5.0, 7.0, 5.0, 17.0],
    [8.0, 9.0, 11.0, 10.0, 18.0],
    [12.0, 13.0, 15.0, 15.0, 19.0],
    [16.0, 17.0, 19.0, 18.0, 18.0],
];
/// 3F991C: heart frames.
pub static mut un_803F991C: [f32; 4] = [24.0, 25.0, 26.0, 27.0];
/// 3F992C
pub static mut nametag_eyepos: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 300.0 }, robjdesc: null_mut() };
/// 3F9940
pub static mut nametag_interest: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
/// 3F9954
pub static mut nametag_CObjDesc: HSD_CameraDescFrustum = HSD_CameraDescFrustum {
    class_name: null_mut(),
    flags: 0,
    projection_type: 3,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut nametag_eyepos,
    interest: &raw mut nametag_interest,
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    top: 0.0,
    bottom: -480.0,
    left: 0.0,
    right: 640.0,
};

/// 4A1ED0 .bss
pub static mut un_804A1ED0: StaticModelDesc = unsafe { core::mem::zeroed() };
/// 4A1EE0 .bss
pub static mut un_804A1EE0: [*mut HSD_GObj; GM_PLAYER_NUM_MAX] = [null_mut(); GM_PLAYER_NUM_MAX];
/// 4A1EF8 .bss
pub static mut un_804A1EF8: [i32; GM_PLAYER_NUM_MAX] = [0; GM_PLAYER_NUM_MAX];
/// 4D6D68 .sbss
pub static mut un_804D6D68: *mut HSD_GObj = null_mut();
/// 4D6D6C .sbss
pub static mut un_804D6D6C: u8 = 0;
/// 4D6D70 .sbss
pub static mut un_804D6D70: [u8; GM_PLAYER_NUM_MAX] = [0; GM_PLAYER_NUM_MAX];
/// 4D6D78 .sbss
pub static mut un_804D6D78: *mut HSD_Text = null_mut();
/// 4D6D7C .sbss
pub static mut un_804D6D7C: i32 = 0;

/// `memzero` (lb_00B0.c): the real behaviour, used unless a test installs a hook.
pub fn memzero_default(mem: *mut c_void, size: i32) {
    unsafe { core::ptr::write_bytes(mem as *mut u8, 0, size as usize) }
}

/// 2FC9B4: texture frame of a plate model.
pub unsafe extern "C" fn un_802FC9B4(slot: u8, arg1: u8, arg2: u8, arg3: u8) -> f32 {
    let player_id: u8 = ext::Player_GetPlayerId(slot as i32) as u8;
    if arg2 != 0 {
        if arg3 == 0 {
            return un_803F98B8[player_id as usize][arg1 as usize];
        }
        if (ext::gm_8016B258(player_id as i32) != 0 || ext::gm_8016B0E8() != 0) && arg1 as i32 == ext::Player_GetTeam(0) {
            return un_803F991C[ext::Player_GetPlayerId(0) as usize];
        }
        return un_803F98B8[4][arg1 as usize];
    }
    if arg3 == 0 {
        return un_803F98B8[player_id as usize][3];
    }
    18.0 // CP Gray
}

pub unsafe extern "C" fn NameTag_RenderCallback(gobj: *mut HSD_GObj, pass: i32) {
    if ext::Ui_RetailHidden(AT_RE_HUD_NAMETAG) != 0 {
        return;
    }
    ext::HSD_GObj_JObjCallback(gobj, pass);
}

pub unsafe extern "C" fn fn_802FCAC4(gobj: *mut HSD_GObj, _pass: i32) {
    // Do not move the live nametag text offscreen just to hide a capture.
    if ext::Wb_RenderActive() != 0 && ext::ifAll_IsHUDHidden() != 0 {
        return;
    }
    if ext::ifAll_IsHUDHidden() != 0 || un_804D6D6C != 0 || ext::Ui_RetailHidden(AT_RE_HUD_NAMETAG) != 0 {
        for i in 0..GM_PLAYER_NUM_MAX {
            let do_it = !(ext::Player_GetPlayerSlotType(i as i32) != Gm_PKind_Human || ext::Player_GetNametagSlotID(i as i32) == 0x78);
            if do_it {
                ext::HSD_SisLib_803A746C(un_804D6D78, un_804A1EF8[i], -5000.0, 0.0);
            }
        }
    } else if ext::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) != 0 {
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

pub unsafe extern "C" fn un_802FCBA0() {
    let mut x: *mut *mut DynamicModelDesc = null_mut();
    let archive = ext::ifAll_GetArchive();
    ext::lbArchive_LoadSections(*archive, (&raw mut x).cast(), c"ScInfPnm_scene_models".as_ptr().cast(), &[]);
    un_804A1ED0.joint = (**x).joint;
    if !(**x).anims.is_null() {
        un_804A1ED0.animjoint = *(**x).anims;
    }
    if !(**x).matanims.is_null() {
        un_804A1ED0.matanim_joint = *(**x).matanims;
    }
    if !(**x).shapeanims.is_null() {
        un_804A1ED0.shapeanim_joint = *(**x).shapeanims;
    }
}

#[inline]
pub unsafe extern "C" fn has_nametag(slot: i32) -> bool {
    !(ext::Player_GetPlayerSlotType(slot) != Gm_PKind_Human || ext::Player_GetNametagSlotID(slot) == NAMETAG_DISABLED)
}

pub unsafe extern "C" fn fn_802FCC44(gobj: *mut HSD_GObj) {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let slot = HSD_GObjGetUserData(gobj) as *mut u8;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    // Ported from m-ex: "Hide Nametag When Invisible" (opt-in: MELEE_MEX=hide_nametag_invisible).
    if ext::Mex_Enabled(c"hide_nametag_invisible".as_ptr().cast()) != 0
        && (*ext::gmVs_GetSceneController()).start.is_teams != 1
    {
        let fp = HSD_GObjGetUserData(ext::Player_GetEntity(*slot as i32)) as *mut Fighter;
        if (((*fp).kind == Ft_Kind_Mewtwo && (*fp).motion_id == 0xEC) || (*fp).x221E_b7() != 0)
            && ext::gm_8016B258(*slot as i32) == 0
        {
            ext::HSD_JObjSetFlags(HSD_JObjGetChild(jobj), JOBJ_HIDDEN as u32);
            if has_nametag(*slot as i32) {
                ext::HSD_SisLib_803A746C(un_804D6D78, un_804A1EF8[*slot as usize], -5000.0, 0.0);
            }
            return;
        }
    }
    if ext::Player_GetPlayerSlotType(*slot as i32) != Gm_PKind_NA
        && ext::Player_GetPlayerState(*slot as i32) != 0
        && ext::Player_GetStocks(*slot as i32) != 0
        && (un_804D6D70[*slot as usize] != 0
            || ext::Player_GetNametagSlotID(*slot as i32) != b'x'
            || ext::Player_80036058(*slot as i32) != 0
            || ext::gm_8016B258(*slot as i32) != 0)
    {
        ext::HSD_JObjClearFlags(HSD_JObjGetChild(jobj), JOBJ_HIDDEN as u32);
    } else {
        ext::HSD_JObjSetFlags(HSD_JObjGetChild(jobj), JOBJ_HIDDEN as u32);
        if has_nametag(*slot as i32) {
            ext::HSD_SisLib_803A746C(un_804D6D78, un_804A1EF8[*slot as usize], -5000.0, 0.0);
        }
        return;
    }
    ext::Player_LoadPlayerCoords(*slot as i32, &mut vec);
    vec.y += ext::Player_800360D8(*slot as i32);
    vec.y -= 2.5;
    ext::lbVector_WorldToScreen((*ext::Camera_80030A50()).hsd_obj.cast(), &vec, &mut vec2, 0);
    HSD_JObjSetTranslateX(jobj, vec2.x);
    HSD_JObjSetTranslateY(jobj, -vec2.y);
    if has_nametag(*slot as i32) {
        ext::HSD_SisLib_803A746C(un_804D6D78, un_804A1EF8[*slot as usize], vec2.x, vec2.y - 56.0);
    }
}

#[inline]
pub unsafe extern "C" fn inlineA1(var_f31: f32) -> f32 {
    let q = var_f31 as i32;
    if q >= 16 {
        28.0
    } else {
        ((q % 4) + 20) as f32
    }
}

pub unsafe extern "C" fn NameTag_Create(slot: i32) {
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    un_804A1EE0[slot as usize] = gobj;
    {
        let jobj = ext::HSD_JObjLoadJoint(un_804A1ED0.joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
        ext::GObj_SetupGXLink(gobj, Some(NameTag_RenderCallback), 9, 0);
        HSD_JObjSetScaleX(jobj, 10.0);
        HSD_JObjSetScaleY(jobj, 10.0);
        HSD_JObjSetScaleZ(jobj, 10.0);
        ext::HSD_JObjAddAnimAll(jobj, un_804A1ED0.animjoint, un_804A1ED0.matanim_joint, un_804A1ED0.shapeanim_joint);
        {
            let team = ext::Player_GetTeam(slot) as u8;
            let a2 = ext::gm_8016B168() as u8;
            let a3 = ext::Player_GetPlayerSlotType(slot) as u8;
            let mut f = un_802FC9B4(slot as u8, team, a2, a3);
            if has_nametag(slot) {
                f = inlineA1(f);
                let name = ext::GetNameText(ext::Player_GetNametagSlotID(slot) as i32);
                un_804A1EF8[slot as usize] = ext::HSD_SisLib_803A6B98(un_804D6D78, -5000.0, 0.0, name.cast());
                ext::HSD_SisLib_803A7548(un_804D6D78, un_804A1EF8[slot as usize], 0.4, 0.55);
            }
            ext::HSD_JObjReqAnimAll(jobj, f);
        }
        ext::HSD_JObjAnimAll(jobj);
    }
    {
        let user_data = ext::HSD_MemAlloc(1) as *mut u8;
        ext::GObj_InitUserData(gobj, 0, Some(ext::cb::mn_8022EB04), user_data.cast());
        *user_data = slot as u8;
    }
    ext::HSD_GObj_SetupProc(gobj, Some(fn_802FCC44), 17);
}

pub unsafe extern "C" fn un_802FD28C(slot: i32) {
    let jobj = (*un_804A1EE0[slot as usize]).hsd_obj as *mut HSD_JObj;
    let team = ext::Player_GetTeam(slot) as u8;
    let a2 = ext::gm_8016B168() as u8;
    let a3 = ext::Player_GetPlayerSlotType(slot) as u8;
    let mut f = un_802FC9B4(slot as u8, team, a2, a3);
    if has_nametag(slot) {
        f = inlineA1(f);
        ext::HSD_SisLib_803A75E0(un_804D6D78, un_804A1EF8[slot as usize]);
        let name = ext::GetNameText(ext::Player_GetNametagSlotID(slot) as i32);
        un_804A1EF8[slot as usize] = ext::HSD_SisLib_803A6B98(un_804D6D78, -5000.0, 0.0, name.cast());
        ext::HSD_SisLib_803A7548(un_804D6D78, un_804A1EF8[slot as usize], 0.4, 0.55);
    }
    ext::HSD_JObjReqAnimAll(jobj, f);
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn un_802FD404() {
    un_804D6D70[0] = 1;
    un_804D6D70[1] = 1;
    un_804D6D70[2] = 1;
    un_804D6D70[3] = 1;
    un_804D6D70[4] = 1;
    un_804D6D70[5] = 1;
}

pub unsafe extern "C" fn un_802FD428() {
    ext::memzero((&raw mut un_804D6D70).cast(), core::mem::size_of::<[u8; GM_PLAYER_NUM_MAX]>() as i32);
}

pub unsafe extern "C" fn un_802FD450() {
    un_804D6D6C = 1;
}

pub unsafe extern "C" fn un_802FD45C() {
    un_804D6D6C = 0;
}

pub unsafe extern "C" fn un_802FD468() {
    for i in 0..GM_PLAYER_NUM_MAX {
        if !un_804A1EE0[i].is_null() {
            ext::HSD_GObjFree(un_804A1EE0[i]);
        }
    }
    ext::HSD_GObjFree(un_804D6D68);
}

#[inline]
pub unsafe extern "C" fn un_802FD4C8_inline(arg0: i32) -> *mut HSD_GObj {
    ext::GObj_Create(0xE, arg0 as u8, 0)
}

pub unsafe extern "C" fn un_802FD4C8() {
    for i in 0..GM_PLAYER_NUM_MAX {
        un_804A1EE0[i] = null_mut();
    }
    un_804D6D68 = null_mut();
    un_804D6D6C = 0;
    ext::memzero((&raw mut un_804D6D70).cast(), core::mem::size_of::<[u8; GM_PLAYER_NUM_MAX]>() as i32);
    let gobj = un_802FD4C8_inline(15);
    un_804D6D68 = gobj;
    let new_var = ext::lb_80013B14((&raw mut nametag_CObjDesc).cast());
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, new_var.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_802FCAC4), 6);
    (*gobj).gxlink_prios = 0x200;
    un_804D6D7C = ext::HSD_SisLib_803A611C(2, gobj, 14, 15, 0, 9, 6, 0);
    // "Break the targets!" "Race to the finish!" etc strings
    ext::HSD_SisLib_803A62A0(2, c"SdIntro.dat".as_ptr() as *mut i8, c"SIS_IntroData".as_ptr() as *mut i8);
    un_804D6D78 = ext::HSD_SisLib_803A6754(2, un_804D6D7C);
    (*un_804D6D78).pos_z = -10.0;
    (*un_804D6D78).default_alignment = 1;
    un_802FCBA0();
    for i in 0..GM_PLAYER_NUM_MAX {
        NameTag_Create(i as i32);
    }
}
