#!/usr/bin/env python3
"""Read docs/rust-port/golden/coverage/scenarios.json for the recorder.
  scn.py names [--group G] [--status S]      one scenario name per line
  scn.py tsv [--rust] NAME...                name <TAB> pad <TAB> scene <TAB> seed <TAB> max_seconds <TAB> env(k=v,...)
                                             (a scenario's "mods": [id, ...] names shipped mods of pc/geno/mods: they are
                                             copied into build/covmods/NAME, git-ignored, which becomes MELEE_MODS_DIR;
                                             --rust, the Rust engine's runs: a package with a geno 12 copy under
                                             rust/mods/geno_fighters/packages/<id> gets that geno.json and its wasm module,
                                             since the Rust engine runs no Lua; the C recorder keeps pc/geno/mods as shipped)
  scn.py batch N                             names of recorded batch N (from the file's "batches")
"""
import json, os, shutil, sys
sys.stdout.reconfigure(newline=chr(10))
here = os.path.dirname(os.path.abspath(__file__))
path = os.path.join(here, "..", "..", "..", "..", "docs", "rust-port", "golden", "coverage", "scenarios.json")
d = json.load(open(path))
sc = d["scenarios"]
a = sys.argv[1:]
if a[0] == "names":
    g = a[a.index("--group") + 1] if "--group" in a else None
    s = a[a.index("--status") + 1] if "--status" in a else None
    for n, v in sc.items():
        if (g is None or v.get("group") == g) and (s is None or v.get("status", "ready") == s):
            print(n)
elif a[0] == "tsv":
    rust = "--rust" in a
    for n in [x for x in a[1:] if x != "--rust"]:
        v = sc[n]
        # "{wt}" in an env value: this worktree (Windows form), so a run root elsewhere still finds the scenario's mods
        wt = os.path.abspath(os.path.join(here, "..", "..", "..", "..")).replace("\\", "/")
        # "{ws}": the workspace (GW_WORKSPACE), where local generated mod sets live (_build/mex-mods, never committed)
        ws = os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace").replace("\\", "/")
        envd = dict(v.get("env", {}))
        if v.get("iso"):  # a disc other than the vanilla one: the recorders read GW_ISO_<iso> from .env
            envd["COV_ISO"] = v["iso"]
        if v.get("mods"):
            # only the named mods mounted (gw_mods mounts every folder of the dir): a fresh copy per scenario
            md = os.path.join(wt, "build", "covmods", n)
            shutil.rmtree(md, ignore_errors=True)
            for m in v["mods"]:
                shutil.copytree(os.path.join(wt, "pc", "geno", "mods", m), os.path.join(md, m))
                over = os.path.join(wt, "rust", "mods", "geno_fighters", "packages", m, "geno.json")
                if rust and os.path.exists(over):
                    shutil.copyfile(over, os.path.join(md, m, "geno.json"))
                    for f in json.load(open(over))["fighters"]:
                        rel = f.get("wasm", {}).get("module")
                        if rel:
                            os.makedirs(os.path.join(md, m, os.path.dirname(rel)), exist_ok=True)
                            shutil.copyfile(os.path.join(wt, "rust", "mods", "geno_fighters", os.path.basename(rel)), os.path.join(md, m, rel))
            envd["MELEE_MODS_DIR"] = md.replace("\\", "/")
        env = ",".join("%s=%s" % (k, val.replace("{wt}", wt).replace("{ws}", ws)) for k, val in sorted(envd.items()))
        print("\t".join([n, v["pad"], v.get("scene", ""), str(v.get("seed", d["defaults"]["seed"])),
                         str(v.get("max_seconds", d["defaults"]["max_seconds"])), env]))
elif a[0] == "batch":
    for n in d["batches"][a[1]]:
        print(n)
