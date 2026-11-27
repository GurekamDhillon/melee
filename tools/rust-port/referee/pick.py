#!/usr/bin/env python3
"""Replay picker, layer 3 (notes/referee.md section 5): keep a replay only if the referee's retail branch coverage
of the functions under check gains outcomes from it.

usage:
  pick.py --melee MELEE_EXE --mex MEX_IMPORT_EXE --fns a,b,c --work DIR [--cov COV.tsv] REPLAY...
For each replay (in order): `melee slpcheck` with the referee capture on (MELEE_REFEREE_OUT, the capture build:
`--features referee`), then `mex-import referee --capture ... --cov-in COV --cov-out NEW`; a replay that adds branch
outcomes is kept (its capture stays, COV becomes NEW), the others are dropped. Prints one line per replay and writes
DIR/kept.txt (paths) and DIR/cov.tsv (cumulative coverage), DIR/<name>/summary.txt per kept replay.
Game runs: hidden window, no OS input, adapter ignored, volume 3, one at a time, below-normal priority where the OS
allows. The replay paths and captures stay under DIR (git-ignored workspace _build/).
"""
import argparse, os, re, shutil, subprocess, sys


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--melee", required=True)
    ap.add_argument("--mex", required=True)
    ap.add_argument("--fns", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--cov")
    ap.add_argument("--frames", default="")
    ap.add_argument("--max", default="400", help="MELEE_REFEREE_MAX: calls kept per function per replay")
    ap.add_argument("replays", nargs="+")
    a = ap.parse_args()
    a.melee, a.mex = os.path.abspath(a.melee), os.path.abspath(a.mex)
    os.makedirs(a.work, exist_ok=True)
    cov = os.path.join(a.work, "cov.tsv")
    if a.cov and os.path.abspath(a.cov) != os.path.abspath(cov):
        shutil.copy(a.cov, cov)
    kept = []
    for rp in a.replays:
        name = os.path.splitext(os.path.basename(rp))[0]
        d = os.path.join(a.work, name)
        os.makedirs(d, exist_ok=True)
        cap = os.path.join(d, "capture.txt")
        env = dict(os.environ, MELEE_INPUT="n", MELEE_PAD_IGNORE_ADAPTER="1", MELEE_WINDOW_HIDDEN="1", MELEE_VOLUME="3",
                   MELEE_WINDOW_TITLE=f"referee-mvp:pick-{name}", MELEE_REFEREE_OUT=cap, MELEE_REFEREE_FNS=a.fns,
                   MELEE_REFEREE_MAX=a.max)
        cmd = [a.melee, "slpcheck", "--slp", rp, "--full"] + (["--frames", a.frames] if a.frames else [])
        flags = 0x00004000 if os.name == "nt" else 0  # BELOW_NORMAL_PRIORITY_CLASS
        with open(os.path.join(d, "slpcheck.log"), "w") as log:
            subprocess.run(cmd, env=env, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
        new = os.path.join(d, "cov.tsv")
        r = subprocess.run([a.mex, "referee", "--capture", cap, "--fns", a.fns, "--cov-out", new, "--out", d]
                           + (["--cov-in", cov] if os.path.exists(cov) else []), capture_output=True, text=True)
        m = re.search(r"new branch outcomes vs --cov-in: (\d+)", r.stdout)
        gained = int(m.group(1)) if m else 0
        tot = re.search(r"total: .*", r.stdout)
        print(f"{name}: +{gained} outcomes; {tot.group(0) if tot else r.stderr.strip()[:200]}")
        if gained > 0:
            kept.append(rp)
            shutil.copy(new, cov)
        else:
            shutil.rmtree(d, ignore_errors=True)
    with open(os.path.join(a.work, "kept.txt"), "w") as f:
        f.write("".join(p + "\n" for p in kept))
    print(f"pick: kept {len(kept)} of {len(a.replays)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
