//! Written by gen/mtu/mk_ef.py from the same table as gen/mtu/tu/ef.c.
//! tu::efn::{eflib,efasync} vs the C (eflib.c + efasync.c + efdata.c, shipping contraction; regenerate the
//! expected file with `python rust/melee_world/gen/mtu/mk_ef.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::efn::eflib as el;
use melee_world::tu::efn::efasync as ea;
use melee_world::tu::efn::efdata as ed;
use melee_world::tu::va::Va;
use mtu_harness::*;
#[path = "mtu/ef_gen.rs"]
mod g;

static mut EFF: [EF_Effect; 10] = unsafe { core::mem::zeroed() };
static mut QE: [EF_QueuedEffect; 8] = unsafe { core::mem::zeroed() };
static mut QH: [EF_QueuedEffect; 1] = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 9] = unsafe { core::mem::zeroed() };
static mut JO: [HSD_JObj; 16] = unsafe { core::mem::zeroed() };
static mut FT: [Fighter; 2] = unsafe { core::mem::zeroed() };
static mut ITM: [Item; 1] = unsafe { core::mem::zeroed() };
static mut DO: [HSD_DObj; 4] = unsafe { core::mem::zeroed() };
static mut MO: [HSD_MObj; 4] = unsafe { core::mem::zeroed() };
static mut TO: [HSD_TObj; 8] = unsafe { core::mem::zeroed() };
static mut TV: [HSD_TObjTev; 8] = unsafe { core::mem::zeroed() };
static mut GEN: [HSD_Generator; 3] = unsafe { core::mem::zeroed() };
static mut APS: [HSD_psAppSRT; 3] = unsafe { core::mem::zeroed() };
static mut PT: [HSD_Particle; 2] = unsafe { core::mem::zeroed() };
static mut PROC: [HSD_GObjProc; 2] = unsafe { core::mem::zeroed() };
static mut DESC: [EF_EffectDesc; 64] = unsafe { core::mem::zeroed() };
static mut DE: [EF_DAT_Entry; 2] = unsafe { core::mem::zeroed() };
static mut DEINIT: [EF_DAT_Entry; 65] = unsafe { core::mem::zeroed() };
static mut BHV: ea::EfMexBehavior = unsafe { core::mem::zeroed() };
static mut BT1: [u8; 16] = [0; 16];
static mut BT2: [u8; 16] = [0; 16];
static STRS: [[u8; 24]; 3] = [*b"MexA.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", *b"MexB.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", *b"MexC.dat\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"];
static mut ARC: [HSD_Archive; 1] = unsafe { core::mem::zeroed() };
static mut PLH: [*mut HSD_GObj; 64] = [null_mut(); 64];
static mut V3: [Vec3; 6] = unsafe { core::mem::zeroed() };
static mut FL: [f32; 6] = [0.0; 6];
static mut CMD: [_HSD_PSCmdList; 5] = unsafe { core::mem::zeroed() };
static mut PREF: [*mut _HSD_PSCmdList; 0x220] = [null_mut(); 0x220];
static mut AJ: [u8; 4] = [0; 4];
static mut EFN: usize = 0;
static mut QN: usize = 0;
static mut OUTI: [i32; 4] = [0; 4];
static mut OUTP: [*mut HSD_GObj; 2] = [null_mut(); 2];
static DID: [i32; 144] = [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1027, 1028, 1029, 1030, 1031, 1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039, 1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1111, 1103, 1112, 1104, 1113, 1105, 1114, 1107, 1108, 1109, 1110, 1106, 1115, 1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1132, 1133, 1134, 1135, 1136, 1137, 1138, 1139, 1143, 1140, 1141, 1142];
static DKIND: [&str; 144] = ["VVFVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VFFVVV", "VFVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "JVVVVV", "JVVVVV", "VFVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFVVVV", "JVVVVV", "VFVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JVVVVV", "JVVVVV", "JFVVVV", "VVVVVV", "JUVVVV", "JUVVVV", "JUVVVV", "JUVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JFVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "VFFUUV", "VFVVVV", "VVVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "JVVVVV", "VVVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "JFVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JFVVVV", "VFVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "VFVVVV", "PPVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV"];
fn is_d(g: i32) -> bool { DID.contains(&g) }
static SPLIST: [i32; 34] = [19000, 19001, 19002, 47000, 47001, 47002, 45, 46, 49, 295, 2, 6, 163, 167, 170, 242, 298, 306, 307, 365, 366, 367, 368, 2018, 212, 579, 227, 252, 255, 247, 31000, 289, 85, 485];
static UPD: [unsafe extern "C" fn(*mut EF_Effect); 17] = [el::efLib_Cb_SetOffsetY_FromParamY, el::efLib_Cb_SetScale_FromParamX, el::efLib_Cb_SetRotYAndTransition, el::efLib_Cb_SetJObjOffsetZ, el::efLib_Cb_SetRotY_FromFighterDir, el::efLib_Cb_SetRotYZ_FromFighter, el::efLib_Cb_Fall_FromParamY, el::efLib_Cb_SetOffset_FromParams, el::efLib_Cb_LifetimeEndSpawn, el::efLib_Cb_SetScaleRotY_FromFighter, el::efLib_Cb_SetRotYZ_FromParamZ_FighterDir, el::efLib_Cb_ftMr_SpecialLw, el::efLib_Cb_ftLg_SpecialLw, el::efLib_Cb_ftKp_SpecialHi, el::efLib_Cb_ftCo_Bury, el::efLib_Cb_ApplyStoredAlpha, el::efLib_Cb_AccumOffset_FromParams];

fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
fn gfx() -> i32 {
    let r = rnd();
    let bank = (rnd() % 48) as i32;
    let id = (rnd() % 64) as i32;
    if r % 24 == 0 { return (fv() % 70) as i32 * 1000 + id; }
    if r % 24 == 1 { return 65000 + id; }
    bank * 1000 + id
}
fn gg() -> i32 {
    let r = rnd();
    const SP: [i32; 5] = [0x121, 0xFF, 0x7918, 0xFC, 0xF7];
    if r % 3 == 0 { return SP[(rnd() % 5) as usize]; }
    gfx()
}
fn spg() -> i32 {
    let r = rnd();
    if r % 6 == 0 { return gfx(); }
    SPLIST[(rnd() % SPLIST.len() as u32) as usize]
}
unsafe fn mxg() -> *mut HSD_GObj {
    let r = rnd() % 6;
    if r == 0 { return null_mut(); }
    if r == 1 { return &raw mut GO[8]; }
    &raw mut GO[6 + (rnd() & 1) as usize]
}
fn mxid() -> i32 {
    let r = rnd() % 8;
    if r == 0 { return (rnd() % 5000) as i32; }
    if r == 1 { return 5000 + 1000 * (rnd() % 5) as i32; }
    5000 + (rnd() % 4100) as i32
}
unsafe fn argp(kind: i32) -> *const c_void {
    match kind {
        0 => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
        1 => &raw mut FL[(rnd() % 6) as usize] as *const c_void,
        _ => &raw mut JO[(rnd() % 16) as usize] as *const c_void,
    }
}
unsafe fn argd(d: usize, i: usize) -> *const c_void {
    match DKIND[d].as_bytes()[i] {
        b'V' => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
        b'F' => &raw mut FL[(rnd() % 6) as usize] as *const c_void,
        b'J' => &raw mut JO[(rnd() % 16) as usize] as *const c_void,
        b'U' => fv().wrapping_mul(0x10101) as usize as *const c_void,
        _ => &raw mut V3[(rnd() % 6) as usize] as *const c_void,
    }
}

unsafe fn fill_fighter(p: &mut Fighter) {
    p.facing_dir = rg(); p.motion_id = fv() as _; p.cmd_vars[0] = fv(); p.cmd_vars[1] = fv(); p.cmd_vars[2] = fv(); p.cmd_vars[3] = fv();
    p.mv.co.common.x4.x = rg(); p.mv.co.common.x4.y = rg(); p.mv.co.common.x4.z = rg();
    p.mv.co.common.x10 = rnd() & 1;
    p.coll_data.floor.normal.x = rg(); p.coll_data.floor.normal.y = rg();
    p.u.kb.hat.kind = fv() as _; p.kind = fv() as _;
}
unsafe fn hash_fighter(p: &Fighter) {
    hm(p.facing_dir.to_bits()); hm(p.motion_id as u32); hm(p.cmd_vars[0]); hm(p.cmd_vars[1]); hm(p.cmd_vars[2]); hm(p.cmd_vars[3]);
    hm(p.mv.co.common.x4.x.to_bits()); hm(p.mv.co.common.x4.y.to_bits()); hm(p.mv.co.common.x4.z.to_bits()); hm(p.mv.co.common.x10);
    hm(p.coll_data.floor.normal.x.to_bits()); hm(p.coll_data.floor.normal.y.to_bits()); hm(p.u.kb.hat.kind as u32); hm(p.kind as u32);
}

unsafe fn setup() {
    EFF = core::mem::zeroed(); QE = core::mem::zeroed(); QH = core::mem::zeroed(); GO = core::mem::zeroed(); JO = core::mem::zeroed();
    FT = core::mem::zeroed(); ITM = core::mem::zeroed(); DO = core::mem::zeroed(); MO = core::mem::zeroed(); TO = core::mem::zeroed();
    TV = core::mem::zeroed(); GEN = core::mem::zeroed(); APS = core::mem::zeroed(); PT = core::mem::zeroed(); PROC = core::mem::zeroed();
    DESC = core::mem::zeroed(); DE = core::mem::zeroed(); PLH = [null_mut(); 64]; V3 = core::mem::zeroed(); FL = [0.0; 6];
    OUTI = [0; 4]; OUTP = [null_mut(); 2]; ARC = core::mem::zeroed(); CMD = core::mem::zeroed();
    EFN = 6; QN = 4;
    for i in 0..9 {
        g::fill_HSD_GObj(&mut GO[i]); GO[i].hsd_obj = &raw mut JO[i] as *mut c_void; GO[i].gx_link = if rnd() & 1 != 0 { 7 } else { 8 };
        GO[i].obj_kind = if rnd() % 3 == 0 { rnd() as u8 } else { 2 };
    }
    for i in 0..16 {
        g::fill_HSD_JObj(&mut JO[i]); JO[i].flags &= !(1u32 << 17);
        if rnd() & 1 != 0 { JO[i].flags |= 1 << 6; }
        if rnd() % 4 == 0 { JO[i].flags |= 1 << 25; }
    }
    fill_fighter(&mut FT[0]); fill_fighter(&mut FT[1]);
    for i in 0..10 { g::fill_EF_Effect(&mut EFF[i]); }
    for i in 0..8 { g::fill_EF_QueuedEffect(&mut QE[i]); }
    g::fill_EF_QueuedEffect(&mut QH[0]);
    for i in 0..4 { g::fill_HSD_DObj(&mut DO[i]); g::fill_HSD_MObj(&mut MO[i]); }
    for i in 0..8 { g::fill_HSD_TObj(&mut TO[i]); g::fill_HSD_TObjTev(&mut TV[i]); }
    for i in 0..3 { g::fill_HSD_Generator(&mut GEN[i]); g::fill_HSD_psAppSRT(&mut APS[i]); }
    for i in 0..2 { g::fill_HSD_Particle(&mut PT[i]); g::fill_HSD_GObjProc(&mut PROC[i]); g::fill_EF_DAT_Entry(&mut DE[i]); }
    for i in 0..64 { g::fill_EF_EffectDesc(&mut DESC[i]); }
    for i in 0..6 { V3[i].x = rnn(); V3[i].y = rnn(); V3[i].z = rnn(); FL[i] = rnn(); }
    for i in 0..16 { BT1[i] = (rnd() % 9) as u8; BT2[i] = (rnd() % 9) as u8; }
    BHV.mdl_num = (rnd() % 12) as i32; BHV.mdl_type = BT1.as_mut_ptr(); BHV.ptcl_num = (rnd() % 12) as i32; BHV.ptcl_type = BT2.as_mut_ptr();
    for i in 0..6 {
        GO[i].user_data = &raw mut EFF[i] as *mut c_void; EFF[i].gobj = &raw mut GO[i]; EFF[i].parent_gobj = &raw mut GO[6 + (rnd() & 1) as usize];
        EFF[i].attach_jobj = if rnd() % 4 != 0 { &raw mut JO[8 + (rnd() % 8) as usize] } else { null_mut() };
        EFF[i].update = if rnd() % 4 != 0 { Some(UPD[(rnd() % UPD.len() as u32) as usize]) } else { None };
        EFF[i].user_data = if rnd() % 3 != 0 { &raw mut JO[(rnd() % 16) as usize] as *mut c_void } else { null_mut() };
        EFF[i].next = &raw mut EFF[(rnd() % 6) as usize];
        if rnd() % 4 == 0 { EFF[i].lifetime = 1; }
        EFF[i].state_flags = if rnd() % 3 == 0 { (rnd() % 3) as u8 } else { 0 };
        EFF[i].scale_flags = (rnd() % 3) as u8;
        EFF[i].is_async = (rnd() & 1) as u8;
    }
    GO[6].user_data = &raw mut FT[0] as *mut c_void; GO[7].user_data = &raw mut FT[1] as *mut c_void; GO[6].classifier = 4; GO[7].classifier = 4;
    GO[8].classifier = 6; GO[8].user_data = &raw mut ITM[0] as *mut c_void;
    ITM[0].mex_original_owner = if rnd() & 3 != 0 { &raw mut GO[6 + (rnd() & 1) as usize] } else { null_mut() };
    for i in 0..2 {
        let mm = rnd() % 5;
        if mm == 0 { FT[i].motion_id = 349; } else if mm == 1 { FT[i].motion_id = 357; } else if mm == 2 { FT[i].motion_id = 359; }
        FT[i].kind = if rnd() % 3 == 0 { 4 } else { (rnd() % 127) as _ };
    }
    for i in 1..16usize { JO[i].parent = if rnd() & 1 != 0 { &raw mut JO[(rnd() % i as u32) as usize] } else { null_mut() }; }
    for i in 0..6 { JO[i].child = &raw mut JO[8 + (rnd() & 1) as usize]; }
    for i in 8..15 { JO[i].next = &raw mut JO[i + 1]; }
    JO[6].child = if rnd() & 1 != 0 { &raw mut JO[10] } else { null_mut() };
    JO[7].child = if rnd() & 1 != 0 { &raw mut JO[11] } else { null_mut() };
    JO[8].child = if rnd() & 1 != 0 { &raw mut JO[12] } else { null_mut() };
    for i in 0..4 { DO[i].mobj = if rnd() % 4 != 0 { &raw mut MO[i] } else { null_mut() }; }
    for i in 0..8 { TO[i].next = &raw mut TO[(i + 1) % 8]; TO[i].tev = &raw mut TV[i]; }
    for i in 0..3 {
        GEN[i].appsrt = if rnd() & 1 != 0 { &raw mut APS[(rnd() % 3) as usize] } else { null_mut() };
        GEN[i].cmdList = if rnd() % 3 != 0 { CMD[(rnd() % 5) as usize].cmdList.as_mut_ptr() } else { null_mut() };
    }
    for i in 0..2 {
        PT[i].appsrt = if rnd() & 1 != 0 { &raw mut APS[(rnd() % 3) as usize] } else { null_mut() };
        PROC[i].s_link = (rnd() % 12) as u8;
    }
    for i in 0..0x220 { PREF[i] = &raw mut CMD[4]; }
    PREF[0x96] = &raw mut CMD[0]; PREF[0x97] = &raw mut CMD[1]; PREF[0x98] = &raw mut CMD[2]; PREF[0x21B] = &raw mut CMD[3];
    for i in 0..65 { el::ext::ptclref_804D0E5C[i] = null_mut(); }
    el::ext::ptclref_804D0E5C[0] = PREF.as_mut_ptr();
    for i in 0..64 {
        DESC[i].model_desc.joint = if rnd() % 4 != 0 { &raw mut AJ[0] as *mut HSD_Joint } else { null_mut() };
        DESC[i].model_desc.animjoint = if rnd() & 1 != 0 { &raw mut AJ[1] as *mut HSD_AnimJoint } else { null_mut() };
        DESC[i].model_desc.matanim_joint = if rnd() % 4 == 0 { &raw mut AJ[2] as *mut HSD_MatAnimJoint } else { null_mut() };
        DESC[i].model_desc.shapeanim_joint = if rnd() % 4 == 0 { &raw mut AJ[3] as *mut HSD_ShapeAnimJoint } else { null_mut() };
        if rnd() & 1 != 0 { DESC[i].lifetime = (rnd() % 700) as f32 / 10.0; }
    }
    for i in 0..2 {
        DE[i].ef_DAT_file = if rnd() & 1 != 0 { STRS[0].as_ptr() as *mut i8 } else { null_mut() };
        DE[i].effDataTable_name = if rnd() & 1 != 0 { STRS[1].as_ptr() as *mut i8 } else { null_mut() };
    }
    for i in 0..65 {
        ea::efAsync_DatEntries[i] = core::ptr::read(&raw const DEINIT[i]);
        ea::efAsync_DatEntries[i].data = if rnd() % 8 != 0 { DESC.as_mut_ptr() as *mut c_void } else { null_mut() };
        ea::efAsync_MexBhv[i] = if rnd() & 1 != 0 { &raw mut BHV } else { null_mut() };
    }
    ea::efAsync_MexReady = (rnd() & 1) as i32;
    // 128 draws as the C driver makes; the table is Ft_Kind_Max (127) long since b9e539b85, so the last one is dropped
    for i in 0..128 {
        let v = if rnd() % 6 == 0 { 0xFF } else { (rnd() % 70) as u8 };
        if let Some(e) = ea::ext::ftData_UnkBytePerCharacter.get_mut(i) { *e = v; }
    }
    ed::efLib_EffectCount = rnd() % 64; ed::efLib_LoadKind = rnd() & 1; ed::efLib_AnimCount = (rnd() % 8) as i32;
    el::efLib_AnimQueue = core::mem::zeroed();
    for i in 0..ed::efLib_AnimCount as usize { *(el::efLib_AnimQueue.as_mut_ptr() as *mut *mut HSD_JObj).add(i) = &raw mut JO[(rnd() % 16) as usize]; }
    for i in 0..8 {
        g::fill_EF_ParamEntry(&mut el::efLib_ParamTable[i]);
        el::efLib_ParamTable[i].gobj = if rnd() & 3 != 0 { &raw mut GO[(rnd() % 8) as usize] as *mut c_void } else { null_mut() };
        let j = rnd() % 4;
        if j == 0 { el::efLib_ParamTable[i].gfx_id = 0x417; } else if j == 1 { el::efLib_ParamTable[i].gfx_id = 0x419; }
    }
    el::ext::hsd_804D7900 = None;
    el::ext::HSD_GObj_JObjKind = 2;
    ea::ext::HSD_GObj_CurrentInvokedProc = if rnd() & 1 != 0 { &raw mut PROC[0] } else { null_mut() };
    el::ext::HSD_GObjPLinkHead = PLH.as_mut_ptr();
    let mut ord = [0usize, 1, 2, 3, 4, 5];
    for i in (1..6usize).rev() { let j = (rnd() % (i as u32 + 1)) as usize; ord.swap(i, j); }
    let split = (rnd() % 7) as usize;
    PLH[11] = if split != 0 { &raw mut GO[ord[0]] } else { null_mut() };
    for i in 0..split.saturating_sub(1) { GO[ord[i]].next = &raw mut GO[ord[i + 1]]; }
    PLH[12] = if split < 6 { &raw mut GO[ord[split]] } else { null_mut() };
    for i in split..5 { GO[ord[i]].next = &raw mut GO[ord[i + 1]]; }
    for i in 0..4 {
        QE[i].spawn_kind = (rnd() % 10) as u8; QE[i].gfx_id = gfx();
        QE[i].jobj = &raw mut JO[8 + (rnd() % 8) as usize];
        if QE[i].spawn_kind == 9 { QE[i].gfx_id = 5000 + (rnd() % 4100) as i32; }
        QE[i].next = if i < 3 { &raw mut QE[i + 1] } else { null_mut() };
    }
    QH[0].next = if rnd() % 4 != 0 { &raw mut QE[0] } else { null_mut() };
}

unsafe fn hash_all() {
    for i in 0..10 { g::hash_EF_Effect(&EFF[i]); }
    for i in 0..8 { g::hash_EF_QueuedEffect(&QE[i]); }
    g::hash_EF_QueuedEffect(&QH[0]);
    for i in 0..9 { g::hash_HSD_GObj(&GO[i]); }
    for i in 0..16 { g::hash_HSD_JObj(&JO[i]); }
    hash_fighter(&FT[0]); hash_fighter(&FT[1]);
    for i in 0..4 { g::hash_HSD_DObj(&DO[i]); g::hash_HSD_MObj(&MO[i]); }
    for i in 0..8 { g::hash_HSD_TObj(&TO[i]); g::hash_HSD_TObjTev(&TV[i]); }
    for i in 0..3 { g::hash_HSD_Generator(&GEN[i]); g::hash_HSD_psAppSRT(&APS[i]); }
    for i in 0..2 { g::hash_HSD_Particle(&PT[i]); g::hash_HSD_GObjProc(&PROC[i]); g::hash_EF_DAT_Entry(&DE[i]); }
    for i in 0..6 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); hm(FL[i].to_bits()); }
    hm(EFN as u32); hm(QN as u32);
    hm(ed::efLib_EffectCount); hm(ed::efLib_LoadKind); hm(ed::efLib_AnimCount as u32);
    for i in 0..16 { hm(pi(*(el::efLib_AnimQueue.as_ptr() as *const *const c_void).add(i))); }
    for i in 0..8 { g::hash_EF_ParamEntry(&el::efLib_ParamTable[i]); }
    for i in 0..3 { hm(fi(el::lbl_803BF810[i])); }
    hm(fi(el::ext::hsd_804D7900));
    for i in 0..65 {
        hm(sh(ea::efAsync_DatEntries[i].ef_DAT_file as *const u8)); hm(sh(ea::efAsync_DatEntries[i].effDataTable_name as *const u8));
        hm(pi(ea::efAsync_DatEntries[i].data)); hm(pi(ea::efAsync_MexBhv[i]));
    }
    hm(ea::efAsync_MexReady as u32);
    hm(OUTI[2] as u32);
}

#[test]
fn ef_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); eprintln!("{}", m); }));
        g::install();
        m::efn::eflib::ext::hook::OSReport = Some(|_| {});
        m::efn::efasync::ext::hook::OSReport = Some(|_| {});
        for i in 0..65 { DEINIT[i] = core::ptr::read(&raw const ea::efAsync_DatEntries[i]); }
        preg(&raw const EFF, core::mem::size_of_val(&EFF)); preg(&raw const QE, core::mem::size_of_val(&QE)); preg(&raw const QH, core::mem::size_of_val(&QH));
        preg(&raw const GO, core::mem::size_of_val(&GO)); preg(&raw const JO, core::mem::size_of_val(&JO)); preg(&raw const FT, core::mem::size_of_val(&FT));
        preg(&raw const ITM, core::mem::size_of_val(&ITM)); preg(&raw const DO, core::mem::size_of_val(&DO)); preg(&raw const MO, core::mem::size_of_val(&MO));
        preg(&raw const TO, core::mem::size_of_val(&TO)); preg(&raw const TV, core::mem::size_of_val(&TV)); preg(&raw const GEN, core::mem::size_of_val(&GEN));
        preg(&raw const APS, core::mem::size_of_val(&APS)); preg(&raw const PT, core::mem::size_of_val(&PT)); preg(&raw const PROC, core::mem::size_of_val(&PROC));
        preg(&raw const DESC, core::mem::size_of_val(&DESC)); preg(&raw const DE, core::mem::size_of_val(&DE));
        preg(&raw const BHV, core::mem::size_of_val(&BHV)); preg(&raw const ARC, core::mem::size_of_val(&ARC)); preg(&raw const PLH, core::mem::size_of_val(&PLH));
        preg(&raw const V3, core::mem::size_of_val(&V3)); preg(&raw const FL, core::mem::size_of_val(&FL));
        preg(&raw const CMD, core::mem::size_of_val(&CMD)); preg(&raw const PREF, core::mem::size_of_val(&PREF)); preg(&raw const AJ, core::mem::size_of_val(&AJ));
        preg(STRS.as_ptr(), core::mem::size_of_val(&STRS)); preg(&raw const BT1, 16); preg(&raw const BT2, 16);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ef.txt"));
        for op in 0..78u32 {
            for k in 0..400u32 {
                case(op, k);
                if std::env::var("MTU_TRACE_CASE").is_ok() { eprintln!("case {} {}", op, k); }
                setup();
                let mut ret = 0u32;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match op {
                    0 => { el::efLib_Init(); }
                    1 => { { let f = fv() as i32; el::efLib_SetFlags(&raw mut GO[6 + (rnd() & 1) as usize], f); } }
                    2 => { el::efLib_Destroy(&raw mut GO[(rnd() % 8) as usize]); }
                    3 => { el::efLib_DestroyAll(&raw mut GO[6 + (rnd() & 1) as usize]); }
                    4 => { el::efLib_PauseAll(&raw mut GO[6 + (rnd() & 1) as usize]); }
                    5 => { el::efLib_ResumeAll(&raw mut GO[6 + (rnd() & 1) as usize]); }
                    6 => { el::efLib_remove_user_data(&raw mut EFF[(rnd() % 6) as usize] as *mut c_void); }
                    7 => { ed::efLib_EffectCount = rnd() % 64; el::efLib_RemoveLast(); }
                    8 => { el::efLib_Update(&raw mut GO[(rnd() % 6) as usize]); }
                    9 => { { let g = gfx(); let p = &raw mut GO[6 + (rnd() & 1) as usize]; ret = pi(el::efLib_Create(g, p)); } }
                    10 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_Create_Attach(g, p, j)); } }
                    11 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_Create_AttachChild(g, p, j)); } }
                    12 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_Create_Attach_Scale(g, p, j)); } }
                    13 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_Create_AttachChild_Scale(g, p, j)); } }
                    14 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_Create_Attach_Scale_FacingDir(g, p, j)); } }
                    15 => { { let g = gfx() as u32; let p = &raw mut GO[6 + (rnd() & 1) as usize]; ret = pi(el::efLib_Create_Attach_Pos(g, p, &raw mut V3[0])); } }
                    16 => { el::efLib_render_callback(&raw mut GO[(rnd() % 8) as usize], (rnd() % 5) as i32 - 1); }
                    17 => { el::efLib_particles_proc_main(&raw mut GO[0]); }
                    18 => { el::efLib_particles_proc_aux(&raw mut GO[0]); }
                    19 => { { let g = gg(); ret = pi(el::efLib_CreateGenerator(g, &raw mut V3[0])); } }
                    20 => { { let g = gfx(); ret = pi(el::efLib_CreateGenerator_AddAppSRT(g)); } }
                    21 => { { let g = gfx(); let d = rnn(); ret = pi(el::efLib_CreateGenerator_Translate_FacingDir(g, &raw mut V3[1], d)); } }
                    22 => { { let g = gfx(); ret = pi(el::efLib_CreateGenerator_Attach(g, &raw mut JO[8 + (rnd() % 8) as usize])); } }
                    23 => { { let g = gfx(); ret = pi(el::efLib_CreateGenerator_Attach_AddAppSRT(g, &raw mut JO[8 + (rnd() % 8) as usize])); } }
                    24 => { { let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [j as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_Attach_Scale(g, &mut va, go)); } }
                    25 => { { let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetScale(g, &mut va)); } }
                    26 => { { let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetFacingDir(g, &mut va)); } }
                    27 => { { let g = gfx(); let j = &raw mut JO[8 + (rnd() % 8) as usize]; let f1 = &raw mut FL[(rnd() % 6) as usize]; let f2 = &raw mut FL[(rnd() % 6) as usize]; let a = [j as *const c_void, f1 as *const c_void, f2 as *const c_void]; let mut va = Va::new(&a); ret = pi(el::efLib_CreateGenerator_AppSRT_SetFacingDirScale(g, &mut va)); } }
                    28 => { { let g = gfx(); let go = &raw mut GO[6 + (rnd() & 1) as usize]; let j = &raw mut JO[8 + (rnd() % 8) as usize]; ret = pi(el::efLib_CreateGenerator_AppSRT_SetPos(g, go, j, &raw mut V3[2])); } }
                    29 => { { let g = spg(); let fl = (rnd() & 1) as i32; el::efLib_SpawnParticleEffect(g / 1000, g, &raw mut JO[8 + (rnd() % 8) as usize], fl); } }
                    30 => { { let b = if rnd() & 3 == 0 { 0x1E } else { (rnd() % 50) as i32 }; let g = spg(); let l = (rnd() % 3) as i32; el::efLib_Cb_SPtcl(l, b, g, &raw mut JO[8 + (rnd() % 8) as usize]); } }
                    31 => { { let b = if rnd() & 3 == 0 { 0x1E } else { (rnd() % 50) as i32 }; let g = spg(); let l = (rnd() % 3) as i32; el::efLib_Cb_DPtcl(l, b, g, &raw mut JO[8 + (rnd() % 8) as usize]); } }
                    32 => { el::efLib_Cb_ParticleRender(&raw mut PT[(rnd() % 2) as usize]); }
                    33 => { el::efLib_Cb_PtclAppSRTHook(&raw mut GEN[(rnd() % 3) as usize]); }
                    34 => { { el::efLib_Cb_SetOffsetY_FromParamY(&raw mut EFF[(rnd() % 6) as usize]); } }
                    35 => { { el::efLib_Cb_SetScale_FromParamX(&raw mut EFF[(rnd() % 6) as usize]); } }
                    36 => { { el::efLib_Cb_SetRotYAndTransition(&raw mut EFF[(rnd() % 6) as usize]); } }
                    37 => { { el::efLib_Cb_SetJObjOffsetZ(&raw mut EFF[(rnd() % 6) as usize]); } }
                    38 => { { el::efLib_Cb_SetRotY_FromFighterDir(&raw mut EFF[(rnd() % 6) as usize]); } }
                    39 => { { el::efLib_Cb_SetRotYZ_FromFighter(&raw mut EFF[(rnd() % 6) as usize]); } }
                    40 => { { el::efLib_Cb_Fall_FromParamY(&raw mut EFF[(rnd() % 6) as usize]); } }
                    41 => { { el::efLib_Cb_SetOffset_FromParams(&raw mut EFF[(rnd() % 6) as usize]); } }
                    42 => { { el::efLib_Cb_LifetimeEndSpawn(&raw mut EFF[(rnd() % 6) as usize]); } }
                    43 => { { el::efLib_Cb_SetScaleRotY_FromFighter(&raw mut EFF[(rnd() % 6) as usize]); } }
                    44 => { { el::efLib_Cb_SetRotYZ_FromParamZ_FighterDir(&raw mut EFF[(rnd() % 6) as usize]); } }
                    45 => { { el::efLib_Cb_ftMr_SpecialLw(&raw mut EFF[(rnd() % 6) as usize]); } }
                    46 => { { el::efLib_Cb_ftLg_SpecialLw(&raw mut EFF[(rnd() % 6) as usize]); } }
                    47 => { { EFF[0].next = &raw mut EFF[1]; el::efLib_Cb_ftKp_SpecialHi(&raw mut EFF[0]); } }
                    48 => { { el::efLib_Cb_ftCo_Bury(&raw mut EFF[(rnd() % 6) as usize]); } }
                    49 => { { el::efLib_Cb_ApplyStoredAlpha(&raw mut EFF[(rnd() % 6) as usize]); } }
                    50 => { { el::efLib_Cb_AccumOffset_FromParams(&raw mut EFF[(rnd() % 6) as usize]); } }
                    51 => { { let c = (rnd() % 4) as i32; let k = fv().wrapping_mul(0x10101); let t = fv().wrapping_mul(0x01010101); el::efLib_SetTevKonstColor(&raw mut JO[(rnd() % 16) as usize], c, k, t); } }
                    52 => { { let a = fv() as u8; el::efLib_SetParamAlpha(&raw mut GO[(rnd() % 8) as usize], a); } }
                    53 => { { let g = gfx(); el::efLib_SetParamGfxId(&raw mut GO[(rnd() % 8) as usize], g); } }
                    54 => { { let z = rw(); el::HSD_JObjSetTranslateZ(&raw mut JO[(rnd() % 16) as usize], z); } }
                    55 => { ea::efAsync_LoadAsync((rnd() % 72) as i32 - 3); }
                    56 => { { let len = fv(); let idx = (rnd() % 65) as i32; ea::efAsync_OnLoad(&raw mut ARC[0], &raw mut V3[0] as *mut u8, len, idx); } }
                    57 => { ea::efAsync_LoadSync((rnd() % 72) as i32 - 3); }
                    58 => { ea::efAsync_QueueFlush(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QH[0] as *mut c_void); }
                    59 => { ea::efAsync_QueueClear(&raw mut QH[0] as *mut c_void); }
                    60 => { ea::efAsync_QueueInit(); }
                    61 => { ea::efAsync_MexInit(); }
                    62 => { ea::efAsync_QueueProcessDeferred(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QE[(rnd() % 4) as usize]); }
                    63 => { { let b = (rnd() % 70) as i32 - 2; ea::efAsync_MexNoteArchive(b, if rnd() & 1 != 0 { &raw mut ARC[0] } else { null_mut() }); } }
                    64 => { { let g = mxg(); let id = mxid(); ret = ea::efAsync_MexResolve(g, id, &raw mut OUTI[0], &raw mut OUTI[1], &raw mut OUTP[0]) as u32; hm(OUTI[0] as u32); hm(OUTI[1] as u32); hm(pi(OUTP[0])); } }
                    65 => { { let g = mxg(); let id = mxid(); let f = rw(); let o = rw(); ea::efAsync_MexSpawn(g, &raw mut QH[0] as *mut c_void, id, &raw mut JO[8 + (rnd() % 8) as usize], &raw mut V3[3], f, o); } }
                    66 => { ea::efAsync_MexProcessDeferred(&raw mut GO[6 + (rnd() & 1) as usize], &raw mut QE[(rnd() % 4) as usize]); }
                    67 => { { let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectScaleXYZ(e, rw()); } }
                    68 => { { let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectRotationZFromPtr(e, &raw mut FL[(rnd() % 6) as usize]); } }
                    69 => { { let e = &raw mut EFF[(rnd() % 6) as usize]; ea::efAsync_SetEffectFacingDir(e, rw()); } }
                    70 => { ea::efAsync_SetEffectRandomRotationZ(&raw mut EFF[(rnd() % 6) as usize]); }
                    71 => { ea::efAsync_SetEffectScale(&raw mut EFF[(rnd() % 6) as usize], &raw mut V3[4]); }
                    72 => { { let kind = rnd() % 9; let g = gfx() as u32; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a1 = argp((rnd() % 3) as i32); let a2 = argp((rnd() % 3) as i32); let a3 = argp(1); ea::efAsync_Spawn(go, &raw mut QH[0] as *mut c_void, kind, g, &raw mut JO[8 + (rnd() % 8) as usize], &[a1, a2, a3]); } }
                    73 => { { let d = ((k + 0) % 144) as usize; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [argd(d, 0), argd(d, 1), argd(d, 2), argd(d, 3), argd(d, 4), argd(d, 5)]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(DID[d], go, &mut va)); } }
                    74 => { { let d = ((k + 48) % 144) as usize; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [argd(d, 0), argd(d, 1), argd(d, 2), argd(d, 3), argd(d, 4), argd(d, 5)]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(DID[d], go, &mut va)); } }
                    75 => { { let d = ((k + 96) % 144) as usize; let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = [argd(d, 0), argd(d, 1), argd(d, 2), argd(d, 3), argd(d, 4), argd(d, 5)]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(DID[d], go, &mut va)); } }
                    76 => { { let go = &raw mut GO[6 + (rnd() & 1) as usize]; FL[0] = (rnd() % 100000) as f32 / 7.0; let a = [&raw mut V3[0] as *const c_void, &raw mut FL[0] as *const c_void]; let mut va = Va::new(&a); ret = pi(ea::efAsync_Dispatch(0x3E8, go, &mut va)); } }
                    77 => { { let mut g = (rnd() % 1500) as i32; if is_d(g) { g += 5000; } let go = &raw mut GO[6 + (rnd() & 1) as usize]; let a = argp(0); let b = [a, a, a, a]; let mut va = Va::new(&b); ret = pi(ea::efAsync_Dispatch(g, go, &mut va)); } }
                    _ => unreachable!(),
                }
                }));
                if let Err(e) = r {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if !(msg.contains(": assertion failed") || msg.contains("lineID") || msg.contains("__assert")) { std::panic::resume_unwind(e); }
                    hm(0xA55E87);
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 78 * 400);
    }
}
