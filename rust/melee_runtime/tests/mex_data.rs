//! `melee_runtime::tu::mex_data` against the behaviour of pc/platform/gw_mex_ftfunction_runtime.c
//! (read from the C: these services read the loaded `MxDt.dat`, which a native build gets from the
//! host, so there is no in-process C to run; the cases below are the C's branches, one by one).
#![allow(static_mut_refs)]

use melee_runtime::tu::mex_data as m;
use std::ffi::CString;
use std::sync::Mutex;

/// The module state is process-global (the C globals); the tests take turns.
static LOCK: Mutex<()> = Mutex::new(());

fn install(data: Option<m::MexData>) {
    unsafe { m::MEX = data };
}

#[test]
fn vanilla_answers_without_mexdata() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    install(None);
    unsafe { m::SKINS = None };
    // gw_Mex_InternalForPortKind: 0..=26 identity, bosses 27..=32 identity while internal_count <= 6,
    // an m-ex slot has no internal id without data
    for fk in 0..=32 {
        assert_eq!(m::Mex_InternalForPortKind(fk), fk);
    }
    assert_eq!(m::Mex_InternalForPortKind(0x21), -1);
    assert_eq!(m::Mex_InternalForPortKind(-1), -1);
    // gw_Mex_FtBaseKind: 0 (Mario) while !gw_mex_ft_on()
    assert_eq!(m::Mex_FtBaseKind(5), 0);
    // gw_Mex_CostumeVisIdx: the costume itself
    for c in [0, 3, 7] {
        assert_eq!(m::Mex_CostumeVisIdx(1, c), c);
    }
    assert!(m::Mex_KirbyCapFile(3).is_null());
    assert!(m::Mex_KirbyCapSymbol(3).is_null());
    assert!(m::Mex_KirbyCostumes(3).is_null());
    assert_eq!(m::Mex_KirbyEffectId(3), -1);
    assert!(m::Mex_KirbyFunc(0, 3).is_null());
    assert_eq!(m::Skins_KirbyRow(4, 6), -1);
    assert_eq!(m::Skins_Like(4, 6), -1);
    assert_eq!(m::Slippi_Version(), 999_999);
    assert_eq!(m::Mex_OnFloatDispatch(2, core::ptr::null_mut()), 0);
}

#[test]
fn mexdata_rows() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut d = m::MexData { on: true, internal_count: 41, effect_bank_max: 64, ..Default::default() };
    d.slots = vec![m::MexSlot { internal: 27 }, m::MexSlot { internal: 28 }, m::MexSlot { internal: -1 }];
    d.base_kind = (0..41).map(|k| if k >= 27 { 8 } else { k }).collect();
    d.costume_vis = vec![Vec::new(); 41];
    d.costume_vis[27] = vec![0, 2, 255, -1];
    d.kirby_cap_file = (0..41).map(|k| if k == 27 { Some(CString::new("PlKbCpWf.dat").unwrap()) } else if k == 28 { Some(CString::new("").unwrap()) } else { None }).collect();
    d.kirby_cap_symbol = (0..41).map(|k| if k == 27 { Some(CString::new("ftDataKirbyCopyWolf").unwrap()) } else { None }).collect();
    d.kirby_effect_id = vec![0xFF; 41];
    d.kirby_effect_id[27] = 7;
    d.kirby_effect_id[28] = 200;
    d.kirby_func = vec![vec![core::ptr::null_mut(); 41]; 2];
    d.kirby_func[1][27] = 0x1234 as *mut core::ffi::c_void;
    install(Some(d));
    // the six bosses move to the last six internal ids when there are more than six
    assert_eq!(m::Mex_InternalForPortKind(27), 35);
    assert_eq!(m::Mex_InternalForPortKind(32), 40);
    assert_eq!(m::Mex_InternalForPortKind(26), 26);
    // port slot -> internal id
    assert_eq!(m::Mex_InternalForPortKind(0x21), 27);
    assert_eq!(m::Mex_InternalForPortKind(0x22), 28);
    assert_eq!(m::Mex_InternalForPortKind(0x23), -1);
    assert_eq!(m::Mex_InternalForPortKind(0x21 + 94), -1);
    assert_eq!(m::Mex_FtBaseKind(27), 8);
    assert_eq!(m::Mex_FtBaseKind(3), 3);
    assert_eq!(m::Mex_FtBaseKind(99), 0);
    // visibility row: shared index; 255 and negative map to 0; out of range -> the costume
    assert_eq!(m::Mex_CostumeVisIdx(0x21, 1), 2);
    assert_eq!(m::Mex_CostumeVisIdx(0x21, 2), 0);
    assert_eq!(m::Mex_CostumeVisIdx(0x21, 3), 0);
    assert_eq!(m::Mex_CostumeVisIdx(0x21, 4), 4);
    assert_eq!(m::Mex_CostumeVisIdx(0x21, -1), -1);
    // Kirby hat rows
    let f = m::Mex_KirbyCapFile(27);
    assert_eq!(unsafe { std::ffi::CStr::from_ptr(f) }.to_str().unwrap(), "PlKbCpWf.dat");
    assert!(m::Mex_KirbyCapFile(28).is_null()); // empty name
    assert!(m::Mex_KirbyCapFile(29).is_null()); // no row
    assert!(m::Mex_KirbyCapFile(-1).is_null());
    assert_eq!(unsafe { std::ffi::CStr::from_ptr(m::Mex_KirbyCapSymbol(27)) }.to_str().unwrap(), "ftDataKirbyCopyWolf");
    assert_eq!(m::Mex_KirbyEffectId(27), 7);
    assert_eq!(m::Mex_KirbyEffectId(28), -1); // past the effect bank table
    assert_eq!(m::Mex_KirbyEffectId(29), -1); // 255 = none
    assert_eq!(m::Mex_KirbyEffectId(-1), -1);
    assert_eq!(m::Mex_KirbyFunc(1, 27) as usize, 0x1234);
    assert!(m::Mex_KirbyFunc(0, 27).is_null());
    assert!(m::Mex_KirbyFunc(2, 27).is_null());
    assert!(m::Mex_KirbyFunc(0, -1).is_null());
    // the disc check on the hat archive
    unsafe { m::MEX.as_mut().unwrap().file_exists = Some(|n| n.to_bytes() != b"PlKbCpWf.dat") };
    assert!(m::Mex_KirbyCapFile(27).is_null());
    install(None);
}

#[test]
fn skins_rows() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    install(None);
    let reg = m::SkinRegistry {
        base: vec![5, 5, 6],
        rows: vec![
            m::SkinRow { fk: 1, index: 5, like: 2, kirby_hat: 3 },
            m::SkinRow { fk: 1, index: 6, like: 9, kirby_hat: -1 },
            m::SkinRow { fk: 2, index: 6, like: 1, kirby_hat: 9 },
        ],
    };
    unsafe { m::SKINS = Some(reg) };
    assert_eq!(m::Skins_Like(1, 4), -1); // an original costume
    assert_eq!(m::Skins_Like(1, 5), 2);
    assert_eq!(m::Skins_Like(1, 6), 0); // like past the base count -> 0
    assert_eq!(m::Skins_Like(7, 5), -1); // unknown fighter
    assert_eq!(m::Skins_KirbyRow(1, 5), 3);
    assert_eq!(m::Skins_KirbyRow(1, 6), 0); // no declared hat: sk_like % 6
    assert_eq!(m::Skins_KirbyRow(2, 6), 1); // declared hat out of 0..=5: sk_like % 6
    assert_eq!(m::Skins_KirbyRow(1, 4), -1);
    // a skin costume copies its original costume's part visibility
    assert_eq!(m::Mex_CostumeVisIdx(1, 5), 2);
    unsafe { m::SKINS = None };
}
