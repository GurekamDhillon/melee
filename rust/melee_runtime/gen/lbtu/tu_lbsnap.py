"""Oracle spec + world of src/melee/lb/lbsnap.c (imported by specs.py / cfg.py)."""
L = "src/melee/lb/"
_TD = ("typedef struct Unk80433380_48 Unk80433380_48; typedef struct Unk80433380_0 Unk80433380_0; "
       "typedef struct Unk80433380 Unk80433380; typedef struct Unk803BACC8 Unk803BACC8;" + chr(10))
SPEC = dict(
    src=L + "lbsnap.c",
    types=["Unk80433380", "Unk80433380_48", "Unk80433380_0", "Unk803BACC8"],
    # __OSBusClock maps to guest memory under TARGET_PC: the oracle makes it a variable; the typedefs let the
    # world name the file's structs without `struct`
    patch=[("#define _p(x) (lbSnap_80433380.x)", "#undef __OSBusClock" + chr(10) + "extern unsigned int __OSBusClock;" + chr(10) + "#define _p(x) (lbSnap_80433380.x)"),
           ("static struct Unk80433380 lbSnap_80433380;", _TD + "static struct Unk80433380 lbSnap_80433380;")],
    ret={"OSGetTime": ("({ u32 x_ = fv(); (long long) (x_ % 8) * 40500000LL + (long long) ((x_ >> 8) % 1000); })",
                       "{ let x = fv(); (x % 8) as i64 * 40500000i64 + ((x >> 8) % 1000) as i64 }"),
         "lbLang_IsSettingJP": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
    effects={
        "lb_8001BFD8": (
            "{ int i; for (i = 0; i < 0x7F; i++) { a1[i].time = fv(); a1[i].file_no = (fv() % 4 == 0) ? -1 : (s16) fv(); a1[i].blocks = (u16) fv(); } *a2 = (int) fv(); *a3 = (int) fv(); }",
            "for i in 0..0x7Fusize { (*a1.add(i)).time = fv(); (*a1.add(i)).file_no = if fv() % 4 == 0 { -1 } else { fv() as i16 }; (*a1.add(i)).blocks = fv() as u16; } *a2 = fv() as i32; *a3 = fv() as i32;"),
        "it_8026C47C": (
            "{ unsigned i; for (i = 0; i < sizeof *a0 / 4; i++) ((int*) a0)[i] = (int) fv(); }",
            "for i in 0..core::mem::size_of::<it_8026C47C_arg0_t>() / 4 { *(a0 as *mut i32).add(i) = fv() as i32; }"),
        "OSTicksToCalendarTime": (
            "if (a1) { a1->sec = fv() % 100; a1->min = fv() % 100; a1->hour = fv() % 100; a1->mday = fv() % 100; a1->mon = fv() % 12; a1->year = fv(); }",
            "if !a1.is_null() { (*a1).sec = (fv() % 100) as i32; (*a1).min = (fv() % 100) as i32; (*a1).hour = (fv() % 100) as i32; (*a1).mday = (fv() % 100) as i32; (*a1).mon = (fv() % 12) as i32; (*a1).year = fv() as i32; }"),
    },
    custom_stubs={
        # the intptr_t image pointer reaches the callee as an s32 (native: truncated, run-dependent): not traced
        "hsd_803B51C8": dict(
            c="int hsd_803B51C8(int a0, int a1, int a2, char* a3, int a4) { (void) a0; T(@ID@); T((u32) a1); T((u32) a2); T(PI(a3)); T((u32) a4); return (int) fv(); }",
            ext="fn hsd_803B51C8(a0: i32, a1: i32, a2: i32, a3: *mut i8, a4: i32) -> i32;",
            hook="|a0, a1, a2, a3, a4| { t(@ID@); t(a1 as u32); t(a2 as u32); t(pi(a3)); t(a4 as u32); fv() as i32 }"),
        "lbArchive_80016DBC": dict(
            c="HSD_Archive* lbArchive_80016DBC(const char* a0, void* a1, ...) { __builtin_va_list ap; const char* sym; int z; "
              "__builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
              "T(@ID@); T(SH(a0)); T(PI(a1)); T(SH(sym)); T((u32) z); *(void**) a1 = ICON; return 0; }",
            ext="fn lbArchive_80016DBC(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut HSD_Archive;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(sh(a0 as *const u8)); t(pi(a1)); t(sh(a2 as *const u8)); t(a3 as u32); "
                 "*(a1 as *mut *mut crate::m::LbMcSnapMemSnapIconData) = &raw mut crate::ICON as *mut _; core::ptr::null_mut() }"),
    },
)

_CHAN = ("(int) (fv() & 1)", "(fv() & 1) as i32")
_I8 = ("(int) (fv() % 8)", "(fv() % 8) as i32")
_SRC_FILL = ("{ int q; for (q = 0; q < 16; q++) SRCT[q] = rnd(); for (q = 0; q < 0x70000 / 4; q++) ((u32*) SRC)[q] = SRCT[q & 15] ^ ((u32) q * 2654435761u); }",
             "for q in 0..16usize { SRCT[q] = rnd(); } for q in 0..0x70000usize / 4 { *(SRC.as_mut_ptr() as *mut u32).add(q) = SRCT[q & 15] ^ (q as u32).wrapping_mul(2654435761u32); }")
CFG = dict(
    world=[("Unk80433380_0", "SNAP", 1), ("Unk80433380_48", "SLOT", 2), ("LbMcSnapMemSnapIconData", "ICON", 2),
           ("unsigned char", "BANNER", 0x1800 + 64), ("unsigned char", "ICNB", 0x400), ("unsigned char", "SRC", 0x70000),
           ("char", "TXT", 0x40)],
    rtype={"unsigned char": "u8", "char": "i8", "Unk80433380_0": "m::Unk80433380_0", "Unk80433380_48": "m::Unk80433380_48",
           "LbMcSnapMemSnapIconData": "m::LbMcSnapMemSnapIconData"},
    c_decl="static u32 SRCT[16];",
    rs_decl="static mut SRCT: [u32; 16] = [0; 16];",
    c_preg=["preg(&lbSnap_80433380, sizeof lbSnap_80433380);", "preg(&lbSnap_803BACC8, sizeof lbSnap_803BACC8);"],
    rs_preg=["preg(&raw const m::lbSnap_80433380, core::mem::size_of_val(&m::lbSnap_80433380)); preg(&raw const m::lbSnap_803BACC8, core::mem::size_of_val(&m::lbSnap_803BACC8));"],
    argmap={"Unk80433380_48": [("&SLOT[0]", "&raw mut SLOT[0]")],
            "void": [("SRC", "SRC.as_mut_ptr() as *mut core::ffi::c_void")],
            "char": [("TXT", "TXT.as_mut_ptr()")],
            "unsigned char": [("SRC", "SRC.as_mut_ptr()")],
            "const unsigned char": [("SRC", "SRC.as_ptr()")],
            "u8": [("SRC", "SRC.as_mut_ptr()")]},
    hints={"chan": _CHAN, "index": _I8, "jndex": _I8, "i": _CHAN,
           "arg0": ("(int) (fv() % 3)", "(fv() % 3) as i32")},
    pre={"lbSnap_8001DA5C": _SRC_FILL, "lbSnap_8001DC0C": _SRC_FILL, "lbSnap_8001DE8C": _SRC_FILL},
    setup=(
"""    __OSBusClock = 162000000u;
    for (k = 0; k < 2; k++) {
        int q;
        fill_Unk80433380_48(&SLOT[k]);
        SLOT[k].num = 8 + (int) (rnd() % 100);
        SLOT[k].card_result = (rnd() % 3 == 0) ? 0 : (int) fv();
        for (q = 0; q < 0x7F; q++) SLOT[k].entries[q].time = rnd() % 6;
    }
    fill_Unk80433380_0(&SNAP[0]);
    SNAP[0].x0 = (rnd() & 1) ? 4 : (int) fv();
    fill_Unk80433380(&lbSnap_80433380);
    lbSnap_80433380.snap = &SNAP[0];
    lbSnap_80433380.slot = &SLOT[0];
    lbSnap_80433380.icon_data = ICON;
    ICON[0].ptr = BANNER; ICON[1].ptr = ICNB;
    for (k = 0; k < 0x40; k++) TXT[k] = (char) (rnd() | 1);
    for (k = 0; k < 0x1800 + 64; k++) BANNER[k] = (unsigned char) rnd();
    for (k = 0; k < 0x400; k++) ICNB[k] = (unsigned char) rnd();""",
"""    m::ext::__OSBusClock = 162000000;
    for k in 0..2usize {
        g::fill_Unk80433380_48(&mut SLOT[k]);
        SLOT[k].num = 8 + (rnd() % 100) as i32;
        SLOT[k].card_result = if rnd() % 3 == 0 { 0 } else { fv() as i32 };
        for q in 0..0x7Fusize { SLOT[k].entries[q].time = rnd() % 6; }
    }
    g::fill_Unk80433380_0(&mut SNAP[0]);
    SNAP[0].x0 = if rnd() & 1 != 0 { 4 } else { fv() as i32 };
    g::fill_Unk80433380(&mut m::lbSnap_80433380);
    m::lbSnap_80433380.snap = &raw mut SNAP[0];
    m::lbSnap_80433380.slot = &raw mut SLOT[0];
    m::lbSnap_80433380.icon_data = &raw mut ICON[0];
    ICON[0].ptr = BANNER.as_mut_ptr(); ICON[1].ptr = ICNB.as_mut_ptr();
    for k in 0..0x40usize { TXT[k] = (rnd() | 1) as i8; }
    for k in 0..0x1800 + 64usize { BANNER[k] = rnd() as u8; }
    for k in 0..0x400usize { ICNB[k] = rnd() as u8; }"""),
    ptrs=("    hash_Unk80433380(&lbSnap_80433380);\n    hash_Unk803BACC8(&lbSnap_803BACC8);\n    HM(PIB(ICON[0].ptr)); HM(PIB(ICON[1].ptr));\n    hash_Unk80433380_0(&SNAP[0]);\n    for (k = 0; k < 2; k++) hash_Unk80433380_48(&SLOT[k]);",
          "    g::hash_Unk80433380(&m::lbSnap_80433380);\n    g::hash_Unk803BACC8(&m::lbSnap_803BACC8);\n    hm(pib(ICON[0].ptr)); hm(pib(ICON[1].ptr));\n    g::hash_Unk80433380_0(&SNAP[0]);\n    for k in 0..2usize { g::hash_Unk80433380_48(&SLOT[k]); }"),
    hash=("    for (k = 0; k < 0x40; k++) HM((u32) (unsigned char) TXT[k]);\n    for (k = 0; k < 0x1800 + 64; k++) HM(BANNER[k]);\n    for (k = 0; k < 0x400; k++) HM(ICNB[k]);",
          "    for k in 0..0x40usize { hm(TXT[k] as u8 as u32); }\n    for k in 0..0x1800 + 64usize { hm(BANNER[k] as u32); }\n    for k in 0..0x400usize { hm(ICNB[k] as u32); }"),
)
