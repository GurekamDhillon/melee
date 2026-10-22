//! tu::ftpeachattacks4 smoke test through recording hooks.
#![allow(static_mut_refs)]
use melee_chara::stubs as st;
use melee_types::*;
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn mk() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    (f, g)
}

use melee_chara::tu::ftpeachattacks4 as t;
#[test]
fn smoke() {
    let (mut f, mut g) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, _, _, _, _, _| log(format!("ms {ms}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTCO_WAIT_IASA = Some(|_| log("wait".into()));
        t::ext::HSD_RANDI = Some(|max| {
            assert_eq!(max, 3);
            1
        });
    }
    f.u.pe.attacks4_motion_id = ftPe_MS_AttackS4Club as _;
    t::ftPe_AttackS4_Enter(gp);
    let want = ftPe_MS_AttackS4Club as i32 + 1;
    assert_eq!(unsafe { f.u.pe.attacks4_motion_id } as i32, want);
    t::ftPe_AttackS4_IASA(gp);
    f.set_allow_interrupt(1);
    t::ftPe_AttackS4_IASA(gp);
    assert_eq!(unsafe { LOG.join(",") }, format!("ms {want},wait"));
}
