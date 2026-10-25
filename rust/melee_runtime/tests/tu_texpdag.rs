//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/texpdag.c.
//! src/tu/texpdag.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py texpdag`).
//! C functions of the TU this test covers (the private ones through their public callers): CalcDistance, HSD_TExpMakeDag, HSD_TExpSchedule, HSD_TExpSimplify, HSD_TExpSimplify2, LGUARD, SimplifyByMerge, SimplifySrc, SimplifyThis, assign_reg, make_dependancy_mtx, make_full_dependancy_mtx, order_dag.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_assign_reg: u32 = 0;
const OP_order_dag: u32 = 1;
const OP_CalcDistance: u32 = 2;
const OP_SimplifySrc: u32 = 3;
const OP_SimplifyThis: u32 = 4;
const OP_SimplifyByMerge: u32 = 5;
const OP_HSD_TExpSimplify: u32 = 6;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::texpdag as m;
use mtu_harness::*;
#[path = "mtu/texpdag_gen.rs"]
mod g;


static mut TEP: [*mut HSD_TExp; 6] = [null_mut(); 6];
static mut DIST: [i32; 6] = [0; 6];
static mut ORD: [i32; 8] = [0; 8];
static mut MINO: [i32; 8] = [0; 8];
static mut MINV: i32 = 0;
static mut DEP: [u32; 8] = [0; 8];
static mut FDEP: [u32; 8] = [0; 8];
fn pick_type(q: usize) -> u8 { if q < 5 && fv() & 1 != 0 { return 1; } let t = (fv() % 6) as u8; if q == 5 && t == 1 { 0 } else { t } }


pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut PPA: [*mut c_void; 4] = [core::ptr::null_mut(); 4];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut TE: [HSD_TExp; 6] = unsafe { core::mem::zeroed() };
pub static mut DG: [HSD_TExpDag; 6] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_TExp() -> *mut HSD_TExp { let r = rnd(); unsafe { &raw mut TE[((r >> 8) % 6) as usize] } }
pub fn picko_HSD_TExp() -> *mut HSD_TExp { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut TE[((r >> 8) % 6) as usize] } } }
pub fn pick_HSD_TExpDag() -> *mut HSD_TExpDag { let r = rnd(); unsafe { &raw mut DG[((r >> 8) % 6) as usize] } }
pub fn picko_HSD_TExpDag() -> *mut HSD_TExpDag { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DG[((r >> 8) % 6) as usize] } } }
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
        preg(&raw const TE, core::mem::size_of_val(&TE));
        preg(&raw const DG, core::mem::size_of_val(&DG));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS)); preg(&raw const PPA, core::mem::size_of_val(&PPA));
    }
}

pub unsafe fn setup_world() {
    TE = core::mem::zeroed();
    DG = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { PPA[i] = core::ptr::null_mut(); }
    for i in 0..6 { g::fill_HSD_TExp(&mut TE[i]); }
    for i in 0..6 { g::fill_HSD_TExpDag(&mut DG[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..6 { g::hash_HSD_TExp(&TE[i]); }
    for i in 0..6 { g::hash_HSD_TExpDag(&DG[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    for i in 0..4 { hm(pi(PPA[i])); }
    g::hashg_all();
    g::hasho_all();
}

// harness = false (Cargo.toml): a case that ends in a panic is cut off inside the hook (see mtu_harness::run_case),
// which a libtest main thread cannot survive at exit.
fn main() { texpdag_matches_c(); println!("tu_texpdag: ok"); }

fn texpdag_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const TEP, 48); preg(&raw const DIST, 24); preg(&raw const ORD, 32); preg(&raw const MINO, 32); preg(&raw const MINV, 4); preg(&raw const DEP, 32); preg(&raw const FDEP, 32); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
        install_abort_hook();
        let mut c = Check::new(include_str!("data/mtu_texpdag.txt"));
        let mut total = 0u32;
        for op in 0..7u32 {
            let n = match op {
                0 => 300u32,
                1 => 300u32,
                2 => 300u32,
                3 => 300u32,
                4 => 300u32,
                5 => 300u32,
                6 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{  for q in 0..6usize { TEP[q] = &raw mut TE[q]; DIST[q] = (fv() % 4) as i32; ORD[q] = (fv() % 6) as i32; MINO[q] = 0; DEP[q] = fv() & 0x3F; FDEP[q] = fv() & 0x3F; DG[q].tev = (&raw mut TE[q]) as *mut _HSD_TETev; DG[q].idx = q as u8; DG[q].nb_dep = (fv() % 3) as u8; DG[q].nb_ref = (fv() % 3) as u8; DG[q].dist = 0;    for i in 0..8usize { DG[q].depend[i] = &raw mut DG[(fv() % 6) as usize]; }    TE[q].tev.c_ref = (fv() % 3) as i32; TE[q].tev.a_ref = (fv() % 3) as i32; TE[q].tev.c_dst = (fv() % 4) as u8; TE[q].tev.a_dst = (fv() % 4) as u8; TE[q].tev.tex = null_mut(); TE[q].tev.chan = if fv() & 1 != 0 { 0xFF } else { (fv() % 3) as u8 };    TE[q].tev.tex_swap = if fv() & 1 != 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.ras_swap = if fv() & 1 != 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.kcsel = if fv() & 1 != 0 { 0xFF } else { (fv() % 4) as u8 }; TE[q].tev.kasel = if fv() & 1 != 0 { 0xFF } else { (fv() % 4) as u8 };    TE[q].tev.c_op = if fv() % 5 == 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.a_op = if fv() % 5 == 0 { 0xFF } else { (fv() % 2) as u8 }; TE[q].tev.c_bias = (fv() % 3) as u8; TE[q].tev.c_scale = (fv() % 3) as u8; TE[q].tev.a_bias = (fv() % 3) as u8; TE[q].tev.a_scale = (fv() % 3) as u8;    TE[q].tev.c_clamp = (fv() & 1) as u8; TE[q].tev.a_clamp = (fv() & 1) as u8;    for i in 0..4usize {      TE[q].tev.c_in[i].r#type = pick_type(q); TE[q].tev.c_in[i].sel = (fv() % 3) as u8; TE[q].tev.c_in[i].exp = { let k = if q < 5 { q + 1 + (fv() % (5 - q) as u32) as usize } else { q }; &raw mut TE[k] };      TE[q].tev.a_in[i].r#type = pick_type(q); TE[q].tev.a_in[i].sel = (fv() % 3) as u8; TE[q].tev.a_in[i].exp = { let k = if q < 5 { q + 1 + (fv() % (5 - q) as u32) as usize } else { q }; &raw mut TE[k] }; } }  MINV = 1000; }
                let mut ret = 0u32;
                let aborted = run_case(|| unsafe { match op {
                    0 => { 
let a0 = (1 + fv() % 6) as i32;
let a1 = (&raw mut DEP) as *mut u32;
let a2 = (&raw mut DG) as *mut HSD_TExpDag;
let a3 = (&raw mut ORD) as *mut i32;
ret = m::assign_reg(&mut m::Ext, a0, a1, a2, a3) as u32;
 }
                    1 => { 
let a0 = (1 + fv() % 5) as i32;
let a1 = (&raw mut DEP) as *mut u32;
let a2 = (&raw mut FDEP) as *mut u32;
let a3 = (&raw mut DG) as *mut HSD_TExpDag;
let a4 = 0;
let a5 = (fv() % 5) as i32;
let a6 = fv() as _;
let a7 = fv() as _;
let a8 = (&raw mut ORD) as *mut i32;
let a9 = &raw mut MINV;
let a10 = (&raw mut MINO) as *mut i32;
m::order_dag(&mut m::Ext, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10);
 }
                    2 => { 
let a0 = (&raw mut TEP) as *mut *mut HSD_TExp;
let a1 = (&raw mut DIST) as *mut i32;
let a2 = &raw mut TE[(fv() % 6) as usize];
let a3 = (1 + fv() % 6) as i32;
let a4 = (fv() % 3) as i32;
m::CalcDistance(a0, a1, a2, a3, a4);
 }
                    3 => { 
let a0 = &raw mut TE[(fv() % 5) as usize];
ret = m::SimplifySrc(&mut m::Ext, a0) as u32;
 }
                    4 => { 
let a0 = &raw mut TE[(fv() % 5) as usize];
ret = m::SimplifyThis(&mut m::Ext, a0) as u32;
 }
                    5 => { 
let a0 = &raw mut TE[(fv() % 5) as usize];
ret = m::SimplifyByMerge(&mut m::Ext, a0) as u32;
 }
                    6 => { 
let a0 = &raw mut TE[(fv() % 5) as usize];
ret = m::HSD_TExpSimplify(&mut m::Ext, a0) as u32;
 }
                    _ => unreachable!(),
                } });
                if aborted { c.abort(op, k); total += 1; continue; }
                hash_world();
{ for i in 0..6usize { hm(DIST[i] as u32); } for i in 0..8usize { hm(ORD[i] as u32); hm(MINO[i] as u32); } hm(MINV as u32); }
                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
