//! tu::ftkirbydata (src/melee/ft/kinds/ftKirby/ftkirbydata.c): the file-name / symbol tables checked
//! against the C text (spot values per table, NULL rows, the 0xFF effect ids) and
//! `ftKb_MexCopyKindData`'s two branches through hooks. No fused sites, no float math.
#![allow(static_mut_refs)]

use core::ffi::CStr;
use melee_chara::tu::ftkirbydata as t;
use melee_types::*;

fn s(p: *mut i8) -> String {
    assert!(!p.is_null());
    unsafe { CStr::from_ptr(p as *const core::ffi::c_char) }.to_str().unwrap().to_string()
}

#[test]
fn tables() {
    unsafe {
        assert_eq!(t::KIND_MAX, Ft_Kind_Max as usize);
        assert_eq!(s(t::ftKb_Init_803CA9D0[0].filename), "PlKbCpMr.dat");
        assert_eq!(s(t::ftKb_Init_803CA9D0[0].name), "ftDataKirbyCopyMario");
        assert_eq!(s(t::ftKb_Init_803CA9D0[26].filename), "PlKbCpFe.dat");
        assert_eq!(s(t::ftKb_Init_803CA9D0[24].name), "ftDataKirbyCopyGamewatch");
        for k in [4, 11, 27, 28, 29, 30, 31, 32, 33, 126] {
            assert!(t::ftKb_Init_803CA9D0[k].filename.is_null() && t::ftKb_Init_803CA9D0[k].name.is_null(), "row {k}");
        }
        // Effect banks: 0xFF where the C has -1 (kinds 4, 24, 27..32), zero beyond the 33 listed.
        assert_eq!(&t::ftKb_Init_803CB46C[..8], &[32, 33, 38, 39, 255, 41, 35, 21]);
        assert_eq!(t::ftKb_Init_803CB46C[24], 255);
        assert_eq!(t::ftKb_Init_803CB46C[25], 47);
        assert_eq!(t::ftKb_Init_803CB46C[126], 0);
        // Per-hat costume rows: Donkey (3), Jigglypuff (15), Mewtwo (16), Falco (22), Game & Watch (24).
        for k in 0..t::KIND_MAX {
            let want = matches!(k, 3 | 15 | 16 | 22 | 24);
            assert_eq!(!t::ftKb_Init_803CB3E8[k].is_null(), want, "costume row {k}");
        }
        let dk = t::ftKb_Init_803CB3E8[3];
        assert_eq!(s((*dk.add(0)).dat_filename), "PlKbNrCpDk.dat");
        assert_eq!(s((*dk.add(5)).matanim_joint_name), "PlyKirbyDkWh_Share_matanim_joint");
        let gw = t::ftKb_Init_803CB3E8[24];
        assert_eq!(s((*gw.add(5)).joint_name), "PlyKirbyGw_Share_joint");
        assert_eq!((*gw.add(0)).dat_filename, (*gw.add(5)).dat_filename);
        // Kirby's own files.
        assert_eq!(s(t::ftKb_Init_CostumeStrings[1].dat_filename), "PlKbYe.dat");
        assert_eq!(s(t::ftKb_Init_CostumeStrings[5].matanim_joint_name), "PlyKirby5KWh_Share_matanim_joint");
        assert_eq!(s(t::ftKb_Init_DemoMotionFilenames.vi_wait_filename), "ftDemoViWaitMotionFileKirby");
        assert_eq!(s(t::ftKb_Init_803CA5A4[3]), "ftDemoVi0502MotionFileKirby");
        assert!(t::ftKb_Init_803CA5A4[1].is_null());
        assert_eq!(&t::ftKb_Init_DatFilename, b"PlKb.dat\0");
        assert_eq!(&t::ftKb_Init_AnimDatFilename, b"PlKbAJ.dat\0");
    }
}

#[test]
fn mex_copy_kind_data() {
    static mut FILE: [u8; 9] = *b"PlKbCpWf\0";
    static mut SYM: [u8; 5] = *b"ftWf\0";
    unsafe {
        // Row supplied by m-ex: stored verbatim, effect id 255 -> 0xFF, negative -> 0xFF.
        t::ext::MEX_KIRBYCAPFILE = Some(|_| (&raw mut FILE) as *mut i8);
        t::ext::MEX_KIRBYCAPSYMBOL = Some(|_| (&raw mut SYM) as *mut i8);
        t::ext::MEX_KIRBYEFFECTID = Some(|_| 255);
        t::ftKb_MexCopyKindData(40, 0, 27);
        assert_eq!(s(t::ftKb_Init_803CA9D0[40].filename), "PlKbCpWf");
        assert_eq!(s(t::ftKb_Init_803CA9D0[40].name), "ftWf");
        assert_eq!(t::ftKb_Init_803CB46C[40], 255);
        t::ext::MEX_KIRBYEFFECTID = Some(|_| -3);
        t::ftKb_MexCopyKindData(41, 0, 27);
        assert_eq!(t::ftKb_Init_803CB46C[41], 255);
        t::ext::MEX_KIRBYEFFECTID = Some(|_| 20);
        // No row: the clone base's names are copied.
        t::ext::MEX_KIRBYCAPFILE = Some(|_| core::ptr::null_mut());
        t::ftKb_MexCopyKindData(42, 1, 27);
        assert_eq!(s(t::ftKb_Init_803CA9D0[42].filename), "PlKbCpFx.dat");
        assert_eq!(t::ftKb_Init_803CB46C[42], 20);
    }
}
