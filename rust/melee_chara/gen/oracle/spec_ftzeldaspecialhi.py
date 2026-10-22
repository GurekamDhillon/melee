CB = "melee_chara::chars::cb::"
HI = ["ftZd_SpecialHiStart_0_Anim", "ftZd_SpecialAirHiStart_0_Anim", "ftZd_SpecialHiStart_0_IASA",
      "ftZd_SpecialAirHiStart_0_IASA", "ftZd_SpecialHiStart_0_Phys", "ftZd_SpecialAirHiStart_0_Phys",
      "ftZd_SpecialHiStart_0_Coll", "ftZd_SpecialAirHiStart_0_Coll", "ftZd_SpecialHi_80139B44",
      "ftZd_SpecialHi_80139BB0", "ftZd_SpecialHiStart_1_Anim", "ftZd_SpecialAirHiStart_1_Anim",
      "ftZd_SpecialHiStart_1_IASA", "ftZd_SpecialAirHiStart_1_IASA", "ftZd_SpecialHiStart_1_Phys",
      "ftZd_SpecialAirHiStart_1_Phys", "ftZd_SpecialHiStart_1_Coll", "ftZelda_80139D60_Helper",
      "ftZd_SpecialAirHiStart_1_Coll", "ftZd_SpecialHi_80139F6C", "ftZd_SpecialHi_80139FE8",
      "ftZd_SpecialHi_8013A058", "ftZd_SpecialHi_8013A244", "ftZd_SpecialHi_Anim", "ftZd_SpecialAirHi_Anim",
      "ftZd_SpecialHi_IASA", "ftZd_SpecialAirHi_IASA", "ftZd_SpecialHi_Phys", "ftZd_SpecialAirHi_Phys",
      "ftZd_SpecialHi_Coll", "ftZd_SpecialAirHi_Coll", "ftZd_SpecialHi_8013A648", "ftZd_SpecialHi_8013A6A8",
      "ftZd_SpecialHi_8013A764"]

# the stick-magnitude threshold x50 lives in [0, 1] in the game: keep it there so both sides of
# `var_f31 < x50` are reached often
PRE_C = "{ ftZelda_DatAttrs* a = (ftZelda_DatAttrs*) OATTR; a->x50 = (float) (rnd() % 1001) / 1000.0f; }"
PRE_RS = "let a = &mut *(w.attr.as_mut_ptr() as *mut ftZelda_DatAttrs); a.x50 = (rnd() % 1001) as f32 / 1000.0;"
HI = [dict(name="ftZd_SpecialHi_8013A058", n=1500, pre_c=PRE_C, pre_rs=PRE_RS) if f == "@A058" else
      dict(name="ftZd_SpecialHi_8013A244", n=1500, pre_c=PRE_C, pre_rs=PRE_RS) if f == "@A244" else f for f in HI]

SPEC = dict(
    src="src/melee/ft/kinds/ftZelda/ftzeldaspecialhi.c",
    rust_mod="ftzeldaspecialhi",
    ext_mods=["ftinl"],
    legacy_stubs=True,
    attr_types=["ftZelda_DatAttrs"],
    prefer=["mv.zd.specialhi"],
    outs={"lb_8000B1CC": [2]},
    rets={"ftParts_GetBoneIndex": ("(int) (rnd() % 40)", "(rnd() % 40)")},
    bitfields=[("x2219_b0", "x2219_b0"), ("x2223_b4", "x2223_b4"), ("invisible", "invisible")],
    default_bf=["x2219_b0", "x2223_b4", "invisible"],
    fnvals={
        "ftZd_SpecialHi_801396AC": CB + "ftzeldaspecialhi__ftZd_SpecialHi_801396AC",
        "ftZd_SpecialHi_8013979C": "t::cb_ftZd_SpecialHi_8013979C",
        "efLib_PauseAll": CB + "ext__efLib_PauseAll",
        "efLib_ResumeAll": CB + "ext__efLib_ResumeAll",
    },
    funcs=[
        "ftZd_SpecialHi_801396AC",
        "ftZd_SpecialHi_801396E0",
        "ftZd_SpecialHi_8013979C",
        "ftZelda_SpecialHi_StartAction_Helper",
        "ftZd_SpecialHi_Enter",
        "ftZd_SpecialAirHi_Enter",
        *HI,
    ],
)
