//! src/tu/textlib_1.rs vs the C (gen/mtu/tu/textlib_1.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py textlib_1`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::textlib_1 as m;
use mtu_harness::*;
#[path = "mtu/textlib_1_gen.rs"]
mod g;

const NM: usize = 3;
static mut DT: [DevText; 4] = unsafe { core::mem::zeroed() };
static mut GOBJ0: HSD_GObj = unsafe { core::mem::zeroed() };
static mut GOBJ1: HSD_GObj = unsafe { core::mem::zeroed() };
static mut PROC0: HSD_GObjProc = unsafe { core::mem::zeroed() };
static mut MEM: [i8; 256] = [0; 256];
static mut OBJS: [un_80304138_objalloc_t; 4] = unsafe { core::mem::zeroed() };
static mut MENU: [un_80304138_objalloc_t; NM] = unsafe { core::mem::zeroed() };
static mut ENT: [[un_80304138_objalloc_t_x8; 8]; NM] = unsafe { core::mem::zeroed() };
static mut STRS: [*mut i8; 3] = [null_mut(); 3];
static mut SBUF: [[i8; 16]; 4] = [[0; 16]; 4];
static mut INTV: [[i32; 8]; NM] = [[0; 8]; NM];
static mut VALS: [[u64; 8]; NM] = [[0; 8]; NM];
static mut NULLOK: i32 = 0;
static mut NROWS: [i32; NM] = [0; NM];

unsafe extern "C" fn cbA(a: u32) -> i32 {
    t(7001);
    t(a);
    t(pi(m::un_804D6E48));
    (fv() % 3) as i32
}
unsafe extern "C" fn cbB(a: u32) -> i32 {
    t(7002);
    t(a);
    t(pi(m::un_804D6E48));
    (fv() & 1) as i32
}
unsafe extern "C" fn cbC(a: u32) -> i32 {
    t(7003);
    t(a);
    m::un_804D6E4C = fv() as u8;
    (fv() % 3) as i32 - 1
}
unsafe fn pickcb() -> soundtest_callback {
    match rnd() % 4 {
        0 => Some(cbA),
        1 => Some(cbB),
        2 => Some(cbC),
        _ => None,
    }
}

unsafe fn setup() {
    DT = core::mem::zeroed();
    GOBJ0 = core::mem::zeroed();
    GOBJ1 = core::mem::zeroed();
    PROC0 = core::mem::zeroed();
    OBJS = core::mem::zeroed();
    MENU = core::mem::zeroed();
    ENT = core::mem::zeroed();
    INTV = [[0; 8]; NM];
    VALS = [[0; 8]; NM];
    MEM = [0; 256];
    g::fill_HSD_GObj(&mut GOBJ0);
    g::fill_HSD_GObj(&mut GOBJ1);
    for i in 0..4 {
        g::fill_DevText(&mut DT[i]);
        DT[i].prev = null_mut();
        DT[i].next = null_mut();
        DT[i].buf = (MEM.as_mut_ptr()) as *mut _;
    }
    for i in 0..4 {
        g::fill_un_80304138_objalloc_t(&mut OBJS[i]);
        OBJS[i].x4 = null_mut();
        OBJS[i].x8 = null_mut();
        OBJS[i].xC = None;
        OBJS[i].x10 = null_mut();
        OBJS[i].x14 = null_mut();
        OBJS[i].prev = null_mut();
        OBJS[i].next = null_mut();
    }
    for i in 0..4 {
        for j in 0..15 {
            SBUF[i][j] = (33 + rnd() % 90) as i8;
        }
        SBUF[i][15] = 0;
    }
    for i in 0..3 {
        STRS[i] = if rnd() % 5 == 0 { null_mut() } else { SBUF[i].as_mut_ptr() };
    }
    for mi in 0..NM {
        let n = 1 + (rnd() % 5) as i32;
        NROWS[mi] = n;
        for i in 0..8usize {
            let e = &mut ENT[mi][i];
            g::fill_un_80304138_objalloc_t_x8(e);
            e.x0 = if i as i32 == n { 9 } else { (rnd() % 9) as i32 };
            if i as i32 > n {
                e.x0 = (rnd() % 10) as i32;
            }
            e.x4 = pickcb();
            e.x8 = if rnd() % 6 == 0 { null_mut() } else { SBUF[(rnd() % 4) as usize].as_mut_ptr() };
            e.xC = STRS.as_mut_ptr();
            e.x14 = rg();
            e.x18 = rg() * 10.0;
            e.x1C = rg() * 10.0;
            INTV[mi][i] = fv() as i32;
            VALS[mi][i] = fv64();
            e.x10 = un_80304138_objalloc_t_x8_x10 { any: &raw mut VALS[mi][i] as *mut c_void };
            if e.x0 == 2 {
                e.x14 = 0.0;
                e.x18 = (1 + rnd() % 3) as f32;
                INTV[mi][i] = (rnd() % e.x18 as u32) as i32;
                e.x10 = un_80304138_objalloc_t_x8_x10 { any: &raw mut INTV[mi][i] as *mut c_void };
            } else if e.x0 == 3 {
                e.x10 = un_80304138_objalloc_t_x8_x10 { any: &raw mut INTV[mi][i] as *mut c_void };
            }
        }
        g::fill_un_80304138_objalloc_t(&mut MENU[mi]);
        MENU[mi].x0 = (rnd() % n as u32) as u8;
        MENU[mi].x8 = ENT[mi].as_mut_ptr();
        MENU[mi].x4 = &raw mut DT[mi];
        MENU[mi].xC = pickcb();
        MENU[mi].x10 = &raw mut GOBJ0;
        MENU[mi].x14 = if rnd() & 1 != 0 { &raw mut PROC0 } else { null_mut() };
        MENU[mi].prev = null_mut();
        MENU[mi].next = null_mut();
        DT[mi].h = n as u8;
        DT[mi].w = (1 + rnd() % 10) as u8;
    }
    let mut perm = [0usize; NM];
    for i in 0..NM {
        perm[i] = i;
    }
    for i in (1..NM).rev() {
        let j = (rnd() % (i as u32 + 1)) as usize;
        perm.swap(i, j);
    }
    let len = 1 + (rnd() % NM as u32) as usize;
    for i in 0..len - 1 {
        MENU[perm[i]].prev = &raw mut MENU[perm[i + 1]];
        MENU[perm[i + 1]].next = &raw mut MENU[perm[i]];
    }
    m::un_804D6E40 = &raw mut MENU[perm[0]];
    m::un_804D6E44 = if rnd() & 1 != 0 { &raw mut MENU[perm[(rnd() % len as u32) as usize]] } else { null_mut() };
    m::un_804D6E48 = if rnd() & 1 != 0 { &raw mut ENT[0][(rnd() % 4) as usize] } else { null_mut() };
    m::un_804D6E4C = if rnd() & 1 != 0 { 0 } else { (rnd() % 10) as u8 };
    const BITS: [u32; 7] = [0x1000, 0x800, 0x400, 0x20, 0x40, 0x100, 0x200];
    for i in 0..4usize {
        let mut trig = 0u32;
        m::ext::HSD_PadCopyStatus[i] = core::mem::zeroed();
        for j in 0..7 {
            if rnd() % 7 == 0 {
                trig |= BITS[j];
            }
        }
        let p = &mut m::ext::HSD_PadCopyStatus[i];
        p.trigger = trig;
        p.button = if rnd() & 1 != 0 { trig | 0x20 } else { trig };
        p.stickX = ((rnd() % 161) as i32 - 80) as i8;
        p.stickY = ((rnd() % 161) as i32 - 80) as i8;
        if rnd() % 3 == 0 {
            p.stickX = 0;
        }
        if rnd() % 3 == 0 {
            p.stickY = 0;
        }
    }
    NULLOK = 0;
}

unsafe fn hash_all() {
    for i in 0..4 {
        g::hash_DevText(&DT[i]);
        g::hash_un_80304138_objalloc_t(&OBJS[i]);
    }
    g::hash_HSD_GObj(&GOBJ0);
    g::hash_HSD_GObj(&GOBJ1);
    for mi in 0..NM {
        g::hash_un_80304138_objalloc_t(&MENU[mi]);
        for i in 0..8 {
            g::hash_un_80304138_objalloc_t_x8(&ENT[mi][i]);
            hm(INTV[mi][i] as u32);
            hm(VALS[mi][i] as u32);
            hm((VALS[mi][i] >> 32) as u32);
        }
    }
    hm(pi(m::un_804D6E40));
    hm(pi(m::un_804D6E44));
    hm(pi(m::un_804D6E48));
    hm(m::un_804D6E4C as u32);
}

#[test]
fn textlib_1_matches_c() {
    unsafe {
        g::install();
        for i in 0..4 {
            preg(&raw const DT[i], core::mem::size_of::<DevText>());
        }
        preg(&raw const GOBJ0, core::mem::size_of::<HSD_GObj>());
        preg(&raw const GOBJ1, core::mem::size_of::<HSD_GObj>());
        preg(&raw const PROC0, core::mem::size_of::<HSD_GObjProc>());
        preg(&raw const MEM, 256);
        for i in 0..4 {
            preg(&raw const OBJS[i], core::mem::size_of::<un_80304138_objalloc_t>());
        }
        for mi in 0..NM {
            preg(&raw const MENU[mi], core::mem::size_of::<un_80304138_objalloc_t>());
        }
        for mi in 0..NM {
            preg(&raw const ENT[mi], core::mem::size_of_val(&ENT[mi]));
        }
        preg(&raw const INTV, core::mem::size_of_val(&INTV));
        preg(&raw const VALS, core::mem::size_of_val(&VALS));
        preg(&raw const SBUF, core::mem::size_of_val(&SBUF));
        preg(&raw const m::un_804A2688, core::mem::size_of::<HSD_ObjAllocData>());
        g::freg_all();
        freg(cbA as *const () as usize);
        freg(cbB as *const () as usize);
        freg(cbC as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_textlib_1.txt"));
        for op in 0..14u32 {
            for k in 0..1500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {
                        let r = (rnd() % 8) as usize;
                        let code = (rnd() % 8) as i32;
                        ret = m::un_80302E00(&raw mut ENT[0][r], code) as u32;
                    }
                    1 => ret = m::un_80302EA4(ENT[(rnd() % NM as u32) as usize].as_mut_ptr()) as u32,
                    2 => m::un_80302FFC(&raw mut MENU[(rnd() % NM as u32) as usize]),
                    3 => ret = m::un_80303444(&raw mut MENU[(rnd() % NM as u32) as usize]) as u32,
                    4 => ret = m::un_80303720(&raw mut MENU[(rnd() % NM as u32) as usize]) as u32,
                    5 => ret = m::un_803039A4((rnd() % 4) as u8) as u32,
                    6 => m::un_80303AC4(&raw mut MENU[(rnd() % NM as u32) as usize]),
                    7 => m::fn_80303EF4(&raw mut GOBJ0),
                    8 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        let cc = fv() as i32;
                        let oi = (rnd() % 4) as usize;
                        let ei = (rnd() % NM as u32) as usize;
                        m::un_80303FD4(&raw mut GOBJ1, &raw mut OBJS[oi], ENT[ei].as_mut_ptr(), a, b, cc);
                    }
                    9 => m::un_80304138(),
                    10 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        let cc = fv() as i32;
                        let ei = (rnd() % NM as u32) as usize;
                        ret = pi(m::un_80304168(ENT[ei].as_mut_ptr() as *mut c_void, a, b, cc));
                    }
                    11 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        let cc = fv() as i32;
                        let mi = (rnd() % NM as u32) as usize;
                        let ei = (rnd() % NM as u32) as usize;
                        NULLOK = 1;
                        ret = pi(m::un_80304210(&raw mut MENU[mi], ENT[ei].as_mut_ptr() as *mut c_void, a, b, cc));
                    }
                    12 => m::un_80304334(&raw mut MENU[(rnd() % NM as u32) as usize]),
                    _ => m::un_80304344(&raw mut MENU[(rnd() % NM as u32) as usize]),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 14 * 1500);
    }
}
