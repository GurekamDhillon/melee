"""Oracle spec of tu::grpura (see gen_drv.py for the keys)."""
TBL_C = """for (i = 0; i < 44; i++) grPu_803E6C0C[i].jobj = ((fv() & 3) == 0) ? 0 : &JP[fv() % 16];"""
TBL_R = """for i in 0..44usize { m::grPu_803E6C0C[i].jobj = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut JP[(fv() % 16) as usize] }; }"""
TBL0_C = """for (i = 0; i < 44; i++) grPu_803E6C0C[i].jobj = 0;"""
TBL0_R = """for i in 0..44usize { m::grPu_803E6C0C[i].jobj = core::ptr::null_mut(); }"""

DP_C = """for (i = 0; i < 8; i++) { DP[i].next = (i + 1 < 8 && fv() % 4 != 0) ? &DP[i + 1] : 0; DP[i].mobj = (fv() % 5 == 0) ? 0 : &MP[fv() % 8]; }
for (i = 0; i < 16; i++) { if (fv() % 4 != 0) JP[i].flags &= ~(JOBJ_PTCL | JOBJ_SPLINE); JP[i].u.dobj = (fv() % 4 == 0) ? 0 : &DP[fv() % 8]; }"""
DP_R = """for i in 0..8usize { DP[i].next = if i + 1 < 8 && fv() % 4 != 0 { &raw mut DP[i + 1] } else { core::ptr::null_mut() }; DP[i].mobj = if fv() % 5 == 0 { core::ptr::null_mut() } else { &raw mut MP[(fv() % 8) as usize] }; }
for i in 0..16usize { if fv() % 4 != 0 { JP[i].flags &= !((JOBJ_PTCL | JOBJ_SPLINE) as u32); } JP[i].u.dobj = if fv() % 4 == 0 { core::ptr::null_mut() } else { &raw mut DP[(fv() % 8) as usize] }; }"""

PU_C = ("for (i = 0; i < 4; i++) { GR[i].u.pura.xC4 = (s16) (fv() % 4); GR[i].u.pura.xC6 = (s16) (fv() % 4); "
        "GR[i].u.pura.xC8 = (s16) ((fv() % 3 == 0) ? 0xE0E + fv() % 4 : fv() % 0xE10); }")
PU_R = ("for i in 0..4usize { GR[i].u.pura.xC4 = (fv() % 4) as i16; GR[i].u.pura.xC6 = (fv() % 4) as i16; "
        "GR[i].u.pura.xC8 = (if fv() % 3 == 0 { 0xE0E + fv() % 4 } else { fv() % 0xE10 }) as i16; }")

SPEC = dict(
    ptr_unions={"u": "dobj"},
    src="src/melee/gr/grpura.c",
    types=["StageInfo", "StageCallbacks", "StageData", "CmSubject"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "Ground_801C3FA4", "Ground_GetMapGObj"],
    pools={
        "HSD_JObj": ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]"),
        "CmSubject": ("&CS", "&raw mut CS"),
    },
    ret={
        "Camera_80029020": ("((fv() & 3) == 0 ? 0 : &CS)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::CS }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetStageGObj": ("((NG && (fv() & 3) == 0) ? 0 : &GO[fv() & 1])",
                                "if super::NG && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 1) as usize] }"),
        "Ground_GetMapGObj": ("((fv() & 3) == 0 ? 0 : &GO[fv() & 3])",
                              "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "Ground_801C3FA4": ("&JP[fv() % 16]", "&raw mut super::JP[(fv() % 16) as usize]"),
    },
    effects={
        "lb_8000B1CC": ("TV(a1); WV(a2);", "super::tv(a1); super::wv(a2);"),
    },
    decl_c="static u8 YP[64];\nstatic CmSubject CS;\nstatic HSD_DObj DP[8];\nstatic int NG;\nstatic HSD_MObj MP[8];",
    decl_r="static mut YP: [u8; 64] = [0; 64];\nstatic mut CS: CmSubject = unsafe { core::mem::zeroed() };\nstatic mut NG: bool = false;\nstatic mut DP: [HSD_DObj; 8] = unsafe { core::mem::zeroed() };\nstatic mut MP: [HSD_MObj; 8] = unsafe { core::mem::zeroed() };",
    setup_c="""NG = 1; zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info); zero(&CS, sizeof CS); fill_CmSubject(&CS);
zero(YP, sizeof YP); yakumono_param = YP; zero(DP, sizeof DP); zero(MP, sizeof MP);
""" + DP_C + chr(10) + TBL_C,
    setup_r="""NG = true; m::ext::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info); CS = core::mem::zeroed(); g::fill_CmSubject(&mut CS);
YP = [0; 64]; m::yakumono_param = (&raw mut YP) as *mut c_void; DP = core::mem::zeroed(); MP = core::mem::zeroed();
""" + DP_R + chr(10) + TBL_R,
    hash_c="hash_StageInfo(&stage_info); hash_CmSubject(&CS); HM(PI(yakumono_param)); { int k; for (k = 0; k < 44; k++) { HM(PI(grPu_803E6C0C[k].jobj)); HM((u32) grPu_803E6C0C[k].joint.x); } for (k = 0; k < 8; k++) { HM(PI(DP[k].next)); HM(PI(DP[k].mobj)); } }",
    hash_r="g::hash_StageInfo(&m::ext::stage_info); g::hash_CmSubject(&CS); hm(pi(m::yakumono_param)); for k in 0..44usize { hm(pi(m::grPu_803E6C0C[k].jobj)); hm(m::grPu_803E6C0C[k].joint.x as u32); } for k in 0..8usize { hm(pi(DP[k].next)); hm(pi(DP[k].mobj)); }",
    preg_c="preg(&CS, sizeof CS); preg(YP, sizeof YP); preg(DP, sizeof DP); preg(MP, sizeof MP); preg(stage_callbacks, sizeof stage_callbacks);",
    preg_r="preg(&raw const CS, core::mem::size_of_val(&CS)); preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const DP, core::mem::size_of_val(&DP)); preg(&raw const MP, core::mem::size_of_val(&MP)); preg(&raw const m::stage_callbacks, core::mem::size_of_val(&m::stage_callbacks));",
    data=[("stage_callbacks", "StageCallbacks", 28), ("grPu_StageData", "StageData", None),
          ("grPu_803E6AA0", "raw", None), ("grPu_803E6AB0", "raw", None), ("grPu_803E6E20", "raw", None)],
    view_null={"x128": 4, "xC4": 4},
    args={
        "grPura_80211E08_noinline": [("(int) ({ u32 v_ = fv() % 28; v_ == 4 ? 5 : v_; })", "{ let v = (fv() % 28) as i32; if v == 4 { 5 } else { v } }")],
        "grPura_80211E08_noinline2": [("(int) ({ u32 v_ = fv() % 28; v_ == 4 ? 5 : v_; })", "{ let v = (fv() % 28) as i32; if v == 4 { 5 } else { v } }")],
        "grPura_80213128": [("&DP[fv() % 8]", "&raw mut DP[(fv() % 8) as usize]")],
        "grPura_80213224": [("&DP[fv() % 8]", "&raw mut DP[(fv() % 8) as usize]")],
        "setupStageCallbacks": [("(int) ({ u32 v_ = fv() % 28; v_ == 4 ? 5 : v_; })", "{ let v = (fv() % 28) as i32; if v == 4 { 5 } else { v } }")],
    },
    pre={"stageGObj1_GObjProc": (PU_C, PU_R), "grPura_80211D00": ("NG = 0;", "NG = false;"), "grPura_802125F0": ("NG = 0;", "NG = false;"), "stageGObj4_OnInit": ("NG = 0;", "NG = false;")},
    # the dispatchers run on_init callbacks that overlay different views of Ground::u (pura / pura2 / pura3 share
    # offset 0, and pura3 holds pointers): no Ground::u in their fill and hash; 802125F0 writes pura.xC4 and
    # pura2.xC8, both covered by the pura2 view
    view={"grPura_80211D00": "-", "stageGObj4_OnInit": "-", "grPura_802125F0": "pura2"},
    hview={"grPura_80211D00": "-", "stageGObj4_OnInit": "-", "grPura_802125F0": "pura2"},
    cases={"stageGObj1_GObjProc": 1000, "grPura_802125F0": 600},
)
