"""jobj.c spec (see lbspecs.py): the raw-pointer entry points of tu/jobj.rs that do not go through the arena adapters
(class machinery, load / resolve refs, ref counting, current jobj). The matrix / flag / tree / animation functions run on the
arena model of crate::jobj and are checked by oracle_jobj / oracle_ik / oracle_shard_jobj_anim."""
from lbspecs import add

POOLS = [("HSD_JObj", "J", 4), ("HSD_Joint", "JT", 3), ("HSD_JObjInfo", "JI", 3), ("HSD_ClassInfo", "CI", 3), ("HSD_DObj", "DO", 2),
         ("HSD_DObjDesc", "DD", 2), ("HSD_RObj", "RO", 2), ("HSD_RObjDesc", "RD", 2), ("HSD_AObj", "AO", 2), ("HSD_SList", "SL", 3),
         ("HSD_Spline", "SPL", 1), ("HSD_Class", "CL", 2)]
OWN = ["hsdJObj", "default_class", "ufc_callbacks", "current_jobj"]
C_DECL = r'''
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static int cb_init(HSD_Class* c) { T(8206); T(PI(c)); return (int) (fv() % 3) - 1; }
static int cb_load(HSD_JObj* j, HSD_Joint* d, HSD_JObj* p) { T(8207); T(PI(j)); T(PI(d)); T(PI(p)); return (int) (fv() % 3) - 1; }
static void cb_relchild(HSD_JObj* j) { T(8208); T(PI(j)); }
static unsigned char MT[4][48];
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_init(c: *mut HSD_Class) -> i32 { t(8206); t(pi(c)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_load(j: *mut HSD_JObj, d: *mut HSD_Joint, p: *mut HSD_JObj) -> i32 { t(8207); t(pi(j)); t(pi(d)); t(pi(p)); (fv() % 3) as i32 - 1 }
unsafe extern "C" fn cb_relchild(j: *mut HSD_JObj) { t(8208); t(pi(j)); }
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];
'''
_SETUP_C = r'''{ int q;
 for (q = 0; q < 3; q++) { HSD_ClassInfo* ci = (HSD_ClassInfo*) &JI[q]; ci->release = (void*) cb_rel; ci->destroy = (void*) cb_dest; ci->amnesia = cb_amn; ci->init = cb_init; JI[q].load = cb_load; JI[q].release_child = cb_relchild; }
 for (q = 0; q < 3; q++) { CI[q].release = (void*) cb_rel; CI[q].destroy = (void*) cb_dest; CI[q].amnesia = cb_amn; CI[q].init = cb_init; }
 for (q = 0; q < 4; q++) { J[q].object.parent.class_info = (HSD_ClassInfo*) &JI[q % 3]; J[q].next = (q < 3 && (fv() & 1)) ? &J[q + 1] : 0; J[q].child = (q < 3 && (fv() & 1)) ? &J[q + 1 + (fv() % (3 - q))] : 0;
   J[q].parent = 0; J[q].flags = fv() & 0xFFFFF; J[q].robj = (fv() & 1) ? &RO[fv() & 1] : 0; J[q].aobj = 0; J[q].scl = (fv() & 1) ? (Vec*) &VA[fv() % 4] : 0; J[q].envelopemtx = (fv() & 1) ? (MtxPtr) MT[q] : 0;
   J[q].object.ref_count = (fv() & 3) ? (u16) (fv() % 3) : (u16) 65535; J[q].object.ref_count_individual = (u16) (fv() % 3);
   if (J[q].flags & 0x4020) { J[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; } else { J[q].u.dobj = (fv() & 1) ? &DO[fv() & 1] : 0; } }
 for (q = 0; q < 3; q++) SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0;
 for (q = 0; q < 3; q++) { JT[q].class_name = (fv() & 3) ? 0 : STRS[fv() % 4]; JT[q].flags = fv() & 0xFFFFF; JT[q].child = (q < 2 && (fv() & 1)) ? &JT[q + 1 + (fv() % (2 - q))] : 0; JT[q].next = (q < 2 && (fv() & 1)) ? &JT[q + 1] : 0;
   JT[q].mtx = (fv() & 1) ? (MtxPtr) MT[q] : 0; JT[q].robjdesc = (fv() & 1) ? &RD[fv() & 1] : 0;
   if (JT[q].flags & 0x4020) { if (JT[q].flags & 0x4000) JT[q].u.spline = &SPL[0]; else JT[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; } else JT[q].u.dobjdesc = (fv() & 1) ? &DD[fv() & 1] : 0; }
 for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); }
 hsdJObj.parent.parent.head.parent = &CI[0]; hsdJObj.parent.parent.init = cb_init; hsdJObj.parent.parent.release = (void*) cb_rel; hsdJObj.parent.parent.amnesia = cb_amn;
 default_class = (fv() & 1) ? &CI[fv() % 3] : 0; ufc_callbacks = (fv() & 1) ? &SL[fv() % 3] : 0; current_jobj = (fv() & 1) ? &J[fv() % 4] : 0; }'''.replace("\n", " ")
_SETUP_R = r'''{
 for q in 0..3usize { let ci = (&raw mut JI[q]) as *mut HSD_ClassInfo; (*ci).release = Some(cb_rel); (*ci).destroy = Some(cb_dest); (*ci).amnesia = Some(cb_amn); (*ci).init = Some(cb_init); JI[q].load = Some(cb_load); JI[q].release_child = Some(cb_relchild); }
 for q in 0..3usize { CI[q].release = Some(cb_rel); CI[q].destroy = Some(cb_dest); CI[q].amnesia = Some(cb_amn); CI[q].init = Some(cb_init); }
 for q in 0..4usize { J[q].object.parent.class_info = (&raw mut JI[q % 3]) as *mut HSD_ClassInfo; J[q].next = if q < 3 && fv() & 1 != 0 { &raw mut J[q + 1] } else { null_mut() };
   J[q].child = if q < 3 && fv() & 1 != 0 { let k = q + 1 + (fv() % (3 - q as u32)) as usize; &raw mut J[k] } else { null_mut() };
   J[q].parent = null_mut(); J[q].flags = fv() & 0xFFFFF; J[q].robj = if fv() & 1 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; J[q].aobj = null_mut(); J[q].scl = if fv() & 1 != 0 { (&raw mut VA[(fv() % 4) as usize]) as *mut Vec } else { null_mut() };
   J[q].envelopemtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() };
   J[q].object.ref_count = if fv() & 3 != 0 { (fv() % 3) as u16 } else { 65535 }; J[q].object.ref_count_individual = (fv() % 3) as u16;
   if J[q].flags & 0x4020 != 0 { J[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } else { J[q].u.dobj = if fv() & 1 != 0 { &raw mut DO[(fv() & 1) as usize] } else { null_mut() }; } }
 for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; }
 for q in 0..3usize { JT[q].class_name = if fv() & 3 != 0 { null_mut() } else { (&raw mut STRS[(fv() % 4) as usize]) as *mut i8 }; JT[q].flags = fv() & 0xFFFFF;
   JT[q].child = if q < 2 && fv() & 1 != 0 { let k = q + 1 + (fv() % (2 - q as u32)) as usize; &raw mut JT[k] } else { null_mut() }; JT[q].next = if q < 2 && fv() & 1 != 0 { &raw mut JT[q + 1] } else { null_mut() };
   JT[q].mtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() }; JT[q].robjdesc = if fv() & 1 != 0 { &raw mut RD[(fv() & 1) as usize] } else { null_mut() };
   if JT[q].flags & 0x4020 != 0 { if JT[q].flags & 0x4000 != 0 { JT[q].u.spline = &raw mut SPL[0]; } else { JT[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } } else { JT[q].u.dobjdesc = if fv() & 1 != 0 { &raw mut DD[(fv() & 1) as usize] } else { null_mut() }; } }
 for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } }
 m::hsdJObj.parent.parent.head.parent = &raw mut CI[0]; m::hsdJObj.parent.parent.init = Some(cb_init); m::hsdJObj.parent.parent.release = Some(cb_rel); m::hsdJObj.parent.parent.amnesia = Some(cb_amn);
 m::default_class = if fv() & 1 != 0 { &raw mut CI[(fv() % 3) as usize] } else { null_mut() }; m::ufc_callbacks = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; m::current_jobj = if fv() & 1 != 0 { &raw mut J[(fv() % 4) as usize] } else { null_mut() }; }'''.replace("\n", " ")
ONLY = ["HSD_JObjSetDefaultClass", "HSD_JObjAlloc", "JObjLoadJointSub", "JObjLoad", "HSD_JObjLoadJoint", "HSD_JObjResolveRefs", "HSD_JObjResolveRefsAll",
        "HSD_JObjUnref", "HSD_JObjUnrefThis", "HSD_JObjSetCurrent", "HSD_JObjGetCurrent", "JObjInit", "JObjRelease", "JObjAmnesia"]
add("jobj", "src/sysdolphin/baselib/jobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    only=ONLY,
    ov={"JObjInit": {0: ("(HSD_Class*) pick_HSD_JObj()", "pick_HSD_JObj() as *mut HSD_Class")}, "JObjRelease": {0: ("(HSD_Class*) pick_HSD_JObj()", "pick_HSD_JObj() as *mut HSD_Class")}},
    pregs=("preg(MT, sizeof MT); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_init); freg((const void*) cb_load); freg((const void*) cb_relchild);",
           "preg(&raw const MT, 192); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_init as usize); freg(cb_load as usize); freg(cb_relchild as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN, own_only=ONLY,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte", "sinf", "cosf", "atan2f", "tanf"],
    ptr64_args={"HSD_IDGetDataFromTable": [1], "HSD_IDInsertToTable": [1], "HSD_IDRemoveByIDFromTable": [1]},
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_JObj() : 0)", "if fv() & 7 != 0 { pick_HSD_JObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("(HSD_ClassInfo*) picko_HSD_JObjInfo()", "picko_HSD_JObjInfo() as *mut _HSD_ClassInfo"),
         "HSD_IDGetDataFromTable": ("(void*) picko_HSD_JObj()", "picko_HSD_JObj() as *mut c_void"),
         "HSD_DObjLoadDesc": ("picko_HSD_DObj()", "picko_HSD_DObj()"), "HSD_RObjLoadDesc": ("picko_HSD_RObj()", "picko_HSD_RObj()"),
         "HSD_MtxAlloc": ("(void*) MT[rnd() % 4]", "(&raw mut MT[(rnd() % 4) as usize]) as *mut c_void")})
