"""pobj.c spec (see lbspecs.py): PObj lists of the three kinds (skin / shape anim / envelope), vertex descriptor lists, the class info with
traced disp / setup_mtx / release / destroy callbacks, envelope and shape-set pools. The shape-animation vertex path (get_shape_*,
interpretShapeAnimDisplayList, drawShapeAnim, setupShapeAnim*) needs index-list fixtures and is not driven here."""
from lbspecs import add

POOLS = [('HSD_PObj', 'P0', 3), ('HSD_ShapeAnim', 'P1', 3), ('HSD_ShapeSet', 'P2', 3), ('HSD_ObjData', 'P3', 3), ('HSD_Envelope', 'P4', 3),
         ('HSD_SList', 'P5', 4), ('HSD_ShapeSetDesc', 'P6', 2), ('HSD_PObjDesc', 'P7', 3), ('HSD_PObjInfo', 'P8', 3), ('HSD_VtxDescList', 'P9', 9),
         ('HSD_JObj', 'P10', 3), ('HSD_ClassInfo', 'P12', 3), ('HSD_EnvelopeDesc', 'P13', 3), ('HSD_AObj', 'P14', 2), ('HSD_AObjDesc', 'P15', 2),
         ('HSD_Joint', 'P16', 2), ('HSD_Class', 'P11', 2)]
OWN = ["hsdPObj", "genoPalPObj", "default_class", "vertex_buffer", "normal_buffer", "vertex_buffer_size", "normal_buffer_size",
       "prev_vtxdesclist_array", "prev_vtxdesc", "mtx_mark", "geno_pal_nforced"]
C_DECL = r'''
static HSD_EnvelopeDesc* EPL[3][4]; static unsigned RK;
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static void cb_amn(HSD_ClassInfo* c) { T(8205); T(PI(c)); }
static void cb_disp(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(8206); T(PI(p)); T(PI(v)); T(PI(m)); T(r); }
static void cb_setup(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(8207); T(PI(p)); T(PI(v)); T(PI(m)); T(r); }
static int cb_load(HSD_PObj* p, HSD_PObjDesc* d) { T(8208); T(PI(p)); T(PI(d)); return (int) (fv() % 3) - 1; }
static unsigned char MT[4][48];
'''
RS_EXTRA = r'''
static mut EPL: [[*mut HSD_EnvelopeDesc; 4]; 3] = [[null_mut(); 4]; 3]; static mut RK: usize = 0;
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
unsafe extern "C" fn cb_amn(c: *mut HSD_ClassInfo) { t(8205); t(pi(c)); }
unsafe extern "C" fn cb_disp(p: *mut HSD_PObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8206); t(pi(p)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_setup(p: *mut HSD_PObj, v: *mut [f32; 4], m: *mut [f32; 4], r: u32) { t(8207); t(pi(p)); t(pi(v)); t(pi(m)); t(r); }
unsafe extern "C" fn cb_load(p: *mut HSD_PObj, d: *mut HSD_PObjDesc) -> i32 { t(8208); t(pi(p)); t(pi(d)); (fv() % 3) as i32 - 1 }
static mut MT: [[u8; 48]; 4] = [[0; 48]; 4];
'''
_SETUP_C = r'''{ int q, l;
 for (q = 0; q < 3; q++) { HSD_ClassInfo* ci = (HSD_ClassInfo*) &P8[q];
   ci->release = (void*) cb_rel; ci->destroy = (void*) cb_dest; ci->amnesia = cb_amn; P8[q].disp = cb_disp; P8[q].setup_mtx = (q == 0) ? PObjSetupMtx : cb_setup; P8[q].load = cb_load; }
 for (l = 0; l < 3; l++) { P9[3 * l].attr = fv() % 26; P9[3 * l + 1].attr = fv() % 26; P9[3 * l + 2].attr = 255; P9[3 * l].vertex = (fv() & 1) ? BUF : 0; P9[3 * l + 1].vertex = (fv() & 1) ? BUF + 64 : 0; P9[3 * l].attr_type = fv() % 4; P9[3 * l + 1].attr_type = fv() % 4; }
 for (q = 0; q < 4; q++) { P5[q].next = (q < 3 && (fv() & 1)) ? &P5[q + 1] : 0; P5[q].data = &P4[fv() % 3]; }
 for (q = 0; q < 3; q++) { P4[q].next = (q < 2 && (fv() & 1)) ? &P4[q + 1] : 0; P4[q].jobj = &P10[fv() % 3]; }
 for (q = 0; q < 3; q++) { int k = 0; int n = fv() % 4; for (; k < n; k++) EPL[q][k] = &P13[fv() % 3]; EPL[q][k] = 0; }
 for (q = 0; q < 3; q++) P13[q].joint = (fv() & 3) ? &P16[fv() & 1] : 0; P13[2].joint = 0;
 for (q = 0; q < 3; q++) { P2[q].aobj = (fv() & 1) ? &P14[fv() & 1] : 0; P2[q].flags &= (u16) ~2; }
 for (q = 0; q < 3; q++) { int kind = (q == 0) ? 0 : (q == 1) ? 0x2000 : (fv() % 3) * 0x1000;
   P0[q].parent.class_info = (HSD_ClassInfo*) &P8[q]; P0[q].next = (q < 2 && (fv() & 1)) ? &P0[q + 1] : 0; P0[q].verts = &P9[3 * (fv() % 3)];
   P0[q].flags = (u16) ((fv() & 0xCFFF) | kind); P0[q].n_display = (u16) (fv() % 4); P0[q].display = BUF;
   if (kind == 0) P0[q].u.jobj = (q == 0 || (fv() & 1)) ? &P10[fv() % 3] : 0; else if (kind == 0x1000) P0[q].u.shape_set = &P2[fv() % 3]; else P0[q].u.envelope_list = (fv() & 3) ? &P5[fv() % 4] : 0; }
 for (q = 0; q < 3; q++) { int kind = P0[q].flags & 0x3000; P7[q].n_display = (u16) (fv() % 4); P7[q].class_name = (fv() & 3) ? 0 : STRS[fv() % 4]; P7[q].next = P0[q].next ? &P7[q + 1] : 0; P7[q].verts = &P9[3 * (fv() % 3)];
   P7[q].flags = (u16) ((fv() & 0xCFFF) | kind); P7[q].display = BUF; if (kind == 0) P7[q].u.joint = (fv() & 3) ? &P16[fv() & 1] : 0; else if (kind == 0x1000) P7[q].u.shape_set = &P6[fv() & 1]; else P7[q].u.envelope_p = (fv() & 3) ? EPL[fv() % 3] : 0; }
 for (q = 0; q < 2; q++) { P6[q].vertex_desc = &P9[0]; P6[q].normal_desc = &P9[3]; P6[q].flags &= (u16) ~2; }
 for (q = 0; q < 4; q++) { int i; for (i = 0; i < 48; i++) MT[q][i] = (unsigned char) rnd(); }
 hsdPObj.parent.release = (void*) cb_rel; hsdPObj.parent.destroy = (void*) cb_dest; hsdPObj.parent.amnesia = cb_amn; hsdPObj.parent.head.parent = &P12[0]; P12[0].release = (void*) cb_rel; P12[0].amnesia = cb_amn; P12[0].destroy = (void*) cb_dest;
 default_class = (fv() & 1) ? &P8[fv() % 3] : 0; mtx_mark[0].obj = (fv() & 1) ? (void*) &P10[fv() % 3] : 0; mtx_mark[1].obj = (fv() & 1) ? (void*) &P10[fv() % 3] : 0; mtx_mark[0].mark = fv() % 4; mtx_mark[1].mark = fv() % 4;
 prev_vtxdesc = (fv() & 1) ? &P9[3 * (fv() % 3)] : 0; prev_vtxdesclist_array = (fv() & 1) ? &P9[3 * (fv() % 3)] : 0; }'''.replace("\n", " ")
_SETUP_R = r'''{
 for q in 0..3usize { let ci = (&raw mut P8[q]) as *mut HSD_ClassInfo; (*ci).release = Some(cb_rel); (*ci).destroy = Some(cb_dest); (*ci).amnesia = Some(cb_amn); P8[q].disp = Some(cb_disp); P8[q].setup_mtx = if q == 0 { Some(m::PObjSetupMtx) } else { Some(cb_setup) }; P8[q].load = Some(cb_load); }
 for l in 0..3usize { P9[3 * l].attr = fv() % 26; P9[3 * l + 1].attr = fv() % 26; P9[3 * l + 2].attr = 255; P9[3 * l].vertex = if fv() & 1 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }; P9[3 * l + 1].vertex = if fv() & 1 != 0 { ((&raw mut BUF) as *mut u8).add(64) as *mut c_void } else { null_mut() }; P9[3 * l].attr_type = fv() % 4; P9[3 * l + 1].attr_type = fv() % 4; }
 for q in 0..4usize { P5[q].next = if q < 3 && fv() & 1 != 0 { &raw mut P5[q + 1] } else { null_mut() }; P5[q].data = (&raw mut P4[(fv() % 3) as usize]) as *mut c_void; }
 for q in 0..3usize { P4[q].next = if q < 2 && fv() & 1 != 0 { &raw mut P4[q + 1] } else { null_mut() }; P4[q].jobj = &raw mut P10[(fv() % 3) as usize]; }
 for q in 0..3usize { let n = (fv() % 4) as usize; let mut k = 0usize; while k < n { EPL[q][k] = &raw mut P13[(fv() % 3) as usize]; k += 1; } EPL[q][k] = null_mut(); }
 for q in 0..3usize { P13[q].joint = if fv() & 3 != 0 { &raw mut P16[(fv() & 1) as usize] } else { null_mut() }; } P13[2].joint = null_mut();
 for q in 0..3usize { P2[q].aobj = if fv() & 1 != 0 { &raw mut P14[(fv() & 1) as usize] } else { null_mut() }; P2[q].flags &= !2u16; }
 for q in 0..3usize { let kind: u32 = if q == 0 { 0 } else if q == 1 { 0x2000 } else { (fv() % 3) * 0x1000 };
   P0[q].parent.class_info = (&raw mut P8[q]) as *mut HSD_ClassInfo; P0[q].next = if q < 2 && fv() & 1 != 0 { &raw mut P0[q + 1] } else { null_mut() }; P0[q].verts = &raw mut P9[(3 * (fv() % 3)) as usize];
   P0[q].flags = ((fv() & 0xCFFF) | kind) as u16; P0[q].n_display = (fv() % 4) as u16; P0[q].display = (&raw mut BUF) as *mut u8;
   if kind == 0 { P0[q].u.jobj = if q == 0 || fv() & 1 != 0 { &raw mut P10[(fv() % 3) as usize] } else { null_mut() }; } else if kind == 0x1000 { P0[q].u.shape_set = &raw mut P2[(fv() % 3) as usize]; } else { P0[q].u.envelope_list = if fv() & 3 != 0 { &raw mut P5[(fv() % 4) as usize] } else { null_mut() }; } }
 for q in 0..3usize { let kind: u32 = (P0[q].flags & 0x3000) as u32; P7[q].n_display = (fv() % 4) as u16; P7[q].class_name = if fv() & 3 != 0 { null_mut() } else { (&raw mut STRS[(fv() % 4) as usize]) as *mut i8 }; P7[q].next = if !P0[q].next.is_null() { &raw mut P7[q + 1] } else { null_mut() }; P7[q].verts = &raw mut P9[(3 * (fv() % 3)) as usize];
   P7[q].flags = ((fv() & 0xCFFF) | kind) as u16; P7[q].display = (&raw mut BUF) as *mut u8; if kind == 0 { P7[q].u.joint = if fv() & 3 != 0 { &raw mut P16[(fv() & 1) as usize] } else { null_mut() }; } else if kind == 0x1000 { P7[q].u.shape_set = &raw mut P6[(fv() & 1) as usize]; } else { P7[q].u.envelope_p = if fv() & 3 != 0 { (&raw mut EPL[(fv() % 3) as usize]) as *mut *mut HSD_EnvelopeDesc } else { null_mut() }; } }
 for q in 0..2usize { P6[q].vertex_desc = &raw mut P9[0]; P6[q].normal_desc = &raw mut P9[3]; P6[q].flags &= !2u16; }
 for q in 0..4usize { for i in 0..48usize { MT[q][i] = rnd() as u8; } }
 m::hsdPObj.parent.release = Some(cb_rel); m::hsdPObj.parent.destroy = Some(cb_dest); m::hsdPObj.parent.amnesia = Some(cb_amn); m::hsdPObj.parent.head.parent = &raw mut P12[0]; P12[0].release = Some(cb_rel); P12[0].amnesia = Some(cb_amn); P12[0].destroy = Some(cb_dest);
 m::default_class = if fv() & 1 != 0 { &raw mut P8[(fv() % 3) as usize] } else { null_mut() }; m::mtx_mark[0].obj = if fv() & 1 != 0 { (&raw mut P10[(fv() % 3) as usize]) as *mut c_void } else { null_mut() }; m::mtx_mark[1].obj = if fv() & 1 != 0 { (&raw mut P10[(fv() % 3) as usize]) as *mut c_void } else { null_mut() }; m::mtx_mark[0].mark = fv() % 4; m::mtx_mark[1].mark = fv() % 4;
 m::prev_vtxdesc = if fv() & 1 != 0 { &raw mut P9[(3 * (fv() % 3)) as usize] } else { null_mut() }; m::prev_vtxdesclist_array = if fv() & 1 != 0 { &raw mut P9[(3 * (fv() % 3)) as usize] } else { null_mut() }; }'''.replace("\n", " ")
VMAT = ("(Mtx*) MT[rnd() % 4]", "(&raw mut MT[(rnd() % 4) as usize]) as *mut _")
add("pobj", "src/sysdolphin/baselib/pobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    skip=["setupShapeAnimArrayDesc", "setupShapeAnimVtxDesc", "get_shape_vertex_xyz", "get_shape_normal_xyz", "get_shape_nbt_xyz",
          "interpretShapeAnimDisplayList", "drawShapeAnim", "PObjDispShapeAnim", "env_hash_f32s", "geno_pal_compute", "geno_pal_selftest_check",
          "geno_pal_force_convert", "GenoPalLoad", "GenoPalSetupMtx", "GenoPalDisp"],
    pregs=("preg(MT, sizeof MT); preg(EPL, sizeof EPL); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_amn); freg((const void*) cb_disp); freg((const void*) cb_setup); freg((const void*) cb_load);",
           "preg(&raw const MT, 192); preg(&raw const EPL, 96); m::use_ext_data = true; freg(cb_rel as usize); freg(cb_dest as usize); freg(cb_amn as usize); freg(cb_disp as usize); freg(cb_setup as usize); freg(cb_load as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_PObjDisp": {0: ("(fv() & 1) ? &P0[1] : &P0[0]", "if fv() & 1 != 0 { &raw mut P0[1] } else { &raw mut P0[0] }"), 1: VMAT, 2: VMAT},
        "PObjUpdateFunc": {0: ("(void*) picko_HSD_PObj()", "picko_HSD_PObj() as *mut c_void")},
        "HSD_PObjSetMtxMark": {0: ("(int) (fv() % 4)", "(fv() % 4) as i32")},
        "SetupEnvelopeModelMtx": {0: ("&P0[1]", "&raw mut P0[1]")}, "SetupSharedVtxModelMtx": {0: ("&P0[0]", "&raw mut P0[0]")},
        "HSD_PObjResolveRefsAll": {0: ("&P0[0]", "&raw mut P0[0]"), 1: ("&P7[0]", "&raw mut P7[0]")},
        "HSD_PObjResolveRefs": {0: ("(RK = fv() % 3, &P0[RK])", "{ RK = (fv() % 3) as usize; &raw mut P0[RK] }"), 1: ("&P7[RK]", "&raw mut P7[RK]")},
        "PObjRelease": {0: ("(HSD_Class*) pick_HSD_PObj()", "pick_HSD_PObj() as *mut HSD_Class")},
        "HSD_PObjFree": {0: ("pick_HSD_PObj()", "pick_HSD_PObj()")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte"], ptr64_args={"HSD_IDGetDataFromTable": [1]},
    notrace_args={"diag_pobj_draw": [0], "GenoPalCacheGet": [0, 1, 2], "GenoPalCachePut": [0, 1, 2], "diag_env_slot": [0, 1], "env_cache_get": [0, 1], "env_cache_put": [0, 1], "env_cache_compare": [0, 1]},  # keys hash host JObj addresses
    ret={"hsdNew": ("(((void*) a0 == (void*) &genoPalPObj || (fv() & 7)) ? (void*) pick_HSD_PObj() : 0)", "if a0 as *mut c_void == (&raw mut m::genoPalPObj) as *mut c_void || fv() & 7 != 0 { pick_HSD_PObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("(HSD_ClassInfo*) picko_HSD_PObjInfo()", "picko_HSD_PObjInfo() as *mut _HSD_ClassInfo"),
         "HSD_JObjGetCurrent": ("pick_HSD_JObj()", "pick_HSD_JObj()"),
         "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()"),
         "HSD_SListAlloc": ("pick_HSD_SList()", "pick_HSD_SList()"),
         "HSD_SListRemove": ("picko_HSD_SList()", "picko_HSD_SList()"),
         "HSD_IDGetDataFromTable": ("(void*) picko_HSD_JObj()", "picko_HSD_JObj() as *mut c_void"),
         "hsdAllocMemPiece": ("(a0 == sizeof(HSD_Envelope)) ? (void*) pick_HSD_Envelope() : (void*) pick_HSD_ShapeSet()", "if a0 == core::mem::size_of::<HSD_Envelope>() as i32 { pick_HSD_Envelope() as *mut c_void } else { pick_HSD_ShapeSet() as *mut c_void }"),
         "_HSD_mkEnvelopeModelNodeMtx": ("(void*) MT[rnd() % 4]", "(&raw mut MT[(rnd() % 4) as usize]) as *mut c_void"),
         "_HSD_TObjGetCurrentByType": ("0", "null_mut()")})
