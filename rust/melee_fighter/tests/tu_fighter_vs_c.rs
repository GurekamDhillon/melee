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
static mut PADB: *mut [[u8; 2048]; 96] = null_mut();
static mut PARTS: *mut [[FighterBone; 16]; NG] = null_mut();
static mut PAI: *mut plAllocInfo = null_mut();
static mut PLCO: [*mut c_void; 23] = [null_mut(); 23];
static mut SRC64: *mut Fighter_804D64FC_t = null_mut();
static mut SEEDV: u32 = 0;
static mut IGO: *mut [HSD_GObj; 2] = null_mut();
static mut ITM: *mut [Item; 2] = null_mut();
static mut PLINKV: [*mut HSD_GObj; 16] = [null_mut(); 16];
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
    k::HSD_RandSeedPtr = Some(|| &raw mut SEEDV);
    k::lbArchive_LoadSymbols_PlCo = Some(|| { t(1900); t(sh_str("PlCo.dat")); (&raw mut PLCO) as *mut *mut c_void });
    k::ftParts_SetupParts = Some(|g| { t(1901); t(pi(g)); (*((*g).user_data as *mut Fighter)).parts = (*PARTS)[0].as_mut_ptr(); });
    h::PLINK_HEAD = Some(|link| PLINKV[link as usize]);
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
    for i in 0..NG {
        for b in 0..16 { (*PARTS)[i][b] = core::mem::zeroed(); (*PARTS)[i][b].joint = &raw mut (*JO)[(i * 5 + b * 3) % NJ]; }
        (*FT)[i].parts = (*PARTS)[i].as_mut_ptr();
    }
    for i in 0..Ft_Kind_Max as usize { melee_fighter::tu::ftdata::gFtDataList[i] = &raw mut (*FDATA)[i % NG]; }
    {   // the C TU's own per-kind tables are zero data
        use melee_fighter::tu::ftdata as fd;
        for i in 0..Ft_Kind_Max as usize {
            fd::CostumeListsForeachCharacter[i].numCostumes = 0;
            fd::ftData_UnkBytePerCharacter[i] = 0;
            fd::ftData_Table_Unk1[i] = None;
            fd::ftData_CharacterStateTables[i] = null_mut();
            fd::ftData_OnLoad[i] = None; fd::ftData_OnDeath[i] = None; fd::ftData_OnAbsorb[i] = None;
            fd::ftData_OnUserDataRemove[i] = None; fd::ftData_UnkMotionStates3[i] = None;
        }
    }
    for i in 0..4 { let c = &mut (*CDATA).x6DC_colorsByPlayer[i]; c.r = fv() as u8; c.g = fv() as u8; c.b = fv() as u8; c.a = fv() as u8; }
    (*CDATA).x7E4_scaleZ = rg();
    fill_plAllocInfo(&mut *PAI);
    (*PAI).internal_id = ((*PAI).internal_id as u32 % 27) as _; (*PAI).slot %= 4;
    for i in 0..23 { PLCO[i] = (*PADB)[72 + i].as_mut_ptr() as *mut c_void; }
    for r in 72..96 { for k in 0..256usize { *((*PADB)[r].as_mut_ptr() as *mut *mut c_void).add(k) = (*PADB)[(k * 7 + r) % 72].as_mut_ptr() as *mut c_void; } }
    for k in 0..512usize { *((*PADB)[79].as_mut_ptr() as *mut u32).add(k) = 0x3F80_0000u32.wrapping_add(k as u32 * 0x9E37); } // x20: f32 per kind (plain words)
    *SRC64 = core::mem::zeroed();
    (*SRC64).x4 = (*PADB)[72].as_mut_ptr() as _; (*SRC64).x8 = (*PADB)[73].as_mut_ptr() as _; (*SRC64).xC = (*PADB)[74].as_mut_ptr() as _; (*SRC64).x10 = (*PADB)[75].as_mut_ptr() as _;
    (*SRC64).x14 = (*PADB)[76].as_mut_ptr() as _; (*SRC64).x18 = (*PADB)[77].as_mut_ptr() as _; (*SRC64).x1C = (*PADB)[78].as_mut_ptr() as _; (*SRC64).x20 = (*PADB)[79].as_mut_ptr() as _;
    PLCO[22] = SRC64 as *mut c_void;
    m::Fighter_804D64FC = null_mut(); h::FT_PARTS_TABLE = null_mut(); h::FIGHTER_804D6540 = null_mut();
    m::Fighter_804D6550 = null_mut(); m::Fighter_804D654C = null_mut(); m::Fighter_804D6548 = null_mut();
    SEEDV = fv();
    for i in 0..2 {
        (*IGO)[i] = core::mem::zeroed(); (*ITM)[i] = core::mem::zeroed();
        (*IGO)[i].user_data = &raw mut (*ITM)[i] as *mut c_void;
        (*ITM)[i].kind = fv() as _; (*ITM)[i].msid = fv() as _;
        (*ITM)[i].pos.x = rw(); (*ITM)[i].pos.y = rw(); (*ITM)[i].pos.z = rw();
        (*ITM)[i].x40_vel.x = rw(); (*ITM)[i].x40_vel.y = rw(); (*ITM)[i].x40_vel.z = rw();
    }
    (*IGO)[0].next = &raw mut (*IGO)[1]; (*IGO)[1].next = null_mut();
    for i in 0..16 { PLINKV[i] = null_mut(); }
    PLINKV[HSD_GOBJ_PLINK_ITEM as usize] = if fv() & 1 != 0 { &raw mut (*IGO)[0] } else { null_mut() };
}
unsafe fn hglob() {
    hm(melee_fighter::fighter::SPAWN_NUM_COUNTER as u32);
    hm(pi(h::P_FT_COMMON_DATA)); hm(pi(m::Fighter_804D6550)); hm(pi(m::Fighter_804D654C)); hm(pi(m::Fighter_804D6548));
    hm(pi(m::Fighter_804D653C)); hm(pi(m::Fighter_804D6538)); hm(pi(m::Fighter_804D6534)); hm(pi(h::FIGHTER_804D6530));
    hm(pi(h::FIGHTER_GRAB_MASH_SHAKE)); hm(pi(m::Fighter_SmashChargeShakeTable)); hm(pi(m::Fighter_804D6524)); hm(pi(h::FIGHTER_804D6520));
    hm(pi(m::Fighter_804D651C)); hm(pi(m::Fighter_804D6518)); hm(pi(m::Fighter_804D6514)); hm(pi(m::Fighter_804D6510));
    hm(pi(m::Fighter_804D650C)); hm(pi(m::Fighter_804D6508)); hm(pi(m::Fighter_804D6504)); hm(pi(m::gCrowdConfig));
    hm(pi(h::FT_PARTS_TABLE)); hm(pi(h::FIGHTER_804D6540)); hm(pi(m::Fighter_804D64FC));
    if !h::FT_PARTS_TABLE.is_null() { for i in 0..Ft_Kind_Max as usize { hm(pi(*(h::FT_PARTS_TABLE as *const *const c_void).add(i))); } }
    if !h::FIGHTER_804D6540.is_null() { for i in 0..Ft_Kind_Max as usize { hm(pi(*(h::FIGHTER_804D6540 as *const *const c_void).add(i))); } }
    if !m::Fighter_804D64FC.is_null() {
        let c = m::Fighter_804D64FC;
        let f: [*const *mut *mut c_void; 7] = [&raw const (*c).x4 as _, &raw const (*c).x8 as _, &raw const (*c).xC as _, &raw const (*c).x10 as _, &raw const (*c).x14 as _, &raw const (*c).x18 as _, &raw const (*c).x1C as _];
        for j in 0..7 { let t = *f[j]; hm(pi(t)); if !t.is_null() { for i in 0..Ft_Kind_Max as usize { hm(pi(*t.add(i))); } } }
        let w = (*c).x20 as *const u32; hm(pi(w)); if !w.is_null() { for i in 0..Ft_Kind_Max as usize { hm(*w.add(i)); } }
    }
    for i in 0..16 { hm(m::RB_NeutralHashWord(i)); }
}
unsafe fn hworld() {
    hglob();
    if HDBG { let q = FT as *const u32; for w in 0..core::mem::size_of_val(&*FT) / 4 { let v = *q.add(w); if v != 0 { println!("W {:05X} {:08X}", w * 4, v); } } }
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
    "Fighter_UnkInitReset_80067C98", "Fighter_UnkUpdateVecFromBones_8006876C", "Fighter_UnkInitLoad_80068914", "Fighter_Create_Inline2",
    "Fighter_Create", "Fighter_FirstInitialize_80067A84", "Fighter_800679B0", "Fighter_LoadCommonData", "ftCommonData_ExtendKindTable",
    "ftCommonData_ExtendCpuTables", "ftSlippi_IsOffscreen", "ftReplay_TraceFighter", "RB_NeutralHash", "ScriptGame_CpuTechnicalClear",
    "ScriptGame_CpuAssistClear", "ftUcf_FlickCounter",
];

#[test]
fn fighter_matches_c() {
    unsafe {
        GO = bz(); JO = bz(); FT = bz(); FDATA = bz(); CDATA = bz(); PADB = bz(); CLB = bz(); PARTS = bz(); PAI = bz(); SRC64 = bz(); IGO = bz(); ITM = bz();
        preg(PADB, core::mem::size_of_val(&*PADB)); preg(CLB, core::mem::size_of_val(&*CLB)); preg(CDATA, core::mem::size_of_val(&*CDATA)); preg(PARTS, core::mem::size_of_val(&*PARTS)); preg(SRC64, core::mem::size_of_val(&*SRC64)); preg(IGO, core::mem::size_of_val(&*IGO)); preg(ITM, core::mem::size_of_val(&*ITM)); preg(PAI, core::mem::size_of_val(&*PAI));
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
            let mut ret = 0u32;
            let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| unsafe {
                world();
                // the draws below are the C driver's, case for case
                let gi = |r: u32| &raw mut (*GO)[(r as usize) % NG];
                let fi_ = |r: u32| &raw mut (*FT)[(r as usize) % NG];
                match op {
                    0 => m::Fighter_8006C5F4(gi(rnd())),
                    1 => m::Fighter_CallAcessoryCallbacks_8006C624(gi(rnd())),
                    2 => m::Fighter_8006C80C(gi(rnd())),
                    3 => m::Fighter_UnkProcessGrab_8006CA5C(gi(rnd())),
                    4 => m::Fighter_8006CB94(gi(rnd())),
                    5 => m::Fighter_8006D9AC(gi(rnd())),
                    6 => m::Fighter_UnkCallCameraCallback_8006D9EC(gi(rnd())),
                    7 => m::Fighter_8006DA4C(gi(rnd())),
                    8 => m::Fighter_8006A1BC(gi(rnd())),
                    9 => m::Fighter_8006A360(gi(rnd())),
                    10 => m::Fighter_8006ABA0(gi(rnd())),
                    11 => m::Fighter_UnkIncrementCounters_8006ABEC(gi(rnd())),
                    12 => m::Fighter_Spaghetti_8006AD10(gi(rnd())),
                    13 => m::Fighter_procUpdate(gi(rnd())),
                    14 => m::Fighter_UnkApplyTransformation_8006C0F0(gi(rnd())),
                    15 => m::Fighter_procMap(gi(rnd())),
                    16 => m::Fighter_ProcessHit_8006D1EC(gi(rnd())),
                    17 => m::Fighter_UnkProcessDeath_80068354(gi(rnd())),
                    18 => m::Fighter_UnkUpdateCostumeJoint_800686E4(gi(rnd())),
                    19 => m::Fighter_ResetInputData_80068854(gi(rnd())),
                    20 => m::Fighter_UnkInitReset_80067C98(&mut *fi_(rnd())),
                    21 => m::Fighter_UnkUpdateVecFromBones_8006876C(&mut *fi_(rnd())),
                    22 => m::Fighter_UnkInitLoad_80068914(gi(rnd()), &*PAI),
                    23 => m::Fighter_Create_Inline2(gi(rnd())),
                    24 => ret = pi(m::Fighter_Create(&*PAI)),
                    25 => m::Fighter_FirstInitialize_80067A84(),
                    26 => m::Fighter_800679B0(),
                    27 => m::Fighter_LoadCommonData(),
                    28 => { let sl = (rnd() & 1) as usize; ret = pi(m::ftCommonData_ExtendKindTable((*PADB)[76 + sl].as_mut_ptr() as *mut *mut c_void, sl)); }
                    29 => { let s_ = if rnd() & 7 != 0 { SRC64 } else { null_mut() }; ret = pi(m::ftCommonData_ExtendCpuTables(s_)); }
                    30 => ret = m::ftSlippi_IsOffscreen(&*fi_(rnd())) as u32,
                    31 => m::ftReplay_TraceFighter(&*fi_(rnd())),
                    32 => ret = m::RB_NeutralHash(),
                    33 => m::ScriptGame_CpuTechnicalClear((fv() % 6) as i32),
                    34 => m::ScriptGame_CpuAssistClear((fv() % 6) as i32),
                    35 => ret = m::ftUcf_FlickCounter((rnd() % 4) as i32) as u32,
                    _ => unreachable!(),
                }
                hworld();
            }));
            if res.is_err() { c.assert_line(op, k); continue; }
            unsafe { c.line(op, k, ret); }
        }
    }
    assert_eq!(c.done(), OPS.len() as u32 * 400);
}
