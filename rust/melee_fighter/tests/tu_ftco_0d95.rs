//! tu::ftco_0d95 smoke tests: non-Samus is a no-op; Samus spawn frame calls the (unported)
//! lb_8000B1CC stub; ftCo_800D9C98 with no grapple clears the callbacks.

use melee_fighter::tu::ftco_0d95::*;
use melee_fighter::types::*;

fn mk() -> (Box<Fighter>, Box<HSD_GObj>) {
    let mut f = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = (&mut *f as *mut Fighter).cast();
    (f, go)
}

#[test]
fn non_samus_noop() {
    let (mut f, mut go) = mk();
    f.kind = Ft_Kind_Mario;
    assert!(!fn_800d9558(&mut *go));
    assert!(!fn_800d9930(&mut *go));
    fn_800d9c64(&mut *go);
    assert_eq!(unsafe { f.mv.ca.specials.grav }, 0.0);
}

#[test]
fn clear_without_grapple() {
    let (f, mut go) = mk();
    ftco_800d9c98(&mut *go);
    assert!(f.death2_cb.is_none() && f.take_dmg_cb.is_none() && f.take_dmg_2_cb.is_none());
}

#[test]
#[should_panic(expected = "lb_8000B1CC")]
fn samus_spawn_frame_reaches_stub() {
    let (mut f, mut go) = mk();
    let mut attrs: Box<ftSs_DatAttrs> = Box::new(unsafe { core::mem::zeroed() });
    attrs.x9C = 1;
    let mut parts: std::vec::Vec<FighterBone> = (0..60).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.kind = Ft_Kind_Samus;
    f.dat_attrs = (&mut *attrs as *mut ftSs_DatAttrs).cast();
    f.parts = parts.as_mut_ptr();
    fn_800d9558(&mut *go);
}
