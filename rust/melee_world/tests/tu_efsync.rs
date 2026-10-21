//! Smoke tests for tu/efsync.rs: the burst velocity and the 0x4C3-style routes that only need
//! the anim-queue drain (no ids reach baselib here).
use melee_runtime::gekko::{cosf, sinf};
use melee_world::tu::efsync as m;

#[test]
fn burst_params_match_c_statement_order() {
    for &(rx, ry) in &[(0.3f32, 1.2f32), (5.9, 0.01), (3.14159, 2.5)] {
        let p = m::burst_params(rx, ry);
        assert_eq!(p.x.to_bits(), ((2.0f32 * cosf(rx)) * sinf(ry)).to_bits());
        assert_eq!(p.y.to_bits(), (2.0f32 * sinf(rx)).to_bits());
        assert_eq!(p.z.to_bits(), ((2.0f32 * cosf(rx)) * cosf(ry)).to_bits());
    }
}

#[test]
fn unknown_sync_id_returns_null() {
    let mut va = m::Va::new(&[]);
    // 0x4F0..: an id with no case (0x513) falls through the switch to NULL.
    let r = unsafe { m::efSync_Spawn(0x513, core::ptr::null_mut(), &mut va) };
    assert!(r.is_null());
}
