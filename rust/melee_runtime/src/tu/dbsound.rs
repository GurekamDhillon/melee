//! Port of src/melee/db/dbsound.c (debug sound overlay: X + d-left cycles FGM/BGM mutes and the
//! voice-count display).
//!
//! The C globals live in [`DbSound`]; calls out of the TU go through [`DbSoundHost`], whose
//! default implementation [`ext::Ext`] is the unported-callee stub set (panics, named like the C).
#![allow(non_snake_case)]

use melee_types::{DevText, GXColor, HSD_GObj};

use super::dbeffect::pad;

/// `db_SoundToggleOrder` (FGM bit 1, BGM bit 0).
pub const DB_SOUND_TOGGLE_ORDER: [i32; 4] = [3, 2, 0, 1];

/// Callees of dbsound.c.
pub trait DbSoundHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn DevText_GetGObj(&mut self) -> *mut HSD_GObj;
    fn DevText_Create(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, buf: *mut i8) -> *mut DevText;
    fn DevText_Show(&mut self, gobj: *mut HSD_GObj, t: *mut DevText);
    fn DevText_HideCursor(&mut self, t: *mut DevText);
    fn DevText_SetBGColor(&mut self, t: *mut DevText, c: GXColor);
    fn DevText_SetTextColor(&mut self, t: *mut DevText, c: GXColor);
    fn DevText_SetScale(&mut self, t: *mut DevText, x: f32, y: f32);
    fn DevText_ShowBackground(&mut self, t: *mut DevText);
    fn DevText_ShowText(&mut self, t: *mut DevText);
    fn DevText_HideBackground(&mut self, t: *mut DevText);
    fn DevText_HideText(&mut self, t: *mut DevText);
    fn DevText_Erase(&mut self, t: *mut DevText);
    fn DevText_SetCursorXY(&mut self, t: *mut DevText, x: i32, y: i32);
    /// `DevText_Printf` with the C format already applied.
    fn DevText_Printf(&mut self, t: *mut DevText, s: &str);
    fn lbAudioAx_80025098(&mut self, on: bool);
    fn lbAudioAx_80028B2C(&mut self) -> i32;
    fn lbAudioAx_80028B4C(&mut self) -> i32;
    fn lbAudioAx_80025064(&mut self, bgm: i32, fgm: i32);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl DbSoundHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown (db/dbinit.c)") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed (db/dbinit.c)") }
        fn DevText_GetGObj(&mut self) -> *mut HSD_GObj { todo!("DevText_GetGObj (if/textdraw.c)") }
        fn DevText_Create(&mut self, _: i32, _: i32, _: i32, _: i32, _: i32, _: *mut i8) -> *mut DevText { todo!("DevText_Create (if/textdraw.c)") }
        fn DevText_Show(&mut self, _: *mut HSD_GObj, _: *mut DevText) { todo!("DevText_Show (if/textdraw.c)") }
        fn DevText_HideCursor(&mut self, _: *mut DevText) { todo!("DevText_HideCursor (if/textdraw.c)") }
        fn DevText_SetBGColor(&mut self, _: *mut DevText, _: GXColor) { todo!("DevText_SetBGColor (if/textdraw.c)") }
        fn DevText_SetTextColor(&mut self, _: *mut DevText, _: GXColor) { todo!("DevText_SetTextColor (if/textdraw.c)") }
        fn DevText_SetScale(&mut self, _: *mut DevText, _: f32, _: f32) { todo!("DevText_SetScale (if/textdraw.c)") }
        fn DevText_ShowBackground(&mut self, _: *mut DevText) { todo!("DevText_ShowBackground (if/textdraw.c)") }
        fn DevText_ShowText(&mut self, _: *mut DevText) { todo!("DevText_ShowText (if/textdraw.c)") }
        fn DevText_HideBackground(&mut self, _: *mut DevText) { todo!("DevText_HideBackground (if/textdraw.c)") }
        fn DevText_HideText(&mut self, _: *mut DevText) { todo!("DevText_HideText (if/textdraw.c)") }
        fn DevText_Erase(&mut self, _: *mut DevText) { todo!("DevText_Erase (if/textdraw.c)") }
        fn DevText_SetCursorXY(&mut self, _: *mut DevText, _: i32, _: i32) { todo!("DevText_SetCursorXY (if/textdraw.c)") }
        fn DevText_Printf(&mut self, _: *mut DevText, _: &str) { todo!("DevText_Printf (if/textdraw.c)") }
        fn lbAudioAx_80025098(&mut self, _: bool) { todo!("lbAudioAx_80025098 (lb/lbaudio_ax.c)") }
        fn lbAudioAx_80028B2C(&mut self) -> i32 { todo!("lbAudioAx_80028B2C (lb/lbaudio_ax.c)") }
        fn lbAudioAx_80028B4C(&mut self) -> i32 { todo!("lbAudioAx_80028B4C (lb/lbaudio_ax.c)") }
        fn lbAudioAx_80025064(&mut self, _: i32, _: i32) { todo!("lbAudioAx_80025064 (lb/lbaudio_ax.c)") }
    }
}

/// The globals of dbsound.c. The DevText keeps a pointer into `text_buf`: keep this in place
/// once [`DbSound::fn_SetupSoundInfo`] has run.
#[derive(Clone, Debug)]
pub struct DbSound {
    /// `db_SoundInfoText_buf`.
    pub text_buf: [i8; 0x70],
    /// `db_804D4AF8` (initialised to 1, never read in this TU).
    pub db_804D4AF8: i32,
    pub sound_toggles: i32,
    pub show_sound_info: i32,
    /// `db_SoundRelated_6B68` / `6B6C`: physical-voice peak and its hold timer.
    pub pvoice_peak: i32,
    pub pvoice_hold: i32,
    /// `db_SoundRelated_6B70` / `6B74`: virtual-voice peak and its hold timer.
    pub vvoice_peak: i32,
    pub vvoice_hold: i32,
    pub text: *mut DevText,
    /// `db_804D6B7C`.
    pub gobj: *mut HSD_GObj,
}

impl Default for DbSound {
    fn default() -> Self {
        Self {
            text_buf: [0; 0x70],
            db_804D4AF8: 1,
            sound_toggles: 0,
            show_sound_info: 0,
            pvoice_peak: 0,
            pvoice_hold: 0,
            vvoice_peak: 0,
            vvoice_hold: 0,
            text: core::ptr::null_mut(),
            gobj: core::ptr::null_mut(),
        }
    }
}

/// One peak-hold step: a new peak latches for 0xF0 frames, then the peak follows the value.
pub fn db_peak_hold(peak: &mut i32, hold: &mut i32, x: i32) {
    if *peak < x {
        *peak = x;
        *hold = 0xF0;
    }
    *hold = hold.wrapping_sub(1);
    if *hold <= 0 {
        *peak = x;
        *hold = 0;
    }
}

impl DbSound {
    /// `fn_SetupSoundInfo`.
    pub fn fn_SetupSoundInfo<H: DbSoundHost + ?Sized>(&mut self, h: &mut H) {
        let bg = GXColor { r: 0x80, g: 0x80, b: 0x80, a: 0x80 };
        let fg = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
        self.gobj = h.DevText_GetGObj();
        self.sound_toggles = 0;
        self.show_sound_info = 0;
        self.pvoice_peak = 0;
        self.pvoice_hold = 0;
        self.vvoice_peak = 0;
        self.vvoice_hold = 0;
        self.text = h.DevText_Create(9, 420, 60, 18, 3, self.text_buf.as_mut_ptr());
        if !self.text.is_null() {
            h.DevText_Show(self.gobj, self.text);
            h.DevText_HideCursor(self.text);
            h.DevText_SetBGColor(self.text, bg);
            h.DevText_SetTextColor(self.text, fg);
            h.DevText_SetScale(self.text, 12.0, 16.0);
            h.DevText_HideBackground(self.text);
            h.DevText_HideText(self.text);
        }
    }

    /// `fn_UpdateSoundInfo`.
    pub fn fn_UpdateSoundInfo<H: DbSoundHost + ?Sized>(&mut self, h: &mut H) {
        h.lbAudioAx_80025098(3 < self.show_sound_info);
        if !self.text.is_null() {
            match self.show_sound_info {
                0..=3 => {
                    h.DevText_HideBackground(self.text);
                    h.DevText_HideText(self.text);
                }
                4..=7 => {
                    let x = h.lbAudioAx_80028B2C();
                    db_peak_hold(&mut self.pvoice_peak, &mut self.pvoice_hold, x);
                    let x = h.lbAudioAx_80028B4C();
                    db_peak_hold(&mut self.vvoice_peak, &mut self.vvoice_hold, x);
                    h.DevText_Erase(self.text);
                    h.DevText_SetCursorXY(self.text, 1, 0);
                    let s = match DB_SOUND_TOGGLE_ORDER[self.sound_toggles as usize] & 3 {
                        0 => "FGM:OFF  BGM:OFF",
                        1 => "FGM:OFF  BGM:ON ",
                        2 => "FGM:ON   BGM:OFF",
                        _ => "FGM:ON   BGM:ON ",
                    };
                    h.DevText_Printf(self.text, s);
                    h.DevText_SetCursorXY(self.text, 1, 1);
                    // The C prints the second reading (x was reassigned) on both lines.
                    h.DevText_Printf(self.text, &format!("PVoice  {:3}  {:3}\n", x, self.pvoice_peak));
                    h.DevText_SetCursorXY(self.text, 1, 2);
                    h.DevText_Printf(self.text, &format!("VVoice  {:3}  {:3}", x, self.vvoice_peak));
                    h.DevText_ShowBackground(self.text);
                    h.DevText_ShowText(self.text);
                }
                _ => {}
            }
        } else {
            self.pvoice_peak = h.lbAudioAx_80028B2C();
            self.pvoice_hold = 0;
            self.vvoice_peak = h.lbAudioAx_80028B4C();
            self.vvoice_hold = 0;
        }
    }

    /// `fn_CheckSoundInfo`.
    pub fn fn_CheckSoundInfo<H: DbSoundHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        let orig = self.sound_toggles;
        if player == 0 {
            if h.db_ButtonsPressed(player) & pad::X != 0 {
                if h.db_ButtonsDown(player) & pad::DPADLEFT != 0 {
                    self.sound_toggles = (self.sound_toggles + 1) % 4;
                    self.show_sound_info = (self.show_sound_info + 1) % 8;
                }
            } else if h.db_ButtonsPressed(player) & pad::DPADLEFT != 0 && h.db_ButtonsDown(player) & pad::X != 0 {
                self.sound_toggles = (self.sound_toggles + 1) % 4;
                self.show_sound_info = (self.show_sound_info + 1) % 8;
            }
            if self.sound_toggles != orig {
                let t = DB_SOUND_TOGGLE_ORDER[self.sound_toggles as usize];
                h.lbAudioAx_80025064(t & 1, t & 2);
            }
        }
        self.fn_UpdateSoundInfo(h);
    }
}
