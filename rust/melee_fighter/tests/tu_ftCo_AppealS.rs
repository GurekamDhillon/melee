//! src/tu/ftCo_AppealS.rs vs the C (gen/cref/tu/ftCo_AppealS.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_AppealS`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_AppealS as m;
use m::ext::hook as hk;

static mut ND: usize = 0;
static mut DR: [usize; 4] = [0; 4];
static mut DBL: i32 = 0;

#[test]
fn ftCo_AppealS_matches_c() {
    init();
    af_init();
    unsafe {
        hk::ftCl_Init_80149318 = vrec!(1);
        hk::ftDr_Init_80149910 = vrec!(2);
        hk::lb_8000B1CC = Some(|j, p0, p1| { t(3); t(af_jobj(j)); t(p0.is_null() as u32); p1.x = rg(); p1.y = rg(); p1.z = rg(); });
        hk::lb_800119DC = Some(|p, n, a, bb, c| { t(4); t(b(p.x)); t(b(p.y)); t(b(p.z)); t(n as u32); t(b(a)); t(b(bb)); t(b(c)); });
        hk::pl_80040120 = Some(|a, bb| { t(5); t(a as u32); t(bb as u32); });
        hk::ftData_80085FD4 = Some(|fr, ms| { t(6); t((fr as *mut _ == F) as u32); t(ms as u32); let r = DR[ND & 3]; ND += 1; r });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::DbLevel = Some(|| DBL);
        hk::ftPe_Init_8011B93C = vrec!(8);
        hk::ftZd_Init_801395C8 = vrec!(9);
        hk::ftKb_MexKirbyLike = Some(|k| { t(10); t(k as u32); chk(10) as i32 });
        hk::ftKb_SpecialN_800F5D04 = Some(|g, bb| { t(11); t(is_go(g)); t(bb as u32); });
        hk::Mex_OnTauntDispatch = Some(|k, g, v| { t(12); t(k as u32); t(is_go(g)); t(v.is_null() as u32); });
        hk::ftAnim_IsFramesRemaining = chkrec!(13);
        hk::ft_8008A2BC = vrec!(14);
        hk::ftCo_SpecialS_CheckInput = chkrec!(15);
        hk::ftCo_Attack100_CheckInput = chkrec!(16);
        hk::ftCo_800D6824 = chkrec!(17);
        hk::ftCo_800D68C0 = chkrec!(18);
        hk::ftCo_Catch_CheckInput = chkrec!(19);
        hk::ftCo_AttackS4_CheckInput = chkrec!(20);
        hk::ftCo_AttackHi4_CheckInput = chkrec!(21);
        hk::ftCo_AttackLw4_CheckInput = chkrec!(22);
        hk::ftCo_AttackS3_CheckInput = chkrec!(23);
        hk::ftCo_AttackHi3_CheckInput = chkrec!(24);
        hk::ftCo_AttackLw3_CheckInput = chkrec!(25);
        hk::ftCo_Attack1_CheckInput = chkrec!(26);
        hk::ftCo_80099794 = chkrec!(27);
        hk::ftCo_80091A4C = chkrec!(28);
        hk::ft_80084FA8 = vrec!(29);
        hk::ft_80084104 = vrec!(30);
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_AppealS.txt"));
    for op in 0..9u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                ND = 0;
                for i in 0..4 { DR[i] = (SRV[4 + i] & 1) as usize; }
            }
            af_fill();
            unsafe { DBL = (rnd() % 5) as i32; }
            let m0 = (rnd() % 400) as i32;
            let m1 = (rnd() % 400) as i32;
            let g = go();
            let ret = match op {
                0 => m::ftCo_800DE9B8(g) as u32,
                1 => m::ftCo_800DE9D8(g) as u32,
                2 => { m::ftCo_800DEA28(g); 0 }
                3 => { m::ftCo_800DEAE8(g, m0, m1); 0 }
                4 => { m::ftCo_800DEBD0(g); 0 }
                5 => { m::ftCo_AppealS_Anim(g); 0 }
                6 => { m::ftCo_AppealS_IASA(g); 0 }
                7 => { m::ftCo_AppealS_Phys(g); 0 }
                _ => { m::ftCo_AppealS_Coll(g); 0 }
            };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 18000);
}
