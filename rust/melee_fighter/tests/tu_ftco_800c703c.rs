//! Smoke test for src/tu/ftco_800c703c.rs (field copies, no arithmetic).
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::types::*;
use tu_s25::zeroed;

#[test]
fn seeds_velocity_and_dmg() {
    let mut cd: Box<ftCommonData> = zeroed();
    cd.x6CC = 42;
    cd.x6D0 = 1.5;
    unsafe { hooks::P_FT_COMMON_DATA = &*cd };
    let mut f: Box<Fighter> = zeroed();
    f.self_vel.x = -3.0;
    f.xA4_unk_vel.z = 9.0;
    let mut go: Box<HSD_GObj> = zeroed();
    go.user_data = &mut *f as *mut Fighter as *mut _;
    melee_fighter::tu::ftco_800c703c::ftco_800c703c(&mut *go);
    assert_eq!((f.xA4_unk_vel.x, f.xA4_unk_vel.y, f.xA4_unk_vel.z), (-3.0, 1.5, 0.0));
    assert_eq!((f.dmg.x1948, f.dmg.x194C), (42, 42));
}
