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
           "itnessbat", "itnesspkfire", "itnesspkfirepillar", "itnesspkflash", "itnesspkflashexplode",
           "itnesspkthunderball", "itnesspkthundertrail", "itnessyoyo"):
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
