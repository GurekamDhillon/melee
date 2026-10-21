//! Port of src/melee/db/dbeffect.c (debug menu: X + D-down cycles the misc visual effects).
//!
//! The C globals live in [`DbEffect`]; calls out of the TU go through [`DbEffectHost`], whose
//! default implementation [`ext::Ext`] is the unported-callee stub set (panics, named like C).

/// HSD_PAD_* button bits (sysdolphin/baselib/controller.h), shared by the db TUs.
pub mod pad {
    pub const DPADLEFT: u32 = 1 << 0;
    pub const DPADRIGHT: u32 = 1 << 1;
    pub const DPADDOWN: u32 = 1 << 2;
    pub const DPADUP: u32 = 1 << 3;
    pub const R: u32 = 1 << 5;
    pub const L: u32 = 1 << 6;
    pub const A: u32 = 1 << 8;
    pub const B: u32 = 1 << 9;
    pub const X: u32 = 1 << 10;
    pub const Y: u32 = 1 << 11;
}

/// Callees of dbeffect.c.
pub trait DbEffectHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn ifAll_ShowHUD(&mut self);
    fn ifAll_IsHUDHidden(&mut self) -> bool;
    fn ifAll_HideHUD(&mut self);
}

#[allow(non_snake_case)]
pub mod ext {
    /// Unported callees (TODO: rewire to db.c / if/ifall.c ports).
    pub struct Ext;
    impl super::DbEffectHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown (db.c)") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed (db.c)") }
        fn ifAll_ShowHUD(&mut self) { todo!("ifAll_ShowHUD (if/ifall.c)") }
        fn ifAll_IsHUDHidden(&mut self) -> bool { todo!("ifAll_IsHUDHidden (if/ifall.c)") }
        fn ifAll_HideHUD(&mut self) { todo!("ifAll_HideHUD (if/ifall.c)") }
    }
}

/// `db_MiscVisualEffectsStatus` (0x804D6B80).
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbEffect {
    pub misc_visual_effects_status: i32,
}

impl DbEffect {
    /// `fn_CheckMiscVisualEffects` (0x8022873C).
    pub fn fn_check_misc_visual_effects<H: DbEffectHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if (h.db_ButtonsDown(player) & pad::X) != 0 && (h.db_ButtonsPressed(player) & pad::DPADDOWN) != 0 {
            self.misc_visual_effects_status = self.misc_visual_effects_status.wrapping_add(1);
            if self.misc_visual_effects_status > 3 {
                self.misc_visual_effects_status = 0;
            }
            if self.misc_visual_effects_status == 0 {
                h.ifAll_ShowHUD();
            } else if !h.ifAll_IsHUDHidden() {
                h.ifAll_HideHUD();
            }
        }
    }
}
