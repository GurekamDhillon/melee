//! src/tu/if_2FF2.rs vs the C (gen/mtu/tu/if_2FF2.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py if_2FF2`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::if_2FF2 as m;
use mtu_harness::*;
#[path = "mtu/if_2FF2_gen.rs"]
mod g;

static mut G: [HSD_GObj; 6] = unsafe { core::mem::zeroed() };
static mut NG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut HG: HSD_GObj = unsafe { core::mem::zeroed() };
static mut TX: [HSD_Text; 3] = unsafe { core::mem::zeroed() };
static mut HPOS: [Vec; 6] = [Vec { x: 0.0, y: 0.0, z: 0.0 }; 6];
static mut LO: HSD_LObj = unsafe { core::mem::zeroed() };
static mut CO: HSD_CObj = unsafe { core::mem::zeroed() };
static mut PROC0: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut ME: MatchEnd = unsafe { core::mem::zeroed() };

unsafe fn setup() {
    G = core::mem::zeroed();
    NG = core::mem::zeroed();
    HG = core::mem::zeroed();
    TX = core::mem::zeroed();
    PROC0 = core::mem::zeroed();
    for i in 0..6 {
        g::fill_HSD_GObj(&mut G[i]);
    }
    for i in 0..4 {
        g::fill_HSD_GObj(&mut NG[i]);
    }
    g::fill_HSD_GObj(&mut HG);
    for i in 0..3 {
        g::fill_HSD_Text(&mut TX[i]);
    }
    for i in 0..6 {
        HPOS[i].x = rw();
        HPOS[i].y = rw();
        HPOS[i].z = rw();
    }
    m::un_804A1F58 = core::mem::zeroed();
    m::un_804A1F58.x0 = fv();
    for i in 0..6usize {
        m::un_804A1F58.x8[i].x0 = if rnd() % 3 == 0 { null_mut() } else { &raw mut G[(rnd() % 6) as usize] };
        m::un_804A1F58.x8[i].x4 = if rnd() % 3 == 0 { null_mut() } else { &raw mut TX[(rnd() % 3) as usize] };
        m::un_804A1F58.x8[i].x8 = fv();
        m::un_804A1F58.x8[i].xC = fv();
        m::un_804A1F58.x8[i].x10 = (rnd() % 3) as u8;
    }
    m::ext::HSD_GObj_CameraKind = (rnd() % 5) as u8;
    m::ext::HSD_GObj_LightKind = (rnd() % 5) as i8;
}

unsafe fn hash_all() {
    for i in 0..6 {
        g::hash_HSD_GObj(&G[i]);
    }
    for i in 0..4 {
        g::hash_HSD_GObj(&NG[i]);
    }
    g::hash_HSD_GObj(&HG);
    for i in 0..3 {
        g::hash_HSD_Text(&TX[i]);
    }
    hm(m::un_804A1F58.x0);
    for i in 0..6 {
        hm(pi(m::un_804A1F58.x8[i].x0));
        hm(pi(m::un_804A1F58.x8[i].x4));
        hm(m::un_804A1F58.x8[i].x8);
        hm(m::un_804A1F58.x8[i].xC);
        hm(m::un_804A1F58.x8[i].x10 as u32);
    }
}

#[test]
fn if_2FF2_matches_c() {
    unsafe {
        g::install();
        for i in 0..6 {
            preg(&raw const G[i], core::mem::size_of::<HSD_GObj>());
        }
        for i in 0..4 {
            preg(&raw const NG[i], core::mem::size_of::<HSD_GObj>());
        }
        preg(&raw const HG, core::mem::size_of::<HSD_GObj>());
        preg(&raw const TX, core::mem::size_of_val(&TX));
        preg(&raw const HPOS, core::mem::size_of_val(&HPOS));
        preg(&raw const LO, core::mem::size_of::<HSD_LObj>());
        preg(&raw const CO, core::mem::size_of::<HSD_CObj>());
        preg(&raw const PROC0, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const ME, core::mem::size_of::<MatchEnd>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_if_2FF2.txt"));
        for op in 0..10u32 {
            for k in 0..1500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {
                        let gg = if rnd() % 4 == 0 { &raw mut NG[0] } else { &raw mut G[(rnd() % 6) as usize] };
                        m::fn_802FF218(gg);
                    }
                    1 => m::fn_802FF360(null_mut()),
                    2 => m::un_802FF364((rnd() % 6) as i32),
                    3 => m::un_802FF498(),
                    4 => m::un_802FF4FC(),
                    5 => m::un_802FF570(),
                    6 => m::un_802FF620(),
                    7 => m::un_802FF6A0(),
                    8 => m::un_802FF710(),
                    _ => ret = pi(m::un_802FF78C()),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 10 * 1500);
    }
}
