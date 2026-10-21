//! Smoke test for src/tu/ftlib.rs (the five ftlib.c functions not in crate::ftlib).
#![allow(static_mut_refs)]
use melee_fighter::tu::ftlib as l;
use melee_fighter::types::*;
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

static LOADS: AtomicU32 = AtomicU32::new(0);

#[test]
fn floor_angle_and_loaders() {
    let mut f: Box<Fighter> = Box::new(unsafe { core::mem::zeroed() });
    let mut go: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    go.user_data = &mut *f as *mut Fighter as *mut _;
    let g = &mut *go as *mut HSD_GObj;
    f.ground_or_air = GA_Ground;
    f.coll_data.floor.normal = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    assert_eq!(l::ftlib_8008777c(g).to_bits(), melee_runtime::lbtrigf::atan2f(-0.0, 1.0).to_bits());
    f.coll_data.floor.normal = Vec3 { x: -0.5, y: 0.8, z: 0.0 };
    assert_eq!(l::ftlib_8008777c(g), melee_runtime::lbtrigf::atan2f(0.5, 0.8));
    f.ground_or_air = GA_Air;
    assert!(std::panic::catch_unwind(|| l::ftlib_8008777c(g)).is_err());

    unsafe {
        l::ext::FTDATA_8008572C = Some(|_| {});
        l::ext::EFASYNC_LOAD_SYNC = Some(|_| {});
        l::ext::FTDATA_80085820 = Some(|_, _| { LOADS.fetch_add(1, SeqCst); });
        l::ext::FTDATA_80085A14 = Some(|_| {});
        l::ext::FTDATA_800857E0 = Some(|_| {});
        l::ext::FTDATA_UNK_BYTE = Some(|k| k);
        l::ext::COSTUME_COUNT = Some(|_| 4);
    }
    l::ftlib_80087574(2);
    assert_eq!(LOADS.load(SeqCst), 4);
    l::ftlib_80087508(2, 1);
    assert_eq!(LOADS.load(SeqCst), 5);
}
