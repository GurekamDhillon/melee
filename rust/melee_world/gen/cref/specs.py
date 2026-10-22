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
for _s in SPECS.values():
    _s.setdefault("common_ret", COMMON_RET)
