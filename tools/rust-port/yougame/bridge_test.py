#!/usr/bin/env python3
"""Plays the YouGame app's side of the stdio bridge against melee.exe (docs/rust-port/notes/yougame.md).

No network, no YouGame account: spawns the exe with YOUGAME_STDIO=1 and a scratch YOUGAME_DATA, then checks
  1. stdout carries only bridge lines (a `melee version` run under the bridge prints nothing to the bridge);
  2. a play run: the first line is the caps line, without pads/rumble/view/volume/fullscreen;
  3. `yg {"ready":true}` arrives after the first presented frame;
  4. `embed quit` ends the process within 5 s, exit code 0;
  5. a second play run: closing stdin ends the process within 5 s;
  6. YOUGAME_DATA got the card / replays folders.
Usage: python tools/rust-port/yougame/bridge_test.py [--exe PATH] [--ready-timeout S]
The disc comes from GW_ISO_VANILLA (or the workspace .env); its path is never printed. Test windows are unfocused,
titled, at volume 3, with a pad script (no OS input).
"""
import argparse, json, os, queue, subprocess, sys, tempfile, threading, time
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
FORBIDDEN = ("pads", "rumble", "view", "volume", "fullscreen")


def iso_path():
    v = os.environ.get("GW_ISO_VANILLA")
    if v:
        return v
    env = Path(os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace")) / ".env"
    if env.exists():
        for line in env.read_text(encoding="utf-8", errors="replace").splitlines():
            line = line.strip()
            if line.startswith("export "):
                line = line[7:].strip()
            if line.startswith("GW_ISO_VANILLA="):
                return line.split("=", 1)[1].strip().strip('"')
    return None


def spawn(exe, args, data, title):
    env = dict(os.environ)
    env.update({
        "YOUGAME_STDIO": "1", "YOUGAME_GAME": "melee-rust-bridge-test", "YOUGAME_DATA": str(data),
        "MELEE_VOLUME": "3", "MELEE_WINDOW_TITLE": title, "MELEE_WINDOW_UNFOCUSED": "1",
        "MELEE_PAD_SCRIPT": str(REPO / "tools/rust-port/play/retail_full.txt"),
    })
    for k in ("YOUGAME_EMBED", "MELEE_CARD_PATH", "MELEE_SETTINGS_CFG", "MELEE_SLP_DIR"):
        env.pop(k, None)
    p = subprocess.Popen([exe] + args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env,
                         cwd=str(data))
    lines = queue.Queue()

    def pump(stream, tag):
        for raw in iter(stream.readline, b""):
            lines.put((tag, time.monotonic(), raw.decode("utf-8", "replace").rstrip("\r\n")))
        lines.put((tag, time.monotonic(), None))

    threading.Thread(target=pump, args=(p.stdout, "out"), daemon=True).start()
    err = []
    threading.Thread(target=lambda: [err.append(l.decode("utf-8", "replace").rstrip()) for l in iter(p.stderr.readline, b"")],
                     daemon=True).start()
    return p, lines, err


def next_out(lines, deadline):
    while True:
        left = deadline - time.monotonic()
        if left <= 0:
            return None, None
        try:
            tag, t, line = lines.get(timeout=left)
        except queue.Empty:
            return None, None
        if tag == "out":
            return t, line


def wait_exit(p, secs):
    t0 = time.monotonic()
    try:
        code = p.wait(timeout=secs)
        return code, time.monotonic() - t0
    except subprocess.TimeoutExpired:
        p.kill()
        p.wait()
        return None, time.monotonic() - t0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default=str(REPO / "build/cargo/release/melee.exe"))
    ap.add_argument("--ready-timeout", type=float, default=90.0)
    a = ap.parse_args()
    iso = iso_path()
    if not iso or not Path(iso).exists():
        print("bridge_test: GW_ISO_VANILLA not set or missing", file=sys.stderr)
        return 2
    fails = []

    def check(ok, what):
        print(("PASS " if ok else "FAIL ") + what)
        if not ok:
            fails.append(what)

    with tempfile.TemporaryDirectory(prefix="yg-bridge-") as tmp:
        data = Path(tmp)
        # 1. stdout is the bridge only
        p, lines, err = spawn(a.exe, ["version"], data, "bridge-test version")
        out = []
        while True:
            _, l = next_out(lines, time.monotonic() + 30)
            if l is None:
                break
            out.append(l)
        p.wait(timeout=10)
        check(bool(out) and all(l.startswith("yg ") for l in out), f"stdout carries only bridge lines ({len(out)} lines)")
        check(any("melee (rust)" in l for l in err), "the version text went to stderr")

        play = ["play", "--iso", iso, "--skip-intro", "--no-audio"]
        # 2-4. caps, ready, quit
        p, lines, err = spawn(a.exe, play, data, "bridge-test quit")
        t0 = time.monotonic()
        _, caps = next_out(lines, t0 + 30)
        ok = False
        try:
            c = json.loads(caps[3:])["caps"] if caps and caps.startswith("yg ") else None
            ok = isinstance(c, list) and "ready" in c and not any(f in c for f in FORBIDDEN)
        except (ValueError, KeyError, TypeError):
            c = None
        check(ok, f"caps first, no pad caps: {c}")
        t_ready, ready = next_out(lines, t0 + a.ready_timeout)
        check(ready == 'yg {"ready":true}', f"ready after the first present ({(t_ready or t0) - t0:.1f} s)")
        p.stdin.write(b"embed quit\n")
        p.stdin.flush()
        code, dt = wait_exit(p, 5.0)
        check(code == 0, f"embed quit: exit {code} in {dt:.2f} s (limit 5 s)")
        time.sleep(0.2)  # the stderr reader drains after the exit
        check(any("yougame: ready" in l for l in err), "the log says the first frame was presented")
        check(any("yougame: quit (the app asked)" in l for l in err), "the log names the quit")

        # 5. stdin EOF
        p, lines, err = spawn(a.exe, play, data, "bridge-test eof")
        t0 = time.monotonic()
        next_out(lines, t0 + 30)
        _, ready = next_out(lines, t0 + a.ready_timeout)
        p.stdin.close()
        code, dt = wait_exit(p, 5.0)
        check(code == 0, f"stdin EOF: exit {code} in {dt:.2f} s (limit 5 s)")

        # 6. data folder
        check((data / "card").is_dir() and (data / "replays").is_dir(), "YOUGAME_DATA holds card/ and replays/")
        if fails:
            print("--- last stderr lines of the failing run ---")
            for l in err[-25:]:
                print("  " + l.replace(iso, "<iso>"))
    print("BRIDGE OK" if not fails else f"BRIDGE FAIL ({len(fails)})")
    return 0 if not fails else 1


if __name__ == "__main__":
    sys.exit(main())
