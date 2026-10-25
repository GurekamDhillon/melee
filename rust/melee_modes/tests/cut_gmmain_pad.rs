//! tu::gmmain (gmMain_8015FD24 pad clamp setup, gmMain_8015FDA0, gmMain_8015FDA4 debug level) vs the C text
//! itself (gen/cref/cut/gmmain_pad.c; regenerate with `python rust/melee_modes/gen/cref/cutref.py gmmain_pad`).
//! The port's `ext` hooks are backed here by the same globals and recording stubs as the C driver.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals)]
mod mtu_harness;
use melee_modes::tu::gmmain as m;
use melee_types::*;
use mtu_harness::*;

static mut TDB: i32 = 0;
static mut DB6B20: bool = false;
static mut LAUNCH: u16 = 0;
static mut PADLIB: PadLibData = unsafe { core::mem::zeroed() };

fn shs(s: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in s.bytes() {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    h
}

#[test]
fn gmmain_pad_matches_c() {
    unsafe {
        use m::ext::*;
        PADSETSPEC = Some(|s| { t(1); t(s as u32); });
        HSD_PADINIT = Some(|n, p, rn, r| { t(2); t(n as u32); t(pi(p)); t(rn as u32); t(pi(r)); });
        HSD_PADLIBDATA = Some(|| &raw mut PADLIB);
        DVDCONVERTPATHTOENTRYNUM = Some(|p| {
            t(3);
            t(shs(p));
            if TDB == DbLKind_NoDebugRom as i32 && fv() & 1 != 0 { -1 } else { (fv() % 5) as i32 }
        });
        SET_DB_804D6B20 = Some(|v| DB6B20 = v);
        DB_GAMELAUNCHBUTTONSTATE = Some(|| LAUNCH as u32);
        DBLEVEL = Some(|| TDB);
        SET_DBLEVEL = Some(|v| TDB = v);
        HSD_ASSERT_HOOK = Some(|_line, cond| assert!(cond, "HSD_ASSERT would hang the C"));
        preg(&raw const m::gmMain_8046B108, core::mem::size_of_val(&m::gmMain_8046B108));
        preg(&raw const m::gmMain_8046B1F8, core::mem::size_of_val(&m::gmMain_8046B1F8));
        let mut c = Check::new(include_str!("data/cut_gmmain_pad.txt"));
        for op in 0..3u32 {
            for k in 0..600u32 {
                case(op, k);
                let b = &raw mut PADLIB as *mut u8;
                for i in 0..core::mem::size_of::<PadLibData>() {
                    *b.add(i) = fv() as u8;
                }
                TDB = (fv() % 6) as i32;
                DB6B20 = fv() & 1 != 0;
                LAUNCH = fv() as u16;
                if op == 2 && (k & 1) != 0 {
                    TDB = DbLKind_NoDebugRom as i32;
                }
                match op {
                    0 => m::gmMain_8015FD24(),
                    1 => m::gmMain_8015FDA0(fv()),
                    _ => m::gmMain_8015FDA4(),
                }
                hm(TDB as u32);
                hm(DB6B20 as u32);
                hm(rh(&PADLIB));
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 1800);
    }
}
