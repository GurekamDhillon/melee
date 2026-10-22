"""Oracle emission for one TU: tu_part.c (C stubs, dumpers, tables, dispatch), the Rust test file,
and the scenario text. Called from gen.py."""
import re
from pathlib import Path

import model
from model import K, T, rust_ty
import gen as G

OUT = G.OUT
RT_DIR = Path(__file__).resolve().parent

CTYPE_BY_SIZE = {1: "unsigned char", 2: "unsigned short", 4: "unsigned int", 8: "unsigned long long"}


def c_decl_var(t, name):
    c = t.get_canonical()
    if c.kind == T.CONSTANTARRAY:
        return f"{c.element_type.spelling} {name}[{c.element_count}]"
    if c.kind == T.INCOMPLETEARRAY:
        return f"{c.element_type.spelling} {name}[1]"
    sp = t.spelling
    if "(*)" in sp:
        return sp.replace("(*)", f"(*{name})", 1)
    return f"{sp} {name}"


def mask_expr(t):
    return CTYPE_BY_SIZE[t.get_canonical().get_size()]


# ----------------------------------------------------------------------------- C side
def c_tr_arg(t, expr, fname, idx, oc):
    """C statement tracing one argument."""
    if (fname, idx) in oc.get("outarg", {}):
        return "tr_o();"
    if (fname, idx) in oc.get("logmem", {}):
        return f"tr_m((const void*) {expr}, {oc['logmem'][(fname, idx)]});"
    k = G.kind(t)
    if k == "i":
        return f"tr_i((unsigned long long) ({mask_expr(t)}) {expr});"
    if k == "f":
        return f"tr_f(__builtin_bit_cast(unsigned, (float) {expr}));"
    if k == "d":
        return f"tr_d(__builtin_bit_cast(unsigned long long, (double) {expr}));"
    if k == "s":
        return f"tr_s((const char*) {expr});"
    if k == "p":
        return f"tr_p((const void*) {expr});"
    if k == "r4":
        return f"tr_i((unsigned long long) __builtin_bit_cast(unsigned, {expr}));"
    raise ValueError(k)


def c_ret(f):
    k = G.kind(f.ret)
    n = f.name
    rt = f.ret.spelling
    if k == "v":
        return ""
    if k == "i":
        return f'return ({rt}) ret_i("{n}", {model.T and f.ret.get_canonical().get_size() * 8});'
    if k == "f":
        return f'return __builtin_bit_cast(float, ret_f("{n}"));'
    if k == "d":
        return f'return __builtin_bit_cast(double, ret_d("{n}"));'
    if k in ("p", "s"):
        return f'return ({rt}) ret_p("{n}");'
    if k == "r4":
        return f'return __builtin_bit_cast({rt}, (unsigned) ret_i("{n}", 32));'
    raise ValueError(k)


def c_stub(f, oc):
    ps = [(G.pname(n) if n else f"p{i}", t) for i, (n, t) in enumerate(f.params)]
    # unnamed or colliding names: use p<i>
    names = [f"p{i}" for i in range(len(ps))]
    decl = ", ".join(G.c_param(t, nm) for nm, (_, t) in zip(names, ps)) or "void"
    L = [f"{f.ret.spelling} {f.name}({decl})", "{"]
    L.append(f'    tr_begin("{f.name}");')
    for i, (nm, (_, t)) in enumerate(zip(names, ps)):
        L.append("    " + c_tr_arg(t, oc.get("argexpr", {}).get((f.name, i), nm), f.name, i, oc))
    cu = oc.get("custom", {}).get(f.name)
    if cu:
        L.append("    " + cu["c"].replace("\n", "\n    "))
    L.append("    tr_end();")
    r = c_ret(f)
    if r:
        L.append("    " + r)
    L.append("}")
    return "\n".join(L)


def c_dispatch_arg(t, i):
    k = G.kind(t)
    sp = t.spelling
    if k == "i":
        return f"({sp}) a[{i}]"
    if k == "f":
        return f"__builtin_bit_cast(float, (unsigned) a[{i}])"
    if k == "d":
        return f"__builtin_bit_cast(double, a[{i}])"
    if k in ("p", "s"):
        return f"({sp}) a[{i}]"
    if k == "r4":
        return f"__builtin_bit_cast({sp}, (unsigned) a[{i}])"
    raise ValueError(k)


def c_dispatch(defs, funcs):
    L = ["int s497_dispatch(const char* name, const unsigned long long* a, int na, unsigned long long* ret)", "{"]
    for n in funcs:
        f = defs[n]
        args = ", ".join(c_dispatch_arg(t, i) for i, (_, t) in enumerate(f.params))
        k = G.kind(f.ret)
        L.append(f'    if (s497_streq(name, "{n}")) {{')
        if k == "v":
            L.append(f"        {n}({args}); return 0;")
        elif k == "i":
            L.append(f"        *ret = (unsigned long long) ({mask_expr(f.ret)}) {n}({args}); return 1;")
        elif k == "f":
            L.append(f"        *ret = __builtin_bit_cast(unsigned, {n}({args})); return 1;")
        elif k in ("p", "s"):
            L.append(f"        *ret = (unsigned long long) {n}({args}); return 2;")
        else:
            raise ValueError(k)
        L.append("    }")
    L.append("    return -1;")
    L.append("}")
    return "\n".join(L)


def c_dumpers(tu, types):
    out, names = [], []
    for tn in types:
        t = model.find_type(tu, tn)
        ct = tn if t.kind == T.TYPEDEF or True else tn
        lv = G.leaves(t)
        # C spelling usable in a cast
        d = t.get_canonical().get_declaration()
        spell = t.spelling
        out.append(G.c_dumper(tn, spell, lv, t.get_canonical().get_size()))
        names.append((tn, t.get_canonical().get_size(), G.c_bffill(tn, spell, lv)))
    L = out
    L.append("static unsigned long long s497_next(unsigned long long* st) { unsigned long long z = (*st += 0x9E3779B97F4A7C15ULL); z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL; z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL; return z ^ (z >> 31); }")
    bfs = []
    for tn, sz, bf in names:
        if bf:
            L.append(bf)
            bfs.append((tn, sz))
    L.append("int s497_bffill(const char* type, unsigned char* base, int count, unsigned long long seed)")
    L.append("{")
    L.append("    int i;")
    for tn, sz in bfs:
        L.append(f'    if (s497_streq(type, "{tn}")) {{ for (i = 0; i < count; i++) {{ unsigned long long st = seed + (unsigned long long) i * 0x1000; bffill_{tn}(base + (unsigned long long) i * {sz}, &st); }} return 0; }}')
    L.append("    return 0;")
    L.append("}")
    L.append("int s497_dump(const char* type, const unsigned char* base, int count)")
    L.append("{")
    L.append("    int i;")
    for tn, sz, bf in names:
        L.append(f'    if (s497_streq(type, "{tn}")) {{ for (i = 0; i < count; i++) {{ d_i("#", (unsigned long long) i); dump_{tn}(base + (unsigned long long) i * {sz}); }} return 0; }}')
    L.append("    return -1;")
    L.append("}")
    return "\n".join(L)


def build_tu_part(stem, cfg, oc):
    oc = dict(oc, logmem=cfg.get("logmem", {}), outarg=cfg.get("outarg", {}))
    tu, defs, callees, ext_vars, own = model.load(cfg["path"])
    mine = model.mine()
    real = set(oc.get("real", [])) | set(cfg.get("skip_callees", []))
    L = ['/* generated by rust/melee_modes/oracle/s497/gen.py oracle; do not edit */', '#include "rt.h"']
    L += oc.get("pre", [])
    src = oc.get("include", cfg["path"])
    if oc.get("patch"):
        # the harness copy of the C file differs only where the console's big-endian byte reads are
        # spelled as value operations (see the notes in cfg.py); same line numbers, same __FILE__
        txt = (model.ROOT / cfg["path"]).read_text(encoding="utf-8")
        for a_, b_ in oc["patch"]:
            assert a_ in txt, a_
            txt = txt.replace(a_, b_)
        pp = G.OUT / stem / (stem + "_patched.c")
        pp.write_text('#line 1 "' + cfg["path"] + '"' + chr(10) + txt, encoding="utf-8", newline=chr(10))
        src = pp.as_posix()
    L.append(f'#include "{src}"')
    L.append(oc.get("c_pre_stubs", ""))
    L.append("")
    L.append("static int s497_streq(const char* a, const char* b) { while (*a && *a == *b) { a++; b++; } return *a == *b; }")
    # extern variable definitions
    for n, v in sorted(ext_vars.items()):
        if n in oc.get("no_define", ()):
            continue
        L.append(c_decl_var(v.type, n) + ";")
    L.append("")
    # stubs
    for n in sorted(callees):
        f = callees[n]
        if n in real:
            continue
        if f.variadic:
            L.append(model_variadic_c(n))
            continue
        L.append(c_stub(f, oc))
        L.append("")
    L.append("")
    L.append(oc.get("c_extra", ""))
    # dumpers + dispatch
    funcs = oc["funcs"]
    L.append(c_dumpers(tu, oc.get("dump_types", [])))
    L.append(c_dispatch(defs, funcs))
    # tables
    ft = []
    for n in sorted(defs):
        if n in oc.get("no_ftab", ()):
            continue
        ft.append(f'    {{ "{n}", (void*) {n} }},')
    for n in sorted(callees):
        f = callees[n]
        if f.variadic or n in real:
            continue
        ft.append(f'    {{ "{n}", (void*) {n} }},')
    for n in oc.get("tcbs", {}):
        ft.append(f'    {{ "{n}", (void*) {n} }},')
    L.append("typedef struct { const char* name; void* p; } S497FEnt;")
    L.append("const S497FEnt s497_ftab[] = {")
    L += ft
    L.append("};")
    L.append("const int s497_nf = sizeof(s497_ftab) / sizeof(s497_ftab[0]);")
    gt = []
    for gname, gfill in oc.get("globals", []):
        cn = oc.get("gc_name", {}).get(gname, gname)
        gt.append(f'    {{ "{gname}", (void*) &{cn}, sizeof({cn}), {1 if gfill else 0} }},')
    L.append("typedef struct { const char* name; void* p; unsigned long long size; int fill; } S497GEnt;")
    L.append("const S497GEnt s497_gtab[] = {")
    L += gt
    L.append("};")
    L.append("const int s497_ng = sizeof(s497_gtab) / sizeof(s497_gtab[0]);")
    return "\n".join(L) + "\n"


def model_variadic_c(n):
    import cfg
    return cfg.VARIADIC_C[n]


# ----------------------------------------------------------------------------- Rust side
def rs_dispatch_arg(t, i):
    k = G.kind(t)
    rt = rust_ty(t)
    c = t.get_canonical()
    if k == "i":
        return f"a[{i}] as {rt}"
    if k == "f":
        return f"f32::from_bits(a[{i}] as u32)"
    if k == "d":
        return f"f64::from_bits(a[{i}])"
    if k in ("p", "s"):
        if k == "p" and c.get_pointee().get_canonical().kind == T.FUNCTIONPROTO:
            return f"core::mem::transmute::<usize, {rt}>(a[{i}] as usize)"
        return f"a[{i}] as usize as {rt}"
    if k == "r4":
        return f"core::mem::transmute::<u32, {rt}>(a[{i}] as u32)"
    raise ValueError(k)


def rs_dispatch(stem, defs, funcs):
    L = ["unsafe fn dispatch(name: &str, a: &[u64]) -> Option<Ret> {", "    use melee_modes::tu::%s as m;" % stem, "    Some(match name {"]
    for n in funcs:
        f = defs[n]
        args = ", ".join(rs_dispatch_arg(t, i) for i, (_, t) in enumerate(f.params))
        k = G.kind(f.ret)
        if k == "v":
            L.append(f'        "{n}" => {{ m::{n}({args}); Ret::Void }}')
        elif k == "i":
            sz = f.ret.get_canonical().get_size()
            L.append(f'        "{n}" => Ret::Int(m::{n}({args}) as {G.RUNS[sz]} as u64),')
        elif k == "f":
            L.append(f'        "{n}" => Ret::Int(m::{n}({args}).to_bits() as u64),')
        else:
            L.append(f'        "{n}" => Ret::Ptr(m::{n}({args}) as usize as u64),')
    L.append("        _ => return None,")
    L.append("    })")
    L.append("}")
    return "\n".join(L)


def rs_retkind(callees, extra):
    L = ["fn retkind(name: &str) -> Rk {", "    match name {"]
    for n in sorted(callees):
        f = callees[n]
        if f.variadic:
            continue
        k = G.kind(f.ret)
        v = {"v": "Rk::Void", "f": "Rk::F", "d": "Rk::D", "p": "Rk::P", "s": "Rk::P"}.get(k)
        if k == "i":
            v = f"Rk::I({f.ret.get_canonical().get_size() * 8})"
        if k == "r4":
            v = "Rk::I(32)"
        L.append(f'        "{n}" => {v},')
    for n, v in extra.items():
        L.append(f'        "{n}" => {v},')
    L.append("        _ => Rk::Void,")
    L.append("    }")
    L.append("}")
    return "\n".join(L)


def rs_dumpers(tu, types):
    out = []
    names = []
    for tn in types:
        t = model.find_type(tu, tn)
        lv = G.leaves(t)
        rt = rust_ty(t) if G.bf_leaves(lv) else "()"
        out.append(G.rs_dumper(tn, rt, lv, t.get_canonical().get_size()))
        names.append((tn, t.get_canonical().get_size(), G.rs_bffill(tn, rt, lv)))
    L = out
    L.append("fn s497_next(st: &mut u64) -> u64 { *st = st.wrapping_add(0x9E3779B97F4A7C15); let mut z = *st; z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9); z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB); z ^ (z >> 31) }")
    bfs = []
    for tn, sz, bf in names:
        if bf:
            L.append(bf)
            bfs.append((tn, sz))
    L.append("unsafe fn bffill(ty: &str, base: *mut u8, count: usize, seed: u64) -> bool {")
    L.append("    match ty {")
    for tn, sz in bfs:
        L.append(f'        "{tn}" => {{ for i in 0..count {{ let mut st = seed.wrapping_add(i as u64 * 0x1000); bffill_{tn}(base.add(i * {sz}), &mut st); }} true }}')
    L.append("        _ => true,")
    L.append("    }")
    L.append("}")
    L.append("unsafe fn dump(ty: &str, base: *const u8, count: usize) -> bool {")
    L.append("    match ty {")
    for tn, sz, bf in names:
        L.append(f'        "{tn}" => {{ for i in 0..count {{ d_i("#", i as u64); dump_{tn}(base.add(i * {sz})); }} true }}')
    L.append("        _ => false,")
    L.append("    }")
    L.append("}")
    return "\n".join(L)


def build_rs_test(stem, cfg, oc):
    tu, defs, callees, ext_vars, own = model.load(cfg["path"])
    real = set(oc.get("real", [])) | set(cfg.get("skip_callees", []))
    mine = model.mine()
    funcs = oc["funcs"]
    L = [f"//! State oracle for src/melee/gm/{stem}.c (generated by rust/melee_modes/oracle/s497/gen.py oracle).",
         "//! The scenario file is run through the Rust port here and through the game's own C (compiled",
         "//! natively with the shipping contraction, see oracle/s497/build.sh) there; the traces must be",
         "//! byte for byte equal: every stubbed callee call with its arguments, every return value and a",
         "//! field-by-field dump of the objects the scenario names.",
         "#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_unsafe, unused_variables, unused_imports, dead_code, clippy::missing_safety_doc)]",
         "mod s497_rt;",
         "use core::sync::atomic::{AtomicBool, Ordering};",
         "use melee_modes::hook::A;",
         "use melee_types::*;",
         "use s497_rt::*;",
         "",
         f"const SCEN: &str = include_str!(\"data/s497_{stem}_scen.txt\");",
         f"const EXP: &str = include_str!(\"data/s497_{stem}_exp.txt\");",
         ""]
    L.append(rs_dumpers(tu, oc.get("dump_types", [])))
    L.append("")
    L.append(rs_dispatch(stem, defs, funcs))
    L.append("")
    L.append(rs_retkind(callees, oc.get("retkind_extra", {})))
    L.append("")
    L.append(oc.get("rs_extra", ""))
    L.append("")
    # custom behaviours
    L.append("unsafe fn custom(name: &str, args: &[A]) -> Option<u64> {")
    L.append("    match name {")
    for n, cu in oc.get("custom", {}).items():
        L.append(f'        "{n}" => {{')
        L.append(cu["rs"])
        L.append("        }")
    L.append("        _ => None,")
    L.append("    }")
    L.append("}")
    L.append("")
    # glue
    tm = f"melee_modes::tu::{stem}"
    L.append("fn glue() -> &'static Glue {")
    L.append("    static G: std::sync::OnceLock<Glue> = std::sync::OnceLock::new();")
    L.append("    G.get_or_init(|| unsafe {")
    L.append("        let mut ftab: std::vec::Vec<(&'static str, usize)> = vec![];")
    for n in sorted(defs):
        if n in oc.get("no_ftab", ()):
            continue
        L.append(f'        ftab.push(("{n}", {tm}::{n} as usize));')
    for n in sorted(callees):
        f = callees[n]
        if f.variadic or n in real:
            continue
        if n in mine and mine[n] != stem and n not in cfg.get("forward", {}):
            owner = mine[n]
            L.append(f'        ftab.push(("{n}", melee_modes::tu::{owner}::{n} as usize));')
        else:
            L.append(f'        ftab.push(("{n}", {tm}::ext::{n} as usize));')
    for n in oc.get("tcbs", {}):
        L.append(f'        ftab.push(("{n}", {n} as usize));')
    L.append("        let mut gtab: std::vec::Vec<(&'static str, usize, usize, bool)> = vec![];")
    for gname, gfill in oc.get("globals", []):
        path = oc.get("gr_name", {}).get(gname)
        if path is None:
            path = f"{tm}::ext::{gname}" if gname in ext_vars else f"{tm}::{gname}"
        L.append(f'        gtab.push(("{gname}", core::ptr::addr_of!({path}) as usize, core::mem::size_of_val(&*core::ptr::addr_of!({path})), {"true" if gfill else "false"}));')
    L.append("        Glue { ftab, gtab, dispatch, dump, bffill, retkind, custom }")
    L.append("    })")
    L.append("}")
    L.append("")
    for n, body in oc.get("tcbs", {}).items():
        L.append(body)
    L.append("")
    L.append(f"static INIT: AtomicBool = AtomicBool::new(false);")
    L.append("")
    L.append("#[test]")
    L.append(f"fn {stem}_matches_c() {{")
    L.append("    unsafe {")
    L.append(f"        {tm}::ext::HOOK = Some(hook);")
    for extra in oc.get("also_hook", []):
        L.append(f"        melee_modes::tu::{extra}::ext::HOOK = Some(hook);")
    L.append("    }")
    L.append("    let got = run(SCEN, glue());")
    L.append("    let n = got.lines().filter(|l| l.starts_with(\"== \")).count();")
    L.append("    assert!(n > 0, \"empty scenario\");")
    L.append("    if got != EXP {")
    L.append("        let tmp = std::env::temp_dir().join(\"s497_%s_got.txt\");" % stem)
    L.append("        let _ = std::fs::write(&tmp, &got);")
    L.append("        panic!(\"trace differs from the C (wrote {}): {}\", tmp.display(), first_diff(&got, EXP));")
    L.append("    }")
    L.append("}")
    return "\n".join(L) + "\n"
