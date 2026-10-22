"""Oracle spec of tu::grmutecity (see gen_drv.py for the keys)."""
CHR = chr(10)

# --- the world around the TU's statics ---------------------------------------------------------
# GO[2] is the car field object: its jobj has a 30-long chain below it (grMuteCity_801F1A34 walks it).
# Lights: PLink[3] -> LGO[0..2] (classifier 0xC for some), all with the LObj chain LP[0..5].
SETUP_C = """NG = 0; zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
zero(&YP, sizeof YP); zero(YPX, sizeof YPX); zero(DD, sizeof DD); zero(&ARCH, sizeof ARCH); zero(&USD, sizeof USD);
zero(X8, sizeof X8); zero(SPL, sizeof SPL); zero(GEN, sizeof GEN); zero(APS, sizeof APS); zero(LP, sizeof LP);
zero(LGO, sizeof LGO); zero(CJ, sizeof CJ); zero(PL, sizeof PL); zero(&GOX, sizeof GOX); zero(&GRX, sizeof GRX); GOX.user_data = &GRX; GOX.hsd_obj = &JP[0];
YP.x0 = &YPX[0]; YP.x4 = &YPX[1]; YP.x8 = &DD[0]; YP.xC = &DD[1];
YP.x2C = (float) (fv() % 300); YP.x30 = (float) (fv() % 400); YP.x34 = 0.01f * (float) (fv() % 20);
YP.x38 = 0.01f * (float) (fv() % 20); YP.x3C = 0.01f * (float) (fv() % 20); YP.x40 = 0.001f * (float) (fv() % 20);
YP.x44 = 0.05f * (float) (fv() % 10); YP.x48 = 0.05f * (float) (fv() % 10); YP.x4C = 0.001f * (float) (fv() % 20);
yakumono_param = &YP;
ARCH.unk4 = ((fv() & 3) == 0) ? 0 : &USD;
USD.unk8 = X8;
for (i = 0; i < 16; i++) JP[i].u.spline = &SPL[fv() & 1];
for (i = 0; i < 31; i++) { fill_HSD_JObj(&CJ[i]); CJ[i].flags &= ~JOBJ_USE_QUATERNION; CJ[i].u.spline = &SPL[fv() & 1]; CJ[i].child = 0; CJ[i].next = 0; }
CJ[0].child = &CJ[1];
for (i = 1; i < 30; i++) CJ[i].next = &CJ[i + 1];
GO[2].hsd_obj = &CJ[0];
for (i = 0; i < 6; i++) { LP[i].flags = (u16) fv(); LP[i].next = (i + 1 < 6) ? &LP[i + 1] : 0; }
for (i = 0; i < 3; i++) { LGO[i].classifier = (fv() % 3 == 0) ? 0xC : (u16) (fv() % 16); LGO[i].next = (i + 1 < 3) ? &LGO[i + 1] : 0; LGO[i].hsd_obj = &LP[0]; }
PL[3] = ((fv() % 5) == 0) ? 0 : &LGO[0];
HSD_GObjPLinkHead = PL;
grMc_804D69D4 = (s32) (fv() % 3);
{
    int k;
    for (k = 0; k < 30; k++) {
        grMc_CarEntry* e = &grMc_8049F4B8[k];
        e->x0 = (float) (fv() % 1100) / 1000.0f; e->x4 = (float) (fv() % 1100) / 1000.0f; e->x8 = rg() * 0.01f;
        e->xC = (float) (fv() % 1100) / 1000.0f; e->x10 = (float) (fv() % 400);
        e->pos.x = rg() * 20.0f; e->pos.y = rg() * 20.0f; e->pos.z = rg() * 20.0f;
        e->x20 = (u16) (fv() % 0x40); e->x22_flags.b0 = (fv() % 3 == 0); e->x22_flags.b1 = (fv() & 1); e->x23 = (u8) fv();
        e->x24 = (fv() % 3 == 0) ? 0 : &GO[fv() & 3];
        e->gen = (fv() % 3 == 0) ? 0 : &GEN[fv() & 1];
        grMc_8049F440[k] = k;
    }
    for (k = 29; k > 0; k--) { u32 s = fv() % (u32) (k + 1); s32 t = grMc_8049F440[k]; grMc_8049F440[k] = grMc_8049F440[s]; grMc_8049F440[s] = t; }
    grMc_8049F440[-1] = (s32) (fv() % 30);
    for (k = 0; k < 2; k++) {
        GEN[k].appsrt = (fv() % 3 == 0) ? 0 : &APS[fv() & 1]; GEN[k].type = (u16) fv();
        APS[k].xA2 = (u8) fv(); APS[k].scale.x = rg(); APS[k].scale.y = rg(); APS[k].scale.z = rg(); APS[k].gp = 0;
    }
}"""
SETUP_R = """NG = false; m::ext::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
YP = core::mem::zeroed(); YPX = [0; 2]; DD = core::mem::zeroed(); ARCH = core::mem::zeroed(); USD = core::mem::zeroed();
X8 = core::mem::zeroed(); SPL = core::mem::zeroed(); GEN = core::mem::zeroed(); APS = core::mem::zeroed(); LP = core::mem::zeroed();
LGO = core::mem::zeroed(); CJ = core::mem::zeroed(); PL = core::mem::zeroed(); GOX = core::mem::zeroed(); GRX = core::mem::zeroed(); GOX.user_data = (&raw mut GRX) as *mut c_void; GOX.hsd_obj = (&raw mut JP[0]) as *mut c_void;
YP.x0 = (&raw mut YPX[0]) as *mut c_void; YP.x4 = (&raw mut YPX[1]) as *mut c_void; YP.x8 = &raw mut DD[0]; YP.xC = &raw mut DD[1];
YP.x2C = (fv() % 300) as f32; YP.x30 = (fv() % 400) as f32; YP.x34 = 0.01f32 * (fv() % 20) as f32;
YP.x38 = 0.01f32 * (fv() % 20) as f32; YP.x3C = 0.01f32 * (fv() % 20) as f32; YP.x40 = 0.001f32 * (fv() % 20) as f32;
YP.x44 = 0.05f32 * (fv() % 10) as f32; YP.x48 = 0.05f32 * (fv() % 10) as f32; YP.x4C = 0.001f32 * (fv() % 20) as f32;
m::yakumono_param = &raw mut YP;
ARCH.unk4 = if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut USD };
USD.unk8 = (&raw mut X8) as *mut UnkStageDat_x8_t;
for i in 0..16usize { JP[i].u.spline = &raw mut SPL[(fv() & 1) as usize]; }
for i in 0..31usize { g::fill_HSD_JObj(&mut CJ[i]); CJ[i].flags &= !(JOBJ_USE_QUATERNION as u32); CJ[i].u.spline = &raw mut SPL[(fv() & 1) as usize]; CJ[i].child = core::ptr::null_mut(); CJ[i].next = core::ptr::null_mut(); }
CJ[0].child = &raw mut CJ[1];
for i in 1..30usize { CJ[i].next = &raw mut CJ[i + 1]; }
GO[2].hsd_obj = (&raw mut CJ[0]) as *mut c_void;
for i in 0..6usize { LP[i].flags = fv() as u16; LP[i].next = if i + 1 < 6 { &raw mut LP[i + 1] } else { core::ptr::null_mut() }; }
for i in 0..3usize { LGO[i].classifier = if fv() % 3 == 0 { 0xC } else { (fv() % 16) as u16 }; LGO[i].next = if i + 1 < 3 { &raw mut LGO[i + 1] } else { core::ptr::null_mut() }; LGO[i].hsd_obj = (&raw mut LP[0]) as *mut c_void; }
PL[3] = if fv() % 5 == 0 { core::ptr::null_mut() } else { &raw mut LGO[0] };
m::ext::HSD_GObjPLinkHead = (&raw mut PL) as *mut *mut HSD_GObj;
m::grMc_804D69D4 = (fv() % 3) as i32;
{
    for k in 0..30usize {
        let e = &mut m::grMc_8049F4B8[k];
        e.x0 = (fv() % 1100) as f32 / 1000.0f32; e.x4 = (fv() % 1100) as f32 / 1000.0f32; e.x8 = rg() * 0.01f32;
        e.xC = (fv() % 1100) as f32 / 1000.0f32; e.x10 = (fv() % 400) as f32;
        e.pos.x = rg() * 20.0f32; e.pos.y = rg() * 20.0f32; e.pos.z = rg() * 20.0f32;
        e.x20 = (fv() % 0x40) as u16; e.x22_flags.set_b0((fv() % 3 == 0) as u8); e.x22_flags.set_b1((fv() & 1) as u8); e.x23 = fv() as u8;
        e.x24 = if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut GO[(fv() & 3) as usize] };
        e.r#gen = if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut GEN[(fv() & 1) as usize] };
        *m::grMc_8049F440().add(k) = k as i32;
    }
    for k in (1..30usize).rev() { let s = (fv() % (k as u32 + 1)) as usize; let t = *m::grMc_8049F440().add(k); *m::grMc_8049F440().add(k) = *m::grMc_8049F440().add(s); *m::grMc_8049F440().add(s) = t; }
    *m::grMc_8049F440().offset(-1) = (fv() % 30) as i32;
    for k in 0..2usize {
        GEN[k].appsrt = if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut APS[(fv() & 1) as usize] }; GEN[k].r#type = fv() as u16;
        APS[k].xA2 = fv() as u8; APS[k].scale.x = rg(); APS[k].scale.y = rg(); APS[k].scale.z = rg(); APS[k].gp = core::ptr::null_mut();
    }
}"""

HASH_C = """hash_StageInfo(&stage_info); HM(PI(yakumono_param)); HM((u32) grMc_804D69D4); HM(B(GRX.u.mutecity2.xCC));
{
    int k;
    for (k = 0; k < 30; k++) {
        grMc_CarEntry* e = &grMc_8049F4B8[k];
        HM(B(e->x0)); HM(B(e->x4)); HM(B(e->x8)); HM(B(e->xC)); HM(B(e->x10)); HM(B(e->pos.x)); HM(B(e->pos.y)); HM(B(e->pos.z));
        HM(e->x20); HM(e->x22_flags.b0); HM(e->x22_flags.b1); HM(e->x23); HM(PI(e->x24)); HM(PI(e->gen)); HM((u32) grMc_8049F440[k]);
    }
    HM((u32) grMc_8049F440[-1]);
    for (k = 0; k < 31; k++) {
        HM(CJ[k].flags); HM(B(CJ[k].translate.x)); HM(B(CJ[k].translate.y)); HM(B(CJ[k].translate.z));
        HM(B(CJ[k].rotate.x)); HM(B(CJ[k].rotate.y)); HM(B(CJ[k].rotate.z)); HM(B(CJ[k].rotate.w));
        HM(B(CJ[k].scale.x)); HM(B(CJ[k].scale.y)); HM(B(CJ[k].scale.z));
    }
    for (k = 0; k < 2; k++) {
        HM(PI(GEN[k].appsrt)); HM(GEN[k].type); HM(PI(APS[k].gp)); HM(APS[k].xA2); HM(B(APS[k].scale.x)); HM(B(APS[k].scale.y)); HM(B(APS[k].scale.z));
    }
}"""
HASH_R = """g::hash_StageInfo(&m::ext::stage_info); hm(pi(m::yakumono_param)); hm(m::grMc_804D69D4 as u32); hm(GRX.u.mutecity2.xCC.to_bits());
{
    for k in 0..30usize {
        let e = &m::grMc_8049F4B8[k];
        hm(e.x0.to_bits()); hm(e.x4.to_bits()); hm(e.x8.to_bits()); hm(e.xC.to_bits()); hm(e.x10.to_bits()); hm(e.pos.x.to_bits()); hm(e.pos.y.to_bits()); hm(e.pos.z.to_bits());
        hm(e.x20 as u32); hm(e.x22_flags.b0() as u32); hm(e.x22_flags.b1() as u32); hm(e.x23 as u32); hm(pi(e.x24)); hm(pi(e.r#gen)); hm(*m::grMc_8049F440().add(k) as u32);
    }
    hm(*m::grMc_8049F440().offset(-1) as u32);
    for k in 0..31usize {
        hm(CJ[k].flags); hm(CJ[k].translate.x.to_bits()); hm(CJ[k].translate.y.to_bits()); hm(CJ[k].translate.z.to_bits());
        hm(CJ[k].rotate.x.to_bits()); hm(CJ[k].rotate.y.to_bits()); hm(CJ[k].rotate.z.to_bits()); hm(CJ[k].rotate.w.to_bits());
        hm(CJ[k].scale.x.to_bits()); hm(CJ[k].scale.y.to_bits()); hm(CJ[k].scale.z.to_bits());
    }
    for k in 0..2usize {
        hm(pi(GEN[k].appsrt)); hm(GEN[k].r#type as u32); hm(pi(APS[k].gp)); hm(APS[k].xA2 as u32); hm(APS[k].scale.x.to_bits()); hm(APS[k].scale.y.to_bits()); hm(APS[k].scale.z.to_bits());
    }
}"""

# the view fields the commands / car code read must point at the right objects and stay in range
MC_C = ("for (i = 0; i < 4; i++) { GR[i].u.mutecity.xC4 = (s16) (fv() % 132); GR[i].u.mutecity.xC6 = (fv() % 3 == 0) ? (s16) (fv() % 4) : 0; }")
MC_R = ("for i in 0..4usize { GR[i].u.mutecity.xC4 = (fv() % 132) as i16; GR[i].u.mutecity.xC6 = if fv() % 3 == 0 { (fv() % 4) as i16 } else { 0 }; }")
CAR2_C = "GR[2].u.mutecity.xC4 = (s16) (fv() % 10); for (i = 0; i < 4; i++) { GR[i].u.mutecity.xCC = &GO[2]; GR[i].u.mutecity.xC8 = &GOX; }"
CAR2_R = "GR[2].u.mutecity.xC4 = (fv() % 10) as i16; for i in 0..4usize { GR[i].u.mutecity.xCC = &raw mut GO[2]; GR[i].u.mutecity.xC8 = &raw mut GOX; }"
CMD_C = MC_C + chr(10) + CAR2_C
CMD_R = MC_R + chr(10) + CAR2_R
COLL_C = ("CD.x0_gobj = (fv() % 5 == 0) ? 0 : &GO[fv() & 3]; for (i = 0; i < 4; i++) GO[i].p_link = (fv() % 3 == 0) ? 9 : (u8) (fv() % 16);")
COLL_R = ("CD.x0_gobj = if fv() % 5 == 0 { core::ptr::null_mut() } else { &raw mut GO[(fv() & 3) as usize] }; for i in 0..4usize { GO[i].p_link = if fv() % 3 == 0 { 9 } else { (fv() % 16) as u8 }; }")

IDARG = ("(s32) (fv() % 30)", "(fv() % 30) as i32")
SPLARG = ("&SPL[fv() & 1]", "&raw mut SPL[(fv() & 1) as usize]")
VECARG = ("&V3[fv() & 3]", "&raw mut V3[(fv() & 3) as usize]")
JOBJARG = ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]")
NULLORGEN = ("((fv() & 3) == 0) ? 0 : &GEN[fv() & 1]", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GEN[(fv() & 1) as usize] }")

SPEC = dict(
    ptr_unions={"u": "spline"},
    src="src/melee/gr/grmutecity.c",
    # arr[-1] of grMuteCity_801F1328 (the word in front of grMc_8049F440, another TU's bss in the retail
    # layout): a real word in front of the array, as in tu::grmutecity
    patch=[("static s32 grMc_8049F440[30];", "static s32 grMc_8049F43C_pad_[31];" + CHR + "#define grMc_8049F440 (grMc_8049F43C_pad_ + 1)")],
    ops=[("grMuteCity_801EFDF8_fields", """NG = 0;
grMuteCity_801EFDF8(&GO[3]);
{
    Ground* g = &GR[3];
    HM((u32) g->u.mutecity.xC4); HM((u32) g->u.mutecity.xC6); HM(PI(g->u.mutecity.xC8)); HM(PI(g->u.mutecity.xCC));
    HM(g->u.mutecity.xD0_flags.b0); HM(g->u.mutecity.xD0_flags.b1); HM(g->u.mutecity.xD0_flags.b23); HM((u32) g->u.mutecity.xD2);
    HM(B(g->u.mutecity.xD4)); HM(B(g->u.mutecity.xD8)); HM(PI(g->u.mutecity.xDC)); HM(PI(g->u.mutecity.xE0));
    HM(B(g->u.mutecity.xE4.x)); HM(B(g->u.mutecity.xE4.y)); HM(B(g->u.mutecity.xE4.z)); HM(B(g->u.mutecity.xF0.x)); HM(B(g->u.mutecity.xF0.y)); HM(B(g->u.mutecity.xF0.z));
    HM(PI(g->u.mutecity.xFC)); HM(PI(g->u.mutecity.x100)); HM(PI(g->u.mutecity.x104)); HM(PI(g->u.mutecity.x108)); HM(PI(g->u.mutecity.x10C)); HM(PI(g->u.mutecity.x110));
    HM(B(g->u.mutecity.x114)); HM(B(g->u.mutecity.x118)); HM(B(g->u.mutecity.x11C)); HM(B(g->u.mutecity.x120));
    HM(B(g->u.mutecity.x124)); HM(B(g->u.mutecity.x128)); HM(B(g->u.mutecity.x12C)); HM(B(g->u.mutecity.x130));
}
{
    Ground* g = &GR[1];
    HM(g->u.mutecity2.xC4_flags.b0); HM(PI(g->u.mutecity2.xC8)); HM(B(g->u.mutecity2.xCC)); HM(B(g->u.mutecity2.xD0));
}
HM((u32) GR[2].u.mutecity.xC4);
ret = 0;""", """NG = false;
m::grMuteCity_801EFDF8(&raw mut GO[3]);
{
    let g = &GR[3];
    hm(g.u.mutecity.xC4 as u32); hm(g.u.mutecity.xC6 as u32); hm(pi(g.u.mutecity.xC8)); hm(pi(g.u.mutecity.xCC));
    hm(g.u.mutecity.xD0_flags.b0() as u32); hm(g.u.mutecity.xD0_flags.b1() as u32); hm(g.u.mutecity.xD0_flags.b23() as u32); hm(g.u.mutecity.xD2 as u32);
    hm(g.u.mutecity.xD4.to_bits()); hm(g.u.mutecity.xD8.to_bits()); hm(pi(g.u.mutecity.xDC)); hm(pi(g.u.mutecity.xE0));
    hm(g.u.mutecity.xE4.x.to_bits()); hm(g.u.mutecity.xE4.y.to_bits()); hm(g.u.mutecity.xE4.z.to_bits()); hm(g.u.mutecity.xF0.x.to_bits()); hm(g.u.mutecity.xF0.y.to_bits()); hm(g.u.mutecity.xF0.z.to_bits());
    hm(pi(g.u.mutecity.xFC)); hm(pi(g.u.mutecity.x100)); hm(pi(g.u.mutecity.x104)); hm(pi(g.u.mutecity.x108)); hm(pi(g.u.mutecity.x10C)); hm(pi(g.u.mutecity.x110));
    hm(g.u.mutecity.x114.to_bits()); hm(g.u.mutecity.x118.to_bits()); hm(g.u.mutecity.x11C.to_bits()); hm(g.u.mutecity.x120.to_bits());
    hm(g.u.mutecity.x124.to_bits()); hm(g.u.mutecity.x128.to_bits()); hm(g.u.mutecity.x12C.to_bits()); hm(g.u.mutecity.x130.to_bits());
}
{
    let g = &GR[1];
    hm(g.u.mutecity2.xC4_flags.b0() as u32); hm(pi(g.u.mutecity2.xC8)); hm(g.u.mutecity2.xCC.to_bits()); hm(g.u.mutecity2.xD0.to_bits());
}
hm(GR[2].u.mutecity.xC4 as u32);
ret = 0;""", (-1, -1))],
    skip={"grMc_StackPad": "empty fake function taking a struct by value (no driver arg type); called from grMuteCity_801F290C"},
    skip_stubs=["__frsqrte", "sinf", "cosf"],  # the game's gekko_fp.c is linked into the oracle (and melee_runtime::gekko in the port)
    types=["StageInfo", "StageCallbacks", "StageData"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "Ground_GetMapGObj", "Ground_801C13D0", "Ground_801C498C",
                 "lbVector_Add", "lbVector_Diff", "lbVector_Sub", "psAddGeneratorAppSRT_begin", "grLib_801C96F8", "grLib_801C9808",
                 "grMaterial_801C8CFC", "Ground_801C3FA4", "Ground_GetStageGObj"],
    pools={
        "HSD_JObj": ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]"),
        "HSD_LObj": ("&LP[fv() % 6]", "&raw mut LP[(fv() % 6) as usize]"),
        "HSD_Spline": SPLARG,
    },
    ret={
        "Ground_GetStageGObj": ("((NG && (fv() & 3) == 0) ? 0 : &GO[a0 == 30 ? 3 : a0 == 29 ? 1 : (a0 & 3)])",
                                "if super::NG && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[if a0 == 30 { 3 } else if a0 == 29 { 1 } else { (a0 & 3) as usize }] }"),
        "Ground_GetMapGObj": ("((fv() & 3) == 0 ? 0 : &GO[fv() & 3])",
                              "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "Ground_801C3FA4": ("&JP[fv() % 16]", "&raw mut super::JP[(fv() % 16) as usize]"),
        "Ground_801C13D0": ("((fv() & 3) == 0 ? 0 : &JP[fv() % 16])",
                            "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::JP[(fv() % 16) as usize] }"),
        "Ground_801C498C": ("((fv() & 3) == 0 ? 0 : &LGO[0])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::LGO[0] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_801C3F20": ("(float) (fv() % 4000)", "(fv() % 4000) as f32"),
        "grDatFiles_801C6330": ("&ARCH", "&raw mut super::ARCH"),
        "grLib_801C9808": NULLORGEN,
        "grLib_801C96F8": NULLORGEN,
        "psAddGeneratorAppSRT_begin": ("((fv() & 3) == 0) ? 0 : &APS[fv() & 1]",
                                       "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::APS[(fv() & 1) as usize] }"),
        "grMaterial_801C8CFC": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]",
                                "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
    },
    effects={
        "Camera_GetTransformPosition": ("WV(a0);", "super::wv(a0);"),
        "Stage_UnkSetVec3TCam_Offset": ("WV(a0);", "super::wv(a0);"),
        "lbVector_Add": ("TV(a0); TV(a1); WV(a0);", "super::tv(a0); super::tv(a1); super::wv(a0);"),
        "lbVector_Sub": ("TV(a0); TV(a1); WV(a0);", "super::tv(a0); super::tv(a1); super::wv(a0);"),
        "lbVector_Diff": ("TV(a0); TV(a1); WV(a2);", "super::tv(a0); super::tv(a1); super::wv(a2);"),
        "lbVector_Normalize": ("TV(a0); WV(a0);", "super::tv(a0); super::wv(a0);"),
        "PSVECCrossProduct": ("TV(a0); TV(a1); WV(a2);", "super::tv(a0); super::tv(a1); super::wv(a2);"),
        "Ground_801C5AEC": ("TV(a1); TV(a2); TV(a3); WV(a0);", "super::tv(a1); super::tv(a2); super::tv(a3); super::wv(a0);"),
        "splGetSplinePoint": ("WV(a0);", "super::wv(a0);"),
        "lbShadow_8000E9F0": ("WV(a0);", "super::wv(a0);"),
        "lb_8000B1CC": ("TV(a1); WV(a2);", "super::tv(a1); super::wv(a2);"),
        "grLib_801C96F8": ("TV(a2);", "super::tv(a2);"),
        "HSD_LObjGetColor": ("a1->r = (u8) rnd(); a1->g = (u8) rnd(); a1->b = (u8) rnd(); a1->a = (u8) rnd();",
                             "(*a1).r = rnd() as u8; (*a1).g = rnd() as u8; (*a1).b = rnd() as u8; (*a1).a = rnd() as u8;"),
    },
    decl_c=CHR.join([
        "static struct grMc_YakumonoParam YP;", "static u8 YPX[2];", "static DynamicsDesc DD[2];", "static UnkArchiveStruct ARCH;",
        "static UnkStageDat USD;", "static struct UnkStageDat_x8_t X8[40];", "static HSD_Spline SPL[2];", "static HSD_Generator GEN[2];",
        "static HSD_psAppSRT APS[2];", "static HSD_LObj LP[6];", "static HSD_GObj LGO[3];", "static HSD_JObj CJ[31];",
        "static HSD_GObj* PL[16];", "static int NG;", "static HSD_GObj GOX;", "static Ground GRX;"]),
    decl_r=CHR.join([
        "static mut YP: m::GrMcYakumonoParam = unsafe { core::mem::zeroed() };", "static mut YPX: [u8; 2] = [0; 2];",
        "static mut DD: [DynamicsDesc; 2] = unsafe { core::mem::zeroed() };", "static mut ARCH: UnkArchiveStruct = unsafe { core::mem::zeroed() };",
        "static mut USD: UnkStageDat = unsafe { core::mem::zeroed() };", "static mut X8: [UnkStageDat_x8_t; 40] = unsafe { core::mem::zeroed() };",
        "static mut SPL: [HSD_Spline; 2] = unsafe { core::mem::zeroed() };", "static mut GEN: [HSD_Generator; 2] = unsafe { core::mem::zeroed() };",
        "static mut APS: [HSD_psAppSRT; 2] = unsafe { core::mem::zeroed() };", "static mut LP: [HSD_LObj; 6] = unsafe { core::mem::zeroed() };",
        "static mut LGO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };", "static mut CJ: [HSD_JObj; 31] = unsafe { core::mem::zeroed() };",
        "static mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];", "static mut NG: bool = false;", "static mut GOX: HSD_GObj = unsafe { core::mem::zeroed() };", "static mut GRX: Ground = unsafe { core::mem::zeroed() };"]),
    setup_c=SETUP_C,
    setup_r=SETUP_R,
    hash_c=HASH_C,
    hash_r=HASH_R,
    preg_c=("preg(&YP, sizeof YP); preg(YPX, sizeof YPX); preg(DD, sizeof DD); preg(&ARCH, sizeof ARCH); preg(&USD, sizeof USD); preg(X8, sizeof X8); "
            "preg(SPL, sizeof SPL); preg(GEN, sizeof GEN); preg(APS, sizeof APS); preg(LP, sizeof LP); preg(LGO, sizeof LGO); preg(CJ, sizeof CJ); "
            "preg(PL, sizeof PL); preg(&GOX, sizeof GOX); preg(grMc_StageCallbacks, sizeof grMc_StageCallbacks); preg(grMc_803E30B0, sizeof grMc_803E30B0);"),
    preg_r=("preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const YPX, 2); preg(&raw const DD, core::mem::size_of_val(&DD)); "
            "preg(&raw const ARCH, core::mem::size_of_val(&ARCH)); preg(&raw const USD, core::mem::size_of_val(&USD)); preg(&raw const X8, core::mem::size_of_val(&X8)); "
            "preg(&raw const SPL, core::mem::size_of_val(&SPL)); preg(&raw const GEN, core::mem::size_of_val(&GEN)); preg(&raw const APS, core::mem::size_of_val(&APS)); "
            "preg(&raw const LP, core::mem::size_of_val(&LP)); preg(&raw const LGO, core::mem::size_of_val(&LGO)); preg(&raw const CJ, core::mem::size_of_val(&CJ)); "
            "preg(&raw const PL, core::mem::size_of_val(&PL)); preg(&raw const GOX, core::mem::size_of_val(&GOX)); preg(&raw const m::grMc_StageCallbacks, core::mem::size_of_val(&m::grMc_StageCallbacks)); "
            "preg(&raw const m::grMc_803E30B0, core::mem::size_of_val(&m::grMc_803E30B0));"),
    data=[("grMc_StageCallbacks", "StageCallbacks", 39), ("grMc_StageData", "StageData", None),
          ("grMc_803E34A4", "raw", None), ("grMc_803E34E0", "raw", None), ("grMc_803E3B7C", "raw", None), ("grMc_803E30B0", "raw", None)],
    view_null={"x110": 4},
    # the dispatchers run on_init callbacks that overlay different views of Ground::u: no Ground::u in their fill / hash
    view={"grMuteCity_801EFC6C": "-", "grMuteCity_801EFD0C": "-", "grMuteCity_801EFDF8": "-"},
    hview={"grMuteCity_801EFC6C": "-", "grMuteCity_801EFD0C": "-", "grMuteCity_801EFDF8": "-"},
    args={
        "grMuteCity_801EFD0C": [("(int) ({ u32 v_ = fv() % 39; v_ == 30 ? 31 : v_; })", "{ let v = (fv() % 39) as i32; if v == 30 { 31 } else { v } }")],
        "grMuteCity_801F1A34": [("&GO[2]", "&raw mut GO[2]"), ("&GO[3]", "&raw mut GO[3]")],
        "grMuteCity_801EFDF8": [("&GO[3]", "&raw mut GO[3]")],
        "grMuteCity_801F0120": [("&GO[3]", "&raw mut GO[3]")],
        "grMuteCity_801F04B8": [("&GO[3]", "&raw mut GO[3]")],
        "grMuteCity_801F094C": [JOBJARG, SPLARG, VECARG, ("rg()", "rg()")],
        "grMc_SplineEvalWrapped": [VECARG, SPLARG, ("rg()", "rg()")],
        "grMc_SplineTangentWrapped": [VECARG, SPLARG, ("rg()", "rg()")],
        "grMuteCity_801F106C": [IDARG],
        "grMuteCity_801F2C10": [VECARG, ("(int) (fv() % 8)", "(fv() % 8) as i32"), ("&JP[fv() % 4]", "&raw mut JP[(fv() % 4) as usize]")],
        "grMuteCity_801F2BBC": [("(enum_t) (0x2F + fv() % 8)", "(0x2F + fv() % 8) as i32")],
        "fn_801F2B58": [("(void*) &GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize] as *mut c_void"), ("(int) (fv() % 8)", "(fv() % 8) as i32"),
                        ("&CD", "&raw mut CD"), ("(int) (fv() % 8)", "(fv() % 8) as i32"),
                        ("(mpLib_GroundEnum) (fv() % 4)", "fv() % 4"), ("rg()", "rg()")],
        "grMuteCity_801F2AB0": [("(s32) ({ u32 v_ = fv() % 4; v_ == 0 ? 0x116 : v_ == 1 ? 0x119 : (s32) (fv() % 2000); })",
                                 "{ let v = fv() % 4; if v == 0 { 0x116 } else if v == 1 { 0x119 } else { (fv() % 2000) as i32 } }"), JOBJARG],
    },
    pre={
        "grMuteCity_801F2C10": ("for (i = 0; i < 4; i++) { Ground* g = &GR[i]; g->u.mutecity.xFC = &JP[fv() % 4]; g->u.mutecity.x100 = &JP[fv() % 4]; g->u.mutecity.x104 = &JP[fv() % 4]; g->u.mutecity.x108 = &JP[fv() % 4]; g->u.mutecity.x10C = &JP[fv() % 4]; "
         "g->u.mutecity.xE4.x = -0.5f * (float) (fv() % 5); g->u.mutecity.xF0.x = 0.5f * (float) (fv() % 5); g->u.mutecity.xE4.y = rg() * 10.0f; g->u.mutecity.xF0.y = rg() * 10.0f; } "
         "for (i = 0; i < 4; i++) { V3[i].x = 0.5f * (float) ((int) (fv() % 9) - 4); V3[i].y = rg() * 10.0f; }",
                                "for i in 0..4usize { let g = &mut GR[i]; g.u.mutecity.xFC = &raw mut JP[(fv() % 4) as usize]; g.u.mutecity.x100 = &raw mut JP[(fv() % 4) as usize]; g.u.mutecity.x104 = &raw mut JP[(fv() % 4) as usize]; g.u.mutecity.x108 = &raw mut JP[(fv() % 4) as usize]; g.u.mutecity.x10C = &raw mut JP[(fv() % 4) as usize]; "
         "g.u.mutecity.xE4.x = -0.5f32 * (fv() % 5) as f32; g.u.mutecity.xF0.x = 0.5f32 * (fv() % 5) as f32; g.u.mutecity.xE4.y = rg() * 10.0f32; g.u.mutecity.xF0.y = rg() * 10.0f32; } "
         "for i in 0..4usize { V3[i].x = 0.5f32 * ((fv() % 9) as i32 - 4) as f32; V3[i].y = rg() * 10.0f32; }"),
        "grMuteCity_801EFC6C": ("NG = 0;", "NG = false;"),
        "grMuteCity_801EFD0C": ("NG = 1;", "NG = true;"),
        "grMuteCity_801F0120": (CMD_C, CMD_R),
        "grMuteCity_801F04B8": (MC_C + chr(10) + "for (i = 0; i < 4; i++) GR[i].u.mutecity.xC8 = &GOX;", MC_R + chr(10) + "for i in 0..4usize { GR[i].u.mutecity.xC8 = &raw mut GOX; }"),
        "grMuteCity_801F1A34": ("GR[2].u.mutecity.xC4 = (s16) (fv() % 10);", "GR[2].u.mutecity.xC4 = (fv() % 10) as i16;"),
        "fn_801F2B58": (COLL_C, COLL_R),
    },
    cases={"grMuteCity_801F2C10": 1500, "grMuteCity_801F04B8": 3000, "grMuteCity_801F0120": 1500, "grMuteCity_801F1A34": 1500, "grMuteCity_801F1328": 1000,
           "grMuteCity_801F106C": 1500, "grMuteCity_801EFDF8": 800, "grMuteCity_801F0D20": 800},
)
