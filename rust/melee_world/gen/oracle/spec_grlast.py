import gro_util

CBS = ["stageGObj0_OnInit", "grLast_8021A90C", "grLast_8021A914", "grLast_8021A918", "grLast_8021A91C",
       "grLast_8021A960", "grLast_8021A968", "grLast_8021A96C", "stageGObj2_OnInit", "grLast_8021A99C",
       "grLast_8021A9A4", "grLast_8021A9A8", "grLast_8021A9AC", "grLast_8021A9C4", "grLast_8021AAA8",
       "grLast_8021AAB0", "grLast_8021AB30", "grLast_8021AB34", "grLast_8021AB78", "grLast_8021AB80",
       "grLast_8021AB84", "grLast_8021AB88", "grLast_8021ABCC", "grLast_8021ABD4", "grLast_8021ABD8",
       "grLast_8021ABDC", "grLast_8021AC20", "grLast_8021AC28", "grLast_8021AC2C", "grLast_8021AC30",
       "grLast_8021ADC8", "grLast_8021ADD0", "grLast_8021B23C", "grLast_8021B240", "grLast_8021B284",
       "grLast_8021B28C", "grLast_8021B290", "grLast_8021B294", "grLast_8021B2D8", "grLast_8021B2E0",
       "grLast_8021B2E4", "grLast_OnInit", "grLast_OnDemoInit", "grLast_OnLoad", "grLast_OnStart",
       "grLast_8021A7EC", "grLast_OnTouchLine", "grLast_OnCheckShadowRender"]
YP = "grLast_YakumonoParam"
MAP = ["map"]
ML = ["map", "last"]
LAST = ["last", "nobf"]

RET_OJO = ("((void*) &OJO[rnd() % NJO])", "(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _)")

# the backdrop gobjs of the map gobj: valid, never the map gobj itself
LV_C = "{ int k; for (k = 0; k < 6; k++) OGP[0].u.map.lv_gobj[k] = &OGO[1 + rnd() % (NGO - 1)]; }"
LV_R = "for k in 0..6usize { OGP[0].u.map.lv_gobj[k] = addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]); }"
# some entries NULL (the state-1 setup only fills the empty ones)
LVN_C = "{ int k; for (k = 0; k < 6; k++) OGP[0].u.map.lv_gobj[k] = rnd() % 3 ? &OGO[1 + rnd() % (NGO - 1)] : 0; }"
LVN_R = "for k in 0..6usize { OGP[0].u.map.lv_gobj[k] = if rnd() % 3 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) } else { core::ptr::null_mut() }; }"


def bits(b01=True):
    c = ("OGP[0].u.map.xC4_b0 = 0; OGP[0].u.map.xC4_b1 = AI[2] % 4 == 0; OGP[0].u.map.xC4_b26 = AI[3] % 2; "
         "OGP[0].u.map.xC4_b27 = AI[1] % 2; OGP[0].u.map.xC4_b28 = 0; OGP[0].u.map.xC4_b29 = 0; OGP[0].u.map.xC4_b30 = 0; OGP[0].u.map.xC4_b31 = 0; ")
    r = ("OGP[0].u.map.set_xC4_b0(0); OGP[0].u.map.set_xC4_b1((a.i[2] % 4 == 0) as u32); OGP[0].u.map.set_xC4_b26(a.i[3] % 2); "
         "OGP[0].u.map.set_xC4_b27(a.i[1] % 2); OGP[0].u.map.set_xC4_b28(0); OGP[0].u.map.set_xC4_b29(0); OGP[0].u.map.set_xC4_b30(0); OGP[0].u.map.set_xC4_b31(0); ")
    return c, r


def wide(n):
    """AI[] values are 0..3: widen AI[0] to 0..n-1 for the functions that switch on it."""
    return f"AI[0] = rnd() % {n}; ", f"a.i[0] = rnd() % {n}; "


def state(expr_c, expr_r, n):
    bc, br = bits()
    wc, wr = wide(n)
    return (wc + bc + "OGP[0].u.map.xC4_b2_25 = " + expr_c + "; " + LV_C, wr + br + "OGP[0].u.map.set_xC4_b2_25(" + expr_r + "); " + LV_R)


def fn(name, u=None, pre=None, **kw):
    d = dict(name=name)
    if u is not None:
        d["u"] = u
    if pre:
        d["pre_c"], d["pre_rs"] = pre
    d.update(kw)
    return d


SPEC = dict(
    src="src/melee/gr/grlast.c",
    rust_mod="grlast",
    bf_skip=["StageCallbacks"],
    u=MAP,
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_RandSeedPtr", "Slippi_Codes", "Ground_801C2090"],
    extra_pools=[YP, "HSD_AObj", "HSD_Generator", "HSD_psAppSRT"],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grLast_804D4968", rs="t::grLast_804D4968", ty="GrJoint", n=1),
        dict(c="grLast_StageCallbacks", rs="t::grLast_StageCallbacks", ty="StageCallbacks", n=10),
        dict(c="grNLa_StageData", rs="t::grNLa_StageData", ty="StageData"),
        dict(c="HSD_RandSeedPtr", rs="t::ext::HSD_RandSeedPtr", kind="p"),
        dict(c="SEED", rs="SEED", kind="u"),
    ] + gro_util.cb_flags("grLast_StageCallbacks", "t::grLast_StageCallbacks", 10),
    data_c=("static int NN, NM;\nu32* HSD_RandSeedPtr; static u32 SEED;\n"
            "int Slippi_Codes(void) { int r; lg_s(\"Slippi_Codes(\"); lg_s(\")\"); r = (int) (rnd() % 3); lg_retu((u32) r); lg_c(';'); return r; }\n"
            "int Ground_801C2090(GXColor* c)\n{\n    int r;\n    lg_s(\"Ground_801C2090(\"); lg_ptr((const void*) c); lg_s(\")\");\n"
            "    c->r = rnd() % 256; c->g = rnd() % 256; c->b = rnd() % 256; c->a = 255;\n"
            "    lg_int(c->r); lg_int(c->g); lg_int(c->b); lg_int(c->a);\n"
            "    r = (int) (rnd() % 2); lg_retu((u32) r); lg_c(';'); return r;\n}"),
    use_rs="static mut NN: i32 = 1; static mut NM: i32 = 1; static mut SEED: u32 = 0;",
    links_c="""    {
        int k;
        NN = 1; NM = 1;
        yakumono_param = (struct grLast_YakumonoParam*) OP_%s[0];
        SEED = rnd();
        HSD_RandSeedPtr = &SEED;
        for (k = 0; k < 2; k++) {
            ((HSD_AObj*) OP_HSD_AObj[k])->curr_frame = rnd() %% 3 == 0 ? 100.0f : (float) (rnd() %% 200);
            ((HSD_Generator*) OP_HSD_Generator[k])->appsrt = rnd() %% 4 ? (HSD_psAppSRT*) OP_HSD_psAppSRT[k] : 0;
        }
    }""" % YP,
    links_rs="""    {
        NN = 1; NM = 1;
        t::yakumono_param = addr_of_mut!(OP_%s[0]) as *mut t::grLast_YakumonoParam;
        SEED = rnd();
        t::ext::HSD_RandSeedPtr = addr_of_mut!(SEED);
        for k in 0..2usize {
            (*(addr_of_mut!(OP_HSD_AObj[k]) as *mut HSD_AObj)).curr_frame = if rnd() %% 3 == 0 { 100.0 } else { (rnd() %% 200) as f32 };
            (*(addr_of_mut!(OP_HSD_Generator[k]) as *mut HSD_Generator)).appsrt = if rnd() %% 4 != 0 { addr_of_mut!(OP_HSD_psAppSRT[k]) as *mut HSD_psAppSRT } else { core::ptr::null_mut() };
        }
    }""" % YP,
    hooks_rs="""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);
    t::ext::SLIPPI_CODES = Some(|| -> i32 { lg_s("Slippi_Codes("); lg_s(")"); let r = (rnd() % 3) as i32; lg_retu(r as u32); lg_c(';'); r });
    t::ext::GROUND_801C2090 = Some(|c: *mut GXColor| -> i32 {
        lg_s("Ground_801C2090("); lg_ptr(c as usize); lg_s(")");
        (*c).r = (rnd() % 256) as u8; (*c).g = (rnd() % 256) as u8; (*c).b = (rnd() % 256) as u8; (*c).a = 255;
        lg_int((*c).r as u32); lg_int((*c).g as u32); lg_int((*c).b as u32); lg_int((*c).a as u32);
        let r = (rnd() % 2) as i32; lg_retu(r as u32); lg_c(';'); r });""",
    no_hook=["grDisplay_801C5DB0"],
    outs={"lb_8000B1CC": [2], "lbVector_Sub": [0], "lbVector_Normalize": [0], "lbVector_EulerAnglesFromPartialONB": [0]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[(u32) a0 % NGO] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(a0 as u32 % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetMapGObj": ("((a0 == 1 || a0 == 3 || NM || rnd() % 4) ? (void*) &OGO[(u32) a0 % NGO] : (void*) 0)",
                              "(if a0 == 1 || a0 == 3 || NM != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(a0 as u32 % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetYakumonoParam": ("((void*) OP_%s[0])" % YP, "(addr_of_mut!(OP_%s[0]) as *mut _)" % YP),
        "Ground_801C3FA4": RET_OJO,
        "Stage_80225194": ("(rnd() % 3 == 0 ? 0xB0 : rnd() % 3 == 0 ? 0xFB : (int) (rnd() % 100))",
                           "(if rnd() % 3 == 0 { 0xB0 } else if rnd() % 3 == 0 { 0xFB } else { (rnd() % 100) as i32 })"),
        "grAnime_801C8318": ("((void*) OP_HSD_AObj[rnd() % 2])", "(addr_of_mut!(OP_HSD_AObj[(rnd() % 2) as usize]) as *mut _)"),
        "grAnime_801C83D0": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "grAnime_801C84A4": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "grLib_801C96E8": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
    },
    fnvals={**{n: "t::" + n for n in CBS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=[
        fn("grLast_OnDemoInit", pre=("NM = 0;", "NM = 0;"), c_args=["(AI[0] % 2) ? 26 : 5"], rs_args=["if a.i[0] % 2 != 0 { 26 } else { 5 }"]),
        fn("grLast_OnInit"), fn("grLast_OnLoad"), fn("grLast_OnStart"), fn("grLast_8021A7EC"),
        fn("grLast_8021A7F4", pre=("AI[0] = rnd() % 10; NN = AI[0] == 3;", "a.i[0] = rnd() % 10; NN = (a.i[0] == 3) as i32;"),
           c_args=["(int) (AI[0] % 10)"], rs_args=["(a.i[0] % 10) as i32"]),
        fn("stageGObj0_OnInit"), fn("grLast_8021A90C"), fn("grLast_8021A914"), fn("grLast_8021A918"),
        fn("grLast_8021A91C"), fn("grLast_8021A960"), fn("grLast_8021A968"), fn("grLast_8021A96C"),
        fn("stageGObj2_OnInit"), fn("grLast_8021A99C"), fn("grLast_8021A9A4"), fn("grLast_8021A9A8"),
        fn("grLast_8021A9AC"), fn("grLast_8021A9C4"),
        fn("grLast_8021AAA8"),
        fn("grLast_8021AAB0", u=ML, n=1500, pre=state("1 + AI[0] % 17", "1 + a.i[0] % 17", 17)),
        fn("grLast_8021AB30"), fn("grLast_8021AB34"), fn("grLast_8021AB78"), fn("grLast_8021AB80"),
        fn("grLast_8021AB84"), fn("grLast_8021AB88"), fn("grLast_8021ABCC"), fn("grLast_8021ABD4"),
        fn("grLast_8021ABD8"), fn("grLast_8021ABDC"), fn("grLast_8021AC20"), fn("grLast_8021AC28"),
        fn("grLast_8021AC2C"),
        fn("grLast_8021AC30", u=LAST, n=1500),
        fn("grLast_8021ADC8"),
        fn("grLast_8021ADD0", u=LAST, n=3000,
           pre=("OGP[0].u.last.xE0 = AI[0] % 3 ? (HSD_Generator*) OP_HSD_Generator[AI[0] % 2] : 0;",
                "OGP[0].u.last.xE0 = if a.i[0] % 3 != 0 { addr_of_mut!(OP_HSD_Generator[(a.i[0] % 2) as usize]) as *mut HSD_Generator } else { core::ptr::null_mut() };")),
        fn("grLast_8021B23C"), fn("grLast_8021B240"), fn("grLast_8021B284"), fn("grLast_8021B28C"),
        fn("grLast_8021B290"), fn("grLast_8021B294"), fn("grLast_8021B2D8"), fn("grLast_8021B2E0"),
        fn("grLast_8021B2E4"),
        fn("grLast_8021B2E8", u=ML, n=1500,
           pre=state("AI[0] % 18", "a.i[0] % 18", 18)),
        fn("grLast_8021B5C4", u=MAP, n=1500, pre=state("AI[0] % 19", "a.i[0] % 19", 19)),
        fn("grLast_8021B920", u=ML, n=1500, c_args=["&OGO[0]", "(int) (AI[0] % 19)"], rs_args=["g", "(a.i[0] % 19) as i32"],
           pre=("AI[0] = rnd() % 19; " + LVN_C, "a.i[0] = rnd() % 19; " + LVN_R)),
        fn("grLast_8021C40C", c_args=["&OGO[0]", "((GXColor) { AI[0] * 80, AI[1] * 80, AI[2] * 80, 255 })", "AF[0]"],
           rs_args=["g", "GXColor { r: (a.i[0] * 80) as u8, g: (a.i[1] * 80) as u8, b: (a.i[2] * 80) as u8, a: 255 }", "a.f[0]"]),
        fn("grLast_8021C500", n=1500,
           pre=("OGP[0].u.map.xC4_b27 = AI[0] % 2; OGP[0].u.map.x104 = (float) (AI[1] % 4);",
                "OGP[0].u.map.set_xC4_b27(a.i[0] % 2); OGP[0].u.map.x104 = (a.i[1] % 4) as f32;")),
        fn("grLast_8021C640"), fn("grLast_8021C6AC"),
        fn("grLast_OnTouchLine", c_args=["AI[0]"], rs_args=["a.i[0] as _"]),
        fn("grLast_OnCheckShadowRender", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
        fn("Ground_StageSlotProofCallbacks"),
        fn("Ground_StageSlotFDBackdrop", pre=("NM = 0;", "NM = 0;")),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grLast_804D4968", rs="t::grLast_804D4968", ty="GrJoint", n=1),
    dict(c="grLast_StageCallbacks", rs="t::grLast_StageCallbacks", ty="StageCallbacks", n=10),
    dict(c="grNLa_StageData", rs="t::grNLa_StageData", ty="StageData"),
]))
