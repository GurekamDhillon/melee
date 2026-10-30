#!/usr/bin/env python3
"""Coverage report: for every coverage scenario, how many frames the Rust build matches the C golden.

    coverage_report.py [--run] [--melee EXE] [--iso DISC.iso] [--group G] [--frames N] [--md] [--out FILE]

Scenarios come from docs/rust-port/golden/coverage/scenarios.json; goldens are docs/rust-port/golden/coverage/<name>.csv
(recorded by tools/rust-port/golden/coverage/record_coverage.sh). A scenario is "wired" when `melee golden --scenario NAME`
accepts it (the arms of the scenario match in rust/melee/src/main.rs `fn golden`); all others print "not wired".

Without --run nothing is executed: wired scenarios show "pending" (no Rust run yet). With --run each wired scenario is run
(`melee golden --iso ISO --out TMP.csv --scenario NAME [--idle-pads for cpu*]`) and compared with compare_golden.py
--allow-short; the matching prefix length is reported as "frames matching of golden rows". --iso defaults to $GW_ISO_VANILLA
(never printed). Output is a per-scenario table, per-group totals and the pass-rate table of docs/rust-port/STATUS.md.
"""
import argparse, csv, json, os, re, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
GOLD = os.path.join(ROOT, "docs", "rust-port", "golden", "coverage")
MAIN_RS = os.path.join(ROOT, "rust", "melee", "src", "main.rs")
COMPARE = os.path.join(ROOT, "tools", "rust-port", "compare_golden.py")


def wired_names():
    """Scenario names accepted by `melee golden` (string-literal arms of the scenario match in fn golden)."""
    try:
        src = open(MAIN_RS, encoding="utf-8").read()
    except OSError:
        return set()
    i = src.find("fn golden(")
    j = src.find("\nfn ", i + 1)
    body = src[i:j if j > 0 else len(src)]
    k = body.find("match scenario.as_str()")
    if k < 0:
        return set()
    arms = body[k:]
    arms = arms[: arms.find("s => {")] if "s => {" in arms else arms
    return set(re.findall(r'^\s*"([A-Za-z0-9_]+)"\s*=>', arms, re.M))


def rows(path):
    with open(path, newline="") as f:
        return max(0, sum(1 for _ in f) - 1)


def match_prefix(golden, cand):
    """Frames matching from 0: the length of the identical row prefix (same rule as compare_golden --allow-short)."""
    n = 0
    with open(golden, newline="") as a, open(cand, newline="") as b:
        ra, rb = csv.reader(a), csv.reader(b)
        next(ra, None); next(rb, None)
        for x, y in zip(ra, rb):
            if x != y:
                break
            n += 1
    return n


def run_one(exe, iso, name, golden, frames, timeout):
    with tempfile.TemporaryDirectory() as td:
        out = os.path.join(td, name + ".csv")
        cmd = [exe, "golden", "--iso", iso, "--out", out, "--scenario", name]
        if name.startswith("cpu"):
            cmd.append("--idle-pads")
        if frames:
            cmd += ["--frames", str(frames)]
        try:
            p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            return None, "timeout"
        if not os.path.exists(out):
            tail = (p.stderr or "").strip().splitlines()[-1:] or ["no output"]
            return 0, tail[0][:80]
        n = match_prefix(golden, out)
        return n, ("full" if n >= rows(golden) else "stops")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", action="store_true", help="run wired scenarios with the Rust build (long)")
    ap.add_argument("--melee", default=os.path.join(ROOT, "build", "cargo", "release", "melee.exe"))
    ap.add_argument("--iso", default=os.environ.get("GW_ISO_VANILLA", ""))
    ap.add_argument("--group")
    ap.add_argument("--frames", type=int)
    ap.add_argument("--timeout", type=int, default=1800)
    ap.add_argument("--md", action="store_true", help="markdown tables")
    ap.add_argument("--out")
    a = ap.parse_args()
    if a.run and not (a.iso and os.path.exists(a.melee)):
        sys.exit("coverage_report: --run needs --iso/$GW_ISO_VANILLA and a built melee (--melee)")
    d = json.load(open(os.path.join(GOLD, "scenarios.json")))
    wired = wired_names()
    lines, tot = [], {}
    for name, v in d["scenarios"].items():
        g = v.get("group", "?")
        if a.group and g != a.group:
            continue
        csvp = os.path.join(GOLD, name + ".csv")
        st = v.get("status", "ready")
        t = tot.setdefault(g, dict(total=0, recorded=0, failed=0, blocked=0, wired=0, frames=0, matched=0))
        t["total"] += 1
        if st == "blocked":
            t["blocked"] += 1
            lines.append((g, name, "blocked", "", "", "no scene keyword"))
            continue
        if not os.path.exists(csvp):
            t["failed"] += 1
            lines.append((g, name, "no golden", "", "", "C recording failed"))
            continue
        t["recorded"] += 1
        gr = rows(csvp)
        t["frames"] += gr
        if name not in wired:
            lines.append((g, name, "recorded", gr, "not wired", ""))
            continue
        t["wired"] += 1
        if a.run:
            n, note = run_one(a.melee, a.iso, name, csvp, a.frames, a.timeout)
            t["matched"] += n or 0
            lines.append((g, name, "recorded", gr, "timeout" if n is None else n, note))
        else:
            lines.append((g, name, "recorded", gr, "pending", "not run"))
    out = []
    sep = " | " if a.md else "  "
    hdr = ("group", "scenario", "C golden", "rows", "rust frames", "note")
    if a.md:
        out += ["| " + " | ".join(hdr) + " |", "|" + "---|" * len(hdr)]
    for l in lines:
        out.append(("| " if a.md else "") + sep.join(str(x) for x in l) + (" |" if a.md else ""))
    out.append("")
    th = ("group", "total", "recorded", "failed", "blocked", "wired", "golden rows", "rust frames matched")
    if a.md:
        out += ["| " + " | ".join(th) + " |", "|" + "---|" * len(th)]
    else:
        out.append(sep.join(th))
    for g, t in sorted(tot.items()):
        m = t["matched"] if a.run else "pending consolidation"
        r = [g, t["total"], t["recorded"], t["failed"], t["blocked"], t["wired"], t["frames"], m]
        out.append(("| " if a.md else "") + sep.join(map(str, r)) + (" |" if a.md else ""))
    text = "\n".join(out) + "\n"
    if a.out:
        open(a.out, "w", encoding="utf-8", newline="\n").write(text)
    else:
        sys.stdout.write(text)


if __name__ == "__main__":
    main()
