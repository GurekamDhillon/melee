"""lbanim.c spec (see lbspecs.py)."""
from lbspecs import add

POOLS = [("FigaTrack", "TRK", 4), ("FigaTree", "TRE", 2), ("HSD_JObj", "JO", 2), ("HSD_AObj", "AO", 4), ("HSD_FObj", "FO", 4)]
_SETUP_C = ("{ int q; FOI = 0; AOI = 0; for (q = 0; q < 4; q++) { TRK[q].obj_type = (u8) ((fv() % 4) ? 1 + fv() % 4 : 8 + fv() % 20); } "
            "for (q = 0; q < 2; q++) JO[q].aobj = (fv() & 1) ? &AO[3] : 0; }")
_SETUP_R = ("{ FOI = 0; AOI = 0; for q in 0..4usize { TRK[q].obj_type = if fv() % 4 != 0 { 1 + (fv() % 4) as u8 } else { 8 + (fv() % 20) as u8 }; } "
            "for q in 0..2usize { JO[q].aobj = if fv() & 1 != 0 { &raw mut AO[3] } else { null_mut() }; } }")
FRM = ("(s8) (fv() % 4)", "(fv() % 4) as i8")
FRM1 = ("(s8) (1 + fv() % 3)", "(1 + fv() % 3) as i8")
add("lbanim", "src/melee/lb/lbanim.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=400,
    c_decl="static int FOI, AOI;", rs_extra="static mut FOI: usize = 0; static mut AOI: usize = 0;",
    ov={"lbAnim_InitFrames": {0: ("&TRK[0]", "&raw mut TRK[0]"), 1: FRM1}, "fn_8001E60C": {0: ("&TRK[0]", "&raw mut TRK[0]"), 1: FRM1},
        "lbAnim_8001E6D8": {2: ("&TRK[0]", "&raw mut TRK[0]"), 3: FRM}, "lbAnim_8001E7E8": {2: ("&TRK[0]", "&raw mut TRK[0]"), 3: FRM}},
    call={"lbAnim_8001E8F8": ("lbAnim_8001E8F8(a0)", "m::lbAnim_8001E8F8(if a0.is_null() { None } else { Some(&*a0) })")}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS],
    effects={"HSD_AObjSetFObj": ("a0->fobj = a1;", "(*a0).fobj = a1;"), "HSD_JObjSetFlags": ("a0->flags |= a1;", "(*a0).flags |= a1;"), "HSD_JObjClearFlags": ("a0->flags &= ~a1;", "(*a0).flags &= !a1;")},
    ret={"HSD_FObjAlloc": ("&FO[FOI++ % 4]", "{ FOI += 1; &raw mut FO[(FOI - 1) % 4] }"), "HSD_AObjAlloc": ("&AO[AOI++ % 3]", "{ AOI += 1; &raw mut AO[(AOI - 1) % 3] }")})
