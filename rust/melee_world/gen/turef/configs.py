"""Per-TU stub configuration (see stubgen.py for the spec language)."""
SRC = "src/melee/gr/%s.c"
CUSTOM = ["OSReport", "grAnime_801C752C", "efSync_Spawn"]

CONFIGS = {}
for _s in ("grpushon", "grrcruise", "grshrine", "grshrineroute", "grstory", "grtcaptain", "grtclink", "grtdonkey"):
    CONFIGS[_s] = dict(src=SRC % _s, custom=CUSTOM,
                       ret={"Mex_GrBind": ("expr", "a0", "a0")})

_GOBJ = ("expr", "(rnd() % 4 == 0) ? 0 : mkgobj()", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { mkgobj() }")
for _s in ("grtcaptain", "grtclink", "grtdonkey"):
    CONFIGS[_s]["ret"].update({"Ground_GetStageGObj": _GOBJ, "Ground_801C4210": ("ri", 5)})

_TRVEC_A0 = (r'printf("v"); TRF(a0->x); TRF(a0->y); TRF(a0->z); printf("\n");',
             r'out("v"); trf((*a0).x); trf((*a0).y); trf((*a0).z); out("\n");')
_OUT_A1A2 = ("if (a1) { a1->x = rf(); a1->y = rf(); a1->z = rf(); } if (a2) { a2->x = rf(); a2->y = rf(); a2->z = rf(); }",
             "if !a1.is_null() { (*a1).x = rf(); (*a1).y = rf(); (*a1).z = rf(); } if !a2.is_null() { (*a2).x = rf(); (*a2).y = rf(); (*a2).z = rf(); }")
CONFIGS["grshrine"]["ret"].update({"Ground_GetStageGObj": _GOBJ})
CONFIGS["grshrine"]["effect"] = {"lb_80011A50": _TRVEC_A0, "lb_8000B1CC": _OUT_A1A2}
