"""Rust half of the tu::ground oracle world (see spec_ground.py)."""
from world_ground_c import ROWNAMES

DECL_R = r"""
macro_rules! z { ($($x:ident),*) => { $( $x = core::mem::zeroed(); )* } }
use core::ptr::null_mut;
static mut ANULL: i32 = 0; static mut STDNULL: i32 = 0; static mut GSN: i32 = 0; static mut AMODE: i32 = 0; static mut MI: i32 = 0;
static mut JI: i32 = 0; static mut GI: i32 = 0; static mut NGRI: i32 = 0; static mut LNI: i32 = 0; static mut WP: i32 = 0; static mut WI: i32 = 0;
static mut ARO: i32 = 0; static mut SD0N: i32 = 0; static mut CKV: i8 = 0; static mut GFAIL: i32 = 0; static mut JLN: i32 = 0;
static mut SP: [StageParam; 4] = unsafe { core::mem::zeroed() }; static mut GP: GroundParam = unsafe { core::mem::zeroed() };
static mut ARCH: UnkArchiveStruct = unsafe { core::mem::zeroed() }; static mut STD: UnkStageDat = unsafe { core::mem::zeroed() };
static mut X8: [UnkStageDat_x8_t; 8] = unsafe { core::mem::zeroed() }; static mut HJ: [HSD_Joint; 8] = unsafe { core::mem::zeroed() };
static mut CAMD: [HSD_CameraDescPerspective; 2] = unsafe { core::mem::zeroed() };
static mut GJA: [GrJoint; 8] = unsafe { core::mem::zeroed() }; static mut GJB: [GrJoint; 8] = unsafe { core::mem::zeroed() };
static mut SPA: [i16; 8] = [0; 8];
#[repr(C)] #[derive(Clone, Copy)] struct EntT { joint: *mut c_void, pairs: *mut i16, pair_count: i32 }
static mut ENT: [EntT; 4] = unsafe { core::mem::zeroed() }; static mut PRS: [[i16; 8]; 4] = [[0; 8]; 4];
static mut SPL: [HSD_Spline; 4] = unsafe { core::mem::zeroed() }; static mut SPLP: [*mut HSD_Spline; 4] = [null_mut(); 4];
static mut LOE: [m::LightOverrideEntry; 6] = unsafe { core::mem::zeroed() }; static mut GSE: [GroundShadowEntry; 6] = unsafe { core::mem::zeroed() };
static mut LD: [HSD_LightDesc; 6] = unsafe { core::mem::zeroed() }; static mut LAN: [HSD_LightAnim; 6] = unsafe { core::mem::zeroed() };
static mut ANL: [[*mut HSD_LightAnim; 2]; 6] = [[null_mut(); 2]; 6]; static mut LL: [LightList; 6] = unsafe { core::mem::zeroed() };
static mut LLP: [[*mut LightList; 8]; 3] = [[null_mut(); 8]; 3];
static mut LO: [HSD_LObj; 4] = unsafe { core::mem::zeroed() }; static mut WO: [HSD_WObj; 4] = unsafe { core::mem::zeroed() };
static mut AO: [HSD_AObj; 4] = unsafe { core::mem::zeroed() };
static mut FOG: HSD_Fog = unsafe { core::mem::zeroed() }; static mut FD: HSD_FogDesc = unsafe { core::mem::zeroed() };
static mut FGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut NG: [HSD_GObj; 4] = unsafe { core::mem::zeroed() }; static mut NGR: [Ground; 4] = unsafe { core::mem::zeroed() };
static mut JN: [HSD_JObj; 8] = unsafe { core::mem::zeroed() }; static mut COB: HSD_CObj = unsafe { core::mem::zeroed() };
static mut CJ: [CollJoint; 16] = unsafe { core::mem::zeroed() }; static mut CLN: [CollLine; 24] = unsafe { core::mem::zeroed() };
static mut MCD: MapCollData = unsafe { core::mem::zeroed() }; static mut MCP: [MapCollData; 4] = unsafe { core::mem::zeroed() };
static mut MV: [Vec2; 8] = unsafe { core::mem::zeroed() }; static mut MLN: [MapLine; 8] = unsafe { core::mem::zeroed() };
static mut MJN: [MapJoint; 8] = unsafe { core::mem::zeroed() };
static mut ARENA: [u8; 4096] = [0; 4096]; static mut BUF: [u8; 64] = [0; 64]; static mut PL: [*mut HSD_GObj; 16] = [null_mut(); 16];
static mut PAIR: StageIdPair = unsafe { core::mem::zeroed() }; static mut GID: [GroundItemData; 3] = unsafe { core::mem::zeroed() };
static mut ITD: [*mut GroundItemData; 4] = [null_mut(); 4]; static mut AYA: [*mut c_void; 6] = [null_mut(); 6];
static mut ART: Article = unsafe { core::mem::zeroed() }; static mut ISA: ItemStateArray = unsafe { core::mem::zeroed() };
static mut TB: [*mut Article; 256] = [null_mut(); 256]; static mut DUM: [i32; 8] = [0; 8]; static mut DUMD: [u8; 16] = [0; 16];
static mut ARCHB: [u8; 16] = [0; 16];
#[repr(C)] #[derive(Clone, Copy)] struct LnT { n: *mut c_void, a: *mut c_void, f: *mut c_void }
static mut LNP: [LnT; 4] = [LnT { n: null_mut(), a: null_mut(), f: null_mut() }; 4];
static mut SCB: [[StageCallbacks; 8]; 3] = unsafe { core::mem::zeroed() };
static mut STRB: [[u8; 8]; 4] = [*b"GrNr\0\0\0\0", *b"GrCs\0\0\0\0", *b"GrPs\0\0\0\0", *b"GrZz\0\0\0\0"];
static mut YAK: [f32; 48] = [0.0; 48]; static mut OUTI: [i32; 8] = [0; 8]; static mut OUTF: [f32; 4] = [0.0; 4];
static mut I35: [i32; 40] = [0; 40]; static mut GXC: [_GXColor; 4] = unsafe { core::mem::zeroed() }; static mut OUTU: u32 = 0;
static mut SD0: [*mut StageData; 160] = [null_mut(); 160]; static mut ROWTAB: [*mut StageData; 160] = [null_mut(); 160];

unsafe extern "C" fn sd_oi() { t(9001); }
unsafe extern "C" fn sd_od(a: i32) { t(9002); t(a as u32); }
unsafe extern "C" fn sd_ol() { t(9003); }
unsafe extern "C" fn sd_os() { t(9004); }
unsafe extern "C" fn sd_c4() -> i32 { t(9005); (fv() & 1) as i32 }
unsafe extern "C" fn sd_tl(a: i32) -> *mut DynamicsDesc { t(9006); t(a as u32); if fv() & 1 != 0 { null_mut() } else { (&raw mut DUMD) as *mut DynamicsDesc } }
unsafe extern "C" fn sd_cs(p: *mut Vec, a: i32, j: *mut HSD_JObj) -> i32 { t(9007); tv(p); t(a as u32); t(pi(j)); (fv() & 1) as i32 }
unsafe extern "C" fn pr90(p: *mut Vec, i: i32) -> i32 { t(9008); tv(p); t(i as u32); (fv() & 1) as i32 }
unsafe extern "C" fn pr94(p: *mut Vec, i: i32) -> i32 { t(9009); tv(p); t(i as u32); (fv() & 1) as i32 }
unsafe extern "C" fn lnfn(a: i32) { t(9010); t(a as u32); }
unsafe extern "C" fn cbi(a: i32) { t(9011); t(a as u32); }
unsafe extern "C" fn ev1(g: *mut HSD_GObj) { t(9012); t(pi(g)); }
unsafe extern "C" fn ev2(g: *mut HSD_GObj) { t(9013); t(pi(g)); }
unsafe extern "C" fn ev3(g: *mut HSD_GObj) { t(9014); t(pi(g)); }

unsafe fn rowid(p: *const StageData) -> u32 {
    if p.is_null() { return 0; }
    for i in 0..160usize { if ROWTAB[i] as *const StageData == p { return (i + 1) as u32; } }
    0xFFFF
}
unsafe fn pick_stk() -> i32 {
    let cnt = GP.stage_param_count;
    let mut wild = 0;
    if cnt <= 0 { return (fv() % 8) as i32 - 1; }
    for i in 0..cnt as usize { if SP[i].stkind as i32 == -1 { wild = 1; } }
    let r = fv() % 6;
    if r == 0 && (wild != 0 || cnt == 1) { return (fv() % 8) as i32 - 1; }
    SP[(fv() % cnt as u32) as usize].stkind as i32
}
unsafe fn pick_stk_any() -> i32 {
    if fv() % 6 == 0 { return (fv() % 8) as i32 - 1; }
    pick_stk()
}
unsafe fn pick_kind() -> i32 {
    let mut k = (fv() % 160) as i32;
    if k == 23 || k == 26 { k = 1 + (fv() % 20) as i32; }
    k
}
unsafe fn pick_gk() -> i32 {
    const GK: [i32; 10] = [Gr_Kind_Castle as i32, Gr_Kind_Corneria as i32, Gr_Kind_Unk26 as i32, Gr_Kind_Inishie2 as i32, Gr_Kind_RCruise as i32,
                           Gr_Kind_Yorster as i32, Gr_Kind_MuteCity as i32, Gr_Kind_Shrine as i32, Gr_Kind_Kongo as i32, Gr_Kind_OldKongo as i32];
    let r = fv() % 14;
    if r < 10 { return GK[r as usize]; }
    pick_kind()
}
unsafe fn pick_idx() -> i32 {
    let r = fv() % 6;
    if r == 0 { return (fv() % 10) as i32; }
    if r == 1 { return 0x94 + (fv() % 5) as i32; }
    if r == 2 { return 0x99 + (fv() % 26) as i32; }
    if r == 3 { return 0xBD + (fv() % 10) as i32; }
    if r == 4 { return 0x7F; }
    (fv() % 261) as i32
}
unsafe fn setpair(nn: i32) {
    let mut k = (fv() % 160) as i32;
    if nn != 0 && (k == 23 || k == 26) { k = 1 + (fv() % 20) as i32; }
    if nn == 0 && fv() % 12 == 0 { k = if fv() & 1 != 0 { 23 } else { 26 }; }
    PAIR.grkind = k as _;
    PAIR.stkind = pick_stk() as _;
}
unsafe fn set_gsn(n: i32) { GSN = n; STD.unk24 = n; }
unsafe fn lights_pre() {
    let style = fv() % 3;
    if style == 0 { set_gsn(0); return; }
    set_gsn(5);
    let sd = m::stage_datas[m::stage_info.grkind as usize];
    if (*sd).callbacks.is_null() { (*sd).callbacks = (&raw mut SCB[0]) as *mut StageCallbacks; }
    (*(*sd).callbacks).anon_10.anon_0.set_flags_b0(1);
    STD.unkC = 1 + (fv() % 8) as i32;
}
unsafe fn pick_bgmflags() -> u32 {
    let mut f = 0u32;
    if fv() % 2 != 0 { f |= 4; }
    if fv() % 3 == 0 { f |= 0x40; }
    if fv() % 3 == 0 { f |= 0x10; } else if fv() % 3 == 0 { f |= 0x20; }
    if f & 4 == 0 { f |= if fv() % 2 != 0 { 1 } else { 2 }; }
    f
}
unsafe fn pick_snd() -> u32 {
    if fv() % 4 == 0 { return 540000; }
    if fv() % 3 == 0 { return 540001; }
    fv()
}
unsafe fn pick_gr() -> *mut Ground {
    if fv() % 8 == 0 { return null_mut(); }
    &raw mut GR[(fv() & 3) as usize]
}
unsafe fn alloc_(n: i32) -> *mut c_void {
    if AMODE >= 1 {
        let k = MI; MI += 1;
        if k & 3 == 0 { return (&raw mut MCP[((k >> 2) & 3) as usize]) as *mut c_void; }
        if AMODE == 2 && k & 3 == 2 { return null_mut(); }
    } else {
        if n == 64 { return (&raw mut BUF) as *mut c_void; }
        if n == core::mem::size_of::<Ground>() as i32 { if GFAIL == 2 { return null_mut(); } let k = NGRI; NGRI += 1; return (&raw mut NGR[(k & 3) as usize]) as *mut c_void; }
        if n == core::mem::size_of::<LnT>() as i32 { let k = LNI; LNI += 1; return (&raw mut LNP[(k & 3) as usize]) as *mut c_void; }
    }
    let p = (&raw mut ARENA as *mut u8).add(ARO as usize) as *mut c_void;
    ARO += (n + 7) & !7;
    p
}
unsafe fn setrow(d: *mut StageData) {
    (*d).grkind = (fv() % 160) as _;
    (*d).callbacks = if fv() % 6 == 0 { null_mut() } else { (&raw mut SCB[(fv() % 3) as usize]) as *mut StageCallbacks };
    (*d).data1 = (&raw mut STRB[(fv() & 3) as usize]) as *mut i8;
    (*d).on_init = Some(sd_oi); (*d).on_demo_init = Some(sd_od); (*d).on_load = Some(sd_ol); (*d).on_start = Some(sd_os); (*d).callback4 = Some(sd_c4);
    (*d).on_touch_line = if fv() & 3 == 0 { None } else { Some(sd_tl) };
    (*d).on_check_shadow_render = if fv() & 3 == 0 { None } else { Some(sd_cs) };
    (*d).flags2 = fv();
    (*d).joints = if fv() % 5 == 0 { null_mut() } else { (&raw mut GJB) as *mut GrJoint };
    (*d).joint_count = if (*d).joints.is_null() { 0 } else { (fv() % 7) as usize };
}
unsafe fn setup_world() {
    ANULL = 0; STDNULL = 0; GFAIL = 0; JLN = 0; GSN = 5; AMODE = 0; MI = 0; JI = 0; GI = 0; NGRI = 0; LNI = 0; WP = 1; WI = 1; ARO = 0;
    CKV = (fv() % 30) as i8;
    z!(SP, GP, ARCH, STD, X8, HJ, CAMD, GJA, GJB, SPA, ENT, PRS, SPL, SPLP, LOE, GSE, LD, LAN, ANL, LL, LLP, LO, WO, AO, FOG, FD, FGO, NG, NGR, JN,
       COB, CJ, CLN, MCD, MCP, MV, MLN, MJN, ARENA, BUF, PL, PAIR, GID, ITD, AYA, ART, ISA, TB, DUM, DUMD, LNP, SCB, YAK, OUTI, OUTF, I35, GXC);
    OUTU = 0;
    m::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::stage_info);

    for i in 0..16usize {
        JP[i].child = if 2 * i + 1 < 16 { &raw mut JP[2 * i + 1] } else { null_mut() };
        JP[i].next = if i & 1 != 0 && i + 1 < 16 { &raw mut JP[i + 1] } else { null_mut() };
        JP[i].parent = if i > 0 { &raw mut JP[(i - 1) / 2] } else { null_mut() };
        AO[i & 3].curr_frame = rg();
        JP[i].aobj = if fv() % 4 == 0 { &raw mut AO[(fv() & 3) as usize] } else { null_mut() };
    }
    for i in 0..4usize {
        GR[i].map_id = (fv() % 8) as _;
        GR[i].x8_callback = if fv() % 3 == 0 { None } else { Some(ev1) };
        GR[i].xC_callback = if fv() % 3 == 0 { None } else { Some(ev2) };
        GR[i].x1C_callback = if fv() % 3 == 0 { None } else { Some(ev3) };
        GR[i].x18 = if fv() % 2 != 0 { &raw mut NG[(fv() & 3) as usize] } else { null_mut() };
        for j in 0..8usize { GR[i].x20[j] = if fv() % 3 == 0 { -1 } else { (fv() % 20) as i32 }; }
        GO[i].next = if i < 2 { &raw mut GO[i + 1] } else { null_mut() };
        GO[i].classifier = if fv() % 3 == 0 { 12 } else if fv() % 3 == 0 { 13 } else { (fv() % 20) as u16 };
        GO[i].obj_kind = if fv() % 2 != 0 { 7 } else { (fv() % 10) as u8 };
    }
    m::ext::HSD_GObj_LightKind = 7;
    m::ext::HSD_GObjPLinkHead = if fv() % 6 == 0 { null_mut() } else { (&raw mut PL) as *mut *mut HSD_GObj };
    PL[3] = if fv() % 5 == 0 { null_mut() } else { &raw mut GO[0] };

    g::fill_GroundParam(&mut GP);
    GP.stage_params = (&raw mut SP) as *mut StageParam; GP.stage_param_count = 1 + (fv() % 4) as i32;
    if fv() % 3 == 0 { GP.y = 1.0; }
    for i in 0..4usize {
        SP[i].stkind = (if i == 0 && fv() % 4 == 0 { -1 } else { (fv() % 6) as i32 }) as _;
        { let r6 = fv() % 6; SP[i].x4 = if r6 == 0 { -2 } else if r6 == 1 { -1 } else { (fv() % 40) as i32 }; }
        SP[i].x8 = if fv() % 3 == 0 { -1 } else { (fv() % 40) as i32 };
        { let r6 = fv() % 6; SP[i].xC = if r6 == 0 { -2i32 as u32 } else if r6 == 1 { -1i32 as u32 } else { fv() % 40 }; }
        SP[i].x10 = if fv() % 3 == 0 { -1i32 as u32 } else { fv() % 40 };
        SP[i].x14 = (fv() % 12) as i16; SP[i].x16 = (fv() % 130) as i16; SP[i].x18 = (fv() % 200) as i16;
        for j in 0..37usize { SP[i].x1A[j] = ((fv() % 50) as i32 - 10) as i16; }
    }
    m::stage_info.param = &raw mut GP;

    ARCH.unk0 = (&raw mut ARCHB) as *mut HSD_Archive; ARCH.unk4 = &raw mut STD; ARCH.unk8 = fv();
    STD.unk0 = (&raw mut ENT) as *mut c_void; STD.unk4 = (fv() % 5) as i32; STD.unk8 = (&raw mut X8) as *mut UnkStageDat_x8_t; STD.unkC = 1 + (fv() % 8) as i32;
    STD.unk10 = (&raw mut SPLP) as *mut *mut HSD_Spline; STD.unk14 = (fv() % 5) as i32; STD.unk18 = (&raw mut LOE) as *mut c_void; STD.unk1C = (fv() % 7) as i32;
    STD.unk20 = (&raw mut GSE) as *mut GroundShadowEntry; STD.unk24 = GSN;
    for i in 0..8usize {
        HJ[i].child = if 2 * i + 1 < 8 { &raw mut HJ[2 * i + 1] } else { null_mut() };
        HJ[i].next = if i & 1 != 0 && i + 1 < 8 { &raw mut HJ[i + 1] } else { null_mut() };
        HJ[i].flags = fv();
        HJ[i].rotation.x = rg(); HJ[i].rotation.y = rg(); HJ[i].rotation.z = rg();
        HJ[i].scale.x = rg(); HJ[i].scale.y = rg(); HJ[i].scale.z = rg();
        HJ[i].position.x = rg(); HJ[i].position.y = rg(); HJ[i].position.z = rg();
        X8[i].unk0 = &raw mut HJ[i];
        X8[i].x10 = if fv() % 3 == 0 { null_mut() } else { &raw mut CAMD[(fv() & 1) as usize] };
        X8[i].x18 = (&raw mut LLP[(fv() % 3) as usize]) as *mut *mut LightList;
        X8[i].x1C = if fv() % 4 == 0 { null_mut() } else { &raw mut FD };
        X8[i].unk20 = (&raw mut GJA) as *mut GrJoint; X8[i].unk24 = (fv() % 5) as i32;
        X8[i].x2C = (&raw mut SPA) as *mut i16; X8[i].x30 = (fv() % 5) as i32;
        GJA[i].x = (fv() % 16) as i16; GJA[i].y = (fv() % 8) as i16; GJA[i].z = (fv() % 50) as i16;
        GJB[i].x = (fv() % 16) as i16; GJB[i].y = (fv() % 8) as i16; GJB[i].z = (fv() % 50) as i16;
        SPA[i] = ((fv() % 24) as i32 - 2) as i16;
    }
    for i in 0..4usize {
        ENT[i].joint = (&raw mut HJ[(fv() % 8) as usize]) as *mut c_void; ENT[i].pairs = (&raw mut PRS[i]) as *mut i16; ENT[i].pair_count = (fv() % 4) as i32;
        for j in 0..4usize {
            PRS[i][2 * j] = (fv() % 12) as i16;
            if fv() & 1 != 0 { PRS[i][2 * j + 1] = (220 + fv() % 40) as i16; } else { PRS[i][2 * j + 1] = (fv() % 261) as i16; }
        }
        SPLP[i] = &raw mut SPL[i];
        SPL[i].numcv = (fv() % 5) as i16;
    }
    for i in 0..6usize {
        LOE[i].desc = &raw mut LD[(fv() % 6) as usize]; LOE[i].set_a((fv() & 1) as u8); LOE[i].set_b((fv() & 1) as u8); LOE[i].set_c((fv() & 1) as u8);
        LD[i].flags = fv() as u16;
        LAN[i].next = null_mut();
        ANL[i][0] = &raw mut LAN[i]; ANL[i][1] = null_mut();
        LL[i].desc = &raw mut LD[i]; LL[i].anims = (&raw mut ANL[i]) as *mut *mut HSD_LightAnim;
    }
    LD[0].flags &= !3; LD[1].flags &= !3;
    for i in 0..5usize { GSE[i].unk0 = &raw mut LAN[i]; GSE[i].set_flag((fv() & 1) as u8); }
    LLP[0][0] = &raw mut LL[0]; LLP[0][1] = &raw mut LL[1]; LLP[0][2] = &raw mut LL[2]; LLP[0][3] = &raw mut LL[3]; LLP[0][4] = &raw mut LL[4]; LLP[0][5] = null_mut();
    LLP[1][0] = &raw mut LL[1]; LLP[1][1] = &raw mut LL[0]; LLP[1][2] = &raw mut LL[5]; LLP[1][3] = &raw mut LL[2]; LLP[1][4] = null_mut();
    LLP[2][0] = &raw mut LL[0]; LLP[2][1] = &raw mut LL[1]; LLP[2][2] = &raw mut LL[3]; LLP[2][3] = &raw mut LL[5]; LLP[2][4] = &raw mut LL[4]; LLP[2][5] = null_mut();
    for i in 0..4usize {
        LO[i].next = if i & 1 == 0 { &raw mut LO[i + 1] } else { null_mut() };
        LO[i].aobj = if fv() % 3 == 0 { &raw mut AO[(fv() & 3) as usize] } else { null_mut() };
        LO[i].position = if fv() % 3 == 0 { null_mut() } else { &raw mut WO[(fv() & 3) as usize] };
        LO[i].interest = if fv() % 3 == 0 { null_mut() } else { &raw mut WO[(fv() & 3) as usize] };
    }
    FOG.start = rg(); FOG.end = rg(); FOG.color.r = fv() as u8; FOG.color.g = fv() as u8; FOG.color.b = fv() as u8; FOG.color.a = fv() as u8;
    FD.start = rg(); FD.end = rg();
    FGO.hsd_obj = if fv() % 5 == 0 { null_mut() } else { (&raw mut FOG) as *mut c_void };
    m::stage_info.x12C = if fv() % 5 == 0 { null_mut() } else { &raw mut FGO };
    for i in 0..4usize { GXC[i].r = fv() as u8; GXC[i].g = fv() as u8; GXC[i].b = fv() as u8; GXC[i].a = fv() as u8; }
    for i in 0..40usize { I35[i] = fv() as i32; }
    for i in 0..8usize { OUTI[i] = fv() as i32; }
    for i in 0..4usize { OUTF[i] = rg(); }
    OUTU = fv();
    for i in 0..48usize { YAK[i] = rg(); }
    for i in 0..64usize { BUF[i] = (fv() & 1) as u8; }
    for i in 0..4usize { JN[i].child = if fv() & 1 != 0 { &raw mut JP[(1 + fv() % 15) as usize] } else { null_mut() }; }

    for i in 0..64usize { m::stage_info.map_gobjs[i] = if fv() % 3 == 0 { null_mut() } else { &raw mut GO[(fv() & 3) as usize] }; }
    for i in 0..261usize { m::stage_info.x280[i] = if fv() & 1 != 0 { &raw mut JP[(fv() % 16) as usize] } else { null_mut() }; }
    m::stage_info.x694[0] = if fv() & 1 != 0 { (&raw mut JP[(fv() % 16) as usize]) as *mut c_void } else { null_mut() };
    m::stage_info.x694[1] = if fv() & 1 != 0 { GO[(fv() & 3) as usize].hsd_obj } else { null_mut() };
    m::stage_info.x694[2] = if fv() & 1 != 0 { (&raw mut COB) as *mut c_void } else { null_mut() };
    m::stage_info.x694[3] = null_mut();
    for i in 0..3usize {
        LNP[i].n = if i < 2 { (&raw mut LNP[i + 1]) as *mut c_void } else { null_mut() };
        LNP[i].a = (fv() % 50) as usize as *mut c_void; LNP[i].f = lnfn as *mut c_void;
    }
    m::stage_info.x6A4 = if fv() % 3 == 0 { null_mut() } else { (&raw mut LNP[0]) as *mut c_void };
    for i in 0..3usize { GID[i].unk0 = (fv() % 30) as i32; GID[i].unk4 = &raw mut ART; ITD[i] = &raw mut GID[i]; }
    ITD[3] = null_mut();
    m::stage_info.itemdata = if fv() % 3 == 0 { null_mut() } else { (&raw mut ITD) as *mut *mut GroundItemData };
    AYA[0] = null_mut(); AYA[1] = (&raw mut DUM[0]) as *mut c_void; AYA[2] = (&raw mut DUM[1]) as *mut c_void;
    AYA[3] = if fv() & 1 != 0 { (&raw mut DUM[2]) as *mut c_void } else { null_mut() }; AYA[4] = (&raw mut DUM[3]) as *mut c_void; AYA[5] = null_mut();
    m::stage_info.ald_yaku_all = if fv() % 3 == 0 { null_mut() } else { (&raw mut AYA) as *mut *mut *mut c_void };
    ART.xC_itemStates = &raw mut ISA;
    for i in 0..256usize { TB[i] = &raw mut ART; }
    m::ext::it_804D6D38 = (&raw mut TB) as *mut *mut Article;
    m::stage_info.coll_data = if fv() % 4 == 0 { null_mut() } else { &raw mut MCD };
    m::stage_info.map_ptcl = if fv() & 1 != 0 { (&raw mut DUM[4]) as *mut c_void } else { null_mut() };
    m::stage_info.map_texg = if fv() & 1 != 0 { (&raw mut DUM[5]) as *mut c_void } else { null_mut() };
    m::stage_info.yakumono_param = if fv() & 1 != 0 { (&raw mut YAK) as *mut c_void } else { null_mut() };
    m::stage_info.map_plit = if fv() % 3 == 0 { null_mut() } else { (&raw mut LLP[(fv() % 3) as usize]) as *mut *mut LightList };
    m::stage_info.x90 = if fv() % 3 == 0 { None } else { Some(pr90) }; m::stage_info.x94 = if fv() % 3 == 0 { None } else { Some(pr94) };
    m::stage_info.on_touch_line = if fv() % 3 == 0 { None } else { Some(sd_tl) };
    m::stage_info.on_check_shadow_render = if fv() % 3 == 0 { None } else { Some(sd_cs) };
    for i in 0..3usize { for j in 0..8usize {
        SCB[i][j].anon_10.anon_0.set_flags_b0((fv() % 4 == 0) as u8); SCB[i][j].anon_10.anon_0.set_flags_b1((fv() % 4 == 0) as u8); SCB[i][j].anon_10.anon_0.set_flags_b2((fv() % 2 == 0) as u8);
    } }
    { let vv: [*mut Vec; 6] = [&raw mut m::stage_info.x130, &raw mut m::stage_info.x13C, &raw mut m::stage_info.x148, &raw mut m::stage_info.x154, &raw mut m::stage_info.x160, &raw mut m::stage_info.x16C];
      for i in 0..6usize { let k2 = if fv() % 2 == 0 { 30.0f32 } else { 1.0f32 }; (*vv[i]).x *= k2; (*vv[i]).y *= k2; (*vv[i]).z *= k2; } }
    { let kk = if fv() % 3 == 0 { 100.0f32 } else { 1.0f32 }; m::stage_info.x70C = rg() * kk; m::stage_info.x710 = rg() * kk; m::stage_info.x718 = rg() * kk; m::stage_info.x71C = rg() * kk; }
    if fv() % 3 == 0 { m::stage_info.x714 = -1; }
    if fv() % 3 == 0 { m::stage_info.x720 = -1; }
    m::stage_info.grkind = pick_kind() as _;
    for i in 0..8usize { MV[i].x = rg(); MV[i].y = rg(); }
    MCD.verts = (&raw mut MV) as *mut Vec2; MCD.vert_count = (fv() % 7) as i32; MCD.lines = (&raw mut MLN) as *mut MapLine; MCD.line_count = (fv() % 7) as i32;
    MCD.joints = (&raw mut MJN) as *mut MapJoint; MCD.joint_count = (fv() % 7) as i32; MCD.x2C = fv() as i32;
    for i in 0..5usize { MCD.ranges[i].start = (fv() % 100) as i16; MCD.ranges[i].count = (fv() % 100) as i16; }
    for i in 0..8usize {
        MLN[i].v0_idx = fv() as u16; MLN[i].v1_idx = fv() as u16; MLN[i].prev_id0 = fv() as i16; MLN[i].next_id0 = fv() as i16;
        MLN[i].prev_id1 = fv() as i16; MLN[i].next_id1 = fv() as i16; MLN[i].hi_flags = fv() as u16; MLN[i].lo_flags = fv() as u16;
        MJN[i].left_bound = rg(); MJN[i].bottom_bound = rg(); MJN[i].right_bound = rg(); MJN[i].top_bound = rg();
        MJN[i].vtx_start = fv() as i16; MJN[i].vtx_count = fv() as i16;
        for k in 0..5usize { MJN[i].ranges[k].start = fv() as i16; MJN[i].ranges[k].count = fv() as i16; }
    }
    for i in 0..16usize { CJ[i].xC = fv() as i16; }
    m::ext::mpColl_804D64AC = (fv() % 100) as i32;
    m::Ground_804D6950 = (&raw mut BUF) as *mut u8; m::Ground_804D6954 = (fv() % 100) as i16;

    if SD0N == 0 { for i in 0..160usize { SD0[i] = m::stage_datas[i]; } SD0N = 1; }
    for i in 0..160usize { m::stage_datas[i] = SD0[i]; }
    m::Ground_MexStageDatas = core::mem::zeroed();
    m::Ground_MexInitStages_done = false;
    for i in 0..160usize { ROWTAB[i] = null_mut(); }
    ROWTAB[0] = &raw mut m::Ground_StageData;
@ROWSETUP@
    for i in 0..89usize { ROWTAB[71 + i] = &raw mut m::Ground_MexStageDatas[i]; }
    for i in 1..68usize { setrow(ROWTAB[i]); }
    for i in 71..160usize { setrow(ROWTAB[i]); }
}
"""

ROWSETUP_R = "\n".join(f"    ROWTAB[{i + 1}] = &raw mut m::ext::{n};" for i, n in enumerate(ROWNAMES))
DECL_R = DECL_R.replace("@ROWSETUP@", ROWSETUP_R)
