//! menuguard (PORT-LEVEL GUARD, not in the C): the menu's mode launches, checked against the modes the Rust port is
//! known to run end to end. A mode outside the list is refused the way retail refuses a locked entry: the menu plays
//! the error SFX (`lbAudioAx_80024030(3)`, as mncharsel/mndiagram do) and stays put, instead of launching a scene that
//! stops on an unported TODO. One log line per refusal.
//!
//! Only the engine turns it on (`ENABLED`, melee_engine seams install), so the TU oracles see the C behaviour.
//! `MELEE_MENUGUARD=off` lets every mode through (to test a mode being ported).

use melee_types::*;

/// Set by the engine at install.
pub static mut ENABLED: bool = false;

/// The modes the menu may launch: they reach their scenes and back without a TODO stop.
pub const KNOWN_WORKING: &[GameModeKind] = &[
    GM_TITLE, GM_MENU, GM_VS, GM_CLASSIC, GM_TRAINING, GM_ADVENTURE, GM_ALLSTAR,
    // Event Match and the Stadium (tools/rust-port/play/onep_modes.sh)
    GM_EVENT, GM_TARGET_TEST, GM_HOME_RUN_CONTEST,
    GM_10MAN_VS, GM_100MAN_VS, GM_3MIN_VS, GM_15MIN_VS, GM_ENDLESS_VS, GM_CRUEL_VS,
];

fn name(mode: i32) -> &'static str {
    match mode as GameModeKind {
        GM_CLASSIC => "Classic", GM_ADVENTURE => "Adventure", GM_ALLSTAR => "All-Star", GM_EVENT => "Event Match",
        GM_TARGET_TEST => "Target Test", GM_HOME_RUN_CONTEST => "Home-Run Contest", GM_TRAINING => "Training",
        GM_10MAN_VS | GM_100MAN_VS | GM_3MIN_VS | GM_15MIN_VS | GM_ENDLESS_VS | GM_CRUEL_VS => "Multi-Man Melee",
        GM_TOURNAMENT => "Tournament", GM_CAMERA_MODE => "Camera Mode",
        GM_TOY_GALLERY => "Trophy Gallery", GM_TOY_LOTTERY => "Trophy Lottery", GM_TOY_COLLECTION => "Trophy Collection",
        GM_STAMINA_VS | GM_SUPER_SUDDEN_DEATH_VS | GM_GIANT_VS | GM_TINY_VS | GM_INVISIBLE_VS | GM_CAMERA_VS
        | GM_SINGLE_BUTTON_VS | GM_LIGHTNING_VS | GM_SLOMO_VS => "Special Melee",
        _ => "mode",
    }
}

/// True when `mode` must not launch: the error SFX has played and the caller returns without changing scene.
///
/// # Safety
/// Menu-thread state (the menu's statics).
pub unsafe fn deny(mode: i32) -> bool {
    if !ENABLED || KNOWN_WORKING.contains(&(mode as GameModeKind)) {
        return false;
    }
    if std::env::var("MELEE_MENUGUARD").is_ok_and(|v| v == "off") {
        return false;
    }
    eprintln!("menuguard: {} (mode {mode}) is not ported yet; refused at the menu", name(mode));
    crate::tu::mnmain::ext::lbAudioAx_80024030(3);   // as sfxForward plays, through the same stub
    crate::tu::mnmain::mn_804A04F0.entering_menu = 0;
    true
}

/// The title screen's B / Y shortcut into the developer debug menu (a TARGET_PC addition, not retail): only with
/// `MELEE_DEBUG_MENU=1`. The debug menu is not ported (its menu tables crash `un_80303FD4`), and a player who backs out
/// of the main menu to the title with B and presses B again must get the retail title, not a crash.
pub fn debug_menu_shortcut() -> bool {
    std::env::var("MELEE_DEBUG_MENU").is_ok_and(|v| v == "1")
}
