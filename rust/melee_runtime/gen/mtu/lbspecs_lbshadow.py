"""lbshadow spec (see lbspecs.py)."""
from lbspecs import add

POOLS = [("HSD_GObj", "GO", 4), ("LbShadow", "LS", 2), ("HSD_JObj", "JO", 3), ("HSD_Shadow", "SHD", 3), ("HSD_CObj", "CO", 2),
         ("HSD_LObj", "LO", 3), ("Fighter", "FTR", 3), ("CmSubject", "CM", 2), ("HSD_TObj", "TO", 2), ("HSD_ImageDesc", "ID", 2),
         ("HSD_Spline", "SP", 1)]
OWN = ["HSD_GObj_804D7814", "HSD_PerfCurrentStat"]

_P_C = "(Vec3*) FA"
_P_R = "(&raw mut FA) as *mut Vec"
_CP_C = "VA"
_CP_R = "(&raw mut VA) as *mut Vec"
_U_C = "((fv() & 7) == 0 ? 1.0f : (float) (fv() % 1001) / 1000.0f)"
_U_R = "if fv() & 7 == 0 { 1.0f32 } else { (fv() % 1001) as f32 / 1000.0f32 }"

_SETUP_C = (
    "{ int q; for (q = 0; q < 16; q++) PLH[q] = 0; for (q = 0; q < 8; q++) GXH[q] = 0; "
    "PLH[8] = (fv() & 7) ? &GO[0] : 0; GXH[4] = (fv() & 7) ? &GO[fv() & 3] : 0; HSD_GObjPLinkHead = PLH; HSD_GObjGXLinkHead = GXH; "
    "for (q = 0; q < 4; q++) { GO[q].next = q < 3 ? &GO[q + 1] : 0; GO[q].next_gx = q < 3 ? &GO[q + 1] : 0; GO[q].user_data = &FTR[q % 3]; GO[q].hsd_obj = &LO[q % 3]; } "
    "for (q = 0; q < 3; q++) LO[q].next = q < 2 ? &LO[q + 1] : 0; "
    "for (q = 0; q < 3; q++) { FTR[q].x890_cameraBox = (fv() & 3) ? &CM[fv() & 1] : 0; FTR[q].x20A4.shadow = (fv() & 3) ? &SHD[fv() % 3] : 0; SHD[q].camera = &CO[q & 1]; SHD[q].texture = &TO[q & 1]; } "
    "for (q = 0; q < 2; q++) TO[q].imagedesc = &ID[q]; "
    "SP[0].type = (u8) (fv() % 5); SP[0].numcv = (s16) (1 + fv() % 6); SP[0].tension = rw(); SP[0].cv = (Vec3*) (BUF + 12); "
    "for (q = 0; q < 63; q++) ((float*) BUF)[q] = rw(); }")
_SETUP_R = (
    "{ for q in 0..16usize { PLH[q] = null_mut(); } for q in 0..8usize { GXH[q] = null_mut(); } "
    "PLH[8] = if fv() & 7 != 0 { &raw mut GO[0] } else { null_mut() }; GXH[4] = if fv() & 7 != 0 { &raw mut GO[(fv() & 3) as usize] } else { null_mut() }; "
    "m::ext::HSD_GObjPLinkHead = (&raw mut PLH) as *mut *mut HSD_GObj; m::ext::HSD_GObjGXLinkHead = (&raw mut GXH) as *mut *mut HSD_GObj; "
    "for q in 0..4usize { GO[q].next = if q < 3 { &raw mut GO[q + 1] } else { null_mut() }; GO[q].next_gx = if q < 3 { &raw mut GO[q + 1] } else { null_mut() }; "
    "GO[q].user_data = (&raw mut FTR[q % 3]) as *mut c_void; GO[q].hsd_obj = (&raw mut LO[q % 3]) as *mut c_void; } "
    "for q in 0..3usize { LO[q].next = if q < 2 { &raw mut LO[q + 1] } else { null_mut() }; } "
    "for q in 0..3usize { FTR[q].x890_cameraBox = if fv() & 3 != 0 { &raw mut CM[(fv() & 1) as usize] } else { null_mut() }; "
    "FTR[q].x20A4.shadow = if fv() & 3 != 0 { &raw mut SHD[(fv() % 3) as usize] } else { null_mut() }; SHD[q].camera = &raw mut CO[q & 1]; SHD[q].texture = &raw mut TO[q & 1]; } "
    "for q in 0..2usize { TO[q].imagedesc = &raw mut ID[q]; } "
    "SP[0].r#type = (fv() % 5) as u8; SP[0].numcv = (1 + fv() % 6) as i16; SP[0].tension = rw(); SP[0].cv = ((&raw mut BUF) as *mut u8).add(12) as *mut Vec; "
    "for q in 0..63usize { *((&raw mut BUF) as *mut f32).add(q) = rw(); } }")

add("lbshadow", "src/melee/lb/lbshadow.c", dict(
    pub_only=True, pools=POOLS, setup=(_SETUP_C, _SETUP_R), ncases=400,
    c_pre="#include <sysdolphin/baselib/jobj.h>\n", c_decl="static HSD_GObj* PLH[16]; static HSD_GObj* GXH[8]; Mtx HSD_identityMtx;",
    rs_extra="static mut PLH: [*mut HSD_GObj; 16] = [null_mut(); 16]; static mut GXH: [*mut HSD_GObj; 8] = [null_mut(); 8];",
    pregs=("preg(PLH, sizeof PLH); preg(GXH, sizeof GXH);", "preg(&raw const PLH, core::mem::size_of_val(&PLH)); preg(&raw const GXH, core::mem::size_of_val(&GXH));"),
    ov={"splGetCardinalTangent": {0: (_P_C, _P_R), 1: (_CP_C, _CP_R), 3: (_U_C, _U_R)},
        "splGetBSplineTangent": {0: (_P_C, _P_R), 1: (_CP_C, _CP_R), 2: (_U_C, _U_R)},
        "splGetBezierTangent": {0: (_P_C, _P_R), 1: (_CP_C, _CP_R), 2: (_U_C, _U_R)},
        "lbShadow_8000E9F0": {0: (_P_C, _P_R), 2: (_U_C, _U_R)}},
    call={"splGetCardinalTangent": ("splGetCardinalTangent(a0, a1, a2, a3)", "m::splGetCardinalTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2, a3)"),
          "splGetBSplineTangent": ("splGetBSplineTangent(a0, a1, a2)", "m::splGetBSplineTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2)"),
          "splGetBezierTangent": ("splGetBezierTangent(a0, a1, a2)", "m::splGetBezierTangent(&mut *a0, core::slice::from_raw_parts(a1, 4), a2)"),
          "lbShadow_8000E9F0": ("lbShadow_8000E9F0(a0, a1, a2)", "m::lbShadow_8000E9F0(&mut *a0, &*a1, a2)")}),
    pub_only=True, skip_stubs=["__frsqrte"], skip_data=["HSD_identityMtx"], types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS],
    ret={"ftLib_800872B0": ("((fv() & 7) ? pick_LbShadow() : 0)", "if fv() & 7 != 0 { pick_LbShadow() } else { null_mut() }"),
         "HSD_ShadowAlloc": ("((fv() & 7) ? pick_HSD_Shadow() : 0)", "if fv() & 7 != 0 { pick_HSD_Shadow() } else { null_mut() }"),
         "HSD_LObjGetPosition": ("(int) ((fv() & 7) != 0)", "((fv() & 7) != 0) as i32")})
