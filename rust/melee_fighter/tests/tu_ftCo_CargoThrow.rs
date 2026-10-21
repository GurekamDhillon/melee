//! src/tu/ftCo_CargoThrow.rs vs the C (gen/cref/tu/ftCo_CargoThrow.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_CargoThrow`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
mod tu_ftco_cargo;
use tu_ftco_harness::*;
use tu_ftco_cargo::*;
use melee_fighter::tu::ftCo_CargoThrow as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(pi(g)) }); )* } } }

#[test]
fn ftCo_CargoThrow_matches_c() {
    cargo_init();
    v!(ftCo_8008F744 = 1, ftCo_8009BC58 = 2, ftCo_8009B518 = 3, ftCo_Damage_Phys = 4, ftAnim_8006EBA4 = 10,
       ftCo_800DD724 = 13, ftCommon_8007D92C = 15, ftCo_ThrowF_IASA = 16, ftCo_ThrowF_Phys = 17,
       ftCo_ThrowB_Anim = 18, ftCo_ThrowB_IASA = 19, ftCo_ThrowB_Phys = 20, ftCo_ThrowHi_Anim = 21,
       ftCo_ThrowHi_IASA = 22, ftCo_ThrowHi_Phys = 23, ftCo_ThrowLw_Anim = 24, ftCo_ThrowLw_IASA = 25,
       ftCo_ThrowLw_Phys = 26, ft_80084DB0 = 28);
    unsafe {
        hk::ft_80082708 = Some(|g| { t(5); t(pi(g)); SRV[5] % 2 });
        hk::ft_80081D0C = Some(|g| { t(6); t(pi(g)); SRV[6] % 2 });
        hk::ftCommon_8007D5D4 = Some(|f| { t(7); t(pi(f as *const Fighter)) });
        hk::ftCommon_8007D7FC = Some(|f| { t(8); t(pi(f as *const Fighter)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(9); t(pi(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(pi(o))
        });
        hk::ftCommon_8007E2F4 = Some(|f, v| { t(11); t(pi(f as *const Fighter)); t(v as i32 as u32) });
        hk::ftCo_800DE3FC = Some(|g, ms, s| { t(12); t(pi(g)); t(ms as u32); t(b(s)) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(14); t(pi(g)); sr(14) });
        hk::ft_8008403C = Some(|g, cb| { t(27); t(pi(g)); t((cb == Some(m::ftCo_8009C170_cb as unsafe extern "C" fn(*mut HSD_GObj))) as u32) });
        hk::ft_80082C74 = Some(|g, cb| { t(29); t(pi(g)); t((cb == Some(m::ftCo_8009C45C_cb as unsafe extern "C" fn(*mut HSD_GObj))) as u32) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_CargoThrow.txt"));
    for op in 0..27u32 {
        for k in 0..1000u32 {
            case(op, k);
            cargo_fill();
            let g = go();
            let ret = match op {
                0 => { m::ftCo_CargoWait2_Anim(g); 0 }
                1 => { m::ftCo_CargoWait2_Phys(g); 0 }
                2 => { m::ftCo_CargoWait2_Coll(g); 0 }
                3 => m::ftCo_8009BF3C(g) as u32,
                4 => { m::ftCo_8009C02C(g, (rnd() % 400) as i32); 0 }
                5 => { m::ftCo_CargoThrowF_Anim(g); 0 }
                6 => { m::ftCo_CargoThrowF_IASA(g); 0 }
                7 => { m::ftCo_CargoThrowF_Phys(g); 0 }
                8 => { m::ftCo_8009C170(g); 0 }
                9 => { m::ftCo_CargoThrowF_Coll(g); 0 }
                10 => { m::ftCo_CargoThrowB_Anim(g); 0 }
                11 => { m::ftCo_CargoThrowB_IASA(g); 0 }
                12 => { m::ftCo_CargoThrowB_Phys(g); 0 }
                13 => { m::ftCo_CargoThrowB_Coll(g); 0 }
                14 => { m::ftCo_CargoThrowHi_Anim(g); 0 }
                15 => { m::ftCo_CargoThrowHi_IASA(g); 0 }
                16 => { m::ftCo_CargoThrowHi_Phys(g); 0 }
                17 => { m::ftCo_CargoThrowHi_Coll(g); 0 }
                18 => { m::ftCo_CargoThrowLw_Anim(g); 0 }
                19 => { m::ftCo_CargoThrowLw_IASA(g); 0 }
                20 => { m::ftCo_CargoThrowLw_Phys(g); 0 }
                21 => { m::ftCo_CargoThrowLw_Coll(g); 0 }
                22 => { m::ftCo_CargoThrowAir_Anim(g); 0 }
                23 => { m::ftCo_CargoThrowAir_IASA(g); 0 }
                24 => { m::ftCo_CargoThrowAir_Phys(g); 0 }
                25 => { m::ftCo_CargoThrowAir_Coll(g); 0 }
                _ => { m::ftCo_8009C45C(g); 0 }
            };
            c.line(op, k, ret, state_hash());
        }
    }
    assert_eq!(c.done(), 27000);
}
