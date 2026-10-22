//! tu::grfzerocar vs the C (gen/cref/tu/grfzerocar.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_world/gen/cref/run_tu.py grfzerocar`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "mtu_harness_cref/mod.rs"]
mod mtu_harness;
use melee_types::*;
use melee_world::tu::grfzerocar as m;
use mtu_harness::*;
#[path = "mtu/grfzerocar_gen.rs"]
mod g;

static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut JR: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
static mut JP: [HSD_JObj; 64] = unsafe { core::mem::zeroed() };
static mut JN: usize = 0;
static mut ARCH: UnkArchiveStruct = unsafe { core::mem::zeroed() };
static mut USD: UnkStageDat = unsafe { core::mem::zeroed() };
#[repr(C, align(16))]
struct Buf([u8; 4 * 0x34 + 64]);
static mut DMBB: Buf = Buf([0; 4 * 0x34 + 64]);
static mut CDATA: [i16; 30] = [0; 30];

#[test]
fn grfzerocar_matches_c() {
    unsafe {
        g::install();
        preg(&raw const G, core::mem::size_of::<HSD_GObj>());
        preg(&raw const JR, core::mem::size_of_val(&JR));
        preg(&raw const JP, core::mem::size_of_val(&JP));
        preg(&raw const ARCH, core::mem::size_of::<UnkArchiveStruct>());
        preg(&raw const USD, core::mem::size_of::<UnkStageDat>());
        preg(&raw const DMBB, 4 * 0x34 + 64);
        preg(&raw const CDATA, 60);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_grfzerocar.txt"));
        for op in 0..2u32 {
            for k in 0..400u32 {
                case(op, k);
                G = core::mem::zeroed();
                JR = core::mem::zeroed();
                JP = core::mem::zeroed();
                DMBB = Buf([0; 4 * 0x34 + 64]);
                g::fill_HSD_GObj(&mut G);
                for i in 0..2 {
                    g::fill_HSD_JObj(&mut JR[i]);
                }
                for i in 0..64 {
                    g::fill_HSD_JObj(&mut JP[i]);
                }
                JN = 0;
                JR[0].child = &raw mut JR[1];
                G.hsd_obj = &raw mut JR[0] as *mut _;
                for i in 0..30 {
                    CDATA[i] = (100 + fv() % 100) as i16;
                }
                ARCH.unk0 = core::ptr::null_mut();
                ARCH.unk4 = &raw mut USD;
                USD.unk8 = DMBB.0.as_mut_ptr() as *mut _;
                for i in [0usize, 2] {
                    let d = DMBB.0.as_mut_ptr().add(i * 0x34) as *mut DynamicModelDesc;
                    (*d).anims = if fv() & 1 != 0 { DMBB.0.as_mut_ptr().add(8) as *mut _ } else { core::ptr::null_mut() };
                    (*d).matanims = if fv() & 1 != 0 { DMBB.0.as_mut_ptr().add(16) as *mut _ } else { core::ptr::null_mut() };
                }
                *(DMBB.0.as_mut_ptr().add(8) as *mut *mut u8) = DMBB.0.as_mut_ptr().add(24);
                *(DMBB.0.as_mut_ptr().add(16) as *mut *mut u8) = DMBB.0.as_mut_ptr().add(32);
                let ret = 0u32;
                match op {
                    0 => {
                        let cnt = ((fv() % 2) * 2) as i32;
                        let mode = (fv() & 1) as i32;
                        m::grFZeroCar_801CAFBC(&raw mut G, CDATA.as_mut_ptr() as *mut _, cnt, mode)
                    }
                    _ => {
                        for i in 0..30 {
                            let e = &m::grFZeroCar_803E0BD8[i];
                            hm(e.unk0 as i32 as u32);
                            hm(e.unk2 as i32 as u32);
                            for j in 0..4 {
                                hm(e.children[j] as i32 as u32);
                            }
                        }
                        let r = &m::grFZeroCar_803B7E50;
                        for f in [r.scale.x, r.scale.y, r.scale.z, r.translate.x, r.translate.y, r.translate.z, r.rotate.x, r.rotate.y, r.rotate.z, r.rotate.w] {
                            hm(f.to_bits());
                        }
                        hm(rh(&m::grFZeroCar_804D4598));
                    }
                }
                g::hash_HSD_GObj(&G);
                for i in 0..2 {
                    g::hash_HSD_JObj(&JR[i]);
                }
                for i in 0..64 {
                    g::hash_HSD_JObj(&JP[i]);
                }
                hm(JN as u32);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 800);
    }
}
