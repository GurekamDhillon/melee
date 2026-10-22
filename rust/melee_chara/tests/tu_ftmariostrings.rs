//! tu::ftmariostrings is pure data: spot-check the table wiring.
use melee_chara::tu::ftmariostrings as t;

#[test]
fn tables() {
    assert_eq!(t::ftMr_Init_CostumeStrings.len(), 5);
    assert_eq!(t::ftMr_Init_CostumeStrings[3], ["PlMrBu.dat", "PlyMario5KBu_Share_joint", "PlyMario5KBu_Share_matanim_joint"]);
    assert_eq!(t::ftMr_Init_DemoViMotionFilenames[1], "ftDemoVi1101MotionFileMario");
    assert_eq!(t::ftMr_Init_DemoMotionFilenames[3], "ftDemoViWaitMotionFileMario");
}
