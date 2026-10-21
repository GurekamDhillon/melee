#![allow(non_snake_case)]
use melee_runtime::tu::dbeffect::{pad, DbEffect, DbEffectHost};

#[derive(Default)]
struct H { down: u32, pressed: u32, hidden: bool, log: Vec<&'static str> }
impl DbEffectHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn ifAll_ShowHUD(&mut self) { self.hidden = false; self.log.push("show") }
    fn ifAll_IsHUDHidden(&mut self) -> bool { self.hidden }
    fn ifAll_HideHUD(&mut self) { self.hidden = true; self.log.push("hide") }
}

#[test]
fn cycles_0_to_3_and_toggles_hud() {
    let mut s = DbEffect::default();
    let mut h = H::default();
    s.fn_check_misc_visual_effects(&mut h, 0);
    assert_eq!(s.misc_visual_effects_status, 0);
    h.down = pad::X;
    h.pressed = pad::DPADDOWN;
    let mut seen = vec![];
    for _ in 0..5 {
        s.fn_check_misc_visual_effects(&mut h, 0);
        seen.push(s.misc_visual_effects_status);
    }
    assert_eq!(seen, [1, 2, 3, 0, 1]);
    assert_eq!(h.log, ["hide", "show", "hide"]);
}
