#!/usr/bin/env python3
"""Stub generator for the melee_modes TU state oracles.

For one game TU (libclang, the cref flags) it finds every function the TU's own code calls but
does not define (also through the `static inline` header helpers it uses) and every extern
global, then writes:
  * C stubs   (BUILD/<stem>_stubs.inc): each prints "Name arg arg..." and returns a value from the
    shared seeded stream (spec below), so the C driver and the Rust test see the same call trace;
  * Rust ext  (src/tu_ext.rs, one file for all TUs): the `pub mod` of settable hooks, named like the
    C function; unset hooks panic "TODO ... not ported";
  * Rust test stubs (tests/stubs/<stem>.rs): the same trace/return logic as the C stubs, and
    `install()` that sets the hooks.
Hand-written pieces (variadic / out-parameter callees) are listed in CUSTOM and get no stub.

Return specs (CONFIG["ret"][name]): "z" zero/NULL | ("ri", N) rnd()%N | ("rf",) | ("c", lit) |
("alloc",) fresh zeroed arena object of the pointee | ("allocn", n) | ("s", "text").
Effects (CONFIG["effect"][name]) = (C stmts, Rust stmts) run after the trace; args are a0, a1...
"""
import os, re, sys
from pathlib import Path
import clang.cindex as ci

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
BUILD = ROOT / "rust/target/cref"
CI = ci.CursorKind
TK = ci.TypeKind
sys.path.insert(0, str(HERE))
import calls_common as cc  # noqa: E402

SCALARS = {
    TK.CHAR_S: ("i8", "int"), TK.SCHAR: ("i8", "int"), TK.UCHAR: ("u8", "int"), TK.CHAR_U: ("u8", "int"),
    TK.SHORT: ("i16", "int"), TK.USHORT: ("u16", "int"), TK.INT: ("i32", "int"), TK.UINT: ("u32", "uint"),
    TK.LONG: ("i32", "int"), TK.ULONG: ("u32", "uint"), TK.LONGLONG: ("i64", "int"), TK.ULONGLONG: ("u64", "uint"),
    TK.FLOAT: ("f32", "f"), TK.DOUBLE: ("f64", "d"), TK.BOOL: ("bool", "int"), TK.ENUM: ("i32", "int"),
}


def strip(sp):
    sp = sp.replace("const ", "").replace("struct ", "").replace("union ", "").replace("enum ", "").strip()
    return sp


def rust_ptee(t):
    c = t.get_canonical()
    if c.kind == TK.VOID:
        return "core::ffi::c_void"
    if c.kind in (TK.CHAR_S, TK.SCHAR, TK.CHAR_U) and strip(t.spelling) == "char":
        return "core::ffi::c_char"
    return rust_type(t)


def rust_type(t):
    c = t.get_canonical()
    if c.kind == TK.VOID:
        return "()"
    if c.kind == TK.POINTER:
        pt = c.get_pointee()
        if pt.get_canonical().kind in (TK.FUNCTIONPROTO, TK.FUNCTIONNOPROTO):
            return strip(t.spelling)  # the typedef (GObj_RenderFunc, HSD_GObjEvent ...)
        tp = c.get_pointee() if t.kind != TK.POINTER else t.get_pointee()
        # keep the spelled pointee when t is a plain pointer (typedef names of structs survive)
        if t.kind == TK.TYPEDEF:
            tp = c.get_pointee()
        const = tp.is_const_qualified()
        return ("*const " if const else "*mut ") + rust_ptee(tp)
    if c.kind in SCALARS:
        return SCALARS[c.kind][0]
    if c.kind == TK.RECORD:
        return strip(t.spelling)
    if c.kind == TK.CONSTANTARRAY:
        return f"[{rust_type(c.element_type)}; {c.element_count}]"
    raise ValueError("type " + t.spelling)


def cls(t):
    c = t.get_canonical()
    if c.kind == TK.VOID:
        return "void"
    if c.kind == TK.POINTER:
        pt = c.get_pointee().get_canonical()
        if pt.kind in (TK.FUNCTIONPROTO, TK.FUNCTIONNOPROTO):
            return "fn"
        if pt.kind in (TK.CHAR_S, TK.SCHAR, TK.CHAR_U):
            return "str"
        return "ptr"
    if c.kind in SCALARS:
        return SCALARS[c.kind][1]
    raise ValueError("class " + t.spelling)


def c_decl_args(fn):
    return [(a.type, f"a{i}") for i, a in enumerate(fn.get_arguments())]


def ret_expr(rtype, spec, lang):
    c = cls(rtype)
    if c == "void":
        return None
    if spec is None:
        spec = {"int": ("ri", 3), "uint": ("ri", 3), "f": ("rf",), "d": ("rf",), "ptr": ("alloc",), "str": ("s", ""), "fn": "z"}[c]
    if spec == "z":
        if lang == "c":
            return "0" if c not in ("f", "d") else "0.0f"
        return {"int": "0", "uint": "0", "f": "0.0", "d": "0.0", "ptr": "core::ptr::null_mut()", "fn": "None", "str": "core::ptr::null()"}[c]
    k = spec[0]
    if lang == "c":
        ct = rtype.spelling
        if k == "ri":
            return f"({ct}) (rnd() % {spec[1]})"
        if k == "rf":
            return f"({ct}) rf()"
        if k == "c":
            return f"({ct}) ({spec[1]})"
        if k == "alloc":
            pt = rtype.get_canonical().get_pointee()
            return f"({ct}) AZ(sizeof({pt.spelling.replace('const ', '')}))" if pt.get_canonical().kind != TK.VOID else f"({ct}) AZ(64)"
        if k == "allocn":
            return f"({ct}) AZ({spec[1]})"
        if k == "allocr":
            return f"({ct}) AZR({spec[1]})"
        if k == "expr":
            return f"({ct}) ({spec[1]})"
        if k == "s":
            return f"({ct}) \"{spec[1]}\""
    else:
        rt = rust_type(rtype)
        if k == "ri":
            return f"(rnd() % {spec[1]}) as {rt}"
        if k == "rf":
            return f"rf() as {rt}"
        if k == "c":
            return f"({spec[1]}) as {rt}"
        if k == "alloc":
            pt = rtype.get_canonical().get_pointee()
            if pt.get_canonical().kind == TK.VOID:
                return f"az(64) as {rt}"
            return f"az(core::mem::size_of::<{rust_ptee(rtype.get_canonical().get_pointee())}>()) as {rt}"
        if k == "allocn":
            return f"az({spec[1]}) as {rt}"
        if k == "allocr":
            return f"azr({spec[1]}) as {rt}"
        if k == "expr":
            return f"({spec[2]}) as {rt}"
        if k == "s":
            return f"c\"{spec[1]}\".as_ptr() as {rt}"
    raise ValueError(spec)


def trace_c(t, name):
    c = cls(t)
    return {"int": f"TRI({name});", "uint": f"TRU({name});", "f": f"TRF({name});", "d": f"TRD({name});",
            "ptr": f"TRP({name});", "fn": f"TRP({name});", "str": f"TRS({name});"}[c]


def trace_rs(t, name):
    c = cls(t)
    return {"int": f"tri({name} as i64);", "uint": f"tru({name} as u64);", "f": f"trf({name});", "d": f"trd({name});",
            "ptr": f"trp({name} as *const u8);", "fn": f"trp(match {name} {{ Some(f) => f as usize as *const u8, None => core::ptr::null() }});",
            "str": f"trs({name} as *const core::ffi::c_char);"}[c]


def c_global(t, n, init):
    dims = ""
    while t.kind == TK.CONSTANTARRAY and not (t.spelling.isidentifier() or t.get_declaration().kind == CI.TYPEDEF_DECL):
        dims += f"[{t.element_count}]"
        t = t.element_type
    return f"{t.spelling} {n}{dims}" + (f" = {init}" if init else "") + ";"


def gen(stem, cfg, ana):
    """ana = calls_common.analyze(...) result. Returns (c_text, rust_ext_text, rust_stub_text)."""
    ret = cfg.get("ret", {})
    custom = set(cfg.get("custom", ()))
    effect = cfg.get("effect", {})
    ginit = cfg.get("ginit", {})
    c = [f"/* generated by stubgen.py for {stem} */"]
    rs_ext, rs_stub = [], []
    names = []
    for n, fn in sorted(ana["externs"].items()):
        if n in custom or n in SKIP:
            continue
        args = c_decl_args(fn)
        rt = fn.result_type
        var = fn.type.is_function_variadic()
        if var:
            raise SystemExit(f"{n}: variadic needs CUSTOM")
        try:
            for t, _ in args:
                rust_type(t)
            rust_type(rt)
        except ValueError as e:
            raise SystemExit(f"{n}: {e}")
        names.append(n)
        # ---- C
        sig = ", ".join(f"{t.spelling} {a}" for t, a in args) or "void"
        body = [f'printf("{n}");'] + [trace_c(t, a) for t, a in args] + ['printf("\\n");']
        if n in effect:
            body.append(effect[n][0])
        re_ = ret_expr(rt, ret.get(n), "c")
        if re_ is not None:
            body.append(f"return {re_};")
        c.append(f"{rt.spelling} {n}({sig}) {{ " + " ".join(body) + " }")
        # ---- Rust stub
        rsig = ", ".join(f"{a}: {rust_type(t)}" for t, a in args)
        rrt = rust_type(rt)
        rbody = [f'out("{n}");'] + [trace_rs(t, a) for t, a in args] + ['out("\\n");']
        if n in effect:
            rbody.append(effect[n][1])
        re2 = ret_expr(rt, ret.get(n), "rs")
        rbody.append(re2 if re2 is not None else "")
        rs_stub.append(f"    fn t_{n}({rsig}){'' if rrt == '()' else ' -> ' + rrt} {{ unsafe {{ {' '.join(rbody)} }} }}")
    for n in sorted(ana["globals"]):
        v = ana["globals"][n]
        if n in cfg.get("gcustom", ()):
            continue
        init = ginit.get(n)
        t = v.type
        c.append(c_global(t, n, init))
    return "\n".join(c) + "\n", names, rs_stub


CUSTOM_RS = {
    "lbArchive_80016DBC": """pub static mut HOOK_lbArchive_80016DBC: Option<fn(*const core::ffi::c_char, &[*mut core::ffi::c_void]) -> *mut HSD_Archive> = None;
/// C `lbArchive_80016DBC(file, dst0, name0, dst1, name1, ..., 0)` (melee/lb/lbarchive.h) is variadic:
/// `rest` is the flat (dst, name, dst, name, ...) list without the terminating 0.
pub fn lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    match unsafe { HOOK_lbArchive_80016DBC } { Some(f) => f(file, rest), None => panic!("TODO: lbArchive_80016DBC is not ported") }
}""",
    "lbArchive_LoadSymbols": """pub static mut HOOK_lbArchive_LoadSymbols: Option<fn(*const core::ffi::c_char, &[*mut core::ffi::c_void]) -> *mut HSD_Archive> = None;
/// C `lbArchive_LoadSymbols(file, dst0, name0, ..., 0)` (melee/lb/lbarchive.h), variadic like `lbArchive_80016DBC`.
pub fn lbArchive_LoadSymbols(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    match unsafe { HOOK_lbArchive_LoadSymbols } { Some(f) => f(file, rest), None => panic!("TODO: lbArchive_LoadSymbols is not ported") }
}""",
    "lb_80011E24": """pub static mut HOOK_lb_80011E24: Option<fn(*mut HSD_JObj, *mut *mut HSD_JObj, i32, i32) -> i32> = None;
/// C `lb_80011E24(jobj, out, id, ...)` (melee/lb/lb_00F9.h): find joint `id`; the gm callers pass exactly `-1` after it.
pub fn lb_80011E24(jobj: *mut HSD_JObj, out: *mut *mut HSD_JObj, id: i32, end: i32) -> i32 {
    match unsafe { HOOK_lb_80011E24 } { Some(f) => f(jobj, out, id, end), None => panic!("TODO: lb_80011E24 is not ported") }
}""",
    "HSD_SisLib_803A6B98": """pub static mut HOOK_HSD_SisLib_803A6B98: Option<fn(*mut HSD_Text, f32, f32, *const core::ffi::c_char) -> i32> = None;
/// C `HSD_SisLib_803A6B98(text, x, y, fmt, ...)` (printf-like; the gm callers pass a plain string).
pub fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const core::ffi::c_char) -> i32 {
    match unsafe { HOOK_HSD_SisLib_803A6B98 } { Some(f) => f(text, x, y, fmt), None => panic!("TODO: HSD_SisLib_803A6B98 is not ported") }
}""",
    "HSD_SisLib_803A70A0": """pub static mut HOOK_HSD_SisLib_803A70A0: Option<fn(*mut HSD_Text, i32, *const core::ffi::c_char) -> i32> = None;
/// C `HSD_SisLib_803A70A0(text, index, fmt, ...)` (printf-like; the gm callers pass a plain string).
pub fn HSD_SisLib_803A70A0(text: *mut HSD_Text, index: i32, fmt: *const core::ffi::c_char) -> i32 {
    match unsafe { HOOK_HSD_SisLib_803A70A0 } { Some(f) => f(text, index, fmt), None => panic!("TODO: HSD_SisLib_803A70A0 is not ported") }
}""",
    "GXLoadPosMtxImm": """pub static mut HOOK_GXLoadPosMtxImm: Option<fn(*const [f32; 4], u32)> = None;
/// C `GXLoadPosMtxImm(Mtx, id)` (dolphin/gx).
pub fn GXLoadPosMtxImm(mtx: *const [f32; 4], id: u32) {
    match unsafe { HOOK_GXLoadPosMtxImm } { Some(f) => f(mtx, id), None => panic!("TODO: GXLoadPosMtxImm is not ported") }
}""",
    "GXSetProjection": """pub static mut HOOK_GXSetProjection: Option<fn(*const [f32; 4], i32)> = None;
/// C `GXSetProjection(Mtx44, type)` (dolphin/gx).
pub fn GXSetProjection(mtx: *const [f32; 4], ty: i32) {
    match unsafe { HOOK_GXSetProjection } { Some(f) => f(mtx, ty), None => panic!("TODO: GXSetProjection is not ported") }
}""",
    "MTXOrtho": """pub static mut HOOK_MTXOrtho: Option<fn(*mut [f32; 4], f32, f32, f32, f32, f32, f32)> = None;
/// C `MTXOrtho(Mtx44, t, b, l, r, n, f)` (dolphin/mtx; the port's `Mtx44` form, see fn_80185408).
pub fn MTXOrtho(mtx: *mut [f32; 4], t: f32, b: f32, l: f32, r: f32, n: f32, f: f32) {
    match unsafe { HOOK_MTXOrtho } { Some(h) => h(mtx, t, b, l, r, n, f), None => panic!("TODO: MTXOrtho is not ported") }
}""",
}
SKIP = {"__assert", "__frsqrte"}


def rust_ext_for(fns, values):
    """All extern functions of all TUs (name -> clang cursor) as hook shims."""
    out = []
    for n in sorted(fns):
        fn = fns[n]
        if n in SKIP:
            continue
        if n in CUSTOM_RS:
            out.append(CUSTOM_RS[n])
            continue
        args = c_decl_args(fn)
        rsig = ", ".join(f"{a}: {rust_type(t)}" for t, a in args)
        rrt = rust_type(fn.result_type)
        tys = ", ".join(rust_type(t) for t, _ in args)
        names = ", ".join(a for _, a in args)
        ret = "" if rrt == "()" else f" -> {rrt}"
        if n in values:
            out.append(f"""pub static mut HOOK_{n}: Option<fn({tys}){ret}> = None;
/// C `{n}` ({fn.location.file.name.replace(chr(92), '/').split('/src/')[-1] if fn.location.file else '?'}); referenced as a function value, so it is `extern "C"`.
pub unsafe extern "C" fn {n}({rsig}){ret} {{
    match HOOK_{n} {{ Some(f) => f({names}), None => panic!("TODO: {n} is not ported") }}
}}""")
        else:
            out.append(f"""pub static mut HOOK_{n}: Option<fn({tys}){ret}> = None;
/// C `{n}` ({fn.location.file.name.replace(chr(92), '/').split('/src/')[-1] if fn.location.file else '?'}).
pub fn {n}({rsig}){ret} {{
    match unsafe {{ HOOK_{n} }} {{ Some(f) => f({names}), None => panic!("TODO: {n} is not ported") }}
}}""")
    return "\n".join(out)
