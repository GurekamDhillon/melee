#!/usr/bin/env python3
"""Plays the YouGame app's side of the stdio bridge against melee.exe (docs/rust-port/notes/yougame.md).

No network, no YouGame account: spawns the exe with YOUGAME_STDIO=1 and a scratch YOUGAME_DATA, answers its SDK
requests from a fake account (Cloud below), then checks
  1. stdout carries only bridge lines (a `melee version` run under the bridge prints nothing to the bridge);
  2. disc supply as the app does it: yougame-native.json passes the app's spec rules (version 1, known fields and
     placeholders, "disc": true for {disc}, net "internet", no pad mirror, no player box), and the play runs start
     from its args with {disc}/{data}/{build} filled in; a wrong disc (a GALP01 header) exits 4 with a `crash:` line
     that gives the reason and no path;
  3. a play run: the first line is the caps line, without pads/rumble/view/volume/fullscreen;
  4. `yg {"ready":true}` arrives after the first presented frame;
  5. `embed quit` ends the process within 5 s, exit code 0;
  6. saves (sdk.md "Saves from a native build"): `hello` then `saveFileList` first; an account-newer settings.cfg is
     downloaded at start; a local-only audio.cfg is uploaded (base64, sha256 matches); the sync note is written;
  7. a second play run: a burst of writes while it runs is one upload carrying the last write, nothing is uploaded
     while nothing changes; closing stdin ends the process within 5 s;
  8. YOUGAME_DATA got the card / replays folders.
Usage: python tools/rust-port/yougame/bridge_test.py [--exe PATH] [--ready-timeout S]
The disc comes from GW_ISO_VANILLA (or the workspace .env); its path is never printed. Test windows are unfocused,
titled, at volume 3, with a pad script (no OS input).
"""
import argparse, base64, hashlib, json, os, queue, re, subprocess, sys, tempfile, threading, time
os.environ.setdefault("MELEE_WINDOW_HIDDEN", "1")  # agents' game windows are invisible (MELEE_WINDOW_HIDDEN=0 shows them)
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SPEC = Path(__file__).resolve().parent / "yougame-native.json"
FORBIDDEN = ("pads", "rumble", "view", "volume", "fullscreen")
PH = re.compile(r"\{([a-z]+)\}")


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


def check_spec(text):
    """The app's own spec rules (desktop app 0.13 parseLaunchSpec). Returns (spec, error)."""
    try:
        o = json.loads(text)
    except ValueError:
        return None, "not JSON"
    if not isinstance(o, dict):
        return None, "not an object"
    for k in o:
        if k not in ("version", "args", "disc", "embed", "controllers", "net", "memoryMb"):
            return None, "unknown field " + k
    if o.get("version") != 1:
        return None, "version must be 1"
    args = o.get("args", [])
    if not isinstance(args, list) or len(args) > 64:
        return None, "bad args"
    for a in args:
        if not isinstance(a, str) or len(a) > 1024 or any(c in a for c in "\0\r\n"):
            return None, "bad arg"
        for m in PH.finditer(a):
            if m.group(1) not in ("build", "data", "disc"):
                return None, "unknown placeholder " + m.group(0)
        if "{" in PH.sub("", a):
            return None, "stray {"
    if "disc" in o and not isinstance(o["disc"], bool):
        return None, "disc must be true or false"
    if not o.get("disc") and any("{disc}" in a for a in args):
        return None, "{disc} needs disc true"
    if o.get("controllers", "none") not in ("mirror", "none"):
        return None, "bad controllers"
    if o.get("net") not in ("internet", "yougame", "none"):
        return None, "no net key (a spec without one runs offline)"
    mem = o.get("memoryMb", 4096)
    if not isinstance(mem, int) or isinstance(mem, bool) or not 512 <= mem <= 16384:
        return None, "bad memoryMb"
    return o, None


def fill(args, build, data, disc):
    """The app's launchArgs: {build}/{data}/{disc}; a /sub/path after one gets the platform's separator."""
    vals = {"build": build, "data": data, "disc": disc}
    return [re.sub(r"\{([a-z]+)\}([^{]*)", lambda m: vals[m.group(1)] + m.group(2).replace("/", os.sep), a) for a in args]


def iso_now(offset=0.0):
    return time.strftime("%Y-%m-%dT%H:%M:%S.000Z", time.gmtime(time.time() + offset))


class Cloud:
    """A fake account's save files, answering like the app's page (sdk.md "Native builds"); records every request."""

    def __init__(self, cloud=True):
        self.files, self.cloud, self.log, self.lock = {}, cloud, [], threading.Lock()

    def put(self, name, data, updated):
        self.files[name] = {"name": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "tag": None,
                            "updated_at": updated, "data": base64.b64encode(data).decode()}

    def answer(self, msg):
        t = msg.get("type")
        with self.lock:
            self.log.append(msg)
            if t == "hello":
                return {"game": "melee-rust-bridge-test", "player": {"id": "p1", "name": "Player"}, "nameShared": False}
            if t == "saveFileList":
                return {"files": [{k: v for k, v in f.items() if k != "data"} for f in self.files.values()],
                        "used": 0, "quota": 250 << 20, "cloud": self.cloud}
            if t == "saveFileRead":
                f = self.files.get(msg.get("name"))
                return {"file": dict(f) if f else None, "cloud": self.cloud}
            if t == "saveFileWrite":
                data = base64.b64decode(msg["data"], validate=True)
                now = iso_now()
                if self.cloud:
                    self.put(msg["name"], data, now)
                return {"cloud": self.cloud, "updated_at": now, "sha256": hashlib.sha256(data).hexdigest(), "used": 0,
                        "quota": 250 << 20}
            if t == "saveFileDelete":
                self.files.pop(msg.get("name"), None)
                return {"cloud": self.cloud}
            return None

    def types(self):
        with self.lock:
            return [m.get("type") for m in self.log]

    def writes(self, name):
        with self.lock:
            return [m for m in self.log if m.get("type") == "saveFileWrite" and m.get("name") == name]


def spawn(exe, args, data, title, cloud=None, extra_env=None):
    env = dict(os.environ)
    env.update({
        "YOUGAME_STDIO": "1", "YOUGAME_GAME": "melee-rust-bridge-test", "YOUGAME_DATA": str(data),
        "YOUGAME_DESKTOP": "0.0.0-test",
        "MELEE_VOLUME": "3", "MELEE_WINDOW_TITLE": title, "MELEE_WINDOW_UNFOCUSED": "1",
        "MELEE_PAD_SCRIPT": str(REPO / "tools/rust-port/play/retail_full.txt"),
    })
    env.update(extra_env or {})
    for k in ("YOUGAME_EMBED", "MELEE_CARD_PATH", "MELEE_SETTINGS_CFG", "MELEE_SLP_DIR", "MELEE_AUDIO_CFG",
              "MELEE_MODS_DIR", "MELEE_NUCLEUS_DIR", "MELEE_COSMETICS_DIR", "MELEE_BRAWL_DISC"):
        env.pop(k, None)
    cloud = cloud if cloud is not None else Cloud(cloud=False)
    p = subprocess.Popen([exe] + args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env,
                         cwd=str(data))
    lines = queue.Queue()
    wlock = threading.Lock()

    def pump(stream, tag):
        for raw in iter(stream.readline, b""):
            line = raw.decode("utf-8", "replace").rstrip("\r\n")
            if line.startswith("yg {") and '"id"' in line and '"type"' in line:
                try:
                    msg = json.loads(line[3:])
                    res = cloud.answer(msg)
                    reply = {"re": msg["id"], "ok": res is not None}
                    if res is None:
                        reply["error"] = "not handled by the test"
                    else:
                        reply["result"] = res
                    with wlock:
                        p.stdin.write(("yg " + json.dumps(reply) + "\n").encode())
                        p.stdin.flush()
                except (ValueError, KeyError, OSError) as e:
                    lines.put((tag, time.monotonic(), "<bad request: %s>" % e))
                continue
            lines.put((tag, time.monotonic(), line))
        lines.put((tag, time.monotonic(), None))

    threading.Thread(target=pump, args=(p.stdout, "out"), daemon=True).start()
    err = []
    threading.Thread(target=lambda: [err.append(l.decode("utf-8", "replace").rstrip()) for l in iter(p.stderr.readline, b"")],
                     daemon=True).start()
    p.wlock = wlock
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


def send(p, text):
    with p.wlock:
        p.stdin.write(text)
        p.stdin.flush()


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
    last_err = []

    def check(ok, what):
        print(("PASS " if ok else "FAIL ") + what.replace(iso, "<iso>"))
        if not ok:
            fails.append(what)

    with tempfile.TemporaryDirectory(prefix="yg-bridge-") as tmp:
        data = Path(tmp) / "data"
        data.mkdir()
        # 1. stdout is the bridge only
        p, lines, err = spawn(a.exe, ["version"], data, "bridge-test version")
        out = []
        while True:
            _, l = next_out(lines, time.monotonic() + 30)
            if l is None:
                break
            out.append(l)
        p.wait(timeout=30)
        check(bool(out) and all(l.startswith("yg ") for l in out), "stdout carries only bridge lines (%d lines)" % len(out))
        check(any("melee (rust)" in l for l in err), "the version text went to stderr")

        # 2. the spec, and the args the app builds from it
        spec, why = check_spec(SPEC.read_text(encoding="utf-8"))
        check(spec is not None, "yougame-native.json passes the app's spec rules (%s)" % (why or "ok"))
        if spec is None:
            print("BRIDGE FAIL (spec)")
            return 1
        check(spec.get("disc") is True and any("{disc}" in x for x in spec["args"]), "the spec plays on the listing's disc ({disc})")
        check(spec.get("net") == "internet" and spec.get("controllers", "none") == "none" and "embed" not in spec,
              "net internet, no pad mirror, own window")
        build_dir = str(Path(a.exe).resolve().parent)
        play = fill(spec["args"], build_dir, str(data), iso) + ["--skip-intro", "--no-audio"]
        bad = Path(tmp) / "wrong-disc.iso"
        hdr = bytearray(0x8000)
        hdr[:6] = b"GALP01"
        hdr[0x1C:0x20] = bytes([0xC2, 0x33, 0x9F, 0x3D])
        bad.write_bytes(bytes(hdr))
        p, lines, err = spawn(a.exe, fill(spec["args"], build_dir, str(data), str(bad)), data, "bridge-test wrong disc",
                              extra_env={"MELEE_NO_DIALOG": "1"})
        code, dt = wait_exit(p, 30.0)
        time.sleep(0.3)
        crash = [l for l in err if l.startswith("crash: ")]
        check(code == 4, "wrong disc: exit %s (want 4) in %.1f s" % (code, dt))
        cl = crash[-1] if crash else ""
        check("GALP01" in cl and "1.02" in cl and str(bad) not in cl and "\\" not in cl and "/" not in cl,
              "wrong disc: a crash line with the reason and no path: %r" % cl)
        check(not any(str(bad) in l for l in err), "wrong disc: the log never names the file's path")

        # 6 (setup). saves: an account-newer settings.cfg, a local-only audio.cfg
        cloud = Cloud()
        cloud.put("settings.cfg", b"# yougame cloud test\n", iso_now(3600))
        (data / "settings.cfg").write_bytes(b"# local, older\n")
        old = time.time() - 86400
        os.utime(data / "settings.cfg", (old, old))
        (data / "audio.cfg").write_bytes(b"volume = 3\n")

        # 3-5. caps, ready, quit
        p, lines, err = spawn(a.exe, play, data, "bridge-test quit", cloud=cloud)
        t0 = time.monotonic()
        _, caps = next_out(lines, t0 + 30)
        ok = False
        c = None
        try:
            c = json.loads(caps[3:])["caps"] if caps and caps.startswith("yg ") else None
            ok = isinstance(c, list) and "ready" in c and not any(f in c for f in FORBIDDEN)
        except (ValueError, KeyError, TypeError):
            pass
        check(ok, "caps first, no pad caps: %s" % c)
        t_ready, ready = next_out(lines, t0 + a.ready_timeout)
        check(ready == 'yg {"ready":true}', "ready after the first present (%.1f s)" % ((t_ready or t0) - t0))
        send(p, b"embed quit\n")
        code, dt = wait_exit(p, 5.0)
        check(code == 0, "embed quit: exit %s in %.2f s (limit 5 s)" % (code, dt))
        time.sleep(0.3)
        check(any("yougame: ready" in l for l in err), "the log says the first frame was presented")
        check(any("yougame: quit (the app asked)" in l for l in err), "the log names the quit")
        ty = cloud.types()
        check(ty[:2] == ["hello", "saveFileList"], "hello, then saveFileList first: %s" % ty[:4])
        check("saveFileRead" in ty and any('down ["settings.cfg"]' in l for l in err),
              "the account-newer settings.cfg was downloaded at start")
        check(len(cloud.writes("audio.cfg")) >= 1, "the local-only audio.cfg was uploaded")
        check(cloud.files.get("audio.cfg", {}).get("sha256") == hashlib.sha256((data / "audio.cfg").read_bytes()).hexdigest(),
              "the account's audio.cfg matches the local file (base64 + sha256)")
        check((data / "yougame-sync.txt").is_file(), "the sync note was written")
        last_err = err

        # 7. a save event while running, then stdin EOF
        p, lines, err = spawn(a.exe, play, data, "bridge-test eof", cloud=cloud)
        t0 = time.monotonic()
        next_out(lines, t0 + 30)
        next_out(lines, t0 + a.ready_timeout)
        n0 = len(cloud.writes("audio.cfg"))
        for v in (5, 6, 7):
            (data / "audio.cfg").write_bytes(b"volume = %d\n" % v)
            time.sleep(0.2)
        time.sleep(4.5)
        n1 = len(cloud.writes("audio.cfg"))
        check(n1 - n0 == 1, "a burst of writes while running is one upload (%d)" % (n1 - n0))
        check(n1 > 0 and base64.b64decode(cloud.writes("audio.cfg")[-1]["data"]) == b"volume = 7\n", "the upload carries the last write")
        time.sleep(2.5)
        check(len(cloud.writes("audio.cfg")) == n1, "nothing more is uploaded while nothing changes")
        with p.wlock:
            p.stdin.close()
        code, dt = wait_exit(p, 5.0)
        check(code == 0, "stdin EOF: exit %s in %.2f s (limit 5 s)" % (code, dt))
        last_err = err

        # 8. data folder
        check((data / "card").is_dir() and (data / "replays").is_dir(), "YOUGAME_DATA holds card/ and replays/")
        if fails:
            print("--- last stderr lines of the last run ---")
            for l in last_err[-25:]:
                print("  " + l.replace(iso, "<iso>"))
    print("BRIDGE OK" if not fails else "BRIDGE FAIL (%d)" % len(fails))
    return 0 if not fails else 1


if __name__ == "__main__":
    sys.exit(main())
