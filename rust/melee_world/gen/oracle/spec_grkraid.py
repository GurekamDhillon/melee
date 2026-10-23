import gro_util

CBS = ["stageGObj0_OnInit", "grKraid_801FE1DC", "grKraid_801FE1E4", "grKraid_801FE1E8",
       "grKraid_801FE1EC", "grKraid_801FE2C8", "grKraid_801FE2D0", "grKraid_801FE35C",
       "grKraid_801FE360", "grKraid_801FE3A4", "grKraid_801FE3AC", "grKraid_801FE3B0",
       "grKraid_801FE3B4", "grKraid_801FE438", "grKraid_801FE440", "grKraid_801FE6D4",
       "grKraid_801FE818", "grKraid_801FE9F8", "grKraid_801FEA00", "grKraid_801FF14C"]
G1 = ["grKraid_801FE1DC", "grKraid_801FE1E4", "grKraid_801FE1E8", "grKraid_801FE1EC", "grKraid_801FE2C8", "grKraid_801FE2D0",
      "grKraid_801FE35C", "grKraid_801FE360", "grKraid_801FE3A4", "grKraid_801FE3AC", "grKraid_801FE3B0", "grKraid_801FE3B4",
      "grKraid_801FE438", "grKraid_801FE440", "grKraid_801FE6D4", "grKraid_801FE818", "grKraid_801FE9F8", "grKraid_801FEA00",
      "grKraid_801FF14C", "grKraid_801FF150", "stageGObj0_OnInit"]
YP = "grKraid_YakumonoParam"
# kraid2 byte indices that index kraid_pos_x[] must be 0..5
K2_PRE_C = "OGP[0].u.kraid2.x2 = rnd() % 6; OGP[0].u.kraid2.x3 = rnd() % 6; OGP[0].u.kraid2.x4 = rnd() % 6;"
K2_PRE_R = ("let g0 = addr_of_mut!(OGP[0]); (*g0).u.kraid2.x2 = (rnd() % 6) as i8; (*g0).u.kraid2.x3 = (rnd() % 6) as i8; "
            "(*g0).u.kraid2.x4 = (rnd() % 6) as i8;")
RET_OGO = ("((void*) &OGO[rnd() % NGO])", "(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _)")
RET_OJO = ("((void*) &OJO[rnd() % NJO])", "(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _)")

SPEC = dict(
    src="src/melee/gr/grkraid.c",
    rust_mod="grkraid",
    bf_skip=["StageCallbacks"],
    u=["kraid", "kraid2"],
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_PadMasterStatus", "Mex_GrBind"],
    extra_pools=[YP],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grKr_804D6A0C", rs="t::grKr_804D6A0C", kind="i"),
        dict(c="grKr_StageCallbacks", rs="t::grKr_StageCallbacks", ty="StageCallbacks", n=5),
        dict(c="grKr_StageData", rs="t::grKr_StageData", ty="StageData"),
        dict(c="grKr_803E4C78", rs="t::grKr_803E4C78", ty="GrJoint", n=6),
        dict(c="HSD_PadMasterStatus", rs="t::ext::HSD_PadMasterStatus", ty="HSD_PadStatus", n=4, fill=True),
    ] + gro_util.cb_flags("grKr_StageCallbacks", "t::grKr_StageCallbacks", 5),
    data_c="static int NN;\nHSD_PadStatus HSD_PadMasterStatus[4];\nvoid* Mex_GrBind(void* fn) { return fn; }",
    use_rs="static mut NN: i32 = 0;",
    links_c="    NN = 0; yakumono_param = (void*) OP_%s[0];" % YP,
    links_rs="    NN = 0; t::yakumono_param = addr_of_mut!(OP_%s[0]) as *mut _;" % YP,
    hooks_rs="    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);",
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetMapGObj": RET_OGO,
        "Ground_801C3FA4": RET_OJO,
        "Ground_GetYakumonoParam": ("((void*) OP_%s[0])" % YP, "(addr_of_mut!(OP_%s[0]) as *mut _)" % YP),
    },
    outs={"lb_8000B1CC": [2]},
    fnvals={**{n: "t::" + n for n in CBS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0",
            **{n: "t::" + n for n in ["grKraid_OnInit", "grKraid_OnDemoInit", "grKraid_OnLoad", "grKraid_OnStart", "grKraid_801FE0BC",
                                       "grKraid_OnTouchLine", "grKraid_OnCheckShadowRender"]}},
    funcs=[
        "grKraid_OnDemoInit", "grKraid_OnLoad", "grKraid_801FE0BC", "grKraid_OnTouchLine",
        dict(name="grKraid_OnInit", pre_c="NN = 1;", pre_rs="NN = 1;"),
        dict(name="grKraid_OnStart"),
        dict(name="grKraid_OnCheckShadowRender", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
        dict(name="grKraid_801FE0C4", c_args=["AI[0] % 5"], rs_args=["(a.i[0] % 5) as _"]),
    ] + [dict(name=n, **({"pre_c": K2_PRE_C, "pre_rs": K2_PRE_R} if n in ("grKraid_801FEA00", "grKraid_801FE818") else {})) for n in G1] + [
        dict(name="grKraid_801FE6D8", c_args=["&OJO[AI[0] % NGO]", "AF[0]"], rs_args=["addr_of_mut!(OJO[(a.i[0] % NGO as u32) as usize])", "a.f[0]"]),
        dict(name="grKraid_801FEE54", pre_c=K2_PRE_C, pre_rs=K2_PRE_R),
        dict(name="grKraid_801FF068"),
        dict(name="grKraid_801FF0E0", c_args=["&OGO[0]", "AI[0] % 2"], rs_args=["g", "(a.i[0] % 2) as _"]),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grKr_StageCallbacks", rs="t::grKr_StageCallbacks", ty="StageCallbacks", n=5),
    dict(c="grKr_StageData", rs="t::grKr_StageData", ty="StageData"),
    dict(c="grKr_803E4C78", rs="t::grKr_803E4C78", ty="GrJoint", n=6),
]))
