//! src/tu/textlib.rs vs the C (gen/mtu/tu/textlib.c: the whole TU, shipping contraction; regenerate
//! the expected file with `python rust/melee_world/gen/mtu/run_tu.py textlib`). The glyph byte
//! `DevTextGlyph { color:2; unk:6 }` is kept in the GameCube bit order (color on top) by the port;
//! the native C build keeps it low, so the C driver converts the odd buffer bytes before hashing and
//! this test fills them in the converted form.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::textdraw::{devtext_drawlist, devtext_poolhead};
use melee_world::tu::textlib as m;
use melee_world::tu::textlib::PrintfArg;
use melee_world::tu::textlib_1::un_804D6E44;
use mtu_harness::*;
#[path = "mtu/textlib_gen.rs"]
mod g;

static mut P: [DevText; 8] = unsafe { core::mem::zeroed() };
static mut BUFS: [[i8; 192]; 8] = [[0; 192]; 8];
static mut STR: [i8; 128] = [0; 128];
static mut LONGS: [i8; 64] = [0; 64];
static mut MENU: un_80304138_objalloc_t = unsafe { core::mem::zeroed() };

unsafe extern "C" fn cbx(_a: u32) -> i32 {
    0
}

fn gcb(b: u32) -> u32 {
    ((b & 3) << 6) | (b >> 2)
}

unsafe fn setup() {
    P = core::mem::zeroed();
    for i in 0..8usize {
        g::fill_DevText(&mut P[i]);
        P[i].w = (1 + rnd() % 10) as u8;
        P[i].h = (1 + rnd() % 6) as u8;
        P[i].cursor_x = (rnd() % P[i].w as u32) as u8;
        P[i].cursor_y = (rnd() % P[i].h as u32) as u8;
        P[i].id = ((rnd() % 7) as i32 - 2) as i8;
        P[i].buf = BUFS[i].as_mut_ptr();
        for j in 0..192 {
            let b = if rnd() % 4 == 0 { 0 } else { rnd() as u8 };
            BUFS[i][j] = (if j & 1 != 0 { gcb(b as u32) as u8 } else { b }) as i8;
        }
        P[i].prev = null_mut();
        P[i].next = null_mut();
    }
    for i in 0..128 {
        STR[i] = if rnd() % 9 == 0 { 0 } else { (rnd() % 94 + 33) as i8 };
    }
    STR[127] = 0;
    for i in 0..40 {
        STR[i] = if rnd() % 5 == 0 { b'\n' as i8 } else { STR[i] };
    }
    for i in 0..50 {
        LONGS[i] = (b'a' + (i % 26) as u8) as i8;
    }
    LONGS[50] = 0;
    let mut perm = [0usize; 8];
    for i in 0..8 {
        perm[i] = i;
    }
    for i in (1..8usize).rev() {
        let j = (rnd() % (i as u32 + 1)) as usize;
        perm.swap(i, j);
    }
    let nd = (rnd() % 4) as usize;
    let nf = 1 + (rnd() % 3) as usize;
    devtext_drawlist = null_mut();
    devtext_poolhead[0] = null_mut();
    let mut prev: *mut DevText = null_mut();
    for i in 0..nd {
        let t = &raw mut P[perm[i]];
        (*t).prev = prev;
        if !prev.is_null() {
            (*prev).next = t;
        } else {
            devtext_drawlist = t;
        }
        prev = t;
    }
    prev = null_mut();
    for i in 0..nf {
        let t = &raw mut P[perm[nd + i]];
        (*t).prev = prev;
        if !prev.is_null() {
            (*prev).next = t;
        } else {
            devtext_poolhead[0] = t;
        }
        prev = t;
    }
    MENU = core::mem::zeroed();
    un_804D6E44 = if rnd() & 1 != 0 { &raw mut MENU } else { null_mut() };
}

unsafe fn hash_all() {
    for i in 0..8 {
        g::hash_DevText(&P[i]);
        for j in 0..192 {
            hm(BUFS[i][j] as u8 as u32);
        }
    }
    hm(pi(devtext_drawlist));
    hm(pi(devtext_poolhead[0]));
    hm(pi(un_804D6E44));
    hm(fi(MENU.xC));
    hm(rh(&STR));
}

#[test]
fn textlib_matches_c() {
    unsafe {
        g::install();
        for i in 0..8 {
            preg(&raw const P[i], core::mem::size_of::<DevText>());
        }
        preg(&raw const MENU, core::mem::size_of::<un_80304138_objalloc_t>());
        g::freg_all();
        freg(cbx as *const () as usize);
        let mut c = Check::new(include_str!("data/mtu_textlib.txt"));
        for op in 0..21u32 {
            for k in 0..1200u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                let t = &raw mut P[(rnd() % 8) as usize];
                match op {
                    0 => {
                        let id = (rnd() % 8) as i32 - 3;
                        let x = fv() as i32;
                        let y = fv() as i32;
                        let w = 1 + (rnd() % 10) as i32;
                        let h = 1 + (rnd() % 6) as i32;
                        let r = m::DevText_Create(id as i8, x, y, w, h, BUFS[(rnd() % 8) as usize].as_mut_ptr());
                        ret = pi(r);
                    }
                    1 => m::DevText_EraseFirstLine(t),
                    2 => {
                        let x = (rnd() % 30) as i32 - 8;
                        let y = (rnd() % 20) as i32 - 6;
                        m::DevText_SetCursorXY(t, x, y);
                    }
                    3 => {
                        let x = (rnd() % 30) as i32 - 8;
                        m::DevText_SetCursorX(t, x);
                    }
                    4 => m::DevText_HideCursor(t),
                    5 => m::DevText_80302AC0(t),
                    6 => m::DevText_ShowBackground(t),
                    7 => m::DevText_HideBackground(t),
                    8 => m::DevText_ShowText(t),
                    9 => m::DevText_HideText(t),
                    10 => {
                        let x = rw();
                        let y = rw();
                        m::DevText_SetScale(t, x, y);
                    }
                    11 => {
                        let x = fv() as i32 - 20;
                        let y = fv() as i32 - 20;
                        m::DevText_SetXY(t, x, y);
                    }
                    12 => ret = m::DevText_StoreColorIndex(t, rnd() as u8) as u32,
                    13 | 14 => {
                        let r = rnd() as u8;
                        let g_ = rnd() as u8;
                        let b = rnd() as u8;
                        let a = rnd() as u8;
                        let col = _GXColor { r, g: g_, b, a };
                        let o = if op == 13 { m::DevText_SetTextColor(t, col) } else { m::DevText_SetBGColor(t, col) };
                        ret = (o.r as u32) << 24 | (o.g as u32) << 16 | (o.b as u32) << 8 | o.a as u32;
                    }
                    15 => m::DevText_Erase(t),
                    16 => {
                        let s = if rnd() % 8 == 0 { null_mut() } else { STR.as_mut_ptr().add((rnd() % 10) as usize) };
                        if rnd() & 1 != 0 {
                            (*t).flags |= DEVTEXT_FLAG_NOWRAP as u8;
                        } else {
                            (*t).flags &= !(DEVTEXT_FLAG_NOWRAP as u8);
                        }
                        m::DevText_Print(t, s);
                    }
                    17 => m::DevText_PrintInt(t, fv() as i32),
                    18 => {
                        let a = fv() as i32;
                        let n1 = (rnd() % 20001) as i32 - 10000;
                        let f = n1 as f32 / 4.0;
                        let fmt = |s: &'static [u8]| s.as_ptr() as *const i8;
                        let s1 = STR.as_ptr().add(1);
                        let ls = LONGS.as_ptr();
                        match rnd() % 10 {
                            0 => m::DevText_Printf(t, fmt(b"%d\0"), &[PrintfArg::Int(a)]),
                            1 => m::DevText_Printf(t, fmt(b"%02x\0"), &[PrintfArg::Int(a as u8 as i32)]),
                            2 => m::DevText_Printf(t, fmt(b"%04x\0"), &[PrintfArg::Int(a as u16 as i32)]),
                            3 => m::DevText_Printf(t, fmt(b"%08x\0"), &[PrintfArg::Int(a)]),
                            4 => m::DevText_Printf(t, fmt(b"%3.2f\0"), &[PrintfArg::Dbl(f as f64)]),
                            5 => {
                                let ch = b'A' as i32 + (rnd() % 26) as i32;
                                m::DevText_Printf(
                                    t,
                                    fmt(b"[%s|%5d|%-4x|%c]\0"),
                                    &[PrintfArg::Str(s1), PrintfArg::Int(a), PrintfArg::Int(a), PrintfArg::Int(ch)],
                                )
                            }
                            6 => m::DevText_Printf(t, fmt(b"%s%s\0"), &[PrintfArg::Str(ls), PrintfArg::Str(ls)]),
                            7 => m::DevText_Printf(
                                t,
                                fmt(b"%+d %u %X %o %#x\0"),
                                &[PrintfArg::Int(a), PrintfArg::Int(a), PrintfArg::Int(a), PrintfArg::Int(a), PrintfArg::Int(a)],
                            ),
                            8 => m::DevText_Printf(
                                t,
                                fmt(b"%10.3f|%-9.1f|%.0f %%\0"),
                                &[
                                    PrintfArg::Dbl((n1 as f32 / 8.0) as f64),
                                    PrintfArg::Dbl(((n1 / 4) as f32 / 2.0) as f64),
                                    PrintfArg::Dbl((n1 / 8) as f32 as f64),
                                ],
                            ),
                            _ => m::DevText_Printf(
                                t,
                                fmt(b"%.3s|%05d|%5.2d|% d\0"),
                                &[PrintfArg::Str(ls), PrintfArg::Int(a), PrintfArg::Int(a), PrintfArg::Int(a)],
                            ),
                        }
                    }
                    19 => ret = pi(m::un_80302DF0()),
                    _ => {
                        let cb: soundtest_callback = if rnd() & 1 != 0 { Some(cbx) } else { None };
                        m::un_80302DF8(&raw mut MENU, cb);
                    }
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 21 * 1200);
    }
}
