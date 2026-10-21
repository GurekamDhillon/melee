#![allow(non_snake_case)]
mod common_devtext;
use common_devtext::Dt;
use melee_runtime::tu::dbcpu::*;
use melee_runtime::tu::dbeffect::pad;

#[derive(Default)]
struct H { dt: Dt, down: u32, pressed: u32 }
forward_devtext!(H);
impl DbCpuHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn Player_GetPtrForSlot(&mut self, s: i32) -> CpuHandicapRow {
        CpuHandicapRow { player_state: s, cpu_level: 9, cpu_type: 4, handicap: 5, unk50: 1.0, attack_ratio: 0.5, defense_ratio: 2.125 }
    }
}

#[test]
fn setup_toggle_update() {
    let mut s = DbCpu::default();
    let mut h = H::default();
    s.fn_setup_cpu_handicap_info(&mut h);
    assert_eq!(h.dt.log[0], "create 6 20 20 60 7 0x34c");
    assert_eq!(h.dt.log.last().unwrap(), "scale 9 12");
    h.dt.log.clear();
    s.fn_update_cpu_handicap_info(&mut h);
    assert!(h.dt.log.is_empty());
    h.down = pad::B;
    h.pressed = pad::DPADDOWN;
    s.fn_check_cpu_handicap_info(&mut h, 0);
    assert!(s.show);
    assert_eq!(h.dt.log, ["showbg", "showtext"]);
    h.dt.log.clear();
    s.fn_update_cpu_handicap_info(&mut h);
    assert_eq!(h.dt.log.len(), 3 + 6);
    assert_eq!(h.dt.log[3], "p \n0 9  4 5 1.00 0.50 2.12");
    s.fn_check_cpu_handicap_info(&mut h, 0);
    assert!(!s.show);
}
