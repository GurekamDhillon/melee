//! src/tu/fighter.rs (the per-frame fighter functions) vs the C (gen/cref/tu/fighter.c: the whole TU,
//! shipping contraction, recording stubs for every callee, a random fighter world filled field by field;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py fighter`, stubs and fill code with
//! gen/mtu/stubs.py and gen/mtu/fill.py).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_variables, unused_parens, unused_unsafe, unused_mut, unused_imports, clippy::all)]
mod mtu_h;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::hooks as h;
use melee_fighter::tu::fighter as m;
use melee_fighter::types::*;
use mtu_h::*;

mod fighter_stubs;
mod fighter_state {
    use super::*;
    include!("fighter_state/gen.rs");
}
use fighter_state::*;

const NG: usize = 3;
const NJ: usize = 8;
static mut GO: *mut [HSD_GObj; NG] = null_mut();
static mut JO: *mut [HSD_JObj; NJ] = null_mut();
static mut FT: *mut [Fighter; NG] = null_mut();
static mut FDATA: *mut [ftData; NG] = null_mut();
static mut CDATA: *mut ftCommonData = null_mut();
static mut PADB: *mut [[u8; 2048]; 80] = null_mut();
static mut CLB: *mut [UnkCostumeStruct; 256] = null_mut();

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

static mut SMASH_TARGET: bool = false;
static mut MSL: [MotionState; 400] = unsafe { core::mem::zeroed() };
/// The C TU's own data (zero-filled arrays and scalars the stub header defines): the port reads them through
/// accessor hooks, which here answer from zero data.
unsafe fn install_data() {
    use m::ext::hook as k;
    k::pad_nml = Some(|_| [0.0; 6]);
    k::pad_button = Some(|_| 0);
    k::HSD_GObj_JObjKind = Some(|| 0);
    k::stage_grkind = Some(|| 0);
    k::gm_CStickSmashTargetTest = Some(|| &raw mut SMASH_TARGET);
    k::ftData_MotionStateList = Some(|| (&raw mut MSL) as *mut MotionState);
    k::DbLevel_at_least_DebugRom = Some(|| false);
}

fn bz<T>() -> *mut T { Box::leak(Box::new(unsafe { core::mem::zeroed::<T>() })) }

unsafe fn world() {
    *GO = core::mem::zeroed();
    *JO = core::mem::zeroed();
    *FT = core::mem::zeroed();
    *FDATA = core::mem::zeroed();
    for i in 0..NG { fill_Fighter(&mut (*FT)[i]); }
    for i in 0..NJ { fill_HSD_JObj(&mut (*JO)[i]); }
    for i in 0..NG { fill_HSD_GObj(&mut (*GO)[i]); }
    for i in 0..NG {
        (*GO)[i].user_data = &raw mut (*FT)[i] as *mut c_void;
        (*GO)[i].hsd_obj = &raw mut (*JO)[i] as *mut c_void;
        (*GO)[i].classifier = HSD_GOBJ_CLASS_FIGHTER as _;
        (*FT)[i].gobj = &raw mut (*GO)[i];
        (*FT)[i].ft_data = &raw mut (*FDATA)[i];
        (*FT)[i].grab_cb = Some(cbev1); (*FT)[i].grabbed_cb = Some(cbev3);
        (*FT)[i].kind = ((*FT)[i].kind as u32 % 27) as _; (*FT)[i].x200C &= 3; (*FT)[i].x2010 &= 3; (*FT)[i].player_id %= 4; (*FT)[i].x618_player_id %= 4; (*FT)[i].x61A_controller_index %= 4;
        { let mut pk = i * 24; (*FDATA)[i].x0 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].ext_attr = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x8 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].xC = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x10 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x14 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x18 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x1C = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x20 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x24 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x28 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x2C = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x30 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x34 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x38 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x3C = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x40 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x44 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x48_items = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x4C_sfx = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x50 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x54 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x58 = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; (*FDATA)[i].x5C = (*PADB)[{ pk += 1; pk - 1 }].as_mut_ptr() as *mut _; }
    }
    for i in 0..NJ {
        (*JO)[i].parent = if i != 0 { &raw mut (*JO)[(i - 1) / 2] } else { null_mut() };
        (*JO)[i].child = if 2 * i + 1 < NJ { &raw mut (*JO)[2 * i + 1] } else { null_mut() };
        (*JO)[i].next = if (i & 1) != 0 && i + 1 < NJ { &raw mut (*JO)[i + 1] } else { null_mut() };
    }
    *CDATA = core::mem::zeroed();
    for i in 0..27 { melee_fighter::tu::ftdata::CostumeListsForeachCharacter[i].costume_list = (*CLB).as_mut_ptr(); }
    h::P_FT_COMMON_DATA = CDATA;
}
unsafe fn hworld() {
    for i in 0..NG { hash_Fighter(&(*FT)[i]); }
    for i in 0..NJ { hash_HSD_JObj(&(*JO)[i]); }
    for i in 0..NG { hash_HSD_GObj(&(*GO)[i]); }
}

const OPS: &[&str] = &[
    "Fighter_8006C5F4", "Fighter_CallAcessoryCallbacks_8006C624", "Fighter_8006C80C", "Fighter_UnkProcessGrab_8006CA5C",
    "Fighter_8006CB94", "Fighter_8006D9AC", "Fighter_UnkCallCameraCallback_8006D9EC", "Fighter_8006DA4C", "Fighter_8006A1BC",
    "Fighter_8006A360", "Fighter_8006ABA0", "Fighter_UnkIncrementCounters_8006ABEC", "Fighter_Spaghetti_8006AD10",
    "Fighter_procUpdate", "Fighter_UnkApplyTransformation_8006C0F0", "Fighter_procMap", "Fighter_ProcessHit_8006D1EC",
    "Fighter_UnkProcessDeath_80068354", "Fighter_UnkUpdateCostumeJoint_800686E4", "Fighter_ResetInputData_80068854",
];

#[test]
fn fighter_matches_c() {
    unsafe {
        GO = bz(); JO = bz(); FT = bz(); FDATA = bz(); CDATA = bz(); PADB = bz(); CLB = bz();
        preg(GO, core::mem::size_of_val(&*GO));
        preg(JO, core::mem::size_of_val(&*JO));
        preg(FT, core::mem::size_of_val(&*FT));
        preg(FDATA, core::mem::size_of_val(&*FDATA));
        freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize);
        fighter_stubs::install();
        install_data();
    }
    let mut c = Check::new(include_str!("data/tu_fighter_ops.txt"));
    let sel: Option<usize> = std::env::var("TUOP").ok().and_then(|v| v.parse().ok());
    for op in 0..OPS.len() as u32 {
        for k in 0..400u32 {
            case(op, k);
            if std::env::var("TUV").is_ok() { eprintln!("op {op} k {k}"); }
            let res = std::panic::catch_unwind(|| unsafe {
                world();
                let g = &raw mut (*GO)[(rnd() as usize) % NG];
                match op {
                    0 => m::Fighter_8006C5F4(g),
                    1 => m::Fighter_CallAcessoryCallbacks_8006C624(g),
                    2 => m::Fighter_8006C80C(g),
                    3 => m::Fighter_UnkProcessGrab_8006CA5C(g),
                    4 => m::Fighter_8006CB94(g),
                    5 => m::Fighter_8006D9AC(g),
                    6 => m::Fighter_UnkCallCameraCallback_8006D9EC(g),
                    7 => m::Fighter_8006DA4C(g),
                    8 => m::Fighter_8006A1BC(g),
                    9 => m::Fighter_8006A360(g),
                    10 => m::Fighter_8006ABA0(g),
                    11 => m::Fighter_UnkIncrementCounters_8006ABEC(g),
                    12 => m::Fighter_Spaghetti_8006AD10(g),
                    13 => m::Fighter_procUpdate(g),
                    14 => m::Fighter_UnkApplyTransformation_8006C0F0(g),
                    15 => m::Fighter_procMap(g),
                    16 => m::Fighter_ProcessHit_8006D1EC(g),
                    17 => m::Fighter_UnkProcessDeath_80068354(g),
                    18 => m::Fighter_UnkUpdateCostumeJoint_800686E4(g),
                    19 => m::Fighter_ResetInputData_80068854(g),
                    _ => unreachable!(),
                }
                hworld();
            });
            if res.is_err() { c.assert_line(op, k); continue; }
            unsafe { c.line(op, k, 0); }
        }
    }
    assert_eq!(c.done(), OPS.len() as u32 * 400);
}
