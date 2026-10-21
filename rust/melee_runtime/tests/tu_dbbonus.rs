#![allow(non_snake_case)]
mod common_devtext;
use common_devtext::Dt;
use melee_runtime::tu::dbbonus::*;
use melee_runtime::tu::dbeffect::pad;

#[derive(Default)]
struct H { dt: Dt, down: u32, pressed: u32, refreshed: u32 }
forward_devtext!(H);
impl DbBonusHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn gm_GetMatchEndPlayerScore(&mut self, s: i32) -> i32 { s * 1000 }
    // Every even bonus is earned: 108 rows, more than the two 30-row screens hold.
    fn pl_80039418(&mut self, _: i32, b: i32) -> i32 { if b % 2 == 0 { b + 1 } else { 0 } }
    fn pl_80039450(&mut self, _: i32) { self.refreshed += 1 }
    fn gmDecisionGetType(&mut self, b: i32) -> i32 { (b % 4 == 0) as i32 }
    fn db_bonus_names(&mut self, b: i32) -> String { format!("B{b}") }
    fn Player_GetEntity(&mut self, _: i32) -> Option<u32> { Some(1) }
}

#[test]
fn toggle_states_and_screen_spill() {
    let mut s = DbBonus::default();
    let mut h = H::default();
    s.fn_setup_bonus_info(&mut h);
    assert_eq!(s.x0, 0xFF);
    assert_eq!(h.dt.log[0], "create 10 0 0 25 30 0x5dc");
    h.dt.log.clear();
    s.fn_check_bonus_info(&mut h, 1);
    assert!(h.dt.log.is_empty());
    h.down = pad::B;
    h.pressed = pad::DPADLEFT;
    s.fn_check_bonus_info(&mut h, 1);
    assert_eq!((s.x0, s.x1_b0, h.refreshed), (1, false, 1));
    let p: Vec<&String> = h.dt.log.iter().filter(|l| l.starts_with("p ")).collect();
    assert_eq!(p[0], "p 1P       0 2P    1000\n");
    assert_eq!(p[2], "p 2P screen 0");
    assert_eq!(p[3], "p B0 : 1");
    assert_eq!(p[4], "p B2");
    // 27 rows on screen 0, 30 on screen 1, then the overflow notice.
    assert_eq!(p.len(), 3 + 27 + 30 + 1);
    assert_eq!(p.last().unwrap().as_str(), "p screen over!!");
    s.fn_check_bonus_info(&mut h, 1);
    assert_eq!((s.x0, s.x1_b0), (1, true));
    h.dt.log.clear();
    s.fn_check_bonus_info(&mut h, 1);
    assert_eq!((s.x0, s.x1_b0), (0xFF, false));
    assert_eq!(h.dt.log, ["hidebg", "hidetext", "hidebg", "hidetext"]);
}
