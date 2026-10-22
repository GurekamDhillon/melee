#!/usr/bin/env python3
"""Find the first oracle case that crashes the C oracle.exe:  bisect.py <stem>  (worktree root)."""
import os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
stem = sys.argv[1]
from cfgs import CFGS  # noqa: E402

cfg = CFGS[stem]
exe = ROOT / "rust/target/tu-oracle" / stem / "oracle.exe"
names = subprocess.run([str(exe), "--list"], capture_output=True, text=True).stdout.split()
lines = []
for n in names:
    cases = cfg.get("funcs", {}).get(n, {}).get("cases", cfg.get("cases", 60))
    for i in range(cases):
        seed = (int.from_bytes(n.encode()[:8].ljust(8, b"\0"), "little") * 0x9E3779B97F4A7C15 + i * 0x1234567) & ((1 << 64) - 1)
        lines.append(f"{n} {seed:x}")


def ok(k):
    r = subprocess.run([str(exe)], input="\n".join(lines[:k]) + "\n", capture_output=True, text=True)
    return r.returncode == 0


lo, hi = 0, len(lines)
if ok(hi):
    print("all ok")
else:
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if ok(mid):
            lo = mid
        else:
            hi = mid
    print("first failing line", hi, lines[hi - 1])
