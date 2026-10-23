//! Smoke test for tu/it_2725.rs: the null-gobj classifier and the scalar clamp of a zero-hitbox item.
use melee_items::tu::it_2725 as m;

#[test]
fn it_2725_smoke() {
    unsafe {
        assert!(!m::it_80272D1C(core::ptr::null_mut()));
        let mut item: melee_types::Item = core::mem::zeroed();
        let mut g: melee_types::HSD_GObj = core::mem::zeroed();
        g.user_data = &raw mut item as *mut core::ffi::c_void;
        m::it_802753BC(&raw mut g, 7);
        assert_eq!(item.xC9C, 7);
    }
}
