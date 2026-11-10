#!/usr/bin/env python3
"""Per-character summary of slpcheck runs: char_report.py OUT_ROOT  (OUT_ROOT/<char>/all.csv from run_all.sh).

Prints, per character directory: replays run, status counts, fighter-frames compared vs replay frames, frame-exact
files, and every first divergence (frame, port, field, engine vs replay, the engine log's action at that point when
SLPCHECK prints it). No player names: files are named by Lunar replay id.
"""
import csv, os, sys, re, collections


def main():
    root = sys.argv[1]
    tot = collections.Counter()
    for ch in sorted(os.listdir(root)):
        p = os.path.join(root, ch, "all.csv")
        if not os.path.exists(p):
            continue
        rows = list(csv.DictReader(open(p, newline="")))
        st = collections.Counter(r["status"] for r in rows)
        rf = sum(int(r["replay_frames"] or 0) for r in rows)
        last = sum(max(0, int(r["last_frame"] or -123) + 123) for r in rows)
        fc = sum(int(r["frames_compared"] or 0) for r in rows)
        print(f"## {ch}: {len(rows)} run, " + ", ".join(f"{k} {v}" for k, v in st.most_common())
              + f"; matched frames {last}/{rf} ({100 * last / max(rf, 1):.1f}%), fighter-frames {fc}")
        tot.update({"run": len(rows), "match": st["match"], "rf": rf, "last": last})
        for r in sorted(rows, key=lambda r: r["first_field"]):
            if r["status"] == "match":
                continue
            log = os.path.join(root, ch, r["file"].replace(".slp", ".log"))
            act = ""
            if os.path.exists(log):
                m = re.search(r"action[= ](\d+)", open(log, errors="replace").read())
                act = m.group(1) if m else ""
            print(f"  {r['file'][:24]} {r['status']} stage {r['stage']} {r['roster']} f{r['first_frame']} p{r['first_port']} "
                  f"{r['first_field']} eng={r['engine_value']} rec={r['replay_value']} act={act} {r['panic'][:80]}")
    print(f"TOTAL {tot['run']} run, {tot['match']} frame-exact, matched frames {tot['last']}/{tot['rf']} "
          f"({100 * tot['last'] / max(tot['rf'], 1):.1f}%)")


if __name__ == "__main__":
    main()
