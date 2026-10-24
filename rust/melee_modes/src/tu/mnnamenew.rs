//! src/melee/mn/mnnamenew.c: the name-entry screen (Data > Name > New / Edit, and the CSS name
//! tag editor): the on-screen keyboard (hiragana / katakana / latin tables), glyph variants
//! (dakuten, small kana), cursor and key animation, auto-name picking and the scene builders.
//! The glyph tables hold UTF-8 byte strings. Callees in TUs that are not ported yet are `ext`
//! stubs named like the C function.
//!
//! The retail build reads the anim settings, the key map and the glyph table as one
//! `MnNameNewDataLayout` through the address of the first; the PC build (and this port)
//! reaches each real object directly (`mnNameNew_803EDA58`, `mnNameNew_KeyMap`,
//! `mnNameNew_GlyphTable`, `unk_vec`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `typedef volatile char GlyphChar` (volatility has no meaning here).
pub type GlyphChar = c_char;
/// `typedef char* GlyphRow[4]`.
pub type GlyphRow = [*mut c_char; 4];

/// `MnNameNewKeyMap` (mnnamenew.c): keyboard cursor position -> joint index and glyph strings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnNameNewKeyMap {
    pub key_jobj_ids: [u16; 8],
    pub x34: [*mut GlyphChar; 50],
    pub xFC: [*mut GlyphChar; 50],
    pub character_bytes: [*mut GlyphChar; 50],
}

melee_core::gc_layout!(MnNameNewKeyMap, "MnNameNewKeyMap", size 0x268, align 4, [
    key_jobj_ids @ 0x0: 16 "u16[8]",
    x34 @ 0x10: 200 "GlyphChar*[50]",
    xFC @ 0xD8: 200 "GlyphChar*[50]",
    character_bytes @ 0x1A0: 200 "GlyphChar*[50]",
]);

/// `MnNameNewGlyphTable` (mnnamenew.c): lowercase / uppercase glyph variant rows of each key.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnNameNewGlyphTable {
    pub lower_glyphs: [[*mut GlyphChar; 4]; 50],
    pub upper_glyphs: [[*mut GlyphChar; 4]; 50],
}

melee_core::gc_layout!(MnNameNewGlyphTable, "MnNameNewGlyphTable", size 0x640, align 4, [
    lower_glyphs @ 0x0: 800 "GlyphChar*[50][4]",
    upper_glyphs @ 0x320: 800 "GlyphChar*[50][4]",
]);

/// `MnNameNewDataLayout` (mnnamenew.c): the retail contiguous data run the two tables above and
/// the anim settings are views of.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MnNameNewDataLayout {
    pub anim: [AnimLoopSettings; 3],
    pub key_jobj_ids: [u16; 8],
    pub x34: [*mut c_char; 50],
    pub xFC: [*mut c_char; 50],
    pub character_bytes: [*mut c_char; 50],
    pub lower_glyphs: [[*mut c_char; 4]; 50],
    pub upper_glyphs: [[*mut c_char; 4]; 50],
    pub x8CC: Vec3,
    pub x8D8: Vec3,
}

melee_core::gc_layout!(MnNameNewDataLayout, "MnNameNewDataLayout", size 0x8E4, align 4, [
    anim @ 0x0: 36 "AnimLoopSettings[3]",
    key_jobj_ids @ 0x24: 16 "u16[8]",
    x34 @ 0x34: 200 "char*[50]",
    xFC @ 0xFC: 200 "char*[50]",
    character_bytes @ 0x1C4: 200 "char*[50]",
    lower_glyphs @ 0x28C: 800 "GlyphRow[50]",
    upper_glyphs @ 0x5AC: 800 "GlyphRow[50]",
    x8CC @ 0x8CC: 12 "Vec3",
    x8D8 @ 0x8D8: 12 "Vec3",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn mn_802295AC() -> u8 = crate::tu::adapt::mn_802295AC__671d2a;
        fn mn_80229B2C() -> *mut HSD_GObj = crate::tu::adapt::mn_80229B2C__cb2183;
        fn mn_80229DC0() -> *mut HSD_GObj = crate::tu::adapt::mn_80229DC0__35b68e;
        fn mn_8022BCF8() -> *mut HSD_GObj = crate::tu::adapt::mn_8022BCF8__af4c64;
        fn mn_8022BE34() -> *mut HSD_GObj = crate::tu::adapt::mn_8022BE34__0ceef2;
        fn mn_8022BEDC(a0: *mut HSD_GObj) = crate::tu::adapt::mn_8022BEDC__95f428;
        fn mn_8022C304() = crate::tu::adapt::mn_8022C304__3bbc69;
        fn mn_8022EBDC() = crate::tu::adapt::mn_8022EBDC__a19b61;
        fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022ED6C__f73cae;
        fn mn_8022EFD8(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022EFD8__c1f371;
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32 = crate::tu::adapt::mn_8022F298__0e9b78;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjAddChild(jobj: *mut HSD_JObj, child: *mut HSD_JObj);
        fn HSD_JObjReqAnim(jobj: *mut HSD_JObj, frame: f32);
        fn HSD_JObjAnim(jobj: *mut HSD_JObj);
        fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, arg1: i32, arg2: *mut GXColor);
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn HSD_SisLib_803A5E70() = crate::tu::adapt::HSD_SisLib_803A5E70__5f9c7d;
        fn mnCharSel_802640A0() -> i32 = crate::tu::adapt::mnCharSel_802640A0__c8d517;
        fn GetNameCount() -> i32 = crate::tu::adapt::GetNameCount__b961fe;
        fn IsNameUnique(name: *mut c_char) -> i32 = crate::tu::adapt::IsNameUnique__f9fddb;
        fn CreateNameAtIndex(slot: i32) = crate::tu::adapt::CreateNameAtIndex__5eec5d;
        fn mnName_80239EBC(jobj: *mut HSD_JObj, y: f32) = crate::tu::adapt::mnName_80239EBC__737a0b;
        fn mnName_80239F5C(jobj: *mut HSD_JObj, x: f32) = crate::tu::adapt::mnName_80239F5C__711f9f;
        fn mnName_8023A9B4(a0: u8) = crate::tu::adapt::mnName_8023A9B4__53fde4;
        fn IsNameNotAllowed(name_idx: *mut c_char) -> i32 = crate::tu::adapt::IsNameNotAllowed__c0ffe3;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn GetPersistentNameData(a0: i32) -> *mut NameTagData = crate::tu::gmmain_lib::GetPersistentNameData;
        fn gmMainLib_8015ECB0() -> i32 = crate::tu::gmmain_lib::gmMainLib_8015ECB0;
        fn GetRumbleSettingOfPort(port: isize) -> i32 = crate::tu::gmmain_lib::GetRumbleSettingOfPort;
        fn gm_80190FE4(a0: i32) = crate::tu::adapt::gm_80190FE4__f31b7e;
        fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) = crate::tu::adapt::lb_8000B1CC__69851d;
        fn lbCardGame_SaveChanges() = crate::tu::adapt::lbCardGame_SaveChanges__0fc9e6;
        fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageUS__90e41e;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc);
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        extern "C" fn HSD_Free(ptr: *mut c_void) = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_MemAlloc(size: isize) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__849f1b;
        fn HSD_Randi(max_val: i32) -> i32;
    }
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut MenMainBack_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMainPanel_Top: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMain_lights: *mut *mut LightList = unsafe { core::mem::zeroed() };
    pub static mut MenMain_fog: *mut HSD_FogDesc = unsafe { core::mem::zeroed() };
    pub static mut MenMain_cam: *mut HSD_CObjDesc = unsafe { core::mem::zeroed() };
    pub static mut NotAllowedNamesList: *mut *mut c_char = unsafe { core::mem::zeroed() };
    pub static mut AutoNamesList: *mut *mut u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_SisLib_804D1124: [*mut SIS; 5] = unsafe { core::mem::zeroed() };
    pub static mut mnNameNew_804A06F0: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut mnNameNew_804A0700: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut mnNameNew_804A0710: StaticModelDesc = unsafe { core::mem::zeroed() };
    pub static mut mnNameNew_804A0720: [StaticModelDesc; 2] = unsafe { core::mem::zeroed() };
}
use ext::*;

pub static mut mnNameNew_PortInUse: u8 = 0;
pub static mut mnNameNew_CurrentNameText: [c_char; 16] = [0; 16];
pub static mut mnNameNew_804D6C08: *mut HSD_GObj = null_mut();
static mut mnNameNew_803EDA58: [AnimLoopSettings; 3] = [
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 19.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 20.0_f32,
        end_frame: 39.0_f32,
        loop_frame: -0.1_f32,
    },
    AnimLoopSettings {
        start_frame: 0.0_f32,
        end_frame: 10.0_f32,
        loop_frame: -0.1_f32,
    },
];
static mut mnNameNew_804D4C10: f32 = 10.0_f32;
static mut mnNameNew_KeyMap: MnNameNewKeyMap = MnNameNewKeyMap {
    key_jobj_ids: [
        3 as i32 as u16,
        10 as i32 as u16,
        11 as i32 as u16,
        7 as i32 as u16,
        9 as i32 as u16,
        8 as i32 as u16,
        5 as i32 as u16,
        5 as i32 as u16,
    ],
    x34: [
        c"\xE3\x81\x82".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x84".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x86".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x88".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x8A".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x8B".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x8D".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x8F".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x91".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x93".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x95".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x97".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x99".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x9B".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x9D".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\x9F".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xA1".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xA4".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xA6".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xA8".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAA".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAB".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAC".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAD".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAE".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xAF".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xB2".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xB5".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xB8".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xBB".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xBE".as_ptr() as *mut GlyphChar,
        c"\xE3\x81\xBF".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x80".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x81".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x82".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x84".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x86".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x88".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x89".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x8A".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x8B".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x8C".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x8D".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x8F".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x92".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\x93".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xBC".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x82".as_ptr() as *mut GlyphChar,
    ],
    xFC: [
        c"\xE3\x82\xA2".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xA4".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xA6".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xA8".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xAA".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xAB".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xAD".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xAF".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xB1".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xB3".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xB5".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xB7".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xB9".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xBB".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xBD".as_ptr() as *mut GlyphChar,
        c"\xE3\x82\xBF".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x81".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x84".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x86".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x88".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8A".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8B".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8C".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8D".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8E".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x8F".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x92".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x95".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x98".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x9B".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x9E".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\x9F".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA0".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA1".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA2".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA4".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA6".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA8".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xA9".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xAA".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xAB".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xAC".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xAD".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xAF".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xB2".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xB3".as_ptr() as *mut GlyphChar,
        c"\xE3\x83\xBC".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x82".as_ptr() as *mut GlyphChar,
    ],
    character_bytes: [
        c"\xEF\xBC\xAA".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB4".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x99".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x8E".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA9".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB3".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x98".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x84".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA8".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB2".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x97".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x86".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA7".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB1".as_ptr() as *mut GlyphChar,
        c"\xE3\x80\x80".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x96".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x85".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA6".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB0".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xBA".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x95".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA0".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA5".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xAF".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB9".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x94".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x9F".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA4".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xAE".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB8".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x93".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x81".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA3".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xAD".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB7".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x92".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x9D".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA2".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xAC".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB6".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x91".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x8B".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xA1".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xAB".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\xB5".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x90".as_ptr() as *mut GlyphChar,
        c"\xEF\xBC\x8D".as_ptr() as *mut GlyphChar,
    ],
};
pub static mut mnNameNew_NullCharacter: [c_char; 1] = unsafe {
    core::mem::transmute::<[u8; 1], [c_char; 1]>(*b"\0")
};
static mut mnNameNew_GlyphTable: MnNameNewGlyphTable = unsafe {
    MnNameNewGlyphTable {
        lower_glyphs: [
            [
                c"\xE3\x81\x82".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x81".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x84".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x83".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x86".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x85".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x88".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x87".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x8A".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x89".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x8B".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x8C".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x8D".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x8E".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x8F".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x90".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x91".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x92".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x93".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x94".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x95".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x96".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x97".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x98".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x99".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x9A".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x9B".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x9C".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x9D".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\x9E".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\x9F".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA0".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xA1".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA2".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xA4".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA5".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA3".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xA6".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA7".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xA8".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xA9".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAA".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAB".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAC".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAD".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAE".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xAF".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB0".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB1".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xB2".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB3".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB4".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xB5".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB6".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB7".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xB8".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xB9".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xBA".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xBB".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xBC".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x81\xBD".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x81\xBE".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x81\xBF".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x80".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x81".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x82".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x84".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\x83".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x86".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\x85".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x88".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\x87".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x80\x80".as_ptr()
                    as *mut GlyphChar,
                c" ".as_ptr() as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x80\x80".as_ptr()
                    as *mut GlyphChar,
                c" ".as_ptr() as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\x89".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x8A".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x8B".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x8C".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x8D".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x8F".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x92".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x82\x93".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xBC".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x80\x82".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
        ],
        upper_glyphs: [
            [
                c"\xE3\x82\xA2".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xA1".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xA4".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xA3".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xA6".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xA5".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xA8".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xA7".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xAA".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xA9".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xAB".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xAC".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xAD".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xAE".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xAF".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xB0".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xB1".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xB2".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xB3".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xB4".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xB5".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xB6".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xB7".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xB8".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xB9".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xBA".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xBB".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xBC".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xBD".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x82\xBE".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x82\xBF".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x80".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x81".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x82".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x84".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x85".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x83".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x86".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x87".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x88".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x89".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8A".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8B".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8C".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8D".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8E".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x8F".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x90".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x91".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x92".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x93".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x94".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x95".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x96".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x97".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x98".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x99".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x9A".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x9B".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x9C".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\x9D".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
            ],
            [
                c"\xE3\x83\x9E".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\x9F".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA0".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA1".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA2".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA4".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\xA3".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA6".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\xA5".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA8".as_ptr()
                    as *mut GlyphChar,
                c"\xE3\x83\xA7".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x80\x80".as_ptr()
                    as *mut GlyphChar,
                c" ".as_ptr() as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x80\x80".as_ptr()
                    as *mut GlyphChar,
                c" ".as_ptr() as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
            ],
            [
                c"\xE3\x83\xA9".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xAA".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xAB".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xAC".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xAD".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xAF".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xB2".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x83\xB3".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xEF\xBD\x9E".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
            [
                c"\xE3\x80\x81".as_ptr()
                    as *mut GlyphChar,
                &raw const mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar,
                null_mut(),
                null_mut(),
            ],
        ],
    }
};
pub static mut unk_vec: Vec3 = Vec3 {
    x: -0.8_f32,
    y: 0.4_f32,
    z: 0.0_f32,
};
static mut mnNameNew_803EE330: Vec3 = Vec3 {
    x: -0.7_f32,
    y: 0.7_f32,
    z: 0.0_f32,
};
pub unsafe extern "C" fn mnNameNew_8023B0F8(mut arg0: *mut HSD_GObj, mut arg1: u8) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut data: *mut NameNewEntry = null_mut();
    data = (*arg0).user_data as *mut NameNewEntry;
    jobj = (*data).jobjs[10 as i32 as usize] as *mut HSD_JObj;
    if arg1 as i32 == 0x33 {
        HSD_JObjReqAnimAll(jobj, 1.0_f32);
    } else if (*data).mode as i32 == 0 {
        HSD_JObjReqAnimAll(jobj, 2.0_f32);
    } else {
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
    }
    HSD_JObjAnimAll(jobj);
    jobj = (*data).jobjs[11 as i32 as usize] as *mut HSD_JObj;
    if arg1 as i32 == 0x34 {
        HSD_JObjReqAnimAll(jobj, 1.0_f32);
    } else if (*data).mode as i32 == 1 {
        HSD_JObjReqAnimAll(jobj, 2.0_f32);
    } else {
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
    }
    HSD_JObjAnimAll(jobj);
    jobj = (*data).jobjs[7 as i32 as usize] as *mut HSD_JObj;
    if arg1 as i32 == 0x35 {
        HSD_JObjReqAnimAll(jobj, 1.0_f32);
    } else if (*data).mode as i32 == 2 {
        HSD_JObjReqAnimAll(jobj, 2.0_f32);
    } else {
        HSD_JObjReqAnimAll(jobj, 0.0_f32);
    }
    HSD_JObjAnimAll(jobj);
}
pub unsafe extern "C" fn mnNameNew_8023B224(mut arg0: u8) {
    let mut data: *mut NameNewEntry = null_mut();
    let mut name_index: u8 = 0;
    data = (*mnNameNew_804D6C08).user_data as *mut NameNewEntry;
    name_index = (*data).name_index;
    if arg0 as i32 != 0 {
        lbCardGame_SaveChanges();
    }
    if gm_GetCurrentGameMode() as i32
        == GM_TOURNAMENT as i32
    {
        HSD_SisLib_803A5E70();
        mn_8022EBDC();
        if arg0 as i32 != 0 {
            gm_80190FE4(name_index as i32);
            return;
        }
        gm_80190FE4(0x78 as i32);
        return;
    }
    if gm_GetCurrentGameMode() as i32 == GM_MENU as i32 {
        mn_804D6BC8.cooldown = 5;
        if arg0 as i32 != 0
            && GetNameCount() > 0x18
        {
            mnName_8023A9B4(
                ((*data).name_index as i32 / 6 as i32)
                    as u8,
            );
            return;
        }
        mnName_8023A9B4(0 as u8);
        return;
    }
    HSD_SisLib_803A5E70();
    mn_8022EBDC();
    mnCharSel_802640A0();
}
pub unsafe extern "C" fn mnNameNew_8023B314(mut arg0: *mut NameNewEntry, mut arg1: i32) {
    let mut text: *mut HSD_Text = null_mut();
    let mut idx: u8 = 0;
    let mut selection: i32 = 0;
    selection = arg1;
    if selection < 0x32 {
        match (*arg0).mode as i32 {
            0 => {
                selection = 0x33 as i32;
            }
            1 => {
                selection = 0x34 as i32;
            }
            2 => {
                selection = 0x35 as i32;
            }
            _ => {}
        }
    }
    text = (*arg0).desc_text;
    idx = mnNameNew_804D4F7C[(selection as i32
        - 0x32 as i32) as usize];
    if !text.is_null() {
        if (*text).sis_buffer
            == *(HSD_SisLib_804D1124[0 as i32 as usize] as *mut *mut SIS)
                .offset(idx as isize)
        {
            return;
        }
        HSD_SisLib_803A5CC4((*arg0).desc_text);
    }
    text = HSD_SisLib_803A5ACC(
        0,
        mn_804D6BB4 as i32,
        -9.5_f32,
        9.1_f32,
        17.0_f32,
        364.68332_f32,
        38.38772_f32,
    );
    (*arg0).desc_text = text;
    (*text).font_size.x = 0.0521_f32 as f32;
    (*text).font_size.y = 0.0521_f32 as f32;
    HSD_SisLib_803A6368(text, idx as i32);
}
static mut mnNameNew_804D4F68: GXColor = _GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
static mut mnNameNew_804D4F6C: GXColor = _GXColor {
    r: 0xa6 as u8,
    g: 0x81 as u8,
    b: 0x3d as u8,
    a: 0xff as u8,
};
static mut mnNameNew_804D4F70: GXColor = _GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
static mut mnNameNew_804D4F74: GXColor = _GXColor {
    r: 0x74 as u8,
    g: 0x4f as u8,
    b: 0xb as u8,
    a: 0xff as u8,
};
static mut mnNameNew_804D4F78: GXColor = _GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
pub static mut mnNameNew_804D4F7C: [u8; 8] = [
    'B' as i32 as u8,
    'C' as i32 as u8,
    'D' as i32 as u8,
    'E' as i32 as u8,
    'F' as i32 as u8,
    'G' as i32 as u8,
    'H' as i32 as u8,
    'H' as i32 as u8,
];
pub static mut mnNameNew_804DBF44: GXColor = _GXColor {
    r: 0xa6 as u8,
    g: 0x81 as u8,
    b: 0x3d as u8,
    a: 0xff as u8,
};
pub static mut mnNameNew_804DBF48: GXColor = _GXColor {
    r: 0 as u8,
    g: 0 as u8,
    b: 0 as u8,
    a: 0xff as u8,
};
unsafe extern "C" fn mnNameNew_SetKeyColor(
    mut text: *mut HSD_Text,
    mut index: i32,
    mut color_ptr: *mut GXColor,
    mut color: *mut GXColor,
) {
    *color = *color_ptr;
    color_ptr = color;
    HSD_SisLib_803A74F0(text, index, color_ptr);
}
pub unsafe extern "C" fn mnNameNew_KeySetup(
    mut arg0: *mut NameNewEntry,
    mut arg1: u8,
) -> *mut HSD_Text {
    let mut text_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut selected_key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    let mut j: i32 = 0;
    let mut text: *mut HSD_Text = null_mut();
    let mut ref1: *mut HSD_JObj = null_mut();
    let mut ref2: *mut HSD_JObj = null_mut();
    let mut str_table: *mut *mut c_char = core::ptr::null_mut::<
        *mut c_char,
    >();
    let mut x_range: f32 = 0.;
    let mut y_range: f32 = 0.;
    let mut pos_x: f32 = 0.;
    let mut pos_y: f32 = 0.;
    let mut pos_z: f32 = 0.;
    let mut base_x: f32 = 0.;
    let mut base_y: f32 = 0.;
    let mut font_x: f32 = 0.;
    let mut col_x: f32 = 0.;
    let mut i: i32 = 0;
    let mut key_jobj: *mut HSD_JObj = null_mut();
    let mut color_ptr: *mut GXColor = null_mut();
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    key_color = mnNameNew_804DBF44;
    selected_key_color = mnNameNew_804DBF48;
    match arg1 as i32 {
        0 => {
            (*arg0).mode = 0;
            str_table = &raw mut mnNameNew_KeyMap.x34 as *mut *mut GlyphChar
                as *mut *mut c_char;
        }
        1 => {
            (*arg0).mode = 1;
            str_table = &raw mut mnNameNew_KeyMap.xFC as *mut *mut GlyphChar
                as *mut *mut c_char;
        }
        2 => {
            (*arg0).mode = 2;
            str_table = &raw mut mnNameNew_KeyMap.character_bytes as *mut *mut GlyphChar
                as *mut *mut c_char;
        }
        _ => {}
    }
    if !(*arg0).key_text.is_null() {
        HSD_SisLib_803A5CC4((*arg0).key_text);
    }
    text = HSD_SisLib_803A6754(
        0,
        mn_804D6BB5 as i32,
    );
    (*arg0).key_text = text;
    key_jobj = (*arg0).jobjs[16 as i32 as usize] as *mut HSD_JObj;
    if key_jobj.is_null() {
        key_jobj = null_mut();
    } else {
        key_jobj = (*key_jobj).child as *mut HSD_JObj;
    }
    i = 0 as i32;
    while i < 50 {
        if i == 0x2d {
            break;
        }
        if key_jobj.is_null() {
            key_jobj = null_mut();
        } else {
            key_jobj = (*key_jobj).next as *mut HSD_JObj;
        }
        i += 1;
    }
    lb_8000B1CC(key_jobj, &raw mut unk_vec, &raw mut text_pos);
    pos_x = text_pos.x;
    pos_y = -text_pos.y;
    pos_z = text_pos.z;
    (*text).pos_x = pos_x;
    (*text).pos_y = pos_y;
    (*text).pos_z = pos_z;
    (*text).font_size.x = 0.03_f32 as f32;
    (*text).font_size.y = 0.04_f32 as f32;
    (*text).text_color = mnNameNew_804D4F6C;
    ref1 = (*arg0).jobjs[16 as i32 as usize] as *mut HSD_JObj;
    if ref1.is_null() {
        ref1 = null_mut();
    } else {
        ref1 = (*ref1).child as *mut HSD_JObj;
    }
    i = 0 as i32;
    while i < 50 {
        if i == 0x28 {
            break;
        }
        if ref1.is_null() {
            ref1 = null_mut();
        } else {
            ref1 = (*ref1).next as *mut HSD_JObj;
        }
        i += 1;
    }
    base_x = HSD_JObjGetTranslationX(key_jobj);
    x_range = HSD_JObjGetTranslationX(ref1) - base_x;
    ref2 = (*arg0).jobjs[16 as i32 as usize] as *mut HSD_JObj;
    if ref2.is_null() {
        ref2 = null_mut();
    } else {
        ref2 = (*ref2).child as *mut HSD_JObj;
    }
    i = 0 as i32;
    while i < 50 {
        if i == 0x2e {
            break;
        }
        if ref2.is_null() {
            ref2 = null_mut();
        } else {
            ref2 = (*ref2).next as *mut HSD_JObj;
        }
        i += 1;
    }
    base_y = HSD_JObjGetTranslationY(key_jobj);
    y_range = -(HSD_JObjGetTranslationY(ref2) - base_y);
    j = 0 as i32;
    while j < 0x32 {
        font_x = (*text).font_size.x;
        col_x = (9 as i32
            - j as i32 / 5 as i32) as f32 * x_range;
        HSD_SisLib_803A6B98(text, col_x as f32 / font_x as f32, (j as i32 % 5 as i32) as f32
                * y_range as f32
                / (*text).font_size.y as f32, *str_table.offset(j as isize), &[]);
        if j == mn_804A04F0.hovered_selection as i32 {
            color_ptr = &raw mut selected_key_color;
        } else {
            color_ptr = &raw mut key_color;
        }
        mnNameNew_SetKeyColor(text, j, color_ptr, &raw mut color);
        j += 1;
    }
    return text;
}
pub unsafe extern "C" fn mnNameNew_8023BAA8(
    mut arg0: *mut NameNewEntry,
    mut arg1: i32,
    mut arg2: u8,
) -> i32 {
    let mut mode: u8 = 0;
    if arg2 as u32 >= 0x32
        && (arg2 as u32) < 0x3a
    {
        if arg1 as i32 & 1 as i32 != 0 {
            if arg2 as i32 == 0x39 {
                return (*arg0).last_key_sel as i32 / 5 as i32 * 5 as i32 + 4 as i32;
            }
            if arg2 as u32 > 0x32 {
                return arg2 as i32 - 1 as i32;
            }
            return 0x38 as i32;
        }
        if arg1 as i32 & 2 as i32 != 0 {
            if arg2 as i32 == 0x39 {
                return (*arg0).last_key_sel as i32 / 5 as i32 * 5 as i32;
            }
            if arg2 as i32 == 0x38 {
                return 0x32 as i32;
            }
            return arg2 as i32 + 1 as i32;
        }
        if arg1 as i32 & 4 as i32 != 0 {
            if arg2 as i32 != 0x38
                && arg2 as i32 != 0x39
            {
                return (*arg0).last_key_sel as i32 % 5 as i32;
            }
        } else if arg1 as i32 & 8 as i32 != 0
            && arg2 as i32 != 0x38
            && arg2 as i32 != 0x39
        {
            return (*arg0).last_key_sel as i32 % 5 as i32 + 0x2d as i32
        }
    } else {
        if arg1 as i32 & 1 as i32 != 0 {
            if arg2 as i32 % 5 as i32
                != 0
            {
                return arg2 as i32 - 1 as i32;
            }
            return 0x39 as i32;
        }
        if arg1 as i32 & 2 as i32 != 0 {
            if arg2 as i32 % 5 as i32
                == 4
            {
                return 0x39 as i32;
            }
            return arg2 as i32 + 1 as i32;
        }
        if arg1 as i32 & 4 as i32 != 0 {
            if (arg2 as i32 / 5 as i32)
                < 9
            {
                return arg2 as i32 + 5 as i32;
            }
            mode = (*arg0).mode;
            match mode as i32 {
                0 => return 0x33 as i32,
                1 => return 0x34 as i32,
                2 => return 0x35 as i32,
                _ => {}
            }
        } else if arg1 as i32 & 8 as i32 != 0 {
            if arg2 as i32 / 5 as i32
                != 0
            {
                return arg2 as i32 - 5 as i32;
            }
            mode = (*arg0).mode;
            match mode as i32 {
                0 => return 0x33 as i32,
                1 => return 0x34 as i32,
                2 => return 0x35 as i32,
                _ => {}
            }
        }
    }
    return arg2 as i32;
}
unsafe extern "C" fn GetAutoNameCharacter(
    mut names: *mut *mut u8,
    mut char_idx: i32,
) -> u8 {
    return *(*names).offset(char_idx as isize);
}
unsafe extern "C" fn PickAutoNameInline(mut arg0: *mut HSD_GObj) -> i32 {
    let mut cur_text: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut data: *mut NameNewEntry = null_mut();
    let mut names: *mut *mut u8 = core::ptr::null_mut::<*mut u8>();
    let mut count: i32 = 0;
    let mut pick: i32 = 0;
    let mut dup: i32 = 0;
    let mut i: i32 = 0;
    let mut tmp: u8 = 0;
    let mut name_idx: i32 = 0;
    let mut ch: i8 = 0;
    let mut name_ptr: *mut *mut u8 = core::ptr::null_mut::<*mut u8>();
    let mut text: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut text_start: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut char_idx: i32 = 0;
    let mut null_ch: i8 = 0;
    let mut list_null_ch: i8 = 0;
    data = (*arg0).user_data as *mut NameNewEntry;
    cur_text = &raw mut mnNameNew_CurrentNameText as *mut c_char;
    loop {
        dup = 0 as i32;
        loop {
            list_null_ch = *(&raw mut mnNameNew_NullCharacter
                as *mut c_char) as i8;
            count = 0 as i32;
            names = AutoNamesList;
            while list_null_ch as i32
                != **names as i8 as i32
            {
                names = names.offset(1);
                count += 1;
            }
            pick = HSD_Randi(count);
            if !(IsNameUnique(
                *AutoNamesList.offset(pick as isize) as *mut c_char,
            ) != 0)
            {
                break;
            }
        }
        if pick == (*data).auto_history[0 as i32 as usize] as i32 {
            dup = 1 as i32;
        } else if pick == (*data).auto_history[1 as i32 as usize] as i32 {
            dup = 1 as i32;
        } else if pick == (*data).auto_history[2 as i32 as usize] as i32 {
            dup = 1 as i32;
        } else if pick == (*data).auto_history[3 as i32 as usize] as i32 {
            dup = 1 as i32;
        } else if pick == (*data).auto_history[4 as i32 as usize] as i32 {
            dup = 1 as i32;
        }
        if !(dup != 0) {
            break;
        }
    }
    name_idx = 0 as i32;
    char_idx = name_idx;
    *cur_text.offset(0 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    text_start = cur_text;
    text = text_start;
    *cur_text.offset(3 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *cur_text.offset(6 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *cur_text.offset(9 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    name_ptr = AutoNamesList.offset(pick as isize) as *mut *mut u8;
    loop {
        null_ch = *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8;
        ch = GetAutoNameCharacter(name_ptr, char_idx) as i8;
        if !(null_ch as i32 != ch as i32) {
            break;
        }
        *text.offset(0 as i32 as isize) = ch as c_char;
        *text.offset(1 as i32 as isize) = GetAutoNameCharacter(
            name_ptr,
            char_idx + 1 as i32,
        ) as c_char;
        *text.offset(2 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
            as *mut c_char);
        char_idx += 2;
        name_idx += 1;
        text = text.offset(3 as i32 as isize);
    }
    *text_start
        .offset((name_idx as i32 * 3 as i32) as isize) = null_ch
        as c_char;
    let mut history: *mut u8 = &raw mut (*data).auto_history as *mut u8;
    tmp = *history.offset(0 as i32 as isize);
    *history.offset(0 as i32 as isize) = pick as u8;
    i = 1 as i32;
    while i < 5 {
        let mut entry: *mut u8 = history.offset(i as isize) as *mut u8;
        let mut next: u8 = *entry;
        *entry = tmp;
        tmp = next;
        i += 1;
    }
    return null_ch as i32;
}
pub unsafe extern "C" fn PickAutoName(mut arg0: *mut HSD_GObj) -> i32 {
    return PickAutoNameInline(arg0);
}
pub unsafe extern "C" fn NameContainsOnlySpaces() -> i32 {
    let mut null_char: i16 = 0;
    let mut text: *mut c_char = &raw mut mnNameNew_CurrentNameText
        as *mut c_char;
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 4 {
        null_char = *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8
            as i16;
        if null_char as i32
            != *text.offset(0 as i32 as isize) as i8 as i32
        {
            if *(c"\xE3\x80\x80".as_ptr()) as i8 as i32
                != *text.offset(0 as i32 as isize) as i8
                    as i32
                || *(c"\xE3\x80\x80".as_ptr()
                    as *mut GlyphChar)
                    .offset(1 as i32 as isize) as i32
                    != *text.offset(1 as i32 as isize)
                        as i32
            {
                return 0;
            }
        }
        text = text.offset(3 as i32 as isize);
        i += 1;
    }
    return 1;
}
unsafe extern "C" fn CopyCurrentNameToNametag(mut nametag: *mut NameTagData) {
    let mut idx: i32 = 0;
    let mut ch: u8 = 0;
    let mut text: *mut u8 = null_mut();
    let mut null_ch: i8 = 0;
    let mut i: i32 = 0;
    text = &raw mut mnNameNew_CurrentNameText as *mut c_char as *mut u8;
    idx = 0 as i32;
    i = 0 as i32;
    while i < 4 {
        let mut ptr: *mut u8 = null_mut();
        null_ch = *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8;
        if null_ch as i32 == *text as i8 as i32 {
            break;
        }
        ptr = text;
        loop {
            null_ch = *(&raw mut mnNameNew_NullCharacter as *mut c_char)
                as i8;
            ch = *ptr;
            if !(null_ch as i32 != ch as i8 as i32) {
                break;
            }
            (*nametag).namedata[idx as usize] = ch as i8 as c_char;
            idx += 1;
            ptr = ptr.offset(1 as i32 as isize);
        }
        text = text.offset(3 as i32 as isize);
        i += 1;
    }
    (*nametag).namedata[idx as usize] = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char) as i8 as c_char;
}
pub unsafe extern "C" fn WriteCharactersForNameAtIndex(
    mut arg0: u8,
    mut arg1: i32,
) -> i32 {
    let mut nametag: *mut NameTagData = null_mut();
    let mut ret: i32 = 0;
    nametag = GetPersistentNameData(arg0 as i32);
    CopyCurrentNameToNametag(nametag);
    ret = GetRumbleSettingOfPort(arg1 as isize) as i32;
    (*nametag).rumble_enabled = ret as u8;
    return ret;
}
unsafe extern "C" fn AddCharacterToName_getGlyphs(
    mut arg0: *mut GlyphRow,
    mut arg1: u8,
) -> *mut *mut c_char {
    return arg0.offset(arg1 as isize) as *mut GlyphRow as *mut *mut c_char;
}
pub unsafe extern "C" fn AddCharacterToName(
    mut arg0: *mut c_char,
    mut arg1: u8,
    mut arg2: u8,
    mut arg3: u8,
) -> *mut c_char {
    let mut ch: c_char = 0;
    let mut dest: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut table: *mut *mut c_char = core::ptr::null_mut::<
        *mut c_char,
    >();
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    match arg3 as i32 {
        0 | 1 => {
            let mut null: c_char = 0;
            let mut idx: i32 = 0;
            if (arg1 as i32 - 0x30 as i32) as u8
                as u32 <= 1
            {
                if arg2 as i32 % 2 as i32
                    != 0
                {
                    table = AddCharacterToName_getGlyphs(
                        &raw mut mnNameNew_GlyphTable.upper_glyphs
                            as *mut [*mut GlyphChar; 4] as *mut GlyphRow,
                        arg1,
                    );
                } else {
                    table = AddCharacterToName_getGlyphs(
                        &raw mut mnNameNew_GlyphTable.lower_glyphs
                            as *mut [*mut GlyphChar; 4] as *mut GlyphRow,
                        arg1,
                    );
                }
            } else if arg3 as i32 == 0
                && arg2 as i32 % 2 as i32
                    == 0
                || arg3 as i32 == 1
                    && arg2 as i32 % 2 as i32
                        != 0
            {
                table = AddCharacterToName_getGlyphs(
                    &raw mut mnNameNew_GlyphTable.lower_glyphs
                        as *mut [*mut GlyphChar; 4] as *mut GlyphRow,
                    arg1,
                );
            } else {
                table = AddCharacterToName_getGlyphs(
                    &raw mut mnNameNew_GlyphTable.upper_glyphs
                        as *mut [*mut GlyphChar; 4] as *mut GlyphRow,
                    arg1,
                );
            }
            dest = arg0;
            idx = 0 as i32;
            loop {
                null = (*(&raw mut mnNameNew_NullCharacter as *mut c_char)
                    as i32 & 0xffff as i32)
                    as c_char;
                ch = (*(*table
                    .offset(
                        (arg2 as i32 / 2 as i32) as isize,
                    ))
                    .offset(idx as isize) as u32
                    & 0xff as u32) as c_char;
                if !(null as i32 != ch as i32) {
                    break;
                }
                *dest.offset(idx as isize) = ch;
                idx += 1;
            }
            *arg0.offset(idx as isize) = null;
        }
        2 => {
            *arg0.offset(0 as i32 as isize) = *(*(&raw mut mnNameNew_KeyMap
                .character_bytes as *mut *mut GlyphChar as *mut *mut c_char)
                .offset(arg1 as isize))
                .offset(0 as i32 as isize);
            *arg0.offset(1 as i32 as isize) = *(*(&raw mut mnNameNew_KeyMap
                .character_bytes as *mut *mut GlyphChar as *mut *mut c_char)
                .offset(arg1 as isize))
                .offset(1 as i32 as isize);
            *arg0.offset(2 as i32 as isize) = *(*(&raw mut mnNameNew_KeyMap
                .character_bytes as *mut *mut GlyphChar as *mut *mut c_char)
                .offset(arg1 as isize))
                .offset(2 as i32 as isize);
        }
        _ => {}
    }
    return arg0;
}
pub unsafe extern "C" fn mnNameNew_GlyphVariantInput(mut gobj: *mut HSD_GObj) {
    let mut data: *mut NameNewEntry = null_mut();
    let mut buttons: u32 = 0;
    let mut count: i32 = 0;
    let mut old_hover: u16 = 0;
    let mut old_sel: u8 = 0;
    let mut cur_pos: u8 = 0;
    let mut table: *mut *mut GlyphChar = core::ptr::null_mut::<*mut GlyphChar>();
    let mut total: i32 = 0;
    let mut null_ch: i8 = 0;
    data = (*mnNameNew_804D6C08).user_data as *mut NameNewEntry;
    buttons = mn_80229624(mnNameNew_PortInUse as u32);
    mn_804A04F0.buttons = buttons as u64;
    count = 0 as i32;
    if buttons & 0x200 as u32 != 0 {
        HSD_GObjFree((*data).variant_gobj as *mut HSD_GObj);
        (*data).variant_gobj = null_mut();
        AddCharacterToName(
            (&raw mut mnNameNew_CurrentNameText as *mut c_char)
                .offset(
                    ((*data).cursor_pos as i32 * 3 as i32)
                        as isize,
                ) as *mut c_char,
            mn_804A04F0.hovered_selection as u8,
            mn_804A04F0.confirmed_selection,
            (*data).mode,
        );
        sfxForward();
        cur_pos = (*data).cursor_pos;
        old_hover = mn_804A04F0.hovered_selection;
        count = old_hover as i32;
        if (cur_pos as u32) < 3 {
            (*data).cursor_pos = (cur_pos as i32
                + 1 as i32) as u8;
        } else {
            mn_804A04F0.hovered_selection = 0x39;
        }
        mnNameNew_8023CE4C();
        if count != 0x30 && count != 0x31
            && mn_804A04F0.confirmed_selection as i32
                % 2 as i32 != 0
        {
            if (*data).mode as i32 == 0 {
                (*data).mode = 1;
            } else {
                (*data).mode = 0;
            }
            mnNameNew_KeySetup(data, (*data).mode);
            mnNameNew_8023B0F8(
                mnNameNew_804D6C08,
                mn_804A04F0.hovered_selection as u8,
            );
            mnNameNew_8023B314(data, mn_804A04F0.hovered_selection as i32);
        }
    } else {
        if buttons & 0x20 as u32 != 0 {
            HSD_GObjFree((*data).variant_gobj as *mut HSD_GObj);
            (*data).variant_gobj = null_mut();
            mnNameNew_8023CE4C();
            return;
        }
        null_ch = *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8;
        table = &raw mut *(&raw mut mnNameNew_GlyphTable.lower_glyphs
            as *mut [*mut GlyphChar; 4])
            .offset((*data).x1 as isize) as *mut *mut GlyphChar;
        while null_ch as i32 != **table as i8 as i32 {
            table = table.offset(1);
            count += 1;
        }
        total = (count as i32 * 2 as i32) as i32;
        old_sel = mn_804A04F0.confirmed_selection;
        if buttons & 1 as u32 != 0 {
            if mn_804A04F0.confirmed_selection as i32
                % 2 as i32 != 0
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 - 1 as i32) as u8;
            } else {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 + 1 as i32) as u8;
            }
        } else if buttons & 2 as u32 != 0 {
            if mn_804A04F0.confirmed_selection as i32
                % 2 as i32 != 0
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 - 1 as i32) as u8;
            } else {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 + 1 as i32) as u8;
            }
        } else if buttons & 4 as u32 != 0 {
            if mn_804A04F0.confirmed_selection as i32
                / 2 as i32 == 0
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 % 2 as i32
                    + (total as i32 / 2 as i32
                        - 1 as i32) * 2 as i32) as u8;
            } else {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 - 2 as i32) as u8;
            }
        } else if buttons & 8 as u32 != 0 {
            if mn_804A04F0.confirmed_selection as i32
                / 2 as i32
                == total as i32 / 2 as i32
                    - 1 as i32
            {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 % 2 as i32) as u8;
            } else {
                mn_804A04F0.confirmed_selection = (mn_804A04F0.confirmed_selection
                    as i32 + 2 as i32) as u8;
            }
        }
        if old_sel as i32
            != mn_804A04F0.confirmed_selection as i32
        {
            sfxMove();
        }
    };
}
unsafe extern "C" fn copyName(
    mut name_text: *mut c_char,
    mut name_buffer: *mut c_char,
) {
    let mut ch: c_char = 0;
    let mut dest: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut src: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut dest_iter: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut src_iter: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut len: i32 = 0;
    let mut null_char: c_char = 0;
    let mut i: i32 = 0;
    src = name_text;
    dest = name_buffer;
    len = 0;
    i = 0;
    while i < 4 {
        src_iter = src;
        dest_iter = dest;
        loop {
            null_char = *(&raw mut mnNameNew_NullCharacter as *mut c_char);
            ch = *src_iter;
            if !(null_char as i8 as i32 != ch as i8 as i32)
            {
                break;
            }
            let fresh0 = dest_iter;
            dest_iter = dest_iter.offset(1);
            *fresh0 = ch;
            dest = dest.offset(1);
            len += 1;
            src_iter = src_iter.offset(1);
        }
        src = src.offset(3 as i32 as isize);
        i += 1;
    }
    *name_buffer.offset(len as isize) = null_char;
}
unsafe extern "C" fn mnNameNew_CountVariants(
    mut glyphs: *mut GlyphRow,
    mut selected_key: u8,
) -> i32 {
    let mut ptrs: *mut *mut c_char = &raw mut *glyphs
        .offset(selected_key as isize) as *mut *mut c_char;
    let mut count: i32 = 0;
    let mut terminator: i8 = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char) as i8;
    while terminator as i32
        != **ptrs.offset(0 as i32 as isize) as i8 as i32
    {
        ptrs = ptrs.offset(1);
        count += 1;
    }
    return count;
}
pub unsafe extern "C" fn mnNameNew_MainInput(mut arg0: *mut HSD_GObj) {
    let mut space_lead: c_char = 0;
    let mut unused: [c_char; 12] = [0; 12];
    let mut name_buffer: [c_char; 16] = [0; 16];
    let mut data: *mut NameNewEntry = null_mut();
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    let mut buttons: u32 = 0;
    let mut hovered: *mut u16 = null_mut();
    let mut key_char: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut null_char: i8 = 0;
    let mut n: i32 = 0;
    let mut cursor: u8 = 0;
    let mut entry: *mut NameNewEntry = (*mnNameNew_804D6C08).user_data
        as *mut NameNewEntry;
    data = entry;
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    if !(*data).variant_gobj.is_null() {
        mnNameNew_GlyphVariantInput(arg0);
        return;
    }
    mn_804A04F0.buttons = mn_80229624(mnNameNew_PortInUse as u32) as u64;
    buttons = mn_804A04F0.buttons as u32;
    n = 0 as i32;
    if buttons & 0x200 as u32 != 0 {
        hovered = &raw mut mn_804A04F0.hovered_selection;
        let mut sel: u16 = *hovered;
        if (sel as u32) < 0x32 {
            if (*data).mode as i32 != 2
                && (sel as u32) < 0x32
            {
                key_char = (*(&raw mut mnNameNew_GlyphTable.lower_glyphs
                    as *mut [*mut GlyphChar; 4] as *mut GlyphRow)
                    .offset(sel as u8 as isize))[0 as i32 as usize];
                space_lead = core::mem::transmute::<
                    [u8; 4],
                    [c_char; 4],
                >(*b"\xE3\x80\x80\0")[0 as i32 as usize];
                if space_lead as i32
                    == *key_char.offset(0 as i32 as isize) as i8
                        as i32
                    && core::mem::transmute::<
                        [u8; 4],
                        [c_char; 4],
                    >(*b"\xE3\x80\x80\0")[1 as i32 as usize] as i8
                        as i32
                        == *key_char.offset(1 as i32 as isize) as i8
                            as i32
                {
                    n = 1 as i32;
                } else {
                    n = 0 as i32;
                }
                if n == 0 {
                    lbAudioAx_80024030(1 as i32);
                    mn_804A04F0.confirmed_selection = 0;
                    n = mnNameNew_CountVariants(
                        &raw mut mnNameNew_GlyphTable.lower_glyphs
                            as *mut [*mut GlyphChar; 4] as *mut GlyphRow,
                        *hovered as u8,
                    );
                    let mut variant_count: u8 = (n as i32
                        * 2 as i32) as u8;
                    (*data).variant_gobj = mnNameNew_GlyphVariantSetup(
                        data,
                        variant_count as u16,
                        *hovered as i32 & 0xff as i32,
                    );
                    return;
                }
                cursor = (*data).cursor_pos;
                mnNameNew_CurrentNameText[(cursor as i32
                    * 3 as i32) as usize] = core::mem::transmute::<
                    [u8; 4],
                    [c_char; 4],
                >(*b"\xE3\x80\x80\0")[0 as i32 as usize];
                mnNameNew_CurrentNameText[(cursor as i32
                    * 3 as i32 + 1 as i32) as usize] = core::mem::transmute::<
                    [u8; 4],
                    [c_char; 4],
                >(*b"\xE3\x80\x80\0")[1 as i32 as usize];
                mnNameNew_CurrentNameText[(cursor as i32
                    * 3 as i32 + 2 as i32) as usize] = *(&raw mut mnNameNew_NullCharacter
                    as *mut c_char as *mut GlyphChar);
                lbAudioAx_80024030(1 as i32);
                if ((*data).cursor_pos as i32) < 3 {
                    (*data).cursor_pos = ((*data).cursor_pos as i32
                        + 1 as i32) as u8;
                } else {
                    *hovered = 0x39;
                }
                mnNameNew_8023CE4C();
                return;
            }
            AddCharacterToName(
                (&raw mut mnNameNew_CurrentNameText as *mut c_char)
                    .offset(
                        ((*data).cursor_pos as i32
                            * 3 as i32) as isize,
                    ) as *mut c_char,
                sel as u8,
                0,
                (*data).mode,
            );
            lbAudioAx_80024030(1 as i32);
            if ((*data).cursor_pos as i32) < 3 {
                (*data).cursor_pos = ((*data).cursor_pos as i32
                    + 1 as i32) as u8;
            } else {
                *hovered = 0x39;
            }
            mnNameNew_8023CE4C();
            return;
        } else {
            match sel as i32 {
                50 => {
                    lbAudioAx_80024030(0 as i32);
                    mnNameNew_8023B224(0 as u8);
                    return;
                }
                51 => {
                    if (*data).mode as i32 != 0 {
                        lbAudioAx_80024030(1 as i32);
                        (*data).mode = 0;
                        mnNameNew_KeySetup(data, 0);
                        mnNameNew_8023B0F8(mnNameNew_804D6C08, *hovered as u8);
                        return;
                    }
                }
                52 => {
                    if (*data).mode as i32 != 1 {
                        lbAudioAx_80024030(1 as i32);
                        (*data).mode = 1;
                        mnNameNew_KeySetup(data, 1);
                        mnNameNew_8023B0F8(mnNameNew_804D6C08, *hovered as u8);
                        return;
                    }
                }
                53 => {
                    if (*data).mode as i32 != 2 {
                        lbAudioAx_80024030(1 as i32);
                        (*data).mode = 2;
                        mnNameNew_KeySetup(data, 2);
                        mnNameNew_8023B0F8(mnNameNew_804D6C08, *hovered as u8);
                        return;
                    }
                }
                54 => {
                    lbAudioAx_80024030(0 as i32);
                    cursor = (*data).cursor_pos;
                    let mut slot: *mut c_char = (&raw mut mnNameNew_CurrentNameText
                        as *mut c_char)
                        .offset(
                            (cursor as i32 * 3 as i32)
                                as isize,
                        ) as *mut c_char;
                    if *(&raw mut mnNameNew_NullCharacter as *mut c_char
                        as *mut GlyphChar) as i8 as i32
                        == *slot.offset(0 as i32 as isize) as i8
                            as i32
                    {
                        n = 0 as i32;
                    } else {
                        n = 1 as i32;
                    }
                    if n != 0 {
                        *slot.offset(0 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
                            as *mut c_char as *mut GlyphChar);
                        mnNameNew_8023CE4C();
                        return;
                    }
                    if cursor as i32 != 0 {
                        mnNameNew_CurrentNameText[((cursor as i32
                            - 1 as i32) as u8 as i32
                            * 3 as i32) as usize] = *(&raw mut mnNameNew_NullCharacter
                            as *mut c_char as *mut GlyphChar);
                        (*data).cursor_pos = ((*data).cursor_pos as i32
                            - 1 as i32) as u8;
                        mnNameNew_8023CE4C();
                        return;
                    }
                    lbAudioAx_80024030(3 as i32);
                    return;
                }
                55 => {
                    lbAudioAx_80024030(1 as i32);
                    PickAutoName(mnNameNew_804D6C08);
                    null_char = *(&raw mut mnNameNew_NullCharacter
                        as *mut c_char as *mut GlyphChar) as i8;
                    let mut p: *mut c_char = &raw mut mnNameNew_CurrentNameText
                        as *mut c_char;
                    if null_char as i32 != *p as i8 as i32
                    {
                        n = 1 as i32;
                        p = p.offset(3 as i32 as isize);
                        if null_char as i32
                            != *p as i8 as i32
                        {
                            n = 2 as i32;
                            p = p.offset(3 as i32 as isize);
                            if null_char as i32
                                != *p as i8 as i32
                            {
                                n = 3 as i32;
                                p = p.offset(3 as i32 as isize);
                                if null_char as i32
                                    != *p as i8 as i32
                                {
                                    n = 4 as i32;
                                }
                            }
                        }
                    }
                    if n == 4 {
                        (*data).cursor_pos = 3;
                    } else {
                        (*data).cursor_pos = n as u8;
                    }
                    mnNameNew_8023CE4C();
                    return;
                }
                56 | 57 => {
                    copyName(
                        &raw mut mnNameNew_CurrentNameText as *mut c_char,
                        &raw mut name_buffer as *mut c_char,
                    );
                    if mnNameNew_CurrentNameText[0 as i32 as usize] as i8
                        as i32
                        == *(&raw mut mnNameNew_NullCharacter as *mut c_char
                            as *mut GlyphChar) as i8 as i32
                    {
                        n = 1 as i32;
                    } else {
                        n = 0 as i32;
                    }
                    if n == 0
                        && NameContainsOnlySpaces() == 0
                        && IsNameUnique(&raw mut name_buffer as *mut c_char)
                            == 0
                        && IsNameNotAllowed(
                            &raw mut name_buffer as *mut c_char,
                        ) == 0
                    {
                        n = 1 as i32;
                    } else {
                        n = 0 as i32;
                    }
                    if n != 0 {
                        lbAudioAx_80024030(1 as i32);
                        let mut name_index: u8 = (*data).name_index;
                        CreateNameAtIndex(name_index as i32);
                        WriteCharactersForNameAtIndex(
                            (*data).name_index,
                            mn_802295AC() as i32,
                        );
                        mnNameNew_8023B224(1 as u8);
                        return;
                    }
                    lbAudioAx_80024030(3 as i32);
                    return;
                }
                _ => {}
            }
        }
    } else if buttons & 0x100 as u32 != 0 {
        if mn_804A04F0.hovered_selection as i32
            == 0x38
            || mn_804A04F0.hovered_selection as i32
                == 0x39
        {
            copyName(
                &raw mut mnNameNew_CurrentNameText as *mut c_char,
                &raw mut name_buffer as *mut c_char,
            );
            if mnNameNew_CurrentNameText[0 as i32 as usize] as i8
                as i32
                == *(&raw mut mnNameNew_NullCharacter as *mut c_char
                    as *mut GlyphChar) as i8 as i32
            {
                n = 1 as i32;
            } else {
                n = 0 as i32;
            }
            if n == 0
                && NameContainsOnlySpaces() == 0
                && IsNameUnique(&raw mut name_buffer as *mut c_char)
                    == 0
                && IsNameNotAllowed(&raw mut name_buffer as *mut c_char)
                    == 0
            {
                n = 1 as i32;
            } else {
                n = 0 as i32;
            }
            if n != 0 {
                lbAudioAx_80024030(1 as i32);
                CreateNameAtIndex((*data).name_index as i32);
                WriteCharactersForNameAtIndex((*data).name_index, mn_802295AC() as i32);
                mnNameNew_8023B224(1 as u8);
                return;
            }
            lbAudioAx_80024030(3 as i32);
            return;
        }
        mn_804A04F0.hovered_selection = 0x39;
        return;
    } else if buttons & 0xc0 as u32 != 0 {
        lbAudioAx_80024030(1 as i32);
        if buttons & 0x40 as u32 != 0 {
            let mut mode: u8 = (*data).mode;
            if mode as i32 != 0 {
                (*data).mode = ((*data).mode as i32
                    - 1 as i32) as u8;
            } else {
                (*data).mode = 2;
            }
        } else if ((*data).mode as i32) < 2 {
            (*data).mode = ((*data).mode as i32 + 1 as i32)
                as u8;
        } else {
            (*data).mode = 0;
        }
        mnNameNew_KeySetup(data, (*data).mode);
        mnNameNew_8023B0F8(mnNameNew_804D6C08, mn_804A04F0.hovered_selection as u8);
        mnNameNew_8023B314(data, mn_804A04F0.hovered_selection as i32);
        return;
    } else if buttons & 0x20 as u32 != 0 {
        lbAudioAx_80024030(0 as i32);
        if mnNameNew_CurrentNameText[0 as i32 as usize] as i8
            as i32
            == *(&raw mut mnNameNew_NullCharacter as *mut c_char
                as *mut GlyphChar) as i8 as i32
        {
            n = 1 as i32;
        }
        if n != 0 {
            mnNameNew_8023B224(0 as u8);
            return;
        }
        let mut cursor_pos: u8 = (*data).cursor_pos;
        cursor = cursor_pos;
        if *(&raw mut mnNameNew_NullCharacter as *mut c_char
            as *mut GlyphChar) as i8 as i32
            == mnNameNew_CurrentNameText[(cursor as i32
                * 3 as i32) as usize] as i8 as i32
        {
            n = 0 as i32;
        } else {
            n = 1 as i32;
        }
        if n != 0 {
            mnNameNew_CurrentNameText[(cursor as i32
                * 3 as i32) as usize] = *(&raw mut mnNameNew_NullCharacter
                as *mut c_char as *mut GlyphChar);
            mnNameNew_8023CE4C();
            return;
        }
        if cursor as i32 != 0 {
            mnNameNew_CurrentNameText[((cursor as i32
                - 1 as i32) as u8 as i32
                * 3 as i32) as usize] = *(&raw mut mnNameNew_NullCharacter
                as *mut c_char as *mut GlyphChar);
            (*data).cursor_pos = ((*data).cursor_pos as i32
                - 1 as i32) as u8;
            mnNameNew_8023CE4C();
            return;
        }
        lbAudioAx_80024030(3 as i32);
        return;
    } else {
        let mut new_sel: u8 = 0;
        hovered = &raw mut mn_804A04F0.hovered_selection;
        new_sel = mnNameNew_8023BAA8(data, buttons as i32, *hovered as u8) as u8;
        if new_sel as i32 != *hovered as i32 {
            lbAudioAx_80024030(2 as i32);
            *hovered = new_sel as u16;
            if (new_sel as i32) < 0x32 {
                (*data).last_key_sel = new_sel;
            }
        }
    };
}
unsafe extern "C" fn mnNameNew_GetEntryData() -> *mut NameNewEntry {
    return (*mnNameNew_804D6C08).user_data as *mut NameNewEntry;
}
pub unsafe extern "C" fn mnNameNew_8023CE4C() {
    let mut text_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut name_char_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut name_char_color_ptr: [*mut GXColor; 1] = [core::ptr::null_mut::<
        GXColor,
    >(); 1];
    let mut data: *mut NameNewEntry = null_mut();
    let mut jobj_a: *mut HSD_JObj = null_mut();
    let mut jobj_b: *mut HSD_JObj = null_mut();
    let mut old_text: *mut HSD_Text = null_mut();
    let mut y_minus: f32 = 0.;
    let mut char_spacing: f32 = 0.;
    let mut first_x: f32 = 0.;
    let mut i: i32 = 0;
    let mut text: *mut HSD_Text = null_mut();
    let mut z: f32 = 0.;
    data = mnNameNew_GetEntryData();
    jobj_a = (*data).jobjs[14 as i32 as usize] as *mut HSD_JObj;
    jobj_b = (*data).jobjs[15 as i32 as usize] as *mut HSD_JObj;
    first_x = HSD_JObjGetTranslationX(jobj_a);
    char_spacing = HSD_JObjGetTranslationX(jobj_b) - first_x;
    if !(*data).name_disp_text.is_null() {
        HSD_SisLib_803A5CC4((*data).name_disp_text);
    }
    text = HSD_SisLib_803A6754(
        0,
        mn_804D6BB5 as i32,
    );
    lb_8000B1CC(jobj_a, &raw mut mnNameNew_803EE330, &raw mut text_pos);
    y_minus = -text_pos.y as f32;
    z = text_pos.z;
    name_char_color_ptr[0 as i32 as usize] = &raw mut name_char_color;
    (*text).pos_x = text_pos.x;
    i = 0 as i32;
    (*text).pos_y = y_minus as f32;
    (*text).pos_z = z;
    (*text).font_size.x = 0.04_f32 as f32;
    (*text).font_size.y = 0.05_f32 as f32;
    (*text).text_color = mnNameNew_804D4F6C;
    while i < 4 {
        if *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8
            as i32
            == mnNameNew_CurrentNameText[(i as i32
                * 3 as i32) as usize] as i8 as i32
        {
            break;
        }
        HSD_SisLib_803A6B98(text, char_spacing as f32 * i as f32
                / (*text).font_size.x as f32, 0.0_f32, (&raw mut mnNameNew_CurrentNameText as *mut c_char)
                .offset((i as i32 * 3 as i32) as isize)
                as *mut c_char, &[]);
        name_char_color = mnNameNew_804D4F78;
        HSD_SisLib_803A74F0(
            text,
            i,
            name_char_color_ptr[0 as i32 as usize],
        );
        i += 1;
    }
    (*data).name_disp_text = text;
}
pub unsafe extern "C" fn fn_8023CFC8(mut arg0: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut root: *mut HSD_JObj = null_mut();
    let mut new_var: u8 = 0;
    let mut old_sel: u8 = 0;
    let mut one: f32 = 0.;
    let mut count: i32 = 0;
    let mut data: *mut GlyphVariantEntry = null_mut();
    let mut pad1: u32 = 0;
    let mut pad2: u32 = 0;
    let mut key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut selected_key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    data = (*arg0).user_data as *mut GlyphVariantEntry;
    new_var = (*data).selection;
    old_sel = new_var;
    if old_sel as i32
        != mn_804A04F0.confirmed_selection as i32
    {
        root = (*data).jobjs[4 as i32 as usize] as *mut HSD_JObj;
        if root.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*root).child as *mut HSD_JObj;
        }
        count = old_sel as i32;
        while count > 0 {
            if jobj.is_null() {
                jobj = null_mut();
            } else {
                jobj = (*jobj).next as *mut HSD_JObj;
            }
            count -= 1;
        }
        root = jobj;
        HSD_JObjReqAnimAll(root, 0.0_f32);
        HSD_JObjAnimAll(root);
        key_color = mnNameNew_804D4F74;
        HSD_SisLib_803A74F0((*data).text, (*data).selection as i32, &raw mut key_color);
        root = (*data).jobjs[4 as i32 as usize] as *mut HSD_JObj;
        new_var = mn_804A04F0.confirmed_selection;
        if root.is_null() {
            jobj = null_mut();
        } else {
            jobj = (*root).child as *mut HSD_JObj;
        }
        count = new_var as i32;
        while count > 0 {
            if jobj.is_null() {
                jobj = null_mut();
            } else {
                jobj = (*jobj).next as *mut HSD_JObj;
            }
            count -= 1;
        }
        one = 1.0_f32;
        HSD_JObjReqAnimAll(jobj, one as f32);
        HSD_JObjAnimAll(jobj);
        selected_key_color = mnNameNew_804D4F70;
        HSD_SisLib_803A74F0(
            (*data).text,
            mn_804A04F0.confirmed_selection as i32,
            &raw mut selected_key_color,
        );
        (*data).selection = mn_804A04F0.confirmed_selection;
    }
}
pub unsafe extern "C" fn fn_8023D0F8(mut arg0: *mut c_void) {
    let mut entry: *mut GlyphVariantEntry = arg0 as *mut GlyphVariantEntry;
    HSD_SisLib_803A5CC4((*entry).text);
    HSD_Free(arg0);
}
pub unsafe extern "C" fn mnNameNew_8023D130(
    mut arg0: *mut GlyphVariantEntry,
    mut arg1: u16,
    mut arg2: u8,
    mut arg3: i32,
) -> *mut HSD_Text {
    let mut pos_z: f32 = 0.;
    let mut y_range: f32 = 0.;
    let mut i: i32 = 0;
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    let mut color_ptr: *mut GXColor = null_mut();
    let mut col_x: f32 = 0.;
    let mut pos_x: f32 = 0.;
    let mut jobj14: *mut HSD_JObj = null_mut();
    let mut jobj18: *mut HSD_JObj = null_mut();
    let mut text: *mut HSD_Text = null_mut();
    let mut jobj1C: *mut HSD_JObj = null_mut();
    let mut font_x: f32 = 0.;
    let mut x_range: f32 = 0.;
    let mut table_upper: *mut *mut c_char = core::ptr::null_mut::<
        *mut c_char,
    >();
    let mut pos_y: f32 = 0.;
    let mut table_lower: *mut *mut c_char = core::ptr::null_mut::<
        *mut c_char,
    >();
    let mut text_pos: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut glyph_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    text = HSD_SisLib_803A6754(
        0,
        mn_804D6BB4 as i32,
    );
    jobj14 = (*arg0).jobjs[4 as i32 as usize] as *mut HSD_JObj;
    jobj18 = (*arg0).jobjs[5 as i32 as usize] as *mut HSD_JObj;
    jobj1C = (*arg0).jobjs[6 as i32 as usize] as *mut HSD_JObj;
    lb_8000B1CC(jobj14, &raw mut unk_vec, &raw mut text_pos);
    pos_x = text_pos.x;
    pos_y = -text_pos.y;
    pos_z = text_pos.z;
    (*text).pos_x = pos_x;
    (*text).pos_y = pos_y;
    (*text).pos_z = pos_z;
    (*text).font_size.x = 0.03_f32 as f32;
    (*text).font_size.y = 0.04_f32 as f32;
    (*text).text_color = mnNameNew_804D4F6C;
    x_range = HSD_JObjGetTranslationX(jobj18) - HSD_JObjGetTranslationX(jobj14);
    y_range = -(HSD_JObjGetTranslationY(jobj1C) - HSD_JObjGetTranslationY(jobj14));
    table_upper = AddCharacterToName_getGlyphs(
        &raw mut mnNameNew_GlyphTable.upper_glyphs as *mut [*mut GlyphChar; 4]
            as *mut GlyphRow,
        arg3 as u8,
    );
    table_lower = AddCharacterToName_getGlyphs(
        &raw mut mnNameNew_GlyphTable.lower_glyphs as *mut [*mut GlyphChar; 4]
            as *mut GlyphRow,
        arg3 as u8,
    );
    arg1 = arg1 as u8 as u16;
    i = 0 as i32;
    while i < arg1 as i32 {
        let mut str: *mut c_char = core::ptr::null_mut::<
            c_char,
        >();
        if (arg3 as i32 - 0x30 as i32) as u8
            as u32 <= 1
        {
            if i as i32 % 2 as i32
                != 0
            {
                str = *table_upper
                    .offset(
                        (i as i32 / 2 as i32) as isize,
                    );
            } else {
                str = *table_lower
                    .offset(
                        (i as i32 / 2 as i32) as isize,
                    );
            }
        } else if arg2 as i32 == 0
            && i as i32 % 2 as i32
                == 0
            || arg2 as i32 == 1
                && i as i32 % 2 as i32
                    != 0
        {
            str = *table_lower
                .offset((i as i32 / 2 as i32) as isize);
        } else {
            str = *table_upper
                .offset((i as i32 / 2 as i32) as isize);
        }
        font_x = (*text).font_size.x;
        col_x = (i as i32 / 2 as i32) as f32 * x_range;
        HSD_SisLib_803A6B98(text, col_x as f32 / font_x as f32, (i as i32 % 2 as i32) as f32
                * y_range as f32
                / (*text).font_size.y as f32, str, &[(font_x as f64).to_bits() as usize, (col_x as f64).to_bits() as usize]);
        if i == mn_804A04F0.confirmed_selection as i32 {
            color_ptr = &raw mut mnNameNew_804D4F70;
        } else {
            color_ptr = &raw mut mnNameNew_804D4F74;
        }
        mnNameNew_SetKeyColor(text, i, color_ptr, &raw mut glyph_color);
        i += 1;
    }
    (*arg0).text = text;
    return text;
}
static mut mnNameNew_803B8528: Vec3 = Vec3 {
    x: -0.5_f32,
    y: 0.7_f32,
    z: 0.0_f32,
};
unsafe extern "C" fn GlyphVariantCount(mut count: u16, mut out: *mut i32) {
    count = (count as i32 & 0xff as i32) as u16;
    *out = count as i32;
}
unsafe extern "C" fn AnimateGlyphVariant(
    mut variant: *mut HSD_JObj,
    mut user_data: *mut GlyphVariantEntry,
    mut i: i32,
) {
    HSD_JObjReqAnimAll(
        variant,
        ((*user_data).selection as i32 == i) as i32
            as f32,
    );
    HSD_JObjAnimAll(variant);
}
unsafe extern "C" fn CreateGlyphVariant(
    mut variant_desc: *mut StaticModelDesc,
    mut user_data: *mut GlyphVariantEntry,
    mut i: i32,
    mut out: *mut *mut HSD_JObj,
) {
    let mut variant: *mut HSD_JObj = null_mut();
    variant = HSD_JObjLoadJoint((*variant_desc).joint as *mut HSD_Joint);
    HSD_JObjAddAnimAll(
        variant,
        (*variant_desc).animjoint,
        (*variant_desc).matanim_joint,
        (*variant_desc).shapeanim_joint,
    );
    AnimateGlyphVariant(variant, user_data, i);
    *out = variant;
}
pub unsafe extern "C" fn mnNameNew_GlyphVariantSetup(
    mut arg0: *mut NameNewEntry,
    mut arg1: u16,
    mut arg2: i32,
) -> *mut HSD_GObj {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut variant: *mut HSD_JObj = null_mut();
    let mut key_jobj: *mut HSD_JObj = null_mut();
    let mut ref_jobj: *mut HSD_JObj = null_mut();
    let mut i: i32 = 0;
    let mut variant_count: i32 = 0;
    let mut ref2: *mut HSD_JObj = null_mut();
    let mut position: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut offset: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut dx: f32 = 0.;
    let mut dy: f32 = 0.;
    let mut setup_desc: *mut StaticModelDesc = core::ptr::null_mut::<
        StaticModelDesc,
    >();
    let mut ref3: *mut HSD_JObj = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut variant_desc: *mut StaticModelDesc = core::ptr::null_mut::<
        StaticModelDesc,
    >();
    setup_desc = &raw mut mnNameNew_804A0710;
    gobj = GObj_Create(6, 7, 0x80);
    jobj = HSD_JObjLoadJoint((*setup_desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        6,
        0x80,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_8023CFC8),
        0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*setup_desc).animjoint,
        (*setup_desc).matanim_joint,
        (*setup_desc).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(jobj, arg1 as u8 as f32 / 2.0_f32);
    HSD_JObjAnimAll(jobj);
    let mut user_data: *mut GlyphVariantEntry = HSD_MemAlloc(
        core::mem::size_of::<GlyphVariantEntry>() as isize,
    ) as *mut GlyphVariantEntry;
    if user_data.is_null() {
        OSReport(
            c"Can't get user_data.\n".as_ptr(),
        );
        __assert("mnnamenew.c", 1611, "user_data");
    };
    GObj_InitUserData(
        gobj,
        0,
        Some(fn_8023D0F8),
        user_data as *mut c_void,
    );
    (*user_data).selection = mn_804A04F0.confirmed_selection;
    i = 0 as i32;
    while i < 7 {
        lb_80011E24(jobj as *mut HSD_JObj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[i, -1]);
        i += 1;
    }
    offset = mnNameNew_803B8528;
    if arg2 as u8 as u32 >= 0x32
        && (arg2 as u8 as u32) < 0x3a
    {
        key_jobj = (*arg0)
            .jobjs[mnNameNew_KeyMap
            .key_jobj_ids[(arg2 as u8 as i32
            - 0x32 as i32) as usize] as usize] as *mut HSD_JObj;
    } else {
        key_jobj = HSD_JObjGetChild((*arg0).jobjs[16 as i32 as usize]);
        i = 0 as i32;
        while i < 50 {
            if i == arg2 as u8 as i32 {
                break;
            }
            if key_jobj.is_null() {
                key_jobj = null_mut();
            } else {
                key_jobj = (*key_jobj).next as *mut HSD_JObj;
            }
            i += 1;
        }
    }
    lb_8000B1CC(key_jobj, &raw mut offset, &raw mut position);
    HSD_JObjSetTranslate(jobj, &raw mut position);
    ref_jobj = (*user_data).jobjs[4 as i32 as usize] as *mut HSD_JObj;
    ref2 = (*user_data).jobjs[5 as i32 as usize] as *mut HSD_JObj;
    ref3 = (*user_data).jobjs[6 as i32 as usize] as *mut HSD_JObj;
    dx = HSD_JObjGetTranslationX(ref2) - HSD_JObjGetTranslationX(ref_jobj);
    dy = HSD_JObjGetTranslationY(ref3) - HSD_JObjGetTranslationY(ref_jobj);
    variant_desc = &raw mut mnNameNew_804A0720 as *mut StaticModelDesc;
    GlyphVariantCount(arg1, &raw mut variant_count);
    i = 0 as i32;
    while i < variant_count {
        let mut created: *mut HSD_JObj = null_mut();
        CreateGlyphVariant(variant_desc, user_data, i, &raw mut created);
        variant = created;
        HSD_JObjSetTranslateX(
            variant,
            dx * (i as i32 / 2 as i32) as f32,
        );
        HSD_JObjSetTranslateY(
            variant,
            dy * (i as i32 % 2 as i32) as f32,
        );
        HSD_JObjAddChild(ref_jobj, variant);
        i += 1;
    }
    mnNameNew_8023D130(user_data, arg1, (*arg0).mode, arg2);
    return gobj as *mut HSD_GObj;
}
pub unsafe extern "C" fn mnNameNew_8023DA08(mut arg0: *mut NameNewEntry) -> i32 {
    let mut anim: *mut AnimLoopSettings = null_mut();
    let mut end_frame: *mut f32 = null_mut();
    let mut frame: f32 = 0.;
    let mut all_anims_done: i32 = 0;
    anim = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings;
    all_anims_done = 1 as i32;
    frame = mn_8022ED6C(
        (*arg0).jobjs[12 as i32 as usize],
        anim.offset(2 as i32 as isize) as *mut AnimLoopSettings,
    ) as f32;
    end_frame = &raw mut (*anim.offset(2 as i32 as isize)).end_frame;
    if frame < mnNameNew_803EDA58[2 as i32 as usize].end_frame {
        all_anims_done = 0 as i32;
    }
    if mn_8022ED6C(
        (*arg0).jobjs[13 as i32 as usize],
        anim.offset(2 as i32 as isize) as *mut AnimLoopSettings,
    ) < *end_frame
    {
        all_anims_done = 0 as i32;
    }
    frame = mn_8022EFD8((*arg0).jobjs[4 as i32 as usize], anim) as f32;
    end_frame = &raw mut (*(&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings))
        .end_frame;
    if frame < (*(&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings)).end_frame {
        all_anims_done = 0 as i32;
    }
    if mn_8022EFD8((*arg0).jobjs[2 as i32 as usize], anim) < *end_frame {
        all_anims_done = 0 as i32;
    }
    if mn_8022EFD8((*arg0).jobjs[6 as i32 as usize], anim) < *end_frame {
        all_anims_done = 0 as i32;
    }
    return all_anims_done;
}
pub unsafe extern "C" fn fn_8023DAEC(mut arg0: *mut HSD_GObj) {
    let mut all_anims_done: i32 = 0;
    let mut data: *mut NameNewEntry = null_mut();
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    let mut end_frame: *mut f32 = null_mut();
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    data = (*arg0).user_data as *mut NameNewEntry;
    if !(*data).key_text.is_null() {
        HSD_SisLib_803A5CC4((*data).key_text);
        (*data).key_text = null_mut();
    }
    if !(*data).name_disp_text.is_null() {
        HSD_SisLib_803A5CC4((*data).name_disp_text);
        (*data).name_disp_text = null_mut();
    }
    if !(*data).desc_text.is_null() {
        HSD_SisLib_803A5CC4((*data).desc_text);
        (*data).desc_text = null_mut();
    }
    all_anims_done = 1 as i32;
    end_frame = &raw mut (*(&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings)
        .offset(1 as i32 as isize))
        .end_frame;
    if mn_8022EFD8(
        (*data).jobjs[4 as i32 as usize],
        (&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    ) < *end_frame
    {
        all_anims_done = 0 as i32;
    }
    if mn_8022EFD8(
        (*data).jobjs[2 as i32 as usize],
        (&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    ) < *end_frame
    {
        all_anims_done = 0 as i32;
    }
    if mn_8022EFD8(
        (*data).jobjs[6 as i32 as usize],
        (&raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings)
            .offset(1 as i32 as isize) as *mut AnimLoopSettings,
    ) < *end_frame
    {
        all_anims_done = 0 as i32;
    }
    if all_anims_done != 0
        || mn_804A04F0.x10 as i32 == 1
    {
        HSD_GObjFree(arg0);
    }
}
pub unsafe extern "C" fn fn_8023DBE8(mut arg0: *mut HSD_GObj) {
    let mut flow: *mut MenuFlow = null_mut();
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut data: *mut NameNewEntry = null_mut();
    let mut hovered: *mut u16 = null_mut();
    let mut hovered_value: u16 = 0;
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut parent: *mut HSD_JObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut unused: [c_char; 4] = [0; 4];
    let mut frame: f32 = 0.;
    let mut normal_key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut highlighted_key_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut cursor: u8 = 0;
    let mut sel: u8 = 0;
    let mut i: i32 = 0;
    flow = &raw mut mn_804A04F0;
    gobj = arg0;
    data = (*gobj).user_data as *mut NameNewEntry;
    if (*flow).x10 as i32 != 1 {
        HSD_JObjSetFlagsAll(
            (*data).jobjs[16 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*data).jobjs[12 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_JObjSetFlagsAll(
            (*data).jobjs[13 as i32 as usize],
            JOBJ_HIDDEN as u32,
        );
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_8023DAEC),
            0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
        return;
    }
    mnNameNew_8023DA08(data);
    cursor = (*data).x1;
    hovered = &raw mut (*flow).hovered_selection;
    if cursor as i32 != *hovered as i32 {
        if cursor as u32 >= 0x32
            && (cursor as u32) < 0x3a
        {
            jobj = (*data)
                .jobjs[mnNameNew_KeyMap
                .key_jobj_ids[(cursor as i32 - 0x32 as i32)
                as usize] as usize] as *mut HSD_JObj;
        } else {
            parent = (*data).jobjs[16 as i32 as usize] as *mut HSD_JObj;
            if parent.is_null() {
                jobj = null_mut();
            } else {
                jobj = (*parent).child as *mut HSD_JObj;
            }
            i = 0 as i32;
            while i < 50 {
                if i == cursor as i32 {
                    break;
                }
                if jobj.is_null() {
                    jobj = null_mut();
                } else {
                    jobj = (*jobj).next as *mut HSD_JObj;
                }
                i += 1;
            }
        }
        frame = 0.0_f32 as f32;
        HSD_JObjReqAnimAll(jobj, frame);
        HSD_JObjAnimAll(jobj);
        if ((*data).x1 as u32) < 0x32 {
            normal_key_color = mnNameNew_804D4F6C;
            HSD_SisLib_803A74F0(
                (*data).key_text,
                (*data).x1 as i32,
                &raw mut normal_key_color,
            );
        }
        sel = *hovered as u8;
        if sel as u32 >= 0x32
            && (sel as u32) < 0x3a
        {
            jobj = (*data)
                .jobjs[mnNameNew_KeyMap
                .key_jobj_ids[(sel as i32 - 0x32 as i32)
                as usize] as usize] as *mut HSD_JObj;
        } else {
            parent = (*data).jobjs[16 as i32 as usize] as *mut HSD_JObj;
            if parent.is_null() {
                jobj = null_mut();
            } else {
                jobj = (*parent).child as *mut HSD_JObj;
            }
            i = 0 as i32;
            while i < 50 {
                if i == sel as i32 {
                    break;
                }
                if jobj.is_null() {
                    jobj = null_mut();
                } else {
                    jobj = (*jobj).next as *mut HSD_JObj;
                }
                i += 1;
            }
        }
        frame = 1.0_f32 as f32;
        HSD_JObjReqAnimAll(jobj, frame);
        HSD_JObjAnimAll(jobj);
        hovered_value = *hovered;
        if (hovered_value as u32) < 0x32 {
            highlighted_key_color = mnNameNew_804D4F68;
            HSD_SisLib_803A74F0(
                (*data).key_text,
                hovered_value as i32,
                &raw mut highlighted_key_color,
            );
        }
        mnNameNew_8023B314(data, *hovered as i32);
        (*data).x1 = *hovered as u8;
        mnNameNew_8023B0F8(gobj, (*data).x1);
    }
    if mnNameNew_804D4C10 + (*data).cursor_pos as f32
        != mn_8022F298((*data).jobjs[13 as i32 as usize])
    {
        cursor = (*data).cursor_pos;
        jobj = (*((*gobj).user_data as *mut NameNewEntry))
            .jobjs[13 as i32 as usize] as *mut HSD_JObj;
        HSD_JObjReqAnimAll(jobj, mnNameNew_804D4C10 + cursor as f32);
        HSD_JObjAnimAll(jobj);
    }
}
pub unsafe extern "C" fn mnNameNew_8023E0D8(mut arg0: *mut NameNewEntry) {
    let mut layout: *mut MnNameNewDataLayout = core::ptr::null_mut::<
        MnNameNewDataLayout,
    >();
    let mut anim: *mut AnimLoopSettings = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut jobj_ids: *mut u16 = null_mut();
    let mut i: i32 = 0;
    layout = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings
        as *mut MnNameNewDataLayout;
    anim = &raw mut mnNameNew_803EDA58 as *mut AnimLoopSettings;
    jobj = (*arg0).jobjs[12 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnim(jobj, (*anim.offset(2 as i32 as isize)).start_frame);
    HSD_JObjAnim(jobj);
    jobj = (*arg0).jobjs[13 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnim(jobj, (*anim.offset(2 as i32 as isize)).start_frame);
    HSD_JObjAnim(jobj);
    jobj = (*arg0).jobjs[4 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnim(jobj, (*anim.offset(0 as i32 as isize)).start_frame);
    HSD_JObjAnim(jobj);
    jobj = (*arg0).jobjs[5 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnimAll(
        jobj,
        ((*arg0).x1 as i32 == 0x38)
            as i32 as f32,
    );
    HSD_JObjAnimAll(jobj);
    jobj = (*arg0).jobjs[2 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnim(jobj, (*anim.offset(0 as i32 as isize)).start_frame);
    HSD_JObjAnim(jobj);
    jobj = (*arg0).jobjs[5 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnimAll(
        jobj,
        ((*arg0).x1 as i32 == 0x32)
            as i32 as f32,
    );
    HSD_JObjAnimAll(jobj);
    jobj = (*arg0).jobjs[6 as i32 as usize] as *mut HSD_JObj;
    HSD_JObjReqAnim(jobj, (*anim.offset(0 as i32 as isize)).start_frame);
    HSD_JObjAnim(jobj);
    jobj_ids = &raw mut mnNameNew_KeyMap.key_jobj_ids as *mut u16;
    i = 0x32 as i32;
    while i < 0x3a {
        jobj = (*arg0)
            .jobjs[*jobj_ids
            .offset((i as i32 - 0x32 as i32) as isize)
            as usize] as *mut HSD_JObj;
        HSD_JObjReqAnimAll(
            jobj,
            ((*arg0).x1 as i32 == i) as i32 as f32,
        );
        HSD_JObjAnimAll(jobj);
        i += 1;
    }
}
pub unsafe extern "C" fn InitNameEntryUIState(
    mut arg0: *mut NameNewEntry,
    mut arg1: i32,
) -> i32 {
    let mut result: i32 = 0;
    let mut count: i8 = 0;
    let mut names: *mut *mut u8 = core::ptr::null_mut::<*mut u8>();
    let mut null_char: i8 = 0;
    count = mn_804A04F0.hovered_selection as u8 as i8;
    (*arg0).x1 = count as u8;
    result = lbLang_IsSavedLanguageUS() as i32;
    if result != 0 {
        (*arg0).mode = 2;
    } else {
        (*arg0).mode = 0;
    }
    count = 0;
    (*arg0).cursor_pos = 0;
    (*arg0).last_key_sel = (*arg0).x1;
    (*arg0).variant_gobj = null_mut();
    (*arg0).name_index = arg1 as u8;
    (*arg0).key_text = null_mut();
    (*arg0).name_disp_text = null_mut();
    (*arg0).desc_text = null_mut();
    null_char = *(&raw mut mnNameNew_NullCharacter as *mut c_char) as i8;
    names = AutoNamesList;
    while null_char as i32 != **names as i8 as i32 {
        names = names.offset(1);
        count += 1;
    }
    (*arg0).auto_history[0 as i32 as usize] = count as u8;
    (*arg0).auto_history[1 as i32 as usize] = count as u8;
    (*arg0).auto_history[2 as i32 as usize] = count as u8;
    (*arg0).auto_history[3 as i32 as usize] = count as u8;
    (*arg0).auto_history[4 as i32 as usize] = count as u8;
    return result;
}
unsafe extern "C" fn mnNameNew_InitKeyJobjs(
    mut user_data: *mut NameNewEntry,
    mut child_out: *mut *mut HSD_JObj,
) {
    let mut k: i32 = 0;
    let mut key_jobj: *mut HSD_JObj = null_mut();
    let mut x_range: f32 = 0.;
    let mut y_range: f32 = 0.;
    k = 0 as i32;
    while k < 0x32 {
        key_jobj = HSD_JObjLoadJoint(mnNameNew_804A0700.joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            key_jobj,
            mnNameNew_804A0700.animjoint,
            mnNameNew_804A0700.matanim_joint,
            mnNameNew_804A0700.shapeanim_joint,
        );
        HSD_JObjReqAnimAll(
            key_jobj,
            (k as u8 as i32 == (*user_data).x1 as i32)
                as i32 as f32,
        );
        HSD_JObjAnimAll(key_jobj);
        x_range = HSD_JObjGetTranslationX(
            (*user_data).jobjs[17 as i32 as usize],
        )
            - HSD_JObjGetTranslationX(
                (*user_data).jobjs[16 as i32 as usize],
            );
        y_range = HSD_JObjGetTranslationY(
            (*user_data).jobjs[18 as i32 as usize],
        )
            - HSD_JObjGetTranslationY(
                (*user_data).jobjs[16 as i32 as usize],
            );
        mnName_80239F5C(
            key_jobj,
            x_range
                * (k as u8 as i32 / 5 as i32) as f32,
        );
        mnName_80239EBC(
            key_jobj,
            y_range
                * (k as u8 as i32 % 5 as i32) as f32,
        );
        *child_out = key_jobj;
        HSD_JObjAddChild(
            (*user_data).jobjs[16 as i32 as usize],
            *child_out,
        );
        k += 1;
    }
}
pub unsafe extern "C" fn mnNameNew_8023E32C(mut arg0: i32) {
    let mut gobj: *mut HSD_GObj = null_mut();
    let mut root_jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut NameNewEntry = null_mut();
    let mut i: i32 = 0;
    let mut setup_desc: *mut StaticModelDesc = core::ptr::null_mut::<
        StaticModelDesc,
    >();
    setup_desc = &raw mut mnNameNew_804A06F0;
    gobj = GObj_Create(6, 7, 0x80);
    mnNameNew_804D6C08 = gobj;
    root_jobj = HSD_JObjLoadJoint((*setup_desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        root_jobj as *mut c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_GObj_SetupProc(
        gobj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(fn_8023DBE8)),
        0,
    );
    HSD_JObjAddAnimAll(
        root_jobj,
        (*setup_desc).animjoint,
        (*setup_desc).matanim_joint,
        (*setup_desc).shapeanim_joint,
    );
    HSD_JObjReqAnimAll(root_jobj, 0.0_f32);
    HSD_JObjAnimAll(root_jobj);
    user_data = HSD_MemAlloc(core::mem::size_of::<NameNewEntry>() as isize)
        as *mut NameNewEntry;
    if user_data.is_null() {
        OSReport(
            c"Can't get user_data.\n".as_ptr(),
        );
        __assert("mnnamenew.c", 1983, "user_data");
    };
    GObj_InitUserData(
        gobj,
        0,
        Some(HSD_Free),
        user_data as *mut c_void,
    );
    InitNameEntryUIState(user_data, arg0);
    i = 0 as i32;
    while i < 0x13 {
        lb_80011E24(root_jobj as *mut HSD_JObj, (&raw mut (*user_data).jobjs as *mut *mut HSD_JObj).offset(i as isize)
                as *mut *mut HSD_JObj, &[i, -1]);
        i += 1;
    }
    mnNameNew_8023E0D8(user_data);
    let mut child: *mut HSD_JObj = null_mut();
    mnNameNew_InitKeyJobjs(user_data, &raw mut child);
    (*user_data).key_text = mnNameNew_KeySetup(user_data, (*user_data).mode);
    mnNameNew_8023B314(user_data, (*user_data).x1 as i32);
    mnNameNew_8023B0F8(gobj, (*user_data).x1);
}
pub unsafe extern "C" fn mnNameNew_EnterFromMnName(mut arg0: *mut c_void) {
    let mut unused: [c_char; 8] = [0; 8];
    let mut text: *mut c_char = &raw mut mnNameNew_CurrentNameText
        as *mut c_char;
    mn_804A04F0.x10 = 1;
    if lbLang_IsSavedLanguageUS() != 0 {
        mn_804A04F0.hovered_selection = 0x2d;
    } else {
        mn_804A04F0.hovered_selection = 0;
    }
    mnNameNew_PortInUse = 4;
    *text.offset(0 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(3 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(6 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(9 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    mnNameNew_8023E32C(arg0 as i32);
}
pub unsafe extern "C" fn mnNameNew_EnterFromMnCharSel(
    mut arg0: *mut HSD_Archive,
    mut arg1: i32,
) {
    let mut name_count: u8 = 0;
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut text: *mut c_char = core::ptr::null_mut::<
        c_char,
    >();
    let mut is_us: i32 = 0;
    text = &raw mut mnNameNew_CurrentNameText as *mut c_char;
    mn_804D6BC8.cooldown = 0x14;
    mn_804D6BC8.x2 = 0;
    mn_804D6BC8.x4 = 0 as i32;
    mn_804A04F0.buttons = 0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x12 as MenuKind8;
    if lbLang_IsSavedLanguageUS() != 0 {
        mn_804A04F0.hovered_selection = 0x2d;
    } else {
        mn_804A04F0.hovered_selection = 0;
    }
    mn_804A04F0.x10 = 1;
    HSD_SisLib_803A5E70();
    mnNameNew_PortInUse = arg1 as u8;
    lbArchive_LoadSections(arg0, &[(&raw mut MenMainBack_Top.joint as *mut *mut c_void, c"MenMainBack_Top_joint".as_ptr()), (&raw mut MenMainBack_Top.animjoint as *mut *mut c_void, c"MenMainBack_Top_animjoint".as_ptr()), (&raw mut MenMainBack_Top.matanim_joint as *mut *mut c_void, c"MenMainBack_Top_matanim_joint".as_ptr()), (&raw mut MenMainBack_Top.shapeanim_joint as *mut *mut c_void, c"MenMainBack_Top_shapeanim_joint".as_ptr()), (&raw mut MenMain_cam as *mut *mut c_void, c"ScMenMain_cam_int1_camera".as_ptr()), (&raw mut MenMain_lights as *mut *mut c_void, c"ScMenMain_scene_lights".as_ptr()), (&raw mut MenMain_fog as *mut *mut c_void, c"ScMenMain_fog".as_ptr()), (&raw mut MenMainPanel_Top.joint as *mut *mut c_void, c"MenMainPanel_Top_joint".as_ptr()), (&raw mut MenMainPanel_Top.animjoint as *mut *mut c_void, c"MenMainPanel_Top_animjoint".as_ptr()), (&raw mut MenMainPanel_Top.matanim_joint as *mut *mut c_void, c"MenMainPanel_Top_matanim_joint".as_ptr()), (&raw mut MenMainPanel_Top.shapeanim_joint as *mut *mut c_void, c"MenMainPanel_Top_shapeanim_joint".as_ptr()), (&raw mut mnNameNew_804A06F0.joint as *mut *mut c_void, c"MenMainConEtNw_Top_joint".as_ptr()), (&raw mut mnNameNew_804A06F0.animjoint as *mut *mut c_void, c"MenMainConEtNw_Top_animjoint".as_ptr()), (&raw mut mnNameNew_804A06F0.matanim_joint as *mut *mut c_void, c"MenMainConEtNw_Top_matanim_joint".as_ptr()), (&raw mut mnNameNew_804A06F0.shapeanim_joint as *mut *mut c_void, c"MenMainConEtNw_Top_shapeanim_joint".as_ptr()), (&raw mut mnNameNew_804A0700.joint as *mut *mut c_void, c"MenMainBaseEtNw_Top_joint".as_ptr()), (&raw mut mnNameNew_804A0700.animjoint as *mut *mut c_void, c"MenMainBaseEtNw_Top_animjoint".as_ptr()), (&raw mut mnNameNew_804A0700.matanim_joint as *mut *mut c_void, c"MenMainBaseEtNw_Top_matanim_joint".as_ptr()), (&raw mut mnNameNew_804A0700.shapeanim_joint as *mut *mut c_void, c"MenMainBaseEtNw_Top_shapeanim_joint".as_ptr()), (&raw mut mnNameNew_804A0710.joint as *mut *mut c_void, c"MenMainSubEtNw_Top_joint".as_ptr()), (&raw mut mnNameNew_804A0710.animjoint as *mut *mut c_void, c"MenMainSubEtNw_Top_animjoint".as_ptr()), (&raw mut mnNameNew_804A0710.matanim_joint as *mut *mut c_void, c"MenMainSubEtNw_Top_matanim_joint".as_ptr()), (&raw mut mnNameNew_804A0710.shapeanim_joint as *mut *mut c_void, c"MenMainSubEtNw_Top_shapeanim_joint".as_ptr()), (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as i32 as isize))
            .joint as *mut *mut c_void, c"MenMainSbaseEtNw_Top_joint".as_ptr()), (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as i32 as isize))
            .animjoint as *mut *mut c_void, c"MenMainSbaseEtNw_Top_animjoint".as_ptr()), (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as i32 as isize))
            .matanim_joint as *mut *mut c_void, c"MenMainSbaseEtNw_Top_matanim_joint".as_ptr()), (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as i32 as isize))
            .shapeanim_joint as *mut *mut c_void, c"MenMainSbaseEtNw_Top_shapeanim_joint".as_ptr())]);
    is_us = lbLang_IsSavedLanguageUS() as i32;
    if is_us != 0 {
        lbArchive_LoadSections(arg0, &[(&raw mut AutoNamesList as *mut *mut c_void, c"mnNameAutoNameUs".as_ptr()), (&raw mut NotAllowedNamesList as *mut *mut c_void, c"mnNameRefuseNameUs".as_ptr())]);
    } else {
        lbArchive_LoadSections(arg0, &[(&raw mut AutoNamesList as *mut *mut c_void, c"mnNameAutoName".as_ptr()), (&raw mut NotAllowedNamesList as *mut *mut c_void, c"mnNameRefuseName".as_ptr())]);
    }
    mn_8022C304();
    mn_8022BCF8();
    mn_8022BEDC(mn_8022BE34());
    mn_80229B2C();
    mn_80229DC0();
    name_count = GetNameCount() as u8;
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0, 1, 0x80),
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(mnNameNew_MainInput)),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
    *text.offset(0 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(3 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(6 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    *text.offset(9 as i32 as isize) = *(&raw mut mnNameNew_NullCharacter
        as *mut c_char);
    mnNameNew_8023E32C(name_count as i32);
    lbAudioAx_80023F28(gmMainLib_8015ECB0());
}
pub unsafe extern "C" fn mnNameNew_8023EA08(mut arg0: *mut c_void) {
    mnNameNew_EnterFromMnCharSel(arg0 as *mut HSD_Archive, 4);
}
static mut mnNameNew_804D4F98: [u8; 8] = [
    0 as i32 as u8,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];
pub static mut mnNameNew_803EE720: [*mut c_char; 1] = unsafe {
    [&raw const mnNameNew_804D4F98 as *mut u8 as *mut c_char]
};
pub static mut mnNameNew_803EE724: [*mut c_char; 1] = unsafe {
    [&raw const mnNameNew_804D4F98 as *mut u8 as *mut c_char]
};
pub const NULL: i32 = 0;
