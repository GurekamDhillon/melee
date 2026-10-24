//! src/tu/lb_0195.rs vs the C (gen/cref/tu/lb_0195.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_0195`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_0195 as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_0195_gen.rs"]
mod g;

unsafe extern "C" fn evfn() {
    t(7003);
}

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_0195_matches_c() {
    unsafe {
        g::install();
        reg(&raw const m::lb_804329F0);
        g::freg_all();
        freg(evfn as unsafe extern "C" fn() as usize);
        let mut c = Check::new(include_str!("data/mtu_lb_0195.txt"));
        for op in 0..11u32 {
            for k in 0..500u32 {
                let mut ret = 0u32;
                case(op, k);
                m::lb_804329F0 = core::mem::zeroed();
                g::fill_lb_804329F0_t(&mut m::lb_804329F0);
                let s = &raw mut m::lb_804329F0;
                if rnd() % 4 == 0 {
                    (*s).x38 = (*s).x0[0].x0 as u64;
                }
                if rnd() % 3 == 0 {
                    (*s).x0[1].x0 = (fv() as i64) * 1000;
                }
                if rnd() % 3 == 0 {
                    (*s).x0[0].x0 = (fv() as i64) * 1000;
                }
                if rnd() % 3 == 0 {
                    (*s).x38 = (fv() as u64) * 1000;
                }
                if rnd() % 3 == 0 {
                    (*s).x40 = (fv() as i64) * 5000;
                }
                (*s).x0[0].x10 = (fv() & 1) as i32;
                (*s).x0[1].x10 = (fv() & 1) as i32;
                match op {
                    0 => m::lb_8001955C(),
                    1 => m::lb_800195D0(),
                    2 => m::fn_800195FC(),
                    3 => m::lb_80019628(),
                    4 => {
                        let hi = fv() as u64;
                        let v = (hi << 32) | fv() as u64;
                        m::lb_80019880(v)
                    }
                    5 => ret = m::lb_80019894() as u32,
                    6 => m::lb_800198E0(),
                    7 => m::lb_80019900(),
                    8 => ret = m::lb_80019A30((rnd() & 1) as i32) as u32,
                    9 => m::lb_80019A48(),
                    _ => m::lb_80019AAC(Some(evfn)),
                }
                g::hash_lb_804329F0_t(&m::lb_804329F0);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 11 * 500);
    }
}
