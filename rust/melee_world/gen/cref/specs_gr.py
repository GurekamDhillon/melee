"""Oracle specs of the gr/ TU ports (shard 577-584); see gen_tu.py for the keys and gen_drv.py for
the driver keys."""
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "Mex_GrBind": ("a0", "a0"),
}
SPECS = {}
SPECS["groldyoshi"] = dict(
    src="src/melee/gr/groldyoshi.c",
    types=["StageInfo", "StageCallbacks", "StageData"],
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"],
    ret={
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Ground_GetStageGObj": ("((fv() & 3) == 0 ? 0 : &GO[fv() & 3])",
                                "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() & 3) as usize] }"),
        "Ground_801C3FA4": ("((NJ && (fv() & 7) == 0) ? 0 : &JP[fv() % 16])",
                            "if super::NJ && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::JP[(fv() % 16) as usize] }"),
    },
    decl_c="static __typeof__(*yakumono_param) YP;" + chr(10) + "static int NJ;",
    decl_r="static mut YP: m::OyYakumonoParam = unsafe { core::mem::zeroed() };" + chr(10) + "static mut NJ: bool = false;",
    setup_c="""NJ = 0; zero(&YP, sizeof YP); zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);
YP.x0 = (s16) (fv() % 40); YP.x2 = (s16) (fv() % 40); YP.x4 = rg(); YP.x8 = rg(); YP.xC = rg();
YP.x10 = (s16) (fv() % 40); YP.x12 = (s16) (fv() % 40); YP.x14 = (s16) (fv() % 40); YP.x16 = (s16) (fv() % 40);
YP.x18 = (s16) (fv() % 40);
yakumono_param = &YP;""",
    setup_r="""NJ = false; m::ext::stage_info = core::mem::zeroed(); g::fill_StageInfo(&mut m::ext::stage_info);
YP = core::mem::zeroed();
YP.x0 = (fv() % 40) as i16; YP.x2 = (fv() % 40) as i16; YP.x4 = rg(); YP.x8 = rg(); YP.xC = rg();
YP.x10 = (fv() % 40) as i16; YP.x12 = (fv() % 40) as i16; YP.x14 = (fv() % 40) as i16; YP.x16 = (fv() % 40) as i16;
YP.x18 = (fv() % 40) as i16;
m::yakumono_param = &raw mut YP;""",
    hash_c="hash_StageInfo(&stage_info); HM(RH(&YP, sizeof YP)); HM(PI(yakumono_param));",
    hash_r="g::hash_StageInfo(&m::ext::stage_info); hm(rh(&YP)); hm(pi(m::yakumono_param));",
    preg_c="preg(&YP, sizeof YP); preg(grOy_StageCallbacks, sizeof grOy_StageCallbacks); preg(&grOy_StageData, sizeof grOy_StageData);",
    preg_r="preg(&raw const YP, core::mem::size_of_val(&YP)); preg(&raw const m::grOy_StageCallbacks, core::mem::size_of_val(&m::grOy_StageCallbacks)); preg(&raw const m::grOy_StageData, core::mem::size_of_val(&m::grOy_StageData));",
    data=[("grOy_StageCallbacks", "StageCallbacks", 6), ("grOy_StageData", "StageData", None),
          ("grOy_803E6574", "raw", None), ("grOy_803B83F0", "raw", None)],
    args={
        "grOldYoshi_8020E854": [("(int) (fv() % 6)", "(fv() % 6) as i32")],
        "fn_8020F2A8": [("(void*) &GR[fv() & 3]", "&raw mut GR[(fv() & 3) as usize] as *mut c_void"),
                        ("(int) (fv() % 6)", "(fv() % 6) as i32"),
                        ("&CD", "&raw mut CD"),
                        ("(int) fv()", "fv() as i32"),
                        ("(mpLib_GroundEnum) (fv() % 4)", "fv() % 4"),
                        ("rg()", "rg()")],
    },
    # these run several on_init callbacks that overlay different views of Ground::u (a raw-byte alias of
    # a bitfield view differs C vs GC order): fill no view (zero union) and hash the view without bitfields
    view={"grOldYoshi_8020E79C": "-", "grOldYoshi_8020E854": "-"},
    hview={"grOldYoshi_8020E79C": "oldyoshiguest", "grOldYoshi_8020E854": "oldyoshiguest"},
    pre={"grOldYoshi_8020F088": ("NJ = 1;", "NJ = true;")},
    cases={"fn_8020F2A8": 600, "grOldYoshi_8020EC10": 1000, "grOldYoshi_8020F088": 600},
)
for _s in SPECS.values():
    _s.setdefault("common_ret", COMMON_RET)
