//! tu::ftkirbycaptureyoshi (src/melee/ft/kinds/ftKirby/ftkirbycaptureyoshi.c): no fused sites, no pure
//! math; a smoke test of the capture entry and the egg-hide helper through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbycaptureyoshi as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut k: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut y: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut kg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut yg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    kg.user_data = &mut *k as *mut Fighter as *mut _;
    yg.user_data = &mut *y as *mut Fighter as *mut _;
    let (kp, yp): (*mut HSD_GObj, *mut HSD_GObj) = (&mut *kg, &mut *yg);
    y.facing_dir = 1.0;
    k.facing_dir = 1.0;
    k.anon_221b.set_x221B_b5(1);
    k.anon_221b.set_x221B_b7(1);
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, msid, flags, a, b, c, _| log(format!("cms {msid:x} {flags} {a} {b} {c}")));
        st::FTANIM_8006EBA4 = Some(|_| log("8006EBA4".into()));
        t::ext::FTCOLL_8007B62C = Some(|_, v| log(format!("b62c {v}")));
    }
    unsafe {
        t::ext::FTCO_RELEASEITEMANDVICTIM = Some(|_| log("release".into()));
        t::ext::FTCO_800DB368 = Some(|a, f| log(format!("db368 {}", unsafe { (*a).facing_dir + (*f).facing_dir })));
    }
    // The capture entry points the pulled fighter at the egg owner and flips its facing.
    t::fn_8010A930(kp, yp);
    assert_eq!(k.x1A5C, yp);
    assert_eq!(k.victim_gobj, yp);
    assert_eq!(k.anon_221b.x221B_b5(), 0);
    assert_eq!(k.anon_221b.x221B_b7(), 0);
    assert_eq!(k.facing_dir, -1.0);
    assert!(k.accessory1_cb.is_some());
    // The four state callbacks are empty; SpecialNYs_8010AA2C hides Kirby and calls ftColl_8007B62C(gobj, 2).
    t::ftCo_CaptureKirbyYoshi_Anim(kp);
    t::ftCo_CaptureKirbyYoshi_IASA(kp);
    t::ftCo_CaptureKirbyYoshi_Phys(kp);
    t::ftCo_CaptureKirbyYoshi_Coll(kp);
    t::ftKb_SpecialNYs_8010AA2C(kp);
    assert_eq!(k.invisible(), 1);
    assert_eq!(unsafe { LOG.join(",") }, "release,db368 0,cms 14b 0 0 1 0,8006EBA4,b62c 2");
}
