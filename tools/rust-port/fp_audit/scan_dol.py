#!/usr/bin/env python3
"""Console float-rounding sites, straight from the retail DOL.

Decodes every instruction of every function in config/GALE01/symbols.txt and lists the ones whose
rounding is console-specific: fused multiply-add (fmadd/fmsub/fnmadd/fnmsub, single and double),
estimates (fres, frsqrte, ps_res, ps_rsqrte), every paired-single op, rounding points (frsp,
fctiw, fctiwz), double-precision arithmetic, and the int-to-float magic sequence (lis 0x4330).

  python tools/rust-port/fp_audit/scan_dol.py [--dol PATH] [--csv OUT.csv] [--summary]

The DOL defaults to <workspace>/melee/orig/GALE01/sys/main.dol, else orig/GALE01/sys/main.dol of
this worktree. Only addresses, mnemonics and symbol names are printed, never code bytes or data.
Output CSV: addr,func,tu,class,mnem  (one row per site; class "call" rows give the bl target in mnem). --summary prints per-function counts.
"""
import argparse, csv, re, struct, sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WS = ROOT.parents[1]

A59 = {18: "fdivs", 20: "fsubs", 21: "fadds", 22: "fsqrts", 24: "fres", 25: "fmuls",
       28: "fmsubs", 29: "fmadds", 30: "fnmsubs", 31: "fnmadds"}
A63 = {18: "fdiv", 20: "fsub", 21: "fadd", 22: "fsqrt", 23: "fsel", 25: "fmul", 26: "frsqrte",
       28: "fmsub", 29: "fmadd", 30: "fnmsub", 31: "fnmadd"}
X63 = {12: "frsp", 14: "fctiw", 15: "fctiwz"}
A4 = {10: "ps_sum0", 11: "ps_sum1", 12: "ps_muls0", 13: "ps_muls1", 14: "ps_madds0",
      15: "ps_madds1", 18: "ps_div", 20: "ps_sub", 21: "ps_add", 23: "ps_sel", 24: "ps_res",
      25: "ps_mul", 26: "ps_rsqrte", 28: "ps_msub", 29: "ps_madd", 30: "ps_nmsub", 31: "ps_nmadd"}
X4 = {40: "ps_neg", 72: "ps_mr", 136: "ps_nabs", 264: "ps_abs", 528: "ps_merge00",
      560: "ps_merge01", 592: "ps_merge10", 624: "ps_merge11"}

FUSED = {"fmadds", "fmsubs", "fnmsubs", "fnmadds"}
FUSED_D = {"fmadd", "fmsub", "fnmsub", "fnmadd"}
EST = {"fres", "frsqrte", "ps_res", "ps_rsqrte"}
DBL = {"fdiv", "fsub", "fadd", "fmul", "fsqrt"}


def classify(w):
    op = w >> 26
    a5 = (w >> 1) & 31
    x10 = (w >> 1) & 1023
    if op == 59:
        m = A59.get(a5)
        if m in FUSED: return "fused_s", m
        if m in EST: return "estimate", m
        return None
    if op == 63:
        if a5 in A63 and a5 != 23:
            m = A63[a5]
            if m in FUSED_D: return "fused_d", m
            if m in EST: return "estimate", m
            if m in DBL: return "double", m
            return None
        m = X63.get(x10)
        if m == "frsp": return "frsp", m
        if m: return "f2i", m
        return None
    if op == 4:
        m = A4.get(a5) or X4.get(x10)
        if m in EST: return "estimate", m
        if m: return ("ps_move" if m in ("ps_mr", "ps_neg", "ps_abs", "ps_nabs") or "merge" in m
                      else "ps_arith"), m
        if ((w >> 1) & 63) in (6, 7, 38, 39): return "ps_move", "psq_lx/stx"
        return None
    if op in (56, 57, 60, 61):
        return "ps_move", ["psq_l", "psq_lu", "psq_st", "psq_stu"][[56, 57, 60, 61].index(op)]
    if op == 18 and (w & 1):
        return "call", None
    if op == 15 and (w & 0xFFFF) == 0x4330:
        return "i2f", "lis 0x4330"
    return None


def load_dol(p):
    b = Path(p).read_bytes()
    segs = []
    for i in range(18):
        off, addr, size = (struct.unpack_from(">I", b, o + 4 * i)[0] for o in (0, 0x48, 0x90))
        if size and i < 7:
            segs.append((addr, b[off:off + size]))
    return segs


def read_word(segs, a):
    for base, data in segs:
        if base <= a < base + len(data):
            return struct.unpack_from(">I", data, a - base)[0]
    return None


def splits():
    out, cur = [], None
    for line in (ROOT / "config/GALE01/splits.txt").read_text().splitlines():
        if line and not line[0].isspace() and line.endswith(":") and line != "Sections:":
            cur = line[:-1]
        m = re.match(r"\s+\.(text|init)\s+start:0x(\w+) end:0x(\w+)", line)
        if m and cur:
            out.append((int(m.group(2), 16), int(m.group(3), 16), cur))
    return out


def functions():
    for line in (ROOT / "config/GALE01/symbols.txt").read_text().splitlines():
        m = re.match(r"(\S+) = \.(text|init):0x(\w+); // type:function size:0x(\w+)", line)
        if m:
            yield m.group(1), int(m.group(3), 16), int(m.group(4), 16)


def scan(dol):
    segs, sp = load_dol(dol), splits()
    tu_of = lambda a: next((t for s, e, t in sp if s <= a < e), "?")
    rows, fns = [], list(functions())
    names = {s: n for n, s, _ in fns}
    for name, start, size in fns:
        tu = tu_of(start)
        rows.append((start, name, tu, "func", f"size=0x{size:X}"))  # every function, even with no site
        for a in range(start, start + size, 4):
            w = read_word(segs, a)
            c = w is not None and classify(w)
            if c and c[0] == "call":
                off = w & 0x03FFFFFC
                off -= (1 << 26) if off & (1 << 25) else 0
                tgt = (off if w & 2 else a + off) & 0xFFFFFFFF
                rows.append((a, name, tu, "call", names.get(tgt, f"{tgt:08X}")))
            elif c:
                rows.append((a, name, tu, c[0], c[1]))
    return rows


def default_dol():
    for p in (WS / "melee/orig/GALE01/sys/main.dol", ROOT / "orig/GALE01/sys/main.dol"):
        if p.exists():
            return p
    sys.exit("main.dol not found; pass --dol")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dol")
    ap.add_argument("--csv")
    ap.add_argument("--summary", action="store_true")
    ap.add_argument("--disasm", metavar="FUNC", help="print one function's instructions (needs capstone)")
    a = ap.parse_args()
    if a.disasm:
        import capstone
        md = capstone.Cs(capstone.CS_ARCH_PPC, capstone.CS_MODE_32 | capstone.CS_MODE_BIG_ENDIAN | capstone.CS_MODE_PS)
        segs = load_dol(a.dol or default_dol())
        fns = list(functions()); names = {s: n for n, s, _ in fns}
        for name, start, size in fns:
            if name == a.disasm:
                for adr in range(start, start + size, 4):
                    w = read_word(segs, adr)
                    ins = next(md.disasm(struct.pack(">I", w), adr), None)
                    txt = f"{ins.mnemonic} {ins.op_str}" if ins else ".word"
                    if (w >> 26) == 18:
                        off = w & 0x03FFFFFC; off -= (1 << 26) if off & (1 << 25) else 0
                        txt += "  ; " + names.get((adr + off) & 0xFFFFFFFF, "")
                    print(f"{adr:08X}  {txt}")
        return
    rows = scan(a.dol or default_dol())
    if a.csv:
        with open(a.csv, "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(["addr", "func", "tu", "class", "mnem"])
            for r in rows:
                w.writerow([f"{r[0]:08X}", *r[1:]])
    tot = Counter(r[3] for r in rows if r[3] not in ("call", "func"))
    print("sites by class:", dict(tot), "total", len(rows))
    if a.summary:
        per = defaultdict(Counter)
        for r in rows:
            per[(r[2], r[1])][r[3]] += 1
        for (tu, fn), c in sorted(per.items()):
            print(tu, fn, " ".join(f"{k}={v}" for k, v in sorted(c.items())))


if __name__ == "__main__":
    main()
