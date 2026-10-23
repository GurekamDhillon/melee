"""Oracle spec of tu::ground (see gen_drv.py for the keys).

World: the usual GO/GR/JP/CD/V3 plus (world_ground_c.py / world_ground_r.py, the same world in C and Rust)
  stage_info with every pointer field linked into the pools, GP / SP[4] (GroundParam + stage params), the
  stage archive ARCH -> STD -> X8[8] (joints HJ, fog FD, lights LLP / LL / LD, shadow entries GSE, joint pairs
  ENT / PRS, GJA / SPA), the stage rows (every grXX_StageData global + Ground_MexStageDatas filled with tracing
  callbacks, stage_datas[] restored from a snapshot), NG / NGR / JN (what GObj_Create / HSD_MemAlloc /
  HSD_JObjLoadJoint hand out), CJ / CLN (the map collision tables), MCD / MCP / ARENA (collision data builds),
  BUF (Ground_804D6950) ...
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from world_ground_c import DECL_C, ROWNAMES  # noqa: E402
from world_ground_r import DECL_R  # noqa: E402

CHR = chr(10)

# `extern`s of the data the generated header defines (it is included after the declarations)
EXTERNS_C = CHR.join(f"extern StageData {n};" for n in ROWNAMES) + CHR + (
    "extern HSD_GObj** HSD_GObjPLinkHead; extern s8 HSD_GObj_LightKind; extern Article** it_804D6D38; extern int mpColl_804D64AC;" + CHR)
DECL_C = EXTERNS_C + DECL_C
DECL_R = DECL_R + r"""
fn rhn(p: *const u8, n: usize) -> u32 {
    let mut h = 0x811C_9DC5u32;
    for i in 0..n { h = (h ^ unsafe { *p.add(i) } as u32).wrapping_mul(0x0100_0193); }
    h
}
"""

PN0 = ("stage_info.param = ((fv() % 4) == 0) ? 0 : &GP;",
       "m::stage_info.param = if fv() % 4 == 0 { null_mut() } else { &raw mut GP };")
AN = ("ANULL = ((fv() % 4) == 0) ? 1 + (int) (fv() & 1) : 0;", "ANULL = if fv() % 4 == 0 { 1 + (fv() & 1) as i32 } else { 0 };")
GCNT = ("if ((fv() % 6) == 0) GP.stage_param_count = (int) (fv() % 3) - 1;", "if fv() % 6 == 0 { GP.stage_param_count = (fv() % 3) as i32 - 1; }")
GFAILP = ("GFAIL = ((fv() % 8) == 0) ? 1 + (int) (fv() % 3) : 0;", "GFAIL = if fv() % 8 == 0 { 1 + (fv() % 3) as i32 } else { 0 };")
JLNP = ("JLN = 1;", "JLN = 1;")
STDN = ("STDNULL = (fv() % 4) == 0; ARCH.unk4 = STDNULL ? 0 : &STD;",
        "STDNULL = (fv() % 4 == 0) as i32; ARCH.unk4 = if STDNULL != 0 { null_mut() } else { &raw mut STD };")
WPI = ("WP = (int) (fv() & 1); WI = (int) (fv() & 1);", "WP = (fv() & 1) as i32; WI = (fv() & 1) as i32;")
AM1 = ("AMODE = 1; MI = 0;", "AMODE = 1; MI = 0;")
SETPAIR0 = ("setpair(0);", "setpair(0);")
SETPAIR1 = ("setpair(1);", "setpair(1);")
LIGHTS = ("lights_pre();", "lights_pre();")


def both(*pairs):
    return (CHR.join(p[0] for p in pairs), CHR.join(p[1] for p in pairs))


GOA = ("&GO[fv() & 3]", "&raw mut GO[(fv() & 3) as usize]")
JPA = ("&JP[fv() % 16]", "&raw mut JP[(fv() % 16) as usize]")
VA = ("&V3[fv() & 3]", "&raw mut V3[(fv() & 3) as usize]")
HJA = ("&HJ[fv() % 8]", "&raw mut HJ[(fv() % 8) as usize]")
MAPID = ("(int) (fv() % 8)", "(fv() % 8) as i32")
IDXA = ("pick_idx()", "pick_idx()")
STKA = ("(StKind) pick_stk()", "pick_stk() as _")
STKANY = ("(StKind) pick_stk_any()", "pick_stk_any() as _")
RGA = ("rg()", "rg()")
PAIRA = ("&PAIR", "&raw mut PAIR")
PICKGR = ("pick_gr()", "pick_gr()")
OI = lambda n: (f"&OUTI[{n}]", f"&raw mut OUTI[{n}]")
OF = lambda n: (f"&OUTF[{n}]", f"&raw mut OUTF[{n}]")
SLOTI = ("(int) (fv() % 12) - 2", "(fv() % 12) as i32 - 2")


def build_hash():
    c = ("hash_StageInfo(&stage_info); hash_GroundParam(&GP); HM(PI(Ground_804D6950)); HM((u32) Ground_804D6954); HM(RH(BUF, 64)); HM((u32) mpColl_804D64AC);\n"
         "for (i = 0; i < 160; i++) { HM(rowid(stage_datas[i])); if (ROWTAB[i]) hash_StageData(ROWTAB[i]); }\n"
         "for (i = 0; i < 4; i++) { hash_HSD_GObj(&NG[i]); hash_Ground(&NGR[i]); }\n"
         "for (i = 0; i < 8; i++) hash_HSD_JObj(&JN[i]);\n"
         "for (i = 0; i < 16; i++) hash_CollJoint(&CJ[i]);\n"
         "for (i = 0; i < 24; i++) hash_CollLine(&CLN[i]);\n"
         "for (i = 0; i < 4; i++) hash_MapCollData(&MCP[i]);\n"
         "HM(RH(ARENA, (unsigned) ARO)); hash_HSD_Fog(&FOG); hash_HSD_GObj(&FGO);\n"
         "for (i = 0; i < 6; i++) HM(LD[i].flags);\n"
         "for (i = 0; i < 3; i++) { int j; for (j = 0; j < 8; j++) HM(PI((const void*) LLP[i][j])); }\n"
         "for (i = 0; i < 8; i++) { HM((u32) OUTI[i]); HM(PI((const void*) ISA.x0_itemStateDesc[i].xC_script)); }\n"
         "for (i = 0; i < 4; i++) HM(B(OUTF[i]));\n"
         "for (i = 0; i < 40; i++) HM((u32) I35[i]);\n"
         "HM(OUTU); HM(RH(GXC, sizeof GXC));\n"
         "for (i = 0; i < 4; i++) { HM(PI(LNP[i].n)); HM(PI(LNP[i].a)); HM(FI(LNP[i].f)); }\n"
         "HM((u32) JI); HM((u32) GI); HM((u32) NGRI); HM((u32) LNI);")
    r = ("g::hash_StageInfo(&m::stage_info); g::hash_GroundParam(&GP); hm(pi(m::Ground_804D6950)); hm(m::Ground_804D6954 as u32); hm(rh(&BUF)); hm(m::ext::mpColl_804D64AC as u32);\n"
         "for i in 0..160usize { hm(rowid(m::stage_datas[i])); if !ROWTAB[i].is_null() { g::hash_StageData(&*ROWTAB[i]); } }\n"
         "for i in 0..4usize { g::hash_HSD_GObj(&NG[i]); g::hash_Ground(&NGR[i]); }\n"
         "for i in 0..8usize { g::hash_HSD_JObj(&JN[i]); }\n"
         "for i in 0..16usize { g::hash_CollJoint(&CJ[i]); }\n"
         "for i in 0..24usize { g::hash_CollLine(&CLN[i]); }\n"
         "for i in 0..4usize { g::hash_MapCollData(&MCP[i]); }\n"
         "hm(rhn((&raw const ARENA) as *const u8, ARO as usize)); g::hash_HSD_Fog(&FOG); g::hash_HSD_GObj(&FGO);\n"
         "for i in 0..6usize { hm(LD[i].flags as u32); }\n"
         "for i in 0..3usize { for j in 0..8usize { hm(pi(LLP[i][j])); } }\n"
         "for i in 0..8usize { hm(OUTI[i] as u32); hm(pi(ISA.x0_itemStateDesc[i].xC_script)); }\n"
         "for i in 0..4usize { hm(OUTF[i].to_bits()); }\n"
         "for i in 0..40usize { hm(I35[i] as u32); }\n"
         "hm(OUTU); hm(rh(&GXC));\n"
         "for i in 0..4usize { hm(pi(LNP[i].n)); hm(pi(LNP[i].a)); hm(fi(if LNP[i].f.is_null() { None } else { Some(core::mem::transmute::<*mut c_void, unsafe extern \"C\" fn(i32)>(LNP[i].f)) })); }\n"
         "hm(JI as u32); hm(GI as u32); hm(NGRI as u32); hm(LNI as u32);")
    return c, r


HASH_C, HASH_R = build_hash()

REGS = [("stage_info", "m::stage_info"), ("SP", "SP"), ("GP", "GP"), ("ARCH", "ARCH"), ("STD", "STD"), ("X8", "X8"), ("HJ", "HJ"),
        ("CAMD", "CAMD"), ("GJA", "GJA"), ("GJB", "GJB"), ("SPA", "SPA"), ("ENT", "ENT"), ("PRS", "PRS"), ("SPL", "SPL"), ("SPLP", "SPLP"),
        ("LOE", "LOE"), ("GSE", "GSE"), ("LD", "LD"), ("LAN", "LAN"), ("ANL", "ANL"), ("LL", "LL"), ("LLP", "LLP"), ("LO", "LO"), ("WO", "WO"),
        ("AO", "AO"), ("FOG", "FOG"), ("FD", "FD"), ("FGO", "FGO"), ("NG", "NG"), ("NGR", "NGR"), ("JN", "JN"), ("COB", "COB"), ("CJ", "CJ"),
        ("CLN", "CLN"), ("MCD", "MCD"), ("MCP", "MCP"), ("MV", "MV"), ("MLN", "MLN"), ("MJN", "MJN"), ("ARENA", "ARENA"), ("BUF", "BUF"),
        ("PL", "PL"), ("PAIR", "PAIR"), ("GID", "GID"), ("ITD", "ITD"), ("AYA", "AYA"), ("ART", "ART"), ("ISA", "ISA"), ("TB", "TB"),
        ("DUM", "DUM"), ("DUMD", "DUMD"), ("ARCHB", "ARCHB"), ("LNP", "LNP"), ("SCB", "SCB"), ("STRB", "STRB"), ("YAK", "YAK"), ("OUTI", "OUTI"),
        ("OUTF", "OUTF"), ("I35", "I35"), ("GXC", "GXC"), ("OUTU", "OUTU"), ("CKV", "CKV")]
PREG_C = " ".join(f"preg(&{c}, sizeof({c}));" for c, _ in REGS)
PREG_R = " ".join(f"preg(&raw const {r}, core::mem::size_of_val(&{r}));" for _, r in REGS)

# the callbacks the world stores: registered after the TU's functions (same order in both)
FREG_C = " ".join(f"freg((const void*) {n});" for n in ("sd_oi", "sd_od", "sd_ol", "sd_os", "sd_c4", "sd_tl", "sd_cs", "pr90", "pr94", "lnfn", "cbi", "ev1", "ev2", "ev3"))
FREG_R = " ".join(f"freg({n} as *const () as usize);" for n in ("sd_oi", "sd_od", "sd_ol", "sd_os", "sd_c4", "sd_tl", "sd_cs", "pr90", "pr94", "lnfn", "cbi", "ev1", "ev2", "ev3"))


def eff(c, r):
    return (c, r)


NULLV = "core::ptr::null_mut()"

SPEC = dict(
    src="src/melee/gr/ground.c",
    extra_main=["src/melee/gr/ground_stage_slots.inc", "src/melee/gr/ground_stage_slot_lighting.inc",
                "src/melee/gr/ground_stage_slot_retail.inc"],
    # the one-shot flag of Ground_MexInitStages is a function static: lift it to file scope so each case can reset it
    patch=[("void Ground_MexInitStages(void)\n{\n    static bool done;\n    int k, n;\n\n    if (done) {\n        return;\n    }\n    done = true;",
            "bool Ground_MexInitStages_done;\nvoid Ground_MexInitStages(void)\n{\n    int k, n;\n\n    if (Ground_MexInitStages_done) {\n        return;\n    }\n    Ground_MexInitStages_done = true;")],
    types=["StageInfo", "StageCallbacks", "StageData", "GroundParam", "CollJoint", "CollLine", "MapCollData", "HSD_Fog"],
    null_ret_ok=["HSD_GObj_SetupProc", "lbVector_Add", "lbVector_CrossprodNormalized", "lbVector_EulerAnglesFromONB"],
    skip_stubs=["__frsqrte", "sinf", "cosf", "memset"],  # gekko_fp.c / the CRT are linked into the oracle
    custom_stubs={
        "HSD_ForeachAnim": dict(
            c="void HSD_ForeachAnim(void* a0, HSD_Type a1, HSD_TypeMask a2, void* a3, AObj_Arg_Type a4, ...) { __builtin_va_list ap; "
              "__builtin_va_start(ap, a4); T(@ID@); T(PI(a0)); T((u32) a1); T(a2); T(PI(a3)); T((u32) a4); "
              "if (a4 == 1) { T64(__builtin_va_arg(ap, double)); } else { T(__builtin_va_arg(ap, unsigned)); } "
              "__builtin_va_end(ap); }",
            ext="fn HSD_ForeachAnim(a0: *mut c_void, a1: i32, a2: u32, a3: *mut c_void, a4: i32, a5: f64, a6: u32);",
            hook="|a0, a1, a2, a3, a4, a5, a6| { t(@ID@); t(pi(a0)); t(a1 as u32); t(a2); t(pi(a3)); t(a4 as u32); if a4 == 1 { t64(a5); } else { t(a6); } }"),
    },
    # ScriptGame_RefsGroundBorn(map_id, (int) gobj): the pointer truncated to 32 bits is named by the region it points into
    trace={"ScriptGame_RefsGroundBorn": {1: ("PI32(a1)", "pi32(a1 as u32)")}},
    ret={
        "GObj_Create": ("(GFAIL == 1) ? 0 : &NG[GI++ & 3]", "if super::GFAIL == 1 { core::ptr::null_mut() } else { let k = super::GI; super::GI += 1; &raw mut super::NG[(k & 3) as usize] }"),
        "HSD_JObjLoadJoint": ("(JLN && (fv() & 7) == 0) ? 0 : &JN[JI++ & 7]", "if super::JLN != 0 && fv() & 7 == 0 { core::ptr::null_mut() } else { let k = super::JI; super::JI += 1; &raw mut super::JN[(k & 7) as usize] }"),
        "HSD_JObjAlloc": ("((fv() & 7) == 0 || GFAIL == 3) ? 0 : &JN[JI++ & 7]",
                          "if fv() & 7 == 0 || super::GFAIL == 3 { core::ptr::null_mut() } else { let k = super::JI; super::JI += 1; &raw mut super::JN[(k & 7) as usize] }"),
        "HSD_MemAlloc": ("alloc_(a0)", "super::alloc_(a0)"),
        "HSD_FogLoadDesc": ("&FOG", "&raw mut super::FOG"),
        "HSD_CObjGetCurrent": ("&COB", "&raw mut super::COB"),
        "HSD_LObjGetPositionWObj": ("(WP ? &WO[0] : 0)", "if super::WP != 0 { &raw mut super::WO[0] } else { core::ptr::null_mut() }"),
        "HSD_LObjGetInterestWObj": ("(WI ? &WO[1] : 0)", "if super::WI != 0 { &raw mut super::WO[1] } else { core::ptr::null_mut() }"),
        "lb_80011AC4": ("&LO[0]", "&raw mut super::LO[0]"),
        "lb_80013B14": ("&COB", "&raw mut super::COB"),
        "Player_GetEntity": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "Player_GetEntityAtIndex": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "grDatFiles_801C6330": ("(ANULL ? 0 : &ARCH)", "if super::ANULL != 0 { core::ptr::null_mut() } else { &raw mut super::ARCH }"),
        "grDatFiles_GetArchive": ("(ANULL == 1 ? 0 : &ARCH)", "if super::ANULL == 1 { core::ptr::null_mut() } else { &raw mut super::ARCH }"),
        "mpGetGroundCollJoint": ("CJ", "(&raw mut super::CJ) as *mut CollJoint"),
        "mpGetGroundCollLine": ("CLN", "(&raw mut super::CLN) as *mut CollLine"),
        "Mex_GrFile": ("STRB[fv() & 3]", "(&raw const super::STRB[(fv() & 3) as usize]) as *const i8"),
        "Mex_GrRowCallbacks": ("((fv() % 4) == 0) ? 0 : (void*) SCB[fv() % 3]",
                               "if fv() % 4 == 0 { core::ptr::null_mut() } else { (&raw mut super::SCB[(fv() % 3) as usize]) as *mut c_void }"),
        "Mex_GrCallbacks": ("((fv() % 4) == 0) ? 0 : (void*) SCB[fv() % 3]",
                            "if fv() % 4 == 0 { core::ptr::null_mut() } else { (&raw mut super::SCB[(fv() % 3) as usize]) as *mut c_void }"),
        "Mex_GrOnTouchLine": ("((fv() & 1) == 0) ? 0 : (void*) DUMD", "if fv() & 1 == 0 { core::ptr::null_mut() } else { (&raw mut super::DUMD) as *mut c_void }"),
        "gm_801B6320": ("&CKV", "(&raw mut super::CKV) as *mut c_void"),
        "it_8026BE84": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "it_8027B5B0": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "it_802F2094": ("((fv() & 1) == 0) ? 0 : &GO[fv() & 3]", "if fv() & 1 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "ScriptGame_StagePrepare": ("a0", "a0"),
        "ScriptGame_StageSlotGround": ("((fv() & 7) == 0) ? &GO[fv() & 3] : 0", "if fv() & 7 == 0 { &raw mut super::GO[(fv() & 3) as usize] } else { core::ptr::null_mut() }"),
        "TTMod_ForCharacter": ("(int) (fv() % 5) - 1", "(fv() % 5) as i32 - 1"),
        "TTMod_PlatformCount": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
        "TTMod_TargetCount": ("(int) (fv() % 4)", "(fv() % 4) as i32"),
        "Mex_GrInternalCount": ("60 + (int) (fv() % 110)", "60 + (fv() % 110) as i32"),
        "Mex_GrBgmCount": ("1 + (a0 & 3) % 3", "1 + (a0 & 3) % 3"),
        "Mex_GrBgmId": ("(a1 == 1 && a0 % 3 == 0) ? 0 : 1 + (a0 * 7 + a1 * 5) % 40", "if a1 == 1 && a0 % 3 == 0 { 0 } else { 1 + (a0.wrapping_mul(7).wrapping_add(a1.wrapping_mul(5))) % 40 }"),
        "Mex_GrBgmChance": ("(a0 * 3 + a1 * 11) % 50", "(a0.wrapping_mul(3).wrapping_add(a1.wrapping_mul(11))) % 50"),
    },
    effects={
        "GObj_InitUserData": ("if (a0) { a0->user_data_kind = a1; a0->user_data = a3; }",
                              "if !a0.is_null() { (*a0).user_data_kind = a1; (*a0).user_data = a3; }"),
        "HSD_GObjObject_80390A70": ("if (a0) { a0->obj_kind = a1; a0->hsd_obj = a2; }",
                                    "if !a0.is_null() { (*a0).obj_kind = a1; (*a0).hsd_obj = a2; }"),
        "HSD_CObjGetEyeVector": ("WV(a1);", "super::wv(a1);"),
        "HSD_CObjGetEyePosition": ("WV(a1);", "super::wv(a1);"),
        "HSD_LObjGetInterest": ("WV(a1);", "super::wv(a1);"),
        "HSD_LObjGetPosition": ("WV(a1);", "super::wv(a1);"),
        "HSD_WObjGetPosition": ("WV(a1);", "super::wv(a1);"),
        "HSD_LObjSetInterest": ("TV(a1);", "super::tv(a1);"),
        "HSD_LObjSetPosition": ("TV(a1);", "super::tv(a1);"),
        "HSD_WObjSetPosition": ("TV(a1);", "super::tv(a1);"),
        "Player_LoadPlayerCoords": ("WV(a1);", "super::wv(a1);"),
        "ftLib_80086644": ("WV(a1);", "super::wv(a1);"),
        "Stage_UnkSetVec3TCam_Offset": ("WV(a0);", "super::wv(a0);"),
        "lb_8000B1CC": ("WV(a2);", "super::wv(a2);"),
        "splGetSplinePoint": ("WV(a0);", "super::wv(a0);"),
        "lbShadow_8000E9F0": ("WV(a0);", "super::wv(a0);"),
        "lbVector_Normalize": ("TV(a0); WV(a0);", "super::tv(a0); super::wv(a0);"),
        "lbVector_CrossprodNormalized": ("TV(a0); TV(a1); WV(a2);", "super::tv(a0); super::tv(a1); super::wv(a2);"),
        "lbVector_Add": ("TV(a0); TV(a1); WV(a0);", "super::tv(a0); super::tv(a1); super::wv(a0);"),
        "lbVector_EulerAnglesFromONB": (
            "TV(a1); TV(a2); TV(a3); if (a0) { a0->x = ((fv() % 4) == 0) ? 40000.0f : rg(); a0->y = ((fv() % 4) == 0) ? -35000.0f : rg(); "
            "a0->z = ((fv() % 4) == 0) ? 31000.0f : rg(); }",
            "super::tv(a1); super::tv(a2); super::tv(a3); if !a0.is_null() { (*a0).x = if fv() % 4 == 0 { 40000.0 } else { rg() }; "
            "(*a0).y = if fv() % 4 == 0 { -35000.0 } else { rg() }; (*a0).z = if fv() % 4 == 0 { 31000.0 } else { rg() }; }"),
        "PSVECAdd": ("TV(a0); TV(a1); if (a2) { a2->x = a0->x + a1->x; a2->y = a0->y + a1->y; a2->z = a0->z + a1->z; }",
                     "super::tv(a0); super::tv(a1); if !a2.is_null() { (*a2).x = (*a0).x + (*a1).x; (*a2).y = (*a0).y + (*a1).y; (*a2).z = (*a0).z + (*a1).z; }"),
        "PSMTXIdentity": ("if (a0) { int r_, c_; for (r_ = 0; r_ < 3; r_++) for (c_ = 0; c_ < 4; c_++) a0[r_][c_] = (r_ == c_) ? 1.0f : 0.0f; }",
                          "if !a0.is_null() { let mm = a0 as *mut [f32; 4]; for r in 0..3usize { for c in 0..4usize { (*mm.add(r))[c] = if r == c { 1.0 } else { 0.0 }; } } }"),
        "TTMod_Platform": ("if (a2) *a2 = rg(); if (a3) *a3 = rg(); if (a4) *a4 = rg(); if (a5) *a5 = rg(); if (a6) *a6 = rg();",
                           "if !a2.is_null() { *a2 = rg(); } if !a3.is_null() { *a3 = rg(); } if !a4.is_null() { *a4 = rg(); } if !a5.is_null() { *a5 = rg(); } if !a6.is_null() { *a6 = rg(); }"),
        "TTMod_Target": ("if (a2) *a2 = rg(); if (a3) *a3 = rg(); if (a4) *a4 = rg();",
                         "if !a2.is_null() { *a2 = rg(); } if !a3.is_null() { *a3 = rg(); } if !a4.is_null() { *a4 = rg(); }"),
        "it_8027B5B0": ("TV(a1);", "super::tv(a1);"),
        "it_802F2094": ("TV(a1);", "super::tv(a1);"),
        "it_8026BE84": ("T(PI((const void*) a0->x0)); T(PI((const void*) a0->x4)); TV(&a0->x8_vec); T((u32) a0->x14); T((u32) a0->x18); T((u32) a0->x1C.b0);",
                        "t(pi((*a0).x0)); t(pi((*a0).x4)); super::tv(&(*a0).x8_vec); t((*a0).x14 as u32); t((*a0).x18 as u32); t((*a0).x1C.anon_0.b0() as u32);"),
        "HSD_JObjLoadJoint": ("if (a0) { T(a0->flags); T(B(a0->scale.x)); T(B(a0->scale.y)); T(B(a0->scale.z)); T(B(a0->rotation.x)); T(B(a0->position.x)); T(PI((const void*) a0->child)); }",
                              "if !a0.is_null() { t((*a0).flags); t((*a0).scale.x.to_bits()); t((*a0).scale.y.to_bits()); t((*a0).scale.z.to_bits()); t((*a0).rotation.x.to_bits()); t((*a0).position.x.to_bits()); t(pi((*a0).child)); }"),
    },
    ops=[("Ground_MexInitStages_twice", "Ground_MexInitStages(); Ground_MexInitStages(); ret = 0;", "m::Ground_MexInitStages(); m::Ground_MexInitStages(); ret = 0;")],
    decl_c=DECL_C,
    decl_r=DECL_R,
    setup_c="setup_world();",
    setup_r="setup_world();",
    hash_c=HASH_C,
    hash_r=HASH_R,
    preg_c=PREG_C + " " + FREG_C,
    preg_r=PREG_R + " " + FREG_R,
    skip={"panicMissingStageParam": "while (true) {} after reporting: never returns (the PC build takes the zeros fallback instead)"},
    data=[("Ground_StageData", "StageData", None), ("Ground_TTMod_BoxVerts", "raw", None), ("Ground_TTMod_BoxDisp", "raw", None),
          ("Ground_TTMod_BoxMat", "raw", None), ("Ground_804D4508", "raw", None)],
    pools={
        "HSD_Joint": HJA, "HSD_LObj": ("&LO[fv() & 3]", "&raw mut LO[(fv() & 3) as usize]"),
        "HSD_CObj": ("&COB", "&raw mut COB"), "HSD_Spline": ("&SPL[fv() & 3]", "&raw mut SPL[(fv() & 3) as usize]"),
        "StageInfo": ("&stage_info", "&raw mut m::stage_info"), "MapCollData": ("&MCD", "&raw mut MCD"),
        "StageIdPair": PAIRA, "UnkArchiveStruct": ("&ARCH", "&raw mut ARCH"), "UnkStageDat": ("&STD", "&raw mut STD"),
        "HSD_LightDesc": ("&LD[fv() % 6]", "&raw mut LD[(fv() % 6) as usize]"), "_GXColor": ("&GXC[fv() & 3]", "&raw mut GXC[(fv() & 3) as usize]"),
    },
    args={
        "Ground_StageSlotDataFile": [("(int) (fv() % 170) - 5", "(fv() % 170) as i32 - 5")],
        "Ground_StageSlotRetailInstall": [PAIRA, ("(HSD_Archive*) ARCHB", "(&raw mut ARCHB) as *mut HSD_Archive")],
        "Ground_StageSlotJoints": [("(int) (fv() % 170) - 5", "(fv() % 170) as i32 - 5"), ("&OUTU", "&raw mut OUTU")],
        "Ground_MexStageCallbacks": [("((fv() % 10) == 0) ? (((fv() & 1) != 0) ? 23 : 26) : (int) (fv() % 160)", "if fv() % 10 == 0 { if fv() & 1 != 0 { 23 } else { 26 } } else { (fv() % 160) as i32 }")],
        "mem_free": [("(void*) &GO[fv() & 3]", "(&raw mut GO[(fv() & 3) as usize]) as *mut c_void")],
        "Ground_801C06B8": [("(GrKind) (((fv() % 6) == 0) ? (((fv() & 1) != 0) ? Gr_Kind_Izumi : Gr_Kind_PStadium) : (((fv() % 8) == 0) ? 26 : (int) (fv() % 160)))",
                            "(if fv() % 6 == 0 { if fv() & 1 != 0 { Gr_Kind_Izumi as i32 } else { Gr_Kind_PStadium as i32 } } else if fv() % 8 == 0 { 26 } else { (fv() % 160) as i32 }) as _")],
        "Ground_801C0754": [PAIRA],
        "Ground_TTMod_BuildCollData": [("&MCD", "&raw mut MCD"), ("(int) (fv() % 4)", "(fv() % 4) as i32"), ("(int) (fv() % 4)", "(fv() % 4) as i32")],
        "Ground_TTMod_InitLines": [("&MCD", "&raw mut MCD"), ("(int) (fv() % 4)", "(fv() % 4) as i32")],
        "Ground_801C0800": [PAIRA],
        "Ground_OnLoad": [PAIRA], "Ground_801C0FB8": [PAIRA], "Ground_DemoInit": [PAIRA, ("(int) (fv() % 8)", "(fv() % 8) as i32")],
        "Ground_801C10B8": [GOA, ("ev1", "Some(ev1)")],
        "Ground_801C126C": [HJA, OI(0)],
        "Ground_801C13D0": [MAPID, ("(int) (fv() % 12)", "(fv() % 12) as i32")],
        "Ground_GetStageGObj": [("(int) (fv() % 10)", "(fv() % 10) as i32")],
        "Ground_801C1A20": [HJA, MAPID],
        "Ground_StageSlotPrepare": [GOA, MAPID],
        "Ground_801C1D6C": [("fv()", "fv()")],
        "Ground_801C1DE4": [OI(0), OI(1)],
        "find_light_override": [("&ARCH", "&raw mut ARCH"), ("&LD[fv() % 6]", "&raw mut LD[(fv() % 6) as usize]"), OI(0), OI(1), OI(2)],
        "find_light_override_in_dat": [("&STD", "&raw mut STD"), ("&STD", "&raw mut STD"), ("&LD[fv() % 6]", "&raw mut LD[(fv() % 6) as usize]"),
                                        OI(0), OI(1), OI(2)],
        "Ground_801C20E0": [("&ARCH", "&raw mut ARCH"), ("LLP[fv() % 3]", "(&raw mut LLP[(fv() % 3) as usize]) as *mut *mut LightList")],
        "Ground_801C2374": [("&LO[fv() & 3]", "&raw mut LO[(fv() & 3) as usize]")],
        "Ground_801C247C": [MAPID, ("(int) (fv() % 6)", "(fv() % 6) as i32")],
        "Ground_MexParamStkind": [STKANY],
        "Ground_801C24F8": [STKANY, ("pick_bgmflags()", "pick_bgmflags()"), OI(0)],
        "Ground_801C28AC": [STKANY, ("pick_bgmflags()", "pick_bgmflags()"), OI(0)],
        "Ground_801C28CC": [("&I35[0]", "&raw mut I35 as *mut i32"), STKANY],
        "Ground_801C2AE8": [STKA],
        "Ground_GetMapGObj": [("(int) (fv() % 64)", "(fv() % 64) as i32")],
        "Ground_801C2BBC": [GOA, ("(int) (fv() % 64)", "(fv() % 64) as i32")],
        "Ground_801C2BD4": [("((fv() & 1) != 0) ? stage_info.x694[fv() & 3] : (void*) &JP[fv() % 16]",
                             "if fv() & 1 != 0 { m::stage_info.x694[(fv() & 3) as usize] } else { (&raw mut JP[(fv() % 16) as usize]) as *mut c_void }")],
        "Ground_801C2C8C": [("((fv() & 1) != 0) ? stage_info.x694[fv() & 3] : (void*) &JP[fv() % 16]",
                             "if fv() & 1 != 0 { m::stage_info.x694[(fv() & 3) as usize] } else { (&raw mut JP[(fv() % 16) as usize]) as *mut c_void }")],
        "Ground_801C2CF4": [IDXA], "Ground_801C2D0C": [IDXA, JPA], "Ground_801C2D24": [IDXA, VA],
        "Ground_InitMapColl": [JPA, MAPID],
        "Ground_UpdateMapColl": [GOA],
        "Ground_801C3128": [MAPID, ("cbi", "Some(cbi)")],
        "Ground_801C3214": [MAPID], "Ground_801C3260": [MAPID], "Ground_801C32AC": [MAPID],
        "Ground_801C32D4": [MAPID, ("(int) (fv() % 16)", "(fv() % 16) as i32")],
        "Ground_801C33C0": [MAPID, ("(int) (fv() % 16)", "(fv() % 16) as i32")],
        "Ground_801C34AC": [MAPID, ("((fv() % 8) == 0) ? 0 : &JP[fv() % 16]", "if fv() % 8 == 0 { null_mut() } else { &raw mut JP[(fv() % 16) as usize] }"),
                            ("((fv() % 8) == 0) ? 0 : &HJ[fv() % 8]", "if fv() % 8 == 0 { null_mut() } else { &raw mut HJ[(fv() % 8) as usize] }")],
        "Ground_801C36F4": [MAPID, ("((fv() % 3) == 0) ? &JP[fv() % 16] : &JP[0]", "if fv() % 3 == 0 { &raw mut JP[(fv() % 16) as usize] } else { &raw mut JP[0] }"),
                            ("((fv() % 8) == 0) ? 0 : (void*) &HJ[fv() % 8]", "if fv() % 8 == 0 { null_mut() } else { (&raw mut HJ[(fv() % 8) as usize]) as *mut c_void }")],
        "Ground_801C3D44": [("((fv() & 1) != 0) ? (void*) pr90 : (void*) 0", "if fv() & 1 != 0 { pr90 as usize as *mut c_void } else { null_mut() }"), RGA, RGA],
        "Ground_801C3DB4": [("((fv() & 1) != 0) ? (void*) pr94 : (void*) 0", "if fv() & 1 != 0 { pr94 as usize as *mut c_void } else { null_mut() }"), RGA, RGA],
        "Ground_801C3E18": [JPA], "Ground_801C3F20": [JPA],
        "Ground_801C3FA4": [("((fv() % 6) == 0) ? &NG[0] : &GO[fv() & 3]", "if fv() % 6 == 0 { &raw mut NG[0] } else { &raw mut GO[(fv() & 3) as usize] }"), ("(int) (fv() % 20) - 2", "(fv() % 20) as i32 - 2")],
        "Ground_TTMod_AddPlatformBoxes": [("(int) (fv() % 4)", "(fv() % 4) as i32")],
        "Ground_801C4368": [OF(0), OF(1)],
        "Ground_801C43A4": [("(void*) &V3[fv() & 3]", "(&raw mut V3[(fv() & 3) as usize]) as *mut c_void")],
        "Ground_801C43C4": [("((fv() % 5) == 0) ? (void*) 0 : (void*) &LAN[fv() % 5]",
                             "if fv() % 5 == 0 { null_mut() } else { (&raw mut LAN[(fv() % 5) as usize]) as *mut c_void }")],
        "Ground_801C445C": [("((fv() % 8) == 0) ? 0 : &LO[fv() & 3]", "if fv() % 8 == 0 { null_mut() } else { &raw mut LO[(fv() & 3) as usize] }")],
        "Ground_801C461C": [GOA],
        "removeStageGObj": [GOA],
        "Ground_801C4B50": [("&SPL[fv() & 3]", "&raw mut SPL[(fv() & 3) as usize]"), VA, VA, RGA],
        "Ground_801C4DA0": [VA, OF(0)],
        "Ground_801C4FAC": [("&COB", "&raw mut COB")],
        "Ground_801C4A08": [("((fv() % 8) == 0) ? 0 : &GO[fv() & 3]", "if fv() % 8 == 0 { null_mut() } else { &raw mut GO[(fv() & 3) as usize] }")],
        "Ground_801C53EC": [("fv()", "fv()")],
        "Ground_801C5440": [PICKGR, SLOTI, ("pick_snd()", "pick_snd()")],
        "Ground_801C54DC": [PICKGR, SLOTI], "Ground_801C5544": [PICKGR, SLOTI], "Ground_801C55AC": [PICKGR],
        "Ground_801C5630": [PICKGR, SLOTI, RGA], "Ground_801C5694": [PICKGR, SLOTI, RGA],
        "initSinglePlayerDisplay": [("&stage_info", "&raw mut m::stage_info")],
        "Ground_801C58E0": [("(int) (fv() % 8)", "(fv() % 8) as i32"), IDXA],
        "Ground_801C5AD0": [("pick_kind()", "pick_kind()")],
    },
    pre={
        "Ground_StageSlotRetailInstall": both(SETPAIR1, ("stage_info.grkind = PAIR.grkind;", "m::stage_info.grkind = PAIR.grkind;"), LIGHTS, AM1),
        "Ground_801BFFB0": ("Ground_804D6950 = ((fv() % 4) == 0) ? 0 : BUF;",
                            "m::Ground_804D6950 = if fv() % 4 == 0 { null_mut() } else { (&raw mut BUF) as *mut u8 };"),
        "Ground_801C0498": PN0, "Ground_801C0508": PN0,
        "Ground_801C0754": SETPAIR0,
        "Ground_TTMod_BuildCollData": both(("AMODE = ((fv() % 8) == 0) ? 2 : 1; MI = 0;", "AMODE = if fv() % 8 == 0 { 2 } else { 1 }; MI = 0;"), PN0),
        "Ground_TTMod_InitLines": ("MCD.line_count = 3 + (int) (fv() % 4);", "MCD.line_count = 3 + (fv() % 4) as i32;"),
        "Ground_801C0800": both(("setpair((fv() % 8) != 0);", "setpair((fv() % 8 != 0) as i32);"),
                                ("if ((fv() % 2) != 0) stage_info.grkind = (GrKind) (Gr_Kind_TMario + (int) (fv() % 26));",
                                 "if fv() % 2 != 0 { m::stage_info.grkind = (Gr_Kind_TMario as i32 + (fv() % 26) as i32) as _; }"), LIGHTS, ("AMODE = ((fv() % 8) == 0) ? 2 : 1; MI = 0;", "AMODE = if fv() % 8 == 0 { 2 } else { 1 }; MI = 0;")),
        "Ground_801C0C2C": ("if ((fv() % 2) == 0) { stage_info.unk8C.b7 = 1; stage_info.x718 = 50.0f; stage_info.x71C = 50.0f; }",
                            "if fv() % 2 == 0 { m::stage_info.unk8C.set_b7(1); m::stage_info.x718 = 50.0; m::stage_info.x71C = 50.0; }"),
        "Ground_801C0A70": ("if ((fv() % 2) == 0) stage_info.grkind = (GrKind) (2 + fv() % 30);",
                            "if fv() % 2 == 0 { m::stage_info.grkind = (2 + fv() % 30) as _; }"),
        "Ground_OnLoad": SETPAIR0, "Ground_801C0FB8": SETPAIR0, "Ground_DemoInit": SETPAIR0,
        "Ground_801C1158": ("stage_info.grkind = ((fv() % 2) != 0) ? (GrKind) (14 + (int) (fv() % 3)) : (GrKind) pick_kind();",
                            "m::stage_info.grkind = if fv() % 2 != 0 { (14 + (fv() % 3) as i32) as _ } else { pick_kind() as _ };"),
        "Ground_801C126C": ("OUTI[0] = (int) (fv() % 12);", "OUTI[0] = (fv() % 12) as i32;"),
        "Ground_801C13D0": both(AN, JLNP),
        "Ground_MexParamStkind": both(PN0, GCNT, ("if ((fv() % 8) == 0) GP.stage_params = 0;", "if fv() % 8 == 0 { GP.stage_params = null_mut(); }")),
        "Ground_801C24F8": GCNT, "Ground_801C28AC": GCNT, "Ground_801C28CC": GCNT,
        "Ground_GetStageGObj": both(PN0, GFAILP),
        "Ground_801C1A20": both(PN0, GFAILP),
        "foo": both(AN, STDN), "Ground_801C1E94": both(AN, STDN, PN0),
        "Ground_801C2374": PN0,
        "Ground_801C247C": STDN,
        "Ground_InitMapColl": both(AN, ("if ((fv() % 8) == 0) stage_info.grkind = ((fv() & 1) != 0) ? 23 : 26;", "if fv() % 8 == 0 { m::stage_info.grkind = if fv() & 1 != 0 { 23 } else { 26 }; }")), "Ground_UpdateMapColl": AN, "Ground_801C3128": AN, "Ground_801C3214": AN, "Ground_801C3260": AN,
        "Ground_801C32AC": AN, "Ground_801C32D4": AN, "Ground_801C33C0": AN, "Ground_801C34AC": AN, "Ground_801C4A08": AN,
        "Ground_801C39C0": ("stage_info.grkind = (GrKind) pick_gk();", "m::stage_info.grkind = pick_gk() as _;"),
        "Ground_801C3BB4": ("stage_info.grkind = (GrKind) pick_gk();", "m::stage_info.grkind = pick_gk() as _;"),
        "Ground_TTMod_AddPlatformBoxes": both(PN0, JLNP),
        "Ground_801C445C": both(WPI, PN0),
        "Ground_801C461C": both(("for (i = 0; i < 4; i++) GO[i].hsd_obj = &LO[i];", "for i in 0..4usize { GO[i].hsd_obj = (&raw mut LO[i]) as *mut c_void; }"), WPI, PN0),
        "Ground_801C466C": both(LIGHTS, ("ANULL = ((fv() % 4) == 0) ? 2 : 0; if (ANULL) set_gsn(0);", "ANULL = if fv() % 4 == 0 { 2 } else { 0 }; if ANULL != 0 { set_gsn(0); }")),
        "Ground_801C498C": ("HSD_GObjPLinkHead = PL;", "m::ext::HSD_GObjPLinkHead = (&raw mut PL) as *mut *mut HSD_GObj;"),
        "Ground_801C4DD0": ("stage_info.grkind = ((fv() % 3) != 0) ? (((fv() & 1) != 0) ? Gr_Kind_Kongo : Gr_Kind_OldKongo) : pick_kind();",
                            "m::stage_info.grkind = if fv() % 3 != 0 { if fv() & 1 != 0 { Gr_Kind_Kongo as _ } else { Gr_Kind_OldKongo as _ } } else { pick_kind() as _ };"),
        "Ground_801C4E20": ("stage_info.grkind = ((fv() % 3) != 0) ? (((fv() & 1) != 0) ? Gr_Kind_Kongo : Gr_Kind_OldKongo) : pick_kind();",
                            "m::stage_info.grkind = if fv() % 3 != 0 { if fv() & 1 != 0 { Gr_Kind_Kongo as _ } else { Gr_Kind_OldKongo as _ } } else { pick_kind() as _ };"),
    },
    cases={"Ground_StageSlotRetailInstall": 1500, "Ground_801C0800": 2000, "Ground_GetStageGObj": 1500, "Ground_801C466C": 1500,
           "Ground_801C24F8": 1000, "Ground_801C28AC": 600, "Ground_801C0C2C": 1000, "Ground_801C34AC": 800, "Ground_801C4A08": 800,
           "Ground_801C4FAC": 3000, "Ground_801C445C": 600, "Ground_801C0754": 1500, "Ground_MexInitStages": 600, "foo": 600,
           "Ground_801C1E94": 600, "Ground_801C4210": 600, "Ground_801C5940": 600, "Ground_801C2D24": 800, "Ground_801C20E0": 800,
           "Ground_TTMod_BuildCollData": 600, "Ground_801C13D0": 600, "Ground_801C1A20": 600, "Ground_InitMapColl": 600,
           "Ground_UpdateMapColl": 600, "Ground_801C4B50": 800},
)
