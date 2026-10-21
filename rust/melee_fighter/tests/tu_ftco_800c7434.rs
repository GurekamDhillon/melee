//! Smoke test for src/tu/ftco_800c7434.rs (stateful: callees recorded through the ext hooks).
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
            assert!(fl == 0 || fl == 0x1080, "flags {fl:#x}");
            assert_eq!((a, b, c), (0.0, 1.0, 0.0));
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

use melee_fighter::tu::ftco_800c7434 as m;
use melee_fighter::tu::ftco_800c7590::ext as e90;
static mut CALLS: u32 = 0;
#[test]
fn gkoops_and_leadead_trigger() {
    let (mut go, f) = setup(Ft_Kind_GKoops);
    m::ftco_800c7434(&mut *go);
    check(&mut go, &f, 0xE);
    let (mut go, f) = setup(Ft_Kind_Fox);
    m::ftco_800c7434(&mut *go);
    check(&mut go, &f, ftCo_MS_RebirthWait);
    m::ftco_800c74ac(&mut *go);
    assert_eq!(unsafe { THUNK }, 1);

    // ftCo_800C74F4: not leadead -> false, no item query
    let (mut go, mut f) = setup(Ft_Kind_Fox);
    unsafe {
        m::ext::IT_802EAF28 = Some(|_| { CALLS += 1; 0 });
        m::ext::IT_802EADD8 = Some(|_| CALLS += 100);
    }
    assert!(!m::ftco_800c74f4(&mut *go));
    assert_eq!(unsafe { CALLS }, 0);
    f.dmg.x1860_element = HitElement_Leadead;
    f.capture_timer = 3;
    assert!(!m::ftco_800c74f4(&mut *go));
    f.capture_timer = 0;
    f.motion_id = ftCo_MS_CaptureLeadead;
    assert!(!m::ftco_800c74f4(&mut *go));
    assert_eq!(unsafe { CALLS }, 0);
    // all conditions hold -> enters CaptureLeadead (ftCo_800C7590, every callee stubbed)
    f.motion_id = ftCo_MS_Wait;
    let mut cd = Box::new(ftCommonData::zeroed());
    unsafe {
        melee_fighter::hooks::P_FT_COMMON_DATA = &mut *cd;
        e90::FTCOMMON_8007DB58 = Some(|_| {});
        e90::FTCO_8009750C = Some(|_| {});
        e90::FTCO_800DD168 = Some(|_| {});
        e90::FTCO_8008DCE0 = Some(|_, _, _| {});
        e90::FTCOMMON_8007D7FC = Some(|_| {});
        e90::PLAYER_80033BB8 = Some(|_| 0);
        e90::PLAYER_GET_HANDICAP = Some(|_| 0);
        e90::IT_802EAAEC = Some(|_, _, _, _| {});
        e90::FTCOMMON_8007EBAC = Some(|_, _, _| {});
        e90::CAMERA_REQUEST_QUAKE = Some(|_, _| {});
        e90::FTCOMMON_INIT_GRAB = Some(|f, _, t| f.grab_timer = t);
    }
    assert!(m::ftco_800c74f4(&mut *go));
    assert_eq!(unsafe { CALLS }, 1);
    assert_eq!(unsafe { MS }, ftCo_MS_CaptureLeadead);
    assert!(f.take_dmg_cb.is_some() && f.death2_cb.is_some());
    unsafe { (f.take_dmg_cb.unwrap())(&mut *go) };
    unsafe { (f.death2_cb.unwrap())(&mut *go) };
    assert_eq!(unsafe { CALLS }, 201);
}
