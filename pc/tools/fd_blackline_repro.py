#!/usr/bin/env python3
"""Deterministic repro/measure for the Final Destination black-band bug (window-size dependent).

For each client size WxH: launch an idle Fox v Falco match on FD (MELEE_CPU_IDLE, no items,
no time limit) in an off-screen window of that size, take a screenshot at a fixed frame, and
count the near-black pixels. --resize-from WxH starts at that size and resizes to the target
through the SDL path (MELEE_RESIZE_AT) before the shot. No disc data is committed: the ISO comes
from GW_ISO_VANILLA (.env). Needs PIL + numpy.

  python pc/tools/fd_blackline_repro.py --out DIR 1529x931 1529x882 1920x1009 [--resize-from 1529x931]
Env: GW_MELEE, GW_BUILD_ROOT as for tools/port. Prints one line per size: black fraction of the
frame and of the stage band (rows where the platform draws), PASS if the black fraction is within
--tol of the reference (the first size, or --ref-size).
"""
import argparse, os, subprocess, sys, time
import numpy as np
from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))


def measure(path):
    a = np.asarray(Image.open(path).convert("RGB")).astype(int)
    h, w, _ = a.shape
    black = a.max(axis=2) <= 3
    # stage band: the lower 60% of the frame, where the platform's top surface is
    band = black[int(h * 0.40):int(h * 0.95), :]
    return black.mean(), band.mean(), (w, h)


def run(size, resize_from, out, iso, frame, extra_env):
    w, h = map(int, size.split("x"))
    env = dict(os.environ)
    for k in [k for k in env if k.startswith("MELEE_")]:
        del env[k]
    shot = os.path.join(out, "%s%s.png" % (size, "_from_" + resize_from if resize_from else ""))
    if os.path.exists(shot):
        os.remove(shot)
    sw, sh = (map(int, resize_from.split("x")) if resize_from else (w, h))
    env.update({
        "MELEE_SCENE": "mode=vs;p1=fox;p2=falco/cpu0;stage=fd;time=0;items=0",
        "MELEE_CPU_IDLE": "1", "MELEE_VOLUME": "0", "MELEE_INPUT": "none", "MELEE_FPS": "u",
        "MELEE_NO_ONBOARD": "1", "MELEE_MODS_DIR": os.path.join(ROOT, "_build", "nomods"),
        "MELEE_WINDOW_X": "30000", "MELEE_WINDOW_Y": "30000",
        "MELEE_WINDOW_W": str(sw), "MELEE_WINDOW_H": str(sh),
        "MELEE_SHOT_AT": "%d:%s" % (frame, shot.replace("\\", "/")),
    })
    if resize_from:
        env["MELEE_RESIZE_AT"] = "%d:%dx%d" % (frame - 60, w, h)
    env.update(extra_env)
    sandbox = "fdbl-%s-%d" % (size, int(time.time()))
    run_sh = os.path.join(ROOT, "tools", "port", "run.sh").replace("\\", "/")
    log = open(shot + ".log", "wb")
    p = subprocess.Popen([os.environ.get("GW_BASH", "C:/Program Files/Git/usr/bin/bash.exe"), run_sh, sandbox, "--iso", iso], env=env, stdout=log, stderr=subprocess.STDOUT)
    t0 = time.time()
    while time.time() - t0 < 90 and not os.path.exists(shot):
        time.sleep(1)
    time.sleep(2)
    subprocess.run(["taskkill", "/F", "/T", "/PID", str(p.pid)], capture_output=True)
    # the game is a child of run.sh: kill only the run directory's own exe
    subprocess.run(["powershell", "-NoProfile", "-Command",
                    "Get-CimInstance Win32_Process -Filter \"name='melee-pc.exe'\" | Where-Object { $_.ExecutablePath -like '*%s*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }" % sandbox],
                   capture_output=True)
    if not os.path.exists(shot):
        return None
    return shot


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("sizes", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--resize-from", default="")
    ap.add_argument("--frame", type=int, default=1100)
    ap.add_argument("--tol", type=float, default=0.01, help="allowed extra black fraction of the stage band")
    ap.add_argument("--ref-size", default="")
    ap.add_argument("--env", action="append", default=[], help="extra KEY=VAL for the game (repeatable)")
    a = ap.parse_args()
    iso = os.environ.get("GW_ISO_VANILLA")
    if not iso:
        for line in open(os.path.join(ROOT, ".env")):
            if line.startswith("export GW_ISO_VANILLA="):
                iso = line.split("=", 1)[1].strip().strip('"')
    a.out = os.path.abspath(a.out); os.makedirs(a.out, exist_ok=True)
    ref = None
    bad = 0
    for s in ([a.ref_size] if a.ref_size else []) + a.sizes:
        shot = run(s, a.resize_from if s != a.ref_size else "", a.out, iso, a.frame, dict(e.split("=", 1) for e in a.env))
        if shot is None:
            print("%-10s NO SHOT" % s); bad += 1; continue
        f, b, dim = measure(shot)
        if ref is None:
            ref = b
        verdict = "ref" if ref == b and s == (a.ref_size or a.sizes[0]) else ("PASS" if b <= ref + a.tol else "FAIL")
        print("%-10s shot %dx%d  black frame %.4f  band %.4f  %s" % (s, dim[0], dim[1], f, b, verdict))
        bad += verdict == "FAIL"
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
