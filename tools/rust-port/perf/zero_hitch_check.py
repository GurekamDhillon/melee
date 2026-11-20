#!/usr/bin/env python3
"""Zero-hitch gate: read one frame profile (MELEE_PROFILE=1 output) and FAIL if any frame is over budget.

  python zero_hitch_check.py PROFILE.csv [--budget 16.0] [--label NAME] [--skip-ms 0]

Budget rules (docs/rust-port/notes/zero-hitch.md):
  * owner line: no frame with more than 16 ms of game work (its interval minus pacing sleep and display wait) and
    no tick longer than 16 ms (the owner's number is 16, not the 16.7 ms slot);
  * tighter line, reported but not failed on: no present over 1.5x the display interval.
Each offender is printed with the frame's split (file read+parse, GPU pipelines, textures, big allocations, pacing
sleep, wasm mods, ui, draw, unaccounted) and the events that fell inside it (scene, stage, fighter, envoy, file), and is
grouped under one cause: the largest measured component, else "engine step (unsplit)".
--skip-ms drops frames that start before that time (the window-creation frame: the first present is not a game frame
the player can see stutter in; it is still listed, as "startup", and does not fail the gate unless --strict-start).
Contention (notes/profiling.md): a profile with the `contended` column marks an over-budget frame "contended" when
the machine was >= 80% busy and the game thread got under 70% of the frame's work time on the CPU (starved by other
processes). Those are listed apart and do not fail the gate (re-run on a quiet machine); --strict-contended fails on
them too. Older profiles without the column count every offender as clean.
A first frame holding stage/fighter events (a `--match` run's match-start tick) is not exempt: it is listed as
"match start" and counted.
Exit 0 = pass, 1 = clean offenders, 2 = bad input.
"""
import csv, json, os, sys
from collections import defaultdict


def main():
    a = sys.argv[1:]
    if not a:
        print(__doc__); return 2
    path = a[0]
    opt = lambda k, d: a[a.index(k) + 1] if k in a else d
    budget = float(opt("--budget", "16.0"))
    label = opt("--label", os.path.basename(path))
    strict_start = "--strict-start" in a
    strict_cont = "--strict-contended" in a
    jl = path[:-4] + ".jsonl"
    rows = list(csv.DictReader(open(path, newline="")))
    if not rows:
        print(f"{label}: empty profile"); return 2
    events = []
    machine = []
    if os.path.exists(jl):
        for ln in open(jl, encoding="utf-8", errors="replace"):
            try: j = json.loads(ln)
            except Exception: continue
            if j.get("type") == "event": events.append(j)
            elif j.get("type") == "machine": machine.append(j)
    events.sort(key=lambda e: e["t_ms"])
    f = lambda r, k: float(r.get(k) or 0)
    offenders, tight = [], 0
    first_t = f(rows[0], "t_ms")
    for i, r in enumerate(rows):
        iv, tk = f(r, "interval_ms"), f(r, "tick_max_ms")
        exp = f(r, "expected_ms") or 16.667
        if iv > 1.5 * exp and exp < 16.0: tight += 1
        # game work in the frame: the interval minus the pacer's sleep and the wait on the display (never game
        # work, in any mode: a hidden window's timer jitter or vsync can stretch an interval with no work in it)
        work = iv - f(r, "pacing_sleep_ms") - f(r, "display_wait_ms")
        if work <= budget and tk <= budget: continue
        t1 = f(r, "t_ms"); t0 = t1 - iv
        ev = [e for e in events if t0 - 0.5 <= e["t_ms"] <= t1 + 0.5]
        alloc_ms = sum(e.get("ms", 0) for e in ev if e["kind"] == "alloc")
        fighter_ms = sum(e.get("ms", 0) for e in ev if e["kind"] == "fighter")
        parts = {
            "file read+parse": f(r, "file_ms") + f(r, "parse_ms"),
            "GPU pipelines": f(r, "pipeline_ms"),
            "texture upload": f(r, "texture_ms"),
            "big allocation": alloc_ms,
            "fighter create": fighter_ms,
            "pacing sleep": f(r, "pacing_sleep_ms"),
            "wasm mods": f(r, "mods_ms"),
            "ui": f(r, "ui_ms"),
            "draw": f(r, "draw_ms") + f(r, "prepare_ms"),
            "unaccounted (OS/loop)": f(r, "unaccounted_ms"),
        }
        known = sum(v for k, v in parts.items() if k not in ("unaccounted (OS/loop)",))
        parts["engine step (unsplit)"] = max(0.0, f(r, "tick_ms") - parts["file read+parse"] - alloc_ms - fighter_ms - parts["wasm mods"])
        cause = max(parts, key=parts.get)
        ctx = sorted({e["kind"] + ": " + e.get("text", "")[:60] for e in ev if e["kind"] in ("scene", "stage", "envoy", "window")})
        files = [e.get("text", "") for e in ev if e["kind"] == "file"]
        # timed phases (spans of kind note/fighter/stage/envoy with ms >= 0.5): the named split of the engine step
        phases = [f'{e.get("text", "")[:48]} {e.get("ms", 0):.1f}' for e in ev if e["kind"] in ("note", "fighter", "stage", "envoy") and e.get("ms", 0) >= 0.5]
        # the first frame is window creation ("startup", exempt) unless the match starts in it (a `--match` run): then it
        # is the match-start tick and counts like any other frame
        match_start = any(e["kind"] in ("stage", "fighter") for e in ev)
        startup = i == 0 and not match_start
        tag = "startup: " if startup else ("match start: " if match_start else "")
        cont = r.get("contended", "") == "1"
        offenders.append(dict(cont=cont, cpu=r.get("thread_cpu_ms", ""), sys=r.get("sys_cpu_pct", ""), t=t0 / 1000, iv=iv, tk=tk, scene=r.get("scene", ""), tick=r.get("tick", ""), cause=tag + cause,
                              parts=parts, ctx=ctx, files=files, phases=phases, startup=startup))
    real = [o for o in offenders if strict_start or not o["startup"]]
    cont = [o for o in real if o["cont"]]
    clean = [o for o in real if not o["cont"]]
    print(f"== {label}: {len(rows)} frames, budget {budget} ms: {len(real)} over budget ({len(clean)} clean, {len(cont)} contended)"
          + (f" (+{len(offenders) - len(real)} startup)" if len(offenders) != len(real) else "")
          + f"; {tight} presents over 1.5x the display interval")
    by = defaultdict(list)
    for o in offenders: by[o["cause"]].append(o)
    for o in offenders:
        if o["cont"]: o["cause"] = "CONTENDED (machine busy, game thread starved): " + o["cause"]
    by = defaultdict(list)
    for o in offenders: by[o["cause"]].append(o)
    for c, L in sorted(by.items(), key=lambda kv: -sum(o["iv"] for o in kv[1])):
        print(f"  [{c}] {len(L)} frames, worst {max(max(o['iv'], o['tk']) for o in L):.1f} ms")
        for o in L:
            top = ", ".join(f"{k} {v:.1f}" for k, v in sorted(o["parts"].items(), key=lambda kv: -kv[1]) if v >= 0.5)
            fl = (f"; files: {', '.join(o['files'][:4])}" + (f" +{len(o['files']) - 4}" if len(o['files']) > 4 else "")) if o["files"] else ""
            print(f"    {o['t']:7.2f}s tick {o['tick']:>6} {o['scene']:<22} frame {o['iv']:6.1f} tick {o['tk']:6.1f} | {top}{fl}"
                  + (f" | {'; '.join(o['ctx'])}" if o["ctx"] else "")
                  + (f" | game thread CPU {o['cpu']} ms, system {o['sys']}%" if o["cpu"] else ""))
            if o["phases"]: print(f"        phases ms: {'; '.join(o['phases'])}")
    busy = [m for m in machine if float(m.get("cpu_pct", 0) or 0) > 85]
    if busy: print(f"  NOTE: machine over 85% CPU in {len(busy)} samples: the run was not quiet, re-run")
    if cont: print(f"  NOTE: {len(cont)} contended frames (not counted as failures): re-run on a quiet machine to judge them")
    return 1 if (clean or (strict_cont and cont)) else 0


if __name__ == "__main__":
    sys.exit(main())
