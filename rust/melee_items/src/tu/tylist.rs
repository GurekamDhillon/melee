//! Native port of src/melee/ty/tylist.c: the Trophy Gallery's full-screen trophy list (the
//! scrolling name / count rows, the cursor, the 3 sort tabs, the held-direction repeat logic and
//! the scene setup / teardown). Every function of the TU is here, including the TARGET_PC
//! `tyList_PcActive` / `tyList_PcSetActive` flag. Callees in other TUs (toy.c, sislib, ...) go
//! through `ext` (an unset hook panics "TODO: <name> is not ported"; names are the C names).
//!
//! Native-layout notes (the C reads GameCube memory through several overlapping views):
//! * `TyListRow` is the same record as `TyListArg` (`jobj` = `jobjs[0]`, `text0..2` = `texts[0..2]`);
//!   the port uses `TyListArg` for both.
//! * `TyArchiveData` (`gobj`, `data`) is `ToyED8Data` (`gobj`, `archive`); `Toy_sbss_804D6ED8` is
//!   the latter. `Toy_sbss_804D6EE0` is the `TyDisplayData` whose `x140` slot is `selected_entry`.
//! * `TyListData.cobj` (+0x28) is `HSD_GObj.hsd_obj`; `TyListWaitData.x20/x24` (+0x20/+0x24) are
//!   the two halves of `HSD_GObj.gxlink_prios`, so storing `x20 = 0x42100000, x24 = 0` is
//!   `gxlink_prios = 0x4210000000000000`.
//! * `SisFontData.rows[n]` is the 12-slot pointer table the C indexes at `font + 0x4B8`.
//! * The C `memzero(&obj, sizeof obj)` calls pass the native `size_of`.
//! `OSPanic(__FILE__, line, ...)` is `panic_at(line)`; the one `OSReport` is the hand-modelled
//! `ext::OSReport`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, unused_assignments, dead_code)]
use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: i32) -> i32;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) => unsafe { melee_runtime::tu::aobj::HSD_AObjSetRate(a0, a1) };
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void => unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0, a1) };
    fn HSD_CObjEndCurrent() => unsafe { melee_runtime::tu::cobj::HSD_CObjEndCurrent() };
    fn HSD_CObjEraseScreen(a0: *mut HSD_CObj, a1: i32, a2: i32, a3: i32) => unsafe { melee_runtime::tu::cobj::HSD_CObjEraseScreen(a0, a1, a2, a3) };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 => unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec);
    fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) => unsafe { melee_runtime::tu::cobj::HSD_CObjSetScissor(a0, a1 as _) };
    fn HSD_CObjSetViewport(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) => unsafe { melee_runtime::tu::cobj::HSD_CObjSetViewport(a0, a1 as _) };
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObjProc_RemoveAllProcs(a0: *mut HSD_GObj);
    fn HSD_GObjProc_RemoveProc(a0: *mut HSD_GObjProc);
    fn HSD_GObj_80390CD4(a0: *mut HSD_GObj);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0, a1) };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) => unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0, a1) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn HSD_JObjAnim(a0: *mut HSD_JObj);
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj => unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0) };
    fn HSD_JObjReqAnim(a0: *mut HSD_JObj, a1: f32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_JObjUnref(a0: *mut HSD_JObj) => unsafe { melee_runtime::tu::jobj::HSD_JObjUnref(a0) };
    fn HSD_PadRenewStatus() => unsafe { melee_runtime::tu::controller::HSD_PadRenewStatus() };
    fn HSD_SetEraseColor(a0: u8, a1: u8, a2: u8, a3: u8);
    fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0, a1, a2, a3, a4, a5, a6) };
    fn HSD_SisLib_803A5E70() => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5E70() };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0, a1, a2, a3, a4, a5, a6, a7) };
    fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) => unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0, a1) };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text => unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0, a1) };
    fn Toy_803048C0(a0: i32) -> i32 => unsafe { crate::tu::toy::Toy_803048C0(a0) };
    fn Toy_80304924(a0: i32) -> i32 => unsafe { crate::tu::toy::Toy_80304924(a0) as _ };
    fn Toy_80304988(a0: i32) => unsafe { crate::tu::toy::Toy_80304988(a0) };
    fn Toy_80305B88() -> i32 => unsafe { crate::tu::toy::Toy_80305B88() };
    fn Toy_80305C44() -> i32 => unsafe { crate::tu::toy::Toy_80305C44() };
    fn Toy_80305D00() -> f32 => unsafe { crate::tu::toy::Toy_80305D00() };
    fn Toy_80305DB0() -> f32 => unsafe { crate::tu::toy::Toy_80305DB0() };
    fn Toy_803062BC(a0: i32) -> i16 => unsafe { crate::tu::toy::Toy_803062BC(a0) };
    fn Toy_803067BC(a0: i32, a1: i32) => unsafe { crate::tu::toy::Toy_803067BC(a0, a1) };
    fn Toy_80306954(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::toy::Toy_80306954(a0, a1) };
    fn Toy_80306A48(a0: *mut HSD_JObj, a1: *mut i8, a2: *mut i8, a3: *mut i8, a4: *mut HSD_Archive, a5: i32) => unsafe { crate::tu::toy::Toy_80306A48(a0, a1 as _, a2 as _, a3 as _, a4, a5) };
    fn Toy_80307470(a0: i32) => unsafe { crate::tu::toy::Toy_80307470(a0) };
    fn Toy_803082F8(a0: i16) -> i32 => unsafe { crate::tu::toy::Toy_803082F8(a0) };
    fn Toy_80308354(a0: i16) -> i32 => unsafe { crate::tu::toy::Toy_80308354(a0) };
    fn Toy_803083D8(a0: *mut HSD_JObj, a1: i32) => unsafe { crate::tu::toy::Toy_803083D8(a0, a1) };
    fn Toy_80310324() => unsafe { crate::tu::toy::Toy_80310324() };
    fn Toy_80310660(a0: i32) => unsafe { crate::tu::toy::Toy_80310660(a0) };
    fn Toy_GetTrophyTotal() -> i32 => unsafe { crate::tu::toy::Toy_GetTrophyTotal() };
    fn Toy_LoadLObjList(a0: *mut *mut LightList, a1: *mut i32) -> *mut HSD_LObj => unsafe { crate::tu::toy::Toy_LoadLObjList(a0, a1) };
    fn lbAudioAx_80024030(a0: i32) => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0) };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj => unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0) };
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
        pub unsafe extern "C" fn Toy_80306954(a0: *mut HSD_GObj, a1: i32) { super::Toy_80306954(a0, a1) }
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
    pub static mut HSD_SisLib_804D1124: [*mut SIS; 5] = unsafe { core::mem::zeroed() };
    pub static mut Toy_804A284C: [u16; 302] = unsafe { core::mem::zeroed() };
    pub static mut Toy_sbss_804D6ED8: *mut ToyED8Data = unsafe { core::mem::zeroed() };
    pub static mut Toy_sbss_804D6EDC: *mut i16 = unsafe { core::mem::zeroed() };
    pub static mut Toy_sbss_804D6EE0: *mut TyDisplayData = unsafe { core::mem::zeroed() };
    pub static mut Toy_str_ScMenFigure_cam_int1_camera: [i8; 1] = unsafe { core::mem::zeroed() };
    pub static mut lb_ColorBlack: GXColor = unsafe { core::mem::zeroed() };
    pub static mut lb_ColorWhite: GXColor = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

macro_rules! cs {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}

/// `HSD_PAD_XY` (sysdolphin/baselib/controller.h): `HSD_PAD_X | HSD_PAD_Y`.
const HSD_PAD_XY: u32 = (1 << 10) | (1 << 11);

/// `OSPanic(__FILE__, line, ...)` failure.
#[cold]
fn panic_at(line: u32) -> ! {
    panic!("tylist.c:{line}")
}

/// `JObjDirty` host: the shipping `HSD_JObjSetMtxDirty` is `if (!HSD_JObjMtxIsDirty(j))
/// HSD_JObjSetMtxDirtySub(j)`, and the sub is in the unported `jobj.c`.
struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, jobj: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(jobj) {
            ext::HSD_JObjSetMtxDirtySub(jobj);
        }
    }
    fn set_mtx_dirty_sub(&mut self, jobj: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `sfxBack` (mn/inlines.h, static inline).
#[inline]
unsafe fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}
/// `sfxForward` (mn/inlines.h, static inline).
#[inline]
unsafe fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
/// `sfxMove` (mn/inlines.h, static inline).
#[inline]
unsafe fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

#[inline]
unsafe fn toy_mode_state() -> *mut TyModeState {
    &raw mut ext::Toy_804A284C as *mut TyModeState
}

/* Atlas step 10 (TARGET_PC): the port's own bookkeeping, set and cleared where retail begins and
 * ends the list; no retail state is read from it or written through it. */
static mut tyList_pc_active: i32 = 0;

/// `int tyList_PcActive(void)` (TARGET_PC)
pub unsafe fn tyList_PcActive() -> i32 {
    tyList_pc_active
}

/// `void tyList_PcSetActive(int)` (TARGET_PC)
pub unsafe fn tyList_PcSetActive(on: i32) {
    tyList_pc_active = on;
}

/* 4A2AC0 */
static mut _tyList_804A2AC0: TyListState = unsafe { core::mem::zeroed() };
/* 4A2D6C */
static mut _tyList_804A2D6C: TyListGobjEntry = unsafe { core::mem::zeroed() };
/* 4A2D84 */
static mut _tyList_804A2D84: [*mut HSD_GObj; 5] = [null_mut(); 5];
/* 4D6EE8 */
static mut _tyList_804D6EE8: i32 = 0;
/* 4D6EEC */
static mut _tyList_804D6EEC: i32 = 0;

/// `static char* _tyList_80312834(char* buf, u32 num)`: formats a number into a string buffer
/// using digit glyphs from the font.
pub unsafe fn _tyList_80312834(mut buf: *mut i8, mut num: u32) -> *mut i8 {
    let lookup: *mut u8 = (*(ext::HSD_SisLib_804D1124[0] as *mut SisFontData)).digits;
    let original: u32 = num;
    let mut idx: u32;

    if num >= 100 {
        idx = (num / 100) * 2;
        *buf = *lookup.add(idx as usize) as i8;
        buf = buf.add(1);
        *buf = *lookup.add((idx + 1) as usize) as i8;
        buf = buf.add(1);
        num = num % 100;
    }

    if num >= 10 {
        idx = (num / 10) * 2;
        *buf = *lookup.add(idx as usize) as i8;
        buf = buf.add(1);
        *buf = *lookup.add((idx + 1) as usize) as i8;
        buf = buf.add(1);
        num = num % 10;
    } else if original >= 100 {
        *buf = *lookup.add(0) as i8;
        buf = buf.add(1);
        *buf = *lookup.add(1) as i8;
        buf = buf.add(1);
    }

    idx = num * 2;
    *buf = *lookup.add(idx as usize) as i8;
    buf = buf.add(1);
    *buf = *lookup.add((idx + 1) as usize) as i8;
    buf = buf.add(1);
    *buf = 0;
    buf
}

/// `static void _tyList_80312904(void* arg0, s8 arg1)` (`arg0` is a `TyListRow` = `TyListArg`)
pub unsafe fn _tyList_80312904(arg0: *mut TyListArg, arg1: i8) {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;
    let row: *mut TyListArg = arg0;
    let skip: i32;

    if (*row).idx == -1 {
        return;
    }

    if ext::Toy_GetTrophyTotal() < 0xB {
        let v: i8 = (*row).x24;
        if v == -1 || v as i32 == (*state).entryCount as i32 - 2 {
            skip = 0;
        } else {
            skip = 1;
        }
    } else {
        skip = 1;
    }

    if skip == 0 {
        return;
    }

    let f30: f32 = HSD_JObjGetTranslationX(&*(*row).jobjs[0]) - 6.5f32;
    let f29: f32 = (-(*row).x30 - HSD_JObjGetTranslationY(&*((*(*state).gobj).hsd_obj as *mut HSD_JObj))) - 0.41f32;
    let f31: f32 = HSD_JObjGetTranslationZ(&*((*(*state).gobj).hsd_obj as *mut HSD_JObj));

    if arg1 != 0x63 {
        if (*row).x24 == arg1 {
            (*(*row).texts[0]).text_color = ext::lb_ColorBlack;
            (*(*row).texts[1]).text_color = ext::lb_ColorBlack;
            (*(*row).texts[2]).text_color = ext::lb_ColorBlack;
        } else {
            (*(*row).texts[0]).text_color = ext::lb_ColorWhite;
            (*(*row).texts[1]).text_color = ext::lb_ColorWhite;
            (*(*row).texts[2]).text_color = ext::lb_ColorWhite;
        }
    }

    let mut text: *mut HSD_Text = (*row).texts[0];
    (*text).pos_x = f30;
    (*text).pos_y = f29;
    (*text).pos_z = f31;
    text = (*row).texts[0];
    (*text).font_size.x = 0.028f32;
    (*text).font_size.y = 0.029f32;
    (*(*row).texts[0]).default_kerning = 1;
    ext::HSD_SisLib_803A6368((*row).texts[0], ext::Toy_803082F8((*row).idx));

    text = (*row).texts[1];
    (*text).pos_x = 14.7f32 + f30;
    (*text).pos_y = f29;
    (*text).pos_z = f31;
    text = (*row).texts[1];
    (*text).font_size.x = 0.028f32;
    (*text).font_size.y = 0.029f32;
    ext::HSD_SisLib_803A6368((*row).texts[1], 0x13B);

    let digits: *mut u8 = (*(ext::HSD_SisLib_804D1124[0] as *mut SisFontData)).rows[(*row).x28 as usize];
    _tyList_80312834(digits as *mut i8, ext::Toy_803048C0(ext::Toy_80308354((*row).idx)) as u32);

    (*(*row).texts[2]).default_alignment = 2;
    (*(*row).texts[2]).default_kerning = 1;
    text = (*row).texts[2];
    (*text).pos_x = 10.5f32 + f30;
    (*text).pos_y = f29;
    (*text).pos_z = f31;
    text = (*row).texts[2];
    (*text).font_size.x = 0.038f32;
    (*text).font_size.y = 0.029f32;
    ext::HSD_SisLib_803A6368((*row).texts[2], (*row).x28 + 0x12E);
}

/// `static void _tyList_80312BAC(TyListState*, s8)`
pub unsafe fn _tyList_80312BAC(state: *mut TyListState, arg1: i8) {
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    let mut trophy_idx: i32;

    let old_idx: i16 = *ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize);
    ext::Toy_803067BC((*state).x29B as i32, (*state).x29C as i32);
    (*state).selectedIdx = ext::Toy_803062BC(old_idx as i32);

    if ext::Toy_GetTrophyTotal() <= 0xA {
        trophy_idx = (*state).selectedIdx as i32;
        let mut i: i32 = 0;
        while i < (*(*state).x278).x24 as i32 + 1 {
            trophy_idx -= 1;
            if trophy_idx < 0 {
                trophy_idx = ext::Toy_GetTrophyTotal() - 1;
            }
            i += 1;
        }
    } else {
        trophy_idx = (*state).selectedIdx as i32;
        let mut i: i32 = 0;
        while i < (*(*state).x278).x24 as i32 + 1 {
            trophy_idx -= 1;
            if trophy_idx < 0 {
                trophy_idx = ext::Toy_GetTrophyTotal() - 1;
            }
            i += 1;
        }
    }

    let first_row: *mut TyListArg = (*state).x270;
    let mut row: *mut TyListArg = first_row;
    if !(*state).jobj.is_null() {
        HSD_JObjSetTranslateY(&mut *(*state).jobj, (*first_row).x30, &mut Dirty);
    }

    let mut i: i32 = 0;
    while i < (*state).entryCount as i32 {
        (*row).idx = trophy_idx as i16;
        _tyList_80312904(row, arg1);
        _tyList_80313464(row);
        if (*row).x24 >= 0 && ((*row).x24 as i32) < (*state).entryCount as i32 - 2 && (*row).idx as i32 == ext::Toy_GetTrophyTotal() - 1 {
            if !(*state).jobj.is_null() {
                HSD_JObjSetTranslateY(&mut *(*state).jobj, (*row).x30, &mut Dirty);
            }
        }
        row = (*row).links[1];
        trophy_idx += 1;
        if trophy_idx >= ext::Toy_GetTrophyTotal() {
            trophy_idx = 0;
        }
        i += 1;
    }

    i = 0;
    while i < 3 {
        if i == (*state).x29B as i32 {
            ext::HSD_JObjReqAnim((*archive).jobjs[i as usize], 1.0);
        } else {
            ext::HSD_JObjReqAnim((*archive).jobjs[i as usize], 0.0);
        }
        ext::HSD_AObjSetRate((*(*(*(*(*archive).jobjs[i as usize]).u.dobj).mobj).tobj).aobj, 0.0);
        ext::HSD_JObjAnim((*archive).jobjs[i as usize]);
        i += 1;
    }
}

/// `static void _tyList_80312E88(TyListArg*, float)`
pub unsafe fn _tyList_80312E88(arg: *mut TyListArg, delta: f32) {
    (*arg).x30 += delta;

    if delta == 999.0f32 {
        (*arg).x30 = (*arg).x2C;
    } else if delta < 0.0f32 {
        if (*arg).x30 < (*arg).x2C {
            (*arg).x30 = (*arg).x2C;
        }
    } else {
        if (*arg).x30 > (*arg).x2C {
            (*arg).x30 = (*arg).x2C;
        }
    }

    let mut i: usize = 0;
    while i < 3 {
        if (*arg).jobjs[i].is_null() {
            i += 1;
            continue;
        }
        if i == 2 {
            i += 1;
            continue;
        }
        HSD_JObjSetTranslateY(&mut *(*arg).jobjs[i], (*arg).x30, &mut Dirty);

        if (*arg).idx as i32 == ext::Toy_GetTrophyTotal() - 1 {
            HSD_JObjSetTranslateY(&mut *_tyList_804A2AC0.jobj, (*arg).x30, &mut Dirty);
        }
        i += 1;
    }

    _tyList_80312904(arg, 0x63);
    _tyList_80313464(arg);
}

/// `static s32 _tyList_8031305C(void* a, TyListState*, s8 movedFlag)`
pub unsafe fn _tyList_8031305C(a: *mut c_void, state: *mut TyListState, movedFlag: i8) -> i32 {
    let mut entry: *mut TyListArg;
    let mut delta: f32;
    let mut i: i32;
    i = ((*state).x29F as i32 > 0) as i32;
    if i != 0 {
        delta = (*state).x2A4;
        if (*state).x2A1 == 0 {
            delta *= -1.0f32;
        }
        entry = (*state).entries.as_mut_ptr();
        i = 0;
        while i < (*state).entryCount as i32 {
            let x2A1: i8 = (*state).x2A1;
            if (x2A1 != 0 || (*entry).x24 != -1) && (x2A1 != 1 || (*state).entryCount as i32 - 1 != (*entry).x24 as i32) {
                _tyList_80312E88(entry, delta);
            }
            entry = entry.add(1);
            i += 1;
        }
        (*state).x29F -= 1;
        if (*state).x29F == 0 {
            i = 0;
            entry = (*state).entries.as_mut_ptr();
            while i < (*state).entryCount as i32 {
                _tyList_80312E88(entry, 999.0f32);
                if (*state).x2A1 == 0 {
                    (*entry).x24 -= 1;
                    if (*entry).x24 < -1 {
                        (*entry).x24 = ((*state).entryCount as i32 - 2) as i8;
                    }
                } else {
                    (*entry).x24 += 1;
                    if (*entry).x24 as i32 >= (*state).entryCount as i32 - 1 {
                        (*entry).x24 = -1;
                    }
                }
                entry = (*entry).links[1];
                i += 1;
            }
            if (*state).x2A1 == 0 {
                (*(*state).x270).idx = (*(*state).x274).idx + 1;
                if (*(*state).x270).idx as i32 >= ext::Toy_GetTrophyTotal() {
                    (*(*state).x270).idx = 0;
                }
                _tyList_80312904((*state).x270, _tyList_804A2D6C.x0C);
                (*state).x278 = (*(*state).x278).links[1];
                (*state).selectedIdx = (*(*state).x278).idx;
                (*state).x270 = (*(*state).x270).links[1];
                (*state).x274 = (*(*state).x274).links[1];
            } else {
                (*(*state).x274).idx = (*(*state).x270).idx - 1;
                if (*(*state).x274).idx < 0 {
                    (*(*state).x274).idx = (ext::Toy_GetTrophyTotal() - 1) as i16;
                }
                _tyList_80312904(entry, _tyList_804A2D6C.x0C);
                (*state).x278 = (*(*state).x278).links[0];
                (*state).selectedIdx = (*(*state).x278).idx;
                (*state).x270 = (*(*state).x270).links[0];
                (*state).x274 = (*(*state).x274).links[0];
            }
            if (*state).x29E > 0 {
                (*state).x29E -= 1;
            }
            if (*state).x29E == 0 {
                ext::HSD_JObjClearFlagsAll((*state).x288, JOBJ_HIDDEN as u32);
                _tyList_80312904((*state).x278, (*(*state).x278).x24);
                if ext::HSD_PadCopyStatus[0].button & HSD_PAD_XY != 0 {
                    (*state).x2A0 = 5;
                } else {
                    (*state).x2A0 = 0;
                }
            }
            if movedFlag != 0 {
                sfxMove();
            }
        }
    }
    (*state).x29F as i32
}

/// `static void _tyList_80313358(TyListState*, s8, s8, s8)`
pub unsafe fn _tyList_80313358(state: *mut TyListState, arg2: i8, arg3: i8, arg4: i8) {
    if arg2 != -1 {
        (*state).x29E = arg2;
        (*state).x2A1 = arg4;
    }

    (*state).x29F = arg3;
    (*state).x2A4 = (*state).x2A8 / arg3 as f32;

    if (*state).x2A1 == 0 {
        let mut i: i32 = 0;
        while i < (*state).entryCount as i32 {
            let entry: *mut TyListArg = &raw mut (*state).entries[i as usize];
            let sub: *mut TyListArg = (*entry).links[0];
            (*entry).x2C = (*sub).x30;
            _tyList_80312904(entry, ((*state).entryCount as i32 + 1) as i8);
            i += 1;
        }
    } else {
        let mut i: i32 = 0;
        while i < (*state).entryCount as i32 {
            let entry: *mut TyListArg = &raw mut (*state).entries[i as usize];
            let sub: *mut TyListArg = (*entry).links[1];
            (*entry).x2C = (*sub).x30;
            _tyList_80312904(entry, ((*state).entryCount as i32 + 1) as i8);
            i += 1;
        }
    }
}

/// `static void _tyList_80313464(TyListArg*)`
pub unsafe fn _tyList_80313464(arg: *mut TyListArg) {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;

    let val: i32 = *ext::Toy_sbss_804D6EDC.add((*arg).idx as usize) as i32;

    ext::Toy_803083D8((*arg).jobjs[2], val);

    if !(*arg).jobjs[1].is_null() {
        ext::HSD_JObjUnref((*arg).jobjs[1]);
        (*arg).jobjs[1] = null_mut();
    }

    if ext::Toy_80304924(val) != 0 {
        (*arg).jobjs[1] = _tyList_80313508((*state).gobj, cs!("ToyFigureListMarkN_Top_joint"), -8.4f32, (*arg).x30, 0.0f32) as *mut HSD_JObj;
    }
}

/// `static void* _tyList_80313508(HSD_GObj* parent, char* symbol_name, float x, float y, float z)`
pub unsafe fn _tyList_80313508(parent: *mut HSD_GObj, symbol_name: *mut i8, x: f32, y: f32, z: f32) -> *mut c_void {
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    let joint: *mut c_void = ext::HSD_ArchiveGetPublicAddress((*archive).archive, symbol_name);

    if joint.is_null() {
        // OSPanic(__FILE__, 823, "*** Can't Get Public Address!(tyList Model)\n")
        panic_at(823);
    } else {
        let jobj: *mut HSD_JObj = ext::HSD_JObjLoadJoint(joint as *mut HSD_Joint);

        if x != 0.0 || y != 0.0 || z != 0.0 {
            HSD_JObjSetTranslateX(&mut *jobj, x, &mut Dirty);
            HSD_JObjSetTranslateY(&mut *jobj, y, &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *jobj, z, &mut Dirty);
        }

        if !parent.is_null() {
            let parentJobj: *mut HSD_JObj = (*parent).hsd_obj as *mut HSD_JObj;
            ext::HSD_JObjAddChild(parentJobj, jobj);
            let mut child: *mut HSD_JObj = (*parentJobj).child;
            while !(*child).next.is_null() {
                child = (*child).next;
            }
            return child as *mut c_void;
        } else {
            let gobj: *mut HSD_GObj = ext::GObj_Create(6, 7, 0);
            ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
            ext::GObj_SetupGXLink(gobj, Some(ext::cb::HSD_GObj_JObjCallback), 0x39, 0);
            return gobj as *mut c_void;
        }
    }
}

/// `static void _tyList_80313774(void)`
pub unsafe fn _tyList_80313774() {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;
    let mode: *mut TyModeState = toy_mode_state();
    let disp: *mut TyDisplayData = ext::Toy_sbss_804D6EE0;
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    let state_tail: *mut TyListGobjEntry = &raw mut _tyList_804A2D6C;
    let mut entry: *mut TyListArg;
    let root_jobj: *mut HSD_JObj;
    let mut pos: f32;
    let mut step: f32;
    let mut idx: i32;
    let entry_count: i32;
    let mut i: i32;
    let trophy_total: i32;

    (*state).selectedIdx = ext::Toy_804A284C[0x12A] as i16;
    (*state).x29B = (*mode).x1 as i8;
    (*state).x29C = (*mode).x2 as i8;

    if ext::Toy_GetTrophyTotal() <= 0xA {
        (*state_tail).x0D = (*mode).x3 as i8;
        (*state_tail).x0C = (*state_tail).x0D;
    } else {
        (*state_tail).x0D = (*mode).x3 as i8;
        (*state_tail).x0C = (*state_tail).x0D;
    }

    trophy_total = ext::Toy_GetTrophyTotal();
    if ext::Toy_GetTrophyTotal() <= 0xA {
        (*state).entryCount = (trophy_total + 2) as i8;
        entry_count = trophy_total + 2;
        idx = ext::Toy_803062BC((*(*(*disp).selected_entry).prev).trophy_id as i32) as i32;
    } else {
        (*state).entryCount = 0xC;
        entry_count = 0xC;
        idx = ext::Toy_803062BC((*(*(*disp).selected_entry).prev).trophy_id as i32) as i32;
    }

    i = 0;
    while i < entry_count {
        entry = &raw mut (*state).entries[i as usize];
        if i == 0 {
            (*entry).links[0] = &raw mut (*state).entries[(entry_count - 1) as usize];
        } else {
            (*entry).links[0] = &raw mut (*state).entries[(i - 1) as usize];
        }
        if i == entry_count - 1 {
            (*entry).links[1] = &raw mut (*state).entries[0];
        } else {
            (*entry).links[1] = &raw mut (*state).entries[(i + 1) as usize];
        }
        i += 1;
    }

    if ext::Toy_GetTrophyTotal() <= 0xA {
        (*state).x278 = &raw mut (*state).entries[1];
        (*state).x270 = &raw mut (*state).entries[0];
        (*state).x274 = &raw mut (*state).entries[((*state).entryCount as i32 - 1) as usize];
    } else {
        (*state).x278 = &raw mut (*state).entries[1];
        (*state).x270 = &raw mut (*state).entries[0];
        (*state).x274 = &raw mut (*state).entries[((*state).entryCount as i32 - 1) as usize];
    }

    (*state).gobj = _tyList_80313508(null_mut(), cs!("ToyFigurePanel_listpos01_joint"), 0.0, 0.0, 0.0) as *mut HSD_GObj;
    root_jobj = (*(*state).gobj).hsd_obj as *mut HSD_JObj;
    HSD_JObjGetTranslationX(&*root_jobj);
    step = HSD_JObjGetTranslationY(&*root_jobj);
    HSD_JObjGetTranslationZ(&*root_jobj);
    step = 5.11f32 - step;
    pos = -step;
    (*state).x2A8 = step;

    entry = &raw mut (*state).entries[0];
    i = 0;
    while i < (*state).entryCount as i32 {
        (*entry).x28 = i;
        (*entry).jobjs[0] = _tyList_80313508((*state).gobj, cs!("ToyFigureListBase_Top_joint"), 0.0, pos, 0.0) as *mut HSD_JObj;
        (*entry).jobjs[2] = HSD_JObjGetChild((*entry).jobjs[0].as_ref());
        ext::Toy_80306A48(
            (*entry).jobjs[0],
            null_mut(),
            cs!("ToyFigureListBase_Top_matanim_joint"),
            null_mut(),
            (*archive).archive,
            0,
        );
        (*entry).texts[0] = ext::HSD_SisLib_803A5ACC(0, _tyList_804D6EE8, 0.0, 0.0, 17.2f32, 640.0, 64.0);
        (*entry).texts[1] = ext::HSD_SisLib_803A5ACC(0, _tyList_804D6EE8, 0.0, 0.0, 17.2f32, 64.0, 64.0);
        (*entry).texts[2] = ext::HSD_SisLib_803A5ACC(0, _tyList_804D6EE8, 0.0, 0.0, 17.2f32, 192.0, 64.0);
        (*entry).x30 = pos;
        (*entry).x24 = (i - 1) as i8;
        (*entry).idx = idx as i16;
        idx += 1;
        if idx >= ext::Toy_GetTrophyTotal() {
            idx = 0;
        }
        pos += step;
        entry = (*entry).links[1];
        i += 1;
    }

    (*state).selectedIdx = (*(*state).x278).idx;
    (*state).jobj = _tyList_80313508((*state).gobj, cs!("ToyFigureListBaseend_Top_joint"), 0.0, (*state).entries[0].x30, 0.0) as *mut HSD_JObj;
    if ext::Toy_GetTrophyTotal() <= 0xA {
        (*state).x288 = _tyList_80313508(
            (*state).gobj,
            cs!("ToyFigureListCursor_Top_joint"),
            0.0,
            (*state).entries[((*state_tail).x0C as i32 + 1) as usize].x30,
            0.0,
        ) as *mut HSD_JObj;
    } else {
        (*state).x288 = _tyList_80313508(
            (*state).gobj,
            cs!("ToyFigureListCursor_Top_joint"),
            0.0,
            (*state).entries[((*state_tail).x0C as i32 + 1) as usize].x30,
            0.0,
        ) as *mut HSD_JObj;
    }

    i = 0;
    while i < (*state_tail).x0C as i32 {
        _tyList_80313358(state, 0, 1, 1);
        _tyList_8031305C(state_tail as *mut c_void, state, 0);
        i += 1;
    }

    entry = &raw mut (*state).entries[0];
    i = 0;
    while i < (*state).entryCount as i32 {
        if (*entry).x24 == (*state_tail).x0C {
            (*state).selectedIdx = (*entry).idx;
            (*state).x278 = entry;
            break;
        }
        entry = entry.add(1);
        i += 1;
    }

    _tyList_80312BAC(state, (*state_tail).x0C);
}

/// `static inline s32 get_repeat_count(void)`
#[inline]
unsafe fn get_repeat_count() -> i32 {
    _tyList_804A2AC0.x29D as i32
}

/// `static inline void tyList_80313BD8_inline(TyListState*, TyListGobjEntry*)`
#[inline]
unsafe fn tyList_80313BD8_inline(state: *mut TyListState, g: *mut TyListGobjEntry) {
    let mut p: *mut TyListArg = (*state).entries.as_mut_ptr();
    let mut i: i32 = 0;
    while i < (*state).entryCount as i32 {
        if (*p).x24 == (*g).x0C {
            (*state).selectedIdx = (*p).idx;
            (*state).x278 = p;
            sfxMove();
            HSD_JObjSetTranslateY(&mut *(*state).x288, (*p).x30, &mut Dirty);
        }
        _tyList_80312904(p, _tyList_804A2D6C.x0C);
        p = p.add(1);
        i += 1;
    }
}

/// `static void _tyList_80313BD8(HSD_GObj*)`
pub unsafe extern "C" fn _tyList_80313BD8(gobj: *mut HSD_GObj) {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;
    let g: *mut TyListGobjEntry = &raw mut _tyList_804A2D6C;
    let mut f30: f32;
    let mut f31: f32;
    let v: i8;

    if ext::Toy_GetTrophyTotal() > 10 {
        if _tyList_8031305C(&raw mut _tyList_804A2D6C as *mut c_void, state, 1) != 0 {
            return;
        }
        if (*state).x2A0 != 0 {
            (*state).x2A0 -= 1;
            return;
        }
        if (*state).x29E != 0 {
            let d: i32 = get_repeat_count();
            let half: i32 = d / 2;
            let mut r: i32;
            let x29E: i32 = (*state).x29E as i32;
            if (d % 2) != 0 {
                if x29E <= half {
                    r = 5 - x29E;
                } else {
                    r = 4 - (d - x29E);
                }
            } else if x29E == half {
                r = 4 - (d - (x29E + 1));
            } else if x29E < half {
                r = 4 - x29E;
            } else {
                r = 4 - (d - x29E);
            }
            if r < 2 {
                r = 2;
            }
            _tyList_80313358(state, -1, r as i8, -1);
            return;
        }
    }

    f30 = ext::Toy_80305D00();
    f31 = ext::Toy_80305DB0();
    if f30 < -0.8f32 {
        (*g).x12 = (*g).x12.wrapping_add(1);
        f30 = -1.0;
        f31 = 0.0;
    } else if f30 > 0.8f32 {
        (*g).x12 = (*g).x12.wrapping_add(1);
        f30 = 1.0;
        f31 = 0.0;
    } else if f31 < -0.6f32 || (ext::Toy_80305C44() & 4) != 0 {
        (*g).x12 = (*g).x12.wrapping_add(1);
        f30 = 0.0;
        f31 = 1.0;
    } else if f31 > 0.6f32 || (ext::Toy_80305C44() & 8) != 0 {
        (*g).x12 = (*g).x12.wrapping_add(1);
        f30 = 0.0;
        f31 = -1.0;
    } else {
        f31 = 0.0;
        (*g).x0F = 0;
        f30 = f31;
        (*g).x12 = 0;
        (*g).x14 = 6;
        (*g).x13 = 0;
        (*g).x11 = 0;
        (*g).x10 = 0;
    }

    if ext::mn_8022F218() != 0 {
        sfxBack();
        ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        ext::Toy_80310660(0);
        _tyList_803148E4(0);
        ext::mn_8022F268();
        (*toy_mode_state()).x4 = 1;
        return;
    }

    if (ext::Toy_80305B88() & 0x200) != 0 {
        sfxBack();
        ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        ext::Toy_80310660(0);
        _tyList_803148E4(0);
        (*toy_mode_state()).x4 = 1;
        return;
    }

    if ext::Toy_GetTrophyTotal() == 0 {
        return;
    }

    if (ext::Toy_80305B88() & 0x1100) != 0 {
        sfxForward();
        ext::HSD_GObjProc_RemoveProc(ext::HSD_GObj_CurrentInvokedProc);
        ext::Toy_80310660(0);
        _tyList_803148E4(1);
        ext::Toy_80310324();
        return;
    }

    if (*g).x12 > (*g).x14 {
        (*g).x12 = (*g).x14.wrapping_add(1);
        (*g).x14 = 3;
        (*g).x0F = 0;
    }

    if (f30 < 0.0f32 && f30 != (*g).x10 as f32) || (ext::Toy_80305B88() & 0x41) != 0 {
        (*g).x10 = f30 as i8;
        (*state).x29B = (*state).x29B.wrapping_sub(1);
        if (*state).x29B < 0 {
            (*state).x29B = 2;
        }
        _tyList_80312BAC(state, (*g).x0C);
        sfxForward();
        return;
    }

    if (f30 > 0.0f32 && f30 != (*g).x10 as f32) || (ext::Toy_80305B88() & 0x22) != 0 {
        (*g).x10 = f30 as i8;
        (*state).x29B = (*state).x29B.wrapping_add(1);
        if (*state).x29B >= 3 {
            (*state).x29B = 0;
        }
        _tyList_80312BAC(state, (*g).x0C);
        sfxForward();
        return;
    }

    if ext::Toy_GetTrophyTotal() < 2 {
        return;
    }

    if ext::Toy_GetTrophyTotal() > 10 {
        if (ext::Toy_80305C44() & 0x400) != 0 {
            ext::HSD_JObjSetFlagsAll((*state).x288, JOBJ_HIDDEN as u32);
            if (*(*state).x274).idx == 0 || (*(*(*state).x274).links[0]).idx as i32 + 9 < ext::Toy_GetTrophyTotal() {
                _tyList_80313358(state, 9, 4, 0);
            } else {
                _tyList_80313358(state, (ext::Toy_GetTrophyTotal() - (*(*state).x274).idx as i32) as i8, 4, 0);
            }
            (*state).x29D = (*state).x29E;
            return;
        }
        if (ext::Toy_80305C44() & 0x800) != 0 {
            ext::HSD_JObjSetFlagsAll((*state).x288, JOBJ_HIDDEN as u32);
            if (*(*state).x270).idx as i32 == ext::Toy_GetTrophyTotal() - 1 || (*(*(*state).x270).links[1]).idx as i32 - 9 > 0 {
                _tyList_80313358(state, 9, 4, 1);
            } else {
                _tyList_80313358(state, (*(*(*state).x270).links[1]).idx as i8, 4, 1);
            }
            (*state).x29D = (*state).x29E;
            return;
        }
    }

    if f31 == 0.0 {
        return;
    }
    if (*g).x0F != 0 {
        return;
    }
    (*g).x0F = 1;
    (*g).x0C = ((*g).x0C as f32 + f31) as i8;
    if (*g).x0C < 0 {
        (*g).x0C = 0;
    } else if (*g).x0C as i32 > (*state).entryCount as i32 - 3 {
        (*g).x0C = ((*state).entryCount as i32 - 3) as i8;
    }

    if (*g).x0C != (*g).x0D {
        (*g).x12 = 0;
        (*g).x13 = 0;
        (*g).x11 = f31 as i8;
        tyList_80313BD8_inline(state, g);
        (*g).x0D = (*g).x0C;
        return;
    }

    if ext::Toy_GetTrophyTotal() <= 10 {
        return;
    }
    if (*g).x11 == 0 {
        (*g).x13 = 5;
        (*g).x12 = (*g).x14.wrapping_add(1);
        (*g).x14 = 3;
        (*g).x0F = 0;
    }
    (*g).x11 = f31 as i8;
    v = (*g).x13;
    if v == 0 {
        (*g).x13 = 1;
        return;
    }
    if v < 5 {
        (*g).x13 = v + 1;
        return;
    }

    {
        let mut i: i32 = 0;
        let mut p: *mut TyListArg = (*state).entries.as_mut_ptr();
        while i < (*state).entryCount as i32 {
            _tyList_80312904(p, (*state).entryCount);
            p = p.add(1);
            i += 1;
        }
    }
    ext::HSD_JObjSetFlagsAll((*state).x288, JOBJ_HIDDEN as u32);
    if f31 > 0.0f32 {
        _tyList_80313358(state, 1, 6, 0);
    } else {
        _tyList_80313358(state, 1, 6, 1);
    }
}

/// `static void _tyList_8031438C(HSD_GObj*)`
pub unsafe extern "C" fn _tyList_8031438C(gobj: *mut HSD_GObj) {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;
    let entry: *mut TyListGobjEntry = &raw mut _tyList_804A2D6C;
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;

    if (*entry).x16 != 0 {
        if (*entry).x16 > 1 {
            let mut i: i32 = 0;
            while i < 3 {
                if i == (*state).x29B as i32 {
                    ext::HSD_JObjReqAnim((*archive).jobjs[i as usize], 1.0);
                } else {
                    ext::HSD_JObjReqAnim((*archive).jobjs[i as usize], 0.0);
                }
                ext::HSD_AObjSetRate((*(*(*(*(*archive).jobjs[0]).u.dobj).mobj).tobj).aobj, 0.0);
                i += 1;
            }
            ext::HSD_JObjAnimAll(*(*archive).x0.add(10));
        } else {
            // `TyListWaitData* wait_data = (TyListWaitData*) entry->x4;` (x20 / x24 = gxlink_prios halves)
            let wait_data: *mut HSD_GObj = (*entry).x4;
            if !wait_data.is_null() {
                (*wait_data).gxlink_prios = 0x4210_0000_0000_0000;
            }
            (*state).x290 = ext::HSD_SisLib_803A6754(3, _tyList_804D6EEC);
            (*(*state).x290).pos_z = 17.2f32;
            let new_var: *mut HSD_Text = (*state).x290;
            (*(*state).x290).font_size.x = 0.038f32;
            (*new_var).font_size.y = 0.029f32;
            (*(*state).x290).default_kerning = 1;
            (*(*state).x290).default_alignment = 2;
            ext::HSD_SisLib_803A6B98((*state).x290, 290.0, 320.0, cs!("%d"), ext::Toy_GetTrophyTotal());
        }
        (*entry).x16 -= 1;
        return;
    }

    ext::HSD_GObjProc_RemoveAllProcs(gobj);
    ext::HSD_GObj_SetupProc((*entry).x0, Some(_tyList_80313BD8), 0);
    ext::HSD_GObj_80390CD4((*entry).x0);
}

/// `static void _tyList_80314504(HSD_GObj*, int)`
pub unsafe extern "C" fn _tyList_80314504(gobj: *mut HSD_GObj, unused: i32) {
    // `TyListData* data = (TyListData*) gobj;` (`cobj` at +0x28 is `hsd_obj`)
    let cobj = (*gobj).hsd_obj as *mut HSD_CObj;

    if ext::HSD_CObjSetCurrent(cobj) != 0 {
        ext::HSD_SetEraseColor(0, 0, 0, 0xFF);
        ext::HSD_CObjEraseScreen(cobj, 1, 0, 0);
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_FogSet(null_mut());
        ext::HSD_CObjEndCurrent();
    }
}

/// `static void _tyList_8031457C(void)`
pub unsafe fn _tyList_8031457C() {
    let entry: *mut TyListGobjEntry = &raw mut _tyList_804A2D6C;
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    let mut viewport: HSD_RectS16 = core::mem::zeroed();
    let mut scissor: Scissor = core::mem::zeroed();
    let mut interest: Vec = core::mem::zeroed();
    let mut eye: Vec = core::mem::zeroed();

    // @todo (C) Is this really a reference to Toy_str_ScMenFigure_cam_int1_camera, or de-duplicated?
    let desc: *mut HSD_CameraDescPerspective =
        ext::HSD_ArchiveGetPublicAddress((*archive).archive, ext::Toy_str_ScMenFigure_cam_int1_camera.as_ptr()) as *mut HSD_CameraDescPerspective;
    if !desc.is_null() {
        (*entry).x0 = ext::GObj_Create(1, 2, 0);
        let cobj = ext::lb_80013B14(desc);
        let kind: u8 = ext::HSD_GObj_CameraKind;
        ext::HSD_GObjObject_80390A70((*entry).x0, kind, cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax((*entry).x0, Some(ext::cb::Toy_80306954), 0);
        (*(*entry).x0).gxlink_prios = 0x9010000000000000u64;
        ext::HSD_GObj_SetupProc((*entry).x0, Some(_tyList_8031438C), 0);
        ext::HSD_GObj_80390CD4((*entry).x0);
        (*entry).x16 = 0x1A;
    }
    _tyList_804D6EEC = ext::HSD_SisLib_803A611C(3, (*entry).x0, 0xC, 0xC, 0, 0x3F, 0, 0);
    if !desc.is_null() {
        (*entry).x4 = ext::GObj_Create(1, 2, 0);
        let cobj = ext::lb_80013B14(desc);
        let kind: u8 = ext::HSD_GObj_CameraKind;
        ext::HSD_GObjObject_80390A70((*entry).x4, kind, cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax((*entry).x4, Some(_tyList_80314504), 0);
        (*(*entry).x4).gxlink_prios = 0x0210000000000000u64;
        interest.x = 1.1f32;
        interest.y = -0.24f32;
        interest.z = 0.0;
        ext::HSD_CObjSetInterest(cobj, &mut interest);
        eye.x = 1.1f32;
        eye.y = -0.24f32;
        eye.z = 40.0596313f32;
        ext::HSD_CObjSetEyePosition(cobj, &mut eye);
        viewport.xmin = 0x76;
        viewport.ymin = 0x4E;
        viewport.xmax = 0x230;
        viewport.ymax = 0x19C;
        ext::HSD_CObjSetViewport(cobj, &mut viewport);
        viewport.ymin = 0x60;
        if ext::Toy_GetTrophyTotal() < 10 {
            viewport.ymax = (ext::Toy_GetTrophyTotal() * 0x1E + 0x60) as i16;
        } else {
            viewport.ymax = 0x18A;
        }
        scissor.left = viewport.xmin as u16;
        scissor.right = viewport.xmax as u16;
        scissor.top = viewport.ymin as u16;
        scissor.bottom = viewport.ymax as u16;
        ext::HSD_CObjSetScissor(cobj, &mut scissor);
        _tyList_804D6EE8 = ext::HSD_SisLib_803A611C(0, (*entry).x4, 0xB, 0xB, 0, 0x3E, 0, 0);
    }
}

/// `void tyList_803147C4(void)`
pub unsafe fn tyList_803147C4() {
    tyList_pc_active = 1;
    ext::memzero(&raw mut _tyList_804A2D6C as *mut c_void, core::mem::size_of::<TyListGobjEntry>() as i32);
    _tyList_8031457C();
    ext::memzero(&raw mut _tyList_804A2D84 as *mut c_void, core::mem::size_of::<[*mut HSD_GObj; 5]>() as i32);
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    let gobj: *mut *mut HSD_GObj = (&raw mut _tyList_804A2D84) as *mut *mut HSD_GObj;

    if (*archive).archive.is_null() {
        ext::OSReport("*** BG data aren't being loaded!\n");
        // OSPanic(__FILE__, 1590, "")
        panic_at(1590);
    }

    let jobj: *mut *mut LightList = ext::HSD_ArchiveGetPublicAddress((*archive).archive, cs!("ScMenFigure_scene_lights")) as *mut *mut LightList;
    if !jobj.is_null() {
        *gobj = ext::GObj_Create(2, 3, 0);
        let new_var: u8 = ext::HSD_GObj_LightKind as u8;
        ext::HSD_GObjObject_80390A70(*gobj, new_var, ext::Toy_LoadLObjList(jobj, null_mut()) as *mut c_void);
        ext::GObj_SetupGXLink(*gobj, Some(ext::cb::HSD_GObj_LObjCallback), 52, 0);
    }

    ext::Toy_80307470(0);
    if ext::Toy_GetTrophyTotal() != 0 {
        ext::memzero(&raw mut _tyList_804A2AC0 as *mut c_void, core::mem::size_of::<TyListState>() as i32);
        _tyList_80313774();
    }
    ext::HSD_PadRenewStatus();
}

/// `static void _tyList_803148E4(s32)`
pub unsafe fn _tyList_803148E4(arg0: i32) {
    let state: *mut TyListState = &raw mut _tyList_804A2AC0;
    let entry: *mut TyListGobjEntry = &raw mut _tyList_804A2D6C;
    let gobj_2C4: *mut *mut HSD_GObj = (&raw mut _tyList_804A2D84) as *mut *mut HSD_GObj;
    let archive: *mut ToyED8Data = ext::Toy_sbss_804D6ED8;
    tyList_pc_active = 0;

    if ext::Toy_GetTrophyTotal() != 0 {
        if arg0 != 0 {
            let mode: *mut TyModeState = toy_mode_state();
            ext::Toy_804A284C[0x12A] = (*state).selectedIdx as u16;
            ext::Toy_804A284C[0x12B] = *ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize) as u16;
            (*mode).x1 = (*state).x29B as u8;
            (*mode).x2 = (*state).x29C as u8;
            (*mode).x3 = (*entry).x0C as u8;
        } else {
            if ext::Toy_GetTrophyTotal() != 0 {
                let val: i16 = *ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize);
                ext::Toy_803067BC(0, 0);
                (*state).selectedIdx = ext::Toy_803062BC(val as i32);
            }
            {
                let mode: *mut TyModeState = toy_mode_state();
                ext::Toy_804A284C[0x12A] = (*state).selectedIdx as u16;
                ext::Toy_804A284C[0x12B] = *ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize) as u16;
                (*mode).x1 = 0;
                (*mode).x2 = 0;
                (*mode).x3 = 0;
            }
        }

        if ext::Toy_80304924(*ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize) as i32) != 0 {
            ext::Toy_80304988(*ext::Toy_sbss_804D6EDC.add((*state).selectedIdx as usize) as i32);
        }
    }

    if !(*state).gobj.is_null() {
        if arg0 != 0 {
            ext::HSD_GObjFree((*state).gobj);
        }
        (*state).gobj = null_mut();
    }

    if arg0 != 0 {
        ext::HSD_SisLib_803A5E70();
    }

    if !(*archive).gobj.is_null() {
        if arg0 != 0 {
            ext::HSD_GObjFree((*archive).gobj);
        }
        (*archive).gobj = null_mut();
    }

    if !(*gobj_2C4).is_null() && arg0 != 0 {
        ext::HSD_GObjFree(*gobj_2C4);
    }

    if !(*entry).x0.is_null() {
        ext::HSD_GObjProc_RemoveAllProcs((*entry).x0);
        if arg0 != 0 {
            ext::HSD_GObjFree((*entry).x0);
        }
    }

    if !(*entry).x4.is_null() && arg0 != 0 {
        ext::HSD_GObjFree((*entry).x4);
    }
}
