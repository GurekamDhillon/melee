//! src/tu/lb_0192.rs vs the C (gen/cref/tu/lb_0192.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_0192`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_0192 as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_0192_gen.rs"]
mod g;

static mut TXT: HSD_Text = unsafe { core::mem::zeroed() };
static DVDT: [i32; 16] = [5, 4, 6, 11, -1, 1, 0, 2, 3, 7, 8, 9, 10, 12, 13, 5];

unsafe extern "C" fn cbfn() {
    t(7002);
}

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_0192_matches_c() {
    unsafe {
        g::install();
        reg(&raw const TXT);
        reg(&raw const m::ext::gmMainLib_8046B0F0);
        g::freg_all();
        freg(cbfn as unsafe extern "C" fn() as usize);
        let mut c = Check::new(include_str!("data/mtu_lb_0192.txt"));
        for op in 0..2u32 {
            for k in 0..600u32 {
                let mut ret = 0u32;
                case(op, k);
                TXT = core::mem::zeroed();
                g::fill_HSD_Text(&mut TXT);
                g::fill_gmMainLib_8046B0F0_t(&mut m::ext::gmMainLib_8046B0F0);
                m::lb_804D63D0 = core::ptr::null_mut();
                if op == 0 {
                    ret = m::lb_80019230() as u32;
                } else {
                    let cb: Option<unsafe extern "C" fn()> = if rnd() & 1 != 0 { Some(cbfn) } else { None };
                    m::lb_800192A8(cb);
                }
                g::hash_HSD_Text(&TXT);
                g::hash_gmMainLib_8046B0F0_t(&m::ext::gmMainLib_8046B0F0);
                hm(pi(m::lb_804D63D0));
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 2 * 600);
    }
}
