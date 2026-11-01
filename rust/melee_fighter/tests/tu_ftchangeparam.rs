//! src/tu/ftchangeparam.rs vs the C (gen/cref/tu/ftchangeparam.c, shipping contraction),
//! 8000 cases; plus a smoke test of the reload path (ftCo_800D105C) with its hooks.
#![allow(static_mut_refs)]
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftchangeparam as c;
use melee_fighter::types::*;
use tu_s25::*;

fn words<T>(p: &mut T) -> &mut [u32] {
    unsafe { core::slice::from_raw_parts_mut(p as *mut T as *mut u32, core::mem::size_of::<T>() / 4) }
}

fn rs_nz(h: &mut H) -> f32 {
    let s = h.rw();
    if s == 0.0 { 0.5 } else { s }
}

#[test]
fn ftchangeparam_vs_c() {
    let exp = expected("tu_ftchangeparam");
    let mut k24: Box<Fighter_804D6524_t> = zeroed();
    let mut k20: Box<Fighter_804D6520_t> = zeroed();
    let mut k1c: Box<Fighter_804D651C_t> = zeroed();
    let mut cd: Box<ftCommonData> = zeroed();
    unsafe {
        c::ext::Fighter_804D6524 = &*k24 as *const _ as *mut _;
        c::ext::Fighter_804D651C = &*k1c as *const _ as *mut _;
        hooks::FIGHTER_804D6520 = &*k20;
        hooks::P_FT_COMMON_DATA = &*cd;
    }
    let mut n = 0;
    for op in 0..4u32 {
        for k in 0..2000u32 {
            let mut h = H::case(op, k);
            let mut at: Box<ftCo_DatAttrs> = zeroed();
            let mut x2d0: Box<Fighter_x2D0_t> = zeroed();
            let mut f: Box<Fighter> = zeroed();
            let mut go: Box<HSD_GObj> = zeroed();
            let mut go2: Box<HSD_GObj> = zeroed();
            for w in words(&mut *k24) { *w = h.rw().to_bits(); }
            for w in words(&mut *k20) { *w = h.rw().to_bits(); }
            for w in words(&mut *k1c) { *w = h.rw().to_bits(); }
            for w in words(&mut *at) { *w = h.rw().to_bits(); }
            for i in 0..5 { x2d0.x14[i] = h.rw(); }
            x2d0.x28 = (h.rnd() % 6) as i32;
            let a = h.rw();
            let s = rs_nz(&mut h);
            let e = h.rw();
            f.x34_scale.y = if h.rnd() & 1 != 0 { 1.0 } else { rs_nz(&mut h) };
            f.x197C = if h.rnd() & 1 != 0 { &mut *go2 } else { core::ptr::null_mut() };
            let m = (h.rnd() & 1) as u8;
            f.set_is_metal(m);
            cd.x310 = h.rw();
            f.x2D0 = &mut *x2d0;
            go.user_data = &mut *f as *mut Fighter as *mut _;
            let mut ret = 0u32;
            match op {
                0 => ret = c::ftCo_CalcYScaledKnockback(a, s, e).to_bits(),
                1 => {
                    let sc = if h.rnd() & 3 != 0 { s } else { 1.0 };
                    c::ftco_800cf6e8(&mut at, sc)
                }
                2 => ret = c::ftco_800d0ec8(&f).to_bits(),
                _ => c::ftco_800d0cbc(&mut *go),
            }
            h.h = 0x811C9DC5;
            for i in 0..words(&mut *at).len() {
                let w = words(&mut *at)[i];
                h.hm(w);
            }
            for i in 0..5 { h.hf(x2d0.x14[i]); }
            assert_eq!(format!("{} {} {:08X} {:08X}", op, k, ret, h.h), exp[n], "case {op} {k}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}

fn geno(_f: &mut Fighter) {}
static mut DISPATCHED: i32 = -1;
fn mex(kind: i32, _g: *mut HSD_GObj) { unsafe { DISPATCHED = kind } }
fn present(_k: i32) -> bool { true }

#[test]
fn reload_smoke() {
    let mut attrs: Box<ftCo_DatAttrs> = zeroed();
    attrs.gravity = 0.1;
    attrs.weight = 100.0;
    let mut pick: Box<itPickup> = zeroed();
    pick.gr_light_offset.x = 2.0;
    let mut v2: Box<Vec2> = zeroed();
    v2.x = 3.0;
    let mut fd: Box<ftData> = zeroed();
    fd.x0 = &mut *attrs;
    fd.x40 = &mut *pick;
    fd.x50 = &mut *v2;
    let mut k18: Box<Fighter_804D6518_t> = zeroed();
    k18.x0 = 2.0;
    k18.x4 = 0.5;
    let mut f: Box<Fighter> = zeroed();
    f.ft_data = &mut *fd;
    f.x34_scale.y = 1.0;
    f.kind = 7;
    f.set_x2229_b1(1);
    let mut go: Box<HSD_GObj> = zeroed();
    go.user_data = &mut *f as *mut Fighter as *mut _;
    unsafe {
        c::ext::Fighter_804D6518 = &*k18 as *const _ as *mut _;
        c::ext::Geno_ApplyAttrs = Some(geno);
        c::ext::Mex_OnReapplyAttrDispatch = Some(mex);
        c::ext::ftKindCalcIndiviParamTable_present = Some(present);
    }
    c::ftco_800d105c(&mut *go);
    assert_eq!(f.co_attrs.gravity, 0.2);
    assert_eq!(f.co_attrs.weight, 50.0);
    assert_eq!(f.x294_itPickup.gr_light_offset.x, 2.0);
    assert_eq!(f.x2C4.x, 3.0);
    assert_eq!(unsafe { DISPATCHED }, 7);
    f.co_attrs.gravity = 9.0;
    c::ftco_800d0fa0(&mut *go);
    assert_eq!(f.co_attrs.gravity, 0.1);
}
