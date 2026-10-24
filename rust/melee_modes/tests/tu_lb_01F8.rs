//! src/tu/lb_01F8.rs vs the C (gen/cref/tu/lb_01F8.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_01F8`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_01F8 as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_01F8_gen.rs"]
mod g;

static mut MB: [u8; 1024] = [0; 1024];
static mut MBN: i32 = 0;
static mut SOB: HSD_SObj = unsafe { core::mem::zeroed() };
static mut DEC: [u8; 64] = [0; 64];
static mut G: HSD_GObj = unsafe { core::mem::zeroed() };

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_01F8_matches_c() {
    unsafe {
        g::install();
        reg(&raw const MB);
        reg(&raw const SOB);
        reg(&raw const DEC);
        reg(&raw const G);
        reg(&raw const m::lbl_804335B8);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_lb_01F8.txt"));
        for op in 0..3u32 {
            for k in 0..400u32 {
                let mut ret = 0u32;
                case(op, k);
                MBN = 0;
                let l = &raw mut m::lbl_804335B8;
                *l = core::mem::zeroed();
                g::fill_lbl_804335B8_t(&mut *l);
                SOB = core::mem::zeroed();
                g::fill_HSD_SObj(&mut SOB);
                let mb = (&raw mut MB) as *mut u8;
                (*l).x20 = mb.add(600) as _;
                (*l).x44 = mb.add(700) as _;
                (*l).x68 = mb.add(800) as _;
                (*l).unk94 = mb.add(900) as _;
                (*l).x6C = if rnd() & 1 != 0 { 0x280 } else { fv() as u16 };
                match op {
                    0 => ret = (m::lbMthp8001F890(&raw mut G) == &raw mut SOB) as u32,
                    1 => m::lbMthp8001F928(&raw mut G, fv() as i32),
                    _ => {
                        let w = if rnd() & 1 != 0 { 0x280 } else { fv() as i32 };
                        let h = fv() as i32;
                        m::lbMthp8001FAA0(c"movie.mth".as_ptr(), w, h);
                    }
                }
                if TRACEON {
                    let (a, b) = (core::mem::size_of_val(&*l), core::mem::size_of_val(&SOB));
                    println!("D {} {}", a, b);
                    println!("S P {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X}", pi((*l).x20), pi((*l).x44), pi((*l).x68), pi((*l).x88.image), pi((*l).x90), pi((*l).unk94), pi((*l).x88.tlut));
                    for i in 0..a / 4 {
                        println!("D {:08X}", *((l as *const u32).add(i)));
                    }
                    for i in 0..b / 4 {
                        println!("S {:08X}", *(((&raw const SOB) as *const u32).add(i)));
                    }
                }
                g::hash_lbl_804335B8_t(&*l);
                g::hash_HSD_SObj(&SOB);
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 3 * 400);
    }
}
