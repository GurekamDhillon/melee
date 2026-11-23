"""The YouGame page's volume slider (`embed volume <gain>`) reaches our mixer live, on top of our own volume.

Runs one hidden embedded match as the app would start it (YOUGAME_STDIO=1, YOUGAME_EMBED=1), refuses every bridge
request, sends the page's volume lines and checks each `gw: AX: mixer gain` log line: gain = MELEE_VOLUME/100 * page.
usage: page_volume.py <melee.exe> <iso>   (the iso path is never printed)
"""
import os, re, subprocess, sys, tempfile, threading, time

exe, iso = sys.argv[1], sys.argv[2]
GAME = 3  # MELEE_VOLUME, the agent-run level
STEPS = ["0.500", "1.000", "0.250", "0.000", "0.750", "1"]
data = tempfile.mkdtemp(prefix="ygvol-")
env = dict(os.environ, MELEE_AUDIO_DUMP=os.path.join(data, "mix.wav"), YOUGAME_STDIO="1", YOUGAME_EMBED="1", YOUGAME_DATA=data, MELEE_INPUT="n",
           MELEE_PAD_IGNORE_ADAPTER="1", MELEE_WINDOW_HIDDEN="1", MELEE_VOLUME=str(GAME),
           MELEE_WINDOW_TITLE="volcache: page volume test", MELEE_SKIP_INTRO="1")
p = subprocess.Popen([exe, "play", "--iso", iso, "--match", "fox/cpu9,marth/cpu9,32", "--items", "OFF", "--max-frames", "2400"],
                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, text=True, bufsize=1)
log, out = [], []
def rd_err():
    for l in p.stderr:
        log.append(l.rstrip("\n"))
def rd_out():
    for l in p.stdout:
        out.append(l.rstrip("\n"))
        m = re.match(r'yg \{"id":(\d+)', l)
        if m:
            try:
                p.stdin.write('yg {"re":%s,"ok":false,"error":"test: no app"}\n' % m.group(1)); p.stdin.flush()
            except OSError:
                pass
threading.Thread(target=rd_err, daemon=True).start()
threading.Thread(target=rd_out, daemon=True).start()
def wait_for(pat, t):
    end = time.time() + t
    while time.time() < end:
        for l in log:
            if re.search(pat, l):
                return l
        time.sleep(0.1)
    return None
ok = True
try:
    if not wait_for(r"heartbeat tick 600 ", 90):
        print("FAIL: audio never started"); print(chr(10).join(log[-25:])); print("out:", out[:5]); ok = False; raise SystemExit
    caps = next((l for l in out if '"caps"' in l), "")
    print("caps:", caps)
    ok &= '"volume"' in caps
    for s in STEPS:
        n = len(log)
        p.stdin.write(f"embed volume {s}\n"); p.stdin.flush()
        want = GAME / 100 * float(s)
        end, got = time.time() + 5, None
        while time.time() < end and got is None:
            for l in log[n:]:
                m = re.search(r"mixer gain ([\d.]+)", l)
                if m: got = float(m.group(1)); break
            time.sleep(0.05)
        good = got is not None and abs(got - want) < 1e-4
        ok &= good
        print(f"page {s:>5}: mixer gain {got} (want {want:.4f}) {'ok' if good else 'FAIL'}"
              + ("  <- silent" if good and want == 0 else ""))
    n = len(log)
    p.stdin.write("embed volume 1.5\n"); p.stdin.flush(); time.sleep(1)
    bad = any("ignored a volume command" in l for l in log[n:]) and not any("page volume" in l for l in log[n:])
    ok &= bad
    print("out-of-range 1.5 ignored:", bad)
    if not bad: print(chr(10).join(l for l in log[n:] if "heartbeat" not in l))
finally:
    try:
        p.stdin.write("embed quit\n"); p.stdin.flush()
    except OSError:
        pass
    try:
        p.wait(timeout=15)
    except subprocess.TimeoutExpired:
        p.kill()
for l in log:
    if "page volume" in l or "mixer gain" in l or "caps" in l:
        print("  log:", l)
print("PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
