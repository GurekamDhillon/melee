"""displayfunc.c spec (see lbspecs.py): JObj trees with traced make_pmtx / disp methods, DObj chains with a traced disp, the z-sorted
lists (links, counts, bottoms), the billboard matrices through the real C math calls (stubbed and traced)."""
from lbspecs import add

POOLS = [("HSD_JObj", "J", 4), ("HSD_JObjInfo", "JI", 3), ("HSD_DObj", "DO", 3), ("HSD_DObjInfo", "DI", 2), ("HSD_CObj", "CO", 2),
         ("HSD_SList", "SL", 3), ("HSD_ZList", "ZL", 4), ("HSD_ClassInfo", "CI", 2)]
OWN = ["zlist_alloc_data", "sptcl_callback", "erase_color", "zsort_listing", "zsort_sorting", "zlist_top", "zlist_bottom", "zlist_texedge_top",
       "zlist_texedge_bottom", "zlist_texedge_nb", "zlist_xlu_top", "zlist_xlu_bottom", "zlist_xlu_nb", "zOne", "yOne", "zOne2"]
C_DECL = r'''
static unsigned char MT[4][48];
static void cb_pmtx(HSD_JObj* j, Mtx v, Mtx m) { T(8210); T(PI(j)); T(PI(v)); T(PI(m)); }
static void cb_jdisp(HSD_JObj* j, Mtx v, Mtx m, u32 trsp, u32 r) { T(8211); T(PI(j)); T(PI(v)); T(PI(m)); T(trsp); T(r); }
static void cb_ddisp(HSD_DObj* d, Mtx v, Mtx m, u32 r) { T(8212); T(PI(d)); T(PI(v)); T(PI(m)); T(r); }
static void cb_sp(s32 a, s32 b, s32 c, HSD_JObj* j) { T(8213); T((u32) a); T((u32) b); T((u32) c); T(PI(j)); }
static unsigned ZK;
'''
RS_EXTRA = r'''
use m::HSD_ZList;
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];
unsafe extern "C" fn cb_pmtx(j: *mut HSD_JObj, v: *mut [f32; 4], m: *mut [f32; 4]) { t(8210); t(pi(j)); t(pi(v)); t(pi(m)); }
unsafe extern "C" fn cb_jdisp(j: *mut HSD_JObj, v: *mut [f32; 4], m: *mut [f32; 4], trsp: u32, r: u32) { t(8211); t(pi(j)); t(pi(v)); t(pi(m)); t(trsp); t(r); }
unsafe extern "C" fn cb_ddisp(d: *mut HSD_DObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8212); t(pi(d)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_sp(a: i32, b: i32, c: i32, j: *mut HSD_JObj) { t(8213); t(a as u32); t(b as u32); t(c as u32); t(pi(j)); }
static mut ZK: u32 = 0;
'''
_SETUP_C = r'''{ int q, n, k, ord[4];
 for (q = 0; q < 3; q++) { JI[q].make_pmtx = cb_pmtx; JI[q].disp = cb_jdisp; }
 for (q = 0; q < 2; q++) DI[q].disp = cb_ddisp;
 for (q = 0; q < 3; q++) SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0;
 for (q = 0; q < 3; q++) SL[q].data = (void*) (unsigned long long) ((fv() & 1) ? (fv() | 0x80000000u) : fv());
 for (q = 0; q < 3; q++) { DO[q].next = (q < 2 && (fv() & 1)) ? &DO[q + 1] : 0; DO[q].parent.class_info = (HSD_ClassInfo*) &DI[q & 1]; DO[q].flags = fv() & 0xFF; }
 for (q = 0; q < 4; q++) { J[q].object.parent.class_info = (HSD_ClassInfo*) &JI[q % 3]; J[q].parent = (q > 0 && (fv() & 3)) ? &J[fv() % q] : 0; J[q].flags = fv() & 0x1FFFFFF; if (q < 2) J[q].flags &= ~0x4020u;
   J[q].envelopemtx = (fv() & 1) ? (MtxPtr) MT[q] : 0;
   if (J[q].flags & 0x20) J[q].u.ptcl = (fv() & 1) ? &SL[fv() % 3] : 0; else J[q].u.dobj = (fv() & 1) ? &DO[fv() % 3] : 0; }
 for (q = 0; q < 2; q++) { int i; (void) i; }
 for (q = 0; q < 4; q++) { ZL[q].jobj = &J[fv() % 4]; ZL[q].vmtx = (fv() & 1) ? (MtxPtr) MT[q] : 0; ZL[q].sort.texedge = 0; ZL[q].sort.xlu = 0; ZL[q].next = 0; }
 n = fv() % 5; if (n > 4) n = 4; zlist_texedge_nb = 0; zlist_texedge_top = 0; zlist_xlu_nb = 0; zlist_xlu_top = 0;
 for (q = 0; q < 4; q++) ord[q] = q; for (q = 3; q > 0; q--) { k = fv() % (q + 1); { int tq = ord[q]; ord[q] = ord[k]; ord[k] = tq; } }
 k = fv() % 5; { HSD_ZList** pp = &zlist_texedge_top; for (q = 0; q < k && q < 4; q++) { *pp = &ZL[ord[q]]; pp = &ZL[ord[q]].sort.texedge; zlist_texedge_nb++; } }
 for (q = 3; q > 0; q--) { k = fv() % (q + 1); { int tq = ord[q]; ord[q] = ord[k]; ord[k] = tq; } }
 k = fv() % 5; { HSD_ZList** pp = &zlist_xlu_top; for (q = 0; q < k && q < 4; q++) { *pp = &ZL[ord[q]]; pp = &ZL[ord[q]].sort.xlu; zlist_xlu_nb++; } }
 zlist_top = (fv() & 1) ? &ZL[0] : 0; for (q = 0; q < 3; q++) ZL[q].next = (zlist_top && (fv() & 1)) ? &ZL[q + 1] : 0;
 zlist_bottom = (fv() & 1) ? &zlist_top : &ZL[fv() % 4].next; zlist_texedge_bottom = (fv() & 1) ? &zlist_texedge_top : &ZL[fv() % 4].sort.texedge; zlist_xlu_bottom = (fv() & 1) ? &zlist_xlu_top : &ZL[fv() % 4].sort.xlu;
 sptcl_callback = (fv() & 1) ? cb_sp : 0;
 for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); } }'''.replace("\n", " ")
_SETUP_R = r'''{ let mut ord = [0usize; 4];
 for q in 0..3usize { JI[q].make_pmtx = Some(cb_pmtx); JI[q].disp = Some(cb_jdisp); }
 for q in 0..2usize { DI[q].disp = Some(cb_ddisp); }
 for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; }
 for q in 0..3usize { SL[q].data = (if fv() & 1 != 0 { fv() | 0x8000_0000 } else { fv() }) as usize as *mut c_void; }
 for q in 0..3usize { DO[q].next = if q < 2 && fv() & 1 != 0 { &raw mut DO[q + 1] } else { null_mut() }; DO[q].parent.class_info = (&raw mut DI[q & 1]) as *mut HSD_ClassInfo; DO[q].flags = fv() & 0xFF; }
 for q in 0..4usize { J[q].object.parent.class_info = (&raw mut JI[q % 3]) as *mut HSD_ClassInfo; J[q].parent = if q > 0 && fv() & 3 != 0 { &raw mut J[(fv() % q as u32) as usize] } else { null_mut() }; J[q].flags = fv() & 0x1FFFFFF; if q < 2 { J[q].flags &= !0x4020u32; }
   J[q].envelopemtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() };
   if J[q].flags & 0x20 != 0 { J[q].u.ptcl = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() }; } else { J[q].u.dobj = if fv() & 1 != 0 { &raw mut DO[(fv() % 3) as usize] } else { null_mut() }; } }
 for q in 0..4usize { ZL[q].jobj = &raw mut J[(fv() % 4) as usize]; ZL[q].vmtx = if fv() & 1 != 0 { (&raw mut MT[q]) as *mut [f32; 4] } else { null_mut() }; ZL[q].sort.texedge = null_mut(); ZL[q].sort.xlu = null_mut(); ZL[q].next = null_mut(); }
 let _n = fv() % 5; m::zlist_texedge_nb = 0; m::zlist_texedge_top = null_mut(); m::zlist_xlu_nb = 0; m::zlist_xlu_top = null_mut();
 for q in 0..4usize { ord[q] = q; } for q in (1..4usize).rev() { let k = (fv() % (q as u32 + 1)) as usize; ord.swap(q, k); }
 let k = (fv() % 5) as usize; { let mut pp: *mut *mut m::HSD_ZList = &raw mut m::zlist_texedge_top; for q in 0..k.min(4) { *pp = &raw mut ZL[ord[q]]; pp = &raw mut ZL[ord[q]].sort.texedge; m::zlist_texedge_nb += 1; } }
 for q in (1..4usize).rev() { let k = (fv() % (q as u32 + 1)) as usize; ord.swap(q, k); }
 let k = (fv() % 5) as usize; { let mut pp: *mut *mut m::HSD_ZList = &raw mut m::zlist_xlu_top; for q in 0..k.min(4) { *pp = &raw mut ZL[ord[q]]; pp = &raw mut ZL[ord[q]].sort.xlu; m::zlist_xlu_nb += 1; } }
 m::zlist_top = if fv() & 1 != 0 { &raw mut ZL[0] } else { null_mut() }; for q in 0..3usize { ZL[q].next = if !m::zlist_top.is_null() && fv() & 1 != 0 { &raw mut ZL[q + 1] } else { null_mut() }; }
 m::zlist_bottom = if fv() & 1 != 0 { &raw mut m::zlist_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].next }; m::zlist_texedge_bottom = if fv() & 1 != 0 { &raw mut m::zlist_texedge_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].sort.texedge }; m::zlist_xlu_bottom = if fv() & 1 != 0 { &raw mut m::zlist_xlu_top } else { let k = (fv() % 4) as usize; &raw mut ZL[k].sort.xlu };
 m::sptcl_callback = if fv() & 1 != 0 { Some(cb_sp) } else { None };
 for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } } }'''.replace("\n", " ")
VM = ("((fv() & 3) ? (MtxPtr) MT[rnd() % 4] : 0)", "if fv() & 3 != 0 { (&raw mut MT[(rnd() % 4) as usize]) as *mut _ } else { null_mut() }")
VM2 = ("(MtxPtr) MT[rnd() % 4]", "(&raw mut MT[(rnd() % 4) as usize]) as *mut _")
add("displayfunc_native", "src/sysdolphin/baselib/displayfunc.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    hash=("{ int i; for (i = 0; i < 3; i++) HM((u32) (unsigned long long) SL[i].data); }", "{ for i in 0..3usize { hm(SL[i].data as usize as u32); } }"),
    pregs=("preg(MT, sizeof MT); freg((const void*) cb_pmtx); freg((const void*) cb_jdisp); freg((const void*) cb_ddisp); freg((const void*) cb_sp);",
           "preg(&raw const MT, 192); m::use_ext_data = true; freg(cb_pmtx as usize); freg(cb_jdisp as usize); freg(cb_ddisp as usize); freg(cb_sp as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_JObjMakePositionMtx": {1: VM2, 2: VM2}, "HSD_JObjDispSub": {0: ("&J[fv() & 1]", "&raw mut J[(fv() & 1) as usize]"), 1: VM2, 2: VM2}, "HSD_JObjDispDObj": {1: VM},
        "HSD_JObjDisp": {1: VM}, "_HSD_mkEnvelopeModelNodeMtx": {1: VM2}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=["zsort_listing", "zsort_sorting"], own_hash=OWN,
    sub=[(r"\b0x3C\b", "__builtin_offsetof(HSD_ZList, sort.texedge)"), (r"\b0x40\b", "__builtin_offsetof(HSD_ZList, sort.xlu)")],
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte"],
    ret={"HSD_CObjGetCurrent": ("pick_HSD_CObj()", "pick_HSD_CObj()"), "HSD_ObjAlloc": ("(void*) pick_HSD_ZList()", "pick_HSD_ZList() as *mut c_void"),
         "HSD_MtxAlloc": ("(void*) MT[rnd() % 4]", "(&raw mut MT[(rnd() % 4) as usize]) as *mut c_void")})
