SPEC = dict(
    src="src/melee/ft/kinds/ftYoshi/ftyoshispeciallw.c",
    rust_mod="ftyoshispeciallw",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftYoshiAttributes"],
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    outs={"lb_8000B1CC": [2]},
    bitfields=[("throw_flags_b0", "anon_2210.anon_0.throw_flags_b0"), ("x2223_b4", "x2223_b4")],
    fnvals={
        "fn_8012E644": "melee_chara::chars::cb::ftyoshispeciallw__fn_8012E644",
    },
    funcs=[
        dict(name="fn_8012E644", attrs=["ftYs_DatAttrs"]),
        dict(name="ftYs_SpecialLw_Enter", bf=["throw_flags_b0", "x2223_b4"]),
        dict(name="ftYs_SpecialAirLw_Enter", bf=["throw_flags_b0", "x2223_b4"]),
        "ftYs_SpecialLw_Anim",
        "ftYs_SpecialAirLw_Anim",
        "ftYs_SpecialLw_Phys",
        "ftYs_SpecialAirLw_Phys",
        "ftYs_SpecialLw_Coll",
        "ftYs_SpecialAirLw_Coll",
        "ftYs_SpecialS_8012EA04",
        "ftYs_SpecialLwLanding_Anim",
        "ftYs_SpecialLwLanding_Phys",
        "ftYs_SpecialLwLanding_Coll",
        "ftYs_SpecialS_8012EAD8",
    ],
)
