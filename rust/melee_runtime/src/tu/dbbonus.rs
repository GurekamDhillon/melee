//! Port of src/melee/db/dbbonus.c (debug: per-player match bonus list on two DevText screens).
#![allow(non_snake_case)]

use super::dbcpu::{DevTextHost, DevTextId, Rgba};
use super::dbeffect::pad;

/// `db_TextColors[2]` (bg, fg).
pub const DB_TEXT_COLORS: [(Rgba, Rgba); 2] = [
    (Rgba(0x00, 0x00, 0x00, 0x00), Rgba(0xFF, 0x00, 0x00, 0xFF)),
    (Rgba(0x80, 0x40, 0xFF, 0x80), Rgba(0x00, 0x00, 0x00, 0xFF)),
];

/// Number of bonus kinds scanned (`bonus < 0xD7`).
pub const BONUS_COUNT: i32 = 0xD7;

pub trait DbBonusHost: DevTextHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn gm_GetMatchEndPlayerScore(&mut self, slot: i32) -> i32;
    fn pl_80039418(&mut self, player: i32, bonus: i32) -> i32;
    fn pl_80039450(&mut self, player: i32);
    fn gmDecisionGetType(&mut self, bonus: i32) -> i32;
    fn db_bonus_names(&mut self, bonus: i32) -> String;
    /// `Player_GetEntity(player) != NULL`
    fn Player_GetEntity(&mut self, player: i32) -> Option<u32>;
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire to db.c, gm, pl/plbonus.c, pl/player.c, if/textlib.c).
    pub struct Ext;
    impl DbBonusHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed") }
        fn gm_GetMatchEndPlayerScore(&mut self, _: i32) -> i32 { todo!("gm_GetMatchEndPlayerScore") }
        fn pl_80039418(&mut self, _: i32, _: i32) -> i32 { todo!("pl_80039418") }
        fn pl_80039450(&mut self, _: i32) { todo!("pl_80039450") }
        fn gmDecisionGetType(&mut self, _: i32) -> i32 { todo!("gmDecisionGetType") }
        fn db_bonus_names(&mut self, _: i32) -> String { todo!("db_bonus_names") }
        fn Player_GetEntity(&mut self, _: i32) -> Option<u32> { todo!("Player_GetEntity") }
    }
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
}

/// `db_804D6B98` ({u8 x0; UnkFlagStruct x1}) and `db_804D6B9C` (2 x {DevText*, char[1500]},
/// HSD_MemAlloc'd; the buffers are host-owned here).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DbBonus {
    pub x0: u8,
    pub x1_b0: bool,
    pub texts: [DevTextId; 2],
}

impl DbBonus {
    fn colors(&self) -> (Rgba, Rgba) {
        DB_TEXT_COLORS[self.x1_b0 as usize]
    }

    /// `fn_SetupBonusInfo` (0x80228CF4).
    pub fn fn_setup_bonus_info<H: DbBonusHost + ?Sized>(&mut self, h: &mut H) {
        self.fn_80228d18();
        self.fn_80228d38(h);
    }

    /// `fn_80228D18`.
    pub fn fn_80228d18(&mut self) {
        self.x0 = 0xFF;
        self.x1_b0 = false;
    }

    /// `fn_80228D38`.
    pub fn fn_80228d38<H: DbBonusHost + ?Sized>(&mut self, h: &mut H) {
        let gobj = h.DevText_GetGObj();
        for i in 0..2 {
            let t = h.DevText_Create(i as i32 + 10, i as i32 * 300, 0, 25, 30, 1500);
            self.texts[i] = t;
            if t.is_some() {
                let (bg, fg) = self.colors();
                h.DevText_Show(gobj, t);
                h.DevText_HideCursor(t);
                h.DevText_SetBGColor(t, bg);
                h.DevText_SetTextColor(t, fg);
                h.DevText_SetScale(t, 12.0, 16.0);
            }
        }
    }

    /// `fn_80228E54`: print `player`'s bonuses from `start` on screen `screen`, spilling to the
    /// next screen when 30 rows are used.
    pub fn fn_80228e54<H: DbBonusHost + ?Sized>(&mut self, h: &mut H, player: i32, screen: i32, start: i32) {
        let mut y_pos = 0;
        let t = self.texts[screen as usize];
        h.DevText_Erase(t);
        h.DevText_SetCursorXY(t, 0, 0);
        if screen == 0 {
            let (a, b) = (h.gm_GetMatchEndPlayerScore(0), h.gm_GetMatchEndPlayerScore(1));
            h.DevText_Printf(t, &format!("1P {:7} 2P {:7}\n", a, b));
            let (a, b) = (h.gm_GetMatchEndPlayerScore(2), h.gm_GetMatchEndPlayerScore(3));
            h.DevText_Printf(t, &format!("3P {:7} 4P {:7}\n", a, b));
            h.DevText_Printf(t, &format!("{}P screen {}", player + 1, screen));
            y_pos = 3;
        }
        h.DevText_SetCursorXY(t, 0, y_pos);
        for bonus in start..BONUS_COUNT {
            if h.pl_80039418(player, bonus) != 0 {
                if y_pos >= 0x1E {
                    let next = screen + 1;
                    if next < 2 {
                        self.fn_80228e54(h, player, next, bonus);
                        return;
                    }
                    h.DevText_Printf(t, "screen over!!");
                    return;
                }
                let name = h.db_bonus_names(bonus);
                if h.gmDecisionGetType(bonus) == 0 {
                    h.DevText_Printf(t, &name);
                } else {
                    let n = h.pl_80039418(player, bonus);
                    h.DevText_Printf(t, &format!("{} : {}", name, n));
                }
                y_pos += 1;
                h.DevText_SetCursorXY(t, 0, y_pos);
            }
        }
    }

    /// `fn_8022900C`: B + D-left toggles: show for `player`, then invert colours, then hide.
    pub fn fn_8022900c<H: DbBonusHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if self.x0 as i32 == player {
            if self.x1_b0 {
                self.x0 = 0xFF;
                self.x1_b0 = false;
                for i in 0..2 {
                    h.DevText_HideBackground(self.texts[i]);
                    h.DevText_HideText(self.texts[i]);
                }
            } else {
                self.x1_b0 = true;
                for i in 0..2 {
                    let (bg, fg) = self.colors();
                    h.DevText_SetBGColor(self.texts[i], bg);
                    h.DevText_SetTextColor(self.texts[i], fg);
                }
            }
        } else {
            self.x0 = player as u8;
            for i in 0..2 {
                let (bg, fg) = self.colors();
                h.DevText_ShowBackground(self.texts[i]);
                h.DevText_ShowText(self.texts[i]);
                h.DevText_SetBGColor(self.texts[i], bg);
                h.DevText_SetTextColor(self.texts[i], fg);
            }
        }
    }

    /// `fn_CheckBonusInfo` (0x802291A0).
    pub fn fn_check_bonus_info<H: DbBonusHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if h.db_ButtonsDown(player) & pad::B != 0 && h.db_ButtonsPressed(player) & pad::DPADLEFT != 0 {
            self.fn_8022900c(h, player);
        }
        if self.x0 as i32 == player {
            if h.Player_GetEntity(player).is_some() {
                h.pl_80039450(player);
            }
            self.fn_80228e54(h, player, 0, 0);
        }
    }
}
