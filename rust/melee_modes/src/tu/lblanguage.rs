//! src/melee/lb/lblanguage.c: the game language setting (`gmMainLib_804D3EE0->language`) and the
//! saved language in the game prefs. Checked bit for bit against the C by a state oracle
//! (tests/tu_lblanguage.rs over gen/turef/tu_lblanguage_ref.c).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use melee_types::*;

/// `enum { LANG_JP, LANG_US, LANG_COUNT }` (lb/lblanguage.h).
pub const LANG_JP: enum_t = 0;
pub const LANG_US: enum_t = 1;
pub const LANG_COUNT: enum_t = 2;

/// Callees and globals of other TUs (gm/gmmain_lib.c). Each is a settable hook; unset it panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $h } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // BEGIN GENERATED ext

    // END GENERATED ext
    /// The global `struct gmm_x0* gmMainLib_804D3EE0` (gm/gmmain_lib.c), read through a hook.
    hook!(H_gmMainLib_804D3EE0, gmMainLib_804D3EE0, () -> *mut gmm_x0);
    hook!(H_gmMainLib_GetGamePrefs, gmMainLib_GetGamePrefs, () -> *mut GamePrefs);
}

/// `lbLang_GetLanguageSetting`.
pub fn lbLang_GetLanguageSetting() -> enum_t {
    unsafe { (*ext::gmMainLib_804D3EE0()).language as enum_t }
}

/// `lbLang_SetLanguageSetting`: stores `language` when it is a valid language; returns it as given.
pub fn lbLang_SetLanguageSetting(language: enum_t) -> enum_t {
    if language >= 0 && language < LANG_COUNT {
        unsafe { (*ext::gmMainLib_804D3EE0()).language = language as u8 };
    }
    language
}

/// `lbLang_IsSettingJP`.
pub fn lbLang_IsSettingJP() -> bool {
    unsafe { (*ext::gmMainLib_804D3EE0()).language as enum_t == LANG_JP }
}

/// `lbLang_IsSettingUS`.
pub fn lbLang_IsSettingUS() -> bool {
    unsafe { (*ext::gmMainLib_804D3EE0()).language as enum_t == LANG_US }
}

/// `lbLang_GetSavedLanguage`.
pub fn lbLang_GetSavedLanguage() -> enum_t {
    unsafe { (*ext::gmMainLib_GetGamePrefs()).saved_language as enum_t }
}

/// `lbLang_SetSavedLanguage`.
pub fn lbLang_SetSavedLanguage(language: enum_t) {
    if language >= 0 && language < LANG_COUNT {
        unsafe { (*ext::gmMainLib_GetGamePrefs()).saved_language = language as u8 };
    }
}

/// `lbLang_IsSavedLanguageJP`.
pub fn lbLang_IsSavedLanguageJP() -> bool {
    unsafe { (*ext::gmMainLib_GetGamePrefs()).saved_language as enum_t == LANG_JP }
}

/// `lbLang_IsSavedLanguageUS`.
pub fn lbLang_IsSavedLanguageUS() -> bool {
    unsafe { (*ext::gmMainLib_GetGamePrefs()).saved_language as enum_t == LANG_US }
}
