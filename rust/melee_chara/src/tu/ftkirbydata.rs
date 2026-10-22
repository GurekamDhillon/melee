//! src/melee/ft/kinds/ftKirby/ftkirbydata.c: the file-name / symbol-name tables of Kirby's copy
//! abilities and the m-ex hook `ftKb_MexCopyKindData`. No fused sites (tools/rust-port/fma_sites.py).
//! The tables are generated from the C by `gen/kirby/gen_data.py` (strings are NUL-terminated byte arrays,
//! C `NULL` is a null pointer); pointers are native. The table rows are `static mut` because
//! `ftKb_MexCopyKindData` rewrites them at m-ex load, like the C globals.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;
use core::ptr::null_mut;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(MEX_KIRBYCAPFILE, Mex_KirbyCapFile, (internal: i32) -> *mut i8);
    hook!(MEX_KIRBYCAPSYMBOL, Mex_KirbyCapSymbol, (internal: i32) -> *mut i8);
    hook!(MEX_KIRBYEFFECTID, Mex_KirbyEffectId, (internal: i32) -> i32);
}

/// `Ft_Kind_Max` table length.
pub const KIND_MAX: usize = 127;

pub static ftKb_Init_DatFilename: [u8; 9] = *b"PlKb.dat\0";
pub static ftKb_Init_DataName: [u8; 12] = *b"ftDataKirby\0";
pub static ftKb_Init_803CA320: [u8; 11] = *b"PlKbNr.dat\0";
pub static ftKb_Init_803CA32C: [u8; 23] = *b"PlyKirby5K_Share_joint\0";
pub static ftKb_Init_803CA344: [u8; 31] = *b"PlyKirby5K_Share_matanim_joint\0";
pub static ftKb_Init_803CA364: [u8; 11] = *b"PlKbYe.dat\0";
pub static ftKb_Init_803CA370: [u8; 25] = *b"PlyKirby5KYe_Share_joint\0";
pub static ftKb_Init_803CA38C: [u8; 33] = *b"PlyKirby5KYe_Share_matanim_joint\0";
pub static ftKb_Init_803CA3B0: [u8; 11] = *b"PlKbBu.dat\0";
pub static ftKb_Init_803CA3BC: [u8; 25] = *b"PlyKirby5KBu_Share_joint\0";
pub static ftKb_Init_803CA3D8: [u8; 33] = *b"PlyKirby5KBu_Share_matanim_joint\0";
pub static ftKb_Init_803CA3FC: [u8; 11] = *b"PlKbRe.dat\0";
pub static ftKb_Init_803CA408: [u8; 25] = *b"PlyKirby5KRe_Share_joint\0";
pub static ftKb_Init_803CA424: [u8; 33] = *b"PlyKirby5KRe_Share_matanim_joint\0";
pub static ftKb_Init_803CA448: [u8; 11] = *b"PlKbGr.dat\0";
pub static ftKb_Init_803CA454: [u8; 25] = *b"PlyKirby5KGr_Share_joint\0";
pub static ftKb_Init_803CA470: [u8; 33] = *b"PlyKirby5KGr_Share_matanim_joint\0";
pub static ftKb_Init_803CA494: [u8; 11] = *b"PlKbWh.dat\0";
pub static ftKb_Init_803CA4A0: [u8; 25] = *b"PlyKirby5KWh_Share_joint\0";
pub static ftKb_Init_803CA4BC: [u8; 33] = *b"PlyKirby5KWh_Share_matanim_joint\0";
pub static ftKb_Init_AnimDatFilename: [u8; 11] = *b"PlKbAJ.dat\0";
pub static ftKb_Init_803CAAD8: [u8; 15] = *b"PlKbNrCpDk.dat\0";
pub static ftKb_Init_803CAAE8: [u8; 23] = *b"PlyKirbyDk_Share_joint\0";
pub static ftKb_Init_803CAB00: [u8; 31] = *b"PlyKirbyDk_Share_matanim_joint\0";
pub static ftKb_Init_803CAB20: [u8; 15] = *b"PlKbYeCpDk.dat\0";
pub static ftKb_Init_803CAB30: [u8; 25] = *b"PlyKirbyDkYe_Share_joint\0";
pub static ftKb_Init_803CAB4C: [u8; 33] = *b"PlyKirbyDkYe_Share_matanim_joint\0";
pub static ftKb_Init_803CAB70: [u8; 15] = *b"PlKbBuCpDk.dat\0";
pub static ftKb_Init_803CAB80: [u8; 25] = *b"PlyKirbyDkBu_Share_joint\0";
pub static ftKb_Init_803CAB9C: [u8; 33] = *b"PlyKirbyDkBu_Share_matanim_joint\0";
pub static ftKb_Init_803CABC0: [u8; 15] = *b"PlKbReCpDk.dat\0";
pub static ftKb_Init_803CABD0: [u8; 25] = *b"PlyKirbyDkRe_Share_joint\0";
pub static ftKb_Init_803CABEC: [u8; 33] = *b"PlyKirbyDkRe_Share_matanim_joint\0";
pub static ftKb_Init_803CAC10: [u8; 15] = *b"PlKbGrCpDk.dat\0";
pub static ftKb_Init_803CAC20: [u8; 25] = *b"PlyKirbyDkGr_Share_joint\0";
pub static ftKb_Init_803CAC3C: [u8; 33] = *b"PlyKirbyDkGr_Share_matanim_joint\0";
pub static ftKb_Init_803CAC60: [u8; 15] = *b"PlKbWhCpDk.dat\0";
pub static ftKb_Init_803CAC70: [u8; 25] = *b"PlyKirbyDkWh_Share_joint\0";
pub static ftKb_Init_803CAC8C: [u8; 33] = *b"PlyKirbyDkWh_Share_matanim_joint\0";
pub static ftKb_Init_803CACF8: [u8; 15] = *b"PlKbNrCpPr.dat\0";
pub static ftKb_Init_803CAD08: [u8; 23] = *b"PlyKirbyPr_Share_joint\0";
pub static ftKb_Init_803CAD20: [u8; 31] = *b"PlyKirbyPr_Share_matanim_joint\0";
pub static ftKb_Init_803CAD40: [u8; 15] = *b"PlKbYeCpPr.dat\0";
pub static ftKb_Init_803CAD50: [u8; 25] = *b"PlyKirbyPrYe_Share_joint\0";
pub static ftKb_Init_803CAD6C: [u8; 33] = *b"PlyKirbyPrYe_Share_matanim_joint\0";
pub static ftKb_Init_803CAD90: [u8; 15] = *b"PlKbBuCpPr.dat\0";
pub static ftKb_Init_803CADA0: [u8; 25] = *b"PlyKirbyPrBu_Share_joint\0";
pub static ftKb_Init_803CADBC: [u8; 33] = *b"PlyKirbyPrBu_Share_matanim_joint\0";
pub static ftKb_Init_803CADE0: [u8; 15] = *b"PlKbReCpPr.dat\0";
pub static ftKb_Init_803CADF0: [u8; 25] = *b"PlyKirbyPrRe_Share_joint\0";
pub static ftKb_Init_803CAE0C: [u8; 33] = *b"PlyKirbyPrRe_Share_matanim_joint\0";
pub static ftKb_Init_803CAE30: [u8; 15] = *b"PlKbGrCpPr.dat\0";
pub static ftKb_Init_803CAE40: [u8; 25] = *b"PlyKirbyPrGr_Share_joint\0";
pub static ftKb_Init_803CAE5C: [u8; 33] = *b"PlyKirbyPrGr_Share_matanim_joint\0";
pub static ftKb_Init_803CAE80: [u8; 15] = *b"PlKbWhCpPr.dat\0";
pub static ftKb_Init_803CAE90: [u8; 25] = *b"PlyKirbyPrWh_Share_joint\0";
pub static ftKb_Init_803CAEAC: [u8; 33] = *b"PlyKirbyPrWh_Share_matanim_joint\0";
pub static ftKb_Init_803CAF18: [u8; 15] = *b"PlKbNrCpMt.dat\0";
pub static ftKb_Init_803CAF28: [u8; 23] = *b"PlyKirbyMt_Share_joint\0";
pub static ftKb_Init_803CAF40: [u8; 31] = *b"PlyKirbyMt_Share_matanim_joint\0";
pub static ftKb_Init_803CAF60: [u8; 15] = *b"PlKbYeCpMt.dat\0";
pub static ftKb_Init_803CAF70: [u8; 25] = *b"PlyKirbyMtYe_Share_joint\0";
pub static ftKb_Init_803CAF8C: [u8; 33] = *b"PlyKirbyMtYe_Share_matanim_joint\0";
pub static ftKb_Init_803CAFB0: [u8; 15] = *b"PlKbBuCpMt.dat\0";
pub static ftKb_Init_803CAFC0: [u8; 25] = *b"PlyKirbyMtBu_Share_joint\0";
pub static ftKb_Init_803CAFDC: [u8; 33] = *b"PlyKirbyMtBu_Share_matanim_joint\0";
pub static ftKb_Init_803CB000: [u8; 15] = *b"PlKbReCpMt.dat\0";
pub static ftKb_Init_803CB010: [u8; 25] = *b"PlyKirbyMtRe_Share_joint\0";
pub static ftKb_Init_803CB02C: [u8; 33] = *b"PlyKirbyMtRe_Share_matanim_joint\0";
pub static ftKb_Init_803CB050: [u8; 15] = *b"PlKbGrCpMt.dat\0";
pub static ftKb_Init_803CB060: [u8; 25] = *b"PlyKirbyMtGr_Share_joint\0";
pub static ftKb_Init_803CB07C: [u8; 33] = *b"PlyKirbyMtGr_Share_matanim_joint\0";
pub static ftKb_Init_803CB0A0: [u8; 15] = *b"PlKbWhCpMt.dat\0";
pub static ftKb_Init_803CB0B0: [u8; 25] = *b"PlyKirbyMtWh_Share_joint\0";
pub static ftKb_Init_803CB0CC: [u8; 33] = *b"PlyKirbyMtWh_Share_matanim_joint\0";
pub static ftKb_Init_803CB138: [u8; 15] = *b"PlKbNrCpFc.dat\0";
pub static ftKb_Init_803CB148: [u8; 23] = *b"PlyKirbyFc_Share_joint\0";
pub static ftKb_Init_803CB160: [u8; 31] = *b"PlyKirbyFc_Share_matanim_joint\0";
pub static ftKb_Init_803CB180: [u8; 15] = *b"PlKbYeCpFc.dat\0";
pub static ftKb_Init_803CB190: [u8; 25] = *b"PlyKirbyFcYe_Share_joint\0";
pub static ftKb_Init_803CB1AC: [u8; 33] = *b"PlyKirbyFcYe_Share_matanim_joint\0";
pub static ftKb_Init_803CB1D0: [u8; 15] = *b"PlKbBuCpFc.dat\0";
pub static ftKb_Init_803CB1E0: [u8; 25] = *b"PlyKirbyFcBu_Share_joint\0";
pub static ftKb_Init_803CB1FC: [u8; 33] = *b"PlyKirbyFcBu_Share_matanim_joint\0";
pub static ftKb_Init_803CB220: [u8; 15] = *b"PlKbReCpFc.dat\0";
pub static ftKb_Init_803CB230: [u8; 25] = *b"PlyKirbyFcRe_Share_joint\0";
pub static ftKb_Init_803CB24C: [u8; 33] = *b"PlyKirbyFcRe_Share_matanim_joint\0";
pub static ftKb_Init_803CB270: [u8; 15] = *b"PlKbGrCpFc.dat\0";
pub static ftKb_Init_803CB280: [u8; 25] = *b"PlyKirbyFcGr_Share_joint\0";
pub static ftKb_Init_803CB29C: [u8; 33] = *b"PlyKirbyFcGr_Share_matanim_joint\0";
pub static ftKb_Init_803CB2C0: [u8; 15] = *b"PlKbWhCpFc.dat\0";
pub static ftKb_Init_803CB2D0: [u8; 25] = *b"PlyKirbyFcWh_Share_joint\0";
pub static ftKb_Init_803CB2EC: [u8; 33] = *b"PlyKirbyFcWh_Share_matanim_joint\0";
pub static ftKb_Init_803CB358: [u8; 15] = *b"PlKbNrCpGw.dat\0";
pub static ftKb_Init_803CB368: [u8; 23] = *b"PlyKirbyGw_Share_joint\0";
pub static ftKb_Init_803CB380: [u8; 31] = *b"PlyKirbyGw_Share_matanim_joint\0";

pub static mut ftKb_Init_803CA5A4: [*mut i8; 4] = [
    concat!("ftDemoVi0501MotionFileKirby", "\0").as_ptr() as *mut i8,
    null_mut(),
    null_mut(),
    concat!("ftDemoVi0502MotionFileKirby", "\0").as_ptr() as *mut i8,
];

pub static mut ftKb_Init_DemoMotionFilenames: Fighter_DemoStrings = Fighter_DemoStrings {
    result_filename: concat!("ftDemoResultMotionFileKirby", "\0").as_ptr() as *mut i8,
    intro_filename: concat!("ftDemoIntroMotionFileKirby", "\0").as_ptr() as *mut i8,
    ending_filename: concat!("ftDemoEndingMotionFileKirby", "\0").as_ptr() as *mut i8,
    vi_wait_filename: concat!("ftDemoViWaitMotionFileKirby", "\0").as_ptr() as *mut i8,
};

pub static mut ftKb_Init_CostumeStrings: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA320.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA32C.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA344.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA364.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA370.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA38C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA3B0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA3BC.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA3D8.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA3FC.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA408.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA424.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA448.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA454.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA470.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CA494.as_ptr() as *mut i8, joint_name: ftKb_Init_803CA4A0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CA4BC.as_ptr() as *mut i8 },
];

pub static mut ftKb_Init_803CACB0: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAAD8.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAAE8.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAB00.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAB20.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAB30.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAB4C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAB70.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAB80.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAB9C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CABC0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CABD0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CABEC.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAC10.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAC20.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAC3C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAC60.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAC70.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAC8C.as_ptr() as *mut i8 },
];

pub static mut ftKb_Init_803CAED0: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CACF8.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAD08.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAD20.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAD40.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAD50.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAD6C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAD90.as_ptr() as *mut i8, joint_name: ftKb_Init_803CADA0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CADBC.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CADE0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CADF0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAE0C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAE30.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAE40.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAE5C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAE80.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAE90.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAEAC.as_ptr() as *mut i8 },
];

pub static mut ftKb_Init_803CB0F0: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAF18.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAF28.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAF40.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAF60.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAF70.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAF8C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CAFB0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CAFC0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CAFDC.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB000.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB010.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB02C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB050.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB060.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB07C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB0A0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB0B0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB0CC.as_ptr() as *mut i8 },
];

pub static mut ftKb_Init_803CB310: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB138.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB148.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB160.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB180.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB190.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB1AC.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB1D0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB1E0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB1FC.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB220.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB230.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB24C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB270.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB280.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB29C.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB2C0.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB2D0.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB2EC.as_ptr() as *mut i8 },
];

pub static mut ftKb_Init_803CB3A0: [Fighter_CostumeStrings; 6] = [
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
    Fighter_CostumeStrings { dat_filename: ftKb_Init_803CB358.as_ptr() as *mut i8, joint_name: ftKb_Init_803CB368.as_ptr() as *mut i8, matanim_joint_name: ftKb_Init_803CB380.as_ptr() as *mut i8 },
];

/// `ftKb_Init_803CA9D0`: per-kind Kirby hat archive file / symbol names.
pub static mut ftKb_Init_803CA9D0: [ftKirby_CopyName; 127] = [
    ftKirby_CopyName { filename: concat!("PlKbCpMr.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyMario", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpFx.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyFox", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpCa.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyCaptain", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpDk.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyDonkey", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: concat!("PlKbCpKp.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyKoopa", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpLk.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyLink", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpSk.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopySeak", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpNs.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyNess", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpPe.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyPeach", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpPp.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyPopo", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: concat!("PlKbCpPk.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyPikachu", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpSs.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopySamus", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpYs.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyYoshi", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpPr.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyPurin", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpMt.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyMewtwo", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpLg.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyLuigi", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpMs.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyMars", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpZd.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyZelda", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpCl.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyClink", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpDr.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyDrmario", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpFc.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyFalco", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpPc.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyPichu", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpGw.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyGamewatch", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpGn.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyGanon", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: concat!("PlKbCpFe.dat", "\0").as_ptr() as *mut i8, name: concat!("ftDataKirbyCopyEmblem", "\0").as_ptr() as *mut i8 },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
    ftKirby_CopyName { filename: null_mut(), name: null_mut() },
];

/// `ftKb_Init_803CB3E8`: per-kind Kirby hat COSTUME model name rows (m-ex MexData.kirby_data.costumes).
pub static mut ftKb_Init_803CB3E8: [*mut Fighter_CostumeStrings; 127] = [
    null_mut(),
    null_mut(),
    null_mut(),
    (&raw mut ftKb_Init_803CACB0) as *mut Fighter_CostumeStrings,
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    (&raw mut ftKb_Init_803CAED0) as *mut Fighter_CostumeStrings,
    (&raw mut ftKb_Init_803CB0F0) as *mut Fighter_CostumeStrings,
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    (&raw mut ftKb_Init_803CB310) as *mut Fighter_CostumeStrings,
    null_mut(),
    (&raw mut ftKb_Init_803CB3A0) as *mut Fighter_CostumeStrings,
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
    null_mut(),
];

/// `ftKb_Init_803CB46C`: per-kind Kirby hat effect bank id (0xFF = none).
pub static mut ftKb_Init_803CB46C: [u8; 127] = [32, 33, 38, 39, 255, 41, 35, 21, 42, 45, 46, 46, 36, 34, 40, 43, 44, 37, 20, 21, 35, 32, 33, 36, 255, 47, 48, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

/// `ftKb_MexCopyKindData(dst, src, internal)` (TARGET_PC): fill fighter kind `dst`'s Kirby hat archive
/// and effect bank from MxDt.dat's MexData.kirby_data, for m-ex INTERNAL kind `internal`. `src` is the
/// clone base, used only when the disc has no row. The effect id is stored as 0xFF when m-ex says
/// "no effect bank" (255, or a negative id), which `ftKb_SpecialN_800EEC34`'s existing guard covers.
pub fn ftKb_MexCopyKindData(dst: i32, src: i32, internal: i32) {
    let file = ext::Mex_KirbyCapFile(internal);
    let symbol = ext::Mex_KirbyCapSymbol(internal);
    let effect = ext::Mex_KirbyEffectId(internal);
    unsafe {
        if !file.is_null() && !symbol.is_null() {
            ftKb_Init_803CA9D0[dst as usize].filename = file;
            ftKb_Init_803CA9D0[dst as usize].name = symbol;
        } else {
            ftKb_Init_803CA9D0[dst as usize] = ftKb_Init_803CA9D0[src as usize];
        }
        ftKb_Init_803CB46C[dst as usize] = (if effect < 0 { 0xFF } else { effect }) as u8;
    }
    // One line per added fighter (OSReport in the C); it prints what m-ex supplied, never a read-back
    // of the clone base's pointers.
    let cs = |p: *mut i8| -> String {
        if p.is_null() {
            "(clone base)".to_string()
        } else {
            unsafe { core::ffi::CStr::from_ptr(p as *const core::ffi::c_char) }.to_string_lossy().into_owned()
        }
    };
    eprintln!("gw: kind {} Kirby hat: {} / {}, effect {}", dst, cs(file), cs(symbol), effect);
}

