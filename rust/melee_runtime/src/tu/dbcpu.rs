//! Port of src/melee/db/dbcpu.c (debug: CPU handicap info overlay, B + D-down).
//! Also holds the DevText host trait shared by the db overlay TUs.
#![allow(non_snake_case)]

use super::dbeffect::pad;

/// Opaque DevText handle (`DevText*`); `None` is NULL.
pub type DevTextId = Option<u32>;

/// GXColor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

/// The StaticPlayer fields fn_UpdateCpuHandicapInfo prints.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CpuHandicapRow {
    pub player_state: i32,
    pub cpu_level: u8,
    pub cpu_type: u8,
    pub handicap: u8,
    pub unk50: f32,
    pub attack_ratio: f32,
    pub defense_ratio: f32,
}

/// DevText calls (if/textlib.c, not ported).
#[allow(non_snake_case)]
pub trait DevTextHost {
    fn DevText_GetGObj(&mut self) -> Option<u32>;
    /// `DevText_Create(id, x, y, w, h, buf)`; the buffer (`buf_len` bytes) is owned by the host.
    fn DevText_Create(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, buf_len: usize) -> DevTextId;
    fn DevText_Show(&mut self, gobj: Option<u32>, t: DevTextId);
    fn DevText_HideCursor(&mut self, t: DevTextId);
    fn DevText_SetBGColor(&mut self, t: DevTextId, c: Rgba);
    fn DevText_SetTextColor(&mut self, t: DevTextId, c: Rgba);
    fn DevText_SetScale(&mut self, t: DevTextId, x: f32, y: f32);
    fn DevText_Erase(&mut self, t: DevTextId);
    fn DevText_SetCursorXY(&mut self, t: DevTextId, x: i32, y: i32);
    fn DevText_SetCursorX(&mut self, t: DevTextId, x: i32);
    /// `DevText_Printf`, already formatted.
    fn DevText_Printf(&mut self, t: DevTextId, s: &str);
    fn DevText_ShowBackground(&mut self, t: DevTextId);
    fn DevText_HideBackground(&mut self, t: DevTextId);
    fn DevText_ShowText(&mut self, t: DevTextId);
    fn DevText_HideText(&mut self, t: DevTextId);
}

#[allow(non_snake_case)]
pub trait DbCpuHost: DevTextHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn Player_GetPtrForSlot(&mut self, slot: i32) -> CpuHandicapRow;
}

#[allow(non_snake_case)]
pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire to if/textlib.c, db.c, pl/player.c ports).
    pub struct Ext;
    impl DevTextHost for Ext {
        fn DevText_GetGObj(&mut self) -> Option<u32> { todo!("DevText_GetGObj") }
        fn DevText_Create(&mut self, _: i32, _: i32, _: i32, _: i32, _: i32, _: usize) -> DevTextId { todo!("DevText_Create") }
        fn DevText_Show(&mut self, _: Option<u32>, _: DevTextId) { todo!("DevText_Show") }
        fn DevText_HideCursor(&mut self, _: DevTextId) { todo!("DevText_HideCursor") }
        fn DevText_SetBGColor(&mut self, _: DevTextId, _: Rgba) { todo!("DevText_SetBGColor") }
        fn DevText_SetTextColor(&mut self, _: DevTextId, _: Rgba) { todo!("DevText_SetTextColor") }
        fn DevText_SetScale(&mut self, _: DevTextId, _: f32, _: f32) { todo!("DevText_SetScale") }
        fn DevText_Erase(&mut self, _: DevTextId) { todo!("DevText_Erase") }
        fn DevText_SetCursorXY(&mut self, _: DevTextId, _: i32, _: i32) { todo!("DevText_SetCursorXY") }
        fn DevText_SetCursorX(&mut self, _: DevTextId, _: i32) { todo!("DevText_SetCursorX") }
        fn DevText_Printf(&mut self, _: DevTextId, _: &str) { todo!("DevText_Printf") }
        fn DevText_ShowBackground(&mut self, _: DevTextId) { todo!("DevText_ShowBackground") }
        fn DevText_HideBackground(&mut self, _: DevTextId) { todo!("DevText_HideBackground") }
        fn DevText_ShowText(&mut self, _: DevTextId) { todo!("DevText_ShowText") }
        fn DevText_HideText(&mut self, _: DevTextId) { todo!("DevText_HideText") }
    }
    impl DbCpuHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed") }
        fn Player_GetPtrForSlot(&mut self, _: i32) -> CpuHandicapRow { todo!("Player_GetPtrForSlot") }
    }
}

pub const BG_CLEAR: Rgba = Rgba(0, 0, 0, 0);
pub const FG_WHITE: Rgba = Rgba(0xFF, 0xFF, 0xFF, 0xFF);

/// `db_CpuHandicapInfo` (text + 0x34C buffer) and `db_ShowCpuHandicapInfo.b0`.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbCpu {
    pub text: DevTextId,
    pub show: bool,
}

/// One row, as `"\n%d %d %2d %d %2.2f %2.2f %2.2f"` prints it (floats promoted to double).
pub fn format_cpu_row(p: &CpuHandicapRow) -> String {
    format!(
        "\n{} {} {:2} {} {:2.2} {:2.2} {:2.2}",
        p.player_state,
        p.cpu_level,
        p.cpu_type,
        p.handicap,
        p.unk50 as f64,
        p.attack_ratio as f64,
        p.defense_ratio as f64
    )
}

/// The Setup sequence every db overlay uses after DevText_Create.
pub(crate) fn show_overlay<H: DevTextHost + ?Sized>(h: &mut H, gobj: Option<u32>, t: DevTextId, bg: Rgba, fg: Rgba, sx: f32, sy: f32) {
    h.DevText_Show(gobj, t);
    h.DevText_HideCursor(t);
    h.DevText_SetBGColor(t, bg);
    h.DevText_SetTextColor(t, fg);
    h.DevText_SetScale(t, sx, sy);
}

impl DbCpu {
    /// `fn_SetupCpuHandicapInfo` (0x8022659C).
    pub fn fn_setup_cpu_handicap_info<H: DbCpuHost + ?Sized>(&mut self, h: &mut H) {
        let gobj = h.DevText_GetGObj();
        self.show = false;
        self.text = h.DevText_Create(6, 20, 20, 60, 7, 0x34C);
        if self.text.is_some() {
            show_overlay(h, gobj, self.text, BG_CLEAR, FG_WHITE, 9.0, 12.0);
        }
    }

    /// `fn_UpdateCpuHandicapInfo` (0x8022666C).
    pub fn fn_update_cpu_handicap_info<H: DbCpuHost + ?Sized>(&mut self, h: &mut H) {
        if self.show {
            let t = self.text;
            h.DevText_Erase(t);
            h.DevText_SetCursorXY(t, 0, 0);
            h.DevText_Printf(t, "A B  C D E    F    G");
            for slot in 0..6 {
                let p = h.Player_GetPtrForSlot(slot);
                h.DevText_Printf(t, &format_cpu_row(&p));
            }
        }
    }

    /// `fn_CheckCpuHandicapInfo` (0x80226730).
    pub fn fn_check_cpu_handicap_info<H: DbCpuHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if (h.db_ButtonsDown(player) & pad::B) != 0 && (h.db_ButtonsPressed(player) & pad::DPADDOWN) != 0 {
            self.show = !self.show;
            if !self.show {
                h.DevText_HideBackground(self.text);
                h.DevText_HideText(self.text);
                return;
            }
            h.DevText_ShowBackground(self.text);
            h.DevText_ShowText(self.text);
        }
    }
}
