#!/usr/bin/env python3
"""Where does the C side of a TU oracle fault?   python rust/melee_modes/oracle_tu/crashloc.py <stem> <func>... [--cases N]

Links the built oracle (target/tu-oracle/<stem>/wrap.lo.ll, from `tuoracle.py <stem> --build`) as dbg.exe with a
map file and runs `--dump <func> <seed>` under lldb (console, no window) for the first N seeds; for every fault it
prints the faulting symbol (nearest map symbol below the pc), the access address and the first instruction."""
import re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CL = r"C:\Program Files\LLVM\bin\clang.exe"
LLDB = r"C:\Program Files\LLVM\bin\lldb.exe"


def link(cdir):
    exe = cdir / "dbg.exe"
    subprocess.run([CL, "-O1", "-w", str(cdir / "main.c"), str(cdir / "wrap.lo.ll"), "-o", str(exe),
                    "-Wl,/STACK:268435456", "-Wl,/OPT:NOICF", f"-Wl,/MAP:{cdir / 'dbg.map'}"], cwd=ROOT, check=True)
    syms = []
    for ln in (cdir / "dbg.map").read_text(errors="replace").splitlines():
        m = re.match(r"\s*0001:[0-9a-f]+\s+(\S+)\s+([0-9a-f]{16})\s", ln)
        if m:
            syms.append((int(m.group(2), 16), m.group(1)))
    syms.sort()
    return exe, syms


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    cases = 3
    if "--cases" in sys.argv:
        cases = int(sys.argv[sys.argv.index("--cases") + 1])
        args = [a for a in args if a != str(cases)]
    stem, funcs = args[0], args[1:]
    cdir = ROOT / "rust/target/tu-oracle" / stem
    exe, syms = link(cdir)
    for n in funcs:
        seen = set()
        for i in range(cases):
            seed = (int.from_bytes(n.encode()[:8].ljust(8, b"\0"), "little") * 0x9E3779B97F4A7C15 + i * 0x1234567) & ((1 << 64) - 1)
            r = subprocess.run([LLDB, "-b", "-o", "run", "-k", "image list", "-k", "bt 6", "--", str(exe), "--dump", n, f"{seed:x}"],
                               capture_output=True, text=True, timeout=120, cwd=ROOT)
            out = r.stdout
            if "stop reason = Exception" not in out:
                print(f"{n} case {i}: no fault")
                continue
            acc = re.search(r"Access violation (\w+) location (0x[0-9a-f]+)", out)
            base = re.search(r"\[\s*\d+\].*?(0x[0-9a-f]+) .*dbg\.exe", out)
            frames = re.findall(r"frame #\d+: (0x[0-9a-f]+)", out)
            names = []
            for pc in frames:
                nm = "?"
                if base:
                    rva = int(pc, 16) - int(base.group(1), 16) + 0x140000000
                    best = [x for x in syms if x[0] <= rva]
                    if best:
                        nm = f"{best[-1][1]}+{rva - best[-1][0]:#x}"
                names.append(nm)
            key = (tuple(names[:2]), acc.group(2) if acc else "")
            if key in seen:
                continue
            seen.add(key)
            print(f"{n} case {i}: {acc.group(1) if acc else '?'} {acc.group(2) if acc else ''}  " + " < ".join(names))


main()
