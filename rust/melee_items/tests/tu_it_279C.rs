//! Smoke test for tu/it_279C.rs: the attack-line zeroing of the Pokemon spawn state.
use melee_items::tu::it_279C as m;

#[test]
fn it_279c_smoke() {
    unsafe {
        let mut item: melee_types::Item = core::mem::zeroed();
        item.xDD4_itemVar.pokemon_spawn.x4.x = 5.0;
        let mut g: melee_types::HSD_GObj = core::mem::zeroed();
        g.user_data = &raw mut item as *mut core::ffi::c_void;
        m::it_8027A13C(&raw mut g);
        assert_eq!(item.xDD4_itemVar.pokemon_spawn.x4.x, 0.0);
        assert_eq!(m::Pokemon_ID_Metamon, 19);
    }
}
