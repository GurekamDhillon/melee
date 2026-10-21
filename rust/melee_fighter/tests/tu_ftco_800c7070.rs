//! Smoke test for src/tu/ftco_800c7070.rs (stateful: callees recorded through the ext hooks).
#![allow(static_mut_refs)]
use melee_fighter::tu::ftco_800c7070::ext as e70;
use melee_fighter::tu::ftco_800c70d0::ext as e0d0;
use melee_fighter::types::*;

static mut MS: i32 = -1;
static mut THUNK: u32 = 0;

fn setup(kind: FighterKind) -> (Box<HSD_GObj>, Box<Fighter>) {
    let mut go = Box::new(HSD_GObj::zeroed());
    let mut f = Box::new(Fighter::zeroed());
    f.kind = kind;
    go.user_data = &mut *f as *mut Fighter as *mut _;
    unsafe {
        MS = -1;
        THUNK = 0;
        e70::FIGHTER_CHANGE_MOTION_STATE = Some(|_g, ms, fl, a, b, c, o| {
            assert_eq!((fl, a, b, c), (0, 0.0, 1.0, 0.0));
            assert!(o.is_null());
            MS = ms;
        });
        e0d0::FT_8008521C = Some(|_g| THUNK += 1);
    }
    (go, f)
}

fn check(go: &mut HSD_GObj, f: &Fighter, ms: i32) {
    let _ = go;
    assert_eq!(unsafe { MS }, ms);
    assert_eq!((f.x2219_b1(), f.x2219_b2()), (1, 1));
}

#[test]
fn rebirth_wait() {
    let (mut go, f) = setup(Ft_Kind_Fox);
    melee_fighter::tu::ftco_800c7070::ftco_800c7070(&mut *go);
    check(&mut go, &f, ftCo_MS_RebirthWait);
}
