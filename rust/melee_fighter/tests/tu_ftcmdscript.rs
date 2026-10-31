//! src/tu/ftcmdscript.rs vs the C (gen/cref/tu/ftcmdscript.c, shipping contraction),
//! 6000 cases: the interpreter over random scripts and the buffer writers.
#![allow(static_mut_refs)]
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftcmdscript as c;
use melee_fighter::types::*;
use tu_s25::*;

static mut LA: f32 = 0.0;
fn lb(y: f32, x: f32) -> f32 {
    t(y.to_bits());
    t(x.to_bits());
    unsafe { LA }
}

const CODES: [u8; 51] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 127, 128, 129, 130, 131,
    132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 192, 193, 194,
];
fn code(h: &mut H) -> u8 { CODES[(h.rnd() % 51) as usize] }
fn arg(h: &mut H) -> u8 {
    if h.rnd() & 1 != 0 { (h.rnd() % 4) as u8 } else { h.rnd() as u8 }
}

fn hstate(h: &mut H, f: &Fighter) {
    let d = &f.cpu;
    h.hm(d.buttons);
    h.hm(d.lstick.x as u8 as u32);
    h.hm(d.lstick.y as u8 as u32);
    h.hm(d.cstick.x as u8 as u32);
    h.hm(d.cstick.y as u8 as u32);
    h.hm(d.ltrigger as u32);
    h.hm(d.rtrigger as u32);
    h.hm(d.x18 as u32);
    h.hm(d.command_duration);
    h.hm(if d.csP.is_null() { 0xFFFFFFFF } else { (d.csP as usize - d.buffer.as_ptr() as usize) as u32 });
}

#[test]
fn ftcmdscript_vs_c() {
    let exp = expected("tu_ftcmdscript");
    let mut sc: Box<[[u8; 40]; 4]> = zeroed();
    let mut scp: [*mut u8; 4] = [core::ptr::null_mut(); 4];
    let mut tb: Box<Fighter_804D64FC_t> = zeroed();
    tb.cmdscripts = scp.as_mut_ptr();
    let cd: Box<ftCommonData> = zeroed();
    unsafe {
        c::ext::Fighter_804D64FC = &*tb as *const _ as *mut _;
        c::ext::LB_8000D008 = Some(lb);
        hooks::P_FT_COMMON_DATA = &*cd;
    }
    let mut n = 0;
    for op in 0..2u32 {
        for k in 0..3000u32 {
            let mut h = H::case(op, k);
            unsafe { TR = 0 };
            let mut f: Box<Fighter> = zeroed();
            let mut f2: Box<Fighter> = zeroed();
            for i in 0..256 {
                let r = h.rnd() % 8;
                f.cpu.buffer[i] = (if r == 0 { 127 } else if r < 6 { code(&mut h) } else { arg(&mut h) }) as i8;
            }
            f.cpu.buffer[253] = 127;
            f.cpu.buffer[254] = 127;
            f.cpu.buffer[255] = 127;
            f.cur_pos.x = h.rw();
            f.cur_pos.y = h.rw();
            f2.cur_pos.x = h.rw();
            f2.cur_pos.y = h.rw();
            f.cpu.x54.x = h.rw();
            f.cpu.x54.y = h.rw();
            f.facing_dir = h.rg();
            f.motion_id = (h.rnd() % 4) as _;
            f.cpu.x44 = if h.rnd() & 1 != 0 { &mut *f2 } else { core::ptr::null_mut() };
            f.cpu.lstick.x = h.rnd() as i8;
            f.cpu.lstick.y = h.rnd() as i8;
            f.cpu.buttons = h.rnd();
            unsafe { LA = h.rw() };
            if op == 0 {
                f.cpu.csP = if h.rnd() % 8 != 0 {
                    let o = (h.rnd() % 32) as usize;
                    unsafe { f.cpu.buffer.as_mut_ptr().add(o) }
                } else {
                    core::ptr::null_mut()
                };
                f.cpu.command_duration = h.rnd() % 3;
            } else {
                for j in 0..4 {
                    let mut nn = 0;
                    let m = h.rnd() % 8;
                    for _ in 0..m {
                        let cc = code(&mut h);
                        if cc == 127 {
                            continue;
                        }
                        sc[j][nn] = cc;
                        nn += 1;
                        if cc > 127 {
                            sc[j][nn] = arg(&mut h);
                            nn += 1;
                        }
                        if cc > 191 {
                            sc[j][nn] = arg(&mut h);
                            nn += 1;
                        }
                    }
                    sc[j][nn] = 127;
                    scp[j] = sc[j].as_mut_ptr();
                }
                c::ftco_800b4a78(&mut f);
                let i = (h.rnd() % 4) as i32;
                c::ftco_800b4880(&mut f, i);
                if h.rnd() & 1 != 0 {
                    let a = code(&mut h);
                    let b = arg(&mut h);
                    c::ftco_800b46b8(&mut f, a, b);
                }
                if h.rnd() & 1 != 0 {
                    let a = code(&mut h);
                    let b = arg(&mut h);
                    let d = arg(&mut h);
                    c::ftco_800b4778(&mut f, a, b, d);
                }
                if h.rnd() & 1 != 0 {
                    let i = (h.rnd() % 4) as i32;
                    c::ftco_800b4880(&mut f, i);
                }
                c::ftco_800b49f4(&mut f);
            }
            h.h = 0x811C9DC5;
            if op == 1 {
                h.hm((f.cpu.write_pos as usize - f.cpu.buffer.as_ptr() as usize) as u32);
                for i in 0..64 {
                    h.hm(f.cpu.buffer[i] as u8 as u32);
                }
                f.cpu.buffer[253] = 127;
                f.cpu.buffer[254] = 127;
                f.cpu.buffer[255] = 127;
            }
            for _ in 0..8 {
                unsafe { LA = h.rw() };
                c::ftco_800b3e04(&mut f);
                hstate(&mut h, &f);
            }
            let got = format!("{} {} {:08X} {:08X}", op, k, h.h, unsafe { TR });
            assert_eq!(got, exp[n], "case {op} {k}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}
