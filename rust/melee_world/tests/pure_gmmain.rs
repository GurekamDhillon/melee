//! `gmmain` (the save-data accessors) vs the verbatim C definitions over a random `gmm_x0` record
//! (`melee_runtime/gen/pure/drv_gmmain.c`). Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py gmmain`.
#![allow(static_mut_refs)]
#[path = "../../melee_runtime/tests/pure_common/mod.rs"]
mod pure_common;
use melee_types::*;
use melee_world::gmmain::*;
use pure_common::*;

struct Hs(u64);
impl Hs {
    fn w(&mut self, w: u32) {
        for x in w.to_le_bytes() {
            self.0 ^= x as u64;
            self.0 = self.0.wrapping_mul(0x100000001B3);
        }
    }
}

#[test]
fn gmmain_matches_c() {
    let mut out = format!("sizeof_gmm_x0 {}\n", core::mem::size_of::<gmm_x0>());
    let mut rec: Box<gmm_x0> = unsafe { Box::new(core::mem::zeroed()) };
    let base = &mut *rec as *mut gmm_x0 as usize;
    unsafe { gmMainLib_804D3EE0 = &mut *rec };
    let mut r = Rng::new();
    for op in 0..15 {
        for k in 0..40 {
            let a = r.u() as u32;
            let sk = (r.u() % 9) as u8;
            let mut ret = 0i32;
            let mut h = Hs(0xCBF29CE484222325);
            let p = base as *mut u8;
            for j in 0..core::mem::size_of::<gmm_x0>() {
                unsafe { *p.add(j) = r.u() as u8 };
            }
            for i in 0..9u8 {
                unsafe { (*persistent_fighter(i)).x7C.set_b0((r.u() & 1) as _) };
            }
            let off = |h: &mut Hs, q: usize| h.w(if q == 0 { 0xFFFFFFFF } else { (q - base) as u32 });
            unsafe {
                match op {
                    0 => gmMainLib_8015D8B0(a % 96),
                    1 => off(&mut h, gmMainLib_8015CCF0() as usize),
                    2 => off(&mut h, gmMainLib_8015CDC8() as usize),
                    3 => off(&mut h, gmMainLib_8015CDD4() as usize),
                    4 => off(&mut h, gmMainLib_8015CDE0() as usize),
                    5 => off(&mut h, gmMainLib_8015D06C(sk) as usize),
                    6 => ret = gmMainLib_8015D48C(sk) as i32,
                    7 => gmMainLib_8015D4E8(sk, a as i32),
                    8 => off(&mut h, gmMainLib_GetBonusMatchTotal() as usize),
                    9 => off(&mut h, gmMainLib_GetCoinMatchTotal() as usize),
                    10 => off(&mut h, gmMainLib_GetKOTotal() as usize),
                    11 => off(&mut h, gmMainLib_GetStockMatchTotal() as usize),
                    12 => off(&mut h, gmMainLib_GetTimeMatchTotal() as usize),
                    _ => {}
                }
                h.w(ret as u32);
                let sd = &*save();
                for m in 0..sd.x1B40.len() {
                    h.w(sd.x1B40[m]);
                }
                h.w(sd.x1A48);
                h.w(sd.bonus_matches);
                h.w(sd.coin_matches);
                h.w(sd.stock_matches);
                h.w(sd.time_matches);
                h.w(sd.x1A5C as u32);
                for i in 0..9u8 {
                    let f = &*persistent_fighter(i);
                    h.w(f.x7C.b0() as u32);
                    h.w(f.x7C.x84 as u32);
                }
            }
            out += &line(&format!("op{op}"), k, h.0);
        }
    }
    compare_runtime_data("gmmain", &out);
}
