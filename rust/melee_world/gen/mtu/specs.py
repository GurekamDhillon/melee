"""Per-TU oracle specs for gen_tu.py / run_tu.py (see gen_tu.py and RUST_PORT.md, "melee_world::tu
state oracle"). Keys as in melee_modes/gen/cref/specs.py: src, types, ret {callee: (C expr, Rust
expr)}, effects, null_ret_ok, data_size, patch, custom_stubs, fn32_args."""
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}

SPECS = {
    "grdynamicattr": dict(src="src/melee/gr/grdynamicattr.c", types=["grDynamicAttr_UnkStruct"], skip_stubs=["__frsqrte"], skip_data=["guess", "y"]),
}
SPECS["grdisplay"] = dict(
    src="src/melee/gr/grdisplay.c",
    types=["Ground", "HSD_GObj", "LbShadow", "HSD_JObj"],
    skip_data=["result"],
    ret={
        "Camera_80030A50": ("((fv() & 1) ? &CAMG : 0)", "if fv() & 1 != 0 { &raw mut super::CAMG } else { core::ptr::null_mut() }"),
        "Ground_801C1E84": ("((fv() & 1) ? &FOGG : 0)", "if fv() & 1 != 0 { &raw mut super::FOGG } else { core::ptr::null_mut() }"),
        "HSD_CObjGetCurrent": ("((fv() % 3) ? &CAMCO : &OTHCO)", "if fv() % 3 != 0 { &raw mut super::CAMCO } else { &raw mut super::OTHCO }"),
        "ftLib_800872B0": ("&SHAD[fv() % 3]", "&raw mut super::SHAD[(fv() % 3) as usize]"),
        "Camera_8003108C": ("(int) (fv() & 7)", "(fv() & 7) as i32"),
    })

_STAGE_RET = {
    "Ground_GetStageGObj": ("((fv() & 1) ? &SGO : 0)", "if fv() & 1 != 0 { &raw mut super::SGO } else { core::ptr::null_mut() }"),
    "Ground_GetYakumonoParam": ("(void*) YPB", "&raw mut super::YPB as *mut c_void"),
    "Mex_GrBind": ("a0", "a0"),
}
_STAGE_TYPES = ["HSD_GObj", "Ground", "StageInfo", "StageCallbacks", "StageData"]
_STAGE_SKIP = ["gp", "jobj"]
_STAGE_REAL = {"grDisplay_801C5DB0": "melee_world::tu::grdisplay::grDisplay_801C5DB0"}
SPECS["grfigure1"] = dict(src="src/melee/gr/grfigure1.c", types=_STAGE_TYPES, ret=_STAGE_RET,
                          null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"], real_cb=_STAGE_REAL, skip_data=_STAGE_SKIP)
SPECS["grfigure2"] = dict(src="src/melee/gr/grfigure2.c", types=_STAGE_TYPES, ret=_STAGE_RET,
                          null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"],
                          real_cb={**_STAGE_REAL, "grFigure1_8020E270": "melee_world::tu::grfigure1::grFigure1_8020E270"}, skip_data=_STAGE_SKIP)
SPECS["grfigure3"] = dict(src="src/melee/gr/grfigure3.c", types=_STAGE_TYPES, ret=_STAGE_RET,
                          null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"], real_cb=_STAGE_REAL, skip_data=_STAGE_SKIP)
SPECS["grfigureget"] = dict(
    src="src/melee/gr/grfigureget.c",
    types=["HSD_GObj", "Ground", "StageInfo", "StageCallbacks", "StageData", "CollData", "grFigureGet_Params"],
    ret={**_STAGE_RET,
         "Ground_GetYakumonoParam": ("(void*) &YP", "&raw mut super::YP as *mut c_void"),
         "ftLib_80086984": ("&FCD", "&raw mut super::FCD"),
         "it_802F2094": ("&IG[fv() % 3]", "&raw mut super::IG[(fv() % 3) as usize]"),
         "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
         },
    effects={"tyDisplay_8031C354": ("*a1 = (int) (fv() % 50);", "*a1 = (fv() % 50) as i32;")},
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"], real_cb=_STAGE_REAL, skip_data=_STAGE_SKIP)
SPECS["grflatzone"] = dict(
    src="src/melee/gr/grflatzone.c",
    types=["HSD_GObj", "Ground", "StageInfo", "StageCallbacks", "StageData", "grFlatzone_YakumonoParam", "HSD_JObj", "grDynamicAttr_UnkStruct",
           "CollData"],
    # GameCube union aliasing (see src/tu/grflatzone.rs): flatzone.xD0 / unk.xD0 / flatzone2.xD0 are one word
    # and flatzone.xD4 is flatzone2.timer; native 8-byte pointers break that, so the compiled copy of the TU
    # names the storage through flatzone2 everywhere (same substitution the port makes).
    patch=[("if (gp->u.flatzone.xD0 == 3) {", "if (gp->u.flatzone2.xD0 == 3) {"),
           ("gp->u.flatzone.xD0 = 0;", "gp->u.flatzone2.xD0 = 0;"),
           ("gp->u.flatzone.xD4 = yakumono_param->unk20;", "gp->u.flatzone2.timer = yakumono_param->unk20;"),
           ("gp->u.flatzone.xD4 = (s32) ((f32) gp->u.flatzone.xD4 / 2.0f);",
            "gp->u.flatzone2.timer = (s32) ((f32) gp->u.flatzone2.timer / 2.0f);"),
           ("gp->u.unk.xD0 = 3;", "gp->u.flatzone2.xD0 = 3;")],
    ret={**_STAGE_RET,
         "Ground_GetMapGObj": ("(((fv() & 7) == 0) ? 0 : &MGO[a0 % 10])",
                               "if fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::MGO[(a0 % 10) as usize] }"),
         "Ground_GetYakumonoParam": ("(void*) &YP", "&raw mut super::YP as *mut c_void"),
         "it_802EEFA8": ("(((fv() % 4) == 0) ? 0 : &IGO)", "if fv() % 4 == 0 { core::ptr::null_mut() } else { &raw mut super::IGO }"),
         "grDynamicAttr_801CA0F8": ("((fv() & 1) ? &DYN[0] : 0)", "if fv() & 1 != 0 { &raw mut super::DYN[0] } else { core::ptr::null_mut() }"),
         "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
         },
    effects={"mpCheckMultiple": (
        "a4->x = rw(); a4->y = rw(); a4->z = rw(); *a5 = ((fv() & 1) ? -1 : (int) (fv() % 100));",
        "(*a4).x = rw(); (*a4).y = rw(); (*a4).z = rw(); *a5 = if fv() & 1 != 0 { -1 } else { (fv() % 100) as i32 };")},
    null_ret_ok=["HSD_GObj_SetupProc"], real_cb=_STAGE_REAL, skip_data=_STAGE_SKIP + ["result"])
SPECS["grfourside"] = dict(
    src="src/melee/gr/grfourside.c",
    types=["HSD_GObj", "Ground", "StageInfo", "StageCallbacks", "StageData", "grFourside_YakumonoParam", "HSD_JObj",
           "CmSubject", "CollData", "GrJoint"],
    ret={**_STAGE_RET,
         "Ground_GetYakumonoParam": ("(void*) &YP", "&raw mut super::YP as *mut c_void"),
         "Ground_GetMapGObj": ("&MGO[a0 % 8]", "&raw mut super::MGO[(a0 % 8) as usize]"),
         "Ground_801C3FA4": ("&FJ[a1 % 10]", "&raw mut super::FJ[(a1 % 10) as usize]"),
         "Camera_80029020": ("((fv() & 1) ? &CMS : 0)", "if fv() & 1 != 0 { &raw mut super::CMS } else { core::ptr::null_mut() }"),
         "HSD_CObjGetCurrent": ("&COBJ", "&raw mut super::COBJ"),
         },
    effects={"lb_8000B1CC": ("a2->x = rw(); a2->y = rw(); a2->z = rw();", "(*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw();"),
             "HSD_CObjGetEyeVector": ("a1->x = rw(); a1->y = rg(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rg(); (*a1).z = rw();")},
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"], real_cb=_STAGE_REAL, skip_data=_STAGE_SKIP + ["result"])
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
