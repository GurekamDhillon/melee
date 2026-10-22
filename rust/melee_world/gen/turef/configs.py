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

_RANDI = ("expr", "a0 > 0 ? (int) (rnd() % (u32) a0) : 0", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }")
_JOBJ_OR_NULL = ("expr", "(rnd() % 5 == 0) ? 0 : mkjobj()", "if rnd() % 5 == 0 { core::ptr::null_mut() } else { mkjobj() }")
_TRVEC_A1 = (r"printf(%sv%s); TRF(a1->x); TRF(a1->y); TRF(a1->z); printf(%s\n%s);" % (('"',) * 4),
             r"out(%sv%s); trf((*a1).x); trf((*a1).y); trf((*a1).z); out(%s\n%s);" % (('"',) * 4))
CONFIGS["grstory"]["ret"].update({"Ground_GetStageGObj": _GOBJ, "HSD_Randi": _RANDI, "it_8026B3C0": ("ri", 2),
                                   "Ground_801C3FA4": _JOBJ_OR_NULL,
                                   "Ground_GetYakumonoParam": ("expr", "mkyp()", "mkyp()")})
CONFIGS["grstory"]["effect"] = {"it_802D8618": _TRVEC_A1, "lb_8000B1CC": _OUT_A1A2}

# ---- grpushon -----------------------------------------------------------------------------------
_VEC_RAND_A1 = ("if (a1) { a1->x = rf(); a1->y = rf(); a1->z = rf(); }",
                "if !a1.is_null() { (*a1).x = rf(); (*a1).y = rf(); (*a1).z = rf(); }")
_VEC_RAND_A0 = ("if (a0) { a0->x = rf(); a0->y = rf(); a0->z = rf(); }",
                "if !a0.is_null() { (*a0).x = rf(); (*a0).y = rf(); (*a0).z = rf(); }")
_VEC_SUB = ("a0->x = a0->x - a1->x; a0->y = a0->y - a1->y; a0->z = a0->z - a1->z;",
            "(*a0).x = (*a0).x - (*a1).x; (*a0).y = (*a0).y - (*a1).y; (*a0).z = (*a0).z - (*a1).z;")
_VEC_ADD = ("a0->x = a0->x + a1->x; a0->y = a0->y + a1->y; a0->z = a0->z + a1->z;",
            "(*a0).x = (*a0).x + (*a1).x; (*a0).y = (*a0).y + (*a1).y; (*a0).z = (*a0).z + (*a1).z;")
_VEC_NORM = ("a0->x = a0->x * 0.5f; a0->y = a0->y * 0.25f; a0->z = a0->z * 2.0f;",
             "(*a0).x = (*a0).x * 0.5; (*a0).y = (*a0).y * 0.25; (*a0).z = (*a0).z * 2.0;")
_COLOR_RAND = ("a1->r = (u8) rnd(); a1->g = (u8) rnd(); a1->b = (u8) rnd(); a1->a = (u8) rnd();",
               "(*a1).r = rnd() as u8; (*a1).g = rnd() as u8; (*a1).b = rnd() as u8; (*a1).a = rnd() as u8;")
_FTLIB_POS = ("if (a1) { if (FMODE == 1) { a1->x = 1500.0f + 10.0f * (float) (rnd() % 5); a1->y = 330.0f + 4.0f * (float) (rnd() % 5); a1->z = rf(); } else { a1->x = rf(); a1->y = rf(); a1->z = rf(); } }",
              "if !a1.is_null() { if FMODE == 1 { (*a1).x = 1500.0 + 10.0 * ((rnd() % 5) as f32); (*a1).y = 330.0 + 4.0 * ((rnd() % 5) as f32); (*a1).z = rf(); } else { (*a1).x = rf(); (*a1).y = rf(); (*a1).z = rf(); } }")
CONFIGS["grpushon"]["patch"] = [
    ("((HSD_GObj*) HSD_GObjGXLinkHead)->next_gx", "HSD_GObjGXLinkHead[4]"),
    # two dead decomp-matching expression statements that load through a NULL gobj; the optimised
    # shipping build drops them, the -O0 oracle link would execute them
    ("    GET_GROUND(0);\n    GET_GROUND(0);\n", ""),
    ("grZakoGenerator_801CAEB0(Ground_801C5840(), Ground_801C5940());",
     "{ int t840_ = Ground_801C5840(); int t940_ = Ground_801C5940(); grZakoGenerator_801CAEB0(t840_, t940_); }"),
]
CONFIGS["grpushon"]["ret"].update({
    "Ground_GetStageGObj": ("expr", "(rnd() % 4 == 0) ? 0 : mkstagegobj(a0)", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { mkstagegobj(a0) }"),
    "HSD_Randi": _RANDI,
    "Ground_801C0498": ("expr", "SCALE != 0.0f ? SCALE : (float) (rnd() % 100 + 1) / 8.0f",
                        "if SCALE != 0.0 { SCALE } else { ((rnd() % 100 + 1) as f32) / 8.0 }"),
    "Ground_801C3D44": ("ri", 2), "Ground_801C5840": ("ri", 7), "Ground_801C5940": ("ri", 7),
    "Ground_GetMapGObj": ("expr", "(rnd() % 3 == 0) ? 0 : mkmapgobj()", "if rnd() % 3 == 0 { core::ptr::null_mut() } else { mkmapgobj() }"),
    "Ground_GetP1Fighter": ("expr", "(rnd() % 4 == 0) ? 0 : mkfgobj()", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { mkfgobj() }"),
    "Ground_GetYakumonoParam": ("expr", "mkypp()", "mkypp()"),
    "HSD_LObjGetFlags": ("expr", "rnd()", "rnd()"),
    "HSD_LObjGetPosition": ("expr", "(rnd() % 8 != 0)", "(rnd() % 8 != 0) as i32"),
    "HSD_LObjGetInterest": ("expr", "(rnd() % 8 != 0)", "(rnd() % 8 != 0) as i32"),
    "HSD_LObjLoadDesc": ("expr", "(LDNULL && rnd() % 3 == 0) ? 0 : mklobj()",
                         "if LDNULL && rnd() % 3 == 0 { core::ptr::null_mut() } else { mklobj() }"),
    "lbVector_Sub": ("expr", "a0", "a0"), "lbVector_Add": ("expr", "a0", "a0"),
    "mpJointFromLine": ("expr", "(int) (rnd() % 4) - 1", "(rnd() % 4) as i32 - 1"),
    "mpLineGetKind": ("expr", "1 << (rnd() % 5)", "1 << (rnd() % 5)"),
    "gm_MexVanillaKind": ("expr", "(rnd() % 3 == 0) ? (int) (rnd() % 8) : a0",
                          "if rnd() % 3 == 0 { (rnd() % 8) as i32 } else { a0 }"),
})
CONFIGS["grpushon"]["effect"] = {
    "Stage_UnkSetVec3TCam_Offset": _VEC_RAND_A0, "ftLib_80086644": _FTLIB_POS,
    "HSD_LObjGetPosition": _VEC_RAND_A1, "HSD_LObjGetInterest": _VEC_RAND_A1, "HSD_LObjGetColor": _COLOR_RAND,
    "lbVector_Sub": _VEC_SUB, "lbVector_Add": _VEC_ADD, "lbVector_Normalize": _VEC_NORM,
}

# ---- grrcruise ----------------------------------------------------------------------------------
_VEC_DIFF = ("a2->x = a0->x - a1->x; a2->y = a0->y - a1->y; a2->z = a0->z - a1->z;",
             "(*a2).x = (*a0).x - (*a1).x; (*a2).y = (*a0).y - (*a1).y; (*a2).z = (*a0).z - (*a1).z;")
# nonzero positions: the cruise camera maths divides by their differences (0/0 NaN signs are the
# hardware's business, not the port's)
_OUT_A1A2_NZ = ("if (a1) { a1->x = rnz(); a1->y = rnz(); a1->z = rnz(); } if (a2) { a2->x = rnz(); a2->y = rnz(); a2->z = rnz(); }",
                "if !a1.is_null() { (*a1).x = rnz(); (*a1).y = rnz(); (*a1).z = rnz(); } if !a2.is_null() { (*a2).x = rnz(); (*a2).y = rnz(); (*a2).z = rnz(); }")
CONFIGS["grrcruise"]["patch"] = [
    # the native Map_VanishEntry / grRCruise_Entry are wider than the 8-byte / 0x198-byte GC sizes
    # the C allocates by: allocate by the native element sizes
    ("gp->u.map.vanish = HSD_MemAlloc(sizeof(lbl_803E5014));",
     "gp->u.map.vanish = HSD_MemAlloc(ARRAY_SIZE(lbl_803E5014) * sizeof(struct Map_VanishEntry));"),
    ("gp->u.map.chikuwa = HSD_MemAlloc(sizeof(*gp->u.map.chikuwa));",
     "gp->u.map.chikuwa = HSD_MemAlloc(17 * sizeof(struct grRCruise_Entry));\n"
     "    gp->u.rcruise.entries = (struct grRCruise_Entry*) gp->u.map.chikuwa;"),
    # the console overlays that native layouts break (see tu/grrcruise.rs): the same words
    ("gp2->u.rcruise2.xEC = jgobj;", "gp2->u.rcruise2.xEC = jgobj;\n    gp2->u.scroll.anim_gobj = jgobj;"),
]
CONFIGS["grrcruise"]["ret"].update({
    "Ground_GetStageGObj": ("expr", "a0 == 3 ? (G3 ? G3 : (G3 = mkgobj())) : ((GNULL && rnd() % 4 == 0) ? 0 : mkgobj())",
                            "if a0 == 3 { if G3.is_null() { G3 = mkgobj(); } G3 } else if GNULL && rnd() % 4 == 0 { core::ptr::null_mut() } else { mkgobj() }"),
    "Ground_GetMapGObj": ("expr", "(rnd() % 3 == 0) ? 0 : mkgobj()", "if rnd() % 3 == 0 { core::ptr::null_mut() } else { mkgobj() }"),
    "Ground_GetYakumonoParam": ("expr", "mkyp()", "mkyp()"),
    "Ground_801C3FA4": ("expr", "(JNULL && rnd() % 5 == 0) ? 0 : mkjtree(0)",
                        "if JNULL && rnd() % 5 == 0 { core::ptr::null_mut() } else { mkjtree(0) }"),
    "Ground_801C32D4": ("ri", 50),
    "HSD_MemAlloc": ("expr", "AZ(a0)", "az(a0 as usize)"),
    "HSD_Randi": _RANDI,
    "HSD_Randf": ("expr", "(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
    "HSD_JObjGetDObj": ("expr", "mkdobjs()", "mkdobjs()"),
    "HSD_ArchiveGetPublicAddress": ("expr", "(rnd() % 3 == 0) ? 0 : AZ(16)", "if rnd() % 3 == 0 { core::ptr::null_mut() } else { az(16) as *mut _ }"),
    "grDatFiles_GetArchive": ("expr", "(rnd() % 3 == 0) ? 0 : AZ(64)", "if rnd() % 3 == 0 { core::ptr::null_mut() } else { az(64) as *mut _ }"),
    "grAnime_801C83D0": ("ri", 2), "Camera_8003118C": ("ri", 2),
    "mpJointFromLine": ("expr", "(rnd() % 3 == 0) ? (int) (0x1B + rnd() % 20) : (int) (rnd() % 60)",
                        "if rnd() % 3 == 0 { (0x1B + rnd() % 20) as i32 } else { (rnd() % 60) as i32 }"),
    "lbVector_Sub": ("expr", "a0", "a0"), "lbVector_Add": ("expr", "a0", "a0"),
})
CONFIGS["grrcruise"]["effect"] = {
    "lb_8000B1CC": _OUT_A1A2_NZ, "Stage_UnkSetVec3TCam_Offset": _VEC_RAND_A0,
    "lbVector_Sub": _VEC_SUB, "lbVector_Add": _VEC_ADD, "lbVector_Diff": _VEC_DIFF,
}

# ---- grshrineroute ------------------------------------------------------------------------------
CONFIGS["grshrineroute"]["patch"] = [
    # two dead decomp-matching expression statements that load through a NULL gobj (see grpushon)
    ("    GET_GROUND(0);\n    GET_GROUND(0);\n", ""),
    ("grZakoGenerator_801CAEB0(Ground_801C5840(), Ground_801C5940());",
     "{ int t840_ = Ground_801C5840(); int t940_ = Ground_801C5940(); grZakoGenerator_801CAEB0(t840_, t940_); }"),
]
CONFIGS["grshrineroute"]["custom"] = CUSTOM + ["efSync_Spawn"]
CONFIGS["grshrineroute"]["ret"].update({
    "Ground_GetYakumonoParam": ("expr", "mkyp()", "mkyp()"),
    "Ground_GetStageGObj": ("expr", "(GNULL && rnd() % 4 == 0) ? 0 : mkgobj()",
                            "if GNULL && rnd() % 4 == 0 { core::ptr::null_mut() } else { mkgobj() }"),
    "Ground_GetMapGObj": ("expr", "(rnd() % 3 == 0) ? 0 : mkgobj()", "if rnd() % 3 == 0 { core::ptr::null_mut() } else { mkgobj() }"),
    "Ground_GetP1Fighter": ("expr", "(rnd() % 4 == 0) ? 0 : PGOBJ", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { PGOBJ }"),
    "Ground_801C2CF4": ("expr", "(rnd() % 6 == 0) ? 0 : mkjobj2()", "if rnd() % 6 == 0 { core::ptr::null_mut() } else { mkjobj2() }"),
    "Ground_801C3FA4": ("expr", "(JNULL && rnd() % 5 == 0) ? 0 : mkjobj2()",
                        "if JNULL && rnd() % 5 == 0 { core::ptr::null_mut() } else { mkjobj2() }"),
    "Ground_801C2D24": ("ri", 3),
    "Ground_801C3DB4": ("expr", "(rnd() % 4 == 0) ? -1 : (int) (0xBD + rnd() % 6)",
                        "if rnd() % 4 == 0 { -1 } else { (0xBD + rnd() % 6) as i32 }"),
    "Ground_801C5764": ("ri", 2), "Ground_801C5840": ("ri", 7), "Ground_801C5940": ("ri", 7),
    "Ground_801C0498": ("expr", "SCALE != 0.0f ? SCALE : (float) (rnd() % 100 + 1) / 8.0f",
                        "if SCALE != 0.0 { SCALE } else { ((rnd() % 100 + 1) as f32) / 8.0 }"),
    "HSD_Randi": _RANDI,
    "HSD_Randf": ("expr", "(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
    "HSD_JObjGetFlags": ("expr", "rnd()", "rnd()"),
    "HSD_LObjGetFlags": ("expr", "rnd()", "rnd()"),
    "HSD_LObjGetPosition": ("expr", "(rnd() % 8 != 0)", "(rnd() % 8 != 0) as i32"),
    "HSD_LObjGetInterest": ("expr", "(rnd() % 8 != 0)", "(rnd() % 8 != 0) as i32"),
    "HSD_LObjLoadDesc": ("expr", "(LDNULL && rnd() % 3 == 0) ? 0 : mklobj()",
                         "if LDNULL && rnd() % 3 == 0 { core::ptr::null_mut() } else { mklobj() }"),
    "Player_GetEntity": ("expr", "(rnd() % 3 == 0) ? mkfgobj() : PGOBJ", "if rnd() % 3 == 0 { mkfgobj() } else { PGOBJ }"),
    "Player_GetEntityAtIndex": ("expr", "(rnd() % 2) ? 0 : mkfgobj()", "if rnd() % 2 != 0 { core::ptr::null_mut() } else { mkfgobj() }"),
    "ftLib_80086BE0": ("ri", 4), "ftLib_8008731C": ("ri", 2), "ftLib_80086EC0": ("ri", 2), "ftLib_8008732C": ("ri", 2),
    "lb_8000B134": ("ri", 2), "grLib_801C96E8": ("ri", 2),
    "lbVector_Sub": ("expr", "a0", "a0"), "lbVector_Add": ("expr", "a0", "a0"),
    "Mex_GrBind": ("expr", "a0", "a0"),
})
_FT_POS = ("if (a1) { if (FPOS) { a1->x = -250.0f + (float) (rnd() % 100); a1->y = -300.0f + (float) (rnd() % 100); a1->z = rf(); } else { a1->x = rf(); a1->y = rf(); a1->z = rf(); } }",
           "if !a1.is_null() { if FPOS { (*a1).x = -250.0 + ((rnd() % 100) as f32); (*a1).y = -300.0 + ((rnd() % 100) as f32); (*a1).z = rf(); } else { (*a1).x = rf(); (*a1).y = rf(); (*a1).z = rf(); } }")
CONFIGS["grshrineroute"]["effect"] = {
    "Ground_801C2D24": _VEC_RAND_A1, "ftLib_80086644": _FT_POS, "lb_8000B1CC": _OUT_A1A2,
    "Stage_UnkSetVec3TCam_Offset": _VEC_RAND_A0,
    "HSD_LObjGetPosition": _VEC_RAND_A1, "HSD_LObjGetInterest": _VEC_RAND_A1, "HSD_LObjGetColor": _COLOR_RAND,
    "lbVector_Sub": _VEC_SUB, "lbVector_Add": _VEC_ADD, "lbVector_Normalize": _VEC_NORM,
}
