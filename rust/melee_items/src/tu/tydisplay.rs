//! Native port of src/melee/ty/tydisplay.c: the Trophy Display ("Collection") scene. Lays the
//! collected trophies out on a table (spiral, rows or pyramid), builds their display models,
//! fits and drives the camera, and exposes the per-category archive helpers other scenes use
//! (`tyDisplay_8031C454` / `tyDisplay_8031C5E4`).
//!
//! Every function of the TU is here. Callees in other TUs go through `ext` (an unset hook
//! panics "TODO: <name> is not ported"; names are the C names); the trophy functions of
//! `toy.c` are called directly from [`super::toy`].
//!
//! Native-layout notes: the three symbol-name tables are separate `[&str; 43]` arrays (the
//! retail build indexes across adjacent tables; the PC build already names each table); the
//! name getters return `*const i8` where the C returns the address as an `s32`;
//! `_tyDisplay_804A2DD0` (a `TyDspArchiveHolder` in the C) is a `ToyListEntry`, the type
//! `Toy_80308250` / `Toy_803087F4` read it as; allocation sizes are `size_of` of the native
//! records.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ty/tydisplay.c`): the stick dead-zone
//! remap `-(0.2f*sign - v)/0.8f`, the pan updates `stick*(0.02f*fov) +/- angle`, the ring layout's
//! `2.0f*HSD_Randf() + pos` and `x*x + z*z`, the distance `dx*dx + dz*dz`, `9.0f*col + xoff`
//! and `x08 * (1/300) + 3.0f`; the MSL `sqrtf` fuses its f64 Newton step. Each is marked `// fused`.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_unsafe,
    static_mut_refs,
    unused_imports,
    unused_variables,
    unused_mut,
    unused_assignments,
    dead_code,
    clippy::all
)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf, tanf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::inline::*;
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
    fn DevText_80302AC0(a0: *mut DevText) => unsafe { melee_world::hud::textlib::devtext_80302ac0(&mut *a0) };
    fn DevText_Create(a0: i8, a1: i32, a2: i32, a3: i32, a4: i32, a5: *mut i8) -> *mut DevText => unsafe { melee_world::tu::textlib::DevText_Create(a0, a1, a2, a3, a4, a5) };
    fn DevText_Erase(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_Erase(a0) };
    fn DevText_GetGObj() -> *mut HSD_GObj => unsafe { melee_world::tu::textdraw::DevText_GetGObj() };
    fn DevText_HideCursor(a0: *mut DevText) => unsafe { melee_world::tu::textlib::DevText_HideCursor(a0) };
    fn DevText_Print(a0: *mut DevText, a1: *mut i8) => unsafe { melee_world::tu::textlib::DevText_Print(a0, a1) };
    fn DevText_Remove(a0: *mut *mut DevText) => unsafe { melee_world::tu::textdraw::DevText_Remove(a0) };
    fn DevText_SetBGColor(a0: *mut DevText, a1: _GXColor) -> _GXColor => unsafe { melee_world::tu::textlib::DevText_SetBGColor(a0, a1) };
    fn DevText_SetCursorXY(a0: *mut DevText, a1: i32, a2: i32) => unsafe { melee_world::tu::textlib::DevText_SetCursorXY(a0, a1, a2) };
    fn DevText_SetScale(a0: *mut DevText, a1: f32, a2: f32) => unsafe { melee_world::tu::textlib::DevText_SetScale(a0, a1, a2) };
    fn DevText_SetTextColor(a0: *mut DevText, a1: _GXColor) -> _GXColor => unsafe { melee_world::tu::textlib::DevText_SetTextColor(a0, a1) };
    fn DevText_Show(a0: *mut HSD_GObj, a1: *mut DevText) => unsafe { melee_world::tu::textdraw::DevText_Show(a0, a1) };
    fn DevText_StoreColorIndex(a0: *mut DevText, a1: u8) -> u8 => unsafe { melee_world::tu::textlib::DevText_StoreColorIndex(a0, a1) };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void => unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1) };
    fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjGetFov(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetFov(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_FogLoadDesc(a0: *mut HSD_FogDesc) -> *mut HSD_Fog;
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObjProc_RemoveAllProcs(a0: *mut HSD_GObj);
    fn HSD_GObj_80390CD4(a0: *mut HSD_GObj);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn HSD_JObjAlloc() -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjAlloc() };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_LObjSetColor(a0: *mut HSD_LObj, a1: _GXColor) => unsafe { melee_runtime::tu::lobj::HSD_LObjSetColor(a0, a1) };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void => unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0) };
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn gm_801A4B60();
    fn lbArchive_80016EFC(a0: *mut HSD_Archive) => unsafe { melee_runtime::tu::lbarchive::lbArchive_80016EFC(a0) };
    fn lbAudioAx_80024030(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0) };
    fn lbLang_IsSavedLanguageJP() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageJP() as _ };
    fn lbLang_IsSettingUS() -> i32 => unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingUS() as _ };
    fn lbVector_ApplyEulerRotation(a0: *mut Vec, a1: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_ApplyEulerRotation(a0, a1) };
    fn lb_8000D008(a0: f32, a1: f32) -> f32 => unsafe { melee_runtime::tu::lb_00CE::lb_8000D008(a0, a1) };
    fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16, a3: i32) -> i32 => unsafe { melee_runtime::tu::lbspdisplay::lb_8001204C(a0, a1, a2, a3) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
    fn lbArchive_LoadSymbols(a0: *const i8, a1: &[*mut c_void]) -> *mut HSD_Archive => unsafe { melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(a0 as _, a1) };
    fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut HSD_Archive;
    fn DevText_Printf(a0: *mut DevText, a1: &str);
    fn memzero(a0: *mut c_void, a1: i32) => unsafe { melee_runtime::tu::lb_00B0::memzero(a0, a1) };
    fn mn_8022F218() -> i32;
    fn mn_8022F268();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_JObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    /// Variadic C callees modelled by hand (fixed arity; the format is applied by the caller).
    pub static mut OSReport_hook: Option<fn(&str)> = None;
    /// `OSReport`: output only; unset = discard.
    pub fn OSReport(s: &str) {
        if let Some(f) = unsafe { OSReport_hook } {
            f(s)
        }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut DbLevel: DbLKind = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_FogKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT


use super::toy::{
    c_str, cstr, Dirty, Toy_803048C0, Toy_803049F4, Toy_80304B0C, Toy_80304CC8, Toy_80305058, Toy_80305B88, Toy_80305C44,
    Toy_80305D00, Toy_80305DB0, Toy_80305EB4, Toy_80305FB8, Toy_803060BC, Toy_803068E0, Toy_80306930, Toy_80306A48, Toy_80308250,
    Toy_803087F4, Toy_803124BC, Toy_8031263C, Toy_GetTrophyTotal, Toy_LoadLObjList, Toy_804A284C, Toy_804A2AA8,
    Toy_sbss_804D6EAC, Toy_sbss_804D6EB0, Toy_sbss_804D6EC8, Toy_sbss_804D6ED4,
};

/// `HSD_ASSERT(line, 0)` failure.
#[cold]
fn hsd_assert_fail(line: u32) -> ! {
    panic!("tydisplay.c:{line}: assertion failed")
}

const AT_RE_TOY_TEXT: i32 = 11;

#[inline]
unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}

#[inline]
unsafe fn toy_mode_state() -> *mut TyModeState {
    &raw mut Toy_804A284C as *mut TyModeState
}

/// The stick dead-zone remap `(f32)(-(0.2f*sign - v)/0.8f)` with the shipping fused
/// multiply-subtract `fmuladd(0.2, sign, -v)`.
#[inline]
fn stick_remap(val: f32) -> f32 {
    if val > -0.2 && val < 0.2 {
        0.0
    } else {
        let sign: i32 = if val > 0.0 { 1 } else { -1 };
        -fmadds(0.2, sign as f32, -val) / 0.8 // fused
    }
}

// ---------------------------------------------------------------------------------------------
// Data defined by tydisplay.c
// ---------------------------------------------------------------------------------------------

const _tyDisplay_804D6F10_len: usize = 300;

/* 4A2D98 */
static mut _tyDisplay_devtext_buf: [u8; 9 * (3 * 2)] = [0; 9 * (3 * 2)];
/* 4A2DD0: used as a `ToyListEntry` (its `archive` is the holder's +0x14). */
static mut _tyDisplay_804A2DD0: ToyListEntry = unsafe { core::mem::zeroed() };
/* 4A2DE8 */
static mut _tyDisplay_804A2DE8: [*mut HSD_Archive; 0xB0 / 4] = [core::ptr::null_mut(); 0xB0 / 4];
/* 4D6F10 */
static mut _tyDisplay_804D6F10: *mut *mut HSD_JObj = core::ptr::null_mut();
/* 4D6F14 */
static mut _tyDisplay_804D6F14: *mut TyDspGrid = core::ptr::null_mut();
/* 4D6F18 */
static mut _tyDisplay_804D6F18: *mut TyDspConfig = core::ptr::null_mut();
/* 4D6F1C */
static mut _tyDisplay_804D6F1C: *mut TyDspBgData = core::ptr::null_mut();
/* 4D6F20 */
static mut _tyDisplay_804D6F20: i32 = 0;
/* 4D6F24 */
static mut _tyDisplay_804D6F24: *mut DevText = core::ptr::null_mut();
/* 4D6F28 */
static mut _tyDisplay_804D6F28: i32 = 0;
/* 4D6F2C */
static mut _tyDisplay_804D6F2C: *mut HSD_GObj = core::ptr::null_mut();

/* 3FF01C */
static _tyDisplay_803FF01C: [u8; 22] = *b"ToyDspStand_Top_joint\0";
/* 4D5ABC */
static mut _tyDisplay_804D5ABC: [u16; 1] = [0x15];
/* C8C8C8FF */
static _tyDisplay_color_C8C8C8FF: GXColor = GXColor { r: 0xC8, g: 0xC8, b: 0xC8, a: 0xFF };
static _tyDisplay_804DE024: GXColor = GXColor { r: 0xE2, g: 0xE2, b: 0xE2, a: 0xFF };
static _tyDisplay_804DE028: GXColor = GXColor { r: 0x40, g: 0x80, b: 0xD0, a: 0x60 };

/* 3FF11C */
static mut _tyDisplay_803FF11C: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 130.0, z: 500.0 }, robjdesc: core::ptr::null_mut() };
/* 3FF130 */
static mut _tyDisplay_803FF130: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: core::ptr::null_mut() };
/* 3FF144 */
static mut _tyDisplay_803FF144: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: core::ptr::null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut _tyDisplay_803FF11C,
    interest: &raw mut _tyDisplay_803FF130,
    roll: 0.0,
    up_vector: core::ptr::null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 30.0,
    aspect: 913.0f32 / 750.0f32,
};

const fn names<const N: usize>(s: [&'static str; N]) -> [*const u8; N] {
    let mut out = [core::ptr::null::<u8>(); N];
    let mut i = 0;
    while i < N {
        out[i] = s[i].as_ptr();
        i += 1;
    }
    out
}

/// Joint symbol names per category (`_tyDisplay_803B8988`), NUL-terminated.
static _tyDisplay_803B8988: [&str; 43] = [
    "ToyDspQues_Top_joint\0", "ToyDspMycharaCmA_Top_joint\0", "ToyDspMycharaCmB_Top_joint\0", "ToyDspMycharaCmC_Top_joint\0",
    "ToyDspMycharaCmD_Top_joint\0", "ToyDspMycharaCmE_Top_joint\0", "ToyDspMycharaR1A_Top_joint\0", "ToyDspMycharaR1B_Top_joint\0",
    "ToyDspMycharaR1C_Top_joint\0", "ToyDspMycharaR1D_Top_joint\0", "ToyDspMycharaR1E_Top_joint\0", "ToyDspMycharaR2A_Top_joint\0",
    "ToyDspMycharaR2B_Top_joint\0", "ToyDspMycharaR2C_Top_joint\0", "ToyDspMycharaR2D_Top_joint\0", "ToyDspMycharaR2E_Top_joint\0",
    "ToyDspMapA_Top_joint\0", "ToyDspMapB_Top_joint\0", "ToyDspMapC_Top_joint\0", "ToyDspMapD_Top_joint\0",
    "ToyDspMapE_Top_joint\0", "ToyDspFgseriesA_Top_joint\0", "ToyDspFgseriesB_Top_joint\0", "ToyDspFgseriesC_Top_joint\0",
    "ToyDspFgseriesD_Top_joint\0", "ToyDspFgseriesE_Top_joint\0", "ToyDspFgetcA_Top_joint\0", "ToyDspFgetcB_Top_joint\0",
    "ToyDspFgetcC_Top_joint\0", "ToyDspFgetcD_Top_joint\0", "ToyDspFgetcE_Top_joint\0", "ToyDspPokemonA_Top_joint\0",
    "ToyDspPokemonB_Top_joint\0", "ToyDspPokemonC_Top_joint\0", "ToyDspPokemonD_Top_joint\0", "ToyDspPokemonE_Top_joint\0",
    "ToyDspItemA_Top_joint\0", "ToyDspItemB_Top_joint\0", "ToyDspItemC_Top_joint\0", "ToyDspItemD_Top_joint\0",
    "ToyDspItemE_Top_joint\0", "ToyDspStand_Top_joint\0", "ToyDspQues_Top_joint\0",
];

/// Matanim symbol names per category (`_tyDisplay_803B8A34`), NUL-terminated.
static _tyDisplay_803B8A34: [&str; 43] = [
    "\0",
    "ToyDspMycharaCmA_Top_matanim_joint\0",
    "ToyDspMycharaCmB_Top_matanim_joint\0",
    "ToyDspMycharaCmC_Top_matanim_joint\0",
    "ToyDspMycharaCmD_Top_matanim_joint\0",
    "ToyDspMycharaCmE_Top_matanim_joint\0",
    "ToyDspMycharaR1A_Top_matanim_joint\0",
    "ToyDspMycharaR1B_Top_matanim_joint\0",
    "ToyDspMycharaR1C_Top_matanim_joint\0",
    "ToyDspMycharaR1D_Top_matanim_joint\0",
    "ToyDspMycharaR1E_Top_matanim_joint\0",
    "ToyDspMycharaR2A_Top_matanim_joint\0",
    "ToyDspMycharaR2B_Top_matanim_joint\0",
    "ToyDspMycharaR2C_Top_matanim_joint\0",
    "ToyDspMycharaR2D_Top_matanim_joint\0",
    "ToyDspMycharaR2E_Top_matanim_joint\0",
    "ToyDspMapA_Top_matanim_joint\0",
    "ToyDspMapB_Top_matanim_joint\0",
    "ToyDspMapC_Top_matanim_joint\0",
    "ToyDspMapD_Top_matanim_joint\0",
    "ToyDspMapE_Top_matanim_joint\0",
    "ToyDspFgseriesA_Top_matanim_joint\0",
    "ToyDspFgseriesB_Top_matanim_joint\0",
    "ToyDspFgseriesC_Top_matanim_joint\0",
    "ToyDspFgseriesD_Top_matanim_joint\0",
    "ToyDspFgseriesE_Top_matanim_joint\0",
    "ToyDspFgetcA_Top_matanim_joint\0",
    "ToyDspFgetcB_Top_matanim_joint\0",
    "ToyDspFgetcC_Top_matanim_joint\0",
    "ToyDspFgetcD_Top_matanim_joint\0",
    "ToyDspFgetcE_Top_matanim_joint\0",
    "ToyDspPokemonA_Top_matanim_joint\0",
    "ToyDspPokemonB_Top_matanim_joint\0",
    "ToyDspPokemonC_Top_matanim_joint\0",
    "ToyDspPokemonD_Top_matanim_joint\0",
    "ToyDspPokemonE_Top_matanim_joint\0",
    "ToyDspItemA_Top_matanim_joint\0",
    "ToyDspItemB_Top_matanim_joint\0",
    "ToyDspItemC_Top_matanim_joint\0",
    "ToyDspItemD_Top_matanim_joint\0",
    "ToyDspItemE_Top_matanim_joint\0",
    "\0",
    "\0",
];

/// Archive file names per category (`_tyDisplay_803B8AE0`), NUL-terminated.
static _tyDisplay_803B8AE0: [&str; 43] = [
    "TyQuesD.dat\0", "TyMycCmA.dat\0", "TyMycCmB.dat\0", "TyMycCmC.dat\0", "TyMycCmD.dat\0", "TyMycCmE.dat\0", "TyMycR1A.dat\0",
    "TyMycR1B.dat\0", "TyMycR1C.dat\0", "TyMycR1D.dat\0", "TyMycR1E.dat\0", "TyMycR2A.dat\0", "TyMycR2B.dat\0", "TyMycR2C.dat\0",
    "TyMycR2D.dat\0", "TyMycR2E.dat\0", "TyMapA.dat\0", "TyMapB.dat\0", "TyMapC.dat\0", "TyMapD.dat\0", "TyMapE.dat\0",
    "TySeriA.dat\0", "TySeriB.dat\0", "TySeriC.dat\0", "TySeriD.dat\0", "TySeriE.dat\0", "TyEtcA.dat\0", "TyEtcB.dat\0",
    "TyEtcC.dat\0", "TyEtcD.dat\0", "TyEtcE.dat\0", "TyPokeA.dat\0", "TyPokeB.dat\0", "TyPokeC.dat\0", "TyPokeD.dat\0",
    "TyPokeE.dat\0", "TyItemA.dat\0", "TyItemB.dat\0", "TyItemC.dat\0", "TyItemD.dat\0", "TyItemE.dat\0", "TyStandD.dat\0",
    "TyQuesD.dat\0",
];

/// `table.entries[idx]` with the `idx == -1 -> 0` fix-up the three name getters share.
#[inline]
fn table_name(table: &'static [&'static str; 43], idx: i8) -> *const i8 {
    let idx = if idx == -1 { 0 } else { idx };
    table[idx as usize].as_ptr() as *const i8
}

// ---------------------------------------------------------------------------------------------
// Sorting helpers
// ---------------------------------------------------------------------------------------------

/// The shared body of `_tyDisplay_8031830C` / `_tyDisplay_80318714`: the C's three-way
/// quicksort over an array of 8-byte elements, ordered by `lt(a, b)` (the C compares
/// `base[i].val < base[lo].val`, resp. `>`). Element moves are swaps, as in the C.
unsafe fn tysort<T: Copy>(base: *mut T, lo: i32, hi: i32, lt: &dyn Fn(&T, &T) -> bool) {
    #[inline]
    unsafe fn swap<T: Copy>(base: *mut T, a: i32, b: i32) {
        let tmp = *base.offset(a as isize);
        *base.offset(a as isize) = *base.offset(b as isize);
        *base.offset(b as isize) = tmp;
    }
    if lo < hi {
        let mid = (lo + hi) / 2;
        if lo != mid {
            swap(base, lo, mid);
        }
        let mut pivot = lo;
        let mut i = lo + 1;
        while i <= hi {
            if lt(&*base.offset(i as isize), &*base.offset(lo as isize)) {
                pivot += 1;
                if pivot != i {
                    swap(base, pivot, i);
                }
            }
            i += 1;
        }
        if lo != pivot {
            swap(base, lo, pivot);
        }
        if lo < pivot - 1 {
            let mut mid2 = pivot + lo;
            mid2 = (mid2 - 1) / 2;
            if lo != mid2 {
                swap(base, lo, mid2);
            }
            let mut pivot2 = lo;
            i = lo + 1;
            while i <= pivot - 1 {
                if lt(&*base.offset(i as isize), &*base.offset(lo as isize)) {
                    pivot2 += 1;
                    if pivot2 != i {
                        swap(base, pivot2, i);
                    }
                }
                i += 1;
            }
            if lo != pivot2 {
                swap(base, lo, pivot2);
            }
            tysort(base, lo, pivot2 - 1, lt);
            tysort(base, pivot2 + 1, pivot - 1, lt);
        }
        if pivot + 1 < hi {
            let mid3 = (pivot + 1 + hi) / 2;
            if pivot + 1 != mid3 {
                swap(base, pivot + 1, mid3);
            }
            let mut pivot3 = pivot + 1;
            i = pivot + 2;
            while i <= hi {
                if lt(&*base.offset(i as isize), &*base.offset((pivot + 1) as isize)) {
                    pivot3 += 1;
                    if pivot3 != i {
                        swap(base, pivot3, i);
                    }
                }
                i += 1;
            }
            if pivot + 1 != pivot3 {
                swap(base, pivot + 1, pivot3);
            }
            tysort(base, pivot + 1, pivot3 - 1, lt);
            tysort(base, pivot3 + 1, hi, lt);
        }
    }
}

/// Sort `TySortElem`s ascending by their float `val` (position records are sorted through it).
pub unsafe fn _tyDisplay_8031830C(base: *mut TySortElem, lo: i32, hi: i32) {
    tysort(base, lo, hi, &|a: &TySortElem, b: &TySortElem| a.val < b.val);
}

/// Sort descending by `val` read as an integer.
pub unsafe fn _tyDisplay_80318714(base: *mut TySortElem, lo: i32, hi: i32) {
    tysort(base as *mut TySortElemInt, lo, hi, &|a: &TySortElemInt, b: &TySortElemInt| a.val > b.val);
}

#[inline]
fn val_bits(e: &TySortElem) -> i32 {
    e.val.to_bits() as i32
}

/// The `grid->sort` pre-pass the three layout builders inline: place the `cfg->x08` trophies,
/// then partially partition the first two thirds by int `val` and sort the halves.
unsafe fn layout_sort(cfg: *mut TyDspConfig, grid: *mut TyDspGrid) {
    _tyDisplay_80318B1C((*cfg).x08);
    if (*cfg).x08 > 1 {
        let n2 = ((*cfg).x08 / 3) * 2;
        if n2 > 0 {
            let sort = (*grid).sort.as_mut_ptr();
            let mid = n2 / 2;
            if mid != 0 {
                let tmp0 = *sort;
                *sort = *sort.offset(mid as isize);
                *sort.offset(mid as isize) = tmp0;
            }
            let mut pivot: i32 = 0;
            let mut n: i32 = 1;
            while n2 >= n {
                if val_bits(&*sort.offset(n as isize)) > val_bits(&*sort) {
                    pivot += 1;
                    if pivot != n {
                        let tmp1 = *sort.offset(pivot as isize);
                        *sort.offset(pivot as isize) = *sort.offset(n as isize);
                        *sort.offset(n as isize) = tmp1;
                    }
                }
                n += 1;
            }
            if pivot != 0 {
                let tmp2 = *sort;
                *sort = *sort.offset(pivot as isize);
                *sort.offset(pivot as isize) = tmp2;
            }
            _tyDisplay_80318714(sort, 0, pivot - 1);
            _tyDisplay_80318714(sort, pivot + 1, n2);
        }
    }
}

/// The `grid->pos` sort the ring layouts inline (partition on `z`, then `_tyDisplay_8031830C`).
unsafe fn layout_sort_pos(grid: *mut TyDspGrid, n2: i32) {
    let pos = (*grid).pos.as_mut_ptr();
    let mid = n2 / 2;
    if mid != 0 {
        let tmp0 = *pos;
        *pos = *pos.offset(mid as isize);
        *pos.offset(mid as isize) = tmp0;
    }
    let mut pivot: i32 = 0;
    let mut n: i32 = 1;
    while n2 >= n {
        if (*pos.offset(n as isize)).z < (*pos).z {
            pivot += 1;
            if pivot != n {
                let tmp1 = *pos.offset(pivot as isize);
                *pos.offset(pivot as isize) = *pos.offset(n as isize);
                *pos.offset(n as isize) = tmp1;
            }
        }
        n += 1;
    }
    if pivot != 0 {
        let tmp2 = *pos;
        *pos = *pos.offset(pivot as isize);
        *pos.offset(pivot as isize) = tmp2;
    }
    _tyDisplay_8031830C(pos as *mut TySortElem, 0, pivot - 1);
    _tyDisplay_8031830C(pos as *mut TySortElem, pivot + 1, n2);
}

/// `_tyDisplay_80318B1C`: pick the trophies shown (`arg0` of them, rotated by a random start)
/// and store each id and its int sort value in `grid->sort`.
pub unsafe fn _tyDisplay_80318B1C(arg0: i32) {
    let grid = _tyDisplay_804D6F14;
    let rand_result = if arg0 > 1 { ext::HSD_Randi(arg0 - 1) } else { 0 };
    let mut start = rand_result;
    if arg0 > TY_TROPHY_COUNT {
        let mut placed = 0;
        let mut i = 0;
        while placed < arg0 {
            if i >= TY_TROPHY_COUNT {
                let mut rand_id = ext::HSD_Randi(0x124);
                let mut check = tyDisplay_8031B9DC(rand_id);
                while (*check).x00 == -1 {
                    rand_id = ext::HSD_Randi(0x124);
                    check = tyDisplay_8031B9DC(rand_id);
                }
                (*grid).sort[start as usize].key = rand_id;
                let k = (*grid).sort[start as usize].key;
                (*grid).sort[start as usize].val = f32::from_bits(Toy_803060BC(k, 7) as i32 as u32);
                start += 1;
                if start >= arg0 {
                    start = 0;
                }
                placed += 1;
            } else {
                let check = tyDisplay_8031B9DC(i);
                if (*check).x00 != -1 {
                    (*grid).sort[start as usize].key = i;
                    (*grid).sort[start as usize].val = f32::from_bits(Toy_803060BC(i, 7) as i32 as u32);
                    start += 1;
                    if start >= arg0 {
                        start = 0;
                    }
                    placed += 1;
                }
            }
            i += 1;
        }
    } else {
        for i in 0..TY_TROPHY_COUNT {
            if Toy_803048C0(i) != 0 {
                tyDisplay_8031B9DC(i);
                (*grid).sort[start as usize].key = i;
                (*grid).sort[start as usize].val = f32::from_bits(Toy_803060BC(i, 7) as i32 as u32);
                start += 1;
                if start >= arg0 {
                    start = 0;
                }
            }
        }
    }
}

/// `dx*dx + dz*dz` with the shipping contraction `fmuladd(dx, dx, dz*dz)`.
#[inline]
fn _tyDisplay_80318CB4_calc_dist_sq(dz: f32, dx: f32) -> f32 {
    fmadds(dx, dx, dz * dz) // fused
}

/// `_tyDisplay_80318CB4_place_toys` (static inline).
unsafe fn _tyDisplay_80318CB4_place_toys(grid: *mut TyDspGrid, cfg: *mut TyDspConfig) {
    let mut jobj_idx: usize = 0;
    for k in 0..(*cfg).x08 {
        (*cfg).x78 = _tyDisplay_8031BC54((*grid).sort[k as usize].key);
        let gobj = (*cfg).x78;
        if !gobj.is_null() {
            *_tyDisplay_804D6F10.add(jobj_idx) = (*gobj).hsd_obj as *mut HSD_JObj;
            let j = *_tyDisplay_804D6F10.add(jobj_idx);
            HSD_JObjSetTranslateX(&mut *j, (*grid).pos[jobj_idx].x, &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *j, (*grid).pos[jobj_idx].z, &mut Dirty);
            jobj_idx += 1;
        }
    }
}

/// Spiral ("ring") layout (`arg0 != 0` fixed spacing, else jittered and randomised).
pub unsafe fn _tyDisplay_80318CB4(arg0: i32) {
    let grid = _tyDisplay_804D6F14;
    let cfg = _tyDisplay_804D6F18;
    let mut ring_count: i32 = 0;
    let mut ring_max: i32 = 6;
    let mut prev_ring_size: i32 = 0;
    let mut angle: f32 = 0.0;
    let mut base_step: f32;
    let mut radius: f32;
    let dist_limit: f32 = 8.0;
    let mut count: i32 = 0;

    ext::memzero(grid as *mut c_void, core::mem::size_of::<TyDspGrid>() as i32);
    (*grid).x08_min_z = -3.5;
    (*grid).x04_min_x = -3.5;
    (*grid).x10_max_z = 3.5;
    (*grid).x0C_max_x = 3.5;
    base_step = if arg0 != 0 { 9.0 } else { 11.0 };
    radius = base_step;
    let mut i: i32 = 0;
    loop {
        count = (*cfg).x08;
        if !(i < count) {
            break;
        }
        let iu = i as usize;
        if i == 0 {
            (*grid).pos[iu].x = 0.0;
            (*grid).pos[iu].z = 0.0;
        } else {
            let rad = 0.017453292f32 * angle;
            (*grid).pos[iu].x = radius * cosf(rad);
            (*grid).pos[iu].z = radius * sinf(rad);
            if arg0 == 0 {
                (*grid).pos[iu].x = fmadds(2.0, ext::HSD_Randf(), (*grid).pos[iu].x); // fused
                (*grid).pos[iu].z = fmadds(2.0, ext::HSD_Randf(), (*grid).pos[iu].z); // fused
            }
            if ext::HSD_Randi(3) != 0 {
                let theta = atan2f((*grid).pos[iu].z, (*grid).pos[iu].x);
                // MSL sqrtf (the inline Newton-Raphson on frsqrte)
                let mut mag = sqrtf(fmadds((*grid).pos[iu].x, (*grid).pos[iu].x, (*grid).pos[iu].z * (*grid).pos[iu].z));
                let start: i32 = if i < 0x24 {
                    0
                } else {
                    let ring_span = prev_ring_size + prev_ring_size;
                    i - (ring_span - 6)
                };
                let mut collided = 0;
                while collided == 0 {
                    (*grid).pos[iu].x = mag * cosf(theta);
                    let rad = theta;
                    (*grid).pos[iu].z = mag * sinf(rad);
                    let mut tries = (mag / 0.1f32) as i32;
                    if ext::HSD_Randi(2) != 0 {
                        let half = mag / 2.0;
                        if half as i32 > 1 {
                            tries -= ext::HSD_Randi(half as i32);
                        }
                    }
                    let mut k = i - 1;
                    while k >= start {
                        let dx = (*grid).pos[iu].x - (*grid).pos[k as usize].x;
                        let dist = sqrtf(_tyDisplay_80318CB4_calc_dist_sq((*grid).pos[iu].z - (*grid).pos[k as usize].z, dx));
                        if dist > 2.1474836e9f32 || dist < -2.1474836e9f32 {
                            ext::OSReport("*** tyDisplay Atari Irregul!\n");
                            hsd_assert_fail(0xC6);
                        }
                        let near = if dist as i32 <= dist_limit as i32 { 1 } else { 0 };
                        if near != 0 {
                            collided = 1;
                            break;
                        }
                        k -= 1;
                    }
                    if tries == 0 {
                        break;
                    }
                    if collided == 0 {
                        mag -= 0.1;
                    }
                }
            }
            ring_count += 1;
            if ring_count >= ring_max {
                base_step = if arg0 != 0 { 9.0 } else { 11.0 };
                radius += base_step;
                prev_ring_size = ring_max;
                ring_count = 0;
                ring_max += 6;
                angle = if arg0 != 0 { 0.0 } else { ext::HSD_Randi(0x1E) as f32 };
            } else {
                let full_turn = 360.0f32;
                angle += full_turn / ring_max as f32;
            }
        }
        if (*grid).pos[iu].x < (*grid).x04_min_x {
            (*grid).x04_min_x = (*grid).pos[iu].x;
        }
        if (*grid).pos[iu].x > (*grid).x0C_max_x {
            (*grid).x0C_max_x = (*grid).pos[iu].x;
        }
        if (*grid).pos[iu].z < (*grid).x08_min_z {
            (*grid).x08_min_z = (*grid).pos[iu].z;
        }
        if (*grid).pos[iu].z > (*grid).x10_max_z {
            (*grid).x10_max_z = (*grid).pos[iu].z;
        }
        i += 1;
    }
    if count > 1 {
        let sort_end = count - 1;
        if sort_end > 0 {
            layout_sort_pos(grid, sort_end);
        }
    }
    layout_sort(cfg, grid);
    _tyDisplay_80318CB4_place_toys(grid, cfg);
}

/// Row layout (staggered when `arg0 != 0`).
pub unsafe fn _tyDisplay_80319540(arg0: i32) {
    let cfg = _tyDisplay_804D6F18;
    let grid = _tyDisplay_804D6F14;
    let mut row: i32 = 0;
    let mut col: i32 = 0;
    ext::memzero(grid as *mut c_void, core::mem::size_of::<TyDspGrid>() as i32);
    (*grid).x08_min_z = -3.5;
    (*grid).x04_min_x = -3.5;
    (*grid).x10_max_z = 3.5;
    (*grid).x0C_max_x = 3.5;
    let mut remainder: i32 = if (*cfg).x08 <= 1 { 0 } else { (*cfg).x08 % (*cfg).x75 as i8 as i32 };
    {
        let mut i: i32 = 0;
        while i < (*cfg).x08 {
            let cur = &raw mut (*grid).pos[i as usize];
            if i == 0 {
                (*cur).x = 0.0;
                (*cur).z = 0.0;
            } else {
                (*cur).x = 9.0f32 * col as f32;
                if arg0 != 0 && (row % 2) != 0 {
                    (*cur).x += 3.5;
                }
                (*cur).z = 9.0f32 * row as f32;
            }
            col += 1;
            if remainder != 0 {
                remainder -= 1;
                if remainder == 0 {
                    col = 0;
                    row += 1;
                }
            } else if col >= (*cfg).x75 as i8 as i32 {
                col = 0;
                row += 1;
            }
            let px = (*cur).x;
            if px < (*grid).x04_min_x {
                (*grid).x04_min_x = px;
            }
            let px = (*cur).x;
            if px > (*grid).x0C_max_x {
                (*grid).x0C_max_x = px;
            }
            let pz = (*cur).z;
            if pz < (*grid).x08_min_z {
                (*grid).x08_min_z = pz;
            }
            let pz = (*cur).z;
            if pz > (*grid).x10_max_z {
                (*grid).x10_max_z = pz;
            }
            i += 1;
        }
    }
    layout_sort(cfg, grid);
    {
        let mut i: i32 = 0;
        while i < (*cfg).x08 {
            (*cfg).x78 = _tyDisplay_8031BC54((*grid).sort[i as usize].key);
            let gobj = (*cfg).x78;
            if !gobj.is_null() {
                *_tyDisplay_804D6F10.add(i as usize) = (*gobj).hsd_obj as *mut HSD_JObj;
                let j = *_tyDisplay_804D6F10.add(i as usize);
                HSD_JObjSetTranslateX(&mut *j, (*grid).pos[i as usize].x, &mut Dirty);
                HSD_JObjSetTranslateZ(&mut *j, (*grid).pos[i as usize].z, &mut Dirty);
            }
            i += 1;
        }
    }
}

/// Pyramid layout (`arg0 != 0` flips it).
pub unsafe fn _tyDisplay_80319994(arg0: i32) {
    let cfg = _tyDisplay_804D6F18;
    let grid = _tyDisplay_804D6F14;
    let mut xoff: f32 = 0.0;
    let pivot = arg0;
    let mut count: i32;
    ext::memzero(grid as *mut c_void, core::mem::size_of::<TyDspGrid>() as i32);
    (*grid).x08_min_z = -3.5;
    (*grid).x04_min_x = -3.5;
    (*grid).x10_max_z = 3.5;
    (*grid).x0C_max_x = 3.5;
    {
        let mut row: i32 = 0;
        let mut col: i32 = 0;
        let mut ring: i32 = 1;
        let mut i: i32 = 0;
        loop {
            count = (*cfg).x08;
            if !(i < count) {
                break;
            }
            let cur = &raw mut (*grid).pos[i as usize];
            if i == 0 {
                (*cur).x = 0.0;
                (*cur).z = 0.0;
            } else {
                (*cur).x = fmadds(9.0, col as f32, xoff); // fused
                if pivot != 0 {
                    (*cur).z = -9.0f32 * row as f32;
                } else {
                    (*cur).z = 9.0f32 * row as f32;
                }
            }
            col += 1;
            if col >= ring {
                xoff -= 4.5;
                col = 0;
                row += 1;
                ring += 1;
            }
            let x = (*cur).x;
            if x < (*grid).x04_min_x {
                (*grid).x04_min_x = x;
            }
            let x = (*cur).x;
            if x > (*grid).x0C_max_x {
                (*grid).x0C_max_x = x;
            }
            let z = (*cur).z;
            if z < (*grid).x08_min_z {
                (*grid).x08_min_z = z;
            }
            let z = (*cur).z;
            if z > (*grid).x10_max_z {
                (*grid).x10_max_z = z;
            }
            i += 1;
        }
        if pivot != 0 && count > 1 {
            let n2 = count - 1;
            if n2 > 0 {
                layout_sort_pos(grid, n2);
            }
        }
    }
    layout_sort(cfg, grid);
    {
        let mut count: i32 = 0;
        while count < (*cfg).x08 {
            (*cfg).x78 = _tyDisplay_8031BC54((*grid).sort[count as usize].key);
            let gobj = (*cfg).x78;
            if !gobj.is_null() {
                *_tyDisplay_804D6F10.add(count as usize) = (*gobj).hsd_obj as *mut HSD_JObj;
                let j = *_tyDisplay_804D6F10.add(count as usize);
                HSD_JObjSetTranslateX(&mut *j, (*grid).pos[count as usize].x, &mut Dirty);
                HSD_JObjSetTranslateZ(&mut *j, (*grid).pos[count as usize].z, &mut Dirty);
            }
            count += 1;
        }
    }
}

/// Fit the camera (position, fov, pan limits) and the table backdrop to the laid-out grid.
pub unsafe fn _tyDisplay_80319EF0() {
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp28 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let grid = _tyDisplay_804D6F14;
    let cfg = _tyDisplay_804D6F18;
    let cobj = (*(*cfg).x00).hsd_obj as *mut HSD_CObj;
    let bg = _tyDisplay_804D6F1C;
    let mut range = (*grid).x0C_max_x - (*grid).x04_min_x;
    if range < 0.0 {
        range = -range;
    }
    interest.x = range / 2.0 + (*grid).x04_min_x;
    if (*grid).x00 == 3 {
        interest.x = 0.0;
    }
    interest.y = 0.0;
    {
        let zmin = (*grid).x08_min_z;
        range = (*grid).x10_max_z - zmin;
        if range < 0.0 {
            range = -range;
        }
        interest.z = range / 2.0 + zmin;
    }
    let mut eyepos = interest;
    interest.z -= 10.0;
    (*cfg).x5C = interest;
    ext::HSD_CObjGetEyePosition(cobj, &mut sp28);
    sp28.x = eyepos.x;
    sp28.z = 500.0 + eyepos.z;
    (*cfg).x68 = sp28;
    ext::HSD_CObjSetInterest(cobj, &mut interest);
    ext::HSD_CObjSetEyePosition(cobj, &mut sp28);
    {
        let mut xrange = (*grid).x0C_max_x - (*grid).x04_min_x;
        if xrange < 0.0 {
            xrange = -xrange;
        }
        (*cfg).x40 = 14.0 + xrange;
    }
    (*cfg).x44 = 1.0;
    while 500.0f32 * tanf(0.017453292f32 * ((*cfg).x44 / 2.0)) < (*cfg).x40 / 2.0 {
        (*cfg).x44 += 0.1;
    }
    if (*cfg).x44 < 3.0 {
        (*cfg).x44 = 3.2;
    }
    ext::HSD_CObjSetFov(cobj, (*cfg).x44);
    range = 0.0033333334;
    (*cfg).x4C = fmadds((*cfg).x08 as f32, range, 3.0); // fused
    {
        let fov2 = (*cfg).x44;
        (*cfg).x50 = fov2 + fov2 / 5.0;
    }
    if (*cfg).x44 < 3.0 {
        (*cfg).x48 = ((*cfg).x50 - (*cfg).x4C) / 30.0;
    } else {
        (*cfg).x48 = ((*cfg).x50 - (*cfg).x4C) / 60.0;
    }
    {
        let mode = (*grid).x00;
        match mode {
            0 | 1 => {
                (*cfg).x54 = -((14.0 + (*cfg).x40) / 2.0 - (*cfg).x5C.x);
                (*cfg).x58 = (14.0 + (*cfg).x40) / 2.0 + (*cfg).x5C.x;
            }
            2 => {
                (*cfg).x54 = -((7.0 + (*cfg).x40) / 2.0 - (*cfg).x5C.x);
                (*cfg).x58 = (7.0 + (*cfg).x40) / 2.0 + (*cfg).x5C.x;
            }
            3 => {
                (*cfg).x54 = -((*cfg).x40 / 2.0 - (*cfg).x5C.x);
                (*cfg).x58 = (*cfg).x40 / 2.0 + (*cfg).x5C.x;
            }
            _ => {}
        }
    }
    {
        let xdiff = (*cfg).x58 - (*cfg).x54;
        (*cfg).x1C = 57.29578f32 * ext::lb_8000D008(xdiff / 2.0, 500.0);
    }
    (*cfg).x18 = 57.29578f32 * ext::lb_8000D008((*cfg).x40 / 2.0, 500.0);
    {
        HSD_JObjSetTranslate(&mut *((*(*bg).gobj4).hsd_obj as *mut HSD_JObj), &eyepos, &mut Dirty);
    }
    {
        let mut zrange = 14.0 + ((*grid).x10_max_z - (*grid).x08_min_z);
        range = (*grid).x0C_max_x - (*grid).x04_min_x;
        let xrange = range;
        let mut scale = ((*cfg).x08 / 30) as f32;
        if zrange < xrange {
            zrange = 14.0 + xrange;
        }
        if 38.0f32 * scale < zrange {
            while 38.0f32 * scale < zrange {
                scale += 0.1;
            }
        } else {
            while 38.0f32 * scale > zrange {
                scale -= 0.1;
            }
        }
        if scale > 2.1474836e9f32 || scale < -2.1474836e9f32 {
            ext::OSReport("*** tyDisplay Table Scale Irregul!\n");
            hsd_assert_fail(0x28C);
        }
        if scale as i32 != 0 {
            HSD_JObjSetScaleX(&mut *(*bg).jobj, scale, &mut Dirty);
            HSD_JObjSetScaleZ(&mut *(*bg).jobj, scale, &mut Dirty);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Camera procs
// ---------------------------------------------------------------------------------------------

/// Re-aim the camera's interest from the pan angles in `cfg->x0C/x10` (shared tail).
unsafe fn camera_aim_tail(cfg: *mut TyDspConfig) {
    let mut interest2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut tempvec1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut tempvec2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let cobj2 = (*(*cfg).x00).hsd_obj as *mut HSD_CObj;
    ext::HSD_CObjGetInterest(cobj2, &mut interest2);
    ext::HSD_CObjGetEyePosition(cobj2, &mut eye2);
    tempvec1.x = (*cfg).x68.x;
    tempvec1.y = 0.0;
    tempvec1.z = -500.0;
    tempvec2.x = 0.017453292f32 * (*cfg).x0C;
    tempvec2.y = 0.017453292f32 * (*cfg).x10;
    tempvec2.z = 0.0;
    ext::lbVector_ApplyEulerRotation(&mut tempvec1, &mut tempvec2);
    tempvec1.z = (*cfg).x5C.z;
    ext::HSD_CObjSetInterest(cobj2, &mut tempvec1);
}

/// Per-frame camera proc: stick pans, C-stick / triggers zoom, A resets, B / START leave.
pub unsafe extern "C" fn _tyDisplay_8031A4EC(arg0: *mut HSD_GObj) {
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;
    let cfg = _tyDisplay_804D6F18;
    ext::HSD_CObjGetInterest(cobj, &mut interest);
    ext::HSD_CObjGetEyePosition(cobj, &mut eye);
    let mut fov = ext::HSD_CObjGetFov(cobj);
    (*cfg).x20 = Toy_80305D00();
    (*cfg).x24 = Toy_80305DB0();
    (*cfg).x20 = stick_remap((*cfg).x20);
    (*cfg).x24 = stick_remap((*cfg).x24);
    (*cfg).x30 = Toy_80305EB4();
    (*cfg).x34 = Toy_80305FB8();
    (*cfg).x30 = stick_remap((*cfg).x30);
    (*cfg).x34 = stick_remap((*cfg).x34);
    if (*cfg).x74 != 0 {
        (*cfg).x74 = (*cfg).x74 - 1;
        return;
    }
    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::mn_8022F268();
        (*toy_mode_state()).x4 = 1;
        return;
    }
    if Toy_80305B88() & 0x1200 != 0 {
        sfxBack();
        (*toy_mode_state()).x4 = 1;
        return;
    }
    {
        let stick = (*cfg).x20;
        if stick != 0.0 {
            (*cfg).x10 = -fmadds(stick, 0.02f32 * fov, -(*cfg).x10); // fused
            if (*cfg).x10 < -(*cfg).x1C {
                (*cfg).x10 = -(*cfg).x1C;
            }
            if (*cfg).x10 > (*cfg).x1C {
                (*cfg).x10 = (*cfg).x1C;
            }
        }
    }
    {
        let stick = (*cfg).x24;
        if stick != 0.0 {
            (*cfg).x0C = fmadds(stick, 0.02f32 * fov, (*cfg).x0C); // fused
            if (*cfg).x0C < -(*cfg).x18 {
                (*cfg).x0C = -(*cfg).x18;
            }
            if (*cfg).x0C > (*cfg).x18 {
                (*cfg).x0C = (*cfg).x18;
            }
        }
    }
    if Toy_80305C44() & 0x424 != 0 {
        fov += (*cfg).x48;
        if fov > (*cfg).x50 {
            fov = (*cfg).x50;
        }
        ext::HSD_CObjSetFov(cobj, fov);
    }
    if Toy_80305C44() & 0x848 != 0 {
        fov -= (*cfg).x48;
        if fov < (*cfg).x4C {
            fov = (*cfg).x4C;
        }
        ext::HSD_CObjSetFov(cobj, fov);
    }
    if Toy_80305B88() & 0x100 != 0 {
        ext::HSD_CObjSetInterest(cobj, &mut (*cfg).x5C);
        ext::HSD_CObjSetFov(cobj, (*cfg).x44);
        (*cfg).x10 = 0.0;
        (*cfg).x0C = 0.0;
        ext::HSD_CObjSetEyePosition(cobj, &mut (*cfg).x68);
    }
    camera_aim_tail(cfg);
}

/// Debug "place trophies" camera proc (L held on entry): moves the held model with the d-pad.
pub unsafe extern "C" fn _tyDisplay_8031A94C(arg0: *mut HSD_GObj) {
    let mut sp7c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp70 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let cfg = _tyDisplay_804D6F18;
    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;
    let trophy = (*((*(*cfg).x78).hsd_obj as *mut HSD_JObj)).child;
    ext::HSD_CObjGetInterest(cobj, &mut sp7c);
    ext::HSD_CObjGetEyePosition(cobj, &mut sp70);
    let mut fov = ext::HSD_CObjGetFov(cobj);
    (*cfg).x20 = Toy_80305D00();
    (*cfg).x24 = Toy_80305DB0();
    (*cfg).x20 = stick_remap((*cfg).x20);
    (*cfg).x24 = stick_remap((*cfg).x24);
    (*cfg).x30 = Toy_80305EB4();
    (*cfg).x34 = Toy_80305FB8();
    (*cfg).x30 = stick_remap((*cfg).x30);
    (*cfg).x34 = stick_remap((*cfg).x34);
    if (*cfg).x74 != 0 {
        (*cfg).x74 = (*cfg).x74 - 1;
        return;
    }
    if Toy_80305C44() & 0x200 != 0 {
        _tyDisplay_804D6F28 += 1;
        if _tyDisplay_804D6F28 > 0x78 {
            sfxBack();
            (*toy_mode_state()).x4 = 1;
        }
    } else {
        _tyDisplay_804D6F28 = 0;
        if (Toy_80305C44() & 0x100 != 0 && (*cfg).x20 < -0.8) || (Toy_80305B88() & 1) != 0 {
            HSD_JObjAddTranslationX(&mut *trophy, -0.01, &mut Dirty);
            _tyDisplay_8031BA78((*cfg).x7C, 0, HSD_JObjGetTranslationX(&*trophy));
        }
        if (Toy_80305C44() & 0x100 != 0 && (*cfg).x20 > 0.8) || (Toy_80305B88() & 2) != 0 {
            HSD_JObjAddTranslationX(&mut *trophy, 0.01, &mut Dirty);
            _tyDisplay_8031BA78((*cfg).x7C, 0, HSD_JObjGetTranslationX(&*trophy));
        }
        if (Toy_80305C44() & 0x100 != 0 && (*cfg).x24 > 0.8) || (Toy_80305B88() & 8) != 0 {
            HSD_JObjAddTranslationZ(&mut *trophy, -0.01, &mut Dirty);
            _tyDisplay_8031BA78((*cfg).x7C, 2, HSD_JObjGetTranslationZ(&*trophy));
        }
        if (Toy_80305C44() & 0x100 != 0 && (*cfg).x24 < -0.8) || (Toy_80305B88() & 4) != 0 {
            HSD_JObjAddTranslationZ(&mut *trophy, 0.01, &mut Dirty);
            _tyDisplay_8031BA78((*cfg).x7C, 2, HSD_JObjGetTranslationZ(&*trophy));
        }
        if Toy_80305B88() & 0x20 != 0 {
            ext::HSD_GObjFree((*cfg).x78);
            (*cfg).x78 = core::ptr::null_mut();
            while (*cfg).x78.is_null() {
                (*cfg).x7C = (*cfg).x7C + 1;
                if (*cfg).x7C >= TY_TROPHY_COUNT {
                    (*cfg).x7C = 0;
                }
                (*cfg).x78 = _tyDisplay_8031BC54((*cfg).x7C);
            }
            return;
        }
        if Toy_80305B88() & 0x40 != 0 {
            ext::HSD_GObjFree((*cfg).x78);
            (*cfg).x78 = core::ptr::null_mut();
            while (*cfg).x78.is_null() {
                (*cfg).x7C = (*cfg).x7C - 1;
                if (*cfg).x7C < 0 {
                    (*cfg).x7C = 0x124;
                }
                (*cfg).x78 = _tyDisplay_8031BC54((*cfg).x7C);
            }
            return;
        }
        if Toy_80305C44() & 0x100 == 0 {
            let stick = (*cfg).x20;
            let zero = 0.0f32;
            if stick != zero {
                (*cfg).x10 = -fmadds(stick, 0.02f32 * fov, -(*cfg).x10); // fused
                if (*cfg).x10 < -(*cfg).x1C {
                    (*cfg).x10 = -(*cfg).x1C;
                }
                if (*cfg).x10 > (*cfg).x1C {
                    (*cfg).x10 = (*cfg).x1C;
                }
            }
            {
                let stick2 = (*cfg).x24;
                if stick2 != zero {
                    (*cfg).x0C = fmadds(stick2, 0.02f32 * fov, (*cfg).x0C); // fused
                    if (*cfg).x0C < -(*cfg).x18 {
                        (*cfg).x0C = -(*cfg).x18;
                    }
                    if (*cfg).x0C > (*cfg).x18 {
                        (*cfg).x0C = (*cfg).x18;
                    }
                }
            }
        }
        if (*cfg).x34 > 0.8 {
            sp70.y += 1.0;
            ext::HSD_CObjSetEyePosition(cobj, &mut sp70);
        }
        if (*cfg).x34 < -0.8 {
            sp70.y -= 1.0;
            ext::HSD_CObjSetEyePosition(cobj, &mut sp70);
        }
        if Toy_80305C44() & 0x400 != 0 {
            fov += (*cfg).x48;
            if fov > (*cfg).x50 {
                fov = (*cfg).x50;
            }
            ext::HSD_CObjSetFov(cobj, fov);
        }
        if Toy_80305C44() & 0x800 != 0 {
            fov -= (*cfg).x48;
            if fov < (*cfg).x4C {
                fov = (*cfg).x4C;
            }
            ext::HSD_CObjSetFov(cobj, fov);
        }
        if Toy_80305B88() & 0x1000 != 0 {
            ext::HSD_CObjSetInterest(cobj, &mut (*cfg).x5C);
            ext::HSD_CObjSetFov(cobj, (*cfg).x44);
            (*cfg).x10 = 0.0;
            (*cfg).x0C = 0.0;
            ext::HSD_CObjSetEyePosition(cobj, &mut (*cfg).x68);
        }
        camera_aim_tail(cfg);
    }
}

// ---------------------------------------------------------------------------------------------
// Scene
// ---------------------------------------------------------------------------------------------

/// Load the backdrop (table) model into `bg->gobj4`.
pub unsafe fn _tyDisplay_8031B1FC() {
    let ptr = _tyDisplay_804D6F1C;
    let zero = 0;
    if (*ptr).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        hsd_assert_fail(0x3FD);
    }
    let mut gobj = (*ptr).gobj0;
    if !gobj.is_null() {
        ext::HSD_GObjFree(gobj);
        (*ptr).gobj0 = core::ptr::null_mut();
    }
    gobj = (*ptr).gobj4;
    if !gobj.is_null() {
        ext::HSD_GObjFree(gobj);
        (*ptr).gobj4 = core::ptr::null_mut();
    }
    let joint = ext::HSD_ArchiveGetPublicAddress((*ptr).archive, cstr!("ToyDspBg_Top_joint")) as *mut HSD_Joint;
    if !joint.is_null() {
        (*ptr).gobj4 = ext::GObj_Create(9, 9, zero);
        let jobj = ext::HSD_JObjLoadJoint(joint);
        ext::HSD_GObjObject_80390A70((*ptr).gobj4, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink((*ptr).gobj4, Some(ext::cb::HSD_GObj_JObjCallback), 0x3C, zero as u32);
        ext::lb_8001204C(jobj, &raw mut (*ptr).jobj, _tyDisplay_804D5ABC.as_mut_ptr(), 1);
        return;
    }
    ext::OSReport(&format!("*** Can not Load Panel Label({})\n", "ToyDspBg_Top_joint"));
    hsd_assert_fail(0x43E);
}

/// Load the scene lights (and fog) into the camera record.
pub unsafe fn _tyDisplay_8031B328() {
    let ptr = _tyDisplay_804D6F1C;
    let scene = Toy_sbss_804D6ED4;
    let mut lobj: *mut HSD_LObj = core::ptr::null_mut();
    let temp3 = ptr;
    if (*temp3).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        panic!("tydisplay.c:1113");
    }
    let light_data = ext::HSD_ArchiveGetPublicAddress((*temp3).archive, cstr!("ScMenDisplay_scene_lights")) as *mut *mut LightList;
    if !light_data.is_null() {
        (*scene).x00 = ext::GObj_Create(2, 3, 0);
        lobj = Toy_LoadLObjList(light_data, core::ptr::null_mut());
        ext::HSD_GObjObject_80390A70((*scene).x00, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
        let temp = (*scene).x00;
        ext::GObj_SetupGXLink(temp, Some(ext::cb::HSD_GObj_LObjCallback), 0x34, 0);
    }
    if _tyDisplay_804D6F20 != 0 {
        ext::HSD_LObjSetColor(lobj, _tyDisplay_color_C8C8C8FF);
    }
    let fog_desc = ext::HSD_ArchiveGetPublicAddress((*temp3).archive, cstr!("ScMenDisplay_fog")) as *mut HSD_FogDesc;
    if !fog_desc.is_null() {
        (*scene).x08 = ext::GObj_Create(3, 4, 0);
        ext::HSD_GObjObject_80390A70((*scene).x08, ext::HSD_GObj_FogKind as u8, ext::HSD_FogLoadDesc(fog_desc) as *mut c_void);
        ext::GObj_SetupGXLink((*scene).x08, Some(Toy_80306930), 0x35, 0);
    }
}

/// `tyDisplay_SetGridSize` (static inline): choose the column / row counts for the grid.
unsafe fn tyDisplay_SetGridSize(cfg: *mut TyDspConfig, grid: *mut TyDspGrid) {
    match (*grid).x00 {
        0 | 1 => {
            let count = (*cfg).x08;
            let mut toggle = 0;
            (*cfg).x75 = 1;
            (*cfg).x76 = 1;
            loop {
                let a = (*cfg).x75 as i8 as i32;
                if !(a * ((*cfg).x76 as i8 as i32) < count) {
                    break;
                }
                toggle ^= 1;
                if toggle != 0 && ((*cfg).x75 as i8) < 0x14 {
                    (*cfg).x75 = (*cfg).x75.wrapping_add(1);
                } else {
                    (*cfg).x76 = (*cfg).x76.wrapping_add(1);
                }
            }
            (*cfg).x75 = (*cfg).x75.wrapping_add(1);
            (*cfg).x76 = (*cfg).x76.wrapping_add(1);
        }
        2 | 3 => {
            let count = (*cfg).x08;
            let mut toggle = 0;
            (*cfg).x75 = 1;
            (*cfg).x76 = 1;
            loop {
                let a = (*cfg).x75 as i8 as i32;
                if !(a * ((*cfg).x76 as i8 as i32) < count) {
                    break;
                }
                toggle ^= 1;
                if toggle != 0 && ((*cfg).x75 as i8) < 0x14 {
                    (*cfg).x75 = (*cfg).x75.wrapping_add(1);
                } else {
                    (*cfg).x76 = (*cfg).x76.wrapping_add(1);
                }
            }
        }
        _ => {}
    }
}

pub unsafe fn tyDisplay_Scene_OnEnter(_arg0: *mut c_void) {
    let mut sp18: *mut c_void = core::ptr::null_mut();
    _tyDisplay_804D6F10 = ext::HSD_MemAlloc((core::mem::size_of::<*mut HSD_JObj>() * _tyDisplay_804D6F10_len) as i32) as *mut *mut HSD_JObj;
    _tyDisplay_804D6F14 = ext::HSD_MemAlloc(core::mem::size_of::<TyDspGrid>() as i32) as *mut TyDspGrid;
    _tyDisplay_804D6F18 = ext::HSD_MemAlloc(core::mem::size_of::<TyDspConfig>() as i32) as *mut TyDspConfig;
    _tyDisplay_804D6F1C = ext::HSD_MemAlloc(core::mem::size_of::<TyDspBgData>() as i32) as *mut TyDspBgData;
    Toy_sbss_804D6ED4 = ext::HSD_MemAlloc(core::mem::size_of::<ToyCameraControl>() as i32) as *mut ToyCameraControl;
    let cfg = _tyDisplay_804D6F18;
    let data = _tyDisplay_804D6F1C;
    let grid = _tyDisplay_804D6F14;
    if Toy_80305C44() & 0x40 != 0 {
        (*grid).x00 = 2;
    } else if Toy_80305C44() & 0x20 != 0 {
        (*grid).x00 = 1;
    } else if Toy_80305C44() & 0x800 != 0 {
        (*grid).x00 = 3;
    } else {
        (*grid).x00 = 0;
    }
    _tyDisplay_804D6F20 = 0;
    _tyDisplay_804D6F2C = core::ptr::null_mut();
    _tyDisplay_804D6F24 = core::ptr::null_mut();
    if ext::DbLevel as i32 >= DbLKind_DebugRom as i32 && (Toy_80305C44() & 0x10) != 0 {
        _tyDisplay_804D6F20 = 1;
        (*cfg).x7C = 0;
    }
    Toy_8031263C();
    ext::memzero(_tyDisplay_804D6F1C as *mut c_void, core::mem::size_of::<TyDspBgData>() as i32);
    ext::memzero(_tyDisplay_804D6F18 as *mut c_void, core::mem::size_of::<TyDspConfig>() as i32);
    ext::memzero(Toy_sbss_804D6ED4 as *mut c_void, core::mem::size_of::<ToyCameraControl>() as i32);
    (*cfg).x08 = Toy_GetTrophyTotal();
    if _tyDisplay_804D6F20 != 0 {
        (*cfg).x08 = 1;
        _tyDisplay_8031C1D0();
    }
    tyDisplay_SetGridSize(cfg, grid);
    {
        let jp = ext::lbLang_IsSavedLanguageJP();
        let archive_name = if jp != 0 { cstr!("TyMnDisp.dat") } else { cstr!("TyMnDisp.usd") };
        (*data).archive = ext::lbArchive_80016DBC(archive_name, (&raw mut sp18) as *mut c_void, cstr!("ToyDspBg_Top_joint"), 0);
    }
    for i in 0..0x2Bi32 {
        let ret = _tyDisplay_8031BBF4(i as i8);
        (*data).archives[i as usize] = ext::lbArchive_LoadSymbols(ret, &[]);
    }
    (*data).x104 = 0;
    {
        let cfg2 = _tyDisplay_804D6F18;
        let cobj = ext::lb_80013B14(ext::HSD_ArchiveGetPublicAddress((*_tyDisplay_804D6F1C).archive, cstr!("ScMenDisplay_cam_int1_camera"))
            as *mut HSD_CameraDescPerspective);
        (*cfg2).x00 = ext::GObj_Create(1, 2, 0);
        ext::HSD_GObjObject_80390A70((*cfg2).x00, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax((*cfg2).x00, Some(Toy_803068E0), 0);
        (*(*cfg2).x00).gxlink_prios = 0x1230000000000000u64;
        if _tyDisplay_804D6F20 != 0 {
            ext::HSD_GObj_SetupProc((*cfg2).x00, Some(_tyDisplay_8031A94C), 0);
        } else {
            ext::HSD_GObj_SetupProc((*cfg2).x00, Some(_tyDisplay_8031A4EC), 0);
        }
        ext::HSD_GObj_80390CD4((*cfg2).x00);
    }
    _tyDisplay_8031B328();
    _tyDisplay_8031B1FC();
    if _tyDisplay_804D6F20 != 0 {
        ext::memzero(grid as *mut c_void, core::mem::size_of::<TyDspGrid>() as i32);
        (*grid).x08_min_z = -3.5;
        (*grid).x04_min_x = -3.5;
        (*grid).x10_max_z = 3.5;
        (*grid).x0C_max_x = 3.5;
        (*cfg).x78 = _tyDisplay_8031BC54((*cfg).x7C);
    } else {
        let m = (*grid).x00;
        match m {
            0 | 1 => _tyDisplay_80318CB4(m),
            2 => _tyDisplay_80319540(ext::HSD_Randi(2)),
            3 => _tyDisplay_80319994(ext::HSD_Randi(2)),
            _ => {}
        }
    }
    _tyDisplay_80319EF0();
}

/// Tear the scene down (only clears the pointers: the scene's gobjs are freed by the scene exit).
pub unsafe fn _tyDisplay_8031B850() {
    let anim = &raw mut Toy_804A2AA8;
    let ptr = _tyDisplay_804D6F1C;
    let pgobj = _tyDisplay_804D6F18;
    let scene = Toy_sbss_804D6ED4;
    if !(*ptr).archive.is_null() {
        (*ptr).archive = core::ptr::null_mut();
    }
    if !Toy_sbss_804D6EC8.is_null() {
        Toy_sbss_804D6EC8 = core::ptr::null_mut();
    }
    if !(*anim).gobj.is_null() {
        (*anim).gobj = core::ptr::null_mut();
        (*anim).jobj[1] = core::ptr::null_mut();
        (*anim).jobj[0] = core::ptr::null_mut();
    }
    if !(*ptr).gobj0.is_null() {
        (*ptr).gobj0 = core::ptr::null_mut();
    }
    if !(*ptr).gobj4.is_null() {
        (*ptr).gobj4 = core::ptr::null_mut();
    }
    if !(*scene).x00.is_null() {
        (*scene).x00 = core::ptr::null_mut();
    }
    if !(*scene).x04.is_null() {
        (*scene).x04 = core::ptr::null_mut();
    }
    if !(*scene).x08.is_null() {
        (*scene).x08 = core::ptr::null_mut();
    }
    let gobj = (*pgobj).x00;
    if !gobj.is_null() {
        ext::HSD_GObjProc_RemoveAllProcs(gobj);
        (*pgobj).x00 = core::ptr::null_mut();
    }
    if !_tyDisplay_804D6F2C.is_null() {
        if !_tyDisplay_804A2DD0.archive.is_null() {
            ext::lbArchive_80016EFC(_tyDisplay_804A2DD0.archive);
            _tyDisplay_804A2DD0.archive = core::ptr::null_mut();
        }
        ext::HSD_GObjFree(_tyDisplay_804D6F2C);
        _tyDisplay_804D6F2C = core::ptr::null_mut();
    }
    if !_tyDisplay_804D6F24.is_null() {
        ext::DevText_Remove(&raw mut _tyDisplay_804D6F24);
        _tyDisplay_804D6F24 = core::ptr::null_mut();
    }
}

pub unsafe fn tyDisplay_Scene_OnFrame() {
    let state = toy_mode_state();
    if (*state).x4 != 0 {
        _tyDisplay_8031B850();
        ext::gm_801A4B60();
    }
}

/// The display-table row for trophy `id` (US table first under the US language, else the JP one).
pub unsafe fn tyDisplay_8031B9DC(id: i32) -> *mut TyDspEntry {
    let mut found = false;
    let mut p: *mut TyDspEntry = core::ptr::null_mut();
    if ext::lbLang_IsSettingUS() != 0 {
        p = Toy_sbss_804D6EAC;
        loop {
            let val = (*p).x00;
            if val == -1 {
                break;
            }
            if val == id {
                found = true;
                break;
            }
            p = p.add(1);
        }
    }
    if !found {
        p = Toy_sbss_804D6EB0;
        loop {
            let val = (*p).x00;
            if val == -1 {
                break;
            }
            if val == id {
                found = true;
                break;
            }
            p = p.add(1);
        }
    }
    p
}

/// Store a debug-edited position component and refresh the debug window.
pub unsafe fn _tyDisplay_8031BA78(arg0: i32, arg1: i32, farg0: f32) {
    let data = tyDisplay_8031B9DC(arg0);
    match arg1 {
        1 => {}
        0 => (*data).x08 = farg0,
        2 => (*data).x0C = farg0,
        _ => {}
    }
    if !_tyDisplay_804D6F24.is_null() {
        ext::DevText_Erase(_tyDisplay_804D6F24);
        ext::DevText_SetCursorXY(_tyDisplay_804D6F24, 0, 0);
        let s = format!("X  {:3.2}\nZ  {:3.2}\0", (*data).x08 as f64, (*data).x0C as f64);
        ext::DevText_Print(_tyDisplay_804D6F24, s.as_ptr() as *mut i8);
    }
}

/// Joint symbol name of category `idx` (the C returns the address as an `s32`).
pub unsafe fn tyDisplay_8031BB34(idx: i8) -> *const i8 {
    table_name(&_tyDisplay_803B8988, idx)
}

/// Matanim symbol name of category `idx`.
pub unsafe fn tyDisplay_8031BB94(idx: i8) -> *const i8 {
    table_name(&_tyDisplay_803B8A34, idx)
}

/// Archive file name of category `arg0` (the C returns the address as an `s32`).
pub unsafe fn _tyDisplay_8031BBF4(arg0: i8) -> *const i8 {
    table_name(&_tyDisplay_803B8AE0, arg0)
}

/// Build the display gobj for trophy `arg0` (its category model on a root jobj plus the stand).
pub unsafe fn _tyDisplay_8031BC54(arg0: i32) -> *mut HSD_GObj {
    let id = arg0;
    let data = _tyDisplay_804D6F1C;
    let entry = tyDisplay_8031B9DC(id);
    let gobj = ext::GObj_Create(6, 7, 0);
    let root = ext::HSD_JObjAlloc();
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, root as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0x3C, 0);
    let c = (*entry).x04 as u32;
    let mut cat = c as u8;
    if (c as u8) as i8 == -1 {
        cat = 0;
    }
    let temp = _tyDisplay_803B8988[cat as i8 as usize].as_ptr() as *const i8;
    let child = ext::HSD_JObjLoadJoint(ext::HSD_ArchiveGetPublicAddress((*data).archives[c as usize], temp) as *mut HSD_Joint);
    ext::HSD_JObjAddChild(root, child);
    let c2 = (*entry).x04 as u32;
    let mut cat = c2 as u8;
    if (c2 as u8) as i8 == -1 {
        cat = 0;
    }
    let temp2 = _tyDisplay_803B8A34[cat as i8 as usize].as_ptr() as *const i8;
    Toy_80306A48(child, core::ptr::null(), temp2, core::ptr::null(), (*data).archives[c2 as usize], (*entry).x05 as i32);
    ext::HSD_JObjRemoveAnimAll(child);
    HSD_JObjSetTranslateX(&mut *child, (*entry).x08, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *child, (*entry).x0C, &mut Dirty);
    ext::HSD_JObjAddChild(
        root,
        ext::HSD_JObjLoadJoint(ext::HSD_ArchiveGetPublicAddress((*data).archives[41], _tyDisplay_803FF01C.as_ptr() as *const i8) as *mut HSD_Joint),
    );
    if !_tyDisplay_804D6F24.is_null() {
        ext::DevText_Erase(_tyDisplay_804D6F24);
        ext::DevText_SetCursorXY(_tyDisplay_804D6F24, 0, 0);
        let s = format!("X  {:3.2}\nZ  {:3.2}\0", (*entry).x08 as f64, (*entry).x0C as f64);
        ext::DevText_Print(_tyDisplay_804D6F24, s.as_ptr() as *mut i8);
        _tyDisplay_8031BF34(id);
    }
    gobj
}

/// Show trophy `arg0`'s 3D model (scaled 0.6, shifted right) next to the debug window.
pub unsafe fn _tyDisplay_8031BF34(arg0: i32) {
    let anim = &raw mut Toy_804A2AA8;
    if !_tyDisplay_804D6F2C.is_null() && !_tyDisplay_804A2DD0.archive.is_null() {
        ext::lbArchive_80016EFC(_tyDisplay_804A2DD0.archive);
        _tyDisplay_804A2DD0.archive = core::ptr::null_mut();
    }
    Toy_80308250(&raw mut _tyDisplay_804A2DD0, arg0 as i16, 0);
    _tyDisplay_804D6F2C = Toy_803087F4(&raw mut _tyDisplay_804A2DD0);
    ext::HSD_JObjClearFlagsAll((*anim).jobj[0], JOBJ_HIDDEN as u32);
    ext::HSD_JObjSetFlagsAll((*anim).jobj[1], JOBJ_HIDDEN as u32);
    ext::HSD_JObjClearFlagsAll((*anim).jobj[0], JOBJ_HIDDEN as u32);
    let jobj = (*_tyDisplay_804D6F2C).hsd_obj as *mut HSD_JObj;
    HSD_JObjSetScaleX(&mut *jobj, 0.6, &mut Dirty);
    HSD_JObjSetScaleY(&mut *jobj, 0.6, &mut Dirty);
    HSD_JObjSetScaleZ(&mut *jobj, 0.6, &mut Dirty);
    HSD_JObjAddTranslationX(&mut *jobj, 12.0, &mut Dirty);
}

/// Create the debug position window.
pub unsafe fn _tyDisplay_8031C1D0() {
    _tyDisplay_804D6F24 = ext::DevText_Create(1, 40, 40, 9, 3, _tyDisplay_devtext_buf.as_mut_ptr() as *mut i8);
    if !_tyDisplay_804D6F24.is_null() {
        let gobj = ext::DevText_GetGObj();
        ext::DevText_Show(gobj, _tyDisplay_804D6F24);
        ext::DevText_HideCursor(_tyDisplay_804D6F24);
        ext::DevText_80302AC0(_tyDisplay_804D6F24);
        ext::DevText_SetBGColor(_tyDisplay_804D6F24, _tyDisplay_804DE028);
        ext::DevText_SetScale(_tyDisplay_804D6F24, 12.0, 18.0);
        ext::DevText_Erase(_tyDisplay_804D6F24);
        ext::DevText_SetCursorXY(_tyDisplay_804D6F24, 0, 0);
        ext::DevText_StoreColorIndex(_tyDisplay_804D6F24, 0);
        ext::DevText_SetTextColor(_tyDisplay_804D6F24, _tyDisplay_804DE024);
        let s = format!("X  {:3.2}\nZ  {:3.2}\0", 0.0f64, 0.0f64);
        ext::DevText_Print(_tyDisplay_804D6F24, s.as_ptr() as *mut i8);
    }
}

pub unsafe fn tyDisplay_8031C2CC() {
    Toy_803124BC();
}

pub unsafe fn tyDisplay_8031C2EC() -> i32 {
    if Toy_80304B0C(5) {
        if ext::HSD_Randi(2) == 0 {
            return Toy_80305058(2, 5, 1, 65.0);
        }
    }
    Toy_80305058(2, 0, 1, 60.0)
}

/// Collect up to `max` other collected trophies of the same category and kind as `id`.
pub unsafe fn tyDisplay_8031C354(id: i32, buf: *mut i32, max: i32, kind: i32) -> i32 {
    let mut kind = kind;
    let mut buf = buf;
    if id == -1 {
        return 0;
    }
    let data = tyDisplay_8031B9DC(id);
    if kind == 99 {
        kind = Toy_803060BC(id, 6) as i32;
    }
    let mut count = 0;
    for i in 0..TY_TROPHY_COUNT {
        if i == id {
            continue;
        }
        if !Toy_80304CC8(i) {
            continue;
        }
        if !Toy_803049F4(i) {
            continue;
        }
        let other = tyDisplay_8031B9DC(i);
        let val = Toy_803060BC(i, 6) as i32;
        if (*other).x04 != (*data).x04 {
            continue;
        }
        if val != kind {
            continue;
        }
        count += 1;
        *buf = i;
        buf = buf.add(1);
        if count >= max {
            break;
        }
    }
    count
}

/// Load the display archives for trophy `arg0` (returns 1 if its category archive was already
/// loaded, -1 for no trophy).
pub unsafe fn tyDisplay_8031C454(arg0: i32) -> i32 {
    // Retail reads the archive file names past the two symbol tables laid out back to back; the
    // port links the tables separately, so the file-name table is named directly.
    let tables = &_tyDisplay_803B8AE0;
    let mut result = 0;
    let arch_arr = &raw mut _tyDisplay_804A2DE8 as *mut *mut HSD_Archive;
    Toy_803124BC();
    if arg0 == -1 {
        return -1;
    }
    let entry = tyDisplay_8031B9DC(arg0);
    let mut idx = (*entry).x04;
    if (*arch_arr.add((*entry).x04 as usize)).is_null() {
        idx = (*entry).x04;
        if idx as i8 == -1 {
            idx = 0;
        }
        *arch_arr.add((*entry).x04 as usize) = ext::lbArchive_LoadSymbols(tables[idx as i8 as usize].as_ptr() as *const i8, &[]);
    } else {
        result = 1;
    }
    if (*arch_arr.add(42)).is_null() {
        *arch_arr.add(42) = ext::lbArchive_LoadSymbols(tables[42].as_ptr() as *const i8, &[]);
    }
    if (*arch_arr.add(41)).is_null() {
        *arch_arr.add(41) = ext::lbArchive_LoadSymbols(tables[41].as_ptr() as *const i8, &[]);
    }
    result
}

/// Build the standalone display model for trophy `arg0` (null if its archive is not loaded).
pub unsafe fn tyDisplay_8031C5E4(arg0: i32) -> *mut HSD_JObj {
    let archives = &raw mut _tyDisplay_804A2DE8 as *mut *mut HSD_Archive;
    let entry = tyDisplay_8031B9DC(arg0);
    if (*archives.add((*entry).x04 as usize)).is_null() {
        return core::ptr::null_mut();
    }
    let root = ext::HSD_JObjAlloc();
    let child;
    {
        let c = (*entry).x04;
        let mut cat = c;
        if c as i8 == -1 {
            cat = 0;
        }
        child = ext::HSD_JObjLoadJoint(
            ext::HSD_ArchiveGetPublicAddress(*archives.add(c as usize), _tyDisplay_803B8988[cat as i8 as usize].as_ptr() as *const i8)
                as *mut HSD_Joint,
        );
        ext::HSD_JObjAddChild(root, child);
    }
    {
        let c = (*entry).x04;
        let mut cat = c;
        // the tables are not adjacent in the port
        let matanim = &_tyDisplay_803B8A34;
        if c as i8 == -1 {
            cat = 0;
        }
        let temp = matanim[cat as i8 as usize].as_ptr() as *const i8;
        Toy_80306A48(child, core::ptr::null(), temp, core::ptr::null(), *archives.add(c as usize), (*entry).x05 as i32);
    }
    ext::HSD_JObjRemoveAnimAll(child);
    HSD_JObjSetTranslateX(&mut *child, (*entry).x08, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *child, (*entry).x0C, &mut Dirty);
    ext::HSD_JObjAddChild(
        root,
        ext::HSD_JObjLoadJoint(
            ext::HSD_ArchiveGetPublicAddress(*archives.add(42), _tyDisplay_803B8988[42].as_ptr() as *const i8) as *mut HSD_Joint,
        ),
    );
    ext::HSD_JObjAddChild(
        root,
        ext::HSD_JObjLoadJoint(
            ext::HSD_ArchiveGetPublicAddress(*archives.add(41), _tyDisplay_803B8988[41].as_ptr() as *const i8) as *mut HSD_Joint,
        ),
    );
    root
}

pub unsafe fn tyDisplay_8031C8B8() {
    for i in 0..0x2Busize {
        _tyDisplay_804A2DE8[i] = core::ptr::null_mut();
    }
}
