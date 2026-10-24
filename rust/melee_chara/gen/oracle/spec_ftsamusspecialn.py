CB = "melee_chara::chars::cb::"
SPEC = dict(
    src="src/melee/ft/kinds/ftSamus/ftsamusspecialn.c",
    rust_mod="ftsamusspecialn",
    ext_mods=["ftinl", "ftcaptainspecialn"],
    njo=64,  # FtPart_56 is indexed
    legacy_stubs=True,
    attr_types=["ftSs_DatAttrs"],
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    outs={"lb_8000B1CC": [2]},
    bitfields=[("throw_flags_b0", "anon_2210.anon_0.throw_flags_b0"), ("x2219_b0", "x2219_b0")],
    motions=[349, 351, 350, 352],
    fnvals={
        "efLib_PauseAll": CB + "ext__efLib_PauseAll",
        "efLib_ResumeAll": CB + "ext__efLib_ResumeAll",
    },
    funcs=[
        "ftSs_SpecialN_80129258",
        dict(name="ftSs_SpecialS_8012A074", bf=["throw_flags_b0", "x2219_b0"]),
    ],
)
