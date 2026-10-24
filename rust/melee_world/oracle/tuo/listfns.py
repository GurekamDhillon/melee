#!/usr/bin/env python3
"""listfns.py <src/melee/..../x.c>: definitions of the TU (static/inline marked) with signatures."""
import os, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g
ci, K = g.ci, g.K
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
p = sys.argv[1]
tu = ci.Index.create().parse(p, args=NAT, options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
ab = Path(p).resolve().as_posix().lower()
for c in tu.cursor.walk_preorder():
    if c.kind == K.FUNCTION_DECL and c.is_definition() and c.location.file and Path(c.location.file.name).resolve().as_posix().lower() == ab:
        toks = [t.spelling for t in list(c.get_tokens())[:6]]
        flag = ("static " if c.storage_class == ci.StorageClass.STATIC else "") + ("inline " if "inline" in toks else "")
        args = ", ".join(f"{a.type.spelling} {a.spelling}" for a in c.get_arguments())
        print(f"{c.location.line:5} {flag}{c.result_type.spelling} {c.spelling}({args})")
