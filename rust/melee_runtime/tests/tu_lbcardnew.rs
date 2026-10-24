//! Written by gen/mtu/mk_lb.py from the same description as gen/mtu/tu/lbcardnew.c.
//! src/tu/lbcardnew.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/mtu/run_tu.py lbcardnew`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, dead_code)]
const OP_convertSdkResult: u32 = 0;
const OP_lb_80019C38_noinline: u32 = 1;
const OP_getNewTask: u32 = 2;
const OP_resetTaskArray: u32 = 3;
const OP_executeNextTask: u32 = 4;
const OP_resetState: u32 = 5;
const OP_onCardComplete: u32 = 6;
const OP_convertHsdResult: u32 = 7;
const OP_fn_8001A0B0: u32 = 8;
const OP_taskMount: u32 = 9;
const OP_taskCheck: u32 = 10;
const OP_setTaskFilename: u32 = 11;
const OP_setupCardEntries: u32 = 12;
const OP_taskOpen: u32 = 13;
const OP_taskUnk3: u32 = 14;
const OP_taskFormat: u32 = 15;
const OP_taskDelete: u32 = 16;
const OP_taskRename: u32 = 17;
const OP_taskCreate: u32 = 18;
const OP_readCardFileSize: u32 = 19;
const OP_taskRead: u32 = 20;
const OP_taskWrite: u32 = 21;
const OP_taskSetStatus: u32 = 22;
const OP_taskReadHeader: u32 = 23;
const OP_taskListSnapshots: u32 = 24;
const OP_taskFindFile: u32 = 25;
const OP_lb_8001B6E0: u32 = 26;
const OP_lbCardNew_CompleteNextTask: u32 = 27;
const OP_lbCardNew_CompleteAllTasks: u32 = 28;
const OP_setupTask: u32 = 29;
const OP_lb_8001A4CC_dontinline: u32 = 30;
const OP_lb_8001B7E0: u32 = 31;
const OP_lb_8001B8C8: u32 = 32;
const OP_lbCardNew_DeleteSnap: u32 = 33;
const OP_lb_8001BA44: u32 = 34;
const OP_lb_8001BB48: u32 = 35;
const OP_lb_8001BC18: u32 = 36;
const OP_lb_8001BD34: u32 = 37;
const OP_lb_8001BE30: u32 = 38;
const OP_lb_8001BF04: u32 = 39;
const OP_lb_8001BFD8: u32 = 40;
const OP_lb_8001C0F4: u32 = 41;
const OP_lb_8001C2D8: u32 = 42;
const OP_lbCardNew_ProbeEx: u32 = 43;
const OP_lb_8001C4A8: u32 = 44;
const OP_lbCardNew_AllocWorkArea: u32 = 45;
const OP_lbCardNew_ForgetMemory: u32 = 46;
const OP_lbCardNew_Init: u32 = 47;
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lbcardnew as m;
use mtu_harness::*;
#[path = "mtu/lbcardnew_gen.rs"]
mod g;

use m::LbCardEntry; static mut WRK: [u8; 16] = [0; 16]; static mut NODES: [u8; 2100] = [0; 2100]; pub const CRC: [i32; 15] = [0, 0, 0, 0, -1, -2, -3, -4, -5, -6, -13, -14, -128, -7, -9]; pub const HSR: [i32; 16] = [0, 0, 1, -258, -259, -260, -261, -262, -263, -257, -264, -265, -266, -267, -5, -3];

pub static mut VA: [Vec3; 4] = unsafe { core::mem::zeroed() };
pub static mut U8A: [u8; 16] = [0; 16];
pub static mut IA: [i32; 4] = [0; 4];
pub static mut UA: [u32; 4] = [0; 4];
pub static mut FA: [f32; 4] = [0.0; 4];
pub static mut BUF: [u8; 256] = [0; 256];
pub static mut STRS: [[u8; 64]; 4] = [[0; 64]; 4];
pub static mut DID: [DVDDiskID; 1] = unsafe { core::mem::zeroed() };
pub static mut ENT: [LbCardEntry; 12] = unsafe { core::mem::zeroed() };
pub static mut SNP: [lbCardNew_SnapshotEntry; 127] = unsafe { core::mem::zeroed() };
pub fn pick_DVDDiskID() -> *mut DVDDiskID { let r = rnd(); unsafe { &raw mut DID[((r >> 8) % 1) as usize] } }
pub fn picko_DVDDiskID() -> *mut DVDDiskID { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut DID[((r >> 8) % 1) as usize] } } }
pub fn pick_LbCardEntry() -> *mut LbCardEntry { let r = rnd(); unsafe { &raw mut ENT[((r >> 8) % 12) as usize] } }
pub fn picko_LbCardEntry() -> *mut LbCardEntry { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut ENT[((r >> 8) % 12) as usize] } } }
pub fn pick_lbCardNew_SnapshotEntry() -> *mut lbCardNew_SnapshotEntry { let r = rnd(); unsafe { &raw mut SNP[((r >> 8) % 127) as usize] } }
pub fn picko_lbCardNew_SnapshotEntry() -> *mut lbCardNew_SnapshotEntry { let r = rnd(); unsafe { if r % 6 == 0 { core::ptr::null_mut() } else { &raw mut SNP[((r >> 8) % 127) as usize] } } }
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
        preg(&raw const DID, core::mem::size_of_val(&DID));
        preg(&raw const ENT, core::mem::size_of_val(&ENT));
        preg(&raw const SNP, core::mem::size_of_val(&SNP));
        preg(&raw const VA, core::mem::size_of_val(&VA)); preg(&raw const U8A, 16); preg(&raw const IA, 16); preg(&raw const UA, 16); preg(&raw const FA, 16);
        preg(&raw const BUF, 256); preg(&raw const STRS, core::mem::size_of_val(&STRS));
    }
}

pub unsafe fn setup_world() {
    DID = core::mem::zeroed();
    ENT = core::mem::zeroed();
    SNP = core::mem::zeroed();
    for (k, st) in [&b"a.dat"[..], &b"sym"[..], &b"LbAd.dat"[..], &b"x"[..]].iter().enumerate() { STRS[k] = [0; 64]; STRS[k][..st.len()].copy_from_slice(st); }
    for i in 0..16 { U8A[i] = rnd() as u8; }
    for i in 0..4 { IA[i] = fv() as i32; UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for i in 0..256 { BUF[i] = rnd() as u8; }
    BUF[255] = 0;
    for i in 0..1 { g::fill_DVDDiskID(&mut DID[i]); }
    for i in 0..12 { g::fill_LbCardEntry(&mut ENT[i]); }
    for i in 0..127 { g::fill_lbCardNew_SnapshotEntry(&mut SNP[i]); }
    g::fillg_all();
    g::fillo_all();
}

pub unsafe fn hash_world() {
    for i in 0..1 { g::hash_DVDDiskID(&DID[i]); }
    for i in 0..12 { g::hash_LbCardEntry(&ENT[i]); }
    for i in 0..127 { g::hash_lbCardNew_SnapshotEntry(&SNP[i]); }
    for i in 0..4 { hm(VA[i].x.to_bits()); hm(VA[i].y.to_bits()); hm(VA[i].z.to_bits()); hm(FA[i].to_bits()); hm(IA[i] as u32); hm(UA[i]); }
    for i in 0..16 { hm(U8A[i] as u32); }
    for i in 0..256 { hm(BUF[i] as u32); }
    g::hashg_all();
    g::hasho_all();
}

#[test]
fn lbcardnew_matches_c() {
    unsafe {
        g::install();
        preg_world();
        g::preg_own();
        g::freg_all();
        freg_world();
preg(&raw const WRK, 16); preg(&raw const NODES, 2100);
        let mut c = Check::new(include_str!("data/mtu_lbcardnew.txt"));
        let mut total = 0u32;
        for op in 0..48u32 {
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
                21 => 300u32,
                22 => 300u32,
                23 => 300u32,
                24 => 300u32,
                25 => 300u32,
                26 => 300u32,
                27 => 300u32,
                28 => 300u32,
                29 => 300u32,
                30 => 300u32,
                31 => 300u32,
                32 => 300u32,
                33 => 300u32,
                34 => 300u32,
                35 => 300u32,
                36 => 300u32,
                37 => 300u32,
                38 => 300u32,
                39 => 300u32,
                40 => 300u32,
                41 => 300u32,
                42 => 300u32,
                43 => 300u32,
                44 => 300u32,
                45 => 300u32,
                46 => 300u32,
                47 => 300u32,
                _ => unreachable!(),
            };
            for k in 0..n {
                case(op, k);
                setup_world();
{ m::state.work_area = if op == OP_lbCardNew_AllocWorkArea && fv() & 1 != 0 { null_mut() } else { (&raw mut WRK) as *mut c_void }; m::state.lib_area = if op == OP_lbCardNew_AllocWorkArea && fv() & 1 != 0 { null_mut() } else { (&raw mut NODES) as *mut c_void }; m::state.save_data = (&raw mut BUF) as *mut c_void; m::state.status = if fv() & 1 != 0 { &raw mut IA[0] } else { null_mut() }; m::state.comment = (&raw mut BUF as *mut u8).add(16) as *mut i8; m::state.banner = (&raw mut BUF as *mut u8).add(64) as *mut c_void; m::state.icons = (&raw mut BUF as *mut u8).add(128) as *mut c_void; m::state.snapshot_entries = (&raw mut SNP) as *mut lbCardNew_SnapshotEntry; m::state.free_blocks = if fv() & 1 != 0 { &raw mut IA[1] } else { null_mut() }; m::state.free_files = if fv() & 1 != 0 { &raw mut IA[2] } else { null_mut() }; m::state.sectorsize = if fv() & 1 != 0 { 0x2000 } else { fv() as i32 }; m::state.unused_bytes = (fv() % 0x400000) as i32; m::state.unused_files = (fv() % 4) as i32; m::state.mounted = (fv() & 1) as i32; m::state.tasks_remaining = (fv() % 4) as i32 - 1; m::state.saved_error = (fv() % 18) as i32; m::state.chan = (fv() % 3) as i32; for q in 0..11usize { m::state.task_array[q].r#type = fv() % 16; } m::state.task_array[(fv() % 11) as usize].r#type = 14; for q in 0..9usize { ENT[q].file_size = (fv() % 1000) as i32; ENT[q].data = (&raw mut BUF as *mut u8).add(16 * q) as *mut c_void; } ENT[9].file_size = -1; for q in 0..10usize { m::state.task_array[q].file_entries = (&raw mut ENT) as *mut c_void; } }
                let mut ret = 0u32;
                match op {
                    0 => { 
let a0 = if fv() & 1 != 0 { CRC[(fv() % 15) as usize] } else { fv() as i32 };
ret = m::convertSdkResult(a0) as u32;
 }
                    1 => { 

ret = pi(m::lb_80019C38_noinline());
 }
                    2 => { 

ret = pi(m::getNewTask());
 }
                    3 => { 

m::resetTaskArray();
 }
                    4 => { 
let a0 = fv() % 18;
ret = m::executeNextTask(a0) as u32;
 }
                    5 => { 
let a0 = fv() as _;
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = (&raw mut IA) as *mut i32;
let a3 = unsafe { core::mem::transmute::<usize, _>(pick_cb(4)) };
m::resetState(a0, a1, a2, a3);
 }
                    6 => { 
let a0 = fv() as _;
let a1 = if fv() & 1 != 0 { CRC[(fv() % 15) as usize] } else { fv() as i32 };
m::onCardComplete(a0, a1);
 }
                    7 => { 
let a0 = if fv() & 1 != 0 { HSR[(fv() % 16) as usize] } else { fv() as i32 };
ret = m::convertHsdResult(a0) as u32;
 }
                    8 => { 
let a0 = (fv() % 9) as i32;
let a1 = if fv() & 1 != 0 { HSR[(fv() % 16) as usize] } else { fv() as i32 };
m::fn_8001A0B0(a0, a1);
 }
                    9 => { 

ret = m::taskMount() as u32;
 }
                    10 => { 

ret = m::taskCheck() as u32;
 }
                    11 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a1 = (&raw mut ENT) as *mut c_void;
m::setTaskFilename(a0, a1);
 }
                    12 => { 
let a0 = &raw mut m::state.card_state;
let a1 = (&raw mut BUF) as *mut c_void;
let a2 = (&raw mut ENT) as *mut LbCardEntry;
m::setupCardEntries(a0, a1, a2);
 }
                    13 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a1 = (&raw mut ENT) as *mut LbCardEntry;
ret = m::taskOpen(a0, a1) as u32;
 }
                    14 => { 

ret = m::taskUnk3() as u32;
 }
                    15 => { 

ret = m::taskFormat() as u32;
 }
                    16 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = m::taskDelete(a0) as u32;
 }
                    17 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = m::taskRename(a0, a1) as u32;
 }
                    18 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = m::taskCreate(a0) as u32;
 }
                    19 => { 
let a0 = (&raw mut IA) as *mut i32;
ret = m::readCardFileSize(a0) as u32;
 }
                    20 => { 
let a0 = (&raw mut ENT) as *mut LbCardEntry;
ret = m::taskRead(a0) as u32;
 }
                    21 => { 
let a0 = (&raw mut ENT) as *mut LbCardEntry;
ret = m::taskWrite(a0) as u32;
 }
                    22 => { 

ret = m::taskSetStatus() as u32;
 }
                    23 => { 

ret = m::taskReadHeader() as u32;
 }
                    24 => { 

ret = m::taskListSnapshots() as u32;
 }
                    25 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = m::taskFindFile(a0) as u32;
 }
                    26 => { 
let a0 = (fv() % 9) as i32;
ret = m::lb_8001B6E0(a0) as u32;
 }
                    27 => { 

ret = m::lbCardNew_CompleteNextTask() as u32;
 }
                    28 => { 
let a0 = (fv() % 14) as i32;
ret = m::lbCardNew_CompleteAllTasks(a0) as u32;
 }
                    29 => { 
let a0 = fv() as _;
let a1 = fv() as _;
ret = pi(m::setupTask(a0, a1));
 }
                    30 => { 
let a0 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a1 = (&raw mut ENT) as *mut c_void;
m::lb_8001A4CC_dontinline(a0, a1);
 }
                    31 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut c_void;
let a3 = (&raw mut BUF) as *mut c_void;
let a4 = (&raw mut IA) as *mut i32;
ret = m::lb_8001B7E0(a0, a1, a2, a3, a4) as u32;
 }
                    32 => { 
let a0 = fv() as _;
ret = m::lb_8001B8C8(a0) as u32;
 }
                    33 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut BUF) as *mut c_void;
ret = m::lbCardNew_DeleteSnap(a0, a1, a2) as u32;
 }
                    34 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001BA44(a0, a1, a2) as u32;
 }
                    35 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut c_void;
let a3 = (&raw mut BUF) as *mut c_void;
let a4 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a5 = (&raw mut BUF) as *mut c_void;
let a6 = (&raw mut BUF) as *mut c_void;
let a7 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001BB48(a0, a1, a2, a3, a4, a5, a6, a7) as u32;
 }
                    36 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut *mut c_void;
let a3 = (&raw mut BUF) as *mut c_void;
let a4 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a5 = (&raw mut BUF) as *mut c_void;
let a6 = (&raw mut BUF) as *mut c_void;
let a7 = (&raw mut IA) as *mut i32;
ret = m::lb_8001BC18(a0, a1, a2, a3, a4, a5, a6, a7) as u32;
 }
                    37 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut c_void;
let a3 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001BD34(a0, a1, a2, a3) as u32;
 }
                    38 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut c_void;
let a3 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a4 = (&raw mut BUF) as *mut c_void;
let a5 = (&raw mut BUF) as *mut c_void;
let a6 = (&raw mut BUF) as *mut c_void;
let a7 = pick_cb(4) as *mut c_void;
ret = m::lb_8001BE30(a0, a1, a2, a3, a4, a5, a6, a7) as u32;
 }
                    39 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut ENT) as *mut c_void;
let a3 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a4 = (&raw mut BUF) as *mut c_void;
let a5 = (&raw mut BUF) as *mut c_void;
let a6 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001BF04(a0, a1, a2, a3, a4, a5, a6) as u32;
 }
                    40 => { 
let a0 = fv() as _;
let a1 = (&raw mut SNP) as *mut lbCardNew_SnapshotEntry;
let a2 = (&raw mut IA) as *mut i32;
let a3 = (&raw mut IA) as *mut i32;
ret = m::lb_8001BFD8(a0, a1, a2, a3) as u32;
 }
                    41 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a3 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a4 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001C0F4(a0, a1, a2, a3, a4) as u32;
 }
                    42 => { 
let a0 = fv() as _;
let a1 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a2 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
let a3 = (&raw mut STRS[(rnd() % 4) as usize]) as *mut i8;
ret = m::lb_8001C2D8(a0, a1, a2, a3) as u32;
 }
                    43 => { 
let a0 = fv() as _;
ret = m::lbCardNew_ProbeEx(a0) as u32;
 }
                    44 => { 
let a0 = (&raw mut ENT) as *mut c_void;
let a1 = (&raw mut BUF) as *mut c_void;
ret = m::lb_8001C4A8(a0, a1) as u32;
 }
                    45 => { 

m::lbCardNew_AllocWorkArea();
 }
                    46 => { 

m::lbCardNew_ForgetMemory();
 }
                    47 => { 

m::lbCardNew_Init();
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
