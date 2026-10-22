J = lambda e: dict(c=f"&OJO[{e}]", rs=f"addr_of_mut!(OJO[{e}])")
JA = ["&OJO[AI[2] % NGO]"], ["addr_of_mut!(OJO[(a.i[2] % NGO as u32) as usize])"]
GEN_C = "(HSD_Generator*) OP_HSD_Generator[AI[0] % 2]"
GEN_R = "addr_of_mut!(OP_HSD_Generator[(a.i[0] % 2) as usize]) as *mut HSD_Generator"
# generator list: GEN(0) -> GEN(1); their jobj fields hit the argument jobj's tree some of the time
PRE_GEN_C = ("GEN(0)->jobj = &OJO[(AI[1] % NGO) + NGO * (AI[2] % 3)]; "
             "GEN(1)->jobj = &OJO[(AI[2] % NGO) + NGO * (AI[3] % 3)];")
PRE_GEN_R = ("(*gen_(0)).jobj = addr_of_mut!(OJO[((a.i[1] % NGO as u32) + NGO as u32 * (a.i[2] % 3)) as usize]); "
             "(*gen_(1)).jobj = addr_of_mut!(OJO[((a.i[2] % NGO as u32) + NGO as u32 * (a.i[3] % 3)) as usize]);")

SPEC = dict(
    src="src/melee/gr/grlib.c",
    rust_mod="grlib",
    ext_mods=["grinl"],
    u=[], bf_skip=["Fighter", "Item"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_GObjPLinkHead", "hsd_804D78FC", "HSD_GObj_JObjKind"],
    extra_pools=["Fighter", "Item", "DynamicModelDesc", "DynamicsDesc", "HSD_AObj", "HSD_Generator"],
    statics=[
        dict(c="grLib_8049EF58", rs="t::grLib_8049EF58", kind="f", n=12),
        dict(c="hsd_804D78FC", rs="t::ext::hsd_804D78FC", kind="p"),
        dict(c="HSD_GObjPLinkHead", rs="t::ext::HSD_GObjPLinkHead", kind="p"),
        dict(c="HSD_GObj_JObjKind", rs="t::ext::HSD_GObj_JObjKind", kind="C"),
        dict(c="PL", rs="PL", kind="p", n=16),
        dict(c="QA", rs="QA", kind="p", n=4),
    ],
    data_c=("HSD_GObj** HSD_GObjPLinkHead; HSD_Generator* hsd_804D78FC; u8 HSD_GObj_JObjKind;\n"
            "static HSD_GObj* PL[16]; static HSD_AnimJoint* QA[4];\n"
            "#define GEN(i) ((HSD_Generator*) OP_HSD_Generator[i])"),
    use_rs=("static mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];\n"
            "static mut QA: [*mut HSD_AnimJoint; 4] = [core::ptr::null_mut(); 4];\n"
            "unsafe fn gen_(i: usize) -> *mut HSD_Generator { addr_of_mut!(OP_HSD_Generator[i]) as *mut HSD_Generator }"),
    links_c="""    {
        int k;
        for (k = 0; k < 16; k++) PL[k] = 0;
        HSD_GObjPLinkHead = PL;
        PL[8] = &OGO[0]; OGO[0].next = &OGO[1]; OGO[1].next = rnd() % 2 ? &OGO[4] : 0;
        OGO[0].user_data = OP_Fighter[0]; OGO[1].user_data = OP_Fighter[1]; OGO[4].user_data = OP_Fighter[0];
        PL[9] = rnd() % 4 ? &OGO[2] : 0; OGO[2].next = &OGO[3]; OGO[3].next = 0;
        OGO[2].user_data = OP_Item[0]; OGO[3].user_data = OP_Item[1];
        hsd_804D78FC = rnd() % 5 ? GEN(0) : 0; GEN(0)->next = GEN(1); GEN(1)->next = 0;
        GEN(0)->appsrt = rnd() % 2 ? (HSD_psAppSRT*) OP_HSD_psAppSRT[0] : 0;
        GEN(1)->appsrt = rnd() % 2 ? (HSD_psAppSRT*) OP_HSD_psAppSRT[1] : 0;
        for (k = 0; k < 4; k++) QA[k] = (HSD_AnimJoint*) ((unsigned char*) OSCR + 8 * k);
        stage_info.quake_model_set = rnd() % 4 ? (DynamicModelDesc*) OP_DynamicModelDesc[0] : 0;
        ((DynamicModelDesc*) OP_DynamicModelDesc[0])->anims = QA;
        OJO[0].aobj = rnd() % 3 ? (HSD_AObj*) OP_HSD_AObj[rnd() % 2] : 0;
        OJO[0].flags |= 0;
    }""",
    links_rs="""    {
        for k in 0..16 { PL[k] = core::ptr::null_mut(); }
        t::ext::HSD_GObjPLinkHead = addr_of_mut!(PL) as *mut *mut HSD_GObj;
        PL[8] = addr_of_mut!(OGO[0]); OGO[0].next = addr_of_mut!(OGO[1]);
        OGO[1].next = if rnd() % 2 != 0 { addr_of_mut!(OGO[4]) } else { core::ptr::null_mut() };
        OGO[0].user_data = addr_of_mut!(OP_Fighter[0]) as *mut c_void; OGO[1].user_data = addr_of_mut!(OP_Fighter[1]) as *mut c_void;
        OGO[4].user_data = addr_of_mut!(OP_Fighter[0]) as *mut c_void;
        PL[9] = if rnd() % 4 != 0 { addr_of_mut!(OGO[2]) } else { core::ptr::null_mut() };
        OGO[2].next = addr_of_mut!(OGO[3]); OGO[3].next = core::ptr::null_mut();
        OGO[2].user_data = addr_of_mut!(OP_Item[0]) as *mut c_void; OGO[3].user_data = addr_of_mut!(OP_Item[1]) as *mut c_void;
        t::ext::hsd_804D78FC = if rnd() % 5 != 0 { gen_(0) } else { core::ptr::null_mut() };
        (*gen_(0)).next = gen_(1); (*gen_(1)).next = core::ptr::null_mut();
        (*gen_(0)).appsrt = if rnd() % 2 != 0 { addr_of_mut!(OP_HSD_psAppSRT[0]) as *mut HSD_psAppSRT } else { core::ptr::null_mut() };
        (*gen_(1)).appsrt = if rnd() % 2 != 0 { addr_of_mut!(OP_HSD_psAppSRT[1]) as *mut HSD_psAppSRT } else { core::ptr::null_mut() };
        for k in 0..4 { QA[k] = (addr_of_mut!(OSCR) as *mut u8).add(8 * k) as *mut HSD_AnimJoint; }
        (*(addr_of_mut!(G_stage_info) as *mut StageInfo)).quake_model_set = if rnd() % 4 != 0 { addr_of_mut!(OP_DynamicModelDesc[0]) as *mut DynamicModelDesc } else { core::ptr::null_mut() };
        (*(addr_of_mut!(OP_DynamicModelDesc[0]) as *mut DynamicModelDesc)).anims = addr_of_mut!(QA) as *mut *mut HSD_AnimJoint;
        OJO[0].aobj = if rnd() % 3 != 0 { addr_of_mut!(OP_HSD_AObj[(rnd() % 2) as usize]) as *mut HSD_AObj } else { core::ptr::null_mut() };
    }""",
    hooks_rs="    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);",
    outs={"Ground_801C2D24": [1]},
    rets={
        "itGetKind": ("(rnd() % 3 == 0 ? 160 : (int) (rnd() % 200))", "(if rnd() % 3 == 0 { 160 } else { (rnd() % 200) as i32 })"),
    },
    fnvals={
        "grLib_801C9BC8": "t::grLib_801C9BC8", "grLib_801C9C40": "t::grLib_801C9C40",
        "HSD_AObjSetFlags": "t::ext::HSD_AObjSetFlags",
    },
    funcs=[
        dict(name="grLib_801C96E8"),
        dict(name="grLib_801C96F8", rs_path="t::grLib_801C96F8"),
        dict(name="grLib_801C97DC", c_args=["AI[0]", "AI[1]", "&OJO[AI[2] % NGO]"], rs_args=["a.i[0] as _", "a.i[1] as _", "addr_of_mut!(OJO[(a.i[2] % NGO as u32) as usize])"]),
        dict(name="grLib_801C9808", c_args=["AI[0]", "AI[1]", "&OJO[AI[2] % NGO]"], rs_args=["a.i[0] as _", "a.i[1] as _", "addr_of_mut!(OJO[(a.i[2] % NGO as u32) as usize])"]),
        dict(name="grLib_801C9834", c_args=[GEN_C], rs_args=[GEN_R]),
        dict(name="grLib_801C9854", c_args=["&OJO[AI[2] % NGO]"], rs_args=["addr_of_mut!(OJO[(a.i[2] % NGO as u32) as usize])"]),
        dict(name="grLib_801C9874", c_args=[GEN_C], rs_args=[GEN_R]),
        dict(name="grLib_801C98A0", c_args=["AI[0] % 5 ? &OJO[AI[1] % NGO] : 0"], rs_args=["if a.i[0] % 5 != 0 { addr_of_mut!(OJO[(a.i[1] % NGO as u32) as usize]) } else { core::ptr::null_mut() }"],
             pre_c=PRE_GEN_C, pre_rs=PRE_GEN_R),
        dict(name="grLib_801C9908", c_args=["AI[0] % 5 ? &OJO[AI[1] % NGO] : 0"], rs_args=["if a.i[0] % 5 != 0 { addr_of_mut!(OJO[(a.i[1] % NGO as u32) as usize]) } else { core::ptr::null_mut() }"],
             pre_c=PRE_GEN_C, pre_rs=PRE_GEN_R),
        dict(name="grLib_801C99C0", c_args=["AI[0]", "AI[1]", "&OJO[AI[2] % NGO]", "AI[3] % 2"], rs_args=["a.i[0] as _", "a.i[1] as _", "addr_of_mut!(OJO[(a.i[2] % NGO as u32) as usize])", "(a.i[3] % 2) as _"]),
        dict(name="grLib_801C9A10"),
        dict(name="grLib_801C9A70"),
        dict(name="grLib_801C9B20", c_args=["&OJO[0]", "(DynamicsDesc*) OP_DynamicsDesc[0]", "(DynamicsDesc*) OP_DynamicsDesc[1]"],
             rs_args=["addr_of_mut!(OJO[0])", "addr_of_mut!(OP_DynamicsDesc[0]) as *mut DynamicsDesc", "addr_of_mut!(OP_DynamicsDesc[1]) as *mut DynamicsDesc"]),
        dict(name="grLib_801C9B6C", c_args=["(DynamicsDesc*) OP_DynamicsDesc[AI[0] % 2]"], rs_args=["addr_of_mut!(OP_DynamicsDesc[(a.i[0] % 2) as usize]) as *mut DynamicsDesc"]),
        dict(name="grLib_801C9B8C", c_args=["(DynamicsDesc*) OP_DynamicsDesc[AI[0] % 2]"], rs_args=["addr_of_mut!(OP_DynamicsDesc[(a.i[0] % 2) as usize]) as *mut DynamicsDesc"]),
        dict(name="grLib_801C9BC8"),
        dict(name="grLib_801C9C40"),
        dict(name="grLib_801C9CEC", c_args=["AI[0] % 6"], rs_args=["(a.i[0] % 6) as _"]),
        dict(name="grLib_801C9E40"),
        dict(name="grLib_801C9E50"),
        dict(name="grLib_801C9E60", pre_c="stage_info.grkind = rnd() % 30;",
             pre_rs="(*(addr_of_mut!(G_stage_info) as *mut StageInfo)).grkind = rnd() % 30;"),
        dict(name="grLib_801C9EE8"),
    ],
)
