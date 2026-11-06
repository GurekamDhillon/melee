# SPDX-License-Identifier: MIT
"""Branch coverage of the ported game code from an `llvm-cov export` JSON (notes/coverage-testing.md).
   python -I report.py COV.json OUT_DIR [--root RUST_DIR]
Counts LLVM branch outcomes (each `if`/`match` arm condition counts twice: taken true, taken false) and regions in the
game TUs: rust/<crate>/src/tu/**.rs of melee_fighter, melee_items, melee_world, melee_runtime, melee_modes (the
asked-for five) and melee_chara (reported apart). Generic instantiations of one source function are merged by location.
A function is named by the `fn NAME` at its first line: the decomp's function name (ported TUs keep it).
Writes OUT_DIR/summary.txt, functions.tsv (every game function), files.tsv; prints the summary."""
import collections, json, os, re, sys

FIVE = ["melee_fighter", "melee_items", "melee_world", "melee_runtime", "melee_modes"]
CRATES = FIVE + ["melee_chara"]
FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")


DECOMP = None


def decomp_stems(root):
    """Lower-case stems of the decomp's C files (src/melee, src/sysdolphin): a TU file of the same stem is ported game
    code; the port's own TUs (script_game, geno_*, the tests) have none and are left out."""
    st = set()
    for d in ("src/melee", "src/sysdolphin"):
        for dp, _, fs in os.walk(os.path.join(root, d)):
            st.update(f[:-2].lower() for f in fs if f.endswith(".c"))
    return st


def crate_of(path):
    p = path.replace("\\", "/")
    for c in CRATES:
        if f"/rust/{c}/src/tu/" in p:
            stem = os.path.basename(p)[:-3].lower()
            if DECOMP is not None and stem not in DECOMP:
                SKIPPED.add(p[p.index(f"/rust/{c}/"):][6:])
                return None, None
            return c, p[p.index(f"/rust/{c}/"):][6:]
    return None, None


SKIPPED = set()


def main():
    global DECOMP
    js, out = sys.argv[1], sys.argv[2]
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
    if "--all-tu" not in sys.argv:
        DECOMP = decomp_stems(root)
    os.makedirs(out, exist_ok=True)
    data = json.load(open(js))["data"][0]
    lines_cache = {}
    # (file, fn start line) -> {"name", "crate", "br": {loc: [t, f]}, "rg": {loc: count}}
    funcs = {}
    for f in data["functions"]:
        fname = f["filenames"][0]
        crate, rel = crate_of(fname)
        if not crate or not f["regions"]:
            continue
        start = min(r[0] for r in f["regions"] if r[5] == 0) if any(r[5] == 0 for r in f["regions"]) else f["regions"][0][0]
        key = (rel, start)
        e = funcs.get(key)
        if e is None:
            if fname not in lines_cache:
                try:
                    lines_cache[fname] = open(fname, encoding="utf-8", errors="replace").read().split("\n")
                except OSError:
                    lines_cache[fname] = []
            src = lines_cache[fname]
            name = "?"
            for ln in range(start - 1, max(start - 6, -1), -1):
                if 0 <= ln < len(src):
                    m = FN.search(src[ln])
                    if m:
                        name = m.group(1)
                        break
            e = funcs[key] = {"name": name, "crate": crate, "file": rel, "line": start, "br": {}, "rg": {}}
        for b in f.get("branches", []):
            loc = tuple(b[0:4])
            t = e["br"].setdefault(loc, [0, 0])
            t[0] += b[4]
            t[1] += b[5]
        for r in f["regions"]:
            if r[5] != 0:  # skip expansion/skipped/gap regions
                continue
            loc = tuple(r[0:4])
            e["rg"][loc] = e["rg"].get(loc, 0) + r[4]
    rows = []
    for e in funcs.values():
        nb = 2 * len(e["br"])
        cb = sum((t > 0) + (fl > 0) for t, fl in e["br"].values())
        nr = len(e["rg"])
        cr = sum(1 for c in e["rg"].values() if c > 0)
        rows.append((e["crate"], e["file"], e["line"], e["name"], nb, cb, nr, cr))
    rows.sort()
    with open(os.path.join(out, "functions.tsv"), "w") as fo:
        fo.write("crate\tfile\tline\tfunction\tbranches\tbranches_covered\tregions\tregions_covered\n")
        for r in rows:
            fo.write("\t".join(map(str, r)) + "\n")
    per_file = collections.defaultdict(lambda: [0, 0, 0, 0, 0, 0])
    per_crate = collections.defaultdict(lambda: [0, 0, 0, 0, 0, 0])
    for c, fi, _, _, nb, cb, nr, cr in rows:
        for agg in (per_file[(c, fi)], per_crate[c]):
            agg[0] += nb; agg[1] += cb; agg[2] += nr; agg[3] += cr; agg[4] += 1; agg[5] += cr > 0
    with open(os.path.join(out, "files.tsv"), "w") as fo:
        fo.write("crate\tfile\tbranches\tbranches_covered\tregions\tregions_covered\tfunctions\tfunctions_entered\n")
        for (c, fi), a in sorted(per_file.items()):
            fo.write(f"{c}\t{fi}\t" + "\t".join(map(str, a)) + "\n")

    def pct(a, b):
        return f"{100.0 * a / b:5.1f}%" if b else "  n/a"
    L = []
    def total(cs, label):
        a = [sum(per_crate[c][k] for c in cs) for k in range(6)]
        L.append(f"{label:<28} branches {a[1]:>7}/{a[0]:<7} {pct(a[1], a[0])}   regions {a[3]:>7}/{a[2]:<7} {pct(a[3], a[2])}   functions entered {a[5]:>6}/{a[4]:<6} {pct(a[5], a[4])}")
    total(FIVE, "TOTAL (the five crates)")
    total(CRATES, "TOTAL (+ melee_chara)")
    L.append("")
    for c in CRATES:
        total([c], c)
    L.append("")
    L.append("50 largest functions never entered (by branch outcomes, then regions):")
    unc = sorted((r for r in rows if r[7] == 0), key=lambda r: (-r[4], -r[6]))
    for c, fi, ln, nm, nb, cb, nr, cr in unc[:50]:
        L.append(f"  {nm:<40} {nb:>5} branches {nr:>5} regions  {fi}:{ln}")
    txt = "\n".join(L) + "\n"
    open(os.path.join(out, "summary.txt"), "w").write(txt)
    print(txt)


if __name__ == "__main__":
    main()
