# SPDX-License-Identifier: MIT
"""Seed-match results of a corpus index (notes/coverage-testing.md):  python -I summarize.py CORPUS_DIR
Per mode (inputs = the CPUs fed their recorded inputs, ai = the engine's own CPU AI): how many matches are
Dolphin-exact over the whole NHV1 row (seed ignored), and the first-difference class of the rest:
  frame bucket of the first NHV1 difference (pre-GO entry < 123, the first second, later) x the NHV1 columns
  (s<2*port+j>: fighter slot), plus slpcheck's first Post-Frame field against the console's own recording.
Also per set and the Dolphin nondeterminism count."""
import collections, json, os, re, sys


def bucket(f):
    f = int(f)
    return "entry(<123)" if f < 123 else "first-second(<183)" if f < 183 else "first-minute(<3723)" if f < 3723 else "late"


def post_class(s):
    if s.startswith("match"):
        return "post-frame match"
    m = re.search(r"field_(\w+?)_engine", s)
    return f"post-frame {m.group(1)}" if m else s[:40]


def main():
    corpus = sys.argv[1]
    rows = [json.loads(l) for l in open(os.path.join(corpus, "index.jsonl"))]
    rows = [r for r in rows if "inputs" in r]
    L = [f"{len(rows)} matches checked"]
    nd = sum(1 for r in rows if r.get("nondet", "-") != "-")
    L.append(f"Dolphin runs disagreeing: {nd}")
    for mode in ("inputs", "ai"):
        ok = [r for r in rows if r[mode] == "match"]
        post_ok = [r for r in rows if r[f"post_{mode}"].startswith("match")]
        L.append("")
        L.append(f"== {mode}: NHV1-exact {len(ok)}/{len(rows)};  slpcheck Post-Frame exact {len(post_ok)}/{len(rows)}")
        by_set = collections.Counter(r["set"] for r in ok)
        tot_set = collections.Counter(r["set"] for r in rows)
        L.append("   per set: " + ", ".join(f"{s} {by_set[s]}/{n}" for s, n in sorted(tot_set.items())))
        cls = collections.Counter()
        for r in rows:
            if r[mode] == "match":
                continue
            f, _, cols = r[mode].partition(":")
            if not f.lstrip("-").isdigit():
                cls[f"error {r[mode]}"] += 1
                continue
            kind = "+".join(sorted({c for c in cols.split(",") if c}))
            cls[f"{bucket(f):<20} {kind}"] += 1
        L.append("   first NHV1 difference (frame bucket, columns):")
        for k, n in cls.most_common(15):
            L.append(f"     {n:>4}  {k}")
        pc = collections.Counter(post_class(r[f"post_{mode}"]) for r in rows)
        L.append("   slpcheck first Post-Frame field: " + ", ".join(f"{k} {n}" for k, n in pc.most_common(10)))
    txt = "\n".join(L) + "\n"
    open(os.path.join(corpus, "results.txt"), "w").write(txt)
    print(txt)


if __name__ == "__main__":
    main()
