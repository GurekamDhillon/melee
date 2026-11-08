//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbcommand.c.
//! src/tu/lbcommand.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbcommand`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_Command_00: u32 = 0;
const OP_Command_01: u32 = 1;
const OP_Command_02: u32 = 2;
const OP_Command_03: u32 = 3;
const OP_Command_04: u32 = 4;
const OP_Command_05: u32 = 5;
const OP_Command_06: u32 = 6;
const OP_Command_07: u32 = 7;
const OP_Command_08: u32 = 8;
const OP_Command_09: u32 = 9;
const OP_Command_Execute: u32 = 10;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbcommand as m;
use mtu_harness::*;
#[path = "mtu/lbcommand_gen.rs"]
mod g;


static mut CI: [CommandInfo; 3] = unsafe { core::mem::zeroed() };
/// The command words. A stream is 4-byte words and its jump targets (Command_05/07) are 32-bit guest addresses, so the
/// words live below 4 GiB (a 64-bit process has no low addresses by default: `VirtualAlloc` at a hint).
struct LowCu(*mut CmdUnion);
impl core::ops::Index<usize> for LowCu {
    type Output = CmdUnion;
    fn index(&self, i: usize) -> &CmdUnion { assert!(i < 20); unsafe { &*self.base().add(i) } }
}
impl core::ops::IndexMut<usize> for LowCu {
    fn index_mut(&mut self, i: usize) -> &mut CmdUnion { assert!(i < 20); unsafe { &mut *self.base().add(i) } }
}
impl LowCu {
    fn base(&self) -> *mut CmdUnion {
        #[cfg(windows)]
        extern "system" { fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void; }
        // Unix: an anonymous mapping at exactly the hint (MAP_FIXED_NOREPLACE), null when taken, like VirtualAlloc.
        #[cfg(not(windows))]
        unsafe fn VirtualAlloc(addr: *mut c_void, size: usize, _ty: u32, _prot: u32) -> *mut c_void {
            extern "C" { fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut c_void; }
            let p = mmap(addr, size, 3, 0x02 | 0x20 | 0x10_0000, -1, 0);
            if p as isize == -1 || p != addr { core::ptr::null_mut() } else { p }
        }
        if self.0.is_null() {
            let mut hint = 0x8100_0000usize;
            let p = loop {
                let p = unsafe { VirtualAlloc(hint as *mut c_void, 4096, 0x3000, 0x04) };
                if !p.is_null() { break p; }
                hint += 0x0100_0000;
                assert!(hint < 0xF000_0000, "no low memory");
            };
            unsafe { (*(&raw const CU as *mut LowCu)).0 = p as *mut CmdUnion };
        }
        self.0
    }
}
unsafe impl Sync for LowCu {}
static mut CU: LowCu = LowCu(core::ptr::null_mut());
static mut CMD: u32 = 0;
fn hp<T>(p: *const T) -> u32 { let v = p as usize as u64; if (v >> 32) == 0 { v as u32 } else { pi(p) } }
unsafe fn setcu(cmd: u32) {
    for q in 0..16usize {
        CU[q] = core::mem::zeroed();
        match cmd {
            1 => CU[q].Command_00.set_value(fv()),
            2 => CU[q].Command_02.set_value(fv()),
            3 => CU[q].Command_03.set_value(fv() % 5),
            5 | 7 => CU[q].Command_05.ptr = ((&raw mut CU[(fv() % 16) as usize]) as usize as u32).to_be(),
            9 => { CU[q].Command_09.set_id(fv()); CU[q].Command_09.set_param_1(fv()); CU[q].Command_09.set_param_2(fv()); }
            _ => CU[q].Command_04.x = fv(),
        }
    }
}
unsafe fn setci(cmd: u32) {
    for q in 0..3usize {
        let c = &mut CI[q];
        *c = core::mem::zeroed();
        c.timer = rw(); c.frame_count = rw(); c.anon_8.u = &raw mut CU[(fv() % 16) as usize];
        c.loop_count = if cmd == 3 { fv() % 2 } else if cmd == 4 { 2 } else if cmd == 5 { fv() % 3 } else if cmd == 6 { 1 + fv() % 3 } else { fv() % 4 };
        c.event_return[0] = &raw mut CU[(fv() % 16) as usize]; c.event_return[1] = (fv() % 4) as usize as *mut CmdUnion; c.event_return[2] = &raw mut CU[(fv() % 16) as usize];
        c.loop_count_dup = fv(); c.unk_x18 = fv();
    }
}


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, h: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(h)); }
pub unsafe extern "C" fn cbi1(a: i32) { t(7006); t(a as u32); }
pub unsafe extern "C" fn cbi2(a: i32) { t(7007); t(a as u32); }
pub fn pick_fn(kind: i32) -> usize {
    let r = rnd();
    if r & 3 == 0 { return 0; }
    match kind {
        1 => if r & 4 != 0 { cbev1 as usize } else { cbev2 as usize },
        2 => if r & 4 != 0 { cbp1 as usize } else { cbp2 as usize },
        3 => cbev3 as usize,
        4 => if r & 4 != 0 { cbi1 as usize } else { cbi2 as usize },
        _ => 0,
    }
}
pub fn pick_cb(kind: i32) -> usize { let f = pick_fn(kind); if f != 0 { f } else if kind == 2 { cbp1 as usize } else if kind == 3 { cbev3 as usize } else if kind == 4 { cbi1 as usize } else { cbev1 as usize } }
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize); }
pub fn preg_world() {
    unsafe {
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

#[test]
#[ignore = "the C oracle walks native 8-byte CmdUnion arrays; the port walks the raw 4-byte stream (tests/lbcommand_raw.rs)"]
fn lbcommand_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const CI, core::mem::size_of_val(&CI)); preg(&raw const CU, core::mem::size_of_val(&CU));
        let mut c = Check::new(include_str!("data/mtu_lbcommand.txt"));
        let mut total = 0u32;
        for op in 0..11u32 {
            let n = match op {
                0 => 300u32,
                1 => 300u32,
                2 => 300u32,
                3 => 300u32,
                4 => 300u32,
                5 => 300u32,
                6 => 300u32,
                7 => 300u32,
                8 => 300u32,
                9 => 300u32,
                10 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ CMD = if op < 10 { op } else { fv() % 12 }; setcu(CMD); setci(CMD); }
                let mut ret = 0u32;
                match op {
                    0 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_00(a0);
 }
                    1 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_01(a0);
 }
                    2 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_02(a0);
 }
                    3 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_03(a0);
 }
                    4 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_04(a0);
 }
                    5 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_05(a0);
 }
                    6 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_06(a0);
 }
                    7 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_07(a0);
 }
                    8 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_08(a0);
 }
                    9 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
m::Command_09(a0);
 }
                    10 => { 
let a0 = &raw mut CI[(fv() % 3) as usize];
let a1 = CMD;
ret = m::Command_Execute(a0, a1) as u32;
 }
                    _ => unreachable!(),
                }
                hash_world();
{ for q in 0..3usize { let c = &CI[q]; hm(c.timer.to_bits()); hm(c.frame_count.to_bits()); hm(pi(c.anon_8.u)); hm(c.loop_count); for k in 0..3usize { hm(hp(c.event_return[k])); } hm(c.loop_count_dup); hm(c.unk_x18); } }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
