//! Smoke test of tu::ftpikachu: the file-name tables (the init callbacks are thin: they forward
//! to ftclink's item helpers and group 3's attribute loader, covered by their own tests).
use melee_chara::tu::ftpikachu as t;

#[test]
fn names() {
    assert_eq!(t::ftPk_Init_DatFilename, "PlPk.dat");
    assert_eq!(t::ftPk_Init_DataName, "ftDataPikachu");
    assert_eq!(t::ftPk_Init_CostumeStrings[0][0], "PlPkNr.dat");
    assert_eq!(t::ftPk_Init_CostumeStrings[2][2], "PlyPikachu5KBu_Share_matanim_joint");
    assert_eq!(t::ftPk_Init_AnimDatFilename, "PlPkAJ.dat");
    assert_eq!(t::ftPk_Init_DemoMotionFilenames[3], "ftDemoViWaitMotionFilePikachu");
}
