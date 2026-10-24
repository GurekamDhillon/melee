//! src/tu/lb_0146.rs vs the C (gen/cref/tu/lb_0146.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_0146`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_0146 as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_0146_gen.rs"]
mod g;

static mut COB: HSD_CObj = unsafe { core::mem::zeroed() };
static mut A0: lb_80014638_arg0_t = unsafe { core::mem::zeroed() };
static mut A1: lb_80014638_arg1_t = unsafe { core::mem::zeroed() };
static mut V: [Vec3; 2] = unsafe { core::mem::zeroed() };
static mut M: [[f32; 4]; 3] = [[0.0; 4]; 3];

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_0146_matches_c() {
    unsafe {
        g::install();
        reg(&raw const COB);
        reg(&raw const A0);
        reg(&raw const A1);
        reg(&raw const V);
        reg(&raw const M);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_lb_0146.txt"));
        for op in 0..3u32 {
            for k in 0..600u32 {
                let mut ret = 0u32;
                case(op, k);
                COB = core::mem::zeroed();
                g::fill_lb_80014638_arg0_t(&mut A0);
                g::fill_lb_80014638_arg1_t(&mut A1);
                g::fill_Vec3(&mut V[0]);
                g::fill_Vec3(&mut V[1]);
                for i in 0..3 {
                    for j in 0..4 {
                        M[i][j] = rg();
                    }
                }
                match op {
                    0 => {
                        if rnd() & 1 != 0 {
                            A1.x8.z = A0.x0.z + 0.5;
                            A0.xC.z = A0.x0.z;
                        }
                        ret = m::lb_80014638(&A0, &A1) as u32;
                    }
                    1 => {
                        let a1 = if rnd() & 1 != 0 { 2 } else { (fv() % 4) as i32 };
                        ret = m::lb_80014770(V.as_mut_ptr(), a1) as u32;
                    }
                    _ => {
                        let a1 = if rnd() & 1 != 0 { 0 } else { fv() % 3 };
                        ret = m::lb_800149E0(M.as_mut_ptr(), a1) as u32;
                    }
                }
                g::hash_lb_80014638_arg0_t(&A0);
                g::hash_lb_80014638_arg1_t(&A1);
                g::hash_Vec3(&V[0]);
                g::hash_Vec3(&V[1]);
                for i in 0..3 {
                    for j in 0..4 {
                        hm(M[i][j].to_bits());
                    }
                }
                for col in [m::lb_ColorWhite, m::lb_ColorBlack, m::red, m::translucent_red, m::yellow] {
                    hm(u32::from_le_bytes([col.r, col.g, col.b, col.a]));
                }
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 3 * 600);
    }
}
