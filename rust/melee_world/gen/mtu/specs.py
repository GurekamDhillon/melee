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
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
