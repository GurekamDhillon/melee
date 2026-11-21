//! Slippi online submenu (`--slippi-menus on`), plan step 6. Logic follows Hero's `mu_slippi_menu.c` (Melee Unlocked,
//! GPLv3, used with permission), itself a native form of Slippi's Online/Menus Gecko codes (GPL-2.0-or-later):
//! the 1P menu's hidden third row (`SEL_1P_2`) becomes Online, and menu kind 8 (empty in retail) becomes the online
//! submenu, whose art and descriptions come from Slippi's patched MnMaAll / SdMenu served by `--slippi-menus`.
//!
//! The rows are Hero's Slippi page (Direct, Unranked, Teams, Party; no Slippi Ranked row). This round opens Direct
//! and Unranked; Teams and Party stay locked. The YouGame page is a later round: `ROWS` is the one place it plugs in.
//! Nothing here runs unless [`ENABLED`] is set (the host sets it only when the system files loaded).

#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use melee_types::*;

use super::mnmain::{
    mn_803EB444, mn_803EB57C, mn_803EB5A0, mn_803EB5C4, mn_803EB66C, mn_803EB6B0, mn_804A04F0, mn_80229624, mn_80229894, sfxBack,
    sfxForward, sfxMove,
};

/// The system-file layer loaded and the host asked for the online menus.
pub static ENABLED: AtomicBool = AtomicBool::new(false);

/// Slippi's matchmaking mode ids (Hero `MU_SLP_MODE_*`, Slippi `ONLINE_MODE_*`).
pub const MODE_RANKED: i32 = 0;
pub const MODE_UNRANKED: i32 = 1;
pub const MODE_DIRECT: i32 = 2;
pub const MODE_TEAMS: i32 = 3;
pub const MODE_PARTY: i32 = 4;

/// The Slippi page's rows, top to bottom (Hero `mu_online_slippi_rows`).
pub const ROWS: [i32; 4] = [MODE_DIRECT, MODE_UNRANKED, MODE_TEAMS, MODE_PARTY];

/// The row the player confirmed, as a mode id (-1: none yet). The host reads it with [`take_pick`].
static PICK: AtomicI32 = AtomicI32::new(-1);

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Whether `row` of the online submenu can be picked (Hero `mu_slippi_option_unlocked`, trimmed to this round).
pub fn row_unlocked(row: i32) -> bool {
    matches!(ROWS.get(row as usize), Some(&MODE_DIRECT) | Some(&MODE_UNRANKED))
}

/// The confirmed mode, once (the host starts matchmaking for it).
pub fn take_pick() -> Option<i32> {
    let p = PICK.swap(-1, Ordering::Relaxed);
    (p >= 0).then_some(p)
}

static mut ANIM: [AnimLoopSettings; 8] = unsafe { core::mem::zeroed() };
static mut DESCRIPTIONS: [u16; 8] = [0; 8];

/// Menu kind 8 becomes the online submenu (Hero `mu_mnmain_install_online_menu`, Slippi OnMenuPrep). The row art
/// reuses retail's preview loops read on past their three entries (as on the console), option art from frame 140,
/// and Slippi's description ids in its patched SdMenu. Called once per main-menu entry while enabled.
pub unsafe fn install() {
    if !enabled() {
        return;
    }
    // Slippi's description ids per mode slot 0..7 (Online/Menus, from Slippi's code, not the disc).
    const SOURCE_DESCRIPTIONS: [u16; 8] = [0x645, 0x646, 0x647, 0x64B, 0x64C, 0x648, 0x649, 0x64A]; // [Slippi text id]
    let mut original: [AnimLoopSettings; 8] = core::mem::zeroed();
    for i in 0..3 {
        original[i] = mn_803EB57C[i];
        original[3 + i] = mn_803EB5A0[i];
    }
    original[6] = mn_803EB5C4[0];
    original[7] = mn_803EB5C4[1];
    for (i, &mode) in ROWS.iter().enumerate() {
        ANIM[i] = original[mode as usize];
        DESCRIPTIONS[i] = SOURCE_DESCRIPTIONS[mode as usize];
    }
    let k = &mut mn_803EB6B0[MENU_KIND_8 as usize];
    k.anim_loop = ANIM.as_mut_ptr();
    k.start_frame = 140.0;
    k.description_indices = DESCRIPTIONS.as_mut_ptr();
    k.selection_count = ROWS.len() as u8;
    k.think = Some(online_menu_think);
    // The 1P menu's row 2 (Online): Slippi's description and the character select's preview (its own frames are blank).
    mn_803EB66C[SEL_1P_2 as usize] = 0x644; // [Slippi text id]
    mn_803EB444[SEL_1P_2 as usize] = mn_803EB57C[0];
}

/// Picking Online on the 1P menu (Hero `mu_slippi_switch_to_online_menu`, Slippi OnlineModeOptionSelected).
pub unsafe fn open_from_1p() {
    install();
    let first = (0..ROWS.len() as i32).find(|&r| row_unlocked(r)).unwrap_or(0);
    eprintln!("online menu: opened (Slippi page, {} rows)", ROWS.len());
    mn_80229894(MENU_KIND_8 as s32, first as u16, 1);
}

/// The online submenu's think (Hero `mu_slippi_online_menu_think`, trimmed to cursor, back and confirm).
pub unsafe extern "C" fn online_menu_think(_gp: *mut HSD_GObj) {
    let count = ROWS.len() as i32;
    let buttons = mn_80229624(4);
    mn_804A04F0.buttons = buttons as u64;
    let cur = mn_804A04F0.hovered_selection as i32;
    if buttons & MenuInput_Confirm as u32 != 0 {
        if !row_unlocked(cur) {
            sfxBack();
            return;
        }
        sfxForward();
        let mode = ROWS[cur as usize];
        eprintln!("online menu: picked {} (mode {mode})", mode_name(mode));
        PICK.store(mode, Ordering::Relaxed);
        mn_804A04F0.entering_menu = 1;
    } else if buttons & MenuInput_Back as u32 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0;
        eprintln!("online menu: back to the 1P menu");
        mn_80229894(MENU_KIND_1P as s32, SEL_1P_2 as u16, 0);
    } else if buttons & (MenuInput_Up as u32 | MenuInput_Down as u32) != 0 {
        sfxMove();
        let step = if buttons & MenuInput_Up as u32 != 0 { count - 1 } else { 1 };
        let mut s = cur;
        for _ in 0..count {
            s = (s + step) % count;
            if row_unlocked(s) {
                break;
            }
        }
        mn_804A04F0.hovered_selection = s as u16;
    }
}

pub fn mode_name(mode: i32) -> &'static str {
    match mode {
        MODE_RANKED => "Ranked",
        MODE_UNRANKED => "Unranked",
        MODE_DIRECT => "Direct",
        MODE_TEAMS => "Teams",
        MODE_PARTY => "Party",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_locks() {
        assert_eq!(ROWS[0], MODE_DIRECT);
        assert!(row_unlocked(0) && row_unlocked(1));
        assert!(!row_unlocked(2) && !row_unlocked(3) && !row_unlocked(4));
        assert_eq!(take_pick(), None);
        PICK.store(MODE_DIRECT, Ordering::Relaxed);
        assert_eq!(take_pick(), Some(MODE_DIRECT));
        assert_eq!(take_pick(), None);
    }
}
