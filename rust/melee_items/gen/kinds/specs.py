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
for _n in ("itarwinglaser", "itbat", "itbombhei", "itbox", "itcapsule", "itcerebi", "itchicorita", "itclimbersblizzard"):
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

# ---- shard 777-784: itporygon2 itrabbitc itraikou itrshell itsamusbomb itsamuschargeshot itsamusgrapple itsamusmissile
for _n in ("itporygon2", "itrabbitc", "itraikou", "itrshell", "itsamusbomb", "itsamuschargeshot", "itsamusmissile"):
    SPECS[_n] = dict(src=f"{K}{_n}.c", types=_T, ret=dict(COMMON_RET), effects=dict(COMMON_EFF),
                     null_ret_ok=["Item_80268B18", "Item_80268B5C", "it_8026F6BC", "HSD_Randi"])

SPECS["itrabbitc"]["ret"]["it_80272CC0"] = ("(&JO[3 + (fv() % 3)])", "&raw mut crate::JO[3 + (fv() % 3) as usize]")
SPECS["itraikou"]["custom_stubs"] = {"efSync_Spawn": _EFS4}

_EFS3J = dict(
    c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; void* a2; __builtin_va_start(ap, a1); "
      "a2 = __builtin_va_arg(ap, void*); __builtin_va_end(ap); T(@ID@); T((u32) a0); T(PI(a1)); T(PI(a2)); return 0; }",
    ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj) -> *mut c_void;",
    hook="|a0, a1, a2| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); core::ptr::null_mut() }")
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
