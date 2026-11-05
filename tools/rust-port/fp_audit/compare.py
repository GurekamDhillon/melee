#!/usr/bin/env python3
"""Compare console fusion (scan_dol.py CSV) with the C build's fusion (clang_sites.py CSV), per function.

A callee that the clang IR calls but the DOL function does not `bl` was inlined by MWCC; its clang
sites are added to the caller's expected count (recursively). Prints one line per function:
  status tu func dol=N clang=M [inlined=...] lines=...
status: match (equal counts) | mismatch.
  python tools/rust-port/fp_audit/compare.py DOL.csv CLANG.csv [--all] [--out OUT.csv] [--sites SITES.csv]
A site is predicted unfused by the console when it is a square of a memory/computed expression
(clang_sites.classify_square); the count of predicted-fused sites must equal the DOL's fused ops.
"""
import argparse, csv, re
from collections import Counter, defaultdict


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dol"); ap.add_argument("clang")
    ap.add_argument("--all", action="store_true"); ap.add_argument("--out")
    ap.add_argument("--sites", help="per clang site: predicted console fusion (fused/unfused) in context")
    a = ap.parse_args()
    dfused, dcalls, dtu = Counter(), defaultdict(set), {}
    for r in csv.DictReader(open(a.dol)):
        dtu[r["func"]] = r["tu"]
        if r["class"] in ("fused_s", "fused_d"):
            dfused[r["func"]] += 1
        elif r["class"] == "call":
            dcalls[r["func"]].add(r["mnem"])
            dcalls[r["func"]].add(re.sub(r"__F\w*$", "", r["mnem"]))  # sqrtf__Ff is clang's sqrtf
    csites, ccalls, defined = defaultdict(list), defaultdict(Counter), set()
    for r in csv.DictReader(open(a.clang)):
        if r["pass"] != "raw":
            continue
        k = (r["tu"], r["func"])
        defined.add(k)
        if r["type"] == "call":
            ccalls[k][r["line"]] += 1
        else:
            csites[k].append((r["type"], r["line"]))

    def inlined_all(names, top):
        return all(n not in dcalls[top] for n in names if n)

    def decide(cls, argd, top):
        """True if MWCC fuses this clang fmuladd site (see clang_sites.classify_square)."""
        if "sqmem" in cls:
            return False
        m = re.search(r"sqlocal\{([^}]*)\}", cls)
        if m:
            return inlined_all(m.group(1).split("|"), top)
        m = re.search(r"sqparam(\d+)", cls)
        if m:
            i = int(m.group(1))
            d = argd[i] if argd and i < len(argd) else "-"
            lm = re.match(r"L\{([^}]*)\}", d)
            return bool(lm) and inlined_all(lm.group(1).split("|"), top)
        return True

    def expected(tu, f, top, argd=None, depth=0, seen=()):
        """(expected fused count, inlined callee names, per-site predictions) for f inside top."""
        sites = [(f, cls, ln, decide(cls, argd, top)) for cls, ln in csites[(tu, f)]]
        n, inl = sum(x[3] for x in sites), []
        if depth > 8:
            return n, inl, sites
        for call, times in ccalls[(tu, f)].items():
            c, _, rest = call.partition("(")
            cargs = rest.rstrip(")").split(",") if rest.rstrip(")") else []
            # inlined by MWCC: defined in this TU's IR, and the DOL function never calls it
            if (tu, c) in defined and c not in dcalls[top] and c not in seen and c != f:
                m, sub, ss = expected(tu, c, top, cargs, depth + 1, seen + (f,))
                if m:
                    inl.append(c)
                n += m * times; inl += sub; sites += ss * times
        return n, inl, sites

    rows, site_rows = [], []
    for f in sorted(set(dfused) | {f for _, f in csites if f in dtu}):
        tu = dtu.get(f, "")
        if not tu.startswith(("melee/", "sysdolphin/")):
            continue
        e, inl, sites = expected(tu, f, f)
        st = "match" if e == dfused[f] else "mismatch"
        rows.append((st, tu, f, dfused[f], len(csites[(tu, f)]), e, ";".join(inl),
                     " ".join(f"{c}:{l}" for c, l in csites[(tu, f)])))
        for origin, cls, ln, fused in sites:
            site_rows.append((f, st, origin, ln, cls, "fused" if fused else "unfused"))
    for r in rows:
        if a.all or r[0] != "match":
            print(f"{r[0]} {r[1]} {r[2]} dol={r[3]} clang={r[4]} expect={r[5]} inl={r[6]} lines={r[7]}")
    c = Counter(r[0] for r in rows)
    print("functions:", dict(c), "dol sites in matching functions:", sum(r[3] for r in rows if r[0] == "match"),
          "in mismatching:", sum(r[3] for r in rows if r[0] != "match"))
    if a.out:
        with open(a.out, "w", newline="") as fo:
            w = csv.writer(fo); w.writerow(["status", "tu", "func", "dol", "clang", "expect", "inlined", "clang_lines"])
            w.writerows(rows)
    if a.sites:
        with open(a.sites, "w", newline="") as fo:
            w = csv.writer(fo); w.writerow(["func", "func_status", "origin", "line", "class", "console"])
            w.writerows(site_rows)


if __name__ == "__main__":
    main()
