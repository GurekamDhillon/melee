#!/usr/bin/env python3
"""Generate a NATIVE x86_64 compile_commands.json for the game TUs, and optionally syntax-check them.

The TU list is the one the workspace build compiles (_build/masstest/files.txt). Flags mirror
_build/masstest/pipe_win.sh, minus the ppc32 target / gwtool step, plus -ffp-contract=off.

  python docs/rust-port/gen_compile_commands.py --files <files.txt> [--out compile_commands.json]
      [--check] [--mask-asserts] [--gale-include <dir containing sysdolphin/baselib/*.inc>]

--mask-asserts defines _Static_assert away and renames MSL's size_t so the parse continues past
the layout assertions that fail on a 64-bit target; it is a probe, not a build.
build/GALE01/include holds generated font .inc files (debug_font.inc, sislib_font.inc); they are
not in the worktree, so --gale-include can point at another checkout's build dir for --check.
"""
import argparse, collections, json, os, re, subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLANG = os.environ.get("CLANG", r"C:/Program Files/LLVM/bin/clang.exe")
FLAGS = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ffp-contract=off", "-fno-fast-math", "-O2",
         "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "src/MSL", "-isystem", "libs/dolphin/include",
         "-isystem", "libs/dolphin/src", "-isystem", "build/GALE01/include",
         "-include", "src/MSL/math_ppc.h"]
WARN = ["-Wpointer-to-int-cast", "-Wint-to-pointer-cast", "-Wvoid-pointer-to-int-cast",
        "-Wint-to-void-pointer-cast", "-Wshorten-64-to-32", "-Wint-conversion"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--files", required=True, type=Path)
    ap.add_argument("--out", type=Path, default=ROOT / "compile_commands.json")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--mask-asserts", action="store_true")
    ap.add_argument("--gale-include")
    a = ap.parse_args()
    tus = [l.strip() for l in a.files.read_text().splitlines() if l.strip()]
    db = [{"directory": ROOT.as_posix(), "file": f,
           "arguments": [CLANG, *FLAGS, "-c", f, "-o", f + ".o"]} for f in tus]
    a.out.write_text(json.dumps(db, indent=1))
    print(f"wrote {a.out} ({len(db)} TUs)")
    if not a.check:
        return
    flags = [a.gale_include if (x == "build/GALE01/include" and a.gale_include) else x for x in FLAGS]
    if a.mask_asserts:
        flags += ["-D_Static_assert(a,b)=", "-Dsize_t=__gw_probe_size_t"]

    def run(f):
        p = subprocess.run([CLANG, *flags, *WARN, "-fsyntax-only", "-ferror-limit=0", f],
                           cwd=ROOT, capture_output=True, text=True)
        return f, p.returncode, p.stderr

    res = list(ThreadPoolExecutor(os.cpu_count() or 4).map(run, tus))
    errs, warns = collections.Counter(), collections.Counter()
    for _, _, e in res:
        for m in re.finditer(r"(warning|error): (.*?)(?: \[(-W[^\]]+)\])?$", e, re.M):
            if m.group(1) == "error":
                errs[re.sub(r"'[^']*'", "'X'", m.group(2))] += 1
            elif m.group(3):
                warns[m.group(3)] += 1
    print("TUs", len(res), "failed", sum(1 for r in res if r[1]))
    for m, c in errs.most_common(20):
        print(" E", c, m)
    for w, c in warns.most_common(20):
        print(" W", c, w)


if __name__ == "__main__":
    main()
