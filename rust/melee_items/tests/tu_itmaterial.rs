//! Smoke test for tu/itmaterial.rs: the static initialiser and the diffuse walk on an empty jobj.
use melee_items::tu::itmaterial as m;
use melee_types::*;

#[test]
fn itmaterial_smoke() {
    unsafe {
        m::it_mobj_static_init();
        assert_eq!(m::it_mobj.tevdesc_tmpl.flags, 1);
        assert_eq!(m::it_mobj.texp_tmpl.reg, 0xFF);
        // A gobj with no jobj tree: the walk must return at once.
        let mut g: HSD_GObj = core::mem::zeroed();
        let mut c = _GXColor { r: 1, g: 2, b: 3, a: 4 };
        m::it_80278574(&raw mut g, &raw mut c);
    }
}
