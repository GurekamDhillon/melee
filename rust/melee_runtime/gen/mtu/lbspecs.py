"""Specs (gen_tu.py keys) and driver configs (mk_lb.py) for the lb/*.c ports in melee_runtime::tu."""
SPECS = {}
CFG = {}
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}


_GCSZ = None


def gc_sizeof_subs(*names):
    """spec `sub` entries replacing `sizeof(NAME)` by the console's size of the record (melee_types GC_BYTES)."""
    import re, pathlib
    global _GCSZ
    if _GCSZ is None:
        gen = (pathlib.Path(__file__).resolve().parents[4] / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")
        _GCSZ = {m.group(1): int(m.group(2), 16) for m in re.finditer(r"impl GcDecode for (\w+) \{\s*const GC_BYTES: u32 = (0x[0-9A-Fa-f]+);", gen)}
    return [(r"sizeof\s*\(\s*(?:struct\s+)?_?" + n + r"\s*\)", f"{_GCSZ[n]:#x}") for n in names]


def add(stem, src, cfg=None, **kw):
    SPECS[stem] = dict(src=src, common_ret=COMMON_RET, **kw)
    CFG[stem] = cfg or {}


GO4 = [("HSD_GObj", "GO", 4)]

add("lbbgflash", "src/melee/lb/lbbgflash.c", dict(pools=GO4 + [("GXColor", "COL", 4)],
    setup=("{ lbl_80433658.x32 = (u8) (1 + fv() % 60); lbl_80433658.x38 = (int) (fv() % 700) - 100; lbl_80433658.x3C = (int) (fv() % 40); lbl_80433658.x34 = (int) (fv() % 900) - 50; lbl_80433658.x30 = (fv() & 3) ? 0 : (u8) fv(); lbl_80433658.x33 = (fv() & 3) ? 0 : 1; }",
           "{ m::lbl_80433658.x32 = (1 + fv() % 60) as u8; m::lbl_80433658.x38 = (fv() % 700) as i32 - 100; m::lbl_80433658.x3C = (fv() % 40) as i32; m::lbl_80433658.x34 = (fv() % 900) as i32 - 50; m::lbl_80433658.x30 = if fv() & 3 != 0 { 0 } else { fv() as u8 }; m::lbl_80433658.x33 = if fv() & 3 != 0 { 0 } else { 1 }; }")),
    types=["HSD_GObj", "GXColor"], pools=["HSD_GObj"], own_fill=["lbl_80433658"],
    null_ret_ok=["HSD_CObjLoadDesc"],
    ret={"GObj_Create": ("pick_HSD_GObj()", "pick_HSD_GObj()"),
         "HSD_CObjGetCurrent": ("((fv() & 1) ? (HSD_CObj*) BUF : 0)", "if fv() & 1 != 0 { (&raw mut BUF) as *mut HSD_CObj } else { null_mut() }")})


def archive_stub(name, fill_c, fill_r):
    """lbArchive_80016DBC / lbArchive_LoadSymbols (filename, &slot, "symbol", 0): variadic in C, fixed arity in Rust.
    The stub stores a pointer into BUF in the slot (the archive's symbol address)."""
    return dict(
        c="HSD_Archive* " + name + "(const char* a0, void* a1, ...) { __builtin_va_list ap; const char* sym; int z; "
          "__builtin_va_start(ap, a1); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
          "T(@ID@); T(SH(a0)); T(PI(a1)); T(SH(sym)); T((u32) z); " + fill_c + " return 0; }",
        ext="fn " + name + "(a0: *const i8, a1: *mut c_void, a2: *const i8, a3: i32) -> *mut c_void;",
        hook="|a0, a1, a2, a3| { t(@ID@); t(sh(a0 as *const u8)); t(pi(a1)); t(sh(a2 as *const u8)); t(a3 as u32); " + fill_r + " core::ptr::null_mut() }")


BUFSLOT_C = "*(void**) a1 = BUF + 8 * (fv() % 8);"
BUFSLOT_R = "*(a1 as *mut *mut c_void) = (&raw mut BUF as *mut u8).add(8 * (fv() % 8) as usize) as *mut c_void;"

# ---------------------------------------------------------------------------------------- lbcardgame
_ctime_c = "a1->sec = (int) fv(); a1->min = (int) fv(); a1->hour = (int) fv(); a1->mday = (int) fv(); a1->mon = (int) fv(); a1->year = (int) (fv() % 3000) - 100; a1->wday = (int) fv(); a1->yday = (int) fv(); a1->msec = (int) fv(); a1->usec = (int) fv();"
_ctime_r = "(*a1).sec = fv() as i32; (*a1).min = fv() as i32; (*a1).hour = fv() as i32; (*a1).mday = fv() as i32; (*a1).mon = fv() as i32; (*a1).year = (fv() % 3000) as i32 - 100; (*a1).wday = fv() as i32; (*a1).yday = fv() as i32; (*a1).msec = fv() as i32; (*a1).usec = fv() as i32;"
add("lbcardgame", "src/melee/lb/lbcardgame.c", dict(
    pools=GO4 + [("SceneDesc", "SD", 2), ("DynamicModelDesc", "DM", 2), ("SceneCameraDesc", "SC", 2), ("HSD_JObj", "JO", 2)],
    c_decl="static u8* ICN[4]; static DynamicModelDesc* DMP[2]; static unsigned char NTB[7 * 0x1F2C + 16];",
    rs_extra="static mut ICN: [*mut u8; 4] = [null_mut(); 4]; static mut DMP: [*mut DynamicModelDesc; 2] = [null_mut(); 2]; static mut NTB: [u8; 7 * 0x1F2C + 16] = [0; 7 * 0x1F2C + 16];",
    pregs=("preg(ICN, sizeof ICN); preg(DMP, sizeof DMP); preg(NTB, sizeof NTB);",
           "preg(&raw const ICN, core::mem::size_of_val(&ICN)); preg(&raw const DMP, core::mem::size_of_val(&DMP)); preg(&raw const NTB, core::mem::size_of_val(&NTB));"),
    setup=("{ int q; for (q = 0; q < 4; q++) ICN[q] = (fv() & 7) ? (u8*) (BUF + 8 * q) : 0; state.icon_data = (op == OP_lbCardGame_LoadArchive && (fv() & 1)) ? 0 : ICN;"
           " DMP[0] = &DM[0]; DMP[1] = &DM[1]; SD[0].models = DMP; SD[0].cameras = SC; SD[1].models = DMP + 1; SD[1].cameras = SC + 1;"
           " state.scene_data = (fv() & 7) ? &SD[fv() & 1] : 0; state.jobj_translate_idx = 0; state.enable = (fv() & 7) != 0 || op == OP_lb_8001C8BC || op == OP_lbCardGame_SaveChanges; state.x10 = fv() & 1;"
           " OS_BusClock = (fv() & 1) ? 162000000u : 4u + fv() % 100000u; }",
           "{ for q in 0..4usize { ICN[q] = if fv() & 7 != 0 { (&raw mut BUF as *mut u8).add(8 * q) } else { null_mut() }; }"
           " m::state.icon_data = if op == OP_lbCardGame_LoadArchive && fv() & 1 != 0 { null_mut() } else { (&raw mut ICN) as *mut *mut c_void };"
           " DMP[0] = &raw mut DM[0]; DMP[1] = &raw mut DM[1]; SD[0].models = (&raw mut DMP) as *mut *mut DynamicModelDesc; SD[0].cameras = &raw mut SC[0];"
           " SD[1].models = (&raw mut DMP as *mut *mut DynamicModelDesc).add(1); SD[1].cameras = &raw mut SC[1];"
           " m::state.scene_data = if fv() & 7 != 0 { &raw mut SD[(fv() & 1) as usize] } else { null_mut() }; m::state.jobj_translate_idx = 0;"
           " m::state.enable = ((fv() & 7 != 0) || op == OP_lb_8001C8BC || op == OP_lbCardGame_SaveChanges) as i32; m::state.x10 = fv() & 1;"
           " m::ext::OS_BusClock = if fv() & 1 != 0 { 162000000u32 } else { 4u32 + fv() % 100000u32 }; }"),
    ),
    types=["HSD_GObj", "SceneDesc", "DynamicModelDesc", "SceneCameraDesc", "HSD_JObj"],
    pools=["HSD_GObj", "SceneDesc", "DynamicModelDesc", "SceneCameraDesc", "HSD_JObj"],
    own_fill=["state"], own_hash=["manifest", "lb_803BAB60", "jobj_translate"], own_preg=["filename"],
    skip_data=["result"],
    patch=[('#include "lbcardgame.h"\n', '#include "lbcardgame.h"\n#include <dolphin/os.h>\n#undef __OSBusClock\nextern u32 OS_BusClock;\n#define __OSBusClock OS_BusClock\n')],
    null_ret_ok=["HSD_CObjLoadDesc", "HSD_GObj_SetupProc"],
    custom_stubs={"lbArchive_80016DBC": archive_stub("lbArchive_80016DBC", BUFSLOT_C, BUFSLOT_R)},
    ret={"GObj_Create": ("pick_HSD_GObj()", "pick_HSD_GObj()"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj()", "pick_HSD_JObj()"),
         "gmMainLib_GetCardData": ("(struct GmCardData*) BUF", "(&raw mut BUF) as *mut GmCardData"),
         "gmMainLib_GetNameTagDataBanks": ("(struct NameTagDataBank*) NTB", "(&raw mut NTB) as *mut NameTagDataBank"),
         "gm_GetPowerTime": ("(s32*) &IA[1]", "(&raw mut IA[1])")},
    effects={"OSTicksToCalendarTime": (_ctime_c, _ctime_r)})


import pathlib
for _p in sorted(pathlib.Path(__file__).parent.glob("lbspecs_*.py")):
    __import__(_p.stem)
