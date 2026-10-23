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
            "T(B(a0->facing_dir)); T(PI(a0->x0_parent_gobj)); T(PI(a0->x4_parent_gobj2)); T((u32) a0->x44_flag.b0);")
_SPAWN_R = ("t((*a0).kind as u32); t((*a0).x3C_damage as u32); t((*a0).x40 as u32); t((*a0).vel.x.to_bits()); t((*a0).vel.y.to_bits()); "
            "t((*a0).vel.z.to_bits()); t((*a0).prev_pos.x.to_bits()); t((*a0).prev_pos.y.to_bits()); t((*a0).prev_pos.z.to_bits()); "
            "t((*a0).pos.x.to_bits()); t((*a0).pos.y.to_bits()); t((*a0).pos.z.to_bits()); t((*a0).facing_dir.to_bits()); "
            "t(pi((*a0).x0_parent_gobj)); t(pi((*a0).x4_parent_gobj2)); t((*a0).x44_flag.anon_0.b0() as u32);")
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
        "ftLib_80087074": ("if (a1) { a1->x = rw(); a1->y = rw(); a1->z = rw(); }",
                           "if !a1.is_null() { (*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw(); }"),
    })

add("itcoll", "src/melee/it/itcoll.c", custom_stubs=EF_CUSTOM, manual_data=["Fighter_804D6524"], null_ret_ok=["ftCommon_8007F804"],
    ret={"it_802746F8": ("&JO[rnd() % 8]", "&raw mut JO[(rnd() % 8) as usize]"),
         "ftLib_80086960": ("((fv() & 1) && a0 >= &GO[4] && a0 <= &GO[6])", "{ let v = fv() & 1; (v != 0 && a0 >= &raw mut GO[4] && a0 <= &raw mut GO[6]) as i32 }")},
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
         "ftLib_800870BC": ("({ *(u32*) a1 = fv(); (int) (fv() & 1); })", "{ *(a1 as *mut u32) = fv(); (fv() & 1) as i32 }")},
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
             "ftLib_8008770C": ("*(u32*) a1 = fv();", "*(a1 as *mut u32) = fv();"),
             "ftLib_80087744": ("*(u32*) a1 = fv();", "*(a1 as *mut u32) = fv();")})
add("it_26B1", "src/melee/it/it_26B1.c", skip_freg=["RunCallbackUnk"], skip_stubs=["__frsqrte"], skip_data=["guess", "y", "item_data"],
    xpools=[("BobOmbRain", "BOR", 2), ("ItemStateArray", "STA", 2), ("HSD_Joint", "JT", 2)],
    manual_data=["it_804D6D24", "it_804D6D38"],
    null_ret_ok=["ftLib_8008627C"],
    ret={"ftLib_8008627C": ("pick_fgo()", "pick_fgo()"), "ftLib_80086368": ("pick_fgo()", "pick_fgo()"),
         "ftLib_80086984": ("pick_CollData()", "pick_CollData()"),
         "it_80272D40": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "itFlipper_Spawn": ("pick_igo()", "pick_igo()"), "itSword_Spawn": ("pick_igo()", "pick_igo()"),
         "it_8027D670": ("pick_igo()", "pick_igo()"), "it_80283AE4": ("pick_igo()", "pick_igo()"), "it_802841B4": ("pick_igo()", "pick_igo()"),
         "it_80284854": ("pick_igo()", "pick_igo()"), "it_80286088": ("pick_igo()", "pick_igo()"), "it_802896CC": ("pick_igo()", "pick_igo()"),
         "it_80291BE0": ("pick_igo()", "pick_igo()"), "it_80292D48": ("pick_igo()", "pick_igo()"), "it_80294DC0": ("pick_igo()", "pick_igo()")})
add("it_2725", "src/melee/it/it_2725.c")
add("it_279C", "src/melee/it/it_279C.c")
