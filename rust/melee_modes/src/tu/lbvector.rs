//! lbvector.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::Vec as Vec3;

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lbvector.c `lbVector_Mirror_fp`: reflect `a` in the XY plane about a unit axis (3 fused sites).
pub fn lbVector_Mirror_fp(a: &mut Vec3, unit_mirror_axis: &Vec3) {
    melee_core::ct!("lbVector_Mirror_fp");
    let m = unit_mirror_axis;
    let f = fmadds(m.x, a.x, m.y * a.y) * -2.0;
    a.x = fmadds(m.x, f, a.x);
    a.y = fmadds(m.y, f, a.y);
}
