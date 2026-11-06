# SPDX-License-Identifier: MIT
"""Record one CPU seed match on the console (the patched Slippi Dolphin of notes/dolphin-oracle.md, vanilla disc).
   python -I record.py CONFIG_JSON OUT.slp LANE_DIR [--timeout S]
The disc boots straight into the configured VS match (gecko_boot.py) with Slippi's recording codes on; Dolphin
(headless, Null video, no audio, no window, Idle priority, no input of any kind) writes the .slp to LANE_DIR/replays.
When the file is closed (Game End written, UBJSON raw length filled in) this script ends that Dolphin process,
by its own PID: in recording mode nothing else stops the console (it goes on to the results screen).
The disc path comes from GW_ISO_VANILLA in the workspace .env and is never printed."""
import json, os, shutil, subprocess, sys, time

here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import gecko_boot, dol  # noqa: E402

WS = os.environ.get("GW_WORKSPACE", "E:/Projects/Melee Workspace")
DOLPHIN = os.environ.get("DOLPHIN_ORACLE_DIR", os.path.join(WS, "_build", "dolphin-oracle"))
EXE = os.path.join(DOLPHIN, "src", "Binary", "x64", "dolphin-emu-nogui.exe")
NETPLAY_INI = os.path.join(DOLPHIN, "src", "Binary", "x64", "Sys", "GameSettings", "Netplay", "GALE01r2.ini")


def closed_slp(path):
    try:
        with open(path, "rb") as f:
            d = f.read()
    except OSError:
        return False
    return len(d) > 64 and d[11:15] != b"\0\0\0\0" and d.rstrip().endswith(b"}")


def main():
    cfg = json.loads(sys.argv[1]) if sys.argv[1].startswith("{") else json.load(open(sys.argv[1]))
    out, lane = sys.argv[2], os.path.abspath(sys.argv[3])
    timeout = float(sys.argv[sys.argv.index("--timeout") + 1]) if "--timeout" in sys.argv else 600
    user = os.path.join(lane, "user")
    rep = os.path.join(lane, "replays")
    shutil.rmtree(rep, ignore_errors=True)
    os.makedirs(os.path.join(user, "Config"), exist_ok=True)
    os.makedirs(os.path.join(user, "GameSettings"), exist_ok=True)
    os.makedirs(rep, exist_ok=True)
    ini = open(os.path.join(here, "..", "dolphin", "Dolphin.ini"), encoding="utf-8").read()
    ini += f"\n[Slippi]\nSaveReplays = True\nReplayMonthlyFolders = False\nReplayDir = {rep}\nEnableJukebox = False\nEnableSpectator = False\n"
    open(os.path.join(user, "Config", "Dolphin.ini"), "w").write(ini)
    shutil.copy(os.path.join(here, "..", "dolphin", "Logger.ini"), os.path.join(user, "Config", "Logger.ini"))
    open(os.path.join(user, "GameSettings", "GALE01.ini"), "w").write(gecko_boot.game_ini(cfg, NETPLAY_INI))
    env = dict(os.environ)
    for k in list(env):
        if k.startswith("MELEE_DOLPHIN"):
            del env[k]
    err = open(os.path.join(lane, "dolphin.stderr.txt"), "w")
    p = subprocess.Popen([EXE, "-u", user, "-p", "headless", "-v", "Null", "-e", dol.iso_path()],
                         stdout=subprocess.DEVNULL, stderr=err, stdin=subprocess.DEVNULL, env=env,
                         creationflags=0x40 | 0x08000000)  # IDLE_PRIORITY_CLASS | CREATE_NO_WINDOW
    t0, got = time.time(), None
    while time.time() - t0 < timeout and p.poll() is None:
        time.sleep(1)
        for n in os.listdir(rep):
            if n.endswith(".slp") and closed_slp(os.path.join(rep, n)):
                got = os.path.join(rep, n)
        if got:
            break
    if p.poll() is None:
        p.kill()  # our own child, by its handle
        p.wait()
    if not got:
        print(f"RECORD error: no closed .slp after {time.time() - t0:.0f} s (exit {p.returncode})")
        return 1
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    shutil.move(got, out)
    print(f"RECORD ok {os.path.getsize(out)} bytes in {time.time() - t0:.0f} s")
    return 0


if __name__ == "__main__":
    sys.exit(main())
