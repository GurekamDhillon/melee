"""Descriptions of the stage TUs with their own state (see mk_stage_driver.py)."""
from mk_stage_driver import (STD_C_DECL, STD_RS_DECL, STD_PREG, STD_SETUP_C, STD_SETUP_RS, STD_HASH_C,
                             STD_HASH_RS, gobj_fn, pred_fn, void_fn, pred0_fn, table_op, std)

CONFIGS = {}

# ------------------------------------------------------------------------------------ grfigureget
_PL_C = """    zero(PLINK, sizeof PLINK); zero(FIG, sizeof FIG);
    for (i = 0; i < 3; i++) fill_HSD_GObj(&FIG[i]);
    HSD_GObjPLinkHead = PLINK;
    n = (int) (rnd() % 4);
    for (i = 0; i < n; i++) FIG[i].next = (i + 1 < n) ? &FIG[i + 1] : 0;
    PLINK[HSD_GOBJ_PLINK_FIGHTER] = n ? &FIG[0] : 0;
"""
_PL_RS = """    PLINK = [null_mut(); 16];
    FIG = core::mem::zeroed();
    for i in 0..3 {
        g::fill_HSD_GObj(&mut FIG[i]);
    }
    m::ext::HSD_GObjPLinkHead = PLINK.as_mut_ptr();
    n = rnd() % 4;
    for i in 0..n as usize {
        FIG[i].next = if i + 1 < n as usize { &raw mut FIG[i + 1] } else { null_mut() };
    }
    PLINK[HSD_GOBJ_PLINK_FIGHTER as usize] = if n != 0 { &raw mut FIG[0] } else { null_mut() };
"""
_PL_C_DECL = "static HSD_GObj FIG[3];\nstatic HSD_GObj* PLINK[16];\n"
_PL_RS_DECL = ("static mut FIG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };\n"
               "static mut PLINK: [*mut HSD_GObj; 16] = [null_mut(); 16];\n")
_PL_PREG = [("FIG", "sizeof FIG", "&raw const FIG", "core::mem::size_of_val(&FIG)"),
            ("PLINK", "sizeof PLINK", "&raw const PLINK", "core::mem::size_of_val(&PLINK)")]
_PL_HASH_C = "    for (i = 0; i < 3; i++) hash_HSD_GObj(&FIG[i]);\n"
_PL_HASH_RS = "    for i in 0..3 {\n        g::hash_HSD_GObj(&FIG[i]);\n    }\n"

_fg_c_decl = STD_C_DECL + _PL_C_DECL + """static grFigureGet_Params YP;
static CollData FCD, CD;
static HSD_GObj IG[3], CGO;
static Item ITD;
"""
_fg_rs_decl = STD_RS_DECL + _PL_RS_DECL + """static mut YP: m::grFigureGet_Params = m::grFigureGet_Params { x0: 0, x4: 0, x8: 0, xC: 0.0, x10: 0.0, x14: 0.0 };
static mut FCD: CollData = unsafe { core::mem::zeroed() };
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut IG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut CGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut ITD: Item = unsafe { core::mem::zeroed() };
"""
_fg_setup_c = STD_SETUP_C + _PL_C + """    zero(&YP, sizeof YP); zero(&FCD, sizeof FCD); zero(&CD, sizeof CD); zero(IG, sizeof IG); zero(&CGO, sizeof CGO);
    zero(&ITD, sizeof ITD);
    fill_grFigureGet_Params(&YP);
    YP.x8 = (int) (rnd() % 4);
    YP.x4 = (rnd() & 1) ? 0 : (int) (rnd() % 40);
    YP.x0 = (int) (rnd() % 30);
    YP.xC = 0.0f;
    YP.x10 = (float) (rnd() % 200);
    YP.x14 = rw();
    fill_CollData(&FCD); fill_CollData(&CD); fill_HSD_GObj(&CGO);
    for (i = 0; i < 3; i++) fill_HSD_GObj(&IG[i]);
    yakumono_param = &YP;
    SGP.u.figureget.x4 = (int) (rnd() % 3);
    SGP.u.figureget.x8 = (int) (rnd() % 4);
    SGP.u.figureget.xC = (int) (rnd() % 4);
    SGP.u.figureget.x0 = (int) (rnd() % 4);
    SGP.u.figureget.x28[0] = ((rnd() & 3) == 0) ? &CGO : 0;
    CD.x0_gobj = ((rnd() & 3) == 0) ? 0 : &CGO;
    CGO.classifier = (rnd() & 3) ? HSD_GOBJ_CLASS_ITEM : (u16) fv();
    CGO.user_data = &ITD;
    ITD.kind = (rnd() & 3) ? It_Kind_Coin : (int) fv();
    CD.x34_flags.b1234 = (rnd() & 3) ? 5 : fv();
"""
_fg_setup_rs = STD_SETUP_RS + _PL_RS + """    YP = core::mem::zeroed();
    FCD = core::mem::zeroed();
    CD = core::mem::zeroed();
    IG = core::mem::zeroed();
    CGO = core::mem::zeroed();
    ITD = core::mem::zeroed();
    g::fill_grFigureGet_Params(&mut YP);
    YP.x8 = (rnd() % 4) as i32;
    YP.x4 = if rnd() & 1 != 0 { 0 } else { (rnd() % 40) as i32 };
    YP.x0 = (rnd() % 30) as i32;
    YP.xC = 0.0;
    YP.x10 = (rnd() % 200) as f32;
    YP.x14 = rw();
    g::fill_CollData(&mut FCD);
    g::fill_CollData(&mut CD);
    g::fill_HSD_GObj(&mut CGO);
    for i in 0..3 {
        g::fill_HSD_GObj(&mut IG[i]);
    }
    m::yakumono_param = &raw mut YP;
    SGP.u.figureget.x4 = (rnd() % 3) as i32;
    SGP.u.figureget.x8 = (rnd() % 4) as i32;
    SGP.u.figureget.xC = (rnd() % 4) as i32;
    SGP.u.figureget.x0 = (rnd() % 4) as i32;
    SGP.u.figureget.x28[0] = if rnd() & 3 == 0 { &raw mut CGO } else { null_mut() };
    CD.x0_gobj = if rnd() & 3 == 0 { null_mut() } else { &raw mut CGO };
    CGO.classifier = if rnd() & 3 != 0 { HSD_GOBJ_CLASS_ITEM as u16 } else { fv() as u16 };
    CGO.user_data = &raw mut ITD as *mut _;
    ITD.kind = if rnd() & 3 != 0 { It_Kind_Coin } else { fv() as i32 };
    CD.x34_flags.set_b1234(if rnd() & 3 != 0 { 5 } else { fv() as u8 });
"""
_fg_pre_c = """    for (i = 0; i < 3; i++) {
        HM(PI(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = 0;
        HM(PI(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = 0;
    }
"""
_fg_pre_rs = """    for i in 0..3 {
        hm(pi(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = null_mut();
        hm(pi(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = null_mut();
    }
"""
_fg_hash_c = _fg_pre_c + STD_HASH_C + _PL_HASH_C + """    hash_grFigureGet_Params(&YP); hash_CollData(&FCD); hash_CollData(&CD); hash_HSD_GObj(&CGO);
    for (i = 0; i < 3; i++) hash_HSD_GObj(&IG[i]);
    HM(PI(yakumono_param));
"""
_fg_hash_rs = _fg_pre_rs + STD_HASH_RS + _PL_HASH_RS + """    g::hash_grFigureGet_Params(&YP);
    g::hash_CollData(&FCD);
    g::hash_CollData(&CD);
    g::hash_HSD_GObj(&CGO);
    for i in 0..3 {
        g::hash_HSD_GObj(&IG[i]);
    }
    hm(pi(m::yakumono_param));
"""
_fg_ops = [
    ("demoinit", "grFigureGet_OnDemoInit((int) fv());", "m::grFigureGet_OnDemoInit(fv() as i32);"),
    void_fn("grFigureGet_OnInit"), void_fn("grFigureGet_OnLoad"), void_fn("grFigureGet_OnStart"),
    pred0_fn("grFigureGet_802195C4"),
    ("setup_gobj", "ret = PI(grFigureGet_802195CC((int) (rnd() % 2)));", "ret = pi(m::grFigureGet_802195CC((rnd() % 2) as i32));"),
    gobj_fn("stageGObj0_OnInit"), pred_fn("grFigureGet_802196E0"), gobj_fn("grFigureGet_802196E8"),
    gobj_fn("grFigureGet_802196EC"), gobj_fn("grFigureGet_802196F0"), pred_fn("grFigureGet_80219890"),
    gobj_fn("grFigureGet_80219898"), gobj_fn("grFigureGet_80219B0C"),
    ("joint_cb", "{ int j = (int) fv(); int x = (int) fv(); float d = rw(); grFigureGet_80219B10(&SGP, j, &CD, x, (mpLib_GroundEnum) fv(), d); }",
     "{ let j = fv() as i32; let x = fv() as i32; let d = rw(); m::grFigureGet_80219B10(&raw mut SGP as *mut _, j, &raw mut CD, x, fv() as _, d); }"),
    ("c34", "{ HSD_GObj* g = ((rnd() & 3) == 0) ? 0 : &SGO; grFigureGet_80219C34(g); }",
     "{ let g = if rnd() & 3 == 0 { null_mut() } else { &raw mut SGO }; m::grFigureGet_80219C34(g); }"),
    ("c50", "{ HSD_GObj* g = ((rnd() & 3) == 0) ? 0 : &SGO; ret = (u32) grFigureGet_80219C50(g); }",
     "{ let g = if rnd() & 3 == 0 { null_mut() } else { &raw mut SGO }; ret = m::grFigureGet_80219C50(g) as u32; }"),
    ("touchline", "ret = PI(grFigureGet_OnTouchLine((enum_t) fv()));", "ret = pi(m::grFigureGet_OnTouchLine(fv() as _));"),
    ("shadow", "ret = (u32) grFigureGet_OnCheckShadowRender(&SV, (int) fv(), (HSD_JObj*) SJO);",
     "ret = m::grFigureGet_OnCheckShadowRender(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32;"),
    ("frames", "{ int j; for (j = 0; j < 12; j++) { grFigureGet_80219898(&SGO); CD.cur_pos.x = rw(); "
              "if (rnd() & 1) { int a = (int) fv(); grFigureGet_80219B10(&SGP, 0, &CD, a, (mpLib_GroundEnum) 0, 0.0f); } } }",
     "{ for _ in 0..12 { m::grFigureGet_80219898(&raw mut SGO); CD.cur_pos.x = rw(); "
     "if rnd() & 1 != 0 { let a = fv() as i32; m::grFigureGet_80219B10(&raw mut SGP as *mut _, 0, &raw mut CD, a, 0, 0.0); } } }"),
]
CONFIGS["grfigureget"] = dict(
    stem="grfigureget", c_decl=_fg_c_decl, rs_decl=_fg_rs_decl,
    pregs=list(STD_PREG) + _PL_PREG + [
        ("&YP", "sizeof YP", "&raw const YP", "core::mem::size_of::<m::grFigureGet_Params>()"),
        ("&FCD", "sizeof FCD", "&raw const FCD", "core::mem::size_of::<CollData>()"),
        ("&CD", "sizeof CD", "&raw const CD", "core::mem::size_of::<CollData>()"),
        ("IG", "sizeof IG", "&raw const IG", "core::mem::size_of_val(&IG)"),
        ("&CGO", "sizeof CGO", "&raw const CGO", "core::mem::size_of::<HSD_GObj>()"),
        ("&ITD", "sizeof ITD", "&raw const ITD", "core::mem::size_of::<Item>()"),
        ("grFigureGet_StageCallbacks", "sizeof grFigureGet_StageCallbacks", "&raw const m::grFigureGet_StageCallbacks",
         "core::mem::size_of_val(&m::grFigureGet_StageCallbacks)"),
        ("&grFigureGet_StageData", "sizeof grFigureGet_StageData", "&raw const m::grFigureGet_StageData",
         "core::mem::size_of_val(&m::grFigureGet_StageData)")],
    c_extra="", rs_extra="", setup_c=_fg_setup_c, setup_rs=_fg_setup_rs, hash_c=_fg_hash_c, hash_rs=_fg_hash_rs,
    ops=_fg_ops + [table_op("grFigureGet_StageCallbacks", 2, "grFigureGet_StageData")], extra_freg=[])
