//! Port of src/melee/db/dbanim.c (debug: fighter animation info overlay, collision-bubble and
//! misc fighter visual toggles, mushroom shortcuts).
#![allow(non_snake_case)]

use super::dbcpu::{show_overlay, DevTextHost, DevTextId, BG_CLEAR, FG_WHITE};
use super::dbeffect::pad;

/// Fighter gobj handle (`Fighter_GObj*`).
pub type FighterGObj = u32;

/// The Fighter fields fn_UpdateAnimationInfo prints.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FtAnimView {
    pub player_id: i32,
    pub motion_id: i32,
    pub anim_id: i32,
    pub cur_anim_frame: f32,
    pub x221c_u16_y: u16,
}

pub trait DbAnimHost: DevTextHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    /// `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]` walked through `->next`.
    fn fighter_gobjs(&mut self) -> Vec<FighterGObj>;
    /// `GET_FIGHTER(gobj)->x21FC_flag.byte`
    fn ft_x21FC(&mut self, g: FighterGObj) -> u8;
    fn ft_set_x21FC(&mut self, g: FighterGObj, v: u8);
    fn ft_anim_view(&mut self, g: FighterGObj) -> FtAnimView;
    fn db_motionstate_names(&mut self, i: i32) -> String;
    fn db_submotion_names(&mut self, i: i32) -> String;
    fn ftLib_IsFighter(&mut self, g: FighterGObj) -> i32;
    fn fn_EnableShowEnemyStompRange(&mut self);
    fn fn_DisableShowEnemyStompRange(&mut self);
    fn fn_EnableShowItemPickupRange(&mut self);
    fn fn_DisableShowItemPickupRange(&mut self);
    fn fn_EnableShowCoinPickupRange(&mut self);
    fn fn_DisableShowCoinPickupRange(&mut self);
    fn Player_GetPlayerSlotType(&mut self, player: i32) -> i32;
    fn Player_GetEntity(&mut self, player: i32) -> FighterGObj;
    fn fn_80225E6C(&mut self, g: FighterGObj);
    fn Fighter_PoisonMushroomApply(&mut self, g: FighterGObj);
    fn Fighter_SuperMushroomApply(&mut self, g: FighterGObj);
    fn Fighter_PoisonMushroomEnd(&mut self, g: FighterGObj);
    fn Fighter_SuperMushroomEnd(&mut self, g: FighterGObj);
}

pub mod ext {
    use super::*;
    use crate::tu::dbcpu::Rgba;
    /// Unported callees (TODO: rewire to db.c, ft, pl, if/textlib.c ports).
    pub struct Ext;
    impl DbAnimHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed") }
        fn fighter_gobjs(&mut self) -> Vec<FighterGObj> { todo!("HSD_GObjPLinkHead[FIGHTER]") }
        fn ft_x21FC(&mut self, _: FighterGObj) -> u8 { todo!("Fighter x21FC") }
        fn ft_set_x21FC(&mut self, _: FighterGObj, _: u8) { todo!("Fighter x21FC") }
        fn ft_anim_view(&mut self, _: FighterGObj) -> FtAnimView { todo!("Fighter fields") }
        fn db_motionstate_names(&mut self, _: i32) -> String { todo!("db_motionstate_names") }
        fn db_submotion_names(&mut self, _: i32) -> String { todo!("db_submotion_names") }
        fn ftLib_IsFighter(&mut self, _: FighterGObj) -> i32 { todo!("ftLib_IsFighter") }
        fn fn_EnableShowEnemyStompRange(&mut self) { todo!("fn_EnableShowEnemyStompRange") }
        fn fn_DisableShowEnemyStompRange(&mut self) { todo!("fn_DisableShowEnemyStompRange") }
        fn fn_EnableShowItemPickupRange(&mut self) { todo!("fn_EnableShowItemPickupRange") }
        fn fn_DisableShowItemPickupRange(&mut self) { todo!("fn_DisableShowItemPickupRange") }
        fn fn_EnableShowCoinPickupRange(&mut self) { todo!("fn_EnableShowCoinPickupRange") }
        fn fn_DisableShowCoinPickupRange(&mut self) { todo!("fn_DisableShowCoinPickupRange") }
        fn Player_GetPlayerSlotType(&mut self, _: i32) -> i32 { todo!("Player_GetPlayerSlotType") }
        fn Player_GetEntity(&mut self, _: i32) -> FighterGObj { todo!("Player_GetEntity") }
        fn fn_80225E6C(&mut self, _: FighterGObj) { todo!("fn_80225E6C") }
        fn Fighter_PoisonMushroomApply(&mut self, _: FighterGObj) { todo!("Fighter_PoisonMushroomApply") }
        fn Fighter_SuperMushroomApply(&mut self, _: FighterGObj) { todo!("Fighter_SuperMushroomApply") }
        fn Fighter_PoisonMushroomEnd(&mut self, _: FighterGObj) { todo!("Fighter_PoisonMushroomEnd") }
        fn Fighter_SuperMushroomEnd(&mut self, _: FighterGObj) { todo!("Fighter_SuperMushroomEnd") }
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

/// `db_AnimationInfo` (text + 0x5A4 buffer) and the `db_804D6B48` bitfield word
/// (ShowFighterCollisionBubbles:3, MiscFighterVisualsStatus:6, ShowAnimationInfo:1).
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbAnim {
    pub text: DevTextId,
    /// 3-bit field.
    pub show_fighter_collision_bubbles: u8,
    /// 6-bit field.
    pub misc_fighter_visuals_status: u8,
    /// 1-bit field.
    pub show_animation_info: bool,
}

/// `"%03.2f"`.
pub fn format_frame(f: f32) -> String {
    melee_core::ct!("format_frame");
    format!("{:03.2}", f as f64)
}

impl DbAnim {
    /// `fn_SetupAnimationInfo` (0x802267C8).
    pub fn fn_setup_animation_info<H: DbAnimHost + ?Sized>(&mut self, h: &mut H) {
        let gobj = h.DevText_GetGObj();
        self.show_fighter_collision_bubbles = 1;
        self.show_animation_info = false;
        self.misc_fighter_visuals_status = 0;
        self.text = h.DevText_Create(7, 20, 20, 60, 12, 0x5A4);
        if self.text.is_some() {
            show_overlay(h, gobj, self.text, BG_CLEAR, FG_WHITE, 9.0, 12.0);
        }
    }

    /// `fn_ToggleMiscFighterVisuals` (0x802268B8).
    pub fn fn_toggle_misc_fighter_visuals<H: DbAnimHost + ?Sized>(&mut self, h: &mut H) {
        if self.misc_fighter_visuals_status != 0 {
            self.misc_fighter_visuals_status = (self.misc_fighter_visuals_status << 1) & 0x3F;
        } else {
            self.misc_fighter_visuals_status = 1;
        }
        let st = self.misc_fighter_visuals_status as u32;
        for g in h.fighter_gobjs() {
            let b = h.ft_x21FC(g) as u32;
            h.ft_set_x21FC(g, ((b & 3) | ((st << 2) & !3)) as u8);
        }
        if st & 0x02 != 0 { h.fn_EnableShowEnemyStompRange() } else { h.fn_DisableShowEnemyStompRange() }
        if st & 0x08 != 0 { h.fn_EnableShowItemPickupRange() } else { h.fn_DisableShowItemPickupRange() }
        if st & 0x20 != 0 { h.fn_EnableShowCoinPickupRange() } else { h.fn_DisableShowCoinPickupRange() }
    }

    /// `fn_UpdateAnimationInfo` (0x802269C0).
    pub fn fn_update_animation_info<H: DbAnimHost + ?Sized>(&mut self, h: &mut H) {
        if !self.show_animation_info {
            return;
        }
        let t = self.text;
        let mut print_newline = false;
        h.DevText_Erase(t);
        h.DevText_SetCursorXY(t, 0, 0);
        for g in h.fighter_gobjs() {
            let ft = h.ft_anim_view(g);
            if !print_newline {
                print_newline = true;
            } else {
                h.DevText_Printf(t, "\n");
            }
            if ft.motion_id < 0x155 {
                let n = h.db_motionstate_names(ft.motion_id);
                h.DevText_Printf(t, &format!("{} {}", ft.player_id, n));
            } else {
                h.DevText_Printf(t, &format!("{} {}", ft.player_id, ft.motion_id));
            }
            h.DevText_SetCursorX(t, 23);
            if ft.anim_id != -1 {
                if ft.anim_id < 0x127 {
                    let n = h.db_submotion_names(ft.anim_id);
                    h.DevText_Printf(t, &n);
                } else {
                    h.DevText_Printf(t, &ft.anim_id.to_string());
                }
            }
            h.DevText_SetCursorX(t, 44);
            h.DevText_Printf(t, &format_frame(ft.cur_anim_frame));
            let y = ft.x221c_u16_y;
            if y != 0 {
                h.DevText_SetCursorX(t, 52);
                h.DevText_Printf(t, if y & 1 != 0 { "L" } else { " " });
                h.DevText_Printf(t, if y & 2 != 0 { "R" } else { " " });
                h.DevText_Printf(t, if y & 4 != 0 { "T" } else { " " });
            }
        }
    }

    /// `fn_CheckAnimationInfo` (0x80226BD4).
    pub fn fn_check_animation_info<H: DbAnimHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if h.db_ButtonsDown(player) & pad::R != 0 {
            if h.db_ButtonsPressed(player) & pad::DPADUP != 0 {
                self.show_fighter_collision_bubbles = (self.show_fighter_collision_bubbles + 1) & 7;
                if self.show_fighter_collision_bubbles > 3 {
                    self.show_fighter_collision_bubbles = 1;
                }
                for g in h.fighter_gobjs() {
                    h.ft_set_x21FC(g, self.show_fighter_collision_bubbles);
                }
            }
            if h.db_ButtonsPressed(player) & pad::DPADRIGHT != 0 && h.Player_GetPlayerSlotType(player) != 3 {
                let g = h.Player_GetEntity(player);
                let x = (h.ft_x21FC(g) as u32 & 3) + 1;
                h.ft_set_x21FC(g, if x > 3 { 1 } else { x as u8 });
                h.fn_80225E6C(g);
            }
            if h.db_ButtonsPressed(player) & pad::DPADLEFT != 0 {
                self.fn_toggle_misc_fighter_visuals(h);
            }
        }
        if h.db_ButtonsDown(player) & pad::Y != 0 && h.db_ButtonsPressed(player) & pad::DPADDOWN != 0 {
            self.show_animation_info = !self.show_animation_info;
            if !self.show_animation_info {
                h.DevText_HideBackground(self.text);
                h.DevText_HideText(self.text);
            } else {
                h.DevText_ShowBackground(self.text);
                h.DevText_ShowText(self.text);
            }
        }
        if h.db_ButtonsDown(player) & pad::Y != 0 {
            if h.db_ButtonsPressed(player) & pad::DPADLEFT != 0 && h.Player_GetPlayerSlotType(player) != 3 {
                let g = h.Player_GetEntity(player);
                if h.db_ButtonsDown(player) & pad::A != 0 {
                    h.Fighter_PoisonMushroomApply(g);
                } else {
                    h.Fighter_SuperMushroomApply(g);
                }
            }
            if h.db_ButtonsPressed(player) & pad::DPADRIGHT != 0 && h.Player_GetPlayerSlotType(player) != 3 {
                let g = h.Player_GetEntity(player);
                if h.db_ButtonsDown(player) & pad::A != 0 {
                    h.Fighter_PoisonMushroomEnd(g);
                } else {
                    h.Fighter_SuperMushroomEnd(g);
                }
            }
        }
    }
}

/// `fn_8022697C`: the owner's x21FC flag byte, or 0 when ftLib_IsFighter rejects it.
pub fn fn_8022697c<H: DbAnimHost + ?Sized>(h: &mut H, owner: FighterGObj) -> u8 {
    melee_core::ct!("fn_8022697c");
    if h.ftLib_IsFighter(owner) != 0 {
        h.ft_x21FC(owner)
    } else {
        0
    }
}
