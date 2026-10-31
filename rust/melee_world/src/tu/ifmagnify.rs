//! Port of src/melee/if/ifmagnify.c (built `TARGET_PC`): the off-screen fighter magnifier bubbles
//! and edge arrows. Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifmagnify.rs.
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
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
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
    fn Camera_8002A4AC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::cmcamera::Camera_8002A4AC(a0 as _); } };
    fn Camera_80030130() -> i32 = || crate::cm::camera::world().camera_80030130() as i32;
    fn Camera_80030A50() -> *mut HSD_GObj = || unsafe { crate::cm::camera::Camera_80030A50() as _ };
    fn Camera_80030BBC(a0: *mut Vec, a1: *mut S32Vec2) -> i32;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::GObj_InitUserData(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32;
    fn Ground_801C0604() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0604() as _ };
    fn Ground_801C0618() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0618() as _ };
    fn Ground_801C062C() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C062C() as _ };
    fn Ground_801C0640() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0640() as _ };
    fn Ground_801C0654() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0654() as _ };
    fn Ground_801C0668() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0668() as _ };
    fn Ground_801C067C() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C067C() as _ };
    fn Ground_801C0690() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C0690() as _ };
    fn Ground_801C06A4() -> *mut _GXColor = || unsafe { crate::tu::ground::Ground_801C06A4() as _ };
    fn HSD_CObjEndCurrent() = || { unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) = |a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32| { unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_CObjGetOrtho(a0: *mut HSD_CObj, a1: *mut f32, a2: *mut f32, a3: *mut f32, a4: *mut f32);
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { melee_runtime::tu::cobj_world::HSD_CObjSetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) = |a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32| { unsafe { melee_runtime::tu::cobj::HSD_CObjSetOrtho(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn HSD_CObjSetScissorx4(a0: *mut HSD_CObj, a1: u16, a2: u16, a3: u16, a4: u16);
    fn HSD_CObjSetViewport(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) = |a0: *mut HSD_CObj, a1: *mut _HSD_RectS16| { unsafe { melee_runtime::tu::cobj::HSD_CObjSetViewport(a0 as _, a1 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { melee_runtime::tu::gobj_native::HSD_GObj_80390ED0(a0 as _, a1 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn Player_80036978(a0: i32, a1: *mut Vec);
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::pl::Player_GetEntity(a0 as _) as _ };
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = |a0: i32| unsafe { crate::pl::Player_GetPlayerSlotType(a0 as _) as _ };
    fn Player_GetTeam(a0: i32) -> i32 = |a0: i32| unsafe { crate::pl::Player_GetTeam(a0 as _) as _ };
    fn Stage_GetCamBoundsBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ };
    fn Stage_GetCamBoundsLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn View_AspectBits() -> i32;
    fn Wb_RenderActive() -> i32;
    fn atan2f(a0: f32, a1: f32) -> f32 = |a0: f32, a1: f32| unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ };
    fn ftDrawCommon_80080C28(a0: *mut HSD_GObj, a1: i32);
    fn ftLib_80086B64(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B64(a0 as _) as _ };
    fn ftLib_80086B80(a0: *mut HSD_GObj) -> f32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B80(a0 as _) as _ };
    fn ftLib_80086B90(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086B90(a0 as _, a1 as _); } };
    fn ftLib_80086ED0(a0: *mut HSD_GObj) -> i32;
    fn ftLib_UpdateLogicOffCamera(a0: *mut HSD_GObj);
    fn gmVs_GetSceneController() -> *mut VsSceneController = || unsafe { crate::gm::gmVs_GetSceneController() as _ };
    fn gm_80160854(a0: u8, a1: u8, a2: u8, a3: u8) -> u32;
    fn gm_80160968(a0: u32) -> _GXColor;
    fn gm_8016B168() -> i32 = || unsafe { crate::gm::gm_8016B168() as _ };
    fn ifAll_GetArchive() -> *mut *mut HSD_Archive = || unsafe { crate::tu::ifall::ifAll_GetArchive() as _ };
    fn ifAll_IsHUDHidden() -> i32 = || unsafe { crate::tu::ifall::ifAll_IsHUDHidden() as _ };
    fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]) = |a0: *mut HSD_Archive, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]| { unsafe { let mut v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void]; for (s, n) in rest.iter() { v.push(*s as *mut core::ffi::c_void); v.push(*n as *mut core::ffi::c_void); } v.push(core::ptr::null_mut()); melee_runtime::tu::lbarchive::lbArchive_LoadSections(a0 as _, &v); } };
    fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, ids: &[i32]) -> i32;
    fn lb_800122C8(a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: i32) = |a0: *mut HSD_ImageDesc, a1: u16, a2: u16, a3: i32| unsafe { melee_runtime::tu::lbspdisplay::lb_800122C8(a0 as _, a1 as _, a2 as _, a3 != 0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn memzero(a0: *mut c_void, a1: i32) = super::memzero_default;
    fn un_802FD928(a0: u8, a1: u8, a2: *mut _GXColor) = |a0: u8, a1: u8, a2: *mut _GXColor| { unsafe { crate::tu::if_2FD9::un_802FD928(a0 as _, a1 as _, a2 as _); } };
    fn un_802FD9D8(a0: u8) = |a0: u8| { unsafe { crate::tu::if_2FD9::un_802FD9D8(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Ground_801C0604() -> *mut _GXColor { super::Ground_801C0604() }
        pub unsafe extern "C" fn Ground_801C0618() -> *mut _GXColor { super::Ground_801C0618() }
        pub unsafe extern "C" fn Ground_801C062C() -> *mut _GXColor { super::Ground_801C062C() }
        pub unsafe extern "C" fn Ground_801C0640() -> *mut _GXColor { super::Ground_801C0640() }
        pub unsafe extern "C" fn Ground_801C0654() -> *mut _GXColor { super::Ground_801C0654() }
        pub unsafe extern "C" fn Ground_801C0668() -> *mut _GXColor { super::Ground_801C0668() }
        pub unsafe extern "C" fn Ground_801C067C() -> *mut _GXColor { super::Ground_801C067C() }
        pub unsafe extern "C" fn Ground_801C0690() -> *mut _GXColor { super::Ground_801C0690() }
        pub unsafe extern "C" fn Ground_801C06A4() -> *mut _GXColor { super::Ground_801C06A4() }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_804D7814;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
}
// END GENERATED EXT

crate::jobj_inlines!();

use melee_core::fp::fmadds;
use melee_runtime::cabi::f2u8;

/// `AT_RE_HUD_MAGNIFY` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_MAGNIFY: i32 = 4;

/// 3F97C0
pub static mut ifMagnify_803F97C0: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 300.0 }, robjdesc: null_mut() };
/// 3F97D4
pub static mut ifMagnify_803F97D4: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
/// 3F97E8
pub static mut ifMagnify_803F97E8: HSD_CameraDescFrustum = HSD_CameraDescFrustum {
    class_name: null_mut(),
    flags: 0,
    projection_type: PROJ_ORTHO as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut ifMagnify_803F97C0,
    interest: &raw mut ifMagnify_803F97D4,
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    top: 0.0,
    bottom: 0.0,
    left: 0.0,
    right: 0.0,
};
/// 4D57E8
pub static mut ifMagnify_804D57E8: [i8; 5] = [b'l' as i8, b'u' as i8, b'p' as i8, b'e' as i8, 0];
/// 4DDB60
pub static ifMagnify_804DDB60: i32 = 0;

/// 4A1DE0
pub static mut ifMagnify_804A1DE0: ifMagnify = unsafe { core::mem::zeroed() };

/// 3F9828: the corner colour getters.
pub static mut ifMagnify_803F9828: [Option<unsafe extern "C" fn() -> *mut _GXColor>; 9] = [
    Some(ext::cb::Ground_801C0604),
    Some(ext::cb::Ground_801C0690),
    Some(ext::cb::Ground_801C0618),
    Some(ext::cb::Ground_801C0654),
    Some(ext::cb::Ground_801C06A4),
    Some(ext::cb::Ground_801C0668),
    Some(ext::cb::Ground_801C062C),
    Some(ext::cb::Ground_801C067C),
    Some(ext::cb::Ground_801C0640),
];

/// 3F984C
pub static mut ifMagnify_803F984C: [[u8; 4]; 16] = [
    [0, 0, 0, 0], [0, 1, 0, 1], [1, 2, 1, 2], [2, 2, 2, 2],
    [0, 0, 3, 3], [0, 1, 3, 4], [1, 2, 4, 5], [2, 2, 5, 5],
    [3, 3, 6, 6], [3, 4, 6, 7], [4, 5, 7, 8], [5, 5, 8, 8],
    [6, 6, 6, 6], [6, 7, 6, 7], [7, 8, 7, 8], [8, 8, 8, 8],
];

/// `ifMagnify_LogicArmed` (TARGET_PC; a C `bool`, i.e. an int).
pub static mut ifMagnify_LogicArmed: i32 = 0;
/// `ifMagnify_LogicOffscreen` (TARGET_PC): the off-screen flag computed by logic.
pub static mut ifMagnify_LogicOffscreen: [u8; 6] = [0; 6];

/// `memzero` (lb_00B0.c): the real behaviour, used unless a test installs a hook.
pub fn memzero_default(mem: *mut c_void, size: i32) {
    melee_core::ct!("memzero_default");
    unsafe { core::ptr::write_bytes(mem as *mut u8, 0, size as usize) }
}

/// The view aspect ratio the window reports, as an int-punned float.
pub unsafe extern "C" fn ifMagnify_ViewAspect() -> f32 {
    melee_core::ct!("ifMagnify_ViewAspect");
    f32::from_bits(ext::View_AspectBits() as u32)
}

pub unsafe extern "C" fn ifMagnify_VisualOffscreen(fighter: *mut HSD_GObj) -> bool {
    melee_core::ct!("ifMagnify_VisualOffscreen");
    let mut world = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::ftLib_80086B90(fighter, &mut world);
    ext::Camera_80030BBC(&mut world, null_mut()) == 0
}

#[inline]
pub unsafe extern "C" fn ifMagnify_IsHUDVisible() -> bool {
    melee_core::ct!("ifMagnify_IsHUDVisible");
    if (*ext::gmVs_GetSceneController()).state.hud_enabled == 0 || ext::ifAll_IsHUDHidden() != 0 || ext::Camera_80030130() != 0 {
        return false;
    }
    true
}

#[inline]
pub unsafe extern "C" fn ifMagnify_GetPlayerColor(color: *mut _GXColor, slot: i32) {
    melee_core::ct!("ifMagnify_GetPlayerColor");
    let a0 = slot as u8;
    let team = ext::Player_GetTeam(slot);
    let g = ext::gm_8016B168();
    let kind = ext::Player_GetPlayerSlotType(slot);
    *color = ext::gm_80160968(ext::gm_80160854(a0, team as u8, g as u8, kind as u8));
}

pub unsafe extern "C" fn ifMagnify_802FB6E8(slot: i32) -> i32 {
    melee_core::ct!("ifMagnify_802FB6E8");
    if ifMagnify_802FC998(slot) as i32 != 0 {
        return ifMagnify_804A1DE0.player[slot as usize].state.edge() as i32;
    }
    0
}

pub unsafe extern "C" fn ifMagnify_802FB73C(player: *mut ifMagnifyPlayer, pos: *mut Vec2, out: *mut Vec2) -> *mut ifMagnifyPlayer {
    melee_core::ct!("ifMagnify_802FB73C");
    let mut horizontal_bound: f32 = 252.70001;
    // TARGET_PC: the horizontal bound follows the window's aspect ratio
    horizontal_bound *= ifMagnify_ViewAspect() / (4.0f32 / 3.0f32);
    let edge_ratio: f32 = 162.7f32 / horizontal_bound;
    let mut x = (*pos).x;
    let y = (*pos).y;
    if 0.0f32 == x {
        if y > 0.0 {
            (*out).y = 162.7;
        } else {
            (*out).y = -162.7;
        }
        (*out).x = 0.0;
    } else {
        let ratio = y / x;
        if ratio > edge_ratio || ratio < -edge_ratio {
            if y > 0.0 {
                (*out).y = 162.7;
            } else {
                (*out).y = -162.7;
            }
            let mut x_clamped = (*out).y;
            x_clamped = x_clamped * x;
            x_clamped /= y;
            if x_clamped < -horizontal_bound {
                (*out).x = -horizontal_bound;
            } else if x_clamped > horizontal_bound {
                (*out).x = horizontal_bound;
            } else {
                (*out).x = x_clamped;
            }
        } else {
            if x > 0.0 {
                (*out).x = horizontal_bound;
            } else {
                (*out).x = -horizontal_bound;
            }
            let mut y_clamped = (*out).x;
            y_clamped = y_clamped * y;
            y_clamped /= x;
            if y_clamped < -162.7 {
                (*out).y = -162.7;
            } else if y_clamped > 162.7 {
                (*out).y = 162.7;
            } else {
                (*out).y = y_clamped;
            }
        }
    }

    let out_x = (*out).x;
    x = -horizontal_bound;
    if out_x == x {
        (*player).state.set_edge(2);
        return player;
    }
    x = horizontal_bound;
    if out_x == x {
        (*player).state.set_edge(4);
        return player;
    }
    x = 162.7;
    if (*out).y == x {
        (*player).state.set_edge(1);
        return player;
    }
    (*player).state.set_edge(3);
    player
}

pub unsafe extern "C" fn ifMagnify_802FB8C0(gobj: *mut HSD_GObj, code: i32) {
    melee_core::ct!("ifMagnify_802FB8C0");
    let mut screen_pos = S32Vec2 { x: 0, y: 0 };
    let mut edge_pos = Vec2 { x: 0.0, y: 0.0 };
    let mut dir = Vec2 { x: 0.0, y: 0.0 };
    let mut translate = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut color = _GXColor { r: 0, g: 0, b: 0, a: 0 };

    if code != 0 {
        return;
    }

    let player = (*gobj).user_data as *mut ifMagnifyPlayer;
    let slot = player.offset_from(&raw mut ifMagnify_804A1DE0.player[0]) as i32;
    let mut is_colored = false;
    let mut should_display = ifMagnify_IsHUDVisible();
    // the Atlas retail mask takes retail's own hidden-HUD path: no bubble plate, the arrow is released
    if ext::Ui_RetailHidden(AT_RE_HUD_MAGNIFY) != 0 {
        should_display = false;
    }
    if should_display && (*player).state.is_offscreen() != 0 {
        let fighter_gobj = ext::Player_GetEntity(slot);
        if !fighter_gobj.is_null() {
            {
                let mut world = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::ftLib_80086B90(fighter_gobj, &mut world);
                ext::Camera_80030BBC(&mut world, &mut screen_pos);
            }
            dir.x = screen_pos.x as f32 - 320.0;
            dir.y = -(screen_pos.y as f32 - 240.0);
            dir.x *= ifMagnify_ViewAspect() / (4.0f32 / 3.0f32);

            HSD_JObjSetRotationZ((*player).jobj, ext::atan2f(dir.y, dir.x));

            ifMagnify_802FB73C(player, &mut dir, &mut edge_pos);
            translate.x = 0.09125f32 * edge_pos.x;
            translate.y = 0.1f32 * edge_pos.y;
            translate.z = 0.0;
            HSD_JObjSetTranslate((*(*player).gobj).hsd_obj.cast(), &mut translate);

            ext::HSD_GObj_JObjCallback(gobj, code);
            if (*player).state.edge() == 4 || (*player).state.edge() == 2 {
                ifMagnify_GetPlayerColor(&mut color, slot);
                let mut color_copy = color;
                let cp: *mut _GXColor = &mut color_copy;
                let arrow_kind = if (*player).state.edge() == 2 { 1 } else { 2 };
                ext::un_802FD928(slot as u8, arrow_kind, cp);
                is_colored = true;
            }
        }
    }
    if !is_colored {
        ext::un_802FD9D8(slot as u8);
    }
}

#[inline]
pub unsafe extern "C" fn ifMagnify_GetCornerColors(colors: *mut _GXColor, world_pos: *mut Vec3) {
    melee_core::ct!("ifMagnify_GetCornerColors");
    for j in 0..4usize {
        let y_class: f32;
        let x_class: f32;
        if (*world_pos).y > ext::Stage_GetCamBoundsTopOffset() {
            y_class = 0.0;
        } else if (*world_pos).y < ext::Stage_GetCamBoundsBottomOffset() {
            y_class = 3.0;
        } else if (*world_pos).y > 0.5f32 * (ext::Stage_GetCamBoundsTopOffset() + ext::Stage_GetCamBoundsBottomOffset()) {
            y_class = 1.0;
        } else {
            y_class = 2.0;
        }
        if (*world_pos).x < ext::Stage_GetCamBoundsLeftOffset() {
            x_class = 0.0;
        } else if (*world_pos).x > ext::Stage_GetCamBoundsRightOffset() {
            x_class = 3.0;
        } else if (*world_pos).x < 0.5f32 * (ext::Stage_GetCamBoundsLeftOffset() + ext::Stage_GetCamBoundsRightOffset()) {
            x_class = 1.0;
        } else {
            x_class = 2.0;
        }
        let color_ids = &ifMagnify_803F984C[((y_class as i32 * 4) + x_class as i32) as usize];
        *colors.add(j) = *(ifMagnify_803F9828[color_ids[j] as usize].unwrap())();
    }
}

pub unsafe extern "C" fn ifMagnify_802FBBDC(gobj: *mut HSD_GObj) {
    melee_core::ct!("ifMagnify_802FBBDC");
    let mut top = 0.0f32;
    let mut bottom = 0.0f32;
    let mut left = 0.0f32;
    let mut right = 0.0f32;
    let mut interest_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut colors = [_GXColor { r: 0, g: 0, b: 0, a: 0 }; 4];
    let mut world_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    // TARGET_PC
    if ext::Wb_RenderActive() != 0 && ext::ifAll_IsHUDHidden() != 0 {
        return;
    }
    let magnify = &raw mut ifMagnify_804A1DE0;
    for i in 0..6usize {
        (*magnify).player[i].state.set_is_offscreen(0);
    }

    let mut should_display = ifMagnify_IsHUDVisible();
    // masked: the same path as retail's hidden HUD
    if ext::Ui_RetailHidden(AT_RE_HUD_MAGNIFY) != 0 {
        should_display = false;
    }
    if should_display {
        let cobj: *mut HSD_CObj = (*gobj).hsd_obj.cast();
        ext::HSD_CObjGetOrtho(cobj, &mut top, &mut bottom, &mut left, &mut right);
        // This camera captures a fixed-size bubble texture, not the main window.
        if ext::HSD_CObjSetCurrent(cobj) != 0 {
            ext::HSD_GObj_80390ED0(gobj, 7);
            ext::HSD_CObjEndCurrent();
        }

        for i in 0..6usize {
            let player = &raw mut (*magnify).player[i];
            let fighter_gobj = ext::Player_GetEntity(i as i32);
            if (*player).state.ignore_offscreen() != 0
                || fighter_gobj.is_null()
                || !ifMagnify_VisualOffscreen(fighter_gobj)
                || ext::ftLib_80086ED0(fighter_gobj) == 0
            {
                continue;
            }

            let scale = ext::ftLib_80086B80(fighter_gobj) / 8.0f32;
            ext::HSD_CObjSetOrtho(cobj, top * scale, bottom * scale, left * scale, right * scale);
            ext::ftLib_80086B90(fighter_gobj, &mut interest_pos);
            ext::HSD_CObjSetInterest(cobj, &mut interest_pos);
            interest_pos.z = 300.0;
            ext::HSD_CObjSetEyePosition(cobj, &mut interest_pos);
            if ext::HSD_CObjSetCurrent(cobj) == 0 {
                continue;
            }

            ext::Player_80036978(i as i32, &mut world_pos);
            let mut is_outside = true;
            if !(world_pos.x < ext::Stage_GetCamBoundsLeftOffset()) && !(world_pos.x > ext::Stage_GetCamBoundsRightOffset()) {
                is_outside = false;
            }
            let x_blend: f32;
            if is_outside {
                x_blend = 0.0;
            } else {
                let x_class: f32;
                if world_pos.x < ext::Stage_GetCamBoundsLeftOffset() {
                    x_class = 0.0;
                } else if world_pos.x > ext::Stage_GetCamBoundsRightOffset() {
                    x_class = 3.0;
                } else if world_pos.x < 0.5f32 * (ext::Stage_GetCamBoundsLeftOffset() + ext::Stage_GetCamBoundsRightOffset()) {
                    x_class = 1.0;
                } else {
                    x_class = 2.0;
                }
                if (x_class as i32 - 1) == 0 {
                    // 1.0 - ((x - L) / ((0.5 * (L + R)) - L))
                    let num = world_pos.x - ext::Stage_GetCamBoundsLeftOffset();
                    let sum = ext::Stage_GetCamBoundsLeftOffset() + ext::Stage_GetCamBoundsRightOffset();
                    let l3 = ext::Stage_GetCamBoundsLeftOffset();
                    let den = fmadds(0.5, sum, -l3);
                    x_blend = 1.0f32 - (num / den);
                } else {
                    // 1.0 - ((x - (0.5 * (L + R))) / (R - (0.5 * (L + R))))
                    let sum1 = ext::Stage_GetCamBoundsLeftOffset() + ext::Stage_GetCamBoundsRightOffset();
                    let num = fmadds(-0.5, sum1, world_pos.x);
                    let r3 = ext::Stage_GetCamBoundsRightOffset();
                    let sum2 = ext::Stage_GetCamBoundsLeftOffset() + ext::Stage_GetCamBoundsRightOffset();
                    let den = fmadds(-0.5, sum2, r3);
                    x_blend = 1.0f32 - (num / den);
                }
            }
            let x_inv = 1.0f32 - x_blend;
            is_outside = true;
            if !(world_pos.y > ext::Stage_GetCamBoundsTopOffset()) && !(world_pos.y < ext::Stage_GetCamBoundsBottomOffset()) {
                is_outside = false;
            }
            let y_blend: f32;
            if is_outside {
                y_blend = 0.0;
            } else {
                let y_class: f32;
                if world_pos.y > ext::Stage_GetCamBoundsTopOffset() {
                    y_class = 0.0;
                } else if world_pos.y < ext::Stage_GetCamBoundsBottomOffset() {
                    y_class = 3.0;
                } else if world_pos.y > 0.5f32 * (ext::Stage_GetCamBoundsTopOffset() + ext::Stage_GetCamBoundsBottomOffset()) {
                    y_class = 1.0;
                } else {
                    y_class = 2.0;
                }
                if (y_class as i32 - 1) == 0 {
                    // 1.0 - ((T - y) / -((0.5 * (T + B)) - T))
                    let num = ext::Stage_GetCamBoundsTopOffset() - world_pos.y;
                    let sum = ext::Stage_GetCamBoundsTopOffset() + ext::Stage_GetCamBoundsBottomOffset();
                    let t3 = ext::Stage_GetCamBoundsTopOffset();
                    let den = -fmadds(0.5, sum, -t3);
                    y_blend = 1.0f32 - (num / den);
                } else {
                    // 1.0 - (((0.5 * (T + B)) - y) / ((0.5 * (T + B)) - B))
                    let sum1 = ext::Stage_GetCamBoundsTopOffset() + ext::Stage_GetCamBoundsBottomOffset();
                    let num = fmadds(0.5, sum1, -world_pos.y);
                    let sum2 = ext::Stage_GetCamBoundsTopOffset() + ext::Stage_GetCamBoundsBottomOffset();
                    let b3 = ext::Stage_GetCamBoundsBottomOffset();
                    let den = fmadds(0.5, sum2, -b3);
                    y_blend = 1.0f32 - (num / den);
                }
            }
            let y_inv = 1.0f32 - y_blend;
            ifMagnify_GetCornerColors(colors.as_mut_ptr(), &mut world_pos);

            let y_blend = 1.0f32 - y_inv;
            let x_blend = 1.0f32 - x_inv;
            let mix0 = x_inv * y_blend;
            let mix1 = x_blend * y_blend;
            let mix2 = x_blend * y_inv;
            let mix3 = x_inv * y_inv;
            // (c0 * mix1) + (c1 * mix0) + (c2 * mix2) + (c3 * mix3): fused as clang does, left multiply first
            let chan = |c0: u8, c1: u8, c2: u8, c3: u8| -> u8 {
                let t1 = fmadds(c0 as f32, mix1, c1 as f32 * mix0);
                let t2 = fmadds(c2 as f32, mix2, t1);
                let t3 = fmadds(c3 as f32, mix3, t2);
                f2u8(t3 as f64)
            };
            let ra = chan(colors[0].a, colors[1].a, colors[2].a, colors[3].a);
            let rr = chan(colors[0].r, colors[1].r, colors[2].r, colors[3].r);
            let rg = chan(colors[0].g, colors[1].g, colors[2].g, colors[3].g);
            let rb = chan(colors[0].b, colors[1].b, colors[2].b, colors[3].b);

            ext::HSD_SetEraseColor(rr, rg, rb, ra);
            ext::HSD_CObjEraseScreen(cobj, 1, 0, 1);
            ext::HSD_GObj_804D7814 = fighter_gobj;
            ext::ftDrawCommon_80080C28(fighter_gobj, 0);
            ext::ftDrawCommon_80080C28(fighter_gobj, 1);
            ext::ftDrawCommon_80080C28(fighter_gobj, 2);
            ext::HSD_GObj_804D7814 = null_mut();
            ext::lb_800122C8((*player).idesc, 0, 0, 1);
            ext::HSD_CObjEndCurrent();
            (*player).state.set_is_offscreen(1);
        }

        ext::HSD_CObjSetOrtho(cobj, top, bottom, left, right);
    }
}

pub unsafe extern "C" fn ifMagnify_802FC3BC() { melee_core::ct!("ifMagnify_802FC3BC");}

pub unsafe extern "C" fn ifMagnify_802FC3C0(slot: i32) {
    melee_core::ct!("ifMagnify_802FC3C0");
    let player = &raw mut ifMagnify_804A1DE0.player[slot as usize];
    let mut child: *mut HSD_JObj = null_mut();
    if !(*player).gobj.is_null() {
        ext::HSD_GObjFree((*player).gobj);
    }

    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    ext::GObj_InitUserData(gobj, 0xE, core::mem::transmute::<unsafe extern "C" fn(), Option<unsafe extern "C" fn(*mut c_void)>>(ifMagnify_802FC3BC), player.cast());

    let jobj = ext::HSD_JObjLoadJoint((**(ifMagnify_804A1DE0.model_desc as *mut *mut DynamicModelDesc)).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ifMagnify_802FB8C0), 0xB, 0);

    ext::lb_80011E24(jobj, &mut child, &[2]);
    if slot == 0 {
        (*player).idesc = (*(*(*(*(*child).u.dobj).next).mobj).tobj).imagedesc;
    } else {
        let base = &raw mut ifMagnify_804A1DE0;
        (*base).image_descs[slot as usize - 1] = *ifMagnify_804A1DE0.player[0].idesc;
        (*player).idesc = &raw mut (*base).image_descs[slot as usize - 1];
        let w = (*(*player).idesc).width;
        let h = (*(*player).idesc).height;
        let f = (*(*player).idesc).format;
        (*(*player).idesc).image_ptr = ext::HSD_MemAlloc(((ext::GXGetTexBufferSize(w, h, f as u32, 0, 0).wrapping_add(0x1F)) & !0x1F) as i32);
        (*(*(*(*(*child).u.dobj).next).mobj).tobj).imagedesc = (*player).idesc;
    }

    ext::lb_80011E24(jobj, &mut (*player).jobj, &[1]);

    {
        let mut color = _GXColor { r: 0, g: 0, b: 0, a: 0 };
        ifMagnify_GetPlayerColor(&mut color, slot);

        let mobj = (*(*(*player).jobj).u.dobj).mobj;
        (*(*mobj).mat).diffuse.r = color.r;
        (*(*mobj).mat).diffuse.g = color.g;
        (*(*mobj).mat).diffuse.b = color.b;

        let mobj = (*(*child).u.dobj).mobj;
        (*(*mobj).mat).diffuse.r = color.r;
        (*(*mobj).mat).diffuse.g = color.g;
        (*(*mobj).mat).diffuse.b = color.b;
    }

    (*player).gobj = gobj;
    (*player).state.set_is_offscreen(0);
    (*player).state.set_ignore_offscreen(0);
}

pub unsafe extern "C" fn ifMagnify_802FC618() {
    melee_core::ct!("ifMagnify_802FC618");
    let player0 = &raw mut ifMagnify_804A1DE0.player[0];
    let mut viewport = _HSD_RectS16 { xmin: 0, xmax: 0, ymin: 0, ymax: 0 };

    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let cobj = ext::lb_80013B14((&raw mut ifMagnify_803F97E8).cast());
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(
        gobj,
        core::mem::transmute::<unsafe extern "C" fn(*mut HSD_GObj), Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>>(ifMagnify_802FBBDC),
        0,
    );
    // see ifMagnify_UpdateLogicOffscreen
    ifMagnify_LogicArmed = 1;
    (*gobj).gxlink_prios = 0x10;

    let idesc = (*player0).idesc;
    let half_height = 0.1f32 * (*idesc).height as f32;
    let half_width = 0.1f32 * (*idesc).width as f32;
    ext::HSD_CObjSetOrtho(cobj, half_height, -half_height, -half_width, half_width);

    viewport.xmin = 0;
    viewport.xmax = (*(*player0).idesc).width as i16;
    viewport.ymin = 0;
    viewport.ymax = (*(*player0).idesc).height as i16;
    ext::HSD_CObjSetViewport(cobj, &mut viewport);
    ext::HSD_CObjSetScissorx4(cobj, viewport.xmin as u16, viewport.xmax as u16, viewport.ymin as u16, viewport.ymax as u16);
}

pub unsafe extern "C" fn ifMagnify_802FC750() {
    melee_core::ct!("ifMagnify_802FC750");
    let base = &raw mut ifMagnify_804A1DE0;
    for i in 0..6usize {
        if !(*base).player[i].gobj.is_null() {
            ext::HSD_GObjFree((*base).player[i].gobj);
            (*base).player[i].gobj = null_mut();
        }
    }
}

pub unsafe extern "C" fn ifMagnify_802FC7C0(magnify: *mut ifMagnify) {
    melee_core::ct!("ifMagnify_802FC7C0");
    // the C reads this const through a volatile pointer
    let default_val: i32 = core::ptr::read_volatile(&raw const ifMagnify_804DDB60);

    let result = ext::Ground_801C0604();
    if !result.is_null() {
        (*magnify).x4 = *(result as *const i32);
    } else {
        (*magnify).x4 = default_val;
    }

    let result = ext::Ground_801C0618();
    if !result.is_null() {
        (*magnify).x8 = *(result as *const i32);
    } else {
        (*magnify).x8 = default_val;
    }

    let result = ext::Ground_801C062C();
    if !result.is_null() {
        (*magnify).xC = *(result as *const i32);
    } else {
        (*magnify).xC = default_val;
    }

    let result = ext::Ground_801C0640();
    if !result.is_null() {
        (*magnify).x10 = *(result as *const i32);
    } else {
        (*magnify).x10 = default_val;
    }
}

pub unsafe extern "C" fn ifMagnify_802FC870() {
    melee_core::ct!("ifMagnify_802FC870");
    // memzero(&ifMagnify_804A1DE0, 0x74): the GameCube head (model_desc .. player[5]) of the record
    ext::memzero((&raw mut ifMagnify_804A1DE0).cast(), core::mem::offset_of!(ifMagnify, image_descs) as i32);
    ifMagnify_802FC7C0(&raw mut ifMagnify_804A1DE0);
    let archive = ext::ifAll_GetArchive();
    ext::lbArchive_LoadSections(
        *archive,
        (&raw mut ifMagnify_804A1DE0).cast(),
        ifMagnify_804D57E8.as_ptr().cast(),
        &[],
    );
    for i in 0..6 {
        ifMagnify_802FC3C0(i);
    }
    ifMagnify_802FC618();
}

pub unsafe extern "C" fn ifMagnify_802FC8E8() {
    melee_core::ct!("ifMagnify_802FC8E8");
    for i in 0..6usize {
        ifMagnify_804A1DE0.player[i].state.set_ignore_offscreen(1);
    }
}

pub unsafe extern "C" fn ifMagnify_802FC940() {
    melee_core::ct!("ifMagnify_802FC940");
    for i in 0..6usize {
        ifMagnify_804A1DE0.player[i].state.set_ignore_offscreen(0);
    }
}

/// TARGET_PC: stop computing the logic-side off-screen flags (scene init).
pub unsafe extern "C" fn ifMagnify_LogicDisarm() {
    melee_core::ct!("ifMagnify_LogicDisarm");
    ifMagnify_LogicArmed = 0;
    for i in 0..6usize {
        ifMagnify_LogicOffscreen[i] = 0;
    }
}

/// TARGET_PC: the render pass's off-screen predicate evaluated by logic (see the C comment),
/// then the fighters' off-camera flags for the next frame.
pub unsafe extern "C" fn ifMagnify_UpdateLogicOffscreen() {
    melee_core::ct!("ifMagnify_UpdateLogicOffscreen");
    let magnify = &raw mut ifMagnify_804A1DE0;
    if ifMagnify_LogicArmed == 0 {
        return;
    }
    let show = ifMagnify_IsHUDVisible();
    for i in 0..6usize {
        let fighter_gobj = ext::Player_GetEntity(i as i32);
        ifMagnify_LogicOffscreen[i] = (show
            && (*magnify).player[i].state.ignore_offscreen() == 0
            && !fighter_gobj.is_null()
            && ext::ftLib_80086B64(fighter_gobj) != 0
            && ext::ftLib_80086ED0(fighter_gobj) != 0) as u8;
    }
    if !ext::Camera_80030A50().is_null() {
        ext::Camera_8002A4AC(ext::Camera_80030A50());
    }
    for i in 0..6usize {
        let fighter_gobj = ext::Player_GetEntity(i as i32);
        if !fighter_gobj.is_null() {
            ext::ftLib_UpdateLogicOffCamera(fighter_gobj);
        }
    }
}

pub unsafe extern "C" fn ifMagnify_802FC998(ply_slot: i32) -> bool {
    melee_core::ct!("ifMagnify_802FC998");
    if ifMagnify_LogicArmed != 0 {
        return ifMagnify_LogicOffscreen[ply_slot as usize] != 0;
    }
    ifMagnify_804A1DE0.player[ply_slot as usize].state.is_offscreen() != 0
}
