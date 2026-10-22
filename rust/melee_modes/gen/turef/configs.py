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
