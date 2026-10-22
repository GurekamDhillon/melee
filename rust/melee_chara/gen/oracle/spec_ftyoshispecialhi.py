TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
        ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None)]

SPEC = dict(
    src="src/melee/ft/kinds/ftYoshi/ftyoshispecialhi.c",
    rust_mod="ftyoshispecialhi",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftYs_DatAttrs"],
    prefer=["mv.ys.specialhi", "u.ys"],
    provides=["__assert"],
    data_c="""
void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; for (;;) {} }
""",
    outs={"lb_8000B1CC": [2]},
    bitfields=[("throw_flags_b0", "anon_2210.anon_0.throw_flags_b0")],
    motions=[348, 350, 353, 355],
    opt_links=[("target_item_gobj", "OIG[0]"), ("u.ys.x2238", "OIG[1]")],
    opt_targets_rs={"OIG[0]": "&mut w.oig[0] as *mut HSD_GObj", "OIG[1]": "&mut w.oig[1] as *mut HSD_GObj"},
    fnvals={
        "fn_8012E110": "t::cb_fn_8012E110",
        "ftYs_Init_8012BA8C": "melee_chara::chars::cb::ftyoshi__ftYs_Init_8012BA8C",
        "fn_8012E3B4": "t::cb_fn_8012E3B4",
        "fn_8012E44C": "t::cb_fn_8012E44C",
    },
    funcs=[
        "ftYs_SpecialS_8012DF18",
        "ftYs_SpecialS_8012DF8C",
        dict(name="fn_8012E110", bf=["throw_flags_b0"]),
        "ftYs_SpecialS_8012E270",
        "ftYs_SpecialHi_Coll",
        "ftYs_SpecialAirHi_Coll",
        "setDamageCallbacks",
        "setAccessory4Callback",
        "ftYs_SpecialHi_Enter",
        "ftYs_SpecialAirHi_Enter",
        "fn_8012E3B4",
        "fn_8012E44C",
        "ftYs_SpecialHi_Anim",
        "ftYs_SpecialAirHi_Anim",
        "ftYs_SpecialHi_Phys",
        "ftYs_SpecialAirHi_Phys",
    ],
)
