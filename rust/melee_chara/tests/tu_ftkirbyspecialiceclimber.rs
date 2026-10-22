//! Smoke test for tu::ftkirbyspecialiceclimber (stateful; one fused site in fn_801090D4) through
//! recording hooks. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialiceclimber as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut ICE: HSD_GObj = unsafe { core::mem::zeroed() };
static mut HAT: KirbyHatStruct = unsafe { core::mem::zeroed() };

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..256).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        melee_fighter::hooks::FTPARTS_GET_BONE_INDEX = Some(|_, p| p + 100);
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| log(format!("ms {ms} {fl:x}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCO_FALL_ENTER = Some(|_| log("Fall".into()));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, a, lag| log(format!("landfs {a} {lag}")));
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            assert!(off.is_null());
            *out = Vec3 { x: 1.0, y: 2.0, z: 0.0 };
        });
        t::ext::FT_80459B88_HAT = Some(|k| {
            assert_eq!(k, Ft_Kind_Peach as i32);
            unsafe { &raw mut HAT }
        });
        t::ext::FTCOMMON_SETACCESSORY = Some(|_, j| log(format!("setacc {}", j as usize)));
        t::ext::LB_8000C2F8 = Some(|_, _| log("attach".into()));
        t::ext::IT_802C17DC = Some(|_| log("destroy".into()));
        t::ext::IT_802C16F8 = Some(|_| log("release".into()));
        t::ext::IT_802C1590 = Some(|_, pos, kind, fd| {
            log(format!("ice {} {} {kind} {fd}", unsafe { (*pos).x }, unsafe { (*pos).y }));
            unsafe { &raw mut ICE }
        });
        t::ext::FT_PLAYSFX = Some(|_, id, _, _| log(format!("sfx {id:x}")));
        t::ext::FT_800881D8 = Some(|_, id, _, _| log(format!("voice {id:x}")));
    }

    // Ground enter: no item, accessory attached from the Peach hat's dynamics[1].
    f.cmd_vars[0] = 9;
    f.u.kb.xC0 = core::ptr::null_mut();
    unsafe { HAT.hat_dynamics[1] = 0x1234 as *mut _ };
    t::ftKb_SpecialNIc_80108D64(gp);
    assert_eq!(take(), [format!("ms {} 0", ftKb_MS_PpSpecialN), "setacc 4660".into(), "attach".into()]);
    assert_eq!(f.cmd_vars[0], 0);
    assert!(f.accessory4_cb.is_some());

    // Air enter: first use gives the vertical boost, a repeat does not.
    da.specialn_pp_air_vertical_momentum = 3.5;
    f.u.kb.xC4 = 0;
    t::ftKb_SpecialNIc_80108E14(gp);
    take();
    assert_eq!((f.self_vel.y, unsafe { f.u.kb.xC4 }, unsafe { f.u.kb.xC8 }), (3.5, 1, 0.0));
    f.self_vel.y = 0.0;
    t::ftKb_SpecialNIc_80108E14(gp);
    take();
    assert_eq!((f.self_vel.y, unsafe { f.u.kb.xC8 }), (0.0, -10.0));

    // A missing Peach hat skips the accessory.
    unsafe { t::ext::FT_80459B88_HAT = Some(|_| core::ptr::null_mut()) };
    t::ftKb_SpecialNIc_80108D64(gp);
    assert_eq!(take(), [format!("ms {} 0", ftKb_MS_PpSpecialN)]);

    // Ice spawn (cmd 1) at the hand: x += x_spawn * facing, y += y_spawn + xC8.
    da.specialn_pp_x_spawn = 2.0;
    da.specialn_pp_y_spawn = 0.5;
    f.facing_dir = -1.0;
    f.u.kb.xC8 = 0.25;
    f.cmd_vars[0] = 1;
    t::fn_801090D4(gp);
    assert_eq!(take(), [format!("ice -1 2.75 {} -1", It_Kind_Kirby_IceClimberIce), "sfx 1fbe5".into()]);
    assert!(!unsafe { f.u.kb.xC0 }.is_null());
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    assert_eq!(f.cmd_vars[0], 0);

    // Release (cmd 2): item popped, sfx, and the held-item slot cleared.
    f.cmd_vars[0] = 2;
    t::fn_801090D4(gp);
    assert_eq!(take(), ["release", "voice 2233b", "sfx 1fbe8"]);
    assert!(unsafe { f.u.kb.xC0 }.is_null());
    f.cmd_vars[0] = 0;
    t::fn_801090D4(gp);
    assert!(take().is_empty());

    // Coll: losing the ground kills a live item; air Coll lands in the special fall.
    unsafe { f.u.kb.xC0 = &raw mut ICE };
    t::ftKb_PpSpecialN_Coll(gp);
    assert_eq!(take(), ["destroy", "Fall"]);
    assert!(unsafe { f.u.kb.xC0 }.is_null());
    da.specialn_pp_landing_lag = 12.0;
    unsafe { f.u.kb.xC0 = &raw mut ICE };
    f.u.kb.xC4 = 1;
    t::ftKb_PpSpecialAirN_Coll(gp);
    assert_eq!(take(), ["destroy", "landfs 0 12"]);
    assert_eq!(unsafe { (f.u.kb.xC4, f.u.kb.xC8) }, (0, 0.0));

    // Item-owner clear callbacks.
    unsafe { f.u.kb.xC0 = &raw mut ICE };
    t::ftKb_SpecialNIc_80108CE8(gp, unsafe { &raw mut ICE });
    assert!(unsafe { f.u.kb.xC0 }.is_null());
    unsafe { f.u.kb.xC0 = &raw mut ICE };
    t::ftKb_SpecialNIc_80108D04(gp);
    assert_eq!(take(), ["destroy"]);
    assert_eq!(t::ftKb_SpecialNYs_80109238(gp), FtPart_TransN2 + 100);
}
