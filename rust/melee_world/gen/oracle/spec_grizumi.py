import gro_util

CBS = ["grIzumi_801CBDD4", "grIzumi_801CBE00", "grIzumi_801CBE08", "grIzumi_801CBE0C", "grIzumi_801CBE10",
       "grIzumi_801CBE54", "grIzumi_801CBE5C", "grIzumi_801CBE60", "grIzumi_801CBE64", "grIzumi_801CC0CC",
       "grIzumi_801CC0D4", "grIzumi_801CC338", "grIzumi_801CC33C", "grIzumi_801CC350", "grIzumi_801CC358",
       "grIzumi_801CCA10", "grIzumi_801CCA14", "grIzumi_801CCA54", "grIzumi_801CCA5C", "grIzumi_801CCA60",
       "grIzumi_801CCA64", "grIzumi_801CCB08", "grIzumi_801CCB10", "grIzumi_801CCB14",
       "grIzumi_801CBB84", "grIzumi_801CBB88", "grIzumi_OnLoad", "grIzumi_OnStart", "grIzumi_801CBCE0",
       "grIzumi_801CD278", "grIzumi_801CD280"]
YP = "grIzumi_YakumonoParam"
UALL = ["izumi", "izumi2", "izumi3", "nobf"]

# stage gobj ids map to distinct gobjs (as in the game); a platform / reflection gobj aliasing another would overlay its Ground union
RET_OGO = ("((void*) &OGO[NGO - 1])", "(addr_of_mut!(OGO[NGO - 1]) as *mut _)")
RET_OJO = ("((void*) &OJO[rnd() % NJO])", "(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _)")

# every jobj / gobj pointer a function dereferences through an inline accessor must be valid
FIX_C = ("{ int k; for (k = 0; k < NGO; k++) { OGP[k].u.izumi2.xC4 = &OJO[rnd() % NJO]; OGP[k].u.izumi2.xC8 = &OJO[rnd() % NJO]; } "
         "OGP[0].u.izumi.xCC = &OGO[1 + rnd() % 4]; OGP[0].u.izumi.xD0 = rnd() % 4 ? &OJO[rnd() % NJO] : 0; "
         "OGP[0].u.izumi.xD4 = rnd() % 4 ? &OJO[rnd() % NJO] : 0; if (rnd() % 4 == 0) OGP[0].u.izumi.xCC = 0; }")
FIX_R = ("for k in 0..NGO { OGP[k].u.izumi2.xC4 = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[k].u.izumi2.xC8 = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); } "
         "OGP[0].u.izumi.xCC = addr_of_mut!(OGO[1 + (rnd() % 4) as usize]); OGP[0].u.izumi.xD0 = if rnd() % 4 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { core::ptr::null_mut() }; "
         "OGP[0].u.izumi.xD4 = if rnd() % 4 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { core::ptr::null_mut() }; if rnd() % 4 == 0 { OGP[0].u.izumi.xCC = core::ptr::null_mut(); }")
# izumi3 (a platform): state 0..4, a jobj to move
P3_C = ("OGP[0].u.izumi3.xC4 = AI[0] % 5; OGP[0].u.izumi3.xCC = &OJO[rnd() % NJO]; OGP[0].u.izumi3.xC6 = (s16) (rnd() % 8) - 4;")
P3_R = ("OGP[0].u.izumi3.xC4 = (a.i[0] % 5) as i16; OGP[0].u.izumi3.xCC = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.izumi3.xC6 = (rnd() % 8) as i16 - 4;")
# reflection record of gobj 1, hung on the gobj's user_data
REF_C = "OGO[1].user_data = OP_IzumiReflection[0]; OGP[0].u.izumi.xC8 = &OGO[1]; OGP[0].u.izumi.xC4 = rnd() % 3 ? (HSD_TObj*) OP_HSD_TObj[0] : 0;"
REF_R = ("OGO[1].user_data = addr_of_mut!(OP_IzumiReflection[0]) as *mut c_void; OGP[0].u.izumi.xC8 = addr_of_mut!(OGO[1]); "
         "OGP[0].u.izumi.xC4 = if rnd() % 3 != 0 { addr_of_mut!(OP_HSD_TObj[0]) as *mut HSD_TObj } else { core::ptr::null_mut() };")
# CD090: a dobj / mobj / tobj chain hung on the walked jobj tree
WALK_C = ("{ int k; HSD_DObj* d0 = (HSD_DObj*) OP_HSD_DObj[0]; HSD_DObj* d1 = (HSD_DObj*) OP_HSD_DObj[1]; "
          "HSD_MObj* m0 = (HSD_MObj*) OP_HSD_MObj[0]; HSD_MObj* m1 = (HSD_MObj*) OP_HSD_MObj[1]; "
          "HSD_TObj* t0 = (HSD_TObj*) OP_HSD_TObj[0]; HSD_TObj* t1 = (HSD_TObj*) OP_HSD_TObj[1]; "
          "for (k = 0; k < NJO; k++) OJO[k].u.dobj = 0; "
          "OJO[0].u.dobj = d0; d0->next = d1; d1->next = 0; d0->mobj = rnd() % 3 ? m0 : 0; d1->mobj = m1; "
          "m0->tobj = t0; m1->tobj = rnd() % 3 ? t1 : 0; t0->next = rnd() % 2 ? t1 : 0; t1->next = 0; "
          "t0->imagedesc = (HSD_ImageDesc*) OP_HSD_ImageDesc[rnd() % 2]; t1->imagedesc = (HSD_ImageDesc*) OP_HSD_ImageDesc[rnd() % 2]; "
          "OJO[NGO].u.dobj = rnd() % 2 ? d1 : 0; }")
WALK_R = ("{ let d0 = addr_of_mut!(OP_HSD_DObj[0]) as *mut HSD_DObj; let d1 = addr_of_mut!(OP_HSD_DObj[1]) as *mut HSD_DObj; "
          "let m0 = addr_of_mut!(OP_HSD_MObj[0]) as *mut HSD_MObj; let m1 = addr_of_mut!(OP_HSD_MObj[1]) as *mut HSD_MObj; "
          "let t0 = addr_of_mut!(OP_HSD_TObj[0]) as *mut HSD_TObj; let t1 = addr_of_mut!(OP_HSD_TObj[1]) as *mut HSD_TObj; "
          "for k in 0..NJO { OJO[k].u.dobj = core::ptr::null_mut(); } "
          "OJO[0].u.dobj = d0; (*d0).next = d1; (*d1).next = core::ptr::null_mut(); (*d0).mobj = if rnd() % 3 != 0 { m0 } else { core::ptr::null_mut() }; (*d1).mobj = m1; "
          "(*m0).tobj = t0; (*m1).tobj = if rnd() % 3 != 0 { t1 } else { core::ptr::null_mut() }; (*t0).next = if rnd() % 2 != 0 { t1 } else { core::ptr::null_mut() }; (*t1).next = core::ptr::null_mut(); "
          "(*t0).imagedesc = addr_of_mut!(OP_HSD_ImageDesc[(rnd() % 2) as usize]) as *mut HSD_ImageDesc; (*t1).imagedesc = addr_of_mut!(OP_HSD_ImageDesc[(rnd() % 2) as usize]) as *mut HSD_ImageDesc; "
          "OJO[NGO].u.dobj = if rnd() % 2 != 0 { d1 } else { core::ptr::null_mut() }; }")


LOAD_C = """        PL[3] = rnd() % 4 ? &OGO[2] : 0; OGO[2].next = &OGO[3]; OGO[3].next = rnd() % 2 ? &OGO[4] : 0; OGO[4].next = 0;         OGO[2].classifier = rnd() % 2 ? 0xC : 5; OGO[3].classifier = rnd() % 2 ? 0xC : 5; OGO[4].classifier = 0xC;         OGO[2].hsd_obj = OP_HSD_LObj[0]; OGO[3].hsd_obj = OP_HSD_LObj[1]; OGO[4].hsd_obj = OP_HSD_LObj[1];         ((HSD_LObj*) OP_HSD_LObj[0])->next = rnd() % 2 ? (HSD_LObj*) OP_HSD_LObj[1] : 0; ((HSD_LObj*) OP_HSD_LObj[1])->next = 0; """
LOAD_R = """        PL[3] = if rnd() % 4 != 0 { addr_of_mut!(OGO[2]) } else { core::ptr::null_mut() };         OGO[2].next = addr_of_mut!(OGO[3]); OGO[3].next = if rnd() % 2 != 0 { addr_of_mut!(OGO[4]) } else { core::ptr::null_mut() }; OGO[4].next = core::ptr::null_mut();         OGO[2].classifier = if rnd() % 2 != 0 { 0xC } else { 5 }; OGO[3].classifier = if rnd() % 2 != 0 { 0xC } else { 5 }; OGO[4].classifier = 0xC;         OGO[2].hsd_obj = addr_of_mut!(OP_HSD_LObj[0]) as *mut c_void; OGO[3].hsd_obj = addr_of_mut!(OP_HSD_LObj[1]) as *mut c_void; OGO[4].hsd_obj = addr_of_mut!(OP_HSD_LObj[1]) as *mut c_void;         (*(addr_of_mut!(OP_HSD_LObj[0]) as *mut HSD_LObj)).next = if rnd() % 2 != 0 { addr_of_mut!(OP_HSD_LObj[1]) as *mut HSD_LObj } else { core::ptr::null_mut() };         (*(addr_of_mut!(OP_HSD_LObj[1]) as *mut HSD_LObj)).next = core::ptr::null_mut(); """


def fn(name, u=None, pre=None, **kw):
    d = dict(name=name)
    if u is not None:
        d["u"] = u
    if pre:
        d["pre_c"], d["pre_rs"] = pre
    d.update(kw)
    return d


SPEC = dict(
    src="src/melee/gr/grizumi.c",
    rust_mod="grizumi",
    bf_skip=["StageCallbacks"],
    patch=[
        # IzumiUnkCC (`u8 pad[0x18]; HSD_GObj* x18`) names Ground.x18 by its GameCube offset 0x18; natively
        # that offset is xC_callback, so the patched C writes the field the offset means
        ("((IzumiUnkCC*) HSD_GObjGetUserData(gp->u.izumi.xCC))->x18 =", "((Ground*) HSD_GObjGetUserData(gp->u.izumi.xCC))->x18 ="),
    ],
    u=UALL,
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_GObjPLinkHead", "HSD_GObj_CameraKind", "HSD_Free"],
    extra_pools=[YP, "IzumiReflection", "HSD_TObj", "HSD_DObj", "HSD_MObj", "HSD_ImageDesc", "HSD_LObj", "UnkArchiveStruct"],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grIz_803E0D60", rs="t::grIz_803E0D60", ty="GrJoint", n=3),
        dict(c="grIz_StageCallbacks", rs="t::grIz_StageCallbacks", ty="StageCallbacks", n=11),
        dict(c="grIz_StageData", rs="t::grIz_StageData", ty="StageData"),
        dict(c="HSD_GObjPLinkHead", rs="t::ext::HSD_GObjPLinkHead", kind="p"),
        dict(c="HSD_GObj_CameraKind", rs="t::ext::HSD_GObj_CameraKind", kind="C"),
        dict(c="PL", rs="PL", kind="p", n=16),
    ] + gro_util.cb_flags("grIz_StageCallbacks", "t::grIz_StageCallbacks", 11),
    data_c=("static int NN, NA;\nHSD_GObj** HSD_GObjPLinkHead; u8 HSD_GObj_CameraKind; static HSD_GObj* PL[16];\n"
            "void HSD_Free(void* p) { lg_s(\"HSD_Free(\"); lg_ptr(p); lg_s(\")\"); lg_c(';'); }\n"
            "int ScriptGame_StageSlotNativeContext(void) { int r; lg_s(\"ScriptGame_StageSlotNativeContext(\"); lg_s(\")\"); r = (int) (rnd() % 2); lg_retu((u32) r); lg_c(';'); return r; }\n"
            "int ScriptGame_StageSlotRetailLoading(void) { int r; lg_s(\"ScriptGame_StageSlotRetailLoading(\"); lg_s(\")\"); r = (int) (rnd() % 2); lg_retu((u32) r); lg_c(';'); return r; }\n"
            "void ScriptGame_StageSlotRetailBuffer(void* p) { lg_s(\"ScriptGame_StageSlotRetailBuffer(\"); lg_ptr(p); lg_s(\")\"); lg_c(';'); }"),
    use_rs="static mut NN: i32 = 0; static mut NA: i32 = 0;\nstatic mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];",
    links_c="""    {
        int k;
        NN = 1; NA = 1;
        yakumono_param = (void*) OP_%s[0];
        for (k = 0; k < 16; k++) PL[k] = 0;
        HSD_GObjPLinkHead = PL;
    }""" % YP,
    links_rs="""    {
        NN = 1; NA = 1;
        t::yakumono_param = addr_of_mut!(OP_%s[0]) as *mut _;
        for k in 0..16 { PL[k] = core::ptr::null_mut(); }
        t::ext::HSD_GObjPLinkHead = addr_of_mut!(PL) as *mut *mut HSD_GObj;
    }""" % YP,
    hooks_rs="""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);
    t::ext::SCRIPTGAME_STAGESLOTNATIVECONTEXT = Some(|| -> i32 { lg_s("ScriptGame_StageSlotNativeContext("); lg_s(")"); let r = (rnd() % 2) as i32; lg_retu(r as u32); lg_c(';'); r });
    t::ext::SCRIPTGAME_STAGESLOTRETAILLOADING = Some(|| -> i32 { lg_s("ScriptGame_StageSlotRetailLoading("); lg_s(")"); let r = (rnd() % 2) as i32; lg_retu(r as u32); lg_c(';'); r });
    melee_world::tu::grinl2::ext::GRDISPLAY_801C5DB0 = Some(|a0: *mut HSD_GObj, a1: i32| { lg_s("grDisplay_801C5DB0("); lg_ptr(a0 as usize); lg_int(a1 as u32); lg_s(")"); lg_c(';'); });
    t::ext::SCRIPTGAME_STAGESLOTRETAILBUFFER = Some(|p: *mut c_void| { lg_s("ScriptGame_StageSlotRetailBuffer("); lg_ptr(p as usize); lg_s(")"); lg_c(';'); });""",
    no_hook=["grDisplay_801C5DB0"],
    outs={"lb_8000B1CC": [2], "HSD_CObjGetEyePosition": [1], "HSD_CObjGetInterest": [1]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[(u32) a0 % NGO] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(a0 as u32 % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetYakumonoParam": ("((void*) OP_%s[0])" % YP, "(addr_of_mut!(OP_%s[0]) as *mut _)" % YP),
        "Ground_801C3FA4": RET_OJO,
        "GObj_Create": RET_OGO,
        "HSD_MemAlloc": ("((void*) OP_IzumiReflection[0])", "(addr_of_mut!(OP_IzumiReflection[0]) as *mut _)"),
        "grDatFiles_801C6330": ("((void*) OP_UnkArchiveStruct[0])", "(addr_of_mut!(OP_UnkArchiveStruct[0]) as *mut _)"),
        "grDatFiles_GetArchive": ("((void*) OP_UnkArchiveStruct[1])", "(addr_of_mut!(OP_UnkArchiveStruct[1]) as *mut _)"),
        "HSD_ArchiveGetPublicAddress": ("((NA || rnd() % 4) ? (void*) OP_HSD_ImageDesc[rnd() % 2] : (void*) 0)",
                                        "(if NA != 0 || rnd() % 4 != 0 { addr_of_mut!(OP_HSD_ImageDesc[(rnd() % 2) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
    },
    fnvals={**{n: "t::" + n for n in CBS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0",
            **{n: "t::" + n for n in ["grIzumi_801CD220", "grIzumi_801CCB90", "grIzumi_801CCEA0"]},
            "HSD_Free": "t::ext::HSD_Free", "HSD_AObjSetFlags": "t::ext::HSD_AObjSetFlags"},
    funcs=[
        fn("grIzumi_StageSlotParamSwap", c_args=["(void*) OP_%s[1]" % YP], rs_args=["addr_of_mut!(OP_%s[1]) as *mut c_void" % YP]),
        fn("grIzumi_801CBB84", c_args=["AI[0] % 2"], rs_args=["(a.i[0] % 2) as _"]),
        fn("grIzumi_801CBB88", pre=(WALK_C, WALK_R)), fn("grIzumi_OnLoad"), fn("grIzumi_OnStart"), fn("grIzumi_801CBCE0"),
        fn("grIzumi_801CBCE8", pre=("NN = (AI[0] % 11) == 3; " + WALK_C, "NN = ((a.i[0] % 11) == 3) as i32; " + WALK_R),
           c_args=["(int) (AI[0] % 11)"], rs_args=["(a.i[0] % 11) as i32"]),
        fn("grIzumi_801CBDD4"), fn("grIzumi_801CBE00"), fn("grIzumi_801CBE08"), fn("grIzumi_801CBE0C"),
        fn("grIzumi_801CBE10"), fn("grIzumi_801CBE54"), fn("grIzumi_801CBE5C"), fn("grIzumi_801CBE60"),
        fn("grIzumi_801CBE64", pre=(WALK_C, WALK_R)),
        fn("grIzumi_801CC0CC"),
        fn("grIzumi_801CC0D4", pre=(FIX_C, FIX_R)),
        fn("grIzumi_801CC338"), fn("grIzumi_801CC33C"), fn("grIzumi_801CC350"),
        fn("grIzumi_801CC358", pre=(P3_C, P3_R)),
        fn("grIzumi_801CCA10"), fn("grIzumi_801CCA14"), fn("grIzumi_801CCA54"), fn("grIzumi_801CCA5C"),
        fn("grIzumi_801CCA60"), fn("grIzumi_801CCA64"), fn("grIzumi_801CCB08"), fn("grIzumi_801CCB10"),
        fn("grIzumi_801CCB14"), fn("grIzumi_801CCB18", pre=("NA = 0;", "NA = 0;")),
        fn("grIzumi_801CCB90", c_args=["&OGO[0]", "AI[0]"], rs_args=["g", "a.i[0] as _"]),
        fn("grIzumi_801CCBDC", c_args=["AF[0]", "&AV3[0]", "AI[0]", "&OJO[AI[1] % NJO]"],
           rs_args=["a.f[0]", "&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[(a.i[1] % NJO as u32) as usize])"]),
        fn("grIzumi_801CCD98"),
        fn("grIzumi_801CCEA0", c_args=["&OGO[1]", "AI[0]"], rs_args=["addr_of_mut!(OGO[1])", "a.i[0] as _"],
           pre=("OGO[1].user_data = OP_IzumiReflection[0];", "OGO[1].user_data = addr_of_mut!(OP_IzumiReflection[0]) as *mut c_void;")),
        fn("grIzumi_801CD090", pre=(WALK_C, WALK_R), c_args=["&OGO[0]", "(HSD_ImageDesc*) OP_HSD_ImageDesc[AI[0] % 2]"],
           rs_args=["g", "addr_of_mut!(OP_HSD_ImageDesc[(a.i[0] % 2) as usize]) as *mut HSD_ImageDesc"]),
        fn("grIzumi_801CD220", pre=(REF_C, REF_R), c_args=["&OGO[0]", "AI[0]"], rs_args=["g", "a.i[0] as _"]),
        fn("grIzumi_801CD278", c_args=["AI[0]"], rs_args=["a.i[0] as _"]),
        fn("grIzumi_801CD280", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
        fn("grIzumi_801CD2D4"),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grIz_803E0D60", rs="t::grIz_803E0D60", ty="GrJoint", n=3),
    dict(c="grIz_StageCallbacks", rs="t::grIz_StageCallbacks", ty="StageCallbacks", n=11),
    dict(c="grIz_StageData", rs="t::grIz_StageData", ty="StageData"),
]))
