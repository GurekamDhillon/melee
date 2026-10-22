CBS = ["grBattle_GObj0", "grBattle_GObj1", "grBattle_GObj2", "grBattle_BG", "grBattle_GObj4", "grBattle_GObj5", "grBattle_GObj6"]
FN = []
for c in CBS:
    FN += [f"{c}_Callback{i}" for i in range(4)]
FN = [f for f in FN if f != "grBattle_BG_Callback2"]
FN += ["grBattle_OnInit", "grBattle_OnDemoInit", "grBattle_OnLoad", "grBattle_OnStart", "grBattle_80219D7C",
       "grBattle_OnTouchLine"]

SPEC = dict(
    src="src/melee/gr/grbattle.c",
    rust_mod="grbattle",
    bf_skip=["StageCallbacks"],
    u=["battle_bg"],
    ext_mods=["grinl"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    statics=[
        dict(c="isDemoFight", rs="t::isDemoFight", kind="C"),
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grNBa_StageCallbacks", rs="t::grNBa_StageCallbacks", ty="StageCallbacks", n=7),
        dict(c="grNBa_StageData", rs="t::grNBa_StageData", ty="StageData"),
    ],
    extra_pools=["grBattle_YakumonoParam"],
    data_c="static int NN;",
    use_rs="static mut NN: i32 = 0;",
    links_c="    NN = 0; yakumono_param = (void*) OP_grBattle_YakumonoParam[0];",
    links_rs="    NN = 0; t::yakumono_param = addr_of_mut!(OP_grBattle_YakumonoParam[0]) as *mut _;",
    hooks_rs="    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);",
    outs={"grLib_801C9A70": [1]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "HSD_JObjGetFlags": ("rnd()", "rnd()"),
        "Stage_80225194": ("(rnd() % 3 == 0 ? 273 : rnd() % 2 ? 245 : 7)", "(if rnd() % 3 == 0 { 273 } else if rnd() % 2 != 0 { 245 } else { 7 })"),
        # the stage gobj / map gobj exist (the C asserts on NULL) unless NN is clear and the dice say so
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetMapGObj": ("((NN || rnd() % 4) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                              "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_801C2CF4": ("(void*) &OJO[rnd() % NJO]", "addr_of_mut!(OJO[(rnd() % NJO as u32) as usize])"),
    },
    fnvals={**{f"{c}_Callback{i}": f"t::{c}_Callback{i}" for c in CBS for i in range(4)},
            "grBattle_OnInit": "t::grBattle_OnInit", "grBattle_OnDemoInit": "t::grBattle_OnDemoInit",
            "grBattle_OnLoad": "t::grBattle_OnLoad", "grBattle_OnStart": "t::grBattle_OnStart",
            "grBattle_80219D7C": "t::grBattle_80219D7C", "grBattle_OnTouchLine": "t::grBattle_OnTouchLine",
            "grBattle_OnCheckShadowRender": "t::grBattle_OnCheckShadowRender",
            "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=FN + [
        dict(name="grBattle_80219D84", pre_c="AI[0] = rnd() % 7;", pre_rs="a.i[0] = rnd() % 7;"),
        dict(name="grBattle_BG_Callback2", pre_c="NN = 1;", pre_rs="NN = 1;"),
        dict(name="setup_unk_gobj", rs_path="t::setup_unk_gobj"),
        dict(name="reset_bg_timer", c_args=["&OGP[0]"], rs_args=["addr_of_mut!(OGP[0])"], rs_path="t::reset_bg_timer"),
        dict(name="grBattle_OnCheckShadowRender", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"],
             rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
    ],
)
