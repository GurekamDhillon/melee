//! tu::gm_16A2 vs the C (gen/cref/tu/gm_16A2.c: the whole TU, shipping contraction, 4-byte pointers;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gm_16A2`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_gm_16A2_hs;
mod tu_gm_16A2_stubs;
mod tu_gm_harness;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::tu::gm_16A2 as m;
use melee_types::*;
use tu_gm_16A2_hs::*;
use tu_gm_harness::*;

static mut VSS: VsSceneState = unsafe { core::mem::zeroed() };
static mut SMR: StartMeleeRules = unsafe { core::mem::zeroed() };
static mut BUFA: [i8; 320] = [0; 320];
static mut BUFB: [i8; 320] = [0; 320];
static mut BUFC: [i8; 16] = [0; 16];

unsafe extern "C" fn cbR(a: i32) -> i32 {
    t(950);
    t(a as u32);
    sr(2) as i32
}
unsafe extern "C" fn cbE(slot: i32, rem: u8) {
    t(951);
    t(slot as u32);
    t(rem as u32);
}

fn install_manual() {
    use m::ext::hook as h;
    unsafe {
        h::HSD_Randi = Some(|max| {
            t(900);
            t(max as u32);
            if max > 0 {
                (rnd() % max as u32) as i32
            } else {
                0
            }
        });
        h::gm_GetNumCostumesForCKind = Some(|ck| {
            t(901);
            t(ck as u32);
            (1 + rnd() % 9) as u8
        });
        h::memzero = Some(|p, n| {
            t(902);
            t(n as u32);
            core::ptr::write_bytes(p as *mut u8, 0, n as usize);
        });
        h::fn_8016AE60 = Some(|| {
            t(903);
            addr_of_mut!(m::lbl_8046B488)
        });
        h::gmVs_GetSceneState = Some(|| {
            t(904);
            addr_of_mut!(VSS)
        });
        h::Ground_801C2D24 = Some(|i, v| {
            t(905);
            t(i as u32);
            (*v).x = rg();
            (*v).y = rg();
            (*v).z = rg();
            rnd() & 1 != 0
        });
        h::Player_SetStructFunc = Some(|slot, f| {
            t(906);
            t(slot as u32);
            t((f == m::fn_8016A488 as *mut c_void) as u32);
        });
        h::gm_GetStartMeleeRules = Some(|| {
            t(907);
            addr_of_mut!(SMR)
        });
    }
}

unsafe fn state_hash() -> u32 {
    let mut h = 0;
    h = hs_lbl_8046B488_t(&m::lbl_8046B488, h);
    h = hs_lbl_8046B668_t(&m::lbl_8046B668, h);
    h = hs_VsSceneState(&VSS, h);
    h = hs_StartMeleeRules(&SMR, h);
    h = hm(h, hbuf(BUFA.as_ptr() as *const u8, 320));
    h = hm(h, hbuf(BUFB.as_ptr() as *const u8, 320));
    h = hm(h, hbuf(BUFC.as_ptr() as *const u8, 16));
    h = hm(h, m::EVENT_PLAYER_INIT_CB.is_some() as u32);
    h
}

unsafe fn mklist(b: *mut i8, n: usize, range: u32, special: i32) {
    for i in 0..n {
        *b.add(i) = if rnd() % 4 == 0 && special >= 0 { special as i8 } else { (rnd() % range) as i8 };
    }
    *b.add(n) = -2;
}

#[test]
fn gm_16A2_matches_c() {
    tu_gm_16A2_stubs::install();
    install_manual();
    let mut c = Check::new(include_str!("data/tu_gm_16A2.txt"));
    let lbl = unsafe { addr_of_mut!(m::lbl_8046B488) };
    let l2 = unsafe { addr_of_mut!(m::lbl_8046B668) };
    let bufa = unsafe { BUFA.as_mut_ptr() };
    let bufb = unsafe { BUFB.as_mut_ptr() };
    for op in 1..=41u32 {
        for k in 0..700u32 {
            case(op, k);
            let mut ret = 0u32;
            unsafe {
                zero(&mut *lbl);
                zero(&mut *l2);
                zero(&mut VSS);
                zero(&mut SMR);
                zero(&mut BUFA);
                zero(&mut BUFB);
                zero(&mut BUFC);
                m::EVENT_PLAYER_INIT_CB = None;
                fl_lbl_8046B488_t(&mut *lbl);
                fl_lbl_8046B668_t(&mut *l2);
                fl_VsSceneState(&mut VSS);
                fl_StartMeleeRules(&mut SMR);
                (*lbl).pad_1BC = [0; 4]; // the C's callback slot
                VSS.hud_enabled = (rnd() & 1) as u8;
                (*lbl).x20[(rnd() % 100) as usize] = -2;
                for i in 0..6 {
                    (*lbl).x1A6[i] = (rnd() % 100) as i8;
                }
                (*lbl).x7 = (rnd() % 6) as u8;
                (*lbl).x8 = (rnd() % 4) as u8;
                (*lbl).x9 = (rnd() % 8) as u8;
                (*lbl).xB = (rnd() & 1) as u8;
                (*lbl).xE = (rnd() & 1) as u8;
                for i in 0..3 {
                    let v: i8 = if rnd() % 3 == 0 {
                        4
                    } else if rnd() % 5 == 0 {
                        0x21
                    } else {
                        (rnd() % 40) as i8
                    };
                    *(addr_of_mut!((*lbl).x0) as *mut i8).add(i) = v;
                }
                (*l2).arr2[(rnd() % 27) as usize] = -2;
                for i in 0..28 {
                    (*l2).arr1[i] = (rnd() % 30) as i8;
                }
                match op {
                    1 => t((m::gm_1601_GetUnkData() == lbl) as u32),
                    2 => {
                        let mut a = (rnd() % 0x1B0) as i32;
                        if a == 0x10 {
                            a = 0x0F;
                        }
                        ret = m::gm_80169370(a) as u8 as u32
                    }
                    3 => ret = m::gm_80169384() as u32,
                    4 => ret = m::gm_80169394() as u32,
                    5 => ret = m::fn_801693A8() as u32,
                    6 => {
                        let a = (rnd() % 6) as i32;
                        ret = m::gm_801693BC(a) as u32
                    }
                    7 => m::fn_80169434(if rnd() & 1 != 0 { Some(cbR) } else { None }),
                    8 => {
                        if rnd() & 1 != 0 {
                            (*lbl).x1B8 = Some(cbR);
                        }
                        let a = rnd() & 1 != 0;
                        ret = m::fn_80169444(a) as u32
                    }
                    9 => ret = m::gm_801694A0(null_mut()) as u32,
                    10 => {
                        t((m::gm_80169520() == (*lbl).x20.as_mut_ptr() as *mut c_void) as u32);
                        t((m::gm_80169530() == (*lbl).xA2.as_mut_ptr() as *mut c_void) as u32);
                        t((m::gm_80169540() == (*lbl).x124.as_mut_ptr() as *mut c_void) as u32);
                    }
                    11 => {
                        let a = (rnd() % 6) as i32;
                        m::fn_80169550(a)
                    }
                    12 => {
                        let n = (rnd() % 40) as isize;
                        m::fn_80169574(n, bufa)
                    }
                    13 => {
                        let a0: u8 = if rnd() % 6 == 0 { 0x21 } else { (rnd() % 40) as u8 };
                        let a1: u8 = if rnd() % 3 == 0 { a0 } else { (rnd() % 40) as u8 };
                        let a2 = (rnd() % 8) as u8;
                        let n = (1 + rnd() % 29) as usize;
                        for i in 0..n {
                            *bufb.add(i) = if rnd() % 2 != 0 { a0 } else { (rnd() % 40) as u8 } as i8;
                        }
                        for i in 0..n {
                            *bufa.add(i) = rnd() as i8;
                        }
                        *bufa.add(n) = -2;
                        m::fn_801695BC(a0, a1, a2, bufb as *const u8, bufa)
                    }
                    14 => {
                        let c: i8 = if rnd() % 6 == 0 { 0x21 } else { (rnd() % 40) as i8 };
                        let co: i8 = if rnd() % 4 == 0 { 254u8 as i8 } else { (rnd() % 8) as i8 };
                        let nc: i8 = if rnd() % 3 == 0 { c } else { (rnd() % 40) as i8 };
                        let nco = (rnd() % 8) as i8;
                        let n = (rnd() % 30) as usize;
                        mklist(bufa, n, 100, -1);
                        m::fn_801697FC(c, co, nc, nco, bufa)
                    }
                    15 => {
                        let mut a0 = [0u8; 3];
                        let a1 = (rnd() % 40) as u8;
                        let a2 = (rnd() % 8) as u8;
                        let n = (1 + rnd() % 19) as usize;
                        for i in 0..3 {
                            a0[i] = if rnd() % 6 == 0 { 0x21 } else { (rnd() % 40) as u8 };
                        }
                        for i in 0..n {
                            *bufb.add(i) = if rnd() % 2 != 0 { a0[(rnd() % 3) as usize] } else { (rnd() % 40) as u8 } as i8;
                        }
                        for i in 0..n {
                            *bufa.add(i) = rnd() as i8;
                        }
                        *bufa.add(n) = -2;
                        m::fn_8016989C(a0.as_mut_ptr(), a1, a2, bufb as *mut u8, bufa)
                    }
                    16 => {
                        let a0 = (rnd() % 4) as u8;
                        let n = (rnd() % 30) as usize;
                        mklist(bufb, n, 100, -1);
                        for i in 0..40 {
                            *bufa.add(i) = rnd() as i8;
                        }
                        (*lbl).x0 = (rnd() % 40) as i8;
                        (*lbl).x1 = if rnd() % 4 != 0 { 0x21 } else { (rnd() % 40) as i8 };
                        (*lbl).x2 = if rnd() % 4 != 0 { 0x21 } else { (rnd() % 40) as i8 };
                        m::fn_80169900(a0, lbl, bufa, bufb)
                    }
                    17 => {
                        let a0 = (rnd() % 3) as u8;
                        let n = (rnd() % 12) as usize;
                        mklist(bufb, n, 100, -1);
                        for i in 0..40 {
                            *bufa.add(i) = rnd() as i8;
                        }
                        for i in 0..27 {
                            (*lbl).x1C0[i] = if rnd() % 4 == 0 { -1 } else { (rnd() % 30) as i8 };
                        }
                        m::fn_80169A84(a0, bufa, bufb)
                    }
                    18 | 19 => {
                        let a0: i8 = if rnd() % 3 == 0 { 4 } else { (rnd() % 40) as i8 };
                        let a1 = rnd() as i8;
                        if op == 18 {
                            m::fn_80169C54(a0, a1)
                        } else {
                            m::fn_80169F50(a0, a1)
                        }
                    }
                    20 => m::fn_8016A09C(),
                    21 => m::gm_8016A164(),
                    22 => ret = m::fn_8016A1E4() as u32,
                    23 => ret = m::gm_8016A1F8() as u32,
                    24 => {
                        m::gm_8016A21C(addr_of_mut!(SMR));
                        t((SMR.x54 as *mut c_void == lbl as *mut c_void) as u32)
                    }
                    25 => {
                        let k0: i8 = if rnd() % 3 == 0 { 4 } else { (rnd() % 40) as i8 };
                        let k1: i8 = if rnd() % 5 == 0 { 0x21 } else { (rnd() % 40) as i8 };
                        let k2 = (rnd() % 40) as i8;
                        let a3 = rnd() as u8;
                        let a4 = rnd() as u8;
                        let a5 = rnd() as u8;
                        let mode = (rnd() % 3) as i32;
                        let a7 = (rnd() % 4) as i32;
                        let color = (rnd() & 1) as u8;
                        let p87 = (rnd() % 40) as u8;
                        let p8b = (rnd() % 8) as u8;
                        let x6 = (rnd() % 100) as i32;
                        let x7 = (rnd() % 60) as i32;
                        let x9 = (rnd() % 9) as i32;
                        let xa = (rnd() % 5) as i32;
                        let flag2 = rnd() as i32;
                        let flag1 = rnd() as i32;
                        let f1 = rg();
                        let f2 = rg();
                        (*lbl).pad_1DB[0] = -1;
                        m::gm_8016A22C(k0, k1, k2, a3, a4, a5, mode, a7, color, p87, p8b, x6, x7, x9, xa, flag2, flag1, f1, f2)
                    }
                    26 => m::gm_8016A404(if rnd() & 1 != 0 { Some(cbE) } else { None }),
                    27 => {
                        let a = rg();
                        m::gm_8016A414(a)
                    }
                    28 => {
                        let a = rnd() as i8;
                        m::gm_8016A424(a)
                    }
                    29 => m::gm_8016A434(),
                    30 => m::fn_8016A450(),
                    31 => m::fn_8016A46C(),
                    32 => {
                        let a = (rnd() % 6) as i32;
                        m::fn_8016A488(a)
                    }
                    33 => {
                        m::gm_8016A404(if rnd() & 1 != 0 { Some(cbE) } else { None });
                        m::fn_80169434(if rnd() & 1 != 0 { Some(cbR) } else { None });
                        (*lbl).set_unk_10_b1(((rnd() % 8) != 0) as u8);
                        (*lbl).xA = (rnd() % 4) as u8;
                        m::fn_8016A4C8();
                    }
                    34 => {
                        m::gm_8016A92C(addr_of_mut!(SMR));
                        t((SMR.x58 == l2) as u32)
                    }
                    35 => {
                        SMR.x58 = if rnd() & 1 != 0 { l2 } else { null_mut() };
                        ret = m::gm_8016A944() as u32
                    }
                    36 => t((m::gm_8016A97C() == (*l2).arr2.as_mut_ptr() as *mut c_void) as u32),
                    37 => t((m::gm_8016A98C() == l2) as u32),
                    38 => {
                        let a0 = rnd() as i8;
                        let a1: i8 = if rnd() % 5 == 0 { 254u8 as i8 } else { rnd() as i8 };
                        ret = m::gm_8016A998(a0, a1) as u32
                    }
                    39 => {
                        let a0 = rnd() as u8;
                        let a1: i8 = if rnd() % 5 == 0 { 254u8 as i8 } else { rnd() as i8 };
                        ret = m::gm_8016A9E8(a0, a1) as u32
                    }
                    40 => {
                        SMR.x58 = if rnd() % 4 != 0 { l2 } else { null_mut() };
                        let ck = (rnd() % 30) as i8;
                        let co: i8 = if rnd() % 5 == 0 { 254u8 as i8 } else { (rnd() % 8) as i8 };
                        for i in 0..28 {
                            if rnd() % 3 == 0 {
                                (*l2).arr2[i] = co;
                            }
                        }
                        for i in 0..28 {
                            if rnd() % 3 == 0 {
                                (*l2).arr1[i] = ck;
                            }
                        }
                        ret = m::gm_8016AC44(ck, co) as u32
                    }
                    _ => ret = m::gm_PcTestLegacyCostumes() as u32,
                }
                c.line(op, k, ret, state_hash());
            }
        }
    }
    assert_eq!(c.done(), 41 * 700);
}
