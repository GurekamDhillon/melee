//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/psdisp.c.
//! src/tu/psdisp.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py psdisp`).
//! C functions of the TU this test covers (the private ones through their public callers): calcTornadoLastPos, getClrTrail, getColorMatAmb, getColorPrimEnv, particleSort, psAbsGtF32, psAbsLtF32, psDispParticles, psDispSub, psDispSubAPPSRTPoint, psDispSubAbsGtF32, psDispSubAbsLtF32, psDispSubAppSRT, psDispSubMakePolygon, psDispSubPoint, psDispSubPointTrail, psMaskAbsF32, psMaskAbsGtF32, psMaskAbsLtF32, psScaleAppSRTAxes, psSetColor, psSetCurrentMtx, psSetupVtxFormat, psUpdateBillboardAxes, setBlendMode, setVtxDesc, setupChanCtrl, setupChanReg, setupTevReg.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_calcTornadoLastPos: u32 = 0;
const OP_getColorPrimEnv: u32 = 1;
const OP_setupChanCtrl: u32 = 2;
const OP_setupChanReg: u32 = 3;
const OP_setupTevReg: u32 = 4;
const OP_particleSort: u32 = 5;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::psdisp as m;
use mtu_harness::*;
#[path = "mtu/psdisp_gen.rs"]
mod g;



pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut P0: [HSD_Particle; 4] = unsafe { core::mem::zeroed() };
pub static mut GEN: [HSD_Generator; 2] = unsafe { core::mem::zeroed() };
pub static mut P1: [GXColor; 2] = unsafe { core::mem::zeroed() };
pub static mut SRT: [HSD_psAppSRT; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_Particle() -> *mut HSD_Particle { let r = rnd(); unsafe { &raw mut P0[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_Particle() -> *mut HSD_Particle { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P0[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_Generator() -> *mut HSD_Generator { let r = rnd(); unsafe { &raw mut GEN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Generator() -> *mut HSD_Generator { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut GEN[((r >> 8) % 2) as usize] } } }
pub fn pick_GXColor() -> *mut GXColor { let r = rnd(); unsafe { &raw mut P1[((r >> 8) % 2) as usize] } }
pub fn picko_GXColor() -> *mut GXColor { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P1[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_psAppSRT() -> *mut HSD_psAppSRT { let r = rnd(); unsafe { &raw mut SRT[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_psAppSRT() -> *mut HSD_psAppSRT { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SRT[((r >> 8) % 2) as usize] } } }
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
pub fn freg_world() { freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cbi1 as usize); freg(cbi2 as usize);  }
pub fn preg_world() {
    unsafe {
        preg(&raw const P0, core::mem::size_of_val(&P0));
        preg(&raw const GEN, core::mem::size_of_val(&GEN));
        preg(&raw const P1, core::mem::size_of_val(&P1));
        preg(&raw const SRT, core::mem::size_of_val(&SRT));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    P0 = core::mem::zeroed();
    GEN = core::mem::zeroed();
    P1 = core::mem::zeroed();
    SRT = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..4 { g::fill_HSD_Particle(&mut P0[i]); }
    for i in 0..2 { g::fill_HSD_Generator(&mut GEN[i]); }
    for i in 0..2 { g::fill_GXColor(&mut P1[i]); }
    for i in 0..2 { g::fill_HSD_psAppSRT(&mut SRT[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_Particle(&P0[i]); }
    for i in 0..2 { g::hash_HSD_Generator(&GEN[i]); }
    for i in 0..2 { g::hash_GXColor(&P1[i]); }
    for i in 0..2 { g::hash_HSD_psAppSRT(&SRT[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { psdisp_matches_c(); println!("tu_psdisp: ok"); }

fn psdisp_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_psdisp.txt"));
        let mut total = 0u32;
        for op in 0..6u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                3 => 400u32,
                4 => 400u32,
                5 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{  for q in 0..4usize { P0[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].r#gen = &raw mut GEN[(fv() & 1) as usize]; P0[q].appsrt = if fv() & 1 != 0 { &raw mut SRT[(fv() & 1) as usize] } else { null_mut() };    P0[q].kind = fv() ^ (fv() << 13) ^ (fv() << 26); if fv() & 1 != 0 { P0[q].primColCount = (1 + fv() % 100) as u16; P0[q].primColRemain = (fv() % 101) as u16; } else { P0[q].primColCount = 0; }    if fv() & 1 != 0 { P0[q].envColCount = (1 + fv() % 100) as u16; P0[q].envColRemain = (fv() % 101) as u16; } else { P0[q].envColCount = 0; }    if fv() & 1 != 0 { P0[q].matColCount = (1 + fv() % 100) as u16; P0[q].matColRemain = (fv() % 101) as u16; } else { P0[q].matColCount = 0; }    if fv() & 1 != 0 { P0[q].ambColCount = (1 + fv() % 100) as u16; P0[q].ambColRemain = (fv() % 101) as u16; } else { P0[q].ambColCount = 0; } }  for q in 0..16usize { m::ext::hsd_804D0908[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } for q in 0..17usize { m::particle_list[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = pick_HSD_Particle();
let a1 = &raw mut FA[(rnd() % 4) as usize];
let a2 = &raw mut FA[(rnd() % 4) as usize];
let a3 = &raw mut FA[(rnd() % 4) as usize];
m::calcTornadoLastPos(a0, a1, a2, a3);
 }
                    1 => { 
let a0 = pick_HSD_Particle();
let a1 = pick_GXColor();
let a2 = pick_GXColor();
m::getColorPrimEnv(a0, a1, a2);
 }
                    2 => { 
let a0 = pick_HSD_Particle();
m::setupChanCtrl(a0);
 }
                    3 => { 
let a0 = pick_HSD_Particle();
m::setupChanReg(a0);
 }
                    4 => { 
let a0 = pick_HSD_Particle();
m::setupTevReg(a0);
 }
                    5 => { 
let a0 = (fv() & 15) as i32;
let a1 = fv() as _;
let a2 = (&raw mut PPA[0]) as *mut *mut HSD_Particle;
let a3 = (&raw mut PPA[1]) as *mut *mut HSD_Particle;
ret = pi(m::particleSort(a0, a1, a2, a3));
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();

                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
