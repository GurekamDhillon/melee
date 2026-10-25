"""tobj.c spec (see lbspecs.py)."""
from lbspecs import add

POOLS = [("HSD_TObj", "TOB", 6), ("HSD_TexAnim", "TA", 2), ("HSD_AObj", "AOB", 2), ("HSD_AObjDesc", "AD", 2), ("HSD_ImageDesc", "IMG", 3),
         ("HSD_Tlut", "TL", 3), ("HSD_TlutDesc", "TLD", 3), ("HSD_TObjTev", "TEV", 2), ("HSD_TObjTevDesc", "TEVD", 2), ("HSD_TObjDesc", "TDS", 2),
         ("HSD_TexLODDesc", "LOD", 1), ("HSD_TObjInfo", "TINF", 1), ("HSD_ClassInfo", "CIN", 2), ("HSD_LObj", "LOB", 1), ("HSD_CObj", "COB", 1),
         ("HSD_ObjData", "OBD", 2)]
OWN = ["hsdTObj", "default_class", "tobj_head"]
C_DECL = r'''
static HSD_ImageDesc* IMT[3][4];
static HSD_Tlut* TLT[3][4];
static HSD_TlutDesc* TLDT[3][4];
static unsigned char HP[4096];
static unsigned char MA[1024];
static unsigned char TXA[4][64];
static int HPI, MAI; static float VF[4] = { 0.0f, 1.0f, 2.0f, 0.5f };
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static const unsigned char FMT[12] = { 0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 14, 7 };
'''
RS_EXTRA = r'''
static mut IMT: [[*mut HSD_ImageDesc; 4]; 3] = [[null_mut(); 4]; 3];
static mut TLT: [[*mut HSD_Tlut; 4]; 3] = [[null_mut(); 4]; 3];
static mut TLDT: [[*mut HSD_TlutDesc; 4]; 3] = [[null_mut(); 4]; 3];
static mut HP: [u8; 4096] = [0; 4096];
static mut MA: [u8; 1024] = [0; 1024];
static mut TXA: [[u8; 64]; 4] = [[0; 64]; 4];
static mut HPI: usize = 0;
static mut MAI: usize = 0; static mut VF: [f32; 4] = [0.0, 1.0, 2.0, 0.5];
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
const FMT: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 14, 7];
'''
_SETUP_C = (
    "{ int q, j; HPI = 0; MAI = 0; for (q = 0; q < 3; q++) for (j = 0; j < 4; j++) { IMT[q][j] = (j < 3) ? &IMG[(j + q) % 3] : 0; TLT[q][j] = (j < 1 + (int) (fv() % 3)) ? &TL[(j + q) % 3] : 0; TLDT[q][j] = (j < 3) ? &TLD[(j + q) % 3] : 0; } "
    "for (q = 0; q < 3; q++) TLT[q][3] = 0; "
    "for (q = 0; q < 6; q++) { TOB[q].next = (q < 3) ? &TOB[q + 1] : 0; if (q == 3) TOB[q].next = 0; TOB[q].parent.parent.class_info = (HSD_ClassInfo*) &TINF[0]; TOB[q].id = (GXTexMapID) (fv() % 9); TOB[q].coord = (GXTexCoordID) (fv() % 9); "
    "TOB[q].repeat_s = (fv() % 16) ? 1 + fv() % 4 : 0; TOB[q].repeat_t = (fv() % 16) ? 1 + fv() % 4 : 0; TOB[q].wrap_s = (GXTexWrapMode) (fv() % 3); TOB[q].wrap_t = (GXTexWrapMode) (fv() % 3); "
    "TOB[q].imagedesc = &IMG[fv() % 3]; TOB[q].lod = (q < 2 || (fv() & 1)) ? &LOD[0] : 0; TOB[q].tlut = (fv() & 3) ? &TL[fv() % 3] : 0; TOB[q].tluttbl = TLT[fv() % 3]; TOB[q].imagetbl = IMT[fv() % 3]; "
    "TOB[q].tlut_no = (fv() & 1) ? -1 : (s32) (fv() % 4); TOB[q].tev = (q < 2 || (fv() & 3)) ? &TEV[fv() & 1] : 0; TOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; } "
    "for (q = 0; q < 3; q++) { IMG[q].format = (GXTexFmt) FMT[fv() % 12]; IMG[q].image_ptr = (fv() & 7) ? (void*) BUF : 0; IMG[q].mipmap = fv() & 1; IMG[q].width = 1 + fv() % 64; IMG[q].height = 1 + fv() % 64; "
    "TL[q].lut = BUF; TL[q].n_entries = fv() % 300; TL[q].fmt = (GXTlutFmt) (fv() % 3); } "
    "for (q = 0; q < 2; q++) { TA[q].next = q < 1 ? &TA[1] : 0; TA[q].id = (GXTexMapID) (fv() % 9); TA[q].aobjdesc = (fv() & 3) ? &AD[fv() & 1] : 0; TA[q].imagetbl = IMT[fv() % 3]; TA[q].tluttbl = TLDT[fv() % 3]; TA[q].n_tluttbl = fv() % 3; } "
    "for (q = 0; q < 2; q++) { TDS[q].next = q < 1 ? &TDS[1] : 0; TDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; TDS[q].imagedesc = &IMG[fv() % 3]; TDS[q].tlutdesc = (fv() & 1) ? &TLD[fv() % 3] : 0; TDS[q].tev = (fv() & 1) ? &TEVD[fv() & 1] : 0; TDS[q].lod = (fv() & 1) ? &LOD[0] : 0; } "
    "TINF[0].make_mtx = MakeTextureMtx; TINF[0].load = TObjLoad; TINF[0].make_texp = TObjMakeTExp; TINF[0].parent.release = (void*) cb_rel; TINF[0].parent.destroy = (void*) cb_dest; "
    "hsdTObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; "
    "default_class = (fv() & 1) ? &TINF[0] : 0; tobj_head = (fv() & 1) ? &TOB[fv() % 4] : 0; }")
_SETUP_R = (
    "{ HPI = 0; MAI = 0; for q in 0..3usize { for j in 0..4usize { IMT[q][j] = if j < 3 { &raw mut IMG[(j + q) % 3] } else { null_mut() }; "
    "TLT[q][j] = if (j as u32) < 1 + fv() % 3 { &raw mut TL[(j + q) % 3] } else { null_mut() }; TLDT[q][j] = if j < 3 { &raw mut TLD[(j + q) % 3] } else { null_mut() }; } } "
    "for q in 0..3usize { TLT[q][3] = null_mut(); } "
    "for q in 0..6usize { TOB[q].next = if q < 3 { &raw mut TOB[q + 1] } else { null_mut() }; TOB[q].parent.parent.class_info = (&raw mut TINF[0]) as *mut HSD_ClassInfo; "
    "TOB[q].id = (fv() % 9) as _; TOB[q].coord = (fv() % 9) as _; TOB[q].repeat_s = if fv() % 16 != 0 { (1 + fv() % 4) as u8 } else { 0 }; TOB[q].repeat_t = if fv() % 16 != 0 { (1 + fv() % 4) as u8 } else { 0 }; "
    "TOB[q].wrap_s = (fv() % 3) as _; TOB[q].wrap_t = (fv() % 3) as _; TOB[q].imagedesc = &raw mut IMG[(fv() % 3) as usize]; TOB[q].lod = if q < 2 || (fv() & 1 != 0) { &raw mut LOD[0] } else { null_mut() }; "
    "TOB[q].tlut = if fv() & 3 != 0 { &raw mut TL[(fv() % 3) as usize] } else { null_mut() }; TOB[q].tluttbl = (&raw mut TLT[(fv() % 3) as usize]) as *mut *mut HSD_Tlut; TOB[q].imagetbl = (&raw mut IMT[(fv() % 3) as usize]) as *mut *mut HSD_ImageDesc; "
    "TOB[q].tlut_no = if fv() & 1 != 0 { -1 } else { (fv() % 4) as _ }; TOB[q].tev = if q < 2 || (fv() & 3 != 0) { &raw mut TEV[(fv() & 1) as usize] } else { null_mut() }; "
    "TOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; } "
    "for q in 0..3usize { IMG[q].format = FMT[(fv() % 12) as usize] as _; IMG[q].image_ptr = if fv() & 7 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }; IMG[q].mipmap = fv() & 1; "
    "IMG[q].width = (1 + fv() % 64) as u16; IMG[q].height = (1 + fv() % 64) as u16; TL[q].lut = (&raw mut BUF) as *mut c_void; TL[q].n_entries = (fv() % 300) as u16; TL[q].fmt = (fv() % 3) as _; } "
    "for q in 0..2usize { TA[q].next = if q < 1 { &raw mut TA[1] } else { null_mut() }; TA[q].id = (fv() % 9) as _; TA[q].aobjdesc = if fv() & 3 != 0 { &raw mut AD[(fv() & 1) as usize] } else { null_mut() }; "
    "TA[q].imagetbl = (&raw mut IMT[(fv() % 3) as usize]) as *mut *mut HSD_ImageDesc; TA[q].tluttbl = (&raw mut TLDT[(fv() % 3) as usize]) as *mut *mut HSD_TlutDesc; TA[q].n_tluttbl = (fv() % 3) as u16; } "
    "for q in 0..2usize { TDS[q].next = if q < 1 { &raw mut TDS[1] } else { null_mut() }; TDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; "
    "TDS[q].imagedesc = &raw mut IMG[(fv() % 3) as usize]; TDS[q].tlutdesc = if fv() & 1 != 0 { &raw mut TLD[(fv() % 3) as usize] } else { null_mut() }; "
    "TDS[q].tev = if fv() & 1 != 0 { &raw mut TEVD[(fv() & 1) as usize] } else { null_mut() }; TDS[q].lod = if fv() & 1 != 0 { &raw mut LOD[0] } else { null_mut() }; } "
    "TINF[0].make_mtx = Some(m::MakeTextureMtx); TINF[0].load = Some(m::TObjLoad); TINF[0].make_texp = Some(core::mem::transmute::<usize, _>(m::TObjMakeTExp as usize)); "
    "TINF[0].parent.release = Some(cb_rel); TINF[0].parent.destroy = Some(cb_dest); "
    "m::hsdTObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); "
    "m::default_class = if fv() & 1 != 0 { &raw mut TINF[0] } else { null_mut() }; m::tobj_head = if fv() & 1 != 0 { &raw mut TOB[(fv() % 4) as usize] } else { null_mut() }; }")
add("tobj", "src/sysdolphin/baselib/tobj.c", dict(
    pub_only=True, c_extra="float fabsf(float x) { union { float f; u32 u; } c; c.f = x; c.u &= 0x7FFFFFFFu; return c.f; }", pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=250, c_decl=C_DECL, rs_extra=RS_EXTRA,
    pregs=("preg(IMT, sizeof IMT); preg(TLT, sizeof TLT); preg(TLDT, sizeof TLDT); preg(HP, sizeof HP); preg(MA, sizeof MA); preg(TXA, sizeof TXA); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn);",
           "preg(&raw const IMT, 96); preg(&raw const TLT, 96); preg(&raw const TLDT, 96); preg(&raw const HP, 4096); preg(&raw const MA, 1024); preg(&raw const TXA, 256); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"TObjRelease": {0: ("(HSD_Class*) pick_HSD_TObj()", "pick_HSD_TObj() as *mut HSD_Class")}, "MakeColorGenTExp": {1: ("&TOB[fv() % 2]", "&raw mut TOB[(fv() % 2) as usize]")}, "TObjUpdateFunc": {0: ("(void*) &TOB[fv() % 2]", "(&raw mut TOB[(fv() % 2) as usize]) as *mut c_void"), 1: ("(int) (fv() % 24)", "(fv() % 24) as i32"), 2: ("(HSD_ObjData*) &VF[fv() % 4]", "(&raw mut VF[(fv() % 4) as usize]) as *mut HSD_ObjData")},
        "HSD_Index2TexMap": {0: ("fv() % 10", "fv() % 10")}, "HSD_Index2TexCoord": {0: ("fv() % 10", "fv() % 10")}, "HSD_Index2TexMtx": {0: ("fv() % 12", "fv() % 12")},
        "HSD_TObjReqAnimByFlags": {2: ("fv() | 1", "fv() | 1")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset", "fabsf"],
    effects={"HSD_SetupTevStage": ("T(RH(a0, sizeof *a0));", "t(rh(&*a0));")},
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_TObj() : 0)", "if fv() & 7 != 0 { pick_HSD_TObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
         "hsdAllocMemPiece": ("(void*) (HP + 32 * (HPI++ % 120))", "{ HPI += 1; (&raw mut HP as *mut u8).add(32 * ((HPI - 1) % 120)) as *mut c_void }"),
         "HSD_MemAlloc": ("(void*) (MA + 8 * (MAI++ % 100))", "{ MAI += 1; (&raw mut MA as *mut u8).add(8 * ((MAI - 1) % 100)) as *mut c_void }"),
         "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()"),
         "HSD_LObjGetCurrentByType": ("picko_HSD_LObj()", "picko_HSD_LObj()"), "HSD_CObjGetCurrent": ("picko_HSD_CObj()", "picko_HSD_CObj()"),
         "HSD_CObjGetViewingMtxPtrDirect": ("(float (*)[4]) BUF", "(&raw mut BUF) as *mut c_void"),
         "HSD_CObjGetInvViewingMtxPtrDirect": ("(float (*)[4]) (BUF + 64)", "(&raw mut BUF as *mut u8).add(64) as *mut c_void"),
         "HSD_TExpTev": ("(HSD_TExp*) TXA[fv() & 3]", "(&raw mut TXA[(fv() & 3) as usize]) as *mut HSD_TExp"),
         "HSD_TExpCnst": ("(HSD_TExp*) TXA[fv() & 3]", "(&raw mut TXA[(fv() & 3) as usize]) as *mut HSD_TExp")})
