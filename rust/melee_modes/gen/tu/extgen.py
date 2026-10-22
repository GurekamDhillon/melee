#!/usr/bin/env python3
"""Describe a game TU for the port: its function definitions, the callees it uses but does not
define (as `ext_fns!` text), and the globals it touches.

  python rust/melee_modes/gen/tu/extgen.py src/melee/gm/gmresult.c [--defs] [--ext] [--globals]

Uses libclang with the same flags as the oracle build (x86_64 msvc target), so the signatures are
the ones the native C has. Rust types: the melee_types name when it exists, else the primitive.
Where a callee already has a Rust port somewhere under rust/*/src (not raw/), a `// ported:` hint
lists the file.
"""
import os, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run as cref  # noqa

import clang.cindex as ci

LLVM = os.environ.get("LIBCLANG_PATH", r"C:\Program Files\LLVM\bin")
for cand in (LLVM, r"C:\Program Files\LLVM\bin"):
    p = Path(cand) / "libclang.dll"
    if p.exists():
        ci.Config.set_library_file(str(p))
        break

K = ci.TypeKind
PRIM = {K.VOID: "()", K.BOOL: "bool", K.CHAR_S: "i8", K.SCHAR: "i8", K.UCHAR: "u8", K.CHAR_U: "u8", K.SHORT: "i16",
        K.USHORT: "u16", K.INT: "i32", K.UINT: "u32", K.LONG: "i32", K.ULONG: "u32", K.LONGLONG: "i64",
        K.ULONGLONG: "u64", K.FLOAT: "f32", K.DOUBLE: "f64", K.LONGDOUBLE: "f64"}


def known_names():
    names = set()
    src = (ROOT / "rust/melee_types/src/gen.rs").read_text(encoding="utf-8", errors="replace")
    names |= set(re.findall(r"^pub (?:struct|type|enum|union) (\w+)", src, re.M))
    names |= {"Vec3", "Vec2"}
    return names


KNOWN = known_names()
UNKNOWN = set()


def rs_type(t, ctx_const=False):
    # typedef name known to melee_types (but not the plain integer typedefs, mapped structurally)
    spell = t.spelling.replace("const ", "").replace("struct ", "").replace("enum ", "").replace("union ", "").strip()
    if spell in KNOWN:
        return spell
    c = t.get_canonical()
    if c.kind in PRIM:
        return PRIM[c.kind]
    if c.kind == K.ENUM:
        return "u32"
    if c.kind == K.POINTER:
        pt = c.get_pointee()
        const = pt.is_const_qualified()
        pc = pt.get_canonical()
        if pc.kind == K.FUNCTIONPROTO or pc.kind == K.FUNCTIONNOPROTO:
            return "Option<" + fn_ptr(pc) + ">"
        if pc.kind == K.VOID:
            inner = "c_void"
        else:
            # use the declared (non-canonical) spelling to keep melee_types names
            decl_pt = t.get_pointee() if t.kind == K.POINTER else pt
            if decl_pt.kind == K.TYPEDEF or decl_pt.kind == K.ELABORATED:
                inner = rs_type(decl_pt)
            else:
                inner = rs_type(pt)
            if inner in ("()",):
                inner = "c_void"
        return ("*const " if const else "*mut ") + inner
    if c.kind == K.RECORD:
        n = c.spelling.replace("struct ", "").replace("union ", "")
        if n in KNOWN:
            return n
        UNKNOWN.add(n)
        return n
    if c.kind in (K.CONSTANTARRAY, K.INCOMPLETEARRAY):
        el = c.element_type
        return f"*mut {rs_type(el)}"
    return f"/*{c.kind}*/ u32"


def fn_ptr(pc):
    args = ", ".join(rs_type(a) for a in pc.argument_types()) if pc.kind == K.FUNCTIONPROTO else ""
    r = rs_type(pc.get_result())
    s = f"unsafe extern \"C\" fn({args})"
    if r != "()":
        s += f" -> {r}"
    return s


def parse(src):
    idx = ci.Index.create()
    flags = [f for f in cref.FLAGS if f not in ("-S", "-emit-llvm", "-Xclang", "-disable-llvm-passes")]
    # drop the 'one-arg' leftovers of -Xclang pairs
    flags = [f for f in flags if f != "-O2"]
    tu = idx.parse(str(ROOT / src), args=flags + ["-ferror-limit=0"],
                   options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    return tu


def main():
    src = sys.argv[1]
    want = set(a for a in sys.argv[2:] if a.startswith("--"))
    if not want:
        want = {"--defs", "--ext", "--globals"}
    tu = parse(src)
    here = (ROOT / src).resolve()
    defs, called, gvars = {}, {}, {}
    defined_names = set()

    def in_main(c):
        return c.location.file and Path(c.location.file.name).resolve() == here or (
            c.location.file and Path(c.location.file.name).suffix == ".inc")

    def walk_fn(c, owner):
        for n in c.walk_preorder():
            if n.kind == ci.CursorKind.DECL_REF_EXPR:
                d = n.referenced
                if d is None:
                    continue
                if d.kind == ci.CursorKind.FUNCTION_DECL:
                    called.setdefault(d.spelling, d)
                elif d.kind == ci.CursorKind.VAR_DECL and d.storage_class != ci.StorageClass.NONE or (
                        d.kind == ci.CursorKind.VAR_DECL and d.semantic_parent.kind == ci.CursorKind.TRANSLATION_UNIT):
                    gvars.setdefault(d.spelling, (d, set()))[1].add(owner)

    for c in tu.cursor.get_children():
        if c.kind == ci.CursorKind.FUNCTION_DECL and c.is_definition() and in_main(c):
            defs[c.spelling] = c
            defined_names.add(c.spelling)
            walk_fn(c, c.spelling)
        elif c.kind == ci.CursorKind.VAR_DECL and c.is_definition() and in_main(c):
            gvars.setdefault(c.spelling, (c, set()))
            # initialisers may reference functions (tables)
            walk_fn(c, "<init " + c.spelling + ">")

    ported = {}
    for p in (ROOT / "rust").glob("*/src/**/*.rs"):
        if "/raw/" in p.as_posix() or "melee_modes/src/tu/" in p.as_posix():
            continue
        try:
            t = p.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue
        for m in re.finditer(r"\bfn (\w+)\s*[(<]", t):
            ported.setdefault(m.group(1), []).append(p.relative_to(ROOT / "rust").as_posix())

    if "--defs" in want:
        print("// ---- definitions ----")
        for n, c in sorted(defs.items(), key=lambda kv: kv[1].location.line):
            args = ", ".join(f"{a.spelling or '_'}: {rs_type(a.type)}" for a in c.get_arguments())
            r = rs_type(c.result_type)
            st = "static " if c.storage_class == ci.StorageClass.STATIC else ""
            hint = f"   // ported: {ported[n][0]}" if n in ported else ""
            print(f"// {st}{n}({args}) -> {r}  [{c.location.line}]{hint}")
    if "--ext" in want:
        print("// ---- callees not defined in this TU ----")
        for n, d in sorted(called.items()):
            if n in defined_names or n.startswith("__builtin"):
                continue
            args = ", ".join(f"a{i}: {rs_type(a.type)}" for i, a in enumerate(d.get_arguments()))
            if d.type.kind == K.FUNCTIONPROTO and d.type.is_function_variadic():
                args += (", " if args else "") + "/* ... */"
            r = rs_type(d.result_type)
            hint = f"  // ported: {', '.join(ported[n][:2])}" if n in ported else ""
            print(f"        fn {n}({args}) -> {r};{hint}")
    if "--globals" in want:
        print("// ---- globals ----")
        for n, (d, users) in sorted(gvars.items()):
            t = d.type
            try:
                sz = t.get_size()
            except Exception:
                sz = -1
            kind = "defined" if d.is_definition() or d.storage_class == ci.StorageClass.STATIC else "extern"
            st = "static" if d.storage_class == ci.StorageClass.STATIC else ("const" if t.is_const_qualified() else "")
            print(f"// {kind:7} {st:6} {n}: {t.spelling}  size {sz}  rust {rs_type(t)}  used by {len(users)}")
    if UNKNOWN:
        print("// types not in melee_types:", ", ".join(sorted(UNKNOWN)))


if __name__ == "__main__":
    main()
