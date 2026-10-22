"""libclang analysis of a game TU for stubgen.py (same flags as rust/melee_fighter/gen/cref/run.py)."""
import os
from pathlib import Path
import clang.cindex as ci

ROOT = Path(__file__).resolve().parents[4]
ci.Config.set_library_file(r"C:\Program Files\LLVM\bin\libclang.dll")
GALE = os.environ.get("GALE_INCLUDE", str(ROOT / "../../melee/build/GALE01/include"))
FLAGS = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-w", "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t",
         "-mlong-double-64", "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
         "-I", str(ROOT / "pc/gameworld"), "-isystem", str(ROOT / "tools/rust-port/include"),
         "-isystem", str(ROOT / "src/MSL"), "-isystem", str(ROOT / "libs/dolphin/include"),
         "-isystem", str(ROOT / "libs/dolphin/src"), "-isystem", GALE, "-include", str(ROOT / "src/MSL/math_ppc.h")]
CK = ci.CursorKind


def analyze(src):
    """Returns dict: externs{name: FUNCTION_DECL}, values{set of names used as function values},
    globals{name: VAR_DECL}, inline{name}, local{name}, tu."""
    tu = ci.Index.create().parse(str(ROOT / src), args=FLAGS)
    bad = [d for d in tu.diagnostics if d.severity >= 3]
    if bad:
        raise SystemExit("clang errors: " + "; ".join(str(d) for d in bad[:5]))
    mine = str(ROOT / src).replace("\\", "/").lower()

    def inmine(c):
        f = c.location.file
        return bool(f) and str(f.name).replace("\\", "/").lower() == mine

    defs = {}
    for c in tu.cursor.walk_preorder():
        if c.kind == CK.FUNCTION_DECL and c.is_definition():
            defs[c.spelling] = c
    local = {n for n, d in defs.items() if inmine(d)}
    externs, values, globs, inline = {}, set(), {}, set()
    seen = set()
    callee_locs = set()

    def walk(fn):
        if fn.spelling in seen:
            return
        seen.add(fn.spelling)
        for c in fn.walk_preorder():
            if c.kind == CK.CALL_EXPR and c.referenced is not None and c.referenced.kind == CK.FUNCTION_DECL:
                r = c.referenced
                # the callee DECL_REF_EXPR is a direct descendant: remember its location
                for ch in c.walk_preorder():
                    if ch.kind == CK.DECL_REF_EXPR and ch.referenced is not None and ch.referenced == r:
                        callee_locs.add((ch.location.file.name if ch.location.file else "", ch.location.offset))
                        break
                d = defs.get(r.spelling)
                if d is not None and r.spelling in local:
                    continue
                if d is not None:
                    inline.add(r.spelling)
                    walk(d)
                else:
                    externs.setdefault(r.spelling, r)
        for c in fn.walk_preorder():
            if c.kind == CK.DECL_REF_EXPR and c.referenced is not None:
                r = c.referenced
                key = (c.location.file.name if c.location.file else "", c.location.offset)
                if r.kind == CK.FUNCTION_DECL and key not in callee_locs and r.spelling not in defs:
                    values.add(r.spelling)
                    externs.setdefault(r.spelling, r)
                elif r.kind == CK.VAR_DECL and (r.linkage == ci.LinkageKind.EXTERNAL) and not inmine(r):
                    globs.setdefault(r.spelling, r)

    for n in sorted(local):
        walk(defs[n])
    # a value use may be seen before its CALL_EXPR sibling registered the location; recompute
    return dict(externs=externs, values=values, globals=globs, inline=inline, local=local, tu=tu)
