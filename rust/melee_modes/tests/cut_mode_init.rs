//! tu::gmmode_init vs the C text itself (gen/cref/cut/mode_init.c; regenerate with
//! `python rust/melee_modes/gen/cref/cutref.py mode_init`).
#![allow(static_mut_refs)]
mod mtu_harness;
use melee_modes::tu::gmmain_lib::{gmMainLib_804D3EE0, gmMainLib_8045A6C0, gmMainLib_8015CDD4, gmMainLib_8015CDE0};
use melee_modes::tu::gmmode_init as m;
use mtu_harness::*;

fn hex(p: *const u8) -> String {
    (0..6).map(|i| format!("{:02X}", unsafe { *p.add(i) })).collect()
}

#[test]
fn mode_init_matches_c() {
    unsafe {
        gmMainLib_804D3EE0 = &raw mut gmMainLib_8045A6C0;
        let r0 = gmMainLib_8015CDD4() as *mut u8;
        let r1 = gmMainLib_8015CDE0() as *mut u8;
        assert_eq!(r0.add(6), r1, "the two settings records are adjacent, as the C driver assumes");
        let mut n = 0;
        let mut lines = include_str!("data/cut_mode_init.txt").lines();
        for op in 0..2 {
            for k in 0..20 {
                case(op, k);
                for i in 0..12 {
                    *r0.add(i) = fv() as u8;
                }
                if op == 0 { m::gm_Mode_Adventure_OnInit() } else { m::gm_Mode_AllStar_OnInit() }
                assert_eq!(lines.next().unwrap(), format!("{} {} {} {}", op, k, hex(r0), hex(r1)));
                n += 1;
            }
        }
        assert_eq!(n, 40);
    }
}
