STR = "(AI[0] & 1) ? (void*) OSTR : (void*) 0"
STR_RS = "if a.i[0] & 1 != 0 { addr_of_mut!(OSTR) as *mut c_void } else { core::ptr::null_mut() }"

SPEC = dict(
    src="src/melee/gr/grdatfiles.c",
    rust_mod="grdatfiles",
    ext_mods=["grinl"],
    pre_c="UnkArchiveStruct* ScriptGame_StageSlotArchive(int);",
    globals=[dict(c="stage_info", ty="StageInfo")],
    variadic={"lbArchive_800171CC": "si", "lbArchive_80016DBC": "si"},
    data_c="static char OSTR[16] = \"GrXx.dat\";\nstatic UnkStageDat OSD[4];\nstatic struct UnkStageDat_x8_t OX8[4];\nstatic UnkStageDatInternal OSI[6];\nstatic UnkStageDatInternal* OPI[6];",
    use_rs="static mut OSTR: [u8; 16] = *b\"GrXx.dat\\0\\0\\0\\0\\0\\0\\0\\0\";\nstatic mut OSD: [UnkStageDat; 4] = unsafe { core::mem::zeroed() };\nstatic mut OX8: [UnkStageDat_x8_t; 4] = unsafe { core::mem::zeroed() };\nstatic mut OSI: [UnkStageDatInternal; 6] = unsafe { core::mem::zeroed() };\nstatic mut OPI: [*mut UnkStageDatInternal; 6] = [core::ptr::null_mut(); 6];",
    statics=[
        dict(c="grDatFiles_8049EE10", rs="t::grDatFiles_8049EE10", ty="UnkArchiveStruct", n=4, fill=True),
        dict(c="grDatFiles_803E07E4", rs="t::grDatFiles_803E07E4", ty="StageParam"),
        dict(c="grDatFiles_803E0848", rs="t::grDatFiles_803E0848", ty="GroundParam"),
        dict(c="grDatFiles_803E0924", rs="t::grDatFiles_803E0924", ty="UnkStageDat"),
        dict(c="OSD", rs="OSD", ty="UnkStageDat", n=4, fill=True),
        dict(c="OX8", rs="OX8", ty="UnkStageDat_x8_t", n=4, fill=True),
        dict(c="OSI", rs="OSI", ty="UnkStageDatInternal", n=6, fill=True),
        dict(c="OPI", rs="OPI", kind="p", n=6),
    ],
    links_c="""    {
        int i, j; u32 m = rnd() % 4;
        for (i = 0; i < 6; i++) OPI[i] = (rnd() % 3) ? &OSI[i] : (UnkStageDatInternal*) 0;
        for (j = 0; j < 4; j++) OX8[j].unk0 = (rnd() & 1) ? (void*) &OGO[j] : (void*) 0;
        for (i = 0; i < 4; i++) { OSD[i].unk8 = OX8; OSD[i].unk28 = OPI; OSD[i].unk2C = (s32) (rnd() % 6); OSD[i].unkC = (s32) (rnd() % 5); }
        for (i = 0; i < (int) m; i++) { grDatFiles_8049EE10[i].unk0 = (void*) &OGO[i]; grDatFiles_8049EE10[i].unk4 = (rnd() % 4) ? &OSD[i] : (UnkStageDat*) 0; }
    }""",
    links_rs="""    {
        let m = rnd() % 4;
        for i in 0..6 { OPI[i] = if rnd() % 3 != 0 { addr_of_mut!(OSI[i]) } else { core::ptr::null_mut() }; }
        for j in 0..4 { OX8[j].unk0 = if rnd() & 1 != 0 { addr_of_mut!(OGO[j]) as *mut _ } else { core::ptr::null_mut() }; }
        for i in 0..4 { OSD[i].unk8 = addr_of_mut!(OX8) as *mut UnkStageDat_x8_t; OSD[i].unk28 = addr_of_mut!(OPI) as *mut *mut UnkStageDatInternal; OSD[i].unk2C = (rnd() % 6) as i32; OSD[i].unkC = (rnd() % 5) as i32; }
        for i in 0..(m as usize) { t::grDatFiles_8049EE10[i].unk0 = addr_of_mut!(OGO[i]) as *mut _; t::grDatFiles_8049EE10[i].unk4 = if rnd() % 4 != 0 { addr_of_mut!(OSD[i]) } else { core::ptr::null_mut() }; }
    }""",
    hooks_rs="    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);",
    rets={"HSD_ArchiveGetPublicAddress": ("(rnd() % 5 ? (void*) &OSD[rnd() % 4] : (void*) 0)",
                                           "(if rnd() % 5 != 0 { addr_of_mut!(OSD[(rnd() % 4) as usize]) as *mut c_void } else { core::ptr::null_mut() })")},
    fnvals={},
    funcs=[
        dict(name="grDatFiles_801C5FC0", c_args=["(void*) &OGO[0]", "(void*) &OGO[1]", "AI[0]"],
             rs_args=["addr_of_mut!(OGO[0]) as *mut _", "addr_of_mut!(OGO[1]) as *mut _", "a.i[0]"]),
        dict(name="grDatFiles_801C6038", c_args=[STR, "(s32) (AI[1] & 1)", "(s32) (AI[2] & 1)"],
             rs_args=[STR_RS, "(a.i[1] & 1) as i32", "(a.i[2] & 1) as i32"]),
        dict(name="grDatFiles_801C6228", c_args=["(AI[0] == 3) ? (UnkStageDat*) 0 : &OSD[AI[0]]"],
             rs_args=["if a.i[0] == 3 { core::ptr::null_mut() } else { addr_of_mut!(OSD[a.i[0] as usize]) }"]),
        "grDatFiles_801C6288",
        "grDatFiles_801C62B4",
        "grDatFiles_GetArchive",
        dict(name="grDatFiles_801C6330", c_args=["(s32) AI[0] - 1"], rs_args=["a.i[0] as i32 - 1"]),
        dict(name="grDatFiles_801C6478", c_args=["(void*) &OGO[2]", "(s32) AI[0]"], rs_args=["addr_of_mut!(OGO[2]) as *mut _", "a.i[0] as i32"]),
        dict(name="grDatFiles_StageSlotInstall", c_args=["(HSD_Archive*) &OGO[3]"], rs_args=["addr_of_mut!(OGO[3]) as *mut _"]),
    ],
)
