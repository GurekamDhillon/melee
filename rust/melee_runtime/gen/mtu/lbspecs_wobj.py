"""wobj.c spec (see lbspecs.py)."""
from lbspecs import add

POOLS = [("HSD_WObj", "WO", 3), ("HSD_AObj", "AO", 3), ("HSD_RObj", "RO", 2), ("HSD_JObj", "JO", 2), ("HSD_WObjInfo", "WI", 2),
         ("HSD_ClassInfo", "CI", 2), ("HSD_Spline", "SPL", 1), ("HSD_ObjData", "OBD", 2), ("HSD_WObjDesc", "WD", 2), ("HSD_WObjAnim", "WA", 2),
         ("HSD_AObjDesc", "AD", 2), ("HSD_RObjDesc", "RD", 2), ("HSD_RObjAnimJoint", "RJ", 2)]
OWN = ["hsdWObj", "default_class"]
_SETUP_C = (
    "{ int q; for (q = 0; q < 2; q++) WI[q].load = WObjLoad; "
    "for (q = 0; q < 3; q++) { WO[q].parent.parent.class_info = (HSD_ClassInfo*) &WI[q & 1]; WO[q].aobj = (fv() & 3) ? &AO[fv() % 3] : 0; WO[q].robj = (fv() & 3) ? &RO[fv() & 1] : 0; } "
    "for (q = 0; q < 3; q++) AO[q].jobj = &JO[q & 1]; for (q = 0; q < 2; q++) JO[q].u.spline = (fv() & 3) ? &SPL[0] : 0; "
    "hsdWObj.parent.parent.head.parent = &CI[0]; CI[0].release = (void*) cbev1; CI[0].amnesia = (void*) cbev1; default_class = (fv() & 1) ? &CI[1] : 0; }")
_SETUP_R = (
    "{ for q in 0..2usize { WI[q].load = Some(m::WObjLoad); } "
    "for q in 0..3usize { WO[q].parent.parent.class_info = (&raw mut WI[q & 1]) as *mut HSD_ClassInfo; WO[q].aobj = if fv() & 3 != 0 { &raw mut AO[(fv() % 3) as usize] } else { null_mut() }; "
    "WO[q].robj = if fv() & 3 != 0 { &raw mut RO[(fv() & 1) as usize] } else { null_mut() }; } "
    "for q in 0..3usize { AO[q].jobj = &raw mut JO[q & 1]; } for q in 0..2usize { JO[q].u.spline = if fv() & 3 != 0 { &raw mut SPL[0] } else { null_mut() }; } "
    "m::hsdWObj.parent.parent.head.parent = &raw mut CI[0]; CI[0].release = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_Class)>(cbev1 as usize)); "
    "CI[0].amnesia = Some(core::mem::transmute::<usize, unsafe extern \"C\" fn(*mut HSD_ClassInfo)>(cbev1 as usize)); m::default_class = if fv() & 1 != 0 { &raw mut CI[1] } else { null_mut() }; }")
add("wobj", "src/sysdolphin/baselib/wobj.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=300,
    ov={"WObjRelease": {0: ("(HSD_Class*) pick_HSD_WObj()", "pick_HSD_WObj() as *mut HSD_Class")}, "WObjUpdateFunc": {0: ("(void*) picko_HSD_WObj()", "picko_HSD_WObj() as *mut c_void"), 1: ("(int) (fv() % 9)", "(fv() % 9) as i32")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=OWN,
    ret={"hsdNew": ("((fv() & 7) ? (void*) pick_HSD_WObj() : 0)", "if fv() & 7 != 0 { pick_HSD_WObj() as *mut c_void } else { null_mut() }"),
         "hsdSearchClassInfo": ("picko_HSD_ClassInfo()", "picko_HSD_ClassInfo()"),
         "HSD_AObjLoadDesc": ("picko_HSD_AObj()", "picko_HSD_AObj()"), "HSD_RObjLoadDesc": ("picko_HSD_RObj()", "picko_HSD_RObj()")})
