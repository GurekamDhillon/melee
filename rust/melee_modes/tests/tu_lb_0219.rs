//! src/tu/lb_0219.rs vs the C (gen/cref/tu/lb_0219.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_0219`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_0219 as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_0219_gen.rs"]
mod g;

static mut GNEW: HSD_GObj = unsafe { core::mem::zeroed() };
static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut UD: m::BgFlashUserData = unsafe { core::mem::zeroed() };
static mut LBF: [u8; 64] = [0; 64];
static mut CB: [u8; 64] = [0; 64];

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_0219_matches_c() {
    unsafe {
        g::install();
        reg(&raw const GNEW);
        reg(&raw const G);
        reg(&raw const UD);
        reg(&raw const LBF);
        reg(&raw const CB);
        reg(&raw const m::bgflash_alloc_data);
        reg(&raw const m::lbl_804D63DC);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_lb_0219.txt"));
        for op in 0..8u32 {
            for k in 0..400u32 {
                let ret = 0u32;
                case(op, k);
                GNEW = core::mem::zeroed();
                G = core::mem::zeroed();
                UD = core::mem::zeroed();
                g::fill_BgFlashUserData(&mut UD);
                g::fill_HSD_ObjAllocData(&mut m::bgflash_alloc_data);
                g::fill_HSD_GObj(&mut G);
                G.user_data = (&raw mut UD) as *mut core::ffi::c_void;
                m::flash_gobj = &raw mut G;
                m::lbl_804D63DC = if rnd() & 1 != 0 { (&raw mut LBF) as *mut Fighter_804D653C_t } else { core::ptr::null_mut() };
                m::flash_scale = rg();
                match op {
                    0 => m::lbBgFlash_Free(if rnd() & 1 != 0 { (&raw mut UD) as *mut core::ffi::c_void } else { core::ptr::null_mut() }),
                    1 => m::lbBgFlash_SetFlashScale(rg()),
                    2 => m::lbBgFlash_Init(fv() as i32),
                    3 => m::lbBgFlash_Proc(&raw mut G),
                    4 => {
                        let a = fv() as i32;
                        m::fn_80021C18(&raw mut G, (&raw mut CB) as *mut CommandInfo, a)
                    }
                    5 => m::fn_80021C1C(),
                    6 => {
                        let a = fv();
                        let b = fv();
                        m::lbBgFlash_80021C48(a, b)
                    }
                    _ => m::fn_80021C80(&raw mut G),
                }
                g::hash_BgFlashUserData(&UD);
                g::hash_HSD_ObjAllocData(&m::bgflash_alloc_data);
                g::hash_HSD_GObj(&G);
                g::hash_HSD_GObj(&GNEW);
                hm(pi(m::flash_gobj));
                hm(pi(m::lbl_804D63DC));
                hm(m::flash_scale.to_bits());
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8 * 400);
    }
}
