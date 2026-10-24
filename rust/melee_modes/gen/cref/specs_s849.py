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
for _s in SPECS.values():
    _s.setdefault("narrow_si", True)
    _s.setdefault("common_ret", {})
