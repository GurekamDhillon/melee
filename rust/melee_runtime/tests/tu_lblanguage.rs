//! tu::lblanguage vs the C (state oracle): gen/turef/tu_lblanguage_ref.c is the game's lblanguage.c compiled
//! natively over a synthetic gmMainLib state; expected text in tests/data/tu_lblanguage.txt.
#![allow(static_mut_refs)]
#[macro_use]
#[path = "lbtest/mod.rs"]
mod common_lb;
use common_lb::*;
use melee_runtime::tu::lblanguage as t;
use melee_types::*;

static mut G0: gmm_x0 = unsafe { core::mem::zeroed() };
static mut GP: GamePrefs = unsafe { core::mem::zeroed() };

#[test]
fn lblanguage() {
    unsafe {
        t::ext::H_gmMainLib_804D3EE0 = Some(|| core::ptr::addr_of_mut!(G0));
        t::ext::H_gmMainLib_GetGamePrefs = Some(|| core::ptr::addr_of_mut!(GP));
        for i in 0..400 {
            let a = (rnd() % 9) as i32 - 3;
            G0.language = (rnd() % 5) as u8;
            GP.saved_language = (rnd() % 5) as u8;
            outf!("{} lang={} saved={}\n", i, G0.language, GP.saved_language);
            outf!(
                "  get={} jp={} us={} sget={} sjp={} sus={}\n",
                t::lbLang_GetLanguageSetting(),
                t::lbLang_IsSettingJP() as i32,
                t::lbLang_IsSettingUS() as i32,
                t::lbLang_GetSavedLanguage(),
                t::lbLang_IsSavedLanguageJP() as i32,
                t::lbLang_IsSavedLanguageUS() as i32
            );
            outf!("  set({})={}", a, t::lbLang_SetLanguageSetting(a));
            t::lbLang_SetSavedLanguage(a);
            outf!(" -> lang={} saved={}\n", G0.language, GP.saved_language);
        }
    }
    check("tu_lblanguage");
}
