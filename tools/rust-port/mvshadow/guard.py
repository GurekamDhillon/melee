"""The union-drift guard: run the mvshadow-instrumented engine over the goldens, the coverage scenarios and a replay set,
and fail on any drifted-read site that tools/rust-port/mvshadow/known.tsv does not list (each known site carries the
reason it is harmless). Per round on a build machine, not in every gate: it builds its own instrumented exe.

    python tools/rust-port/mvshadow/guard.py [--replays DIR] [--jobs N] [--scenarios a,b] [--out DIR]

Scenarios: the 4 goldens, every row of docs/rust-port/golden/coverage/scenarios.tsv, and the guard's own
tools/rust-port/mvshadow/scenarios.tsv (fixed-site regressions such as pikachu_thunder_self).

Needs GW_ISO_VANILLA (never printed), LIBCLANG and GALE_INCLUDE as instrument.py, cargo on PATH (the pinned nightly).
It instruments a scratch copy of HEAD (committed state only) under build/mvshadow (or --out), never the working tree.
A site is (file, enclosing fn, mv path); line numbers do not count, so an unrelated edit does not make a site new.
Exit 0 when every site is known, 1 when one is new (listed), 2 on a setup error. Scenario failures that the
uninstrumented engine has as well (a missing mod, the second disc) are reported, not failed on.
"""
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
KNOWN = HERE / "known.tsv"
GOLDENS = ["pad_falcon_fox_fd", "pad_marth_sheik_bf", "cpu_fox_falco_dl", "cpu_peach_samus_ys_items"]


def arg(name, default=None):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def run(cmd, **kw):
    r = subprocess.run(cmd, **kw)
    if r.returncode:
        raise SystemExit(f"guard: {cmd[0]} failed ({r.returncode})")
    return r


def enclosing_fn(src, line):
    lines = src.splitlines()
    for i in range(min(line, len(lines)) - 1, -1, -1):
        m = re.search(r"\bfn (\w+)", lines[i])
        if m:
            return m.group(1)
    return "?"


def main():
    if not os.environ.get("GW_ISO_VANILLA"):
        print("guard: GW_ISO_VANILLA not set", file=sys.stderr)
        sys.exit(2)
    out = Path(arg("--out", ROOT / "build/mvshadow")).resolve()
    tree, logs = out / "tree", out / "logs"
    jobs = int(arg("--jobs", "4"))
    if tree.exists():
        run(["rm", "-rf", str(tree)])
    tree.mkdir(parents=True)
    logs.mkdir(parents=True, exist_ok=True)
    for f in logs.glob("*.log"):
        f.unlink()
    # committed HEAD only: what is checked is what was committed
    archive = subprocess.run(["git", "-C", str(ROOT), "archive", "HEAD"], capture_output=True, check=True).stdout
    run(["tar", "-x", "-C", str(tree)], input=archive)
    run([sys.executable, str(tree / "tools/rust-port/mvshadow/instrument.py"), str(tree),
         "--unchecked", str(out / "unchecked.tsv")])
    target = out / "cargo"
    run(["cargo", "build", "-q", "--release", "-p", "melee", "--bin", "melee", "--features", "melee_core/mvshadow"],
        cwd=tree / "rust", env={**os.environ, "CARGO_TARGET_DIR": str(target)})
    exe = target / "release" / ("melee.exe" if os.name == "nt" else "melee")
    env = {**os.environ, "MELEE_WINDOW_HIDDEN": "1", "MELEE_WINDOW_UNFOCUSED": "1", "MELEE_VOLUME": "0",
           "MELEE_WINDOW_TITLE": "mvshadow guard"}
    iso = os.environ["GW_ISO_VANILLA"]

    tsv = tree / "docs/rust-port/golden/coverage/scenarios.tsv"
    cov = [(l.split("\t")[0], l.split("\t")[2]) for l in tsv.read_text().splitlines() if l and not l.startswith("#")]
    runs = [(s, "3601") for s in GOLDENS] + cov
    # the guard's own scenarios (regressions for fixed sites: pikachu_thunder_self), read from its table
    extra = tree / "tools/rust-port/mvshadow/scenarios.tsv"
    own = [(l.split("\t")[0], l.split("\t")[2]) for l in extra.read_text().splitlines() if l and not l.startswith("#")]
    runs += own
    own_names = {n for n, _ in own}
    only = arg("--scenarios")
    if only:
        runs = [r for r in runs if r[0] in only.split(",")]

    def scenario(r):
        name, frames = r
        e = {**env, "MELEE_SCENARIO_TSV": str(extra)} if name in own_names else env
        with open(logs / f"{name}.log", "w") as lf:
            p = subprocess.run([str(exe), "golden", "--engine", "tu", "--iso", iso, "--scenario", name, "--frames", frames,
                                "--out", str(logs / f"{name}.csv")], stdout=subprocess.DEVNULL, stderr=lf, cwd=tree,
                               env=e, timeout=1200)
        return name, p.returncode

    def replay(path):
        with open(logs / f"slp_{path.stem}.log", "w") as lf:
            p = subprocess.run([str(exe), "slpcheck", "--iso", iso, "--slp", str(path)], stdout=lf, stderr=lf, cwd=tree,
                               env=env, timeout=1200)
        return path.stem, p.returncode

    with ThreadPoolExecutor(jobs) as ex:
        res = list(ex.map(scenario, runs))
        reps = sorted([*Path(arg("--replays")).glob("*.slp"), *Path(arg("--replays")).glob("*.slpz")]) if arg("--replays") else []
        res += list(ex.map(replay, reps))
    failed = [n for n, rc in res if rc]

    sites = {}
    srcs = {}
    for lf in sorted(logs.glob("*.log")):
        for line in lf.read_text(errors="replace").splitlines():
            m = re.match(r"MVSHADOW (\S+):(\d+) (\S+) ", line)
            if not m:
                continue
            f, ln, path = m.group(1), int(m.group(2)), m.group(3)
            src = srcs.setdefault(f, (tree / "rust" / f).read_text(errors="replace") if (tree / "rust" / f).exists() else "")
            key = (f, enclosing_fn(src, ln), path)
            sites.setdefault(key, set()).add(lf.stem)
    known = {}
    for l in KNOWN.read_text().splitlines() if KNOWN.exists() else []:
        if l and not l.startswith("#"):
            f, fn, path, why = (l.split("\t") + [""] * 4)[:4]
            known[(f, fn, path)] = why
    new = {k: v for k, v in sites.items() if k not in known}
    gone = [k for k in known if k not in sites]
    print(f"guard: {len(runs)} scenarios + {len(reps)} replays; {len(failed)} did not exit 0 "
          f"(compare with an uninstrumented run before blaming the guard): {' '.join(failed[:12])}")
    print(f"guard: {len(sites)} drifted-read sites, {len(sites) - len(new)} known, {len(new)} new, "
          f"{len(gone)} known ones not hit this run")
    for (f, fn, path), where in sorted(new.items()):
        print(f"NEW\t{f}\t{fn}\t{path}\t({len(where)} runs: {', '.join(sorted(where)[:4])})")
    sys.exit(1 if new else 0)


if __name__ == "__main__":
    main()
