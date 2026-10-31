//! src/melee/ft/kinds/ftMario/ftmariostrings.c: Mario's file-name tables. Pure data (no
//! functions); `ftMr_CostumeList` is zero-initialised BSS. Checked by tests/tu_ftmariostrings.rs.
#![allow(non_upper_case_globals)]

pub const ftMr_Init_DatFilename: &str = "PlMr.dat";
pub const ftMr_Init_DataName: &str = "ftDataMario";
pub const ftMr_Strings_803C57E8: &str = "PlMrNr.dat";
pub const ftMr_Strings_803C57F4: &str = "PlyMario5K_Share_joint";
pub const ftMr_Strings_803C580C: &str = "PlyMario5K_Share_matanim_joint";
pub const ftMr_Strings_803C582C: &str = "PlMrYe.dat";
pub const ftMr_Strings_803C5838: &str = "PlyMario5KYe_Share_joint";
pub const ftMr_Strings_803C5854: &str = "PlyMario5KYe_Share_matanim_joint";
pub const ftMr_Strings_803C5878: &str = "PlMrBk.dat";
pub const ftMr_Strings_803C5884: &str = "PlyMario5KBk_Share_joint";
pub const ftMr_Strings_803C58A0: &str = "PlyMario5KBk_Share_matanim_joint";
pub const ftMr_Strings_803C58C4: &str = "PlMrBu.dat";
pub const ftMr_Strings_803C58D0: &str = "PlyMario5KBu_Share_joint";
pub const ftMr_Strings_803C58EC: &str = "PlyMario5KBu_Share_matanim_joint";
pub const ftMr_Strings_803C5910: &str = "PlMrGr.dat";
pub const ftMr_Strings_803C591C: &str = "PlyMario5KGr_Share_joint";
pub const ftMr_Strings_803C5938: &str = "PlyMario5KGr_Share_matanim_joint";
pub const ftMr_Init_AnimDatFilename: &str = "PlMrAJ.dat";

/// `ftMr_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftMr_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileMario",
    "ftDemoIntroMotionFileMario",
    "ftDemoEndingMotionFileMario",
    "ftDemoViWaitMotionFileMario",
];

pub const ftMr_Init_DemoViMotionFilenames: [&str; 2] = ["ftDemoVi0102MotionFileMario", "ftDemoVi1101MotionFileMario"];

/// `ftMr_Init_CostumeStrings`: (dat, joint, matanim joint) per costume.
pub const ftMr_Init_CostumeStrings: [[&str; 3]; 5] = [
    [ftMr_Strings_803C57E8, ftMr_Strings_803C57F4, ftMr_Strings_803C580C],
    [ftMr_Strings_803C582C, ftMr_Strings_803C5838, ftMr_Strings_803C5854],
    [ftMr_Strings_803C5878, ftMr_Strings_803C5884, ftMr_Strings_803C58A0],
    [ftMr_Strings_803C58C4, ftMr_Strings_803C58D0, ftMr_Strings_803C58EC],
    [ftMr_Strings_803C5910, ftMr_Strings_803C591C, ftMr_Strings_803C5938],
];

/// `ftMr_CostumeList`: zero-initialised BSS array of the C (`UnkCostumeStruct ftMr_CostumeList[5]`), filled when the costumes load.
pub static mut ftMr_CostumeList: [melee_types::UnkCostumeStruct; 5] = unsafe { core::mem::zeroed() };
