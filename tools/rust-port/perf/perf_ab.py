#!/usr/bin/env python3
"""perf-ab summary (docs/rust-port/notes/profiling.md): compare frame profiles of two builds run alternately.

  python perf_ab.py RUNS_DIR [--base NAME] [--head NAME]

RUNS_DIR holds <side>-<label>-<i>.csv frame profiles (MELEE_PROFILE=1), side = base | head, i = run number. For each
label and phase it prints the median over runs of each run's median and p99 (ms), and head - base: the median of
the paired differences (run i of head minus run i of base) with their range [min, max] over the pairs. A range that
contains 0 is "within noise"; the base runs' own spread (max - min of their run medians) is printed beside it.
Frames over 16.7 ms of game work are counted per run, clean and contended (the profile's `contended` column).
Exit 0 always (a report, not a gate); 2 on bad input.
"""
import csv, glob, os, re, statistics, sys
from collections import defaultdict

PHASES = [
    ("frame interval", lambda r: f(r, "interval_ms")),
    ("game work", lambda r: f(r, "interval_ms") - f(r, "pacing_sleep_ms") - f(r, "display_wait_ms")),
    ("tick (sum)", lambda r: f(r, "tick_ms")),
    ("tick (longest)", lambda r: f(r, "tick_max_ms")),
    ("prepare", lambda r: f(r, "prepare_ms")),
    ("draw", lambda r: f(r, "draw_ms")),
    ("display wait", lambda r: f(r, "display_wait_ms")),
    ("present", lambda r: f(r, "present_ms")),
    ("ui", lambda r: f(r, "ui_ms")),
    ("wasm mods", lambda r: f(r, "mods_ms")),
    ("unaccounted", lambda r: f(r, "unaccounted_ms")),
    ("game thread CPU", lambda r: f(r, "thread_cpu_ms")),
]
BUDGET = 16.7


def f(r, k):
    try:
        return float(r.get(k) or 0)
    except ValueError:
        return 0.0


def q(v, p):
    if not v:
        return 0.0
    s = sorted(v)
    return s[min(len(s) - 1, int(p * (len(s) - 1) + 0.5))]


def run_stats(path):
    rows = list(csv.DictReader(open(path, newline="")))[1:]  # the first present is the window's creation
    st = {}
    for name, fn in PHASES:
        v = [fn(r) for r in rows]
        st[name] = (statistics.median(v) if v else 0.0, q(v, 0.99))
    clean = cont = 0
    for r in rows:
        work = f(r, "interval_ms") - f(r, "pacing_sleep_ms") - f(r, "display_wait_ms")
        if work > BUDGET or f(r, "tick_max_ms") > BUDGET:
            if r.get("contended") == "1":
                cont += 1
            else:
                clean += 1
    st["_slow"] = (clean, cont)
    st["_n"] = len(rows)
    return st


def main():
    a = sys.argv[1:]
    if not a:
        print(__doc__)
        return 2
    d = a[0]
    opt = lambda k, dflt: a[a.index(k) + 1] if k in a else dflt
    names = {"base": opt("--base", "base"), "head": opt("--head", "head")}
    runs = defaultdict(dict)  # (label) -> {(side, i): stats}
    for p in sorted(glob.glob(os.path.join(d, "*.csv"))):
        m = re.match(r"(base|head)-(.+)-(\d+)\.csv$", os.path.basename(p))
        if not m:
            continue
        runs[m.group(2)][(m.group(1), int(m.group(3)))] = run_stats(p)
    if not runs:
        print(f"perf_ab: no <side>-<label>-<i>.csv profiles in {d}")
        return 2
    print(f"perf-ab: {names['base']} vs {names['head']}")
    for lab, R in sorted(runs.items()):
        idx = sorted({i for (_, i) in R})
        pairs = [i for i in idx if ("base", i) in R and ("head", i) in R]
        nb = sum(1 for k in R if k[0] == "base")
        nh = sum(1 for k in R if k[0] == "head")
        print(f"\n== {lab}: {nb} base runs, {nh} head runs, {len(pairs)} pairs; frames per run {statistics.median([s['_n'] for s in R.values()]):.0f}")
        print(f"  {'phase (ms)':<18} {'base p50':>8} {'head p50':>8} {'base p99':>8} {'head p99':>8}   {'p50 diff [range]':<24} {'p99 diff [range]':<24} {'base spread':>11}")
        for name, _ in PHASES:
            side = lambda s, k: [R[(s, i)][name][k] for i in idx if (s, i) in R]
            b50, h50, b99, h99 = side("base", 0), side("head", 0), side("base", 1), side("head", 1)
            if not b50 or not h50:
                continue
            d50 = [R[("head", i)][name][0] - R[("base", i)][name][0] for i in pairs]
            d99 = [R[("head", i)][name][1] - R[("base", i)][name][1] for i in pairs]
            fmt = lambda dv: (f"{statistics.median(dv):+.3f} [{min(dv):+.3f},{max(dv):+.3f}]" + ("" if min(dv) <= 0 <= max(dv) else " *")) if dv else "-"
            print(f"  {name:<18} {statistics.median(b50):8.3f} {statistics.median(h50):8.3f} {statistics.median(b99):8.3f} {statistics.median(h99):8.3f}   {fmt(d50):<24} {fmt(d99):<24} {max(b50) - min(b50):11.3f}")
        for s in ("base", "head"):
            sl = [R[(s, i)]["_slow"] for i in idx if (s, i) in R]
            print(f"  {names[s]}: frames over {BUDGET} ms per run: clean {[c for c, _ in sl]}, contended {[c for _, c in sl]}")
    print("\n  * = the paired differences all have one sign (outside the run-to-run noise)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
