//! src/tu/textdraw.rs vs the C (gen/mtu/tu/textdraw.c: the whole TU, shipping contraction;
//! regenerate the expected file with `python rust/melee_world/gen/mtu/run_tu.py textdraw`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut)]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu::textdraw as m;
use mtu_harness::*;
#[path = "mtu/textdraw_gen.rs"]
mod g;

static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
static mut GOBJ: HSD_GObj = unsafe { core::mem::zeroed() };
static mut COBJA: HSD_CObj = unsafe { core::mem::zeroed() };
static mut COBJB: HSD_CObj = unsafe { core::mem::zeroed() };
static mut COBJC: HSD_CObj = unsafe { core::mem::zeroed() };
static mut BUFS: [[i8; 192]; 32] = [[0; 192]; 32];
static mut STR: [i8; 128] = [0; 128];
static mut HEAD: *mut DevText = null_mut();

unsafe fn p(i: usize) -> *mut DevText {
    (&raw mut m::devtext_pool.entries as *mut DevText).add(i)
}

unsafe fn setup() {
    m::devtext_pool = core::mem::zeroed();
    GO = core::mem::zeroed();
    GOBJ = core::mem::zeroed();
    for i in 0..4 {
        g::fill_HSD_GObj(&mut GO[i]);
    }
    g::fill_HSD_GObj(&mut GOBJ);
    for i in 0..32usize {
        let e = &mut *p(i);
        g::fill_DevText(e);
        e.w = (1 + rnd() % 10) as u8;
        e.h = (1 + rnd() % 6) as u8;
        e.id = ((rnd() % 7) as i32 - 2) as i8;
        e.set_unk((rnd() % 24) as u8);
        e.buf = (BUFS[i].as_mut_ptr()) as *mut _;
        for j in 0..192 {
            BUFS[i][j] = if rnd() % 4 == 0 { 0 } else { rnd() as i8 };
        }
        e.prev = null_mut();
        e.next = null_mut();
    }
    for i in 0..128 {
        STR[i] = if rnd() % 9 == 0 { 0 } else { (rnd() % 94 + 33) as i8 };
    }
    STR[127] = 0;
    let mut perm = [0usize; 32];
    for i in 0..32 {
        perm[i] = i;
    }
    for i in (1..32usize).rev() {
        let j = (rnd() % (i as u32 + 1)) as usize;
        perm.swap(i, j);
    }
    let nd = (rnd() % 7) as usize;
    let nf = (rnd() % 10) as usize;
    m::devtext_drawlist = null_mut();
    m::devtext_poolhead[0] = null_mut();
    let mut prev: *mut DevText = null_mut();
    for i in 0..nd {
        let t = p(perm[i]);
        (*t).prev = prev;
        if !prev.is_null() {
            (*prev).next = t;
        } else {
            m::devtext_drawlist = t;
        }
        prev = t;
    }
    prev = null_mut();
    for i in 0..nf {
        let t = p(perm[nd + i]);
        (*t).prev = prev;
        if !prev.is_null() {
            (*prev).next = t;
        } else {
            m::devtext_poolhead[0] = t;
        }
        prev = t;
    }
    HEAD = if nd != 0 && rnd() & 1 != 0 { p(perm[(rnd() % nd as u32) as usize]) } else { m::devtext_drawlist };
    let n = rnd() % 4;
    m::devtext_cobj = if n == 0 {
        null_mut()
    } else if n == 1 {
        &raw mut COBJC
    } else {
        null_mut()
    };
    m::devtext_gobj = if rnd() & 1 != 0 { &raw mut GO[(rnd() % 4) as usize] } else { null_mut() };
    m::devtext_setup_classifier = fv() as i32;
    m::devtext_setup_p_link = fv() as i32;
    m::devtext_setup_priority = fv() as i32;
    m::devtext_setup_gx_link = fv() as i32;
    m::devtext_setup_render_priority = fv() as i32;
    m::ext::HSD_GObj_CameraKind = (rnd() % 5) as u8;
}

unsafe fn hash_all() {
    for i in 0..32 {
        g::hash_DevText(&*p(i));
    }
    for i in 0..4 {
        g::hash_HSD_GObj(&GO[i]);
    }
    g::hash_HSD_GObj(&GOBJ);
    hm(pi(HEAD));
    hm(pi(m::devtext_drawlist));
    hm(pi(m::devtext_poolhead[0]));
    hm(pi(m::devtext_cobj));
    hm(pi(m::devtext_gobj));
    hm(m::devtext_setup_classifier as u32);
    hm(m::devtext_setup_p_link as u32);
    hm(m::devtext_setup_priority as u32);
    hm(m::devtext_setup_gx_link as u32);
    hm(m::devtext_setup_render_priority as u32);
    hm(rh(&STR));
}

#[test]
fn textdraw_matches_c() {
    unsafe {
        g::install();
        for i in 0..32 {
            preg(p(i), core::mem::size_of::<DevText>());
        }
        for i in 0..4 {
            preg(&raw const GO[i], core::mem::size_of::<HSD_GObj>());
        }
        preg(&raw const GOBJ, core::mem::size_of::<HSD_GObj>());
        preg(&raw const COBJA, core::mem::size_of::<HSD_CObj>());
        preg(&raw const COBJB, core::mem::size_of::<HSD_CObj>());
        preg(&raw const COBJC, core::mem::size_of::<HSD_CObj>());
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_textdraw.txt"));
        for op in 0..14u32 {
            for k in 0..1500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {
                        let s = if rnd() % 8 == 0 { null_mut() } else { STR.as_mut_ptr().add((rnd() % 8) as usize) };
                        ret = m::DevText_StrLen(s) as u32;
                    }
                    1 => {
                        let mut num = if rnd() % 16 == 0 { i32::MIN } else { fv() as i32 };
                        if rnd() & 1 != 0 {
                            num = num.wrapping_neg();
                        }
                        let off = (rnd() % 4) as usize;
                        m::DevText_NumToStr(num, STR.as_mut_ptr().add(off));
                    }
                    2 => m::DevText_InitPool(),
                    3 => {
                        if !HEAD.is_null() {
                            m::DevText_Remove(&raw mut HEAD);
                        }
                    }
                    4 => {
                        let t = if rnd() % 8 == 0 { null_mut() } else { p((rnd() % 32) as usize) };
                        m::DevText_Unlink(t);
                    }
                    5 => {
                        let t = if rnd() % 8 == 0 { null_mut() } else { p((rnd() % 32) as usize) };
                        ret = m::DevText_IsListed(t) as u32;
                    }
                    6 => m::DevText_SetupCObj(),
                    7 => m::DevText_Draw(p((rnd() % 32) as usize)),
                    8 => {
                        let pass = if rnd() & 1 != 0 { 2 } else { fv() as i32 };
                        m::DevText_DrawAll(&raw mut GOBJ, pass as isize);
                    }
                    9 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        let cc = fv() as i32;
                        let d = (fv() & 63) as i32;
                        m::DevText_CreateCObj(a, b, cc, d, fv() as u8);
                    }
                    10 => {
                        let a = fv() as i32;
                        let b = fv() as i32;
                        let cc = fv() as i32;
                        let d = (fv() & 63) as i32;
                        let e = fv() as i32;
                        ret = pi(m::DevText_Setup(a, b, cc, d, e, fv() as u8));
                    }
                    11 => m::DevText_AddToList(&raw mut m::devtext_drawlist, p((rnd() % 32) as usize)),
                    12 => m::DevText_Show(&raw mut GOBJ, p((rnd() % 32) as usize)),
                    _ => ret = pi(m::DevText_GetGObj()),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 14 * 1500);
    }
}
