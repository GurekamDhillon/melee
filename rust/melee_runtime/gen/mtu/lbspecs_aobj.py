"""aobj.c spec (see lbspecs.py): HSD_ForeachAnim over a small acyclic object graph, the alloc / flags / frame functions."""
from lbspecs import add

POOLS = [("HSD_AObj", "AO", 4), ("HSD_FObj", "FO", 2), ("HSD_AObjDesc", "AD", 2), ("HSD_JObj", "JO", 3), ("HSD_DObj", "DO", 3), ("HSD_MObj", "MO", 2),
         ("HSD_TObj", "TO", 3), ("HSD_PObj", "PO", 2), ("HSD_RObj", "RO", 2), ("HSD_WObj", "WO", 2), ("HSD_CObj", "CO", 2), ("HSD_LObj", "LO", 3),
         ("HSD_Fog", "FG", 2), ("HSD_ShapeSet", "SS", 2), ("HSD_SList", "SL", 3), ("HSD_FObjDesc", "FD", 2)]
OWN = ["aobj_alloc_data", "endcallback_list", "HSD_AObj_804D762C", "HSD_AObj_804D7630"]
A = "(fv() & 3) ? &AO[fv() % 4] : 0"
AR = "if fv() & 3 != 0 { &raw mut AO[(fv() % 4) as usize] } else { null_mut() }"

_c, _r = [], []
for pool, n in [("JO", 3), ("DO", 3), ("MO", 2), ("TO", 3), ("RO", 2), ("WO", 2), ("CO", 2), ("LO", 3), ("FG", 2)]:
    _c.append(f"for (q = 0; q < {n}; q++) {{ {pool}[q].aobj = {A}; }}")
    _r.append(f"for q in 0..{n}usize {{ {pool}[q].aobj = {AR}; }}")
_SETUP_C = (
    "{ int q; AOI = 0; " + " ".join(_c) + " "
    "JO[0].child = &JO[1]; JO[0].next = 0; JO[1].child = 0; JO[1].next = &JO[2]; JO[2].child = 0; JO[2].next = 0; "
    "for (q = 0; q < 3; q++) { JO[q].u.dobj = (fv() & 3) ? &DO[fv() % 3] : 0; JO[q].robj = (fv() & 3) ? &RO[fv() & 1] : 0; } "
    "for (q = 0; q < 3; q++) { DO[q].next = q < 2 ? &DO[q + 1] : 0; DO[q].mobj = (fv() & 3) ? &MO[fv() & 1] : 0; DO[q].pobj = (fv() & 3) ? &PO[0] : 0; } "
    "for (q = 0; q < 2; q++) MO[q].tobj = (fv() & 3) ? &TO[0] : 0; for (q = 0; q < 3; q++) TO[q].next = q < 2 ? &TO[q + 1] : 0; "
    "PO[0].next = &PO[1]; PO[1].next = 0; for (q = 0; q < 2; q++) PO[q].u.shape_set = (fv() & 3) ? &SS[fv() & 1] : 0; RO[0].next = &RO[1]; RO[1].next = 0; for (q = 0; q < 2; q++) WO[q].robj = (fv() & 3) ? &RO[0] : 0; "
    "for (q = 0; q < 3; q++) LO[q].next = q < 2 ? &LO[q + 1] : 0; "
    "SL[0].next = (fv() & 1) ? &SL[1] : 0; SL[1].next = 0; SL[2].next = 0; SL[0].data = (void*) cbv0; SL[1].data = (void*) cbv1; endcallback_list = (fv() & 7) ? &SL[0] : 0; "
    "HSD_AObj_804D762C = fv() & 1; HSD_AObj_804D7630 = (fv() % 3) == 0; }")
_SETUP_R = (
    "{ AOI = 0; " + " ".join(_r) + " "
    "JO[0].child = &raw mut JO[1]; JO[0].next = null_mut(); JO[1].child = null_mut(); JO[1].next = &raw mut JO[2]; JO[2].child = null_mut(); JO[2].next = null_mut(); "
    "for q in 0..3usize { JO[q].u.dobj = if fv() & 3 != 0 { &raw mut DO[(fv() % 3) as usize] } else { null_mut() }; JO[q].robj = if fv() & 3 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; } "
    "for q in 0..3usize { DO[q].next = if q < 2 { &raw mut DO[q + 1] } else { null_mut() }; DO[q].mobj = if fv() & 3 != 0 { &raw mut MO[(fv() & 1) as usize] } else { null_mut() }; "
    "DO[q].pobj = if fv() & 3 != 0 { &raw mut PO[0] } else { null_mut() }; } "
    "for q in 0..2usize { MO[q].tobj = if fv() & 3 != 0 { &raw mut TO[0] } else { null_mut() }; } for q in 0..3usize { TO[q].next = if q < 2 { &raw mut TO[q + 1] } else { null_mut() }; } "
    "PO[0].next = &raw mut PO[1]; PO[1].next = null_mut(); for q in 0..2usize { PO[q].u.shape_set = if fv() & 3 != 0 { &raw mut SS[(fv() & 1) as usize] } else { null_mut() }; } RO[0].next = &raw mut RO[1]; RO[1].next = null_mut(); for q in 0..2usize { WO[q].robj = if fv() & 3 != 0 { &raw mut RO[0] } else { null_mut() }; } "
    "for q in 0..3usize { LO[q].next = if q < 2 { &raw mut LO[q + 1] } else { null_mut() }; } "
    "SL[0].next = if fv() & 1 != 0 { &raw mut SL[1] } else { null_mut() }; SL[1].next = null_mut(); SL[2].next = null_mut(); SL[0].data = cbv0 as usize as *mut c_void; SL[1].data = cbv1 as usize as *mut c_void; "
    "m::endcallback_list = if fv() & 7 != 0 { &raw mut SL[0] } else { null_mut() }; "
    "m::HSD_AObj_804D762C = (fv() & 1) as i32; m::HSD_AObj_804D7630 = (fv() % 3 == 0) as i32; }")

C_DECL = r'''
static int AOI;
static void cbv0(void) { T(8100); }
static void cbv1(void) { T(8101); }
static void cb_A(HSD_AObj* a) { T(8000); T(PI(a)); }
static void cb_AF(HSD_AObj* a, float f) { T(8001); T(PI(a)); T(B(f)); }
static void cb_AV(HSD_AObj* a, void* v) { T(8002); T(PI(a)); T(PI(v)); }
static void cb_AU(HSD_AObj* a, u32 d) { T(8003); T(PI(a)); T(d); }
static void cb_AO(HSD_AObj* a, void* o) { T(8004); T(PI(a)); T(PI(o)); }
static void cb_AOF(HSD_AObj* a, void* o, float f) { T(8005); T(PI(a)); T(PI(o)); T(B(f)); }
static void cb_AOV(HSD_AObj* a, void* o, void* v) { T(8006); T(PI(a)); T(PI(o)); T(PI(v)); }
static void cb_AOU(HSD_AObj* a, void* o, u32 d) { T(8007); T(PI(a)); T(PI(o)); T(d); }
static void cb_AOT(HSD_AObj* a, void* o, int t) { T(8008); T(PI(a)); T(PI(o)); T(t); }
static void cb_AOTF(HSD_AObj* a, void* o, int t, float f) { T(8009); T(PI(a)); T(PI(o)); T(t); T(B(f)); }
static void cb_AOTV(HSD_AObj* a, void* o, int t, void* v) { T(8010); T(PI(a)); T(PI(o)); T(t); T(PI(v)); }
static void cb_AOTU(HSD_AObj* a, void* o, int t, u32 d) { T(8011); T(PI(a)); T(PI(o)); T(t); T(d); }
static void* FN[12] = { cb_A, cb_AF, cb_AV, cb_AU, cb_AO, cb_AOF, cb_AOV, cb_AOU, cb_AOT, cb_AOTF, cb_AOTV, cb_AOTU };
static const int TY[10] = { 6, 3, 8, 9, 11, 7, 2, 10, 12, 5 };
static void* objfor(int type)
{
    if ((fv() & 7) == 0) return 0;
    switch (type) {
    case 6: return &JO[0]; case 3: return &DO[0]; case 8: return &MO[0]; case 9: return &PO[0]; case 11: return &TO[0];
    case 7: return &LO[0]; case 2: return &CO[0]; case 10: return &RO[0]; case 12: return &WO[0]; default: return &FG[0];
    }
}
'''
RS_EXTRA = r'''
static mut AOI: usize = 0;
unsafe extern "C" fn cbv0() { t(8100); }
unsafe extern "C" fn cbv1() { t(8101); }
unsafe extern "C" fn cb_A(a: *mut HSD_AObj) { t(8000); t(pi(a)); }
unsafe extern "C" fn cb_AF(a: *mut HSD_AObj, f: f32) { t(8001); t(pi(a)); t(f.to_bits()); }
unsafe extern "C" fn cb_AV(a: *mut HSD_AObj, v: *mut c_void) { t(8002); t(pi(a)); t(pi(v)); }
unsafe extern "C" fn cb_AU(a: *mut HSD_AObj, d: u32) { t(8003); t(pi(a)); t(d); }
unsafe extern "C" fn cb_AO(a: *mut HSD_AObj, o: *mut c_void) { t(8004); t(pi(a)); t(pi(o)); }
unsafe extern "C" fn cb_AOF(a: *mut HSD_AObj, o: *mut c_void, f: f32) { t(8005); t(pi(a)); t(pi(o)); t(f.to_bits()); }
unsafe extern "C" fn cb_AOV(a: *mut HSD_AObj, o: *mut c_void, v: *mut c_void) { t(8006); t(pi(a)); t(pi(o)); t(pi(v)); }
unsafe extern "C" fn cb_AOU(a: *mut HSD_AObj, o: *mut c_void, d: u32) { t(8007); t(pi(a)); t(pi(o)); t(d); }
unsafe extern "C" fn cb_AOT(a: *mut HSD_AObj, o: *mut c_void, ty: i32) { t(8008); t(pi(a)); t(pi(o)); t(ty as u32); }
unsafe extern "C" fn cb_AOTF(a: *mut HSD_AObj, o: *mut c_void, ty: i32, f: f32) { t(8009); t(pi(a)); t(pi(o)); t(ty as u32); t(f.to_bits()); }
unsafe extern "C" fn cb_AOTV(a: *mut HSD_AObj, o: *mut c_void, ty: i32, v: *mut c_void) { t(8010); t(pi(a)); t(pi(o)); t(ty as u32); t(pi(v)); }
unsafe extern "C" fn cb_AOTU(a: *mut HSD_AObj, o: *mut c_void, ty: i32, d: u32) { t(8011); t(pi(a)); t(pi(o)); t(ty as u32); t(d); }
fn fnat(i: usize) -> usize { [cb_A as usize, cb_AF as usize, cb_AV as usize, cb_AU as usize, cb_AO as usize, cb_AOF as usize, cb_AOV as usize, cb_AOU as usize, cb_AOT as usize, cb_AOTF as usize, cb_AOTV as usize, cb_AOTU as usize][i] }
const TY: [i32; 10] = [6, 3, 8, 9, 11, 7, 2, 10, 12, 5];
unsafe fn objfor(ty: i32) -> *mut c_void {
    if fv() & 7 == 0 { return null_mut(); }
    match ty { 6 => &raw mut JO[0] as *mut c_void, 3 => &raw mut DO[0] as *mut c_void, 8 => &raw mut MO[0] as *mut c_void, 9 => &raw mut PO[0] as *mut c_void,
        11 => &raw mut TO[0] as *mut c_void, 7 => &raw mut LO[0] as *mut c_void, 2 => &raw mut CO[0] as *mut c_void, 10 => &raw mut RO[0] as *mut c_void,
        12 => &raw mut WO[0] as *mut c_void, _ => &raw mut FG[0] as *mut c_void }
}
'''
FOR_C = ("{ int ty = TY[fv() % 10]; int at = (int) (fv() % 14); void* ob = objfor(ty); u32 mk = (fv() & 1) ? 0x7FFFu : (fv() & 0x7FFFu); void* fn = FN[at > 11 ? 0 : at]; "
         "switch (at) { case 1: case 5: case 9: HSD_ForeachAnim(ob, ty, mk, fn, at, (double) rw()); break; case 2: case 6: case 10: HSD_ForeachAnim(ob, ty, mk, fn, at, (void*) BUF); break; "
         "case 3: case 7: case 11: HSD_ForeachAnim(ob, ty, mk, fn, at, (u32) fv()); break; default: HSD_ForeachAnim(ob, ty, mk, fn, at); } }")
FOR_R = ("{ let ty = TY[(fv() % 10) as usize]; let at = (fv() % 14) as i32; let ob = objfor(ty); let mk = if fv() & 1 != 0 { 0x7FFFu32 } else { fv() & 0x7FFF }; "
         "let f = fnat(if at > 11 { 0 } else { at as usize }) as *mut c_void; "
         "let arg = match at { 1 | 5 | 9 => m::CallbackArg { f: (rw() as f64) as f32 }, 2 | 6 | 10 => m::CallbackArg { v: (&raw mut BUF) as *mut c_void }, 3 | 7 | 11 => m::CallbackArg { d: fv() }, _ => m::CallbackArg { d: 0 } }; "
         "m::HSD_ForeachAnim(ob, ty as _, mk as _, f, at as _, arg); }")
add("aobj", "src/sysdolphin/baselib/aobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    pregs=("freg((const void*) cbv0); freg((const void*) cbv1); { int i; for (i = 0; i < 12; i++) freg(FN[i]); } preg(&AOI, 4);",
           "freg(cbv0 as usize); freg(cbv1 as usize); for i in 0..12 { freg(fnat(i)); } melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_ForeachAnim": {0: ("0", "null_mut::<c_void>()"), 1: ("0", "0i32"), 2: ("0", "0u32"), 3: ("0", "null_mut::<c_void>()"), 4: ("0", "0i32")}},
    stmt={"HSD_ForeachAnim": (FOR_C, FOR_R)}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=OWN[1:], own_hash=OWN,
    skip_stubs=["memset", "fmodf"],
    ret={"HSD_ObjAlloc": ("((fv() & 7) ? (void*) &AO[AOI++ % 4] : 0)", "if fv() & 7 != 0 { AOI += 1; (&raw mut AO[(AOI - 1) % 4]) as *mut c_void } else { null_mut() }"),
         "HSD_FObjLoadDesc": ("picko_HSD_FObj()", "picko_HSD_FObj()"), "HSD_JObjLoadJoint": ("picko_HSD_JObj()", "picko_HSD_JObj()"),
         "HSD_IDGetDataFromTable": ("(void*) picko_HSD_JObj()", "picko_HSD_JObj() as *mut c_void")})
