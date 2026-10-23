"""Per-TU oracle specs for gen_tu.py / run_tu.py of the melee_items::tu item-kind ports (shard 657-664).
Keys as in melee_world/gen/mtu/specs.py: src, types, ret {callee: (C expr, Rust expr)}, effects {callee: (C stmts,
Rust stmts)}, custom_stubs, skip_stubs, null_ret_ok. The world (GO / IT / JO ...) is declared by mk_driver.py before
the generated stubs, so the overrides can name it (`GO[2]`; Rust `&raw mut crate::GO[2]`)."""
K = "src/melee/it/kinds/"
_T = ["HSD_GObj", "Item", "HSD_JObj", "ItemAttr", "ItemCommonData"]

_P = ("(int) (fv() & 1)", "(fv() & 1) as i32")
_GO2 = ("((fv() & 1) ? &GO[2] : 0)", "if fv() & 1 != 0 { &raw mut crate::GO[2] } else { core::ptr::null_mut() }")
_GO1 = ("((fv() & 1) ? &GO[1] : 0)", "if fv() & 1 != 0 { &raw mut crate::GO[1] } else { core::ptr::null_mut() }")

COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
    "Item_80268B18": _GO2,
    "Item_80268B5C": _GO2,
    "it_8026F6BC": _GO1,
    "lbVector_Diff": ("a2", "a2"),
    "Stage_8022519C": ("((fv() % 3) == 0 ? 14 : (int) (fv() % 20))", "if fv() % 3 == 0 { 14 } else { (fv() % 20) as _ }"),
}
for _n in ("it_80272C6C", "it_8026F8B4", "it_8026DA08", "it_802763E0", "it_80276308", "it_8026D8A4", "it_8027A09C",
           "it_8027A118", "it_80273030", "it_802751D8", "itColl_BounceOffShield", "it_80274658", "it_80273130",
           "it_8026B894", "it_8026EA20", "it_8026EA9C", "mpGetSpeed", "it_8026F3D4", "it_8026DFB0"):
    COMMON_RET[_n] = _P

_V = lambda a: (f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); }}",
                f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); }}")
COMMON_EFF = {
    "lb_8000B1CC": _V("a2"),
    "lbVector_Diff": _V("a2"),
    "lbVector_Normalize": _V("a0"),
    "lbVector_Mirror": _V("a0"),
    "grCorneria_801DDCF0": _V("a0"),
    "it_80276408": _V("a2"),
    "mpGetSpeed": (
        "if (a1) { a1->x = rw(); } if (a2) { a2->y = rw(); }",
        "if !a1.is_null() { (*a1).x = rw(); } if !a2.is_null() { (*a2).y = rw(); }"),
    "it_8026EA9C": (
        "if (a3) { a3->x = rw(); a3->y = rw(); a3->z = rw(); } if (a4) { *a4 = (int) (fv() % 8); } if (a5) { *a5 = fv(); }",
        "if !a3.is_null() { (*a3).x = rw(); (*a3).y = rw(); (*a3).z = rw(); } if !a4.is_null() { *a4 = (fv() % 8) as i32; } "
        "if !a5.is_null() { *a5 = fv(); }"),
    "it_8026EA20": (
        "if (a3) { a3->x = rw(); a3->y = rw(); a3->z = rw(); } if (a4) { *a4 = (int) (fv() % 8); } if (a5) { *a5 = fv(); }",
        "if !a3.is_null() { (*a3).x = rw(); (*a3).y = rw(); (*a3).z = rw(); } if !a4.is_null() { *a4 = (fv() % 8) as i32; } "
        "if !a5.is_null() { *a5 = fv(); }"),
}

SPECS = {}
for _n in ("itarwinglaser", "itbat", "itbombhei", "itbox", "itcapsule", "itcerebi", "itchicorita", "itclimbersblizzard",
           "itwhitebea", "itwstar", "ityaku", "ityoshiegglay", "ityoshieggthrow", "ityoshistar", "ityoshitongue",
           "itzeldadinfire"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi"])

SPECS["itarwinglaser"]["skip_stubs"] = ["sinf", "cosf"]
SPECS["itclimbersblizzard"]["skip_stubs"] = ["sinf", "cosf"]
_EFS3 = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); TVEC((Vec3*) a2); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); tvec(a2); core::ptr::null_mut() }")
_EFS4 = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; float* a3; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); a3 = __builtin_va_arg(ap, float*); __builtin_va_end(ap); "
      "T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); T(PI(a3)); T(B(*a3)); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32) -> *mut c_void;",
    hook="|a0, a1, a2, a3| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); t(pi(a3)); t((*a3).to_bits()); core::ptr::null_mut() }")
SPECS["itarwinglaser"]["custom_stubs"] = {"efSync_Spawn": _EFS3}
SPECS["itbox"]["custom_stubs"] = {"efSync_Spawn": _EFS3}
SPECS["itcerebi"]["custom_stubs"] = {"efSync_Spawn": _EFS4}

# ---------------------------------------------------------------- shard 809-816
_EFA = dict(
    c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { __builtin_va_list ap; float* a5 = 0; "
      "if (a2 == 4) { __builtin_va_start(ap, a4); a5 = __builtin_va_arg(ap, float*); __builtin_va_end(ap); } "
      "T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); if (a2 == 4) { T(PI(a5)); T(B(*a5)); } }",
    ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut f32);",
    hook="|a0, a1, a2, a3, a4, a5| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); "
         "if a2 == 4 { t(pi(a5)); t((*a5).to_bits()); } }")
_EFSJ = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); core::ptr::null_mut() }")
for _n in ("itwstar", "ityoshiegglay", "ityoshieggthrow"):
    SPECS[_n]["custom_stubs"] = {"efAsync_Spawn": _EFA}
SPECS["itzeldadinfire"]["custom_stubs"] = {"efSync_Spawn": _EFSJ}
SPECS["itzeldadinfire"]["skip_stubs"] = ["sinf", "cosf"]
SPECS["itwhitebea"]["skip_stubs"] = ["__frsqrte"]
_VOUT = lambda a: _V(a)
SPECS["itwhitebea"]["ret"].update({
    "it_8026B684": ("((fv() & 1) ? 1.0f : -1.0f)", "if fv() & 1 != 0 { 1.0 } else { -1.0 }"),
    "it_8027CBFC": ("(float) ((int) (fv() % 200) - 50)", "((fv() % 200) as i32 - 50) as f32"),
    "mpLib_80056A1C": ("&SFXA[fv() % 4]", "&raw mut crate::SFXA[(fv() % 4) as usize]"),
    "it_802750E8": _P, "it_8027B798": _P, "it_8027C794": _P, "it_8027C79C": _P, "it_8027C824": _P,
})
SPECS["itwhitebea"]["effects"].update({
    "it_8028ECF0": _V("a1"), "it_8027B798": _V("a1"), "it_8027BA54": _V("a1"), "it_8027C8D0": _V("a0"),
})
SPECS["itwhitebea"]["null_ret_ok"] += ["it_8028ECF0"]
SPECS["itwstar"]["ret"].update({
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
})
SPECS["itwstar"]["null_ret_ok"] += ["it_80272C90", "it_80272CC0"]
SPECS["itzeldadinfire"]["ret"].update({
    "ftZd_SpecialLw_8013B540": _P, "ftZd_SpecialLw_8013B574": _P,
})
SPECS["itzeldadinfire"]["effects"].update({
    "ftLib_800865D8": ("if (a1) { *a1 = rw(); } if (a2) { *a2 = rw(); }",
                       "if !a1.is_null() { *a1 = rw(); } if !a2.is_null() { *a2 = rw(); }"),
})
SPECS["itzeldadinfire"]["null_ret_ok"] += ["it_802C4580"]
SPECS["ityoshitongue"]["null_ret_ok"] += ["ftLib_80086630"]
SPECS["ityoshitongue"]["ret"]["ftLib_80086630"] = ("((fv() & 1) ? &JO[3] : 0)", "if fv() & 1 != 0 { &raw mut crate::JO[3] } else { core::ptr::null_mut() }")
for _n in ("ityoshistar", "ityoshieggthrow", "ityoshiegglay", "itwstar", "ityaku", "itzeldadinfire"):
    SPECS[_n]["ret"]["it_80272C6C"] = _P
