//! melee_runtime::tu::video::{HSD_VISetConfigure, HSD_VISetBlack} (the melee_modes `ext` forwards to them) vs
//! the C text itself (gen/cref/cut/video_cfg.c; regenerate with `python rust/melee_modes/gen/cref/cutref.py video_cfg`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_runtime::tu::video as v;
use melee_types::*;
use mtu_harness::*;

#[test]
fn video_cfg_matches_c() {
    unsafe {
        assert_eq!(core::mem::size_of::<HSD_VIInfo>(), core::mem::size_of_val(&v::HSD_VIData));
        let mut c = Check::new(include_str!("data/cut_video_cfg.txt"));
        for op in 0..2u32 {
            for k in 0..300u32 {
                case(op, k);
                let n = core::mem::size_of::<HSD_VIInfo>();
                let b = &raw mut v::HSD_VIData as *mut u8;
                for i in 0..n {
                    *b.add(i) = fv() as u8;
                }
                let mut rm: GXRenderModeObj = core::mem::zeroed();
                let r = &raw mut rm as *mut u8;
                for i in 0..core::mem::size_of::<GXRenderModeObj>() {
                    *r.add(i) = fv() as u8;
                }
                if op == 0 {
                    v::HSD_VISetConfigure(&raw mut rm);
                } else {
                    v::HSD_VISetBlack(fv() as i32);
                }
                hm(rh(&v::HSD_VIData));
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 600);
    }
}
