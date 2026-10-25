//! tu::gmmain_lib (trophy getters, MexApplyTournamentDefaults, gmMainLib_8015FCC0, gmMainLib_8015F600) vs
//! the C (gen/cref/tu/gmmain_lib.c: the whole TU, shipping contraction; regenerate with
//! `python rust/melee_modes/gen/cref/run_tu.py gmmain_lib`). The port's `ext` is not the generated
//! shape, so the recording hooks are installed here by hand with the C stubs' ids (see the
//! `// stub` list at the end of gen/cref/tu/gmmain_lib_gen.h).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals)]
mod mtu_harness;
use melee_modes::tu::gmmain_lib as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmmain_lib_gen.rs"]
mod g;

pub static mut NAMES: [[u8; 8]; 4] = [[0; 8]; 4];

fn shs(s: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in s.bytes() {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    h
}

fn install() {
    use m::ext::*;
    unsafe {
        OSGETRESETCODE = Some(|| { t(1007); if fv() & 1 != 0 { 0x8000_0000 } else { fv() } });
        LBTIME_GETTIMEINSECONDS = Some(|| { t(1033); fv() });
        MEX_ENABLED = Some(|n| { t(1003); t(shs(n)); (fv() & 1) as i32 });
        TOY_80311960 = Some(|| t(1011));
        TOY_803124BC = Some(|| t(1012));
        TOY_80305058 = Some(|a, b, c, d| { t(1010); t(a as u32); t(b as u32); t(c as u32); t(d.to_bits()); fv() as i16 });
        TOY_SETUNLOCKSTATE = Some(|a, b| { t(1013); t(a as u32); t(b as u32); });
        TOY_803048C0 = Some(|a| { t(1009); t(a as u32); fv() as i32 });
        GM_801623FC = Some(|a| { t(1014); t(a as u32); });
        GM_80164430 = Some(|a| { t(1015); t(a); fv() as i32 });
        GM_80164504 = Some(|a| { t(1016); t(a); });
        GM_INCREMENTPOWERCOUNT = Some(|| t(1027));
        LBLANG_GETLANGUAGESETTING = Some(|| { t(1029); fv() as i32 });
        LBLANG_SETSAVEDLANGUAGE = Some(|a| { t(1031); t(a as u32); });
        MNNAME_8023749C = Some(|a| {
            t(1035);
            t(a as u32);
            if fv() & 1 != 0 { core::ptr::null() } else { NAMES[(fv() % 4) as usize].as_ptr() as *const i8 }
        });
    }
}

unsafe fn fill_all() {
    g::fill_gmm_x0(&mut m::gmMainLib_8045A6C0);
    g::fill_GameRules(&mut m::gmMainLib_DefaultGameRules);
    g::fill_GamePrefs(&mut m::gmMainLib_DefaultGamePrefs);
    g::fill_gmMainLib_8046B0F0_t(&mut m::gmMainLib_8046B0F0);
    for i in 0..4 {
        for j in 0..7 {
            NAMES[i][j] = (1 + fv() % 100) as u8;
        }
        NAMES[i][(fv() % 7) as usize] = 0;
        NAMES[i][7] = 0;
    }
}

unsafe fn hash_all() {
    g::hash_gmm_x0(&m::gmMainLib_8045A6C0);
    g::hash_GameRules(&m::gmMainLib_DefaultGameRules);
    g::hash_GamePrefs(&m::gmMainLib_DefaultGamePrefs);
    g::hash_gmMainLib_8046B0F0_t(&m::gmMainLib_8046B0F0);
}

#[test]
fn gmmain_lib_matches_c() {
    unsafe {
        m::gmMainLib_init_statics();
        install();
        preg(&raw const m::gmMainLib_8045A6C0, core::mem::size_of::<gmm_x0>());
        preg(&raw const m::gmMainLib_DefaultGameRules, core::mem::size_of::<GameRules>());
        preg(&raw const m::gmMainLib_DefaultGamePrefs, core::mem::size_of::<GamePrefs>());
        preg(&raw const m::gmMainLib_8046B0F0, core::mem::size_of::<gmMainLib_8046B0F0_t>());
        preg(&raw const NAMES, 32);
        let mut c = Check::new(include_str!("data/mtu_gmmain_lib.txt"));
        for op in 0..6u32 {
            for k in 0..800u32 {
                case(op, k);
                fill_all();
                let mut ret = 0u32;
                match op {
                    0 => ret = pi(m::gmMainLib_GetTrophyFlags()),
                    1 => ret = pi(m::gmMainLib_GetTrophyCategoryFlags()),
                    2 => ret = pi(m::gmMainLib_GetTrophyCount()),
                    3 => m::gmMainLib_MexApplyTournamentDefaults(),
                    4 => m::gmMainLib_8015FCC0(),
                    _ => m::gmMainLib_8015F600(1 + (k % 8) as i32, ((k / 8) & 1) as i32),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4800);
    }
}
