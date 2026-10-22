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
