# SPDX-License-Identifier: MIT
"""Discrimination bits of a corpus (notes/coverage-testing.md):  python -I disc_report.py CORPUS_DIR [--sets a,b] [--base SETS]
Reads CORPUS_DIR/checks/<id>/{inputs,ai}/disc.tsv (melee_core::disc::dump, written by disc_recheck.sh) and the
run's first NHV1 difference against Dolphin (<mode>/compare.txt). A bit counts as verified when it was first set on a
frame before that run's first difference (a run that matches the console all the way verifies all its bits).
Totals: the call sites of each helper in the ported game TUs (the decomp-named files report.py counts), by source.
--base SETS: also print what the other sets add over those (e.g. --base pairs,stages,levels,items,doubles for soak).
Writes CORPUS_DIR/disc.txt."""
import collections, json, os, re, sys

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import report  # noqa: E402

ALTS = ["fused", "unfused", "f64-intermediate", "zero-sign", "flooring", "wrapping"]
APPLIES = {"fma": [0, 1, 2], "fnm": [3, 0, 1], "div": [4], "rem": [4], "shl": [5], "shr": [5], "cvt": [5]}
PAT = {"fma": r"\b(fmadds|fmsubs|fmsubs_neg)\(", "fnm": r"\b(fnmsubs|fnmadds)\(", "div": r"\bpdiv\(", "rem": r"\bprem\(",
       "shl": r"\bpshl\(", "shr": r"\bpshr\(", "cvt": r"\b(fctiwz|fctiwz_f32)\("}


def static_sites(root):
    report.DECOMP = report.decomp_stems(root)
    tot = collections.Counter()
    for c in report.CRATES:
        d = os.path.join(root, "rust", c, "src", "tu")
        for dp, _, fs in os.walk(d):
            for f in fs:
                if not f.endswith(".rs") or f[:-3].lower() not in report.DECOMP:
                    continue
                for line in open(os.path.join(dp, f), encoding="utf-8", errors="replace"):
                    s = line.split("//")[0]
                    if re.search(r"^\s*(use|pub use|fn|pub fn)\b", s):
                        continue
                    for k, p in PAT.items():
                        tot[k] += len(re.findall(p, s))
    return tot


def game_site(loc):
    p = loc.replace("\\", "/")
    m = re.match(r"(.*)/src/tu/([^/:]+)\.rs:", p) or re.match(r"(.*)src/tu/([^/:]+)\.rs:", p)
    if not m:
        return False
    return m.group(2).lower() in report.DECOMP and any(f"{c}/" in p for c in report.CRATES)


def first_diff(path):
    try:
        t = open(path).read()
    except OSError:
        return None
    m = re.search(r"first divergence at frame (-?\d+)", t)
    return int(m.group(1)) if m else 1 << 30


def main():
    corpus = sys.argv[1]
    root = os.path.abspath(os.path.join(here, "..", "..", ".."))
    tot = static_sites(root)
    idx = {json.loads(l)["id"]: json.loads(l) for l in open(os.path.join(corpus, "index.jsonl"))}
    sets = sys.argv[sys.argv.index("--sets") + 1].split(",") if "--sets" in sys.argv else None
    base = sys.argv[sys.argv.index("--base") + 1].split(",") if "--base" in sys.argv else None
    # site -> kind, alt bits seen (any run), verified alt bits, per set
    seen = collections.defaultdict(lambda: collections.defaultdict(int))
    ver = collections.defaultdict(lambda: collections.defaultdict(int))
    kinds = {}
    runs = 0
    for id_, r in idx.items():
        if sets and r["set"] not in sets and not (base and r["set"] in base):
            continue
        for mode in ("inputs", "ai"):
            d = os.path.join(corpus, "checks", id_, mode)
            p = os.path.join(d, "disc.tsv")
            if not os.path.exists(p):
                continue
            F = first_diff(os.path.join(d, "compare.txt"))
            if F is None:
                continue
            runs += 1
            for line in open(p):
                f = line.rstrip("\n").split("\t")
                loc, kind, alts = f[0], f[1], int(f[4], 16)
                if not alts or not game_site(loc):
                    continue
                kinds[loc] = kind
                for b in range(6):
                    if alts >> b & 1:
                        seen[loc][r["set"]] |= 1 << b
                        fr = f[5 + b]
                        if fr != "-" and int(fr) < F:
                            ver[loc][r["set"]] |= 1 << b
    L = [f"{runs} engine runs read; totals are the helper call sites in the decomp-named game TUs"]

    def table(label, include):
        L.append("")
        L.append(label)
        for kind, alts in APPLIES.items():
            for b in alts:
                s = sum(1 for loc in kinds if kinds[loc] == kind and any(v >> b & 1 for st, v in seen[loc].items() if include(st)))
                v = sum(1 for loc in kinds if kinds[loc] == kind and any(x >> b & 1 for st, x in ver[loc].items() if include(st)))
                L.append(f"  {kind:<4} {ALTS[b]:<17} discriminated+verified {v:>5} / {tot[kind]:<5}   (discriminated in any run {s})")
    table("all sets" if not sets else f"sets {','.join(sets)}" + (f" + {','.join(base)}" if base else ""), lambda st: True)
    if base:
        table(f"base only ({','.join(base)})", lambda st: st in base)
        L.append("")
        L.append(f"added by {','.join(sets)} over the base (sites with a verified bit the base does not verify):")
        for kind, alts in APPLIES.items():
            for b in alts:
                n = sum(1 for loc in kinds if kinds[loc] == kind
                        and any(x >> b & 1 for st, x in ver[loc].items() if st in sets)
                        and not any(x >> b & 1 for st, x in ver[loc].items() if st in base))
                L.append(f"  {kind:<4} {ALTS[b]:<17} +{n}")
    txt = "\n".join(L) + "\n"
    open(os.path.join(corpus, "disc.txt" if not sets else f"disc_{'_'.join(sets)}.txt"), "w").write(txt)
    print(txt)


if __name__ == "__main__":
    main()
