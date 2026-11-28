#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Pick a content-covering subset of the regression set for the coverage map (notes/coverage-map.md).
   python -I pick.py META.tsv REGRESS.tsv N OUT.tsv
Tier 1 only (Slippi >= 3.0). Greedy cover of: every internal character (FighterKind, so Sheik/Zelda, Popo/Nana), every
stage, every item kind seen, 3-4 player games; then each legal stage (20) and each character (6) up to a quota; replays whose
regress status is "match" first (the engine followed the console). Deterministic (ordered by an md5 of the file name, so sets mix)."""
import hashlib, sys

LEGAL = [2, 3, 8, 28, 31, 32]


def main():
    meta, reg, n, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
    status = {}
    for l in list(open(reg, encoding="utf-8"))[1:]:
        r = l.rstrip("\n").split("\t")
        status[(r[0], r[1])] = r[3]
    rows = []
    for l in list(open(meta, encoding="utf-8"))[1:]:
        r = l.rstrip("\n").split("\t")
        if r[2] == "None" or int(r[2].split(".")[0]) < 3:
            continue
        el = {("c", int(c)) for c in r[5].split(",") if c} | {("s", int(r[3]))} | {("i", int(c)) for c in r[6].split(",") if c}
        if int(r[9]) > 2:
            el.add(("p", int(r[9])))
        rows.append((r, el, status.get((r[0], r[1])) == "match"))
    rows.sort(key=lambda x: (not x[2], hashlib.md5(x[0][1].encode()).hexdigest()))
    picked, have = [], set()
    while len(picked) < n:
        best = max(rows, key=lambda x: (len(x[1] - have), x[2]), default=None)
        if best is None or not (best[1] - have):
            break
        picked.append(best); have |= best[1]; rows.remove(best)
    # quotas: each legal stage 20, each internal character 6
    cnt = {}
    for r, el, _ in picked:
        for e in el:
            cnt[e] = cnt.get(e, 0) + 1
    for quota_key, q in [(("s", s), 20) for s in LEGAL] + [(("c", c), 6) for c in range(27)]:
        for x in list(rows):
            if len(picked) >= n or cnt.get(quota_key, 0) >= q:
                break
            if quota_key in x[1] and x[2]:
                picked.append(x); rows.remove(x)
                for e in x[1]:
                    cnt[e] = cnt.get(e, 0) + 1
    with open(out, "w") as fo:
        fo.write("set\trelpath\tversion\tstage\tinternal_chars\titems\tplayers\tregress_match\n")
        for r, el, m in picked:
            fo.write(f"{r[0]}\t{r[1]}\t{r[2]}\t{r[3]}\t{r[5]}\t{r[6]}\t{r[9]}\t{int(m)}\n")
    print(f"picked {len(picked)}, elements {len(have)}")


if __name__ == "__main__":
    main()
