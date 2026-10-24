//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbcardgame.c.
//! src/tu/lbcardgame.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbcardgame`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_lb_8001C600: u32 = 0;
const OP_lb_8001C658: u32 = 1;
const OP_getCurrentIcon: u32 = 2;
const OP_lb_8001C87C: u32 = 3;
const OP_lb_8001C8BC: u32 = 4;
const OP_updateCardStatus: u32 = 5;
const OP_lbCardGame_SetCardStatus: u32 = 6;
const OP_lb_8001CBBC: u32 = 7;
const OP_fn_8001CC30: u32 = 8;
const OP_lb_8001CC4C: u32 = 9;
const OP_dont_inline_helper: u32 = 10;
const OP_lb_8001CC84: u32 = 11;
const OP_lb_8001CDB4: u32 = 12;
const OP_lbCardGame_SaveChanges: u32 = 13;
const OP_lbCardGame_DecideGameMode: u32 = 14;
const OP_gobj1_Proc: u32 = 15;
const OP_gobj0_RenderFunc: u32 = 16;
const OP_lbCardGame_InitScene: u32 = 17;
const OP_lbCardGame_LoadArchive: u32 = 18;
const OP_lbCardGame_Reset: u32 = 19;
const OP_lbCardGame_Init: u32 = 20;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbcardgame as m;
use mtu_harness::*;
#[path = "mtu/lbcardgame_gen.rs"]
mod g;

static mut ICN: [*mut c_void; 4] = [null_mut(); 4]; static mut DMP: [*mut DynamicModelDesc; 2] = [null_mut(); 2]; static mut NTB: [u8; 7 * 0x1F2C + 16] = [0; 7 * 0x1F2C + 16];

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut GO: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
pub static mut SD: [SceneDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut DM: [DynamicModelDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut SC: [SceneCameraDesc; 2] = unsafe { core::mem::zeroed() };
pub static mut JO: [HSD_JObj; 2] = unsafe { core::mem::zeroed() };
pub fn pick_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { &raw mut GO[((r >> 8) % 4) as usize] } }
pub fn picko_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut GO[((r >> 8) % 4) as usize] } } }
pub fn pick_SceneDesc() -> *mut SceneDesc { let r = rnd(); unsafe { &raw mut SD[((r >> 8) % 2) as usize] } }
pub fn picko_SceneDesc() -> *mut SceneDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SD[((r >> 8) % 2) as usize] } } }
pub fn pick_DynamicModelDesc() -> *mut DynamicModelDesc { let r = rnd(); unsafe { &raw mut DM[((r >> 8) % 2) as usize] } }
pub fn picko_DynamicModelDesc() -> *mut DynamicModelDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DM[((r >> 8) % 2) as usize] } } }
pub fn pick_SceneCameraDesc() -> *mut SceneCameraDesc { let r = rnd(); unsafe { &raw mut SC[((r >> 8) % 2) as usize] } }
pub fn picko_SceneCameraDesc() -> *mut SceneCameraDesc { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SC[((r >> 8) % 2) as usize] } } }
pub fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { &raw mut JO[((r >> 8) % 2) as usize] } }
pub fn picko_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut JO[((r >> 8) % 2) as usize] } } }
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
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const SD, core::mem::size_of_val(&SD));
        preg(&raw const DM, core::mem::size_of_val(&DM));
        preg(&raw const SC, core::mem::size_of_val(&SC));
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    GO = core::mem::zeroed();
    SD = core::mem::zeroed();
    DM = core::mem::zeroed();
    SC = core::mem::zeroed();
    JO = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..4 { g::fill_HSD_GObj(&mut GO[i]); }
    for i in 0..2 { g::fill_SceneDesc(&mut SD[i]); }
    for i in 0..2 { g::fill_DynamicModelDesc(&mut DM[i]); }
    for i in 0..2 { g::fill_SceneCameraDesc(&mut SC[i]); }
    for i in 0..2 { g::fill_HSD_JObj(&mut JO[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..4 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..2 { g::hash_SceneDesc(&SD[i]); }
    for i in 0..2 { g::hash_DynamicModelDesc(&DM[i]); }
    for i in 0..2 { g::hash_SceneCameraDesc(&SC[i]); }
    for i in 0..2 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

#[test]
fn lbcardgame_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const ICN, core::mem::size_of_val(&ICN)); preg(&raw const DMP, core::mem::size_of_val(&DMP)); preg(&raw const NTB, core::mem::size_of_val(&NTB));
        let mut c = Check::new(include_str!("data/mtu_lbcardgame.txt"));
        let mut total = 0u32;
        for op in 0..21u32 {
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
                11 => 300u32,
                12 => 300u32,
                13 => 300u32,
                14 => 300u32,
                15 => 300u32,
                16 => 300u32,
                17 => 300u32,
                18 => 300u32,
                19 => 300u32,
                20 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ for q in 0..4usize { ICN[q] = if fv() & 7 != 0 { (&raw mut BUF as *mut u8).add(8 * q) as *mut c_void } else { null_mut() }; } m::state.icon_data = if op == OP_lbCardGame_LoadArchive && fv() & 1 != 0 { null_mut() } else { (&raw mut ICN) as *mut *mut c_void }; DMP[0] = &raw mut DM[0]; DMP[1] = &raw mut DM[1]; SD[0].models = (&raw mut DMP) as *mut *mut DynamicModelDesc; SD[0].cameras = &raw mut SC[0]; SD[1].models = (&raw mut DMP as *mut *mut DynamicModelDesc).add(1); SD[1].cameras = &raw mut SC[1]; m::state.scene_data = if fv() & 7 != 0 { &raw mut SD[(fv() & 1) as usize] } else { null_mut() }; m::state.jobj_translate_idx = 0; m::state.enable = ((fv() & 7 != 0) || op == OP_lb_8001C8BC || op == OP_lbCardGame_SaveChanges) as i32; m::state.x10 = fv() & 1; m::ext::OS_BusClock = if fv() & 1 != 0 { 162000000u32 } else { 4u32 + fv() % 100000u32 }; }
                let mut ret = 0u32;
                match op {
                    0 => { 

m::lb_8001C600();
 }
                    1 => { 

ret = pi(m::lb_8001C658());
 }
                    2 => { 

ret = pi(m::getCurrentIcon());
 }
                    3 => { 

ret = m::lb_8001C87C() as u32;
 }
                    4 => { 

ret = m::lb_8001C8BC() as u32;
 }
                    5 => { 

ret = m::updateCardStatus() as u32;
 }
                    6 => { 
let a0 = fv() as _;
m::lbCardGame_SetCardStatus(a0);
 }
                    7 => { 

ret = m::lb_8001CBBC() as u32;
 }
                    8 => { 
let a0 = fv() as _;
m::fn_8001CC30(a0);
 }
                    9 => { 

ret = m::lb_8001CC4C() as u32;
 }
                    10 => { 

ret = m::dont_inline_helper() as u32;
 }
                    11 => { 

m::lb_8001CC84();
 }
                    12 => { 

m::lb_8001CDB4();
 }
                    13 => { 

m::lbCardGame_SaveChanges();
 }
                    14 => { 

ret = m::lbCardGame_DecideGameMode() as u32;
 }
                    15 => { 
let a0 = pick_HSD_GObj();
m::gobj1_Proc(a0);
 }
                    16 => { 
let a0 = pick_HSD_GObj();
let a1 = fv() as _;
m::gobj0_RenderFunc(a0, a1);
 }
                    17 => { 

m::lbCardGame_InitScene();
 }
                    18 => { 
let a0 = fv() as _;
m::lbCardGame_LoadArchive(a0);
 }
                    19 => { 

m::lbCardGame_Reset();
 }
                    20 => { 

m::lbCardGame_Init();
 }
                    _ => unreachable!(),
                }
                hash_world();

                c.line(op, k, ret);
                total += 1;
            }
        }
        assert_eq!(c.done(), total);
    }
}
