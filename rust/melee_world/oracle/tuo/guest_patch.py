#!/usr/bin/env python3
"""Redirect the TU's guest-memory reads of OS globals (e.g. `*(u32*)0x800000F8`, an inttoptr
constant in the IR) to plain variables: guest_patch.py <outdir>; reads <outdir>/guest.txt
(lines `<signed decimal address> <symbol>`) and patches <outdir>/unit.ll in place."""
import pathlib
import sys

o = pathlib.Path(sys.argv[1])
s = (o / "unit.ll").read_text()
for ln in (o / "guest.txt").read_text().splitlines():
    if ln.strip():
        a, sym = ln.split()
        n = s.count(f"ptr inttoptr (i64 {a} to ptr)")
        s = s.replace(f"ptr inttoptr (i64 {a} to ptr)", f"ptr @{sym}")
        print(f"guest {a} -> {sym}: {n} uses")
(o / "unit.ll").write_text(s)
