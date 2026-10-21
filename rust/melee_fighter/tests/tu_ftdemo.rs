//! Smoke test for src/tu/ftdemo.rs (demo anim pair choice, motion-string callback, facing dir).
#![allow(static_mut_refs)]
use melee_fighter::tu::ftdemo as d;

fn cb(_u: i32, a: &mut i32, b: &mut i32) { *a = 7; *b = 8; }
fn kcb(kind: i32) -> Option<fn(i32, &mut i32, &mut i32)> { if kind == 1 { Some(cb) } else { None } }

#[test]
fn anim_pairs() {
    unsafe { d::ext::FTDATA_UNK_DEMO_CALLBACKS0 = Some(kcb) };
    assert_eq!(d::demo_anim_pair(0, 0), (0, 9));
    assert_eq!(d::demo_anim_pair(0, 5), (10, 11));
    assert_eq!(d::demo_anim_pair(0, 6), (10, 11));
    assert_eq!(d::demo_anim_pair(0, 7), (12, 12));
    assert_eq!(d::demo_anim_pair(0, 8), (13, 13));
    assert_eq!(d::demo_anim_pair(1, 9), (7, 8));
    assert_eq!(d::demo_anim_pair(2, 9), (0, 9));
}

#[test]
fn motion_string_null_panics() {
    fn none(_: i32) -> Option<fn(i32) -> *mut core::ffi::c_char> { None }
    unsafe { d::ext::FTDATA_803C24EC = Some(none) };
    assert!(std::panic::catch_unwind(|| d::ftdemo_get_motion_file_string(0, 1)).is_err());
}
