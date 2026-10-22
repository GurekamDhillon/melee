#!/usr/bin/env python3
"""List, per TU, the callees and file-scope variables it uses that it does not define."""
import os, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g
ci, K, T = g.ci, g.K, g.T
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]

def scan(path):
    tu = ci.Index.create().parse(path, args=NAT)
    here = Path(path).resolve()
    defs, calls, vars_ = set(), {}, {}
    for c in tu.cursor.walk_preorder():
        if c.kind == K.FUNCTION_DECL and c.is_definition() and c.location.file and Path(str(c.location.file)).resolve() == here:
            defs.add(c.spelling)
    for c in tu.cursor.walk_preorder():
        if not (c.location.file and Path(str(c.location.file)).resolve() == here):
            continue
        r = c.referenced
        if c.kind == K.CALL_EXPR and r is not None and r.kind == K.FUNCTION_DECL and r.spelling not in defs:
            calls[r.spelling] = r
        if c.kind == K.DECL_REF_EXPR and r is not None and r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT:
            if not (r.location.file and Path(str(r.location.file)).resolve() == here):
                vars_[r.spelling] = r
        if c.kind == K.DECL_REF_EXPR and r is not None and r.kind == K.FUNCTION_DECL and r.spelling not in defs:
            calls.setdefault(r.spelling, r)
    print("==", path)
    for n, r in sorted(calls.items()):
        print("  fn ", r.result_type.spelling, n, "(", ", ".join(a.type.spelling for a in r.get_arguments()), ")", "[var]" if r.type.is_function_variadic() else "")
    for n, r in sorted(vars_.items()):
        print("  var", r.type.spelling, n)
for p in sys.argv[1:]:
    scan(p)
