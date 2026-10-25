#!/usr/bin/env python3
"""Compare two neutral-hash CSVs (MELEE_NEUTRAL_HASHLOG format, docs/rust-port/neutral-hash.md).

    compare_golden.py GOLDEN.csv CANDIDATE.csv [--max-report N] [--allow-short] [--from N]

`--from N` compares only the golden frames >= N, for a run resumed from a state snapshot at frame N
(`melee golden --from-snapshot`, tools/rust-port/snap/); the candidate must start at exactly N.

Exit 0 when every frame of GOLDEN is present in CANDIDATE with identical columns (and the frame
sets are equal, unless --allow-short lets CANDIDATE stop early), 1 on a mismatch, 2 on bad input.
On the first mismatching frame it names the columns that differ: `seed`, `items`/`itemsum`, or a
fighter slot `sN` (N = player*2 + follower), which localises the divergence to one fighter.
"""
import argparse
import csv
import sys


def load(path):
    try:
        with open(path, newline="") as f:
            rows = list(csv.reader(f))
    except OSError as e:
        sys.exit("compare_golden: %s" % e)
    if not rows or rows[0][:2] != ["frame", "hash"]:
        print("compare_golden: %s: not a neutral-hash CSV (header %r)" % (path, rows[0] if rows else None), file=sys.stderr)
        sys.exit(2)
    header = rows[0]
    frames = {}
    order = []
    for r in rows[1:]:
        if not r:
            continue
        f = int(r[0])
        if f in frames:
            # a frame logged twice (should not happen in a straight run); keep both for the report
            f = (f, len(order))
        frames[f] = r
        order.append(f)
    return header, frames, order


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("golden")
    ap.add_argument("candidate")
    ap.add_argument("--max-report", type=int, default=10)
    ap.add_argument("--allow-short", action="store_true", help="candidate may end before the golden does")
    ap.add_argument("--from", dest="start", type=int, default=None, help="compare golden frames >= N only (resumed run)")
    a = ap.parse_args()
    gh, g, gorder = load(a.golden)
    ch, c, corder = load(a.candidate)
    if a.start is not None:
        gorder = [f for f in gorder if not isinstance(f, tuple) and f >= a.start]
        g = {f: g[f] for f in gorder}
        if not corder or corder[0] != a.start:
            print("candidate starts at frame %s, not %d" % (corder[0] if corder else None, a.start))
            return 1
    if gh != ch:
        print("header differs:\n  golden    %s\n  candidate %s" % (",".join(gh), ",".join(ch)))
        return 1
    bad = 0
    missing = [f for f in gorder if f not in c]
    extra = [f for f in c if f not in g]
    first = None
    for f in gorder:
        if f not in c:
            continue
        if g[f] != c[f]:
            if first is None:
                first = f
            if bad < a.max_report:
                cols = [gh[i] for i in range(1, len(gh)) if g[f][i] != c[f][i]]
                print("frame %s: differs in %s" % (f, ", ".join(cols)))
                for i in range(1, len(gh)):
                    if g[f][i] != c[f][i]:
                        print("    %-8s golden %s  candidate %s" % (gh[i], g[f][i], c[f][i]))
            bad += 1
    ok = bad == 0
    if missing and not a.allow_short:
        print("%d golden frame(s) missing from candidate (first %s, last %s)" % (len(missing), missing[0], missing[-1]))
        ok = False
    if extra and not a.allow_short:
        print("%d candidate frame(s) not in golden" % len(extra))
        ok = False
    compared = len(gorder) - len(missing)
    if ok:
        print("OK: %d frame(s) identical (%s vs %s)" % (compared, a.golden, a.candidate))
        return 0
    if bad:
        print("MISMATCH: %d of %d compared frame(s) differ; first divergence at frame %s" % (bad, compared, first))
    return 1


if __name__ == "__main__":
    sys.exit(main())
