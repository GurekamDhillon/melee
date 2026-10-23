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
