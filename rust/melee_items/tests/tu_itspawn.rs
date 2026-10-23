//! Smoke test for tu/itspawn.rs: the first-tier weighted total of the spawner over a small stage table.
use melee_items::tu::itspawn as m;

#[test]
fn itspawn_smoke() {
    unsafe {
        let mut counts = vec![0i32; 300];
        counts[0] = 3;
        counts[1] = 5;
        let mut table: melee_types::ItemPickTable = core::mem::zeroed();
        // kinds 0 and 1 are present with weights 3 and 5: 3.99 -> 3, then 3 + 5.99 -> 8.
        m::it_8026CA4C(&raw mut table, counts.as_mut_ptr(), 0b11, 0, 1.0);
        assert_eq!(table.x8, 8);
    }
}
