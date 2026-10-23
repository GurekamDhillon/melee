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

_RH1 = ("T(RH(a1, sizeof *a1));", "t(rh(&*a1));")
SPECS["textdraw"] = dict(
    src="src/melee/if/textdraw.c",
    types=["DevText", "HSD_GObj"],
    ret={
        "GObj_Create": ("(((fv() & 3) == 0) ? 0 : &GO[fv() % 4])",
                        "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GO[(fv() % 4) as usize] }"),
        "HSD_CObjAlloc": ("&COBJA", "&raw mut super::COBJA"),
        "HSD_CObjLoadDesc": ("((fv() & 1) ? &COBJB : 0)", "if fv() & 1 != 0 { &raw mut super::COBJB } else { core::ptr::null_mut() }"),
        "Wb_RenderActive": ("(int) ((fv() & 7) == 0)", "((fv() & 7) == 0) as i32"),
    },
    effects={"DrawRectangle": ("T(RH(a4, 4));", "t(rh(&*a4));"), "DrawASCII": ("T(RH(a3, 4));", "t(rh(&*a3));"),
             "HSD_CObjSetViewport": _RH1, "HSD_CObjSetScissor": _RH1, "HSD_CObjSetEyePosition": _RH1,
             "HSD_CObjSetInterest": _RH1})
SPECS["textlib"] = dict(
    src="src/melee/if/textlib.c", types=["DevText", "un_80304138_objalloc_t"],
    skip_data=["devtext_drawlist", "devtext_poolhead", "un_804D6E44"],
    skip_stubs=["memcpy", "memzero", "vsnprintf", "__builtin_va_end"],
    effects={"DevText_NumToStr": ("a1[0] = (char) (33 + fv() % 90); a1[1] = (fv() & 1) ? '\\n' : 'q'; a1[2] = 0;",
                                  "*a1 = (33 + fv() % 90) as i8; *a1.add(1) = if fv() & 1 != 0 { 10 } else { b'q' as i8 }; *a1.add(2) = 0;")})
_GXC = ("((GXColor) { (u8) fv(), (u8) fv(), (u8) fv(), (u8) fv() })", "_GXColor { r: fv() as u8, g: fv() as u8, b: fv() as u8, a: fv() as u8 }")
SPECS["textlib_1"] = dict(
    src="src/melee/if/textlib_1.c",
    types=["DevText", "un_80304138_objalloc_t", "un_80304138_objalloc_t_x8", "HSD_GObj"],
        ret={"DevText_SetTextColor": _GXC, "DevText_SetBGColor": _GXC,
         "DevText_StoreColorIndex": ("(u8) fv()", "fv() as u8"),
         "DevText_Create": ("((fv() & 3) == 0 ? 0 : &DT[fv() % 4])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::DT[(fv() % 4) as usize] }"),
         "DevText_GetGObj": ("&GOBJ0", "&raw mut super::GOBJ0"),
         "GObj_Create": ("((fv() & 3) == 0 ? 0 : &GOBJ1)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GOBJ1 }"),
         "HSD_MemAlloc": ("((fv() & 3) == 0 ? 0 : (void*) MEM)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::MEM as *mut c_void }"),
         "HSD_ObjAlloc": ("((NULLOK && (fv() & 3) == 0) ? 0 : (void*) &OBJS[fv() % 4])", "if super::NULLOK != 0 && fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::OBJS[(fv() % 4) as usize] as *mut c_void }"),
         "HSD_GObj_SetupProc": ("&PROC0", "&raw mut super::PROC0")},
    custom_stubs={"DevText_Printf": dict(
        c="void DevText_Printf(DevText* a0, char* a1, ...) { __builtin_va_list ap; const char* f = a1; T(@ID@); T(PI(a0)); T(SH(a1)); "
          "__builtin_va_start(ap, a1); while (*f) { if (*f++ != '%') continue; while (*f == '-' || *f == '+' || *f == ' ' || *f == '#' || (*f >= '0' && *f <= '9') || *f == '.') f++; "
          "if (*f == 'f') { T64(__builtin_va_arg(ap, double)); } else { T((u32) __builtin_va_arg(ap, int)); } f++; } __builtin_va_end(ap); }",
        ext="fn DevText_Printf(a0: *mut DevText, a1: *const i8, a2: &[crate::tu::textlib::PrintfArg]);",
        hook="|a0, a1, a2| { t(@ID@); t(pi(a0)); t(sh(a1 as *const u8)); for a in a2 { match a { melee_world::tu::textlib::PrintfArg::Int(v) => t(*v as u32), melee_world::tu::textlib::PrintfArg::Dbl(d) => t64(*d), _ => {} } } }")})
SPECS["if_2F72"] = dict(
    src="src/melee/if/if_2F72.c",
    types=["HSD_GObj", "Element_803F9628", "HudIndex", "HSD_JObj"],
    data_size={"ifStatus_803F9628": 8},
    skip_stubs=["memzero"], skip_data=["result"],
    ret={
        "GObj_Create": ("(((fv() & 3) == 0) ? 0 : &NG[fv() % 4])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::NG[(fv() % 4) as usize] }"),
        "HSD_JObjLoadJoint": ("(((fv() & 3) == 0) ? 0 : &NJ[fv() % 4])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::NJ[(fv() % 4) as usize] }"),
        "HSD_GObj_SetupProc": ("&PROC0", "&raw mut super::PROC0"),
        "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
        "ifAll_GetPlayerHUDPosition": ("&HPOS[(u32) a0 % 6]", "&raw mut super::HPOS[(a0 as u32 % 6) as usize]"),
        "ifStatus_GetHUDInfo": ("&HI", "&raw mut super::HI"),
        "Player_80036428": ("(int) (fv() % 6)", "(fv() % 6) as i32"),
        "gm_8016AEC8": ("gm_ret()", "super::gm_ret()"),
        "lbGetJObjCurrFrame": ("(float) (fv() % 40) / 2.0f", "(fv() % 40) as f32 / 2.0"),
    },
    custom_stubs={"lbArchive_LoadSections": dict(
        c="void lbArchive_LoadSections(HSD_Archive* a0, void** a1, ...) { __builtin_va_list ap; const char* sym; int z; "
          "__builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
          "T(@ID@); T(PI(a0)); T(PI(a1)); T(SH(sym)); T((u32) z); *a1 = ((fv() & 1) ? (void*) DMDA : 0); }",
        ext="fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut *mut c_void, a2: *const u8, a3: i32);",
        hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(pi(a1)); t(sh(a2)); t(a3 as u32); *a1 = if fv() & 1 != 0 { &raw mut super::DMDA as *mut c_void } else { core::ptr::null_mut() }; }")})
SPECS["if_2FD9"] = dict(
    src="src/melee/if/if_2FD9.c",
    types=["HSD_GObj"],
    skip_data=["result"],
    ret={
        "Player_GetPlayerSlotType": ("(Gm_PKind) (fv() % 3)", "fv() % 3"),
        "GObj_Create": ("&NG[fv() % 4]", "&raw mut super::NG[(fv() % 4) as usize]"),
        "HSD_JObjLoadJoint": ("&NJ[fv() % 4]", "&raw mut super::NJ[(fv() % 4) as usize]"),
        "HSD_GObj_SetupProc": ("&PROC0", "&raw mut super::PROC0"),
        "ifAll_GetArchive": ("&ARCP", "&raw mut super::ARCP"),
    },
    custom_stubs={"lbArchive_LoadSections": dict(
        c="void lbArchive_LoadSections(HSD_Archive* a0, void** a1, ...) { __builtin_va_list ap; const char* sym; int z; "
          "__builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
          "T(@ID@); T(PI(a0)); T(PI(a1)); T(SH(sym)); T((u32) z); *a1 = (void*) DMDA; }",
        ext="fn lbArchive_LoadSections(a0: *mut HSD_Archive, a1: *mut *mut c_void, a2: *const u8, a3: i32);",
        hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(pi(a1)); t(sh(a2)); t(a3 as u32); *a1 = &raw mut super::DMDA as *mut c_void; }")})
SPECS["if_2FF2"] = dict(
    src="src/melee/if/if_2FF2.c",
    types=["HSD_GObj", "HSD_Text"],
    skip_stubs=["memzero"],
    ret={
        "HSD_SisLib_803A6754": ("&TX[fv() % 3]", "&raw mut super::TX[(fv() % 3) as usize]"),
        "GObj_Create": ("(((fv() & 3) == 0) ? 0 : &NG[fv() % 4])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::NG[(fv() % 4) as usize] }"),
        "ifAll_GetHUDGObj": ("&HG", "&raw mut super::HG"),
        "ifAll_GetPlayerHUDPosition": ("&HPOS[(u32) a0 % 6]", "&raw mut super::HPOS[(a0 as u32 % 6) as usize]"),
        "HSD_LObjLoadDesc": ("&LO", "&raw mut super::LO"),
        "HSD_CObjLoadDesc": ("((fv() & 1) ? &CO : 0)", "if fv() & 1 != 0 { &raw mut super::CO } else { core::ptr::null_mut() }"),
        "DevText_Setup": ("((fv() & 1) ? &HG : 0)", "if fv() & 1 != 0 { &raw mut super::HG } else { core::ptr::null_mut() }"),
        "HSD_GObj_SetupProc": ("&PROC0", "&raw mut super::PROC0"),
        "gm_8016B774": ("&ME", "&raw mut super::ME"),
    },
    effects={
        "HSD_CObjLoadDesc": (
            "{ HSD_CameraDescPerspective* d = (HSD_CameraDescPerspective*) a0; T(B(d->nnear)); T(B(d->ffar)); T(B(d->fov)); T(B(d->aspect)); T(B(d->roll)); "
            "T(d->flags); T(d->projection_type); T(RH(&d->viewport, 8)); T(RH(&d->scissor, 8)); T(PI(d->up_vector)); T(PI(d->class_name)); "
            "T(B(d->eyepos->pos.x)); T(B(d->eyepos->pos.y)); T(B(d->eyepos->pos.z)); T(B(d->interest->pos.x)); T(B(d->interest->pos.y)); T(B(d->interest->pos.z)); "
            "T(PI(d->eyepos->robjdesc)); T(PI(d->eyepos->class_name)); }",
            "{ let d = &*(a0 as *mut HSD_CameraDescPerspective); t(d.nnear.to_bits()); t(d.ffar.to_bits()); t(d.fov.to_bits()); t(d.aspect.to_bits()); t(d.roll.to_bits()); "
            "t(d.flags as u32); t(d.projection_type as u32); t(rh(&d.viewport)); t(rh(&d.scissor)); t(pi(d.up_vector)); t(pi(d.class_name)); "
            "t((*d.eyepos).pos.x.to_bits()); t((*d.eyepos).pos.y.to_bits()); t((*d.eyepos).pos.z.to_bits()); t((*d.interest).pos.x.to_bits()); t((*d.interest).pos.y.to_bits()); t((*d.interest).pos.z.to_bits()); "
            "t(pi((*d.eyepos).robjdesc)); t(pi((*d.eyepos).class_name)); }"),
        "HSD_LObjLoadDesc": (
            "{ HSD_LightDesc* d; for (d = a0; d; d = d->next) { T(d->flags); T(d->attnflags); T(RH(&d->color, 4)); T(PI(d->class_name)); "
            "if (d->position) { T(B(d->position->pos.x)); T(B(d->position->pos.y)); T(B(d->position->pos.z)); } else T(0xAAAA); "
            "T(PI(d->interest)); if (d->u.point) { T(B(d->u.point->ref_br)); T(B(d->u.point->ref_dist)); T(d->u.point->dist_func); } else T(0xBBBB); } }",
            "{ let mut d = a0; while !d.is_null() { t((*d).flags as u32); t((*d).attnflags as u32); t(rh(&(*d).color)); t(pi((*d).class_name)); "
            "if !(*d).position.is_null() { t((*(*d).position).pos.x.to_bits()); t((*(*d).position).pos.y.to_bits()); t((*(*d).position).pos.z.to_bits()); } else { t(0xAAAA); } "
            "t(pi((*d).interest)); if !(*d).u.point.is_null() { t((*(*d).u.point).ref_br.to_bits()); t((*(*d).u.point).ref_dist.to_bits()); t((*(*d).u.point).dist_func); } else { t(0xBBBB); } d = (*d).next; } }"),
    },
    custom_stubs={
        "HSD_SisLib_803A70A0": dict(
            c="s32 HSD_SisLib_803A70A0(HSD_Text* a0, s32 a1, char* a2, ...) { __builtin_va_list ap; const char* f = a2; T(@ID@); T(PI(a0)); T((u32) a1); T(SH(a2)); "
              "__builtin_va_start(ap, a2); while (*f) { if (*f++ == '%') { f++; T((u32) __builtin_va_arg(ap, int)); } } __builtin_va_end(ap); return (s32) fv(); }",
            ext="fn HSD_SisLib_803A70A0(a0: *mut HSD_Text, a1: i32, a2: *const i8, a3: &[i32]) -> i32;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(a1 as u32); t(sh(a2 as *const u8)); for v in a3 { t(*v as u32); } fv() as i32 }"),
        "HSD_SisLib_803A6B98": dict(
            c="int HSD_SisLib_803A6B98(HSD_Text* a0, float a1, float a2, const char* a3, ...) { __builtin_va_list ap; const char* f = a3; T(@ID@); T(PI(a0)); T(B(a1)); T(B(a2)); T(SH(a3)); "
              "__builtin_va_start(ap, a3); while (*f) { if (*f++ == '%') { f++; T((u32) __builtin_va_arg(ap, int)); } } __builtin_va_end(ap); return (int) fv(); }",
            ext="fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: &[i32]) -> i32;",
            hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(a1.to_bits()); t(a2.to_bits()); t(sh(a3 as *const u8)); for v in a4 { t(*v as u32); } fv() as i32 }")})
SPECS["itanimlist"] = dict(
    crate="melee_items", src="src/melee/it/itanimlist.c", types=[],
    # on the console `ptr + 1` (an itAnimlistCmdUnk is 4 bytes) is the next command word; natively a
    # CmdUnion is 8 bytes, so the port (and the compiled copy of the TU) step one CmdUnion instead
    patch=[("cmd->u = (union CmdUnion*) (ptr + 1);", "cmd->u = cmd->u + 1;")],
    ret={"Command_Execute": ("CE_RET", "super::CE_RET"), "lb_80014258": ("(int) (fv() % 3)", "(fv() % 3) as i32")},
    effects={
        "it_80278800": (
            "{ T(B(a3->x)); T(B(a3->y)); T(B(a3->z)); T(B(a4->x)); T(B(a4->y)); T(B(a4->z)); }",
            "{ t((*a3).x.to_bits()); t((*a3).y.to_bits()); t((*a3).z.to_bits()); t((*a4).x.to_bits()); t((*a4).y.to_bits()); t((*a4).z.to_bits()); }"),
        "Command_Execute": (
            "{ int ok = (a1 >= 10 && a1 <= 25) && (a0->u < &W[6]) && ((fv() & 3) != 0); CE_RET = ok ? 0 : 1; "
            "if (CE_RET) { u32 q = fv() & 3; if (q == 0) a0->u = 0; else if (q == 1) a0->timer = 1.0f + (float) (fv() % 3); "
            "else if (a0->u < &W[10]) a0->u = &W[6 + (fv() % 4)]; else a0->u = 0; } }",
            "{ let ok = (a1 >= 10 && a1 <= 25) && ((*a0).anon_8.u < &raw mut super::W[6]) && (fv() & 3) != 0; "
            "super::CE_RET = if ok { 0 } else { 1 }; "
            "if super::CE_RET != 0 { let q = fv() & 3; if q == 0 { (*a0).anon_8.u = core::ptr::null_mut(); } "
            "else if q == 1 { (*a0).timer = 1.0 + (fv() % 3) as f32; } "
            "else if (*a0).anon_8.u < &raw mut super::W[10] { (*a0).anon_8.u = &raw mut super::W[6 + (fv() % 4) as usize]; } "
            "else { (*a0).anon_8.u = core::ptr::null_mut(); } } }"),
        "lb_80014258": (
            "if (a2 && (GKIND == 0 || GKIND == 6) && CMDX.u == &W[0] && (fv() & 1)) a2(a0, &CMDX, GKIND == 0 ? 21 : 22);",
            "if let Some(f) = a2 { if (super::GKIND == 0 || super::GKIND == 6) && (*(&raw mut super::CMDX)).anon_8.u == &raw mut super::W[0] && fv() & 1 != 0 "
            "{ f(a0, &raw mut super::CMDX, if super::GKIND == 0 { 21 } else { 22 }); } }")},
)
for _s in SPECS.values():
    _s["common_ret"] = COMMON_RET
