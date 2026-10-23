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
           "itmato", "itmball", "itmetalb", "itmetamon", "itmew", "itmewtwodisable", "itmewtwoshadowball", "itmsbomb"):
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

SPECS["itmetalb"]["ret"]["it_8027236C"] = ("((fv() & 3) ? &GO[2] : 0)", "if fv() & 3 != 0 { &raw mut crate::GO[2] } else { core::ptr::null_mut() }")
SPECS["itmetalb"]["null_ret_ok"].append("it_8027236C")

SPECS["itmball"]["custom_stubs"] = {"efAsync_Spawn": dict(
    c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); }",
    ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);",
    hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); }")}

SPECS["itmsbomb"]["ret"]["it_80276308"] = ("(int) ((fv() % 4) * 4)", "((fv() % 4) * 4) as i32")
SPECS["itmsbomb"]["ret"]["it_802763B8"] = ("(int) (fv() % 3)", "(fv() % 3) as i32")
SPECS["itmsbomb"]["ret"]["it_802763E0"] = ("(int) (fv() % 3)", "(fv() % 3) as i32")
SPECS["itmsbomb"]["ret"]["it_8026D604"] = _P
SPECS["itmato"]["ret"]["ScriptGame_TargetDestroyed"] = _P

_SB = SPECS["itmewtwoshadowball"]
_SB["skip_stubs"] = ["sinf", "cosf"]
_SB["custom_stubs"] = {"efSync_Spawn": _EFS3}
_SB["ret"]["ftLib_80086630"] = ("&JO[5]", "&raw mut crate::JO[5]")
_SB["null_ret_ok"].append("ftLib_80086630")
_RARE = ("(int) ((fv() % 5) == 0)", "((fv() % 5) == 0) as i32")
for _n in ("ftMt_SpecialN_CheckShadowBallRemove", "ftMt_SpecialN_CheckShadowBallCancel", "ftKb_SpecialNMt_80106F9C", "ftKb_SpecialNMt_80106FEC"):
    _SB["ret"][_n] = _RARE
for _n in ("ftMt_SpecialN_GetChargeLevel", "ftKb_SpecialNMt_80106F44"):
    _SB["ret"][_n] = ("(int) (fv() % 4) - 1", "(fv() % 4) as i32 - 1")
    _SB["effects"][_n] = (
        "if (a1) { *a1 = (int) (fv() % 6); } if (a2) { *a2 = (int) (fv() % 5) + 1; }",
        "if !a1.is_null() { *a1 = (fv() % 6) as i32; } if !a2.is_null() { *a2 = (fv() % 5) as i32 + 1; }")
_SB["effects"]["HSD_MtxGetRotation"] = _V("a1")
_SB["ret"]["atan2f"] = ("(float) ((int) (fv() % 629) - 314) / 100.0f", "((fv() % 629) as i32 - 314) as f32 / 100.0")
