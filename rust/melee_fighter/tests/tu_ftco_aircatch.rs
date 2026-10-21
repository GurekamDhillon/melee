//! tu::ftco_aircatch smoke tests (stateful: the callees are `ext` stubs with override slots).

use melee_fighter::tu::ftco_aircatch::{self as ac, ext};
use melee_fighter::types::*;

fn mk() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut f = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = (&mut *f as *mut Fighter).cast();
    (f, go)
}

#[test]
fn tether_check_and_enter() {
    unsafe {
        ext::SCRIPTGAME_CAPS = Some(|_, _| true);
        ext::MEX_HAS_ZAIR = Some(|_| false);
        ext::CHANGE_MOTION = Some(|_, ms, _, _, _, _, _| assert_eq!(ms, ftSs_MS_AirCatch as i32));
    }
    let (mut f, mut go) = mk();
    f.kind = Ft_Kind_Mario;
    f.input.held_buttons[0] = HSD_PAD_LR as HSD_Pad;
    f.input.pressed_buttons = HSD_PAD_A as HSD_Pad;
    assert!(!ac::ftco_800c3b10(&mut *go));
    f.kind = Ft_Kind_Samus;
    f.ground_or_air = GA_Ground; // grounded: enter is a no-op, but the tether is still spent
    assert!(ac::ftco_800c3b10(&mut *go));
    assert_eq!(f.used_tether(), 1);
    assert!(!ac::ftco_800c3b10(&mut *go));
}

#[test]
fn aircatchhit_phys_takes_pos_delta() {
    let (mut f, mut go) = mk();
    f.pos_delta = Vec3 { x: 3.0, y: 1.0, z: 0.0 };
    f.co_attrs.gravity = 0.5;
    f.co_attrs.terminal_velocity = 2.0;
    ac::ftco_aircatchhit_phys(&mut *go);
    assert_eq!(f.self_vel.x, 3.0);
    assert_eq!(f.self_vel.y, 0.5);
}

#[test]
#[should_panic(expected = "ft_80081D0C")]
fn coll_reaches_stub() {
    let (_f, mut go) = mk();
    ac::ftco_aircatch_coll(&mut *go);
}
