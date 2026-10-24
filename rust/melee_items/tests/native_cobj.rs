//! src/tu/cobj.rs (pointer-level HSD_CObj projection accessors) against `melee_runtime::cobj`, the model port
//! that is checked against the C by melee_runtime's oracle_cobj. Random projection types and parameters; bit equality.
#![allow(non_snake_case)]
use melee_items::tu::cobj as n;
use melee_runtime::cobj as m;
use melee_types::HSD_CObj;

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        self.0 >> 8
    }
    fn f(&mut self) -> f32 {
        ((self.next() % 40000) as i32 - 20000) as f32 / 64.0
    }
}

#[test]
fn cobj_accessors_match_model() {
    let mut r = Rng(7);
    for _ in 0..5000 {
        let ty = (r.next() % 5) as u8;
        let p = [r.f(), r.f(), r.f(), r.f()];
        let near = r.f();
        let mut mo: m::HsdCObj<u32> = Default::default();
        mo.projection_type = ty;
        mo.projection_param = p;
        mo.near = near;
        let mut c: HSD_CObj = HSD_CObj::zeroed();
        c.projection_type = ty;
        unsafe {
            let q = &mut c.projection_param.frustum;
            q.top = p[0];
            q.bottom = p[1];
            q.left = p[2];
            q.right = p[3];
        }
        c.near = near;
        let cp = &mut c as *mut HSD_CObj;
        unsafe {
            assert_eq!(n::HSD_CObjGetFov(cp).to_bits(), m::get_fov(Some(&mo)).to_bits());
            assert_eq!(n::HSD_CObjGetTop(cp).to_bits(), m::get_top(Some(&mo)).to_bits());
            assert_eq!(n::HSD_CObjGetBottom(cp).to_bits(), m::get_bottom(Some(&mo)).to_bits());
            assert_eq!(n::HSD_CObjGetLeft(cp).to_bits(), m::get_left(Some(&mut mo), 1.0).to_bits());
            assert_eq!(n::HSD_CObjGetRight(cp).to_bits(), m::get_right(Some(&mut mo), 1.0).to_bits());
            let v = r.f();
            n::HSD_CObjSetFov(cp, v);
            m::set_fov(Some(&mut mo), v);
            n::HSD_CObjSetTop(cp, v + 1.0);
            m::set_top(Some(&mut mo), v + 1.0);
            n::HSD_CObjSetBottom(cp, v + 2.0);
            m::set_bottom(Some(&mut mo), v + 2.0);
            n::HSD_CObjSetLeft(cp, v + 3.0);
            m::set_left(Some(&mut mo), v + 3.0);
            n::HSD_CObjSetRight(cp, v + 4.0);
            m::set_right(Some(&mut mo), v + 4.0);
            let q = c.projection_param.frustum;
            let nv = [q.top, q.bottom, q.left, q.right];
            for i in 0..4 {
                assert_eq!(nv[i].to_bits(), mo.projection_param[i].to_bits(), "type {ty} slot {i}");
            }
        }
    }
}
