"""mobj.c spec (see lbspecs.py): the raw-pointer half (`tu::mobj::native`)."""
from lbspecs import add

POOLS = [("HSD_MObj", "MOB", 3), ("HSD_TObj", "TOB", 6), ("HSD_Material", "MAT", 2), ("HSD_PEDesc", "PED", 2), ("HSD_AObj", "AOB", 2),
         ("HSD_MatAnim", "MAN", 2), ("HSD_MObjDesc", "MDS", 2), ("HSD_ClassInfo", "CIN", 2), ("HSD_MObjInfo", "MINF", 2), ("HSD_TObjInfo", "TINF", 1),
         ("HSD_ImageDesc", "IMD", 2), ("HSD_TObjDesc", "TDS", 2), ("HSD_PEDesc", "PE2", 1)]
POOLS = [p for p in POOLS if p[1] != "PE2"]
OWN = ["hsdMObj", "default_class", "current_mobj", "tobj_toon", "tobj_shadows"]
C_DECL = r'''
static unsigned char TXA[4][64];
static HSD_TExp* TXP[3];
static unsigned char HP[2048];
static int HPI;
static void cb_setup(HSD_MObj* m, u32 r) { T(8200); T(PI(m)); T(r); }
static void cb_unset(HSD_MObj* m, u32 r) { T(8201); T(PI(m)); T(r); }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static void cb_tmake(HSD_TObj* t, u32 a, u32 b, HSD_TExp** x, HSD_TExp** y, HSD_TExp** l) { T(8300); T(PI(t)); T(a); T(b); T(PI(x)); T(PI(y)); T(PI(l)); }
'''
RS_EXTRA = r'''
static mut TXA: [[u8; 64]; 4] = [[0; 64]; 4];
static mut TXP: [*mut HSD_TExp; 3] = [null_mut(); 3];
static mut HP: [u8; 2048] = [0; 2048];
static mut HPI: usize = 0;
unsafe extern "C" fn cb_setup(m: *mut HSD_MObj, r: u32) { t(8200); t(pi(m)); t(r); }
unsafe extern "C" fn cb_unset(m: *mut HSD_MObj, r: u32) { t(8201); t(pi(m)); t(r); }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_tmake(tb: *mut HSD_TObj, a: u32, b: u32, x: *mut *mut HSD_TExp, y: *mut *mut HSD_TExp, l: *mut *mut HSD_TExp) { t(8300); t(pi(tb)); t(a); t(b); t(pi(x)); t(pi(y)); t(pi(l)); }
'''
_SETUP_C = (
    "{ int q; HPI = 0; for (q = 0; q < 3; q++) { MOB[q].parent.class_info = (HSD_ClassInfo*) &MINF[q & 1]; MOB[q].tobj = (fv() & 3) ? &TOB[fv() & 1] : 0; MOB[q].mat = &MAT[fv() & 1]; "
    "MOB[q].pe = (fv() & 1) ? &PED[fv() & 1] : 0; MOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; MOB[q].tevdesc = (fv() & 1) ? (struct _HSD_TExpTevDesc*) TXA[3] : 0; MOB[q].texp = (fv() & 1) ? (HSD_TExp*) TXA[2] : 0; } "
    "for (q = 0; q < 6; q++) { TOB[q].next = (q == 0 || q == 1 || q == 3) ? &TOB[q + 1] : 0; TOB[q].parent.parent.class_info = (HSD_ClassInfo*) &TINF[0]; } "
    "for (q = 0; q < 2; q++) { MINF[q].load = MObjLoad; MINF[q].make_texp = MObjMakeTExp; MINF[q].setup_tev = MObjSetupTev; MINF[q].setup = (void*) cb_setup; MINF[q].unset = (void*) cb_unset; "
    "MINF[q].parent.release = (void*) cb_rel; MINF[q].parent.destroy = (void*) cb_dest; } "
    "TINF[0].make_texp = (void*) cb_tmake; "
    "for (q = 0; q < 2; q++) { MDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; MDS[q].mat = &MAT[0]; MDS[q].pedesc = (fv() & 1) ? &PED[0] : 0; } "
    "hsdMObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; default_class = (fv() & 1) ? &CIN[1] : 0; current_mobj = 0; "
    "tobj_toon = (fv() & 1) ? &TOB[5] : 0; tobj_shadows = (fv() & 1) ? &TOB[3] : 0; "
    "for (q = 0; q < 3; q++) TXP[q] = 0; }")
_SETUP_R = (
    "{ HPI = 0; for q in 0..3usize { MOB[q].parent.class_info = (&raw mut MINF[q & 1]) as *mut HSD_ClassInfo; MOB[q].tobj = if fv() & 3 != 0 { &raw mut TOB[(fv() & 1) as usize] } else { null_mut() }; "
    "MOB[q].mat = &raw mut MAT[(fv() & 1) as usize]; MOB[q].pe = if fv() & 1 != 0 { &raw mut PED[(fv() & 1) as usize] } else { null_mut() }; "
    "MOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; "
    "MOB[q].tevdesc = if fv() & 1 != 0 { (&raw mut TXA[3]) as *mut _ } else { null_mut() }; MOB[q].texp = if fv() & 1 != 0 { (&raw mut TXA[2]) as *mut HSD_TExp } else { null_mut() }; } "
    "for q in 0..6usize { TOB[q].next = if q == 0 || q == 1 || q == 3 { &raw mut TOB[q + 1] } else { null_mut() }; TOB[q].parent.parent.class_info = (&raw mut TINF[0]) as *mut HSD_ClassInfo; } "
    "for q in 0..2usize { MINF[q].load = Some(m::MObjLoad); MINF[q].make_texp = Some(m::MObjMakeTExp); MINF[q].setup_tev = Some(m::MObjSetupTev); "
    "MINF[q].setup = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_MObj, u32)>(cb_setup as usize)); "
    "MINF[q].unset = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_MObj, u32)>(cb_unset as usize)); MINF[q].parent.release = Some(cb_rel); MINF[q].parent.destroy = Some(cb_dest); } "
    "TINF[0].make_texp = Some(core::mem::transmute::<usize, _>(cb_tmake as usize)); "
    "for q in 0..2usize { MDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; MDS[q].mat = &raw mut MAT[0]; MDS[q].pedesc = if fv() & 1 != 0 { &raw mut PED[0] } else { null_mut() }; } "
    "m::hsdMObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current_mobj = null_mut(); "
    "m::tobj_toon = if fv() & 1 != 0 { &raw mut TOB[5] } else { null_mut() }; m::tobj_shadows = if fv() & 1 != 0 { &raw mut TOB[3] } else { null_mut() }; "
    "for q in 0..3usize { TXP[q] = null_mut(); } }")
add("mobj", "src/sysdolphin/baselib/mobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA, rmod="melee_runtime::tu::mobj::native",
    pregs=("preg(TXA, sizeof TXA); preg(TXP, sizeof TXP); preg(HP, sizeof HP); freg((const void*) cb_setup); freg((const void*) cb_unset); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_tmake);",
           "preg(&raw const TXA, 256); preg(&raw const TXP, core::mem::size_of_val(&TXP)); preg(&raw const HP, 2048); freg(cb_setup as usize); freg(cb_unset as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_tmake as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"MObjUpdateFunc": {0: ("(void*) picko_HSD_MObj()", "picko_HSD_MObj() as *mut c_void"), 1: ("(int) (fv() % 20)", "(fv() % 20) as i32")},
        "MObjMakeTExp": {2: ("&TXP[fv() % 3]", "&raw mut TXP[(fv() % 3) as usize]")},
        "HSD_MObjSetToonTextureImage": {0: ("&IMD[fv() & 1]", "&raw mut IMD[(fv() & 1) as usize]")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset"],
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_MObj() : 0)", "if fv() & 7 != 0 { pick_HSD_MObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
         "hsdAllocMemPiece": ("(void*) (HP + 32 * (HPI++ % 60))", "{ HPI += 1; (&raw mut HP as *mut u8).add(32 * ((HPI - 1) % 60)) as *mut c_void }"),
         "HSD_TObjLoadDesc": ("((fv() & 3) ? &TOB[fv() % 3] : 0)", "if fv() & 3 != 0 { &raw mut TOB[(fv() % 3) as usize] } else { null_mut() }"), "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()"),
         "HSD_TExpTev": ("(HSD_TExp*) TXA[fv() & 1]", "(&raw mut TXA[(fv() & 1) as usize]) as *mut HSD_TExp"),
         "HSD_TExpCnst": ("(HSD_TExp*) TXA[fv() & 1]", "(&raw mut TXA[(fv() & 1) as usize]) as *mut HSD_TExp")})
