//! Every melee_modes copy of the jobj.h inline setters (HSD_JObjSetRotationX / SetScaleY / SetTranslateX /
//! SetTranslateY) vs the header's C text (gen/cref/cut/jobj_setters.c; regenerate with
//! `python rust/melee_modes/gen/cref/cutref.py jobj_setters`). `HSD_JObjSetMtxDirtySub` is a recording hook
//! on both sides; the state hash covers every byte of the joint. (script_game.rs carries a seventh, c2rust-identical copy
//! of SetRotationX behind the `oracle::ext_call` recorder; it is not run here.)
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::{geno_game, mnitemsw, mnmain, mnname};
use melee_types::*;
use mtu_harness::*;

static mut J: HSD_JObj = unsafe { core::mem::zeroed() };

type Setter = unsafe extern "C" fn(*mut HSD_JObj, f32);

unsafe fn dirty_sub(j: *mut HSD_JObj) {
    t(500);
    t(pi(j));
}

unsafe extern "C" fn dirty_sub_c(j: *mut HSD_JObj) {
    dirty_sub(j)
}

#[test]
fn jobj_setters_match_c() {
    unsafe {
        geno_game::ext::hook::HSD_JObjSetMtxDirtySub = Some(|j| dirty_sub(j));
        mnname::ext::H_HSD_JObjSetMtxDirtySub = Some(dirty_sub_c);
        mnitemsw::ext::H_HSD_JObjSetMtxDirtySub = Some(dirty_sub_c);
        mnmain::ext::H_HSD_JObjSetMtxDirtySub = Some(dirty_sub_c);
        preg(&raw const J, core::mem::size_of::<HSD_JObj>());
        let impls: [(u32, &str, Setter); 6] = [
            (0, "geno_game::HSD_JObjSetRotationX", geno_game::HSD_JObjSetRotationX),
            (1, "mnmain::HSD_JObjSetScaleY", mnmain::HSD_JObjSetScaleY),
            (2, "mnitemsw::HSD_JObjSetTranslateX", mnitemsw::HSD_JObjSetTranslateX),
            (2, "mnname::HSD_JObjSetTranslateX", mnname::HSD_JObjSetTranslateX),
            (3, "mnitemsw::HSD_JObjSetTranslateY", mnitemsw::HSD_JObjSetTranslateY),
            (3, "mnname::HSD_JObjSetTranslateY", mnname::HSD_JObjSetTranslateY),
        ];
        let want = include_str!("data/cut_jobj_setters.txt");
        for (op, name, f) in impls {
            let mut lines = want.lines().skip(op as usize * 800);
            for k in 0..800u32 {
                case(op, k);
                let b = &raw mut J as *mut u8;
                for i in 0..core::mem::size_of::<HSD_JObj>() {
                    *b.add(i) = fv() as u8;
                }
                J.flags = fv() & !(JOBJ_USE_QUATERNION as u32);
                if fv() & 1 != 0 {
                    J.flags &= !(JOBJ_MTX_INDEP_SRT as u32);
                }
                if fv() & 1 != 0 {
                    J.flags &= !(JOBJ_USER_DEF_MTX as u32);
                }
                let v = rg();
                f(&raw mut J, v);
                hm(rh(&J));
                let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, 0, H, TR);
                assert_eq!(got, lines.next().unwrap(), "{name} case {k}");
            }
        }
    }
}
