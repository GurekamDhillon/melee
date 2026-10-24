//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/lb_00B0.c.
//! src/tu/lb_00B0.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_runtime/gen/lbtu/run_tu.py lb_00B0`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]
#[path = "lbtu_harness/mod.rs"]
mod mtu_harness;
use core::ptr::null_mut;
use melee_types::*;
use melee_runtime::tu::lb_00B0 as m;
use mtu_harness::*;
#[path = "lbtu_gen/lb_00B0_gen.rs"]
mod g;

static mut JO: [HSD_JObj; 4] = unsafe { core::mem::zeroed() };
static mut JT: [HSD_Joint; 2] = unsafe { core::mem::zeroed() };
static mut AO: [HSD_AObj; 4] = unsafe { core::mem::zeroed() };
static mut DO: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
static mut PO: [HSD_PObj; 3] = unsafe { core::mem::zeroed() };
static mut SS: [HSD_ShapeSet; 2] = unsafe { core::mem::zeroed() };
static mut MO: [HSD_MObj; 2] = unsafe { core::mem::zeroed() };
static mut TO: [HSD_TObj; 3] = unsafe { core::mem::zeroed() };
static mut RO: [HSD_RObj; 3] = unsafe { core::mem::zeroed() };
static mut AJ: [HSD_AnimJoint; 4] = unsafe { core::mem::zeroed() };
static mut AD: [HSD_AObjDesc; 2] = unsafe { core::mem::zeroed() };
static mut TD: [HSD_TevDesc; 3] = unsafe { core::mem::zeroed() };
static mut TED: [HSD_TExpTevDesc; 2] = unsafe { core::mem::zeroed() };
static mut TE: [HSD_TExp; 3] = unsafe { core::mem::zeroed() };
static mut LO: [HSD_LObj; 3] = unsafe { core::mem::zeroed() };
static mut DMD: [DynamicModelDesc; 1] = unsafe { core::mem::zeroed() };
static mut V: [Vec; 3] = unsafe { core::mem::zeroed() };
static mut MZ: [u8; 64] = unsafe { core::mem::zeroed() };
static mut FL: [f32; 4] = [0.0; 4];
static mut FPP: [*mut f32; 2] = [null_mut(); 2];
static mut AJP: [*mut HSD_AnimJoint; 4] = [null_mut(); 4];
static mut MAJP: [*mut HSD_MatAnimJoint; 4] = [null_mut(); 4];
static mut SAJP: [*mut HSD_ShapeAnimJoint; 4] = [null_mut(); 4];
static mut MAJB: [[u8; 16]; 4] = [[0; 16]; 4];
static mut SAJB: [[u8; 16]; 4] = [[0; 16]; 4];

unsafe fn setup() {
    let _ = 0;
    JO = core::mem::zeroed();
    JT = core::mem::zeroed();
    AO = core::mem::zeroed();
    DO = core::mem::zeroed();
    PO = core::mem::zeroed();
    SS = core::mem::zeroed();
    MO = core::mem::zeroed();
    TO = core::mem::zeroed();
    RO = core::mem::zeroed();
    AJ = core::mem::zeroed();
    AD = core::mem::zeroed();
    TD = core::mem::zeroed();
    TED = core::mem::zeroed();
    TE = core::mem::zeroed();
    LO = core::mem::zeroed();
    DMD = core::mem::zeroed();
    V = core::mem::zeroed();
    MZ = core::mem::zeroed();
    for i in 0..4 { g::fill_HSD_JObj(&mut JO[i]); }
    for i in 0..2 { g::fill_HSD_Joint(&mut JT[i]); }
    for i in 0..4 { g::fill_HSD_AObj(&mut AO[i]); }
    for i in 0..3 { g::fill_HSD_DObj(&mut DO[i]); }
    for i in 0..3 { g::fill_HSD_PObj(&mut PO[i]); }
    for i in 0..2 { g::fill_HSD_ShapeSet(&mut SS[i]); }
    for i in 0..2 { g::fill_HSD_MObj(&mut MO[i]); }
    for i in 0..3 { g::fill_HSD_TObj(&mut TO[i]); }
    for i in 0..3 { g::fill_HSD_RObj(&mut RO[i]); }
    for i in 0..4 { g::fill_HSD_AnimJoint(&mut AJ[i]); }
    for i in 0..2 { g::fill_HSD_AObjDesc(&mut AD[i]); }
    for i in 0..3 { g::fill_HSD_TevDesc(&mut TD[i]); }
    for i in 0..2 { g::fill_HSD_TExpTevDesc(&mut TED[i]); }
    for i in 0..3 { g::fill_HSD_TExp(&mut TE[i]); }
    for i in 0..3 { g::fill_HSD_LObj(&mut LO[i]); }
    for i in 0..1 { g::fill_DynamicModelDesc(&mut DMD[i]); }
    for i in 0..3 { g::fill_Vec(&mut V[i]); }
    for k in 0..4 {
        JO[k].next = null_mut(); JO[k].parent = null_mut(); JO[k].child = null_mut();
        JO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        JO[k].u.dobj = if rnd() & 1 != 0 { &raw mut DO[(rnd() % 3) as usize] } else { null_mut() };
    }
    for k in 1..4usize {
        let p = (rnd() % k as u32) as usize;
        let mut c = JO[p].child;
        if c.is_null() { JO[p].child = &raw mut JO[k]; } else { while !(*c).next.is_null() { c = (*c).next; } (*c).next = &raw mut JO[k]; }
        JO[k].parent = &raw mut JO[p];
    }
    for k in 0..2usize { JT[k].next = if rnd() & 1 != 0 { &raw mut JT[1 - k] } else { null_mut() }; JT[k].child = null_mut(); }
    JT[1].next = null_mut();
    JT[0].child = if rnd() & 1 != 0 { &raw mut JT[1] } else { null_mut() };
    for k in 0..3usize {
        DO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut DO[k + 1] } else { null_mut() };
        DO[k].pobj = if rnd() & 1 != 0 { &raw mut PO[(rnd() % 3) as usize] } else { null_mut() };
        DO[k].mobj = if rnd() & 1 != 0 { &raw mut MO[(rnd() % 2) as usize] } else { null_mut() };
        PO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut PO[k + 1] } else { null_mut() };
        PO[k].flags = if rnd() & 1 != 0 { 0x1000 } else { fv() as u16 };
        PO[k].u.shape_set = &raw mut SS[(rnd() % 2) as usize];
        TO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut TO[k + 1] } else { null_mut() };
        TO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        RO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut RO[k + 1] } else { null_mut() };
        LO[k].next = if k < 2 { &raw mut LO[k + 1] } else { null_mut() };
        TD[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut TD[k + 1] } else { null_mut() };
        TD[k].u.tevconf.clr_out_reg = fv() % 4; TD[k].u.tevconf.alpha_out_reg = fv() % 4;
        TE[k].cnst.next = if k < 2 && rnd() & 1 != 0 { &raw mut TE[k + 1] } else { null_mut() };
        TE[k].cnst.reg = (fv() % 10) as u8;
    }
    LO[2].flags &= !3;
    for k in 0..2usize {
        SS[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        MO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        MO[k].tobj = if rnd() & 1 != 0 { &raw mut TO[(rnd() % 3) as usize] } else { null_mut() };
        MO[k].tevdesc = &raw mut TED[k];
        MO[k].texp = if rnd() & 1 != 0 { &raw mut TE[(rnd() % 3) as usize] } else { null_mut() };
        TED[k].desc.next = if rnd() & 1 != 0 { &raw mut TD[(rnd() % 3) as usize] } else { null_mut() };
        TED[k].desc.u.tevconf.clr_out_reg = fv() % 4; TED[k].desc.u.tevconf.alpha_out_reg = fv() % 4;
    }
    for k in 0..4usize {
        AJ[k].child = if k < 3 && rnd() & 1 != 0 { &raw mut AJ[k + 1] } else { null_mut() };
        AJ[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut AJ[k + 2] } else { null_mut() };
        AJ[k].aobjdesc = if rnd() & 1 != 0 { &raw mut AD[(rnd() % 2) as usize] } else { null_mut() };
        AJP[k] = &raw mut AJ[k]; MAJP[k] = MAJB[k].as_mut_ptr() as *mut _; SAJP[k] = SAJB[k].as_mut_ptr() as *mut _;
    }
    DMD[0].anims = if rnd() & 1 != 0 { AJP.as_mut_ptr() } else { null_mut() };
    DMD[0].matanims = if rnd() & 1 != 0 { MAJP.as_mut_ptr() } else { null_mut() };
    DMD[0].shapeanims = if rnd() & 1 != 0 { SAJP.as_mut_ptr() } else { null_mut() };
    for k in 0..4 { FL[k] = rf(); }
    FPP[0] = &raw mut FL[0]; FPP[1] = &raw mut FL[1];
    for k in 0..64 { MZ[k] = (rnd() | 1) as u8; }
}
unsafe fn hash_all() {
    for k in 0..4 { hm(pib(JO[k].u.dobj)); JO[k].u.dobj = null_mut(); }
    for k in 0..3 { hm(pib(PO[k].u.shape_set)); PO[k].u.shape_set = null_mut(); hm(pib(TE[k].cnst.next)); TE[k].cnst.next = null_mut(); }
    for i in 0..4 { g::hash_HSD_JObj(&JO[i]); }
    for i in 0..2 { g::hash_HSD_Joint(&JT[i]); }
    for i in 0..4 { g::hash_HSD_AObj(&AO[i]); }
    for i in 0..3 { g::hash_HSD_DObj(&DO[i]); }
    for i in 0..3 { g::hash_HSD_PObj(&PO[i]); }
    for i in 0..2 { g::hash_HSD_ShapeSet(&SS[i]); }
    for i in 0..2 { g::hash_HSD_MObj(&MO[i]); }
    for i in 0..3 { g::hash_HSD_TObj(&TO[i]); }
    for i in 0..3 { g::hash_HSD_RObj(&RO[i]); }
    for i in 0..4 { g::hash_HSD_AnimJoint(&AJ[i]); }
    for i in 0..2 { g::hash_HSD_AObjDesc(&AD[i]); }
    for i in 0..3 { g::hash_HSD_TevDesc(&TD[i]); }
    for i in 0..2 { g::hash_HSD_TExpTevDesc(&TED[i]); }
    for i in 0..3 { g::hash_HSD_TExp(&TE[i]); }
    for i in 0..3 { g::hash_HSD_LObj(&LO[i]); }
    for i in 0..1 { g::hash_DynamicModelDesc(&DMD[i]); }
    for i in 0..3 { g::hash_Vec(&V[i]); }
    for k in 0..4 { hm(FL[k].to_bits()); }
    for k in 0..64 { hm(MZ[k] as u32); }
}
#[test]
fn lb_00B0_matches_c() {
    unsafe {
        g::install();
        preg(&raw const JO, core::mem::size_of_val(&JO));
        preg(&raw const JT, core::mem::size_of_val(&JT));
        preg(&raw const AO, core::mem::size_of_val(&AO));
        preg(&raw const DO, core::mem::size_of_val(&DO));
        preg(&raw const PO, core::mem::size_of_val(&PO));
        preg(&raw const SS, core::mem::size_of_val(&SS));
        preg(&raw const MO, core::mem::size_of_val(&MO));
        preg(&raw const TO, core::mem::size_of_val(&TO));
        preg(&raw const RO, core::mem::size_of_val(&RO));
        preg(&raw const AJ, core::mem::size_of_val(&AJ));
        preg(&raw const AD, core::mem::size_of_val(&AD));
        preg(&raw const TD, core::mem::size_of_val(&TD));
        preg(&raw const TED, core::mem::size_of_val(&TED));
        preg(&raw const TE, core::mem::size_of_val(&TE));
        preg(&raw const LO, core::mem::size_of_val(&LO));
        preg(&raw const DMD, core::mem::size_of_val(&DMD));
        preg(&raw const V, core::mem::size_of_val(&V));
        preg(&raw const MZ, core::mem::size_of_val(&MZ));
        preg(&raw const FL, 16); preg(&raw const MAJB, 64); preg(&raw const SAJB, 64); preg(&raw const FPP, core::mem::size_of_val(&FPP)); preg(&raw const AJP, core::mem::size_of_val(&AJP)); preg(&raw const MAJP, core::mem::size_of_val(&MAJP)); preg(&raw const SAJP, core::mem::size_of_val(&SAJP));
        g::freg_all();

        let mut c = Check::new(include_str!("data/lbtu_lb_00B0.txt"));
        for op in 0..42u32 {
            for k in 0..400u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                match op {
                    0 => {   ret = m::lb_8000B074(&raw mut JO[0]) as u32; } // lb_8000B074
                    1 => {   ret = m::lb_8000B09C(&raw mut JO[0]) as u32; } // lb_8000B09C
                    2 => {   ret = m::lb_8000B134(&raw mut JO[0]) as u32; } // lb_8000B134
                    3 => {   m::lb_8000B1CC(&raw mut JO[0], &raw mut V[0], &raw mut V[1]); } // lb_8000B1CC
                    4 => {   m::lb_8000B4FC(&raw mut JO[0], &raw mut JT[0]); } // lb_8000B4FC
                    5 => {   m::lb_8000B5DC(&raw mut JO[0], &raw mut JT[0]); } // lb_8000B5DC
                    6 => {   m::lb_8000B6A4(&raw mut JO[0], &raw mut JT[0]); } // lb_8000B6A4
                    7 => {   m::lb_8000B760(&raw mut JO[0], &raw mut JT[0]); } // lb_8000B760
                    8 => {   m::lb_8000B804(&raw mut JO[0], &raw mut JT[0]); } // lb_8000B804
                    9 => {  let v2: i32 = (fv() & 1) as i32; m::lb_8000B9D8(&raw mut JO[0], &raw mut FPP[0], v2); } // lb_8000B9D8
                    10 => {  let v1: f32 = (rf()) as f32; m::lb_8000BA0C(&raw mut JO[0], v1); } // lb_8000BA0C
                    11 => {  let v1: f32 = (rf()) as f32; m::lbDObjSetRateAll(&raw mut DO[0], v1); } // lbDObjSetRateAll
                    12 => {  let v1: f32 = (rf()) as f32; m::lbDObjReqAnimAll(&raw mut DO[0], v1); } // lbDObjReqAnimAll
                    13 => {   ret = pi(m::lbFindJObjWithAObj(&raw mut JO[0])); } // lbFindJObjWithAObj
                    14 => {   ret = (m::lbGetJObjFramerate(&raw mut JO[0])).to_bits(); } // lbGetJObjFramerate
                    15 => {   ret = (m::lbGetJObjCurrFrame(&raw mut JO[0])).to_bits(); } // lbGetJObjCurrFrame
                    16 => {   ret = (m::lbGetJObjEndFrame(&raw mut JO[0])).to_bits(); } // lbGetJObjEndFrame
                    17 => {   ret = pi(m::lb_8000BECC(&raw mut AJ[0])); } // lb_8000BECC
                    18 => {   ret = (m::lb_8000BFF0(&raw mut AJ[0])).to_bits(); } // lb_8000BFF0
                    19 => {  let v1: i32 = (fv() % 4) as i32; let v2: *mut *mut HSD_AnimJoint = if fv() & 1 != 0 { AJP.as_mut_ptr() } else { null_mut() }; let v3: *mut *mut HSD_MatAnimJoint = if fv() & 1 != 0 { MAJP.as_mut_ptr() } else { null_mut() }; let v4: *mut *mut HSD_ShapeAnimJoint = if fv() & 1 != 0 { SAJP.as_mut_ptr() } else { null_mut() }; m::lb_8000C07C(&raw mut JO[0], v1, v2, v3, v4); } // lb_8000C07C
                    20 => {  let v1: i32 = (fv() % 4) as i32; m::lb_8000C0E8(&raw mut JO[0], v1, &raw mut DMD[0]); } // lb_8000C0E8
                    21 => {  let v1: i32 = (fv() % 65) as _; m::memzero(MZ.as_mut_ptr() as *mut core::ffi::c_void, v1); } // memzero
                    22 => {   m::lb_8000C1C0(&raw mut JO[0], &raw mut JO[1]); } // lb_8000C1C0
                    23 => {   m::lb_8000C228(&raw mut JO[0], &raw mut JO[1]); } // lb_8000C228
                    24 => {   m::lb_8000C290(&raw mut JO[0], &raw mut JO[1]); } // lb_8000C290
                    25 => {   m::lb_8000C2F8(&raw mut JO[0], &raw mut JO[1]); } // lb_8000C2F8
                    26 => {   m::lb_8000C390(&raw mut JO[0]); } // lb_8000C390
                    27 => {  let v1: u32 = (fv()) as u32; let v2: f32 = (rf()) as f32; m::lb_8000C420(&raw mut JO[0], v1, v2); } // lb_8000C420
                    28 => {  let v3: f32 = (rf()) as f32; let v4: f32 = (rf()) as f32; m::lb_8000C490(&raw mut JO[0], &raw mut JO[1], &raw mut JO[2], v3, v4); } // lb_8000C490
                    29 => {   m::lbCopyJObjSRT(&raw mut JO[0], &raw mut JO[1]); } // lbCopyJObjSRT
                    30 => {  let v3: f32 = (rf()) as f32; let v4: f32 = (rf()) as f32; m::lb_8000C868(&raw mut JT[0], &raw mut JO[0], &raw mut JO[1], v3, v4); } // lb_8000C868
                    31 => {  let v0: i32 = (fv() % 9) as i32; let v2: *mut HSD_TExp = if fv() & 1 != 0 { &raw mut TE[0] } else { null_mut() }; let v3: *mut HSD_TExp = if fv() & 1 != 0 { &raw mut TE[1] } else { null_mut() }; ret = m::lbGetFreeColorRegImpl(v0, &raw mut TD[0], v2, v3) as u32; } // lbGetFreeColorRegImpl
                    32 => {  let v0: i32 = (fv() % 9) as i32; ret = m::lbGetFreeColorRegister(v0, &raw mut MO[0], &raw mut TE[0]) as u32; } // lbGetFreeColorRegister
                    33 => {  let v0: i32 = (fv() % 8) as i32; ret = m::lb_8000CC8C(v0) as u32; } // lb_8000CC8C
                    34 => {  let v0: i32 = (fv() % 4) as i32; ret = m::lb_8000CCA4(v0) as u32; } // lb_8000CCA4
                    35 => {  let v0: i32 = (fv() % 9) as i32; let v2: *mut HSD_TExp = null_mut(); let v3: *mut HSD_TExp = null_mut(); ret = m::lbGetFreeAlphaRegImpl(v0, &raw mut TD[0], v2, v3) as u32; } // lbGetFreeAlphaRegImpl
                    36 => {  let v0: i32 = (fv() % 9) as i32; ret = m::lbGetFreeAlphaRegister(v0, &raw mut MO[0], &raw mut TE[0]) as u32; } // lbGetFreeAlphaRegister
                    37 => {  let v0: i32 = (fv() % 8) as i32; ret = m::lb_8000CD90(v0) as u32; } // lb_8000CD90
                    38 => {  let v0: i32 = (fv() % 4) as i32; ret = m::lb_8000CDA8(v0) as u32; } // lb_8000CDA8
                    39 => {   ret = pi(m::lb_8000CDC0(&raw mut LO[0])); } // lb_8000CDC0
                    40 => {   m::lb_8000CE30(&raw mut DO[0], &raw mut DO[1]); } // lb_8000CE30
                    41 => {   m::lb_8000CE40(&raw mut JO[0], &raw mut DO[0]); } // lb_8000CE40
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 16800);
    }
}
