#!/usr/bin/env python3
"""Run a built TU oracle (tuoracle.py <stem> --build) one function at a time and report the functions
whose C side crashes or hangs.  python rust/melee_modes/oracle_tu/probe.py <stem> [cases]"""
import subprocess, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parent))
stem = sys.argv[1]
cases = int(sys.argv[2]) if len(sys.argv) > 2 else 5
from cfgs import CFGS
cfg = CFGS[stem]
exe = ROOT / "rust/target/tu-oracle" / stem / "oracle.exe"
names = list(cfg["funcs"])
for n in names:
    inp = []
    for i in range(cases):
        seed = (int.from_bytes(n.encode()[:8].ljust(8, b"\0"), "little") * 0x9E3779B97F4A7C15 + i * 0x1234567) & ((1 << 64) - 1)
        inp.append(f"{n} {seed:x}")
    try:
        r = subprocess.run([str(exe)], input="\n".join(inp) + "\n", capture_output=True, text=True, timeout=20)
        if r.returncode:
            print(f"CRASH {n} rc={r.returncode:#x}")
        elif not r.stdout.strip():
            print(f"EMPTY {n}")
    except subprocess.TimeoutExpired:
        print(f"HANG {n}")
