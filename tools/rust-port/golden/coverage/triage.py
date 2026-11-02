#!/usr/bin/env python3
"""Automatic first-divergence triage for the coverage scenarios (notes/triage.md).

  python tools/rust-port/golden/coverage/triage.py [--results R.tsv | --run-all] [-j N] [--summary-only] [NAME...]

For every FAIL row of run_rust.sh (its output TSV, or run it with --run-all; NAME... forces scenarios) it
  1. runs the Rust engine (build/cargo/release) and finds the first divergent golden frame, the differing hash columns
     (compare_golden.py) and whether the engine diverged or stopped (panic/crash);
  2. records the C call trace of the same scenario (trace build in _build/agents/triage, tools/rust-port/trace/
     build_trace_reuse.sh) for the window [frame-LOOKBACK, frame+1] and the C state snapshot of the divergent frame,
     CACHED per scenario under <root>/ctcache/<C build commit>/<scenario>/ (a re-run skips the C side);
  3. runs the Rust engine with the call trace (build/cargo-ct, --features melee_core/calltrace) for the same window and
     runs seqdiff.py (--anchor Fighter_8006A1BC): first frame and first function whose call sequence differs (the trace
     holds function entries only, not arguments; value differences come from step 4);
  4. runs `melee diff --frame N --snap <C snapshot>` and takes the first field the golden hash reads ('*'), else the first field;
  5. resolves the function to its Rust TU file and C source file.
Output: docs/rust-port/golden/coverage/triage.tsv (+ clusters printed, and written with --md FILE).

Env: GW_WORKSPACE (default E:/Projects/Melee Workspace), TRIAGE_ROOT (C trace build root, default <ws>/_build/agents/triage),
MELEE_EXE / MELEE_CT_EXE (Rust exes). Headless and unfocused: no OS input, no window focus. Disc path is read from the
workspace .env and never printed; nothing disc-derived is written outside <root> (snapshots stay there, untracked).
"""
import argparse, csv, json, os, re, shutil, subprocess, sys, time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
WS = Path(os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace"))
ROOT = Path(os.environ.get("TRIAGE_ROOT", WS / "_build/agents/triage"))
COV = REPO / "docs/rust-port/golden/coverage"
HERE = Path(__file__).resolve().parent
EXE = os.environ.get("MELEE_EXE", str(REPO / "build/cargo/release/melee.exe"))
CTEXE = os.environ.get("MELEE_CT_EXE", str(REPO / "build/cargo-ct/release/melee.exe"))
ANCHOR = "Fighter_8006A1BC"
LOOKBACKS = (60, 400)
REF_GAP = 20   # frames just before the divergence are not used to learn inlining noise
BASH = shutil.which("bash") or "bash"


def load_env():
    env = dict(os.environ)
    p = WS / ".env"
    for line in p.read_text(errors="replace").splitlines():
        line = line.strip()
        if line.startswith("export "):
            line = line[7:]
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env.setdefault(k.strip(), v.strip().strip('"').strip("'"))
    env["MELEE_WINDOW_UNFOCUSED"] = "1"
    return env


ENV = load_env()
ISO = ENV.get("GW_ISO_VANILLA", "")


def sh(cmd, env=None, timeout=1800, cwd=None):
    """Run, capture combined output (text), never print it. Returns (rc, text)."""
    for attempt in range(8):
        try:
            r = subprocess.run(cmd, env=env or ENV, cwd=cwd or REPO, capture_output=True, timeout=timeout)
            return r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")
        except subprocess.TimeoutExpired as e:
            return 124, "timeout " + str(e)
        except OSError as e:   # WinError 1455 (paging file) when several engines run at once: wait and retry
            last = e
            time.sleep(15)
    return 125, "spawn failed: " + str(last)


def c_key():
    """Cache key = the C trace build's commit (build-provenance.json), plus a hash prefix when its sources were dirty."""
    j = json.loads((ROOT / "build-provenance.json").read_text())
    k = j["melee_commit"][:9]
    if j.get("source_dirty"):
        k += "-d" + j["source_sha256"][:8]
    return k


def scn_tsv(name):
    rc, out = sh([sys.executable, str(HERE / "scn.py"), "tsv", name])
    f = out.rstrip("\n").split("\t")
    f += [""] * (6 - len(f))
    return dict(pad=f[1], scene=f[2], seed=f[3], secs=f[4], envs=f[5])


def read_rows(path):
    try:
        with open(path, newline="") as f:
            return list(csv.reader(f))
    except OSError:
        return []


def compare(golden, cand):
    rc, out = sh([sys.executable, str(REPO / "tools/rust-port/compare_golden.py"), str(golden), str(cand), "--allow-short", "--print-count", "--max-report", "1"])
    m = re.search(r"^COUNT (\d+)", out, re.M)
    n = int(m.group(1)) if m else 0
    cols = []
    m = re.search(r"^frame \S+: differs in (.*)$", out, re.M)
    if m:
        cols = [c.strip() for c in m.group(1).split(",")]
    return n, cols


def rust_run(s, pre, frames, out_csv, log, ct=None, window=None, exe=EXE):
    env = dict(ENV, MELEE_ENGINE_PRE_DRAWS=str(pre))
    cmd = [exe, "golden", "--engine", "tu", "--iso", ISO, "--scenario", s, "--frames", str(frames), "--out", str(out_csv)]
    if ct:
        cmd += ["--calltrace", str(ct), "--calltrace-frames", window]
    rc, out = sh(cmd, env=env)
    Path(log).write_text(out.replace(ISO, "<iso>") if ISO else out)
    return rc, out


def calibrate_pre(s, golden, work):
    """The same search run_rust.sh does: MELEE_ENGINE_PRE_DRAWS 0..3, first value that makes frames 0..1 match."""
    for k in range(4):
        cal = work / "cal.csv"
        cal.unlink(missing_ok=True)
        rust_run(s, k, 2, cal, work / "cal.log")
        if cal.exists() and cal.stat().st_size and compare(golden, cal)[0] >= 2:
            return k
    return 0


def panic_info(log_text):
    m = re.search(r"panicked at ([^\n]*?):(\d+):(\d+):?\s*\n?([^\n]*)", log_text)
    if m:
        return m.group(1), int(m.group(2)), m.group(4).strip()[:120]
    if re.search(r"access violation|STATUS_ACCESS_VIOLATION|0xc0000005", log_text, re.I):
        return None, 0, "access violation"
    return None, 0, ""


def c_cache_dir(s):
    d = ROOT / "ctcache" / c_key() / s
    d.mkdir(parents=True, exist_ok=True)
    return d


def find_cached_seq(d, lo, hi):
    best = None
    for f in d.glob("c_*-*.seq"):
        m = re.fullmatch(r"c_(\d+)-(\d+)\.seq", f.name)
        if m and int(m.group(1)) <= lo and int(m.group(2)) >= hi:
            if best is None or f.stat().st_size < best.stat().st_size:
                best = f
    return best


def c_run(s, d, lo, hi, snap_frame, want_seq, rows):
    """One C run of the trace build: ordered call list for match frames lo..hi and/or a snapshot of snap_frame. Idempotent."""
    t = scn_tsv(s)
    work = ROOT / "runs-tri" / s
    work.mkdir(parents=True, exist_ok=True)
    sandbox = f"triage-{s}"
    pad = str((REPO / "tools/rust-port/golden").resolve()).replace("/", "\\")
    padw = subprocess.run(["cygpath", "-m", str(REPO / "tools/rust-port/golden")], capture_output=True, text=True).stdout.strip() or pad
    hashlog = work / "c_hash.csv"
    hashlog.unlink(missing_ok=True)
    seq = work / "c.seq"
    seq.unlink(missing_ok=True)
    env = dict(ENV, GW_BUILD_ROOT=str(ROOT), GW_MELEE=str(REPO), MELEE_UNATTENDED="1", MELEE_WINDOW_X="30000", MELEE_WINDOW_Y="30000",
               MELEE_SCRIPTS="0", MELEE_TURBO="1", MELEE_PAD_IGNORE_ADAPTER="1", SDL_WINDOW_ACTIVATE_WHEN_SHOWN="0",
               MELEE_TEST_SEED=t["seed"], MELEE_PAD_SCRIPT=f"{padw}/{t['pad']}", MELEE_NEUTRAL_HASHLOG=hashlog.as_posix())
    if t["scene"]:
        env["MELEE_SCENE"] = t["scene"]
    for e in filter(None, t["envs"].split(",")):
        k, _, v = e.partition("=")
        env[k] = v
    if want_seq:
        env["MELEE_CALLTRACE_SEQ"] = seq.as_posix()
        env["MELEE_CALLTRACE_FRAMES"] = f"{lo}-{hi}"
    snapfile = d / f"{s}.f{snap_frame}.snap"
    if snap_frame is not None and not snapfile.exists():
        env["MELEE_SNAP_DUMP"] = f"{snap_frame}:{d.as_posix()}/{s}.f%d.snap"
    run = [BASH, str(WS / "tools/port/run.sh"), "--max-seconds", t["secs"], sandbox, "--iso", ISO]
    rc, out = sh(run, env=env, timeout=int(t["secs"]) + 600)
    (work / "c.log").write_text(out.replace(ISO, "<iso>") if ISO else out)
    meta = d / "meta.json"
    m = json.loads(meta.read_text()) if meta.exists() else {}
    if hashlog.exists():
        n, _ = compare(COV / f"{s}.csv", hashlog)
        m["c_hash_matches_golden"] = (n >= rows)
        m["c_hash_rows_matching"] = n
    m["c_commit"] = c_key()
    meta.write_text(json.dumps(m))
    if want_seq and seq.exists():
        shutil.move(str(seq), str(d / f"c_{lo}-{hi}.seq"))
    return rc


def noise_names(cseq, cmap, rct, lo, hi, frame):
    """Names whose call COUNT differs per anchored chunk in known-good frames (lo..frame-REF_GAP; the golden hash matched there):
    functions C inlines or the port restructures. Dropped from the sequence compare (they would flag every frame)."""
    sys.path.insert(0, str(REPO / "tools/rust-port/trace"))
    from calltrace import read_map
    syms = read_map(str(cmap))
    nm = lambda n: (re.sub(r"^_?gw_", "", n) if n.startswith(("_gw_", "gw_")) else (n.lstrip("_") or n)).lower()
    cf, rf = defaultdict(list), defaultdict(list)
    for line in Path(cseq).read_text().splitlines():
        if line.startswith("#"):
            continue
        f, rva, q = line.split(",")
        f = int(f)
        if lo <= f <= hi:
            n = syms.get(int(rva, 16), (None, None))[0]
            cf[f].append((int(q), nm(n) if n else "rva_" + rva))
    for line in Path(rct).read_text().splitlines()[1:]:
        f, rest = line.split(",", 1)
        n, q = rest.rsplit(",", 1)
        if lo <= int(f) <= hi:
            rf[int(f)].append((int(q), nm(n)))
    anc = nm(ANCHOR)

    def chunks(d):
        out, cur = [], None
        for f in sorted(d):
            for _, n in sorted(d[f]):
                if n == anc:
                    cur = {}
                    out.append((f, cur))
                if cur is not None:
                    cur[n] = cur.get(n, 0) + 1
        return out
    def seqs(d):
        out, cur = [], None
        for f in sorted(d):
            for _, n in sorted(d[f]):
                if n == anc:
                    cur = []
                    out.append((f, cur))
                if cur is not None:
                    cur.append(n)
        return out
    cc, rc = chunks(cf), chunks(rf)
    bad = set()
    for (fc, a), (fr, b) in zip(cc, rc):
        if fc > frame - REF_GAP:
            break
        for n in set(a) | set(b):
            if a.get(n, 0) != b.get(n, 0):
                bad.add(n)
    # order noise: calls that sit at different places in chunks of known-good frames (frame-boundary work, inlining)
    sc, sr = seqs(cf), seqs(rf)
    for k, ((fc, a), (fr, b)) in enumerate(zip(sc, sr)):
        if fc > frame - REF_GAP or k > 40:
            break
        a = [n for n in a if n not in bad]
        b = [n for n in b if n not in bad]
        for _ in range(400):
            i = next((j for j in range(min(len(a), len(b))) if a[j] != b[j]), None)
            if i is None:
                break
            if b[i + 1:i + 2] == a[i:i + 1]:
                x = {b[i]}
            elif a[i + 1:i + 2] == b[i:i + 1]:
                x = {a[i]}
            else:
                x = {a[i], b[i]}
            bad |= x
            a = [n for n in a if n not in x]
            b = [n for n in b if n not in x]
    return bad


def seqdiff(cseq, cmap, rct, lo, hi, ign=None):
    base = [sys.executable, str(REPO / "tools/rust-port/trace/seqdiff.py"), "--c", str(cseq), "--map", str(cmap), "--rust", str(rct), "--frames", f"{lo}-{hi}", "--context", "8"]
    if ign:
        base += ["--ignore-inlined", str(ign)]
    rc, out = sh(base + ["--anchor", ANCHOR, "--auto-ignore", "40"])
    if "no common frames" in out or rc not in (0, 1):
        rc, out = sh(base)
    return rc, out


def parse_seqdiff(out):
    r = {"found": False}
    m = re.search(r"first difference: (?:chunk (\d+) \(C frame (\d+), Rust frame (\d+)\) |frame (\d+) )position (\d+) \(C (\d+) calls, Rust (\d+) calls\)", out)
    if not m:
        r["text"] = out.strip().splitlines()[-1][:160] if out.strip() else ""
        return r
    r["found"] = True
    r["edge"] = False
    r["frame"] = int(m.group(2) or m.group(4))
    pos = int(m.group(5))
    cl = re.search(r"^\s*C\s*:(.*)$", out, re.M)
    rl = re.search(r"^\s*Rust:(.*)$", out, re.M)
    cl = cl.group(1).split() if cl else []
    rl = rl.group(1).split() if rl else []
    off = min(pos, 8)
    cf = cl[off] if off < len(cl) else None
    rf = rl[off] if off < len(rl) else None
    # who owns the difference: a name one side has an extra call of is the culprit (it is the other side's missing/extra function)
    if cf is None:
        fn, kind = rf, "C sequence ends (Rust runs more)"
    elif rf is None:
        fn, kind = cf, "Rust sequence ends (C calls more)"
    elif rl[off + 1:off + 2] == [cf]:
        fn, kind = rf, "extra call in Rust"
    elif cl[off + 1:off + 2] == [rf]:
        fn, kind = cf, "extra call in C"
    else:
        fn, kind = cf, "different call"
    if cf is None or rf is None:
        r["found"] = False   # one side just ends: the trace window's edge (the C file stops at hi), not a divergence
        r["text"] = f"window edge at C frame {r['frame']} ({kind})"
        return r
    r.update(fn=fn, kind=kind, c=cf, rust=rf, ctx_c=" ".join(cl[max(0, off - 3):off + 3]), ctx_r=" ".join(rl[max(0, off - 3):off + 3]))
    m = re.search(r"auto-ignored: (.*)", out)
    r["auto"] = m.group(1) if m else ""
    return r


_loc = {}


def locate(name):
    """(rust tu file, C file) defining a function name (case-insensitive, as the trace spells it)."""
    if not name:
        return "", ""
    if name in _loc:
        return _loc[name]
    rs = cs = ""
    rc, out = sh(["git", "grep", "-i", "-l", "-E", rf"fn\s+{re.escape(name)}\b", "--", "rust/*/src"])
    files = [l for l in out.splitlines() if l.endswith(".rs")]
    files.sort(key=lambda p: ("/tu/" not in p, p))
    rs = files[0] if files else ""
    rc, out = sh(["git", "grep", "-i", "-l", "-E", rf"^[A-Za-z_][A-Za-z0-9_ \*]*\b{re.escape(name)}\s*\(", "--", "src/*.c", "pc/*.c"])
    files = sorted((l for l in out.splitlines() if l.endswith(".c")), key=lambda p: ("test" in p, p))
    cs = files[0] if files else ""
    _loc[name] = (rs, cs)
    return _loc[name]


def tu_of(rs):
    return Path(rs).stem if rs else ""


def first_field(scn, frame, snap, pre):
    env = dict(ENV, MELEE_ENGINE_PRE_DRAWS=str(pre))
    rc, out = sh([EXE, "diff", "--scenario", scn, "--frame", str(frame), "--snap", str(snap), "--max", "500"], env=env)
    lines = [l for l in out.splitlines() if re.match(r"^\s{2}\S", l) and " gc " in l]
    if not lines:
        return "", (out.strip().splitlines() or [""])[-1][:140], ""
    pick = next((l for l in lines if re.search(r"\s\*\S", l)), lines[0])
    m = re.match(r"^\s+(\S+)\s+(.+?)\s{2,}(\*?)(\S+)\s+gc\s+(\S+)\s+C (.*?)\s+\|\s+Rust (.*)$", pick)
    if not m:
        return pick.strip()[:160], "", ""
    hashed = "hashed " if m.group(3) else ""
    return f"{m.group(1)} {m.group(2).strip()} {m.group(4)} (gc {m.group(5)}): C {m.group(6)} | Rust {m.group(7)}  [{hashed.strip() or 'not hashed'}]"[:260], "", re.sub(r"\[\d+\]", "[]", f"{m.group(1)} {m.group(4)}")


def triage_one(s, args):
    t0 = time.time()
    work = ROOT / "tri-work" / s
    work.mkdir(parents=True, exist_ok=True)
    golden = COV / f"{s}.csv"
    grows = read_rows(golden)
    rows = len(grows) - 1
    row = dict(scenario=s, first_frame="", first_function="", tu="", rust_file="", c_file="", first_field="", kind="", hash_cols="",
               c_trace_cache="", seq_kind="", early_hash_invisible="", field_key="", pre="", note="")
    pre = calibrate_pre(s, golden, work)
    row["pre"] = pre
    rcsv = work / "rust.csv"
    rcsv.unlink(missing_ok=True)
    rc, rout = rust_run(s, pre, rows, rcsv, work / "rust.log")
    n, cols = compare(golden, rcsv) if rcsv.exists() else (0, [])
    crows = max(0, len(read_rows(rcsv)) - 1)
    pfile, pline, pmsg = panic_info(rout)
    stopped = n < rows and n >= crows
    if n >= rows:
        row["note"] = "PASS now"
        return row
    frame = int(grows[1 + n][0]) if n < rows else n
    row["first_frame"] = frame
    row["hash_cols"] = ",".join(cols)
    if stopped:
        row["kind"] = "crash" if (pmsg or rc not in (0,)) else "stop"
        row["note"] = (pmsg or f"exit {rc}")[:120] + (f" @ {Path(pfile).name}:{pline}" if pfile else "")
    else:
        row["kind"] = "divergence"
    # ---- C side (cached) and seq diff over growing windows
    d = c_cache_dir(s)
    cmap = d.parent / "melee-pc.map"
    if not cmap.exists():
        shutil.copy(ROOT / "melee-pc.map", cmap)
    row["c_trace_cache"] = str(d).replace("\\", "/")
    snap_frame = frame if not stopped else None
    res = {"found": False}
    nz = set()
    for lb in LOOKBACKS:
        lo, hi = max(0, frame - lb), frame + 1
        cseq = find_cached_seq(d, lo, hi)
        if cseq is None:
            c_run(s, d, lo, hi, snap_frame, True, rows)
            snap_frame = None if (d / f"{s}.f{frame}.snap").exists() else snap_frame
            cseq = find_cached_seq(d, lo, hi)
        if cseq is None:
            row["seq_kind"] = "no C trace (C run failed, see tri-work)"
            break
        rct = work / "rust.ct"
        rct.unlink(missing_ok=True)
        rust_run(s, pre, hi + 1, work / "rust_ct.csv", work / "rust_ct.log", ct=rct, window=f"{lo}-{hi}", exe=CTEXE)
        if not rct.exists():
            row["seq_kind"] = "no Rust trace"
            break
        ign = work / "noise.txt"
        try:
            nz = noise_names(cseq, cmap, rct, lo, hi, frame)
        except Exception as e:  # noqa
            nz = set()
        ign.write_text(chr(10).join(sorted(nz)) + chr(10))
        early = []
        for _ in range(12):
            rc2, sout = seqdiff(cseq, cmap, rct, lo, hi, ign)
            (work / "seqdiff.txt").write_text(sout)
            res = parse_seqdiff(sout)
            # a difference more than REF_GAP frames before the hash divergence did not matter to the hash for that long (a C function the
            # port calls under another name, an inlined helper): record it, learn the pair as noise and look further. Closer ones are kept
            if res["found"] and res["frame"] < frame - REF_GAP:
                early.append(f"{res['c']}/{res['rust']}@{res['frame']}")
                nz |= {x for x in (res["c"], res["rust"]) if x}
                ign.write_text(chr(10).join(sorted(nz)) + chr(10))
                continue
            break
        row["early_hash_invisible"] = " ".join(early)
        if res["found"]:
            break
    # ---- snapshot for the field diff
    meta = json.loads((d / "meta.json").read_text()) if (d / "meta.json").exists() else {}
    if not meta.get("c_hash_matches_golden", True):
        row["note"] += "; C trace build hash != golden (C sources changed?)"
    if res.get("found"):
        row["note"] += f"; {len(nz)} noise names dropped"
        row["first_function"] = res["fn"]
        row["seq_kind"] = f"C frame {res['frame']}: {res['kind']} (C {res['c']} / Rust {res['rust']})"
        if res.get("auto"):
            row["note"] += "; auto-ignored: " + res["auto"][:80]
    elif not row["seq_kind"]:
        row["seq_kind"] = "no call-sequence difference in window (" + res.get("text", "") + ")"
    if stopped and pfile:
        # crash: the panic site names the TU even when the sequence diff shows nothing
        rsrc = Path(REPO / "rust" / pfile.replace("\\", "/"))
        guess = ""
        if rsrc.exists():
            ls = rsrc.read_text(errors="replace").splitlines()
            for i in range(min(pline, len(ls)) - 1, -1, -1):
                m = re.search(r"\bfn\s+(\w+)", ls[i])
                if m:
                    guess = m.group(1)
                    break
        if guess and not row["first_function"]:
            row["first_function"] = guess.lower()
            row["seq_kind"] += "; function from panic site"
        row["rust_file"] = ("rust/" + pfile).replace("\\", "/") if rsrc.exists() else ""
    if row["first_function"]:
        rs, cs = locate(row["first_function"])
        row["rust_file"] = row["rust_file"] or rs
        row["c_file"] = cs
        row["tu"] = tu_of(row["rust_file"]) or ("C:" + Path(cs).stem if cs else "")
    snap = d / f"{s}.f{frame}.snap"
    if not stopped:
        if not snap.exists():
            c_run(s, d, 0, 0, frame, False, rows)
        if snap.exists():
            ff, why, fk = first_field(s, frame, snap, pre)
            row["field_key"] = fk
            row["first_field"] = ff or ("(melee diff: " + why + ")")
        else:
            row["first_field"] = "(no C snapshot)"
    else:
        row["first_field"] = "(engine stopped before the frame)" if not cols else ""
    row["note"] = row["note"].strip("; ")
    row["secs"] = int(time.time() - t0)
    return row


COLS = ["scenario", "first_frame", "first_function", "tu", "rust_file", "c_file", "first_field", "kind", "hash_cols", "c_trace_cache", "seq_kind", "early_hash_invisible", "field_key", "pre", "note"]


def write_tsv(rows, path):
    with open(path, "w", newline="") as f:
        f.write("\t".join(COLS) + "\n")
        for r in rows:
            f.write("\t".join(str(r.get(c, "")).replace("\t", " ").replace("\n", " ") for c in COLS) + "\n")


def read_tsv(path):
    L = Path(path).read_text().splitlines()
    h = L[0].split("\t")
    return [dict(zip(h, l.split("\t"))) for l in L[1:] if l]


def clusters(rows):
    md = []
    for title, key in (("first function", "first_function"), ("TU", "tu")):
        g = defaultdict(list)
        for r in rows:
            k = r.get(key)
            if not k:
                k = "(value only) " + r["field_key"] if r.get("field_key") else "(none: no call difference found)"
            g[k].append(r)
        md.append(f"### Clusters by {title}\n")
        md.append(f"| {title} | n | TU / C file | kinds | scenarios (first frame) |\n|---|---|---|---|---|")
        for k, v in sorted(g.items(), key=lambda kv: (-len(kv[1]), kv[0])):
            tus = sorted({(r.get('tu') or '-') + " / " + (r.get('c_file') or '-') for r in v})[:2]
            kinds = ",".join(sorted({r["kind"] for r in v}))
            sc = ", ".join(f"{r['scenario']} ({r['first_frame']})" for r in sorted(v, key=lambda r: int(r['first_frame'] or 0)))
            md.append(f"| `{k}` | {len(v)} | {'; '.join(tus)} | {kinds} | {sc} |")
        md.append("")
    return "\n".join(md)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("names", nargs="*")
    ap.add_argument("--results", help="run_rust.sh --out TSV: triage every FAIL row")
    ap.add_argument("--run-all", action="store_true", help="run run_rust.sh --all first")
    ap.add_argument("-j", type=int, default=2)
    ap.add_argument("--out", default=str(COV / "triage.tsv"))
    ap.add_argument("--md", help="write the cluster tables to this file")
    ap.add_argument("--summary-only", action="store_true", help="only re-cluster the existing --out TSV")
    a = ap.parse_args()
    if a.summary_only:
        rows = read_tsv(a.out)
    else:
        names = list(a.names)
        if a.run_all:
            res = ROOT / "run_rust_all.tsv"
            sh([BASH, str(HERE / "run_rust.sh"), "-j", "3", "--out", str(res), "--all"], timeout=7200)
            a.results = str(res)
        if a.results:
            for l in Path(a.results).read_text().splitlines():
                f = l.split("\t")
                if len(f) > 1 and f[1] == "FAIL":
                    names.append(f[0])
        names = sorted(set(names))
        if not names:
            print("no failing scenarios", file=sys.stderr)
            return 2
        for p in (ROOT / "melee-pc.exe", Path(EXE), Path(CTEXE)):
            if not p.exists():
                print(f"missing {p}", file=sys.stderr)
                return 2
        with ThreadPoolExecutor(a.j) as ex:
            rows = list(ex.map(lambda s: triage_one(s, a), names))
        rows.sort(key=lambda r: r["scenario"])
        write_tsv(rows, a.out)
        for r in rows:
            print(r["scenario"], r["first_frame"], r["first_function"], r["tu"], r["kind"], flush=True)
    md = clusters(rows)
    print(md)
    if a.md:
        Path(a.md).write_text(md + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
