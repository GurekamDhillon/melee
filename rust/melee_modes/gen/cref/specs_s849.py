"""Oracle specs of shard 849-856 (see specs.py for the keys)."""
SPECS = {}
SPECS["lb_013B"] = dict(
    src="src/melee/lb/lb_013B.c",
    types=["ColorOverlay", "Fighter_804D653C_t"],
    custom_stubs={
        "Command_Execute": dict(
            c="int Command_Execute(CommandInfo* a0, u32 a1) { int r; ColorOverlay* c = (ColorOverlay*) a0; T(@ID@); T(PI(a0)); T(a1); "
              "r = a1 < 0xA ? 1 : (a1 < 0x15 ? 0 : (int) (fv() & 1)); if (r) { c->x8_ptr1++; if (rnd() % 4 == 0) c->x0_timer = (int) (rnd() % 3); } return r; }",
            ext="fn Command_Execute(a0: *mut CommandInfo, a1: u32) -> i32;",
            hook="|a0, a1| { t(@ID@); t(pi(a0)); t(a1); let r = if a1 < 0xA { 1 } else if a1 < 0x15 { 0 } else { (fv() & 1) as i32 }; "
                 "if r != 0 { let c = a0 as *mut ColorOverlay; unsafe { (*c).x8_ptr1 = (*c).x8_ptr1.add(1); if rnd() % 4 == 0 { (*c).x0_timer = (rnd() % 3) as i32; } } } r }"),
        "lbArchive_80017040": dict(
            c="int lbArchive_80017040(HSD_Archive** a0, const char* a1, void* a2, ...) { __builtin_va_list ap; const char* sym; int z; "
              "__builtin_va_start(ap, a2); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
              "T(@ID@); T(PI(a0)); T(SH(a1)); T(PI(a2)); T(SH(sym)); T((u32) z); *(void**) a2 = RUM; return (int) (fv() & 1); }",
            ext="fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: *const i8, a4: i32) -> i32;",
            hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(sh(a1 as *const u8)); t(pi(a2)); t(sh(a3 as *const u8)); t(a4 as u32); "
                 "unsafe { *(a2 as *mut *mut c_void) = (&raw mut super::RUM) as *mut c_void; } (fv() & 1) as i32 }"),
    },
)
SPECS["lb_0146"] = dict(
    src="src/melee/lb/lb_0146.c",
    types=["lb_80014638_arg0_t", "lb_80014638_arg1_t", "Vec3"],
    ret={"HSD_CObjGetCurrent": ("&COB", "&raw mut super::COB")},
)
SPECS["lb_0192"] = dict(
    src="src/melee/lb/lb_0192.c",
    types=["HSD_Text", "gmMainLib_8046B0F0_t"],
    ret={
        "DVDGetDriveStatus": ("DVDT[rnd() % 16]", "super::DVDT[(rnd() % 16) as usize]"),
        "lbLang_IsSavedLanguageJP": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
        "HSD_SisLib_803A6754": ("&TXT", "&raw mut super::TXT"),
    },
    custom_stubs={
        "HSD_SisLib_803A6B98": dict(
            c="int HSD_SisLib_803A6B98(HSD_Text* a0, float a1, float a2, const char* a3, ...) { __builtin_va_list ap; const char* s; "
              "__builtin_va_start(ap, a3); s = __builtin_va_arg(ap, const char*); __builtin_va_end(ap); "
              "T(@ID@); T(PI(a0)); T(B(a1)); T(B(a2)); T(SH(a3)); T(SH(s)); return (int) fv(); }",
            ext="fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: *const i8) -> i32;",
            hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(a1.to_bits()); t(a2.to_bits()); t(sh(a3 as *const u8)); t(sh(a4 as *const u8)); fv() as i32 }"),
        "HSD_SisLib_803A70A0": dict(
            c="s32 HSD_SisLib_803A70A0(HSD_Text* a0, s32 a1, char* a2, ...) { __builtin_va_list ap; const char* s; "
              "__builtin_va_start(ap, a2); s = __builtin_va_arg(ap, const char*); __builtin_va_end(ap); "
              "T(@ID@); T(PI(a0)); T((u32) a1); T(SH(a2)); T(SH(s)); return (s32) fv(); }",
            ext="fn HSD_SisLib_803A70A0(a0: *mut HSD_Text, a1: i32, a2: *const i8, a3: *const i8) -> i32;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(a1 as u32); t(sh(a2 as *const u8)); t(sh(a3 as *const u8)); fv() as i32 }"),
        # the C passes (u32) of the text pointer (a PPC-ism); the stub names it by identity
        "HSD_SisLib_803A84BC": dict(
            c="void HSD_SisLib_803A84BC(HSD_GObj* a0, int a1) { T(@ID@); T(PI(a0)); T((u32) a1 == (u32) (unsigned long long) &TXT); }",
            ext="fn HSD_SisLib_803A84BC(a0: *mut HSD_GObj, a1: *mut HSD_Text);",
            hook="|a0, a1| { t(@ID@); t(pi(a0)); t((a1 == (&raw mut super::TXT)) as u32); }"),
    },
)
SPECS["lb_0195"] = dict(
    src="src/melee/lb/lb_0195.c",
    types=["UnkArrElem", "lb_804329F0_t"],
    ret={"lbCardNew_CompleteNextTask": ("(int) (fv() % 14)", "(fv() % 14) as i32")},
)
SPECS["lb_01F8"] = dict(
    src="src/melee/lb/lb_01F8.c",
    types=["lbl_804335B8_t", "HSD_SObj"],
    skip_stubs=["memset"],
    ret={
        "HSD_MemAlloc": ("(void*) (MB + 64 * (MBN++ & 7))",
                         "{ let r = (&raw mut super::MB as *mut u8).add(64 * ((super::MBN as usize) & 7)) as *mut c_void; super::MBN += 1; r }"),
        "HSD_SObjLib_803A477C": ("&SOB", "&raw mut super::SOB"),
        "THPVideoDecode": ("&DEC", "&raw mut super::DEC as *mut c_void"),
    },
    effects={
        "lbFile_80016760": ("*a1 = (void*) (MB + 528); *a2 = fv();",
                            "*a1 = (&raw mut super::MB as *mut u8).add(528) as *mut c_void; *a2 = fv();"),
        "THPVideoDecode": ("T(RH(a0, 0x1C)); *(int*) a1 = (int) fv();",
                           "t(rh(&*(a0 as *const [u8; 0x1C]))); *(a1 as *mut i32) = fv() as i32;"),
    },
)
SPECS["lb_0219"] = dict(
    src="src/melee/lb/lb_0219.c",
    types=["BgFlashUserData", "HSD_ObjAllocData", "HSD_GObj"],
    null_ret_ok=["HSD_GObj_SetupProc"],
    ret={
        "GObj_Create": ("((rnd() % 4 == 0) ? 0 : &GNEW)", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { &raw mut super::GNEW }"),
        "HSD_ObjAlloc": ("((rnd() % 4 == 0) ? 0 : &UD)", "if rnd() % 4 == 0 { core::ptr::null_mut() } else { &raw mut super::UD as *mut c_void }"),
        "lb_80014258": ("(int) (rnd() % 3 == 0)", "(rnd() % 3 == 0) as i32"),
        "lb_800144C8": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
    },
    effects={
        "GObj_InitUserData": ("a0->user_data_kind = a1; a0->user_data_remove_func = a2; a0->user_data = a3;",
                              "(*a0).user_data_kind = a1; (*a0).user_data_remove_func = a2; (*a0).user_data = a3;"),
        "lbBgFlash_InitState": ("T(a0->r); T(a0->g); T(a0->b); T(a0->a);",
                                "t((*a0).r as u32); t((*a0).g as u32); t((*a0).b as u32); t((*a0).a as u32);"),
    },
    custom_stubs={
        "lbArchive_LoadSymbols": dict(
            c="HSD_Archive* lbArchive_LoadSymbols(const char* a0, void* a1, ...) { __builtin_va_list ap; "
              "const char* sym; int z; __builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); "
              "z = __builtin_va_arg(ap, int); __builtin_va_end(ap); T(@ID@); T(SH(a0)); T(PI(a1)); T(SH(sym)); "
              "T((u32) z); *(void**) a1 = (rnd() & 1) ? LBF : 0; return 0; }",
            ext="fn lbArchive_LoadSymbols(a0: *const u8, a1: *mut c_void, a2: *const u8, a3: i32) -> *mut c_void;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(sh(a0)); t(pi(a1)); t(sh(a2)); t(a3 as u32); "
                 "unsafe { *(a1 as *mut *mut c_void) = if rnd() & 1 != 0 { (&raw mut super::LBF) as *mut c_void } else { core::ptr::null_mut() }; } core::ptr::null_mut() }"),
    },
)
def _fillf(arg, n, rarg=None):
    """effect: n random floats through the pointer `arg` (same draws on both sides)"""
    return (f"{{ int i_; for (i_ = 0; i_ < {n}; i_++) ((float*) {arg})[i_] = rg(); }}",
            f"for i_ in 0..{n} {{ *({rarg or arg} as *mut f32).add(i_) = rg(); }}")


def _eff(*parts):
    return (" ".join(p[0] for p in parts), " ".join(p[1] for p in parts))


SPECS["lb_020A"] = dict(
    src="src/melee/lb/lb_020A.c",
    types=["HSD_JObj", "IKState", "Vec3"],
    skip_stubs=["__frsqrte", "fabs"],
    skip_data=["result"],
    # fn_8002113C reads and writes `rot` (a Vec3) through a Quaternion*: the .w lands in whatever
    # the compiler puts after it (UB). The port keeps the .w with the rotation; make the oracle's
    # slot 16 bytes so it does too.
    patch=[("    Vec3 rot;" + chr(10) + "    Quaternion rot2;", "    Quaternion rot;" + chr(10) + "    Quaternion rot2;"),
           ("HSD_MkRotationMtx(tmpMtx, &rot);", "HSD_MkRotationMtx(tmpMtx, (Vec3*) &rot);"),
           ("HSD_QuatLib_8037EB28(result, &rot);", "HSD_QuatLib_8037EB28(result, (Vec3*) &rot);")],
    ret={
        "PSVECMag": ("rg()", "rg()"),
        "lbVector_Angle": ("rg()", "rg()"),
        "lbVector_Normalize": ("rg()", "rg()"),
        "lbVector_Diff": ("a2", "a2"),
        "acosf": ("a0 + 0.25f", "a0 + 0.25"),
        "atan2f": ("a0 - a1", "a0 - a1"),
        "MatToQuat": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
        "HSD_QuatLib_8037EB28": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
    },
    effects={
        "HSD_JObjSetupMatrixSub": _eff(_fillf("&a0->mtx", 12, "&raw mut (*a0).mtx"),
                                       ("a0->flags &= ~JOBJ_MTX_DIRTY;", "(*a0).flags &= !(JOBJ_MTX_DIRTY as u32);")),
        "HSD_JObjSetMtxDirtySub": ("a0->flags |= JOBJ_MTX_DIRTY;", "(*a0).flags |= JOBJ_MTX_DIRTY as u32;"),
        "HSD_MtxInverseConcat": _fillf("a2", 12),
        "PSMTXConcat": _fillf("a2", 12),
        "PSMTXCopy": ("{ int i_; for (i_ = 0; i_ < 12; i_++) ((float*) a1)[i_] = ((float*) a0)[i_]; }",
                      "for i_ in 0..12 { *(a1 as *mut f32).add(i_) = *(a0 as *const f32).add(i_); }"),
        "PSMTXTranspose": _fillf("a1", 12),
        "PSMTXMultVec": _fillf("a2", 3),
        "PSMTXRotAxisRad": _fillf("a0", 12),
        "HSD_MkRotationMtx": _fillf("a0", 12),
        "HSD_MtxQuat": _fillf("a0", 12),
        "PSMTXQuat": _fillf("a0", 12),
        "MTXRotRad": _fillf("a0", 12),
        "MatToQuat": _fillf("a1", 4),
        "HSD_QuatLib_8037EB28": _fillf("a1", 3),
        "lbVector_Normalize": _fillf("a0", 3),
        "lbVector_Diff": ("a2->x = a0->x - a1->x; a2->y = a0->y - a1->y; a2->z = a0->z - a1->z;",
                          "(*a2).x = (*a0).x - (*a1).x; (*a2).y = (*a0).y - (*a1).y; (*a2).z = (*a0).z - (*a1).z;"),
    },
)
SPECS["mncharsel"] = dict(src="src/melee/mn/mncharsel.c", types=["CSSData"])
for _s in SPECS.values():
    _s.setdefault("narrow_si", True)
    _s.setdefault("common_ret", {})
