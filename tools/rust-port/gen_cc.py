#!/usr/bin/env python3
"""Write a compile_commands.json for c2rust (native x86_64, -DTARGET_PC) covering <paths>.

  python3 tools/rust-port/gen_cc.py --root <worktree> --gale-include <dir> --out <json> <paths...>

<paths> are worktree-relative files, directories or globs. Directories/globs are expanded and
filtered to the game TU list (docs/rust-port/game_tus.txt) so dead code (MetroTRK, Runtime, unused
libs/dolphin) is never picked up; an explicitly named .c file is always taken.

Flags mirror docs/rust-port/gen_compile_commands.py (LLP64 Windows target so C `long` stays 32-bit
as on the GameCube; -ffp-contract=off), plus two probe masks needed for a 64-bit parse:
  -D_Static_assert(a,b)=   the 104 GameCube sizeof() asserts fail with 8-byte pointers
  -Dsize_t=gc_size_t       MSL's `typedef unsigned long size_t` clashes with clang's 64-bit size_t
  -mlong-double-64         long double is f64 (as on PPC); otherwise c2rust emits the f128 crate
  tools/rust-port/include  stdarg.h override (plain __builtin_va_arg) searched before src/MSL
"""
import argparse, glob, json, os
from pathlib import Path

FLAGS = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ffp-contract=off", "-fno-fast-math",
         "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t",
         "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "tools/rust-port/include", "-isystem", "src/MSL", "-isystem", "libs/dolphin/include",
         "-isystem", "libs/dolphin/src", "-isystem", "@GALE@",
         "-include", "src/MSL/math_ppc.h", "-mlong-double-64", "-Wno-everything"]


def expand(root, paths, tus):
    out = []
    for p in paths:
        p = p.replace("\\", "/").rstrip("/")
        full = root / p
        if full.is_file() and p.endswith(".c"):
            out.append(p)
            continue
        if full.is_dir():
            hits = [t for t in tus if t.startswith(p + "/")]
        else:
            hits = [Path(h).relative_to(root).as_posix() for h in glob.glob(str(root / p), recursive=True)]
            hits = [h for h in hits if h in tus or h.endswith(".c") and Path(p).name == Path(h).name]
        if not hits:
            raise SystemExit(f"gen_cc: no game TUs under {p}")
        out += hits
    return sorted(dict.fromkeys(out))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, required=True)
    ap.add_argument("--gale-include", required=True)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--list", action="store_true", help="print the TU list only")
    ap.add_argument("paths", nargs="+")
    a = ap.parse_args()
    root = a.root.resolve()
    tus = set(l.strip() for l in (root / "docs/rust-port/game_tus.txt").read_text().splitlines() if l.strip())
    files = expand(root, a.paths, tus)
    if a.list:
        print("\n".join(files)); return
    flags = [a.gale_include if f == "@GALE@" else f for f in FLAGS]
    db = [{"directory": root.as_posix(), "file": f, "arguments": ["clang", *flags, "-c", f]} for f in files]
    a.out.write_text(json.dumps(db, indent=1))
    print(f"gen_cc: {len(db)} TUs -> {a.out}")


if __name__ == "__main__":
    main()
