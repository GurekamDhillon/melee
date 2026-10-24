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
           "it_8026B894", "it_8026EA20", "it_8026EA9C", "mpGetSpeed", "it_8026F3D4", "it_8026DFB0",
           "ftNs_AttackS4_CheckNessBatRemove", "ftCo_800BF228", "ftNs_SpecialN_CheckSpecialNHold",
           "ftKb_SpecialNNs_800FEC78", "ftNs_SpecialHi_CheckSpecialHiHold", "it_8026E058", "mpColl_800471F8"):
    COMMON_RET[_n] = _P

_V = lambda a: (f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); }}",
                f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); }}")
COMMON_RET["it_802AF940"] = _GO2
COMMON_RET["it_802AC43C"] = _GO2
COMMON_RET["lbVector_CrossprodNormalized"] = ("a2", "a2")
COMMON_RET["lbVector_Angle"] = ("(float) (rnd() % 400) / 100.0f", "(rnd() % 400) as f32 / 100.0")
COMMON_RET["Stage_80225194"] = ("((fv() % 3) == 0 ? 0x54 : (int) (fv() % 20))", "if fv() % 3 == 0 { 0x54 } else { (fv() % 20) as _ }")
COMMON_RET["ftLib_80086A0C"] = ("(float) (rnd() % 5) / 2.0f", "(rnd() % 5) as f32 / 2.0")
COMMON_RET["ftLib_GetMotionId"] = ("(int) (0x150 + rnd() % 16)", "(0x150 + rnd() % 16) as i32")
COMMON_RET["it_802A3C98"] = ("(float) (rnd() % 60) / 10.0f", "(rnd() % 60) as f32 / 10.0")
COMMON_EFF = {
    "lb_8000B1CC": _V("a2"),
    "ftLib_80086644": _V("a1"),
    "it_8026BB68": _V("a1"),
    "it_802AB3F0": _V("a1"),
    "lbVector_CrossprodNormalized": _V("a2"),
    "it_802A3C98": _V("a2"),
    "it_802AB468": ("if (a1) { *a1 = rw(); }", "if !a1.is_null() { *a1 = rw(); }"),
    "ftLib_800865D8": ("if (a1) { *a1 = rg(); } if (a2) { *a2 = rg(); }",
                       "if !a1.is_null() { *a1 = rg(); } if !a2.is_null() { *a2 = rg(); }"),
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
           "itluigifireball", "itmaril", "itmariocape", "itmariofireball", "itmarumine", "itmasterhandbullet",
           "itmasterhandlaser", "itmatadogas",
           "itnessbat", "itnesspkfire", "itnesspkfirepillar", "itnesspkflash", "itnesspkflashexplode",
           "itnesspkthunderball", "itnesspkthundertrail", "itnessyoyo",
           "itmato", "itmball", "itmetalb", "itmetamon", "itmew", "itmewtwodisable", "itmewtwoshadowball", "itmsbomb",
           "itscball", "itseakchain", "itseakneedleheld", "itseakneedlethrown", "itseakvanish", "itsonans", "itspycloak", "itsscope",
           "itsscopebeam", "itstar", "itstarrod", "itstarrodstar", "itsuikun", "itsword", "ittaru", "ittarucann"):
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

# ---------------------------------------------------------------------------- shard 769-776
for _n in ("itpeachparasol", "itpeachtoad", "itpeachtoadspore", "itpeachturnip", "itpikachuthunder",
           "itpikachutjoltair", "itpikachutjoltground", "itpippi"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi"])

_GOX = ("(fv() % 3 == 0 ? &GO[0] : ((fv() & 1) ? &GO[1] : 0))",
        "if fv() % 3 == 0 { &raw mut crate::GO[0] } else if fv() & 1 != 0 { &raw mut crate::GO[1] } else { core::ptr::null_mut() }")
_N3 = lambda a: (f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); }}",
                 f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); }}")
for _n in ("it_8026DA70", "it_8026DAA8", "it_8026E9A4", "ftPe_8011D518", "ftPe_SpecialHi_NotActive", "ftPe_SpecialN_IsActive",
           "ftKb_SpecialNPe_8010C4AC", "ftPk_SpecialLw_CheckProperty"):
    for _s in ("itpeachparasol", "itpeachtoad", "itpeachturnip", "itpikachuthunder", "itpikachutjoltground"):
        SPECS[_s]["ret"][_n] = _P
for _s in ("itpikachutjoltair", "itpikachutjoltground"):
    SPECS[_s]["ret"]["it_802B3368"] = _GOX
    SPECS[_s]["ret"]["it_802B3EFC"] = _GOX
    SPECS[_s]["ret"]["it_802B4224"] = _GO2
    SPECS[_s]["ret"]["it_8026E9A4"] = _P
    SPECS[_s]["effects"]["it_8026E9A4"] = _N3("a3")
    SPECS[_s]["effects"]["it_802B3F20"] = _V("a1")
    SPECS[_s]["skip_stubs"] = ["sinf", "cosf"]
    SPECS[_s]["null_ret_ok"] += ["it_802B3368", "it_802B3EFC", "it_802B4224"]
SPECS["itpeachtoadspore"]["skip_stubs"] = ["sinf", "cosf"]
# ---- shard 777-784: itporygon2 itrabbitc itraikou itrshell itsamusbomb itsamuschargeshot itsamusgrapple itsamusmissile
for _n in ("itporygon2", "itrabbitc", "itraikou", "itrshell", "itsamusbomb", "itsamuschargeshot", "itsamusmissile"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi"])

SPECS["itrabbitc"]["ret"]["it_80272CC0"] = ("(&JO[3 + (fv() % 3)])", "&raw mut crate::JO[3 + (fv() % 3) as usize]")
SPECS["itraikou"]["custom_stubs"] = {"efSync_Spawn": _EFS4}

_EFSJ = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); core::ptr::null_mut() }")
_EFS3J = _EFSJ
SPECS["itpeachtoadspore"]["custom_stubs"] = {"efSync_Spawn": _EFSJ}
SPECS["itpikachutjoltground"]["custom_stubs"] = {"efSync_Spawn": _EFSJ}
SPECS["itpeachturnip"]["ret"]["it_8026DAA8"] = ("(int) (fv() & 15)", "(fv() & 15) as i32")
SPECS["itpikachutjoltground"]["effects"]["it_8026BB68"] = _V("a1")

# shard 721-728
for _n in ("itkyasarinegg", "itleadead", "itlgun", "itlgunbeam", "itlgunray", "itlikelike", "itlinkarrow", "itlinkbomb"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi"])
    SPECS[_n]["skip_stubs"] = ["sinf", "cosf"]

SPECS["itkyasarinegg"]["custom_stubs"] = {"efSync_Spawn": _EFS3}
SPECS["itkyasarinegg"]["ret"]["it_8027B5B0"] = _GO2
SPECS["itkyasarinegg"]["null_ret_ok"].append("it_8027B5B0")

_EFS5 = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; float* a3; float* a4; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); a3 = __builtin_va_arg(ap, float*); a4 = __builtin_va_arg(ap, float*); __builtin_va_end(ap); "
      "T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); T(PI(a3)); T(B(*a3)); T(PI(a4)); T(B(*a4)); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut f32, a4: *mut f32) -> *mut c_void;",
    hook="|a0, a1, a2, a3, a4| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); t(pi(a3)); t((*a3).to_bits()); t(pi(a4)); t((*a4).to_bits()); core::ptr::null_mut() }")
SPECS["itlgunbeam"]["custom_stubs"] = {"efSync_Spawn": _EFS5}

_L = SPECS["itleadead"]
_L["ret"].update({
    "ftLib_8008627C": ("((fv() & 3) ? &GO[1] : 0)", "if fv() & 3 != 0 { &raw mut crate::GO[1] } else { core::ptr::null_mut() }"),
    "it_80272CC0": ("&JO[3 + fv() % 3]", "&raw mut crate::JO[3 + (fv() % 3) as usize]"),
    "it_8027B5B0": _GO2,
    "it_8027CBFC": ("(float) (rnd() % 20)", "(rnd() % 20) as f32"),
})
_L["null_ret_ok"] += ["ftLib_8008627C", "it_8027B5B0", "ftLib_80086630"]
_L["effects"].update({
    "ftLib_80086644": _V("a1"),
    "mpLib_80056C54": (
        "if (a2) { *a2 = (int) (fv() % 4); } if (a3) { a3->x = rw(); a3->y = rw(); a3->z = rw(); } if (a4) { *a4 = fv(); } "
        "if (a5) { a5->x = rw(); a5->y = rw(); a5->z = rw(); }",
        "if !a2.is_null() { *a2 = (fv() % 4) as i32; } if !a3.is_null() { (*a3).x = rw(); (*a3).y = rw(); (*a3).z = rw(); } "
        "if !a4.is_null() { *a4 = fv(); } if !a5.is_null() { (*a5).x = rw(); (*a5).y = rw(); (*a5).z = rw(); }"),
})

_L["deep_ecb"] = True

_K = SPECS["itlikelike"]
_K["ret"].update({
    "ftLib_8008627C": ("((fv() & 3) ? &GO[1] : 0)", "if fv() & 3 != 0 { &raw mut crate::GO[1] } else { core::ptr::null_mut() }"),
    "it_80272CC0": ("&JO[3 + fv() % 3]", "&raw mut crate::JO[3 + (fv() % 3) as usize]"),
    "it_8027B5B0": _GO2,
    "it_8027CBFC": ("(float) (rnd() % 20)", "(rnd() % 20) as f32"),
    "mpColl_8004C750": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
})
_K["null_ret_ok"] += ["ftLib_8008627C", "it_8027B5B0"]
_K["effects"].update({"ftLib_80086644": _V("a1")})
_K["deep_ecb"] = True

_EFS_JV = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; Vec3* a3; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); a3 = __builtin_va_arg(ap, Vec3*); __builtin_va_end(ap); "
      "T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); T(PI(a3)); TVEC(a3); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: *mut Vec) -> *mut c_void;",
    hook="|a0, a1, a2, a3| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); t(pi(a3)); tvec(a3); core::ptr::null_mut() }")
_A = SPECS["itlinkarrow"]
_A["custom_stubs"] = {"efSync_Spawn": _EFS_JV}
_A["ret"].update({
    "ftLib_80086630": ("&JO[3 + fv() % 3]", "&raw mut crate::JO[3 + (fv() % 3) as usize]"),
    "HSD_JObjLoadJoint": ("&JO[6 + fv() % 14]", "&raw mut crate::JO[6 + (fv() % 14) as usize]"),
})
_A["effects"].update({
    "mpLineGetNormal": _V("a1"),
    "ftCo_80094098": ("if (a1) { *a1 = rw(); }", "if !a1.is_null() { *a1 = rw(); }"),
})
_A["null_ret_ok"] += ["mpLineGetNormal"]

_B = SPECS["itlinkbomb"]
_B["deep_ecb"] = True
_B["effects"].update({})
# ------------------------------------------------------------------------ shard 737-744
_EFA = dict(
    c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); }",
    ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);",
    hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); }")
_EFS2 = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); TVEC((Vec3*) a2); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); tvec(a2); core::ptr::null_mut() }")
_GO2F = ("&GO[2]", "&raw mut crate::GO[2]")
_MP = ("(int) (fv() & 1)", "(fv() & 1) as i32")
_NEW = ("itluigifireball", "itmaril", "itmariocape", "itmariofireball", "itmarumine", "itmasterhandbullet",
        "itmasterhandlaser", "itmatadogas")
for _n in _NEW:
    SPECS[_n]["skip_stubs"] = ["sinf", "cosf", "__frsqrte"]
    SPECS[_n]["ret"]["it_8027781C"] = _P
    SPECS[_n]["ret"]["ftMr_SpecialS_CheckItemCapeRemove"] = _P
for _n in ("itluigifireball", "itmariofireball"):
    SPECS[_n]["custom_stubs"] = {"efAsync_Spawn": _EFA}
SPECS["itmariocape"]["custom_stubs"] = {"efAsync_Spawn": _EFA}
SPECS["itmasterhandbullet"]["custom_stubs"] = {"efSync_Spawn": dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); TVEC((Vec3*) a2); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); tvec(a2); core::ptr::null_mut() }")}
SPECS["itmasterhandlaser"]["custom_stubs"] = {"efSync_Spawn": SPECS["itmasterhandbullet"]["custom_stubs"]["efSync_Spawn"]}
SPECS["itmarumine"]["custom_stubs"] = {"efSync_Spawn": _EFS4}
SPECS["itmatadogas"]["custom_stubs"] = {"efSync_Spawn": _EFS4}
# the spawned item must exist where the C does not test for it
for _n in ("itmariofireball", "itmasterhandbullet", "itmasterhandlaser"):
    SPECS[_n]["ret"]["Item_80268B18"] = _GO2F
SPECS["itmaril"]["ret"]["it_8026DB40"] = ("(int) (fv() & 15)", "(fv() & 15) as i32")
SPECS["itmasterhandlaser"]["ret"]["mpCheckMultiple"] = _MP
SPECS["itmasterhandlaser"]["effects"]["mpCheckMultiple"] = (
    "if (a4) { a4->x = rw(); a4->y = rw(); a4->z = rw(); }",
    "if !a4.is_null() { (*a4).x = rw(); (*a4).y = rw(); (*a4).z = rw(); }")
for _f in ("it_8027AE34", "it_8027AF50"):
    SPECS["itmasterhandbullet"]["ret"][_f] = _P
for _n in ("itnesspkflash", "itnesspkthunderball"):
    SPECS[_n]["skip_stubs"] = ["sinf", "cosf"]
SPECS["itnessyoyo"]["skip_stubs"] = ["__frsqrte"]
for _n in ("itnessbat", "itnesspkfire", "itnesspkfirepillar", "itnesspkflash", "itnesspkflashexplode",
           "itnesspkthunderball", "itnesspkthundertrail", "itnessyoyo"):
    SPECS[_n]["null_ret_ok"] += ["it_802AF940", "it_802AC43C", "lbVector_CrossprodNormalized"]

# ---- itnessyoyo: string links, matrices ------------------------------------------------------------
_Y = SPECS["itnessyoyo"]
_Y["types"] = _T + ["ItemLink"]
_Y["null_ret_ok"] += ["GObj_Create", "HSD_JObjLoadJoint", "HSD_ObjAlloc"]
_Y["ret"]["GObj_Create"] = ("((fv() % 5) == 0 ? 0 : &LG[CNT[0]++ % 6])",
                            "if fv() % 5 == 0 { core::ptr::null_mut() } else { let n = crate::CNT[0]; crate::CNT[0] += 1; &raw mut crate::LG[n % 6] }")
_Y["ret"]["HSD_ObjAlloc"] = ("&LK[CNT[1]++ % 6]",
                             "{ let n = crate::CNT[1]; crate::CNT[1] += 1; &raw mut crate::LK[n % 6] as *mut c_void }")
_Y["ret"]["HSD_JObjLoadJoint"] = ("&LJ[CNT[2]++ % 6]",
                                  "{ let n = crate::CNT[2]; crate::CNT[2] += 1; &raw mut crate::LJ[n % 6] }")
_Y["effects"]["PSMTXIdentity"] = (
    "if (a0) { int q; for (q = 0; q < 12; q++) ((float*) a0)[q] = (q % 5 == 0) ? 1.0f : 0.0f; }",
    "if !a0.is_null() { let mm = a0 as *mut f32; for q in 0..12usize { *mm.add(q) = if q % 5 == 0 { 1.0 } else { 0.0 }; } }")
_Y["effects"]["PSMTXConcat"] = (
    "if (a2) { int q; for (q = 0; q < 12; q++) ((float*) a2)[q] = rw(); }",
    "if !a2.is_null() { let mm = a2 as *mut f32; for q in 0..12usize { *mm.add(q) = rw(); } }")
_Y["effects"]["PSMTXCopy"] = (
    "if (a0 && a1) { int q; for (q = 0; q < 12; q++) ((float*) a1)[q] = ((float*) a0)[q]; }",
    "if !a0.is_null() && !a1.is_null() { let s = a0 as *const f32; let d = a1 as *mut f32; for q in 0..12usize { *d.add(q) = *s.add(q); } }")
# shard 761-768
_GO2A = ("(&GO[2])", "&raw mut crate::GO[2]")
MY_RET = {n: _P for n in ("it_8027C794", "it_8027C824", "it_8027C79C", "it_8026D5CC", "it_8026DA70", "it_8027B798", "it_8026D324",
                          "lbLang_IsSettingUS", "lbLang_IsSettingJP", "it_802750E8", "it_802751D8", "itColl_BounceOffShield", "it_802E35CC")}
MY_RET.update({
    "it_80276308": ("(int) ((fv() % 3) * 4)", "((fv() % 3) * 4) as i32"),
    "it_8026DAA8": ("(int) (fv() & 15)", "(fv() & 15) as i32"),
    "it_8026DB40": ("(int) (fv() & 15)", "(fv() & 15) as i32"),
    "ftLib_8008627C": _GO1,
    "gm_8016AE80": ("((fv() % 3) == 0 ? -1 : (int) (fv() & 1))", "if fv() % 3 == 0 { -1 } else { (fv() & 1) as i32 }"),
    "it_8028EB88": _GO2,
    "it_8027B5B0": _GO2,
    "it_802E0100": _GO2,
    "it_802E0488": _GO2,
    "it_802DD7F0": _GO2A,
})
MY_EFF = {n: _V("a1") for n in ("ftLib_800866DC", "ftLib_80086644", "it_8026BB88", "it_8027B798", "it_8027BA54")}
MY_EFF["it_8027C8D0"] = _V("a0")
for _n in ("itnokonoko", "itoctarock", "itoctarockstone", "itoldkuri", "itoldottosea", "itparasol", "itpatapata", "itpeachexplode"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret={**COMMON_RET, **MY_RET}, effects={**COMMON_EFF, **MY_EFF},
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi", "it_8028EB88", "it_8027B5B0",
                                  "it_802E0100", "it_802E0488", "ftLib_8008627C"])
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
_GOR = ("((fv() & 1) ? &GO[1] : 0)", "if fv() & 1 != 0 { &raw mut crate::GO[1] } else { core::ptr::null_mut() }")
_M = SPECS["itsamusmissile"]
_M["custom_stubs"] = {"efSync_Spawn": _EFS3J}
_M["ret"]["it_8026B634"] = _GOR
_M["ret"]["it_8026C258"] = _GOR
_M["effects"]["it_8026B634"] = _V("a1")
_M["effects"]["it_8026BB88"] = _V("a1")
_M["effects"]["lbVector_NormalizeXY"] = _V("a0")
_M["effects"]["lbVector_Rotate"] = _V("a0")
_M["effects"]["lbVector_Sub"] = (
    "if (a0 && a1) { a0->x -= a1->x; a0->y -= a1->y; a0->z -= a1->z; }",
    "if !a0.is_null() && !a1.is_null() { (*a0).x -= (*a1).x; (*a0).y -= (*a1).y; (*a0).z -= (*a1).z; }")

_B = SPECS["itsamusbomb"]
_B["effects"]["lbVector_NormalizeXY"] = _V("a0")
_B["ret"]["ftSs_Init_80128A1C"] = _P
_B["skip_stubs"] = ["__frsqrte"]

_C = SPECS["itsamuschargeshot"]
_C["custom_stubs"] = {"efSync_Spawn": _EFS3J}
_C["skip_stubs"] = ["sinf", "cosf"]
_C["ret"]["ftLib_80086630"] = ("(&JO[3 + (fv() % 3)])", "&raw mut crate::JO[3 + (fv() % 3) as usize]")
_NEG = ("((fv() % 3) == 0 ? -1 : (int) (fv() % 3))", "if fv() % 3 == 0 { -1 } else { (fv() % 3) as i32 }")
_C["ret"]["ftSs_SpecialLw_80129100"] = _NEG
_C["ret"]["ftKb_SpecialNSs_800FCC14"] = _NEG
for _n in ("ftSs_SpecialLw_80129158", "ftSs_SpecialN_801291A8", "ftKb_SpecialNSs_800FCC6C", "ftKb_SpecialNSs_800FCCBC"):
    _C["ret"][_n] = _P
_IE = ("if (a1) { *a1 = (int) (fv() % 9); } if (a2) { *a2 = (int) (fv() % 9); }",
       "if !a1.is_null() { *a1 = (fv() % 9) as i32; } if !a2.is_null() { *a2 = (fv() % 9) as i32; }")
_C["effects"]["ftSs_SpecialLw_80129100"] = _IE
_C["effects"]["ftKb_SpecialNSs_800FCC14"] = _IE

_R = SPECS["itrshell"]
_R["custom_stubs"] = {"efAsync_Spawn": dict(
    c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { __builtin_va_list ap; Vec3* a5; "
      "__builtin_va_start(ap, a4); a5 = __builtin_va_arg(ap, Vec3*); __builtin_va_end(ap); "
      "T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); T(PI(a5)); TVEC(a5); }",
    ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut Vec);",
    hook="|a0, a1, a2, a3, a4, a5| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); t(pi(a5)); tvec(a5); }")}
_R["ret"]["it_8026B5E4"] = _GOR
for _n in ("it_8026DF34", "it_80276348"):
    _R["ret"][_n] = _P
_R["effects"]["it_8026B5E4"] = _V("a1")


# ------------------------------------------------------------------------------------------ itsamusgrapple
SPECS["itsamusgrapple"] = dict(
    src=f"{K}itsamusgrapple.c",
    types=_T + ["Fighter", "ItemLink", "FighterBone", "ftSs_DatAttrs", "itSamusGrappleAttributes", "itSamusGrapple_HitboxData"],
    ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
    null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi", "ft_80082E3C", "ftCo_800A4A40", "GObj_Create",
                 "HSD_ObjAlloc", "HSD_JObjLoadJoint", "ftLib_80086630"],
    # the game reads two packed bitfields of the hitbox command by byte offset (PPC, MSB-first): the native copy reads
    # the same two fields by name (see the header of src/tu/itsamusgrapple.rs)
    patch=[("    damage_arg = *damage;\n    damage_arg &= 0x3FF;",
            "    (void) damage; damage_arg = data->create_hitbox.create_hitbox_0.damage;"),
           ("hitbox->x40_b3 = hitbox_flags->hit_grounded;",
            "hitbox->x40_b3 = data->create_hitbox.create_hitbox_4.hit_grounded;")],
    custom_stubs={"efSync_Spawn": dict(
        c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; void* a3; __builtin_va_start(ap, a1); "
          "a2 = __builtin_va_arg(ap, void*); a3 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); "
          "T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); T(PI(a3)); TVEC((Vec3*) a2); return 0; }",
        ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut c_void) -> *mut c_void;",
        hook="|a0, a1, a2, a3| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); t(pi(a3)); tvec(a2); core::ptr::null_mut() }")},
)
_G = SPECS["itsamusgrapple"]
_G["ret"]["GObj_Create"] = ("((fv() % 8) == 0 ? 0 : &LGO[LGN++ % 24])",
                            "if fv() % 8 == 0 { core::ptr::null_mut() } else { let i = unsafe { crate::LGN }; unsafe { crate::LGN += 1; } &raw mut crate::LGO[i % 24] }")
_G["ret"]["HSD_ObjAlloc"] = ("&LK[LKN++ % 24]",
                             "{ let i = unsafe { crate::LKN }; unsafe { crate::LKN += 1; } &raw mut crate::LK[i % 24] as *mut _ }")
_G["ret"]["HSD_JObjLoadJoint"] = ("&JO[16 + (fv() % 8)]", "&raw mut crate::JO[16 + (fv() % 8) as usize]")
_G["ret"]["ftLib_80086630"] = ("(&JO[3 + (fv() % 3)])", "&raw mut crate::JO[3 + (fv() % 3) as usize]")
_G["ret"]["ftCo_800A4A40"] = ("((fv() & 1) ? &FT[1] : 0)", "if fv() & 1 != 0 { &raw mut crate::FT[1] } else { core::ptr::null_mut() }")
_G["ret"]["ft_80082E3C"] = _GOR
_G["ret"]["ftCo_800C3A14"] = _P
_G["ret"]["mpCheckAllRemap"] = _P
_G["ret"]["ftParts_GetBoneIndex"] = ("(int) (fv() % 200)", "(fv() % 200) as _")
_G["ret"]["it_802A3C98"] = ("(float) (rnd() % 40) / 4.0f", "(rnd() % 40) as f32 / 4.0")
_G["ret"]["it_802A3D90"] = ("((fv() % 3) == 0 ? 0 : (int) (fv() & 0x1FFF))", "if fv() % 3 == 0 { 0 } else { (fv() & 0x1FFF) as i32 }")
_G["effects"]["it_802A3C98"] = _V("a2")
_G["effects"]["GObj_InitUserData"] = ("if (a0) { a0->user_data = a3; }", "if !a0.is_null() { (*a0).user_data = a3; }")
_G["effects"]["HSD_GObjObject_80390A70"] = ("if (a0) { a0->hsd_obj = a2; }", "if !a0.is_null() { (*a0).hsd_obj = a2; }")
_I = lambda e: (f"(int) ({e[0]})", f"({e[1]}) as i32")
_SMALL9 = ("(int) (fv() % 9)", "(fv() % 9) as i32")
_ONEISH = ("(int) (fv() % 3)", "(fv() % 3) as i32")
SPECS["itseakneedleheld"]["ret"].update({
    "ftSk_SpecialS_80111FA0": _SMALL9, "ftKb_SpecialNSk_80106020": _SMALL9,
    "ftSk_SpecialS_80111F70": _ONEISH, "ftKb_SpecialNSk_80105FF0": _ONEISH,
})
SPECS["itseakneedlethrown"]["skip_stubs"] = ["sinf", "cosf"]
SPECS["itsscope"]["ret"]["it_8026DF34"] = _P  # a bool-returning callee: the stub returns 0 / 1 like the real one

# ---- itseakchain: a 16-link pool (LK / LGO / LJO), one Fighter (FP) behind GO[2]
SPECS["itseakchain"]["types"] = _T + ["ItemLink"]
SPECS["itseakchain"]["ret"].update({
    "Item_80268B18": _GO1,
    "GObj_Create": ("((fv() % 7) == 0 ? 0 : &LGO[LGC++ % 16])",
                    "if fv() % 7 == 0 { core::ptr::null_mut() } else { let i = crate::LGC; crate::LGC += 1; &raw mut crate::LGO[i % 16] }"),
    "HSD_ObjAlloc": ("(void*) &LK[LKC++ % 16]",
                     "{ let i = crate::LKC; crate::LKC += 1; &raw mut crate::LK[i % 16] as *mut c_void }"),
    "HSD_JObjLoadJoint": ("(HSD_JObj*) &LJO[fv() % 32]", "&raw mut crate::LJO[(fv() % 32) as usize]"),
    "it_802A3C98": ("(float) (fv() % 12)", "(fv() % 12) as f32"),
    "mpColl_80048844": _P,
    "ftLib_GetMotionId": ("(int) (344 + fv() % 16)", "(344 + fv() % 16) as i32"),
})
SPECS["itseakchain"]["effects"].update({
    "it_802A3C98": _V("a2"),
    "it_8026BB68": _V("a1"),
    "PSMTXConcat": ("if (a2) { int q; for (q = 0; q < 12; q++) ((float*) a2)[q] = rw(); }",
                    "if !a2.is_null() { for q in 0..12 { *(a2 as *mut f32).add(q) = rw(); } }"),
    "PSMTXCopy": ("if (a0 && a1) { int q; for (q = 0; q < 12; q++) ((float*) a1)[q] = ((float*) a0)[q]; }",
                  "if !a0.is_null() && !a1.is_null() { for q in 0..12 { *(a1 as *mut f32).add(q) = *(a0 as *mut f32).add(q); } }"),
    "PSMTXIdentity": ("if (a0) { int q; for (q = 0; q < 12; q++) ((float*) a0)[q] = (q % 5 == 0) ? 1.0f : 0.0f; }",
                      "if !a0.is_null() { for q in 0..12 { *(a0 as *mut f32).add(q) = if q % 5 == 0 { 1.0 } else { 0.0 }; } }"),
})
SPECS["itseakchain"]["null_ret_ok"] += ["GObj_Create", "HSD_ObjAlloc", "HSD_JObjLoadJoint"]
SPECS["itsscopebeam"]["ret"]["it_8026DAA8"] = ("(int) (fv() & 3)", "(fv() & 3) as i32")
SPECS["itsscopebeam"]["ret"]["atan2f"] = ("((float) ((int) (rnd() % 82) - 41) * 0.01f)", "((rnd() % 82) as i32 - 41) as f32 * 0.01")

SPECS["itsuikun"]["custom_stubs"] = {"efSync_Spawn": _EFS4}
SPECS["itsword"]["ret"]["ftLib_80086FA8"] = ("(int) (fv() & 1)", "(fv() & 1) as i32")
SPECS["itsword"]["ret"]["it_80272CC0"] = ("(&JO[3 + (fv() % 3)])", "&raw mut crate::JO[3 + (fv() % 3) as usize]")

SPECS["ittaru"]["custom_stubs"] = {"efSync_Spawn": _EFS3}
SPECS["ittarucann"]["custom_stubs"] = {"efSync_Spawn": _EFS3}
SPECS["ittarucann"]["skip_stubs"] = ["sinf", "cosf"]

SPECS["ittarucann"]["effects"]["ftCo_800C92E4"] = (
    "if (a3) { T((u32) a3->state); T(a3->damage); T((u32) a3->kb_angle); T(a3->unkC); T(a3->unk10); T(a3->unk14); T(a3->element); T((u32) a3->sfx_severity); T((u32) a3->sfx_kind); }",
    "if !a3.is_null() { t((*a3).state as u32); t((*a3).damage); t((*a3).kb_angle as u32); t((*a3).unkC); t((*a3).unk10); t((*a3).unk14); t((*a3).element); t((*a3).sfx_severity as u32); t((*a3).sfx_kind as u32); }")
