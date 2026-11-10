//! src/tu/ftcolanim.rs vs the C (gen/cref/tu/ftcolanim.c), 9000 cases: every function with
//! recording stubs for the colanim interpreter, ftAction, Mex and the PlCo tables.
#![allow(static_mut_refs)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftcolanim as c;
use melee_fighter::types::*;
use tu_s25::*;

const KMAX: usize = Ft_Kind_Max as usize;
static mut HS: *mut H = core::ptr::null_mut();
static mut FP: *mut Fighter = core::ptr::null_mut();
static mut GOP: usize = 0;
static mut GO2P: usize = 0;
static mut TBL: *mut Fighter_804D653C_t = core::ptr::null_mut();
static mut SPY: *mut Fighter_804D653C_t = core::ptr::null_mut();
static mut VAN: i32 = 0;
static mut LOOPS: i32 = 0;

fn h() -> &'static mut H { unsafe { &mut *HS } }
fn sr(i: usize) -> bool { h().srv[i] & 1 != 0 }
fn which(co: *mut ColorOverlay) -> u32 {
    unsafe {
        let f = &mut *FP;
        if co == &mut f.x408 as *mut _ { 1 } else if co == &mut f.x488 as *mut _ { 2 } else if co == &mut f.x508 as *mut _ { 3 } else { 9 }
    }
}

fn cms(g: *mut HSD_GObj, ms: i32, fl: MotionFlags, a: f32, b: f32, cc: f32, o: *mut HSD_GObj) {
    t(1); t(ms as u32); t(fl as u32); t(a.to_bits()); t(b.to_bits()); t(cc.to_bits());
    t((g as usize == unsafe { GOP }) as u32); t(o.is_null() as u32);
}
fn pge(slot: i32, i: i32) -> *mut HSD_GObj {
    t(2); t(slot as u32); t(i as u32);
    if sr(0) { unsafe { GO2P as *mut HSD_GObj } } else { core::ptr::null_mut() }
}
fn d4f24(g: *mut HSD_GObj, i: i32) { t(3); t((g as usize == unsafe { GO2P }) as u32); t(i as u32); }
fn gm(slot: i32, sub: i32) { t(4); t(slot as u32); t(sub as u32); }
fn l498(co: *mut ColorOverlay) { t(5); t(which(co)); }
fn l4c8(co: *mut ColorOverlay, tb: *mut Fighter_804D653C_t, idx: i32, b: i32) -> bool {
    t(6); t(which(co));
    t(unsafe { if tb == TBL { 1 } else if tb == SPY { 2 } else { 9 } });
    t(idx as u32); t(b as u32);
    sr(1)
}
fn a10(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(10); }
fn a11(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(11); }
fn a12(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(12); }
fn a13(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(13); }
fn a14(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(14); }
fn a15(_g: *mut HSD_GObj, _c: *mut CommandInfo) { t(15); }
fn c53e4(fp: *mut Fighter) -> i32 { t(16); t((fp == unsafe { FP }) as u32); sr(2) as i32 }
fn mex(kind: i32, g: *mut HSD_GObj, v: *mut core::ffi::c_void) {
    t(17); t(kind as u32); t((g as usize == unsafe { GOP }) as u32);
    t((v as usize == unsafe { &raw const VAN } as usize) as u32);
}
fn l258(g: *mut HSD_GObj, co: *mut ColorOverlay, cb: c::ColAnimCb) -> bool {
    t(7); t(which(co));
    t(if fnsym::same(cb as usize, c::ft_800bff34 as usize) { 1 } else if fnsym::same(cb as usize, c::ft_800bff70 as usize) { 2 } else { 9 });
    if h().rnd() & 1 != 0 {
        let a = 21 + (h().rnd() % 3) as i32;
        cb(g, core::ptr::null_mut(), a);
    }
    unsafe {
        if LOOPS > 0 && h().rnd() & 1 != 0 {
            LOOPS -= 1;
            return true;
        }
    }
    false
}

#[test]
fn ftcolanim_vs_c() {
    let exp = expected("tu_ftcolanim");
    let mut tbl: Box<[Fighter_804D653C_t; 0x7B]> = zeroed();
    let mut spy: Box<[Fighter_804D653C_t; 8]> = zeroed();
    let mut ms4: Box<[*mut core::ffi::c_void; KMAX]> = zeroed();
    let cd: Box<ftCommonData> = zeroed();
    unsafe {
        TBL = tbl.as_mut_ptr();
        SPY = spy.as_mut_ptr();
        c::ext::Fighter_804D653C = TBL;
        c::ext::Fighter_804D6538 = SPY;
        c::ext::ftData_UnkMotionStates4 = ms4.as_ptr();
        hooks::P_FT_COMMON_DATA = &*cd;
        c::ext::FIGHTER_CHANGE_MOTION_STATE = Some(cms);
        c::ext::PLAYER_GET_ENTITY_AT_INDEX = Some(pge);
        hooks::FTCO_800D4F24 = Some(d4f24);
        c::ext::GM_80167320 = Some(gm);
        c::ext::LB_80014498 = Some(l498);
        c::ext::LB_800144C8 = Some(l4c8);
        c::ext::LB_80014258 = Some(l258);
        c::ext::FTACTION_80071028 = Some(a10);
        c::ext::FTACTION_800711DC = Some(a11);
        c::ext::FTACTION_80071B50 = Some(a12);
        c::ext::FTACTION_80071CA4 = Some(a13);
        c::ext::FTACTION_800730B8 = Some(a14);
        c::ext::FTACTION_80073108 = Some(a15);
        c::ext::FTCO_800C53E4 = Some(c53e4);
        c::ext::MEX_ON_ACTION_STATE_CHANGE_DISPATCH = Some(mex);
    }
    let mut n = 0;
    for op in 0..9u32 {
        for k in 0..1000u32 {
            let mut hh = H::case(op, k);
            unsafe { HS = &mut hh; TR = 0; }
            let mut f: Box<Fighter> = zeroed();
            let mut f2: Box<Fighter> = zeroed();
            let mut go: Box<HSD_GObj> = zeroed();
            let mut go2: Box<HSD_GObj> = zeroed();
            let mut cb: Box<CmSubject> = zeroed();
            go.user_data = &mut *f as *mut Fighter as *mut _;
            go2.user_data = &mut *f2 as *mut Fighter as *mut _;
            unsafe {
                FP = &mut *f;
                GOP = &mut *go as *mut HSD_GObj as usize;
                GO2P = &mut *go2 as *mut HSD_GObj as usize;
            }
            let h = h();
            for i in 0..0x7B { tbl[i].unk5 = (h.rnd() & 1) as u8; }
            for i in 0..KMAX { ms4[i] = core::ptr::null_mut(); }
            f.kind = (h.rnd() % KMAX as u32) as _;
            ms4[f.kind as usize] = unsafe { &raw mut VAN } as *mut _;
            f.gobj = &mut *go;
            f.player_idx = (h.rnd() % 6) as u8;
            f.set_is_sub_fighter((h.rnd() & 1) as u8);
            f.set_x2222_b5((h.rnd() & 1) as u8);
            f.set_x2226_b4((h.rnd() & 1) as u8);
            f.set_stamina_dead((h.rnd() & 1) as u8);
            f.set_x221D_b6((h.rnd() & 1) as _);
            f.set_x2221_b0((h.rnd() & 1) as u8);
            f.dmg.x18F0 = (h.rnd() % 2) as _;
            f.x1990 = (h.rnd() % 2) as _;
            f.x1994 = (h.rnd() % 2) as _;
            f.x408.x0_timer = h.rnd() as i32;
            f.x488.x0_timer = h.rnd() as i32;
            f.x408.x28_colanim.i = (h.rnd() % 2) as _;
            f2.x408.x0_timer = h.rnd() as i32;
            f2.x488.x0_timer = h.rnd() as i32;
            f.x890_cameraBox = &mut *cb;
            unsafe { LOOPS = (h.rnd() % 4) as i32 };
            let g: *mut HSD_GObj = &mut *go;
            match op {
                0 => c::ftco_800bfd04(g),
                1 => c::ftco_800bfd9c(g),
                2 => {
                    // C evaluates the call's arguments right to left (MS ABI).
                    let b = (h.rnd() & 1) as i32;
                    let a = (h.rnd() % 0x80) as i32;
                    let r = c::ftco_800bffd0(&mut f, a, b);
                    t(r as u32);
                }
                3 => c::ft_800c0098(&mut f),
                4 => c::ftco_800c0134(&mut f),
                5 => { let a = (h.rnd() % 0x7B) as i32; c::ftco_800c0200(&mut f, a) }
                6 => { let a = (h.rnd() % 0x7B) as i32; c::ftco_800c0358(&f, &mut f2, a) }
                7 => c::ftco_800c0408(g),
                _ => {
                    c::ftco_800bffac(&mut f);
                    c::ftco_800c0074(&mut f);
                    c::ftco_sleep_anim(g);
                    c::ftco_sleep_iasa(g);
                }
            }
            h.h = 0x811C9DC5;
            h.hm(f.invisible() as u32); h.hm(f.x221E_b1() as u32); h.hm(f.x221E_b2() as u32);
            h.hm(f.x2219_b1() as u32); h.hm(f.is_sleeping() as u32); h.hm(f.x221F_b1() as u32);
            h.hm(f.x2221_b3() as u32); h.hm(cb.state as u32);
            h.hm(f2.x408.x0_timer as u32); h.hm(f2.x488.x0_timer as u32);
            let got = format!("{} {} {:08X} {:08X}", op, k, h.h, unsafe { TR });
            assert_eq!(got, exp[n], "case {op} {k}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}
