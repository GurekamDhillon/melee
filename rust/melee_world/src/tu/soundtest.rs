//! Port of src/melee/if/soundtest.c: the debug menu tree ("Sound Test Menu", the versus / adventure /
//! classic / all-star launchers, the memory-card test rows, the toy and event pages) and the tables that
//! describe it (strings, menu rows `un_80304138_objalloc_t_x8` with callback and value pointers, the
//! `un_803FA258` settings block the launchers read). A menu callback takes the menu event
//! (`0` = back, `1` = confirm, `6` = start) and returns nonzero when it handled the scene change.
//! The `TARGET_PC` branches of the C are what is ported (the two extra "Unlock All" rows, the separately
//! linked `un_803F9FA4` / `un_803FA258` objects). Callees of unported TUs go through `ext`; `OSReport`
//! diagnostics are not reproduced. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_soundtest.rs.
//!
//! Return values of the callbacks that fall off the end in the C (`un_802FF934`, `un_80300758`,
//! `un_80300790`, `fn_80300CC8`, `fn_80300DE0`, `fn_80300ED0`, `fn_803011EC`, `un_80301CE0`) are
//! undefined; the port returns 0 and the oracle does not compare them.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, clippy::all)]

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
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_SObjLib_803A477C(a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj = |a0: *mut HSD_GObj, a1: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8| unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A477C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _) as _ };
    fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A49E0(a0 as _, a1 as _); } };
    fn HSD_SObjLib_803A55DC(a0: *mut HSD_GObj, a1: u16, a2: u16, a3: i32) = |a0: *mut HSD_GObj, a1: u16, a2: u16, a3: i32| { unsafe { melee_runtime::tu::sobjlib::HSD_SObjLib_803A55DC(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Toy_80311680();
    fn Toy_803124BC();
    fn Toy_SetUnlockState(a0: i32, a1: i32);
    fn gmMainLib_8015CCF0() -> *mut u32 = || unsafe { crate::gmmain::gmMainLib_8015CCF0() as _ };
    fn gmMainLib_8015CDC8() -> *mut gmm_x0_528_t = || unsafe { crate::gmmain::gmMainLib_8015CDC8() as _ };
    fn gmMainLib_8015CDD4() -> *mut gmm_x0_528_t = || unsafe { crate::gmmain::gmMainLib_8015CDD4() as _ };
    fn gmMainLib_8015CDE0() -> *mut gmm_x0_528_t = || unsafe { crate::gmmain::gmMainLib_8015CDE0() as _ };
    fn gmMainLib_8015D06C(a0: u8) -> *mut i32 = |a0: u8| unsafe { crate::gmmain::gmMainLib_8015D06C(a0 as _) as _ };
    fn gmMainLib_8015D48C(a0: u8) -> i32 = |a0: u8| unsafe { crate::gmmain::gmMainLib_8015D48C(a0 as _) as _ };
    fn gmMainLib_8015D4E8(a0: u8, a1: i32) = |a0: u8, a1: i32| { unsafe { let _ = crate::gmmain::gmMainLib_8015D4E8(a0 as _, a1 as _); } };
    fn gmMainLib_8015FA34(a0: i32);
    fn gmMainLib_8015FB68();
    fn gmMainLib_GetBonusMatchTotal() -> *mut u32 = || unsafe { crate::gmmain::gmMainLib_GetBonusMatchTotal() as _ };
    fn gmMainLib_GetCoinMatchTotal() -> *mut u32 = || unsafe { crate::gmmain::gmMainLib_GetCoinMatchTotal() as _ };
    fn gmMainLib_GetKOTotal() -> *mut i32 = || unsafe { crate::gmmain::gmMainLib_GetKOTotal() as _ };
    fn gmMainLib_GetStockMatchTotal() -> *mut u32 = || unsafe { crate::gmmain::gmMainLib_GetStockMatchTotal() as _ };
    fn gmMainLib_GetTimeMatchTotal() -> *mut u32 = || unsafe { crate::gmmain::gmMainLib_GetTimeMatchTotal() as _ };
    fn gm_8016468C();
    fn gm_80164F18();
    fn gm_801A4B60();
    fn gm_801BEB74(a0: u8);
    fn gm_801BEFA4(a0: i32);
    fn gm_801BEFC0(a0: i32);
    fn gm_801BF040(a0: i8);
    fn gm_801BF708(a0: i8);
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32);
    fn gm_GetAllStarData() -> *mut UnkAllstarData;
    fn gm_SetNextGameModeStateId(a0: u8);
    fn gm_SetupAllPlayerDefaults(a0: *mut PlayerInitData);
    fn gm_SetupRulesDefaults(a0: *mut StartMeleeRules);
    fn lbArchive_LoadSymbols(a0: *const i8, a1: *mut c_void, a2: *const i8) = |a0: *const i8, a1: *mut c_void, a2: *const i8| { unsafe { let v: std::vec::Vec<*mut core::ffi::c_void> = std::vec![a1 as *mut core::ffi::c_void, a2 as *mut core::ffi::c_void, core::ptr::null_mut()]; let _ = melee_runtime::tu::lbarchive::lbArchive_LoadSymbols(a0 as _, &v); } };
    fn lbAudioAx_80023694() -> i32 = || unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694() as _ };
    fn lbAudioAx_800236DC() -> i32 = || unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() as _ };
    fn lbAudioAx_80023B24(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023B24(a0 as _) as _ };
    fn lbAudioAx_80023F28(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ };
    fn lbAudioAx_80024030(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } };
    fn lbAudioAx_800245D4(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800245D4(a0 as _); } };
    fn lbAudioAx_800245F4(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800245F4(a0 as _) as _ };
    fn lbAudioAx_80024614(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024614(a0 as _); } };
    fn lbAudioAx_80024634(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024634(a0 as _); } };
    fn lbAudioAx_80024BD0() -> i32 = || unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024BD0() as _ };
    fn lbAudioAx_80024C08(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024C08(a0 as _) as _ };
    fn lbAudioAx_80024E50(a0: i32) = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50(a0 != 0) };
    fn lbCardGame_LoadArchive(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(a0 as _); } };
    fn lbCardGame_SaveChanges() = || { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } };
    fn lbCardGame_SetCardStatus(a0: u32) = |a0: u32| { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SetCardStatus(a0 as _); } };
    fn lbCardNew_AllocWorkArea() = || { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } };
    fn lbCardNew_CompleteNextTask() -> i32 = || unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteNextTask() as _ };
    fn lbLang_SetSavedLanguage(a0: i32) = |a0: i32| { unsafe { melee_runtime::tu::lblanguage::lbLang_SetSavedLanguage(a0 as _); } };
    fn lbSnap_8001D40C(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D40C(a0 as _) as _ };
    fn lbSnap_8001D5FC(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D5FC(a0 as _, a1 as _) as _ };
    fn lbSnap_8001D7B0(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D7B0(a0 as _, a1 as _, a2 as _) as _ };
    fn lbSnap_8001DE8C(a0: *mut c_void) -> i32 = |a0: *mut c_void| unsafe { melee_runtime::tu::lbsnap::lbSnap_8001DE8C(a0 as _) as _ };
    fn lbSnap_8001E058(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E058(a0 as _, a1 as _) as _ };
    fn lbSnap_8001E204() -> i32 = || unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E204() as _ };
    fn lbSnap_8001E210() -> i32 = || unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E210() as _ };
    fn lbSnap_8001E218(a0: *mut c_void, a1: *mut c_void /* Unk80433380_48 */) = |a0: *mut c_void, a1: *mut c_void /* Unk80433380_48 */| { unsafe { melee_runtime::tu::lbsnap::lbSnap_8001E218(a0 as _, a1 as _); } };
    fn lb_8001B8C8(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbcardnew::lb_8001B8C8(a0 as _) as _ };
    fn lb_8001C87C() -> u32 = || unsafe { melee_runtime::tu::lbcardgame::lb_8001C87C() as _ };
    fn lb_8001C8BC() -> i32 = || unsafe { melee_runtime::tu::lbcardgame::lb_8001C8BC() as _ };
    fn lb_8001CBBC() -> u32 = || unsafe { melee_runtime::tu::lbcardgame::lb_8001CBBC() as _ };
    fn lb_8001CC4C() -> i32 = || unsafe { melee_runtime::tu::lbcardgame::lb_8001CC4C() as _ };
    fn un_80302DF0() -> *mut un_80304138_objalloc_t = || unsafe { crate::tu::textlib::un_80302DF0() as _ };
    fn un_80302DF8(a0: *mut un_80304138_objalloc_t, a1: Option<unsafe extern "C" fn(u32) -> i32>) = |a0: *mut un_80304138_objalloc_t, a1: Option<unsafe extern "C" fn(u32) -> i32>| { unsafe { crate::tu::textlib::un_80302DF8(a0 as _, a1 as _); } };
    fn un_80304210(a0: *mut un_80304138_objalloc_t, a1: *mut c_void, a2: i32, a3: i32, a4: i32) -> *mut un_80304138_objalloc_t = |a0: *mut un_80304138_objalloc_t, a1: *mut c_void, a2: i32, a3: i32, a4: i32| unsafe { crate::tu::textlib_1::un_80304210(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ };
    fn un_80304334(a0: *mut un_80304138_objalloc_t) = |a0: *mut un_80304138_objalloc_t| { unsafe { crate::tu::textlib_1::un_80304334(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_SObjLib_803A49E0(a0: *mut HSD_GObj, a1: i32) { super::HSD_SObjLib_803A49E0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::shared::DbLevel;
    pub use melee_runtime::tu::dbsound::db_804D4AF8;
    pub static mut db_804D6B88: i32 = unsafe { core::mem::zeroed() };
    pub static mut db_build_timestamp: [i8; 1] = unsafe { core::mem::zeroed() };
    pub static mut gmMainLib_8045A6C0: [u8; 73728] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `char name[N] = "..."`: the string padded with zeros to N bytes.
pub const fn cstr<const N: usize>(s: &[u8]) -> [i8; N] {
    let mut o = [0i8; N];
    let mut i = 0;
    while i < s.len() {
        o[i] = s[i] as i8;
        i += 1;
    }
    o
}

/// `struct SoundTestMenuData` (soundtest.h): the Sound Test menu's rows and the archive it loads.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SoundTestMenuData {
    pub entries: [un_80304138_objalloc_t_x8; 11],
    pub x160: [i8; 0xC],
    pub x16C: [i8; 0x18],
}

/// `struct UnkSoundTestData0` (soundtest.h), size 0xC.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnkSoundTestData0 {
    pub x0: u16,
    pub x2: u16,
    pub x4: i32,
    pub x8: i32,
}

/// `struct UnkSoundTestData1` (soundtest.h), size 0x20.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UnkSoundTestData1 {
    pub unk_0: i32,
    pub unk_4: i32,
    pub unk_8: u8,
    pub unk_9: u8,
    pub unk_A: u8,
    pub unk_B: u8,
    pub unk_C: u8,
    pub unk_D: u8,
    pub unk_E: u8,
    pub unk_F: u8,
    pub unk_10: u8,
    pub unk_11: u8,
    pub unk_12: u8,
    pub unk_13: u8,
    pub unk_14: u8,
    pub unk_15: u8,
    pub unk_16: u8,
    pub unk_17: u8,
    pub unk_18: u8,
    pub unk_19: u8,
    pub unk_1A: u8,
    pub unk_1B: u8,
    pub unk_1C: u8,
    pub unk_1D: u8,
    pub unk_1E: u8,
}

/// `struct un_803FA128_x130_t`: the overlay `un_802FFF2C` reads the settings through (the C casts the
/// address of `un_803FA258` to it, so its field offsets are the offsets into `un_803FA258_t`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_803FA128_x130_t {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    pub xC: i32,
    pub x10: [i32; 4],
    pub x20_pad: i32,
    pub x24: [i32; 4],
    pub x34_pad: i32,
    pub x38: [i32; 4],
    pub x48: [i32; 4],
    pub x58: [i32; 4],
    pub x68: [i32; 4],
    pub x78: [f32; 4],
    pub x88: [f32; 4],
    pub x98: [f32; 4],
    pub xA8: [i32; 4],
    pub xB8: [i32; 4],
    pub xC8: i32,
    pub xCC: [i32; 4],
    pub xDC: [i32; 4],
    pub xEC: f32,
}

/// `struct un_803FA258_t`: the debug menu's settings block (offsets are the GameCube's up to 0x138;
/// the pointer at 0x138 is native-sized).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_803FA258_t {
    pub x0: i32,
    pub x4: [i32; 4],
    pub x14: [i32; 4],
    pub x24: [i32; 4],
    pub x34: i32,
    pub x38: [i32; 4],
    pub x48: [i32; 4],
    pub x58: [i32; 4],
    pub x68: [i32; 4],
    pub x78: [f32; 4],
    pub x88: [f32; 4],
    pub x98: [f32; 4],
    pub xA8: [i32; 4],
    pub xB8: [i32; 4],
    pub xC8: i32,
    pub xCC: [i32; 4],
    pub xDC: [i32; 4],
    pub xEC: f32,
    pub xF0: u16,
    pub _padF2: [u8; 2],
    pub xF4: u8,
    pub xF5: u8,
    pub xF6: u8,
    pub xF7: u8,
    pub xF8: i32,
    pub xFC: i32,
    pub x100: i32,
    pub x104: i32,
    pub x108: i32,
    pub x10C: i32,
    pub x110: i32,
    pub x114: i32,
    pub x118: i32,
    pub x11C: i32,
    pub x120: i32,
    pub x124: i32,
    pub x128: i32,
    pub x12C: i32,
    pub x130: i32,
    pub x134: i32,
    pub x138: *mut c_void,
    pub x13C: i32,
    pub x140: i32,
    pub x144: [i32; 17],
    pub x188: i32,
    pub x18C: i32,
}

// BEGIN GENERATED DATA (rust/melee_world/gen/mtu/mk_soundtest_data.py from src/melee/if/soundtest.c)
pub static mut un_804D5850: i32 = unsafe { 127 };
pub static mut un_804D5854: i32 = unsafe { 127 };
pub static mut un_804D5858: i32 = unsafe { 127 };
pub static mut un_804D585C: i32 = unsafe { -1 };
pub static mut un_804D5860: [i8; 5] = unsafe { cstr::<5>(b"NONE") };
pub static mut un_804D5868: [i8; 8] = unsafe { cstr::<8>(b"ACTIVED") };
pub static mut un_804D5870: [i8; 5] = unsafe { cstr::<5>(b"AUTO") };
pub static mut un_804D5878: [i8; 4] = unsafe { cstr::<4>(b"OFF") };
pub static mut un_804D587C: [i8; 3] = unsafe { cstr::<3>(b"ON") };
pub static mut un_804D5880: [*mut i8; 2] = unsafe { [(&raw mut un_804D5878) as *mut _ as *mut i8, (&raw mut un_804D587C) as *mut _ as *mut i8] };
pub static mut un_804D5888: [*mut i8; 2] = unsafe { [(&raw mut un_803FA3F4) as *mut _ as *mut i8, (&raw mut un_803FA404) as *mut _ as *mut i8] };
pub static mut un_804D5890: [i8; 7] = unsafe { cstr::<7>(b"Master") };
pub static mut un_804D5898: [i8; 8] = unsafe { cstr::<8>(b"Develop") };
pub static mut un_804D58A0: [i8; 8] = unsafe { cstr::<8>(b"Addr4 :") };
pub static mut un_804D58A8: [i8; 8] = unsafe { cstr::<8>(b"u32   :") };
pub static mut un_804D58B0: [i8; 8] = unsafe { cstr::<8>(b"Addr2 :") };
pub static mut un_804D58B8: [i8; 8] = unsafe { cstr::<8>(b"u16   :") };
pub static mut un_804D58C0: [i8; 8] = unsafe { cstr::<8>(b"Addr  :") };
pub static mut un_804D58C8: [i8; 8] = unsafe { cstr::<8>(b"u8    :") };
pub static mut un_804D58D0: [i8; 8] = unsafe { cstr::<8>(b"Nothing") };
pub static mut un_804D58D8: [i8; 8] = unsafe { cstr::<8>(b"Verylow") };
pub static mut un_804D58E0: [i8; 4] = unsafe { cstr::<4>(b"Low") };
pub static mut un_804D58E4: [i8; 7] = unsafe { cstr::<7>(b"Middle") };
pub static mut un_804D58EC: [i8; 5] = unsafe { cstr::<5>(b"High") };
pub static mut un_804D58F4: [i8; 5] = unsafe { cstr::<5>(b"Rule") };
pub static mut un_804D58FC: [*mut i8; 2] = unsafe { [(&raw mut un_803FB064) as *mut _ as *mut i8, (&raw mut un_803FB074) as *mut _ as *mut i8] };
pub static mut un_804D5908: i32 = unsafe { 1 };
pub static mut un_804D590C: i32 = unsafe { 1 };
pub static mut un_804D5910: i32 = unsafe { 1 };
pub static mut un_804D5914: [i8; 7] = unsafe { cstr::<7>(b"SIMPLE") };
pub static mut un_804D591C: [i8; 8] = unsafe { cstr::<8>(b"Chara :") };
pub static mut un_804D5924: [i8; 8] = unsafe { cstr::<8>(b"Color :") };
pub static mut un_804D592C: [i8; 7] = unsafe { cstr::<7>(b"Mode :") };
pub static mut un_804D5934: [i8; 6] = unsafe { cstr::<6>(b"Luigi") };
pub static mut un_804D593C: [i8; 5] = unsafe { cstr::<5>(b"Mars") };
pub static mut un_804D5944: [i8; 7] = unsafe { cstr::<7>(b"Mewtwo") };
pub static mut un_804D594C: [i8; 6] = unsafe { cstr::<6>(b"Purin") };
pub static mut un_804D5954: [i8; 6] = unsafe { cstr::<6>(b"Falco") };
pub static mut un_804D595C: [i8; 7] = unsafe { cstr::<7>(b"C-Link") };
pub static mut un_804D5964: [i8; 4] = unsafe { cstr::<4>(b"Roi") };
pub static mut un_804D5968: [i8; 6] = unsafe { cstr::<6>(b"Pichu") };
pub static mut un_804D5970: [i8; 6] = unsafe { cstr::<6>(b"Ganon") };
pub static mut un_804D5978: [i8; 8] = unsafe { cstr::<8>(b"<Init>\n") };
pub static mut un_804D5980: [i8; 8] = unsafe { cstr::<8>(b"<Save>\n") };
pub static mut un_804D5988: [i8; 8] = unsafe { cstr::<8>(b"<Load>\n") };
pub static mut un_804D5990: HSD_SObjDesc = unsafe { HSD_SObjDesc { image: (&raw mut un_803FD274) as *mut _ as *mut HSD_ImageDesc, tlut: null_mut() } };
pub static mut un_804D5998: [i8; 5] = unsafe { cstr::<5>(b"Init") };
pub static mut un_804D59A0: [i8; 7] = unsafe { cstr::<7>(b"Format") };
pub static mut un_804D59A8: [i8; 7] = unsafe { cstr::<7>(b"Create") };
pub static mut un_804D59B0: [i8; 5] = unsafe { cstr::<5>(b"Save") };
pub static mut un_804D59B8: [i8; 5] = unsafe { cstr::<5>(b"Load") };
pub static mut un_804D59C0: [i8; 7] = unsafe { cstr::<7>(b"Delete") };
pub static mut un_804D59C8: [i8; 7] = unsafe { cstr::<7>(b"Normal") };
pub static mut un_804D59D0: [i8; 5] = unsafe { cstr::<5>(b"Team") };
pub static mut un_804D59D8: [i8; 6] = unsafe { cstr::<6>(b"Giant") };
pub static mut un_804D59E0: [i8; 6] = unsafe { cstr::<6>(b"Bonus") };
pub static mut un_804D59E8: [i8; 6] = unsafe { cstr::<6>(b"Corps") };
pub static mut un_804D59F0: [i8; 7] = unsafe { cstr::<7>(b"Target") };
pub static mut un_804D59F8: [i8; 7] = unsafe { cstr::<7>(b"TEST >") };
pub static mut un_804D6DA8: *mut i32 = unsafe { null_mut() };
pub static mut un_804D6DAC: i32 = unsafe { 0 };
pub static mut un_804D6DB0: i32 = unsafe { 0 };
pub static mut un_804D6DB4: i32 = unsafe { 0 };
pub static mut un_804D6DB8: i32 = unsafe { 0 };
pub static mut un_804D6DBC: i32 = unsafe { 0 };
pub static mut un_804D6DC0: i32 = unsafe { 0 };
pub static mut un_804D6DC8: i32 = unsafe { 0 };
pub static mut un_804D6DCC: i32 = unsafe { 0 };
pub static mut un_804D6DD0: f32 = unsafe { 0.0 };
pub static mut un_804D6DD4: i32 = unsafe { 0 };
pub static mut un_804D6DD8: i32 = unsafe { 0 };
pub static mut un_804D6DDC: i32 = unsafe { 0 };
pub static mut un_804D6DE0: i32 = unsafe { 0 };
pub static mut un_804D6DE4: i32 = unsafe { 0 };
pub static mut un_804D6DE8: i32 = unsafe { 0 };
pub static mut un_804D6DEC: i32 = unsafe { 0 };
pub static mut un_804D6DF0: i32 = unsafe { 0 };
pub static mut un_804D6DF4: i32 = unsafe { 0 };
pub static mut un_804D6DF8: i32 = unsafe { 0 };
pub static mut un_804D6DFC: i32 = unsafe { 0 };
pub static mut un_804D6E00: i32 = unsafe { 0 };
pub static mut un_804D6E04: *mut c_void = unsafe { null_mut() };
pub static mut un_804D6E08: *mut HSD_GObj = unsafe { null_mut() };
pub static mut un_804D6E0C: i32 = unsafe { 0 };
pub static mut un_804D6E10: i32 = unsafe { 0 };
pub static mut un_804D6E14: i32 = unsafe { 0 };
pub static mut gw_dev_unlock_chars_label: [i8; 22] = unsafe { cstr::<22>(b"Unlock All Characters") };
pub static mut gw_dev_unlock_stages_label: [i8; 18] = unsafe { cstr::<18>(b"Unlock All Stages") };
pub static mut lbl_803F9EF0: [i8; 32] = unsafe { cstr::<32>(b"Remove Target %x (n %x) Id %d\n") };
pub static mut lbl_803F9F10: [i8; 24] = unsafe { cstr::<24>(b"Remove All Over\n") };
pub static mut un_803F9F28: [i8; 16] = unsafe { cstr::<16>(b"Sound Test Menu") };
pub static mut un_803F9F38: [i8; 12] = unsafe { cstr::<12>(b"Sound Mode ") };
pub static mut un_803F9F44: [i8; 11] = unsafe { cstr::<11>(b"Mas Volume") };
pub static mut un_803F9F50: [i8; 11] = unsafe { cstr::<11>(b"FGM Volume") };
pub static mut un_803F9F5C: [i8; 11] = unsafe { cstr::<11>(b"BGM Volume") };
pub static mut un_803F9F68: [i8; 10] = unsafe { cstr::<10>(b"DSP Level") };
pub static mut un_803F9F74: [i8; 10] = unsafe { cstr::<10>(b"FGM Group") };
pub static mut un_803F9F80: [i8; 9] = unsafe { cstr::<9>(b"FGM Name") };
pub static mut un_803F9F8C: [i8; 9] = unsafe { cstr::<9>(b"BGM Name") };
pub static mut un_803F9F98: [i8; 11] = unsafe { cstr::<11>(b"-- EXIT --") };
pub static mut un_803F9FA4: SoundTestMenuData = unsafe { SoundTestMenuData { entries: [un_80304138_objalloc_t_x8 { x0: 0, x4: Some(un_802FF88C), x8: (&raw mut un_803F9F28) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_802FF934), x8: (&raw mut un_803F9F38) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DBC) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_802FF958), x8: (&raw mut un_803F9F44) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D5850) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_802FF958), x8: (&raw mut un_803F9F50) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D5854) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_802FF958), x8: (&raw mut un_803F9F5C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D5858) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_802FF99C), x8: (&raw mut un_803F9F68) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DAC) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_802FF9DC), x8: (&raw mut un_803F9F74) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DB0) as *mut _ as *mut c_void, x14: 0.0, x18: 55.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_802FFB58), x8: (&raw mut un_803F9F80) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DB4) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_802FFBAC), x8: (&raw mut un_803F9F8C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DB8) as *mut _ as *mut c_void, x14: 0.0, x18: 98.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_802FFC6C), x8: (&raw mut un_803F9F98) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: Some(un_802FFC30), x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }], x160: cstr::<12>(b"SmSt.dat"), x16C: cstr::<24>(b"smSoundTestLoadData") } };
pub static mut un_803FA128: [i32; 76] = unsafe { [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 2, 2, 5, 5, 4, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 5, 7, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5] };
pub static mut un_803FA258: un_803FA258_t = unsafe { un_803FA258_t { x0: 0, x4: [0, 32, 0, 6], x14: [8, 6, 6, 8], x24: [0, 0, 3, 3], x34: 0, x38: [0, 0, 0, 0], x48: [0, 0, 0, 0], x58: [0, 0, 0, 0], x68: [0, 0, 0, 0], x78: [1.0, 1.0, 1.0, 1.0], x88: [1.0, 1.0, 1.0, 1.0], x98: [1.0, 1.0, 1.0, 1.0], xA8: [4, 4, 4, 4], xB8: [5, 5, 5, 5], xC8: 0, xCC: [0, 0, 0, 0], xDC: [0, 0, 0, 0], xEC: 1.0, xF0: 0x0, _padF2: [0, 0], xF4: 0x0, xF5: 0x0, xF6: 0x0, xF7: 0x0, xF8: 0, xFC: 1, x100: 1, x104: 0, x108: 0, x10C: 1, x110: 0, x114: 0, x118: 0, x11C: 1, x120: 0, x124: 0, x128: 0, x12C: 0, x130: 0, x134: 0, x138: null_mut(), x13C: 0, x140: 0, x144: [0, 0, 0, 0, 0, 1, 1, 1, 3, 1, 2, 3, 3, 0, 0, 0, 0], x188: 1, x18C: 0 } };
pub static mut un_803FA3E8: [*mut i8; 3] = unsafe { [(&raw mut un_804D5860) as *mut _ as *mut i8, (&raw mut un_804D5868) as *mut _ as *mut i8, (&raw mut un_804D5870) as *mut _ as *mut i8] };
pub static mut un_803FA3F4: [i8; 13] = unsafe { cstr::<13>(b"GmLangTypeJP") };
pub static mut un_803FA404: [i8; 13] = unsafe { cstr::<13>(b"GmLangTypeUS") };
pub static mut un_803FA414: [i8; 13] = unsafe { cstr::<13>(b"No-Debug-Rom") };
pub static mut un_803FA424: [i8; 14] = unsafe { cstr::<14>(b"Debug-Develop") };
pub static mut un_803FA434: [i8; 10] = unsafe { cstr::<10>(b"Debug-Rom") };
pub static mut un_803FA440: [*mut i8; 5] = unsafe { [(&raw mut un_804D5890) as *mut _ as *mut i8, (&raw mut un_803FA414) as *mut _ as *mut i8, (&raw mut un_803FA424) as *mut _ as *mut i8, (&raw mut un_803FA434) as *mut _ as *mut i8, (&raw mut un_804D5898) as *mut _ as *mut i8] };
pub static mut un_803FA454: [i8; 17] = unsafe { cstr::<17>(b"Versus Mode    >") };
pub static mut un_803FA468: [i8; 12] = unsafe { cstr::<12>(b"Result Test") };
pub static mut un_803FA474: [i8; 12] = unsafe { cstr::<12>(b"Language : ") };
pub static mut un_803FA480: [i8; 12] = unsafe { cstr::<12>(b"Publicity: ") };
pub static mut un_803FA48C: [i8; 11] = unsafe { cstr::<11>(b"DbLevel : ") };
pub static mut un_803FA498: [i8; 16] = unsafe { cstr::<16>(b"Ik Debug Flag :") };
pub static mut un_803FA4A8: [i8; 14] = unsafe { cstr::<14>(b"New DefCalc :") };
pub static mut un_803FA4B8: [i8; 19] = unsafe { cstr::<19>(b"Global Data Edit >") };
pub static mut un_803FA4CC: [i8; 17] = unsafe { cstr::<17>(b"Mode Team Test >") };
pub static mut un_803FA4E0: [un_80304138_objalloc_t_x8; 13] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut ext::db_build_timestamp) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803001DC), x8: (&raw mut un_803FA454) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301420), x8: (&raw mut un_803FA468) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80300218), x8: (&raw mut un_803FA474) as *mut _ as *mut i8, xC: (&raw mut un_804D5888) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x0) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80300248), x8: (&raw mut un_803FA480) as *mut _ as *mut i8, xC: (&raw mut un_804D5880) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x4[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FA48C) as *mut _ as *mut i8, xC: (&raw mut un_803FA440) as *mut _ as *mut *mut i8, x10: (&raw mut ext::DbLevel) as *mut _ as *mut c_void, x14: 0.0, x18: 5.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FA498) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut ext::db_804D4AF8) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FA4A8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut ext::db_804D6B88) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300290), x8: (&raw mut un_803FA4B8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803002FC), x8: (&raw mut un_803FA4CC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(gw_dev_unlock_all_chars), x8: (&raw mut gw_dev_unlock_chars_label) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(gw_dev_unlock_all_stages), x8: (&raw mut gw_dev_unlock_stages_label) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FA640: [i8; 21] = unsafe { cstr::<21>(b"< Global Data Edit >") };
pub static mut un_803FA658: [un_80304138_objalloc_t_x8; 8] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FA640) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 6, x4: Some(un_803003C4), x8: (&raw mut un_804D58A0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF0) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 4.0 }, un_80304138_objalloc_t_x8 { x0: 7, x4: Some(un_80300410), x8: (&raw mut un_804D58A8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF4) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 6, x4: Some(un_80300378), x8: (&raw mut un_804D58B0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF0) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 2.0 }, un_80304138_objalloc_t_x8 { x0: 6, x4: Some(un_80300410), x8: (&raw mut un_804D58B8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF4) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 6, x4: Some(un_80300338), x8: (&raw mut un_804D58C0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF0) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 5, x4: Some(un_80300410), x8: (&raw mut un_804D58C8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF4) as *mut _ as *mut c_void, x14: 0.0, x18: 0.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FA758: [i8; 14] = unsafe { cstr::<14>(b"<VERSUS MODE>") };
pub static mut un_803FA768: [i8; 19] = unsafe { cstr::<19>(b"MODE : DaiRanTou >") };
pub static mut un_803FA77C: [i8; 19] = unsafe { cstr::<19>(b"MODE : Rule      >") };
pub static mut un_803FA790: [un_80304138_objalloc_t_x8; 4] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FA758) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300480), x8: (&raw mut un_803FA768) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803004B4), x8: (&raw mut un_803FA77C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FA810: [i8; 11] = unsafe { cstr::<11>(b"Time Mode ") };
pub static mut un_803FA81C: [i8; 11] = unsafe { cstr::<11>(b"Stock Mode") };
pub static mut un_803FA828: [i8; 10] = unsafe { cstr::<10>(b"Coin Mode") };
pub static mut un_803FA834: [i8; 13] = unsafe { cstr::<13>(b"Endless Mode") };
pub static mut un_803FA844: [*mut i8; 4] = unsafe { [(&raw mut un_803FA810) as *mut _ as *mut i8, (&raw mut un_803FA81C) as *mut _ as *mut i8, (&raw mut un_803FA828) as *mut _ as *mut i8, (&raw mut un_803FA834) as *mut _ as *mut i8] };
pub static mut un_803FA854: [i8; 9] = unsafe { cstr::<9>(b"Veryhigh") };
pub static mut un_803FA860: [*mut i8; 6] = unsafe { [(&raw mut un_804D58D0) as *mut _ as *mut i8, (&raw mut un_804D58D8) as *mut _ as *mut i8, (&raw mut un_804D58E0) as *mut _ as *mut i8, (&raw mut un_804D58E4) as *mut _ as *mut i8, (&raw mut un_804D58EC) as *mut _ as *mut i8, (&raw mut un_803FA854) as *mut _ as *mut i8] };
pub static mut un_803FA878: [i8; 9] = unsafe { cstr::<9>(b"< Rule >") };
pub static mut un_803FA884: [i8; 13] = unsafe { cstr::<13>(b"Time(Min)  :") };
pub static mut un_803FA894: [i8; 13] = unsafe { cstr::<13>(b"Time(Sec)  :") };
pub static mut un_803FA8A4: [i8; 13] = unsafe { cstr::<13>(b"Stock(cnt) :") };
pub static mut un_803FA8B4: [i8; 13] = unsafe { cstr::<13>(b"DamageRatio:") };
pub static mut un_803FA8C4: [i8; 20] = unsafe { cstr::<20>(b"Vibration Select ->") };
pub static mut un_803FA8D8: [i8; 13] = unsafe { cstr::<13>(b"Item Switch:") };
pub static mut un_803FA8E8: [un_80304138_objalloc_t_x8; 9] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FA878) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_804D58F4) as *mut _ as *mut i8, xC: (&raw mut un_803FA844) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xC8) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FA884) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xCC[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 100.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FA894) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xCC[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 59.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FA8A4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xCC[2]) as *mut _ as *mut c_void, x14: -1.0, x18: 99.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FA8B4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xEC) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803004E8), x8: (&raw mut un_803FA8C4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FA8D8) as *mut _ as *mut i8, xC: (&raw mut un_803FA860) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xCC[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 6.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FAA08: [i8; 11] = unsafe { cstr::<11>(b"< Rumble >") };
pub static mut un_803FAA14: [i8; 11] = unsafe { cstr::<11>(b"Rumble_1 :") };
pub static mut un_803FAA20: [i8; 11] = unsafe { cstr::<11>(b"Rumble_2 :") };
pub static mut un_803FAA2C: [i8; 11] = unsafe { cstr::<11>(b"Rumble_3 :") };
pub static mut un_803FAA38: [i8; 11] = unsafe { cstr::<11>(b"Rumble_4 :") };
pub static mut un_803FAA44: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FAA08) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FAA14) as *mut _ as *mut i8, xC: (&raw mut un_804D5880) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xDC[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FAA20) as *mut _ as *mut i8, xC: (&raw mut un_804D5880) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xDC[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FAA2C) as *mut _ as *mut i8, xC: (&raw mut un_804D5880) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xDC[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FAA38) as *mut _ as *mut i8, xC: (&raw mut un_804D5880) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xDC[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FAB04: [i8; 9] = unsafe { cstr::<9>(b"dummy   ") };
pub static mut un_803FAB10: [i8; 9] = unsafe { cstr::<9>(b"TEST    ") };
pub static mut un_803FAB1C: [i8; 9] = unsafe { cstr::<9>(b"Izumi   ") };
pub static mut un_803FAB28: [i8; 9] = unsafe { cstr::<9>(b"PStadium") };
pub static mut un_803FAB34: [i8; 9] = unsafe { cstr::<9>(b"Castle  ") };
pub static mut un_803FAB40: [i8; 9] = unsafe { cstr::<9>(b"Kongo   ") };
pub static mut un_803FAB4C: [i8; 9] = unsafe { cstr::<9>(b"Zebes   ") };
pub static mut un_803FAB58: [i8; 9] = unsafe { cstr::<9>(b"Corneria") };
pub static mut un_803FAB64: [i8; 9] = unsafe { cstr::<9>(b"Story   ") };
pub static mut un_803FAB70: [i8; 9] = unsafe { cstr::<9>(b"Onett   ") };
pub static mut un_803FAB7C: [i8; 9] = unsafe { cstr::<9>(b"MuteCity") };
pub static mut un_803FAB88: [i8; 9] = unsafe { cstr::<9>(b"RCruise ") };
pub static mut un_803FAB94: [i8; 9] = unsafe { cstr::<9>(b"Garden  ") };
pub static mut un_803FABA0: [i8; 9] = unsafe { cstr::<9>(b"GreatBay") };
pub static mut un_803FABAC: [i8; 9] = unsafe { cstr::<9>(b"Shrine  ") };
pub static mut un_803FABB8: [i8; 9] = unsafe { cstr::<9>(b"Kraid   ") };
pub static mut un_803FABC4: [i8; 9] = unsafe { cstr::<9>(b"Yoster  ") };
pub static mut un_803FABD0: [i8; 9] = unsafe { cstr::<9>(b"Greens  ") };
pub static mut un_803FABDC: [i8; 9] = unsafe { cstr::<9>(b"Fourside") };
pub static mut un_803FABE8: [i8; 9] = unsafe { cstr::<9>(b"Inishie1") };
pub static mut un_803FABF4: [i8; 9] = unsafe { cstr::<9>(b"Inishie2") };
pub static mut un_803FAC00: [i8; 9] = unsafe { cstr::<9>(b"Akaneia ") };
pub static mut un_803FAC0C: [i8; 9] = unsafe { cstr::<9>(b"Venom   ") };
pub static mut un_803FAC18: [i8; 9] = unsafe { cstr::<9>(b"Pura    ") };
pub static mut un_803FAC24: [i8; 9] = unsafe { cstr::<9>(b"BigBlue ") };
pub static mut un_803FAC30: [i8; 9] = unsafe { cstr::<9>(b"Icemt   ") };
pub static mut un_803FAC3C: [i8; 9] = unsafe { cstr::<9>(b"Icetop  ") };
pub static mut un_803FAC48: [i8; 9] = unsafe { cstr::<9>(b"Flatzone") };
pub static mut un_803FAC54: [i8; 9] = unsafe { cstr::<9>(b"old ppp ") };
pub static mut un_803FAC60: [i8; 9] = unsafe { cstr::<9>(b"old yosh") };
pub static mut un_803FAC6C: [i8; 9] = unsafe { cstr::<9>(b"old kong") };
pub static mut un_803FAC78: [i8; 9] = unsafe { cstr::<9>(b"battle  ") };
pub static mut un_803FAC84: [i8; 9] = unsafe { cstr::<9>(b"last    ") };
pub static mut un_803FAC90: [i8; 9] = unsafe { cstr::<9>(b"TMario  ") };
pub static mut un_803FAC9C: [i8; 9] = unsafe { cstr::<9>(b"TCaptain") };
pub static mut un_803FACA8: [i8; 9] = unsafe { cstr::<9>(b"TClink  ") };
pub static mut un_803FACB4: [i8; 9] = unsafe { cstr::<9>(b"TDonkey ") };
pub static mut un_803FACC0: [i8; 9] = unsafe { cstr::<9>(b"TDrmario") };
pub static mut un_803FACCC: [i8; 9] = unsafe { cstr::<9>(b"TFalco  ") };
pub static mut un_803FACD8: [i8; 9] = unsafe { cstr::<9>(b"TFox    ") };
pub static mut un_803FACE4: [i8; 9] = unsafe { cstr::<9>(b"TIceclim") };
pub static mut un_803FACF0: [i8; 9] = unsafe { cstr::<9>(b"TKirby  ") };
pub static mut un_803FACFC: [i8; 9] = unsafe { cstr::<9>(b"TKoopa  ") };
pub static mut un_803FAD08: [i8; 9] = unsafe { cstr::<9>(b"TLink   ") };
pub static mut un_803FAD14: [i8; 9] = unsafe { cstr::<9>(b"TLuigi  ") };
pub static mut un_803FAD20: [i8; 9] = unsafe { cstr::<9>(b"TMars   ") };
pub static mut un_803FAD2C: [i8; 9] = unsafe { cstr::<9>(b"TMewtwo ") };
pub static mut un_803FAD38: [i8; 9] = unsafe { cstr::<9>(b"TNess   ") };
pub static mut un_803FAD44: [i8; 9] = unsafe { cstr::<9>(b"TPeach  ") };
pub static mut un_803FAD50: [i8; 9] = unsafe { cstr::<9>(b"TPichu  ") };
pub static mut un_803FAD5C: [i8; 9] = unsafe { cstr::<9>(b"TPikachu") };
pub static mut un_803FAD68: [i8; 9] = unsafe { cstr::<9>(b"TPurin  ") };
pub static mut un_803FAD74: [i8; 9] = unsafe { cstr::<9>(b"TSamus  ") };
pub static mut un_803FAD80: [i8; 9] = unsafe { cstr::<9>(b"TSeak   ") };
pub static mut un_803FAD8C: [i8; 9] = unsafe { cstr::<9>(b"TYoshi  ") };
pub static mut un_803FAD98: [i8; 9] = unsafe { cstr::<9>(b"TZelda  ") };
pub static mut un_803FADA4: [i8; 9] = unsafe { cstr::<9>(b"TGamewat") };
pub static mut un_803FADB0: [i8; 9] = unsafe { cstr::<9>(b"TEmblem ") };
pub static mut un_803FADBC: [i8; 9] = unsafe { cstr::<9>(b"TGanon  ") };
pub static mut un_803FADC8: [i8; 11] = unsafe { cstr::<11>(b"1-1kinoko ") };
pub static mut un_803FADD4: [i8; 11] = unsafe { cstr::<11>(b"1-2castle ") };
pub static mut un_803FADE0: [i8; 11] = unsafe { cstr::<11>(b"2-1kongo  ") };
pub static mut un_803FADEC: [i8; 11] = unsafe { cstr::<11>(b"2-2garden ") };
pub static mut un_803FADF8: [i8; 11] = unsafe { cstr::<11>(b"3-1meikyu ") };
pub static mut un_803FAE04: [i8; 11] = unsafe { cstr::<11>(b"3-2shrine ") };
pub static mut un_803FAE10: [i8; 11] = unsafe { cstr::<11>(b"4-1zebes  ") };
pub static mut un_803FAE1C: [i8; 11] = unsafe { cstr::<11>(b"4-2dassyut") };
pub static mut un_803FAE28: [i8; 11] = unsafe { cstr::<11>(b"5-1greens ") };
pub static mut un_803FAE34: [i8; 11] = unsafe { cstr::<11>(b"5-2greens ") };
pub static mut un_803FAE40: [i8; 11] = unsafe { cstr::<11>(b"5-3greens ") };
pub static mut un_803FAE4C: [i8; 11] = unsafe { cstr::<11>(b"6-1corneri") };
pub static mut un_803FAE58: [i8; 11] = unsafe { cstr::<11>(b"6-2corneri") };
pub static mut un_803FAE64: [i8; 11] = unsafe { cstr::<11>(b"7pstadium ") };
pub static mut un_803FAE70: [i8; 11] = unsafe { cstr::<11>(b"8-1bbroute") };
pub static mut un_803FAE7C: [i8; 11] = unsafe { cstr::<11>(b"8-2mutecit") };
pub static mut un_803FAE88: [i8; 11] = unsafe { cstr::<11>(b"9-1onett  ") };
pub static mut un_803FAE94: [i8; 11] = unsafe { cstr::<11>(b"10-1icemt ") };
pub static mut un_803FAEA0: [i8; 11] = unsafe { cstr::<11>(b"10-2      ") };
pub static mut un_803FAEAC: [i8; 11] = unsafe { cstr::<11>(b"11-1battle") };
pub static mut un_803FAEB8: [i8; 11] = unsafe { cstr::<11>(b"11-2battle") };
pub static mut un_803FAEC4: [i8; 11] = unsafe { cstr::<11>(b"12-1last  ") };
pub static mut un_803FAED0: [i8; 11] = unsafe { cstr::<11>(b"12-2last  ") };
pub static mut un_803FAEDC: [i8; 11] = unsafe { cstr::<11>(b"tukisusume") };
pub static mut un_803FAEE8: [i8; 11] = unsafe { cstr::<11>(b"figureget ") };
pub static mut un_803FAEF4: [i8; 11] = unsafe { cstr::<11>(b"homerun   ") };
pub static mut un_803FAF00: [i8; 11] = unsafe { cstr::<11>(b"heal      ") };
pub static mut un_803FAF0C: [*mut i8; 86] = unsafe { [(&raw mut un_803FAB04) as *mut _ as *mut i8, (&raw mut un_803FAB10) as *mut _ as *mut i8, (&raw mut un_803FAB1C) as *mut _ as *mut i8, (&raw mut un_803FAB28) as *mut _ as *mut i8, (&raw mut un_803FAB34) as *mut _ as *mut i8, (&raw mut un_803FAB40) as *mut _ as *mut i8, (&raw mut un_803FAB4C) as *mut _ as *mut i8, (&raw mut un_803FAB58) as *mut _ as *mut i8, (&raw mut un_803FAB64) as *mut _ as *mut i8, (&raw mut un_803FAB70) as *mut _ as *mut i8, (&raw mut un_803FAB7C) as *mut _ as *mut i8, (&raw mut un_803FAB88) as *mut _ as *mut i8, (&raw mut un_803FAB94) as *mut _ as *mut i8, (&raw mut un_803FABA0) as *mut _ as *mut i8, (&raw mut un_803FABAC) as *mut _ as *mut i8, (&raw mut un_803FABB8) as *mut _ as *mut i8, (&raw mut un_803FABC4) as *mut _ as *mut i8, (&raw mut un_803FABD0) as *mut _ as *mut i8, (&raw mut un_803FABDC) as *mut _ as *mut i8, (&raw mut un_803FABE8) as *mut _ as *mut i8, (&raw mut un_803FABF4) as *mut _ as *mut i8, (&raw mut un_803FAC00) as *mut _ as *mut i8, (&raw mut un_803FAC0C) as *mut _ as *mut i8, (&raw mut un_803FAC18) as *mut _ as *mut i8, (&raw mut un_803FAC24) as *mut _ as *mut i8, (&raw mut un_803FAC30) as *mut _ as *mut i8, (&raw mut un_803FAC3C) as *mut _ as *mut i8, (&raw mut un_803FAC48) as *mut _ as *mut i8, (&raw mut un_803FAC54) as *mut _ as *mut i8, (&raw mut un_803FAC60) as *mut _ as *mut i8, (&raw mut un_803FAC6C) as *mut _ as *mut i8, (&raw mut un_803FAC78) as *mut _ as *mut i8, (&raw mut un_803FAC84) as *mut _ as *mut i8, (&raw mut un_803FAC90) as *mut _ as *mut i8, (&raw mut un_803FAC9C) as *mut _ as *mut i8, (&raw mut un_803FACA8) as *mut _ as *mut i8, (&raw mut un_803FACB4) as *mut _ as *mut i8, (&raw mut un_803FACC0) as *mut _ as *mut i8, (&raw mut un_803FACCC) as *mut _ as *mut i8, (&raw mut un_803FACD8) as *mut _ as *mut i8, (&raw mut un_803FACE4) as *mut _ as *mut i8, (&raw mut un_803FACF0) as *mut _ as *mut i8, (&raw mut un_803FACFC) as *mut _ as *mut i8, (&raw mut un_803FAD08) as *mut _ as *mut i8, (&raw mut un_803FAD14) as *mut _ as *mut i8, (&raw mut un_803FAD20) as *mut _ as *mut i8, (&raw mut un_803FAD2C) as *mut _ as *mut i8, (&raw mut un_803FAD38) as *mut _ as *mut i8, (&raw mut un_803FAD44) as *mut _ as *mut i8, (&raw mut un_803FAD50) as *mut _ as *mut i8, (&raw mut un_803FAD5C) as *mut _ as *mut i8, (&raw mut un_803FAD68) as *mut _ as *mut i8, (&raw mut un_803FAD74) as *mut _ as *mut i8, (&raw mut un_803FAD80) as *mut _ as *mut i8, (&raw mut un_803FAD8C) as *mut _ as *mut i8, (&raw mut un_803FAD98) as *mut _ as *mut i8, (&raw mut un_803FADA4) as *mut _ as *mut i8, (&raw mut un_803FADB0) as *mut _ as *mut i8, (&raw mut un_803FADBC) as *mut _ as *mut i8, (&raw mut un_803FADC8) as *mut _ as *mut i8, (&raw mut un_803FADD4) as *mut _ as *mut i8, (&raw mut un_803FADE0) as *mut _ as *mut i8, (&raw mut un_803FADEC) as *mut _ as *mut i8, (&raw mut un_803FADF8) as *mut _ as *mut i8, (&raw mut un_803FAE04) as *mut _ as *mut i8, (&raw mut un_803FAE10) as *mut _ as *mut i8, (&raw mut un_803FAE1C) as *mut _ as *mut i8, (&raw mut un_803FAE28) as *mut _ as *mut i8, (&raw mut un_803FAE34) as *mut _ as *mut i8, (&raw mut un_803FAE40) as *mut _ as *mut i8, (&raw mut un_803FAE4C) as *mut _ as *mut i8, (&raw mut un_803FAE58) as *mut _ as *mut i8, (&raw mut un_803FAE64) as *mut _ as *mut i8, (&raw mut un_803FAE70) as *mut _ as *mut i8, (&raw mut un_803FAE7C) as *mut _ as *mut i8, (&raw mut un_803FAE88) as *mut _ as *mut i8, (&raw mut un_803FAE94) as *mut _ as *mut i8, (&raw mut un_803FAEA0) as *mut _ as *mut i8, (&raw mut un_803FAEAC) as *mut _ as *mut i8, (&raw mut un_803FAEB8) as *mut _ as *mut i8, (&raw mut un_803FAEC4) as *mut _ as *mut i8, (&raw mut un_803FAED0) as *mut _ as *mut i8, (&raw mut un_803FAEDC) as *mut _ as *mut i8, (&raw mut un_803FAEE8) as *mut _ as *mut i8, (&raw mut un_803FAEF4) as *mut _ as *mut i8, (&raw mut un_803FAF00) as *mut _ as *mut i8] };
pub static mut un_803FB064: [i8; 13] = unsafe { cstr::<13>(b"Battle Royal") };
pub static mut un_803FB074: [i8; 12] = unsafe { cstr::<12>(b"Team Battle") };
pub static mut un_803FB080: [i8; 14] = unsafe { cstr::<14>(b"< DaiRanTou >") };
pub static mut un_803FB090: [i8; 16] = unsafe { cstr::<16>(b"Char  Select ->") };
pub static mut un_803FB0A0: [i8; 16] = unsafe { cstr::<16>(b"Scale Select ->") };
pub static mut un_803FB0B0: [i8; 16] = unsafe { cstr::<16>(b"Kind  Select ->") };
pub static mut un_803FB0C0: [i8; 16] = unsafe { cstr::<16>(b"Color Select ->") };
pub static mut un_803FB0D0: [i8; 16] = unsafe { cstr::<16>(b"  Sub Color  ->") };
pub static mut un_803FB0E0: [i8; 16] = unsafe { cstr::<16>(b"Damage Set   ->") };
pub static mut un_803FB0F0: [i8; 16] = unsafe { cstr::<16>(b"  OffenceRatio>") };
pub static mut un_803FB100: [i8; 16] = unsafe { cstr::<16>(b"  DefenceRatio>") };
pub static mut un_803FB110: [i8; 16] = unsafe { cstr::<16>(b"CpuType  Set ->") };
pub static mut un_803FB120: [i8; 16] = unsafe { cstr::<16>(b"CpuLevel Set ->") };
pub static mut un_803FB130: [i8; 16] = unsafe { cstr::<16>(b"Team  Select ->") };
pub static mut un_803FB140: [i8; 10] = unsafe { cstr::<10>(b"Stage   :") };
pub static mut un_803FB14C: [i8; 12] = unsafe { cstr::<12>(b"MeleeKind :") };
pub static mut un_803FB158: [i8; 16] = unsafe { cstr::<16>(b"-----------EXIT") };
pub static mut un_803FB168: [un_80304138_objalloc_t_x8; 16] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FB080) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_8030051C), x8: (&raw mut un_803FB090) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300550), x8: (&raw mut un_803FB0A0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300584), x8: (&raw mut un_803FB0B0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803005B8), x8: (&raw mut un_803FB0C0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803005EC), x8: (&raw mut un_803FB0D0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300620), x8: (&raw mut un_803FB0E0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300654), x8: (&raw mut un_803FB0F0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300688), x8: (&raw mut un_803FB100) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803006BC), x8: (&raw mut un_803FB110) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803006F0), x8: (&raw mut un_803FB120) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300724), x8: (&raw mut un_803FB130) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB140) as *mut _ as *mut i8, xC: (&raw mut un_803FAF0C) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x4[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 86.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB14C) as *mut _ as *mut i8, xC: (&raw mut un_804D58FC) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x4[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 2.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_802FFEA4), x8: (&raw mut un_803FB158) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FB368: [i8; 14] = unsafe { cstr::<14>(b"CKind_Captain") };
pub static mut un_803FB378: [i8; 13] = unsafe { cstr::<13>(b"CKind_Donkey") };
pub static mut un_803FB388: [i8; 10] = unsafe { cstr::<10>(b"CKind_Fox") };
pub static mut un_803FB394: [i8; 16] = unsafe { cstr::<16>(b"CKind_GameWatch") };
pub static mut un_803FB3A4: [i8; 12] = unsafe { cstr::<12>(b"CKind_Kirby") };
pub static mut un_803FB3B0: [i8; 12] = unsafe { cstr::<12>(b"CKind_Koopa") };
pub static mut un_803FB3BC: [i8; 11] = unsafe { cstr::<11>(b"CKind_Link") };
pub static mut un_803FB3C8: [i8; 12] = unsafe { cstr::<12>(b"CKind_Luigi") };
pub static mut un_803FB3D4: [i8; 12] = unsafe { cstr::<12>(b"CKind_Mario") };
pub static mut un_803FB3E0: [i8; 11] = unsafe { cstr::<11>(b"CKind_Mars") };
pub static mut un_803FB3EC: [i8; 13] = unsafe { cstr::<13>(b"CKind_Mewtwo") };
pub static mut un_803FB3FC: [i8; 11] = unsafe { cstr::<11>(b"CKind_Ness") };
pub static mut un_803FB408: [i8; 12] = unsafe { cstr::<12>(b"CKind_Peach") };
pub static mut un_803FB414: [i8; 14] = unsafe { cstr::<14>(b"CKind_Pikachu") };
pub static mut un_803FB424: [i8; 15] = unsafe { cstr::<15>(b"CKind_PopoNana") };
pub static mut un_803FB434: [i8; 12] = unsafe { cstr::<12>(b"CKind_Purin") };
pub static mut un_803FB440: [i8; 12] = unsafe { cstr::<12>(b"CKind_Samus") };
pub static mut un_803FB44C: [i8; 12] = unsafe { cstr::<12>(b"CKind_Yoshi") };
pub static mut un_803FB458: [i8; 13] = unsafe { cstr::<13>(b"CKind_Ze->Se") };
pub static mut un_803FB468: [i8; 13] = unsafe { cstr::<13>(b"CKind_Se->Ze") };
pub static mut un_803FB478: [i8; 12] = unsafe { cstr::<12>(b"CKind_Falco") };
pub static mut un_803FB484: [i8; 12] = unsafe { cstr::<12>(b"CKind_Clink") };
pub static mut un_803FB490: [i8; 14] = unsafe { cstr::<14>(b"CKind_Drmario") };
pub static mut un_803FB4A0: [i8; 13] = unsafe { cstr::<13>(b"CKind_Emblem") };
pub static mut un_803FB4B0: [i8; 12] = unsafe { cstr::<12>(b"CKind_Pichu") };
pub static mut un_803FB4BC: [i8; 12] = unsafe { cstr::<12>(b"CKind_Ganon") };
pub static mut un_803FB4C8: [i8; 14] = unsafe { cstr::<14>(b"CKind_MasterH") };
pub static mut un_803FB4D8: [i8; 10] = unsafe { cstr::<10>(b"CKind_Boy") };
pub static mut un_803FB4E4: [i8; 11] = unsafe { cstr::<11>(b"CKind_Girl") };
pub static mut un_803FB4F0: [i8; 13] = unsafe { cstr::<13>(b"CKind_GKoops") };
pub static mut un_803FB500: [i8; 13] = unsafe { cstr::<13>(b"Ckind_CrezyH") };
pub static mut un_803FB510: [i8; 15] = unsafe { cstr::<15>(b"ChKind_Sandbag") };
pub static mut un_803FB520: [i8; 12] = unsafe { cstr::<12>(b"ChKind_Popo") };
pub static mut un_803FB52C: [i8; 12] = unsafe { cstr::<12>(b"ChKind_None") };
pub static mut un_803FB538: [*mut i8; 34] = unsafe { [(&raw mut un_803FB368) as *mut _ as *mut i8, (&raw mut un_803FB378) as *mut _ as *mut i8, (&raw mut un_803FB388) as *mut _ as *mut i8, (&raw mut un_803FB394) as *mut _ as *mut i8, (&raw mut un_803FB3A4) as *mut _ as *mut i8, (&raw mut un_803FB3B0) as *mut _ as *mut i8, (&raw mut un_803FB3BC) as *mut _ as *mut i8, (&raw mut un_803FB3C8) as *mut _ as *mut i8, (&raw mut un_803FB3D4) as *mut _ as *mut i8, (&raw mut un_803FB3E0) as *mut _ as *mut i8, (&raw mut un_803FB3EC) as *mut _ as *mut i8, (&raw mut un_803FB3FC) as *mut _ as *mut i8, (&raw mut un_803FB408) as *mut _ as *mut i8, (&raw mut un_803FB414) as *mut _ as *mut i8, (&raw mut un_803FB424) as *mut _ as *mut i8, (&raw mut un_803FB434) as *mut _ as *mut i8, (&raw mut un_803FB440) as *mut _ as *mut i8, (&raw mut un_803FB44C) as *mut _ as *mut i8, (&raw mut un_803FB458) as *mut _ as *mut i8, (&raw mut un_803FB468) as *mut _ as *mut i8, (&raw mut un_803FB478) as *mut _ as *mut i8, (&raw mut un_803FB484) as *mut _ as *mut i8, (&raw mut un_803FB490) as *mut _ as *mut i8, (&raw mut un_803FB4A0) as *mut _ as *mut i8, (&raw mut un_803FB4B0) as *mut _ as *mut i8, (&raw mut un_803FB4BC) as *mut _ as *mut i8, (&raw mut un_803FB4C8) as *mut _ as *mut i8, (&raw mut un_803FB4D8) as *mut _ as *mut i8, (&raw mut un_803FB4E4) as *mut _ as *mut i8, (&raw mut un_803FB4F0) as *mut _ as *mut i8, (&raw mut un_803FB500) as *mut _ as *mut i8, (&raw mut un_803FB510) as *mut _ as *mut i8, (&raw mut un_803FB520) as *mut _ as *mut i8, (&raw mut un_803FB52C) as *mut _ as *mut i8] };
pub static mut un_803FB5C0: [i8; 15] = unsafe { cstr::<15>(b"< CharSelect >") };
pub static mut un_803FB5D0: [i8; 10] = unsafe { cstr::<10>(b"Chara_1 :") };
pub static mut un_803FB5DC: [i8; 10] = unsafe { cstr::<10>(b"Chara_2 :") };
pub static mut un_803FB5E8: [i8; 10] = unsafe { cstr::<10>(b"Chara_3 :") };
pub static mut un_803FB5F4: [i8; 10] = unsafe { cstr::<10>(b"Chara_4 :") };
pub static mut un_803FB600: [i8; 11] = unsafe { cstr::<11>(b"All Char :") };
pub static mut un_803FB60C: [un_80304138_objalloc_t_x8; 7] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FB5C0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB5D0) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x4[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB5DC) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x14[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB5E8) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x14[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB5F4) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x14[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80300758), x8: (&raw mut un_803FB600) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x14[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FB6EC: [i8; 10] = unsafe { cstr::<10>(b"< Scale >") };
pub static mut un_803FB6F8: [i8; 11] = unsafe { cstr::<11>(b"Scale__1 :") };
pub static mut un_803FB704: [i8; 11] = unsafe { cstr::<11>(b"Scale__2 :") };
pub static mut un_803FB710: [i8; 11] = unsafe { cstr::<11>(b"Scale__3 :") };
pub static mut un_803FB71C: [i8; 11] = unsafe { cstr::<11>(b"Scale__4 :") };
pub static mut un_803FB728: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FB6EC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FB6F8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x98[0]) as *mut _ as *mut c_void, x14: 0.1, x18: 5.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FB704) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x98[1]) as *mut _ as *mut c_void, x14: 0.1, x18: 5.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FB710) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x98[2]) as *mut _ as *mut c_void, x14: 0.1, x18: 5.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FB71C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x98[3]) as *mut _ as *mut c_void, x14: 0.1, x18: 5.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FB7E8: [i8; 9] = unsafe { cstr::<9>(b"Gm_Human") };
pub static mut un_803FB7F4: [i8; 9] = unsafe { cstr::<9>(b"Gm_Cpu  ") };
pub static mut un_803FB800: [i8; 9] = unsafe { cstr::<9>(b"Gm_Demo ") };
pub static mut un_803FB80C: [i8; 9] = unsafe { cstr::<9>(b"Gm_None ") };
pub static mut un_803FB818: [*mut i8; 4] = unsafe { [(&raw mut un_803FB7E8) as *mut _ as *mut i8, (&raw mut un_803FB7F4) as *mut _ as *mut i8, (&raw mut un_803FB800) as *mut _ as *mut i8, (&raw mut un_803FB80C) as *mut _ as *mut i8] };
pub static mut un_803FB828: [i8; 11] = unsafe { cstr::<11>(b"< P Kind >") };
pub static mut un_803FB834: [i8; 11] = unsafe { cstr::<11>(b"PKind__1 :") };
pub static mut un_803FB840: [i8; 11] = unsafe { cstr::<11>(b"PKind__2 :") };
pub static mut un_803FB84C: [i8; 11] = unsafe { cstr::<11>(b"PKind__3 :") };
pub static mut un_803FB858: [i8; 11] = unsafe { cstr::<11>(b"PKind__4 :") };
pub static mut un_803FB864: [i8; 12] = unsafe { cstr::<12>(b"All PKind :") };
pub static mut un_803FB870: [un_80304138_objalloc_t_x8; 7] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FB828) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB834) as *mut _ as *mut i8, xC: (&raw mut un_803FB818) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x24[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB840) as *mut _ as *mut i8, xC: (&raw mut un_803FB818) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x24[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB84C) as *mut _ as *mut i8, xC: (&raw mut un_803FB818) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x24[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FB858) as *mut _ as *mut i8, xC: (&raw mut un_803FB818) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x24[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80300790), x8: (&raw mut un_803FB864) as *mut _ as *mut i8, xC: (&raw mut un_803FB818) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x34) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FB950: [i8; 10] = unsafe { cstr::<10>(b"< Color >") };
pub static mut un_803FB95C: [i8; 10] = unsafe { cstr::<10>(b"Color_1 :") };
pub static mut un_803FB968: [i8; 10] = unsafe { cstr::<10>(b"Color_2 :") };
pub static mut un_803FB974: [i8; 10] = unsafe { cstr::<10>(b"Color_3 :") };
pub static mut un_803FB980: [i8; 10] = unsafe { cstr::<10>(b"Color_4 :") };
pub static mut un_803FB98C: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FB950) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FB95C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x38[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FB968) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x38[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FB974) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x38[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FB980) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x38[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FBA4C: [i8; 14] = unsafe { cstr::<14>(b"< Sub Color >") };
pub static mut un_803FBA5C: [i8; 13] = unsafe { cstr::<13>(b"SubColor_1 :") };
pub static mut un_803FBA6C: [i8; 13] = unsafe { cstr::<13>(b"SubColor_2 :") };
pub static mut un_803FBA7C: [i8; 13] = unsafe { cstr::<13>(b"SubColor_3 :") };
pub static mut un_803FBA8C: [i8; 13] = unsafe { cstr::<13>(b"SubColor_4 :") };
pub static mut un_803FBA9C: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FBA4C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBA5C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x48[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBA6C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x48[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBA7C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x48[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBA8C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x48[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FBB5C: [i8; 11] = unsafe { cstr::<11>(b"< Damage >") };
pub static mut un_803FBB68: [i8; 11] = unsafe { cstr::<11>(b"Damage_1 :") };
pub static mut un_803FBB74: [i8; 11] = unsafe { cstr::<11>(b"Damage_2 :") };
pub static mut un_803FBB80: [i8; 11] = unsafe { cstr::<11>(b"Damage_3 :") };
pub static mut un_803FBB8C: [i8; 11] = unsafe { cstr::<11>(b"Damage_4 :") };
pub static mut un_803FBB98: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FBB5C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBB68) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x68[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 999.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBB74) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x68[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 999.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBB80) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x68[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 999.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FBB8C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x68[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 999.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FBC58: [i8; 18] = unsafe { cstr::<18>(b"< Offence Ratio >") };
pub static mut un_803FBC6C: [i8; 13] = unsafe { cstr::<13>(b"OffRatio_1 :") };
pub static mut un_803FBC7C: [i8; 13] = unsafe { cstr::<13>(b"OffRatio_2 :") };
pub static mut un_803FBC8C: [i8; 13] = unsafe { cstr::<13>(b"OffRatio_3 :") };
pub static mut un_803FBC9C: [i8; 13] = unsafe { cstr::<13>(b"OffRatio_4 :") };
pub static mut un_803FBCAC: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FBC58) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBC6C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x78[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBC7C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x78[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBC8C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x78[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBC9C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x78[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FBD6C: [i8; 18] = unsafe { cstr::<18>(b"< Defence Ratio >") };
pub static mut un_803FBD80: [i8; 13] = unsafe { cstr::<13>(b"DefRatio_1 :") };
pub static mut un_803FBD90: [i8; 13] = unsafe { cstr::<13>(b"DefRatio_2 :") };
pub static mut un_803FBDA0: [i8; 13] = unsafe { cstr::<13>(b"DefRatio_3 :") };
pub static mut un_803FBDB0: [i8; 13] = unsafe { cstr::<13>(b"DefRatio_4 :") };
pub static mut un_803FBDC0: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FBD6C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBD80) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x88[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBD90) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x88[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBDA0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x88[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 8, x4: None, x8: (&raw mut un_803FBDB0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x88[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FBE80: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Stay  ") };
pub static mut un_803FBE8C: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Walk  ") };
pub static mut un_803FBE98: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Escape") };
pub static mut un_803FBEA4: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Jump  ") };
pub static mut un_803FBEB0: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Normal") };
pub static mut un_803FBEBC: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Manual") };
pub static mut un_803FBEC8: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Nana  ") };
pub static mut un_803FBED4: [i8; 15] = unsafe { cstr::<15>(b"CpTp_Defensive") };
pub static mut un_803FBEE4: [i8; 14] = unsafe { cstr::<14>(b"CpTp_Struggle") };
pub static mut un_803FBEF4: [i8; 12] = unsafe { cstr::<12>(b"CpTp_Freak ") };
pub static mut un_803FBF00: [i8; 15] = unsafe { cstr::<15>(b"CpTp_Cooperate") };
pub static mut un_803FBF10: [i8; 14] = unsafe { cstr::<14>(b"CpTp_SpLwLink") };
pub static mut un_803FBF20: [i8; 15] = unsafe { cstr::<15>(b"CpTp_SpLwSamus") };
pub static mut un_803FBF30: [i8; 14] = unsafe { cstr::<14>(b"CpTp_OnlyItem") };
pub static mut un_803FBF40: [i8; 13] = unsafe { cstr::<13>(b"CpTp_EvZelda") };
pub static mut un_803FBF50: [i8; 11] = unsafe { cstr::<11>(b"CpTp_NoAct") };
pub static mut un_803FBF5C: [i8; 9] = unsafe { cstr::<9>(b"CpTp_Air") };
pub static mut un_803FBF68: [i8; 10] = unsafe { cstr::<10>(b"CpTp_Item") };
pub static mut un_803FBF74: [*mut i8; 18] = unsafe { [(&raw mut un_803FBE80) as *mut _ as *mut i8, (&raw mut un_803FBE8C) as *mut _ as *mut i8, (&raw mut un_803FBE98) as *mut _ as *mut i8, (&raw mut un_803FBEA4) as *mut _ as *mut i8, (&raw mut un_803FBEB0) as *mut _ as *mut i8, (&raw mut un_803FBEBC) as *mut _ as *mut i8, (&raw mut un_803FBEC8) as *mut _ as *mut i8, (&raw mut un_803FBED4) as *mut _ as *mut i8, (&raw mut un_803FBEE4) as *mut _ as *mut i8, (&raw mut un_803FBEF4) as *mut _ as *mut i8, (&raw mut un_803FBF00) as *mut _ as *mut i8, (&raw mut un_803FBF10) as *mut _ as *mut i8, (&raw mut un_803FBF20) as *mut _ as *mut i8, (&raw mut un_803FBF30) as *mut _ as *mut i8, (&raw mut un_803FBF40) as *mut _ as *mut i8, (&raw mut un_803FBF50) as *mut _ as *mut i8, (&raw mut un_803FBF5C) as *mut _ as *mut i8, (&raw mut un_803FBF68) as *mut _ as *mut i8] };
pub static mut un_803FBFBC: [i8; 13] = unsafe { cstr::<13>(b"< Cpu Type >") };
pub static mut un_803FBFCC: [i8; 11] = unsafe { cstr::<11>(b"Type__C1 :") };
pub static mut un_803FBFD8: [i8; 11] = unsafe { cstr::<11>(b"Type__C2 :") };
pub static mut un_803FBFE4: [i8; 11] = unsafe { cstr::<11>(b"Type__C3 :") };
pub static mut un_803FBFF0: [i8; 11] = unsafe { cstr::<11>(b"Type__C4 :") };
pub static mut un_803FBFFC: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FBFBC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FBFCC) as *mut _ as *mut i8, xC: (&raw mut un_803FBF74) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xA8[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 18.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FBFD8) as *mut _ as *mut i8, xC: (&raw mut un_803FBF74) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xA8[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 18.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FBFE4) as *mut _ as *mut i8, xC: (&raw mut un_803FBF74) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xA8[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 18.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FBFF0) as *mut _ as *mut i8, xC: (&raw mut un_803FBF74) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.xA8[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 18.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC0BC: [i8; 14] = unsafe { cstr::<14>(b"< Cpu Level >") };
pub static mut un_803FC0CC: [i8; 11] = unsafe { cstr::<11>(b"Level_C1 :") };
pub static mut un_803FC0D8: [i8; 11] = unsafe { cstr::<11>(b"Level_C2 :") };
pub static mut un_803FC0E4: [i8; 11] = unsafe { cstr::<11>(b"Level_C3 :") };
pub static mut un_803FC0F0: [i8; 11] = unsafe { cstr::<11>(b"Level_C4 :") };
pub static mut un_803FC0FC: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC0BC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC0CC) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xB8[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC0D8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xB8[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC0E4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xB8[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC0F0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xB8[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC1BC: [i8; 9] = unsafe { cstr::<9>(b"Red-Team") };
pub static mut un_803FC1C8: [i8; 11] = unsafe { cstr::<11>(b"Green-Team") };
pub static mut un_803FC1D4: [i8; 10] = unsafe { cstr::<10>(b"Blue-Team") };
pub static mut un_803FC1E0: [*mut i8; 3] = unsafe { [(&raw mut un_803FC1BC) as *mut _ as *mut i8, (&raw mut un_803FC1C8) as *mut _ as *mut i8, (&raw mut un_803FC1D4) as *mut _ as *mut i8] };
pub static mut un_803FC1EC: [i8; 16] = unsafe { cstr::<16>(b"< Team Select >") };
pub static mut un_803FC1FC: [i8; 9] = unsafe { cstr::<9>(b"Team_1 :") };
pub static mut un_803FC208: [i8; 9] = unsafe { cstr::<9>(b"Team_2 :") };
pub static mut un_803FC214: [i8; 9] = unsafe { cstr::<9>(b"Team_3 :") };
pub static mut un_803FC220: [i8; 9] = unsafe { cstr::<9>(b"Team_4 :") };
pub static mut un_803FC22C: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC1EC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FC1FC) as *mut _ as *mut i8, xC: (&raw mut un_803FC1E0) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x58[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FC208) as *mut _ as *mut i8, xC: (&raw mut un_803FC1E0) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x58[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FC214) as *mut _ as *mut i8, xC: (&raw mut un_803FC1E0) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x58[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FC220) as *mut _ as *mut i8, xC: (&raw mut un_803FC1E0) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x58[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC2EC: [u32; 77] = unsafe { [0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000001, 0x00000003, 0x00000002, 0x00000002, 0x00000005, 0x00000005, 0x00000004, 0x00000006, 0x00000006, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000008, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000006, 0x00000006, 0x00000005, 0x00000007, 0x00000007, 0x00000007, 0x00000007, 0x00000007, 0x00000007, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005, 0x00000005] };
pub static mut un_803FC420: [i8; 12] = unsafe { cstr::<12>(b"<TEST MODE>") };
pub static mut un_803FC42C: [i8; 18] = unsafe { cstr::<18>(b"MODE : Hanyu    >") };
pub static mut un_803FC440: [i8; 18] = unsafe { cstr::<18>(b"MODE : Otoguro  >") };
pub static mut un_803FC454: [i8; 18] = unsafe { cstr::<18>(b"MODE : Taniguti >") };
pub static mut un_803FC468: [i8; 18] = unsafe { cstr::<18>(b"MODE : Nagasima >") };
pub static mut un_803FC47C: [i8; 18] = unsafe { cstr::<18>(b"MODE : Kim      >") };
pub static mut un_803FC490: [i8; 18] = unsafe { cstr::<18>(b"MODE : Sakoda   >") };
pub static mut un_803FC4A4: [i8; 18] = unsafe { cstr::<18>(b"MODE : Sugano   >") };
pub static mut un_803FC4B8: [i8; 18] = unsafe { cstr::<18>(b"MODE : Yoshiki  >") };
pub static mut un_803FC4CC: [un_80304138_objalloc_t_x8; 10] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC420) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803007C8), x8: (&raw mut un_803FC42C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803007FC), x8: (&raw mut un_803FC440) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300830), x8: (&raw mut un_803FC454) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300864), x8: (&raw mut un_803FC468) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300898), x8: (&raw mut un_803FC47C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803008CC), x8: (&raw mut un_803FC490) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300900), x8: (&raw mut un_803FC4A4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300934), x8: (&raw mut un_803FC4B8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC60C: [i8; 15] = unsafe { cstr::<15>(b"< Hanyu Test >") };
pub static mut un_803FC61C: [i8; 15] = unsafe { cstr::<15>(b"Select Char  >") };
pub static mut un_803FC62C: [i8; 15] = unsafe { cstr::<15>(b"Select Stage >") };
pub static mut un_803FC63C: [un_80304138_objalloc_t_x8; 4] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC60C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300968), x8: (&raw mut un_803FC61C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803009A4), x8: (&raw mut un_803FC62C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC6BC: [i8; 20] = unsafe { cstr::<20>(b"< Melee Count Set >") };
pub static mut un_803FC6D0: [i8; 11] = unsafe { cstr::<11>(b"Time     >") };
pub static mut un_803FC6DC: [i8; 11] = unsafe { cstr::<11>(b"Stock    >") };
pub static mut un_803FC6E8: [i8; 11] = unsafe { cstr::<11>(b"Coin     >") };
pub static mut un_803FC6F4: [i8; 11] = unsafe { cstr::<11>(b"Decision >") };
pub static mut un_803FC700: [i8; 11] = unsafe { cstr::<11>(b"Default  >") };
pub static mut un_803FC70C: [un_80304138_objalloc_t_x8; 7] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC6BC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC6D0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DE0) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC6DC) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DE4) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC6E8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DE8) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC6F4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DEC) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC700) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DF0) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1000.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC7EC: [i8; 21] = unsafe { cstr::<21>(b"< Player Count Set >") };
pub static mut un_803FC804: [i8; 10] = unsafe { cstr::<10>(b"Chara   :") };
pub static mut un_803FC810: [i8; 10] = unsafe { cstr::<10>(b"Count   :") };
pub static mut un_803FC81C: [i8; 10] = unsafe { cstr::<10>(b"Default :") };
pub static mut un_803FC828: [un_80304138_objalloc_t_x8; 5] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC7EC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_803009E0), x8: (&raw mut un_803FC804) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_804D6DC8) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80300A88), x8: (&raw mut un_803FC810) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DCC) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC81C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DD4) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 10.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC8C8: [un_80304138_objalloc_t_x8; 5] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC7EC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_803009E0), x8: (&raw mut un_803FC804) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_804D6DC8) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 8, x4: Some(un_80300A88), x8: (&raw mut un_803FC810) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DD0) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 0.1 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FC81C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DD4) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+06, x1C: 10.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FC968: [i8; 17] = unsafe { cstr::<17>(b"< Otoguro Test >") };
pub static mut un_803FC97C: [i8; 16] = unsafe { cstr::<16>(b"Set Gold Coin :") };
pub static mut un_803FC98C: [i8; 11] = unsafe { cstr::<11>(b"Figure Max") };
pub static mut un_803FC998: [i8; 15] = unsafe { cstr::<15>(b"Samus Mask Get") };
pub static mut un_803FC9A8: [i8; 18] = unsafe { cstr::<18>(b"Mario & Yoshi Get") };
pub static mut un_803FC9BC: [i8; 18] = unsafe { cstr::<18>(b"Set Melee Count >") };
pub static mut un_803FC9D0: [i8; 21] = unsafe { cstr::<21>(b"Set HomeRun Record >") };
pub static mut un_803FC9E8: [i8; 19] = unsafe { cstr::<19>(b"Set Target Clear >") };
pub static mut un_803FC9FC: [i8; 17] = unsafe { cstr::<17>(b"Set Beat Total :") };
pub static mut un_803FCA10: [i8; 14] = unsafe { cstr::<14>(b"Go to  Meikyu") };
pub static mut un_803FCA20: [i8; 14] = unsafe { cstr::<14>(b"Go to  Kinoko") };
pub static mut un_803FCA30: [i8; 13] = unsafe { cstr::<13>(b"Melee  Start") };
pub static mut un_803FCA40: [un_80304138_objalloc_t_x8; 13] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FC968) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80300C80), x8: (&raw mut un_803FC97C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DDC) as *mut _ as *mut c_void, x14: 0.0, x18: 999.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300BBC), x8: (&raw mut un_803FC98C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300BF8), x8: (&raw mut un_803FC998) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300C3C), x8: (&raw mut un_803FC9A8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300D78), x8: (&raw mut un_803FC9BC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300E74), x8: (&raw mut un_803FC9D0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300F3C), x8: (&raw mut un_803FC9E8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80300F98), x8: (&raw mut un_803FC9FC) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DF4) as *mut _ as *mut c_void, x14: 0.0, x18: 1e+08, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300AF4), x8: (&raw mut un_803FCA10) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300B58), x8: (&raw mut un_803FCA20) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300AB8), x8: (&raw mut un_803FCA30) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FCBE0: [i8; 18] = unsafe { cstr::<18>(b"< Taniguti Test >") };
pub static mut un_803FCBF4: [i8; 17] = unsafe { cstr::<17>(b"Melee    Start >") };
pub static mut un_803FCC08: [i8; 14] = unsafe { cstr::<14>(b"Reg:Normal  >") };
pub static mut un_803FCC18: [i8; 14] = unsafe { cstr::<14>(b"Reg:Easy    >") };
pub static mut un_803FCC28: [i8; 14] = unsafe { cstr::<14>(b"Reg:Allstar >") };
pub static mut un_803FCC38: [un_80304138_objalloc_t_x8; 6] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FCBE0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80300FEC), x8: (&raw mut un_803FCBF4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301028), x8: (&raw mut un_803FCC08) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D5908) as *mut _ as *mut c_void, x14: 1.0, x18: 12.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301074), x8: (&raw mut un_803FCC18) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D590C) as *mut _ as *mut c_void, x14: 1.0, x18: 11.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_803010C0), x8: (&raw mut un_803FCC28) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D5910) as *mut _ as *mut c_void, x14: 1.0, x18: 13.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FCCF8: [i8; 10] = unsafe { cstr::<10>(b"ADVENTURE") };
pub static mut un_803FCD04: [i8; 9] = unsafe { cstr::<9>(b"ALL-STAR") };
pub static mut un_803FCD10: [*mut i8; 3] = unsafe { [(&raw mut un_804D5914) as *mut _ as *mut i8, (&raw mut un_803FCCF8) as *mut _ as *mut i8, (&raw mut un_803FCD04) as *mut _ as *mut i8] };
pub static mut un_803FCD1C: [i8; 24] = unsafe { cstr::<24>(b"< Regular Ending Test >") };
pub static mut un_803FCD34: [un_80304138_objalloc_t_x8; 5] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FCD1C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_804D591C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_804D6DF8) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_804D5924) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6DFC) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_804D592C) as *mut _ as *mut i8, xC: (&raw mut un_803FCD10) as *mut _ as *mut *mut i8, x10: (&raw mut un_804D6E00) as *mut _ as *mut c_void, x14: 0.0, x18: 3.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FCDD4: [i8; 18] = unsafe { cstr::<18>(b"< Nagasima Test >") };
pub static mut un_803FCDE8: [i8; 19] = unsafe { cstr::<19>(b"Visual-Scene Start") };
pub static mut un_803FCDFC: [i8; 14] = unsafe { cstr::<14>(b"Opening Start") };
pub static mut un_803FCE0C: [i8; 14] = unsafe { cstr::<14>(b"Omake15 Start") };
pub static mut un_803FCE1C: [i8; 21] = unsafe { cstr::<21>(b"RegularEnding Test >") };
pub static mut un_803FCE34: [i8; 21] = unsafe { cstr::<21>(b"RegularEnding Real >") };
pub static mut un_803FCE4C: [un_80304138_objalloc_t_x8; 7] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FCDD4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803013B0), x8: (&raw mut un_803FCDE8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301328), x8: (&raw mut un_803FCDFC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_8030136C), x8: (&raw mut un_803FCE0C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803012D4), x8: (&raw mut un_803FCE1C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301280), x8: (&raw mut un_803FCE34) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FCF2C: [i8; 10] = unsafe { cstr::<10>(b"NoChara 0") };
pub static mut un_803FCF38: [i8; 10] = unsafe { cstr::<10>(b"GameWatch") };
pub static mut un_803FCF44: [i8; 9] = unsafe { cstr::<9>(b"Dr Mario") };
pub static mut un_803FCF50: [i8; 11] = unsafe { cstr::<11>(b"NoChara 12") };
pub static mut un_803FCF5C: [*mut i8; 13] = unsafe { [(&raw mut un_803FCF2C) as *mut _ as *mut i8, (&raw mut un_803FCF38) as *mut _ as *mut i8, (&raw mut un_804D5934) as *mut _ as *mut i8, (&raw mut un_804D593C) as *mut _ as *mut i8, (&raw mut un_804D5944) as *mut _ as *mut i8, (&raw mut un_804D594C) as *mut _ as *mut i8, (&raw mut un_804D5954) as *mut _ as *mut i8, (&raw mut un_804D595C) as *mut _ as *mut i8, (&raw mut un_803FCF44) as *mut _ as *mut i8, (&raw mut un_804D5964) as *mut _ as *mut i8, (&raw mut un_804D5968) as *mut _ as *mut i8, (&raw mut un_804D5970) as *mut _ as *mut i8, (&raw mut un_803FCF50) as *mut _ as *mut i8] };
pub static mut un_803FCF90: [i8; 13] = unsafe { cstr::<13>(b"< Kim Test >") };
pub static mut un_803FCFA0: [i8; 12] = unsafe { cstr::<12>(b"Result Test") };
pub static mut un_803FCFAC: [i8; 20] = unsafe { cstr::<20>(b"Card Check - scene>") };
pub static mut un_803FCFC0: [i8; 20] = unsafe { cstr::<20>(b"Card Check - mode >") };
pub static mut un_803FCFD4: [i8; 10] = unsafe { cstr::<10>(b"EasyEnd :") };
pub static mut un_803FCFE0: [i8; 12] = unsafe { cstr::<12>(b"GoHoubi Id:") };
pub static mut un_803FCFEC: [i8; 17] = unsafe { cstr::<17>(b"GoHoubi Figure :") };
pub static mut un_803FD000: [i8; 12] = unsafe { cstr::<12>(b"Intro Easy ") };
pub static mut un_803FD00C: [i8; 16] = unsafe { cstr::<16>(b"AllStar Enemy :") };
pub static mut un_803FD01C: [i8; 21] = unsafe { cstr::<21>(b"Test Movie Opening >") };
pub static mut un_803FD034: [i8; 13] = unsafe { cstr::<13>(b"KUMITE SUB :") };
pub static mut un_803FD044: [i8; 18] = unsafe { cstr::<18>(b"Progressive Test ") };
pub static mut un_803FD058: [i8; 11] = unsafe { cstr::<11>(b"Approach :") };
pub static mut un_803FD064: [un_80304138_objalloc_t_x8; 14] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FCF90) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301420), x8: (&raw mut un_803FCFA0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301490), x8: (&raw mut un_803FCFAC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301454), x8: (&raw mut un_803FCFC0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80301544), x8: (&raw mut un_803FCFD4) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x134) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301508), x8: (&raw mut un_803FCFE0) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x12C) as *mut _ as *mut c_void, x14: 0.0, x18: 65.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301508), x8: (&raw mut un_803FCFEC) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x130) as *mut _ as *mut c_void, x14: 0.0, x18: 299.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803013EC), x8: (&raw mut un_803FD000) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_803014CC), x8: (&raw mut un_803FD00C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x138) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301580), x8: (&raw mut un_803FD01C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD034) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x13C) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803015BC), x8: (&raw mut un_803FD044) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_803015F8), x8: (&raw mut un_803FD058) as *mut _ as *mut i8, xC: (&raw mut un_803FCF5C) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x140) as *mut _ as *mut c_void, x14: 0.0, x18: 13.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FD224: [i8; 10] = unsafe { cstr::<10>(b"<Format>\n") };
pub static mut un_803FD230: [i8; 11] = unsafe { cstr::<11>(b"result %d\n") };
pub static mut un_803FD23C: [i8; 14] = unsafe { cstr::<14>(b"<Accessable>\n") };
pub static mut un_803FD24C: [i8; 10] = unsafe { cstr::<10>(b"<Create>\n") };
pub static mut un_803FD258: [i8; 10] = unsafe { cstr::<10>(b"<Delete>\n") };
pub static mut un_803FD264: [i8; 14] = unsafe { cstr::<14>(b"<SnapMount0>\n") };
pub static mut un_803FD274: HSD_ImageDesc = unsafe { HSD_ImageDesc { image_ptr: null_mut(), width: 640, height: 480, format: GX_TF_RGB565, mipmap: 0, minLOD: 0.0, maxLOD: 0.0 } };
pub static mut un_803FD28C: [i8; 13] = unsafe { cstr::<13>(b"<SnapLoad0>\n") };
pub static mut un_803FD29C: [i8; 15] = unsafe { cstr::<15>(b"<SnapDelete0>\n") };
pub static mut un_803FD2AC: [i8; 13] = unsafe { cstr::<13>(b"<SnapSwap0>\n") };
pub static mut un_803FD2BC: [i8; 17] = unsafe { cstr::<17>(b"< Yoshiki Test >") };
pub static mut un_803FD2D0: [i8; 11] = unsafe { cstr::<11>(b"Accessable") };
pub static mut un_803FD2DC: [i8; 11] = unsafe { cstr::<11>(b"SnapMount0") };
pub static mut un_803FD2E8: [i8; 11] = unsafe { cstr::<11>(b"SnapLoad0 ") };
pub static mut un_803FD2F4: [i8; 13] = unsafe { cstr::<13>(b"SnapDelete0 ") };
pub static mut un_803FD304: [i8; 11] = unsafe { cstr::<11>(b"SnapSwap0 ") };
pub static mut un_803FD310: [un_80304138_objalloc_t_x8; 13] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FD2BC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301634), x8: (&raw mut un_804D5998) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803016D8), x8: (&raw mut un_804D59A0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301734), x8: (&raw mut un_803FD2D0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_8030178C), x8: (&raw mut un_804D59A8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301800), x8: (&raw mut un_804D59B0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301840), x8: (&raw mut un_804D59B8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_803018BC), x8: (&raw mut un_804D59C0) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_8030191C), x8: (&raw mut un_803FD2DC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301964), x8: (&raw mut un_803FD2E8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6E0C) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301A64), x8: (&raw mut un_803FD2F4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6E10) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301AD4), x8: (&raw mut un_803FD304) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_804D6E14) as *mut _ as *mut c_void, x14: 0.0, x18: 127.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FD4B0: [*mut i8; 5] = unsafe { [(&raw mut un_804D59C8) as *mut _ as *mut i8, (&raw mut un_804D59D0) as *mut _ as *mut i8, (&raw mut un_804D59D8) as *mut _ as *mut i8, (&raw mut un_804D59E0) as *mut _ as *mut i8, (&raw mut un_804D59E8) as *mut _ as *mut i8] };
pub static mut un_803FD4C4: [i8; 11] = unsafe { cstr::<11>(b"Tukisusume") };
pub static mut un_803FD4D0: [i8; 10] = unsafe { cstr::<10>(b"FigureGet") };
pub static mut un_803FD4DC: [*mut i8; 4] = unsafe { [(&raw mut un_804D59C8) as *mut _ as *mut i8, (&raw mut un_804D59F0) as *mut _ as *mut i8, (&raw mut un_803FD4C4) as *mut _ as *mut i8, (&raw mut un_803FD4D0) as *mut _ as *mut i8] };
pub static mut un_803FD4EC: [i8; 18] = unsafe { cstr::<18>(b"<INTRO EASY TEST>") };
pub static mut un_803FD500: [i8; 12] = unsafe { cstr::<12>(b"StageNum : ") };
pub static mut un_803FD50C: [i8; 13] = unsafe { cstr::<13>(b"Stage Flag :") };
pub static mut un_803FD51C: [i8; 13] = unsafe { cstr::<13>(b"Stage Type :") };
pub static mut un_803FD52C: [i8; 16] = unsafe { cstr::<16>(b"DISP LEFT NUM :") };
pub static mut un_803FD53C: [i8; 16] = unsafe { cstr::<16>(b"DISP RIGHTNUM :") };
pub static mut un_803FD54C: [i8; 14] = unsafe { cstr::<14>(b"METAL LEFT  :") };
pub static mut un_803FD55C: [i8; 14] = unsafe { cstr::<14>(b"METAL RIGHT :") };
pub static mut un_803FD56C: [i8; 14] = unsafe { cstr::<14>(b"LEFT PLAYER :") };
pub static mut un_803FD57C: [i8; 14] = unsafe { cstr::<14>(b"RIGHT PLAYER:") };
pub static mut un_803FD58C: [un_80304138_objalloc_t_x8; 16] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FD4EC) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD500) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x100) as *mut _ as *mut c_void, x14: 1.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD50C) as *mut _ as *mut i8, xC: (&raw mut un_803FD4B0) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x104) as *mut _ as *mut c_void, x14: 0.0, x18: 5.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD51C) as *mut _ as *mut i8, xC: (&raw mut un_803FD4DC) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x108) as *mut _ as *mut c_void, x14: 0.0, x18: 4.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD52C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x10C) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD53C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x11C) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD54C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xF8) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD55C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.xFC) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD56C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x110) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD56C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x114) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD56C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x118) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD57C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x120) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD57C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x124) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD57C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x128) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301B48), x8: (&raw mut un_804D59F8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FD78C: [i8; 14] = unsafe { cstr::<14>(b"<RESULT TEST>") };
pub static mut un_803FD79C: [i8; 10] = unsafe { cstr::<10>(b"1P KIND :") };
pub static mut un_803FD7A8: [i8; 10] = unsafe { cstr::<10>(b"2P KIND :") };
pub static mut un_803FD7B4: [i8; 10] = unsafe { cstr::<10>(b"3P KIND :") };
pub static mut un_803FD7C0: [i8; 10] = unsafe { cstr::<10>(b"4P KIND :") };
pub static mut un_803FD7CC: [i8; 13] = unsafe { cstr::<13>(b"All Player :") };
pub static mut un_803FD7DC: [i8; 10] = unsafe { cstr::<10>(b"1P RANK :") };
pub static mut un_803FD7E8: [i8; 10] = unsafe { cstr::<10>(b"2P RANK :") };
pub static mut un_803FD7F4: [i8; 10] = unsafe { cstr::<10>(b"3P RANK :") };
pub static mut un_803FD800: [i8; 10] = unsafe { cstr::<10>(b"4P RANK :") };
pub static mut un_803FD80C: [i8; 10] = unsafe { cstr::<10>(b"1P WIN  :") };
pub static mut un_803FD818: [i8; 10] = unsafe { cstr::<10>(b"2P WIN  :") };
pub static mut un_803FD824: [i8; 10] = unsafe { cstr::<10>(b"3P WIN  :") };
pub static mut un_803FD830: [i8; 10] = unsafe { cstr::<10>(b"4P WIN  :") };
pub static mut un_803FD83C: [i8; 12] = unsafe { cstr::<12>(b"ALL COLOR :") };
pub static mut un_803FD848: [i8; 12] = unsafe { cstr::<12>(b"STEP ANIM :") };
pub static mut un_803FD854: [i8; 15] = unsafe { cstr::<15>(b"PANEL HIDDEN :") };
pub static mut un_803FD864: [un_80304138_objalloc_t_x8; 19] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FD78C) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD79C) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x144[0]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD7A8) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x144[1]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD7B4) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x144[2]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: None, x8: (&raw mut un_803FD7C0) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x144[3]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 2, x4: Some(un_80301CE0), x8: (&raw mut un_803FD7CC) as *mut _ as *mut i8, xC: (&raw mut un_803FB538) as *mut _ as *mut *mut i8, x10: (&raw mut un_803FA258.x144[4]) as *mut _ as *mut c_void, x14: 0.0, x18: 34.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD7DC) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[5]) as *mut _ as *mut c_void, x14: 1.0, x18: 4.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD7E8) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[6]) as *mut _ as *mut c_void, x14: 1.0, x18: 4.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD7F4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[7]) as *mut _ as *mut c_void, x14: 1.0, x18: 4.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD800) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[8]) as *mut _ as *mut c_void, x14: 1.0, x18: 4.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD80C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[9]) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD818) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[10]) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD824) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[11]) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD830) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[12]) as *mut _ as *mut c_void, x14: 1.0, x18: 3.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD83C) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[15]) as *mut _ as *mut c_void, x14: 0.0, x18: 9.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD848) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[13]) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: None, x8: (&raw mut un_803FD854) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x144[14]) as *mut _ as *mut c_void, x14: 0.0, x18: 1.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301C80), x8: (&raw mut un_804D59F8) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FDAC4: [i8; 16] = unsafe { cstr::<16>(b"< Sakoda Test >") };
pub static mut un_803FDAD4: [i8; 14] = unsafe { cstr::<14>(b"Event Stage :") };
pub static mut un_803FDAE4: [i8; 18] = unsafe { cstr::<18>(b"FixCamera Start >") };
pub static mut un_803FDAF8: [un_80304138_objalloc_t_x8; 4] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FDAC4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 3, x4: Some(un_80301D7C), x8: (&raw mut un_803FDAD4) as *mut _ as *mut i8, xC: null_mut(), x10: (&raw mut un_803FA258.x188) as *mut _ as *mut c_void, x14: 1.0, x18: 51.0, x1C: 1.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301D40), x8: (&raw mut un_803FDAE4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
pub static mut un_803FDB78: [i8; 16] = unsafe { cstr::<16>(b"< Sugano Test >") };
pub static mut un_803FDB88: [i8; 18] = unsafe { cstr::<18>(b"STAFFROLL START >") };
pub static mut un_803FDB9C: [un_80304138_objalloc_t_x8; 4] = unsafe { [un_80304138_objalloc_t_x8 { x0: 0, x4: None, x8: (&raw mut un_803FDB78) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301DCC), x8: (&raw mut un_803FCBF4) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 1, x4: Some(un_80301E08), x8: (&raw mut un_803FDB88) as *mut _ as *mut i8, xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }, un_80304138_objalloc_t_x8 { x0: 9, x4: None, x8: null_mut(), xC: null_mut(), x10: null_mut(), x14: 0.0, x18: 0.0, x1C: 0.0 }] };
// END GENERATED DATA

#[inline]
pub(crate) unsafe fn sfx_back() {
    ext::lbAudioAx_80024030(0);
}

#[inline]
pub(crate) unsafe fn sfx_forward() {
    ext::lbAudioAx_80024030(1);
}

/// `((int**) un_804D6DA8)[k]`: the loaded symbol table is read as an array of pointers.
#[inline]
unsafe fn sym_ptr(k: usize) -> *mut i32 {
    *(un_804D6DA8 as *mut *mut i32).add(k)
}

/// `u8* gmMainLib_8045A6C0 + xF0`: the save-data copy the four `xF4..xF7` bytes are read from / written to.
#[inline]
pub(crate) unsafe fn save_base() -> *mut u8 {
    (&raw mut ext::gmMainLib_8045A6C0 as *mut u8).add(un_803FA258.xF0 as usize)
}

/// A callback that opens the sub-menu `$tbl` with the back / start handler `$cb`.
macro_rules! open_menu {
    ($(fn $name:ident($tbl:ident, $cb:ident);)*) => {$(
        pub unsafe extern "C" fn $name(arg0: soundtest_callback_arg0) -> i32 {
            un_802FFD94(arg0 as i32, &raw mut $tbl as *mut c_void, Some($cb));
            0
        }
    )*};
}

/// A callback that on confirm plays the cue, selects game-mode state `$n` and leaves the scene.
macro_rules! to_state {
    ($(fn $name:ident($n:expr);)*) => {$(
        pub unsafe extern "C" fn $name(arg0: soundtest_callback_arg0) -> i32 {
            if arg0 == 1 {
                sfx_forward();
                ext::gm_SetNextGameModeStateId($n);
                ext::gm_801A4B60();
            }
            0
        }
    )*};
}

/// A callback that on confirm plays the cue, changes the game mode after the scene and leaves it.
macro_rules! to_mode {
    ($(fn $name:ident($m:expr);)*) => {$(
        pub unsafe extern "C" fn $name(arg0: soundtest_callback_arg0) -> i32 {
            if arg0 == 1 {
                sfx_forward();
                ext::gm_ChangeGameModeAfterCurrentScene($m as i32);
                ext::gm_801A4B60();
            }
            0
        }
    )*};
}

/// `void un_802FF7DC(void)`: loads the sound test archive's symbols into the menu rows.
pub unsafe extern "C" fn un_802FF7DC() {
    melee_core::ct!("un_802FF7DC");
    let data = &raw mut un_803F9FA4;
    ext::lbArchive_LoadSymbols((*data).x160.as_ptr(), &raw mut un_804D6DA8 as *mut c_void, (*data).x16C.as_ptr());
    let syms = un_804D6DA8;
    (*data).entries[1].x18 = *syms.add(0) as f32;
    (*data).entries[1].xC = *syms.add(1) as isize as *mut *mut i8;
    (*data).entries[6].xC = *syms.add(2) as isize as *mut *mut i8;
    (*data).entries[7].xC = *syms.add(3) as isize as *mut *mut i8;
    (*data).entries[7].x18 = *syms.add(4) as f32;
    (*data).entries[8].xC = *syms.add(7) as isize as *mut *mut i8;
}

/// `bool un_802FF884(char*)`
pub unsafe extern "C" fn un_802FF884(_arg0: *mut i8) -> i32 {
    melee_core::ct!("un_802FF884");
    0
}

pub unsafe extern "C" fn un_802FF88C(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FF88C");
    un_804D5858 = 0x7F;
    un_804D6DB8 = 0;
    un_804D5854 = 0x7F;
    un_804D5850 = 0x7F;
    un_804D6DB4 = 0;
    un_804D6DB0 = 0;
    un_804D6DC0 = 0;
    un_804D585C = -1;
    un_804D6DBC = ext::lbAudioAx_80024BD0();
    ext::lbAudioAx_800236DC();
    ext::lbAudioAx_800245D4(0x7F);
    ext::lbAudioAx_800245F4(0x7F);
    ext::lbAudioAx_80024614(0x7F);
    if !ext::un_80302DF0().is_null() {
        ext::un_80302DF8(ext::un_80302DF0(), Some(fn_802FFCC8));
    }
    let x = ext::un_80302DF0();
    (*(*x).x4).scale_x = 16.0;
    (*(*x).x4).scale_y = 32.0;
    1
}

pub unsafe extern "C" fn un_802FF934(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FF934");
    ext::lbAudioAx_80024C08(un_804D6DBC);
    0
}

pub unsafe extern "C" fn un_802FF958(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FF958");
    let mut tmp = un_804D5854 as u8;
    ext::lbAudioAx_800245F4(tmp as i32);
    tmp = un_804D5858 as u8;
    ext::lbAudioAx_80024614(tmp as i32);
    tmp = un_804D5850 as u8;
    ext::lbAudioAx_800245D4(tmp as i32);
    0
}

pub unsafe extern "C" fn un_802FF99C(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FF99C");
    let tmp = un_804D6DAC as u8;
    ext::lbAudioAx_80024634(tmp as i32);
    un_802FFB58(arg0);
    0
}

pub unsafe extern "C" fn un_802FF9DC(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FF9DC");
    un_804D6DB4 = 0;
    let mut i: i32 = 0;
    while i < un_804D6DB0 {
        un_804D6DB4 = un_804D6DB4.wrapping_add(*sym_ptr(6).offset(i as isize));
        i += 1;
    }
    un_803F9FA4.entries[7].x14 = un_804D6DB4 as f32;
    let total = un_804D6DB4.wrapping_add(*sym_ptr(6).offset(un_804D6DB0 as isize));
    un_803F9FA4.entries[7].x18 = total as f32;
    0
}

pub unsafe extern "C" fn un_802FFB58(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FFB58");
    if arg0 == 1 {
        ext::lbAudioAx_80023694();
        ext::lbAudioAx_80023B24(*sym_ptr(5).offset(un_804D6DB4 as isize));
    } else if arg0 == 0 {
        ext::lbAudioAx_80023694();
    }
    0
}

pub unsafe extern "C" fn un_802FFBAC(arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FFBAC");
    if arg0 == 1 {
        if un_804D585C == un_804D6DB8 {
            let x = un_804D6DC0.wrapping_add(1) & 1;
            un_804D6DC0 = x;
            ext::lbAudioAx_80024E50(x);
        } else {
            ext::lbAudioAx_80023F28(un_804D6DB8);
            un_804D585C = un_804D6DB8;
            un_804D6DC0 = 0;
        }
    } else if arg0 == 0 {
        ext::lbAudioAx_800236DC();
        un_804D585C = -1;
        un_804D6DC0 = 0;
    }
    0
}

pub unsafe extern "C" fn un_802FFC30(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FFC30");
    ext::lbAudioAx_800236DC();
    ext::lbAudioAx_800245D4(0x7F);
    ext::lbAudioAx_800245F4(0x7F);
    ext::lbAudioAx_80024614(0x7F);
    1
}

pub unsafe extern "C" fn un_802FFC6C(update_scene: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("un_802FFC6C");
    if update_scene == 1 {
        ext::lbAudioAx_80023694();
        sfx_back();
        ext::lbAudioAx_800236DC();
        ext::lbAudioAx_800245D4(0x7F);
        ext::lbAudioAx_800245F4(0x7F);
        ext::lbAudioAx_80024614(0x7F);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_TITLE as i32);
        ext::gm_801A4B60();
    }
    1
}

pub unsafe extern "C" fn fn_802FFCC8(_arg0: soundtest_callback_arg0) -> i32 {
    melee_core::ct!("fn_802FFCC8");
    0
}

to_state! {
    fn un_803001DC(0x1);
    fn un_803002FC(0x2);
    fn un_80300AB8(0x4);
    fn un_80300FEC(0x4);
    fn un_80301490(0xD);
    fn un_803014CC(0x7);
    fn un_80301508(0x3);
    fn un_80301544(0x9);
    fn un_80301580(0xC);
    fn un_803015F8(0xA);
    fn un_80301DCC(0x4);
    fn un_80301E08(0xE);
}

to_mode! {
    fn un_80300968(GM_HANYU_CSS);
    fn un_803009A4(GM_HANYU_SSS);
    fn un_803013B0(GM_DEBUG_CUTSCENE);
    fn un_80301454(GM_BOOT);
    fn un_803015BC(GM_PROGRESSIVE_SCAN);
    fn un_80301D40(GM_CAMERA_VS);
}

/// `xF4..xF7 = save[xF0 + 0x1868..]`
#[inline]
pub(crate) unsafe fn load_f4(src: *mut u8) {
    un_803FA258.xF4 = *src.add(0x1868);
    un_803FA258.xF5 = *src.add(0x1869);
    un_803FA258.xF6 = *src.add(0x186A);
    un_803FA258.xF7 = *src.add(0x186B);
}

open_menu! {
    fn un_80300480(un_803FB168, fn_802FFE0C);
    fn un_803004B4(un_803FA8E8, fn_802FFE6C);
    fn un_803004E8(un_803FAA44, fn_802FFE6C);
    fn un_8030051C(un_803FB60C, fn_802FFE6C);
    fn un_80300550(un_803FB728, fn_802FFE6C);
    fn un_80300584(un_803FB870, fn_802FFE6C);
    fn un_803005B8(un_803FB98C, fn_802FFE6C);
    fn un_803005EC(un_803FBA9C, fn_802FFE6C);
    fn un_80300620(un_803FBB98, fn_802FFE6C);
    fn un_80300654(un_803FBCAC, fn_802FFE6C);
    fn un_80300688(un_803FBDC0, fn_802FFE6C);
    fn un_803006BC(un_803FBFFC, fn_802FFE6C);
    fn un_803006F0(un_803FC0FC, fn_802FFE6C);
    fn un_80300724(un_803FC22C, fn_802FFE6C);
}

open_menu! {
    fn un_803007C8(un_803FC63C, fn_802FFE6C);
    fn un_803007FC(un_803FCA40, fn_802FFE6C);
    fn un_80300830(un_803FCC38, fn_802FFE6C);
    fn un_80300864(un_803FCE4C, fn_802FFE6C);
    fn un_80300898(un_803FD064, fn_802FFE6C);
    fn un_803008CC(un_803FDAF8, fn_802FFE6C);
    fn un_80300900(un_803FDB9C, fn_802FFE6C);
    fn un_80300934(un_803FD310, fn_802FFE6C);
}

open_menu! {
    fn un_803013EC(un_803FD58C, un_80301B48);
    fn un_80301420(un_803FD864, un_80301C80);
}

// Split into src/melee/if/if_2FFC.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::if_2FFC::{fn_802FFE0C, fn_802FFE6C, un_802FFCD0, un_802FFD94, un_802FFEA4, un_802FFEE0, un_802FFF2C, un_80300218, un_80300248, un_80300290, un_80300338, un_80300378, un_803003C4, un_80300410};

// Split into src/melee/if/if_3004.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::if_3004::{fn_80300CC8, fn_80300DE0, fn_80300ED0, fn_8030110C, fn_803011EC, gw_dev_save_profile, gw_dev_unlock_all_chars, gw_dev_unlock_all_stages, un_80300758, un_80300790, un_803009E0, un_80300A88, un_80300AF4, un_80300B58, un_80300BBC, un_80300BF8, un_80300C3C, un_80300C80, un_80300D78, un_80300E74, un_80300F3C, un_80300F98, un_80301028, un_80301074, un_803010C0, un_80301280, un_803012D4, un_80301328, un_8030136C, un_80301634, un_803016D8, un_80301734, un_8030178C, un_80301800, un_80301840, un_803018BC, un_8030191C, un_80301964, un_80301A64, un_80301AD4, un_80301B48, un_80301BA8, un_80301C64, un_80301C80, un_80301CE0, un_80301D7C};
