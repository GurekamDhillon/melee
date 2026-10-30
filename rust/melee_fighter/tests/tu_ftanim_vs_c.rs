//! src/tu/ftanim.rs (fighter animation: per-part anim walk, request / rate, skeleton setup) vs the C
//! (gen/cref/tu/ftanim_ops.c: the whole TU, shipping contraction, recording stubs, three random fighters over an
//! 8-node joint tree). Regenerate: `bash rust/melee_fighter/gen/mtu/regen_ftanim.sh`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_variables, unused_parens, unused_unsafe, unused_mut, unused_imports, clippy::all)]
mod mtu_h;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftanim as m;
use melee_fighter::types::*;
use mtu_h::*;

mod ftanim_stubs;
mod ftanim_state {
    use super::*;
    include!("ftanim_state/gen.rs");
}
use ftanim_state::*;

const NG: usize = 3;
const NJ: usize = 8;
const NP: usize = 16;
static mut GO: *mut [HSD_GObj; NG] = null_mut();
static mut JO: *mut [HSD_JObj; NJ] = null_mut();
static mut FT: *mut [Fighter; NG] = null_mut();
static mut FDATA: *mut [ftData; NG] = null_mut();
static mut PARTS: *mut [[FighterBone; 64]; NG] = null_mut();
static mut PT: *mut FighterPartsTable = null_mut();
static mut PTP: [*mut FighterPartsTable; Ft_Kind_Max as usize] = [null_mut(); Ft_Kind_Max as usize];
static mut CD: *mut ftCommonData = null_mut();
static mut PADB: *mut [[u8; 2048]; 96] = null_mut();
static mut JNT: *mut [HSD_Joint; 2] = null_mut();
static mut FIGA: *mut [FigaTree; NG] = null_mut();
static mut FNODES: *mut [[i8; 40]; NG] = null_mut();
static mut FTRACKS: *mut [[FigaTrack; 64]; NG] = null_mut();
static mut X8S: *mut [ftData_x8; NG] = null_mut();
static mut X8ROWS: *mut [[*mut u16; 4]; NG] = null_mut();
static mut X8VALS: *mut [[[u16; 4]; 4]; NG] = null_mut();
static mut TOBJ: *mut HSD_TObj = null_mut();
static mut AOB: *mut HSD_AObj = null_mut();
static mut X1CP: *mut [[*mut ftData_x1C; 5]; NG] = null_mut();
static mut X1CS: *mut [[ftData_x1C; 5]; NG] = null_mut();
static mut X1CL: *mut [[[u8; 8]; 5]; NG] = null_mut();
static mut ANIMB: *mut [*mut HSD_AnimJoint; 512] = null_mut();
static mut CB0: [Option<unsafe extern "C" fn(*mut HSD_GObj)>; Ft_Kind_Max as usize] = [None; Ft_Kind_Max as usize];
static mut CB4: [Option<unsafe extern "C" fn(*mut HSD_GObj, i32, f32)>; Ft_Kind_Max as usize] = [None; Ft_Kind_Max as usize];

fn pick_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*GO)[((r >> 8) as usize) % NG] } } }
fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*JO)[((r >> 8) as usize) % NJ] } } }
fn pick_Fighter() -> *mut Fighter { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*FT)[((r >> 8) as usize) % NG] } } }
fn pick_Item() -> *mut Item { null_mut() }
fn pick_HSD_DObj() -> *mut HSD_DObj { null_mut() }
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, hh: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(hh)); }
pub unsafe extern "C" fn cb4(g: *mut HSD_GObj, i: i32, f: f32) { t(7006); t(pi(g)); t(i as u32); t(f.to_bits()); }
fn pick_fn(kind: i32) -> usize {
    let r = rnd();
    if r & 3 == 0 { return 0; }
    match kind {
        1 => if r & 4 != 0 { cbev1 as usize } else { cbev2 as usize },
        2 => if r & 4 != 0 { cbp1 as usize } else { cbp2 as usize },
        3 => cbev3 as usize,
        _ => 0,
    }
}

fn bz<T>() -> *mut T { Box::leak(Box::new(unsafe { core::mem::zeroed::<T>() })) }

unsafe fn install_hooks() {
    use m::ext::hook as k;
    k::ftData_UnkCallbackPairs0_x0 = Some(|kind| CB0[kind as usize]);
    k::ftData_UnkCallbackPairs0_x4 = Some(|kind| CB4[kind as usize]);
    k::costume_matanim = Some(|_, _| null_mut());
    k::HSD_ForeachAnim_SetLoop = Some(|j, mask| { t(2100); t(pi(j)); t(6); t(mask); t(AOBJ_ARG_AU as u32); t(2102); t(AOBJ_LOOP as u32); });
    k::HSD_ForeachAnim_SetRate = Some(|j, mask, rate| { t(2100); t(pi(j)); t(6); t(mask); t(AOBJ_ARG_AF as u32); t(2101); t(rate.to_bits()); });
}

unsafe fn world() {
    *GO = core::mem::zeroed();
    *JO = core::mem::zeroed();
    *FT = core::mem::zeroed();
    *FDATA = core::mem::zeroed();
    *CD = core::mem::zeroed();
    *JNT = core::mem::zeroed();
    (*JNT)[0].child = &raw mut (*JNT)[1];
    for i in 0..NG { fill_Fighter(&mut (*FT)[i]); }
    for i in 0..NG { (*FT)[i].anon_594.anon_0_0.set_x594_bits(fv() as _); (*FT)[i].anon_594.anon_0_0.set_x597_bits(fv() as _); }
    for i in 0..NJ { fill_HSD_JObj(&mut (*JO)[i]); }
    for i in 0..NG { fill_HSD_GObj(&mut (*GO)[i]); }
    for i in 0..NG {
        (*GO)[i].user_data = &raw mut (*FT)[i] as *mut c_void;
        (*GO)[i].hsd_obj = &raw mut (*JO)[i] as *mut c_void;
        (*GO)[i].classifier = HSD_GOBJ_CLASS_FIGHTER as _;
        (*FT)[i].gobj = &raw mut (*GO)[i];
        (*FT)[i].ft_data = &raw mut (*FDATA)[i];
        (*FT)[i].grab_cb = Some(cbev1); (*FT)[i].grabbed_cb = Some(cbev3);
        (*FT)[i].kind = ((*FT)[i].kind as u32 % 27) as _; (*FT)[i].player_id %= 4; (*FT)[i].x618_player_id %= 4;
        for b in 0..64 { if b < NP { fill_FighterBone(&mut (*PARTS)[i][b]); } else { (*PARTS)[i][b] = core::mem::zeroed(); (*PARTS)[i][b].anon_8.anon_0.set_flags_b1(1); } (*PARTS)[i][b].joint = &raw mut (*JO)[(i * 5 + b * 3) % NJ]; }
        { let b = &mut (*PARTS)[i][0].anon_8.anon_0; b.set_flags_b1(1); b.set_flags_b0(0); b.set_flags_b5(0); }
        (*FT)[i].parts = (*PARTS)[i].as_mut_ptr();
        (*FT)[i].x108_costume_joint = &raw mut (*JNT)[0];
        {
            let n_ = (rnd() % 5) as usize;
            for q in 0..n_ { (*FNODES)[i][q] = (1 + rnd() % 4) as i8; }
            for q in n_..40 { (*FNODES)[i][q] = -1; }
            (*FTRACKS)[i] = core::mem::zeroed(); (*FIGA)[i] = core::mem::zeroed();
            (*FIGA)[i].nodes = (*FNODES)[i].as_mut_ptr(); (*FIGA)[i].tracks = (*FTRACKS)[i].as_mut_ptr(); (*FIGA)[i].frames = rg();
            (*FT)[i].x590 = &raw mut (*FIGA)[i];
        }
    }
    for i in 0..NG {
        // the ftData pointer words: one pad block each (the C driver's order)
        { let mut pk = i * 24; macro_rules! nx { () => {{ pk += 1; (*PADB)[pk - 1].as_mut_ptr() as *mut _ }}; }
          let d = &mut (*FDATA)[i];
          d.x0 = nx!(); d.ext_attr = nx!(); d.x8 = nx!(); d.xC = nx!(); d.x10 = nx!(); d.x14 = nx!(); d.x18 = nx!(); d.x1C = nx!();
          d.x20 = nx!(); d.x24 = nx!(); d.x28 = nx!(); d.x2C = nx!(); d.x30 = nx!(); d.x34 = nx!(); d.x38 = nx!(); d.x3C = nx!();
          d.x40 = nx!(); d.x44 = nx!(); d.x48_items = nx!(); d.x4C_sfx = nx!(); d.x50 = nx!(); d.x54 = nx!(); d.x58 = nx!(); d.x5C = nx!(); }
    }
    for i in 0..NG {
        *(&raw mut (*X8S)[i]) = core::mem::zeroed();
        (*X8S)[i].x8.x8 = rnd() % 4;
        for r in 0..4 {
            for c in 0..4 { (*X8VALS)[i][r][c] = (rnd() % 3) as u16; }
            (*X8ROWS)[i][r] = if r == 0 || (rnd() % 4) != 0 { (*X8VALS)[i][r].as_mut_ptr() } else { null_mut() };
        }
        (*X8S)[i].x8.xC = (*X8ROWS)[i].as_mut_ptr();
        (*FDATA)[i].x8 = &raw mut (*X8S)[i];
        for r in 0..5 {
            (*X1CS)[i][r] = core::mem::zeroed();
            (*X1CS)[i][r].x2 = (rnd() % 6) as _;
            for e in 0..8 { (*X1CL)[i][r][e] = (rnd() % 16) as u8; }
            (*X1CS)[i][r].x4 = (*X1CL)[i][r].as_mut_ptr();
            (*X1CS)[i][r].x8 = (*ANIMB).as_mut_ptr().add(256);
            (*X1CP)[i][r] = &raw mut (*X1CS)[i][r];
        }
        (*FDATA)[i].x1C = (*X1CP)[i].as_mut_ptr() as *mut _;
    }
    for i in 0..512 { (*ANIMB)[i] = (*PADB)[10 + i % 8].as_mut_ptr() as *mut HSD_AnimJoint; }
    *TOBJ = core::mem::zeroed(); (*TOBJ).aobj = (*PADB)[5].as_mut_ptr() as *mut HSD_AObj;
    for i in 0..NG {
        let n = (*FT)[i].tobj_list.costume_tobjs.len();
        (*FT)[i].tobj_list.n_costume_tobjs %= (n + 1) as u32;
        for q in 0..n { (*FT)[i].tobj_list.costume_tobjs[q] = TOBJ; }
    }
    for i in 0..NJ {
        (*JO)[i].parent = if i != 0 { &raw mut (*JO)[(i - 1) / 2] } else { null_mut() };
        (*JO)[i].child = if 2 * i + 1 < NJ { &raw mut (*JO)[2 * i + 1] } else { null_mut() };
        (*JO)[i].next = if (i & 1) != 0 && i + 1 < NJ { &raw mut (*JO)[i + 1] } else { null_mut() };
    }
    *AOB = core::mem::zeroed(); (*AOB).curr_frame = rg();
    for i in 0..NJ { (*JO)[i].aobj = AOB; }
    *PT = core::mem::zeroed();
    (*PT).parts_num = (6 + rnd() % 11) as _;
    for i in 0..Ft_Kind_Max as usize { PTP[i] = PT; }
    h::FT_PARTS_TABLE = PTP.as_mut_ptr();
    for i in 0..Ft_Kind_Max as usize {
        melee_fighter::tu::ftdata::CostumeListsForeachCharacter[i].costume_list = null_mut();
        CB0[i] = if rnd() & 1 != 0 { Some(cbev1) } else { None };
        CB4[i] = if rnd() & 1 != 0 { Some(cb4) } else { None };
    }
    h::P_FT_COMMON_DATA = CD;
}
unsafe fn hworld() {
    for i in 0..NG { hash_Fighter(&(*FT)[i]); }
    for i in 0..NJ { hash_HSD_JObj(&(*JO)[i]); }
    for i in 0..NG { hash_HSD_GObj(&(*GO)[i]); }
    for i in 0..NG { for b in 0..NP { hash_FighterBone(&(*PARTS)[i][b]); } }
}

#[test]
fn ftanim_matches_c() {
    unsafe {
        GO = bz(); JO = bz(); FT = bz(); FDATA = bz(); PARTS = bz(); PT = bz(); CD = bz(); PADB = bz(); JNT = bz(); FIGA = bz();
        FNODES = bz(); FTRACKS = bz(); X8S = bz(); X8ROWS = bz(); X8VALS = bz(); TOBJ = bz(); AOB = bz(); X1CP = bz(); X1CS = bz(); X1CL = bz(); ANIMB = bz();
        preg(GO, core::mem::size_of_val(&*GO));
        preg(JO, core::mem::size_of_val(&*JO));
        preg(FT, core::mem::size_of_val(&*FT));
        preg(FDATA, core::mem::size_of_val(&*FDATA));
        preg(PADB, core::mem::size_of_val(&*PADB));
        freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize); freg(cb4 as usize);
        ftanim_stubs::install();
        install_hooks();
    }
    let mut c = Check::new(include_str!("data/tu_ftanim_ops.txt"));
    const NOPS: u32 = 12;
    for op in 0..NOPS {
        for k in 0..500u32 {
            case(op, k);
            if std::env::var("TUV").is_ok() { eprintln!("op {op} k {k}"); }
            let mut ret = 0u32;
            let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| unsafe {
                world();
                let gi = (rnd() as usize) % NG;
                let g = &raw mut (*GO)[gi];
                let fp = &raw mut (*FT)[gi];
                match op {
                    0 => { let a3 = pick_HSD_JObj(); let a2 = pick_HSD_JObj(); let a1 = pick_HSD_JObj(); m::ftAnim_8006E054(&mut *fp, a1, a2, a3); } // C argument order: last first
                    1 => m::ftAnim_8006E7B8(&mut *fp, (rnd() % NP as u32) as _),
                    2 => m::ftAnim_8006E9B4(g),
                    3 => { let a = rw(); let r = rg(); let bl = rw(); m::ftAnim_8006EBE8(g, a, r, bl); }
                    4 => m::ftAnim_8006FE48(g),
                    5 => m::ftAnim_80070308(g),
                    6 => m::ftAnim_80070654(g),
                    7 => m::ftAnim_80070758(pick_HSD_JObj()),
                    8 => m::ftAnim_800707B0(g),
                    9 => m::ftAnim_80070904(&mut *fp, (rnd() % NP as u32) as _, (*PADB)[0].as_mut_ptr() as *mut HSD_AnimJoint),
                    10 => { let pt = (rnd() % NP as u32) as _; let ti = (rnd() as usize) % NG; m::ftAnim_80070A10(&mut *fp, pt, &raw mut (*FIGA)[ti]); }
                    11 => m::ftAnim_80070E74(g),
                    _ => unreachable!(),
                }
                hworld();
            }));
            if res.is_err() { c.assert_line(op, k); continue; }
            unsafe { c.line(op, k, ret); }
        }
    }
    assert_eq!(c.done(), NOPS * 500);
}
