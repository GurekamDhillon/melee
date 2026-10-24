"""World descriptions of the lb TU oracles (read by mk_driver.py). Every entry is a (C, Rust) pair drawing the same
numbers in the same order."""
CFG = {}
CFG["lbtime"] = dict(
    world=[("OSCalendarTime", "CT", 2)],
    argmap={"OSCalendarTime": [("&CT[0]", "&raw mut CT[0]"), ("&CT[1]", "&raw mut CT[1]")]},
    setup=("    __OSBusClock = 162000000u;", "    m::ext::__OSBusClock = 162000000;"),
    hints={"lbTime_8000AEE4:b": ("(int) fv() - (int) (fv() & 0x300)", "(fv() as i32).wrapping_sub((fv() & 0x300) as i32)")},
)

_FS = ("(float) ((int) (fv() % 81) - 40) / 4.0f", "((fv() % 81) as i32 - 40) as f32 / 4.0")
_PS = ("(float) (fv() % 64) / 8.0f", "(fv() % 64) as f32 / 8.0")
CFG["lb_00CE"] = dict(
    world=[],
    hints={"expf:arg8": _FS, "powf:arg0": _PS, "powf:arg1": ("(float) ((int) (fv() % 25) - 12) / 4.0f", "((fv() % 25) as i32 - 12) as f32 / 4.0"),
           "powi:base": ("(int) (fv() % 9) - 4", "(fv() % 9) as i32 - 4"), "powi:exponent": ("(int) (fv() % 40) - 3", "(fv() % 40) as i32 - 3")},
)

CFG["lbtrigf"] = dict(
    world=[],
    main_init=("    { union { u32 u; float f; } c; c.u = 0x7FFFFFFFu; MSL_TrigF_80400770[0] = c.f; c.u = 0x7F800000u; MSL_TrigF_80400774[0] = c.f; }", ""),
)

_MTF = ("    { int i, j, m; for (m = 0; m < 2; m++) for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) MT[m][i][j] = rf(); }",
        "    for m in 0..2 { for i in 0..3 { for j in 0..4 { MT[m][i][j] = rf(); } } }")
_MTH = ("    { int m, i, j; for (m = 0; m < 2; m++) for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) HM(B(MT[m][i][j])); }",
        "    for m in 0..2 { for i in 0..3 { for j in 0..4 { hm(MT[m][i][j].to_bits()); } } }")
CFG["lbvector"] = dict(
    world=[("Vec", "V", 4), ("HSD_CObj", "CO", 1), ("Quaternion", "Q", 1), ("Mtx", "MT", 2)],
    argmap={"Vec": [(f"&V[{i}]", f"&raw mut V[{i}]") for i in range(4)],
            "HSD_CObj": [("&CO[0]", "&raw mut CO[0]")],
            "Quaternion": [("&Q[0]", "&raw mut Q[0]")],
            "float[4]": [("MT[0]", "MT[0].as_mut_ptr()"), ("MT[1]", "MT[1].as_mut_ptr()")]},
    setup=_MTF, hash=_MTH,
    hints={"lbVector_Rotate:axis": ("(int) (fv() % 6)", "(fv() % 6) as i32"),
           "lbVector_WorldToScreen:d": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
    pre={"lbVector_WorldToScreen": (
        "V[0].x = (float) ((int) (rnd() % 2001) - 1000) * 0.5f; V[0].y = (float) ((int) (rnd() % 2001) - 1000) * 0.5f; V[0].z = (float) ((int) (rnd() % 2001) - 1000) * 0.5f;",
        "V[0].x = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5; V[0].y = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5; V[0].z = ((rnd() % 2001) as i32 - 1000) as f32 * 0.5;")},
)

import importlib, pathlib
for _p in sorted(pathlib.Path(__file__).parent.glob("tu_*.py")):
    _m = importlib.import_module(_p.stem)
    CFG[_p.stem[3:]] = _m.CFG
