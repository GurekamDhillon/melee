//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/particle.c.
//! src/tu/particle.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py particle`).
//! C functions of the TU this test covers (the private ones through their public callers): HSD_StageSlotParticleLinkClear, hsd_803983A4, hsd_80398A08, hsd_80398F0C, hsd_80398F8C, hsd_803991D8, hsd_8039930C, hsd_8039CEAC, hsd_8039CF4C, hsd_8039D048, hsd_8039D0A0, psEnableTexture, psGenerateParticle0, psInitDataBank, psInitDataBankLoad, psInitDataBankLocate, psReadFloat, psSpawnChild.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_psGenerateParticle0: u32 = 0;
const OP_hsd_80398F0C: u32 = 1;
const OP_hsd_80398F8C: u32 = 2;
const OP_hsd_8039930C: u32 = 3;
const OP_hsd_8039D0A0: u32 = 4;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::particle as m;
use mtu_harness::*;
#[path = "mtu/particle_gen.rs"]
mod g;


unsafe extern "C" fn cb_hook(p: *mut HSD_Particle) -> i32 { t(8401); t(pi(p)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_part(p: *mut HSD_Particle) -> i32 { t(8402); t(pi(p)); if fv() & 1 != 0 { -1 } else { 0 } }


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
pub static mut SRT: [HSD_psAppSRT; 2] = unsafe { core::mem::zeroed() };
pub static mut UF: [_PSUserFunc; 2] = unsafe { core::mem::zeroed() };
pub static mut J: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_Particle() -> *mut HSD_Particle { let r = rnd(); unsafe { &raw mut P0[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_Particle() -> *mut HSD_Particle { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut P0[((r >> 8) % 4) as usize] } } }
pub fn pick_HSD_Generator() -> *mut HSD_Generator { let r = rnd(); unsafe { &raw mut GEN[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_Generator() -> *mut HSD_Generator { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut GEN[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_psAppSRT() -> *mut HSD_psAppSRT { let r = rnd(); unsafe { &raw mut SRT[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_psAppSRT() -> *mut HSD_psAppSRT { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SRT[((r >> 8) % 2) as usize] } } }
pub fn pick__PSUserFunc() -> *mut _PSUserFunc { let r = rnd(); unsafe { &raw mut UF[((r >> 8) % 2) as usize] } }
pub fn picko__PSUserFunc() -> *mut _PSUserFunc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut UF[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut J[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut J[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const SRT, core::mem::size_of_val(&SRT));
        preg(&raw const UF, core::mem::size_of_val(&UF));
        preg(&raw const J, core::mem::size_of_val(&J));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    P0 = core::mem::zeroed();
    GEN = core::mem::zeroed();
    SRT = core::mem::zeroed();
    UF = core::mem::zeroed();
    J = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..4 { g::fill_HSD_Particle(&mut P0[i]); }
    for i in 0..2 { g::fill_HSD_Generator(&mut GEN[i]); }
    for i in 0..2 { g::fill_HSD_psAppSRT(&mut SRT[i]); }
    for i in 0..2 { g::fill__PSUserFunc(&mut UF[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut J[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_Particle(&P0[i]); }
    for i in 0..2 { g::hash_HSD_Generator(&GEN[i]); }
    for i in 0..2 { g::hash_HSD_psAppSRT(&SRT[i]); }
    for i in 0..2 { g::hash__PSUserFunc(&UF[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&J[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { particle_matches_c(); println!("tu_particle: ok"); }

fn particle_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
freg(cb_hook as usize); freg(cb_part as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_particle.txt"));
        let mut total = 0u32;
        for op in 0..5u32 {
            let n = match op {
                0 => 400u32,
                1 => 400u32,
                2 => 400u32,
                3 => 400u32,
                4 => 400u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{  for q in 0..2usize { UF[q].hookCreate = if fv() & 1 != 0 { Some(cb_hook) } else { None }; UF[q].hookDelete = if fv() & 1 != 0 { Some(cb_hook) } else { None }; }  for q in 0..2usize { GEN[q].userfunc = if fv() & 1 != 0 { &raw mut UF[(fv() & 1) as usize] } else { null_mut() }; GEN[q].appsrt = if fv() & 1 != 0 { &raw mut SRT[(fv() & 1) as usize] } else { null_mut() }; GEN[q].linkNo = (fv() % 16) as u8; GEN[q].idnum = (fv() % 3) as u16; GEN[q].numChild = fv() % 8; }  for q in 0..4usize { P0[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].r#gen = &raw mut GEN[(fv() & 1) as usize]; P0[q].appsrt = if fv() & 1 != 0 { &raw mut SRT[(fv() & 1) as usize] } else { null_mut() }; P0[q].idnum = (fv() % 3) as u16; P0[q].linkNo = (fv() % 16) as u8;    P0[q].cmdList = null_mut(); P0[q].cmdWait = 0; P0[q].life = (1 + fv() % 3) as u16; P0[q].kind = (fv() ^ (fv() << 9)) & !0x8800u32; P0[q].callback = if fv() & 1 != 0 { Some(cb_part) } else { None };    if P0[q].sizeCount > 100 { P0[q].sizeCount = 0; } if P0[q].rotateCount > 100 { P0[q].rotateCount = 0; } }  for q in 0..16usize { m::hsd_804D0908[q] = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() }; } for q in 0..8usize { m::hsd_804D08E8[q] = if fv() & 1 != 0 { &raw mut J[(fv() & 1) as usize] } else { null_mut() }; } }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = if fv() & 1 != 0 { (&raw mut PPA[0]) as *mut *mut HSD_Particle } else { null_mut() };
let a1 = (fv() % 16) as i32;
let a2 = (fv() % 4) as i32;
let a3 = fv() & !0x8804u32;
let a4 = fv() as _;
let a5 = null_mut();
let a6 = fv() as _;
let a7 = fv() as _;
let a8 = rw();
let a9 = rw();
let a10 = rw();
let a11 = rw();
let a12 = rw();
let a13 = rw();
let a14 = rw();
let a15 = rw();
let a16 = rw();
let a17 = picko_HSD_Generator();
let a18 = (fv() & 1) as i32;
ret = pi(m::psGenerateParticle0(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18));
 }
                    1 => { 
let a0 = (fv() % 16) as i32;
let a1 = (fv() % 4) as i32;
let a2 = (fv() & !0x8804u32) as i32;
let a3 = fv() as _;
let a4 = null_mut();
let a5 = fv() as _;
let a6 = fv() as _;
let a7 = null_mut();
let a8 = rw();
let a9 = rw();
let a10 = rw();
let a11 = rw();
let a12 = rw();
let a13 = rw();
let a14 = rw();
let a15 = rw();
let a16 = rw();
m::hsd_80398F0C(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16);
 }
                    2 => { 
let a0 = pick_HSD_Particle();
let a1 = rw();
m::hsd_80398F8C(a0, a1);
 }
                    3 => { 
let a0 = &raw mut P0[(fv() & 3) as usize];
let a1 = if fv() & 1 != 0 { &raw mut P0[(fv() & 3) as usize] } else { null_mut() };
ret = pi(m::hsd_8039930C(a0, a1));
 }
                    4 => { 
let a0 = &raw mut GEN[(fv() & 1) as usize];
m::hsd_8039D0A0(a0);
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
