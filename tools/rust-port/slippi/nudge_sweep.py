#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Nudge sweep (docs/rust-port/notes/nudge.md): nudge one fighter field of the engine by a tiny amount at one frame of a
replay, run both engines (clean and nudged) over the same recorded inputs, and compare their Slippi desync checksums
frame by frame with Slippi's own test (melee_core::slippi_online::compare): low halves differ by more than 1 (s16) = Hard
(the game ends), high halves differ = Soft (a warning), else Same. Engine versus engine, never versus the replay.

    python -I nudge_sweep.py --exe MELEE.exe --dir REPLAYDIR --out OUTDIR [--n 20] [--jobs 4] [--window 900]
                             [--fields x,y,percent,kb_vel.y,self_vel.x] [--amounts 1u,16u,0.01]

Headless (MELEE_WINDOW_HIDDEN, MELEE_INPUT=n ...), below-normal priority, no network, no OS input. OUTDIR/rows.csv has one
row per nudged run, replays numbered (no names); per-run checksum files are deleted after scoring.
"""
import argparse
import csv
import os
import re
import struct
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(__file__))
from slp_rollbacks import events  # noqa: E402

NEUTRAL = {14, 15, 16, 17, 20, 21}  # Wait, Walk*, Dash, Run


def post_frames(path):
    """{frame: {port: action}} of the main fighters, and the first frame."""
    d = open(path, "rb").read()
    k = d.find(b"raw[$U#l")
    ln = struct.unpack(">I", d[k + 8:k + 12])[0]
    out = {}
    for c, p in events(d[k + 12:k + 12 + ln]):
        if c == 0x38 and p[5] == 0:
            f = struct.unpack(">i", p[0:4])[0]
            out.setdefault(f, {})[p[4]] = struct.unpack(">H", p[7:9])[0]
    return out


def pick_frames(pf, port):
    fs = sorted(f for f, v in pf.items() if port in v)
    last = fs[-1]
    res = []
    for want, ok in ((600, lambda a: a in NEUTRAL), (last // 2, lambda a: a >= 14), (int(last * 0.8), lambda a: a >= 14)):
        for f in fs:
            if f >= want and ok(pf[f][port]) and all(a >= 14 for a in pf[f].values()):
                res.append(f)
                break
    return fs[0], last, res


def s16(v):
    return v - 0x10000 if v & 0x8000 else v


def read_ck(path):
    d = {}
    for line in open(path, encoding="ascii", errors="replace"):
        p = line.split()
        if len(p) >= 2:
            d[int(p[0])] = int(p[1], 16)
    return d


def classify(clean, nud, from_frame):
    """Per-frame Slippi compare of two checksum maps from `from_frame` on."""
    first_soft = first_hard = None
    maxd = 0
    last_bad = None
    n = 0
    for f in sorted(clean):
        if f < from_frame or f not in nud:
            continue
        n += 1
        a, b = clean[f], nud[f]
        d = abs(s16(a & 0xFFFF) - s16(b & 0xFFFF))
        maxd = max(maxd, d)
        if d > 1:
            st = "H"
            if first_hard is None:
                first_hard = f
        elif (a >> 16) != (b >> 16):
            st = "S"
        else:
            st = "="
        if st != "=":
            last_bad = f
            if first_soft is None:
                first_soft = f
    end = max(f for f in clean if f in nud) if n else None
    healed_at = None
    if first_soft is not None and last_bad != end:
        healed_at = last_bad + 1
    if first_soft is None:
        cls = "invisible"  # never differed in the checksum inside the window
    elif first_hard is not None:
        cls = "hard"
    elif healed_at is not None:
        cls = "healed"
    else:
        cls = "soft"
    return cls, first_soft, first_hard, healed_at, maxd, n


def run(exe, env, replay, tag, nudge, frames, ck):
    e = dict(env)
    e["MELEE_WINDOW_TITLE"] = "nudge:" + tag
    if nudge:
        e["SLPCHECK_NUDGE"] = nudge
    else:
        e.pop("SLPCHECK_NUDGE", None)
    cf = 0x4000 if os.name == "nt" else 0  # BELOW_NORMAL_PRIORITY_CLASS
    r = subprocess.run([exe, "slpcheck", "--slp", replay, "--full", "--frames", str(frames), "--slippi-checksum-out", ck],
                       env=e, capture_output=True, text=True, errors="replace", creationflags=cf, timeout=3600)
    return r.stderr + r.stdout


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", required=True)
    ap.add_argument("--dir", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--n", type=int, default=20)
    ap.add_argument("--jobs", type=int, default=4)
    ap.add_argument("--window", type=int, default=900)
    ap.add_argument("--fields", default="x,y,percent,kb_vel.y,self_vel.x")
    ap.add_argument("--amounts", default="1u,16u,0.01")
    ap.add_argument("--pct-amounts", default="1u,16u,0.1")
    ap.add_argument("--min-size", type=int, default=2_500_000)
    ap.add_argument("--max-size", type=int, default=6_000_000)
    ap.add_argument("--files", help="file with one replay basename per line (default: n evenly spaced by name in the size band)")
    ap.add_argument("--control", action="store_true", help="also run a no-op nudge (0u) per replay at its mid frame")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    env = dict(os.environ, MELEE_INPUT="n", MELEE_PAD_IGNORE_ADAPTER="1", MELEE_WINDOW_HIDDEN="1", MELEE_VOLUME="3", MELEE_WINDOW_UNFOCUSED="1")
    if a.files:
        names = [l.strip() for l in open(a.files) if l.strip()]
    else:
        c = sorted(f for f in os.listdir(a.dir) if f.endswith(".slp") and a.min_size <= os.path.getsize(os.path.join(a.dir, f)) <= a.max_size)
        step = max(1, len(c) // a.n)
        names = c[::step][: a.n]
    amounts = a.amounts.split(",")
    pamounts = a.pct_amounts.split(",")
    fields = a.fields.split(",")
    jobs = []
    meta = []
    for i, nm in enumerate(names):
        path = os.path.join(a.dir, nm)
        pf = post_frames(path)
        ports = sorted({p for v in pf.values() for p in v})
        port = ports[i % len(ports)]
        first, last, fr = pick_frames(pf, port)
        if len(fr) < 3:
            print(f"replay {i}: could not pick 3 frames, skipped", file=sys.stderr)
            continue
        end = max(fr) + a.window
        meta.append((i, path, first, port, fr, end))
    rows = []

    def clean_job(m):
        i, path, first, port, fr, end = m
        ck = os.path.join(a.out, f"r{i}.clean.ck")
        run(a.exe, env, path, f"r{i}-clean", None, end - first + 2, ck)
        return read_ck(ck)

    def nudge_job(m, clean, kind, fr, field, amt):
        i, path, first, port, _frs, end = m
        ck = os.path.join(a.out, f"r{i}.{kind}.{field}.{amt}.{fr}.ck")
        stop = min(fr + a.window, end)
        log = run(a.exe, env, path, f"r{i}-{kind}-{field}-{amt}-f{fr}", f"{fr}:{port}:{field}:{amt}", stop - first + 2, ck)
        mo = re.search(r"NUDGE applied frame (-?\d+) slot \d+ \S+ old=([0-9a-f]+) new=([0-9a-f]+) act=(\d+)", log)
        if not os.path.exists(ck) or not mo:
            return [i, kind, field, amt, fr, port, "error", "", "", "", "", "", "", ""]
        nud = read_ck(ck)
        os.remove(ck)
        old, new = int(mo.group(2), 16), int(mo.group(3), 16)
        zero = int((old & 0x7FFFFFFF) == 0)
        cls, fs, fh, hl, md, n = classify(clean, nud, fr)
        return [i, kind, field, amt, fr, port, cls, fs if fs is not None else "", fh if fh is not None else "",
                hl if hl is not None else "", md, zero, mo.group(4), n]

    with ThreadPoolExecutor(a.jobs) as ex:
        cleans = list(ex.map(clean_job, meta))
        for m, c in zip(meta, cleans):
            print(f"replay {m[0]}: clean ok, {len(c)} frames, port {m[3]}, frames {m[4]}", flush=True)
        futs = []
        for m, c in zip(meta, cleans):
            i, path, first, port, frs, end = m
            for fr, kind in zip(frs, ("early", "mid", "late")):
                for field in fields:
                    for amt in (pamounts if field == "percent" else amounts):
                        futs.append(ex.submit(nudge_job, m, c, kind, fr, field, amt))
                if a.control and kind == "mid":
                    futs.append(ex.submit(nudge_job, m, c, "control", fr, "x", "0u"))
        for k, f in enumerate(futs):
            rows.append(f.result())
            if k % 25 == 0:
                print(f"{k + 1}/{len(futs)}", flush=True)
    for m in meta:
        try:
            os.remove(os.path.join(a.out, f"r{m[0]}.clean.ck"))
        except OSError:
            pass
    with open(os.path.join(a.out, "rows.csv"), "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["replay", "when", "field", "amount", "frame", "port", "class", "first_soft", "first_hard", "healed_at", "max_low_delta", "from_zero", "action", "frames_compared"])
        w.writerows(rows)
    print("done", len(rows), "rows")


if __name__ == "__main__":
    main()
