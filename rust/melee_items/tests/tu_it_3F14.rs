//! Smoke test for tu/it_3F14.rs: the globals start null.
use melee_items::tu::it_3F14 as m;

#[test]
fn it_3f14_smoke() {
    unsafe {
        assert!(m::it_804D6D40.is_null());
        assert!(m::it_804D6D38.is_null());
        assert!(m::it_804D6D30.is_null());
    }
}
