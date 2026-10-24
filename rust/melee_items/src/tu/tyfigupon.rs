//! Native port of src/melee/ty/tyfigupon.c: the Trophy Lottery ("Figure Pon") scene. The coin
//! machine (panel, background, lever, digit and bet objects), the bet / lever / coin-drop
//! state machine, awarding the trophy and its popup, the scene camera and the Atlas readback.
//!
//! Every function of the TU is here. Callees in other TUs go through `ext` (an unset hook
//! panics "TODO: <name> is not ported"; names are the C names); the trophy functions of
//! `toy.c` are called directly from [`super::toy`].
//!
//! Native-layout notes: `Toy_sbss_804D6ED4` is accessed as the `ToyCameraControl` that owns it
//! (the C's `TyFiguponED4` view has the same slots: `x0` = `x00`, `x4` = `x04`, `xC` = `archive`);
//! the integer slots the C fills with addresses (`un_804D6EF4_t.x00/x08/x0C`, `TyFiguponData.x10`)
//! are `usize` in melee_types; `_tyFigupon_804D6EF8` is a `ToyListEntry` (what `Toy_80308250` /
//! `Toy_803087F4` read it as); allocation sizes are `size_of` of the native records. The two
//! procs `_tyFigupon_80315574` / `_tyFigupon_803155C8` are `void(void)` in the C and registered
//! through a cast to a gobj proc; here they take (and ignore) the gobj.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ty/tyfigupon.c`): the coin's start
//! jitter `3.0f*HSD_Randf() + pos` (x2) and the stick remap `-(0.2f*sign - v)/0.8f`; each is
//! marked `// fused`.
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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_RemoveUserData(a0: *mut HSD_GObj);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32);
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void;
    fn HSD_CObjEndCurrent();
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32);
    fn HSD_CObjGetEyeDistance(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetEyeVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32;
    fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjInit(a0: *mut HSD_CObj, a1: *mut HSD_CObjDesc);
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32;
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObjProc_RemoveAllProcs(a0: *mut HSD_GObj);
    fn HSD_GObjProc_RemoveProc(a0: *mut HSD_GObjProc);
    fn HSD_GObj_80390CD4(a0: *mut HSD_GObj);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn HSD_PadRenewStatus();
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text;
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32;
    fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8);
    fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32);
    fn PSMTXMultVec(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec);
    fn PSMTXRotAxisRad(a0: *mut c_void, a1: *mut Vec, a2: f32);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECScale(a0: *mut Vec, a1: *mut Vec, a2: f32);
    fn PSVECSubtract(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn gm_801623D8() -> i32;
    fn gm_801623FC(a0: i32);
    fn gm_801677F0() -> u8;
    fn gm_801678F8(a0: i32, a1: i32, a2: i32);
    fn gm_801A4B60();
    fn lbArchive_80016EFC(a0: *mut HSD_Archive);
    fn lbAudioAx_80023694() -> i32;
    fn lbAudioAx_800236DC() -> i32;
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32;
    fn lbAudioAx_80023870(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
    fn lbAudioAx_80023F28(a0: i32) -> i32;
    fn lbAudioAx_80024030(a0: i32);
    fn lbCardGame_SaveChanges();
    fn lbLang_IsSavedLanguageJP() -> i32;
    fn lbLang_IsSavedLanguageUS() -> i32;
    fn lbVector_Normalize(a0: *mut Vec) -> f32;
    fn lb_8001204C(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, a2: *mut u16, a3: i32) -> i32;
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj;
    fn lbArchive_LoadSymbols(a0: *const i8, a1: &[*mut c_void]) -> *mut HSD_Archive;
    fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut HSD_Archive;
    fn DevText_Printf(a0: *mut DevText, a1: &str);
    fn memzero(a0: *mut c_void, a1: i32);
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
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT


use super::toy::{
    c_str, cstr, Dirty, Toy_803048C0, Toy_80304B0C, Toy_80304B94, Toy_80304CC8, Toy_80305058, Toy_80305B88, Toy_80305C44,
    Toy_80305DB0, Toy_803060BC, Toy_803068E0, Toy_80306B18, Toy_80306BB8, Toy_80306D14, Toy_80306D70, Toy_80308250, Toy_80308328,
    Toy_803083D8, Toy_803087F4, Toy_8031234C, Toy_8031263C, Toy_LoadLObjList, Toy_RemoveUserData, Toy_SetUnlockState,
    Toy_804A284C, Toy_804A2AA8, Toy_sbss_804D6EC8, Toy_sbss_804D6ED4, Toy_str_ScMenFigure_cam_int1_camera,
};

/// `HSD_ASSERT` / `OSPanic` failure.
#[cold]
fn panic_at(line: u32) -> ! {
    panic!("tyfigupon.c:{line}")
}

#[inline]
unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
#[inline]
unsafe fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

#[inline]
unsafe fn toy_mode_state() -> *mut TyModeState {
    &raw mut Toy_804A284C as *mut TyModeState
}

/* 4D6EF0 */
static mut _tyFigupon_804D6EF0: *mut TyFiguponData = core::ptr::null_mut();
/* 4D6EF4 */
static mut _tyFigupon_804D6EF4: *mut un_804D6EF4_t = core::ptr::null_mut();
/* 4D6EF8: allocated as a 0x18-byte block and used as a `ToyListEntry` (its +0x14 slot is the archive). */
static mut _tyFigupon_804D6EF8: *mut ToyListEntry = core::ptr::null_mut();
/* 4D6EFC */
static mut _tyFigupon_804D6EFC: i32 = 0;
/* 4D6F00 */
static mut _tyFigupon_804D6F00: i32 = 0;
/* 4D6F04 */
static mut _tyFigupon_804D6F04: *mut HSD_CObjDesc = core::ptr::null_mut();
/* 4D6F08 */
static mut _tyFigupon_804D6F08: *mut HSD_CameraDescPerspective = core::ptr::null_mut();

/* 3B8968 */
static _tyFigupon_803B8968: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
/* 3FEB88 */
static mut _tyFigupon_803FEB88: [u16; 13] = [4, 2, 3, 5, 6, 7, 8, 9, 13, 10, 11, 12, 14];
/* 4D5AA0 */
static mut _tyFigupon_804D5AA0: [u16; 2] = [12, 0];

/* 3FEED4 */
static mut _tyFigupon_803FEED4: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 50.0 }, robjdesc: core::ptr::null_mut() };
/* 3FEEE8 */
static mut _tyFigupon_803FEEE8: HSD_WObjDesc = HSD_WObjDesc { class_name: core::ptr::null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: core::ptr::null_mut() };
/* 3FEEFC */
static mut _tyFigupon_803FEEFC: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: core::ptr::null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut _tyFigupon_803FEED4,
    interest: &raw mut _tyFigupon_803FEEE8,
    roll: 0.0,
    up_vector: core::ptr::null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 40.0,
    aspect: 913.0f32 / 750.0f32,
};

#[inline]
unsafe fn ef4() -> *mut un_804D6EF4_t {
    _tyFigupon_804D6EF4
}

#[inline]
unsafe fn ud_of(gobj: *mut HSD_GObj) -> *mut TyFiguponUD {
    HSD_GObjGetUserData(&*gobj) as *mut TyFiguponUD
}

/// Attach the (optional) anim / matanim / shapeanim symbols of the panel archive to `jobj`.
pub unsafe fn _tyFigupon_80314AA8(jobj: *mut HSD_JObj, anim_str: *const i8, matanim_str: *const i8, shapeanim_str: *const i8) {
    let data = _tyFigupon_804D6EF4;
    let ajoint = if !anim_str.is_null() { ext::HSD_ArchiveGetPublicAddress((*data).archive, anim_str) } else { core::ptr::null_mut() };
    let majoint = if !matanim_str.is_null() { ext::HSD_ArchiveGetPublicAddress((*data).archive, matanim_str) } else { core::ptr::null_mut() };
    let sajoint = if !shapeanim_str.is_null() { ext::HSD_ArchiveGetPublicAddress((*data).archive, shapeanim_str) } else { core::ptr::null_mut() };
    ext::HSD_JObjAddAnimAll(jobj, ajoint as *mut HSD_AnimJoint, majoint as *mut HSD_MatAnimJoint, sajoint as *mut HSD_ShapeAnimJoint);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
}

/// Number of obtainable trophies the player holds of the lottery's categories.
pub unsafe fn _tyFigupon_80314B54() -> i32 {
    let mut count = 0;
    for i in 0..TY_TROPHY_COUNT {
        if Toy_80304CC8(i) && Toy_803048C0(i) != 0 {
            let result = Toy_803060BC(i, 6) as i32;
            if result != 8 && result as u32 > 1 {
                count += 1;
            }
        }
    }
    count
}

/// Scene camera render callback: clear to dark blue and draw the lights pass.
pub unsafe extern "C" fn _tyFigupon_80314BE4(gobj: *mut HSD_GObj, _unused: i32) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        ext::HSD_SetEraseColor(0x19, 0x19, 0x33, 0xFF);
        ext::HSD_CObjEraseScreen((*gobj).hsd_obj as *mut HSD_CObj, 1, 0, 0);
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_FogSet(core::ptr::null_mut());
        ext::HSD_CObjEndCurrent();
    }
}

/// A falling coin: drops toward `jobjs[2]`, then is freed with a landing sound.
pub unsafe extern "C" fn _tyFigupon_80314C5C(gobj: *mut HSD_GObj) {
    let tp1 = (*gobj).user_data as *mut Toy;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let temp_r29 = _tyFigupon_804D6EF4;
    if !tp1.is_null() {
        if (*tp1).x8 % 30 == 0 {
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
        }
        ext::HSD_JObjAnimAll(jobj);
        let old = (*tp1).x8;
        (*tp1).x8 = old.wrapping_sub(1);
        if old != 0 {
            (*tp1).translate.y += (*tp1).offset.y;
            if HSD_JObjGetTranslationX(&*jobj) > HSD_JObjGetTranslationX(&*(*temp_r29).jobjs[2]) {
                HSD_JObjAddTranslationX(&mut *jobj, (*tp1).translate.x, &mut Dirty);
            }
            if HSD_JObjGetTranslationZ(&*jobj) > HSD_JObjGetTranslationZ(&*(*temp_r29).jobjs[2]) {
                HSD_JObjAddTranslationZ(&mut *jobj, (*tp1).translate.z, &mut Dirty);
            }
            HSD_JObjAddTranslationY(&mut *jobj, (*tp1).translate.y, &mut Dirty);
        } else {
            ext::lbAudioAx_800237A8(146, 127, 64);
            (*tp1).x8 = 0;
            ext::HSD_GObjFree(gobj);
        }
    } else {
        let tp = ext::HSD_MemAlloc(core::mem::size_of::<Toy>() as i32) as *mut Toy;
        if !tp.is_null() {
            ext::GObj_InitUserData(gobj, 0, Some(Toy_RemoveUserData), tp as *mut c_void);
        }
        ext::HSD_JObjReqAnimAll(jobj, ext::HSD_Randi(15) as f32);
        ext::HSD_AObjSetRate((*(*jobj).child).aobj, 1.0);
        {
            let mut temp_f30 = HSD_JObjGetTranslationX(&*(*temp_r29).jobjs[1]);
            let mut temp_f29 = HSD_JObjGetTranslationY(&*(*temp_r29).jobjs[1]);
            let temp_f31 = HSD_JObjGetTranslationZ(&*(*temp_r29).jobjs[1]);
            let temp_f28 = HSD_JObjGetTranslationX(&*(*temp_r29).jobjs[2]);
            let temp_f27 = HSD_JObjGetTranslationY(&*(*temp_r29).jobjs[2]);
            let temp_f26 = HSD_JObjGetTranslationZ(&*(*temp_r29).jobjs[2]);
            temp_f30 = fmadds(3.0, ext::HSD_Randf(), temp_f30); // fused
            temp_f29 = fmadds(3.0, ext::HSD_Randf(), temp_f29); // fused
            {
                let d = temp_f28 - temp_f30;
                let var_f3 = if d < 0.0 { -d } else { d };
                (*tp).translate.x = -0.74f32 * (var_f3 / 30.0);
                let dz = temp_f26 - temp_f31;
                let adz = if dz < 0.0 { -dz } else { dz };
                (*tp).translate.z = -0.72f32 * (adz / 30.0);
                (*tp).translate.y = 0.4;
                (*tp).offset.y = -0.026666667;
                let dy = temp_f27 - temp_f29;
                let ady = if dy < 0.0 { -dy } else { dy };
                (*tp).x8 = ((ady as i32) as f32 + 35.0f32) as i32;
            }
            (*temp_r29).x56 = (*tp).x8 as i16;
            HSD_JObjSetTranslateX(&mut *jobj, temp_f30, &mut Dirty);
            HSD_JObjSetTranslateY(&mut *jobj, temp_f29, &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *jobj, temp_f31, &mut Dirty);
        }
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        ext::lbAudioAx_80023870(147, 127, 64, 26);
    }
}

/// Lever / pull animation proc.
pub unsafe extern "C" fn _tyFigupon_803152BC(arg0: *mut HSD_GObj) {
    let temp_r30 = ud_of(arg0);
    let temp_r31 = _tyFigupon_804D6EF4;
    if !temp_r30.is_null() {
        let temp_r3 = (*temp_r30).x8;
        (*temp_r30).x8 = temp_r3.wrapping_sub(1);
        if temp_r3 != 0 {
            ext::HSD_JObjAnimAll((*temp_r31).jobjs[0]);
            if ((*temp_r30).x8 % 30) == 0 {
                ext::HSD_JObjReqAnimAll((*temp_r31).jobjs[0], 0.0);
            }
        } else {
            (*temp_r30).x8 = 0;
            ext::GObj_RemoveUserData(arg0);
            ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        }
    } else {
        let temp_r3_2 = ext::HSD_MemAlloc(core::mem::size_of::<TyFiguponUD>() as i32) as *mut TyFiguponUD;
        if !temp_r3_2.is_null() {
            ext::GObj_InitUserData(arg0, 0, Some(Toy_RemoveUserData), temp_r3_2 as *mut c_void);
        }
        ext::HSD_JObjReqAnimAll((*temp_r31).jobjs[0], 0.0);
        (*temp_r3_2).x8 = 0x1E;
        (*temp_r3_2).x8 = (*temp_r3_2).x8 + ((((*temp_r31).x5E as i32 * 2) / 30) * 0x1E);
    }
}

/// Show `arg0` in `arg2` digit jobjs starting at `jobjs[arg1]` (`arg3 == 1`: the counter-up
/// frame mapping). `arg4` was an address the C never reads.
pub unsafe fn _tyFigupon_803153EC(arg0: i32, arg1: i32, arg2: i32, arg3: i32, _arg4: i32) {
    let mut digits_s: [i32; 4] = [0; 4];
    let temp_r30 = _tyFigupon_804D6EF4;
    let mut num = arg0;
    let mut count: usize = 0;
    loop {
        // The C writes past `digits` for a five-digit number; the extra digits are dropped.
        if count < 4 {
            digits_s[count] = num % 10;
        }
        count += 1;
        num /= 10;
        if !(num > 0) {
            break;
        }
    }
    for i in 0..arg2 {
        let digit = digits_s[i as usize];
        let target = (*temp_r30).jobjs[(arg1 + i) as usize];
        if digit != 0 {
            if arg3 == 1 {
                ext::HSD_JObjReqAnimAll(target, (0x32 - (digit * 5)) as f32);
            } else {
                ext::HSD_JObjReqAnimAll(target, digit as f32);
            }
        } else {
            ext::HSD_JObjReqAnimAll(target, 0.0);
        }
        let jobj_ptr = &raw mut (*temp_r30).jobjs[(arg1 + i) as usize];
        ext::HSD_AObjSetRate((**jobj_ptr).aobj, 1.0);
        ext::HSD_JObjAnimAll(*jobj_ptr);
    }
}

/// The "NEW!" banner countdown (hidden again when it hits 0).
pub unsafe extern "C" fn _tyFigupon_80315574(_gobj: *mut HSD_GObj) {
    let data = _tyFigupon_804D6EF0;
    if (*data).x24 == 0 {
        (*(*data).x18).hidden = 1;
        (*data).x10 = 0;
        ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
    } else {
        (*data).x24 = (*data).x24 - 1;
    }
}

/// `tyFigupon_GetTotalCount` (static inline).
unsafe fn tyFigupon_GetTotalCount() -> i32 {
    let mut total = 0;
    for i in 0..9 {
        if i != 8 && i as u32 > 1 && Toy_80304B0C(i) {
            total += Toy_80304B94(i);
        }
    }
    total
}

/// `setupPercentDisplay` (static inline): the "% collected" digits.
unsafe fn setupPercentDisplay(ef4_arg: *mut un_804D6EF4_t) {
    let mut sc: i32 = (*ef4_arg).x5E as i32;
    let ef4_3 = _tyFigupon_804D6EF4;
    let mut pct: f32;
    let fval: f32 = ((*ef4_3).x54 as i32 + _tyFigupon_80314B54()) as f32;
    if sc != 0 {
        sc -= 1;
    }
    if (*ef4_3).x54 == 0 {
        pct = 0.0;
    } else {
        pct = ((*ef4_3).x54 as f32 / fval) + ((sc * 5) as f32 / 100.0);
    }
    if pct >= 1.0 {
        pct = 999.0;
    } else {
        pct *= 1000.0;
    }
    _tyFigupon_803153EC((pct as i32) as u32 as i32, 9, 3, 2, 0);
}

/// Refresh the bet arrow animation for the current bet count (`default_frame` is the C's
/// `anim_frame` initialiser for this call site).
unsafe fn bet_anim(default_frame: i32) {
    let ef4_2 = _tyFigupon_804D6EF4;
    let mut anim_frame = default_frame;
    if (*ef4_2).x5E == 0x14 {
        anim_frame = 3;
    }
    if (*ef4_2).x5E <= 1 {
        anim_frame = 4;
    }
    ext::HSD_JObjReqAnimAll((*ef4_2).jobjs[8], anim_frame as f32);
    ext::HSD_AObjSetRate((*(*ef4_2).jobjs[8]).aobj, 1.0);
    ext::HSD_JObjAnimAll((*ef4_2).jobjs[8]);
}

/// `setupBetAnim` (static inline): reset the bet to 1.
unsafe fn setupBetAnim(ef4_arg: *mut un_804D6EF4_t) {
    (*ef4_arg).x5E = 1;
    _tyFigupon_803153EC((*ef4_arg).x5E as i32, 6, 2, 0, 0);
    bet_anim(1);
}

/// The lottery state machine (one step per frame; `ef4->x5C` is the phase).
pub unsafe extern "C" fn _tyFigupon_803155C8(_gobj: *mut HSD_GObj) {
    let aa8 = &raw mut Toy_804A2AA8;
    let ef4 = _tyFigupon_804D6EF4;
    let data = _tyFigupon_804D6EF0;
    let jobj = (*((*ef4).x08 as *mut HSD_GObj)).hsd_obj as *mut HSD_JObj;
    match (*ef4).x5C {
        2 => {
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                (*data).x29 = 1;
                (*ef4).x58 = 0x14;
                ext::HSD_JObjReqAnimAll(jobj, 0.0);
                (*ef4).x5C = 3;
                ext::gm_801678F8(ext::gm_801677F0() as i32, 0x1E, 0);
                return;
            }
        }
        3 => {
            if (*ef4).x54 != 0 {
                ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 0.0);
            } else {
                ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 1.0);
            }
            ext::HSD_JObjAnimAll(jobj);
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                ext::lbAudioAx_800237A8(0xA8, 0x7F, 0x40);
                (*ef4).x58 = 0xA;
                (*ef4).x58 += 0xA;
                (*ef4).x5C = 4;
                return;
            }
        }
        4 => {
            if (*ef4).x54 != 0 {
                ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 0.0);
            } else {
                ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 1.0);
            }
            ext::HSD_JObjAnimAll(jobj);
            if (*ef4).x58 < 0xE {
                ext::HSD_JObjReqAnimAll(jobj, 30.0);
            }
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                (*ef4).x58 = 0xE;
                (*ef4).x5C = 5;
                return;
            }
        }
        5 => {
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                let sc = (*ef4).x5D as i32;
                if sc >= 3 {
                    (*ef4).x58 = (1.62f32 * sc as f32) as i16;
                    (*ef4).x58 += 0xE;
                    ext::HSD_JObjReqAnimAll(jobj, 50.0);
                    ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 0.0);
                    ext::lbAudioAx_80023870(0xA9, 0x7F, 0x40, 0x8A);
                } else {
                    (*ef4).x58 = 1;
                }
                (*ef4).x5C = 6;
                return;
            }
        }
        6 => {
            if (*ef4).x5D >= 3 {
                if (*ef4).x54 != 0 {
                    ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 0.0);
                } else {
                    ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 1.0);
                }
                ext::HSD_JObjAnimAll(jobj);
                if ((*ef4).x58 % 40) == 0 {
                    ext::HSD_JObjReqAnimAll(jobj, 50.0);
                    ext::lbAudioAx_80023870(0xA9, 0x7F, 0x40, 0x8A);
                }
                if (*ef4).x58 < 0xE {
                    ext::HSD_JObjReqAnimAll(jobj, 50.0);
                }
            }
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                if (*data).x29 as i8 != 0 {
                    let new_var = _tyFigupon_80314B54();
                    let mut bet_count = (*ef4).x5D as i32;
                    let fval = ((*ef4).x54 as i32 + new_var) as f32;
                    ext::lbAudioAx_80023694();
                    if bet_count != 0 {
                        bet_count -= 1;
                    }
                    let mut pct = 100.0f32 * (((*ef4).x54 as f32 / fval) + ((bet_count * 5) as f32 / 100.0));
                    if pct >= 100.0 {
                        pct = 99.9;
                    }
                    (*data).x20 = Toy_80305058(0x63, 3, 0, pct);
                    _tyFigupon_80316420((*data).x20 as i16 as i32);
                    {
                        let inv = _tyFigupon_80314B54();
                        (*ef4).x54 = (tyFigupon_GetTotalCount() - inv) as i16;
                    }
                    (*data).x29 = 0;
                    (*ef4).x58 = 0x78;
                    (*ef4).x5C = 8;
                    return;
                }
                (*ef4).x5C = 9;
                return;
            }
        }
        8 => {
            (*ef4).x58 -= 1;
            if (*ef4).x58 == 0 {
                if !(*_tyFigupon_804D6EF8).archive.is_null() {
                    ext::lbArchive_80016EFC((*_tyFigupon_804D6EF8).archive);
                    (*_tyFigupon_804D6EF8).archive = core::ptr::null_mut();
                }
                if !(*aa8).gobj.is_null() {
                    ext::HSD_GObjFree((*aa8).gobj);
                    (*aa8).gobj = core::ptr::null_mut();
                    (*aa8).jobj[1] = core::ptr::null_mut();
                    (*aa8).jobj[0] = core::ptr::null_mut();
                }
                ext::HSD_SisLib_803A6368((*data).x14, 0);
                Toy_803083D8((*ef4).jobjs[0xC], 0x3E7);
                if (*ef4).x54 == 0 {
                    ext::HSD_JObjReqAnimAll((*ef4).jobjs[0xE], 1.0);
                    ext::HSD_JObjAnimAll(jobj);
                }
                (*ef4).x5C = 9;
                return;
            }
        }
        9 => {
            if (ext::gm_801623D8() as u32 / 10) != 0 {
                setupBetAnim(ef4);
            }
            setupPercentDisplay(ef4);
            (*ef4).x58 = 0;
            (*ef4).x56 = 0;
            ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
            (*ef4).x5C = 0;
        }
        _ => {}
    }
}

/// `tyFigupon_GetCoinCount` (static inline).
#[inline]
unsafe fn tyFigupon_GetCoinCount() -> u32 {
    ext::gm_801623D8() as u32 / 10
}

/// `tyFigupon_CreateCoin` (static inline).
unsafe fn tyFigupon_CreateCoin(ef4_arg: *mut un_804D6EF4_t) {
    let gobj = ext::GObj_Create(9, 9, 0);
    let jobj = ext::HSD_JObjLoadJoint(ext::HSD_ArchiveGetPublicAddress((*ef4_arg).archive, cstr!("ToyFigurePonCoin_Top_joint")) as *mut HSD_Joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0x3C, 0);
    _tyFigupon_80314AA8(
        jobj,
        cstr!("ToyFigurePonCoin_Top_animjoint"),
        cstr!("ToyFigurePonCoin_Top_matanim_joint"),
        cstr!("ToyFigurePonCoin_Top_shapeanim_joint"),
    );
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::HSD_GObj_SetupProc(gobj, Some(_tyFigupon_80314C5C), 0);
    ext::HSD_GObj_80390CD4(gobj);
}

/// `tyFigupon_StoreDigits` (static inline): decimal digits, least significant first.
unsafe fn tyFigupon_StoreDigits(digits: *mut i32, value: i32) {
    let mut value = value;
    let mut i = 0isize;
    loop {
        *digits.offset(i) = value % 10;
        i += 1;
        value /= 10;
        if !(value > 0) {
            break;
        }
    }
}

/// `tyFigupon_FinishCoinDrop` (static inline).
unsafe fn tyFigupon_FinishCoinDrop(gobj: *mut HSD_GObj, ud: *mut TyFiguponUD, ef4_arg: *mut un_804D6EF4_t) {
    (*ud).x8 = 0;
    (*ef4_arg).x5E = 0;
    _tyFigupon_803153EC((ext::gm_801623D8() as u32 / 10) as i32, 3, 3, 1, 0);
    _tyFigupon_803153EC((*ef4_arg).x5E as u32 as i32, 6, 2, 0, 0);
    bet_anim(0);
    ext::GObj_RemoveUserData(gobj);
    (*ef4_arg).x58 = (*ef4_arg).x56;
    (*ef4_arg).x58 += 0xE;
    (*ef4_arg).x5C = 2;
    ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
}

/// Coin-insert animation proc: spends one coin (10 balance) per two frames of the bet.
pub unsafe extern "C" fn _tyFigupon_80315C44(arg0: *mut HSD_GObj) {
    let mut ef4_l = _tyFigupon_804D6EF4;
    let mut ud = ud_of(arg0);
    if !ud.is_null() {
        if (*ud).x8 != 0 {
            if (*ud).x10 == 0 && (*ud).x14 == 0 && (*ud).x18 != 0 {
                ext::HSD_JObjAnimAll((*ef4_l).jobjs[5]);
            }
            if (*ud).x10 == 0 {
                if (*ud).x14 == 0 {
                    ext::HSD_JObjReqAnimAll((*ef4_l).jobjs[4], 0.0);
                }
                ext::HSD_JObjAnimAll((*ef4_l).jobjs[4]);
            }
            if (*ud).x10 == 0 {
                ext::HSD_JObjReqAnimAll((*ef4_l).jobjs[3], 0.0);
            }
            ext::HSD_JObjAnimAll((*ef4_l).jobjs[3]);
            (*ud).x8 = (*ud).x8 - 1;
            if (*ud).x8 % 2 == 0 {
                tyFigupon_CreateCoin(_tyFigupon_804D6EF4);
                ext::gm_801623FC(ext::gm_801623D8() - 0xA);
                (*ef4_l).x5E = (*ef4_l).x5E - 1;
                let total = tyFigupon_GetCoinCount();
                (*ud).x18 = 0;
                (*ud).x14 = 0;
                (*ud).x10 = 0;
                tyFigupon_StoreDigits(&raw mut (*ud).x10, total as i32);
                let count = (*ef4_l).x5E as i32;
                (*ud).x30 = 0;
                (*ud).x2C = 0;
                (*ud).x28 = 0;
                tyFigupon_StoreDigits(&raw mut (*ud).x28, count);
                _tyFigupon_803153EC((ext::gm_801623D8() as u32 / 10) as i32, 3, 3, 1, 0);
                _tyFigupon_803153EC((*ef4_l).x5E as u32 as i32, 6, 2, 0, 0);
                ef4_l = _tyFigupon_804D6EF4;
                bet_anim(2);
                ext::gm_801678F8(ext::gm_801677F0() as i32, 0x10, 0);
            }
        } else {
            tyFigupon_FinishCoinDrop(arg0, ud, ef4_l);
        }
    } else {
        ud = ext::HSD_MemAlloc(core::mem::size_of::<TyFiguponUD>() as i32) as *mut TyFiguponUD;
        if !ud.is_null() {
            ext::GObj_InitUserData(arg0, 0, Some(Toy_RemoveUserData), ud as *mut c_void);
            let total = tyFigupon_GetCoinCount();
            (*ud).x18 = 0;
            (*ud).x14 = 0;
            (*ud).x10 = 0;
            tyFigupon_StoreDigits(&raw mut (*ud).x10, total as i32);
            _tyFigupon_803153EC((ext::gm_801623D8() as u32 / 10) as i32, 3, 3, 1, 0);
            for j in [3usize, 4, 5] {
                let tobj_aobj = (*(*(*(*(*(*ef4_l).jobjs[j]).child).u.dobj).mobj).tobj).aobj;
                ext::HSD_AObjSetRate(tobj_aobj, 2.0);
            }
            (*ud).x8 = (*ef4_l).x5E as i32 * 2;
        }
    }
}

/// The won trophy falls into view and lands.
pub unsafe extern "C" fn _tyFigupon_80316170(arg0: *mut HSD_GObj) {
    let aa8 = &raw mut Toy_804A2AA8;
    let y = HSD_JObjGetTranslationY(&*((*arg0).hsd_obj as *mut HSD_JObj));
    let mut ud = ud_of(arg0);
    if !ud.is_null() {
        if y + (*ud).x44 <= -6.2 {
            if Toy_803048C0((*aa8).xC as i32) == 1 {
                Toy_80306D14();
            }
            (*toy_mode_state()).x0 = 1;
            ext::lbAudioAx_800237A8(0xA7, 0x7F, 0x40);
            HSD_JObjSetTranslateY(&mut *((*arg0).hsd_obj as *mut HSD_JObj), -7.2, &mut Dirty);
            ext::gm_801678F8(ext::gm_801677F0() as i32, 0xC, 0);
            ext::GObj_RemoveUserData(arg0);
            ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
            return;
        }
        (*ud).x44 -= 0.6;
        HSD_JObjAddTranslationY(&mut *((*arg0).hsd_obj as *mut HSD_JObj), (*ud).x44, &mut Dirty);
    } else {
        ud = ext::HSD_MemAlloc(core::mem::size_of::<TyFiguponUD>() as i32) as *mut TyFiguponUD;
        if !ud.is_null() {
            ext::GObj_InitUserData(arg0, 0, Some(Toy_RemoveUserData), ud as *mut c_void);
        }
        (*ud).x44 = 0.0;
    }
}

/// Whether trophy `arg0`'s category counts as a "real" win (the C's `var_r30`).
pub unsafe fn _tyFigupon_8031638C(arg0: i16) -> i32 {
    let temp_r0 = Toy_803060BC(arg0 as i32, 6) as i32;
    let mut var_r30 = 0;
    match temp_r0 {
        0 | 1 | 4 | 6 | 8 => {}
        7 => {
            if Toy_803048C0(arg0 as i32) > 0xFA {
                var_r30 = 1;
            }
        }
        5 => var_r30 = 1,
        2 | 3 => var_r30 = 1,
        _ => {}
    }
    var_r30
}

/// Award trophy `arg0`: show its model, unlock it, save, and show the name / "NEW!" popup.
pub unsafe fn _tyFigupon_80316420(arg0: i32) {
    let id = arg0 as i16;
    let ef4_l = _tyFigupon_804D6EF4;
    let data = _tyFigupon_804D6EF0;
    let aa8 = &raw mut Toy_804A2AA8;
    let count = Toy_803048C0(id as i32);
    Toy_80308250(_tyFigupon_804D6EF8, arg0 as i16, 0);
    let gobj = Toy_803087F4(_tyFigupon_804D6EF8);
    ext::HSD_JObjClearFlagsAll((*aa8).jobj[0], JOBJ_HIDDEN as u32);
    ext::HSD_JObjSetFlagsAll((*aa8).jobj[1], JOBJ_HIDDEN as u32);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    HSD_JObjSetTranslateX(&mut *jobj, -1.8, &mut Dirty);
    HSD_JObjSetTranslateY(&mut *jobj, 18.6, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *jobj, 12.0, &mut Dirty);
    HSD_JObjSetScaleX(&mut *jobj, 0.79, &mut Dirty);
    HSD_JObjSetScaleY(&mut *jobj, 0.79, &mut Dirty);
    HSD_JObjSetScaleZ(&mut *jobj, 0.79, &mut Dirty);
    let angle = 30.0f32 * (2.0f32 * (ext::HSD_Randf() - 0.5f32));
    HSD_JObjSetRotationY(&mut *jobj, 0.017453292f32 * angle, &mut Dirty);
    ext::HSD_GObj_SetupProc(gobj, Some(_tyFigupon_80316170), 0);
    ext::HSD_GObj_80390CD4(gobj);
    if count == 0 {
        Toy_80306B18((*data).x8, 0, 0x48, 1);
        ext::HSD_GObj_80390CD4((*data).x8);
    }
    Toy_SetUnlockState(arg0, 1);
    Toy_8031234C(1);
    ext::lbCardGame_SaveChanges();
    ext::HSD_SisLib_803A6368((*data).x14, Toy_80308328(arg0));
    Toy_803083D8((*ef4_l).jobjs[0xC], id as i32);
    if (*toy_mode_state()).x0 == 2 {
        if (*data).x10 == 0 {
            (*data).x10 = ext::HSD_GObj_SetupProc((*data).x0, Some(_tyFigupon_80315574), 0) as usize;
        }
        (*data).x24 = 0x12C;
        (*(*_tyFigupon_804D6EF0).x18).hidden = 0;
    }
}

/// Camera orbit proc: the C-stick tilts the view about the interest point.
pub unsafe extern "C" fn _tyFigupon_803168DC(arg0: *mut HSD_GObj) {
    let data = &raw mut Toy_804A2AA8;
    let cobj = (*arg0).hsd_obj as *mut HSD_CObj;
    let mut interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cross = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut eye_vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut mtx: Mtx = core::mem::zeroed();
    let mut cx: f32 = 0.0;
    let mut cy: f32 = 0.0;
    let rot_x: f32;
    let rot_y: f32;
    let pad = &raw mut ext::HSD_PadCopyStatus as *mut HSD_PadStatus;
    for i in 0..4 {
        cx = (*pad.add(i)).nml_subStickX;
        cy = (*pad.add(i)).nml_subStickY;
        if !(-0.4 < cx && cx < 0.4 && -0.4 < cy && cy < 0.4) {
            break;
        }
    }
    if -0.4 < cx && cx < 0.4 && -0.4 < cy && cy < 0.4 {
        rot_y = 0.0;
        rot_x = 0.0;
    } else {
        let dead: f32;
        if -0.4 < cx && cx < 0.4 {
            rot_x = 0.0;
        } else {
            dead = if cx < 0.0 { -0.4 } else { 0.4 };
            rot_x = 30.0f32 * ((cx - dead) / 0.6);
        }
        if -0.4 < cy && cy < 0.4 {
            rot_y = 0.0;
        } else {
            let dead = if cy < 0.0 { -0.4 } else { 0.4 };
            rot_y = 30.0f32 * ((cy - dead) / 0.6);
        }
    }
    ext::HSD_CObjInit(cobj, _tyFigupon_804D6F04);
    let mut up_copy = _tyFigupon_803B8968;
    ext::HSD_CObjGetEyeVector(cobj, &mut eye_vec);
    ext::HSD_CObjGetInterest(cobj, &mut interest);
    ext::PSVECCrossProduct(&mut up_copy, &mut eye_vec, &mut cross);
    ext::lbVector_Normalize(&mut cross);
    ext::PSMTXRotAxisRad(mtx.as_mut_ptr() as *mut c_void, &mut cross, 0.017453292f32 * rot_y);
    ext::PSMTXMultVec(mtx.as_mut_ptr() as *mut c_void, &mut eye_vec, &mut eye_vec);
    ext::PSMTXRotAxisRad(mtx.as_mut_ptr() as *mut c_void, &mut up_copy, 0.017453292f32 * rot_x);
    ext::PSMTXMultVec(mtx.as_mut_ptr() as *mut c_void, &mut eye_vec, &mut eye_vec);
    ext::PSVECScale(&mut eye_vec, &mut eye_vec, ext::HSD_CObjGetEyeDistance(cobj));
    ext::PSVECSubtract(&mut interest, &mut eye_vec, &mut eye_pos);
    ext::HSD_CObjSetEyePosition(cobj, &mut eye_pos);
    if !(*data).gobj.is_null() {
        if rot_y > 10.0 {
            ext::HSD_JObjClearFlagsAll((*data).jobj[0], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*data).jobj[1], JOBJ_HIDDEN as u32);
            return;
        }
        if rot_y < -10.0 {
            ext::HSD_JObjClearFlagsAll((*data).jobj[1], JOBJ_HIDDEN as u32);
            ext::HSD_JObjSetFlagsAll((*data).jobj[0], JOBJ_HIDDEN as u32);
        }
    }
}

pub unsafe extern "C" fn _tyFigupon_80316BF8(_arg0: *mut HSD_GObj) {
    ext::lbAudioAx_80023F28(0x35);
    ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
}

/// Machine input proc: bet up/down, max bet, pull the lever, leave.
pub unsafe extern "C" fn _tyFigupon_80316C24(arg0: *mut HSD_GObj) {
    let ef4_l = _tyFigupon_804D6EF4;
    let data = _tyFigupon_804D6EF0;
    let stick_scroll: f32;
    if (*ef4_l).x5C != 0 {
        return;
    }
    {
        let stick = Toy_80305DB0();
        if stick > -0.2 && stick < 0.2 {
            stick_scroll = 0.0;
        } else {
            let dir: i32 = if stick > 0.0 { 1 } else { -1 };
            stick_scroll = -fmadds(0.2, dir as f32, -stick) / 0.8; // fused
        }
    }
    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::mn_8022F268();
        (*toy_mode_state()).x4 = 1;
        return;
    }
    if Toy_80305B88() & 0x200 != 0 {
        if (ext::gm_801623D8() as u32 / 10) == 0 || (*ef4_l).x5E <= 1 {
            sfxBack();
            (*toy_mode_state()).x4 = 1;
            return;
        }
        (*ef4_l).x5E = 1;
        _tyFigupon_803153EC((*ef4_l).x5E as i32, 6, 2, 0, 0);
        bet_anim(2);
        setupPercentDisplay(ef4_l);
        sfxMove();
        (*data).x28 = 1;
        return;
    }
    {
        let x28_val = (*data).x28;
        if x28_val as i8 != 0 {
            (*data).x28 = x28_val.wrapping_sub(1);
            return;
        }
    }
    if (ext::gm_801623D8() as u32 / 10) == 0 || (*ef4_l).x5E == 0 {
        if stick_scroll != 0.0 || (Toy_80305B88() & 0x1D7C) != 0 {
            ext::lbAudioAx_80024030(3);
            (*data).x28 = 0x1E;
        }
    } else {
        if Toy_80305B88() & 0x10 != 0 {
            if ext::gm_801623D8() as u32 / 10 > 0x14 {
                (*ef4_l).x5E = 0x14;
            } else {
                (*ef4_l).x5E = (ext::gm_801623D8() as u32 / 10) as i8;
            }
            _tyFigupon_803153EC((*ef4_l).x5E as i32, 6, 2, 0, 0);
            bet_anim(1);
            setupPercentDisplay(ef4_l);
            sfxMove();
            (*data).x28 = 1;
            return;
        }
        if Toy_80305B88() & 0x1160 != 0 {
            let temp = (*ef4_l).x5E as u8;
            if temp as i8 != 0 {
                (*ef4_l).x5D = temp as i8;
                (*ef4_l).x56 = 0x1E;
                ext::HSD_GObj_SetupProc((*ef4_l).x00 as *mut HSD_GObj, Some(_tyFigupon_80315C44), 0);
                ext::HSD_GObj_80390CD4((*ef4_l).x00 as *mut HSD_GObj);
                ext::HSD_GObj_SetupProc((*ef4_l).x0C as *mut HSD_GObj, Some(_tyFigupon_803152BC), 0);
                ext::HSD_GObj_80390CD4((*ef4_l).x0C as *mut HSD_GObj);
                (*ef4_l).x5C = 1;
                ext::HSD_GObj_SetupProc(arg0, Some(_tyFigupon_803155C8), 0);
                ext::HSD_GObj_80390CD4(arg0);
                return;
            }
        }
        if stick_scroll <= -0.1 || (Toy_80305C44() & 0x400) != 0 || (Toy_80305B88() & 4) != 0 {
            let temp = (*ef4_l).x5E as u8;
            if (temp as i8) > 1 {
                (*ef4_l).x5E = temp.wrapping_sub(1) as i8;
                _tyFigupon_803153EC((*ef4_l).x5E as i32, 6, 2, 0, 0);
                bet_anim(2);
                setupPercentDisplay(ef4_l);
                sfxMove();
                (*data).x28 = 3;
            }
        } else if stick_scroll >= 0.1 || (Toy_80305C44() & 0x800) != 0 || (Toy_80305B88() & 8) != 0 {
            let total = ext::gm_801623D8() as u32 / 10;
            let temp = (*ef4_l).x5E as u8;
            if (temp as i8 as i32 as u32) < total && (temp as i8) < 0x14 {
                (*ef4_l).x5E = temp.wrapping_add(1) as i8;
                _tyFigupon_803153EC((*ef4_l).x5E as i32, 6, 2, 0, 0);
                bet_anim(1);
                setupPercentDisplay(ef4_l);
                sfxMove();
                (*data).x28 = 3;
            }
        } else {
            bet_anim(0);
        }
    }
}

/// `tyFigupon_CountAvailable` (static inline).
unsafe fn tyFigupon_CountAvailable(result: *mut i32) {
    let mut count = 0;
    for i in 0..9 {
        if i != 8 && i as u32 > 1 && Toy_80304B0C(i) {
            count += Toy_80304B94(i);
        }
    }
    *result = count;
}

/// Build the machine's panel, background, coin, digit, bet, lever and parlor objects.
pub unsafe fn _tyFigupon_8031753C() {
    let ef4_l = _tyFigupon_804D6EF4;
    let mut new_count: i32 = 0;
    if (*ef4_l).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        panic_at(1372);
    }
    if (*ef4_l).x00 != 0 {
        ext::HSD_GObjFree((*ef4_l).x00 as *mut HSD_GObj);
        (*ef4_l).x00 = 0;
    }
    let panel_joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonPanel_Top_joint")) as *mut HSD_Joint;
    if !panel_joint.is_null() {
        let mut digits_s: [i32; 4] = [0; 4];
        (*ef4_l).x00 = ext::GObj_Create(9, 9, 0) as usize;
        let mut jobj = ext::HSD_JObjLoadJoint(panel_joint);
        ext::HSD_GObjObject_80390A70((*ef4_l).x00 as *mut HSD_GObj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        let render_cb: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)> = Some(ext::cb::HSD_GObj_JObjCallback);
        ext::GObj_SetupGXLink((*ef4_l).x00 as *mut HSD_GObj, render_cb, 0x3C, 1);
        ext::lb_8001204C(jobj, (*ef4_l).jobjs.as_mut_ptr(), _tyFigupon_803FEB88.as_mut_ptr(), 0xD);
        _tyFigupon_80314AA8((*ef4_l).jobjs[0xC], core::ptr::null(), cstr!("ToyFigurePonPanel_zsmash_matanim_joint"), core::ptr::null());
        Toy_803083D8((*ef4_l).jobjs[0xC], 0x3E7);
        let mut joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonBg_Top_joint")) as *mut HSD_Joint;
        (*ef4_l).unk4 = ext::GObj_Create(9, 9, 0);
        jobj = ext::HSD_JObjLoadJoint(joint);
        ext::HSD_GObjObject_80390A70((*ef4_l).unk4, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink((*ef4_l).unk4, render_cb, 0x3C, 0);
        _tyFigupon_80314AA8(
            jobj,
            cstr!("ToyFigurePonBg_Top_animjoint"),
            cstr!("ToyFigurePonBg_Top_matanim_joint"),
            cstr!("ToyFigurePonBg_Top_shapeanim_joint"),
        );
        ext::HSD_GObj_SetupProc((*ef4_l).unk4, Some(Toy_80306BB8), 0);
        ext::HSD_GObj_80390CD4((*ef4_l).unk4);
        jobj = ext::HSD_JObjLoadJoint(ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonCoin_Top_joint")) as *mut HSD_Joint);
        ext::HSD_JObjAddChild((*ef4_l).jobjs[0], jobj);
        _tyFigupon_80314AA8(
            jobj,
            cstr!("ToyFigurePonCoin_Top_animjoint"),
            cstr!("ToyFigurePonCoin_Top_matanim_joint"),
            cstr!("ToyFigurePonCoin_Top_shapeanim_joint"),
        );
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        let mut count: usize = 0;
        let mut total = (ext::gm_801623D8() as u32 / 10) as i32;
        joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonNm_Top_joint")) as *mut HSD_Joint;
        loop {
            if count < 4 {
                digits_s[count] = total % 10;
            }
            count += 1;
            total /= 10;
            if !(total > 0) {
                break;
            }
        }
        for i in 0..3usize {
            jobj = ext::HSD_JObjLoadJoint(joint);
            ext::HSD_JObjAddChild((*ef4_l).jobjs[3 + i], jobj);
            _tyFigupon_80314AA8(
                jobj,
                cstr!("ToyFigurePonNm_Top_animjoint"),
                cstr!("ToyFigurePonNm_Top_matanim_joint"),
                cstr!("ToyFigurePonNm_Top_shapeanim_joint"),
            );
            if digits_s[i] != 0 {
                ext::HSD_JObjReqAnimAll(jobj, (50 - digits_s[i] * 5) as f32);
            } else {
                ext::HSD_JObjReqAnimAll(jobj, 0.0);
            }
            ext::HSD_JObjAnimAll(jobj);
        }
        let bet_joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonBet_Top_joint")) as *mut HSD_Joint;
        for count in 0..2usize {
            jobj = ext::HSD_JObjLoadJoint(bet_joint);
            ext::HSD_JObjAddChild((*ef4_l).jobjs[6 + count], jobj);
            _tyFigupon_80314AA8(
                jobj,
                cstr!("ToyFigurePonBet_Top_animjoint"),
                cstr!("ToyFigurePonBet_Top_matanim_joint"),
                cstr!("ToyFigurePonBet_Top_shapeanim_joint"),
            );
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
            ext::HSD_JObjAnimAll(jobj);
        }
        _tyFigupon_80314AA8((*ef4_l).jobjs[8], core::ptr::null(), cstr!("ToyFigurePonPanel_zarrow_matanim_joint"), core::ptr::null());
        joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonLever_Top_joint")) as *mut HSD_Joint;
        (*ef4_l).x08 = ext::GObj_Create(9, 9, 0) as usize;
        jobj = ext::HSD_JObjLoadJoint(joint);
        ext::HSD_GObjObject_80390A70((*ef4_l).x08 as *mut HSD_GObj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
        ext::GObj_SetupGXLink((*ef4_l).x08 as *mut HSD_GObj, Some(ext::cb::HSD_GObj_JObjCallback), 0x3C, 0);
        _tyFigupon_80314AA8(
            jobj,
            cstr!("ToyFigurePonLever_Top_animjoint"),
            cstr!("ToyFigurePonLever_Top_matanim_joint"),
            cstr!("ToyFigurePonLever_Top_shapeanim_joint"),
        );
        ext::lb_8001204C(jobj, &raw mut (*ef4_l).jobjs[0xE], _tyFigupon_804D5AA0.as_mut_ptr(), 1);
        let trophy_total = _tyFigupon_80314B54();
        tyFigupon_CountAvailable(&mut new_count);
        if new_count - trophy_total != 0 {
            ext::HSD_JObjReqAnimAll((*ef4_l).jobjs[0xE], 0.0);
        } else {
            ext::HSD_JObjReqAnimAll((*ef4_l).jobjs[0xE], 1.0);
        }
        ext::HSD_JObjAnimAll((*ef4_l).jobjs[0xE]);
        (*ef4_l).x0C = ext::GObj_Create(9, 9, 0) as usize;
        let par_joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonPar_Top_joint")) as *mut HSD_Joint;
        for count in 0..3usize {
            jobj = ext::HSD_JObjLoadJoint(par_joint);
            ext::HSD_JObjAddChild((*ef4_l).jobjs[9 + count], jobj);
            _tyFigupon_80314AA8(
                jobj,
                cstr!("ToyFigurePonPar_Top_animjoint"),
                cstr!("ToyFigurePonPar_Top_matanim_joint"),
                cstr!("ToyFigurePonPar_Top_shapeanim_joint"),
            );
            ext::HSD_JObjReqAnimAll(jobj, 0.0);
            ext::HSD_JObjAnimAll(jobj);
        }
        return;
    }
    ext::OSReport(&format!("*** Can not Load Panel Label({})\n", "ToyFigurePonPanel_Top_joint"));
    panic_at(1552);
}

/// Cameras, text objects and the input procs of the scene.
pub unsafe fn _tyFigupon_80317A60() {
    let data = _tyFigupon_804D6EF0;
    let ef4_l = _tyFigupon_804D6EF4;
    let cam_desc =
        ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, Toy_str_ScMenFigure_cam_int1_camera.as_ptr() as *const i8) as *mut HSD_CameraDescPerspective;
    (*data).x0 = ext::GObj_Create(1, 2, 0);
    let mut cobj = ext::lb_80013B14(cam_desc);
    _tyFigupon_804D6F04 = cam_desc as *mut HSD_CObjDesc;
    ext::HSD_GObjObject_80390A70((*data).x0, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax((*data).x0, Some(_tyFigupon_80314BE4), 0);
    (*(*data).x0).gxlink_prios = 0x5010000000000000u64;
    (*data).x4 = ext::GObj_Create(1, 2, 0);
    cobj = ext::lb_80013B14(&raw mut _tyFigupon_803FEEFC);
    _tyFigupon_804D6F08 = &raw mut _tyFigupon_803FEEFC;
    ext::HSD_GObjObject_80390A70((*data).x4, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax((*data).x4, Some(Toy_803068E0), 0);
    (*(*data).x4).gxlink_prios = 0x2680000000000000u64;
    {
        let is_jp = ext::lbLang_IsSavedLanguageJP();
        if is_jp != 0 {
            ext::HSD_SisLib_803A62A0(0, cstr!("SdToy.dat"), cstr!("SIS_ToyData"));
        } else {
            ext::HSD_SisLib_803A62A0(0, cstr!("SdToy.usd"), cstr!("SIS_ToyData_E"));
        }
    }
    _tyFigupon_804D6EFC = ext::HSD_SisLib_803A611C(0, (*data).x0, 0xB, 0xB, 0, 0x3E, 0, 0);
    (*data).x14 = ext::HSD_SisLib_803A5ACC(0, _tyFigupon_804D6EFC, 5.2, 5.2, 17.2, 448.0, 64.0);
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        let text = (*data).x14;
        (*text).x34.x = 1.6;
        (*text).x34.y = 0.7;
    } else {
        let text = (*data).x14;
        (*text).x34.x = 2.0;
        (*text).x34.y = 1.0;
    }
    let text = (*data).x14;
    (*text).font_size.x = 0.0235;
    (*text).font_size.y = 0.0715;
    (*(*data).x14).default_alignment = 1;
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        (*(*data).x14).default_kerning = 1;
    }
    (*(*data).x14).default_fitting = 1;
    _tyFigupon_804D6F00 = ext::HSD_SisLib_803A611C(0, (*data).x0, 0xD, 0xB, 0, 0x3E, 0, 0);
    (*data).x18 = ext::HSD_SisLib_803A5ACC(0, _tyFigupon_804D6F00, -13.2, 9.4, 17.2, 320.0, 32.0);
    let text = (*data).x18;
    (*text).font_size.x = 0.027;
    (*text).font_size.y = 0.028;
    (*(*data).x18).default_kerning = 1;
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        let text = (*data).x18;
        (*text).x34.x = 0.8;
        (*text).x34.y = 0.8;
        (*(*data).x18).pos_x = -13.599999;
        (*(*data).x18).pos_y = 9.599999;
    }
    ext::HSD_SisLib_803A6368((*data).x18, 0x13C);
    (*data).x10 = 0;
    (*data).x24 = 0;
    (*data).x28 = 8;
    (*(*_tyFigupon_804D6EF0).x18).hidden = 1;
    ext::HSD_GObj_SetupProc((*data).x0, Some(_tyFigupon_80316C24), 0);
    ext::HSD_GObj_SetupProc((*data).x0, Some(_tyFigupon_803168DC), 0);
    ext::HSD_GObj_SetupProc((*data).x4, Some(_tyFigupon_803168DC), 0);
    ext::HSD_GObj_SetupProc((*data).x0, Some(_tyFigupon_80316BF8), 0);
    ext::HSD_GObj_80390CD4((*data).x0);
}

/// `tyFigupon_InitScene` (static inline). `ED4` is the `Toy_sbss_804D6ED4` camera record, whose
/// first slot (`x00`) is the scene-light gobj and `archive` (+0xC) the light archive.
unsafe fn tyFigupon_InitScene(ef4_2: &mut *mut un_804D6EF4_t) -> *mut ToyCameraControl {
    _tyFigupon_80317A60();
    *ef4_2 = _tyFigupon_804D6EF4;
    let ed4 = Toy_sbss_804D6ED4;
    if (**ef4_2).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        panic_at(1575);
    }
    ed4
}

/// `tyFigupon_UpdateRemainingCount` (static inline).
unsafe fn tyFigupon_UpdateRemainingCount(ef4_arg: *mut un_804D6EF4_t) {
    let total_b54 = _tyFigupon_80314B54();
    let mut count = 0;
    for i in 0..9 {
        if i != 8 && i as u32 > 1 && Toy_80304B0C(i) {
            count += Toy_80304B94(i);
        }
    }
    (*ef4_arg).x54 = (count - total_b54) as i16;
}

pub unsafe fn tyFigupon_Scene_OnEnter(_arg0: *mut c_void) {
    let mut sp20: *mut c_void = core::ptr::null_mut();
    let mut ef4_2: *mut un_804D6EF4_t = core::ptr::null_mut();
    _tyFigupon_804D6EF0 = ext::HSD_MemAlloc(core::mem::size_of::<TyFiguponData>() as i32) as *mut TyFiguponData;
    _tyFigupon_804D6EF4 = ext::HSD_MemAlloc(core::mem::size_of::<un_804D6EF4_t>() as i32) as *mut un_804D6EF4_t;
    _tyFigupon_804D6EF8 = ext::HSD_MemAlloc(core::mem::size_of::<ToyListEntry>() as i32) as *mut ToyListEntry;
    Toy_sbss_804D6ED4 = ext::HSD_MemAlloc(core::mem::size_of::<ToyCameraControl>() as i32) as *mut ToyCameraControl;
    ext::memzero(_tyFigupon_804D6EF0 as *mut c_void, core::mem::size_of::<TyFiguponData>() as i32);
    ext::memzero(_tyFigupon_804D6EF4 as *mut c_void, core::mem::size_of::<un_804D6EF4_t>() as i32);
    ext::memzero(_tyFigupon_804D6EF8 as *mut c_void, core::mem::size_of::<ToyListEntry>() as i32);
    ext::memzero(Toy_sbss_804D6ED4 as *mut c_void, core::mem::size_of::<ToyCameraControl>() as i32);
    let data = _tyFigupon_804D6EF0;
    Toy_sbss_804D6EC8 = core::ptr::null_mut();
    let ef4_l = _tyFigupon_804D6EF4;
    Toy_8031263C();
    ToyMode_set_state1();
    let archive_name = if ext::lbLang_IsSavedLanguageJP() != 0 { cstr!("TyMnFigp.dat") } else { cstr!("TyMnFigp.usd") };
    (*ef4_l).archive = ext::lbArchive_LoadSymbols(archive_name, &[(&raw mut sp20) as *mut c_void, cstr!("ToyFigurePonPanel_Top_joint") as *mut c_void]);
    (*ef4_l).x58 = 0;
    let ed4 = tyFigupon_InitScene(&mut ef4_2);
    {
        let temp = ext::HSD_ArchiveGetPublicAddress((*ef4_2).archive, cstr!("ScMenFigure_scene_lights")) as *mut *mut LightList;
        if !temp.is_null() {
            (*ed4).x00 = ext::GObj_Create(2, 3, 0);
            let lobj = Toy_LoadLObjList(temp, core::ptr::null_mut());
            ext::HSD_GObjObject_80390A70((*ed4).x00, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
            let gobj = (*ed4).x00;
            ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_LObjCallback), 0x34, 0);
        }
    }
    ext::memzero(Toy_sbss_804D6ED4 as *mut c_void, core::mem::size_of::<ToyCameraControl>() as i32);
    Toy_80306D70(0);
    _tyFigupon_8031753C();
    let joint = ext::HSD_ArchiveGetPublicAddress((*ef4_l).archive, cstr!("ToyFigurePonNget_Top_joint")) as *mut HSD_Joint;
    (*data).x8 = ext::GObj_Create(0xA, 0xA, 0);
    let jobj = ext::HSD_JObjLoadJoint(joint);
    ext::HSD_GObjObject_80390A70((*data).x8, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    _tyFigupon_80314AA8(
        jobj,
        cstr!("ToyFigurePonNget_Top_animjoint"),
        cstr!("ToyFigurePonNget_Top_matanim_joint"),
        cstr!("ToyFigurePonNget_Top_shapeanim_joint"),
    );
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    HSD_JObjAddTranslationZ(&mut *jobj, 0.5, &mut Dirty);
    let ud = ext::HSD_MemAlloc(core::mem::size_of::<TyFiguponUD>() as i32);
    if !ud.is_null() {
        ext::GObj_InitUserData((*data).x8, 0, Some(Toy_RemoveUserData), ud);
    }
    ext::GObj_SetupGXLink((*data).x8, Some(ext::cb::HSD_GObj_JObjCallback), 0x3D, 0);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    {
        let has_coin_credit = if (ext::gm_801623D8() as u32 / 10) != 0 { 1 } else { 0 };
        (*ef4_l).x5E = has_coin_credit;
    }
    _tyFigupon_803153EC((*ef4_l).x5E as i32, 6, 2, 0, 0);
    tyFigupon_UpdateRemainingCount(ef4_l);
    setupPercentDisplay(ef4_l);
    ext::HSD_PadRenewStatus();
}

/// `((TyModeState*) Toy_804A284C)->x0 = 1` plus the preceding `Toy_80305918(3, 0, 1)`.
#[inline]
unsafe fn ToyMode_set_state1() {
    super::toy::Toy_80305918(3, 0, 1);
    (*toy_mode_state()).x0 = 1;
}

/// Release the scene's pointers (the objects themselves are freed by the scene exit).
pub unsafe fn _tyFigupon_803181BC() -> i32 {
    let ef4_l = _tyFigupon_804D6EF4;
    let temp_r31 = _tyFigupon_804D6EF0;
    let ed4 = Toy_sbss_804D6ED4;
    if !(*ef4_l).archive.is_null() {
        (*ef4_l).archive = core::ptr::null_mut();
    }
    if !Toy_sbss_804D6EC8.is_null() {
        Toy_sbss_804D6EC8 = core::ptr::null_mut();
    }
    if !(*ed4).archive.is_null() {
        (*ed4).archive = core::ptr::null_mut();
    }
    if (*ef4_l).x00 != 0 {
        (*ef4_l).x00 = 0;
    }
    if (*ef4_l).x08 != 0 {
        (*ef4_l).x08 = 0;
    }
    if (*ef4_l).x0C != 0 {
        (*ef4_l).x0C = 0;
    }
    if !(*ed4).x00.is_null() {
        (*ed4).x00 = core::ptr::null_mut();
    }
    if !(*ed4).x04.is_null() {
        (*ed4).x04 = core::ptr::null_mut();
    }
    if !(*temp_r31).x0.is_null() {
        ext::HSD_GObjProc_RemoveAllProcs((*temp_r31).x0);
        (*temp_r31).x0 = core::ptr::null_mut();
    }
    if !(*temp_r31).x4.is_null() {
        (*temp_r31).x4 = core::ptr::null_mut();
    }
    if !(*temp_r31).x8.is_null() {
        ext::HSD_GObjFree((*temp_r31).x8);
        (*temp_r31).x8 = core::ptr::null_mut();
    }
    ext::lbAudioAx_800236DC()
}

pub unsafe fn tyFigupon_Scene_OnFrame() {
    let state = toy_mode_state();
    if (*state).x4 != 0 {
        _tyFigupon_803181BC();
        ext::gm_801A4B60();
    }
}

/// Atlas step 10 readbacks for the framed Lottery: 0 the coin balance the machine shows
/// (`gm_801623D8() / 10`), 1 the bet retail has chosen (1 to 20), 2 nonzero while a draw is
/// running, -1 when the scene's state is not there.
pub unsafe fn tyFigupon_PcReadback(what: i32) -> i32 {
    let ef4_l = _tyFigupon_804D6EF4;
    match what {
        0 => (ext::gm_801623D8() as u32 / 10) as i32,
        1 => {
            if !ef4_l.is_null() {
                (*ef4_l).x5E as i32
            } else {
                -1
            }
        }
        2 => {
            if !ef4_l.is_null() {
                (*ef4_l).x5C as i32
            } else {
                -1
            }
        }
        _ => -1,
    }
}
