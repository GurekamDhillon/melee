"""Per-TU stub configuration (see stubgen.py for the spec language)."""
OBJ = ("effect",)
SETOBJ = ("a0->hsd_obj = a2;", "(*a0).hsd_obj = a2;")

CONFIGS = {
    "gm_180A": dict(
        src="src/melee/gm/gm_180A.c",
        ret={"gm_CKindToSelKind": ("ri", 27),
             "gmMainLib_8015D06C": ("allocr", 4),
             "HSD_JObjLoadJoint": ("expr", "mkchain()", "mkchain()")},
        effect={"HSD_GObjObject_80390A70": SETOBJ},
        custom=["lbArchive_80016DBC"],
        ginit={"HSD_GObj_JObjKind": "0x2A"},
    ),
}

for _s in ("gm_181A", "gm_1832", "gm_186E", "gm_1879", "gm_1884", "gm_18A1", "gm_19EF"):
    CONFIGS.setdefault(_s, dict(src=f"src/melee/gm/{_s}.c"))

CONFIGS["gm_18A1"] = dict(
    src="src/melee/gm/gm_18A1.c",
    ret={"Player_GetDamage": ("ri", 4),
         "fn_8017E318": ("expr", "(int) (rnd() % 3) - 1", "(rnd() % 3) as i32 - 1"),
         "gmMainLib_8015CDE0": ("allocr", 8),
         "gm_GetFrameCount": ("expr", "rnd() % 0xA000", "rnd() % 0xA000"),
         "gm_GetStartMeleeRules": ("expr", "&g_rules", "core::ptr::addr_of_mut!(G_RULES)")},
)

CONFIGS["gm_186E"] = dict(
    src="src/melee/gm/gm_186E.c",
    ret={"HSD_CObjSetCurrent": ("ri", 2), "lb_80011AC4": ("allocn", 64),
         "Player_GetEntityAtIndex": ("expr", "(rnd() % 4 == 0) ? 0 : mkgobj()", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { mkgobj() }"),
         "HSD_JObjLoadJoint": ("expr", "mkjobj()", "mkjobj()")},
    effect={"HSD_GObjObject_80390A70": SETOBJ},
    custom=["lbArchive_80016DBC", "lb_80011E24"],
    ginit={"HSD_GObj_JObjKind": "0x2A", "HSD_GObj_CameraKind": "0x2B", "HSD_GObj_LightKind": "0x2C", "HSD_GObj_FogKind": "0x2D"},
)

_VEC_GET = ("a1->x = rf(); a1->y = rf(); a1->z = rf();", "(*a1).x = rf(); (*a1).y = rf(); (*a1).z = rf();")
_VEC_SET = ('TRF(a1->x); TRF(a1->y); TRF(a1->z); printf("\\n");', 'trf((*a1).x); trf((*a1).y); trf((*a1).z); out("\\n");')
CONFIGS["gm_1879"] = dict(
    src="src/melee/gm/gm_1879.c",
    ret={"gm_GetButtonsTriggered": ("expr", "btn()", "btn()"),
         "lb_8000B09C": ("ri", 2),
         "lb_80011AC4": ("allocn", 64),
         "lbAudioAx_80026E84": ("expr", "(ou64) (rnd() % 7) << 40", "((rnd() % 7) as u64) << 40"),
         "gm_GetCurrentGameMode": ("expr", "(rnd() % 2) ? 28 : 5", "if rnd() % 2 != 0 { 28 } else { 5 }")},
    effect={"HSD_GObjObject_80390A70": SETOBJ,
            "HSD_CObjGetInterest": _VEC_GET, "HSD_CObjGetEyePosition": _VEC_GET,
            "HSD_CObjSetInterest": _VEC_SET, "HSD_CObjSetEyePosition": _VEC_SET},
    custom=["lbArchive_80016DBC"],
    ginit={"HSD_GObj_JObjKind": "0x2A", "HSD_GObj_CameraKind": "0x2B", "HSD_GObj_LightKind": "0x2C", "HSD_GObj_FogKind": "0x2D"},
)

_MEMZERO = ("memset(a0, 0, (size_t) a1);", "core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize);")
_ITEM_POS = ('TRF(a0->x); TRF(a0->y); TRF(a0->z); printf("\\n");', 'trf((*a0).x); trf((*a0).y); trf((*a0).z); out("\\n");')
CONFIGS["gm_1884"] = dict(
    src="src/melee/gm/gm_1884.c",
    ret={"pl_8004134C": ("ri", 1200), "pl_80041300": ("ri", 1200), "pl_800412D0": ("ri", 1200),
         "Player_GetPlayerCharacter": ("ri", 25),
         "gm_GetDbPauseFlag": ("ri", 2), "lbLang_IsSettingUS": ("ri", 2), "lbLang_IsSavedLanguageUS": ("ri", 2),
         "gm_801A36C0": ("expr", "btn()", "btn()"),
         "Player_GetEntity": ("expr", "mkgobj()", "mkgobj()")},
    effect={"HSD_GObjObject_80390A70": SETOBJ, "Stage_80224E64": _VEC_GET, "Player_800328D4": _VEC_SET,
            "it_8026D258": _ITEM_POS, "memzero": _MEMZERO},
    custom=["lbArchive_80016DBC", "lb_80011E24"],
    ginit={"HSD_GObj_JObjKind": "0x2A", "HSD_GObj_CameraKind": "0x2B", "HSD_GObj_LightKind": "0x2C", "HSD_GObj_FogKind": "0x2D"},
)


CONFIGS["gm_19EF"] = dict(
    src="src/melee/gm/gm_19EF.c",
    ret={"gm_GetCurrentGameMode": ("expr", "(rnd() % 3) + 3", "(rnd() % 3) as u8 + 3"),
         "gm_80160474": ("expr", "(int) (rnd() % 8) - 1", "(rnd() % 8) as i32 - 1"),
         "gm_801604DC": ("s", "Scn"), "gm_80160564": ("s", "Mdl"),
         "Toy_803060BC": ("expr", "(float) (rnd() % 100 + 1) / 8.0f", "((rnd() % 100 + 1) as f32) / 8.0"),
         "HSD_JObjLoadJoint": ("expr", "mktree()", "mktree()"),
         "lb_80011AC4": ("allocn", 64),
         "gm_GetButtonsTriggered": ("expr", "btn()", "btn()")},
    effect={"HSD_GObjObject_80390A70": SETOBJ},
    custom=["lbArchive_80016DBC", "lbArchive_LoadSymbols", "HSD_SisLib_803A6B98", "lb_80011E24"],
    ginit={"HSD_GObj_JObjKind": "0x2A", "HSD_GObj_CameraKind": "0x2B", "HSD_GObj_LightKind": "0x2C", "HSD_GObj_FogKind": "0x2D"},
    patch=[("((HSD_JObj**) ((u8*) &lbl_80479A98 + 0x28))[i]", "lbl_80479A98.x28.typed.jobjs[i - 1]"),
           ("lbl_80479A98.jobj_slots[i] = node;", "lbl_80479A98.x28.typed.jobjs[i - 10] = node;"),
           ("ptr = &((HSD_JObj**) &lbl_80479A98)[j = 10];\n        ptr += 10;", "ptr = &lbl_80479A98.x28.typed.jobjs[(j = 10) - 1];")],
)

CONFIGS["gm_181A"] = dict(
    src="src/melee/gm/gm_181A.c",
    ret={"Player_GetFalls": ("ri", 2), "Player_GetPlayerSlotType": ("ri", 4), "Player_GetTeam": ("ri", 3),
         "gm_GetCurrentGameMode": ("expr", "0x21 + rnd() % 7", "0x21 + (rnd() % 7) as u8"),
         "gm_8016AEEC": ("ri", 2), "gm_8016AEFC": ("expr", "(rnd() % 2) ? 0x3B : 0", "if rnd() % 2 != 0 { 0x3B } else { 0 }"),
         "gm_GetFrameCount": ("ri", 5000), "gm_801A4BA8": ("expr", "gv()", "gv()"),
         "gm_801BF6D8": ("ri", 4), "gm_801BF6F8": ("ri", 4), "Player_800368F8": ("ri", 2), "HSD_Randi": ("ri", 100),
         "gmMainLib_8015D6BC": ("ri", 2), "gmMainLib_8015D710": ("ri", 2), "gmMainLib_8015DB00": ("ri", 4),
         "gm_CKindToSelKind": ("ri", 27), "gm_MexVanillaKind": ("ri", 27), "Mex_Enabled": ("ri", 2),
         "gm_GetCurrentSceneIndex": ("expr", "2 + rnd() % 3", "2 + (rnd() % 3) as i32"),
         "Player_GetPlayerId": ("ri", 4), "Player_GetNametagSlotID": ("ri", 100),
         "gm_GetButtonsTriggered": ("expr", "bt()", "bt()")},
    effect={"gm_SetupPlayerDefaults": ("pid_randomize(a0);", "pid_randomize(a0);"),
            "gm_8016EDDC": ("pid_print(a1);", "pid_print(a1);")},
    custom=["lbArchive_80016DBC"],
    ginit={},
    patch=[("regclear_record_state* s = (regclear_record_state*) &lbl_80472ED8;",
            "regclear_record_state* s = (regclear_record_state*) ((u8*) &lbl_80473594 - 0x6BC);")],
)
