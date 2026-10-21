//! Smoke test for src/tu/ft_0c8c.rs (stamina KO state bits; unported callees not reached).
use melee_fighter::tu::ft_0c8c as t;
use melee_fighter::types::*;

fn mk() -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = (&mut *f as *mut Fighter).cast();
    (g, f)
}

static mut LAST: i32 = -1;
fn sfx(_f: &mut Fighter, id: i32, vol: i32, pan: i32) {
    assert_eq!((vol, pan), (127, 64));
    unsafe { LAST = id };
}

#[test]
fn ko_bits() {
    let (mut g, mut f) = mk();
    let gp: *mut HSD_GObj = &mut *g;
    f.set_x2224_b2(1);
    assert!(!t::ftco_800c8c84(gp));
    // not yet knocked back: only arms x2224_b3
    f.kind = Ft_Kind_Mario;
    t::fn_800c8e74(gp);
    assert_eq!(f.x2224_b3(), 1);
    // b3 clear: no-op
    f.set_x2224_b3(0);
    t::ftco_800c8d00(gp);

    let mut s: Box<FtSFX> = Box::new(unsafe { core::mem::zeroed() });
    s.xC = 0x1234;
    let mut d: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    d.x4C_sfx = &mut *s;
    f.ft_data = &mut *d;
    unsafe { melee_fighter::hooks::PLAY_SFX = Some(sfx) };
    t::fn_800c8e40(&mut f, core::ptr::null_mut());
    assert_eq!(unsafe { LAST }, 0x1234);
}
