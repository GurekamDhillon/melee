//! Port of src/melee/lb/lb_0192.c (the disc-error / cover-open screen: `lb_800192A8` shows the
//! message for the drive status and runs a render loop until the drive is fine again).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_0192.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::{c_void, CStr};
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn DVDGetDriveStatus() -> i32;
    fn GXInvalidateTexAll();
    fn GXInvalidateVtxCache();
    fn HSD_Init_803755A8() = crate::tu::adapt::HSD_Init_803755A8__49528d;
    fn HSD_Rumble_80378524(a0: i32) = crate::tu::adapt::HSD_Rumble_80378524__8fd3f6;
    fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: *const i8) -> i32;
    fn HSD_SisLib_803A70A0(a0: *mut HSD_Text, a1: i32, a2: *const i8, a3: *const i8) -> i32;
    fn HSD_SisLib_803A84BC(a0: *mut HSD_GObj, a1: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A84BC__b2aa0b;
    fn HSD_StartRender(a0: u32) = crate::tu::adapt::HSD_StartRender__c91577;
    fn HSD_VICopyXFBAsync(a0: u32) = crate::tu::adapt::HSD_VICopyXFBAsync__35ed8e;
    fn lbAudioAx_80024F08() = crate::tu::adapt::lbAudioAx_80024F08__08bb8a;
    fn lbAudioAx_80024F6C() = crate::tu::adapt::lbAudioAx_80024F6C__7ce9bf;
    fn lbLang_IsSavedLanguageJP() -> i32 = crate::tu::adapt::lbLang_IsSavedLanguageJP__e2b095;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut gmMainLib_8046B0F0: gmMainLib_8046B0F0_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `struct strings`: the three message lines.
pub struct strings {
    pub x0: &'static CStr,
    pub x4: &'static CStr,
    pub x8: &'static CStr,
}

const fn s(x0: &'static CStr, x4: &'static CStr, x8: &'static CStr) -> strings {
    strings { x0, x4, x8 }
}

/// `jp_msg` (static)
pub static jp_msg: [strings; 6] = [
    s(c"", c"ディスクカバーを閉めてください。", c""),
    s(c"", c"スマッシュブラザーズＤＸのディスクをセットしてください。", c""),
    s(
        c"このディスクは",
        c"スマッシュブラザーズＤＸのディスクではありません。",
        c"スマッシュブラザーズＤＸのディスクをセットしてください。",
    ),
    s(c"ディスクを読めませんでした。", c"詳しくは本体の取扱説明書をお読みください。", c""),
    s(
        c"エラーが発生しました。",
        c"本体のパワーボタンを押して電源をＯＦＦにし、",
        c"本体の取扱説明書の指示にしたがってください。",
    ),
    s(c"", c"ディスクを読み込んでいます。", c""),
];

/// `us_msg` (static)
pub static us_msg: [strings; 6] = [
    s(c"", c"Close the Disc Cover.", c""),
    s(c"", c"Insert the Super Smash Bros. Melee Game Disc.", c""),
    s(
        c"The inserted Game Disc is not",
        c"Super Smash Bros. Melee.",
        c"Insert the Super Smash Bros. Melee Game Disc.",
    ),
    s(c"The Game Disc could not be read.", c"Read the NINTENDO GAMECUBE Instruction Booklet", c"for more information."),
    s(
        c"An error has occurred. Turn the power OFF and",
        c"check the NINTENDO GAMECUBE Instruction Booklet",
        c"for further instructions.",
    ),
    s(c"", c"Reading the Game Disc...", c""),
];

/// 4D63D0 (static)
pub static mut lb_804D63D0: *mut HSD_Text = core::ptr::null_mut();

/// Drive status -> message index (-1: nothing to show).
pub unsafe fn lb_80019230() -> i32 {
    melee_core::ct!("lb_80019230");
    match ext::DVDGetDriveStatus() {
        5 => 0,
        4 => 1,
        6 => 2,
        11 => 3,
        -1 => 4,
        1 => 5,
        _ => -1,
    }
}

/// 0192A8: shows the disc message until the drive status goes back to normal; `cb` runs each
/// frame.
pub unsafe fn lb_800192A8(cb: Option<unsafe extern "C" fn()>) {
    melee_core::ct!("lb_800192A8");
    let mut prev_i: i32 = -1;
    let mut i: i32 = lb_80019230();

    if i != -1 && i != 5 {
        ext::lbAudioAx_80024F08();
        ext::HSD_Rumble_80378524(1);
        lb_804D63D0 = ext::HSD_SisLib_803A6754(0, -1);
        let jp = ext::lbLang_IsSavedLanguageJP() != 0;
        if jp {
            let text = lb_804D63D0;
            (*text).font_size.x = 0.67f32;
            (*text).font_size.y = 0.75f32;
        } else {
            let text = lb_804D63D0;
            (*text).font_size.x = 0.58f32;
            (*text).font_size.y = 0.70f32;
        }
        (*lb_804D63D0).default_alignment = 1;
        (*lb_804D63D0).default_kerning = 1;
        if ext::lbLang_IsSavedLanguageJP() != 0 {
            let m = &jp_msg[i as usize];
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, -68.0, c"%s".as_ptr(), m.x0.as_ptr());
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, -34.0, c"%s".as_ptr(), m.x4.as_ptr());
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, 0.0, c"%s".as_ptr(), m.x8.as_ptr());
        } else {
            let m = &us_msg[i as usize];
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, -68.0, c"%s".as_ptr(), m.x0.as_ptr());
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, -34.0, c"%s".as_ptr(), m.x4.as_ptr());
            ext::HSD_SisLib_803A6B98(lb_804D63D0, 0.0, 0.0, c"%s".as_ptr(), m.x8.as_ptr());
        }

        loop {
            if i != prev_i {
                prev_i = i;
                if i != -1 {
                    let m = if ext::lbLang_IsSavedLanguageJP() != 0 { &jp_msg[i as usize] } else { &us_msg[i as usize] };
                    ext::HSD_SisLib_803A70A0(lb_804D63D0, 0, c"%s".as_ptr(), m.x0.as_ptr());
                    ext::HSD_SisLib_803A70A0(lb_804D63D0, 1, c"%s".as_ptr(), m.x4.as_ptr());
                    ext::HSD_SisLib_803A70A0(lb_804D63D0, 2, c"%s".as_ptr(), m.x8.as_ptr());
                }
            }
            ext::GXInvalidateVtxCache();
            ext::GXInvalidateTexAll();
            ext::HSD_StartRender(0 /* HSD_RP_SCREEN */);
            ext::HSD_SisLib_803A84BC(core::ptr::null_mut(), lb_804D63D0);
            ext::HSD_Init_803755A8();
            ext::HSD_VICopyXFBAsync(0 /* HSD_RP_SCREEN */);
            if let Some(cb) = cb {
                cb();
            }
            if i == -1 {
                break;
            }
            i = lb_80019230();
        }

        ext::lbAudioAx_80024F6C();
        ext::HSD_Rumble_80378524(0);
        ext::HSD_SisLib_803A5CC4(lb_804D63D0);
        ext::gmMainLib_8046B0F0.xC = 1;
    }
}
