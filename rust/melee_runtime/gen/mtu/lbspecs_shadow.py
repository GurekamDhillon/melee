"""shadow.c spec: shadow objects (camera, shadow texture with its image desc, the object list) and viewing rects."""
from lbspecs import add

POOLS = [('HSD_Shadow', 'P0', 2), ('HSD_JObj', 'P1', 2), ('HSD_ViewingRect', 'P2', 2), ('HSD_CObj', 'CO', 2), ('HSD_TObj', 'TO', 2),
         ('HSD_ImageDesc', 'IM', 2), ('HSD_SList', 'SL', 3), ('HSD_ClassInfo', 'CI', 1)]
C_DECL = r'''
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
'''
OWN = ["shadow_alloc_data"]
add("shadow", "src/sysdolphin/baselib/shadow.c", dict(
    pub_only=True, pools=POOLS, ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    setup=("{ int q; CI[0].release = (void*) cb_rel; CI[0].destroy = (void*) cb_dest; for (q = 0; q < 2; q++) CO[q].parent.parent.class_info = &CI[0]; for (q = 0; q < 3; q++) { SL[q].next = (q < 2 && (fv() & 1)) ? &SL[q + 1] : 0; SL[q].data = &P1[fv() & 1]; }"
           " for (q = 0; q < 2; q++) { P0[q].camera = &CO[q]; P0[q].texture = &TO[q]; TO[q].imagedesc = &IM[q]; P0[q].objects = (fv() & 1) ? &SL[fv() % 3] : 0;"
           " IM[q].image_ptr = (fv() & 1) ? BUF : 0; IM[q].width = (u16) (fv() % 300); IM[q].height = (u16) (fv() % 300); } }",
           "{ CI[0].release = Some(cb_rel); CI[0].destroy = Some(cb_dest); for q in 0..2usize { CO[q].parent.parent.class_info = &raw mut CI[0]; } for q in 0..3usize { SL[q].next = if q < 2 && fv() & 1 != 0 { &raw mut SL[q + 1] } else { null_mut() }; SL[q].data = (&raw mut P1[(fv() & 1) as usize]) as *mut c_void; }"
           " for q in 0..2usize { P0[q].camera = &raw mut CO[q]; P0[q].texture = &raw mut TO[q]; TO[q].imagedesc = &raw mut IM[q]; P0[q].objects = if fv() & 1 != 0 { &raw mut SL[(fv() % 3) as usize] } else { null_mut() };"
           " IM[q].image_ptr = if fv() & 1 != 0 { (&raw mut BUF) as *mut c_void } else { null_mut() }; IM[q].width = (fv() % 300) as u16; IM[q].height = (fv() % 300) as u16; } }"),
    pregs=("freg((const void*) cb_rel); freg((const void*) cb_dest);", "freg(cb_rel as usize); freg(cb_dest as usize); m::use_ext_data = true; melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte", "tanf"],
    ret={"HSD_CObjAlloc": ("pick_HSD_CObj()", "pick_HSD_CObj()"), "HSD_TObjAlloc": ("pick_HSD_TObj()", "pick_HSD_TObj()"),
         "HSD_ImageDescAlloc": ("pick_HSD_ImageDesc()", "pick_HSD_ImageDesc()"), "HSD_ObjAlloc": ("(void*) pick_HSD_Shadow()", "pick_HSD_Shadow() as *mut c_void"),
         "HSD_SListAllocAndPrepend": ("pick_HSD_SList()", "pick_HSD_SList()"), "HSD_SListRemove": ("picko_HSD_SList()", "picko_HSD_SList()"),
         "HSD_MemAlloc": ("(void*) BUF", "(&raw mut BUF) as *mut c_void"), "HSD_CObjSetCurrent": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "GXGetTexBufferSize": ("(u32) (fv() % 5)", "fv() % 5")})
