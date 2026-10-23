#!/usr/bin/env python3
"""Print the `hook!` lines (rust/melee_world/src/tu/hookmac.rs) for every callee of a C TU that is
defined elsewhere, from the C prototypes (Rust types: the melee_types spelling where one exists).

  python rust/melee_world/gen/oracle/gen_ext.py src/melee/gr/grbattle.c [--skip NAME,NAME] [--inl]

--inl lists the callees the shared grinl module already hooks too (they are skipped by default)."""
import re, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_chara/gen/oracle"))
import oracle as O  # noqa: E402

K, T = O.K, O.T
GEN = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8")
KNOWN = set(re.findall(r"^pub (?:type|struct|enum|union) (\w+)", GEN, re.M))
INTS = {T.INT: "i32", T.UINT: "u32", T.SHORT: "i16", T.USHORT: "u16", T.UCHAR: "u8", T.SCHAR: "i8",
        T.CHAR_S: "i8", T.CHAR_U: "u8", T.LONG: "i32", T.ULONG: "u32", T.LONGLONG: "i64", T.ULONGLONG: "u64",
        T.FLOAT: "f32", T.DOUBLE: "f64", T.LONGDOUBLE: "f64", T.BOOL: "bool"}


def clean(sp):
    sp = re.sub(r"\b(const|volatile|struct|union|enum)\b", "", sp)
    return re.sub(r"\s+", " ", sp).strip()


def rs_type(t, depth=0):
    c = t.get_canonical()
    sp = clean(t.spelling)
    if c.kind == T.POINTER:
        p = c.get_pointee().get_canonical()
        if p.kind in (T.FUNCTIONPROTO, T.FUNCTIONNOPROTO):
            if sp in KNOWN:
                return sp
            args = ", ".join(rs_type(a, depth + 1) for a in p.argument_types()) if p.kind == T.FUNCTIONPROTO else ""
            r = "" if p.get_result().kind == T.VOID else f" -> {rs_type(p.get_result(), depth + 1)}"
            return f"Option<unsafe extern \"C\" fn({args}){r}>"
        if sp in KNOWN and "*" not in sp:
            return sp
        pt = t.get_pointee()
        if p.kind == T.VOID:
            return "*mut c_void"
        if p.kind in (T.CHAR_S, T.SCHAR, T.CHAR_U, T.UCHAR) and "const" in t.spelling:
            return "*const i8"
        inner = rs_type(pt, depth + 1)
        return f"*mut {inner}"
    if c.kind == T.VOID:
        return "()"
    if c.kind == T.RECORD:
        n = c.get_declaration().spelling
        return n or "c_void"
    if sp in KNOWN and c.kind != T.BOOL:
        return sp
    if c.kind == T.ENUM:
        et = c.get_declaration().enum_type.get_canonical().kind
        return "u32" if et in (T.UINT, T.ULONG) else "i32"
    if c.kind in INTS:
        return INTS[c.kind]
    if c.kind == T.CONSTANTARRAY:
        return f"*mut {rs_type(c.element_type, depth + 1)}"
    raise SystemExit(f"gen_ext: unhandled type {t.spelling} ({c.kind})")


def main():
    args = sys.argv[1:]
    skip = set()
    if "--skip" in args:
        i = args.index("--skip")
        skip = set(args[i + 1].split(","))
        del args[i:i + 2]
    src = args[0]
    includes = ["melee/gr/types.h", "melee/gr/forward.h", "melee/gr/ground.h", "sysdolphin/baselib/gobj.h", "sysdolphin/baselib/jobj.h"]
    srcpath = (ROOT / src).as_posix()
    pr = O.Probe(srcpath, includes)
    O.BUILD.mkdir(parents=True, exist_ok=True)
    head = [f'#include "{i}"' for i in includes] + ['#include "oracle.h"', f'#include "{srcpath}"']
    p1 = O.BUILD / "genext_p1.c"
    p1.write_text("\n".join(head) + "\nint main(void) { return 0; }\n", encoding="utf-8")
    obj1 = O.ship_obj(p1, "genext_p1")
    nm = [l.split()[-1] for l in O.run([O.NM, "--undefined-only", str(obj1)], "nm").stdout.splitlines() if " U " in " " + l.strip() + " "]
    undef = sorted({n for n in nm if not n.startswith("__imp_")})
    gk = O.gekko_obj()
    gk_defs = {l.split()[-1] for l in O.run([O.NM, "--defined-only", str(gk)], "nm").stdout.splitlines() if l.strip()}
    for n in undef:
        if n in skip or n in gk_defs or n in ("memcpy", "memset", "memmove", "memcmp", "strlen", "strcmp", "sqrt", "sqrtf", "fabs", "fabsf"):
            continue
        if n not in pr.funcs:
            print(f"    // VAR/UNKNOWN {n}")
            continue
        fn = pr.funcs[n]
        params = []
        for i, a in enumerate(fn.get_arguments()):
            params.append(f"{a.spelling or 'a' + str(i)}: {rs_type(a.type)}")
        if fn.type.kind == T.FUNCTIONPROTO and fn.type.is_function_variadic():
            params.append("va: &[usize]")
        ret = fn.result_type
        r = "" if ret.get_canonical().kind == T.VOID else f" -> {rs_type(ret)}"
        # the hook name: upper-case C name
        print(f"    hook!({n.upper()}, {n}, ({', '.join(params)}){r});")


if __name__ == "__main__":
    main()
