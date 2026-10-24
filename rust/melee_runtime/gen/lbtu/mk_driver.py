#!/usr/bin/env python3
"""Writes both halves of the melee_runtime::tu lb state oracle from one description, so the C driver
(gen/lbtu/tu/<stem>.c) and the Rust test (tests/lbtu_<stem>.rs) cannot drift apart:

  python rust/melee_runtime/gen/lbtu/mk_driver.py <stem> [...]      (worktree root)

Per TU, specs.py lists the oracle spec (stubs / effects / types) and cfg.py the *world*: static arrays of
records (`world`), redrawn field by field from the seeded generator on every case (the generated fill_<T>),
linked by the TU's `setup` snippet (C, Rust). Each function of the TU (statics included, header inlines
left out) becomes one op whose arguments are drawn by parameter type (`argmap` maps a pointer-to-record
type to world objects) or by hint. After the call the whole world, the return value and the call trace
of the recording stubs are hashed. Then
  python rust/melee_runtime/gen/lbtu/run_tu.py <stem>      (C reference output -> tests/data/lbtu_<stem>.txt)
  cargo test -p melee_runtime --test lbtu_<stem>
"""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_tu  # noqa: E402
from gen_tu import ci, K, T, canon, Tu  # noqa: E402

N_CASES = 400


def classify(t):
    """-> kind of a parameter / return type."""
    c = canon(t)
    k = c.kind
    if k == T.POINTER:
        if gen_tu.is_fnptr(t):
            return "fnptr"
        p = canon(c.get_pointee())
        nm = p.get_declaration().spelling
        if nm:
            return "ptr:" + nm
        return "ptr:" + c.get_pointee().spelling
    if k == T.CONSTANTARRAY:
        return "ptr:" + canon(c.element_type).spelling
    if k == T.VOID:
        return "void"
    if k == T.BOOL:
        return "bool"
    if k == T.FLOAT:
        return "f32"
    if k == T.DOUBLE:
        return "f64"
    if k == T.ENUM:
        return "int"
    if k in (T.INT, T.SHORT, T.CHAR_S, T.SCHAR, T.LONG, T.LONGLONG):
        return "int" if c.get_size() <= 4 else "i64"
    if k in (T.UINT, T.USHORT, T.UCHAR, T.CHAR_U, T.ULONG, T.ULONGLONG):
        return "uint" if c.get_size() <= 4 else "u64"
    return "other:" + str(k)


def is_inline(fn):
    toks = [x.spelling for x in fn.get_tokens()]
    head = toks[: toks.index("(")] if "(" in toks else toks
    return "inline" in head


def build_ops(tu, cfg):
    ops = []
    over = cfg.get("over", {})
    hints = cfg.get("hints", {})
    argmap = cfg.get("argmap", {})
    for name in sorted(tu.defs, key=lambda n: tu.defs[n].location.offset):
        fn = tu.defs[name]
        if name in cfg.get("skip_ops", ()):
            continue
        if is_inline(fn) and name not in over:
            continue
        if name in over:
            ops.append((name, over[name][0], over[name][1]))
            continue
        ft = canon(fn.type)
        params = list(fn.get_arguments())
        cnt = {}
        decl_c, decl_r, args = [], [], []
        for i, p in enumerate(params):
            kind = classify(p.type)
            pn = p.spelling
            key = name + ":" + pn
            if key in hints:
                d_c, d_r = hints[key]
                ct = p.type.spelling
                rty = gen_tu.rust_ty(p.type)
                decl_c.append(f"{ct} v{i} = ({ct}) ({d_c});")
                decl_r.append(f"let v{i}: {rty} = {d_r};")
                args.append((f"v{i}", f"v{i}"))
                continue
            if kind.startswith("ptr:"):
                nm = kind[4:]
                lst = argmap.get(nm)
                if lst is None:
                    raise SystemExit(f"{name}: parameter {pn} ({p.type.spelling}) needs an argmap/hint entry for '{nm}'")
                n = cnt[nm] = cnt.get(nm, 0)
                cnt[nm] += 1
                a_c, a_r = lst[n % len(lst)]
                args.append((a_c, a_r))
                continue
            if pn in hints:
                d_c, d_r = hints[pn]
            elif kind == "int":
                d_c, d_r = "(int) fv()", "fv() as i32"
            elif kind == "uint":
                d_c, d_r = "fv()", "fv()"
            elif kind == "i64":
                d_c, d_r = "(long long) fv64()", "fv64() as i64"
            elif kind == "u64":
                d_c, d_r = "fv64()", "fv64()"
            elif kind == "f32":
                d_c, d_r = "rf()", "rf()"
            elif kind == "bool":
                d_c, d_r = "(fv() & 1)", "fv() & 1 != 0"
            else:
                raise SystemExit(f"{name}: parameter {pn} ({kind}) needs a hint")
            ct = p.type.spelling
            rty = gen_tu.rust_ty(p.type)
            decl_c.append(f"{ct} v{i} = ({ct}) ({d_c});")
            if kind == "bool":
                decl_r.append(f"let v{i}: bool = {d_r};")
            else:
                decl_r.append(f"let v{i}: {rty} = ({d_r}) as {rty};")
            args.append((f"v{i}", f"v{i}"))
        rk = classify(ft.get_result())
        call_c = f"{name}({', '.join(a[0] for a in args)})"
        call_r = f"m::{name}({', '.join(a[1] for a in args)})"
        if name in cfg.get("noret", ()) or rk == "void":
            sc, sr = f"{call_c};", f"{call_r};"
        elif rk in ("int", "uint", "bool"):
            sc, sr = f"ret = (u32) {call_c};", f"ret = {call_r} as u32;"
        elif rk in ("i64", "u64"):
            sc, sr = f"{{ u64_t_ r64 = (u64_t_) {call_c}; ret = (u32) r64 ^ (u32) (r64 >> 32); }}", \
                     f"{{ let r64 = {call_r} as u64; ret = (r64 as u32) ^ ((r64 >> 32) as u32); }}"
        elif rk == "f32":
            sc, sr = f"ret = B({call_c});", f"ret = ({call_r}).to_bits();"
        elif rk.startswith("ptr"):
            sc, sr = f"ret = PI({call_c});", f"ret = pi({call_r});"
        else:
            raise SystemExit(f"{name}: return {rk}")
        pre_c, pre_r = cfg.get("pre", {}).get(name, ("", ""))
        ops.append((name, pre_c + " " + " ".join(decl_c) + " " + sc, pre_r + " " + " ".join(decl_r) + " " + sr))
    ops.extend(cfg.get("extra_ops", []))
    return ops


def world_decls(cfg):
    c, r = [], []
    for ct, nm, cnt in cfg["world"]:
        c.append(f"static {ct} {nm}[{cnt}];")
        r.append(f"static mut {nm}: [{cfg.get('rtype', {}).get(ct, ct)}; {cnt}] = unsafe {{ core::mem::zeroed() }};")
    return "\n".join(c), "\n".join(r)


def emit(stem):
    from specs import SPECS
    from cfg import CFG
    spec = SPECS[stem]
    cfg = CFG[stem]
    tu = Tu(gen_tu.patched_source(stem, spec), spec.get("args", ()), (ROOT / spec["src"]).parent)
    ops = build_ops(tu, cfg)
    nops = len(ops)
    wc, wr = world_decls(cfg)
    recs = [(ct, nm, cnt) for ct, nm, cnt in cfg["world"] if ct in cfg.get("types_fill", spec["types"])]
    # ---- C
    fill_c = "\n".join(f"    for (i = 0; i < {cnt}; i++) fill_{ct}(&{nm}[i]);" for ct, nm, cnt in recs)
    zero_c = "\n".join(f"    zero({nm}, sizeof {nm});" for ct, nm, cnt in cfg["world"])
    hash_c = "\n".join(f"    for (i = 0; i < {cnt}; i++) hash_{ct}(&{nm}[i]);" for ct, nm, cnt in recs)
    preg_c = "\n".join(f"    preg({nm}, sizeof {nm});" for ct, nm, cnt in cfg["world"])
    for extra in cfg.get("c_preg", []):
        preg_c += "\n    " + extra
    c = [f"/* TU: {spec['src']} */",
         "/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a",
         " * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */",
         "#include TU_SRC", '#include "mtu_common.h"', '#include "lbtu_common.h"', wc, cfg.get("c_decl", ""),
         f'#include "{stem}_gen.h"', ""]
    c.append("static void setup(void)\n{\n    int i, k;\n    (void) i; (void) k;\n" + zero_c + "\n" + fill_c + "\n" + cfg.get("setup", ("", ""))[0] + "\n}")
    c.append("static void hash_all(void)\n{\n    int i, k;\n    (void) i; (void) k;\n" + cfg.get("ptrs", ("", ""))[0] + "\n" + hash_c + "\n" + cfg.get("hash", ("", ""))[0] + "\n}")
    c += ["int main(void)", "{", "    int op, k;", preg_c, "    freg_all();"]
    c.append(cfg.get("main_init", ("", ""))[0])
    c += [f"    for (op = 0; op < {nops}; op++) {{", f"        for (k = 0; k < {N_CASES}; k++) {{", "            u32 ret = 0;",
          "            tu_case(op, k);", "            setup();", "            switch (op) {"]
    for i, (n, sc, sr) in enumerate(ops):
        c.append(f"            case {i}: {{ {sc} }} break; /* {n} */")
    c += ["            }", "            hash_all();", '            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR);',
          "        }", "    }", "    return 0;", "}", ""]
    (HERE / "tu" / f"{stem}.c").write_text("\n".join(c), encoding="utf-8", newline="\n")
    # ---- Rust
    fill_r = "\n".join(f"    for i in 0..{cnt} {{ g::fill_{ct}(&mut {nm}[i]); }}" for ct, nm, cnt in recs)
    zero_r = "\n".join(f"    {nm} = core::mem::zeroed();" for ct, nm, cnt in cfg["world"])
    hash_r = "\n".join(f"    for i in 0..{cnt} {{ g::hash_{ct}(&{nm}[i]); }}" for ct, nm, cnt in recs)
    preg_r = "\n".join(f"        preg(&raw const {nm}, core::mem::size_of_val(&{nm}));" for ct, nm, cnt in cfg["world"])
    for extra in cfg.get("rs_preg", []):
        preg_r += "\n        " + extra
    r = [f"//! Written by gen/lbtu/mk_driver.py from the same description as gen/lbtu/tu/{stem}.c.",
         f"//! src/tu/{stem}.rs vs the C (the whole TU, shipping contraction; regenerate the expected file",
         f"//! with `python rust/melee_runtime/gen/lbtu/run_tu.py {stem}`).",
         "#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens)]",
         '#[path = "lbtu_harness/mod.rs"]', "mod mtu_harness;", "use core::ptr::null_mut;", "use melee_types::*;",
         f"use melee_runtime::tu::{stem} as m;", "use mtu_harness::*;", f'#[path = "lbtu_gen/{stem}_gen.rs"]', "mod g;", "",
         wr, cfg.get("rs_decl", ""), ""]
    r.append("unsafe fn setup() {\n    let _ = 0;\n" + zero_r + "\n" + fill_r + "\n" + cfg.get("setup", ("", ""))[1] + "\n}")
    r.append("unsafe fn hash_all() {\n" + cfg.get("ptrs", ("", ""))[1] + "\n" + hash_r + "\n" + cfg.get("hash", ("", ""))[1] + "\n}")
    r += ["#[test]", f"fn {stem}_matches_c() {{", "    unsafe {", "        g::install();", preg_r, "        g::freg_all();",
          cfg.get("main_init", ("", ""))[1],
          f'        let mut c = Check::new(include_str!("data/lbtu_{stem}.txt"));', f"        for op in 0..{nops}u32 {{",
          f"            for k in 0..{N_CASES}u32 {{", "                case(op, k);", "                setup();",
          "                let mut ret = 0u32;", "                match op {"]
    for i, (n, sc, sr) in enumerate(ops):
        r.append(f"                    {i} => {{ {sr} }} // {n}")
    r += ["                    _ => unreachable!(),", "                }", "                hash_all();", "                c.line(op, k, ret);",
          "            }", "        }", f"        assert_eq!(c.done(), {nops * N_CASES});", "    }", "}", ""]
    (ROOT / "rust/melee_runtime/tests" / f"lbtu_{stem}.rs").write_text("\n".join(r), encoding="utf-8", newline="\n")
    print(f"mk_driver: {stem}: {nops} ops x {N_CASES} cases")


if __name__ == "__main__":
    for s in sys.argv[1:]:
        emit(s)
