#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Two game instances through the online menu's Unranked, paired by YouGame's rooms (a local fake) or by Slippi's
matchmaking (the local mock), as Hero's blend does it. Loopback only; test identities; no account, no user.json.

    python -I tools/rust-port/yougame/rooms_pair.py OUTDIR --case yougame|slippi|cancel [--frames N]

--case yougame: A and B both run under a fake YouGame app (YOUGAME_STDIO=1, fake_rooms.App) with a test Slippi
  login (MELEE_YOUGAME_TEST_SLIPPI_LOGIN=1 -> the blend). A queues first, B finds A waiting in YouGame's queue, the
  fake rooms service pairs them: the match must come from YouGame's pool.
--case slippi: A is under the fake app with the test login (the blend); B is a plain Slippi-menu player on the mock.
  A is alone in YouGame's queue, so the blend searches Slippi's pool too and the mock pairs A with B.
--case cancel: A alone, blended; it cancels at the online select (B on the select): both searches end, the queue
  hears `cancel`, nothing is paired.
Both peers walk the menus by pad script to the online select (tools/rust-port/menus/slippi_online*_css.txt), lock in,
and the match windows write per-frame Slippi checksums (MELEE_SLIPPI_NP_OUT) that must be equal.
Every game run: MELEE_INPUT=n, MELEE_PAD_IGNORE_ADAPTER=1, MELEE_WINDOW_HIDDEN=1, MELEE_VOLUME=3, a titled window,
a hard timeout. At most two game processes at once (each menu process hands over to its match process).
"""
import argparse, os, subprocess, sys, time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))
from fake_rooms import Rooms, App  # noqa: E402


def env_file():
    ws = Path(os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace")) / ".env"
    out = {}
    if ws.exists():
        for line in ws.read_text(encoding="utf-8", errors="replace").splitlines():
            line = line.strip()
            if line.startswith("export "):
                line = line[7:].strip()
            if "=" in line and not line.startswith("#"):
                k, v = line.split("=", 1)
                out[k.strip()] = v.strip().strip('"')
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--case", choices=["yougame", "slippi", "cancel"], required=True)
    ap.add_argument("--frames", type=int, default=900)
    ap.add_argument("--rooms-port", type=int, default=43190)
    ap.add_argument("--mm-port", type=int, default=43175)
    ap.add_argument("--exe", default=str(REPO / "build" / "cargo" / os.environ.get("PROFILE", "fast") / "melee.exe"))
    a = ap.parse_args()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    for f in ("a.csv", "b.csv"):
        (out / f).unlink(missing_ok=True)
    logf = open(out / "harness.log", "w")
    rooms = Rooms(a.rooms_port, logf).start()
    mm = subprocess.Popen([sys.executable, "-I", str(REPO / "tools/rust-port/slippi/mock_mm.py"), "--port", str(a.mm_port)],
                          stdout=open(out / "mm.log", "w"), stderr=subprocess.STDOUT)
    time.sleep(1.5)
    base = dict(os.environ)
    base.update({k: v for k, v in env_file().items() if k.startswith("GW_ISO")})
    base.update(MELEE_INPUT="n", MELEE_VOLUME="3", MELEE_WINDOW_HIDDEN="1", MELEE_PAD_IGNORE_ADAPTER="1", MELEE_WINDOW_UNFOCUSED="1",
                MELEE_KEYBOARD="0", MELEE_RETAIL_FIRST="1", MELEE_CARD="0", MELEE_SLIPPI_MM_URL=f"ws://127.0.0.1:{a.mm_port}/v1/queue",
                MELEE_SLIPPI_NP_FRAMES=str(a.frames), MELEE_SLIPPI_NP_MAX_SECONDS="240", MELEE_SLIPPI_NP_PADGEN="7")
    menus = REPO / "tools/rust-port/menus"

    def spawn(name, yougame, script, np_port, code, csv):
        e = dict(base)
        e.update(MELEE_WINDOW_TITLE=f"slippimenu:{name}:rooms-{a.case}", MELEE_PAD_SCRIPT=str(menus / script),
                 MELEE_SLIPPI_NP_PORT=str(np_port), MELEE_SLIPPI_TEST_CODE=code, MELEE_SLIPPI_NP_OUT=str(out / csv))
        if yougame:
            data = out / f"ygdata-{name}"
            data.mkdir(exist_ok=True)
            e.update(YOUGAME_STDIO="1", YOUGAME_DATA=str(data), MELEE_YOUGAME_NO_WATCHDOG="1", MELEE_YOUGAME_TEST_SLIPPI_LOGIN="1")
        p = subprocess.Popen([a.exe, "play", "--skip-intro", "--max-frames", "3000", "--slippi-menus", "on"],
                             cwd=str(REPO / "rust"), env=e, stdin=subprocess.PIPE,
                             stdout=subprocess.PIPE if yougame else open(out / f"{name}-menu.log", "w"),
                             stderr=open(out / f"{name}-menu.err", "w"))
        if yougame:
            App(p, name.lower(), a.rooms_port, logf)
        return p

    procs = []
    if a.case == "cancel":
        procs.append(spawn("A", True, "slippi_online_unranked_css_cancel.txt", a.mm_port + 100, "AAAA#001", "a.csv"))
    else:
        procs.append(spawn("A", True, "slippi_online_unranked_css.txt", a.mm_port + 100, "AAAA#001", "a.csv"))
        time.sleep(1.0 if a.case == "yougame" else 0.2)
        procs.append(spawn("B", a.case == "yougame", "slippi_online_b_unranked_css.txt", a.mm_port + 101, "BBBB#002", "b.csv"))
    for p in procs:
        try:
            p.wait(timeout=240)
        except subprocess.TimeoutExpired:
            p.kill()
    verdict = ""
    if a.case != "cancel":
        # the match windows (relaunched children) write the CSVs; bounded wait
        until = time.time() + 240
        while time.time() < until:
            rows = [len((out / f).read_text().split()) if (out / f).exists() else 0 for f in ("a.csv", "b.csv")]
            if min(rows) > a.frames:
                break
            time.sleep(2)
        time.sleep(3)
        try:
            A = (out / "a.csv").read_text().split()[1:]
            B = (out / "b.csv").read_text().split()[1:]
            d = [x for x, y in zip(A, B) if x != y]
            u = len(set(r.split(",")[1] for r in A))
            verdict = ("CSV-EQUAL" if not d and A else f"CSV-DIFFER {len(d)} rows") + f" {min(len(A), len(B))} common rows, {u} distinct checksums"
        except (OSError, IndexError) as e:
            verdict = f"NO-CSV ({e})"
    else:
        time.sleep(2)
    mm.kill()
    errs = "".join((out / f).read_text(errors="replace") for f in ("A-menu.err", "B-menu.err") if (out / f).exists())
    keys = ("online search", "unranked blend", "yougame:", "online css", "online menu")
    for line in errs.splitlines():
        if any(k in line for k in keys):
            print(line)
    for line in rooms.events:
        print(line)
    print([l for l in (out / "mm.log").read_text(errors="replace").splitlines() if "match" in l][:3])
    print(a.case, verdict)


if __name__ == "__main__":
    main()
