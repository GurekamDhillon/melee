//! Smoke test of tu::ftnana: OnLoad, OnDeath, the Popo-link checks and the data strings,
//! through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftnana as t;
use melee_chara::tu::ftnanaspecials as sp;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { std::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut attrs: Box<ftIceClimberAttributes> = Box::new(unsafe { core::mem::zeroed() });
    attrs.xC4 = 1.75;
    attrs.xC8 = 2.5;
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.dat_attrs = &mut *attrs as *mut _ as *mut _;
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::FTPP_INIT_ONLOADFORNANA = Some(|_| log("popo_onload".into()));
        st::FTPARTS_80074A4C = Some(|_, idx, v| log(format!("parts {idx} {v}")));
        st::FIGHTER_UNKSETFLAG_8006CFBC = Some(|_| log("flag".into()));
        sp::ext::FTPARTSETROTX = Some(|_, part, v| log(format!("rotx {part} {v}")));
    }

    // OnLoad: sets the bit, runs Popo's loader, copies xC4 into x40.
    t::ftNn_Init_OnLoad(gp);
    assert_eq!(take(), "popo_onload");
    assert_eq!(f.x2222_b4(), 1);
    assert_eq!(f.x40, 1.75);

    // OnDeath: armor from xC8, both model parts reset, Nana's state cleared.
    f.u.nn.x2234 = 9;
    f.u.nn.x224C = 8;
    f.u.nn.x2250 = 7.0;
    unsafe { f.u.nn.set_x2230_b0(1) };
    t::ftNn_Init_OnDeath(gp);
    assert_eq!(take(), "parts 0 0,parts 1 0");
    assert_eq!(f.dmg.armor0, 2.5);
    assert_eq!(unsafe { (f.u.nn.x2234, f.u.nn.x224C, f.u.nn.x2250, f.u.nn.x2230_b0()) }, (0, 0, 0.0, 0));

    // 80122FAC releases the Popo link.
    t::ftNn_Init_80122FAC(gp);
    assert_eq!(take(), "rotx 0 0,flag");

    // 8012309C: Nana is in the Belay motions 362..=366 and not frozen.
    for (m, want) in [(361, false), (362, true), (366, true), (367, false)] {
        f.motion_id = m;
        assert_eq!(t::ftNn_Init_8012309C(gp), want, "motion {m}");
    }
    f.set_is_sleeping(1);
    assert!(!t::ftNn_Init_8012309C(gp));
    f.set_is_sleeping(0);

    // 8012300C: refuses when frozen / flagged; otherwise acts for the "free" sub-states only.
    f.set_x2219_b5(1);
    assert!(!t::ftNn_Init_8012300C(gp));
    f.set_x2219_b5(0);
    unsafe { f.x2070.x0.set_x2071_b0_3(1) };
    assert!(!t::ftNn_Init_8012300C(gp));
    assert_eq!(take(), "");

    assert_eq!(t::ftNn_Init_DatFilename, "PlNn.dat");
    assert_eq!(t::ftNn_Init_CostumeStrings[2][1], "PlyNana5KAq_Share_joint");
    assert_eq!(t::ftNn_Init_DemoMotionFilenames[3], "ftDemoViWaitMotionFilePopo");
}
