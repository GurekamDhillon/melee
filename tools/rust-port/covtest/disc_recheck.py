# SPDX-License-Identifier: MIT
"""Re-run the engine side of every checked match with the discrimination build (DISC=1 build_cov.sh):
   python -I disc_recheck.py CORPUS_DIR [--jobs N] [--only SET,...]
For checks/<id>/{inputs,ai}: `slpcheck --cpu-slots|--cpu-ai` with MELEE_DISC_OUT=<mode>/disc.tsv, and the NHV1 rows
compared again with the run's kept dolphin.csv (so the verification frame is this exe's). Skips runs that have one."""
import json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

here = os.path.dirname(os.path.abspath(__file__))
WS = os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace")
EXE = os.environ.get("MELEE_EXE", os.path.join(WS, "_build", "cargo-agents", "covtest", "covdisc", "melee.exe"))
sys.path.insert(0, here)
import dol  # noqa: E402


def one(corpus, id_):
    chk = os.path.join(corpus, "checks", id_)
    slp = os.path.join(corpus, "slp", id_ + ".slp")
    if not os.path.exists(os.path.join(chk, "dolphin.csv")):
        return
    for mode, flag in (("inputs", "--cpu-slots"), ("ai", "--cpu-ai")):
        d = os.path.join(chk, mode)
        if os.path.exists(os.path.join(d, "disc.tsv")):
            continue
        os.makedirs(d, exist_ok=True)
        env = dict(os.environ, MELEE_DISC_OUT=os.path.join(d, "disc.tsv"), LLVM_PROFILE_FILE=os.path.join(d, "disc.%p.profraw"),
                   MELEE_WINDOW_UNFOCUSED="1", MELEE_VOLUME="3")
        subprocess.run([EXE, "slpcheck", "--slp", slp, "--full", "--iso", dol.iso_path(), flag, "--nhv1-out", os.path.join(d, "rust_disc.csv")],
                       env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, creationflags=0x40)
        with open(os.path.join(d, "compare.txt"), "w") as f:
            subprocess.run([sys.executable, os.path.join(here, "..", "compare_golden.py"), os.path.join(d, "rust_disc.csv"),
                            os.path.join(chk, "dolphin.csv"), "--ignore-seed", "--allow-short", "--max-report", "1"], stdout=f, stderr=subprocess.STDOUT)
        for x in os.listdir(d):
            if x.endswith(".profraw"):
                os.remove(os.path.join(d, x))


def main():
    corpus = sys.argv[1]
    jobs = int(sys.argv[sys.argv.index("--jobs") + 1]) if "--jobs" in sys.argv else 6
    only = sys.argv[sys.argv.index("--only") + 1].split(",") if "--only" in sys.argv else None
    ids = [json.loads(l) for l in open(os.path.join(corpus, "index.jsonl"))]
    ids = [r["id"] for r in ids if not only or r["set"] in only]
    with ThreadPoolExecutor(jobs) as ex:
        for k, _ in enumerate(ex.map(lambda i: one(corpus, i), ids)):
            if k % 50 == 0:
                print(f"disc_recheck {k}/{len(ids)}", flush=True)
    print("DISC_RECHECK done")


if __name__ == "__main__":
    main()
