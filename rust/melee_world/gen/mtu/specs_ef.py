"""Spec of the combined ef oracle (eflib.c + efasync.c + efdata.c compiled as one C TU, ported as
tu::efn::{eflib,efasync,efdata}); driven by gen/mtu/mk_ef.py."""
import os, re
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SPECS = {}
DBG_PATCH = []
if os.environ.get("MTU_PATCHFILE"):
    for blk in open(os.environ["MTU_PATCHFILE"], encoding="utf-8").read().split("#####\n"):
        if blk.strip():
            o, n = blk.split("=====\n")
            DBG_PATCH.append((o, n))

VEC_C = "if (%s) { %s->x = RNN(); %s->y = RNN(); %s->z = RNN(); }"
VEC_R = "if !%s.is_null() { (*%s).x = super::rnn(); (*%s).y = super::rnn(); (*%s).z = super::rnn(); }"


def vec_out(a):
    return (VEC_C % (a, a, a, a), VEC_R % (a, a, a, a))


def ptr_or_null(mask, arr, mod):
    """`((fv() & mask) ? &ARR[fv() % mod] : 0)` as C and Rust"""
    return (f"((fv() & {mask}) ? &{arr}[fv() % {mod}] : 0)",
            f"if fv() & {mask} != 0 {{ &raw mut super::{arr}[(fv() % {mod}) as usize] }} else {{ core::ptr::null_mut() }}")


# the macro that gives the variadic efSync_Spawn call sites a count (a stub cannot know how many pointers follow)
SPAWN_MACRO = (
    "#define EF_NARGS_(a, b, c, d, e, f, g, h, N, ...) N\n"
    "#define EF_NARGS(...) EF_NARGS_(__VA_ARGS__, 8, 7, 6, 5, 4, 3, 2, 1, 0)\n"
    "extern void* efSync_SpawnN(int n, ...);\n"
    "#define efSync_Spawn(...) efSync_SpawnN(EF_NARGS(__VA_ARGS__), __VA_ARGS__)\n")


def own_paths():
    """name -> Rust module of every function the ports define (for the callback table)"""
    out = {}
    for mod in ("eflib", "efasync"):
        txt = (ROOT / f"rust/melee_world/src/tu/efn/{mod}.rs").read_text(encoding="utf-8")
        for m in re.finditer(r'^pub unsafe (?:extern "C" )?fn (\w+)\(', txt, re.M):
            out.setdefault(m.group(1), mod)
    return out


_OWN = own_paths()


def freg_path(kind, name):
    if kind == "cb":
        txt = (ROOT / "rust/melee_world/src/tu/efn/eflib.rs").read_text(encoding="utf-8")
        if re.search(r"pub unsafe extern \"C\" fn " + re.escape(name) + r"\(", txt):
            return f"m::efn::eflib::ext::cb::{name}"
        return "0xDEAD0000usize"  # a stub the scan took for address-taken (a macro argument); never stored
    mod = _OWN.get(name)
    if mod is None:
        raise SystemExit(f"ef oracle: {name} is not defined in a port module")
    return f"m::efn::{mod}::{name}"


SPECS["ef"] = dict(
    src="src/melee/ef/eflib.c",
    srcs=["src/melee/ef/efdata.c", "src/melee/ef/eflib.c", "src/melee/ef/efasync.c"],
    mods=["efn/eflib", "efn/efasync"],
    args=["-iquote", "src/melee/ef"],
    patch=[("HSD_ObjAllocData efAsync_AllocData;\n", "HSD_ObjAllocData efAsync_AllocData;\n" + SPAWN_MACRO)],
    patch_all=DBG_PATCH,
    freg_path=freg_path,
    types=["EF_Effect", "EF_QueuedEffect", "EF_DAT_Entry", "EF_ParamEntry", "HSD_GObj", "HSD_JObj", "HSD_Generator",
           "HSD_psAppSRT", "HSD_Particle", "HSD_DObj", "HSD_MObj", "HSD_TObj", "HSD_TObjTev", "HSD_GObjProc",
           "EF_EffectDesc"],
    data_size={"ftData_UnkBytePerCharacter": 128},
    skip_data=["guess", "result"],
    skip_stubs=["__HI", "__LO", "__frsqrte", "cosf", "sinf", "memset", "memzero", "__va_arg", "__builtin_va_end",
                "__builtin_va_start"],
    ret={
        "GObj_Create": ("&GO[fv() % 6]", "&raw mut super::GO[(fv() % 6) as usize]"),
        "HSD_ArchiveGetPublicAddress": (
            "((SH(a1) == SH(\"effBehaviorTable\")) ? (void*) &BHV : (void*) &DE[fv() & 1])",
            "if sh(a1 as *const u8) == sh(b\"effBehaviorTable\\0\".as_ptr()) { &raw mut super::BHV as *mut c_void } "
            "else { &raw mut super::DE[(fv() & 1) as usize] as *mut c_void }"),
        "HSD_GObj_SetupProc": ptr_or_null(7, "PROC", 2),
        "HSD_JObjGetDObj": ptr_or_null(7, "DO", 4),
        "HSD_JObjLoadJoint": ptr_or_null(15, "JO", 16),
        "HSD_MObjGetTObj": ("&TO[fv() & 7]", "&raw mut super::TO[(fv() & 7) as usize]"),
        "HSD_ObjAlloc": (
            "((a0 == &efLib_AllocData) ? ((EFN < 10) ? (void*) &EFF[EFN++] : (void*) 0) : ((QN < 8) ? (void*) &QE[QN++] : (void*) 0))",
            "if a0 as usize == (&raw mut m::efn::efdata::efLib_AllocData) as usize { if super::EFN < 10 { super::EFN += 1; &raw mut super::EFF[super::EFN - 1] as *mut c_void } else { core::ptr::null_mut() } } "
            "else if super::QN < 8 { super::QN += 1; &raw mut super::QE[super::QN - 1] as *mut c_void } else { core::ptr::null_mut() }"),
        "Mex_EffectString": ("((fv() & 3) ? (const char*) STRS[fv() % 3] : 0)",
                             "if fv() & 3 != 0 { super::STRS[(fv() % 3) as usize].as_ptr() as *const i8 } else { core::ptr::null() }"),
        "hsd_8039EFAC": ptr_or_null(7, "GEN", 3),
        "hsd_8039F05C": ptr_or_null(7, "GEN", 3),
        "hsd_8039F6CC": ptr_or_null(7, "GEN", 3),
        "psAddGeneratorAppSRT_begin": (
            "(a0->appsrt = ((fv() & 7) ? &APS[fv() % 3] : 0))",
            "{ let r = if fv() & 7 != 0 { &raw mut super::APS[(fv() % 3) as usize] } else { core::ptr::null_mut() }; (*a0).appsrt = r; r }"),
        "psAddParticleAppSRT_begin": (
            "(a0->appsrt = ((fv() & 7) ? &APS[fv() % 3] : 0))",
            "{ let r = if fv() & 7 != 0 { &raw mut super::APS[(fv() % 3) as usize] } else { core::ptr::null_mut() }; (*a0).appsrt = r; r }"),
        "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
        "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
        "Mex_EffectCount": ("(int) (rnd() % 70)", "(rnd() % 70) as i32"),
    },
    effects={
        "lb_8000B1CC": vec_out("a2"),
        "HSD_MtxGetScale": vec_out("a1"),
        "HSD_GObjObject_80390A70": ("a0->hsd_obj = a2; a0->obj_kind = a1;", "(*a0).hsd_obj = a2; (*a0).obj_kind = a1;"),
        "GObj_InitUserData": ("a0->user_data = a3; a0->user_data_remove_func = a2;", "(*a0).user_data = a3; (*a0).user_data_remove_func = a2;"),
        "psRemoveParticleAppSRT": ("a0->appsrt = 0;", "(*a0).appsrt = core::ptr::null_mut();"),
    },
    null_ret_ok=["GObj_Create", "HSD_GObj_SetupProc", "HSD_JObjGetDObj", "HSD_JObjLoadJoint", "HSD_ObjAlloc",
                 "Mex_EffectString", "hsd_8039EFAC", "hsd_8039F05C", "hsd_8039F6CC", "psAddGeneratorAppSRT_begin",
                 "psAddParticleAppSRT_begin", "HSD_ArchiveGetPublicAddress", "HSD_MObjGetTObj"],
    custom_stubs={
        "efSync_SpawnN": dict(
            rust_name="efSync_Spawn",
            c="void* efSync_SpawnN(int n, ...) { __builtin_va_list ap; int i; s32 gfx; HSD_GObj* g; T(@ID@); __builtin_va_start(ap, n); gfx = __builtin_va_arg(ap, s32); g = __builtin_va_arg(ap, HSD_GObj*); "
              "T((u32) gfx); T(PI(g)); T((u32) n); for (i = 2; i < n; i++) { void* p = __builtin_va_arg(ap, void*); T(PI(p)); T(B(*(float*) p)); } __builtin_va_end(ap); return (void*) 0; }",
            ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;",
            hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t((a2.len() + 2) as u32); for p in a2 { t(pi(*p as *const c_void)); t((*(*p as *const f32)).to_bits()); } core::ptr::null_mut() }"),
        "lbArchive_80017040": dict(
            c="int lbArchive_80017040(HSD_Archive** a0, const char* a1, void* a2, ...) { __builtin_va_list ap; const char* sym; int z; __builtin_va_start(ap, a2); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
              "T(@ID@); T(PI(a0)); T(SH(a1)); T(PI(a2)); T(SH(sym)); T((u32) z); if (a0) { *a0 = (fv() & 1) ? (HSD_Archive*) &ARC[0] : 0; } *(EF_DAT_Entry**) a2 = &DE[fv() & 1]; return (int) (fv() & 1); }",
            ext="fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: &[*const c_void]) -> i32;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(pi(a0)); t(sh(a1 as *const u8)); t(pi(a2)); t(sh(a3[0] as *const u8)); t(a3[1] as usize as u32); "
                 "if !a0.is_null() { *a0 = if fv() & 1 != 0 { &raw mut super::ARC as *mut HSD_Archive } else { core::ptr::null_mut() }; } "
                 "*(a2 as *mut *mut EF_DAT_Entry) = &raw mut super::DE[(fv() & 1) as usize]; (fv() & 1) as i32 }"),
        "sprintf": dict(
            c="int sprintf(char* a0, const char* a1, ...) { (void) a0; (void) a1; return 0; }",
            ext="fn sprintf(a0: *mut i8, a1: *const i8, a2: &[*const c_void]) -> i32;",
            hook="|_a0, _a1, _a2| 0"),
    },
)
