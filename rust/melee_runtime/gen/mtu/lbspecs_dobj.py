"""dobj.c spec (see lbspecs.py): the raw-pointer half (`tu::dobj::hsd`)."""
from lbspecs import add

POOLS = [("HSD_DObj", "DOB", 3), ("HSD_MObj", "MOB", 2), ("HSD_PObj", "POB", 2), ("HSD_AObj", "AOB", 2), ("HSD_MatAnim", "MA", 2),
         ("HSD_ShapeAnimDObj", "SAD", 2), ("HSD_ShapeAnim", "SHA", 2), ("HSD_DObjDesc", "DD", 2), ("HSD_MObjDesc", "MD", 2),
         ("HSD_PObjDesc", "PD", 2), ("HSD_ClassInfo", "CIN", 2), ("HSD_DObjInfo", "DINF", 2), ("HSD_MObjInfo", "MINF", 1), ("HSD_PObjInfo", "PINF", 1)]
OWN = ["hsdDObj", "default_class", "current_dobj"]
C_DECL = r'''
static void cb_setup(HSD_MObj* m, u32 r) { T(8200); T(PI(m)); T(r); }
static void cb_unset(HSD_MObj* m, u32 r) { T(8201); T(PI(m)); T(r); }
static void cb_disp(HSD_PObj* p, void* v, void* pm, u32 r) { T(8202); T(PI(p)); T(PI(v)); T(PI(pm)); T(r); }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_setup(m: *mut HSD_MObj, r: u32) { t(8200); t(pi(m)); t(r); }
unsafe extern "C" fn cb_unset(m: *mut HSD_MObj, r: u32) { t(8201); t(pi(m)); t(r); }
unsafe extern "C" fn cb_disp(p: *mut HSD_PObj, v: *mut c_void, pm: *mut c_void, r: u32) { t(8202); t(pi(p)); t(pi(v)); t(pi(pm)); t(r); }
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
'''
_SETUP_C = (
    "{ int q; for (q = 0; q < 3; q++) { DOB[q].next = q < 2 ? &DOB[q + 1] : 0; DOB[q].mobj = &MOB[fv() & 1]; DOB[q].pobj = (fv() & 7) ? &POB[0] : 0; "
    "DOB[q].aobj = (fv() & 3) ? &AOB[fv() & 1] : 0; DOB[q].parent.class_info = (HSD_ClassInfo*) &DINF[q & 1]; } "
    "POB[0].next = &POB[1]; POB[1].next = 0; for (q = 0; q < 2; q++) { POB[q].parent.class_info = (HSD_ClassInfo*) &PINF[0]; MOB[q].parent.class_info = (HSD_ClassInfo*) &MINF[0]; } "
    "for (q = 0; q < 2; q++) { DINF[q].load = DObjLoad; DINF[q].parent.release = (void*) cb_rel; DINF[q].parent.destroy = (void*) cb_dest; } "
    "MINF[0].setup = (void*) cb_setup; MINF[0].unset = (void*) cb_unset; PINF[0].disp = (void*) cb_disp; "
    "DD[0].next = &DD[1]; DD[1].next = 0; "
    "hsdDObj.parent.head.parent = &CIN[0]; CIN[0].release = (void*) cb_rel; CIN[0].amnesia = (void*) cb_amn; default_class = (fv() & 1) ? &CIN[1] : 0; current_dobj = 0; }")
_SETUP_R = (
    "{ for q in 0..3usize { DOB[q].next = if q < 2 { &raw mut DOB[q + 1] } else { null_mut() }; DOB[q].mobj = &raw mut MOB[(fv() & 1) as usize]; "
    "DOB[q].pobj = if fv() & 7 != 0 { &raw mut POB[0] } else { null_mut() }; DOB[q].aobj = if fv() & 3 != 0 { &raw mut AOB[(fv() & 1) as usize] } else { null_mut() }; "
    "DOB[q].parent.class_info = (&raw mut DINF[q & 1]) as *mut HSD_ClassInfo; } "
    "POB[0].next = &raw mut POB[1]; POB[1].next = null_mut(); for q in 0..2usize { POB[q].parent.class_info = (&raw mut PINF[0]) as *mut HSD_ClassInfo; MOB[q].parent.class_info = (&raw mut MINF[0]) as *mut HSD_ClassInfo; } "
    "for q in 0..2usize { DINF[q].load = Some(m::DObjLoad); DINF[q].parent.release = Some(cb_rel); DINF[q].parent.destroy = Some(cb_dest); } "
    "MINF[0].setup = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_MObj, u32)>(cb_setup as usize)); "
    "MINF[0].unset = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_MObj, u32)>(cb_unset as usize)); "
    "PINF[0].disp = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_PObj, *mut [f32; 4], *mut [f32; 4], u32)>(cb_disp as usize)); "
    "DD[0].next = &raw mut DD[1]; DD[1].next = null_mut(); "
    "m::hsdDObj.parent.head.parent = &raw mut CIN[0]; CIN[0].release = Some(cb_rel); CIN[0].amnesia = Some(cb_amn); m::default_class = if fv() & 1 != 0 { &raw mut CIN[1] } else { null_mut() }; m::current_dobj = null_mut(); }")
MT = {1: ("(void*) BUF", "(&raw mut BUF) as *mut [f32; 4]"), 2: ("(void*) (BUF + 64)", "(&raw mut BUF as *mut u8).add(64) as *mut [f32; 4]")}
add("dobj", "src/sysdolphin/baselib/dobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA, rmod="melee_runtime::tu::dobj::hsd",
    pregs=("freg((const void*) cb_setup); freg((const void*) cb_unset); freg((const void*) cb_disp); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn);",
           "freg(cb_setup as usize); freg(cb_unset as usize); freg(cb_disp as usize); freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_DObjDisp": MT, "DObjRelease": {0: ("(HSD_Class*) pick_HSD_DObj()", "pick_HSD_DObj() as *mut HSD_Class")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_DObj() : 0)", "if fv() & 7 != 0 { pick_HSD_DObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
         "HSD_MObjLoadDesc": ("picko_HSD_MObj()", "picko_HSD_MObj()"), "HSD_PObjLoadDesc": ("picko_HSD_PObj()", "picko_HSD_PObj()")})
