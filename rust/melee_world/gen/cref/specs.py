"""Per-TU oracle specs for gen_tu.py / run_tu.py (see gen_tu.py).

Keys: src, types (state records to fill / hash), unions {union type: [member views]} (USEL picks the
view that is filled / hashed), ret {callee: (C expr, Rust expr)} (a stub's return value, in terms of
its args a0..), effects {callee: (C stmts, Rust stmts)} (writes through out-parameters, run before the
return), null_ret_ok, data_size {extern array: n}, patch [(old, new)] (literal edits of the compiled
copy of the TU, each must match once), custom_stubs, skip_data."""
COMMON_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
}
SPECS = {}
SPECS["grfzerocar"] = dict(
    src="src/melee/gr/grfzerocar.c", types=["HSD_GObj", "HSD_JObj", "grFZeroCarEntry"],
    custom_stubs={
        "HSD_ForeachAnim": dict(
            c="void HSD_ForeachAnim(void* a0, HSD_Type a1, HSD_TypeMask a2, void* a3, AObj_Arg_Type a4, ...) { "
              "__builtin_va_list ap; double d; __builtin_va_start(ap, a4); d = __builtin_va_arg(ap, double); "
              "__builtin_va_end(ap); T(@ID@); T(PI(a0)); T((u32) a1); T((u32) a2); T(PI(a3)); T((u32) a4); T64(d); }",
            ext="fn HSD_ForeachAnim(a0: *mut c_void, a1: i32, a2: u32, a3: *mut c_void, a4: i32, a5: f64);",
            hook="|a0, a1, a2, a3, a4, a5| { t(@ID@); t(pi(a0)); t(a1 as u32); t(a2); t(pi(a3)); t(a4 as u32); t64(a5); }"),
    },
    ret={
        "Ground_801C13D0": ("((a0 < 100 && (fv() & 7) == 0) ? 0 : &JP[JN++ & 63])", "if a0 < 100 && fv() & 7 == 0 { core::ptr::null_mut() } else { let i = super::JN; super::JN += 1; &raw mut super::JP[i & 63] }"),
        "Ground_801C4100": ("(((fv() & 3) == 0) ? 0 : &JP[JN++ & 63])", "if fv() & 3 == 0 { core::ptr::null_mut() } else { let i = super::JN; super::JN += 1; &raw mut super::JP[i & 63] }"),
        "HSD_JObjAlloc": ("&JP[JN++ & 63]", "{ let i = super::JN; super::JN += 1; &raw mut super::JP[i & 63] }"),
        "grDatFiles_801C6330": ("&ARCH", "&raw mut super::ARCH"),
    })
SPECS["grgarden"] = dict(
    src="src/melee/gr/grgarden.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "Vec", "HSD_Generator",
           "HSD_psAppSRT", "grGarden_YakumonoParam"],
    unions={"GroundVars": ["garden", "garden2"]},
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04"],
    ret={
        "Ground_801C3FA4": ("&JO[a1 & 7]", "&raw mut super::JO[(a1 & 7) as usize]"),
        "Ground_GetStageGObj": ("(((fv() & 7) == 0) ? 0 : &GS[a0 & 7])",
                                "if fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 & 7) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "Mex_GrBind": ("a0", "a0"),
        "grLib_801C96F8": ("(((fv() & 3) == 0) ? 0 : &GEN)", "if fv() & 3 == 0 { core::ptr::null_mut() } else { &raw mut super::GEN }"),
        "grMaterial_801C8CFC": ("&GI", "&raw mut super::GI"),
        "ftLib_80086B80": ("rw()", "rw()"),
        "Stage_80225194": ("((fv() & 1) ? 62 : fv())", "if fv() & 1 != 0 { 62 } else { fv() }"),
        "gm_8016B238": ("((fv() & 1) ? 0 : fv())", "if fv() & 1 != 0 { 0 } else { fv() as i32 }"),
    },
    effects={
        "ftLib_80086644": ("a1->x = rw(); a1->y = rw(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
        "ftLib_80086684": ("a1->x = rw(); a1->y = rw(); a1->z = rw();", "(*a1).x = rw(); (*a1).y = rw(); (*a1).z = rw();"),
        "lb_8000B1CC": ("a2->x = rw(); a2->y = rw(); a2->z = rw();", "(*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw();"),
    })
for _s in SPECS.values():
    _s.setdefault("common_ret", COMMON_RET)
