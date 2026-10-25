"""cobj.c spec (see lbspecs.py): the raw-pointer half in tu/cobj.rs; the accessors of tu/cobj_world.rs run inside the C-defined callers."""
from lbspecs import add

POOLS = [("HSD_CObj", "COB", 3), ("HSD_WObj", "WOB", 4), ("HSD_AObj", "AOB", 2), ("HSD_CObjDesc", "CDS", 2), ("HSD_CObjInfo", "CINF", 2),
         ("HSD_ClassInfo", "CIN", 2), ("HSD_CameraAnim", "CAN", 2), ("HSD_WObjDesc", "WDS", 2), ("HSD_WObjAnim", "WAN", 2), ("HSD_AObjDesc", "ADS", 2),
         ("GXRenderModeObj", "RMO", 1), ("HSD_ObjData", "OBD", 2), ("Scissor", "SCS", 1), ("HSD_RectS16", "RS16", 1)]
OWN = ["hsdCObj", "default_class", "current"]
C_DECL = r'''
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static int cb_init(HSD_Class* c) { T(8206); T(PI(c)); return (int) (fv() % 3) - 1; }
static unsigned char MT[2][64];
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8206); t(pi(c)); (fv() % 3) as i32 - 1 }
static mut MT: [[u8; 64]; 2] = [[0; 64]; 2];
'''
_SETUP_C = (
    "{ int q; for (q = 0; q < 3; q++) { COB[q].parent.parent.class_info = (HSD_ClassInfo*) &CINF[q & 1]; COB[q].eyepos = &WOB[fv() % 4]; COB[q].interest = &WOB[fv() % 4]; "
    "COB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; COB[q].proj_mtx = (fv() & 3) ? (Mtx*) MT[q & 1] : 0; COB[q].projection_type = (u8) (1 + fv() % 3); } "
    "for (q = 0; q < 2; q++) { CINF[q].load = CObjLoad; CINF[q].parent.parent.init = cb_init; CINF[q].parent.parent.release = (void*) cb_rel; CINF[q].parent.parent.destroy = (void*) cb_dest; } "
    "for (q = 0; q < 2; q++) { CDS[q].class_name = (fv() & 1) ? (char*) STRS[0] : 0; CDS[q].common.eyepos = &WDS[0]; CDS[q].common.interest = &WDS[1]; CDS[q].common.up_vector = (fv() & 1) ? (Vec3*) &VA[fv() % 4] : 0; } "
    "hsdCObj.parent.parent.head.parent = &CIN[0]; CIN[0].init = cb_init; CIN[0].release = (void*) cb_rel; CIN[0].destroy = (void*) cb_dest; CIN[0].amnesia = (void*) cb_amn; for (q = 0; q < 4; q++) WOB[q].parent.parent.class_info = &CIN[0]; "
    "default_class = (fv() & 1) ? &CIN[1] : 0; current = (fv() & 1) ? &COB[fv() % 3] : 0; }")
_SETUP_R = (
    "{ for q in 0..3usize { COB[q].parent.parent.class_info = (&raw mut CINF[q & 1]) as *mut HSD_ClassInfo; COB[q].eyepos = &raw mut WOB[(fv() % 4) as usize]; "
    "COB[q].interest = &raw mut WOB[(fv() % 4) as usize]; COB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; "
    "COB[q].proj_mtx = if fv() & 3 != 0 { (&raw mut MT[q & 1]) as *mut Mtx } else { null_mut() }; COB[q].projection_type = (1 + fv() % 3) as u8; } "
    "for q in 0..2usize { CINF[q].load = Some(m::CObjLoad); CINF[q].parent.parent.init = Some(cb_init); CINF[q].parent.parent.release = Some(cb_rel); CINF[q].parent.parent.destroy = Some(cb_dest); } "
    "for q in 0..2usize { CDS[q].class_name = if fv() & 1 != 0 { (&raw mut STRS[0]) as *mut i8 } else { null_mut() }; CDS[q].common.eyepos = &raw mut WDS[0]; CDS[q].common.interest = &raw mut WDS[1]; CDS[q].common.up_vector = if fv() & 1 != 0 { &raw mut VA[(fv() % 4) as usize] } else { null_mut() }; } "
    "m::hsdCObj.parent.parent.head.parent = &raw mut CIN[0]; CIN[0].init = Some(cb_init); CIN[0].release = Some(cb_rel); CIN[0].destroy = Some(cb_dest); CIN[0].amnesia = Some(cb_amn); for q in 0..4usize { WOB[q].parent.parent.class_info = &raw mut CIN[0]; } "
    "m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current = if fv() & 1 != 0 { &raw mut COB[(fv() % 3) as usize] } else { null_mut() }; }")
add("cobj", "src/sysdolphin/baselib/cobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    pregs=("preg(MT, sizeof MT); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_init);",
           "preg(&raw const MT, 128); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_init as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    call={"makeProjectionMtx": ("makeProjectionMtx(a0, a1)", "m::makeProjectionMtx(a0, &mut *a1)")},
    ov={"makeProjectionMtx": {1: ("(void*) BUF", "(&raw mut BUF) as *mut [[f32; 4]; 4]")}, "CObjRelease": {0: ("(HSD_Class*) pick_HSD_CObj()", "pick_HSD_CObj() as *mut HSD_Class")}, "CObjInit": {0: ("(HSD_Class*) pick_HSD_CObj()", "pick_HSD_CObj() as *mut HSD_Class")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset", "tanf", "__frsqrte"],
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_CObj() : 0)", "if fv() & 7 != 0 { pick_HSD_CObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
         "HSD_WObjAlloc": ("pick_HSD_WObj()", "pick_HSD_WObj()"), "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()"),
         "HSD_VIGetRenderMode": ("&RMO[0]", "&raw mut RMO[0]")})
