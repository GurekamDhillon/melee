#!/usr/bin/env python3
"""Match a Rust float trace (melee_core::fptrace, `--cfg fptrace`) against a Gekko trace (gekko.py over a
Dolphin dump) by value, and classify each Rust result the PPC side never produced.

    match.py rust.tsv ppc.tsv [--all]

Both logs carry every value as the f64 bit pattern of the register value (an f32 is widened exactly).
The Rust log only has the ops of `#[trace]`-annotated functions and fp helpers, so PPC ops with no Rust
counterpart are expected and only counted. A Rust result is

  ok         its output bits are an output of some PPC op (in order, searching forward from the last match)
  fusion+    Rust fused (fmadd*) where the PPC result equals the unfused mul-then-add
  fusion-    Rust did mul then add where the PPC did one fmadds/fmadd on the same inputs
  frsp       the PPC value is the Rust value rounded to f32 (Rust kept a double), or the reverse
  force25    the PPC value is the Rust op recomputed with frC rounded to 25 bits (Dolphin Force25Bit)
  prec       same inputs, different output, none of the above (single vs double op, or another order)
  neg        the PPC produced the negated value (fnmsubs where Rust has fmsubs and negates later): equal
  miss       no PPC op has these inputs or this output (a different path, or an op the PPC folded)

docs/rust-port/notes/semantics.md (round 3) has the workflow.
"""
import argparse
import struct
import sys
from fractions import Fraction


def f64(b):
    return struct.unpack("<d", struct.pack("<Q", b))[0]


def b64(x):
    return struct.unpack("<Q", struct.pack("<d", x))[0]


def to_f32(x):
    """Round an f64 to f32 (round to nearest even), returned as f64."""
    try:
        return struct.unpack("<f", struct.pack("<f", x))[0]
    except OverflowError:
        return float("inf") if x > 0 else float("-inf")


def force25(x):
    """Dolphin Force25Bit: round the f64 mantissa to 25 bits (ties away from zero on the low 27 bits)."""
    b = b64(x)
    b = (b & 0xFFFFFFFFF8000000) + (b & 0x8000000)
    return f64(b & 0xFFFFFFFFFFFFFFFF)


def fma_exact(a, c, b):
    if any(v != v or v in (float("inf"), float("-inf")) for v in (a, b, c)):
        return a * c + b
    return float(Fraction(a) * Fraction(c) + Fraction(b))


def load(path):
    rows = []
    with open(path) as f:
        hdr = None
        for line in f:
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            cols = line.split("\t")
            if hdr is None:
                hdr = cols
                continue
            r = dict(zip(hdr, cols))
            r["ins"] = [int(x, 16) for x in r.get("inputs", "").split(",") if x]
            r["o"] = int(r.get("out", r.get("out0", "0")), 16)
            r["outs"] = [r["o"]] + ([int(r["out1"], 16)] if r.get("prec") == "ps" and r.get("out1") else [])
            rows.append(r)
    return rows


FUSED = ("fmadd", "fmadds", "fmsub", "fmsubs", "fnmadd", "fnmadds", "fnmsub", "fnmsubs",
         "ps_madd", "ps_msub", "ps_nmadd", "ps_nmsub", "ps_madds0", "ps_madds1")


def classify(r, ppc, by_ins):
    op, ins, o = r["op"], r["ins"], r["o"]
    key = tuple(sorted(ins))
    cands = by_ins.get(key, [])
    if op.startswith("fmadd") and len(ins) == 3:
        a, c, b = (f64(x) for x in ins)
        prod_s, prod_d = to_f32(a * c), a * c
        for p in ppc:
            pv = f64(p["o"])
            if p["op"].startswith(("fadd", "ps_add")) and b in (f64(x) for x in p["ins"]) and \
                    any(f64(x) in (prod_s, prod_d) for x in p["ins"]):
                return "fusion+", p
    if op in ("mul", "add", "sub") or op.startswith(("fmul", "fadd", "fsub")):
        for p in ppc:
            if p["op"] in FUSED and set(ins) <= set(p["ins"]):
                return "fusion-", p
    rv = f64(o)
    for p in cands:
        pv = f64(p["o"])
        if to_f32(rv) == pv or to_f32(pv) == rv:
            return "frsp", p
        if p["op"] in FUSED or p["op"].startswith(("fmul", "ps_mul")):
            v = [f64(x) for x in p["ins"]]
            if len(v) == 3:
                alt = fma_exact(v[0], force25(v[1]), v[2])
            elif len(v) == 2:
                alt = v[0] * force25(v[1])
            else:
                alt = None
            if alt is not None and to_f32(alt) == pv and to_f32(rv) != pv:
                return "force25", p
        return "prec", p
    for p in ppc:
        pv = f64(p["o"])
        if pv == pv and rv == rv and pv != 0 and abs(pv - rv) <= abs(pv) * 2.0 ** -20:
            return "prec", p
    return "miss", None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rust")
    ap.add_argument("ppc")
    ap.add_argument("--all", action="store_true", help="print the ok rows too")
    a = ap.parse_args()
    rust, ppc = load(a.rust), load(a.ppc)
    ppc = [p for p in ppc if p.get("prec") != "cvt" or p["op"].startswith("frsp")]
    by_ins = {}
    for p in ppc:
        by_ins.setdefault(tuple(sorted(p["ins"])), []).append(p)
    out_pos = {}
    for i, p in enumerate(ppc):
        for o in p["outs"]:  # a paired-single op yields both lanes
            out_pos.setdefault(o, []).append(i)
    counts = {}
    used = set()
    cur = 0
    for r in rust:
        if r["op"] == "i2f":
            continue  # ints converted on the PPC side through the 0x43300000 magic: nothing to match
        hit = next((i for i in out_pos.get(r["o"], []) if i >= cur), None)
        if hit is None:
            hit = next(iter(out_pos.get(r["o"], [])), None)
        if hit is not None:
            used.add(hit)
            cur = hit
            k, p = "ok", ppc[hit]
        elif out_pos.get(r["o"] ^ (1 << 63)):
            k, p = "neg", ppc[out_pos[r["o"] ^ (1 << 63)][0]]  # the same value negated (fmsubs vs fnmsubs + a later -x)
        else:
            k, p = classify(r, ppc, by_ins)
        counts[k] = counts.get(k, 0) + 1
        if k != "ok" or a.all:
            ps = f"  ppc {p.get('pc', '?')} {p.get('func', '?')} {p['op']} {p['inputs']} -> {p['o']:016X}" if p else ""
            print(f"{k:8} rust #{r['seq']} {r['func']} {r['op']}/{r['prec']} {r['inputs']} -> {r['o']:016X}{ps}")
    print(f"# rust {len(rust)} ops: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())) +
          f"; ppc {len(ppc)} float ops, {len(used)} matched")
    return 0 if set(counts) <= {"ok", "neg"} else 1


if __name__ == "__main__":
    sys.exit(main())
