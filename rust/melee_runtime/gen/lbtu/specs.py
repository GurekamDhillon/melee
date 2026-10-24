"""Per-TU oracle specs for gen_tu.py / run_tu.py of the melee_runtime::tu lb ports (shard 841-848).
Keys: src, types, ret {callee: (C expr, Rust expr)}, effects {callee: (C stmts, Rust stmts)}, custom_stubs,
skip_stubs, skip_data, null_ret_ok, patch."""
L = "src/melee/lb/"
SPECS = {}
SPECS["lbtime"] = dict(
    src=L + "lbtime.c", types=["OSCalendarTime"],
    # TARGET_PC maps __OSBusClock to guest memory (*(u32*) 0x800000F8): the oracle makes it a variable
    patch=[('#include "lbtime.h"', '#include "lbtime.h"' + chr(10) + '#undef __OSBusClock' + chr(10) + 'extern unsigned int __OSBusClock;')],
    ret={"OSGetTime": ("(long long) ((int) (fv() % 5) - 1) * 40500000LL * (long long) (fv() % 3000000) + (long long) fv()",
                       "((fv() % 5) as i32 as i64 - 1) * 40500000i64 * (fv() % 3000000) as i64 + fv() as i64")},
    effects={"OSTicksToCalendarTime": (
        "if (a1) { a1->sec = fv(); a1->min = fv(); a1->hour = fv(); a1->mday = fv(); a1->mon = fv(); a1->year = fv(); }",
        "if !a1.is_null() { (*a1).sec = fv() as i32; (*a1).min = fv() as i32; (*a1).hour = fv() as i32; (*a1).mday = fv() as i32; (*a1).mon = fv() as i32; (*a1).year = fv() as i32; }")},
)

SPECS["lb_00CE"] = dict(src=L + "lb_00CE.c", types=[], skip_stubs=["__frsqrte"],
                        ext_default={"atanf": "crate::lbtrigf::atanf"})

SPECS["lbtrigf"] = dict(src=L + "lbtrigf.c", types=[], skip_stubs=["__frsqrte"],
                        data_size={"MSL_TrigF_80400770": 1, "MSL_TrigF_80400774": 1})

_VW = lambda a: (f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); }}",
                 f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); }}")
_MW = lambda a, n: (f"{{ int i, j; for (i = 0; i < {n}; i++) for (j = 0; j < 4; j++) {a}[i][j] = rf(); }}",
                    f"for i in 0..{n} {{ for j in 0..4 {{ (*{a}.add(i))[j] = rf(); }} }}")
SPECS["lbvector"] = dict(
    src=L + "lbvector.c", types=["Vec", "HSD_CObj", "Quaternion"], skip_stubs=["__frsqrte"],
    ext_default={"PSVECCrossProduct": "psvec_cross_default", "acosf": "crate::lbtrigf::acosf",
                 "asinf": "crate::lbtrigf::asinf", "atan2f": "crate::lbtrigf::atan2f"},
    ret={"HSD_CObjGetProjectionType": ("((fv() % 5) == 4 ? 2 : ((fv() & 1) ? 1 : 3))", "if fv() % 5 == 4 { 2 } else if fv() & 1 != 0 { 1 } else { 3 }"),
         "HSD_CObjGetViewingMtxPtr": ("MT[1]", "crate::MT[1].as_mut_ptr()"),
         "PSVECCrossProduct": ("0", "()"),},
    effects={"PSVECCrossProduct": _VW("a2"),
             "HSD_CObjGetEyePosition": _VW("a1"), "HSD_CObjGetUpVector": _VW("a1"), "HSD_CObjGetInterest": _VW("a1"),
             "C_MTXLookAt": _MW("a0", 3), "MTXPerspective": _MW("a0", 4), "MTXOrtho": _MW("a0", 4),
             "GXProject": (
                 "{ int i; for (i = 0; i < 7; i++) T(B(a4[i])); for (i = 0; i < 6; i++) T(B(a5[i])); for (i = 0; i < 12; i++) T(B(a3[i / 4][i % 4])); } *a6 = rf(); *a7 = rf(); *a8 = rf();",
                 "for i in 0..7 { t((*a4.add(i)).to_bits()); } for i in 0..6 { t((*a5.add(i)).to_bits()); } for i in 0..12 { t((*a3.add(i / 4))[i % 4].to_bits()); } *a6 = rf(); *a7 = rf(); *a8 = rf();"),
             },
)

# per-TU modules gen/lbtu/tu_<stem>.py (SPEC, CFG)
import importlib, pathlib
for _p in sorted(pathlib.Path(__file__).parent.glob("tu_*.py")):
    _m = importlib.import_module(_p.stem)
    SPECS[_p.stem[3:]] = _m.SPEC
