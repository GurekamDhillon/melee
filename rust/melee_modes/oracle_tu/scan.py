#!/usr/bin/env python3
"""Scan a game TU with libclang (the oracle's own flags): defined functions, external functions
it calls or takes the address of, and the global variables it defines or references."""
import os, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g
ci, K, T = g.ci, g.K, g.T
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]


def parse_tu(path, extra=()):
    idx = ci.Index.create()
    opts = ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD
    tu = idx.parse(str(path), args=NAT + list(extra), options=opts)
    errs = [d for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    if errs:
        raise SystemExit("clang errors:\n" + "\n".join(str(e) for e in errs[:20]))
    return tu


def scan(path):
    tu = parse_tu(path)
    main = Path(path).resolve()
    funcs, ext, gvars, refs = {}, {}, {}, {}
    defined = set()
    for c in tu.cursor.get_children():
        if c.kind == K.FUNCTION_DECL and c.is_definition():
            defined.add(c.spelling)
    def walk(c, infunc):
        for ch in c.get_children():
            if ch.kind == K.DECL_REF_EXPR and ch.referenced is not None:
                r = ch.referenced
                if r.kind == K.FUNCTION_DECL:
                    refs.setdefault(r.spelling, r)
                elif r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT:
                    refs.setdefault(r.spelling, r)
            walk(ch, infunc)
    for c in tu.cursor.get_children():
        if c.kind == K.FUNCTION_DECL and c.is_definition():
            f = c.location.file
            if f and Path(f.name).resolve() == main:
                funcs[c.spelling] = c
                walk(c, True)
        elif c.kind == K.VAR_DECL:
            if c.is_definition() and c.storage_class != ci.StorageClass.EXTERN:
                gvars[c.spelling] = c
                walk(c, False)
    return tu, funcs, refs, gvars, defined


if __name__ == "__main__":
    tu, funcs, refs, gvars, defined = scan(sys.argv[1])
    print("FUNCS", len(funcs))
    for n, c in funcs.items():
        print("  ", c.result_type.spelling, n, "(", ", ".join(a.type.spelling + " " + a.spelling for a in c.get_arguments()), ")",
              "[static]" if c.storage_class == ci.StorageClass.STATIC else "")
    print("GVARS (defined in TU)")
    for n, c in gvars.items():
        f = c.location.file.name if c.location.file else "?"
        print("  ", c.type.spelling, n, "@", Path(f).name, "[static]" if c.storage_class == ci.StorageClass.STATIC else "")
    print("REFS")
    for n, r in refs.items():
        if r.kind == K.FUNCTION_DECL:
            if n in funcs: continue
            if r.is_definition():
                print("   fn-inline-def", n)
            else:
                print("   fn-ext", r.result_type.spelling, n, "(", ", ".join(a.type.spelling for a in r.get_arguments()), ")")
        else:
            if n in gvars: continue
            print("   var-ext", r.type.spelling, n, "def" if r.is_definition() else "")
