//! Smoke test of tu::ftpichu: the death costume-part switch, plus the file-name tables.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpichu as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe { st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("part {m} {v}"))) };

    // The costume's own model part is shown (0); the other two are hidden (-1).
    for (costume, want) in [
        (0u8, ["part 0 0", "part 1 -1", "part 2 -1", "part 3 -1"]),
        (1, ["part 0 0", "part 1 0", "part 2 -1", "part 3 -1"]),
        (2, ["part 0 0", "part 1 -1", "part 2 0", "part 3 -1"]),
        (3, ["part 0 0", "part 1 -1", "part 2 -1", "part 3 0"]),
    ] {
        f.x619_costume_id = costume;
        t::ftPc_Init_OnDeath(gp);
        assert_eq!(take(), want);
    }
    f.x619_costume_id = 9;
    t::ftPc_Init_OnDeath(gp);
    assert_eq!(take(), ["part 0 0"]);

    // File names: four costumes, each (dat, joint, matanim joint).
    assert_eq!(t::ftPc_Init_DatFilename, "PlPc.dat");
    assert_eq!(t::ftPc_Init_CostumeStrings[3][0], "PlPcGr.dat");
    assert_eq!(t::ftPc_Init_DemoMotionFilenames[0], "ftDemoResultMotionFilePichu");
}
