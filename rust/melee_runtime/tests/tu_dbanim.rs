#![allow(non_snake_case)]
mod common_devtext;
use common_devtext::Dt;
use melee_runtime::tu::dbanim::*;
use melee_runtime::tu::dbeffect::pad;

#[derive(Default)]
struct H { dt: Dt, down: u32, pressed: u32, flags: [u8; 2], log: Vec<String> }
forward_devtext!(H);
impl DbAnimHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn fighter_gobjs(&mut self) -> Vec<FighterGObj> { vec![0, 1] }
    fn ft_x21FC(&mut self, g: FighterGObj) -> u8 { self.flags[g as usize] }
    fn ft_set_x21FC(&mut self, g: FighterGObj, v: u8) { self.flags[g as usize] = v }
    fn ft_anim_view(&mut self, g: FighterGObj) -> FtAnimView {
        if g == 0 {
            FtAnimView { player_id: 0, motion_id: 14, anim_id: 3, cur_anim_frame: 4.5, x221c_u16_y: 5 }
        } else {
            FtAnimView { player_id: 1, motion_id: 0x200, anim_id: 0x130, cur_anim_frame: -1.0, x221c_u16_y: 0 }
        }
    }
    fn db_motionstate_names(&mut self, i: i32) -> String { format!("MS{i}") }
    fn db_submotion_names(&mut self, i: i32) -> String { format!("SM{i}") }
    fn ftLib_IsFighter(&mut self, g: FighterGObj) -> i32 { (g == 0) as i32 }
    fn fn_EnableShowEnemyStompRange(&mut self) { self.log.push("stomp+".into()) }
    fn fn_DisableShowEnemyStompRange(&mut self) { self.log.push("stomp-".into()) }
    fn fn_EnableShowItemPickupRange(&mut self) { self.log.push("item+".into()) }
    fn fn_DisableShowItemPickupRange(&mut self) { self.log.push("item-".into()) }
    fn fn_EnableShowCoinPickupRange(&mut self) { self.log.push("coin+".into()) }
    fn fn_DisableShowCoinPickupRange(&mut self) { self.log.push("coin-".into()) }
    fn Player_GetPlayerSlotType(&mut self, _: i32) -> i32 { 0 }
    fn Player_GetEntity(&mut self, p: i32) -> FighterGObj { p as u32 }
    fn fn_80225E6C(&mut self, _: FighterGObj) { self.log.push("e6c".into()) }
    fn Fighter_PoisonMushroomApply(&mut self, _: FighterGObj) { self.log.push("poison".into()) }
    fn Fighter_SuperMushroomApply(&mut self, _: FighterGObj) { self.log.push("super".into()) }
    fn Fighter_PoisonMushroomEnd(&mut self, _: FighterGObj) { self.log.push("poison-end".into()) }
    fn Fighter_SuperMushroomEnd(&mut self, _: FighterGObj) { self.log.push("super-end".into()) }
}

#[test]
fn misc_visuals_shift_through_bits() {
    let mut s = DbAnim::default();
    let mut h = H::default();
    s.fn_setup_animation_info(&mut h);
    assert_eq!(s.show_fighter_collision_bubbles, 1);
    h.flags = [3, 1];
    let mut seen = vec![];
    for _ in 0..7 {
        s.fn_toggle_misc_fighter_visuals(&mut h);
        seen.push(s.misc_fighter_visuals_status);
    }
    assert_eq!(seen, [1, 2, 4, 8, 16, 32, 0]);
    assert_eq!(h.flags, [3, 1]); // status 0 -> only the low two bits survive
    s.fn_toggle_misc_fighter_visuals(&mut h);
    assert_eq!(h.flags, [3 | 4, 1 | 4]);
    assert_eq!(fn_8022697c(&mut h, 0), 7);
    assert_eq!(fn_8022697c(&mut h, 1), 0);
}

#[test]
fn bubbles_cycle_and_overlay_text() {
    let mut s = DbAnim::default();
    let mut h = H::default();
    s.fn_setup_animation_info(&mut h);
    h.down = pad::R;
    h.pressed = pad::DPADUP;
    for want in [2, 3, 1] {
        s.fn_check_animation_info(&mut h, 0);
        assert_eq!(h.flags, [want, want]);
    }
    h.pressed = pad::DPADRIGHT;
    h.flags[1] = 3;
    s.fn_check_animation_info(&mut h, 1);
    assert_eq!(h.flags[1], 1);
    h.down = pad::Y | pad::A;
    h.pressed = pad::DPADDOWN | pad::DPADLEFT;
    s.fn_check_animation_info(&mut h, 0);
    assert!(s.show_animation_info);
    assert_eq!(h.log.last().unwrap(), "poison");
    h.dt.log.clear();
    s.fn_update_animation_info(&mut h);
    let p: Vec<&str> = h.dt.log.iter().filter(|l| l.starts_with("p ")).map(|l| &l[2..]).collect();
    assert_eq!(p, ["0 MS14", "SM3", "4.50", "L", " ", "T", "\n", "1 512", "304", "-1.00"]);
}
