import gro_util

CBS = ["grKinokoRoute_80207634", "grKinokoRoute_802078E8", "grKinokoRoute_802078F0", "grKinokoRoute_80207A94",
       "grKinokoRoute_80207ADC", "grKinokoRoute_80207B20", "grKinokoRoute_80207B28", "grKinokoRoute_80207B2C",
       "grKinokoRoute_80207A98", "grKinokoRoute_80207ACC", "grKinokoRoute_80207AD4", "grKinokoRoute_80207AD8",
       "grKinokoRoute_80207B5C", "grKinokoRoute_80207C80", "grKinokoRoute_80207C88", "grKinokoRoute_80208368"]
OTHERS = ["grKinokoRoute_80207420", "grKinokoRoute_8020741C", "grKinokoRoute_80207490", "grKinokoRoute_802074D8",
          "grKinokoRoute_80207544", "grKinokoRoute_80208754", "grKinokoRoute_8020875C", "grKinokoRoute_80207B30",
          "grKinokoRoute_80208660", "grKinokoRoute_80208480", "grKinokoRoute_802084B4"]

RET_OGO = ("((void*) &OGO[rnd() % NGO])", "(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _)")
NJ = ("((NN || rnd() % 4) ? (void*) &OJO[rnd() % NJO] : (void*) 0)",
      "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })")
# entries[i].jobj of the map gobj's Ground: some set, some not
ENT_C = "{ int k; for (k = 0; k < 4; k++) OGP[0].u.kinokoroute.entries[k].jobj = rnd() % 3 ? &OJO[rnd() % NJO] : 0; }"
ENT_R = ("for k in 0..4usize { OGP[0].u.kinokoroute.entries[k].jobj = if rnd() % 3 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { core::ptr::null_mut() }; }")
ITEM_C = ("OGO[0].user_data = OP_Item[0]; ((Item*) OP_Item[0])->xDD4_itemVar.yaku.x4 = AI[0] % 2 ? &OJO[1 + AI[1] % 3] : 0;")
ITEM_R = ("OGO[0].user_data = addr_of_mut!(OP_Item[0]) as *mut c_void; "
          "(*(addr_of_mut!(OP_Item[0]) as *mut Item)).xDD4_itemVar.yaku.anon_4.x4 = if a.i[0] % 2 != 0 { addr_of_mut!(OJO[1 + (a.i[1] % 3) as usize]) } else { core::ptr::null_mut() };")

# callbacks of both union kinds may run on any stage Ground: normalise both members' pointers, skip the
# bitfield byte (see gro.py)
UBOTH = ["kinokoroute", "kinokoroute2", "nobf"]

SPEC = dict(
    src="src/melee/gr/grkinokoroute.c",
    rust_mod="grkinokoroute",
    includes=["melee/gr/types.h", "melee/gr/forward.h", "melee/gr/ground.h", "sysdolphin/baselib/gobj.h", "sysdolphin/baselib/jobj.h", "melee/it/types.h"],
    bf_skip=["StageCallbacks", "Item"],
    u=["kinokoroute"],  # kinokoroute2 starts with a bitfield byte that overlays entries[0].pos.x: one union member per function
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_GObjPLinkHead", "efSync_Spawn"],
    extra_pools=["StageInfo", "Item", "HSD_Generator", "HSD_psAppSRT"],
    patch=[
        # two calls in one argument list: the shipping PPC build evaluates left to right, this
        # x86-64 MSVC-target oracle right to left
        ("grZakoGenerator_801CAEB0(Ground_801C5840(), Ground_801C5940());",
         "{ int a_ = Ground_801C5840(); int b_ = Ground_801C5940(); grZakoGenerator_801CAEB0(a_, b_); }"),
        ("    struct {\n        char pad[0xDD8];\n        HSD_JObj* jobj;\n    }* gp = gobj->user_data;",
         "    struct { HSD_JObj* jobj; } gp_s = { ((Item*) gobj->user_data)->xDD4_itemVar.yaku.x4 }, *gp = &gp_s;"),
        ("((UnkFlagStruct*) ((u8*) gp2 + 0xC4))->b0 = 1;",
         "((Ground*) gp2)->u.kinokoroute2.flags_0 = 1;"),
    ],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grNKr_StageCallbacks", rs="t::grNKr_StageCallbacks", ty="StageCallbacks", n=4),
        dict(c="grNKr_StageData", rs="t::grNKr_StageData", ty="StageData"),
        dict(c="HSD_GObjPLinkHead", rs="t::ext::HSD_GObjPLinkHead", kind="p"),
        dict(c="PL", rs="PL", kind="p", n=16),
    ] + gro_util.cb_flags("grNKr_StageCallbacks", "t::grNKr_StageCallbacks", 4),
    data_c=("static int NN, NS;\nHSD_GObj** HSD_GObjPLinkHead; static HSD_GObj* PL[16];\n"
            "void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...)\n{\n    __builtin_va_list ap; Vec3* v; void* r;\n"
            "    __builtin_va_start(ap, a1); v = __builtin_va_arg(ap, Vec3*); __builtin_va_end(ap);\n"
            "    lg_s(\"efSync_Spawn(\"); lg_int((u32) a0); lg_ptr((const void*) a1); lg_v3((const Vec3*) v); lg_s(\")\");\n"
            "    r = gen_p(); lg_retp(r); lg_c(';'); return r;\n}"),
    use_rs="static mut NN: i32 = 0; static mut NS: i32 = 1;\nstatic mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];",
    links_c="""    {
        int k;
        NN = 0; NS = 1;
        yakumono_param = (void*) OP_StageInfo[0];
        for (k = 0; k < 16; k++) PL[k] = 0;
        HSD_GObjPLinkHead = PL;
        PL[9] = rnd() % 4 ? &OGO[2] : 0; OGO[2].next = &OGO[3]; OGO[3].next = rnd() % 2 ? &OGO[4] : 0; OGO[4].next = 0;
        for (k = 0; k < 2; k++) ((HSD_Generator*) OP_HSD_Generator[k])->appsrt = (HSD_psAppSRT*) OP_HSD_psAppSRT[k];
    }""",
    links_rs="""    {
        NN = 0; NS = 1;
        t::yakumono_param = addr_of_mut!(OP_StageInfo[0]) as *mut _;
        for k in 0..16 { PL[k] = core::ptr::null_mut(); }
        t::ext::HSD_GObjPLinkHead = addr_of_mut!(PL) as *mut *mut HSD_GObj;
        PL[9] = if rnd() % 4 != 0 { addr_of_mut!(OGO[2]) } else { core::ptr::null_mut() };
        OGO[2].next = addr_of_mut!(OGO[3]); OGO[3].next = if rnd() % 2 != 0 { addr_of_mut!(OGO[4]) } else { core::ptr::null_mut() }; OGO[4].next = core::ptr::null_mut();
        for k in 0..2usize { (*(addr_of_mut!(OP_HSD_Generator[k]) as *mut HSD_Generator)).appsrt = addr_of_mut!(OP_HSD_psAppSRT[k]) as *mut HSD_psAppSRT; }
    }""",
    hooks_rs=("    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);\n"
              "    melee_world::tu::grkinokoroute::ext::EFSYNC_SPAWN = Some(|a0, a1, a2| { lg_s(\"efSync_Spawn(\"); lg_int(a0 as u32); lg_ptr(a1 as usize); lg_v3(a2 as *const Vec3); lg_s(\")\"); let r = gen_p(); lg_retp(r as usize); lg_c(';'); r });"),
    outs={"lbVector_Sub": [0], "lbVector_Add": [0], "lbVector_Diff": [2], "lbVector_Normalize": [0],
          "Ground_801C2D24": [1], "ftLib_80086644": [1], "ftLib_80086684": [1],
          "Stage_UnkSetVec3TCam_Offset": [0], "lb_8000B1CC": [2]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetStageGObj": ("(NS && (NN || rnd() % 4) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                                "(if NS != 0 && (NN != 0 || rnd() % 4 != 0) { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetYakumonoParam": ("((void*) OP_StageInfo[0])", "(addr_of_mut!(OP_StageInfo[0]) as *mut _)"),
        "Ground_801C2CF4": NJ,
        "Ground_801C3DB4": ("(rnd() % 3 == 0 ? -1 : (int) (rnd() % 8))", "(if rnd() % 3 == 0 { -1 } else { (rnd() % 8) as i32 })"),
        "Ground_801C5764": ("(int) (rnd() % 3)", "((rnd() % 3) as i32)"),
        "itGetKind": ("(rnd() % 3 == 0 ? 160 : (int) (rnd() % 200))", "(if rnd() % 3 == 0 { 160 } else { (rnd() % 200) as i32 })"),
    },
    fnvals={**{n: "t::" + n for n in CBS + OTHERS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=[dict(name=n) for n in ("grKinokoRoute_8020741C", "grKinokoRoute_80207544", "grKinokoRoute_80208754",
                                  "grKinokoRoute_80207A94", "grKinokoRoute_80207AD4", "grKinokoRoute_80207AD8",
                                  "grKinokoRoute_80207B28", "grKinokoRoute_80207B2C", "grKinokoRoute_80208368",
                                  "grKinokoRoute_802078E8", "grKinokoRoute_80207ACC", "grKinokoRoute_80207B20",
                                  "grKinokoRoute_80207C80", "grKinokoRoute_802087B0")] + [
        dict(name="grKinokoRoute_80207420", pre_c="NN = 1;", pre_rs="NN = 1;", u=UBOTH),
        dict(name="grKinokoRoute_80207490"),
        dict(name="grKinokoRoute_802074D8"),
        dict(name="grKinokoRoute_8020754C", c_args=["AI[0] % 4"], rs_args=["(a.i[0] % 4) as _"], pre_c="NN = 1;", pre_rs="NN = 1;", u=UBOTH),
        dict(name="grKinokoRoute_80207634"),
        dict(name="grKinokoRoute_802078F0", pre_c=ENT_C, pre_rs=ENT_R),
        dict(name="grKinokoRoute_80207A98"),
        dict(name="grKinokoRoute_80207ADC"),
        dict(name="grKinokoRoute_80207B30"),
        dict(name="grKinokoRoute_80207B5C", pre_c="NN = 1;", pre_rs="NN = 1;", u=["kinokoroute2"]),
        dict(name="grKinokoRoute_80207C88", pre_c="NN = 1; NS = 0;", pre_rs="NN = 1; NS = 0;", u=["kinokoroute2"]),
        dict(name="grKinokoRoute_8020836C", c_args=["&OGO[0]", "AI[0] % 2"], rs_args=["g", "(a.i[0] % 2) as _"]),
        dict(name="grKinokoRoute_80208480", c_args=["AI[0] % 3"], rs_args=["(a.i[0] % 3) as _"]),
        dict(name="grKinokoRoute_802084B4", pre_c=ITEM_C, pre_rs=ITEM_R, u=["kinokoroute2"]),
        dict(name="grKinokoRoute_80208564"),
        dict(name="grKinokoRoute_80208660", c_args=["AI[0]", "&OGO[0]"], rs_args=["a.i[0] as _", "g"]),
        dict(name="grKinokoRoute_802086EC", c_args=["&AV3[0]", "AF[0]"], rs_args=["&mut a.v3[0]", "a.f[0]"]),
        dict(name="grKinokoRoute_8020875C", c_args=["&AV3[0]", "AI[0]", "&OJO[AI[1] % NGO]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[(a.i[1] % NGO as u32) as usize])"]),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grNKr_StageCallbacks", rs="t::grNKr_StageCallbacks", ty="StageCallbacks", n=4),
    dict(c="grNKr_StageData", rs="t::grNKr_StageData", ty="StageData"),
]))
