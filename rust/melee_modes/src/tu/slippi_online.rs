//! Slippi online submenu (`--slippi-menus on`), plan step 6. Logic follows Hero's `mu_slippi_menu.c` (Melee Unlocked,
//! GPLv3, used with permission), itself a native form of Slippi's Online/Menus Gecko codes (GPL-2.0-or-later):
//! the 1P menu's hidden third row (`SEL_1P_2`) becomes Online, and menu kind 8 (empty in retail) becomes the online
//! submenu, whose art and descriptions come from Slippi's patched MnMaAll / SdMenu served by `--slippi-menus`.
//!
//! The rows are the art's own order (Ranked, Unranked, Direct, Teams, Party: the label of each row is baked into
//! Slippi's MnMaAll slot by slot, so a row's slot is its mode id). Ranked is greyed and never starts matchmaking
//! (Hero: a listing without a ladder keeps Ranked greyed and the page starts on Unranked). This round opens Unranked
//! and Direct; Teams and Party stay locked. The YouGame page is a later round: `ROWS` is the one place it plugs in.
//! Nothing here runs unless [`ENABLED`] is set (the host sets it only when the system files loaded).

#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use melee_types::*;
use super::mnmain::ext::{lbAudioAx_80024030, HSD_GObjFree, HSD_SisLib_803A5CC4};

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
pub const ROWS: [i32; 5] = [MODE_RANKED, MODE_UNRANKED, MODE_DIRECT, MODE_TEAMS, MODE_PARTY];

/// The row the player confirmed, as a mode id (-1: none yet). The host reads it with [`take_pick`].
static PICK: AtomicI32 = AtomicI32::new(-1);

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Whether `row` of the online submenu can be picked (Hero `mu_slippi_option_unlocked`, trimmed to this round).
pub fn row_unlocked(row: i32) -> bool {
    matches!(ROWS.get(row as usize), Some(&MODE_DIRECT) | Some(&MODE_UNRANKED))
}

/// The online menu's row of `mode` (0 if it has none).
pub fn row_of(mode: i32) -> usize {
    ROWS.iter().position(|&m| m == mode).unwrap_or(0)
}

/// The confirmed mode, once (the host starts matchmaking for it).
/// Pick `mode` as the online submenu's confirm does (the match process came back to the online select after an
/// Unranked game: the menus pick the same mode again, idle, and Start searches).
pub fn repick(mode: i32) {
    PICK.store(mode, Ordering::Relaxed);
}

pub fn take_pick() -> Option<i32> {
    let p = PICK.swap(-1, Ordering::Relaxed);
    // never ranked: no path starts Ranked matchmaking, whatever set the pick
    (p >= 0 && p != MODE_RANKED).then_some(p)
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
        // Direct asks for the opponent's connect code on Melee's name-entry keyboard first (Hero `load_code_entry`).
        // MELEE_SLIPPI_DIRECT_CODE (tests) skips the keyboard.
        if mode == MODE_DIRECT && std::env::var("MELEE_SLIPPI_DIRECT_CODE").map_or(true, |v| v.is_empty()) {
            open_code_entry(_gp);
            return;
        }
        PICK.store(mode, Ordering::Relaxed);
        mn_804A04F0.entering_menu = 1;
        // on to the online select while the host searches (Slippi: the CSS shows the search)
        super::slippi_css::begin(mode, &std::env::var("MELEE_SLIPPI_DIRECT_CODE").unwrap_or_default());
        super::slippi_css::enter_from_menu();
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

// ---- connect-code entry (plan step 7) ----------------------------------------------------------------------------
// Hero's `load_code_entry` (mu_slippi_css.c) opens Melee's name-entry keyboard in a connect-code mode, and his
// keyboard input (`css_codes.c` kb_input) collects code letters instead of a 4-letter name and leaves without making
// a name tag. Here the keyboard is reached the way the Names screen's "New" reaches it (mnName_8023AC40, then
// mnNameNew_EnterFromMnName): the vanilla keys, cursor and sounds; only the input routine is ours while the mode is on.
// The US keyboard has no '#' key: the first digit after the tag letters puts the '#' in (a code is letters, '#',
// digits). The name field shows the last four characters typed. Start (or the OK key) confirms, B erases (leaves when
// empty), X clears.

/// 0: off, 1: the Names screen is opening and the keyboard follows on its first frame, 2: the keyboard takes input.
static CODE_STATE: AtomicI32 = AtomicI32::new(0);
static mut CODE: [u8; CODE_MAX] = [0; CODE_MAX];
static mut CODE_LEN: usize = 0;
/// The confirmed code (Direct's search target), once.
static CODE_DONE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
/// Hero's name field holds 8 characters (tag, '#', digits).
pub const CODE_MAX: usize = 8;

pub fn code_entry_pending() -> bool {
    CODE_STATE.load(Ordering::Relaxed) == 1
}
pub fn code_entry_active() -> bool {
    CODE_STATE.load(Ordering::Relaxed) == 2
}
/// The connect code typed for Direct (the host takes it with the pick).
pub fn take_direct_code() -> Option<String> {
    CODE_DONE.lock().ok()?.take()
}

unsafe fn open_code_entry(gp: *mut HSD_GObj) {
    CODE_LEN = 0;
    CODE_STATE.store(1, Ordering::Relaxed);
    eprintln!("online menu: Direct: connect-code entry");
    super::mnname::mnName_8023AC40();
    HSD_GObjFree(gp);
}

/// The Names screen's think on its first frame while code entry is pending: press its "New" (the vanilla path).
pub unsafe fn code_entry_from_names() {
    use super::mnname::*;
    let gobj2 = (*mnName_804D6BF8).user_data as *mut MnName_GObj;
    mnName_80239878(1, gobj2 as *mut HSD_GObj);
    mnName_80238A04(gobj2 as *mut HSD_GObj, 0x18, 1);
    if !(*gobj2).text2.is_null() {
        HSD_SisLib_803A5CC4((*gobj2).text2);
    }
    let count = mnName_CountValid();
    super::mnnamenew::mnNameNew_EnterFromMnName(count as s32 as *mut core::ffi::c_void);
    CODE_STATE.store(2, Ordering::Relaxed);
}

/// A key's glyph as a code character: fullwidth or ASCII letters and digits (Hero `kb_symbol`, without his share-code
/// folding of O/I/L).
pub fn glyph_to_code_char(g: &[u8]) -> Option<u8> {
    let s = core::str::from_utf8(g.split(|&b| b == 0).next().unwrap_or(&[])).ok()?;
    let c = s.chars().next()?;
    let c = match c as u32 {
        0xFF21..=0xFF3A => (b'A' + (c as u32 - 0xFF21) as u8) as char,
        0xFF41..=0xFF5A => (b'A' + (c as u32 - 0xFF41) as u8) as char,
        0xFF10..=0xFF19 => (b'0' + (c as u32 - 0xFF10) as u8) as char,
        _ => c.to_ascii_uppercase(),
    };
    (c.is_ascii_uppercase() || c.is_ascii_digit()).then_some(c as u8)
}

/// Adds `c` to `code` by the code rules; false if it does not fit (a letter after '#', or the field is full).
pub fn code_push(code: &mut [u8; CODE_MAX], len: &mut usize, c: u8) -> bool {
    let hash = code[..*len].contains(&b'#');
    if c.is_ascii_digit() && !hash {
        if *len == 0 || *len + 2 > CODE_MAX {
            return false;
        }
        code[*len] = b'#';
        *len += 1;
    } else if c.is_ascii_uppercase() && hash || *len >= CODE_MAX {
        return false;
    }
    code[*len] = c;
    *len += 1;
    true
}

/// A complete code: letters, '#', at least one digit.
pub fn code_complete(code: &[u8]) -> bool {
    match code.iter().position(|&b| b == b'#') {
        Some(h) => h > 0 && h + 1 < code.len() && code[h + 1..].iter().all(u8::is_ascii_digit),
        None => false,
    }
}

unsafe fn code_show(data: *mut NameNewEntry) {
    use super::mnnamenew::*;
    let code = &CODE[..CODE_LEN];
    // the field's four slots keep the vanilla cursor; the whole code is drawn by code_draw (mnNameNew_8023CE4C)
    let shown = &code[code.len().saturating_sub(4)..];
    let text = &raw mut mnNameNew_CurrentNameText as *mut u8;
    for slot in 0..4 {
        let p = text.add(slot * 3);
        match shown.get(slot) {
            Some(&c) => {
                // the fullwidth form, as the keyboard types it (U+FF03 '#', U+FF10.. digits, U+FF21.. letters)
                let cp = 0xFF00 + (c as u32 - 0x20);
                let mut buf = [0u8; 4];
                let e = char::from_u32(cp).unwrap().encode_utf8(&mut buf);
                core::ptr::copy_nonoverlapping(e.as_ptr(), p, 3);
            }
            None => *p = mnNameNew_NullCharacter[0] as u8,
        }
    }
    (*data).cursor_pos = shown.len().min(3) as u8;
    mnNameNew_8023CE4C();
}

/// The name field while code entry is on: every character of the code (up to [`CODE_MAX`]) at half the slot pitch,
/// not just the last four (Hero shows the whole code). Returns false when code entry is off (the vanilla draw runs).
pub unsafe fn code_draw(text: *mut HSD_Text, slot_pitch: f32) -> bool {
    use melee_runtime::tu::hsd_3A64 as sis;
    if !code_entry_active() {
        return false;
    }
    (*text).font_size.x = 0.028;
    let pitch = slot_pitch * 0.5;
    for (i, &c) in CODE[..CODE_LEN].iter().enumerate() {
        let ch = char::from_u32(0xFF00 + (c as u32 - 0x20)).unwrap();
        let mut b = [0u8; 4];
        let n = sis::HSD_SisLib_803A6B98(text, pitch * i as f32 / (*text).font_size.x, 0.0, Some(ch.encode_utf8(&mut b)));
        let col = super::mnnamenew::mnNameNew_804D4F78;
        sis::HSD_SisLib_803A74F0(text, n, &col);
    }
    true
}

unsafe fn code_leave(confirmed: bool) {
    CODE_STATE.store(0, Ordering::Relaxed);
    super::mnnamenew::mnNameNew_8023B224(0); // back to the Names screen; no name tag is made
    if confirmed {
        let code = String::from_utf8_lossy(&CODE[..CODE_LEN]).into_owned();
        eprintln!("online menu: Direct: connect code {code} confirmed");
        if let Ok(mut d) = CODE_DONE.lock() {
            *d = Some(code);
        }
        PICK.store(MODE_DIRECT, Ordering::Relaxed);
        mn_804A04F0.entering_menu = 1;
        super::slippi_css::begin(MODE_DIRECT, &String::from_utf8_lossy(&CODE[..CODE_LEN]));
        super::slippi_css::enter_from_menu();
    } else {
        eprintln!("online menu: Direct: code entry cancelled");
        mn_804A04F0.entering_menu = 0;
        super::mnname::ext::HSD_SisLib_803A5D30();
        mn_80229894(MENU_KIND_8 as s32, row_of(MODE_DIRECT) as u16, 3);
    }
}

unsafe fn code_confirm() -> bool {
    if code_complete(&CODE[..CODE_LEN]) {
        lbAudioAx_80024030(1);
        code_leave(true);
        true
    } else {
        lbAudioAx_80024030(3);
        false
    }
}

/// The keyboard's input while code entry is on (Hero `kb_input`, for connect codes).
pub unsafe fn code_input() {
    use super::mnnamenew::*;
    let data = (*mnNameNew_804D6C08).user_data as *mut NameNewEntry;
    let hov = &raw mut mn_804A04F0.hovered_selection;
    let b = mn_80229624(mnNameNew_PortInUse as u32);
    mn_804A04F0.buttons = b as u64;
    if b & MenuInput_AButton as u32 != 0 {
        let sel = *hov;
        if sel < 0x32 {
            let mut g = [0 as core::ffi::c_char; 8];
            AddCharacterToName(g.as_mut_ptr(), sel as u8, 0, (*data).mode);
            let g: [u8; 8] = core::mem::transmute(g);
            match glyph_to_code_char(&g) {
                Some(c) if code_push(&mut CODE, &mut CODE_LEN, c) => {
                    lbAudioAx_80024030(1);
                    if CODE_LEN == CODE_MAX {
                        *hov = 0x39;
                    }
                }
                _ => lbAudioAx_80024030(3),
            }
        } else if sel == 0x36 {
            if CODE_LEN > 0 {
                code_erase();
                lbAudioAx_80024030(0);
            } else {
                lbAudioAx_80024030(3);
            }
        } else if sel == 0x38 || sel == 0x39 {
            if code_confirm() {
                return;
            }
        } else if sel == 0x32 {
            lbAudioAx_80024030(0);
            return code_leave(false);
        } else {
            lbAudioAx_80024030(3);
        }
    } else if b & MenuInput_StartButton as u32 != 0 {
        if code_confirm() {
            return;
        }
    } else if b & MenuInput_Back as u32 != 0 {
        lbAudioAx_80024030(0);
        if CODE_LEN == 0 {
            return code_leave(false);
        }
        code_erase();
    } else if b & MenuInput_XButton as u32 != 0 {
        CODE_LEN = 0;
        lbAudioAx_80024030(0);
    } else if b & (MenuInput_Up | MenuInput_Down | MenuInput_Left | MenuInput_Right) as u32 != 0 {
        let next = mnNameNew_8023BAA8(data, b as i32, *hov as u8) as u16;
        if next != *hov {
            lbAudioAx_80024030(2);
            mnNameNew_8023B0F8(mnNameNew_804D6C08, next as u8);
            *hov = next;
            if next < 0x32 {
                (*data).last_key_sel = next as u8;
            }
            mnNameNew_8023B314(data, next as i32);
        }
    }
    code_show(data);
}

unsafe fn code_erase() {
    CODE_LEN -= 1;
    // the '#' goes with the last digit
    if CODE_LEN > 0 && CODE[CODE_LEN - 1] == b'#' {
        CODE_LEN -= 1;
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
    fn code_rules() {
        let mut c = [0u8; CODE_MAX];
        let mut n = 0;
        assert!(!code_push(&mut c, &mut n, b'0')); // a digit needs a tag first
        for &k in b"BBBB002" {
            assert!(code_push(&mut c, &mut n, k));
        }
        assert_eq!(&c[..n], b"BBBB#002");
        assert!(code_complete(&c[..n]));
        assert!(!code_push(&mut c, &mut n, b'3')); // full
        assert!(!code_complete(b"BBBB") && !code_complete(b"#12") && !code_complete(b"AB#"));
        let mut n2 = 2;
        let mut c2 = *b"A#\0\0\0\0\0\0";
        assert!(!code_push(&mut c2, &mut n2, b'Z')); // no letters after '#'
        assert_eq!(glyph_to_code_char("Ｂ".as_bytes()), Some(b'B'));
        assert_eq!(glyph_to_code_char("２".as_bytes()), Some(b'2'));
        assert_eq!(glyph_to_code_char("＄".as_bytes()), None);
    }

    #[test]
    fn rows_and_locks() {
        assert_eq!(ROWS[0], MODE_RANKED);
        assert!(!row_unlocked(0) && !row_unlocked(row_of(MODE_RANKED) as i32));
        assert!(row_unlocked(1) && row_unlocked(2));
        assert!(!row_unlocked(3) && !row_unlocked(4));
        PICK.store(MODE_RANKED, Ordering::Relaxed);
        assert_eq!(take_pick(), None);
        assert_eq!(take_pick(), None);
        PICK.store(MODE_DIRECT, Ordering::Relaxed);
        assert_eq!(take_pick(), Some(MODE_DIRECT));
        assert_eq!(take_pick(), None);
    }
}
