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
        HM(PIB(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = 0;
        HM(PIB(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = 0;
    }
"""
_fg_pre_rs = """    for i in 0..3 {
        hm(pib(SGP.u.figureget.x28[i])); SGP.u.figureget.x28[i] = null_mut();
        hm(pib(SGP.u.figureget.x34[i])); SGP.u.figureget.x34[i] = null_mut();
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
        ("&ITD", "16", "&raw const ITD", "16"),
        ("grFigureGet_StageCallbacks", "sizeof grFigureGet_StageCallbacks", "&raw const m::grFigureGet_StageCallbacks",
         "core::mem::size_of_val(&m::grFigureGet_StageCallbacks)"),
        ("&grFigureGet_StageData", "sizeof grFigureGet_StageData", "&raw const m::grFigureGet_StageData",
         "core::mem::size_of_val(&m::grFigureGet_StageData)")],
    c_extra="", rs_extra="", setup_c=_fg_setup_c, setup_rs=_fg_setup_rs, hash_c=_fg_hash_c, hash_rs=_fg_hash_rs,
    ops=_fg_ops + [table_op("grFigureGet_StageCallbacks", 2, "grFigureGet_StageData")], extra_freg=[])


# ------------------------------------------------------------------------------------ grflatzone
_fz_c_decl = STD_C_DECL + """static HSD_JObj SJOB;
static struct grFlatzone_YakumonoParam YP;
static HSD_GObj MGO[10], IGO;
static Ground MGP[10];
static HSD_JObj MJO[10];
static Item ITD;
static grDynamicAttr_UnkStruct DYN[2];
"""
_fz_rs_decl = STD_RS_DECL + """static mut SJOB: HSD_JObj = unsafe { core::mem::zeroed() };
static mut YP: m::grFlatzone_YakumonoParam = unsafe { core::mem::zeroed() };
static mut MGO: [HSD_GObj; 10] = unsafe { core::mem::zeroed() };
static mut IGO: HSD_GObj = unsafe { core::mem::zeroed() };
static mut MGP: [Ground; 10] = unsafe { core::mem::zeroed() };
static mut MJO: [HSD_JObj; 10] = unsafe { core::mem::zeroed() };
static mut ITD: Item = unsafe { core::mem::zeroed() };
static mut DYN: [grDynamicAttr_UnkStruct; 2] = unsafe { core::mem::zeroed() };
"""
_fz_setup_c = STD_SETUP_C + """    zero(&SJOB, sizeof SJOB); zero(&YP, sizeof YP); zero(MGO, sizeof MGO); zero(&IGO, sizeof IGO);
    zero(MGP, sizeof MGP); zero(MJO, sizeof MJO); zero(&ITD, sizeof ITD); zero(DYN, sizeof DYN);
    fill_HSD_JObj(&SJOB);
    SGO.hsd_obj = &SJOB;
    fill_grFlatzone_YakumonoParam(&YP);
    YP.unk0 = (int) (rnd() % 60); YP.unk4 = (int) (rnd() % 60); YP.unk8 = (int) (rnd() % 60); YP.unkC = (int) (rnd() % 60);
    YP.unk10 = (int) (rnd() % 100); YP.unk14 = 1 + (int) (rnd() % 6); YP.unk18 = (int) (rnd() % 80) - 40;
    YP.unk1C = (int) (rnd() % 80) - 40; YP.unk20 = (int) (rnd() % 50); YP.unk2C = 1 + (int) (rnd() % 5);
    YP.unk30 = (int) (rnd() % 10); YP.unk34 = (int) (rnd() % 10); YP.unk3C = (int) (rnd() % 80);
    YP.unk24 = rw(); YP.unk28 = rw(); YP.unk38 = rw();
    yakumono_param = &YP;
    fill_HSD_GObj(&IGO); IGO.user_data = ((rnd() & 3) == 0) ? 0 : &ITD;
    fill_CollData(&ITD.x378_itemColl);
    for (i = 0; i < 2; i++) fill_grDynamicAttr_UnkStruct(&DYN[i]);
    for (i = 0; i < 10; i++) {
        fill_HSD_GObj(&MGO[i]); fill_Ground(&MGP[i]); fill_HSD_JObj(&MJO[i]);
        MGO[i].user_data = ((rnd() & 7) == 0) ? 0 : &MGP[i];
        MGO[i].hsd_obj = ((rnd() & 7) == 0) ? 0 : (void*) &MJO[i];
    }
    n = (int) (rnd() % 4);
    SGP.u.flatzone2.xD0 = (n == 0) ? 2 : ((n == 1) ? 1 : (int) (rnd() % 7) - 1);
    SGP.u.flatzone2.timer = (int) (rnd() % 45) - 3;
    SGP.u.flatzone2.xC8 = (rnd() & 1) ? ((rnd() & 1) ? 1.0f : -1.0f) : rw();
    SGP.u.flatzone2.xCC = 0;
"""
_fz_setup_rs = STD_SETUP_RS + """    SJOB = core::mem::zeroed();
    YP = core::mem::zeroed();
    MGO = core::mem::zeroed();
    IGO = core::mem::zeroed();
    MGP = core::mem::zeroed();
    MJO = core::mem::zeroed();
    ITD = core::mem::zeroed();
    DYN = core::mem::zeroed();
    g::fill_HSD_JObj(&mut SJOB);
    SGO.hsd_obj = &raw mut SJOB as *mut _;
    g::fill_grFlatzone_YakumonoParam(&mut YP);
    YP.unk0 = (rnd() % 60) as i32;
    YP.unk4 = (rnd() % 60) as i32;
    YP.unk8 = (rnd() % 60) as i32;
    YP.unkC = (rnd() % 60) as i32;
    YP.unk10 = (rnd() % 100) as i32;
    YP.unk14 = 1 + (rnd() % 6) as i32;
    YP.unk18 = (rnd() % 80) as i32 - 40;
    YP.unk1C = (rnd() % 80) as i32 - 40;
    YP.unk20 = (rnd() % 50) as i32;
    YP.unk2C = 1 + (rnd() % 5) as i32;
    YP.unk30 = (rnd() % 10) as i32;
    YP.unk34 = (rnd() % 10) as i32;
    YP.unk3C = (rnd() % 80) as i32;
    YP.unk24 = rw();
    YP.unk28 = rw();
    YP.unk38 = rw();
    m::yakumono_param = &raw mut YP;
    g::fill_HSD_GObj(&mut IGO);
    IGO.user_data = if rnd() & 3 == 0 { null_mut() } else { &raw mut ITD as *mut _ };
    g::fill_CollData(&mut ITD.x378_itemColl);
    for i in 0..2 {
        g::fill_grDynamicAttr_UnkStruct(&mut DYN[i]);
    }
    for i in 0..10 {
        g::fill_HSD_GObj(&mut MGO[i]);
        g::fill_Ground(&mut MGP[i]);
        g::fill_HSD_JObj(&mut MJO[i]);
        MGO[i].user_data = if rnd() & 7 == 0 { null_mut() } else { &raw mut MGP[i] as *mut _ };
        MGO[i].hsd_obj = if rnd() & 7 == 0 { null_mut() } else { &raw mut MJO[i] as *mut _ };
    }
    n = rnd() % 4;
    SGP.u.flatzone2.xD0 = if n == 0 { 2 } else if n == 1 { 1 } else { (rnd() % 7) as i32 - 1 };
    SGP.u.flatzone2.timer = (rnd() % 45) as i32 - 3;
    SGP.u.flatzone2.xC8 = if rnd() & 1 != 0 {
        if rnd() & 1 != 0 { 1.0 } else { -1.0 }
    } else {
        rw()
    };
    SGP.u.flatzone2.xCC = null_mut();
"""
_fz_pre_c = """    for (i = 0; i < 10; i++) { HM(PIB(MGP[i].u.flatzone2.xCC)); MGP[i].u.flatzone2.xCC = 0; }
    HM(PIB(SGP.u.flatzone2.xCC)); SGP.u.flatzone2.xCC = 0;
"""
_fz_pre_rs = """    for i in 0..10 {
        hm(pib(MGP[i].u.flatzone2.xCC));
        MGP[i].u.flatzone2.xCC = null_mut();
    }
    hm(pib(SGP.u.flatzone2.xCC));
    SGP.u.flatzone2.xCC = null_mut();
"""
_fz_hash_c = _fz_pre_c + STD_HASH_C + """    hash_HSD_JObj(&SJOB); hash_grFlatzone_YakumonoParam(&YP); HM(PI(yakumono_param));
    for (i = 0; i < 10; i++) { hash_HSD_GObj(&MGO[i]); hash_Ground(&MGP[i]); hash_HSD_JObj(&MJO[i]); }
    hash_HSD_GObj(&IGO); hash_CollData(&ITD.x378_itemColl);
    for (i = 0; i < 2; i++) hash_grDynamicAttr_UnkStruct(&DYN[i]);
"""
_fz_hash_rs = _fz_pre_rs + STD_HASH_RS + """    g::hash_HSD_JObj(&SJOB);
    g::hash_grFlatzone_YakumonoParam(&YP);
    hm(pi(m::yakumono_param));
    for i in 0..10 {
        g::hash_HSD_GObj(&MGO[i]);
        g::hash_Ground(&MGP[i]);
        g::hash_HSD_JObj(&MJO[i]);
    }
    g::hash_HSD_GObj(&IGO);
    g::hash_CollData(&ITD.x378_itemColl);
    for i in 0..2 {
        g::hash_grDynamicAttr_UnkStruct(&DYN[i]);
    }
"""
_FZ = "grFlatzone_"
_fz_ops = [
    ("demoinit", "grFlatzone_80216E74((int) fv());", "m::grFlatzone_80216E74(fv() as i32);"),
    void_fn("grFlatzone_80216E78"), void_fn("grFlatzone_UnkStage0_OnLoad"), void_fn("grFlatzone_UnkStage0_OnStart"),
    pred0_fn("grFlatzone_80216F40"),
    ("setup_gobj", "ret = PI(grFlatzone_80216F48((int) (rnd() % 9)));", "ret = pi(m::grFlatzone_80216F48((rnd() % 9) as i32));"),
    gobj_fn("stageGObj0_OnInit"), pred_fn("grFlatzone_8021705C"), gobj_fn("grFlatzone_80217064"),
    gobj_fn("grFlatzone_80217068"), gobj_fn("fn_8021706C"), gobj_fn("grFlatzone_8021707C"),
    pred_fn("grFlatzone_802171CC"), gobj_fn("grFlatzone_802171D4"), gobj_fn("grFlatzone_802173D0"),
    gobj_fn("grFlatzone_802173D4"), pred_fn("grFlatzone_80217430"), gobj_fn("grFlatzone_80217438"),
    gobj_fn("grFlatzone_8021743C"), gobj_fn("fn_80217440"), gobj_fn("grFlatzone_80217450"),
    pred_fn("grFlatzone_802174E4"),
    ("802174EC", "SGP.u.flatzone3.xC7 = (rnd() % 5 == 4) ? 8 : (u8) (rnd() % 4); SGP.u.flatzone3.xC9 = (u8) (rnd() % 5); grFlatzone_802174EC(&SGO);",
     "SGP.u.flatzone3.xC7 = if rnd() % 5 == 4 { 8 } else { (rnd() % 4) as u8 }; SGP.u.flatzone3.xC9 = (rnd() % 5) as u8; m::grFlatzone_802174EC(&raw mut SGO);"),
    gobj_fn("grFlatzone_80217644"), gobj_fn("grFlatzone_80217648"), pred_fn("grFlatzone_802176B4"),
    ("802176BC", "SGP.u.flatzone.xC4 = (u8) (rnd() & 1); SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; grFlatzone_802176BC(&SGO);",
     "SGP.u.flatzone.xC4 = (rnd() & 1) as u8; SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; m::grFlatzone_802176BC(&raw mut SGO);"), gobj_fn("grFlatzone_80217CD0"), gobj_fn("grFlatzone_80217CD4"),
    pred_fn("grFlatzone_80217D28"), gobj_fn("grFlatzone_80217D30"), gobj_fn("grFlatzone_80217DA4"),
    gobj_fn("grFlatzone_80217DA8"), pred_fn("grFlatzone_80217DFC"), gobj_fn("grFlatzone_80217E04"),
    gobj_fn("grFlatzone_80217E78"), gobj_fn("grFlatzone_80217E7C"), pred_fn("grFlatzone_80217E80"),
    gobj_fn("grFlatzone_80217E88"), gobj_fn("grFlatzone_80217E8C"), gobj_fn("grFlatzone_80217E90"),
    pred_fn("grFlatzone_80217EE8"), gobj_fn("grFlatzone_80217EF0"), gobj_fn("grFlatzone_8021805C"),
    ("80218060", "grFlatzone_80218060((int) (rnd() & 1));", "m::grFlatzone_80218060((rnd() & 1) as i32);"),
    void_fn("grFlatzone_802181B4"), void_fn("grFlatzone_80218260"),
    ("touchline", "ret = PI(grFlatzone_802182B4((enum_t) fv()));", "ret = pi(m::grFlatzone_802182B4(fv() as _));"),
    ("shadow", "ret = (u32) grFlatzone_802182BC(&SV, (int) fv(), (HSD_JObj*) SJO);",
     "ret = m::grFlatzone_802182BC(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32;"),
    ("crane_state1", "SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 1; SGP.u.flatzone2.timer = YP.unk2C * (int) (rnd() % 5) + 1; "
     "SGP.u.flatzone2.xC8 = rw(); grFlatzone_802176BC(&SGO);",
     "SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 1; SGP.u.flatzone2.timer = YP.unk2C * (rnd() % 5) as i32 + 1; "
     "SGP.u.flatzone2.xC8 = rw(); m::grFlatzone_802176BC(&raw mut SGO);"),
    ("crane_state2", "SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 2; SGP.u.flatzone2.xC8 = rw(); grFlatzone_802176BC(&SGO);",
     "SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 0; SGP.u.flatzone2.xD0 = 2; SGP.u.flatzone2.xC8 = rw(); m::grFlatzone_802176BC(&raw mut SGO);"),
    # whole sequences: the crane (gobj 3), the machine (gobj 5), the item rain (gobj 8), the building timer (gobj 2)
    ("crane_seq",
     "{ int j; grFlatzone_80217648(&SGO); SGP.u.flatzone2.xCC = (rnd() & 1) ? &DYN[0] : 0; SGP.u.flatzone.xC4 = 1; for (j = 0; j < 90; j++) { grFlatzone_802176BC(&SGO); "
     "if ((rnd() & 31) == 0) SGP.u.flatzone.xC4 = 1; } }",
     "{ m::grFlatzone_80217648(&raw mut SGO); SGP.u.flatzone2.xCC = if rnd() & 1 != 0 { &raw mut DYN[0] } else { null_mut() }; SGP.u.flatzone.xC4 = 1; for _ in 0..90 { m::grFlatzone_802176BC(&raw mut SGO); "
     "if rnd() & 31 == 0 { SGP.u.flatzone.xC4 = 1; } } }"),
    ("machine_seq",
     "{ int j; grFlatzone_80217450(&SGO); SGP.u.flatzone3.xC4 = 0; SGP.u.flatzone3.xC7 = (u8) (rnd() % 4); "
     "SGP.u.flatzone3.xC9 = (u8) (1 + rnd() % 4); for (j = 0; j < 60; j++) { grFlatzone_802174EC(&SGO); if ((rnd() & 15) == 0) "
     "SGP.u.flatzone3.xC7 = (u8) (rnd() % 4); } }",
     "{ m::grFlatzone_80217450(&raw mut SGO); SGP.u.flatzone3.xC4 = 0; SGP.u.flatzone3.xC7 = (rnd() % 4) as u8; "
     "SGP.u.flatzone3.xC9 = (1 + rnd() % 4) as u8; for _ in 0..60 { m::grFlatzone_802174EC(&raw mut SGO); if rnd() & 15 == 0 { "
     "SGP.u.flatzone3.xC7 = (rnd() % 4) as u8; } } }"),
    ("rain_seq",
     "{ int j; grFlatzone_80217E90(&SGO); SGP.u.flatzone3.xC4 = 1; for (j = 0; j < 80; j++) grFlatzone_80217EF0(&SGO); }",
     "{ m::grFlatzone_80217E90(&raw mut SGO); SGP.u.flatzone3.xC4 = 1; for _ in 0..80 { m::grFlatzone_80217EF0(&raw mut SGO); } }"),
    ("timer_seq",
     "{ int j; grFlatzone_8021707C(&SGO); SGP.u.flatzone.xC4 = 0; for (j = 0; j < 200; j++) grFlatzone_802171D4(&SGO); }",
     "{ m::grFlatzone_8021707C(&raw mut SGO); SGP.u.flatzone.xC4 = 0; for _ in 0..200 { m::grFlatzone_802171D4(&raw mut SGO); } }"),
]
CONFIGS["grflatzone"] = dict(
    stem="grflatzone", c_decl=_fz_c_decl, rs_decl=_fz_rs_decl,
    pregs=list(STD_PREG) + [
        ("&SJOB", "sizeof SJOB", "&raw const SJOB", "core::mem::size_of::<HSD_JObj>()"),
        ("&YP", "sizeof YP", "&raw const YP", "core::mem::size_of::<m::grFlatzone_YakumonoParam>()"),
        ("MGO", "sizeof MGO", "&raw const MGO", "core::mem::size_of_val(&MGO)"),
        ("&IGO", "sizeof IGO", "&raw const IGO", "core::mem::size_of::<HSD_GObj>()"),
        ("MGP", "sizeof MGP", "&raw const MGP", "core::mem::size_of_val(&MGP)"),
        ("MJO", "sizeof MJO", "&raw const MJO", "core::mem::size_of_val(&MJO)"),
        ("&ITD", "16", "&raw const ITD", "16"),
        ("DYN", "sizeof DYN", "&raw const DYN", "core::mem::size_of_val(&DYN)"),
        ("grFz_StageCallbacks", "sizeof grFz_StageCallbacks", "&raw const m::grFz_StageCallbacks",
         "core::mem::size_of_val(&m::grFz_StageCallbacks)"),
        ("&grFz_StageData", "sizeof grFz_StageData", "&raw const m::grFz_StageData",
         "core::mem::size_of_val(&m::grFz_StageData)")],
    c_extra="", rs_extra="", setup_c=_fz_setup_c, setup_rs=_fz_setup_rs, hash_c=_fz_hash_c, hash_rs=_fz_hash_rs,
    ops=_fz_ops + [table_op("grFz_StageCallbacks", 9, "grFz_StageData")], extra_freg=[])


# ------------------------------------------------------------------------------------ grfourside
# PTRMODE bits: 1 = foursideUfo.xC holds a pointer, 2 = fourside.x0/x4/x8 hold pointers,
# 4 = the crane's x1 bitfield byte is in use (hashed by value; the byte differs between bit orders).
_fs_c_decl = STD_C_DECL + """static HSD_JObj SJOB;
static struct grFourside_YakumonoParam YP;
static HSD_GObj MGO[8];
static Ground MGP[8];
static HSD_JObj MJO[8], FJ[10];
static CmSubject CMS;
static HSD_CObj COBJ;
static CollData CD;
static int PTRMODE;
"""
_fs_rs_decl = STD_RS_DECL + """static mut SJOB: HSD_JObj = unsafe { core::mem::zeroed() };
static mut YP: m::grFourside_YakumonoParam = unsafe { core::mem::zeroed() };
static mut MGO: [HSD_GObj; 8] = unsafe { core::mem::zeroed() };
static mut MGP: [Ground; 8] = unsafe { core::mem::zeroed() };
static mut MJO: [HSD_JObj; 8] = unsafe { core::mem::zeroed() };
static mut FJ: [HSD_JObj; 10] = unsafe { core::mem::zeroed() };
static mut CMS: CmSubject = unsafe { core::mem::zeroed() };
static mut COBJ: HSD_CObj = unsafe { core::mem::zeroed() };
static mut CD: CollData = unsafe { core::mem::zeroed() };
static mut PTRMODE: i32 = 0;
"""
_fs_setup_c = STD_SETUP_C + """    zero(&SJOB, sizeof SJOB); zero(&YP, sizeof YP); zero(MGO, sizeof MGO); zero(MGP, sizeof MGP); zero(MJO, sizeof MJO);
    zero(FJ, sizeof FJ); zero(&CMS, sizeof CMS); zero(&COBJ, sizeof COBJ); zero(&CD, sizeof CD);
    PTRMODE = 0;
    fill_HSD_JObj(&SJOB); SJOB.flags &= ~JOBJ_USE_QUATERNION;
    SGO.hsd_obj = &SJOB;
    fill_grFourside_YakumonoParam(&YP);
    YP.heli_wait = (int) (rnd() % 60); YP.heli_wait_add = (int) (rnd() % 40); YP.heli_stay_time = (int) (rnd() % 60);
    YP.crane_wait = (int) (rnd() % 50); YP.crane_wait_add = (int) (rnd() % 30); YP.crane_iron_wait = (int) (rnd() % 40);
    YP.crane_iron_wait_add = (int) (rnd() % 20);
    YP.crane_iron_up_min = (float) (rnd() % 20) * 0.5f; YP.crane_iron_up_max = (float) (rnd() % 60) * 0.5f;
    YP.crane_iron_down_min = (float) (rnd() % 20) * 0.5f; YP.crane_iron_down_max = (float) (rnd() % 60) * 0.5f;
    YP.crane_iron_spd = (float) (rnd() % 30) / 10.0f; YP.crane_iron_stop_acl = rw();
    YP.ufo_wait = (int) (rnd() % 60); YP.ufo_cs_offs = rw(); YP.ufo_stay_time = (int) (rnd() % 60);
    YP.ufo_stay_time_add = (int) (rnd() % 30); YP.ufo_challenge = (u16) (rnd() % 6); YP.x46 = (u16) (rnd() % 10);
    YP.x48 = (u16) (rnd() % 10);
    yakumono_param = &YP;
    fill_CmSubject(&CMS);
    fill_CollData(&CD);
    CD.x34_flags.b1234 = (rnd() & 1) ? 1 : fv();
    for (i = 0; i < 8; i++) {
        fill_HSD_GObj(&MGO[i]); fill_Ground(&MGP[i]); fill_HSD_JObj(&MJO[i]); MJO[i].flags &= ~JOBJ_USE_QUATERNION;
        MGO[i].user_data = &MGP[i];
        MGO[i].hsd_obj = &MJO[i];
    }
    for (i = 0; i < 10; i++) { fill_HSD_JObj(&FJ[i]); FJ[i].flags &= ~JOBJ_USE_QUATERNION; }
"""
_fs_setup_rs = STD_SETUP_RS + """    SJOB = core::mem::zeroed();
    YP = core::mem::zeroed();
    MGO = core::mem::zeroed();
    MGP = core::mem::zeroed();
    MJO = core::mem::zeroed();
    FJ = core::mem::zeroed();
    CMS = core::mem::zeroed();
    COBJ = core::mem::zeroed();
    CD = core::mem::zeroed();
    PTRMODE = 0;
    g::fill_HSD_JObj(&mut SJOB);
    SJOB.flags &= !(JOBJ_USE_QUATERNION as u32);
    SGO.hsd_obj = &raw mut SJOB as *mut _;
    g::fill_grFourside_YakumonoParam(&mut YP);
    YP.heli_wait = (rnd() % 60) as i32;
    YP.heli_wait_add = (rnd() % 40) as i32;
    YP.heli_stay_time = (rnd() % 60) as i32;
    YP.crane_wait = (rnd() % 50) as i32;
    YP.crane_wait_add = (rnd() % 30) as i32;
    YP.crane_iron_wait = (rnd() % 40) as i32;
    YP.crane_iron_wait_add = (rnd() % 20) as i32;
    YP.crane_iron_up_min = (rnd() % 20) as f32 * 0.5;
    YP.crane_iron_up_max = (rnd() % 60) as f32 * 0.5;
    YP.crane_iron_down_min = (rnd() % 20) as f32 * 0.5;
    YP.crane_iron_down_max = (rnd() % 60) as f32 * 0.5;
    YP.crane_iron_spd = (rnd() % 30) as f32 / 10.0;
    YP.crane_iron_stop_acl = rw();
    YP.ufo_wait = (rnd() % 60) as i32;
    YP.ufo_cs_offs = rw();
    YP.ufo_stay_time = (rnd() % 60) as i32;
    YP.ufo_stay_time_add = (rnd() % 30) as i32;
    YP.ufo_challenge = (rnd() % 6) as u16;
    YP.x46 = (rnd() % 10) as u16;
    YP.x48 = (rnd() % 10) as u16;
    m::yakumono_param = &raw mut YP;
    g::fill_CmSubject(&mut CMS);
    g::fill_CollData(&mut CD);
    CD.x34_flags.set_b1234(if rnd() & 1 != 0 { 1 } else { fv() as u8 });
    for i in 0..8 {
        g::fill_HSD_GObj(&mut MGO[i]);
        g::fill_Ground(&mut MGP[i]);
        g::fill_HSD_JObj(&mut MJO[i]);
        MJO[i].flags &= !(JOBJ_USE_QUATERNION as u32);
        MGO[i].user_data = &raw mut MGP[i] as *mut _;
        MGO[i].hsd_obj = &raw mut MJO[i] as *mut _;
    }
    for i in 0..10 {
        g::fill_HSD_JObj(&mut FJ[i]);
        FJ[i].flags &= !(JOBJ_USE_QUATERNION as u32);
    }
"""
_fs_pre_c = """    if (PTRMODE & 1) { HM(PIB(SGP.u.foursideUfo.xC)); SGP.u.foursideUfo.xC = 0; }
    if (PTRMODE & 2) {
        HM(PIB(SGP.u.fourside.x0)); HM(PIB(SGP.u.fourside.x4)); HM(PIB(SGP.u.fourside.x8));
        SGP.u.fourside.x0 = 0; SGP.u.fourside.x4 = 0; SGP.u.fourside.x8 = 0;
    }
    if (PTRMODE & 4) {
        HM(SGP.u.foursideCrane.x1.b0); HM(SGP.u.foursideCrane.x1.b1); HM(SGP.u.foursideCrane.x1.b2); HM(SGP.u.foursideCrane.x1.b3);
        HM(SGP.u.foursideCrane.x1.b4); HM(SGP.u.foursideCrane.x1.b5); HM(SGP.u.foursideCrane.x1.b6); HM(SGP.u.foursideCrane.x1.b7);
        ((unsigned char*) &SGP.u)[1] = 0;
    }
"""
_fs_pre_rs = """    if PTRMODE & 1 != 0 {
        hm(pib(SGP.u.foursideUfo.xC));
        SGP.u.foursideUfo.xC = null_mut();
    }
    if PTRMODE & 2 != 0 {
        hm(pib(SGP.u.fourside.x0));
        hm(pib(SGP.u.fourside.x4));
        hm(pib(SGP.u.fourside.x8));
        SGP.u.fourside.x0 = null_mut();
        SGP.u.fourside.x4 = null_mut();
        SGP.u.fourside.x8 = null_mut();
    }
    if PTRMODE & 4 != 0 {
        let x = &SGP.u.foursideCrane.x1;
        hm(x.b0() as u32);
        hm(x.b1() as u32);
        hm(x.b2() as u32);
        hm(x.b3() as u32);
        hm(x.b4() as u32);
        hm(x.b5() as u32);
        hm(x.b6() as u32);
        hm(x.b7() as u32);
        *(&raw mut SGP.u as *mut u8).add(1) = 0;
    }
"""
_fs_hash_c = _fs_pre_c + STD_HASH_C + """    hash_HSD_JObj(&SJOB); hash_grFourside_YakumonoParam(&YP); HM(PI(yakumono_param)); hash_CmSubject(&CMS); hash_CollData(&CD);
    for (i = 0; i < 8; i++) { hash_HSD_GObj(&MGO[i]); hash_Ground(&MGP[i]); hash_HSD_JObj(&MJO[i]); }
    for (i = 0; i < 10; i++) hash_HSD_JObj(&FJ[i]);
"""
_fs_hash_rs = _fs_pre_rs + STD_HASH_RS + """    g::hash_HSD_JObj(&SJOB);
    g::hash_grFourside_YakumonoParam(&YP);
    hm(pi(m::yakumono_param));
    g::hash_CmSubject(&CMS);
    g::hash_CollData(&CD);
    for i in 0..8 {
        g::hash_HSD_GObj(&MGO[i]);
        g::hash_Ground(&MGP[i]);
        g::hash_HSD_JObj(&MJO[i]);
    }
    for i in 0..10 {
        g::hash_HSD_JObj(&FJ[i]);
    }
"""
# per-view preparation of the stage gobj's union (C, Rust)
_bits_c = "".join(f"SGP.u.foursideCrane.x1.b{i} = rnd() & 1; " for i in range(8))
_bits_rs = "".join(f"SGP.u.foursideCrane.x1.set_b{i}((rnd() & 1) as u8); " for i in range(8))
_crane_c = ("PTRMODE |= 4; SGP.u.foursideCrane.x0 = (u8) (rnd() % 8); " + _bits_c + "SGP.u.foursideCrane.x4 = (int) (rnd() % 320) - 5; "
            "SGP.u.foursideCrane.x8 = (float) (rnd() % 200) / 10.0f - 10.0f; SGP.u.foursideCrane.xC = (float) (rnd() % 200) / 10.0f - 10.0f; "
            "SGP.u.foursideCrane.x10 = (float) (rnd() % 200) / 10.0f - 10.0f; SGP.u.foursideCrane.x14 = (float) (rnd() % 30) / 10.0f; "
            "SGP.u.foursideCrane.x18 = rw(); SGP.u.foursideCrane.x1C = rw(); ")
_crane_rs = ("PTRMODE |= 4; SGP.u.foursideCrane.x0 = (rnd() % 8) as u8; " + _bits_rs + "SGP.u.foursideCrane.x4 = (rnd() % 320) as i32 - 5; "
             "SGP.u.foursideCrane.x8 = (rnd() % 200) as f32 / 10.0 - 10.0; SGP.u.foursideCrane.xC = (rnd() % 200) as f32 / 10.0 - 10.0; "
             "SGP.u.foursideCrane.x10 = (rnd() % 200) as f32 / 10.0 - 10.0; SGP.u.foursideCrane.x14 = (rnd() % 30) as f32 / 10.0; "
             "SGP.u.foursideCrane.x18 = rw(); SGP.u.foursideCrane.x1C = rw(); ")
_ufo_c = ("PTRMODE |= 1; SGP.u.foursideUfo.x0 = (u8) (rnd() % 5); SGP.u.foursideUfo.x1 = (rnd() & 3) ? (u8) (rnd() % 3) : (u8) 0xFF; "
          "SGP.u.foursideUfo.x2 = (u8) (rnd() % 8); SGP.u.foursideUfo.x3 = (u8) (rnd() % 3); "
          "SGP.u.foursideUfo.x4 = (int) (rnd() % 80) - 5; SGP.u.foursideUfo.x8 = (int) (rnd() & 1); "
          "SGP.u.foursideUfo.xC = (rnd() & 1) ? &CMS : 0; ")
_ufo_rs = ("PTRMODE |= 1; SGP.u.foursideUfo.x0 = (rnd() % 5) as u8; SGP.u.foursideUfo.x1 = if rnd() & 3 != 0 { (rnd() % 3) as u8 } else { 0xFF }; "
           "SGP.u.foursideUfo.x2 = (rnd() % 8) as u8; SGP.u.foursideUfo.x3 = (rnd() % 3) as u8; "
           "SGP.u.foursideUfo.x4 = (rnd() % 80) as i32 - 5; SGP.u.foursideUfo.x8 = (rnd() & 1) as i32; "
           "SGP.u.foursideUfo.xC = if rnd() & 1 != 0 { &raw mut CMS } else { null_mut() }; ")
_heli_c = ("SGP.u.fourside2.x0 = (u8) (rnd() % 5); SGP.u.fourside2.x1 = (u8) (rnd() % 4); SGP.u.fourside2.x4 = (int) (rnd() % 900); "
           "SGP.u.fourside2.x8 = (int) (rnd() % 30); ")
_heli_rs = ("SGP.u.fourside2.x0 = (rnd() % 5) as u8; SGP.u.fourside2.x1 = (rnd() % 4) as u8; SGP.u.fourside2.x4 = (rnd() % 900) as i32; "
            "SGP.u.fourside2.x8 = (rnd() % 30) as i32; ")
_sky_c = ("PTRMODE |= 2; SGP.u.fourside.x0 = (rnd() & 1) ? &FJ[0] : 0; SGP.u.fourside.x4 = (rnd() & 1) ? &FJ[1] : 0; "
          "SGP.u.fourside.x8 = (rnd() & 1) ? &FJ[2] : 0; ")
_sky_rs = ("PTRMODE |= 2; SGP.u.fourside.x0 = if rnd() & 1 != 0 { &raw mut FJ[0] } else { null_mut() }; "
           "SGP.u.fourside.x4 = if rnd() & 1 != 0 { &raw mut FJ[1] } else { null_mut() }; "
           "SGP.u.fourside.x8 = if rnd() & 1 != 0 { &raw mut FJ[2] } else { null_mut() }; ")


def _op(name, pc, prs, fn, kind="gobj"):
    """kind: gobj -> void fn(gobj); pred -> ret = fn(gobj)"""
    if kind == "gobj":
        return (name, f"{pc}{fn}(&SGO);", f"{prs}m::{fn}(&raw mut SGO);")
    return (name, f"{pc}ret = (u32) {fn}(&SGO);", f"{prs}ret = m::{fn}(&raw mut SGO) as u32;")


_FSP = "grFourside_"
_fs_ops = [
    ("demoinit", "grFourside_801F2D0C((int) fv());", "m::grFourside_801F2D0C(fv() as i32);"),
    ("grFourside_801F2D10", "PTRMODE |= 7; grFourside_801F2D10();", "PTRMODE |= 7; m::grFourside_801F2D10();"), void_fn("grFourside_801F2DA0"), void_fn("grFourside_801F2DA4"),
    pred0_fn("grFourside_801F2DC8"),
    ("setup_gobj", "PTRMODE |= 7; ret = PI(grFourside_801F2DD0((int) (rnd() % 7)));", "PTRMODE |= 7; ret = pi(m::grFourside_801F2DD0((rnd() % 7) as i32));"),
    gobj_fn("stageGObj0_OnInit"), pred_fn("grFourside_801F2EE8"), gobj_fn("grFourside_801F2EF0"),
    gobj_fn("grFourside_801F2EF4"), gobj_fn("stageGObj2_OnInit"), pred_fn("grFourside_801F2F24"),
    gobj_fn("grFourside_801F2F2C"), gobj_fn("grFourside_801F2F30"),
    _op("2F34", "PTRMODE |= 2; ", "PTRMODE |= 2; ", "grFourside_801F2F34"),
    pred_fn("grFourside_801F3070"), gobj_fn("grFourside_801F3078"), gobj_fn("grFourside_801F309C"),
    ("30A0", "{ int j = (int) fv(); int x = (int) fv(); float d = rw(); grFourside_801F30A0(&SGP, j, &CD, x, (mpLib_GroundEnum) fv(), d); }",
     "{ let j = fv() as i32; let x = fv() as i32; let d = rw(); m::grFourside_801F30A0(&raw mut SGP as *mut _, j, &raw mut CD, x, fv() as _, d); }"),
    gobj_fn("grFourside_801F30F0"), pred_fn("grFourside_801F3144"), gobj_fn("grFourside_801F314C"),
    gobj_fn("grFourside_801F3150"),
    _op("3154", "PTRMODE |= 4; " + _bits_c, "PTRMODE |= 4; " + _bits_rs, "grFourside_801F3154"),
    pred_fn("grFourside_801F326C"),
    _op("3274", _crane_c, _crane_rs, "grFourside_801F3274"),
    gobj_fn("grFourside_801F37F8"),
    _op("37FC", "PTRMODE |= 1; ", "PTRMODE |= 1; ", "grFourside_801F37FC"),
    pred_fn("grFourside_801F388C"),
    _op("3894", _ufo_c, _ufo_rs, "grFourside_801F3894"),
    gobj_fn("grFourside_801F3B6C"),
    _op("3B70", _ufo_c, _ufo_rs, "grFourside_801F3B70"),
    gobj_fn("grFourside_801F3C40"), pred_fn("grFourside_801F3CC0"),
    _op("3CC8", _heli_c, _heli_rs, "grFourside_801F3CC8"),
    gobj_fn("grFourside_801F3F0C"),
    ("3F10", "MGP[3].u.fourside2.x0 = (u8) (rnd() % 5); MGP[3].u.fourside2.x1 = (u8) (rnd() % 3); "
     "MGP[3].u.fourside2.x4 = 695 + (int) (rnd() % 12); ret = (u32) grFourside_801F3F10();",
     "MGP[3].u.fourside2.x0 = (rnd() % 5) as u8; MGP[3].u.fourside2.x1 = (rnd() % 3) as u8; "
     "MGP[3].u.fourside2.x4 = 695 + (rnd() % 12) as i32; ret = m::grFourside_801F3F10() as u32;"),
    ("render_cb", _sky_c + "fn_801F3F74(&SGO, (int) (rnd() % 3));", _sky_rs + "m::fn_801F3F74(&raw mut SGO, (rnd() % 3) as i32);"),
    ("touchline", "ret = PI(grFourside_801F41E0((enum_t) fv()));", "ret = pi(m::grFourside_801F41E0(fv() as _));"),
    ("shadow", "ret = (u32) grFourside_801F41E8(&SV, (int) fv(), (HSD_JObj*) SJO);",
     "ret = m::grFourside_801F41E8(&raw mut SV, fv() as i32, &raw mut SJO as *mut _) as u32;"),
    ("crane_seq",
     "{ int j; PTRMODE |= 4; " + _bits_c + "grFourside_801F3154(&SGO); for (j = 0; j < 900; j++) grFourside_801F3274(&SGO); }",
     "{ PTRMODE |= 4; " + _bits_rs + "m::grFourside_801F3154(&raw mut SGO); for _ in 0..900 { m::grFourside_801F3274(&raw mut SGO); } }"),
    ("ufo_seq",
     "{ int j; PTRMODE |= 1; grFourside_801F37FC(&SGO); for (j = 0; j < 500; j++) { grFourside_801F3894(&SGO); "
     "if ((rnd() & 7) == 0) { MGP[5].u.fourside2.x8 += 1; } } }",
     "{ PTRMODE |= 1; m::grFourside_801F37FC(&raw mut SGO); for _ in 0..500 { m::grFourside_801F3894(&raw mut SGO); "
     "if rnd() & 7 == 0 { MGP[5].u.fourside2.x8 += 1; } } }"),
    ("heli_seq",
     "{ int j; grFourside_801F3C40(&SGO); for (j = 0; j < 700; j++) { grFourside_801F3CC8(&SGO); "
     "if ((rnd() & 63) == 0) { MGP[5].u.fourside2.x0 = (u8) (rnd() % 3); MGP[5].u.fourside2.x1 = (u8) (rnd() % 3); } } }",
     "{ m::grFourside_801F3C40(&raw mut SGO); for _ in 0..700 { m::grFourside_801F3CC8(&raw mut SGO); "
     "if rnd() & 63 == 0 { MGP[5].u.fourside2.x0 = (rnd() % 3) as u8; MGP[5].u.fourside2.x1 = (rnd() % 3) as u8; } } }"),
]
CONFIGS["grfourside"] = dict(
    stem="grfourside", c_decl=_fs_c_decl, rs_decl=_fs_rs_decl,
    pregs=list(STD_PREG) + [
        ("&SJOB", "sizeof SJOB", "&raw const SJOB", "core::mem::size_of::<HSD_JObj>()"),
        ("&YP", "sizeof YP", "&raw const YP", "core::mem::size_of::<m::grFourside_YakumonoParam>()"),
        ("MGO", "sizeof MGO", "&raw const MGO", "core::mem::size_of_val(&MGO)"),
        ("MGP", "sizeof MGP", "&raw const MGP", "core::mem::size_of_val(&MGP)"),
        ("MJO", "sizeof MJO", "&raw const MJO", "core::mem::size_of_val(&MJO)"),
        ("FJ", "sizeof FJ", "&raw const FJ", "core::mem::size_of_val(&FJ)"),
        ("&CMS", "sizeof CMS", "&raw const CMS", "core::mem::size_of::<CmSubject>()"),
        ("&COBJ", "16", "&raw const COBJ", "16"),
        ("&CD", "sizeof CD", "&raw const CD", "core::mem::size_of::<CollData>()"),
        ("grFs_StageCallbacks", "sizeof grFs_StageCallbacks", "&raw const m::grFs_StageCallbacks",
         "core::mem::size_of_val(&m::grFs_StageCallbacks)"),
        ("&grFs_StageData", "sizeof grFs_StageData", "&raw const m::grFs_StageData",
         "core::mem::size_of_val(&m::grFs_StageData)"),
        ("grFs_803E3CE8", "sizeof grFs_803E3CE8", "&raw const m::grFs_803E3CE8", "core::mem::size_of_val(&m::grFs_803E3CE8)")],
    c_extra="", rs_extra="", setup_c=_fs_setup_c, setup_rs=_fs_setup_rs, hash_c=_fs_hash_c, hash_rs=_fs_hash_rs,
    ops=_fs_ops + [table_op("grFs_StageCallbacks", 7, "grFs_StageData")], extra_freg=[])
