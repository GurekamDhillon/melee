//! tu::lbfile vs the C (state oracle): gen/turef/tu_lbfile_ref.c is the game's lbfile.c compiled natively with
//! DevCom, the disc pump, DVD, the heap and the language getters stubbed and logged; the expected text is
//! tests/data/tu_lbfile.txt (python rust/melee_modes/gen/turef/tu_shard833.py tu_lbfile).
#![allow(static_mut_refs, non_snake_case)]
#[macro_use]
#[path = "lbtest/mod.rs"]
mod common_lb;
use common_lb::*;
use core::ffi::{c_char, c_void};
use melee_modes::tu::lbfile as t;
use melee_types::*;

struct Pend {
    on: bool,
    cb: HSD_DevComCallback,
    args: usize,
    countdown: i32,
}
static mut PEND: Pend = Pend { on: false, cb: None, args: 0, countdown: 0 };
static mut PUMPS: i32 = 0;
static mut G_CB2: i32 = 0;

unsafe extern "C" fn cb2(_a: i32, b: usize, _c: *mut c_void, _d: i32) {
    G_CB2 += 1;
    lg(&format!("cb2({});", b as u32));
}

mod gen_stubs {
    use super::*;
    include!("lbtest/lbfile_stubs.rs");
}

fn h_lb_800195D0() {
    unsafe {
        lg("lb_800195D0();");
        PUMPS += 1;
        assert!(PUMPS <= 100000, "pump loop");
        if PEND.on {
            let c = PEND.countdown;
            PEND.countdown -= 1;
            if c == 0 {
                PEND.on = false;
                (PEND.cb.unwrap())(0, PEND.args, core::ptr::null_mut(), 0);
            }
        }
    }
}
fn h_HSD_DevComRequest(file: i32, src: usize, dest: usize, size: usize, ty: i32, pri: i32, cb: HSD_DevComCallback, args: usize) -> i32 {
    unsafe {
        lg(&format!(
            "HSD_DevComRequest({},{},{},{},{},{},{},{});",
            file,
            pidu(src as u64),
            pidu(dest as u64),
            size as u32,
            ty,
            pri,
            pidf(cb.map_or(0usize, |f| f as usize)),
            pidu(args as u64)
        ));
        PEND.on = true;
        PEND.cb = cb;
        PEND.args = args;
        PEND.countdown = (rnd() % 3) as i32;
    }
    0
}
fn h_DVDFastOpen(entry: i32, info: *mut DVDFileInfo) -> i32 {
    let ok = (rnd() % 5 != 0) as i32;
    lg(&format!("DVDFastOpen({},{});", entry, pid(info as *const c_void)));
    if ok != 0 {
        unsafe { (*info).length = rnd() % 100000 };
    }
    ok
}
fn h_lbDvd_8001819C(name: *const c_char) -> *mut HSD_Archive {
    lg(&format!("lbDvd_8001819C({});", pids(name)));
    if rnd() % 3 == 0 {
        core::ptr::null_mut()
    } else {
        arena_at(512) as *mut HSD_Archive
    }
}
fn h_lbHeap_80015BD0(heap: i32, size: usize) -> *mut c_void {
    lg(&format!("lbHeap_80015BD0({},{});", heap, size as u32));
    if rnd() % 5 == 0 {
        core::ptr::null_mut()
    } else {
        let k = rnd() % 8;
        arena_at(k as usize * 64) as *mut c_void
    }
}

const NAMES: [&[u8]; 15] = [
    b"PlFx\0",
    b"IfAll.usd\0",
    b"GmTitle.\0",
    b"a.b\0",
    b"x.\0",
    b"\0",
    b"noext\0",
    b"abcdefghijklmnopqrstuvwxyz0\0",
    b"abcdefghijklmnopqrstuvwxy.\0",
    b"abcdefghijklmnopqrstuvwxyz01234\0",
    b"...\0",
    b".dat\0",
    b"Pl.Fx\0",
    b"ab.c\0",
    b"q..\0",
];

fn sumres(res: &[u8; 32]) -> u32 {
    let mut h: u32 = 2166136261;
    for &b in res {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    h
}

#[test]
fn lbfile() {
    arena_init();
    install_common();
    unsafe {
        preg("buf", arena_at(0), 1 << 20);
        pfreg("cb", t::lbFile_8001615C as usize);
        pfreg("cb2", cb2 as usize);
        gen_stubs::install_generated();
        t::ext::H_lb_800195D0 = Some(h_lb_800195D0);
        t::ext::H_HSD_DevComRequest = Some(h_HSD_DevComRequest);
        t::ext::H_DVDFastOpen = Some(h_DVDFastOpen);
        t::ext::H_lbDvd_8001819C = Some(h_lbDvd_8001819C);
        t::ext::H_lbHeap_80015BD0 = Some(h_lbHeap_80015BD0);
        let g_res = t::lbFileGetFullName(b"\0".as_ptr() as *const c_char);
        log_take();
        for i in 0..700 {
            let f = i % 12;
            let mut sz: usize = 0xABCD;
            let mut dst: *mut c_void = core::ptr::null_mut();
            let mut ret: i32 = 0;
            let args = rnd();
            let name = NAMES[(rnd() % 15) as usize].as_ptr() as *const c_char;
            let name_s = std::ffi::CStr::from_ptr(name).to_str().unwrap();
            let dstv: usize = if rnd() % 2 != 0 { arena_at(rnd() as usize % 8 * 64) as usize } else { 0x8000_0000usize + (rnd() % 4) as usize * 32 };
            log_take();
            PEND.on = false;
            PUMPS = 0;
            t::set_cancel(rnd() & 1 != 0);
            let cbsel = |s: bool| -> HSD_DevComCallback { if s { Some(cb2) } else { Some(t::lbFile_8001615C) } };
            let asserted = match f {
                0 => try_run(|| ret = (t::lbFileGetFullName(name) as *const u8 == g_res as *const u8) as i32),
                1 => {
                    let a = (rnd() % 50) as i32;
                    let b = (rnd() % 1000) as usize;
                    let c = (rnd() % 100000) as usize;
                    let d = (rnd() % 4) as i32;
                    let e = (rnd() % 3) as i32;
                    try_run(|| t::lbFile_800161C4(a, b, dstv, c, d, e))
                }
                2 => {
                    let a = (rnd() % 50) as i32;
                    try_run(|| ret = t::lbFile_8001634C(a) as u32 as i32)
                }
                3 => try_run(|| ret = t::lbFileGetSize(name) as u32 as i32),
                4 => {
                    let a = (rnd() % 50) as i32;
                    let b = (rnd() % 3) as i32;
                    let cb = cbsel(rnd() & 1 != 0);
                    try_run(|| t::lbFile_800164A4(a, dstv, &mut sz, b, cb, args as usize))
                }
                5 => {
                    let cb = cbsel(rnd() & 1 != 0);
                    try_run(|| t::lbFile_80016580(name, dstv as *mut c_void, &mut sz, cb, args as usize))
                }
                6 => try_run(|| t::lbFile_8001668C(name, dstv as *mut c_void, &mut sz)),
                7 => try_run(|| t::lbFile_80016760(name, &mut dst, &mut sz)),
                8 => {
                    let h = (rnd() % 7) as i32;
                    try_run(|| dst = t::lbFile_LoadToHeap(h, name, &mut sz))
                }
                9 => {
                    let cap = (rnd() % 100000) as usize;
                    let d = if rnd() % 6 == 0 { core::ptr::null_mut() } else { dstv as *mut c_void };
                    try_run(|| ret = t::lbFile_LoadInto(name, d, cap, &mut sz))
                }
                10 => {
                    let h = (rnd() % 7) as i32;
                    try_run(|| ret = t::lbFile_800168A0(h, name, &mut dst, &mut sz) as i32)
                }
                _ => {
                    let cf = (rnd() % 3 == 0) as i32;
                    try_run(|| t::lbFile_8001615C_body(cf))
                }
            };
            let res = t::result_buffer();
            let rs = std::str::from_utf8(&res[..res.iter().position(|&b| b == 0).unwrap()]).unwrap();
            outf!(
                "{} f={} name={} asserted={} ret={} cancel={} sz={} dst={} res={} sum={:08x} pend={} g_cb2={} log={}\n",
                i,
                f,
                name_s,
                asserted as i32,
                ret,
                t::cancel_flag() as i32,
                sz as u32,
                pid(dst),
                rs,
                sumres(&res),
                PEND.on as i32,
                G_CB2,
                log_take()
            );
        }
    }
    check("tu_lbfile");
}
