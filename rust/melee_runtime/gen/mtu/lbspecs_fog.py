"""fog.c spec (see lbspecs.py): native-layout fog objects, fog adj, descriptors, the two class infos."""
from lbspecs import add

POOLS = [("HSD_Fog", "FG", 3), ("HSD_FogAdj", "FGA", 3), ("HSD_FogDesc", "FD", 2), ("HSD_FogAdjDesc", "FAD", 2), ("HSD_AObj", "AO", 2),
         ("HSD_AObjDesc", "AD", 2), ("HSD_ObjData", "OBD", 2), ("HSD_ClassInfo", "CI", 3), ("HSD_FogInfo", "FIN", 2)]
OWN = ["hsdFog", "hsdFogAdj"]
C_DECL = r'''
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
'''
RS_EXTRA = r'''
unsafe extern "C" fn cb_rel(c: *mut HSD_Class) { t(8203); t(pi(c)); }
unsafe extern "C" fn cb_dest(c: *mut HSD_Class) { t(8204); t(pi(c)); }
'''
_SETUP_C = ("{ int q; for (q = 0; q < 3; q++) { FG[q].fog_adj = (fv() & 1) ? &FGA[fv() % 3] : 0; FG[q].aobj = (fv() & 1) ? &AO[fv() & 1] : 0; FG[q].parent.parent.class_info = &CI[0]; FGA[q].parent.parent.class_info = &CI[0]; } "
            "for (q = 0; q < 2; q++) FD[q].fogadjdesc = (fv() & 1) ? &FAD[fv() & 1] : 0; hsdFog.parent.parent.head.parent = &CI[1]; for (q = 0; q < 3; q++) { CI[q].release = (void*) cb_rel; CI[q].destroy = (void*) cb_dest; } }")
_SETUP_R = ("{ for q in 0..3usize { FG[q].fog_adj = if fv() & 1 != 0 { &raw mut FGA[(fv() % 3) as usize] } else { null_mut() }; FG[q].aobj = if fv() & 1 != 0 { &raw mut AO[(fv() & 1) as usize] } else { null_mut() }; "
            "FG[q].parent.parent.class_info = &raw mut CI[0]; FGA[q].parent.parent.class_info = &raw mut CI[0]; } "
            "for q in 0..2usize { FD[q].fogadjdesc = if fv() & 1 != 0 { &raw mut FAD[(fv() & 1) as usize] } else { null_mut() }; } m::hsdFog.parent.parent.head.parent = &raw mut CI[1]; for q in 0..3usize { CI[q].release = Some(cb_rel); CI[q].destroy = Some(cb_dest); } }")
add("fog", "src/sysdolphin/baselib/fog.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA,
    ov={"FogUpdateFunc": {0: ("(void*) picko_HSD_Fog()", "picko_HSD_Fog() as *mut c_void")}, "FogRelease": {0: ("pick_HSD_Fog()", "pick_HSD_Fog() as *mut HSD_Class")}},
    pregs=("freg((const void*) cb_rel); freg((const void*) cb_dest);", "freg(cb_rel as usize); freg(cb_dest as usize); m::use_ext_data = true; melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    effects={"GXGetViewportv": ("{ int q; for (q = 0; q < 6; q++) a0[q] = rw(); }", "{ for q in 0..6usize { *a0.add(q) = rw(); } }"),
             "GXGetProjectionv": ("{ int q; a0[0] = (float) (fv() % 2); for (q = 1; q < 7; q++) a0[q] = rw(); }", "{ *a0 = (fv() % 2) as f32; for q in 1..7usize { *a0.add(q) = rw(); } }")},
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    skip_stubs=["memcpy", "memset"],
    ret={"hsdNew": ("((fv() & 7) ? ((void*) a0 == (void*) &hsdFog ? (void*) pick_HSD_Fog() : (void*) pick_HSD_FogAdj()) : 0)",
                    "if fv() & 7 != 0 { if a0 as *mut c_void == (&raw mut m::hsdFog) as *mut c_void { pick_HSD_Fog() as *mut c_void } else { pick_HSD_FogAdj() as *mut c_void } } else { null_mut() }"),
         "HSD_CObjGetCurrent": ("0", "null_mut()"), "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()")})
