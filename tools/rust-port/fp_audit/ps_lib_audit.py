#!/usr/bin/env python3
"""Paired-single library routines: does the port model the retail dataflow?

The Dolphin SDK's matrix and vector library in the retail DOL is hand-written paired-single assembly (psq_l, ps_madd,
ps_sum0, fres/frsqrte Newton steps), not the C_MTX* reference C. A routine whose ported body is the C reference rounds
differently (PSMTXQuat's exact 2/n against fres + one Newton step was 1-2 ULP on Fox's tail and reached the hurt
capsules: notes/fp-audit.md "Paired-single library routines"). This lists every function in the DOL with paired-single
arithmetic or an estimate instruction (scan_dol.classify) that pc/gameworld/mtx_pc.c defines, and whether the C body
names the paired-single steps it models (paired-single, ps_*, psq_*, fres, frsqrte) in the comment block above it or
inside it, the convention for a routine rewritten to the DOL's dataflow. Routines with only moves (psq_l/psq_st, merges) round nothing and are not listed.

  python tools/rust-port/fp_audit/ps_lib_audit.py [--dol PATH]

Output: one row per routine (address, name, ps arithmetic count, estimates, C definition line, status
"retail" / "REFERENCE" / "not in mtx_pc.c"); exit 1 if a routine defined in mtx_pc.c is still REFERENCE.
Only addresses and mnemonic counts are printed, never code bytes.
"""
import argparse, re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_dol  # noqa: E402

ROOT = scan_dol.ROOT
# the C body is taken as rewritten to the DOL's dataflow when its comment or body names the paired-single steps
MARK = re.compile(r"paired-single|\bps_[a-z0-9]+|psq_|(?<![a-z])fres\b|frsqrte")
# routines checked and kept as they are: their arithmetic is per-lane single operations whose C order gives the same
# bits (adds, subtracts, scales), so no rewrite is needed
SAME_BITS = {"PSVECAdd", "PSVECSubtract", "PSVECScale", "PSMTXTrans", "PSMTXScale", "PSMTXRotTrig"}


def c_defs(text):
    """name -> (line, comment+body text) for each top-level function definition in mtx_pc.c."""
    lines = text.splitlines()
    out = {}
    for i, l in enumerate(lines):
        m = re.match(r"^[A-Za-z_][\w \*]*?\b(\w+)\s*\([^;]*$", l)
        if not m or l.startswith(("static ", "#")) and "(" not in l:
            continue
        name = m.group(1)
        # everything since the previous function's closing brace (its comment block), plus the body
        j = i - 1
        while j >= 0 and not lines[j].startswith("}"):
            j -= 1
        k = i
        while k < len(lines) and lines[k] != "}":
            k += 1
        out.setdefault(name, (i + 1, "\n".join(lines[j + 1:k + 1])))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dol")
    a = ap.parse_args()
    segs = scan_dol.load_dol(a.dol or scan_dol.default_dol())
    defs = c_defs((ROOT / "pc/gameworld/mtx_pc.c").read_text(errors="replace"))
    bad = 0
    print(f"{'address':8}  {'routine':22} {'ps_arith':>8} {'est':>4}  {'mtx_pc.c':>8}  status")
    for name, start, size in scan_dol.functions():
        if not (name.startswith(("PSMTX", "PSVEC", "PSQUAT")) or name.startswith("C_MTX")):
            continue
        n_ps = n_est = 0
        for adr in range(start, start + size, 4):
            w = scan_dol.read_word(segs, adr)
            c = w is not None and scan_dol.classify(w)
            if c and c[0] == "ps_arith":
                n_ps += 1
            elif c and c[0] == "estimate":
                n_est += 1
        if n_ps == 0 and n_est == 0:
            continue
        d = defs.get(name)
        if d is None:
            st = "not in mtx_pc.c"
        elif MARK.search(d[1]):
            st = "retail"
        elif name in SAME_BITS:
            st = "same bits (per-lane single ops)"
        else:
            st = "REFERENCE"
            bad += 1
        print(f"{start:08X}  {name:22} {n_ps:8} {n_est:4}  {d[0] if d else '-':>8}  {st}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
