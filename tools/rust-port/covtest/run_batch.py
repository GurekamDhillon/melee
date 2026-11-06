# SPDX-License-Identifier: MIT
"""Record and check a batch of CPU seed matches in parallel lanes (notes/coverage-testing.md).
   python -I run_batch.py CONFIGS.jsonl CORPUS_DIR [--lanes N] [--limit K] [--only SET,...]
CORPUS_DIR (outside git, e.g. _build/covcorpus): slp/<id>.slp, checks/<id>/, lanes/lN/, prof/lN.profdata,
index.jsonl (one line per match: the config, what it covers and both results). Already indexed ids are skipped, so
a batch resumes. Each lane: record.py (one Dolphin), then check.sh (two Dolphin playbacks + the engine twice), then
its coverage profiles merged into prof/lN.profdata (llvm-profdata from rustup's llvm-tools) and deleted.
Default lanes: max(2, cores // 5) (each lane peaks at two Dolphins and one engine; all at Idle priority)."""
import json, os, subprocess, sys, threading, time

here = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.expanduser("~/.rustup/toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin")
PROFDATA = os.environ.get("LLVM_PROFDATA", os.path.join(TOOLS, "llvm-profdata.exe"))
BASH = os.environ.get("BASH_EXE", "C:/Program Files/Git/usr/bin/bash.exe")  # not WSL's bash.exe


def arg(k, d=None):
    return sys.argv[sys.argv.index(k) + 1] if k in sys.argv else d


def main():
    cfgs = [json.loads(l) for l in open(sys.argv[1])]
    corpus = os.path.abspath(sys.argv[2])
    only = arg("--only")
    if only:
        cfgs = [c for c in cfgs if c["set"] in only.split(",")]
    if arg("--limit"):
        cfgs = cfgs[:int(arg("--limit"))]
    lanes = int(arg("--lanes", max(2, (os.cpu_count() or 4) // 5)))
    for d in ("slp", "checks", "lanes", "prof"):
        os.makedirs(os.path.join(corpus, d), exist_ok=True)
    idx_path = os.path.join(corpus, "index.jsonl")
    done = set()
    if os.path.exists(idx_path):
        done = {json.loads(l)["id"] for l in open(idx_path)}
    todo = [c for c in cfgs if c["id"] not in done]
    print(f"{len(todo)} to run ({len(done)} already indexed), {lanes} lanes", flush=True)
    lock = threading.Lock()
    t0 = time.time()
    count = [0]

    def lane(n):
        ld = os.path.join(corpus, "lanes", f"l{n}")
        pd = os.path.join(corpus, "prof", f"l{n}.profdata")
        while True:
            with lock:
                if not todo:
                    return
                c = todo.pop(0)
            slp = os.path.join(corpus, "slp", c["id"] + ".slp")
            chk = os.path.join(corpus, "checks", c["id"])
            rec = "ok"
            if not os.path.exists(slp):
                r = subprocess.run([sys.executable, "-I", os.path.join(here, "record.py"), json.dumps(c), slp, ld, "--timeout", "300"],
                                   capture_output=True, text=True)
                rec = (r.stdout.strip().splitlines() or ["RECORD error: no output"])[-1]
            res = {}
            if os.path.exists(slp):
                env = dict(os.environ, PROFRAW_DIR=ld, USER_BASE=ld)
                r = subprocess.run([BASH, os.path.join(here, "check.sh"), slp, chk], capture_output=True, text=True, env=env)
                line = ([l for l in r.stdout.splitlines() if l.startswith("COVCHECK")] or ["COVCHECK ? error"])[-1]
                res["covcheck"] = line
                for tok in line.split()[2:]:
                    k, _, v = tok.partition("=")
                    res[k] = v
                raws = [os.path.join(ld, f) for f in os.listdir(ld) if f.endswith(".profraw")]
                if raws:
                    ins = raws + ([pd] if os.path.exists(pd) else [])
                    m = subprocess.run([PROFDATA, "merge", "-sparse", *ins, "-o", pd + ".new"], capture_output=True, text=True)
                    if m.returncode == 0:
                        os.replace(pd + ".new", pd)
                        for f in raws:
                            os.remove(f)
                    else:
                        res["profdata"] = m.stderr.strip()[:200]
            covers = {"chars": [p[0] for p in c["players"]], "stage": c["stage"], "levels": sorted({p[1] for p in c["players"]}),
                      "items": c["items"], "teams": c["teams"], "players": len(c["players"])}
            row = dict(id=c["id"], set=c["set"], config=c, covers=covers, record=rec, **res)
            with lock:
                with open(idx_path, "a") as f:
                    f.write(json.dumps(row) + "\n")
                count[0] += 1
                el = time.time() - t0
                print(f"[{count[0]}/{count[0] + len(todo)}] {el / 60:.1f} min  {c['id']}: {res.get('covcheck', rec)}", flush=True)

    th = [threading.Thread(target=lane, args=(i,)) for i in range(lanes)]
    for t in th:
        t.start()
    for t in th:
        t.join()
    print("BATCH done", flush=True)


if __name__ == "__main__":
    main()
