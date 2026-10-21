//! ftmetal.c smoke tests: the timer/health bookkeeping paths that touch no JObj.
use melee_fighter::tu::ftmetal as m;
use melee_fighter::types::*;

fn gobj(fp: &mut Fighter) -> HSD_GObj {
    let mut g = HSD_GObj::zeroed();
    g.user_data = fp as *mut Fighter as *mut _;
    g
}

#[test]
fn metal_bookkeeping() {
    let mut fp = Fighter::zeroed();
    let mut g = gobj(&mut fp);
    let gp: *mut HSD_GObj = &mut g;
    let f = unsafe { &mut *((*gp).user_data as *mut Fighter) };
    f.set_is_metal(1);
    m::ftCo_800C8348(gp, 300, 50);
    assert_eq!((f.metal_timer, f.metal_health, f.is_metal()), (300, 50, 1));
    m::ftCo_800C8438(gp); // not always metal: back to normal, no JObj work
    assert_eq!((f.metal_timer, f.metal_health, f.is_metal()), (0, 0, 0));
    f.set_is_always_metal(1);
    f.set_is_metal(1);
    f.metal_timer = 9;
    m::ftCo_800C8540(gp); // always metal: only the counters reset
    assert_eq!((f.metal_timer, f.metal_health, f.is_metal()), (0, 0, 1));
}
