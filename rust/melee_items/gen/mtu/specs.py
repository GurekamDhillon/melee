"""Per-TU oracle specs for gen_tu.py / run_tu.py (melee_items::tu). Keys as in melee_world/gen/mtu/specs.py, plus
`pools` (record types the generated fill functions point their pointer fields into, see mk_driver.py)."""
from mk_driver import WORLD_TYPES, POOL_TYPES, COMMON_GLOBALS

COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}
SPECS = {}


def add(stem, src, types=(), **kw):
    kw["manual_data"] = list(kw.get("manual_data", [])) + list(COMMON_GLOBALS)
    SPECS[stem] = dict(src=src, types=list(WORLD_TYPES) + list(types), pools=POOL_TYPES, common_ret=COMMON_RET, **kw)


add("ithitbox", "src/melee/it/ithitbox.c")

_SPAWN_C = ("T((u32) a0->kind); T((u32) a0->x3C_damage); T((u32) a0->x40); T(B(a0->vel.x)); T(B(a0->vel.y)); T(B(a0->vel.z)); "
            "T(B(a0->prev_pos.x)); T(B(a0->prev_pos.y)); T(B(a0->prev_pos.z)); T(B(a0->pos.x)); T(B(a0->pos.y)); T(B(a0->pos.z)); "
            "T(B(a0->facing_dir)); T(PI(a0->x0_parent_gobj)); T(PI(a0->x4_parent_gobj2)); T((u32) a0->x44_flag.x0.b0);")
_SPAWN_R = ("t((*a0).kind as u32); t((*a0).x3C_damage as u32); t((*a0).x40 as u32); t((*a0).vel.x.to_bits()); t((*a0).vel.y.to_bits()); "
            "t((*a0).vel.z.to_bits()); t((*a0).prev_pos.x.to_bits()); t((*a0).prev_pos.y.to_bits()); t((*a0).prev_pos.z.to_bits()); "
            "t((*a0).pos.x.to_bits()); t((*a0).pos.y.to_bits()); t((*a0).pos.z.to_bits()); t((*a0).facing_dir.to_bits()); "
            "t(pi((*a0).x0_parent_gobj)); t(pi((*a0).x4_parent_gobj2)); t((*a0).x44_flag.x0.b0() as u32);")
add("itdrop", "src/melee/it/itdrop.c",
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R),
             "ftLib_800874CC": ("{ int ii; for (ii = 0; ii < a2 && ii < 8; ii++) T(((int*) a1)[ii]); }",
                                "for ii in 0..(a2.min(8).max(0)) { t(*(a1 as *const i32).add(ii as usize) as u32); }")},
)


from specs_ef import EF_CUSTOM, EF_PRE, EFARG_RS  # noqa: E402

add("iteffect", "src/melee/it/iteffect.c", types=["it_804D6D40_t"],
    data_size={n: 16 for n in ("it_803F1ED8", "it_803F1EE4", "it_803F1EF0")}, manual_data=["it_803F1ED8", "it_803F1EE4", "it_803F1EF0"], custom_stubs={**EF_CUSTOM, "lbArchive_80017040": dict(
        c="bool lbArchive_80017040(HSD_Archive** a0, const char* a1, void* a2, ...) { __builtin_va_list ap; const char* sym; int z; "
          "__builtin_va_start(ap, a2); sym = __builtin_va_arg(ap, const char*); z = __builtin_va_arg(ap, int); __builtin_va_end(ap); "
          "T(@ID@); T(PI(a0)); T(SH(a1)); T(PI(a2)); T(SH(sym)); T((u32) z); *(void**) a2 = ARCP; return (int) (fv() & 1); }",
        ext="fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: *const i8, a4: i32) -> i32;",
        hook="|a0, a1, a2, a3, a4| { t(@ID@); t(pi(a0)); t(sh(a1 as *const u8)); t(pi(a2)); t(sh(a3 as *const u8)); t(a4 as u32); "
             "*(a2 as *mut *mut it_804D6D20_t) = &raw mut crate::ARC; (fv() & 1) as i32 }")},
    ret={"it_80272CC0": ("pick_HSD_JObj()", "pick_HSD_JObj()"), "lbLang_IsSettingUS": ("(int) (fv() & 1)", "(fv() & 1) as i32")})

add("itdraw", "src/melee/it/itdraw.c",
    ret={"HSD_CObjGetCurrent": ("COBP", "&raw mut crate::COB"), "Camera_80031060": ("(int) (fv() % 3)", "(fv() % 3) as i32")},
    effects={
        "PSMTXIdentity": ("{ int q; for (q = 0; q < 12; q++) ((float*) a0)[q] = (q % 5 == 0) ? 1.0f : 0.0f; }",
                          "for q in 0..12usize { *(a0 as *mut f32).add(q) = if q % 5 == 0 { 1.0 } else { 0.0 }; }"),
        "PSMTXConcat": ("{ int q; for (q = 0; q < 12; q++) { T(B(((float*) a0)[q])); T(B(((float*) a1)[q])); ((float*) a2)[q] = rw(); } }",
                        "for q in 0..12usize { t((*(a0 as *const f32).add(q)).to_bits()); t((*(a1 as *const f32).add(q)).to_bits()); *(a2 as *mut f32).add(q) = rw(); }"),
        "HSD_JObjDispAll": ("if (a1) { int q; for (q = 0; q < 12; q++) T(B(((float*) a1)[q])); }",
                            "if !a1.is_null() { for q in 0..12usize { t((*(a1 as *const f32).add(q)).to_bits()); } }"),
        "ftLib_GetShakeOffset": ("if (a1) { a1->x = rw(); a1->y = rw(); a1->z = rw(); }",
                           "if !a1.is_null() { (*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw(); }"),
    })

add("itcoll", "src/melee/it/itcoll.c", custom_stubs=EF_CUSTOM, manual_data=["Fighter_804D6524"], null_ret_ok=["ftCommon_8007F804"],
    ret={"it_802746F8": ("&JO[rnd() % 8]", "&raw mut JO[(rnd() % 8) as usize]"),
         "ftLib_IsFighter": ("((fv() & 1) && a0 >= &GO[4] && a0 <= &GO[6])", "{ let v = fv() & 1; (v != 0 && a0 >= &raw mut GO[4] && a0 <= &raw mut GO[6]) as i32 }")},
    effects={"lbVector_RotateAboutUnitAxis": ("if (a0) { a0->x = rw(); a0->y = rw(); a0->z = rw(); }",
                                              "if !a0.is_null() { (*a0).x = rw(); (*a0).y = rw(); (*a0).z = rw(); }")})

_MP = ("a0->cur_pos.x = rw(); a0->cur_pos.y = rw(); a0->floor.index = (s32) fv();", "(*a0).cur_pos.x = rw(); (*a0).cur_pos.y = rw(); (*a0).floor.index = fv() as i32;")
add("itgroundcoll", "src/melee/it/itgroundcoll.c",
    ret={"mpCheckAllRemap": ("(int) (fv() & 1)", "(fv() & 1) as i32")},
    effects={"mpColl_800471F8": _MP, "mpColl_800477E0": _MP, "mpColl_8004B108": _MP, "mpColl_8004B2DC": _MP,
             "mpCheckAllRemap": ("if (a0) { a0->x = rw(); a0->y = rw(); a0->z = rw(); }",
                                 "if !a0.is_null() { (*a0).x = rw(); (*a0).y = rw(); (*a0).z = rw(); }")})

add("itmaplib", "src/melee/it/itmaplib.c", skip_stubs=["__frsqrte"],
    ret={"it_80272CC0": ("&JO[rnd() % 8]", "&raw mut JO[(rnd() % 8) as usize]"), "it_80272D40": ("(int) (fv() & 1)", "(fv() & 1) as i32")})

_ART = ("&ART[rnd() % 2]", "&raw mut ART[(rnd() % 2) as usize] as *mut _")
_LT = ("&LT[rnd() % 2]", "&raw mut LT[(rnd() % 2) as usize] as *mut _")
add("item", "src/melee/it/item.c", skip_data=["it_mobj"],
    # bit-order note: the C oracle is x86 (first-declared bitfield = LSB), the port keeps the GameCube order
    # (b0 = MSB). `xDAA_byte = 1` therefore means "b7 set" on the console: spell that in bits so both sides agree.
    patch=[("item_data->xDAA_byte = 1;", "item_data->xDAA_byte = 0; item_data->xDAA_flag.b7 = 1;")],
    ret={"HSD_ObjAlloc": ("((rnd() % 5) == 0 ? 0 : (void*) &DBT[rnd() % 4])", "if rnd() % 5 == 0 { core::ptr::null_mut() } else { (&raw mut DBT[(rnd() % 4) as usize]) as *mut _ }"),
         "GObj_Create": ("((rnd() & 3) == 0 ? 0 : &GO[rnd() % 4])", "if rnd() & 3 == 0 { core::ptr::null_mut() } else { &raw mut GO[(rnd() % 4) as usize] }"),
         "HSD_JObjAlloc": ("&JO[rnd() % 8]", "&raw mut JO[(rnd() % 8) as usize]"),
         "HSD_JObjGetDObj": ("((rnd() & 1) ? (HSD_DObj*) DOBBUF[rnd() % 4] : 0)", "if rnd() & 1 != 0 { (&raw mut super::DOBBUF[(rnd() % 4) as usize]) as *mut _ } else { core::ptr::null_mut() }"),
         "Camera_80029044": ("((rnd() & 1) ? &CM[rnd() % 2] : 0)", "if rnd() & 1 != 0 { &raw mut CM[(rnd() % 2) as usize] } else { core::ptr::null_mut() }"),
         "Geno_ArticleDesc": _ART, "Geno_ItemDesc": _ART, "Mex_ItemCustomDesc": _ART,
         "Geno_ArticleLogic": _LT, "Geno_ItemLogic": _LT, "Mex_ItemCustomLogic": _LT,
         "Mex_SpawnArticleKind": ("(int) (rnd() % 300)", "(rnd() % 300) as i32"),
         "Mex_SsmForPortKind": ("(int) (rnd() % 20) - 3", "(rnd() % 20) as i32 - 3"),
         "gm_8017E068": ("(int) (rnd() % 14) - 1", "(rnd() % 14) as i32 - 1"),
         "it_80272D1C": ("1", "1"),
         "ScriptGame_HitRuleBaseDamage": ("(float) (rnd() % 60)", "(rnd() % 60) as f32")})
_XP_MAT = [("HSD_MObj", "MO", 2), ("HSD_Material", "MA", 2), ("HSD_TObj", "TO", 5), ("HSD_DObj", "DO", 3),
           ("HSD_PEDesc", "PE", 2), ("ColorOverlay", "CO", 2), ("HSD_MObjInfo", "MI", 1)]
add("itmaterial", "src/melee/it/itmaterial.c", types=["it_MObjInfo", "HSD_TevDesc", "HSD_TECnst", "GXColor"], xpools=_XP_MAT,
    manual_data=["HSD_GObj_804D7814", "tobj_shadows", "tobj_toon", "hsdMObj"],
    skip_freg=[],
    ret={"lbGetFreeColorRegister": ("(int) (rnd() % 8)", "(rnd() % 8) as i32"),
         "HSD_StateAssignTev": ("(int) (rnd() % 8)", "(rnd() % 8) as i32"),
         "ftCo_800C0674": ("&CO[rnd() % 2]", "&raw mut crate::g::CO[(rnd() % 2) as usize]"),
         "HSD_JObjGetDObj": ("pick_HSD_DObj()", "pick_HSD_DObj()")},
    effects={"HSD_SetupTevStage": ("hash_HSD_TevDesc(a0);", "hash_HSD_TevDesc(&*a0);"),
             "HSD_TExpSetReg": ("hash_HSD_TECnst(&a0->cnst); { const unsigned char* v = a0->cnst.val; if (v) { T(v[0]); if ((int) a0->cnst.comp != 5) { T(v[1]); T(v[2]); T(v[3]); } } }", "hash_HSD_TECnst(&(*a0).cnst); { let v = (*a0).cnst.val as *const u8; if !v.is_null() { t(*v as u32); if (*a0).cnst.comp as i32 != 5 { t(*v.add(1) as u32); t(*v.add(2) as u32); t(*v.add(3) as u32); } } }"),
             "HSD_SetupRenderModeWithCustomPE": ("if (a1) hash_HSD_PEDesc(a1);", "if !a1.is_null() { hash_HSD_PEDesc(&*a1); }"),
             "HSD_SetMaterialColor": ("hash_GXColor(&a0); hash_GXColor(&a1); hash_GXColor(&a2);", "hash_GXColor(&a0); hash_GXColor(&a1); hash_GXColor(&a2);")})
from specs_ef import EF_CUSTOM as EF_CUSTOM_  # noqa: E402
add("itspawn", "src/melee/it/itspawn.c", custom_stubs=EF_CUSTOM_, skip_freg=["bisectValue", "it_8026C88C_inline", "it_8026D018_inline", "it_8026D018_inline2", "it_8026D018_inline3"], types=["ItemPickTable", "RandomItemSpawner"],
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()"),
         "it_80272828": ("pick_igo()", "pick_igo()"),
         "HSD_MemAlloc": ("(a0 > 0 ? (AO += ((u32) a0 + 15u) & ~15u, (void*) (AB + AO - (((u32) a0 + 15u) & ~15u))) : (void*) 0)",
                          "if a0 > 0 { let sz = (a0 as u32 + 15) & !15; crate::AO += sz; (&raw mut crate::AB as *mut u8).add((crate::AO - sz) as usize) as *mut c_void } else { core::ptr::null_mut() }"),
         "Ground_801C2AD8": ("((fv() & 1) ? CNT : (s32*) 0)", "if fv() & 1 != 0 { (&raw mut crate::CNT) as *mut i32 } else { core::ptr::null_mut() }"),
         "gm_8016AE80": ("GMV", "crate::GMV"),
         "gm_8016AEA4": ("(u64) FM64", "crate::FM64")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R), "Item_80268B9C": (_SPAWN_C, _SPAWN_R)})
def TVc(a): return f"T(B({a}->x)); T(B({a}->y)); T(B({a}->z));"
def TVr(a): return f"t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());"
def WVc(a): return f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); }}"
def WVr(a): return f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); }}"
def VEFF(inputs=(), outputs=()):
    c = " ".join(f"if ({a}) {{ {TVc(a)} }}" for a in inputs) + " " + " ".join(WVc(a) for a in outputs)
    r = " ".join(f"if !{a}.is_null() {{ {TVr(a)} }}" for a in inputs) + " " + " ".join(WVr(a) for a in outputs)
    return (c, r)


add("itzako", "src/melee/it/itzako.c", custom_stubs={}, skip_freg=["it_8027B798_CalcAngle", "product_xyz", "itzako_sqrtf", "return_sqrt_value3"], skip_stubs=["__frsqrte", "cosf", "sinf"],
    types=["it_804D6D40_t", "ftCommonData"], manual_data=["p_ftCommonData", "it_804D6D40"], null_ret_ok=["lbVector_CrossprodNormalized", "lbVector_Diff", "lbVector_EulerAnglesFromONB"],
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()"), "Item_80268B5C": ("pick_igo()", "pick_igo()"), "it_802F2094": ("pick_igo()", "pick_igo()"),
         "ftLib_GetMotionId": ("(int) ((rnd() % 5 == 0) ? 0x14E : (rnd() % 5 == 0) ? 0x14D : (int) fv())", "if rnd() % 5 == 0 { 0x14E } else if rnd() % 5 == 0 { 0x14D } else { fv() as i32 }"),
         "Ground_801C5840": ("((rnd() % 4 == 0) ? -1 : (int) (fv() % 8))", "if rnd() % 4 == 0 { -1 } else { (fv() % 8) as i32 }"),
         "tyDisplay_8031C354": ("({ int n_ = (int) (fv() % 4); int ii_; for (ii_ = 0; ii_ < n_; ii_++) a1[ii_] = (int) fv(); n_; })",
                                "{ let n_ = (fv() % 4) as i32; for ii_ in 0..n_ { *a1.add(ii_ as usize) = fv() as i32; } n_ }"),
         "ftLib_GetSubColor": ("({ *(u32*) a1 = fv(); (int) (fv() & 1); })", "{ *(a1 as *mut u32) = fv(); (fv() & 1) as i32 }")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R), "Item_80268B5C": (_SPAWN_C, _SPAWN_R),
             "Camera_GetTransformPosition": VEFF(outputs=["a0"]),
             "it_8026B684": VEFF(inputs=["a0"]),
             "it_8026BB88": VEFF(outputs=["a1"]),
             "lb_8000B1CC": VEFF(outputs=["a2"]),
             "lbVector_Angle": VEFF(inputs=["a0", "a1"]), "lbVector_AngleXY": VEFF(inputs=["a0", "a1"]),
             "lbVector_Diff": VEFF(inputs=["a0", "a1"], outputs=["a2"]),
             "lbVector_Normalize": VEFF(inputs=["a0"], outputs=["a0"]),
             "lbVector_Rotate": VEFF(inputs=["a0"], outputs=["a0"]),
             "lbVector_CrossprodNormalized": VEFF(inputs=["a1", "a2"], outputs=["a0"]),
             "lbVector_EulerAnglesFromONB": VEFF(inputs=["a1", "a2", "a3"], outputs=["a0"]),
             "it_80278574": ("T(((u8*) a1)[0]); T(((u8*) a1)[1]); T(((u8*) a1)[2]); T(((u8*) a1)[3]);",
                             "t(*(a1 as *const u8) as u32); t(*(a1 as *const u8).add(1) as u32); t(*(a1 as *const u8).add(2) as u32); t(*(a1 as *const u8).add(3) as u32);"),
             "ftLib_GetGameWatchColor": ("*(u32*) a1 = fv();", "*(a1 as *mut u32) = fv();"),
             "ftLib_GetGameWatchOutlineColor": ("*(u32*) a1 = fv();", "*(a1 as *mut u32) = fv();")})
add("it_26B1", "src/melee/it/it_26B1.c")
add("it_2725", "src/melee/it/it_2725.c")
add("it_279C", "src/melee/it/it_279C.c")

add("itdkinoko", "src/melee/it/kinds/itdkinoko.c")

add("itclinkmilk", "src/melee/it/kinds/itclinkmilk.c",
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()"), "ftCl_Init_8014920C": ("(int) (fv() % 3)", "(fv() % 3) as i32")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R)})

add("itcrazyhandbomb", "src/melee/it/kinds/itcrazyhandbomb.c",
    ret={"Item_80268B18": ("&GO[rnd() % 4]", "&raw mut GO[(rnd() % 4) as usize]")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R)})

add("itcoin", "src/melee/it/kinds/itcoin.c",
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()"), "tyDisplay_8031C5E4": ("pick_HSD_JObj()", "pick_HSD_JObj()")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R)})

add("itdrmariopill", "src/melee/it/kinds/itdrmariopill.c", skip_freg=["my_sqrtf"], skip_stubs=["__frsqrte", "cosf", "sinf"],
    manual_data=["it_804D6D28"],
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R),
             "ftLib_GetPos": VEFF(outputs=["a1"]),
             "it_8026BB68": VEFF(outputs=["a1"])})

add("itclimbersice", "src/melee/it/kinds/itclimbersice.c", custom_stubs=EF_CUSTOM_,
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R), "it_8026BB68": VEFF(outputs=["a1"])})

_MTX_ID_C = "{ int q; for (q = 0; q < 12; q++) ((float*) a0)[q] = (q % 5 == 0) ? 1.0f : 0.0f; }"
_MTX_ID_R = "for q in 0..12usize { *(a0 as *mut f32).add(q) = if q % 5 == 0 { 1.0 } else { 0.0 }; }"
_MTX_CC = "{ int q; for (q = 0; q < 12; q++) { T(B(((float*) a0)[q])); T(B(((float*) a1)[q])); ((float*) a2)[q] = rw(); } }"
_MTX_CR = "for q in 0..12usize { t((*(a0 as *const f32).add(q)).to_bits()); t((*(a1 as *const f32).add(q)).to_bits()); *(a2 as *mut f32).add(q) = rw(); }"
_MTX_CPC = "{ int q; for (q = 0; q < 12; q++) { T(B(((float*) a0)[q])); ((float*) a1)[q] = ((float*) a0)[q]; } }"
_MTX_CPR = "for q in 0..12usize { t((*(a0 as *const f32).add(q)).to_bits()); *(a1 as *mut f32).add(q) = *(a0 as *const f32).add(q); }"
add("itclimbersstring", "src/melee/it/kinds/itclimbersstring.c", xpools=[("ItemLink", "LK", 6)],
    ret={"Item_80268B18": ("pick_igo()", "pick_igo()"),
         "GObj_Create": ("((rnd() % 5 == 0) ? (HSD_GObj*) 0 : &GO[7 + (rnd() & 1)])", "if rnd() % 5 == 0 { core::ptr::null_mut() } else { &raw mut GO[7 + (rnd() & 1) as usize] }"),
         "HSD_JObjLoadJoint": ("pick_HSD_JObj()", "pick_HSD_JObj()"),
         "HSD_ObjAlloc": ("(void*) &LK[(LKN++) % 6]", "{ let i = crate::LKN; crate::LKN += 1; (&raw mut crate::g::LK[i % 6]) as *mut c_void }"),
         "ftLib_GetMotionId": ("(int) ((rnd() & 1) ? 0x15A + rnd() % 11 : fv())", "if rnd() & 1 != 0 { 0x15A + (rnd() % 11) as i32 } else { fv() as i32 }")},
    effects={"Item_80268B18": (_SPAWN_C, _SPAWN_R),
             "PSMTXIdentity": (_MTX_ID_C, _MTX_ID_R),
             "PSMTXConcat": (_MTX_CC, _MTX_CR),
             "PSMTXCopy": (_MTX_CPC, _MTX_CPR),
             "it_802A3C98": VEFF(inputs=["a0", "a1"], outputs=["a2"]),
             "lbVector_Normalize": VEFF(inputs=["a0"], outputs=["a0"]),
             "it_802A6DC8": VEFF(inputs=["a1", "a2"]),
             "it_802A4420": ("T(B(a0->pos.x)); T(B(a0->vel.y)); a0->pos.x = rw(); a0->pos.y = rw(); a0->vel.x = rw();",
                             "t((*a0).pos.x.to_bits()); t((*a0).vel.y.to_bits()); (*a0).pos.x = rw(); (*a0).pos.y = rw(); (*a0).vel.x = rw();")})

add("itdosei", "src/melee/it/kinds/itdosei.c", manual_data=[],
    ret={})

_GXC = ("((GXColor) { (u8) fv(), (u8) fv(), (u8) fv(), (u8) fv() })", "_GXColor { r: fv() as u8, g: fv() as u8, b: fv() as u8, a: fv() as u8 }")
SPECS["toy"] = dict(
    src="src/melee/ty/toy.c", types=[], skip_stubs=["__frsqrte", "sinf", "cosf", "memcpy"], skip_data=["result"], no_freg=True, pools=POOL_TYPES, common_ret=COMMON_RET, manual_data=[],
    ret={"Mex_Enabled": ("(int) ((rnd() & 15) == 0)", "((rnd() & 15) == 0) as i32"), "DevText_SetBGColor": _GXC, "DevText_SetTextColor": _GXC,
         "gmMainLib_GetTrophyFlags": ("SAVEF", "&raw mut crate::SAVEF as *mut _"),
         "gmMainLib_GetTrophyCategoryFlags": ("&CATF", "&raw mut crate::CATF as *mut _"),
         "gmMainLib_GetTrophyCount": ("&TCNT", "&raw mut crate::TCNT")},
    effects={"memzero": ("zero(a0, (unsigned) a1);", "core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize);")},
)
