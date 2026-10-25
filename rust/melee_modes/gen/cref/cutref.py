#!/usr/bin/env python3
"""C reference built from verbatim line slices of the game's sources (for code the whole-TU oracle
cannot include: `.inc` files, header inlines, TU pieces).

  python rust/melee_modes/gen/cref/cutref.py <name> [...]        (from the worktree root; Windows, LLVM)

gen/cref/cut/<name>.c is the driver. A line `//CUT <file>::<begin-regex>::<end-regex>` is replaced by
the slice of <file> from the first line matching <begin-regex> to the first following line matching
<end-regex> (both inclusive), so the C under test is always the game's current text. The result is
compiled like the shipping build (run_tu.py flags: clang default contraction, -O2 -disable-llvm-passes,
gwtool's rewrites via melee_runtime/oracle/lower.py) and run; stdout goes to tests/data/cut_<name>.txt.
The driver uses the mtu_common.h harness (rnd/fv/rg, trace and state hash) like gen/cref/tu/*.c.
"""
import re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import run_tu as R  # noqa: E402

ROOT = R.ROOT


def expand(driver: str) -> str:
    out = []
    for line in driver.splitlines():
        m = re.match(r"//CUT (.*?)::(.*?)::(.*)$", line)
        if not m:
            out.append(line)
            continue
        f, b, e = m.groups()
        src = (ROOT / f).read_bytes().decode("utf-8").replace("\r\n", "\n").split("\n")
        i = next((k for k, l in enumerate(src) if re.search(b, l)), None)
        assert i is not None, f"{f}: begin /{b}/ not found"
        j = next((k for k in range(i, len(src)) if re.search(e, src[k])), None)
        assert j is not None, f"{f}: end /{e}/ not found after line {i + 1}"
        out.append(f"/* cut from {f}:{i + 1}-{j + 1} */")
        out.extend(src[i:j + 1])
        out.append("/* end cut */")
    return "\n".join(out) + "\n"


def main():
    R.BUILD.mkdir(parents=True, exist_ok=True)
    for n in sys.argv[1:]:
        drv = HERE / "cut" / f"{n}.c"
        tmp = R.BUILD / f"cut_{n}.c"
        tmp.write_bytes(expand(drv.read_text(encoding="utf-8")).encode("utf-8"))
        lls = []
        for s in (tmp, ROOT / "pc/gameworld/gekko_fp.c"):
            ll = R.BUILD / f"cut_{n}__{s.stem}.ll"
            lo = R.BUILD / f"cut_{n}__{s.stem}.lo.ll"
            R.run([R.CLANG, *R.FLAGS, "-I", str(HERE / "tu"), "-I", str(HERE / "cut"), str(s), "-o", str(ll)], n)
            R.run([sys.executable, str(R.LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        exe = R.BUILD / f"cut_{n}.exe"
        R.run([R.CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-fuse-ld=lld"], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (R.DATA / f"cut_{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cutref: {n}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    main()
